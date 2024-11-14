//! File system helper operations.

use crate::{
    cli::{Format, SelectedHash},
    error::{Error, Result},
    print::{error::ErrorMsg, id_file::IdFileMsg, PrintSender, PrinterCmd},
};
use async_walkdir::{DirEntry as AsyncDirEntry, WalkDir};
use futures_lite::stream::StreamExt as _;
use omnibor::{hashes::Sha256, ArtifactId};
use std::path::Path;
use tokio::fs::File as AsyncFile;
use url::Url;

// Identify, recursively, all the files under a directory.
pub async fn id_directory(
    tx: &PrintSender,
    path: &Path,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    let mut entries = WalkDir::new(path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(source)) => {
                tx.send(PrinterCmd::msg(
                    ErrorMsg::new(Error::WalkDirFailed {
                        path: path.to_path_buf(),
                        source,
                    }),
                    format,
                ))
                .await?
            }
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(path).await?;
                id_file(tx, &mut file, path, format, hash).await?;
            }
        }
    }

    Ok(())
}

/// Identify a single file.
pub async fn id_file(
    tx: &PrintSender,
    file: &mut AsyncFile,
    path: &Path,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    let url = hash_file(hash, file, path).await?;

    tx.send(PrinterCmd::msg(
        IdFileMsg {
            path: path.to_path_buf(),
            id: url.clone(),
        },
        format,
    ))
    .await?;

    Ok(())
}

/// Hash the file and produce a `gitoid`-scheme URL.
pub async fn hash_file(hash: SelectedHash, file: &mut AsyncFile, path: &Path) -> Result<Url> {
    match hash {
        SelectedHash::Sha256 => sha256_id_async_file(file, path).await.map(|id| id.url()),
    }
}

/// Check if the file is for a directory.
pub async fn file_is_dir(file: &AsyncFile, path: &Path) -> Result<bool> {
    file.metadata()
        .await
        .map(|meta| meta.is_dir())
        .map_err(|source| Error::FileFailedMetadata {
            path: path.to_path_buf(),
            source,
        })
}

/// Check if the entry is for a directory.
pub async fn entry_is_dir(entry: &AsyncDirEntry) -> Result<bool> {
    entry
        .file_type()
        .await
        .map(|file_type| file_type.is_dir())
        .map_err(|source| Error::UnknownFileType {
            path: entry.path(),
            source,
        })
}

/// Open an asynchronous file.
pub async fn open_async_file(path: &Path) -> Result<AsyncFile> {
    AsyncFile::open(path)
        .await
        .map_err(|source| Error::FileFailedToOpen {
            path: path.to_path_buf(),
            source,
        })
}

/// Identify a file using a SHA-256 hash.
pub async fn sha256_id_async_file(file: &mut AsyncFile, path: &Path) -> Result<ArtifactId<Sha256>> {
    ArtifactId::id_async_reader(file)
        .await
        .map_err(|source| Error::FileFailedToId {
            path: path.to_path_buf(),
            source,
        })
}
