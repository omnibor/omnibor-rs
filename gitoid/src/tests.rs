use super::*;
use hash_algorithm::HashAlgorithm::*;
use object_type::ObjectType::*;
use std::fs::File;
use std::io::BufReader;
use url::Url;

#[test]
fn generate_sha1_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::new_from_bytes(Sha1, Blob, input);

    assert_eq!(
        result.hash().as_hex(),
        "95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    assert_eq!(
        result.to_string(),
        "sha1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );
}

#[test]
fn generate_sha1_gitoid_from_buffer() -> Result<()> {
    let reader = BufReader::new(File::open("test/data/hello_world.txt")?);
    let result = GitOid::new_from_reader(Sha1, Blob, reader, 11)?;

    assert_eq!(
        result.hash().as_hex(),
        "95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    assert_eq!(
        result.to_string(),
        "sha1:95d09f2b10159347eece71399a7e2e907ea3df4f"
    );

    Ok(())
}

#[test]
fn generate_sha256_gitoid_from_bytes() {
    let input = b"hello world";
    let result = GitOid::new_from_bytes(Sha256, Blob, input);

    assert_eq!(
        result.hash().as_hex(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );
}

#[test]
fn generate_sha256_gitoid_from_buffer() -> Result<()> {
    let reader = BufReader::new(File::open("test/data/hello_world.txt")?);
    let result = GitOid::new_from_reader(Sha256, Blob, reader, 11)?;

    assert_eq!(
        result.hash().as_hex(),
        "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    assert_eq!(
        result.to_string(),
        "sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    Ok(())
}

#[test]
fn validate_uri() -> Result<()> {
    let content = b"hello world";
    let gitoid = GitOid::new_from_bytes(Sha256, Blob, content);

    assert_eq!(
        gitoid.url()?.to_string(),
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03"
    );

    Ok(())
}

#[test]
fn try_from_url_bad_scheme() {
    let url = Url::parse(
        "whatever:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
    )
    .unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "invalid scheme in URL 'whatever'"),
    }
}

#[test]
fn try_from_url_missing_object_type() {
    let url = Url::parse("gitoid:").unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing object type in URL 'gitoid:'"),
    }
}

#[test]
fn try_from_url_bad_object_type() {
    let url = Url::parse("gitoid:whatever").unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "unknown object type 'whatever'"),
    }
}

#[test]
fn try_from_url_missing_hash_algorithm() {
    let url = Url::parse("gitoid:blob:").unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(
            e.to_string(),
            "missing hash algorithm in URL 'gitoid:blob:'"
        ),
    }
}

#[test]
fn try_from_url_bad_hash_algorithm() {
    let url = Url::parse("gitoid:blob:sha10000").unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "unknown hash algorithm 'sha10000'"),
    }
}

#[test]
fn try_from_url_missing_hash() {
    let url = Url::parse("gitoid:blob:sha256:").unwrap();

    match GitOid::new_from_url(url) {
        Ok(_) => panic!("gitoid parsing should fail"),
        Err(e) => assert_eq!(e.to_string(), "missing hash in URL 'gitoid:blob:sha256:'"),
    }
}

#[test]
fn try_url_roundtrip() {
    let url = Url::parse(
        "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
    )
    .unwrap();
    let gitoid = GitOid::new_from_url(url.clone()).unwrap();
    let output = gitoid.url().unwrap();

    eprintln!("{}", url);
    eprintln!("{}", output);

    assert_eq!(url, output);
}
