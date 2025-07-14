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

    /// Get the manifest Artifact ID as little-endian bytes.
    pub fn get_le_bytes_embed(&self) -> Vec<u8> {
        todo!()
    }

    /// Get the manifest Artifact ID as big-endian bytes.
    pub fn get_be_bytes_embed(&self) -> Vec<u8> {
        todo!()
    }

    /// Get the manifest Artifact ID as native-endian bytes.
    pub fn get_ne_bytes_embed(&self) -> Vec<u8> {
        todo!()
    }
}
