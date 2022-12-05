//! Easily construct `GitOid`s.

use crate::GitOid;
use crate::HashAlgorithm;
use crate::ObjectType;
use crate::Result;
use std::io::BufReader;
use std::io::Read;

/// Builder of GitOids with a specific hash algorithm and object type.
pub struct GitOidBuilder {
    /// The hash algorithm to use.
    hash_algorithm: HashAlgorithm,

    /// The object type to use.
    object_type: ObjectType,
}

impl GitOidBuilder {
    /// Get a new builder with a specific hash algorithm and object type.
    pub fn new(hash_algorithm: HashAlgorithm, object_type: ObjectType) -> GitOidBuilder {
        GitOidBuilder {
            hash_algorithm,
            object_type,
        }
    }

    /// Build a `GitOid` from bytes.
    pub fn build_from_bytes(&self, content: &[u8]) -> GitOid {
        GitOid::new_from_bytes(self.hash_algorithm, self.object_type, content)
    }

    /// Build a `GitOid` from a string slice.
    pub fn build_from_str(&self, s: &str) -> GitOid {
        GitOid::new_from_str(self.hash_algorithm, self.object_type, s)
    }

    /// Build a `GitOid` from an arbitrary buffered reader.
    pub fn build_from_reader<R>(
        &self,
        reader: BufReader<R>,
        expected_length: usize,
    ) -> Result<GitOid>
    where
        R: Read,
    {
        GitOid::new_from_reader(
            self.hash_algorithm,
            self.object_type,
            reader,
            expected_length,
        )
    }
}
