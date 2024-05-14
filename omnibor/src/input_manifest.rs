//! [`InputManifest`] type that represents build inputs for an artifact.

use crate::{ArtifactId, Error, Result, SupportedHash};
use gitoid::{Blob, HashAlgorithm, ObjectType};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputManifest<H: SupportedHash> {
    /// The artifact the manifest is describing.
    target: ArtifactId<H>,

    /// The relations recorded in the manifest.
    relations: Vec<Relation<H>>,
}

impl<H: SupportedHash> InputManifest<H> {
    /// Get the ID of the artifact this manifest is describing.
    #[inline]
    pub fn target(&self) -> ArtifactId<H> {
        self.target
    }

    /// Get the relations inside an [`InputManifest`].
    #[inline]
    pub fn relations(&self) -> &[Relation<H>] {
        &self.relations[..]
    }

    /// Construct an [`InputManifest`] from a file.
    pub fn from_file(file: &File) -> Result<Self> {
        let file = BufReader::new(file);
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

        // Then parse the "manifest-for" line.
        let relation = parse_relation::<H>(
            &lines
                .next()
                .ok_or_else(|| Error::MissingManifestForRelation)?
                .map_err(Error::FailedManifestRead)?,
        )?;

        if relation.kind() != RelationKind::ManifestFor {
            return Err(Error::MissingManifestForRelation);
        }

        let target = relation.artifact();

        let mut relations = Vec::new();

        for line in lines {
            let line = line.map_err(Error::FailedManifestRead)?;
            let relation = parse_relation::<H>(&line)?;
            relations.push(relation);
        }

        Ok(InputManifest { target, relations })
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

impl<H: SupportedHash> Display for InputManifest<H> {
    // The OmniBOR spec actually specifies using _only_ `\n` for newlines,
    // regardless of the host platform specification, so Clippy's recommendation
    // here would cause our code to violate the spec.
    #[allow(clippy::write_with_newline)]
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        // Per the spec, this prefix is present to substantially shorten
        // the metadata info that would otherwise be attached to all IDs in
        // a manifest if they were written in full form. Instead, only the
        // hex-encoded hashes are recorded elsewhere, because all the metadata
        // is identical in a manifest and only recorded once at the beginning.
        write!(f, "gitoid:{}:{}\n", Blob::NAME, H::HashAlgorithm::NAME)?;

        // Write a single entry for the artifact being described.
        write!(
            f,
            "{} {} {}",
            RelationKind::ManifestFor,
            self.target.object_type(),
            self.target.as_hex()
        )?;

        for relation in &self.relations {
            let aid = relation.artifact;

            write!(
                f,
                "{} {} {}",
                relation.kind,
                aid.object_type(),
                aid.as_hex()
            )?;

            if let Some(mid) = relation.manifest {
                write!(f, " bom {}", mid.as_hex())?;
            }

            write!(f, "\n")?;
        }

        Ok(())
    }
}

/// A single input artifact represented in a [`InputManifest`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Relation<H: SupportedHash> {
    /// The kind of relation being represented.
    kind: RelationKind,

    /// The ID of the artifact itself.
    artifact: ArtifactId<H>,

    /// The ID of the manifest, if a manifest is present.
    manifest: Option<ArtifactId<H>>,
}

impl<H: SupportedHash> Relation<H> {
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

/// Describes the relationship between a manifest's target artifact and other artifacts.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelationKind {
    /// Is a build input for the target artifact.
    InputFor,

    /// Is a tool used to build the target artifact.
    BuiltBy,

    /// Is the artifact being described by this manifest.
    ManifestFor,
}

impl Display for RelationKind {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RelationKind::InputFor => write!(f, "input"),
            RelationKind::BuiltBy => write!(f, "built-by"),
            RelationKind::ManifestFor => write!(f, "manifest-for"),
        }
    }
}

impl FromStr for RelationKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "input" => Ok(RelationKind::InputFor),
            "built-by" => Ok(RelationKind::BuiltBy),
            "manifest-for" => Ok(RelationKind::ManifestFor),
            _ => Err(Error::InvalidRelationKind(s.to_owned())),
        }
    }
}
