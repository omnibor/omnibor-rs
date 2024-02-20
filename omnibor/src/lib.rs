//! OmniBOR Artifact Identifiers and Artifact Input Manifests in Rust.
//!
//! ## What is OmniBOR?
//!
//! [OmniBOR][omnibor] is a draft specification which defines two key concepts:
//!
//! - __Artifact Identifiers__: independently-reproducible identifiers for
//!   software artifacts.
//! - __Artifact Input Manifests__: record the IDs of every input used in the
//!   build process for an artifact.
//!
//! Artifact IDs enable _anyone_ to identify and cross-reference information for
//! software artifacts without a central authority. Unlike [pURL][purl] or [CPE][cpe],
//! OmniBOR Artifact IDs don't rely on a third-party, they are _inherent
//! identifiers_ determined only by an artifact itself. They're based on
//! [Git's Object IDs (GitOIDs)][gitoid] in both construction and choice of
//! cryptographic hash functions.
//!
//! Artifact Input Manifests allow consumers to reconstruct Artifact Dependency
//! Graphs that give _fine-grained_ visibility into how artifacts in your
//! software supply chain were made. With these graphs, consumers could
//! in the future identify the presence of exact files associated with known
//! vulnerabilities, side-stepping the complexities of matching version numbers
//! across platforms and patching practicies.
//!
//! [__You can view the OmniBOR specification here.__][omnibor_spec]
//!
//! The United States Cybersecurity & Infrastructure Security Agency (CISA),
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

pub(crate) mod sealed;

mod artifact_id;
mod error;
mod supported_hash;

pub(crate) use crate::error::Result;

pub use crate::artifact_id::ArtifactId;
pub use crate::error::Error;
pub use crate::supported_hash::Sha256;
pub use crate::supported_hash::SupportedHash;
