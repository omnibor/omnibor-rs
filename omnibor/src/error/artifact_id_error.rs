use {
    hex::FromHexError as HexError,
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
        io::{Error as IoError, SeekFrom},
    },
};

#[cfg(doc)]
use crate::{artifact_id::ArtifactId, input_manifest::InputManifest};

/// An error arising from Artifact ID operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum ArtifactIdError {
    /// Failed to open file for identification.
    FailedToOpenFileForId {
        /// The path of the file we failed to open.
        path: Box<str>,
        /// The underlying IO error.
        source: Box<IoError>,
    },

    /// Failed to asynchronously read the reader.
    FailedRead(Box<IoError>),

    /// Failed to reset reader back to the start.
    FailedSeek(SeekFrom, Box<IoError>),

    /// Failed to check reader position.
    FailedCheckReaderPos(Box<IoError>),

    /// Missing scheme in URL.
    MissingScheme(Box<str>),

    /// Invalid scheme in URL.
    InvalidScheme(Box<str>),

    /// Missing object type in URL.
    MissingObjectType(Box<str>),

    /// Missing hash algorithm in URL.
    MissingHashAlgorithm(Box<str>),

    /// Missing hash in URL.
    MissingHash(Box<str>),

    /// Mismatched object type.
    MismatchedObjectType {
        /// The expected object type.
        expected: Box<str>,
        /// The received object type.
        got: Box<str>,
    },

    /// Mismatched hash algorithm.
    MismatchedHashAlgorithm {
        /// The expected hash algorithm.
        expected: Box<str>,
        /// The received hash algorithm.
        got: Box<str>,
    },

    /// Invalid hex string.
    InvalidHex(Box<str>, Box<HexError>),
}

impl Display for ArtifactIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ArtifactIdError::FailedToOpenFileForId { path, .. } => {
                write!(f, "failed to open file for identification '{path}'")
            }
            ArtifactIdError::FailedRead(_) => write!(f, "failed to asynchronously read the reader"),
            ArtifactIdError::FailedSeek(seek_from, _) => write!(
                f,
                "failed to reset reader to '{}'",
                SeekFromDisplay(seek_from)
            ),
            ArtifactIdError::FailedCheckReaderPos(_) => {
                write!(f, "failed to check reader position")
            }
            ArtifactIdError::MissingScheme(s) => write!(f, "missing scheme in URL '{s}'"),
            ArtifactIdError::InvalidScheme(s) => write!(f, "invalid scheme in URL '{s}'"),
            ArtifactIdError::MissingObjectType(s) => {
                write!(f, "missing object type in URL '{s}'")
            }
            ArtifactIdError::MissingHashAlgorithm(s) => {
                write!(f, "missing hash algorithm in URL '{s}'")
            }
            ArtifactIdError::MissingHash(s) => write!(f, "missing hash in URL '{s}'"),
            ArtifactIdError::MismatchedObjectType { expected, got } => write!(
                f,
                "mismatched object type; expected '{expected}', got '{got}'",
            ),
            ArtifactIdError::MismatchedHashAlgorithm { expected, got } => write!(
                f,
                "mismatched hash algorithm; expected '{expected}', got '{got}'",
            ),
            ArtifactIdError::InvalidHex(s, _) => write!(f, "invalid hex string '{s}'"),
        }
    }
}

impl Error for ArtifactIdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ArtifactIdError::FailedToOpenFileForId { source, .. } => Some(source),
            ArtifactIdError::FailedRead(source) => Some(source),
            ArtifactIdError::FailedSeek(_, source) => Some(source),
            ArtifactIdError::FailedCheckReaderPos(source) => Some(source),
            ArtifactIdError::InvalidHex(_, source) => Some(source),
            ArtifactIdError::MissingScheme(_)
            | ArtifactIdError::InvalidScheme(_)
            | ArtifactIdError::MissingObjectType(_)
            | ArtifactIdError::MissingHashAlgorithm(_)
            | ArtifactIdError::MissingHash(_)
            | ArtifactIdError::MismatchedObjectType { .. }
            | ArtifactIdError::MismatchedHashAlgorithm { .. } => None,
        }
    }
}

/// Helper struct to implement `Display` from `SeekFrom`.
struct SeekFromDisplay<'s>(&'s SeekFrom);

impl Display for SeekFromDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.0 {
            SeekFrom::Start(pos) => write!(f, "{pos} bytes from start"),
            SeekFrom::End(pos) => write!(f, "{pos} bytes from end"),
            SeekFrom::Current(pos) => write!(f, "{pos} bytes from current position"),
        }
    }
}
