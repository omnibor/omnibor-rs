use crate::{
    app::App,
    cli::StoreAddArgs,
    error::{Error, Result},
};
use omnibor::{hash_algorithm::Sha256, storage::Storage, InputManifest};

/// Run the `store add` subcommand.
pub async fn run(app: &App, args: &StoreAddArgs) -> Result<()> {
    let mut storage = app.storage()?;

    let mut manifest =
        InputManifest::<Sha256>::from_path(&args.manifest).map_err(Error::UnableToReadManifest)?;

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

    manifest.set_target(target_aid);

    storage
        .write_manifest(&manifest)
        .map_err(Error::FailedToAddManifest)?;

    Ok(())
}
