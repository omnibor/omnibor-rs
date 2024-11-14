//! The `debug config` command, which helps debug the CLI configuration.

use crate::{
    cli::Config,
    error::{Error, Result},
    print::{root_dir::RootDirMsg, PrinterCmd},
};
use tokio::sync::mpsc::Sender;

/// Run the `debug config` subcommand.
pub async fn run(tx: &Sender<PrinterCmd>, config: &Config) -> Result<()> {
    let root = config.dir().ok_or(Error::NoRoot)?.to_path_buf();

    tx.send(PrinterCmd::msg(RootDirMsg { path: root }, config.format()))
        .await
        .map_err(|_| Error::PrintChannelClose)?;

    Ok(())
}
