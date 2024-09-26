# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [omnibor-v0.6.0] - 2024-08-26

Version `0.6.0` of `omnibor` brings a lot of improvements! A quick summary:

- We now produce a Rust library, C static library, and C dynamic library, to
  enable linking with the `omnibor` crate and using it through the Foreign
  Function Interface (FFI).
- The `ArtifactId` type now has a notion of a "safe name," which is a version
  of the string representation of an `ArtifactId` which we expect to be safe
  to set as a file name. There are times when you may want to do this, and it's
  good and useful to have a defined way of representing this.
- Moved the previously-available binary crate under the `omnibor` package out
  to its own `omnibor-cli` crate. This simplifies a lot of things for us in
  terms of handling releases, and hopefully makes using and interacting with
  the crates simpler too.
- Added support for `InputManifest`, including building them, and storing them
  on disk! This was a big lift, and means we now cover a much larger chunk of
  the OmniBOR specification.

We have not yet implemented embedding mode for manifests, meaning when they're
created, the target artifact is not updated to reflect the manifest's ID. The
code _is_ built to support adding this in the future, and we've left `todo`s
in the code for anyone interested in contributing!

## [omnibor-v0.5.1] - 2024-03-07

### Changed

- Teach `cargo-dist` to build CLI (#170)

## [omnibor-v0.5.0] - 2024-03-07

### Changed

- Add gitoid crate README, update crate desc. (#128)
- Implemented async printing in CLI (#130)
- Add 'cargo xtask' for custom tasks (#131)
- Flush prints before exit, add short format (#133)
- Split up CLI crate into modules. (#135)
- Small fixups in the CLI code. (#136)
- First pass at an OmniBOR package README (#141)
- Add size test for ArtifactId<Sha256> (#142)
- Optionally-implement serde for ArtifactId (#146)
- Introduce `omnibor` FFI. (#160)
- Bump `gitoid`: 0.5.0 -> 0.7.0 (#168)
- Update `omnibor` crate CHANGELOG.md
- Release

### Fixed

- Fix 'find' command short format. (#139)
- Don't stop looking at first 'find' match (#143)

## [omnibor-v0.4.0] - 2024-02-22

### Added

- Add `tree` cmd and --format args to CLI (#118)
- Add --hash flag to CLI commands (#119)

### Changed

- Renamed ArtifactId methods and add docs (#115)
- Introduce new OmniBOR CLI. (#117)
- Combine CLI id/tree cmds, add find cmd (#122)

### Fixed

- Fix double-print in tree command. (#120)

## [omnibor-v0.3.0] - 2024-02-20

### Changed

- First draft of README rewrite (#88)
- Windows test, FFI test, and commit msg CI (#106)
- Initial full ArtifactId impl (#114)

### Fixed

- Remove unused dependencies.
- Remove unused dependencies.
- Fix broken compilation of `omnibor` crate.
- Missing prior version bump for OmniBOR in main

[omnibor-v0.6.0]: https://github.com/omnibor/omnibor-rs/compare/omnibor-v0.5.1..omnibor-v0.6.0
[omnibor-v0.5.1]: https://github.com/omnibor/omnibor-rs/compare/omnibor-v0.5.0..omnibor-v0.5.1
[omnibor-v0.5.0]: https://github.com/omnibor/omnibor-rs/compare/omnibor-v0.4.0..omnibor-v0.5.0
[omnibor-v0.4.0]: https://github.com/omnibor/omnibor-rs/compare/omnibor-v0.3.0..omnibor-v0.4.0
[omnibor-v0.3.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.5.0..omnibor-v0.3.0
