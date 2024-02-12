//! The types of objects for which a `GitOid` can be made.

use crate::Error;
use core::str::FromStr;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub trait ObjectType: Display + FromStr {
    const NAME: &'static str;
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
        pub struct $name;

        impl_from_str!($name, $s);
        impl_display!($name, $s);
        impl ObjectType for $name {
            const NAME: &'static str = $s;
        }
    };
}

define_object_type!(Blob, "blob");
define_object_type!(Tree, "tree");
define_object_type!(Tag, "tag");
define_object_type!(Commit, "commit");
