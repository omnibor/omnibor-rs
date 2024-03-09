
# `omnibor-cli`

This package defines an [OmniBOR] Command Line Interface, intended to enable
easier generation and handling of OmniBOR identifiers and manifests.

> [!NOTE]
> This is currently a work-in-progress. Today, the `omnibor` CLI only supports
> working with identifiers, not manifests.

## Installation

The OmniBOR CLI provides pre-built binaries for a number of platforms. Check
out the release notes for the most recent [release] to get an installer for
any of the following platforms:

- Apple Silicon macOS
- Intel macOS
- x64 Windows
- x64 Linux
- x64 MUSL Linux

Huge shoutout to the folks at [Axo] for making [`cargo-dist`], which makes
producing these binaries _extremely_ easy.

You can also try installing using [`cargo-binstall`], though we haven't
tested that yet.

If you want to build from source, and you have a recent enough Rust
toolchain, you can:

- Run `cargo install omnibor-cli`
- Check out this repository and run `cargo install --path omnibor-cli`

We currently do not commit to a Minimum Supported Rust Version (MSRV),
and generally track stable. This may change in the future.

## Examples



<details>
<summary><code>id</code> with Plain Format</summary>

```sh
$ omnibor id Cargo.toml
# Cargo.toml => gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f
```
</details>



<details>
<summary><code>id</code> with JSON Format</summary>

```sh
$ omnibor id Cargo.toml -f json
# {"id":"gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f","path":"Cargo.toml"}
```
</details>



<details>
<summary><code>id</code> with Short Format</summary>

```sh
$ omnibor id Cargo.toml -f short
# gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f
```
</details>



<details>
<summary><code>find</code> with Plain Format</summary>

```sh
$ omnibor find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f .
# gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f => ./Cargo.toml
```
</details>



<details>
<summary><code>find</code> with JSON Format</summary>

```sh
$ omnibor find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f . -f json
# {"id":"gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f","path":"./Cargo.toml"}
```
</details>



<details>
<summary><code>find</code> with Short Format</summary>

```sh
$ omnibor find gitoid:blob:sha256:c54d66281dea2bf213083f9bd3345d89dc6657fa554b1c9ef14cfe4bab14893f . -f short
# ./Cargo.toml
```
</details>

## Usage

<details>
<summary><code>omnibor --help</code></summary>

```
Usage: omnibor [OPTIONS] <COMMAND>

Commands:
  id    For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it
  find  Find file matching an Artifact ID
  help  Print this message or the help of the given subcommand(s)

Options:
  -b, --buffer <BUFFER>  How many print messages to buffer at one time, tunes printing perf
  -h, --help             Print help
  -V, --version          Print version
```
</details>

<details>
<summary><code>omnibor id --help</code></summary>

```
For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it

Usage: omnibor id [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to identify

Options:
  -f, --format <FORMAT>  Output format (can be "plain", "short", or "json") [default: plain]
  -H, --hash <HASH>      Hash algorithm (can be "sha256") [default: sha256]
  -h, --help             Print help
```
</details>

<details>
<summary><code>omnibor find --help</code></summary>

```
Find file matching an Artifact ID

Usage: omnibor find [OPTIONS] <URL> <PATH>

Arguments:
  <URL>   `gitoid` URL to match
  <PATH>  The root path to search under

Options:
  -f, --format <FORMAT>  Output format (can be "plain", "short", or "json") [default: plain]
  -h, --help             Print help
```
</details>

### Output Formats

Both the `id` and `find` subcommand support a `-f`/`--format` flag which can be
any of the following:

- `plain` (default): A simple human-readable format which maps between
  paths and identifiers, separated by a fat arrow (`=>`).
- `short`: Just prints the thing being searched for (for the `id` command, an
  Artifact Identifier, for the `find` command, a filesystem path).
- `json`: Prints a JSON object with `path` and `id` string-type fields.

The `short` format is recommended for piping or redirecting into other commands.

The `json` format is recommended for more structured contexts, and can be
passed to `jq` to manipulate.

## Minimum Supported Rust Version (MSRV)

This crate does not maintain a Minimum Supported Rust Version, and generally
tracks the latest Rust stable version.

## License

The OmniBOR CLI source code is licensed under the Apache-2.0 license.

[OmniBOR]: https://omnibor.io
[release]: https://github.com/omnibor/omnibor-rs/releases
[Axo]: https://axo.dev/
[`cargo-dist`]: https://github.com/axodotdev/cargo-dist
[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall
