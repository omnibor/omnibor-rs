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
use serde_json::Value as JsonValue;
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
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use url::Url;

#[tokio::main]
async fn main() -> ExitCode {
    let args = Cli::parse();

    // TODO(alilleybrinker): Make this channel Msg limit configurable.
    let (tx, mut rx) = mpsc::channel::<Msg>(args.buffer.unwrap_or(100));

    // Do all printing in a separate task we spawn to _just_ do printing.
    // This stops printing from blocking the worker tasks.
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            // TODO(alilleybrinker): Handle this error.
            let _ = msg.print().await;
        }
    });

    let result = match args.command {
        Command::Id(ref args) => run_id(&tx, args).await,
        Command::Find(ref args) => run_find(&tx, args).await,
    };

    if let Err(e) = result {
        // TODO(alilleybrinker): Handle this erroring out, probably by
        //                       sync-printing as a last resort.
        let _ = tx.send(Msg::error(e, args.format())).await;
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

    /// How many print messages to buffer at one time, tunes printing perf
    #[arg(short = 'b', long = "buffer")]
    buffer: Option<usize>,
}

impl Cli {
    fn format(&self) -> Format {
        match &self.command {
            Command::Id(args) => args.format,
            Command::Find(args) => args.format,
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

#[derive(Debug)]
struct Msg {
    content: Content,
    status: Status,
}

impl Msg {
    fn id(path: &Path, url: &Url, format: Format) -> Self {
        let status = Status::Success;
        let path = path.display().to_string();
        let url = url.to_string();

        match format {
            Format::Plain => Msg::plain(status, &format!("{} => {}", path, url)),
            Format::Json => Msg::json(status, json!({ "path": path, "id": url })),
        }
    }

    fn error<E: Into<Error>>(error: E, format: Format) -> Msg {
        fn _error(error: Error, format: Format) -> Msg {
            let status = Status::Error;

            match format {
                Format::Plain => Msg::plain(status, &format!("error: {}", error.to_string())),
                Format::Json => Msg::json(status, json!({"error": error.to_string()})),
            }
        }

        _error(error.into(), format)
    }

    /// Construct a new plain Msg.
    fn plain(status: Status, s: &str) -> Self {
        Msg {
            content: Content::Plain(s.to_string()),
            status,
        }
    }

    /// Construct a new JSON Msg.
    fn json(status: Status, j: JsonValue) -> Self {
        Msg {
            content: Content::Json(j),
            status,
        }
    }

    /// Print the Msg to the appropriate sink.
    async fn print(self) -> Result<()> {
        let to_output = self.content.to_string();
        self.resolve_sink().write_all(to_output.as_bytes()).await?;
        Ok(())
    }

    /// Get the sink associated with the type of Msg.
    fn resolve_sink(&self) -> Box<dyn AsyncWrite + Unpin + Send> {
        match self.status {
            Status::Success => Box::new(tokio::io::stdout()),
            Status::Error => Box::new(tokio::io::stderr()),
        }
    }
}

#[derive(Debug)]
enum Content {
    Json(JsonValue),
    Plain(String),
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Content::Plain(s) => write!(f, "{}\n", s),
            Content::Json(j) => write!(f, "{}\n", j),
        }
    }
}

#[derive(Debug)]
enum Status {
    Success,
    Error,
}

/*===========================================================================
 * Command Implementations
 *-------------------------------------------------------------------------*/

/// Run the `id` subcommand.
async fn run_id(tx: &Sender<Msg>, args: &IdArgs) -> Result<()> {
    let mut file = open_async_file(&args.path).await?;

    if file_is_dir(&file).await? {
        id_directory(tx, &args.path, args.format, args.hash).await
    } else {
        id_file(tx, &mut file, &args.path, args.format, args.hash).await
    }
}

/// Run the `find` subcommand.
async fn run_find(tx: &Sender<Msg>, args: &FindArgs) -> Result<()> {
    let FindArgs { url, path, format } = args;

    let id = ArtifactId::<Sha256>::id_url(url.clone())?;
    let url = id.url();

    let mut entries = WalkDir::new(&path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(e)) => tx.send(Msg::error(e, *format)).await?,
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(&path).await?;
                let file_url = hash_file(SelectedHash::Sha256, &mut file, &path).await?;

                if url == file_url {
                    tx.send(Msg::id(&path, &url, *format)).await?;
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

/*===========================================================================
 * Helper Functions
 *-------------------------------------------------------------------------*/

// Identify, recursively, all the files under a directory.
async fn id_directory(
    tx: &Sender<Msg>,
    path: &Path,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    let mut entries = WalkDir::new(path);

    loop {
        match entries.next().await {
            None => break,
            Some(Err(e)) => tx.send(Msg::error(e, format)).await?,
            Some(Ok(entry)) => {
                let path = &entry.path();

                if entry_is_dir(&entry).await? {
                    continue;
                }

                let mut file = open_async_file(&path).await?;
                id_file(tx, &mut file, &path, format, hash).await?;
            }
        }
    }

    Ok(())
}

/// Identify a single file.
async fn id_file(
    tx: &Sender<Msg>,
    file: &mut AsyncFile,
    path: &Path,
    format: Format,
    hash: SelectedHash,
) -> Result<()> {
    let url = hash_file(hash, file, &path).await?;
    tx.send(Msg::id(path, &url, format)).await?;
    Ok(())
}

/// Hash the file and produce a `gitoid`-scheme URL.
async fn hash_file(hash: SelectedHash, file: &mut AsyncFile, path: &Path) -> Result<Url> {
    match hash {
        SelectedHash::Sha256 => sha256_id_async_file(file, &path).await.map(|id| id.url()),
    }
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
