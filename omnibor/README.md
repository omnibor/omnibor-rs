
<div align="center">

<br>

# `omnibor`

<br>

__Reproducible identifiers &amp; fine-grained build dependency tracking for software artifacts.__

[![Website](https://img.shields.io/badge/website-omnibor.io-blue)](https://omnibor.io) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](https://github.com/omnibor/omnibor-rs/blob/main/LICENSE)

</div>

This is a Rust implementation of the [OmniBOR] specification, which defines
a reproducible identifier scheme and a fine-grained build dependency tracking
mechanism for software artifacts.

The goal for OmniBOR is to be incorporated into software build tools, linkers,
and more, so software consumers can:

- Reproducibly identify software artifacts;
- Deeply inspect all the components used to construct a software artifact,
  beyond just what packages are in a dependency tree;
- Detect when dependencies change through updated artifact identifiers.

The last point is key: Artifact ID's incorporate dependency information,
forming a variant of a Merkle Tree! This Directed Acyclic Graph (DAG) of
dependencies cascades up evidence of artifact dependency changes, enabling
consumers of software artifacts to know when changes happen, and what
precisely has changed.

> [!IMPORTANT]
> The OmniBOR spec, and this Rust package, are still a work-in-progress.
> This also means it's a great time to contribute!
>
> If you want to contribute to the specification instead, check out the
> [OmniBOR spec] repository.

## Using

### Using from Rust

Run the following to add the library to your own crate.

```sh
$ cargo add omnibor
```

The `omnibor` crate currently exposes the following features:

| Name    | Description                                                 | Default? |
|:--------|:------------------------------------------------------------|:---------|
| `serde` | Add support for serializing and deserializing `ArtifactId`s | No       |

To turn on a feature, you can run `cargo add omnibor --features="<feature name>"`, or
[edit your `Cargo.toml` to activate the feature][features].

### Using from other languages

The `omnibor` crate is designed to also be used from other programming languages!

All API's in the crate are exposed over a Foreign Function Interface, usable by
anything that can consume C code.

The crate is configured to produce files suitable for either static or dynamic linking
with non-Rust code. Additionally, you'll need to use `cbindgen` to produce a header
file which describes the contents of the linkable library.

## Testing

`omnibor` provides a variety of tests, which can be run with `cargo test`. To ensure
serialization and deserialization code are tested as well, run
`cargo test --features="serde"`.

## Design

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
  matching of artifacts to identifiers.
- __Memory efficient__: The OmniBor data is designed to be of minimal size in
  memory, with the understanding that real-world uses of `omnibor` are likely
  to work with large number of identifiers at a time. For example, the type
  `ArtifactId<Sha256>` is exactly 32 bytes, just the number of bytes necessary
  to store the SHA-256 hash.

Usage in `no_std` environments is currently planned but not yet implemented.

## Stability Policy

`omnibor` does intend to follow semantic versioning when publishing new versions.
It is currently pre-`1.0`, which for us means we do not generally aim for
stability of the APIs exposed, as we are still iterating and designing what
we consider to be an ideal API for future stabilization.

That said, we do not break stability in patch releases.

### Minimum Supported Rust Version (MSRV)

This crate does not maintain a Minimum Supported Rust Version, and generally
tracks the latest Rust stable version.

## Contributing

We recommend checking out the full [`CONTRIBUTING.md`] for the OmniBOR Rust
project, which outlines our process.

## License

All of the OmniBOR Rust implementation is Apache 2.0 licensed. Contributions
to `omnibor` are assumed to be made in compliance with the Apache 2.0 license.

[OmniBOR]: https://omnibor.io
[OmniBOR spec]: https://github.com/omnibor/spec
[features]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#choosing-features
[`CONTRIBUTING.md`]: https://github.com/omnibor/omnibor-rs/blob/main/CONTRIBUTING.md
