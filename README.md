# gitbom-rs
"An experimental implementation of GitBOM in Rust"

* GitBOM is now [OmniBOR](https://omnibor.io/)

**NOTICE: This project is still a work in progress and is not ready for any use beyond experimental**

## What is GitBOM?

To quote [the GitBOM website](https://gitbom.dev/):

```
GitBOM is a minimalistic scheme for build tools to:
1. Build a compact artifact tree, tracking every source code file incorporated into each build artifact
2. Embed a unique, content addressable reference for that artifact tree, the GitBOM identifier, into the artifact at build time
```

For information, see [the website](https://gitbom.dev/) and the [list of GitBOM resources](https://gitbom.dev/resources/)

## What is gitbom-rs?

gitbom-rs is an experimental implementation of gitBOM in Rust. This is an important learning exercise and will inform future implementations of gitBOM in the future (both in Rust and in other languages)

## C-Bindings

(TODO - improve this flow)

This project contains an experimental C API for certain functions in the gitoid crate. The intent is to allow any language that can use C bindings to be able to use this API to use the functions in this crate.

We use [cbindgen](https://github.com/eqrion/cbindgen) to generate the C headers for the public C API.

```
$ cargo install --force cbindgen
$ cd gitoid
$ cbindgen --config ../cbindgen.toml --crate gitoid --output gitoid.h
```

Open up the gitoid.h file (this is the main part that needs to be improved - we shouldn't have to manually edit this file).

At the top of the file, you should see these lines

```
/**
 * @file
 * @brief "GitBom"
 */


#ifndef gitbom_h
#define gitbom_h
```

Add `#define NUM_HASH_BYTES 32` to these lines (do not edit anything else in the file!)


```
/**
 * @file
 * @brief "GitBom"
 */


#ifndef gitbom_h
#define gitbom_h
#define NUM_HASH_BYTES 32
```

Save and close.

There are tests for the C bindings in `gitoid/src/ffi/test.c` To exercise them:

```
$ cargo build
$ gcc --std=c11 -o c-test gitoid/test/c/test.c -L target/debug/ -lgitoid
$ LD_LIBRARY_PATH=target/debug/ ./c-test
```
