//! Record of software build inputs by Artifact ID.

mod input_manifest_builder;
mod manifest_source;
mod manifest_source_async;

use std::io::BufWriter;

pub use input_manifest_builder::InputManifestBuilder;
pub use manifest_source::ManifestSource;
pub use manifest_source_async::ManifestSourceAsync;

use crate::{hash_algorithm::Sha256, IdentifyAsync};

use {
    crate::{
        artifact_id::ArtifactId,
        error::InputManifestError,
        hash_algorithm::HashAlgorithm,
        object_type::{Blob, ObjectType},
        storage::Storage,
        Identify,
    },
    std::{
        cmp::Ordering,
        fmt::{Debug, Display, Formatter, Result as FmtResult},
        io::Write,
    },
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
#[derive(PartialEq, Eq)]
pub struct InputManifest<H>
where
    H: HashAlgorithm,
{
    /// The artifact the manifest is describing.
    ///
    /// A manifest without this is "detached" because we don't know
    /// what artifact it's describing.
    target: Option<ArtifactId<H>>,

    /// The inputs recorded in the manifest.
    inputs: Vec<Input<H>>,
}

impl InputManifest<Sha256> {
    /// Load the input manifest at the path with the SHA-256 hash function.
    pub fn sha256<M, I>(source: M, target: I) -> Result<Self, InputManifestError>
    where
        M: ManifestSource<Sha256>,
        I: Identify<Sha256>,
    {
        InputManifest::load(source, target)
    }

    /// Load the input manifest at the path with the SHA-256 hash function, detached.
    pub fn sha256_detached<M>(source: M) -> Result<Self, InputManifestError>
    where
        M: ManifestSource<Sha256>,
    {
        InputManifest::load_detached(source)
    }

    /// Load the input manifest at the path with the SHA-256 hash function asynchronously.
    pub async fn sha256_async<M, I>(source: M, target: I) -> Result<Self, InputManifestError>
    where
        M: ManifestSourceAsync<Sha256>,
        I: IdentifyAsync<Sha256>,
    {
        let target = target.identify_async().await?;
        InputManifest::load_async(source, target).await
    }

    /// Load the input manifest at the path with the SHA-256 hash function asynchronously, detached.
    pub async fn sha256_detached_async<M>(source: M) -> Result<Self, InputManifestError>
    where
        M: ManifestSourceAsync<Sha256>,
    {
        InputManifest::load_detached_async(source).await
    }
}

impl<H: HashAlgorithm> InputManifest<H> {
    /// Get a builder for [`InputManifest`]s.
    pub fn builder<S>(storage: S) -> InputManifestBuilder<H, S>
    where
        S: Storage<H>,
    {
        InputManifestBuilder::new(storage)
    }

    pub(crate) fn with_relations(
        relations: impl Iterator<Item = Input<H>>,
        target: Option<ArtifactId<H>>,
    ) -> Self {
        InputManifest {
            target,
            inputs: relations.collect(),
        }
    }

    /// Construct an [`InputManifest`] from a source.
    pub fn load<M, I>(source: M, target: I) -> Result<Self, InputManifestError>
    where
        M: ManifestSource<H>,
        I: Identify<H>,
    {
        let target = target.identify()?;
        source.resolve(Some(target))
    }

    /// Construct an [`InputManifest`] from a source, without a target.
    pub fn load_detached<M>(source: M) -> Result<Self, InputManifestError>
    where
        M: ManifestSource<H>,
    {
        source.resolve(None)
    }

    /// Construct an [`InputManifest`] from a source, asynchronously.
    pub async fn load_async<M, I>(source: M, target: I) -> Result<Self, InputManifestError>
    where
        M: ManifestSourceAsync<H>,
        I: IdentifyAsync<H>,
    {
        let target = target.identify_async().await?;
        source.resolve_async(Some(target)).await
    }

    /// Construct an [`InputManifest`] from a source, asynchronously, without a target.
    pub async fn load_detached_async<M>(source: M) -> Result<Self, InputManifestError>
    where
        M: ManifestSourceAsync<H>,
    {
        source.resolve_async(None).await
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

    /// Set a new target.
    pub(crate) fn set_target(&mut self, target: Option<ArtifactId<H>>) -> &mut Self {
        self.target = target;
        self
    }

    /// Identify if the manifest is a "detached" manifest.
    ///
    /// "Detached" manifests are ones without a target [`ArtifactId`].
    #[inline]
    pub fn is_detached(&self) -> bool {
        self.target.is_none()
    }

    /// Get the header used at the top of the [`InputManifest`].
    #[inline]
    pub fn header(&self) -> String {
        format!("gitoid:{}:{}\n", Blob::NAME, H::NAME)
    }

    /// Get the inputs recorded inside an [`InputManifest`].
    #[inline]
    pub fn inputs(&self) -> &[Input<H>] {
        &self.inputs[..]
    }

    /// Check if the manifest contains the given input.
    #[inline]
    pub fn contains_artifact<I>(&self, artifact: I) -> Result<bool, InputManifestError>
    where
        I: Identify<H>,
    {
        let artifact_id = ArtifactId::new(artifact)?;
        Ok(self.inputs().iter().any(|rel| rel.artifact == artifact_id))
    }

    /// Get the manifest as bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut writer = BufWriter::new(Vec::new());
        // PANIC SAFETY: Writing into a `Vec` will always succeed.
        self.write(&mut writer).unwrap();
        writer.into_inner().unwrap()
    }

    /// Write the input manifest out to a buffered writer.
    ///
    /// # Why `BufWriter<dyn Write>`?
    ///
    /// This avoids monomorphizing the writer function for every concrete sink,
    /// and ensures that writers are always buffered.
    pub fn write(&self, out: &mut BufWriter<dyn Write>) -> std::io::Result<()> {
        // Per the spec, this prefix is present to substantially shorten
        // the metadata info that would otherwise be attached to all IDs in
        // a manifest if they were written in full form. Instead, only the
        // hex-encoded hashes are recorded elsewhere, because all the metadata
        // is identical in a manifest and only recorded once at the beginning.
        write!(out, "{}", self.header())?;

        for relation in &self.inputs {
            write!(out, "{}", relation.artifact.as_hex())?;

            if let Some(manifest_aid) = relation.manifest {
                write!(out, " manifest {}", manifest_aid.as_hex())?;
            }

            writeln!(out)?;
        }

        Ok(())
    }
}

impl<H: HashAlgorithm> Debug for InputManifest<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("InputManifest")
            .field("target", &self.target)
            .field("relations", &self.inputs)
            .finish()
    }
}

impl<H: HashAlgorithm> Display for InputManifest<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let str = String::from_utf8(self.as_bytes().to_vec()).unwrap();
        write!(f, "{str}")
    }
}

impl<H: HashAlgorithm> Clone for InputManifest<H> {
    fn clone(&self) -> Self {
        InputManifest {
            target: self.target,
            inputs: self.inputs.clone(),
        }
    }
}

/// A single input recorded in an `InputManifest`.
#[derive(Copy)]
pub struct Input<H>
where
    H: HashAlgorithm,
{
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
impl<H: HashAlgorithm> Debug for Input<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Relation")
            .field("artifact", &self.artifact)
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl<H: HashAlgorithm> Display for Input<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.artifact.as_hex())?;

        if let Some(manifest) = self.manifest {
            write!(f, " manifest {}", manifest.as_hex())?;
        }

        writeln!(f)
    }
}

impl<H: HashAlgorithm> Clone for Input<H> {
    fn clone(&self) -> Self {
        Input {
            artifact: self.artifact,
            manifest: self.manifest,
        }
    }
}

impl<H: HashAlgorithm> PartialEq for Input<H> {
    fn eq(&self, other: &Self) -> bool {
        self.artifact.eq(&other.artifact) && self.manifest.eq(&other.manifest)
    }
}

impl<H: HashAlgorithm> Eq for Input<H> {}

impl<H: HashAlgorithm> PartialOrd for Input<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H: HashAlgorithm> Ord for Input<H> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.artifact.cmp(&other.artifact)
    }
}

impl<H: HashAlgorithm> Input<H> {
    pub(crate) fn new(artifact: ArtifactId<H>, manifest: Option<ArtifactId<H>>) -> Input<H> {
        Input { artifact, manifest }
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
