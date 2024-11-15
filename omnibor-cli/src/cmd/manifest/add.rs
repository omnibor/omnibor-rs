//! The `manifest add` command, which adds manifests.

use crate::{app::App, cli::ManifestAddArgs, error::Result, print::PrintSender};

/// Run the `manifest add` subcommand.
pub async fn run(_tx: &PrintSender, _app: &App, _args: &ManifestAddArgs) -> Result<()> {
    todo!()
}
