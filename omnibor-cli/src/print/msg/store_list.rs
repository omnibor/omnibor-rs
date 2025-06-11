use crate::print::{CommandOutput, Status};
use console::Style;
use omnibor::{hash_algorithm::Sha256, ArtifactId};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct StoreListMsg {
    /// The Artifact ID of the manifest itself.
    pub manifest: ArtifactId<Sha256>,
    /// The Artifact ID of the manifest's target artifact.
    pub target: Option<ArtifactId<Sha256>>,
}

impl StoreListMsg {
    fn manifest(&self) -> String {
        self.manifest.to_string()
    }

    fn target(&self) -> String {
        self.target
            .map(|target| target.to_string())
            .unwrap_or("(unknown target)".to_owned())
    }
}

impl CommandOutput for StoreListMsg {
    fn plain_output(&self) -> String {
        format!(
            "{} (manifest for {})",
            Style::new().blue().bold().apply_to(self.manifest()),
            Style::new().green().bold().apply_to(self.target())
        )
    }

    fn short_output(&self) -> String {
        self.manifest()
    }

    fn json_output(&self) -> serde_json::Value {
        json!({"manifest": self.manifest(), "target": self.target()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
