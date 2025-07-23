//! BoringSSL-based cryptography backend.

#![allow(clippy::new_without_default)]

use {
    super::HashProvider,
    crate::hash_algorithm::Sha256,
    boring::sha,
    digest::{
        consts::U32, FixedOutput, FixedOutputReset, HashMarker, Output, OutputSizeUser, Reset,
        Update,
    },
};

/// Use the BoringSSL hash implementation.
#[cfg_attr(docsrs, doc(cfg(feature = "provider-boringssl")))]
#[derive(Clone, Copy)]
pub struct BoringSsl {
    #[doc(hidden)]
    _phantom: (),
}

impl BoringSsl {
    /// Construct a new `BoringSsl` provider.
    pub fn new() -> Self {
        BoringSsl { _phantom: () }
    }
}

impl HashProvider<Sha256> for BoringSsl {
    type Digester = Sha256Digester;

    fn digester(&self) -> Self::Digester {
        Sha256Digester::default()
    }
}

/// `boringssl` SHA-256 implementing the `Digest` trait.
#[doc(hidden)]
pub struct Sha256Digester {
    hash: sha::Sha256,
}

impl Clone for Sha256Digester {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash.clone(),
        }
    }
}

impl Update for Sha256Digester {
    fn update(&mut self, data: &[u8]) {
        self.hash.update(data);
    }
}

impl OutputSizeUser for Sha256Digester {
    type OutputSize = U32;
}

impl FixedOutput for Sha256Digester {
    fn finalize_into(self, out: &mut Output<Self>) {
        out.copy_from_slice(self.hash.finish().as_slice());
    }

    fn finalize_fixed(self) -> Output<Self> {
        let mut out = Output::<Self>::default();
        out.copy_from_slice(self.hash.finish().as_slice());
        out
    }
}

impl HashMarker for Sha256Digester {}

impl Reset for Sha256Digester {
    fn reset(&mut self) {
        self.hash = sha::Sha256::new();
    }
}

impl FixedOutputReset for Sha256Digester {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        self.clone().finalize_into(out);
        self.reset()
    }
}

impl Default for Sha256Digester {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::new(),
        }
    }
}
