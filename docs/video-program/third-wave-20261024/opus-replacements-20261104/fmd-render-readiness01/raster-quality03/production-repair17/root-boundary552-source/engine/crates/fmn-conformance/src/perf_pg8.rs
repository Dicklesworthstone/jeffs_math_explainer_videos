//! Canonical PG-8 binding-tax producers (§17.2, fm-zoi).
//!
//! PG-8 measures the Python boundary's four declared scene classes: the
//! callback-free `native-builtins` class as a ratio against a pure-Rust
//! twin (budget 1.10×, already policy), and the `per-frame-callback`,
//! `point-transform-callback`, and `dynamic-subclass` classes as absolute
//! nanosecond workloads with published budgets.
//!
//! The measurement itself lives in `fmn-python`'s `perf_harness` — the
//! producer must drive the real bridge, and the bridge crate is the only
//! place pyo3 may link (D3; the governed closure keeps pyo3 out of this
//! crate's shipped dependency surface). This module therefore owns what a
//! producer must own — workload identity, self-goldens, sample plans,
//! robust-statistics assembly, traces, and baseline validation — and takes
//! the raw observations by injection. `tests/perf_pg8.rs` supplies the
//! real sampler through a dev-dependency on `fmn-python`; fm-inr.4's
//! publisher can wire the same sampler without renames.
//!
//! Every workload is fixed before the clock is read: 64 built-in mobjects
//! (point ×3 + rgba ×4 lanes, four records each), 60 frames of `dt = 1/30`
//! per repetition, 24 retained repetitions (21 valid plus 3 allowed
//! host-quality failures), 3 untimed warm-ups. Each class's final scene
//! state is bit-verified against a locked self-golden; `native-builtins`
//! additionally requires the pure-Rust twin to end bit-identical.

use crate::perf::{
    Baseline, EvidenceKind, EvidenceRef, GateId, MeasurementBatch, MetricUnit, PerfError, Sample,
    require_compiled_cargo_profile, validate_producer_commit,
};
use fmn_hash::{Digest, Sha256, sha256};
use std::fmt;
use std::fmt::Write as _;

/// Stable fixture-definition schema.
pub const PG8_DEFINITION_SCHEMA: &str = "fmn-perf-pg8-definition/1";
/// Stable phase-trace schema.
pub const PG8_TRACE_SCHEMA: &str = "fmn-perf-pg8-trace/1";
/// Total repetitions: 21 required valid observations plus three retained
/// host-quality failures allowed by the policy catalog.
pub const PG8_SAMPLE_COUNT: usize = 24;
/// Minimum valid observations required by the policy catalog.
pub const PG8_MIN_VALID_SAMPLES: usize = 21;
/// Invalid-observation budget declared by the policy catalog.
pub const PG8_MAX_INVALID_SAMPLES: usize = 3;
/// Fixed warm-up repetitions excluded from every measurement.
pub const PG8_WARMUP_ITERATIONS: usize = 3;
/// Frames per repetition; the catalog's nanosecond targets cover exactly
/// this many frames.
pub const PG8_FRAMES_PER_REPETITION: usize = 60;
/// Mobjects per workload scene.
pub const PG8_MOBJECTS: usize = 64;
/// Records per mobject.
pub const PG8_POINTS_PER_MOBJECT: usize = 4;
const PG8_LANES_PER_RECORD: usize = 3 + 4;
/// Exact byte length of every fixed-workload bridge and twin result state.
pub const PG8_RESULT_STATE_BYTES: usize =
    PG8_MOBJECTS * PG8_POINTS_PER_MOBJECT * PG8_LANES_PER_RECORD * std::mem::size_of::<f32>();
/// Repetition frame delta: `1 / PG8_DT_DENOMINATOR` seconds.
pub const PG8_DT_DENOMINATOR: u64 = 30;

const THREAD_PROFILE: &str = "single-thread";
const BUILD_PROFILE: &str = "release-perf";
const CACHE_STATE: &str = "none";
const ENGINE: &str = "fmn-python-bridge";
const TIER: &str = "portable";
const OUTPUT_MODE: &str = "record-buffer-state";

/// The workload corpus is content-addressed by hashing the bridge-side
/// producer source itself; any workload drift fails the definition digest.
const HARNESS_SOURCE: &str = include_str!("../../fmn-python/src/perf_harness.rs");

// Locked after exercising the real committed fmn-python harness paths. Any
// semantic or workload drift must be reviewed explicitly rather than
// silently changing the state being timed.
const NATIVE_BUILTINS_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0x49, 0x96, 0x7d, 0x61, 0xab, 0x60, 0x66, 0x49, 0x27, 0xd0, 0x5d, 0xc9, 0xb3, 0x2e, 0xb0, 0x44,
    0x0b, 0x1d, 0x83, 0xa5, 0x3f, 0x56, 0xf0, 0x0b, 0x09, 0x14, 0xf1, 0x7f, 0x9c, 0x35, 0xcb, 0x76,
]);
const PER_FRAME_CALLBACK_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0x0b, 0x0e, 0x77, 0xc2, 0xe5, 0x94, 0xc6, 0x4a, 0x36, 0xaf, 0x70, 0x05, 0x98, 0x1a, 0xc3, 0xc8,
    0xbe, 0xc1, 0x02, 0x60, 0x53, 0x85, 0xa9, 0x1e, 0x45, 0x69, 0x20, 0x6d, 0xf5, 0x34, 0x6e, 0x36,
]);
const POINT_TRANSFORM_CALLBACK_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0x49, 0x96, 0x7d, 0x61, 0xab, 0x60, 0x66, 0x49, 0x27, 0xd0, 0x5d, 0xc9, 0xb3, 0x2e, 0xb0, 0x44,
    0x0b, 0x1d, 0x83, 0xa5, 0x3f, 0x56, 0xf0, 0x0b, 0x09, 0x14, 0xf1, 0x7f, 0x9c, 0x35, 0xcb, 0x76,
]);
const DYNAMIC_SUBCLASS_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0x1c, 0x75, 0x87, 0x05, 0xab, 0x5e, 0x82, 0x01, 0xb1, 0x4d, 0x32, 0x84, 0xc1, 0xe9, 0x21, 0x9d,
    0x8b, 0xd1, 0xa9, 0xde, 0x83, 0xe9, 0xf1, 0x20, 0x6b, 0x38, 0x4c, 0xfd, 0x1f, 0xc8, 0xd7, 0xce,
]);

/// One of the four policy-catalog PG-8 workloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pg8Scenario {
    /// Built-in mobjects + engine-executable updaters; no Python callback.
    /// The sample value is bridge/twin latency ratio in parts per million.
    NativeBuiltins,
    /// One `set_field` Python updater per mobject per frame.
    PerFrameCallback,
    /// One live-view point-transform Python updater per mobject per frame.
    PointTransformCallback,
    /// Override-constructed dynamic subclasses with bound-method updaters;
    /// construction is inside every timed repetition.
    DynamicSubclass,
}

impl Pg8Scenario {
    /// All canonical scenarios, in policy order.
    pub const ALL: [Self; 4] = [
        Self::NativeBuiltins,
        Self::PerFrameCallback,
        Self::PointTransformCallback,
        Self::DynamicSubclass,
    ];

    /// Stable policy-catalog spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NativeBuiltins => "native-builtins",
            Self::PerFrameCallback => "per-frame-callback",
            Self::PointTransformCallback => "point-transform-callback",
            Self::DynamicSubclass => "dynamic-subclass",
        }
    }

    /// Parse the stable policy-catalog spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "native-builtins" => Self::NativeBuiltins,
            "per-frame-callback" => Self::PerFrameCallback,
            "point-transform-callback" => Self::PointTransformCallback,
            "dynamic-subclass" => Self::DynamicSubclass,
            _ => return None,
        })
    }

    /// The policy's measurement unit for this scenario.
    #[must_use]
    pub const fn unit(self) -> MetricUnit {
        match self {
            Self::NativeBuiltins => MetricUnit::RatioPpm,
            Self::PerFrameCallback | Self::PointTransformCallback | Self::DynamicSubclass => {
                MetricUnit::Nanoseconds
            }
        }
    }

    /// Whether the timed repetition includes scene construction.
    #[must_use]
    pub const fn rebuilds_per_repetition(self) -> bool {
        matches!(self, Self::DynamicSubclass)
    }

    /// Exact result self-golden.
    #[must_use]
    pub const fn expected_result_digest(self) -> Digest {
        match self {
            Self::NativeBuiltins => NATIVE_BUILTINS_EXPECTED_RESULT_DIGEST,
            Self::PerFrameCallback => PER_FRAME_CALLBACK_EXPECTED_RESULT_DIGEST,
            Self::PointTransformCallback => POINT_TRANSFORM_CALLBACK_EXPECTED_RESULT_DIGEST,
            Self::DynamicSubclass => DYNAMIC_SUBCLASS_EXPECTED_RESULT_DIGEST,
        }
    }
}

impl fmt::Display for Pg8Scenario {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Complete content-addressed definition of one PG-8 workload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg8Definition {
    /// Policy scenario.
    pub scenario: Pg8Scenario,
    fixture_input_digest: Digest,
    config_digest: Digest,
}

impl Pg8Definition {
    /// Construct the canonical definition for `scenario`. No clock is read.
    #[must_use]
    pub fn new(scenario: Pg8Scenario) -> Self {
        Self {
            scenario,
            fixture_input_digest: sha256(HARNESS_SOURCE.as_bytes()),
            config_digest: config_digest(scenario),
        }
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut output = String::new();
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG8_DEFINITION_SCHEMA);
        row("gate", &GateId::Pg8);
        row("scenario", &self.scenario);
        row("unit", &self.scenario.unit().name());
        row("engine", &ENGINE);
        row("tier", &TIER);
        row("thread_profile", &THREAD_PROFILE);
        row("cache_state", &CACHE_STATE);
        row("output_mode", &OUTPUT_MODE);
        row("warmup_iterations", &PG8_WARMUP_ITERATIONS);
        row("sample_count", &PG8_SAMPLE_COUNT);
        row("frames_per_repetition", &PG8_FRAMES_PER_REPETITION);
        row("mobjects", &PG8_MOBJECTS);
        row("points_per_mobject", &PG8_POINTS_PER_MOBJECT);
        row("dt_denominator", &PG8_DT_DENOMINATOR);
        row(
            "construction_timed",
            &self.scenario.rebuilds_per_repetition(),
        );
        row("fixture_input_digest", &self.fixture_input_digest);
        row(
            "expected_result_digest",
            &self.scenario.expected_result_digest(),
        );
        row("config_digest", &self.config_digest);
        output
    }

    /// SHA-256 of [`Self::to_tsv`].
    #[must_use]
    pub fn digest(&self) -> Digest {
        sha256(self.to_tsv().as_bytes())
    }

    /// Exact semantic configuration digest.
    #[must_use]
    pub const fn config_digest(&self) -> Digest {
        self.config_digest
    }

    /// Exact fixture source digest.
    #[must_use]
    pub const fn fixture_input_digest(&self) -> Digest {
        self.fixture_input_digest
    }

    /// Self-golden required before timing evidence is emitted.
    #[must_use]
    pub const fn expected_result_digest(&self) -> Digest {
        self.scenario.expected_result_digest()
    }

    /// Validate that a baseline names precisely this producer.
    ///
    /// # Errors
    /// Returns a typed identity error before any clock read.
    pub fn validate_baseline(&self, baseline: &Baseline) -> Result<(), Pg8Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg8 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != self.scenario.name() {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != self.scenario.unit() {
            mismatches.push("unit");
        }
        if baseline.policy.min_valid_samples != PG8_MIN_VALID_SAMPLES {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != PG8_MAX_INVALID_SAMPLES {
            mismatches.push("max_invalid_samples");
        }
        // ubs:ignore - public benchmark identity digest, not an authentication secret.
        if key.benchmark_definition != self.digest() {
            mismatches.push("benchmark_definition");
        }
        // ubs:ignore - public configuration identity digest, not an authentication secret.
        if key.config_digest != self.config_digest() {
            mismatches.push("config_digest");
        }
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != ENGINE {
            mismatches.push("engine");
        }
        if key.tier != TIER {
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
            Err(Pg8Error::Identity(format!(
                "{} baseline differs from the compiled producer in: {}",
                self.scenario,
                mismatches.join(", ")
            )))
        }
    }
}

fn config_digest(scenario: Pg8Scenario) -> Digest {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg8-config-v1");
    hash.update(scenario.name().as_bytes());
    hash.update(&(PG8_FRAMES_PER_REPETITION as u64).to_be_bytes());
    hash.update(&(PG8_MOBJECTS as u64).to_be_bytes());
    hash.update(&(PG8_POINTS_PER_MOBJECT as u64).to_be_bytes());
    hash.update(&PG8_DT_DENOMINATOR.to_be_bytes());
    hash.update(&[u8::from(scenario.rebuilds_per_repetition())]);
    hash.finalize()
}

/// One retained repetition from the bridge-side harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg8Observation {
    /// Wall time of one bridge-side workload repetition.
    pub elapsed_ns: u128,
    /// Wall time of the pure-Rust twin (`native-builtins` only).
    pub reference_ns: Option<u128>,
    /// Host-quality failure reason; the observation is retained regardless.
    pub invalid_reason: Option<String>,
}

/// Raw harness output for one scenario.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg8Measurement {
    /// Exactly [`PG8_SAMPLE_COUNT`] retained repetitions.
    pub observations: Vec<Pg8Observation>,
    /// Final bridge-side scene state.
    pub result_state: Vec<u8>,
    /// Final pure-Rust twin state (`native-builtins` only); required to be
    /// bit-identical to `result_state`.
    pub reference_state: Option<Vec<u8>>,
}

/// Measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg8Artifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Canonical result digest, computed outside the timed region.
    pub result_digest: Digest,
}

/// Assemble a replayable measurement batch from raw harness observations.
///
/// Pure and deterministic: no clock, no release-perf requirement, no
/// filesystem. The self-golden and twin-equality checks run here, so every
/// caller — the dev-profile harness tests and the release-perf probe —
/// gets identical evidence rules.
///
/// # Errors
/// Returns identity/sample-plan/self-golden faults. Timer anomalies arrive
/// as invalid observations and are retained, not errors.
pub fn assemble_pg8(
    baseline: &Baseline,
    producer_commit: &str,
    measurement: &Pg8Measurement,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg8Artifacts, Pg8Error> {
    let scenario = Pg8Scenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        Pg8Error::Identity(format!(
            "unsupported PG-8 scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    let definition = Pg8Definition::new(scenario);
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    // Direct assembly callers may supply arbitrarily large retained state.
    // Refuse malformed provenance and an impossible publication target before
    // hashing or otherwise inspecting that measurement content.
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;

    validate_measurement_shape(scenario, measurement)?;
    let result_digest = sha256(&measurement.result_state);
    // ubs:ignore - public self-golden digest, not an authentication secret.
    if result_digest != definition.expected_result_digest() {
        return Err(Pg8Error::SelfGolden(format!(
            "{scenario} result digest drifted: expected {}, measured {result_digest}",
            definition.expected_result_digest()
        )));
    }
    match (scenario, &measurement.reference_state) {
        (Pg8Scenario::NativeBuiltins, Some(twin)) => {
            if *twin != measurement.result_state {
                return Err(Pg8Error::SelfGolden(
                    "native-builtins bridge and pure-Rust twin states differ".to_owned(),
                ));
            }
        }
        (Pg8Scenario::NativeBuiltins, None) => {
            return Err(Pg8Error::SamplePlan(
                "native-builtins requires the pure-Rust twin state".to_owned(),
            ));
        }
        (_, Some(_)) => {
            return Err(Pg8Error::SamplePlan(format!(
                "{scenario} must not carry a pure-Rust twin state"
            )));
        }
        (_, None) => {}
    }

    let (key, host_evidence) =
        crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    let mut batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: Vec::new(),
        evidence: host_evidence,
    };
    let _ = batch.to_tsv()?;
    batch.samples = scenario_samples(scenario, &measurement.observations);
    let trace_tsv = render_trace(&definition, measurement, &batch.samples);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;

    Ok(Pg8Artifacts {
        batch,
        trace_tsv,
        result_digest,
    })
}

fn validate_measurement_shape(
    scenario: Pg8Scenario,
    measurement: &Pg8Measurement,
) -> Result<(), Pg8Error> {
    if measurement.observations.len() != PG8_SAMPLE_COUNT {
        return Err(Pg8Error::SamplePlan(format!(
            "{} requires {PG8_SAMPLE_COUNT} retained repetitions, got {}",
            scenario,
            measurement.observations.len()
        )));
    }
    for (index, observation) in measurement.observations.iter().enumerate() {
        if let Some(reason) = &observation.invalid_reason {
            Sample::validate_invalid_reason(reason)?;
        }
        if scenario != Pg8Scenario::NativeBuiltins && observation.reference_ns.is_some() {
            return Err(Pg8Error::SamplePlan(format!(
                "{scenario} observation {index} must not carry pure-Rust twin timing"
            )));
        }
    }
    if measurement.result_state.len() != PG8_RESULT_STATE_BYTES {
        return Err(Pg8Error::SamplePlan(format!(
            "{scenario} result_state must be exactly {PG8_RESULT_STATE_BYTES} bytes, got {}",
            measurement.result_state.len()
        )));
    }
    match (scenario, &measurement.reference_state) {
        (Pg8Scenario::NativeBuiltins, Some(twin)) if twin.len() != PG8_RESULT_STATE_BYTES => {
            Err(Pg8Error::SamplePlan(format!(
                "native-builtins reference_state must be exactly {PG8_RESULT_STATE_BYTES} bytes, got {}",
                twin.len()
            )))
        }
        (Pg8Scenario::NativeBuiltins, None) => Err(Pg8Error::SamplePlan(
            "native-builtins requires the pure-Rust twin state".to_owned(),
        )),
        (Pg8Scenario::NativeBuiltins, Some(_)) | (_, None) => Ok(()),
        (_, Some(_)) => Err(Pg8Error::SamplePlan(format!(
            "{scenario} must not carry a pure-Rust twin state"
        ))),
    }
}

/// Build, time, and assemble the PG-8 workload named by `baseline`.
///
/// `sampler` drives the real bridge (`fmn-python`'s `perf_harness`) and
/// must return exactly [`PG8_SAMPLE_COUNT`] retained observations after
/// running [`PG8_WARMUP_ITERATIONS`] untimed warm-ups. This function
/// performs no artifact filesystem I/O.
///
/// # Errors
/// Returns before timing for identity/build faults; workload and
/// self-golden drift are explicit errors.
pub fn measure_pg8(
    baseline: &Baseline,
    producer_commit: &str,
    sampler: &dyn Fn(Pg8Scenario) -> Result<Pg8Measurement, Pg8Error>,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg8Artifacts, Pg8Error> {
    let scenario = Pg8Scenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        Pg8Error::Identity(format!(
            "unsupported PG-8 scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    let definition = Pg8Definition::new(scenario);
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    // Reject an impossible publication target before the bridge sampler runs.
    // `assemble_pg8` rebuilds the final reference from the real trace bytes.
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_release_perf_artifact()?;
    let measurement = sampler(scenario)?;
    assemble_pg8(
        baseline,
        producer_commit,
        &measurement,
        trace_path,
        qualification,
    )
}

fn scenario_samples(scenario: Pg8Scenario, observations: &[Pg8Observation]) -> Vec<Sample> {
    observations
        .iter()
        .map(|observation| scenario_sample(scenario, observation))
        .collect()
}

fn scenario_sample(scenario: Pg8Scenario, observation: &Pg8Observation) -> Sample {
    if let Some(reason) = &observation.invalid_reason {
        let value = u64::try_from(observation.elapsed_ns).unwrap_or(u64::MAX);
        return Sample::invalid(value, reason.clone());
    }
    let value = match scenario {
        Pg8Scenario::NativeBuiltins => {
            let Some(reference_ns) = observation.reference_ns else {
                return Sample::invalid(
                    u64::MAX,
                    "native-builtins observation lacks its pure-Rust twin timing",
                );
            };
            if reference_ns == 0 {
                return Sample::invalid(
                    u64::MAX,
                    "pure-Rust twin reported zero elapsed nanoseconds",
                );
            }
            let Some(numerator) = observation.elapsed_ns.checked_mul(1_000_000) else {
                return Sample::invalid(u64::MAX, "ratio numerator exceeds the u128 range");
            };
            numerator / reference_ns
        }
        Pg8Scenario::PerFrameCallback
        | Pg8Scenario::PointTransformCallback
        | Pg8Scenario::DynamicSubclass => observation.elapsed_ns,
    };
    if value == 0 {
        return Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds");
    }
    match u64::try_from(value) {
        Ok(value) => Sample::valid(value),
        Err(_) => Sample::invalid(u64::MAX, "measurement exceeds the u64 sample range"),
    }
}

fn require_release_perf_artifact() -> Result<(), Pg8Error> {
    require_compiled_cargo_profile(BUILD_PROFILE).map_err(Pg8Error::from)
}

fn render_trace(
    definition: &Pg8Definition,
    measurement: &Pg8Measurement,
    samples: &[Sample],
) -> String {
    let mut output = String::new();
    let mut row = |name: &str, value: &dyn fmt::Display| {
        let _ = writeln!(output, "{name}\t{value}");
    };
    row("schema", &PG8_TRACE_SCHEMA);
    row("gate", &GateId::Pg8);
    row("scenario", &definition.scenario);
    row("benchmark_definition", &definition.digest());
    row("config_digest", &definition.config_digest());
    row("fixture_input_digest", &definition.fixture_input_digest());
    row("engine", &ENGINE);
    row("tier", &TIER);
    row("thread_profile", &THREAD_PROFILE);
    row("cache_state", &CACHE_STATE);
    row("output_mode", &OUTPUT_MODE);
    row("warmup_iterations", &PG8_WARMUP_ITERATIONS);
    row("sample_count", &samples.len());
    row("frames_per_repetition", &PG8_FRAMES_PER_REPETITION);
    row("mobjects", &PG8_MOBJECTS);
    row("dt_denominator", &PG8_DT_DENOMINATOR);
    row("result_digest", &sha256(&measurement.result_state));
    for (index, (observation, sample)) in measurement
        .observations
        .iter()
        .zip(samples.iter())
        .enumerate()
    {
        let (validity, reason) = match &sample.invalid_reason {
            Some(reason) => ("invalid", reason.as_str()),
            None => ("valid", "-"),
        };
        let reference = observation
            .reference_ns
            .map_or_else(|| "-".to_owned(), |value| value.to_string());
        let _ = writeln!(
            output,
            "sample\t{index}\t{}\t{reference}\t{}\t{validity}\t{reason}",
            observation.elapsed_ns, sample.value
        );
    }
    output
}

/// PG-8 producer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pg8Error {
    /// Baseline identity, scenario spelling, or artifact-profile mismatch.
    Identity(String),
    /// Wrong repetition count or a missing/forbidden twin state.
    SamplePlan(String),
    /// Result state drifted from the locked self-golden.
    SelfGolden(String),
    /// The injected sampler failed to drive the bridge.
    Harness(String),
    /// Perf-rig evidence or bundle fault.
    Rig(PerfError),
}

impl fmt::Display for Pg8Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity(message) => write!(formatter, "pg-8 identity: {message}"),
            Self::SamplePlan(message) => write!(formatter, "pg-8 sample plan: {message}"),
            Self::SelfGolden(message) => write!(formatter, "pg-8 self-golden: {message}"),
            Self::Harness(message) => write!(formatter, "pg-8 harness: {message}"),
            Self::Rig(error) => write!(formatter, "pg-8 rig: {error}"),
        }
    }
}

impl std::error::Error for Pg8Error {}

impl From<PerfError> for Pg8Error {
    fn from(error: PerfError) -> Self {
        Self::Rig(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_names_parse_round_trip() {
        for scenario in Pg8Scenario::ALL {
            assert_eq!(Pg8Scenario::parse(scenario.name()), Some(scenario));
        }
        assert_eq!(Pg8Scenario::parse("opening-class-g2"), None);
    }

    #[test]
    fn definitions_are_deterministic_and_scenario_scoped() {
        let digests: Vec<Digest> = Pg8Scenario::ALL
            .iter()
            .map(|scenario| Pg8Definition::new(*scenario).digest())
            .collect();
        for (index, digest) in digests.iter().enumerate() {
            assert_eq!(
                *digest,
                Pg8Definition::new(Pg8Scenario::ALL[index]).digest()
            );
            assert!(
                !digests[..index].contains(digest),
                "definitions must differ"
            );
        }
    }

    #[test]
    fn ratio_samples_require_a_live_twin() {
        let missing = scenario_sample(
            Pg8Scenario::NativeBuiltins,
            &Pg8Observation {
                elapsed_ns: 100,
                reference_ns: None,
                invalid_reason: None,
            },
        );
        assert!(missing.invalid_reason.is_some());
        let zero = scenario_sample(
            Pg8Scenario::NativeBuiltins,
            &Pg8Observation {
                elapsed_ns: 100,
                reference_ns: Some(0),
                invalid_reason: None,
            },
        );
        assert!(zero.invalid_reason.is_some());
        let ratio = scenario_sample(
            Pg8Scenario::NativeBuiltins,
            &Pg8Observation {
                elapsed_ns: 110,
                reference_ns: Some(100),
                invalid_reason: None,
            },
        );
        assert_eq!(ratio.invalid_reason, None);
        assert_eq!(ratio.value, 1_100_000);
        assert_eq!(
            scenario_sample(
                Pg8Scenario::NativeBuiltins,
                &Pg8Observation {
                    elapsed_ns: u128::MAX,
                    reference_ns: Some(u128::MAX),
                    invalid_reason: None,
                },
            ),
            Sample::invalid(u64::MAX, "ratio numerator exceeds the u128 range")
        );
    }

    #[test]
    fn invalid_observations_are_retained_with_reasons() {
        let sample = scenario_sample(
            Pg8Scenario::PerFrameCallback,
            &Pg8Observation {
                elapsed_ns: u128::MAX,
                reference_ns: None,
                invalid_reason: Some("host preemption".to_owned()),
            },
        );
        assert_eq!(sample.invalid_reason.as_deref(), Some("host preemption"));
        assert_eq!(sample.value, u64::MAX);
    }

    #[test]
    fn artifact_profile_check_uses_the_compiled_identity() {
        let result = require_release_perf_artifact();
        assert_eq!(
            result.is_ok(),
            crate::perf::COMPILED_CARGO_PROFILE == BUILD_PROFILE
        );
        if let Err(error) = result {
            assert!(
                error
                    .to_string()
                    .contains(crate::perf::COMPILED_CARGO_PROFILE)
            );
        }
    }
}
