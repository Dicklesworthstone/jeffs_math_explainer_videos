#![cfg(all(feature = "native-build", unix))]
//! Project-local Cargo configuration must affect compilation, while explicit
//! target/output selection must override the project's default cross target.

use fmn_platform::process::{ProcessRunner, ProcessSpec, StdProcessRunner};
use fmn_studio::RebuildDriver;
use fmn_studio::project::{CargoBuildConfig, CargoRebuildDriver, CargoTarget};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn project_local_configuration_is_used_without_changing_the_parent_working_directory() {
    let path = std::env::temp_dir().join(format!("fmn-project-config-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let root = Temp(path);
    fs::create_dir(root.0.join("src")).unwrap();
    fs::create_dir(root.0.join(".cargo")).unwrap();
    fs::write(
        root.0.join("Cargo.toml"),
        "[package]\nname='config-worker'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    fs::write(
        root.0.join("Cargo.lock"),
        "version = 4\n[[package]]\nname = \"config-worker\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(
        root.0.join("src/main.rs"),
        "fn main(){println!(\"{}\", env!(\"FMN_TEST_PROJECT_CONFIGURATION\"));}\n",
    )
    .unwrap();
    fs::write(root.0.join(".cargo/config.toml"), "[build]\ntarget='wasm32-unknown-unknown'\ntarget-dir='redirected-output'\n[env]\nFMN_TEST_PROJECT_CONFIGURATION={value='project-local-config',force=true}\n").unwrap();
    let cargo = PathBuf::from(env!("CARGO"));
    let mut env = Vec::new();
    for key in [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "TMPDIR",
        "TMP",
        "TEMP",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.push((key.into(), value));
        }
    }
    env.push((
        "RUSTC".into(),
        cargo
            .parent()
            .unwrap()
            .join("rustc")
            .to_str()
            .unwrap()
            .into(),
    ));
    let runner = Arc::new(StdProcessRunner::default());
    let triple = CargoRebuildDriver::host_triple(&*runner, &cargo, &env).unwrap();
    let before = std::env::current_dir().unwrap();
    let manifest = root.0.join("Cargo.toml");
    let output = root.0.join("native-output");
    // Negative control: --manifest-path alone cannot see project-local config.
    let old = runner
        .run(&ProcessSpec {
            program: cargo.clone(),
            argv: vec![
                "build".into(),
                "--locked".into(),
                "--offline".into(),
                "--manifest-path".into(),
                manifest.to_str().unwrap().into(),
                "--target".into(),
                triple.clone(),
                "--target-dir".into(),
                output.to_str().unwrap().into(),
            ],
            env: env.clone(),
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(60),
            max_output_bytes: 1024 * 1024,
        })
        .unwrap();
    assert!(!old.success());
    assert!(String::from_utf8_lossy(&old.stderr).contains("FMN_TEST_PROJECT_CONFIGURATION"));
    let mut config = CargoBuildConfig::new(
        cargo,
        manifest,
        "config-worker".into(),
        CargoTarget::Bin("config-worker".into()),
        triple,
        output.clone(),
        root.0.join("images"),
    );
    config.build_env = env;
    let mut builder = CargoRebuildDriver::new(config, runner.clone()).unwrap();
    let worker = builder.rebuild().unwrap();
    let actual = runner
        .run(&ProcessSpec {
            program: worker.executable,
            argv: Vec::new(),
            env: Vec::new(),
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(10),
            max_output_bytes: 1024,
        })
        .unwrap();
    assert!(actual.success());
    assert_eq!(actual.stdout, b"project-local-config\n");
    assert_eq!(std::env::current_dir().unwrap(), before);
    assert!(output.exists());
    assert!(!root.0.join("redirected-output").exists());
    builder.cleanup().unwrap();
}
