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
use smart_default::SmartDefault;
use std::default::Default;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use tokio::fs::File as AsyncFile;
use tokio::runtime::Runtime;
use url::Url;

fn main() -> ExitCode {
    let args = Cli::parse();

    let result = match args.command {
        Command::Id(ref args) => run_id(args),
        Command::Find(ref args) => run_find(args),
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
            Command::Find(args) => Some(args.format),
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it.
    Id(IdArgs),

    /// Find file matching an Artifact ID.
    Find(FindArgs),
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
struct FindArgs {
    /// `gitoid` URL to match
    url: Url,

    /// The root path to search under
    path: PathBuf,

    /// Output format (can be "plain" or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    format: Format,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault)]
enum Format {
    #[default]
    Plain,
    Json,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault)]
enum SelectedHash {
    #[default]
    Sha256,
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
fn run_id(args: &IdArgs) -> Result<()> {
    Runtime::new()
        .context("failed to initialize the async runtime")?
        .block_on(async move {
            let mut file = open_async_file(&args.path).await?;

            if file_is_dir(&file).await? {
                id_directory(&args.path, args.format, args.hash).await
            } else {
                id_file(&mut file, &args.path, args.format, args.hash).await
            }
        })
}

/// Run the `find` subcommand.
fn run_find(args: &FindArgs) -> Result<()> {
    let FindArgs { url, path, format } = args;

    Runtime::new()
        .context("failed to initialize the async runtime")?
        .block_on(async move {
            let id = ArtifactId::<Sha256>::id_url(url.clone())?;
            let url = id.url();

            let mut entries = WalkDir::new(&path);

            loop {
                match entries.next().await {
                    None => break,
                    Some(Err(e)) => print_error(e, *format),
                    Some(Ok(entry)) => {
                        let path = &entry.path();

                        if entry_is_dir(&entry).await? {
                            continue;
                        }

                        let mut file = open_async_file(&path).await?;
                        let file_url = hash_file(SelectedHash::Sha256, &mut file, &path).await?;

                        if url == file_url {
                            print_id(&path, &url, *format);
                            return Ok(());
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

// Identify, recursively, all the files under a directory.
async fn id_directory(path: &Path, format: Format, hash: SelectedHash) -> Result<()> {
    let mut entries = WalkDir::new(path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(e)) => print_error(e, format),
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(&path).await?;
                id_file(&mut file, &path, format, hash).await?;
            }
        }
    }

    Ok(())
}

/// Identify a single file.
async fn id_file(
    file: &mut AsyncFile,
    path: &Path,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    let url = hash_file(hash, file, &path).await?;
    print_id(path, &url, format);
    Ok(())
}

/// Hash the file and produce a `gitoid`-scheme URL.
async fn hash_file(hash: SelectedHash, file: &mut AsyncFile, path: &Path) -> Result<Url> {
    match hash {
        SelectedHash::Sha256 => sha256_id_async_file(file, &path).await.map(|id| id.url()),
    }
}

/// Print IDs for path and file in the chosen format.
fn print_id(path: &Path, url: &Url, format: Format) {
    let path = path.display().to_string();
    let url = url.to_string();

    match format {
        Format::Plain => println!("path: {}, id: {}", path, url),
        Format::Json => println!("{}", json!({ "path": path, "id": url })),
    }
}

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

/// Check if the file is for a directory.
async fn file_is_dir(file: &AsyncFile) -> Result<bool> {
    Ok(file.metadata().await.map(|meta| meta.is_dir())?)
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

/// Open an asynchronous file.
async fn open_async_file(path: &Path) -> Result<AsyncFile> {
    AsyncFile::open(path)
        .await
        .with_context(|| format!("failed to open file '{}'", path.display()))
}

/// Identify a file using a SHA-256 hash.
async fn sha256_id_async_file(file: &mut AsyncFile, path: &Path) -> Result<ArtifactId<Sha256>> {
    ArtifactId::id_async_reader(file)
        .await
        .with_context(|| format!("failed to produce Artifact ID for '{}'", path.display()))
}
