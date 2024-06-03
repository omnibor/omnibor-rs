use boring::sha;
use digest::{FixedOutput, HashMarker, Output, OutputSizeUser, Update};
use digest::consts::{U20, U32};

/// Boring SHA-256 implementation.
pub struct BoringSha256 {
    hash: sha::Sha256,
}

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

/// Boring SHA-1 implementation.
pub struct BoringSha1 {
    hash: sha::Sha1,
}

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
