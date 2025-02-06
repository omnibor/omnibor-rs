//! Cryptography libraries providing hash function implementations.

#[cfg(doc)]
use crate::artifact_id::ArtifactId;
use crate::hash_algorithm::HashAlgorithm;
use digest::Digest;

#[cfg(feature = "backend-boringssl")]
mod boringssl;
#[cfg(feature = "backend-boringssl")]
pub use crate::hash_provider::boringssl::BoringSsl;

#[cfg(feature = "backend-openssl")]
mod openssl;
#[cfg(feature = "backend-openssl")]
pub use crate::hash_provider::openssl::OpenSsl;

#[cfg(feature = "backend-rustcrypto")]
mod rustcrypto;
#[cfg(feature = "backend-rustcrypto")]
pub use crate::hash_provider::rustcrypto::RustCrypto;

/// A cryptography library for producing [`ArtifactId`]s with SHA-256.
pub trait HashProvider<H: HashAlgorithm>: Copy {
    /// The type used to produce the SHA-256 digest.
    type Digester: Digest;

    /// Get the SHA-256 digester.
    fn digester(&self) -> Self::Digester;
}
