//! OpenSSL-based cryptography backend.

use crate::impl_hash_algorithm;
use crate::sealed::Sealed;
use crate::HashAlgorithm;
use openssl::sha;
use digest::consts::U20;
use digest::consts::U32;
use digest::generic_array::GenericArray;
use digest::Digest;
use digest::FixedOutput;
use digest::HashMarker;
use digest::Output;
use digest::OutputSizeUser;
use digest::Update;

#[cfg(feature = "sha1")]
/// SHA-1 algorithm
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

/// OpenSSL SHA-256 implementation.
#[doc(hidden)]
pub struct OpenSSLSha256 {
    hash: sha::Sha256,
}

#[cfg(all(feature = "sha256", feature = "openssl"))]
impl_hash_algorithm!(Sha256, OpenSSLSha256, "sha256");

impl Update for OpenSSLSha256 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

impl OutputSizeUser for OpenSSLSha256 {
    type OutputSize = U32;
}

impl FixedOutput for OpenSSLSha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(self.hash.finish().as_slice());
    }

    fn finalize_fixed(self) -> Output<Self> {
        let mut out = Output::<Self>::default();
        out.copy_from_slice(self.hash.finish().as_slice());
        out
    }
}

impl HashMarker for OpenSSLSha256 {}

impl Default for OpenSSLSha256 {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::new(),
        }
    }
}

#[cfg(feature = "sha1")]
/// SHA-1 algorithm
pub struct Sha1 {
    #[doc(hidden)]
    _private: (),
}

/// OpenSSL SHA-1 implementation.
#[doc(hidden)]
pub struct OpenSSLSha1 {
    hash: sha::Sha1,
}

#[cfg(all(feature = "sha1", feature = "openssl"))]
impl_hash_algorithm!(Sha1, OpenSSLSha1, "sha1");

impl Update for OpenSSLSha1 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

impl OutputSizeUser for OpenSSLSha1 {
    type OutputSize = U20;
}

impl FixedOutput for OpenSSLSha1 {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(self.hash.finish().as_slice());
    }

    fn finalize_fixed(self) -> Output<Self> {
        let mut out = Output::<Self>::default();
        out.copy_from_slice(self.hash.finish().as_slice());
        out
    }
}

impl HashMarker for OpenSSLSha1 {}

impl Default for OpenSSLSha1 {
    fn default() -> Self {
        Self {
            hash: sha::Sha1::new(),
        }
    }
}
