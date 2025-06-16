use crate::{
    app::App,
    cli::StoreAddArgs,
    error::{Error, Result},
};
use omnibor::{hash_algorithm::Sha256, storage::Storage, InputManifest};

/// Run the `store add` subcommand.
pub async fn run(app: &App, args: &StoreAddArgs) -> Result<()> {
    let mut storage = app.storage()?;

    let manifest =
        InputManifest::<Sha256>::from_path(&args.manifest).map_err(Error::UnableToReadManifest)?;

    let manifest_aid = storage
        .write_manifest(&manifest)
        .map_err(Error::FailedToAddManifest)?;

    if let Some(target) = args.target.clone() {
        let target_aid = target.into_artifact_id().map_err(Error::IdFailed)?;
        storage
            .update_target_for_manifest(manifest_aid, target_aid)
            .map_err(Error::FailedToUpdateTarget)?;
    }

    Ok(())
}
