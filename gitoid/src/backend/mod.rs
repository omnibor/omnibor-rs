//! Cryptography backends, providing hash function implementations.

#[cfg(feature = "backend-boringssl")]
pub mod boringssl;

#[cfg(feature = "backend-openssl")]
pub mod openssl;

#[cfg(feature = "backend-rustcrypto")]
pub mod rustcrypto;
