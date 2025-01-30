use {
    crate::{
        artifact_id::ArtifactId,
        error::Result,
        gitoid::internal::{gitoid_from_async_reader, gitoid_from_buffer, gitoid_from_reader},
        hash_algorithm::{HashAlgorithm, Sha256},
        hash_provider::HashProvider,
        input_manifest::InputManifest,
        object_type::Blob,
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

pub struct ArtifactIdBuilder<H: HashAlgorithm, P: HashProvider<H>> {
    _hash_algorithm: PhantomData<H>,
    provider: P,
}

#[cfg(feature = "backend-rustcrypto")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::RustCrypto> {
    pub fn with_rustcrypto() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::RustCrypto::new(),
        }
    }
}

#[cfg(feature = "backend-boringssl")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::BoringSsl> {
    pub fn with_boringssl() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::BoringSsl::new(),
        }
    }
}

#[cfg(feature = "backend-openssl")]
impl ArtifactIdBuilder<Sha256, crate::hash_provider::OpenSsl> {
    pub fn with_openssl() -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider: crate::hash_provider::OpenSsl::new(),
        }
    }
}

impl<H: HashAlgorithm, P: HashProvider<H>> ArtifactIdBuilder<H, P> {
    pub fn with_provider(provider: P) -> Self {
        Self {
            _hash_algorithm: PhantomData,
            provider,
        }
    }

    pub fn identify_bytes(&self, bytes: &[u8]) -> ArtifactId<H> {
        // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
        let gitoid = gitoid_from_buffer::<H, Blob>(self.provider.digester(), bytes).unwrap();
        ArtifactId::from_gitoid(gitoid)
    }

    pub fn identify_string(&self, s: &str) -> ArtifactId<H> {
        self.identify_bytes(s.as_bytes())
    }

    pub fn identify_file(&self, file: &mut File) -> Result<ArtifactId<H>> {
        self.identify_reader(file)
    }

    pub fn identify_path(&self, path: &Path) -> Result<ArtifactId<H>> {
        let mut file = File::open(path)?;
        self.identify_file(&mut file)
    }

    pub fn identify_reader<R: Read + Seek>(&self, reader: R) -> Result<ArtifactId<H>> {
        let gitoid = gitoid_from_reader::<H, Blob, _>(self.provider.digester(), reader)?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    pub async fn identify_async_file(&self, file: &mut AsyncFile) -> Result<ArtifactId<H>> {
        self.identify_async_reader(file).await
    }

    pub async fn identify_async_path(&self, path: &Path) -> Result<ArtifactId<H>> {
        let mut file = AsyncFile::open(path).await?;
        self.identify_async_file(&mut file).await
    }

    pub async fn identify_async_reader<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        reader: R,
    ) -> Result<ArtifactId<H>> {
        let gitoid =
            gitoid_from_async_reader::<H, Blob, _>(self.provider.digester(), reader).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }

    pub fn identify_manifest(&self, manifest: &InputManifest<H>) -> ArtifactId<H> {
        self.identify_bytes(&manifest.as_bytes()[..])
    }
}
