//! The types of objects for which a `GitOid` can be made.

use crate::sealed::Sealed;
#[cfg(doc)]
use crate::GitOid;

/// Object types usable to construct a [`GitOid`].
///
/// This is a sealed trait to ensure it's only used for hash
/// algorithms which are actually supported by Git.
///
/// For more information on sealed traits, read Predrag
/// Gruevski's ["A Definitive Guide to Sealed Traits in Rust"][1].
pub trait ObjectType: Sealed {
    #[doc(hidden)]
    const NAME: &'static str;
}

macro_rules! define_object_type {
    ( $name:tt, $s:literal ) => {
        impl Sealed for $name {}

        impl ObjectType for $name {
            const NAME: &'static str = $s;
        }
    };
}

/// A Blob GitOid object.
pub struct Blob {
    #[doc(hidden)]
    _private: (),
}

define_object_type!(Blob, "blob");

/// A Tree GitOid object.
pub struct Tree {
    #[doc(hidden)]
    _private: (),
}

define_object_type!(Tree, "tree");

/// A Tag GitOid object.
pub struct Tag {
    #[doc(hidden)]
    _private: (),
}

define_object_type!(Tag, "tag");

/// A Commit GitOid object.
pub struct Commit {
    _private: (),
}

define_object_type!(Commit, "commit");
