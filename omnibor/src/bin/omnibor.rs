use anyhow::anyhow;
use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;
use async_recursion::async_recursion;
use async_walkdir::WalkDir;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use futures_lite::stream::StreamExt as _;
use omnibor::Sha256;
use serde_json::json;
use std::default::Default;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use tokio::fs::File as AsyncFile;
use tokio::runtime::Runtime;

fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match args.command {
        Command::Id(ref args) => run_id(args),
        Command::Tree(ref args) => run_tree(args),
    };

    if let Err(e) = result {
        if let Some(format) = &args.format() {
            print_error(e, *format);
        } else {
            print_plain_error(e);
        }

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

impl Cli {
    fn format(&self) -> Option<Format> {
        match &self.command {
            Command::Id(args) => Some(args.format),
            Command::Tree(args) => Some(args.format),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print the Artifact ID of the path given.
    Id(IdArgs),
    /// Print the Artifact IDs of a directory tree.
    Tree(TreeArgs),
}

#[derive(Debug, Args)]
struct IdArgs {
    /// The path to identify.
    path: PathBuf,

    /// The format of output
    #[arg(short = 'f', long = "format", default_value_t)]
    format: Format,
}

#[derive(Debug, Args)]
struct TreeArgs {
    /// The root of the tree to identify.
    path: PathBuf,

    /// The format of output (can be "plain" or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    format: Format,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Format {
    Plain,
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Format::Plain
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Format::Plain => write!(f, "plain"),
            Format::Json => write!(f, "json"),
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Format> {
        match s {
            "plain" => Ok(Format::Plain),
            "json" => Ok(Format::Json),
            _ => Err(anyhow!("unknown format '{}'", s)),
        }
    }
}

/*===========================================================================
 * Command Implementations
 *-------------------------------------------------------------------------*/

/// Type alias for the specific ID we're using.
type ArtifactId = omnibor::ArtifactId<Sha256>;

/// Run the `id` subcommand.
///
/// This command just produces the `gitoid` URL for the given file.
fn run_id(args: &IdArgs) -> Result<()> {
    let path = &args.path;
    let file = File::open(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    let id = ArtifactId::id_reader(&file).context("failed to produce Artifact ID")?;

    match args.format {
        Format::Plain => {
            println!("{}", id.url());
        }
        Format::Json => {
            let output = json!({ "id": id.url().to_string() });
            println!("{}", output);
        }
    }

    Ok(())
}

/// Run the `tree` subcommand.
///
/// This command produces the `gitoid` URL for all files in a directory tree.
fn run_tree(args: &TreeArgs) -> Result<()> {
    #[async_recursion]
    async fn process_dir(path: &Path, format: Format) -> Result<()> {
        let mut entries = WalkDir::new(path);

        loop {
            match entries.next().await {
                Some(Ok(entry)) => {
                    let path = &entry.path();

                    let file_type = entry
                        .file_type()
                        .await
                        .with_context(|| format!("unable to identify file type for '{}'", path.display()))?;

                    if file_type.is_dir() {
                        process_dir(path, format).await?;
                        continue;
                    }

                    let mut file = AsyncFile::open(path)
                        .await
                        .with_context(|| format!("failed to open file '{}'", path.display()))?;

                    let id = ArtifactId::id_async_reader(&mut file)
                        .await
                        .with_context(|| {
                            format!("failed to produce Artifact ID for '{}'", path.display())
                        })?;

                    match format {
                        Format::Plain => println!("{} => {}", path.display(), id.url()),
                        Format::Json => {
                            let output = json!({
                                "path": path.display().to_string(),
                                "id": id.url().to_string()
                            });

                            println!("{}", output);
                        }
                    }
                }
                Some(Err(e)) => print_error(Error::from(e), format),
                None => break,
            }
        }

        Ok(())
    }

    let runtime = Runtime::new().context("failed to initialize the async runtime")?;
    runtime.block_on(process_dir(&args.path, args.format))
}

/// Print an error, respecting formatting.
fn print_error(error: Error, format: Format) {
    match format {
        Format::Plain => print_plain_error(error),
        Format::Json => {
            let output = json!({
                "error": error.to_string(),
            });

            eprintln!("{}", output);
        }
    }
}

/// Print an error in plain formatting.
fn print_plain_error(error: Error) {
    eprintln!("error: {}", error);
}
