//! How manifests are stored and accessed.
//!
//! __See [Storing Input Manifests][idx] documentation for more info.__
//!
//! [idx]: crate#storing-input-manifests

pub(crate) mod file_system_storage;
pub(crate) mod in_memory_storage;
#[cfg(test)]
mod test;

pub use crate::storage::file_system_storage::FileSystemStorage;
pub use crate::storage::in_memory_storage::InMemoryStorage;

use crate::{
    artifact_id::ArtifactId, error::InputManifestError, hash_algorithm::HashAlgorithm,
    input_manifest::InputManifest,
};

/// Represents the interface for storing and querying manifests.
pub trait Storage<H: HashAlgorithm> {
    /// Get all manifests from the storage.
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError>;

    /// Get the manifest for a specific target artifact.
    fn get_manifest_for_target(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>;

    /// Get a manifest by its Artifact ID.
    fn get_manifest_with_id(
        &self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>;

    /// Write a manifest to the storage.
    ///
    /// If the manifest has a target attached, update any indices.
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError>;

    /// Update the manifest file to reflect the target ID.
    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), InputManifestError>;

    /// Remove the manifest for the target artifact.
    ///
    /// Returns the manifest to the caller.
    fn remove_manifest_for_target(
        &mut self,
        target_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError>;

    /// Remove a manifest by its Artifact ID.
    ///
    /// Returns the manifest to the caller.
    fn remove_manifest_with_id(
        &mut self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError>;
}

impl<H: HashAlgorithm, S: Storage<H>> Storage<H> for &mut S {
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError> {
        (**self).get_manifests()
    }

    fn get_manifest_for_target(
        &self,
        target_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        (**self).get_manifest_for_target(target_aid)
    }

    fn get_manifest_with_id(
        &self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        (**self).get_manifest_with_id(manifest_aid)
    }

    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError> {
        (**self).write_manifest(manifest)
    }

    fn update_target_for_manifest(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), InputManifestError> {
        (**self).update_target_for_manifest(manifest_aid, target_aid)
    }

    /// Remove the manifest for the target artifact.
    ///
    /// Returns the manifest to the caller.
    fn remove_manifest_for_target(
        &mut self,
        target_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        (**self).remove_manifest_for_target(target_aid)
    }

    /// Remove a manifest by its Artifact ID.
    ///
    /// Returns the manifest to the caller.
    fn remove_manifest_with_id(
        &mut self,
        manifest_aid: ArtifactId<H>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        (**self).remove_manifest_with_id(manifest_aid)
    }
}
