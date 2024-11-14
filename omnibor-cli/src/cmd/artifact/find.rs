//! The `artifact find` command, which finds files by ID.

use crate::{
    cli::{Config, FindArgs, SelectedHash},
    error::{Error, Result},
    fs::*,
    print::{error::ErrorMsg, find_file::FindFileMsg, PrintSender, PrinterCmd},
};
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt as _;

/// Run the `artifact find` subcommand.
pub async fn run(tx: &PrintSender, config: &Config, args: &FindArgs) -> Result<()> {
    let FindArgs { aid, path } = args;

    let url = aid.url();

    let mut entries = WalkDir::new(path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(source)) => {
                tx.send(PrinterCmd::msg(
                    ErrorMsg::new(Error::WalkDirFailed {
                        path: path.to_path_buf(),
                        source,
                    }),
                    config.format(),
                ))
                .await?
            }
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(path).await?;
                let file_url = hash_file(SelectedHash::Sha256, &mut file, path).await?;

                if url == file_url {
                    tx.send(PrinterCmd::msg(
                        FindFileMsg {
                            path: path.to_path_buf(),
                            id: url.clone(),
                        },
                        config.format(),
                    ))
                    .await?;
                }
            }
        }
    }

    Ok(())
}
