//! Control whether an `InputManifest`'s `ArtifactId` is stored in the target artifact.
//!
//! "Embedding" is one of the core operations in OmniBOR. When an Input Manifest
//! is created, its Artifact ID can be embedded in the "target artifact" (the
//! artifact whose build inputs the Input Manifest is describing). If embedding
//! is done, then the Artifact ID of the target artifact will now depend on
//! the Input Manifest, meaning that any change in the Input Manifest's contents
//! caused by a change in build dependencies will cause a change in the
//! Artifact ID of the target artifact.
//!
//! This is how OmniBOR produces a Merkle-tree-like structure for tracking
//! fine-grained build dependencies (individual files, including intermediate
//! files, used to build things like binaries).
//!
//! __Embedding is not required, but it is highly recommended.__ An Input
//! Manifest without an associated target is considered "detached," and is
//! generally not going to be very useful unless it's augmented with knowledge
//! of the target artifact.
//!
//! # Embedding Options
//!
//! This module contains three key structs:
//!
//! - `NoEmbed`: Do not perform embedding.
//! - `AutoEmbed`: Automatically infer the filetype of the target artifact and
//!   attempt to embed in it.
//! - `CustomEmbed`: The user provides a function for doing embedding.
//!
//! `AutoEmbed` is only available if the `infer-filetypes` feature is turned on.
//!
//! If `CustomEmbed` is selected, the function takes the path to the target
//! artifact, plus a [`CustomEmbedProvider`], which provides methods for getting
//! the embedding data as bytes or a UTF-8 string.
//!
//! [__See the main documentation on embedding for more information.__][idx]
//!
//! [idx]: crate#embedding

#[cfg(feature = "infer-filetypes")]
pub(crate) mod auto_embed;
pub(crate) mod custom_embed_provider;

use crate::{error::InputManifestError, hash_algorithm::HashAlgorithm, util::sealed::Sealed};
use std::{marker::PhantomData, path::Path};

pub use crate::embed::custom_embed_provider::CustomEmbedProvider;

#[cfg(feature = "infer-filetypes")]
use crate::embed::auto_embed::embed_manifest_in_target;

/// Defines how embedding should be handled in target artifacts.
pub trait Embed<H>: Sealed
where
    H: HashAlgorithm,
{
    /// Attempt to embed the manifest's Artifact ID in the target artifact.
    ///
    /// Returns None if no embedding was attempted. Otherwise returns a Result
    /// to indicate success or failure.
    fn try_embed(
        &self,
        target_path: &Path,
        embed_provider: CustomEmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>>;

    /// Indicates if the embedder will attempt to embed.
    fn will_embed(&self) -> bool {
        true
    }
}

/// Do not embed in the target file.
#[derive(Debug, Copy, Clone)]
pub struct NoEmbed;

impl Sealed for NoEmbed {}

impl<H: HashAlgorithm> Embed<H> for NoEmbed {
    // Do nothing, as we're not actually embedding.
    fn try_embed(
        &self,
        _target_path: &Path,
        _embed_provider: CustomEmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>> {
        None
    }

    fn will_embed(&self) -> bool {
        false
    }
}

#[cfg(feature = "infer-filetypes")]
/// Automatically infer the filetype of the target file, and attempt to embed.
#[derive(Debug, Copy, Clone)]
pub struct AutoEmbed;

#[cfg(feature = "infer-filetypes")]
impl Sealed for AutoEmbed {}

#[cfg(feature = "infer-filetypes")]
impl<H: HashAlgorithm> Embed<H> for AutoEmbed {
    fn try_embed(
        &self,
        target_path: &Path,
        embed_provider: CustomEmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>> {
        Some(embed_manifest_in_target(target_path, embed_provider))
    }
}

/// A custom embedding function.
pub struct CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, CustomEmbedProvider<H>) -> Result<(), InputManifestError>,
{
    op: F,
    _phantom: PhantomData<H>,
}

impl<H, F> CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, CustomEmbedProvider<H>) -> Result<(), InputManifestError>,
{
    /// Construct a new custom embedder.
    pub fn new(op: F) -> Self {
        CustomEmbed {
            op,
            _phantom: PhantomData,
        }
    }
}

impl<H, F> Sealed for CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, CustomEmbedProvider<H>) -> Result<(), InputManifestError>,
{
}

impl<H, F> Embed<H> for CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, CustomEmbedProvider<H>) -> Result<(), InputManifestError>,
{
    fn try_embed(
        &self,
        target_path: &Path,
        embed_provider: CustomEmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>> {
        Some((self.op)(target_path, embed_provider))
    }
}
