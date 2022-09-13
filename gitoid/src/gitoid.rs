//! A gitoid representing a single artifact.

use crate::{Error, HashAlgorithm, ObjectType, Result, NUM_HASH_BYTES};
use core::fmt::{self, Display, Formatter};
use core::marker::Unpin;
use sha2::digest::DynDigest;
use std::hash::Hash;
use std::io::{BufReader, Read};
use tokio::io::AsyncReadExt;
use url::Url;

/// A struct that computes [gitoids][g] based on the selected algorithm
///
/// [g]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitOid {
    /// The hash algorithm being used.
    hash_algorithm: HashAlgorithm,

    /// The type of object being represented.
    object_type: ObjectType,

    /// The length of the hashed data in number of bytes.
    ///
    /// Invariant: this must always be less than `NUM_HASH_BYTES`.
    len: usize,

    /// The buffer storing the actual hashed bytes.
    value: [u8; NUM_HASH_BYTES],
}

impl Display for GitOid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.hash_algorithm, self.hash())
    }
}

impl GitOid {
    //===========================================================================================
    // Constructors
    //-------------------------------------------------------------------------------------------

    /// Create a new `GitOid` based on a slice of bytes.
    pub fn new_from_bytes(
        hash_algorithm: HashAlgorithm,
        object_type: ObjectType,
        content: &[u8],
    ) -> Self {
        let digester = hash_algorithm.create_digester();
        let reader = BufReader::new(content);
        let expected_length = content.len();

        // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
        let (len, value) = bytes_from_buffer(digester, reader, expected_length).unwrap();

        GitOid {
            hash_algorithm,
            object_type,
            value,
            len,
        }
    }

    /// Create a `GitOid` from a UTF-8 string slice.
    pub fn new_from_str(hash_algorithm: HashAlgorithm, object_type: ObjectType, s: &str) -> Self {
        let content = s.as_bytes();
        GitOid::new_from_bytes(hash_algorithm, object_type, content)
    }

    /// Create a `GitOid` from a reader.
    ///
    /// This requires an `expected_length` as part of a correctness check.
    pub fn new_from_reader<R>(
        hash_algorithm: HashAlgorithm,
        object_type: ObjectType,
        reader: BufReader<R>,
        expected_length: usize,
    ) -> Result<Self>
    where
        R: Read,
    {
        let digester = hash_algorithm.create_digester();
        let (len, value) = bytes_from_buffer(digester, reader, expected_length)?;

        Ok(GitOid {
            hash_algorithm,
            object_type,
            len,
            value,
        })
    }

    /// Create a `GitOid` from an asynchronous reader.
    ///
    /// This requires an `expected_length` as part of a correctness check.
    pub async fn new_from_async_reader<R>(
        hash_algorithm: HashAlgorithm,
        object_type: ObjectType,
        reader: R,
        expected_length: usize,
    ) -> Result<Self>
    where
        R: AsyncReadExt + Unpin,
    {
        // Construct a new digester.
        let digester = hash_algorithm.create_digester();
        let (len, value) = bytes_from_async_buffer(digester, reader, expected_length).await?;

        Ok(GitOid {
            hash_algorithm,
            object_type,
            len,
            value,
        })
    }

    //===========================================================================================
    // Getters
    //-------------------------------------------------------------------------------------------

    /// Get a URL for the current `GitOid`.
    pub fn uri(&self) -> Result<Url> {
        let s = format!(
            "gitoid:{}:{}:{}",
            self.object_type,
            self.hash_algorithm,
            self.hash()
        );
        Ok(Url::parse(&s)?)
    }

    /// Get the hex value of the hash data, without the hash type.
    pub fn hash(&self) -> String {
        hex::encode(self.bytes())
    }

    /// Get the hash data as a slice of bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.value[0..self.len]
    }

    /// Get the hash algorithm used for the `GitOid`.
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.hash_algorithm
    }

    /// Get the object type of the `GitOid`.
    pub fn object_type(&self) -> ObjectType {
        self.object_type
    }
}

//===============================================================================================
// Helpers
//-----------------------------------------------------------------------------------------------

/// The async version of generating a `GitOid` from a buffer
async fn bytes_from_async_buffer<R>(
    mut digester: Box<dyn DynDigest>,
    mut reader: R,
    expected_length: usize,
) -> Result<(usize, [u8; NUM_HASH_BYTES])>
where
    R: AsyncReadExt + Unpin,
{
    let prefix = format!("blob {}\0", expected_length);

    let mut buf = [0u8; 8192]; // the size of a buffer for buffered read
    let mut amount_read: usize = 0;

    // set the prefix
    digester.update(prefix.as_bytes());

    // Keep reading the input until there is no more
    loop {
        match reader.read(&mut buf).await? {
            // Done
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
    let mut ret = [0u8; NUM_HASH_BYTES];

    let len = NUM_HASH_BYTES.min(hash.len());
    ret[..len].copy_from_slice(&hash);
    Ok((len, ret))
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
fn bytes_from_buffer<R>(
    mut digester: Box<dyn DynDigest>,
    mut reader: BufReader<R>,
    expected_length: usize,
) -> Result<(usize, [u8; NUM_HASH_BYTES])>
where
    BufReader<R>: Read,
{
    let prefix = format!("blob {}\0", expected_length);

    let mut buf = [0; 4096]; // Linux default page size is 4096
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
    let mut ret = [0u8; NUM_HASH_BYTES];

    let len = std::cmp::min(NUM_HASH_BYTES, hash.len());
    ret[..len].copy_from_slice(&hash);
    Ok((len, ret))
}
