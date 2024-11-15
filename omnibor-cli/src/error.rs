//! Error types.

use async_channel::SendError;
use omnibor::Error as OmniborError;
use serde_json::Error as JsonError;
use std::{io::Error as IoError, path::PathBuf, result::Result as StdResult};
use tokio::task::JoinError;

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

    #[error("work channel closed for sending")]
    WorkChannelCloseSend(#[source] SendError<PathBuf>),

    #[error("failed to join worker task")]
    CouldNotJoinWorker(#[source] JoinError),

    #[error("print channel closed")]
    PrintChannelClose,

    #[error("did not find configuration file '{}'", path.display())]
    ConfigNotFound {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("could not read default configuration file '{}'", path.display())]
    ConfigDefaultCouldNotRead {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("could not read configuration file '{}'", path.display())]
    ConfigCouldNotRead {
        path: PathBuf,
        #[source]
        source: IoError,
    },

    #[error("can't read configuration file")]
    CantReadConfig(#[source] JsonError),
}

pub type Result<T> = StdResult<T, Error>;