use im::Vector;
use pin_project::pin_project;
use sha1::Sha1;
use sha2::{digest::DynDigest, Digest, Sha256};
use std::fmt::{Display, Formatter, Result};
use std::io::{BufReader, Error, ErrorKind, Read, Result as IOResult};
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

/// Represents a source of data which will be read to produce a `GitOid`.
#[pin_project]
pub struct Source<R> {
    /// The reader itself.
    #[pin]
    reader: R,
    /// The length of the data being read.
    length: usize,
}

#[allow(clippy::len_without_is_empty)]
impl<R> Source<R> {
    /// Create a new `Source` based on a `reader` and `length`.
    pub fn new(reader: R, length: usize) -> Self {
        Self { reader, length }
    }

    /// Get the length of the read data.
    pub fn len(&self) -> usize {
        self.length
    }
}

impl<R: AsyncRead> AsyncRead for Source<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IOResult<()>> {
        self.project().reader.poll_read(cx, buf)
    }
}

/// The available algorithms for computing hashes
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub enum HashAlgorithm {
    /// [SHA1](https://en.wikipedia.org/wiki/SHA-1)
    SHA1,
    /// [SHA256](https://en.wikipedia.org/wiki/SHA-2)
    SHA256,
}

impl HashAlgorithm {
    /// Based on the `GitOid`'s hashing algorithm, generate an instance of
    /// a digester
    pub fn create_digester(&self) -> Box<dyn DynDigest> {
        match self {
            HashAlgorithm::SHA1 => Box::new(Sha1::new()),
            HashAlgorithm::SHA256 => Box::new(Sha256::new()),
        }
    }
}

/// The number of bytes required to store the largest hash. Currently 32 for SHA256
/// If another `HashAlgorithm` is added, update to reflect.
const NUM_HASH_BYTES: usize = 32;

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            HashAlgorithm::SHA1 => write!(f, "SHA1"),
            HashAlgorithm::SHA256 => write!(f, "SHA256"),
        }
    }
}

/// A struct that computes [git oids](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
/// based on the selected algorithm
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitOid {
    /// The hash algorithm being used.
    hash_algorithm: HashAlgorithm,

    /// The length of the hashed data in number of bytes.
    ///
    /// Invariant: this must always be less than `NUM_HASH_BYTES`.
    len: usize,

    /// The buffer storing the actual hashed bytes.
    value: [u8; NUM_HASH_BYTES],
}

impl Display for GitOid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.hash_algorithm, self.hex_hash())
    }
}

impl GitOid {
    /// Get the hex value of the hashcode, without the hash type.
    pub fn hex_hash(&self) -> String {
        hex::encode(self.hash_value())
    }

    /// Get the hash data as a slice.
    pub fn hash_value(&self) -> &[u8] {
        &self.value[0..self.len]
    }

    /// Get the hash algorithm used for this GitOid.
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.hash_algorithm
    }

    /// Create a new `GitOid` based on an in-memory array.
    pub fn new(hash_algorithm: HashAlgorithm, content: &[u8]) -> Self {
        let digest = hash_algorithm.create_digester();
        let reader = BufReader::new(content);
        let expected_length = content.len();

        // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
        let (len, value) = GitOid::generate_from_buffer(digest, reader, expected_length).unwrap();

        GitOid {
            hash_algorithm,
            value,
            len,
        }
    }

    /// Create a `GitOid` from a `String`.
    pub fn new_from_str(hash_algorithm: HashAlgorithm, s: &str) -> Self {
        let content = s.as_bytes();
        GitOid::new(hash_algorithm, content)
    }

    /// Create a `GitOid` from a reader.
    ///
    /// This requires an `expected_length` as part of a correctness check.
    pub fn new_from_reader<R>(
        hash_algorithm: HashAlgorithm,
        reader: BufReader<R>,
        expected_length: usize,
    ) -> IOResult<Self>
    where
        BufReader<R>: std::io::Read,
    {
        let digest = hash_algorithm.create_digester();
        let (len, value) = GitOid::generate_from_buffer(digest, reader, expected_length)?;

        Ok(GitOid {
            hash_algorithm,
            len,
            value,
        })
    }

    /// Generate a bunch of `GitOid`s from a bunch of async readers for a given algorithm
    pub async fn new_from_async_readers<R, I>(
        hash_algorithm: HashAlgorithm,
        content: I,
    ) -> IOResult<Vector<GitOid>>
    where
        R: AsyncReadExt + Unpin,
        I: IntoIterator<Item = Source<R>>,
    {
        // Construct a new digester.
        let digester = hash_algorithm.create_digester();

        // Get an iterator of futures which will generate `GitOid`s for each item read.
        let futs = content.into_iter().map(|reader| {
            let expected_length = reader.len();
            let digester = digester.clone();
            GitOid::generate_from_async_buffer(digester, reader, expected_length)
        });

        // Go through each future and await the response.
        //
        // The cool thing is that this will block on any given future but other futures
        // may become satisfied so the look effectively blocks on the longest-to-satisfy future!
        let mut ret = Vector::new();
        for res in futures::future::join_all(futs).await {
            let (len, bytes) = res?;
            ret.push_back(GitOid {
                hash_algorithm,
                len,
                value: bytes,
            });
        }

        Ok(ret)
    }

    /// The async version of generating a `GitOid` from a buffer
    async fn generate_from_async_buffer<R>(
        mut digester: Box<dyn DynDigest>,
        mut reader: R,
        expected_length: usize,
    ) -> IOResult<(usize, [u8; NUM_HASH_BYTES])>
    where
        R: AsyncReadExt + std::marker::Unpin,
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
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Expected length {} actual length {}",
                    expected_length, amount_read
                ),
            ));
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
    fn generate_from_buffer<R>(
        mut digester: Box<dyn DynDigest>,
        mut reader: BufReader<R>,
        expected_length: usize,
    ) -> IOResult<(usize, [u8; NUM_HASH_BYTES])>
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
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Expected length {} actual length {}",
                    expected_length, amount_read
                ),
            ));
        }

        let hash = digester.finalize();
        let mut ret = [0u8; NUM_HASH_BYTES];

        let len = std::cmp::min(NUM_HASH_BYTES, hash.len());
        ret[..len].copy_from_slice(&hash);
        Ok((len, ret))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_generate_sha1_git_oid() {
        let input = "hello world".as_bytes();

        let result = GitOid::new(HashAlgorithm::SHA1, input);

        assert_eq!(
            result.to_string(),
            "SHA1:95d09f2b10159347eece71399a7e2e907ea3df4f"
        )
    }

    #[test]
    fn test_generate_sha1_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt").unwrap();
        let reader = BufReader::new(file);

        let result = GitOid::new_from_reader(HashAlgorithm::SHA1, reader, 11).unwrap();

        assert_eq!(
            "95d09f2b10159347eece71399a7e2e907ea3df4f",
            result.hex_hash()
        )
    }

    #[test]
    fn test_generate_sha256_git_oid() {
        let input = "hello world".as_bytes();

        let result = GitOid::new(HashAlgorithm::SHA256, input);

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            result.hex_hash()
        );
    }

    #[test]
    fn test_generate_sha256_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt").unwrap();
        let reader = BufReader::new(file);

        let result = GitOid::new_from_reader(HashAlgorithm::SHA256, reader, 11).unwrap();

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            result.hex_hash()
        );
    }

    #[tokio::test]
    async fn test_async_read() {
        let mut to_read = Vec::new();
        for _ in 0..50 {
            to_read.push(Source::new(
                tokio::fs::File::open("test/data/hello_world.txt")
                    .await
                    .unwrap(),
                11,
            ));
        }

        let res = GitOid::new_from_async_readers(HashAlgorithm::SHA256, to_read)
            .await
            .unwrap();

        assert_eq!(50, res.len());
        assert_eq!(
            "SHA256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            res[0].to_string()
        );
    }
}
