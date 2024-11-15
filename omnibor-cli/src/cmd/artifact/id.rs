//! The `artifact id` command, which identifies files.

use crate::{app::App, cli::IdArgs, error::Result, fs::*, print::PrintSender};

/// Run the `artifact id` subcommand.
pub async fn run(tx: &PrintSender, app: &App, args: &IdArgs) -> Result<()> {
    let mut file = open_async_file(&args.path).await?;

    if file_is_dir(&file, &args.path).await? {
        id_directory(app, tx, &args.path).await?;
    } else {
        id_file(
            tx,
            &mut file,
            &args.path,
            app.args.format(),
            app.args.hash(),
        )
        .await?;
    }

    Ok(())
}
