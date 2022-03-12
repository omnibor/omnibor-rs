use im::Vector;
use sha2::{digest::DynDigest, Digest, Sha256};
use std::io::{BufReader, Error, ErrorKind, Read, Result as IOResult};

/// The available algorithms for computing hashes
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub enum HashAlgorithm {
    /// [SHA1](https://en.wikipedia.org/wiki/SHA-1)
    SHA1,
    /// [SHA256](https://en.wikipedia.org/wiki/SHA-2)
    SHA256,
}

/// A struct that computes [git oids](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
/// based on the selected algorithm
#[derive(Clone, Copy, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitOid {
    hash_algorithm: HashAlgorithm,
}

impl GitOid {
    pub fn new(hash_algo: HashAlgorithm) -> Self {
        GitOid {
            hash_algorithm: hash_algo,
        }
    }
    /// Given a byte array, generate a hash based on the `GitOid`'s
    /// hashing algorithm
    pub fn generate_git_oid(&self, content: &[u8]) -> String {
        let r = BufReader::new(content);

        // normally `unwrap` is code smell... but in this case, we know
        // that there will not be an I/O error reading the content and
        // we know that the length of the content is the number of bytes
        // to be read
        return self.generate_git_oid_from_buffer(r, content.len()).unwrap();
    }

    /// Based on the `GitOid`'s hashing algorithm, generate an instance of
    /// a digester
    fn create_digest(&self) -> Box<dyn DynDigest> {
        let ret: Box<dyn sha2::digest::DynDigest> = match self.hash_algorithm {
            HashAlgorithm::SHA1 => Box::new(sha1::Sha1::new()),
            HashAlgorithm::SHA256 => Box::new(Sha256::new()),
        };

        return ret;
    }

    /// Take a `BufReader` and generate a hash based on the `GitOid`'s hashing
    /// algorithm. Will return an `Err` if the `BufReader` generates an `Err`
    /// or if the `expected_length` is different from the actual length. Why
    /// the latter `Err`? The prefix string includes the number of bytes
    /// being hashed and that's the `expected_length`. If the actual bytes
    /// hashed differs, then something went wrong and the hash is not valid
    pub fn generate_git_oid_from_buffer<R>(
        &self,
        mut reader: BufReader<R>,
        expected_length: usize,
    ) -> IOResult<String>
    where
        BufReader<R>: std::io::Read,
    {
        let prefix = format!("blob {}\0", expected_length);

        let mut buf = [0; 4096]; // Linux default page size is 4096
        let mut amount_read: usize = 0;

        let mut hasher = self.create_digest();

        // set the prefix
        hasher.update(prefix.as_bytes());

        // keep reading the input until there is no more
        loop {
            match reader.read(&mut buf) {
                // done
                Ok(0) => {
                    break;
                }

                // update the hash and accumulate the count
                Ok(size) => {
                    hasher.update(&buf[..size]);
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

        let hash = hasher.finalize();
        return Ok(hex::encode(hash));
    }
}

/// A [persistent](https://en.wikipedia.org/wiki/Persistent_data_structure) collection
/// of [git oids](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects).
/// Why persistent? While Rust and the borrow checker is great about ownership and
/// mutation, always knowing that a Ref will not change if passed as a parameter
/// to a function eliminates a class of errors.
#[derive(Clone, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct GitBom {
    git_oids: Vector<String>,
}

impl GitBom {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            git_oids: Vector::new(),
        }
    }

    /// Append a `gitoid` hash and return a new instance of the
    /// `GitBom` that includes the appended item.
    ///
    /// Why `ToString` rather than `String` or `&str` or other stuff?
    /// Mostly convenience. Make it easy to call the function.
    pub fn add<I>(&self, gitoid: I) -> Self
    where
        I: ToString,
    {
        self.add_many(vec![gitoid])
    }

    /// Append many git oids and return a new `GitBom`
    pub fn add_many<I, S>(&self, gitoids: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let mut updated = self.git_oids.clone(); // im::Vector has O(1) cloning
        for gitoid in gitoids {
            updated.push_back(gitoid.to_string());
        }
        Self { git_oids: updated }
    }

    /// Return the `Vector` of git oids
    pub fn get_vector(&self) -> Vector<String> {
        self.git_oids.clone()
    }
}

#[cfg(test)]
mod tests {
    use im::vector;
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_add() {
        assert_eq!(
            GitBom::new().add("Hello").get_vector(),
            vector!["Hello".to_string()]
        )
    }

    #[test]
    fn test_add_many() {
        assert_eq!(
            GitBom::new()
                .add_many(vec!["Hello", "Cat", "Dog"])
                .get_vector(),
            vector!["Hello".to_string(), "Cat".to_string(), "Dog".to_string()]
        )
    }

    #[test]
    fn test_generate_sha1_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid::new(HashAlgorithm::SHA1);

        let result = new_gitoid.generate_git_oid(input);
        assert_eq!(result, "95d09f2b10159347eece71399a7e2e907ea3df4f")
    }

    #[test]
    fn test_generate_sha1_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt");
        match file {
            Ok(f) => {
                let reader = BufReader::new(f);

                let new_gitoid = GitOid::new(HashAlgorithm::SHA1);

                let result = new_gitoid.generate_git_oid_from_buffer(reader, 11).unwrap();

                assert_eq!("95d09f2b10159347eece71399a7e2e907ea3df4f", result)
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_generate_sha256_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid::new(HashAlgorithm::SHA256);

        let result = new_gitoid.generate_git_oid(input);

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            result
        );
    }

    #[test]
    fn test_generate_sha256_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt");
        match file {
            Ok(f) => {
                let reader = BufReader::new(f);

                let new_gitoid = GitOid::new(HashAlgorithm::SHA256);

                let result = new_gitoid.generate_git_oid_from_buffer(reader, 11).unwrap();

                assert_eq!(
                    "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
                    result
                );
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_add_gitoid_to_gitbom() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA256,
        };

        let generated_gitoid = new_gitoid.generate_git_oid(input);

        let new_gitbom = GitBom::new();
        let new_gitbom = new_gitbom.add(generated_gitoid);

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            new_gitbom.get_vector()[0]
        )
    }
}
