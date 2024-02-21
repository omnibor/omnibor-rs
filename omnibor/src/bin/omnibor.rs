use anyhow::Context as _;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use omnibor::Sha256;
use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match args.command {
        Command::Id(args) => run_id(args),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}


/*===========================================================================
 * CLI Arguments
 *-------------------------------------------------------------------------*/

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the Artifact ID of the path given.
    Id(IdArgs),
}

#[derive(Debug, Args)]
struct IdArgs {
    /// The path to identify.
    path: PathBuf,
}


/*===========================================================================
 * Command Implementations
 *-------------------------------------------------------------------------*/

/// Type alias for the specific ID we're using.
type ArtifactId = omnibor::ArtifactId<Sha256>;

/// Run the `id` subcommand.
///
/// This command just produces the `gitoid` URL for the given file.
fn run_id(args: IdArgs) -> Result<()> {
    let path = &args.path;
    let file = File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    let id = ArtifactId::id_reader(&file).context("failed to produce Artifact ID")?;
    println!("{}", id.url());
    Ok(())
}
