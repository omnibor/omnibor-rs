use crate::{
    embedding_mode::{Embed, GetMode, NoEmbed},
    storage::FileSystemStorage,
    ArtifactId, EmbeddingMode, Error, InputManifest, Relation, RelationKind, Result, Storage,
    SupportedHash,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    fs::File,
    marker::PhantomData,
};

/// Handles producing IDs and manifests during artifact construction.
pub struct Identifier<M: EmbeddingMode, S: Storage> {
    /// Indicates whether manifests should be embedded in the artifact or not.
    mode: PhantomData<M>,
    /// The storage system used to store manifests.
    storage: S,
}

impl Identifier<Embed, FileSystemStorage> {
    fn default() -> Self {
        Self::new(FileSystemStorage)
    }
}

impl<M: EmbeddingMode> Identifier<M, FileSystemStorage> {
    /// Construct a new [`Identifier`] with default storage.
    pub fn with_default_storage() -> Self {
        Identifier::new(FileSystemStorage)
    }
}

impl<M: EmbeddingMode, S: Storage> Identifier<M, S> {
    /// Create a new [`Identifier`].
    pub fn new(storage: S) -> Self {
        Identifier {
            mode: PhantomData,
            storage,
        }
    }

    /// Start a new transaction to create an `InputManifest`.
    pub fn start_transaction<H: SupportedHash>(&self) -> Transaction<'_, H, M, S> {
        Transaction::start(self)
    }
}

impl<S: Storage> Debug for Identifier<Embed, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Identifier")
            .field("mode", &GetMode::<Embed>::mode())
            .finish_non_exhaustive()
    }
}

impl<S: Storage> Debug for Identifier<NoEmbed, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Identifier")
            .field("mode", &GetMode::<NoEmbed>::mode())
            .finish_non_exhaustive()
    }
}

impl Default for Identifier<Embed, FileSystemStorage> {
    fn default() -> Self {
        Self::default()
    }
}

/// A single transaction to produce a manifest.
pub struct Transaction<'ident, H: SupportedHash, M: EmbeddingMode, S: Storage + 'ident> {
    /// The relations to be written to a new manifest by this transaction.
    relations: Vec<Relation<H>>,

    /// The identifier allowing access to configuration and storage.
    identifier: &'ident Identifier<M, S>,

    /// Indicates whether the transaction was closed prior to being dropped.
    finished: bool,
}

impl<'ident, H: SupportedHash, M: EmbeddingMode, S: Storage + 'ident> Transaction<'ident, H, M, S> {
    /// Start a new transaction.
    fn start(identifier: &'ident Identifier<M, S>) -> Self {
        Transaction {
            relations: Vec::new(),
            identifier,
            finished: false,
        }
    }

    /// Add an artifact to the transaction.
    pub fn add_artifact(
        &mut self,
        kind: RelationKind,
        artifact: ArtifactId<H>,
    ) -> Result<&mut Self> {
        if self.finished {
            return Err(Error::TransactionClosed);
        }

        let manifest = self
            .identifier
            .storage
            .get_manifest_id_for_artifact(artifact);

        self.relations.push(Relation::new(kind, artifact, manifest));

        Ok(self)
    }
}

impl<'ident, H: SupportedHash, S: Storage + 'ident> Transaction<'ident, H, NoEmbed, S> {
    pub fn finish(&mut self) -> Result<()> {
        todo!()
    }
}

impl<'ident, H: SupportedHash, S: Storage + 'ident> Transaction<'ident, H, Embed, S> {
    /// Complete the transaction, updating the artifact itself if in embedding mode.
    pub fn finish(&mut self, target: &mut File) -> Result<TransactionIds<H>> {
        if self.finished {
            return Err(Error::TransactionClosed);
        }

        let manifest = InputManifest::with_relations(&self.relations);
        let manifest_aid = self.identifier.storage.write_manifest(None, &manifest)?;
        let target_aid = update_target(target, manifest_aid)?;

        self.identifier
            .storage
            .update_target_for_manifest(manifest_aid, target_aid)?;

        self.finished = true;

        Ok(TransactionIds {
            target_aid,
            manifest_aid,
            manifest,
        })
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

/// Update the target file to embed the hash.
fn update_target<H: SupportedHash>(
    _file: &mut File,
    _manifest_aid: ArtifactId<H>,
) -> Result<ArtifactId<H>> {
    // Update the file to reflect the input manifest.
    //
    // If it's an ELF file, add an ELF section.
    // If it's a known text file format with support for comments, add a comment.
    // Anything else, error out with an unsupported format for embedding.
    //
    // Return the new ArtifactId of the updated file.

    todo!()
}
