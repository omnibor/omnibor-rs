use {
    hex::FromHexError as HexError,
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        io::{Error as IoError, SeekFrom},
    },
    url::ParseError as UrlError,
};

#[cfg(doc)]
use crate::{artifact_id::ArtifactId, input_manifest::InputManifest};

/// An error arising from Artifact ID operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ArtifactIdError {
    /// Failed to open file for identification.
    #[error("failed to open file for identification '{path}'")]
    FailedToOpenFileForId {
        /// The path of the file we failed to open.
        path: Box<str>,
        /// The underlying IO error.
        #[source]
        source: Box<IoError>,
    },

    /// Failed to asynchronously read the reader.
    #[error("failed to asynchronously read the reader")]
    FailedRead(#[source] Box<IoError>),

    /// Failed to reset reader back to the start.
    #[error("failed to reset reader to '{}'", SeekFromDisplay(.0))]
    FailedSeek(SeekFrom, #[source] Box<IoError>),

    /// Failed to check reader position.
    #[error("failed to check reader position")]
    FailedCheckReaderPos(#[source] Box<IoError>),

    /// Invalid scheme in URL.
    #[error("invalid scheme in URL '{0}'")]
    InvalidScheme(Box<str>),

    /// Missing object type in URL.
    #[error("missing object type in URL '{0}'")]
    MissingObjectType(Box<str>),

    /// Missing hash algorithm in URL.
    #[error("missing hash algorithm in URL '{0}'")]
    MissingHashAlgorithm(Box<str>),

    /// Missing hash in URL.
    #[error("missing hash in URL '{0}'")]
    MissingHash(Box<str>),

    /// Mismatched object type.
    #[error("mismatched object type; expected '{expected}', got '{got}'")]
    MismatchedObjectType {
        /// The expected object type.
        expected: Box<str>,
        /// The received object type.
        got: Box<str>,
    },

    /// Mismatched hash algorithm.
    #[error("mismatched hash algorithm; expected '{expected}', got '{got}'")]
    MismatchedHashAlgorithm {
        /// The expected hash algorithm.
        expected: Box<str>,
        /// The received hash algorithm.
        got: Box<str>,
    },

    /// Invalid hex string.
    #[error("invalid hex string '{0}'")]
    InvalidHex(Box<str>, #[source] Box<HexError>),

    /// URL for Artifact ID is not a valid URL
    #[error("URL for Artifact ID is not a valid URL '{0}'")]
    FailedToParseUrl(Box<str>, #[source] Box<UrlError>),
}

/// Helper struct to implement `Display` from `SeekFrom`.
struct SeekFromDisplay<'s>(&'s SeekFrom);

impl Display for SeekFromDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.0 {
            SeekFrom::Start(pos) => write!(f, "{} bytes from start", pos),
            SeekFrom::End(pos) => write!(f, "{} bytes from end", pos),
            SeekFrom::Current(pos) => write!(f, "{} bytes from current position", pos),
        }
    }
}
