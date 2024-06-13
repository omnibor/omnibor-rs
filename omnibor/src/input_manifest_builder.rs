use crate::embedding::EmbeddingMode;
use crate::embedding_mode::Embed;
use crate::embedding_mode::GetMode;
use crate::embedding_mode::Mode;
use crate::embedding_mode::NoEmbed;
use crate::hashes::SupportedHash;
use crate::storage::FileSystemStorage;
use crate::storage::Storage;
use crate::ArtifactId;
use crate::Error;
use crate::InputManifest;
use crate::Relation;
use crate::RelationKind;
use crate::Result;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fs::File;
use std::fs::OpenOptions;
use std::marker::PhantomData;
use std::path::Path;

/// An [`InputManifest`] builder.
pub struct InputManifestBuilder<H: SupportedHash, M: EmbeddingMode, S: Storage<H>> {
    /// The relations to be written to a new manifest by this transaction.
    relations: Vec<Relation<H>>,

    /// Indicates whether manifests should be embedded in the artifact or not.
    mode: PhantomData<M>,

    /// The storage system used to store manifests.
    storage: S,
}

impl<H: SupportedHash, M: EmbeddingMode> Default for InputManifestBuilder<H, M, FileSystemStorage> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: SupportedHash, S: Storage<H>> Debug for InputManifestBuilder<H, Embed, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("mode", &GetMode::<Embed>::mode())
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H: SupportedHash, S: Storage<H>> Debug for InputManifestBuilder<H, NoEmbed, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifestBuilder")
            .field("mode", &GetMode::<NoEmbed>::mode())
            .field("relations", &self.relations)
            .finish_non_exhaustive()
    }
}

impl<H: SupportedHash, M: EmbeddingMode> InputManifestBuilder<H, M, FileSystemStorage> {
    /// Construct a new [`InputManifestBuilder`] with filesystem storage.
    pub fn new() -> Self {
        Self::with_storage(FileSystemStorage)
    }
}

impl<H: SupportedHash, M: EmbeddingMode, S: Storage<H>> InputManifestBuilder<H, M, S> {
    /// Construct a new [`InputManifestBuilder`] with a specific type of storage.
    pub fn with_storage(storage: S) -> Self {
        Self {
            relations: Vec::new(),
            mode: PhantomData,
            storage,
        }
    }

    /// Add an artifact to the transaction.
    pub fn add_artifact(
        &mut self,
        kind: RelationKind,
        artifact: ArtifactId<H>,
    ) -> Result<&mut Self> {
        let manifest = self.storage.get_manifest_id_for_artifact(artifact);

        self.relations.push(Relation::new(kind, artifact, manifest));

        Ok(self)
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
    ) -> Result<TransactionIds<H>> {
        // Construct a new input manifest.
        let mut manifest = InputManifest::with_relations(&self.relations);

        // Write the manifest to storage.
        let manifest_aid = self.storage.write_manifest(&manifest)?;

        // Get the ArtifactID of the target, possibly embedding the
        // manifest ArtifactID into the target first.
        let target_aid = match embed_mode {
            Mode::Embed => {
                let mut file = OpenOptions::new().read(true).open(target)?;
                embed_manifest_in_target(target, &mut file, manifest_aid)?;
                ArtifactId::id_reader(file)?
            }
            Mode::NoEmbed => {
                let file = File::open(target)?;
                ArtifactId::id_reader(file)?
            }
        };

        // Update the manifest in storage with the target ArtifactID.
        self.storage
            .update_target_for_manifest(manifest_aid, target_aid)?;

        // Update the manifest in memory with the target ArtifactID.
        manifest.set_target(target_aid);

        Ok(TransactionIds {
            target_aid,
            manifest_aid,
            manifest,
        })
    }
}

impl<H: SupportedHash, S: Storage<H>> InputManifestBuilder<H, NoEmbed, S> {
    /// Complete the transaction without updating the target artifact.
    pub fn finish(&mut self, target: &Path) -> Result<TransactionIds<H>> {
        Self::finish_with_optional_embedding(self, target, GetMode::<NoEmbed>::mode())
    }
}

impl<H: SupportedHash, S: Storage<H>> InputManifestBuilder<H, Embed, S> {
    /// Complete the transaction, updating the target artifact.
    pub fn finish_and_embed(&mut self, target: &Path) -> Result<TransactionIds<H>> {
        Self::finish_with_optional_embedding(self, target, GetMode::<Embed>::mode())
    }
}

pub struct TransactionIds<H: SupportedHash> {
    /// The ArtifactId of the target.
    target_aid: ArtifactId<H>,

    /// The ArtifactId of the manifest.
    manifest_aid: ArtifactId<H>,

    /// The manifest.
    manifest: InputManifest<H>,
}

impl<H: SupportedHash> TransactionIds<H> {
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
fn embed_manifest_in_target<H: SupportedHash>(
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

fn embed_in_elf_file<H: SupportedHash>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>> {
    todo!("embedding mode for ELF files is not yet implemented")
}

fn embed_in_text_file_with_prefix_comment<H: SupportedHash>(
    _path: &Path,
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
    _prefix: &str,
) -> Result<ArtifactId<H>> {
    todo!("embedding mode for text files is not yet implemented")
}

fn embed_in_text_file_with_wrapped_comment<H: SupportedHash>(
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
        todo!()
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
