use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::Error;

/// The types of objects for which a `GitOid` can be made.
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
            _ => Err(Error::UnknownObjectType(s.to_owned())),
        }
    }
}
