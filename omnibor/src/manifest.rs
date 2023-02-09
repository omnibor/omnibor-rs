use gitoid::GitOid;
use gitoid::HashAlgorithm;
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// An Artifact Input Manifest (AIM) for a software artifact.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Manifest {
    /// The hash algorithm associated with all entries in the manifest.
    hash_algorithm: HashAlgorithm,
    /// The individual entries in the manifest.
    entries: BTreeSet<ManifestEntry>,
}

/// An individual entry in the `Manifest`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ManifestEntry {
    /// The identifier for the input.
    artifact_id: GitOid,
    /// The identifier for the manifest of inputs to the input.
    manifest_id: Option<GitOid>,
}

impl ManifestEntry {
    // Get the ID of the artifact.
    pub fn artifact_id(&self) -> GitOid {
        self.artifact_id
    }

    /// Get the ID of the manifest describing the artifact's inputs.
    pub fn manifest_id(&self) -> Option<GitOid> {
        self.manifest_id
    }
}

// Ensure `ManifestEntry` ordering only depends on the artifact ID.
impl PartialOrd for ManifestEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.artifact_id().partial_cmp(&other.artifact_id())
    }
}
