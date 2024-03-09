<div align="center">

![OmniBOR Logo](https://raw.githubusercontent.com/omnibor/omnibor-rs/main/assets/omnibor-logo.svg)

<span style="font-size: 1.333rem; font-weight: 600">Reproducible identifiers for software artifacts &amp; fine-grained build dependency tracking</span>

[![Website](https://img.shields.io/badge/website-omnibor.io-blue)](https://omnibor.io) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](https://github.com/omnibor/omnibor-rs/blob/main/LICENSE)

</div>

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

## What's in this Repository?

| Crate Name    | Type                                                      | Purpose                                   |                         |                            |                       |                         | 
|:--------------|:----------------------------------------------------------|:------------------------------------------|:------------------------|:---------------------------|:----------------------|:------------------------|
| `omnibor`     | ![Library](https://img.shields.io/badge/Library-darkblue) | OmniBOR Identifiers and Manifests         | [README][omnibor_r]     | [Changelog][omnibor_c]     | [API Docs][omnibor_d] | [Crate][omnibor_cr]     |
| `omnibor-cli` | ![Binary](https://img.shields.io/badge/Binary-darkgreen)  | CLI for OmniBOR Identifiers and Manifests | [README][omnibor_cli_r] | [Changelog][omnibor_cli_c] | N/A                   | [Crate][omnibor_cli_cr] |
| `gitoid`      | ![Library](https://img.shields.io/badge/Library-darkblue) | Git Object Identifiers (GitOIDs)          | [README][gitoid_r]      | [Changelog][gitoid_c]      | [API Docs][gitoid_d]  | [Crate][gitoid_cr]      |
| `xtask`       | ![Binary](https://img.shields.io/badge/Binary-darkgreen)  | OmniBOR Rust Workspace Automation         | [README][xtask_r]       | N/A                        | N/A                   | N/A                     |

## Contributing

__We happily accept contributions to any of the packages in this repository!__

All contributed commits _must_ include a Developer Certificate of Origin
sign-off (use the `--signoff` flag when running `git commit`). This is checked
by Continuous Integration tests to make sure you don't miss it! You can
[learn more on the DCO website][dco].

Contributors do not sign any Contributor License Agreement. Your contributions
remain owned by you, licensed for use in OmniBOR under the terms of the Apache
2.0 license.

Check out the full [Contributing Guide][contributing] to learn more!

## Security

The project maintains an official [Security Policy][security] and accepts
security disclosures through GitHub.

## Code of Conduct

All discussions, issues, pull requests, and other communication spaces
associated with this project require participants abide by the project's
[Code of Conduct][coc] (Contributor Covenant 2.0).

## License

All crates in this repository are Apache 2.0 licensed. You can read the full
license text in the [`LICENSE`][license] file.

[contributing]: CONTRIBUTING.md
[cbindgen]: https://github.com/eqrion/cbindgen
[cisa_report]: https://www.cisa.gov/sites/default/files/2023-10/Software-Identification-Ecosystem-Option-Analysis-508c.pdf
[cpe]: https://nvd.nist.gov/products/cpe
[gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
[gitoid_cr]: https://crates.io/crates/gitoid
[gitoid_r]: https://github.com/omnibor/omnibor-rs/blob/main/gitoid/README.md
[gitoid_c]: https://github.com/omnibor/omnibor-rs/blob/main/gitoid/CHANGELOG.md
[gitoid_d]: https://docs.rs/crate/gitoid/latest
[license]: https://github.com/omnibor/omnibor-rs/blob/main/LICENSE
[omnibor]: https://omnibor.io
[omnibor_cr]: https://crates.io/crates/omnibor
[omnibor_r]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor/README.md
[omnibor_c]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor/CHANGELOG.md
[omnibor_d]: https://docs.rs/crate/omnibor/latest
[omnibor_cli_r]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor-cli/README.md
[omnibor_cli_c]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor-cli/CHANGELOG.md
[omnibor_cli_cr]: https://crates.io/crates/omnibor-cli
[omnibor_spec]: https://github.com/omnibor/spec
[purl]: https://github.com/package-url/purl-spec
[xtask_r]: https://github.com/omnibor/omnibor-rs/blob/main/xtask/README.md
[dco]: https://developercertificate.org/
[security]: https://github.com/omnibor/omnibor-rs/blob/main/SECURITY.md
[coc]: https://github.com/omnibor/omnibor-rs/blob/main/CODE_OF_CONDUCT.md
