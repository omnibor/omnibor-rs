//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
};
use omnibor::{
    artifact_id::{ArtifactId, ArtifactIdBuilder},
    hash_algorithm::Sha256,
    hash_provider::{HashProvider, RustCrypto},
    input_manifest::{
        embedding_mode::{EmbeddingMode, NoEmbed},
        InputManifestBuilder, ShouldStore,
    },
    storage::{FileSystemStorage, Storage},
};
use pathbuf::pathbuf;
use std::{
    env::current_dir,
    fs::File,
    io::Write,
    ops::Not as _,
    path::{Path, PathBuf},
};
use tracing::info;

/// Run the `manifest create` subcommand.
pub async fn run(app: &App, args: &ManifestCreateArgs) -> Result<()> {
    if args.no_store && args.no_out {
        return Err(Error::NoStoreAndNoOut);
    }

    let root = app.args.dir().ok_or(Error::NoRoot)?;
    let storage =
        FileSystemStorage::new(RustCrypto::new(), root).map_err(Error::StorageInitFailed)?;
    let builder = InputManifestBuilder::<Sha256, NoEmbed, _, _>::new(storage, RustCrypto::new());
    create_with_builder(args, builder)?;
    Ok(())
}

fn create_with_builder<E, S, P>(
    args: &ManifestCreateArgs,
    mut builder: InputManifestBuilder<Sha256, E, S, P>,
) -> Result<()>
where
    E: EmbeddingMode,
    S: Storage<Sha256>,
    P: HashProvider<Sha256>,
{
    for input in &args.inputs {
        let aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        builder
            .add_relation(aid)
            .map_err(Error::AddRelationFailed)?;
    }

    let should_store = if args.no_store {
        ShouldStore::No
    } else {
        ShouldStore::Yes
    };

    let linked_manifest = builder
        .finish(&args.target, should_store)
        .map_err(Error::ManifestBuildFailed)?;

    if args.no_out.not() {
        let path = manifest_file_path(args.output.as_deref(), linked_manifest.target_aid())?;

        let mut output_file = match File::create_new(&path) {
            Ok(file) => file,
            Err(source) => {
                let mut existing_file = File::open(&path).unwrap();
                let existing_file_aid = ArtifactIdBuilder::with_rustcrypto()
                    .identify_file(&mut existing_file)
                    .unwrap();
                if existing_file_aid == linked_manifest.manifest_aid() {
                    info!("matching manifest already found at '{}'", path.display());
                    return Ok(());
                } else {
                    return Err(Error::CantWriteManifest {
                        path: path.to_path_buf(),
                        source,
                    });
                }
            }
        };

        output_file
            // SAFETY: We just constructed the manifest, so we know it's fine.
            .write_all(&linked_manifest.manifest().as_bytes())
            .map_err(|source| Error::CantWriteManifest {
                path: path.to_path_buf(),
                source,
            })?;

        info!(
            "wrote manifest '{}' to '{}'",
            linked_manifest.manifest_aid(),
            path.display()
        );
    }

    Ok(())
}

fn manifest_file_path(output: Option<&Path>, target_aid: ArtifactId<Sha256>) -> Result<PathBuf> {
    let dir = match &output {
        Some(dir) => dir.to_path_buf(),
        None => match current_dir() {
            Ok(dir) => dir,
            Err(_) => return Err(Error::NoOutputDir),
        },
    };

    Ok(pathbuf![&dir, &target_aid.as_file_name()])
}
