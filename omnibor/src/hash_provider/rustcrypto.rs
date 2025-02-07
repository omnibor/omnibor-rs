//! RustCrypto-based cryptography backend.

#![allow(clippy::derivable_impls)]
#![allow(clippy::new_without_default)]

use {
    super::HashProvider,
    crate::hash_algorithm::Sha256,
    digest::{consts::U32, FixedOutput, HashMarker, Output, OutputSizeUser, Update},
    sha2 as sha,
};

/// Use the RustCrypto hash implementation.
#[cfg_attr(docsrs, doc(cfg(feature = "backend-rustcrypto")))]
#[derive(Clone, Copy)]
pub struct RustCrypto {
    #[doc(hidden)]
    _phantom: (),
}

impl RustCrypto {
    /// Construct a new `RustCrypto` provider.
    pub fn new() -> Self {
        RustCrypto { _phantom: () }
    }
}

impl HashProvider<Sha256> for RustCrypto {
    type Digester = Sha256Digester;

    fn digester(&self) -> Self::Digester {
        Sha256Digester::default()
    }
}

/// `rustcrypto` SHA-256 implementing the `Digest` trait.
///
/// This just wraps the internal type that already implements `Digest` for
/// consistency with the other cryptography providers.
#[doc(hidden)]
pub struct Sha256Digester {
    hash: sha::Sha256,
}

impl Update for Sha256Digester {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.hash, data);
    }
}

impl OutputSizeUser for Sha256Digester {
    type OutputSize = U32;
}

impl FixedOutput for Sha256Digester {
    fn finalize_into(self, out: &mut Output<Self>) {
        FixedOutput::finalize_into(self.hash, out)
    }

    fn finalize_fixed(self) -> Output<Self> {
        self.hash.finalize_fixed()
    }
}

impl HashMarker for Sha256Digester {}

impl Default for Sha256Digester {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::default(),
        }
    }
}
