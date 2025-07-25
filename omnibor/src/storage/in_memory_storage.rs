use {
    crate::{
        artifact_id::ArtifactId,
        error::InputManifestError,
        hash_algorithm::{HashAlgorithm, Sha256},
        input_manifest::InputManifest,
        storage::{query::Match, Storage},
        Identify,
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
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError> {
        let possible_position = self
            .sha256_manifests
            .iter()
            .position(|entry| entry.manifest.target() == Some(target_aid));

        let pos = match possible_position {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let manifest = self.sha256_manifests.remove(pos).manifest;

        Ok(Some(manifest))
    }

    fn remove_manifest_with_id(
        &mut self,
        manifest_aid: ArtifactId<Sha256>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError> {
        let possible_position = self
            .sha256_manifests
            .iter()
            .position(|entry| entry.manifest_aid == manifest_aid);

        let pos = match possible_position {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let manifest = self.sha256_manifests.remove(pos).manifest;

        Ok(Some(manifest))
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

    fn get_manifest<I>(
        &self,
        matcher: Match<Sha256, I>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError>
    where
        I: Identify<Sha256>,
    {
        match matcher {
            Match::Target(matcher) => self.get_manifest_for_target(matcher.id()?),
            Match::Manifest(matcher) => self.get_manifest_with_id(matcher.id()?),
        }
    }

    fn update_manifest_target<I1, I2>(
        &mut self,
        manifest_aid: I1,
        target_aid: I2,
    ) -> Result<(), InputManifestError>
    where
        I1: Identify<Sha256>,
        I2: Identify<Sha256>,
    {
        let manifest_aid = manifest_aid.identify()?;
        let target_aid = target_aid.identify()?;

        self.sha256_manifests
            .iter_mut()
            .find(|entry| entry.manifest_aid == manifest_aid)
            .map(|entry| entry.manifest.set_target(Some(target_aid)));

        Ok(())
    }

    fn remove_manifest<I>(
        &mut self,
        matcher: Match<Sha256, I>,
    ) -> Result<Option<InputManifest<Sha256>>, InputManifestError>
    where
        I: Identify<Sha256>,
    {
        match matcher {
            Match::Target(matcher) => self.remove_manifest_for_target(matcher.id()?),
            Match::Manifest(matcher) => self.remove_manifest_with_id(matcher.id()?),
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
