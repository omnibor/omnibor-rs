//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
    print::{msg::manifest_create::ManifestCreateMsg, PrinterCmd},
};
use omnibor::{embed::AutoEmbed, hash_algorithm::Sha256, storage::Storage, InputManifestBuilder};
use std::ops::Not as _;
use tracing::warn;

/// Run the `manifest create` subcommand.
pub async fn run(app: &App, args: &ManifestCreateArgs) -> Result<()> {
    let storage = app.storage()?;

    let manifest_builder = InputManifestBuilder::<Sha256, _>::new(storage);

    create_with_builder(app, args, manifest_builder).await?;

    Ok(())
}

async fn create_with_builder<S>(
    app: &App,
    args: &ManifestCreateArgs,
    mut manifest_builder: InputManifestBuilder<Sha256, S>,
) -> Result<()>
where
    S: Storage<Sha256>,
{
    for input in &args.inputs {
        let input_aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        manifest_builder
            .add_relation(input_aid)
            .map_err(Error::AddRelationFailed)?;
    }

    let manifest = manifest_builder
        .continue_on_failed_embed(true)
        .build_for_target(&args.target, AutoEmbed)
        .map_err(Error::ManifestBuildFailed)?;

    if args.store {
        app.storage()?
            .write_manifest(&manifest)
            .map_err(Error::FailedToAddManifest)?;
    }

    if args.store.not() {
        warn!("manifest may be *detached*; if adding to OmniBOR store separately, pass --target to retain target information")
    }

    app.print_tx
        .send(PrinterCmd::msg(
            ManifestCreateMsg { manifest },
            app.args.format(),
        ))
        .await?;

    Ok(())
}
