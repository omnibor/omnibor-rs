use gitoid::GitOid;
use gitoid::HashAlgorithm;
use std::collections::BTreeSet;

/// An Artifact Input Manifest (AIM) for a software artifact.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Manifest {
    /// The hash algorithm associated with all records in the manifest.
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
    // Get the ID of the artifact in question.
    pub fn artifact_id(&self) -> GitOid {
        self.artifact_id
    }

    /// Get the ID of the manifest describing the artifact's inputs.
    pub fn manifest_id(&self) -> Option<GitOid> {
        self.manifest_id
    }
}

impl PartialOrd for ManifestEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.artifact_id().partial_cmp(&other.artifact_id())
    }
}
