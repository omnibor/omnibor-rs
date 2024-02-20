#[cfg(doc)]
use crate::ArtifactId;
use gitoid::Error as GitOidError;
use std::error::Error as StdError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

/// Errors arising from [`ArtifactId`] use.
#[derive(Debug)]
pub enum Error {
    /// An error arising from the underlying `gitoid` crate.
    GitOid(GitOidError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::GitOid(inner) => write!(f, "{}", inner),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::GitOid(inner) => Some(inner),
        }
    }
}

impl From<GitOidError> for Error {
    fn from(inner: GitOidError) -> Error {
        Error::GitOid(inner)
    }
}
