use crate::print::{CommandOutput, Status};
use console::Style;
use omnibor::{hash_algorithm::Sha256, InputManifest};

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
                .apply_to(format!("{}\n", self.manifest.header()))
                .to_string(),
        );

        for relation in self.manifest.relations() {
            output.push_str(&relation.to_string());
        }

        output
    }

    fn short_output(&self) -> String {
        unimplemented!("this command does not support 'short' format")
    }

    fn json_output(&self) -> serde_json::Value {
        unimplemented!("this command does not support 'json' format")
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
