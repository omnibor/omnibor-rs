use crate::print::{CommandOutput, Status};
use console::Style;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FindFileMsg {
    pub path: PathBuf,
    pub id: String,
}

impl FindFileMsg {
    fn path_string(&self) -> String {
        self.path.display().to_string()
    }

    fn id_string(&self) -> String {
        self.id.clone()
    }
}

impl CommandOutput for FindFileMsg {
    fn plain_output(&self) -> String {
        format!(
            "{} {} {}",
            Style::new().blue().bold().apply_to(self.id_string()),
            Style::new().dim().apply_to("=>"),
            self.path_string()
        )
    }

    fn short_output(&self) -> String {
        self.path_string()
    }

    fn json_output(&self) -> serde_json::Value {
        json!({"path": self.path_string(), "id": self.id_string()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
