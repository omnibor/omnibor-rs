use crate::{
    app::App,
    cli::{ManifestCriteria, StoreGetArgs},
    error::{Error, Result},
    print::{msg::store_get::StoreGetMsg, PrinterCmd},
};
use omnibor::{
    hash_algorithm::Sha256,
    storage::{Match, Storage},
    ArtifactId,
};
use tracing::warn;

/// Run the `store get` subcommand.
pub async fn run(app: &App, args: &StoreGetArgs) -> Result<()> {
    if args.all {
        return get_all(app).await;
    }

    for criteria in args.manifest.criteria() {
        match criteria {
            ManifestCriteria::Target(target) => {
                let target_aid = target.into_artifact_id().map_err(Error::IdFailed)?;
                if get_by_target(app, target_aid).await.is_err() {
                    warn!("manifest with target '{}' not found", target_aid);
                }
            }
            ManifestCriteria::Id(id) => {
                let artifact_aid = id.into_artifact_id().map_err(Error::IdFailed)?;
                if get_by_id(app, artifact_aid).await.is_err() {
                    warn!("manifest with ID '{}' not found", artifact_aid);
                }
            }
        }
    }

    Ok(())
}

async fn get_all(app: &App) -> Result<()> {
    let storage = app.storage()?;

    for manifest in storage.get_manifests().map_err(Error::CantGetManifests)? {
        app.print_tx
            .send(PrinterCmd::msg(StoreGetMsg { manifest }, app.args.format()))
            .await?;
    }

    Ok(())
}

async fn get_by_target(app: &App, target: ArtifactId<Sha256>) -> Result<()> {
    let storage = app.storage()?;
    let manifest = storage
        .get_manifest(Match::Target(target))
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
        .get_manifest(Match::Manifest(id))
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundWithId(id))?;

    app.print_tx
        .send(PrinterCmd::msg(StoreGetMsg { manifest }, app.args.format()))
        .await?;

    Ok(())
}
