//! Canonical PG-5 per-commit determinism producer.
//!
//! This module connects the common performance evidence format to the three
//! production mechanisms whose schedule independence PG-5 promises:
//!
//! - direct certified renders of every committed scene at `{1, 4, 16}`
//!   requested threads;
//! - whole-frame fan-out through [`fmn_runtime::FramePipeline`] over a
//!   certified two-team [`ExecutionPlan`];
//! - that same bounded pipeline writing directly into the preallocated
//!   [`fmn_output::OrderedEmitter`] ring and publishing in frame order.
//!
//! The three policy samples are mismatch counts, one per mechanism. Candidate
//! mismatches remain valid samples so the common exact-policy verifier can
//! issue the blocking verdict. Separately, the one-thread reference digest is
//! self-goldened, which prevents a corpus or renderer drift from silently
//! redefining what every schedule is compared against.
//!
//! Weekly `{32, 96}+` execution and certified-platform DSR receipts are a
//! separate lifecycle surface. This producer does not manufacture either.

use crate::perf::{
    Baseline, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GateScope,
    MeasurementBatch, MetricUnit, PerfError, Sample, require_compiled_cargo_profile,
    validate_producer_commit,
};
use crate::scene_goldens::{self, SCENES, TILING};
use fmn_frame::FrameBuffer;
use fmn_hash::{Digest, Sha256, sha256};
use fmn_mobject::Stage;
use fmn_output::{
    EmitterConfig, EmitterHandle, EmitterReport, FrameReservation, FrameSink, OrderedEmitter,
    SinkBinding, SinkFailure, SinkWrite,
};
use fmn_platform::topology::{HardwareTopology, SimdTier};
use fmn_render::{
    Binning, EngineIdentity, FrameConfig, FrameJob, MonoTable, RenderPlan, Tier,
    engine::encode_frame, frame_digest,
};
use fmn_runtime::{
    Determinism, ExecutionEngine, ExecutionPlan, FramePipeline, LocalityLane, OutputPixelFormat,
    PipelineEvent, PipelineStages, PipelineStats, PlanRequest, RenderIntent, SurfaceSpec, TeamPlan,
    TeamRole, TopologyFingerprint, TuningSource,
};
use std::fmt;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Stable workload-definition schema.
pub const PG5_DEFINITION_SCHEMA: &str = "fmn-perf-pg5-definition/2";
/// Stable phase-trace schema.
pub const PG5_TRACE_SCHEMA: &str = "fmn-perf-pg5-trace/1";
/// Policy-catalog scenario implemented by this producer.
pub const PG5_SCENARIO: &str = "certified-thread-matrix";
/// One exact mismatch sample for each declared schedule mechanism.
pub const PG5_SAMPLE_COUNT: usize = 3;
/// The complete committed scene-golden corpus is the direct workload.
pub const PG5_CORPUS_SCENES: usize = 27;
/// Permanent per-commit direct-render thread matrix.
pub const PG5_DIRECT_THREADS: [usize; 3] = [1, 4, 16];
/// Synthetic physical-core count used only to derive the fixed certified plan.
pub const PG5_PIPELINE_CORES: u32 = 64;
/// The fixed certified in-flight limit for both schedule lanes.
pub const PG5_PIPELINE_DEPTH: usize = 2;
/// Certified offline planning yields two independent render teams.
pub const PG5_PIPELINE_TEAMS: usize = 2;
/// Workers in each certified render team on the synthetic topology.
pub const PG5_THREADS_PER_TEAM: usize = 32;

const BUILD_PROFILE: &str = "release-perf";
const THREAD_PROFILE: &str = "matrix-1-4-16-frame-parallel-ordered-pipeline";
const CACHE_STATE: &str = "independent-cold-scenes";
const OUTPUT_MODE: &str = "scene-golden-anchored-documents";

/// The complete content-addressed definition of the PG-5 per-commit workload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pg5Definition {
    execution_plan_digest: Digest,
}

impl Pg5Definition {
    /// Construct the sole canonical per-commit PG-5 definition.
    ///
    /// # Errors
    /// Returns a typed schedule error if the fixed certified plan can no longer
    /// be derived or no longer satisfies the declared two-team contract.
    pub fn new() -> Result<Self, Pg5Error> {
        let plan = pipeline_plan()?;
        validate_pipeline_plan(&plan)?;
        Ok(Self::from_execution_plan(&plan))
    }

    fn from_execution_plan(plan: &ExecutionPlan) -> Self {
        Self {
            execution_plan_digest: sha256(execution_plan_tsv(plan).as_bytes()),
        }
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(self) -> String {
        let mut output = String::new();
        {
            let mut row = |name: &str, value: &dyn fmt::Display| {
                let _ = writeln!(output, "{name}\t{value}");
            };
            row("schema", &PG5_DEFINITION_SCHEMA);
            row("gate", &GateId::Pg5);
            row("scenario", &PG5_SCENARIO);
            row("unit", &MetricUnit::Mismatches.name());
            row("target", &0);
            row("sample_count", &PG5_SAMPLE_COUNT);
            row("sample_0", &"direct-thread-matrix");
            row("sample_1", &"frame-parallel");
            row("sample_2", &"ordered-pipeline");
            row("direct_threads", &"1,4,16");
            row("corpus_scenes", &PG5_CORPUS_SCENES);
            row("engine", &pg5_identity().engine.name());
            row("tier", &pg5_identity().tier.name());
            row("thread_profile", &THREAD_PROFILE);
            row("cache_state", &CACHE_STATE);
            row("output_mode", &OUTPUT_MODE);
            row("frame_width_px", &scene_goldens::WIDTH);
            row("frame_height_px", &scene_goldens::HEIGHT);
            row("pipeline_physical_cores", &PG5_PIPELINE_CORES);
            row("pipeline_frames_in_flight", &PG5_PIPELINE_DEPTH);
            row("pipeline_render_teams", &PG5_PIPELINE_TEAMS);
            row("pipeline_threads_per_team", &PG5_THREADS_PER_TEAM);
            row("scene_golden_lock_digest", &self.corpus_lock_digest());
            row("config_digest", &self.config_digest());
            row("execution_plan_digest", &self.execution_plan_digest());
        }
        for (index, case) in SCENES.iter().enumerate() {
            let _ = writeln!(output, "scene\t{index}\t{}", case.name);
        }
        output
    }

    /// SHA-256 of [`Self::to_tsv`].
    #[must_use]
    pub fn digest(self) -> Digest {
        sha256(self.to_tsv().as_bytes())
    }

    /// Exact renderer/configuration identity for every produced raw frame.
    #[must_use]
    pub fn config_digest(self) -> Digest {
        fmn_render::engine::journal_digest(pg5_identity(), &scene_goldens::frame_config(), TILING)
    }

    /// Digest of the committed certified corpus lock that fixes scene input.
    #[must_use]
    pub fn corpus_lock_digest(self) -> Digest {
        scene_goldens::certified_lock_digest()
    }

    /// Canonical identity of the fixed certified two-team schedule.
    #[must_use]
    pub const fn execution_plan_digest(self) -> Digest {
        self.execution_plan_digest
    }
    /// Recompute the reference aggregate from produced cases — the trace
    /// identity the common verifier replays. The committed scene-golden
    /// lock is the anchor; this aggregate is derived, never pinned.
    ///
    /// # Errors
    /// Propagates fixture errors from the corpus field hashing.
    pub fn reference_digest(&self, cases: &[Pg5CaseResult]) -> Result<Digest, Pg5Error> {
        aggregate_reference_digest(self.corpus_lock_digest(), cases)
    }

    /// Validate the embedded corpus lock and fixed scene count.
    ///
    /// # Errors
    /// Returns a typed fixture error before any scene is built.
    pub fn validate_corpus_lock(self) -> Result<(), Pg5Error> {
        if SCENES.len() != PG5_CORPUS_SCENES {
            return Err(Pg5Error::Fixture(format!(
                "compiled corpus has {} scenes, expected {PG5_CORPUS_SCENES}",
                SCENES.len()
            )));
        }
        scene_goldens::validate_certified_lock().map_err(Pg5Error::Fixture)
    }

    /// Validate that a baseline names precisely this producer.
    ///
    /// # Errors
    /// Returns a typed identity error before any corpus scene is built.
    pub fn validate_baseline(self, baseline: &Baseline) -> Result<(), Pg5Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg5 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != PG5_SCENARIO {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::Mismatches {
            mismatches.push("unit");
        }
        if baseline.policy.direction != Direction::Exactly {
            mismatches.push("direction");
        }
        if baseline.policy.target != Some(0) {
            mismatches.push("target");
        }
        if baseline.policy.min_valid_samples != PG5_SAMPLE_COUNT {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != 0 {
            mismatches.push("max_invalid_samples");
        }
        if baseline.policy.max_mad_bps != 0 {
            mismatches.push("max_mad_bps");
        }
        if baseline.policy.alert_regression_bps != 0 {
            mismatches.push("alert_regression_bps");
        }
        if baseline.policy.block_regression_bps != 0 {
            mismatches.push("block_regression_bps");
        }
        if baseline.policy.enforcement != Enforcement::Blocking {
            mismatches.push("enforcement");
        }
        if baseline.policy.scope != GateScope::Core {
            mismatches.push("scope");
        }
        if baseline.policy.require_regression_profile {
            mismatches.push("require_regression_profile");
        }
        // ubs:ignore — public benchmark identity, not authentication material.
        if key.benchmark_definition != self.digest() {
            mismatches.push("benchmark_definition");
        }
        // ubs:ignore — public configuration identity, not authentication material.
        if key.config_digest != self.config_digest() {
            mismatches.push("config_digest");
        }
        // ubs:ignore — public execution-plan identity, not authentication material.
        if key.execution_plan_digest != self.execution_plan_digest() {
            mismatches.push("execution_plan_digest");
        }
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != pg5_identity().engine.name() {
            mismatches.push("engine");
        }
        if key.tier != pg5_identity().tier.name() {
            mismatches.push("tier");
        }
        if key.thread_profile != THREAD_PROFILE {
            mismatches.push("thread_profile");
        }
        if key.cache_state != CACHE_STATE {
            mismatches.push("cache_state");
        }
        if key.output_mode != OUTPUT_MODE {
            mismatches.push("output_mode");
        }
        if key.external_tool_fingerprint.is_some() {
            mismatches.push("external_tool_fingerprint");
        }
        if mismatches.is_empty() {
            Ok(())
        } else {
            Err(Pg5Error::Identity(format!(
                "{PG5_SCENARIO} baseline differs from the compiled producer in: {}",
                mismatches.join(", ")
            )))
        }
    }
}

/// One corpus scene's retained reference and schedule identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg5CaseResult {
    /// Stable scene-golden case name.
    pub scene: &'static str,
    /// Certified compiled-tier reference at one requested thread.
    pub one_thread: Digest,
    /// Same frame at four requested threads.
    pub four_threads: Digest,
    /// Same frame at sixteen requested threads.
    pub sixteen_threads: Digest,
    /// Same frame through the certified two-team runtime pipeline.
    pub frame_parallel: Digest,
    /// Same frame through the runtime pipeline and ordered emitter ring.
    pub ordered_pipeline: Digest,
}

/// Measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg5Artifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Aggregate identity of the one-thread reference corpus.
    pub reference_digest: Digest,
    /// Per-scene reference and schedule identities.
    pub cases: Vec<Pg5CaseResult>,
}

/// Measure all three per-commit PG-5 schedule mechanisms.
///
/// `trace_path` is recorded and content-addressed, but this function performs
/// no filesystem I/O. The CLI publishes the returned trace before the raw
/// bundle using exclusive-create semantics.
///
/// # Errors
/// Returns before scene construction for identity, path, profile, corpus-lock,
/// or execution-plan errors. Renderer/pipeline/emitter errors are explicit.
/// Candidate mismatches remain valid samples so the common exact verifier can
/// issue the blocking verdict.
pub fn measure_pg5(
    baseline: &Baseline,
    producer_commit: &str,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg5Artifacts, Pg5Error> {
    let execution_plan = pipeline_plan()?;
    validate_pipeline_plan(&execution_plan)?;
    let definition = Pg5Definition::from_execution_plan(&execution_plan);
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_compiled_cargo_profile(BUILD_PROFILE)?;
    definition.validate_corpus_lock()?;

    let mut direct = Vec::with_capacity(PG5_CORPUS_SCENES);
    for index in 0..SCENES.len() {
        let prepared = prepare_scene(index).map_err(Pg5Error::Fixture)?;
        direct.push((
            render_prepared(&prepared, PG5_DIRECT_THREADS[0])?,
            render_prepared(&prepared, PG5_DIRECT_THREADS[1])?,
            render_prepared(&prepared, PG5_DIRECT_THREADS[2])?,
        ));
    }

    // Reference anchor: every scene's certified artifact document must
    // reproduce the committed cross-platform lock — the same quantity the
    // fast-tier scene_goldens gate pins. The schedule sweep below then
    // measures whether every execution schedule reproduces the committed
    // bits, with schedule mismatches flowing to the common verifier.
    let corpus = scene_goldens::corpus();
    let golden_store = scene_goldens::store();
    for (index, case) in SCENES.iter().enumerate() {
        let bytes = scene_goldens::artifact(case, corpus, index, &certified_document);
        if let Err(error) = golden_store.check(case.name, &bytes) {
            return Err(Pg5Error::Render(format!(
                "reference corpus self-golden drift at {}: {error}",
                case.name
            )));
        }
    }

    let frame_parallel = run_frame_parallel(&execution_plan)?;
    let ordered = run_ordered_pipeline(&execution_plan)?;
    if frame_parallel.digests.len() != SCENES.len() || ordered.digests.len() != SCENES.len() {
        return Err(Pg5Error::Schedule(format!(
            "schedule output count mismatch: corpus {}, frame-parallel {}, ordered {}",
            SCENES.len(),
            frame_parallel.digests.len(),
            ordered.digests.len()
        )));
    }

    let mut cases = Vec::with_capacity(PG5_CORPUS_SCENES);
    for (index, case) in SCENES.iter().enumerate() {
        let (one_thread, four_threads, sixteen_threads) = direct[index];
        cases.push(Pg5CaseResult {
            scene: case.name,
            one_thread,
            four_threads,
            sixteen_threads,
            frame_parallel: frame_parallel.digests[index],
            ordered_pipeline: ordered.digests[index],
        });
    }

    let reference_digest = aggregate_reference_digest(definition.corpus_lock_digest(), &cases)?;
    // The per-scene anchor above is the committed-provenance check; the
    // aggregate below is derived trace identity, never a pinned constant.
    let direct_mismatches = cases
        .iter()
        .map(|case| {
            u64::from(case.four_threads != case.one_thread)
                + u64::from(case.sixteen_threads != case.one_thread)
        })
        .sum();
    let frame_parallel_mismatches = cases
        .iter()
        .map(|case| u64::from(case.frame_parallel != case.one_thread))
        .sum();
    let ordered_mismatches = cases
        .iter()
        .map(|case| u64::from(case.ordered_pipeline != case.one_thread))
        .sum();
    let mismatch_counts = [
        direct_mismatches,
        frame_parallel_mismatches,
        ordered_mismatches,
    ];

    let trace_tsv = render_trace(
        definition,
        reference_digest,
        &mismatch_counts,
        &frame_parallel,
        &ordered,
        &cases,
    );
    let trace_evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    let (key, mut evidence) = crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    evidence.push(trace_evidence);
    let batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: mismatch_counts.into_iter().map(Sample::valid).collect(),
        evidence,
    };
    let _ = batch.to_tsv()?;

    Ok(Pg5Artifacts {
        batch,
        trace_tsv,
        reference_digest,
        cases,
    })
}

/// This build's certified arithmetic at the crate-wide compiled tier.
#[must_use]
pub const fn pg5_identity() -> EngineIdentity {
    EngineIdentity {
        tier: Tier::COMPILED,
        ..EngineIdentity::certified()
    }
}

#[derive(Debug)]
struct PreparedScene {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
}

fn prepare_scene(index: usize) -> Result<PreparedScene, String> {
    let case = SCENES
        .get(index)
        .ok_or_else(|| format!("scene index {index} is outside the committed corpus"))?;
    let built = (case.build)(scene_goldens::corpus());
    let config = scene_goldens::frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&built.stage, 0)
        .map_err(|error| format!("{} render plan: {error}", case.name))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| format!("{} monotone table: {error}", case.name))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .map_err(|error| format!("{} binning: {error}", case.name))?;
    binning
        .prune_occluded(&plan)
        .map_err(|error| format!("{} binning: {error}", case.name))?;
    Ok(PreparedScene {
        plan,
        mono,
        binning,
        config,
    })
}

/// The certified single-threaded frame of a stage, encoded into its
/// canonical document — byte-for-byte the fast-tier scene_goldens closure,
/// so the producer's lock anchor is the committed quantity itself.
fn certified_document(stage: &Stage) -> Vec<u8> {
    let config = scene_goldens::frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0).expect("scene-golden anchor plan syncs");
    let mono = MonoTable::build(&plan, config.map).expect("scene-golden anchor monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("scene-golden anchor binning");
    binning
        .prune_occluded(&plan)
        .expect("scene-golden anchor pruning");
    let frame =
        FrameJob::with_identity(&plan, &mono, &binning, config, EngineIdentity::certified())
            .expect("scene-golden anchor frame artifacts")
            .render(1)
            .expect("scene-golden anchor renders");
    encode_frame(&frame).expect("scene-golden anchor encodes")
}
fn render_prepared(prepared: &PreparedScene, threads: usize) -> Result<Digest, Pg5Error> {
    let job = FrameJob::with_identity(
        &prepared.plan,
        &prepared.mono,
        &prepared.binning,
        prepared.config,
        pg5_identity(),
    )
    .map_err(|error| Pg5Error::Render(error.to_string()))?;
    let frame = job
        .render(threads)
        .map_err(|error| Pg5Error::Render(error.to_string()))?;
    frame_digest(&frame).map_err(|error| Pg5Error::Render(error.to_string()))
}

fn execution_plan_tsv(plan: &ExecutionPlan) -> String {
    let mut output = String::new();
    {
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &"fmn-perf-pg5-execution-plan/1");
        row("determinism", &determinism_name(plan.determinism));
        row("intent", &intent_name(plan.intent));
        row("engine", &execution_engine_name(plan.engine));
        row("frames_in_flight", &plan.frames_in_flight);
        row("render_team_count", &plan.render_teams.len());
        row("fine_tile", &plan.fine_tile);
        row("macro_tile", &plan.macro_tile);
        row("simd_tier", &plan.simd_tier.name());
        row("output_format", &plan.output_format.name());
        row("estimated_in_flight_bytes", &plan.estimated_in_flight_bytes);
        row("topology_fingerprint", &plan.topology_fingerprint.value());
        row("tuning_source", &tuning_source_name(plan.tuning_source));
    }
    write_team_contract(&mut output, "scene", &plan.scene_team);
    for (index, team) in plan.render_teams.iter().enumerate() {
        write_team_contract(&mut output, &format!("render-{index}"), team);
    }
    write_team_contract(&mut output, "output", &plan.output_team);
    output
}

fn write_team_contract(output: &mut String, label: &str, team: &TeamPlan) {
    let mut row = |name: &str, value: &dyn fmt::Display| {
        let _ = writeln!(output, "team.{label}.{name}\t{value}");
    };
    row("role", &team_role_name(team.role));
    row("cpu_count", &team.cpu_ids.len());
    row("cpu_ids", &format_u32s(&team.cpu_ids));
    row("scratch_bytes_per_worker", &team.scratch_bytes_per_worker);
    row("shares_cores", &team.shares_cores);
    row("locality_lane_count", &team.locality_lanes.len());
    for (index, lane) in team.locality_lanes.iter().enumerate() {
        write_lane_contract(output, label, index, lane);
    }
}

fn write_lane_contract(output: &mut String, label: &str, index: usize, lane: &LocalityLane) {
    let prefix = format!("team.{label}.lane.{index}");
    let _ = writeln!(output, "{prefix}.processor_group\t{}", lane.processor_group);
    let _ = writeln!(
        output,
        "{prefix}.numa_node\t{}",
        format_option_u32(lane.numa_node)
    );
    let _ = writeln!(
        output,
        "{prefix}.l3_domain\t{}",
        format_option_usize(lane.l3_domain)
    );
    let _ = writeln!(output, "{prefix}.cpu_count\t{}", lane.cpu_ids.len());
    let _ = writeln!(output, "{prefix}.cpu_ids\t{}", format_u32s(&lane.cpu_ids));
}

fn format_u32s(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn format_option_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn format_option_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

const fn determinism_name(value: Determinism) -> &'static str {
    match value {
        Determinism::Standard => "standard",
        Determinism::Certified => "certified",
    }
}

const fn intent_name(value: RenderIntent) -> &'static str {
    match value {
        RenderIntent::Preview => "preview",
        RenderIntent::Offline => "offline",
    }
}

const fn execution_engine_name(value: ExecutionEngine) -> &'static str {
    match value {
        ExecutionEngine::CertifiedCpu => "certified-cpu",
        ExecutionEngine::FastCpu => "fast-cpu",
        ExecutionEngine::Metal => "metal",
        ExecutionEngine::Cuda => "cuda",
    }
}

fn team_role_name(value: TeamRole) -> String {
    match value {
        TeamRole::Scene => "scene".to_owned(),
        TeamRole::Render(index) => format!("render-{index}"),
        TeamRole::Output => "output".to_owned(),
    }
}

const fn tuning_source_name(value: TuningSource) -> &'static str {
    match value {
        TuningSource::CertifiedProfile => "certified-profile",
        TuningSource::StandardBaseline => "standard-baseline",
        TuningSource::StandardAutotuneCache => "standard-autotune-cache",
    }
}

fn pipeline_plan() -> Result<ExecutionPlan, Pg5Error> {
    let topology = pipeline_topology();
    ExecutionPlan::derive(
        PlanRequest::certified(
            RenderIntent::Offline,
            SurfaceSpec::lumen(scene_goldens::WIDTH, scene_goldens::HEIGHT),
            OutputPixelFormat::Rgba16F,
        )
        .with_max_frames_in_flight(PG5_PIPELINE_DEPTH)
        .with_max_cpu_threads(PG5_PIPELINE_CORES as usize),
        &topology,
        None,
    )
    .map_err(|error| Pg5Error::Schedule(error.to_string()))
}

fn pipeline_topology() -> HardwareTopology {
    let mut topology = HardwareTopology::fallback(PG5_PIPELINE_CORES);
    // `fallback` ordinarily reports the live host's detected capability. This
    // topology is definition material, not host attestation, so pin that one
    // otherwise-host-dependent field to the certified scalar definition.
    topology.simd_tier = SimdTier::Portable;
    topology
}

fn validate_pipeline_plan(plan: &ExecutionPlan) -> Result<(), Pg5Error> {
    let topology = pipeline_topology();
    let exact = plan.determinism == Determinism::Certified
        && plan.intent == RenderIntent::Offline
        && plan.engine == ExecutionEngine::CertifiedCpu
        && plan.output_format == OutputPixelFormat::Rgba16F
        && plan.frames_in_flight == PG5_PIPELINE_DEPTH
        && plan.scene_team.role == TeamRole::Scene
        && plan.scene_team.threads() == 1
        && plan.scene_team.scratch_bytes_per_worker != 0
        && !plan.scene_team.locality_lanes.is_empty()
        && plan.render_teams.len() == PG5_PIPELINE_TEAMS
        && plan.render_teams.iter().enumerate().all(|(index, team)| {
            team.role == TeamRole::Render(index)
                && team.threads() == PG5_THREADS_PER_TEAM
                && team.scratch_bytes_per_worker != 0
                && !team.locality_lanes.is_empty()
        })
        && plan.output_team.role == TeamRole::Output
        && plan.output_team.threads() != 0
        && plan.output_team.scratch_bytes_per_worker != 0
        && !plan.output_team.locality_lanes.is_empty()
        && plan.fine_tile == TILING.fine_tile
        && plan.macro_tile == TILING.macro_tile
        && plan.simd_tier == topology.simd_tier
        && plan.topology_fingerprint == TopologyFingerprint::of(&topology)
        && plan.tuning_source == TuningSource::CertifiedProfile
        && plan.estimated_in_flight_bytes != 0;
    if exact {
        Ok(())
    } else {
        Err(Pg5Error::Schedule(format!(
            "derived certified plan drifted: determinism={:?}, engine={:?}, output={:?}, depth={}, teams={:?}, tiles={}/{}",
            plan.determinism,
            plan.engine,
            plan.output_format,
            plan.frames_in_flight,
            plan.render_teams
                .iter()
                .map(TeamPlan::threads)
                .collect::<Vec<_>>(),
            plan.fine_tile,
            plan.macro_tile,
        )))
    }
}

#[derive(Debug)]
struct DigestStages;

impl PipelineStages for DigestStages {
    type Frame = usize;
    type Prepared = PreparedScene;
    type Rasterized = FrameBuffer;
    type Output = Digest;
    type Error = String;

    fn prepare(
        &self,
        frame: Self::Frame,
        _scene_team: &TeamPlan,
    ) -> Result<Self::Prepared, Self::Error> {
        prepare_scene(frame)
    }

    fn rasterize(
        &self,
        prepared: Self::Prepared,
        render_team: &TeamPlan,
    ) -> Result<Self::Rasterized, Self::Error> {
        let job = FrameJob::with_identity(
            &prepared.plan,
            &prepared.mono,
            &prepared.binning,
            prepared.config,
            pg5_identity(),
        )
        .map_err(|error| error.to_string())?;
        job.render(render_team.threads())
            .map_err(|error| error.to_string())
    }

    fn convert(
        &self,
        rasterized: Self::Rasterized,
        _output_team: &TeamPlan,
    ) -> Result<Self::Output, Self::Error> {
        frame_digest(&rasterized).map_err(|error| error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScheduleReceipt {
    digests: Vec<Digest>,
    max_in_flight: usize,
    render_team_frames: Vec<u64>,
    emitter_max_outstanding: Option<usize>,
}

fn run_frame_parallel(plan: &ExecutionPlan) -> Result<ScheduleReceipt, Pg5Error> {
    let stages = DigestStages;
    let events = (0..SCENES.len()).map(|index| {
        PipelineEvent::<_, ()>::frame(u64::try_from(index).unwrap_or(u64::MAX), index)
    });
    let mut digests = Vec::with_capacity(SCENES.len());
    // ubs:ignore — public pipeline counters; the closure's digest is not a secret.
    let stats = FramePipeline::new(plan, &stages)
        .run(
            events,
            |sequence, digest| {
                // ubs:ignore — public frame count derived from a digest vector length.
                let expected = u64::try_from(digests.len()).unwrap_or(u64::MAX);
                if sequence != expected {
                    return Err(format!(
                        "frame-parallel output moved out of order: expected {expected}, got {sequence}"
                    ));
                }
                digests.push(digest);
                Ok(())
            },
            |(), _| Ok(()),
        )
        .map_err(|error| Pg5Error::Schedule(error.to_string()))?;
    validate_pipeline_receipt("frame-parallel", &stats, SCENES.len())?;
    Ok(ScheduleReceipt {
        digests,
        max_in_flight: stats.max_in_flight,
        render_team_frames: stats.render_team_frames,
        emitter_max_outstanding: None,
    })
}

#[derive(Debug)]
struct EmitterInput {
    sequence: u64,
    scene_index: usize,
}

#[derive(Debug)]
struct ReservedScene {
    prepared: PreparedScene,
    reservation: FrameReservation,
}

#[derive(Debug)]
struct EmitterStages {
    handle: EmitterHandle,
}

impl PipelineStages for EmitterStages {
    type Frame = EmitterInput;
    type Prepared = ReservedScene;
    type Rasterized = FrameReservation;
    type Output = FrameReservation;
    type Error = String;

    fn prepare(
        &self,
        frame: Self::Frame,
        _scene_team: &TeamPlan,
    ) -> Result<Self::Prepared, Self::Error> {
        let prepared = prepare_scene(frame.scene_index)?;
        let reservation = self
            .handle
            .reserve(frame.sequence)
            .map_err(|error| error.to_string())?;
        Ok(ReservedScene {
            prepared,
            reservation,
        })
    }

    fn rasterize(
        &self,
        prepared: Self::Prepared,
        render_team: &TeamPlan,
    ) -> Result<Self::Rasterized, Self::Error> {
        let ReservedScene {
            prepared,
            mut reservation,
        } = prepared;
        let job = FrameJob::with_identity(
            &prepared.plan,
            &prepared.mono,
            &prepared.binning,
            prepared.config,
            pg5_identity(),
        )
        .map_err(|error| error.to_string())?;
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

#[derive(Debug)]
struct DigestSink {
    frames: Arc<Mutex<Vec<(u64, Digest)>>>,
}

impl FrameSink for DigestSink {
    fn write_frame(
        &mut self,
        sequence: u64,
        frame: &FrameBuffer,
    ) -> Result<SinkWrite, SinkFailure> {
        let digest = frame_digest(frame).map_err(|error| SinkFailure::new(error.to_string()))?;
        lock(&self.frames).push((sequence, digest));
        Ok(SinkWrite::Consumed)
    }
}

fn run_ordered_pipeline(plan: &ExecutionPlan) -> Result<ScheduleReceipt, Pg5Error> {
    let frames = Arc::new(Mutex::new(Vec::with_capacity(SCENES.len())));
    let emitter = OrderedEmitter::new(
        EmitterConfig::new(
            scene_goldens::frame_config()
                .layout()
                .map_err(|error| Pg5Error::Schedule(error.to_string()))?,
            PG5_PIPELINE_DEPTH,
            0,
        )
        .map_err(|error| Pg5Error::Schedule(error.to_string()))?,
        vec![SinkBinding::reliable(
            "pg5-digest",
            DigestSink {
                frames: Arc::clone(&frames),
            },
        )],
    )
    .map_err(|error| Pg5Error::Schedule(error.to_string()))?;
    let stages = EmitterStages {
        handle: emitter.handle(),
    };
    let events = (0..SCENES.len()).map(|scene_index| {
        let sequence = u64::try_from(scene_index).unwrap_or(u64::MAX);
        PipelineEvent::<_, ()>::frame(
            sequence,
            EmitterInput {
                sequence,
                scene_index,
            },
        )
    });
    // ubs:ignore — public pipeline counters; the closure's digest is not a secret.
    let stats = FramePipeline::new(plan, &stages)
        .run(
            events,
            |sequence, reservation| {
                if reservation.sequence() != sequence {
                    return Err(format!(
                        "ordered reservation moved: expected {sequence}, got {}",
                        reservation.sequence()
                    ));
                }
                reservation.publish().map_err(|error| error.to_string())
            },
            |(), _| Ok(()),
        )
        .map_err(|error| Pg5Error::Schedule(error.to_string()))?;
    validate_pipeline_receipt("ordered-pipeline", &stats, SCENES.len())?;
    drop(stages);
    let report = emitter
        .finish()
        .map_err(|error| Pg5Error::Schedule(error.to_string()))?;
    validate_emitter_report(&report, SCENES.len())?;

    let recorded = lock(&frames).clone();
    if recorded.len() != SCENES.len() {
        return Err(Pg5Error::Schedule(format!(
            "ordered sink retained {} frames, expected {}",
            recorded.len(),
            SCENES.len()
        )));
    }
    let mut digests = Vec::with_capacity(recorded.len());
    for (index, (sequence, digest)) in recorded.into_iter().enumerate() {
        let expected = u64::try_from(index).unwrap_or(u64::MAX);
        if sequence != expected {
            return Err(Pg5Error::Schedule(format!(
                "ordered sink sequence moved: expected {expected}, got {sequence}"
            )));
        }
        digests.push(digest);
    }
    Ok(ScheduleReceipt {
        digests,
        max_in_flight: stats.max_in_flight,
        render_team_frames: stats.render_team_frames,
        emitter_max_outstanding: Some(report.stats.max_outstanding),
    })
}

fn validate_pipeline_receipt(
    name: &str,
    stats: &PipelineStats,
    expected_frames: usize,
) -> Result<(), Pg5Error> {
    let expected = u64::try_from(expected_frames).unwrap_or(u64::MAX);
    if stats.submitted != expected
        || stats.emitted != expected
        || stats.outstanding_slots != 0
        || stats.max_in_flight == 0
        || stats.max_in_flight > PG5_PIPELINE_DEPTH
        || stats.render_team_frames.len() != PG5_PIPELINE_TEAMS
        || stats.render_team_frames.contains(&0)
        || stats.render_team_frames.iter().sum::<u64>() != expected
    {
        return Err(Pg5Error::Schedule(format!(
            "{name} receipt drifted: submitted={}, emitted={}, outstanding={}, max_in_flight={}, team_frames={:?}",
            stats.submitted,
            stats.emitted,
            stats.outstanding_slots,
            stats.max_in_flight,
            stats.render_team_frames
        )));
    }
    Ok(())
}

fn validate_emitter_report(report: &EmitterReport, expected_frames: usize) -> Result<(), Pg5Error> {
    let expected = u64::try_from(expected_frames).unwrap_or(u64::MAX);
    if report.stats.capacity != PG5_PIPELINE_DEPTH
        || report.stats.reserved != expected
        || report.stats.published != expected
        || report.stats.emitted != expected
        || report.stats.outstanding != 0
        || report.stats.max_outstanding == 0
        || report.stats.max_outstanding > PG5_PIPELINE_DEPTH
        || report.stats.failed
        || report.sinks.len() != 1
        || report.sinks[0].accepted != expected
        || report.sinks[0].dropped != 0
    {
        return Err(Pg5Error::Schedule(format!(
            "ordered emitter receipt drifted: stats={:?}, sinks={:?}",
            report.stats, report.sinks
        )));
    }
    Ok(())
}

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(PoisonError::into_inner)
}

fn aggregate_reference_digest(
    corpus_lock_digest: Digest,
    cases: &[Pg5CaseResult],
) -> Result<Digest, Pg5Error> {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg5-reference-v1");
    hash.update(corpus_lock_digest.as_bytes());
    for case in cases {
        hash_field(&mut hash, case.scene.as_bytes())?;
        hash.update(case.one_thread.as_bytes());
    }
    Ok(hash.finalize())
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) -> Result<(), Pg5Error> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| Pg5Error::Fixture("corpus field exceeds u64".to_owned()))?;
    hash.update(&length.to_be_bytes());
    hash.update(bytes);
    Ok(())
}

fn render_trace(
    definition: Pg5Definition,
    reference_digest: Digest,
    mismatch_counts: &[u64; PG5_SAMPLE_COUNT],
    frame_parallel: &ScheduleReceipt,
    ordered: &ScheduleReceipt,
    cases: &[Pg5CaseResult],
) -> String {
    let mut output = String::new();
    {
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG5_TRACE_SCHEMA);
        row("gate", &GateId::Pg5);
        row("scenario", &PG5_SCENARIO);
        row("benchmark_definition", &definition.digest());
        row("config_digest", &definition.config_digest());
        row("execution_plan_digest", &definition.execution_plan_digest());
        row("scene_golden_lock_digest", &definition.corpus_lock_digest());
        row("engine", &pg5_identity().engine.name());
        row("tier", &pg5_identity().tier.name());
        row("direct_threads", &"1,4,16");
        row("corpus_scenes", &cases.len());
        row("reference_digest", &reference_digest);
        row("direct_mismatches", &mismatch_counts[0]);
        row("frame_parallel_mismatches", &mismatch_counts[1]);
        row("ordered_pipeline_mismatches", &mismatch_counts[2]);
        row(
            "frame_parallel_max_in_flight",
            &frame_parallel.max_in_flight,
        );
        row(
            "frame_parallel_team_frames",
            &format_team_frames(&frame_parallel.render_team_frames),
        );
        row("ordered_max_in_flight", &ordered.max_in_flight);
        row(
            "ordered_team_frames",
            &format_team_frames(&ordered.render_team_frames),
        );
        row(
            "ordered_emitter_max_outstanding",
            &ordered.emitter_max_outstanding.unwrap_or_default(),
        );
    }
    for (index, case) in cases.iter().enumerate() {
        let _ = writeln!(
            output,
            "scene\t{index}\t{}\t{}\t{}\t{}\t{}\t{}",
            case.scene,
            case.one_thread,
            case.four_threads,
            case.sixteen_threads,
            case.frame_parallel,
            case.ordered_pipeline,
        );
    }
    output
}

fn format_team_frames(frames: &[u64]) -> String {
    frames
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// PG-5 producer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pg5Error {
    /// Common performance schema/evidence failure.
    Perf(PerfError),
    /// Baseline and compiled-producer identity differ.
    Identity(String),
    /// Canonical corpus or derived artifact construction failed.
    Fixture(String),
    /// Certified renderer or reference self-golden failure.
    Render(String),
    /// Runtime pipeline, execution-plan, or ordered-emitter failure.
    Schedule(String),
}

impl fmt::Display for Pg5Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perf(error) => error.fmt(formatter),
            Self::Identity(detail) => write!(formatter, "PG-5 identity: {detail}"),
            Self::Fixture(detail) => write!(formatter, "PG-5 fixture: {detail}"),
            Self::Render(detail) => write!(formatter, "PG-5 render: {detail}"),
            Self::Schedule(detail) => write!(formatter, "PG-5 schedule: {detail}"),
        }
    }
}

impl std::error::Error for Pg5Error {}

impl From<PerfError> for Pg5Error {
    fn from(error: PerfError) -> Self {
        Self::Perf(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_binds_the_complete_corpus_and_schedule_axes() {
        let definition = Pg5Definition::new().expect("fixed PG-5 definition");
        let text = definition.to_tsv();
        assert!(text.contains("scenario\tcertified-thread-matrix\n"));
        assert!(text.contains("direct_threads\t1,4,16\n"));
        assert!(text.contains("sample_1\tframe-parallel\n"));
        assert!(text.contains("sample_2\tordered-pipeline\n"));
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("scene\t"))
                .count(),
            PG5_CORPUS_SCENES
        );
        // The definition digest pins the complete axis set for the portable
        // tier; the reference aggregate is derived from the committed lock
        // (checked per scene by the producer), never pinned here.
        if Tier::COMPILED.name() == "portable" {
            // Moved by adjudicated inputs only: the scene-golden lock (the
            // 67b1c438 stroke/fill-profile re-pin, fm-sq8.9's exact vertical
            // mirror, 685c4f68's near-straight quadratic strokes, the three
            // Tex scenes' rule-17 italic corrections at franken_markdown
            // 4c8caa2, c1ee7836's Reference extents for Brace, marks, Title
            // and BulletedList, then title_underlined.v2 framed into view,
            // then tex_overbrace.v1's \cdots dots and "n terms" space at
            // franken_markdown a3578a0, then tex_fraction.v1 and
            // tex_display_sum.v1's null delimiters at franken_markdown
            // e4e8e2e, then matrix_2x2.v1 and decimal_matrix.v1's array
            // struts at franken_markdown bd00a24, then tex_overbrace.v1's
            // LaTeX brace fill and tex_fraction.v1 and tex_display_sum.v1's
            // emission order at franken_markdown 3ed1d33, then their rule
            // contours from the bottom-right corner at franken_markdown
            // 126a530)
            // and the journaled map orientation.
            assert_eq!(
                definition.digest().to_string(),
                "4b5ed59a580db24cbaf3f5d0a64584edc9f89c7119d953ebf01ea2f499929ff5"
            );
        }
    }

    #[test]
    fn embedded_lock_and_compiled_corpus_are_exactly_aligned() {
        Pg5Definition::new()
            .expect("fixed PG-5 definition")
            .validate_corpus_lock()
            .expect("committed lock and compiled corpus agree");
    }

    #[test]
    fn fixed_pipeline_plan_has_two_certified_render_teams() {
        let plan = pipeline_plan().expect("fixed plan derives");
        validate_pipeline_plan(&plan).expect("fixed plan identity");
        assert_eq!(plan.simd_tier, SimdTier::Portable);
        assert_eq!(plan.frames_in_flight, PG5_PIPELINE_DEPTH);
        assert_eq!(
            plan.render_teams
                .iter()
                .map(TeamPlan::threads)
                .collect::<Vec<_>>(),
            vec![PG5_THREADS_PER_TEAM; PG5_PIPELINE_TEAMS]
        );
    }

    #[test]
    fn execution_plan_digest_binds_complete_team_placement() {
        let plan = pipeline_plan().expect("fixed plan derives");
        // ubs:ignore — public execution-plan self-golden, not authentication material.
        let expected = Pg5Definition::from_execution_plan(&plan).execution_plan_digest();

        let mut changed = plan.clone();
        changed.scene_team.scratch_bytes_per_worker += 1;
        assert_ne!(
            Pg5Definition::from_execution_plan(&changed).execution_plan_digest(),
            expected
        );

        let mut changed = plan.clone();
        changed.output_team.shares_cores = !changed.output_team.shares_cores;
        assert_ne!(
            Pg5Definition::from_execution_plan(&changed).execution_plan_digest(),
            expected
        );

        let mut changed = plan;
        changed
            .render_teams
            .first_mut()
            .expect("fixed plan has a render team")
            .locality_lanes
            .first_mut()
            .expect("fixed render team has a locality lane")
            .processor_group += 1;
        assert_ne!(
            Pg5Definition::from_execution_plan(&changed).execution_plan_digest(),
            expected
        );
    }
}
