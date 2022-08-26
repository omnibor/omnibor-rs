use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io::Error as IoError;
use url::ParseError as UrlError;

/// A `Result` with `gitoid::Error` as the error type.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// An error arising during `GitOid` construction or use.
#[derive(Debug)]
pub enum Error {
    /// The expected and actual length of the data being read didn't
    /// match, indicating something has likely gone wrong.
    BadLength { expected: usize, actual: usize },
    /// Could not construct a valid URL based on the `GitOid` data.
    Url(UrlError),
    /// Could not perform the IO operations necessary to construct the `GitOid`.
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadLength { expected, actual } => {
                write!(f, "expected length {}, actual length {}", expected, actual)
            }
            Error::Url(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::BadLength { .. } => None,
            Error::Url(e) => Some(e),
            Error::Io(e) => Some(e),
        }
    }
}

impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Url(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}
