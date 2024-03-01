//! Defines the Command Line Interface.

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use smart_default::SmartDefault;
use std::default::Default;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// How many print messages to buffer at one time, tunes printing perf
    #[arg(short = 'b', long = "buffer")]
    pub buffer: Option<usize>,
}

impl Cli {
    pub fn format(&self) -> Format {
        match &self.command {
            Command::Id(args) => args.format,
            Command::Find(args) => args.format,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// For files, prints their Artifact ID. For directories, recursively prints IDs for all files under it.
    Id(IdArgs),

    /// Find file matching an Artifact ID.
    Find(FindArgs),
}

#[derive(Debug, Args)]
pub struct IdArgs {
    /// Path to identify
    pub path: PathBuf,

    /// Output format (can be "plain", "short", or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    pub format: Format,

    /// Hash algorithm (can be "sha256")
    #[arg(short = 'H', long = "hash", default_value_t)]
    pub hash: SelectedHash,
}

#[derive(Debug, Args)]
pub struct FindArgs {
    /// `gitoid` URL to match
    pub url: Url,

    /// The root path to search under
    pub path: PathBuf,

    /// Output format (can be "plain", "short", or "json")
    #[arg(short = 'f', long = "format", default_value_t)]
    pub format: Format,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault)]
pub enum Format {
    #[default]
    Plain,
    Json,
    Short,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Format::Plain => write!(f, "plain"),
            Format::Json => write!(f, "json"),
            Format::Short => write!(f, "short"),
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Format> {
        match s {
            "plain" => Ok(Format::Plain),
            "json" => Ok(Format::Json),
            "short" => Ok(Format::Short),
            _ => Err(anyhow!("unknown format '{}'", s)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, SmartDefault)]
pub enum SelectedHash {
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
