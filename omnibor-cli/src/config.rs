use crate::{
    cli::DEFAULT_CONFIG,
    error::{Error, Result},
};
use serde::Deserialize;
use std::{fs::File, path::Path};
use tokio::runtime::Handle;
use tracing::debug;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub perf: PerfConfig,
}

impl Config {
    pub fn init(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            debug!("no config path provided");
            return Ok(Config::default());
        };

        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                // If we simply didn't find the default file, that's fine. It's
                // allowed to be missing, we just use the default config.
                if file_was_not_found(&error) && is_default_path(path) {
                    return Ok(Config::default());
                }

                match (file_was_not_found(&error), is_default_path(path)) {
                    // Not found, is default.
                    (true, true) => return Ok(Config::default()),
                    // Not found, is not default.
                    (true, false) => {
                        return Err(Error::ConfigNotFound {
                            path: path.to_path_buf(),
                            source: error,
                        });
                    }
                    // Found, is default.
                    (false, true) => {
                        return Err(Error::ConfigDefaultCouldNotRead {
                            path: path.to_path_buf(),
                            source: error,
                        })
                    }
                    // Found, is not default.
                    (false, false) => {
                        return Err(Error::ConfigCouldNotRead {
                            path: path.to_path_buf(),
                            source: error,
                        })
                    }
                }
            }
        };

        let config = serde_json::from_reader(file).map_err(Error::CantReadConfig)?;
        Ok(config)
    }
}

fn file_was_not_found(error: &std::io::Error) -> bool {
    matches!(error.kind(), std::io::ErrorKind::NotFound)
}

fn is_default_path(path: &Path) -> bool {
    let Some(Some(default_path)) = DEFAULT_CONFIG.get() else {
        return false;
    };

    path == default_path
}

#[derive(Debug, Default, Deserialize)]
pub struct PerfConfig {
    /// The max number of print items that can be held in the print queue.
    print_queue_size: PrintQueueSize,

    /// The max number of work items that can be held in the work queue.
    work_queue_size: WorkQueueSize,

    /// The number of worker tasks to spawn.
    num_workers: NumWorkers,
}

impl PerfConfig {
    pub fn print_queue_size(&self) -> usize {
        self.print_queue_size.0
    }

    pub fn work_queue_size(&self) -> usize {
        self.work_queue_size.0
    }

    pub fn num_workers(&self) -> usize {
        self.num_workers.0
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct PrintQueueSize(usize);

impl Default for PrintQueueSize {
    fn default() -> Self {
        PrintQueueSize(100)
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct WorkQueueSize(usize);

impl Default for WorkQueueSize {
    fn default() -> Self {
        WorkQueueSize(100)
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct NumWorkers(usize);

impl Default for NumWorkers {
    fn default() -> Self {
        let num = Handle::current().metrics().num_workers() - 1;
        NumWorkers(num)
    }
}
