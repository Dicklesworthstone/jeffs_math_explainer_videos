//! PG-1/PG-3/PG-4 production-front-door benchmark definitions and probes.
//!
//! These gates are intentionally process-level.  A Lumen microbenchmark is
//! not evidence about CLI startup, Reel publication, Studio supervision, or
//! the Python Reference ratio.  This module therefore binds every definition
//! to the exact `fmn` executable and drives that executable by absolute path.

use crate::perf::{
    Baseline, EvidenceKind, EvidenceRef, GateId, MeasurementBatch, MetricUnit, PerfError, Sample,
    require_compiled_cargo_profile, validate_producer_commit,
};
use fmn_anim::{Animation, Timeline, apply_matrix_2d, fade_in, show_creation};
use fmn_core::constants::BLUE_E;
use fmn_core::rng::RngRoot;
use fmn_hash::{Digest, Sha256, sha256};
use fmn_library::{Circle, NumberPlane, Text, VStyle};
use fmn_mobject::Stage;
use fmn_scene::export_timeline_bundle;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::io::{BufRead as _, BufReader, Read, Write as _};
use std::net::{Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Stable fixture/measurement definition schema.
pub const FRONTDOOR_DEFINITION_SCHEMA: &str = "fmn-perf-frontdoor-definition/1";
/// Stable phase-trace schema.
pub const FRONTDOOR_TRACE_SCHEMA: &str = "fmn-perf-frontdoor-trace/1";
/// Qualified Reference evidence accepted by PG-1.
pub const PG1_REFERENCE_SCHEMA: &str = "fmn-perf-pg1-reference/1";
/// Exact pinned Reference commit from the project contract.
pub const REFERENCE_COMMIT: &str = "6199a00d4c1b1127ebe45cb629c3f22538b10e13";
/// PG-1 retains nine required observations plus two invalid slots.
pub const PG1_SAMPLE_COUNT: usize = 11;
/// PG-3/4 retain 21 required observations plus three invalid slots.
pub const FRONTDOOR_SAMPLE_COUNT: usize = 24;
/// Fixed timeout for a small provenance probe.
pub const PROVENANCE_TIMEOUT: Duration = Duration::from_secs(10);
/// Fixed timeout for one production render or Studio launch.
pub const PROCESS_TIMEOUT: Duration = Duration::from_secs(120);

const BUILD_PROFILE: &str = "release-perf";
const THREAD_PROFILE: &str = "fixed-8";
const ENGINE: &str = "fast-cpu";
const MAX_EXECUTABLE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PROCESS_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_REFERENCE_BYTES: usize = 256 * 1024;

/// One permanent policy-catalog workload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontdoorScenario {
    /// G2 end-to-end wall ratio against the pinned Reference.
    OpeningClassG2,
    /// G4 end-to-end wall ratio against the pinned Reference.
    OpeningClassG4,
    /// Native 4K 2D export throughput.
    Export4k2d,
    /// Warm Studio 1080p preview throughput.
    Preview1080p,
    /// Fresh-process Studio first-frame latency.
    ColdCliFirstFrame,
    /// Warm 30-second scene trailing-edit-to-frame latency.
    TrailingEditToFrame,
}

impl FrontdoorScenario {
    /// All scenarios in policy-catalog order.
    pub const ALL: [Self; 6] = [
        Self::OpeningClassG2,
        Self::OpeningClassG4,
        Self::Export4k2d,
        Self::Preview1080p,
        Self::ColdCliFirstFrame,
        Self::TrailingEditToFrame,
    ];

    /// Stable policy spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OpeningClassG2 => "opening-class-g2",
            Self::OpeningClassG4 => "opening-class-g4",
            Self::Export4k2d => "export-4k-2d",
            Self::Preview1080p => "preview-1080p",
            Self::ColdCliFirstFrame => "cold-cli-first-frame",
            Self::TrailingEditToFrame => "trailing-edit-to-frame",
        }
    }

    /// Parse only the exact policy spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|scenario| scenario.name() == value)
    }

    /// Owning performance plane.
    #[must_use]
    pub const fn gate(self) -> GateId {
        match self {
            Self::OpeningClassG2 | Self::OpeningClassG4 => GateId::Pg1,
            Self::Export4k2d | Self::Preview1080p => GateId::Pg3,
            Self::ColdCliFirstFrame | Self::TrailingEditToFrame => GateId::Pg4,
        }
    }

    /// Unit stored in the common performance rig.
    #[must_use]
    pub const fn unit(self) -> MetricUnit {
        match self {
            Self::OpeningClassG2 | Self::OpeningClassG4 => MetricUnit::RatioPpm,
            Self::Export4k2d | Self::Preview1080p => MetricUnit::FramesPerSecondMilli,
            Self::ColdCliFirstFrame | Self::TrailingEditToFrame => MetricUnit::Nanoseconds,
        }
    }

    /// Fixed output surface.
    #[must_use]
    pub const fn output_mode(self) -> &'static str {
        match self {
            Self::OpeningClassG2 | Self::OpeningClassG4 => "ffmpeg-video",
            Self::Export4k2d => "ffmpeg-video",
            Self::Preview1080p | Self::ColdCliFirstFrame | Self::TrailingEditToFrame => {
                "studio-png-stream"
            }
        }
    }

    /// Declared cache state.
    #[must_use]
    pub const fn cache_state(self) -> &'static str {
        match self {
            Self::ColdCliFirstFrame => "fresh-empty",
            Self::TrailingEditToFrame => "verified-warm",
            Self::Preview1080p => "warm",
            Self::OpeningClassG2 | Self::OpeningClassG4 | Self::Export4k2d => "fresh-process",
        }
    }

    /// Fixed sample count from the policy catalog.
    #[must_use]
    pub const fn sample_count(self) -> usize {
        match self {
            Self::OpeningClassG2 | Self::OpeningClassG4 => PG1_SAMPLE_COUNT,
            Self::Export4k2d
            | Self::Preview1080p
            | Self::ColdCliFirstFrame
            | Self::TrailingEditToFrame => FRONTDOOR_SAMPLE_COUNT,
        }
    }

    /// Fixed output dimensions.
    #[must_use]
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Export4k2d => (3840, 2160),
            _ => (1920, 1080),
        }
    }
}

impl fmt::Display for FrontdoorScenario {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Compile provenance and content identity reported by one `fmn` artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FmnArtifactIdentity {
    /// Canonical executable path actually launched.
    pub executable: PathBuf,
    /// SHA-256 of the executable bytes.
    pub executable_digest: Digest,
    /// Package version.
    pub program_version: String,
    /// Embedded build identity.
    pub build_id: String,
    /// Rust target triple.
    pub target_triple: String,
    /// Actual Cargo profile from the artifact, never inferred from a path.
    pub cargo_profile: String,
    /// Crate-wide compiled SIMD tier.
    pub compiled_tier: String,
    /// Exact embedded `SUITE.lock` identity.
    pub suite_lock_digest: Digest,
}

/// Qualified, content-addressed Reference timing evidence for PG-1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg1Reference {
    /// Repository evidence path recorded in the current measurement.
    pub evidence: EvidenceRef,
    /// Pinned Reference commit.
    pub reference_commit: String,
    /// Host fingerprint shared with the current qualified run.
    pub host_fingerprint: Digest,
    /// Exact workload definition digest.
    pub benchmark_definition: Digest,
    /// Matching output lane.
    pub output_mode: String,
    /// Robust qualified Reference median in nanoseconds.
    pub median_ns: u64,
    /// Retained valid repetitions.
    pub valid_samples: usize,
    /// Retained invalid repetitions.
    pub invalid_samples: usize,
}

impl Pg1Reference {
    /// Parse bounded canonical Reference evidence and bind its exact bytes.
    ///
    /// # Errors
    /// Refuses unknown/duplicate fields, a non-canonical integer, an
    /// unqualified sample plan, or provenance outside the pinned contract.
    pub fn from_tsv(path: impl Into<String>, bytes: &[u8]) -> Result<Self, FrontdoorError> {
        if bytes.len() > MAX_REFERENCE_BYTES {
            return Err(FrontdoorError::Identity(format!(
                "Reference evidence is {} bytes; limit is {MAX_REFERENCE_BYTES}",
                bytes.len()
            )));
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| FrontdoorError::Identity("Reference evidence is not UTF-8".to_owned()))?;
        let mut lines = text.lines();
        if lines.next() != Some(PG1_REFERENCE_SCHEMA) {
            return Err(FrontdoorError::Identity(
                "Reference evidence schema is not fmn-perf-pg1-reference/1".to_owned(),
            ));
        }
        let mut fields = BTreeMap::new();
        for (index, line) in lines.enumerate() {
            let (name, value) = line.split_once('\t').ok_or_else(|| {
                FrontdoorError::Identity(format!("malformed Reference row {}", index + 2))
            })?;
            if name.is_empty() || value.is_empty() || fields.insert(name, value).is_some() {
                return Err(FrontdoorError::Identity(format!(
                    "ambiguous Reference row {}",
                    index + 2
                )));
            }
        }
        let expected = [
            "reference_commit",
            "host_fingerprint",
            "benchmark_definition",
            "output_mode",
            "median_ns",
            "valid_samples",
            "invalid_samples",
        ];
        if fields.len() != expected.len() || expected.iter().any(|name| !fields.contains_key(name))
        {
            return Err(FrontdoorError::Identity(
                "Reference evidence fields do not match the version-1 schema".to_owned(),
            ));
        }
        let field = |name| {
            fields.get(name).copied().ok_or_else(|| {
                FrontdoorError::Identity(format!(
                    "Reference evidence omitted required field {name:?}"
                ))
            })
        };
        let reference_commit = field("reference_commit")?.to_owned();
        if reference_commit != REFERENCE_COMMIT {
            return Err(FrontdoorError::Identity(format!(
                "Reference commit {reference_commit:?} is not the pinned {REFERENCE_COMMIT}"
            )));
        }
        let host_fingerprint = parse_digest(field("host_fingerprint")?, "host_fingerprint")?;
        let benchmark_definition =
            parse_digest(field("benchmark_definition")?, "benchmark_definition")?;
        let output_mode = field("output_mode")?.to_owned();
        if output_mode != "ffmpeg-video" {
            return Err(FrontdoorError::Identity(format!(
                "Reference output mode {output_mode:?} is not ffmpeg-video"
            )));
        }
        let median_ns = parse_u64(field("median_ns")?, "median_ns")?;
        let valid_samples = parse_usize(field("valid_samples")?, "valid_samples")?;
        let invalid_samples = parse_usize(field("invalid_samples")?, "invalid_samples")?;
        if median_ns == 0 || valid_samples < 9 || invalid_samples > 2 {
            return Err(FrontdoorError::Identity(
                "Reference evidence is not a qualified 9-valid/2-invalid observation".to_owned(),
            ));
        }
        Ok(Self {
            evidence: EvidenceRef::from_bytes(EvidenceKind::RawSamples, path, bytes)?,
            reference_commit,
            host_fingerprint,
            benchmark_definition,
            output_mode,
            median_ns,
            valid_samples,
            invalid_samples,
        })
    }
}

/// Exact benchmark definition for one process-level scenario.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontdoorDefinition {
    /// Policy scenario.
    pub scenario: FrontdoorScenario,
    /// Exact executable identity.
    pub artifact: FmnArtifactIdentity,
    /// Deterministic FMTL/1 source bytes.
    pub fixture_digest: Digest,
    /// Alternate trailing-edit source identity, absent outside that scenario.
    pub edit_fixture_digest: Option<Digest>,
    /// PG-1 Reference evidence, absent on PG-3/4.
    pub reference: Option<Pg1Reference>,
}

impl FrontdoorDefinition {
    /// Construct and validate one definition.
    ///
    /// # Errors
    /// Refuses a profile other than `release-perf`, a Reference on the wrong
    /// plane, or PG-1 without a qualified Reference observation.
    pub fn new(
        scenario: FrontdoorScenario,
        artifact: FmnArtifactIdentity,
        fixture_digest: Digest,
        reference: Option<Pg1Reference>,
    ) -> Result<Self, FrontdoorError> {
        if artifact.cargo_profile != BUILD_PROFILE {
            return Err(FrontdoorError::Identity(format!(
                "fmn artifact uses Cargo profile {:?}; {BUILD_PROFILE:?} is required",
                artifact.cargo_profile
            )));
        }
        match (scenario.gate(), reference.as_ref()) {
            (GateId::Pg1, None) => {
                return Err(FrontdoorError::Identity(
                    "PG-1 requires qualified Reference evidence".to_owned(),
                ));
            }
            (GateId::Pg1, Some(reference)) if reference.benchmark_definition != fixture_digest => {
                return Err(FrontdoorError::Identity(
                    "Reference workload digest does not match the FMTL fixture".to_owned(),
                ));
            }
            (GateId::Pg3 | GateId::Pg4, Some(_)) => {
                return Err(FrontdoorError::Identity(
                    "Reference evidence is only valid for PG-1".to_owned(),
                ));
            }
            _ => {}
        }
        let edit_fixture_digest = (scenario == FrontdoorScenario::TrailingEditToFrame)
            .then(|| trailing_edit_fixture([0.0, 0.5, 0.0]).map(|bytes| sha256(&bytes)))
            .transpose()?;
        Ok(Self {
            scenario,
            artifact,
            fixture_digest,
            edit_fixture_digest,
            reference,
        })
    }

    /// Canonical definition bytes.
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let (width, height) = self.scenario.dimensions();
        let mut output = String::new();
        for (name, value) in [
            ("schema", FRONTDOOR_DEFINITION_SCHEMA.to_owned()),
            ("gate", self.scenario.gate().name().to_owned()),
            ("scenario", self.scenario.name().to_owned()),
            ("unit", self.scenario.unit().name().to_owned()),
            ("build_profile", self.artifact.cargo_profile.clone()),
            ("build_id", self.artifact.build_id.clone()),
            ("target_triple", self.artifact.target_triple.clone()),
            ("compiled_tier", self.artifact.compiled_tier.clone()),
            (
                "suite_lock_digest",
                self.artifact.suite_lock_digest.to_string(),
            ),
            (
                "executable_digest",
                self.artifact.executable_digest.to_string(),
            ),
            ("fixture_digest", self.fixture_digest.to_string()),
            (
                "edit_fixture_digest",
                self.edit_fixture_digest
                    .map_or_else(|| "none".to_owned(), |value| value.to_string()),
            ),
            ("resolution", format!("{width}x{height}")),
            ("fps", "30".to_owned()),
            ("engine", ENGINE.to_owned()),
            ("thread_profile", THREAD_PROFILE.to_owned()),
            ("cache_state", self.scenario.cache_state().to_owned()),
            ("output_mode", self.scenario.output_mode().to_owned()),
            ("sample_count", self.scenario.sample_count().to_string()),
            (
                "reference_digest",
                self.reference.as_ref().map_or_else(
                    || "none".to_owned(),
                    |value| value.evidence.digest.to_string(),
                ),
            ),
        ] {
            let _ = writeln!(output, "{name}\t{value}");
        }
        output
    }

    /// Content address of canonical definition bytes.
    #[must_use]
    pub fn digest(&self) -> Digest {
        sha256(self.to_tsv().as_bytes())
    }

    /// Semantic configuration digest distinct from the executable/source.
    #[must_use]
    pub fn config_digest(&self) -> Digest {
        let mut hash = Sha256::new();
        hash.update(b"fmn-perf-frontdoor-config-v1");
        hash.update(self.scenario.name().as_bytes());
        let (width, height) = self.scenario.dimensions();
        hash.update(&width.to_le_bytes());
        hash.update(&height.to_le_bytes());
        hash.update(&30_u32.to_le_bytes());
        hash.update(THREAD_PROFILE.as_bytes());
        hash.update(self.scenario.cache_state().as_bytes());
        hash.update(self.scenario.output_mode().as_bytes());
        if let Some(digest) = self.edit_fixture_digest {
            hash.update(digest.as_bytes());
        }
        hash.finalize()
    }

    /// Validate a baseline before any process is spawned.
    ///
    /// # Errors
    /// Returns every identity mismatch as an explicit refusal.
    pub fn validate_baseline(&self, baseline: &Baseline) -> Result<(), FrontdoorError> {
        let key = &baseline.key;
        let mut differences = Vec::new();
        if baseline.policy.gate != self.scenario.gate() {
            differences.push("gate");
        }
        if baseline.policy.scenario != self.scenario.name() {
            differences.push("scenario");
        }
        if baseline.policy.unit != self.scenario.unit() {
            differences.push("unit");
        }
        if key.build_profile != BUILD_PROFILE {
            differences.push("build_profile");
        }
        if key.benchmark_definition != self.digest() {
            differences.push("benchmark_definition");
        }
        if key.config_digest != self.config_digest() {
            differences.push("config_digest");
        }
        if key.engine != ENGINE {
            differences.push("engine");
        }
        if key.tier != self.artifact.compiled_tier {
            differences.push("tier");
        }
        if key.thread_profile != THREAD_PROFILE {
            differences.push("thread_profile");
        }
        if key.cache_state != self.scenario.cache_state() {
            differences.push("cache_state");
        }
        if key.output_mode != self.scenario.output_mode() {
            differences.push("output_mode");
        }
        if key.suite_lock_digest != self.artifact.suite_lock_digest {
            differences.push("suite_lock_digest");
        }
        if !differences.is_empty() {
            return Err(FrontdoorError::Identity(format!(
                "{} baseline differs from its producer definition: {}",
                self.scenario,
                differences.join(", ")
            )));
        }
        Ok(())
    }
}

/// Build the exact one-second Opening-class FMTL fixture shared by PG-1/3/4.
///
/// # Errors
/// Returns native text, plane, animation, or FMTL export failures.
pub fn opening_class_fixture() -> Result<Vec<u8>, FrontdoorError> {
    let corpus = crate::scene_goldens::corpus();
    let title = Text::new("Functions become visible transformations")
        .font_size(44.0)
        .build(&corpus.book)
        .map_err(|error| FrontdoorError::Fixture(format!("title layout: {error}")))?
        .vmob
        .shifted([0.0, 3.2, 0.0]);
    let plane = NumberPlane::new()
        .build(&corpus.book)
        .map_err(|error| FrontdoorError::Fixture(format!("number plane: {error}")))?
        .into_vmob();
    let mut stage = Stage::new();
    let title = stage.add(title);
    let plane = stage.add(plane);
    stage
        .add_many_to_scene(&[plane, title])
        .map_err(|error| FrontdoorError::Fixture(format!("scene roots: {error}")))?;
    stage.set_stroke(plane, Some(BLUE_E), Some(1.25), None, None, true);

    let mut title_in = fade_in(&mut stage, title, [0.0, 0.0, 0.0], 1.0)
        .map_err(|error| FrontdoorError::Fixture(format!("title FadeIn: {error}")))?;
    title_in.update_rate_info(Some(0.5), None, None);
    let mut plane_in = show_creation(plane);
    plane_in.update_rate_info(Some(0.5), None, None);
    let mut plane_transform = apply_matrix_2d(&mut stage, plane, [[1.0, 0.55], [0.2, 1.0]])
        .map_err(|error| FrontdoorError::Fixture(format!("plane ApplyMatrix: {error}")))?;
    plane_transform.update_rate_info(Some(0.5), None, None);
    let target = stage
        .copy_family(title)
        .map_err(|error| FrontdoorError::Fixture(format!("title target: {error}")))?;
    stage.shift(target, [0.0, 0.15, 0.0]);
    let mut title_shift = fmn_anim::Transform::new(title, target);
    title_shift.update_rate_info(Some(0.5), None, None);

    let mut timeline =
        Timeline::new(30).map_err(|error| FrontdoorError::Fixture(format!("timeline: {error}")))?;
    timeline
        .play(vec![Box::new(title_in), Box::new(plane_in)])
        .and_then(|timeline| timeline.play(vec![Box::new(plane_transform), Box::new(title_shift)]))
        .map_err(|error| FrontdoorError::Fixture(format!("timeline play: {error}")))?;
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
        .map_err(|error| FrontdoorError::Fixture(format!("FMTL export: {error}")))
}

/// Build the exact source bytes owned by one front-door scenario.
///
/// # Errors
/// Returns the same bounded fixture errors as the underlying constructor.
pub fn frontdoor_fixture(scenario: FrontdoorScenario) -> Result<Vec<u8>, FrontdoorError> {
    match scenario {
        FrontdoorScenario::OpeningClassG2
        | FrontdoorScenario::OpeningClassG4
        | FrontdoorScenario::Export4k2d => opening_class_fixture(),
        FrontdoorScenario::Preview1080p | FrontdoorScenario::ColdCliFirstFrame => {
            primitive_preview_fixture()
        }
        FrontdoorScenario::TrailingEditToFrame => trailing_edit_fixture([0.5, 0.0, 0.0]),
    }
}

fn primitive_preview_fixture() -> Result<Vec<u8>, FrontdoorError> {
    let mut stage = Stage::new();
    let circle = stage.add(Circle::new().radius(0.9));
    stage
        .add_to_scene(circle)
        .map_err(|error| FrontdoorError::Fixture(format!("primitive scene root: {error}")))?;
    let target = stage
        .copy_family(circle)
        .map_err(|error| FrontdoorError::Fixture(format!("primitive target: {error}")))?;
    stage.shift(target, [0.6, 0.2, 0.0]);
    let mut transform = fmn_anim::Transform::new(circle, target);
    transform.update_rate_info(Some(1.0), None, None);
    let mut timeline = Timeline::new(30)
        .map_err(|error| FrontdoorError::Fixture(format!("primitive timeline: {error}")))?;
    timeline
        .play(vec![Box::new(transform)])
        .map_err(|error| FrontdoorError::Fixture(format!("primitive play: {error}")))?;
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
        .map_err(|error| FrontdoorError::Fixture(format!("primitive FMTL export: {error}")))
}

/// Probe one executable by absolute path and verify its self-reported compile
/// identity against the executable bytes.
///
/// # Errors
/// Refuses non-files, oversized artifacts, nonzero/timeout results, malformed
/// robot output, and profiles other than `expected_profile`.
pub fn probe_fmn_artifact(
    executable: &Path,
    expected_profile: &str,
) -> Result<FmnArtifactIdentity, FrontdoorError> {
    let executable = fs::canonicalize(executable)
        .map_err(|error| FrontdoorError::Io(format!("resolve fmn executable: {error}")))?;
    let metadata = fs::metadata(&executable)
        .map_err(|error| FrontdoorError::Io(format!("stat fmn executable: {error}")))?;
    if !metadata.is_file() || metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(FrontdoorError::Identity(format!(
            "fmn executable must be a regular file of at most {MAX_EXECUTABLE_BYTES} bytes"
        )));
    }
    let bytes = fs::read(&executable)
        .map_err(|error| FrontdoorError::Io(format!("read fmn executable: {error}")))?;
    let output = run_bounded(
        &executable,
        &["--robot", "--version"],
        None,
        PROVENANCE_TIMEOUT,
    )?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(FrontdoorError::Process(format!(
            "fmn --robot --version exited {:?} with stderr {:?}",
            output.status.code(),
            bounded_text(&output.stderr)
        )));
    }
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_| FrontdoorError::Protocol("version output is not UTF-8".to_owned()))?;
    if stdout.lines().count() != 1 || !stdout.contains("\"kind\":\"version\"") {
        return Err(FrontdoorError::Protocol(
            "version probe did not emit exactly one version record".to_owned(),
        ));
    }
    let field = |name| {
        json_string_field(stdout, name).ok_or_else(|| {
            FrontdoorError::Protocol(format!("version record omits string field {name:?}"))
        })
    };
    let cargo_profile = field("cargo_profile")?.to_owned();
    if cargo_profile != expected_profile {
        return Err(FrontdoorError::Identity(format!(
            "fmn artifact reports Cargo profile {cargo_profile:?}; expected {expected_profile:?}"
        )));
    }
    Ok(FmnArtifactIdentity {
        executable,
        executable_digest: sha256(&bytes),
        program_version: field("program_version")?.to_owned(),
        build_id: field("build_id")?.to_owned(),
        target_triple: field("target_triple")?.to_owned(),
        cargo_profile,
        compiled_tier: field("compiled_tier")?.to_owned(),
        suite_lock_digest: parse_digest(field("suite_lock_digest")?, "suite_lock_digest")?,
    })
}

/// Exact integer PG-1 ratio conversion.
#[must_use]
pub fn ratio_sample(elapsed_ns: u128, reference_ns: u64) -> Sample {
    if elapsed_ns == 0 || reference_ns == 0 {
        return Sample::invalid(0, "elapsed/reference time was zero");
    }
    let value = elapsed_ns
        .saturating_mul(1_000_000)
        .checked_div(u128::from(reference_ns))
        .and_then(|value| u64::try_from(value).ok());
    value.map_or_else(
        || Sample::invalid(u64::MAX, "ratio-ppm overflow"),
        Sample::valid,
    )
}

/// Exact integer PG-3 throughput conversion.
#[must_use]
pub fn fps_milli_sample(frames: u64, elapsed_ns: u128) -> Sample {
    if frames == 0 || elapsed_ns == 0 {
        return Sample::invalid(0, "frame count or elapsed time was zero");
    }
    let value = u128::from(frames)
        .saturating_mul(1_000_000_000_000)
        .checked_div(elapsed_ns)
        .and_then(|value| u64::try_from(value).ok());
    value.map_or_else(
        || Sample::invalid(u64::MAX, "fps-milli overflow"),
        Sample::valid,
    )
}

/// Canonical output of one front-door measurement before publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontdoorArtifacts {
    /// Common-rig batch containing all bounded repetitions.
    pub batch: MeasurementBatch,
    /// Structured process/phase trace named by `batch.evidence`.
    pub trace_tsv: String,
    /// Exact FMTL source used by the child process.
    pub fixture_digest: Digest,
}

/// Measure one PG-1/3/4 scenario through the shipped `fmn` process.
///
/// `work_root` must not exist.  It is created once and deliberately retained
/// with the source and product artifacts so an observation is inspectable.
/// `trace_path` and the derived fixture evidence path must be valid common-rig
/// repository artifact paths.  PG-1 alone requires `reference_evidence`.
///
/// # Errors
/// Refuses identity/provenance drift before timing, existing output paths,
/// child failures, malformed robot/HTTP records, or a postflight host change.
pub fn measure_frontdoor(
    baseline: &Baseline,
    producer_commit: &str,
    fmn_executable: &Path,
    work_root: &Path,
    trace_path: impl Into<String>,
    reference_evidence: Option<(&str, &[u8])>,
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<FrontdoorArtifacts, FrontdoorError> {
    let scenario = FrontdoorScenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        FrontdoorError::Identity(format!(
            "unsupported front-door scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    validate_producer_commit(producer_commit)?;
    require_compiled_cargo_profile(BUILD_PROFILE)?;
    let artifact = probe_fmn_artifact(fmn_executable, BUILD_PROFILE)?;
    if artifact.build_id != format!("git:{producer_commit}") {
        return Err(FrontdoorError::Identity(format!(
            "fmn build ID {:?} does not match producer commit {producer_commit}",
            artifact.build_id
        )));
    }

    let fixture = frontdoor_fixture(scenario)?;
    let fixture_digest = sha256(&fixture);
    let reference = match reference_evidence {
        Some((path, bytes)) => Some(Pg1Reference::from_tsv(path, bytes)?),
        None => None,
    };
    let definition = FrontdoorDefinition::new(scenario, artifact, fixture_digest, reference)?;
    definition.validate_baseline(baseline)?;
    if let Some(reference) = definition.reference.as_ref()
        && reference.host_fingerprint != baseline.key.host_fingerprint
    {
        return Err(FrontdoorError::Identity(
            "Reference and FrankenManim host fingerprints differ".to_owned(),
        ));
    }
    let uses_ffmpeg = scenario.output_mode() == "ffmpeg-video";
    if uses_ffmpeg && baseline.key.external_tool_fingerprint.is_none() {
        return Err(FrontdoorError::Identity(
            "ffmpeg-video baseline omits the external tool fingerprint".to_owned(),
        ));
    }
    if !uses_ffmpeg && baseline.key.external_tool_fingerprint.is_some() {
        return Err(FrontdoorError::Identity(
            "native/Studio front-door baseline unexpectedly names an external tool".to_owned(),
        ));
    }

    let trace_path = trace_path.into();
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    if work_root.exists() {
        return Err(FrontdoorError::Io(format!(
            "work root {:?} already exists; refusing to overwrite it",
            work_root
        )));
    }
    fs::create_dir(work_root)
        .map_err(|error| FrontdoorError::Io(format!("create work root: {error}")))?;
    let source_path = work_root.join("frontdoor-scene.fmtl");
    write_new_file(&source_path, &fixture, "FMTL fixture")?;
    let fixture_path = source_path
        .to_str()
        .ok_or_else(|| FrontdoorError::Io("FMTL fixture path is not valid UTF-8".to_owned()))?;
    let fixture_evidence = EvidenceRef::from_bytes(EvidenceKind::Golden, fixture_path, &fixture)?;

    let (key, mut evidence) = crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    evidence.push(fixture_evidence);
    if scenario == FrontdoorScenario::TrailingEditToFrame {
        let edited = trailing_edit_fixture([0.0, 0.5, 0.0])?;
        let edited_path = work_root.join("frontdoor-scene-edited.fmtl");
        write_new_file(&edited_path, &edited, "edited FMTL fixture")?;
        evidence.push(EvidenceRef::from_bytes(
            EvidenceKind::Golden,
            utf8_path(&edited_path, "edited FMTL fixture")?,
            &edited,
        )?);
    }
    if let Some(reference) = definition.reference.as_ref() {
        evidence.push(reference.evidence.clone());
    }
    let mut observations = Vec::with_capacity(scenario.sample_count());
    let samples = match scenario {
        FrontdoorScenario::OpeningClassG2 | FrontdoorScenario::OpeningClassG4 => {
            measure_render_samples(
                &definition,
                &source_path,
                work_root,
                "video",
                scenario.sample_count(),
                baseline.key.external_tool_fingerprint,
                &mut observations,
            )?
        }
        FrontdoorScenario::Export4k2d => measure_render_samples(
            &definition,
            &source_path,
            work_root,
            "video",
            scenario.sample_count(),
            baseline.key.external_tool_fingerprint,
            &mut observations,
        )?,
        FrontdoorScenario::Preview1080p => measure_preview_samples(
            &definition,
            &source_path,
            work_root,
            scenario.sample_count(),
            &mut observations,
        )?,
        FrontdoorScenario::ColdCliFirstFrame => measure_cold_samples(
            &definition,
            &source_path,
            work_root,
            scenario.sample_count(),
            &mut observations,
        )?,
        FrontdoorScenario::TrailingEditToFrame => measure_trailing_edit_samples(
            &definition,
            &source_path,
            work_root,
            scenario.sample_count(),
            &mut observations,
        )?,
    };
    let trace_tsv = render_frontdoor_trace(&definition, &observations, &samples);
    evidence.push(EvidenceRef::from_bytes(
        EvidenceKind::PhaseTrace,
        trace_path,
        trace_tsv.as_bytes(),
    )?);
    let batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples,
        evidence,
    };
    let _ = batch.to_tsv()?;
    Ok(FrontdoorArtifacts {
        batch,
        trace_tsv,
        fixture_digest,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbeObservation {
    index: usize,
    phase: &'static str,
    elapsed_ns: u128,
    frames: u64,
    closure_digest: Option<Digest>,
    external_tool_digest: Option<Digest>,
    source_digest: Digest,
    state: &'static str,
}

fn measure_render_samples(
    definition: &FrontdoorDefinition,
    source_path: &Path,
    work_root: &Path,
    format: &str,
    sample_count: usize,
    expected_external_tool: Option<Digest>,
    observations: &mut Vec<ProbeObservation>,
) -> Result<Vec<Sample>, FrontdoorError> {
    let reference_median_ns = if definition.scenario.gate() == GateId::Pg1 {
        Some(
            definition
                .reference
                .as_ref()
                .ok_or_else(|| {
                    FrontdoorError::Identity(
                        "PG-1 definition omitted required Reference evidence".to_owned(),
                    )
                })?
                .median_ns,
        )
    } else {
        None
    };
    let mut samples = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let output_root = work_root.join(format!("sample-{index:02}"));
        fs::create_dir(&output_root)
            .map_err(|error| FrontdoorError::Io(format!("create render output root: {error}")))?;
        let observation = render_once(
            definition,
            source_path,
            &output_root,
            format,
            expected_external_tool,
            index,
        )?;
        if let Some(expected) = reference_median_ns {
            samples.push(ratio_sample(observation.elapsed_ns, expected));
        } else {
            samples.push(fps_milli_sample(observation.frames, observation.elapsed_ns));
        }
        observations.push(observation);
    }
    Ok(samples)
}

fn render_once(
    definition: &FrontdoorDefinition,
    source_path: &Path,
    output_root: &Path,
    format: &str,
    expected_external_tool: Option<Digest>,
    index: usize,
) -> Result<ProbeObservation, FrontdoorError> {
    let (width, height) = definition.scenario.dimensions();
    let resolution = format!("{width}x{height}");
    let source = utf8_path(source_path, "FMTL source")?;
    let output = utf8_path(output_root, "render output root")?;
    let args = vec![
        "--robot".to_owned(),
        "--format".to_owned(),
        format.to_owned(),
        "--resolution".to_owned(),
        resolution,
        "--fps".to_owned(),
        "30".to_owned(),
        "--threads".to_owned(),
        "8".to_owned(),
        "--video_dir".to_owned(),
        output.to_owned(),
        source.to_owned(),
    ];
    let started = Instant::now();
    let output = run_bounded(
        &definition.artifact.executable,
        &args,
        None,
        PROCESS_TIMEOUT,
    )?;
    let elapsed_ns = started.elapsed().as_nanos();
    let record = require_render_record(&output)?;
    let frames = json_u64_field(record, "frames")
        .ok_or_else(|| FrontdoorError::Protocol("render record omits frames".to_owned()))?;
    let closure_digest = json_string_field(record, "closure_digest")
        .map(|value| parse_digest(value, "closure_digest"))
        .transpose()?;
    let external_tool_digest = json_string_field(record, "sha256")
        .map(|value| parse_digest(value, "ffmpeg sha256"))
        .transpose()?;
    if definition.scenario.output_mode() == "ffmpeg-video" {
        if external_tool_digest != expected_external_tool {
            return Err(FrontdoorError::Protocol(
                "render ffmpeg identity differs from the baseline".to_owned(),
            ));
        }
    } else if external_tool_digest.is_some() {
        return Err(FrontdoorError::Protocol(
            "native export unexpectedly reported an external tool".to_owned(),
        ));
    }
    Ok(ProbeObservation {
        index,
        phase: "render-process",
        elapsed_ns,
        frames,
        closure_digest,
        external_tool_digest,
        source_digest: definition.fixture_digest,
        state: "fresh-process",
    })
}

fn measure_preview_samples(
    definition: &FrontdoorDefinition,
    source_path: &Path,
    work_root: &Path,
    sample_count: usize,
    observations: &mut Vec<ProbeObservation>,
) -> Result<Vec<Sample>, FrontdoorError> {
    let cache_root = work_root.join("preview-cache");
    let mut studio = StudioSession::start(definition, source_path, &cache_root)?;
    let mut samples = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let frame = u64::try_from(index % 30).map_err(|_| {
            FrontdoorError::Identity("preview frame index does not fit u64".to_owned())
        })?;
        let started = Instant::now();
        let response = studio.post_scrub(frame, false)?;
        let elapsed_ns = started.elapsed().as_nanos();
        if !response.contains(&format!("\"frame_index\":{frame}")) {
            studio.abort();
            return Err(FrontdoorError::Protocol(format!(
                "Studio scrub response omitted frame {frame}"
            )));
        }
        samples.push(fps_milli_sample(1, elapsed_ns));
        observations.push(ProbeObservation {
            index,
            phase: "studio-scrub",
            elapsed_ns,
            frames: 1,
            closure_digest: None,
            external_tool_digest: None,
            source_digest: definition.fixture_digest,
            state: "warm",
        });
    }
    studio.finish()?;
    Ok(samples)
}

fn measure_cold_samples(
    definition: &FrontdoorDefinition,
    source_path: &Path,
    work_root: &Path,
    sample_count: usize,
    observations: &mut Vec<ProbeObservation>,
) -> Result<Vec<Sample>, FrontdoorError> {
    let mut samples = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let cache_root = work_root.join(format!("cold-cache-{index:02}"));
        if cache_root.exists() {
            return Err(FrontdoorError::Io(format!(
                "cold cache {:?} already exists",
                cache_root
            )));
        }
        let mut studio = StudioSession::start(definition, source_path, &cache_root)?;
        let elapsed_ns = studio.ready_elapsed_ns;
        samples.push(nanoseconds_sample(elapsed_ns));
        observations.push(ProbeObservation {
            index,
            phase: "studio-first-frame",
            elapsed_ns,
            frames: 1,
            closure_digest: None,
            external_tool_digest: None,
            source_digest: definition.fixture_digest,
            state: "fresh-empty",
        });
        studio.finish()?;
    }
    Ok(samples)
}

fn measure_trailing_edit_samples(
    definition: &FrontdoorDefinition,
    source_path: &Path,
    work_root: &Path,
    sample_count: usize,
    observations: &mut Vec<ProbeObservation>,
) -> Result<Vec<Sample>, FrontdoorError> {
    let cache_root = work_root.join("trailing-edit-cache");
    let mut studio = StudioSession::start(definition, source_path, &cache_root)?;
    let response = studio.post_scrub(899, true)?;
    if !response.contains("\"frame_index\":899") {
        studio.abort();
        return Err(FrontdoorError::Protocol(
            "Studio could not establish the trailing frame before edit".to_owned(),
        ));
    }
    let variants = [
        trailing_edit_fixture([0.0, 0.5, 0.0])?,
        trailing_edit_fixture([0.5, 0.0, 0.0])?,
    ];
    let mut samples = Vec::with_capacity(sample_count);
    for index in 0..sample_count {
        let started = Instant::now();
        fs::write(source_path, &variants[index % variants.len()])
            .map_err(|error| FrontdoorError::Io(format!("write trailing edit: {error}")))?;
        let response = studio.post_restart()?;
        let elapsed_ns = started.elapsed().as_nanos();
        if !response.contains("\"frame_index\":899") {
            studio.abort();
            return Err(FrontdoorError::Protocol(
                "Studio restart did not publish the retained trailing frame".to_owned(),
            ));
        }
        samples.push(nanoseconds_sample(elapsed_ns));
        observations.push(ProbeObservation {
            index,
            phase: "studio-trailing-restart",
            elapsed_ns,
            frames: 1,
            closure_digest: None,
            external_tool_digest: None,
            source_digest: sha256(&variants[index % variants.len()]),
            state: "verified-warm",
        });
    }
    studio.finish()?;
    Ok(samples)
}

fn trailing_edit_fixture(shift: [f64; 3]) -> Result<Vec<u8>, FrontdoorError> {
    let mut stage = Stage::new();
    let circle = stage.add(Circle::new().radius(0.9));
    stage
        .add_to_scene(circle)
        .map_err(|error| FrontdoorError::Fixture(format!("trailing scene root: {error}")))?;
    let target = stage
        .copy_family(circle)
        .map_err(|error| FrontdoorError::Fixture(format!("trailing target: {error}")))?;
    stage.shift(target, shift);
    let mut transform = fmn_anim::Transform::new(circle, target);
    transform.update_rate_info(Some(1.0), None, None);
    let mut timeline = Timeline::new(30)
        .map_err(|error| FrontdoorError::Fixture(format!("trailing timeline: {error}")))?;
    timeline
        .wait(29.0)
        .and_then(|timeline| timeline.play(vec![Box::new(transform)]))
        .map_err(|error| FrontdoorError::Fixture(format!("trailing timeline steps: {error}")))?;
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
        .map_err(|error| FrontdoorError::Fixture(format!("trailing FMTL export: {error}")))
}

fn nanoseconds_sample(elapsed_ns: u128) -> Sample {
    if elapsed_ns == 0 {
        Sample::invalid(0, "monotonic clock reported zero elapsed nanoseconds")
    } else {
        u64::try_from(elapsed_ns).map_or_else(
            |_| Sample::invalid(u64::MAX, "nanosecond sample overflow"),
            Sample::valid,
        )
    }
}

struct StudioSession {
    child: Child,
    authority: String,
    query: String,
    ready_elapsed_ns: u128,
}

impl StudioSession {
    fn start(
        definition: &FrontdoorDefinition,
        source_path: &Path,
        cache_root: &Path,
    ) -> Result<Self, FrontdoorError> {
        if cache_root.exists() {
            return Err(FrontdoorError::Io(format!(
                "Studio cache {:?} already exists",
                cache_root
            )));
        }
        let (width, height) = definition.scenario.dimensions();
        let resolution = format!("{width}x{height}");
        let source = utf8_path(source_path, "Studio source")?;
        let cache = utf8_path(cache_root, "Studio cache")?;
        let started = Instant::now();
        let mut child = Command::new(&definition.artifact.executable)
            .args([
                "studio",
                "--robot",
                "--no-browser",
                "--port",
                "0",
                "--resolution",
                &resolution,
                "--fps",
                "30",
                "--threads",
                "8",
                "--cache_dir",
                cache,
                source,
                "FrontdoorScene",
            ])
            .env_remove("PYTHONHOME")
            .env_remove("PYTHONPATH")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| FrontdoorError::Io(format!("launch Studio: {error}")))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| FrontdoorError::Process("Studio stdout pipe is absent".to_owned()))?;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("fmn-perf-studio-ready".to_owned())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                let result = reader.read_line(&mut line).map(|_| line);
                let _ = sender.send(result);
            })
            .map_err(|error| FrontdoorError::Process(format!("Studio ready reader: {error}")))?;
        let ready = match receiver.recv_timeout(PROCESS_TIMEOUT) {
            Ok(Ok(line)) => line,
            Ok(Err(error)) => {
                abort_child(&mut child);
                return Err(FrontdoorError::Io(format!("read Studio ready: {error}")));
            }
            Err(_) => {
                abort_child(&mut child);
                return Err(FrontdoorError::Process(format!(
                    "Studio ready exceeded {} ms",
                    PROCESS_TIMEOUT.as_millis()
                )));
            }
        };
        let ready_elapsed_ns = started.elapsed().as_nanos();
        if !ready.contains("\"kind\":\"studio_ready\"") {
            abort_child(&mut child);
            return Err(FrontdoorError::Protocol(format!(
                "Studio did not emit a ready record: {:?}",
                ready.chars().take(160).collect::<String>()
            )));
        }
        let url = json_string_field(&ready, "url")
            .ok_or_else(|| FrontdoorError::Protocol("Studio ready omits url".to_owned()))?;
        let location = url.strip_prefix("http://").ok_or_else(|| {
            FrontdoorError::Protocol("Studio URL is not loopback HTTP".to_owned())
        })?;
        let (authority, target) = location.split_once('/').ok_or_else(|| {
            FrontdoorError::Protocol("Studio URL omits capability target".to_owned())
        })?;
        let (_, query) = target.split_once('?').ok_or_else(|| {
            FrontdoorError::Protocol("Studio URL omits capability query".to_owned())
        })?;
        Ok(Self {
            child,
            authority: authority.to_owned(),
            query: query.to_owned(),
            ready_elapsed_ns,
        })
    }

    fn post_scrub(&mut self, frame: u64, commit: bool) -> Result<String, FrontdoorError> {
        self.post(
            &format!("/api/scrub?{}", self.query),
            format!("frame={frame}&commit={commit}").as_bytes(),
        )
    }

    fn post_restart(&mut self) -> Result<String, FrontdoorError> {
        self.post(&format!("/api/restart?{}", self.query), b"")
    }

    fn post(&mut self, target: &str, body: &[u8]) -> Result<String, FrontdoorError> {
        let mut stream = TcpStream::connect(&self.authority)
            .map_err(|error| FrontdoorError::Io(format!("connect Studio: {error}")))?;
        stream
            .set_read_timeout(Some(PROCESS_TIMEOUT))
            .and_then(|()| stream.set_write_timeout(Some(PROCESS_TIMEOUT)))
            .map_err(|error| FrontdoorError::Io(format!("set Studio timeout: {error}")))?;
        write!(
            stream,
            "POST {target} HTTP/1.1\r\nHost: {}\r\nOrigin: http://{}\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.authority,
            self.authority,
            body.len()
        )
        .and_then(|()| stream.write_all(body))
        .map_err(|error| FrontdoorError::Io(format!("write Studio request: {error}")))?;
        let mut response = Vec::new();
        (&mut stream)
            .take((MAX_PROCESS_OUTPUT_BYTES + 1) as u64)
            .read_to_end(&mut response)
            .map_err(|error| FrontdoorError::Io(format!("read Studio response: {error}")))?;
        let _ = stream.shutdown(Shutdown::Both);
        if response.len() > MAX_PROCESS_OUTPUT_BYTES {
            return Err(FrontdoorError::Protocol(
                "Studio response exceeds the process-output bound".to_owned(),
            ));
        }
        let response = String::from_utf8(response)
            .map_err(|_| FrontdoorError::Protocol("Studio response is not UTF-8".to_owned()))?;
        if !response.starts_with("HTTP/1.1 200 OK") {
            return Err(FrontdoorError::Protocol(format!(
                "Studio returned {:?}",
                response.chars().take(160).collect::<String>()
            )));
        }
        Ok(response)
    }

    fn finish(&mut self) -> Result<(), FrontdoorError> {
        drop(self.child.stdin.take());
        let deadline = Instant::now() + PROVENANCE_TIMEOUT;
        let status = loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| FrontdoorError::Io(format!("poll Studio shutdown: {error}")))?
            {
                break status;
            }
            if Instant::now() >= deadline {
                self.abort();
                return Err(FrontdoorError::Process(
                    "Studio did not shut down after stdin EOF".to_owned(),
                ));
            }
            std::thread::sleep(Duration::from_millis(2));
        };
        let stderr = read_bounded_pipe(self.child.stderr.take(), "Studio stderr")?;
        if !status.success() || !stderr.is_empty() {
            return Err(FrontdoorError::Process(format!(
                "Studio shutdown exited {:?} with stderr {:?}",
                status.code(),
                bounded_text(&stderr)
            )));
        }
        Ok(())
    }

    fn abort(&mut self) {
        abort_child(&mut self.child);
    }
}

impl Drop for StudioSession {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            abort_child(&mut self.child);
        }
    }
}

fn abort_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn require_render_record(output: &BoundedOutput) -> Result<&str, FrontdoorError> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(FrontdoorError::Process(format!(
            "fmn render exited {:?} with stderr {:?}",
            output.status.code(),
            bounded_text(&output.stderr)
        )));
    }
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_| FrontdoorError::Protocol("render output is not UTF-8".to_owned()))?;
    if stdout.lines().count() != 1 || !stdout.contains("\"kind\":\"render\"") {
        return Err(FrontdoorError::Protocol(format!(
            "render emitted an unexpected record set: {:?}",
            stdout.chars().take(160).collect::<String>()
        )));
    }
    Ok(stdout.trim_end())
}

fn render_frontdoor_trace(
    definition: &FrontdoorDefinition,
    observations: &[ProbeObservation],
    samples: &[Sample],
) -> String {
    let mut trace = String::new();
    for (name, value) in [
        ("schema", FRONTDOOR_TRACE_SCHEMA.to_owned()),
        ("scenario", definition.scenario.name().to_owned()),
        ("definition_digest", definition.digest().to_string()),
        ("fixture_digest", definition.fixture_digest.to_string()),
        (
            "edit_fixture_digest",
            definition
                .edit_fixture_digest
                .map_or_else(|| "none".to_owned(), |value| value.to_string()),
        ),
        (
            "executable_digest",
            definition.artifact.executable_digest.to_string(),
        ),
        ("build_id", definition.artifact.build_id.clone()),
        ("target_triple", definition.artifact.target_triple.clone()),
        ("cargo_profile", definition.artifact.cargo_profile.clone()),
        ("compiled_tier", definition.artifact.compiled_tier.clone()),
        ("cache_state", definition.scenario.cache_state().to_owned()),
        ("output_mode", definition.scenario.output_mode().to_owned()),
        ("sample_count", observations.len().to_string()),
        (
            "reference_digest",
            definition.reference.as_ref().map_or_else(
                || "none".to_owned(),
                |reference| reference.evidence.digest.to_string(),
            ),
        ),
    ] {
        let _ = writeln!(trace, "{name}\t{value}");
    }
    for (observation, sample) in observations.iter().zip(samples) {
        let _ = writeln!(
            trace,
            "sample\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            observation.index,
            observation.phase,
            observation.elapsed_ns,
            observation.frames,
            sample.value,
            sample.invalid_reason.as_deref().unwrap_or("none"),
            observation
                .closure_digest
                .map_or_else(|| "none".to_owned(), |value| value.to_string()),
            observation
                .external_tool_digest
                .map_or_else(|| "none".to_owned(), |value| value.to_string()),
            observation.source_digest,
            observation.state,
        );
    }
    trace
}

fn write_new_file(path: &Path, bytes: &[u8], label: &str) -> Result<(), FrontdoorError> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| FrontdoorError::Io(format!("create {label}: {error}")))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| FrontdoorError::Io(format!("write {label}: {error}")))
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str, FrontdoorError> {
    path.to_str()
        .ok_or_else(|| FrontdoorError::Io(format!("{label} path is not valid UTF-8")))
}

fn json_u64_field(record: &str, field: &str) -> Option<u64> {
    let prefix = format!("\"{field}\":");
    let tail = record.split_once(&prefix)?.1;
    let digits = tail
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    (digits != 0).then(|| tail[..digits].parse().ok()).flatten()
}

/// Errors from process-level performance producers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontdoorError {
    /// Build, host, workload, or baseline identity mismatch.
    Identity(String),
    /// Deterministic scene fixture construction failed.
    Fixture(String),
    /// Child process failed or exceeded a bound.
    Process(String),
    /// Robot/Studio protocol output was malformed.
    Protocol(String),
    /// Host filesystem or socket I/O failed.
    Io(String),
    /// Common performance-rig failure.
    Perf(PerfError),
}

impl fmt::Display for FrontdoorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity(detail) => write!(formatter, "front-door identity: {detail}"),
            Self::Fixture(detail) => write!(formatter, "front-door fixture: {detail}"),
            Self::Process(detail) => write!(formatter, "front-door process: {detail}"),
            Self::Protocol(detail) => write!(formatter, "front-door protocol: {detail}"),
            Self::Io(detail) => write!(formatter, "front-door I/O: {detail}"),
            Self::Perf(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FrontdoorError {}

impl From<PerfError> for FrontdoorError {
    fn from(error: PerfError) -> Self {
        Self::Perf(error)
    }
}

struct BoundedOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_bounded<S: AsRef<OsStr>>(
    executable: &Path,
    args: &[S],
    cwd: Option<&Path>,
    timeout: Duration,
) -> Result<BoundedOutput, FrontdoorError> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONPATH");
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let mut child = command
        .spawn()
        .map_err(|error| FrontdoorError::Io(format!("launch {:?}: {error}", executable)))?;
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| FrontdoorError::Io(format!("poll child: {error}")))?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(FrontdoorError::Process(format!(
                "process exceeded {} ms",
                timeout.as_millis()
            )));
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let stdout = read_bounded_pipe(child.stdout.take(), "stdout")?;
    let stderr = read_bounded_pipe(child.stderr.take(), "stderr")?;
    Ok(BoundedOutput {
        status,
        stdout,
        stderr,
    })
}

fn read_bounded_pipe(pipe: Option<impl Read>, label: &str) -> Result<Vec<u8>, FrontdoorError> {
    let mut bytes = Vec::new();
    pipe.ok_or_else(|| FrontdoorError::Process(format!("child {label} pipe is absent")))?
        .take((MAX_PROCESS_OUTPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| FrontdoorError::Io(format!("read child {label}: {error}")))?;
    if bytes.len() > MAX_PROCESS_OUTPUT_BYTES {
        return Err(FrontdoorError::Protocol(format!(
            "child {label} exceeds {MAX_PROCESS_OUTPUT_BYTES} bytes"
        )));
    }
    Ok(bytes)
}

fn json_string_field<'a>(record: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("\"{field}\":\"");
    record
        .split_once(&prefix)
        .and_then(|(_, tail)| tail.split_once('"'))
        .map(|(value, _)| value)
}

fn parse_digest(value: &str, field: &str) -> Result<Digest, FrontdoorError> {
    Digest::from_hex(value)
        .map_err(|error| FrontdoorError::Identity(format!("bad {field}: {error}")))
}

fn parse_u64(value: &str, field: &str) -> Result<u64, FrontdoorError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| FrontdoorError::Identity(format!("{field} is not a canonical u64")))?;
    if parsed.to_string() != value {
        return Err(FrontdoorError::Identity(format!(
            "{field} is not a canonical u64"
        )));
    }
    Ok(parsed)
}

fn parse_usize(value: &str, field: &str) -> Result<usize, FrontdoorError> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| FrontdoorError::Identity(format!("{field} is not a canonical usize")))?;
    if parsed.to_string() != value {
        return Err(FrontdoorError::Identity(format!(
            "{field} is not a canonical usize"
        )));
    }
    Ok(parsed)
}

fn bounded_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf::{BenchmarkKey, Direction, Enforcement, GatePolicy, GateScope, Verdict};
    use fmn_scene::TimelineBundle;

    fn digest(byte: u8) -> Digest {
        Digest::from_bytes([byte; 32])
    }

    fn artifact() -> FmnArtifactIdentity {
        FmnArtifactIdentity {
            executable: PathBuf::from("/opt/fmn/bin/fmn"),
            executable_digest: digest(1),
            program_version: "0.3.0".to_owned(),
            build_id: "git:0123456789abcdef0123456789abcdef01234567".to_owned(),
            target_triple: "x86_64-unknown-linux-gnu".to_owned(),
            cargo_profile: BUILD_PROFILE.to_owned(),
            compiled_tier: "portable".to_owned(),
            suite_lock_digest: digest(2),
        }
    }

    fn reference_bytes(fixture: Digest) -> String {
        format!(
            "{PG1_REFERENCE_SCHEMA}\n\
             reference_commit\t{REFERENCE_COMMIT}\n\
             host_fingerprint\t{}\n\
             benchmark_definition\t{fixture}\n\
             output_mode\tffmpeg-video\n\
             median_ns\t2000000000\n\
             valid_samples\t9\n\
             invalid_samples\t2\n",
            digest(3),
        )
    }

    #[test]
    fn scenario_catalog_matches_the_six_policy_rows() {
        assert_eq!(FrontdoorScenario::ALL.len(), 6);
        for scenario in FrontdoorScenario::ALL {
            assert_eq!(FrontdoorScenario::parse(scenario.name()), Some(scenario));
            assert!(matches!(
                scenario.gate(),
                GateId::Pg1 | GateId::Pg3 | GateId::Pg4
            ));
            assert!(matches!(
                scenario.sample_count(),
                PG1_SAMPLE_COUNT | FRONTDOOR_SAMPLE_COUNT
            ));
        }
        assert_eq!(FrontdoorScenario::parse("preview"), None);
        assert_eq!(FrontdoorScenario::parse("preview-1080p\n"), None);
    }

    #[test]
    fn pg1_reference_is_exact_pinned_and_content_addressed() {
        let fixture = digest(4);
        let bytes = reference_bytes(fixture);
        let parsed =
            Pg1Reference::from_tsv("tests/artifacts/perf/pg1/reference.tsv", bytes.as_bytes())
                .expect("qualified Reference evidence");
        assert_eq!(parsed.reference_commit, REFERENCE_COMMIT);
        assert_eq!(parsed.benchmark_definition, fixture);
        assert_eq!(parsed.median_ns, 2_000_000_000);
        assert_eq!(parsed.evidence.digest, sha256(bytes.as_bytes()));

        let unqualified = bytes.replace("valid_samples\t9", "valid_samples\t8");
        assert!(matches!(
            Pg1Reference::from_tsv(
                "tests/artifacts/perf/pg1/unqualified.tsv",
                unqualified.as_bytes(),
            ),
            Err(FrontdoorError::Identity(_))
        ));
        let wrong_pin = bytes.replace(REFERENCE_COMMIT, &"0".repeat(40));
        assert!(matches!(
            Pg1Reference::from_tsv(
                "tests/artifacts/perf/pg1/wrong-pin.tsv",
                wrong_pin.as_bytes(),
            ),
            Err(FrontdoorError::Identity(_))
        ));
    }

    #[test]
    fn definitions_bind_artifact_fixture_and_reference_axes() {
        let fixture = digest(4);
        let reference_text = reference_bytes(fixture);
        let reference = Pg1Reference::from_tsv(
            "tests/artifacts/perf/pg1/reference.tsv",
            reference_text.as_bytes(),
        )
        .expect("Reference evidence");
        let pg1 = FrontdoorDefinition::new(
            FrontdoorScenario::OpeningClassG2,
            artifact(),
            fixture,
            Some(reference),
        )
        .expect("PG-1 definition");
        let text = pg1.to_tsv();
        assert!(text.contains("build_profile\trelease-perf\n"));
        assert!(text.contains(&format!("fixture_digest\t{fixture}\n")));
        assert!(text.contains("output_mode\tffmpeg-video\n"));

        assert!(matches!(
            FrontdoorDefinition::new(FrontdoorScenario::OpeningClassG2, artifact(), fixture, None,),
            Err(FrontdoorError::Identity(_))
        ));
        assert!(matches!(
            FrontdoorDefinition::new(
                FrontdoorScenario::Preview1080p,
                artifact(),
                fixture,
                pg1.reference,
            ),
            Err(FrontdoorError::Identity(_))
        ));
        let mut wrong_profile = artifact();
        wrong_profile.cargo_profile = "release".to_owned();
        assert!(matches!(
            FrontdoorDefinition::new(
                FrontdoorScenario::Preview1080p,
                wrong_profile,
                fixture,
                None,
            ),
            Err(FrontdoorError::Identity(_))
        ));
    }

    #[test]
    fn integer_sample_conversions_do_not_hide_zero_or_overflow() {
        assert_eq!(
            ratio_sample(1_000_000_000, 2_000_000_000),
            Sample::valid(500_000)
        );
        assert_eq!(fps_milli_sample(60, 1_000_000_000), Sample::valid(60_000));
        assert!(ratio_sample(0, 1).invalid_reason.is_some());
        assert!(ratio_sample(1, 0).invalid_reason.is_some());
        assert!(fps_milli_sample(0, 1).invalid_reason.is_some());
        assert!(fps_milli_sample(1, 0).invalid_reason.is_some());
    }

    #[test]
    fn fmtl_fixtures_are_deterministic_and_have_the_declared_duration() {
        let opening_a = opening_class_fixture().expect("opening fixture");
        let opening_b = opening_class_fixture().expect("repeat opening fixture");
        assert_eq!(opening_a, opening_b);
        assert_eq!(
            sha256(&opening_a).to_string(),
            // The bundle binds its renderer and animation-law identity:
            // re-pinned for fmtl-law:2 (835cc580) and renderer version 7
            // (fm-sq8.9, +Y up the frame); then for begin-time targets,
            // since FadeIn (3e3bc154) and fixed-method transforms (a6ad13e3)
            // no longer allocate construction-time copies. All 30 certified
            // frames are byte-identical across both re-pins.
            "939ab39bae8d220314f89dc97a94c71968a7e0ba5105abf1b94e44bbe78786a0"
        );
        let opening = TimelineBundle::from_bytes(&opening_a).expect("decode opening fixture");
        assert_eq!(opening.fps(), 30);
        assert_eq!(opening.segment_count(), 2);
        assert_eq!(opening.duration_seconds(), 1.0);
        assert_eq!(
            opening
                .segment_frame_range(0)
                .expect("first range")
                .chain(opening.segment_frame_range(1).expect("second range"))
                .count(),
            30
        );

        let trailing_a = trailing_edit_fixture([0.5, 0.0, 0.0]).expect("trailing fixture");
        let trailing_b = trailing_edit_fixture([0.0, 0.5, 0.0]).expect("edited fixture");
        assert_ne!(trailing_a, trailing_b);
        let trailing = TimelineBundle::from_bytes(&trailing_a).expect("decode trailing fixture");
        assert_eq!(trailing.fps(), 30);
        assert_eq!(trailing.segment_count(), 2);
        assert_eq!(trailing.duration_seconds(), 30.0);
        assert_eq!(
            trailing
                .segment_frame_range(0)
                .expect("wait range")
                .chain(trailing.segment_frame_range(1).expect("edit range"))
                .count(),
            900
        );
    }

    #[test]
    fn injected_frontdoor_regression_blocks_with_a_flamegraph() {
        let fixture = digest(4);
        let definition =
            FrontdoorDefinition::new(FrontdoorScenario::Preview1080p, artifact(), fixture, None)
                .expect("preview definition");
        let policy = GatePolicy {
            gate: GateId::Pg3,
            scenario: FrontdoorScenario::Preview1080p.name().to_owned(),
            unit: MetricUnit::FramesPerSecondMilli,
            direction: Direction::AtLeast,
            target: Some(60_000),
            min_valid_samples: 21,
            max_invalid_samples: 3,
            max_mad_bps: 1_000,
            alert_regression_bps: 500,
            block_regression_bps: 1_000,
            enforcement: Enforcement::Blocking,
            scope: GateScope::Core,
            require_regression_profile: true,
        };
        let key = BenchmarkKey {
            profile_id: "linux-x86-64-pg".to_owned(),
            build_profile: BUILD_PROFILE.to_owned(),
            host_fingerprint: digest(5),
            toolchain_fingerprint: digest(6),
            suite_lock_digest: definition.artifact.suite_lock_digest,
            benchmark_definition: definition.digest(),
            gate: GateId::Pg3,
            scenario: FrontdoorScenario::Preview1080p.name().to_owned(),
            unit: MetricUnit::FramesPerSecondMilli,
            engine: ENGINE.to_owned(),
            tier: definition.artifact.compiled_tier.clone(),
            thread_profile: THREAD_PROFILE.to_owned(),
            execution_plan_digest: digest(7),
            config_digest: definition.config_digest(),
            cache_state: FrontdoorScenario::Preview1080p.cache_state().to_owned(),
            output_mode: FrontdoorScenario::Preview1080p.output_mode().to_owned(),
            external_tool_fingerprint: None,
            bare_metal: true,
            isolated: true,
        };
        let baseline_batch = MeasurementBatch {
            key: key.clone(),
            producer_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            samples: vec![Sample::valid(70_000); FRONTDOOR_SAMPLE_COUNT],
            evidence: Vec::new(),
        };
        let baseline = Baseline::observed(
            1,
            policy,
            &baseline_batch,
            "tests/artifacts/perf/pg3/preview-baseline.tsv",
        )
        .expect("observed baseline");
        let flame_bytes = b"<svg>injected preview slowdown</svg>";
        let run = MeasurementBatch {
            key,
            producer_commit: "fedcba9876543210fedcba9876543210fedcba98".to_owned(),
            samples: vec![Sample::valid(60_000); FRONTDOOR_SAMPLE_COUNT],
            evidence: vec![
                EvidenceRef::from_bytes(
                    EvidenceKind::Flamegraph,
                    "tests/artifacts/perf/pg3/injected-regression.svg",
                    flame_bytes,
                )
                .expect("flamegraph evidence"),
            ],
        };
        let report = baseline.evaluate(Some(&baseline_batch), &run);
        assert_eq!(report.verdict, Verdict::Block);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "baseline-regression")
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.code != "regression-profile-missing")
        );
    }
}
