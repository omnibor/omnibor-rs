//! [`InputManifest`] type that represents build inputs for an artifact.

use crate::hashes::SupportedHash;
use crate::ArtifactId;
use crate::Error;
use crate::Result;
use gitoid::Blob;
use gitoid::HashAlgorithm;
use gitoid::ObjectType;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

/*
Input Manifest builder...

The user is creating an artifact.
As the artifact is being created, we have a builder into which we log the artifact IDs (and detect any manifests if present)
of everything used to make the artifact, along with the type of relation.
When the artifact creation is done, we finalize the manifest, creating an `InputManifest` with no target.
The Input Manifest doesn't refer to the artifact in its contents.
Then we update the target if embedding mode is on.
Then we write the input manifest to disk with the target in its file name.

This is more than the builder pattern lol.
*/

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
pub struct InputManifest<H: SupportedHash> {
    /// The artifact the manifest is describing.
    ///
    /// A manifest without this is "detached" because we don't know
    /// what artifact it's describing.
    target: Option<ArtifactId<H>>,

    /// The relations recorded in the manifest.
    relations: Vec<Relation<H>>,
}

impl<H: SupportedHash> InputManifest<H> {
    pub(crate) fn with_relations(relations: &[Relation<H>]) -> Self {
        InputManifest {
            target: None,
            relations: relations.to_vec(),
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
    #[allow(unused)]
    pub(crate) fn set_target(&mut self, target: ArtifactId<H>) -> &mut Self {
        self.target = Some(target);
        self
    }

    /// Get the relations inside an [`InputManifest`].
    #[inline]
    pub fn relations(&self) -> &[Relation<H>] {
        &self.relations[..]
    }

    /// Construct an [`InputManifest`] from a file at a specified path.
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = BufReader::new(File::open(path)?);
        let mut lines = file.lines();

        let first_line = lines
            .next()
            .ok_or(Error::ManifestMissingHeader)?
            .map_err(Error::FailedManifestRead)?;

        let parts = first_line.split(':').collect::<Vec<_>>();

        if parts.len() != 3 {
            return Err(Error::MissingHeaderParts);
        }

        // Panic Safety: we've already checked the length.
        let (gitoid, blob, hash_algorithm) = (parts[0], parts[1], parts[2]);

        if gitoid != "gitoid" {
            return Err(Error::MissingGitOidInHeader);
        }

        if blob != "blob" {
            return Err(Error::MissingObjectTypeInHeader);
        }

        if hash_algorithm != H::HashAlgorithm::NAME {
            return Err(Error::WrongHashAlgorithm {
                expected: H::HashAlgorithm::NAME,
                got: hash_algorithm.to_owned(),
            });
        }

        let target = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| ArtifactId::<H>::try_from_safe_name(s).ok());

        let mut relations = Vec::new();
        for line in lines {
            let line = line.map_err(Error::FailedManifestRead)?;
            let relation = parse_relation::<H>(&line)?;
            relations.push(relation);
        }

        Ok(InputManifest { target, relations })
    }

    /// Write the manifest out at the given path.
    #[allow(clippy::write_with_newline)]
    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<()> {
        // Per the spec, this prefix is present to substantially shorten
        // the metadata info that would otherwise be attached to all IDs in
        // a manifest if they were written in full form. Instead, only the
        // hex-encoded hashes are recorded elsewhere, because all the metadata
        // is identical in a manifest and only recorded once at the beginning.
        write!(writer, "gitoid:{}:{}\n", Blob::NAME, H::HashAlgorithm::NAME)?;

        for relation in &self.relations {
            let aid = relation.artifact;

            write!(
                writer,
                "{} {} {}",
                relation.kind,
                aid.object_type(),
                aid.as_hex()
            )?;

            if let Some(mid) = relation.manifest {
                write!(writer, " bom {}", mid.as_hex())?;
            }

            write!(writer, "\n")?;
        }

        Ok(())
    }
}

impl<H: SupportedHash> Debug for InputManifest<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifest")
            .field("target", &self.target)
            .field("relations", &self.relations)
            .finish()
    }
}

impl<H: SupportedHash> Clone for InputManifest<H> {
    fn clone(&self) -> Self {
        InputManifest {
            target: self.target,
            relations: self.relations.clone(),
        }
    }
}

/// Parse a single relation line.
fn parse_relation<H: SupportedHash>(input: &str) -> Result<Relation<H>> {
    let parts = input.split(' ').collect::<Vec<_>>();

    if parts.len() < 3 {
        return Err(Error::MissingRelationParts);
    }

    // Panic Safety: we've already checked the length.
    let (kind, object_type, aid_hex, bom_indicator, manifest_aid_hex) =
        (parts[0], parts[1], parts[2], parts.get(3), parts.get(4));

    if object_type != "blob" {
        return Err(Error::MissingObjectTypeInRelation);
    }

    let kind = RelationKind::from_str(kind)?;

    let artifact = ArtifactId::<H>::from_str(&format!(
        "gitoid:{}:{}:{}",
        object_type,
        H::HashAlgorithm::NAME,
        aid_hex
    ))?;

    let manifest = {
        if let (Some(bom_indicator), Some(manifest_aid_hex)) = (bom_indicator, manifest_aid_hex) {
            if *bom_indicator != "bom" {
                return Err(Error::MissingBomIndicatorInRelation);
            }

            match ArtifactId::<H>::from_str(&format!(
                "gitoid:{}:{}:{}",
                object_type,
                H::HashAlgorithm::NAME,
                manifest_aid_hex
            )) {
                Ok(aid) => Some(aid),
                Err(_) => None,
            }
        } else {
            None
        }
    };

    Ok(Relation {
        kind,
        artifact,
        manifest,
    })
}

/// A single input artifact represented in a [`InputManifest`].
#[derive(Copy, PartialEq, Eq)]
pub struct Relation<H: SupportedHash> {
    /// The kind of relation being represented.
    kind: RelationKind,

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
impl<H: SupportedHash> Debug for Relation<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Relation")
            .field("kind", &self.kind)
            .field("artifact", &self.artifact)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: SupportedHash> Clone for Relation<H> {
    fn clone(&self) -> Self {
        Relation {
            kind: self.kind,
            artifact: self.artifact,
            manifest: self.manifest,
        }
    }
}

impl<H: SupportedHash> Relation<H> {
    pub(crate) fn new(
        kind: RelationKind,
        artifact: ArtifactId<H>,
        manifest: Option<ArtifactId<H>>,
    ) -> Relation<H> {
        Relation {
            kind,
            artifact,
            manifest,
        }
    }

    /// Get the kind of relation being described.
    #[inline]
    pub fn kind(&self) -> RelationKind {
        self.kind
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

/// Describes the relationship between an [`InputManifest`]'s target artifact and other artifacts.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelationKind {
    /// Is a build input for the target artifact.
    Input,

    /// Is a tool used to build the target artifact.
    BuiltBy,
}

impl Display for RelationKind {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RelationKind::Input => write!(f, "input"),
            RelationKind::BuiltBy => write!(f, "built-by"),
        }
    }
}

impl FromStr for RelationKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "input" => Ok(RelationKind::Input),
            "built-by" => Ok(RelationKind::BuiltBy),
            _ => Err(Error::InvalidRelationKind(s.to_owned())),
        }
    }
}
