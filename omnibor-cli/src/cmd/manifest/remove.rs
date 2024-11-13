//! The `manifest remove` command, which removes manifests.

use crate::{
    cli::{Config, ManifestRemoveArgs},
    error::Result,
    print::PrinterCmd,
};
use tokio::sync::mpsc::Sender;

/// Run the `manifest remove` subcommand.
pub async fn run(
    _tx: &Sender<PrinterCmd>,
    _config: &Config,
    _args: &ManifestRemoveArgs,
) -> Result<()> {
    todo!()
}
