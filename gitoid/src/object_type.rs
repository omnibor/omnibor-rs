//! The types of objects for which a `GitOid` can be made.

use crate::Error;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::str::FromStr;

/// The types of objects for which a `GitOid` can be made.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectType {
    /// An opaque git blob.
    Blob,
    /// A Git tree.
    Tree,
    /// A Git commit.
    Commit,
    /// A Git tag.
    Tag,
    /// An invalid object
    Invalid,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ObjectType::Blob => "blob",
                ObjectType::Tree => "tree",
                ObjectType::Commit => "commit",
                ObjectType::Tag => "tag",
                ObjectType::Invalid => "invalid",
            }
        )
    }
}

impl FromStr for ObjectType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            // Invalid objects can't be constructed from a string.
            _ => Err(Error::UnknownObjectType(s.to_owned())),
        }
    }
}
