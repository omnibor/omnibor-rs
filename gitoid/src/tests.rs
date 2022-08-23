use super::*;
use std::fs::File;
use std::io::BufReader;

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
