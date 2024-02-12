//! A content-addressable identity for software artifacts.
//!
//! GitOIDs are a mechanism for identifying artifacts in a manner which is
//! independently reproducible because it relies solely on the contents of
//! the artifact itself.
//!
//! The GitOID scheme comes from the Git version control system, which uses
//! this mechanism to identify commits, tags, files (called "blobs"), and
//! directories (called "trees"). It's also used by the GitBOM standard for
//! identifying inputs which produce software artifacts.
//!
//! `gitoid` is also an IANA-registered URL scheme, meaning that GitOIDs
//! are represented and shared as URLs. A `gitoid` URL looks like:
//!
//! ```text
//! gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03
//! ```
//!
//! This scheme starts with "`gitoid`", followed by the object type ("`blob`"
//! in this case), the hash algorithm ("`sha256`"), and the hash of the
//! artifact the GitOID represents. Each of these parts is separated
//! by a colon.
//!
//! The valid object types for a GitOID are:
//!
//! - `blob`
//! - `tree`
//! - `commit`
//! - `tag`
//!
//! The valid hash algorithms are:
//!
//! - `sha1`
//! - `sha256`
//!
//! GitOIDs provide a convenient mechanism to establish artifact identity and
//! validate artifact integrity (this artifact hasn't been modified) and
//! agreement (I have the same artifact you have).

mod error;
#[doc(hidden)]
pub mod ffi;
mod gitoid;
mod hash_algorithm;
mod object_type;
pub(crate) mod sealed;
#[cfg(test)]
mod tests;

pub use crate::error::Error;
pub(crate) use crate::error::Result;
pub use crate::gitoid::GitOid;
pub use crate::hash_algorithm::HashAlgorithm;
pub use crate::object_type::ObjectType;

pub mod object {
    //! Object types supported for [`GitOid`] construction.

    #[cfg(doc)]
    use crate::GitOid;

    pub use crate::object_type::{Blob, Commit, Tag, Tree};
}

pub mod hash {
    //! Hash algorithms supported for [`GitOid`] construction.

    #[cfg(doc)]
    use crate::GitOid;

    pub use sha1::Sha1;
    pub use sha2::Sha256;

    /// Sha-1CD hasher.
    pub type Sha1Cd = sha1collisiondetection::Sha1CD;
}
