//! Error type.

use {
    hex::FromHexError as HexError,
    std::io::Error as IoError,
    url::{ParseError as UrlError, Url},
};

#[cfg(doc)]
use crate::{artifact_id::ArtifactId, input_manifest::InputManifest};

/// Represents any errors from the `omnibor` crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid scheme in URL '{0}'")]
    InvalidScheme(Url),

    #[error("missing object type in URL '{0}'")]
    MissingObjectType(Url),

    #[error("missing hash algorithm in URL '{0}'")]
    MissingHashAlgorithm(Url),

    #[error("missing hash in URL '{0}'")]
    MissingHash(Url),

    #[error("unknown object type")]
    UnknownObjectType,

    #[error("mismatched object type; expected '{expected}'")]
    MismatchedObjectType { expected: &'static str },

    #[error("mismatched hash algorithm; expected '{expected}'")]
    MismatchedHashAlgorithm { expected: &'static str },

    #[error("unexpected hash length; expected '{expected}', got '{observed}'")]
    UnexpectedHashLength { expected: usize, observed: usize },

    #[error("unexpected read length; expected '{expected}', got '{observed}'")]
    UnexpectedReadLength { expected: usize, observed: usize },

    #[error("invalid hex string")]
    InvalidHex(#[from] HexError),

    #[error(transparent)]
    Url(#[from] UrlError),

    #[error(transparent)]
    Io(#[from] IoError),

    #[error("no storage root found; provide one or set the 'OMNIBOR_DIR' environment variable")]
    NoStorageRoot,

    #[error("unable to access file system storage root '{0}'; please check permissions")]
    CantAccessRoot(String, #[source] IoError),

    #[error("object store '{0}' is not a directory")]
    ObjectStoreNotDir(String),

    #[error("'{0}' is not a valid object store path")]
    InvalidObjectStorePath(String),

    #[error("object store '{0}' is not empty")]
    ObjectStoreDirNotEmpty(String),

    #[error("can't create object store '{0}'")]
    CantCreateObjectStoreDir(String, #[source] IoError),

    #[error("can't write manifest directory '{0}'")]
    CantWriteManifestDir(String, #[source] IoError),

    #[error("can't open target index file '{0}'")]
    CantOpenTargetIndex(String, #[source] IoError),

    #[error("can't open target index temp file for upsert '{0}'")]
    CantOpenTargetIndexTemp(String, #[source] IoError),

    #[error("can't delete target index temp file for upsert '{0}'")]
    CantDeleteTargetIndexTemp(String, #[source] IoError),

    #[error("can't write manifest file '{0}'")]
    CantWriteManifest(String, #[source] IoError),

    #[error("the target index file has been corrupted and can't be parsed")]
    CorruptedTargetIndex,

    #[error("the target index file has been corrupted and can't be parsed")]
    CorruptedTargetIndexIoReason(#[source] IoError),

    #[error("the target index file has been corrupted and can't be parsed")]
    CorruptedTargetIndexOmniBorReason(#[source] Box<Error>),

    #[error("missing manifest_aid or target_aid from target index upsert operation")]
    InvalidTargetIndexUpsert,

    #[error("invalid relation kind in input manifest: '{0}'")]
    InvalidRelationKind(String),

    #[error("input manifest missing header line")]
    ManifestMissingHeader,

    #[error("missing 'gitoid' in manifest header")]
    MissingGitOidInHeader,

    #[error("missing object type 'blob' in manifest header")]
    MissingObjectTypeInHeader,

    #[error("missing object type 'blob' in manifest relation")]
    MissingObjectTypeInRelation,

    #[error("missing one or more header parts")]
    MissingHeaderParts,

    #[error("missing bom indicator in relation")]
    MissingBomIndicatorInRelation,

    #[error("missing one or more relation parts")]
    MissingRelationParts,

    #[error("wrong hash algorithm; expected '{expected}', got '{got}'")]
    WrongHashAlgorithm { expected: &'static str, got: String },

    #[error("missing manifest-for entry in manifest")]
    MissingManifestForRelation,

    #[error("the transaction to make an input manifest was already closed")]
    TransactionClosed,

    #[error("unknown file type for manifest ID embedding")]
    UnknownEmbeddingTarget,

    #[error("failed to read input manifest file")]
    FailedManifestRead(#[source] IoError),
}
