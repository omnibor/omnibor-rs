//! The `id` command, which identifies files.

use crate::cli::IdArgs;
use crate::fs::*;
use crate::print::PrinterCmd;
use anyhow::Result;
use tokio::sync::mpsc::Sender;

/// Run the `id` subcommand.
pub async fn run(tx: &Sender<PrinterCmd>, args: &IdArgs) -> Result<()> {
    let mut file = open_async_file(&args.path).await?;

    if file_is_dir(&file).await? {
        id_directory(tx, &args.path, args.format, args.hash).await?;
    } else {
        id_file(tx, &mut file, &args.path, args.format, args.hash).await?;
    }

    Ok(())
}
