//! The `artifact find` command, which finds files by ID.

use std::path::PathBuf;

use crate::{
    cli::{Config, FindArgs, Format, SelectedHash},
    error::{Error, Result},
    fs::*,
    print::{find_file::FindFileMsg, PrintSender, PrinterCmd},
};
use async_channel::{bounded, Receiver};
use futures_lite::stream::StreamExt as _;
use futures_util::pin_mut;
use tokio::task::JoinSet;
use tracing::debug;
use url::Url;

// TODO: Make this tunable on the CLI.
/// Capacity of the async channel between the producer loop and the consumers.
const ID_CHAN_CAPACITY: usize = 20;

/// Run the `artifact find` subcommand.
pub async fn run(tx: &PrintSender, config: &Config, args: &FindArgs) -> Result<()> {
    let FindArgs { aid, path } = args;
    let url = aid.url();

    let (sender, receiver) = bounded(ID_CHAN_CAPACITY);

    tokio::spawn(walk_target(
        sender,
        tx.clone(),
        config.format(),
        path.to_path_buf(),
    ));

    let mut join_set = JoinSet::new();

    // TODO: Make this tunable on the CLI, with the logic here as a fallback.
    // Subtract 1, since we've spawned one task separately.
    let num_workers = tokio::runtime::Handle::current().metrics().num_workers() - 1;

    debug!(num_workers = %num_workers);

    for _ in 0..num_workers {
        join_set.spawn(open_and_match_files(
            receiver.clone(),
            tx.clone(),
            config.format(),
            url.clone(),
        ));
    }

    while let Some(result) = join_set.join_next().await {
        result.map_err(Error::CouldNotJoinWorker)??;
    }

    Ok(())
}

async fn open_and_match_files(
    path_rx: Receiver<PathBuf>,
    tx: PrintSender,
    format: Format,
    url: Url,
) -> Result<()> {
    pin_mut!(path_rx);

    while let Some(path) = path_rx.next().await {
        let mut file = open_async_file(&path).await?;
        let file_url = hash_file(SelectedHash::Sha256, &mut file, &path).await?;

        if url == file_url {
            tx.send(PrinterCmd::msg(
                FindFileMsg {
                    path: path.to_path_buf(),
                    id: url.clone(),
                },
                format,
            ))
            .await?;
        }
    }

    Ok(())
}
