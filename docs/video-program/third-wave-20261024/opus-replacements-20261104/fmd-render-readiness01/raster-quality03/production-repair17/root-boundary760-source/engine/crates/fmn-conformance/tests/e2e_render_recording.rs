//! Companion fast-tier catalog for render-only FMTL exports.
//! A ScenarioSpec runs the real Rust API, codec and renderer through the same
//! Gauntlet Runner as e2e_scenarios.rs, including bounded NDJSON and automatic
//! FMNA failure bundles. No new harness or golden-blessing path is introduced.
use fmn_conformance::e2e::{
    Assertion, FieldPred, Invocation, LogEvent, LogExpect, RunCtx, RunOutcome, Runner,
    ScenarioClass, ScenarioError, ScenarioSpec, StructuralAssert, Surface, counters, spans,
};
use fmn_conformance::golden::Mode;
use fmn_core::color::Srgb;
use fmn_core::constants::BLUE_C;
use fmn_hash::sha256;
use fmn_library::Circle;
use fmn_library::style::Style;
use fmn_mobject::{Mobject, Stage};
use fmn_render::{
    Binning, FrameConfig, FrameJob, MonoTable, RenderPlan, ScreenMap, Tiling, Viewport,
};
use fmn_scene::journal::{CommandKind, CommandRecord, EffectClass, Entry};
use fmn_scene::recording::SceneBundleRecorder;
use fmn_scene::{BundleExportLimits, TimelineBundle};
use std::path::PathBuf;

fn error(value: impl std::fmt::Display) -> ScenarioError {
    ScenarioError::new(value.to_string())
}

fn render(stage: &Stage) -> Result<Vec<u8>, ScenarioError> {
    let config = FrameConfig::new(
        Viewport {
            width: 96,
            height: 54,
        },
        ScreenMap::y_up(20.0, [48.0, 27.0]),
        Srgb::from_rgb8(0x22, 0x22, 0x22).to_linear(1.0),
    );
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0).map_err(error)?;
    let mono = MonoTable::build(&plan, config.map).map_err(error)?;
    let mut binning = Binning::build(
        &plan,
        config.viewport,
        Tiling {
            macro_tile: 64,
            fine_tile: 8,
        },
        config.map,
    )
    .map_err(error)?;
    binning.prune_occluded(&plan).map_err(error)?;
    let frame = FrameJob::new(&plan, &mono, &binning, config)
        .map_err(error)?
        .render(1)
        .map_err(error)?;
    fmn_render::engine::encode_frame(&frame).map_err(error)
}

fn invoke(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    ctx.set_fps((8, 1));
    ctx.record_asset(
        "render-recording.scene.source",
        include_bytes!("e2e_render_recording.rs"),
    );
    ctx.event(LogEvent::new(spans::PREFLIGHT).field("history", 128u64));
    let mut stage = Stage::new();
    for _ in 0..128 {
        stage.add(Mobject::from_points(&[[42.0, 42.0, 0.0]]));
    }
    let root = stage.add(
        Circle::new()
            .radius(0.6)
            .style(Style::default().color(BLUE_C).fill_opacity(0.6))
            .build(),
    );
    stage.add_to_scene(root).map_err(error)?;
    ctx.event(LogEvent::new(spans::SCENE_CONSTRUCT).field("roots", stage.roots().len()));
    let before = stage.snapshot().to_bytes().map_err(error)?;
    let limits = BundleExportLimits::default();
    let mut full = SceneBundleRecorder::new(8, limits).map_err(error)?;
    let mut compact = SceneBundleRecorder::new_render_only(8, limits).map_err(error)?;
    full.capture_terminal_still(&stage).map_err(error)?;
    compact.capture_terminal_still(&stage).map_err(error)?;
    let full = full.finish().map_err(error)?;
    let compact = compact.finish().map_err(error)?;
    if compact.frame_count != 1 || full.bytes.len() <= compact.bytes.len() {
        return Err(ScenarioError::new(
            "render projection did not remove captured arena history",
        ));
    }
    if !stage.contains(root) || before != stage.snapshot().to_bytes().map_err(error)? {
        return Err(ScenarioError::new(
            "recording mutated the live stage or its ordinary snapshot",
        ));
    }
    ctx.event(
        LogEvent::new("recording.projection")
            .field("mode", "render-only")
            .field("full_bytes", full.bytes.len())
            .field("render_bytes", compact.bytes.len())
            .field("sha256", compact.digest.to_hex()),
    );
    let ordinary = TimelineBundle::from_bytes(&full.bytes).map_err(error)?;
    let replay = TimelineBundle::from_bytes(&compact.bytes).map_err(error)?;
    let expected = render(&stage)?;
    let ordinary_frame = render(&ordinary.stage_at(0).map_err(error)?)?;
    let replay_frame = render(&replay.stage_at(0).map_err(error)?)?;
    if expected != ordinary_frame || expected != replay_frame {
        return Err(ScenarioError::new(
            "render-only replay differs from the observed/full-state frame",
        ));
    }
    let mut repeat = SceneBundleRecorder::new_render_only(8, limits).map_err(error)?;
    repeat.capture_terminal_still(&stage).map_err(error)?;
    if repeat
        .finish_with_max_bytes(compact.bytes.len())
        .map_err(error)?
        .bytes
        != compact.bytes
    {
        return Err(ScenarioError::new(
            "exact-budget render-only replay is not deterministic",
        ));
    }
    ctx.event(
        LogEvent::new(spans::RENDER_FRAME)
            .field("frames", 1u64)
            .field("equal", true),
    );
    ctx.counter(counters::FRAMES_RASTERIZED, 1);
    ctx.record_journal(Entry {
        command: CommandRecord {
            kind: CommandKind::Play,
            identity: sha256(b"render-only FMTL recording"),
            label: "render-only FMTL recording".to_owned(),
        },
        effect: EffectClass::Pure,
        reads: Vec::new(),
        subprocesses: Vec::new(),
        checkpoint: None,
        state_hash: sha256(&replay_frame),
    });
    Ok(RunOutcome::ok()
        .with_artifact("render-only.fmtl", compact.bytes)
        .with_artifact("render-only.frame", replay_frame)
        .with_counter("frames", 1))
}

fn scenario() -> ScenarioSpec {
    ScenarioSpec::new(
        "recording.render-only.v1",
        ScenarioClass::ParityDrill,
        Surface::RustApi,
        Invocation::new(invoke),
    )
    .assertions(vec![
        Assertion::ExitCode(0),
        Assertion::Structural(StructuralAssert::ArtifactCountEq(2)),
        Assertion::Structural(StructuralAssert::NoEmptyArtifacts),
        Assertion::Structural(StructuralAssert::CounterEq("frames", 1)),
        Assertion::FileInventory(vec![
            "render-only.fmtl".to_owned(),
            "render-only.frame".to_owned(),
        ]),
        Assertion::NdjsonSchema,
    ])
    .logs(vec![
        LogExpect::span_present(
            "recording.projection",
            vec![FieldPred::str_eq("mode", "render-only")],
        ),
        LogExpect::span_present(spans::RENDER_FRAME, vec![FieldPred::bool_eq("equal", true)]),
        LogExpect::event_order(spans::SCENE_CONSTRUCT, "recording.projection"),
        LogExpect::event_order("recording.projection", spans::RENDER_FRAME),
        LogExpect::counter_ge(counters::FRAMES_RASTERIZED, 1),
    ])
}

#[test]
fn render_only_recording_fast_scenario() {
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let runner = Runner::new(
        scratch.join("render_recording_e2e_logs"),
        scratch.join("render_recording_e2e_goldens"),
        Mode::Check,
    );
    let report = runner.run_gated(scenario(), false);
    assert!(report.is_pass(), "{}", report.summary());
}
