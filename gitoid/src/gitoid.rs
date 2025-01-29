//! A gitoid representing a single artifact.

use crate::{
    internal::gitoid_from_buffer, util::stream_len::stream_len, Error, HashAlgorithm, ObjectType,
    Result,
};
use core::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use digest::OutputSizeUser;

#[cfg(feature = "async")]
use {
    crate::{internal::gitoid_from_async_reader, util::stream_len::async_stream_len},
    tokio::io::{AsyncRead, AsyncSeek},
};

#[cfg(feature = "std")]
use {
    crate::{gitoid_url_parser::GitOidUrlParser, internal::gitoid_from_reader},
    serde::{
        de::{Deserializer, Error as DeserializeError, Visitor},
        Deserialize, Serialize, Serializer,
    },
    std::{
        fmt::Display,
        io::{Read, Seek},
        result::Result as StdResult,
        str::FromStr,
    },
    url::Url,
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
    pub(crate) value: H::Array,
}

#[cfg(feature = "std")]
pub(crate) const GITOID_URL_SCHEME: &str = "gitoid";

impl<H, O> GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    /// Create a new `GitOid` based on a slice of bytes.
    pub fn id_bytes<B: AsRef<[u8]>>(content: B) -> GitOid<H, O> {
        fn inner<H, O>(content: &[u8]) -> GitOid<H, O>
        where
            H: HashAlgorithm,
            O: ObjectType,
        {
            // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
            gitoid_from_buffer(H::new(), content, content.len()).unwrap()
        }

        inner(content.as_ref())
    }

    /// Create a `GitOid` from a UTF-8 string slice.
    pub fn id_str<S: AsRef<str>>(s: S) -> GitOid<H, O> {
        fn inner<H, O>(s: &str) -> GitOid<H, O>
        where
            H: HashAlgorithm,
            O: ObjectType,
        {
            GitOid::id_bytes(s.as_bytes())
        }

        inner(s.as_ref())
    }

    #[cfg(feature = "std")]
    /// Create a `GitOid` from a reader.
    pub fn id_reader<R: Read + Seek>(mut reader: R) -> Result<GitOid<H, O>> {
        let expected_length = stream_len(&mut reader)? as usize;
        GitOid::id_reader_with_length(reader, expected_length)
    }

    #[cfg(feature = "std")]
    /// Generate a `GitOid` from a reader, providing an expected length in bytes.
    pub fn id_reader_with_length<R>(reader: R, expected_length: usize) -> Result<GitOid<H, O>>
    where
        R: Read + Seek,
    {
        gitoid_from_reader(H::new(), reader, expected_length)
    }

    #[cfg(feature = "async")]
    /// Generate a `GitOid` from an asynchronous reader.
    pub async fn id_async_reader<R: AsyncRead + AsyncSeek + Unpin>(
        mut reader: R,
    ) -> Result<GitOid<H, O>> {
        let expected_length = async_stream_len(&mut reader).await? as usize;
        GitOid::id_async_reader_with_length(reader, expected_length).await
    }

    #[cfg(feature = "async")]
    /// Generate a `GitOid` from an asynchronous reader, providing an expected length in bytes.
    pub async fn id_async_reader_with_length<R: AsyncRead + AsyncSeek + Unpin>(
        reader: R,
        expected_length: usize,
    ) -> Result<GitOid<H, O>> {
        gitoid_from_async_reader(H::new(), reader, expected_length).await
    }

    #[cfg(feature = "std")]
    /// Construct a new `GitOid` from a `Url`.
    pub fn try_from_url(url: Url) -> Result<GitOid<H, O>> {
        GitOid::try_from(url)
    }

    #[cfg(feature = "std")]
    /// Get a URL for the current `GitOid`.
    pub fn url(&self) -> Url {
        // PANIC SAFETY: We know that this is a valid URL;
        //               our `Display` impl is the URL representation.
        Url::parse(&self.to_string()).unwrap()
    }

    /// Get the underlying bytes of the hash.
    pub fn as_bytes(&self) -> &[u8] {
        &self.value[..]
    }

    #[cfg(feature = "std")]
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
        <H::Alg as OutputSizeUser>::output_size()
    }
}

#[cfg(feature = "std")]
impl<H, O> FromStr for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<GitOid<H, O>> {
        let url = Url::parse(s)?;
        GitOid::try_from_url(url)
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

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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
        let self_as_url_str = self.url().to_string();
        serializer.serialize_str(&self_as_url_str)
    }
}

#[cfg(feature = "std")]
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
                let url = Url::parse(value).map_err(E::custom)?;
                let id = GitOid::try_from(url).map_err(E::custom)?;
                Ok(id)
            }
        }

        deserializer.deserialize_str(GitOidVisitor(PhantomData, PhantomData))
    }
}

#[cfg(feature = "std")]
impl<H, O> TryFrom<Url> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Error = Error;

    fn try_from(url: Url) -> Result<GitOid<H, O>> {
        GitOidUrlParser::new(&url).parse()
    }
}
