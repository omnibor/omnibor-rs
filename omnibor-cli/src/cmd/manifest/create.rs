//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
    print::{msg::manifest_create::ManifestCreateMsg, PrinterCmd},
};
use omnibor::{
    hash_algorithm::Sha256,
    hash_provider::{HashProvider, RustCrypto},
    storage::Storage,
    EmbeddingMode, InputManifestBuilder,
};

/// Run the `manifest create` subcommand.
pub async fn run(app: &App, args: &ManifestCreateArgs) -> Result<()> {
    let storage = app.storage()?;

    let manifest_builder = InputManifestBuilder::<Sha256, _, _>::new(
        EmbeddingMode::NoEmbed,
        storage,
        RustCrypto::new(),
    );

    create_with_builder(app, args, manifest_builder).await?;
    Ok(())
}

async fn create_with_builder<P, S>(
    app: &App,
    args: &ManifestCreateArgs,
    mut manifest_builder: InputManifestBuilder<Sha256, P, S>,
) -> Result<()>
where
    P: HashProvider<Sha256>,
    S: Storage<Sha256>,
{
    for input in &args.inputs {
        let input_aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        manifest_builder
            .add_relation(input_aid)
            .map_err(Error::AddRelationFailed)?;
    }

    let manifest = manifest_builder
        .finish(&args.target)
        .map_err(Error::ManifestBuildFailed)?;

    app.print_tx
        .send(PrinterCmd::msg(
            ManifestCreateMsg { manifest },
            app.args.format(),
        ))
        .await?;

    Ok(())
}
