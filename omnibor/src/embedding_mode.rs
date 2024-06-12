use crate::sealed::Sealed;
#[cfg(doc)]
use crate::InputManifest;
use std::marker::PhantomData;

/// Helper struct for converting from type-level mode data to value-level.
pub(crate) struct GetMode<M: EmbeddingMode>(PhantomData<M>);

impl GetMode<Embed> {
    pub(crate) fn mode() -> Mode {
        Mode::Embed
    }
}

impl GetMode<NoEmbed> {
    pub(crate) fn mode() -> Mode {
        Mode::NoEmbed
    }
}

/// The embedding mode to use when making new [`InputManifest`]s.
pub trait EmbeddingMode: Sealed {}

/// Indicates that embedding mode should be used.
pub struct Embed {
    _private: PhantomData<()>,
}

impl Sealed for Embed {}
impl EmbeddingMode for Embed {}

/// Indicates that non-embedding mode should be used.
pub struct NoEmbed {
    _private: PhantomData<()>,
}

impl Sealed for NoEmbed {}
impl EmbeddingMode for NoEmbed {}

/// The mode to run the [`Identifier`] in.
#[derive(Debug)]
pub(crate) enum Mode {
    /// Embed the identifier for a manifest into the artifact.
    Embed,

    /// Do not embed the identifier for a manifest into the artifact.
    NoEmbed,
}
