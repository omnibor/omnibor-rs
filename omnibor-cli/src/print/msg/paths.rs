use crate::print::{CommandOutput, Status};
use console::Style;
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct PathsMsg {
    data: BTreeMap<String, Option<PathBuf>>,
}

impl PathsMsg {
    pub fn new() -> Self {
        PathsMsg {
            data: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, name: &'static str, path: Option<&Path>) {
        self.data
            .insert(name.to_string(), path.map(ToOwned::to_owned));
    }
}

fn opt_path(path: &Option<PathBuf>) -> String {
    path.as_deref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| String::from("None"))
}

impl CommandOutput for PathsMsg {
    fn plain_output(&self) -> String {
        let pad_width = self.data.keys().map(|key| key.len()).max().unwrap_or(10) + 2;

        self.data
            .iter()
            .fold(String::new(), |mut output, (name, path)| {
                output.push_str(&format!(
                    "{:>width$}: {}\n",
                    Style::new().blue().bold().apply_to(name),
                    opt_path(path),
                    width = pad_width,
                ));
                output
            })
    }

    fn short_output(&self) -> String {
        self.data.values().fold(String::new(), |mut output, path| {
            output.push_str(&format!("{}\n", opt_path(path)));
            output
        })
    }

    fn json_output(&self) -> serde_json::Value {
        self.data
            .iter()
            .fold(serde_json::Map::new(), |mut map, (name, path)| {
                map.insert(name.to_string(), json!(opt_path(path)));
                map
            })
            .into()
    }

    fn status(&self) -> Status {
        Status::Success
    }
}
