#![allow(unused)]

use crate::{
    error::{ArtifactIdError, InputManifestError},
    hash_algorithm::HashAlgorithm,
    storage::{Match, Storage},
    ArtifactId, Identify,
};
use petgraph::{algo::astar, graph::NodeIndex, Graph};

/// A node in the Artifact Dependency Graph.
#[derive(Debug, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct AdgNode<H>
where
    H: HashAlgorithm,
{
    /// The Artifact ID of the node's artifact.
    artifact_id: ArtifactId<H>,
}

impl<H> AdgNode<H>
where
    H: HashAlgorithm,
{
    pub(crate) fn new(artifact_id: ArtifactId<H>) -> Self {
        AdgNode { artifact_id }
    }

    /// Get the Artifact ID for the node.
    pub fn artifact_id(&self) -> ArtifactId<H> {
        self.artifact_id
    }
}

impl<H> Clone for AdgNode<H>
where
    H: HashAlgorithm,
{
    fn clone(&self) -> Self {
        AdgNode {
            artifact_id: self.artifact_id,
        }
    }
}
