#![allow(unused)]

use crate::{
    adg::AdgNode,
    error::ArtifactIdError,
    hash_algorithm::HashAlgorithm,
    storage::{Match, Storage},
    ArtifactId, Identify,
};
use petgraph::{algo::astar, graph::NodeIndex, Graph};
use std::ops::Not;

// Convenience type for our internal dependency graph.
pub(crate) type AdgInner<H> = Graph<AdgNode<H>, ()>;

/// The Artifact Dependency Graph.
#[derive(Debug)]
pub struct Adg<H>
where
    H: HashAlgorithm,
{
    /// The internal graph type.
    graph: AdgInner<H>,
    /// The target artifact the ADG is describing.
    target: NodeIndex,
}

impl<H> Adg<H>
where
    H: HashAlgorithm,
{
    // Actual construction is handled by the storage.
    pub(crate) fn new(graph: AdgInner<H>, target: NodeIndex) -> Self {
        Self { graph, target }
    }

    /// Get the target artifact for this ADG.
    pub fn target(&self) -> ArtifactId<H> {
        self.graph[self.target].artifact_id()
    }

    /// Get the path to a dependency, if present in the graph.
    pub fn dependency_path(&self, dep: ArtifactId<H>) -> Result<Vec<&AdgNode<H>>, ArtifactIdError> {
        let dep = dep.identify()?;

        let search = astar(
            &self.graph,
            // Start from the target.
            self.target,
            // End when we find the dependency.
            |n| self.graph[n].artifact_id() == dep,
            // Each edge has weight 1.
            |_| 1,
            // We don't do estimation.
            |_| 0,
        );

        match search {
            None => Ok(Vec::new()),
            Some((_, path)) => Ok(path.iter().map(|idx| &self.graph[*idx]).collect()),
        }
    }

    /// Check if the ADG's target artifact depends on a specific artifact.
    pub fn depends_on(&self, dep: ArtifactId<H>) -> Result<bool, ArtifactIdError> {
        let dep = dep.identify()?;

        // Rather than doing a graph algo, check if the dep is one of the nodes.
        Ok(self
            .graph
            .node_indices()
            .any(|idx| self.graph[idx].artifact_id() == dep))
    }
}
