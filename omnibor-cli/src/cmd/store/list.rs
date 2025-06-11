use crate::{
    app::App,
    cli::StoreListArgs,
    error::Error,
    print::{msg::store_list::StoreListMsg, PrinterCmd},
};
use omnibor::{storage::Storage, ArtifactIdBuilder};

/// Run the `store list` subcommand.
pub async fn run(app: &App, _args: &StoreListArgs) -> Result<(), Error> {
    let storage = app.storage()?;
    let builder = ArtifactIdBuilder::with_rustcrypto();
    let manifests = storage.get_manifests().map_err(Error::CantGetManifests)?;

    for manifest in manifests {
        app.print_tx
            .send(PrinterCmd::msg(
                StoreListMsg {
                    manifest: builder.identify_manifest(&manifest),
                    target: manifest.target(),
                },
                app.args.format(),
            ))
            .await?;
    }

    Ok(())
}
