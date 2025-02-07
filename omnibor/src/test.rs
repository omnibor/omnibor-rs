//! Tests against the OmniBOR crate as a whole.

use {
    crate::{
        artifact_id::{ArtifactId, ArtifactIdBuilder},
        hash_algorithm::Sha256,
    },
    anyhow::Result,
    serde_test::{assert_tokens, Token},
    std::fs::File,
    tokio::{fs::File as AsyncFile, runtime::Runtime},
    url::Url,
};

/// SHA-256 hash of a file containing "hello world"
///
/// Taken from a Git repo as ground truth.
const ARTIFACT_ID_HELLO_WORLD_SHA256: &str =
    "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";

/// ArtifactID should be exactly 32 bytes, the size of the buffer.
#[test]
fn artifact_id_sha256_size() {
    assert_eq!(size_of::<ArtifactId<Sha256>>(), 32);
}

#[test]
fn generate_sha256_artifact_id_from_bytes() {
    let input = b"hello world";
    let result = ArtifactIdBuilder::with_rustcrypto().identify_bytes(input);

    assert_eq!(result.to_string(), ARTIFACT_ID_HELLO_WORLD_SHA256);
}

#[test]
fn generate_sha256_artifact_id_from_buffer() -> Result<()> {
    let mut file = File::open("test/data/hello_world.txt")?;
    let result = ArtifactIdBuilder::with_rustcrypto().identify_file(&mut file)?;

    assert_eq!(result.to_string(), ARTIFACT_ID_HELLO_WORLD_SHA256);

    Ok(())
}

#[test]
fn generate_sha256_artifact_id_from_async_buffer() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let mut file = AsyncFile::open("test/data/hello_world.txt").await?;
        let result = ArtifactIdBuilder::with_rustcrypto()
            .identify_async_file(&mut file)
            .await?;

        assert_eq!(result.to_string(), ARTIFACT_ID_HELLO_WORLD_SHA256);

        Ok(())
    })
}

#[test]
fn newline_normalization_from_file() -> Result<()> {
    let mut unix_file = File::open("test/data/unix_line.txt")?;
    let mut windows_file = File::open("test/data/windows_line.txt")?;

    let builder = ArtifactIdBuilder::with_rustcrypto();

    let unix_artifact_id = builder.identify_file(&mut unix_file)?;
    let windows_artifact_id = builder.identify_file(&mut windows_file)?;

    assert_eq!(
        unix_artifact_id.to_string(),
        windows_artifact_id.to_string()
    );

    Ok(())
}

#[test]
fn newline_normalization_from_async_file() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.block_on(async {
        let mut unix_file = AsyncFile::open("test/data/unix_line.txt").await?;
        let mut windows_file = AsyncFile::open("test/data/windows_line.txt").await?;

        let builder = ArtifactIdBuilder::with_rustcrypto();

        let unix_artifact_id = builder.identify_async_file(&mut unix_file).await?;
        let windows_artifact_id = builder.identify_async_file(&mut windows_file).await?;

        assert_eq!(
            unix_artifact_id.to_string(),
            windows_artifact_id.to_string()
        );

        Ok(())
    })
}

#[test]
fn newline_normalization_in_memory() -> Result<()> {
    let with_crlf = b"some\r\nstring\r\n";
    let wout_crlf = b"some\nstring\n";

    let builder = ArtifactIdBuilder::with_rustcrypto();

    let with_crlf_artifact_id = builder.identify_bytes(&with_crlf[..]);
    let wout_crlf_artifact_id = builder.identify_bytes(&wout_crlf[..]);

    assert_eq!(
        with_crlf_artifact_id.to_string(),
        wout_crlf_artifact_id.to_string()
    );

    Ok(())
}

#[test]
fn validate_uri() -> Result<()> {
    let content = b"hello world";
    let artifact_id = ArtifactIdBuilder::with_rustcrypto().identify_bytes(content);

    assert_eq!(
        artifact_id.url().to_string(),
        ARTIFACT_ID_HELLO_WORLD_SHA256
    );

    Ok(())
}

#[test]
#[should_panic]
fn try_from_url_bad_scheme() {
    let url = Url::parse("gitiod:blob:sha1:95d09f2b10159347eece71399a7e2e907ea3df4f").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
#[should_panic]
fn try_from_url_missing_object_type() {
    let url = Url::parse("gitoid:").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
#[should_panic]
fn try_from_url_bad_object_type() {
    let url = Url::parse("gitoid:whatever").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
#[should_panic]
fn try_from_url_missing_hash_algorithm() {
    let url = Url::parse("gitoid:blob:").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
#[should_panic]
fn try_from_url_bad_hash_algorithm() {
    let url = Url::parse("gitoid:blob:sha10000").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
#[should_panic]
fn try_from_url_missing_hash() {
    let url = Url::parse("gitoid:blob:sha256:").unwrap();
    ArtifactId::<Sha256>::try_from_url(url).unwrap();
}

#[test]
fn try_url_roundtrip() {
    let url = Url::parse(ARTIFACT_ID_HELLO_WORLD_SHA256).unwrap();
    let artifact_id = ArtifactId::<Sha256>::try_from_url(url.clone()).unwrap();
    let output = artifact_id.url();
    assert_eq!(url, output);
}

#[test]
fn valid_artifact_id_ser_de() {
    let id = ArtifactIdBuilder::with_rustcrypto().identify_string("hello world");
    assert_tokens(&id, &[Token::Str(ARTIFACT_ID_HELLO_WORLD_SHA256)]);
}
