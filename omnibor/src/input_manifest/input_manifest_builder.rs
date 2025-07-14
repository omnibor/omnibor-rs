use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        embed::{embed_provider::EmbedProvider, Embed},
        error::{EmbeddingError, InputManifestError},
        hash_algorithm::HashAlgorithm,
        hash_provider::HashProvider,
        input_manifest::{InputManifest, InputManifestRelation},
        storage::Storage,
    },
    std::{
        collections::BTreeSet,
        fmt::{Debug, Formatter, Result as FmtResult},
        path::Path,
    },
};

/// A builder for [`InputManifest`]s.
pub struct InputManifestBuilder<H, P, S, E>
where
    H: HashAlgorithm,
    P: HashProvider<H>,
    S: Storage<H>,
    E: Embed<H>,
{
    /// The relations to be written to a new manifest by this transaction.
    relations: BTreeSet<InputManifestRelation<H>>,

    /// Indicates whether manifests should be embedded in the artifact or not.
    ///
    /// This must be set during the process of configuring the builder. If not,
    /// then building will fail.
    embed: E,

    /// The cryptography library providing the hash implementation.
    hash_provider: P,

    /// The storage system used to store manifests.
    ///
    /// The builder needs to have access to storage because it both reads from
    /// storage to potentially fill-in artifact IDs for the input manifests of
    /// build inputs and to write out created input manifests to storage at
    /// completion time.
    storage: S,
}

impl<H, P, S, E> Debug for InputManifestBuilder<H, P, S, E>
where
    H: HashAlgorithm,
    P: HashProvider<H>,
    S: Storage<H>,
    E: Embed<H>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H, P, S, E> InputManifestBuilder<H, P, S, E>
where
    H: HashAlgorithm,
    P: HashProvider<H>,
    S: Storage<H>,
    E: Embed<H>,
{
    /// Construct a new [`InputManifestBuilder`].
    pub fn new(storage: S, hash_provider: P, mode: E) -> Self {
        Self {
            relations: BTreeSet::new(),
            embed: mode,
            storage,
            hash_provider,
        }
    }

    /// Add a relation to an artifact to the transaction.
    ///
    /// If an Input Manifest for the given `ArtifactId` is found in the storage
    /// this builder is using, then this relation will also include the
    /// `ArtifactId` of that Input Manifest.
    pub fn add_relation(
        &mut self,
        artifact: ArtifactId<H>,
    ) -> Result<&mut Self, InputManifestError> {
        let manifest_aid = self
            .storage
            .get_manifest_for_target(artifact)?
            .map(|manifest| {
                ArtifactIdBuilder::with_provider(self.hash_provider).identify_manifest(&manifest)
            });

        self.relations
            .insert(InputManifestRelation::new(artifact, manifest_aid));

        Ok(self)
    }

    /// Update embedding mode.
    pub fn set_embed<E2>(self, embed: E2) -> InputManifestBuilder<H, P, S, E2>
    where
        E2: Embed<H>,
    {
        InputManifestBuilder {
            embed,
            relations: self.relations,
            hash_provider: self.hash_provider,
            storage: self.storage,
        }
    }

    /// Build the input manifest, possibly embedding in the target artifact.
    pub fn build(
        &mut self,
        target: impl AsRef<Path>,
    ) -> Result<Result<InputManifest<H>, EmbeddingError>, InputManifestError> {
        fn inner<H2, P2, S2, E2>(
            manifest_builder: &mut InputManifestBuilder<H2, P2, S2, E2>,
            target: &Path,
        ) -> Result<Result<InputManifest<H2>, EmbeddingError>, InputManifestError>
        where
            H2: HashAlgorithm,
            P2: HashProvider<H2>,
            S2: Storage<H2>,
            E2: Embed<H2>,
        {
            let aid_builder = ArtifactIdBuilder::with_provider(manifest_builder.hash_provider);

            // Construct a new input manifest.
            let mut manifest =
                InputManifest::with_relations(manifest_builder.relations.iter().cloned());

            // Write the manifest to storage.
            let manifest_aid = manifest_builder.storage.write_manifest(&manifest)?;

            // Try to embed the manifest's Artifact ID in the target if we're in embedding mode.
            if let Err(err) = manifest_builder
                .embed
                .try_embed(target, EmbedProvider::new(manifest_aid))?
            {
                return Ok(Err(err));
            }

            // Get the Artifact ID of the target.
            let target_aid = aid_builder.identify_path(target)?;

            // Update the manifest in storage with the target ArtifactID.
            manifest_builder
                .storage
                .update_target_for_manifest(manifest_aid, target_aid)?;

            // Update the manifest in memory with the target ArtifactID.
            manifest.set_target(Some(target_aid));

            // Clear out the set of relations so you can reuse the builder.
            manifest_builder.relations.clear();

            Ok(Ok(manifest))
        }

        inner(self, target.as_ref())
    }

    /// Access the underlying storage for the builder.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Check if the builder will try to embed in the target artifact.
    pub fn will_embed(&self) -> bool {
        self.embed.will_embed()
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
        use crate::{embed::NoEmbed, hash_provider::RustCrypto};

        let builder = ArtifactIdBuilder::with_rustcrypto();

        let target = pathbuf![
            env!("CARGO_MANIFEST_DIR"),
            "test",
            "data",
            "hello_world.txt"
        ];

        let first_input_aid = builder.identify_string("test_1");
        let second_input_aid = builder.identify_string("test_2");

        let manifest =
            InputManifestBuilder::<Sha256, _, _, _>::new(storage, RustCrypto::new(), NoEmbed)
                .add_relation(first_input_aid)
                .unwrap()
                .add_relation(second_input_aid)
                .unwrap()
                .build(&target)
                .unwrap()
                .unwrap();

        // Check the first relation in the manifest.
        let first_relation = &manifest.relations()[0];
        assert_eq!(
            first_relation.artifact().as_hex(),
            second_input_aid.as_hex()
        );

        // Check the second relation in the manifest.
        let second_relation = &manifest.relations()[1];
        assert_eq!(
            second_relation.artifact().as_hex(),
            first_input_aid.as_hex()
        );
    }

    #[cfg(feature = "provider-rustcrypto")]
    #[test]
    fn in_memory_builder_works() {
        use crate::hash_provider::RustCrypto;

        let storage = InMemoryStorage::new(RustCrypto::new());
        basic_builder_test(storage);
    }

    #[cfg(feature = "provider-rustcrypto")]
    #[test]
    fn file_system_builder_works() {
        use crate::hash_provider::RustCrypto;

        let storage_root = pathbuf![env!("CARGO_MANIFEST_DIR"), "test", "fs_storage"];
        let mut storage = FileSystemStorage::new(RustCrypto::new(), &storage_root).unwrap();
        basic_builder_test(&mut storage);
        storage.cleanup().unwrap();
    }
}
