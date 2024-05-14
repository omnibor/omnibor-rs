#[cfg(doc)]
use crate::ArtifactId;
use gitoid::Error as GitOidError;
use std::io::Error as IoError;
use std::result::Result as StdResult;
use url::ParseError as UrlError;

pub type Result<T> = StdResult<T, Error>;

/// Errors arising from [`ArtifactId`] use.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    #[error("failed to read input manifest file")]
    FailedManifestRead(#[from] IoError),

    #[error(transparent)]
    GitOid(#[from] GitOidError),

    #[error(transparent)]
    Url(#[from] UrlError),
}
