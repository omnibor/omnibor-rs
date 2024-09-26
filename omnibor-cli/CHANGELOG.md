# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [omnibor-cli-v0.7.0] - 2024-09-26

This release features the first implementation of the `omnibor manifest`
subcommand! At the moment this only allows creation of new manifests with
`omnibor manifest create`, but is the starting point of actually being able
to use the CLI for real use cases.

## [omnibor-cli-v0.6.0] - 2024-03-08

### Changed

- Split out CLI to its own package. (#171)

### Fixed

- Fix broken version parsing in release (#172)

[omnibor-cli-v0.7.0]: https://github.com/omnibor/omnibor-rs/compare/omnibor-cli-v0.6.0..omnibor-cli-v0.7.0
[omnibor-v0.6.0]: https://github.com/omnibor/omnibor-rs/compare/omnibor-v0.5.1..omnibor-cli-v0.6.0
