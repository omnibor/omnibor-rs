//! Reproducible artifact identifier.

pub(crate) mod identify;
pub(crate) mod identify_async;

pub use crate::artifact_id::identify::Identify;
pub use crate::artifact_id::identify_async::IdentifyAsync;

use {
    crate::{
        error::ArtifactIdError,
        gitoid::GitOid,
        hash_algorithm::{HashAlgorithm, Sha256},
        object_type::Blob,
    },
    core::{
        cmp::Ordering,
        fmt::{Debug, Formatter, Result as FmtResult},
        hash::{Hash, Hasher},
    },
    std::{fmt::Display, str::FromStr},
};

#[cfg(feature = "serde")]
use {
    serde::{de::Deserializer, Deserialize, Serialize, Serializer},
    std::result::Result as StdResult,
};

/// A universally reproducible software identifier.
///
/// An Artifact ID is a Git Object Identifier (GitOID), with only a type of
/// "blob," with SHA-256 as the hash function, and with unconditional newline
/// normalization.
///
/// If that explanation makes sense, then congrats, that's all you need to know!
///
/// Otherwise, to explain in more detail:
///
/// ## The GitOID Construction
///
/// The Git Version Control System identifies all objects checked into a
/// repository by calculating a Git Object Identifier. This identifier is based
/// around a hash function and what we'll call the "GitOID Construction" that
/// determines what gets input into the hash function.
///
/// In the GitOID Construction, you first hash in a prefix string of the form:
///
/// ```ignore,custom
/// <object_type> <size_of_input_in_bytes>\0
/// ```
///
/// The `<object_type>` can be `blob`, `commit`, `tag`, or `tree`. The last
/// three are used for commits, tags, and directories respectively; `blob` is
/// used for files.
///
/// The `<size_of_input_in_bytes>` is what it sounds like; Git calculates the
/// size of an input file and includes that in the hash.
///
/// After hashing in the prefix string, Git then hashes the contents of the
/// file being identified. That's the GitOID Construction! Artifact IDs use
/// this same construction, with the `<object_type>` always set to `blob`.
///
/// ## Choice of Hash Function
///
/// We also restrict the hash function to _only_ SHA-256 today,
/// though the specification leaves open the possibility of transitioning to
/// an alternative in the future if SHA-256 is cryptographically broken.
///
/// This is a difference from Git's default today. Git normally uses SHA-1,
/// and is in the slow process of transitioning to SHA-256. So why not use
/// SHA-1 to match Git's current default?
///
/// First, it's worth saying that Git can use SHA-1 _or_ a variant of SHA-1
/// called "SHA-1CD" (sometimes spelled "SHA-1DC"). Back in 2017, researchers
/// from Google and CWI Amsterdam announced the "SHAttered" attack against
/// SHA-1, where they had successively engineered a collision (two different
/// documents which produced the same SHA-1 hash). The SHA-1CD algorithm was
/// developed in response. It's a variant of SHA-1 which attempts to detect
/// when the input is attempting to produce a collision like the one in the
/// SHAttered attack, and on detection modifies the hashing algorithm to
/// produce a different hash and stop that collision.
///
/// Different versions of Git will use either SHA-1 or SHA-1CD by default. This
/// means that for Artifact IDs our choice of hash algorithm was between three
/// choices: SHA-1, SHA-1CD, or SHA-256.
///
/// The split of SHA-1 and SHA-1CD doesn't matter for most Git users, since
/// a single repository will just use one or the other and most files will
/// not trigger the collision detection code path that causes their outputs to
/// diverge. For Artifact IDs though, it's a problem, since we care strongly
/// about our IDs being universally reproducible. Thus, the split creates a
/// challenge for our potential use of SHA-1.
///
/// Additionally, it's worth noting that attacks against SHA-1 continue to
/// become more practical as computing hardware improves. In October 2024
/// NIST, the National Institute of Standards and Technology in the United
/// States, published an initial draft of a document "Transitioning the Use of
/// Cryptographic Algorithms and Key Lengths." While it is not yet an official
/// NIST recommendation, it does explicitly disallow the use of SHA-1 for
/// digital signature generation, considers its use for digital signature
/// verification to be a "legacy use" requiring special approval, and otherwise
/// prepares to sunset any use of SHA-1 by 2030.
///
/// NIST is not a regulatory agency, but their recommendations _are_ generally
/// incorporated into policies both in government and in private industry, and
/// a NIST recommendation to fully transition away from SHA-1 is something we
/// think should be taken seriously.
///
/// For all of the above reasons, we opted to base Artifact IDs on SHA-256,
/// rather than SHA-1 or SHA-1CD.
///
/// ## Unconditional Newline Normalization
///
/// The final requirement of note is the unconditional newline normalization
/// performed for Artifact IDs. This is a feature that Git offers which is
/// configurable, permitting users of Git to decide whether checked-out files
/// should have newlines converted to the ones for their current platform, and
/// whether the checked-in copies should have _their_ newlines converted.
///
/// For our case, we care that users of Artifact IDs can produce the same ID
/// regardless of what platform they're on. To ensure this, we always normalize
/// newlines from `\r\n` to `\n` (CRLF to LF / Windows to Unix). We perform
/// this regardless of the _type_ of input file, whether it's a binary or text
/// file. Since we aren't storing files, only identifying them, we don't have
/// to worry about not newline normalizing binaries.
///
/// So that's it! Artifact IDs are Git Object Identifiers made with the `blob`
/// type, SHA-256 as the hash algorithm, and unconditional newline
/// normalization.
pub struct ArtifactId<H: HashAlgorithm> {
    #[doc(hidden)]
    gitoid: GitOid<H, Blob>,
}

impl ArtifactId<Sha256> {
    /// Identify the target artifact with the SHA-256 hash function.
    ///
    /// # Example
    ///
    /// ```
    /// # use omnibor::{ArtifactId, error::ArtifactIdError};
    /// let artifact_id = ArtifactId::sha256(b"hello, world")?;
    /// # Ok::<(), ArtifactIdError>(())
    /// ```
    pub fn sha256<I>(target: I) -> Result<Self, ArtifactIdError>
    where
        I: Identify<Sha256>,
    {
        ArtifactId::new(target)
    }

    /// Identify the target artifact with the SHA-256 hash function asynchronously.
    ///
    /// # Example
    ///
    /// ```
    /// # use omnibor::{ArtifactId, error::ArtifactIdError};
    /// # tokio_test::block_on(async {
    /// let artifact_id = ArtifactId::sha256_async("test/data/c/main.c").await?;
    /// # Ok(())
    /// # })?;
    /// # Ok::<(), ArtifactIdError>(())
    /// ```
    pub async fn sha256_async<I>(target: I) -> Result<Self, ArtifactIdError>
    where
        I: IdentifyAsync<Sha256>,
    {
        ArtifactId::new_async(target).await
    }
}

impl<H: HashAlgorithm> ArtifactId<H> {
    /// Identify the target artifact.
    ///
    /// # Example
    ///
    /// ```
    /// # use omnibor::{ArtifactId, error::ArtifactIdError, hash_algorithm::Sha256};
    /// let artifact_id = ArtifactId::<Sha256>::new("test/data/c/main.c")?;
    /// # Ok::<(), ArtifactIdError>(())
    /// ```
    pub fn new<I>(target: I) -> Result<ArtifactId<H>, ArtifactIdError>
    where
        I: Identify<H>,
    {
        target.identify()
    }

    /// Identify the target artifact asynchronously.
    ///
    /// # Example
    ///
    /// ```
    /// # use omnibor::{ArtifactId, error::ArtifactIdError, hash_algorithm::Sha256};
    /// # tokio_test::block_on(async {
    /// let artifact_id = ArtifactId::<Sha256>::new_async("test/data/c/main.c").await?;
    /// # Ok(())
    /// # })?;
    /// # Ok::<(), ArtifactIdError>(())
    /// ```
    pub async fn new_async<I>(target: I) -> Result<ArtifactId<H>, ArtifactIdError>
    where
        I: IdentifyAsync<H>,
    {
        target.identify_async().await
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

    /// Get the underlying bytes of the [`ArtifactId`] hash.
    ///
    /// This slice is the raw underlying buffer of the [`ArtifactId`], exactly
    /// as produced by the hasher.
    ///
    /// # Example
    ///
    /// ```
    /// # use omnibor::ArtifactId;
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::new(b"hello, world").unwrap();
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
    /// ```
    /// # use omnibor::ArtifactId;
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::new(b"hello, world").unwrap();
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
    /// ```
    /// # use omnibor::ArtifactId;
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::new(b"hello, world").unwrap();
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
    /// ```
    /// # use omnibor::ArtifactId;
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::new(b"hello, world").unwrap();
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
    /// ```
    /// # use omnibor::ArtifactId;
    /// # use omnibor::hash_algorithm::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::new(b"hello, world").unwrap();
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
