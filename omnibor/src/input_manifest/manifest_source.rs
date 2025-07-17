use crate::{
    artifact_id::ArtifactId, error::InputManifestError, hash_algorithm::HashAlgorithm,
    input_manifest::Input, object_type::Blob, object_type::ObjectType,
    util::clone_as_boxstr::CloneAsBoxstr, InputManifest,
};
use std::{
    ffi::{OsStr, OsString},
    fs::{read_to_string, File},
    io::Read,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Types that can be used to produce an [`InputManifest`].
pub trait ManifestSource<H>
where
    H: HashAlgorithm,
{
    /// Construct an [`InputManifest`] from the source.
    fn resolve(self, target: Option<ArtifactId<H>>)
        -> Result<InputManifest<H>, InputManifestError>;
}

impl<H> ManifestSource<H> for &[u8]
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.to_vec().resolve(target)
    }
}

impl<H> ManifestSource<H> for Vec<u8>
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let s = String::from_utf8(self).map_err(|_| InputManifestError::InvalidCharInManifest)?;
        parse_input_manifest(&s, target)
    }
}

impl<H> ManifestSource<H> for &str
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        parse_input_manifest(self, target)
    }
}

impl<H> ManifestSource<H> for &String
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve(target)
    }
}

impl<H> ManifestSource<H> for &OsStr
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let s = self
            .to_str()
            .ok_or_else(|| InputManifestError::InvalidCharInManifest)?;
        parse_input_manifest(s, target)
    }
}

impl<H> ManifestSource<H> for &OsString
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve(target)
    }
}

impl<H> ManifestSource<H> for &Path
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let s = read_to_string(self)
            .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?;
        parse_input_manifest(&s, target)
    }
}

impl<H> ManifestSource<H> for &PathBuf
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        self.deref().resolve(target)
    }
}

impl<H> ManifestSource<H> for File
where
    H: HashAlgorithm,
{
    fn resolve(
        mut self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        (&mut self).resolve(target)
    }
}

impl<H> ManifestSource<H> for &mut File
where
    H: HashAlgorithm,
{
    fn resolve(
        self,
        target: Option<ArtifactId<H>>,
    ) -> Result<InputManifest<H>, InputManifestError> {
        let mut s = String::new();
        self.read_to_string(&mut s)
            .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?;
        parse_input_manifest(&s, target)
    }
}

fn parse_input_manifest<H>(
    s: &str,
    target: Option<ArtifactId<H>>,
) -> Result<InputManifest<H>, InputManifestError>
where
    H: HashAlgorithm,
{
    let mut lines = s.lines();

    let first_line = lines
        .next()
        .ok_or(InputManifestError::ManifestMissingHeader)?;

    let parts = first_line.split(':').collect::<Vec<_>>();

    if parts.len() != 3 {
        return Err(InputManifestError::MissingHeaderParts);
    }

    // Panic Safety: we've already checked the length.
    let (gitoid, blob, hash_algorithm) = (parts[0], parts[1], parts[2]);

    if gitoid != "gitoid" {
        return Err(InputManifestError::MissingGitOidInHeader);
    }

    if blob != "blob" {
        return Err(InputManifestError::MissingObjectTypeInHeader);
    }

    if hash_algorithm != H::NAME {
        return Err(InputManifestError::WrongHashAlgorithm {
            expected: H::NAME.clone_as_boxstr(),
            got: hash_algorithm.clone_as_boxstr(),
        });
    }

    let mut relations = Vec::new();
    for line in lines {
        let relation = parse_relation::<H>(&line)?;
        relations.push(relation);
    }

    Ok(InputManifest {
        target,
        inputs: relations,
    })
}

/// Parse a single relation line.
fn parse_relation<H: HashAlgorithm>(input: &str) -> Result<Input<H>, InputManifestError> {
    let parts = input.split(' ').collect::<Vec<_>>();

    if parts.is_empty() {
        return Err(InputManifestError::MissingRelationParts(
            input.to_string().into_boxed_str(),
        ));
    }

    // Panic Safety: we've already checked the length.
    let (aid_hex, manifest_indicator, manifest_aid_hex) = (parts[0], parts.get(1), parts.get(2));

    let artifact =
        ArtifactId::<H>::from_str(&format!("gitoid:{}:{}:{}", Blob::NAME, H::NAME, aid_hex))?;

    let manifest = match (manifest_indicator, manifest_aid_hex) {
        (None, None) | (Some(_), None) | (None, Some(_)) => None,
        (Some(manifest_indicator), Some(manifest_aid_hex)) => {
            if *manifest_indicator != "manifest" {
                return Err(InputManifestError::MissingBomIndicatorInRelation);
            }

            let gitoid_url = &format!("gitoid:{}:{}:{}", Blob::NAME, H::NAME, manifest_aid_hex);

            ArtifactId::<H>::from_str(gitoid_url).ok()
        }
    };

    Ok(Input { artifact, manifest })
}
