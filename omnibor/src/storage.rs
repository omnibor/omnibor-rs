//! How manifests are stored and accessed.

use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        error::Error,
        hash_algorithm::{HashAlgorithm, Sha256},
        hash_provider::HashProvider,
        input_manifest::InputManifest,
        pathbuf,
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
    tracing::{debug, info},
    walkdir::{DirEntry, WalkDir},
};

/// Represents the interface for storing and querying manifests.
pub trait Storage<H: HashAlgorithm> {
    /// Check if we have the manifest for a specific artifact.
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<H>) -> bool;

    /// Get the manifest for a specific artifact.
    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, Error>;

    /// Get the ID of the manifest for the artifact.
    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<ArtifactId<H>>, Error>;

    /// Write a manifest to the storage.
    fn write_manifest(&mut self, manifest: &InputManifest<H>) -> Result<ArtifactId<H>, Error>;

    /// Update the manifest file to reflect the target ID.
    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), Error>;

    /// Get all manifests from the storage.
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, Error>;
}

impl<H: HashAlgorithm, S: Storage<H>> Storage<H> for &mut S {
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<H>) -> bool {
        (**self).has_manifest_for_artifact(target_aid)
    }

    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, Error> {
        (**self).get_manifest_for_artifact(target_aid)
    }

    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<ArtifactId<H>>, Error> {
        (**self).get_manifest_id_for_artifact(target_aid)
    }

    fn write_manifest(&mut self, manifest: &InputManifest<H>) -> Result<ArtifactId<H>, Error> {
        (**self).write_manifest(manifest)
    }

    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), Error> {
        (**self).update_target_for_manifest(manifest_aid, target_aid)
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, Error> {
        (**self).get_manifests()
    }
}

/// File system storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct FileSystemStorage<H: HashAlgorithm, P: HashProvider<H>> {
    _hash_algorithm: PhantomData<H>,
    hash_provider: P,
    root: PathBuf,
}

impl<H: HashAlgorithm, P: HashProvider<H>> FileSystemStorage<H, P> {
    /// Start building a new [`FileSystemStorage`].
    pub fn new(hash_provider: P, root: impl AsRef<Path>) -> Result<FileSystemStorage<H, P>, Error> {
        let root = root.as_ref().to_owned();

        if root.exists() {
            let meta = fs::metadata(&root)
                .map_err(|e| Error::CantAccessRoot(root.display().to_string(), e))?;

            if meta.is_dir().not() {
                return Err(Error::ObjectStoreNotDir(root.display().to_string()));
            }
        } else {
            create_dir_all(&root)
                .map_err(|e| Error::CantCreateObjectStoreDir(root.display().to_string(), e))?;
        }

        Ok(FileSystemStorage {
            _hash_algorithm: PhantomData,
            hash_provider,
            root,
        })
    }

    /// Build a [`FileSystemStorage`] with a root set from
    /// the `OMNIBOR_DIR` environment variable.
    pub fn from_env(hash_provider: P) -> Result<FileSystemStorage<H, P>, Error> {
        var_os("OMNIBOR_DIR")
            .ok_or(Error::NoStorageRoot)
            .map(|root| FileSystemStorage {
                _hash_algorithm: PhantomData,
                hash_provider,
                root: PathBuf::from(root),
            })
    }

    /// Fully delete the contents of the root dir.
    ///
    /// This is just used for tests to ensure idempotency.
    #[cfg(test)]
    pub fn cleanup(self) -> Result<(), Error> {
        fs::remove_dir_all(&self.root)?;
        fs::create_dir_all(&self.root)?;
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
    fn target_index(&self) -> Result<TargetIndex, Error> {
        TargetIndex::new(self.target_file_path())
    }

    /// Get the path for storing a manifest with this [`ArtifactId`].
    fn manifest_path(&self, aid: ArtifactId<H>) -> PathBuf {
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
}

impl<H: HashAlgorithm, P: HashProvider<H>> Storage<H> for FileSystemStorage<H, P> {
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<H>) -> bool {
        self.manifests()
            .any(|entry| entry.target_aid == Some(target_aid))
    }

    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, Error> {
        match self
            .manifests()
            .find(|entry| entry.target_aid == Some(target_aid))
        {
            Some(entry) => entry.manifest().map(Some),
            None => Ok(None),
        }
    }

    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<ArtifactId<H>>, Error> {
        match self.get_manifest_for_artifact(target_aid) {
            Ok(Some(manifest)) => Ok(Some(
                ArtifactIdBuilder::with_provider(self.hash_provider).identify_manifest(&manifest),
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn write_manifest(&mut self, manifest: &InputManifest<H>) -> Result<ArtifactId<H>, Error> {
        let builder = ArtifactIdBuilder::with_provider(self.hash_provider);
        let manifest_aid = builder.identify_manifest(manifest);
        let path = self.manifest_path(manifest_aid);
        let parent_dirs = path
            .parent()
            .ok_or_else(|| Error::InvalidObjectStorePath(path.display().to_string()))?;

        create_dir_all(parent_dirs)
            .map_err(|e| Error::CantWriteManifestDir(parent_dirs.display().to_string(), e))?;

        write(&path, manifest.as_bytes())
            .map_err(|e| Error::CantWriteManifest(path.display().to_string(), e))?;

        info!("wrote manifest '{}' to store", manifest_aid);

        Ok(manifest_aid)
    }

    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), Error> {
        self.target_index()?
            .upsert()
            .manifest_aid(manifest_aid)
            .target_aid(target_aid)
            .run()
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, Error> {
        self.manifests()
            .map(|entry: ManifestsEntry<H>| InputManifest::from_path(&entry.manifest_path))
            .collect()
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
        let hash = format!("{}{}", prefix, remainder);
        let front = meta.replace('_', ":");
        format!("{}:{}", front, hash)
    };

    debug!(gitoid_url = %gitoid_url);

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
    fn manifest(&self) -> Result<InputManifest<H>, Error> {
        let mut manifest = InputManifest::from_path(&self.manifest_path)?;
        manifest.set_target(self.target_aid);
        Ok(manifest)
    }
}

/// Represents the target index file on disk.
struct TargetIndex {
    path: PathBuf,
}

impl TargetIndex {
    /// Create a new [`TargetIndex`]
    fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();

        if path.exists().not() {
            File::create_new(path)?;
        }

        Ok(TargetIndex {
            path: path.to_owned(),
        })
    }

    /// Find an entry for a specific manifest [`ArtifactId`].
    fn find<H: HashAlgorithm>(
        &self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<Option<ArtifactId<H>>, Error> {
        let file = File::open(&self.path)
            .map_err(|e| Error::CantOpenTargetIndex(self.path.display().to_string(), e))?;

        let reader = BufReader::new(&file);

        for line in reader.lines() {
            let line = line.map_err(Error::CorruptedTargetIndexIoReason)?;

            let (line_manifest_aid, line_target_aid) = match line.split_once(' ') {
                Some(pair) => pair,
                None => return Err(Error::CorruptedTargetIndex),
            };

            let line_manifest_aid = ArtifactId::from_str(line_manifest_aid)?;

            if line_manifest_aid == manifest_aid {
                return Ok(ArtifactId::from_str(line_target_aid).ok());
            }
        }

        Ok(None)
    }

    // Begin an "upsert" operation in the [`TargetIndex`].
    //
    // This either updates or inserts, as appropriate, into the index.
    fn upsert<H: HashAlgorithm>(&self) -> TargetIndexUpsert<H> {
        let root = self.path.parent().unwrap();
        TargetIndexUpsert::new(root)
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
    fn manifest_aid(mut self, manifest_aid: ArtifactId<H>) -> Self {
        self.manifest_aid = Some(manifest_aid);
        self
    }

    /// Set the target [`ArtifactId`] for the upsert.
    fn target_aid(mut self, target_aid: ArtifactId<H>) -> Self {
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
    fn run(self) -> Result<(), Error> {
        let manifest_aid = self.manifest_aid.ok_or(Error::InvalidTargetIndexUpsert)?;
        let target_aid = self.target_aid.ok_or(Error::InvalidTargetIndexUpsert)?;

        let file = File::open(self.target_file())
            .map_err(|e| Error::CantOpenTargetIndex(self.target_file().display().to_string(), e))?;

        // Read the current target index from disk.
        let mut target_index = HashMap::new();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(Error::CorruptedTargetIndexIoReason)?;

            let (line_manifest_aid, line_target_aid) =
                line.split_once(' ').ok_or(Error::CorruptedTargetIndex)?;

            let line_manifest_aid = ArtifactId::from_str(line_manifest_aid)
                .map_err(|e| Error::CorruptedTargetIndexOmniBorReason(Box::new(e)))?;

            let line_target_aid = ArtifactId::from_str(line_target_aid)
                .map_err(|e| Error::CorruptedTargetIndexOmniBorReason(Box::new(e)))?;

            target_index.insert(line_manifest_aid, line_target_aid);
        }

        // Update the index in memory.
        target_index
            .entry(manifest_aid)
            .and_modify(|old_target_aid| *old_target_aid = target_aid)
            .or_insert(target_aid);

        // Write out updated index to a tempfile.
        let mut tempfile = File::create(self.tempfile()).map_err(|e| {
            Error::CantOpenTargetIndexTemp(self.tempfile().display().to_string(), e)
        })?;

        let mut writer = BufWriter::new(&mut tempfile);
        for (manifest_aid, target_aid) in target_index {
            if let Err(e) = writeln!(writer, "{} {}", manifest_aid, target_aid) {
                fs::remove_file(self.tempfile()).map_err(|e| {
                    Error::CantDeleteTargetIndexTemp(self.tempfile().display().to_string(), e)
                })?;
                return Err(e.into());
            }
        }

        // Replace the prior index with the new one.
        if let Err(e) = fs::rename(self.tempfile(), self.target_file()) {
            fs::remove_dir(self.tempfile()).map_err(|e| {
                Error::CantDeleteTargetIndexTemp(self.tempfile().display().to_string(), e)
            })?;
            return Err(e.into());
        }

        Ok(())
    }
}

/// In-memory storage for [`InputManifest`]s.
///
/// Note that this "storage" doesn't persist anything. We use it for testing, and it
/// may be useful in other applications where you only care about producing and using
/// manifests in the short-term, and not in persisting them to a disk or some other
/// durable location.
#[derive(Debug)]
pub struct InMemoryStorage<P: HashProvider<Sha256>> {
    /// The cryptography library providing a hash implementation.
    hash_provider: P,

    /// Stored SHA-256 [`InputManifest`]s.
    sha256_manifests: Vec<ManifestEntry<Sha256>>,
}

impl<P: HashProvider<Sha256>> InMemoryStorage<P> {
    /// Construct a new `InMemoryStorage` instance.
    pub fn new(hash_provider: P) -> Self {
        Self {
            hash_provider,
            sha256_manifests: Vec::new(),
        }
    }

    /// Find the manifest entry that matches the target [`ArtifactId`]
    fn match_by_target_aid(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Option<&ManifestEntry<Sha256>> {
        self.sha256_manifests
            .iter()
            .find(|entry| entry.manifest.target() == Some(target_aid))
    }
}

impl<P: HashProvider<Sha256>> Storage<Sha256> for InMemoryStorage<P> {
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<Sha256>) -> bool {
        self.match_by_target_aid(target_aid).is_some()
    }

    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<Option<InputManifest<Sha256>>, Error> {
        Ok(self
            .match_by_target_aid(target_aid)
            .map(|entry| entry.manifest.clone()))
    }

    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<Option<ArtifactId<Sha256>>, Error> {
        Ok(self
            .match_by_target_aid(target_aid)
            .and_then(|entry| entry.manifest.target()))
    }

    fn write_manifest(
        &mut self,
        manifest: &InputManifest<Sha256>,
    ) -> Result<ArtifactId<Sha256>, Error> {
        let builder = ArtifactIdBuilder::with_provider(self.hash_provider);
        let manifest_aid = builder.identify_manifest(manifest);

        self.sha256_manifests.push(ManifestEntry {
            manifest_aid,
            manifest: manifest.clone(),
        });

        Ok(manifest_aid)
    }

    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<Sha256>,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<(), Error> {
        self.sha256_manifests
            .iter_mut()
            .find(|entry| entry.manifest_aid == manifest_aid)
            .map(|entry| entry.manifest.set_target(Some(target_aid)));

        Ok(())
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<Sha256>>, Error> {
        Ok(self
            .sha256_manifests
            .iter()
            .map(|entry| entry.manifest.clone())
            .collect())
    }
}

/// An entry in the in-memory manifest storage.
struct ManifestEntry<H: HashAlgorithm> {
    /// The [`ArtifactId`] of the manifest.
    manifest_aid: ArtifactId<H>,

    /// The manifest itself.
    manifest: InputManifest<H>,
}

impl<H: HashAlgorithm> Debug for ManifestEntry<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestEntry")
            .field("manifest_aid", &self.manifest_aid)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: HashAlgorithm> Clone for ManifestEntry<H> {
    fn clone(&self) -> Self {
        ManifestEntry {
            manifest_aid: self.manifest_aid,
            manifest: self.manifest.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileSystemStorage;
    use crate::{artifact_id::ArtifactId, hash_algorithm::Sha256, pathbuf};
    use std::str::FromStr;

    #[cfg(feature = "backend-rustcrypto")]
    #[test]
    fn correct_aid_storage_path() {
        use crate::hash_provider::RustCrypto;

        let root = pathbuf![env!("CARGO_MANIFEST_DIR"), "test", "fs_storage"];
        let storage = FileSystemStorage::new(RustCrypto::new(), &root).unwrap();

        let aid = ArtifactId::<Sha256>::from_str(
            "gitoid:blob:sha256:9d09789f20162dca6d80d2d884f46af22c824f6409d4f447332d079a2d1e364f",
        )
        .unwrap();

        let path = storage.manifest_path(aid);
        let path = path.strip_prefix(&root).unwrap();
        let expected = pathbuf![
            "manifests",
            "gitoid_blob_sha256",
            "9d",
            "09789f20162dca6d80d2d884f46af22c824f6409d4f447332d079a2d1e364f"
        ];

        assert_eq!(path, expected);
    }
}
