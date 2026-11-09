//! Copied into a private Cargo project by project_cargo acceptance.
use fmn_core::color::LinearRgba;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::{RuntimeConfig, Scene};
use fmn_studio::native::{
    NativeReplayPolicy, NativeSceneProgram, NativeSceneWorker, NativeSegment, NativeWorkerConfig,
};
use fmn_studio::{
    ServiceError, WorkerErrorCode, WorkerServeOutcome, protocol_digest, serve_worker,
};

const OFFSET: f64 = 0.0;
fn error(value: impl std::fmt::Display) -> ServiceError {
    ServiceError::new(WorkerErrorCode::ExecutionFailed, value.to_string())
}
fn program() -> Result<NativeSceneProgram, ServiceError> {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        71,
    )
    .map_err(error)?;
    let points = [
        [-0.5_f32, -0.5, 0.0],
        [0.0, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.0, 0.0],
        [0.5, 0.5, 0.0],
        [0.0, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
        [-0.5, 0.0, 0.0],
        [-0.5, -0.5, 0.0],
    ];
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).map_err(error)?;
    for (index, point) in points.iter().enumerate() {
        buffer.write(index, "point", point);
        buffer.write(index, "fill_rgba", &[0.2, 0.7, 1.0, 1.0]);
        buffer.write(index, "stroke_width", &[0.0]);
    }
    let root = scene.stage_mut().add(Mobject::from_buffer(buffer));
    scene.stage_mut().shift_many(&[root], [OFFSET, 0.0, 0.0]);
    scene.add(&[root]).map_err(error)?;
    NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(1.0),
        }],
        8,
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    if std::fs::metadata(&executable)?.len() > 512 * 1024 * 1024 {
        return Err("worker too large".into());
    }
    let build = protocol_digest(&std::fs::read(executable)?);
    let renderer = RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 64,
                height: 64,
            },
            ScreenMap {
                scale: 16.0,
                origin: [32.0, 32.0],
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
        "ProjectScene",
        build,
        protocol_digest(include_bytes!("main.rs")),
        9,
        8,
        renderer,
    );
    config.replay = NativeReplayPolicy::ColdVerified;
    let limits = config.limits;
    let mut worker = NativeSceneWorker::new(config, program)?;
    match serve_worker(
        &mut worker,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        limits,
    )? {
        WorkerServeOutcome::Shutdown | WorkerServeOutcome::PeerClosed => Ok(()),
        _ => Err("worker terminated abnormally".into()),
    }
}
