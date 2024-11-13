//! The `manifest create` command, which creates manifests.

use crate::{
    cli::{Config, ManifestCreateArgs},
    print::PrinterCmd,
};
use anyhow::{anyhow, Result};
use omnibor::{
    embedding::NoEmbed, hashes::Sha256, storage::FileSystemStorage, InputManifestBuilder,
    IntoArtifactId, RelationKind,
};
use tokio::sync::mpsc::Sender;
use tracing::info;

/// Run the `manifest create` subcommand.
pub async fn run(
    _tx: &Sender<PrinterCmd>,
    config: &Config,
    args: &ManifestCreateArgs,
) -> Result<()> {
    let root = config
        .dir()
        .ok_or_else(|| anyhow!("no root directory found"))?;

    info!(root = %root.display());

    let storage = FileSystemStorage::new(root)?;

    let mut builder = InputManifestBuilder::<Sha256, NoEmbed, _>::with_storage(storage);

    for input in &args.inputs {
        let aid = input.clone().into_artifact_id()?;
        builder.add_relation(RelationKind::Input, aid)?;
    }

    if let Some(built_by) = &args.built_by {
        let aid = built_by.clone().into_artifact_id()?;
        builder.add_relation(RelationKind::BuiltBy, aid)?;
    }

    builder.finish(&args.target)?;

    Ok(())
}
