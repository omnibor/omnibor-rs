//! A content-addressable identity for software artifacts.
//!
//! ## What are GitOIDs?
//!
//! Git Object Identifiers ([GitOIDs][gitoid]) are a mechanism for
//! identifying artifacts in a manner which is independently reproducible
//! because it relies only on the contents of the artifact itself.
//!
//! The GitOID scheme comes from the Git version control system, which uses
//! this mechanism to identify commits, tags, files (called "blobs"), and
//! directories (called "trees").
//!
//! This implementation of GitOIDs is produced by the [OmniBOR][omnibor]
//! working group, which uses GitOIDs as the basis for OmniBOR Artifact
//! Identifiers.
//!
//! ### GitOID URL Scheme
//!
//! `gitoid` is also an IANA-registered URL scheme, meaning that GitOIDs
//! are represented and shared as URLs. A `gitoid` URL looks like:
//!
//! ```text
//! gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03
//! ```
//!
//! This scheme starts with "`gitoid`", followed by the object type
//! ("`blob`" in this case), the hash algorithm ("`sha256`"), and the
//! hash produced by the GitOID hash construction. Each of these parts is
//! separated by a colon.
//!
//! ### GitOID Hash Construction
//!
//! GitOID hashes are made by hashing a prefix string containing the object
//! type and the size of the object being hashed in bytes, followed by a null
//! terminator, and then hashing the object itself. So GitOID hashes do not
//! match the result of only hashing the object.
//!
//! ### GitOID Object Types
//!
//! The valid object types for a GitOID are:
//!
//! - `blob`
//! - `tree`
//! - `commit`
//! - `tag`
//!
//! Currently, this crate implements convenient handling of `blob` objects,
//! but does not handle ensuring the proper formatting of `tree`, `commit`,
//! or `tag` objects to match the Git implementation.
//!
//! ### GitOID Hash Algorithms
//!
//! The valid hash algorithms are:
//!
//! - `sha1`
//! - `sha1dc`
//! - `sha256`
//!
//! `sha1dc` is actually Git's default algorithm, and is equivalent to `sha1`
//! in _most_ cases. Where it differs is when the hasher detects what it
//! believes to be an attempt to generate a purposeful SHA-1 collision,
//! in which case it modifies the hash process to produce a different output
//! and avoid the malicious collision.
//!
//! Git does this under the hood, but does not clearly distinguish to end
//! users that the underlying hashing algorithm isn't equivalent to SHA-1.
//! This is fine for Git, where the specific hash used is an implementation
//! detail and only matters within a single repository, but for the OmniBOR
//! working group it's important to distinguish whether plain SHA-1 or
//! SHA-1DC is being used, so it's distinguished in the code for this crate.
//!
//! This means for compatibility with Git that SHA-1DC should be used.
//!
//! ## Why Care About GitOIDs?
//!
//! GitOIDs provide a convenient mechanism to establish artifact identity and
//! validate artifact integrity (this artifact hasn't been modified) and
//! agreement (I have the same artifact you have). The fact that they're based
//! only on the type of object ("`blob`", usually) and the artifact itself
//! means they can be derived independently, enabling distributed artifact
//! identification that avoids a central decider.
//!
//! Alternative identity schemes, like Package URLs (purls) or Common Platform
//! Enumerations (CPEs) rely on central authorities to produce identifiers or
//! define the taxonomy in which identifiers are produced.
//!
//! ## Using this Crate
//!
//! The central type of this crate is [`GitOid`], which is generic over both
//! the hash algorithm used and the object type being identified. These are
//! defined by the [`HashAlgorithm`] and [`ObjectType`] traits.
//!
//! ## Example
//!
//! ```text
//! # use gitoid::{Sha256, Blob};
//! type GitOid = gitoid::GitOid<Sha256, Blob>;
//!
//! let gitoid = GitOid::from_str("hello, world");
//! println!("gitoid: {}", gitoid);
//! ```
//!
//! [gitoid]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
//! [omnibor]: https://omnibor.io

mod gitoid_parser;
pub(crate) mod internal;

use {
    crate::{
        error::ArtifactIdError, gitoid::gitoid_parser::GitOidParser, hash_algorithm::HashAlgorithm,
        object_type::ObjectType,
    },
    std::{
        cmp::Ordering,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        marker::PhantomData,
        str::FromStr,
    },
};

#[cfg(feature = "serde")]
use {
    serde::{
        de::{Deserializer, Error as DeserializeError, Visitor},
        Deserialize, Serialize, Serializer,
    },
    std::result::Result as StdResult,
};

/// A struct that computes [gitoids][g] based on the selected algorithm
///
/// [g]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
#[repr(C)]
pub struct GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    #[doc(hidden)]
    pub(crate) _phantom: PhantomData<O>,

    #[doc(hidden)]
    pub(crate) value: <H as HashAlgorithm>::Array,
}

pub(crate) const GITOID_URL_SCHEME: &str = "gitoid";

impl<H, O> GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    /// Get the underlying bytes of the hash.
    pub fn as_bytes(&self) -> &[u8] {
        &self.value[..]
    }

    /// Convert the hash to a hexadecimal string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Get the hash algorithm used for the `GitOid`.
    pub const fn hash_algorithm(&self) -> &'static str {
        H::NAME
    }

    /// Get the object type of the `GitOid`.
    pub const fn object_type(&self) -> &'static str {
        O::NAME
    }

    /// Get the length of the hash in bytes.
    pub fn hash_len(&self) -> usize {
        self.value.len()
    }
}

impl<H, O> FromStr for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Err = ArtifactIdError;

    fn from_str(s: &str) -> Result<GitOid<H, O>, ArtifactIdError> {
        GitOidParser::new(s).parse()
    }
}

impl<H, O> Clone for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, O> Copy for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
}

impl<H, O> PartialEq<GitOid<H, O>> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn eq(&self, other: &GitOid<H, O>) -> bool {
        self.value == other.value
    }
}

impl<H, O> Eq for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
}

impl<H, O> PartialOrd<GitOid<H, O>> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H, O> Ord for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<H, O> Hash for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn hash<H2>(&self, state: &mut H2)
    where
        H2: Hasher,
    {
        self.value.hash(state);
    }
}

impl<H, O> Debug for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("GitOid")
            .field("object_type", &O::NAME)
            .field("hash_algorithm", &H::NAME)
            .field("value", &self.value)
            .finish()
    }
}

impl<H, O> Display for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}:{}:{}:{}",
            GITOID_URL_SCHEME,
            O::NAME,
            H::NAME,
            self.as_hex()
        )
    }
}

#[cfg(feature = "serde")]
impl<H, O> Serialize for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize self as the URL string.
        let self_as_url_str = self.to_string();
        serializer.serialize_str(&self_as_url_str)
    }
}

#[cfg(feature = "serde")]
impl<'de, H, O> Deserialize<'de> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize self from the URL string.
        struct GitOidVisitor<H: HashAlgorithm, O: ObjectType>(PhantomData<H>, PhantomData<O>);

        impl<H: HashAlgorithm, O: ObjectType> Visitor<'_> for GitOidVisitor<H, O> {
            type Value = GitOid<H, O>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                formatter.write_str("a gitoid-scheme URL")
            }

            fn visit_str<E>(self, value: &str) -> StdResult<Self::Value, E>
            where
                E: DeserializeError,
            {
                let id = GitOid::from_str(value).map_err(E::custom)?;
                Ok(id)
            }
        }

        deserializer.deserialize_str(GitOidVisitor(PhantomData, PhantomData))
    }
}
