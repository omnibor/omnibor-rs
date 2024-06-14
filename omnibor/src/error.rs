#[cfg(doc)]
use crate::ArtifactId;
#[cfg(doc)]
use crate::InputManifest;
use gitoid::Error as GitOidError;
use std::io::Error as IoError;
use std::result::Result as StdResult;
use url::ParseError as UrlError;

pub type Result<T> = StdResult<T, Error>;

/// Errors arising from [`ArtifactId`] use or [`InputManifest`] interaction.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no storage root found; provide one or set the 'OMNIBOR_DIR' environment variable")]
    NoStorageRoot,

    #[error("produced an invalid path to a manifest in the object store")]
    InvalidObjectStorePath,

    #[error("the target index file has been corrupted and can't be parsed")]
    CorruptedTargetIndex,

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
    FailedManifestRead(#[from] IoError),

    #[error(transparent)]
    GitOid(#[from] GitOidError),

    #[error(transparent)]
    Url(#[from] UrlError),
}
