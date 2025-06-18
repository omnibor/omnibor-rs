//! A gitoid representing a single artifact.

use {
    crate::{
        error::ArtifactIdError, gitoid::gitoid_url_parser::GitOidUrlParser,
        hash_algorithm::HashAlgorithm, object_type::ObjectType,
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    std::{
        cmp::Ordering,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
        marker::PhantomData,
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
    pub(crate) value: <H as HashAlgorithm>::Array,
}

pub(crate) const GITOID_URL_SCHEME: &str = "gitoid";

impl<H, O> GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    /// Construct a new `GitOid` from a `Url`.
    pub fn try_from_url(url: Url) -> Result<GitOid<H, O>, ArtifactIdError> {
        GitOid::try_from(url)
    }

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
        let url = Url::parse(s).map_err(|source| {
            ArtifactIdError::FailedToParseUrl(s.clone_as_boxstr(), Box::new(source))
        })?;
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

impl<H, O> TryFrom<Url> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Error = ArtifactIdError;

    fn try_from(url: Url) -> Result<GitOid<H, O>, ArtifactIdError> {
        GitOidUrlParser::new(&url).parse()
    }
}
