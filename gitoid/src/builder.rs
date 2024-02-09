//! Easily construct `GitOid`s.

use crate::GitOid;
use crate::NamedDigest;
use crate::ObjectType;
use crate::Result;
use digest::OutputSizeUser;
use generic_array::ArrayLength;
use generic_array::GenericArray;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::marker::PhantomData;

/// Builder of GitOids with a specific hash algorithm and object type.
pub struct GitOidBuilder<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    /// The object type to use.
    object_type: ObjectType,

    #[doc(hidden)]
    _phantom: PhantomData<D>,
}

impl<D> GitOidBuilder<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    /// Get a new builder with a specific hash algorithm and object type.
    pub fn new(object_type: ObjectType) -> GitOidBuilder<D> {
        GitOidBuilder {
            object_type,
            _phantom: PhantomData,
        }
    }

    /// Build a `GitOid` from bytes.
    pub fn build_from_bytes(&self, content: &[u8]) -> GitOid<D> {
        GitOid::new_from_bytes(self.object_type, content)
    }

    /// Build a `GitOid` from a string slice.
    pub fn build_from_str(&self, s: &str) -> GitOid<D> {
        GitOid::new_from_str(self.object_type, s)
    }

    /// Build a `GitOid` from an arbitrary buffered reader.
    pub fn build_from_reader<R>(&self, reader: BufReader<R>) -> Result<GitOid<D>>
    where
        R: Read + Seek,
    {
        GitOid::new_from_reader(self.object_type, reader)
    }
}
