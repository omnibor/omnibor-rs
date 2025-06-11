use crate::print::{CommandOutput, Status};
use console::Style;
use serde_json::json;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone)]
pub struct IdFileMsg {
    pub path: PathBuf,
    pub id: Url,
}

impl IdFileMsg {
    fn path_string(&self) -> String {
        self.path.display().to_string()
    }

    fn id_string(&self) -> String {
        self.id.to_string()
    }
}

impl CommandOutput for IdFileMsg {
    fn plain_output(&self) -> String {
        format!(
            "{} {} {}",
            Style::new().blue().bold().apply_to(self.path_string()),
            Style::new().dim().apply_to("=>"),
            self.id_string()
        )
    }

    fn short_output(&self) -> String {
        self.id_string()
    }

    fn json_output(&self) -> serde_json::Value {
        json!({"path": self.path_string(), "id": self.id_string()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
