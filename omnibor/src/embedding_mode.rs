//! Control whether an [`InputManifest`](crate::InputManifest)'s [`ArtifactId`](crate::ArtifactId) is stored in an artifact.

/// Indicate whether to embed the identifier for an input manifest in an artifact.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EmbeddingMode {
    /// Embed the identifier for the input manifest.
    Embed,
    /// Do not embed the identifier for the input manifest.
    NoEmbed,
}
