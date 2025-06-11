//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
};
use omnibor::{
    hash_algorithm::Sha256,
    hash_provider::{HashProvider, RustCrypto},
    storage::Storage,
    ArtifactId, ArtifactIdBuilder, EmbeddingMode, InputManifestBuilder,
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
    let storage = app.storage()?;
    let builder = InputManifestBuilder::<Sha256, _, _>::new(
        EmbeddingMode::NoEmbed,
        storage,
        RustCrypto::new(),
    );
    create_with_builder(args, builder)?;
    Ok(())
}

fn create_with_builder<P, S>(
    args: &ManifestCreateArgs,
    mut builder: InputManifestBuilder<Sha256, P, S>,
) -> Result<()>
where
    P: HashProvider<Sha256>,
    S: Storage<Sha256>,
{
    for input in &args.inputs {
        let aid = input.clone().into_artifact_id().map_err(Error::IdFailed)?;
        builder
            .add_relation(aid)
            .map_err(Error::AddRelationFailed)?;
    }

    let linked_manifest = builder
        .finish(&args.target)
        .map_err(Error::ManifestBuildFailed)?;

    let manifest_aid = ArtifactIdBuilder::with_rustcrypto().identify_manifest(&linked_manifest);

    if args.no_out.not() {
        let target_aid = ArtifactIdBuilder::with_rustcrypto()
            .identify_path(&args.target)
            .map_err(|source| Error::FileFailedToId {
                path: args.target.clone(),
                source,
            })?;

        let path = manifest_file_path(args.output.as_deref(), target_aid)?;

        let mut output_file = match File::create_new(&path) {
            Ok(file) => file,
            Err(source) => {
                let mut existing_file = File::open(&path).unwrap();
                let existing_file_aid = ArtifactIdBuilder::with_rustcrypto()
                    .identify_file(&mut existing_file)
                    .unwrap();
                if existing_file_aid == manifest_aid {
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
            .write_all(&linked_manifest.as_bytes())
            .map_err(|source| Error::CantWriteManifest {
                path: path.to_path_buf(),
                source,
            })?;

        info!("wrote manifest '{}' to '{}'", manifest_aid, path.display());
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
