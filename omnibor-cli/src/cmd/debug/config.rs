//! The `debug config` command, which helps debug the CLI configuration.

use crate::{
    cli::Config,
    error::{Error, Result},
    print::{root_dir::RootDirMsg, PrintSender, PrinterCmd},
};

/// Run the `debug config` subcommand.
pub async fn run(tx: &PrintSender, config: &Config) -> Result<()> {
    let root = config.dir().ok_or(Error::NoRoot)?.to_path_buf();

    tx.send(PrinterCmd::msg(RootDirMsg { path: root }, config.format()))
        .await?;

    Ok(())
}
