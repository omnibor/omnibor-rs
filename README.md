# OmniBOR Rust

This repository contains two Rust crates. To use either crate, please review
their respective `README.md` files:

- [`gitoid`][gitoid_crate]: an implementation of GitOIDs ([View the `README.md`][gitoid_readme])
- [`omnibor`][omnibor_crate]: an implementation of OmniBOR IDs and manifests ([View the `README.md`][omnibor_readme])

## What is OmniBOR?

[OmniBOR][omnibor] is a draft specification which defines two key concepts:

- __Artifact Identifiers__: independently-reproducible identifiers for
  software artifacts.
- __Artifact Input Manifests__: record the IDs of every input used in the
  build process for an artifact.

Artifact IDs enable _anyone_ to identify and cross-reference information for
software artifacts without a central authority. Unlike [pURL][purl] or [CPE][cpe],
OmniBOR Artifact IDs don't rely on a third-party, they are _inherent
identifiers_ determined only by an artifact itself. They're based on
[Git's Object IDs (GitOIDs)][gitoid] in both construction and choice of
cryptographic hash functions.

Artifact Input Manifests allow consumers to reconstruct Artifact Dependency
Graphs that give _fine-grained_ visibility into how artifacts in your
software supply chain were made. With these graphs, consumers could
in the future identify the presence of exact files associated with known
vulnerabilities, side-stepping the complexities of matching version numbers
across platforms and patching practicies.

[__You can view the OmniBOR specification here.__][omnibor_spec]

The United States Cybersecurity & Infrastructure Security Agency (CISA),
identified OmniBOR as a major candidate for software identities
in its 2023 report ["Software Identification Ecosystem Option
Analysis."][cisa_report]

## Rust Implementation Design Goals

The OmniBOR Rust implementation is designed with the following goals in mind:

- __Cross-language readiness__: The OmniBOR Rust implementation should be
  built with solid Foreign Function Interface (FFI) support, so it can be
  used as the basis for libraries in other languages.
- __Multi-platform__: The OmniBOR Rust implementation should be ready for
  use in as many contexts as possible, including embedded environments. This
  means supporting use without an allocator to dynamically allocate memory,
  and minimizing the size of any types resident in memory.
- __Fast__: The OmniBOR Rust implementation should run as quickly as possible,
  and be designed for high-performance use cases like rapid large scale
  matching of artifacts to identifiers or construction and analysis of artifact
  dependency graphs.

## Contributing

We're happy to accept contributions!

For bug fixes and minor changes to the implementation, feel free to open an issue
in the issue tracker explaining what you'd like to fix, and then open a Pull
Request with the change.

For larger design changes, you may also want to discuss the changes either in the
issue tracker or in the repository's Discussions page.

## License

The `omnibor` and `gitoid` crates are both Apache 2.0 licensed. You can read the
full license text in the [`LICENSE`][license] file.

[cbindgen]: https://github.com/eqrion/cbindgen
[cisa_report]: https://www.cisa.gov/sites/default/files/2023-10/Software-Identification-Ecosystem-Option-Analysis-508c.pdf
[cpe]: https://nvd.nist.gov/products/cpe
[gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
[gitoid_crate]: https://crates.io/crates/gitoid
[gitoid_readme]: https://github.com/omnibor/omnibor-rs/blob/main/gitoid/README.md
[license]: https://github.com/omnibor/omnibor-rs/blob/main/LICENSE
[omnibor]: https://omnibor.io
[omnibor_crate]: https://crates.io/crates/omnibor
[omnibor_readme]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor/README.md
[omnibor_spec]: https://github.com/omnibor/spec
[purl]: https://github.com/package-url/purl-spec
