//! The `manifest add` command, which adds manifests.

use crate::{
    cli::{Config, ManifestAddArgs},
    error::Result,
    print::PrintSender,
};

/// Run the `manifest add` subcommand.
pub async fn run(_tx: &PrintSender, _config: &Config, _args: &ManifestAddArgs) -> Result<()> {
    todo!()
}
