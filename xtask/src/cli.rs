//! The `cargo xtask` Command Line Interface (CLI).

use clap::{arg, builder::PossibleValue, value_parser, ArgMatches, Command, ValueEnum};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Define the CLI and parse arguments from the command line.
pub fn args() -> ArgMatches {
    Command::new("xtask")
        .about("Task runner for the OmniBOR Rust workspace")
        .help_expected(true)
        .subcommand(
            Command::new("release")
                .about("Release a new version of a workspace crate")
                .arg(
                    arg!(-c --crate <CRATE>)
                        .required(true)
                        .value_parser(value_parser!(Crate))
                        .help("the crate to release"),
                )
                .arg(
                    arg!(-b --bump <BUMP>)
                        .required(true)
                        .value_parser(value_parser!(Bump))
                        .help("the version to bump"),
                )
                .arg(
                    arg!(-x - -execute)
                        .required(false)
                        .default_value("false")
                        .value_parser(value_parser!(bool))
                        .help("not a dry run, actually execute the release"),
                )
                .arg(
                    arg!(--"allow-dirty")
                        .required(false)
                        .default_value("false")
                        .value_parser(value_parser!(bool))
                        .help("allow Git worktree to be dirty"),
                ),
        )
        .get_matches()
}

/// The crate to release; can be "gitoid" or "omnibor"
#[derive(Debug, Clone, Copy)]
pub enum Crate {
    /// The `gitoid` crate, found in the `gitoid` folder.
    GitOid,

    /// The `omnibor` crate, found in the `omnibor` folder.
    OmniBor,

    /// The `omnibor-cli` crate, found in the `omnibor-cli` folder.
    OmniBorCli,
}

impl Crate {
    /// Get the name of the crate, as it should be shown in the CLI
    /// and as it exists on the filesystem.
    pub fn name(&self) -> &'static str {
        match self {
            Crate::GitOid => "gitoid",
            Crate::OmniBor => "omnibor",
            Crate::OmniBorCli => "omnibor-cli",
        }
    }
}

impl Display for Crate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.name())
    }
}

// This is needed for `clap` to be able to parse string values
// into this `Crate` enum.
impl ValueEnum for Crate {
    fn value_variants<'a>() -> &'a [Self] {
        &[Crate::GitOid, Crate::OmniBor, Crate::OmniBorCli]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(self.name()))
    }
}

/// The version to bump; can be "major", "minor", or "patch"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    /// Bump the major version.
    Major,

    /// Bump the minor version.
    Minor,

    /// Bump the patch version.
    Patch,
}

impl Display for Bump {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Bump::Major => write!(f, "major"),
            Bump::Minor => write!(f, "minor"),
            Bump::Patch => write!(f, "patch"),
        }
    }
}
// This is needed for `clap` to be able to parse string values
// into this `Bump` enum.
impl ValueEnum for Bump {
    fn value_variants<'a>() -> &'a [Self] {
        &[Bump::Major, Bump::Minor, Bump::Patch]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Bump::Major => PossibleValue::new("major"),
            Bump::Minor => PossibleValue::new("minor"),
            Bump::Patch => PossibleValue::new("patch"),
        })
    }
}
