//! Easily construct `GitOid`s.

#![allow(clippy::new_without_default)]

use crate::GitOid;
use crate::HashAlgorithm;
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
pub struct GitOidBuilder<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
    <H as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, H::OutputSize>: Copy,
{
    #[doc(hidden)]
    _hash_algorithm: PhantomData<H>,

    #[doc(hidden)]
    _object_type: PhantomData<O>,
}

impl<H, O> GitOidBuilder<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
    <H as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, H::OutputSize>: Copy,
{
    /// Get a new builder with a specific hash algorithm and object type.
    pub fn new() -> GitOidBuilder<H, O> {
        GitOidBuilder {
            _hash_algorithm: PhantomData,
            _object_type: PhantomData,
        }
    }

    /// Build a `GitOid` from bytes.
    pub fn build_from_bytes(&self, content: &[u8]) -> GitOid<H, O> {
        GitOid::new_from_bytes(content)
    }

    /// Build a `GitOid` from a string slice.
    pub fn build_from_str(&self, s: &str) -> GitOid<H, O> {
        GitOid::new_from_str(s)
    }

    /// Build a `GitOid` from an arbitrary buffered reader.
    pub fn build_from_reader<R>(&self, reader: BufReader<R>) -> Result<GitOid<H, O>>
    where
        R: Read + Seek,
    {
        GitOid::new_from_reader(reader)
    }
}
