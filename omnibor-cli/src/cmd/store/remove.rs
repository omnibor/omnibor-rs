use crate::{
    app::App,
    cli::{ManifestCriteria, StoreRemoveArgs},
    error::{Error, Result},
    print::{msg::store_remove::StoreRemoveMsg, PrinterCmd},
};
use omnibor::{hash_algorithm::Sha256, hash_provider::RustCrypto, storage::Storage, ArtifactId};
use tracing::warn;

/// Run the `store remove` subcommand.
pub async fn run(app: &App, args: &StoreRemoveArgs) -> Result<()> {
    for criteria in args.manifest.criteria() {
        match criteria {
            ManifestCriteria::Target(target) => {
                let target_aid = target.into_artifact_id().map_err(Error::IdFailed)?;
                if remove_by_target(app, target_aid).await.is_err() {
                    warn!("failed to remove manifest with target ID '{}'", target_aid);
                }
            }
            ManifestCriteria::Id(id) => {
                let artifact_aid = id.into_artifact_id().map_err(Error::IdFailed)?;
                if remove_by_id(app, artifact_aid).await.is_err() {
                    warn!("failed to remove manifest with ID '{}'", artifact_aid);
                }
            }
        }
    }

    Ok(())
}

async fn remove_by_target(app: &App, target: ArtifactId<Sha256>) -> Result<()> {
    let mut storage = app.storage()?;
    let manifest = storage
        .get_manifest_for_target(target)
        .map_err(Error::CantGetManifests)?
        .ok_or_else(|| Error::ManifestNotFoundForTarget(target))?;

    let provider = RustCrypto::new();

    // SAFETY: Unwrapping a manifest is infallible.
    let manifest_aid = ArtifactId::identify(provider, &manifest).unwrap();

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

    let provider = RustCrypto::new();

    // SAFETY: Unwrapping a manifest is infallible.
    let manifest_aid = ArtifactId::identify(provider, &manifest).unwrap();

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
