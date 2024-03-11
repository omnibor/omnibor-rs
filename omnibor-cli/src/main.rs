mod cli;
mod find;
mod fs;
mod id;
mod print;

use crate::cli::Cli;
use crate::cli::Command;
use crate::print::Printer;
use crate::print::PrinterCmd;
use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;
use tokio::sync::mpsc::Sender;
use tracing_subscriber::filter::EnvFilter;

// The environment variable to use when configuring the log.
const LOG_VAR: &str = "OMNIBOR_LOG";

#[tokio::main]
async fn main() -> ExitCode {
    init_log();

    let args = Cli::parse();
    let printer = Printer::launch(args.buffer);

    let exit_code = match run(printer.tx(), &args.command).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            printer.send(PrinterCmd::error(e, args.format())).await;
            ExitCode::FAILURE
        }
    };

    printer.join().await;

    exit_code
}

// Initialize the logging / tracing.
fn init_log() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env(LOG_VAR))
        .init();
}

/// Select and run the chosen command.
async fn run(tx: &Sender<PrinterCmd>, cmd: &Command) -> Result<()> {
    match cmd {
        Command::Id(ref args) => id::run(tx, args).await?,
        Command::Find(ref args) => find::run(tx, args).await?,
    }

    // Ensure we always send the "End" printer command.
    tx.send(PrinterCmd::End).await?;

    Ok(())
}
