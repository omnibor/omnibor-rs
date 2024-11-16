use crate::{app::App, cli::StoreAddArgs, error::Result, print::PrintSender};

/// Run the `store add` subcommand.
pub async fn run(_tx: &PrintSender, _app: &App, _args: &StoreAddArgs) -> Result<()> {
    todo!()
}
