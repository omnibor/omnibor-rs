//! BoringSSL-based cryptography backend.

use crate::impl_hash_algorithm;
use crate::sealed::Sealed;
use crate::HashAlgorithm;
use boring::sha;
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
/// SHA-1 algorithm,
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

/// Boring SHA-256 implementation.
pub struct BoringSha256 {
    hash: sha::Sha256,
}

#[cfg(all(feature = "sha256", feature = "boringssl"))]
impl_hash_algorithm!(Sha256, BoringSha256, "sha256");

impl Update for BoringSha256 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

impl OutputSizeUser for BoringSha256 {
    type OutputSize = U32;
}

impl FixedOutput for BoringSha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(self.hash.finish().as_slice());
    }

    fn finalize_fixed(self) -> Output<Self> {
        let mut out = Output::<Self>::default();
        out.copy_from_slice(self.hash.finish().as_slice());
        out
    }
}

impl HashMarker for BoringSha256 {}

impl Default for BoringSha256 {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::new(),
        }
    }
}

#[cfg(feature = "sha1")]
/// SHA-1 algorithm,
pub struct Sha1 {
    #[doc(hidden)]
    _private: (),
}

/// Boring SHA-1 implementation.
pub struct BoringSha1 {
    hash: sha::Sha1,
}

#[cfg(all(feature = "sha1", feature = "boringssl"))]
impl_hash_algorithm!(Sha1, BoringSha1, "sha1");

impl Update for BoringSha1 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

impl OutputSizeUser for BoringSha1 {
    type OutputSize = U20;
}

impl FixedOutput for BoringSha1 {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(self.hash.finish().as_slice());
    }

    fn finalize_fixed(self) -> Output<Self> {
        let mut out = Output::<Self>::default();
        out.copy_from_slice(self.hash.finish().as_slice());
        out
    }
}

impl HashMarker for BoringSha1 {}

impl Default for BoringSha1 {
    fn default() -> Self {
        Self {
            hash: sha::Sha1::new(),
        }
    }
}
