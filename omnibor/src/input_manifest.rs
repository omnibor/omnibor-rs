//! [`InputManifest`] type that represents build inputs for an artifact.

use crate::{ArtifactId, SupportedHash};
use gitoid::{Blob, HashAlgorithm, ObjectType};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A manifest describing the inputs used to build an artifact.
#[derive(Debug)]
pub struct InputManifest<H: SupportedHash> {
    ids: Vec<InputRecord<H>>,
}

impl<H: SupportedHash> InputManifest<H> {
    /// Get the IDs contained inside an [`InputManifest`].
    pub fn ids(&self) -> &[InputRecord<H>] {
        &self.ids[..]
    }
}

impl<H: SupportedHash> Display for InputManifest<H> {
    // The OmniBOR spec actually specifies using _only_ `\n` for newlines,
    // regardless of the host platform specification, so Clippy's recommendation
    // here would cause our code to violate the spec.
    #[allow(clippy::write_with_newline)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "gitoid:{}:{}\n", Blob::NAME, H::HashAlgorithm::NAME)?;

        for id in &self.ids {
            write!(f, "{}", id)?;
        }

        Ok(())
    }
}

/// A single input artifact represented in a [`BuildManifest`].
#[derive(Debug)]
pub struct InputRecord<H: SupportedHash> {
    // The ID of the artifact itself.
    artifact_id: ArtifactId<H>,

    // The ID of the manifest, if a manifest is present.
    manifest_id: Option<ArtifactId<H>>,
}

impl<H: SupportedHash> InputRecord<H> {
    /// Get the ID of the artifact.
    pub fn artifact_id(&self) -> ArtifactId<H> {
        self.artifact_id
    }

    /// Get the manifest ID, if present.
    pub fn manifest_id(&self) -> Option<ArtifactId<H>> {
        self.manifest_id
    }
}

impl<H: SupportedHash> Display for InputRecord<H> {
    // The OmniBOR spec actually specifies using _only_ `\n` for newlines,
    // regardless of the host platform specification, so Clippy's recommendation
    // here would cause our code to violate the spec.
    #[allow(clippy::write_with_newline)]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let aid = self.artifact_id;

        match self.manifest_id {
            Some(mid) => write!(
                f,
                "{} {} bom {}\n",
                aid.object_type(),
                aid.as_hex(),
                mid.as_hex()
            ),
            None => write!(f, "{} {}\n", aid.object_type(), aid.as_hex()),
        }
    }
}
