use crate::Result;
use gitoid::{GitOid, HashAlgorithm};
use std::{collections::BTreeSet, io::Write, path::Path};

/// An Artifact Input Manifest (AIM) from a binary or file.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Manifest {
    /// The hash algorithm associated with all records in the manifest.
    hash_algorithm: HashAlgorithm,
    /// The individual entries in the manifest.
    entries: BTreeSet<ManifestEntry>,
}

/// An individual entry in the `Manifest`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ManifestEntry {
    /// A terminal object.
    Input(GitOid),
    /// Another manifest.
    Manifest(ManifestRef),
}

/// An identifier for a `Manifest` referenced inside a `Manifest`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ManifestRef {
    /// The `GitOid` for the artifact whose inputs are in the manifest.
    target_id: GitOid,
    /// The `GitOid` for the manifest itself.
    manifest_id: GitOid,
}

/// An ELF section containing a byte representation of a manifest.
pub struct ElfSection(Vec<u8>);

/// A handle to produce the plaintext representation of a manifest.
pub struct ManifestText<'m>(&'m Manifest);

impl Manifest {
    /// Load a manifest embedded in an ELF file.
    pub fn from_elf_binary<P>(path: P) -> Result<Manifest>
    where
        P: AsRef<Path>,
    {
        fn inner(_path: &Path) -> Result<Manifest> {
            todo!()
        }

        inner(path.as_ref())
    }

    /// Load a manifest from a plaintext file.
    pub fn from_text_file<P>(path: P) -> Result<Manifest>
    where
        P: AsRef<Path>,
    {
        fn inner(_path: &Path) -> Result<Manifest> {
            todo!()
        }

        inner(path.as_ref())
    }

    /// Get the bytes to embed the manifest in an ELF binary.
    pub fn as_elf_section(&self) -> ElfSection {
        todo!()
    }

    /// Get a handle to the plaintext representation of the manifest.
    pub fn as_text(&self) -> ManifestText {
        todo!()
    }
}
