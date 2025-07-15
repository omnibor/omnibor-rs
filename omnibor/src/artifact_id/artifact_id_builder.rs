use {
    crate::{
        artifact_id::{identify_async::IdentifyAsync, ArtifactId, Identify},
        hash_algorithm::{HashAlgorithm, Sha256},
        hash_provider::HashProvider,
    },
    std::marker::PhantomData,
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

    /// Identify the target artifact.
    pub fn identify<I>(&self, target: I) -> Result<ArtifactId<H>, I::Error>
    where
        I: Identify<H>,
    {
        target.identify(self.provider)
    }

    /// Identify the target artifact asynchronously.
    pub async fn identify_async<I>(&self, target: I) -> Result<ArtifactId<H>, I::Error>
    where
        I: IdentifyAsync<H>,
    {
        target.identify_async(self.provider).await
    }
}
