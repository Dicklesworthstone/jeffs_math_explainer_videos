//! Real Lumen coverage for fm-3df's bounded frame pipeline.
//!
//! The runtime crate deliberately schedules opaque owned values. This test is
//! the adapter proof: a frozen frame becomes a retained render plan and bins,
//! independent render teams rasterize it, and the ordered seam emits exactly
//! the scalar-definition bytes at every supported queue depth.

use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, GREEN_B, RED_C, WHITE, YELLOW_C};
use fmn_library::style::VStyle;
use fmn_library::{Circle, Rectangle};
use fmn_mobject::{Mob, Stage};
use fmn_platform::clock::{Clock, StdClock};
use fmn_platform::topology::HardwareTopology;
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob, encode_frame};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use fmn_runtime::{
    AutotuneCache, AutotuneProfile, CancellationToken, ExecutionPlan, FramePipeline,
    OutputPixelFormat, PipelineEvent, PipelineStages, PlanRequest, ProfilePath, ProfilePhase,
    ProfileRecord, ProfileRecorder, RenderIntent, SurfaceSpec, TeamPlan, TopologyFingerprint,
};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const WIDTH: u32 = 96;
const HEIGHT: u32 = 64;
const SCALE: f64 = 22.0;
const FRAME_COUNT: u64 = 12;

#[derive(Debug, Clone, Copy)]
struct FrameSpec {
    sequence: u64,
}

#[derive(Debug)]
struct PreparedFrame {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
}

struct LumenStages {
    tiling: Tiling,
    profile: ProfileRecorder,
    clock: Arc<dyn Clock>,
    profile_path: ProfilePath,
}

#[derive(Default)]
struct DisabledProfileClock {
    reads: AtomicUsize,
}

impl Clock for DisabledProfileClock {
    fn monotonic(&self) -> std::time::Duration {
        self.reads.fetch_add(1, Ordering::Relaxed);
        std::time::Duration::ZERO
    }

    fn wall(&self) -> std::time::SystemTime {
        self.reads.fetch_add(1, Ordering::Relaxed);
        std::time::SystemTime::UNIX_EPOCH
    }
}

impl PipelineStages for LumenStages {
    type Frame = FrameSpec;
    type Prepared = PreparedFrame;
    type Rasterized = Vec<u8>;
    type Output = Vec<u8>;
    type Error = String;

    fn prepare(
        &self,
        frame: Self::Frame,
        _scene_team: &TeamPlan,
    ) -> Result<Self::Prepared, Self::Error> {
        let profile_path = self.profile_path.with_frame(frame.sequence);
        Ok(prepare_profiled(
            frame,
            self.tiling,
            &self.profile,
            self.clock.as_ref(),
            profile_path,
        ))
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
        } = prepared;
        let job =
            FrameJob::new(&plan, &mono, &binning, config).map_err(|error| error.to_string())?;
        let frame = job
            .render(render_team.threads())
            .map_err(|error| error.to_string())?;
        encode_frame(&frame).map_err(|error| error.to_string())
    }

    fn convert(
        &self,
        rasterized: Self::Rasterized,
        _output_team: &TeamPlan,
    ) -> Result<Self::Output, Self::Error> {
        Ok(rasterized)
    }
}

fn add(stage: &mut Stage, mobject: impl Into<fmn_mobject::Mobject>) -> Mob {
    let mob = stage.add(mobject);
    stage.add_to_scene(mob).expect("new mobject is live");
    mob
}

fn frame_config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        ScreenMap {
            scale: SCALE,
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: true,
        },
        Srgb::from_rgb8(0x18, 0x18, 0x18).to_linear(1.0),
    )
}

fn prepare(spec: FrameSpec, tiling: Tiling) -> PreparedFrame {
    let clock = DisabledProfileClock::default();
    let prepared = prepare_profiled(
        spec,
        tiling,
        &ProfileRecorder::disabled(),
        &clock,
        ProfilePath::scene(0).with_frame(spec.sequence),
    );
    assert_eq!(clock.reads.load(Ordering::Relaxed), 0);
    prepared
}

fn prepare_profiled(
    spec: FrameSpec,
    tiling: Tiling,
    profile: &ProfileRecorder,
    clock: &dyn Clock,
    profile_path: ProfilePath,
) -> PreparedFrame {
    let mut stage = Stage::new();
    let phase = spec.sequence as f64 / FRAME_COUNT as f64;
    let color = match spec.sequence % 4 {
        0 => BLUE_C,
        1 => GREEN_B,
        2 => RED_C,
        _ => YELLOW_C,
    };

    let circle = add(
        &mut stage,
        Circle::new()
            .radius(0.38 + 0.16 * phase)
            .arc_center([-1.15 + 2.3 * phase, -0.18, 0.0]),
    );
    stage.set_fill(circle, Some(color), Some(0.78), Some(0.0), true);
    stage.set_stroke(circle, Some(WHITE), Some(2.0), Some(0.85), None, true);

    let rectangle = add(
        &mut stage,
        Rectangle::new()
            .width(1.1)
            .height(0.58)
            .color(WHITE)
            .build()
            .expect("the frame-pipeline rectangle is unrounded"),
    );
    stage.shift(rectangle, [0.62 - 1.24 * phase, 0.42, 0.0]);
    stage.set_fill(
        rectangle,
        Some(YELLOW_C),
        Some(0.24 + 0.5 * phase),
        Some(0.0),
        true,
    );
    stage.set_stroke(rectangle, Some(color), Some(1.5), Some(1.0), None, true);

    let config = frame_config();
    let plan = {
        let _span = profile.span(
            clock,
            profile_path,
            ProfilePhase::RenderIrSync,
            fmn_runtime::ProfileLane::prepare(),
        );
        let mut plan = RenderPlan::new();
        plan.sync(&stage, spec.sequence)
            .expect("valid frame-pipeline fixture");
        plan
    };
    let mono = {
        let _span = profile.span(
            clock,
            profile_path,
            ProfilePhase::GeometryCompile,
            fmn_runtime::ProfileLane::prepare(),
        );
        MonoTable::build(&plan, config.map).expect("bounded frame-pipeline monotone table")
    };
    let binning = {
        let _span = profile.span(
            clock,
            profile_path,
            ProfilePhase::Binning,
            fmn_runtime::ProfileLane::prepare(),
        );
        let mut binning = Binning::build(&plan, config.viewport, tiling, config.map)
            .expect("bounded conformance binning");
        binning
            .prune_occluded(&plan)
            .expect("binning belongs to this render plan");
        binning
    };
    PreparedFrame {
        plan,
        mono,
        binning,
        config,
    }
}

fn scalar_definition(spec: FrameSpec, tiling: Tiling) -> Vec<u8> {
    let prepared = prepare(spec, tiling);
    let job = FrameJob::new(
        &prepared.plan,
        &prepared.mono,
        &prepared.binning,
        prepared.config,
    )
    .expect("matching frame artifacts");
    let frame = job.render(1).expect("scalar engine renders");
    encode_frame(&frame).expect("canonical frame encoding")
}

fn plan(depth: usize) -> ExecutionPlan {
    // Four workers per frame leaves six independent teams on this synthetic
    // host. The queue-depth sweep therefore covers both stage overlap and real
    // whole-frame concurrency without spawning 96 workers per tiny fixture.
    let topology = HardwareTopology::fallback(24);
    let fingerprint = TopologyFingerprint::of(&topology);
    let mut cache = AutotuneCache::default();
    cache
        .insert(AutotuneProfile {
            topology: fingerprint,
            threads_per_render_team: 4,
            frames_in_flight: 6,
            fine_tile: 16,
            macro_tile: 128,
            scratch_bytes_per_worker: 64 * 1024,
        })
        .expect("valid deterministic fixture profile");
    ExecutionPlan::derive(
        PlanRequest::standard(
            RenderIntent::Offline,
            SurfaceSpec::lumen(WIDTH, HEIGHT),
            OutputPixelFormat::Rgba16F,
        )
        .with_max_frames_in_flight(depth),
        &topology,
        Some(&cache),
    )
    .expect("synthetic topology yields a plan")
}

#[test]
fn lumen_bytes_are_queue_depth_and_team_schedule_independent() {
    let tiling = Tiling {
        macro_tile: 128,
        fine_tile: 16,
    };
    let specs: Vec<FrameSpec> = (0..FRAME_COUNT)
        .map(|sequence| FrameSpec { sequence })
        .collect();
    let expected: Vec<(u64, Vec<u8>)> = specs
        .iter()
        .copied()
        .map(|spec| (spec.sequence, scalar_definition(spec, tiling)))
        .collect();
    for depth in 1..=6 {
        let plan = plan(depth);
        assert_eq!(plan.frames_in_flight, depth);
        assert_eq!(plan.render_teams.len(), depth);

        let events = specs
            .iter()
            .copied()
            .map(|spec| PipelineEvent::<_, ()>::frame(spec.sequence, spec));
        let mut actual = Vec::new();
        let profile = ProfileRecorder::enabled();
        let clock: Arc<dyn Clock> = Arc::new(StdClock::new());
        let profile_path = ProfilePath::scene(1).with_play(depth as u64);
        let stages = LumenStages {
            tiling,
            profile: profile.clone(),
            clock: Arc::clone(&clock),
            profile_path,
        };
        let stats = FramePipeline::with_clock_and_profile(
            &plan,
            &stages,
            CancellationToken::new(),
            clock,
            profile.clone(),
        )
        .with_profile_path(profile_path)
        .run(
            events,
            |sequence, bytes| {
                actual.push((sequence, bytes));
                Ok(())
            },
            |(), _| Ok(()),
        )
        .expect("real Lumen pipeline completes");

        assert_eq!(actual, expected, "queue depth {depth}");
        assert_eq!(stats.submitted, FRAME_COUNT);
        assert_eq!(stats.emitted, FRAME_COUNT);
        assert_eq!(stats.outstanding_slots, 0);
        assert!(stats.max_in_flight <= depth);
        assert_eq!(stats.render_team_frames.iter().sum::<u64>(), FRAME_COUNT);

        let phases = profile
            .snapshot()
            .records()
            .iter()
            .filter_map(|record| match record {
                ProfileRecord::Span(span) => Some(span.phase),
                ProfileRecord::Counter(_) => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            phases,
            BTreeSet::from([
                ProfilePhase::SceneUpdate,
                ProfilePhase::GeometryCompile,
                ProfilePhase::RenderIrSync,
                ProfilePhase::Binning,
                ProfilePhase::Prepare,
                ProfilePhase::Raster,
                ProfilePhase::ColorConversion,
                ProfilePhase::Emit,
            ]),
            "the real Lumen path emits every coarse pipeline phase"
        );
    }
}
