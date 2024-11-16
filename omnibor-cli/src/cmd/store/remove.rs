use crate::{app::App, cli::StoreRemoveArgs, error::Result, print::PrintSender};

/// Run the `store remove` subcommand.
pub async fn run(_tx: &PrintSender, _app: &App, _args: &StoreRemoveArgs) -> Result<()> {
    todo!()
}
