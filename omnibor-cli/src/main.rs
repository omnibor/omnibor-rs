mod app;
mod cli;
mod cmd;
mod config;
mod error;
mod fs;
mod log;
mod print;

use crate::{
    app::App,
    cli::{Args, ArtifactCommand, Command, DebugCommand, ManifestCommand},
    cmd::{artifact, debug, manifest},
    config::Config,
    error::Result,
    log::init_log,
    print::{error::ErrorMsg, PrintSender, Printer, PrinterCmd},
};
use clap::Parser as _;
use std::{error::Error as StdError, process::ExitCode};
use tokio::runtime::Runtime;
use tracing::{error, trace};

fn main() -> ExitCode {
    let runtime = Runtime::new().expect("runtime construction succeeds");
    runtime.block_on(async { run().await })
}

async fn run() -> ExitCode {
    let args = Args::parse();
    init_log(args.verbosity(), args.console());

    let config = match Config::init(args.config()) {
        Ok(config) => config,
        Err(error) => {
            log_error(&error);
            return ExitCode::FAILURE;
        }
    };

    let app = App { args, config };
    trace!(app = ?app);

    let printer = Printer::launch(app.config.perf.print_queue_size());

    match run_cmd(printer.tx(), &app).await {
        Ok(_) => {
            printer.join().await;
            ExitCode::SUCCESS
        }
        Err(e) => {
            printer
                .send(PrinterCmd::msg(ErrorMsg::new(e), app.args.format()))
                .await;
            printer.join().await;
            ExitCode::FAILURE
        }
    }
}

/// Select and run the chosen command.
async fn run_cmd(tx: &PrintSender, app: &App) -> Result<()> {
    match app.args.command() {
        Command::Artifact(ref args) => match args.command() {
            ArtifactCommand::Id(ref args) => artifact::id::run(tx, app, args).await?,
            ArtifactCommand::Find(ref args) => artifact::find::run(tx, app, args).await?,
        },
        Command::Manifest(ref args) => match args.command() {
            ManifestCommand::Add(ref args) => manifest::add::run(tx, app, args).await?,
            ManifestCommand::Remove(ref args) => manifest::remove::run(tx, app, args).await?,
            ManifestCommand::Create(ref args) => manifest::create::run(tx, app, args).await?,
        },
        Command::Debug(ref args) => match args.command() {
            DebugCommand::Paths(ref args) => debug::paths::run(tx, app, args).await?,
        },
    }

    // Ensure we always send the "End" printer command.
    tx.send(PrinterCmd::End).await?;

    Ok(())
}

fn log_error(error: &dyn StdError) {
    error!("{}", error);

    if let Some(child) = error.source() {
        log_error(child);
    }
}
