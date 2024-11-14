//! The `manifest remove` command, which removes manifests.

use crate::{
    cli::{Config, ManifestRemoveArgs},
    error::Result,
    print::PrintSender,
};

/// Run the `manifest remove` subcommand.
pub async fn run(_tx: &PrintSender, _config: &Config, _args: &ManifestRemoveArgs) -> Result<()> {
    todo!()
}
