use crate::print::{CommandOutput, Status};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RootDirMsg {
    pub path: PathBuf,
}

impl RootDirMsg {
    fn path_string(&self) -> String {
        self.path.display().to_string()
    }
}

impl CommandOutput for RootDirMsg {
    fn plain_output(&self) -> String {
        format!("root_dir: {}", self.path_string())
    }

    fn short_output(&self) -> String {
        self.path_string()
    }

    fn json_output(&self) -> serde_json::Value {
        json!({"path": self.path_string()})
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
