//! Cryptography backends, providing hash function implementations.

#[cfg(feature = "boringssl")]
pub mod boringssl;

#[cfg(feature = "rustcrypto")]
pub mod rustcrypto;
