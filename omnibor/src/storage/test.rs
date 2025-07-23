use super::FileSystemStorage;
use crate::{artifact_id::ArtifactId, hash_algorithm::Sha256, pathbuf};
use std::str::FromStr;

#[cfg(feature = "provider-rustcrypto")]
#[test]
fn correct_aid_storage_path() {
    let root = pathbuf![env!("CARGO_MANIFEST_DIR"), "test", "fs_storage"];
    let storage = FileSystemStorage::new(&root).unwrap();

    let aid = ArtifactId::<Sha256>::from_str(
        "gitoid:blob:sha256:9d09789f20162dca6d80d2d884f46af22c824f6409d4f447332d079a2d1e364f",
    )
    .unwrap();

    let path = storage.manifest_path(aid);
    let path = path.strip_prefix(&root).unwrap();
    let expected = pathbuf![
        "manifests",
        "gitoid_blob_sha256",
        "9d",
        "09789f20162dca6d80d2d884f46af22c824f6409d4f447332d079a2d1e364f"
    ];

    assert_eq!(path, expected);
}
