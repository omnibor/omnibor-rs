//! A content-addressable identity for a software artifact.

mod gitoid;
mod hash_algorithm;
mod source;
#[cfg(test)]
mod tests;

pub use crate::gitoid::*;
pub use crate::hash_algorithm::*;
pub use crate::source::*;
