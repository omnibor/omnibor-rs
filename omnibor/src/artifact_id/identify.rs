use crate::{
    error::ArtifactIdError,
    gitoid::internal::{gitoid_from_buffer, gitoid_from_reader},
    hash_algorithm::HashAlgorithm,
    hash_provider::registry::get_hash_provider,
    object_type::Blob,
    util::clone_as_boxstr::CloneAsBoxstr,
    ArtifactId, InputManifest,
};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{BufReader, Cursor, Read, Seek},
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

/// Types that can be identified with an `ArtifactId`.
pub trait Identify<H>
where
    H: HashAlgorithm,
{
    /// Produce an [`ArtifactId`] with the given hash provider.
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError>;
}

impl<H> Identify<H> for &[u8]
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider().digester();
        let gitoid = gitoid_from_buffer::<H, Blob>(&mut *digester, self).unwrap();
        Ok(ArtifactId::from_gitoid(gitoid))
    }
}

impl<H, const N: usize> Identify<H> for [u8; N]
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self[..].identify()
    }
}

impl<H, const N: usize> Identify<H> for &[u8; N]
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self[..].identify()
    }
}

impl<H> Identify<H> for &str
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify()
    }
}

impl<H> Identify<H> for &String
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify()
    }
}

impl<H> Identify<H> for &OsStr
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify()
    }
}

impl<H> Identify<H> for &OsString
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Path::new(self).identify()
    }
}

impl<H> Identify<H> for &Path
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        File::open(self)
            .map_err(|source| ArtifactIdError::FailedToOpenFileForId {
                path: self.clone_as_boxstr(),
                source: Box::new(source),
            })?
            .identify()
    }
}

impl<H> Identify<H> for &PathBuf
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.deref().identify()
    }
}

impl<H> Identify<H> for File
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        (&self).identify()
    }
}

impl<H> Identify<H> for &File
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider().digester();
        gitoid_from_reader::<H, Blob, _>(&mut *digester, self).map(ArtifactId::from_gitoid)
    }
}

impl<H> Identify<H> for Box<File>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.deref().identify()
    }
}

impl<H> Identify<H> for Rc<File>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.deref().identify()
    }
}

impl<H> Identify<H> for Arc<File>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.deref().identify()
    }
}

impl<H, R> Identify<H> for BufReader<R>
where
    H: HashAlgorithm,
    R: Read + Seek,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider().digester();
        gitoid_from_reader::<H, Blob, _>(&mut *digester, self).map(ArtifactId::from_gitoid)
    }
}

impl<H, R> Identify<H> for &mut R
where
    H: HashAlgorithm,
    R: Read + Seek,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider().digester();
        gitoid_from_reader::<H, Blob, _>(&mut *digester, self).map(ArtifactId::from_gitoid)
    }
}

impl<H, T> Identify<H> for Cursor<T>
where
    H: HashAlgorithm,
    T: AsRef<[u8]>,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        let mut digester = get_hash_provider().digester();
        gitoid_from_reader::<H, Blob, _>(&mut *digester, self).map(ArtifactId::from_gitoid)
    }
}

impl<H> Identify<H> for InputManifest<H>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.as_bytes().identify()
    }
}

impl<H> Identify<H> for &InputManifest<H>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        self.as_bytes().identify()
    }
}

impl<H> Identify<H> for ArtifactId<H>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Ok(self)
    }
}

impl<H> Identify<H> for &ArtifactId<H>
where
    H: HashAlgorithm,
{
    fn identify(self) -> Result<ArtifactId<H>, ArtifactIdError> {
        Ok(*self)
    }
}
