use {
    crate::{
        artifact_id::ArtifactId,
        error::ArtifactIdError,
        gitoid::internal::{gitoid_from_async_reader, gitoid_from_buffer, gitoid_from_reader},
        hash_algorithm::{HashAlgorithm, Sha256},
        hash_provider::HashProvider,
        input_manifest::InputManifest,
        object_type::Blob,
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    std::{
        fs::File,
        io::{Read, Seek},
        marker::PhantomData,
        path::Path,
    },
    tokio::{
        fs::File as AsyncFile,
        io::{AsyncRead, AsyncSeek},
    },
};

/// A builder for [`ArtifactId`]s.
pub struct ArtifactIdBuilder<H: HashAlgorithm, P: HashProvider<H>> {
    _hash_algorithm: PhantomData<H>,
    provider: P,
}

#[cfg(feature = "provider-rustcrypto")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::RustCrypto> {
    /// Create a new [`ArtifactIdBuilder`] with `RustCrypto` as the [`HashProvider`].
    pub fn with_rustcrypto() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::RustCrypto::new(),
        }
    }
}

#[cfg(feature = "provider-boringssl")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::BoringSsl> {
    /// Create a new [`ArtifactIdBuilder`] with `BoringSsl` as the [`HashProvider`].
    pub fn with_boringssl() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::BoringSsl::new(),
        }
    }
}

#[cfg(feature = "provider-openssl")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::OpenSsl> {
    /// Create a new [`ArtifactIdBuilder`] with `OpenSsl` as the [`HashProvider`].
    pub fn with_openssl() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::OpenSsl::new(),
        }
    }
}

impl<H: HashAlgorithm, P: HashProvider<H>> ArtifactIdBuilder<H, P> {
    /// Create a new [`ArtifactIdBuilder`] with the given [`HashProvider`].
    pub fn with_provider(provider: P) -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider,
        }
    }

    /// Create an [`ArtifactId`] for the given bytes.
    pub fn identify_bytes(&self, bytes: &[u8]) -> ArtifactId<H> {
        // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
        let gitoid = gitoid_from_buffer::<H, Blob>(self.provider.digester(), bytes).unwrap();
        ArtifactId::from_gitoid(gitoid)
    }

    /// Create an [`ArtifactId`] for the given string.
    pub fn identify_string(&self, s: &str) -> ArtifactId<H> {
        self.identify_bytes(s.as_bytes())
    }

    /// Create an [`ArtifactId`] for the given file.
    pub fn identify_file(&self, file: &mut File) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.identify_reader(file)
    }

    /// Create an [`ArtifactId`] for the file at the given path.
    pub fn identify_path(&self, path: &Path) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut file =
            File::open(path).map_err(|source| ArtifactIdError::FailedToOpenFileForId {
                path: path.clone_as_boxstr(),
                source: Box::new(source),
            })?;
        self.identify_file(&mut file)
    }

    /// Create an [`ArtifactId`] for the given arbitrary seekable reader.
    pub fn identify_reader<R: Read + Seek>(
        &self,
        reader: R,
    ) -> Result<ArtifactId<H>, ArtifactIdError> {
        let gitoid = gitoid_from_reader::<H, Blob, _>(self.provider.digester(), reader)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Create an [`ArtifactId`] for the given file, asynchronously.
    pub async fn identify_async_file(
        &self,
        file: &mut AsyncFile,
    ) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.identify_async_reader(file).await
    }

    /// Create an [`ArtifactId`] for the file at the given path, asynchronously.
    pub async fn identify_async_path(&self, path: &Path) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut file = AsyncFile::open(path).await.map_err(|source| {
            ArtifactIdError::FailedToOpenFileForId {
                path: path.clone_as_boxstr(),
                source: Box::new(source),
            }
        })?;
        self.identify_async_file(&mut file).await
    }

    /// Create an [`ArtifactId`] for the given arbitrary seekable reader, asynchronously.
    pub async fn identify_async_reader<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        reader: R,
    ) -> Result<ArtifactId<H>, ArtifactIdError> {
        let gitoid =
            gitoid_from_async_reader::<H, Blob, _>(self.provider.digester(), reader).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    /// Create an [`ArtifactId`] for the given [`InputManifest`]
    pub fn identify_manifest(&self, manifest: &InputManifest<H>) -> ArtifactId<H> {
        self.identify_bytes(&manifest.as_bytes()[..])
    }
}
