//! The Gauntlet's scripted end-to-end scenario harness (§16.6, W10, fm-fjq).
//!
//! Everything below the unit level and above the isolated engine corpora
//! lands here: whole scenarios driven through a real user-visible surface,
//! with the run's structured log captured as a deterministic artifact and
//! any failure auto-bundled into a one-command repro.
//!
//! ## What a scenario is
//!
//! A scenario is a checked-in Rust [`ScenarioSpec`] — type-safe and
//! compile-checked, deliberately not a YAML/DSL the harness would have to
//! parse (D1: no new dependencies, no second schema to govern). A spec
//! names its [`ScenarioClass`], its [`Surface`], its [`Tier`], the
//! [`Invocation`] that drives the surface, its [`Assertion`]s over the
//! outcome, and its [`LogExpect`]ations over the captured event stream.
//!
//! ## Surfaces
//!
//! [`Surface::RustApi`] and [`Surface::CliInProcess`] are live: the former
//! drives the public `fmn-scene`/`fmn-render` lifecycle directly, the
//! latter drives `fmn_cli`'s in-process runner (never a subprocess).
//! [`Surface::PythonInProcess`] drives the feature-gated PyO3 portal adapter;
//! [`Surface::PythonPending`] remains only for unlanded Python surface rows.
//! [`Surface::StudioInProcess`] drives the production Studio composition root;
//! [`Surface::StudioSubprocess`] executes Cargo's shipping CLI artifact through
//! its isolated worker and authenticated loopback host.
//!
//! ## The log contract and `LogExpect`
//!
//! The invocation records a structured event stream into its [`RunCtx`];
//! the runner persists it as deterministic NDJSON under
//! `CARGO_TARGET_TMPDIR/e2e_logs/` (one file per scenario per run). The
//! vocabulary rides the pipeline instrumentation of fm-bgr: the canonical
//! span names ([`spans`]) mirror `PipelineStage`/the run's lifecycle
//! points, and the canonical counter names ([`counters`]) mirror
//! `PipelineStats` field-for-field (`frames_submitted` ↔ `submitted`,
//! `frames_prepared` ↔ `prepared`, and so on), so a scenario that runs
//! `FramePipeline` records its stats with no translation layer. What the
//! contract says must be assertable is assertable: preflight counts,
//! tile-cache hits, purity classifications, engine identity, and
//! `ExecutionPlan` decisions are spans with typed fields, and
//! [`LogExpect`] is the small assertion DSL over the stream —
//! `span_present` with field predicates, `counter_ge`, `event_order`,
//! `no_event`.
//!
//! ## Assertions
//!
//! The certified path asserts through the D-16 self-golden rig
//! ([`Assertion::GoldenLock`]): every artifact the outcome names is
//! bit-locked via [`crate::golden`], so a drift is the same merge blocker
//! here as in the engine corpora. Standard scenarios assert structurally
//! ([`Assertion::Structural`]): counts, size envelopes, exit codes
//! ([`Assertion::ExitCode`]), the emitted-file inventory
//! ([`Assertion::FileInventory`]), and the NDJSON schema check
//! ([`Assertion::NdjsonSchema`]) that round-trips the captured log itself.
//!
//! ## Failure → repro bundle
//!
//! On any failure — assertion, log expectation, or invocation error — the
//! runner writes a repro bundle through fmn-scene's FMNA container
//! ([`fmn_scene::journal::ReproBundle`]): the scenario's journal, its
//! content-hashed input closure, its seed and frame rate, plus the NDJSON
//! run log. The failure report names both paths, so every red scenario is
//! a deterministic replay one command away.
//!
//! ## Regression drills
//!
//! A spec marked [`ScenarioSpec::regression`] is a *drill*: the harness
//! corrupts the run deterministically — [`RegressionKind::GoldenDrift`]
//! flips one byte of the first artifact before golden locking (always in
//! check mode, so a drill can never bless a corrupted lock),
//! [`RegressionKind::LogExpectation`] corrupts the captured log against
//! the spec's own expectation — and then expects RED. The drill is
//! confirmed only when the failure came from the injection (a vacuous
//! injection — nothing dropped because the targeted span or counter never
//! occurred — is a harness error, never a confirmation), the failure
//! artifact exists and names the scenario class, and the repro bundle
//! parses with the class in its journal. Independent failures elsewhere
//! in the same run do not block confirmation but are preserved in the
//! report. Drills are the proof that the failure machinery itself works;
//! a drill that goes GREEN is a harness alarm, not a pass.
//!
//! ## Tiers
//!
//! [`Tier::Fast`] is the per-commit default: every scenario CI runs on
//! every change. [`Tier::Full`] is the slow matrix (wide geometry, high
//! thread counts, exhaustive quality presets) and runs only when
//! `FMN_E2E_FULL=1` is set — the nightly job's gate. Tier gating is a
//! skip, never a failure.
//!
//! ## Registration doctrine
//!
//! Every feature bead that changes user-visible behavior registers at
//! least one e2e scenario here. Scenarios are data, not harness edits:
//! the catalog lives in `tests/e2e_scenarios.rs` as [`ScenarioSpec`]
//! values built on this module's API, and the harness never changes when
//! a scenario is added. Keep names in the golden rig's character set
//! (`[a-z0-9._-]`), drive determinism from [`RunCtx::seed`], and prefer
//! the canonical [`spans`]/[`counters`] vocabulary over ad-hoc strings so
//! expectations stay greppable across the suite.

use crate::golden::{GoldenStore, Mode, Scope, Verdict};
use fmn_hash::{serial::Limits as SerialLimits, sha256::sha256};
use fmn_scene::journal::{
    AssetRead, CommandKind, CommandRecord, EffectClass, Entry, Journal, ReproBundle,
};
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

// Run logs are bounded evidence records, not a general-purpose JSON surface.
// These ceilings leave ample room for the scenario catalog while ensuring
// malformed artifacts cannot drive input-sized diagnostics or allocations.
const MAX_LOG_ARTIFACT_BYTES: usize = 1_048_576;
const MAX_LOG_LINE_BYTES: usize = 65_536;
const MAX_LOG_RECORDS: usize = 4_096;
const MAX_LOG_NAME_BYTES: usize = 256;
const MAX_LOG_FIELD_KEY_BYTES: usize = 128;
const MAX_LOG_STRING_BYTES: usize = 16_384;
const MAX_LOG_EVENT_FIELDS: usize = 128;
const MAX_REPRO_JOURNAL_ENTRIES: usize = 4_096;
const MAX_REPRO_CLOSURE_ASSETS: usize = 4_096;
const MAX_REPRO_PATH_BYTES: usize = 16_384;
const MAX_REPRO_STRING_BYTES: usize = 16_384;
const MAX_REPRO_ENTRY_ITEMS: usize = 4_096;
const MAX_REPRO_EFFECT_TAGS: usize = 16;
const MAX_REPRO_JOURNAL_BYTES: usize = SerialLimits::DEFAULT.max_field;
// FMNA framing (24-byte header + 32-byte checksum), entry count, and the
// empty input-event count carried by journals produced through RunCtx.
// FMNA/3 header + entry count + input-event count + render-backend count.
const EMPTY_REPRO_JOURNAL_BYTES: usize = 56 + 4 + 4 + 4;

/// Canonical span (event) names for the e2e log contract.
///
/// The pipeline-shaped names mirror fm-bgr's `PipelineStage`/lifecycle
/// vocabulary; scenarios SHOULD use these constants rather than ad-hoc
/// strings so `LogExpect`s stay greppable across the suite. The
/// `harness.*` spans are emitted by the runner itself on every run.
pub mod spans {
    /// Pre-run capability/asset preflight (counts ride as fields).
    pub const PREFLIGHT: &str = "preflight";
    /// Engine identity decision (certified vs fast CPU vs GPU).
    pub const ENGINE: &str = "engine.identity";
    /// An `ExecutionPlan` decision (team geometry, tile plan, lanes).
    pub const EXECUTION_PLAN: &str = "execution_plan";
    /// A purity classification (per segment: pure/stateful/barrier).
    pub const PURITY: &str = "purity.classification";
    /// Tile-cache activity (hits/misses ride as counters).
    pub const TILE_CACHE: &str = "tile_cache";
    /// A `PipelineStats` snapshot (fields mirror the stats struct).
    pub const PIPELINE_STATS: &str = "pipeline.stats";
    /// Scene construction completed.
    pub const SCENE_CONSTRUCT: &str = "scene.construct";
    /// One frame rasterized.
    pub const RENDER_FRAME: &str = "render.frame";
    /// One frame handed to the sink.
    pub const EMIT: &str = "emit.frame";
    /// Runner marker: the run began (carries scenario/class/surface/tier/seed).
    pub const HARNESS_BEGIN: &str = "harness.begin";
    /// Runner marker: the invocation returned.
    pub const HARNESS_INVOCATION: &str = "harness.invocation";
    /// Runner marker: one assertion/expectation was evaluated.
    pub const HARNESS_ASSERTION: &str = "harness.assertion";
    /// Runner marker: a regression injection was applied (drills only).
    pub const HARNESS_REGRESSION: &str = "harness.regression";
    /// Runner marker: the run concluded (carries the status).
    pub const HARNESS_END: &str = "harness.end";
}

/// Canonical counter names for the e2e log contract, mirroring
/// `PipelineStats` field-for-field where a pipeline is in play.
pub mod counters {
    /// Frames accepted by the pipeline (`PipelineStats::submitted`).
    pub const FRAMES_SUBMITTED: &str = "frames_submitted";
    /// Frames successfully prepared (`PipelineStats::prepared`).
    pub const FRAMES_PREPARED: &str = "frames_prepared";
    /// Frames successfully rasterized (`PipelineStats::rasterized`).
    pub const FRAMES_RASTERIZED: &str = "frames_rasterized";
    /// Frames successfully converted (`PipelineStats::converted`).
    pub const FRAMES_CONVERTED: &str = "frames_converted";
    /// Frames emitted in sequence order (`PipelineStats::emitted`).
    pub const FRAMES_EMITTED: &str = "frames_emitted";
    /// Explicit barriers executed (`PipelineStats::barriers`).
    pub const BARRIERS: &str = "barriers";
    /// Global-slot backpressure stalls (`PipelineStats::backpressure_waits`).
    pub const BACKPRESSURE_WAITS: &str = "backpressure_waits";
    /// Tile-cache hits.
    pub const TILE_CACHE_HITS: &str = "tile_cache_hits";
    /// Tile-cache misses.
    pub const TILE_CACHE_MISSES: &str = "tile_cache_misses";
    /// TeX typesetting operations performed.
    pub const TYPESETS: &str = "typesets";
}

/// The e2e scenario classes (W10's seed taxonomy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioClass {
    /// One scene across engines/geometries/quality presets.
    RenderMatrix,
    /// The same run repeated must be bit-identical.
    DeterminismDrill,
    /// A named failure must surface with its ruled error and exit code.
    FailurePath,
    /// Construct → snapshot → transform → snapshot lifecycle points.
    LifecycleDrill,
    /// Two surfaces/engines must agree on the same scene.
    ParityDrill,
}

impl ScenarioClass {
    /// The stable lowercase tag used in logs, journals, and failure output.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::RenderMatrix => "render_matrix",
            Self::DeterminismDrill => "determinism_drill",
            Self::FailurePath => "failure_path",
            Self::LifecycleDrill => "lifecycle_drill",
            Self::ParityDrill => "parity_drill",
        }
    }
}

impl fmt::Display for ScenarioClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// The user-visible surface a scenario runs through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// The public Rust API (`fmn-scene`/`fmn-render` lifecycle).
    RustApi,
    /// The CLI through `fmn_cli`'s in-process runner — never a subprocess.
    CliInProcess,
    /// The Python binding: declared, landing with its bead, never stubbed.
    PythonPending,
    /// The Python binding through its embedded-CPython Gauntlet adapter.
    PythonInProcess,
    /// Studio's production composition root in-process.
    StudioInProcess,
    /// Studio's shipping CLI, isolated worker, and real loopback HTTP host.
    StudioSubprocess,
}

impl Surface {
    /// The stable lowercase tag used in logs and failure output.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::RustApi => "rust_api",
            Self::CliInProcess => "cli_in_process",
            Self::PythonPending => "python",
            Self::PythonInProcess => "python_in_process",
            Self::StudioInProcess => "studio_in_process",
            Self::StudioSubprocess => "studio_subprocess",
        }
    }

    /// Whether this surface has landed.
    #[must_use]
    pub const fn is_pending(self) -> bool {
        matches!(self, Self::PythonPending)
    }

    /// The skip reason for a pending surface, `None` for a live one.
    #[must_use]
    pub const fn pending_reason(self) -> Option<&'static str> {
        match self {
            Self::RustApi
            | Self::CliInProcess
            | Self::PythonInProcess
            | Self::StudioInProcess
            | Self::StudioSubprocess => None,
            Self::PythonPending => {
                Some("surface pending: the Python binding lands with its bead (never stubbed)")
            }
        }
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// Which CI tier a scenario belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Per-commit CI: the default.
    Fast,
    /// The slow matrix, gated on `FMN_E2E_FULL=1` (nightly).
    Full,
}

impl Tier {
    /// The stable lowercase tag used in logs.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }

    /// Whether this tier runs given the full-matrix gate.
    #[must_use]
    pub const fn enabled(self, full_matrix: bool) -> bool {
        match self {
            Self::Fast => true,
            Self::Full => full_matrix,
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// Whether the full matrix is enabled: `FMN_E2E_FULL=1` in the environment.
#[must_use]
pub fn full_matrix_from_env() -> bool {
    matches!(std::env::var("FMN_E2E_FULL"), Ok(v) if v == "1")
}

/// A field value in the structured log: the NDJSON value subset.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// A string.
    Str(String),
    /// An unsigned integer (pipeline counters are all unsigned).
    U64(u64),
    /// A boolean.
    Bool(bool),
}

impl FieldValue {
    fn write(&self, out: &mut String) {
        match self {
            Self::Str(s) => escape_json_string(s, out),
            Self::U64(v) => out.push_str(&v.to_string()),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        }
    }
}

impl From<u64> for FieldValue {
    fn from(v: u64) -> Self {
        Self::U64(v)
    }
}

impl From<usize> for FieldValue {
    fn from(v: usize) -> Self {
        Self::U64(u64::try_from(v).unwrap_or(u64::MAX))
    }
}

impl From<bool> for FieldValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<&str> for FieldValue {
    fn from(v: &str) -> Self {
        Self::Str(v.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(v: String) -> Self {
        Self::Str(v)
    }
}

/// One structured event: a span name plus ordered typed fields.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEvent {
    name: String,
    fields: Vec<(String, FieldValue)>,
}

impl LogEvent {
    /// An event named `name` (prefer the canonical [`spans`] constants).
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            fields: Vec::new(),
        }
    }

    /// Append a field; insertion order is the NDJSON field order.
    #[must_use]
    pub fn field(mut self, key: &str, value: impl Into<FieldValue>) -> Self {
        self.fields.push((key.to_string(), value.into()));
        self
    }
}

/// A predicate over one event's fields, for [`LogExpect::SpanPresent`].
///
/// Keys and expected strings are `&'static str`: predicates are part of
/// the checked-in scenario data, never runtime-built.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldPred {
    /// The field exists (any value).
    Exists(&'static str),
    /// The field is a string equal to the given value.
    StrEq(&'static str, &'static str),
    /// The field is an unsigned integer at least the given bound.
    U64Ge(&'static str, u64),
    /// The field is an unsigned integer equal to the given value.
    U64Eq(&'static str, u64),
    /// The field is a boolean equal to the given value.
    BoolEq(&'static str, bool),
}

impl FieldPred {
    /// [`FieldPred::Exists`].
    #[must_use]
    pub const fn exists(key: &'static str) -> Self {
        Self::Exists(key)
    }

    /// [`FieldPred::StrEq`].
    #[must_use]
    pub const fn str_eq(key: &'static str, value: &'static str) -> Self {
        Self::StrEq(key, value)
    }

    /// [`FieldPred::U64Ge`].
    #[must_use]
    pub const fn u64_ge(key: &'static str, bound: u64) -> Self {
        Self::U64Ge(key, bound)
    }

    /// [`FieldPred::U64Eq`].
    #[must_use]
    pub const fn u64_eq(key: &'static str, value: u64) -> Self {
        Self::U64Eq(key, value)
    }

    /// [`FieldPred::BoolEq`].
    #[must_use]
    pub const fn bool_eq(key: &'static str, value: bool) -> Self {
        Self::BoolEq(key, value)
    }

    /// Whether the predicate holds against one event's fields.
    #[must_use]
    pub fn check(&self, fields: &[(String, FieldValue)]) -> bool {
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k.as_str() == key)
                .map(|(_, v)| v)
        };
        match self {
            Self::Exists(k) => get(k).is_some(),
            Self::StrEq(k, v) => matches!(get(k), Some(FieldValue::Str(have)) if have == v),
            Self::U64Ge(k, bound) => matches!(get(k), Some(FieldValue::U64(have)) if have >= bound),
            Self::U64Eq(k, v) => matches!(get(k), Some(FieldValue::U64(have)) if have == v),
            Self::BoolEq(k, v) => matches!(get(k), Some(FieldValue::Bool(have)) if have == v),
        }
    }
}

/// One record in the captured stream: an event or a counter sample.
/// Sequence numbers are assigned by the runner, gapless from zero.
#[derive(Debug, Clone, PartialEq)]
pub enum LogRecord {
    /// A structured event (a span occurrence).
    Event {
        /// Position in the unified stream.
        seq: u64,
        /// The span name.
        name: String,
        /// Ordered typed fields.
        fields: Vec<(String, FieldValue)>,
    },
    /// A counter sample.
    Counter {
        /// Position in the unified stream.
        seq: u64,
        /// The counter name.
        name: String,
        /// The sampled value.
        value: u64,
    },
}

impl LogRecord {
    const fn seq(&self) -> u64 {
        match self {
            Self::Event { seq, .. } | Self::Counter { seq, .. } => *seq,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Event { name, .. } | Self::Counter { name, .. } => name,
        }
    }

    fn validate_limits(&self) -> Result<(), String> {
        if self.name().len() > MAX_LOG_NAME_BYTES {
            return Err(format!(
                "record name exceeds the {MAX_LOG_NAME_BYTES}-byte limit"
            ));
        }
        let Self::Event { fields, .. } = self else {
            return Ok(());
        };
        if fields.len() > MAX_LOG_EVENT_FIELDS {
            return Err(format!("event has more than {MAX_LOG_EVENT_FIELDS} fields"));
        }
        for (index, (key, value)) in fields.iter().enumerate() {
            if key.len() > MAX_LOG_FIELD_KEY_BYTES {
                return Err(format!(
                    "field {} key exceeds the {MAX_LOG_FIELD_KEY_BYTES}-byte limit",
                    index + 1
                ));
            }
            if fields
                .iter()
                .take(index)
                .any(|(previous, _)| previous == key)
            {
                return Err(format!("duplicate event field key at field {}", index + 1));
            }
            if let FieldValue::Str(value) = value
                && value.len() > MAX_LOG_STRING_BYTES
            {
                return Err(format!(
                    "field {} string exceeds the {MAX_LOG_STRING_BYTES}-byte limit",
                    index + 1
                ));
            }
        }
        Ok(())
    }

    /// Serialize as one NDJSON line: `{"seq":N,"kind":"event",...}`.
    /// Key order is defined (the schema is order-defined, like every other
    /// canonical form in the project); the parser relies on it.
    fn to_line(&self) -> String {
        let mut out = String::new();
        match self {
            Self::Event { seq, name, fields } => {
                out.push_str("{\"seq\":");
                out.push_str(&seq.to_string());
                out.push_str(",\"kind\":\"event\",\"name\":");
                escape_json_string(name, &mut out);
                out.push_str(",\"fields\":{");
                for (index, (key, value)) in fields.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    escape_json_string(key, &mut out);
                    out.push(':');
                    value.write(&mut out);
                }
                out.push_str("}}");
            }
            Self::Counter { seq, name, value } => {
                out.push_str("{\"seq\":");
                out.push_str(&seq.to_string());
                out.push_str(",\"kind\":\"counter\",\"name\":");
                escape_json_string(name, &mut out);
                out.push_str(",\"value\":");
                out.push_str(&value.to_string());
                out.push('}');
            }
        }
        out
    }

    /// Parse one NDJSON line back into a record (the inverse of
    /// [`LogRecord::to_line`], strict about the defined key order).
    fn parse(line: &str) -> Result<Self, String> {
        if line.len() > MAX_LOG_LINE_BYTES {
            return Err(format!(
                "record exceeds the {MAX_LOG_LINE_BYTES}-byte line limit"
            ));
        }
        let mut p = LineParser::new(line);
        p.expect(b'{')?;
        p.key("seq")?;
        let seq = p.uint()?;
        p.expect(b',')?;
        p.key("kind")?;
        let kind = p.string(16, "record kind")?;
        p.expect(b',')?;
        p.key("name")?;
        let name = p.string(MAX_LOG_NAME_BYTES, "record name")?;
        let record = match kind.as_str() {
            "event" => {
                p.expect(b',')?;
                p.key("fields")?;
                p.expect(b'{')?;
                let mut fields = Vec::new();
                loop {
                    match p.bytes.get(p.pos) {
                        Some(b'}') => {
                            p.pos += 1;
                            break;
                        }
                        Some(_) => {
                            if fields.len() == MAX_LOG_EVENT_FIELDS {
                                return Err(format!(
                                    "event has more than {MAX_LOG_EVENT_FIELDS} fields"
                                ));
                            }
                            let key = p.string(MAX_LOG_FIELD_KEY_BYTES, "field key")?;
                            if fields
                                .iter()
                                .any(|(previous, _): &(String, FieldValue)| previous == &key)
                            {
                                return Err("duplicate event field key".to_string());
                            }
                            p.expect(b':')?;
                            let value = p.value()?;
                            fields.push((key, value));
                            match p.bytes.get(p.pos) {
                                Some(b',') => p.pos += 1,
                                Some(b'}') => {
                                    p.pos += 1;
                                    break;
                                }
                                other => {
                                    return Err(format!(
                                        "expected ',' or '}}' in fields, found {other:?}"
                                    ));
                                }
                            }
                        }
                        None => return Err("unterminated fields object".to_string()),
                    }
                }
                Self::Event { seq, name, fields }
            }
            "counter" => {
                p.expect(b',')?;
                p.key("value")?;
                let value = p.uint()?;
                Self::Counter { seq, name, value }
            }
            other => return Err(format!("unknown record kind {other:?}")),
        };
        p.expect(b'}')?;
        if !p.at_end() {
            return Err("trailing bytes after record".to_string());
        }
        record.validate_limits()?;
        if record.to_line() != line {
            return Err("record is not in canonical NDJSON form".to_string());
        }
        Ok(record)
    }
}

/// Escape a string into its JSON form (quotes included) per RFC 8259.
pub(crate) fn escape_json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A minimal parser for the NDJSON line schema this module
/// emits — strict about the defined key order and refusing anything the
/// serializer cannot produce.
struct LineParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> LineParser<'a> {
    fn new(line: &'a str) -> Self {
        Self {
            bytes: line.as_bytes(),
            pos: 0,
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        // ubs:ignore — compares public JSON syntax, not secret material
        if self.bytes.get(self.pos) == Some(&byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected {:?} at byte {}", byte as char, self.pos))
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn string(&mut self, max_bytes: usize, label: &str) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = Vec::with_capacity(self.bytes.len().saturating_sub(self.pos).min(max_bytes));
        loop {
            let Some(&b) = self.bytes.get(self.pos) else {
                return Err("unterminated string".to_string());
            };
            self.pos += 1;
            match b {
                b'"' => {
                    return String::from_utf8(out)
                        .map_err(|_| "invalid utf-8 in string".to_string());
                }
                b'\\' => {
                    let Some(&esc) = self.bytes.get(self.pos) else {
                        return Err("unterminated escape".to_string());
                    };
                    self.pos += 1;
                    match esc {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let code = self.hex4()?;
                            let Some(ch) = char::from_u32(code) else {
                                return Err(format!("invalid unicode escape {code:#06x}"));
                            };
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err(format!("invalid escape \\{}", esc as char)),
                    }
                }
                0x00..=0x1f => {
                    return Err("unescaped control byte in string".to_string());
                }
                _ => out.push(b),
            }
            if out.len() > max_bytes {
                return Err(format!("{label} exceeds the {max_bytes}-byte limit"));
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let mut code = 0u32;
        for _ in 0..4 {
            let Some(&b) = self.bytes.get(self.pos) else {
                return Err("truncated \\u escape".to_string());
            };
            self.pos += 1;
            let digit = (b as char)
                .to_digit(16)
                .ok_or_else(|| format!("invalid hex digit {:?}", b as char))?;
            code = code * 16 + digit;
        }
        Ok(code)
    }

    fn key(&mut self, expected: &str) -> Result<(), String> {
        let found = self.string(MAX_LOG_FIELD_KEY_BYTES, "object key")?;
        if found != expected {
            return Err(format!("expected key {expected:?}"));
        }
        self.expect(b':')
    }

    fn uint(&mut self) -> Result<u64, String> {
        let start = self.pos;
        while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(format!("expected unsigned integer at byte {start}"));
        }
        let digit_count = self.pos - start;
        if digit_count > 20 {
            return Err("unsigned integer exceeds 20 digits".to_string());
        }
        // ubs:ignore — checks public decimal syntax, not secret material
        if digit_count > 1 && self.bytes.get(start) == Some(&b'0') {
            return Err("unsigned integer has a leading zero".to_string());
        }
        let digit_bytes = self
            .bytes
            .get(start..self.pos)
            .ok_or_else(|| "invalid integer range".to_string())?;
        let digits =
            std::str::from_utf8(digit_bytes).map_err(|_| "invalid integer bytes".to_string())?;
        digits
            .parse::<u64>()
            .map_err(|_| "unsigned integer is out of range".to_string())
    }

    fn value(&mut self) -> Result<FieldValue, String> {
        match self.bytes.get(self.pos) {
            Some(b'"') => Ok(FieldValue::Str(
                self.string(MAX_LOG_STRING_BYTES, "field string")?,
            )),
            Some(b't') => {
                self.literal("true")?;
                Ok(FieldValue::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(FieldValue::Bool(false))
            }
            Some(b'0'..=b'9') => Ok(FieldValue::U64(self.uint()?)),
            other => Err(format!("unexpected value start {other:?}")),
        }
    }

    fn literal(&mut self, lit: &str) -> Result<(), String> {
        let tail = self
            .bytes
            .get(self.pos..)
            .ok_or_else(|| "invalid literal range".to_string())?;
        if tail.starts_with(lit.as_bytes()) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err(format!("expected literal {lit:?} at byte {}", self.pos))
        }
    }
}

/// An expectation over the captured event stream — the log-assertion DSL.
///
/// Positions for [`LogExpect::EventOrder`] are indices into the unified
/// record stream (events and counter samples share one sequence).
#[derive(Debug, Clone, PartialEq)]
pub enum LogExpect {
    /// Some event named `span` occurs with all field predicates satisfied.
    SpanPresent {
        /// The span name (prefer the canonical [`spans`] constants).
        span: &'static str,
        /// Predicates every satisfying occurrence must pass.
        fields: Vec<FieldPred>,
    },
    /// The peak recorded value of counter `counter` is at least `bound`.
    /// A counter that was never recorded fails.
    CounterGe {
        /// The counter name (prefer the canonical [`counters`] constants).
        counter: &'static str,
        /// The inclusive lower bound on the peak sample.
        bound: u64,
    },
    /// The first occurrence of `before` precedes the first occurrence of
    /// `after`; both must occur (vacuous order is a failure, not a pass).
    EventOrder {
        /// The event that must come first.
        before: &'static str,
        /// The event that must come later.
        after: &'static str,
    },
    /// No event or counter with this name occurs anywhere in the stream.
    NoEvent(&'static str),
}

impl LogExpect {
    /// [`LogExpect::SpanPresent`].
    #[must_use]
    pub const fn span_present(span: &'static str, fields: Vec<FieldPred>) -> Self {
        Self::SpanPresent { span, fields }
    }

    /// [`LogExpect::CounterGe`].
    #[must_use]
    pub const fn counter_ge(counter: &'static str, bound: u64) -> Self {
        Self::CounterGe { counter, bound }
    }

    /// [`LogExpect::EventOrder`].
    #[must_use]
    pub const fn event_order(before: &'static str, after: &'static str) -> Self {
        Self::EventOrder { before, after }
    }

    /// [`LogExpect::NoEvent`].
    #[must_use]
    pub const fn no_event(name: &'static str) -> Self {
        Self::NoEvent(name)
    }

    /// One-line human description for logs and failure messages.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::SpanPresent { span, fields } => {
                format!("span_present({span:?}, {} predicates)", fields.len())
            }
            Self::CounterGe { counter, bound } => format!("counter_ge({counter:?}, {bound})"),
            Self::EventOrder { before, after } => {
                format!("event_order({before:?} before {after:?})")
            }
            Self::NoEvent(name) => format!("no_event({name:?})"),
        }
    }

    /// Evaluate against the captured record stream.
    pub fn check(&self, log: &[LogRecord]) -> Result<(), String> {
        match self {
            Self::SpanPresent { span, fields } => {
                let mut named = 0usize;
                for record in log {
                    if let LogRecord::Event {
                        name, fields: have, ..
                    } = record
                        && name.as_str() == *span
                    {
                        named += 1;
                        if fields.iter().all(|pred| pred.check(have)) {
                            return Ok(());
                        }
                    }
                }
                if named > 0 {
                    Err(format!(
                        "span {span:?} occurred {named} time(s) but no occurrence satisfied \
                         the field predicates"
                    ))
                } else {
                    Err(format!("span {span:?} never occurred"))
                }
            }
            Self::CounterGe { counter, bound } => {
                let mut peak: Option<u64> = None;
                for record in log {
                    if let LogRecord::Counter { name, value, .. } = record
                        && name.as_str() == *counter
                    {
                        peak = Some(peak.map_or(*value, |p| p.max(*value)));
                    }
                }
                match peak {
                    Some(v) if v >= *bound => Ok(()),
                    Some(v) => Err(format!(
                        "counter {counter:?} peaked at {v}, below bound {bound}"
                    )),
                    None => Err(format!("counter {counter:?} was never recorded")),
                }
            }
            Self::EventOrder { before, after } => {
                let first = |want: &str| {
                    log.iter().position(
                        |r| matches!(r, LogRecord::Event { name, .. } if name.as_str() == want),
                    )
                };
                match (first(before), first(after)) {
                    (Some(b), Some(a)) if b < a => Ok(()),
                    (Some(b), Some(a)) => Err(format!(
                        "event {before:?} first occurs at stream index {b}, not before \
                         {after:?} at index {a}"
                    )),
                    (None, _) => Err(format!("event {before:?} never occurred")),
                    (_, None) => Err(format!("event {after:?} never occurred")),
                }
            }
            Self::NoEvent(name) => {
                if log.iter().any(|r| r.name() == *name) {
                    Err(format!("forbidden event {name:?} occurred"))
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Why a scenario invocation could not complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError(String);

impl ScenarioError {
    /// A failure with a human-readable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScenarioError {}

impl From<String> for ScenarioError {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl From<&str> for ScenarioError {
    fn from(message: &str) -> Self {
        Self(message.to_string())
    }
}

/// One emitted file: name plus bytes. Artifact names feed golden locks
/// and the file inventory, so they use the golden rig's character set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// The artifact name (`[a-z0-9._-]`, as the golden rig requires).
    pub name: String,
    /// The exact emitted bytes.
    pub bytes: Vec<u8>,
}

/// What a scenario invocation produced: exit code, emitted files, and
/// structural counters (last write wins per name). This is the channel
/// [`Assertion`]s evaluate; the [`RunCtx`] event stream is the channel
/// [`LogExpect`]s evaluate.
#[derive(Debug, Default)]
pub struct RunOutcome {
    /// The surface's exit code (0 = success; CLI scenarios use the
    /// schema-owned codes `fmn_cli::RunOutput::code` reports).
    pub exit_code: i32,
    /// Every file the run emitted, named.
    pub artifacts: Vec<Artifact>,
    /// Structural counters (frame counts, typesets, sizes).
    pub counters: Vec<(String, u64)>,
}

impl RunOutcome {
    /// A successful outcome with no artifacts or counters.
    #[must_use]
    pub fn ok() -> Self {
        Self::default()
    }

    /// Append an emitted artifact.
    #[must_use]
    pub fn with_artifact(mut self, name: &str, bytes: Vec<u8>) -> Self {
        self.artifacts.push(Artifact {
            name: name.to_string(),
            bytes,
        });
        self
    }

    /// Record a structural counter (last write wins per name).
    #[must_use]
    pub fn with_counter(mut self, name: &str, value: u64) -> Self {
        self.counters.push((name.to_string(), value));
        self
    }

    /// Set the exit code.
    #[must_use]
    pub const fn exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    /// The last-recorded value of a structural counter.
    #[must_use]
    pub fn counter(&self, name: &str) -> Option<u64> {
        self.counters
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| *v)
    }
}

/// The scenario-body signature, factored out of [`Invocation`].
type ScenarioBody = Box<dyn FnOnce(&mut RunCtx) -> Result<RunOutcome, ScenarioError> + Send>;

/// The scenario body: a closure that drives the surface and reports what
/// it produced. The [`RunCtx`] receives the structured event stream, the
/// journal entries, and the input closure as the run progresses.
pub struct Invocation(ScenarioBody);

impl Invocation {
    /// Wrap the scenario body.
    #[must_use]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut RunCtx) -> Result<RunOutcome, ScenarioError> + Send + 'static,
    {
        Self(Box::new(f))
    }

    fn invoke(self, ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
        (self.0)(ctx)
    }
}

impl fmt::Debug for Invocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invocation(..)")
    }
}

#[derive(Debug)]
enum RecordingError {
    LogRecordLimit { limit: usize },
    InvalidLogRecord(String),
    LogStorageAllocationFailed,
    JournalEntryLimit { limit: usize },
    JournalByteLimit { limit: usize, needed: usize },
    InvalidJournalEntry(String),
    JournalStorageAllocationFailed,
    ClosureAssetLimit { limit: usize },
    InvalidClosureAsset(String),
    ClosureStorageAllocationFailed,
}

impl fmt::Display for RecordingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LogRecordLimit { limit } => {
                write!(f, "run log reached the {limit}-record retention limit")
            }
            Self::InvalidLogRecord(detail) => {
                write!(f, "run log record was invalid before retention: {detail}")
            }
            Self::LogStorageAllocationFailed => {
                f.write_str("run log record storage allocation failed")
            }
            Self::JournalEntryLimit { limit } => {
                write!(f, "repro journal reached the {limit}-entry retention limit")
            }
            Self::JournalByteLimit { limit, needed } => write!(
                f,
                "repro journal needs {needed} canonical bytes, over the {limit}-byte limit"
            ),
            Self::InvalidJournalEntry(detail) => {
                write!(
                    f,
                    "repro journal entry was invalid before retention: {detail}"
                )
            }
            Self::JournalStorageAllocationFailed => {
                f.write_str("repro journal entry storage allocation failed")
            }
            Self::ClosureAssetLimit { limit } => write!(
                f,
                "repro input closure reached the {limit}-asset retention limit"
            ),
            Self::InvalidClosureAsset(detail) => {
                write!(
                    f,
                    "repro input-closure asset was invalid before retention: {detail}"
                )
            }
            Self::ClosureStorageAllocationFailed => {
                f.write_str("repro input-closure storage allocation failed")
            }
        }
    }
}

fn add_repro_wire_bytes(
    total: &mut usize,
    additional: usize,
    context: &'static str,
) -> Result<(), String> {
    *total = total
        .checked_add(additional)
        .ok_or_else(|| format!("{context} canonical byte count overflowed usize"))?;
    Ok(())
}

fn check_repro_string(field: &'static str, value: &str, limit: usize) -> Result<(), String> {
    if value.len() > limit {
        Err(format!(
            "{field} is {} bytes, over the {limit}-byte limit",
            value.len()
        ))
    } else {
        Ok(())
    }
}

fn journal_entry_wire_bytes(entry: &Entry) -> Result<usize, String> {
    check_repro_string(
        "command label",
        &entry.command.label,
        MAX_REPRO_STRING_BYTES,
    )?;

    // command kind + identity + label length prefix
    let mut bytes = 1 + 32 + 8;
    add_repro_wire_bytes(&mut bytes, entry.command.label.len(), "journal entry")?;

    // Custom commands are canonically coerced to Opaque by Journal admission,
    // so their claimed effect payload never reaches the wire.
    add_repro_wire_bytes(&mut bytes, 1, "journal effect")?;
    if entry.command.kind != CommandKind::Custom
        && let EffectClass::Stateful(tags) = &entry.effect
    {
        if tags.len() > MAX_REPRO_EFFECT_TAGS {
            return Err(format!(
                "stateful effect has {} tags, over the {MAX_REPRO_EFFECT_TAGS}-tag limit",
                tags.len()
            ));
        }
        add_repro_wire_bytes(&mut bytes, 4, "journal effect count")?;
        add_repro_wire_bytes(&mut bytes, tags.len(), "journal effect tags")?;
    }

    if entry.reads.len() > MAX_REPRO_ENTRY_ITEMS {
        return Err(format!(
            "entry has {} asset reads, over the {MAX_REPRO_ENTRY_ITEMS}-read limit",
            entry.reads.len()
        ));
    }
    add_repro_wire_bytes(&mut bytes, 4, "journal asset-read count")?;
    for read in &entry.reads {
        check_repro_string("asset-read path", &read.path, MAX_REPRO_PATH_BYTES)?;
        // path length prefix + path + digest
        add_repro_wire_bytes(&mut bytes, 8, "journal asset-read path length")?;
        add_repro_wire_bytes(&mut bytes, read.path.len(), "journal asset-read path")?;
        add_repro_wire_bytes(&mut bytes, 32, "journal asset-read digest")?;
    }

    if entry.subprocesses.len() > MAX_REPRO_ENTRY_ITEMS {
        return Err(format!(
            "entry has {} subprocess records, over the {MAX_REPRO_ENTRY_ITEMS}-record limit",
            entry.subprocesses.len()
        ));
    }
    add_repro_wire_bytes(&mut bytes, 4, "journal subprocess count")?;
    for subprocess in &entry.subprocesses {
        check_repro_string(
            "subprocess tool digest",
            &subprocess.tool_sha256_hex,
            MAX_REPRO_STRING_BYTES,
        )?;
        check_repro_string(
            "subprocess destination",
            &subprocess.destination,
            MAX_REPRO_PATH_BYTES,
        )?;
        // two string prefixes + strings + argv digest
        add_repro_wire_bytes(&mut bytes, 8, "subprocess tool digest length")?;
        add_repro_wire_bytes(
            &mut bytes,
            subprocess.tool_sha256_hex.len(),
            "subprocess tool digest",
        )?;
        add_repro_wire_bytes(&mut bytes, 32, "subprocess argv digest")?;
        add_repro_wire_bytes(&mut bytes, 8, "subprocess destination length")?;
        add_repro_wire_bytes(
            &mut bytes,
            subprocess.destination.len(),
            "subprocess destination",
        )?;
    }

    add_repro_wire_bytes(&mut bytes, 1, "journal checkpoint presence")?;
    if let Some(checkpoint) = &entry.checkpoint {
        if checkpoint.len() > SerialLimits::DEFAULT.max_field {
            return Err(format!(
                "checkpoint is {} bytes, over the {}-byte field limit",
                checkpoint.len(),
                SerialLimits::DEFAULT.max_field
            ));
        }
        add_repro_wire_bytes(&mut bytes, 8, "journal checkpoint length")?;
        add_repro_wire_bytes(&mut bytes, checkpoint.len(), "journal checkpoint")?;
    }
    add_repro_wire_bytes(&mut bytes, 32, "journal state digest")?;
    Ok(bytes)
}

/// The per-run context handed to the invocation: the deterministic seed,
/// the structured log, the repro journal, and the input closure.
///
/// Determinism doctrine: all randomness derives from [`RunCtx::seed`]
/// (itself derived from the scenario name), never from a wall clock or
/// the environment.
pub struct RunCtx {
    /// The scenario's deterministic seed ([`scenario_seed`]).
    pub seed: u64,
    fps: (u32, u32),
    log: Vec<LogRecord>,
    recording_error: Option<RecordingError>,
    journal: Journal,
    journal_bytes: usize,
    closure: Vec<AssetRead>,
}

impl RunCtx {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            fps: (60, 1),
            log: Vec::new(),
            recording_error: None,
            journal: Journal::new(),
            journal_bytes: EMPTY_REPRO_JOURNAL_BYTES,
            closure: Vec::new(),
        }
    }

    fn next_seq(&self) -> u64 {
        u64::try_from(self.log.len()).unwrap_or(u64::MAX)
    }

    fn record(&mut self, record: LogRecord) {
        if self.recording_error.is_some() {
            return;
        }
        if self.log.len() >= MAX_LOG_RECORDS {
            self.recording_error = Some(RecordingError::LogRecordLimit {
                limit: MAX_LOG_RECORDS,
            });
            return;
        }
        if let Err(detail) = record.validate_limits() {
            self.recording_error = Some(RecordingError::InvalidLogRecord(detail));
            return;
        }
        if self.log.try_reserve(1).is_err() {
            self.recording_error = Some(RecordingError::LogStorageAllocationFailed);
            return;
        }
        self.log.push(record);
    }

    fn recording_error(&self) -> Option<&RecordingError> {
        self.recording_error.as_ref()
    }

    /// The rational frame rate the repro bundle records (default 60/1).
    pub fn set_fps(&mut self, fps: (u32, u32)) {
        self.fps = fps;
    }

    /// The recorded frame rate.
    #[must_use]
    pub const fn fps(&self) -> (u32, u32) {
        self.fps
    }

    /// Append a structured event to the run log. An invalid event or a
    /// record beyond the retention ceiling latches a harness error and stops
    /// subsequent log retention.
    pub fn event(&mut self, event: LogEvent) {
        let seq = self.next_seq();
        self.record(LogRecord::Event {
            seq,
            name: event.name,
            fields: event.fields,
        });
    }

    /// Record a counter sample in the run log (peak semantics for
    /// [`LogExpect::CounterGe`]: record progressive values freely). An
    /// invalid or over-limit record follows [`Self::event`]'s fail-closed
    /// behavior.
    pub fn counter(&mut self, name: &str, value: u64) {
        let seq = self.next_seq();
        self.record(LogRecord::Counter {
            seq,
            name: name.to_string(),
            value,
        });
    }

    /// Append a journal entry that will ship in the repro bundle. Invalid,
    /// over-limit, or unreservable entries latch a harness error before
    /// retention and stop subsequent evidence recording.
    pub fn record_journal(&mut self, entry: Entry) {
        if self.recording_error.is_some() {
            return;
        }
        if self.journal.entries().len() >= MAX_REPRO_JOURNAL_ENTRIES {
            self.recording_error = Some(RecordingError::JournalEntryLimit {
                limit: MAX_REPRO_JOURNAL_ENTRIES,
            });
            return;
        }
        let entry_bytes = match journal_entry_wire_bytes(&entry) {
            Ok(bytes) => bytes,
            Err(detail) => {
                self.recording_error = Some(RecordingError::InvalidJournalEntry(detail));
                return;
            }
        };
        let needed = match self.journal_bytes.checked_add(entry_bytes) {
            Some(needed) => needed,
            None => {
                self.recording_error = Some(RecordingError::InvalidJournalEntry(
                    "canonical byte count overflowed usize".to_string(),
                ));
                return;
            }
        };
        if needed > MAX_REPRO_JOURNAL_BYTES {
            self.recording_error = Some(RecordingError::JournalByteLimit {
                limit: MAX_REPRO_JOURNAL_BYTES,
                needed,
            });
            return;
        }
        if self.journal.record(entry).is_err() {
            self.recording_error = Some(RecordingError::JournalStorageAllocationFailed);
            return;
        }
        self.journal_bytes = needed;
    }

    /// Record an input-closure read: the path as addressed and the
    /// content hash of the bytes that were read. Invalid, over-limit, or
    /// unreservable rows latch a harness error before retention and stop
    /// subsequent evidence recording.
    pub fn record_asset(&mut self, path: &str, bytes: &[u8]) {
        if self.recording_error.is_some() {
            return;
        }
        if self.closure.len() >= MAX_REPRO_CLOSURE_ASSETS {
            self.recording_error = Some(RecordingError::ClosureAssetLimit {
                limit: MAX_REPRO_CLOSURE_ASSETS,
            });
            return;
        }
        if let Err(detail) = check_repro_string("asset path", path, MAX_REPRO_PATH_BYTES) {
            self.recording_error = Some(RecordingError::InvalidClosureAsset(detail));
            return;
        }
        let mut owned_path = String::new();
        if owned_path.try_reserve_exact(path.len()).is_err() || self.closure.try_reserve(1).is_err()
        {
            self.recording_error = Some(RecordingError::ClosureStorageAllocationFailed);
            return;
        }
        owned_path.push_str(path);
        self.closure.push(AssetRead {
            path: owned_path,
            digest: sha256(bytes),
        });
    }

    /// The captured record stream (events and counters in sequence).
    #[must_use]
    pub fn records(&self) -> &[LogRecord] {
        &self.log
    }

    /// The journal accumulated so far.
    #[must_use]
    pub const fn journal(&self) -> &Journal {
        &self.journal
    }

    /// The recorded input closure.
    #[must_use]
    pub fn closure(&self) -> &[AssetRead] {
        &self.closure
    }

    /// Renumber the stream after a drill's log corruption so the NDJSON
    /// sequence stays gapless.
    fn renumber(&mut self) {
        for (index, record) in self.log.iter_mut().enumerate() {
            let seq = u64::try_from(index).unwrap_or(u64::MAX);
            match record {
                LogRecord::Event { seq: s, .. } | LogRecord::Counter { seq: s, .. } => *s = seq,
            }
        }
    }
}

/// A structural assertion over the [`RunOutcome`]: counts, envelopes,
/// exit-adjacent facts that do not need the golden rig.
#[derive(Debug, Clone, PartialEq)]
pub enum StructuralAssert {
    /// Exactly this many artifacts were emitted.
    ArtifactCountEq(usize),
    /// At least this many artifacts were emitted.
    ArtifactCountGe(usize),
    /// Every emitted artifact is at most this many bytes (size envelope).
    ArtifactBytesLe(u64),
    /// No emitted artifact is empty.
    NoEmptyArtifacts,
    /// A structural counter equals exactly.
    CounterEq(&'static str, u64),
    /// A structural counter is at least this bound.
    CounterGe(&'static str, u64),
}

impl StructuralAssert {
    /// One-line human description for logs and failure messages.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::ArtifactCountEq(n) => format!("artifact-count == {n}"),
            Self::ArtifactCountGe(n) => format!("artifact-count >= {n}"),
            Self::ArtifactBytesLe(b) => format!("artifact-bytes <= {b}"),
            Self::NoEmptyArtifacts => "no-empty-artifacts".to_string(),
            Self::CounterEq(counter, value) => format!("counter {counter:?} == {value}"),
            Self::CounterGe(counter, bound) => format!("counter {counter:?} >= {bound}"),
        }
    }

    fn check(&self, outcome: &RunOutcome) -> Result<(), String> {
        match self {
            Self::ArtifactCountEq(n) => {
                if outcome.artifacts.len() == *n {
                    Ok(())
                } else {
                    Err(format!(
                        "{} artifacts emitted, expected exactly {n}",
                        outcome.artifacts.len()
                    ))
                }
            }
            Self::ArtifactCountGe(n) => {
                if outcome.artifacts.len() >= *n {
                    Ok(())
                } else {
                    Err(format!(
                        "{} artifacts emitted, expected at least {n}",
                        outcome.artifacts.len()
                    ))
                }
            }
            Self::ArtifactBytesLe(bound) => {
                match outcome
                    .artifacts
                    .iter()
                    .find(|a| a.bytes.len() as u64 > *bound)
                {
                    None => Ok(()),
                    Some(a) => Err(format!(
                        "artifact {:?} is {} bytes, above envelope {bound}",
                        a.name,
                        a.bytes.len()
                    )),
                }
            }
            Self::NoEmptyArtifacts => match outcome.artifacts.iter().find(|a| a.bytes.is_empty()) {
                None => Ok(()),
                Some(a) => Err(format!("artifact {:?} is empty", a.name)),
            },
            Self::CounterEq(counter, value) => match outcome.counter(counter) {
                Some(have) if have == *value => Ok(()),
                Some(have) => Err(format!("counter {counter:?} is {have}, expected {value}")),
                None => Err(format!("counter {counter:?} was never recorded")),
            },
            Self::CounterGe(counter, bound) => match outcome.counter(counter) {
                Some(have) if have >= *bound => Ok(()),
                Some(have) => Err(format!(
                    "counter {counter:?} is {have}, below bound {bound}"
                )),
                None => Err(format!("counter {counter:?} was never recorded")),
            },
        }
    }
}

/// One assertion over a scenario's outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum Assertion {
    /// Bit-lock every emitted artifact through the D-16 self-golden rig
    /// (the certified path; check/bless follows the runner's golden mode).
    GoldenLock {
        /// The lock-file family under the runner's goldens directory.
        suite: &'static str,
        /// Per-platform or whole-matrix locking (see [`crate::golden`]).
        scope: Scope,
    },
    /// A structural predicate over the outcome.
    Structural(StructuralAssert),
    /// The surface's exit code equals exactly.
    ExitCode(i32),
    /// The exact set of emitted artifact names (order-insensitive).
    FileInventory(Vec<String>),
    /// The captured log serializes to NDJSON and parses back
    /// line-for-line identically, with gapless sequence numbers.
    NdjsonSchema,
}

impl Assertion {
    /// One-line human description for logs and failure messages.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::GoldenLock { suite, scope } => {
                let scope = match scope {
                    Scope::PerPlatform => "per-platform",
                    Scope::Certified => "certified",
                };
                format!("golden-lock suite={suite:?} scope={scope}")
            }
            Self::Structural(s) => format!("structural {}", s.describe()),
            Self::ExitCode(code) => format!("exit-code {code}"),
            Self::FileInventory(names) => format!("file-inventory [{}]", names.join(", ")),
            Self::NdjsonSchema => "ndjson-schema".to_string(),
        }
    }
}

/// A deterministic corruption the harness injects to prove the failure
/// machinery (the regression drills).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegressionKind {
    /// Flip one byte of the first emitted artifact before golden locking:
    /// the D-16 rig must go RED. Golden evaluation is forced to check mode
    /// for the drill, so a corrupted lock can never be blessed.
    GoldenDrift,
    /// Corrupt the captured log against the spec's own expectation (drop
    /// the expected span's events, erase the expected counter's samples,
    /// or emit the forbidden event): the [`LogExpect`] must go RED.
    LogExpectation,
}

impl RegressionKind {
    /// The stable lowercase tag used in logs and failure output.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::GoldenDrift => "golden_drift",
            Self::LogExpectation => "log_expectation",
        }
    }
}

impl fmt::Display for RegressionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}

/// A checked-in end-to-end scenario: name, class, surface, invocation,
/// assertions, log expectations, and an optional regression injection.
///
/// Construct with [`ScenarioSpec::new`] and the `with_*` combinators, or
/// as a plain struct literal — both are data.
pub struct ScenarioSpec {
    /// The scenario name: `[a-z0-9._-]`, non-empty, no leading dot (the
    /// golden rig's character set; it becomes a log/bundle file name).
    pub name: &'static str,
    /// The scenario class.
    pub class: ScenarioClass,
    /// The surface the invocation drives.
    pub surface: Surface,
    /// The CI tier (default [`Tier::Fast`]).
    pub tier: Tier,
    /// The scenario body.
    pub invocation: Invocation,
    /// Assertions over the outcome, evaluated in order.
    pub assertions: Vec<Assertion>,
    /// Expectations over the captured event stream, evaluated in order.
    pub logs: Vec<LogExpect>,
    /// `Some(kind)` marks this spec as a regression drill: the harness
    /// injects the corruption and expects RED (see the module docs).
    pub regression: Option<RegressionKind>,
}

impl ScenarioSpec {
    /// A fast-tier scenario with no assertions or expectations yet.
    #[must_use]
    pub fn new(
        name: &'static str,
        class: ScenarioClass,
        surface: Surface,
        invocation: Invocation,
    ) -> Self {
        Self {
            name,
            class,
            surface,
            tier: Tier::Fast,
            invocation,
            assertions: Vec::new(),
            logs: Vec::new(),
            regression: None,
        }
    }

    /// Set the CI tier.
    #[must_use]
    pub const fn tier(mut self, tier: Tier) -> Self {
        self.tier = tier;
        self
    }

    /// Replace the assertion list.
    #[must_use]
    pub fn assertions(mut self, assertions: Vec<Assertion>) -> Self {
        self.assertions = assertions;
        self
    }

    /// Append one assertion.
    #[must_use]
    pub fn assert(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Replace the log-expectation list.
    #[must_use]
    pub fn logs(mut self, logs: Vec<LogExpect>) -> Self {
        self.logs = logs;
        self
    }

    /// Append one log expectation.
    #[must_use]
    pub fn log_expect(mut self, expect: LogExpect) -> Self {
        self.logs.push(expect);
        self
    }

    /// Mark the spec as a regression drill (`inject_regression`).
    #[must_use]
    pub const fn regression(mut self, kind: RegressionKind) -> Self {
        self.regression = Some(kind);
        self
    }
}

impl fmt::Debug for ScenarioSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScenarioSpec")
            .field("name", &self.name)
            .field("class", &self.class)
            .field("surface", &self.surface)
            .field("tier", &self.tier)
            .field("assertions", &self.assertions)
            .field("logs", &self.logs)
            .field("regression", &self.regression)
            .finish_non_exhaustive()
    }
}

/// The deterministic scenario seed: the first eight bytes of the
/// scenario name's SHA-256, big-endian. Stable across runs and machines.
#[must_use]
pub fn scenario_seed(name: &str) -> u64 {
    let digest = sha256(name.as_bytes());
    let bytes = digest.as_bytes();
    let mut eight = [0u8; 8];
    eight.copy_from_slice(&bytes[..8]);
    u64::from_be_bytes(eight)
}

/// The scenario-name character rule, mirroring the golden rig's: names
/// become log and bundle file names and must never be a traversal vector.
fn valid_scenario_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
        && !name.starts_with('.')
}

/// What one scenario run concluded.
#[derive(Debug)]
pub enum Status {
    /// Every assertion and expectation held.
    Passed {
        /// The NDJSON run-log artifact.
        log: PathBuf,
    },
    /// At least one assertion, expectation, or the invocation failed.
    Failed(Failure),
    /// The run did not happen: pending surface, or a full-tier scenario
    /// without `FMN_E2E_FULL=1`. Never a failure.
    Skipped(&'static str),
    /// The spec itself violates the harness's rules (bad name, a drill
    /// with nothing to corrupt).
    SpecInvalid(String),
    /// The harness's own machinery failed (log/bundle I/O, a drill whose
    /// failure artifacts do not verify).
    HarnessError(String),
    /// A drill went RED for the injected reason, with the failure
    /// artifact and repro bundle verified. The drill's pass state.
    RegressionConfirmed {
        /// The (expected) failure, artifacts included.
        failure: Failure,
    },
    /// A drill went GREEN: the injection did not produce a failure, so
    /// the failure machinery is suspect. Always a test failure.
    RegressionNotDetected {
        /// The NDJSON run-log artifact.
        log: PathBuf,
        /// The injection that failed to register.
        kind: RegressionKind,
    },
}

impl Status {
    /// The stable lowercase tag for summaries.
    #[must_use]
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::Passed { .. } => "passed",
            Self::Failed(_) => "failed",
            Self::Skipped(_) => "skipped",
            Self::SpecInvalid(_) => "spec_invalid",
            Self::HarnessError(_) => "harness_error",
            Self::RegressionConfirmed { .. } => "regression_confirmed",
            Self::RegressionNotDetected { .. } => "regression_not_detected",
        }
    }
}

/// A failed run's full diagnostics: reasons, the run-log artifact, and
/// the FMNA repro bundle.
#[derive(Debug, Clone)]
pub struct Failure {
    /// The scenario's class (drills assert the artifacts carry it).
    pub class: ScenarioClass,
    /// Every assertion/expectation/invocation failure, in evaluation order.
    pub reasons: Vec<String>,
    /// The NDJSON run-log artifact.
    pub log: PathBuf,
    /// The FMNA repro bundle (journal + input closure + seed + fps).
    pub repro: PathBuf,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "scenario class {} failed with {} reason(s):",
            self.class.tag(),
            self.reasons.len()
        )?;
        for reason in &self.reasons {
            writeln!(f, "  - {reason}")?;
        }
        write!(
            f,
            "run log: {}\nrepro bundle: {}",
            self.log.display(),
            self.repro.display()
        )
    }
}

/// One scenario's run report.
#[derive(Debug)]
pub struct RunReport {
    /// The scenario name.
    pub scenario: &'static str,
    /// What the run concluded.
    pub status: Status,
}

impl RunReport {
    /// Whether the run did its job: a plain pass, or a confirmed drill.
    /// Skips, failures, and unconfirmed drills are all `false`.
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(
            self.status,
            Status::Passed { .. } | Status::RegressionConfirmed { .. }
        )
    }

    /// Whether the run went RED: a plain failure, or a drill's expected
    /// (confirmed) failure.
    #[must_use]
    pub const fn went_red(&self) -> bool {
        matches!(
            self.status,
            Status::Failed(_) | Status::RegressionConfirmed { .. }
        )
    }

    /// The NDJSON run-log artifact, when the run happened (pass, fail, or
    /// unconfirmed drill). Skips, invalid specs, and harness errors that
    /// occurred before the log could be written have none.
    #[must_use]
    pub fn log_artifact(&self) -> Option<&Path> {
        match &self.status {
            Status::Passed { log } | Status::RegressionNotDetected { log, .. } => Some(log),
            Status::Failed(failure) | Status::RegressionConfirmed { failure } => Some(&failure.log),
            Status::Skipped(_) | Status::SpecInvalid(_) | Status::HarnessError(_) => None,
        }
    }

    /// The FMNA repro bundle, when the run went RED (plain failure or
    /// confirmed drill).
    #[must_use]
    pub fn repro_bundle(&self) -> Option<&Path> {
        match &self.status {
            Status::Failed(failure) | Status::RegressionConfirmed { failure } => {
                Some(&failure.repro)
            }
            _ => None,
        }
    }

    /// One-line summary for test output.
    #[must_use]
    pub fn summary(&self) -> String {
        match &self.status {
            Status::Passed { log } => {
                format!("PASS {} (log: {})", self.scenario, log.display())
            }
            Status::Failed(failure) => format!("FAIL {} — {failure}", self.scenario),
            Status::Skipped(reason) => format!("SKIP {} ({reason})", self.scenario),
            Status::SpecInvalid(detail) => format!("SPEC-INVALID {} ({detail})", self.scenario),
            Status::HarnessError(detail) => format!("HARNESS-ERROR {} ({detail})", self.scenario),
            Status::RegressionConfirmed { failure } => format!(
                "DRILL-CONFIRMED {} (repro: {})",
                self.scenario,
                failure.repro.display()
            ),
            Status::RegressionNotDetected { log, kind } => format!(
                "DRILL-MISSED {} ({kind} injection went GREEN; log: {})",
                self.scenario,
                log.display()
            ),
        }
    }
}

/// The scenario runner: executes specs, captures the NDJSON run log,
/// evaluates assertions and log expectations, and bundles repros on
/// failure.
pub struct Runner {
    log_dir: PathBuf,
    goldens_dir: PathBuf,
    golden_mode: Mode,
}

impl Runner {
    /// A runner with explicit roots and golden mode (the parallel-safe
    /// form tests use; see `tests/golden_rig.rs`'s mode doctrine).
    #[must_use]
    pub fn new(
        log_dir: impl Into<PathBuf>,
        goldens_dir: impl Into<PathBuf>,
        golden_mode: Mode,
    ) -> Self {
        Self {
            log_dir: log_dir.into(),
            goldens_dir: goldens_dir.into(),
            golden_mode,
        }
    }

    /// The CI configuration: logs under `CARGO_TARGET_TMPDIR/e2e_logs/`
    /// (the OS temp dir outside `cargo test`), goldens at this crate's
    /// committed `goldens/`, golden mode from `UPDATE_GOLDENS`.
    #[must_use]
    pub fn from_env() -> Self {
        let log_dir = std::env::var("CARGO_TARGET_TMPDIR").map_or_else(
            |_| std::env::temp_dir().join("fmn_e2e_logs"),
            |dir| PathBuf::from(dir).join("e2e_logs"),
        );
        let goldens_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens");
        Self::new(log_dir, goldens_dir, Mode::from_env())
    }

    /// The directory per-run NDJSON logs and repro bundles land in.
    #[must_use]
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// The directory golden suites resolve under.
    #[must_use]
    pub fn goldens_dir(&self) -> &Path {
        &self.goldens_dir
    }

    /// Run one scenario, reading the full-matrix gate from
    /// `FMN_E2E_FULL` in the environment.
    pub fn run(&self, spec: ScenarioSpec) -> RunReport {
        self.run_gated(spec, full_matrix_from_env())
    }

    /// Run one scenario with an explicit full-matrix gate (the
    /// parallel-safe form: tests never mutate process env).
    pub fn run_gated(&self, spec: ScenarioSpec, full_matrix: bool) -> RunReport {
        let name = spec.name;
        let status = self.dispatch(spec, full_matrix);
        RunReport {
            scenario: name,
            status,
        }
    }

    /// Run a catalog, reading the full-matrix gate from `FMN_E2E_FULL`.
    pub fn run_all(&self, specs: impl IntoIterator<Item = ScenarioSpec>) -> Vec<RunReport> {
        self.run_all_gated(specs, full_matrix_from_env())
    }

    /// Run a catalog with an explicit full-matrix gate.
    pub fn run_all_gated(
        &self,
        specs: impl IntoIterator<Item = ScenarioSpec>,
        full_matrix: bool,
    ) -> Vec<RunReport> {
        specs
            .into_iter()
            .map(|spec| self.run_gated(spec, full_matrix))
            .collect()
    }

    fn dispatch(&self, spec: ScenarioSpec, full_matrix: bool) -> Status {
        if let Some(reason) = spec.surface.pending_reason() {
            return Status::Skipped(reason);
        }
        if !spec.tier.enabled(full_matrix) {
            return Status::Skipped("tier full requires FMN_E2E_FULL=1");
        }
        if !valid_scenario_name(spec.name) {
            return Status::SpecInvalid(format!(
                "invalid scenario name {:?}: use [a-z0-9._-], non-empty, no leading dot",
                spec.name
            ));
        }
        if let Some(kind) = spec.regression
            && let Err(detail) = validate_drill(kind, &spec)
        {
            return Status::SpecInvalid(detail);
        }
        self.execute(spec)
    }

    fn execute(&self, spec: ScenarioSpec) -> Status {
        let name = spec.name;
        let seed = scenario_seed(name);
        let mut ctx = RunCtx::new(seed);
        // The journal's first entry is the harness's own, so even a
        // scenario that records nothing bundles a class-tagged journal.
        ctx.record_journal(Entry {
            command: CommandRecord {
                kind: CommandKind::Custom,
                identity: sha256(format!("e2e:{name}:{seed}").as_bytes()),
                label: format!(
                    "e2e.begin class={} name={name} seed={seed}",
                    spec.class.tag()
                ),
            },
            effect: EffectClass::Opaque,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: None,
            state_hash: sha256(name.as_bytes()),
        });
        if let Some(status) = self.recording_error_status(&mut ctx, name) {
            return status;
        }
        ctx.event(
            LogEvent::new(spans::HARNESS_BEGIN)
                .field("scenario", name)
                .field("class", spec.class.tag())
                .field("surface", spec.surface.tag())
                .field("tier", spec.tier.tag())
                .field("seed", seed),
        );

        let mut reasons: Vec<String> = Vec::new();
        let mut outcome = match spec.invocation.invoke(&mut ctx) {
            Ok(outcome) => {
                ctx.event(
                    LogEvent::new(spans::HARNESS_INVOCATION)
                        .field("ok", true)
                        .field("exit_code", outcome.exit_code.to_string())
                        .field("artifacts", outcome.artifacts.len()),
                );
                outcome
            }
            Err(error) => {
                ctx.event(
                    LogEvent::new(spans::HARNESS_INVOCATION)
                        .field("ok", false)
                        .field("error", error.to_string()),
                );
                reasons.push(format!("invocation failed: {error}"));
                RunOutcome::ok()
            }
        };
        if let Some(status) = self.recording_error_status(&mut ctx, name) {
            return status;
        }

        if let Some(kind) = spec.regression {
            match kind {
                RegressionKind::GoldenDrift => match outcome.artifacts.first_mut() {
                    Some(artifact) if !artifact.bytes.is_empty() => {
                        let mid = artifact.bytes.len() / 2;
                        artifact.bytes[mid] ^= 0xFF;
                        ctx.event(
                            LogEvent::new(spans::HARNESS_REGRESSION)
                                .field("kind", kind.tag())
                                .field(
                                    "detail",
                                    format!("flipped byte {mid} of artifact {:?}", artifact.name),
                                ),
                        );
                    }
                    _ => {
                        return self.harness_error(
                            &mut ctx,
                            name,
                            "golden-drift drill produced no non-empty artifact to corrupt"
                                .to_string(),
                        );
                    }
                },
                RegressionKind::LogExpectation => {
                    let (detail, effected) = corrupt_log_for_drill(&mut ctx, &spec.logs);
                    if !effected {
                        return self.harness_error(
                            &mut ctx,
                            name,
                            format!("log-expectation injection was vacuous: {detail}"),
                        );
                    }
                    ctx.event(
                        LogEvent::new(spans::HARNESS_REGRESSION)
                            .field("kind", kind.tag())
                            .field("detail", detail),
                    );
                }
            }
        }
        if let Some(status) = self.recording_error_status(&mut ctx, name) {
            return status;
        }

        // A golden-drift drill always evaluates goldens in check mode:
        // the injected corruption must never be blessable.
        // ubs:ignore — compares a non-secret internal enum
        let golden_mode = if spec.regression == Some(RegressionKind::GoldenDrift) {
            Mode::Check
        } else {
            self.golden_mode
        };

        for assertion in &spec.assertions {
            let result = self.evaluate(assertion, &outcome, ctx.records(), golden_mode);
            match result {
                Ok(()) => ctx.event(
                    LogEvent::new(spans::HARNESS_ASSERTION)
                        .field("assertion", assertion.describe())
                        .field("ok", true),
                ),
                Err(detail) => {
                    reasons.push(format!(
                        "assertion {} failed: {detail}",
                        assertion.describe()
                    ));
                    ctx.event(
                        LogEvent::new(spans::HARNESS_ASSERTION)
                            .field("assertion", assertion.describe())
                            .field("ok", false)
                            .field("detail", detail),
                    );
                }
            }
        }
        for expect in &spec.logs {
            let result = expect.check(ctx.records());
            match result {
                Ok(()) => ctx.event(
                    LogEvent::new(spans::HARNESS_ASSERTION)
                        .field("assertion", expect.describe())
                        .field("ok", true),
                ),
                Err(detail) => {
                    reasons.push(format!(
                        "log expectation {} failed: {detail}",
                        expect.describe()
                    ));
                    ctx.event(
                        LogEvent::new(spans::HARNESS_ASSERTION)
                            .field("assertion", expect.describe())
                            .field("ok", false)
                            .field("detail", detail),
                    );
                }
            }
        }
        if let Some(status) = self.recording_error_status(&mut ctx, name) {
            return status;
        }

        let failed = !reasons.is_empty();
        ctx.event(
            LogEvent::new(spans::HARNESS_END)
                .field("status", if failed { "failed" } else { "passed" })
                .field("reasons", reasons.len()),
        );
        if let Some(status) = self.recording_error_status(&mut ctx, name) {
            return status;
        }
        let log = match self.write_log(&ctx, name) {
            Ok(path) => path,
            Err(detail) => {
                return Status::HarnessError(format!("could not write the run log: {detail}"));
            }
        };

        if !failed {
            return match spec.regression {
                None => Status::Passed { log },
                Some(kind) => Status::RegressionNotDetected { log, kind },
            };
        }

        let bundle = ReproBundle {
            scene_label: name.to_string(),
            seed,
            fps: ctx.fps(),
            closure: ctx.closure.clone(),
            journal: ctx.journal,
        };
        let bytes = match bundle.to_bytes() {
            Ok(bytes) => bytes,
            Err(error) => {
                return Status::HarnessError(format!(
                    "repro bundle would not serialize: {error} (run log at {})",
                    log.display()
                ));
            }
        };
        let repro = self.log_dir.join(format!("{name}.repro.fmna"));
        if let Err(error) = std::fs::write(&repro, &bytes) {
            return Status::HarnessError(format!(
                "could not write the repro bundle {}: {error} (run log at {})",
                repro.display(),
                log.display()
            ));
        }
        let failure = Failure {
            class: spec.class,
            reasons,
            log,
            repro,
        };
        match spec.regression {
            None => Status::Failed(failure),
            Some(kind) => Self::confirm_drill(kind, spec.class, failure),
        }
    }

    fn recording_error_status(&self, ctx: &mut RunCtx, name: &str) -> Option<Status> {
        let detail = ctx.recording_error()?.to_string();
        Some(self.harness_error(
            ctx,
            name,
            format!("run evidence recording failed: {detail}"),
        ))
    }

    /// Evaluate one assertion. Golden locks resolve against the runner's
    /// goldens directory under the given mode; the NDJSON schema check
    /// round-trips the captured record stream itself.
    fn evaluate(
        &self,
        assertion: &Assertion,
        outcome: &RunOutcome,
        records: &[LogRecord],
        golden_mode: Mode,
    ) -> Result<(), String> {
        match assertion {
            Assertion::GoldenLock { suite, scope } => {
                let store = GoldenStore::new(&self.goldens_dir, suite, *scope)
                    .map_err(|e| e.to_string())?;
                let mut drifts = Vec::new();
                for artifact in &outcome.artifacts {
                    match store.check_with_mode(&artifact.name, &artifact.bytes, golden_mode) {
                        Ok(Verdict::Match | Verdict::Blessed { .. }) => {}
                        Err(error) => drifts.push(error.to_string()),
                    }
                }
                if drifts.is_empty() {
                    Ok(())
                } else {
                    Err(drifts.join("; "))
                }
            }
            Assertion::Structural(s) => s.check(outcome),
            Assertion::ExitCode(code) => {
                if outcome.exit_code == *code {
                    Ok(())
                } else {
                    Err(format!("exit code {}, expected {code}", outcome.exit_code))
                }
            }
            Assertion::FileInventory(expected) => {
                let mut actual: Vec<&str> =
                    outcome.artifacts.iter().map(|a| a.name.as_str()).collect();
                actual.sort_unstable();
                let mut want: Vec<&str> = expected.iter().map(String::as_str).collect();
                want.sort_unstable();
                if actual == want {
                    Ok(())
                } else {
                    Err(format!(
                        "emitted-file inventory mismatch: expected {want:?}, got {actual:?}"
                    ))
                }
            }
            Assertion::NdjsonSchema => check_ndjson_roundtrip(records),
        }
    }

    /// Write the run's NDJSON log; returns the artifact path.
    fn write_log(&self, ctx: &RunCtx, name: &str) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.log_dir).map_err(|error| {
            format!(
                "could not create log dir {}: {error}",
                self.log_dir.display()
            )
        })?;
        let body = render_log_artifact(ctx.records())?;
        let path = self.log_dir.join(format!("{name}.ndjson"));
        std::fs::write(&path, body)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        Ok(path)
    }

    /// A mid-run harness failure: close the log best-effort and report.
    fn harness_error(&self, ctx: &mut RunCtx, name: &str, detail: String) -> Status {
        ctx.event(
            LogEvent::new(spans::HARNESS_END)
                .field("status", "harness_error")
                .field("reasons", 1u64),
        );
        match self.write_log(ctx, name) {
            Ok(path) => Status::HarnessError(format!("{detail} (run log at {})", path.display())),
            Err(log_detail) => Status::HarnessError(format!(
                "{detail}; also could not write the run log: {log_detail}"
            )),
        }
    }

    /// Verify a drill's expected RED: the failure came from the
    /// injection, the failure artifact exists, parses, and names the
    /// scenario class, and the repro bundle parses with the class in its
    /// journal. Anything less is a harness error, not a drill pass.
    fn confirm_drill(kind: RegressionKind, class: ScenarioClass, failure: Failure) -> Status {
        let needle = match kind {
            RegressionKind::GoldenDrift => "self-golden drift",
            RegressionKind::LogExpectation => "log expectation",
        };
        if !failure.reasons.iter().any(|r| r.contains(needle)) {
            return Status::HarnessError(format!(
                "{} drill went RED but not via the injection (no reason contains {needle:?}): {:?}",
                kind.tag(),
                failure.reasons
            ));
        }
        let records = match read_log_artifact(&failure.log) {
            Ok(records) => records,
            Err(detail) => return Status::HarnessError(format!("failure log invalid: {detail}")),
        };
        let mut saw_class = false;
        for record in records {
            if let LogRecord::Event { name, fields, .. } = record
                && name == spans::HARNESS_BEGIN
            {
                saw_class = fields
                    .iter()
                    .any(|(k, v)| k == "class" && *v == FieldValue::Str(class.tag().to_string()));
            }
        }
        if !saw_class {
            return Status::HarnessError(format!(
                "failure log {} does not name scenario class {}",
                failure.log.display(),
                class.tag()
            ));
        }
        let bytes = match read_bounded_file(
            &failure.repro,
            SerialLimits::DEFAULT.max_total,
            "repro bundle",
        ) {
            Ok(bytes) => bytes,
            Err(detail) => return Status::HarnessError(detail),
        };
        let bundle = match ReproBundle::from_bytes(&bytes) {
            Ok(bundle) => bundle,
            Err(error) => {
                return Status::HarnessError(format!(
                    "repro bundle {} does not parse: {error}",
                    failure.repro.display()
                ));
            }
        };
        match bundle.journal.entries().first() {
            Some(entry) if entry.command.label.contains(class.tag()) => {
                Status::RegressionConfirmed { failure }
            }
            Some(_) => Status::HarnessError(format!(
                "repro bundle journal does not name scenario class {}",
                class.tag()
            )),
            None => Status::HarnessError("repro bundle journal is empty".to_string()),
        }
    }
}

fn parse_log_artifact_body(path: &Path, body: &str) -> Result<Vec<LogRecord>, String> {
    if body.len() > MAX_LOG_ARTIFACT_BYTES {
        return Err(format!(
            "{}: artifact exceeds the {MAX_LOG_ARTIFACT_BYTES}-byte limit",
            path.display()
        ));
    }
    if body.is_empty() {
        return Err(format!("{}: empty log artifact", path.display()));
    }
    if !body.ends_with('\n') {
        return Err(format!(
            "{}: log artifact must end with one LF",
            path.display()
        ));
    }
    if body.as_bytes().contains(&b'\r') {
        return Err(format!(
            "{}: log artifact contains a carriage return",
            path.display()
        ));
    }

    let content = body
        .strip_suffix('\n')
        .ok_or_else(|| format!("{}: log artifact must end with one LF", path.display()))?;
    if content.is_empty() {
        return Err(format!("{}: empty log artifact", path.display()));
    }
    let mut parsed = Vec::new();
    for (index, line) in content.split('\n').enumerate() {
        if line.is_empty() {
            return Err(format!(
                "{} line {}: blank records are forbidden",
                path.display(),
                index + 1
            ));
        }
        if parsed.len() == MAX_LOG_RECORDS {
            return Err(format!(
                "{}: artifact has more than {MAX_LOG_RECORDS} records",
                path.display()
            ));
        }
        let record = LogRecord::parse(line)
            .map_err(|detail| format!("{} line {}: {detail}", path.display(), index + 1))?;
        let expected_seq = u64::try_from(parsed.len()).unwrap_or(u64::MAX);
        if record.seq() != expected_seq {
            return Err(format!(
                "{} line {}: sequence {}, expected {expected_seq}",
                path.display(),
                index + 1,
                record.seq()
            ));
        }
        parsed.push(record);
    }
    Ok(parsed)
}

fn read_bounded_file(path: &Path, max_bytes: usize, kind: &str) -> Result<Vec<u8>, String> {
    let max_bytes_u64 = u64::try_from(max_bytes)
        .map_err(|_| format!("{kind} byte limit is not representable as u64"))?;
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("could not inspect {kind} {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{}: {kind} is not a regular file", path.display()));
    }
    if metadata.len() > max_bytes_u64 {
        return Err(format!(
            "{}: {kind} exceeds the {max_bytes}-byte limit",
            path.display(),
        ));
    }
    let file = std::fs::File::open(path)
        .map_err(|error| format!("could not open {kind} {}: {error}", path.display()))?;
    let read_limit = max_bytes_u64
        .checked_add(1)
        .ok_or_else(|| format!("{kind} byte limit cannot be incremented"))?;
    let mut bytes = Vec::new();
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("could not read {kind} {}: {error}", path.display()))?;
    if bytes.len() > max_bytes {
        return Err(format!(
            "{}: {kind} exceeds the {max_bytes}-byte limit",
            path.display(),
        ));
    }
    Ok(bytes)
}

fn read_log_artifact(path: &Path) -> Result<Vec<LogRecord>, String> {
    let bytes = read_bounded_file(path, MAX_LOG_ARTIFACT_BYTES, "log artifact")?;
    let body = std::str::from_utf8(&bytes)
        .map_err(|_| format!("{}: log artifact is not UTF-8", path.display()))?;
    parse_log_artifact_body(path, body)
}

/// Validate an on-disk NDJSON run-log artifact against the declared
/// canonical schema: the bounded file is UTF-8 with LF-only framing and a
/// final LF, every nonempty line exactly matches the serializer grammar,
/// and sequence numbers are gapless from zero. Artifacts are limited to 1
/// MiB, 4,096 records, and 64 KiB per record. The runner validates the same
/// contract in-memory at [`Assertion::NdjsonSchema`] time.
///
/// # Errors
/// A description of the first schema violation, or of the I/O failure.
pub fn validate_log_artifact(path: &Path) -> Result<(), String> {
    read_log_artifact(path).map(|_| ())
}

/// Static drill validation: the corruption must have something to bite
/// on. Anything not checkable until the run (e.g. artifacts existing) is
/// a runtime harness error instead.
fn validate_drill(kind: RegressionKind, spec: &ScenarioSpec) -> Result<(), String> {
    match kind {
        RegressionKind::GoldenDrift => {
            if spec
                .assertions
                .iter()
                .any(|a| matches!(a, Assertion::GoldenLock { .. }))
            {
                Ok(())
            } else {
                Err(format!(
                    "{}: golden-drift drill requires a GoldenLock assertion to violate",
                    spec.name
                ))
            }
        }
        RegressionKind::LogExpectation => {
            if drill_log_target(&spec.logs).is_some() {
                Ok(())
            } else {
                Err(format!(
                    "{}: log-expectation drill requires a span_present, counter_ge, or \
                     no_event expectation to corrupt",
                    spec.name
                ))
            }
        }
    }
}

/// The expectation a log-corruption drill targets, in spec order.
enum DrillTarget<'a> {
    Span(&'a str),
    Counter(&'a str),
    Forbidden(&'a str),
}

fn drill_log_target(logs: &[LogExpect]) -> Option<DrillTarget<'_>> {
    for expect in logs {
        match expect {
            LogExpect::SpanPresent { span, .. } => return Some(DrillTarget::Span(span)),
            LogExpect::CounterGe { counter, .. } => return Some(DrillTarget::Counter(counter)),
            LogExpect::NoEvent(name) => return Some(DrillTarget::Forbidden(name)),
            LogExpect::EventOrder { .. } => {}
        }
    }
    None
}

/// Apply the log-expectation corruption: drop the expected span's events,
/// erase the expected counter's samples, or emit the forbidden event;
/// then renumber so the stream stays gapless. Returns the human detail
/// and whether the corruption actually bit — a vacuous injection (the
/// targeted span/counter never occurred, so nothing was dropped) cannot
/// be the cause of a RED and must not confirm a drill.
fn corrupt_log_for_drill(ctx: &mut RunCtx, logs: &[LogExpect]) -> (String, bool) {
    match drill_log_target(logs) {
        Some(DrillTarget::Span(span)) => {
            let before = ctx.log.len();
            ctx.log
                .retain(|r| !matches!(r, LogRecord::Event { name, .. } if name == span));
            let dropped = before - ctx.log.len();
            ctx.renumber();
            (
                format!("dropped {dropped} event(s) named {span:?}"),
                dropped > 0,
            )
        }
        Some(DrillTarget::Counter(counter)) => {
            let before = ctx.log.len();
            ctx.log
                .retain(|r| !matches!(r, LogRecord::Counter { name, .. } if name == counter));
            let dropped = before - ctx.log.len();
            ctx.renumber();
            (
                format!("dropped {dropped} counter sample(s) named {counter:?}"),
                dropped > 0,
            )
        }
        Some(DrillTarget::Forbidden(name)) => {
            ctx.event(LogEvent::new(name).field("injected", true));
            (format!("emitted forbidden event {name:?}"), true)
        }
        // validate_drill guarantees a target; this arm is unreachable in
        // practice but kept total rather than panicking.
        None => ("no corruption target (invalid drill)".to_string(), false),
    }
}

fn render_log_artifact(records: &[LogRecord]) -> Result<String, String> {
    if records.is_empty() {
        return Err("log artifact has no records".to_string());
    }
    if records.len() > MAX_LOG_RECORDS {
        return Err(format!(
            "log artifact has more than {MAX_LOG_RECORDS} records"
        ));
    }
    let mut body = String::new();
    for (index, record) in records.iter().enumerate() {
        record
            .validate_limits()
            .map_err(|detail| format!("line {}: {detail}", index + 1))?;
        let expect_seq = u64::try_from(index).unwrap_or(u64::MAX);
        if record.seq() != expect_seq {
            return Err(format!(
                "line {} has sequence {}, expected {expect_seq}",
                index + 1,
                record.seq()
            ));
        }
        let line = record.to_line();
        if line.len() > MAX_LOG_LINE_BYTES {
            return Err(format!(
                "line {} exceeds the {MAX_LOG_LINE_BYTES}-byte limit",
                index + 1
            ));
        }
        let parsed = LogRecord::parse(&line)
            .map_err(|detail| format!("line {} failed to parse: {detail}", index + 1))?;
        if parsed != *record {
            return Err(format!("line {} did not round-trip", index + 1));
        }
        let next_len = body
            .len()
            .checked_add(line.len())
            .and_then(|len| len.checked_add(1))
            .ok_or_else(|| "log artifact size overflow".to_string())?;
        if next_len > MAX_LOG_ARTIFACT_BYTES {
            return Err(format!(
                "log artifact exceeds the {MAX_LOG_ARTIFACT_BYTES}-byte limit"
            ));
        }
        body.push_str(&line);
        body.push('\n');
    }
    Ok(body)
}

/// The NDJSON schema check: every captured record serializes to a bounded,
/// canonical line that parses back identically, and sequence numbers are
/// gapless.
fn check_ndjson_roundtrip(records: &[LogRecord]) -> Result<(), String> {
    render_log_artifact(records).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(ctx: &RunCtx) -> &[LogRecord] {
        ctx.records()
    }

    #[test]
    fn ndjson_roundtrip_events_and_counters() {
        let mut ctx = RunCtx::new(7);
        ctx.event(
            LogEvent::new("preflight")
                .field("fonts", 3u64)
                .field("ready", true)
                .field("label", "quo\\ted \"text\"\nline\u{000b}—done"),
        );
        ctx.counter("frames_emitted", 12);
        ctx.event(LogEvent::new("empty.fields"));
        assert_eq!(check_ndjson_roundtrip(records(&ctx)), Ok(()));
    }

    #[test]
    fn ndjson_rejects_garbage() {
        assert!(LogRecord::parse("not json").is_err());
        assert!(LogRecord::parse("{\"seq\":0,\"kind\":\"mystery\",\"name\":\"x\"}").is_err());
        assert!(
            LogRecord::parse("{\"kind\":\"event\",\"seq\":0,\"name\":\"x\",\"fields\":{}}")
                .is_err()
        );
    }

    #[test]
    fn ndjson_rejects_noncanonical_json_spellings() {
        let lines = [
            r#" {"seq":0,"kind":"counter","name":"x","value":1}"#,
            r#"{"seq":00,"kind":"counter","name":"x","value":1}"#,
            r#"{"seq":0, "kind":"counter","name":"x","value":1}"#,
            r#"{"seq":0,"kind":"counter","name":"x\/y","value":1}"#,
            r#"{"seq":0,"kind":"counter","name":"\u0078","value":1}"#,
            r#"{"seq":0,"kind":"counter","name":"x","value":1} "#,
            r#"{"seq":0,"kind":"counter","name":"x\b","value":1}"#,
            r#"{"seq":0,"kind":"event","name":"x","fields":{"a":1,"a":2}}"#,
            r#"{"seq":184467440737095516150,"kind":"counter","name":"x","value":1}"#,
        ];
        for line in lines {
            assert!(
                LogRecord::parse(line).is_err(),
                "noncanonical line was accepted: {line}"
            );
        }

        let raw_control = format!(
            "{{\"seq\":0,\"kind\":\"counter\",\"name\":\"x{}\",\"value\":1}}",
            '\u{0001}'
        );
        assert!(LogRecord::parse(&raw_control).is_err());
    }

    #[test]
    fn ndjson_enforces_line_and_field_limits_before_acceptance() {
        let oversized_line = "x".repeat(MAX_LOG_LINE_BYTES + 1);
        assert!(LogRecord::parse(&oversized_line).is_err());

        let oversized_name = "n".repeat(MAX_LOG_NAME_BYTES + 1);
        let line =
            format!("{{\"seq\":0,\"kind\":\"counter\",\"name\":\"{oversized_name}\",\"value\":1}}");
        assert!(LogRecord::parse(&line).is_err());

        let oversized_value = "v".repeat(MAX_LOG_STRING_BYTES + 1);
        let line = format!(
            "{{\"seq\":0,\"kind\":\"event\",\"name\":\"x\",\"fields\":{{\"value\":\"{oversized_value}\"}}}}"
        );
        assert!(LogRecord::parse(&line).is_err());

        let mut fields = String::new();
        for index in 0..=MAX_LOG_EVENT_FIELDS {
            if index > 0 {
                fields.push(',');
            }
            fields.push_str(&format!("\"f{index}\":{index}"));
        }
        let line =
            format!("{{\"seq\":0,\"kind\":\"event\",\"name\":\"x\",\"fields\":{{{fields}}}}}");
        assert!(LogRecord::parse(&line).is_err());

        let mut ctx = RunCtx::new(1);
        ctx.event(LogEvent::new("duplicate").field("x", 1u64).field("x", 2u64));
        assert!(ctx.records().is_empty());
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::InvalidLogRecord(detail))
                if detail.contains("duplicate event field key")
        ));
    }

    #[test]
    fn ndjson_artifact_framing_is_canonical_and_bounded() {
        let path = Path::new("run.ndjson");
        let first = LogRecord::Counter {
            seq: 0,
            name: "frames".to_string(),
            value: 1,
        }
        .to_line();
        let second = LogRecord::Counter {
            seq: 1,
            name: "frames".to_string(),
            value: 2,
        }
        .to_line();
        let canonical = format!("{first}\n{second}\n");
        assert_eq!(
            parse_log_artifact_body(path, &canonical)
                .expect("canonical artifact parses")
                .len(),
            2
        );

        let malformed = [
            canonical.trim_end_matches('\n').to_string(),
            canonical.replace('\n', "\r\n"),
            format!("{first}\n\n{second}\n"),
            "\n".to_string(),
            format!("{first}\n{first}\n"),
        ];
        for body in malformed {
            assert!(
                parse_log_artifact_body(path, &body).is_err(),
                "malformed artifact was accepted"
            );
        }

        let oversized = "x".repeat(MAX_LOG_ARTIFACT_BYTES + 1);
        assert!(parse_log_artifact_body(path, &oversized).is_err());

        let mut too_many = String::new();
        for seq in 0..=MAX_LOG_RECORDS {
            let record = LogRecord::Counter {
                seq: u64::try_from(seq).unwrap_or(u64::MAX),
                name: "n".to_string(),
                value: 0,
            };
            too_many.push_str(&record.to_line());
            too_many.push('\n');
        }
        assert!(parse_log_artifact_body(path, &too_many).is_err());
    }

    #[test]
    fn run_context_caps_records_before_retention() {
        let mut ctx = RunCtx::new(1);
        for _ in 0..=MAX_LOG_RECORDS {
            ctx.counter("bounded", 1);
        }

        assert_eq!(ctx.records().len(), MAX_LOG_RECORDS);
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::LogRecordLimit { limit }) if *limit == MAX_LOG_RECORDS
        ));

        ctx.event(LogEvent::new("ignored-after-limit"));
        assert_eq!(ctx.records().len(), MAX_LOG_RECORDS);
    }

    fn minimal_journal_entry() -> Entry {
        Entry {
            command: CommandRecord {
                kind: CommandKind::Play,
                identity: sha256(b"bounded journal entry"),
                label: "bounded journal entry".to_string(),
            },
            effect: EffectClass::Pure,
            reads: Vec::new(),
            subprocesses: Vec::new(),
            checkpoint: None,
            state_hash: sha256(b"bounded journal state"),
        }
    }

    #[test]
    fn run_context_caps_journal_entries_before_retention() {
        let mut ctx = RunCtx::new(1);
        for _ in 0..=MAX_REPRO_JOURNAL_ENTRIES {
            ctx.record_journal(minimal_journal_entry());
        }

        assert_eq!(ctx.journal().entries().len(), MAX_REPRO_JOURNAL_ENTRIES);
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::JournalEntryLimit { limit })
                if *limit == MAX_REPRO_JOURNAL_ENTRIES
        ));

        ctx.record_asset("ignored-after-limit", b"content");
        assert!(ctx.closure().is_empty());
    }

    #[test]
    fn run_context_caps_closure_rows_before_retention() {
        let mut ctx = RunCtx::new(1);
        for _ in 0..=MAX_REPRO_CLOSURE_ASSETS {
            ctx.record_asset("bounded.asset", b"content");
        }

        assert_eq!(ctx.closure().len(), MAX_REPRO_CLOSURE_ASSETS);
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::ClosureAssetLimit { limit })
                if *limit == MAX_REPRO_CLOSURE_ASSETS
        ));
    }

    #[test]
    fn run_context_rejects_overlong_closure_path_before_retention() {
        let mut exact = RunCtx::new(1);
        let exact_path = "p".repeat(MAX_REPRO_PATH_BYTES);
        exact.record_asset(&exact_path, b"content");
        assert_eq!(exact.closure().len(), 1);
        assert!(exact.recording_error().is_none());

        let mut ctx = RunCtx::new(1);
        let path = "p".repeat(MAX_REPRO_PATH_BYTES + 1);
        ctx.record_asset(&path, b"content");
        assert!(ctx.closure().is_empty());
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::InvalidClosureAsset(detail))
                if detail.contains("asset path")
        ));
    }

    #[test]
    fn run_context_enforces_exact_journal_byte_boundary() {
        let entry = minimal_journal_entry();
        let entry_bytes = journal_entry_wire_bytes(&entry).expect("minimal entry has a wire size");
        let mut ctx = RunCtx::new(1);
        ctx.journal_bytes = MAX_REPRO_JOURNAL_BYTES - entry_bytes;

        ctx.record_journal(entry.clone());
        assert_eq!(ctx.journal_bytes, MAX_REPRO_JOURNAL_BYTES);
        assert_eq!(ctx.journal().entries().len(), 1);
        assert!(ctx.recording_error().is_none());

        ctx.record_journal(entry);
        assert_eq!(ctx.journal().entries().len(), 1);
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::JournalByteLimit { limit, needed })
                if *limit == MAX_REPRO_JOURNAL_BYTES && *needed > *limit
        ));
    }

    #[test]
    fn run_context_journal_accounting_matches_canonical_bytes() {
        let mut entry = minimal_journal_entry();
        entry.effect = EffectClass::Stateful(vec![
            fmn_scene::journal::ImpureEffectTag::DtUpdater,
            fmn_scene::journal::ImpureEffectTag::SceneUpdater,
        ]);
        entry.reads.push(AssetRead {
            path: "formula.tex".to_string(),
            digest: sha256(b"formula"),
        });
        entry
            .subprocesses
            .push(fmn_scene::journal::SubprocessRecord {
                tool_sha256_hex: sha256(b"tool").to_hex(),
                argv_digest: sha256(b"argv"),
                destination: "render.mp4".to_string(),
            });
        entry.checkpoint = Some(b"checkpoint".to_vec());

        let mut ctx = RunCtx::new(1);
        ctx.record_journal(entry);
        assert!(ctx.recording_error().is_none());
        assert_eq!(
            ctx.journal_bytes,
            ctx.journal
                .to_bytes()
                .expect("admitted journal serializes")
                .len()
        );

        let mut custom = minimal_journal_entry();
        custom.command.kind = CommandKind::Custom;
        custom.effect = EffectClass::Stateful(vec![
            fmn_scene::journal::ImpureEffectTag::UnclassifiedAnimation,
        ]);
        ctx.record_journal(custom);
        assert!(ctx.recording_error().is_none());
        assert_eq!(
            ctx.journal_bytes,
            ctx.journal
                .to_bytes()
                .expect("coerced journal serializes")
                .len()
        );
    }

    #[test]
    fn run_context_rejects_overlong_journal_fields_before_retention() {
        let mut entry = minimal_journal_entry();
        entry.command.label = "l".repeat(MAX_REPRO_STRING_BYTES + 1);
        let mut ctx = RunCtx::new(1);
        ctx.record_journal(entry);

        assert!(ctx.journal().entries().is_empty());
        assert!(matches!(
            ctx.recording_error(),
            Some(RecordingError::InvalidJournalEntry(detail))
                if detail.contains("command label")
        ));
    }

    #[test]
    fn bounded_file_reader_checks_type_and_size_before_consuming() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("Cargo.toml");
        let size = usize::try_from(
            std::fs::metadata(&path)
                .expect("crate manifest metadata")
                .len(),
        )
        .expect("crate manifest size fits usize");
        assert!(size > 0);
        assert_eq!(
            read_bounded_file(&path, size, "probe")
                .expect("file at the exact limit reads")
                .len(),
            size
        );
        assert!(read_bounded_file(&path, size.saturating_sub(1), "probe").is_err());
        assert!(read_bounded_file(&root, size, "probe").is_err());
    }

    #[test]
    fn field_predicates() {
        let fields = vec![
            ("a".to_string(), FieldValue::U64(5)),
            ("b".to_string(), FieldValue::Str("x".to_string())),
            ("c".to_string(), FieldValue::Bool(true)),
        ];
        assert!(FieldPred::exists("a").check(&fields));
        assert!(!FieldPred::exists("z").check(&fields));
        assert!(FieldPred::u64_ge("a", 5).check(&fields));
        assert!(!FieldPred::u64_ge("a", 6).check(&fields));
        assert!(FieldPred::u64_eq("a", 5).check(&fields));
        assert!(FieldPred::str_eq("b", "x").check(&fields));
        assert!(!FieldPred::str_eq("b", "y").check(&fields));
        assert!(FieldPred::bool_eq("c", true).check(&fields));
        assert!(!FieldPred::str_eq("a", "5").check(&fields));
    }

    #[test]
    fn log_expect_evaluation() {
        let mut ctx = RunCtx::new(1);
        ctx.event(LogEvent::new("a").field("n", 2u64));
        ctx.counter("c", 3);
        ctx.counter("c", 9);
        ctx.event(LogEvent::new("b"));
        let log = records(&ctx);
        assert!(
            LogExpect::span_present("a", vec![FieldPred::u64_ge("n", 2)])
                .check(log)
                .is_ok()
        );
        assert!(
            LogExpect::span_present("a", vec![FieldPred::u64_ge("n", 3)])
                .check(log)
                .is_err()
        );
        assert!(
            LogExpect::span_present("missing", vec![])
                .check(log)
                .is_err()
        );
        assert!(LogExpect::counter_ge("c", 9).check(log).is_ok());
        assert!(LogExpect::counter_ge("c", 10).check(log).is_err());
        assert!(LogExpect::counter_ge("never", 0).check(log).is_err());
        assert!(LogExpect::event_order("a", "b").check(log).is_ok());
        assert!(LogExpect::event_order("b", "a").check(log).is_err());
        assert!(LogExpect::event_order("a", "missing").check(log).is_err());
        assert!(LogExpect::no_event("zzz").check(log).is_ok());
        assert!(LogExpect::no_event("a").check(log).is_err());
        assert!(LogExpect::no_event("c").check(log).is_err());
    }

    #[test]
    fn seed_is_deterministic_and_name_derived() {
        assert_eq!(scenario_seed("s.v1"), scenario_seed("s.v1"));
        assert_ne!(scenario_seed("s.v1"), scenario_seed("s.v2"));
    }

    #[test]
    fn name_validation_matches_golden_charset() {
        assert!(valid_scenario_name("circle_tex_label.v1"));
        assert!(!valid_scenario_name(""));
        assert!(!valid_scenario_name(".hidden"));
        assert!(!valid_scenario_name("UPPER"));
        assert!(!valid_scenario_name("has/slash"));
    }

    #[test]
    fn structural_asserts() {
        let outcome = RunOutcome::ok()
            .with_artifact("a.y4m", vec![1, 2, 3])
            .with_counter("frames", 4);
        assert!(StructuralAssert::ArtifactCountEq(1).check(&outcome).is_ok());
        assert!(
            StructuralAssert::ArtifactCountEq(2)
                .check(&outcome)
                .is_err()
        );
        assert!(StructuralAssert::ArtifactCountGe(1).check(&outcome).is_ok());
        assert!(StructuralAssert::ArtifactBytesLe(3).check(&outcome).is_ok());
        assert!(
            StructuralAssert::ArtifactBytesLe(2)
                .check(&outcome)
                .is_err()
        );
        assert!(StructuralAssert::NoEmptyArtifacts.check(&outcome).is_ok());
        assert!(
            StructuralAssert::CounterEq("frames", 4)
                .check(&outcome)
                .is_ok()
        );
        assert!(
            StructuralAssert::CounterGe("frames", 5)
                .check(&outcome)
                .is_err()
        );
        let empty = RunOutcome::ok().with_artifact("e", Vec::new());
        assert!(StructuralAssert::NoEmptyArtifacts.check(&empty).is_err());
    }
}
