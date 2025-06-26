//! The `artifact find` command, which finds files by ID.

use crate::{
    app::App,
    cli::{Format, IdFindArgs, SelectedHash},
    error::{Error, Result},
    fs::*,
    print::{msg::find_file::FindFileMsg, PrintSender, PrinterCmd},
};
use async_channel::{bounded, Receiver};
use futures_lite::stream::StreamExt as _;
use futures_util::pin_mut;
use std::path::PathBuf;
use tokio::task::JoinSet;
use tracing::debug;

/// Run the `artifact find` subcommand.
pub async fn run(app: &App, args: &IdFindArgs) -> Result<()> {
    let IdFindArgs { aid, path } = args;
    let url = aid.to_string();

    let (sender, receiver) = bounded(app.config.perf.work_queue_size());

    tokio::spawn(walk_target(
        sender,
        app.print_tx.clone(),
        app.args.format(),
        path.to_path_buf(),
    ));

    let mut join_set = JoinSet::new();

    let num_workers = app.config.perf.num_workers();
    debug!(num_workers = %num_workers);

    for _ in 0..num_workers {
        join_set.spawn(open_and_match_files(
            receiver.clone(),
            app.print_tx.clone(),
            app.args.format(),
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
    s: String,
) -> Result<()> {
    pin_mut!(path_rx);

    while let Some(path) = path_rx.next().await {
        let mut file = open_async_file(&path).await?;
        let file_url = hash_file(SelectedHash::Sha256, &mut file, &path).await?;

        if s == file_url {
            tx.send(PrinterCmd::msg(
                FindFileMsg {
                    path: path.to_path_buf(),
                    id: s.clone(),
                },
                format,
            ))
            .await?;
        }
    }

    Ok(())
}
