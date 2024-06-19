//! A content-addressable identity for software artifacts.
//!
//! ## What are GitOIDs?
//!
//! Git Object Identifiers ([GitOIDs][gitoid]) are a mechanism for
//! identifying artifacts in a manner which is independently reproducible
//! because it relies only on the contents of the artifact itself.
//!
//! The GitOID scheme comes from the Git version control system, which uses
//! this mechanism to identify commits, tags, files (called "blobs"), and
//! directories (called "trees").
//!
//! This implementation of GitOIDs is produced by the [OmniBOR][omnibor]
//! working group, which uses GitOIDs as the basis for OmniBOR Artifact
//! Identifiers.
//!
//! ### GitOID URL Scheme
//!
//! `gitoid` is also an IANA-registered URL scheme, meaning that GitOIDs
//! are represented and shared as URLs. A `gitoid` URL looks like:
//!
//! ```text
//! gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03
//! ```
//!
//! This scheme starts with "`gitoid`", followed by the object type
//! ("`blob`" in this case), the hash algorithm ("`sha256`"), and the
//! hash produced by the GitOID hash construction. Each of these parts is
//! separated by a colon.
//!
//! ### GitOID Hash Construction
//!
//! GitOID hashes are made by hashing a prefix string containing the object
//! type and the size of the object being hashed in bytes, followed by a null
//! terminator, and then hashing the object itself. So GitOID hashes do not
//! match the result of only hashing the object.
//!
//! ### GitOID Object Types
//!
//! The valid object types for a GitOID are:
//!
//! - `blob`
//! - `tree`
//! - `commit`
//! - `tag`
//!
//! Currently, this crate implements convenient handling of `blob` objects,
//! but does not handle ensuring the proper formatting of `tree`, `commit`,
//! or `tag` objects to match the Git implementation.
//!
//! ### GitOID Hash Algorithms
//!
//! The valid hash algorithms are:
//!
//! - `sha1`
//! - `sha1dc`
//! - `sha256`
//!
//! `sha1dc` is actually Git's default algorithm, and is equivalent to `sha1`
//! in _most_ cases. Where it differs is when the hasher detects what it
//! believes to be an attempt to generate a purposeful SHA-1 collision,
//! in which case it modifies the hash process to produce a different output
//! and avoid the malicious collision.
//!
//! Git does this under the hood, but does not clearly distinguish to end
//! users that the underlying hashing algorithm isn't equivalent to SHA-1.
//! This is fine for Git, where the specific hash used is an implementation
//! detail and only matters within a single repository, but for the OmniBOR
//! working group it's important to distinguish whether plain SHA-1 or
//! SHA-1DC is being used, so it's distinguished in the code for this crate.
//!
//! This means for compatibility with Git that SHA-1DC should be used.
//!
//! ## Why Care About GitOIDs?
//!
//! GitOIDs provide a convenient mechanism to establish artifact identity and
//! validate artifact integrity (this artifact hasn't been modified) and
//! agreement (I have the same artifact you have). The fact that they're based
//! only on the type of object ("`blob`", usually) and the artifact itself
//! means they can be derived independently, enabling distributed artifact
//! identification that avoids a central decider.
//!
//! Alternative identity schemes, like Package URLs (purls) or Common Platform
//! Enumerations (CPEs) rely on central authorities to produce identifiers or
//! define the taxonomy in which identifiers are produced.
//!
//! ## Using this Crate
//!
//! The central type of this crate is [`GitOid`], which is generic over both
//! the hash algorithm used and the object type being identified. These are
//! defined by the [`HashAlgorithm`] and [`ObjectType`] traits.
//!
//! ## Example
//!
//! ```text
//! # use gitoid::{Sha256, Blob};
//! type GitOid = gitoid::GitOid<Sha256, Blob>;
//!
//! let gitoid = GitOid::from_str("hello, world");
//! println!("gitoid: {}", gitoid);
//! ```
//!
//! [gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
//! [omnibor]: https://omnibor.io

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(
    feature = "sha1",
    feature = "sha1cd",
    feature = "sha256",
    feature = "rustcrypto"
)))]
compile_error!(
    r#"At least one hash algorithm feature must be active: "sha1", "sha1cd", or "sha256""#
);

#[cfg(all(feature = "sha1cd", feature = "boringssl", not(feature = "rustcrypto")))]
compile_error!(r#"The "boringssl" feature does not support the "sha1cd" algorithm"#);

#[cfg(all(
    feature = "rustcrypto",
    not(any(feature = "sha1", feature = "sha1cd", feature = "sha256"))
))]
compile_error!(
    r#"The "rustcrypto" feature requires at least one of the following algorithms: "sha1", "sha1cd", or "sha256""#
);

#[cfg(not(any(feature = "rustcrypto", feature = "boringssl")))]
compile_error!(r#"At least one of the "rustcrypto" or "boringssl" features must be enabled"#);

mod backend;
mod error;
mod gitoid;
mod hash_algorithm;
mod object_type;
pub(crate) mod sealed;
#[cfg(test)]
mod tests;

#[cfg(feature = "boringssl")]
pub use crate::backend::boringssl;
#[cfg(feature = "rustcrypto")]
pub use crate::backend::rustcrypto;
pub use crate::error::Error;
pub(crate) use crate::error::Result;
pub use crate::gitoid::GitOid;
pub use crate::hash_algorithm::HashAlgorithm;
pub use crate::object_type::Blob;
pub use crate::object_type::Commit;
pub use crate::object_type::ObjectType;
pub use crate::object_type::Tag;
pub use crate::object_type::Tree;
