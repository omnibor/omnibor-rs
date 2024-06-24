//! RustCrypto-based cryptography backend.

use crate::impl_hash_algorithm;
use crate::sealed::Sealed;
#[cfg(doc)]
use crate::GitOid;
use crate::HashAlgorithm;
use digest::generic_array::GenericArray;
use digest::Digest;
use digest::OutputSizeUser;

#[cfg(feature = "sha1")]
/// SHA-1 algorithm,
pub struct Sha1 {
    #[doc(hidden)]
    _private: (),
}

#[cfg(feature = "sha1")]
impl_hash_algorithm!(Sha1, sha1::Sha1, "sha1");

#[cfg(feature = "sha1cd")]
/// SHA-1Cd (collision detection) algorithm.
pub struct Sha1Cd {
    #[doc(hidden)]
    _private: (),
}

#[cfg(feature = "sha1cd")]
impl_hash_algorithm!(Sha1Cd, sha1collisiondetection::Sha1CD, "sha1cd");

#[cfg(feature = "sha256")]
/// SHA-256 algorithm.
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

#[cfg(feature = "sha256")]
impl_hash_algorithm!(Sha256, sha2::Sha256, "sha256");
