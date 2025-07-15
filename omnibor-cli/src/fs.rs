//! File system helper operations.

use crate::{
    app::App,
    cli::{Format, SelectedHash},
    error::{Error, Result},
    print::{msg::error::ErrorMsg, msg::id_file::IdFileMsg, PrintSender, PrinterCmd},
};
use async_channel::{bounded, Receiver, Sender as WorkSender};
use async_walkdir::{DirEntry as AsyncDirEntry, WalkDir};
use futures_util::{pin_mut, StreamExt};
use omnibor::{hash_algorithm::Sha256, ArtifactId, ArtifactIdBuilder};
use std::path::{Path, PathBuf};
use tokio::{fs::File as AsyncFile, task::JoinSet};
use tracing::debug;

// Identify, recursively, all the files under a directory.
pub async fn id_directory(
    app: &App,
    hash: SelectedHash,
    tx: &PrintSender,
    path: &Path,
) -> Result<()> {
    let (sender, receiver) = bounded(app.config.perf.work_queue_size());

    tokio::spawn(walk_target(
        sender,
        tx.clone(),
        app.args.format(),
        path.to_path_buf(),
    ));

    let mut join_set = JoinSet::new();

    // TODO: Make this tunable on the CLI, with the logic here as a fallback.
    // Subtract 1, since we've spawned one task separately.
    let num_workers = tokio::runtime::Handle::current().metrics().num_workers() - 1;

    debug!(num_workers = %num_workers);

    for _ in 0..num_workers {
        join_set.spawn(open_and_id_files(
            receiver.clone(),
            tx.clone(),
            app.args.format(),
            hash,
        ));
    }

    while let Some(result) = join_set.join_next().await {
        result.map_err(Error::CouldNotJoinWorker)??;
    }

    Ok(())
}

/// Walk the target path structure, printing errors and sending discovered
/// paths out to workers.
pub async fn walk_target(
    path_sender: WorkSender<PathBuf>,
    print_tx: PrintSender,
    format: Format,
    path: PathBuf,
) -> Result<()> {
    let mut entries = WalkDir::new(&path);

    loop {
        match entries.next().await {
            None => break Ok(()),
            Some(Err(source)) => {
                print_tx
                    .send(PrinterCmd::msg(
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

                path_sender
                    .send(path.clone())
                    .await
                    .map_err(Error::WorkChannelCloseSend)?;
            }
        }
    }
}

/// Listen on the path receiver and identify each file found.
///
/// The semantics of the channel being used mean each path sent will only
/// be received by one receiver.
async fn open_and_id_files(
    path_rx: Receiver<PathBuf>,
    print_tx: PrintSender,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    pin_mut!(path_rx);

    while let Some(path) = path_rx.next().await {
        let mut file = open_async_file(&path).await?;
        id_file(&print_tx, &mut file, &path, format, hash).await?;
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
    let s = hash_file(hash, file, path).await?;

    tx.send(PrinterCmd::msg(
        IdFileMsg {
            path: path.to_path_buf(),
            id: s.clone(),
        },
        format,
    ))
    .await?;

    Ok(())
}

/// Hash the file and produce a `gitoid`-scheme URL.
pub async fn hash_file(hash: SelectedHash, file: &mut AsyncFile, path: &Path) -> Result<String> {
    match hash {
        SelectedHash::Sha256 => sha256_id_async_file(file, path)
            .await
            .map(|id| id.to_string()),
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
    ArtifactIdBuilder::with_rustcrypto()
        .identify_async(file)
        .await
        .map_err(|source| Error::FileFailedToId {
            path: path.to_path_buf(),
            source,
        })
}
