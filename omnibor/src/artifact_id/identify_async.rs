use crate::{
    error::ArtifactIdError, gitoid::internal::gitoid_from_async_reader,
    hash_algorithm::HashAlgorithm, hash_provider::HashProvider, object_type::Blob,
    util::clone_as_boxstr::CloneAsBoxstr, ArtifactId,
};
use std::{ffi::OsStr, path::Path};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncSeek, BufReader},
};

/// Produce an [`ArtifactId`] asynchronously.
pub trait IdentifyAsync<H>
where
    H: HashAlgorithm,
{
    /// The error produced during identification.
    type Error;

    // For now, we're bound to Tokio, which uses a work-stealing executor, so
    // we just turn off this warning. In theory it would be nice to be able to
    // be more generic and work with other executors.
    #[allow(async_fn_in_trait)]
    /// Produce an [`ArtifactId`] with the given hash provider asynchronously.
    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>;
}

impl<H> IdentifyAsync<H> for &str
where
    H: HashAlgorithm,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        Path::new(self).identify_async(provider).await
    }
}

impl<H> IdentifyAsync<H> for &OsStr
where
    H: HashAlgorithm,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        Path::new(self).identify_async(provider).await
    }
}

impl<H> IdentifyAsync<H> for &Path
where
    H: HashAlgorithm,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        File::open(self)
            .await
            .map_err(|source| ArtifactIdError::FailedToOpenFileForId {
                path: self.clone_as_boxstr(),
                source: Box::new(source),
            })?
            .identify_async(provider)
            .await
    }
}

impl<H> IdentifyAsync<H> for &mut File
where
    H: HashAlgorithm,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        let gitoid = gitoid_from_async_reader::<H, Blob, _>(provider.digester(), self).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}

impl<H> IdentifyAsync<H> for File
where
    H: HashAlgorithm,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(mut self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        (&mut self).identify_async(provider).await
    }
}

impl<H, R> IdentifyAsync<H> for BufReader<R>
where
    H: HashAlgorithm,
    R: AsyncRead + AsyncSeek + Unpin,
{
    type Error = ArtifactIdError;

    async fn identify_async<P>(self, provider: P) -> Result<ArtifactId<H>, Self::Error>
    where
        P: HashProvider<H>,
    {
        let gitoid = gitoid_from_async_reader::<H, Blob, _>(provider.digester(), self).await?;
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}
