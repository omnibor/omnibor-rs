//! Defines how manifests are stored and accessed.

use crate::hashes::SupportedHash;
use crate::supported_hash::Sha256;
use crate::ArtifactId;
use crate::InputManifest;
use crate::Result;
use std::fmt::Debug;

/// Represents the interface for storing and querying manifests.
pub trait Storage<H: SupportedHash> {
    /// Check if we have the manifest for a specific artifact.
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<H>) -> bool;

    /// Get the manifest for a specific artifact.
    fn get_manifest_for_artifact(&self, target_aid: ArtifactId<H>) -> Option<InputManifest<H>>;

    /// Get the ID of the manifest for the artifact.
    fn get_manifest_id_for_artifact(&self, target_aid: ArtifactId<H>) -> Option<ArtifactId<H>>;

    /// Write a manifest to the storage.
    fn write_manifest(&mut self, manifest: &InputManifest<H>) -> Result<ArtifactId<H>>;

    /// Update the manifest file to reflect the target ID.
    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<()>;

    /// Get all manifests from the storage.
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>>;
}

/// File system storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct FileSystemStorage;

impl<H: SupportedHash> Storage<H> for FileSystemStorage {
    fn has_manifest_for_artifact(&self, _aid: ArtifactId<H>) -> bool {
        todo!("file system storage is not yet implemented")
    }

    fn get_manifest_for_artifact(&self, _aid: ArtifactId<H>) -> Option<InputManifest<H>> {
        todo!("file system storage is not yet implemented")
    }

    fn get_manifest_id_for_artifact(&self, _aid: ArtifactId<H>) -> Option<ArtifactId<H>> {
        todo!("file system storage is not yet implemented")
    }

    fn write_manifest(&mut self, _manifest: &InputManifest<H>) -> Result<ArtifactId<H>> {
        todo!("file system storage is not yet implemented")
    }

    fn update_target_for_manifest(
        &mut self,
        _manifest_aid: ArtifactId<H>,
        _target_aid: ArtifactId<H>,
    ) -> Result<()> {
        todo!("file system storage is not yet implemented")
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>> {
        todo!("file system storage is not yet implemented")
    }
}

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
        InMemoryStorage::default()
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

impl Storage<Sha256> for InMemoryStorage {
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<Sha256>) -> bool {
        self.match_by_target_aid(target_aid).is_some()
    }

    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Option<InputManifest<Sha256>> {
        self.match_by_target_aid(target_aid)
            .map(|entry| entry.manifest.clone())
    }

    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Option<ArtifactId<Sha256>> {
        self.match_by_target_aid(target_aid)
            .and_then(|entry| entry.manifest.target())
    }

    fn write_manifest(&mut self, manifest: &InputManifest<Sha256>) -> Result<ArtifactId<Sha256>> {
        let manifest_aid = ArtifactId::<Sha256>::id_bytes(manifest.as_bytes()?);

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
    ) -> Result<()> {
        self.sha256_manifests
            .iter_mut()
            .find(|entry| entry.manifest_aid == manifest_aid)
            .map(|entry| entry.manifest.set_target(target_aid));

        Ok(())
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<Sha256>>> {
        Ok(self
            .sha256_manifests
            .iter()
            .map(|entry| entry.manifest.clone())
            .collect())
    }
}

/// An entry in the in-memory manifest storage.
struct ManifestEntry<H: SupportedHash> {
    /// The [`ArtifactId`] of the manifest.
    manifest_aid: ArtifactId<H>,

    /// The manifest itself.
    manifest: InputManifest<H>,
}

impl<H: SupportedHash> Debug for ManifestEntry<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestEntry")
            .field("manifest_aid", &self.manifest_aid)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: SupportedHash> Clone for ManifestEntry<H> {
    fn clone(&self) -> Self {
        ManifestEntry {
            manifest_aid: self.manifest_aid,
            manifest: self.manifest.clone(),
        }
    }
}
