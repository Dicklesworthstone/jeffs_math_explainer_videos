//! The W10 e2e harness's own proof (fm-fjq): the runner flow end to end —
//! GREEN runs through the real Rust-API render surface, deterministic
//! NDJSON artifacts, failure → FMNA repro bundling, both regression drills
//! confirmed RED end to end, tier gating, pending-surface skips, and the
//! pipeline-stats log-contract ride-along.
//!
//! Everything runs against scratch roots under `CARGO_TARGET_TMPDIR` with
//! explicit golden modes (never the environment), so the suite is
//! parallel-safe and never touches the committed `goldens/`.

use fmn_conformance::e2e::{
    Assertion, FieldPred, Invocation, LogEvent, LogExpect, RegressionKind, RunCtx, RunOutcome,
    RunReport, Runner, ScenarioClass, ScenarioError, ScenarioSpec, Status, StructuralAssert,
    Surface, Tier, counters, scenario_seed, spans,
};
use fmn_conformance::golden::{Mode, Scope};
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, TEAL_B, WHITE};
use fmn_hash::sha256::sha256;
use fmn_library::style::Style;
use fmn_library::{Circle, Ellipse, VStyle};
use fmn_mobject::Stage;
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob, encode_frame};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use fmn_runtime::{PipelineStage, PipelineStats, StageUtilization};
use fmn_scene::journal::{CommandKind, CommandRecord, EffectClass, Entry, ReproBundle};
use std::path::PathBuf;
use std::time::Duration;

const WIDTH: u32 = 96;
const HEIGHT: u32 = 54;
const SCALE: f64 = 20.0;
const TILING: Tiling = Tiling {
    macro_tile: 64,
    fine_tile: 8,
};
const SUITE: &str = "e2e_harness";
const ARTIFACT: &str = "lifecycle.frame.v1";

fn logs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("e2e_harness_logs")
}

fn goldens_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("e2e_harness_goldens")
}

fn runner(mode: Mode) -> Runner {
    Runner::new(logs_root(), goldens_root(), mode)
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
        Srgb::from_rgb8(0x22, 0x22, 0x22).to_linear(1.0),
    )
}

/// One real certified-engine frame of a small composed stage: the same
/// Lumen path `tests/scene_runtime.rs` drives, minimal geometry.
fn render_frame(stage: &Stage) -> Result<Vec<u8>, ScenarioError> {
    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0)
        .map_err(|error| ScenarioError::new(error.to_string()))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| ScenarioError::new(error.to_string()))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .map_err(|e| ScenarioError::new(e.to_string()))?;
    let job = FrameJob::new(&plan, &mono, &binning, config)
        .map_err(|e| ScenarioError::new(e.to_string()))?;
    let frame = job
        .render(1)
        .map_err(|e| ScenarioError::new(e.to_string()))?;
    encode_frame(&frame).map_err(|e| ScenarioError::new(e.to_string()))
}

/// A real Rust-API scenario: construct a composed stage through the
/// public lifecycle, render it on the certified CPU engine, emit the
/// canonical vocabulary, journal the run, and hand back one artifact.
fn lifecycle_scenario(name: &'static str) -> ScenarioSpec {
    ScenarioSpec::new(
        name,
        ScenarioClass::LifecycleDrill,
        Surface::RustApi,
        Invocation::new(|ctx: &mut RunCtx| {
            ctx.set_fps((8, 1));
            ctx.event(
                LogEvent::new(spans::PREFLIGHT)
                    .field("families", 2u64)
                    .field("assets", 0u64),
            );
            ctx.record_asset("lifecycle.scene.source", b"circle+ellipse composition");

            let circle = Circle::new()
                .radius(0.6)
                .style(Style::default().color(BLUE_C).fill_opacity(0.4))
                .build();
            let ellipse = Ellipse::new()
                .width(1.1)
                .height(0.5)
                .style(Style::default().color(TEAL_B).fill_opacity(0.3))
                .build();
            let mut stage = Stage::new();
            let back = stage.add(ellipse);
            stage.add_to_scene(back).expect("back joins the scene");
            let front = stage.add(circle);
            stage.add_to_scene(front).expect("front joins the scene");
            stage.set_stroke(front, Some(WHITE), Some(1.0), Some(0.9), None, true);
            ctx.event(LogEvent::new(spans::SCENE_CONSTRUCT).field("roots", 2u64));

            ctx.event(
                LogEvent::new(spans::ENGINE)
                    .field("identity", "certified")
                    .field("threads", 1u64),
            );
            let bytes = render_frame(&stage)?;
            ctx.event(
                LogEvent::new(spans::RENDER_FRAME)
                    .field("frames", 1u64)
                    .field("bytes", bytes.len()),
            );
            ctx.counter(counters::FRAMES_SUBMITTED, 1);
            ctx.counter(counters::FRAMES_PREPARED, 1);
            ctx.counter(counters::FRAMES_RASTERIZED, 1);
            ctx.counter(counters::FRAMES_CONVERTED, 1);
            ctx.counter(counters::FRAMES_EMITTED, 1);
            ctx.record_journal(Entry {
                command: CommandRecord {
                    kind: CommandKind::Play,
                    identity: sha256(b"lifecycle frame render"),
                    label: "render certified frame".to_string(),
                },
                effect: EffectClass::Pure,
                reads: Vec::new(),
                subprocesses: Vec::new(),
                checkpoint: None,
                state_hash: sha256(&bytes),
            });

            Ok(RunOutcome::ok()
                .with_artifact(ARTIFACT, bytes)
                .with_counter("frames", 1))
        }),
    )
}

/// The fully-asserted GREEN form of the lifecycle scenario.
fn green_scenario(name: &'static str) -> ScenarioSpec {
    lifecycle_scenario(name)
        .assertions(vec![
            Assertion::GoldenLock {
                suite: SUITE,
                scope: Scope::Certified,
            },
            Assertion::Structural(StructuralAssert::ArtifactCountEq(1)),
            Assertion::Structural(StructuralAssert::NoEmptyArtifacts),
            Assertion::Structural(StructuralAssert::CounterEq("frames", 1)),
            Assertion::ExitCode(0),
            Assertion::FileInventory(vec![ARTIFACT.to_string()]),
            Assertion::NdjsonSchema,
        ])
        .logs(vec![
            LogExpect::span_present(spans::PREFLIGHT, vec![FieldPred::u64_ge("families", 1)]),
            LogExpect::span_present(
                spans::ENGINE,
                vec![FieldPred::str_eq("identity", "certified")],
            ),
            LogExpect::event_order(spans::SCENE_CONSTRUCT, spans::RENDER_FRAME),
            LogExpect::counter_ge(counters::FRAMES_RASTERIZED, 1),
            LogExpect::no_event("pipeline.error"),
        ])
}

/// Bless the scratch lock for a scenario builder, asserting the bless run
/// itself goes through the full harness GREEN.
fn bless(name: &'static str, build: impl Fn(&'static str) -> ScenarioSpec) {
    let report = runner(Mode::Bless).run_gated(build(name), true);
    assert!(
        report.is_pass(),
        "bless run should pass: {}",
        report.summary()
    );
}

/// Extract the `Failure` from a report expected to be `Status::Failed`.
fn expect_failed(report: &RunReport) -> &fmn_conformance::e2e::Failure {
    match &report.status {
        Status::Failed(failure) => Some(failure),
        _ => None,
    }
    .expect("expected Status::Failed")
}

/// Extract the `Failure` from a report expected to be a confirmed drill.
fn expect_confirmed(report: &RunReport) -> &fmn_conformance::e2e::Failure {
    match &report.status {
        Status::RegressionConfirmed { failure } => Some(failure),
        _ => None,
    }
    .expect("expected Status::RegressionConfirmed")
}

/// Extract the log path from a report expected to be `Status::Passed`.
fn expect_passed(report: &RunReport) -> &PathBuf {
    match &report.status {
        Status::Passed { log } => Some(log),
        _ => None,
    }
    .expect("expected Status::Passed")
}

#[test]
fn green_path_full_flow() {
    bless("green.lifecycle.v1", green_scenario);
    let report = runner(Mode::Check).run_gated(green_scenario("green.lifecycle.v1"), true);
    assert!(report.is_pass(), "expected pass: {}", report.summary());
    let log = expect_passed(&report);
    assert!(log.exists(), "the run log artifact exists");
    let body = std::fs::read_to_string(log).expect("run log reads");
    assert!(body.contains("\"kind\":\"event\""));
    assert!(body.contains(spans::HARNESS_BEGIN));
    assert!(body.contains("\"class\":\"lifecycle_drill\""));
    assert!(body.contains("\"surface\":\"rust_api\""));
    assert!(body.contains("\"status\":\"passed\""));
}

#[test]
fn ndjson_artifact_is_deterministic_across_reruns() {
    bless("determinism.log.v1", green_scenario);
    for run in 1..=2 {
        let report = runner(Mode::Check).run_gated(green_scenario("determinism.log.v1"), true);
        assert!(report.is_pass(), "run {run}: {}", report.summary());
        let log = expect_passed(&report);
        let bytes = std::fs::read(log).expect("run log reads");
        if run == 1 {
            std::fs::write(logs_root().join("determinism.first.copy"), &bytes)
                .expect("copy writes");
        } else {
            let first =
                std::fs::read(logs_root().join("determinism.first.copy")).expect("copy reads");
            assert_eq!(first, bytes, "the NDJSON artifact is byte-deterministic");
        }
    }
}

#[test]
fn failing_assertion_bundles_repro() {
    let spec = lifecycle_scenario("fail.lifecycle.v1")
        .assert(Assertion::Structural(StructuralAssert::ArtifactCountEq(2)));
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(!report.is_pass());
    let failure = expect_failed(&report);
    assert_eq!(failure.class, ScenarioClass::LifecycleDrill);
    assert!(
        failure
            .reasons
            .iter()
            .any(|r| r.contains("artifact-count == 2")),
        "the failing assertion is named: {:?}",
        failure.reasons
    );
    assert!(failure.log.exists(), "the failure artifact exists");
    assert!(failure.repro.exists(), "the repro bundle exists");
    let message = failure.to_string();
    assert!(
        message.contains(&failure.repro.display().to_string()),
        "the failure output names the bundle path"
    );
    // The bundle is a real FMNA container carrying the run.
    let bytes = std::fs::read(&failure.repro).expect("bundle reads");
    let bundle = ReproBundle::from_bytes(&bytes).expect("bundle parses");
    assert_eq!(bundle.scene_label, "fail.lifecycle.v1");
    assert_eq!(bundle.seed, scenario_seed("fail.lifecycle.v1"));
    assert_eq!(
        bundle.fps,
        (8, 1),
        "the invocation's frame rate rides along"
    );
    assert_eq!(bundle.closure.len(), 1, "the input closure was recorded");
    assert!(
        bundle
            .journal
            .entries()
            .first()
            .expect("harness journal entry")
            .command
            .label
            .contains("lifecycle_drill"),
        "the journal names the scenario class"
    );
    assert_eq!(
        bundle.journal.entries().len(),
        2,
        "harness + invocation entries"
    );
}

#[test]
fn golden_drift_drill_is_confirmed_red() {
    // An honest lock exists first; the drill then flips one artifact byte
    // and the D-16 rig must catch the drift.
    bless("drill.golden.v1", green_scenario);
    let report = runner(Mode::Check).run_gated(
        green_scenario("drill.golden.v1").regression(RegressionKind::GoldenDrift),
        true,
    );
    let failure = expect_confirmed(&report);
    assert_eq!(failure.class, ScenarioClass::LifecycleDrill);
    assert!(
        failure
            .reasons
            .iter()
            .any(|r| r.contains("self-golden drift")),
        "RED via the golden rig: {:?}",
        failure.reasons
    );
    assert!(failure.log.exists() && failure.repro.exists());
    // The D-16 rig's drift sidecar proves the corrupted bytes were captured.
    let sidecars = goldens_root().join(format!("{SUITE}.certified.actual"));
    assert!(sidecars.is_dir(), "the rig wrote a sidecar dir");
    assert!(
        std::fs::read_dir(&sidecars)
            .expect("sidecar dir reads")
            .count()
            >= 1,
        "the sidecar holds the drifted artifact"
    );
}

#[test]
fn log_expectation_drill_is_confirmed_red() {
    // The same scenario passes GREEN without the injection...
    bless("drill.logexpect.v1", green_scenario);
    let green = runner(Mode::Check).run_gated(green_scenario("drill.logexpect.v1"), true);
    assert!(green.is_pass(), "untampered run: {}", green.summary());
    // ...and the drill's log corruption turns it RED via the LogExpect.
    let drill = runner(Mode::Check).run_gated(
        green_scenario("drill.logexpect.v1").regression(RegressionKind::LogExpectation),
        true,
    );
    let failure = expect_confirmed(&drill);
    assert!(
        failure
            .reasons
            .iter()
            .any(|r| r.contains("log expectation")),
        "RED via the log contract: {:?}",
        failure.reasons
    );
    assert!(failure.log.exists() && failure.repro.exists());
}

#[test]
fn drill_red_for_the_wrong_reason_is_a_harness_error() {
    // The invocation errors before emitting the targeted span, so the
    // injected drop is vacuous — the RED cannot be attributed to the
    // injection and the drill must NOT confirm.
    let spec = ScenarioSpec::new(
        "drill.vacuous.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(|_ctx| Err(ScenarioError::new("boom"))),
    )
    .log_expect(LogExpect::span_present("never.emitted", vec![]))
    .regression(RegressionKind::LogExpectation);
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(
        matches!(report.status, Status::HarnessError(_)),
        "a vacuous injection must not confirm: {}",
        report.summary()
    );

    // A golden-drift drill whose invocation errors produces no artifact
    // to corrupt: harness error, not a confirmation.
    let spec = ScenarioSpec::new(
        "drill.noartifact.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(|_ctx| Err(ScenarioError::new("boom"))),
    )
    .assert(Assertion::GoldenLock {
        suite: SUITE,
        scope: Scope::Certified,
    })
    .regression(RegressionKind::GoldenDrift);
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(
        matches!(report.status, Status::HarnessError(_)),
        "a drill with nothing to corrupt must not confirm: {}",
        report.summary()
    );
}

#[test]
fn drill_without_a_target_is_spec_invalid() {
    let no_golden = lifecycle_scenario("drill.untargeted.v1")
        .assert(Assertion::Structural(StructuralAssert::ArtifactCountEq(1)))
        .regression(RegressionKind::GoldenDrift);
    let report = runner(Mode::Check).run_gated(no_golden, true);
    assert!(matches!(report.status, Status::SpecInvalid(_)));

    let no_log_target = lifecycle_scenario("drill.untargeted2.v1")
        .log_expect(LogExpect::event_order(
            spans::SCENE_CONSTRUCT,
            spans::RENDER_FRAME,
        ))
        .regression(RegressionKind::LogExpectation);
    let report = runner(Mode::Check).run_gated(no_log_target, true);
    assert!(matches!(report.status, Status::SpecInvalid(_)));
}

#[test]
fn pending_surfaces_skip_without_running() {
    let spec = ScenarioSpec::new(
        "pending.surface.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonPending,
        // Would fail if it ever ran: pending surfaces must be skipped,
        // never stubbed into a pass.
        Invocation::new(|_ctx| Err(ScenarioError::new("must not run"))),
    );
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(
        matches!(report.status, Status::Skipped(_)),
        "{}",
        report.summary()
    );
    assert!(
        !logs_root().join("pending.surface.v1.ndjson").exists(),
        "a skipped run leaves no log artifact"
    );
}

#[test]
fn tier_gating() {
    let full = lifecycle_scenario("tier.full.v1").tier(Tier::Full);
    let skipped = runner(Mode::Check).run_gated(full, false);
    assert!(
        matches!(skipped.status, Status::Skipped(_)),
        "{}",
        skipped.summary()
    );

    let full = lifecycle_scenario("tier.full.v1")
        .tier(Tier::Full)
        .assert(Assertion::Structural(StructuralAssert::ArtifactCountEq(1)));
    let ran = runner(Mode::Check).run_gated(full, true);
    assert!(ran.is_pass(), "{}", ran.summary());

    let fast = lifecycle_scenario("tier.fast.v1")
        .assert(Assertion::Structural(StructuralAssert::ArtifactCountEq(1)));
    assert!(runner(Mode::Check).run_gated(fast, false).is_pass());
}

#[test]
fn invalid_name_is_spec_invalid() {
    let spec = ScenarioSpec::new(
        "Bad/Name",
        ScenarioClass::RenderMatrix,
        Surface::RustApi,
        Invocation::new(|_ctx| Ok(RunOutcome::ok())),
    );
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(matches!(report.status, Status::SpecInvalid(_)));
}

#[test]
fn invocation_error_fails_and_bundles() {
    let spec = ScenarioSpec::new(
        "fail.invocation.v1",
        ScenarioClass::FailurePath,
        Surface::RustApi,
        Invocation::new(|_ctx| Err(ScenarioError::new("named failure T2"))),
    );
    let report = runner(Mode::Check).run_gated(spec, true);
    let failure = expect_failed(&report);
    assert!(
        failure
            .reasons
            .iter()
            .any(|r| r.contains("named failure T2")),
        "{:?}",
        failure.reasons
    );
    assert!(failure.log.exists() && failure.repro.exists());
}

#[test]
fn exit_code_and_inventory_assertions() {
    let spec = ScenarioSpec::new(
        "failure.exclusive.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(|ctx| {
            ctx.event(LogEvent::new(spans::PREFLIGHT).field("rules", 1u64));
            Ok(RunOutcome::ok().exit_code(2))
        }),
    )
    .assert(Assertion::ExitCode(2))
    .assert(Assertion::FileInventory(vec![]))
    .log_expect(LogExpect::span_present(spans::PREFLIGHT, vec![]));
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(report.is_pass(), "{}", report.summary());

    let wrong = ScenarioSpec::new(
        "failure.exclusive2.v1",
        ScenarioClass::FailurePath,
        Surface::CliInProcess,
        Invocation::new(|_ctx| Ok(RunOutcome::ok().exit_code(0))),
    )
    .assert(Assertion::ExitCode(2));
    let report = runner(Mode::Check).run_gated(wrong, true);
    assert!(matches!(report.status, Status::Failed(_)));
}

#[test]
fn pipeline_stats_ride_the_log_contract() {
    // The e2e vocabulary mirrors fm-bgr's pipeline types field-for-field;
    // this is the documented bridge every FramePipeline scenario writes.
    fn record_pipeline_stats(ctx: &mut RunCtx, stats: &PipelineStats) {
        ctx.counter(counters::FRAMES_SUBMITTED, stats.submitted);
        ctx.counter(counters::FRAMES_PREPARED, stats.prepared);
        ctx.counter(counters::FRAMES_RASTERIZED, stats.rasterized);
        ctx.counter(counters::FRAMES_CONVERTED, stats.converted);
        ctx.counter(counters::FRAMES_EMITTED, stats.emitted);
        ctx.counter(counters::BARRIERS, stats.barriers);
        ctx.counter(counters::BACKPRESSURE_WAITS, stats.backpressure_waits);
        ctx.event(
            LogEvent::new(spans::PIPELINE_STATS)
                .field("submitted", stats.submitted)
                .field("emitted", stats.emitted)
                .field("max_in_flight", stats.max_in_flight),
        );
    }

    let utilization = |jobs: u64, workers: usize| StageUtilization {
        jobs,
        busy: Duration::ZERO,
        workers,
    };
    let spec = ScenarioSpec::new(
        "pipeline.stats.v1",
        ScenarioClass::DeterminismDrill,
        Surface::RustApi,
        Invocation::new(move |ctx| {
            let stats = PipelineStats {
                elapsed: Duration::ZERO,
                first_output_latency: None,
                submitted: 3,
                prepared: 3,
                rasterized: 3,
                converted: 3,
                emitted: 3,
                barriers: 1,
                backpressure_waits: 0,
                max_in_flight: 2,
                outstanding_slots: 0,
                scene: utilization(3, 1),
                prepare: utilization(3, 1),
                raster: utilization(3, 1),
                convert: utilization(3, 1),
                emit: utilization(3, 1),
                barrier: utilization(1, 1),
                render_team_frames: vec![3],
                render_team_busy: vec![Duration::ZERO],
            };
            ctx.event(LogEvent::new(spans::EXECUTION_PLAN).field("render_teams", 1u64));
            record_pipeline_stats(ctx, &stats);
            Ok(RunOutcome::ok().with_counter("frames", stats.emitted))
        }),
    )
    .logs(vec![
        LogExpect::counter_ge(counters::FRAMES_EMITTED, 3),
        LogExpect::counter_ge(counters::BARRIERS, 1),
        LogExpect::span_present(
            spans::PIPELINE_STATS,
            vec![
                FieldPred::u64_eq("submitted", 3),
                FieldPred::u64_eq("max_in_flight", 2),
            ],
        ),
        LogExpect::event_order(spans::EXECUTION_PLAN, spans::PIPELINE_STATS),
    ]);
    let report = runner(Mode::Check).run_gated(spec, true);
    assert!(report.is_pass(), "{}", report.summary());

    // The stage vocabulary lines up with fm-bgr's own display names.
    assert_eq!(PipelineStage::Raster.to_string(), "raster");
    assert_eq!(PipelineStage::Emit.to_string(), "emit");
}
