/// Handles producing IDs and manifests during artifact construction.
#[derive(Debug)]
pub struct Identifier {
    mode: IdentifierMode,
}

impl Identifier {
    /// Create a new identifier.
    pub fn new(mode: IdentifierMode) -> Self {
        Identifier { mode }
    }
}

/// The mode to run the [`Identifier`] in.
#[derive(Debug)]
pub enum IdentifierMode {
    /// Embed the identifier for a manifest into the artifact.
    Embed,
    /// Do not embed the identifier for a manifest into the artifact.
    NoEmbed,
}
