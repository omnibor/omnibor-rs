use {crate::util::sealed::Sealed, core::marker::PhantomData};

#[cfg(doc)]
use crate::input_manifest::InputManifest;

/// The embedding mode to use when making new [`InputManifest`]s.
pub trait EmbeddingMode: Sealed {
    #[doc(hidden)]
    fn mode() -> Mode;
}

/// Indicates that embedding mode should be used.
pub struct Embed {
    _private: PhantomData<()>,
}

impl Sealed for Embed {}
impl EmbeddingMode for Embed {
    fn mode() -> Mode {
        Mode::Embed
    }
}

/// Indicates that non-embedding mode should be used.
pub struct NoEmbed {
    _private: PhantomData<()>,
}

impl Sealed for NoEmbed {}
impl EmbeddingMode for NoEmbed {
    fn mode() -> Mode {
        Mode::NoEmbed
    }
}

/// The mode to run the [`Identifier`] in.
#[doc(hidden)]
#[derive(Debug)]
pub enum Mode {
    /// Embed the identifier for a manifest into the artifact.
    Embed,

    /// Do not embed the identifier for a manifest into the artifact.
    NoEmbed,
}
