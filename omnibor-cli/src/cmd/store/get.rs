use crate::{
    app::App,
    cli::{ManifestCriteria, StoreGetArgs},
    error::{Error, Result},
    print::{msg::store_get::StoreGetMsg, PrinterCmd},
};
use omnibor::{hash_algorithm::Sha256, storage::Storage, ArtifactId};

/// Run the `store get` subcommand.
pub async fn run(app: &App, args: &StoreGetArgs) -> Result<()> {
    match args.manifest.criteria() {
        ManifestCriteria::Target(target) => {
            get_by_target(app, target.into_artifact_id().map_err(Error::IdFailed)?).await
        }
        ManifestCriteria::Id(id) => {
            get_by_id(app, id.into_artifact_id().map_err(Error::IdFailed)?).await
        }
    }
}

async fn get_by_target(app: &App, target: ArtifactId<Sha256>) -> Result<()> {
    let storage = app.storage()?;
    let manifest = storage
        .get_manifest_for_target(target)
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundForTarget(target))?;

    app.print_tx
        .send(PrinterCmd::msg(StoreGetMsg { manifest }, app.args.format()))
        .await?;

    Ok(())
}

async fn get_by_id(app: &App, id: ArtifactId<Sha256>) -> Result<()> {
    let storage = app.storage()?;
    let manifest = storage
        .get_manifest_with_id(id)
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundWithId(id))?;

    app.print_tx
        .send(PrinterCmd::msg(StoreGetMsg { manifest }, app.args.format()))
        .await?;

    Ok(())
}
