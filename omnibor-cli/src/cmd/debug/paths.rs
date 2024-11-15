//! The `debug config` command, which helps debug the CLI configuration.

use crate::{
    app::App,
    cli::DebugPathsArgs,
    error::{Error, Result},
    print::{paths::PathsMsg, PrintSender, PrinterCmd},
};
use std::{collections::HashMap, path::Path};

/// Run the `debug paths` subcommand.
pub async fn run(tx: &PrintSender, app: &App, args: &DebugPathsArgs) -> Result<()> {
    let root = app.args.dir().ok_or(Error::NoRoot)?.to_path_buf();

    let mut to_insert: HashMap<&'static str, Option<&Path>> = HashMap::new();
    to_insert.insert("dir", Some(&root));
    to_insert.insert("config", app.args.config());

    let mut msg = PathsMsg::new();

    to_insert
        .into_iter()
        .filter(|(key, _)| {
            let key: String = key.to_string();
            args.keys.contains(&key)
        })
        .for_each(|(key, path)| msg.insert(key, path));

    tx.send(PrinterCmd::msg(msg, app.args.format())).await?;

    Ok(())
}
