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
    let result = GitOid::new_from_reader(Sha1, Blob, reader)?;

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
    let result = GitOid::new_from_reader(Sha256, Blob, reader)?;

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
        gitoid.url().to_string(),
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
    let output = gitoid.url();

    eprintln!("{}", url);
    eprintln!("{}", output);

    assert_eq!(url, output);
}

#[test]
fn try_finder() {
    let hash_algorithm = HashAlgorithm::Sha256;
    let object_type = ObjectType::Blob;
    let gitoids = vec![
        GitOid::new_from_str(hash_algorithm, object_type, "a"),
        GitOid::new_from_str(hash_algorithm, object_type, "b"),
        GitOid::new_from_str(hash_algorithm, object_type, "c"),
        GitOid::new_from_str(hash_algorithm, object_type, "d"),
        GitOid::new_from_str(hash_algorithm, object_type, "e"),
        GitOid::new_from_str(hash_algorithm, object_type, "f"),
        GitOid::new_from_str(hash_algorithm, object_type, "g"),
        GitOid::new_from_str(hash_algorithm, object_type, "h"),
        GitOid::new_from_str(hash_algorithm, object_type, "i"),
        GitOid::new_from_str(hash_algorithm, object_type, "j"),
        GitOid::new_from_str(hash_algorithm, object_type, "k"),
        GitOid::new_from_str(hash_algorithm, object_type, "l"),
        GitOid::new_from_str(hash_algorithm, object_type, "m"),
        GitOid::new_from_str(hash_algorithm, object_type, "n"),
        GitOid::new_from_str(hash_algorithm, object_type, "o"),
        GitOid::new_from_str(hash_algorithm, object_type, "p"),
        GitOid::new_from_str(hash_algorithm, object_type, "q"),
        GitOid::new_from_str(hash_algorithm, object_type, "r"),
        GitOid::new_from_str(hash_algorithm, object_type, "s"),
        GitOid::new_from_str(hash_algorithm, object_type, "t"),
        GitOid::new_from_str(hash_algorithm, object_type, "u"),
        GitOid::new_from_str(hash_algorithm, object_type, "v"),
        GitOid::new_from_str(hash_algorithm, object_type, "w"),
        GitOid::new_from_str(hash_algorithm, object_type, "x"),
        GitOid::new_from_str(hash_algorithm, object_type, "y"),
        GitOid::new_from_str(hash_algorithm, object_type, "z"),
    ];

    let finder = Finder::for_gitoids(hash_algorithm, object_type, &gitoids);

    let to_find = vec![
        ("a", GitOid::new_from_str(hash_algorithm, object_type, "a")),
        ("c", GitOid::new_from_str(hash_algorithm, object_type, "c")),
        ("e", GitOid::new_from_str(hash_algorithm, object_type, "e")),
        ("g", GitOid::new_from_str(hash_algorithm, object_type, "g")),
        ("i", GitOid::new_from_str(hash_algorithm, object_type, "i")),
        ("k", GitOid::new_from_str(hash_algorithm, object_type, "k")),
        ("m", GitOid::new_from_str(hash_algorithm, object_type, "m")),
        ("o", GitOid::new_from_str(hash_algorithm, object_type, "o")),
        ("q", GitOid::new_from_str(hash_algorithm, object_type, "q")),
        ("s", GitOid::new_from_str(hash_algorithm, object_type, "s")),
        ("u", GitOid::new_from_str(hash_algorithm, object_type, "u")),
        ("w", GitOid::new_from_str(hash_algorithm, object_type, "w")),
        ("y", GitOid::new_from_str(hash_algorithm, object_type, "y")),
    ];

    let found: Vec<_> = finder.find_all(to_find).map(|(id, _gitoid)| id).collect();
    let expected = vec![
        "a", "c", "e", "g", "i", "k", "m", "o", "q", "s", "u", "w", "y",
    ];
    assert_eq!(found, expected);
}
