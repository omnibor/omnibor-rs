use crate::{hash_algorithm::HashAlgorithm, ArtifactId};

/// Match an `InputManifest` in storage.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Match<H>
where
    H: HashAlgorithm,
{
    /// Match by the target's Artifact ID.
    Target(ArtifactId<H>),
    /// Match by the manifest's own Artifact ID.
    Manifest(ArtifactId<H>),
}
