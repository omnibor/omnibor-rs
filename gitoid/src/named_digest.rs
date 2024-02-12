//! Trait specifying valid [`GitOid`] hash algorithms.

use crate::named_digest::private::Sealed;
#[cfg(doc)]
use crate::GitOid;
use digest::Digest;
use sha1::Sha1;
use sha1collisiondetection::Sha1CD as Sha1Cd;
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
///
/// For more information on sealed traits, read Predrag
/// Gruevski's ["A Definitive Guide to Sealed Traits in Rust"][1].
///
/// [1]: https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/
pub trait HashAlgorithm: Digest + Sealed {
    /// The name of the hash algorithm in lowercase ASCII.
    const NAME: &'static str;
}

macro_rules! impl_hash_algorithm {
    ( $type:ty, $name:literal ) => {
        impl Sealed for $type {}

        impl HashAlgorithm for $type {
            const NAME: &'static str = $name;
        }
    };
}

impl_hash_algorithm!(Sha1, "sha1");
impl_hash_algorithm!(Sha256, "sha256");
impl_hash_algorithm!(Sha1Cd, "sha1cd");