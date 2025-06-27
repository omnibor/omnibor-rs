//! Cryptography libraries providing hash function implementations.
//!
//! __See [Hash Algorithms and Hash Providers][idx] documentation for more info.__
//!
//! [idx]: crate#hash-algorithms-and-hash-providers

#[cfg(doc)]
use crate::artifact_id::ArtifactId;
use crate::hash_algorithm::HashAlgorithm;
use digest::Digest;

#[cfg(feature = "provider-boringssl")]
mod boringssl;
#[cfg(feature = "provider-boringssl")]
pub use crate::hash_provider::boringssl::BoringSsl;

#[cfg(feature = "provider-openssl")]
mod openssl;
#[cfg(feature = "provider-openssl")]
pub use crate::hash_provider::openssl::OpenSsl;

#[cfg(feature = "provider-rustcrypto")]
mod rustcrypto;
#[cfg(feature = "provider-rustcrypto")]
pub use crate::hash_provider::rustcrypto::RustCrypto;

/// A cryptography library for producing [`ArtifactId`]s with SHA-256.
pub trait HashProvider<H: HashAlgorithm>: Copy {
    /// The type used to produce the SHA-256 digest.
    type Digester: Digest;

    /// Get the SHA-256 digester.
    fn digester(&self) -> Self::Digester;
}
