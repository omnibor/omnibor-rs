use crate::{cli::Args, config::Config, print::PrintSender};
use std::fmt::Debug;

pub struct App {
    /// The user's command line arguments.
    pub args: Args,

    /// Configuration data.
    pub config: Config,

    /// Sender for print data.
    pub print_tx: PrintSender,
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
