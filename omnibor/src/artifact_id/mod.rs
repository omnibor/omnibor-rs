//! Reproducible artifact identifier.

pub(crate) mod artifact_id;
pub(crate) mod artifact_id_builder;

pub use crate::artifact_id::artifact_id::ArtifactId;
pub use crate::artifact_id::artifact_id_builder::ArtifactIdBuilder;
