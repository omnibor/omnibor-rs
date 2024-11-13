mod cli;
mod cmd;
mod fs;
mod print;

use crate::{
    cli::{ArtifactCommand, Command, Config, DebugCommand, ManifestCommand},
    cmd::{artifact, debug, manifest},
    print::{Printer, PrinterCmd},
};
use anyhow::Result;
use clap::Parser as _;
use std::process::ExitCode;
use tokio::sync::mpsc::Sender;
use tracing::trace;
use tracing_subscriber::filter::EnvFilter;

// The environment variable to use when configuring the log.
const LOG_VAR: &str = "OMNIBOR_LOG";

#[tokio::main]
async fn main() -> ExitCode {
    init_log();
    let config = Config::parse();
    let printer = Printer::launch(config.buffer());

    trace!(config = ?config);

    match run(printer.tx(), &config).await {
        Ok(_) => {
            printer.join().await;
            ExitCode::SUCCESS
        }
        Err(e) => {
            printer.send(PrinterCmd::error(e, config.format())).await;
            printer.join().await;
            ExitCode::FAILURE
        }
    }
}

// Initialize the logging / tracing.
fn init_log() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env(LOG_VAR))
        .init();
}

/// Select and run the chosen command.
async fn run(tx: &Sender<PrinterCmd>, config: &Config) -> Result<()> {
    match config.command() {
        Command::Artifact(ref args) => match args.command() {
            ArtifactCommand::Id(ref args) => artifact::id::run(tx, config, args).await?,
            ArtifactCommand::Find(ref args) => artifact::find::run(tx, config, args).await?,
        },
        Command::Manifest(ref args) => match args.command() {
            ManifestCommand::Add(ref args) => manifest::add::run(tx, config, args).await?,
            ManifestCommand::Remove(ref args) => manifest::remove::run(tx, config, args).await?,
            ManifestCommand::Create(ref args) => manifest::create::run(tx, config, args).await?,
        },
        Command::Debug(ref args) => match args.command() {
            DebugCommand::Config => debug::config::run(tx, config).await?,
        },
    }

    // Ensure we always send the "End" printer command.
    tx.send(PrinterCmd::End).await?;

    Ok(())
}
