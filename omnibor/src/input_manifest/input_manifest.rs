//! [`InputManifest`] type that represents build inputs for an artifact.

use {
    crate::{
        artifact_id::ArtifactId,
        error::InputManifestError,
        hash_algorithm::HashAlgorithm,
        object_type::{Blob, ObjectType},
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    std::{
        cmp::Ordering,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        fs::File,
        io::{BufRead, BufReader, Write},
        path::Path,
        str::FromStr,
    },
};

/// A manifest describing the inputs used to build an artifact.
///
/// The manifest is constructed with a specific target artifact in mind.
/// The rest of the manifest then describes relations to other artifacts.
/// Each relation can be thought of as describing edges between nodes
/// in an Artifact Dependency Graph.
///
/// Each relation encodes a kind which describes how the other artifact
/// relates to the target artifact.
///
/// Relations may additionally refer to the [`InputManifest`] of the
/// related artifact.
#[derive(PartialEq, Eq)]
pub struct InputManifest<H: HashAlgorithm> {
    /// The artifact the manifest is describing.
    ///
    /// A manifest without this is "detached" because we don't know
    /// what artifact it's describing.
    target: Option<ArtifactId<H>>,

    /// The relations recorded in the manifest.
    relations: Vec<InputManifestRelation<H>>,
}

impl<H: HashAlgorithm> InputManifest<H> {
    pub(crate) fn with_relations(
        relations: impl Iterator<Item = InputManifestRelation<H>>,
    ) -> Self {
        InputManifest {
            target: None,
            relations: relations.collect(),
        }
    }

    /// Get the ID of the artifact this manifest is describing.
    ///
    /// If the manifest does not have a target, it is a "detached" manifest.
    ///
    /// Detached manifests may still be usable if the target artifact was
    /// created in embedding mode, in which case it will carry the [`ArtifactId`]
    /// of its own input manifest in its contents.
    #[inline]
    pub fn target(&self) -> Option<ArtifactId<H>> {
        self.target
    }

    /// Identify if the manifest is a "detached" manifest.
    ///
    /// "Detached" manifests are ones without a target [`ArtifactId`].
    pub fn is_detached(&self) -> bool {
        self.target.is_none()
    }

    /// Set a new target.
    pub fn set_target(&mut self, target: Option<ArtifactId<H>>) -> &mut Self {
        self.target = target;
        self
    }

    /// Get the header used at the top of the [`InputManifest`].
    pub fn header(&self) -> String {
        format!("gitoid:{}:{}\n", Blob::NAME, H::NAME)
    }

    /// Get the relations inside an [`InputManifest`].
    #[inline]
    pub fn relations(&self) -> &[InputManifestRelation<H>] {
        &self.relations[..]
    }

    /// Construct an [`InputManifest`] from a file at a specified path.
    pub fn from_path(path: &Path) -> Result<Self, InputManifestError> {
        let file = BufReader::new(
            File::open(path)
                .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?,
        );
        let mut lines = file.lines();

        let first_line = lines
            .next()
            .ok_or(InputManifestError::ManifestMissingHeader)?
            .map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?;

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
            let line =
                line.map_err(|source| InputManifestError::FailedManifestRead(Box::new(source)))?;
            let relation = parse_relation::<H>(&line)?;
            relations.push(relation);
        }

        Ok(InputManifest {
            target: None,
            relations,
        })
    }

    /// Get the manifest as bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        // Per the spec, this prefix is present to substantially shorten
        // the metadata info that would otherwise be attached to all IDs in
        // a manifest if they were written in full form. Instead, only the
        // hex-encoded hashes are recorded elsewhere, because all the metadata
        // is identical in a manifest and only recorded once at the beginning.
        let _ = write!(bytes, "{}", self.header());

        for relation in &self.relations {
            let aid = relation.artifact;

            let _ = write!(bytes, "{}", aid.as_hex());

            if let Some(mid) = relation.manifest {
                let _ = write!(bytes, " manifest {}", mid.as_hex());
            }

            let _ = writeln!(bytes);
        }

        bytes
    }
}

impl<H: HashAlgorithm> Debug for InputManifest<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifest")
            .field("target", &self.target)
            .field("relations", &self.relations)
            .finish()
    }
}

impl<H: HashAlgorithm> Clone for InputManifest<H> {
    fn clone(&self) -> Self {
        InputManifest {
            target: self.target,
            relations: self.relations.clone(),
        }
    }
}

/// Parse a single relation line.
fn parse_relation<H: HashAlgorithm>(
    input: &str,
) -> Result<InputManifestRelation<H>, InputManifestError> {
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

    Ok(InputManifestRelation { artifact, manifest })
}

/// A single row in an [`InputManifest`].
#[derive(Copy)]
pub struct InputManifestRelation<H: HashAlgorithm> {
    /// The ID of the artifact itself.
    artifact: ArtifactId<H>,

    /// The ID of the manifest, if a manifest is present.
    manifest: Option<ArtifactId<H>>,
}

// We implement this ourselves instead of deriving it because
// the auto-derive logic will only conditionally derive it based
// on whether the `H` type parameter implements `Debug`, which
// isn't actually relevant in this case because we don't _really_
// store a value of type-`H`, we just use it for type-level
// programming.
impl<H: HashAlgorithm> Debug for InputManifestRelation<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Relation")
            .field("artifact", &self.artifact)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: HashAlgorithm> Display for InputManifestRelation<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.artifact.as_hex())?;

        if let Some(manifest) = self.manifest {
            write!(f, " manifest {}", manifest.as_hex())?;
        }

        writeln!(f)
    }
}

impl<H: HashAlgorithm> Clone for InputManifestRelation<H> {
    fn clone(&self) -> Self {
        InputManifestRelation {
            artifact: self.artifact,
            manifest: self.manifest,
        }
    }
}

impl<H: HashAlgorithm> PartialEq for InputManifestRelation<H> {
    fn eq(&self, other: &Self) -> bool {
        self.artifact.eq(&other.artifact) && self.manifest.eq(&other.manifest)
    }
}

impl<H: HashAlgorithm> Eq for InputManifestRelation<H> {}

impl<H: HashAlgorithm> PartialOrd for InputManifestRelation<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H: HashAlgorithm> Ord for InputManifestRelation<H> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.artifact.cmp(&other.artifact)
    }
}

impl<H: HashAlgorithm> InputManifestRelation<H> {
    pub(crate) fn new(
        artifact: ArtifactId<H>,
        manifest: Option<ArtifactId<H>>,
    ) -> InputManifestRelation<H> {
        InputManifestRelation { artifact, manifest }
    }

    /// Get the ID of the artifact.
    #[inline]
    pub fn artifact(&self) -> ArtifactId<H> {
        self.artifact
    }

    /// Get the manifest ID, if present.
    #[inline]
    pub fn manifest(&self) -> Option<ArtifactId<H>> {
        self.manifest
    }
}
