//! The `artifact id` command, which identifies files.

use crate::{app::App, cli::IdCreateArgs, error::Result, fs::*};

/// Run the `artifact id` subcommand.
pub async fn run(app: &App, args: &IdCreateArgs) -> Result<()> {
    let mut file = open_async_file(&args.path).await?;

    if file_is_dir(&file, &args.path).await? {
        id_directory(app, args.hash(), &app.print_tx, &args.path).await?;
    } else {
        id_file(
            &app.print_tx,
            &mut file,
            &args.path,
            app.args.format(),
            args.hash(),
        )
        .await?;
    }

    Ok(())
}
