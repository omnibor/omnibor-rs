//! OmniBOR Artifact Identifiers and Artifact Input Manifests in Rust.
//!
//! ## What is OmniBOR?
//!
//! [OmniBOR][omnibor] is a draft specification which defines two key concepts:
//!
//! - __Artifact Identifiers__: independently-reproducible identifiers for
//!   software artifacts. Use [`ArtifactId`](crate::artifact_id::ArtifactId)
//!   for these.
//! - __Artifact Input Manifests__: record the IDs of every input used in the
//!   build process for an artifact. Use
//!   [`InputManifest`](crate::input_manifest::InputManifest) for these.
//!
//! Artifact IDs enable _anyone_ to identify and cross-reference information for
//! software artifacts without a central authority. Unlike [pURL][purl] or [CPE][cpe],
//! OmniBOR Artifact IDs don't rely on a third-party, they are _inherent
//! identifiers_ determined only by an artifact itself. They're based on
//! [Git Object Identifiers (GitOIDs)][gitoid] in both construction and choice of
//! cryptographic hash functions.
//!
//! Artifact Input Manifests allow consumers to reconstruct Artifact Dependency
//! Graphs that give _fine-grained_ visibility into how artifacts in their
//! software supply chain were made. With these graphs, consumers could
//! in the future identify the presence of exact files associated with known
//! vulnerabilities, side-stepping the complexities of matching version numbers
//! across platforms and patching practices.
//!
//! [__You can view the OmniBOR specification here.__][omnibor_spec]
//!
//! The United States Cybersecurity & Infrastructure Security Agency (CISA)
//! identified OmniBOR as a major candidate for software identities
//! in its 2023 report ["Software Identification Ecosystem Option
//! Analysis."][cisa_report]
//!
//! [contributing]: CONTRIBUTING.md
//! [cbindgen]: https://github.com/eqrion/cbindgen
//! [cisa_report]: https://www.cisa.gov/sites/default/files/2023-10/Software-Identification-Ecosystem-Option-Analysis-508c.pdf
//! [cpe]: https://nvd.nist.gov/products/cpe
//! [gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
//! [gitoid_crate]: https://crates.io/crates/gitoid
//! [omnibor]: https://omnibor.io
//! [omnibor_crate]: https://crates.io/crates/omnibor
//! [omnibor_spec]: https://github.com/omnibor/spec
//! [purl]: https://github.com/package-url/purl-spec

/*===============================================================================================
 * Lint Configuration
 */

#![allow(clippy::module_inception)]

/*===============================================================================================
 * Compilation Protections
 */

#[cfg(not(any(
    feature = "backend-rustcrypto",
    feature = "backend-boringssl",
    feature = "backend-openssl"
)))]
compile_error!(
    r#"At least one of the "backend-rustcrypto", "backend-boringssl", \n"#
    r#"\tor "backend-openssl" features must be enabled"#
);

/*===============================================================================================
 * Internal Modules
 */

pub(crate) mod gitoid;
pub(crate) mod object_type;
pub(crate) mod util;

/*===============================================================================================
 * Testing
 */

#[cfg(feature = "backend-rustcrypto")]
#[cfg(test)]
mod test;

/*===============================================================================================
 * FFI
 */

// Hidden since we don't want to commit to stability.
#[doc(hidden)]
pub mod ffi;

/*===============================================================================================
 * Public API
 */

pub mod artifact_id;
pub mod error;
pub mod hash_algorithm;
pub mod hash_provider;
pub mod input_manifest;
pub mod storage;
