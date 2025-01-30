use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        error::{Error, Result},
        hash_algorithm::HashAlgorithm,
        hash_provider::HashProvider,
        input_manifest::{
            embedding_mode::{EmbeddingMode, Mode},
            InputManifest, Relation,
        },
        storage::Storage,
    },
    std::{
        collections::BTreeSet,
        fmt::{Debug, Formatter, Result as FmtResult},
        fs::{File, OpenOptions},
        marker::PhantomData,
        path::Path,
    },
};

/// An [`InputManifest`] builder.
pub struct InputManifestBuilder<
    H: HashAlgorithm,
    M: EmbeddingMode,
    S: Storage<H>,
    P: HashProvider<H>,
> {
    /// The relations to be written to a new manifest by this transaction.
    relations: BTreeSet<Relation<H>>,

    /// Indicates whether manifests should be embedded in the artifact or not.
    mode: PhantomData<M>,

    /// The storage system used to store manifests.
    storage: S,

    /// The cryptography library providing the SHA-256 implementation.
    sha256_provider: P,
}

/// Should a manifest be stored after creation?
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ShouldStore {
    /// Yes, store the manifest.
    Yes,
    /// No, do not store the manifest.
    No,
}

impl<H: HashAlgorithm, M: EmbeddingMode, S: Storage<H>, P: HashProvider<H>> Debug
    for InputManifestBuilder<H, M, S, P>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("mode", &M::mode())
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H: HashAlgorithm, M: EmbeddingMode, S: Storage<H>, P: HashProvider<H>>
    InputManifestBuilder<H, M, S, P>
{
    /// Construct a new [`InputManifestBuilder`] with a specific type of storage.
    pub fn new(storage: S, sha256_provider: P) -> Self {
        Self {
            relations: BTreeSet::new(),
            mode: PhantomData,
            storage,
            sha256_provider,
        }
    }

    /// Add a relation to an artifact to the transaction.
    pub fn add_relation(&mut self, artifact: ArtifactId<H>) -> Result<&mut Self> {
        let manifest = self.storage.get_manifest_id_for_artifact(artifact)?;
        self.relations.insert(Relation::new(artifact, manifest));
        Ok(self)
    }

    /// Complete the transaction without updating the target artifact.
    pub fn finish(
        &mut self,
        target: &Path,
        should_store: ShouldStore,
    ) -> Result<LinkedInputManifest<H>> {
        Self::finish_with_optional_embedding(self, target, M::mode(), should_store)
    }

    /// Complete creation of a new [`InputManifest`], possibly embedding in the target.
    ///
    /// This is provided as a helper method which the two public methods call into
    /// because the logic is pretty specific in terms of order-of-operations, and
    /// _nearly_ the same except for the embedding choice.
    fn finish_with_optional_embedding(
        &mut self,
        target: &Path,
        embed_mode: Mode,
        should_store: ShouldStore,
    ) -> Result<LinkedInputManifest<H>> {
        let builder = ArtifactIdBuilder::with_provider(self.sha256_provider);

        // Construct a new input manifest.
        let mut manifest = InputManifest::with_relations(self.relations.iter().cloned());

        let manifest_aid = if should_store == ShouldStore::Yes {
            // Write the manifest to storage.
            self.storage.write_manifest(&manifest)?
        } else {
            // Otherwise, just build it.
            builder.identify_manifest(&manifest)
        };

        // Get the ArtifactID of the target, possibly embedding the
        // manifest ArtifactID into the target first.
        let target_aid = match embed_mode {
            Mode::Embed => {
                let mut file = OpenOptions::new().read(true).write(true).open(target)?;
                embed_manifest_in_target(target, &mut file, manifest_aid)?;
                builder.identify_file(&mut file)?
            }
            Mode::NoEmbed => {
                let mut file = File::open(target)?;
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

        Ok(LinkedInputManifest {
            target_aid,
            manifest_aid,
            manifest,
        })
    }

    /// Get a reference to the underlying storage.
    pub fn storage(&self) -> &S {
        &self.storage
    }
}

/// An [`InputManifest`] with a known target artifact.
pub struct LinkedInputManifest<H: HashAlgorithm> {
    /// The ArtifactId of the target.
    target_aid: ArtifactId<H>,

    /// The ArtifactId of the manifest.
    manifest_aid: ArtifactId<H>,

    /// The manifest.
    manifest: InputManifest<H>,
}

impl<H: HashAlgorithm> Debug for LinkedInputManifest<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("TransactionIds")
            .field("target_aid", &self.target_aid)
            .field("manifest_aid", &self.manifest_aid)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: HashAlgorithm> LinkedInputManifest<H> {
    /// Get the ArtifactId of the file targeted by the transaction.
    pub fn target_aid(&self) -> ArtifactId<H> {
        self.target_aid
    }

    /// Get the ArtifactId of the input manifest produced by the transaction.
    pub fn manifest_aid(&self) -> ArtifactId<H> {
        self.manifest_aid
    }

    /// Get the input manifest produced by the transaction.
    pub fn manifest(&self) -> &InputManifest<H> {
        &self.manifest
    }
}

/// Embed the manifest's [`ArtifactId`] into the target file.
fn embed_manifest_in_target<H: HashAlgorithm>(
    path: &Path,
    file: &mut File,
    manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>> {
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
        TargetType::Unknown => Err(Error::UnknownEmbeddingTarget),
    }
}

fn embed_in_elf_file<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>> {
    todo!("embedding mode for ELF files is not yet implemented")
}

fn embed_in_text_file_with_prefix_comment<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
    _prefix: &str,
) -> Result<ArtifactId<H>> {
    todo!("embedding mode for text files is not yet implemented")
}

fn embed_in_text_file_with_wrapped_comment<H: HashAlgorithm>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
    _prefix: &str,
    _suffix: &str,
) -> Result<ArtifactId<H>> {
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
            hash_algorithm::Sha256,
            input_manifest::embedding_mode::NoEmbed,
            pathbuf,
            storage::{FileSystemStorage, InMemoryStorage},
        },
        std::str::FromStr,
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

        let expected_target_aid = ArtifactId::<Sha256>::from_str(
            "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
        )
        .unwrap();

        let expected_manifest_aid = ArtifactId::<Sha256>::from_str(
            "gitoid:blob:sha256:9a5b2fc9692cd80380660cea055012a7e5e91aa8a2154551cba423c8ba1043c0",
        )
        .unwrap();

        let ids = InputManifestBuilder::<Sha256, NoEmbed, _, _>::new(storage, RustCrypto::new())
            .add_relation(first_input_aid)
            .unwrap()
            .add_relation(second_input_aid)
            .unwrap()
            .finish(&target, ShouldStore::Yes)
            .unwrap();

        // Check the ArtifactIDs of the target and the manifest.
        assert_eq!(ids.target_aid.as_hex(), expected_target_aid.as_hex());
        assert_eq!(ids.manifest_aid.as_hex(), expected_manifest_aid.as_hex());

        // Check the first relation in the manifest.
        let first_relation = &ids.manifest.relations()[0];
        assert_eq!(
            first_relation.artifact().as_hex(),
            second_input_aid.as_hex()
        );

        // Check the second relation in the manifest.
        let second_relation = &ids.manifest.relations()[1];
        assert_eq!(
            second_relation.artifact().as_hex(),
            first_input_aid.as_hex()
        );

        // Make sure we update the target in the manifest.
        assert_eq!(
            ids.manifest.target().map(|target| target.as_hex()),
            Some(ids.target_aid.as_hex())
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
