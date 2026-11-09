//! fm-fjq seed scenario catalog: the Gauntlet's e2e scenarios **as data**
//! over the [`fmn_conformance::e2e`] harness API.
//!
//! Every scenario is a checked-in, compile-checked [`ScenarioSpec`]: the
//! invocation drives a **real surface** (the Rust API today; the CLI
//! in-process runner for the failure drill), the assertions ride the D-16
//! self-golden rig (certified rows) or structural predicates (standard
//! rows), and the log expectations assert on the structured event stream
//! the harness captures into each run's deterministic NDJSON artifact.
//!
//! ## Seeded classes (fm-fjq)
//!
//! - **RENDER-MATRIX** — a Circle+Tex-label scene through the real native
//!   sinks × {y4m, PNG-sequence} × the CLI's quality presets
//!   (`-l` 854×480 in the fast tier; `-m`/`--hd`/`--uhd` in the
//!   `FMN_E2E_FULL=1` tier, one frame per row for the nightly budget) ×
//!   {certified, standard} engine identities. Certified rows bit-lock every
//!   emitted artifact in the `e2e` suite (`Scope::Certified`); standard
//!   rows assert structure (frame count, geometry, exit code, emitted-file
//!   inventory). The CLI rows also cover ordered two-scene batch publication
//!   through the real Asupersync-backed composition root.
//! - **DETERMINISM DRILLS** — the same render twice in a row is
//!   byte-identical; the certified terminal frame is byte-identical at
//!   {1, 4} threads (fast tier) and at 16 threads (full tier) — PG-5's
//!   thread-invariance form pointed at a real scene render.
//! - **FAILURE-PATH DRILLS** — an unsupported TeX construct surfaces the
//!   precise named `TexError::Math` refusal; a corrupted cache entry is
//!   evicted and recomputed, never fatal; an invalid CLI flag combination
//!   (`-l -m`) exits 2 with the `quality-exclusive` rule identity through
//!   `fmn_cli`'s in-process runner; certified Metal selection exits 3 before
//!   creating its requested output path.
//! - **LIFECYCLE DRILL** — construct → snapshot → transform → snapshot
//!   with geometry assertions (the `scene_goldens` lifecycle form), plus
//!   a journal round-trip: bytes out, bytes in, identical content hash,
//!   and `plan_replay` reusing exactly the non-barrier prefix; PG-5 binds its
//!   checked-in definition to a representative `{1, 4, 16}` certified render,
//!   while PG-6 warms and reuses the real frame arena/output buffer for a
//!   registered corpus scene and requires an equal digest with zero measured
//!   allocations. PG-6's full tier also renders the exact three-view UHD 3D
//!   gallery through the production solid/surface and renderer APIs and checks
//!   every frame against its certified lock.
//! - **LOG ASSERTIONS** — the preflight typeset fires before frame zero
//!   with the typeset count recorded; the segment purity classification
//!   is recorded for play (pure) and wait (stateful); the engine identity
//!   and the `ExecutionPlan` tuning provenance are recorded on
//!   reproducible-class runs.
//!
//! ## Doctrine
//!
//! Each feature bead registers at least one e2e scenario when it lands
//! user-visible behavior; the fast tier runs per-commit and the full
//! matrix is env-gated (`FMN_E2E_FULL=1`) for the nightly budget. The
//! The Python standard PNG surface is live through its feature-gated embedded
//! CPython adapter. Studio is live through the production CLI composition root;
//! both have registered frame scenarios, while unsupported Python modes remain
//! precise capability refusals;
//! Studio also has a mandatory native subprocess lifecycle row over Cargo's
//! exact shipping CLI artifact, authenticated loopback HTTP and multipart PNG.
//! Every seeded class carries one deliberately-injected
//! regression drill (`RegressionKind`): the runner drives the scenario
//! red, and the repro bundle plus log artifact must appear.

#[allow(dead_code)]
#[path = "support/semantic_routes.rs"]
mod semantic_routes;

// fm-longform-native-e2e-hoeffding-l59d: the Hoeffding's D explainer
// (crates/fmn/examples/hoeffding_d), compiled here from its own sources so
// both tiers render the real chapters through `fmn::render`. The chapters
// name their siblings as `crate::kit` etc., hence the crate-root modules.
#[allow(dead_code)]
#[path = "../../fmn/examples/hoeffding_d/chapters/mod.rs"]
mod chapters;
#[allow(dead_code)]
#[path = "../../fmn/examples/hoeffding_d/kit.rs"]
mod kit;
#[allow(dead_code)]
#[path = "../../fmn/examples/hoeffding_d/narration.rs"]
mod narration;
#[allow(dead_code)]
#[path = "../../fmn/examples/hoeffding_d/sound.rs"]
mod sound;
#[allow(dead_code)]
#[path = "../../fmn/examples/hoeffding_d/stats.rs"]
mod stats;

use fmn_anim::animation::Animation as _;
use fmn_anim::{FramePacket, MoveAlongPath, RateFunc, Timeline};
use fmn_conformance::e2e::{
    self, Assertion, FieldPred, Invocation, LogEvent, LogExpect, RegressionKind, RunCtx,
    RunOutcome, Runner, ScenarioClass, ScenarioError, ScenarioSpec, Status, Surface, Tier,
};
use fmn_conformance::golden::Scope;
use fmn_conformance::perf::{BenchmarkKey, GateId, MetricUnit};
use fmn_conformance::perf_host::measurement_identity;
use fmn_conformance::perf_pg5::{PG5_DIRECT_THREADS, Pg5Definition, pg5_identity};
use fmn_conformance::perf_pg6::{PG6_THREADS, Pg6Definition, pg6_identity};
use fmn_conformance::perf_pg6_peak::{
    PG6_PEAK_CASE_COUNT, PG6_PEAK_HEIGHT, PG6_PEAK_WIDTH, Pg6PeakDefinition,
    render_locked_gallery_once,
};
use fmn_conformance::scene_goldens::{self, TILING};
use fmn_core::color::Srgb;
use fmn_core::constants::{
    BLUE_C, BOTTOM, DOWN, FRAME_X_RADIUS, FRAME_Y_RADIUS, LEFT, LEFT_SIDE, RED_C, RIGHT,
    RIGHT_SIDE, TEAL_B, TOP, UL, UP, WHITE, YELLOW,
};
use fmn_core::rng::RngRoot;
use fmn_frame::convert::{rgba_to_nv12, rgba16f_to_rgba8};
use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameLayout, PixelFormat};
use fmn_hash::sha256;
use fmn_library::style::VStyle;
use fmn_library::vmobject::VMobject;
use fmn_library::{Circle, FunctionGraph, NumberPlane};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};
use fmn_output::sinks::{
    NativeArtifactKind, NativeArtifactReport, PngSink, PngSinkConfig, PngTarget, SinkLimits,
    Y4mSink, Y4mSinkConfig,
};
use fmn_output::{ClosureItem, FrameSink, ManifestMode, ProvenanceManifest, SinkWrite};
use fmn_platform::fs::FileSystem;
use fmn_platform::topology::HardwareTopology;
use fmn_render::FrameArena;
use fmn_render::bin::{Binning, ScreenMap, Viewport};
use fmn_render::engine::{
    EngineIdentity, EngineKind, FrameConfig, FrameJob, encode_frame, frame_digest,
};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use fmn_runtime::{
    ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec, TuningSource,
};
use fmn_scene::journal::{CommandKind, CommandRecord, EffectClass, Entry, Journal, plan_replay};
use fmn_scene::{
    CaptureReason, IntegrationError, PlayOverrides, RuntimeConfig, Scene, SceneError, SceneProgram,
    SceneSink,
};
use fmn_tex::{Prim, TexError};
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Shared constants
// ---------------------------------------------------------------------------

/// Scene frame rate for every e2e render: small, exact, and identical
/// across the matrix so frame counts are a function of the scenario, not
/// the preset.
const FPS: u32 = 8;

/// The Reference's frame height in scene units (`sizes.frame_height`).
const FRAME_HEIGHT_UNITS: f64 = 8.0;

/// The e2e golden suite: one lock shared by the certified matrix.
const E2E_SUITE: &str = "e2e";

/// The full-tier opt-in environment gate (nightly budget, R22).
const FULL_TIER_ENV: &str = "FMN_E2E_FULL";

/// One CLI quality preset as geometry: the flag name, its default-config
/// resolution (`fmn-config`'s `resolution_options`), and the tier the row
/// runs in. Scale follows the Reference rule `height / frame_height`.
#[derive(Clone, Copy)]
struct Preset {
    flag: &'static str,
    width: u32,
    height: u32,
    tier: Tier,
}

const PRESETS: &[Preset] = &[
    Preset {
        flag: "low",
        width: 854,
        height: 480,
        tier: Tier::Fast,
    },
    Preset {
        flag: "medium",
        width: 1280,
        height: 720,
        tier: Tier::Full,
    },
    Preset {
        flag: "hd",
        width: 1920,
        height: 1080,
        tier: Tier::Full,
    },
    Preset {
        flag: "uhd",
        width: 3840,
        height: 2160,
        tier: Tier::Full,
    },
];

/// The native sinks the matrix covers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SinkKind {
    Y4m,
    PngSequence,
}

/// Process-unique suffix for on-disk sink destinations: PNG sequence
/// directories are claimed no-clobber, so rerunning a scenario in one
/// process must never collide with its own previous claim.
static RUN_SUFFIX: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Small stable-name mappers (event field values are the harness's contract)
// ---------------------------------------------------------------------------

fn engine_kind_name(kind: EngineKind) -> &'static str {
    match kind {
        EngineKind::CertifiedCpu => "certified_cpu",
        EngineKind::FastCpu => "fast_cpu",
        EngineKind::Metal => "metal",
        EngineKind::Cuda => "cuda",
    }
}

fn tuning_source_name(source: TuningSource) -> &'static str {
    match source {
        TuningSource::CertifiedProfile => "certified_profile",
        TuningSource::StandardBaseline => "standard_baseline",
        TuningSource::StandardAutotuneCache => "standard_autotune_cache",
    }
}

fn fail(message: impl Into<String>) -> ScenarioError {
    ScenarioError::new(message.into())
}

/// [`Assertion::Structural`] `CounterEq` as data.
fn counter_eq(name: &'static str, value: u64) -> Assertion {
    Assertion::Structural(e2e::StructuralAssert::CounterEq(name, value))
}

/// [`Assertion::Structural`] `CounterGe` as data.
fn counter_ge(name: &'static str, bound: u64) -> Assertion {
    Assertion::Structural(e2e::StructuralAssert::CounterGe(name, bound))
}

/// Event-field spelling for booleans, so `FieldPred::StrEq` assertions
/// compare against the recorded string form with no serialization guesswork.
const fn truth(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

// ---------------------------------------------------------------------------
// The e2e scene: Circle + Tex label, one animated shift, one wait
// ---------------------------------------------------------------------------

/// Build the Tex label mobject through the corpus's bundled engine.
fn tex_label(source: &'static str, font_size: f64) -> Result<VMobject, ScenarioError> {
    fmn_library::Tex::new(source)
        .font_size(font_size)
        .build(&scene_goldens::corpus().tex)
        .map(|built| built.vmob)
        .map_err(|error| fail(format!("tex label {source:?} typesets: {error}")))
}

/// The render-matrix / determinism scene program. Construction is a pure
/// function of the pre-built label and the scene seed; one play (pure
/// shift, 0.25 s ⇒ 2 frames at 8 fps) and one wait (0.125 s ⇒ 1 frame)
/// give a three-frame sequence with motion.
struct MatrixProgram {
    label: Option<VMobject>,
    wait_only: bool,
}

impl SceneProgram for MatrixProgram {
    fn name(&self) -> &str {
        "e2e_render_matrix"
    }

    fn construct(&mut self, scene: &mut Scene, sink: &mut dyn SceneSink) -> Result<(), SceneError> {
        let circle = scene.add_mobject(Circle::new().radius(0.9))?;
        scene
            .stage_mut()
            .set_fill(circle, Some(BLUE_C), Some(0.35), Some(0.0), true);
        scene
            .stage_mut()
            .set_stroke(circle, Some(WHITE), Some(2.0), Some(0.95), None, true);

        let label = self.label.take().ok_or(SceneError::InvalidLifecycle(
            "matrix program constructed twice",
        ))?;
        let label = scene.add_mobject(label)?;
        scene.stage_mut().shift(label, [0.0, -0.15, 0.0]);

        if !self.wait_only {
            let builder = circle
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.25),
                    rate_func: Some(fmn_core::rate::linear),
                    ..AnimateArgs::default()
                })
                .map_err(|error| SceneError::Animation(error.into()))?
                .shift([0.6, 0.2, 0.0])
                .map_err(|error| SceneError::Animation(error.into()))?;
            let animation = fmn_anim::prepare_animation(builder, scene.stage_mut())?;
            scene.play(vec![animation], PlayOverrides::default(), sink)?;
        }
        scene.wait(Some(0.125), sink)?;
        Ok(())
    }
}

/// A sink that freezes each captured packet for the render stage.
#[derive(Default)]
struct PacketSink {
    packets: Vec<FramePacket>,
}

impl SceneSink for PacketSink {
    fn capture(
        &mut self,
        _reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        self.packets.push(packet);
        Ok(())
    }
}

/// Run the matrix scene and return its frozen packets.
fn run_matrix_scene(seed: u64, wait_only: bool) -> Result<Vec<FramePacket>, ScenarioError> {
    let label = tex_label(r"\odot", 40.0)?;
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: FPS,
            ..RuntimeConfig::default()
        },
        seed,
    )
    .map_err(|error| fail(format!("scene constructs: {error}")))?;
    let mut program = MatrixProgram {
        label: Some(label),
        wait_only,
    };
    let mut sink = PacketSink::default();
    scene
        .run(&mut program, &mut sink)
        .map_err(|error| fail(format!("scene runs: {error}")))?;
    Ok(sink.packets)
}

/// The frame configuration for one preset: the Reference's `#333333`
/// background, origin at frame center, scale `height / 8`.
fn preset_frame_config(preset: &Preset) -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: preset.width,
            height: preset.height,
        },
        ScreenMap {
            scale: f64::from(preset.height) / FRAME_HEIGHT_UNITS,
            origin: [
                f64::from(preset.width) / 2.0,
                f64::from(preset.height) / 2.0,
            ],
            y_up: true,
        },
        Srgb::from_rgb8(0x33, 0x33, 0x33).to_linear(1.0),
    )
}

/// Render one frozen packet through an explicit engine identity.
fn render_packet(
    packet: &FramePacket,
    identity: EngineIdentity,
    config: FrameConfig,
    threads: usize,
) -> Result<FrameBuffer, ScenarioError> {
    let stage = packet.materialize_stage();
    let mut plan = RenderPlan::new();
    let revision = u64::try_from(packet.frame_index())
        .map_err(|_| fail("negative frame index reached the renderer"))?;
    plan.sync(&stage, revision)
        .map_err(|error| fail(format!("render plan accepts packet geometry: {error}")))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| fail(format!("monotone table accepts packet geometry: {error}")))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .map_err(|error| fail(format!("occlusion pruning matches the plan: {error}")))?;
    FrameJob::with_identity(&plan, &mono, &binning, config, identity)
        .map_err(|error| fail(format!("frame job matches the plan: {error}")))?
        .render(threads)
        .map_err(|error| fail(format!("engine renders the frame: {error}")))
}

/// Convert a certified linear-light frame into the sink's negotiated
/// pixel format.
fn convert_for_sink(
    frame: &FrameBuffer,
    sink: SinkKind,
    width: u32,
    height: u32,
) -> Result<FrameBuffer, ScenarioError> {
    let mut rgba8 = FrameBuffer::new(
        FrameLayout::tight(PixelFormat::Rgba8, width, height)
            .map_err(|error| fail(format!("rgba8 layout: {error}")))?,
    );
    rgba16f_to_rgba8(frame, &mut rgba8)
        .map_err(|error| fail(format!("rgba16f → rgba8: {error}")))?;
    match sink {
        SinkKind::PngSequence => Ok(rgba8),
        SinkKind::Y4m => {
            let mut nv12 = FrameBuffer::new(
                FrameLayout::tight(PixelFormat::Nv12, width, height)
                    .map_err(|error| fail(format!("nv12 layout: {error}")))?,
            );
            // The ordinary video form: BT.709 limited range, left-sited
            // chroma — matching the C420mpeg2 container tag.
            rgba_to_nv12(&rgba8, &mut nv12, ColorRange::Limited, ChromaSiting::Left)
                .map_err(|error| fail(format!("rgba8 → nv12: {error}")))?;
            Ok(nv12)
        }
    }
}

/// The scenario-private output directory under the target tmpdir.
fn scenario_dir(scenario: &str) -> Result<PathBuf, ScenarioError> {
    let base =
        std::env::var_os("CARGO_TARGET_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let suffix = RUN_SUFFIX.fetch_add(1, Ordering::Relaxed);
    let dir = base
        .join("fmn_e2e")
        .join(format!("{scenario}.{}.{}", std::process::id(), suffix));
    std::fs::create_dir_all(&dir)
        .map_err(|error| fail(format!("create {}: {error}", dir.display())))?;
    Ok(dir)
}

fn sink_limits(frames: u64) -> Result<SinkLimits, ScenarioError> {
    SinkLimits::new(16, 1 << 30, 1 << 34, 1 << 34)
        .and_then(|limits| limits.requiring_exact_frames(frames))
        .map_err(|error| fail(format!("sink limits: {error}")))
}

/// fm-hyqz: the PNG-sequence sink encodes frames concurrently. Twelve
/// certified frames publish byte-identical at 1 and 4 encoder workers (the
/// same members and tree digest), and the run logs the sink's frame
/// concurrency, its in-flight byte bound, and the encoder lanes the profile
/// saw. Lane counts depend on scheduling, so they are logged, not asserted.
fn png_sequence_frame_concurrency_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    const PRESET: Preset = Preset {
        flag: "png-concurrency",
        width: 160,
        height: 90,
        tier: Tier::Fast,
    };
    const FRAMES: u32 = 12;
    let frames = (0..FRAMES)
        .map(|index| {
            let layout = FrameLayout::tight(PixelFormat::Rgba8, PRESET.width, PRESET.height)
                .map_err(|error| fail(format!("frame layout: {error}")))?;
            let mut frame = FrameBuffer::new(layout);
            for (pixel, rgba) in frame
                .as_bytes_mut()
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .enumerate()
            {
                let pixel = u32::try_from(pixel).unwrap_or(u32::MAX);
                let (x, y) = (pixel % PRESET.width, pixel / PRESET.width);
                let block = x.abs_diff(10 * index + 20) < 12 && y.abs_diff(45) < 12;
                *rgba = if block {
                    [255, 200, 40, 255]
                } else {
                    let ramp =
                        |value: u32, span: u32| u8::try_from(value * 255 / span).unwrap_or(255);
                    [ramp(x, PRESET.width), ramp(y, PRESET.height), 51, 255]
                };
            }
            Ok(frame)
        })
        .collect::<Result<Vec<_>, ScenarioError>>()?;
    let serial_dir = scenario_dir("png_sequence_concurrency_serial")?;
    let (serial, serial_report) =
        publish_png_sequence(&serial_dir, &PRESET, &frames, true, 1, None)?;
    let recorder = fmn_platform::profile::ProfileRecorder::enabled();
    let profile = fmn_output::OutputProfile::new(
        Arc::new(fmn_platform::clock::StdClock::new()),
        recorder.clone(),
        fmn_platform::profile::ProfilePath::scene(0),
    );
    let pooled_dir = scenario_dir("png_sequence_concurrency_pooled")?;
    let (pooled, pooled_report) =
        publish_png_sequence(&pooled_dir, &PRESET, &frames, true, 4, Some(profile))?;
    let probe = PngSinkConfig {
        target: PngTarget::Sequence {
            directory: pooled_dir.join("probe"),
            stem: "frame".to_string(),
            digits: 4,
        },
        width: PRESET.width,
        height: PRESET.height,
        first_sequence: 0,
        compression: fmn_codec::CompressionLevel::Best,
        threads: 4,
        limits: sink_limits(u64::from(FRAMES))?,
        profile: None,
    };
    let concurrency = probe.sequence_frame_concurrency() as u64;
    let frame_bytes = u64::from(PRESET.width) * u64::from(PRESET.height) * 4;
    let ndjson = recorder.snapshot().to_ndjson();
    let encode_spans = ndjson
        .lines()
        .filter(|line| line.contains("\"phase\":\"encode\""))
        .count() as u64;
    let lanes = ndjson
        .lines()
        .filter(|line| line.contains("\"lane_role\":\"encoder\""))
        .filter_map(|line| line.split("\"lane_index\":").nth(1))
        .filter_map(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        .collect::<std::collections::BTreeSet<_>>()
        .len() as u64;
    let identical = serial == pooled && serial_report.digest == pooled_report.digest;
    ctx.event(
        LogEvent::new("e2e.png.frame_concurrency")
            .field("frames", u64::from(FRAMES))
            .field("threads", 4_u64)
            .field("concurrency", concurrency)
            .field("in_flight_bound_bytes", concurrency * 2 * frame_bytes)
            .field("encode_spans", encode_spans)
            .field("encoder_lanes_observed", lanes)
            .field("identical", truth(identical)),
    );
    if !identical || concurrency < 2 || encode_spans != u64::from(FRAMES) {
        return Err(fail(format!(
            "frame-concurrent PNG sequence: identical={identical} concurrency={concurrency} \
             encode_spans={encode_spans} lanes={lanes} serial={} pooled={}",
            serial.len(),
            pooled.len()
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("png_concurrency_identical", 1)
        .with_counter("png_encode_spans", encode_spans))
}

/// Drive the real native y4m sink over the rendered frames and read the
/// published artifact back from disk.
fn publish_y4m(
    dir: &std::path::Path,
    preset: &Preset,
    frames: &[FrameBuffer],
) -> Result<(String, Vec<u8>, NativeArtifactReport), ScenarioError> {
    let destination = dir.join("out.y4m");
    let mut sink = Y4mSink::new(
        Arc::new(fmn_platform::fs::StdFs),
        Y4mSinkConfig {
            destination: destination.clone(),
            width: preset.width,
            height: preset.height,
            fps: (FPS, 1),
            colorspace: fmn_codec::Y4mColorspace::C420Mpeg2,
            first_sequence: 0,
            limits: sink_limits(frames.len() as u64)?,
            profile: None,
        },
    )
    .map_err(|error| fail(format!("y4m sink constructs: {error}")))?;
    let receipt = sink.receipt();
    for (index, frame) in frames.iter().enumerate() {
        match sink.write_frame(index as u64, frame) {
            Ok(SinkWrite::Consumed) => {}
            Ok(SinkWrite::WouldBlock) => {
                return Err(fail("reliable y4m sink reported WouldBlock"));
            }
            Err(error) => return Err(fail(format!("y4m sink write: {error}"))),
        }
    }
    sink.finish()
        .map_err(|error| fail(format!("y4m sink finish: {error}")))?;
    let report = receipt
        .take()
        .map_err(|error| fail(format!("y4m receipt: {error}")))?;
    let bytes = std::fs::read(&destination)
        .map_err(|error| fail(format!("read {}: {error}", destination.display())))?;
    Ok(("out.y4m".to_string(), bytes, report))
}

/// The published PNG-sequence payload: `(file name, bytes)` pairs in
/// sequence order, plus the sink's completion report.
type PublishedSequence = (Vec<(String, Vec<u8>)>, NativeArtifactReport);

/// Drive the real canonical PNG-sequence sink with `threads` encoder workers
/// and read every published frame back from disk (file order = sequence
/// order).
fn publish_png_sequence(
    dir: &std::path::Path,
    preset: &Preset,
    frames: &[FrameBuffer],
    certified: bool,
    threads: usize,
    profile: Option<fmn_output::OutputProfile>,
) -> Result<PublishedSequence, ScenarioError> {
    let directory = dir.join("frames");
    let mut sink = PngSink::new(
        Arc::new(fmn_platform::fs::StdFs),
        PngSinkConfig {
            target: PngTarget::Sequence {
                directory: directory.clone(),
                stem: "frame".to_string(),
                digits: 4,
            },
            width: preset.width,
            height: preset.height,
            first_sequence: 0,
            compression: if certified {
                fmn_codec::CompressionLevel::Best
            } else {
                fmn_codec::CompressionLevel::Default
            },
            threads,
            limits: sink_limits(frames.len() as u64)?,
            profile,
        },
    )
    .map_err(|error| fail(format!("png sink constructs: {error}")))?;
    let receipt = sink.receipt();
    for (index, frame) in frames.iter().enumerate() {
        match sink.write_frame(index as u64, frame) {
            Ok(SinkWrite::Consumed) => {}
            Ok(SinkWrite::WouldBlock) => {
                return Err(fail("reliable png sink reported WouldBlock"));
            }
            Err(error) => return Err(fail(format!("png sink write: {error}"))),
        }
    }
    sink.finish()
        .map_err(|error| fail(format!("png sink finish: {error}")))?;
    let report = receipt
        .take()
        .map_err(|error| fail(format!("png receipt: {error}")))?;
    let mut files: Vec<PathBuf> = std::fs::read_dir(&directory)
        .map_err(|error| fail(format!("list {}: {error}", directory.display())))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()
        .map_err(|error| fail(format!("read dir entry: {error}")))?;
    files.sort();
    let mut published = Vec::new();
    for file in files {
        if file.extension().is_some_and(|ext| ext == "png") {
            let name = file
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .ok_or_else(|| fail("png sequence child has no file name"))?;
            let bytes = std::fs::read(&file)
                .map_err(|error| fail(format!("read {}: {error}", file.display())))?;
            published.push((name, bytes));
        }
    }
    Ok((published, report))
}

// ---------------------------------------------------------------------------
// Render-matrix invocation
// ---------------------------------------------------------------------------

/// One matrix row: run the scene, render every packet under the row's
/// engine identity, convert, publish through the real native sink, and
/// return the emitted files as artifacts plus the structural counters and
/// the machinery events (engine identity, plan decision, per-frame render,
/// per-file emission).
fn render_matrix_run(
    ctx: &mut RunCtx,
    preset: &Preset,
    sink_kind: SinkKind,
    certified: bool,
    stem: &str,
) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let identity = if certified {
        EngineIdentity::certified()
    } else {
        EngineIdentity::fast()
    };
    let threads = if certified { 1 } else { 4 };
    let config = preset_frame_config(preset);

    // The ExecutionPlan decision for this row, derived over a fixed
    // synthetic topology so the recorded decision is a function of the
    // request, never of the host running the test.
    let topology = HardwareTopology::fallback(8);
    let request = if certified {
        PlanRequest::certified(
            RenderIntent::Offline,
            SurfaceSpec::lumen(preset.width, preset.height),
            OutputPixelFormat::Rgba8,
        )
    } else {
        PlanRequest::standard(
            RenderIntent::Offline,
            SurfaceSpec::lumen(preset.width, preset.height),
            OutputPixelFormat::Rgba8,
        )
    };
    let plan = ExecutionPlan::derive(request, &topology, None)
        .map_err(|error| fail(format!("execution plan derives: {error}")))?;
    ctx.event(
        LogEvent::new(e2e::spans::EXECUTION_PLAN)
            .field(
                "determinism",
                if certified { "certified" } else { "standard" },
            )
            .field("engine", engine_kind_name(identity.engine))
            .field("tuning_source", tuning_source_name(plan.tuning_source))
            .field("fine_tile", u64::from(plan.fine_tile))
            .field("macro_tile", u64::from(plan.macro_tile)),
    );
    ctx.event(
        LogEvent::new(e2e::spans::ENGINE)
            .field("engine", engine_kind_name(identity.engine))
            .field("reproducible", if certified { "true" } else { "false" })
            .field("renderer_version", u64::from(identity.renderer_version)),
    );

    let packets = run_matrix_scene(ctx.seed, preset.tier == Tier::Full)?;
    ctx.event(
        LogEvent::new(e2e::spans::SCENE_CONSTRUCT)
            .field("scene", "e2e_render_matrix")
            .field("seed", ctx.seed)
            .field("frames", packets.len() as u64),
    );

    let mut converted = Vec::with_capacity(packets.len());
    for (index, packet) in packets.iter().enumerate() {
        let frame = render_packet(packet, identity, config, threads)?;
        converted.push(convert_for_sink(
            &frame,
            sink_kind,
            preset.width,
            preset.height,
        )?);
        ctx.event(
            LogEvent::new(e2e::spans::RENDER_FRAME)
                .field("frame", index as u64)
                .field("engine", engine_kind_name(identity.engine)),
        );
    }

    let dir = scenario_dir("render_matrix")?;
    let mut outcome = RunOutcome::ok();
    let mut total_bytes = 0_u64;
    match sink_kind {
        SinkKind::Y4m => {
            let (name, bytes, report) = publish_y4m(&dir, preset, &converted)?;
            if report.kind != NativeArtifactKind::Y4m {
                return Err(fail("y4m receipt reports the wrong artifact kind"));
            }
            total_bytes += bytes.len() as u64;
            ctx.event(
                LogEvent::new(e2e::spans::EMIT)
                    .field("file", name.as_str())
                    .field("bytes", bytes.len() as u64),
            );
            outcome = outcome.with_artifact(stem, bytes);
        }
        SinkKind::PngSequence => {
            let (published, report) =
                publish_png_sequence(&dir, preset, &converted, certified, 1, None)?;
            if report.kind != NativeArtifactKind::PngSequence {
                return Err(fail("png receipt reports the wrong artifact kind"));
            }
            for (index, (file, bytes)) in published.into_iter().enumerate() {
                total_bytes += bytes.len() as u64;
                ctx.event(
                    LogEvent::new(e2e::spans::EMIT)
                        .field("file", file.as_str())
                        .field("bytes", bytes.len() as u64),
                );
                let artifact = format!("{stem}.{index:04}");
                outcome = outcome.with_artifact(&artifact, bytes);
            }
        }
    }

    let frames = converted.len() as u64;
    ctx.counter(e2e::counters::FRAMES_RASTERIZED, frames);
    ctx.counter(e2e::counters::FRAMES_CONVERTED, frames);
    ctx.counter(e2e::counters::FRAMES_EMITTED, frames);
    outcome = outcome
        .with_counter("frames", frames)
        .with_counter("width", u64::from(preset.width))
        .with_counter("height", u64::from(preset.height))
        .with_counter("emitted_bytes", total_bytes);
    Ok(outcome)
}

fn matrix_spec(
    preset: &Preset,
    sink_kind: SinkKind,
    certified: bool,
    drill_name: Option<&'static str>,
) -> ScenarioSpec {
    let base: &'static str = match (preset.flag, sink_kind, certified) {
        ("low", SinkKind::Y4m, true) => "render_matrix.low.y4m.certified.v1",
        ("low", SinkKind::Y4m, false) => "render_matrix.low.y4m.standard.v1",
        ("low", SinkKind::PngSequence, true) => "render_matrix.low.png_seq.certified.v1",
        ("low", SinkKind::PngSequence, false) => "render_matrix.low.png_seq.standard.v1",
        ("medium", SinkKind::Y4m, true) => "render_matrix.medium.y4m.certified.v1",
        ("medium", SinkKind::Y4m, false) => "render_matrix.medium.y4m.standard.v1",
        ("medium", SinkKind::PngSequence, true) => "render_matrix.medium.png_seq.certified.v1",
        ("medium", SinkKind::PngSequence, false) => "render_matrix.medium.png_seq.standard.v1",
        ("hd", SinkKind::Y4m, true) => "render_matrix.hd.y4m.certified.v1",
        ("hd", SinkKind::Y4m, false) => "render_matrix.hd.y4m.standard.v1",
        ("hd", SinkKind::PngSequence, true) => "render_matrix.hd.png_seq.certified.v1",
        ("hd", SinkKind::PngSequence, false) => "render_matrix.hd.png_seq.standard.v1",
        ("uhd", SinkKind::Y4m, true) => "render_matrix.uhd.y4m.certified.v1",
        ("uhd", SinkKind::Y4m, false) => "render_matrix.uhd.y4m.standard.v1",
        ("uhd", SinkKind::PngSequence, true) => "render_matrix.uhd.png_seq.certified.v1",
        ("uhd", SinkKind::PngSequence, false) => "render_matrix.uhd.png_seq.standard.v1",
        _ => "render_matrix.unknown.v1",
    };
    // The drill row gets its own scenario name (its log and bundle files)
    // but reuses the base row's artifact stem: the harness evaluates a
    // golden drill in forced check mode against the base row's blessed
    // lock entries, so the injected flip lands as self-golden drift —
    // never as a blessable new entry.
    let name: &'static str = drill_name.unwrap_or(base);
    let stem: &'static str = base;

    let expected_frames: u64 = match preset.tier {
        Tier::Fast => 3,
        Tier::Full => 1,
    };
    let preset_copy = *preset;
    let invocation = Invocation::new(move |ctx: &mut RunCtx| {
        render_matrix_run(ctx, &preset_copy, sink_kind, certified, stem)
    });

    let mut assertions = vec![
        Assertion::ExitCode(0),
        counter_eq("frames", expected_frames),
        counter_eq("width", u64::from(preset.width)),
        counter_eq("height", u64::from(preset.height)),
    ];
    if certified {
        // Certified rows bit-lock every emitted artifact against the
        // shared certified-matrix lock (D-16).
        assertions.push(Assertion::GoldenLock {
            suite: E2E_SUITE,
            scope: Scope::Certified,
        });
    } else {
        // Standard rows assert the emitted-file inventory instead: the
        // fast engine's bits may legitimately differ across SIMD tiers.
        let inventory = match sink_kind {
            SinkKind::Y4m => vec![stem.to_string()],
            SinkKind::PngSequence => (0..expected_frames)
                .map(|index| format!("{stem}.{index:04}"))
                .collect(),
        };
        assertions.push(Assertion::FileInventory(inventory));
    }
    // The runner's own NDJSON log contract is under test on every run.
    assertions.push(Assertion::NdjsonSchema);

    ScenarioSpec {
        name,
        class: ScenarioClass::RenderMatrix,
        surface: Surface::RustApi,
        tier: preset.tier,
        invocation,
        assertions,
        logs: vec![
            LogExpect::span_present(
                e2e::spans::ENGINE,
                vec![
                    FieldPred::str_eq("engine", engine_kind_name_str(certified)),
                    FieldPred::str_eq("reproducible", truth(certified)),
                ],
            ),
            LogExpect::span_present(
                e2e::spans::EXECUTION_PLAN,
                vec![FieldPred::str_eq(
                    "tuning_source",
                    if certified {
                        "certified_profile"
                    } else {
                        "standard_baseline"
                    },
                )],
            ),
            LogExpect::event_order(e2e::spans::SCENE_CONSTRUCT, e2e::spans::RENDER_FRAME),
            LogExpect::counter_ge(e2e::counters::FRAMES_EMITTED, expected_frames),
        ],
        regression: None,
    }
}

const fn engine_kind_name_str(certified: bool) -> &'static str {
    if certified {
        "certified_cpu"
    } else {
        "fast_cpu"
    }
}

// ---------------------------------------------------------------------------
// Determinism drills
// ---------------------------------------------------------------------------

/// Twice-in-a-row: the identical scene rendered twice within one run must
/// produce byte-identical artifacts (PG-5's reproducibility form).
fn determinism_repeat_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let preset = PRESETS[0];
    let identity = EngineIdentity::certified();
    let config = preset_frame_config(&preset);

    let mut runs = Vec::with_capacity(2);
    for run in 0..2_u64 {
        let packets = run_matrix_scene(ctx.seed, false)?;
        let mut encoded = Vec::with_capacity(packets.len());
        for packet in &packets {
            let frame = render_packet(packet, identity, config, 1)?;
            encoded.push(
                encode_frame(&frame).map_err(|error| fail(format!("frame encodes: {error}")))?,
            );
        }
        ctx.event(
            LogEvent::new(e2e::spans::RENDER_FRAME)
                .field("run", run)
                .field("frames", encoded.len() as u64),
        );
        runs.push(encoded.concat());
    }
    let equal = runs[0] == runs[1];
    ctx.counter("repeat_byte_equal", u64::from(equal));
    if !equal {
        return Err(fail(
            "twice-in-a-row certified renders diverged byte-for-byte",
        ));
    }
    Ok(RunOutcome::ok()
        .with_artifact("determinism.repeat.v1", runs[0].clone())
        .with_counter("frames", 3)
        .with_counter("repeat_byte_equal", 1))
}

/// Thread-count invariance: the certified terminal frame at {1, 4}
/// threads (and 16 in the full tier) is byte-identical — PG-5's form.
fn determinism_threads_run(
    ctx: &mut RunCtx,
    max_threads: usize,
) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let preset = PRESETS[0];
    let identity = EngineIdentity::certified();
    let config = preset_frame_config(&preset);

    let packets = run_matrix_scene(ctx.seed, false)?;
    let terminal = packets
        .last()
        .ok_or_else(|| fail("scene produced no terminal packet"))?;
    let scalar = render_packet(terminal, identity, config, 1)?;
    let scalar_doc = encode_frame(&scalar).map_err(|error| fail(format!("encode: {error}")))?;
    let mut equal = true;
    for threads in [4, 16]
        .into_iter()
        .filter(|threads| *threads <= max_threads)
    {
        let threaded = render_packet(terminal, identity, config, threads)?;
        let threaded_doc =
            encode_frame(&threaded).map_err(|error| fail(format!("encode: {error}")))?;
        ctx.event(
            LogEvent::new(e2e::spans::RENDER_FRAME)
                .field("threads", threads as u64)
                .field("byte_equal", truth(threaded_doc == scalar_doc)),
        );
        equal &= threaded_doc == scalar_doc;
    }
    ctx.counter("thread_byte_equal", u64::from(equal));
    if !equal {
        return Err(fail(format!(
            "certified terminal frame drifted at up to {max_threads} threads"
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("frames", 3)
        .with_counter("max_threads", max_threads as u64)
        .with_counter("thread_byte_equal", 1))
}

// ---------------------------------------------------------------------------
// Failure-path drills
// ---------------------------------------------------------------------------

/// Unsupported TeX construct: `\dx` is a named, tier-tagged refusal
/// (fmd-math T2; `\substack` was the example until it graduated under
/// fm-j5t) that must surface verbatim through the e2e runner — never a
/// blank render, never a panic.
fn failure_tex_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let engine = &scene_goldens::corpus().tex;
    let source = r"\dx";
    match engine.typeset(fmn_tex::Mode::Math(fmd_math::Style::Text), source) {
        Ok(_) => Err(fail(
            "unsupported construct \\dx typeset successfully — the named refusal is gone",
        )),
        Err(error) => {
            let message = error.to_string();
            let is_math = matches!(error, TexError::Math(_));
            let names_construct = message.contains("dx");
            let named_refusal = message.contains("not yet supported");
            ctx.event(
                LogEvent::new("e2e.failure")
                    .field(
                        "error_kind",
                        if is_math { "TexError::Math" } else { "other" },
                    )
                    .field("names_construct", truth(names_construct))
                    .field("named_refusal", truth(named_refusal)),
            );
            ctx.counter(
                "error_named",
                u64::from(is_math && names_construct && named_refusal),
            );
            if !(is_math && names_construct && named_refusal) {
                return Err(fail(format!(
                    "unsupported-construct refusal lost its precise identity: {message}"
                )));
            }
            Ok(RunOutcome::ok()
                .with_counter("error_named", 1)
                .with_counter("exit_code", 0))
        }
    }
}

/// Corrupt cache: a flipped byte in a store object is detected, evicted,
/// and reported as a miss — and the read-through path recomputes. The
/// whole drill runs in the deterministic lab (VirtualFs + FakeClock), so
/// the corruption is a function of the scenario, never the host disk.
fn failure_cache_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_cache::{CacheKey, NamespacePolicy, Store, StoreConfig};
    use fmn_platform::clock::FakeClock;
    use fmn_platform::fs::VirtualFs;

    let fs = Arc::new(VirtualFs::new());
    let clock = Arc::new(FakeClock::new());
    let cache_root = if cfg!(windows) {
        r"C:\e2e-cache"
    } else {
        "/e2e-cache"
    };
    let store = Store::open(fs.clone(), clock, cache_root, StoreConfig::default())
        .map_err(|error| fail(format!("store opens: {error}")))?;
    let namespace = store
        .namespace(
            "e2e",
            1,
            NamespacePolicy {
                ceiling_bytes: None,
            },
        )
        .map_err(|error| fail(format!("namespace opens: {error}")))?;
    let key = CacheKey::of_content(b"e2e.corruption-drill.victim");
    namespace
        .put(&key, b"good value")
        .map_err(|error| fail(format!("cache put: {error}")))?;

    // Find the one object file and flip a byte in the middle.
    let mut objects = Vec::new();
    collect_virtual_objects(fs.as_ref(), std::path::Path::new(cache_root), &mut objects);
    if objects.len() != 1 {
        return Err(fail(format!(
            "expected exactly one cache object, found {}",
            objects.len()
        )));
    }
    let victim = &objects[0];
    let mut bytes = fs
        .read(victim)
        .map_err(|error| fail(format!("read cache object: {error}")))?;
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x40;
    fs.write_atomic(victim, &bytes)
        .map_err(|error| fail(format!("corrupt cache object: {error}")))?;

    // Detection ⇒ eviction ⇒ miss, never an error, never a panic.
    let after = namespace
        .get(&key)
        .map_err(|error| fail(format!("corrupt lookup escalated to an error: {error}")))?;
    let evicted = !fs.exists(victim);
    let miss = after.is_none();
    ctx.event(
        LogEvent::new("e2e.failure")
            .field("cache_corrupt_miss", truth(miss))
            .field("cache_corrupt_evicted", truth(evicted)),
    );

    // The read-through path recomputes and repopulates cleanly.
    let recomputed: Vec<u8> = match namespace.get_or_compute(&key, || {
        Ok::<_, std::convert::Infallible>(b"good value".to_vec())
    }) {
        Ok(value) => value,
        Err(error) => match error {},
    };
    let recovered = recomputed == b"good value"
        && namespace
            .get(&key)
            .map_err(|error| fail(format!("post-recovery get: {error}")))?
            .as_deref()
            == Some(&b"good value"[..]);
    ctx.counter("cache_corrupt_miss", u64::from(miss));
    ctx.counter("cache_recovery_ok", u64::from(evicted && recovered));
    if !(miss && evicted && recovered) {
        return Err(fail(
            "corrupt cache entry was not (detected, evicted, recomputed)",
        ));
    }
    Ok(RunOutcome::ok()
        .with_counter("cache_corrupt_miss", 1)
        .with_counter("cache_recovery_ok", 1))
}

fn collect_virtual_objects(
    fs: &fmn_platform::fs::VirtualFs,
    dir: &std::path::Path,
    out: &mut Vec<PathBuf>,
) {
    if let Ok(children) = fs.list_dir(dir) {
        for child in children {
            if fs.read(&child).is_ok() {
                // Component-wise: Windows paths separate with backslashes.
                if child.components().any(|c| c.as_os_str() == "objects") {
                    out.push(child);
                }
            } else {
                collect_virtual_objects(fs, &child, out);
            }
        }
    }
}

/// Invalid CLI flag combination: `-l -m` is a usage error with the
/// `quality-exclusive` rule identity, exit code 2, one NDJSON line on
/// stdout, and an empty stderr — through fmn_cli's in-process runner
/// (never a subprocess). Parse errors never touch the host filesystem.
fn failure_cli_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let output = fmn_cli::run(["--robot", "-l", "-m"]);
    let single_line = output.stdout.lines().count() == 1 && !output.stdout.is_empty();
    let names_rule = output.stdout.contains("\"rule\":\"quality-exclusive\"");
    let names_exit = output.stdout.contains("\"exit_name\":\"usage\"");
    let clean_stderr = output.stderr.is_empty();
    ctx.event(
        LogEvent::new("e2e.failure")
            .field("rule", "quality-exclusive")
            .field("rule_named", truth(names_rule))
            .field("exit_named", truth(names_exit))
            .field("ndjson_single_line", truth(single_line))
            .field("stderr_empty", truth(clean_stderr)),
    );
    ctx.counter("cli_rule_named", u64::from(names_rule && names_exit));
    if output.code != 2 || !names_rule || !names_exit || !single_line || !clean_stderr {
        return Err(fail(format!(
            "quality-exclusive lost its named identity: code={} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .exit_code(2)
        .with_counter("cli_rule_named", 1))
}

/// The CLI rejects invalid quality policies before probing an external tool.
fn failure_cli_video_quality_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_platform::process::{FfmpegExecutable, FfmpegLocator, FfmpegLocatorError};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct CountTool(AtomicU64);
    impl FfmpegLocator for CountTool {
        fn locate_ffmpeg(
            &self,
            _: &std::path::Path,
        ) -> Result<FfmpegExecutable, FfmpegLocatorError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(FfmpegLocatorError::NotFound)
        }
    }
    let locator = CountTool(AtomicU64::new(0));
    let fs = Arc::new(fmn_platform::fs::VirtualFs::new());
    let runner = Arc::new(fmn_platform::process::ScriptedRunner::new());
    let cases: &[(&[&str], u8, &str)] = &[
        (
            &["--crf", "16", "--video-bitrate", "2000000"],
            2,
            "mutually exclusive",
        ),
        (&["--gif", "--preset", "slow"], 2, "ffmpeg video"),
        (&["--crf", "52"], 2, "0..=51"),
        (&["--vcodec", "h264_nvenc", "--crf", "16"], 3, "software"),
        (
            &["--vcodec", "libx265", "--tune", "film"],
            3,
            "require libx264",
        ),
        (
            &["--transparent", "--video-bitrate", "2000000"],
            3,
            "opaque MP4/MOV",
        ),
    ];
    let mut refused = 0;
    for (args, code, message) in cases {
        let output = fmn_cli::run_with_capabilities(
            args.iter().copied().chain([
                "--robot",
                fmn_cli::BUILTIN_SCENE_SOURCE,
                "circle_shift.v1",
            ]),
            fs.clone(),
            runner.clone(),
            &locator,
        );
        if output.code != *code
            || !output.stderr.is_empty()
            || output.stdout.lines().count() != 1
            || !output.stdout.contains(message)
        {
            return Err(fail(format!(
                "video-quality admission {args:?}: code={} stdout={:?} stderr={:?}",
                output.code, output.stdout, output.stderr
            )));
        }
        refused += 1;
    }
    let tool_calls = locator.0.load(Ordering::SeqCst);
    ctx.event(
        LogEvent::new("e2e.failure.cli_video_quality")
            .field("refused", refused)
            .field("tool_calls", tool_calls),
    );
    Ok(RunOutcome::ok()
        .with_counter("cli_video_quality_refusals", refused)
        .with_counter("cli_video_quality_tool_calls", tool_calls))
}

/// Native CLI job limits are admitted without spawning; invalid limits fail
/// with stable identities before a tool or output generation is selected.
fn failure_cli_ffmpeg_limits_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_platform::process::{FfmpegExecutable, FfmpegLocator, FfmpegLocatorError};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct CountTool(AtomicU64);
    impl FfmpegLocator for CountTool {
        fn locate_ffmpeg(
            &self,
            _: &std::path::Path,
        ) -> Result<FfmpegExecutable, FfmpegLocatorError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(FfmpegLocatorError::NotFound)
        }
    }
    let locator = CountTool(AtomicU64::new(0));
    let fs = Arc::new(fmn_platform::fs::VirtualFs::new());
    let runner = Arc::new(fmn_platform::process::ScriptedRunner::new());
    let mut refused = 0;
    for (args, code) in [
        (&["--ffmpeg-timeout", "0"][..], 2),
        (&["--ffmpeg-max-input-bytes", "0"], 2),
        (&["--gif", "--ffmpeg-max-log-bytes", "73"], 2),
        (&["--ffmpeg-timeout", "18446744073709551615"], 3),
    ] {
        let output = fmn_cli::run_with_capabilities(
            args.iter().copied().chain([
                "--robot",
                fmn_cli::BUILTIN_SCENE_SOURCE,
                "circle_shift.v1",
            ]),
            fs.clone(),
            runner.clone(),
            &locator,
        );
        if output.code != code || !output.stderr.is_empty() || output.stdout.lines().count() != 1 {
            return Err(fail(format!(
                "CLI ffmpeg limit admission {args:?}: code={} stdout={:?}",
                output.code, output.stdout
            )));
        }
        refused += 1;
    }
    let invocation = fmn_cli::parse_args([
        "--ffmpeg-timeout",
        "21600",
        "--ffmpeg-max-artifact-bytes",
        "17179869184",
        "--ffmpeg-max-input-bytes",
        "8589934592",
    ])
    .map_err(|error| fail(error.to_string()))?;
    let fmn_cli::Invocation::Render(command) = invocation else {
        return Err(fail(
            "ffmpeg resource flags did not select a render request",
        ));
    };
    let (limits, input) = command
        .ffmpeg_limits
        .effective_limits()
        .map_err(|error| fail(error.to_string()))?;
    let tool_calls = locator.0.load(Ordering::SeqCst);
    ctx.event(
        LogEvent::new("e2e.failure.cli_ffmpeg_limits")
            .field("refused", refused)
            .field("tool_calls", tool_calls)
            .field("admitted_timeout_seconds", limits.timeout.as_secs())
            .field("admitted_artifact_bytes", limits.max_artifact_bytes)
            .field("admitted_input_bytes", input),
    );
    Ok(RunOutcome::ok()
        .with_counter("cli_ffmpeg_limit_refusals", refused)
        .with_counter("cli_ffmpeg_limit_tool_calls", tool_calls)
        .with_counter("cli_ffmpeg_admitted_timeout", limits.timeout.as_secs())
        .with_counter("cli_ffmpeg_admitted_artifact", limits.max_artifact_bytes)
        .with_counter("cli_ffmpeg_admitted_input", input))
}

/// The facade's MP4 request with no ffmpeg capability is a named capability
/// refusal (D2) that runs no scene code and publishes nothing; a native
/// format from the same options still renders.
fn failure_facade_video_capability_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    struct Shift {
        ran: bool,
    }
    impl fmn::SceneConstruct for Shift {
        fn construct(&mut self, stage: &mut fmn::Stage<'_>) -> fmn::Result<()> {
            self.ran = true;
            let dot = stage.add(fmn::library::Circle::new().radius(0.5))?;
            let shift = dot.animate().shift([0.5, 0.0, 0.0])?;
            stage.play(shift)?;
            Ok(())
        }
    }
    let options = |path: &str, format: fmn::RenderFormat| -> Result<_, ScenarioError> {
        let mut options = fmn::RenderOptions::with_format(path, format)
            .map_err(|error| fail(format!("bundled config: {error}")))?;
        options.config.camera.resolution = (32, 18);
        options.config.camera.fps = 8;
        options.config.render.threads = fmn_config::config::ThreadPolicy::Fixed(1);
        Ok(options)
    };
    let fs = Arc::new(fmn_platform::fs::VirtualFs::new());
    let mut refused_scene = Shift { ran: false };
    let refusal = fmn::render_with_fs(
        &mut refused_scene,
        options("/facade.mp4", fmn::RenderFormat::Mp4)?,
        fs.clone(),
    );
    let named = matches!(&refusal, Err(fmn::RenderError::Capability(message))
        if message.contains("ffmpeg") && message.contains("PNG-sequence"));
    let published = fs.read(std::path::Path::new("/facade.mp4")).is_ok();
    let mut native_scene = Shift { ran: false };
    let native = fmn::render_with_fs(
        &mut native_scene,
        options("/facade.y4m", fmn::RenderFormat::Y4m)?,
        fs.clone(),
    )
    .map_err(|error| fail(format!("native y4m render failed: {error}")))?;
    ctx.event(
        LogEvent::new("e2e.failure")
            .field("rule", "facade-video-capability")
            .field("refusal_named", truth(named))
            .field("scene_ran", truth(refused_scene.ran))
            .field("published", truth(published))
            .field("native_frames", native.artifact.frame_count.to_string()),
    );
    if !named || refused_scene.ran || published || !native_scene.ran {
        return Err(fail(format!(
            "facade video refusal lost its identity: {refusal:?}, ran={}, published={published}",
            refused_scene.ran
        )));
    }
    ctx.counter("facade_video_refusal_named", 1);
    ctx.counter("facade_native_frames", native.artifact.frame_count);
    Ok(RunOutcome::ok()
        .with_counter("facade_video_refusal_named", 1)
        .with_counter("facade_native_frames", native.artifact.frame_count))
}

/// fm-cli-flag-timing-ht01: a window flag on a file render is refused by name
/// with the capability exit before anything is published, and the same
/// render without it keeps the requested timing (60 fps header, 23 frames).
fn failure_cli_window_flag_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("failure_cli_window_flag")?;
    let render = |name: &str, extra: &[&str]| -> Result<_, ScenarioError> {
        let output_dir = dir.join(name);
        let output_text = output_dir
            .to_str()
            .ok_or_else(|| fail("window-flag output path is not UTF-8"))?
            .to_owned();
        let mut argv = vec![
            "--robot",
            "--format",
            "y4m",
            "--resolution",
            "64x36",
            "--fps",
            "60",
            "--threads",
            "1",
        ];
        argv.extend_from_slice(extra);
        argv.extend_from_slice(&["--video_dir", &output_text]);
        argv.extend_from_slice(&[fmn_cli::BUILTIN_SCENE_SOURCE, "circle_shift.v1"]);
        Ok((fmn_cli::run(argv), output_dir))
    };
    let (refused, refused_dir) = render("presenter", &["-p"])?;
    let published_nothing = !refused_dir.exists()
        || std::fs::read_dir(&refused_dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
    let names_studio = refused.stdout.contains("fmn studio");
    let (plain, plain_dir) = render("plain", &[])?;
    let artifact = std::fs::read_dir(&plain_dir)
        .map_err(|error| fail(format!("list plain output: {error}")))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "y4m"))
        .ok_or_else(|| fail("the plain render published no y4m"))?;
    let bytes =
        std::fs::read(&artifact).map_err(|error| fail(format!("read plain y4m: {error}")))?;
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| fail("y4m header line missing"))?;
    let header = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let frames = bytes
        .windows(6)
        .filter(|window| window == b"FRAME\n")
        .count();
    let rate_ok = header.split_whitespace().any(|token| token == "F60:1");
    ctx.event(
        LogEvent::new("e2e.failure")
            .field("rule", "window-flag-on-file-render")
            .field("refused_code", u64::from(refused.code))
            .field("published_nothing", truth(published_nothing))
            .field("names_studio", truth(names_studio))
            .field("plain_header", header.clone())
            .field("plain_frames", frames),
    );
    if refused.code != 4
        || !published_nothing
        || !names_studio
        || plain.code != 0
        || !rate_ok
        || frames != 23
    {
        return Err(fail(format!(
            "window flag: refused code={} published_nothing={published_nothing} \
             names_studio={names_studio} stdout={:?}; plain code={} header={header:?} frames={frames}",
            refused.code, refused.stdout, plain.code
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_window_flag_refused", 1)
        .with_counter("cli_plain_render_frames", frames as u64))
}

/// Collect real host capabilities through the same in-process CLI dispatch
/// used by the shipping executable. The optional encoder is deliberately
/// absent, so this scenario does not depend on an installed ffmpeg or probe
/// its hardware. Strict field parsing also lives in the doctor_smoke target;
/// the harness's NdjsonSchema assertion covers its own event log only.
fn cli_doctor_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_doctor")?;
    let missing = dir.join("missing-ffmpeg");
    let cache = dir.join("cache");
    let missing_text = missing
        .to_str()
        .ok_or_else(|| fail("doctor scenario ffmpeg path is not UTF-8"))?;
    let cache_text = cache
        .to_str()
        .ok_or_else(|| fail("doctor scenario cache path is not UTF-8"))?;
    let base = [
        "doctor",
        "--ffmpeg",
        missing_text,
        "--cache-dir",
        cache_text,
        "--threads",
        "1",
    ];
    let robot = fmn_cli::run(base.into_iter().chain(["--robot"]));
    let kinds = [
        "topology",
        "execution_plan",
        "ffmpeg",
        "cache",
        "fonts",
        "math_packs",
        "certification",
    ];
    let lines: Vec<_> = robot.stdout.lines().collect();
    if robot.code != 0
        || !robot.stderr.is_empty()
        || !robot.stdout.ends_with('\n')
        || lines.len() != kinds.len()
        || !lines.iter().zip(kinds).all(|(line, kind)| {
            line.starts_with(&format!(
                "{{\"schema\":\"fmn.doctor\",\"version\":1,\"kind\":\"{kind}\","
            )) && line.ends_with('}')
        })
    {
        return Err(fail("doctor omitted its ordered version-1 robot records"));
    }
    let human = fmn_cli::run(base);
    if human.code != 0
        || !human.stderr.is_empty()
        || !human.stdout.starts_with("FrankenManim doctor\n")
        || ![
            "topology:",
            "cores:",
            "SIMD:",
            "plan:",
            "ffmpeg: unavailable",
            "alternative:",
            "cache:",
            "fonts:",
            "math packs:",
            "certification:",
        ]
        .iter()
        .all(|label| human.stdout.lines().any(|line| line.starts_with(label)))
    {
        return Err(fail("doctor human report omitted a capability section"));
    }
    let required = fmn_cli::run(base.into_iter().chain(["--robot", "--require-ffmpeg"]));
    let refusal = required.stdout.lines().last().unwrap_or_default();
    if required.code != 4
        || !required.stderr.is_empty()
        || required.stdout.lines().count() != 8
        || !refusal.contains("\"kind\":\"error\"")
        || !refusal.contains("\"exit_name\":\"capability\"")
        || !refusal.contains("native PNG-sequence, GIF, and y4m outputs remain available")
        || cache.exists()
        || missing.exists()
    {
        return Err(fail(
            "doctor lost its read-only capability refusal contract",
        ));
    }
    ctx.event(
        LogEvent::new("e2e.cli.doctor")
            .field("records", 7_u64)
            .field("human_sections", 10_u64)
            .field("required_ffmpeg_exit", 4_u64)
            .field("read_only", "true"),
    );
    Ok(RunOutcome::ok()
        .with_counter("doctor_records", 7)
        .with_counter("doctor_human_sections", 10)
        .with_counter("doctor_required_ffmpeg_exit", 4))
}

/// The certified contract permanently refuses accelerator work before
/// output publication, even when the binary was compiled with an annex.
fn failure_cli_certified_metal_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_certified_metal_refusal")?;
    let destination = dir.join("must-not-exist");
    let destination_text = destination
        .to_str()
        .ok_or_else(|| fail("certified Metal refusal path is not UTF-8"))?;
    let output = fmn_cli::run([
        "--robot",
        "--reproducible",
        "--engine",
        "metal",
        "--format",
        "png_sequence",
        "--video_dir",
        destination_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let single_line = output.stdout.lines().count() == 1 && !output.stdout.is_empty();
    let names_exit = output.stdout.contains("\"exit_name\":\"config\"");
    let names_contract = output.stdout.contains("requires render.engine=cpu");
    let clean_stderr = output.stderr.is_empty();
    let no_output = !destination.exists();
    let refused = output.code == 3
        && single_line
        && names_exit
        && names_contract
        && clean_stderr
        && no_output;
    ctx.event(
        LogEvent::new("e2e.failure")
            .field("contract", "certified-cpu-only")
            .field("exit_named", truth(names_exit))
            .field("contract_named", truth(names_contract))
            .field("ndjson_single_line", truth(single_line))
            .field("stderr_empty", truth(clean_stderr))
            .field("output_absent", truth(no_output)),
    );
    ctx.counter("cli_certified_metal_refused", u64::from(refused));
    if !refused {
        return Err(fail(format!(
            "certified Metal refusal drifted: code={} stdout={:?} stderr={:?} output_exists={}",
            output.code,
            output.stdout,
            output.stderr,
            destination.exists(),
        )));
    }
    Ok(RunOutcome::ok()
        .exit_code(3)
        .with_counter("cli_certified_metal_refused", 1))
}

/// fm-certified-closure-integrity-4fei: the certified promise, checked from
/// manifests alone. Two certified renders of one input at different thread
/// counts share a semantic digest and agree on every certified output; a
/// render of a different input (another resolution) is not comparable.
fn determinism_certified_manifest_compare_run(
    ctx: &mut RunCtx,
) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("certified_manifest_compare")?;
    let render = |name: &str, threads: &str, resolution: &str| {
        let out = dir.join(name);
        let out_text = out
            .to_str()
            .ok_or_else(|| fail("certified compare path is not UTF-8"))?
            .to_owned();
        let output = fmn_cli::run([
            "--robot",
            "--reproducible",
            "--format",
            "png_sequence",
            "--resolution",
            resolution,
            "--fps",
            "8",
            "--threads",
            threads,
            "--video_dir",
            out_text.as_str(),
            fmn_cli::BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ]);
        if output.code != 0 {
            return Err(fail(format!(
                "certified render {name} failed: code={} stdout={:?} stderr={:?}",
                output.code, output.stdout, output.stderr
            )));
        }
        let bytes = std::fs::read(out.join("circle_shift.v1.manifest").join("manifest.fmnp"))
            .map_err(|error| fail(format!("read {name} manifest: {error}")))?;
        ProvenanceManifest::from_bytes(&bytes)
            .map_err(|error| fail(format!("decode {name} manifest: {error}")))
    };
    let one = render("threads1", "1", "96x54")?;
    let four = render("threads4", "4", "96x54")?;
    let other = render("other_input", "1", "80x45")?;
    let same = one
        .compare(&four)
        .map_err(|error| fail(format!("compare: {error}")))?;
    let different = one
        .compare(&other)
        .map_err(|error| fail(format!("compare: {error}")))?;
    let has_platform_record = one.items.iter().any(ClosureItem::is_platform_specific);
    // C6 names every compiled font face by its exact bytes (bundling is
    // not an exemption).
    let fonts_recorded = fmn_text::bundled_faces().iter().all(|(name, bytes)| {
        let path = format!("bundled-font/{name}");
        one.items.iter().any(|item| {
            item.item_id == 6
                && item.virtual_path.as_deref() == Some(path.as_str())
                && item.digest == sha256(bytes)
        })
    });
    // ADR-0025: the manifest carries exactly the identity the binary reports,
    // and that identity names its sources (a clean commit, or the commit plus
    // the digest of its exact uncommitted state).
    let version = fmn_cli::run(["--version", "--robot"]);
    let reported = version
        .stdout
        .split("\"build_id\":\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default();
    let build_named = !reported.is_empty()
        && one.identity.build_id == reported
        && fmn_output::certified_build_refusal(reported).is_none();
    ctx.event(
        LogEvent::new("e2e.determinism.manifest")
            .field("same_input_agree", truth(same.certified_bits_agree()))
            .field("same_input_semantic_equal", truth(same.semantic_equal))
            .field(
                "other_input_semantic_equal",
                truth(different.semantic_equal),
            )
            .field("platform_record", truth(has_platform_record))
            .field("fonts_recorded", truth(fonts_recorded))
            .field("build_identity_named", truth(build_named)),
    );
    let agree = same.certified_bits_agree();
    let distinguished = !different.semantic_equal && !different.certified_bits_agree();
    ctx.counter("certified_manifest_agree", u64::from(agree));
    ctx.counter(
        "certified_manifest_input_distinguished",
        u64::from(distinguished),
    );
    if !agree || !distinguished || !has_platform_record || !fonts_recorded || !build_named {
        return Err(fail(format!(
            "certified manifest comparison drifted: same={same:?} other={different:?} platform_record={has_platform_record} fonts_recorded={fonts_recorded} build_id={:?} reported={reported:?}",
            one.identity.build_id
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("certified_manifest_agree", 1)
        .with_counter("certified_manifest_input_distinguished", 1))
}

/// fm-inr.1: caller-authored baseline qualification bits are not authority.
/// The real producer identity seam must downgrade them unless a live opaque
/// host token from `perf_host` is present in the same process.
fn failure_host_attestation_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let key = BenchmarkKey {
        profile_id: "caller-claimed-host-v1".to_owned(),
        build_profile: fmn_conformance::perf::COMPILED_CARGO_PROFILE.to_owned(),
        host_fingerprint: sha256(b"caller-host"),
        toolchain_fingerprint: sha256(b"caller-toolchain"),
        suite_lock_digest: sha256(b"caller-suite"),
        benchmark_definition: sha256(b"definition"),
        gate: GateId::Pg2,
        scenario: "fill-canonical".to_owned(),
        unit: MetricUnit::MegaPixelsPerSecondMilli,
        engine: "fast-cpu".to_owned(),
        tier: "portable".to_owned(),
        thread_profile: "fixed-8".to_owned(),
        execution_plan_digest: sha256(b"plan"),
        config_digest: sha256(b"config"),
        cache_state: "warm".to_owned(),
        output_mode: "raw".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    };
    let (found, evidence) = measurement_identity(&key, None)
        .map_err(|error| fail(format!("host identity refusal: {error}")))?;
    let refused = !found.bare_metal && !found.isolated && evidence.is_empty();
    ctx.event(
        LogEvent::new("performance.host_attestation")
            .field("token_present", "false")
            .field("bare_metal", truth(found.bare_metal))
            .field("isolated", truth(found.isolated))
            .field("caller_claim_refused", truth(refused)),
    );
    ctx.counter("host_qualification_refused", u64::from(refused));
    if !refused {
        return Err(fail(
            "caller-authored host booleans survived without a live attestation token",
        ));
    }
    Ok(RunOutcome::ok().with_counter("host_qualification_refused", 1))
}

/// Studio's first real Gauntlet row: select a shipped scene through the CLI
/// schema and compose its preview through the same worker renderer root used by
/// `fmn studio`. The CLI runtime-boundary suite separately proves that this
/// root survives the disposable-worker and loopback-host boundary.
fn studio_preview_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let invocation = fmn_cli::parse_args([
        "studio",
        "--no-browser",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        fmn_cli::BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ])
    .map_err(|error| fail(format!("parse Studio scenario: {error}")))?;
    let fmn_cli::Invocation::Studio(command) = invocation else {
        return Err(fail("Studio scenario parsed to a different front door"));
    };
    let stream = fmn_cli::compose_studio_preview_frame(&fmn_platform::fs::StdFs, &command, 1)
        .map_err(|error| fail(format!("compose Studio preview: {error}")))?;
    stream
        .validate(fmn_studio::ProtocolLimits::default())
        .map_err(|error| fail(format!("validate Studio frame protocol: {error}")))?;
    let backend_is_stream = stream.render_backends.len() == 1
        && stream.render_backends[0].role() == fmn_scene::RenderBackendRole::FrameStream;
    let fmn_studio::FramePayload::Pipe { bytes, .. } = stream.payload else {
        return Err(fail(
            "Studio scenario did not return its bounded pipe frame",
        ));
    };
    let decoded = fmn_codec::decode_png(
        &bytes,
        &fmn_codec::PngLimits {
            max_pixels: u64::from(stream.width) * u64::from(stream.height),
            ..fmn_codec::PngLimits::default()
        },
    )
    .map_err(|error| fail(format!("decode Studio preview PNG: {error}")))?;
    let dimensions_match = decoded.width == 96 && decoded.height == 54;
    ctx.event(
        LogEvent::new("e2e.studio.preview")
            .field("scene", stream.scene.as_str())
            .field("frame", stream.frame_index)
            .field("encoding", "png")
            .field("backend_is_stream", truth(backend_is_stream))
            .field("dimensions_match", truth(dimensions_match)),
    );
    ctx.counter("studio_preview_frames", 1);
    ctx.counter("studio_png_dimensions", u64::from(dimensions_match));
    ctx.counter("studio_backend_journaled", u64::from(backend_is_stream));
    if stream.scene != "circle_shift.v1"
        || stream.frame_index != 1
        || !backend_is_stream
        || !dimensions_match
    {
        return Err(fail(format!(
            "Studio preview drifted: scene={:?} frame={} backend_stream={} dimensions={}x{}",
            stream.scene, stream.frame_index, backend_is_stream, decoded.width, decoded.height
        )));
    }
    Ok(RunOutcome::ok()
        .with_artifact("studio_frame.png", bytes)
        .with_counter("studio_preview_frames", 1)
        .with_counter("studio_png_dimensions", 1)
        .with_counter("studio_backend_journaled", 1))
}

/// Exercise the live mixed 3D registration through the public Studio root.
/// Surfaces, point clouds, images and vectors must reach the camera renderer,
/// retain their motion, and produce identical pixels at either thread width.
fn studio_camera_preview_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let fs = fmn_platform::fs::VirtualFs::new();
    let mut baseline = None;
    let mut camera_frames = 0_u64;
    let mut moving_runs = 0_u64;
    let mut matching_runs = 0_u64;
    let mut artifact = Vec::new();
    for threads in ["1", "4"] {
        let invocation = fmn_cli::parse_args([
            "studio",
            "--no-browser",
            "--resolution",
            "64x48",
            "--fps",
            "8",
            "--threads",
            threads,
            fmn_cli::BUILTIN_SCENE_SOURCE,
            "mixed_camera.v1",
        ])
        .map_err(|error| fail(format!("parse live camera Studio: {error}")))?;
        let fmn_cli::Invocation::Studio(command) = invocation else {
            return Err(fail("camera Studio parsed to a different front door"));
        };
        let mut frames = Vec::new();
        for index in [0, 8] {
            let stream = fmn_cli::compose_studio_preview_frame(&fs, &command, index)
                .map_err(|error| fail(format!("compose live camera frame: {error}")))?;
            stream
                .validate(fmn_studio::ProtocolLimits::default())
                .map_err(|error| fail(format!("validate live camera frame: {error}")))?;
            let camera_identity = stream.render_backends.len() == 1
                && stream.render_backends[0]
                    .identity()
                    .windows(b"lumen-retained-camera-cpu-v1".len())
                    .any(|part| part == b"lumen-retained-camera-cpu-v1");
            if !camera_identity
                || stream.scene != "mixed_camera.v1"
                || stream.frame_index != index as u64
                || (stream.width, stream.height) != (64, 48)
            {
                return Err(fail(
                    "live Studio lost its requested scene, frame or camera renderer",
                ));
            }
            let fmn_studio::FramePayload::Pipe { bytes, .. } = stream.payload else {
                return Err(fail("live camera Studio omitted its bounded PNG payload"));
            };
            let decoded = fmn_codec::decode_png(&bytes, &fmn_codec::PngLimits::default())
                .map_err(|error| fail(format!("decode live camera frame: {error}")))?;
            if (decoded.width, decoded.height) != (64, 48) {
                return Err(fail(
                    "live camera PNG dimensions differ from the frame protocol",
                ));
            }
            camera_frames += 1;
            ctx.event(
                LogEvent::new("e2e.studio.camera_preview")
                    .field("scene", "mixed_camera.v1")
                    .field("threads", threads)
                    .field("frame", index as u64)
                    .field("camera_identity", truth(camera_identity))
                    .field("sha256", sha256(&decoded.rgba).to_hex()),
            );
            frames.push(decoded.rgba);
            if threads == "1" && index == 8 {
                artifact = bytes;
            }
        }
        moving_runs += u64::from(frames[0] != frames[1]);
        if let Some(expected) = &baseline {
            matching_runs += u64::from(&frames == expected);
        } else {
            baseline = Some(frames);
        }
    }
    Ok(RunOutcome::ok()
        .with_artifact("studio_camera_frame.png", artifact)
        .with_counter("studio_camera_frames", camera_frames)
        .with_counter("studio_camera_moving_runs", moving_runs)
        .with_counter("studio_camera_matching_runs", matching_runs))
}

const STUDIO_IO_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const STUDIO_HEADER_LIMIT: usize = 16 * 1024;
const STUDIO_BODY_LIMIT: usize = 8 * 1024 * 1024;

/// Own the real CLI process from spawn through every failure path. Pipe
/// readers retain no diagnostics containing the readiness capability.
struct StudioProcess {
    child: std::process::Child,
    stdout_done: std::sync::mpsc::Receiver<Result<usize, std::io::Error>>,
    stderr_done: std::sync::mpsc::Receiver<Result<usize, std::io::Error>>,
}

fn studio_io_error(operation: &'static str, error: std::io::Error) -> ScenarioError {
    fail(format!("Studio {operation}: {:?}", error.kind()))
}

fn studio_drain(mut reader: impl Read) -> Result<usize, std::io::Error> {
    let mut count = 0usize;
    let mut bytes = [0_u8; 4096];
    loop {
        let read = reader.read(&mut bytes)?;
        if read == 0 {
            return Ok(count);
        }
        // Continue draining to prevent pipe backpressure, retaining only a
        // bounded count. Any output beyond readiness is a test refusal.
        count = count.saturating_add(read).min(STUDIO_BODY_LIMIT);
    }
}

impl StudioProcess {
    fn launch(directory: &std::path::Path) -> Result<(Self, StudioEndpoint), ScenarioError> {
        Self::launch_named(directory, "tex_span.v1")
    }

    fn launch_named(
        directory: &std::path::Path,
        scene: &str,
    ) -> Result<(Self, StudioEndpoint), ScenarioError> {
        let (ready_send, ready_recv) = std::sync::mpsc::sync_channel(1);
        let (stdout_send, stdout_done) = std::sync::mpsc::sync_channel(1);
        let (stderr_send, stderr_done) = std::sync::mpsc::sync_channel(1);
        let mut command = std::process::Command::new(env!("CARGO_BIN_FILE_FMN_CLI_fmn"));
        command
            .args([
                "studio",
                "--robot",
                "--no-browser",
                "--resolution",
                "96x54",
                "--fps",
                "8",
                "--threads",
                "1",
                fmn_cli::BUILTIN_SCENE_SOURCE,
                scene,
            ])
            .current_dir(directory)
            .env_clear()
            .env("PATH", "")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        // Native Windows system DLL lookup retains its host-owned directory;
        // no Python, user configuration, or executable search path is inherited.
        if cfg!(windows)
            && let Some(root) = std::env::var_os("SystemRoot")
        {
            command.env("SystemRoot", root);
        }
        let child = command
            .spawn()
            .map_err(|error| studio_io_error("spawn", error))?;
        let mut process = Self {
            child,
            stdout_done,
            stderr_done,
        };
        let stdout = process
            .child
            .stdout
            .take()
            .ok_or_else(|| fail("Studio stdout is missing"))?;
        let stderr = process
            .child
            .stderr
            .take()
            .ok_or_else(|| fail("Studio stderr is missing"))?;
        std::thread::Builder::new()
            .name("studio-e2e-stdout".to_owned())
            .spawn(move || {
                let mut reader = std::io::BufReader::new(stdout);
                let mut line = Vec::new();
                let ready = reader
                    .by_ref()
                    .take((STUDIO_HEADER_LIMIT + 1) as u64)
                    .read_until(b'\n', &mut line)
                    .map(|_| line);
                let _ = ready_send.send(ready);
                let _ = stdout_send.send(studio_drain(reader));
            })
            .map_err(|error| studio_io_error("stdout reader", error))?;
        std::thread::Builder::new()
            .name("studio-e2e-stderr".to_owned())
            .spawn(move || {
                let _ = stderr_send.send(studio_drain(stderr));
            })
            .map_err(|error| studio_io_error("stderr reader", error))?;
        let ready = ready_recv
            .recv_timeout(STUDIO_IO_TIMEOUT)
            .map_err(|_| fail("Studio readiness deadline expired"))?
            .map_err(|error| studio_io_error("readiness", error))?;
        if ready.len() > STUDIO_HEADER_LIMIT || !ready.ends_with(b"\n") {
            return Err(fail("Studio readiness exceeded its bounded line contract"));
        }
        let endpoint = StudioEndpoint::from_ready(&ready, scene)?;
        Ok((process, endpoint))
    }

    fn wait_until(
        &mut self,
        deadline: std::time::Instant,
    ) -> Result<std::process::ExitStatus, ScenarioError> {
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| studio_io_error("reap", error))?
            {
                return Ok(status);
            }
            if std::time::Instant::now() >= deadline {
                return Err(fail("Studio shutdown deadline expired"));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn finish(&mut self) -> Result<(), ScenarioError> {
        drop(self.child.stdin.take());
        let status = self.wait_until(std::time::Instant::now() + STUDIO_IO_TIMEOUT)?;
        if !status.success() {
            return Err(fail("Studio did not exit successfully after stdin EOF"));
        }
        for receiver in [&self.stdout_done, &self.stderr_done] {
            let count = receiver
                .recv_timeout(STUDIO_IO_TIMEOUT)
                .map_err(|_| fail("Studio pipe reader did not finish after process exit"))?
                .map_err(|error| studio_io_error("pipe drain", error))?;
            if count != 0 {
                return Err(fail(
                    "Studio emitted unexpected diagnostics after readiness",
                ));
            }
        }
        Ok(())
    }
}

impl Drop for StudioProcess {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        if self
            .wait_until(std::time::Instant::now() + std::time::Duration::from_secs(3))
            .is_err()
        {
            let _ = self.child.kill();
            let _ = self.wait_until(std::time::Instant::now() + std::time::Duration::from_secs(3));
        }
    }
}

/// The capability never implements Debug, enters an error, or reaches logs.
struct StudioEndpoint {
    address: std::net::SocketAddr,
    capability: String,
}

impl StudioEndpoint {
    fn from_ready(bytes: &[u8], scene: &str) -> Result<Self, ScenarioError> {
        let ready =
            std::str::from_utf8(bytes).map_err(|_| fail("Studio readiness is not UTF-8"))?;
        if !ready.contains("\"kind\":\"studio_ready\"")
            || !ready.contains(&format!("\"scene\":\"{scene}\""))
            || !ready.contains("\"worker_generation\":1")
        {
            return Err(fail("Studio readiness has the wrong production identity"));
        }
        let url = ready
            .split_once("\"url\":\"")
            .and_then(|(_, rest)| rest.split_once('"').map(|(url, _)| url))
            .and_then(|url| url.strip_prefix("http://"))
            .ok_or_else(|| fail("Studio readiness omitted its HTTP capability URL"))?;
        let (authority, capability) = url
            .split_once("/?cap=")
            .ok_or_else(|| fail("Studio readiness URL has an unexpected shape"))?;
        let address = authority
            .parse::<std::net::SocketAddr>()
            .map_err(|_| fail("Studio readiness authority is not a socket address"))?;
        if !address.ip().is_loopback()
            || address.port() == 0
            || capability.len() != 64
            || !capability.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(fail(
                "Studio readiness did not supply a loopback session capability",
            ));
        }
        Ok(Self {
            address,
            capability: capability.to_owned(),
        })
    }

    fn connect(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<std::io::BufReader<std::net::TcpStream>, ScenarioError> {
        let mut stream = std::net::TcpStream::connect_timeout(&self.address, STUDIO_IO_TIMEOUT)
            .map_err(|error| studio_io_error("connect", error))?;
        stream
            .set_write_timeout(Some(STUDIO_IO_TIMEOUT))
            .map_err(|error| studio_io_error("write timeout", error))?;
        let header = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nOrigin: http://{}\r\nUser-Agent: OpenAI File Downloader, XaiImageApiFetch/1.0\r\nX-FMN-Capability: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.address,
            self.address,
            self.capability,
            body.len(),
        );
        stream
            .write_all(header.as_bytes())
            .and_then(|()| stream.write_all(body))
            .map_err(|error| studio_io_error("request write", error))?;
        Ok(std::io::BufReader::new(stream))
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        status: u16,
    ) -> Result<Vec<u8>, ScenarioError> {
        let mut reader = self.connect(method, path, body)?;
        let deadline = std::time::Instant::now() + STUDIO_IO_TIMEOUT;
        let header = studio_read_header(&mut reader, deadline)?;
        if !header.starts_with(&format!("HTTP/1.1 {status} ")) {
            return Err(fail(format!(
                "Studio {path} returned an unexpected HTTP status"
            )));
        }
        if !studio_header_field(&header, "Content-Type")?.starts_with("application/json") {
            return Err(fail("Studio API response is not JSON"));
        }
        let count = studio_header_number(&header, "Content-Length")?;
        let bytes = studio_read_body(&mut reader, count, deadline)?;
        if bytes
            .windows(self.capability.len())
            .any(|part| part == self.capability.as_bytes())
        {
            return Err(fail(
                "Studio API unexpectedly disclosed its session capability",
            ));
        }
        Ok(bytes)
    }

    fn stream(&self) -> Result<StudioFrameReader, ScenarioError> {
        let mut reader = self.connect("GET", "/stream", b"")?;
        let header =
            studio_read_header(&mut reader, std::time::Instant::now() + STUDIO_IO_TIMEOUT)?;
        if !header.starts_with("HTTP/1.1 200 ")
            || studio_header_field(&header, "Content-Type")?
                != format!(
                    "multipart/x-mixed-replace; boundary={}",
                    fmn_studio::MULTIPART_BOUNDARY
                )
        {
            return Err(fail("Studio did not open its real multipart PNG stream"));
        }
        Ok(StudioFrameReader { reader })
    }
}

fn studio_read_some(
    reader: &mut std::io::BufReader<std::net::TcpStream>,
    bytes: &mut [u8],
    deadline: std::time::Instant,
) -> Result<usize, ScenarioError> {
    let remaining = deadline
        .checked_duration_since(std::time::Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| fail("Studio HTTP response deadline expired"))?;
    reader
        .get_mut()
        .set_read_timeout(Some(remaining))
        .map_err(|error| studio_io_error("read timeout", error))?;
    let read = reader
        .read(bytes)
        .map_err(|error| studio_io_error("response read", error))?;
    if read == 0 {
        return Err(fail("Studio response ended before its declared boundary"));
    }
    Ok(read)
}

fn studio_read_header(
    reader: &mut std::io::BufReader<std::net::TcpStream>,
    deadline: std::time::Instant,
) -> Result<String, ScenarioError> {
    let mut header = Vec::new();
    while header.len() < STUDIO_HEADER_LIMIT {
        let mut byte = [0];
        studio_read_some(reader, &mut byte, deadline)?;
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            return String::from_utf8(header)
                .map_err(|_| fail("Studio HTTP headers are not UTF-8"));
        }
    }
    Err(fail("Studio HTTP header exceeded its byte budget"))
}

fn studio_header_field<'a>(header: &'a str, key: &str) -> Result<&'a str, ScenarioError> {
    let mut values = header.lines().skip(1).filter_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case(key).then(|| value.trim())
    });
    let value = values
        .next()
        .ok_or_else(|| fail("Studio response omitted a required header"))?;
    if values.next().is_some() {
        return Err(fail("Studio response duplicated a required header"));
    }
    Ok(value)
}

fn studio_header_number(header: &str, key: &str) -> Result<usize, ScenarioError> {
    studio_header_field(header, key)?
        .parse()
        .map_err(|_| fail("Studio response has an invalid numeric header"))
}

fn studio_read_body(
    reader: &mut std::io::BufReader<std::net::TcpStream>,
    count: usize,
    deadline: std::time::Instant,
) -> Result<Vec<u8>, ScenarioError> {
    if count > STUDIO_BODY_LIMIT {
        return Err(fail("Studio response body exceeded its byte budget"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(count)
        .map_err(|_| fail("Studio response body storage refused"))?;
    bytes.resize(count, 0);
    let mut offset = 0;
    while offset < count {
        offset += studio_read_some(reader, &mut bytes[offset..], deadline)?;
    }
    Ok(bytes)
}

struct StudioFrameReader {
    reader: std::io::BufReader<std::net::TcpStream>,
}

impl StudioFrameReader {
    fn frame(&mut self, frame: usize, publication: usize) -> Result<Vec<u8>, ScenarioError> {
        let deadline = std::time::Instant::now() + STUDIO_IO_TIMEOUT;
        let header = studio_read_header(&mut self.reader, deadline)?;
        if !header.starts_with(&format!("--{}\r\n", fmn_studio::MULTIPART_BOUNDARY))
            || studio_header_field(&header, "Content-Type")? != "image/png"
            || studio_header_number(&header, "X-FMN-Frame-Index")? != frame
            || studio_header_number(&header, "X-FMN-Publication-Sequence")? != publication
        {
            return Err(fail(
                "Studio multipart frame or publication identity drifted",
            ));
        }
        let count = studio_header_number(&header, "Content-Length")?;
        let png = studio_read_body(&mut self.reader, count, deadline)?;
        if studio_read_body(&mut self.reader, 2, deadline)? != b"\r\n"
            || sha256(&png).to_string() != studio_header_field(&header, "X-FMN-SHA256")?
        {
            return Err(fail("Studio multipart PNG digest or framing drifted"));
        }
        let decoded = fmn_codec::decode_png(
            &png,
            &fmn_codec::PngLimits {
                max_pixels: 96 * 54,
                ..Default::default()
            },
        )
        .map_err(|_| fail("Studio multipart PNG failed native decode"))?;
        if decoded.width != 96 || decoded.height != 54 {
            return Err(fail("Studio multipart PNG dimensions drifted"));
        }
        let mut pixels = decoded.rgba.as_chunks::<4>().0.iter();
        let first = pixels
            .next()
            .ok_or_else(|| fail("Studio multipart PNG has no pixels"))?;
        if pixels.all(|pixel| pixel == first) {
            return Err(fail("Studio multipart PNG is a uniform empty preview"));
        }
        Ok(png)
    }
}

fn studio_contains(bytes: &[u8], expected: &str) -> Result<(), ScenarioError> {
    let text = std::str::from_utf8(bytes).map_err(|_| fail("Studio JSON is not UTF-8"))?;
    if !text.contains(expected) {
        return Err(fail(format!(
            "Studio response omitted expected field {expected}"
        )));
    }
    Ok(())
}

fn studio_view(bytes: &[u8], frame: usize) -> Result<(), ScenarioError> {
    // Constructed frame zero plus the 0.25 s hold's eight native samples.
    studio_contains(
        bytes,
        &format!(
            "\"view\":{{\"frame_index\":{frame},\"frame_count\":9,\"fps\":30,\"width\":96,\"height\":54,\"scale\":6.75,\"origin\":[48,27],\"y_up\":true,\"input_events\":true,\"input_revision\":"
        ),
    )
}

const STUDIO_BUILTIN_INPUT_CASES: [(&str, usize, bool); 5] = [
    ("circle_shift.v1", 13, false),
    ("tex_span.v1", 9, true),
    ("formula_sheet.v1", 13, true),
    ("semantic_witness.v1", 9, true),
    ("sound_cue.v1", 31, false),
];

/// Ordinary native scene registrations cross the shipping host's guarded input
/// boundary, then reproduce their edited state after a real worker restart.
fn studio_builtin_inputs_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    fn input(
        endpoint: &StudioEndpoint,
        frames: &mut StudioFrameReader,
        frame: usize,
        revision: &mut usize,
        publication: &mut usize,
        fields: &str,
    ) -> Result<Vec<u8>, ScenarioError> {
        let body = format!("{fields}&worker_generation=1&frame={frame}&revision={revision}");
        endpoint.request("POST", "/api/event", body.as_bytes(), 200)?;
        *revision += 1;
        *publication += 1;
        frames.frame(frame, *publication)
    }

    ctx.set_fps((30, 1));
    ctx.record_asset(
        "studio/builtin-source.rs",
        include_bytes!("../../fmn/src/lib.rs"),
    );
    ctx.record_asset(
        "studio/registrations.rs",
        include_bytes!("../../fmn-cli/src/studio_live.rs"),
    );
    let directory = scenario_dir("studio_builtin_inputs")?;
    let mut outcome = RunOutcome::ok();
    for (name, count, has_spans) in STUDIO_BUILTIN_INPUT_CASES {
        let (mut process, endpoint) = StudioProcess::launch_named(&directory, name)?;
        let mut frames = endpoint.stream()?;
        let initial_png = frames.frame(0, 0)?;
        let initial = endpoint.request("GET", "/api/inspect", b"", 200)?;
        studio_contains(&initial, "\"scene_time\":0,")?;
        studio_contains(&initial, &format!("\"frame_count\":{count},\"fps\":30"))?;
        studio_contains(&initial, "\"input_events\":true,\"input_revision\":0")?;
        if has_spans {
            studio_contains(&initial, "\"source_span\":{\"kind\":\"math_glyph\"")?;
        }
        let last = count - 1;
        endpoint.request(
            "POST",
            "/api/scrub",
            format!("frame={last}&commit=true").as_bytes(),
            200,
        )?;
        frames.frame(last, 1)?;
        let mut revision = 1;
        let mut publication = 1;
        input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_press&key=a&modifiers=2",
        )?;
        let selected_png = input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_release&key=a&modifiers=2",
        )?;
        input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_press&key=arrow_right",
        )?;
        let moved_png = input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_release&key=arrow_right",
        )?;
        if moved_png == selected_png {
            return Err(fail(format!(
                "{name}: native editing did not move any rendered pixels"
            )));
        }
        input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_press&key=z&modifiers=2",
        )?;
        let undone_png = input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_release&key=z&modifiers=2",
        )?;
        if undone_png != selected_png {
            return Err(fail(format!(
                "{name}: native undo changed the original selected image"
            )));
        }
        input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_press&key=z&modifiers=3",
        )?;
        let redone_png = input(
            &endpoint,
            &mut frames,
            last,
            &mut revision,
            &mut publication,
            "type=key_release&key=z&modifiers=3",
        )?;
        if redone_png != moved_png {
            return Err(fail(format!("{name}: native redo lost the edited image")));
        }
        let edited = endpoint.request("GET", "/api/inspect", b"", 200)?;
        studio_contains(
            &edited,
            &format!("\"frame_index\":{last},\"frame_count\":{count}"),
        )?;
        studio_contains(&edited, &format!("\"input_revision\":{revision}"))?;
        let stale = endpoint.request(
            "POST",
            "/api/event",
            format!("type=key_press&key=arrow_right&worker_generation=1&frame={last}&revision=0")
                .as_bytes(),
            422,
        )?;
        studio_contains(&stale, "stale native input revision")?;
        if endpoint.request("GET", "/api/inspect", b"", 200)? != edited {
            return Err(fail(format!(
                "{name}: stale input changed the edited scene"
            )));
        }

        endpoint.request("POST", "/api/scrub", b"frame=0&commit=false", 200)?;
        publication += 1;
        if frames.frame(0, publication)? != initial_png {
            return Err(fail(format!(
                "{name}: later edits leaked into constructed frame zero"
            )));
        }
        endpoint.request(
            "POST",
            "/api/scrub",
            format!("frame={last}&commit=true").as_bytes(),
            200,
        )?;
        publication += 1;
        if frames.frame(last, publication)? != moved_png {
            return Err(fail(format!(
                "{name}: forward scrub discarded the committed edits"
            )));
        }
        let committed = endpoint.request("GET", "/api/inspect", b"", 200)?;
        let restarted = endpoint.request("POST", "/api/restart", b"", 200)?;
        studio_contains(&restarted, "\"worker_generation\":2")?;
        studio_contains(&restarted, &format!("\"frame_index\":{last}"))?;
        publication += 1;
        let restored_png = frames.frame(last, publication)?;
        let restored = endpoint.request("GET", "/api/inspect", b"", 200)?;
        if restored_png != moved_png || restored != committed {
            return Err(fail(format!(
                "{name}: worker replay changed native pixels or inspected state"
            )));
        }
        drop(frames);
        process.finish()?;
        ctx.event(
            LogEvent::new("e2e.studio.builtin_input")
                .field("scene", name)
                .field("phase", "edited_undone_replayed")
                .field("frame_count", count as u64)
                .field("inputs", (revision - 1) as u64)
                .field("native_spans", truth(has_spans)),
        );
        outcome = outcome
            .with_artifact(&format!("{name}.replayed.png"), restored_png)
            .with_artifact(&format!("{name}.inspector.json"), restored);
    }
    ctx.counter("studio_live_builtin_families", 5);
    Ok(outcome.with_counter("studio_live_builtin_families", 5))
}

fn studio_interactive_lifecycle_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((30, 1));
    ctx.record_asset(
        "studio/builtin-source.rs",
        include_bytes!("../../fmn/src/lib.rs"),
    );
    let directory = scenario_dir("studio_interactive_lifecycle")?;
    let (mut process, endpoint) = StudioProcess::launch_named(&directory, "interactive.v1")?;
    let mut frames = endpoint.stream()?;
    let initial_png = frames.frame(0, 0)?;
    let initial = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_contains(&initial, "\"input_events\":true,\"input_revision\":0")?;
    studio_contains(&initial, "\"parents\":[0]")?;
    endpoint.request("POST", "/api/scrub", b"frame=0&commit=true", 200)?;
    if frames.frame(0, 1)? != initial_png {
        return Err(fail("Native input checkpoint changed the initial image"));
    }
    ctx.event(LogEvent::new("e2e.studio.interactive").field("phase", "checkpoint"));

    let inputs = [
        "type=key_press&key=t&modifiers=2",
        "type=key_release&key=t&modifiers=2",
        "type=mouse_press&x=-1.5&y=0&button=left&modifiers=2",
        "type=mouse_release&x=-1.5&y=0&button=left&modifiers=2",
        "type=key_press&key=arrow_right",
        "type=key_release&key=arrow_right",
    ];
    let mut moved_png = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        let revision = index + 1;
        let body = format!("{input}&worker_generation=1&frame=0&revision={revision}");
        endpoint.request("POST", "/api/event", body.as_bytes(), 200)?;
        moved_png = frames.frame(0, index + 2)?;
    }
    let decode = |png: &[u8]| {
        fmn_codec::decode_png(
            png,
            &fmn_codec::PngLimits {
                max_pixels: 96 * 54,
                ..Default::default()
            },
        )
        .map_err(|_| fail("Native interactive PNG could not be decoded"))
    };
    if decode(&initial_png)?.rgba == decode(&moved_png)?.rgba {
        return Err(fail(
            "Native keyboard movement did not change decoded pixels",
        ));
    }
    let moved = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_contains(&moved, "\"input_revision\":7")?;
    if moved == initial {
        return Err(fail(
            "Native keyboard movement did not change inspected state",
        ));
    }
    ctx.event(LogEvent::new("e2e.studio.interactive").field("phase", "edited"));

    for (body, diagnostic) in [
        (
            "type=key_press&key=arrow_right&worker_generation=1&frame=0&revision=0",
            "stale native input revision",
        ),
        (
            "type=key_press&key=arrow_right&worker_generation=1&frame=0&revision=7&target=99999",
            "stale native input object",
        ),
        (
            "type=mouse_motion&x=10000000&y=0&dx=0&dy=0&worker_generation=1&frame=0&revision=7",
            "budget",
        ),
    ] {
        let refusal = endpoint.request("POST", "/api/event", body.as_bytes(), 422)?;
        studio_contains(&refusal, diagnostic)?;
        if endpoint.request("GET", "/api/inspect", b"", 200)? != moved {
            return Err(fail("Refused native input mutated inspected state"));
        }
    }
    ctx.event(LogEvent::new("e2e.studio.interactive").field("phase", "refused_atomically"));

    endpoint.request("POST", "/api/scrub", b"frame=2&commit=true", 200)?;
    let committed_png = frames.frame(2, 8)?;
    if decode(&committed_png)?.rgba != decode(&moved_png)?.rgba {
        return Err(fail("Scrubbing discarded the native canvas edit"));
    }
    let committed = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_contains(&committed, "\"input_revision\":8")?;
    let restart = endpoint.request("POST", "/api/restart", b"", 200)?;
    for field in [
        "\"worker_generation\":2",
        "\"reused_entries\":8",
        "\"reexecuted_entries\":0",
        "\"frame_index\":2",
    ] {
        studio_contains(&restart, field)?;
    }
    let restored_png = frames.frame(2, 9)?;
    if decode(&restored_png)?.rgba != decode(&committed_png)?.rgba {
        return Err(fail("Native input replay changed decoded pixels"));
    }
    let restored = endpoint.request("GET", "/api/inspect", b"", 200)?;
    if restored != committed {
        return Err(fail("Native input replay changed inspected state"));
    }
    ctx.event(LogEvent::new("e2e.studio.interactive").field("phase", "restarted"));

    endpoint.request(
        "POST",
        "/api/event",
        b"type=key_press&key=arrow_right&worker_generation=2&frame=2&revision=8",
        200,
    )?;
    let continued_png = frames.frame(2, 10)?;
    if decode(&continued_png)?.rgba == decode(&restored_png)?.rgba {
        return Err(fail("Native replay lost the interactive selection"));
    }
    endpoint.request(
        "POST",
        "/api/event",
        b"type=key_press&key=z&modifiers=2&worker_generation=2&frame=2&revision=9",
        200,
    )?;
    let undone_png = frames.frame(2, 11)?;
    if decode(&undone_png)?.rgba != decode(&restored_png)?.rgba {
        return Err(fail("Native undo did not restore the replayed image"));
    }
    drop(frames);
    process.finish()?;
    ctx.event(LogEvent::new("e2e.studio.interactive").field("phase", "continued_and_undone"));
    ctx.counter("studio_interactive_inputs", 8);
    Ok(RunOutcome::ok()
        .with_artifact("interactive_initial.png", initial_png)
        .with_artifact("interactive_replayed.png", restored_png)
        .with_artifact("interactive_initial_inspector.json", initial)
        .with_artifact("interactive_replayed_inspector.json", restored)
        .with_counter("studio_interactive_inputs", 8))
}

/// The registered native lifecycle crosses the actual shipping executable,
/// supervisor, disposable worker, authenticated HTTP host, and PNG stream.
fn studio_subprocess_lifecycle_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((30, 1));
    ctx.record_asset(
        "studio/builtin-source.rs",
        include_bytes!("../../fmn/src/lib.rs"),
    );
    let directory = scenario_dir("studio_native_lifecycle")?;
    let (mut process, endpoint) = StudioProcess::launch(&directory)?;
    let mut frames = endpoint.stream()?;
    let initial_png = frames.frame(0, 0)?;
    let initial = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_view(&initial, 0)?;
    studio_contains(&initial, "\"scene_time\":0,")?;
    for span in [
        "\"source_span\":{\"kind\":\"math_glyph\",\"start\":6,\"end\":7,\"source_bytes\":11,\"excerpt\":\"x\"",
        "\"source_span\":{\"kind\":\"math_rule\",\"start\":0,\"end\":5,\"source_bytes\":11,\"excerpt\":\"\\\\frac\"",
        "\"source_span\":{\"kind\":\"text_glyph\",\"start\":0,\"end\":1,\"source_bytes\":5,\"excerpt\":\"h\"",
    ] {
        studio_contains(&initial, span)?;
    }
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "initial")
            .field("frame", 0u64)
            .field("publication", 0u64),
    );

    endpoint.request("POST", "/api/scrub", b"frame=0&commit=true", 200)?;
    if frames.frame(0, 1)? != initial_png {
        return Err(fail(
            "Studio checkpoint commit changed the original frame's PNG",
        ));
    }
    studio_view(&endpoint.request("GET", "/api/inspect", b"", 200)?, 0)?;
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "checkpoint_committed")
            .field("frame", 0u64)
            .field("publication", 1u64),
    );

    let committed = endpoint.request("POST", "/api/scrub", b"frame=1&commit=true", 200)?;
    studio_contains(&committed, "\"frame_index\":1")?;
    let committed_png = frames.frame(1, 2)?;
    let committed_inspect = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_view(&committed_inspect, 1)?;
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "committed")
            .field("frame", 1u64)
            .field("publication", 2u64),
    );

    endpoint.request("POST", "/api/scrub", b"frame=0&commit=false", 200)?;
    let transient_png = frames.frame(0, 3)?;
    if transient_png != initial_png {
        return Err(fail(
            "Studio transient scrub changed the original frame's PNG",
        ));
    }
    studio_view(&endpoint.request("GET", "/api/inspect", b"", 200)?, 0)?;
    let refusal = endpoint.request("POST", "/api/scrub", b"frame=9&commit=false", 422)?;
    studio_contains(
        &refusal,
        "native target exceeds the scene or execution budget",
    )?;
    studio_view(&endpoint.request("GET", "/api/inspect", b"", 200)?, 0)?;
    let event_refusal =
        endpoint.request("POST", "/api/event", b"type=key_press&key=arrow_left", 422)?;
    studio_contains(
        &event_refusal,
        "worker_generation, frame and revision are required",
    )?;
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "transient_and_refusals")
            .field("frame", 0u64)
            .field("publication", 3u64),
    );

    let overlays = endpoint.request("GET", "/api/overlays?layers=31", b"", 200)?;
    for field in [
        "\"layers\":31",
        "\"tiles\":[{",
        "\"control_points\":[[",
        "\"bounds\":[[",
        "\"winding\":\"",
    ] {
        studio_contains(&overlays, field)?;
    }
    let restart = endpoint.request("POST", "/api/restart", b"", 200)?;
    for field in [
        "\"status\":\"restarted\"",
        "\"worker_generation\":2",
        "\"reused_entries\":2",
        "\"replayed_entries\":1",
        "\"reexecuted_entries\":0",
        "\"frame_index\":1",
    ] {
        studio_contains(&restart, field)?;
    }
    let replayed_png = frames.frame(1, 4)?;
    if replayed_png != committed_png {
        return Err(fail(
            "Studio worker restart changed the committed frame's PNG",
        ));
    }
    let restored = endpoint.request("GET", "/api/inspect", b"", 200)?;
    studio_view(&restored, 1)?;
    if restored != committed_inspect {
        return Err(fail(
            "Studio restart did not reproduce the committed live inspector",
        ));
    }
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "restarted")
            .field("frame", 1u64)
            .field("publication", 4u64)
            .field("generation", 2u64),
    );
    drop(frames);
    process.finish()?;
    ctx.event(
        LogEvent::new("e2e.studio.lifecycle")
            .field("phase", "clean_eof")
            .field("exit_code", 0u64),
    );
    ctx.counter("studio_native_lifecycle_steps", 6);
    Ok(RunOutcome::ok()
        .with_artifact("studio_initial_inspector.json", initial)
        .with_artifact("studio_restored_inspector.json", restored)
        .with_artifact("studio_overlays.json", overlays)
        .with_artifact("studio_replayed_frame.png", replayed_png)
        .with_counter("studio_native_lifecycle_steps", 6))
}

/// The PyO3 portal's first production-output Gauntlet row: a real Python
/// subclass constructs a Circle, lifecycle capture crosses the native bridge,
/// retained Lumen rasterizes it, and Reel publishes an atomic PNG generation.
fn python_portal_png_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    python_portal_sequence_run(ctx, false)
}

/// Live coefficients and matrix entries through actual native animation/output.
fn python_live_tex_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_live_tex")?;
    let report = manimlib::run_portal_gauntlet_live_tex(&root, ctx.seed)
        .map_err(|error| fail(format!("Python live equations: {error}")))?;
    let first = fmn_codec::decode_png(&report.first_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode first live equation: {error}")))?;
    let last = fmn_codec::decode_png(&report.last_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode last live equation: {error}")))?;
    if (first.width, first.height) != (192, 108)
        || (last.width, last.height) != (192, 108)
        || first.rgba == last.rgba
    {
        return Err(fail("live equations did not animate native 192x108 pixels"));
    }
    ctx.event(
        LogEvent::new("e2e.python.live_tex")
            .field("frames", report.frame_count)
            .field("thread_counts", report.thread_counts)
            .field("failure_paths", report.failure_paths),
    );
    Ok(RunOutcome::ok()
        .with_counter("live_tex_frames", report.frame_count)
        .with_counter("live_tex_thread_counts", report.thread_counts)
        .with_counter("live_tex_failure_paths", report.failure_paths)
        .with_artifact("live_tex_first.png", report.first_png)
        .with_artifact("live_tex_last.png", report.last_png))
}

/// Complex coefficients and matrix entries through actual native animation/output.
fn python_complex_readouts_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_complex_readouts")?;
    let report = manimlib::run_portal_gauntlet_complex_readouts(&root, ctx.seed)
        .map_err(|error| fail(format!("Python complex readouts: {error}")))?;
    let first = fmn_codec::decode_png(&report.first_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode first complex readout: {error}")))?;
    let last = fmn_codec::decode_png(&report.last_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode last complex readout: {error}")))?;
    if (first.width, first.height) != (192, 108)
        || (last.width, last.height) != (192, 108)
        || first.rgba == last.rgba
    {
        return Err(fail(
            "complex readouts did not animate native 192x108 pixels",
        ));
    }
    ctx.event(
        LogEvent::new("e2e.python.complex_readouts")
            .field("frames", report.frame_count)
            .field("thread_counts", report.thread_counts)
            .field("failure_paths", report.failure_paths),
    );
    Ok(RunOutcome::ok()
        .with_counter("complex_readouts_frames", report.frame_count)
        .with_counter("complex_readouts_thread_counts", report.thread_counts)
        .with_counter("complex_readouts_failure_paths", report.failure_paths)
        .with_artifact("complex_readouts_first.png", report.first_png)
        .with_artifact("complex_readouts_last.png", report.last_png))
}

/// Native macros retain formula spans through matching and live-number edits.
fn python_tex_preamble_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_tex_preamble")?;
    let report = manimlib::run_portal_gauntlet_tex_preamble(&root, ctx.seed)
        .map_err(|error| fail(format!("Python TeX preambles: {error}")))?;
    let first = fmn_codec::decode_png(&report.first_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode first preamble frame: {error}")))?;
    let last = fmn_codec::decode_png(&report.last_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode last preamble frame: {error}")))?;
    if (first.width, first.height) != (192, 108)
        || (last.width, last.height) != (192, 108)
        || first.rgba == last.rgba
    {
        return Err(fail("TeX preambles did not animate native 192x108 pixels"));
    }
    ctx.event(
        LogEvent::new("e2e.python.tex_preamble")
            .field("frames", report.frame_count)
            .field("thread_counts", report.thread_counts)
            .field("failure_paths", report.failure_paths),
    );
    Ok(RunOutcome::ok()
        .with_counter("tex_preamble_frames", report.frame_count)
        .with_counter("tex_preamble_thread_counts", report.thread_counts)
        .with_counter("tex_preamble_failure_paths", report.failure_paths)
        .with_artifact("tex_preamble_first.png", report.first_png)
        .with_artifact("tex_preamble_last.png", report.last_png))
}

fn python_textured_surfaces_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let (first, last) = manimlib::run_portal_gauntlet_textures()
        .map_err(|error| fail(format!("textured surfaces: {error}")))?;
    let limits = fmn_codec::PngLimits::default();
    let decoded = fmn_codec::decode_png(&first, &limits)
        .map_err(|error| fail(format!("decode texture frame: {error}")))?;
    if (decoded.width, decoded.height) != (96, 54) || first == last {
        return Err(fail("textured surface did not animate native pixels"));
    }
    ctx.event(
        LogEvent::new("e2e.python.textures")
            .field("frames", 4_u64)
            .field("thread_counts", 3_u64),
    );
    Ok(RunOutcome::ok()
        .with_artifact("textures_first.png", first)
        .with_artifact("textures_last.png", last))
}

fn python_scene_console_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let checks = manimlib::run_portal_gauntlet_console()
        .map_err(|error| fail(format!("Python scene console: {error}")))?;
    ctx.event(LogEvent::new("e2e.python.scene_console").field("checks", checks));
    Ok(RunOutcome::ok().with_counter("console_checks", checks))
}

/// Authored Surface hooks (fm-5wq.13): subclasses of the stock solids
/// sample their authored `uv_func` and dispatch `init_points` once at
/// construction, and render the same bytes as the equivalent
/// ParametricSurface.
fn python_surface_hooks_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let cases = manimlib::run_portal_gauntlet_surface_lifecycle()
        .map_err(|error| fail(format!("Python surface hooks: {error}")))?;
    ctx.event(LogEvent::new("e2e.python.surface_hooks").field("cases", cases));
    Ok(RunOutcome::ok().with_counter("surface_hook_cases", cases))
}

/// Structural parity (fm-5wq.36): the fixed construction set, extracted in
/// the embedded portal by the same `structural_facts.py` the Reference ran,
/// matches the checked-in Reference facts except for BN-keyed and open-bead
/// rows. A stale open-bead row (its bug fixed) fails, forcing its removal.
fn python_structural_facts_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let report = manimlib::run_portal_gauntlet_structural_facts()
        .map_err(|error| fail(format!("structural parity with the Reference: {error}")))?;
    ctx.event(
        LogEvent::new("e2e.python.structural_facts")
            .field("compared", report.compared)
            .field("equal", report.equal)
            .field("equal_with_exclusions", report.equal_with_exclusions)
            .field("summary", report.summary.as_str()),
    );
    Ok(RunOutcome::ok()
        .with_counter("structural_subjects", report.compared)
        .with_counter("structural_equal", report.equal))
}

/// The Appendix-A class sweep (fm-5wq.26): every public mobject class's
/// default and declared constructions, public methods included, match the
/// pinned Reference's facts in the embedded portal. Anything else must be
/// covered by a cited BN, ADR or open-bead row.
fn python_class_sweep_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let report = manimlib::run_portal_gauntlet_class_sweep()
        .map_err(|error| fail(format!("class sweep against the Reference: {error}")))?;
    ctx.event(
        LogEvent::new("e2e.python.class_sweep")
            .field("compared", report.compared)
            .field("equal", report.equal)
            .field("equal_with_exclusions", report.equal_with_exclusions)
            .field("summary", report.summary.as_str()),
    );
    Ok(RunOutcome::ok()
        .with_counter("class_sweep_subjects", report.compared)
        .with_counter("class_sweep_equal", report.equal))
}

/// Envelopes, not blankets (fm-5wq.45): the embedded portal's `Tex` passes the
/// exclusion table, and the same facts with the root's height doubled fail it
/// with a named envelope violation. The blanket bbox rows admitted both.
fn python_structural_envelope_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let report = manimlib::run_portal_gauntlet_envelope_drill()
        .map_err(|error| fail(format!("structural envelope drill: {error}")))?;
    ctx.event(
        LogEvent::new("e2e.python.structural_envelope")
            .field("subject", report.subject.as_str())
            .field("clean_verdict", report.clean_verdict.as_str())
            .field("planted_verdict", report.planted_verdict.as_str())
            .field("violated_row", report.violated_row.as_str())
            .field(
                "planted_size_rel_milli",
                (report.size_rel * 1000.0).round() as u64,
            ),
    );
    Ok(RunOutcome::ok().with_counter("envelope_violations_detected", 1))
}

/// Reference corpus regressions (fm-5wq.30 first; fm-5wq.27..29 and
/// fm-sq8.10 add their cases): minimized constructs from corpus scenes the
/// portal once refused or rendered differently, each checked against what the
/// pinned Reference showed. One log event per case; a zero-case run fails.
fn python_reference_corpus_regressions_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let cases = manimlib::run_portal_gauntlet_reference_corpus_regressions()
        .map_err(|error| fail(format!("Reference corpus regressions: {error}")))?;
    for case in &cases {
        ctx.event(
            LogEvent::new("e2e.python.reference_corpus_regression")
                .field("bead", case.bead.as_str())
                .field("construct", case.construct.as_str())
                .field("observed", case.observed.as_str())
                .field("expected", case.expected.as_str())
                .field("provenance", case.provenance.as_str())
                .field("verdict", "pass"),
        );
    }
    Ok(RunOutcome::ok().with_counter("corpus_regression_cases", cases.len() as u64))
}

fn python_svg_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let report = manimlib::run_portal_gauntlet_svg()
        .map_err(|error| fail(format!("Python native SVG: {error}")))?;
    let document = fmn_library::svg::SvgDocument::parse(&report.document)
        .map_err(|error| fail(format!("parse native SVG: {error}")))?;
    if document.width != 160.0 || document.height != 90.0 || document.shapes.len() < 6 {
        return Err(fail("SVG lost its viewport or native text/math outlines"));
    }
    ctx.event(
        LogEvent::new("e2e.python.svg")
            .field("thread_counts", report.thread_counts)
            .field("failure_paths", report.failure_paths),
    );
    Ok(RunOutcome::ok()
        .with_counter("svg_thread_counts", report.thread_counts)
        .with_counter("svg_failure_paths", report.failure_paths)
        .with_artifact("diagram.svg", report.document))
}

fn python_surface_mesh_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_surface_mesh")?;
    let (png, checks) = manimlib::run_portal_gauntlet_surface_mesh(&root, ctx.seed)
        .map_err(|error| fail(format!("Python surface mesh: {error}")))?;
    let image = fmn_codec::decode_png(&png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode surface mesh: {error}")))?;
    let visible = image
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[0] > pixel[1] && pixel[0] > pixel[2])
        .count() as u64;
    if (image.width, image.height) != (128, 128) || visible == 0 {
        return Err(fail("surface mesh did not render visible 128x128 geometry"));
    }
    ctx.event(
        LogEvent::new("e2e.python.surface_mesh")
            .field("checks", checks)
            .field("analytic_oracle", true)
            .field("thread_replay", true),
    );
    Ok(RunOutcome::ok()
        .with_counter("surface_mesh_checks", checks)
        .with_counter("surface_mesh_visible_pixels", visible)
        .with_artifact("surface_mesh.png", png))
}

fn python_studio_capture_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let report = manimlib::run_portal_gauntlet_studio_capture()
        .map_err(|error| fail(format!("Python Studio capture: {error}")))?;
    let first = fmn_codec::decode_png(&report.first_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode first Studio frame: {error}")))?;
    let last = fmn_codec::decode_png(&report.last_png, &fmn_codec::PngLimits::default())
        .map_err(|error| fail(format!("decode last Studio frame: {error}")))?;
    if report.frames != 6
        || (first.width, first.height) != (96, 54)
        || (last.width, last.height) != (96, 54)
        || first.rgba == last.rgba
    {
        return Err(fail(
            "Studio capture did not render six moving native frames",
        ));
    }
    ctx.event(
        LogEvent::new("e2e.python.studio_capture")
            .field("frames", report.frames)
            .field("read_only", true)
            .field("native_inspector", true),
    );
    Ok(RunOutcome::ok()
        .with_counter("python_studio_frames", report.frames)
        .with_artifact("studio_first.png", report.first_png)
        .with_artifact("studio_last.png", report.last_png)
        .with_artifact("studio_inspector.json", report.last_inspector))
}

fn python_portal_animation_lifecycle_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    python_portal_sequence_run(ctx, true)
}

fn python_portal_native_outputs_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_native_outputs")?;
    let manimlib::PortalOutputGauntletReport {
        frames,
        formats,
        publication_failures: refusals,
        thread_replays: replays,
        sample_frames,
        video_formats,
        video_frames,
        video_capability_refusals: video_refusals,
    } = manimlib::run_portal_gauntlet_native_outputs(&root)
        .map_err(|error| fail(format!("decoded Python native outputs: {error}")))?;
    manimlib::run_portal_gauntlet_video_options()
        .map_err(|error| fail(format!("negotiated Python output profiles: {error}")))?;
    ctx.event(
        LogEvent::new("e2e.python.native_outputs")
            .field("frames", frames)
            .field("formats", formats)
            .field("publication_failures", refusals)
            .field("thread_replays", replays)
            .field("sample_frames", sample_frames)
            .field("video_formats", video_formats)
            .field("video_frames", video_frames)
            .field("video_capability_refusals", video_refusals),
    );
    let mut outcome = RunOutcome::ok()
        .with_counter("portal_output_options_suite", 1)
        .with_counter("native_output_frames", frames)
        .with_counter("native_output_formats", formats)
        .with_counter("native_output_publication_failures", refusals)
        .with_counter("native_output_thread_replays", replays)
        .with_counter("native_output_sample_frames", sample_frames)
        .with_counter("video_output_formats", video_formats)
        .with_counter("video_output_frames", video_frames)
        .with_counter("video_output_capability_refusals", video_refusals);
    for name in [
        "native_scene.py",
        "motion.gif",
        "motion.y4m",
        "primaries.gif",
        "primaries.y4m",
        "soundtrack.wav",
        "cue.wav",
        "quiet.wav",
        "camera-pan.gif",
        "camera-updater.gif",
        "background-changes.gif",
        "light-changes.gif",
        "overridden-light.gif",
        "escaped-markup.png",
        "literal-markup.png",
        "centered-typesetting.gif",
        "rectangle-flash.gif",
    ] {
        let bytes = std::fs::read(root.join(name))
            .map_err(|error| fail(format!("read native output {name}: {error}")))?;
        outcome = outcome.with_artifact(name, bytes);
    }
    if std::env::var("FMN_REQUIRE_FFMPEG").as_deref() == Ok("1") {
        if video_formats != 2 || video_frames != 8 {
            return Err(fail("required real MP4/MOV acceptance did not execute"));
        }
        for name in ["motion.mp4", "motion.mov", "soundtrack.mp4", "tone.wav"] {
            let bytes = std::fs::read(root.join(name))
                .map_err(|error| fail(format!("read video output {name}: {error}")))?;
            outcome = outcome.with_artifact(name, bytes);
        }
    }
    Ok(outcome)
}

fn python_portal_sequence_run(
    ctx: &mut RunCtx,
    animation_lifecycle: bool,
) -> Result<RunOutcome, ScenarioError> {
    let (directory, scene) = if animation_lifecycle {
        ("python_animation_lifecycle", "AnimationLifecycleScene")
    } else {
        ("python_portal_png", "_GauntletPortalScene")
    };
    let root = scenario_dir(directory)?;
    let destination = root.join("frames");
    let report = if animation_lifecycle {
        manimlib::run_portal_gauntlet_animation_lifecycle(&destination, ctx.seed)
    } else {
        manimlib::run_portal_gauntlet_png_sequence(&destination, ctx.seed)
    }
    .map_err(|error| fail(format!("run Python portal PNG scenario: {error}")))?;
    let mut png_paths: Vec<PathBuf> = std::fs::read_dir(&destination)
        .map_err(|error| fail(format!("read {}: {error}", destination.display())))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| fail(format!("read Python portal directory entry: {error}")))
        })
        .collect::<Result<_, _>>()?;
    png_paths.retain(|path| path.extension().is_some_and(|extension| extension == "png"));
    png_paths.sort();
    let pngs: Vec<Vec<u8>> = png_paths
        .iter()
        .map(|path| {
            std::fs::read(path).map_err(|error| fail(format!("read {}: {error}", path.display())))
        })
        .collect::<Result<_, _>>()?;
    let byte_count = pngs.iter().try_fold(0_u64, |total, bytes| {
        total
            .checked_add(
                u64::try_from(bytes.len()).map_err(|_| fail("PNG byte count exceeds u64"))?,
            )
            .ok_or_else(|| fail("Python portal byte count overflow"))
    })?;
    let first = pngs
        .first()
        .ok_or_else(|| fail("Python portal published no PNG frames"))?;
    let signature = first.starts_with(b"\x89PNG\r\n\x1a\n");
    let decoded = fmn_codec::decode_png(
        first,
        &fmn_codec::PngLimits {
            max_pixels: 96 * 54,
            ..fmn_codec::PngLimits::default()
        },
    )
    .map_err(|error| fail(format!("decode Python portal PNG: {error}")))?;
    let dimensions_match = decoded.width == 96 && decoded.height == 54;
    let engine_journaled = {
        let fields: Vec<_> = report.engine.split(':').collect();
        fields.len() == 3 && fields[0] == "fast-cpu" && fields[2].parse::<u32>().is_ok()
    };
    let complete = report.path == destination
        && report.frame_count == u64::try_from(pngs.len()).unwrap_or(u64::MAX)
        && report.bytes == byte_count
        && report.digest.len() == 64
        && report.threads == 1
        && signature
        && dimensions_match
        && engine_journaled;
    ctx.event(
        LogEvent::new("e2e.python.render")
            .field("scene", scene)
            .field("format", "png_sequence")
            .field("complete", truth(complete))
            .field("png_signature", truth(signature))
            .field("dimensions_match", truth(dimensions_match))
            .field("engine_journaled", truth(engine_journaled)),
    );
    ctx.counter("python_render_frames", report.frame_count);
    ctx.counter("python_png_signature", u64::from(signature));
    ctx.counter("python_engine_journaled", u64::from(engine_journaled));
    if !complete {
        return Err(fail(format!(
            "Python portal render drifted: path={} frames={} files={} bytes={} tree_bytes={} \
             digest_len={} engine={:?} threads={} signature={} dimensions={}x{}",
            report.path.display(),
            report.frame_count,
            pngs.len(),
            report.bytes,
            byte_count,
            report.digest.len(),
            report.engine,
            report.threads,
            signature,
            decoded.width,
            decoded.height,
        )));
    }
    Ok(RunOutcome::ok()
        .with_artifact("python_portal_frame.png", first.clone())
        .with_counter("python_render_frames", report.frame_count)
        .with_counter("python_png_signature", 1)
        .with_counter("python_engine_journaled", 1))
}

/// fm-sq8.11: a pure-2D Python scene renders on Lumen's retained planar
/// route, and its frames are byte-identical to native `fmn` replaying the
/// same scene's FMTL/1 export (same engine, one thread); a surface scene keeps
/// the camera route and says why. The installed-wheel suite
/// (scene_execution.py) checks that certified provenance names the route.
fn python_portal_planar_route_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_portal_planar_route")?;
    let report = manimlib::run_portal_gauntlet_planar_route(&root)
        .map_err(|error| fail(format!("run Python portal planar route: {error}")))?;
    let native_dir = root.join("native");
    std::fs::create_dir(&native_dir)
        .map_err(|error| fail(format!("create native output directory: {error}")))?;
    let output = fmn_cli::run([
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "64x36",
        "--threads",
        "1",
        "--video_dir",
        native_dir
            .to_str()
            .ok_or_else(|| fail("native output path is not UTF-8"))?,
        report
            .bundle
            .to_str()
            .ok_or_else(|| fail("bundle path is not UTF-8"))?,
        "Flat",
    ]);
    let pngs = |directory: &std::path::Path| -> Result<Vec<Vec<u8>>, ScenarioError> {
        let mut paths: Vec<_> = std::fs::read_dir(directory)
            .map_err(|error| fail(format!("list {}: {error}", directory.display())))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
            .collect();
        paths.sort();
        paths
            .iter()
            .map(|path| {
                std::fs::read(path)
                    .map_err(|error| fail(format!("read {}: {error}", path.display())))
            })
            .collect()
    };
    let portal = pngs(&report.portal_sequence)?;
    let native = pngs(&native_dir.join("Flat"))?;
    let identical_frames = portal.iter().zip(&native).filter(|(a, b)| a == b).count();
    let native_identical = output.code == 0
        && portal.len() as u64 == report.frame_count
        && native.len() == portal.len()
        && identical_frames == portal.len();
    let (planar, camera, reason) = &report.planar_routes;
    let planar_route = *planar == report.frame_count && *camera == 0 && reason.is_none();
    let (surface_planar, surface_camera, surface_reason) = &report.surface_routes;
    let surface_route = *surface_planar == 0 && *surface_camera > 0 && surface_reason.is_some();
    ctx.event(
        LogEvent::new("e2e.python.planar_route")
            .field("frames", report.frame_count)
            .field("planar_frames", *planar)
            .field("camera_frames", *camera)
            .field("identical_frames", identical_frames)
            .field("native_identical", truth(native_identical))
            .field("surface_camera_frames", *surface_camera)
            .field(
                "surface_reason",
                surface_reason.clone().unwrap_or_else(|| "none".to_owned()),
            )
            .field("teams_used", report.planar_plan.0)
            .field("teams_planned", report.planar_plan.1)
            .field("team_threads", report.planar_plan.2)
            .field("frames_in_flight", report.planar_plan.3),
    );
    let (teams_used, teams_planned, team_threads, frames_in_flight) = report.planar_plan;
    let plan_reported = teams_used >= 1
        && teams_used <= teams_planned
        && team_threads >= 1
        && frames_in_flight >= 1;
    if !(native_identical && planar_route && surface_route && plan_reported) {
        return Err(fail(format!(
            "portal planar route: routes={:?} surface={:?} plan={:?} frames={} portal={} \
             native={} identical={identical_frames} cli_code={} stderr={:?}",
            report.planar_routes,
            report.surface_routes,
            report.planar_plan,
            report.frame_count,
            portal.len(),
            native.len(),
            output.code,
            output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("python_planar_route_frames", *planar)
        .with_counter("python_planar_native_identical", 1)
        .with_counter("python_planar_surface_camera", 1))
}

/// The portal's final-state still route: semantic construction completes with
/// intermediate raster work skipped, then the same Lumen/Reel composition root
/// atomically publishes exactly one canonical PNG.
/// Reference defaults and composite braces through the production portal
/// route (fm-5wq.44, fm-5wq.43). The Reference's `Scene` merges
/// `manim_config.camera` first (1920x1080, 30 fps, `#333333`); a portal scene
/// with no camera configuration once rendered at 60 fps on black, and a brace
/// around a detached `VGroup` once had width 0 at the origin.
fn python_portal_reference_defaults_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_portal_reference_defaults")?;
    let destination = root.join("defaults.y4m");
    let report = manimlib::run_portal_gauntlet_reference_defaults(&destination)
        .map_err(|error| fail(format!("run Python portal reference defaults: {error}")))?;
    let bytes = std::fs::read(&destination)
        .map_err(|error| fail(format!("read {}: {error}", destination.display())))?;
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| fail("y4m header line missing"))?;
    let header = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let rate = header
        .split(' ')
        .find_map(|field| field.strip_prefix('F'))
        .unwrap_or("")
        .to_owned();
    let frames = bytes.windows(6).filter(|w| w == b"FRAME\n").count();
    // First frame's first luma sample and first Cb/Cr samples (32x18, 4:2:0).
    let first = header_end + 1 + 6;
    let (luma, chroma) = (32 * 18, 16 * 9);
    let sample = |offset: usize| bytes.get(first + offset).copied().unwrap_or(0);
    let (y, u, v) = (sample(0), sample(luma), sample(luma + chroma));
    // BT.709 limited range for #333333: 16 + 219 * 0x33 / 255 = 59.8.
    let background_is_reference =
        (58..=62).contains(&y) && u.abs_diff(128) <= 1 && v.abs_diff(128) <= 1;
    let rate_is_reference = rate == "30:1" && report.fps == 30;
    let frames_match = frames == 15 && report.frame_count == 15;
    let brace_spans =
        (report.brace_width - report.target_width).abs() < 1e-3 && report.target_width > 4.0;
    // Cross-front-door: the portal Scene's first camera layer is the typed
    // configuration standalone fmn resolves (defaults, no user file).
    let native = fmn_config::Config::resolve(&[], None)
        .map_err(|error| fail(format!("resolve native config: {error}")))?
        .config
        .camera;
    let (resolution, fps, background, opacity) = &report.scene_camera;
    // The Reference's Tex is align* display math: Tex(r"\frac{1}{2}") is 0.9876
    // tall at 6199a00d (TeX Live 2025, measured 2026-10-04); text style is
    // about 0.58 (fm-tex-display-style-nclg).
    let display_style = (report.display_fraction_height / 0.9876 - 1.0).abs() < 0.02;
    // The README quick example's Tex is 3.940671 x 1.149994 on the Reference
    // (measured 2026-10-04); text style would be about half as tall.
    let (readme_width, readme_height) = report.readme_formula;
    let readme_envelope = (readme_width / 3.940_671 - 1.0).abs() < 0.10
        && (readme_height / 1.149_994 - 1.0).abs() < 0.10;
    let front_doors_agree = *resolution == native.resolution
        && *fps == native.fps
        && background.eq_ignore_ascii_case(&native.background_color)
        && (*opacity - native.background_opacity).abs() < 1e-12;
    ctx.event(
        LogEvent::new("e2e.python.reference_defaults")
            .field("rate", rate.as_str())
            .field("frames", frames)
            .field("background_yuv", format!("{y},{u},{v}").as_str())
            .field("brace_width", format!("{:.4}", report.brace_width).as_str())
            .field(
                "target_width",
                format!("{:.4}", report.target_width).as_str(),
            )
            .field("background_is_reference", truth(background_is_reference))
            .field("rate_is_reference", truth(rate_is_reference))
            .field("brace_spans", truth(brace_spans))
            .field("front_doors_agree", truth(front_doors_agree))
            .field("display_style", truth(display_style))
            .field(
                "readme_formula",
                format!("{readme_width:.4}x{readme_height:.4}"),
            )
            .field("readme_envelope", truth(readme_envelope)),
    );
    if !(background_is_reference
        && rate_is_reference
        && frames_match
        && brace_spans
        && front_doors_agree
        && display_style
        && readme_envelope)
    {
        return Err(fail(format!(
            "portal Reference defaults drifted: header={header:?} frames={frames} \
             receipt_fps={} receipt_frames={} yuv=({y},{u},{v}) brace={} target={} \
             portal_camera={:?} native_camera=({:?}, {}, {}, {}) \
             display_fraction_height={} readme_formula={:?}",
            report.fps,
            report.frame_count,
            report.brace_width,
            report.target_width,
            report.scene_camera,
            native.resolution,
            native.fps,
            native.background_color,
            native.background_opacity,
            report.display_fraction_height,
            report.readme_formula
        )));
    }
    Ok(RunOutcome::ok()
        .with_artifact("python_portal_defaults.y4m", bytes)
        .with_counter("python_defaults_reference_background", 1)
        .with_counter("python_defaults_reference_rate", 1)
        .with_counter("python_defaults_brace_spans_composite", 1)
        .with_counter("python_defaults_front_doors_agree", 1)
        .with_counter("python_defaults_display_style_tex", 1)
        .with_counter("python_defaults_readme_formula_envelope", 1))
}

fn python_portal_png_still_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("python_portal_png_still")?;
    let destination = root.join("final.png");
    let report = manimlib::run_portal_gauntlet_png_still(&destination, ctx.seed)
        .map_err(|error| fail(format!("run Python portal PNG still scenario: {error}")))?;
    let bytes = std::fs::read(&destination)
        .map_err(|error| fail(format!("read {}: {error}", destination.display())))?;
    let signature = bytes.starts_with(b"\x89PNG\r\n\x1a\n");
    let decoded = fmn_codec::decode_png(
        &bytes,
        &fmn_codec::PngLimits {
            max_pixels: 96 * 54,
            ..fmn_codec::PngLimits::default()
        },
    )
    .map_err(|error| fail(format!("decode Python portal still PNG: {error}")))?;
    let dimensions_match = decoded.width == 96 && decoded.height == 54;
    let engine_journaled = {
        let fields: Vec<_> = report.engine.split(':').collect();
        fields.len() == 3 && fields[0] == "fast-cpu" && fields[2].parse::<u32>().is_ok()
    };

    // Planted negative: a directory at the requested file path is the wrong
    // node kind and must survive a precise publication refusal.
    let blocked_destination = root.join("blocked.png");
    std::fs::create_dir(&blocked_destination).map_err(|error| {
        fail(format!(
            "create planted PNG destination directory {}: {error}",
            blocked_destination.display()
        ))
    })?;
    let blocked = manimlib::run_portal_gauntlet_png_still(&blocked_destination, ctx.seed);
    let wrong_node_kind_refused = blocked.is_err() && blocked_destination.is_dir();
    let complete = report.path == destination
        && report.frame_count == 1
        && report.bytes == u64::try_from(bytes.len()).unwrap_or(u64::MAX)
        && report.digest.len() == 64
        && report.threads == 1
        && signature
        && dimensions_match
        && engine_journaled
        && wrong_node_kind_refused;
    ctx.event(
        LogEvent::new("e2e.python.render")
            .field("scene", "_GauntletPortalScene")
            .field("format", "png")
            .field("complete", truth(complete))
            .field("png_signature", truth(signature))
            .field("dimensions_match", truth(dimensions_match))
            .field("engine_journaled", truth(engine_journaled))
            .field("wrong_node_kind_refused", truth(wrong_node_kind_refused)),
    );
    if !complete {
        return Err(fail(format!(
            "Python portal still drifted: path={} frames={} bytes={} file_bytes={} \
             digest_len={} engine={:?} threads={} signature={} dimensions={}x{} \
             wrong_node_kind_refused={}",
            report.path.display(),
            report.frame_count,
            report.bytes,
            bytes.len(),
            report.digest.len(),
            report.engine,
            report.threads,
            signature,
            decoded.width,
            decoded.height,
            wrong_node_kind_refused,
        )));
    }
    Ok(RunOutcome::ok()
        .with_artifact("python_portal_still.png", bytes)
        .with_counter("python_still_frames", 1)
        .with_counter("python_still_png_signature", 1)
        .with_counter("python_still_engine_journaled", 1)
        .with_counter("python_still_wrong_node_kind_refused", 1))
}

/// The shipped CLI's first positive native-registration path: one program
/// selected from `@builtin`, rendered by Lumen, converted by fmn-frame, and
/// atomically published by Reel's ordered PNG-sequence sink.
fn cli_builtin_render_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_builtin_render")?;
    let dir_text = dir
        .to_str()
        .ok_or_else(|| fail("CLI scenario output path is not UTF-8"))?;
    let output = fmn_cli::run([
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        dir_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let sequence = dir.join("circle_shift.v1");
    let complete = sequence.join("FMN_COMPLETE").is_file();
    let pngs = std::fs::read_dir(&sequence)
        .map_err(|error| fail(format!("list CLI PNG sequence: {error}")))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .count();
    let signature = std::fs::read(sequence.join("frame_000000.png"))
        .map(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
        .unwrap_or(false);
    let manifest_generation = dir.join("circle_shift.v1.manifest");
    let manifest = std::fs::read(manifest_generation.join("manifest.fmnp"))
        .ok()
        .and_then(|bytes| ProvenanceManifest::from_bytes(&bytes).ok());
    let manifest_complete = manifest_generation.join("FMN_COMPLETE").is_file();
    let manifest_valid = manifest.as_ref().is_some_and(|manifest| {
        manifest.mode == ManifestMode::Standard
            && manifest.outputs.len() == 1
            && (1..=10).all(|id| manifest.items.iter().any(|item| item.item_id == id))
    });
    let robot_record = output.stdout.lines().count() == 1
        && output.stdout.contains("\"kind\":\"render\"")
        && output.stdout.contains("\"source\":\"builtin\"")
        && output.stdout.contains("\"frames\":3")
        && output.stdout.contains("circle_shift.v1.manifest");
    ctx.event(
        LogEvent::new("e2e.cli.render")
            .field("source", "builtin")
            .field("format", "png_sequence")
            .field("complete", truth(complete))
            .field("png_signature", truth(signature))
            .field("manifest_complete", truth(manifest_complete))
            .field("manifest_valid", truth(manifest_valid))
            .field("robot_record", truth(robot_record)),
    );
    ctx.counter("cli_render_frames", pngs as u64);
    ctx.counter("cli_png_signature", u64::from(signature));
    ctx.counter(
        "cli_manifest",
        u64::from(manifest_complete && manifest_valid),
    );
    if output.code != 0
        || !output.stderr.is_empty()
        || !complete
        || pngs != 3
        || !signature
        || !manifest_complete
        || !manifest_valid
        || !robot_record
    {
        return Err(fail(format!(
            "native CLI render failed: code={} pngs={pngs} complete={complete} signature={signature} manifest_complete={manifest_complete} manifest_valid={manifest_valid} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_render_frames", 3)
        .with_counter("cli_png_signature", 1)
        .with_counter("cli_manifest", 1))
}

/// The shared CLI/facade audio composer consumes real scene lifecycle events
/// and sound requests. Native PCM is the oracle here; the CLI's process-
/// capability tests separately exercise delivery to the two-stage video mux.
fn scene_soundtrack_windows_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn::rendering::{PreparedSceneSoundtrack, SoundtrackTimeline};
    use fmn_output::{AudioDecodeLimits, AudioDecoder, MixerConfig, frames_to_samples};
    struct Score;
    impl fmn::SceneConstruct for Score {
        fn name(&self) -> &str {
            "scene_soundtrack_windows"
        }
        fn construct(&mut self, stage: &mut fmn::Stage<'_>) -> fmn::Result<()> {
            stage.scene_mut().force_skipping();
            stage
                .scene_mut()
                .add_sound("omitted.wav", 0.0, None, None)?;
            stage.wait(0.125)?;
            stage.scene_mut().revert_to_original_skipping_status();
            stage
                .scene_mut()
                .add_sound("background.wav", 0.0, None, None)?;
            stage.wait(0.25)?;
            stage
                .scene_mut()
                .add_sound("overlay.wav", -2.0 / 59.0, Some(-3.0), Some(-6.0))?;
            stage.wait(0.5)?;
            Ok(())
        }
    }
    #[derive(Default)]
    struct ClockSink {
        timeline: SoundtrackTimeline,
        captured: u64,
    }
    impl fmn_scene::SceneSink for ClockSink {
        fn event(
            &mut self,
            event: fmn_scene::LifecycleEvent,
        ) -> Result<(), fmn_scene::IntegrationError> {
            self.timeline
                .observe(event.time.frames(), event.skipping)
                .map_err(|error| fmn_scene::IntegrationError::new("soundtrack", error.to_string()))
        }
        fn capture(
            &mut self,
            _: fmn_scene::CaptureReason,
            _: FramePacket,
        ) -> Result<(), fmn_scene::IntegrationError> {
            self.captured += 1;
            Ok(())
        }
    }
    let mut sink = ClockSink::default();
    let scene = fmn::run_scene(
        &mut Score,
        fmn_scene::RuntimeConfig {
            fps: 59,
            ..fmn_scene::RuntimeConfig::default()
        },
        7,
        &mut sink,
    )
    .map_err(|error| fail(format!("sound scene lifecycle: {error}")))?
    .into_scene();
    let requests = scene.sound_requests();
    let rebased_end = sink
        .timeline
        .output_frame(scene.time().frames())
        .map_err(|error| fail(error.to_string()))?;
    if requests.len() != 2
        || scene.time().frames() != 53
        || rebased_end != 45
        || sink.captured != 45
    {
        return Err(fail(
            "sound scene lost skip/no-op or rational frame semantics",
        ));
    }
    let fs = fmn_platform::fs::VirtualFs::new();
    let background: Vec<f32> = (0..16_000)
        .map(|index| (index % 97) as f32 / 194.0 - 0.25)
        .collect();
    let background = fmn_codec::encode_wav(1, 44_100, fmn_codec::SampleFormat::S16, &background);
    fs.insert("background.wav", background.clone());
    fs.insert(
        "overlay.wav",
        fmn_codec::encode_wav(1, 48_000, fmn_codec::SampleFormat::S16, &vec![0.25; 8_000]),
    );
    let mut decoder = AudioDecoder::new(AudioDecodeLimits::default(), None)
        .map_err(|error| fail(error.to_string()))?;
    let end = frames_to_samples(53, 59, 48_000).map_err(|error| fail(error.to_string()))? as u64;
    let prepared =
        PreparedSceneSoundtrack::new(&fs, requests, &mut decoder, MixerConfig::default(), None)
            .map_err(|error| fail(error.to_string()))?
            .with_timeline_frames(end);
    let full = prepared.mix(1).map_err(|error| fail(error.to_string()))?;
    let start = frames_to_samples(8, 59, 48_000).map_err(|error| fail(error.to_string()))? as u64;
    let split = frames_to_samples(23, 59, 48_000).map_err(|error| fail(error.to_string()))? as u64;
    let left = prepared
        .mix_window(start, split - start, 4)
        .map_err(|error| fail(error.to_string()))?;
    let right = prepared
        .mix_window(split, end - split, 1)
        .map_err(|error| fail(error.to_string()))?;
    let mut joined = left.audio.samples;
    joined.extend(right.audio.samples);
    if joined != full.audio.samples[start as usize * 2..end as usize * 2]
        || prepared.inputs()[0].report.source_digest != sha256(&background)
        || !prepared.invocations().is_empty()
    {
        return Err(fail(
            "soundtrack windows changed phase, ducking, source identity, or native decode",
        ));
    }
    let rebased = PreparedSceneSoundtrack::new(
        &fs,
        requests,
        &mut decoder,
        MixerConfig::default(),
        Some(&sink.timeline),
    )
    .map_err(|error| fail(error.to_string()))?
    .mix(4)
    .map_err(|error| fail(error.to_string()))?;
    if full.audio.samples[..start as usize * 2]
        .iter()
        .any(|sample| *sample != 0.0)
        || !rebased.audio.samples[..100]
            .iter()
            .any(|sample| sample.abs() > 0.01)
    {
        return Err(fail("skipped prefix was not removed from emitted audio"));
    }
    ctx.event(
        LogEvent::new("e2e.soundtrack.windows")
            .field("cues", 2_u64)
            .field("captured_frames", sink.captured)
            .field("sample_window_equal", "true")
            .field("skipped_prefix_removed", "true")
            .field("source_identity", "true"),
    );
    Ok(RunOutcome::ok()
        .with_counter("soundtrack_cues", 2)
        .with_counter("soundtrack_windows_equal", 1)
        .with_counter("soundtrack_skip_rebased", 1)
        .with_counter("soundtrack_source_identity", 1))
}

/// One certified in-process render of the CLI's `formula_sheet.v1` builtin
/// against `cache`, returning the render record and its frames in order.
fn formula_sheet_render(
    scenario: &str,
    cache: &std::path::Path,
    threads: &str,
) -> Result<(String, Vec<Vec<u8>>), ScenarioError> {
    let dir = scenario_dir(scenario)?;
    let dir_text = dir
        .to_str()
        .ok_or_else(|| fail("typeset scenario output path is not UTF-8"))?;
    let cache_text = cache
        .to_str()
        .ok_or_else(|| fail("typeset scenario cache path is not UTF-8"))?;
    let output = fmn_cli::run([
        "--robot",
        "--reproducible",
        "--format",
        "png_sequence",
        "--resolution",
        "160x90",
        "--fps",
        "8",
        "--threads",
        threads,
        "--cache-dir",
        cache_text,
        "--video_dir",
        dir_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        fmn::builtins::FORMULA_SHEET_SCENE_NAME,
    ]);
    if output.code != 0 {
        return Err(fail(format!(
            "formula sheet render failed: code={} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    let record = output
        .stdout
        .lines()
        .find(|line| line.contains("\"kind\":\"render\""))
        .ok_or_else(|| fail(format!("no render record in {:?}", output.stdout)))?
        .to_owned();
    let sequence = dir.join(fmn::builtins::FORMULA_SHEET_SCENE_NAME);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&sequence)
        .map_err(|error| fail(format!("list {}: {error}", sequence.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect();
    paths.sort();
    let frames = paths
        .iter()
        .map(|path| {
            std::fs::read(path).map_err(|error| fail(format!("read {}: {error}", path.display())))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((record, frames))
}

/// One unsigned field of a render record's `typesetting` object.
fn typesetting_field(record: &str, field: &str) -> Result<u64, ScenarioError> {
    let typesetting = record
        .split_once("\"typesetting\":{")
        .ok_or_else(|| fail(format!("render record has no typesetting object: {record}")))?
        .1;
    let prefix = format!("\"{field}\":");
    let tail = typesetting
        .split_once(&prefix)
        .ok_or_else(|| fail(format!("typesetting.{field} missing: {typesetting}")))?
        .1;
    let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
    tail[..digits]
        .parse()
        .map_err(|_| fail(format!("typesetting.{field} is not a count: {typesetting}")))
}

/// The formula sheet's twenty formulas plus the scale-calibration probe.
const FORMULA_SHEET_LAYOUTS: u64 = 21;

/// fm-typeset-cache-preflight-wiring-1sn0: the CLI renders a Tex-heavy
/// builtin twice against one persistent typeset cache. The first run lays out
/// all twenty formulas and publishes them; the second, a fresh session that
/// shares nothing but the store, serves every one from disk, lays out none,
/// and publishes byte-identical certified frames.
fn typeset_cache_warm_second_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let cache = scenario_dir("typeset_cache_root")?.join("cache");
    let (cold, cold_frames) = formula_sheet_render("typeset_cache_cold", &cache, "2")?;
    let (warm, warm_frames) = formula_sheet_render("typeset_cache_warm", &cache, "2")?;
    let mut counts = Vec::new();
    for (run, record) in [("cold", &cold), ("warm", &warm)] {
        let row = [
            typesetting_field(record, "disk_hits")?,
            typesetting_field(record, "misses")?,
            typesetting_field(record, "bytes_read")?,
            typesetting_field(record, "bytes_written")?,
        ];
        ctx.event(
            LogEvent::new("e2e.typeset.cache")
                .field("run", run)
                .field("disk_hits", row[0])
                .field("misses", row[1])
                .field("bytes_read", row[2])
                .field("bytes_written", row[3])
                .field("persistent", truth(record.contains("\"persistent\":true"))),
        );
        counts.push(row);
    }
    let identical = !cold_frames.is_empty() && cold_frames == warm_frames;
    ctx.event(
        LogEvent::new("e2e.typeset.cache_bits")
            .field("frames", cold_frames.len() as u64)
            .field("hit_equals_miss", truth(identical)),
    );
    let [cold_hits, cold_misses, _, written] = counts[0];
    let [warm_hits, warm_misses, read, warm_written] = counts[1];
    if cold_hits != 0
        || cold_misses != FORMULA_SHEET_LAYOUTS
        || warm_hits != FORMULA_SHEET_LAYOUTS
        || warm_misses != 0
        || read != written
        || warm_written != 0
        || !identical
    {
        return Err(fail(format!(
            "warm second run broke: cold={:?} warm={:?} identical={identical}",
            counts[0], counts[1]
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("typeset_cold_misses", cold_misses)
        .with_counter("typeset_warm_disk_hits", warm_hits)
        .with_counter("typeset_warm_misses", warm_misses)
        .with_counter("typeset_hit_equals_miss", 1))
}

/// fm-typeset-cache-preflight-wiring-1sn0: the formula sheet's twenty static
/// formulas are typeset by the front door's preflight on several workers
/// before the first frame, and no layout happens inside a play.
fn typeset_preflight_before_first_play(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let cache = scenario_dir("typeset_preflight_root")?.join("cache");
    let (record, frames) = formula_sheet_render("typeset_preflight", &cache, "4")?;
    let requests = typesetting_field(&record, "requests")?;
    let workers = typesetting_field(&record, "workers")?;
    let active = typesetting_field(&record, "active_workers")?;
    let preflighted = typesetting_field(&record, "layouts_before_construct")?;
    let before = typesetting_field(&record, "layouts_before_first_frame")?;
    let inside = typesetting_field(&record, "layouts_inside_play")?;
    let layouts = typesetting_field(&record, "misses")?;
    let wall_ns = typesetting_field(&record, "wall_ns")?;
    ctx.event(
        LogEvent::new(e2e::spans::PREFLIGHT)
            .field("requests", requests)
            .field("workers", workers)
            .field("active_workers", active)
            .field("wall_ns", wall_ns)
            .field("layouts_before_construct", preflighted)
            .field("layouts_in_construct", before.saturating_sub(preflighted))
            .field("layouts_before_first_frame", before)
            .field("layouts_inside_play", inside)
            .field("before_first_frame", truth(before == layouts)),
    );
    ctx.counter(e2e::counters::TYPESETS, layouts);
    let parallel = std::thread::available_parallelism().map_or(1, usize::from) < 2
        || (workers >= 2 && active >= 2);
    if requests != 20
        || layouts != FORMULA_SHEET_LAYOUTS
        || before != layouts
        || preflighted != layouts
        || inside != 0
        || !parallel
        || frames.is_empty()
    {
        return Err(fail(format!(
            "preflight contract broke: requests={requests} workers={workers} active={active} \
             layouts={layouts} before_construct={preflighted} before_first_frame={before} \
             inside_play={inside}"
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("typeset_preflight_requests", requests)
        .with_counter("typeset_layouts_inside_play", inside)
        .with_counter("preflight_before_first_frame", 1))
}

/// The CLI's camera builtins (fm-cli-camera-route-i1zc): a lit surface,
/// depth-sorted dots, a sampled image, and 2D drawn over 3D, each through the
/// fixed camera.
const CLI_CAMERA_BUILTINS: [&str; 4] = [
    "surface_cube.v1",
    "dot_cloud_depth.v1",
    "image_quad.v1",
    "mixed_camera.v1",
];

/// One builtin through the in-process CLI: `--reproducible`, a 64x36 PNG
/// sequence at 8 fps on `threads` threads. Returns the CLI's output and the
/// frames in order.
fn cli_builtin_frames(
    name: &str,
    threads: &str,
) -> Result<(fmn_cli::RunOutput, Vec<Vec<u8>>), ScenarioError> {
    let dir = scenario_dir(&format!("cli_builtin_{threads}"))?;
    let dir_text = dir
        .to_str()
        .ok_or_else(|| fail("CLI scenario output path is not UTF-8"))?;
    let output = fmn_cli::run([
        "--robot",
        "--reproducible",
        "--format",
        "png_sequence",
        "--resolution",
        "64x36",
        "--fps",
        "8",
        "--threads",
        threads,
        "--video_dir",
        dir_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        name,
    ]);
    let sequence = dir.join(name);
    let mut paths: Vec<_> = std::fs::read_dir(&sequence)
        .map_err(|error| fail(format!("list {}: {error}", sequence.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect();
    paths.sort();
    let frames = paths
        .iter()
        .map(|path| {
            std::fs::read(path).map_err(|error| fail(format!("read {}: {error}", path.display())))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((output, frames))
}

/// fm-cli-camera-route-i1zc: the CLI's 3D builtins render through the camera
/// route under `--reproducible`, animate visibly, and lock their certified
/// frames in the e2e lock. Before this scenario no lock held the CLI's 3D
/// output, so a camera-route regression could pass every gate.
fn cli_camera_builtins_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let mut outcome = RunOutcome::ok();
    let mut frames_total = 0_u64;
    for name in CLI_CAMERA_BUILTINS {
        let (output, frames) = cli_builtin_frames(name, "1")?;
        let camera_route = output.stdout.contains("\"route\":\"camera-cpu\"");
        let animated = frames.windows(2).any(|pair| pair[0] != pair[1]);
        ctx.event(
            LogEvent::new("e2e.cli.camera")
                .field("builtin", name)
                .field("frames", frames.len() as u64)
                .field("camera_route", truth(camera_route))
                .field("animated", truth(animated))
                .field(
                    "first_frame_sha256",
                    frames
                        .first()
                        .map_or_else(String::new, |frame| fmn_hash::sha256(frame).to_hex()),
                ),
        );
        if output.code != 0 || !camera_route || !animated {
            return Err(fail(format!(
                "CLI camera builtin {name}: code={} frames={} camera_route={camera_route} \
                 animated={animated} stdout={:?} stderr={:?}",
                output.code,
                frames.len(),
                output.stdout,
                output.stderr
            )));
        }
        frames_total += frames.len() as u64;
        let stem = name.trim_end_matches(".v1");
        for (index, frame) in frames.into_iter().enumerate() {
            outcome = outcome.with_artifact(
                &format!("render_matrix.cli_camera.{stem}.v1.{index:04}"),
                frame,
            );
        }
    }
    Ok(outcome
        .with_counter("cli_camera_builtins", CLI_CAMERA_BUILTINS.len() as u64)
        .with_counter("cli_camera_frames", frames_total))
}

/// fm-facade-render-api-9ewt: the CLI's 25 primitive builtins render on the
/// retained 2D route under `--reproducible` and lock their certified frames
/// in the e2e lock. This is the before/after proof the sink extraction needs:
/// no other lock holds the shipping CLI's 2D output (the scene corpus and the
/// render matrix bypass the CLI).
fn cli_primitive_builtins_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let mut outcome = RunOutcome::ok();
    let mut frames_total = 0_u64;
    for name in fmn::builtins::PRIMITIVE_SCENE_NAMES {
        let (output, frames) = cli_builtin_frames(name, "1")?;
        let retained_route = output.stdout.contains("\"route\":\"retained-cpu\"");
        ctx.event(
            LogEvent::new("e2e.cli.primitive")
                .field("builtin", name)
                .field("frames", frames.len() as u64)
                .field("retained_route", truth(retained_route))
                .field(
                    "first_frame_sha256",
                    frames
                        .first()
                        .map_or_else(String::new, |frame| fmn_hash::sha256(frame).to_hex()),
                ),
        );
        if output.code != 0 || !retained_route || frames.is_empty() {
            return Err(fail(format!(
                "CLI primitive builtin {name}: code={} frames={} retained_route={retained_route} \
                 stdout={:?} stderr={:?}",
                output.code,
                frames.len(),
                output.stdout,
                output.stderr
            )));
        }
        frames_total += frames.len() as u64;
        let stem = name.trim_end_matches(".v1");
        for (index, frame) in frames.into_iter().enumerate() {
            outcome = outcome.with_artifact(
                &format!("render_matrix.cli_primitive.{stem}.v1.{index:04}"),
                frame,
            );
        }
    }
    Ok(outcome
        .with_counter(
            "cli_primitive_builtins",
            fmn::builtins::PRIMITIVE_SCENE_NAMES.len() as u64,
        )
        .with_counter("cli_primitive_frames", frames_total))
}

/// Every builtin the CLI ships: the 25 primitive programs and the camera
/// programs.
fn cli_all_builtins() -> impl Iterator<Item = &'static str> {
    fmn::builtins::PRIMITIVE_SCENE_NAMES
        .into_iter()
        .chain(CLI_CAMERA_BUILTINS)
}

/// The CLI builtins' determinism drill (fm-cli-camera-route-i1zc,
/// fm-facade-render-api-9ewt): every builtin's certified PNG sequence is
/// byte-identical at 1 and 4 render threads.
fn cli_builtins_threads_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let mut identical = 0_u64;
    for name in cli_all_builtins() {
        let (one, serial) = cli_builtin_frames(name, "1")?;
        let (four, parallel) = cli_builtin_frames(name, "4")?;
        let same = one.code == 0 && four.code == 0 && !serial.is_empty() && serial == parallel;
        ctx.event(
            LogEvent::new("e2e.cli.builtin_threads")
                .field("builtin", name)
                .field("frames", serial.len() as u64)
                .field("threads_1_4_identical", truth(same)),
        );
        if !same {
            return Err(fail(format!(
                "CLI builtin {name} differs between 1 and 4 threads: codes {}/{} frames {}/{}",
                one.code,
                four.code,
                serial.len(),
                parallel.len()
            )));
        }
        identical += 1;
    }
    Ok(RunOutcome::ok().with_counter("cli_builtins_threads_identical", identical))
}

/// Count the ordinary scene schedule without touching Reel, then drive each
/// authored segment through its own real native sink.
fn cli_prerun_subdivide_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_prerun_subdivide")?;
    let dir_text = dir
        .to_str()
        .ok_or_else(|| fail("CLI scenario output path is not UTF-8"))?;
    let output = fmn_cli::run([
        "--robot",
        "--prerun",
        "--subdivide",
        "--format",
        "y4m",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--video_dir",
        dir_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        "circle_shift.v1",
    ]);
    let records = output.stdout.lines().collect::<Vec<_>>();
    let prerun_record = records.first().is_some_and(|record| {
        record.contains("\"kind\":\"prerun\"")
            && record.contains("\"frames\":3")
            && record.contains("\"segments\":2")
    });
    let render_records = records.len() == 3
        && records.get(1).is_some_and(|record| {
            record.contains("\"subdivision\":0") && record.contains("\"frames\":2")
        })
        && records.get(2).is_some_and(|record| {
            record.contains("\"subdivision\":1") && record.contains("\"frames\":1")
        });
    let root = dir.join("circle_shift.v1");
    let mut rendered_frames = 0_usize;
    let mut complete = true;
    for index in 0..2 {
        let artifact = root.join(format!("{index:05}.y4m"));
        let bytes = std::fs::read(&artifact).unwrap_or_default();
        rendered_frames += bytes
            .windows(6)
            .filter(|window| *window == b"FRAME\n")
            .count();
        complete &= artifact
            .with_file_name(format!("{index:05}.y4m.manifest"))
            .join("FMN_COMPLETE")
            .is_file();
    }
    ctx.event(
        LogEvent::new("e2e.cli.prerun_subdivide")
            .field("prerun_record", truth(prerun_record))
            .field("render_records", truth(render_records))
            .field("complete", truth(complete))
            .field("rendered_frames", rendered_frames as u64),
    );
    ctx.counter("cli_prerun_frames", u64::from(prerun_record) * 3);
    ctx.counter("cli_subdivision_segments", u64::from(render_records) * 2);
    ctx.counter("cli_subdivision_frames", rendered_frames as u64);
    if output.code != 0
        || !output.stderr.is_empty()
        || !prerun_record
        || !render_records
        || !complete
        || rendered_frames != 3
    {
        return Err(fail(format!(
            "CLI prerun/subdivision failed: code={} rendered_frames={rendered_frames} complete={complete} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_prerun_frames", 3)
        .with_counter("cli_subdivision_segments", 2)
        .with_counter("cli_subdivision_frames", 3))
}

/// The batch front door over two real native registrations. Asupersync may
/// complete scene jobs in either order; the CLI contract reports them in the
/// request order and publishes each sequence atomically.
fn cli_batch_render_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_batch_render")?;
    let artifact_dir = dir.join("artifacts");
    let manifest_dir = dir.join("manifests");
    std::fs::create_dir(&artifact_dir)
        .map_err(|error| fail(format!("create batch artifact directory: {error}")))?;
    std::fs::create_dir(&manifest_dir)
        .map_err(|error| fail(format!("create batch manifest directory: {error}")))?;
    let artifact_text = artifact_dir
        .to_str()
        .ok_or_else(|| fail("CLI batch output path is not UTF-8"))?;
    let manifest_text = manifest_dir
        .to_str()
        .ok_or_else(|| fail("CLI batch manifest path is not UTF-8"))?;
    let scenes = ["circle_shift.v1", "rectangle_shift.v1"];
    let output = fmn_cli::run([
        "batch",
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "8",
        "--threads",
        "1",
        "--max-scenes",
        "2",
        "--manifest-dir",
        manifest_text,
        "--video_dir",
        artifact_text,
        fmn_cli::BUILTIN_SCENE_SOURCE,
        scenes[0],
        scenes[1],
    ]);
    let mut frames = 0_usize;
    let mut complete = true;
    let mut signatures = true;
    let mut manifests = true;
    for scene in scenes {
        let sequence = artifact_dir.join(scene);
        complete &= sequence.join("FMN_COMPLETE").is_file();
        frames = frames.saturating_add(
            std::fs::read_dir(&sequence)
                .map_err(|error| fail(format!("list batch sequence {scene}: {error}")))?
                .filter_map(Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
                .count(),
        );
        signatures &= std::fs::read(sequence.join("frame_000000.png"))
            .map(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
            .unwrap_or(false);
        let generation = manifest_dir.join(scene);
        manifests &= generation.join("FMN_COMPLETE").is_file()
            && std::fs::read(generation.join("manifest.fmnp"))
                .ok()
                .and_then(|bytes| ProvenanceManifest::from_bytes(&bytes).ok())
                .is_some_and(|manifest| {
                    manifest.outputs.len() == 1
                        && (1..=10).all(|id| manifest.items.iter().any(|item| item.item_id == id))
                });
    }
    let lines: Vec<&str> = output.stdout.lines().collect();
    let ordered_records = lines.len() == 3
        && lines[0].contains("\"scene\":\"circle_shift.v1\"")
        && lines[1].contains("\"scene\":\"rectangle_shift.v1\"")
        && lines[2].contains("\"kind\":\"batch\"")
        && lines[2].contains("\"status\":\"ok\"")
        && lines[2].contains("\"succeeded\":2");
    ctx.event(
        LogEvent::new("e2e.cli.batch")
            .field("source", "builtin")
            .field("format", "png_sequence")
            .field("complete", truth(complete))
            .field("png_signatures", truth(signatures))
            .field("manifests", truth(manifests))
            .field("ordered_records", truth(ordered_records)),
    );
    ctx.counter("cli_batch_scenes", 2);
    ctx.counter("cli_batch_frames", frames as u64);
    if output.code != 0
        || !output.stderr.is_empty()
        || !complete
        || frames != 6
        || !signatures
        || !manifests
        || !ordered_records
    {
        return Err(fail(format!(
            "native CLI batch failed: code={} frames={frames} complete={complete} signatures={signatures} manifests={manifests} ordered={ordered_records} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_batch_scenes", 2)
        .with_counter("cli_batch_frames", 6))
}

/// Stock-CLI proof for the two native RGBA artifact adapters that share
/// Lumen's final conversion but have distinct publication semantics: PNG
/// captures exactly the completed scene state, while GIF streams the full
/// schedule without crossing the ffmpeg boundary.
fn cli_native_png_gif_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_native_png_gif")?;
    let png_dir = dir.join("png");
    let gif_dir = dir.join("gif");
    std::fs::create_dir(&png_dir)
        .map_err(|error| fail(format!("create CLI PNG directory: {error}")))?;
    std::fs::create_dir(&gif_dir)
        .map_err(|error| fail(format!("create CLI GIF directory: {error}")))?;
    let png_dir_text = png_dir
        .to_str()
        .ok_or_else(|| fail("CLI PNG output path is not UTF-8"))?;
    let gif_dir_text = gif_dir
        .to_str()
        .ok_or_else(|| fail("CLI GIF output path is not UTF-8"))?;

    let render = |format: &str, output: &str| {
        fmn_cli::run([
            "--robot",
            "--format",
            format,
            "--resolution",
            "96x54",
            "--fps",
            "8",
            "--threads",
            "1",
            "--video_dir",
            output,
            fmn_cli::BUILTIN_SCENE_SOURCE,
            "circle_shift.v1",
        ])
    };
    let png = render("png", png_dir_text);
    let gif = render("gif", gif_dir_text);
    let png_signature = std::fs::read(png_dir.join("circle_shift.png"))
        .map(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
        .unwrap_or(false);
    let gif_signature = std::fs::read(gif_dir.join("circle_shift.gif"))
        .map(|bytes| bytes.starts_with(b"GIF89a") && bytes.last() == Some(&b';'))
        .unwrap_or(false);
    let robot_records = png.stdout.lines().count() == 1
        && png.stdout.contains("\"format\":\"png\"")
        && png.stdout.contains("\"frames\":1")
        && gif.stdout.lines().count() == 1
        && gif.stdout.contains("\"format\":\"gif\"")
        && gif.stdout.contains("\"frames\":3");
    ctx.event(
        LogEvent::new("e2e.cli.native_rgba_artifacts")
            .field("png_signature", truth(png_signature))
            .field("gif_signature", truth(gif_signature))
            .field("robot_records", truth(robot_records)),
    );
    ctx.counter("cli_png_still_frames", u64::from(png_signature));
    ctx.counter(
        "cli_gif_stream_frames",
        if gif_signature && gif.stdout.contains("\"frames\":3") {
            3
        } else {
            0
        },
    );
    if png.code != 0
        || gif.code != 0
        || !png.stderr.is_empty()
        || !gif.stderr.is_empty()
        || !png_signature
        || !gif_signature
        || !robot_records
    {
        return Err(fail(format!(
            "native PNG/GIF CLI path failed: png=({},{:?},{:?}) gif=({},{:?},{:?})",
            png.code, png.stdout, png.stderr, gif.code, gif.stdout, gif.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_png_still_frames", 1)
        .with_counter("cli_gif_stream_frames", 3))
}

/// A compiled FMTL/1 artifact crosses the stock CLI boundary, reconstructs
/// through fmn-scene's shared reader, renders through Lumen, and publishes
/// through Reel. This is the source-unedited native-artifact path rather than
/// another built-in registration.
fn cli_compiled_bundle_render_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let dir = scenario_dir("cli_compiled_bundle_render")?;
    let source = dir.join("compiled_wait.fmtl");
    let output_dir = dir.join("output");
    std::fs::create_dir(&output_dir)
        .map_err(|error| fail(format!("create compiled output directory: {error}")))?;
    let mut timeline =
        Timeline::new(8).map_err(|error| fail(format!("create compiled timeline: {error}")))?;
    timeline
        .wait(0.25)
        .map_err(|error| fail(format!("author compiled wait: {error}")))?;
    let bytes =
        fmn_scene::export_timeline_bundle(timeline, &mut Stage::new(), &RngRoot::from_seed(0))
            .map_err(|error| fail(format!("export compiled timeline: {error}")))?;
    std::fs::write(&source, bytes)
        .map_err(|error| fail(format!("write compiled artifact: {error}")))?;
    let output = fmn_cli::run([
        "--robot",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--threads",
        "1",
        "--video_dir",
        output_dir
            .to_str()
            .ok_or_else(|| fail("compiled output path is not UTF-8"))?,
        source
            .to_str()
            .ok_or_else(|| fail("compiled source path is not UTF-8"))?,
        "CompiledWait",
    ]);
    let sequence = output_dir.join("CompiledWait");
    let complete = sequence.join("FMN_COMPLETE").is_file();
    let pngs = std::fs::read_dir(&sequence)
        .map_err(|error| fail(format!("list compiled CLI PNG sequence: {error}")))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "png"))
        .count();
    let signature = std::fs::read(sequence.join("frame_000000.png"))
        .map(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
        .unwrap_or(false);
    let robot_record = output.stdout.lines().count() == 1
        && output.stdout.contains("\"kind\":\"render\"")
        && output.stdout.contains("\"source\":\"compiled\"")
        && output.stdout.contains("\"source_artifact\":")
        && output.stdout.contains("\"frames\":2");
    ctx.event(
        LogEvent::new("e2e.cli.render")
            .field("source", "compiled")
            .field("format", "png_sequence")
            .field("complete", truth(complete))
            .field("png_signature", truth(signature))
            .field("robot_record", truth(robot_record)),
    );
    ctx.counter("cli_compiled_render_frames", pngs as u64);
    ctx.counter("cli_compiled_png_signature", u64::from(signature));
    if output.code != 0
        || !output.stderr.is_empty()
        || !complete
        || pngs != 2
        || !signature
        || !robot_record
    {
        return Err(fail(format!(
            "compiled CLI render failed: code={} pngs={pngs} complete={complete} signature={signature} stdout={:?} stderr={:?}",
            output.code, output.stdout, output.stderr
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("cli_compiled_render_frames", 2)
        .with_counter("cli_compiled_png_signature", 1))
}

// ---------------------------------------------------------------------------
// Lifecycle drill
// ---------------------------------------------------------------------------

/// The certified single-threaded frame of a stage, encoded into its
/// canonical document — the same renderer `tests/scene_goldens.rs`
/// injects into the corpus artifact form.
fn render_certified_doc(stage: &Stage) -> Vec<u8> {
    let config = scene_goldens::frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0)
        .expect("valid certified document fixture");
    let mono =
        MonoTable::build(&plan, config.map).expect("bounded certified document monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .expect("occlusion pruning matches the plan");
    let frame =
        FrameJob::with_identity(&plan, &mono, &binning, config, EngineIdentity::certified())
            .expect("frame job matches the plan")
            .render(1)
            .expect("the engine renders the frame");
    encode_frame(&frame).expect("the frame encodes into its canonical document")
}

/// Surface and vector geometry share the production camera-bound painter.
/// Keep the same viewport, scene scale, and certified identity as the 2D rows.
fn render_certified_camera_doc(stage: &Stage) -> Vec<u8> {
    use fmn_render::{
        Camera, CameraConfig, CameraFrame, RetainedFrameRenderer, RetainedFrameRendererConfig,
    };

    let config = scene_goldens::frame_config();
    let mut camera_frame = CameraFrame::default();
    camera_frame
        .set_shape([
            f64::from(config.viewport.width) / config.map.scale,
            f64::from(config.viewport.height) / config.map.scale,
        ])
        .expect("bounded camera fixture shape");
    let camera = Camera::new(CameraConfig {
        resolution: (config.viewport.width, config.viewport.height),
        background: config.background,
        frame: camera_frame,
        ..CameraConfig::default()
    })
    .expect("valid camera document fixture");
    let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
        frame: config,
        tiling: TILING,
        engine: EngineIdentity::certified(),
        threads: 1,
    })
    .expect("bounded camera document renderer");
    renderer
        .render_with_camera(stage, &camera)
        .expect("the camera route renders the surface and vector frame");
    encode_frame(renderer.frame()).expect("the camera frame encodes into its canonical document")
}

/// A graph generated after native plane placement must occupy that plane's
/// chart. The independent drawing oracle translates the original geometry;
/// the negative witness leaves the graph at the old origin.
fn coordinate_placement_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_library::CoordinateSystem;

    let book = fmn_text::FontBook::bundled()
        .map_err(|error| fail(format!("coordinate fonts: {error}")))?;
    let base = NumberPlane::new()
        .x_range([-1.0, 1.0, 1.0])
        .y_range([-1.0, 1.0, 1.0])
        .faded_line_ratio(1)
        .build(&book)
        .map_err(|error| fail(format!("construct coordinate plane: {error}")))?;
    let offset = [-2.0, 1.0, 0.0];
    let placed = base.clone().shifted(offset);
    let function = |x: f64| 0.5 * x;
    let original_graph = base
        .get_graph(function, None)
        .build()
        .map_err(|error| fail(format!("construct original graph: {error}")))?;
    let graph = placed
        .get_graph(function, None)
        .build()
        .map_err(|error| fail(format!("construct placed graph: {error}")))?;
    let near = |a: [f64; 3], b: [f64; 3]| a.into_iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-9);
    let endpoints = graph
        .points()
        .first()
        .is_some_and(|&point| near(point, [-3.0, 0.5, 0.0]))
        && graph
            .points()
            .last()
            .is_some_and(|&point| near(point, [-1.0, 1.5, 0.0]));
    let inverse = [[-1.0, 0.5], [0.0, 0.0], [2.0, -3.0]]
        .into_iter()
        .all(|point| near(placed.p2c(placed.c2p(&point)), [point[0], point[1], 0.0]));
    let drawing = |plane: VMobject, curve: VMobject| -> Result<Stage, ScenarioError> {
        let mut stage = Stage::new();
        let plane = stage.add(plane);
        let curve = stage.add(curve);
        stage
            .add_to_scene(plane)
            .map_err(|error| fail(error.to_string()))?;
        stage
            .add_to_scene(curve)
            .map_err(|error| fail(error.to_string()))?;
        Ok(stage)
    };
    let actual = drawing(placed.vmob().clone(), graph)?;
    let expected = drawing(
        base.vmob().clone().shifted(offset),
        original_graph.clone().shifted(offset),
    )?;
    let wrong = drawing(placed.vmob().clone(), original_graph)?;
    let pixels = render_certified_doc(&actual);
    let rendered_match = pixels == render_certified_doc(&expected);
    let rejects_unmoved_graph = pixels != render_certified_doc(&wrong);
    let visible = pixels != render_certified_doc(&Stage::new());
    ctx.event(
        LogEvent::new("e2e.render_matrix.coordinate_placement")
            .field("endpoints", truth(endpoints))
            .field("inverse", truth(inverse))
            .field("rendered_match", truth(rendered_match))
            .field("rejects_unmoved_graph", truth(rejects_unmoved_graph))
            .field("visible", truth(visible))
            .field("sha256", sha256(&pixels).to_hex()),
    );
    Ok(RunOutcome::ok()
        .with_counter("coordinate_endpoints", u64::from(endpoints))
        .with_counter("coordinate_inverse", u64::from(inverse))
        .with_counter("coordinate_rendered_match", u64::from(rendered_match))
        .with_counter(
            "coordinate_rejects_unmoved_graph",
            u64::from(rejects_unmoved_graph),
        )
        .with_counter("coordinate_visible", u64::from(visible)))
}

/// A live chart follows the real native Transform samples; graphs created
/// afterward use that moved map while a previously captured map stays frozen.
fn live_coordinate_animation_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_anim::{AnimConfig, Transform};
    use fmn_library::CoordinateSystem;

    let book = fmn_text::FontBook::bundled()
        .map_err(|error| fail(format!("live coordinate fonts: {error}")))?;
    let plane = NumberPlane::new()
        .x_range([-1.0, 1.0, 1.0])
        .y_range([-1.0, 1.0, 1.0])
        .faded_line_ratio(1)
        .build(&book)
        .map_err(|error| fail(format!("build live plane: {error}")))?;
    let mut stage = Stage::new();
    let root = stage.add(plane.vmob().clone());
    stage
        .add_to_scene(root)
        .map_err(|error| fail(format!("adopt live plane: {error}")))?;
    let live = plane
        .bind(&stage, root)
        .map_err(|error| fail(format!("bind live plane: {error}")))?;
    let frozen = live
        .snapshot(&stage)
        .map_err(|error| fail(format!("freeze initial chart: {error}")))?;
    let target = stage
        .copy_family(root)
        .map_err(|error| fail(format!("copy chart animation target: {error}")))?;
    let offset = [-2.0, 1.0, 0.0];
    stage.shift(target, offset);
    let mut timeline =
        Timeline::new(4).map_err(|error| fail(format!("chart animation clock: {error}")))?;
    timeline
        .play(vec![Box::new(Transform::new(root, target).with_config(
            AnimConfig {
                run_time: 0.5,
                rate_func: RateFunc::linear(),
                ..AnimConfig::default()
            },
        ))])
        .map_err(|error| fail(format!("schedule chart movement: {error}")))?;
    let mut sampled_origins = Vec::new();
    timeline
        .render(&mut stage, &RngRoot::from_seed(53), &mut |packet| {
            sampled_origins.push(live.c2p(&packet.materialize_stage(), &[0.0, 0.0]));
        })
        .map_err(|error| fail(format!("animate live chart: {error}")))?;
    let origins = sampled_origins
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| fail(format!("read chart animation sample: {error}")))?;
    let near = |a: [f64; 3], b: [f64; 3]| a.into_iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-6);
    let follows_frames =
        origins.len() == 2 && near(origins[0], [-1.0, 0.5, 0.0]) && near(origins[1], offset);
    let moved = live
        .snapshot(&stage)
        .map_err(|error| fail(format!("read moved chart: {error}")))?;
    let frozen_unchanged =
        near(frozen.c2p(&[0.0, 0.0]), [0.0; 3]) && near(moved.c2p(&[0.0, 0.0]), offset);
    let inverse = [[-1.0, 0.5], [0.0, 0.0], [2.0, -3.0]]
        .into_iter()
        .all(|coords| near(moved.p2c(moved.c2p(&coords)), [coords[0], coords[1], 0.0]));
    let graph = moved
        .get_graph(|x| 0.5 * x)
        .build()
        .map_err(|error| fail(format!("build graph after chart movement: {error}")))?;
    let endpoint = graph
        .points()
        .last()
        .is_some_and(|&point| near(point, [-1.0, 1.5, 0.0]));
    let graph = stage.add(graph);
    stage
        .add_to_scene(graph)
        .map_err(|error| fail(format!("adopt live graph: {error}")))?;

    let mut expected = Stage::new();
    let expected_plane = expected.add(plane.vmob().clone().shifted(offset));
    expected
        .add_to_scene(expected_plane)
        .map_err(|error| fail(format!("adopt expected plane: {error}")))?;
    let original_graph = plane
        .get_graph(|x| 0.5 * x, None)
        .build()
        .map_err(|error| fail(format!("build independent expected graph: {error}")))?;
    let expected_graph = expected.add(original_graph.shifted(offset));
    expected
        .add_to_scene(expected_graph)
        .map_err(|error| fail(format!("adopt expected graph: {error}")))?;
    let rendered = render_certified_doc(&stage);
    let rendered_match = rendered == render_certified_doc(&expected);
    let visible = rendered != render_certified_doc(&Stage::new());
    ctx.event(
        LogEvent::new("e2e.render_matrix.live_coordinate_animation")
            .field("frames", origins.len() as u64)
            .field("follows_frames", truth(follows_frames))
            .field("frozen_unchanged", truth(frozen_unchanged))
            .field("inverse", truth(inverse))
            .field("endpoint", truth(endpoint))
            .field("rendered_match", truth(rendered_match))
            .field("visible", truth(visible))
            .field("sha256", sha256(&rendered).to_hex()),
    );
    Ok(RunOutcome::ok()
        .with_counter("live_coordinate_frames", origins.len() as u64)
        .with_counter("live_coordinate_follows_frames", u64::from(follows_frames))
        .with_counter(
            "live_coordinate_frozen_unchanged",
            u64::from(frozen_unchanged),
        )
        .with_counter("live_coordinate_inverse", u64::from(inverse))
        .with_counter("live_coordinate_endpoint", u64::from(endpoint))
        .with_counter("live_coordinate_rendered_match", u64::from(rendered_match))
        .with_counter("live_coordinate_visible", u64::from(visible)))
}

/// Construct → snapshot → transform → snapshot: the scene_goldens
/// lifecycle form with geometry assertions (member/point counts move with
/// the lifecycle points, never silently empty).
fn lifecycle_construct_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let corpus = scene_goldens::corpus();
    let case = scene_goldens::scene_named("circle_tex_label.v1")
        .ok_or_else(|| fail("circle_tex_label.v1 is missing from the corpus"))?;
    let built = (case.build)(corpus);

    let count_geometry = |stage: &Stage| -> (u64, u64) {
        let mut members = 0_u64;
        let mut points = 0_u64;
        for &root in &built.roots {
            for member in stage.family(root) {
                members += 1;
                points += stage.get_points(member).unwrap_or_default().len() as u64;
            }
        }
        (members, points)
    };

    let (members_pre, points_pre) = count_geometry(&built.stage);
    let mut stage = built.stage;
    scene_goldens::apply_lifecycle_transform(&mut stage, &built.roots, 0);
    let (members_post, points_post) = count_geometry(&stage);

    ctx.event(
        LogEvent::new(e2e::spans::SCENE_CONSTRUCT)
            .field("scene", case.name)
            .field("members", members_pre)
            .field("points", points_pre),
    );
    ctx.event(
        LogEvent::new("e2e.lifecycle.transform")
            .field("members", members_post)
            .field("points", points_post),
    );
    ctx.counter("lifecycle_members", members_pre);
    ctx.counter("lifecycle_points_pre", points_pre);
    ctx.counter("lifecycle_points_post", points_post);

    if members_pre == 0 || points_pre == 0 || points_post == 0 {
        return Err(fail(
            "lifecycle geometry went empty — the snapshot would lock nothing",
        ));
    }

    // The locked artifact is the corpus lifecycle form: geometry
    // snapshots plus certified frames at both lifecycle points.
    let bytes = scene_goldens::artifact(case, corpus, 0, &render_certified_doc);
    Ok(RunOutcome::ok()
        .with_artifact("lifecycle.circle_tex_label.construct_transform.v1", bytes)
        .with_counter("lifecycle_members", members_pre)
        .with_counter("lifecycle_points_pre", points_pre)
        .with_counter("lifecycle_points_post", points_post))
}

/// Installing a rebuilt value preserves shared placements and retires private history.
fn lifecycle_install_shared_family_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut stage = Stage::new();
    let owner = stage.add(Mobject::from_points(&[[0.0; 3]]));
    let private = stage.add(Mobject::new());
    let private_leaf = stage.add(Mobject::from_points(&[[-2.0, -1.0, 0.0]]));
    let shared = stage.add(Mobject::new());
    let shared_leaf = stage.add(Mobject::from_points(&[[2.0, 1.0, 0.0]]));
    let rooted = stage.add(Mobject::from_points(&[[3.0, -1.0, 0.0]]));
    let outside = stage.add(Mobject::new());
    for (parent, child) in [
        (owner, private),
        (private, private_leaf),
        (private, rooted),
        (owner, shared),
        (shared, shared_leaf),
        (outside, shared),
    ] {
        stage
            .attach(parent, child)
            .map_err(|error| fail(format!("install source family attaches: {error}")))?;
    }
    // One admission preserves both shared and independently rooted placements.
    stage
        .add_many_to_scene(&[owner, outside, rooted])
        .map_err(|error| fail(format!("install source family enters scene: {error}")))?;
    let calls = Rc::new(Cell::new(0_u64));
    let updater_calls = Rc::clone(&calls);
    stage
        .add_updater(
            owner,
            move |_, _| updater_calls.set(updater_calls.get() + 1),
            false,
        )
        .map_err(|error| fail(format!("install owner updater registers: {error}")))?;

    let mut replacement = Mobject::from_points(&[[5.0, 0.0, 0.0], [6.0, 0.0, 0.0]]);
    replacement
        .submobjects
        .push(Mobject::from_points(&[[7.0, 1.0, 0.0]]));
    stage
        .install(owner, replacement)
        .map_err(|error| fail(format!("value installs over its existing handle: {error}")))?;
    stage.update(0.125);

    let private_reclaimed = !stage.contains(private) && !stage.contains(private_leaf);
    let shared_preserved = stage.family(shared) == vec![shared, shared_leaf]
        && stage.get_points(shared_leaf) == Some(vec![[2.0, 1.0, 0.0]])
        && stage
            .get(outside)
            .is_some_and(|entry| entry.submobjects() == [shared]);
    let rooted_preserved = stage.get_points(rooted) == Some(vec![[3.0, -1.0, 0.0]]);
    let roots_preserved = stage.roots() == [owner, outside, rooted];
    let installed = stage.family(owner);
    let replacement_geometry = stage.get_points(owner)
        == Some(vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0]])
        && installed.len() == 2
        && stage.get_points(installed[1]) == Some(vec![[7.0, 1.0, 0.0]]);
    ctx.event(
        LogEvent::new("e2e.lifecycle.install_shared_family")
            .field("private_reclaimed", truth(private_reclaimed))
            .field("shared_preserved", truth(shared_preserved))
            .field("rooted_preserved", truth(rooted_preserved))
            .field("roots_preserved", truth(roots_preserved))
            .field("replacement_geometry", truth(replacement_geometry))
            .field("updater_calls", calls.get()),
    );
    Ok(RunOutcome::ok()
        .with_counter("install_private_reclaimed", u64::from(private_reclaimed))
        .with_counter("install_shared_preserved", u64::from(shared_preserved))
        .with_counter("install_rooted_preserved", u64::from(rooted_preserved))
        .with_counter("install_roots_preserved", u64::from(roots_preserved))
        .with_counter(
            "install_replacement_geometry",
            u64::from(replacement_geometry),
        )
        .with_counter("install_updater_calls", calls.get()))
}

/// A redraw may reuse a live source that another scene placement also owns.
fn lifecycle_always_redraw_shared_source_run(
    ctx: &mut RunCtx,
) -> Result<RunOutcome, ScenarioError> {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut stage = Stage::new();
    let source = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
    let other_parent = stage.add(Mobject::new());
    stage
        .attach(other_parent, source)
        .map_err(|error| fail(format!("shared source attaches: {error}")))?;
    let calls = Rc::new(Cell::new(0_u64));
    let factory_calls = Rc::clone(&calls);
    let redraw = stage.always_redraw(move |_| {
        factory_calls.set(factory_calls.get() + 1);
        source
    });
    // Batch admission preserves both placements of their shared child.
    stage
        .add_many_to_scene(&[other_parent, redraw])
        .map_err(|error| fail(format!("shared redraw enters scene: {error}")))?;

    let mut shared_links = true;
    let mut current_geometry = true;
    for point in [[1.0, -1.0, 0.0], [2.0, 1.0, 0.0], [-3.0, 2.0, 0.0]] {
        stage
            .set_points(source, &[point])
            .map_err(|error| fail(format!("shared redraw source moves: {error}")))?;
        stage.update(0.125);
        shared_links &= stage
            .get(redraw)
            .is_some_and(|entry| entry.submobjects() == [source])
            && stage
                .get(other_parent)
                .is_some_and(|entry| entry.submobjects() == [source]);
        current_geometry &= stage.get_points(source) == Some(vec![point]);
    }
    let source_alive = stage.contains(source);
    let other_parent_alive = stage.contains(other_parent);
    let roots_preserved = stage.roots() == [other_parent, redraw];
    ctx.event(
        LogEvent::new("e2e.lifecycle.always_redraw_shared_source")
            .field("factory_calls", calls.get())
            .field("source_alive", truth(source_alive))
            .field("other_parent_alive", truth(other_parent_alive))
            .field("shared_links", truth(shared_links))
            .field("current_geometry", truth(current_geometry))
            .field("roots_preserved", truth(roots_preserved)),
    );
    Ok(RunOutcome::ok()
        .with_counter("redraw_factory_calls", calls.get())
        .with_counter("redraw_source_alive", u64::from(source_alive))
        .with_counter("redraw_other_parent_alive", u64::from(other_parent_alive))
        .with_counter("redraw_shared_links", u64::from(shared_links))
        .with_counter("redraw_current_geometry", u64::from(current_geometry))
        .with_counter("redraw_roots_preserved", u64::from(roots_preserved)))
}

/// Pure execution alone does not prove that the FMTL record-lerp law can
/// reconstruct a frame. Discrete draw state must survive ordinary and shared
/// random-access replay, even when the endpoints have identical values.
fn lifecycle_bundle_render_state_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    fn bundle_failure(error: impl std::fmt::Display) -> ScenarioError {
        fail(error.to_string())
    }
    struct DepthSwitch {
        state: fmn_anim::AnimState,
    }
    impl fmn_anim::Animation for DepthSwitch {
        fn state(&self) -> &fmn_anim::AnimState {
            &self.state
        }
        fn state_mut(&mut self) -> &mut fmn_anim::AnimState {
            &mut self.state
        }
        fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
            stage
                .get_mut(self.state.mobject())
                .expect("the scene retains its animated drawable")
                .uniforms_mut()
                .depth_test = (0.5..1.0).contains(&alpha);
        }
        fn interpolate_submobject(&mut self, _: &mut Stage, _: &[Mob], _: f64) {}
        fn effect_signature(&self) -> fmn_anim::AnimationSignature {
            fmn_anim::AnimationSignature::Pure
        }
    }
    let make = || -> Result<(Stage, Timeline), ScenarioError> {
        let mut stage = Stage::new();
        let mob = stage.add(VMobject::from_points(vec![
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ]));
        stage.add_to_scene(mob).map_err(bundle_failure)?;
        let mut timeline = Timeline::new(4).map_err(bundle_failure)?;
        timeline
            .play(vec![Box::new(DepthSwitch {
                state: fmn_anim::AnimState::new(
                    mob,
                    fmn_anim::AnimConfig {
                        rate_func: RateFunc::linear(),
                        ..fmn_anim::AnimConfig::default()
                    },
                ),
            })])
            .map_err(bundle_failure)?;
        Ok((stage, timeline))
    };
    let rng = RngRoot::from_seed(31);
    let (mut stage, mut timeline) = make()?;
    let mut observed = Vec::new();
    let reports = timeline
        .render(&mut stage, &rng, &mut |packet| {
            observed.push(packet.materialize_stage())
        })
        .map_err(bundle_failure)?;
    let nominated_pure = reports.len() == 1 && reports[0].purity.is_pure();
    let (mut stage, timeline) = make()?;
    let bytes =
        fmn_scene::export_timeline_bundle(timeline, &mut stage, &rng).map_err(bundle_failure)?;
    let bundle = fmn_scene::TimelineBundle::from_bytes(&bytes).map_err(bundle_failure)?;
    let recorded = bundle.segment_kind(0) == Some(fmn_scene::BundleSegmentKind::Stateful);
    let shared = fmn_scene::TimelineBundle::from_bytes(&bytes)
        .map_err(bundle_failure)?
        .into_shared()
        .map_err(bundle_failure)?;
    let mut cache = fmn_scene::timeline_bundle::TimelineFrameCache::default();
    let mut matches = 0_u64;
    for index in [3, 1, 0, 2] {
        let expected = observed
            .get(index as usize)
            .ok_or_else(|| fail("native timeline omitted a scheduled capture"))?;
        let expected_bytes = expected
            .snapshot()
            .to_render_bytes()
            .map_err(bundle_failure)?;
        let ordinary = bundle.stage_at(index).map_err(bundle_failure)?;
        let random_access = cache
            .materialize(&shared.frame_job(index).map_err(bundle_failure)?)
            .map_err(bundle_failure)?;
        let wanted_depth = index == 1 || index == 2;
        let ordinary_depth = ordinary
            .roots()
            .first()
            .and_then(|mob| ordinary.get(*mob))
            .is_some_and(|entry| entry.uniforms().depth_test);
        let equal = ordinary
            .snapshot()
            .to_render_bytes()
            .map_err(bundle_failure)?
            == expected_bytes
            && random_access
                .snapshot()
                .to_render_bytes()
                .map_err(bundle_failure)?
                == expected_bytes
            && ordinary_depth == wanted_depth;
        matches += u64::from(equal);
        ctx.event(
            LogEvent::new("e2e.lifecycle.bundle_render_state")
                .field("frame", u64::from(index))
                .field("nominated_pure", truth(nominated_pure))
                .field("recorded", truth(recorded))
                .field("depth_test", truth(ordinary_depth))
                .field("equal", truth(equal))
                .field("sha256", sha256(&expected_bytes).to_hex()),
        );
    }
    Ok(RunOutcome::ok()
        .with_counter(
            "bundle_render_state_frames",
            u64::from(bundle.frame_count()),
        )
        .with_counter("bundle_render_state_pure", u64::from(nominated_pure))
        .with_counter("bundle_render_state_recorded", u64::from(recorded))
        .with_counter("bundle_render_state_matches", matches))
}

/// Live 3D helpers feed ordinary native surface and curve primitives through
/// the same renderer as independently translated owning-chart constructions.
fn live_coordinate_surface_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let book = fmn_text::FontBook::bundled()
        .map_err(|error| fail(format!("live surface fonts: {error}")))?;
    let axes = fmn_library::ThreeDAxes::new()
        .x_range([-1.0, 1.0, 1.0])
        .y_range([-1.0, 1.0, 1.0])
        .z_range([-1.0, 1.0, 1.0])
        .build(&book)
        .map_err(|error| fail(format!("build live surface chart: {error}")))?;
    let mut stage = Stage::new();
    let chart = stage.add(axes.clone());
    let live = axes
        .bind(&stage, chart)
        .map_err(|error| fail(format!("bind live surface chart: {error}")))?;
    let offset = [0.5, -0.25, 0.25];
    stage.shift(chart, offset);
    let height = |x: f64, y: f64| 0.25 * x + 0.5 * y;
    let path = |t: f64| [t, 0.0, 0.5];
    let surface = live
        .get_surface(&stage, height)
        .map_err(|error| fail(format!("map live surface: {error}")))?
        .resolution(5, 5)
        .color(TEAL_B)
        .shading([0.0; 3])
        .build();
    let vertices = surface.points().len() as u64;
    let curve = live
        .get_parametric_curve(&stage, path)
        .map_err(|error| fail(format!("map live 3D curve: {error}")))?
        .t_range([-0.75, 0.75, 0.25])
        .build()
        .map_err(|error| fail(format!("sample live 3D curve: {error}")))?;
    let surface = stage.add(surface);
    let curve = stage.add(curve);
    stage
        .add_many_to_scene(&[surface, curve])
        .map_err(|error| fail(format!("adopt live 3D geometry: {error}")))?;

    let mut expected = Stage::new();
    let surface = expected.add(
        axes.get_graph(height)
            .resolution(5, 5)
            .color(TEAL_B)
            .shading([0.0; 3])
            .build(),
    );
    let curve = expected.add(
        axes.get_parametric_curve(path)
            .t_range([-0.75, 0.75, 0.25])
            .build()
            .map_err(|error| fail(format!("sample independent 3D curve: {error}")))?,
    );
    expected
        .add_many_to_scene(&[surface, curve])
        .map_err(|error| fail(format!("adopt independent 3D geometry: {error}")))?;
    let unmoved = render_certified_camera_doc(&expected);
    expected.shift(surface, offset);
    expected.shift(curve, offset);
    let rendered = render_certified_camera_doc(&stage);
    let matches = rendered == render_certified_camera_doc(&expected);
    let rejects_unmoved = rendered != unmoved;
    let visible = rendered != render_certified_camera_doc(&Stage::new());
    ctx.event(
        LogEvent::new("e2e.render_matrix.live_coordinate_surface")
            .field("vertices", vertices)
            .field("matches", truth(matches))
            .field("rejects_unmoved", truth(rejects_unmoved))
            .field("visible", truth(visible))
            .field("sha256", sha256(&rendered).to_hex()),
    );
    Ok(RunOutcome::ok()
        .with_counter("live_surface_vertices", vertices)
        .with_counter("live_surface_matches", u64::from(matches))
        .with_counter("live_surface_rejects_unmoved", u64::from(rejects_unmoved))
        .with_counter("live_surface_visible", u64::from(visible)))
}

/// A native timeline must capture the configured temporal detail and duration
/// of a trace, with a visible tapered stroke and no construction-time sample.
fn lifecycle_traced_path_cadence_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_library::{StrokeProfile, TracedPath};
    use std::cell::Cell;
    use std::rc::Rc;

    let calls = Rc::new(Cell::new(0_u64));
    let mut stage = Stage::new();
    let trace = TracedPath::new()
        .with_time_traced(0.5)
        .with_time_per_anchor(0.25)
        .with_stroke_width(StrokeProfile::Taper(vec![0.0, 4.0]))
        .with_stroke_opacity(StrokeProfile::Taper(vec![0.0, 1.0]))
        .add_to_stage(&mut stage, {
            let calls = Rc::clone(&calls);
            move |stage| {
                calls.set(calls.get() + 1);
                [stage.time(), 0.5 * stage.time(), 0.0]
            }
        })
        .map_err(|error| fail(format!("construct native trace: {error}")))?;
    stage
        .add_to_scene(trace)
        .map_err(|error| fail(format!("adopt native trace: {error}")))?;
    stage.update(0.0);
    let starts_empty = stage
        .get_points(trace)
        .is_some_and(|points| points.is_empty())
        && calls.get() == 0;
    let mut timeline =
        Timeline::new(8).map_err(|error| fail(format!("create native trace clock: {error}")))?;
    timeline
        .wait(1.0)
        .map_err(|error| fail(format!("schedule native trace: {error}")))?;
    let mut captures = Vec::new();
    let reports = timeline
        .render(&mut stage, &RngRoot::from_seed(41), &mut |packet| {
            captures.push(packet.materialize_stage());
        })
        .map_err(|error| fail(format!("capture native trace: {error}")))?;
    let first = captures
        .first()
        .ok_or_else(|| fail("trace omitted its first capture"))?;
    let last = captures
        .last()
        .ok_or_else(|| fail("trace omitted its final capture"))?;
    let first_observation = first.get_points(trace) == Some(vec![[0.125, 0.0625, 0.0]]);
    let points = last
        .get_points(trace)
        .ok_or_else(|| fail("trace capture lost its drawable"))?;
    let anchors: Vec<_> = points.iter().copied().step_by(2).collect();
    let cadence_and_window = anchors
        == vec![
            [0.5, 0.25, 0.0],
            [0.625, 0.3125, 0.0],
            [0.875, 0.4375, 0.0],
            [1.0, 0.5, 0.0],
        ];
    let entry = last
        .get(trace)
        .ok_or_else(|| fail("trace capture lost its style"))?;
    let widths = entry
        .buffer
        .read_column("stroke_width")
        .ok_or_else(|| fail("trace has no widths"))?;
    let rgba = entry
        .buffer
        .read_column("stroke_rgba")
        .ok_or_else(|| fail("trace has no stroke color"))?;
    let tapers = widths.first() == Some(&0.0)
        && widths.last() == Some(&4.0)
        && rgba.get(3) == Some(&0.0)
        && rgba.last() == Some(&1.0);
    let stateful = reports.len() == 1 && !reports[0].purity.is_pure();
    let rendered = render_certified_doc(last);
    let visible = rendered != render_certified_doc(&Stage::new());
    ctx.event(
        LogEvent::new("e2e.lifecycle.traced_path_cadence")
            .field("frames", captures.len() as u64)
            .field("source_calls", calls.get())
            .field("starts_empty", truth(starts_empty))
            .field("first_observation", truth(first_observation))
            .field("cadence_and_window", truth(cadence_and_window))
            .field("tapers", truth(tapers))
            .field("stateful", truth(stateful))
            .field("visible", truth(visible))
            .field("sha256", sha256(&rendered).to_hex()),
    );
    Ok(RunOutcome::ok()
        .with_counter("traced_path_frames", captures.len() as u64)
        .with_counter("traced_path_source_calls", calls.get())
        .with_counter("traced_path_starts_empty", u64::from(starts_empty))
        .with_counter(
            "traced_path_first_observation",
            u64::from(first_observation),
        )
        .with_counter(
            "traced_path_cadence_and_window",
            u64::from(cadence_and_window),
        )
        .with_counter("traced_path_tapers", u64::from(tapers))
        .with_counter("traced_path_stateful", u64::from(stateful))
        .with_counter("traced_path_visible", u64::from(visible)))
}

/// Native updater attributes must branch with a copied family and remain
/// frozen in frame snapshots, including their references to the traced source.
fn lifecycle_owned_tail_branches_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let tail_failure = |error| fail(format!("native tail branch: {error}"));
    let mut original = Stage::new();
    let source = original.add(VMobject::from_points(vec![[0.0; 3]]));
    let tail = fmn_library::TracingTail::new()
        .with_time_traced(0.5)
        .with_time_per_anchor(0.25)
        .and_then(|tail| tail.add_to_stage(&mut original, source))
        .map_err(|error| fail(format!("construct owned tail: {error}")))?;
    let family = original.add(fmn_library::vmobject::v_group(Vec::new()));
    original.attach(family, source).map_err(tail_failure)?;
    original.attach(family, tail).map_err(tail_failure)?;
    original.add_to_scene(family).map_err(tail_failure)?;
    original
        .set_points(source, &[[1.0, 0.0, 0.0]])
        .map_err(tail_failure)?;
    original.update(0.25);
    let frozen = original.snapshot();
    let early_frame = render_certified_doc(&original);
    let early_points = original.get_points(tail);

    let mut copied = Stage::new();
    let copied_family = original
        .copy_into(family, &mut copied)
        .map_err(tail_failure)?;
    copied.add_to_scene(copied_family).map_err(tail_failure)?;
    let children = copied
        .get(copied_family)
        .ok_or_else(|| fail("copied tail family is missing"))?
        .submobjects()
        .to_vec();
    let [copied_source, copied_tail] = children.as_slice() else {
        return Err(fail("copied tail family lost its source or trail"));
    };
    copied
        .set_points(*copied_source, &[[2.0, -1.0, 0.0]])
        .map_err(tail_failure)?;
    copied.update(0.25);
    let independent_history = original.get_points(tail) == early_points;
    let anchors = |stage: &Stage, mob| {
        stage
            .get_points(mob)
            .unwrap_or_default()
            .into_iter()
            .step_by(2)
            .collect::<Vec<_>>()
    };
    let copied_source_remapped =
        anchors(&copied, *copied_tail) == vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, -1.0, 0.0]];
    original
        .set_points(source, &[[2.0, 1.0, 0.0]])
        .map_err(tail_failure)?;
    original.update(0.25);
    let original_history =
        anchors(&original, tail) == vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]];
    let original_frame = render_certified_doc(&original);
    let copied_frame = render_certified_doc(&copied);
    let snapshot_frozen = frozen.materialize().get_points(tail) == early_points
        && render_certified_doc(&frozen.materialize()) == early_frame;

    // Replaying from the captured frame must restore the internal history as
    // well as the already drawn points, after both live branches have advanced.
    original.restore(&frozen);
    original
        .set_points(source, &[[2.0, 1.0, 0.0]])
        .map_err(tail_failure)?;
    original.update(0.25);
    let snapshot_replay = render_certified_doc(&original) == original_frame
        && anchors(&original, tail) == vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]];
    let empty_frame = render_certified_doc(&Stage::new());
    let visible = original_frame != empty_frame && copied_frame != empty_frame;
    let distinct_frames = original_frame != copied_frame;
    ctx.event(
        LogEvent::new("e2e.lifecycle.owned_tail_branches")
            .field("source_remapped", truth(copied_source_remapped))
            .field("independent_history", truth(independent_history))
            .field("original_history", truth(original_history))
            .field("snapshot_frozen", truth(snapshot_frozen))
            .field("snapshot_replay", truth(snapshot_replay))
            .field("visible", truth(visible))
            .field("distinct_frames", truth(distinct_frames))
            .field("original_sha256", sha256(&original_frame).to_hex())
            .field("copied_sha256", sha256(&copied_frame).to_hex()),
    );
    Ok(RunOutcome::ok()
        .with_counter(
            "owned_tail_source_remapped",
            u64::from(copied_source_remapped),
        )
        .with_counter(
            "owned_tail_independent_history",
            u64::from(independent_history),
        )
        .with_counter("owned_tail_original_history", u64::from(original_history))
        .with_counter("owned_tail_snapshot_frozen", u64::from(snapshot_frozen))
        .with_counter("owned_tail_snapshot_replay", u64::from(snapshot_replay))
        .with_counter("owned_tail_visible", u64::from(visible))
        .with_counter("owned_tail_distinct_frames", u64::from(distinct_frames)))
}

/// A succession's reverse rate curve must revisit the correct member, and
/// native timeline seeking must reproduce those same certified pixels in any
/// request order even though later members begin just in time.
fn lifecycle_succession_reverse_seek_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_anim::{AnimConfig, Succession, Transform};

    let mut stage = Stage::new();
    let mob = stage.add(fmn_library::Square::new().side_length(1.0).color(TEAL_B));
    stage
        .add_to_scene(mob)
        .map_err(|error| fail(format!("adopt succession square: {error}")))?;
    let initial_frame = render_certified_doc(&stage);
    let mut members: Vec<Box<dyn fmn_anim::Animation>> = Vec::new();
    // Keep the entire one-unit square inside the certified viewport at peak.
    for x in [1.0, 2.0] {
        let target = stage
            .copy_family(mob)
            .map_err(|error| fail(format!("copy succession target: {error}")))?;
        stage.shift(target, [x, 0.0, 0.0]);
        members.push(Box::new(Transform::new(mob, target).with_config(
            AnimConfig {
                run_time: 1.0,
                rate_func: RateFunc::linear(),
                ..AnimConfig::default()
            },
        )));
    }
    let succession = Succession::new(&mut stage, members)
        .map_err(|error| fail(format!("construct succession: {error}")))?
        .with_rate_func(RateFunc::Base(fmn_core::rate::there_and_back));
    let mut timeline =
        Timeline::new(4).map_err(|error| fail(format!("succession clock: {error}")))?;
    timeline
        .play(vec![Box::new(succession)])
        .map_err(|error| fail(format!("schedule succession: {error}")))?;
    let rng = RngRoot::from_seed(47);
    let mut serial_frames = Vec::new();
    let mut centers = Vec::new();
    let reports = timeline
        .render(&mut stage, &rng, &mut |packet| {
            let captured = packet.materialize_stage();
            centers.push(captured.get_center(mob));
            serial_frames.push(render_certified_doc(&captured));
        })
        .map_err(|error| fail(format!("render reverse succession: {error}")))?;
    let checkpoint_replay = reports.len() == 1 && !reports[0].purity.is_pure();
    let returns_to_start = serial_frames.last() == Some(&initial_frame)
        && centers.last().is_some_and(|point| point[0].abs() < 1e-9);
    let moves = centers
        .get(3)
        .is_some_and(|point| (point[0] - 2.0).abs() < 1e-6)
        && serial_frames
            .get(3)
            .is_some_and(|frame| *frame != initial_frame);
    let empty_frame = render_certified_doc(&Stage::new());
    let visible = serial_frames.iter().all(|frame| *frame != empty_frame);
    let mut seek_matches = 0_u64;
    for frame in [8, 1, 4, 2, 7, 3, 6, 5] {
        let packet = timeline
            .seek(&mut stage, &rng, frame)
            .map_err(|error| fail(format!("seek reverse succession frame {frame}: {error}")))?;
        let rendered = render_certified_doc(&packet.materialize_stage());
        let expected = serial_frames
            .get((frame - 1) as usize)
            .ok_or_else(|| fail("succession omitted a scheduled frame"))?;
        let equal = rendered == *expected;
        seek_matches += u64::from(equal);
        ctx.event(
            LogEvent::new("e2e.lifecycle.succession_reverse_seek")
                .field("frame", frame as u64)
                .field("checkpoint_replay", truth(checkpoint_replay))
                .field("returns_to_start", truth(returns_to_start))
                .field("equal", truth(equal))
                .field("sha256", sha256(&rendered).to_hex()),
        );
    }
    Ok(RunOutcome::ok()
        .with_counter("succession_frames", serial_frames.len() as u64)
        .with_counter("succession_checkpoint_replay", u64::from(checkpoint_replay))
        .with_counter("succession_returns_to_start", u64::from(returns_to_start))
        .with_counter("succession_moves", u64::from(moves))
        .with_counter("succession_visible", u64::from(visible))
        .with_counter("succession_seek_matches", seek_matches))
}

/// fm-c1up: TracingTail seeds its updater history while its public geometry
/// stays empty until the first positive-dt observation is published.
fn lifecycle_tracing_tail_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let initial = [2.0, 1.0, 0.0];
    let current = [3.0, 2.0, 0.0];
    let mut stage = Stage::new();
    let source = stage.add(VMobject::from_points(vec![initial]));
    let tail = fmn_library::TracingTail::new()
        .with_time_traced(0.5)
        .with_time_per_anchor(0.125)
        .and_then(|tail| tail.add_to_stage(&mut stage, source))
        .map_err(|error| fail(format!("tracing tail constructs: {error}")))?;
    stage
        .add_to_scene(tail)
        .map_err(|error| fail(format!("tracing tail enters scene: {error}")))?;
    let initial_points = stage
        .get_points(tail)
        .ok_or_else(|| fail("tracing tail handle is stale after construction"))?
        .len() as u64;
    stage
        .set_points(source, &[current])
        .map_err(|error| fail(format!("traced source moves: {error}")))?;
    stage.update(0.0);
    let zero_dt_points = stage
        .get_points(tail)
        .ok_or_else(|| fail("tracing tail handle is stale after zero dt"))?
        .len() as u64;
    stage.update(0.125);
    let points = stage
        .get_points(tail)
        .ok_or_else(|| fail("tracing tail handle is stale after positive dt"))?;
    let seed_preserved = points.first() == Some(&initial);
    let endpoint_matches = points.last() == Some(&current);
    let entry = stage
        .get(tail)
        .ok_or_else(|| fail("tracing tail style handle is stale"))?;
    let widths = entry
        .buffer
        .read_column("stroke_width")
        .ok_or_else(|| fail("tracing tail has no stroke width field"))?;
    let rgba = entry
        .buffer
        .read_column("stroke_rgba")
        .ok_or_else(|| fail("tracing tail has no stroke color field"))?;
    let tapers_preserved = widths.first() == Some(&0.0)
        && widths.last() == Some(&3.0)
        && rgba.get(3) == Some(&0.0)
        && rgba.last() == Some(&1.0);
    let stateful = !fmn_anim::purity::classify_wait(&stage, false).is_pure();

    ctx.event(
        LogEvent::new("e2e.lifecycle.tracing_tail")
            .field("initial_points", initial_points)
            .field("zero_dt_points", zero_dt_points)
            .field("updated_points", points.len() as u64)
            .field("seed_preserved", truth(seed_preserved))
            .field("endpoint_matches", truth(endpoint_matches))
            .field("tapers_preserved", truth(tapers_preserved))
            .field("stateful", truth(stateful)),
    );
    Ok(RunOutcome::ok()
        .with_counter("trace_initial_points", initial_points)
        .with_counter("trace_zero_dt_points", zero_dt_points)
        .with_counter("trace_updated_points", points.len() as u64)
        .with_counter("trace_seed_preserved", u64::from(seed_preserved))
        .with_counter("trace_endpoint_matches", u64::from(endpoint_matches))
        .with_counter("trace_tapers_preserved", u64::from(tapers_preserved))
        .with_counter("trace_stateful", u64::from(stateful)))
}

/// Journal round-trip: the scene's command record (add, pure play,
/// stateful wait) serializes, deserializes, hashes identically, and
/// `plan_replay` reuses exactly the non-barrier prefix — and stops with
/// the named reason when the incoming stream diverges.
fn lifecycle_journal_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let mut journal = Journal::new();
    let commands = [
        (CommandKind::Add, EffectClass::Pure, "add circle+label"),
        (CommandKind::Play, EffectClass::Pure, "play shift(circle)"),
        (
            CommandKind::Wait,
            EffectClass::Stateful(vec![fmn_scene::journal::ImpureEffectTag::StopCondition]),
            "wait(0.125)",
        ),
    ];
    let mut records = Vec::new();
    for (index, (kind, effect, label)) in commands.iter().enumerate() {
        let identity = sha256(label.as_bytes());
        let state_hash = sha256(format!("e2e-state-{index}").as_bytes());
        let record = CommandRecord {
            kind: *kind,
            identity,
            label: (*label).to_string(),
        };
        let entry = Entry {
            command: record.clone(),
            effect: effect.clone(),
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: (index == 0).then(|| format!("checkpoint-{index}").into_bytes()),
            state_hash,
        };
        journal
            .record(entry.clone())
            .map_err(|error| fail(format!("journal records: {error}")))?;
        // The same entries ride the run's repro bundle (the §18 closure).
        ctx.record_journal(entry);
        records.push(record);
    }

    let bytes = journal
        .to_bytes()
        .map_err(|error| fail(format!("journal encodes: {error}")))?;
    let restored =
        Journal::from_bytes(&bytes).map_err(|error| fail(format!("journal decodes: {error}")))?;
    let hash_before = journal
        .content_hash()
        .map_err(|error| fail(format!("journal hashes: {error}")))?;
    let hash_after = restored
        .content_hash()
        .map_err(|error| fail(format!("restored journal hashes: {error}")))?;
    let roundtrip_equal = hash_before == hash_after && restored.entries().len() == records.len();

    // Full replay: the incoming stream matches the record.
    let plan = plan_replay(&restored, &records, &|_read| true);
    let full_reuse = plan.reuse == records.len() && plan.reason.is_none();
    let resumes_at_checkpoint = plan.resume_checkpoint == Some(0);

    // Diverged replay: a different third command stops the walk with the
    // named reason at exactly that index.
    let mut divergent = records.clone();
    divergent[2] = CommandRecord {
        kind: CommandKind::Wait,
        identity: sha256(b"wait(0.5)"),
        label: "wait(0.5)".to_string(),
    };
    let divergent_plan = plan_replay(&restored, &divergent, &|_read| true);
    let mismatch_detected = divergent_plan.reuse == 2
        && matches!(
            divergent_plan.reason,
            Some(fmn_scene::journal::InvalidationReason::CommandMismatch { index: 2 })
        );

    ctx.event(
        LogEvent::new("e2e.lifecycle.journal")
            .field("entries", records.len() as u64)
            .field("roundtrip_equal", truth(roundtrip_equal))
            .field("replay_reuse", plan.reuse as u64)
            .field("mismatch_detected", truth(mismatch_detected)),
    );
    ctx.counter("journal_roundtrip_equal", u64::from(roundtrip_equal));
    ctx.counter("journal_replay_reuse", plan.reuse as u64);
    ctx.counter("journal_mismatch_detected", u64::from(mismatch_detected));

    if !(roundtrip_equal && full_reuse && resumes_at_checkpoint && mismatch_detected) {
        return Err(fail(format!(
            "journal round-trip broke: equal={roundtrip_equal} reuse={} checkpoint={:?} \
             mismatch={mismatch_detected}",
            plan.reuse, plan.resume_checkpoint
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("journal_roundtrip_equal", 1)
        .with_counter("journal_replay_reuse", plan.reuse as u64)
        .with_counter("journal_mismatch_detected", 1))
}

// ---------------------------------------------------------------------------
// fm-3kr / fm-n64 enhanced-surface lifecycle drills
// ---------------------------------------------------------------------------

/// fm-c1up: an 88-key keyboard keeps its musical layout and proportions
/// through native stage adoption and real frame rendering.
fn piano_keyboard_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let keyboard = fmn_library::drawings::piano()
        .map_err(|error| fail(format!("native piano construction: {error}")))?;
    let keys = keyboard.children().len() as u64;
    let black = keyboard
        .children()
        .iter()
        .filter(|key| key.style().fill_color == fmn_core::constants::GREY_E)
        .count() as u64;
    let expected_height = 13.0 / (52.0 * 0.15 + 51.0 * 0.02);
    let centered = keyboard
        .center_point()
        .iter()
        .all(|value| value.abs() < 1e-9);
    let proportions = (keyboard.length_over_dim(0) - 13.0).abs() < 1e-9
        && (keyboard.length_over_dim(1) - expected_height).abs() < 1e-9;
    let mut stage = Stage::new();
    let root = stage.add(keyboard);
    stage
        .add_to_scene(root)
        .map_err(|error| fail(format!("piano stage adoption: {error}")))?;
    let initial = render_certified_doc(&stage);
    stage.shift(root, UP);
    let moved = render_certified_doc(&stage);
    let blank = render_certified_doc(&Stage::new());
    let visible_motion = initial != blank && moved != blank && initial != moved;
    ctx.event(
        LogEvent::new("e2e.drawings.piano")
            .field("keys", keys)
            .field("black_keys", black)
            .field("centered", truth(centered))
            .field("proportions", truth(proportions))
            .field("visible_motion", truth(visible_motion)),
    );
    Ok(RunOutcome::ok()
        .with_counter("piano_keys", keys)
        .with_counter("piano_black_keys", black)
        .with_counter("piano_centered", u64::from(centered))
        .with_counter("piano_proportions", u64::from(proportions))
        .with_counter("piano_visible_motion", u64::from(visible_motion)))
}

/// The drawings shelf's asset-backed families refuse by name (ADR-0020):
/// every default constructor surfaces `AssetNotShipped` naming its class
/// and file — never a placeholder build.
fn drawings_asset_refusals_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_library::drawings::{
        DrawingsAssetError, double_speech_bubble, lightbulb, old_speech_bubble, vectorized_earth,
        video_icon,
    };

    let refusals = [
        (
            "Lightbulb",
            lightbulb(None).err().map(|error| (error, "lightbulb")),
        ),
        (
            "VideoIcon",
            video_icon(None).err().map(|error| (error, "video_icon")),
        ),
        (
            "OldSpeechBubble",
            old_speech_bubble(None, None, fmn_core::constants::LEFT, 1.0)
                .err()
                .map(|error| (error, "Bubbles_speech.svg")),
        ),
        (
            "DoubleSpeechBubble",
            double_speech_bubble(None, None, fmn_core::constants::LEFT, 1.0)
                .err()
                .map(|error| (error, "Bubbles_double_speech.svg")),
        ),
        (
            "VectorizedEarth",
            vectorized_earth(None).err().map(|error| (error, "earth")),
        ),
    ];
    let mut named = 0_u64;
    for (class, outcome) in &refusals {
        match outcome {
            Some((
                DrawingsAssetError::AssetNotShipped {
                    class: named_class,
                    asset: named_asset,
                },
                asset,
            )) => {
                if named_class != class || named_asset != asset {
                    return Err(fail(format!(
                        "{class} refused but named {named_class}/{named_asset}"
                    )));
                }
                named += 1;
            }
            other => {
                return Err(fail(format!(
                    "{class} must refuse by policy, got {other:?}"
                )));
            }
        }
    }
    ctx.event(LogEvent::new("e2e.drawings.asset_refusal").field("named", named));
    ctx.counter("asset_refusals_named", named);
    Ok(RunOutcome::ok().with_counter("asset_refusals_named", named))
}

/// NetworkGraph determinism drill: the same graph + seed replays a
/// bit-identical position digest twice, and a different seed diverges —
/// the OQ-5 kernel rules exercised through the public surface.
fn network_graph_layout_determinism_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn_library::network_graph::{GraphLayout, NetworkGraph};

    let nodes: Vec<&str> = (0..7)
        .map(|i| match i {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            _ => "g",
        })
        .collect();
    let edges = [
        ("a", "b"),
        ("b", "c"),
        ("c", "d"),
        ("d", "e"),
        ("e", "f"),
        ("f", "g"),
        ("g", "a"),
        ("b", "f"),
    ];
    let build = |seed: u64| -> Result<String, ScenarioError> {
        let network = NetworkGraph::from_edge_list(&nodes, &edges)
            .laid_out(&GraphLayout::Spring {
                seed,
                iterations: 40,
                ideal_edge: 1.0,
            })
            .map_err(|error| fail(format!("spring layout: {error}")))?;
        let mut bytes = Vec::new();
        for label in network.nodes() {
            let point = network
                .position(label)
                .ok_or_else(|| fail(format!("unpositioned {label}")))?;
            bytes.extend_from_slice(label.as_bytes());
            bytes.extend_from_slice(&point[0].to_bits().to_le_bytes());
            bytes.extend_from_slice(&point[1].to_bits().to_le_bytes());
        }
        Ok(sha256(&bytes).to_hex())
    };

    let first = build(0x5f3759df)?;
    let second = build(0x5f3759df)?;
    let divergent = build(0x5f3759de)?;
    let replay_equal = u64::from(first == second);
    let seeds_diverge = u64::from(first != divergent);
    if replay_equal != 1 || seeds_diverge != 1 {
        return Err(fail(format!(
            "spring determinism broke: replay_equal={replay_equal} seeds_diverge={seeds_diverge}"
        )));
    }
    ctx.event(
        LogEvent::new("e2e.network_graph.spring_determinism").field("seed", 0x005f_3759df_u64),
    );
    ctx.counter("spring_replay_equal", replay_equal);
    ctx.counter("spring_seeds_diverge", seeds_diverge);
    Ok(RunOutcome::ok()
        .with_counter("spring_replay_equal", replay_equal)
        .with_counter("spring_seeds_diverge", seeds_diverge))
}

/// NeuralNetworkMobject end-to-end drill (fm-74z): introspects an MLP with
/// explicit weights and biases, executes forward pass activation, attaches
/// live training updaters on stage, and proves segment purity classification
/// demoting to stateful under W4PURITY.
fn neural_network_live_training_and_purity_run(
    ctx: &mut RunCtx,
) -> Result<RunOutcome, ScenarioError> {
    use fmn_anim::purity::{ImpureEffect, Purity, classify_wait};
    use fmn_library::neural_network::NeuralNetworkMobject;
    use fmn_mobject::Stage;

    let layer_sizes = [2, 3, 1];
    let weights = vec![
        vec![vec![0.5, -0.5, 1.0], vec![-1.0, 1.0, 0.0]],
        vec![vec![1.5], vec![-0.5], vec![0.5]],
    ];
    let biases = vec![vec![0.1, -0.1, 0.0], vec![0.2]];

    let mut net = NeuralNetworkMobject::introspect_mlp(&layer_sizes, &weights, &biases)
        .map_err(|error| fail(format!("mlp introspection: {error}")))?;

    let out = net
        .feed_forward(&[0.5, 0.8])
        .map_err(|error| fail(format!("feed forward: {error}")))?;
    let feed_forward_matches = u64::from(!out.is_empty() && out[0] > 0.0 && out[0] < 1.0);

    let built = net
        .build()
        .map_err(|error| fail(format!("net build: {error}")))?;
    let mut stage = Stage::new();
    let handle = stage.add(built);
    stage
        .add_to_scene(handle)
        .map_err(|error| fail(format!("add to scene: {error}")))?;

    let initial_purity = classify_wait(&stage, false);
    let initially_pure = u64::from(initial_purity.is_pure());

    stage
        .add_updater(
            handle,
            |_stage, _mob| {
                // simulated live training update
            },
            false,
        )
        .map_err(|error| fail(format!("add updater: {error}")))?;

    let post_purity = classify_wait(&stage, false);
    let purity_is_stateful = u64::from(matches!(
        post_purity,
        Purity::Stateful(ref effs) if effs.contains(&ImpureEffect::SceneUpdater)
    ));

    if feed_forward_matches != 1 || initially_pure != 1 || purity_is_stateful != 1 {
        return Err(fail(format!(
            "neural network e2e broken: feed_forward={feed_forward_matches} pure_init={initially_pure} stateful={purity_is_stateful}"
        )));
    }

    ctx.event(
        LogEvent::new("e2e.neural_network.live_training")
            .field("layers", 3_u64)
            .field("stateful", 1_u64),
    );
    ctx.counter("feed_forward_matches", feed_forward_matches);
    ctx.counter("purity_is_stateful", purity_is_stateful);

    Ok(RunOutcome::ok()
        .with_counter("feed_forward_matches", feed_forward_matches)
        .with_counter("purity_is_stateful", purity_is_stateful))
}

/// CSV → TableMobject end-to-end through the suite's frame parser and
/// Scribe: insertion-order headers survive, the null cell renders empty,
/// and two builds of the same table hash identically.
fn data_table_csv_roundtrip_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    const CSV: &str = "region,score,active\nwest,3.5,true\neast,,false\nnorth,7,true\n";
    let table = fmn_library::data_mobjects::TableMobject::from_csv(CSV, ',')
        .map_err(|error| fail(format!("csv ingest: {error}")))?;

    let header_count = table.headers().len() as u64;
    let row_count = table.rows().len() as u64;
    let order_preserved = u64::from(table.headers() == ["region", "score", "active"]);
    let null_cell_empty = u64::from(table.rows()[1][1].is_empty());
    if header_count != 3 || row_count != 3 || order_preserved != 1 || null_cell_empty != 1 {
        return Err(fail(format!(
            "csv round-trip broke: headers={header_count} rows={row_count}              order={order_preserved} null={null_cell_empty}"
        )));
    }

    let book =
        fmn_text::FontBook::bundled().map_err(|error| fail(format!("bundled fonts: {error}")))?;
    let first = table
        .build(&book)
        .map_err(|error| fail(format!("table build: {error}")))?;
    let second = table
        .build(&book)
        .map_err(|error| fail(format!("table rebuild: {error}")))?;
    let digest_of = |root: &VMobject| {
        let mut bytes = Vec::new();
        let mut stack = vec![root];
        while let Some(current) = stack.pop() {
            for point in current.points() {
                bytes.extend_from_slice(&point[0].to_bits().to_le_bytes());
                bytes.extend_from_slice(&point[1].to_bits().to_le_bytes());
                bytes.extend_from_slice(&point[2].to_bits().to_le_bytes());
            }
            for child in current.children() {
                stack.push(child);
            }
        }
        sha256(&bytes).to_hex()
    };
    let builds_identical = u64::from(digest_of(&first) == digest_of(&second));
    if builds_identical != 1 {
        return Err(fail("two builds of one table diverged"));
    }

    ctx.event(
        LogEvent::new("e2e.data_table.roundtrip")
            .field("headers", header_count)
            .field("rows", row_count),
    );
    ctx.counter("data_table_headers", header_count);
    ctx.counter("data_table_rows", row_count);
    Ok(RunOutcome::ok()
        .with_counter("data_table_headers", header_count)
        .with_counter("data_table_rows", row_count)
        .with_counter("data_table_builds_identical", builds_identical))
}

/// Bounded E2E registration for PG-5's user-visible definition surface.
///
/// The release-perf producer remains the authoritative complete-corpus proof
/// for direct, frame-parallel, and ordered-pipeline scheduling. This fast
/// scenario binds that definition to one real committed scene rendered at the
/// permanent per-commit direct thread counts, so a CLI/definition landing
/// cannot silently lose its executable certified-render anchor.
fn pg5_direct_schedule_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let definition =
        Pg5Definition::new().map_err(|error| fail(format!("PG-5 definition identity: {error}")))?;
    definition
        .validate_corpus_lock()
        .map_err(|error| fail(format!("PG-5 corpus identity: {error}")))?;
    let case = scene_goldens::scene_named("circle_tex_label.v1")
        .ok_or_else(|| fail("PG-5 registered scene is missing"))?;
    let built = (case.build)(scene_goldens::corpus());
    let config = scene_goldens::frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&built.stage, 0)
        .map_err(|error| fail(format!("{} render plan: {error}", case.name)))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| fail(format!("PG-5 monotone table: {error}")))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .map_err(|error| fail(format!("PG-5 binning: {error}")))?;

    let mut digests = Vec::with_capacity(PG5_DIRECT_THREADS.len());
    for threads in PG5_DIRECT_THREADS {
        let job = FrameJob::with_identity(&plan, &mono, &binning, config, pg5_identity())
            .map_err(|error| fail(format!("PG-5 {threads}-thread job: {error}")))?;
        let frame = job
            .render(threads)
            .map_err(|error| fail(format!("PG-5 {threads}-thread render: {error}")))?;
        digests.push(
            frame_digest(&frame)
                .map_err(|error| fail(format!("PG-5 {threads}-thread digest: {error}")))?,
        );
    }
    let reference = digests
        .first()
        .copied()
        .ok_or_else(|| fail("PG-5 direct thread profile is empty"))?;
    let direct_mismatches = u64::try_from(
        digests
            .iter()
            .skip(1)
            // ubs:ignore — public frame self-goldens, not authentication material.
            .filter(|&&digest| digest != reference)
            .count(),
    )
    .map_err(|_| fail("PG-5 mismatch count exceeds u64"))?;

    ctx.record_asset(
        "pg5.certified-thread-matrix.definition",
        definition.to_tsv().as_bytes(),
    );
    ctx.event(
        LogEvent::new("performance.pg5.direct_schedule")
            .field("scene", case.name)
            .field("direct_threads", "1,4,16")
            .field("direct_mismatches", direct_mismatches)
            .field("reference_digest", reference.to_string())
            .field("benchmark_definition", definition.digest().to_string()),
    );
    ctx.counter("pg5_direct_threads", PG5_DIRECT_THREADS.len() as u64);
    ctx.counter("pg5_direct_mismatches", direct_mismatches);
    if direct_mismatches != 0 {
        return Err(fail(format!(
            "PG-5 representative certified render drifted at {direct_mismatches} thread counts"
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("pg5_direct_threads", PG5_DIRECT_THREADS.len() as u64)
        .with_counter("pg5_direct_mismatches", 0))
}

/// PG-6's user-visible evidence surface, registered end to end: a committed
/// corpus scene passes through the same warm/reuse arena lifecycle as the
/// release-perf producer, and the scenario log retains the observable proof.
fn pg6_allocation_reuse_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let definition = Pg6Definition::new();
    definition
        .validate_corpus_lock()
        .map_err(|error| fail(format!("PG-6 corpus identity: {error}")))?;
    let case = scene_goldens::scene_named("circle_tex_label.v1")
        .ok_or_else(|| fail("PG-6 registered scene is missing"))?;
    let built = (case.build)(scene_goldens::corpus());
    let config = scene_goldens::frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&built.stage, 0)
        .map_err(|error| fail(format!("{} render plan: {error}", case.name)))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| fail(format!("PG-6 monotone table: {error}")))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .map_err(|error| fail(format!("PG-6 binning: {error}")))?;

    let mut arena = FrameArena::new();
    let (mut frame, warm_digest, warm_allocations) = {
        let job =
            FrameJob::with_identity_in(&mut arena, &plan, &mono, &binning, config, pg6_identity())
                .map_err(|error| fail(format!("PG-6 warm job: {error}")))?;
        let frame = job
            .render(PG6_THREADS)
            .map_err(|error| fail(format!("PG-6 warm render: {error}")))?;
        let digest =
            frame_digest(&frame).map_err(|error| fail(format!("PG-6 warm digest: {error}")))?;
        (frame, digest, job.allocation_stats().heap_allocs_this_frame)
    };
    let (measured_digest, measured) = {
        let job =
            FrameJob::with_identity_in(&mut arena, &plan, &mono, &binning, config, pg6_identity())
                .map_err(|error| fail(format!("PG-6 measured job: {error}")))?;
        job.render_into(PG6_THREADS, &mut frame)
            .map_err(|error| fail(format!("PG-6 measured render: {error}")))?;
        let digest =
            frame_digest(&frame).map_err(|error| fail(format!("PG-6 measured digest: {error}")))?;
        (digest, job.allocation_stats())
    };
    // ubs:ignore — public deterministic artifact identity, not authentication material.
    let digest_equal = warm_digest == measured_digest;
    let allocation_free = measured.heap_allocs_this_frame == 0;
    ctx.record_asset(
        "pg6.primitive-steady-allocations.definition",
        definition.to_tsv().as_bytes(),
    );
    ctx.event(
        LogEvent::new("performance.pg6.allocations")
            .field("scene", case.name)
            .field("warm_digest", warm_digest.to_string())
            .field("measured_digest", measured_digest.to_string())
            .field("warm_allocations", warm_allocations)
            .field("measured_allocations", measured.heap_allocs_this_frame)
            .field("arena_bytes", measured.arena_buffer_bytes)
            .field("pool_slots", measured.pool_slots),
    );
    ctx.counter("pg6_digest_equal", u64::from(digest_equal));
    ctx.counter("pg6_measured_allocations", measured.heap_allocs_this_frame);
    if !digest_equal || !allocation_free {
        return Err(fail(format!(
            "PG-6 arena reuse drifted: digest_equal={digest_equal}, measured_allocations={}",
            measured.heap_allocs_this_frame
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("pg6_digest_equal", 1)
        .with_counter("pg6_measured_allocations", 0))
}

/// PG-6's scheduled full-tier corpus proof. This is the same three-frame UHD
/// production function the peak-RSS sampler runs during every repetition; the
/// e2e row validates real rendering and lock identity, not a reduced proxy.
fn pg6_peak_gallery_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let definition = Pg6PeakDefinition::new();
    definition
        .validate_corpus_lock()
        .map_err(|error| fail(format!("PG-6 peak corpus identity: {error}")))?;
    let cases = render_locked_gallery_once()
        .map_err(|error| fail(format!("PG-6 peak gallery render: {error}")))?;
    let complete = cases.len() == PG6_PEAK_CASE_COUNT;
    let distinct = complete
        && cases[0].frame_digest != cases[1].frame_digest
        && cases[0].frame_digest != cases[2].frame_digest
        && cases[1].frame_digest != cases[2].frame_digest;
    let preparation_bytes = cases
        .iter()
        .map(|case| case.preparation_bytes)
        .max()
        .unwrap_or(0);

    ctx.record_asset(
        "pg6.gallery-4k-3d-peak.definition",
        definition.to_tsv().as_bytes(),
    );
    ctx.event(
        LogEvent::new("performance.pg6.peak_gallery")
            .field("frames", cases.len())
            .field("width", u64::from(PG6_PEAK_WIDTH))
            .field("height", u64::from(PG6_PEAK_HEIGHT))
            .field("distinct_digests", distinct)
            .field("max_preparation_bytes", preparation_bytes)
            .field("benchmark_definition", definition.digest().to_string()),
    );
    ctx.counter("pg6_peak_frames", cases.len() as u64);
    ctx.counter("pg6_peak_distinct_digests", u64::from(distinct));
    if !complete || !distinct || preparation_bytes == 0 {
        return Err(fail(format!(
            "PG-6 peak gallery proof incomplete: frames={}, distinct={distinct}, max_preparation_bytes={preparation_bytes}",
            cases.len(),
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("pg6_peak_frames", PG6_PEAK_CASE_COUNT as u64)
        .with_counter("pg6_peak_distinct_digests", 1))
}

// ---------------------------------------------------------------------------
// Log-machinery scenarios
// ---------------------------------------------------------------------------

/// Preflight: a Tex-bearing scene's constructed-scene preflight typesets
/// every registered string **before frame zero** — the assertion the
/// harness exists to make, because nothing in the pixels shows it.
fn logs_preflight_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    // The real event order, recorded in-run: (ordinal, kind, payload).
    #[derive(Clone)]
    enum Trace {
        Preflight { typesets: u64 },
        FirstCapture { frame: i64 },
    }
    let trace: Arc<Mutex<Vec<Trace>>> = Arc::new(Mutex::new(Vec::new()));

    let sources: [&'static str; 2] = [r"\odot", r"\frac{1}{2}"];
    let label = tex_label(sources[0], 40.0)?;

    let mut scene = Scene::new(
        RuntimeConfig {
            fps: FPS,
            ..RuntimeConfig::default()
        },
        ctx.seed,
    )
    .map_err(|error| fail(format!("scene constructs: {error}")))?;

    let hook_trace = Arc::clone(&trace);
    scene
        .set_preflight_hook(move |_stage, _anchors| {
            let mut typesets = 0_u64;
            for source in sources {
                scene_goldens::corpus()
                    .tex
                    .typeset(fmn_tex::Mode::Math(fmd_math::Style::Text), source)
                    .map_err(|error| {
                        IntegrationError::new("preflight", format!("typeset {source:?}: {error}"))
                    })?;
                typesets += 1;
            }
            hook_trace
                .lock()
                .map_err(|_| IntegrationError::new("preflight", "trace lock poisoned"))?
                .push(Trace::Preflight { typesets });
            Ok(())
        })
        .map_err(|error| fail(format!("preflight hook installs: {error}")))?;

    struct TraceSink {
        trace: Arc<Mutex<Vec<Trace>>>,
    }
    impl SceneSink for TraceSink {
        fn capture(
            &mut self,
            _reason: CaptureReason,
            packet: FramePacket,
        ) -> Result<(), IntegrationError> {
            let mut guard = self
                .trace
                .lock()
                .map_err(|_| IntegrationError::new("sink", "trace lock poisoned"))?;
            if !guard
                .iter()
                .any(|event| matches!(event, Trace::FirstCapture { .. }))
            {
                guard.push(Trace::FirstCapture {
                    frame: packet.frame_index(),
                });
            }
            Ok(())
        }
    }

    let mut program = MatrixProgram {
        label: Some(label),
        wait_only: false,
    };
    let mut sink = TraceSink {
        trace: Arc::clone(&trace),
    };
    scene
        .run(&mut program, &mut sink)
        .map_err(|error| fail(format!("scene runs: {error}")))?;

    let events = trace
        .lock()
        .map_err(|_| fail("trace lock poisoned"))?
        .clone();
    let mut typesets = 0_u64;
    let mut preflight_ordinal = None;
    let mut first_capture_ordinal = None;
    let mut first_frame = 0_i64;
    for (ordinal, event) in events.iter().enumerate() {
        match event {
            Trace::Preflight { typesets: n } => {
                typesets = *n;
                preflight_ordinal = Some(ordinal);
            }
            Trace::FirstCapture { frame } => {
                first_capture_ordinal = Some(ordinal);
                first_frame = *frame;
            }
        }
    }
    // The contract (§17.1): the constructed-scene preflight — and with it
    // every typeset — completes before the first frame is emitted. (The
    // first *emitted* frame of an animated play is index 1: frame 0 is the
    // segment's begin state, never re-emitted.)
    let ordered = matches!(
        (preflight_ordinal, first_capture_ordinal),
        (Some(before), Some(after)) if before < after
    );

    ctx.event(
        LogEvent::new(e2e::spans::PREFLIGHT)
            .field("typesets", typesets)
            .field("before_first_frame", truth(ordered)),
    );
    ctx.event(
        LogEvent::new(e2e::spans::RENDER_FRAME)
            .field("frame", first_frame.max(0) as u64)
            .field("first_capture", "true"),
    );
    ctx.counter(e2e::counters::TYPESETS, typesets);

    if !ordered || typesets != 2 {
        return Err(fail(format!(
            "preflight contract broke: typesets={typesets} ordered={ordered} \
             first_frame={first_frame}"
        )));
    }
    Ok(RunOutcome::ok()
        .with_counter("typesets", typesets)
        .with_counter("preflight_before_first_frame", 1))
}

/// Purity: the segment classification is recorded for both classes — the
/// play (pure: frame-parallel eligible) and the wait (stateful: serial
/// front-end with its recorded reason).
fn logs_purity_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let label = tex_label(r"\odot", 40.0)?;
    let reports: Arc<Mutex<Vec<(String, String, i64)>>> = Arc::new(Mutex::new(Vec::new()));

    struct PurityProgram {
        label: Option<VMobject>,
        reports: Arc<Mutex<Vec<(String, String, i64)>>>,
    }
    impl SceneProgram for PurityProgram {
        fn name(&self) -> &str {
            "e2e_purity"
        }
        fn construct(
            &mut self,
            scene: &mut Scene,
            sink: &mut dyn SceneSink,
        ) -> Result<(), SceneError> {
            let circle = scene.add_mobject(Circle::new().radius(0.9))?;
            scene
                .stage_mut()
                .set_fill(circle, Some(BLUE_C), Some(0.35), Some(0.0), true);
            let label = self.label.take().ok_or(SceneError::InvalidLifecycle(
                "purity program constructed twice",
            ))?;
            scene.add_mobject(label)?;
            let builder = circle
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.25),
                    rate_func: Some(fmn_core::rate::linear),
                    ..AnimateArgs::default()
                })
                .map_err(|error| SceneError::Animation(error.into()))?
                .shift([0.6, 0.2, 0.0])
                .map_err(|error| SceneError::Animation(error.into()))?;
            let animation = fmn_anim::prepare_animation(builder, scene.stage_mut())?;
            if let Some(report) = scene.play(vec![animation], PlayOverrides::default(), sink)? {
                let purity = if report.purity.is_pure() {
                    "pure"
                } else {
                    "stateful"
                };
                self.reports
                    .lock()
                    .map_err(|_| SceneError::InvalidLifecycle("report lock poisoned"))?
                    .push(("play".to_string(), purity.to_string(), report.n_frames));
            }
            let report = scene.wait_until(0.125, &mut |_stage| false, sink)?;
            let purity = if report.purity.is_pure() {
                "pure"
            } else {
                "stateful"
            };
            self.reports
                .lock()
                .map_err(|_| SceneError::InvalidLifecycle("report lock poisoned"))?
                .push(("wait".to_string(), purity.to_string(), report.n_frames));
            Ok(())
        }
    }

    let mut scene = Scene::new(
        RuntimeConfig {
            fps: FPS,
            ..RuntimeConfig::default()
        },
        ctx.seed,
    )
    .map_err(|error| fail(format!("scene constructs: {error}")))?;
    let mut program = PurityProgram {
        label: Some(label),
        reports: Arc::clone(&reports),
    };
    let mut sink = PacketSink::default();
    scene
        .run(&mut program, &mut sink)
        .map_err(|error| fail(format!("scene runs: {error}")))?;

    let reports = reports
        .lock()
        .map_err(|_| fail("report lock poisoned"))?
        .clone();
    let mut saw_pure_play = false;
    let mut saw_stateful_wait = false;
    for (kind, purity, frames) in &reports {
        ctx.event(
            LogEvent::new(e2e::spans::PURITY)
                .field("segment", kind.as_str())
                .field("purity", purity.as_str())
                .field("frames", (*frames).max(0) as u64),
        );
        saw_pure_play |= kind == "play" && purity == "pure";
        saw_stateful_wait |= kind == "wait" && purity == "stateful";
    }
    ctx.counter("segments_classified", reports.len() as u64);
    if !(saw_pure_play && saw_stateful_wait) {
        return Err(fail(format!("purity classifications drifted: {reports:?}")));
    }
    Ok(RunOutcome::ok()
        .with_counter("segments_classified", reports.len() as u64)
        .with_counter("pure_segments", 1))
}

/// Engine identity and plan decision on a reproducible-class run: the
/// certified identity is recorded with its tuning provenance pinned.
fn logs_engine_identity_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let preset = PRESETS[0];
    let topology = HardwareTopology::fallback(8);
    let plan = ExecutionPlan::derive(
        PlanRequest::certified(
            RenderIntent::Offline,
            SurfaceSpec::lumen(preset.width, preset.height),
            OutputPixelFormat::Rgba8,
        ),
        &topology,
        None,
    )
    .map_err(|error| fail(format!("certified plan derives: {error}")))?;
    ctx.event(
        LogEvent::new(e2e::spans::EXECUTION_PLAN)
            .field("determinism", "certified")
            .field("tuning_source", tuning_source_name(plan.tuning_source))
            .field("fine_tile", u64::from(plan.fine_tile))
            .field("macro_tile", u64::from(plan.macro_tile)),
    );

    let identity = EngineIdentity::certified();
    ctx.event(
        LogEvent::new(e2e::spans::ENGINE)
            .field("engine", engine_kind_name(identity.engine))
            .field("reproducible", "true")
            .field("renderer_version", u64::from(identity.renderer_version)),
    );

    // Render one certified frame so the identity event is attached to real
    // engine work, not a bare declaration.
    let packets = run_matrix_scene(ctx.seed, true)?;
    let terminal = packets
        .last()
        .ok_or_else(|| fail("scene produced no terminal packet"))?;
    let frame = render_packet(terminal, identity, preset_frame_config(&preset), 1)?;
    let doc = encode_frame(&frame).map_err(|error| fail(format!("encode: {error}")))?;
    ctx.event(
        LogEvent::new(e2e::spans::RENDER_FRAME)
            .field("frame", 0_u64)
            .field("engine", engine_kind_name(identity.engine)),
    );
    ctx.counter("engine_certified_frames", 1);
    Ok(RunOutcome::ok()
        .with_counter("engine_certified_frames", 1)
        .with_counter("certified_doc_bytes", doc.len() as u64))
}

/// §15.1's product-facing proof: compile and execute a primitive scene using
/// only the `fmn` facade. The facade delegates to the same Scene runtime that
/// the rest of this catalog exercises; the capture count is the observable
/// proof that the public entry point crossed the real frame boundary.
fn public_facade_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use fmn::prelude::*;

    #[derive(Default)]
    struct FacadeProgram;

    impl SceneConstruct for FacadeProgram {
        fn name(&self) -> &str {
            "public_facade_circle_shift"
        }

        fn construct(&mut self, stage: &mut fmn::Stage<'_>) -> fmn::Result<()> {
            let circle = stage.add(Circle::new().radius(0.75).color(BLUE_C))?;
            let square = stage.add(Square::new().side_length(1.25).color(YELLOW))?;
            let movement = circle
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.25),
                    rate_func: Some(fmn::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .shift([0.5, 0.25, 0.0])?;
            let square_movement = square
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.25),
                    rate_func: Some(fmn::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .shift([-0.25, 0.0, 0.0])?
                .rotate(PI / 4.0)?
                .set_opacity(0.5)?;
            stage.play((movement, square_movement))?;
            stage.wait(0.125)?;
            Ok(())
        }
    }

    #[derive(Default)]
    struct FacadeSink {
        captures: u64,
    }

    impl SceneSink for FacadeSink {
        fn capture(
            &mut self,
            _reason: CaptureReason,
            _packet: fmn::animation::FramePacket,
        ) -> std::result::Result<(), IntegrationError> {
            self.captures += 1;
            Ok(())
        }
    }

    let mut sink = FacadeSink::default();
    let completed = run_scene(
        &mut FacadeProgram,
        RuntimeConfig {
            fps: FPS,
            ..RuntimeConfig::default()
        },
        0xFACADE,
        &mut sink,
    )
    .map_err(|error| fail(format!("public facade: {error}")))?;
    if completed.report().play_count != 2 || sink.captures != 3 {
        return Err(fail(format!(
            "public facade produced {} plays and {} captures, expected 2 and 3",
            completed.report().play_count,
            sink.captures
        )));
    }

    ctx.event(
        LogEvent::new(e2e::spans::SCENE_CONSTRUCT)
            .field("scene", "public_facade_circle_shift")
            .field("captures", sink.captures),
    );
    ctx.counter("public_facade_captures", sink.captures);
    Ok(RunOutcome::ok()
        .with_counter("public_facade_plays", completed.report().play_count)
        .with_counter("public_facade_captures", sink.captures))
}

// ---------------------------------------------------------------------------
// Parity drills (fm-parity-drill-scenarios-hzng): one fast-tier drill per
// Parity Ledger ruling family. Each scenario drives the real Rust API
// surface and asserts the CORRECTED behavior, so reintroducing the
// Reference-buggy expectation turns the drill red — a drill that cannot
// fail is not a drill.
// ---------------------------------------------------------------------------

/// A bare vmobject in `stage` carrying `points` as its point records (the
/// fmn-anim movement-suite construction: shared-anchor quad runs).
fn parity_vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).expect("a fresh handle has an entry");
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len())
        .expect("a bounded point record buffer");
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|point| point.iter().map(|value| *value as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    mob
}

/// C-5 / BN-07 — `add_updater(call = True)` runs the registration update
/// exactly ONCE. The Reference's registration path calls `update(dt = 0)`
/// and then unconditionally calls `update()` again, so every `call=True`
/// attach double-fires its callable; the corrected engine fires one
/// registration pass per updater and ordinary ticks continue from there.
fn parity_updater_call_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    use std::cell::Cell;
    use std::rc::Rc;

    ctx.set_fps((FPS, 1));
    let mut stage = Stage::new();
    let mob = parity_vmob(&mut stage, &[[0.0; 3]]);
    stage
        .add_to_scene(mob)
        .map_err(|error| fail(format!("add to scene: {error}")))?;
    let dt_mob = parity_vmob(&mut stage, &[[0.0; 3]]);
    stage
        .add_to_scene(dt_mob)
        .map_err(|error| fail(format!("add to scene: {error}")))?;

    let attach_calls = Rc::new(Cell::new(0_u64));
    let dt_attach_calls = Rc::new(Cell::new(0_u64));
    {
        let attach_calls = Rc::clone(&attach_calls);
        stage
            .add_updater(
                mob,
                move |_stage, _target| {
                    attach_calls.set(attach_calls.get() + 1);
                },
                true,
            )
            .map_err(|error| fail(format!("register call updater: {error}")))?;
    }
    {
        let dt_attach_calls = Rc::clone(&dt_attach_calls);
        stage
            .add_dt_updater(
                dt_mob,
                move |_stage, _target, _dt| {
                    dt_attach_calls.set(dt_attach_calls.get() + 1);
                },
                true,
            )
            .map_err(|error| fail(format!("register dt updater: {error}")))?;
    }

    // Each call = true registration has fired its own callable exactly
    // once; the Reference's double-call would read 2 here.
    let calls_after_attach = attach_calls.get();
    let dt_after_attach = dt_attach_calls.get();
    stage.update(0.25);
    stage.update(0.25);
    let calls_after_two_ticks = attach_calls.get();
    let dt_after_two_ticks = dt_attach_calls.get();

    // Each call = true registration has fired its own callable exactly
    // once; the Reference's double-call would read 2 here.
    let corrected = calls_after_attach == 1
        && dt_after_attach == 1
        && calls_after_two_ticks == 3
        && dt_after_two_ticks == 3;
    if !corrected {
        return Err(fail(format!(
            "C-5: add_updater(call=true) attach fired call={calls_after_attach} \
             dt={dt_after_attach} (corrected: exactly one each); after two ticks \
             call={calls_after_two_ticks} dt={dt_after_two_ticks} (corrected: 3 each)"
        )));
    }

    ctx.event(
        LogEvent::new("e2e.parity.updater_call_once")
            .field("call_attach_calls", calls_after_attach)
            .field("dt_attach_calls", dt_after_attach)
            .field("call_after_two_ticks", calls_after_two_ticks)
            .field("dt_after_two_ticks", dt_after_two_ticks),
    );
    ctx.counter("parity_updater_attach_calls", calls_after_attach);
    Ok(RunOutcome::ok().with_counter("parity_updater_attach_calls", calls_after_attach))
}

/// C-17 / BN-07 — `NumberPlane.get_y_unit_size()` measures the Y axis. The
/// Reference's body answers `get_x_axis().get_unit_size()`, invisible on
/// the default square-aspect plane; this plane is deliberately
/// non-square-aspect (equal 8-unit edge lengths over a 16-unit x span and
/// a 4-unit y span), so the correct y answer and the pinned x answer
/// differ by 4×.
fn parity_plane_y_unit_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let corpus = scene_goldens::corpus();
    let plane = NumberPlane::new()
        .x_range([-8.0, 8.0, 1.0])
        .y_range([-2.0, 2.0, 1.0])
        .width(8.0)
        .height(8.0)
        .build(&corpus.book)
        .map_err(|error| fail(format!("plane builds: {error}")))?;
    let x_unit = plane.x_unit_size();
    let y_unit = plane.y_unit_size();

    // Corrected answers: 8.0 units of width over 16 x-units → 0.5 scene
    // units per x unit; 8.0 of height over 4 y-units → 2.0 per y unit.
    if (x_unit - 0.5).abs() > 1e-9 || (y_unit - 2.0).abs() > 1e-9 {
        return Err(fail(format!(
            "C-17: y_unit_size() answered {y_unit} while x_unit_size() answered \
             {x_unit}; the corrected plane measures the axis the name names (2.0, x 0.5)"
        )));
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (x_milli, y_milli) = ((x_unit * 1000.0) as u64, (y_unit * 1000.0) as u64);
    ctx.event(
        LogEvent::new("e2e.parity.plane_y_unit_size")
            .field("x_unit_milli", x_milli)
            .field("y_unit_milli", y_milli),
    );
    ctx.counter("parity_plane_y_unit_is_own_axis", 1);
    Ok(RunOutcome::ok().with_counter("parity_plane_y_unit_is_own_axis", 1))
}

/// C-15 / BN-07 — `FunctionGraph(..., color = …)` styles the graph. The
/// Reference declares `color=YELLOW` but never forwards it, so every
/// explicit color request was silently ignored; the corrected builder
/// strokes the graph in the requested color and the no-color default stays
/// YELLOW.
fn parity_function_graph_color_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let default_graph = FunctionGraph::new(|x: f64| x * x)
        .build()
        .map_err(|error| fail(format!("default graph builds: {error}")))?;
    let requested_graph = FunctionGraph::new(|x: f64| x * x)
        .x_range([-2.0, 2.0, 0.05])
        .color(TEAL_B)
        .build()
        .map_err(|error| fail(format!("styled graph builds: {error}")))?;
    let default_color = default_graph.style().stroke_color;
    let requested_color = requested_graph.style().stroke_color;

    let default_stays_yellow = default_color == YELLOW;
    let requested_honored = requested_color == TEAL_B && requested_color != default_color;
    if !default_stays_yellow || !requested_honored {
        return Err(fail(format!(
            "C-15: FunctionGraph color — default strokes {default_color:?} (want YELLOW), \
             requested strokes {requested_color:?} (want TEAL_B, distinct from the default)"
        )));
    }

    ctx.event(
        LogEvent::new("e2e.parity.function_graph_color")
            .field("default_yellow", truth(default_stays_yellow))
            .field("requested_honored", truth(requested_honored)),
    );
    ctx.counter("parity_function_graph_color_honored", 1);
    Ok(RunOutcome::ok().with_counter("parity_function_graph_color_honored", 1))
}

/// C-16 / BN-07 — the frame-marker constants work as unit directions for
/// `to_edge`/`to_corner`. The pinned markers are frame-radius-scaled while
/// `align_on_border` subtracts `direction * buff` unnormalized, so the
/// markers' only realistic use misplaces by a frame-radius factor; the
/// ruling resolves each marker to its unit direction, and this drill pins
/// the exact flush placements the names promise through the real
/// positional engine.
fn parity_frame_markers_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    // The C-16 resolution: marker → the unit direction it names.
    const MARKERS: [(&str, [f64; 3], [f64; 3]); 4] = [
        ("TOP", TOP, UP),
        ("BOTTOM", BOTTOM, DOWN),
        ("LEFT_SIDE", LEFT_SIDE, LEFT),
        ("RIGHT_SIDE", RIGHT_SIDE, RIGHT),
    ];
    let sign = |value: f64| {
        if value > 0.0 {
            1.0
        } else if value < 0.0 {
            -1.0
        } else {
            0.0
        }
    };
    for (name, marker, direction) in MARKERS {
        for axis in 0..3 {
            if sign(marker[axis]) != sign(direction[axis]) {
                return Err(fail(format!(
                    "C-16: frame marker {name} does not carry the sign of its \
                     resolved direction on axis {axis}"
                )));
            }
        }
    }

    let mut stage = Stage::new();
    // A 2×2 box: bounding box exactly [-1, 1]².
    let mob = parity_vmob(
        &mut stage,
        &[
            [-1.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ],
    );
    stage
        .add_to_scene(mob)
        .map_err(|error| fail(format!("add to scene: {error}")))?;

    // Each resolved direction lands the box exactly `buff` inside the
    // named frame edge; a frame-radius-scaled direction argument would
    // multiply the buffer by the frame radius instead.
    stage.to_edge(mob, UP, 0.5);
    let top = stage.get_bounding_box_point(mob, UP);
    if (top[1] - (FRAME_Y_RADIUS - 0.5)).abs() > 1e-9 {
        return Err(fail(format!(
            "C-16: the top-edge marker direction placed the top at y = {}, want {}",
            top[1],
            FRAME_Y_RADIUS - 0.5
        )));
    }
    stage.to_edge(mob, DOWN, 0.5);
    let bottom = stage.get_bounding_box_point(mob, DOWN);
    if (bottom[1] + (FRAME_Y_RADIUS - 0.5)).abs() > 1e-9 {
        return Err(fail(format!(
            "C-16: the bottom-edge marker direction placed the bottom at y = {}, want {}",
            bottom[1],
            -(FRAME_Y_RADIUS - 0.5)
        )));
    }
    stage.to_edge(mob, LEFT, 0.25);
    let left = stage.get_bounding_box_point(mob, LEFT);
    if (left[0] + (FRAME_X_RADIUS - 0.25)).abs() > 1e-9 {
        return Err(fail(format!(
            "C-16: the left-side marker direction placed the left at x = {}, want {}",
            left[0],
            -(FRAME_X_RADIUS - 0.25)
        )));
    }
    stage.to_edge(mob, RIGHT, 0.75);
    let right = stage.get_bounding_box_point(mob, RIGHT);
    if (right[0] - (FRAME_X_RADIUS - 0.75)).abs() > 1e-9 {
        return Err(fail(format!(
            "C-16: the right-side marker direction placed the right at x = {}, want {}",
            right[0],
            FRAME_X_RADIUS - 0.75
        )));
    }
    stage.to_corner(mob, UL, 0.5);
    let corner_top = stage.get_bounding_box_point(mob, UP);
    let corner_left = stage.get_bounding_box_point(mob, LEFT);
    if (corner_top[1] - (FRAME_Y_RADIUS - 0.5)).abs() > 1e-9
        || (corner_left[0] + (FRAME_X_RADIUS - 0.5)).abs() > 1e-9
    {
        return Err(fail(format!(
            "C-16: the UL corner direction placed the corner at ({}, {}), want ({}, {})",
            corner_left[0],
            corner_top[1],
            -(FRAME_X_RADIUS - 0.5),
            FRAME_Y_RADIUS - 0.5
        )));
    }

    ctx.event(
        LogEvent::new("e2e.parity.frame_markers")
            .field("marker_directions", MARKERS.len() as u64)
            .field("exact_placements", 5_u64),
    );
    ctx.counter("parity_frame_marker_placements", 5);
    Ok(RunOutcome::ok().with_counter("parity_frame_marker_placements", 5))
}

/// BN-03 — `point_from_proportion`/`MoveAlongPath` place by TRUE arc
/// length. Metamorphic: uniform α steps visit equal arc-length increments
/// on an uneven curved path (built corner-free, so equal chords are a
/// faithful stand-in for equal arcs), `MoveAlongPath` rides the same
/// corrected layer end-to-end, and the shipped layer provably disagrees
/// with the Reference's chord heuristic at α = 0.5.
fn parity_move_along_path_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let mut stage = Stage::new();
    // Two very unequal curved quads sharing one tangent-continuous anchor:
    // a short arch (≈1.8 units of arc) then a long shallow one (≈4.5). The
    // join carries no corner, so chord steps track arc steps to well under
    // the 1% band below.
    let path = parity_vmob(
        &mut stage,
        &[
            [0.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.0, 1.5, 0.0],
            [1.5, 2.0, 0.0],
            [5.0, 0.0, 0.0],
        ],
    );
    stage
        .add_to_scene(path)
        .map_err(|error| fail(format!("add to scene: {error}")))?;

    // Metamorphic relation: uniform α steps visit equal arc lengths.
    const STEPS: u32 = 64;
    let mut prev = stage
        .point_from_proportion(path, 0.0)
        .map_err(|error| fail(format!("proportion(0): {error}")))?;
    #[allow(clippy::cast_possible_truncation)]
    let mut chords = Vec::with_capacity(STEPS as usize);
    for i in 1..=STEPS {
        let alpha = f64::from(i) / f64::from(STEPS);
        let point = stage
            .point_from_proportion(path, alpha)
            .map_err(|error| fail(format!("proportion({alpha}): {error}")))?;
        chords.push(((point[0] - prev[0]).powi(2) + (point[1] - prev[1]).powi(2)).sqrt());
        prev = point;
    }
    let mean_step = chords.iter().sum::<f64>() / f64::from(STEPS);
    let worst_deviation = chords
        .iter()
        .fold(0.0_f64, |w, c| w.max((c - mean_step).abs()));
    if mean_step <= 0.0 || worst_deviation / mean_step > 0.01 {
        return Err(fail(format!(
            "BN-03: uniform α steps are not equal arc length — mean step {mean_step}, \
             worst deviation {worst_deviation} ({:.3}%, allowed 1%)",
            100.0 * worst_deviation / mean_step
        )));
    }

    // The Reference's equal-curve-length heuristic would land α = 0.5
    // exactly at the shared anchor (x = 1): two curves, equal shares. True
    // arc length sits deep in the long second curve, while α = 0.25 is
    // still inside the short first curve.
    let quarter = stage
        .point_from_proportion(path, 0.25)
        .map_err(|error| fail(format!("proportion(0.25): {error}")))?;
    if quarter[0] >= 1.0 {
        return Err(fail(format!(
            "BN-03: α = 0.25 landed at x = {} — outside the short first curve, \
             whose arc is well under a quarter of the path",
            quarter[0]
        )));
    }
    let half = stage
        .point_from_proportion(path, 0.5)
        .map_err(|error| fail(format!("proportion(0.5): {error}")))?;
    if half[0] <= 1.25 {
        return Err(fail(format!(
            "BN-03: α = 0.5 landed at x = {} — the chord-length boundary answer, \
             not true arc length",
            half[0]
        )));
    }
    // Direct discriminator against the labeled heuristic on the same
    // records: the shipped layer and `quick_point_from_proportion` must
    // disagree decisively at α = 0.5.
    let records = stage
        .get_points(path)
        .ok_or_else(|| fail("the path lost its points"))?;
    let quad =
        fmn_geom::QuadPath::from_points(records).map_err(|error| fail(format!("{error:?}")))?;
    let quick = quad
        .quick_point_from_proportion(0.5)
        .ok_or_else(|| fail("quick proportion(0.5)"))?;
    let heuristic_gap = ((half[0] - quick[0]).powi(2) + (half[1] - quick[1]).powi(2)).sqrt();
    if heuristic_gap <= 0.5 {
        return Err(fail(format!(
            "BN-03: true arc length and the chord heuristic agree at α = 0.5 \
             (gap {heuristic_gap}) — the drill no longer discriminates"
        )));
    }

    // MoveAlongPath rides the corrected layer end-to-end: its interpolated
    // positions match the proportion layer at every α.
    let dot = parity_vmob(&mut stage, &[[9.0, 9.0, 0.0]]);
    let mut animation = MoveAlongPath::new(dot, path);
    animation.state_mut().config.rate_func = RateFunc::linear();
    animation
        .begin(&mut stage)
        .map_err(|error| fail(format!("MoveAlongPath begin: {error}")))?;
    let mut animation_matches = 0_u64;
    for &alpha in &[0.25_f64, 0.5, 0.75] {
        animation.interpolate(&mut stage, alpha);
        let at = *stage
            .get_points(dot)
            .ok_or_else(|| fail("the dot lost its points"))?
            .first()
            .ok_or_else(|| fail("the dot lost its point"))?;
        let want = stage
            .point_from_proportion(path, alpha)
            .map_err(|error| fail(format!("proportion({alpha}): {error}")))?;
        if (at[0] - want[0]).abs() > 1e-3 || (at[1] - want[1]).abs() > 1e-3 {
            return Err(fail(format!(
                "BN-03: MoveAlongPath at α={alpha} sat at ({}, {}), want ({}, {})",
                at[0], at[1], want[0], want[1]
            )));
        }
        animation_matches += 1;
    }

    ctx.event(
        LogEvent::new("e2e.parity.move_along_path_arclength")
            .field("equal_steps", truth(worst_deviation / mean_step <= 0.01))
            .field("animation_matches", animation_matches),
    );
    ctx.counter("parity_arclength_animation_matches", animation_matches);
    Ok(RunOutcome::ok().with_counter("parity_arclength_animation_matches", animation_matches))
}

/// §11.3 span map — `tex_to_color_map` colors by SOURCE identity. The
/// constructor map rides `Typeset::occurrences`: two source occurrences of
/// one substring get independent fates, the longer overlapping entry wins
/// exactly its own span, unmapped glyphs keep the default white, and each
/// mapped ordinal is the glyph of the matched source span (the
/// two-render-and-align hack stays dead).
fn parity_tex_span_map_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((FPS, 1));
    let corpus = scene_goldens::corpus();
    let t2c = [("x", RED_C), ("x^2", TEAL_B), ("y", BLUE_C)];
    let built = fmn_library::Tex::new("x + x^2 + y")
        .t2c(&t2c)
        .build(&corpus.tex)
        .map_err(|error| fail(format!("tex builds: {error}")))?;

    let fill_of = |ord: usize| built.vmob.children()[ord].style().fill_color;
    let all_fill = |ords: &[usize], color: Srgb| ords.iter().all(|&ord| fill_of(ord) == color);

    // Two distinct source occurrences of "x" with independent fates: the
    // standalone x is RED, the x inside the later "x^2" span is TEAL_B.
    let x_occurrences = built.occurrences("x");
    if x_occurrences.len() != 2 {
        return Err(fail(format!(
            "the span map found {} occurrences of \"x\", want 2",
            x_occurrences.len()
        )));
    }
    if !all_fill(&x_occurrences[0], RED_C) || !all_fill(&x_occurrences[1], TEAL_B) {
        return Err(fail(
            "t2c did not give the two \"x\" occurrences their independent mapped fills",
        ));
    }
    // The longer overlapping entry wins exactly its own span: both glyphs
    // of the second term, and nothing else.
    let x_squared = built.occurrences("x^2");
    if x_squared.len() != 1 || x_squared[0].len() != 2 || !all_fill(&x_squared[0], TEAL_B) {
        return Err(fail(format!(
            "the \"x^2\" span mapped {} occurrence(s) with coverage {:?}",
            x_squared.len(),
            x_squared.first()
        )));
    }
    // "y" maps by identity; unmapped glyphs keep the default white.
    let y = built.occurrences("y");
    let plus = built.occurrences("+");
    if y.len() != 1 || !all_fill(&y[0], BLUE_C) {
        return Err(fail("the mapped \"y\" glyph is not BLUE_C"));
    }
    if plus.len() != 2 || !plus.iter().all(|ords| all_fill(ords, WHITE)) {
        return Err(fail(
            "unmapped \"+\" glyphs must keep the default white fill",
        ));
    }
    // The mapped ordinal is the glyph of the matched source span — the
    // span map, not a render-and-align assignment.
    let first_x = x_occurrences[0][0];
    let is_x_glyph = matches!(
        built.typeset.subs[first_x].prim,
        Prim::Glyph(glyph) if built.typeset.layout.glyphs[glyph].ch == 'x'
    );
    if !is_x_glyph {
        return Err(fail(
            "the ordinal selected by the span map is not the glyph of the matched source span",
        ));
    }

    ctx.event(
        LogEvent::new("e2e.parity.tex_span_map_t2c")
            .field("x_occurrences", x_occurrences.len() as u64)
            .field("plus_occurrences", plus.len() as u64),
    );
    ctx.counter("parity_t2c_source_identity", 1);
    Ok(RunOutcome::ok().with_counter("parity_t2c_source_identity", 1))
}

/// §11.7 — Markdown composes `$…$` through fmd-math under fmd's own
/// delimiter rule. An inline island is exactly the text-style `Tex` of its
/// source, and prose dollars (`$5 and $10`) lay out as their escaped spelling
/// rather than as italic mathematics.
fn parity_markdown_inline_math_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    fn relative(v: &VMobject, points: &mut Vec<[f64; 3]>) {
        points.extend_from_slice(v.points());
        for child in v.children() {
            relative(child, points);
        }
    }
    // Shapes compare up to translation; placing a formula inside a line
    // shifts it, and re-relativizing rounds below 1e-10 scene units.
    let same_shape = |a: &VMobject, b: &VMobject| {
        let (mut a_points, mut b_points) = (Vec::new(), Vec::new());
        relative(a, &mut a_points);
        relative(b, &mut b_points);
        let (Some(a0), Some(b0)) = (a_points.first().copied(), b_points.first().copied()) else {
            return a_points.is_empty() && b_points.is_empty();
        };
        a_points.len() == b_points.len()
            && a_points
                .iter()
                .zip(&b_points)
                .all(|(p, q)| (0..3).all(|i| ((p[i] - a0[i]) - (q[i] - b0[i])).abs() < 1e-10))
    };
    let corpus = scene_goldens::corpus();
    let build = |source: &str| {
        fmn_library::markdown::Markdown::new(source)
            .build_with_math(&corpus.book, &corpus.tex)
            .map_err(|error| fail(format!("Markdown {source:?} builds: {error}")))
    };

    let doc = build("area $x_i+y_j$ here")?;
    let words = doc.blocks[0].vmob.children();
    let formula = fmn_library::Tex::new("x_i+y_j")
        .math_style(fmn_tex::Style::Text)
        .build(&corpus.tex)
        .map_err(|error| fail(format!("tex builds: {error}")))?;
    let island = words.len() == 3 && same_shape(&words[1], &formula.vmob);
    let prose = ["It costs $5 and $10 today.", "a $ x$ b", "a $x $ b"];
    let mut literal = 0u64;
    for source in prose {
        if same_shape(
            &build(source)?.vmob,
            &build(&source.replace('$', r"\$"))?.vmob,
        ) {
            literal += 1;
        }
    }
    ctx.event(
        LogEvent::new("e2e.parity.markdown_inline_math")
            .field("island_is_text_style_tex", island)
            .field("prose_sources", prose.len() as u64)
            .field("prose_sources_literal", literal),
    );
    if !island {
        return Err(fail(
            "the inline island is not the text-style Tex of its source between two words",
        ));
    }
    if literal != prose.len() as u64 {
        return Err(fail(format!(
            "{literal} of {} prose-dollar sources laid out as their escaped spelling",
            prose.len()
        )));
    }
    ctx.counter("parity_markdown_prose_dollars", literal);
    Ok(RunOutcome::ok().with_counter("parity_markdown_prose_dollars", literal))
}

// ---------------------------------------------------------------------------
// The long-form regime: the Hoeffding's D explainer
// ---------------------------------------------------------------------------

/// The explainer's background as RGB8 (`kit::BACKGROUND`, `#RRGGBB`).
fn hoeffding_background() -> Result<[u8; 3], ScenarioError> {
    let hex = kit::BACKGROUND.trim_start_matches('#');
    let channel = |at: usize| {
        hex.get(at..at + 2)
            .and_then(|digits| u8::from_str_radix(digits, 16).ok())
            .ok_or_else(|| fail(format!("background {:?} is not #RRGGBB", kit::BACKGROUND)))
    };
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

/// The worked example's statistics, pinned before any chapter renders: the
/// article's Q, D1/D2/D3 and D = 0.4107142857.
fn hoeffding_self_check(ctx: &mut RunCtx) -> Result<(), ScenarioError> {
    let worked = stats::hoeffding(&stats::HEIGHTS, &stats::WEIGHTS);
    let pinned = worked.q == [1.0, 2.0, 4.0, 4.0, 7.0, 3.0, 4.0, 8.5, 8.5, 8.5]
        && (worked.d1, worked.d2, worked.d3) == (196.25, 10696.0, 1329.5)
        && (worked.d - 0.410_714_285_714_285_7).abs() < 1e-15;
    ctx.event(
        LogEvent::new("e2e.hoeffding.self_check")
            .field("d", format!("{:.10}", worked.d))
            .field("pinned", truth(pinned)),
    );
    if !pinned {
        return Err(fail(format!(
            "Hoeffding's D left the worked example: {worked:?}"
        )));
    }
    ctx.counter("hoeffding_self_check", 1);
    Ok(())
}

/// Peak resident set size of this process in KiB (Linux `VmHWM`; 0 where
/// unavailable). Calibration data only, never a gate.
fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("VmHWM:"))
                .and_then(|rest| rest.trim().trim_end_matches("kB").trim().parse().ok())
        })
        .unwrap_or(0)
}

/// Fast-tier geometry: 160x90 at 2 fps (566 frames, 84 s unoptimized). Every
/// play and updater still runs; at 10 fps the unoptimized test profile took
/// 327 s for 2,572 frames, too much for the per-commit tier.
const HOEFFDING_FAST: (u32, u32, u32) = (160, 90, 2);

/// fm-longform-native-e2e-hoeffding-l59d, fast tier: the long-form regime.
/// All eight chapters of the Hoeffding's D explainer (dozens of plays each,
/// `always_redraw` readouts recomputing D from live dot positions every
/// frame, `TransformMatchingTex`, 150-dot `Transform` morphs) render silently
/// through `fmn::render` to PNG sequences. Each chapter publishes one frame
/// per clock frame; its last frame is the bare background, because every
/// chapter closes by fading everything out; and nine in ten of its frames
/// carry ink, so no chapter plays to an empty screen.
fn hoeffding_d_chapters_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    hoeffding_self_check(ctx)?;
    let (width, height, fps) = HOEFFDING_FAST;
    let background = hoeffding_background()?;
    let mut total_frames = 0;
    let mut chapters_clean = 0;
    for entry in chapters::registry() {
        let started = std::time::Instant::now();
        let kit = kit::Kit::new().map_err(|error| fail(format!("kit: {error}")))?;
        let mut scene = (entry.make)(kit);
        let root = format!("/hoeffding/{}", entry.key);
        let mut options = fmn::RenderOptions::with_format(&root, fmn::RenderFormat::PngSequence)
            .map_err(|error| fail(format!("bundled config: {error}")))?;
        options.config.camera.resolution = (width, height);
        options.config.camera.fps = fps;
        options.config.camera.background_color = kit::BACKGROUND.to_owned();
        options.typeset_cache = false;
        let fs = Arc::new(fmn_platform::fs::VirtualFs::new());
        let report = fmn::render_with_fs(scene.as_mut(), options, fs.clone())
            .map_err(|error| fail(format!("{}: {error}", entry.key)))?;
        let frames = report.artifact.frame_count;
        let clock_frames = u64::try_from(report.scene.time.frames()).unwrap_or(u64::MAX);
        let mut inked = 0_u64;
        let mut last_inked = true;
        for index in 0..frames {
            let path = PathBuf::from(format!("{root}/frame_{index:06}.png"));
            let bytes = fs
                .read(&path)
                .map_err(|error| fail(format!("read {}: {error}", path.display())))?;
            let image = fmn_codec::decode_png(&bytes, &fmn_codec::PngLimits::default())
                .map_err(|error| fail(format!("decode {}: {error}", path.display())))?;
            let ink = image.rgba.as_chunks::<4>().0.iter().any(|pixel| {
                pixel[..3]
                    .iter()
                    .zip(background)
                    .any(|(&channel, ground)| channel.abs_diff(ground) > 8)
            });
            inked += u64::from(ink);
            last_inked = ink;
        }
        let clean = frames > 0 && frames == clock_frames && !last_inked && inked * 10 >= frames * 9;
        ctx.event(
            LogEvent::new("e2e.hoeffding.chapter")
                .field("chapter", entry.key)
                .field("frames", frames)
                .field("clock_frames", clock_frames)
                .field("plays", report.scene.play_count)
                .field("inked_frames", inked)
                .field("last_frame_blank", truth(!last_inked))
                .field("wall_ms", started.elapsed().as_millis().to_string())
                .field("clean", truth(clean)),
        );
        if !clean {
            return Err(fail(format!(
                "{}: {frames} frames for {clock_frames} clock frames, {inked} inked, last inked: {last_inked}",
                entry.key
            )));
        }
        total_frames += frames;
        chapters_clean += 1;
    }
    ctx.counter("hoeffding_chapters", chapters_clean);
    ctx.counter("hoeffding_frames", total_frames);
    Ok(RunOutcome::ok()
        .with_counter("hoeffding_self_check", 1)
        .with_counter("hoeffding_chapters", chapters_clean)
        .with_counter("hoeffding_frames", total_frames))
}

/// fm-longform-native-e2e-hoeffding-l59d, the second front door: two of the
/// explainer's segments written as ordinary `manimlib` scene code
/// (crates/fmn-python/tests/hoeffding_portal.py, executed unedited: NumPy
/// data, `always_redraw` readouts recomputing D from live dot centres,
/// `ValueTracker.animate`, `LaggedStartMap`, `ShowPassingFlash`, `Axes.c2p`)
/// render through the portal's production route at 160x90, 4 fps. Each
/// publishes one PNG per reported frame, nine in ten of them show more than
/// the background, and the last does (both scenes end on a full picture).
fn hoeffding_d_portal_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let root = scenario_dir("hoeffding_d_portal")?;
    let scenes = manimlib::run_portal_gauntlet_hoeffding(&root, (160, 90), 4)
        .map_err(|error| fail(format!("Hoeffding portal segments: {error}")))?;
    let mut total_frames = 0;
    let mut scenes_clean = 0;
    for (scene, frames) in &scenes {
        let directory = root.join(scene);
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)
            .map_err(|error| fail(format!("read {}: {error}", directory.display())))?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|error| fail(format!("read {scene} directory entry: {error}")))
            })
            .collect::<Result<_, _>>()?;
        paths.retain(|path| path.extension().is_some_and(|extension| extension == "png"));
        paths.sort();
        let mut drawn = 0_u64;
        let mut last_drawn = false;
        for path in &paths {
            let bytes = std::fs::read(path)
                .map_err(|error| fail(format!("read {}: {error}", path.display())))?;
            let image = fmn_codec::decode_png(&bytes, &fmn_codec::PngLimits::default())
                .map_err(|error| fail(format!("decode {}: {error}", path.display())))?;
            // Drawn: some pixel departs from the frame's corner (background).
            let pixels = image.rgba.as_chunks::<4>().0;
            let corner = pixels.first().copied().unwrap_or_default();
            last_drawn = pixels.iter().any(|pixel| {
                pixel[..3]
                    .iter()
                    .zip(&corner[..3])
                    .any(|(a, b)| a.abs_diff(*b) > 8)
            });
            drawn += u64::from(last_drawn);
        }
        let published = paths.len() as u64;
        let clean = *frames > 0 && published == *frames && last_drawn && drawn * 10 >= frames * 9;
        ctx.event(
            LogEvent::new("e2e.hoeffding.portal_scene")
                .field("scene", scene.as_str())
                .field("frames", *frames)
                .field("published", published)
                .field("drawn_frames", drawn)
                .field("last_frame_drawn", truth(last_drawn))
                .field("clean", truth(clean)),
        );
        if !clean {
            return Err(fail(format!(
                "{scene}: {frames} frames reported, {published} published, {drawn} drawn, last drawn: {last_drawn}"
            )));
        }
        total_frames += frames;
        scenes_clean += 1;
    }
    ctx.counter("hoeffding_portal_scenes", scenes_clean);
    ctx.counter("hoeffding_portal_frames", total_frames);
    Ok(RunOutcome::ok()
        .with_counter("hoeffding_portal_scenes", scenes_clean)
        .with_counter("hoeffding_portal_frames", total_frames))
}

/// fm-longform-native-e2e-hoeffding-l59d, full tier (`FMN_E2E_FULL=1`; run it
/// under `--release`): the whole explainer at 1920x1080, 60 fps, through the
/// host's ffmpeg to one MP4 per chapter with the procedural score mixed
/// natively. Each chapter's video and soundtrack agree to within one frame,
/// its score cues mix without clipping, and the encode names the hashed
/// ffmpeg it ran. Wall time and peak RSS are logged as calibration data,
/// never gated. The MP4s stay in the scenario directory the log names.
fn hoeffding_d_full_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    hoeffding_self_check(ctx)?;
    let dir = scenario_dir("hoeffding_d_full")?;
    let audio = dir.join("audio");
    std::fs::create_dir_all(&audio)
        .map_err(|error| fail(format!("create {}: {error}", audio.display())))?;
    let started = std::time::Instant::now();
    let mut total_frames = 0;
    let mut chapters_clean = 0;
    for entry in chapters::registry() {
        let chapter_started = std::time::Instant::now();
        let mut kit = kit::Kit::new().map_err(|error| fail(format!("kit: {error}")))?;
        kit.score = Some(sound::Score {
            dir: audio.clone(),
            under_voice: false,
        });
        let mut scene = (entry.make)(kit);
        let mut options = fmn::RenderOptions::with_format(
            dir.join(format!("{}.mp4", entry.key)),
            fmn::RenderFormat::Mp4,
        )
        .map_err(|error| fail(format!("bundled config: {error}")))?;
        options.config.camera.resolution = (1920, 1080);
        options.config.camera.fps = 60;
        options.config.camera.background_color = kit::BACKGROUND.to_owned();
        options.typeset_cache = false;
        options.ffmpeg = Some(fmn::FfmpegCapability {
            runner: Arc::new(fmn_platform::process::StdProcessRunner),
            locator: Arc::new(fmn_platform::process::StdFfmpegLocator::from_host_path()),
            // The boundary refuses a workdir under a group-writable ancestor,
            // so its private jobs stay in the system temp dir.
            workdir_root: std::env::temp_dir(),
        });
        let report =
            fmn::render_with_fs(scene.as_mut(), options, Arc::new(fmn_platform::fs::StdFs))
                .map_err(|error| fail(format!("{}: {error}", entry.key)))?;
        let frames = report.artifact.frame_count;
        let clock_frames = u64::try_from(report.scene.time.frames()).unwrap_or(u64::MAX);
        let soundtrack = report.artifact.soundtrack.as_ref();
        // Durations as exact rationals: |samples/rate - frames/fps| <= 1/fps.
        let drift_within_a_frame = soundtrack.is_some_and(|track| {
            let audio = u128::from(track.sample_frames) * 60;
            let video = u128::from(frames) * u128::from(track.sample_rate);
            audio.abs_diff(video) <= u128::from(track.sample_rate)
        });
        let cues = soundtrack.map_or(0, |track| track.cues_mixed);
        let clipped = soundtrack.map_or(0, |track| track.clipped_samples);
        let fingerprint = report
            .artifact
            .ffmpeg
            .first()
            .map(|invocation| invocation.provenance.tool_sha256_hex.clone())
            .unwrap_or_default();
        let clean = frames > 0
            && frames == clock_frames
            && drift_within_a_frame
            && cues > 0
            && clipped == 0
            && fingerprint.len() == 64;
        ctx.event(
            LogEvent::new("e2e.hoeffding.full_chapter")
                .field("chapter", entry.key)
                .field("frames", frames)
                .field("clock_frames", clock_frames)
                .field("bytes", report.artifact.bytes)
                .field("cues_mixed", cues as u64)
                .field("clipped_samples", clipped)
                .field("av_within_one_frame", truth(drift_within_a_frame))
                .field("ffmpeg_sha256", fingerprint.as_str())
                .field("path", report.artifact.path.display().to_string())
                .field("wall_ms", chapter_started.elapsed().as_millis().to_string())
                .field("clean", truth(clean)),
        );
        if !clean {
            return Err(fail(format!(
                "{}: {frames} frames ({clock_frames} on the clock), soundtrack {soundtrack:?}, ffmpeg {fingerprint:?}",
                entry.key
            )));
        }
        total_frames += frames;
        chapters_clean += 1;
    }
    ctx.event(
        LogEvent::new("e2e.hoeffding.full_calibration")
            .field("frames", total_frames)
            .field("wall_ms", started.elapsed().as_millis().to_string())
            .field("peak_rss_kib", peak_rss_kib())
            .field("dir", dir.display().to_string()),
    );
    ctx.counter("hoeffding_full_chapters", chapters_clean);
    Ok(RunOutcome::ok()
        .with_counter("hoeffding_self_check", 1)
        .with_counter("hoeffding_full_chapters", chapters_clean))
}

// ---------------------------------------------------------------------------
// The catalog
// ---------------------------------------------------------------------------

/// A fast-tier scenario with the NDJSON schema assertion folded in:
/// every scenario's captured log validates against the harness's own
/// schema on every run, not just in a dedicated drill. Chain `.tier(...)`
/// and `.regression(...)` at the call site for full-tier rows and drills.
fn spec(
    name: &'static str,
    class: ScenarioClass,
    surface: Surface,
    invocation: Invocation,
    mut assertions: Vec<Assertion>,
    logs: Vec<LogExpect>,
) -> ScenarioSpec {
    assertions.push(Assertion::NdjsonSchema);
    ScenarioSpec::new(name, class, surface, invocation)
        .assertions(assertions)
        .logs(logs)
}

/// The seed scenario catalog: every registered e2e scenario, as data.
#[must_use]
pub fn catalog() -> Vec<ScenarioSpec> {
    let mut specs = vec![
        spec(
            "lifecycle.install_shared_family.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(lifecycle_install_shared_family_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("install_private_reclaimed", 1),
                counter_eq("install_shared_preserved", 1),
                counter_eq("install_rooted_preserved", 1),
                counter_eq("install_roots_preserved", 1),
                counter_eq("install_replacement_geometry", 1),
                counter_eq("install_updater_calls", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.lifecycle.install_shared_family",
                vec![
                    FieldPred::str_eq("private_reclaimed", "true"),
                    FieldPred::str_eq("shared_preserved", "true"),
                    FieldPred::str_eq("rooted_preserved", "true"),
                    FieldPred::str_eq("roots_preserved", "true"),
                    FieldPred::str_eq("replacement_geometry", "true"),
                    FieldPred::u64_eq("updater_calls", 1),
                ],
            )],
        )
        .tier(Tier::Fast),
    ];
    specs.push(spec(
        "render_matrix.coordinate_placement.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(coordinate_placement_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("coordinate_endpoints", 1),
            counter_eq("coordinate_inverse", 1),
            counter_eq("coordinate_rendered_match", 1),
            counter_eq("coordinate_rejects_unmoved_graph", 1),
            counter_eq("coordinate_visible", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.render_matrix.coordinate_placement",
            vec![
                FieldPred::str_eq("endpoints", "true"),
                FieldPred::str_eq("rendered_match", "true"),
                FieldPred::str_eq("rejects_unmoved_graph", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.live_coordinate_animation.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(live_coordinate_animation_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("live_coordinate_frames", 2),
            counter_eq("live_coordinate_follows_frames", 1),
            counter_eq("live_coordinate_frozen_unchanged", 1),
            counter_eq("live_coordinate_inverse", 1),
            counter_eq("live_coordinate_endpoint", 1),
            counter_eq("live_coordinate_rendered_match", 1),
            counter_eq("live_coordinate_visible", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.render_matrix.live_coordinate_animation",
            vec![
                FieldPred::str_eq("follows_frames", "true"),
                FieldPred::str_eq("frozen_unchanged", "true"),
                FieldPred::str_eq("rendered_match", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.live_coordinate_surface.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(live_coordinate_surface_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("live_surface_vertices", 25),
            counter_eq("live_surface_matches", 1),
            counter_eq("live_surface_rejects_unmoved", 1),
            counter_eq("live_surface_visible", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.render_matrix.live_coordinate_surface",
            vec![
                FieldPred::str_eq("matches", "true"),
                FieldPred::str_eq("rejects_unmoved", "true"),
                FieldPred::str_eq("visible", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.bundle_render_state.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_bundle_render_state_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("bundle_render_state_frames", 4),
            counter_eq("bundle_render_state_pure", 1),
            counter_eq("bundle_render_state_recorded", 1),
            counter_eq("bundle_render_state_matches", 4),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.bundle_render_state",
            vec![
                FieldPred::str_eq("nominated_pure", "true"),
                FieldPred::str_eq("recorded", "true"),
                FieldPred::str_eq("equal", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.traced_path_cadence.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_traced_path_cadence_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("traced_path_frames", 8),
            counter_eq("traced_path_source_calls", 8),
            counter_eq("traced_path_starts_empty", 1),
            counter_eq("traced_path_first_observation", 1),
            counter_eq("traced_path_cadence_and_window", 1),
            counter_eq("traced_path_tapers", 1),
            counter_eq("traced_path_stateful", 1),
            counter_eq("traced_path_visible", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.traced_path_cadence",
            vec![
                FieldPred::str_eq("cadence_and_window", "true"),
                FieldPred::str_eq("stateful", "true"),
                FieldPred::str_eq("visible", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.owned_tail_branches.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_owned_tail_branches_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("owned_tail_source_remapped", 1),
            counter_eq("owned_tail_independent_history", 1),
            counter_eq("owned_tail_original_history", 1),
            counter_eq("owned_tail_snapshot_frozen", 1),
            counter_eq("owned_tail_snapshot_replay", 1),
            counter_eq("owned_tail_visible", 1),
            counter_eq("owned_tail_distinct_frames", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.owned_tail_branches",
            vec![
                FieldPred::str_eq("source_remapped", "true"),
                FieldPred::str_eq("snapshot_frozen", "true"),
                FieldPred::str_eq("snapshot_replay", "true"),
                FieldPred::str_eq("distinct_frames", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.succession_reverse_seek.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_succession_reverse_seek_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("succession_frames", 8),
            counter_eq("succession_checkpoint_replay", 1),
            counter_eq("succession_returns_to_start", 1),
            counter_eq("succession_moves", 1),
            counter_eq("succession_visible", 1),
            counter_eq("succession_seek_matches", 8),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.succession_reverse_seek",
            vec![
                FieldPred::str_eq("checkpoint_replay", "true"),
                FieldPred::str_eq("returns_to_start", "true"),
                FieldPred::str_eq("equal", "true"),
            ],
        )],
    ));
    specs.push(
        spec(
            "lifecycle.always_redraw_shared_source.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(lifecycle_always_redraw_shared_source_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("redraw_factory_calls", 4),
                counter_eq("redraw_source_alive", 1),
                counter_eq("redraw_other_parent_alive", 1),
                counter_eq("redraw_shared_links", 1),
                counter_eq("redraw_current_geometry", 1),
                counter_eq("redraw_roots_preserved", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.lifecycle.always_redraw_shared_source",
                vec![
                    FieldPred::u64_eq("factory_calls", 4),
                    FieldPred::str_eq("source_alive", "true"),
                    FieldPred::str_eq("other_parent_alive", "true"),
                    FieldPred::str_eq("shared_links", "true"),
                    FieldPred::str_eq("current_geometry", "true"),
                    FieldPred::str_eq("roots_preserved", "true"),
                ],
            )],
        )
        .tier(Tier::Fast),
    );
    specs.push(spec(
        "lifecycle.tracing_tail_empty_until_update.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_tracing_tail_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("trace_initial_points", 0),
            counter_eq("trace_zero_dt_points", 0),
            counter_ge("trace_updated_points", 3),
            counter_eq("trace_seed_preserved", 1),
            counter_eq("trace_endpoint_matches", 1),
            counter_eq("trace_tapers_preserved", 1),
            counter_eq("trace_stateful", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.tracing_tail",
            vec![
                FieldPred::u64_eq("initial_points", 0),
                FieldPred::u64_eq("zero_dt_points", 0),
                FieldPred::str_eq("seed_preserved", "true"),
                FieldPred::str_eq("endpoint_matches", "true"),
                FieldPred::str_eq("tapers_preserved", "true"),
                FieldPred::str_eq("stateful", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.typeset_cache_warm_second_run.v1",
        ScenarioClass::LifecycleDrill,
        Surface::CliInProcess,
        Invocation::new(typeset_cache_warm_second_run),
        vec![
            counter_eq("typeset_cold_misses", FORMULA_SHEET_LAYOUTS),
            counter_eq("typeset_warm_disk_hits", FORMULA_SHEET_LAYOUTS),
            counter_eq("typeset_warm_misses", 0),
            counter_eq("typeset_hit_equals_miss", 1),
        ],
        vec![
            LogExpect::span_present(
                "e2e.typeset.cache",
                vec![
                    FieldPred::str_eq("run", "warm"),
                    FieldPred::u64_eq("misses", 0),
                    FieldPred::str_eq("persistent", "true"),
                ],
            ),
            LogExpect::span_present(
                "e2e.typeset.cache_bits",
                vec![FieldPred::str_eq("hit_equals_miss", "true")],
            ),
        ],
    ));
    specs.push(spec(
        "lifecycle.typeset_preflight_before_first_play.v1",
        ScenarioClass::LifecycleDrill,
        Surface::CliInProcess,
        Invocation::new(typeset_preflight_before_first_play),
        vec![
            counter_eq("typeset_preflight_requests", 20),
            counter_eq("typeset_layouts_inside_play", 0),
            counter_eq("preflight_before_first_frame", 1),
        ],
        vec![LogExpect::span_present(
            e2e::spans::PREFLIGHT,
            vec![
                FieldPred::u64_eq("requests", 20),
                FieldPred::u64_eq("layouts_in_construct", 0),
                FieldPred::u64_eq("layouts_inside_play", 0),
                FieldPred::str_eq("before_first_frame", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.cli_doctor_capabilities.v1",
        ScenarioClass::LifecycleDrill,
        Surface::CliInProcess,
        Invocation::new(cli_doctor_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("doctor_records", 7),
            counter_eq("doctor_human_sections", 10),
            counter_eq("doctor_required_ffmpeg_exit", 4),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.doctor",
            vec![
                FieldPred::u64_eq("records", 7),
                FieldPred::u64_eq("human_sections", 10),
                FieldPred::u64_eq("required_ffmpeg_exit", 4),
                FieldPred::str_eq("read_only", "true"),
            ],
        )],
    ));

    // RENDER-MATRIX: every preset × native sink × engine class.
    for preset in PRESETS {
        for sink_kind in [SinkKind::Y4m, SinkKind::PngSequence] {
            for certified in [true, false] {
                specs.push(matrix_spec(preset, sink_kind, certified, None));
            }
        }
    }
    specs.push(spec(
        "render_matrix.cli_builtin_png_sequence.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_builtin_render_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_render_frames", 3),
            counter_eq("cli_png_signature", 1),
            counter_eq("cli_manifest", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.render",
            vec![
                FieldPred::str_eq("source", "builtin"),
                FieldPred::str_eq("format", "png_sequence"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("png_signature", "true"),
                FieldPred::str_eq("manifest_complete", "true"),
                FieldPred::str_eq("manifest_valid", "true"),
                FieldPred::str_eq("robot_record", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.cli_camera_builtins.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_camera_builtins_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_camera_builtins", 4),
            Assertion::GoldenLock {
                suite: E2E_SUITE,
                scope: Scope::Certified,
            },
        ],
        vec![LogExpect::span_present(
            "e2e.cli.camera",
            vec![
                FieldPred::str_eq("camera_route", "true"),
                FieldPred::str_eq("animated", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.png_sequence_frame_concurrency.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(png_sequence_frame_concurrency_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("png_concurrency_identical", 1),
            counter_eq("png_encode_spans", 12),
        ],
        vec![LogExpect::span_present(
            "e2e.png.frame_concurrency",
            vec![FieldPred::str_eq("identical", "true")],
        )],
    ));
    specs.push(spec(
        "render_matrix.hoeffding_d_chapters.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(hoeffding_d_chapters_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("hoeffding_self_check", 1),
            counter_eq("hoeffding_chapters", 8),
            // The explainer's pacing on the clock: plays and waits summed.
            counter_eq("hoeffding_frames", 566),
        ],
        vec![LogExpect::span_present(
            "e2e.hoeffding.self_check",
            vec![FieldPred::str_eq("d", "0.4107142857")],
        )],
    ));
    specs.push(spec(
        "render_matrix.hoeffding_d_portal.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(hoeffding_d_portal_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("hoeffding_portal_scenes", 2),
        ],
        vec![LogExpect::span_present(
            "e2e.hoeffding.portal_scene",
            vec![FieldPred::str_eq("clean", "true")],
        )],
    ));
    specs.push(
        spec(
            "render_matrix.hoeffding_d_full_mp4.v1",
            ScenarioClass::RenderMatrix,
            Surface::RustApi,
            Invocation::new(hoeffding_d_full_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("hoeffding_self_check", 1),
                counter_eq("hoeffding_full_chapters", 8),
            ],
            vec![LogExpect::span_present(
                "e2e.hoeffding.full_calibration",
                vec![],
            )],
        )
        .tier(Tier::Full),
    );
    specs.push(spec(
        "render_matrix.cli_primitive_builtins.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_primitive_builtins_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_primitive_builtins", 25),
            Assertion::GoldenLock {
                suite: E2E_SUITE,
                scope: Scope::Certified,
            },
        ],
        vec![LogExpect::span_present(
            "e2e.cli.primitive",
            vec![FieldPred::str_eq("retained_route", "true")],
        )],
    ));
    specs.push(spec(
        "determinism.cli_builtins_threads_1_4.v1",
        ScenarioClass::DeterminismDrill,
        Surface::CliInProcess,
        Invocation::new(cli_builtins_threads_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_builtins_threads_identical", 29),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.builtin_threads",
            vec![FieldPred::str_eq("threads_1_4_identical", "true")],
        )],
    ));
    specs.push(spec(
        "render_matrix.studio_builtin_preview.v1",
        ScenarioClass::RenderMatrix,
        Surface::StudioInProcess,
        Invocation::new(studio_preview_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["studio_frame.png".to_owned()]),
            counter_eq("studio_preview_frames", 1),
            counter_eq("studio_png_dimensions", 1),
            counter_eq("studio_backend_journaled", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.studio.preview",
            vec![
                FieldPred::str_eq("scene", "circle_shift.v1"),
                FieldPred::u64_eq("frame", 1),
                FieldPred::str_eq("encoding", "png"),
                FieldPred::str_eq("backend_is_stream", "true"),
                FieldPred::str_eq("dimensions_match", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.studio_camera_preview.v1",
        ScenarioClass::RenderMatrix,
        Surface::StudioInProcess,
        Invocation::new(studio_camera_preview_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["studio_camera_frame.png".to_owned()]),
            counter_eq("studio_camera_frames", 4),
            counter_eq("studio_camera_moving_runs", 2),
            counter_eq("studio_camera_matching_runs", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.studio.camera_preview",
            vec![
                FieldPred::str_eq("scene", "mixed_camera.v1"),
                FieldPred::str_eq("camera_identity", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.studio_native_worker.v1",
        ScenarioClass::LifecycleDrill,
        Surface::StudioSubprocess,
        Invocation::new(studio_subprocess_lifecycle_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "studio_initial_inspector.json".to_owned(),
                "studio_restored_inspector.json".to_owned(),
                "studio_overlays.json".to_owned(),
                "studio_replayed_frame.png".to_owned(),
            ]),
            counter_eq("studio_native_lifecycle_steps", 6),
        ],
        [
            "initial",
            "checkpoint_committed",
            "committed",
            "transient_and_refusals",
            "restarted",
            "clean_eof",
        ]
        .into_iter()
        .map(|phase| {
            LogExpect::span_present(
                "e2e.studio.lifecycle",
                vec![FieldPred::str_eq("phase", phase)],
            )
        })
        .collect(),
    ));
    specs.push(spec(
        "lifecycle.studio_native_interactive.v1",
        ScenarioClass::LifecycleDrill,
        Surface::StudioSubprocess,
        Invocation::new(studio_interactive_lifecycle_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "interactive_initial.png".to_owned(),
                "interactive_replayed.png".to_owned(),
                "interactive_initial_inspector.json".to_owned(),
                "interactive_replayed_inspector.json".to_owned(),
            ]),
            counter_eq("studio_interactive_inputs", 8),
        ],
        [
            "checkpoint",
            "edited",
            "refused_atomically",
            "restarted",
            "continued_and_undone",
        ]
        .into_iter()
        .map(|phase| {
            LogExpect::span_present(
                "e2e.studio.interactive",
                vec![FieldPred::str_eq("phase", phase)],
            )
        })
        .collect(),
    ));
    specs.push(spec(
        "lifecycle.studio_builtin_inputs.v1",
        ScenarioClass::LifecycleDrill,
        Surface::StudioSubprocess,
        Invocation::new(studio_builtin_inputs_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(
                STUDIO_BUILTIN_INPUT_CASES
                    .iter()
                    .flat_map(|(name, _, _)| {
                        [
                            format!("{name}.replayed.png"),
                            format!("{name}.inspector.json"),
                        ]
                    })
                    .collect(),
            ),
            counter_eq("studio_live_builtin_families", 5),
        ],
        STUDIO_BUILTIN_INPUT_CASES
            .iter()
            .map(|(name, _, _)| {
                LogExpect::span_present(
                    "e2e.studio.builtin_input",
                    vec![
                        FieldPred::str_eq("scene", *name),
                        FieldPred::str_eq("phase", "edited_undone_replayed"),
                    ],
                )
            })
            .collect(),
    ));
    specs.push(spec(
        "render_matrix.python_portal_png_sequence.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_portal_png_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["python_portal_frame.png".to_owned()]),
            counter_ge("python_render_frames", 1),
            counter_eq("python_png_signature", 1),
            counter_eq("python_engine_journaled", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.render",
            vec![
                FieldPred::str_eq("scene", "_GauntletPortalScene"),
                FieldPred::str_eq("format", "png_sequence"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("png_signature", "true"),
                FieldPred::str_eq("dimensions_match", "true"),
                FieldPred::str_eq("engine_journaled", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.python_studio_capture.v1",
        ScenarioClass::LifecycleDrill,
        Surface::PythonInProcess,
        Invocation::new(python_studio_capture_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "studio_first.png".to_owned(),
                "studio_last.png".to_owned(),
                "studio_inspector.json".to_owned(),
            ]),
            counter_eq("python_studio_frames", 6),
        ],
        vec![LogExpect::span_present(
            "e2e.python.studio_capture",
            vec![
                FieldPred::u64_eq("frames", 6),
                FieldPred::bool_eq("read_only", true),
                FieldPred::bool_eq("native_inspector", true),
            ],
        )],
    ));
    let mut portal_output_inventory = vec![
        "native_scene.py".to_owned(),
        "motion.gif".to_owned(),
        "motion.y4m".to_owned(),
        "primaries.gif".to_owned(),
        "primaries.y4m".to_owned(),
        "soundtrack.wav".to_owned(),
        "cue.wav".to_owned(),
        "quiet.wav".to_owned(),
        "camera-pan.gif".to_owned(),
        "camera-updater.gif".to_owned(),
        "background-changes.gif".to_owned(),
        "light-changes.gif".to_owned(),
        "overridden-light.gif".to_owned(),
        "escaped-markup.png".to_owned(),
        "literal-markup.png".to_owned(),
        "centered-typesetting.gif".to_owned(),
        "rectangle-flash.gif".to_owned(),
    ];
    if std::env::var("FMN_REQUIRE_FFMPEG").as_deref() == Ok("1") {
        portal_output_inventory
            .extend(["motion.mp4", "motion.mov", "soundtrack.mp4", "tone.wav"].map(str::to_owned));
    }
    specs.push(spec(
        "render_matrix.python_portal_native_outputs.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_portal_native_outputs_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(portal_output_inventory),
            counter_eq("portal_output_options_suite", 1),
            counter_eq("video_output_capability_refusals", 1),
            counter_eq("native_output_frames", 4),
            counter_eq("native_output_formats", 3),
            counter_eq("native_output_publication_failures", 14),
            counter_eq("native_output_thread_replays", 3),
            counter_eq("native_output_sample_frames", 3000),
        ],
        vec![LogExpect::span_present(
            "e2e.python.native_outputs",
            vec![
                FieldPred::u64_eq("frames", 4),
                FieldPred::u64_eq("formats", 3),
                FieldPred::u64_eq("publication_failures", 14),
                FieldPred::u64_eq("thread_replays", 3),
                FieldPred::u64_eq("sample_frames", 3000),
            ],
        )],
    ));
    // fm-5wq.46: the semantic witness on every golden-producing route. Each
    // route's frame must pass the sanity oracles and its planted vertical
    // mirror must fail them, inside the one invocation: the RenderMatrix
    // class already has its single drill.
    specs.push(spec(
        "render_matrix.semantic_oracles.native.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(semantic_oracles_native_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("semantic_routes_pass", 5),
            counter_eq("semantic_mirrors_rejected", 5),
        ],
        [
            "library_2d",
            "cli_2d",
            "cli_camera",
            "fmtl_native",
            "fmtl_wasm_player",
        ]
        .map(|route| {
            LogExpect::span_present(
                "e2e.oracle.semantic",
                vec![
                    FieldPred::str_eq("route", route),
                    FieldPred::str_eq("frame", "rendered"),
                    FieldPred::str_eq("oracle", "orientation.triangle"),
                    FieldPred::str_eq("verdict", "pass"),
                ],
            )
        })
        .to_vec(),
    ));
    specs.push(spec(
        "render_matrix.semantic_oracles.portal.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(semantic_oracles_portal_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("semantic_routes_pass", 2),
        ],
        ["portal_scene", "portal_camera_readback"]
            .map(|route| {
                LogExpect::span_present(
                    "e2e.oracle.semantic",
                    vec![
                        FieldPred::str_eq("route", route),
                        FieldPred::str_eq("frame", "planted_vertical_flip"),
                        FieldPred::str_eq("oracle", "orientation.triangle"),
                        FieldPred::str_eq("verdict", "fail"),
                    ],
                )
            })
            .to_vec(),
    ));
    specs.push(spec(
        "lifecycle.python_scene_console.v1",
        ScenarioClass::LifecycleDrill,
        Surface::PythonInProcess,
        Invocation::new(python_scene_console_run),
        vec![Assertion::ExitCode(0), counter_eq("console_checks", 7)],
        vec![LogExpect::span_present(
            "e2e.python.scene_console",
            vec![FieldPred::u64_eq("checks", 7)],
        )],
    ));
    specs.push(spec(
        "lifecycle.python_surface_hooks.v1",
        ScenarioClass::LifecycleDrill,
        Surface::PythonInProcess,
        Invocation::new(python_surface_hooks_run),
        vec![Assertion::ExitCode(0), counter_eq("surface_hook_cases", 15)],
        vec![LogExpect::span_present(
            "e2e.python.surface_hooks",
            vec![FieldPred::u64_eq("cases", 15)],
        )],
    ));
    specs.push(spec(
        "parity.structural_facts.v1",
        ScenarioClass::ParityDrill,
        Surface::PythonInProcess,
        Invocation::new(python_structural_facts_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("structural_subjects", 26),
        ],
        vec![LogExpect::span_present(
            "e2e.python.structural_facts",
            vec![FieldPred::u64_eq("compared", 26)],
        )],
    ));
    specs.push(spec(
        "parity.class_sweep.v1",
        ScenarioClass::ParityDrill,
        Surface::PythonInProcess,
        Invocation::new(python_class_sweep_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("class_sweep_subjects", 288),
        ],
        vec![LogExpect::span_present(
            "e2e.python.class_sweep",
            vec![FieldPred::u64_eq("compared", 288)],
        )],
    ));
    specs.push(spec(
        "parity.structural_envelope.v1",
        ScenarioClass::ParityDrill,
        Surface::PythonInProcess,
        Invocation::new(python_structural_envelope_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("envelope_violations_detected", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.structural_envelope",
            vec![
                FieldPred::str_eq("subject", "tex"),
                FieldPred::str_eq("planted_verdict", "differs"),
                FieldPred::str_eq("violated_row", "bn05-text-geometry"),
            ],
        )],
    ));
    specs.push(spec(
        "parity.reference_corpus_regressions.v1",
        ScenarioClass::ParityDrill,
        Surface::PythonInProcess,
        Invocation::new(python_reference_corpus_regressions_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("corpus_regression_cases", 4),
        ],
        vec![LogExpect::span_present(
            "e2e.python.reference_corpus_regression",
            vec![
                FieldPred::str_eq("bead", "fm-5wq.30"),
                FieldPred::str_eq("verdict", "pass"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_svg.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_svg_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["diagram.svg".to_owned()]),
            counter_eq("svg_thread_counts", 3),
            counter_eq("svg_failure_paths", 3),
        ],
        vec![LogExpect::span_present(
            "e2e.python.svg",
            vec![
                FieldPred::u64_eq("thread_counts", 3),
                FieldPred::u64_eq("failure_paths", 3),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_live_tex.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_live_tex_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "live_tex_first.png".to_owned(),
                "live_tex_last.png".to_owned(),
            ]),
            counter_eq("live_tex_frames", 6),
            counter_eq("live_tex_thread_counts", 3),
            counter_eq("live_tex_failure_paths", 2),
        ],
        vec![LogExpect::span_present(
            "e2e.python.live_tex",
            vec![
                FieldPred::u64_eq("frames", 6),
                FieldPred::u64_eq("thread_counts", 3),
                FieldPred::u64_eq("failure_paths", 2),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_complex_readouts.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_complex_readouts_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "complex_readouts_first.png".to_owned(),
                "complex_readouts_last.png".to_owned(),
            ]),
            counter_eq("complex_readouts_frames", 6),
            counter_eq("complex_readouts_thread_counts", 3),
            counter_eq("complex_readouts_failure_paths", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.complex_readouts",
            vec![
                FieldPred::u64_eq("frames", 6),
                FieldPred::u64_eq("thread_counts", 3),
                FieldPred::u64_eq("failure_paths", 1),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_tex_preamble.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_tex_preamble_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "tex_preamble_first.png".to_owned(),
                "tex_preamble_last.png".to_owned(),
            ]),
            counter_eq("tex_preamble_frames", 8),
            counter_eq("tex_preamble_thread_counts", 3),
            counter_eq("tex_preamble_failure_paths", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.tex_preamble",
            vec![
                FieldPred::u64_eq("frames", 8),
                FieldPred::u64_eq("thread_counts", 3),
                FieldPred::u64_eq("failure_paths", 1),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_textured_surfaces.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_textured_surfaces_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![
                "textures_first.png".to_owned(),
                "textures_last.png".to_owned(),
            ]),
        ],
        vec![LogExpect::span_present(
            "e2e.python.textures",
            vec![
                FieldPred::u64_eq("frames", 4),
                FieldPred::u64_eq("thread_counts", 3),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_surface_mesh.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_surface_mesh_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["surface_mesh.png".to_owned()]),
            counter_eq("surface_mesh_checks", 12),
            counter_ge("surface_mesh_visible_pixels", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.surface_mesh",
            vec![FieldPred::u64_eq("checks", 12)],
        )],
    ));
    specs.push(spec(
        "lifecycle.python_animation_semantics.v1",
        ScenarioClass::LifecycleDrill,
        Surface::PythonInProcess,
        Invocation::new(python_portal_animation_lifecycle_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["python_portal_frame.png".to_owned()]),
            counter_ge("python_render_frames", 3),
            counter_eq("python_png_signature", 1),
            counter_eq("python_engine_journaled", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.render",
            vec![
                FieldPred::str_eq("scene", "AnimationLifecycleScene"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("dimensions_match", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "parity.python_portal_reference_defaults.v1",
        ScenarioClass::ParityDrill,
        Surface::PythonInProcess,
        Invocation::new(python_portal_reference_defaults_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["python_portal_defaults.y4m".to_owned()]),
            counter_eq("python_defaults_reference_background", 1),
            counter_eq("python_defaults_reference_rate", 1),
            counter_eq("python_defaults_brace_spans_composite", 1),
            counter_eq("python_defaults_front_doors_agree", 1),
            counter_eq("python_defaults_display_style_tex", 1),
            counter_eq("python_defaults_readme_formula_envelope", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.reference_defaults",
            vec![
                FieldPred::str_eq("rate", "30:1"),
                FieldPred::str_eq("background_is_reference", "true"),
                FieldPred::str_eq("rate_is_reference", "true"),
                FieldPred::str_eq("brace_spans", "true"),
                FieldPred::str_eq("front_doors_agree", "true"),
                FieldPred::str_eq("display_style", "true"),
                FieldPred::str_eq("readme_envelope", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_portal_planar_route.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_portal_planar_route_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("python_planar_native_identical", 1),
            counter_eq("python_planar_surface_camera", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.planar_route",
            vec![
                FieldPred::str_eq("native_identical", "true"),
                FieldPred::u64_eq("camera_frames", 0),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.python_portal_png_still.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_portal_png_still_run),
        vec![
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec!["python_portal_still.png".to_owned()]),
            counter_eq("python_still_frames", 1),
            counter_eq("python_still_png_signature", 1),
            counter_eq("python_still_engine_journaled", 1),
            counter_eq("python_still_wrong_node_kind_refused", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.python.render",
            vec![
                FieldPred::str_eq("scene", "_GauntletPortalScene"),
                FieldPred::str_eq("format", "png"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("png_signature", "true"),
                FieldPred::str_eq("dimensions_match", "true"),
                FieldPred::str_eq("engine_journaled", "true"),
                FieldPred::str_eq("wrong_node_kind_refused", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.cli_prerun_subdivide_y4m.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_prerun_subdivide_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_prerun_frames", 3),
            counter_eq("cli_subdivision_segments", 2),
            counter_eq("cli_subdivision_frames", 3),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.prerun_subdivide",
            vec![
                FieldPred::str_eq("prerun_record", "true"),
                FieldPred::str_eq("render_records", "true"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::u64_eq("rendered_frames", 3),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.cli_batch_png_sequence.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_batch_render_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_batch_scenes", 2),
            counter_eq("cli_batch_frames", 6),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.batch",
            vec![
                FieldPred::str_eq("source", "builtin"),
                FieldPred::str_eq("format", "png_sequence"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("png_signatures", "true"),
                FieldPred::str_eq("manifests", "true"),
                FieldPred::str_eq("ordered_records", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.cli_native_png_gif.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_native_png_gif_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_png_still_frames", 1),
            counter_eq("cli_gif_stream_frames", 3),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.native_rgba_artifacts",
            vec![
                FieldPred::str_eq("png_signature", "true"),
                FieldPred::str_eq("gif_signature", "true"),
                FieldPred::str_eq("robot_records", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "render_matrix.cli_compiled_fmtl_png_sequence.v1",
        ScenarioClass::RenderMatrix,
        Surface::CliInProcess,
        Invocation::new(cli_compiled_bundle_render_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_compiled_render_frames", 2),
            counter_eq("cli_compiled_png_signature", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.cli.render",
            vec![
                FieldPred::str_eq("source", "compiled"),
                FieldPred::str_eq("format", "png_sequence"),
                FieldPred::str_eq("complete", "true"),
                FieldPred::str_eq("png_signature", "true"),
                FieldPred::str_eq("robot_record", "true"),
            ],
        )],
    ));

    // DETERMINISM DRILLS.
    specs.push(spec(
        "determinism.repeat_byte_identical.v1",
        ScenarioClass::DeterminismDrill,
        Surface::RustApi,
        Invocation::new(determinism_repeat_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("repeat_byte_equal", 1),
            Assertion::GoldenLock {
                suite: E2E_SUITE,
                scope: Scope::Certified,
            },
        ],
        vec![
            LogExpect::counter_ge("repeat_byte_equal", 1),
            LogExpect::no_event("e2e.failure"),
        ],
    ));
    specs.push(spec(
        "determinism.threads_1_4_byte_identical.v1",
        ScenarioClass::DeterminismDrill,
        Surface::RustApi,
        Invocation::new(|ctx| determinism_threads_run(ctx, 4)),
        vec![Assertion::ExitCode(0), counter_eq("thread_byte_equal", 1)],
        vec![LogExpect::counter_ge("thread_byte_equal", 1)],
    ));
    specs.push(
        spec(
            "determinism.threads_16_byte_identical.v1",
            ScenarioClass::DeterminismDrill,
            Surface::RustApi,
            Invocation::new(|ctx| determinism_threads_run(ctx, 16)),
            vec![Assertion::ExitCode(0), counter_eq("thread_byte_equal", 1)],
            vec![LogExpect::counter_ge("thread_byte_equal", 1)],
        )
        .tier(Tier::Full),
    );

    specs.push(spec(
        "lifecycle.scene_soundtrack_windows.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(scene_soundtrack_windows_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("soundtrack_cues", 2),
            counter_eq("soundtrack_windows_equal", 1),
            counter_eq("soundtrack_skip_rebased", 1),
            counter_eq("soundtrack_source_identity", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.soundtrack.windows",
            vec![
                FieldPred::u64_eq("cues", 2),
                FieldPred::u64_eq("captured_frames", 45),
                FieldPred::str_eq("sample_window_equal", "true"),
                FieldPred::str_eq("skipped_prefix_removed", "true"),
                FieldPred::str_eq("source_identity", "true"),
            ],
        )],
    ));

    // FAILURE-PATH DRILLS.
    specs.push(spec(
        "failure_path.tex_unsupported_construct_named.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(failure_tex_run),
        vec![Assertion::ExitCode(0), counter_eq("error_named", 1)],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("error_kind", "TexError::Math"),
                FieldPred::str_eq("names_construct", "true"),
                FieldPred::str_eq("named_refusal", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cache_corrupt_recovers.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(failure_cache_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cache_corrupt_miss", 1),
            counter_eq("cache_recovery_ok", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("cache_corrupt_miss", "true"),
                FieldPred::str_eq("cache_corrupt_evicted", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cli_quality_exclusive_named.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(failure_cli_run),
        vec![
            Assertion::ExitCode(2),
            counter_eq("cli_rule_named", 1),
            Assertion::NdjsonSchema,
        ],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("rule", "quality-exclusive"),
                FieldPred::str_eq("rule_named", "true"),
                FieldPred::str_eq("stderr_empty", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cli_video_quality_admission.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(failure_cli_video_quality_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_video_quality_refusals", 6),
            counter_eq("cli_video_quality_tool_calls", 0),
            Assertion::NdjsonSchema,
        ],
        vec![LogExpect::span_present(
            "e2e.failure.cli_video_quality",
            vec![
                FieldPred::u64_eq("refused", 6),
                FieldPred::u64_eq("tool_calls", 0),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cli_ffmpeg_limit_admission.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(failure_cli_ffmpeg_limits_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_ffmpeg_limit_refusals", 4),
            counter_eq("cli_ffmpeg_limit_tool_calls", 0),
            counter_eq("cli_ffmpeg_admitted_timeout", 21_600),
            counter_eq("cli_ffmpeg_admitted_artifact", 8 << 30),
            counter_eq("cli_ffmpeg_admitted_input", 8 << 30),
            Assertion::NdjsonSchema,
        ],
        vec![LogExpect::span_present(
            "e2e.failure.cli_ffmpeg_limits",
            vec![
                FieldPred::u64_eq("refused", 4),
                FieldPred::u64_eq("tool_calls", 0),
                FieldPred::u64_eq("admitted_timeout_seconds", 21_600),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.facade_video_requires_ffmpeg_capability.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(failure_facade_video_capability_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("facade_video_refusal_named", 1),
            counter_ge("facade_native_frames", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("rule", "facade-video-capability"),
                FieldPred::str_eq("refusal_named", "true"),
                FieldPred::str_eq("scene_ran", "false"),
                FieldPred::str_eq("published", "false"),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cli_window_flag_refused_timing_kept.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(failure_cli_window_flag_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("cli_window_flag_refused", 1),
            counter_eq("cli_plain_render_frames", 23),
        ],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("rule", "window-flag-on-file-render"),
                FieldPred::u64_eq("refused_code", 4),
                FieldPred::str_eq("published_nothing", "true"),
                FieldPred::u64_eq("plain_frames", 23),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.cli_certified_metal_prepublication_refusal.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(failure_cli_certified_metal_run),
        vec![
            Assertion::ExitCode(3),
            counter_eq("cli_certified_metal_refused", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.failure",
            vec![
                FieldPred::str_eq("contract", "certified-cpu-only"),
                FieldPred::str_eq("contract_named", "true"),
                FieldPred::str_eq("stderr_empty", "true"),
                FieldPred::str_eq("output_absent", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "determinism.certified_manifest_compare.v1",
        ScenarioClass::DeterminismDrill,
        Surface::CliInProcess,
        Invocation::new(determinism_certified_manifest_compare_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("certified_manifest_agree", 1),
            counter_eq("certified_manifest_input_distinguished", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.determinism.manifest",
            vec![
                FieldPred::str_eq("same_input_agree", "true"),
                FieldPred::str_eq("other_input_semantic_equal", "false"),
                FieldPred::str_eq("platform_record", "true"),
                FieldPred::str_eq("fonts_recorded", "true"),
                FieldPred::str_eq("build_identity_named", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "failure_path.host_qualification_requires_live_token.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(failure_host_attestation_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("host_qualification_refused", 1),
        ],
        vec![LogExpect::span_present(
            "performance.host_attestation",
            vec![
                FieldPred::str_eq("token_present", "false"),
                FieldPred::str_eq("bare_metal", "false"),
                FieldPred::str_eq("isolated", "false"),
                FieldPred::str_eq("caller_claim_refused", "true"),
            ],
        )],
    ));

    // LIFECYCLE DRILLS.
    specs.push(spec(
        "lifecycle.construct_snapshot_transform_snapshot.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_construct_run),
        vec![
            Assertion::ExitCode(0),
            counter_ge("lifecycle_members", 1),
            counter_ge("lifecycle_points_pre", 1),
            counter_ge("lifecycle_points_post", 1),
            Assertion::GoldenLock {
                suite: E2E_SUITE,
                scope: Scope::Certified,
            },
        ],
        vec![
            LogExpect::span_present(
                e2e::spans::SCENE_CONSTRUCT,
                vec![
                    FieldPred::str_eq("scene", "circle_tex_label.v1"),
                    FieldPred::u64_ge("members", 1),
                ],
            ),
            LogExpect::event_order(e2e::spans::SCENE_CONSTRUCT, "e2e.lifecycle.transform"),
        ],
    ));
    specs.push(spec(
        "lifecycle.public_fmn_facade_executes.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(public_facade_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("public_facade_plays", 2),
            counter_eq("public_facade_captures", 3),
        ],
        vec![LogExpect::span_present(
            e2e::spans::SCENE_CONSTRUCT,
            vec![
                FieldPred::str_eq("scene", "public_facade_circle_shift"),
                FieldPred::u64_eq("captures", 3),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.journal_roundtrip_replay.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(lifecycle_journal_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("journal_roundtrip_equal", 1),
            counter_eq("journal_replay_reuse", 3),
            counter_eq("journal_mismatch_detected", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.lifecycle.journal",
            vec![
                FieldPred::str_eq("roundtrip_equal", "true"),
                FieldPred::u64_ge("replay_reuse", 3),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.pg5_definition_direct_schedule.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(pg5_direct_schedule_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("pg5_direct_threads", 3),
            counter_eq("pg5_direct_mismatches", 0),
        ],
        vec![LogExpect::span_present(
            "performance.pg5.direct_schedule",
            vec![
                FieldPred::str_eq("scene", "circle_tex_label.v1"),
                FieldPred::str_eq("direct_threads", "1,4,16"),
                FieldPred::u64_eq("direct_mismatches", 0),
            ],
        )],
    ));
    specs.push(spec(
        "lifecycle.pg6_arena_reuse_zero_allocations.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(pg6_allocation_reuse_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("pg6_digest_equal", 1),
            counter_eq("pg6_measured_allocations", 0),
        ],
        vec![LogExpect::span_present(
            "performance.pg6.allocations",
            vec![
                FieldPred::str_eq("scene", "circle_tex_label.v1"),
                FieldPred::u64_eq("measured_allocations", 0),
            ],
        )],
    ));
    specs.push(
        spec(
            "lifecycle.pg6_peak_gallery_4k_3d.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(pg6_peak_gallery_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("pg6_peak_frames", PG6_PEAK_CASE_COUNT as u64),
                counter_eq("pg6_peak_distinct_digests", 1),
            ],
            vec![LogExpect::span_present(
                "performance.pg6.peak_gallery",
                vec![
                    FieldPred::u64_eq("frames", PG6_PEAK_CASE_COUNT as u64),
                    FieldPred::u64_eq("width", u64::from(PG6_PEAK_WIDTH)),
                    FieldPred::u64_eq("height", u64::from(PG6_PEAK_HEIGHT)),
                    FieldPred::bool_eq("distinct_digests", true),
                    FieldPred::u64_ge("max_preparation_bytes", 1),
                ],
            )],
        )
        .tier(Tier::Full),
    );

    // LOG ASSERTIONS (machinery proofs).
    specs.push(spec(
        "logs.preflight_typesets_before_first_frame.v1",
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(logs_preflight_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("typesets", 2),
            counter_eq("preflight_before_first_frame", 1),
        ],
        vec![
            LogExpect::span_present(
                e2e::spans::PREFLIGHT,
                vec![
                    FieldPred::u64_ge("typesets", 2),
                    FieldPred::str_eq("before_first_frame", "true"),
                ],
            ),
            LogExpect::event_order(e2e::spans::PREFLIGHT, e2e::spans::RENDER_FRAME),
            LogExpect::counter_ge(e2e::counters::TYPESETS, 2),
        ],
    ));
    specs.push(spec(
        "logs.segment_purity_recorded.v1",
        ScenarioClass::DeterminismDrill,
        Surface::RustApi,
        Invocation::new(logs_purity_run),
        vec![Assertion::ExitCode(0), counter_eq("segments_classified", 2)],
        vec![
            LogExpect::span_present(
                e2e::spans::PURITY,
                vec![
                    FieldPred::str_eq("segment", "play"),
                    FieldPred::str_eq("purity", "pure"),
                ],
            ),
            LogExpect::span_present(
                e2e::spans::PURITY,
                vec![
                    FieldPred::str_eq("segment", "wait"),
                    FieldPred::str_eq("purity", "stateful"),
                ],
            ),
        ],
    ));
    specs.push(spec(
        "logs.engine_identity_certified_reproducible.v1",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(logs_engine_identity_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("engine_certified_frames", 1),
        ],
        vec![
            LogExpect::span_present(
                e2e::spans::ENGINE,
                vec![
                    FieldPred::str_eq("engine", "certified_cpu"),
                    FieldPred::str_eq("reproducible", "true"),
                ],
            ),
            LogExpect::span_present(
                e2e::spans::EXECUTION_PLAN,
                vec![
                    FieldPred::str_eq("tuning_source", "certified_profile"),
                    FieldPred::str_eq("determinism", "certified"),
                ],
            ),
            LogExpect::event_order(e2e::spans::EXECUTION_PLAN, e2e::spans::RENDER_FRAME),
        ],
    ));

    // ------------------------------------------------------------------
    // Parity drills (fm-parity-drill-scenarios-hzng): one fast-tier drill
    // per Parity Ledger ruling family — C-5, C-17, C-15, C-16, BN-03, and
    // the §11.3 span map — each asserting the CORRECTED behavior
    // end-to-end so the Reference-buggy answer turns the drill red.
    // ------------------------------------------------------------------
    specs.push(spec(
        "parity.updater_call_runs_once.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_updater_call_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_updater_attach_calls", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.updater_call_once",
            vec![
                FieldPred::u64_eq("call_attach_calls", 1),
                FieldPred::u64_eq("dt_attach_calls", 1),
            ],
        )],
    ));
    specs.push(spec(
        "parity.plane_y_unit_size.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_plane_y_unit_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_plane_y_unit_is_own_axis", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.plane_y_unit_size",
            vec![FieldPred::u64_eq("y_unit_milli", 2_000)],
        )],
    ));
    specs.push(spec(
        "parity.function_graph_color.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_function_graph_color_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_function_graph_color_honored", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.function_graph_color",
            vec![
                FieldPred::str_eq("default_yellow", "true"),
                FieldPred::str_eq("requested_honored", "true"),
            ],
        )],
    ));
    specs.push(spec(
        "parity.frame_markers_to_edge.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_frame_markers_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_frame_marker_placements", 5),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.frame_markers",
            vec![
                FieldPred::u64_eq("marker_directions", 4),
                FieldPred::u64_eq("exact_placements", 5),
            ],
        )],
    ));
    specs.push(spec(
        "parity.move_along_path_true_arclength.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_move_along_path_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_arclength_animation_matches", 3),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.move_along_path_arclength",
            vec![
                FieldPred::str_eq("equal_steps", "true"),
                FieldPred::u64_eq("animation_matches", 3),
            ],
        )],
    ));
    specs.push(spec(
        "parity.tex_span_map_t2c.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_tex_span_map_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_t2c_source_identity", 1),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.tex_span_map_t2c",
            vec![
                FieldPred::u64_eq("x_occurrences", 2),
                FieldPred::u64_eq("plus_occurrences", 2),
            ],
        )],
    ));
    specs.push(spec(
        "parity.markdown_inline_math.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(parity_markdown_inline_math_run),
        vec![
            Assertion::ExitCode(0),
            counter_eq("parity_markdown_prose_dollars", 3),
        ],
        vec![LogExpect::span_present(
            "e2e.parity.markdown_inline_math",
            vec![
                FieldPred::bool_eq("island_is_text_style_tex", true),
                FieldPred::u64_eq("prose_sources", 3),
                FieldPred::u64_eq("prose_sources_literal", 3),
            ],
        )],
    ));
    // ------------------------------------------------------------------
    // fm-3kr / fm-n64 enhanced-surface drills: refusal policy by name,
    // spring-kernel determinism, CSV→Table round-trip.
    // ------------------------------------------------------------------
    specs.push(
        spec(
            "drawings.piano_keyboard.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(piano_keyboard_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("piano_keys", 88),
                counter_eq("piano_black_keys", 36),
                counter_eq("piano_centered", 1),
                counter_eq("piano_proportions", 1),
                counter_eq("piano_visible_motion", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.drawings.piano",
                vec![
                    FieldPred::u64_eq("keys", 88),
                    FieldPred::u64_eq("black_keys", 36),
                    FieldPred::str_eq("visible_motion", "true"),
                ],
            )],
        )
        .tier(Tier::Fast),
    );
    specs.push(
        spec(
            "drawings.asset_refusals_named.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(drawings_asset_refusals_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("asset_refusals_named", 5),
            ],
            vec![LogExpect::span_present(
                "e2e.drawings.asset_refusal",
                vec![FieldPred::u64_eq("named", 5)],
            )],
        )
        .tier(Tier::Fast),
    );
    specs.push(
        spec(
            "network_graph.spring_seed_determinism.v1",
            ScenarioClass::DeterminismDrill,
            Surface::RustApi,
            Invocation::new(network_graph_layout_determinism_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("spring_replay_equal", 1),
                counter_eq("spring_seeds_diverge", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.network_graph.spring_determinism",
                vec![FieldPred::u64_eq("seed", 0x005f_3759df)],
            )],
        )
        .tier(Tier::Fast),
    );
    specs.push(
        spec(
            "neural_network.live_training_and_purity.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(neural_network_live_training_and_purity_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("feed_forward_matches", 1),
                counter_eq("purity_is_stateful", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.neural_network.live_training",
                vec![
                    FieldPred::u64_eq("layers", 3),
                    FieldPred::u64_eq("stateful", 1),
                ],
            )],
        )
        .tier(Tier::Fast),
    );
    specs.push(
        spec(
            "data_table.csv_roundtrip.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(data_table_csv_roundtrip_run),
            vec![
                Assertion::ExitCode(0),
                counter_eq("data_table_headers", 3),
                counter_eq("data_table_rows", 3),
                counter_eq("data_table_builds_identical", 1),
            ],
            vec![LogExpect::span_present(
                "e2e.data_table.roundtrip",
                vec![
                    FieldPred::u64_eq("headers", 3),
                    FieldPred::u64_eq("rows", 3),
                ],
            )],
        )
        .tier(Tier::Fast),
    );

    // ------------------------------------------------------------------
    // Deliberately-injected regression drills: one per seeded class. The
    // runner drives these RED and requires the repro bundle plus the log
    // artifact to appear, tagged with the class.
    // ------------------------------------------------------------------
    specs.push(
        matrix_spec(
            &PRESETS[0],
            SinkKind::Y4m,
            true,
            Some("render_matrix.low.y4m.certified.drill_golden.v1"),
        )
        .regression(RegressionKind::GoldenDrift),
    );
    specs.push(
        spec(
            "determinism.repeat_byte_identical.drill_log.v1",
            ScenarioClass::DeterminismDrill,
            Surface::RustApi,
            Invocation::new(determinism_repeat_run),
            vec![Assertion::ExitCode(0), counter_eq("repeat_byte_equal", 1)],
            vec![LogExpect::counter_ge("repeat_byte_equal", 1)],
        )
        .regression(RegressionKind::LogExpectation),
    );
    specs.push(
        spec(
            "failure_path.tex_unsupported_construct_named.drill_log.v1",
            ScenarioClass::FailurePath,
            Surface::RustApi,
            Invocation::new(failure_tex_run),
            vec![Assertion::ExitCode(0), counter_eq("error_named", 1)],
            vec![LogExpect::span_present(
                "e2e.failure",
                vec![FieldPred::str_eq("error_kind", "TexError::Math")],
            )],
        )
        .regression(RegressionKind::LogExpectation),
    );
    specs.push(
        spec(
            "lifecycle.construct_snapshot_transform_snapshot.drill_golden.v1",
            ScenarioClass::LifecycleDrill,
            Surface::RustApi,
            Invocation::new(lifecycle_construct_run),
            vec![
                Assertion::ExitCode(0),
                Assertion::GoldenLock {
                    suite: E2E_SUITE,
                    scope: Scope::Certified,
                },
            ],
            Vec::new(),
        )
        .regression(RegressionKind::GoldenDrift),
    );

    specs
}

/// One semantic-oracle log line: route, which frame, the oracle, its
/// fixture, what it required, what it measured, and the verdict.
fn semantic_reading_event(
    route: &str,
    frame: &str,
    oracle: &str,
    fixture: &str,
    expected: &str,
    measured: &str,
    pass: bool,
) -> LogEvent {
    LogEvent::new("e2e.oracle.semantic")
        .field("route", route)
        .field("frame", frame)
        .field("oracle", oracle)
        .field("fixture", fixture)
        .field("expected", expected)
        .field("measured", measured)
        .field("verdict", if pass { "pass" } else { "fail" })
}

/// fm-5wq.46: the semantic witness through every native golden-producing
/// route (library, CLI affine and camera, native and wasm FMTL players).
fn semantic_oracles_native_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let (mut passed, mut rejected) = (0_u64, 0_u64);
    for route in semantic_routes::ROUTES {
        let frame = route().map_err(fail)?;
        let (rendered, flipped) = frame.read();
        for (label, readings) in [("rendered", &rendered), ("planted_vertical_flip", &flipped)] {
            for r in readings {
                ctx.event(semantic_reading_event(
                    frame.route,
                    label,
                    r.oracle,
                    r.fixture,
                    &r.expected,
                    &r.measured,
                    r.pass,
                ));
            }
        }
        match frame.verdict() {
            Ok(()) => {
                passed += 1;
                rejected += 1;
            }
            Err(error) => ctx.event(
                LogEvent::new("e2e.oracle.semantic_failure")
                    .field("route", frame.route)
                    .field("error", error),
            ),
        }
    }
    ctx.counter("semantic_routes_pass", passed);
    ctx.counter("semantic_mirrors_rejected", rejected);
    Ok(RunOutcome::ok()
        .with_counter("semantic_routes_pass", passed)
        .with_counter("semantic_mirrors_rejected", rejected))
}

/// fm-5wq.46: the portal's scene render and camera readback; the Python
/// suite asserts both frames and rejects both planted mirrors.
fn semantic_oracles_portal_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let readings = manimlib::run_portal_gauntlet_semantic_witness().map_err(fail)?;
    let mut routes = std::collections::BTreeSet::new();
    for r in &readings {
        routes.insert(r.route.clone());
        ctx.event(semantic_reading_event(
            &r.route,
            &r.frame,
            &r.oracle,
            "",
            "",
            &r.measured,
            r.pass,
        ));
    }
    ctx.counter("semantic_routes_pass", routes.len() as u64);
    Ok(RunOutcome::ok().with_counter("semantic_routes_pass", routes.len() as u64))
}

// ---------------------------------------------------------------------------
// Test entry points
// ---------------------------------------------------------------------------

#[test]
fn live_coordinate_surface_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.live_coordinate_surface.v1")
        .expect("the live 3D coordinate surface scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn live_coordinate_animation_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.live_coordinate_animation.v1")
        .expect("the live coordinate animation scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn coordinate_placement_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.coordinate_placement.v1")
        .expect("the coordinate placement scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn bundle_render_state_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.bundle_render_state.v1")
        .expect("the bundle render-state scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn install_shared_family_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.install_shared_family.v1")
        .expect("the shared-family install lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn always_redraw_shared_source_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.always_redraw_shared_source.v1")
        .expect("the shared-source redraw lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn traced_path_cadence_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.traced_path_cadence.v1")
        .expect("the native traced-path cadence scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn owned_tail_branches_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.owned_tail_branches.v1")
        .expect("the owned-tail copy and snapshot scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn succession_reverse_seek_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.succession_reverse_seek.v1")
        .expect("the reversible succession render and seek scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn tracing_tail_lifecycle_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.tracing_tail_empty_until_update.v1")
        .expect("the tracing-tail lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn piano_keyboard_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "drawings.piano_keyboard.v1")
        .expect("native piano keyboard scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-5wq.46's native semantic-oracle scenario, focused.
#[test]
fn semantic_oracles_native_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.semantic_oracles.native.v1")
        .expect("the native semantic-oracle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-5wq.46's portal semantic-oracle scenario, focused.
#[test]
fn semantic_oracles_portal_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.semantic_oracles.portal.v1")
        .expect("the portal semantic-oracle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Focused acceptance for the Python production-composition seam. The same
/// spec also remains in the per-commit fast-tier catalog.
#[test]
fn python_portal_png_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_portal_png_sequence.v1")
        .expect("Python portal scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// The production Python capture and native read-only worker in the fast tier.
#[test]
fn python_studio_capture_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.python_studio_capture.v1")
        .expect("Python Studio capture is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-longform-native-e2e-hoeffding-l59d: the explainer's portal segments,
/// focused.
#[test]
fn hoeffding_d_portal_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.hoeffding_d_portal.v1")
        .expect("the Hoeffding's D portal scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-longform-native-e2e-hoeffding-l59d: the whole explainer at 1080p60 with
/// sound, focused: green under `FMN_E2E_FULL=1` (run it with `--release`), a
/// SKIP otherwise. Its chapters' fast row runs once, in the fast tier.
#[test]
fn hoeffding_d_full_scenario_passes_when_enabled() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.hoeffding_d_full_mp4.v1")
        .expect("the Hoeffding's D full scenario is registered");
    let report = Runner::from_env().run(scenario);
    if e2e::full_matrix_from_env() {
        assert!(report.is_pass(), "{}", report.summary());
    } else {
        assert!(
            matches!(report.status, Status::Skipped(_)),
            "{}",
            report.summary()
        );
    }
}

/// Quality flag interactions and encoder compatibility fail before tool discovery.
#[test]
fn cli_video_quality_admission_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "failure_path.cli_video_quality_admission.v1")
        .expect("CLI video-quality scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// CLI ffmpeg budgets share native admission and refuse before tool discovery.
#[test]
fn cli_ffmpeg_limit_admission_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "failure_path.cli_ffmpeg_limit_admission.v1")
        .expect("CLI ffmpeg-limit scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// The facade's video formats need an explicit ffmpeg capability (D2).
#[test]
fn facade_video_capability_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "failure_path.facade_video_requires_ffmpeg_capability.v1")
        .expect("facade video capability scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-cli-flag-timing-ht01: window flags refuse file renders; timing holds.
#[test]
fn cli_window_flag_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "failure_path.cli_window_flag_refused_timing_kept.v1")
        .expect("CLI window-flag scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// fm-sq8.11: the portal's planar route, certified parity with native fmn.
#[test]
fn python_planar_route_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_portal_planar_route.v1")
        .expect("Python planar route scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Portal scenes default to the Reference's 30 fps on #333333; braces span
/// detached composites (fm-5wq.44, fm-5wq.43).
#[test]
fn python_reference_defaults_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.python_portal_reference_defaults.v1")
        .expect("Python reference defaults scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Decoded native GIF/y4m/WAV, ordering, cue placement, and failure atomicity.
#[test]
fn python_native_outputs_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_portal_native_outputs.v1")
        .expect("Python native output scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn markdown_inline_math_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.markdown_inline_math.v1")
        .expect("Markdown inline-math scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Native live-number layout, animation, thread replay and failed publication.
#[test]
fn python_live_tex_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_live_tex.v1")
        .expect("Python live equation scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Native complex-number layout, animation, thread replay and failed publication.
#[test]
fn python_complex_readouts_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_complex_readouts.v1")
        .expect("Python complex readout scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Native macro layout, matching, source spans and failed publication.
#[test]
fn python_tex_preamble_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_tex_preamble.v1")
        .expect("Python TeX preamble scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Sampled wireframes through the native portal, renderer and PNG publisher.
#[test]
fn python_svg_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_svg.v1")
        .expect("Python SVG scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_textured_surfaces_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_textured_surfaces.v1")
        .expect("textured surface scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_surface_mesh_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_surface_mesh.v1")
        .expect("Python surface mesh scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Curved transforms, authored hooks and composition through real PNG output.
#[test]
fn python_animation_lifecycle_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.python_animation_semantics.v1")
        .expect("Python animation lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Focused acceptance for the portal's one-frame final-state route, including
/// the no-clobber planted negative.
#[test]
fn python_portal_png_still_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_portal_png_still.v1")
        .expect("Python portal still scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// Focused entry for the same mandatory fast-tier production lifecycle row.
#[test]
fn studio_camera_preview_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "render_matrix.studio_camera_preview.v1")
        .expect("the live Studio camera scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn studio_native_worker_lifecycle_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.studio_native_worker.v1")
        .expect("native Studio lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn studio_native_interactive_lifecycle_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.studio_native_interactive.v1")
        .expect("native Studio interactive lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn studio_native_builtin_input_lifecycle_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.studio_builtin_inputs.v1")
        .expect("native builtin input lifecycle scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn cli_doctor_capabilities_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.cli_doctor_capabilities.v1")
        .expect("doctor capability scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// The fast tier: every non-drill `Tier::Fast` scenario runs green
/// per-commit.
#[test]
fn fast_tier_scenarios_pass() {
    let runner = Runner::from_env();
    let scenarios: Vec<ScenarioSpec> = catalog()
        .into_iter()
        .filter(|scenario| scenario.tier == Tier::Fast && scenario.regression.is_none())
        .collect();
    assert!(
        scenarios.len() >= 10,
        "the fast tier thinned out: {} scenarios",
        scenarios.len()
    );
    let reports = runner.run_all(scenarios);
    let failures: Vec<String> = reports
        .iter()
        .filter(|report| !report.is_pass())
        .map(|report| report.summary())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The full matrix: env-gated for the nightly budget (R22). With the
/// gate off every full-tier scenario must SKIP (never silently pass,
/// never fail); with it on every one must run green.
#[test]
fn full_tier_scenarios_pass_when_enabled() {
    let runner = Runner::from_env();
    let scenarios: Vec<ScenarioSpec> = catalog()
        .into_iter()
        .filter(|scenario| scenario.tier == Tier::Full && scenario.regression.is_none())
        .collect();
    assert!(
        scenarios.len() >= 10,
        "the full tier thinned out: {} scenarios",
        scenarios.len()
    );
    let reports = runner.run_all(scenarios);
    if e2e::full_matrix_from_env() {
        let failures: Vec<String> = reports
            .iter()
            .filter(|report| !report.is_pass())
            .map(|report| report.summary())
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    } else {
        for report in &reports {
            assert!(
                matches!(report.status, Status::Skipped(_)),
                "full-tier scenario ran or failed without {FULL_TIER_ENV}=1: {}",
                report.summary()
            );
        }
    }
}

/// The injected-regression drills: each goes red with useful logs, and
/// the repro bundle plus log artifact appear on disk, tagged with the
/// class.
#[test]
fn injected_regression_drills_go_red_with_repro_bundles() {
    let runner = Runner::from_env();
    let drills: Vec<ScenarioSpec> = catalog()
        .into_iter()
        .filter(|scenario| scenario.regression.is_some())
        .collect();
    let classes: Vec<ScenarioClass> = drills.iter().map(|scenario| scenario.class).collect();
    assert_eq!(
        drills.len(),
        4,
        "one injected regression drill per seeded class"
    );
    let reports = runner.run_all(drills);
    assert_eq!(reports.len(), classes.len());
    for (report, class) in reports.iter().zip(classes) {
        assert!(
            report.went_red(),
            "drill {} did not go red: {}",
            report.scenario,
            report.summary()
        );
        let repro = report
            .repro_bundle()
            .expect("a red drill carries its repro bundle path");
        let log = report
            .log_artifact()
            .expect("a red drill carries its log artifact path");
        assert!(
            repro.exists(),
            "repro bundle missing at {}",
            repro.display()
        );
        assert!(log.exists(), "log artifact missing at {}", log.display());
        match &report.status {
            Status::RegressionConfirmed { failure } => {
                assert_eq!(
                    failure.class, class,
                    "drill {} is tagged with the wrong class",
                    report.scenario
                );
            }
            other => assert!(
                matches!(other, Status::RegressionConfirmed { .. }),
                "drill {} concluded {} instead of RegressionConfirmed",
                report.scenario,
                other.tag()
            ),
        }
    }
}

/// The runner's own NDJSON log artifact validates against its declared
/// schema — the harness's self-check, run against a real scenario's real
/// log output on disk.
#[test]
fn runner_log_artifact_validates_against_declared_schema() {
    let runner = Runner::from_env();
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "logs.preflight_typesets_before_first_frame.v1")
        .expect("the preflight scenario is registered");
    let report = runner.run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
    let log = report
        .log_artifact()
        .expect("a passing run carries its log artifact");
    e2e::validate_log_artifact(log)
        .expect("the run's NDJSON log artifact validates against the harness's declared schema");
}

/// The Parity Ledger ruling drills' stable scenario ids: one per ruling
/// family (C-5, C-17, C-15, C-16, BN-03, and the §11.3 span map). The
/// invariant pins each id so a ruling can never silently lose its e2e
/// drill.
const PARITY_RULING_DRILL_IDS: &[&str] = &[
    "parity.updater_call_runs_once.v1",
    "parity.plane_y_unit_size.v1",
    "parity.function_graph_color.v1",
    "parity.frame_markers_to_edge.v1",
    "parity.move_along_path_true_arclength.v1",
    "parity.tex_span_map_t2c.v1",
];

/// Catalog invariants: unique path-safe names, class coverage, a drill
/// per class, and no Pending-surface stubs.
#[test]
fn catalog_invariants_hold() {
    let scenarios = catalog();
    assert!(scenarios.len() >= 25, "catalog thinned out");

    let mut names = std::collections::BTreeSet::new();
    for scenario in &scenarios {
        assert!(
            names.insert(scenario.name),
            "duplicate scenario name {}",
            scenario.name
        );
        assert!(
            scenario
                .name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c)),
            "scenario name {} violates the artifact-name charset",
            scenario.name
        );
    }

    for class in [
        ScenarioClass::RenderMatrix,
        ScenarioClass::DeterminismDrill,
        ScenarioClass::FailurePath,
        ScenarioClass::LifecycleDrill,
    ] {
        let seeded = scenarios
            .iter()
            .filter(|s| s.class == class && s.regression.is_none())
            .count();
        let drills = scenarios
            .iter()
            .filter(|s| s.class == class && s.regression.is_some())
            .count();
        assert!(seeded >= 2, "{class:?} has {seeded} scenarios");
        assert_eq!(drills, 1, "{class:?} must carry exactly one drill");
    }

    // ParityDrill is the Parity Ledger's ruling class: never empty again,
    // and every ruling family keeps its stable drill id.
    let parity_seeded = scenarios
        .iter()
        .filter(|s| s.class == ScenarioClass::ParityDrill && s.regression.is_none())
        .count();
    assert!(
        parity_seeded >= 1,
        "ParityDrill has {parity_seeded} scenarios"
    );
    for id in PARITY_RULING_DRILL_IDS {
        assert!(
            scenarios.iter().any(|scenario| scenario.name == *id),
            "ParityDrill lost its stable ruling drill {id}"
        );
    }

    // Pending surfaces are declared, never stubbed: nothing registers against
    // them until their owning bead lands real behavior.
    assert!(
        scenarios
            .iter()
            .all(|s| !matches!(s.surface, Surface::PythonPending)),
        "a Pending surface acquired a stub scenario"
    );
    assert!(
        scenarios
            .iter()
            .any(|scenario| scenario.surface == Surface::StudioInProcess),
        "Studio lost its production composition scenario"
    );
    assert!(
        scenarios.iter().any(
            |scenario| scenario.name == "lifecycle.studio_native_worker.v1"
                && scenario.surface == Surface::StudioSubprocess
                && scenario.tier == Tier::Fast
                && scenario.regression.is_none()
        ),
        "Studio lost its mandatory native subprocess lifecycle scenario"
    );
    assert!(
        scenarios
            .iter()
            .any(|scenario| scenario.surface == Surface::PythonInProcess),
        "Python lost its production composition scenario"
    );
}

#[test]
fn python_class_sweep_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.class_sweep.v1")
        .expect("class sweep scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_structural_envelope_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.structural_envelope.v1")
        .expect("structural envelope scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_reference_corpus_regressions_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.reference_corpus_regressions.v1")
        .expect("Reference corpus regressions scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_structural_facts_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "parity.structural_facts.v1")
        .expect("structural facts scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_surface_hooks_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.python_surface_hooks.v1")
        .expect("Python surface hooks scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_scene_console_scenario_passes() {
    let scenario = catalog()
        .into_iter()
        .find(|scenario| scenario.name == "lifecycle.python_scene_console.v1")
        .expect("Python scene console scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

/// The typeset cache and preflight lifecycle scenarios
/// (fm-typeset-cache-preflight-wiring-1sn0), focused.
#[test]
fn typeset_cache_and_preflight_scenarios_pass() {
    for name in [
        "lifecycle.typeset_cache_warm_second_run.v1",
        "lifecycle.typeset_preflight_before_first_play.v1",
    ] {
        let scenario = catalog()
            .into_iter()
            .find(|scenario| scenario.name == name)
            .expect("the typeset scenario is registered");
        let report = Runner::from_env().run(scenario);
        assert!(report.is_pass(), "{}", report.summary());
    }
}

/// The CLI builtin scenarios (fm-cli-camera-route-i1zc,
/// fm-facade-render-api-9ewt), focused.
#[test]
fn cli_builtin_scenarios_pass() {
    for name in [
        "render_matrix.cli_camera_builtins.v1",
        "render_matrix.cli_primitive_builtins.v1",
        "determinism.cli_builtins_threads_1_4.v1",
        "render_matrix.png_sequence_frame_concurrency.v1",
    ] {
        let scenario = catalog()
            .into_iter()
            .find(|scenario| scenario.name == name)
            .expect("the CLI builtin scenario is registered");
        let report = Runner::from_env().run(scenario);
        assert!(report.is_pass(), "{}", report.summary());
    }
}

/// Re-bless evidence for the e2e lock's `render_matrix.cli_primitive.*` and
/// `render_matrix.cli_camera.*` entries (GOVERNANCE.md §4). Opt-in with
/// `FMN_REBLESS_E2E_CLI_BUILTINS=1`, after the bless: every builtin frame is
/// decoded and stacked, and the stack goes on both sides of the lock's
/// artefact, because these entries are new. A later change to them needs a
/// before dump instead.
#[test]
fn rebless_e2e_cli_builtins_panel() {
    use fmn_conformance::golden::{REBLESS_ARTEFACT_DIR, rebless_artefact_name, side_by_side_png};
    if std::env::var("FMN_REBLESS_E2E_CLI_BUILTINS").as_deref() != Ok("1") {
        return;
    }
    let (mut stack, mut width, mut height, mut rows) = (Vec::new(), 0, 0, 0);
    for name in cli_all_builtins() {
        let (_, frames) = cli_builtin_frames(name, "1").expect("the builtin renders");
        for frame in frames {
            let image = fmn_codec::decode_png(&frame, &fmn_codec::PngLimits::default())
                .expect("a locked frame decodes");
            (width, height) = (image.width, image.height);
            stack.extend(image.rgba);
            rows += 1;
        }
    }
    let panel = side_by_side_png(width, height * rows, &stack, &stack).expect("frames stack");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lock = "e2e.certified.lock";
    let digest =
        fmn_hash::sha256(&std::fs::read(manifest.join("goldens").join(lock)).expect("lock"))
            .to_hex();
    let out = manifest.join("../..").join(REBLESS_ARTEFACT_DIR);
    std::fs::create_dir_all(&out).expect("re-bless artefact directory");
    std::fs::write(out.join(rebless_artefact_name(lock, &digest)), panel).expect("panel written");
}
