use crate::{
    app::App,
    cli::StoreAddArgs,
    error::{Error, Result},
};
use omnibor::{storage::Storage, InputManifest};

/// Run the `store add` subcommand.
pub async fn run(app: &App, args: &StoreAddArgs) -> Result<()> {
    let mut storage = app.storage()?;

    let target_aid = match &args.target {
        Some(targetable) => {
            let target_aid = targetable
                .clone()
                .into_artifact_id()
                .map_err(Error::IdFailed)?;

            Some(target_aid)
        }
        None => None,
    };

    let manifest = match target_aid {
        Some(target_aid) => InputManifest::load(&args.manifest, target_aid),
        None => InputManifest::load_detached(&args.manifest),
    }
    .map_err(Error::UnableToReadManifest)?;

    storage
        .write_manifest(&manifest)
        .map_err(Error::FailedToAddManifest)?;

    Ok(())
}
