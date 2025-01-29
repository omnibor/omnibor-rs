use super::*;

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto"))]
use crate::rustcrypto::Sha1;

#[cfg(all(feature = "hash-sha256", feature = "backend-rustcrypto"))]
use crate::rustcrypto::Sha256;

#[cfg(feature = "async")]
use tokio::{fs::File as AsyncFile, runtime::Runtime};

#[cfg(feature = "std")]
use {
    crate::{Blob, GitOid},
    serde_test::{assert_tokens, Token},
    std::fs::File,
    url::Url,
};

/// SHA-1 hash of a file containing "hello world"
///
/// Taken from a Git repo as ground truth.
#[cfg(feature = "hash-sha1")]
const GITOID_HELLO_WORLD_SHA1: &str = "gitoid:blob:sha1:95d09f2b10159347eece71399a7e2e907ea3df4f";

/// SHA-256 hash of a file containing "hello world"
///
/// Taken from a Git repo as ground truth.
#[cfg(feature = "hash-sha256")]
const GITOID_HELLO_WORLD_SHA256: &str =
    "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto", feature = "std"))]
#[test]
fn generate_sha1_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::<Sha1, Blob>::id_bytes(input);

    assert_eq!(result.to_string(), GITOID_HELLO_WORLD_SHA1);
}

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto", feature = "std"))]
#[test]
fn generate_sha1_gitoid_from_buffer() -> Result<()> {
    let reader = File::open("test/data/hello_world.txt")?;
    let result = GitOid::<Sha1, Blob>::id_reader(reader)?;

    assert_eq!(result.to_string(), GITOID_HELLO_WORLD_SHA1);

    Ok(())
}

#[cfg(all(feature = "hash-sha256", feature = "backend-rustcrypto"))]
#[test]
fn generate_sha256_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::<Sha256, Blob>::id_bytes(input);

    assert_eq!(result.to_string(), GITOID_HELLO_WORLD_SHA256);
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "std"
))]
#[test]
fn generate_sha256_gitoid_from_buffer() -> Result<()> {
    let reader = File::open("test/data/hello_world.txt")?;
    let result = GitOid::<Sha256, Blob>::id_reader(reader)?;

    assert_eq!(result.to_string(), GITOID_HELLO_WORLD_SHA256);

    Ok(())
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "async"
))]
#[test]
fn generate_sha256_gitoid_from_async_buffer() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let reader = AsyncFile::open("test/data/hello_world.txt").await?;
        let result = GitOid::<Sha256, Blob>::id_async_reader(reader).await?;

        assert_eq!(result.to_string(), GITOID_HELLO_WORLD_SHA256);

        Ok(())
    })
}

#[cfg(all(feature = "hash-sha256", feature = "backend-rustcrypto"))]
#[test]
fn newline_normalization_from_file() -> Result<()> {
    let unix_file = File::open("test/data/unix_line.txt")?;
    let windows_file = File::open("test/data/windows_line.txt")?;

    let unix_gitoid = GitOid::<Sha256, Blob>::id_reader(unix_file)?;
    let windows_gitoid = GitOid::<Sha256, Blob>::id_reader(windows_file)?;

    assert_eq!(unix_gitoid.to_string(), windows_gitoid.to_string());

    Ok(())
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "async"
))]
#[test]
fn newline_normalization_from_async_file() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let unix_file = AsyncFile::open("test/data/unix_line.txt").await?;
        let windows_file = AsyncFile::open("test/data/windows_line.txt").await?;

        let unix_gitoid = GitOid::<Sha256, Blob>::id_async_reader(unix_file).await?;
        let windows_gitoid = GitOid::<Sha256, Blob>::id_async_reader(windows_file).await?;

        assert_eq!(unix_gitoid.to_string(), windows_gitoid.to_string());

        Ok(())
    })
}

#[cfg(all(feature = "hash-sha256", feature = "backend-rustcrypto"))]
#[test]
fn newline_normalization_in_memory() -> Result<()> {
    let with_crlf = b"some\r\nstring\r\n";
    let wout_crlf = b"some\nstring\n";

    let with_crlf_gitoid = GitOid::<Sha256, Blob>::id_bytes(&with_crlf[..]);
    let wout_crlf_gitoid = GitOid::<Sha256, Blob>::id_bytes(&wout_crlf[..]);

    assert_eq!(with_crlf_gitoid.to_string(), wout_crlf_gitoid.to_string());

    Ok(())
}

#[cfg(all(feature = "hash-sha256", feature = "backend-rustcrypto"))]
#[test]
fn validate_uri() -> Result<()> {
    let content = b"hello world";
    let gitoid = GitOid::<Sha256, Blob>::id_bytes(content);

    assert_eq!(gitoid.url().to_string(), GITOID_HELLO_WORLD_SHA256);

    Ok(())
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "std"
))]
#[test]
fn try_from_url_bad_scheme() {
    let url = Url::parse("gitiod:blob:sha1:95d09f2b10159347eece71399a7e2e907ea3df4f").unwrap();

    match GitOid::<Sha256, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid scheme in URL 'gitiod'"),
    }
}

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto", feature = "std"))]
#[test]
fn try_from_url_missing_object_type() {
    let url = Url::parse("gitoid:").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing object type in URL 'gitoid:'"),
    }
}

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto", feature = "std"))]
#[test]
fn try_from_url_bad_object_type() {
    let url = Url::parse("gitoid:whatever").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "mismatched object type; expected 'blob'"),
    }
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "std"
))]
#[test]
fn try_from_url_missing_hash_algorithm() {
    let url = Url::parse("gitoid:blob:").unwrap();

    match GitOid::<Sha256, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(
            e.to_string(),
            "missing hash algorithm in URL 'gitoid:blob:'"
        ),
    }
}

#[cfg(all(feature = "hash-sha1", feature = "backend-rustcrypto", feature = "std"))]
#[test]
fn try_from_url_bad_hash_algorithm() {
    let url = Url::parse("gitoid:blob:sha10000").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "mismatched hash algorithm; expected 'sha1'"),
    }
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "std"
))]
#[test]
fn try_from_url_missing_hash() {
    let url = Url::parse("gitoid:blob:sha256:").unwrap();

    match GitOid::<Sha256, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing hash in URL 'gitoid:blob:sha256:'"),
    }
}

#[cfg(all(
    feature = "hash-sha256",
    feature = "backend-rustcrypto",
    feature = "std"
))]
#[test]
fn try_url_roundtrip() {
    let url = Url::parse(GITOID_HELLO_WORLD_SHA256).unwrap();
    let gitoid = GitOid::<Sha256, Blob>::try_from_url(url.clone()).unwrap();
    let output = gitoid.url();
    assert_eq!(url, output);
}

// Validate serialization and deserialization work as expected.
#[cfg(all(
    feature = "std",
    feature = "hash-sha256",
    feature = "backend-rustcrypto"
))]
#[test]
fn valid_gitoid_ser_de() {
    let id = GitOid::<Sha256, Blob>::id_str("hello world");
    assert_tokens(&id, &[Token::Str(GITOID_HELLO_WORLD_SHA256)]);
}
