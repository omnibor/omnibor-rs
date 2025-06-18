use crate::print::{CommandOutput, Status};
use omnibor::{hash_algorithm::Sha256, InputManifest};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ManifestFindMsg {
    /// The path the Manifest was found in.
    pub path: PathBuf,
    /// The Input Manifest
    pub manifest: InputManifest<Sha256>,
}

impl CommandOutput for ManifestFindMsg {
    fn plain_output(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Found in '{}'\n", self.path.display()));

        output.push_str(&self.manifest.header().to_string());

        for relation in self.manifest.relations() {
            output.push_str(&relation.to_string());
        }

        output
    }

    fn short_output(&self) -> String {
        self.plain_output()
    }

    fn json_output(&self) -> serde_json::Value {
        serde_json::json!({"path": self.path.display().to_string(), "manifest": self.plain_output()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
