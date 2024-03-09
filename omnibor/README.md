
# `omnibor` Rust package

This crate implements the [OmniBOR] specification for software identity
and fine-grained dependency tracking. This means it is intended to provide
three things:

- __Artifact Identifiers__: independently-reproducible identifiers for
  software artifacts.
- __Artifact Input Manifests__: manifests which record all inputs used to
  produce a software artifact.
- __Artifact Dependency Graphs__: graphs which represent all known
  dependencies, at the file level, for constructing a software artifact.

> [!IMPORTANT]
> The OmniBOR spec, and this Rust package, are still a work-in-progress.

## Using

Run the following to add the library to your own crate.

```sh
$ cargo add omnibor
```

## Design Goals

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

## Minimum Supported Rust Version (MSRV)

This crate does not maintain a Minimum Supported Rust Version, and generally
tracks the latest Rust stable version.

## License

All of the OmniBOR Rust implementation is Apache-2.0 licensed.

[OmniBOR]: https://omnibor.io
