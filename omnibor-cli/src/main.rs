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
    cli::{Args, Command, DebugCommand, IdCommand, ManifestCommand, StoreCommand},
    cmd::{debug, id, manifest, store},
    config::Config,
    error::Result,
    log::init_log,
    print::{msg::error::ErrorMsg, Printer, PrinterCmd},
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

    let printer = Printer::launch(config.perf.print_queue_size());

    let app = App {
        args,
        config,
        print_tx: printer.tx().clone(),
    };
    trace!(app = ?app);

    let exit_code = match run_cmd(&app).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            printer
                .send(PrinterCmd::msg(ErrorMsg::new(e), app.args.format()))
                .await;
            ExitCode::FAILURE
        }
    };

    // Ensure we always send the "End" printer command.
    app.print_tx.send(PrinterCmd::End).await.unwrap();
    printer.join().await;
    exit_code
}

/// Select and run the chosen command.
async fn run_cmd(app: &App) -> Result<()> {
    match app.args.command() {
        Command::Id(ref args) => match args.command {
            IdCommand::Create(ref args) => id::create::run(app, args).await,
            IdCommand::Find(ref args) => id::find::run(app, args).await,
        },
        Command::Manifest(ref args) => match args.command {
            ManifestCommand::Create(ref args) => manifest::create::run(app, args).await,
        },
        Command::Store(ref args) => match args.command {
            StoreCommand::Add(ref args) => store::add::run(app, args).await,
            StoreCommand::Remove(ref args) => store::remove::run(app, args).await,
            StoreCommand::List(ref args) => store::list::run(app, args).await,
            StoreCommand::Get(ref args) => store::get::run(app, args).await,
        },
        Command::Debug(ref args) => match args.command {
            DebugCommand::Paths(ref args) => debug::paths::run(app, args).await,
        },
    }
}

fn log_error(error: &dyn StdError) {
    error!("{}", error);

    if let Some(child) = error.source() {
        log_error(child);
    }
}
