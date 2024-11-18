//! Defines the Command Line Interface.

use crate::error::Error;
use clap::{builder::PossibleValue, ValueEnum};
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

// We use `static`s here to make sure we can safely give out
// references to these values.

// The default root directory.
static DEFAULT_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
// The default config path.
pub static DEFAULT_CONFIG: OnceLock<Option<PathBuf>> = OnceLock::new();

// Help headings
const IMPORTANT: &str = "Important Flags";

#[derive(Debug, Default, clap::Parser)]
#[command(
    name = "omnibor",
    about,
    version,
    propagate_version = true,
    arg_required_else_help = true,
    subcommand_required = true
)]
pub struct Args {
    /// Output format
    #[arg(short = 'f', long = "format", global = true, env = "OMNIBOR_FORMAT")]
    format: Option<Format>,

    /// Directory to store manifests.
    #[arg(short = 'd', long = "dir", global = true, env = "OMNIBOR_DIR")]
    dir: Option<PathBuf>,

    /// Path to a configuration file.
    #[arg(short = 'c', long = "config", global = true, env = "OMNIBOR_CONFIG")]
    config: Option<PathBuf>,

    /// Turn on 'tokio-console' debug integration.
    #[arg(
        short = 'D',
        long = "debug-console",
        default_value_t = false,
        global = true,
        env = "OMNIBOR_DEBUG_CONSOLE"
    )]
    console: bool,

    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Args {
    /// Get the selected format.
    pub fn format(&self) -> Format {
        self.format.unwrap_or_default()
    }

    /// Get the selected verbosity.
    pub fn verbosity(&self) -> Verbosity<InfoLevel> {
        self.verbosity.clone()
    }

    /// Get whether to turn on `tokio-console` integration.
    pub fn console(&self) -> bool {
        self.console
    }

    /// Get the chosen OmniBOR root directory.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref().or_else(|| {
            DEFAULT_DIR
                .get_or_init(|| dirs::data_dir().map(|cache_dir| pathbuf![&cache_dir, "omnibor"]))
                .as_deref()
        })
    }

    /// Get the chosen configuration file.
    pub fn config(&self) -> Option<&Path> {
        self.config.as_deref().or_else(|| {
            DEFAULT_CONFIG
                .get_or_init(|| self.dir().map(|root| pathbuf![root, "config.json"]))
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

    /// Actions related to the filesystem store.
    Store(StoreArgs),

    /// Actions to help debug the OmniBOR CLI.
    Debug(DebugArgs),
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ArtifactArgs {
    #[clap(subcommand)]
    pub command: ArtifactCommand,
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
    #[arg(short = 'p', long = "path", help_heading = IMPORTANT)]
    pub path: PathBuf,

    /// Hash algorithm to use for Artifact IDs.
    #[arg(
        short = 'H',
        long = "hash",
        help_heading = IMPORTANT
    )]
    hash: Option<SelectedHash>,
}

impl IdArgs {
    /// Get the hash to use.
    pub fn hash(&self) -> SelectedHash {
        self.hash.unwrap_or_default()
    }
}

#[derive(Debug, Clone, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct FindArgs {
    /// Artifact ID to match
    #[arg(short = 'a', long = "aid", help_heading = IMPORTANT)]
    pub aid: ArtifactId<Sha256>,

    /// The root path to search under
    #[arg(short = 'p', long = "path", help_heading = IMPORTANT)]
    pub path: PathBuf,
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct ManifestArgs {
    #[clap(subcommand)]
    pub command: ManifestCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ManifestCommand {
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
    /// Inputs to record in the manifest.
    #[arg(short = 'i', long = "input", help_heading = IMPORTANT)]
    pub inputs: Vec<IdentifiableArg>,

    /// The tool that built the target artifact.
    #[arg(short = 'B', long = "built-by", help_heading = IMPORTANT)]
    pub built_by: Option<IdentifiableArg>,

    /// The target the manifest is describing.
    #[arg(short = 't', long = "target", help_heading = IMPORTANT)]
    pub target: PathBuf,

    /// Do not store the manifest in the local store.
    #[arg(long = "no-store", help_heading = IMPORTANT)]
    pub no_store: bool,

    /// Do not write the manifest to a local directory.
    #[arg(long = "no-out", help_heading = IMPORTANT)]
    pub no_out: bool,

    /// Directory to write manifest out to.
    #[arg(short = 'o', long = "output", help_heading = IMPORTANT, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Hash algorithm to use for Artifact IDs.
    #[arg(short = 'H', long = "hash", env = "OMNIBOR_HASH", help_heading = IMPORTANT)]
    pub hash: Option<SelectedHash>,
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct StoreArgs {
    #[clap(subcommand)]
    pub command: StoreCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum StoreCommand {
    /// Add an Input Manifest to the store.
    Add(StoreAddArgs),
    /// Remove an Input Manifest from the store.
    Remove(StoreRemoveArgs),
    /// Review the log of changes to the store.
    Log(StoreLogArgs),
}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct StoreAddArgs {}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct StoreRemoveArgs {}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct StoreLogArgs {}

#[derive(Debug, clap::Args)]
#[command(arg_required_else_help = true)]
pub struct DebugArgs {
    #[clap(subcommand)]
    pub command: DebugCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum DebugCommand {
    Paths(DebugPathsArgs),
}

#[derive(Debug, clap::Args)]
pub struct DebugPathsArgs {
    /// Names of specific paths to get.
    #[arg(
        short = 'k',
        long = "keys",
        help_heading = IMPORTANT
    )]
    pub keys: Vec<String>,
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

// Helper macro, generates `Display` and `FromStr` impls for any type that
// implements `clap::ValueEnum`, delegating to `ValueEnum` functions.
macro_rules! to_and_from_string {
    ($name:ident) => {
        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", possible_value(self.to_possible_value()))
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let ignore_case = true;
                ValueEnum::from_str(s, ignore_case)
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum Format {
    /// A human-readable plaintext format
    #[default]
    Plain,
    /// Shortest possible format (ideal for piping to other commands)
    Short,
    /// JSON format
    Json,
}

to_and_from_string!(Format);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum SelectedHash {
    /// SHA-256 hash
    #[default]
    Sha256,
}

to_and_from_string!(SelectedHash);

fn possible_value(value: Option<PossibleValue>) -> String {
    match value {
        Some(value) => value.get_name().to_string(),
        None => String::from("<skipped>"),
    }
}
