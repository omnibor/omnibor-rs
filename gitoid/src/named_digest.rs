//! Trait specifying valid [`GitOid`] hash algorithms.

use crate::named_digest::private::Sealed;
#[cfg(doc)]
use crate::GitOid;
use digest::Digest;
use sha1::Sha1;
use sha2::Sha256;

mod private {
    pub trait Sealed {}
}

/// Hash algorithms that can be used to make a [`GitOid`].
///
/// This is a sealed trait to ensure it's only used for hash
/// algorithms which are actually supported by Git. No other
/// types, even if they implement [`Digest`] can implement
/// this trait.
pub trait NamedDigest: Digest + Sealed {
    /// The name of the hash algorithm in lowercase ASCII.
    const NAME: &'static str;
}

impl Sealed for Sha1 {}

impl NamedDigest for Sha1 {
    const NAME: &'static str = "sha1";
}

impl Sealed for Sha256 {}

impl NamedDigest for Sha256 {
    const NAME: &'static str = "sha256";
}
