//! Cryptography libraries providing hash algorithm implementations.
//!
//! "Hash Providers" are cryptography libraries providing implementations of
//! the hash algorithms approved for use in OmniBOR Artifact IDs and Input
//! Manifests. Today, that's only the SHA-256 hash algorithm.
//!
//! There are three providers supported in the `omnibor` crate today:
//!
//! - RustCrypto
//! - BoringSSL
//! - OpenSSL
//!
//! Each of these is embodied in a type implementing the `HashProvider` trait,
//! which is parameterized over the hash algorithm.
//!
//! [__See the main documentation on Hash Providers for more information.__][idx]
//!
//! [idx]: crate#hash-providers

#[cfg(feature = "provider-boringssl")]
mod boringssl;
#[cfg(feature = "provider-openssl")]
mod openssl;
pub(crate) mod registry;
#[cfg(feature = "provider-rustcrypto")]
mod rustcrypto;

#[cfg(doc)]
use crate::artifact_id::ArtifactId;
use crate::hash_algorithm::HashAlgorithm;
#[cfg(feature = "provider-boringssl")]
pub use crate::hash_provider::boringssl::BoringSsl;
#[cfg(feature = "provider-openssl")]
pub use crate::hash_provider::openssl::OpenSsl;
#[cfg(feature = "provider-rustcrypto")]
pub use crate::hash_provider::rustcrypto::RustCrypto;
use digest::Digest;

/// A cryptography library for producing [`ArtifactId`]s with SHA-256.
pub trait HashProvider<H: HashAlgorithm>: Copy + Send + Sync + 'static {
    /// The type used to produce the SHA-256 digest.
    type Digester: Digest;

    /// Get the SHA-256 digester.
    fn digester(&self) -> Self::Digester;
}
