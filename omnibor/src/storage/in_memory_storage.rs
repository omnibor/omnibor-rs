use {
    crate::{
        artifact_id::ArtifactId,
        error::InputManifestError,
        hash_algorithm::{HashAlgorithm, Sha256},
        input_manifest::InputManifest,
        storage::{query::Match, Storage},
    },
    std::fmt::Debug,
};

/// In-memory storage for [`InputManifest`]s.
///
/// Note that this "storage" doesn't persist anything. We use it for testing, and it
/// may be useful in other applications where you only care about producing and using
/// manifests in the short-term, and not in persisting them to a disk or some other
/// durable location.
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    /// Stored SHA-256 [`InputManifest`]s.
    sha256_manifests: Vec<ManifestEntry<Sha256>>,
}

impl InMemoryStorage {
    /// Construct a new `InMemoryStorage` instance.
    pub fn new() -> Self {
        Self {
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

    fn match_by_manifest_aid(
        &self,
        manifest_aid: ArtifactId<Sha256>,
    ) -> Option<&ManifestEntry<Sha256>> {
        self.sha256_manifests
            .iter()
            .find(|entry| entry.manifest_aid == manifest_aid)
    }

    fn get_manifest_for_target(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError> {
        Ok(self
            .match_by_target_aid(target_aid)
            .map(|entry| entry.manifest.clone()))
    }

    fn get_manifest_with_id(
        &self,
        manifest_aid: ArtifactId<Sha256>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError> {
        Ok(self
            .match_by_manifest_aid(manifest_aid)
            .map(|entry| entry.manifest.clone()))
    }

    fn remove_manifest_for_target(
        &mut self,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<InputManifest<Sha256>, InputManifestError> {
        let pos = self
            .sha256_manifests
            .iter()
            .position(|entry| entry.manifest.target() == Some(target_aid))
            .ok_or_else(|| {
                InputManifestError::CantFindManifestForTarget(
                    target_aid.to_string().into_boxed_str(),
                )
            })?;

        let manifest = self.sha256_manifests.remove(pos).manifest;

        Ok(manifest)
    }

    fn remove_manifest_with_id(
        &mut self,
        manifest_aid: ArtifactId<Sha256>,
    ) -> Result<InputManifest<Sha256>, InputManifestError> {
        let pos = self
            .sha256_manifests
            .iter()
            .position(|entry| entry.manifest_aid == manifest_aid)
            .ok_or_else(|| {
                InputManifestError::CantFindManifestWithId(
                    manifest_aid.to_string().into_boxed_str(),
                )
            })?;

        let manifest = self.sha256_manifests.remove(pos).manifest;

        Ok(manifest)
    }
}

impl Storage<Sha256> for InMemoryStorage {
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<Sha256>,
    ) -> Result<ArtifactId<Sha256>, InputManifestError> {
        // SAFETY: Identifying a manifest is infallible.
        let manifest_aid = ArtifactId::new(manifest).unwrap();

        self.sha256_manifests.push(ManifestEntry {
            manifest_aid,
            manifest: manifest.clone(),
        });

        Ok(manifest_aid)
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<Sha256>>, InputManifestError> {
        Ok(self
            .sha256_manifests
            .iter()
            .map(|entry| entry.manifest.clone())
            .collect())
    }

    fn get_manifest(
        &self,
        matcher: Match<Sha256>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError> {
        match matcher {
            Match::Target(artifact_id) => self.get_manifest_for_target(artifact_id),
            Match::Manifest(artifact_id) => self.get_manifest_with_id(artifact_id),
        }
    }

    fn update_manifest_target(
        &mut self,
        manifest_aid: ArtifactId<Sha256>,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<(), InputManifestError> {
        self.sha256_manifests
            .iter_mut()
            .find(|entry| entry.manifest_aid == manifest_aid)
            .map(|entry| entry.manifest.set_target(Some(target_aid)));

        Ok(())
    }

    fn remove_manifest(
        &mut self,
        matcher: Match<Sha256>,
    ) -> Result<InputManifest<Sha256>, InputManifestError> {
        match matcher {
            Match::Target(artifact_id) => self.remove_manifest_for_target(artifact_id),
            Match::Manifest(artifact_id) => self.remove_manifest_with_id(artifact_id),
        }
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
