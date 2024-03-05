use crate::cli::{Bump, Crate};
use crate::pipeline::{Pipeline, Step};
use crate::step;
use anyhow::{anyhow, bail, Error, Result};
use cargo_metadata::MetadataCommand;
use clap::ArgMatches;
use pathbuf::pathbuf;
use std::ops::Not as _;
use std::path::PathBuf;
use xshell::{cmd, Shell};

/// Run the release command.
pub fn run(args: &ArgMatches) -> Result<()> {
    let krate: Crate = *args
        .get_one("crate")
        .ok_or_else(|| anyhow!("'--crate' is a required argument"))?;

    let bump: Bump = *args
        .get_one("bump")
        .ok_or_else(|| anyhow!("'--bump' is a required argument"))?;

    let execute: bool = *args
        .get_one("execute")
        .expect("--execute has a default value, so it should never be missing");

    log::info!(
        "running 'release', bumping the {} number for crate '{}'",
        bump,
        krate
    );

    let workspace_root = workspace_root()?;

    let mut pipeline = Pipeline::new([
        step!(CheckDependencies),
        step!(CheckGitReady),
        step!(CheckGitBranch),
        step!(GenerateChangelog {
            workspace_root,
            krate
        }),
        step!(CommitChangelog { krate }),
        step!(ReleaseCrate {
            krate,
            bump,
            execute
        }),
    ]);

    // If we're not executing, force a rollback at the end.
    if execute.not() {
        pipeline.force_rollback();
    }

    pipeline.run()
}

struct CheckDependencies;

impl Step for CheckDependencies {
    fn name(&self) -> &'static str {
        "check-dependencies"
    }

    fn run(&mut self) -> Result<()> {
        let mut missing_cmds = Vec::new();

        check_cmd(&mut missing_cmds, "git");
        check_cmd(&mut missing_cmds, "git-cliff");
        check_cmd(&mut missing_cmds, "cargo");
        check_cmd(&mut missing_cmds, "cargo-release");

        if missing_cmds.is_empty().not() {
            let commands = missing_cmds
                .iter()
                .map(|i| i.name)
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "missing commands: {}; please install before continuing",
                commands
            );
        }

        Ok(())
    }
}

struct CheckGitReady;

impl Step for CheckGitReady {
    fn name(&self) -> &'static str {
        "check-git-ready"
    }

    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;

        // 1. Make sure the index is up to date (ignore errors).
        let _ = cmd!(sh, "git update-index -q --ignore-submodules --refresh")
            .quiet()
            .run();

        // 2. Check for unstaged changes in the working tree.
        if cmd!(sh, "git diff-files --quiet --ignore-submodules")
            .quiet()
            .run()
            .is_err()
        {
            bail!(
                "unstaged changes detected in the working tree; commit or stash before continuing"
            );
        }

        // 3. Check for uncommitted changes in the index.
        if cmd!(
            sh,
            "git diff-index --cached --quiet HEAD --ignore-submodules"
        )
        .quiet()
        .run()
        .is_err()
        {
            bail!("uncommitted changes detected in the index; commit or stash before continuing");
        }

        Ok(())
    }
}

struct CheckGitBranch;

impl Step for CheckGitBranch {
    fn name(&self) -> &'static str {
        "check-git-branch"
    }

    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;

        let current_branch = cmd!(sh, "git rev-parse --abbrev-ref HEAD").quiet().read()?;

        if current_branch != "main" {
            bail!(
                "release must be run on the main branch; change branch to 'main' before continuing"
            );
        }

        Ok(())
    }
}

struct GenerateChangelog {
    workspace_root: PathBuf,
    krate: Crate,
}

impl GenerateChangelog {
    fn config(&self) -> PathBuf {
        pathbuf!["Cargo.toml"]
    }

    fn include(&self) -> PathBuf {
        pathbuf![self.krate.name(), "*"]
    }

    fn output(&self) -> PathBuf {
        pathbuf![self.krate.name(), "CHANGELOG.md"]
    }
}

impl Step for GenerateChangelog {
    fn name(&self) -> &'static str {
        "generate-changelog"
    }

    // Generate the CHANGELOG file for the specified crate.
    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let config = self.config();
        let include = self.include();
        let output = self.output();
        sh.change_dir(&self.workspace_root);
        cmd!(
            sh,
            "git cliff --config {config} --include-path {include} -o {output}"
        )
        .quiet()
        .run()?;
        Ok(())
    }

    // Delete the generated CHANGELOG file.
    fn undo(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let output = self.output();
        cmd!(sh, "rm {output}").quiet().run()?;
        Ok(())
    }
}

struct CommitChangelog {
    krate: Crate,
}

impl CommitChangelog {
    fn commit_msg(&self) -> String {
        format!("chore: update `{}` crate CHANGELOG.md", self.krate.name())
    }
}

impl Step for CommitChangelog {
    fn name(&self) -> &'static str {
        "commit-changelog"
    }

    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let msg = self.commit_msg();
        let changelog = pathbuf![self.krate.name(), "CHANGELOG.md"];
        cmd!(sh, "git add {changelog}").quiet().run()?;
        cmd!(sh, "git commit --signoff -m \"{msg}\"")
            .quiet()
            .run()?;
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let last_msg = cmd!(sh, "git log -1 --pretty=%s").quiet().read()?;
        let expected_msg = self.commit_msg();

        if last_msg != expected_msg {
            bail!("last commit isn't CHANGELOG commit; aborting to avoid breaking git history");
        }

        cmd!(sh, "git reset --hard HEAD~1").quiet().run()?;
        Ok(())
    }
}

#[allow(unused)]
struct ReleaseCrate {
    krate: Crate,
    bump: Bump,
    execute: bool,
}

impl Step for ReleaseCrate {
    fn name(&self) -> &'static str {
        "release-crate"
    }

    fn run(&mut self) -> Result<()> {
        bail!("not yet implemented");
    }
}

/// Check if a command exists on the command line.
fn check_cmd(missing_cmds: &mut Vec<MissingCmd>, name: &'static str) {
    if let Err(err) = which::which(name) {
        let err = anyhow!(err);
        missing_cmds.push(MissingCmd { name, err });
    }
}

#[derive(Debug)]
struct MissingCmd {
    name: &'static str,
    #[allow(unused)]
    err: Error,
}

// Figure out the root of the current Cargo workspace.
fn workspace_root() -> Result<PathBuf> {
    let metadata = MetadataCommand::new().exec()?;
    let root = metadata.workspace_root.into_std_path_buf();
    Ok(root)
}
