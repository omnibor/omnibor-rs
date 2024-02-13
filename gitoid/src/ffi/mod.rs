//! `gitoid` Foreign Function Interface (FFI)

pub(crate) mod error;
mod gitoid;
pub(crate) mod status;
pub(crate) mod util;

// Re-export
pub use crate::ffi::gitoid::*;
