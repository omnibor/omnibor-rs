use crate::{
    cli::Args,
    config::Config,
    error::{Error, Result},
    print::PrintSender,
};
use omnibor::{
    hash_algorithm::Sha256,
    storage::{FileSystemStorage, Storage},
};
use std::fmt::Debug;

pub struct App {
    /// The user's command line arguments.
    pub args: Args,

    /// Configuration data.
    pub config: Config,

    /// Sender for print data.
    pub print_tx: PrintSender,
}

impl App {
    /// Get a handle to the on-disk storage for manifests.
    pub fn storage(&self) -> Result<impl Storage<Sha256>> {
        let root = self.args.dir().ok_or(Error::NoRoot)?;
        let storage = FileSystemStorage::new(root).map_err(Error::StorageInitFailed)?;
        Ok(storage)
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("args", &self.args)
            .field("config", &self.config)
            .field("print_tx", &"<PrintSender>")
            .finish()
    }
}
