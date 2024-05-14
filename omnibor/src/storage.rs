use crate::{ArtifactId, InputManifest, Result, SupportedHash};

/// Represents the interface for storing and querying manifests.
pub trait Storage {
    /// Check if we have the manifest for a specific artifact.
    fn has_manifest_for_artifact<H: SupportedHash>(&self, aid: ArtifactId<H>) -> bool;

    /// Get the manifest for a specific artifact.
    fn get_manifest_for_artifact<H: SupportedHash>(
        &self,
        aid: ArtifactId<H>,
    ) -> Option<InputManifest<H>>;

    /// Get the ID of the manifest for the artifact.
    fn get_manifest_id_for_artifact<H: SupportedHash>(
        &self,
        _aid: ArtifactId<H>,
    ) -> Option<ArtifactId<H>> {
        todo!()
    }

    /// Write a manifest to the storage.
    fn write_manifest<H: SupportedHash>(
        &self,
        target: Option<ArtifactId<H>>,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>>;

    /// Update the manifest file to reflect the safe_name version of the target ID.
    fn update_target_for_manifest<H: SupportedHash>(
        &self,
        manifest_aid: ArtifactId<H>,
        target_aid: ArtifactId<H>,
    ) -> Result<()>;
}

/// File system storage for [`InputManifest`]s.
#[derive(Debug)]
pub struct FileSystemStorage;

impl Storage for FileSystemStorage {
    fn has_manifest_for_artifact<H: SupportedHash>(&self, _aid: ArtifactId<H>) -> bool {
        todo!()
    }

    fn get_manifest_for_artifact<H: SupportedHash>(
        &self,
        _aid: ArtifactId<H>,
    ) -> Option<InputManifest<H>> {
        todo!()
    }

    fn get_manifest_id_for_artifact<H: SupportedHash>(
        &self,
        _aid: ArtifactId<H>,
    ) -> Option<ArtifactId<H>> {
        todo!()
    }

    /// Write a manifest to the storage.
    fn write_manifest<H: SupportedHash>(
        &self,
        _target: Option<ArtifactId<H>>,
        _manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>> {
        todo!()
    }

    /// Update the manifest file to reflect the safe_name version of the target ID.
    fn update_target_for_manifest<H: SupportedHash>(
        &self,
        _manifest_aid: ArtifactId<H>,
        _target_aid: ArtifactId<H>,
    ) -> Result<()> {
        todo!()
    }
}
