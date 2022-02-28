extern crate alloc;
extern crate std;

use alloc::format;
use alloc::string::{String, ToString};
use std::io::{BufReader, Read};
use sha2::{Sha256, Digest};

use base64ct::{Base64, Encoding};
use base16ct;

enum HashAlgorithm {
    SHA1,
    SHA256
}

struct GitOid {
    hash_algorithm: HashAlgorithm,
}

impl GitOid {
    //pub fn generate_sha1_git_oid(&self, x: &[u8]) -> String {
    pub fn generate_git_oid(&self, x: &[u8]) -> String {
        let prefix = format!("blob {}\0", x.len());

        match self.hash_algorithm {
            HashAlgorithm::SHA1 => {
                let mut hasher = sha1_smol::Sha1::new();

                hasher.update(prefix.as_bytes());
                hasher.update(x);

                hasher.digest().to_string()
            },
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(x);

                let hash = hasher.finalize();
                println!("Binary hash: {:?}", hash);

                let hash_string = Base64::encode_string(&hash);
                println!("Base64-encoded hash: {}", hash_string);

                return Base64::encode_string(&hash)
            }
        }
    }

    pub fn generate_sha1_git_oid_from_buffer<R>(
        &self,
        mut reader: BufReader<R>,
        expected_length: usize,
    ) -> String
    where
        BufReader<R>: std::io::Read,
    {
        let prefix = format!("blob {}\0", expected_length);
        let mut hasher = sha1_smol::Sha1::new();

        hasher.update(prefix.as_bytes());

        let mut buf = [0; 4096]; // linux default page size is 4096
        let mut amount_read = 0;
        loop {
            let y = reader.read(&mut buf);
            match y {
                Ok(0) => {
                    break;
                }
                Ok(size) => {
                    hasher.update(&buf[..size]);
                    amount_read = amount_read + size;
                }
                Err(_) => {
                    break;
                }
            }
        }

        hasher.digest().to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_generate_sha1_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA1
        };

        let result = new_gitoid.generate_git_oid(input);
        assert_eq!(result, "95d09f2b10159347eece71399a7e2e907ea3df4f")
    }

    #[test]
    fn test_generate_sha1_git_oid_buffer() {
        let file = File::open("test/data/hello_world.txt");
        match file {
            Ok(f) => {
                let reader = BufReader::new(f);

                let new_gitoid = GitOid {
                    hash_algorithm: HashAlgorithm::SHA256
                };

                let result = new_gitoid.generate_sha1_git_oid_from_buffer(reader, 11);

                assert_eq!("95d09f2b10159347eece71399a7e2e907ea3df4f", result)
            }
            Err(_) => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_generate_sha2_git_oid() {
        let input = "hello world".as_bytes();

        let new_gitoid = GitOid {
            hash_algorithm: HashAlgorithm::SHA256
        };

        let result = new_gitoid.generate_git_oid(input);

        assert_eq!("uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=", result);
    }
}
