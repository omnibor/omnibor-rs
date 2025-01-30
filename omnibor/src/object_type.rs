//! The types of objects for which a `GitOid` can be made.

use crate::util::sealed::Sealed;

#[cfg(doc)]
use crate::gitoid::GitOid;

/// Object types usable to construct a [`GitOid`].
///
/// This is a sealed trait to ensure it's only used for hash
/// algorithms which are actually supported by Git.
///
/// For more information on sealed traits, read Predrag
/// Gruevski's ["A Definitive Guide to Sealed Traits in Rust"][1].
///
/// [1]: https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/
pub trait ObjectType: Sealed {
    #[doc(hidden)]
    const NAME: &'static str;
}

/// A Blob GitOid object.
pub struct Blob {
    #[doc(hidden)]
    _private: (),
}

impl Sealed for Blob {}

impl ObjectType for Blob {
    const NAME: &'static str = "blob";
}
