use crate::{
    app::App,
    cli::{ManifestCriteria, StoreRemoveArgs},
    error::{Error, Result},
    print::{msg::store_remove::StoreRemoveMsg, PrinterCmd},
};
use omnibor::{hash_algorithm::Sha256, storage::Storage, ArtifactId, ArtifactIdBuilder};

/// Run the `store remove` subcommand.
pub async fn run(app: &App, args: &StoreRemoveArgs) -> Result<()> {
    match args.manifest.criteria() {
        ManifestCriteria::Target(target) => {
            remove_by_target(app, target.into_artifact_id().map_err(Error::IdFailed)?).await
        }
        ManifestCriteria::Id(id) => {
            remove_by_id(app, id.into_artifact_id().map_err(Error::IdFailed)?).await
        }
    }
}

async fn remove_by_target(app: &App, target: ArtifactId<Sha256>) -> Result<()> {
    let mut storage = app.storage()?;
    let manifest = storage
        .get_manifest_for_target(target)
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundForTarget(target))?;

    let manifest_aid = ArtifactIdBuilder::with_rustcrypto().identify_manifest(&manifest);

    storage
        .remove_manifest_for_target(target)
        .map_err(Error::FailedToRemoveManifest)?;

    app.print_tx
        .send(PrinterCmd::msg(
            StoreRemoveMsg { manifest_aid },
            app.args.format(),
        ))
        .await?;

    Ok(())
}

async fn remove_by_id(app: &App, id: ArtifactId<Sha256>) -> Result<()> {
    let mut storage = app.storage()?;
    let manifest = storage
        .get_manifest_with_id(id)
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundWithId(id))?;

    let manifest_aid = ArtifactIdBuilder::with_rustcrypto().identify_manifest(&manifest);

    storage
        .remove_manifest_with_id(id)
        .map_err(Error::FailedToRemoveManifest)?;

    app.print_tx
        .send(PrinterCmd::msg(
            StoreRemoveMsg { manifest_aid },
            app.args.format(),
        ))
        .await?;

    Ok(())
}
