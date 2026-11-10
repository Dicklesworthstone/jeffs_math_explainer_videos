//! Canonical PG-7 native-typesetting latency producers.
//!
//! Workload identity is fixed before the clock is read. Formula-cold keeps
//! the engine and CPU warm while deliberately leaving the content cache
//! disabled. Formula-cached proves an exact-key miss, primes through the real
//! preflight mechanism, verifies and decodes the exact stored payload, then
//! times verified store hits. Both formula workloads clear the engine's
//! memory front before every repetition, outside the timed region, and refuse
//! a repetition unless the engine's own counters show it was served by
//! exactly the layer the scenario names: one layout, or one verified store
//! hit, and never a memory-front hit. Native text lays out exactly 10,000
//! non-whitespace glyphs through `fmn-text`. All three paths verify a
//! bit-level result self-golden before and after timing.

use crate::perf::{
    Baseline, EvidenceKind, EvidenceRef, GateId, MeasurementBatch, MetricUnit, PerfError, Sample,
    require_compiled_cargo_profile, validate_producer_commit,
};
use fmd_math::Style;
use fmn_cache::{NamespacePolicy, Store};
use fmn_hash::{Digest, Sha256, sha256};
use fmn_tex::{Mode, TYPESET_FORMAT_VERSION, TexEngine, Typeset};
use fmn_text::{FontBook, TextLayout, TextRequest, layout_text};
use std::fmt;
use std::fmt::Write as _;
use std::hint::black_box;
use std::time::{Duration, Instant};

/// Stable fixture-definition schema.
pub const PG7_DEFINITION_SCHEMA: &str = "fmn-perf-pg7-definition/1";
/// Stable phase-trace schema.
pub const PG7_TRACE_SCHEMA: &str = "fmn-perf-pg7-trace/2";
/// Total repetitions: 21 required valid observations plus three retained
/// host-quality failures allowed by the policy catalog.
pub const PG7_SAMPLE_COUNT: usize = 24;
/// Minimum valid observations required by the policy catalog.
pub const PG7_MIN_VALID_SAMPLES: usize = 21;
/// Invalid-observation budget declared by the policy catalog.
pub const PG7_MAX_INVALID_SAMPLES: usize = 3;
/// Fixed warm-up work excluded from every repetition.
pub const PG7_WARMUP_ITERATIONS: usize = 3;
/// Exact number of non-whitespace glyphs in the native-text fixture.
pub const PG7_TEXT_GLYPHS: usize = 10_000;

const PACK_CONTENT_ID: &str = "fmd-math/pack/default";
const FORMULA_SOURCE: &str = "q^7+z";
const CORPUS_RULES_VERSION: &str = "1";
const CORPUS_SHA256: &str = "a8325e49e0ce78fcc735533952740e9adeaaa5cb10f9c13d73aaa3ba4bf883fc";
const MEDIAN_CORPUS_PAIR_SHA256: &str =
    "5d79ab0a3d5eaf9db101f5dcef2630326c1eff6766a31734e1155f2487988a4d";
const FORMULA_SELECTION: &str =
    "math-occurrence-weighted-lower-median:utf8-bytes,construct-count,pair-sha256";
const FORMULA_OCCURRENCE_RANK: &str = "6010-of-12020";
const FORMULA_FIXTURE_RELATION: &str = "synthetic-utf8-byte-and-construct-count-isomorph";
const FORMULA_SOURCE_BYTES: usize = 5;
const FORMULA_CONSTRUCT_COUNT: usize = 1;
const TEXT_PATTERN: &str = "Abcdefghijklmnopqrstuvwxyz";
const THREAD_PROFILE: &str = "single-thread";
const BUILD_PROFILE: &str = "release-perf";
const FORMULA_OUTPUT_MODE: &str = "typeset-data";
const TEXT_OUTPUT_MODE: &str = "text-layout";
const FORMULA_COLD_CACHE_STATE: &str = "disabled-cold";
const FORMULA_CACHED_CACHE_STATE: &str = "verified-hit";
const TEXT_CACHE_STATE: &str = "none";

// Locked after exercising the real committed fmn-tex/fmn-text paths. Any
// semantic or fixture drift must be reviewed explicitly rather than silently
// changing the workload being timed.
const FORMULA_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0xbd, 0xcf, 0x1c, 0x5a, 0x20, 0xc0, 0x8c, 0xae, 0x08, 0x76, 0xea, 0x98, 0xa1, 0xf3, 0x95, 0x94,
    0xf3, 0x8c, 0x2e, 0xd6, 0x82, 0xaf, 0x7a, 0x71, 0xa5, 0xf2, 0x24, 0xe6, 0x92, 0x9a, 0x28, 0x2c,
]);
const TEXT_EXPECTED_RESULT_DIGEST: Digest = Digest::from_bytes([
    0x31, 0xc6, 0xd1, 0xaa, 0x62, 0xd1, 0x13, 0x6d, 0x6f, 0x23, 0x6f, 0x96, 0x5a, 0xdd, 0x48, 0xa9,
    0x48, 0xb9, 0x2d, 0xb8, 0xd0, 0xda, 0x25, 0x29, 0x52, 0x97, 0x9d, 0xc8, 0xe2, 0x03, 0xac, 0x91,
]);

/// One of the three policy-catalog PG-7 workloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pg7Scenario {
    /// A cache-disabled layout of the canonical corpus-representative formula.
    FormulaCold,
    /// A verified exact-key hit for that formula after production preflight.
    FormulaCached,
    /// Native layout of exactly 10,000 non-whitespace glyphs.
    Text10kGlyph,
}

impl Pg7Scenario {
    /// All canonical scenarios, in policy order.
    pub const ALL: [Self; 3] = [Self::FormulaCold, Self::FormulaCached, Self::Text10kGlyph];

    /// Stable policy-catalog spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FormulaCold => "formula-cold",
            Self::FormulaCached => "formula-cached",
            Self::Text10kGlyph => "text-10k-glyph",
        }
    }

    /// Parse the stable policy-catalog spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "formula-cold" => Self::FormulaCold,
            "formula-cached" => Self::FormulaCached,
            "text-10k-glyph" => Self::Text10kGlyph,
            _ => return None,
        })
    }

    /// Cache-state identity carried by the comparable-run key.
    #[must_use]
    pub const fn cache_state(self) -> &'static str {
        match self {
            Self::FormulaCold => FORMULA_COLD_CACHE_STATE,
            Self::FormulaCached => FORMULA_CACHED_CACHE_STATE,
            Self::Text10kGlyph => TEXT_CACHE_STATE,
        }
    }

    /// Semantic engine identity carried by the comparable-run key.
    #[must_use]
    pub const fn engine(self) -> &'static str {
        match self {
            Self::FormulaCold | Self::FormulaCached => "fmn-tex",
            Self::Text10kGlyph => "fmn-text",
        }
    }

    /// Output identity carried by the comparable-run key.
    #[must_use]
    pub const fn output_mode(self) -> &'static str {
        match self {
            Self::FormulaCold | Self::FormulaCached => FORMULA_OUTPUT_MODE,
            Self::Text10kGlyph => TEXT_OUTPUT_MODE,
        }
    }

    /// The one engine layer that must serve each timed formula repetition.
    const fn timed_layer(self) -> Option<TimedLayer> {
        match self {
            Self::FormulaCold => Some(TimedLayer::Layout),
            Self::FormulaCached => Some(TimedLayer::StoreHit),
            Self::Text10kGlyph => None,
        }
    }

    /// Exact result self-golden.
    #[must_use]
    pub const fn expected_result_digest(self) -> Digest {
        match self {
            Self::FormulaCold | Self::FormulaCached => FORMULA_EXPECTED_RESULT_DIGEST,
            Self::Text10kGlyph => TEXT_EXPECTED_RESULT_DIGEST,
        }
    }
}

impl fmt::Display for Pg7Scenario {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Complete content-addressed definition of one PG-7 workload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg7Definition {
    /// Policy scenario.
    pub scenario: Pg7Scenario,
    fixture_input_digest: Digest,
    config_digest: Digest,
}

impl Pg7Definition {
    /// Construct the canonical definition for `scenario`.
    ///
    /// # Errors
    /// Reports a committed engine/font initialization fault. No clock is read.
    pub fn new(scenario: Pg7Scenario) -> Result<Self, Pg7Error> {
        let (fixture_input_digest, config_digest) = match scenario {
            Pg7Scenario::FormulaCold | Pg7Scenario::FormulaCached => {
                let engine = build_tex_engine()?;
                (
                    sha256(FORMULA_SOURCE.as_bytes()),
                    formula_config_digest(&engine)?,
                )
            }
            Pg7Scenario::Text10kGlyph => {
                let source = text_source();
                let book =
                    FontBook::bundled().map_err(|error| Pg7Error::Fixture(error.to_string()))?;
                (
                    sha256(source.as_bytes()),
                    text_config_digest(&book.available())?,
                )
            }
        };
        Ok(Self {
            scenario,
            fixture_input_digest,
            config_digest,
        })
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut output = String::new();
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG7_DEFINITION_SCHEMA);
        row("gate", &GateId::Pg7);
        row("scenario", &self.scenario);
        row("unit", &MetricUnit::Nanoseconds.name());
        row("engine", &self.scenario.engine());
        row("tier", &"portable");
        row("thread_profile", &THREAD_PROFILE);
        row("cache_state", &self.scenario.cache_state());
        row("output_mode", &self.scenario.output_mode());
        row("warmup_iterations", &PG7_WARMUP_ITERATIONS);
        row("sample_count", &PG7_SAMPLE_COUNT);
        row("fixture_input_digest", &self.fixture_input_digest);
        row(
            "expected_result_digest",
            &self.scenario.expected_result_digest(),
        );
        row("config_digest", &self.config_digest);
        match self.scenario {
            Pg7Scenario::FormulaCold | Pg7Scenario::FormulaCached => {
                row("formula_source", &FORMULA_SOURCE);
                row("formula_mode", &"math-display");
                row("corpus_rules_version", &CORPUS_RULES_VERSION);
                row("corpus_sha256", &CORPUS_SHA256);
                row("corpus_selection", &FORMULA_SELECTION);
                row("corpus_occurrence_rank", &FORMULA_OCCURRENCE_RANK);
                row("median_corpus_pair_sha256", &MEDIAN_CORPUS_PAIR_SHA256);
                row("fixture_relation", &FORMULA_FIXTURE_RELATION);
                row("formula_source_bytes", &FORMULA_SOURCE_BYTES);
                row("formula_construct_count", &FORMULA_CONSTRUCT_COUNT);
                row("pack_content_id", &PACK_CONTENT_ID);
                row("typeset_format_version", &TYPESET_FORMAT_VERSION);
                row(
                    "timed_operation",
                    &if self.scenario == Pg7Scenario::FormulaCold {
                        "cache-disabled-layout"
                    } else {
                        "verified-cache-hit"
                    },
                );
                row(
                    "cache_prime",
                    &if self.scenario == Pg7Scenario::FormulaCached {
                        "tex-engine-preflight"
                    } else {
                        "none"
                    },
                );
                row(
                    "cache_eviction_guard",
                    &if self.scenario == Pg7Scenario::FormulaCached {
                        "exact-key-pin"
                    } else {
                        "none"
                    },
                );
                row("memory_front", &"cleared-before-each-repetition");
                row(
                    "timed_layer",
                    &self.scenario.timed_layer().map_or("none", TimedLayer::name),
                );
            }
            Pg7Scenario::Text10kGlyph => {
                row("text_pattern", &TEXT_PATTERN);
                row("source_bytes", &PG7_TEXT_GLYPHS);
                row("expected_glyphs", &PG7_TEXT_GLYPHS);
                row("markup", &false);
                row("ligatures", &false);
                row("width", &"none");
                row("line_breaker", &"greedy");
                row("alignment", &"left");
                row("justify", &false);
                row("indent_f64_bits", &0_f64.to_bits());
                row("line_spacing_f64_bits", &1_f64.to_bits());
                row("timed_operation", &"native-text-layout");
            }
        }
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
    pub fn validate_baseline(&self, baseline: &Baseline) -> Result<(), Pg7Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg7 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != self.scenario.name() {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::Nanoseconds {
            mismatches.push("unit");
        }
        if baseline.policy.min_valid_samples != PG7_MIN_VALID_SAMPLES {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != PG7_MAX_INVALID_SAMPLES {
            mismatches.push("max_invalid_samples");
        }
        if key.benchmark_definition != self.digest() {
            mismatches.push("benchmark_definition");
        }
        if key.config_digest != self.config_digest() {
            mismatches.push("config_digest");
        }
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != self.scenario.engine() {
            mismatches.push("engine");
        }
        if key.tier != "portable" {
            mismatches.push("tier");
        }
        if key.thread_profile != THREAD_PROFILE {
            mismatches.push("thread_profile");
        }
        if key.cache_state != self.scenario.cache_state() {
            mismatches.push("cache_state");
        }
        if key.output_mode != self.scenario.output_mode() {
            mismatches.push("output_mode");
        }
        if key.external_tool_fingerprint.is_some() {
            mismatches.push("external_tool_fingerprint");
        }
        if mismatches.is_empty() {
            Ok(())
        } else {
            Err(Pg7Error::Identity(format!(
                "{} baseline differs from the compiled producer in: {}",
                self.scenario,
                mismatches.join(", ")
            )))
        }
    }
}

/// Measurement output before the caller persists the two artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg7Artifacts {
    /// Canonical raw measurement bundle, including the trace reference.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the batch's phase-trace evidence row.
    pub trace_tsv: String,
    /// Canonical result digest, computed outside the timed region.
    pub result_digest: Digest,
}

/// Build and time the PG-7 workload named by `baseline`.
///
/// `cache` must be present only for `formula-cached`, and that store must not
/// already contain the canonical key. The cached producer refuses an already
/// warm store so its preflight transition is evidence rather than an
/// assumption. This function performs no artifact filesystem I/O.
///
/// # Errors
/// Returns before timing for identity/build/cache-state faults. Workload
/// errors and self-golden drift are explicit; timer anomalies are retained as
/// invalid samples.
pub fn measure_pg7(
    baseline: &Baseline,
    producer_commit: &str,
    cache: Option<&Store>,
    trace_path: impl Into<String>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg7Artifacts, Pg7Error> {
    let scenario = Pg7Scenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        Pg7Error::Identity(format!(
            "unsupported PG-7 scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    let definition = Pg7Definition::new(scenario)?;
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    // Reject an impossible publication target before any cache or clock work.
    // The final evidence reference is rebuilt from the real trace bytes.
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_release_perf_artifact()?;
    match (scenario, cache) {
        (Pg7Scenario::FormulaCached, None) => {
            return Err(Pg7Error::CacheState(
                "formula-cached requires an injected empty cache store".to_owned(),
            ));
        }
        (Pg7Scenario::FormulaCached, Some(_))
        | (Pg7Scenario::FormulaCold | Pg7Scenario::Text10kGlyph, None) => {}
        (_, Some(_)) => {
            return Err(Pg7Error::CacheState(format!(
                "{scenario} requires cache state {}, not an injected store",
                scenario.cache_state()
            )));
        }
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

    let measured = match scenario {
        Pg7Scenario::FormulaCold => measure_formula_cold(&definition)?,
        Pg7Scenario::FormulaCached => {
            let store = cache.ok_or_else(|| {
                Pg7Error::CacheState("formula-cached store disappeared".to_owned())
            })?;
            measure_formula_cached(&definition, store)?
        }
        Pg7Scenario::Text10kGlyph => measure_text(&definition)?,
    };
    batch.samples = latency_samples(&measured.elapsed_ns);
    let trace_tsv = render_trace(&definition, &measured, &batch.samples);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;

    Ok(Pg7Artifacts {
        batch,
        trace_tsv,
        result_digest: measured.result_digest,
    })
}

fn measure_formula_cold(definition: &Pg7Definition) -> Result<Measured, Pg7Error> {
    let setup_start = Instant::now();
    let engine = build_tex_engine()?;
    let mut phases = vec![PhaseTiming::new(
        "engine-initialization-cache-disabled",
        setup_start.elapsed(),
    )];

    let golden_start = Instant::now();
    let (_, prime, _) = formula_repetition(&engine, TimedLayer::Layout)?;
    let prime_digest = typeset_digest(&prime)?;
    require_self_golden(definition, prime_digest)?;
    phases.push(PhaseTiming::new(
        "prime-self-golden-check",
        golden_start.elapsed(),
    ));

    let warmup_start = Instant::now();
    for _ in 0..PG7_WARMUP_ITERATIONS {
        let (_, output, _) = formula_repetition(&engine, TimedLayer::Layout)?;
        black_box(output);
    }
    phases.push(PhaseTiming::new(
        "warmup-cache-disabled-layout",
        warmup_start.elapsed(),
    ));

    let timed = formula_samples(&engine, TimedLayer::Layout, prime)?;
    let result_digest = typeset_digest(&timed.final_output)?;
    require_unchanged(definition, prime_digest, result_digest)?;
    Ok(Measured {
        phases,
        elapsed_ns: timed.elapsed_ns,
        result_digest,
        cache_before: "disabled",
        cache_after: "disabled",
        cache_payload_digest: None,
        timed_layers: Some(timed.layers),
    })
}

fn measure_formula_cached(definition: &Pg7Definition, store: &Store) -> Result<Measured, Pg7Error> {
    let setup_start = Instant::now();
    let uncached_engine = build_tex_engine()?;
    let namespace = store
        .namespace(
            "typeset",
            TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .map_err(|error| Pg7Error::CacheState(error.to_string()))?;
    let key = uncached_engine
        .cache_key(Mode::Math(Style::Display), FORMULA_SOURCE)
        .ok_or_else(|| Pg7Error::CacheState("canonical PG-7 source has no cache key".to_owned()))?;
    let before = namespace
        .get(&key)
        .map_err(|error| Pg7Error::CacheState(error.to_string()))?;
    if before.is_some() {
        return Err(Pg7Error::CacheState(format!(
            "canonical key {} was already present before preflight; use a fresh store",
            key.digest()
        )));
    }
    let engine = uncached_engine
        .with_cache(store)
        .map_err(|error| Pg7Error::CacheState(error.to_string()))?;
    let mut phases = vec![PhaseTiming::new(
        "engine-and-cache-miss-proof",
        setup_start.elapsed(),
    )];

    let preflight_start = Instant::now();
    let mut outcomes = engine
        .preflight(&[(Mode::Math(Style::Display), FORMULA_SOURCE)])
        .map_err(|error| Pg7Error::Workload(error.to_string()))?;
    let outcome = outcomes.pop().ok_or_else(|| {
        Pg7Error::CacheState("preflight returned no outcome for one input".to_owned())
    })?;
    outcome.map_err(|error| Pg7Error::Workload(error.to_string()))?;
    phases.push(PhaseTiming::new(
        "preflight-prime",
        preflight_start.elapsed(),
    ));

    let proof_start = Instant::now();
    let payload = namespace
        .get(&key)
        .map_err(|error| Pg7Error::CacheState(error.to_string()))?
        .ok_or_else(|| {
            Pg7Error::CacheState(
                "preflight returned success but the exact key is absent".to_owned(),
            )
        })?;
    let decoded = Typeset::from_bytes(&payload).map_err(|error| {
        Pg7Error::CacheState(format!(
            "preflight stored an undecodable typeset payload: {error}"
        ))
    })?;
    let prime_digest = typeset_digest(&decoded)?;
    require_self_golden(definition, prime_digest)?;
    let payload_digest = sha256(&payload);
    let _cache_pin = namespace.pin(&key);
    phases.push(PhaseTiming::new(
        "cache-hit-decode-self-golden-and-pin",
        proof_start.elapsed(),
    ));

    let warmup_start = Instant::now();
    for _ in 0..PG7_WARMUP_ITERATIONS {
        let (_, output, _) = formula_repetition(&engine, TimedLayer::StoreHit)?;
        black_box(output);
    }
    phases.push(PhaseTiming::new(
        "warmup-verified-cache-hit",
        warmup_start.elapsed(),
    ));

    let timed = formula_samples(&engine, TimedLayer::StoreHit, decoded)?;

    let after_start = Instant::now();
    let after_payload = namespace
        .get(&key)
        .map_err(|error| Pg7Error::CacheState(error.to_string()))?
        .ok_or_else(|| {
            Pg7Error::CacheState("exact key disappeared during cached measurement".to_owned())
        })?;
    if sha256(&after_payload) != payload_digest {
        return Err(Pg7Error::CacheState(
            "exact-key payload changed during cached measurement".to_owned(),
        ));
    }
    let result_digest = typeset_digest(&timed.final_output)?;
    require_unchanged(definition, prime_digest, result_digest)?;
    phases.push(PhaseTiming::new(
        "post-measurement-cache-hit-proof",
        after_start.elapsed(),
    ));
    Ok(Measured {
        phases,
        elapsed_ns: timed.elapsed_ns,
        result_digest,
        cache_before: "exact-key-miss",
        cache_after: "exact-key-hit",
        cache_payload_digest: Some(payload_digest),
        timed_layers: Some(timed.layers),
    })
}

fn measure_text(definition: &Pg7Definition) -> Result<Measured, Pg7Error> {
    let setup_start = Instant::now();
    let source = text_source();
    let book = FontBook::bundled().map_err(|error| Pg7Error::Fixture(error.to_string()))?;
    let request = TextRequest::plain(&source);
    let mut phases = vec![PhaseTiming::new(
        "font-book-and-source-initialization",
        setup_start.elapsed(),
    )];

    let golden_start = Instant::now();
    let prime =
        layout_text(&book, &request).map_err(|error| Pg7Error::Workload(error.to_string()))?;
    require_text_shape(&prime)?;
    let prime_digest = text_layout_digest(&prime)?;
    require_self_golden(definition, prime_digest)?;
    phases.push(PhaseTiming::new(
        "prime-self-golden-check",
        golden_start.elapsed(),
    ));

    let warmup_start = Instant::now();
    for _ in 0..PG7_WARMUP_ITERATIONS {
        let output =
            layout_text(&book, &request).map_err(|error| Pg7Error::Workload(error.to_string()))?;
        black_box(output);
    }
    phases.push(PhaseTiming::new(
        "warmup-native-text-layout",
        warmup_start.elapsed(),
    ));

    let mut elapsed_ns = Vec::with_capacity(PG7_SAMPLE_COUNT);
    let mut final_output = prime;
    for _ in 0..PG7_SAMPLE_COUNT {
        let start = Instant::now();
        let output =
            layout_text(&book, &request).map_err(|error| Pg7Error::Workload(error.to_string()))?;
        let elapsed = start.elapsed().as_nanos();
        black_box(&output);
        elapsed_ns.push(elapsed);
        final_output = output;
    }
    require_text_shape(&final_output)?;
    let result_digest = text_layout_digest(&final_output)?;
    require_unchanged(definition, prime_digest, result_digest)?;
    Ok(Measured {
        phases,
        elapsed_ns,
        result_digest,
        cache_before: "not-applicable",
        cache_after: "not-applicable",
        cache_payload_digest: None,
        timed_layers: None,
    })
}

fn build_tex_engine() -> Result<TexEngine, Pg7Error> {
    TexEngine::new(PACK_CONTENT_ID, None).map_err(|error| Pg7Error::Fixture(error.to_string()))
}

fn typeset_formula(engine: &TexEngine, source: &str) -> Result<Typeset, Pg7Error> {
    engine
        .typeset(Mode::Math(Style::Display), source)
        .map_err(|error| Pg7Error::Workload(error.to_string()))
}

/// The engine layer that must serve a timed formula repetition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimedLayer {
    /// A fresh `fmd-math` layout: no memory-front hit and no store hit.
    Layout,
    /// A checksum-verified store hit: no memory-front hit and no layout.
    StoreHit,
}

impl TimedLayer {
    const fn name(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::StoreHit => "verified-store-hit",
        }
    }

    /// The exact counter movement of one repetition served by this layer.
    const fn one_repetition(self) -> LayerCounters {
        match self {
            Self::Layout => LayerCounters {
                memory_front_hits: 0,
                layouts: 1,
                store_hits: 0,
            },
            Self::StoreHit => LayerCounters {
                memory_front_hits: 0,
                layouts: 0,
                store_hits: 1,
            },
        }
    }
}

/// The engine's cumulative layer counters, or a difference of two readings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LayerCounters {
    memory_front_hits: u64,
    layouts: u64,
    store_hits: u64,
}

impl LayerCounters {
    fn read(engine: &TexEngine) -> Self {
        Self {
            memory_front_hits: engine.memory_cache_stats().hits,
            layouts: engine.layout_computations(),
            store_hits: engine.persistent_cache_hits(),
        }
    }

    fn since(self, before: Self) -> Result<Self, Pg7Error> {
        let delta = |after: u64, before: u64, name: &str| {
            after.checked_sub(before).ok_or_else(|| {
                Pg7Error::CacheState(format!("engine {name} counter moved backwards"))
            })
        };
        Ok(Self {
            memory_front_hits: delta(
                self.memory_front_hits,
                before.memory_front_hits,
                "memory-front hit",
            )?,
            layouts: delta(self.layouts, before.layouts, "layout")?,
            store_hits: delta(self.store_hits, before.store_hits, "store hit")?,
        })
    }

    const fn saturating_add(self, other: Self) -> Self {
        Self {
            memory_front_hits: self
                .memory_front_hits
                .saturating_add(other.memory_front_hits),
            layouts: self.layouts.saturating_add(other.layouts),
            store_hits: self.store_hits.saturating_add(other.store_hits),
        }
    }
}

/// Refuse a repetition the named layer did not serve alone.
fn require_timed_layer(layer: TimedLayer, delta: LayerCounters) -> Result<(), Pg7Error> {
    if delta == layer.one_repetition() {
        Ok(())
    } else {
        Err(Pg7Error::CacheState(format!(
            "a timed repetition must be exactly one {}, but the engine counted \
             {} memory-front hit(s), {} layout(s) and {} store hit(s)",
            layer.name(),
            delta.memory_front_hits,
            delta.layouts,
            delta.store_hits
        )))
    }
}

/// One formula repetition with a cold memory front. Clearing the front and
/// both counter readings stay outside the timed region.
fn formula_repetition(
    engine: &TexEngine,
    layer: TimedLayer,
) -> Result<(u128, Typeset, LayerCounters), Pg7Error> {
    engine.clear_memory_cache();
    let before = LayerCounters::read(engine);
    let start = Instant::now();
    let output = typeset_formula(engine, FORMULA_SOURCE)?;
    let elapsed = start.elapsed().as_nanos();
    black_box(&output);
    let delta = LayerCounters::read(engine).since(before)?;
    require_timed_layer(layer, delta)?;
    Ok((elapsed, output, delta))
}

struct FormulaSamples {
    elapsed_ns: Vec<u128>,
    final_output: Typeset,
    layers: LayerCounters,
}

fn formula_samples(
    engine: &TexEngine,
    layer: TimedLayer,
    initial: Typeset,
) -> Result<FormulaSamples, Pg7Error> {
    let mut samples = FormulaSamples {
        elapsed_ns: Vec::with_capacity(PG7_SAMPLE_COUNT),
        final_output: initial,
        layers: LayerCounters::default(),
    };
    for _ in 0..PG7_SAMPLE_COUNT {
        let (elapsed, output, delta) = formula_repetition(engine, layer)?;
        samples.elapsed_ns.push(elapsed);
        samples.final_output = output;
        samples.layers = samples.layers.saturating_add(delta);
    }
    Ok(samples)
}

fn text_source() -> String {
    let mut source = String::with_capacity(PG7_TEXT_GLYPHS);
    while source.len() + TEXT_PATTERN.len() <= PG7_TEXT_GLYPHS {
        source.push_str(TEXT_PATTERN);
    }
    for byte in TEXT_PATTERN
        .bytes()
        .take(PG7_TEXT_GLYPHS.saturating_sub(source.len()))
    {
        source.push(char::from(byte));
    }
    source
}

fn require_text_shape(layout: &TextLayout) -> Result<(), Pg7Error> {
    if layout.glyphs.len() != PG7_TEXT_GLYPHS {
        return Err(Pg7Error::Workload(format!(
            "10k fixture produced {} glyphs, expected {PG7_TEXT_GLYPHS}",
            layout.glyphs.len()
        )));
    }
    if layout.lines.len() != 1 || !layout.decorations.is_empty() {
        return Err(Pg7Error::Workload(format!(
            "10k fixture shape drifted: {} lines, {} decorations",
            layout.lines.len(),
            layout.decorations.len()
        )));
    }
    Ok(())
}

fn require_self_golden(definition: &Pg7Definition, actual: Digest) -> Result<(), Pg7Error> {
    if actual == definition.expected_result_digest() {
        Ok(())
    } else {
        Err(Pg7Error::Workload(format!(
            "{} self-golden drift: expected {}, got {}",
            definition.scenario,
            definition.expected_result_digest(),
            actual
        )))
    }
}

fn require_unchanged(
    definition: &Pg7Definition,
    prime: Digest,
    final_result: Digest,
) -> Result<(), Pg7Error> {
    if prime != final_result {
        return Err(Pg7Error::Workload(format!(
            "{} result changed during measurement: prime {prime}, final {final_result}",
            definition.scenario
        )));
    }
    require_self_golden(definition, final_result)
}

fn typeset_digest(typeset: &Typeset) -> Result<Digest, Pg7Error> {
    let bytes = typeset
        .to_bytes()
        .map_err(|error| Pg7Error::Workload(error.to_string()))?;
    Ok(sha256(&bytes))
}

fn text_layout_digest(layout: &TextLayout) -> Result<Digest, Pg7Error> {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg7-text-layout-v1");
    hash_usize(&mut hash, layout.glyphs.len())?;
    for glyph in &layout.glyphs {
        hash_field(&mut hash, glyph.face.family.as_bytes())?;
        hash.update(&[
            u8::from(glyph.face.key.bold),
            u8::from(glyph.face.key.italic),
        ]);
        hash.update(&glyph.gid.to_be_bytes());
        hash.update(&u32::from(glyph.ch).to_be_bytes());
        hash_f64(&mut hash, glyph.x);
        hash_f64(&mut hash, glyph.y);
        hash_f64(&mut hash, glyph.size);
        hash_usize(&mut hash, glyph.span.0)?;
        hash_usize(&mut hash, glyph.span.1)?;
        hash_usize(&mut hash, glyph.char_index)?;
        hash_usize(&mut hash, glyph.cluster_len)?;
        hash_usize(&mut hash, glyph.submobject_index)?;
        hash_usize(&mut hash, glyph.line)?;
        hash_color(&mut hash, glyph.fill);
    }
    hash_usize(&mut hash, layout.decorations.len())?;
    for decoration in &layout.decorations {
        hash_f64(&mut hash, decoration.x);
        hash_f64(&mut hash, decoration.y);
        hash_f64(&mut hash, decoration.width);
        hash_f64(&mut hash, decoration.height);
        hash_usize(&mut hash, decoration.span.0)?;
        hash_usize(&mut hash, decoration.span.1)?;
        hash_color(&mut hash, decoration.fill);
    }
    hash_usize(&mut hash, layout.lines.len())?;
    for line in &layout.lines {
        hash_f64(&mut hash, line.baseline);
        hash_f64(&mut hash, line.width);
        hash_usize(&mut hash, line.glyphs.0)?;
        hash_usize(&mut hash, line.glyphs.1)?;
    }
    hash_f64(&mut hash, layout.width);
    hash_f64(&mut hash, layout.height);
    hash_f64(&mut hash, layout.depth);
    Ok(hash.finalize())
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) -> Result<(), Pg7Error> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| Pg7Error::Fixture("fixture field exceeds u64".to_owned()))?;
    hash.update(&length.to_be_bytes());
    hash.update(bytes);
    Ok(())
}

fn hash_usize(hash: &mut Sha256, value: usize) -> Result<(), Pg7Error> {
    let value = u64::try_from(value)
        .map_err(|_| Pg7Error::Fixture("fixture length exceeds u64".to_owned()))?;
    hash.update(&value.to_be_bytes());
    Ok(())
}

fn hash_f64(hash: &mut Sha256, value: f64) {
    hash.update(&value.to_bits().to_be_bytes());
}

fn hash_color(hash: &mut Sha256, color: Option<fmn_core::color::Srgb>) {
    match color {
        Some(color) => {
            hash.update(&[1]);
            hash_f64(hash, color.r);
            hash_f64(hash, color.g);
            hash_f64(hash, color.b);
        }
        None => hash.update(&[0]),
    }
}

fn formula_config_digest(engine: &TexEngine) -> Result<Digest, Pg7Error> {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg7-formula-config-v1");
    hash_field(&mut hash, PACK_CONTENT_ID.as_bytes())?;
    hash.update(engine.fingerprint().digest().as_bytes());
    hash.update(&TYPESET_FORMAT_VERSION.to_be_bytes());
    hash.update(b"math-display");
    Ok(hash.finalize())
}

fn text_config_digest(roster: &[String]) -> Result<Digest, Pg7Error> {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-pg7-text-config-v1");
    for family in roster {
        hash_field(&mut hash, family.as_bytes())?;
    }
    hash.update(&[
        0, // markup
        0, // ligatures
        0, // width absent
        0, // greedy
        0, // left
        0, // justify
    ]);
    hash.update(&0_f64.to_bits().to_be_bytes());
    hash.update(&1_f64.to_bits().to_be_bytes());
    Ok(hash.finalize())
}

fn require_release_perf_artifact() -> Result<(), Pg7Error> {
    require_compiled_cargo_profile(BUILD_PROFILE).map_err(Pg7Error::from)
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

#[derive(Debug)]
struct Measured {
    phases: Vec<PhaseTiming>,
    elapsed_ns: Vec<u128>,
    result_digest: Digest,
    cache_before: &'static str,
    cache_after: &'static str,
    cache_payload_digest: Option<Digest>,
    /// Engine layer counts summed over the timed repetitions only.
    timed_layers: Option<LayerCounters>,
}

fn latency_samples(elapsed_ns: &[u128]) -> Vec<Sample> {
    elapsed_ns
        .iter()
        .map(|&value| latency_sample(value))
        .collect()
}

fn latency_sample(elapsed_ns: u128) -> Sample {
    if elapsed_ns == 0 {
        return Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds");
    }
    match u64::try_from(elapsed_ns) {
        Ok(value) => Sample::valid(value),
        Err(_) => Sample::invalid(u64::MAX, "latency exceeds the u64 sample range"),
    }
}

fn render_trace(definition: &Pg7Definition, measured: &Measured, samples: &[Sample]) -> String {
    let mut output = String::new();
    let mut row = |name: &str, value: &dyn fmt::Display| {
        let _ = writeln!(output, "{name}\t{value}");
    };
    row("schema", &PG7_TRACE_SCHEMA);
    row("gate", &GateId::Pg7);
    row("scenario", &definition.scenario);
    row("benchmark_definition", &definition.digest());
    row("config_digest", &definition.config_digest());
    row("fixture_input_digest", &definition.fixture_input_digest());
    row("engine", &definition.scenario.engine());
    row("tier", &"portable");
    row("thread_profile", &THREAD_PROFILE);
    row("cache_state", &definition.scenario.cache_state());
    row("cache_before", &measured.cache_before);
    row("cache_after", &measured.cache_after);
    row(
        "cache_payload_digest",
        &measured
            .cache_payload_digest
            .map_or_else(|| "-".to_owned(), |digest| digest.to_string()),
    );
    row(
        "timed_layer",
        &definition
            .scenario
            .timed_layer()
            .map_or("none", TimedLayer::name),
    );
    let layer_count = |count: fn(&LayerCounters) -> u64| {
        measured
            .timed_layers
            .as_ref()
            .map_or_else(|| "-".to_owned(), |layers| count(layers).to_string())
    };
    row(
        "timed_memory_front_hits",
        &layer_count(|layers| layers.memory_front_hits),
    );
    row("timed_layouts", &layer_count(|layers| layers.layouts));
    row("timed_store_hits", &layer_count(|layers| layers.store_hits));
    row("warmup_iterations", &PG7_WARMUP_ITERATIONS);
    row("sample_count", &samples.len());
    row("result_digest", &measured.result_digest);
    for phase in &measured.phases {
        let _ = writeln!(
            output,
            "phase\t{}\t{}\tnanoseconds",
            phase.name, phase.elapsed_ns
        );
    }
    for (index, (elapsed_ns, sample)) in measured.elapsed_ns.iter().zip(samples).enumerate() {
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

/// PG-7 producer failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pg7Error {
    /// Common performance schema/evidence failure.
    Perf(PerfError),
    /// Baseline and compiled-producer identity differ.
    Identity(String),
    /// Canonical fixture construction failed.
    Fixture(String),
    /// Cache state could not be proved.
    CacheState(String),
    /// Real typesetting/text-layout work failed or drifted.
    Workload(String),
}

impl fmt::Display for Pg7Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perf(error) => error.fmt(formatter),
            Self::Identity(detail) => write!(formatter, "PG-7 identity: {detail}"),
            Self::Fixture(detail) => write!(formatter, "PG-7 fixture: {detail}"),
            Self::CacheState(detail) => write!(formatter, "PG-7 cache state: {detail}"),
            Self::Workload(detail) => write!(formatter, "PG-7 workload: {detail}"),
        }
    }
}

impl std::error::Error for Pg7Error {}

impl From<PerfError> for Pg7Error {
    fn from(error: PerfError) -> Self {
        Self::Perf(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_spelling_is_closed() {
        for scenario in Pg7Scenario::ALL {
            assert_eq!(Pg7Scenario::parse(scenario.name()), Some(scenario));
        }
        assert_eq!(Pg7Scenario::parse("formula"), None);
        assert_eq!(Pg7Scenario::parse("text-10k-glyph\n"), None);
    }

    #[test]
    fn text_source_is_exactly_ten_thousand_ascii_glyphs() {
        let source = text_source();
        assert_eq!(source.len(), PG7_TEXT_GLYPHS);
        assert_eq!(source.chars().count(), PG7_TEXT_GLYPHS);
        assert!(source.chars().all(|character| !character.is_whitespace()));
    }

    #[test]
    fn formula_fixture_has_the_declared_public_shape() {
        assert_eq!(FORMULA_SOURCE.len(), FORMULA_SOURCE_BYTES);
        assert_eq!(FORMULA_SOURCE.matches('^').count(), FORMULA_CONSTRUCT_COUNT);
    }

    #[test]
    fn definitions_state_every_required_workload_axis() {
        let cold = Pg7Definition::new(Pg7Scenario::FormulaCold).expect("cold definition");
        assert!(
            cold.to_tsv()
                .contains("timed_operation\tcache-disabled-layout\n")
        );
        assert!(cold.to_tsv().contains("cache_state\tdisabled-cold\n"));
        assert!(
            cold.to_tsv()
                .contains("corpus_occurrence_rank\t6010-of-12020\n")
        );
        assert!(cold.to_tsv().contains(&format!(
            "median_corpus_pair_sha256\t{MEDIAN_CORPUS_PAIR_SHA256}\n"
        )));
        assert!(
            cold.to_tsv()
                .contains(&format!("fixture_relation\t{FORMULA_FIXTURE_RELATION}\n"))
        );
        assert!(
            cold.to_tsv()
                .contains("memory_front\tcleared-before-each-repetition\n")
        );
        assert!(cold.to_tsv().contains("timed_layer\tlayout\n"));
        assert_eq!(
            cold.digest().to_string(),
            "eb5a2924bb55777de30d722fd511c376208f30070829bdbff37f78a0fb93b090"
        );

        let cached = Pg7Definition::new(Pg7Scenario::FormulaCached).expect("cached definition");
        assert!(
            cached
                .to_tsv()
                .contains("cache_prime\ttex-engine-preflight\n")
        );
        assert!(cached.to_tsv().contains("cache_state\tverified-hit\n"));
        assert!(
            cached
                .to_tsv()
                .contains("cache_eviction_guard\texact-key-pin\n")
        );
        assert!(
            cached
                .to_tsv()
                .contains("memory_front\tcleared-before-each-repetition\n")
        );
        assert!(
            cached
                .to_tsv()
                .contains("timed_layer\tverified-store-hit\n")
        );
        assert_eq!(
            cached.digest().to_string(),
            "b5eea4019fb0d2a1eccb6fe9a57e40616632b3dc513ba8150368aeef21e67e35"
        );

        let text = Pg7Definition::new(Pg7Scenario::Text10kGlyph).expect("text definition");
        assert!(text.to_tsv().contains("expected_glyphs\t10000\n"));
        assert!(text.to_tsv().contains("ligatures\tfalse\n"));
        assert!(text.to_tsv().contains("cache_state\tnone\n"));
        assert!(!text.to_tsv().contains("memory_front"));
        // The text workload is unchanged by fm-a87y, so its digest is too.
        assert_eq!(
            text.digest().to_string(),
            "763bca0fd907333fe2f4fcd881e3376da684b84a5b87e13b635ddc20804e6248"
        );
    }

    #[test]
    fn canonical_outputs_match_self_goldens() {
        let engine = build_tex_engine().expect("formula engine");
        let formula = engine
            .typeset(Mode::Math(Style::Display), FORMULA_SOURCE)
            .expect("canonical formula");
        let source = text_source();
        let book = FontBook::bundled().expect("font book");
        let layout =
            layout_text(&book, &TextRequest::plain(&source)).expect("canonical text layout");
        require_text_shape(&layout).expect("canonical text shape");
        let formula_digest = typeset_digest(&formula).expect("formula digest");
        let text_digest = text_layout_digest(&layout).expect("text digest");
        assert_eq!(
            formula_digest, FORMULA_EXPECTED_RESULT_DIGEST,
            "formula result self-golden drift; text result is {text_digest}"
        );
        assert_eq!(
            text_digest, TEXT_EXPECTED_RESULT_DIGEST,
            "text result self-golden drift"
        );
    }

    #[test]
    fn cached_workload_proves_miss_preflight_hit_and_refuses_reuse() {
        use fmn_cache::{Store, StoreConfig};
        use fmn_platform::clock::FakeClock;
        use fmn_platform::fs::VirtualFs;
        use std::sync::Arc;

        let root = if cfg!(windows) { r"C:\cache" } else { "/cache" };
        let store = Store::open(
            Arc::new(VirtualFs::new()),
            Arc::new(FakeClock::new()),
            root,
            StoreConfig::default(),
        )
        .expect("virtual cache store");
        let definition = Pg7Definition::new(Pg7Scenario::FormulaCached).expect("cached definition");
        let measured =
            measure_formula_cached(&definition, &store).expect("fresh-store measurement");
        assert_eq!(measured.cache_before, "exact-key-miss");
        assert_eq!(measured.cache_after, "exact-key-hit");
        assert!(measured.cache_payload_digest.is_some());
        assert_eq!(measured.elapsed_ns.len(), PG7_SAMPLE_COUNT);
        // Every timed repetition read the store; none was a memory-front hit
        // or a silent fallback to a fresh layout.
        assert_eq!(
            measured.timed_layers,
            Some(LayerCounters {
                memory_front_hits: 0,
                layouts: 0,
                store_hits: PG7_SAMPLE_COUNT as u64,
            })
        );
        let trace = render_trace(
            &definition,
            &measured,
            &latency_samples(&measured.elapsed_ns),
        );
        assert!(
            trace.contains("timed_layer\tverified-store-hit\n"),
            "{trace}"
        );
        assert!(trace.contains("timed_memory_front_hits\t0\n"), "{trace}");
        assert!(trace.contains("timed_layouts\t0\n"), "{trace}");
        assert!(
            trace.contains(&format!("timed_store_hits\t{PG7_SAMPLE_COUNT}\n")),
            "{trace}"
        );

        let error = measure_formula_cached(&definition, &store)
            .expect_err("reused cache must not masquerade as a proved transition");
        assert!(error.to_string().contains("already present"), "{error}");
    }

    #[test]
    fn cold_workload_times_fresh_layouts_not_memory_front_hits() {
        let definition = Pg7Definition::new(Pg7Scenario::FormulaCold).expect("cold definition");
        let measured = measure_formula_cold(&definition).expect("cold measurement");
        assert_eq!(measured.elapsed_ns.len(), PG7_SAMPLE_COUNT);
        assert_eq!(
            measured.timed_layers,
            Some(LayerCounters {
                memory_front_hits: 0,
                layouts: PG7_SAMPLE_COUNT as u64,
                store_hits: 0,
            })
        );
        let trace = render_trace(
            &definition,
            &measured,
            &latency_samples(&measured.elapsed_ns),
        );
        assert!(trace.contains("timed_layer\tlayout\n"), "{trace}");
        assert!(trace.contains("timed_memory_front_hits\t0\n"), "{trace}");
        assert!(
            trace.contains(&format!("timed_layouts\t{PG7_SAMPLE_COUNT}\n")),
            "{trace}"
        );
    }

    #[test]
    fn a_timed_repetition_served_by_the_wrong_layer_is_refused() {
        // Planted negative on the real engine: a repeated request without a
        // cleared front is a memory-front hit, which is what both formula
        // workloads timed between 472a19c0 and fm-a87y.
        let engine = build_tex_engine().expect("formula engine");
        typeset_formula(&engine, FORMULA_SOURCE).expect("first layout");
        let before = LayerCounters::read(&engine);
        typeset_formula(&engine, FORMULA_SOURCE).expect("memory-front hit");
        let memory_hit = LayerCounters::read(&engine)
            .since(before)
            .expect("monotonic counters");
        assert_eq!(
            memory_hit,
            LayerCounters {
                memory_front_hits: 1,
                layouts: 0,
                store_hits: 0,
            }
        );
        for layer in [TimedLayer::Layout, TimedLayer::StoreHit] {
            let error = require_timed_layer(layer, memory_hit)
                .expect_err("a memory-front hit is never a timed layer");
            assert!(error.to_string().contains("1 memory-front hit"), "{error}");
        }

        // A store probe that silently fell back to a layout is refused too.
        let (_, _, layout) =
            formula_repetition(&engine, TimedLayer::Layout).expect("cleared-front layout");
        assert_eq!(layout, TimedLayer::Layout.one_repetition());
        let error = require_timed_layer(TimedLayer::StoreHit, layout)
            .expect_err("a layout is not a verified store hit");
        assert!(error.to_string().contains("verified-store-hit"), "{error}");
        let error = formula_repetition(&engine, TimedLayer::StoreHit)
            .expect_err("an engine without a store cannot produce store hits");
        assert!(error.to_string().contains("1 layout(s)"), "{error}");

        let backwards = LayerCounters::default().since(memory_hit);
        assert!(backwards.is_err());
    }

    #[test]
    fn latency_conversion_is_bounded_integer_math() {
        assert_eq!(latency_sample(42), Sample::valid(42));
        assert_eq!(
            latency_sample(0),
            Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds")
        );
        assert_eq!(
            latency_sample(u128::from(u64::MAX) + 1),
            Sample::invalid(u64::MAX, "latency exceeds the u64 sample range")
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
                Err(Pg7Error::Perf(PerfError::Identity(_)))
            ));
        }
    }

    #[test]
    fn unsupported_construct_diagnostic_remains_precise() {
        let engine = build_tex_engine().expect("formula engine");
        let error = typeset_formula(&engine, r"\dx")
            .expect_err("tier-2 construct must remain a named refusal");
        let detail = error.to_string();
        assert!(detail.contains("dx"), "{detail}");
        assert!(detail.contains("tier T2"), "{detail}");
    }
}
