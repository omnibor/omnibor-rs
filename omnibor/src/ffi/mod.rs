//! `omnibor` Foreign Function Interface (FFI)

mod artifact_id;
pub(crate) mod error;
pub(crate) mod status;
pub(crate) mod util;

// Re-export
pub use crate::ffi::artifact_id::*;
