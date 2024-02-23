mod cli;
mod pipeline;
mod release;

use anyhow::Result;
use env_logger::{Env, Builder as LoggerBuilder};

fn main() -> Result<()> {
    LoggerBuilder::from_env(Env::default().default_filter_or("info")).init();

    let args = cli::args();

    match args.subcommand() {
        Some(("release", args)) => release::run(args),
        Some(_) | None => Ok(()),
    }
}
