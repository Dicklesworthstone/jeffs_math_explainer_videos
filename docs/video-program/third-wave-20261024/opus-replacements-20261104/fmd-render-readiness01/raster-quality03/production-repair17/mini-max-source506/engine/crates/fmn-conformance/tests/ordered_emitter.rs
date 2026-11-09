//! Real Lumen → runtime pipeline → Reel emitter proof for fm-hv4.
//!
//! Lumen renders directly into a preallocated emitter reservation. No
//! frame-sized temporary sits between rasterization and publication, and the
//! sink receives scalar-definition bytes in frame-index order at every queue
//! depth.

use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, GREEN_B, RED_C, WHITE};
use fmn_frame::FrameBuffer;
use fmn_library::Circle;
use fmn_library::VStyle;
use fmn_mobject::Stage;
use fmn_output::{
    EmitterConfig, FrameReservation, FrameSink, OrderedEmitter, SinkBinding, SinkFailure, SinkWrite,
};
use fmn_platform::topology::HardwareTopology;
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob, encode_frame};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use fmn_runtime::{
    ExecutionPlan, FramePipeline, OutputPixelFormat, PipelineEvent, PipelineStages, PlanRequest,
    RenderIntent, SurfaceSpec, TeamPlan,
};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 48;
const FRAME_COUNT: u64 = 9;

#[derive(Debug, Clone, Copy)]
struct FrameSpec {
    sequence: u64,
}

#[derive(Debug)]
struct ReservedFrame {
    spec: FrameSpec,
    reservation: FrameReservation,
}

#[derive(Debug)]
struct PreparedFrame {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
    reservation: FrameReservation,
}

#[derive(Debug)]
struct EmitterStages {
    tiling: Tiling,
}

impl PipelineStages for EmitterStages {
    type Frame = ReservedFrame;
    type Prepared = PreparedFrame;
    type Rasterized = FrameReservation;
    type Output = FrameReservation;
    type Error = String;

    fn prepare(
        &self,
        frame: Self::Frame,
        _scene_team: &TeamPlan,
    ) -> Result<Self::Prepared, Self::Error> {
        let (plan, mono, binning, config) = prepare_without_reservation(frame.spec, self.tiling);
        Ok(PreparedFrame {
            plan,
            mono,
            binning,
            config,
            reservation: frame.reservation,
        })
    }

    fn rasterize(
        &self,
        prepared: Self::Prepared,
        render_team: &TeamPlan,
    ) -> Result<Self::Rasterized, Self::Error> {
        let PreparedFrame {
            plan,
            mono,
            binning,
            config,
            mut reservation,
        } = prepared;
        let job =
            FrameJob::new(&plan, &mono, &binning, config).map_err(|error| error.to_string())?;
        job.render_into(render_team.threads(), reservation.frame_mut())
            .map_err(|error| error.to_string())?;
        Ok(reservation)
    }

    fn convert(
        &self,
        rasterized: Self::Rasterized,
        _output_team: &TeamPlan,
    ) -> Result<Self::Output, Self::Error> {
        Ok(rasterized)
    }
}

fn frame_config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        ScreenMap {
            scale: 18.0,
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: true,
        },
        Srgb::from_rgb8(0x18, 0x18, 0x18).to_linear(1.0),
    )
}

fn prepare_without_reservation(
    spec: FrameSpec,
    tiling: Tiling,
) -> (RenderPlan, MonoTable, Binning, FrameConfig) {
    let phase = spec.sequence as f64 / FRAME_COUNT as f64;
    let color = match spec.sequence % 3 {
        0 => BLUE_C,
        1 => GREEN_B,
        _ => RED_C,
    };
    let mut stage = Stage::new();
    let circle = stage.add(Circle::new().radius(0.3 + 0.22 * phase).arc_center([
        -0.9 + 1.8 * phase,
        0.0,
        0.0,
    ]));
    stage.add_to_scene(circle).expect("new circle is live");
    stage.set_fill(circle, Some(color), Some(0.8), Some(0.0), true);
    stage.set_stroke(circle, Some(WHITE), Some(1.5), Some(0.9), None, true);

    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&stage, spec.sequence)
        .expect("valid ordered-emitter fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded ordered-emitter monotone table");
    let binning = Binning::build(&plan, config.viewport, tiling, config.map)
        .expect("bounded conformance binning");
    (plan, mono, binning, config)
}

fn scalar_definition(spec: FrameSpec, tiling: Tiling) -> Vec<u8> {
    let (plan, mono, binning, config) = prepare_without_reservation(spec, tiling);
    let job = FrameJob::new(&plan, &mono, &binning, config).expect("matching artifacts");
    encode_frame(&job.render(1).expect("scalar render")).expect("canonical bytes")
}

fn execution_plan(depth: usize) -> ExecutionPlan {
    ExecutionPlan::derive(
        PlanRequest::certified(
            RenderIntent::Offline,
            SurfaceSpec::lumen(WIDTH, HEIGHT),
            OutputPixelFormat::Rgba16F,
        )
        .with_max_frames_in_flight(depth)
        .with_max_cpu_threads(16),
        &HardwareTopology::fallback(16),
        None,
    )
    .expect("fixture topology yields a plan")
}

type RecordedFrames = Arc<Mutex<Vec<(u64, Vec<u8>)>>>;

struct CanonicalSink {
    frames: RecordedFrames,
}

impl FrameSink for CanonicalSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let bytes = encode_frame(frame).map_err(|error| SinkFailure::new(error.to_string()))?;
        self.frames
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((sequence, bytes));
        Ok(SinkWrite::Consumed)
    }
}

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(PoisonError::into_inner)
}

#[test]
fn pooled_lumen_frames_reach_sinks_in_order_at_every_queue_depth() {
    let tiling = Tiling {
        macro_tile: 128,
        fine_tile: 16,
    };
    let specs = (0..FRAME_COUNT)
        .map(|sequence| FrameSpec { sequence })
        .collect::<Vec<_>>();
    let expected = specs
        .iter()
        .copied()
        .map(|spec| (spec.sequence, scalar_definition(spec, tiling)))
        .collect::<Vec<_>>();
    let stages = EmitterStages { tiling };

    for depth in 1..=4 {
        let frames = Arc::new(Mutex::new(Vec::new()));
        let emitter = OrderedEmitter::new(
            EmitterConfig::new(frame_config().layout().expect("raw-frame layout"), depth, 0)
                .expect("ring budget"),
            vec![SinkBinding::reliable(
                "canonical",
                CanonicalSink {
                    frames: Arc::clone(&frames),
                },
            )],
        )
        .expect("emitter");
        let handle = emitter.handle();
        let events = specs.iter().copied().map(move |spec| {
            let reservation = handle
                .reserve(spec.sequence)
                .expect("pipeline dispatch reserves in order");
            PipelineEvent::<_, ()>::frame(spec.sequence, ReservedFrame { spec, reservation })
        });

        let plan = execution_plan(depth);
        let pipeline = FramePipeline::new(&plan, &stages)
            .run(
                events,
                |sequence, reservation| {
                    assert_eq!(reservation.sequence(), sequence);
                    reservation.publish().map_err(|error| error.to_string())
                },
                |(), _context| Ok(()),
            )
            .expect("pipeline");
        let output = emitter.finish().expect("ordered sink drain");

        assert_eq!(*lock(&frames), expected, "queue depth {depth}");
        assert_eq!(pipeline.emitted, FRAME_COUNT);
        assert_eq!(pipeline.outstanding_slots, 0);
        assert_eq!(output.stats.emitted, FRAME_COUNT);
        assert_eq!(output.stats.outstanding, 0);
        assert!(output.stats.max_outstanding <= depth);
        assert_eq!(output.stats.ring_bytes, depth * output.stats.frame_bytes);
    }
}
