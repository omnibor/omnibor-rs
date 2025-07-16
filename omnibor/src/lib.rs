//! This crate implements the [OmniBOR][omnibor] specification, so you can
//! __identify artifacts__ like binaries and source code files and __track
//! build dependencies__ used to create them.
//!
//! You can identify a file like so:
//!
//! ```
//! # use omnibor::{ArtifactId, error::ArtifactIdError, hash_provider::RustCrypto};
//! # use std::str::FromStr;
//! let artifact_id = ArtifactId::new(RustCrypto::new(), "./test/data/hello_world.txt")?;
//! // gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03
//! # assert_eq!(artifact_id, ArtifactId::from_str("gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03").unwrap());
//! # Ok::<(), ArtifactIdError>(())
//! ```
//!
//! ... and track build dependencies like so:
//!
//! ```
//! # use omnibor::{
//! #     ArtifactId,
//! #     InputManifestBuilder,
//! #     storage::InMemoryStorage,
//! #     hash_provider::RustCrypto,
//! #     embed::NoEmbed,
//! #     error::InputManifestError,
//! # };
//! # use std::str::FromStr;
//! let input_manifest = InputManifestBuilder::new(InMemoryStorage::new(RustCrypto::new()))
//!     .add_relation("./test/data/c/main.c")?
//!     .add_relation("./test/data/c/util.c")?
//!     .build_for_target("./test/data/c/my_executable", NoEmbed)?;
//! // gitoid:blob:sha256
//! // 6e87984feca6b64467f9028fd6b76a4ec8d13ee23f0ae3b99b548ca0c2d0230b
//! // 726eb0db4f3130fb4caef53ee9d103b6bc2d732e665dd88f53efce4c7b59818b
//! # assert!(input_manifest.contains_artifact(ArtifactId::from_str("gitoid:blob:sha256:6e87984feca6b64467f9028fd6b76a4ec8d13ee23f0ae3b99b548ca0c2d0230b").unwrap()));
//! # assert!(input_manifest.contains_artifact(ArtifactId::from_str("gitoid:blob:sha256:726eb0db4f3130fb4caef53ee9d103b6bc2d732e665dd88f53efce4c7b59818b").unwrap()));
//! # Ok::<_, InputManifestError>(())
//! ```
//!
//! [`ArtifactId`]s are _universally reproducible_, so anyone with the file
//! will produce the same ID. They're [GitOIDs (Git Object Identifiers)][gitoid]
//! with "blob" as the type, SHA-256 as the hash algorithm, and all newlines
//! normalized to Unix style.
//!
//! [`InputManifest`]s are very small text files that record Artifact IDs for
//! all build inputs. If an input has its own Input Manifest, its Artifact ID
//! gets recorded too. When an Input Manifest is created, you can embed its
//! Artifact ID in the "target artifact" whose build input it's describing.
//!
//! With Input Manifests, you get a [Merkle tree]-like record of build inputs,
//! describing the full Artifact Dependency Graph (ADG)—every input used to
//! build the final artifact. Any time a build input changes, all the Artifact
//! IDs of anything derived from it change too! This enables easy detection of
//! build differences, arbitrarily deep in an ADG.
//!
//! There are three ways to use the Rust OmniBOR implementation:
//!
//! - The `omnibor` crate (**you are here**)
//! - The [`omnibor` crate Foreign Function Interface (FFI)][ffi]
//! - The [OmniBOR CLI][cli]
//!
//! # Concepts
//!
//! In addition to [`ArtifactId`], [`InputManifest`], and
//! [`InputManifestBuilder`], there are some key traits and types to know:
//!
//! - __[Identify and IdentifyAsync](#identify-and-identifyasync)__: Traits for
//!   types that can produce an [`ArtifactId`], synchronously or asynchronously.
//! - __[Hash Algorithms](#hash-algorithms)__: Types implementing
//!   [`HashAlgorithm`](crate::hash_algorithm::HashAlgorithm), currently only
//!   [`Sha256`](crate::hash_algorithm::Sha256).
//! - __[Hash Providers](#hash-providers)__: Types implementing
//!   [`HashProvider`](crate::hash_provider::HashProvider), currently
//!   [`RustCrypto`](crate::hash_provider::RustCrypto),
//!   [`OpenSsl`](crate::hash_provider::OpenSsl), and
//!   [`BoringSsl`](crate::hash_provider::BoringSsl).
//! - __[Storage](#storage)__: Types implementing
//!   [`Storage`](crate::storage::Storage), currently
//!   [`FileSystemStorage`](crate::storage::FileSystemStorage) or
//!   [`InMemoryStorage`](crate::storage::InMemoryStorage).
//! - __[Embedding](#embedding)__: Types implementing
//!   [`Embed`](crate::embed::Embed), currently
//!   [`NoEmbed`](crate::embed::NoEmbed),
//!   [`AutoEmbed`](crate::embed::AutoEmbed),
//!   and [`CustomEmbed`](crate::embed::CustomEmbed).
//!
//! ## Identify and IdentifyAsync
//!
//! There are two traits for types that can produce an [`ArtifactId`]:
//!
//! - [`Identify`]: Can produce an Artifact ID synchronously.
//! - [`IdentifyAsync`]: Can produce an Artifact ID asynchronously.
//!
//! These traits are used as bounds on functions throughout the crate that
//! expect an [`ArtifactId`].
//!
//! The full list of "identifiable" types, with an explanation of how they're
//! processed, is as follows:
//!
//! | Type                                                             | `impl Identify` | `impl IdentifyAsync` | Explanation                                                  |
//! |:-----------------------------------------------------------------|:----------------|:---------------------|:-------------------------------------------------------------|
//! | `&[u8]`                                                          | ✅              |                      | Hash the bytes.                                              |
//! | `[u8; N]`                                                        | ✅              |                      | Hash the bytes.                                              |
//! | `&[u8; N]`                                                       | ✅              |                      | Hash the bytes.                                              |
//! | `&str`                                                           | ✅              | ✅                   | Treat as a path, hash the contents of the file at that path. |
//! | `&String`                                                        | ✅              | ✅                   | Treat as a path, hash the contents of the file at that path. |
//! | `&OsStr`                                                         | ✅              | ✅                   | Treat as a path, hash the contents of the file at that path. |
//! | `&OsString`                                                      | ✅              | ✅                   | Treat as a path, hash the contents of the file at that path. |
//! | `&Path`                                                          | ✅              | ✅                   | Hash the contents of the file at that path.                  |
//! | `&PathBuf`                                                       | ✅              | ✅                   | Hash the contents of the file at that path.                  |
//! | `File`                                                           | ✅              |                      | Hash the contents of the file.                               |
//! | `&File`                                                          | ✅              |                      | Hash the contents of the file.                               |
//! | `Box<File>`                                                      | ✅              |                      | Hash the contents of the file.                               |
//! | `Rc<File>`                                                       | ✅              |                      | Hash the contents of the file.                               |
//! | `Arc<File>`                                                      | ✅              |                      | Hash the contents of the file.                               |
//! | `&mut tokio::fs::File`                                           |                 | ✅                   | Hash the contents of the file.                               |
//! | `tokio::fs::File`                                                |                 | ✅                   | Hash the contents of the file.                               |
//! | `BufReader<R> where R: Read + Seek`                              | ✅              |                      | Hash the bytes read off the reader.                          |
//! | `tokio::io::BufReader<R> where R: AsyncRead + AsyncSync + Unpin` |                 | ✅                   | Hash the bytes read off the reader.                          |
//! | `&mut R where R: Read + Seek`                                    | ✅              |                      | Hash the bytes read off the reader.                          |
//! | `Cursor<T> where T: AsRef<[u8]>`                                 | ✅              |                      | Hash the bytes read off the cursor.                          |
//! | `InputManifest<H>`                                               | ✅              |                      | Hash the on-disk representation of the manifest.             |
//! | `&InputManifest<H>`                                              | ✅              |                      | Hash the on-disk representation of the manifest.             |
//! | `ArtifactId<H>`                                                  | ✅              |                      | Copy the Artifact ID.                                        |
//! | `&ArtifactId<H>`                                                 | ✅              |                      | Copy the Artifact ID.                                        |
//!
//! ## Hash Algorithms
//!
//! Artifact IDs and Input Manifests are based on a hash algorithm.
//! Today, OmniBOR only supports SHA-256, though alternatives may be added in
//! the future if SHA-256's cryptographic properties are broken.
//!
//! The [`HashAlgorithm`](crate::hash_algorithm::HashAlgorithm) trait is
//! implemented by the [`Sha256`](crate::hash_algorithm::Sha256) type, and most
//! types in this crate are parameterized over the hash algorithm.
//!
//! ## Hash Providers
//!
//! Hashes are produced by "hash providers," which implement the
//! [`HashProvider`](crate::hash_provider::HashProvider) trait. Today we
//! support [RustCrypto], [OpenSSL], and [BoringSSL], which you can turn on or
//! off at compile time using features:
//!
//! | Feature               | Provider   | Type                                             |
//! |:----------------------|:-----------|:-------------------------------------------------|
//! | `provider-rustcrypto` | RustCrypto | [`RustCrypto`](crate::hash_provider::RustCrypto) |
//! | `provider-openssl`    | OpenSSL    | [`OpenSsl`](crate::hash_provider::OpenSsl)       |
//! | `provider-boringssl`  | BoringSSL  | [`BoringSsl`](crate::hash_provider::BoringSsl)   |
//!
//! ## Storage
//!
//! We expose a [`Storage`](crate::storage::Storage) trait, representing the
//! abstract interface needed for interacting with Input Manifests in memory or
//! on-disk. There are two types,
//! [`FileSystemStorage`](crate::storage::FileSystemStorage) and
//! [`InMemoryStorage`](crate::storage::InMemoryStorage), that implement it.
//!
//! If you want to persist Input Manifests in any way, we recommend using
//! [`FileSystemStorage`](crate::storage::FileSystemStorage), as it correctly
//! complies with the OmniBOR specification's requirements for where manifests
//! should be stored.
//!
//! If you do not need to persist Input Manifests, use
//! [`InMemoryStorage`](crate::storage::InMemoryStorage).
//!
//! ## Embedding
//!
//! When [`InputManifest`]s are built with [`InputManifestBuilder`], you pass
//! in an embedding choice to select whether and how to embed the Artifact ID
//! of the Input Manifest into the target artifact the manifest is describing.
//!
//! If you _do_ embed the manifest's Artifact ID, you ensure changes to the
//! build inputs are reflected in the Artifact ID of the target artifact.
//!
//! There are currently three options, all implementing the
//! [`Embed`](crate::embed::Embed) trait:
//!
//! - [`NoEmbed`](crate::embed::NoEmbed): Do not do embedding. The resulting
//!   manifest is considered "detached."
//! - [`AutoEmbed`](crate::embed::AutoEmbed): Do embedding, with the `omnibor`
//!   crate inferring the filetype of the target artifact to determine how to
//!   embed in it. Requires the `infer-filetypes` feature is on.
//! - [`CustomEmbed`](crate::embed::CustomEmbed): Do embedding, providing your
//!   own embedding function which takes the path to the target artifact, and
//!   an [`EmbedProvider`](crate::embed::EmbedProvider) which gives a hex string
//!   or bytes to embed, as appropriate.
//!
//! # Meta
//!
//! The following sections contain information about the status of this
//! implementation, its conformance to the specification, and its relationship
//! to other software identification schemes.
//!
//! ## Specification Compliance
//!
//! OmniBOR is a [draft specification][omnibor_spec], and this implementation
//! is the primary implementation of it.
//!
//! Currently, this implementation follows the draft 0.2 version of the
//! specification, which includes two major differences relative to version 0.1:
//!
//! - Limitation of supported hashes to SHA-256 only, and
//! - Universal newline normalization.
//!
//! All differences from the specification and this library are in the process
//! of being resolved through specification updates.
//!
//! ## Comparison with other Software Identifiers
//!
//! OmniBOR Artifact IDs are just one scheme for identifying software. Others
//! include the [Common Platform Enumeration (CPE)][cpe],
//! [Package URLs (pURLs)][purl], [Software Hash IDs (SWHIDs)][swhid],
//! [Software ID tags (SWID tags)][swid], and [Nix Derivation Store Paths][nix].
//!
//! Each of these has their own strengths and weaknesses, and the creation of
//! each was motivated by a different purpose. In many cases, these schemes can
//! be complementary to each other. The following table tries to break down
//! some major points of comparison between them:
//!
//! | Scheme  | Derivation | Architecture | Defined By           | Based On                          |
//! |:--------|:-----------|:-------------|:---------------------|:----------------------------------|
//! | CPE     | Defined    | Centralized  | NIST                 | -                                 |
//! | pURL    | Defined    | Federated    | ECMA + Package Hosts | -                                 |
//! | SWID    | Defined    | Distributed  | Software Producer    | -                                 |
//! | SWHID   | Inherent   | Distributed  | -                    | Artifact content                  |
//! | Nix     | Inherent   | Distributed  | -                    | Package build inputs              |
//! | OmniBOR | Inherent   | Distributed  | -                    | Artifact content and build inputs |
//!
//! Let's explain this a bit:
//!
//! 1. __Derivation__: Whether an identifier comes from an authority who defines
//!    it, or can be derived from the thing being identified inherently.
//! 2. __Architecture__: What level of authority delegation exists for producing
//!    the identifier. For example, CPE relies on a central dictionary only
//!    NIST can edit, so it is "centralized," while pURL has a central list of
//!    "types" listing package hosts, but the names of packages on those hosts
//!    are controlled separately, so it's "federated". All inherent schemes are
//!    considered "distributed".
//! 3. __Defined By__: Who has authority to produce the identifiers. CPE's
//!    dictionary is controlled by NIST. pURLs list of types is standardized
//!    under ECMA's [TC54 (a Technical Committee on "Software and System
//!    Transparency")](https://tc54.org/) but each package host controls its
//!    own namespace of names. SWID tags are provided by the producer of the
//!    software. All inherent identifiers do not require a "definer".
//! 4. __Based On__: What materials are used to produce the identifier. This is
//!    not relevant for defined identifiers. For inherent identifiers, SWHID
//!    uses a hash of a file (it also has variant types of identifiers
//!    for things like directories, but all are content-based); Nix derives its
//!    Derivation Store Path from the inputs to a package's build; OmniBOR
//!    derives Artifact IDs from an artifact's contents, which may embed a
//!    reference to the Artifact ID of the file's input manifest and thus also
//!    depend on the identities of its build dependencies.
//!
//! In 2023, CISA, the Cybersecurity and Infrastructure Security Agency,
//! published a report titled ["Software Identification Ecosystem Option
//! Analysis"][cisa_report] that surveyed the state of the software
//! identification ecosystem and made recommendations for which schemes to
//! prefer, when to consider them, and the challenges facing each of them.
//! OmniBOR was one of three schemes recommended by this report, alongside
//! CPE and pURL.
//!
//! We recommend using OmniBOR as a complement to defined identifiers like
//! CPE or pURL, with CPE or pURL identifying the relevant product or package
//! and OmniBOR identifying specific software artifacts.
//!
//! We recommend using OmniBOR instead of other inherent identifiers, unless
//! you are in an ecosystem which already uses an alternative, for the
//! following reasons:
//!
//! - __Inclusion of Length__: OmniBOR uses the "Git Object Identifier"
//!   scheme used by the Git Version Control System (VCS), which includes the
//!   length of the artifact as an input to the hash. This helps protect
//!   against attempts to engineer hash collisions by requiring attackers to
//!   manage the influence of changing the length of an artifact.
//! - __Use of SHA-256__: OmniBOR only supports SHA-256 today, while Software
//!   Hash IDs use SHA-1 and Nix Derivation Store Paths support MD5, SHA-1,
//!   SHA-256, or SHA-512. Nix also truncates its hashes, which OmniBOR does
//!   not do.
//! - __Inclusion of _both_ artifact contents and build inputs__: OmniBOR
//!   Artifact IDs are based on an artifact's contents, and if the artifact
//!   has embedded the Artifact ID of its Input Manifest, that Input Manifest
//!   (and by extension all build inputs) influences the resulting Artifact ID.
//!   This makes an Artifact ID the strongest commitment out of the inherent
//!   identifiers. SWHIDs only incorporate the artifact itself; Nix Derivation
//!   Store Paths are based only on build inputs (the Nix system tries to
//!   enforce reproducibility in practice, though reproducibility from the
//!   same inputs is not guaranteed, and will not be detectable by Derivation
//!   Store Path alone).
//!
//! ## Contribute
//!
//! The following items are things we'd want to complete before committing to
//! stability in this crate with a 1.0.0 release:
//!
//! - [ ] Embedding Support
//!   - [ ] Support for auto-embedding in ELF files
//!   - [ ] Support for auto-embedding in Mach-O binary files
//!   - [ ] Support for auto-embedding in Portable Executable files
//! - [ ] ADG Support
//!   - [ ] Support creating an Artifact Dependency Graph
//! - [ ] FFI Support
//!   - [ ] Exposing the `InputManifestBuilder` API over FFI
//!   - [ ] Exposing the `InputManifest` API over FFI
//!   - [ ] Exposing the `FileSystemStorage` API over FFI
//!   - [ ] Exposing the `InMemoryStorage` API over FFI
//! - [ ] Hash Provider Flexibility
//!   - [ ] Support linking with system-provided OpenSSL
//!   - [ ] Support linking with system-provided BoringSSL
//! - [ ] Documentation
//!   - [ ] Documenting the cancellation safety of all async APIs
//!   - [ ] Adding examples to all public methods.
//!
//! If helping build out any of this sounds appealing to you, we love getting
//! contributions!
//!
//! Check out our [Issue Tracker] and [Contributor Guide].
//!
//! [Merkle Tree]: https://en.wikipedia.org/wiki/Merkle_tree
//! [cisa_report]: https://www.cisa.gov/sites/default/files/2023-10/Software-Identification-Ecosystem-Option-Analysis-508c.pdf
//! [cpe]: https://nvd.nist.gov/products/cpe
//! [gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
//! [gitoid_crate]: https://crates.io/crates/gitoid
//! [omnibor]: https://omnibor.io
//! [omnibor_crate]: https://crates.io/crates/omnibor
//! [omnibor_spec]: https://github.com/omnibor/spec
//! [purl]: https://github.com/package-url/purl-spec
//! [ffi]: crate::ffi
//! [cli]: https://github.com/omnibor/omnibor-rs/releases?q=omnibor-cli
//! [swid]: https://csrc.nist.gov/projects/software-identification-swid
//! [nix]: https://www.tweag.io/blog/2024-03-12-nix-as-software-identifier/
//! [swhid]: https://www.swhid.org/
//! [Issue Tracker]: https://github.com/omnibor/omnibor-rs/issues
//! [Contributor Guide]: https://github.com/omnibor/omnibor-rs/blob/main/CONTRIBUTING.md
//! [RustCrypto]: https://github.com/rustcrypto
//! [OpenSSL]: https://www.openssl.org/
//! [BoringSSL]: https://boringssl.googlesource.com/boringssl

/*===============================================================================================
 * Lint Configuration
 */

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/*===============================================================================================
 * Compilation Protections
 */

#[cfg(not(any(
    feature = "provider-rustcrypto",
    feature = "provider-boringssl",
    feature = "provider-openssl"
)))]
compile_error!(
    r#"At least one of the "provider-rustcrypto", "provider-boringssl", \n"#
    r#"\tor "provider-openssl" features must be enabled"#
);

/*===============================================================================================
 * Internal Modules
 */

pub(crate) mod artifact_id;
pub(crate) mod gitoid;
pub(crate) mod input_manifest;
pub(crate) mod object_type;
pub(crate) mod util;

/*===============================================================================================
 * Testing
 */

#[cfg(feature = "provider-rustcrypto")]
#[cfg(test)]
mod test;

/*===============================================================================================
 * FFI
 */

// Hidden since we don't want to commit to stability.
pub mod ffi;

/*===============================================================================================
 * Public API
 */

pub mod embed;
pub mod error;
pub mod hash_algorithm;
pub mod hash_provider;
pub mod storage;

pub use crate::artifact_id::ArtifactId;
pub use crate::artifact_id::Identify;
pub use crate::artifact_id::IdentifyAsync;
pub use crate::input_manifest::Input;
pub use crate::input_manifest::InputManifest;
pub use crate::input_manifest::InputManifestBuilder;
