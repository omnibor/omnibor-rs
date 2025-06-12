use {crate::error::ArtifactIdError, std::io::Error as IoError};

#[cfg(doc)]
use crate::{artifact_id::ArtifactId, input_manifest::InputManifest};

/// An error arising from Input Manifest operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum InputManifestError {
    /// Input manifest missing header line.
    #[error("input manifest missing header line")]
    ManifestMissingHeader,

    /// Missing 'gitoid' in manifest header.
    #[error("missing 'gitoid' in manifest header")]
    MissingGitOidInHeader,

    /// Missing 'blob' in manifest header.
    #[error("missing object type 'blob' in manifest header")]
    MissingObjectTypeInHeader,

    /// Missing one or more header parts.
    #[error("missing one or more header parts")]
    MissingHeaderParts,

    /// Missing bom indicator in relation.
    #[error("missing bom indicator in relation")]
    MissingBomIndicatorInRelation,

    /// Missing one or more relation parts.
    #[error("missing one or more relation parts in '{0}'")]
    MissingRelationParts(Box<str>),

    /// Wrong hash algorithm.
    #[error("wrong hash algorithm; expected '{expected}', got '{got}'")]
    WrongHashAlgorithm {
        /// The expected hash algorithm.
        expected: Box<str>,
        /// The hash algorithm encountered.
        got: Box<str>,
    },

    /// Unknown file type for manifest ID embedding.
    #[error("unknown file type for manifest ID embedding")]
    UnknownEmbeddingTarget,

    /// Failed to read input manifest file.
    #[error("failed to read input manifest file")]
    FailedManifestRead(#[source] Box<IoError>),

    /// Failed to read the target artifact during input manifest creation.
    #[error("failed to read the target artifact during input manifest creation")]
    FailedTargetArtifactRead(#[source] Box<IoError>),

    /// An error arising from an Artifact ID problem.
    #[error(transparent)]
    ArtifactIdError(#[from] ArtifactIdError),

    /// No storage root found.
    #[error("no storage root found; provide one or set the 'OMNIBOR_DIR' environment variable")]
    NoStorageRoot,

    /// Can't access storage root.
    #[error("unable to access file system storage root '{0}'; please check permissions")]
    CantAccessRoot(Box<str>, #[source] Box<IoError>),

    /// Object store is not a directory.
    #[error("object store is not a directory; '{0}'")]
    ObjectStoreNotDir(Box<str>),

    /// Object store path is not valid.
    #[error("not a valid object store path; '{0}'")]
    InvalidObjectStorePath(Box<str>),

    /// Object store is not empty.
    #[error("object store is not empty; '{0}'")]
    ObjectStoreDirNotEmpty(Box<str>),

    /// Can't create object store.
    #[error("can't create object store '{0}'")]
    CantCreateObjectStoreDir(Box<str>, #[source] Box<IoError>),

    /// Can't write manifest directory.
    #[error("can't write manifest directory '{0}'")]
    CantWriteManifestDir(Box<str>, #[source] Box<IoError>),

    /// Can't open target index file.
    #[error("can't open target index file '{0}'")]
    CantOpenTargetIndex(Box<str>, #[source] Box<IoError>),

    /// Can't create target index file.
    #[error("can't create target index file '{0}'")]
    CantCreateTargetIndex(Box<str>, #[source] Box<IoError>),

    /// Can't open target index temp file during an upsert.
    #[error("can't open target index temp file for upsert '{0}'")]
    CantOpenTargetIndexTemp(Box<str>, #[source] Box<IoError>),

    /// Can't write to target index temp file for upsert.
    #[error("can't write to target index temp file for upsert '{0}'")]
    CantWriteTargetIndexTemp(Box<str>, #[source] Box<IoError>),

    /// Can't delete target index temp file during an upsert.
    #[error("can't delete target index temp file for upsert '{0}'")]
    CantDeleteTargetIndexTemp(Box<str>, #[source] Box<IoError>),

    /// Can't replace target index with temp file.
    #[error("can't replace target index '{index}' with temp file '{temp}'")]
    CantReplaceTargetIndexWithTemp {
        /// The path to the target index temp file.
        temp: Box<str>,
        /// The path to the target index file.
        index: Box<str>,
        /// The underlying IO error.
        #[source]
        source: Box<IoError>,
    },

    /// Can't write manifest file.
    #[error("can't write manifest file '{0}'")]
    CantWriteManifest(Box<str>, #[source] Box<IoError>),

    /// Target index entry is malformed.
    #[error("target index entry '{line_no}' is malformed")]
    TargetIndexMalformedEntry {
        /// The line of the malformed entry.
        line_no: usize,
    },

    /// Can't read entry of the target index file.
    #[error("can't read entry '{line_no}' of the target index file")]
    CantReadTargetIndexLine {
        /// The line of the entry we can't read.
        line_no: usize,
        /// The underlying IO error.
        #[source]
        source: Box<IoError>,
    },

    /// An Artifact ID is missing from target index upsert.
    #[error("missing manifest_aid or target_aid from target index upsert operation")]
    InvalidTargetIndexUpsert,

    /// Failed to clean up storage root.
    #[error("failed to clean up storage root '{0}'")]
    FailedStorageCleanup(Box<str>, #[source] Box<IoError>),

    /// Can't find manifest for target Artifact ID.
    #[error("can't find manifest for target Artifact ID '{0}'")]
    CantFindManifestForTarget(Box<str>),

    /// Can't find manifest with Artifact ID.
    #[error("can't find manifest with Artifact ID '{0}'")]
    CantFindManifestWithId(Box<str>),

    /// Missing target index removal criteria.
    #[error(
        "missing target index removal criteria; make sure to set a target or manifest Artifact ID"
    )]
    MissingTargetIndexRemoveCriteria,

    /// No manifest found to remove in the target index
    #[error("no manifest found to remove in the target index")]
    NoManifestFoundToRemove,

    /// Can't remove manifest from storage.
    #[error("can't remove manifest from storage")]
    CantRemoveManifest(#[source] Box<IoError>),
}
