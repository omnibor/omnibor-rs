use crate::{cli::Args, config::Config};

#[derive(Debug)]
pub struct App {
    /// The user's command line arguments.
    pub args: Args,

    /// Configuration data.
    pub config: Config,
}
