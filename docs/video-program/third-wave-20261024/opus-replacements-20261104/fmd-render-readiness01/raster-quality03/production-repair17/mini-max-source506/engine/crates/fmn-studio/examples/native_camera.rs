//! Native camera Studio: animated camera/light, surface, raster image and glow.
//! Run with `cargo run -p fmn-studio --example native_camera`.
//! `--self-test` drives the real isolated worker and authenticated HTTP host.

use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use fmn_anim::{Animation, rotate};
use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_core::color::LinearRgba;
use fmn_core::constants::PI;
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, Mobject, RecordBuffer, RecordSchema,
    RenderPrimitive, Uniforms,
};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::entropy::{HostEntropy, StdHostEntropy};
use fmn_platform::fs::{FileSystem, StdFs, VirtualFs};
use fmn_render::{
    CameraConfig, CameraFrame, EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap,
    Tiling, Viewport,
};
use fmn_scene::{AssetRead, CameraRig, Journal, PlayOverrides, RuntimeConfig, Scene};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::protocol::studio_seek_command;
use fmn_studio::{
    BuildError, CapabilityToken, FrameHub, ProtocolDigest, RebuildDriver, ServiceError,
    StdWorkerLauncher, StudioHost, StudioHostConfig, StudioWorkerSession, Supervisor,
    SupervisorConfig, SupervisorReply, SupervisorRequest, WorkerArtifact, WorkerErrorCode,
    WorkerResponse, WorkerServeOutcome, protocol_digest, serve_worker,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const NAME: &str = "NativeCamera";
const WORKER_ARG: &str = "--native-camera-worker";

fn error(error: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, error.to_string())
}

fn surface() -> Result<Mobject> {
    const SIDE: usize = 17;
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
        &["point"],
        &["point", "d_normal_point"],
    )?;
    let mut buffer = RecordBuffer::new(schema, SIDE * SIDE)?;
    for u in 0..SIDE {
        for v in 0..SIDE {
            let x = -2.0 + u as f32 * 0.25;
            let y = -2.0 + v as f32 * 0.25;
            let z = 0.25 * (x * x - y * y);
            let index = u * SIDE + v;
            buffer.write(index, "point", &[x, y, z]);
            buffer.write(
                index,
                "d_normal_point",
                &[x - 0.5 * x, y + 0.5 * y, z + 1.0],
            );
            buffer.write(index, "rgba", &[0.2 + u as f32 * 0.035, 0.55, 0.9, 1.0]);
        }
    }
    Ok(Mobject::from_buffer(buffer)
        .with_render_primitive(RenderPrimitive::SurfaceGrid {
            resolution: (SIDE, SIDE),
        })
        .with_uniforms(Uniforms {
            depth_test: true,
            shading: [0.3, 0.2, 0.4],
            ..Uniforms::default()
        }))
}

fn image() -> Result<Mobject> {
    let schema = RecordSchema::new(
        &[("point", 3), ("im_coords", 2), ("opacity", 1)],
        &["point"],
        &["point"],
    )?;
    let mut buffer = RecordBuffer::new(schema, 6)?;
    let vertices = [
        ([4.0, 3.0, 0.0], [0.0, 0.0]),
        ([4.0, 1.5, 0.0], [0.0, 1.0]),
        ([5.5, 3.0, 0.0], [1.0, 0.0]),
        ([5.5, 3.0, 0.0], [1.0, 0.0]),
        ([4.0, 1.5, 0.0], [0.0, 1.0]),
        ([5.5, 1.5, 0.0], [1.0, 1.0]),
    ];
    for (index, (point, uv)) in vertices.into_iter().enumerate() {
        buffer.write(index, "point", &point);
        buffer.write(index, "im_coords", &uv);
        buffer.write(index, "opacity", &[1.0]);
    }
    let image = ImageResource::rgba8(
        2,
        2,
        vec![
            255, 70, 30, 255, 40, 230, 100, 255, 50, 90, 255, 255, 255, 230, 70, 255,
        ],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )?;
    Ok(Mobject::from_buffer(buffer)
        .with_image_resource(image)
        .with_uniforms(Uniforms {
            is_fixed_in_frame: 1.0,
            ..Uniforms::default()
        }))
}

fn glow() -> Result<Mobject> {
    let schema = RecordSchema::new(
        &[("point", 3), ("rgba", 4), ("radius", 1), ("glow_factor", 1)],
        &["point"],
        &["point"],
    )?;
    let mut buffer = RecordBuffer::new(schema, 1)?;
    buffer.write(0, "point", &[-3.0, 0.0, 0.8]);
    buffer.write(0, "rgba", &[0.1, 1.0, 0.6, 1.0]);
    buffer.write(0, "radius", &[1.2]);
    buffer.write(0, "glow_factor", &[2.0]);
    Ok(Mobject::from_buffer(buffer).with_render_primitive(RenderPrimitive::DotCloud))
}

fn camera_config() -> Result<CameraConfig> {
    let mut frame = CameraFrame::default();
    frame.set_euler_angles(Some(-0.4), Some(0.9), Some(0.0))?;
    Ok(CameraConfig {
        resolution: (320, 180),
        fps: 30,
        background: LinearRgba {
            r: 0.01,
            g: 0.015,
            b: 0.025,
            a: 1.0,
        },
        frame,
        ..CameraConfig::default()
    })
}

fn program() -> std::result::Result<NativeSceneProgram, ServiceError> {
    let mut scene = Scene::new(RuntimeConfig::default(), 71).map_err(error)?;
    let surface = scene.stage_mut().add(surface().map_err(error)?);
    let image = scene.stage_mut().add(image().map_err(error)?);
    let glow = scene.stage_mut().add(glow().map_err(error)?);
    scene.add(&[surface, glow, image]).map_err(error)?;
    let base = camera_config().map_err(error)?;
    let rig = CameraRig::new(&mut scene, &base).map_err(error)?;
    let mut destination = base.clone();
    destination
        .frame
        .set_center([0.5, 0.0, 0.1])
        .map_err(error)?;
    destination.frame.set_width(10.0).map_err(error)?;
    destination
        .frame
        .set_euler_angles(Some(0.25), Some(1.1), Some(0.0))
        .map_err(error)?;
    destination.light_source_position = [4.0, 6.0, 10.0];
    // The camera and surface share one Play and one ordinary native clock.
    let mut movement = rig.animate_to(&mut scene, &destination).map_err(error)?;
    // Transform leaves source updaters active by default. This authored
    // transition deliberately pauses them, then lets the normal finish resume
    // the camera drift during the following wait.
    movement.state_mut().config.suspend_mobject_updating = true;
    let mut ticks = 0_u32;
    scene
        .stage_mut()
        .add_dt_updater(
            rig.center()[0],
            move |stage, x, dt| {
                ticks += 1;
                if let Some(value) = stage.tracker_value(x) {
                    let _ = stage
                        .set_tracker_value(x, value + dt * (0.025 + f64::from(ticks) * 0.0001));
                }
            },
            false,
        )
        .map_err(error)?;
    NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(rotate(surface, PI)), Box::new(movement)],
                overrides: PlayOverrides {
                    run_time: Some(4.0),
                    ..PlayOverrides::default()
                },
            },
            NativeSegment::Wait {
                duration: Some(6.0),
            },
        ],
        300,
    )?
    .with_camera_rig(rig)
}

fn config(build_id: ProtocolDigest) -> Result<NativeWorkerConfig> {
    let camera = camera_config()?;
    let renderer = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 320,
                height: 180,
            },
            ScreenMap {
                scale: 22.5,
                origin: [160.0, 90.0],
                y_up: true,
            },
            camera.background,
        ),
        tiling: Tiling {
            macro_tile: 64,
            fine_tile: 8,
        },
        engine: EngineIdentity::certified(),
        threads: 1,
    };
    let mut policy = NativeWorkerConfig::new(
        NAME,
        build_id,
        protocol_digest(include_bytes!("native_camera.rs")),
        301,
        30,
        renderer,
    );
    policy.camera = Some(camera);
    policy.replay = NativeReplayPolicy::ColdVerified;
    Ok(policy)
}

struct SameExecutable(WorkerArtifact);
impl RebuildDriver for SameExecutable {
    fn rebuild(&mut self) -> std::result::Result<WorkerArtifact, BuildError> {
        Ok(self.0.clone())
    }
}

fn response(reply: SupervisorReply) -> Result<WorkerResponse> {
    match reply {
        SupervisorReply::Worker(WorkerResponse::Error { message, .. }) => Err(message.into()),
        SupervisorReply::Worker(response) => Ok(response),
        SupervisorReply::Recovered { .. } => Err("worker recovered; retry the operation".into()),
    }
}

fn http(host: &StudioHost, cap: &str, method: &str, route: &str, body: &str) -> Result<String> {
    let addr = host.local_addr()?;
    let bytes = std::thread::scope(|scope| -> std::io::Result<Vec<u8>> {
        let mut client = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;
        client.set_read_timeout(Some(Duration::from_secs(15)))?;
        client.set_write_timeout(Some(Duration::from_secs(15)))?;
        let server = scope.spawn(|| {
            host.serve_once()
                .map_err(|error| std::io::Error::other(error.to_string()))
        });
        write!(
            client,
            "{method} {route}?cap={cap} HTTP/1.1\r\nHost: {addr}\r\nOrigin: http://{addr}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        let mut bytes = Vec::new();
        client.read_to_end(&mut bytes)?;
        server
            .join()
            .map_err(|_| std::io::Error::other("camera HTTP handler panicked"))??;
        Ok(bytes)
    })?;
    let text = String::from_utf8(bytes)?;
    if !text.starts_with("HTTP/1.1 200 ") {
        return Err(format!("camera HTTP operation failed: {text}").into());
    }
    Ok(text)
}

fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let worker_mode = args.len() == 1 && args[0] == WORKER_ARG;
    let self_test = args.len() == 1 && args[0] == "--self-test";
    if !args.is_empty() && !worker_mode && !self_test {
        return Err("usage: native_camera [--self-test]".into());
    }
    let executable = std::env::current_exe()?;
    let build_id = protocol_digest(&StdFs.read_bounded(&executable, 512 * 1024 * 1024)?);
    let config = config(build_id)?;
    let limits = config.limits;
    if worker_mode {
        let mut worker = NativeSceneWorker::new(config, program)?;
        return match serve_worker(
            &mut worker,
            &mut std::io::stdin().lock(),
            &mut std::io::stdout().lock(),
            limits,
        )? {
            WorkerServeOutcome::Shutdown | WorkerServeOutcome::PeerClosed => Ok(()),
            WorkerServeOutcome::Crashed(report) => Err(report.message.into()),
            WorkerServeOutcome::HandshakeRejected => Err("camera worker handshake rejected".into()),
        };
    }
    let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let cache = Store::open(
        fs,
        Arc::clone(&clock),
        std::env::temp_dir().join("fmn-camera-cache"),
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
    let WorkerResponse::JournalSegment { journal, .. } = response(supervisor.request(
        SupervisorRequest::Play {
            scene: NAME.into(),
            command: studio_seek_command(NAME, 0)?,
        },
        &|_| false,
    )?)?
    else {
        return Err("missing camera journal".into());
    };
    let journal = Journal::from_bytes(&journal)?;
    let reads = journal
        .entries()
        .first()
        .ok_or("empty camera journal")?
        .reads
        .clone();
    let asset_ok: Arc<dyn Fn(&AssetRead) -> bool + Send + Sync> =
        Arc::new(move |read| reads.contains(read));
    let WorkerResponse::Frame(initial) = response(supervisor.request(
        SupervisorRequest::Scrub {
            scene: NAME.into(),
            frame: 0,
        },
        &*asset_ok,
    )?)?
    else {
        return Err("missing camera frame".into());
    };
    let host_config = StudioHostConfig::default();
    let frames = FrameHub::new(host_config.max_frame_history, host_config.max_png_bytes)?;
    let first = frames.publish(&initial, limits)?;
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
        http(&host, &cap, "GET", "/", "")?;
        http(&host, &cap, "POST", "/api/scrub", "frame=37&commit=true")?;
        let moved = frames.latest().ok_or("missing animated camera frame")?;
        assert_ne!(
            first.png, moved.png,
            "real scene and camera animation must change pixels"
        );
        let inspection = http(&host, &cap, "GET", "/api/inspect", "")?;
        assert!(inspection.contains("\"input_events\":false"));
        assert!(inspection.contains("\"frame_index\":37"));
        http(&host, &cap, "POST", "/api/restart", "")?;
        assert_eq!(
            frames.latest().ok_or("missing recovered camera frame")?.png,
            moved.png
        );
        // Beyond the Transform: recover a live, stateful camera updater too.
        http(&host, &cap, "POST", "/api/scrub", "frame=157&commit=true")?;
        let drifting = frames.latest().ok_or("missing camera updater frame")?;
        assert_ne!(drifting.png, moved.png);
        http(&host, &cap, "POST", "/api/restart", "")?;
        assert_eq!(
            frames.latest().ok_or("missing restored updater frame")?.png,
            drifting.png
        );
        println!(
            "OK: animated camera/light, native surface/image/dot PNGs, HTTP inspection and camera-updater restart identity"
        );
        Ok(())
    } else {
        println!("{}", host.launch_url()?);
        eprintln!("Scrub the animated camera scene in Studio. Press Enter here to stop.");
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        std::thread::Builder::new()
            .name("camera-studio-stop".into())
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
        eprintln!("native camera Studio: {error}");
        std::process::exit(1);
    }
}
