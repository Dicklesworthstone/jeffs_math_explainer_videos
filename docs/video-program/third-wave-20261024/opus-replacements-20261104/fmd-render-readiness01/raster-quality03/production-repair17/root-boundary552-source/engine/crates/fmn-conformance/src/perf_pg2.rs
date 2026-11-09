//! Canonical PG-2 raster-throughput producer.
//!
//! This module fixes the workload before reading the clock.  Host and build
//! identity still come from a versioned [`Baseline`]; the producer refuses a
//! baseline whose benchmark/configuration digest does not describe the code it
//! is about to execute.  A measurement is evidence, not a verdict: an
//! unqualified host remains unqualified in the returned [`MeasurementBatch`].

use crate::perf::{
    Baseline, EvidenceKind, EvidenceRef, GateId, MeasurementBatch, MetricUnit, PerfError, Sample,
    require_compiled_cargo_profile, validate_producer_commit,
};
use fmn_core::color::LinearRgba;
use fmn_hash::{Digest, Sha256, sha256};
use fmn_mobject::{JointType, Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_render::{
    Binning, EngineIdentity, FrameConfig, FrameJob, MonoTable, RenderPlan, ScreenMap, Tier, Tiling,
    Viewport, frame_digest,
};
use std::fmt;
use std::fmt::Write as _;
use std::hint::black_box;
use std::time::{Duration, Instant};

/// Stable fixture-definition schema.
pub const PG2_DEFINITION_SCHEMA: &str = "fmn-perf-pg2-definition/1";
/// Stable phase-trace schema.
pub const PG2_TRACE_SCHEMA: &str = "fmn-perf-pg2-trace/1";
/// Total repetitions: 21 required valid observations plus three retained
/// host-quality failures allowed by the policy catalog.
pub const PG2_SAMPLE_COUNT: usize = 24;
/// Minimum valid observations required by the policy catalog.
pub const PG2_MIN_VALID_SAMPLES: usize = 21;
/// Invalid-observation budget declared by the policy catalog.
pub const PG2_MAX_INVALID_SAMPLES: usize = 3;
/// Fixed warm-up work excluded from every repetition.
pub const PG2_WARMUP_ITERATIONS: usize = 3;
/// Plan §17.2's fixed PG-2 worker count.
pub const PG2_THREADS: usize = 8;

const WIDTH: u32 = 512;
const FILL_HEIGHT: u32 = 512;
const STROKE_HEIGHT: u32 = 256;
const FILL_LAYERS: usize = 32;
const FILL_SEGMENTS_PER_PATH: usize = 4;
const STROKE_SEGMENTS: usize = 64;
const STROKE_WIDTH_UNITS: f32 = 600.0;
const STROKE_JOINT: JointType = JointType::Miter;
const STROKE_WIDTH_MILLI_PX: u64 = 6_000;
const FILL_ALPHA_MILLI: u64 = 80;
const STROKE_ALPHA_MILLI: u64 = 1_000;
const FILL_ITERATIONS_PER_SAMPLE: usize = 1;
const STROKE_ITERATIONS_PER_SAMPLE: usize = 4;
const THREAD_PROFILE: &str = "fixed-8";
const CACHE_STATE: &str = "warm";
const OUTPUT_MODE: &str = "raw-rgba16f";
const BUILD_PROFILE: &str = "release-perf";
const FILL_EXPECTED_FRAME_DIGEST: Digest = Digest::from_bytes([
    0xc6, 0x02, 0xc9, 0x7c, 0x5b, 0x57, 0x48, 0xa1, 0x1e, 0x6c, 0x5b, 0xb8, 0x9a, 0xa5, 0x68, 0x84,
    0x2b, 0xbf, 0x9b, 0xf9, 0xf8, 0x5b, 0xdd, 0x09, 0x9f, 0x2e, 0xbe, 0xe4, 0xdd, 0x30, 0xd8, 0x56,
]);
const STROKE_EXPECTED_FRAME_DIGEST: Digest = Digest::from_bytes([
    0x5a, 0xff, 0x78, 0xa5, 0x10, 0x7c, 0xa2, 0x53, 0x28, 0x25, 0x69, 0x16, 0x8b, 0xd1, 0x5d, 0x11,
    0x29, 0xc5, 0xbe, 0x38, 0xa1, 0x35, 0xad, 0xc0, 0xf1, 0x51, 0x37, 0xbd, 0x84, 0xce, 0xe2, 0xc0,
]);

/// One of the two policy-catalog PG-2 workloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pg2Scenario {
    /// Thirty-two translucent, full-frame quadratic rectangles.
    FillCanonical,
    /// One 64-segment alternating-curvature, six-pixel stroke.
    StrokeCanonical,
}

impl Pg2Scenario {
    /// Both canonical scenarios, in policy order.
    pub const ALL: [Self; 2] = [Self::FillCanonical, Self::StrokeCanonical];

    /// Stable policy-catalog spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FillCanonical => "fill-canonical",
            Self::StrokeCanonical => "stroke-canonical",
        }
    }

    /// Parse the stable policy-catalog spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "fill-canonical" => Self::FillCanonical,
            "stroke-canonical" => Self::StrokeCanonical,
            _ => return None,
        })
    }

    const fn height(self) -> u32 {
        match self {
            Self::FillCanonical => FILL_HEIGHT,
            Self::StrokeCanonical => STROKE_HEIGHT,
        }
    }

    const fn iterations_per_sample(self) -> usize {
        match self {
            Self::FillCanonical => FILL_ITERATIONS_PER_SAMPLE,
            Self::StrokeCanonical => STROKE_ITERATIONS_PER_SAMPLE,
        }
    }

    const fn work_units_per_iteration(self) -> u64 {
        let viewport_pixels = WIDTH as u64 * self.height() as u64;
        match self {
            // Plan §17.2 calls this "fill-coverage equivalent": one unit is
            // one output pixel evaluated against one translucent fill layer.
            Self::FillCanonical => viewport_pixels * FILL_LAYERS as u64,
            // Stroke throughput uses output pixels, not segment-pixel pairs.
            Self::StrokeCanonical => viewport_pixels,
        }
    }

    const fn expected_frame_digest(self) -> Digest {
        match self {
            Self::FillCanonical => FILL_EXPECTED_FRAME_DIGEST,
            Self::StrokeCanonical => STROKE_EXPECTED_FRAME_DIGEST,
        }
    }
}

impl fmt::Display for Pg2Scenario {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// The complete, content-addressed definition of one PG-2 workload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pg2Definition {
    /// Policy scenario.
    pub scenario: Pg2Scenario,
}

impl Pg2Definition {
    /// Construct the canonical definition for `scenario`.
    #[must_use]
    pub const fn new(scenario: Pg2Scenario) -> Self {
        Self { scenario }
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(self) -> String {
        let mut output = String::new();
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG2_DEFINITION_SCHEMA);
        row("gate", &GateId::Pg2);
        row("scenario", &self.scenario);
        row("unit", &MetricUnit::MegaPixelsPerSecondMilli.name());
        row("width_px", &WIDTH);
        row("height_px", &self.scenario.height());
        row("threads", &PG2_THREADS);
        row("warmup_iterations", &PG2_WARMUP_ITERATIONS);
        row("sample_count", &PG2_SAMPLE_COUNT);
        row(
            "iterations_per_sample",
            &self.scenario.iterations_per_sample(),
        );
        row(
            "work_units_per_iteration",
            &self.scenario.work_units_per_iteration(),
        );
        row("output_format", &"rgba16f");
        row("screen_map", &"scale-1-origin-0,0");
        row("background_linear_rgba", &"0.02,0.02,0.02,1.0");
        row("aa_policy", &"adaptive");
        row("macro_tile_px", &128);
        row("fine_tile_px", &16);
        row("fixture_input_digest", &fixture_input_digest(self.scenario));
        row(
            "expected_frame_digest",
            &self.scenario.expected_frame_digest(),
        );
        match self.scenario {
            Pg2Scenario::FillCanonical => {
                row("path_count", &FILL_LAYERS);
                row("segments_per_path", &FILL_SEGMENTS_PER_PATH);
                row("overdraw", &FILL_LAYERS);
                row("path_bounds_px", &"-1,-1,513,513");
                row("curvature", &"flat-quadratic-lines");
                row(
                    "fill_linear_rgba",
                    &"0.2+0.6*i/32,0.7-0.5*i/32,0.3+0.4*i/32,0.08-f32",
                );
                row("layer_order", &"i-ascending-0-through-31");
                row("fill_alpha_milli", &FILL_ALPHA_MILLI);
                row("stroke_width_milli_px", &0);
                row("work_unit", &"fill-coverage-pixel");
            }
            Pg2Scenario::StrokeCanonical => {
                row("path_count", &1);
                row("segments_per_path", &STROKE_SEGMENTS);
                row("overdraw", &1);
                row("path_x_bounds_px", &"16,496");
                row("path_centre_y_px", &128);
                row("curvature", &"alternating-control-56px-endpoint-20px");
                row("fill_linear_rgba", &"0,0,0,0-f32");
                row("stroke_linear_rgba", &"0.2,0.7,0.9,1.0-f32");
                row("fill_alpha_milli", &0);
                row("stroke_alpha_milli", &STROKE_ALPHA_MILLI);
                row("stroke_width_milli_px", &STROKE_WIDTH_MILLI_PX);
                row("joint", &"miter");
                row("joint_code", &STROKE_JOINT.to_code());
                row("work_unit", &"output-pixel");
            }
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
        fmn_render::engine::journal_digest(
            EngineIdentity::fast(),
            &frame_config(self.scenario),
            tiling(),
        )
    }

    /// Fixed iterations timed in each repetition.
    #[must_use]
    pub const fn iterations_per_sample(self) -> usize {
        self.scenario.iterations_per_sample()
    }

    /// Exact denominator used to derive `mpx-per-second-milli`.
    #[must_use]
    pub const fn work_units_per_iteration(self) -> u64 {
        self.scenario.work_units_per_iteration()
    }

    /// Self-golden digest required before timing evidence is emitted.
    #[must_use]
    pub const fn expected_frame_digest(self) -> Digest {
        self.scenario.expected_frame_digest()
    }

    /// Validate that a baseline names precisely this producer.
    ///
    /// # Errors
    /// Returns a typed identity error before any fixture is built or clock read.
    pub fn validate_baseline(self, baseline: &Baseline) -> Result<(), Pg2Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg2 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != self.scenario.name() {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::MegaPixelsPerSecondMilli {
            mismatches.push("unit");
        }
        if baseline.policy.min_valid_samples != PG2_MIN_VALID_SAMPLES {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != PG2_MAX_INVALID_SAMPLES {
            mismatches.push("max_invalid_samples");
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
        if key.engine != EngineIdentity::fast().engine.name() {
            mismatches.push("engine");
        }
        if key.tier != Tier::COMPILED.name() {
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
            Err(Pg2Error::Identity(format!(
                "{} baseline differs from the compiled producer in: {}",
                self.scenario,
                mismatches.join(", ")
            )))
        }
    }
}

/// Measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg2Artifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Canonical result-frame digest, computed outside the timed region.
    pub frame_digest: Digest,
}

/// Build and time the PG-2 workload named by `baseline`.
///
/// `trace_path` is recorded and content-addressed, but this function performs
/// no filesystem I/O. The CLI writes the returned trace before the raw bundle
/// using exclusive-create semantics.
///
/// # Errors
/// Returns before timing for identity errors, and otherwise reports fixture,
/// render, or evidence failures. Timer anomalies are retained as invalid
/// samples rather than synthesized into plausible throughput.
pub fn measure_pg2(
    baseline: &Baseline,
    producer_commit: &str,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg2Artifacts, Pg2Error> {
    let scenario = Pg2Scenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        Pg2Error::Identity(format!(
            "unsupported PG-2 scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    let definition = Pg2Definition::new(scenario);
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    // Reject an impossible publication target before any clock or workload
    // work. The final evidence reference is rebuilt from the real trace bytes.
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_release_perf_artifact()?;

    // Validate the complete batch identity before doing expensive work.
    // `MeasurementBatch::to_tsv` is the one canonical aggregate validator.
    let (key, host_evidence) =
        crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    let mut batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: Vec::new(),
        evidence: host_evidence,
    };
    let _ = batch.to_tsv()?;

    let (fixture, mut phases) = Fixture::build(scenario)?;
    let job = fixture.job()?;

    let allocation_start = Instant::now();
    let mut frame = job
        .render(PG2_THREADS)
        .map_err(|error| Pg2Error::Render(error.to_string()))?;
    phases.push(PhaseTiming::new(
        "output-allocation-and-prime",
        allocation_start.elapsed(),
    ));
    let golden_start = Instant::now();
    let prime_digest = frame_digest(&frame).map_err(|error| Pg2Error::Render(error.to_string()))?;
    // ubs:ignore — public frame self-golden, not authentication material.
    if prime_digest != definition.expected_frame_digest() {
        return Err(Pg2Error::Render(format!(
            "{} self-golden drift: expected {}, got {}",
            scenario,
            definition.expected_frame_digest(),
            prime_digest
        )));
    }
    phases.push(PhaseTiming::new(
        "prime-self-golden-check",
        golden_start.elapsed(),
    ));

    let warmup_start = Instant::now();
    for _ in 0..PG2_WARMUP_ITERATIONS {
        job.render_into(PG2_THREADS, &mut frame)
            .map_err(|error| Pg2Error::Render(error.to_string()))?;
    }
    phases.push(PhaseTiming::new("warmup-raster", warmup_start.elapsed()));

    let mut elapsed = Vec::with_capacity(PG2_SAMPLE_COUNT);
    for _ in 0..PG2_SAMPLE_COUNT {
        let start = Instant::now();
        for _ in 0..definition.iterations_per_sample() {
            job.render_into(PG2_THREADS, &mut frame)
                .map_err(|error| Pg2Error::Render(error.to_string()))?;
        }
        let duration = start.elapsed();
        black_box(frame.as_bytes());
        elapsed.push(duration.as_nanos());
    }
    batch.samples = samples_from_elapsed(definition, &elapsed);

    let result_digest =
        frame_digest(&frame).map_err(|error| Pg2Error::Render(error.to_string()))?;
    // ubs:ignore — public deterministic frame identity, not authentication material.
    if result_digest != prime_digest {
        return Err(Pg2Error::Render(format!(
            "{} result changed during the measurement: prime {}, final {}",
            scenario, prime_digest, result_digest
        )));
    }
    let trace_tsv = render_trace(definition, result_digest, &phases, &elapsed, &batch.samples);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;

    Ok(Pg2Artifacts {
        batch,
        trace_tsv,
        frame_digest: result_digest,
    })
}

fn require_release_perf_artifact() -> Result<(), Pg2Error> {
    require_compiled_cargo_profile(BUILD_PROFILE).map_err(Pg2Error::from)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PhaseTiming {
    name: &'static str,
    elapsed_ns: u128,
}

impl PhaseTiming {
    fn new(name: &'static str, elapsed: Duration) -> Self {
        Self {
            name,
            elapsed_ns: elapsed.as_nanos(),
        }
    }
}

struct Fixture {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
}

impl Fixture {
    fn build(scenario: Pg2Scenario) -> Result<(Self, Vec<PhaseTiming>), Pg2Error> {
        let mut phases = Vec::with_capacity(4);

        let stage_start = Instant::now();
        let stage = match scenario {
            Pg2Scenario::FillCanonical => fill_stage()?,
            Pg2Scenario::StrokeCanonical => stroke_stage()?,
        };
        phases.push(PhaseTiming::new("fixture-build", stage_start.elapsed()));

        let config = frame_config(scenario);
        let plan_start = Instant::now();
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .map_err(|error| Pg2Error::Fixture(format!("render plan: {error}")))?;
        phases.push(PhaseTiming::new("render-plan-sync", plan_start.elapsed()));

        let mono_start = Instant::now();
        let mono = MonoTable::build(&plan, config.map)
            .map_err(|error| Pg2Error::Fixture(format!("monotone table: {error}")))?;
        phases.push(PhaseTiming::new("mono-table-build", mono_start.elapsed()));

        let binning_start = Instant::now();
        let binning = Binning::build(&plan, config.viewport, tiling(), config.map)
            .map_err(|error| Pg2Error::Fixture(error.to_string()))?;
        phases.push(PhaseTiming::new("binning-build", binning_start.elapsed()));

        Ok((
            Self {
                plan,
                mono,
                binning,
                config,
            },
            phases,
        ))
    }

    fn job(&self) -> Result<FrameJob<'_>, Pg2Error> {
        FrameJob::with_identity(
            &self.plan,
            &self.mono,
            &self.binning,
            self.config,
            EngineIdentity::fast(),
        )
        .map_err(|error| Pg2Error::Fixture(error.to_string()))
    }
}

fn fill_stage() -> Result<Stage, Pg2Error> {
    let mut stage = Stage::new();
    for layer in 0..FILL_LAYERS {
        let mob = stage.add(fill_rectangle(layer));
        stage
            .add_to_scene(mob)
            .map_err(|error| Pg2Error::Fixture(error.to_string()))?;
    }
    Ok(stage)
}

fn stroke_stage() -> Result<Stage, Pg2Error> {
    let mut stage = Stage::new();
    let mob = stage.add(stroked_chain());
    stage
        .add_to_scene(mob)
        .map_err(|error| Pg2Error::Fixture(error.to_string()))?;
    let uniforms = stage
        .uniforms_mut(mob)
        .ok_or_else(|| Pg2Error::Fixture("fresh stroke handle became stale".to_owned()))?;
    uniforms.joint_type = STROKE_JOINT;
    Ok(stage)
}

fn fill_rectangle(layer: usize) -> Mobject {
    let t = layer as f32 / FILL_LAYERS as f32;
    rectangle(
        -1.0,
        -1.0,
        f64::from(WIDTH) + 1.0,
        f64::from(FILL_HEIGHT) + 1.0,
        [0.2 + 0.6 * t, 0.7 - 0.5 * t, 0.3 + 0.4 * t, 0.08],
    )
}

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64, fill: [f32; 4]) -> Mobject {
    let corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]];
    let mut points = vec![[corners[0][0], corners[0][1], 0.0]];
    for pair in corners.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        points.push([0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1]), 0.0]);
        points.push([b[0], b[1], 0.0]);
    }

    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len())
        .expect("perf fixture sizing is bounded");
    for (index, point) in points.iter().enumerate() {
        buffer.write(
            index,
            "point",
            &[point[0] as f32, point[1] as f32, point[2] as f32],
        );
        buffer.write(index, "fill_rgba", &fill);
    }
    Mobject::from_buffer(buffer)
}

fn stroked_chain() -> Mobject {
    let x_lo = 16.0;
    let x_hi = f64::from(WIDTH) - 16.0;
    let span = (x_hi - x_lo) / STROKE_SEGMENTS as f64;
    let centre_y = f64::from(STROKE_HEIGHT) * 0.5;
    let mut points = Vec::with_capacity(2 * STROKE_SEGMENTS + 1);
    points.push([x_lo, centre_y, 0.0]);
    for segment in 0..STROKE_SEGMENTS {
        let x0 = x_lo + segment as f64 * span;
        let x2 = x0 + span;
        let y2 = centre_y
            + if segment.is_multiple_of(2) {
                20.0
            } else {
                -20.0
            };
        let handle_y = centre_y
            + if segment.is_multiple_of(2) {
                -56.0
            } else {
                56.0
            };
        points.push([0.5 * (x0 + x2), handle_y, 0.0]);
        points.push([x2, y2, 0.0]);
    }

    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len())
        .expect("perf fixture sizing is bounded");
    for (index, point) in points.iter().enumerate() {
        buffer.write(
            index,
            "point",
            &[point[0] as f32, point[1] as f32, point[2] as f32],
        );
        buffer.write(index, "fill_rgba", &[0.0, 0.0, 0.0, 0.0]);
        buffer.write(index, "stroke_rgba", &[0.2, 0.7, 0.9, 1.0]);
        buffer.write(index, "stroke_width", &[STROKE_WIDTH_UNITS]);
    }
    Mobject::from_buffer(buffer)
}

fn frame_config(scenario: Pg2Scenario) -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: scenario.height(),
        },
        ScreenMap::default(),
        LinearRgba {
            r: 0.02,
            g: 0.02,
            b: 0.02,
            a: 1.0,
        },
    )
}

fn fixture_input_digest(scenario: Pg2Scenario) -> Digest {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg2-fixture-input-v1");
    hash_field(&mut hash, scenario.name().as_bytes());
    match scenario {
        Pg2Scenario::FillCanonical => {
            for layer in 0..FILL_LAYERS {
                let mobject = fill_rectangle(layer);
                hash_record_buffer(&mut hash, &mobject.buffer);
            }
        }
        Pg2Scenario::StrokeCanonical => {
            let mobject = stroked_chain();
            hash_record_buffer(&mut hash, &mobject.buffer);
            hash.update(&STROKE_JOINT.to_code().to_bits().to_be_bytes());
        }
    }
    hash.finalize()
}

fn hash_record_buffer(hash: &mut Sha256, buffer: &RecordBuffer) {
    hash.update(&(buffer.len() as u64).to_be_bytes());
    for row in 0..buffer.len() {
        for field in ["point", "fill_rgba", "stroke_rgba", "stroke_width"] {
            hash_field(hash, field.as_bytes());
            match buffer.read(row, field) {
                Some(values) => {
                    hash.update(&[1]);
                    hash.update(&(values.len() as u64).to_be_bytes());
                    for value in values {
                        hash.update(&value.to_bits().to_be_bytes());
                    }
                }
                None => hash.update(&[0]),
            }
        }
    }
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update(&(bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

const fn tiling() -> Tiling {
    Tiling {
        macro_tile: 128,
        fine_tile: 16,
    }
}

fn samples_from_elapsed(definition: Pg2Definition, elapsed_ns: &[u128]) -> Vec<Sample> {
    elapsed_ns
        .iter()
        .map(|&elapsed| throughput_sample(definition, elapsed))
        .collect()
}

fn throughput_sample(definition: Pg2Definition, elapsed_ns: u128) -> Sample {
    if elapsed_ns == 0 {
        return Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds");
    }
    // (work pixels / elapsed ns) * 1e9 / 1e3 gives thousandths
    // of one megapixel per second.
    let Some(numerator) = u128::from(definition.work_units_per_iteration())
        .checked_mul(definition.iterations_per_sample() as u128)
        .and_then(|value| value.checked_mul(1_000_000))
    else {
        return Sample::invalid(0, "throughput numerator exceeds the u128 range");
    };
    let value = numerator / elapsed_ns;
    match u64::try_from(value) {
        Ok(value) => Sample::valid(value),
        Err(_) => Sample::invalid(u64::MAX, "throughput exceeds the u64 sample range"),
    }
}

fn render_trace(
    definition: Pg2Definition,
    result_digest: Digest,
    phases: &[PhaseTiming],
    elapsed: &[u128],
    samples: &[Sample],
) -> String {
    let mut output = String::new();
    let mut row = |name: &str, value: &dyn fmt::Display| {
        let _ = writeln!(output, "{name}\t{value}");
    };
    row("schema", &PG2_TRACE_SCHEMA);
    row("gate", &GateId::Pg2);
    row("scenario", &definition.scenario);
    row("benchmark_definition", &definition.digest());
    row("config_digest", &definition.config_digest());
    row("engine", &EngineIdentity::fast().engine.name());
    row("tier", &Tier::COMPILED.name());
    row("thread_profile", &THREAD_PROFILE);
    row("warmup_iterations", &PG2_WARMUP_ITERATIONS);
    row("sample_count", &samples.len());
    row("iterations_per_sample", &definition.iterations_per_sample());
    row(
        "work_units_per_iteration",
        &definition.work_units_per_iteration(),
    );
    row("frame_digest", &result_digest);
    for phase in phases {
        let _ = writeln!(
            output,
            "phase\t{}\t{}\tnanoseconds",
            phase.name, phase.elapsed_ns
        );
    }
    for (index, (elapsed_ns, sample)) in elapsed.iter().zip(samples).enumerate() {
        let (validity, reason) = match &sample.invalid_reason {
            Some(reason) => ("invalid", reason.as_str()),
            None => ("valid", "-"),
        };
        let _ = writeln!(
            output,
            "sample\t{index}\t{elapsed_ns}\t{}\t{validity}\t{reason}",
            sample.value
        );
    }
    output
}

/// PG-2 producer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pg2Error {
    /// Common performance schema/evidence failure.
    Perf(PerfError),
    /// Baseline and compiled-producer identity differ.
    Identity(String),
    /// Canonical fixture construction failed.
    Fixture(String),
    /// The real Lumen render failed.
    Render(String),
}

impl fmt::Display for Pg2Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perf(error) => error.fmt(formatter),
            Self::Identity(detail) => write!(formatter, "PG-2 identity: {detail}"),
            Self::Fixture(detail) => write!(formatter, "PG-2 fixture: {detail}"),
            Self::Render(detail) => write!(formatter, "PG-2 render: {detail}"),
        }
    }
}

impl std::error::Error for Pg2Error {}

impl From<PerfError> for Pg2Error {
    fn from(error: PerfError) -> Self {
        Self::Perf(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_spelling_is_closed() {
        for scenario in Pg2Scenario::ALL {
            assert_eq!(Pg2Scenario::parse(scenario.name()), Some(scenario));
        }
        assert_eq!(Pg2Scenario::parse("fill"), None);
        assert_eq!(Pg2Scenario::parse("stroke-canonical\n"), None);
    }

    #[test]
    fn definitions_state_every_required_workload_axis() {
        let fill_definition = Pg2Definition::new(Pg2Scenario::FillCanonical);
        let fill = fill_definition.to_tsv();
        assert!(fill.contains("segments_per_path\t4\n"));
        assert!(fill.contains("overdraw\t32\n"));
        assert!(fill.contains("fill_alpha_milli\t80\n"));
        assert!(fill.contains("curvature\tflat-quadratic-lines\n"));
        assert!(fill.contains("background_linear_rgba\t0.02,0.02,0.02,1.0\n"));
        assert!(fill.contains("path_bounds_px\t-1,-1,513,513\n"));
        assert_eq!(
            fill_definition.digest().to_string(),
            "da96dc95191a8416c6b2d337a24d470ba83ebe099bcc4298652f3a700cb15a60"
        );

        let stroke_definition = Pg2Definition::new(Pg2Scenario::StrokeCanonical);
        let stroke = stroke_definition.to_tsv();
        assert!(stroke.contains("segments_per_path\t64\n"));
        assert!(stroke.contains("stroke_width_milli_px\t6000\n"));
        assert!(stroke.contains("alternating-control-56px-endpoint-20px"));
        assert!(stroke.contains("stroke_alpha_milli\t1000\n"));
        assert!(stroke.contains("stroke_linear_rgba\t0.2,0.7,0.9,1.0-f32\n"));
        assert_eq!(
            stroke_definition.digest().to_string(),
            "bf6308d096653785846fe48e4c190982f89cf22756cb601bc7f43fcd648c59b4"
        );
    }

    #[test]
    fn throughput_conversion_is_exact_integer_math() {
        let definition = Pg2Definition::new(Pg2Scenario::StrokeCanonical);
        let elapsed = u128::from(definition.work_units_per_iteration())
            * definition.iterations_per_sample() as u128
            * 1_000_000
            / 120_000;
        assert_eq!(
            throughput_sample(definition, elapsed),
            Sample::valid(120_000)
        );
        assert_eq!(
            throughput_sample(definition, 0),
            Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds")
        );
    }

    #[test]
    fn artifact_profile_check_uses_the_compiled_identity() {
        let result = require_release_perf_artifact();
        if crate::perf::COMPILED_CARGO_PROFILE == BUILD_PROFILE {
            assert_eq!(result, Ok(()));
        } else {
            assert!(matches!(
                result,
                Err(Pg2Error::Perf(PerfError::Identity(_)))
            ));
        }
    }

    #[test]
    fn fixtures_are_repeatable_at_the_declared_thread_count() {
        for scenario in Pg2Scenario::ALL {
            let (fixture, _) = Fixture::build(scenario).expect("canonical fixture builds");
            let job = fixture.job().expect("canonical job builds");
            let first = job.render(PG2_THREADS).expect("first render");
            let second = job.render(PG2_THREADS).expect("second render");
            assert_eq!(
                frame_digest(&first).expect("first digest"),
                Pg2Definition::new(scenario).expected_frame_digest(),
                "{scenario} self-golden drifted"
            );
            assert_eq!(
                frame_digest(&second).expect("second digest"),
                Pg2Definition::new(scenario).expected_frame_digest(),
                "{scenario}"
            );
        }
    }
}
