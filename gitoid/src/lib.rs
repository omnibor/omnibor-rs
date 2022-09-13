//! A content-addressable identity for a software artifact.

mod builder;
mod error;
mod gitoid;
mod hash_algorithm;
mod object_type;
#[cfg(test)]
mod tests;

pub use crate::builder::*;
pub use crate::error::*;
pub use crate::gitoid::*;
pub use crate::hash_algorithm::*;
pub use crate::object_type::*;
