use crate::pipeline::{self, Step};
use crate::step;
use crate::cli::{Crate, Bump};
use anyhow::{anyhow, Result};
use clap::ArgMatches;

/*
    # Run `git-cliff` to generate a changelog.
    # Commit the changelog w/ commit msg in Conventional Commit fmt.
    # Run `cargo-release` to release the new version.
    # If anything fails, rollback prior steps in reverse order.
    # Probably good for each step to have a "do" and "undo" operation.
    #
    # ... In fact I'll probably write this in Rust lol.

    # Need:
    #
    # - git
    # - git-cliff
    # - cargo
    # - cargo-release
 */


/// Run the release command.
pub fn run(args: &ArgMatches) -> Result<()> {
    let krate: Crate = *args.get_one("crate").ok_or_else(|| anyhow!("'--crate' is a required argument"))?;
    let bump: Bump = *args.get_one("bump").ok_or_else(|| anyhow!("'--bump' is a required argument"))?;

    log::info!("running 'release', bumping the {} number for crate '{}'", bump, krate);

    pipeline::run([
        step!(CheckDependencies),
        step!(GenerateChangelog),
        step!(CommitChangelog),
        step!(ReleaseCrate),
    ])
}

struct CheckDependencies;

impl Step for CheckDependencies {
    fn name(&self) -> &'static str {
        "check-dependencies"
    }

    fn run(&mut self) -> Result<()> {
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        Ok(())
    }
}

struct GenerateChangelog;

impl Step for GenerateChangelog {
    fn name(&self) -> &'static str {
        "generate-changelog"
    }

    fn run(&mut self) -> Result<()> {
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        Ok(())
    }
}

struct CommitChangelog;

impl Step for CommitChangelog {
    fn name(&self) -> &'static str {
        "commit-changelog"
    }

    fn run(&mut self) -> Result<()> {
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        Ok(())
    }
}

struct ReleaseCrate;

impl Step for ReleaseCrate {
    fn name(&self) -> &'static str {
        "release-crate"
    }

    fn run(&mut self) -> Result<()> {
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        Ok(())
    }
}
