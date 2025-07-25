//! Store and retrieve `InputManifest`s.
//!
//! The "Store" is an interface for types that store and enable querying of
//! Input Manifests. They exist in particular to support things like filling
//! in manifest information for build inputs during Input Manifest construction,
//! and to ensure (in the case of `FileSystemStorage`) that Input Manifests
//! are persisted to disk in a manner consistent with the OmniBOR specification.
//!
//! [__See Storage documentation for more info.__][idx]
//!
//! [idx]: crate#storage

pub(crate) mod file_system_storage;
pub(crate) mod in_memory_storage;
pub(crate) mod query;
#[cfg(test)]
mod test;

use petgraph::{graph::NodeIndex, Graph};

pub use crate::storage::file_system_storage::FileSystemStorage;
pub use crate::storage::in_memory_storage::InMemoryStorage;
pub use crate::storage::query::Match;

use crate::{
    adg::{AdgNode, DepGraph, DepGraphInner},
    artifact_id::ArtifactId,
    error::InputManifestError,
    hash_algorithm::HashAlgorithm,
    input_manifest::InputManifest,
    Identify,
};

/// Represents the interface for storing and querying manifests.
pub trait Storage<H: HashAlgorithm> {
    /// Write a manifest to the storage.
    ///
    /// If the manifest has a target attached, update any indices.
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError>;

    /// Get all manifests from the storage.
    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError>;

    /// Get a manifest by matching on the criteria.
    ///
    /// Returns `Ok(None)` if no match was found. Returns the manifest if found.
    /// Returns an error otherwise.
    fn get_manifest<I>(
        &self,
        matcher: Match<H, I>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>
    where
        I: Identify<H>;

    /// Update the manifest file to reflect the target ID.
    fn update_manifest_target<I1, I2>(
        &mut self,
        manifest_aid: I1,
        target_aid: I2,
    ) -> Result<(), InputManifestError>
    where
        I1: Identify<H>,
        I2: Identify<H>;

    /// Remove the manifest for the target artifact.
    ///
    /// Returns the manifest to the caller, if found. Returns `Ok(None)` if no
    /// errors were encountered but the manifest was not found in storage.
    /// Returns errors otherwise.
    fn remove_manifest<I>(
        &mut self,
        matcher: Match<H, I>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>
    where
        I: Identify<H>;

    /// Get the Artifact Dependency Graph of the target.
    fn get_adg<I>(&self, target_aid: I) -> Result<DepGraph<H>, InputManifestError>
    where
        I: Identify<H>,
    {
        let target_aid = target_aid.identify()?;
        let graph = build_graph(target_aid, self)?;
        Ok(DepGraph::from_graph(graph))
    }
}

impl<H: HashAlgorithm, S: Storage<H>> Storage<H> for &mut S {
    fn write_manifest(
        &mut self,
        manifest: &InputManifest<H>,
    ) -> Result<ArtifactId<H>, InputManifestError> {
        (**self).write_manifest(manifest)
    }

    fn get_manifests(&self) -> Result<Vec<InputManifest<H>>, InputManifestError> {
        (**self).get_manifests()
    }

    fn get_manifest<I>(
        &self,
        matcher: Match<H, I>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>
    where
        I: Identify<H>,
    {
        (**self).get_manifest(matcher)
    }

    fn update_manifest_target<I1, I2>(
        &mut self,
        manifest_aid: I1,
        target_aid: I2,
    ) -> Result<(), InputManifestError>
    where
        I1: Identify<H>,
        I2: Identify<H>,
    {
        (**self).update_manifest_target(manifest_aid, target_aid)
    }

    fn remove_manifest<I>(
        &mut self,
        matcher: Match<H, I>,
    ) -> Result<Option<InputManifest<H>>, InputManifestError>
    where
        I: Identify<H>,
    {
        (**self).remove_manifest(matcher)
    }
}

/// Build the Artifact Dependency Graph.
fn build_graph<H, S>(
    target_aid: ArtifactId<H>,
    storage: &S,
) -> Result<DepGraphInner<H>, InputManifestError>
where
    H: HashAlgorithm,
    S: Storage<H> + ?Sized,
{
    let mut graph = Graph::new();
    populate_graph(&mut graph, target_aid, None, storage)?;
    Ok(graph)
}

/// Build one layer of the ADG, recursing down for further layers.
fn populate_graph<H, S>(
    graph: &mut DepGraphInner<H>,
    target_aid: ArtifactId<H>,
    parent_idx: Option<NodeIndex>,
    storage: &S,
) -> Result<(), InputManifestError>
where
    H: HashAlgorithm,
    S: Storage<H> + ?Sized,
{
    // Add current node to the graph.
    let self_idx = graph.add_node(AdgNode { id: target_aid });

    // If there's a parent, add an edge from the parent to the current node.
    if let Some(parent_idx) = parent_idx {
        let _ = graph.add_edge(parent_idx, self_idx, ());
    }

    // If there's a manifest for current node, recurse for each child.
    if let Some(manifest) = storage.get_manifest(Match::target(target_aid))? {
        for input in manifest.inputs() {
            populate_graph(graph, input.artifact(), Some(self_idx), storage)?;
        }
    }

    Ok(())
}
