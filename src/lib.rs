use im::{HashSet, Vector};
use pin_project::pin_project;
use sha2::{digest::DynDigest, Digest, Sha256};
use std::fmt::{Display, Formatter, Result};
use std::io;
use std::io::{BufReader, Error, ErrorKind, Read, Result as IOResult};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

#[pin_project]
pub struct Source<R> {
    #[pin]
    reader: R,
    length: usize,
}

impl<R> Source<R> {
    // name as `new` maybe?
    pub fn new(reader: R, length: usize) -> Self {
        Self { reader, length }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    // maybe also from_file ?
}

impl<R: AsyncRead> AsyncRead for Source<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        this.reader.poll_read(cx, buf)
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
    pub fn create_digest(&self) -> Box<dyn DynDigest> {
        let ret: Box<dyn sha2::digest::DynDigest> = match self {
            HashAlgorithm::SHA1 => Box::new(sha1::Sha1::new()),
            HashAlgorithm::SHA256 => Box::new(Sha256::new()),
        };

        return ret;
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
    hash_algorithm: HashAlgorithm,
    len: usize,
    value: [u8; NUM_HASH_BYTES],
}

impl Display for GitOid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}", self.hash_algorithm, self.hex_hash())
    }
}

impl GitOid {
    /// return the hex value of the hashcode, without the hash type
    pub fn hex_hash(&self) -> String {
        hex::encode(&self.value[0..self.len])
    }

    /// get a slice with the hash value. The lifetime of the slice
    /// is the same as the lifetime of the GitOid
    pub fn hash_value<'a>(&'a self) -> &'a [u8] {
        &self.value[0..self.len]
    }

    /// Get the hash algorithm used for this GitOid
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.hash_algorithm
    }

    /// create a new GitOid based on an in-memory array
    pub fn new(hash_algo: HashAlgorithm, content: &[u8]) -> Self {
        let v = GitOid::generate_git_oid_from_buffer(
            hash_algo.create_digest(),
            BufReader::new(content),
            content.len(),
        )
        .unwrap(); // `unwrap` is usually code smell. In this case, we know there will be no I/O errors and the length will be correct
        GitOid {
            hash_algorithm: hash_algo,
            value: v.1,
            len: v.0,
        }
    }

    /// create a GitOid using SHA256 for the string... mostly a helper method
    pub fn new_from_str(the_string: &str) -> Self {
        GitOid::new(HashAlgorithm::SHA256, the_string.as_bytes())
    }

    pub fn new_from_reader<R>(
        hash_algo: HashAlgorithm,
        content: BufReader<R>,
        expected_length: usize,
    ) -> IOResult<Self>
    where
        BufReader<R>: std::io::Read,
    {
        let digest = hash_algo.create_digest();
        let v = GitOid::generate_git_oid_from_buffer(digest, content, expected_length)?;
        Ok(GitOid {
            hash_algorithm: hash_algo,
            len: v.0,
            value: v.1,
        })
    }

    /// generate a bunch of `GitOid`s from a bunch of async
    /// readers for a given algorithm
    pub async fn new_from_async_readers<R, I>(
        hash_algo: HashAlgorithm,
        content: I,
    ) -> IOResult<Vector<GitOid>>
    where
        R: AsyncReadExt + std::marker::Unpin,
        I: IntoIterator<Item = Source<R>>,
    {
        let digest = hash_algo.create_digest();
        let mut future_vec = Vec::new();

        // go get the futures for each hash operation
        for reader in content {
            let expected_length = reader.len();
            future_vec.push(GitOid::generate_git_oid_from_async_buffer(
                digest.clone(),
                reader,
                expected_length,
            ));
        }

        // create the return vector
        let mut ret = Vector::new();

        // go through each future and await the response
        // the cool thing is that this will block on any given future
        // but other futures may become satisfied so the look effectively
        // blocks on the longest-to-satisfy future
        for res in futures::future::join_all(future_vec).await {
            let (len, bytes) = res?;
            ret.push_back(GitOid {
                hash_algorithm: hash_algo,
                len: len,
                value: bytes,
            });
        }

        Ok(ret)
    }

    /// the async version of generating a git_oid from a buffer
    async fn generate_git_oid_from_async_buffer<R>(
        mut digest: Box<dyn DynDigest>,
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
        digest.update(prefix.as_bytes());

        // keep reading the input until there is no more
        loop {
            match reader.read(&mut buf).await? {
                // done
                0 => break,

                // update the hash and accumulate the count
                size => {
                    digest.update(&buf[..size]);
                    amount_read = amount_read + size;
                }
            }
        }

        // make sure we got the length we expected
        if amount_read != expected_length {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Expected length {} actual length {}",
                    expected_length, amount_read
                ),
            ));
        }

        let hash = digest.finalize();
        let mut ret = [0u8; NUM_HASH_BYTES];

        let len = NUM_HASH_BYTES.min(hash.len());
        ret[..len].copy_from_slice(&hash);
        return Ok((len, ret));
    }

    /// Take a `BufReader` and generate a hash based on the `GitOid`'s hashing
    /// algorithm. Will return an `Err` if the `BufReader` generates an `Err`
    /// or if the `expected_length` is different from the actual length. Why
    /// the latter `Err`? The prefix string includes the number of bytes
    /// being hashed and that's the `expected_length`. If the actual bytes
    /// hashed differs, then something went wrong and the hash is not valid
    fn generate_git_oid_from_buffer<R>(
        mut digest: Box<dyn DynDigest>,
        mut reader: BufReader<R>,
        expected_length: usize,
    ) -> IOResult<(usize, [u8; NUM_HASH_BYTES])>
    where
        BufReader<R>: std::io::Read,
    {
        let prefix = format!("blob {}\0", expected_length);

        let mut buf = [0; 4096]; // Linux default page size is 4096
        let mut amount_read: usize = 0;

        // set the prefix
        digest.update(prefix.as_bytes());

        // keep reading the input until there is no more
        loop {
            match reader.read(&mut buf)? {
                // done
                0 => break,

                // update the hash and accumulate the count
                size => {
                    digest.update(&buf[..size]);
                    amount_read = amount_read + size;
                }
            }
        }

        // make sure we got the length we expected
        if amount_read != expected_length {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Expected length {} actual length {}",
                    expected_length, amount_read
                ),
            ));
        }

        let hash = digest.finalize();
        let mut ret = [0u8; NUM_HASH_BYTES];

        let len = std::cmp::min(NUM_HASH_BYTES, hash.len());
        ret[..len].copy_from_slice(&hash);
        return Ok((len, ret));
    }
}

/// A [persistent](https://en.wikipedia.org/wiki/Persistent_data_structure) collection
/// of [git oids](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects).
/// Why persistent? While Rust and the borrow checker is great about ownership and
/// mutation, always knowing that a Ref will not change if passed as a parameter
/// to a function eliminates a class of errors.
#[derive(Clone, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitBom {
    git_oids: HashSet<GitOid>,
}

impl FromIterator<GitOid> for GitBom {
    /// Create a GitBom from many GitOids
    fn from_iter<T>(gitoids: T) -> Self
    where
        T: IntoIterator<Item = GitOid>,
    {
        let me = GitBom::new();
        me.add_many(gitoids)
    }
}

impl GitBom {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            git_oids: HashSet::new(),
        }
    }

    /// Create a GitBom from many GitOids
    pub fn new_from_iterator<I>(gitoids: I) -> Self
    where
        I: IntoIterator<Item = GitOid>,
    {
        let me = GitBom::new();
        me.add_many(gitoids)
    }

    /// Append a `gitoid` hash and return a new instance of the
    /// `GitBom` that includes the appended item.
    ///
    /// Why `ToString` rather than `String` or `&str` or other stuff?
    /// Mostly convenience. Make it easy to call the function.
    pub fn add(&self, gitoid: GitOid) -> Self {
        self.add_many(vec![gitoid])
    }

    /// Append many git oids and return a new `GitBom`
    pub fn add_many<I>(&self, gitoids: I) -> Self
    where
        I: IntoIterator<Item = GitOid>,
    {
        let mut updated = self.git_oids.clone(); // im::HashSet has O(1) cloning
        for gitoid in gitoids {
            updated = updated.update(gitoid);
        }
        Self { git_oids: updated }
    }

    /// Return the `Vector` of git oids
    pub fn get_oids(&self) -> HashSet<GitOid> {
        self.git_oids.clone()
    }

    /// In some cases, getting a sorted `Vector` of oids is desirable.
    /// This function (cost O(n log n)) returns a `Vector` of sorted oids
    pub fn get_sorted_oids(&self) -> Vector<GitOid> {
        let mut ret: Vector<GitOid> = self.git_oids.clone().into_iter().collect();
        ret.sort();
        return ret;
    }
}

#[cfg(test)]
mod tests {
    use im::vector;
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_add() {
        let oid = GitOid::new_from_str("Hello");
        assert_eq!(GitBom::new().add(oid).get_sorted_oids(), vector![oid])
    }

    #[test]
    fn test_add_many() {
        let mut oids: Vector<GitOid> = vec!["eee", "Hello", "Cat", "Dog"]
            .into_iter()
            .map(GitOid::new_from_str)
            .collect();

        let da_bom = GitBom::new().add_many(oids.clone());
        oids.sort();
        assert_eq!(da_bom.get_sorted_oids(), oids);
    }

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

    #[test]
    fn test_add_gitoid_to_gitbom() {
        let input = "hello world".as_bytes();

        let generated_gitoid = GitOid::new(HashAlgorithm::SHA256, input);

        let new_gitbom = GitBom::new();
        let new_gitbom = new_gitbom.add(generated_gitoid);

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            new_gitbom.get_sorted_oids()[0].hex_hash()
        )
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

        let gitbom = res.into_iter().collect::<GitBom>();

        // even though we created 50 gitoids, they should all be the same and thus
        // the gitbom should only have one entry
        assert_eq!(1, gitbom.get_oids().len());
    }
}
