use crate::storage::build_id::BuildId;
use crate::{error::InputManifestError, util::clone_as_boxstr::CloneAsBoxstr};
use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

// WARNING: If you add fields, update `from_path` and `write_file`.
/// Metadata associated with an InputManifest in the Store.
#[derive(Debug)]
pub struct Metadata {
    /// The ID of the build.
    build_id: Option<BuildId>,
    /// The path of the target file described by the manifest.
    target_file_path: Option<PathBuf>,
}

impl Metadata {
    /// Construct a new metadata entry.
    pub fn new() -> Self {
        Metadata {
            build_id: Some(BuildId::new()),
            target_file_path: None,
        }
    }

    /// Construct Metadata for a specific target file.
    pub fn for_path(target_file_path: impl AsRef<Path>) -> Self {
        Metadata {
            build_id: Some(BuildId::new()),
            target_file_path: Some(target_file_path.as_ref().to_owned()),
        }
    }

    /// Initialize an empty Metadata object.
    fn empty() -> Self {
        Metadata {
            build_id: None,
            target_file_path: None,
        }
    }

    /// Load a manifest from a file.
    pub fn from_path(metadata_file_path: impl AsRef<Path>) -> Result<Metadata, InputManifestError> {
        let mut metadata = Metadata::empty();

        let contents = std::fs::read_to_string(metadata_file_path).unwrap();

        for line in contents.lines() {
            let (key, value) = parse_metadata_line(line).unwrap();

            match key.as_ref() {
                "build-id" => {
                    let build_id = value.parse()?;
                    metadata.build_id = Some(build_id);
                }
                "target-file-path" => {
                    // PANIC SAFETY: This parse() impl is infallible.
                    let target_file_path = value.parse().unwrap();
                    metadata.target_file_path = Some(target_file_path);
                }
                _ => todo!(),
            }
        }

        Ok(metadata)
    }

    /// Write the metadata to a file.
    pub fn write_file(
        &self,
        metadata_file_path: impl AsRef<Path>,
    ) -> Result<(), InputManifestError> {
        let mut file = std::fs::File::create(metadata_file_path).unwrap();

        if let Some(build_id) = self.build_id.as_ref() {
            write!(&mut file, "{} = {}", "build-id", build_id).unwrap();
        }

        if let Some(target_file_path) = self.target_file_path.as_ref() {
            write!(
                &mut file,
                "{} = {}",
                "target-file-path",
                target_file_path.display()
            )
            .unwrap();
        }

        Ok(())
    }
}

fn parse_metadata_line(line: &str) -> Result<(String, String), InputManifestError> {
    let parts = line
        .split_once("=")
        .ok_or_else(|| InputManifestError::InvalidMetadataLine(line.clone_as_boxstr()))?;
    let key = parts.0.trim().to_owned();
    let value = parts.1.trim().to_owned();
    Ok((key, value))
}
