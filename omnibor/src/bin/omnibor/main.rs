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

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();
    let printer = Printer::launch(args.buffer);

    let exit_code = match run(printer.tx(), &args.command).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            printer.send(PrinterCmd::error(e, args.format())).await;
            ExitCode::FAILURE
        }
    };

    printer.send(PrinterCmd::End).await;
    printer.join().await;
    exit_code
}

/// Select and run the chosen chosen.
async fn run(tx: &Sender<PrinterCmd>, cmd: &Command) -> Result<()> {
    match cmd {
        Command::Id(ref args) => id::run(tx, args).await,
        Command::Find(ref args) => find::run(tx, args).await,
    }
}
