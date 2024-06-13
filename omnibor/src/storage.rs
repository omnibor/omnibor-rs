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
    fn has_manifest_for_artifact(&self, aid: ArtifactId<H>) -> bool;

    /// Get the manifest for a specific artifact.
    fn get_manifest_for_artifact(&self, aid: ArtifactId<H>) -> Option<InputManifest<H>>;

    /// Get the ID of the manifest for the artifact.
    fn get_manifest_id_for_artifact(&self, _aid: ArtifactId<H>) -> Option<ArtifactId<H>>;

    /// Write a manifest to the storage.
    fn write_manifest(&mut self, manifest: &InputManifest<H>) -> Result<ArtifactId<H>>;

    /// Update the manifest file to reflect the target ID.
    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<()>;
}

/// File system storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct FileSystemStorage;

impl<H: SupportedHash> Storage<H> for FileSystemStorage {
    fn has_manifest_for_artifact(&self, _aid: ArtifactId<H>) -> bool {
        todo!()
    }

    fn get_manifest_for_artifact(&self, _aid: ArtifactId<H>) -> Option<InputManifest<H>> {
        todo!()
    }

    fn get_manifest_id_for_artifact(&self, _aid: ArtifactId<H>) -> Option<ArtifactId<H>> {
        todo!()
    }

    /// Write a manifest to the storage.
    fn write_manifest(&mut self, _manifest: &InputManifest<H>) -> Result<ArtifactId<H>> {
        todo!()
    }

    /// Update the manifest file to reflect the safe_name version of the target ID.
    fn update_target_for_manifest(
        &mut self,
        _manifest_aid: ArtifactId<H>,
        _target_aid: ArtifactId<H>,
    ) -> Result<()> {
        todo!()
    }
}

/// In-memory storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct InMemoryStorage {
    sha256_manifests: Vec<ManifestEntry<Sha256>>,
}

struct ManifestEntry<H: SupportedHash> {
    aid: ArtifactId<H>,
    manifest: InputManifest<H>,
}

impl<H: SupportedHash> Debug for ManifestEntry<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestEntry")
            .field("aid", &self.aid)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: SupportedHash> Clone for ManifestEntry<H> {
    fn clone(&self) -> Self {
        ManifestEntry {
            aid: self.aid,
            manifest: self.manifest.clone(),
        }
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            sha256_manifests: Vec::new(),
        }
    }

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
        // Create an in-memory buffer we can treat like a file.
        // It's capped at 50 KiB, which _should_ be plenty, and it's thrown away once the
        // function call is done.
        //
        // We do this because the in-memory representation of the `InputManifest` is not guaranteed
        // by Rust, but the written-out format of it is something _we_ guarantee, so we can rely on
        // it here.
        //
        // We also truncate the slice to only the non-zero bytes to avoid considering the total
        // capacity of the buffer when producing the hash.
        let manifest_aid = {
            let mut bytes = [0; 50_000];
            manifest.write_to(&mut bytes[..])?;
            let len = bytes.iter().take_while(|b| **b != 0).count();
            ArtifactId::<Sha256>::id_bytes(&bytes[..len])
        };

        self.sha256_manifests.push(ManifestEntry {
            aid: manifest_aid,
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
            .find(|entry| entry.aid == manifest_aid)
            .map(|entry| entry.manifest.set_target(target_aid));

        Ok(())
    }
}
