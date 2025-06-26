use crate::{
    error::Error,
    print::{CommandOutput, Status},
};
use console::Style;
use serde_json::json;
use std::error::Error as StdError;
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
        error_string(&*self.error.lock().unwrap())
    }
}

fn error_string(error: &dyn StdError) -> String {
    fn error_string_inner(err_string: &mut String, error: &dyn StdError, inner: bool) {
        if inner {
            err_string.push_str(&format!(", {error}"));
        } else {
            err_string.push_str(&error.to_string());
        }

        if let Some(child) = error.source() {
            error_string_inner(err_string, child, true);
        }
    }

    let mut err_string = String::from("");
    error_string_inner(&mut err_string, error, false);
    err_string
}

impl CommandOutput for ErrorMsg {
    fn plain_output(&self) -> String {
        format!(
            "{}: {}",
            Style::new().red().apply_to("error"),
            self.error_string()
        )
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
