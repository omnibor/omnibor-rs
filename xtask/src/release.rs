//! The `cargo xtask release` subcommand.

use crate::{
    cli::{Bump, Crate, ReleaseArgs},
    pipeline,
    pipeline::{Pipeline, Step},
};
use anyhow::{anyhow, bail, Context as _, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use pathbuf::pathbuf;
use semver::Version;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    env::var as env_var,
    ops::Not as _,
    path::{Path, PathBuf},
};
use toml::from_str as from_toml_str;
use xshell::{cmd, Shell};

/// Run the release command.
pub fn run(args: &ReleaseArgs) -> Result<()> {
    log::info!(
        "running 'release', bumping the {} version number for crate '{}'",
        args.bump,
        args.krate
    );

    let workspace_metadata = MetadataCommand::new().exec()?;

    let workspace_root = workspace_metadata
        .workspace_root
        .clone()
        .into_std_path_buf();

    let pkg = find_pkg(&workspace_metadata, args.krate)
        .ok_or_else(|| anyhow!("failed to find package in workspace"))?;

    let mut pipeline = pipeline!(
        CheckDependencies,
        CheckGitReady {
            allow_dirty: args.allow_dirty
        },
        CheckGitBranch,
        CheckChangelogVersionBump {
            workspace_root: workspace_root.clone(),
            crate_version: pkg.version.clone(),
            krate: args.krate,
            bump: args.bump,
        },
        GenerateChangelog {
            workspace_root: workspace_root.clone(),
            krate: args.krate
        },
        CommitChangelog { krate: args.krate },
        ReleaseCrate {
            krate: args.krate,
            bump: args.bump,
            execute: args.execute
        }
    );

    // If we're not executing, force a rollback at the end.
    if args.execute.not() {
        pipeline.plan_forced_rollback();
    }

    pipeline.run()
}

/// Get the information for a specific package.
fn find_pkg(workspace_metadata: &Metadata, krate: Crate) -> Option<&Package> {
    for id in &workspace_metadata.workspace_members {
        let pkg = &workspace_metadata[id];

        if pkg.name == krate.to_string() {
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
        .inspect(|name| log::info!("checking command '{name}'"))
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

struct CheckGitReady {
    allow_dirty: bool,
}

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
            let msg =
                "unstaged changes detected in the working tree; commit or stash before continuing";

            if self.allow_dirty.not() {
                bail!(msg);
            }

            log::warn!("{msg}");
        }

        // 3. Check for uncommitted changes in the index.
        if cmd!(
            sh,
            "git diff-index --cached --quiet HEAD --ignore-submodules"
        )
        .run()
        .is_err()
        {
            let msg =
                "uncommitted changes detected in the index; commit or stash before continuing";

            if self.allow_dirty.not() {
                bail!(msg);
            }

            log::warn!("{msg}");
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
    workspace_root: PathBuf,
    crate_version: Version,
    krate: Crate,
    bump: Bump,
}

impl CheckChangelogVersionBump {
    fn config(&self) -> PathBuf {
        pathbuf!["Cargo.toml"]
    }

    fn include(&self) -> PathBuf {
        pathbuf![&self.krate.to_string(), "*"]
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

        sh.change_dir(&self.workspace_root);

        let bumped = {
            let raw = cmd!(
                sh,
                "git cliff --config {config} --include-path {include} --bumped-version"
            )
            .read()?;

            let version = raw
                .rsplit('-')
                .next()
                .ok_or_else(|| anyhow!("expected a version number at the end"))?;

            let stripped = version.strip_prefix('v').unwrap_or(version);
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
        pathbuf![&self.krate.to_string(), "*"]
    }

    fn output(&self) -> PathBuf {
        pathbuf![&self.krate.to_string(), "CHANGELOG.md"]
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
            "git cliff --config {config} --bump --include-path {include} -o {output}"
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
        format!("chore: update `{}` crate CHANGELOG.md", self.krate)
    }
}

impl Step for CommitChangelog {
    fn name(&self) -> &'static str {
        "commit-changelog"
    }

    fn run(&mut self) -> Result<()> {
        let sh = Shell::new()?;
        let msg = self.commit_msg();
        let changelog = pathbuf![&self.krate.to_string(), "CHANGELOG.md"];
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
        let krate = self.krate.to_string();
        let bump = self.bump.to_string();

        let execute = if self.execute {
            &["--execute", "--no-confirm"][..]
        } else {
            &[]
        };

        let token = discover_cargo_auth_token(&sh)?;
        let _env = sh.push_env("CARGO_REGISTRY_TOKEN", &token);

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

/// Find the Cargo authentication token.
///
/// This involves finding the credentials.toml file that Cargo writes
/// its authentication token into, parsing it, and extracting the
/// token to be returned.
///
/// Note that this only correctly handles the Crates.io registry.
fn discover_cargo_auth_token(sh: &Shell) -> Result<String> {
    /// Read a file from the shell.
    ///
    /// This function is only provided to coerce the error type into `anyhow::Error`
    fn read_file(sh: &Shell, path: &Path) -> Result<String> {
        Ok(sh.read_file(path)?)
    }

    /// Parse a string into a TOML-compatible struct.
    ///
    /// This function is only provided to coerce the error type into `anyhow::Error`
    fn parse_toml<T: DeserializeOwned>(s: String) -> Result<T> {
        Ok(from_toml_str::<T>(&s)?)
    }

    /// Simple struct representing the Cargo credentials file.
    #[derive(Deserialize)]
    struct Credentials {
        registry: CredentialsRegistry,
    }

    #[derive(Deserialize)]
    struct CredentialsRegistry {
        token: String,
    }

    let cargo_home = env_var("CARGO_HOME").context("failed to find 'CARGO_HOME'")?;
    let path = pathbuf![&cargo_home, "credentials.toml"];

    let creds: Credentials = read_file(sh, &path)
        .and_then(parse_toml)
        .context(format!("failed to parse '{}'", path.display()))?;

    Ok(creds.registry.token)
}
