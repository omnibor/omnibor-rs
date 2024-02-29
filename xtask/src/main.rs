mod cli;
mod pipeline;
mod release;

use env_logger::{Builder as LoggerBuilder, Env};
use std::io::Write as _;
use std::process::ExitCode;

fn main() -> ExitCode {
    LoggerBuilder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "{:>10}: {}", record.level(), record.args()))
        .init();

    let args = cli::args();

    let res = match args.subcommand() {
        Some(("release", args)) => release::run(args),
        Some(_) | None => Ok(()),
    };

    if let Err(err) = res {
        log::error!("{}", err);

        for err in err.chain().skip(1) {
            log::error!("\tcaused by: {}", err);
        }

        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
