//! The `manifest create` command, which creates manifests.

use crate::{
    app::App,
    cli::ManifestCreateArgs,
    error::{Error, Result},
    print::PrintSender,
};
use omnibor::{
    embedding::{EmbeddingMode, NoEmbed},
    hashes::Sha256,
    storage::{FileSystemStorage, InMemoryStorage, Storage},
    ArtifactId, InputManifestBuilder, IntoArtifactId, RelationKind, ShouldStore,
};
use pathbuf::pathbuf;
use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// Run the `manifest create` subcommand.
pub async fn run(_tx: &PrintSender, app: &App, args: &ManifestCreateArgs) -> Result<()> {
    let root = app.args.dir().ok_or(Error::NoRoot)?;

    if args.no_store {
        let storage = InMemoryStorage::new();
        let builder = InputManifestBuilder::<Sha256, NoEmbed, _>::with_storage(storage);
        create_with_builder(args, builder)
    } else {
        let storage = FileSystemStorage::new(root).map_err(Error::StorageInitFailed)?;
        let builder = InputManifestBuilder::<Sha256, NoEmbed, _>::with_storage(storage);
        create_with_builder(args, builder)
    }
}

fn create_with_builder<E, S>(
    args: &ManifestCreateArgs,
    mut builder: InputManifestBuilder<Sha256, E, S>,
) -> Result<()>
where
    E: EmbeddingMode,
    S: Storage<Sha256>,
{
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

    let should_store = if args.no_store {
        ShouldStore::No
    } else {
        ShouldStore::Yes
    };

    let transaction_ids = builder
        .finish(&args.target, should_store)
        .map_err(Error::ManifestBuildFailed)?;

    let path = manifest_file_path(args.output.as_deref(), transaction_ids.target_aid())?;

    let mut output_file = File::create_new(&path).map_err(|source| Error::CantWriteManifest {
        path: path.to_path_buf(),
        source,
    })?;

    output_file
        // SAFETY: We just constructed the manifest, so we know it's fine.
        .write_all(&transaction_ids.manifest().as_bytes().unwrap())
        .map_err(|source| Error::CantWriteManifest {
            path: path.to_path_buf(),
            source,
        })?;

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

    let file_name = format!("{}.manifest", target_aid.as_hex());

    Ok(pathbuf![&dir, &file_name])
}
