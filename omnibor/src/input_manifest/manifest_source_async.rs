use crate::{
    error::InputManifestError,
    hash_algorithm::HashAlgorithm,
    input_manifest::{manifest_source::seal::ManifestSourceSealed, ManifestSource},
    ArtifactId, InputManifest,
};
use std::{
    ffi::{OsStr, OsString},
    ops::Deref,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncReadExt};

/// Types that can be used to load an `InputManifest` from disk asynchronously.
pub trait ManifestSourceAsync<H>: ManifestSourceSealed
where
    H: HashAlgorithm,
{
    #[allow(async_fn_in_trait)]
    /// Construct an [`InputManifest`] from the source, asynchronously.
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError>;
}

/// Treat as a path, load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &str
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        Path::new(self).resolve_async(target).await
    }
}

/// Treat as a path, load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &String
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve_async(target).await
    }
}

/// Treat as a path, load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &OsStr
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        Path::new(self).resolve_async(target).await
    }
}

/// Treat as a path, load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &OsString
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve_async(target).await
    }
}

/// Load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &Path
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        File::open(self)
            .await
            .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?
            .resolve_async(target)
            .await
    }
}

/// Load the file, read the contents.
impl<H> ManifestSourceAsync<H> for &PathBuf
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve_async(target).await
    }
}

impl ManifestSourceSealed for &mut File {}

/// Read the contents.
impl<H> ManifestSourceAsync<H> for &mut File
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let mut contents = String::new();
        self.read_to_string(&mut contents)
            .await
            .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?;
        contents.resolve(target)
    }
}

impl ManifestSourceSealed for File {}

/// Read the contents.
impl<H> ManifestSourceAsync<H> for File
where
    H: HashAlgorithm,
{
    async fn resolve_async(
        mut self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        (&mut self).resolve_async(target).await
    }
}
