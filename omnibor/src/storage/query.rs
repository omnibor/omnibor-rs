use crate::{error::ArtifactIdError, hash_algorithm::HashAlgorithm, ArtifactId, Identify};
use std::marker::PhantomData;

/// Match an `InputManifest` in storage.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Match<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    /// Match by the target's Artifact ID.
    Target(TargetMatch<H, I>),
    /// Match by the manifest's own Artifact ID.
    Manifest(ManifestMatch<H, I>),
}

impl<H, I> Match<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    /// Construct a new target-based matcher.
    pub fn target(matcher: I) -> Self {
        Match::Target(TargetMatch {
            matcher,
            _phantom: PhantomData,
        })
    }

    /// Construct a new manifest-based matcher.
    pub fn manifest(matcher: I) -> Self {
        Match::Manifest(ManifestMatch {
            matcher,
            _phantom: PhantomData,
        })
    }
}

/// Matches against a target.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TargetMatch<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    matcher: I,
    _phantom: PhantomData<H>,
}

impl<H, I> TargetMatch<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    pub fn id(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.matcher.identify()
    }
}

/// Matches against a manifest.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ManifestMatch<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    matcher: I,
    _phantom: PhantomData<H>,
}

impl<H, I> ManifestMatch<H, I>
where
    H: HashAlgorithm,
    I: Identify<H>,
{
    pub fn id(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.matcher.identify()
    }
}
