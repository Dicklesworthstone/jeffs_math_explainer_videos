#![cfg(not(target_arch = "wasm32"))]
//! Scripted compiler outcomes plus real immutable artifact publication.
//! Actual compiler/process acceptance is in the native-build feature suite.

use fmn_platform::process::{ProcessOutcome, ProcessTermination, ScriptedRunner};
use fmn_studio::project::{CargoBuildConfig, CargoRebuildDriver, CargoTarget};
use fmn_studio::{RebuildDriver, protocol_digest};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        for _ in 0..128 {
            let root = std::env::temp_dir().join(format!(
                "fmn-build-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            if fs::create_dir(&root).is_ok() {
                return Self(root);
            }
        }
        panic!("cannot create private test directory");
    }
    fn config(&self) -> CargoBuildConfig {
        fs::write(
            self.0.join("Cargo.toml"),
            "[package]\nname='worker'\nversion='0.1.0'\n",
        )
        .unwrap();
        let mut config = CargoBuildConfig::new(
            std::env::current_exe().unwrap(),
            self.0.join("Cargo.toml"),
            "worker".into(),
            CargoTarget::Bin("worker".into()),
            "x86_64-unknown-linux-gnu".into(),
            self.0.join("build output"),
            self.0.join("published images"),
        );
        config.build_env = vec![("PATH".into(), "/explicit/toolchain".into())];
        config.worker_env = vec![("SCENE_MODE".into(), "worker".into())];
        config.worker_argv = vec!["--worker".into(), "argument with spaces".into()];
        config
    }
    fn image(&self, bytes: &[u8]) -> PathBuf {
        let path = self
            .0
            .join("build output/x86_64-unknown-linux-gnu/debug")
            .join(format!("worker{}", std::env::consts::EXE_SUFFIX));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn runner(config: &CargoBuildConfig, termination: ProcessTermination) -> Arc<ScriptedRunner> {
    let mut runner = ScriptedRunner::new();
    runner.script(
        &config.cargo,
        ProcessOutcome {
            termination,
            stdout: Vec::new(),
            stderr: b"compiler diagnostic".to_vec(),
        },
    );
    Arc::new(runner)
}

#[test]
fn exact_argv_explicit_environment_and_immutable_generations() {
    let directory = Directory::new();
    let config = directory.config();
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    directory.image(b"first image");
    let mut driver = CargoRebuildDriver::new(config.clone(), runner.clone()).unwrap();
    let first = driver.rebuild().unwrap();
    directory.image(b"second image");
    let second = driver.rebuild().unwrap();
    assert_ne!(first.executable, second.executable);
    assert_eq!(fs::read(&first.executable).unwrap(), b"first image");
    assert_eq!(first.build_id, protocol_digest(b"first image"));
    assert_eq!(second.build_id, protocol_digest(b"second image"));
    assert_eq!(first.argv, config.worker_argv);
    assert_eq!(first.env, config.worker_env);
    assert_eq!(
        driver.rebuild().unwrap(),
        second,
        "unchanged rebuild reuses its private image"
    );
    let specs = runner.runs();
    let spec = &specs[0];
    assert_eq!(spec.program, config.cargo);
    assert_eq!(spec.env, config.build_env);
    assert_eq!(spec.cwd, None);
    assert_eq!(spec.stdin, None);
    assert!(
        spec.argv
            .windows(2)
            .any(|pair| pair == ["--target", "x86_64-unknown-linux-gnu"])
    );
    assert!(spec.argv.windows(2).any(|pair| pair == ["--bin", "worker"]));
    assert!(spec.argv.contains(&"--locked".into()));
    assert!(spec.argv.contains(&"--offline".into()));
    let published = driver.artifact_directory().unwrap().to_owned();
    driver.cleanup().unwrap();
    assert!(!published.exists());
    assert!(
        directory.0.join("build output").exists(),
        "cleanup must not remove Cargo's cache"
    );
}

#[test]
fn compiler_failure_timeout_and_output_overflow_never_publish_an_old_output() {
    for termination in [
        ProcessTermination::Exited(Some(101)),
        ProcessTermination::TimedOut,
        ProcessTermination::OutputLimitExceeded,
    ] {
        let directory = Directory::new();
        let config = directory.config();
        directory.image(b"stale successful output");
        let runner = runner(&config, termination);
        let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
        assert!(
            driver
                .rebuild()
                .unwrap_err()
                .message
                .contains("compiler diagnostic")
        );
        assert_eq!(driver.last_outcome().unwrap().termination, termination);
        assert!(driver.artifact_directory().is_none());
    }
}

#[test]
fn missing_or_oversized_outputs_are_not_successful_artifacts() {
    let directory = Directory::new();
    let mut config = directory.config();
    config.max_artifact_bytes = 4;
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
    assert!(driver.rebuild().is_err());
    directory.image(b"too large");
    assert!(driver.rebuild().is_err());
    assert!(driver.artifact_directory().is_none());
}

#[test]
fn generation_bound_preserves_previous_images_and_unchanged_builds() {
    let directory = Directory::new();
    let mut config = directory.config();
    config.max_artifacts = 1;
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    directory.image(b"one");
    let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
    let first = driver.rebuild().unwrap();
    assert_eq!(driver.rebuild().unwrap(), first);
    directory.image(b"two");
    assert!(driver.rebuild().is_err());
    assert_eq!(fs::read(first.executable).unwrap(), b"one");
}

#[test]
fn hostile_selection_and_environment_refuse_before_spawning() {
    for variant in 0..7 {
        let directory = Directory::new();
        let mut config = directory.config();
        match variant {
            0 => config.cargo = "cargo".into(),
            1 => config.package = "--all".into(),
            2 => config.target = CargoTarget::Bin("*".into()),
            3 => config.target = CargoTarget::Example("../escape".into()),
            4 => config.target_triple = "../../elsewhere".into(),
            5 => config.build_env.push(("path".into(), "/other".into())),
            _ => config.worker_argv.push("bad\0arg".into()),
        }
        let runner = runner(&config, ProcessTermination::Exited(Some(0)));
        assert!(CargoRebuildDriver::new(config, runner.clone()).is_err());
        assert!(runner.runs().is_empty());
    }
}

#[test]
fn dropping_driver_does_not_delete_an_artifact_still_owned_by_a_supervisor() {
    let directory = Directory::new();
    let config = directory.config();
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    directory.image(b"retained worker");
    let artifact = {
        let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
        driver.rebuild().unwrap()
    };
    assert_eq!(fs::read(artifact.executable).unwrap(), b"retained worker");
}

#[test]
fn modified_published_artifact_is_not_silently_reused() {
    let directory = Directory::new();
    let config = directory.config();
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    directory.image(b"original");
    let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
    let first = driver.rebuild().unwrap();
    fs::write(first.executable, b"tampered").unwrap();
    assert!(driver.rebuild().is_err());
}

#[test]
fn examples_release_and_feature_selection_resolve_the_declared_output_only() {
    let directory = Directory::new();
    let mut config = directory.config();
    config.target = CargoTarget::Example("demo".into());
    config.release = true;
    config.features = vec!["native-build".into()];
    let output = config
        .target_dir
        .join(&config.target_triple)
        .join("release/examples")
        .join(format!("demo{}", std::env::consts::EXE_SUFFIX));
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    fs::write(output, b"example").unwrap();
    let runner = runner(&config, ProcessTermination::Exited(Some(0)));
    let mut driver = CargoRebuildDriver::new(config, runner.clone()).unwrap();
    driver.rebuild().unwrap();
    let specs = runner.runs();
    assert!(
        specs[0]
            .argv
            .windows(2)
            .any(|pair| pair == ["--example", "demo"])
    );
    assert!(specs[0].argv.contains(&"--release".into()));
    assert!(
        specs[0]
            .argv
            .windows(2)
            .any(|pair| pair == ["--features", "native-build"])
    );
}

#[test]
fn host_probe_is_bounded_and_requires_one_unambiguous_host() {
    let directory = Directory::new();
    let config = directory.config();
    for (output, expected) in [
        ("cargo 1.0\nhost: x86_64-unknown-linux-gnu\n", true),
        ("cargo\n", false),
        (
            "host: x86_64-unknown-linux-gnu\nhost: aarch64-apple-darwin\n",
            false,
        ),
    ] {
        let mut runner = ScriptedRunner::new();
        runner.script(
            &config.cargo,
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: output.as_bytes().to_vec(),
                stderr: Vec::new(),
            },
        );
        assert_eq!(
            CargoRebuildDriver::host_triple(&runner, &config.cargo, &[]).is_ok(),
            expected
        );
        assert_eq!(runner.runs()[0].argv, ["--version", "--verbose"]);
    }
}
