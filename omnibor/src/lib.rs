//! [OmniBOR](https://omnibor.io) is a specification for a reproducible
//! software identifier we call an "Artifact ID" plus a compact record of
//! build inputs called an "Input Manifest". Together, they let _anyone_
//! precisely identify software binaries and text files, and track precise
//! inputs used to build them. They also form a [Merkle Tree], so _any_
//! change in a dependency causes all artifacts built from it to have a new,
//! different Artifact ID, and anyone can use the Input Manifests to detect
//! _exactly what dependency changed_.
//!
//! This crate exposes APIs for producing and consuming both Artifact IDs and
//! Input Manifests.
//!
//! If you just want documentation for the API of this crate, check out these
//! two types:
//!
//! - [`ArtifactId`]
//! - [`InputManifest`]
//!
//! We also [provide a CLI][cli], based on this crate, as another option for
//! working with Artifact IDs and Input Manifests.
//!
//! # Table of Contents
//!
//! 1. [Examples](#examples)
//! 2. [What is an Artifact ID?](#what-is-an-artifact-id)
//!    1. [The GitOID Construction](#the-gitoid-construction)
//!    2. [Choice of Hash Function](#choice-of-hash-function)
//!    3. [Unconditional Newline Normalization](#unconditional-newline-normalization)
//! 3. [Crate Overview](#crate-overview)
//!    1. [Creating Artifact IDs](#creating-artifact-ids)
//!    2. [Creating Input Manifests](#creating-input-manifests)
//!    3. [Hash Algorithms and Hash Providers](#hash-algorithms-and-hash-providers)
//!    4. [Storing Input Manifests](#storing-input-manifests)
//! 4. [Foreign Function Interface](#foreign-function-interface)
//! 5. [Specification Compliance](#specification-compliance)
//! 6. [Comparison with Other Software Identifiers](#comparison-with-other-software-identifiers)
//! 7. [What's Missing for 1.0.0](#whats-missing-for-100)
//!
//! # Examples
//!
//! The Artifact ID of one of the test files in this repo, `hello_world.txt`, is:
//!
//! ```ignore,custom
//! gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03
//! ```
//!
//! An input manifest for a `.o` file with three inputs in C might look like:
//!
//! ```ignore,custom
//! gitoid:blob:sha256
//! 09c825ac02df9150e4f93d12ba1da5d1ff5846c3e62503c814aa3a300c535772
//! 230f3515d1306690815bd9c3da0d15d8b6fcf43894d17100eb44b6d329a92f61
//! 2f4a51b16b76bbc87c4c27af8ae062b1b50b280f1ab78e3eec155334588dc88e
//! ```
//!
//! # What is an Artifact ID?
//!
//! An Artifact ID is a Git Object Identifier (GitOID), with only a type of
//! "blob," with SHA-256 as the hash function, and with unconditional newline
//! normalization.
//!
//! If that explanation makes sense, then congrats, that's all you need to know!
//!
//! Otherwise, to explain in more detail:
//!
//! ## The GitOID Construction
//!
//! The Git Version Control System identifies all objects checked into a
//! repository by calculating a Git Object Identifier. This identifier is based
//! around a hash function and what we'll call the "GitOID Construction" that
//! determines what gets input into the hash function.
//!
//! In the GitOID Construction, you first hash in a prefix string of the form:
//!
//! ```ignore,custom
//! <object_type> <size_of_input_in_bytes>\0
//! ```
//!
//! The `<object_type>` can be `blob`, `commit`, `tag`, or `tree`. The last
//! three are used for commits, tags, and directories respectively; `blob` is
//! used for files.
//!
//! The `<size_of_input_in_bytes>` is what it sounds like; Git calculates the
//! size of an input file and includes that in the hash.
//!
//! After hashing in the prefix string, Git then hashes the contents of the
//! file being identified. That's the GitOID Construction! Artifact IDs use
//! this same construction, with the `<object_type>` always set to `blob`.
//!
//! ## Choice of Hash Function
//!
//! We also restrict the hash function to _only_ SHA-256 today,
//! though the specification leaves open the possibility of transitioning to
//! an alternative in the future if SHA-256 is cryptographically broken.
//!
//! This is a difference from Git's default today. Git normally uses SHA-1,
//! and is in the slow process of transitioning to SHA-256. So why not use
//! SHA-1 to match Git's current default?
//!
//! First, it's worth saying that Git can use SHA-1 _or_ a variant of SHA-1
//! called "SHA-1CD" (sometimes spelled "SHA-1DC"). Back in 2017, researchers
//! from Google and CWI Amsterdam announced the "SHAttered" attack against
//! SHA-1, where they had successively engineered a collision (two different
//! documents which produced the same SHA-1 hash). The SHA-1CD algorithm was
//! developed in response. It's a variant of SHA-1 which attempts to detect
//! when the input is attempting to produce a collision like the one in the
//! SHAttered attack, and on detection modifies the hashing algorithm to
//! produce a different hash and stop that collision.
//!
//! Different versions of Git will use either SHA-1 or SHA-1CD by default. This
//! means that for Artifact IDs our choice of hash algorithm was between three
//! choices: SHA-1, SHA-1CD, or SHA-256.
//!
//! The split of SHA-1 and SHA-1CD doesn't matter for most Git users, since
//! a single repository will just use one or the other and most files will
//! not trigger the collision detection code path that causes their outputs to
//! diverge. For Artifact IDs though, it's a problem, since we care strongly
//! about our IDs being universally reproducible. Thus, the split creates a
//! challenge for our potential use of SHA-1.
//!
//! Additionally, it's worth noting that attacks against SHA-1 continue to
//! become more practical as computing hardware improves. In October 2024
//! NIST, the National Institute of Standards and Technology in the United
//! States, published an initial draft of a document "Transitioning the Use of
//! Cryptographic Algorithms and Key Lengths." While it is not yet an official
//! NIST recommendation, it does explicitly disallow the use of SHA-1 for
//! digital signature generation, considers its use for digital signature
//! verification to be a "legacy use" requiring special approval, and otherwise
//! prepares to sunset any use of SHA-1 by 2030.
//!
//! NIST is not a regulatory agency, but their recommendations _are_ generally
//! incorporated into policies both in government and in private industry, and
//! a NIST recommendation to fully transition away from SHA-1 is something we
//! think should be taken seriously.
//!
//! For all of the above reasons, we opted to base Artifact IDs on SHA-256,
//! rather than SHA-1 or SHA-1CD.
//!
//! ## Unconditional Newline Normalization
//!
//! The final requirement of note is the unconditional newline normalization
//! performed for Artifact IDs. This is a feature that Git offers which is
//! configurable, permitting users of Git to decide whether checked-out files
//! should have newlines converted to the ones for their current platform, and
//! whether the checked-in copies should have _their_ newlines converted.
//!
//! For our case, we care that users of Artifact IDs can produce the same ID
//! regardless of what platform they're on. To ensure this, we always normalize
//! newlines from `\r\n` to `\n` (CRLF to LF / Windows to Unix). We perform
//! this regardless of the _type_ of input file, whether it's a binary or text
//! file. Since we aren't storing files, only identifying them, we don't have
//! to worry about not newline normalizing binaries.
//!
//! So that's it! Artifact IDs are Git Object Identifiers made with the `blob`
//! type, SHA-256 as the hash algorithm, and unconditional newline
//! normalization.
//!
//! # Crate Overview
//!
//! This crate is built around two central types, [`ArtifactId`]
//! and [`InputManifest`]. The rest of
//! the crate is in service of producing, consuming, and storing these types.
//!
//! ## Creating Artifact IDs
//!
//! [`ArtifactId`]s are created with an [`ArtifactIdBuilder`]. You can get a
//! builder with either [`ArtifactId::builder`] or a constructor on
//! [`ArtifactIdBuilder`] directly. There are convenience constructors for each
//! of the three built-in [`HashProvider`](crate::hash_provider::HashProvider)s:
//!
//! - [`ArtifactIdBuilder::with_rustcrypto`]: Build Artifact IDs with RustCrypto.
//! - [`ArtifactIdBuilder::with_boringssl`]: Build Artifact IDs with BoringSSL.
//! - [`ArtifactIdBuilder::with_openssl`]: Build Artifact IDs with OpenSSL.
//!
//! Artifact IDs can be made from many different kinds of input types, and
//! both synchronously and asynchronously. The following builder methods are
//! available:
//!
//! | Method                                       | Input Type                         | Sync or Async? |
//! |:---------------------------------------------|:-----------------------------------|:---------------|
//! | [`ArtifactIdBuilder::identify_bytes`]        | `&[u8]`                            | Sync           |
//! | [`ArtifactIdBuilder::identify_string`]       | `&str`                             | Sync           |
//! | [`ArtifactIdBuilder::identify_file`]         | `&mut File`                        | Sync           |
//! | [`ArtifactIdBuilder::identify_path`]         | `&Path`                            | Sync           |
//! | [`ArtifactIdBuilder::identify_reader`]       | `R: Read + Sync`                   | Sync           |
//! | [`ArtifactIdBuilder::identify_async_file`]   | `&mut tokio::fs::File`             | Async          |
//! | [`ArtifactIdBuilder::identify_async_path`]   | `&Path`                            | Async          |
//! | [`ArtifactIdBuilder::identify_async_reader`] | `R: AsyncRead + AsyncSync + Unpin` | Async          |
//! | [`ArtifactIdBuilder::identify_manifest`]     | `&InputManifest`                   | Sync           |
//!
//! ## Creating Input Manifests
//!
//! [`InputManifest`]s are created with an [`InputManifestBuilder`]. This type
//! is parameterized over three things: the hash algorithm to use, the hash
//! provider to use, and the storage to use. The usual flow of constructing
//! an [`InputManifest`] is to create a new [`InputManifestBuilder`], add
//! entries with [`InputManifestBuilder::add_relation`], and complete
//! the build with [`InputManifestBuilder::finish`].
//!
//! ## Hash Algorithms and Hash Providers
//!
//! Artifact IDs and Input Manifests are based on a chosen hash algorithm.
//! Today, OmniBOR only supports SHA-256, though alternatives may be added in
//! the future if SHA-256's cryptographic properties are broken.
//!
//! The [`HashAlgorithm`](crate::hash_algorithm::HashAlgorithm) trait is
//! implemented by the [`Sha256`](crate::hash_algorithm::Sha256) type, and
//! both [`ArtifactId`]s and [`InputManifest`]s
//! are parameterized over their [`HashAlgorithm`](crate::hash_algorithm::HashAlgorithm).
//!
//! We also support plugging in arbitrary "hash providers," libraries which
//! provide implementations of cryptographic hashes. Today, we support
//! [RustCrypto](https://github.com/rustcrypto), [OpenSSL](https://www.openssl.org/),
//! and [BoringSSL](https://boringssl.googlesource.com/boringssl).
//!
//! All hash providers are represented by a type implementing the
//! [`HashProvider`](crate::hash_provider::HashProvider) trait; so we have
//! [`RustCrypto`](crate::hash_provider::RustCrypto),
//! [`OpenSsl`](crate::hash_provider::OpenSsl), and
//! [`BoringSsl`](crate::hash_provider::BoringSsl), respectively.
//! All APIs in the crate for creating [`ArtifactId`]s or
//! [`InputManifest`]s are
//! parameterized over the [`HashProvider`](crate::hash_provider::HashProvider).
//! We also provide convenience methods to choose one of the built-in providers.
//!
//! Providers are conditionally compiled in based on crate features. By default,
//! only the `provider-rustcrypto` feature is turned on. Any combination of
//! these may be included. In all cases, they are vendored in and do not link
//! to any system instances of these libraries.
//!
//! In the future we plan to support linking to system instances of OpenSSL and
//! BoringSSL.
//!
//! ## Storing Input Manifests
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
//! # Foreign Function Interface
//!
//! This crate experimentally exposes a Foreign Function Interface (FFI), to
//! make it usable from languages besides Rust. Today this only includes
//! working with Artifact IDs when using the RustCrypto hash provider. This
//! interface is unstable, though we plan to grow it to cover the complete API
//! surface of the crate, including all hash providers and arbitrary other
//! hash providers, and to become stable.
//!
//! # Specification Compliance
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
//! # Comparison with other Software Identifiers
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
//! # What's Missing for 1.0.0
//!
//! The following items are things we'd want to complete before committing to
//! stability in this crate with a 1.0.0 release:
//!
//! - [ ] Embedding Support
//!   - [ ] Support for embedding in ELF files
//!   - [ ] Support for embedding in Mach-O binary files
//!   - [ ] Support for embedding in Portable Executable files
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
//! [cli]: https://github.com/omnibor/omnibor-rs/releases?q=omnibor-cli
//! [swid]: https://csrc.nist.gov/projects/software-identification-swid
//! [nix]: https://www.tweag.io/blog/2024-03-12-nix-as-software-identifier/
//! [swhid]: https://www.swhid.org/
//! [Issue Tracker]: https://github.com/omnibor/omnibor-rs/issues
//! [Contributor Guide]: https://github.com/omnibor/omnibor-rs/blob/main/CONTRIBUTING.md

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
pub use crate::artifact_id::ArtifactIdBuilder;
pub use crate::input_manifest::InputManifest;
pub use crate::input_manifest::InputManifestBuilder;
pub use crate::input_manifest::InputManifestRelation;
