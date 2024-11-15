//! The `debug config` command, which helps debug the CLI configuration.

use crate::{
    app::App,
    error::{Error, Result},
    print::{root_dir::RootDirMsg, PrintSender, PrinterCmd},
};

/// Run the `debug config` subcommand.
pub async fn run(tx: &PrintSender, app: &App) -> Result<()> {
    let root = app.args.dir().ok_or(Error::NoRoot)?.to_path_buf();

    tx.send(PrinterCmd::msg(
        RootDirMsg { path: root },
        app.args.format(),
    ))
    .await?;

    Ok(())
}
