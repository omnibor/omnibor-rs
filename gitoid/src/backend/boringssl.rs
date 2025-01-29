//! BoringSSL-based cryptography backend.

use crate::{impl_hash_algorithm, sealed::Sealed, HashAlgorithm};
use boring::sha;
use digest::{
    consts::{U20, U32},
    generic_array::GenericArray,
    Digest, FixedOutput, HashMarker, Output, OutputSizeUser, Update,
};

#[cfg(feature = "hash-sha1")]
/// SHA-1 algorithm
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

/// Boring SHA-256 implementation.
#[doc(hidden)]
pub struct BoringSha256 {
    hash: sha::Sha256,
}

#[cfg(all(feature = "hash-sha256"))]
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

#[cfg(feature = "hash-sha1")]
/// SHA-1 algorithm
pub struct Sha1 {
    #[doc(hidden)]
    _private: (),
}

/// Boring SHA-1 implementation.
#[doc(hidden)]
pub struct BoringSha1 {
    hash: sha::Sha1,
}

#[cfg(all(feature = "hash-sha1"))]
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
