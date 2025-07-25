//! Operations and types for handling Artifact Dependency Graphs (ADGs).
#![allow(unused)]

use crate::{
    error::InputManifestError,
    hash_algorithm::HashAlgorithm,
    storage::{Match, Storage},
    ArtifactId,
};
use petgraph::{graph::NodeIndex, Graph};

// Convenience type for our internal dependency graph.
pub(crate) type DepGraphInner<H> = Graph<AdgNode<H>, ()>;

/// The Artifact Dependency Graph.
#[derive(Debug)]
pub struct DepGraph<H>
where
    H: HashAlgorithm,
{
    graph: DepGraphInner<H>,
}

impl<H> DepGraph<H>
where
    H: HashAlgorithm,
{
    pub(crate) fn from_graph(graph: DepGraphInner<H>) -> Self {
        Self { graph }
    }
}

/// A node in the Artifact Dependency Graph.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct AdgNode<H>
where
    H: HashAlgorithm,
{
    /// The Artifact ID of the node's artifact.
    pub id: ArtifactId<H>,
}
