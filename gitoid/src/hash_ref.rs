//! Provides a type for representing the hash contained inside a `GitOid`.

use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::ops::Deref;

/// The hash produced for a `GitOid`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HashRef<'h>(&'h [u8]);

impl<'h> HashRef<'h> {
    /// Construct a new `Hash` for the given bytes.
    pub fn new(bytes: &[u8]) -> HashRef<'_> {
        HashRef(bytes)
    }

    /// Get the hash as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    /// Get a hexadecimal-encoded representation of the hash.
    pub fn as_hex(&self) -> String {
        hex::encode(self.0)
    }
}

// Deref to a slice of bytes.
impl<'h> Deref for HashRef<'h> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// Print as the hex encoding.
impl<'h> Display for HashRef<'h> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_hex())
    }
}
