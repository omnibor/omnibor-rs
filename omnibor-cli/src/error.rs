//! Error types.

use omnibor::Error as OmniborError;
use std::{io::Error as IoError, path::PathBuf, result::Result as StdResult};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not identify '{0}'")]
    NotIdentifiable(String),

    #[error("could not find root directory")]
    NoRoot,

    #[error("failed to initialize file system storage")]
    StorageInitFailed(#[source] OmniborError),

    #[error("failed to generate Artifact ID")]
    IdFailed(#[source] OmniborError),

    #[error("failed to add relation to Input Manifest")]
    AddRelationFailed(#[source] OmniborError),

    #[error("failed to build Input Manifest")]
    ManifestBuildFailed(#[source] OmniborError),

    #[error("failed to write to stdout")]
    StdoutWriteFailed(#[source] IoError),

    #[error("failed to write to stderr")]
    StderrWriteFailed(#[source] IoError),

    #[error("failed walking under directory '{}'", path.display())]
    WalkDirFailed { path: PathBuf, source: IoError },

    #[error("unable to identify file type for '{}'", path.display())]
    UnknownFileType {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("failed to open file '{}'", path.display())]
    FileFailedToOpen {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("failed to get file metadata '{}'", path.display())]
    FileFailedMetadata {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("failed to make Artifact ID for '{}'", path.display())]
    FileFailedToId {
        path: PathBuf,
        #[source]
        source: OmniborError,
    },

    #[error("print channel closed")]
    PrintChannelClose,
}

pub type Result<T> = StdResult<T, Error>;
