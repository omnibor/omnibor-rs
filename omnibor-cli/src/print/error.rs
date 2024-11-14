use crate::{
    error::Error,
    print::{CommandOutput, Status},
};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct ErrorMsg {
    pub error: Arc<Mutex<Error>>,
}

impl ErrorMsg {
    pub fn new(error: Error) -> Self {
        ErrorMsg {
            error: Arc::new(Mutex::new(error)),
        }
    }

    fn error_string(&self) -> String {
        // SAFETY: This error type should only have a singular owner anyway.
        self.error.lock().unwrap().to_string()
    }
}

impl CommandOutput for ErrorMsg {
    fn plain_output(&self) -> String {
        format!("error: {}", self.error_string())
    }

    fn short_output(&self) -> String {
        self.error_string()
    }

    fn json_output(&self) -> serde_json::Value {
        json!({"error": self.error_string()})
    }

    fn status(&self) -> Status {
        Status::Error
    }
}
