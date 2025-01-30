//! OpenSSL-based cryptography backend.

#![allow(clippy::new_without_default)]

use {
    super::HashProvider,
    crate::hash_algorithm::Sha256,
    digest::{consts::U32, FixedOutput, HashMarker, Output, OutputSizeUser, Update},
    openssl::sha,
};

#[derive(Clone, Copy)]
pub struct OpenSsl {
    #[doc(hidden)]
    _phantom: (),
}

impl OpenSsl {
    pub fn new() -> Self {
        OpenSsl { _phantom: () }
    }
}

impl HashProvider<Sha256> for OpenSsl {
    type Digester = Sha256Digester;

    fn digester(&self) -> Self::Digester {
        Sha256Digester::default()
    }
}

/// `openssl` SHA-256 implementing the `Digest` trait.
#[doc(hidden)]
pub struct Sha256Digester {
    hash: sha::Sha256,
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

impl Default for Sha256Digester {
    fn default() -> Self {
        Self {
            hash: sha::Sha256::new(),
        }
    }
}
