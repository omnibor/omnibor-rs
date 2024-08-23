//! Tests against the OmniBOR crate as a whole.

use crate::hashes::Sha256;
use crate::hashes::SupportedHash;
use crate::ArtifactId;
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

#[cfg(feature = "serde")]
mod serde_test {
    use crate::hashes::Sha256;
    use crate::ArtifactId;
    use serde_test::assert_tokens;
    use serde_test::Token;

    #[test]
    fn valid_artifact_id_ser_de() {
        let id = ArtifactId::<Sha256>::id_str("hello, world");

        // This validates both serialization and deserialization.
        assert_tokens(&id, &[Token::Str("gitoid:blob:sha256:7d0be525d6521168c74051e5ab1b99e3b6d1c962fba763818f1954ab9e1c821a")]);
    }
}
