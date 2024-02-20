use crate::Error;
use crate::Result;
use crate::SupportedHash;
use gitoid::Blob;
use gitoid::GitOid;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Read;
use std::io::Seek;
use std::str::FromStr;
use tokio::io::AsyncRead;
use tokio::io::AsyncSeek;
use url::Url;

/// An OmniBOR Artifact Identifier.
///
/// This is a content-based unique identifier for any software artifact.
///
/// It is built around, per the specification, any supported hash algorithm.
/// Currently, only SHA-256 is supported, but others may be added in the future.
pub struct ArtifactId<H: SupportedHash> {
    #[doc(hidden)]
    gitoid: GitOid<H::HashAlgorithm, Blob>,
}

impl<H: SupportedHash> ArtifactId<H> {
    /// Construct an [`ArtifactId`] from an existing [`GitOid`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use gitoid::GitOid;
    /// let gitoid = GitOid::from_str("hello, world");
    /// let id: ArtifactId<Sha256> = ArtifactId::from_gitoid(gitoid);
    /// println!("Artifact ID: {}", id);
    /// ```
    pub fn from_gitoid(gitoid: GitOid<H::HashAlgorithm, Blob>) -> ArtifactId<H> {
        ArtifactId { gitoid }
    }

    /// Construct an [`ArtifactId`] from raw bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// println!("Artifact ID: {}", id);
    /// ```
    pub fn from_bytes<B: AsRef<[u8]>>(content: B) -> ArtifactId<H> {
        ArtifactId::from_gitoid(GitOid::from_bytes(content))
    }

    /// Construct an [`ArtifactId`] from a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID: {}", id);
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<S: AsRef<str>>(s: S) -> ArtifactId<H> {
        ArtifactId::from_gitoid(GitOid::from_str(s))
    }

    /// Construct an [`ArtifactId`] from a synchronous reader.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use std::fs::File;
    /// let file = File::open("test/data/hello_world.txt").unwrap();
    /// let id: ArtifactId<Sha256> = ArtifactId::from_reader(&file).unwrap();
    /// println!("Artifact ID: {}", id);
    /// ```
    pub fn from_reader<R: Read + Seek>(reader: R) -> Result<ArtifactId<H>> {
        let gitoid = GitOid::from_reader(reader)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Construct an [`ArtifactId`] from a synchronous reader with an expected length.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use std::fs::File;
    /// let file = File::open("test/data/hello_world.txt").unwrap();
    /// let id: ArtifactId<Sha256> = ArtifactId::from_reader_with_length(&file, 11).unwrap();
    /// println!("Artifact ID: {}", id);
    /// ```
    pub fn from_reader_with_length<R: Read>(
        reader: R,
        expected_length: usize,
    ) -> Result<ArtifactId<H>> {
        let gitoid = GitOid::from_reader_with_length(reader, expected_length)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Construct an [`ArtifactId`] from an asynchronous reader.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use tokio::fs::File;
    /// # tokio_test::block_on(async {
    /// let mut file = File::open("test/data/hello_world.txt").await.unwrap();
    /// let id: ArtifactId<Sha256> = ArtifactId::from_async_reader(&mut file).await.unwrap();
    /// println!("Artifact ID: {}", id);
    /// # })
    /// ```
    pub async fn from_async_reader<R: AsyncRead + AsyncSeek + Unpin>(
        reader: R,
    ) -> Result<ArtifactId<H>> {
        let gitoid = GitOid::from_async_reader(reader).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Construct an [`ArtifactId`] from an asynchronous reader with an expected length.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use tokio::fs::File;
    /// # tokio_test::block_on(async {
    /// let mut file = File::open("test/data/hello_world.txt").await.unwrap();
    /// let id: ArtifactId<Sha256> = ArtifactId::from_async_reader_with_length(&mut file, 11).await.unwrap();
    /// println!("Artifact ID: {}", id);
    /// # })
    /// ```
    pub async fn from_async_reader_with_length<R: AsyncRead + Unpin>(
        reader: R,
        expected_length: usize,
    ) -> Result<ArtifactId<H>> {
        let gitoid = GitOid::from_async_reader_with_length(reader, expected_length).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Construct an [`ArtifactId`] from a [`Url`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// # use url::Url;
    /// let url = Url::parse("gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03").unwrap();
    /// let id: ArtifactId<Sha256> = ArtifactId::from_url(url).unwrap();
    /// println!("Artifact ID: {}", id);
    /// ```
    pub fn from_url(url: Url) -> Result<ArtifactId<H>> {
        ArtifactId::try_from(url)
    }

    /// Get the [`Url`] representation of the [`ArtifactId`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID URL: {}", id.url());
    /// ```
    pub fn url(&self) -> Url {
        self.gitoid.url()
    }

    /// Get the underlying bytes of the [`ArtifactId`] hash.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID bytes: {:?}", id.as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.gitoid.as_bytes()
    }

    /// Get the bytes of the [`ArtifactId`] hash as a hexadecimal string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID bytes as hex: {}", id.as_hex());
    /// ```
    pub fn as_hex(&self) -> String {
        self.gitoid.as_hex()
    }

    /// Get the name of the hash algorithm used in the [`ArtifactId`] as a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID hash algorithm: {}", id.hash_algorithm());
    /// ```
    pub const fn hash_algorithm(&self) -> &'static str {
        self.gitoid.hash_algorithm()
    }

    /// Get the object type used in the [`ArtifactId`] as a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID object type: {}", id.object_type());
    /// ```
    pub const fn object_type(&self) -> &'static str {
        self.gitoid.object_type()
    }

    /// Get the length in bytes of the hash used in the [`ArtifactId`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use omnibor::ArtifactId;
    /// # use omnibor::Sha256;
    /// let id: ArtifactId<Sha256> = ArtifactId::from_str("hello, world");
    /// println!("Artifact ID hash length in bytes: {}", id.hash_len());
    /// ```
    pub fn hash_len(&self) -> usize {
        self.gitoid.hash_len()
    }
}

impl<H: SupportedHash> FromStr for ArtifactId<H> {
    type Err = Error;

    fn from_str(s: &str) -> Result<ArtifactId<H>> {
        Ok(ArtifactId::from_str(s))
    }
}

impl<H: SupportedHash> Clone for ArtifactId<H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H: SupportedHash> Copy for ArtifactId<H> {}

impl<H: SupportedHash> PartialEq<ArtifactId<H>> for ArtifactId<H> {
    fn eq(&self, other: &Self) -> bool {
        self.gitoid == other.gitoid
    }
}

impl<H: SupportedHash> Eq for ArtifactId<H> {}

impl<H: SupportedHash> PartialOrd<ArtifactId<H>> for ArtifactId<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H: SupportedHash> Ord for ArtifactId<H> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gitoid.cmp(&other.gitoid)
    }
}

impl<H: SupportedHash> Hash for ArtifactId<H> {
    fn hash<H2>(&self, state: &mut H2)
    where
        H2: Hasher,
    {
        self.gitoid.hash(state);
    }
}

impl<H: SupportedHash> Debug for ArtifactId<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ArtifactId")
            .field("gitoid", &self.gitoid)
            .finish()
    }
}

impl<H: SupportedHash> Display for ArtifactId<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.gitoid)
    }
}

impl<H: SupportedHash> TryFrom<Url> for ArtifactId<H> {
    type Error = Error;

    fn try_from(url: Url) -> Result<ArtifactId<H>> {
        let gitoid = GitOid::from_url(url)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}
