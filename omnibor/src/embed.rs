//! Control whether an `InputManifest`'s `ArtifactId` is stored in the target artifact.

#[cfg(feature = "infer-filetypes")]
pub(crate) mod auto_embed;
pub(crate) mod embed_provider;

use crate::{error::InputManifestError, hash_algorithm::HashAlgorithm, util::sealed::Sealed};
use std::{marker::PhantomData, path::Path};

pub use crate::embed::embed_provider::EmbedProvider;

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
        embed_provider: EmbedProvider<H>,
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
        _embed_provider: EmbedProvider<H>,
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
        embed_provider: EmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>> {
        Some(embed_manifest_in_target(target_path, embed_provider))
    }
}

/// A custom embedding function.
pub struct CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, EmbedProvider<H>) -> Result<(), InputManifestError>,
{
    op: F,
    _phantom: PhantomData<H>,
}

impl<H, F> CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, EmbedProvider<H>) -> Result<(), InputManifestError>,
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
    F: Fn(&Path, EmbedProvider<H>) -> Result<(), InputManifestError>,
{
}

impl<H, F> Embed<H> for CustomEmbed<H, F>
where
    H: HashAlgorithm,
    F: Fn(&Path, EmbedProvider<H>) -> Result<(), InputManifestError>,
{
    fn try_embed(
        &self,
        target_path: &Path,
        embed_provider: EmbedProvider<H>,
    ) -> Option<Result<(), InputManifestError>> {
        Some((self.op)(target_path, embed_provider))
    }
}
