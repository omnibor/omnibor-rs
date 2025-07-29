
<div align="center">

# `omnibor-cli`

<br>

__Reproducible identifiers &amp; fine-grained build dependency tracking for software artifacts.__

[![Website](https://img.shields.io/badge/website-omnibor.io-blue)](https://omnibor.io) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](https://github.com/omnibor/omnibor-rs/blob/main/LICENSE)

</div>

This package defines an [OmniBOR] Command Line Interface, intended to enable
easier generation and handling of OmniBOR identifiers and manifests.

> [!NOTE]
> This is currently a work-in-progress. Today, the `omnibor` CLI only supports
> working with identifiers, not manifests.

This command is intended to enable integration with shell scripts and other
programs where the overhead of integrating directly with the `omnibor` crate
through the C-language Foreign Function Interface (FFI) may not be worthwhile,
and where the cost of running a shell to execute this CLI isn't problematic.

## Installation

### Install from Release (recommended)

The OmniBOR CLI provides pre-built binaries for the following platforms:

- Apple Silicon macOS
- Intel macOS
- x64 Windows
- x64 Linux
- x64 MUSL Linux

For shell (Linux and macOS; requires `tar`, `unzip`, and either `curl` or `wget`):

```sh
$ curl --proto '=https' --tlsv1.2 -LsSf "https://github.com/omnibor/omnibor-rs/releases/download/omnibor-cli-v0.6.0/omnibor-cli-installer.sh" | sh
```

For Powershell (Windows; requires `tar`, `Expand-Archive`, and `Net.Webclient`):

```powershell
> powershell -c "irm https://github.com/omnibor/omnibor-rs/releases/download/omnibor-cli-v0.6.0/omnibor-cli-installer.ps1 | iex"
```

> [!NOTE]
> Huge shoutout to the folks at [Axo] for making [`cargo-dist`], which makes
> producing these binaries _extremely_ easy.

### Install with `cargo-binstall`

You can also use [`cargo-binstall`] to install the OmniBOR CLI. This requires
both `cargo` and `cargo-binstall` to be installed.

```sh
$ cargo binstall omnibor-cli
```

### Build from source (stable)

You can build from source using Cargo, which requires a recent-enough
Rust toolchain. We do not commit to a Minimum Supported Rust Version,
and generally track stable currently.

```sh
$ cargo install omnibor-cli
```

### Build from source (unstable)

Finally, you can build from the latest source in the repository itself.
While we run continuous integration testing and try not to break the
build in the repository, this runs a higher risk of brokenness or
incompleteness of features relative to versions published to [Crates.io].

This requires `git` to check out the repository, plus a recent-enough
version of Rust. We do not commit to a Minimum Support Rust Version,
and generally track stable currently.

```sh
# Run the following from the root of the repository after checking
# the repository out with `git clone`.
$ cargo install --path omnibor-cli
```

## Examples


<details>
<summary><code>id</code> with Plain Format</summary>

```sh
$ omnibor id create Cargo.toml
# Cargo.toml => gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f
```
</details>



<details>
<summary><code>id</code> with JSON Format</summary>

```sh
$ omnibor id create Cargo.toml -f json
# {"id":"gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f","path":"Cargo.toml"}
```
</details>



<details>
<summary><code>id</code> with Short Format</summary>

```sh
$ omnibor id create Cargo.toml -f short
# gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f
```
</details>



<details>
<summary><code>find</code> with Plain Format</summary>

```sh
$ omnibor id find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f .
# gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f => ./Cargo.toml
```
</details>



<details>
<summary><code>find</code> with JSON Format</summary>

```sh
$ omnibor id find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f . -f json
# {"id":"gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f","path":"./Cargo.toml"}
```
</details>



<details>
<summary><code>find</code> with Short Format</summary>

```sh
$ omnibor id find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f . -f short
# ./Cargo.toml
```
</details>

### Output Formats

Many subcommands support the `-f`/`--format` flag which can be any of the
following:

- `plain` (default): A simple human-readable format which maps between
  paths and identifiers, separated by a fat arrow (`=>`).
- `short`: Just prints the thing being searched for (for the `id` command, an
  Artifact Identifier, for the `find` command, a filesystem path).
- `json`: Prints a JSON object with `path` and `id` string-type fields.

The `short` format is recommended for piping or redirecting into other commands.

The `json` format is recommended for more structured contexts, and can be
passed to `jq` to manipulate.

## License

The OmniBOR CLI source code is licensed under the Apache-2.0 license.

[OmniBOR]: https://omnibor.io
[release]: https://github.com/omnibor/omnibor-rs/releases
[Axo]: https://axo.dev/
[`cargo-dist`]: https://github.com/axodotdev/cargo-dist
[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall
