use crate::{
    error::ArtifactIdError, gitoid::internal::gitoid_from_async_reader,
    hash_algorithm::HashAlgorithm, hash_provider::registry::get_hash_provider_async,
    object_type::Blob, util::clone_as_boxstr::CloneAsBoxstr, ArtifactId,
};
use std::{
    ffi::{OsStr, OsString},
    ops::Deref,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncSeek, BufReader},
};

/// Types that can be identified with an `ArtifactId` asynchronously.
pub trait IdentifyAsync<H>
where
    H: HashAlgorithm,
{
    // For now, we're bound to Tokio, which uses a work-stealing executor, so
    // we just turn off this warning. In theory it would be nice to be able to
    // be more generic and work with other executors.
    #[allow(async_fn_in_trait)]
    /// Produce an [`ArtifactId`] with the given hash provider asynchronously.
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError>;
}

impl<H> IdentifyAsync<H> for &str
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify_async().await
    }
}

impl<H> IdentifyAsync<H> for &String
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify_async().await
    }
}

impl<H> IdentifyAsync<H> for &OsStr
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify_async().await
    }
}

impl<H> IdentifyAsync<H> for &OsString
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify_async().await
    }
}

impl<H> IdentifyAsync<H> for &Path
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        File::open(self)
            .await
            .map_err(|source| ArtifactIdError::FailedToOpenFileForId {
                path: self.clone_as_boxstr(),
                source: Box::new(source),
            })?
            .identify_async()
            .await
    }
}

impl<H> IdentifyAsync<H> for &PathBuf
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.deref().identify_async().await
    }
}

impl<H> IdentifyAsync<H> for &mut File
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider_async().await.digester();
        let gitoid = gitoid_from_async_reader::<H, Blob, _>(&mut *digester, self).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}

impl<H> IdentifyAsync<H> for File
where
    H: HashAlgorithm,
{
    async fn identify_async(mut self) -> Result<ArtifactId<H>, ArtifactIdError> {
        (&mut self).identify_async().await
    }
}

impl<H, R> IdentifyAsync<H> for BufReader<R>
where
    H: HashAlgorithm,
    R: AsyncRead + AsyncSeek + Unpin,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider_async().await.digester();
        let gitoid = gitoid_from_async_reader::<H, Blob, _>(&mut *digester, self).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}

impl<H> IdentifyAsync<H> for ArtifactId<H>
where
    H: HashAlgorithm,
{
    async fn identify_async(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Ok(self)
    }
}
