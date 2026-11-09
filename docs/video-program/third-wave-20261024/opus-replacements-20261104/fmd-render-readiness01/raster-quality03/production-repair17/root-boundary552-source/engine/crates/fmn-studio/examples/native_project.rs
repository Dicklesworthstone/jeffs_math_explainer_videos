//! A complete source-project Studio host. Scene code runs in a compiled child.
//! Usage: native_project CARGO MANIFEST PACKAGE bin:TARGET SCENE WORKER_ARG [WATCH_PATH ...]
//! Use example:TARGET for a Cargo example instead of a binary.

use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::entropy::{HostEntropy, StdHostEntropy};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_platform::process::StdProcessRunner;
use fmn_scene::Journal;
use fmn_studio::project::{CargoBuildConfig, CargoRebuildDriver, CargoTarget};
use fmn_studio::project_watch::{SourceWatch, WatchLimits};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    CapabilityToken, FrameHub, ProtocolLimits, StdWorkerLauncher, StudioHost, StudioHostConfig,
    StudioWorkerSession, Supervisor, SupervisorConfig, SupervisorReply, SupervisorRequest,
    WorkerResponse,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn response(reply: SupervisorReply) -> Result<WorkerResponse> {
    match reply {
        SupervisorReply::Worker(WorkerResponse::Error { message, .. }) => Err(message.into()),
        SupervisorReply::Worker(response) => Ok(response),
        SupervisorReply::Recovered { .. } => Err("worker recovered; retry the operation".into()),
    }
}

fn compiler_environment(cargo: &std::path::Path) -> Result<Vec<(String, String)>> {
    let mut env = Vec::new();
    // Explicit host snapshot, not env::vars(). The worker receives none of it.
    for key in [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SYSTEMROOT",
        "WINDIR",
        "USERPROFILE",
        "APPDATA",
        "LOCALAPPDATA",
    ] {
        if let Some(value) = std::env::var_os(key) {
            env.push((
                key.into(),
                value
                    .into_string()
                    .map_err(|_| format!("{key} is not UTF-8"))?,
            ));
        }
    }
    let rustc = cargo
        .parent()
        .ok_or("Cargo has no directory")?
        .join(format!("rustc{}", std::env::consts::EXE_SUFFIX));
    if !rustc.is_file() {
        return Err("select Cargo beside its rustc (for example: rustup which cargo)".into());
    }
    env.push((
        "RUSTC".into(),
        rustc.to_str().ok_or("rustc path is not UTF-8")?.into(),
    ));
    Ok(env)
}

fn rebuild_http(addr: SocketAddr, cap: &str) -> Result<()> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(330)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    write!(
        stream,
        "POST /api/restart?cap={cap} HTTP/1.1\r\nHost: {addr}\r\nOrigin: http://{addr}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    )?;
    let mut bytes = Vec::new();
    stream.take(128 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 128 * 1024 {
        return Err("rebuild response exceeded its byte limit".into());
    }
    let text = String::from_utf8_lossy(&bytes);
    if !text.starts_with("HTTP/1.1 200 ") {
        return Err(
            format!("source rebuild refused; last published frame retained:\n{text}").into(),
        );
    }
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 6 || args.len() > 128 {
        return Err("usage: native_project CARGO MANIFEST PACKAGE bin:TARGET|example:TARGET SCENE WORKER_ARG [WATCH_PATH ...]".into());
    }
    let cargo = PathBuf::from(&args[0]);
    if !cargo.is_absolute() {
        return Err("Cargo must be an explicit absolute executable path".into());
    }
    let manifest = PathBuf::from(&args[1]).canonicalize()?;
    let project = manifest
        .parent()
        .ok_or("manifest has no parent")?
        .to_path_buf();
    let target = match args[3].split_once(':') {
        Some(("bin", name)) => CargoTarget::Bin(name.into()),
        Some(("example", name)) => CargoTarget::Example(name.into()),
        _ => return Err("target must be bin:NAME or example:NAME".into()),
    };
    let scene = args[4].clone();
    let runner = Arc::new(StdProcessRunner::default());
    let environment = compiler_environment(&cargo)?;
    let triple = CargoRebuildDriver::host_triple(&*runner, &cargo, &environment)?;
    let target_dir = project.join("target/fmn-studio-build");
    let artifact_dir = project.join("target/fmn-studio-workers");
    let mut config = CargoBuildConfig::new(
        cargo,
        manifest.clone(),
        args[2].clone(),
        target,
        triple,
        target_dir.clone(),
        artifact_dir.clone(),
    );
    config.build_env = environment;
    config.worker_argv = vec![args[5].clone()];
    let mut builder = CargoRebuildDriver::new(config, runner)?;
    let mut inputs = vec![manifest, project.join("Cargo.lock")];
    if args.len() == 6 {
        inputs.push(project.join("src"));
    }
    for input in args.iter().skip(6) {
        inputs.push(PathBuf::from(input).canonicalize()?);
    }
    let mut watcher = SourceWatch::new(
        inputs,
        vec![target_dir, artifact_dir],
        Duration::from_millis(300),
        WatchLimits::default(),
    )?;

    let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let cache = Store::open(
        fs,
        Arc::clone(&clock),
        project.join("target/fmn-project-cache"),
        StoreConfig::default(),
    )?
    .namespace(
        "studio-replay",
        1,
        NamespacePolicy {
            ceiling_bytes: Some(64 * 1024 * 1024),
        },
    )?;
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
    supervisor.install_session(&scene, Journal::new())?;
    eprintln!("Building the selected native Studio worker...");
    supervisor.build_and_start(&mut builder)?;
    // A generic project host cannot attest arbitrary file/asset reads. Refuse
    // checkpoint reuse conservatively, then use the existing host's ordered
    // re-execution of invalidated commands after every successful rebuild.
    let initial = (|| -> Result<_> {
        response(supervisor.request(
            SupervisorRequest::Play {
                scene: scene.clone(),
                command: studio_seek_command(&scene, 0)?,
            },
            &|_| false,
        )?)?;
        let WorkerResponse::Frame(frame) = response(supervisor.request(
            SupervisorRequest::Scrub {
                scene: scene.clone(),
                frame: 0,
            },
            &|_| false,
        )?)?
        else {
            return Err("selected worker did not return its native initial frame".into());
        };
        Ok(frame)
    })();
    let initial = match initial {
        Ok(frame) => frame,
        Err(error) => {
            supervisor.shutdown_worker();
            return Err(error);
        }
    };
    let published_directory = builder
        .artifact_directory()
        .map(std::path::Path::to_path_buf);
    let session = Arc::new(StudioWorkerSession::new(
        &scene,
        supervisor,
        Box::new(builder),
        Arc::new(|_| false),
    )?);
    let host_config = StudioHostConfig::default();
    let frames = FrameHub::new(host_config.max_frame_history, host_config.max_png_bytes)?;
    frames.publish(&initial, limits)?;
    let mut secret = [0_u8; 32];
    StdHostEntropy.fill(&mut secret)?;
    let token = CapabilityToken::new(secret)?;
    let cap = token.try_expose_hex()?;
    let host = StudioHost::bind(
        Arc::clone(&session),
        frames.clone(),
        token,
        Arc::clone(&clock),
        host_config,
    )?;
    let addr = host.local_addr()?;
    println!("{}", host.launch_url()?);
    eprintln!(
        "Watching declared source paths. Browser Restart also rebuilds. Press Enter here to stop."
    );
    let stop = Arc::new(AtomicBool::new(false));
    let input_stop = Arc::clone(&stop);
    let input_frames = frames.clone();
    std::thread::Builder::new()
        .name("project-studio-stop".into())
        .spawn(move || {
            let _ = std::io::stdin().read_line(&mut String::new());
            // Wake idle multipart readers before serve_until joins their threads.
            // Closing only after serve_until returns would wait out every idle timeout.
            input_frames.close();
            input_stop.store(true, Ordering::Release);
        })?;
    let watch_stop = Arc::clone(&stop);
    let thread = std::thread::Builder::new()
        .name("project-studio-source-watch".into())
        .spawn(move || {
            let mut last_error = String::new();
            while !watch_stop.load(Ordering::Acquire) {
                match watcher.poll(clock.monotonic()) {
                    Ok(true) => {
                        eprintln!("Source changed; rebuilding the worker...");
                        match rebuild_http(addr, &cap) {
                            Ok(()) => eprintln!("Native worker rebuilt; committed frame restored."),
                            Err(error) => eprintln!("{error}"),
                        }
                        last_error.clear();
                    }
                    Ok(false) => {
                        last_error.clear();
                    }
                    Err(error) => {
                        let message = error.to_string();
                        if message != last_error {
                            eprintln!("Source watch: {message}");
                            last_error = message;
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        })?;
    let result = host.serve_until(&stop);
    stop.store(true, Ordering::Release);
    frames.close();
    let _ = thread.join();
    session.shutdown_worker();
    // No image is removed while a supervisor may still restart it.
    drop(host);
    drop(session);
    if let Some(directory) = published_directory {
        std::fs::remove_dir_all(directory)?;
    }
    result.map_err(Into::into)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("native project Studio: {error}");
        std::process::exit(1);
    }
}
