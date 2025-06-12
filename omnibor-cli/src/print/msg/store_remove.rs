use crate::print::{CommandOutput, Status};
use console::Style;
use omnibor::{hash_algorithm::Sha256, ArtifactId};

#[derive(Debug, Clone)]
pub struct StoreRemoveMsg {
    /// The Input Manifest's Artifact ID
    pub manifest_aid: ArtifactId<Sha256>,
}

impl CommandOutput for StoreRemoveMsg {
    fn plain_output(&self) -> String {
        format!(
            "Removed {}\n",
            Style::new().blue().bold().apply_to(self.manifest_aid)
        )
    }

    fn short_output(&self) -> String {
        String::new()
    }

    fn json_output(&self) -> serde_json::Value {
        serde_json::json!({"removed": self.manifest_aid.to_string()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
