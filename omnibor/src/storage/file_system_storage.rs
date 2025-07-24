use {
    crate::{
        artifact_id::ArtifactId,
        error::InputManifestError,
        hash_algorithm::HashAlgorithm,
        input_manifest::InputManifest,
        pathbuf,
        storage::{query::Match, Storage},
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    std::{
        collections::HashMap,
        env::var_os,
        fmt::Debug,
        fs::{self, create_dir_all, write, File},
        io::{BufRead as _, BufReader, BufWriter, Write as _},
        marker::PhantomData,
        ops::Not as _,
        path::{Path, PathBuf},
        str::FromStr,
    },
    walkdir::{DirEntry, WalkDir},
};

/// File system storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct FileSystemStorage<H: HashAlgorithm> {
    _hash_algorithm: PhantomData<H>,
    root: PathBuf,
}

impl<H: HashAlgorithm> FileSystemStorage<H> {
    /// Start building a new [`FileSystemStorage`].
    pub fn new(root: impl AsRef<Path>) -> Result<FileSystemStorage<H>, InputManifestError> {
        let root = root.as_ref().to_owned();

        if root.exists() {
            let meta = fs::metadata(&root).map_err(|source| {
                InputManifestError::CantAccessRoot(root.clone_as_boxstr(), Box::new(source))
            })?;

            if meta.is_dir().not() {
                return Err(InputManifestError::ObjectStoreNotDir(
                    root.clone_as_boxstr(),
                ));
            }
        } else {
            create_dir_all(&root).map_err(|source| {
                InputManifestError::CantCreateObjectStoreDir(
                    root.clone_as_boxstr(),
                    Box::new(source),
                )
            })?;
        }

        Ok(FileSystemStorage {
            _hash_algorithm: PhantomData,
            root,
        })
    }

    /// Build a [`FileSystemStorage`] with a root set from
    /// the `OMNIBOR_DIR` environment variable.
    pub fn from_env() -> Result<FileSystemStorage<H>, InputManifestError> {
        var_os("OMNIBOR_DIR")
            .ok_or(InputManifestError::NoStorageRoot)
            .map(|root| FileSystemStorage {
                _hash_algorithm: PhantomData,
                root: PathBuf::from(root),
            })
    }

    /// Fully delete the contents of the root dir.
    ///
    /// This is just used for tests to ensure idempotency.
    #[cfg(test)]
    pub fn cleanup(self) -> Result<(), InputManifestError> {
        fs::remove_dir_all(&self.root).map_err(|source| {
            InputManifestError::FailedStorageCleanup(self.root.clone_as_boxstr(), Box::new(source))
        })?;

        fs::create_dir_all(&self.root).map_err(|source| {
            InputManifestError::CantCreateObjectStoreDir(
                self.root.clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        Ok(())
    }

    /// Get the path to the manifest store.
    fn manifests_path(&self) -> PathBuf {
        pathbuf![&self.root, "manifests"]
    }

    /// Get the path to the target index file.
    fn target_file_path(&self) -> PathBuf {
        pathbuf![&self.root, "targets"]
    }

    /// Open the target index file
    fn target_index(&self) -> Result<TargetIndex, InputManifestError> {
        TargetIndex::new(self.target_file_path())
    }

    /// Get the path for storing a manifest with this [`ArtifactId`].
    pub(crate) fn manifest_path(&self, aid: ArtifactId<H>) -> PathBuf {
        let kind = format!("gitoid_{}_{}", aid.object_type(), aid.hash_algorithm());
        let hash = aid.as_hex();
        let (prefix, remainder) = hash.split_at(2);
        pathbuf![&self.manifests_path(), &kind, prefix, remainder]
    }

    /// Iterate over the targets of manifests currently in the object store.
    fn manifests(&self) -> impl Iterator<Item = ManifestsEntry<H>> + '_ {
        WalkDir::new(self.manifests_path())
            .into_iter()
            .filter_map(|result| result.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.file_name().to_str().is_some())
            .filter_map(|entry| {
                let manifest_aid = artifact_id_from_dir_entry(&entry)?;
                let target_aid = self.target_index().ok()?.find(manifest_aid).ok().flatten();
                let manifest_path = entry.path().to_owned();

                Some(ManifestsEntry {
                    target_aid,
                    manifest_path,
                })
            })
    }

    fn get_manifest_for_target(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        let target_index = self.target_index()?;

        self.manifests()
            .find(|entry| entry.target_aid == Some(target_aid))
            .map(|entry| enrich_manifest_with_target(&target_index, entry.manifest()?))
            .transpose()
    }

    fn get_manifest_with_id(
        &self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        let target_index = self.target_index()?;

        self.manifests()
            .find(|entry| ArtifactId::new(&entry.manifest_path).ok() == Some(manifest_aid))
            .map(|entry| enrich_manifest_with_target(&target_index, entry.manifest()?))
            .transpose()
    }

    fn remove_manifest_for_target(
        &mut self,
        target_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        // Remove the relevant entry from the target index.
        self.target_index()?
            .start_remove()
            .set_target_aid(target_aid)
            .run_remove()?;

        // Find the manifest in the store.
        let manifest_entry: ManifestsEntry<H> = self
            .manifests()
            .find(|entry| entry.target_aid == Some(target_aid))
            .ok_or(InputManifestError::NoManifestFoundToRemove)?;

        // Get a copy of it to return.
        let removed_manifest = manifest_entry.manifest()?;

        // Remove it from the store.
        fs::remove_file(&manifest_entry.manifest_path)
            .map_err(|source| InputManifestError::CantRemoveManifest(Box::new(source)))?;

        Ok(removed_manifest)
    }

    fn remove_manifest_with_id(
        &mut self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        // Remove the relevant entry from the target index.
        self.target_index()?
            .start_remove()
            .set_manifest_aid(manifest_aid)
            .run_remove()?;

        // Find the manifest in the store.
        let manifest_entry: ManifestsEntry<H> = self
            .manifests()
            .find(|entry| ArtifactId::new(&entry.manifest_path).ok() == Some(manifest_aid))
            .ok_or(InputManifestError::NoManifestFoundToRemove)?;

        // Get a copy of it to return.
        let removed_manifest = manifest_entry.manifest()?;

        // Remove it from the store.
        fs::remove_file(&manifest_entry.manifest_path)
            .map_err(|source| InputManifestError::CantRemoveManifest(Box::new(source)))?;

        Ok(removed_manifest)
    }
}

/// Enrich an InputManifest with its target if stored in the index.
fn enrich_manifest_with_target<H: HashAlgorithm>(
    target_index: &TargetIndex,
    mut manifest: InputManifest<H>,
) -> Result<InputManifest<H>, InputManifestError> {
    // Get the Artifact ID of the manifest
    // SAFETY: identifying a manifest is infallible.
    let manifest_aid = ArtifactId::new(&manifest).unwrap();

    // Get the target Artifact ID for the manifest
    let target_aid = target_index.find(manifest_aid)?;

    // Update the manifest with the target Artifact ID.
    manifest.set_target(target_aid);

    Ok(manifest)
}

impl<H: HashAlgorithm> Storage<H> for FileSystemStorage<H> {
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError> {
        // SAFETY: Identifying a manifest is infallible.
        let manifest_aid = ArtifactId::new(manifest).unwrap();

        let path = self.manifest_path(manifest_aid);

        let parent_dirs = path
            .parent()
            .ok_or_else(|| InputManifestError::InvalidObjectStorePath(path.clone_as_boxstr()))?;

        create_dir_all(parent_dirs).map_err(|source| {
            InputManifestError::CantWriteManifestDir(
                parent_dirs.clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        write(&path, manifest.as_bytes()).map_err(|source| {
            InputManifestError::CantWriteManifest(path.clone_as_boxstr(), Box::new(source))
        })?;

        if let Some(target_aid) = manifest.target() {
            self.update_manifest_target(manifest_aid, target_aid)?;
        }

        Ok(manifest_aid)
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError> {
        let target_index = self.target_index()?;

        self.manifests()
            .map(|entry: ManifestsEntry<H>| {
                enrich_manifest_with_target(&target_index, entry.manifest()?)
            })
            .collect()
    }

    fn get_manifest(
        &self,
        matcher: Match<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        match matcher {
            Match::Target(artifact_id) => self.get_manifest_for_target(artifact_id),
            Match::Manifest(artifact_id) => self.get_manifest_with_id(artifact_id),
        }
    }

    fn update_manifest_target(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), InputManifestError> {
        self.target_index()?
            .start_upsert()
            .set_manifest_aid(manifest_aid)
            .set_target_aid(target_aid)
            .run_upsert()
    }

    fn remove_manifest(
        &mut self,
        matcher: Match<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        match matcher {
            Match::Target(artifact_id) => self.remove_manifest_for_target(artifact_id),
            Match::Manifest(artifact_id) => self.remove_manifest_with_id(artifact_id),
        }
    }
}

fn artifact_id_from_dir_entry<H: HashAlgorithm>(entry: &DirEntry) -> Option<ArtifactId<H>> {
    let gitoid_url = {
        let path_components = entry
            .path()
            .components()
            .map(|comp| comp.as_os_str().to_str())
            .collect::<Option<Vec<_>>>()?;

        let n_components = path_components.len();
        let remainder = path_components.get(n_components - 1)?;
        let prefix = path_components.get(n_components - 2)?;
        let meta = path_components.get(n_components - 3)?;
        let hash = format!("{prefix}{remainder}");
        let front = meta.replace('_', ":");
        format!("{front}:{hash}")
    };

    ArtifactId::<H>::from_str(&gitoid_url).ok()
}

/// An entry when iterating over manifests in the manifest store.
struct ManifestsEntry<H: HashAlgorithm> {
    /// The [`ArtifactId`] of the target artifact.
    target_aid: Option<ArtifactId<H>>,

    /// The path to the manifest in the manifest store.
    manifest_path: PathBuf,
}

impl<H: HashAlgorithm> ManifestsEntry<H> {
    /// Load the [`InputManifest`] represented by this entry.
    fn manifest(&self) -> Result<InputManifest<H>, InputManifestError> {
        match self.target_aid {
            Some(target) => InputManifest::load(&self.manifest_path, target),
            None => InputManifest::load_detached(&self.manifest_path),
        }
    }
}

/// Represents the target index file on disk.
struct TargetIndex {
    path: PathBuf,
}

impl TargetIndex {
    /// Create a new [`TargetIndex`]
    fn new(path: impl AsRef<Path>) -> Result<Self, InputManifestError> {
        let path = path.as_ref();

        if path.exists().not() {
            File::create_new(path).map_err(|source| {
                InputManifestError::CantCreateTargetIndex(path.clone_as_boxstr(), Box::new(source))
            })?;
        }

        Ok(TargetIndex {
            path: path.to_owned(),
        })
    }

    /// Find an entry for a specific manifest [`ArtifactId`].
    fn find<H: HashAlgorithm>(
        &self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<Option<ArtifactId<H>>, InputManifestError> {
        let file = File::open(&self.path).map_err(|source| {
            InputManifestError::CantOpenTargetIndex(self.path.clone_as_boxstr(), Box::new(source))
        })?;

        let reader = BufReader::new(&file);

        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|source| InputManifestError::CantReadTargetIndexLine {
                line_no: idx,
                source: Box::new(source),
            })?;

            let (line_manifest_aid, line_target_aid) = match line.split_once(' ') {
                Some(pair) => pair,
                None => return Err(InputManifestError::TargetIndexMalformedEntry { line_no: idx }),
            };

            let line_manifest_aid = ArtifactId::from_str(line_manifest_aid)?;

            if line_manifest_aid == manifest_aid {
                return Ok(ArtifactId::from_str(line_target_aid).ok());
            }
        }

        Ok(None)
    }

    /// Begin a removal operation in the [`TargetIndex`].
    fn start_remove<H: HashAlgorithm>(&self) -> TargetIndexRemove<H> {
        let root = self.path.parent().unwrap();
        TargetIndexRemove::new(root)
    }

    // Begin an "upsert" operation in the [`TargetIndex`].
    //
    // This either updates or inserts, as appropriate, into the index.
    fn start_upsert<H: HashAlgorithm>(&self) -> TargetIndexUpsert<H> {
        let root = self.path.parent().unwrap();
        TargetIndexUpsert::new(root)
    }
}

struct TargetIndexRemove<H: HashAlgorithm> {
    root: PathBuf,
    manifest_aid: Option<ArtifactId<H>>,
    target_aid: Option<ArtifactId<H>>,
}

impl<H: HashAlgorithm> TargetIndexRemove<H> {
    fn new(root: impl AsRef<Path>) -> Self {
        TargetIndexRemove {
            root: root.as_ref().to_owned(),
            manifest_aid: None,
            target_aid: None,
        }
    }

    /// Set the manifest [`ArtifactId`] for the upsert.
    fn set_manifest_aid(mut self, manifest_aid: ArtifactId<H>) -> Self {
        self.manifest_aid = Some(manifest_aid);
        self
    }

    /// Set the target [`ArtifactId`] for the upsert.
    fn set_target_aid(mut self, target_aid: ArtifactId<H>) -> Self {
        self.target_aid = Some(target_aid);
        self
    }

    /// Get the path to a temporary file used during upserting.
    fn tempfile(&self) -> PathBuf {
        pathbuf![&self.root, "targets.temp"]
    }

    fn target_file(&self) -> PathBuf {
        pathbuf![&self.root, "targets"]
    }

    /// Run the removal operation, removing based on matching the set criteria.
    ///
    /// This returns the Artifact ID of the removed [`InputManifest`] to the caller.
    fn run_remove(self) -> Result<ArtifactId<H>, InputManifestError> {
        if matches!((self.manifest_aid, self.target_aid), (None, None)) {
            return Err(InputManifestError::MissingTargetIndexRemoveCriteria);
        }

        let file = File::open(self.target_file()).map_err(|source| {
            InputManifestError::CantOpenTargetIndex(
                self.target_file().clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        // Read the current target index from disk.
        let mut target_index = HashMap::new();

        for (idx, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|source| InputManifestError::CantReadTargetIndexLine {
                line_no: idx,
                source: Box::new(source),
            })?;

            let (line_manifest_aid, line_target_aid) = line
                .split_once(' ')
                .ok_or(InputManifestError::TargetIndexMalformedEntry { line_no: idx })?;

            let line_manifest_aid = ArtifactId::from_str(line_manifest_aid)?;

            let line_target_aid = ArtifactId::from_str(line_target_aid)?;

            target_index.insert(line_manifest_aid, line_target_aid);
        }

        // Update the index in memory.
        let entry_to_remove: ArtifactId<H> = *target_index
            .iter()
            .find(|(entry_manifest_aid, entry_target_aid)| {
                match (self.target_aid, self.manifest_aid) {
                    (None, None) => unreachable!(
                        "target_aid and manifest_aid will always have at least one set"
                    ),
                    (None, Some(manifest_aid)) => **entry_manifest_aid == manifest_aid,
                    (Some(target_aid), None) => **entry_target_aid == target_aid,
                    (Some(target_aid), Some(manifest_aid)) => {
                        (**entry_manifest_aid == manifest_aid) && (**entry_target_aid == target_aid)
                    }
                }
            })
            .ok_or(InputManifestError::NoManifestFoundToRemove)?
            .0;

        // PANIC SAFETY: We know the key is there; we just found it.
        let removed_manifest = target_index.remove(&entry_to_remove).unwrap();

        // Write out updated index to a tempfile.
        let mut tempfile = File::create(self.tempfile()).map_err(|source| {
            InputManifestError::CantOpenTargetIndexTemp(
                self.tempfile().clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        // Write the updated in-memory data to the tempfile.
        let mut writer = BufWriter::new(&mut tempfile);
        for (manifest_aid, target_aid) in target_index {
            if let Err(source) = writeln!(writer, "{manifest_aid} {target_aid}") {
                fs::remove_file(self.tempfile()).map_err(|source| {
                    InputManifestError::CantDeleteTargetIndexTemp(
                        self.tempfile().clone_as_boxstr(),
                        Box::new(source),
                    )
                })?;

                return Err(InputManifestError::CantWriteTargetIndexTemp(
                    self.tempfile().clone_as_boxstr(),
                    Box::new(source),
                ));
            }
        }

        // Replace the prior index with the new one.
        if let Err(source) = fs::rename(self.tempfile(), self.target_file()) {
            fs::remove_dir(self.tempfile()).map_err(|source| {
                InputManifestError::CantDeleteTargetIndexTemp(
                    self.tempfile().clone_as_boxstr(),
                    Box::new(source),
                )
            })?;

            return Err(InputManifestError::CantReplaceTargetIndexWithTemp {
                temp: self.tempfile().clone_as_boxstr(),
                index: self.target_file().clone_as_boxstr(),
                source: Box::new(source),
            });
        }

        Ok(removed_manifest)
    }
}

struct TargetIndexUpsert<H: HashAlgorithm> {
    root: PathBuf,
    manifest_aid: Option<ArtifactId<H>>,
    target_aid: Option<ArtifactId<H>>,
}

impl<H: HashAlgorithm> TargetIndexUpsert<H> {
    /// Start a new upsert operation.
    fn new(root: impl AsRef<Path>) -> Self {
        TargetIndexUpsert {
            root: root.as_ref().to_owned(),
            manifest_aid: None,
            target_aid: None,
        }
    }

    /// Set the manifest [`ArtifactId`] for the upsert.
    fn set_manifest_aid(mut self, manifest_aid: ArtifactId<H>) -> Self {
        self.manifest_aid = Some(manifest_aid);
        self
    }

    /// Set the target [`ArtifactId`] for the upsert.
    fn set_target_aid(mut self, target_aid: ArtifactId<H>) -> Self {
        self.target_aid = Some(target_aid);
        self
    }

    /// Get the path to a temporary file used during upserting.
    fn tempfile(&self) -> PathBuf {
        pathbuf![&self.root, "targets.temp"]
    }

    fn target_file(&self) -> PathBuf {
        pathbuf![&self.root, "targets"]
    }

    /// Run the upsert operation.
    fn run_upsert(self) -> Result<(), InputManifestError> {
        let manifest_aid = self
            .manifest_aid
            .ok_or(InputManifestError::InvalidTargetIndexUpsert)?;
        let target_aid = self
            .target_aid
            .ok_or(InputManifestError::InvalidTargetIndexUpsert)?;

        let file = File::open(self.target_file()).map_err(|source| {
            InputManifestError::CantOpenTargetIndex(
                self.target_file().clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        // Read the current target index from disk.
        let mut target_index = HashMap::new();

        for (idx, line) in BufReader::new(file).lines().enumerate() {
            let line = line.map_err(|source| InputManifestError::CantReadTargetIndexLine {
                line_no: idx,
                source: Box::new(source),
            })?;

            let (line_manifest_aid, line_target_aid) = line
                .split_once(' ')
                .ok_or(InputManifestError::TargetIndexMalformedEntry { line_no: idx })?;

            let line_manifest_aid = ArtifactId::from_str(line_manifest_aid)?;

            let line_target_aid = ArtifactId::from_str(line_target_aid)?;

            target_index.insert(line_manifest_aid, line_target_aid);
        }

        // Update the index in memory.
        target_index
            .entry(manifest_aid)
            .and_modify(|old_target_aid| *old_target_aid = target_aid)
            .or_insert(target_aid);

        // Write out updated index to a tempfile.
        let mut tempfile = File::create(self.tempfile()).map_err(|source| {
            InputManifestError::CantOpenTargetIndexTemp(
                self.tempfile().clone_as_boxstr(),
                Box::new(source),
            )
        })?;

        let mut writer = BufWriter::new(&mut tempfile);
        for (manifest_aid, target_aid) in target_index {
            if let Err(source) = writeln!(writer, "{manifest_aid} {target_aid}") {
                fs::remove_file(self.tempfile()).map_err(|source| {
                    InputManifestError::CantDeleteTargetIndexTemp(
                        self.tempfile().clone_as_boxstr(),
                        Box::new(source),
                    )
                })?;

                return Err(InputManifestError::CantWriteTargetIndexTemp(
                    self.tempfile().clone_as_boxstr(),
                    Box::new(source),
                ));
            }
        }

        // Replace the prior index with the new one.
        if let Err(source) = fs::rename(self.tempfile(), self.target_file()) {
            fs::remove_dir(self.tempfile()).map_err(|source| {
                InputManifestError::CantDeleteTargetIndexTemp(
                    self.tempfile().clone_as_boxstr(),
                    Box::new(source),
                )
            })?;

            return Err(InputManifestError::CantReplaceTargetIndexWithTemp {
                temp: self.tempfile().clone_as_boxstr(),
                index: self.target_file().clone_as_boxstr(),
                source: Box::new(source),
            });
        }

        Ok(())
    }
}
