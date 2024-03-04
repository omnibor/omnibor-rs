//! The `find` command, which finds files by ID.

use crate::cli::FindArgs;
use crate::cli::SelectedHash;
use crate::fs::*;
use crate::print::PrinterCmd;
use anyhow::Result;
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt as _;
use omnibor::ArtifactId;
use omnibor::Sha256;
use tokio::sync::mpsc::Sender;

/// Run the `find` subcommand.
pub async fn run(tx: &Sender<PrinterCmd>, args: &FindArgs) -> Result<()> {
    let FindArgs { url, path, format } = args;

    // TODO(alilleybrinker): Correctly handle possible future hash formats.
    let id = ArtifactId::<Sha256>::id_url(url.clone())?;
    let url = id.url();

    let mut entries = WalkDir::new(&path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(e)) => tx.send(PrinterCmd::error(e, *format)).await?,
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(&path).await?;
                let file_url = hash_file(SelectedHash::Sha256, &mut file, &path).await?;

                if url == file_url {
                    tx.send(PrinterCmd::find(&path, &url, *format)).await?;
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}
