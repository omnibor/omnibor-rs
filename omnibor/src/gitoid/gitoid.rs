//! A gitoid representing a single artifact.

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
