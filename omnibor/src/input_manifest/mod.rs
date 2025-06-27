//! Record of software build inputs by Artifact ID.

#[cfg(feature = "infer-filetypes")]
pub(crate) mod embed;
pub(crate) mod embed_provider;
pub mod input_manifest;
pub mod input_manifest_builder;

pub use input_manifest::InputManifest;
pub use input_manifest::InputManifestRelation;
pub use input_manifest_builder::InputManifestBuilder;
