use anyhow::anyhow;
use anyhow::Context as _;
use anyhow::Error;
use anyhow::Result;
use async_walkdir::DirEntry as AsyncDirEntry;
use async_walkdir::WalkDir;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use futures_lite::stream::StreamExt as _;
use omnibor::ArtifactId;
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
    /// Path to identify
    path: PathBuf,

    /// Output format (can be "plain" or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    format: Format,

    /// Hash algorithm (can be "sha256")
    #[arg(short = 'H', long = "hash", default_value_t)]
    hash: SelectedHash,
}

#[derive(Debug, Args)]
struct TreeArgs {
    /// Root of the tree to identify.
    path: PathBuf,

    /// Output format (can be "plain" or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    format: Format,

    /// Hash algorithm (can be "sha256")
    #[arg(short = 'H', long = "hash", default_value_t)]
    hash: SelectedHash,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SelectedHash {
    Sha256,
}

impl Default for SelectedHash {
    fn default() -> Self {
        SelectedHash::Sha256
    }
}

impl Display for SelectedHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SelectedHash::Sha256 => write!(f, "sha256"),
        }
    }
}

impl FromStr for SelectedHash {
    type Err = Error;

    fn from_str(s: &str) -> Result<SelectedHash> {
        match s {
            "sha256" => Ok(SelectedHash::Sha256),
            _ => Err(anyhow!("unknown hash algorithm '{}'", s)),
        }
    }
}

/*===========================================================================
 * Command Implementations
 *-------------------------------------------------------------------------*/

/// Run the `id` subcommand.
///
/// This command just produces the `gitoid` URL for the given file.
fn run_id(args: &IdArgs) -> Result<()> {
    let file = open_file(&args.path)?;
    let url = match args.hash {
        SelectedHash::Sha256 => sha256_id_file(&file, &args.path)?.url(),
    };

    match args.format {
        Format::Plain => println!("{}", url),
        Format::Json => {
            let output = json!({ "id": url.to_string() });
            println!("{}", output);
        }
    }

    Ok(())
}

/// Run the `tree` subcommand.
///
/// This command produces the `gitoid` URL for all files in a directory tree.
fn run_tree(args: &TreeArgs) -> Result<()> {
    let TreeArgs { path, format, hash } = args;

    Runtime::new()
        .context("failed to initialize the async runtime")?
        .block_on(async move {
            let mut entries = WalkDir::new(path);

            loop {
                match entries.next().await {
                    None => break,
                    Some(Err(e)) => print_error(e, *format),
                    Some(Ok(entry)) => {
                        let path = &entry.path();

                        if entry_is_dir(&entry).await? {
                            continue;
                        }

                        let mut file = open_async_file(path).await?;

                        // This 'match' is included to ensure this gets updated
                        // if we ever add a new hash algorithm.
                        let url = match *hash {
                            SelectedHash::Sha256 => {
                                sha256_id_async_file(&mut file, path).await?.url()
                            }
                        };

                        match *format {
                            Format::Plain => println!("{} => {}", path.display(), url),
                            Format::Json => println!(
                                "{}",
                                json!({
                                        "path": path.display().to_string(),
                                        "id": url.to_string()
                                })
                            ),
                        }
                    }
                }
            }

            Ok(())
        })
}

/*===========================================================================
 * Helper Functions
 *-------------------------------------------------------------------------*/

/// Print an error, respecting formatting.
fn print_error<E: Into<Error>>(error: E, format: Format) {
    fn _print_error(error: Error, format: Format) {
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

    _print_error(error.into(), format)
}

/// Print an error in plain formatting.
fn print_plain_error(error: Error) {
    eprintln!("error: {}", error);
}

/// Check if the entry is for a directory.
async fn entry_is_dir(entry: &AsyncDirEntry) -> Result<bool> {
    entry
        .file_type()
        .await
        .with_context(|| {
            format!(
                "unable to identify file type for '{}'",
                entry.path().display()
            )
        })
        .map(|file_type| file_type.is_dir())
}

/// Open a file.
fn open_file(path: &Path) -> Result<File> {
    File::open(path).with_context(|| format!("failed to open '{}'", path.display()))
}

/// Open an asynchronous file.
async fn open_async_file(path: &Path) -> Result<AsyncFile> {
    AsyncFile::open(path)
        .await
        .with_context(|| format!("failed to open file '{}'", path.display()))
}

/// Identify a file using a SHA-256 hash.
fn sha256_id_file(file: &File, path: &Path) -> Result<ArtifactId<Sha256>> {
    ArtifactId::id_reader(file)
        .with_context(|| format!("failed to produce Artifact ID for '{}'", path.display()))
}

/// Identify a file using a SHA-256 hash.
async fn sha256_id_async_file(file: &mut AsyncFile, path: &Path) -> Result<ArtifactId<Sha256>> {
    ArtifactId::id_async_reader(file)
        .await
        .with_context(|| format!("failed to produce Artifact ID for '{}'", path.display()))
}
