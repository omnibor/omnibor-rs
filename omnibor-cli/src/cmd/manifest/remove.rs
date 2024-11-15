//! The `manifest remove` command, which removes manifests.

use crate::{app::App, cli::ManifestRemoveArgs, error::Result, print::PrintSender};

/// Run the `manifest remove` subcommand.
pub async fn run(_tx: &PrintSender, _app: &App, _args: &ManifestRemoveArgs) -> Result<()> {
    todo!()
}
