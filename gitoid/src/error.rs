//! Error arising from `GitOid` construction or use.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::result::Result as StdResult;
#[cfg(feature = "hex")]
use hex::FromHexError as HexError;
#[cfg(feature = "std")]
use std::error::Error as StdError;
#[cfg(feature = "std")]
use std::io::Error as IoError;
#[cfg(feature = "url")]
use url::ParseError as UrlError;
#[cfg(feature = "url")]
use url::Url;

/// A `Result` with `gitoid::Error` as the error type.
pub(crate) type Result<T> = StdResult<T, Error>;

/// An error arising during `GitOid` construction or use.
#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "url")]
    /// Tried to construct a `GitOid` from a `Url` with a scheme besides `gitoid`.
    InvalidScheme(Url),

    #[cfg(feature = "url")]
    /// Tried to construct a `GitOid` from a `Url` without an `ObjectType` in it.
    MissingObjectType(Url),

    #[cfg(feature = "url")]
    /// Tried to construct a `GitOid` from a `Url` without a `HashAlgorithm` in it.
    MissingHashAlgorithm(Url),

    #[cfg(feature = "url")]
    /// Tried to construct a `GitOid` from a `Url` without a hash in it.
    MissingHash(Url),

    /// Tried to parse an unknown object type.
    UnknownObjectType,

    /// The expected object type didn't match the provided type.
    MismatchedObjectType { expected: &'static str },

    /// The expected hash algorithm didn't match the provided algorithm.
    MismatchedHashAlgorithm { expected: &'static str },

    /// The expected size of a hash for an algorithm didn't match the provided size.
    UnexpectedHashLength { expected: usize, observed: usize },

    /// The amount of data read didn't match the expected amount of data
    UnexpectedReadLength { expected: usize, observed: usize },

    #[cfg(feature = "hex")]
    /// Tried to parse an invalid hex string.
    InvalidHex(HexError),

    #[cfg(feature = "url")]
    /// Could not construct a valid URL based on the `GitOid` data.
    Url(UrlError),

    #[cfg(feature = "std")]
    /// Could not perform the IO operations necessary to construct the `GitOid`.
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            #[cfg(feature = "url")]
            Error::InvalidScheme(url) => write!(f, "invalid scheme in URL '{}'", url.scheme()),

            #[cfg(feature = "url")]
            Error::MissingObjectType(url) => write!(f, "missing object type in URL '{}'", url),

            #[cfg(feature = "url")]
            Error::MissingHashAlgorithm(url) => {
                write!(f, "missing hash algorithm in URL '{}'", url)
            }

            #[cfg(feature = "url")]
            Error::MissingHash(url) => write!(f, "missing hash in URL '{}'", url),

            Error::UnknownObjectType => write!(f, "unknown object type"),

            Error::MismatchedObjectType { expected } => {
                write!(f, "mismatched object type; expected '{}'", expected,)
            }

            Error::MismatchedHashAlgorithm { expected } => {
                write!(f, "mismatched hash algorithm; expected '{}'", expected)
            }

            Error::UnexpectedHashLength { expected, observed } => {
                write!(
                    f,
                    "unexpected hash length; expected '{}', got '{}'",
                    expected, observed
                )
            }

            Error::UnexpectedReadLength { expected, observed } => {
                write!(
                    f,
                    "unexpected read length; expected '{}', got '{}'",
                    expected, observed
                )
            }

            #[cfg(feature = "hex")]
            Error::InvalidHex(_) => write!(f, "invalid hex string"),

            #[cfg(feature = "url")]
            Error::Url(e) => write!(f, "{}", e),

            #[cfg(feature = "std")]
            Error::Io(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "url")]
            Error::InvalidScheme(_) => None,

            #[cfg(feature = "url")]
            Error::MissingObjectType(_) => None,

            #[cfg(feature = "url")]
            Error::MissingHashAlgorithm(_) => None,

            #[cfg(feature = "url")]
            Error::MissingHash(_) => None,

            Error::UnknownObjectType
            | Error::MismatchedObjectType { .. }
            | Error::MismatchedHashAlgorithm { .. }
            | Error::UnexpectedHashLength { .. }
            | Error::UnexpectedReadLength { .. } => None,

            #[cfg(feature = "hex")]
            Error::InvalidHex(e) => Some(e),

            #[cfg(feature = "url")]
            Error::Url(e) => Some(e),

            #[cfg(feature = "std")]
            Error::Io(e) => Some(e),
        }
    }
}

#[cfg(feature = "hex")]
impl From<HexError> for Error {
    fn from(e: HexError) -> Error {
        Error::InvalidHex(e)
    }
}

#[cfg(feature = "url")]
impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Url(e)
    }
}

#[cfg(feature = "std")]
impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}
