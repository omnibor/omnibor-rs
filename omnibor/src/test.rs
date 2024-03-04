//! Tests against the OmniBOR crate as a whole.

use crate::ArtifactId;
use crate::Sha256;
use crate::SupportedHash;
use digest::OutputSizeUser;
use gitoid::HashAlgorithm;
use std::mem::size_of;

/// Get the underlying 'Digest'-implementing type for the Sha256 algorithm.
type Sha256Alg = <<Sha256 as SupportedHash>::HashAlgorithm as HashAlgorithm>::Alg;

/// ArtifactID should be exactly 32 bytes, the size of the buffer.
#[test]
fn artifact_id_sha256_size() {
    assert_eq!(size_of::<ArtifactId<Sha256>>(), Sha256Alg::output_size());
}
