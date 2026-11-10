//! Canonical PG-6 primitive steady-state allocation producer.
//!
//! The committed scene-golden corpus is the workload. For every scene, this
//! producer renders one warm frame through a fresh [`FrameArena`], then renders
//! the same scene into the same output buffer through the reused arena. The
//! second frame's engine-owned allocation ledger is one exact policy sample;
//! both frame digests are retained and must match before evidence is emitted.
//!
//! Two of PG-6's three surfaces live here: `primitive-steady-allocations`
//! (above) and the `one-hour-soak-leak` residency soak (below). Peak RSS
//! (`gallery-4k-3d-peak`) requires the 4K 3D gallery corpus and remains a
//! separate, explicit evidence gap.

use crate::perf::{
    Baseline, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GateScope,
    MeasurementBatch, MetricUnit, PerfError, Sample, require_compiled_cargo_profile,
    validate_producer_commit,
};
use crate::scene_goldens::{self, SCENES, TILING};
use fmn_hash::{Digest, Sha256, sha256};
use fmn_mobject::Stage;
use fmn_render::{
    AllocStats, Binning, EngineIdentity, FrameArena, FrameJob, MonoTable, RenderPlan, Tier,
    engine::encode_frame, frame_digest,
};
use std::fmt;
use std::fmt::Write as _;

/// Stable workload-definition schema.
pub const PG6_DEFINITION_SCHEMA: &str = "fmn-perf-pg6-definition/2";
/// Stable phase-trace schema.
pub const PG6_TRACE_SCHEMA: &str = "fmn-perf-pg6-trace/1";
/// Policy-catalog scenario implemented by this producer.
pub const PG6_SCENARIO: &str = "primitive-steady-allocations";
/// Policy-catalog minimum valid scene samples.
pub const PG6_MIN_VALID_SAMPLES: usize = 21;
/// An exact invariant admits no invalid scene samples.
pub const PG6_MAX_INVALID_SAMPLES: usize = 0;
/// Every committed scene-golden case contributes one sample.
pub const PG6_SAMPLE_COUNT: usize = 27;
/// The permanent per-commit worker-team size for this producer.
pub const PG6_THREADS: usize = 4;
/// One excluded warm frame sizes the arena and worker pool for each scene.
pub const PG6_WARMUP_FRAMES_PER_SCENE: usize = 1;

const BUILD_PROFILE: &str = "release-perf";
const THREAD_PROFILE: &str = "fixed-4";
const CACHE_STATE: &str = "warm-reused-frame-arena";
const OUTPUT_MODE: &str = "scene-golden-anchored";

/// The complete content-addressed definition of the allocation workload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pg6Definition;

impl Pg6Definition {
    /// Construct the sole canonical PG-6 allocation definition.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(self) -> String {
        let mut output = String::new();
        {
            let mut row = |name: &str, value: &dyn fmt::Display| {
                let _ = writeln!(output, "{name}\t{value}");
            };
            row("schema", &PG6_DEFINITION_SCHEMA);
            row("gate", &GateId::Pg6);
            row("scenario", &PG6_SCENARIO);
            row("unit", &MetricUnit::Allocations.name());
            row("target", &0);
            row("threads", &PG6_THREADS);
            row("warmup_frames_per_scene", &PG6_WARMUP_FRAMES_PER_SCENE);
            row("sample_count", &PG6_SAMPLE_COUNT);
            row("sample_scope", &"one-post-warmup-frame-per-scene");
            row("lifecycle_point", &"post-construct");
            row("frame_index", &0);
            row("engine", &pg6_identity().engine.name());
            row("tier", &pg6_identity().tier.name());
            row("thread_profile", &THREAD_PROFILE);
            row("cache_state", &CACHE_STATE);
            row("output_mode", &OUTPUT_MODE);
            row("frame_width_px", &scene_goldens::WIDTH);
            row("frame_height_px", &scene_goldens::HEIGHT);
            row("scene_golden_lock_digest", &self.corpus_lock_digest());
            row("config_digest", &self.config_digest());
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

    /// Exact C7/C10 renderer/configuration digest for this artifact.
    #[must_use]
    pub fn config_digest(self) -> Digest {
        fmn_render::engine::journal_digest(pg6_identity(), &scene_goldens::frame_config(), TILING)
    }

    /// Digest of the committed certified corpus lock that fixes scene input.
    #[must_use]
    pub fn corpus_lock_digest(self) -> Digest {
        scene_goldens::certified_lock_digest()
    }
    /// Recompute the result aggregate from produced cases — the trace
    /// identity the common verifier replays. The committed scene-golden
    /// lock is the anchor; this aggregate is derived, never pinned.
    ///
    /// # Errors
    /// Propagates fixture errors from the corpus field hashing.
    pub fn result_digest(&self, cases: &[Pg6CaseResult]) -> Result<Digest, Pg6Error> {
        aggregate_result_digest(self.corpus_lock_digest(), cases)
    }

    /// Validate that the embedded lock and compiled corpus name exactly the
    /// same bounded scene set.
    ///
    /// # Errors
    /// Returns a typed fixture error for a malformed, missing, duplicate, or
    /// stale lock row.
    pub fn validate_corpus_lock(self) -> Result<(), Pg6Error> {
        if SCENES.len() != PG6_SAMPLE_COUNT {
            return Err(Pg6Error::Fixture(format!(
                "compiled corpus has {} scenes, expected {PG6_SAMPLE_COUNT}",
                SCENES.len()
            )));
        }
        scene_goldens::validate_certified_lock().map_err(Pg6Error::Fixture)
    }

    /// Validate that a baseline names precisely this producer.
    ///
    /// # Errors
    /// Returns a typed identity error before any corpus scene is built.
    pub fn validate_baseline(self, baseline: &Baseline) -> Result<(), Pg6Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg6 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != PG6_SCENARIO {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::Allocations {
            mismatches.push("unit");
        }
        if baseline.policy.direction != Direction::Exactly {
            mismatches.push("direction");
        }
        if baseline.policy.target != Some(0) {
            mismatches.push("target");
        }
        if baseline.policy.min_valid_samples != PG6_MIN_VALID_SAMPLES {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != PG6_MAX_INVALID_SAMPLES {
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
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != pg6_identity().engine.name() {
            mismatches.push("engine");
        }
        if key.tier != pg6_identity().tier.name() {
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
            Err(Pg6Error::Identity(format!(
                "{PG6_SCENARIO} baseline differs from the compiled producer in: {}",
                mismatches.join(", ")
            )))
        }
    }
}

/// One corpus scene's retained warm/reuse proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6CaseResult {
    /// Stable scene-golden case name.
    pub scene: &'static str,
    /// Frame identity produced while sizing the arena.
    pub warm_frame_digest: Digest,
    /// Frame identity produced through the reused arena and output buffer.
    pub measured_frame_digest: Digest,
    /// Complete engine-owned ledger for the excluded warm frame.
    pub warm: AllocStats,
    /// Complete engine-owned ledger for the measured frame.
    pub measured: AllocStats,
}

/// Measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6Artifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Aggregate identity of every warm/measured frame pair.
    pub result_digest: Digest,
    /// Per-scene allocation and frame-identity proof.
    pub cases: Vec<Pg6CaseResult>,
}

/// Measure every committed corpus scene through one warm and one reused frame.
///
/// The certified single-threaded frame of a stage, encoded into its
/// canonical document — byte-for-byte the fast-tier scene_goldens closure,
/// so the allocation probe's lock anchor is the committed quantity itself.
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

/// Run the allocation workload...
/// `trace_path` is recorded and content-addressed, but this function performs
/// no filesystem I/O. The CLI publishes the returned trace before the raw
/// bundle using exclusive-create semantics.
///
/// # Errors
/// Returns before scene construction for identity, path, profile, or embedded
/// lock errors. Renderer errors and frame drift are explicit. Nonzero measured
/// allocations remain valid samples so the common exact-policy verifier can
/// issue the blocking verdict.
pub fn measure_pg6(
    baseline: &Baseline,
    producer_commit: &str,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg6Artifacts, Pg6Error> {
    let definition = Pg6Definition::new();
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_compiled_cargo_profile(BUILD_PROFILE)?;
    definition.validate_corpus_lock()?;

    // Reference anchor: the warm and measured frames of every corpus scene
    // must re-derive the committed cross-platform scene-golden artifacts,
    // so allocation-path equivalence means equivalence to the committed
    // bits, not merely to themselves.
    let golden_corpus = scene_goldens::corpus();
    let golden_store = scene_goldens::store();
    for (index, case) in SCENES.iter().enumerate() {
        let bytes = scene_goldens::artifact(case, golden_corpus, index, &certified_document);
        if let Err(error) = golden_store.check(case.name, &bytes) {
            return Err(Pg6Error::Render(format!(
                "reference corpus self-golden drift at {}: {error}",
                case.name
            )));
        }
    }

    let (key, host_evidence) =
        crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    let mut batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: Vec::with_capacity(PG6_SAMPLE_COUNT),
        evidence: host_evidence,
    };
    let _ = batch.to_tsv()?;

    let config = scene_goldens::frame_config();
    let corpus = scene_goldens::corpus();
    let mut cases = Vec::with_capacity(PG6_SAMPLE_COUNT);
    for case in SCENES {
        let built = (case.build)(corpus);
        let mut plan = RenderPlan::new();
        plan.sync(&built.stage, 0)
            .map_err(|error| Pg6Error::Fixture(format!("{} render plan: {error}", case.name)))?;
        let mono = MonoTable::build(&plan, config.map)
            .map_err(|error| Pg6Error::Fixture(format!("{} monotone table: {error}", case.name)))?;
        let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
            .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
        binning
            .prune_occluded(&plan)
            .map_err(|error| Pg6Error::Fixture(error.to_string()))?;

        let mut arena = FrameArena::new();
        let (mut frame, warm_frame_digest, warm) = {
            let job = FrameJob::with_identity_in(
                &mut arena,
                &plan,
                &mono,
                &binning,
                config,
                pg6_identity(),
            )
            .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
            let frame = job
                .render(PG6_THREADS)
                .map_err(|error| Pg6Error::Render(error.to_string()))?;
            let stats = job.allocation_stats();
            let digest =
                frame_digest(&frame).map_err(|error| Pg6Error::Render(error.to_string()))?;
            (frame, digest, stats)
        };

        let (measured_frame_digest, measured) = {
            let job = FrameJob::with_identity_in(
                &mut arena,
                &plan,
                &mono,
                &binning,
                config,
                pg6_identity(),
            )
            .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
            job.render_into(PG6_THREADS, &mut frame)
                .map_err(|error| Pg6Error::Render(error.to_string()))?;
            let stats = job.allocation_stats();
            let digest =
                frame_digest(&frame).map_err(|error| Pg6Error::Render(error.to_string()))?;
            (digest, stats)
        };
        // ubs:ignore — public deterministic frame identity, not authentication material.
        if measured_frame_digest != warm_frame_digest {
            return Err(Pg6Error::Render(format!(
                "{} changed across arena reuse: warm {}, measured {}",
                case.name, warm_frame_digest, measured_frame_digest
            )));
        }
        if warm.heap_allocs_this_frame == 0 {
            return Err(Pg6Error::Render(format!(
                "{} warm frame reported no arena/worker sizing allocations",
                case.name
            )));
        }
        if measured.arena_buffer_bytes != warm.arena_buffer_bytes
            || measured.pool_slots != warm.pool_slots
            || measured.pool_slots != PG6_THREADS
        {
            return Err(Pg6Error::Render(format!(
                "{} storage changed across arena reuse: warm bytes/slots {}/{}, measured {}/{}; expected {PG6_THREADS} worker slots",
                case.name,
                warm.arena_buffer_bytes,
                warm.pool_slots,
                measured.arena_buffer_bytes,
                measured.pool_slots,
            )));
        }
        batch
            .samples
            .push(Sample::valid(measured.heap_allocs_this_frame));
        cases.push(Pg6CaseResult {
            scene: case.name,
            warm_frame_digest,
            measured_frame_digest,
            warm,
            measured,
        });
    }
    // The per-scene anchor above is the committed-provenance check; the
    // aggregate below is derived trace identity, never a pinned constant.
    let result_digest = aggregate_result_digest(definition.corpus_lock_digest(), &cases)?;
    let trace_tsv = render_trace(definition, result_digest, &cases);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;

    Ok(Pg6Artifacts {
        batch,
        trace_tsv,
        result_digest,
        cases,
    })
}

/// This build's certified arithmetic at the crate-wide compiled tier.
#[must_use]
pub const fn pg6_identity() -> EngineIdentity {
    EngineIdentity {
        tier: Tier::COMPILED,
        ..EngineIdentity::certified()
    }
}

fn aggregate_result_digest(
    corpus_lock_digest: Digest,
    cases: &[Pg6CaseResult],
) -> Result<Digest, Pg6Error> {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg6-result-v1");
    hash.update(corpus_lock_digest.as_bytes());
    for case in cases {
        hash_field(&mut hash, case.scene.as_bytes())?;
        hash.update(case.warm_frame_digest.as_bytes());
        hash.update(case.measured_frame_digest.as_bytes());
    }
    Ok(hash.finalize())
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) -> Result<(), Pg6Error> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| Pg6Error::Fixture("corpus field exceeds u64".to_owned()))?;
    hash.update(&length.to_be_bytes());
    hash.update(bytes);
    Ok(())
}

fn render_trace(
    definition: Pg6Definition,
    result_digest: Digest,
    cases: &[Pg6CaseResult],
) -> String {
    let mut output = String::new();
    {
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG6_TRACE_SCHEMA);
        row("gate", &GateId::Pg6);
        row("scenario", &PG6_SCENARIO);
        row("benchmark_definition", &definition.digest());
        row("config_digest", &definition.config_digest());
        row("scene_golden_lock_digest", &definition.corpus_lock_digest());
        row("engine", &pg6_identity().engine.name());
        row("tier", &pg6_identity().tier.name());
        row("thread_profile", &THREAD_PROFILE);
        row("threads", &PG6_THREADS);
        row("warmup_frames_per_scene", &PG6_WARMUP_FRAMES_PER_SCENE);
        row("sample_count", &cases.len());
        row("result_digest", &result_digest);
    }
    for (index, case) in cases.iter().enumerate() {
        let _ = writeln!(
            output,
            "scene\t{index}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            case.scene,
            case.warm_frame_digest,
            case.measured_frame_digest,
            case.warm.heap_allocs_this_frame,
            case.measured.heap_allocs_this_frame,
            case.measured.arena_buffer_bytes,
            case.measured.pool_slots,
        );
    }
    output
}

// ---------------------------------------------------------------------------
// The leak-soak surface (`one-hour-soak-leak`)
// ---------------------------------------------------------------------------

/// Stable soak workload-definition schema.
pub const PG6_SOAK_DEFINITION_SCHEMA: &str = "fmn-perf-pg6-soak-definition/1";
/// Stable soak phase-trace schema.
pub const PG6_SOAK_TRACE_SCHEMA: &str = "fmn-perf-pg6-soak-trace/1";
/// Policy-catalog scenario implemented by the soak producer.
pub const PG6_SOAK_SCENARIO: &str = "one-hour-soak-leak";
/// One RSS-delta sample per window; the catalog requires all three.
pub const PG6_SOAK_WINDOWS: usize = 3;
/// The reason string retained when the host cannot report resident size.
pub const PG6_SOAK_UNSUPPORTED_REASON: &str = "rss-unsupported-host";

const SOAK_CACHE_STATE: &str = "warm-reused-frame-arena-soak";

/// The content-addressed definition of one soak run.
///
/// The window length is a real definition axis: the weekly lane sizes it so
/// three windows fill roughly an hour on the pinned host, while regression
/// tests use small counts. Both produce honest evidence because the count is
/// hashed into the benchmark identity — a short soak can never masquerade as
/// the scheduled one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pg6SoakDefinition {
    /// Full-corpus render passes per measurement window.
    pub iterations_per_window: u32,
}

impl Pg6SoakDefinition {
    /// Construct a soak definition over the committed corpus.
    ///
    /// # Errors
    /// Zero iterations measure nothing and are refused.
    pub fn new(iterations_per_window: u32) -> Result<Self, Pg6Error> {
        if iterations_per_window == 0 {
            return Err(Pg6Error::Fixture(
                "a soak window needs at least one full-corpus iteration".to_owned(),
            ));
        }
        Ok(Self {
            iterations_per_window,
        })
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(self) -> String {
        let mut output = String::new();
        {
            let mut row = |name: &str, value: &dyn fmt::Display| {
                let _ = writeln!(output, "{name}\t{value}");
            };
            row("schema", &PG6_SOAK_DEFINITION_SCHEMA);
            row("gate", &GateId::Pg6);
            row("scenario", &PG6_SOAK_SCENARIO);
            row("unit", &MetricUnit::LeakedBytes.name());
            row("target", &0);
            row("threads", &PG6_THREADS);
            row("windows", &PG6_SOAK_WINDOWS);
            row("iterations_per_window", &self.iterations_per_window);
            row("sample_scope", &"one-rss-delta-per-window");
            row("warmup_frames_per_scene", &PG6_WARMUP_FRAMES_PER_SCENE);
            row("engine", &pg6_identity().engine.name());
            row("tier", &pg6_identity().tier.name());
            row("thread_profile", &THREAD_PROFILE);
            row("cache_state", &SOAK_CACHE_STATE);
            row("output_mode", &OUTPUT_MODE);
            row("frame_width_px", &scene_goldens::WIDTH);
            row("frame_height_px", &scene_goldens::HEIGHT);
            row("scene_golden_lock_digest", &self.corpus_lock_digest());
            row("config_digest", &self.config_digest());
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

    /// Exact C7/C10 renderer/configuration digest for this artifact.
    #[must_use]
    pub fn config_digest(self) -> Digest {
        Pg6Definition::new().config_digest()
    }

    /// Digest of the committed certified corpus lock that fixes scene input.
    #[must_use]
    pub fn corpus_lock_digest(self) -> Digest {
        scene_goldens::certified_lock_digest()
    }

    /// Validate that a baseline names precisely this soak producer.
    ///
    /// # Errors
    /// Returns a typed identity error before any corpus scene is built.
    pub fn validate_baseline(self, baseline: &Baseline) -> Result<(), Pg6Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg6 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != PG6_SOAK_SCENARIO {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::LeakedBytes {
            mismatches.push("unit");
        }
        if baseline.policy.direction != Direction::Exactly {
            mismatches.push("direction");
        }
        if baseline.policy.target != Some(0) {
            mismatches.push("target");
        }
        if baseline.policy.min_valid_samples != PG6_SOAK_WINDOWS {
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
        if !baseline.policy.require_regression_profile {
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
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != pg6_identity().engine.name() {
            mismatches.push("engine");
        }
        if key.tier != pg6_identity().tier.name() {
            mismatches.push("tier");
        }
        if key.thread_profile != THREAD_PROFILE {
            mismatches.push("thread_profile");
        }
        if key.cache_state != SOAK_CACHE_STATE {
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
            Err(Pg6Error::Identity(format!(
                "{PG6_SOAK_SCENARIO} baseline differs from the compiled producer in: {}",
                mismatches.join(", ")
            )))
        }
    }
}

/// One measurement window's retained resident-size proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pg6SoakWindow {
    /// Zero-based window position.
    pub index: usize,
    /// Resident bytes observed before the window's first render.
    pub rss_start: u64,
    /// Resident bytes observed after the window's last render.
    pub rss_end: u64,
    /// `rss_end - rss_start`, floored at zero: shrinking is not leaking.
    pub leaked_bytes: u64,
    /// Full-corpus render passes performed inside the window.
    pub iterations: u32,
}

/// Soak measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6SoakArtifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Per-window resident-size proof; empty on an unsupported host.
    pub windows: Vec<Pg6SoakWindow>,
}

/// Render the committed corpus in steady state and sample resident size.
///
/// After one excluded warm frame per scene, the corpus is rendered
/// `iterations_per_window` times for each of the three windows; the RSS delta
/// across a window is that window's `leaked-bytes` sample. Frame identity is
/// re-checked on every pass, so the soak can never drift into measuring a
/// different workload than the corpus it names.
///
/// `rss_probe` is the injected residency capability —
/// `fmn_platform::topology::current_rss_bytes` over `StdFs` in production. A
/// probe reporting `None` (macOS, Windows, wasm: no procfs) retains all three
/// samples as invalid with [`PG6_SOAK_UNSUPPORTED_REASON`] and skips the burn:
/// evidence the host cannot attribute is recorded as inconclusive, never
/// synthesized.
///
/// # Errors
/// Identity, fixture, and probe errors return before or during the workload;
/// frame drift is explicit. Nonzero leaked bytes remain valid samples so the
/// common exact-policy verifier issues the blocking verdict.
pub fn measure_pg6_soak(
    baseline: &Baseline,
    producer_commit: &str,
    trace_path: impl Into<String>,
    definition: Pg6SoakDefinition,
    rss_probe: &mut dyn FnMut() -> Result<Option<u64>, String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg6SoakArtifacts, Pg6Error> {
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_compiled_cargo_profile(BUILD_PROFILE)?;
    Pg6Definition::new().validate_corpus_lock()?;

    let (key, host_evidence) =
        crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    let mut batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: Vec::with_capacity(PG6_SOAK_WINDOWS),
        evidence: host_evidence,
    };
    let _ = batch.to_tsv()?;

    let probe = |probe: &mut dyn FnMut() -> Result<Option<u64>, String>| {
        probe().map_err(|detail| Pg6Error::Fixture(format!("rss probe: {detail}")))
    };

    let mut windows = Vec::new();
    if probe(rss_probe)?.is_none() {
        for _ in 0..PG6_SOAK_WINDOWS {
            batch
                .samples
                .push(Sample::invalid(0, PG6_SOAK_UNSUPPORTED_REASON));
        }
    } else {
        // Build and warm every scene once, outside every window.
        let config = scene_goldens::frame_config();
        let corpus = scene_goldens::corpus();
        let mut prepared = Vec::with_capacity(SCENES.len());
        for case in SCENES {
            let built = (case.build)(corpus);
            let mut plan = RenderPlan::new();
            plan.sync(&built.stage, 0).map_err(|error| {
                Pg6Error::Fixture(format!("{} render plan: {error}", case.name))
            })?;
            let mono = MonoTable::build(&plan, config.map)
                .map_err(|error| Pg6Error::Fixture(format!("{} monotone: {error}", case.name)))?;
            let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
                .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
            binning
                .prune_occluded(&plan)
                .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
            prepared.push((case.name, plan, mono, binning));
        }
        let mut warmed = Vec::with_capacity(prepared.len());
        for (name, plan, mono, binning) in &prepared {
            let mut arena = FrameArena::new();
            let (frame, digest) = {
                let job = FrameJob::with_identity_in(
                    &mut arena,
                    plan,
                    mono,
                    binning,
                    config,
                    pg6_identity(),
                )
                .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
                let frame = job
                    .render(PG6_THREADS)
                    .map_err(|error| Pg6Error::Render(error.to_string()))?;
                let digest =
                    frame_digest(&frame).map_err(|error| Pg6Error::Render(error.to_string()))?;
                (frame, digest)
            };
            warmed.push((*name, arena, frame, digest));
        }

        for index in 0..PG6_SOAK_WINDOWS {
            let rss_start = probe(rss_probe)?.ok_or_else(|| {
                Pg6Error::Fixture("rss probe stopped reporting mid-soak".to_owned())
            })?;
            for _ in 0..definition.iterations_per_window {
                for ((_, plan, mono, binning), (name, arena, frame, warm_digest)) in
                    prepared.iter().zip(warmed.iter_mut())
                {
                    let job = FrameJob::with_identity_in(
                        arena,
                        plan,
                        mono,
                        binning,
                        config,
                        pg6_identity(),
                    )
                    .map_err(|error| Pg6Error::Fixture(error.to_string()))?;
                    job.render_into(PG6_THREADS, frame)
                        .map_err(|error| Pg6Error::Render(error.to_string()))?;
                    let digest =
                        frame_digest(frame).map_err(|error| Pg6Error::Render(error.to_string()))?;
                    // ubs:ignore — public deterministic frame identity.
                    if digest != *warm_digest {
                        return Err(Pg6Error::Render(format!(
                            "{name} drifted during the soak: warm {warm_digest}, got {digest}"
                        )));
                    }
                }
            }
            let rss_end = probe(rss_probe)?.ok_or_else(|| {
                Pg6Error::Fixture("rss probe stopped reporting mid-soak".to_owned())
            })?;
            let leaked_bytes = rss_end.saturating_sub(rss_start);
            batch.samples.push(Sample::valid(leaked_bytes));
            windows.push(Pg6SoakWindow {
                index,
                rss_start,
                rss_end,
                leaked_bytes,
                iterations: definition.iterations_per_window,
            });
        }
    }

    let trace_tsv = render_soak_trace(definition, &windows);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;

    Ok(Pg6SoakArtifacts {
        batch,
        trace_tsv,
        windows,
    })
}

fn render_soak_trace(definition: Pg6SoakDefinition, windows: &[Pg6SoakWindow]) -> String {
    let mut output = String::new();
    {
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG6_SOAK_TRACE_SCHEMA);
        row("gate", &GateId::Pg6);
        row("scenario", &PG6_SOAK_SCENARIO);
        row("benchmark_definition", &definition.digest());
        row("config_digest", &definition.config_digest());
        row("scene_golden_lock_digest", &definition.corpus_lock_digest());
        row("engine", &pg6_identity().engine.name());
        row("tier", &pg6_identity().tier.name());
        row("thread_profile", &THREAD_PROFILE);
        row("threads", &PG6_THREADS);
        row("iterations_per_window", &definition.iterations_per_window);
        row("windows", &windows.len());
        if windows.is_empty() {
            row("unsupported", &PG6_SOAK_UNSUPPORTED_REASON);
        }
    }
    for window in windows {
        let _ = writeln!(
            output,
            "window\t{}\t{}\t{}\t{}\t{}",
            window.index, window.rss_start, window.rss_end, window.leaked_bytes, window.iterations,
        );
    }
    output
}

/// PG-6 producer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pg6Error {
    /// Common performance schema/evidence failure.
    Perf(PerfError),
    /// Baseline and compiled-producer identity differ.
    Identity(String),
    /// Canonical corpus or derived artifact construction failed.
    Fixture(String),
    /// The real Lumen render or self-golden check failed.
    Render(String),
}

impl fmt::Display for Pg6Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perf(error) => error.fmt(formatter),
            Self::Identity(detail) => write!(formatter, "PG-6 identity: {detail}"),
            Self::Fixture(detail) => write!(formatter, "PG-6 fixture: {detail}"),
            Self::Render(detail) => write!(formatter, "PG-6 render: {detail}"),
        }
    }
}

impl std::error::Error for Pg6Error {}

impl From<PerfError> for Pg6Error {
    fn from(error: PerfError) -> Self {
        Self::Perf(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definition_binds_the_complete_corpus_and_allocation_axes() {
        let definition = Pg6Definition::new();
        let text = definition.to_tsv();
        assert!(text.contains("scenario\tprimitive-steady-allocations\n"));
        assert!(text.contains("sample_scope\tone-post-warmup-frame-per-scene\n"));
        assert!(text.contains("engine\tcertified-cpu\n"));
        assert!(text.contains(&format!("tier\t{}\n", Tier::COMPILED.name())));
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("scene\t"))
                .count(),
            PG6_SAMPLE_COUNT
        );
        // The definition digest pins the complete axis set for the portable
        // tier; the result aggregate is derived from the committed lock
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
                "b4e080d7860c32109e6db021168d7bbc58f60c47ed47952bb57cba568e61b6e0"
            );
        }
    }

    #[test]
    fn embedded_lock_and_compiled_corpus_are_exactly_aligned() {
        Pg6Definition::new()
            .validate_corpus_lock()
            .expect("committed lock and compiled corpus agree");
    }

    #[test]
    fn soak_definition_binds_the_window_axes_and_refuses_empty_windows() {
        assert!(matches!(
            Pg6SoakDefinition::new(0),
            Err(Pg6Error::Fixture(_))
        ));
        let definition = Pg6SoakDefinition::new(40).expect("nonzero window");
        let text = definition.to_tsv();
        assert!(text.contains("scenario\tone-hour-soak-leak\n"));
        assert!(text.contains("unit\tleaked-bytes\n"));
        assert!(text.contains("windows\t3\n"));
        assert!(text.contains("iterations_per_window\t40\n"));
        assert!(text.contains("sample_scope\tone-rss-delta-per-window\n"));
        assert!(text.contains("cache_state\twarm-reused-frame-arena-soak\n"));
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("scene\t"))
                .count(),
            PG6_SAMPLE_COUNT
        );
        // The window length is a real identity axis.
        assert_ne!(
            definition.digest(),
            Pg6SoakDefinition::new(41).expect("other window").digest()
        );
    }

    #[test]
    fn soak_trace_renders_windows_and_marks_unsupported_hosts() {
        let definition = Pg6SoakDefinition::new(2).expect("definition");
        let unsupported = render_soak_trace(definition, &[]);
        assert!(unsupported.starts_with(&format!("schema\t{PG6_SOAK_TRACE_SCHEMA}\n")));
        assert!(unsupported.contains(&format!("unsupported\t{PG6_SOAK_UNSUPPORTED_REASON}\n")));

        let windows = [
            Pg6SoakWindow {
                index: 0,
                rss_start: 1_000,
                rss_end: 1_000,
                leaked_bytes: 0,
                iterations: 2,
            },
            Pg6SoakWindow {
                index: 1,
                rss_start: 1_000,
                rss_end: 5_096,
                leaked_bytes: 4_096,
                iterations: 2,
            },
        ];
        let trace = render_soak_trace(definition, &windows);
        assert!(trace.contains("window\t0\t1000\t1000\t0\t2\n"));
        assert!(trace.contains("window\t1\t1000\t5096\t4096\t2\n"));
        assert!(!trace.contains("unsupported\t"));
    }
}
