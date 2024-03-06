use crate::{
    cli::{Bump, Crate},
    pipeline::{Pipeline, Step},
    step,
};
use anyhow::{anyhow, bail, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::ArgMatches;
use pathbuf::pathbuf;
use semver::Version;
use std::{ops::Not as _, path::PathBuf};
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
        "running 'release', bumping the {} version number for crate '{}'",
        bump,
        krate
    );

    let workspace_metadata = MetadataCommand::new().exec()?;

    let workspace_root = workspace_metadata
        .workspace_root
        .clone()
        .into_std_path_buf();

    let pkg = find_pkg(&workspace_metadata, krate)
        .ok_or_else(|| anyhow!("failed to find package in workspace"))?;

    let mut pipeline = Pipeline::new([
        step!(CheckDependencies),
        step!(CheckGitReady),
        step!(CheckGitBranch),
        step!(CheckChangelogVersionBump {
            crate_version: pkg.version.clone(),
            krate,
            bump,
        }),
        step!(GenerateChangelog {
            workspace_root: workspace_root.clone(),
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

fn find_pkg(workspace_metadata: &Metadata, krate: Crate) -> Option<&Package> {
    for id in &workspace_metadata.workspace_members {
        let pkg = &workspace_metadata[id];

        if pkg.name == krate.name() {
            return Some(pkg);
        }
    }

    None
}

struct CheckDependencies;

impl Step for CheckDependencies {
    fn name(&self) -> &'static str {
        "check-dependencies"
    }

    fn run(&mut self) -> Result<()> {
        let missing_cmds = [
            // Version control
            "git",
            // Rust package management tool.
            "cargo",
            // Automatically produces CHANGELOG.md.
            "git-cliff",
            // Publishes new versions to Crates.io, tags releases, and commits to git.
            "cargo-release",
        ]
        .into_iter()
        .inspect(|name| log::info!("checking command '{}'", name))
        .filter(|name| which::which(name).is_err())
        .collect::<Vec<_>>();

        if missing_cmds.is_empty().not() {
            let commands = missing_cmds.join(", ");
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
        let _ = cmd!(sh, "git update-index -q --ignore-submodules --refresh").run();

        // 2. Check for unstaged changes in the working tree.
        if cmd!(sh, "git diff-files --quiet --ignore-submodules")
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

        let current_branch = cmd!(sh, "git rev-parse --abbrev-ref HEAD").read()?;

        if current_branch != "main" {
            bail!(
                "release must be run on the main branch; change branch to 'main' before continuing"
            );
        }

        Ok(())
    }
}

struct CheckChangelogVersionBump {
    crate_version: Version,
    krate: Crate,
    bump: Bump,
}

impl CheckChangelogVersionBump {
    fn config(&self) -> PathBuf {
        pathbuf!["Cargo.toml"]
    }

    fn include(&self) -> PathBuf {
        pathbuf![self.krate.name(), "*"]
    }
}

impl Step for CheckChangelogVersionBump {
    fn name(&self) -> &'static str {
        "check-changelog-version-bump"
    }

    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let config = self.config();
        let include = self.include();
        let current = &self.crate_version;
        let bumped = {
            let raw = cmd!(
                sh,
                "git cliff --config {config} --include-path {include} --bumped-version"
            )
            .read()?;
            let prefix = format!("{}-v", self.krate.name());
            let stripped = raw.strip_prefix(&prefix).unwrap_or(&raw);
            Version::parse(stripped)?
        };

        let bumped = detect_bumped_version(current, &bumped)?;

        if bumped != self.bump {
            bail!(
                "git-cliff disagrees about version bump: git-cliff: {}, requested: {}",
                bumped,
                self.bump
            );
        }

        Ok(())
    }
}

fn detect_bumped_version(current: &Version, bumped: &Version) -> Result<Bump> {
    match (
        bumped.major > current.major,
        bumped.minor > current.minor,
        bumped.patch > current.patch,
    ) {
        (true, false, false) => Ok(Bump::Major),
        (false, true, false) => Ok(Bump::Minor),
        (false, false, true) => Ok(Bump::Patch),
        _ => bail!("can't bump more than one version number at a time"),
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
        .run()?;
        Ok(())
    }

    // Delete the generated CHANGELOG file.
    fn undo(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let output = self.output();
        cmd!(sh, "rm -f {output}").run()?;
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
        cmd!(sh, "git add {changelog}").run()?;
        cmd!(sh, "git commit --signoff -m {msg}").run()?;
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let last_msg = cmd!(sh, "git log -1 --pretty=%s").read()?;
        let expected_msg = self.commit_msg();

        if last_msg != expected_msg {
            bail!("last commit isn't CHANGELOG commit; aborting to avoid breaking git history");
        }

        cmd!(sh, "git reset --hard HEAD~1").run()?;
        Ok(())
    }
}

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
        let sh = Shell::new()?;
        let krate = self.krate.name();
        let bump = self.bump.to_string();
        let execute = self.execute.then_some("--execute");
        cmd!(
            sh,
            "cargo release -p {krate} --allow-branch main {execute...} {bump}"
        )
        .run()?;
        Ok(())
    }

    // No `undo` method implemented here, because without `--execute`, `cargo release`
    // avoids doing anything permanent to the state of the environment. If `--execute`
    // is enabled, then it'll run, _and_ we won't need to force-rollback at the end
    // because that requires `execute == false`. So either way, no undo is required.
}
