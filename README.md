<div align="center">

<br>

<img alt="OmniBOR Logo" width="400rem" src="https://raw.githubusercontent.com/omnibor/omnibor-rs/main/assets/omnibor-logo.svg">

<br>

__Reproducible identifiers &amp; fine-grained build dependency tracking for software artifacts.__

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

| Crate Name    | Type                                                      | Purpose                                   | Links                                                                                                           |
|:--------------|:----------------------------------------------------------|:------------------------------------------|:----------------------------------------------------------------------------------------------------------------|
| `omnibor`     | ![Library](https://img.shields.io/badge/Library-darkblue) | OmniBOR Identifiers and Manifests         | [README][omnibor_r] &middot; [API Docs][omnibor_d] &middot; [Crate][omnibor_cr] |
| `omnibor-cli` | ![Binary](https://img.shields.io/badge/Binary-darkgreen)  | CLI for OmniBOR Identifiers and Manifests | [README][omnibor_cli_r] &middot; [Crate][omnibor_cli_cr]                    |

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

## Discussions & Support

If you've encountered [specific bugs][bugs] or have specific
[feature requests][features], we recommend opening issues in the
[issue tracker][issues]!

However, if you have more open-ended ideas, want to ask questions
about OmniBOR or the OmniBOR Rust implementation, or want to get support
debugging an issue you've encountered, we recommend opening a new
[discussion][discussion].

If you believe you've found a security vulnerability, please
[report it to us][vuln].

## Security

The project maintains an official [Security Policy][security] and accepts
security disclosures through GitHub.

## Code of Conduct

All discussions, issues, pull requests, and other communication spaces
associated with this project require participants abide by the project's
[Code of Conduct][coc].

## License

All crates in this repository are Apache 2.0 licensed. You can read the full
license text in the [`LICENSE`][license] file.

[contributing]: CONTRIBUTING.md
[cbindgen]: https://github.com/eqrion/cbindgen
[cisa_report]: https://www.cisa.gov/sites/default/files/2023-10/Software-Identification-Ecosystem-Option-Analysis-508c.pdf
[cpe]: https://nvd.nist.gov/products/cpe
[gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
[license]: https://github.com/omnibor/omnibor-rs/blob/main/LICENSE
[omnibor]: https://omnibor.io
[omnibor_cr]: https://crates.io/crates/omnibor
[omnibor_r]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor/README.md
[omnibor_d]: https://docs.rs/crate/omnibor/latest
[omnibor_cli_r]: https://github.com/omnibor/omnibor-rs/blob/main/omnibor-cli/README.md
[omnibor_cli_cr]: https://crates.io/crates/omnibor-cli
[omnibor_spec]: https://github.com/omnibor/spec
[purl]: https://github.com/package-url/purl-spec
[xtask_r]: https://github.com/omnibor/omnibor-rs/blob/main/xtask/README.md
[dco]: https://developercertificate.org/
[security]: https://github.com/omnibor/omnibor-rs/blob/main/SECURITY.md
[coc]: https://github.com/omnibor/omnibor-rs/blob/main/CODE_OF_CONDUCT.md
[bugs]: https://github.com/omnibor/omnibor-rs/issues/new?assignees=&labels=&projects=&template=bug_report.md&title=
[features]: https://github.com/omnibor/omnibor-rs/issues/new?assignees=&labels=&projects=&template=feature_request.md&title=
[issues]: https://github.com/omnibor/omnibor-rs/issues
[discussion]: https://github.com/omnibor/omnibor-rs/discussions
[vuln]: https://github.com/omnibor/omnibor-rs/security/advisories/new
