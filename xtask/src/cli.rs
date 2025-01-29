//! The `cargo xtask` Command Line Interface (CLI).

use clap::{arg, Parser as _};

/// Define the CLI and parse arguments from the command line.
pub fn args() -> Cli {
    Cli::parse()
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Subcommand {
    /// Release a new version of a crate.
    ///
    /// Runs the following steps:
    ///
    /// (1) Verifies external tool dependencies are installed,
    /// (2) Verifies that Git worktree is ready (unless `--allow-dirty`),
    /// (3) Verifies you're on the `main` branch,
    /// (4) Verifies that `git-cliff` agrees about the version bump,
    /// (5) Generates the `CHANGELOG.md`,
    /// (6) Commits the `CHANGELOG.md`,
    /// (7) Runs a dry-run `cargo release` (unless `--execute`).
    ///
    /// Unless `--execute`, all steps will be rolled back after completion
    /// of the pipeline. All previously-executed steps will also be rolled back
    /// if a prior step fails.
    ///
    /// Note that this *does not* account for:
    ///
    /// (1) Running more than one instance of this command at the same time,
    /// (2) Running other programs which may interfere with relevant state (like
    /// Git repo state) at the same time,
    /// (3) Terminating the program prematurely, causing rollback to fail.
    ///
    /// It is your responsibility to cleanup manually if any of the above
    /// situations arise.
    Release(ReleaseArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct ReleaseArgs {
    /// The crate to release.
    #[arg(short = 'c', long = "crate", value_name = "CRATE")]
    pub krate: Crate,

    /// The version to bump.
    #[arg(short = 'b', long = "bump")]
    pub bump: Bump,

    /// Not a dry-run, actually execute the release.
    #[arg(short = 'x', long = "execute")]
    pub execute: bool,

    /// Allow Git worktree to be dirty.
    #[arg(short = 'd', long = "allow-dirty")]
    pub allow_dirty: bool,
}

/// The crate to release; can be "gitoid" or "omnibor"
#[derive(Debug, Clone, Copy, derive_more::Display, clap::ValueEnum)]
pub enum Crate {
    /// The `gitoid` crate, found in the `gitoid` folder.
    Gitoid,

    /// The `omnibor` crate, found in the `omnibor` folder.
    Omnibor,

    /// The `omnibor-cli` crate, found in the `omnibor-cli` folder.
    OmniborCli,
}

/// The version to bump; can be "major", "minor", or "patch"
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, clap::ValueEnum)]
pub enum Bump {
    /// Bump the major version.
    Major,

    /// Bump the minor version.
    Minor,

    /// Bump the patch version.
    Patch,
}
