use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;

/// Errors arising from a failed embedding.
///
/// These are distinguished from InputManifestError because they are all
/// recoverable and ought to usually be handled and retried without embedding.
#[derive(Debug)]
#[non_exhaustive]
pub enum EmbeddingError {
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
}

impl Display for EmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            EmbeddingError::UnknownEmbeddingTarget => {
                write!(f, "unknown file type for manifest ID embedding")
            }
            EmbeddingError::CantEmbedInTarget(s, _) => {
                write!(f, "can't embed manifest Artifact ID in target '{}'", s)
            }
            EmbeddingError::UnsupportedBinaryFormat(s) => {
                write!(f, "unsupported binary format for embedding '{}'", s)
            }
            EmbeddingError::FormatDoesntSupportEmbedding(s) => {
                write!(f, "format doesn't support embedding '{}'", s)
            }
            EmbeddingError::UnknownEmbeddingSupport(s) => {
                write!(f, "unknown embedding support for language '{}'", s)
            }
            EmbeddingError::UnknownProgLang(s) => {
                write!(f, "unknown programming language: '{}'", s)
            }
        }
    }
}

impl Error for EmbeddingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EmbeddingError::CantEmbedInTarget(_, source) => Some(source),
            EmbeddingError::UnknownEmbeddingTarget
            | EmbeddingError::UnsupportedBinaryFormat(_)
            | EmbeddingError::FormatDoesntSupportEmbedding(_)
            | EmbeddingError::UnknownEmbeddingSupport(_)
            | EmbeddingError::UnknownProgLang(_) => None,
        }
    }
}
