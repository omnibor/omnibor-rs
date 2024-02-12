//! Error arising from `GitOid` construction or use.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::result::Result as StdResult;
use hex::FromHexError as HexError;
use std::error::Error as StdError;
use std::io::Error as IoError;
use url::ParseError as UrlError;
use url::Url;

/// A `Result` with `gitoid::Error` as the error type.
pub(crate) type Result<T> = StdResult<T, Error>;

/// An error arising during `GitOid` construction or use.
#[derive(Debug)]
pub enum Error {
    /// The expected and actual length of the data being read didn't
    /// match, indicating something has likely gone wrong.
    BadLength { expected: usize, actual: usize },
    /// Tried to construct a `GitOid` from a `Url` with a scheme besides `gitoid`.
    InvalidScheme(Url),
    /// Tried to construct a `GitOid` from a `Url` without an `ObjectType` in it.
    MissingObjectType(Url),
    /// Tried to construct a `GitOid` from a `Url` without a `HashAlgorithm` in it.
    MissingHashAlgorithm(Url),
    /// Tried to construct a `GitOid` from a `Url` without a hash in it.
    MissingHash(Url),
    /// Tried to parse an unknown object type.
    UnknownObjectType(String),
    /// The expected object type didn't match the provided type.
    MismatchedObjectType { expected: String, observed: String },
    /// The expected hash algorithm didn't match the provided algorithm.
    MismatchedHashAlgorithm { expected: String, observed: String },
    /// The expected size of a hash for an algorithm didn't match the provided size.
    UnexpectedHashLength { expected: usize, observed: usize },
    /// Tried to parse an invalid hex string.
    InvalidHex(HexError),
    /// Could not construct a valid URL based on the `GitOid` data.
    Url(UrlError),
    /// Could not perform the IO operations necessary to construct the `GitOid`.
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::BadLength { expected, actual } => {
                write!(f, "expected length {}, actual length {}", expected, actual)
            }
            Error::InvalidScheme(url) => write!(f, "invalid scheme in URL '{}'", url.scheme()),
            Error::MissingObjectType(url) => write!(f, "missing object type in URL '{}'", url),
            Error::MissingHashAlgorithm(url) => {
                write!(f, "missing hash algorithm in URL '{}'", url)
            }
            Error::MissingHash(url) => write!(f, "missing hash in URL '{}'", url),
            Error::UnknownObjectType(s) => write!(f, "unknown object type '{}'", s),
            Error::MismatchedObjectType { expected, observed } => write!(
                f,
                "mismatched object type; expected '{}', got '{}'",
                expected, observed
            ),
            Error::MismatchedHashAlgorithm { expected, observed } => write!(
                f,
                "mismatched hash algorithm; expected '{}', got '{}'",
                expected, observed
            ),
            Error::UnexpectedHashLength { expected, observed } => {
                write!(
                    f,
                    "unexpected hash length; expected '{}', got '{}'",
                    expected, observed
                )
            }
            Error::InvalidHex(_) => write!(f, "invalid hex string"),
            Error::Url(e) => write!(f, "{}", e),
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::BadLength { .. }
            | Error::InvalidScheme(_)
            | Error::MissingObjectType(_)
            | Error::MissingHashAlgorithm(_)
            | Error::MissingHash(_)
            | Error::UnknownObjectType(_)
            | Error::MismatchedObjectType { .. }
            | Error::MismatchedHashAlgorithm { .. }
            | Error::UnexpectedHashLength { .. } => None,
            Error::InvalidHex(e) => Some(e),
            Error::Url(e) => Some(e),
            Error::Io(e) => Some(e),
        }
    }
}

impl From<HexError> for Error {
    fn from(e: HexError) -> Error {
        Error::InvalidHex(e)
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
