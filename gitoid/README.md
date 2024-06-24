
# `gitoid` crate

This crate implements `GitOid`s, Git Object Identifiers, in Rust. The crate
is created and maintained by the [OmniBOR] project, and is intended primarily
for that project's use cases.

## Usage

The key type of this crate is `GitOid`, which is parameterized over two traits:
`HashAlgorithm` and `ObjectType`. Both of these are sealed traits, which means
they are _only_ implementable by types found in the `gitoid` crate itself. To
use the `GitOid` type, you must provide these type parameters like so:

```rust
use gitoid::{GitOid, Sha256, Blob};

fn main() {
    let id = GitOid::<Sha256, Blob>::from_str("hello, world");
    println!("{}", id);
}
```

If you intend to use just a specific instantiation of the `GitOid` type, you
can make this a bit cleaner with a type alias:

```rust
use gitoid::{Sha256, Blob};

type GitOid = gitoid::GitOid<Sha256, Blob>;

fn main() {
    let id = GitOid::from_str("hello, world");
    println!("{}", id);
}
```

## Design

This crate is designed to limit the size of the `GitOid` type in memory, and to
place as much work as possible at compile time. To that end, the `GitOid` type
uses a generic array under the hood to ensure the storage buffer is exactly sized
to the number of bytes required to store the hashes output by the chosen hash
algorithm. The hash algorithm and object type information are also wired up at
compile time through method calls on the `GitOid` type, so they can be accessible
at runtime without actually being stored on a per-`GitOid`-basis.

### Git Compatibility

This crate actually diverges from Git's handling of object identifiers in two
meaningful ways.

1. The in-memory representation of GitOIDs is different in the `gitoid` crate
   and in `git` itself. In Git, the relevant type is called `object_id`, and
   is [defined as follows][git_object_id]:

   ```c
   struct object_id {
       unsigned char hash[GIT_MAX_RAWSZ];
       int algo;	/* XXX requires 4-byte alignment */
   };
   ```

   This type contains a buffer, sized to hold a number of bytes equal to the
   maximum needed by the largest hash supported by Git (currently 32 bytes
   as required by SHA-256), along with an integer which is used to indicated
   the selected hash algorithm. This is ineffecient in the case of hash
   algorithms whose hash output is smaller than 32 bytes (like SHA-1), and
   also means that algorithm selection is delegated to runtime. It also
   doesn't, at the type level or in the embedded data, distinguish between
   the four types of objects supposed for identification by Git: blobs
   (files), commits, tags, and trees (directories). The object types are
   handled by standard formatting rules for producing the input to the hash
   function which produces the hash (this is what we'll call the "GitOID
   hash construction") instead.

   So this representation is less space efficient than it could be and omits
   some information (object type) in favor of an implicit type based on
   the construction of the input to the hash function.

   In the `gitoid` crate, by comparison, the _only_ thing we store at
   runtime is a buffer sized exactly to the number of bytes needed to store
   the hash output by the chosen hash function, and we use zero-cost compile
   time features to encode the hash algorithm and object type.

   We _do not_ currently implement handling for object types besides `blob`,
   because that's all we need for the OmniBOR project, and would love to
   add support for `tree`, `commit`, and `tag` in the future.
2. The Git project talks about Git Object IDs being done either with the
   SHA-1 hash algorithm or with SHA-256, but that's actually not _quite_
   true. The SHA-1 algorithm is known to be broken, with the ability for
   attackers to instigate collisions, and to limit the impact of this
   breakage, Git by default uses a variant of SHA-1 called SHA-1CD (short
   for "SHA-1 with Collision Detection). This algorithm checks data being
   hashed for the presence of some collision-generating vectors of data, and
   if those are detected, it modifies the hashing in a way that stops the
   collision from happening.

   For Git's purposes, this white lie is tolerable, because the IDs are never
   intended for use outside of Git, but for the purpose of OmniBOR we care
   about being completely accurate about the construction used since IDs are
   intended to be independently reprodicible by _anyone_.

   In this crate, we therefore distinguish between the `sha1` algorithm and
   the `sha1cd` algorithm. This is reflected in the `gitoid`-scheme URLs
   generated when using the `GitOid` type.

## Boring Feature

The `gitoid` crate supports using the BoringSSL cryptographic library for SHA-1
and SHA-256 hashing through the `boring` feature. This can be useful for
environments where BoringSSL is preferred or required for compliance reasons.

### Enabling the Boring Feature

To enable the `boring` feature, add the following to your `Cargo.toml`:

```toml
[dependencies]
gitoid = { version = "0.7.1", features = ["boring"] }
```

When the `boring` feature is enabled, the crate will use BoringSSL's
implementations of SHA-1 and SHA-256 instead of the default RustCrypto
implementations. Note that `sha1cd` is not supported by the `boring` feature
and will fall back to using the RustCrypto implementation.

## Minimum Supported Rust Version (MSRV)

This crate does not maintain a Minimum Supported Rust Version, and generally
tracks the latest Rust stable version.

## License

This crate is Apache 2.0 licensed.


[OmniBOR]: https://omnibor.io
[git_object_id]: https://github.com/git/git/blob/f41f85c9ec8d4d46de0fd5fded88db94d3ec8c11/hash-ll.h#L133-L136
