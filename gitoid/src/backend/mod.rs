//! Cryptography backends, providing hash function implementations.

#[cfg(feature = "boringssl")]
pub mod boringssl;

#[cfg(feature = "openssl")]
pub mod openssl;

#[cfg(feature = "rustcrypto")]
pub mod rustcrypto;
