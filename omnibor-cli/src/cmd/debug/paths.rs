//! The `debug config` command, which helps debug the CLI configuration.

use crate::{
    app::App,
    cli::DebugPathsArgs,
    error::{Error, Result},
    print::{paths::PathsMsg, PrinterCmd},
};
use std::{collections::HashMap, ops::Not, path::Path};

/// Run the `debug paths` subcommand.
pub async fn run(app: &App, args: &DebugPathsArgs) -> Result<()> {
    let root = app.args.dir().ok_or(Error::NoRoot)?.to_path_buf();

    let mut to_insert: HashMap<&'static str, Option<&Path>> = HashMap::new();
    to_insert.insert("dir", Some(&root));
    to_insert.insert("config", app.args.config());

    let mut msg = PathsMsg::new();

    to_insert
        .into_iter()
        .filter(|(key, _)| {
            if args.keys.is_empty().not() {
                let key: String = key.to_string();
                args.keys.contains(&key)
            } else {
                // Keep everything if there's no filter list.
                true
            }
        })
        .for_each(|(key, path)| msg.insert(key, path));

    app.print_tx
        .send(PrinterCmd::msg(msg, app.args.format()))
        .await?;

    Ok(())
}
