//! Run: cargo run -p fmn-studio --example native_live
//! Headless real-process + HTTP acceptance: append `-- --self-test`.
//! The parent owns only host/supervisor state. The factory runs in the child.

use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use fmn_anim::rotate;
use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_core::{color::LinearRgba, constants::PI};
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::entropy::{HostEntropy, StdHostEntropy};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{
    AssetRead, EventListener, EventPayload, EventPropagation, EventTarget, EventType, Journal, Key,
    Modifiers, PlayOverrides, RuntimeConfig, Scene, SceneError,
};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    BuildError, CapabilityToken, FrameHub, FrameStream, ProtocolDigest, RebuildDriver,
    ServiceError, StdWorkerLauncher, StudioHost, StudioHostConfig, StudioWorkerSession, Supervisor,
    SupervisorConfig, SupervisorReply, SupervisorRequest, WorkerArtifact, WorkerErrorCode,
    WorkerResponse, protocol_digest, serve_worker,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const NAME: &str = "NativeLive";
const WORKER_ARG: &str = "--native-live-worker";
const CRASH_KEY: Key = Key::Other(0xffff_fffd);

fn execution_error(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}

fn program() -> std::result::Result<NativeSceneProgram, ServiceError> {
    let mut scene = Scene::new(RuntimeConfig::default(), 71).map_err(execution_error)?;
    let points = [
        [-1.0_f32, -0.5, 0.0],
        [0.0, -0.5, 0.0],
        [1.0, -0.5, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.5, 0.0],
        [0.0, 0.5, 0.0],
        [-1.0, 0.5, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, -0.5, 0.0],
    ];
    let mut buffer =
        RecordBuffer::new(RecordSchema::vmobject(), points.len()).map_err(execution_error)?;
    for (index, point) in points.iter().enumerate() {
        buffer.write(index, "point", point);
        buffer.write(index, "fill_rgba", &[0.2, 0.65, 1.0, 1.0]);
        buffer.write(index, "stroke_width", &[0.0]);
    }
    let root = scene.stage_mut().add(Mobject::from_buffer(buffer));
    scene
        .stage_mut()
        .add_to_scene(root)
        .map_err(execution_error)?;
    let mut tick = 0_u32;
    scene
        .stage_mut()
        .add_updater(
            root,
            move |stage, target| {
                tick += 1;
                let delta = if tick % 120 < 60 { 0.02 } else { -0.02 };
                stage.shift_many(&[target], [delta, 0.0, 0.0]);
            },
            false,
        )
        .map_err(execution_error)?;
    // A protocol-only test key proves panic isolation through the real child;
    // it has no ordinary browser keyboard spelling and is not an editor binding.
    scene
        .event_dispatcher_mut()
        .add_listener(EventListener::new(
            EventType::KeyPress,
            EventTarget::Global,
            |event, _, _, _| {
                if matches!(&event.payload, EventPayload::KeyPress { key, .. } if *key == CRASH_KEY)
                {
                    panic!("native live acceptance: deliberate worker callback panic");
                }
                EventPropagation::Continue
            },
        ))
        .map_err(execution_error)?;
    NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(rotate(root, PI))],
                overrides: PlayOverrides::default(),
            },
            // This object must not exist at frame zero. Its location comes from
            // the first animation's completed state, not a planning-time copy.
            NativeSegment::defer(move |context| {
                let stage = context.stage_mut();
                let center = stage.get_center(root);
                let mut buffer = stage
                    .get(root)
                    .ok_or(SceneError::InvalidState("the opening object is missing"))?
                    .buffer
                    .deep_clone();
                let count = buffer.len();
                buffer.write_range("fill_rgba", 0, &[1.0, 0.35, 0.08, 1.0].repeat(count));
                let late = stage.add(Mobject::from_buffer(buffer));
                stage.shift_many(&[late], center);
                stage.remove_from_scene(root);
                stage.add_to_scene(late)?;
                Ok(vec![
                    NativeSegment::Play {
                        animations: vec![Box::new(rotate(late, PI / 2.0))],
                        overrides: PlayOverrides::default(),
                    },
                    // Native callback state is created only once this new
                    // object's own animation finishes, and rebuilt on recovery.
                    NativeSegment::edit(move |context| {
                        let mut tick = 0_u32;
                        context.stage_mut().add_updater(
                            late,
                            move |stage, target| {
                                tick += 1;
                                let delta = if tick % 120 < 60 { 0.02 } else { -0.02 };
                                stage.shift_many(&[target], [delta, 0.0, 0.0]);
                            },
                            false,
                        )?;
                        Ok(())
                    }),
                    NativeSegment::Wait {
                        duration: Some(28.0),
                    },
                ])
            }),
        ],
        900,
    )
}

fn config(build_id: ProtocolDigest) -> NativeWorkerConfig {
    let renderer = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 640,
                height: 360,
            },
            ScreenMap {
                scale: 45.0,
                origin: [320.0, 180.0],
                y_up: true,
            },
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ),
        tiling: Tiling {
            macro_tile: 64,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads: 1,
    };
    let mut config = NativeWorkerConfig::new(
        NAME,
        build_id,
        protocol_digest(include_bytes!("native_live.rs")),
        901,
        30,
        renderer,
    );
    // This demo has no external assets, I/O or shared factory captures.
    config.replay = NativeReplayPolicy::ColdVerified;
    config.checkpoint_frames = 30;
    config
}

struct SameExecutable(WorkerArtifact);
impl RebuildDriver for SameExecutable {
    fn rebuild(&mut self) -> std::result::Result<WorkerArtifact, BuildError> {
        // Restart the exact already-built example; this is not an incremental
        // Rust compiler. A changed binary is rejected by the build handshake.
        Ok(self.0.clone())
    }
}

fn worker_response(reply: SupervisorReply) -> Result<WorkerResponse> {
    match reply {
        SupervisorReply::Worker(WorkerResponse::Error { message, .. }) => Err(message.into()),
        SupervisorReply::Worker(response) => Ok(response),
        SupervisorReply::Recovered { .. } => Err("worker recovered; retry the operation".into()),
    }
}

fn preview(
    supervisor: &mut Supervisor,
    frame: i64,
    asset_ok: &dyn Fn(&AssetRead) -> bool,
) -> Result<FrameStream> {
    match worker_response(supervisor.request(
        SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame,
        },
        asset_ok,
    )?)? {
        WorkerResponse::Frame(frame) => Ok(frame),
        _ => Err("native worker did not produce a frame".into()),
    }
}

fn process_smoke(supervisor: &mut Supervisor, asset_ok: &dyn Fn(&AssetRead) -> bool) -> Result<()> {
    // Past both late construction and updater registration: the replacement
    // must rebuild the *new* callback state, not only the opening object.
    worker_response(supervisor.request(
        SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 77)?,
        },
        asset_ok,
    )?)?;
    let before = preview(supervisor, 80, asset_ok)?;
    let generation = supervisor.generation();
    let recovered = supervisor.request(
        SupervisorRequest::Event {
            scene: NAME.into(),
            event: EventPayload::KeyPress {
                key: CRASH_KEY,
                modifiers: Modifiers::NONE,
            },
        },
        asset_ok,
    )?;
    assert!(matches!(recovered, SupervisorReply::Recovered { .. }));
    assert_eq!(supervisor.generation(), generation + 1);
    assert_eq!(supervisor.crashes().len(), 1);
    let after = preview(supervisor, 80, asset_ok)?;
    assert_eq!(
        before, after,
        "replacement must reconstruct deferred objects, callbacks and pixels"
    );
    Ok(())
}

fn http(host: &StudioHost, cap: &str, method: &str, route: &str, body: &str) -> Result<String> {
    let addr = host.local_addr()?;
    let response = std::thread::scope(|scope| -> std::io::Result<Vec<u8>> {
        // Connect before spawning the blocking accept, so a refused connection
        // cannot leave the scope waiting forever for an unstarted client.
        let mut client = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        client.set_read_timeout(Some(Duration::from_secs(10)))?;
        client.set_write_timeout(Some(Duration::from_secs(10)))?;
        let server = scope.spawn(|| {
            host.serve_once()
                .map_err(|error| std::io::Error::other(error.to_string()))
        });
        write!(
            client,
            "{method} {route}?cap={cap} HTTP/1.1\r\nHost: {addr}\r\nOrigin: http://{addr}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        let mut response = Vec::new();
        client.read_to_end(&mut response)?;
        server
            .join()
            .map_err(|_| std::io::Error::other("HTTP host thread panicked"))??;
        Ok(response)
    })?;
    let text = String::from_utf8(response)?;
    if !text.starts_with("HTTP/1.1 200 ") {
        return Err(format!("native HTTP operation failed: {text}").into());
    }
    Ok(text)
}

fn http_smoke(host: &StudioHost, frames: &FrameHub, cap: &str) -> Result<()> {
    http(host, cap, "GET", "/", "")?;
    http(host, cap, "POST", "/api/scrub", "frame=29&commit=false")?;
    let opening = frames.latest().ok_or("missing opening native frame")?;
    http(host, cap, "POST", "/api/scrub", "frame=37&commit=true")?;
    let before = frames.latest().ok_or("missing constructed native frame")?;
    assert_eq!(before.frame_index, 37);
    assert_ne!(
        opening.png, before.png,
        "later construction must be visible through HTTP"
    );
    http(
        host,
        cap,
        "POST",
        "/api/event",
        "type=key_press&key=a&modifiers=2",
    )?;
    http(
        host,
        cap,
        "POST",
        "/api/event",
        "type=key_press&key=arrow_up&modifiers=1",
    )?;
    let edited = frames.latest().ok_or("missing edited native frame")?;
    assert_ne!(before.png, edited.png);
    let inspection = http(host, cap, "GET", "/api/inspect", "")?;
    assert!(inspection.contains("\"input_events\":true"));
    assert!(inspection.contains("\"frame_index\":37"));
    http(
        host,
        cap,
        "POST",
        "/api/event",
        "type=key_press&key=z&modifiers=2",
    )?;
    assert_eq!(frames.latest().ok_or("missing undo frame")?.png, before.png);
    http(host, cap, "GET", "/api/overlays", "")?;
    http(host, cap, "POST", "/api/restart", "")?;
    assert_eq!(
        frames.latest().ok_or("missing restarted frame")?.png,
        before.png
    );
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let worker_mode = args.len() == 1 && args[0] == WORKER_ARG;
    let self_test = args.len() == 1 && args[0] == "--self-test";
    if !args.is_empty() && !worker_mode && !self_test {
        return Err("usage: native_live [--self-test]".into());
    }
    let executable = std::env::current_exe()?;
    // Host composition only: hash the exact executable the launcher will own.
    // Scene construction/playback never reads the filesystem or ambient clock.
    let build_id = protocol_digest(&std::fs::read(&executable)?);
    let config = config(build_id);
    let limits = config.limits;
    if worker_mode {
        let mut worker = NativeSceneWorker::new(config, program)?;
        serve_worker(
            &mut worker,
            &mut std::io::stdin().lock(),
            &mut std::io::stdout().lock(),
            limits,
        )?;
        return Ok(());
    }
    let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
    // Host-owned, bounded process-lifetime cache; no files are left behind.
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let cache = Store::open(
        fs,
        Arc::clone(&clock),
        std::env::temp_dir().join("fmn-native-live-cache"),
        StoreConfig::default(),
    )?
    .namespace(
        "studio-replay",
        1,
        NamespacePolicy {
            ceiling_bytes: Some(64 * 1024 * 1024),
        },
    )?;
    let mut builder = SameExecutable(WorkerArtifact {
        executable,
        argv: vec![WORKER_ARG.into()],
        env: Vec::new(),
        cwd: None,
        build_id,
    });
    let mut supervisor = Supervisor::new(
        Box::new(StdWorkerLauncher::default()),
        Arc::clone(&clock),
        cache,
        SupervisorConfig {
            protocol_limits: limits,
            ..SupervisorConfig::default()
        },
    );
    supervisor.install_session(NAME, Journal::new())?;
    supervisor.build_and_start(&mut builder)?;
    let initial = worker_response(supervisor.request(
        SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 0)?,
        },
        &|_| false,
    )?)?;
    let WorkerResponse::JournalSegment { journal, .. } = initial else {
        return Err("missing initial native journal".into());
    };
    let journal = Journal::from_bytes(&journal)?;
    let reads = journal
        .entries()
        .first()
        .ok_or("empty initial journal")?
        .reads
        .clone();
    // The same immutable binary is the entire demo closure. No external asset
    // contents can change independently of its exact build handshake identity.
    let asset_ok: Arc<dyn Fn(&AssetRead) -> bool + Send + Sync> =
        Arc::new(move |read| reads.contains(read));
    if self_test {
        process_smoke(&mut supervisor, &*asset_ok)?;
    }
    let initial = preview(&mut supervisor, 0, &*asset_ok)?;
    let host_config = StudioHostConfig::default();
    let frames = FrameHub::new(host_config.max_frame_history, host_config.max_png_bytes)?;
    frames.publish(&initial, limits)?;
    let session = Arc::new(StudioWorkerSession::new(
        NAME,
        supervisor,
        Box::new(builder),
        asset_ok,
    )?);
    let mut secret = [0_u8; 32];
    StdHostEntropy.fill(&mut secret)?;
    let token = CapabilityToken::new(secret)?;
    let cap = token.try_expose_hex()?;
    let host = StudioHost::bind(
        Arc::clone(&session),
        frames.clone(),
        token,
        clock,
        host_config,
    )?;
    let result = if self_test {
        let result = http_smoke(&host, &frames, &cap);
        if result.is_ok() {
            println!(
                "OK: deferred native construction, late callback crash/recovery, HTTP editing/undo/inspection/overlays/restart and PNG identity"
            );
        }
        result
    } else {
        println!("{}", host.launch_url()?);
        eprintln!("Open this private URL. Press Enter in this terminal to close Studio.");
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        std::thread::Builder::new()
            .name("native-studio-stop".into())
            .spawn(move || {
                let _ = std::io::stdin().read_line(&mut String::new());
                stop.store(true, Ordering::Release);
            })?;
        host.serve_until(&shutdown).map_err(Into::into)
    };
    frames.close();
    session.shutdown_worker();
    result
}

fn main() {
    if let Err(error) = run() {
        eprintln!("native Studio: {error}");
        std::process::exit(1);
    }
}
