use super::*;
use hash_algorithm::HashAlgorithm::*;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[test]
fn generate_sha1_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::new_from_bytes(SHA1, input);

    assert_eq!(
        result.hex_hash(),
        "95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    assert_eq!(
        result.to_string(),
        "SHA1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );
}

#[test]
fn generate_sha1_gitoid_from_buffer() -> Result<()> {
    let reader = BufReader::new(File::open("test/data/hello_world.txt")?);
    let result = GitOid::new_from_reader(SHA1, reader, 11)?;

    assert_eq!(
        result.hex_hash(),
        "95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    assert_eq!(
        result.to_string(),
        "SHA1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    Ok(())
}

#[tokio::test]
async fn generate_sha1_gitoids_from_async_buffers() -> Result<()> {
    let mut to_read = Vec::new();
    for _ in 0..50 {
        to_read.push(Source::new(
            tokio::fs::File::open("test/data/hello_world.txt").await?,
            11,
        ));
    }

    let res = GitOid::new_from_async_readers(SHA1, to_read).await?;

    assert_eq!(50, res.len());
    assert_eq!(
        "SHA1:95d09f2b10159347eece71399a7e2e907ea3df4f",
        res[0].to_string()
    );

    Ok(())
}

#[test]
fn generate_sha256_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::new_from_bytes(SHA256, input);

    assert_eq!(
        result.hex_hash(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "SHA256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );
}

#[test]
fn generate_sha256_gitoid_from_buffer() -> Result<()> {
    let reader = BufReader::new(File::open("test/data/hello_world.txt")?);
    let result = GitOid::new_from_reader(SHA256, reader, 11)?;

    assert_eq!(
        result.hex_hash(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "SHA256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    Ok(())
}

#[tokio::test]
async fn generate_sha256_gitoids_from_async_buffers() -> Result<()> {
    let mut to_read = Vec::new();
    for _ in 0..50 {
        to_read.push(Source::new(
            tokio::fs::File::open("test/data/hello_world.txt").await?,
            11,
        ));
    }

    let res = GitOid::new_from_async_readers(SHA256, to_read).await?;

    assert_eq!(50, res.len());
    assert_eq!(
        "SHA256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
        res[0].to_string()
    );

    Ok(())
}
