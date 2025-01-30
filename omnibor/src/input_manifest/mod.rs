pub mod embedding_mode;
pub(crate) mod input_manifest;
pub(crate) mod input_manifest_builder;

pub use input_manifest::InputManifest;
pub use input_manifest::Relation;
pub use input_manifest_builder::InputManifestBuilder;
pub use input_manifest_builder::ShouldStore;
