//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
    print::PrintSender,
};
use omnibor::{
    embedding::NoEmbed, hashes::Sha256, storage::FileSystemStorage, InputManifestBuilder,
    IntoArtifactId, RelationKind,
};
use tracing::info;

/// Run the `manifest create` subcommand.
pub async fn run(_tx: &PrintSender, app: &App, args: &ManifestCreateArgs) -> Result<()> {
    let root = app.args.dir().ok_or_else(|| Error::NoRoot)?;

    info!(root = %root.display());

    let storage = FileSystemStorage::new(root).map_err(Error::StorageInitFailed)?;

    let mut builder = InputManifestBuilder::<Sha256, NoEmbed, _>::with_storage(storage);

    for input in &args.inputs {
        let aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        builder
            .add_relation(RelationKind::Input, aid)
            .map_err(Error::AddRelationFailed)?;
    }

    if let Some(built_by) = &args.built_by {
        let aid = built_by
            .clone()
            .into_artifact_id()
            .map_err(Error::IdFailed)?;
        builder
            .add_relation(RelationKind::BuiltBy, aid)
            .map_err(Error::AddRelationFailed)?;
    }

    builder
        .finish(&args.target)
        .map_err(Error::ManifestBuildFailed)?;

    Ok(())
}
