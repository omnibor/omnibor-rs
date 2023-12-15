//! A hash algorithm which can be used to make a `GitOid`.

use crate::Error;
use crate::Result;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::str::FromStr;
use sha1::Sha1;
use sha2::digest::DynDigest;
use sha2::Digest;
use sha2::Sha256;

/// The available algorithms for computing hashes
#[repr(C)]
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub enum HashAlgorithm {
    /// [SHA1](https://en.wikipedia.org/wiki/SHA-1)
    Sha1,
    /// [SHA256](https://en.wikipedia.org/wiki/SHA-2)
    Sha256,
}

impl HashAlgorithm {
    /// Based on the `GitOid`'s hashing algorithm, generate an instance of
    /// a digester
    pub(crate) fn create_digester(&self) -> Box<dyn DynDigest> {
        match self {
            HashAlgorithm::Sha1 => Box::new(Sha1::new()),
            HashAlgorithm::Sha256 => Box::new(Sha256::new()),
        }
    }
}

// NOTE: This is kept here in this file because it needs to be updated
//       if any new hash algorithms are added.

/// The number of bytes required to store the largest hash.
///
/// Currently 32 for SHA256.
/// If another `HashAlgorithm` is added, update this value to reflect the
/// new maximum.
pub const NUM_HASH_BYTES: usize = 32;

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgorithm::Sha1 => write!(f, "sha1"),
            HashAlgorithm::Sha256 => write!(f, "sha256"),
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "sha1" => Ok(HashAlgorithm::Sha1),
            "sha256" => Ok(HashAlgorithm::Sha256),
            _ => Err(Error::UnknownHashAlgorithm(s.to_owned())),
        }
    }
}
