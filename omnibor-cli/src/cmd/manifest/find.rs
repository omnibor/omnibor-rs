//! The `manifest find` command, which searches in manifests.

use crate::{
    app::App,
    cli::{Format, ManifestFindArgs},
    error::{Error, Result},
    fs::*,
    print::{msg::manifest_find::ManifestFindMsg, PrintSender, PrinterCmd},
};
use async_channel::{bounded, Receiver};
use futures_lite::stream::StreamExt as _;
use futures_util::pin_mut;
use omnibor::{hash_algorithm::Sha256, hash_provider::RustCrypto, ArtifactId, InputManifest};
use std::path::PathBuf;
use tokio::task::JoinSet;
use tracing::debug;

/// Run the `manifest find` subcommand.
pub async fn run(app: &App, args: &ManifestFindArgs) -> Result<()> {
    let ManifestFindArgs { aid, path } = args;

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
        join_set.spawn(open_and_match_manifests(
            receiver.clone(),
            app.print_tx.clone(),
            app.args.format(),
            *aid,
        ));
    }

    while let Some(result) = join_set.join_next().await {
        result.map_err(Error::CouldNotJoinWorker)??;
    }

    Ok(())
}

async fn open_and_match_manifests(
    path_rx: Receiver<PathBuf>,
    tx: PrintSender,
    format: Format,
    target_aid: ArtifactId<Sha256>,
) -> Result<()> {
    pin_mut!(path_rx);

    while let Some(path) = path_rx.next().await {
        if let Ok(manifest) = InputManifest::<Sha256>::load(&path, None) {
            if manifest
                .contains_artifact(RustCrypto::new(), target_aid)
                .map_err(|source| Error::FileFailedToIdDuringSearch {
                    path: path.to_path_buf(),
                    source,
                })?
            {
                tx.send(PrinterCmd::msg(
                    ManifestFindMsg {
                        path: path.to_path_buf(),
                        manifest: manifest.clone(),
                    },
                    format,
                ))
                .await?;
            }
        }
    }

    Ok(())
}
