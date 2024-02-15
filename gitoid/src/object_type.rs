//! The types of objects for which a `GitOid` can be made.

use crate::sealed::Sealed;
use crate::Error;
#[cfg(doc)]
use crate::GitOid;
use core::str::FromStr;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

/// Object types usable to construct a [`GitOid`]
pub trait ObjectType: Display + FromStr + Copy + Clone + Sealed {
    const NAME: &'static str;
}

macro_rules! impl_copy_and_clone {
    ( $name:tt, $s:literal ) => {
        impl Clone for $name {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl Copy for $name {}
    };
}

macro_rules! impl_from_str {
    ( $name:tt, $s:literal ) => {
        impl FromStr for $name {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $s => Ok($name),
                    _ => Err(Error::UnknownObjectType(s.to_owned())),
                }
            }
        }
    };
}

macro_rules! impl_display {
    ( $name:tt, $s:literal ) => {
        impl Display for $name {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                write!(f, "{}", $s)
            }
        }
    };
}

macro_rules! define_object_type {
    ( $name:tt, $s:literal ) => {
        impl_copy_and_clone!($name, $s);
        impl_from_str!($name, $s);
        impl_display!($name, $s);

        impl Sealed for $name {}

        impl ObjectType for $name {
            /// cbindgen:ignore
            const NAME: &'static str = $s;
        }
    };
}

/// A Blob GitOid object.
pub struct Blob;
define_object_type!(Blob, "blob");

/// A Tree GitOid object.
pub struct Tree;
define_object_type!(Tree, "tree");

/// A Tag GitOid object.
pub struct Tag;
define_object_type!(Tag, "tag");

/// A Commit GitOid object.
pub struct Commit;
define_object_type!(Commit, "commit");
