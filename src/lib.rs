use im::{HashSet, Vector};
use sha2::{digest::DynDigest, Digest, Sha256};
use std::fmt::{Display, Formatter, Result};
use std::io::{BufReader, Error, ErrorKind, Read, Result as IOResult};

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
    value: [u8; 32],
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
        match GitOid::generate_git_oid_from_buffer(digest, content, expected_length) {
            Ok(v) => Ok(GitOid {
                hash_algorithm: hash_algo,
                len: v.0,
                value: v.1,
            }),
            Err(e) => Err(e),
        }
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
    ) -> IOResult<(usize, [u8; 32])>
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
            match reader.read(&mut buf) {
                // done
                Ok(0) => {
                    break;
                }

                // update the hash and accumulate the count
                Ok(size) => {
                    digest.update(&buf[..size]);
                    amount_read = amount_read + size;
                }

                // got an error? return it.
                Err(x) => {
                    return Err(x);
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
        let mut ret = [0u8; 32];

        let len = std::cmp::min(32, hash.len());
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

impl GitBom {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            git_oids: HashSet::new(),
        }
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
}
