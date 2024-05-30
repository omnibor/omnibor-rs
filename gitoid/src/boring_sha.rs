#[cfg(feature = "boring")]
use boring::sha;
#[cfg(feature = "boring")]
use digest::{FixedOutput, HashMarker, Output, OutputSizeUser, Update};
#[cfg(feature = "boring")]
use digest::consts::{U20, U32};

/// Boring SHA-256 implementation.
#[cfg(feature = "boring")]
pub struct BoringSha256 {
    hash: sha::Sha256,
}

#[cfg(feature = "boring")]
impl Update for BoringSha256 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

#[cfg(feature = "boring")]
impl OutputSizeUser for BoringSha256 {
    type OutputSize = U32;
}

#[cfg(feature = "boring")]
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

#[cfg(feature = "boring")]
impl HashMarker for BoringSha256 {}

#[cfg(feature = "boring")]
impl Default for BoringSha256 {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::new(),
        }
    }
}

/// Boring SHA-1 implementation.
#[cfg(feature = "boring")]
pub struct BoringSha1 {
    hash: sha::Sha1,
}

#[cfg(feature = "boring")]
impl Update for BoringSha1 {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

#[cfg(feature = "boring")]
impl OutputSizeUser for BoringSha1 {
    type OutputSize = U20;
}

#[cfg(feature = "boring")]
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

#[cfg(feature = "boring")]
impl HashMarker for BoringSha1 {}

#[cfg(feature = "boring")]
impl Default for BoringSha1 {
    fn default() -> Self {
        Self {
            hash: sha::Sha1::new(),
        }
    }
}
