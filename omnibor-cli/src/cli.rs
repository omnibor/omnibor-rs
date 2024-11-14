//! Defines the Command Line Interface.

use crate::error::Error;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use omnibor::{hashes::Sha256, ArtifactId, IntoArtifactId};
use pathbuf::pathbuf;
use std::{
    default::Default,
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

// The default root directory for OmniBOR.
// We use a `static` here to make sure we can safely give out
// references to it.
static DEFAULT_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

#[derive(Debug, Default, clap::Parser)]
#[command(
    name = "omnibor",
    about,
    version,
    propagate_version = true,
    arg_required_else_help = true,
    subcommand_required = true
)]
pub struct Config {
    /// How many print messages to buffer at once
    #[arg(
        short = 'b',
        long = "buffer",
        default_value = "100",
        global = true,
        env = "OMNIBOR_BUFFER",
        help_heading = "General Flags"
    )]
    buffer: Option<usize>,

    /// Output format
    #[arg(
        short = 'f',
        long = "format",
        global = true,
        env = "OMNIBOR_FORMAT",
        help_heading = "General Flags"
    )]
    format: Option<Format>,

    /// Hash algorithm to use when parsing Artifact IDs
    #[arg(
        short = 'H',
        long = "hash",
        global = true,
        env = "OMNIBOR_HASH",
        help_heading = "General Flags"
    )]
    hash: Option<SelectedHash>,

    /// Directory to store manifests.
    #[arg(
        short = 'd',
        long = "dir",
        global = true,
        env = "OMNIBOR_DIR",
        help_heading = "General Flags"
    )]
    dir: Option<PathBuf>,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Config {
    /// Get the configured buffer size.
    pub fn buffer(&self) -> usize {
        self.buffer.unwrap()
    }

    /// Get the selected format.
    pub fn format(&self) -> Format {
        self.format.unwrap_or_default()
    }

    /// Get the selected hash algorithm.
    pub fn hash(&self) -> SelectedHash {
        self.hash.unwrap_or_default()
    }

    /// Get the chosen OmniBOR root directory.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref().or_else(|| {
            DEFAULT_DIR
                .get_or_init(|| dirs::data_dir().map(|cache_dir| pathbuf![&cache_dir, "omnibor"]))
                .as_deref()
        })
    }

    /// Get the selected subcommand.
    pub fn command(&self) -> &Command {
        self.command.as_ref().unwrap()
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Actions related to Artifact Identifiers.
    Artifact(ArtifactArgs),

    /// Actions related to Input Manifests.
    Manifest(ManifestArgs),

    /// Actions to help debug the OmniBOR CLI.
    Debug(DebugArgs),
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ArtifactArgs {
    #[clap(subcommand)]
    command: Option<ArtifactCommand>,
}

impl ArtifactArgs {
    pub fn command(&self) -> ArtifactCommand {
        self.command.clone().unwrap()
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ArtifactCommand {
    /// For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it.
    Id(IdArgs),

    /// Find file matching an Artifact ID.
    Find(FindArgs),
}

#[derive(Debug, Clone, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct IdArgs {
    /// Path to identify
    #[arg(short = 'p', long = "path", help_heading = "Important Flags")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct FindArgs {
    /// Artifact ID to match
    #[arg(short = 'a', long = "aid", help_heading = "Important Flags")]
    pub aid: ArtifactId<Sha256>,

    /// The root path to search under
    #[arg(short = 'p', long = "path", help_heading = "Important Flags")]
    pub path: PathBuf,
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ManifestArgs {
    #[clap(subcommand)]
    command: ManifestCommand,
}

impl ManifestArgs {
    pub fn command(&self) -> &ManifestCommand {
        &self.command
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum ManifestCommand {
    /// Add a new manifest to the store.
    Add(ManifestAddArgs),
    /// Remove a manifest from the store.
    Remove(ManifestRemoveArgs),
    /// Create a new manifest and add it to the store
    Create(ManifestCreateArgs),
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ManifestAddArgs {}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ManifestRemoveArgs {}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ManifestCreateArgs {
    /// Inputs to record in the manifest
    #[arg(short = 'i', long = "input", help_heading = "Important Flags")]
    pub inputs: Vec<IdentifiableArg>,

    /// The tool that built the target artifact
    #[arg(short = 'B', long = "built-by", help_heading = "Important Flags")]
    pub built_by: Option<IdentifiableArg>,

    /// The target the manifest is describing
    #[arg(short = 't', long = "target", help_heading = "Important Flags")]
    pub target: PathBuf,
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct DebugArgs {
    #[clap(subcommand)]
    command: Option<DebugCommand>,
}

impl DebugArgs {
    pub fn command(&self) -> DebugCommand {
        self.command.as_ref().unwrap().clone()
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum DebugCommand {
    Config,
}

#[derive(Debug, Clone)]
pub enum IdentifiableArg {
    /// An Artifact ID
    ArtifactId(ArtifactId<Sha256>),
    /// A path to a file
    Path(PathBuf),
}

impl FromStr for IdentifiableArg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match (ArtifactId::from_str(s), PathBuf::from_str(s)) {
            (Ok(aid), _) => Ok(IdentifiableArg::ArtifactId(aid)),
            (_, Ok(path)) => Ok(IdentifiableArg::Path(path)),
            (Err(_), Err(_)) => Err(Error::NotIdentifiable(s.to_string())),
        }
    }
}

impl IntoArtifactId<Sha256> for IdentifiableArg {
    fn into_artifact_id(self) -> Result<ArtifactId<Sha256>, omnibor::Error> {
        match self {
            IdentifiableArg::ArtifactId(aid) => Ok(aid),
            IdentifiableArg::Path(path) => path.into_artifact_id(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Format {
    /// A human-readable plaintext format
    #[default]
    Plain,
    /// JSON format
    Json,
    /// Shortest possible format (ideal for piping to other commands)
    Short,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Plain => write!(f, "plain"),
            Format::Json => write!(f, "json"),
            Format::Short => write!(f, "short"),
        }
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ignore_case = true;
        <Self as clap::ValueEnum>::from_str(s, ignore_case)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum SelectedHash {
    /// SHA-256 hash
    #[default]
    Sha256,
}

impl Display for SelectedHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectedHash::Sha256 => write!(f, "sha256"),
        }
    }
}

impl FromStr for SelectedHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ignore_case = true;
        <Self as clap::ValueEnum>::from_str(s, ignore_case)
    }
}
