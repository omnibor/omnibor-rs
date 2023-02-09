use crate::Result;
use gitoid::GitOid;
use gitoid::HashAlgorithm;
use std::collections::BTreeSet;
use std::path::Path;

/// An Artifact Input Manifest (AIM) from a binary or file.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Manifest {
    /// The hash algorithm associated with all records in the manifest.
    hash_algorithm: HashAlgorithm,
    /// The individual entries in the manifest.
    entries: BTreeSet<ManifestEntry>,
}

/// An individual entry in the `Manifest`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ManifestEntry {
    /// The identifier for the input.
    artifact_id: GitOid,
    /// The identifier for the manifest of inputs to the input.
    manifest_id: Option<GitOid>,
}

impl ManifestEntry {
    // Get the ID of the artifact in question.
    pub fn artifact_id(&self) -> GitOid {
        self.artifact_id
    }

    /// Get the ID of the manifest describing the inputs to the artifact, in a manifest exists.
    pub fn manifest_id(&self) -> Option<GitOid> {
        self.manifest_id
    }
}

impl PartialOrd for ManifestEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.artifact_id().partial_cmp(&other.artifact_id())
    }
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
