use clap::{arg, builder::PossibleValue, value_parser, ArgMatches, Command, ValueEnum};
use std::fmt::{Display, Formatter, Result as FmtResult};

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
                    arg!(--execute)
                        .required(false)
                        .value_parser(value_parser!(bool))
                        .help("not a dry run, actually execute the release"),
                ),
        )
        .get_matches()
}

/// The crate to release; can be "gitoid" or "omnibor"
#[derive(Debug, Clone, Copy)]
pub enum Crate {
    GitOid,
    OmniBor,
}

impl Display for Crate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Crate::GitOid => write!(f, "gitoid"),
            Crate::OmniBor => write!(f, "omnibor"),
        }
    }
}

impl ValueEnum for Crate {
    fn value_variants<'a>() -> &'a [Self] {
        &[Crate::GitOid, Crate::OmniBor]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Crate::GitOid => PossibleValue::new("gitoid"),
            Crate::OmniBor => PossibleValue::new("omnibor"),
        })
    }
}

/// The version to bump; can be "major", "minor", or "patch"
#[derive(Debug, Clone, Copy)]
pub enum Bump {
    Major,
    Minor,
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
