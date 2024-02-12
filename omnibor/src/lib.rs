//! OmniBOR in Rust.

use gitoid::hash::Sha256;
use gitoid::object::Blob;
use gitoid::GitOid;

/// An OmniBOR Artifact Identifier.
pub type ArtifactId = GitOid<Sha256, Blob>;
