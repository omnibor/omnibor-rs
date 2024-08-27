use crate::sealed::Sealed;
#[cfg(doc)]
use crate::ArtifactId;
use gitoid::HashAlgorithm;

/// Marker trait for hash algorithms supported for constructing [`ArtifactId`]s.
pub trait SupportedHash: Sealed {
    type HashAlgorithm: HashAlgorithm;
}

/// The SHA-256 hashing algorithm.
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

impl Sealed for Sha256 {}

impl SupportedHash for Sha256 {
    type HashAlgorithm = gitoid::rustcrypto::Sha256;
}
