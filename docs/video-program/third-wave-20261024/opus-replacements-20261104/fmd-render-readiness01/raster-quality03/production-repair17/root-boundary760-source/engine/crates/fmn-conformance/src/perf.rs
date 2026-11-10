//! Fail-closed performance-gate decisions (plan §17.2).
//!
//! A wall-clock number is not a gate result by itself. A result becomes
//! comparable only when the benchmark definition, pinned host, toolchain,
//! suite lock, build profile, engine/tier, thread profile, derived execution
//! plan, semantic config, cache state, output path, and optional external-tool
//! fingerprint all match. Even then, the raw repetitions must be retained,
//! host-quality failures are invalid samples rather than numbers to average,
//! and dispersion must stay inside the scenario's declared envelope.
//!
//! This module owns those laws, not the act of timing. Bench binaries feed it
//! integer measurements in a declared [`MetricUnit`]; the evaluator returns an
//! explicit [`Verdict`]. In particular:
//!
//! - a target-only baseline is [`Verdict::Inconclusive`], never green;
//! - a host or comparability mismatch is inconclusive, not a regression;
//! - excessive dispersion is inconclusive, not a lucky pass;
//! - an alerting regression requires a CPU/flame artifact;
//! - only a [`GateScope::Core`] blocking policy may gate ordinary core changes;
//!   PG-8 and PG-A remain scoped to Python and annex changes respectively.

use fmn_hash::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

/// Stable policy-catalog schema.
pub const POLICY_SCHEMA: &str = "fmn-perf-policy/1";
/// Stable observed-baseline schema.
pub const BASELINE_SCHEMA: &str = "fmn-perf-baseline/1";
/// Stable raw-measurement bundle schema.
pub const SAMPLES_SCHEMA: &str = "fmn-perf-samples/1";
/// Stable robot-facing report schema.
pub const REPORT_SCHEMA: &str = "fmn-perf-report/1";

const POLICY_PREAMBLE: &str = "# Machine-readable §17.2 policy catalog. These are targets and enforcement\n\
# policy, not observed baselines. A row becomes pass-capable only after a\n\
# content-addressed pinned-host observation is committed through\n\
# fmn_conformance::perf::Baseline.\n";
const POLICY_COLUMNS: &str = "# gate\tscenario\tunit\tdirection\ttarget\tmin_valid_samples\tmax_invalid_samples\tmax_mad_bps\talert_regression_bps\tblock_regression_bps\tenforcement\tscope\trequire_regression_profile";

/// Cargo profile selected when this `fmn-conformance` artifact was compiled.
///
/// The package build script derives this from Cargo-controlled `OUT_DIR`
/// structure. Runtime executable placement is deliberately irrelevant: moving
/// an ordinary release binary below a directory named `release-perf` cannot
/// change the embedded identity.
pub const COMPILED_CARGO_PROFILE: &str = env!("FMN_CONFORMANCE_CARGO_PROFILE");

/// Require an exact compile-time Cargo profile before collecting timing data.
///
/// # Errors
/// [`PerfError::Identity`] when the compiled profile differs from `expected`.
pub fn require_compiled_cargo_profile(expected: &str) -> Result<(), PerfError> {
    if COMPILED_CARGO_PROFILE == expected {
        Ok(())
    } else {
        Err(PerfError::Identity(format!(
            "measurement requires Cargo profile {expected:?}, but this artifact was compiled with {COMPILED_CARGO_PROFILE:?}"
        )))
    }
}

const MAX_SAMPLES: usize = 65_536;
const MAX_EVIDENCE: usize = 4_096;
const MAX_RAW_BUNDLE_BYTES: usize = 128 * 1024 * 1024;
const MAX_POLICY_ROWS: usize = 4_096;
const MAX_POLICY_CATALOG_BYTES: usize = 1024 * 1024;
const MAX_BASELINE_BYTES: usize = 64 * 1024;
const MAX_REPORT_FINDINGS: usize = 32;
const MAX_REPORT_BYTES: usize = 256 * 1024;
const MAX_TOKEN_BYTES: usize = 160;
const MAX_DETAIL_BYTES: usize = 1_024;
const MAX_EVIDENCE_PATH_BYTES: usize = 512;
const MAX_DIAGNOSTIC_VALUE_BYTES: usize = 160;
const BPS_DENOMINATOR: u128 = 10_000;
const NONE: &str = "-";

/// PG plane named by §17.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GateId {
    /// End-to-end wall time.
    Pg1,
    /// CPU rasterizer throughput.
    Pg2,
    /// Preview/export frame throughput.
    Pg3,
    /// Cold-start and edit-to-frame latency.
    Pg4,
    /// Schedule- and thread-independent bits.
    Pg5,
    /// Peak memory, leaks, and steady-state allocations.
    Pg6,
    /// Native typesetting latency.
    Pg7,
    /// Python binding tax.
    Pg8,
    /// Accelerator Annex performance.
    PgA,
}

impl GateId {
    /// Every gate, in plan order.
    pub const ALL: [Self; 9] = [
        Self::Pg1,
        Self::Pg2,
        Self::Pg3,
        Self::Pg4,
        Self::Pg5,
        Self::Pg6,
        Self::Pg7,
        Self::Pg8,
        Self::PgA,
    ];

    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pg1 => "pg-1",
            Self::Pg2 => "pg-2",
            Self::Pg3 => "pg-3",
            Self::Pg4 => "pg-4",
            Self::Pg5 => "pg-5",
            Self::Pg6 => "pg-6",
            Self::Pg7 => "pg-7",
            Self::Pg8 => "pg-8",
            Self::PgA => "pg-a",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "pg-1" => Self::Pg1,
            "pg-2" => Self::Pg2,
            "pg-3" => Self::Pg3,
            "pg-4" => Self::Pg4,
            "pg-5" => Self::Pg5,
            "pg-6" => Self::Pg6,
            "pg-7" => Self::Pg7,
            "pg-8" => Self::Pg8,
            "pg-a" => Self::PgA,
            _ => return None,
        })
    }
}

impl fmt::Display for GateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Unit carried by every raw integer sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricUnit {
    /// Elapsed nanoseconds.
    Nanoseconds,
    /// Bytes resident or transferred.
    Bytes,
    /// Heap allocation count.
    Allocations,
    /// Bytes still live after a soak.
    LeakedBytes,
    /// Mismatched artifacts or frames.
    Mismatches,
    /// Frames per second multiplied by 1,000.
    FramesPerSecondMilli,
    /// Megapixels per second multiplied by 1,000.
    MegaPixelsPerSecondMilli,
    /// Dimensionless ratio multiplied by 1,000,000.
    RatioPpm,
}

impl MetricUnit {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nanoseconds => "nanoseconds",
            Self::Bytes => "bytes",
            Self::Allocations => "allocations",
            Self::LeakedBytes => "leaked-bytes",
            Self::Mismatches => "mismatches",
            Self::FramesPerSecondMilli => "fps-milli",
            Self::MegaPixelsPerSecondMilli => "mpx-per-second-milli",
            Self::RatioPpm => "ratio-ppm",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "nanoseconds" => Self::Nanoseconds,
            "bytes" => Self::Bytes,
            "allocations" => Self::Allocations,
            "leaked-bytes" => Self::LeakedBytes,
            "mismatches" => Self::Mismatches,
            "fps-milli" => Self::FramesPerSecondMilli,
            "mpx-per-second-milli" => Self::MegaPixelsPerSecondMilli,
            "ratio-ppm" => Self::RatioPpm,
            _ => return None,
        })
    }
}

/// Which side of a target is better.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Lower values are better.
    AtMost,
    /// Higher values are better.
    AtLeast,
    /// Only exact equality is acceptable.
    Exactly,
}

impl Direction {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AtMost => "at-most",
            Self::AtLeast => "at-least",
            Self::Exactly => "exactly",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "at-most" => Self::AtMost,
            "at-least" => Self::AtLeast,
            "exactly" => Self::Exactly,
            _ => return None,
        })
    }
}

/// Changes that a gate is allowed to block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateScope {
    /// Ordinary CPU/core changes.
    Core,
    /// Python bridge changes and G4a only.
    PythonOnly,
    /// Accelerator Annex changes only (R21).
    AnnexOnly,
}

impl GateScope {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::PythonOnly => "python-only",
            Self::AnnexOnly => "annex-only",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "core" => Self::Core,
            "python-only" => Self::PythonOnly,
            "annex-only" => Self::AnnexOnly,
            _ => return None,
        })
    }
}

/// Strength of a target/regression miss after comparability is established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enforcement {
    /// Measure and publish; a miss alerts but cannot block.
    Observe,
    /// Alert on a miss; used while a profile is being ratcheted.
    Alert,
    /// Block an in-scope change on a miss.
    Blocking,
}

impl Enforcement {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Alert => "alert",
            Self::Blocking => "blocking",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "observe" => Self::Observe,
            "alert" => Self::Alert,
            "blocking" => Self::Blocking,
            _ => return None,
        })
    }
}

/// One machine-readable gate policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GatePolicy {
    /// PG plane.
    pub gate: GateId,
    /// Unique scenario identity within the plane.
    pub scenario: String,
    /// Sample unit.
    pub unit: MetricUnit,
    /// Better direction.
    pub direction: Direction,
    /// Absolute plan target, or `None` for baseline-only annex observations.
    pub target: Option<u64>,
    /// Minimum valid repetitions.
    pub min_valid_samples: usize,
    /// Maximum retained host-quality failures.
    pub max_invalid_samples: usize,
    /// Maximum median absolute deviation, in basis points of the median.
    pub max_mad_bps: u32,
    /// Same-profile regression that first alerts.
    pub alert_regression_bps: u32,
    /// Same-profile regression that may block.
    pub block_regression_bps: u32,
    /// Alert/block stage.
    pub enforcement: Enforcement,
    /// Changes this policy may block.
    pub scope: GateScope,
    /// Whether any alerting regression must carry a CPU/flame artifact.
    pub require_regression_profile: bool,
}

impl GatePolicy {
    /// Validate policy invariants.
    ///
    /// # Errors
    /// Returns a typed error for malformed or self-contradictory policy.
    pub fn validate(&self) -> Result<(), PerfError> {
        validate_token("scenario", &self.scenario)?;
        if !(3..=MAX_SAMPLES).contains(&self.min_valid_samples) {
            return Err(PerfError::Policy(format!(
                "{} requires {} samples; allowed range is 3..={MAX_SAMPLES}",
                self.scenario, self.min_valid_samples
            )));
        }
        if self.max_invalid_samples > MAX_SAMPLES - self.min_valid_samples {
            return Err(PerfError::Policy(format!(
                "{} permits {} invalid samples, exceeding the remaining {}-sample resource budget",
                self.scenario,
                self.max_invalid_samples,
                MAX_SAMPLES - self.min_valid_samples
            )));
        }
        if self.alert_regression_bps > self.block_regression_bps {
            return Err(PerfError::Policy(format!(
                "{} alert threshold {} exceeds block threshold {}",
                self.scenario, self.alert_regression_bps, self.block_regression_bps
            )));
        }
        if self.direction == Direction::Exactly && self.target.is_none() {
            return Err(PerfError::Policy(format!(
                "{} uses exact comparison without a target",
                self.scenario
            )));
        }
        if self.target.is_none() && self.gate != GateId::PgA {
            return Err(PerfError::Policy(format!(
                "{} omits an absolute target outside the PG-A baseline-only plane",
                self.scenario
            )));
        }
        match (self.gate, self.scope) {
            (GateId::Pg8, GateScope::PythonOnly)
            | (GateId::PgA, GateScope::AnnexOnly)
            | (
                GateId::Pg1
                | GateId::Pg2
                | GateId::Pg3
                | GateId::Pg4
                | GateId::Pg5
                | GateId::Pg6
                | GateId::Pg7,
                GateScope::Core,
            ) => {}
            _ => {
                return Err(PerfError::Policy(format!(
                    "{} has scope {} inconsistent with {}",
                    self.scenario,
                    self.scope.name(),
                    self.gate
                )));
            }
        }
        Ok(())
    }
}

/// Parse the committed policy catalog.
///
/// The format is a strict, dependency-free TSV: one schema row followed by
/// thirteen-column policy rows. Every PG plane must appear and `(gate, scenario)`
/// identities must be unique.
///
/// # Errors
/// Returns a line-attributed error for any malformed or incomplete catalog.
pub fn parse_policy_catalog(text: &str) -> Result<Vec<GatePolicy>, PerfError> {
    if text.len() > MAX_POLICY_CATALOG_BYTES {
        return Err(PerfError::Catalog {
            line: 0,
            detail: format!(
                "catalog is {} bytes, exceeding the {MAX_POLICY_CATALOG_BYTES}-byte limit",
                text.len()
            ),
        });
    }
    if !text.ends_with('\n') {
        return Err(PerfError::Catalog {
            line: 0,
            detail: "catalog must end with a final LF newline".to_owned(),
        });
    }
    if text.contains('\r') {
        return Err(PerfError::Catalog {
            line: 0,
            detail: "catalog must use LF line endings".to_owned(),
        });
    }
    let mut schema_seen = false;
    let mut policies = Vec::new();
    let mut identities = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim_end();
        if trimmed.trim().is_empty() || trimmed.trim_start().starts_with('#') {
            continue;
        }
        if !schema_seen {
            let fields =
                split_exact_tsv_fields::<2>(trimmed).map_err(|field_count| PerfError::Catalog {
                    line,
                    detail: format!("schema row has {field_count} fields, expected 2"),
                })?;
            if fields != ["schema", POLICY_SCHEMA] {
                return Err(PerfError::Catalog {
                    line,
                    detail: format!("first data row must be `schema\\t{POLICY_SCHEMA}`"),
                });
            }
            schema_seen = true;
            continue;
        }
        let fields =
            split_exact_tsv_fields::<13>(trimmed).map_err(|field_count| PerfError::Catalog {
                line,
                detail: format!("policy row has {field_count} fields, expected 13"),
            })?;
        let [
            gate,
            scenario,
            unit,
            direction,
            target,
            min_valid_samples,
            max_invalid_samples,
            max_mad_bps,
            alert_regression_bps,
            block_regression_bps,
            enforcement,
            scope,
            require_regression_profile,
        ] = fields;
        let policy = GatePolicy {
            gate: GateId::parse(gate).ok_or_else(|| PerfError::Catalog {
                line,
                detail: format!("unknown gate {}", bounded_debug(gate)),
            })?,
            scenario: {
                validate_token("scenario", scenario).map_err(|error| PerfError::Catalog {
                    line,
                    detail: error.to_string(),
                })?;
                scenario.to_owned()
            },
            unit: MetricUnit::parse(unit).ok_or_else(|| PerfError::Catalog {
                line,
                detail: format!("unknown metric unit {}", bounded_debug(unit)),
            })?,
            direction: Direction::parse(direction).ok_or_else(|| PerfError::Catalog {
                line,
                detail: format!("unknown direction {}", bounded_debug(direction)),
            })?,
            target: parse_optional_u64(target)
                .map_err(|detail| PerfError::Catalog { line, detail })?,
            min_valid_samples: parse_number(min_valid_samples, "min_valid_samples", line)?,
            max_invalid_samples: parse_number(max_invalid_samples, "max_invalid_samples", line)?,
            max_mad_bps: parse_number(max_mad_bps, "max_mad_bps", line)?,
            alert_regression_bps: parse_number(alert_regression_bps, "alert_regression_bps", line)?,
            block_regression_bps: parse_number(block_regression_bps, "block_regression_bps", line)?,
            enforcement: Enforcement::parse(enforcement).ok_or_else(|| PerfError::Catalog {
                line,
                detail: format!("unknown enforcement {}", bounded_debug(enforcement)),
            })?,
            scope: GateScope::parse(scope).ok_or_else(|| PerfError::Catalog {
                line,
                detail: format!("unknown scope {}", bounded_debug(scope)),
            })?,
            require_regression_profile: require_regression_profile.parse().map_err(|_| {
                PerfError::Catalog {
                    line,
                    detail: format!(
                        "bad require_regression_profile {}",
                        bounded_debug(require_regression_profile)
                    ),
                }
            })?,
        };
        policy.validate().map_err(|error| PerfError::Catalog {
            line,
            detail: error.to_string(),
        })?;
        if policies.len() == MAX_POLICY_ROWS {
            return Err(PerfError::Catalog {
                line,
                detail: format!("catalog exceeds the {MAX_POLICY_ROWS}-row limit"),
            });
        }
        let identity = (policy.gate, policy.scenario.clone());
        if !identities.insert(identity) {
            return Err(PerfError::Catalog {
                line,
                detail: format!("duplicate {} scenario {:?}", policy.gate, policy.scenario),
            });
        }
        policies.push(policy);
    }
    if !schema_seen {
        return Err(PerfError::Catalog {
            line: 0,
            detail: "catalog has no schema row".to_owned(),
        });
    }
    for gate in GateId::ALL {
        if !policies.iter().any(|policy| policy.gate == gate) {
            return Err(PerfError::Catalog {
                line: 0,
                detail: format!("catalog has no {gate} policy"),
            });
        }
    }
    if render_policy_catalog(&policies)? != text {
        return Err(PerfError::Catalog {
            line: 0,
            detail: "catalog is valid but not in canonical form".to_owned(),
        });
    }
    Ok(policies)
}

/// Render a canonical policy catalog.
///
/// # Errors
/// Returns a typed error before allocating the output when policies are
/// incomplete, duplicated, invalid, or exceed the catalog resource envelope.
pub fn render_policy_catalog(policies: &[GatePolicy]) -> Result<String, PerfError> {
    if policies.len() > MAX_POLICY_ROWS {
        return Err(PerfError::Catalog {
            line: 0,
            detail: format!("catalog exceeds the {MAX_POLICY_ROWS}-row limit"),
        });
    }

    let mut identities = BTreeSet::new();
    for policy in policies {
        policy.validate()?;
        if !identities.insert((policy.gate, policy.scenario.as_str())) {
            return Err(PerfError::Catalog {
                line: 0,
                detail: format!("duplicate {} scenario {:?}", policy.gate, policy.scenario),
            });
        }
    }
    for gate in GateId::ALL {
        if !policies.iter().any(|policy| policy.gate == gate) {
            return Err(PerfError::Catalog {
                line: 0,
                detail: format!("catalog has no {gate} policy"),
            });
        }
    }

    let mut ordered: Vec<_> = policies.iter().collect();
    ordered.sort_unstable_by(|left, right| {
        (left.gate, left.scenario.as_str()).cmp(&(right.gate, right.scenario.as_str()))
    });

    let mut output_len = POLICY_PREAMBLE
        .len()
        .checked_add("schema\t".len())
        .and_then(|length| length.checked_add(POLICY_SCHEMA.len()))
        .and_then(|length| length.checked_add(1))
        .and_then(|length| length.checked_add(POLICY_COLUMNS.len()))
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| PerfError::Catalog {
            line: 0,
            detail: "catalog size overflow".to_owned(),
        })?;
    for policy in &ordered {
        output_len = output_len
            .checked_add(policy_catalog_row_len(policy))
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| PerfError::Catalog {
                line: 0,
                detail: "catalog size overflow".to_owned(),
            })?;
        if output_len > MAX_POLICY_CATALOG_BYTES {
            return Err(PerfError::Catalog {
                line: 0,
                detail: format!(
                    "rendered catalog exceeds the {MAX_POLICY_CATALOG_BYTES}-byte limit"
                ),
            });
        }
    }

    let mut output = String::with_capacity(output_len);
    output.push_str(POLICY_PREAMBLE);
    let _ = writeln!(output, "schema\t{POLICY_SCHEMA}");
    output.push_str(POLICY_COLUMNS);
    output.push('\n');
    for policy in ordered {
        let _ = write!(
            output,
            "{}\t{}\t{}\t{}\t",
            policy.gate,
            policy.scenario,
            policy.unit.name(),
            policy.direction.name(),
        );
        if let Some(target) = policy.target {
            let _ = write!(output, "{target}");
        } else {
            output.push_str(NONE);
        }
        let _ = writeln!(
            output,
            "\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            policy.min_valid_samples,
            policy.max_invalid_samples,
            policy.max_mad_bps,
            policy.alert_regression_bps,
            policy.block_regression_bps,
            policy.enforcement.name(),
            policy.scope.name(),
            policy.require_regression_profile,
        );
    }
    debug_assert_eq!(output.len(), output_len);
    Ok(output)
}

fn decimal_digits(mut value: u128) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn policy_catalog_row_len(policy: &GatePolicy) -> usize {
    policy.gate.name().len()
        + policy.scenario.len()
        + policy.unit.name().len()
        + policy.direction.name().len()
        + policy
            .target
            .map_or(1, |value| decimal_digits(u128::from(value)))
        + decimal_digits(policy.min_valid_samples as u128)
        + decimal_digits(policy.max_invalid_samples as u128)
        + decimal_digits(u128::from(policy.max_mad_bps))
        + decimal_digits(u128::from(policy.alert_regression_bps))
        + decimal_digits(u128::from(policy.block_regression_bps))
        + policy.enforcement.name().len()
        + policy.scope.name().len()
        + if policy.require_regression_profile {
            "true".len()
        } else {
            "false".len()
        }
        + 12
}

/// Exact environment/benchmark identity required for comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BenchmarkKey {
    /// Named pinned profile.
    pub profile_id: String,
    /// Cargo/build profile, normally `release-perf`.
    pub build_profile: String,
    /// Canonical host fingerprint.
    pub host_fingerprint: Digest,
    /// Exact rust/toolchain fingerprint.
    pub toolchain_fingerprint: Digest,
    /// Exact `SUITE.lock` bytes.
    pub suite_lock_digest: Digest,
    /// Exact benchmark definition bytes.
    pub benchmark_definition: Digest,
    /// PG plane.
    pub gate: GateId,
    /// Policy scenario.
    pub scenario: String,
    /// Unit carried by every sample.
    pub unit: MetricUnit,
    /// Engine identity.
    pub engine: String,
    /// Build/SIMD tier.
    pub tier: String,
    /// Declared fixed/matrix thread configuration.
    pub thread_profile: String,
    /// Derived scheduler/team topology.
    pub execution_plan_digest: Digest,
    /// Semantic configuration digest.
    pub config_digest: Digest,
    /// Declared cache state.
    pub cache_state: String,
    /// Raw/PNG/software-video/hardware-video path.
    pub output_mode: String,
    /// Exact ffmpeg/tool fingerprint, absent on native-only paths.
    pub external_tool_fingerprint: Option<Digest>,
    /// Host is a dedicated bare-metal profile, not RCH/shared calibration.
    pub bare_metal: bool,
    /// Workload-isolation checks passed.
    pub isolated: bool,
}

impl BenchmarkKey {
    /// Validate token fields and the gate/policy relationship.
    ///
    /// # Errors
    /// Returns a typed error when identity is ambiguous or unqualified.
    pub fn validate(&self) -> Result<(), PerfError> {
        validate_token("profile_id", &self.profile_id)?;
        validate_token("build_profile", &self.build_profile)?;
        validate_token("scenario", &self.scenario)?;
        validate_token("engine", &self.engine)?;
        validate_token("tier", &self.tier)?;
        validate_token("thread_profile", &self.thread_profile)?;
        validate_token("cache_state", &self.cache_state)?;
        validate_token("output_mode", &self.output_mode)?;
        Ok(())
    }

    /// Canonical digest used as the baseline lookup key.
    #[must_use]
    pub fn digest(&self) -> Digest {
        let mut hash = Sha256::new();
        hash.update(b"fmn-perf-benchmark-key-v1");
        hash_field(&mut hash, self.profile_id.as_bytes());
        hash_field(&mut hash, self.build_profile.as_bytes());
        hash.update(self.host_fingerprint.as_bytes());
        hash.update(self.toolchain_fingerprint.as_bytes());
        hash.update(self.suite_lock_digest.as_bytes());
        hash.update(self.benchmark_definition.as_bytes());
        hash_field(&mut hash, self.gate.name().as_bytes());
        hash_field(&mut hash, self.scenario.as_bytes());
        hash_field(&mut hash, self.unit.name().as_bytes());
        hash_field(&mut hash, self.engine.as_bytes());
        hash_field(&mut hash, self.tier.as_bytes());
        hash_field(&mut hash, self.thread_profile.as_bytes());
        hash.update(self.execution_plan_digest.as_bytes());
        hash.update(self.config_digest.as_bytes());
        hash_field(&mut hash, self.cache_state.as_bytes());
        hash_field(&mut hash, self.output_mode.as_bytes());
        match self.external_tool_fingerprint {
            Some(digest) => {
                hash.update(&[1]);
                hash.update(digest.as_bytes());
            }
            None => hash.update(&[0]),
        }
        hash.update(&[u8::from(self.bare_metal), u8::from(self.isolated)]);
        hash.finalize()
    }
}

/// One repetition, retained even when invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sample {
    /// Raw integer in the policy's unit.
    pub value: u64,
    /// Why this repetition is excluded from statistics.
    pub invalid_reason: Option<String>,
}

impl Sample {
    /// A valid repetition.
    #[must_use]
    pub const fn valid(value: u64) -> Self {
        Self {
            value,
            invalid_reason: None,
        }
    }

    /// An invalid repetition retained in the bundle.
    #[must_use]
    pub fn invalid(value: u64, reason: impl Into<String>) -> Self {
        Self {
            value,
            invalid_reason: Some(reason.into()),
        }
    }

    pub(crate) fn validate_invalid_reason(reason: &str) -> Result<(), PerfError> {
        validate_detail("invalid_reason", reason)
    }

    fn validate(&self) -> Result<(), PerfError> {
        if let Some(reason) = &self.invalid_reason {
            Self::validate_invalid_reason(reason)?;
        }
        Ok(())
    }
}

/// Robust integer summary over valid repetitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RobustStats {
    /// Valid and invalid repetitions retained.
    pub total_samples: usize,
    /// Repetitions admitted to statistics.
    pub valid_samples: usize,
    /// Repetitions retained but excluded.
    pub invalid_samples: usize,
    /// Minimum valid value.
    pub min: u64,
    /// Median valid value.
    pub median: u64,
    /// Nearest-rank p95.
    pub p95: u64,
    /// Nearest-rank p99.
    pub p99: u64,
    /// Maximum valid value.
    pub max: u64,
    /// Median absolute deviation.
    pub mad: u64,
    /// MAD divided by median, in basis points.
    pub mad_bps: u32,
}

impl RobustStats {
    /// Summarize valid repetitions, retaining invalid counts.
    ///
    /// # Errors
    /// Returns a typed error for oversized input, malformed invalid evidence,
    /// or fewer than `minimum` valid samples.
    pub fn from_samples(samples: &[Sample], minimum: usize) -> Result<Self, PerfError> {
        if minimum == 0 || minimum > MAX_SAMPLES {
            return Err(PerfError::Samples(format!(
                "minimum valid repetitions must be 1..={MAX_SAMPLES}, got {minimum}"
            )));
        }
        if samples.len() > MAX_SAMPLES {
            return Err(PerfError::Samples(format!(
                "{} repetitions exceed the {MAX_SAMPLES} resource limit",
                samples.len()
            )));
        }
        let mut values = Vec::with_capacity(samples.len());
        for sample in samples {
            sample.validate()?;
            if sample.invalid_reason.is_none() {
                values.push(sample.value);
            }
        }
        if values.len() < minimum {
            return Err(PerfError::Samples(format!(
                "{} valid repetitions, need at least {minimum}; {} invalid repetitions retained",
                values.len(),
                samples.len().saturating_sub(values.len())
            )));
        }
        values.sort_unstable();
        let median_value = median(&values)
            .ok_or_else(|| PerfError::Samples("no valid repetitions to summarize".to_owned()))?;
        let mut deviations: Vec<_> = values
            .iter()
            .map(|value| value.abs_diff(median_value))
            .collect();
        deviations.sort_unstable();
        let mad = median(&deviations)
            .ok_or_else(|| PerfError::Samples("no deviations to summarize".to_owned()))?;
        let mad_bps = ratio_bps(mad, median_value);
        let min = values
            .first()
            .copied()
            .ok_or_else(|| PerfError::Samples("no minimum repetition".to_owned()))?;
        let max = values
            .last()
            .copied()
            .ok_or_else(|| PerfError::Samples("no maximum repetition".to_owned()))?;
        let p95 = nearest_rank(&values, 95)
            .ok_or_else(|| PerfError::Samples("no p95 repetition".to_owned()))?;
        let p99 = nearest_rank(&values, 99)
            .ok_or_else(|| PerfError::Samples("no p99 repetition".to_owned()))?;
        Ok(Self {
            total_samples: samples.len(),
            valid_samples: values.len(),
            invalid_samples: samples.len() - values.len(),
            min,
            median: median_value,
            p95,
            p99,
            max,
            mad,
            mad_bps,
        })
    }

    fn validate(&self) -> Result<(), PerfError> {
        if self.total_samples > MAX_SAMPLES {
            return Err(PerfError::Baseline(format!(
                "observation retains {} repetitions, exceeding the {MAX_SAMPLES} resource limit",
                self.total_samples
            )));
        }
        let retained = self
            .valid_samples
            .checked_add(self.invalid_samples)
            .ok_or_else(|| {
                PerfError::Baseline("observation sample counts overflow usize".to_owned())
            })?;
        if retained != self.total_samples {
            return Err(PerfError::Baseline(format!(
                "observation sample counts disagree: total {}, valid {}, invalid {}",
                self.total_samples, self.valid_samples, self.invalid_samples
            )));
        }
        if self.valid_samples == 0 {
            return Err(PerfError::Baseline(
                "observation has no valid repetitions".to_owned(),
            ));
        }
        if !(self.min <= self.median
            && self.median <= self.p95
            && self.p95 <= self.p99
            && self.p99 <= self.max)
        {
            return Err(PerfError::Baseline(format!(
                "observation order must satisfy min <= median <= p95 <= p99 <= max; got {} <= {} <= {} <= {} <= {}",
                self.min, self.median, self.p95, self.p99, self.max
            )));
        }
        if self.mad > self.max - self.min {
            return Err(PerfError::Baseline(format!(
                "observation MAD {} exceeds its value range {}",
                self.mad,
                self.max - self.min
            )));
        }
        let expected_mad_bps = ratio_bps(self.mad, self.median);
        if self.mad_bps != expected_mad_bps {
            return Err(PerfError::Baseline(format!(
                "observation MAD ratio {} bps disagrees with median {} and MAD {}; expected {} bps",
                self.mad_bps, self.median, self.mad, expected_mad_bps
            )));
        }
        Ok(())
    }
}

/// Artifact kind referenced by a baseline or regression report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Raw sample bundle.
    RawSamples,
    /// Live pinned-host qualification record.
    HostAttestation,
    /// Flamegraph SVG.
    Flamegraph,
    /// Samply/perf CPU profile.
    CpuProfile,
    /// Heap/allocation profile.
    AllocationProfile,
    /// Structured phase trace.
    PhaseTrace,
    /// Bit-identity or other golden output.
    Golden,
}

impl EvidenceKind {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RawSamples => "raw-samples",
            Self::HostAttestation => "host-attestation",
            Self::Flamegraph => "flamegraph",
            Self::CpuProfile => "cpu-profile",
            Self::AllocationProfile => "allocation-profile",
            Self::PhaseTrace => "phase-trace",
            Self::Golden => "golden",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "raw-samples" => Self::RawSamples,
            "host-attestation" => Self::HostAttestation,
            "flamegraph" => Self::Flamegraph,
            "cpu-profile" => Self::CpuProfile,
            "allocation-profile" => Self::AllocationProfile,
            "phase-trace" => Self::PhaseTrace,
            "golden" => Self::Golden,
            _ => return None,
        })
    }

    const fn is_regression_profile(self) -> bool {
        matches!(self, Self::Flamegraph | Self::CpuProfile)
    }
}

/// Content-addressed repository evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceRef {
    /// Artifact class.
    pub kind: EvidenceKind,
    /// Canonical repo-relative path below `tests/artifacts/perf/`.
    pub path: String,
    /// SHA-256 of the exact artifact bytes.
    pub digest: Digest,
}

impl EvidenceRef {
    /// Build a content-addressed reference from the exact artifact bytes.
    ///
    /// # Errors
    /// Returns a typed error for an ambiguous or out-of-tree path.
    pub fn from_bytes(
        kind: EvidenceKind,
        path: impl Into<String>,
        bytes: &[u8],
    ) -> Result<Self, PerfError> {
        let evidence = Self {
            kind,
            path: path.into(),
            digest: fmn_hash::sha256(bytes),
        };
        evidence.validate()?;
        Ok(evidence)
    }

    /// Validate the portable, non-traversing path contract.
    ///
    /// # Errors
    /// Returns a typed error for ambiguous or out-of-tree evidence paths.
    pub fn validate(&self) -> Result<(), PerfError> {
        validate_evidence_path(&self.path)
    }
}

/// One measured run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasurementBatch {
    /// Exact comparison key.
    pub key: BenchmarkKey,
    /// Commit that produced this run; provenance, not a comparison key.
    pub producer_commit: String,
    /// All repetitions, including invalid host-quality samples.
    pub samples: Vec<Sample>,
    /// Content-addressed artifacts.
    pub evidence: Vec<EvidenceRef>,
}

impl MeasurementBatch {
    fn validate_intrinsic(&self) -> Result<(), PerfError> {
        self.key.validate()?;
        validate_producer_commit(&self.producer_commit)?;
        if self.samples.len() > MAX_SAMPLES {
            return Err(PerfError::Samples(format!(
                "{} repetitions exceed the {MAX_SAMPLES} resource limit",
                self.samples.len()
            )));
        }
        if self.evidence.len() > MAX_EVIDENCE {
            return Err(PerfError::Evidence(format!(
                "{} artifacts exceed the {MAX_EVIDENCE} resource limit",
                self.evidence.len()
            )));
        }
        for sample in &self.samples {
            sample.validate()?;
        }
        let mut paths = BTreeSet::new();
        for evidence in &self.evidence {
            evidence.validate()?;
            if !paths.insert(evidence.path.as_str()) {
                return Err(PerfError::Evidence(format!(
                    "duplicate artifact path {:?}",
                    evidence.path
                )));
            }
        }
        Ok(())
    }

    fn validate(&self, policy: &GatePolicy) -> Result<(), PerfError> {
        self.validate_intrinsic()?;
        if self.key.gate != policy.gate || self.key.scenario != policy.scenario {
            return Err(PerfError::Identity(format!(
                "run names {}/{} but policy names {}/{}",
                self.key.gate, self.key.scenario, policy.gate, policy.scenario
            )));
        }
        if self.key.unit != policy.unit {
            return Err(PerfError::Identity(format!(
                "run unit {} but policy unit {}",
                self.key.unit.name(),
                policy.unit.name()
            )));
        }
        Ok(())
    }

    /// Canonical raw-bundle TSV.
    ///
    /// Header records fix every comparable identity and the sample unit.
    /// Sample records retain every repetition in original order, including
    /// invalid values and their reasons. Evidence records are path-sorted so
    /// their input order cannot perturb the content address. Tabs and control
    /// characters are forbidden by the field validators.
    ///
    /// # Errors
    /// Returns a typed error for malformed identity, samples, or evidence.
    pub fn to_tsv(&self) -> Result<String, PerfError> {
        self.validate_intrinsic()?;
        let mut output = String::new();
        {
            let mut row = |name: &str, value: &dyn fmt::Display| {
                let _ = writeln!(output, "{name}\t{value}");
            };
            row("schema", &SAMPLES_SCHEMA);
            row("profile_id", &self.key.profile_id);
            row("build_profile", &self.key.build_profile);
            row("host_fingerprint", &self.key.host_fingerprint);
            row("toolchain_fingerprint", &self.key.toolchain_fingerprint);
            row("suite_lock_digest", &self.key.suite_lock_digest);
            row("benchmark_definition", &self.key.benchmark_definition);
            row("gate", &self.key.gate);
            row("scenario", &self.key.scenario);
            row("unit", &self.key.unit.name());
            row("engine", &self.key.engine);
            row("tier", &self.key.tier);
            row("thread_profile", &self.key.thread_profile);
            row("execution_plan_digest", &self.key.execution_plan_digest);
            row("config_digest", &self.key.config_digest);
            row("cache_state", &self.key.cache_state);
            row("output_mode", &self.key.output_mode);
            let external_tool_fingerprint = self
                .key
                .external_tool_fingerprint
                .map_or_else(|| NONE.to_owned(), |digest| digest.to_hex());
            row("external_tool_fingerprint", &external_tool_fingerprint);
            row("bare_metal", &self.key.bare_metal);
            row("isolated", &self.key.isolated);
            row("producer_commit", &self.producer_commit);
            row("sample_count", &self.samples.len());
            row("evidence_count", &self.evidence.len());
        }
        for (index, sample) in self.samples.iter().enumerate() {
            if let Some(reason) = &sample.invalid_reason {
                let _ = writeln!(
                    output,
                    "sample\t{index}\t{}\tinvalid\t{reason}",
                    sample.value
                );
            } else {
                let _ = writeln!(output, "sample\t{index}\t{}\tvalid\t{NONE}", sample.value);
            }
        }
        let mut evidence: Vec<_> = self.evidence.iter().collect();
        evidence.sort_by(|left, right| {
            (left.path.as_str(), left.kind.name(), left.digest).cmp(&(
                right.path.as_str(),
                right.kind.name(),
                right.digest,
            ))
        });
        for (index, evidence) in evidence.into_iter().enumerate() {
            let _ = writeln!(
                output,
                "evidence\t{index}\t{}\t{}\t{}",
                evidence.kind.name(),
                evidence.path,
                evidence.digest,
            );
        }
        Ok(output)
    }

    /// Parse a canonical raw-measurement bundle.
    ///
    /// # Errors
    /// Returns a typed error for malformed, oversized, non-canonical, or
    /// internally inconsistent evidence.
    pub fn from_tsv(text: &str) -> Result<Self, PerfError> {
        if text.len() > MAX_RAW_BUNDLE_BYTES {
            return Err(PerfError::Samples(format!(
                "raw bundle is {} bytes, exceeding the {MAX_RAW_BUNDLE_BYTES}-byte limit",
                text.len()
            )));
        }
        let mut lines = text.lines();
        let schema = next_raw_header(&mut lines, "schema")?;
        if schema != SAMPLES_SCHEMA {
            return Err(PerfError::Samples(format!(
                "unsupported raw bundle schema {}",
                bounded_debug(schema)
            )));
        }
        let profile_id = own_raw_token(next_raw_header(&mut lines, "profile_id")?, "profile_id")?;
        let build_profile = own_raw_token(
            next_raw_header(&mut lines, "build_profile")?,
            "build_profile",
        )?;
        let host_fingerprint = parse_raw_digest(
            next_raw_header(&mut lines, "host_fingerprint")?,
            "host_fingerprint",
        )?;
        let toolchain_fingerprint = parse_raw_digest(
            next_raw_header(&mut lines, "toolchain_fingerprint")?,
            "toolchain_fingerprint",
        )?;
        let suite_lock_digest = parse_raw_digest(
            next_raw_header(&mut lines, "suite_lock_digest")?,
            "suite_lock_digest",
        )?;
        let benchmark_definition = parse_raw_digest(
            next_raw_header(&mut lines, "benchmark_definition")?,
            "benchmark_definition",
        )?;
        let gate_text = next_raw_header(&mut lines, "gate")?;
        let gate = GateId::parse(gate_text)
            .ok_or_else(|| PerfError::Samples(format!("bad gate {}", bounded_debug(gate_text))))?;
        let scenario = own_raw_token(next_raw_header(&mut lines, "scenario")?, "scenario")?;
        let unit_text = next_raw_header(&mut lines, "unit")?;
        let unit = MetricUnit::parse(unit_text)
            .ok_or_else(|| PerfError::Samples(format!("bad unit {}", bounded_debug(unit_text))))?;
        let engine = own_raw_token(next_raw_header(&mut lines, "engine")?, "engine")?;
        let tier = own_raw_token(next_raw_header(&mut lines, "tier")?, "tier")?;
        let thread_profile = own_raw_token(
            next_raw_header(&mut lines, "thread_profile")?,
            "thread_profile",
        )?;
        let execution_plan_digest = parse_raw_digest(
            next_raw_header(&mut lines, "execution_plan_digest")?,
            "execution_plan_digest",
        )?;
        let config_digest = parse_raw_digest(
            next_raw_header(&mut lines, "config_digest")?,
            "config_digest",
        )?;
        let cache_state =
            own_raw_token(next_raw_header(&mut lines, "cache_state")?, "cache_state")?;
        let output_mode =
            own_raw_token(next_raw_header(&mut lines, "output_mode")?, "output_mode")?;
        let external_tool_fingerprint = parse_optional_raw_digest(
            next_raw_header(&mut lines, "external_tool_fingerprint")?,
            "external_tool_fingerprint",
        )?;
        let bare_metal =
            parse_raw_number(next_raw_header(&mut lines, "bare_metal")?, "bare_metal")?;
        let isolated = parse_raw_number(next_raw_header(&mut lines, "isolated")?, "isolated")?;
        let producer_commit_text = next_raw_header(&mut lines, "producer_commit")?;
        validate_producer_commit(producer_commit_text)?;
        let producer_commit = producer_commit_text.to_owned();
        let sample_count: usize =
            parse_raw_number(next_raw_header(&mut lines, "sample_count")?, "sample_count")?;
        let evidence_count: usize = parse_raw_number(
            next_raw_header(&mut lines, "evidence_count")?,
            "evidence_count",
        )?;
        if sample_count > MAX_SAMPLES {
            return Err(PerfError::Samples(format!(
                "{sample_count} repetitions exceed the {MAX_SAMPLES} resource limit"
            )));
        }
        if evidence_count > MAX_EVIDENCE {
            return Err(PerfError::Evidence(format!(
                "{evidence_count} artifacts exceed the {MAX_EVIDENCE} resource limit"
            )));
        }

        let mut samples = Vec::with_capacity(sample_count);
        for expected_index in 0..sample_count {
            let line = lines
                .next()
                .ok_or_else(|| PerfError::Samples(format!("missing sample {expected_index}")))?;
            let fields = split_exact_tsv_fields::<5>(line).map_err(|field_count| {
                PerfError::Samples(format!(
                    "sample {expected_index} has {field_count} fields, expected 5"
                ))
            })?;
            let [record, index, value, status, reason] = fields;
            if record != "sample"
                || parse_raw_number::<usize>(index, "sample index")? != expected_index
            {
                return Err(PerfError::Samples(format!(
                    "expected sample index {expected_index}, found {}",
                    bounded_debug(index)
                )));
            }
            let value = parse_raw_number(value, "sample value")?;
            let sample = match (status, reason) {
                ("valid", NONE) => Sample::valid(value),
                ("invalid", reason) => {
                    Sample::validate_invalid_reason(reason)?;
                    Sample::invalid(value, reason)
                }
                _ => {
                    return Err(PerfError::Samples(format!(
                        "sample {expected_index} has bad status/reason {}/{}",
                        bounded_debug(status),
                        bounded_debug(reason)
                    )));
                }
            };
            samples.push(sample);
        }

        let mut evidence = Vec::with_capacity(evidence_count);
        for expected_index in 0..evidence_count {
            let line = lines.next().ok_or_else(|| {
                PerfError::Evidence(format!("missing evidence record {expected_index}"))
            })?;
            let fields = split_exact_tsv_fields::<5>(line).map_err(|field_count| {
                PerfError::Evidence(format!(
                    "evidence {expected_index} has {field_count} fields, expected 5"
                ))
            })?;
            let [record, index, kind, path, digest] = fields;
            if record != "evidence"
                || parse_raw_number::<usize>(index, "evidence index")? != expected_index
            {
                return Err(PerfError::Evidence(format!(
                    "expected evidence index {expected_index}, found {}",
                    bounded_debug(index)
                )));
            }
            let kind = EvidenceKind::parse(kind).ok_or_else(|| {
                PerfError::Evidence(format!("bad evidence kind {}", bounded_debug(kind)))
            })?;
            validate_evidence_path(path)?;
            evidence.push(EvidenceRef {
                kind,
                path: path.to_owned(),
                digest: parse_raw_digest(digest, "evidence digest")?,
            });
        }
        if let Some(extra) = lines.next() {
            return Err(PerfError::Samples(format!(
                "unexpected trailing raw-bundle record {}",
                bounded_debug(extra)
            )));
        }
        let batch = Self {
            key: BenchmarkKey {
                profile_id,
                build_profile,
                host_fingerprint,
                toolchain_fingerprint,
                suite_lock_digest,
                benchmark_definition,
                gate,
                scenario,
                unit,
                engine,
                tier,
                thread_profile,
                execution_plan_digest,
                config_digest,
                cache_state,
                output_mode,
                external_tool_fingerprint,
                bare_metal,
                isolated,
            },
            producer_commit,
            samples,
            evidence,
        };
        batch.validate_intrinsic()?;
        if batch.to_tsv()? != text {
            return Err(PerfError::Samples(
                "raw bundle is valid but not in canonical form".to_owned(),
            ));
        }
        Ok(batch)
    }

    /// Reference the canonical raw bundle at a repository-relative path.
    ///
    /// # Errors
    /// Returns a typed error if the batch or destination path is malformed.
    pub fn raw_evidence(&self, path: impl Into<String>) -> Result<EvidenceRef, PerfError> {
        let bytes = self.to_tsv()?;
        EvidenceRef::from_bytes(EvidenceKind::RawSamples, path, bytes.as_bytes())
    }
}

/// One observed baseline, or an explicitly unobserved target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baseline {
    /// Monotone policy/baseline revision.
    pub generation: u32,
    /// Policy evaluated by this row.
    pub policy: GatePolicy,
    /// Exact comparison key.
    pub key: BenchmarkKey,
    /// Commit that produced the observation.
    pub producer_commit: String,
    /// Robust observed value and its source bundle; absent means targeted only.
    pub observation: Option<BaselineObservation>,
}

/// Observed baseline statistics and raw-bundle identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineObservation {
    /// Robust summary over the baseline repetitions.
    pub stats: RobustStats,
    /// Content-addressed raw sample bundle.
    pub source: EvidenceRef,
}

impl Baseline {
    /// Create a target-only row. It can document policy but cannot pass a gate.
    ///
    /// # Errors
    /// Returns a typed error for malformed policy or identity.
    pub fn targeted(
        generation: u32,
        policy: GatePolicy,
        key: BenchmarkKey,
        producer_commit: impl Into<String>,
    ) -> Result<Self, PerfError> {
        let baseline = Self {
            generation,
            policy,
            key,
            producer_commit: producer_commit.into(),
            observation: None,
        };
        baseline.validate()?;
        Ok(baseline)
    }

    /// Create an observed baseline from retained repetitions.
    ///
    /// # Errors
    /// Returns a typed error for malformed identity/evidence or insufficient
    /// valid samples.
    pub fn observed(
        generation: u32,
        policy: GatePolicy,
        batch: &MeasurementBatch,
        source_path: impl Into<String>,
    ) -> Result<Self, PerfError> {
        batch.validate(&policy)?;
        let stats = RobustStats::from_samples(&batch.samples, policy.min_valid_samples)?;
        let source = batch.raw_evidence(source_path)?;
        let baseline = Self {
            generation,
            policy,
            key: batch.key.clone(),
            producer_commit: batch.producer_commit.clone(),
            observation: Some(BaselineObservation { stats, source }),
        };
        baseline.validate()?;
        Ok(baseline)
    }

    /// Validate baseline invariants.
    ///
    /// # Errors
    /// Returns a typed error for malformed or internally inconsistent state.
    pub fn validate(&self) -> Result<(), PerfError> {
        if self.generation == 0 {
            return Err(PerfError::Baseline(
                "baseline generation must be nonzero".to_owned(),
            ));
        }
        self.policy.validate()?;
        self.key.validate()?;
        validate_producer_commit(&self.producer_commit)?;
        if self.key.gate != self.policy.gate
            || self.key.scenario != self.policy.scenario
            || self.key.unit != self.policy.unit
        {
            return Err(PerfError::Baseline(format!(
                "key {}/{}/{} does not match policy {}/{}/{}",
                self.key.gate,
                self.key.scenario,
                self.key.unit.name(),
                self.policy.gate,
                self.policy.scenario,
                self.policy.unit.name()
            )));
        }
        if let Some(observation) = &self.observation {
            if !self.key.bare_metal || !self.key.isolated {
                return Err(PerfError::Baseline(
                    "an observed baseline must be both bare-metal and isolation-qualified"
                        .to_owned(),
                ));
            }
            observation.stats.validate()?;
            if observation.stats.valid_samples < self.policy.min_valid_samples {
                return Err(PerfError::Baseline(format!(
                    "observation has {} valid samples, policy requires {}",
                    observation.stats.valid_samples, self.policy.min_valid_samples
                )));
            }
            if observation.stats.invalid_samples > self.policy.max_invalid_samples {
                return Err(PerfError::Baseline(format!(
                    "observation has {} invalid samples, policy permits at most {}",
                    observation.stats.invalid_samples, self.policy.max_invalid_samples
                )));
            }
            if observation.stats.mad_bps > self.policy.max_mad_bps {
                return Err(PerfError::Baseline(format!(
                    "observation MAD {} bps exceeds the policy envelope {} bps",
                    observation.stats.mad_bps, self.policy.max_mad_bps
                )));
            }
            if let (Direction::Exactly, Some(target)) = (self.policy.direction, self.policy.target)
                && (observation.stats.min != target || observation.stats.max != target)
            {
                return Err(PerfError::Baseline(format!(
                    "exact baseline valid range {}..={} does not equal its target {target}",
                    observation.stats.min, observation.stats.max
                )));
            }
            if observation.source.kind != EvidenceKind::RawSamples {
                return Err(PerfError::Baseline(
                    "observation source is not raw-samples".to_owned(),
                ));
            }
            observation.source.validate()?;
        }
        Ok(())
    }

    /// Verify the bytes loaded from the observation's committed source path.
    ///
    /// # Errors
    /// Returns a typed error for a target-only baseline or digest mismatch.
    pub fn verify_observation_source(&self, bytes: &[u8]) -> Result<(), PerfError> {
        let observation = self.observation.as_ref().ok_or_else(|| {
            PerfError::Evidence("target-only baseline has no observation source".to_owned())
        })?;
        let actual = fmn_hash::sha256(bytes);
        if actual != observation.source.digest {
            return Err(PerfError::Evidence(format!(
                "raw sample digest mismatch for {:?}: expected {}, found {}",
                observation.source.path, observation.source.digest, actual
            )));
        }
        Ok(())
    }

    /// Replay a parsed raw bundle against every stored observation invariant.
    ///
    /// # Errors
    /// Returns a typed error when identity, producer, robust statistics, or
    /// content address differs from the baseline.
    pub fn verify_observation_batch(&self, batch: &MeasurementBatch) -> Result<(), PerfError> {
        self.validate()?;
        batch.validate(&self.policy)?;
        let observation = self.observation.as_ref().ok_or_else(|| {
            PerfError::Evidence("target-only baseline has no observation source".to_owned())
        })?;
        if batch.key != self.key {
            return Err(PerfError::Identity(identity_difference(
                &self.key, &batch.key,
            )));
        }
        if batch.producer_commit != self.producer_commit {
            return Err(PerfError::Identity(format!(
                "raw bundle producer {} differs from baseline producer {}",
                batch.producer_commit, self.producer_commit
            )));
        }
        let stats = RobustStats::from_samples(&batch.samples, self.policy.min_valid_samples)?;
        if stats != observation.stats {
            return Err(PerfError::Baseline(format!(
                "raw-bundle statistics {stats:?} differ from stored observation {:?}",
                observation.stats
            )));
        }
        self.verify_observation_source(batch.to_tsv()?.as_bytes())
    }

    /// Evaluate a current run after replaying the baseline's raw observation.
    ///
    /// Environmental and evidence gaps are [`Verdict::Inconclusive`]. They are
    /// deliberately distinct from both green and a measured regression.
    /// `baseline_source` must be the parsed bundle named by the baseline's
    /// content-addressed `raw-samples` reference. Requiring it here prevents a
    /// structurally plausible but unverified serialized baseline from passing.
    #[must_use]
    pub fn evaluate(
        &self,
        baseline_source: Option<&MeasurementBatch>,
        run: &MeasurementBatch,
    ) -> GateReport {
        if let Err(error) = self.validate() {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "invalid-baseline",
                error.to_string(),
            );
        }
        if let Err(error) = run.validate(&self.policy) {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "invalid-run",
                error.to_string(),
            );
        }
        if run.key != self.key {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "identity-mismatch",
                identity_difference(&self.key, &run.key),
            );
        }
        if !run.key.bare_metal || !run.key.isolated {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "host-unqualified",
                "run is not both bare-metal and isolation-qualified".to_owned(),
            );
        }
        let Some(observation) = &self.observation else {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "baseline-unobserved",
                "policy has a target but no pinned-host observation".to_owned(),
            );
        };
        let Some(baseline_source) = baseline_source else {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "baseline-source-unverified",
                format!(
                    "baseline observation {:?} was not supplied for replay",
                    observation.source.path
                ),
            );
        };
        if let Err(error) = self.verify_observation_batch(baseline_source) {
            return GateReport::inconclusive(
                &self.policy,
                self.key.digest(),
                "invalid-baseline-source",
                error.to_string(),
            );
        }
        let stats = match RobustStats::from_samples(&run.samples, self.policy.min_valid_samples) {
            Ok(stats) => stats,
            Err(error) => {
                return GateReport::inconclusive(
                    &self.policy,
                    self.key.digest(),
                    "invalid-samples",
                    error.to_string(),
                );
            }
        };
        if stats.total_samples != observation.stats.total_samples {
            return GateReport {
                schema: REPORT_SCHEMA,
                gate: self.policy.gate,
                scenario: self.policy.scenario.clone(),
                scope: self.policy.scope,
                verdict: Verdict::Inconclusive,
                key_digest: self.key.digest(),
                stats: Some(stats),
                baseline_median: Some(observation.stats.median),
                regression_bps: None,
                findings: vec![Finding::new(
                    "sample-plan-mismatch",
                    format!(
                        "run retains {} repetitions but baseline retains {}",
                        stats.total_samples, observation.stats.total_samples
                    ),
                )],
            };
        }
        if stats.invalid_samples > self.policy.max_invalid_samples {
            return GateReport {
                schema: REPORT_SCHEMA,
                gate: self.policy.gate,
                scenario: self.policy.scenario.clone(),
                scope: self.policy.scope,
                verdict: Verdict::Inconclusive,
                key_digest: self.key.digest(),
                stats: Some(stats),
                baseline_median: Some(observation.stats.median),
                regression_bps: None,
                findings: vec![Finding::new(
                    "invalid-sample-budget-exceeded",
                    format!(
                        "run retains {} invalid repetitions but policy permits at most {}",
                        stats.invalid_samples, self.policy.max_invalid_samples
                    ),
                )],
            };
        }
        if self.policy.direction != Direction::Exactly && stats.mad_bps > self.policy.max_mad_bps {
            return GateReport {
                schema: REPORT_SCHEMA,
                gate: self.policy.gate,
                scenario: self.policy.scenario.clone(),
                scope: self.policy.scope,
                verdict: Verdict::Inconclusive,
                key_digest: self.key.digest(),
                stats: Some(stats),
                baseline_median: Some(observation.stats.median),
                regression_bps: None,
                findings: vec![Finding::new(
                    "excessive-dispersion",
                    format!(
                        "MAD {} bps exceeds the {} bps envelope",
                        stats.mad_bps, self.policy.max_mad_bps
                    ),
                )],
            };
        }

        let regression_bps = regression_bps(self.policy.direction, &observation.stats, &stats);
        let target_missed = self
            .policy
            .target
            .is_some_and(|target| !meets(self.policy.direction, &stats, target));
        let alerting_regression =
            crosses_regression_threshold(regression_bps, self.policy.alert_regression_bps);
        let blocking_regression =
            crosses_regression_threshold(regression_bps, self.policy.block_regression_bps);
        let mut findings = Vec::new();
        let mut verdict = Verdict::Pass;

        if let Some(target) = self.policy.target.filter(|_| target_missed) {
            let measured = if self.policy.direction == Direction::Exactly {
                format!(
                    "valid sample range {}..={} {}",
                    stats.min,
                    stats.max,
                    self.policy.unit.name()
                )
            } else {
                format!("median {} {}", stats.median, self.policy.unit.name())
            };
            findings.push(Finding::new(
                "target-miss",
                format!(
                    "{measured} does not satisfy {} {}",
                    self.policy.direction.name(),
                    target
                ),
            ));
            verdict = miss_verdict(self.policy.enforcement);
        }
        if alerting_regression {
            let detail = if self.policy.direction == Direction::Exactly {
                format!(
                    "valid sample range changed from {}..={} to {}..={} under an exact policy",
                    observation.stats.min, observation.stats.max, stats.min, stats.max
                )
            } else {
                format!(
                    "median regressed {regression_bps} bps from {} to {}",
                    observation.stats.median, stats.median
                )
            };
            findings.push(Finding::new("baseline-regression", detail));
            verdict = verdict.max(if blocking_regression {
                miss_verdict(self.policy.enforcement)
            } else {
                Verdict::Alert
            });
            if self.policy.require_regression_profile
                && !run
                    .evidence
                    .iter()
                    .any(|evidence| evidence.kind.is_regression_profile())
            {
                findings.push(Finding::new(
                    "regression-profile-missing",
                    "alerting regression has no flamegraph or CPU profile artifact".to_owned(),
                ));
                verdict = verdict.max(if self.policy.enforcement == Enforcement::Blocking {
                    Verdict::Block
                } else {
                    Verdict::Alert
                });
            }
        }
        GateReport {
            schema: REPORT_SCHEMA,
            gate: self.policy.gate,
            scenario: self.policy.scenario.clone(),
            scope: self.policy.scope,
            verdict,
            key_digest: self.key.digest(),
            stats: Some(stats),
            baseline_median: Some(observation.stats.median),
            regression_bps: Some(regression_bps),
            findings,
        }
    }

    /// Canonical dependency-free TSV representation.
    ///
    /// # Errors
    /// Returns a typed error before allocating the output buffer when public
    /// baseline state is invalid, or if the canonical document exceeds its
    /// resource limit.
    pub fn to_tsv(&self) -> Result<String, PerfError> {
        self.validate()?;
        let observation = self.observation.as_ref();
        let stats = observation.map(|value| value.stats);
        let source = observation.map(|value| &value.source);
        let mut output = String::new();
        let mut row = |key: &str, value: String| {
            let _ = writeln!(output, "{key}\t{value}");
        };
        row("schema", BASELINE_SCHEMA.to_owned());
        row("generation", self.generation.to_string());
        row("gate", self.policy.gate.name().to_owned());
        row("scenario", self.policy.scenario.clone());
        row("unit", self.policy.unit.name().to_owned());
        row("direction", self.policy.direction.name().to_owned());
        row(
            "target",
            self.policy
                .target
                .map_or_else(|| NONE.to_owned(), |value| value.to_string()),
        );
        row(
            "min_valid_samples",
            self.policy.min_valid_samples.to_string(),
        );
        row(
            "max_invalid_samples",
            self.policy.max_invalid_samples.to_string(),
        );
        row("max_mad_bps", self.policy.max_mad_bps.to_string());
        row(
            "alert_regression_bps",
            self.policy.alert_regression_bps.to_string(),
        );
        row(
            "block_regression_bps",
            self.policy.block_regression_bps.to_string(),
        );
        row("enforcement", self.policy.enforcement.name().to_owned());
        row("scope", self.policy.scope.name().to_owned());
        row(
            "require_regression_profile",
            self.policy.require_regression_profile.to_string(),
        );
        row("profile_id", self.key.profile_id.clone());
        row("build_profile", self.key.build_profile.clone());
        row("host_fingerprint", self.key.host_fingerprint.to_hex());
        row(
            "toolchain_fingerprint",
            self.key.toolchain_fingerprint.to_hex(),
        );
        row("suite_lock_digest", self.key.suite_lock_digest.to_hex());
        row(
            "benchmark_definition",
            self.key.benchmark_definition.to_hex(),
        );
        row("engine", self.key.engine.clone());
        row("tier", self.key.tier.clone());
        row("thread_profile", self.key.thread_profile.clone());
        row(
            "execution_plan_digest",
            self.key.execution_plan_digest.to_hex(),
        );
        row("config_digest", self.key.config_digest.to_hex());
        row("cache_state", self.key.cache_state.clone());
        row("output_mode", self.key.output_mode.clone());
        row(
            "external_tool_fingerprint",
            self.key
                .external_tool_fingerprint
                .map_or_else(|| NONE.to_owned(), |digest| digest.to_hex()),
        );
        row("bare_metal", self.key.bare_metal.to_string());
        row("isolated", self.key.isolated.to_string());
        row("producer_commit", self.producer_commit.clone());
        row(
            "observed_total",
            optional_stat(stats, |value| value.total_samples),
        );
        row(
            "observed_valid",
            optional_stat(stats, |value| value.valid_samples),
        );
        row(
            "observed_invalid",
            optional_stat(stats, |value| value.invalid_samples),
        );
        row("observed_min", optional_stat(stats, |value| value.min));
        row(
            "observed_median",
            optional_stat(stats, |value| value.median),
        );
        row("observed_p95", optional_stat(stats, |value| value.p95));
        row("observed_p99", optional_stat(stats, |value| value.p99));
        row("observed_max", optional_stat(stats, |value| value.max));
        row("observed_mad", optional_stat(stats, |value| value.mad));
        row(
            "observed_mad_bps",
            optional_stat(stats, |value| value.mad_bps),
        );
        row(
            "source_kind",
            source.map_or_else(|| NONE.to_owned(), |value| value.kind.name().to_owned()),
        );
        row(
            "source_path",
            source.map_or_else(|| NONE.to_owned(), |value| value.path.clone()),
        );
        row(
            "source_digest",
            source.map_or_else(|| NONE.to_owned(), |value| value.digest.to_hex()),
        );
        if output.len() > MAX_BASELINE_BYTES {
            return Err(PerfError::Baseline(format!(
                "rendered baseline is {} bytes, exceeding the {MAX_BASELINE_BYTES}-byte limit",
                output.len()
            )));
        }
        Ok(output)
    }

    /// Parse a canonical baseline TSV.
    ///
    /// # Errors
    /// Returns a typed error on missing, duplicate, unknown, or malformed keys.
    pub fn from_tsv(text: &str) -> Result<Self, PerfError> {
        let fields = parse_baseline_fields(text)?;
        let get = |name: &'static str| {
            fields
                .get(name)
                .cloned()
                .ok_or_else(|| PerfError::Baseline(format!("missing {name}")))
        };
        if get("schema")? != BASELINE_SCHEMA {
            return Err(PerfError::Baseline(format!(
                "unsupported schema {}",
                bounded_debug(&get("schema")?)
            )));
        }
        let gate = GateId::parse(&get("gate")?)
            .ok_or_else(|| PerfError::Baseline("bad gate".to_owned()))?;
        let scenario = get("scenario")?;
        let policy = GatePolicy {
            gate,
            scenario: scenario.clone(),
            unit: MetricUnit::parse(&get("unit")?)
                .ok_or_else(|| PerfError::Baseline("bad unit".to_owned()))?,
            direction: Direction::parse(&get("direction")?)
                .ok_or_else(|| PerfError::Baseline("bad direction".to_owned()))?,
            target: parse_optional_u64(&get("target")?).map_err(PerfError::Baseline)?,
            min_valid_samples: parse_baseline_number(&fields, "min_valid_samples")?,
            max_invalid_samples: parse_baseline_number(&fields, "max_invalid_samples")?,
            max_mad_bps: parse_baseline_number(&fields, "max_mad_bps")?,
            alert_regression_bps: parse_baseline_number(&fields, "alert_regression_bps")?,
            block_regression_bps: parse_baseline_number(&fields, "block_regression_bps")?,
            enforcement: Enforcement::parse(&get("enforcement")?)
                .ok_or_else(|| PerfError::Baseline("bad enforcement".to_owned()))?,
            scope: GateScope::parse(&get("scope")?)
                .ok_or_else(|| PerfError::Baseline("bad scope".to_owned()))?,
            require_regression_profile: parse_baseline_number(
                &fields,
                "require_regression_profile",
            )?,
        };
        let key = BenchmarkKey {
            profile_id: get("profile_id")?,
            build_profile: get("build_profile")?,
            host_fingerprint: parse_digest(&get("host_fingerprint")?, "host_fingerprint")?,
            toolchain_fingerprint: parse_digest(
                &get("toolchain_fingerprint")?,
                "toolchain_fingerprint",
            )?,
            suite_lock_digest: parse_digest(&get("suite_lock_digest")?, "suite_lock_digest")?,
            benchmark_definition: parse_digest(
                &get("benchmark_definition")?,
                "benchmark_definition",
            )?,
            gate,
            scenario,
            unit: policy.unit,
            engine: get("engine")?,
            tier: get("tier")?,
            thread_profile: get("thread_profile")?,
            execution_plan_digest: parse_digest(
                &get("execution_plan_digest")?,
                "execution_plan_digest",
            )?,
            config_digest: parse_digest(&get("config_digest")?, "config_digest")?,
            cache_state: get("cache_state")?,
            output_mode: get("output_mode")?,
            external_tool_fingerprint: parse_optional_baseline_digest(
                &get("external_tool_fingerprint")?,
                "external_tool_fingerprint",
            )?,
            bare_metal: parse_baseline_number(&fields, "bare_metal")?,
            isolated: parse_baseline_number(&fields, "isolated")?,
        };
        let observed_valid = get("observed_valid")?;
        let observation = if observed_valid == NONE {
            for name in OBSERVATION_FIELDS {
                if get(name)? != NONE {
                    return Err(PerfError::Baseline(format!(
                        "targeted baseline has nonempty {name}"
                    )));
                }
            }
            None
        } else {
            let stats = RobustStats {
                total_samples: parse_baseline_number(&fields, "observed_total")?,
                valid_samples: parse_baseline_number(&fields, "observed_valid")?,
                invalid_samples: parse_baseline_number(&fields, "observed_invalid")?,
                min: parse_baseline_number(&fields, "observed_min")?,
                median: parse_baseline_number(&fields, "observed_median")?,
                p95: parse_baseline_number(&fields, "observed_p95")?,
                p99: parse_baseline_number(&fields, "observed_p99")?,
                max: parse_baseline_number(&fields, "observed_max")?,
                mad: parse_baseline_number(&fields, "observed_mad")?,
                mad_bps: parse_baseline_number(&fields, "observed_mad_bps")?,
            };
            let source = EvidenceRef {
                kind: EvidenceKind::parse(&get("source_kind")?)
                    .ok_or_else(|| PerfError::Baseline("bad source_kind".to_owned()))?,
                path: get("source_path")?,
                digest: parse_digest(&get("source_digest")?, "source_digest")?,
            };
            Some(BaselineObservation { stats, source })
        };
        let baseline = Self {
            generation: parse_baseline_number(&fields, "generation")?,
            policy,
            key,
            producer_commit: get("producer_commit")?,
            observation,
        };
        baseline.validate()?;
        if baseline.to_tsv()? != text {
            return Err(PerfError::Baseline(
                "baseline is valid but not in canonical form".to_owned(),
            ));
        }
        Ok(baseline)
    }
}

/// Terminal gate classification.
///
/// Ordering is intentional: it lets evaluation accumulate the strongest
/// attributable outcome. Inconclusive is handled before measured outcomes and
/// is never compared as "stronger" or "weaker" than a regression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    /// Comparable and within both target and regression envelope.
    Pass,
    /// Comparable miss under observe/alert policy, or below the block threshold.
    Alert,
    /// Comparable miss under blocking policy.
    Block,
    /// No attributable gate decision can be made.
    Inconclusive,
}

impl Verdict {
    /// Stable machine spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Alert => "alert",
            Self::Block => "block",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// One stable report finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// Robot-facing stable code.
    pub code: &'static str,
    /// Human detail.
    pub detail: String,
}

impl Finding {
    fn new(code: &'static str, detail: String) -> Self {
        Self { code, detail }
    }
}

/// Evaluation report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateReport {
    /// Report schema.
    pub schema: &'static str,
    /// PG plane.
    pub gate: GateId,
    /// Scenario.
    pub scenario: String,
    /// Allowed blocking scope.
    pub scope: GateScope,
    /// Terminal classification.
    pub verdict: Verdict,
    /// Comparable benchmark-key digest.
    pub key_digest: Digest,
    /// Current robust summary, when attributable.
    pub stats: Option<RobustStats>,
    /// Baseline median, when observed.
    pub baseline_median: Option<u64>,
    /// Direction-aware regression in basis points.
    pub regression_bps: Option<u32>,
    /// Stable findings.
    pub findings: Vec<Finding>,
}

impl GateReport {
    fn inconclusive(
        policy: &GatePolicy,
        key_digest: Digest,
        code: &'static str,
        detail: String,
    ) -> Self {
        Self {
            schema: REPORT_SCHEMA,
            gate: policy.gate,
            scenario: policy.scenario.clone(),
            scope: policy.scope,
            verdict: Verdict::Inconclusive,
            key_digest,
            stats: None,
            baseline_median: None,
            regression_bps: None,
            findings: vec![Finding::new(code, detail)],
        }
    }

    /// Validate report invariants and resource bounds.
    ///
    /// # Errors
    /// Returns a typed error for an unknown schema, malformed robot tokens,
    /// inconsistent statistics, or an oversized finding set.
    pub fn validate(&self) -> Result<(), PerfError> {
        if self.schema != REPORT_SCHEMA {
            return Err(PerfError::Report(format!(
                "schema must be {REPORT_SCHEMA:?}"
            )));
        }
        validate_report_token("scenario", &self.scenario)?;
        if let Some(stats) = self.stats {
            stats
                .validate()
                .map_err(|error| PerfError::Report(error.to_string()))?;
        }
        if self.stats.is_none() && self.baseline_median.is_some() {
            return Err(PerfError::Report(
                "baseline_median requires current statistics".to_owned(),
            ));
        }
        if self.regression_bps.is_some() && (self.stats.is_none() || self.baseline_median.is_none())
        {
            return Err(PerfError::Report(
                "regression_bps requires current statistics and baseline_median".to_owned(),
            ));
        }
        if self.findings.len() > MAX_REPORT_FINDINGS {
            return Err(PerfError::Report(format!(
                "{} findings exceed the {MAX_REPORT_FINDINGS}-finding limit",
                self.findings.len()
            )));
        }
        for finding in &self.findings {
            validate_report_token("finding code", finding.code)?;
            if finding.detail.is_empty() || finding.detail.len() > MAX_DETAIL_BYTES {
                return Err(PerfError::Report(format!(
                    "finding detail length must be 1..={MAX_DETAIL_BYTES}, got {}",
                    finding.detail.len()
                )));
            }
        }
        Ok(())
    }

    /// Stable line-oriented robot output: one summary then zero or more
    /// findings. Human decoration is never mixed into these lines.
    ///
    /// # Errors
    /// Returns a typed error before allocating output for invalid public state,
    /// or if the canonical report exceeds its document limit.
    pub fn to_ndjson(&self) -> Result<String, PerfError> {
        self.validate()?;
        let mut output = String::new();
        let _ = write!(
            output,
            "{{\"schema\":\"{}\",\"kind\":\"summary\",\"gate\":\"{}\",\
             \"scenario\":\"{}\",\"scope\":\"{}\",\"verdict\":\"{}\",\
             \"key_digest\":\"{}\",\"stats\":",
            self.schema,
            self.gate,
            escape_json(&self.scenario),
            self.scope.name(),
            self.verdict.name(),
            self.key_digest,
        );
        if let Some(stats) = self.stats {
            let _ = write!(
                output,
                "{{\"total\":{},\"valid\":{},\"invalid\":{},\"min\":{},\
                 \"median\":{},\"p95\":{},\"p99\":{},\"max\":{},\"mad\":{},\
                 \"mad_bps\":{}}}",
                stats.total_samples,
                stats.valid_samples,
                stats.invalid_samples,
                stats.min,
                stats.median,
                stats.p95,
                stats.p99,
                stats.max,
                stats.mad,
                stats.mad_bps,
            );
        } else {
            output.push_str("null");
        }
        output.push_str(",\"baseline_median\":");
        push_optional_number(&mut output, self.baseline_median);
        output.push_str(",\"regression_bps\":");
        push_optional_number(&mut output, self.regression_bps);
        let _ = writeln!(output, ",\"finding_count\":{}}}", self.findings.len());
        for finding in &self.findings {
            let _ = writeln!(
                output,
                "{{\"schema\":\"{}\",\"kind\":\"finding\",\"gate\":\"{}\",\
                 \"scenario\":\"{}\",\"code\":\"{}\",\"detail\":\"{}\"}}",
                self.schema,
                self.gate,
                escape_json(&self.scenario),
                finding.code,
                escape_json(&finding.detail),
            );
        }
        if output.len() > MAX_REPORT_BYTES {
            return Err(PerfError::Report(format!(
                "rendered report is {} bytes, exceeding the {MAX_REPORT_BYTES}-byte limit",
                output.len()
            )));
        }
        Ok(output)
    }
}

/// Malformed policy, baseline, run, evidence, or robot report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PerfError {
    /// Policy invariant.
    Policy(String),
    /// Policy catalog with source line.
    Catalog {
        /// One-based line, or zero for whole-file coverage.
        line: usize,
        /// Diagnostic.
        detail: String,
    },
    /// Baseline invariant.
    Baseline(String),
    /// Comparison identity.
    Identity(String),
    /// Raw samples.
    Samples(String),
    /// Artifact identity/path.
    Evidence(String),
    /// Robot report invariant or resource envelope.
    Report(String),
}

impl fmt::Display for PerfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Policy(detail) => write!(formatter, "performance policy: {detail}"),
            Self::Catalog { line, detail } if *line == 0 => {
                write!(formatter, "performance policy catalog: {detail}")
            }
            Self::Catalog { line, detail } => {
                write!(formatter, "performance policy catalog:{line}: {detail}")
            }
            Self::Baseline(detail) => write!(formatter, "performance baseline: {detail}"),
            Self::Identity(detail) => write!(formatter, "performance identity: {detail}"),
            Self::Samples(detail) => write!(formatter, "performance samples: {detail}"),
            Self::Evidence(detail) => write!(formatter, "performance evidence: {detail}"),
            Self::Report(detail) => write!(formatter, "performance report: {detail}"),
        }
    }
}

impl std::error::Error for PerfError {}

const BASELINE_FIELDS: [&str; 45] = [
    "schema",
    "generation",
    "gate",
    "scenario",
    "unit",
    "direction",
    "target",
    "min_valid_samples",
    "max_invalid_samples",
    "max_mad_bps",
    "alert_regression_bps",
    "block_regression_bps",
    "enforcement",
    "scope",
    "require_regression_profile",
    "profile_id",
    "build_profile",
    "host_fingerprint",
    "toolchain_fingerprint",
    "suite_lock_digest",
    "benchmark_definition",
    "engine",
    "tier",
    "thread_profile",
    "execution_plan_digest",
    "config_digest",
    "cache_state",
    "output_mode",
    "external_tool_fingerprint",
    "bare_metal",
    "isolated",
    "producer_commit",
    "observed_total",
    "observed_valid",
    "observed_invalid",
    "observed_min",
    "observed_median",
    "observed_p95",
    "observed_p99",
    "observed_max",
    "observed_mad",
    "observed_mad_bps",
    "source_kind",
    "source_path",
    "source_digest",
];

const OBSERVATION_FIELDS: [&str; 13] = [
    "observed_total",
    "observed_valid",
    "observed_invalid",
    "observed_min",
    "observed_median",
    "observed_p95",
    "observed_p99",
    "observed_max",
    "observed_mad",
    "observed_mad_bps",
    "source_kind",
    "source_path",
    "source_digest",
];

fn parse_baseline_fields(text: &str) -> Result<BTreeMap<String, String>, PerfError> {
    if text.len() > MAX_BASELINE_BYTES {
        return Err(PerfError::Baseline(format!(
            "baseline is {} bytes, exceeding the {MAX_BASELINE_BYTES}-byte limit",
            text.len()
        )));
    }
    let allowed: BTreeSet<_> = BASELINE_FIELDS.iter().copied().collect();
    let mut fields = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim_end();
        if trimmed.trim().is_empty() || trimmed.trim_start().starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('\t') else {
            return Err(PerfError::Baseline(format!(
                "line {line} is not key-tab-value"
            )));
        };
        if !allowed.contains(key) {
            return Err(PerfError::Baseline(format!(
                "line {line} has unknown key {}",
                bounded_debug(key)
            )));
        }
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(PerfError::Baseline(format!(
                "line {line} duplicates key {}",
                bounded_debug(key)
            )));
        }
    }
    Ok(fields)
}

fn parse_baseline_number<T>(
    fields: &BTreeMap<String, String>,
    name: &'static str,
) -> Result<T, PerfError>
where
    T: std::str::FromStr,
{
    fields
        .get(name)
        .ok_or_else(|| PerfError::Baseline(format!("missing {name}")))?
        .parse()
        .map_err(|_| PerfError::Baseline(format!("bad {name}")))
}

fn split_exact_tsv_fields<const N: usize>(line: &str) -> Result<[&str; N], usize> {
    let field_count = line.split('\t').count();
    if field_count != N {
        return Err(field_count);
    }

    let mut fields = line.split('\t');
    let mut exact = [""; N];
    for field in &mut exact {
        let Some(value) = fields.next() else {
            return Err(field_count);
        };
        *field = value;
    }
    Ok(exact)
}

fn bounded_debug(value: &str) -> String {
    if value.len() <= MAX_DIAGNOSTIC_VALUE_BYTES {
        return format!("{value:?}");
    }

    let mut end = MAX_DIAGNOSTIC_VALUE_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{:?}... <{} bytes total>", &value[..end], value.len())
}

fn own_raw_token(value: &str, name: &'static str) -> Result<String, PerfError> {
    validate_token(name, value)?;
    Ok(value.to_owned())
}

fn next_raw_header<'a, I>(lines: &mut I, expected: &'static str) -> Result<&'a str, PerfError>
where
    I: Iterator<Item = &'a str>,
{
    let line = lines
        .next()
        .ok_or_else(|| PerfError::Samples(format!("missing raw-bundle header {expected}")))?;
    let Some((name, value)) = line.split_once('\t') else {
        return Err(PerfError::Samples(format!(
            "raw-bundle header {expected} is not key-tab-value"
        )));
    };
    if name != expected || value.contains('\t') {
        return Err(PerfError::Samples(format!(
            "expected raw-bundle header {expected:?}, found {}",
            bounded_debug(line)
        )));
    }
    Ok(value)
}

fn parse_raw_number<T>(value: &str, name: &'static str) -> Result<T, PerfError>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| PerfError::Samples(format!("bad raw-bundle {name} {}", bounded_debug(value))))
}

fn parse_raw_digest(value: &str, name: &'static str) -> Result<Digest, PerfError> {
    Digest::from_hex(value)
        .map_err(|error| PerfError::Samples(format!("bad raw-bundle {name}: {error}")))
}

fn parse_optional_raw_digest(value: &str, name: &'static str) -> Result<Option<Digest>, PerfError> {
    if value == NONE {
        Ok(None)
    } else {
        parse_raw_digest(value, name).map(Some)
    }
}

fn parse_number<T>(value: &str, name: &'static str, line: usize) -> Result<T, PerfError>
where
    T: std::str::FromStr,
{
    value.parse().map_err(|_| PerfError::Catalog {
        line,
        detail: format!("bad {name} {}", bounded_debug(value)),
    })
}

fn parse_optional_u64(value: &str) -> Result<Option<u64>, String> {
    if value == NONE {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|_| format!("bad target {}", bounded_debug(value)))
    }
}

fn parse_digest(value: &str, name: &'static str) -> Result<Digest, PerfError> {
    Digest::from_hex(value).map_err(|error| PerfError::Baseline(format!("bad {name}: {error}")))
}

fn parse_optional_baseline_digest(
    value: &str,
    name: &'static str,
) -> Result<Option<Digest>, PerfError> {
    if value == NONE {
        Ok(None)
    } else {
        parse_digest(value, name).map(Some)
    }
}

fn optional_stat<T, U>(stats: Option<RobustStats>, project: T) -> String
where
    T: FnOnce(RobustStats) -> U,
    U: ToString,
{
    stats.map_or_else(|| NONE.to_owned(), |value| project(value).to_string())
}

fn validate_token(name: &'static str, value: &str) -> Result<(), PerfError> {
    if let Some(detail) = portable_token_error(name, value) {
        return Err(PerfError::Identity(detail));
    }
    Ok(())
}

fn validate_report_token(name: &'static str, value: &str) -> Result<(), PerfError> {
    if let Some(detail) = portable_token_error(name, value) {
        return Err(PerfError::Report(detail));
    }
    Ok(())
}

fn portable_token_error(name: &str, value: &str) -> Option<String> {
    if value.is_empty() || value.len() > MAX_TOKEN_BYTES {
        return Some(format!(
            "{name} length must be 1..={MAX_TOKEN_BYTES}, got {}",
            value.len()
        ));
    }
    if !value.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
    }) {
        return Some(format!(
            "{name} is not a lowercase portable token: {value:?}"
        ));
    }
    None
}

fn validate_detail(name: &'static str, value: &str) -> Result<(), PerfError> {
    if value.is_empty() || value.len() > MAX_DETAIL_BYTES || value.chars().any(char::is_control) {
        return Err(PerfError::Samples(format!(
            "{name} must be nonempty, at most {MAX_DETAIL_BYTES} bytes, and control-free"
        )));
    }
    Ok(())
}

/// Validate the canonical producer-commit provenance field.
///
/// Producers call this before profile checks or workload setup so malformed
/// provenance cannot consume timing work or create persistent scratch state.
///
/// # Errors
/// Returns an identity error unless `value` is exactly 40 lowercase
/// hexadecimal characters.
pub fn validate_producer_commit(value: &str) -> Result<(), PerfError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(PerfError::Identity(format!(
            "producer_commit must be 40 lowercase hex characters, got {}",
            bounded_debug(value)
        )));
    }
    Ok(())
}

fn validate_evidence_path(value: &str) -> Result<(), PerfError> {
    if value.len() > MAX_EVIDENCE_PATH_BYTES
        || !value.starts_with("tests/artifacts/perf/")
        || value.ends_with('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(PerfError::Evidence(format!(
            "artifact path must be a canonical file below tests/artifacts/perf/: {}",
            bounded_debug(value)
        )));
    }
    for component in value.split('/') {
        if component.is_empty() || matches!(component, "." | "..") {
            return Err(PerfError::Evidence(format!(
                "artifact path has an unsafe component: {}",
                bounded_debug(value)
            )));
        }
    }
    Ok(())
}

fn median(values: &[u64]) -> Option<u64> {
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        let lower = values.get(middle.checked_sub(1)?)?;
        let upper = values.get(middle)?;
        Some(*lower + upper.saturating_sub(*lower) / 2)
    } else {
        values.get(middle).copied()
    }
}

fn nearest_rank(values: &[u64], percentile: usize) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    let rank = percentile
        .saturating_mul(values.len())
        .div_ceil(100)
        .saturating_sub(1)
        .min(values.len() - 1);
    values.get(rank).copied()
}

fn ratio_bps(numerator: u64, denominator: u64) -> u32 {
    if numerator == 0 {
        return 0;
    }
    if denominator == 0 {
        return u32::MAX;
    }
    let rounded = (u128::from(numerator) * BPS_DENOMINATOR + u128::from(denominator) / 2)
        / u128::from(denominator);
    u32::try_from(rounded).unwrap_or(u32::MAX)
}

fn regression_bps(direction: Direction, baseline: &RobustStats, current: &RobustStats) -> u32 {
    let worse_by = match direction {
        Direction::AtMost => current.median.saturating_sub(baseline.median),
        Direction::AtLeast => baseline.median.saturating_sub(current.median),
        Direction::Exactly => {
            if current.min == baseline.min && current.max == baseline.max {
                0
            } else {
                return u32::MAX;
            }
        }
    };
    ratio_bps(worse_by, baseline.median)
}

const fn crosses_regression_threshold(regression_bps: u32, threshold_bps: u32) -> bool {
    regression_bps != 0 && regression_bps >= threshold_bps
}

const fn meets(direction: Direction, current: &RobustStats, target: u64) -> bool {
    match direction {
        Direction::AtMost => current.median <= target,
        Direction::AtLeast => current.median >= target,
        Direction::Exactly => current.min == target && current.max == target,
    }
}

const fn miss_verdict(enforcement: Enforcement) -> Verdict {
    match enforcement {
        Enforcement::Observe | Enforcement::Alert => Verdict::Alert,
        Enforcement::Blocking => Verdict::Block,
    }
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hash.update(&length.to_le_bytes());
    hash.update(bytes);
}

fn identity_difference(expected: &BenchmarkKey, found: &BenchmarkKey) -> String {
    let mut fields = Vec::new();
    if expected.profile_id != found.profile_id {
        fields.push("profile_id");
    }
    if expected.build_profile != found.build_profile {
        fields.push("build_profile");
    }
    if expected.host_fingerprint != found.host_fingerprint {
        fields.push("host_fingerprint");
    }
    if expected.toolchain_fingerprint != found.toolchain_fingerprint {
        fields.push("toolchain_fingerprint");
    }
    if expected.suite_lock_digest != found.suite_lock_digest {
        fields.push("suite_lock_digest");
    }
    if expected.benchmark_definition != found.benchmark_definition {
        fields.push("benchmark_definition");
    }
    if expected.gate != found.gate {
        fields.push("gate");
    }
    if expected.scenario != found.scenario {
        fields.push("scenario");
    }
    if expected.unit != found.unit {
        fields.push("unit");
    }
    if expected.engine != found.engine {
        fields.push("engine");
    }
    if expected.tier != found.tier {
        fields.push("tier");
    }
    if expected.thread_profile != found.thread_profile {
        fields.push("thread_profile");
    }
    if expected.execution_plan_digest != found.execution_plan_digest {
        fields.push("execution_plan_digest");
    }
    if expected.config_digest != found.config_digest {
        fields.push("config_digest");
    }
    if expected.cache_state != found.cache_state {
        fields.push("cache_state");
    }
    if expected.output_mode != found.output_mode {
        fields.push("output_mode");
    }
    if expected.external_tool_fingerprint != found.external_tool_fingerprint {
        fields.push("external_tool_fingerprint");
    }
    if expected.bare_metal != found.bare_metal {
        fields.push("bare_metal");
    }
    if expected.isolated != found.isolated {
        fields.push("isolated");
    }
    format!("incomparable fields: {}", fields.join(", "))
}

fn escape_json(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output
}

fn push_optional_number<T>(output: &mut String, value: Option<T>)
where
    T: fmt::Display,
{
    if let Some(value) = value {
        let _ = write!(output, "{value}");
    } else {
        output.push_str("null");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_hash::sha256;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn compiled_cargo_profile_is_an_exact_portable_identity() {
        assert!(!COMPILED_CARGO_PROFILE.is_empty());
        assert!(COMPILED_CARGO_PROFILE.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        }));
        // A profile Cargo can actually select for this workspace: the
        // build-directory layout change once made this `build`, and every
        // release-perf producer then refused while this test stayed green.
        assert!(
            matches!(
                COMPILED_CARGO_PROFILE,
                "dev" | "test" | "release" | "bench" | "release-perf"
            ),
            "compiled Cargo profile {COMPILED_CARGO_PROFILE:?} is not a declared profile"
        );
        assert_eq!(
            require_compiled_cargo_profile(COMPILED_CARGO_PROFILE),
            Ok(())
        );
        let other = if COMPILED_CARGO_PROFILE == "release-perf" {
            "dev"
        } else {
            "release-perf"
        };
        assert_eq!(
            require_compiled_cargo_profile(other),
            Err(PerfError::Identity(format!(
                "measurement requires Cargo profile {other:?}, but this artifact was compiled with {COMPILED_CARGO_PROFILE:?}"
            )))
        );
    }

    #[test]
    #[ignore = "run explicitly under each Cargo profile with FMN_TEST_EXPECT_COMPILED_PROFILE"]
    fn compiled_cargo_profile_matrix_probe() {
        let expected = std::env::var("FMN_TEST_EXPECT_COMPILED_PROFILE")
            .expect("profile-matrix probe requires its expected profile");
        assert_eq!(COMPILED_CARGO_PROFILE, expected);
        assert_eq!(
            require_compiled_cargo_profile("release-perf").is_ok(),
            expected == "release-perf"
        );
    }

    fn policy() -> GatePolicy {
        GatePolicy {
            gate: GateId::Pg2,
            scenario: "fill-canonical".to_owned(),
            unit: MetricUnit::MegaPixelsPerSecondMilli,
            direction: Direction::AtLeast,
            target: Some(300_000),
            min_valid_samples: 5,
            max_invalid_samples: 1,
            max_mad_bps: 1_000,
            alert_regression_bps: 500,
            block_regression_bps: 1_000,
            enforcement: Enforcement::Blocking,
            scope: GateScope::Core,
            require_regression_profile: true,
        }
    }

    fn complete_policy_catalog() -> Vec<GatePolicy> {
        GateId::ALL
            .into_iter()
            .map(|gate| {
                let (scope, unit, direction, target) = match gate {
                    GateId::Pg8 => (
                        GateScope::PythonOnly,
                        MetricUnit::RatioPpm,
                        Direction::AtMost,
                        Some(1_100_000),
                    ),
                    GateId::PgA => (
                        GateScope::AnnexOnly,
                        MetricUnit::FramesPerSecondMilli,
                        Direction::AtLeast,
                        None,
                    ),
                    _ => (
                        GateScope::Core,
                        MetricUnit::Nanoseconds,
                        Direction::AtMost,
                        Some(1),
                    ),
                };
                GatePolicy {
                    gate,
                    scenario: format!("{}-fixture", gate.name().replace('-', "")),
                    unit,
                    direction,
                    target,
                    min_valid_samples: 3,
                    max_invalid_samples: 0,
                    max_mad_bps: 1_000,
                    alert_regression_bps: 500,
                    block_regression_bps: 1_000,
                    enforcement: Enforcement::Blocking,
                    scope,
                    require_regression_profile: true,
                }
            })
            .collect()
    }

    fn key() -> BenchmarkKey {
        BenchmarkKey {
            profile_id: "linux-x86-64-8c-v1".to_owned(),
            build_profile: "release-perf".to_owned(),
            host_fingerprint: sha256(b"host"),
            toolchain_fingerprint: sha256(b"toolchain"),
            suite_lock_digest: sha256(b"suite"),
            benchmark_definition: sha256(b"fixture"),
            gate: GateId::Pg2,
            scenario: "fill-canonical".to_owned(),
            unit: MetricUnit::MegaPixelsPerSecondMilli,
            engine: "fast-cpu".to_owned(),
            tier: "x86-64-v3".to_owned(),
            thread_profile: "fixed-8".to_owned(),
            execution_plan_digest: sha256(b"execution-plan"),
            config_digest: sha256(b"config"),
            cache_state: "warm".to_owned(),
            output_mode: "raw".to_owned(),
            external_tool_fingerprint: None,
            bare_metal: true,
            isolated: true,
        }
    }

    fn source(kind: EvidenceKind, name: &str) -> EvidenceRef {
        EvidenceRef::from_bytes(
            kind,
            format!("tests/artifacts/perf/run-1/{name}"),
            name.as_bytes(),
        )
        .unwrap()
    }

    fn samples(value: u64) -> Vec<Sample> {
        vec![
            Sample::valid(value - 2),
            Sample::valid(value - 1),
            Sample::valid(value),
            Sample::valid(value + 1),
            Sample::valid(value + 2),
        ]
    }

    fn measured(value: u64) -> MeasurementBatch {
        MeasurementBatch {
            key: key(),
            producer_commit: COMMIT.to_owned(),
            samples: samples(value),
            evidence: Vec::new(),
        }
    }

    fn observed_baseline(generation: u32, value: u64) -> Baseline {
        let batch = measured(value);
        Baseline::observed(
            generation,
            policy(),
            &batch,
            "tests/artifacts/perf/run-1/raw.tsv",
        )
        .unwrap()
    }

    #[test]
    fn robust_stats_retain_invalid_samples_and_use_integer_mad() {
        let mut values = samples(100);
        values.push(Sample::invalid(1_000_000, "host-load-spike"));
        let stats = RobustStats::from_samples(&values, 5).unwrap();
        assert_eq!(stats.total_samples, 6);
        assert_eq!(stats.valid_samples, 5);
        assert_eq!(stats.invalid_samples, 1);
        assert_eq!(stats.median, 100);
        assert_eq!(stats.mad, 1);
        assert_eq!(stats.mad_bps, 100);
        assert_eq!(stats.p95, 102);
        assert_eq!(stats.p99, 102);
    }

    #[test]
    fn robust_stats_reject_a_zero_minimum_without_panicking() {
        assert!(RobustStats::from_samples(&[], 0).is_err());
    }

    #[test]
    fn raw_bundle_retains_invalid_samples_and_binds_the_baseline_digest() {
        let mut batch = measured(320_000);
        batch
            .samples
            .push(Sample::invalid(999_999, "thermal-throttle"));
        batch.evidence = vec![
            source(EvidenceKind::PhaseTrace, "z-spans.ndjson"),
            source(EvidenceKind::CpuProfile, "a-cpu.json"),
        ];
        let raw = batch.to_tsv().unwrap();
        assert_eq!(
            raw.lines().count(),
            23 + batch.samples.len() + batch.evidence.len()
        );
        assert!(raw.contains("sample\t5\t999999\tinvalid\tthermal-throttle"));
        assert!(
            raw.find("a-cpu.json").unwrap() < raw.find("z-spans.ndjson").unwrap(),
            "evidence records must be canonicalized by path"
        );

        let baseline =
            Baseline::observed(1, policy(), &batch, "tests/artifacts/perf/run-1/raw.tsv").unwrap();
        assert_eq!(
            baseline.observation.as_ref().unwrap().source.digest,
            sha256(raw.as_bytes())
        );
        assert!(baseline.verify_observation_source(raw.as_bytes()).is_ok());
        assert!(
            baseline
                .verify_observation_source(b"corrupted")
                .unwrap_err()
                .to_string()
                .contains("digest mismatch")
        );
        let reparsed = MeasurementBatch::from_tsv(&raw).unwrap();
        assert_eq!(reparsed.key, batch.key);
        assert_eq!(reparsed.samples, batch.samples);
        assert_eq!(reparsed.to_tsv().unwrap(), raw);
        assert!(baseline.verify_observation_batch(&reparsed).is_ok());
    }

    #[test]
    fn raw_bundle_fixed_width_rows_refuse_short_and_long_records() {
        let raw = measured(320_000).to_tsv().expect("fixture must serialize");
        let sample = raw
            .lines()
            .find(|line| line.starts_with("sample\t0\t"))
            .expect("fixture must contain its first sample");

        let short_sample = raw.replacen(sample, "sample\t0\t1", 1);
        let error = MeasurementBatch::from_tsv(&short_sample)
            .expect_err("a short sample record must fail closed");
        assert!(
            error
                .to_string()
                .contains("sample 0 has 3 fields, expected 5")
        );

        let long_sample = format!("{sample}{}", "\textra".repeat(1_024));
        let long_raw = raw.replacen(sample, &long_sample, 1);
        let error = MeasurementBatch::from_tsv(&long_raw)
            .expect_err("a long sample record must fail closed");
        assert!(
            error
                .to_string()
                .contains("sample 0 has 1029 fields, expected 5")
        );

        let mut with_evidence = measured(320_000);
        with_evidence.evidence = vec![source(EvidenceKind::PhaseTrace, "trace.tsv")];
        let raw = with_evidence
            .to_tsv()
            .expect("evidence fixture must serialize");
        let evidence = raw
            .lines()
            .find(|line| line.starts_with("evidence\t0\t"))
            .expect("fixture must contain its evidence record");

        let short_evidence = raw.replacen(evidence, "evidence\t0\tphase-trace", 1);
        let error = MeasurementBatch::from_tsv(&short_evidence)
            .expect_err("a short evidence record must fail closed");
        assert!(
            error
                .to_string()
                .contains("evidence 0 has 3 fields, expected 5")
        );

        let long_evidence = format!("{evidence}{}", "\textra".repeat(1_024));
        let long_raw = raw.replacen(evidence, &long_evidence, 1);
        let error = MeasurementBatch::from_tsv(&long_raw)
            .expect_err("a long evidence record must fail closed");
        assert!(
            error
                .to_string()
                .contains("evidence 0 has 1029 fields, expected 5")
        );
    }

    #[test]
    fn malformed_raw_bundle_values_are_validated_before_owning_and_reported_boundedly() {
        let megabyte = "x".repeat(1024 * 1024);

        let schema_error = MeasurementBatch::from_tsv(&format!("schema\t{megabyte}\n"))
            .expect_err("an oversized schema value must fail closed")
            .to_string();
        assert!(schema_error.len() < 512, "{schema_error}");
        assert!(schema_error.contains("1048576 bytes total"));

        let raw = measured(320_000).to_tsv().expect("fixture must serialize");
        let profile_line = raw
            .lines()
            .find(|line| line.starts_with("profile_id\t"))
            .expect("fixture must contain profile_id");
        let oversized_profile = format!("profile_id\t{megabyte}");
        let malformed = raw.replacen(profile_line, &oversized_profile, 1);
        let profile_error = MeasurementBatch::from_tsv(&malformed)
            .expect_err("an oversized profile token must fail before ownership")
            .to_string();
        assert!(profile_error.len() < 256, "{profile_error}");
        assert!(profile_error.contains("profile_id length"));
        assert!(profile_error.contains("1048576"));

        let sample = raw
            .lines()
            .find(|line| line.starts_with("sample\t0\t"))
            .expect("fixture must contain its first sample");
        let oversized_reason = format!("sample\t0\t319998\tinvalid\t{megabyte}");
        let malformed = raw.replacen(sample, &oversized_reason, 1);
        let reason_error = MeasurementBatch::from_tsv(&malformed)
            .expect_err("an oversized invalid reason must fail before ownership")
            .to_string();
        assert!(reason_error.len() < 256, "{reason_error}");
        assert!(reason_error.contains("invalid_reason must be nonempty"));

        let oversized_value = format!("sample\t0\t{megabyte}\tvalid\t-");
        let malformed = raw.replacen(sample, &oversized_value, 1);
        let number_error = MeasurementBatch::from_tsv(&malformed)
            .expect_err("an oversized numeric field must have a bounded diagnostic")
            .to_string();
        assert!(number_error.len() < 512, "{number_error}");
        assert!(number_error.contains("1048576 bytes total"));

        let mut with_evidence = measured(320_000);
        with_evidence.evidence = vec![source(EvidenceKind::PhaseTrace, "trace.tsv")];
        let evidence_raw = with_evidence
            .to_tsv()
            .expect("evidence fixture must serialize");
        let evidence = evidence_raw
            .lines()
            .find(|line| line.starts_with("evidence\t0\t"))
            .expect("fixture must contain its evidence record");
        let oversized_path = format!(
            "evidence\t0\tphase-trace\ttests/artifacts/perf/{megabyte}\t{}",
            sha256(b"trace")
        );
        let malformed = evidence_raw.replacen(evidence, &oversized_path, 1);
        let path_error = MeasurementBatch::from_tsv(&malformed)
            .expect_err("an oversized evidence path must fail before ownership")
            .to_string();
        assert!(path_error.len() < 512, "{path_error}");
        assert!(path_error.contains("1048597 bytes total"));

        let trailing_error = MeasurementBatch::from_tsv(&format!("{raw}{megabyte}\n"))
            .expect_err("an oversized trailing record must have a bounded diagnostic")
            .to_string();
        assert!(trailing_error.len() < 512, "{trailing_error}");
        assert!(trailing_error.contains("1048576 bytes total"));
    }

    #[test]
    fn bounded_diagnostic_preview_preserves_short_text_and_utf8_boundaries() {
        assert_eq!(bounded_debug("bad\nvalue"), r#""bad\nvalue""#);

        let multibyte = "💥".repeat(100);
        let preview = bounded_debug(&multibyte);
        assert!(preview.len() < 512, "{preview}");
        assert!(preview.contains("400 bytes total"));
    }

    #[test]
    fn an_unobserved_target_is_inconclusive_not_green() {
        let baseline = Baseline::targeted(1, policy(), key(), COMMIT).unwrap();
        let run = measured(310_000);
        let report = baseline.evaluate(None, &run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "baseline-unobserved");
    }

    #[test]
    fn identity_drift_is_inconclusive_not_a_regression() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let mut run = measured(100_000);
        run.key.toolchain_fingerprint = sha256(b"different");
        let report = baseline.evaluate(Some(&baseline_source), &run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "identity-mismatch");
        assert!(report.findings[0].detail.contains("toolchain_fingerprint"));
    }

    #[test]
    fn scheduler_and_external_tool_are_comparison_key_material() {
        let original = key();
        let mut changed_threads = original.clone();
        changed_threads.thread_profile = "fixed-16".to_owned();
        assert_ne!(changed_threads.digest(), original.digest());

        let mut changed_plan = original.clone();
        changed_plan.execution_plan_digest = sha256(b"different-plan");
        assert_ne!(changed_plan.digest(), original.digest());

        let mut changed_tool = original.clone();
        changed_tool.external_tool_fingerprint = Some(sha256(b"ffmpeg"));
        assert_ne!(changed_tool.digest(), original.digest());
    }

    #[test]
    fn shared_host_measurement_cannot_become_an_observed_baseline() {
        let mut batch = measured(320_000);
        batch.key.bare_metal = false;
        let result = Baseline::observed(1, policy(), &batch, "tests/artifacts/perf/run-1/raw.tsv");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("bare-metal and isolation-qualified")
        );
    }

    #[test]
    fn regression_drill_blocks_and_requires_a_profile() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let run = measured(270_000);
        let report = baseline.evaluate(Some(&baseline_source), &run);
        assert_eq!(report.verdict, Verdict::Block);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "target-miss")
        );
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
                .any(|finding| finding.code == "regression-profile-missing")
        );
    }

    #[test]
    fn exact_alert_threshold_is_not_an_off_by_one_safe_harbor() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let mut run = measured(304_000);
        run.evidence = vec![source(EvidenceKind::CpuProfile, "cpu.json")];
        let report = baseline.evaluate(Some(&baseline_source), &run);
        assert_eq!(report.regression_bps, Some(500));
        assert_eq!(report.verdict, Verdict::Alert);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "baseline-regression")
        );
    }

    #[test]
    fn a_profiled_in_envelope_run_passes() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let mut run = measured(319_000);
        run.evidence = vec![source(EvidenceKind::PhaseTrace, "spans.ndjson")];
        let report = baseline.evaluate(Some(&baseline_source), &run);
        assert_eq!(report.verdict, Verdict::Pass);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn high_dispersion_cannot_win_on_a_lucky_median() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let run = MeasurementBatch {
            key: key(),
            producer_commit: COMMIT.to_owned(),
            samples: vec![
                Sample::valid(100_000),
                Sample::valid(200_000),
                Sample::valid(320_000),
                Sample::valid(600_000),
                Sample::valid(900_000),
            ],
            evidence: Vec::new(),
        };
        let report = baseline.evaluate(Some(&baseline_source), &run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "excessive-dispersion");
    }

    #[test]
    fn repetition_plan_and_invalid_sample_budget_fail_closed() {
        let baseline = observed_baseline(1, 320_000);
        let baseline_source = measured(320_000);
        let mut longer_run = measured(320_000);
        longer_run.samples.push(Sample::valid(320_000));
        let report = baseline.evaluate(Some(&baseline_source), &longer_run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "sample-plan-mismatch");

        let mut noisy_source = measured(320_000);
        noisy_source.samples.push(Sample::valid(320_000));
        noisy_source
            .samples
            .push(Sample::invalid(999_999, "scheduler-interference"));
        let noisy_baseline = Baseline::observed(
            1,
            policy(),
            &noisy_source,
            "tests/artifacts/perf/run-1/noisy-raw.tsv",
        )
        .unwrap();
        let mut too_many_invalid = measured(320_000);
        too_many_invalid
            .samples
            .push(Sample::invalid(999_998, "thermal-throttle"));
        too_many_invalid
            .samples
            .push(Sample::invalid(999_999, "scheduler-interference"));
        let report = noisy_baseline.evaluate(Some(&noisy_source), &too_many_invalid);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "invalid-sample-budget-exceeded");
    }

    #[test]
    fn observed_and_targeted_baselines_round_trip() {
        let observed = observed_baseline(2, 320_000);
        let observed_tsv = observed.to_tsv().unwrap();
        assert_eq!(Baseline::from_tsv(&observed_tsv).unwrap(), observed);

        let targeted = Baseline::targeted(3, policy(), key(), COMMIT).unwrap();
        let targeted_tsv = targeted.to_tsv().unwrap();
        assert_eq!(Baseline::from_tsv(&targeted_tsv).unwrap(), targeted);
    }

    #[test]
    fn baseline_writer_rejects_invalid_or_oversized_public_state() {
        let mut invalid = Baseline::targeted(1, policy(), key(), COMMIT).unwrap();
        invalid.generation = 0;
        assert!(
            invalid
                .to_tsv()
                .unwrap_err()
                .to_string()
                .contains("generation must be nonzero")
        );

        let mut oversized = Baseline::targeted(1, policy(), key(), COMMIT).unwrap();
        oversized.policy.scenario = "x".repeat(MAX_BASELINE_BYTES);
        oversized
            .key
            .scenario
            .clone_from(&oversized.policy.scenario);
        let error = oversized.to_tsv().unwrap_err().to_string();
        assert!(error.contains("scenario length must be"), "{error}");

        let canonical = Baseline::targeted(1, policy(), key(), COMMIT)
            .unwrap()
            .to_tsv()
            .unwrap();
        assert!(canonical.len() <= MAX_BASELINE_BYTES);
    }

    #[test]
    fn deserialized_baseline_rejects_internally_inconsistent_statistics() {
        let observed = observed_baseline(2, 320_000);
        let bad_counts = observed
            .to_tsv()
            .unwrap()
            .replace("observed_total\t5\n", "observed_total\t6\n");
        assert!(
            Baseline::from_tsv(&bad_counts)
                .unwrap_err()
                .to_string()
                .contains("sample counts disagree")
        );

        let bad_ratio = observed
            .to_tsv()
            .unwrap()
            .replace("observed_mad_bps\t0\n", "observed_mad_bps\t1\n");
        assert!(
            Baseline::from_tsv(&bad_ratio)
                .unwrap_err()
                .to_string()
                .contains("MAD ratio")
        );

        let plausible_but_wrong = observed
            .to_tsv()
            .unwrap()
            .replace("observed_p95\t320002\n", "observed_p95\t320001\n");
        let manipulated = Baseline::from_tsv(&plausible_but_wrong).unwrap();
        assert!(
            manipulated
                .verify_observation_batch(&measured(320_000))
                .unwrap_err()
                .to_string()
                .contains("differ from stored observation")
        );
        let run = measured(320_000);
        let report = manipulated.evaluate(Some(&run), &run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "invalid-baseline-source");
    }

    #[test]
    fn an_observed_baseline_cannot_pass_without_raw_source_replay() {
        let baseline = observed_baseline(1, 320_000);
        let run = measured(320_000);
        let report = baseline.evaluate(None, &run);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(report.findings[0].code, "baseline-source-unverified");
    }

    #[test]
    fn exact_zero_baseline_passes_zero_and_blocks_nonzero() {
        let mut exact_policy = policy();
        exact_policy.gate = GateId::Pg5;
        exact_policy.scenario = "certified-thread-matrix".to_owned();
        exact_policy.unit = MetricUnit::Mismatches;
        exact_policy.direction = Direction::Exactly;
        exact_policy.target = Some(0);
        exact_policy.min_valid_samples = 3;
        exact_policy.max_invalid_samples = 0;
        exact_policy.max_mad_bps = 0;
        exact_policy.alert_regression_bps = 0;
        exact_policy.block_regression_bps = 0;
        exact_policy.require_regression_profile = false;
        let mut exact_key = key();
        exact_key.gate = GateId::Pg5;
        exact_key.scenario = exact_policy.scenario.clone();
        exact_key.unit = MetricUnit::Mismatches;
        exact_key.thread_profile = "matrix-1-4-16".to_owned();
        let run = |value| MeasurementBatch {
            key: exact_key.clone(),
            producer_commit: COMMIT.to_owned(),
            samples: vec![
                Sample::valid(value),
                Sample::valid(value),
                Sample::valid(value),
            ],
            evidence: Vec::new(),
        };
        let one_mismatch = MeasurementBatch {
            key: exact_key.clone(),
            producer_commit: COMMIT.to_owned(),
            samples: vec![Sample::valid(0), Sample::valid(0), Sample::valid(1)],
            evidence: Vec::new(),
        };
        let varied_mismatches = MeasurementBatch {
            key: exact_key.clone(),
            producer_commit: COMMIT.to_owned(),
            samples: vec![Sample::valid(0), Sample::valid(1), Sample::valid(2)],
            evidence: Vec::new(),
        };
        let baseline_batch = run(0);
        let invalid_baseline = Baseline::observed(
            1,
            exact_policy.clone(),
            &one_mismatch,
            "tests/artifacts/perf/run-1/invalid-identity.tsv",
        );
        assert!(
            invalid_baseline
                .unwrap_err()
                .to_string()
                .contains("does not equal its target")
        );
        let baseline = Baseline::observed(
            1,
            exact_policy,
            &baseline_batch,
            "tests/artifacts/perf/run-1/identity.tsv",
        )
        .unwrap();
        assert_eq!(
            baseline.evaluate(Some(&baseline_batch), &run(0)).verdict,
            Verdict::Pass
        );
        assert_eq!(
            baseline.evaluate(Some(&baseline_batch), &run(1)).verdict,
            Verdict::Block
        );
        let outlier_report = baseline.evaluate(Some(&baseline_batch), &one_mismatch);
        assert_eq!(
            outlier_report.stats.map(|stats| stats.median),
            Some(0),
            "the adversarial sample set must retain the median-zero shape"
        );
        assert_eq!(outlier_report.verdict, Verdict::Block);
        assert!(
            outlier_report
                .findings
                .iter()
                .any(|finding| finding.code == "target-miss")
        );
        assert_eq!(
            baseline
                .evaluate(Some(&baseline_batch), &varied_mismatches)
                .verdict,
            Verdict::Block,
            "dispersion cannot mask an exact invariant failure"
        );
    }

    #[test]
    fn only_annex_policies_may_omit_an_absolute_target() {
        let mut core = policy();
        core.target = None;
        assert!(
            core.validate()
                .unwrap_err()
                .to_string()
                .contains("outside the PG-A baseline-only plane")
        );
    }

    #[test]
    fn policy_catalog_is_strict_complete_and_canonical() {
        let policies = complete_policy_catalog();
        let rendered = render_policy_catalog(&policies).expect("valid policies render");
        let parsed = parse_policy_catalog(&rendered).unwrap();
        assert_eq!(
            render_policy_catalog(&parsed).expect("parsed policies render"),
            rendered
        );

        let short = format!("schema\t{POLICY_SCHEMA}\npg-1\tshort\n");
        let error = parse_policy_catalog(&short)
            .expect_err("a short fixed-width policy row must fail closed");
        assert_eq!(
            error.to_string(),
            "performance policy catalog:2: policy row has 2 fields, expected 13"
        );

        let long = format!("schema\t{POLICY_SCHEMA}\n{}x\n", "x\t".repeat(13));
        let error = parse_policy_catalog(&long)
            .expect_err("a long fixed-width policy row must fail closed");
        assert_eq!(
            error.to_string(),
            "performance policy catalog:2: policy row has 14 fields, expected 13"
        );
    }

    #[test]
    fn policy_catalog_parser_rejects_noncanonical_bytes() {
        let canonical =
            render_policy_catalog(&complete_policy_catalog()).expect("valid policies render");

        let missing_final_lf = canonical
            .strip_suffix('\n')
            .expect("canonical catalog has a final LF");
        assert_eq!(
            parse_policy_catalog(missing_final_lf)
                .expect_err("missing final LF must fail")
                .to_string(),
            "performance policy catalog: catalog must end with a final LF newline"
        );

        let crlf = canonical.replace('\n', "\r\n");
        assert_eq!(
            parse_policy_catalog(&crlf)
                .expect_err("CRLF must fail")
                .to_string(),
            "performance policy catalog: catalog must use LF line endings"
        );

        let without_preamble = canonical
            .strip_prefix(POLICY_PREAMBLE)
            .expect("canonical catalog has its preamble");
        assert!(
            parse_policy_catalog(without_preamble)
                .expect_err("the canonical preamble is authority")
                .to_string()
                .contains("not in canonical form")
        );

        let leading_zero = canonical.replacen("\t3\t0\t", "\t03\t0\t", 1);
        assert!(
            parse_policy_catalog(&leading_zero)
                .expect_err("noncanonical decimal spelling must fail")
                .to_string()
                .contains("not in canonical form")
        );

        let mut lines: Vec<_> = canonical.lines().collect();
        lines.swap(6, 7);
        let reordered = format!("{}\n", lines.join("\n"));
        assert!(
            parse_policy_catalog(&reordered)
                .expect_err("policy rows must be canonical-order")
                .to_string()
                .contains("not in canonical form")
        );
    }

    #[test]
    fn policy_catalog_writer_refuses_invalid_or_oversized_authority() {
        let policies = complete_policy_catalog();

        let mut duplicate = policies.clone();
        duplicate.push(policies[0].clone());
        assert!(
            render_policy_catalog(&duplicate)
                .expect_err("duplicate identity must fail")
                .to_string()
                .contains("duplicate pg-1 scenario")
        );

        assert!(
            render_policy_catalog(&policies[..1])
                .expect_err("incomplete gate coverage must fail")
                .to_string()
                .contains("catalog has no pg-2 policy")
        );

        let mut invalid = policies.clone();
        invalid[0].scenario = "Bad-Scenario".to_owned();
        assert!(
            render_policy_catalog(&invalid)
                .expect_err("invalid policy fields must fail")
                .to_string()
                .contains("not a lowercase portable token")
        );

        let too_many = vec![policies[0].clone(); MAX_POLICY_ROWS + 1];
        assert!(
            render_policy_catalog(&too_many)
                .expect_err("row envelope must run before duplicate handling")
                .to_string()
                .contains("row limit")
        );

        let mut oversized = policies;
        for index in oversized.len()..MAX_POLICY_ROWS {
            let mut policy = oversized[0].clone();
            let prefix = format!("s{index:04}");
            policy.scenario = format!("{prefix}{}", "x".repeat(MAX_TOKEN_BYTES - prefix.len()));
            policy.target = Some(u64::MAX);
            policy.max_invalid_samples = MAX_SAMPLES - policy.min_valid_samples;
            policy.max_mad_bps = u32::MAX;
            policy.alert_regression_bps = u32::MAX;
            policy.block_regression_bps = u32::MAX;
            oversized.push(policy);
        }
        assert!(
            render_policy_catalog(&oversized)
                .expect_err("document envelope must be checked before output allocation")
                .to_string()
                .contains("rendered catalog exceeds")
        );
    }

    #[test]
    fn robot_output_escapes_detail_and_stays_line_oriented() {
        let report = GateReport::inconclusive(
            &policy(),
            key().digest(),
            "fixture",
            "quote \" and newline\nstay data".to_owned(),
        );
        let output = report.to_ndjson().unwrap();
        assert_eq!(output.lines().count(), 2);
        assert!(output.contains("quote \\\" and newline\\nstay data"));
        assert!(!output.contains("newline\nstay"));
    }

    #[test]
    fn report_writer_rejects_invalid_or_oversized_public_state() {
        let report = GateReport::inconclusive(
            &policy(),
            key().digest(),
            "fixture",
            "bounded detail".to_owned(),
        );

        let mut bad_schema = report.clone();
        bad_schema.schema = "fmn-perf-report/0\"";
        assert!(
            bad_schema
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("schema must be")
        );

        let mut oversized_scenario = report.clone();
        oversized_scenario.scenario = "x".repeat(MAX_REPORT_BYTES);
        assert!(
            oversized_scenario
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("scenario length")
        );

        let mut bad_code = report.clone();
        bad_code.findings[0].code = "bad\"\n";
        assert!(
            bad_code
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("finding code is not a lowercase portable token")
        );

        let mut bad_stats = report.clone();
        bad_stats.stats = Some(RobustStats {
            total_samples: 2,
            valid_samples: 1,
            invalid_samples: 0,
            min: 1,
            median: 1,
            p95: 1,
            p99: 1,
            max: 1,
            mad: 0,
            mad_bps: 0,
        });
        assert!(
            bad_stats
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("sample counts disagree")
        );

        let mut oversized_detail = report.clone();
        oversized_detail.findings[0].detail = "x".repeat(MAX_REPORT_BYTES);
        assert!(
            oversized_detail
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("finding detail length")
        );

        let mut too_many = report.clone();
        too_many.findings =
            vec![Finding::new("fixture", "bounded detail".to_owned()); MAX_REPORT_FINDINGS + 1];
        assert!(
            too_many
                .to_ndjson()
                .unwrap_err()
                .to_string()
                .contains("finding limit")
        );

        let mut maximum = report;
        maximum.scenario = "s".repeat(MAX_TOKEN_BYTES);
        maximum.findings =
            vec![Finding::new("f", "\u{1f}".repeat(MAX_DETAIL_BYTES)); MAX_REPORT_FINDINGS];
        let output = maximum.to_ndjson().unwrap();
        assert!(output.len() <= MAX_REPORT_BYTES);
        assert_eq!(output.lines().count(), MAX_REPORT_FINDINGS + 1);
    }

    #[test]
    fn evidence_paths_are_portable_and_cannot_traverse() {
        assert!(
            source(EvidenceKind::CpuProfile, "cpu.json")
                .validate()
                .is_ok()
        );
        for path in [
            "/tests/artifacts/perf/run/cpu.json",
            "tests/artifacts/perf/../cpu.json",
            "tests\\artifacts\\perf\\cpu.json",
            "docs/performance/cpu.json",
            "tests/artifacts/perf/run/",
        ] {
            let evidence = EvidenceRef {
                kind: EvidenceKind::CpuProfile,
                path: path.to_owned(),
                digest: sha256(b"x"),
            };
            assert!(evidence.validate().is_err(), "{path:?} was accepted");
        }
    }
}
