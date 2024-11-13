mod cli;
mod cmd;
mod error;
mod fs;
mod print;

use crate::{
    cli::{ArtifactCommand, Command, Config, DebugCommand, ManifestCommand},
    cmd::{artifact, debug, manifest},
    error::Result,
    print::{Printer, PrinterCmd},
};
use clap::Parser as _;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use std::process::ExitCode;
use tokio::sync::mpsc::Sender;
use tracing::trace;
use tracing_subscriber::filter::EnvFilter;

// The environment variable to use when configuring the log.
const LOG_VAR: &str = "OMNIBOR_LOG";

#[tokio::main]
async fn main() -> ExitCode {
    let config = Config::parse();

    init_log(&config.verbose);

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

/// Initialize the logging / tracing.
fn init_log(verbosity: &Verbosity<InfoLevel>) {
    let level_filter = adapt_level_filter(verbosity.log_level_filter());
    let filter = EnvFilter::from_env(LOG_VAR).add_directive(level_filter.into());

    let format = tracing_subscriber::fmt::format()
        .with_level(true)
        .without_time()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact();

    tracing_subscriber::fmt()
        .event_format(format)
        .with_env_filter(filter)
        .init();
}

/// Convert the clap LevelFilter to the tracing LevelFilter.
fn adapt_level_filter(
    clap_filter: clap_verbosity_flag::LevelFilter,
) -> tracing_subscriber::filter::LevelFilter {
    match clap_filter {
        clap_verbosity_flag::LevelFilter::Off => tracing_subscriber::filter::LevelFilter::OFF,
        clap_verbosity_flag::LevelFilter::Error => tracing_subscriber::filter::LevelFilter::ERROR,
        clap_verbosity_flag::LevelFilter::Warn => tracing_subscriber::filter::LevelFilter::WARN,
        clap_verbosity_flag::LevelFilter::Info => tracing_subscriber::filter::LevelFilter::INFO,
        clap_verbosity_flag::LevelFilter::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
        clap_verbosity_flag::LevelFilter::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
    }
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
