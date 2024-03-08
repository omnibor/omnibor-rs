
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

## License

All of the OmniBOR Rust implementation is Apache-2.0 licensed.

[OmniBOR]: https://omnibor.io
