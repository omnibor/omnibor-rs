use {
    crate::{
        artifact_id::ArtifactId,
        embed::{custom_embed_provider::CustomEmbedProvider, Embed},
        error::InputManifestError,
        hash_algorithm::HashAlgorithm,
        input_manifest::{Input, InputManifest},
        storage::{query::Match, Storage},
        Identify,
    },
    std::{
        collections::BTreeSet,
        fmt::{Debug, Formatter, Result as FmtResult},
        path::Path,
    },
    tracing::warn,
};

/// A builder for `InputManifest`s.
pub struct InputManifestBuilder<H, S>
where
    H: HashAlgorithm,
    S: Storage<H>,
{
    /// The relations to be written to a new manifest by this transaction.
    relations: BTreeSet<Input<H>>,

    /// The storage system used to store manifests.
    ///
    /// The builder needs to have access to storage because it both reads from
    /// storage to potentially fill-in artifact IDs for the input manifests of
    /// build inputs and to write out created input manifests to storage at
    /// completion time.
    storage: S,

    /// Indicates whether to continue building without embedding if embedding fails.
    ///
    /// By default we require embedding to succeed, so an embedding failure will
    /// result in a build failing and nothing being written to the store.
    should_continue: bool,
}

impl<H, S> Debug for InputManifestBuilder<H, S>
where
    H: HashAlgorithm,
    S: Storage<H>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H, S> InputManifestBuilder<H, S>
where
    H: HashAlgorithm,
    S: Storage<H>,
{
    /// Construct a new [`InputManifestBuilder`].
    pub fn new(storage: S) -> Self {
        Self {
            relations: BTreeSet::new(),
            storage,
            should_continue: false,
        }
    }

    /// Add a relation to an artifact to the transaction.
    ///
    /// If an Input Manifest for the given `ArtifactId` is found in the storage
    /// this builder is using, then this relation will also include the
    /// `ArtifactId` of that Input Manifest.
    pub fn add_relation<I>(&mut self, target: I) -> Result<&mut Self, InputManifestError>
    where
        I: Identify<H>,
    {
        let artifact = ArtifactId::new(target)?;

        let manifest_aid = self
            .storage
            .get_manifest(Match::Target(artifact))?
            .map(|manifest| {
                // SAFETY: identifying a manifest is infallible.
                ArtifactId::new(&manifest).unwrap()
            });

        self.relations.insert(Input::new(artifact, manifest_aid));

        Ok(self)
    }

    /// Set if the builder should retry a build on failed embed.
    pub fn continue_on_failed_embed(&mut self, should_continue: bool) -> &mut Self {
        self.should_continue = should_continue;
        self
    }

    /// Build the input manifest, possibly embedding in the target artifact.
    pub fn build_for_target(
        &mut self,
        target: impl AsRef<Path>,
        embed: impl Embed<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        fn inner<H2, S2>(
            manifest_builder: &mut InputManifestBuilder<H2, S2>,
            target: &Path,
            embed: impl Embed<H2>,
        ) -> Result<InputManifest<H2>, InputManifestError>
        where
            H2: HashAlgorithm,
            S2: Storage<H2>,
        {
            // Get the Artifact ID of the target.
            let target_aid = if embed.will_embed() {
                Some(ArtifactId::new(target)?)
            } else {
                None
            };

            // Construct a new input manifest.
            let manifest = InputManifest::with_relations(
                manifest_builder.relations.iter().cloned(),
                target_aid,
            );

            // Get the Artifact ID of the manifest.
            let manifest_aid = ArtifactId::new(&manifest)?;

            // Try to embed the manifest's Artifact ID in the target if we're in embedding mode.
            if let Some(Err(err)) = embed.try_embed(target, CustomEmbedProvider::new(manifest_aid))
            {
                if err.is_embedding_error() && manifest_builder.should_continue {
                    warn!("{}", err);
                } else {
                    return Err(err);
                }
            }

            // Write the manifest to storage.
            manifest_builder.storage.write_manifest(&manifest)?;

            // Clear out the set of relations so you can reuse the builder.
            manifest_builder.relations.clear();

            Ok(manifest)
        }

        inner(self, target.as_ref(), embed)
    }

    /// Access the underlying storage for the builder.
    pub fn storage(&self) -> &S {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            hash_algorithm::Sha256,
            pathbuf,
            storage::{FileSystemStorage, InMemoryStorage},
        },
    };

    #[cfg(feature = "provider-rustcrypto")]
    /// A basic builder test that creates a single manifest and validates it.
    fn basic_builder_test(storage: impl Storage<Sha256>) {
        use crate::embed::NoEmbed;

        let target = pathbuf![
            env!("CARGO_MANIFEST_DIR"),
            "test",
            "data",
            "hello_world.txt"
        ];

        let manifest = InputManifestBuilder::<Sha256, _>::new(storage)
            .add_relation(b"test_1")
            .unwrap()
            .add_relation(b"test_2")
            .unwrap()
            .build_for_target(&target, NoEmbed)
            .unwrap();

        // Check the first relation in the manifest.
        let first_relation = &manifest.inputs()[0];
        assert_eq!(
            first_relation.artifact().as_hex(),
            ArtifactId::<Sha256>::new(b"test_2").unwrap().as_hex()
        );

        // Check the second relation in the manifest.
        let second_relation = &manifest.inputs()[1];
        assert_eq!(
            second_relation.artifact().as_hex(),
            ArtifactId::<Sha256>::new(b"test_1").unwrap().as_hex()
        );
    }

    #[cfg(feature = "provider-rustcrypto")]
    #[test]
    fn in_memory_builder_works() {
        let storage = InMemoryStorage::new();
        basic_builder_test(storage);
    }

    #[cfg(feature = "provider-rustcrypto")]
    #[test]
    fn file_system_builder_works() {
        let storage_root = pathbuf![env!("CARGO_MANIFEST_DIR"), "test", "fs_storage"];
        let mut storage = FileSystemStorage::new(&storage_root).unwrap();
        basic_builder_test(&mut storage);
        storage.cleanup().unwrap();
    }
}
