//! The `debug config` command, which helps debug the CLI configuration.

use crate::{cli::Config, error::Result, print::PrinterCmd};
use tokio::sync::mpsc::Sender;

/// Run the `debug config` subcommand.
pub async fn run(tx: &Sender<PrinterCmd>, config: &Config) -> Result<()> {
    tx.send(PrinterCmd::root_dir(config.dir(), config.format()))
        .await?;
    Ok(())
}
