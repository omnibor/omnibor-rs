use crate::{app::App, cli::StoreLogArgs, error::Result, print::PrintSender};

/// Run the `store log` subcommand.
pub async fn run(_tx: &PrintSender, _app: &App, _args: &StoreLogArgs) -> Result<()> {
    todo!()
}
