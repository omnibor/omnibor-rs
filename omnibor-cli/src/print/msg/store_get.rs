use crate::print::{CommandOutput, Status};
use console::Style;
use omnibor::{hash_algorithm::Sha256, ArtifactId, InputManifest};

#[derive(Debug, Clone)]
pub struct StoreGetMsg {
    /// The Input Manifest
    pub manifest: InputManifest<Sha256>,
}

impl CommandOutput for StoreGetMsg {
    fn plain_output(&self) -> String {
        let mut output = String::new();

        output.push_str(
            &Style::new()
                .blue()
                .bold()
                .apply_to(self.manifest.header())
                .to_string(),
        );

        for relation in self.manifest.inputs() {
            output.push_str(&relation.to_string());
        }

        output
    }

    fn short_output(&self) -> String {
        // SAFETY: Identifying a manifest is infallible.
        let manifest_aid = ArtifactId::new(&self.manifest).unwrap();

        Style::new()
            .blue()
            .bold()
            .apply_to(manifest_aid)
            .to_string()
    }

    fn json_output(&self) -> serde_json::Value {
        serde_json::json!({"manifest": self.plain_output()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
