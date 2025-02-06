//! Record of software build inputs by Artifact ID.

pub mod input_manifest;
pub mod input_manifest_builder;

pub use input_manifest::InputManifest;
pub use input_manifest::InputManifestRelation;
pub use input_manifest_builder::InputManifestBuilder;
