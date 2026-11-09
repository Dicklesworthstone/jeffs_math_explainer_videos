//! Host-only Cargo rebuilds for disposable Studio workers (§13.3).
//!
//! This is a development-host capability, never a Scene or renderer hook.
//! It selects one Cargo package and binary/example, uses the existing bounded
//! exact-image process substrate, and publishes an immutable executable copy.
//! Failed builds cannot overwrite the image used by the last healthy worker.
//! Cargo projects (including build scripts/proc macros) must be trusted code.

mod artifact;

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fmn_platform::process::{ProcessOutcome, ProcessRunner, ProcessSpec};

use crate::{BuildError, RebuildDriver, WorkerArtifact};
use artifact::ArtifactStore;

/// Exactly one executable target; glob selection and library targets refuse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoTarget {
    /// A package binary.
    Bin(String),
    /// An executable package example.
    Example(String),
}

impl CargoTarget {
    fn selection(&self) -> (&'static str, &str) {
        match self {
            Self::Bin(name) => ("--bin", name),
            Self::Example(name) => ("--example", name),
        }
    }
}

/// Explicit compiler, project, output and process authority for one session.
#[derive(Clone, Debug)]
pub struct CargoBuildConfig {
    /// Absolute nightly Cargo executable (prefer `rustup which cargo`).
    /// Nightly -C selects the project's configuration without changing the
    /// supervisor's process cwd or weakening exact-image process authority.
    pub cargo: PathBuf,
    /// Absolute Cargo.toml. Cargo.lock must already be current (`--locked`).
    pub manifest: PathBuf,
    /// One package name, not an arbitrary Cargo package-selector expression.
    pub package: String,
    /// One native binary/example target.
    pub target: CargoTarget,
    /// Explicit host triple. Prevents ambient build.target changing output paths.
    pub target_triple: String,
    /// Absolute incremental Cargo cache/output root, passed via --target-dir.
    pub target_dir: PathBuf,
    /// Absolute parent of a private per-driver immutable artifact directory.
    pub artifact_dir: PathBuf,
    /// Use Cargo's release profile instead of dev.
    pub release: bool,
    /// Refuse network access through Cargo's --offline (default true).
    pub offline: bool,
    /// Explicit selected features (empty means the package's normal defaults).
    pub features: Vec<String>,
    /// Complete compiler environment. Inheritance is disabled by the runner.
    /// The host must supply PATH/toolchain/home/temp variables it intends to use.
    pub build_env: Vec<(String, String)>,
    /// Exact worker-mode arguments, not compiler flags.
    pub worker_argv: Vec<String>,
    /// Separate worker environment; compiler credentials are not copied here.
    pub worker_env: Vec<(String, String)>,
    /// Optional absolute working directory for the worker, not the compiler.
    pub worker_cwd: Option<PathBuf>,
    /// Compiler process-tree deadline, including waiting on Cargo's cache lock.
    pub timeout: Duration,
    /// Per-stream compiler stdout/stderr byte ceiling.
    pub max_output_bytes: u64,
    /// Maximum bytes in one executable copy.
    pub max_artifact_bytes: u64,
    /// Maximum distinct retained images. Identical rebuilds reuse their image.
    pub max_artifacts: usize,
}

impl CargoBuildConfig {
    /// Configure one native target. Paths and environment are checked before
    /// the compiler runs; no ambient tool discovery or arbitrary extra argv.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cargo: PathBuf,
        manifest: PathBuf,
        package: String,
        target: CargoTarget,
        target_triple: String,
        target_dir: PathBuf,
        artifact_dir: PathBuf,
    ) -> Self {
        Self {
            cargo,
            manifest,
            package,
            target,
            target_triple,
            target_dir,
            artifact_dir,
            release: false,
            offline: true,
            features: Vec::new(),
            build_env: Vec::new(),
            worker_argv: Vec::new(),
            worker_env: Vec::new(),
            worker_cwd: None,
            timeout: Duration::from_secs(300),
            max_output_bytes: 8 * 1024 * 1024,
            max_artifact_bytes: 512 * 1024 * 1024,
            max_artifacts: 32,
        }
    }

    fn validate(&self) -> Result<(), BuildError> {
        for path in [
            &self.cargo,
            &self.manifest,
            &self.target_dir,
            &self.artifact_dir,
        ] {
            absolute(path)?;
        }
        if let Some(path) = &self.worker_cwd {
            absolute(path)?;
        }
        name(&self.package)?;
        name(self.target.selection().1)?;
        name(&self.target_triple)?;
        if self.target_triple.split('-').count() < 3 {
            return Err(fail("an explicit host target triple is required"));
        }
        for feature in &self.features {
            if feature.len() > 128
                || feature.is_empty()
                || !feature
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_/".contains(&c))
            {
                return Err(fail("invalid explicit Cargo feature"));
            }
        }
        environment(&self.build_env)?;
        environment(&self.worker_env)?;
        if self.worker_argv.len() > 128
            || self
                .worker_argv
                .iter()
                .any(|arg| arg.len() > 8192 || arg.contains('\0'))
        {
            return Err(fail(
                "worker argv exceeds its count/byte limit or contains NUL",
            ));
        }
        if self.timeout.is_zero()
            || self.max_output_bytes == 0
            || self.max_output_bytes > 16 * 1024 * 1024
            || self.max_artifact_bytes == 0
            || self.max_artifact_bytes > 2 * 1024 * 1024 * 1024
            || self.max_artifacts == 0
            || self.max_artifacts > 128
            || self.features.len() > 128
        {
            return Err(fail("invalid native build deadline or resource bounds"));
        }
        Ok(())
    }

    fn process(&self) -> Result<ProcessSpec, BuildError> {
        let (flag, target) = self.target.selection();
        let project = self
            .manifest
            .parent()
            .ok_or_else(|| fail("manifest has no project directory"))?;
        let mut argv = vec![
            "-Z".into(),
            "unstable-options".into(),
            "-C".into(),
            text(project)?.into(),
            "build".into(),
            "--locked".into(),
            "--color".into(),
            "never".into(),
            "--message-format".into(),
            "short".into(),
            "--manifest-path".into(),
            text(&self.manifest)?.into(),
            "--package".into(),
            self.package.clone(),
            flag.into(),
            target.into(),
            "--target".into(),
            self.target_triple.clone(),
            "--target-dir".into(),
            text(&self.target_dir)?.into(),
        ];
        if self.release {
            argv.push("--release".into());
        }
        if self.offline {
            argv.push("--offline".into());
        }
        if !self.features.is_empty() {
            argv.push("--features".into());
            argv.push(self.features.join(","));
        }
        Ok(ProcessSpec {
            program: self.cargo.clone(),
            argv,
            env: self.build_env.clone(),
            cwd: None,
            stdin: None,
            timeout: self.timeout,
            max_output_bytes: self.max_output_bytes,
        })
    }

    fn output(&self) -> PathBuf {
        let mut path = self
            .target_dir
            .join(&self.target_triple)
            .join(if self.release { "release" } else { "debug" });
        if matches!(&self.target, CargoTarget::Example(_)) {
            path.push("examples");
        }
        path.push(format!(
            "{}{}",
            self.target.selection().1,
            std::env::consts::EXE_SUFFIX
        ));
        path
    }
}

/// A real Cargo implementation of the existing supervisor build interface.
///
/// The bounded process runner is injected for host capability selection/tests.
/// `native-build` enables fmn-platform's audited host exact-process runner.
/// A failed compile, timeout, missing image or publication failure returns no
/// artifact, allowing Supervisor to keep the previous worker. Each successful
/// distinct image is privately copied before returning, so a later Cargo link
/// cannot change the executable used for crash recovery.
///
/// Published paths can outlive this driver through WorkerArtifact clones.
/// Consequently Drop does NOT delete them. Call `cleanup` only after all worker
/// generations and their supervisors are stopped. The incremental target
/// directory is caller-managed and never removed by this type.
pub struct CargoRebuildDriver {
    config: CargoBuildConfig,
    runner: Arc<dyn ProcessRunner>,
    artifacts: ArtifactStore,
    last_outcome: Option<ProcessOutcome>,
}

impl CargoRebuildDriver {
    pub fn new(
        config: CargoBuildConfig,
        runner: Arc<dyn ProcessRunner>,
    ) -> Result<Self, BuildError> {
        config.validate()?;
        Ok(Self {
            artifacts: ArtifactStore::new(config.artifact_dir.clone()),
            config,
            runner,
            last_outcome: None,
        })
    }

    /// Last bounded compiler output, including unsuccessful exits/timeouts.
    #[must_use]
    pub fn last_outcome(&self) -> Option<&ProcessOutcome> {
        self.last_outcome.as_ref()
    }

    /// Private published-image directory, once the first image is admitted.
    #[must_use]
    pub fn artifact_directory(&self) -> Option<&Path> {
        self.artifacts.directory()
    }

    /// Remove only this driver's privately created images. Stop all consumers
    /// first; never call while a supervisor might restart an old generation.
    pub fn cleanup(self) -> Result<(), BuildError> {
        self.artifacts.cleanup()
    }

    /// Query an explicitly selected Cargo executable's host triple. This is a
    /// bounded host operation, not ambient tool discovery. Useful for composing
    /// the explicit --target setting from the same compiler used for builds.
    pub fn host_triple(
        runner: &dyn ProcessRunner,
        cargo: &Path,
        env: &[(String, String)],
    ) -> Result<String, BuildError> {
        absolute(cargo)?;
        environment(env)?;
        let outcome = runner
            .run(&ProcessSpec {
                program: cargo.to_path_buf(),
                argv: vec!["--version".into(), "--verbose".into()],
                env: env.to_vec(),
                cwd: None,
                stdin: None,
                timeout: Duration::from_secs(15),
                max_output_bytes: 64 * 1024,
            })
            .map_err(fail)?;
        if !outcome.success()
            || outcome.stdout.len() > 64 * 1024
            || outcome.stderr.len() > 64 * 1024
        {
            return Err(fail(
                "selected Cargo could not report a bounded host triple",
            ));
        }
        let output = std::str::from_utf8(&outcome.stdout).map_err(fail)?;
        let mut hosts = output
            .lines()
            .filter_map(|line| line.strip_prefix("host: "));
        let host = hosts
            .next()
            .ok_or_else(|| fail("Cargo omitted its host triple"))?;
        name(host)?;
        if hosts.next().is_some() || host.split('-').count() < 3 {
            return Err(fail("Cargo returned an ambiguous host triple"));
        }
        Ok(host.into())
    }
}

impl RebuildDriver for CargoRebuildDriver {
    fn rebuild(&mut self) -> Result<WorkerArtifact, BuildError> {
        self.last_outcome = None;
        self.config.validate()?;
        if !self.config.manifest.is_file() {
            return Err(fail("Cargo manifest is not a regular file"));
        }
        let outcome = self.runner.run(&self.config.process()?).map_err(fail)?;
        // Revalidate injected runners before storing output or publishing images.
        if outcome.stdout.len() as u64 > self.config.max_output_bytes
            || outcome.stderr.len() as u64 > self.config.max_output_bytes
        {
            return Err(fail("compiler runner exceeded its declared output bound"));
        }
        let success = outcome.success();
        self.last_outcome = Some(outcome);
        if !success {
            let outcome = self
                .last_outcome
                .as_ref()
                .ok_or_else(|| fail("missing build result"))?;
            let tail = &outcome.stderr[outcome.stderr.len().saturating_sub(16 * 1024)..];
            let mut diagnostic = String::new();
            for c in String::from_utf8_lossy(tail).chars() {
                if c == '\n' || c == '\t' || !c.is_control() {
                    diagnostic.push(c);
                } else {
                    diagnostic.extend(c.escape_default());
                }
            }
            return Err(fail(format!(
                "Cargo {:?}\n{diagnostic}",
                outcome.termination
            )));
        }
        let (executable, build_id) = self.artifacts.publish(
            &self.config.output(),
            self.config.max_artifact_bytes,
            self.config.max_artifacts,
        )?;
        Ok(WorkerArtifact {
            executable,
            build_id,
            argv: self.config.worker_argv.clone(),
            env: self.config.worker_env.clone(),
            cwd: self.config.worker_cwd.clone(),
        })
    }
}

fn name(value: &str) -> Result<(), BuildError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    {
        return Err(fail("invalid package, executable target or host triple"));
    }
    Ok(())
}

fn absolute(path: &Path) -> Result<(), BuildError> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(fail(
            "native build paths must be absolute without parent traversal",
        ));
    }
    let _ = text(path)?;
    Ok(())
}

fn text(path: &Path) -> Result<&str, BuildError> {
    path.to_str()
        .filter(|s| !s.contains('\0'))
        .ok_or_else(|| fail("native build paths must be losslessly representable UTF-8"))
}

fn environment(env: &[(String, String)]) -> Result<(), BuildError> {
    if env.len() > 128 {
        return Err(fail("too many explicit environment variables"));
    }
    let mut names = BTreeSet::new();
    let mut size = 0_usize;
    for (key, value) in env {
        size = size.saturating_add(key.len()).saturating_add(value.len());
        if key.is_empty()
            || key.contains(['=', '\0'])
            || value.contains('\0')
            || !names.insert(key.to_ascii_uppercase())
            || size > 256 * 1024
        {
            return Err(fail("invalid, duplicate or oversized explicit environment"));
        }
    }
    Ok(())
}

pub(super) fn fail(error: impl std::fmt::Display) -> BuildError {
    BuildError::new(error.to_string())
}
