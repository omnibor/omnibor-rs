//! Store and retrieve `InputManifest`s.
//!
//! The "Store" is an interface for types that store and enable querying of
//! Input Manifests. They exist in particular to support things like filling
//! in manifest information for build inputs during Input Manifest construction,
//! and to ensure (in the case of `FileSystemStorage`) that Input Manifests
//! are persisted to disk in a manner consistent with the OmniBOR specification.
//!
//! [__See Storage documentation for more info.__][idx]
//!
//! [idx]: crate#storage

pub(crate) mod file_system_storage;
pub(crate) mod in_memory_storage;
pub(crate) mod query;
#[cfg(test)]
mod test;

pub use crate::storage::file_system_storage::FileSystemStorage;
pub use crate::storage::in_memory_storage::InMemoryStorage;
pub use crate::storage::query::Match;

use crate::{
    artifact_id::ArtifactId, error::InputManifestError, hash_algorithm::HashAlgorithm,
    input_manifest::InputManifest,
};

/// Represents the interface for storing and querying manifests.
pub trait Storage<H: HashAlgorithm> {
    /// Write a manifest to the storage.
    ///
    /// If the manifest has a target attached, update any indices.
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError>;

    /// Get all manifests from the storage.
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError>;

    /// Get a manifest by matching on the criteria.
    ///
    /// Returns `Ok(None)` if no match was found. Returns the manifest if found.
    /// Returns an error otherwise.
    fn get_manifest(
        &self,
        matcher: Match<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>;

    /// Update the manifest file to reflect the target ID.
    fn update_manifest_target(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), InputManifestError>;

    /// Remove the manifest for the target artifact.
    ///
    /// Returns the manifest to the caller, if found. Returns `Ok(None)` if no
    /// errors were encountered but the manifest was not found in storage.
    /// Returns errors otherwise.
    fn remove_manifest(
        &mut self,
        matcher: Match<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>;
}

impl<H: HashAlgorithm, S: Storage<H>> Storage<H> for &mut S {
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError> {
        (**self).write_manifest(manifest)
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError> {
        (**self).get_manifests()
    }

    fn get_manifest(
        &self,
        matcher: Match<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        (**self).get_manifest(matcher)
    }

    fn update_manifest_target(
        &mut self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<(), InputManifestError> {
        (**self).update_manifest_target(manifest_aid, target_aid)
    }

    fn remove_manifest(
        &mut self,
        matcher: Match<H>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError> {
        (**self).remove_manifest(matcher)
    }
}
