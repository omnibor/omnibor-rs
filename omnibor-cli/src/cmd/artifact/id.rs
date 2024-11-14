//! The `artifact id` command, which identifies files.

use crate::{
    cli::{Config, IdArgs},
    error::Result,
    fs::*,
    print::PrintSender,
};

/// Run the `artifact id` subcommand.
pub async fn run(tx: &PrintSender, config: &Config, args: &IdArgs) -> Result<()> {
    let mut file = open_async_file(&args.path).await?;

    if file_is_dir(&file, &args.path).await? {
        id_directory(tx, &args.path, config.format(), config.hash()).await?;
    } else {
        id_file(tx, &mut file, &args.path, config.format(), config.hash()).await?;
    }

    Ok(())
}
