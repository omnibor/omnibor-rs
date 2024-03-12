//! [`InputManifest`] type that represents build inputs for an artifact.

use crate::{ArtifactId, SupportedHash};
use gitoid::{Blob, HashAlgorithm, ObjectType};
use std::fmt::{Display, Formatter, Result as FmtResult};

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
#[derive(Debug)]
pub struct InputManifest<H: SupportedHash> {
    /// The artifact the manifest is describing.
    target: ArtifactId<H>,

    /// The relations recorded in the manifest.
    relations: Vec<Relation<H>>,
}

impl<H: SupportedHash> InputManifest<H> {
    /// Get the ID of the artifact this manifest is describing.
    pub fn target(&self) -> ArtifactId<H> {
        self.target
    }

    /// Get the relations inside an [`InputManifest`].
    pub fn relations(&self) -> &[Relation<H>] {
        &self.relations[..]
    }
}

impl<H: SupportedHash> Display for InputManifest<H> {
    // The OmniBOR spec actually specifies using _only_ `\n` for newlines,
    // regardless of the host platform specification, so Clippy's recommendation
    // here would cause our code to violate the spec.
    #[allow(clippy::write_with_newline)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "gitoid:{}:{}\n", Blob::NAME, H::HashAlgorithm::NAME)?;

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
#[derive(Debug)]
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
    pub fn kind(&self) -> RelationKind {
        self.kind
    }

    /// Get the ID of the artifact.
    pub fn artifact_id(&self) -> ArtifactId<H> {
        self.artifact
    }

    /// Get the manifest ID, if present.
    pub fn manifest_id(&self) -> Option<ArtifactId<H>> {
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
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RelationKind::InputFor => write!(f, "input"),
            RelationKind::BuiltBy => write!(f, "built-by"),
        }
    }
}
