//! OmniBOR in Rust.

pub(crate) mod sealed;

mod artifact_id;
mod error;
mod supported_hash;

pub(crate) use crate::error::Result;

pub use crate::artifact_id::ArtifactId;
pub use crate::error::Error;
pub use crate::supported_hash::Sha256;
pub use crate::supported_hash::SupportedHash;
