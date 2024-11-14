mod cli;
mod cmd;
mod error;
mod fs;
mod print;

use crate::{
    cli::{ArtifactCommand, Command, Config, DebugCommand, ManifestCommand},
    cmd::{artifact, debug, manifest},
    error::Result,
    print::{error::ErrorMsg, PrintSender, Printer, PrinterCmd},
};
use clap::Parser as _;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use std::process::ExitCode;
use tokio::runtime::Runtime;
use tracing::{trace, Subscriber};
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt as _, registry::LookupSpan,
    util::SubscriberInitExt as _, Layer,
};

// The environment variable to use when configuring the log.
const LOG_VAR: &str = "OMNIBOR_LOG";

fn main() -> ExitCode {
    let runtime = Runtime::new().expect("runtime construction succeeds");
    runtime.block_on(async { run().await })
}

async fn run() -> ExitCode {
    let config = Config::parse();
    init_log(&config.verbose);
    let printer = Printer::launch(config.buffer());
    trace!(config = ?config);

    match run_cmd(printer.tx(), &config).await {
        Ok(_) => {
            printer.join().await;
            ExitCode::SUCCESS
        }
        Err(e) => {
            printer
                .send(PrinterCmd::msg(ErrorMsg::new(e), config.format()))
                .await;
            printer.join().await;
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "console")]
/// Initialize the logging / tracing.
fn init_log(verbosity: &Verbosity<InfoLevel>) {
    let level_filter = adapt_level_filter(verbosity.log_level_filter());
    let filter = EnvFilter::from_env(LOG_VAR).add_directive(level_filter.into());
    let fmt_layer = fmt_layer(filter);
    let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(console_layer)
        .init();
}

#[cfg(not(feature = "console"))]
/// Initialize the logging / tracing.
fn init_log(verbosity: &Verbosity<InfoLevel>) {
    let level_filter = adapt_level_filter(verbosity.log_level_filter());
    let filter = EnvFilter::from_env(LOG_VAR).add_directive(level_filter.into());
    let fmt_layer = fmt_layer(filter);
    tracing_subscriber::registry().with(fmt_layer).init();
}

fn fmt_layer<S>(filter: EnvFilter) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_level(true)
                .without_time()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .compact(),
        )
        .with_filter(filter)
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
async fn run_cmd(tx: &PrintSender, config: &Config) -> Result<()> {
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
