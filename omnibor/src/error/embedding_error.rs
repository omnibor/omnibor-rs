use std::io::Error as IoError;

/// Errors arising from a failed embedding.
///
/// These are distinguished from InputManifestError because they are all
/// recoverable and ought to usually be handled and retried without embedding.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EmbeddingError {
    /// Unknown file type for manifest ID embedding.
    #[error("unknown file type for manifest ID embedding")]
    UnknownEmbeddingTarget,

    /// Can't embed manifest ID in target.
    #[error("can't embed manifest Artifact ID in target '{0}'")]
    CantEmbedInTarget(Box<str>, #[source] Box<IoError>),

    /// Unsupported binary format for embedding.
    #[error("unsupported binary format for embedding '{}'", 0)]
    UnsupportedBinaryFormat(Box<str>),

    /// Format doesn't support embedding.
    #[error("format doesn't support embedding '{}'", 0)]
    FormatDoesntSupportEmbedding(Box<str>),

    /// Unknown embedding support.
    #[error("unknown embedding support for language '{0}'. Consider opening a Pull Request on the OmniBOR Rust repo to fix it!")]
    UnknownEmbeddingSupport(Box<str>),
}
