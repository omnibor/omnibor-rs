//! Defines how manifests are stored and accessed.

use crate::{hashes::SupportedHash, supported_hash::Sha256, ArtifactId, InputManifest, Result};
use std::{mem::size_of, slice::from_raw_parts};

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

    /// Update the manifest file to reflect the safe_name version of the target ID.
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
    sha256_manifests: Vec<(ArtifactId<Sha256>, InputManifest<Sha256>)>,
}

impl Storage<Sha256> for InMemoryStorage {
    fn has_manifest_for_artifact(&self, target_aid: ArtifactId<Sha256>) -> bool {
        self.sha256_manifests
            .iter()
            .any(|manifest| manifest.1.target() == Some(target_aid))
    }

    fn get_manifest_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Option<InputManifest<Sha256>> {
        self.sha256_manifests
            .iter()
            .find(|manifest| manifest.1.target() == Some(target_aid))
            .cloned()
            .map(|pair| pair.1)
    }

    fn get_manifest_id_for_artifact(
        &self,
        target_aid: ArtifactId<Sha256>,
    ) -> Option<ArtifactId<Sha256>> {
        self.sha256_manifests
            .iter()
            .find(|manifest| manifest.1.target() == Some(target_aid))
            .and_then(|manifest| manifest.1.target())
    }

    fn write_manifest(&mut self, manifest: &InputManifest<Sha256>) -> Result<ArtifactId<Sha256>> {
        let content = get_bytes(manifest);
        let manifest_aid = ArtifactId::<Sha256>::id_bytes(content);
        self.sha256_manifests.push((manifest_aid, manifest.clone()));
        Ok(manifest_aid)
    }

    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<Sha256>,
        target_aid: ArtifactId<Sha256>,
    ) -> Result<()> {
        self.sha256_manifests
            .iter_mut()
            .find(|manifest| manifest.0 == manifest_aid)
            .map(|manifest| manifest.1.set_target(target_aid));
        Ok(())
    }
}

/// Get the byte representation of a value in memory.
fn get_bytes<T>(input: &T) -> &[u8] {
    unsafe { from_raw_parts(input as *const _ as *const u8, size_of::<T>()) }
}
