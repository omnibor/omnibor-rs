//! Defines the Command Line Interface.

use omnibor::hashes::Sha256;
use omnibor::ArtifactId;
use refurb::Update;
use std::default::Default;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Default, clap::Parser, Update)]
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
        help_heading = "Performance Tuning Flags",
        long_help = "How many print messages to buffer at once. Can also be set with the `OMNIBOR_BUFFER` environment variable"
    )]
    buffer: Option<usize>,

    /// Output format
    #[arg(
        short = 'f',
        long = "format",
        global = true,
        help_heading = "Output Flags",
        long_help = "Output format. Can also be set with the `OMNIBOR_FORMAT` environment variable"
    )]
    format: Option<Format>,

    /// Hash algorithm to use when parsing Artifact IDs
    #[arg(
        short = 'H',
        long = "hash",
        global = true,
        help_heading = "Input Flags",
        long_help = "Hash algorithm to use when parsing Artifact IDs. Can also be set with the `OMNIBOR_HASH` environment variable"
    )]
    hash: Option<SelectedHash>,

    /// Directory to store manifests.
    #[arg(
        short = 'd',
        long = "dir",
        global = true,
        help_heading = "Storage Flags",
        long_help = "Directory to store manifests. Can also be set with the `OMNIBOR_DIR` environment variable"
    )]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Config {
    /// Load configuration from the environment and CLI args.
    pub fn load() -> Config {
        let mut config = Config::default();
        config.update(&Config::from_env());
        config.update(&Config::from_args());
        config
    }

    /// Load configuration from the environment.
    fn from_env() -> Config {
        Config {
            buffer: env_var_by_key("buffer"),
            format: env_var_by_key("format"),
            hash: env_var_by_key("hash"),
            dir: env_var_by_key("dir"),
            command: None,
        }
    }

    /// Load configuration from CLI args.
    fn from_args() -> Config {
        <Config as clap::Parser>::parse()
    }

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

    /// Get the selected subcommand.
    pub fn command(&self) -> &Command {
        self.command.as_ref().unwrap()
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it.
    Id(IdArgs),

    /// Find file matching an Artifact ID.
    Find(FindArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct IdArgs {
    /// Path to identify
    #[arg(short = 'p', long = "path", help_heading = "Input Flags")]
    pub path: PathBuf,
}

#[derive(Debug, Clone, clap::Args)]
pub struct FindArgs {
    /// Artifact ID to match
    #[arg(short = 'a', long = "aid", help_heading = "Input Flags")]
    pub aid: ArtifactId<Sha256>,

    /// The root path to search under
    #[arg(short = 'p', long = "path", help_heading = "Input Flags")]
    pub path: PathBuf,
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

impl FromStr for SelectedHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ignore_case = true;
        <Self as clap::ValueEnum>::from_str(s, ignore_case)
    }
}

/// Get an environment variable with the given key.
fn env_var_by_key<T: FromStr>(name: &'static str) -> Option<T> {
    let key = format!("OMNIBOR_{}", name.to_uppercase());

    match std::env::var(&key) {
        Ok(val) => val.parse().ok(),
        Err(_) => None,
    }
}
