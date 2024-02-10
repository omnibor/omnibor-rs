//! A gitoid representing a single artifact.

use crate::Error;
use crate::HashRef;
use crate::NamedDigest;
use crate::ObjectType;
use crate::Result;
use core::fmt;
use core::fmt::Display;
use core::fmt::Formatter;
use core::hash::Hash;
use core::ops::Not as _;
use core::str::FromStr;
use digest::OutputSizeUser;
use generic_array::sequence::GenericSequence;
use generic_array::ArrayLength;
use generic_array::GenericArray;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use url::Url;

/// A struct that computes [gitoids][g] based on the selected algorithm
///
/// [g]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
#[repr(C)]
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitOid<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    /// The type of object being represented.
    object_type: ObjectType,

    /// The output of the underlying hashing scheme.
    value: GenericArray<u8, D::OutputSize>,
}

impl<D> Display for GitOid<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", D::NAME, self.hash())
    }
}

impl<D> GitOid<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    //===========================================================================================
    // Constructors
    //-------------------------------------------------------------------------------------------

    /// Create a new `GitOid` based on a slice of bytes.
    pub fn new_from_bytes(object_type: ObjectType, content: &[u8]) -> GitOid<D> {
        GitOid {
            object_type,
            value: {
                let digester = D::new();
                let reader = BufReader::new(content);
                let expected_length = content.len();

                // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
                bytes_from_buffer(digester, reader, expected_length, object_type).unwrap()
            },
        }
    }

    /// Create a `GitOid` from a UTF-8 string slice.
    pub fn new_from_str(object_type: ObjectType, s: &str) -> GitOid<D> {
        let content = s.as_bytes();
        GitOid::new_from_bytes(object_type, content)
    }

    /// Create a `GitOid` from a reader.
    pub fn new_from_reader<R>(object_type: ObjectType, mut reader: R) -> Result<GitOid<D>>
    where
        R: Read + Seek,
    {
        let digester = D::new();
        let expected_length = stream_len(&mut reader)? as usize;
        let value = bytes_from_buffer(digester, reader, expected_length, object_type)?;

        Ok(GitOid { object_type, value })
    }

    /// Construct a new `GitOid` from a `Url`.
    pub fn new_from_url(url: Url) -> Result<GitOid<D>> {
        url.try_into()
    }

    //===========================================================================================
    // Getters
    //-------------------------------------------------------------------------------------------

    /// Get a URL for the current `GitOid`.
    pub fn url(&self) -> Url {
        let s = format!("gitoid:{}:{}:{}", self.object_type, D::NAME, self.hash());
        // PANIC SAFETY: We know that this is a valid URL.
        Url::parse(&s).unwrap()
    }

    /// Get the hash data as a slice of bytes.
    pub fn hash(&self) -> HashRef<'_> {
        HashRef::new(&self.value[..])
    }

    /// Get the hash algorithm used for the `GitOid`.
    pub fn hash_algorithm(&self) -> &'static str {
        D::NAME
    }

    /// Get the object type of the `GitOid`.
    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }

    /// Get the length of the hash in bytes.
    pub fn hash_len(&self) -> usize {
        <D as OutputSizeUser>::output_size()
    }
}

impl<D> TryFrom<Url> for GitOid<D>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
{
    type Error = Error;

    fn try_from(url: Url) -> Result<GitOid<D>> {
        use Error::*;

        // Validate the scheme used.
        if url.scheme() != "gitoid" {
            return Err(InvalidScheme(url));
        }

        // Get the segments as an iterator over string slices.
        let mut segments = url.path().split(':');

        // Parse the object type, if present.
        let object_type = {
            let part = segments
                .next()
                .and_then(|p| p.is_empty().not().then_some(p))
                .ok_or_else(|| MissingObjectType(url.clone()))?;

            ObjectType::from_str(part)?
        };

        // Parse the hash algorithm, if present.
        let hash_algorithm = segments
            .next()
            .and_then(|p| p.is_empty().not().then_some(p))
            .ok_or_else(|| MissingHashAlgorithm(url.clone()))?;

        if hash_algorithm != D::NAME {
            return Err(Error::MismatchedHashAlgorithm {
                expected: D::NAME.to_string(),
                observed: hash_algorithm.to_string(),
            });
        }

        // Parse the hash, if present.
        let hex_str = segments
            .next()
            .and_then(|p| p.is_empty().not().then_some(p))
            .ok_or_else(|| MissingHash(url.clone()))?;

        // TODO(abrinker): When `sha1` et al. move to generic-array 1.0, update this to use the `arr!` macro.
        let mut value = GenericArray::generate(|_| 0);

        hex::decode_to_slice(hex_str, &mut value)?;

        let expected_size = <D as OutputSizeUser>::output_size();

        if value.len() != expected_size {
            return Err(Error::UnexpectedHashLength {
                expected: expected_size,
                observed: value.len(),
            });
        }

        // Construct a new `GitOid` from the parts.
        Ok(GitOid { object_type, value })
    }
}

/// Take a `BufReader` and generate a hash based on the `GitOid`'s hashing algorithm.
///
/// Will return an `Err` if the `BufReader` generates an `Err` or if the
/// `expected_length` is different from the actual length.
///
/// Why the latter `Err`?
///
/// The prefix string includes the number of bytes being hashed and that's the
/// `expected_length`. If the actual bytes hashed differs, then something went
/// wrong and the hash is not valid.
fn bytes_from_buffer<D, R>(
    mut digester: D,
    mut reader: R,
    expected_length: usize,
    object_type: ObjectType,
) -> Result<GenericArray<u8, D::OutputSize>>
where
    D: NamedDigest,
    <D as OutputSizeUser>::OutputSize: ArrayLength<u8>,
    GenericArray<u8, D::OutputSize>: Copy,
    R: Read,
{
    let prefix = format!("{} {}\0", object_type, expected_length);

    // Linux default page size is 4096, so use that.
    let mut buf = [0; 4096];
    let mut amount_read: usize = 0;

    // Set the prefix
    digester.update(prefix.as_bytes());

    // Keep reading the input until there is no more
    loop {
        match reader.read(&mut buf)? {
            // done
            0 => break,

            // Update the hash and accumulate the count
            size => {
                digester.update(&buf[..size]);
                amount_read += size;
            }
        }
    }

    // Make sure we got the length we expected
    if amount_read != expected_length {
        return Err(Error::BadLength {
            expected: expected_length,
            actual: amount_read,
        });
    }

    let hash = digester.finalize();
    let expected_size = <D as OutputSizeUser>::output_size();

    if hash.len() != expected_size {
        return Err(Error::UnexpectedHashLength {
            expected: expected_size,
            observed: hash.len(),
        });
    }

    Ok(hash)
}

// Adapted from the Rust standard library's unstable implementation
// of `Seek::stream_len`.
//
// TODO(abrinker): Remove this when `Seek::stream_len` is stabilized.
//
// License reproduction:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
fn stream_len<R>(mut stream: R) -> Result<u64>
where
    R: Seek,
{
    let old_pos = stream.stream_position()?;
    let len = stream.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}
