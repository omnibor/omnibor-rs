use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        embedding_mode::EmbeddingMode,
        error::InputManifestError,
        hash_algorithm::HashAlgorithm,
        hash_provider::HashProvider,
        input_manifest::{InputManifest, InputManifestRelation},
        storage::Storage,
    },
    std::{
        collections::BTreeSet,
        fmt::{Debug, Formatter, Result as FmtResult},
        fs::{File, OpenOptions},
        path::Path,
    },
};

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
        let manifest = self.storage.get_manifest_id_for_artifact(artifact)?;
        self.relations
            .insert(InputManifestRelation::new(artifact, manifest));
        Ok(self)
    }

    /// Finish building the manifest, updating the artifact if embedding is on.
    pub fn finish(&mut self, target: &Path) -> Result<InputManifest<H>, InputManifestError> {
        let builder = ArtifactIdBuilder::with_provider(self.hash_provider);

        // Construct a new input manifest.
        let mut manifest = InputManifest::with_relations(self.relations.iter().cloned());

        // Write the manifest to storage.
        let manifest_aid = self.storage.write_manifest(&manifest)?;

        // Get the ArtifactID of the target, possibly embedding the
        // manifest ArtifactID into the target first.
        let target_aid = match self.mode {
            EmbeddingMode::Embed => {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(target)
                    .map_err(|source| {
                        InputManifestError::FailedTargetArtifactRead(Box::new(source))
                    })?;
                embed_manifest_in_target(target, &mut file, manifest_aid)?;
                builder.identify_file(&mut file)?
            }
            EmbeddingMode::NoEmbed => {
                let mut file = File::open(target).map_err(|source| {
                    InputManifestError::FailedTargetArtifactRead(Box::new(source))
                })?;
                builder.identify_file(&mut file)?
            }
        };

        // Update the manifest in storage with the target ArtifactID.
        self.storage
            .update_target_for_manifest(manifest_aid, target_aid)?;

        // Update the manifest in memory with the target ArtifactID.
        manifest.set_target(Some(target_aid));

        // Clear out the set of relations so you can reuse the builder.
        self.relations.clear();

        Ok(manifest)
    }

    /// Access the underlying storage for the builder.
    pub fn storage(&self) -> &S {
        &self.storage
    }
}

/// Embed the manifest's [`ArtifactId`] into the target file.
fn embed_manifest_in_target<H: HashAlgorithm>(
    path: &Path,
    file: &mut File,
    manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>, InputManifestError> {
    match TargetType::infer(path, file) {
        TargetType::KnownBinaryType(BinaryType::ElfFile) => {
            embed_in_elf_file(path, file, manifest_aid)
        }
        TargetType::KnownTextType(TextType::PrefixComments { prefix }) => {
            embed_in_text_file_with_prefix_comment(path, file, manifest_aid, &prefix)
        }
        TargetType::KnownTextType(TextType::WrappedComments { prefix, suffix }) => {
            embed_in_text_file_with_wrapped_comment(path, file, manifest_aid, &prefix, &suffix)
        }
        TargetType::Unknown => Err(InputManifestError::UnknownEmbeddingTarget),
    }
}

fn embed_in_elf_file<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>, InputManifestError> {
    todo!("embedding mode for ELF files is not yet implemented")
}

fn embed_in_text_file_with_prefix_comment<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
    _prefix: &str,
) -> Result<ArtifactId<H>, InputManifestError> {
    todo!("embedding mode for text files is not yet implemented")
}

fn embed_in_text_file_with_wrapped_comment<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
    _prefix: &str,
    _suffix: &str,
) -> Result<ArtifactId<H>, InputManifestError> {
    todo!("embedding mode for text files is not yet implemented")
}

#[allow(unused)]
#[derive(Debug)]
enum TargetType {
    KnownBinaryType(BinaryType),
    KnownTextType(TextType),
    Unknown,
}

impl TargetType {
    fn infer(_path: &Path, _file: &File) -> Self {
        todo!("inferring target file type is not yet implemented")
    }
}

#[allow(unused)]
#[derive(Debug)]
enum BinaryType {
    ElfFile,
}

#[allow(unused)]
#[derive(Debug)]
enum TextType {
    PrefixComments { prefix: String },
    WrappedComments { prefix: String, suffix: String },
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

    #[cfg(feature = "backend-rustcrypto")]
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
        .finish(&target)
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

    #[cfg(feature = "backend-rustcrypto")]
    #[test]
    fn in_memory_builder_works() {
        use crate::hash_provider::RustCrypto;

        let storage = InMemoryStorage::new(RustCrypto::new());
        basic_builder_test(storage);
    }

    #[cfg(feature = "backend-rustcrypto")]
    #[test]
    fn file_system_builder_works() {
        use crate::hash_provider::RustCrypto;

        let storage_root = pathbuf![env!("CARGO_MANIFEST_DIR"), "test", "fs_storage"];
        let mut storage = FileSystemStorage::new(RustCrypto::new(), &storage_root).unwrap();
        basic_builder_test(&mut storage);
        storage.cleanup().unwrap();
    }
}
