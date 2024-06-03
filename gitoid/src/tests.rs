#![allow(unused_imports)]

use super::*;
#[cfg(feature = "std")]
use std::fs::File;
#[cfg(feature = "async")]
use tokio::fs::File as AsyncFile;
#[cfg(feature = "async")]
use tokio::runtime::Runtime;
#[cfg(feature = "url")]
use url::Url;
#[cfg(feature = "serde")]
use {
    crate::Blob,
    crate::GitOid,
    serde_test::{assert_tokens, Token},
};

#[cfg(feature = "sha256")]
use crate::Sha256;

#[cfg(all(feature = "sha1", feature = "hex"))]
#[test]
fn generate_sha1_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::<Sha1, Blob>::id_bytes(input);

    assert_eq!(result.as_hex(), "95d09f2b10159347eece71399a7e2e907ea3df4f");

    assert_eq!(
        result.to_string(),
        "gitoid:blob:sha1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );
}

#[cfg(all(feature = "sha1", feature = "std"))]
#[test]
fn generate_sha1_gitoid_from_buffer() -> Result<()> {
    let reader = File::open("test/data/hello_world.txt")?;
    let result = GitOid::<Sha1, Blob>::id_reader(reader)?;

    assert_eq!(result.as_hex(), "95d09f2b10159347eece71399a7e2e907ea3df4f");

    assert_eq!(
        result.to_string(),
        "gitoid:blob:sha1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    Ok(())
}

#[cfg(feature = "sha256")]
#[test]
fn generate_sha256_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::<Sha256, Blob>::id_bytes(input);

    assert_eq!(
        result.as_hex(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );
}

#[cfg(all(feature = "sha256", feature = "std"))]
#[test]
fn generate_sha256_gitoid_from_buffer() -> Result<()> {
    let reader = File::open("test/data/hello_world.txt")?;
    let result = GitOid::<Sha256, Blob>::id_reader(reader)?;

    assert_eq!(
        result.as_hex(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    Ok(())
}

#[cfg(all(feature = "sha256", feature = "async"))]
#[test]
fn generate_sha256_gitoid_from_async_buffer() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let reader = AsyncFile::open("test/data/hello_world.txt").await?;
        let result = GitOid::<Sha256, Blob>::id_async_reader(reader).await?;

        assert_eq!(
            result.as_hex(),
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
        );

        assert_eq!(
            result.to_string(),
            "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
        );

        Ok(())
    })
}

#[cfg(feature = "sha256")]
#[test]
fn validate_uri() -> Result<()> {
    let content = b"hello world";
    let gitoid = GitOid::<Sha256, Blob>::id_bytes(content);

    assert_eq!(
        gitoid.url().to_string(),
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    Ok(())
}

#[cfg(all(feature = "sha256", feature = "url"))]
#[test]
fn try_from_url_bad_scheme() {
    let url = Url::parse(
        "whatever:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
    )
    .unwrap();

    match GitOid::<Sha256, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid scheme in URL 'whatever'"),
    }
}

#[cfg(all(feature = "sha1", feature = "url"))]
#[test]
fn try_from_url_missing_object_type() {
    let url = Url::parse("gitoid:").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing object type in URL 'gitoid:'"),
    }
}

#[cfg(all(feature = "sha1", feature = "url"))]
#[test]
fn try_from_url_bad_object_type() {
    let url = Url::parse("gitoid:whatever").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "mismatched object type; expected 'blob'"),
    }
}

#[cfg(all(feature = "sha256", feature = "url"))]
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

#[cfg(all(feature = "sha1", feature = "url"))]
#[test]
fn try_from_url_bad_hash_algorithm() {
    let url = Url::parse("gitoid:blob:sha10000").unwrap();

    match GitOid::<Sha1, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "mismatched hash algorithm; expected 'sha1'"),
    }
}

#[cfg(all(feature = "sha256", feature = "url"))]
#[test]
fn try_from_url_missing_hash() {
    let url = Url::parse("gitoid:blob:sha256:").unwrap();

    match GitOid::<Sha256, Blob>::try_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing hash in URL 'gitoid:blob:sha256:'"),
    }
}

#[cfg(all(feature = "sha256", feature = "url"))]
#[test]
fn try_url_roundtrip() {
    let url = Url::parse(
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
    )
    .unwrap();
    let gitoid = GitOid::<Sha256, Blob>::try_from_url(url.clone()).unwrap();
    let output = gitoid.url();
    assert_eq!(url, output);
}

#[cfg(all(feature = "serde", feature = "sha256"))]
#[test]
fn valid_gitoid_ser_de() {
    let id = GitOid::<Sha256, Blob>::id_str("hello, world");

    // This validates both serialization and deserialization.
    assert_tokens(
        &id,
        &[Token::Str(
            "gitoid:blob:sha256:7d0be525d6521168c74051e5ab1b99e3b6d1c962fba763818f1954ab9e1c821a",
        )],
    );
}
