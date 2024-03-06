//! A Task Runner for the OmniBOR Rust workspace.

mod cli;
mod pipeline;
mod release;

use env_logger::{Builder as LoggerBuilder, Env};
use std::process::ExitCode;

fn main() -> ExitCode {
    LoggerBuilder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();

    let args = cli::args();

    let res = match args.subcommand() {
        Some(("release", args)) => release::run(args),
        Some(_) | None => Ok(()),
    };

    if let Err(err) = res {
        log::error!("{}", err);

        // We skip the first error in the chain because it's the
        // exact error we've just printed.
        for err in err.chain().skip(1) {
            log::error!("\tcaused by: {}", err);
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
