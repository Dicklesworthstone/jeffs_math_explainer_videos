#![cfg(all(feature = "native-build", unix))]
//! Real Cargo/rustc, immutable native images, supervisor and authenticated HTTP.
//! This suite creates only private temporary projects, never edits this checkout.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_platform::process::{ProcessRunner, ProcessSpec, StdProcessRunner};
use fmn_scene::Journal;
use fmn_studio::project::{CargoBuildConfig, CargoRebuildDriver, CargoTarget};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CapabilityToken, FrameHub, FramePayload, FrameStream, ProtocolLimits, RebuildDriver,
    StdWorkerLauncher, StudioHost, StudioHostConfig, StudioWorkerSession, Supervisor,
    SupervisorConfig, SupervisorReply, SupervisorRequest, WorkerResponse,
};

const NAME: &str = "ProjectScene";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        for _ in 0..128 {
            let path = std::env::temp_dir().join(format!(
                "fmn-real-cargo-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            if fs::create_dir(&path).is_ok() {
                return Self(path);
            }
        }
        panic!("private test directory unavailable");
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn toolchain() -> (PathBuf, Vec<(String, String)>) {
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
    let rustc = cargo.parent().unwrap().join("rustc");
    assert!(
        rustc.is_file(),
        "tests require the pinned toolchain's Cargo beside rustc"
    );
    env.push(("RUSTC".into(), rustc.to_str().unwrap().into()));
    (cargo, env)
}

fn configure(
    root: &Path,
    cargo: PathBuf,
    env: Vec<(String, String)>,
    runner: &dyn ProcessRunner,
) -> CargoBuildConfig {
    let triple = CargoRebuildDriver::host_triple(runner, &cargo, &env).unwrap();
    let mut config = CargoBuildConfig::new(
        cargo,
        root.join("Cargo.toml"),
        "project-worker".into(),
        CargoTarget::Bin("project-worker".into()),
        triple,
        root.join("target"),
        root.join("images"),
    );
    config.build_env = env;
    config.worker_argv = vec!["--worker".into()];
    config
}

fn generate_lock(config: &CargoBuildConfig, runner: &dyn ProcessRunner) {
    let result = runner
        .run(&ProcessSpec {
            program: config.cargo.clone(),
            argv: vec![
                "generate-lockfile".into(),
                "--offline".into(),
                "--manifest-path".into(),
                config.manifest.to_str().unwrap().into(),
            ],
            env: config.build_env.clone(),
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(60),
            max_output_bytes: 2 * 1024 * 1024,
        })
        .unwrap();
    assert!(
        result.success(),
        "lockfile bootstrap: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn actual_compiler_failure_preserves_executable_and_fixed_source_builds_new_generation() {
    let root = Temp::new();
    fs::create_dir(root.0.join("src")).unwrap();
    fs::write(
        root.0.join("Cargo.toml"),
        "[package]\nname='project-worker'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    let source = root.0.join("src/main.rs");
    fs::write(&source, "fn main(){println!(\"first\");}\n").unwrap();
    let runner = Arc::new(StdProcessRunner::default());
    let (cargo, env) = toolchain();
    let config = configure(&root.0, cargo, env, &*runner);
    generate_lock(&config, &*runner);
    let mut driver = CargoRebuildDriver::new(config, runner.clone()).unwrap();
    let first = driver.rebuild().unwrap();
    let execute = |path: PathBuf| {
        let result = runner
            .run(&ProcessSpec {
                program: path,
                argv: Vec::new(),
                env: Vec::new(),
                cwd: None,
                stdin: None,
                timeout: Duration::from_secs(10),
                max_output_bytes: 1024,
            })
            .unwrap();
        assert!(result.success());
        result.stdout
    };
    assert_eq!(execute(first.executable.clone()), b"first\n");
    fs::write(&source, "this is deliberately invalid Rust\n").unwrap();
    assert!(driver.rebuild().is_err());
    assert!(!driver.last_outcome().unwrap().success());
    assert_eq!(execute(first.executable.clone()), b"first\n");
    fs::write(&source, "fn main(){println!(\"second\");}\n").unwrap();
    let second = driver.rebuild().unwrap();
    assert_ne!(first.build_id, second.build_id);
    assert_eq!(execute(second.executable), b"second\n");
    assert_eq!(execute(first.executable), b"first\n");
    driver.cleanup().unwrap();
}

fn worker_source(offset: f64) -> String {
    include_str!("fixtures/project_worker.rs").replace(
        "const OFFSET: f64 = 0.0;",
        &format!("const OFFSET: f64 = {offset:?};"),
    )
}

fn response(reply: SupervisorReply) -> WorkerResponse {
    match reply {
        SupervisorReply::Worker(WorkerResponse::Error { message, .. }) => {
            panic!("worker refused: {message}")
        }
        SupervisorReply::Worker(response) => response,
        SupervisorReply::Recovered { .. } => panic!("unexpected worker recovery"),
    }
}

fn commit(supervisor: &mut Supervisor, frame: i64) {
    assert!(matches!(
        response(
            supervisor
                .request(
                    SupervisorRequest::Play {
                        scene: NAME.into(),
                        command: studio_seek_command(NAME, frame).unwrap(),
                    },
                    &|_| false
                )
                .unwrap()
        ),
        WorkerResponse::JournalSegment { .. }
    ));
}

fn frame(supervisor: &mut Supervisor, index: i64) -> FrameStream {
    let WorkerResponse::Frame(frame) = response(
        supervisor
            .request(
                SupervisorRequest::Scrub {
                    scene: NAME.into(),
                    frame: index,
                },
                &|_| false,
            )
            .unwrap(),
    ) else {
        panic!("expected actual rendered frame")
    };
    frame
}
fn png(frame: &FrameStream) -> &[u8] {
    let FramePayload::Pipe { bytes, .. } = &frame.payload else {
        panic!("inline PNG")
    };
    bytes
}

fn http(host: &StudioHost, cap: &str, route: &str, body: &str) -> String {
    let addr = host.local_addr().unwrap();
    let bytes = std::thread::scope(|scope| {
        let mut socket = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(330)))
            .unwrap();
        let server = scope.spawn(|| host.serve_once().unwrap());
        write!(socket, "POST {route}?cap={cap} HTTP/1.1\r\nHost: {addr}\r\nOrigin: http://{addr}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut bytes = Vec::new();
        socket.take(256 * 1024).read_to_end(&mut bytes).unwrap();
        server.join().unwrap();
        bytes
    });
    String::from_utf8(bytes).unwrap()
}

#[test]
fn real_project_rebuilds_replace_native_pixels_through_supervisor_and_http() {
    let root = Temp::new();
    fs::create_dir(root.0.join("src")).unwrap();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut manifest = String::from(
        "[package]\nname='project-worker'\nversion='0.1.0'\nedition='2024'\n[workspace]\n[dependencies]\n",
    );
    for name in [
        "fmn-core",
        "fmn-mobject",
        "fmn-render",
        "fmn-scene",
        "fmn-studio",
    ] {
        let path = crates
            .join(name)
            .to_str()
            .unwrap()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        manifest.push_str(&format!("{name}={{path=\"{path}\"}}\n"));
    }
    fs::write(root.0.join("Cargo.toml"), manifest).unwrap();
    let source = root.0.join("src/main.rs");
    fs::write(&source, worker_source(0.0)).unwrap();
    let runner = Arc::new(StdProcessRunner::default());
    let (cargo, env) = toolchain();
    let config = configure(&root.0, cargo, env, &*runner);
    generate_lock(&config, &*runner);
    let mut driver = CargoRebuildDriver::new(config, runner).unwrap();
    let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let cache = Store::open(
        fs,
        Arc::clone(&clock),
        root.0.join("cache"),
        StoreConfig::default(),
    )
    .unwrap()
    .namespace(
        "studio-replay",
        1,
        NamespacePolicy {
            ceiling_bytes: Some(64 * 1024 * 1024),
        },
    )
    .unwrap();
    let limits = ProtocolLimits::default();
    let mut supervisor = Supervisor::new(
        Box::new(StdWorkerLauncher::default()),
        Arc::clone(&clock),
        cache,
        SupervisorConfig {
            protocol_limits: limits,
            ..SupervisorConfig::default()
        },
    );
    supervisor.install_session(NAME, Journal::new()).unwrap();
    supervisor.build_and_start(&mut driver).unwrap();
    commit(&mut supervisor, 4);
    let first = frame(&mut supervisor, 4);
    let generation = supervisor.generation();
    let commands = supervisor.current_commands().unwrap();
    fs::write(&source, "invalid Rust\n").unwrap();
    assert!(
        supervisor
            .rebuild_and_restart(&mut driver, &commands, &|_| false)
            .is_err()
    );
    assert_eq!(supervisor.generation(), generation);
    assert_eq!(
        frame(&mut supervisor, 4),
        first,
        "failed compile must leave the old native worker usable"
    );
    fs::write(&source, worker_source(0.8)).unwrap();
    let recovery = supervisor
        .rebuild_and_restart(&mut driver, &commands, &|_| false)
        .unwrap();
    assert_eq!(supervisor.generation(), generation + 1);
    assert_eq!(
        recovery.plan.reuse, 0,
        "changed source is cold-reexecuted, not copied from stale checkpoints"
    );
    for command in commands.into_iter().skip(recovery.plan.reuse) {
        response(
            supervisor
                .request(
                    SupervisorRequest::Play {
                        scene: NAME.into(),
                        command,
                    },
                    &|_| false,
                )
                .unwrap(),
        );
    }
    let second = frame(&mut supervisor, 4);
    assert_ne!(png(&first), png(&second));

    // Use the real browser host's existing Restart action with the real compiler.
    let session = Arc::new(
        StudioWorkerSession::new(NAME, supervisor, Box::new(driver), Arc::new(|_| false)).unwrap(),
    );
    let host_config = StudioHostConfig::default();
    let frames = FrameHub::new(host_config.max_frame_history, host_config.max_png_bytes).unwrap();
    frames.publish(&second, limits).unwrap();
    let token = CapabilityToken::new([7; 32]).unwrap();
    let cap = token.try_expose_hex().unwrap();
    let host = StudioHost::bind(
        Arc::clone(&session),
        frames.clone(),
        token,
        clock,
        host_config,
    )
    .unwrap();
    assert!(http(&host, &cap, "/api/scrub", "frame=4&commit=true").starts_with("HTTP/1.1 200 "));
    let before = frames.latest().unwrap();
    fs::write(&source, "a broken edit\n").unwrap();
    assert!(!http(&host, &cap, "/api/restart", "").starts_with("HTTP/1.1 200 "));
    assert_eq!(frames.latest().unwrap().png, before.png);
    assert!(http(&host, &cap, "/api/scrub", "frame=4").starts_with("HTTP/1.1 200 "));
    assert_eq!(frames.latest().unwrap().png, before.png);
    fs::write(&source, worker_source(-0.8)).unwrap();
    let result = http(&host, &cap, "/api/restart", "");
    assert!(result.starts_with("HTTP/1.1 200 "), "{result}");
    let rebuilt = frames.latest().unwrap();
    assert_eq!(rebuilt.frame_index, 4);
    assert_ne!(rebuilt.png, before.png);
    frames.close();
    session.shutdown_worker();
    println!(
        "OK: real Cargo builds, failed-edit worker preservation, source replacement, HTTP replay and changed native PNGs"
    );
}
