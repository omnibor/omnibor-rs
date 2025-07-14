//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
    print::{msg::manifest_create::ManifestCreateMsg, PrinterCmd},
};
use omnibor::{
    embed::{AutoEmbed, Embed, NoEmbed},
    hash_algorithm::Sha256,
    hash_provider::{HashProvider, RustCrypto},
    storage::Storage,
    InputManifestBuilder,
};
use std::ops::Not as _;
use tracing::warn;

/// Run the `manifest create` subcommand.
pub async fn run(app: &App, args: &ManifestCreateArgs) -> Result<()> {
    let storage = app.storage()?;

    let manifest_builder =
        InputManifestBuilder::<Sha256, _, _, _>::new(storage, RustCrypto::new(), AutoEmbed);

    create_with_builder(app, args, manifest_builder).await?;

    Ok(())
}

async fn create_with_builder<P, S, E>(
    app: &App,
    args: &ManifestCreateArgs,
    mut manifest_builder: InputManifestBuilder<Sha256, P, S, E>,
) -> Result<()>
where
    P: HashProvider<Sha256>,
    S: Storage<Sha256>,
    E: Embed<Sha256>,
{
    for input in &args.inputs {
        let input_aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        manifest_builder
            .add_relation(input_aid)
            .map_err(Error::AddRelationFailed)?;
    }

    let will_embed = manifest_builder.will_embed();

    let manifest = manifest_builder
        .finish(&args.target)
        .map_err(Error::ManifestBuildFailed)?
        .or_else(|embedding_error| {
            warn!("embedding failed; '{}'", embedding_error);
            manifest_builder
                .set_embed(NoEmbed)
                .finish(&args.target)
                // PANIC SAFETY: We know we're set to not embed, so this is safe.
                .map(|inner| inner.unwrap())
                .map_err(Error::ManifestBuildFailed)
        })?;

    if args.store {
        app.storage()?
            .write_manifest(&manifest)
            .map_err(Error::FailedToAddManifest)?;
    }

    if args.store.not() && will_embed.not() {
        warn!("manifest is *detached*; if adding to OmniBOR store separately, pass --target to retain target information")
    }

    app.print_tx
        .send(PrinterCmd::msg(
            ManifestCreateMsg { manifest },
            app.args.format(),
        ))
        .await?;

    Ok(())
}
