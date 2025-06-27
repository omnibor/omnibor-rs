use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        embedding_mode::EmbeddingMode,
        error::{EmbeddingError, InputManifestError},
        hash_algorithm::HashAlgorithm,
        hash_provider::HashProvider,
        input_manifest::{embed_provider::EmbedProvider, InputManifest, InputManifestRelation},
        storage::Storage,
    },
    std::{
        collections::BTreeSet,
        fmt::{Debug, Formatter, Result as FmtResult},
        path::Path,
    },
};

#[cfg(feature = "infer-filetypes")]
use crate::input_manifest::embed::embed_manifest_in_target;

/// A builder for [`InputManifest`]s.
pub struct InputManifestBuilder<H: HashAlgorithm, P: HashProvider<H>, S: Storage<H>> {
    /// The relations to be written to a new manifest by this transaction.
    relations: BTreeSet<InputManifestRelation<H>>,

    /// Indicates whether manifests should be embedded in the artifact or not.
    mode: EmbeddingMode,

    /// The cryptography library providing the hash implementation.
    hash_provider: P,

    /// The storage system used to store manifests.
    storage: S,
}

impl<H: HashAlgorithm, P: HashProvider<H>, S: Storage<H>> Debug for InputManifestBuilder<H, P, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("mode", &self.mode)
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H: HashAlgorithm, P: HashProvider<H>, S: Storage<H>> InputManifestBuilder<H, P, S> {
    /// Construct a new [`InputManifestBuilder`].
    pub fn new(mode: EmbeddingMode, storage: S, hash_provider: P) -> Self {
        Self {
            relations: BTreeSet::new(),
            mode,
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

    #[cfg(feature = "infer-filetypes")]
    /// Finish building the manifest, updating the artifact if embedding is on.
    pub fn finish_with_auto_embedding(
        &mut self,
        target: &Path,
    ) -> Result<Result<InputManifest<H>, EmbeddingError>, InputManifestError> {
        let builder = ArtifactIdBuilder::with_provider(self.hash_provider);

        // Construct a new input manifest.
        let mut manifest = InputManifest::with_relations(self.relations.iter().cloned());

        // Write the manifest to storage.
        let manifest_aid = self.storage.write_manifest(&manifest)?;

        // Try to embed the manifest's Artifact ID in the target if we're in embedding mode.
        if self.mode == EmbeddingMode::Embed {
            if let Err(err) = embed_manifest_in_target(target, manifest_aid)? {
                return Ok(Err(err));
            }
        }

        // Get the Artifact ID of the target.
        let target_aid = builder.identify_path(target)?;

        // Update the manifest in storage with the target ArtifactID.
        self.storage
            .update_target_for_manifest(manifest_aid, target_aid)?;

        // Update the manifest in memory with the target ArtifactID.
        manifest.set_target(Some(target_aid));

        // Clear out the set of relations so you can reuse the builder.
        self.relations.clear();

        Ok(Ok(manifest))
    }

    /// Finish with a custom embedding function.
    pub fn finish_with_custom_embedding<F>(
        &mut self,
        target: &Path,
        custom_embedding: F,
    ) -> Result<Result<InputManifest<H>, EmbeddingError>, InputManifestError>
    where
        F: Fn(
            &Path,
            ArtifactId<H>,
            EmbedProvider,
        ) -> Result<Result<(), EmbeddingError>, InputManifestError>,
    {
        let builder = ArtifactIdBuilder::with_provider(self.hash_provider);

        // Construct a new input manifest.
        let mut manifest = InputManifest::with_relations(self.relations.iter().cloned());

        // Write the manifest to storage.
        let manifest_aid = self.storage.write_manifest(&manifest)?;

        // Try to embed the manifest's Artifact ID in the target if we're in embedding mode.
        if self.mode == EmbeddingMode::Embed {
            if let Err(err) = custom_embedding(target, manifest_aid, EmbedProvider::new())? {
                return Ok(Err(err));
            }
        }

        // Get the Artifact ID of the target.
        let target_aid = builder.identify_path(target)?;

        // Update the manifest in storage with the target ArtifactID.
        self.storage
            .update_target_for_manifest(manifest_aid, target_aid)?;

        // Update the manifest in memory with the target ArtifactID.
        manifest.set_target(Some(target_aid));

        // Clear out the set of relations so you can reuse the builder.
        self.relations.clear();

        Ok(Ok(manifest))
    }

    /// Finish building the manifest without embedding in the target.
    pub fn finish_without_embedding(
        &mut self,
        target: &Path,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let mut builder = InputManifestBuilder {
            relations: self.relations.clone(),
            mode: EmbeddingMode::NoEmbed,
            hash_provider: self.hash_provider,
            storage: &mut self.storage,
        };

        // PANIC SAFETY: Since the embedding node is NoEmbed this will never panic.
        let res = builder
            .finish_with_custom_embedding(target, |_, _, _| Ok(Ok(())))
            .map(|res| res.unwrap());

        self.relations.clear();

        res
    }

    /// Access the underlying storage for the builder.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Whether the builder will embed in the target artifact.
    pub fn will_embed(&self) -> bool {
        self.mode == EmbeddingMode::Embed
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            embedding_mode::EmbeddingMode,
            hash_algorithm::Sha256,
            pathbuf,
            storage::{FileSystemStorage, InMemoryStorage},
        },
    };

    #[cfg(feature = "provider-rustcrypto")]
    /// A basic builder test that creates a single manifest and validates it.
    fn basic_builder_test(storage: impl Storage<Sha256>) {
        use crate::hash_provider::RustCrypto;

        let builder = ArtifactIdBuilder::with_rustcrypto();

        let target = pathbuf![
            env!("CARGO_MANIFEST_DIR"),
            "test",
            "data",
            "hello_world.txt"
        ];

        let first_input_aid = builder.identify_string("test_1");
        let second_input_aid = builder.identify_string("test_2");

        let manifest = InputManifestBuilder::<Sha256, _, _>::new(
            EmbeddingMode::NoEmbed,
            storage,
            RustCrypto::new(),
        )
        .add_relation(first_input_aid)
        .unwrap()
        .add_relation(second_input_aid)
        .unwrap()
        .finish_without_embedding(&target)
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
