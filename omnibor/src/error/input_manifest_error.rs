use {
    crate::error::ArtifactIdError,
    std::error::Error,
    std::fmt::{Display, Formatter, Result as FmtResult},
    std::io::Error as IoError,
};

use crate::error::HashProviderError;
#[cfg(doc)]
use crate::{artifact_id::ArtifactId, input_manifest::InputManifest};

/// An error arising from Input Manifest operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum InputManifestError {
    /// Input manifest missing header line.
    ManifestMissingHeader,

    /// Missing 'gitoid' in manifest header.
    MissingGitOidInHeader,

    /// Missing 'blob' in manifest header.
    MissingObjectTypeInHeader,

    /// Missing one or more header parts.
    MissingHeaderParts,

    /// Missing bom indicator in relation.
    MissingBomIndicatorInRelation,

    /// Missing one or more relation parts.
    MissingRelationParts(Box<str>),

    /// Wrong hash algorithm.
    WrongHashAlgorithm {
        /// The expected hash algorithm.
        expected: Box<str>,
        /// The hash algorithm encountered.
        got: Box<str>,
    },

    /// Failed to read input manifest file.
    FailedManifestRead(Box<IoError>),

    /// Failed to read the target artifact during input manifest creation.
    FailedTargetArtifactRead(Box<IoError>),

    /// An error arising from an Artifact ID problem.
    ArtifactIdError(ArtifactIdError),

    /// No storage root found.
    NoStorageRoot,

    /// Can't access storage root.
    CantAccessRoot(Box<str>, Box<IoError>),

    /// Object store is not a directory.
    ObjectStoreNotDir(Box<str>),

    /// Object store path is not valid.
    InvalidObjectStorePath(Box<str>),

    /// Object store is not empty.
    ObjectStoreDirNotEmpty(Box<str>),

    /// Can't create object store.
    CantCreateObjectStoreDir(Box<str>, Box<IoError>),

    /// Can't write manifest directory.
    CantWriteManifestDir(Box<str>, Box<IoError>),

    /// Can't open target index file.
    CantOpenTargetIndex(Box<str>, Box<IoError>),

    /// Can't create target index file.
    CantCreateTargetIndex(Box<str>, Box<IoError>),

    /// Can't open target index temp file during an upsert.
    CantOpenTargetIndexTemp(Box<str>, Box<IoError>),

    /// Can't write to target index temp file for upsert.
    CantWriteTargetIndexTemp(Box<str>, Box<IoError>),

    /// Can't delete target index temp file during an upsert.
    CantDeleteTargetIndexTemp(Box<str>, Box<IoError>),

    /// Can't replace target index with temp file.
    CantReplaceTargetIndexWithTemp {
        /// The path to the target index temp file.
        temp: Box<str>,
        /// The path to the target index file.
        index: Box<str>,
        /// The underlying IO error.
        source: Box<IoError>,
    },

    /// Can't write manifest file.
    CantWriteManifest(Box<str>, Box<IoError>),

    /// Target index entry is malformed.
    TargetIndexMalformedEntry {
        /// The line of the malformed entry.
        line_no: usize,
    },

    /// Can't read entry of the target index file.
    CantReadTargetIndexLine {
        /// The line of the entry we can't read.
        line_no: usize,
        /// The underlying IO error.
        source: Box<IoError>,
    },

    /// An Artifact ID is missing from target index upsert.
    InvalidTargetIndexUpsert,

    /// Failed to clean up storage root.
    FailedStorageCleanup(Box<str>, Box<IoError>),

    /// Can't find manifest for target Artifact ID.
    CantFindManifestForTarget(Box<str>),

    /// Can't find manifest with Artifact ID.
    CantFindManifestWithId(Box<str>),

    /// Missing target index removal criteria.
    MissingTargetIndexRemoveCriteria,

    /// No manifest found to remove in the target index
    NoManifestFoundToRemove,

    /// Can't remove manifest from storage.
    CantRemoveManifest(Box<IoError>),

    /// Unknown file type for manifest ID embedding.
    UnknownEmbeddingTarget,

    /// Can't embed manifest ID in target.
    CantEmbedInTarget(Box<str>, Box<IoError>),

    /// Unsupported binary format for embedding.
    UnsupportedBinaryFormat(Box<str>),

    /// Format doesn't support embedding.
    FormatDoesntSupportEmbedding(Box<str>),

    /// Unknown embedding support.
    UnknownEmbeddingSupport(Box<str>),

    /// Unknown programming language.
    UnknownProgLang(Box<str>),

    /// Invalid character in Input Manifest.
    InvalidCharInManifest,

    /// Failed to initialize hash providers.
    FailedToInitHashProviders,

    /// Hash provider errors.
    HashProviderError(HashProviderError),
}

impl InputManifestError {
    /// Check if the error is from a failed embedding.
    pub fn is_embedding_error(&self) -> bool {
        matches!(
            self,
            Self::UnknownEmbeddingTarget
                | Self::CantEmbedInTarget(..)
                | Self::UnsupportedBinaryFormat(_)
                | Self::FormatDoesntSupportEmbedding(_)
                | Self::UnknownEmbeddingSupport(_)
                | Self::UnknownProgLang(_)
        )
    }
}

impl Display for InputManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            InputManifestError::ManifestMissingHeader => write!(f, "input manifest missing header line"),
            InputManifestError::MissingGitOidInHeader => write!(f, "missing 'gitoid' in manifest header"),
            InputManifestError::MissingObjectTypeInHeader => write!(f, "missing object type 'blob' in manifest header"),
            InputManifestError::MissingHeaderParts => write!(f, "missing one or more header parts"),
            InputManifestError::MissingBomIndicatorInRelation => write!(f, "missing bom indicator in relation"),
            InputManifestError::MissingRelationParts(s) => write!(f, "missing one or more relation parts in '{s}'"),
            InputManifestError::WrongHashAlgorithm { expected, got } => {
                write!(f, "wrong hash algorithm; expected '{expected}', got '{got}'")
            }
            InputManifestError::FailedManifestRead(_) => write!(f, "failed to read input manifest file"),
            InputManifestError::FailedTargetArtifactRead(_) => write!(f, "failed to read the target artifact during input manifest creation"),
            InputManifestError::ArtifactIdError(source) => write!(f, "{source}"),
            InputManifestError::NoStorageRoot => write!(f, "no storage root found; provide one or set the 'OMNIBOR_DIR' environment variable"),
            InputManifestError::CantAccessRoot(s, ..) => write!(f, "unable to access file system storage root '{s}'; please check permissions"),
            InputManifestError::ObjectStoreNotDir(s) => write!(f, "object store is not a directory; '{s}'"),
            InputManifestError::InvalidObjectStorePath(s) => write!(f, "not a valid object store path; '{s}'"),
            InputManifestError::ObjectStoreDirNotEmpty(s) => write!(f, "object store is not empty; '{s}'"),
            InputManifestError::CantCreateObjectStoreDir(s, ..) => write!(f, "can't create object store '{s}'"),
            InputManifestError::CantWriteManifestDir(s, _) => write!(f, "can't write manifest directory '{s}'"),
            InputManifestError::CantOpenTargetIndex(s, _) => write!(f, "can't open target index file '{s}'"),
            InputManifestError::CantCreateTargetIndex(s, _) => write!(f, "can't create target index file '{s}'"),
            InputManifestError::CantOpenTargetIndexTemp(s, _) => write!(f, "can't open target index temp file for upsert '{s}'"),
            InputManifestError::CantWriteTargetIndexTemp(s, _) => write!(f, "can't write to target index temp file for upsert '{s}'"),
            InputManifestError::CantDeleteTargetIndexTemp(s, _) => write!(f, "can't delete target index temp file for upsert '{s}'"),
            InputManifestError::CantReplaceTargetIndexWithTemp { temp, index, .. } => {
                write!(f, "can't replace target index '{index}' with temp file '{temp}'")
            }
            InputManifestError::CantWriteManifest(s, _) => write!(f, "can't write manifest file '{s}'"),
            InputManifestError::TargetIndexMalformedEntry { line_no } => write!(f, "target index entry '{line_no}' is malformed"),
            InputManifestError::CantReadTargetIndexLine { line_no, .. } => write!(f, "can't read entry '{line_no}' of the target index file"),
            InputManifestError::InvalidTargetIndexUpsert => write!(f, "missing manifest_aid or target_aid from target index upsert operation"),
            InputManifestError::FailedStorageCleanup(s, _) => write!(f, "failed to clean up storage root '{s}'"),
            InputManifestError::CantFindManifestForTarget(s) => write!(f, "can't find manifest for target Artifact ID '{s}'"),
            InputManifestError::CantFindManifestWithId(s) => write!(f, "can't find manifest with Artifact ID '{s}'"),
            InputManifestError::MissingTargetIndexRemoveCriteria => write!(f, "missing target index removal criteria; make sure to set a target or manifest Artifact ID"),
            InputManifestError::NoManifestFoundToRemove => write!(f, "no manifest found to remove in the target index"),
            InputManifestError::CantRemoveManifest(_) => write!(f, "can't remove manifest from storage"),
            InputManifestError::UnknownEmbeddingTarget => {
                write!(f, "unknown file type for manifest ID embedding")
            }
            InputManifestError::CantEmbedInTarget(s, _) => {
                write!(f, "can't embed manifest Artifact ID in target '{s}'")
            }
            InputManifestError::UnsupportedBinaryFormat(s) => {
                write!(f, "unsupported binary format for embedding '{s}'")
            }
            InputManifestError::FormatDoesntSupportEmbedding(s) => {
                write!(f, "format doesn't support embedding '{s}'")
            }
            InputManifestError::UnknownEmbeddingSupport(s) => {
                write!(f, "unknown embedding support for language '{s}'")
            }
            InputManifestError::UnknownProgLang(s) => {
                write!(f, "unknown programming language: '{s}'")
            },
            InputManifestError::InvalidCharInManifest => write!(f, "invalid character in manifest"),
            InputManifestError::FailedToInitHashProviders => write!(f, "failed to initialize hash providers"),
            InputManifestError::HashProviderError(source) => write!(f, "{source}"),
        }
    }
}

impl Error for InputManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InputManifestError::ManifestMissingHeader
            | InputManifestError::MissingGitOidInHeader
            | InputManifestError::MissingObjectTypeInHeader
            | InputManifestError::MissingHeaderParts
            | InputManifestError::MissingBomIndicatorInRelation
            | InputManifestError::MissingRelationParts(_)
            | InputManifestError::WrongHashAlgorithm { .. }
            | InputManifestError::NoStorageRoot
            | InputManifestError::ObjectStoreNotDir(_)
            | InputManifestError::InvalidObjectStorePath(_)
            | InputManifestError::ObjectStoreDirNotEmpty(_)
            | InputManifestError::TargetIndexMalformedEntry { .. }
            | InputManifestError::InvalidTargetIndexUpsert
            | InputManifestError::CantFindManifestForTarget(_)
            | InputManifestError::CantFindManifestWithId(_)
            | InputManifestError::MissingTargetIndexRemoveCriteria
            | InputManifestError::NoManifestFoundToRemove
            | InputManifestError::UnknownEmbeddingTarget
            | InputManifestError::UnsupportedBinaryFormat(_)
            | InputManifestError::FormatDoesntSupportEmbedding(_)
            | InputManifestError::UnknownEmbeddingSupport(_)
            | InputManifestError::UnknownProgLang(_)
            | InputManifestError::InvalidCharInManifest
            | InputManifestError::FailedToInitHashProviders => None,
            InputManifestError::FailedManifestRead(source) => Some(source),
            InputManifestError::FailedTargetArtifactRead(source) => Some(source),
            InputManifestError::ArtifactIdError(source) => Some(source),
            InputManifestError::CantAccessRoot(_, source) => Some(source),
            InputManifestError::CantCreateObjectStoreDir(_, source) => Some(source),
            InputManifestError::CantWriteManifestDir(_, source) => Some(source),
            InputManifestError::CantOpenTargetIndex(_, source) => Some(source),
            InputManifestError::CantCreateTargetIndex(_, source) => Some(source),
            InputManifestError::CantOpenTargetIndexTemp(_, source) => Some(source),
            InputManifestError::CantWriteTargetIndexTemp(_, source) => Some(source),
            InputManifestError::CantDeleteTargetIndexTemp(_, source) => Some(source),
            InputManifestError::CantReplaceTargetIndexWithTemp { source, .. } => Some(source),
            InputManifestError::CantWriteManifest(_, source) => Some(source),
            InputManifestError::CantReadTargetIndexLine { source, .. } => Some(source),
            InputManifestError::FailedStorageCleanup(_, source) => Some(source),
            InputManifestError::CantRemoveManifest(source) => Some(source),
            InputManifestError::CantEmbedInTarget(_, source) => Some(source),
            InputManifestError::HashProviderError(source) => Some(source),
        }
    }
}

impl From<ArtifactIdError> for InputManifestError {
    fn from(value: ArtifactIdError) -> Self {
        InputManifestError::ArtifactIdError(value)
    }
}

impl From<HashProviderError> for InputManifestError {
    fn from(value: HashProviderError) -> Self {
        InputManifestError::HashProviderError(value)
    }
}
