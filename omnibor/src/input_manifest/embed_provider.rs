use crate::{hash_algorithm::HashAlgorithm, ArtifactId};

#[derive(Debug, Default)]
pub struct EmbedProvider {
    _private: (),
}

impl EmbedProvider {
    pub fn new() -> Self {
        EmbedProvider::default()
    }

    pub fn get_str_embed<H>(&self, _manifest_aid: ArtifactId<H>) -> String
    where
        H: HashAlgorithm,
    {
        todo!()
    }

    pub fn get_le_bytes_embed<H>(&self, _manifest_aid: ArtifactId<H>) -> Vec<u8>
    where
        H: HashAlgorithm,
    {
        todo!()
    }

    pub fn get_be_bytes_embed<H>(&self, _manifest_aid: ArtifactId<H>) -> Vec<u8>
    where
        H: HashAlgorithm,
    {
        todo!()
    }

    pub fn get_ne_bytes_embed<H>(&self, _manifest_aid: ArtifactId<H>) -> Vec<u8>
    where
        H: HashAlgorithm,
    {
        todo!()
    }
}
