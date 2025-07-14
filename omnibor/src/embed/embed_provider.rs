use crate::{hash_algorithm::HashAlgorithm, ArtifactId};

#[derive(Debug)]
pub struct EmbedProvider<H>
where
    H: HashAlgorithm,
{
    manifest_aid: ArtifactId<H>,
}

impl<H> EmbedProvider<H>
where
    H: HashAlgorithm,
{
    /// Construct a new embed provider.
    pub fn new(manifest_aid: ArtifactId<H>) -> Self {
        EmbedProvider { manifest_aid }
    }

    /// Get the manifest Artifact ID as a hexadecimal string.
    pub fn get_str_embed(&self) -> String {
        self.manifest_aid.as_hex()
    }

    /// Get the manifest Artifact ID as bytes.
    pub fn get_bytes_embed(&self) -> &[u8] {
        self.manifest_aid.as_bytes()
    }
}
