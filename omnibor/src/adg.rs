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
type DepGraphInner<H> = Graph<AdgNode<H>, ()>;

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
    /// Get the ADG for a target.
    pub fn for_target<S>(target_aid: ArtifactId<H>, storage: &S) -> Result<Self, InputManifestError>
    where
        S: Storage<H>,
    {
        let mut graph = Graph::new();
        populate_graph(&mut graph, target_aid, None, storage)?;
        Ok(Self { graph })
    }
}

fn populate_graph<H, S>(
    graph: &mut DepGraphInner<H>,
    target_aid: ArtifactId<H>,
    parent_idx: Option<NodeIndex>,
    storage: &S,
) -> Result<(), InputManifestError>
where
    H: HashAlgorithm,
    S: Storage<H>,
{
    let self_idx = insert_self(graph, target_aid, parent_idx);

    for input_aid in get_inputs(target_aid, storage)? {
        populate_graph(graph, input_aid, Some(self_idx), storage)?;
    }

    Ok(())
}

fn insert_self<H>(
    graph: &mut DepGraphInner<H>,
    target_aid: ArtifactId<H>,
    parent_idx: Option<NodeIndex>,
) -> NodeIndex
where
    H: HashAlgorithm,
{
    let self_idx = graph.add_node(AdgNode { id: target_aid });

    if let Some(parent_idx) = parent_idx {
        let _ = graph.add_edge(parent_idx, self_idx, ());
    }

    self_idx
}

fn get_inputs<H, S>(
    target_aid: ArtifactId<H>,
    storage: &S,
) -> Result<Vec<ArtifactId<H>>, InputManifestError>
where
    H: HashAlgorithm,
    S: Storage<H>,
{
    storage
        .get_manifest(Match::Target(target_aid))
        .map(|manifest| match manifest {
            // If we have a manifest, get all the input artifact IDs.
            Some(manifest) => manifest
                .inputs()
                .into_iter()
                .map(|input| input.artifact())
                .collect(),
            // If there's no manifest found, don't get inputs.
            None => Vec::new(),
        })
}

/// A node in the Artifact Dependency Graph.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct AdgNode<H>
where
    H: HashAlgorithm,
{
    id: ArtifactId<H>,
}
