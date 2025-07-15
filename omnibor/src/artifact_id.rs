//! Reproducible artifact identifier.

pub(crate) mod identify;
pub(crate) mod identify_async;

pub use crate::artifact_id::identify::Identify;
pub use crate::artifact_id::identify_async::IdentifyAsync;

use {
    crate::{
        error::ArtifactIdError, gitoid::GitOid, hash_algorithm::HashAlgorithm,
        hash_provider::HashProvider, object_type::Blob,
    },
    core::{
        cmp::Ordering,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
    },
    std::{fmt::Display, path::PathBuf, str::FromStr},
};

#[cfg(feature = "serde")]
use {
    serde::{de::Deserializer, Deserialize, Serialize, Serializer},
    std::result::Result as StdResult,
};

#[cfg(doc)]
use crate::hash_algorithm::Sha256;

/// A universally reproducible software identifier.
///
/// This is a content-based unique identifier for any software artifact.
///
/// It is built around, per the specification, any supported hash algorithm.
/// Currently, only SHA-256 is supported, but others may be added in the future.
pub struct ArtifactId<H: HashAlgorithm> {
    #[doc(hidden)]
    gitoid: GitOid<H, Blob>,
}

impl<H: HashAlgorithm> ArtifactId<H> {
    /// Identify the target artifact.
    pub fn identify<P: HashProvider<H>, I: Identify<H>>(
        provider: P,
        target: I,
    ) -> Result<ArtifactId<H>, I::Error> {
        target.identify(provider)
    }

    /// Identify the target artifact asynchronously.
    pub async fn identify_async<P: HashProvider<H>, I: IdentifyAsync<H>>(
        provider: P,
        target: I,
    ) -> Result<ArtifactId<H>, I::Error> {
        target.identify_async(provider).await
    }

    /// Construct an [`ArtifactId`] from an existing `GitOid`.
    ///
    /// This produces an identifier using the provided `GitOid` directly,
    /// without additional validation. The type system ensures the `GitOid`
    /// hash algorithm is one supported for an [`ArtifactId`], and that the
    /// object type is "blob".
    ///
    /// # Note
    ///
    /// This function is not exported because we don't re-export the `GitOid`
    /// type we use, which would mean users of the crate would themselves
    /// need to import a binary-compatible version of the `GitOid` crate as
    /// well. This is extra complexity for minimal gain, so we don't support it.
    ///
    /// If it were ever absolutely needed in the future, we might expose this
    /// constructor with a `#[doc(hidden)]` attribute, or with documentation
    /// which clearly outlines the extra complexity.
    pub(crate) fn from_gitoid(gitoid: GitOid<H, Blob>) -> ArtifactId<H> {
        ArtifactId { gitoid }
    }

    /// Try to construct an [`ArtifactId`] from a filesystem-safe representation.
    pub fn try_from_safe_name(s: &str) -> Result<ArtifactId<H>, ArtifactIdError> {
        ArtifactId::from_str(&s.replace('_', ":"))
    }

    /// Get a filesystem-safe representation of the [`ArtifactId`].
    ///
    /// This is a conservative method that tries to use _only_ characters
    /// which can be expected to work broadly cross-platform.
    ///
    /// What that means for us is that the `:` separator character is
    /// replaced with `_`.
    pub fn as_file_name(&self) -> PathBuf {
        let name = self.gitoid.to_string().replace(':', "_");
        let mut path = PathBuf::from(name);
        path.set_extension("manifest");
        path
    }

    /// Get the underlying bytes of the [`ArtifactId`] hash.
    ///
    /// This slice is the raw underlying buffer of the [`ArtifactId`], exactly
    /// as produced by the hasher.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::{ArtifactId, ArtifactIdBuilder};
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactIdBuilder::with_rustcrypto().identify_string("hello, world");
    /// println!("Artifact ID bytes: {:?}", id.as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.gitoid.as_bytes()
    }

    /// Get the bytes of the [`ArtifactId`] hash as a hexadecimal string.
    ///
    /// This returns a [`String`] rather than [`str`] because the string must be
    /// constructed on the fly, as we do not store a hexadecimal representation
    /// of the hash data.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::{ArtifactId, ArtifactIdBuilder};
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactIdBuilder::with_rustcrypto().identify_string("hello, world");
    /// println!("Artifact ID bytes as hex: {}", id.as_hex());
    /// ```
    pub fn as_hex(&self) -> String {
        self.gitoid.as_hex()
    }

    /// Get the name of the hash algorithm used in the [`ArtifactId`] as a string.
    ///
    /// For [`Sha256`], this is the string `"sha256"`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::{ArtifactId, ArtifactIdBuilder};
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactIdBuilder::with_rustcrypto().identify_string("hello, world");
    /// println!("Artifact ID hash algorithm: {}", id.hash_algorithm());
    /// ```
    pub const fn hash_algorithm(&self) -> &'static str {
        self.gitoid.hash_algorithm()
    }

    /// Get the object type used in the [`ArtifactId`] as a string.
    ///
    /// For all [`ArtifactId`]s this is `"blob"`, but the method is provided
    /// for completeness nonetheless.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::{ArtifactId, ArtifactIdBuilder};
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactIdBuilder::with_rustcrypto().identify_string("hello, world");
    /// println!("Artifact ID object type: {}", id.object_type());
    /// ```
    pub const fn object_type(&self) -> &'static str {
        self.gitoid.object_type()
    }

    /// Get the length in bytes of the hash used in the [`ArtifactId`].
    ///
    /// In the future this method will be `const`, but is not able to be
    /// today due to limitations in the Rust cryptography crates we use
    /// internally.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::{ArtifactId, ArtifactIdBuilder};
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactIdBuilder::with_rustcrypto().identify_string("hello, world");
    /// println!("Artifact ID hash length in bytes: {}", id.hash_len());
    /// ```
    pub fn hash_len(&self) -> usize {
        self.gitoid.hash_len()
    }
}

impl<H: HashAlgorithm> FromStr for ArtifactId<H> {
    type Err = ArtifactIdError;

    fn from_str(s: &str) -> Result<ArtifactId<H>, ArtifactIdError> {
        let gitoid = GitOid::from_str(s)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}

impl<H: HashAlgorithm> Clone for ArtifactId<H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H: HashAlgorithm> Copy for ArtifactId<H> {}

impl<H: HashAlgorithm> PartialEq<ArtifactId<H>> for ArtifactId<H> {
    fn eq(&self, other: &Self) -> bool {
        self.gitoid == other.gitoid
    }
}

impl<H: HashAlgorithm> Eq for ArtifactId<H> {}

impl<H: HashAlgorithm> PartialOrd<ArtifactId<H>> for ArtifactId<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H: HashAlgorithm> Ord for ArtifactId<H> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gitoid.cmp(&other.gitoid)
    }
}

impl<H: HashAlgorithm> Hash for ArtifactId<H> {
    fn hash<H2>(&self, state: &mut H2)
    where
        H2: Hasher,
    {
        self.gitoid.hash(state);
    }
}

impl<H: HashAlgorithm> Debug for ArtifactId<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ArtifactId")
            .field("gitoid", &self.gitoid)
            .finish()
    }
}

impl<H: HashAlgorithm> Display for ArtifactId<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.gitoid)
    }
}

impl<H: HashAlgorithm> From<GitOid<H, Blob>> for ArtifactId<H> {
    fn from(gitoid: GitOid<H, Blob>) -> Self {
        ArtifactId::from_gitoid(gitoid)
    }
}

impl<'r, H: HashAlgorithm> TryFrom<&'r str> for ArtifactId<H> {
    type Error = ArtifactIdError;

    fn try_from(s: &'r str) -> Result<Self, ArtifactIdError> {
        ArtifactId::from_str(s)
    }
}

#[cfg(feature = "serde")]
impl<H: HashAlgorithm> Serialize for ArtifactId<H> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.gitoid.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, H: HashAlgorithm> Deserialize<'de> for ArtifactId<H> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let gitoid = GitOid::<H, Blob>::deserialize(deserializer)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}
