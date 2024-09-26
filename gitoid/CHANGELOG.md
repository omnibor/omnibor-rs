# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2024-09-26

The big news in `gitoid@0.8.0` is that we've added support for swapping out the
cryptography-provider backend, and now support three options:

- `rustcrypto`
- `boringssl`
- `openssl`

All of these can be controlled via features on the crate. You do have to pick
_at least one_ of them, but you can pick more than one if you want and then
decide which to use at runtime.

We also added some benchmarks to enable comparing them.

Huge thank you to [@fkautz] for the work on this!

[@fkautz]: https://github.com/fkautz

## [0.7.1] - 2024-03-09

### Changed

- Update project and crate READMEs (#173)

## [0.7.0] - 2024-03-07

### Changed

- Fix broken `CHANGELOG.md`
- Rename `GitOid` methods for clarity (#167)
- Update `gitoid` crate CHANGELOG.md
- Release

## [0.6.0] - 2024-03-07

### Changed

- Initial full ArtifactId impl (#114)
- Make GitOid display print URL format (#116)
- Add gitoid crate README, update crate desc. (#128)
- Add 'cargo xtask' for custom tasks (#131)
- Introduce gitoid crate features incl. std. (#148)
- Added 'serde' feature to `gitoid` crate (#149)
- Introduce `omnibor` FFI. (#160)
- Update `gitoid` crate CHANGELOG.md
- Release

## [0.5.0] - 2024-02-19

### Added

- Add top-level docs example. (#113)

### Changed

- Windows test, FFI test, and commit msg CI (#106)
- Simplify GitOid crate substantially. (#108)
- Further simplify GitOID interface. (#109)
- Add async constructors for GitOid. (#110)
- Improve GitOid trait bounds (#111)
- Minor cleanup of docs and trait defs. (#112)

## [0.4.0] - 2024-02-14

### Changed

- Rewrite docs intro, reorg docs slightly (#94)
- Improved gitoid_from_buffer, misc. fixups (#95)
- First draft of README rewrite (#88)
- Simplify GitOid trait bounds (#96)

### Fixed

- Fixed broken FFI code on Windows. (#97)
- Windows FFI mistakenly using BufReader (#98)

## [0.3.0] - 2024-02-12

### Changed

- Make GitOid::new_invalid pub(crate)
- Improve clarity / reliability of FFI tests
- Bump dependency versions to latest.
- Make GitOid::new_invalid pub(crate)
- Improve clarity / reliability of FFI tests
- Bump dependency versions to latest.
- Remove BufReader req for GitOid construction (#85)

### Fixed

- Add missing conditional compilation for unix imports
- Hide C test executables
- Add missing conditional compilation for unix imports
- Hide C test executables
- Moved, cleaned up cbindgen.toml

[0.8.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.7.1..gitoid-v0.8.0
[0.7.1]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.7.0..gitoid-v0.7.1
[0.7.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.6.0..gitoid-v0.7.0
[0.6.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.5.0..gitoid-v0.6.0
[0.5.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.4.0..gitoid-v0.5.0
[0.4.0]: https://github.com/omnibor/omnibor-rs/compare/gitoid-v0.3.0..gitoid-v0.4.0
[0.3.0]: https://github.com/omnibor/omnibor-rs/compare/v0.1.3..gitoid-v0.3.0
