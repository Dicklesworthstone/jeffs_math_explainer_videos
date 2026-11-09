//! The deterministic in-crate fuzzing campaign (§16.5 plane 4, fm-t1v).
//!
//! # Why not cargo-fuzz
//!
//! The governed closure (D1) forbids new unreviewed dependencies, and
//! `libfuzzer-sys` sits outside the FrankenSuite. ADR-0003 settles the
//! split: the **coverage-guided** half is fm-ntp's `fuzz/` crate — not a
//! workspace member, never in the shipped graph, class=`fuzz` in
//! SUITE_ALLOWLIST.tsv. This module is the **deterministic in-crate half**:
//! a seeded xorshift driver that runs a fixed number of cases per target,
//! persists interesting inputs to `fixtures/fuzz_corpus/`, and gates CI on
//! the campaign's invariants. Every run exercises the identical inputs, so
//! any failure is a one-command repro with no fuzzer infrastructure at all.
//!
//! # The doctrine (§16.5, R14)
//!
//! Every target is an untrusted-input parser. Per case the campaign asserts:
//!
//! - **error precisely or succeed within budget** — a refusal is a typed,
//!   named, non-empty error (targets may tighten "precise"; the TeX target
//!   requires the text to name a construct or a byte position);
//! - **never hang** — the driver enforces per-case *work bounds*: input
//!   byte caps, bounded mutation counts, and target-internal limits
//!   (fmn-config's `Limits`, fmn-codec's pixel/chunk budgets, fmd-math's
//!   nesting cap, fmd-font's bounds-checked reads). Wall-clock timeouts are
//!   deliberately avoided: structural bounds keep the campaign
//!   deterministic and platform-independent;
//! - **never overallocate** — accepted outputs carry a declared byte size
//!   the driver checks against the target's output budget (the
//!   decompression-bomb refusal);
//! - **never panic** — every case runs under `catch_unwind`; a panic is a
//!   campaign violation with the input preserved for reproduction.
//!
//! # Determinism and the corpus
//!
//! Case `i` of target `T` is generated from a xorshift64 stream seeded by
//! mixing `T`'s campaign seed with `i`, so each case is independently
//! reproducible and the CI run (a reduced case count) is an exact prefix of
//! the scheduled full campaign (`FMN_FUZZ_FULL=1`). Interesting inputs —
//! the first case of each outcome class, and any violation — are persisted
//! as files under `fixtures/fuzz_corpus/<target>/`, content-addressed by
//! SHA-256 (via fmn-hash). `MANIFEST.tsv` is the versioned campaign record:
//! per-target seeds, case counts, budget bounds, and outcome classes.
//!
//! Like the golden rig (D-16), the corpus is checked, never auto-updated:
//! drift fails the test, and `FMN_FUZZ_BLESS=1 FMN_FUZZ_FULL=1` rewrites
//! the manifest and corpus for human review and commit. The rig never
//! deletes files — stale entries are reported for a human to remove.
//! Because every target is pure integer/byte work (no floats, no
//! platform-dependent behavior), one corpus serves every platform — no
//! per-platform locks are needed.
//!
//! The SVG document processor target is **pending** (fm-6nm has not
//! landed); the manifest records it as a pending row rather than stubbing
//! it.

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::fs::File;
use std::io::{self, Read};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use fmn_hash::sha256;

// ---------------------------------------------------------------- PRNG

/// xorshift64* — a small deterministic PRNG, no external crates (the same
/// construction fmn-geom's earclip tests use). The whole campaign's
/// reproducibility rests on this stream being platform-independent integer
/// arithmetic.
#[derive(Clone, Debug)]
pub struct XorShift64(u64);

impl XorShift64 {
    /// Seed from a campaign seed and a case index (splitmix64-style mix),
    /// so every case's stream is independent and case `i` is reproducible
    /// without replaying `0..i`.
    #[must_use]
    pub fn for_case(campaign_seed: u64, case_index: u64) -> Self {
        let mut z = campaign_seed.wrapping_add(case_index.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // A zero state sticks at zero; fold to a fixed nonzero constant.
        Self(if z == 0 { 0x9E37_79B9_7F4A_7C15 } else { z })
    }

    /// The next 64 bits of the stream.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..n` (`n` must be positive), rejection-free via the
    /// multiply-high trick — biased by at most `n / 2^64`, which is
    /// irrelevant here and keeps the stream consumption constant.
    #[must_use]
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        ((self.next_u64() as u128 * u128::from(n)) >> 64) as u64
    }

    /// True with probability `num / den`.
    #[must_use]
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        debug_assert!(num <= den && den > 0);
        self.below(den) < num
    }

    /// A byte of the stream.
    pub fn byte(&mut self) -> u8 {
        (self.next_u64() >> 32) as u8
    }
}

// ---------------------------------------------------------------- budgets

/// Per-target resource bounds, declared before any case runs and recorded
/// in the campaign manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budgets {
    /// Maximum input bytes a case may present (mutations never grow past
    /// this; oversized cases are truncated to it).
    pub max_input_bytes: u64,
    /// Maximum declared output bytes an accepted case may report (the
    /// decompression-bomb refusal), or `None` when the output is bounded
    /// by the input by construction (e.g. an owned parse tree).
    pub max_output_bytes: Option<u64>,
}

// ---------------------------------------------------------------- verdicts

/// What a target reports for one input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The input was accepted; `output_bytes` is the declared size of the
    /// result, checked against [`Budgets::max_output_bytes`].
    Accepted {
        /// Declared output size in bytes.
        output_bytes: u64,
    },
    /// The input was refused with a precise, named error. `class` is a
    /// STABLE short label (corpus file names and the manifest's outcome
    /// classes are built from it — lowercase `[a-z0-9-]`); `message` is
    /// the full error text, checked for precision.
    Refused {
        /// Stable outcome-class label.
        class: String,
        /// Full error text.
        message: String,
    },
    /// The target observed an internal condition that should be
    /// impossible for any input (e.g. an engine wiring fault, a failed
    /// re-encode round-trip). Always a campaign violation.
    Fault {
        /// What impossible thing happened.
        message: String,
    },
}

/// A campaign invariant failure, tied to the case that produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    /// The target panicked (caught by the driver's `catch_unwind`).
    Panic {
        /// Case index.
        case: u64,
    },
    /// A refusal whose error text failed the target's precision bar
    /// (empty, or — for TeX — naming no construct or position).
    ImpreciseError {
        /// Case index.
        case: u64,
        /// The offending error text (may be empty).
        message: String,
    },
    /// An accepted case whose declared output exceeded the budget.
    OutputOverBudget {
        /// Case index.
        case: u64,
        /// Declared output bytes.
        got: u64,
        /// The budget.
        max: u64,
    },
    /// The target reported [`Verdict::Fault`].
    TargetFault {
        /// Case index.
        case: u64,
        /// The fault description.
        message: String,
    },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic { case } => write!(f, "case {case}: target panicked"),
            Self::ImpreciseError { case, message } => {
                write!(f, "case {case}: imprecise error text {message:?}")
            }
            Self::OutputOverBudget { case, got, max } => {
                write!(
                    f,
                    "case {case}: output {got} bytes over the {max}-byte budget"
                )
            }
            Self::TargetFault { case, message } => {
                write!(f, "case {case}: target fault: {message}")
            }
        }
    }
}

// ---------------------------------------------------------------- targets

/// One fuzz target: an untrusted-input parser plus its structure-aware
/// mutators and budget assertions. Concrete targets live with the campaign
/// (`tests/fuzz_campaign.rs`); the driver here is target-agnostic.
pub trait Target {
    /// The manifest/corpus name: lowercase `[a-z0-9_-]`, stable forever.
    fn name(&self) -> &'static str;

    /// The declared resource bounds.
    fn budgets(&self) -> Budgets;

    /// The seed corpus: real, mostly-valid inputs the mutators perturb.
    /// Deterministic — built from bundled assets, owned encoders, or
    /// committed fixtures, never from the environment.
    fn seeds(&self) -> Vec<Vec<u8>>;

    /// Perturb `input` in place (structure-aware where the format earns
    /// it). Growth past `budgets().max_input_bytes` is the mutator's own
    /// responsibility to avoid.
    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>);

    /// Run one case. Must be total: every input yields a [`Verdict`].
    fn run(&self, input: &[u8]) -> Verdict;

    /// The precision bar for a refusal's error text. The default is
    /// non-emptiness; the TeX target overrides it to require a named
    /// construct or byte position (fmd-math's never-garble contract).
    fn refusal_is_precise(&self, message: &str) -> bool {
        !message.is_empty()
    }
}

// ---------------------------------------------------------------- driver

/// How many cases a target runs, and under which stream seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CampaignSpec {
    /// The campaign seed (with the case index, determines every case).
    pub seed: u64,
    /// Cases in the reduced CI run.
    pub ci_cases: u32,
    /// Cases in the scheduled full campaign (`FMN_FUZZ_FULL=1`).
    pub full_cases: u32,
}

/// An interesting input: the first case of an outcome class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interesting {
    /// Case index that produced it.
    pub case: u64,
    /// Outcome-class label (`accepted`, or the refusal class).
    pub class: String,
    /// The exact input bytes.
    pub bytes: Vec<u8>,
}

/// The aggregate result of running one target's campaign.
#[derive(Clone, Debug, Default)]
pub struct CampaignReport {
    /// Cases executed.
    pub cases_run: u64,
    /// Outcome class → case count (`accepted` plus every refusal class).
    pub class_counts: BTreeMap<String, u64>,
    /// First case per outcome class, in discovery order.
    pub interesting: Vec<Interesting>,
    /// Every invariant failure (empty when the campaign is healthy).
    pub violations: Vec<Violation>,
}

impl CampaignReport {
    /// The sorted outcome-class list the manifest records.
    #[must_use]
    pub fn classes(&self) -> Vec<String> {
        self.class_counts.keys().cloned().collect()
    }

    /// A one-line human summary (printed by the campaign test).
    #[must_use]
    pub fn summary_line(&self, target: &str) -> String {
        let classes = self
            .class_counts
            .iter()
            .map(|(class, n)| format!("{class}={n}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{target}: {} cases, {} interesting, {} violations [{}]",
            self.cases_run,
            self.interesting.len(),
            self.violations.len(),
            classes
        )
    }
}

/// The outcome-class label for accepted inputs.
pub const ACCEPTED_CLASS: &str = "accepted";

/// Run `cases` of one target's campaign, classifying every outcome and
/// collecting invariant violations. Fully deterministic in `(spec.seed,
/// cases)`: the first `ci_cases` of a full campaign are exactly the CI run.
///
/// Violation inputs are dumped under `CARGO_TARGET_TMPDIR/fuzz_violations/`
/// (when that env var is set) so a failing campaign leaves a repro on disk
/// without touching the committed corpus.
#[must_use]
pub fn run_campaign(target: &dyn Target, spec: &CampaignSpec, cases: u32) -> CampaignReport {
    let budgets = target.budgets();
    let seeds = target.seeds();
    let mut report = CampaignReport::default();
    let mut seen_classes = std::collections::BTreeSet::new();

    for case in 0..u64::from(cases) {
        let mut rng = XorShift64::for_case(spec.seed, case);

        // Base selection: a seed corpus entry, empty input, or raw random
        // bytes — then structure-aware mutations on top.
        let mut input = match rng.below(seeds.len() as u64 + 2) {
            i if (i as usize) < seeds.len() => seeds[i as usize].clone(),
            i if (i as usize) == seeds.len() => Vec::new(),
            _ => mutate::raw_random(&mut rng, budgets.max_input_bytes.min(8192)),
        };
        let rounds = 1 + rng.below(4);
        for _ in 0..rounds {
            target.mutate(&mut rng, &mut input);
        }
        if input.len() as u64 > budgets.max_input_bytes {
            input.truncate(budgets.max_input_bytes as usize);
        }

        let verdict = catch_unwind(AssertUnwindSafe(|| target.run(&input)));
        let (class, interesting_bytes) = match verdict {
            Err(_payload) => {
                report.violations.push(Violation::Panic { case });
                dump_violation_input(target.name(), case, &input);
                continue;
            }
            Ok(Verdict::Fault { message }) => {
                report
                    .violations
                    .push(Violation::TargetFault { case, message });
                dump_violation_input(target.name(), case, &input);
                continue;
            }
            Ok(Verdict::Accepted { output_bytes }) => {
                if let Some(max) = budgets.max_output_bytes
                    && output_bytes > max
                {
                    report.violations.push(Violation::OutputOverBudget {
                        case,
                        got: output_bytes,
                        max,
                    });
                    dump_violation_input(target.name(), case, &input);
                    continue;
                }
                (ACCEPTED_CLASS.to_owned(), input)
            }
            Ok(Verdict::Refused { class, message }) => {
                if !target.refusal_is_precise(&message) {
                    report
                        .violations
                        .push(Violation::ImpreciseError { case, message });
                    dump_violation_input(target.name(), case, &input);
                    continue;
                }
                (class, input)
            }
        };

        report.cases_run += 1;
        *report.class_counts.entry(class.clone()).or_insert(0) += 1;
        if seen_classes.insert(class.clone()) {
            report.interesting.push(Interesting {
                case,
                class,
                bytes: interesting_bytes,
            });
        }
    }
    report
}

/// Write a violation's input next to the build dir for post-mortem
/// debugging; a no-op outside `cargo test` (no `CARGO_TARGET_TMPDIR`).
fn dump_violation_input(target: &str, case: u64, input: &[u8]) {
    let Ok(tmp) = std::env::var("CARGO_TARGET_TMPDIR") else {
        return;
    };
    let dir = PathBuf::from(tmp).join("fuzz_violations").join(target);
    if create_dir_all(&dir).is_ok() {
        let name = format!("case{case:06}__{}.bin", &sha256(input).to_hex()[..12]);
        let _ = std::fs::write(dir.join(name), input);
    }
}

// ---------------------------------------------------------------- mutators

/// Structure-agnostic and format-aware mutation operators. Every operator
/// is deterministic in the supplied stream and respects explicit caps.
pub mod mutate {
    use super::XorShift64;

    /// Flip one random bit.
    pub fn flip_bit(rng: &mut XorShift64, buf: &mut [u8]) {
        if buf.is_empty() {
            return;
        }
        let i = rng.below(buf.len() as u64) as usize;
        let bit = 1_u8 << rng.below(8);
        buf[i] ^= bit;
    }

    /// Overwrite one random byte with a random value.
    pub fn overwrite_byte(rng: &mut XorShift64, buf: &mut [u8]) {
        if buf.is_empty() {
            return;
        }
        let i = rng.below(buf.len() as u64) as usize;
        buf[i] = rng.byte();
    }

    /// Copy a random slice over a random position (a "splice").
    pub fn splice_chunk(rng: &mut XorShift64, buf: &mut [u8]) {
        if buf.len() < 2 {
            return;
        }
        let len = 1 + rng.below((buf.len() as u64 / 2).max(1));
        let len = len.min(buf.len() as u64) as usize;
        let src = rng.below((buf.len() - len) as u64 + 1) as usize;
        let dst = rng.below((buf.len() - len) as u64 + 1) as usize;
        buf.copy_within(src..src + len, dst);
    }

    /// Duplicate a random slice at the end, never past `cap` total bytes.
    pub fn duplicate_chunk(rng: &mut XorShift64, buf: &mut Vec<u8>, cap: u64) {
        if buf.is_empty() || buf.len() as u64 >= cap {
            return;
        }
        let len = 1 + rng.below(buf.len() as u64);
        let len = len.min(cap - buf.len() as u64) as usize;
        let src = rng.below((buf.len() - len.min(buf.len())) as u64 + 1) as usize;
        let len = len.min(buf.len() - src);
        let chunk: Vec<u8> = buf[src..src + len].to_vec();
        buf.extend_from_slice(&chunk);
    }

    /// Drop a random suffix.
    pub fn truncate(rng: &mut XorShift64, buf: &mut Vec<u8>) {
        if buf.is_empty() {
            return;
        }
        let keep = rng.below(buf.len() as u64) as usize;
        buf.truncate(keep);
    }

    /// Overwrite a random aligned 32-bit big-endian field — the
    /// length/offset corruptions that matter for binary container formats.
    pub fn overwrite_u32be(rng: &mut XorShift64, buf: &mut [u8]) {
        if buf.len() < 4 {
            return;
        }
        let i = rng.below((buf.len() - 4) as u64 + 1) as usize;
        let value = if rng.chance(1, 4) {
            // Interesting extremes: huge lengths, zero, off-by-one.
            match rng.below(4) {
                0 => u32::MAX,
                1 => 0,
                2 => 0x7FFF_FFFF,
                _ => 1,
            }
        } else {
            rng.next_u64() as u32
        };
        buf[i..i + 4].copy_from_slice(&value.to_be_bytes());
    }

    /// Corrupt an sfnt table directory: the font's own `numTables` field or
    /// one 16-byte record (tag, checksum, offset, length). Structure-aware:
    /// these bytes steer every table lookup, so corrupting them reaches
    /// deep parser states that flat byte noise does not.
    pub fn corrupt_sfnt_directory(rng: &mut XorShift64, buf: &mut [u8]) {
        if buf.len() < 12 {
            return;
        }
        let num_tables = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        if rng.chance(1, 6) {
            // Rewrite numTables itself (bounded to keep records readable).
            let n = rng.below(64) as u16;
            buf[4..6].copy_from_slice(&n.to_be_bytes());
            return;
        }
        if num_tables == 0 || buf.len() < 12 + 16 {
            return;
        }
        let max_records = (buf.len() - 12) / 16;
        let record = rng.below(num_tables.min(max_records) as u64) as usize;
        let base = 12 + record * 16;
        // Fields: tag (0..4), checksum (4..8), offset (8..12), length (12..16).
        match rng.below(4) {
            0 => {
                let tag_byte = rng.below(4) as usize;
                buf[base + tag_byte] = rng.byte();
            }
            1 => overwrite_u32be(rng, &mut buf[base + 4..base + 8]),
            2 => overwrite_u32be(rng, &mut buf[base + 8..base + 12]),
            _ => overwrite_u32be(rng, &mut buf[base + 12..base + 16]),
        }
    }

    /// Fully random bytes, up to `cap` long (the raw-noise baseline every
    /// structured mutator is compared against).
    #[must_use]
    pub fn raw_random(rng: &mut XorShift64, cap: u64) -> Vec<u8> {
        let len = rng.below(cap.max(1)) as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(rng.byte());
        }
        out
    }

    /// Append a short run of tokens drawn from `pool` (grammar-biased soup,
    /// fmd-math's chaos-suite construction generalized).
    pub fn token_soup(rng: &mut XorShift64, pool: &[&str], max_tokens: u64) -> String {
        let n = 1 + rng.below(max_tokens.max(2));
        let mut out = String::new();
        for _ in 0..n {
            let i = rng.below(pool.len() as u64) as usize;
            out.push_str(pool[i]);
        }
        out
    }

    /// Splice one token from `pool` into `text` at a char boundary, or
    /// delete a short char-boundary-aligned span. Keeps the string UTF-8.
    pub fn token_splice(rng: &mut XorShift64, text: &mut String, pool: &[&str]) {
        if pool.is_empty() {
            return;
        }
        let boundary = |s: &str, at: u64| -> usize {
            let mut i = (at % (s.len() as u64 + 1)) as usize;
            while i > 0 && !s.is_char_boundary(i) {
                i -= 1;
            }
            i
        };
        match rng.below(3) {
            0..=1 => {
                let at = boundary(text, rng.next_u64());
                let tok = pool[rng.below(pool.len() as u64) as usize];
                text.insert_str(at, tok);
            }
            _ => {
                if !text.is_empty() {
                    let at = boundary(text, rng.next_u64());
                    let mut end = at;
                    for _ in 0..1 + rng.below(8) {
                        if end >= text.len() {
                            break;
                        }
                        end += 1;
                        while end < text.len() && !text.is_char_boundary(end) {
                            end += 1;
                        }
                    }
                    text.replace_range(at..end, "");
                }
            }
        }
    }
}

// ---------------------------------------------------------------- corpus

/// The corpus file name for one interesting input: case index, sanitized
/// class, and a content address (SHA-256 prefix) — deterministic, and a
/// traversal-safe path component by construction.
#[must_use]
pub fn corpus_file_name(case: u64, class: &str, bytes: &[u8]) -> String {
    let mut clean = String::with_capacity(class.len());
    for ch in class.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
            clean.push(ch);
        } else if ch.is_ascii_alphanumeric() {
            clean.push(ch.to_ascii_lowercase());
        } else {
            clean.push('-');
        }
    }
    let clean = if clean.is_empty() { "class" } else { &clean };
    format!(
        "case{case:06}__{clean}__{}.bin",
        &sha256(bytes).to_hex()[..12]
    )
}

/// The interesting inputs a campaign run expects on disk for one target:
/// file name → exact bytes.
#[must_use]
pub fn expected_corpus(report: &CampaignReport) -> BTreeMap<String, Vec<u8>> {
    report
        .interesting
        .iter()
        .map(|i| {
            (
                corpus_file_name(i.case, &i.class, &i.bytes),
                i.bytes.clone(),
            )
        })
        .collect()
}

/// One way the committed corpus disagrees with a regenerated campaign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorpusDrift {
    /// Expected by the campaign, absent on disk.
    Missing(String),
    /// Present but with different bytes (should be impossible given the
    /// content-addressed name; checked anyway).
    Changed(String),
    /// On disk but no longer produced by the campaign (needs a human to
    /// remove it — the rig never deletes).
    Stale(String),
}

const CORPUS_COMPARE_CHUNK_BYTES: usize = 8 * 1024;

fn corpus_reader_matches(mut reader: impl Read, expected: &[u8]) -> Result<bool, io::Error> {
    let mut scratch = [0_u8; CORPUS_COMPARE_CHUNK_BYTES];
    for expected_chunk in expected.chunks(scratch.len()) {
        match reader.read_exact(&mut scratch[..expected_chunk.len()]) {
            Ok(()) if scratch[..expected_chunk.len()] == *expected_chunk => {}
            Ok(()) => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
            Err(error) => return Err(error),
        }
    }

    let mut trailing = [0_u8; 1];
    match reader.read_exact(&mut trailing) {
        Ok(()) => Ok(false),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(true),
        Err(error) => Err(error),
    }
}

fn corpus_file_matches(path: &Path, expected: &[u8]) -> Result<Option<bool>, io::Error> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    corpus_reader_matches(file, expected).map(Some)
}

impl fmt::Display for CorpusDrift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "missing corpus file {name}"),
            Self::Changed(name) => write!(f, "corpus file {name} has drifted bytes"),
            Self::Stale(name) => {
                write!(f, "stale corpus file {name} (remove it and re-bless)")
            }
        }
    }
}

/// Compare the committed corpus directory for one target against the
/// campaign's expectation. An absent directory reports every expected file
/// as missing. When `exact` is false (the reduced CI run), stale files are
/// not reported — the CI prefix cannot know what the full campaign adds.
pub fn check_corpus(
    target_dir: &Path,
    expected: &BTreeMap<String, Vec<u8>>,
    exact: bool,
) -> Result<Vec<CorpusDrift>, io::Error> {
    let mut drift = Vec::new();
    for (name, bytes) in expected {
        let path = target_dir.join(name);
        match corpus_file_matches(&path, bytes)? {
            Some(true) => {}
            Some(false) => drift.push(CorpusDrift::Changed(name.clone())),
            None => drift.push(CorpusDrift::Missing(name.clone())),
        }
    }
    if exact && target_dir.is_dir() {
        for entry in std::fs::read_dir(target_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type()?.is_file() && !expected.contains_key(&name) {
                drift.push(CorpusDrift::Stale(name));
            }
        }
    }
    Ok(drift)
}

/// Write every missing or changed corpus file. Stale files are reported,
/// never removed — a human reviews and deletes, mirroring the golden rig's
/// "never auto-committing" posture. Returns the stale names.
pub fn bless_corpus(
    target_dir: &Path,
    expected: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<String>, io::Error> {
    create_dir_all(target_dir)?;
    for (name, bytes) in expected {
        let path = target_dir.join(name);
        if !corpus_file_matches(&path, bytes)?.unwrap_or(false) {
            std::fs::write(&path, bytes)?;
        }
    }
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(target_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_file() && !expected.contains_key(&name) {
            stale.push(name);
        }
    }
    Ok(stale)
}

fn create_dir_all(path: &Path) -> Result<(), io::Error> {
    std::fs::create_dir_all(path)
}

// ---------------------------------------------------------------- manifest

/// The manifest's format version tag; the first line of `MANIFEST.tsv`.
pub const MANIFEST_HEADER: &str = "# fmn-fuzz-manifest v1";

const MANIFEST_COLUMNS: &str = "# columns: target\tseed\tci_cases\tfull_cases\tmax_input_bytes\tmax_output_bytes\toutcome_classes";
const MAX_MANIFEST_BYTES: usize = 1 << 20;
const MAX_MANIFEST_LINE_BYTES: usize = 16 << 10;
const MAX_MANIFEST_FIELD_BYTES: usize = 4096;
const MAX_MANIFEST_LABEL_BYTES: usize = 128;
const MAX_MANIFEST_ROWS: usize = 4096;
const MAX_MANIFEST_PENDING: usize = 4096;
const MAX_MANIFEST_CLASSES: usize = 256;

/// One manifest row: the versioned record of one target's campaign.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestRow {
    /// Target name (matches [`Target::name`]).
    pub target: String,
    /// Campaign seed.
    pub seed: u64,
    /// Reduced CI case count.
    pub ci_cases: u32,
    /// Full-campaign case count.
    pub full_cases: u32,
    /// Input byte cap.
    pub max_input_bytes: u64,
    /// Output byte cap, or `None` (bounded-by-construction).
    pub max_output_bytes: Option<u64>,
    /// Outcome classes of the full campaign, sorted.
    pub classes: Vec<String>,
}

/// A parsed manifest: the rows plus the pending-target notes (targets that
/// are planned but whose subsystem has not landed — recorded, never
/// stubbed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    /// Committed rows.
    pub rows: Vec<ManifestRow>,
    /// Pending target names and notes from canonical
    /// `# pending: <name> — <note>` records.
    pub pending: Vec<(String, String)>,
}

const PENDING_PREFIX: &str = "# pending: ";
const PENDING_SEPARATOR: &str = " — ";

fn decimal_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

fn checked_manifest_sum(parts: &[usize]) -> Result<usize, String> {
    parts.iter().try_fold(0_usize, |sum, part| {
        sum.checked_add(*part)
            .ok_or_else(|| "rendered manifest size overflow".to_owned())
    })
}

fn add_rendered_line(
    rendered_bytes: &mut usize,
    line_bytes: usize,
    record_kind: &str,
    record_index: usize,
) -> Result<(), String> {
    if line_bytes > MAX_MANIFEST_LINE_BYTES {
        return Err(format!(
            "{record_kind} {record_index} exceeds the {MAX_MANIFEST_LINE_BYTES}-byte line limit"
        ));
    }
    *rendered_bytes = rendered_bytes
        .checked_add(line_bytes)
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or_else(|| "rendered manifest size overflow".to_owned())?;
    if *rendered_bytes > MAX_MANIFEST_BYTES {
        return Err(format!(
            "rendered manifest exceeds the {MAX_MANIFEST_BYTES}-byte limit"
        ));
    }
    Ok(())
}

fn validate_render_label(
    value: &str,
    record_kind: &str,
    record_index: usize,
    field: &str,
    allow_underscore: bool,
) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!(
            "{record_kind} {record_index}: {field} must not be empty"
        ));
    }
    if value.len() > MAX_MANIFEST_LABEL_BYTES {
        return Err(format!(
            "{record_kind} {record_index}: {field} exceeds the {MAX_MANIFEST_LABEL_BYTES}-byte label limit"
        ));
    }
    let valid = value.bytes().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || byte == b'-'
            || (allow_underscore && byte == b'_')
    });
    if !valid {
        let alphabet = if allow_underscore {
            "[a-z0-9_-]"
        } else {
            "[a-z0-9-]"
        };
        return Err(format!(
            "{record_kind} {record_index}: {field} must use only lowercase {alphabet} characters"
        ));
    }
    Ok(())
}

/// Serialize a manifest deterministically (pending records and rows sorted
/// by target).
///
/// All parser grammar and resource limits are checked before the output
/// buffer is allocated, so callers can refuse invalid authority data before
/// attempting a filesystem write.
pub fn render_manifest(
    rows: &[ManifestRow],
    pending: &[(String, String)],
) -> Result<String, String> {
    if rows.is_empty() {
        return Err("manifest records no targets".to_owned());
    }
    if rows.len() > MAX_MANIFEST_ROWS {
        return Err(format!(
            "rendered manifest exceeds the {MAX_MANIFEST_ROWS}-row limit"
        ));
    }
    if pending.len() > MAX_MANIFEST_PENDING {
        return Err(format!(
            "rendered manifest exceeds the {MAX_MANIFEST_PENDING}-pending-record limit"
        ));
    }

    let mut rendered_bytes =
        checked_manifest_sum(&[MANIFEST_HEADER.len(), 1, MANIFEST_COLUMNS.len(), 1])?;

    for (index, (name, note)) in pending.iter().enumerate() {
        let record_index = index + 1;
        validate_render_label(name, "pending record", record_index, "target", true)?;
        if note.len() > MAX_MANIFEST_FIELD_BYTES {
            return Err(format!(
                "pending record {record_index}: note exceeds the {MAX_MANIFEST_FIELD_BYTES}-byte field limit"
            ));
        }
        if note.is_empty()
            || note.trim() != note
            || note.contains('\t')
            || note.chars().any(char::is_control)
        {
            return Err(format!(
                "pending record {record_index}: note must be nonempty, unpadded, and free of control characters"
            ));
        }
        let line_bytes = checked_manifest_sum(&[
            PENDING_PREFIX.len(),
            name.len(),
            PENDING_SEPARATOR.len(),
            note.len(),
        ])?;
        add_rendered_line(
            &mut rendered_bytes,
            line_bytes,
            "pending record",
            record_index,
        )?;
    }

    for (index, row) in rows.iter().enumerate() {
        let record_index = index + 1;
        validate_render_label(&row.target, "target row", record_index, "target", true)?;
        if row.ci_cases == 0 {
            return Err(format!(
                "target row {record_index}: ci_cases must be positive"
            ));
        }
        if row.full_cases < row.ci_cases {
            return Err(format!(
                "target row {record_index}: full_cases must be at least ci_cases"
            ));
        }
        if row.max_input_bytes == 0 {
            return Err(format!(
                "target row {record_index}: max_input_bytes must be positive"
            ));
        }
        if row.classes.is_empty() {
            return Err(format!(
                "target row {record_index}: outcome classes must be nonempty"
            ));
        }
        if row.classes.len() > MAX_MANIFEST_CLASSES {
            return Err(format!(
                "target row {record_index}: outcome_classes exceeds the {MAX_MANIFEST_CLASSES}-class limit"
            ));
        }

        let mut classes_bytes = row.classes.len() - 1;
        let mut previous_class: Option<&str> = None;
        for class in &row.classes {
            validate_render_label(class, "target row", record_index, "outcome class", false)?;
            classes_bytes = classes_bytes
                .checked_add(class.len())
                .ok_or_else(|| "rendered manifest size overflow".to_owned())?;
            if let Some(previous) = previous_class {
                if previous == class {
                    return Err(format!(
                        "target row {record_index}: outcome classes must not contain duplicates"
                    ));
                }
                if previous > class.as_str() {
                    return Err(format!(
                        "target row {record_index}: outcome classes must be strictly sorted"
                    ));
                }
            }
            previous_class = Some(class);
        }
        if classes_bytes > MAX_MANIFEST_FIELD_BYTES {
            return Err(format!(
                "target row {record_index}: outcome_classes exceeds the {MAX_MANIFEST_FIELD_BYTES}-byte field limit"
            ));
        }

        let max_output_bytes = row.max_output_bytes.map_or(1, decimal_len);
        let line_bytes = checked_manifest_sum(&[
            row.target.len(),
            6,
            decimal_len(row.seed),
            decimal_len(u64::from(row.ci_cases)),
            decimal_len(u64::from(row.full_cases)),
            decimal_len(row.max_input_bytes),
            max_output_bytes,
            classes_bytes,
        ])?;
        add_rendered_line(&mut rendered_bytes, line_bytes, "target row", record_index)?;
    }

    let mut sorted_pending: Vec<_> = pending.iter().collect();
    sorted_pending.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    for pair in sorted_pending.windows(2) {
        if pair[0].0 == pair[1].0 {
            return Err("rendered manifest contains a duplicate pending target".to_owned());
        }
    }

    let mut sorted_rows: Vec<_> = rows.iter().collect();
    sorted_rows.sort_unstable_by(|left, right| left.target.cmp(&right.target));
    for pair in sorted_rows.windows(2) {
        if pair[0].target == pair[1].target {
            return Err("rendered manifest contains a duplicate target row".to_owned());
        }
    }
    for (name, _) in &sorted_pending {
        if sorted_rows
            .binary_search_by(|row| row.target.as_str().cmp(name))
            .is_ok()
        {
            return Err("rendered manifest target cannot be both pending and committed".to_owned());
        }
    }

    let mut out = String::with_capacity(rendered_bytes);
    out.push_str(MANIFEST_HEADER);
    out.push('\n');
    out.push_str(MANIFEST_COLUMNS);
    out.push('\n');
    for (name, note) in sorted_pending {
        out.push_str(PENDING_PREFIX);
        out.push_str(name);
        out.push_str(PENDING_SEPARATOR);
        out.push_str(note);
        out.push('\n');
    }
    for row in sorted_rows {
        write!(
            &mut out,
            "{}\t{}\t{}\t{}\t{}\t",
            row.target, row.seed, row.ci_cases, row.full_cases, row.max_input_bytes
        )
        .map_err(|_| "internal manifest formatting failure".to_owned())?;
        if let Some(max_output_bytes) = row.max_output_bytes {
            write!(&mut out, "{max_output_bytes}")
                .map_err(|_| "internal manifest formatting failure".to_owned())?;
        } else {
            out.push('-');
        }
        out.push('\t');
        for (index, class) in row.classes.iter().enumerate() {
            if index != 0 {
                out.push(',');
            }
            out.push_str(class);
        }
        out.push('\n');
    }
    debug_assert_eq!(out.len(), rendered_bytes);
    Ok(out)
}

fn split_manifest_row(line: &str) -> Option<[&str; 7]> {
    let mut fields = line.split('\t');
    let exact = [
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
    ];
    fields.next().is_none().then_some(exact)
}

fn validate_manifest_line(line: &str, lineno: usize) -> Result<(), String> {
    if line.len() > MAX_MANIFEST_LINE_BYTES {
        return Err(format!(
            "line {lineno}: exceeds the {MAX_MANIFEST_LINE_BYTES}-byte limit"
        ));
    }
    if line
        .as_bytes()
        .last()
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        return Err(format!(
            "line {lineno}: trailing horizontal whitespace is not canonical"
        ));
    }
    if line.chars().any(|ch| ch.is_control() && ch != '\t') {
        return Err(format!(
            "line {lineno}: control characters are not permitted"
        ));
    }
    Ok(())
}

fn validate_manifest_field(value: &str, lineno: usize, field: &str) -> Result<(), String> {
    if value.len() > MAX_MANIFEST_FIELD_BYTES {
        return Err(format!(
            "line {lineno}: {field} exceeds the {MAX_MANIFEST_FIELD_BYTES}-byte field limit"
        ));
    }
    Ok(())
}

fn validate_manifest_label(
    value: &str,
    lineno: usize,
    field: &str,
    allow_underscore: bool,
) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("line {lineno}: {field} must not be empty"));
    }
    if value.len() > MAX_MANIFEST_LABEL_BYTES {
        return Err(format!(
            "line {lineno}: {field} exceeds the {MAX_MANIFEST_LABEL_BYTES}-byte label limit"
        ));
    }
    let valid = value.bytes().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || byte == b'-'
            || (allow_underscore && byte == b'_')
    });
    if !valid {
        let alphabet = if allow_underscore {
            "[a-z0-9_-]"
        } else {
            "[a-z0-9-]"
        };
        return Err(format!(
            "line {lineno}: {field} must use only lowercase {alphabet} characters"
        ));
    }
    Ok(())
}

fn manifest_u64(value: &str, lineno: usize, field: &str) -> Result<u64, String> {
    validate_manifest_field(value, lineno, field)?;
    let canonical = !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'));
    if !canonical {
        return Err(format!(
            "line {lineno}: {field} must be a canonical unsigned decimal integer"
        ));
    }
    value
        .parse::<u64>()
        .map_err(|_| format!("line {lineno}: {field} is out of range for u64"))
}

fn manifest_classes(value: &str, lineno: usize) -> Result<Vec<String>, String> {
    validate_manifest_field(value, lineno, "outcome_classes")?;
    let mut classes: Vec<String> = Vec::new();
    for class in value.split(',') {
        if class.is_empty() || class.trim() != class {
            return Err(format!(
                "line {lineno}: outcome classes must be nonempty and carry no surrounding whitespace"
            ));
        }
        validate_manifest_label(class, lineno, "outcome class", false)?;
        if classes.len() == MAX_MANIFEST_CLASSES {
            return Err(format!(
                "line {lineno}: outcome_classes exceeds the {MAX_MANIFEST_CLASSES}-class limit"
            ));
        }
        if let Some(previous) = classes.last() {
            if previous.as_str() == class {
                return Err(format!(
                    "line {lineno}: outcome classes must not contain duplicates"
                ));
            }
            if previous.as_str() > class {
                return Err(format!(
                    "line {lineno}: outcome classes must be strictly sorted"
                ));
            }
        }
        classes.push(class.to_owned());
    }
    Ok(classes)
}

/// Parse a `MANIFEST.tsv`. Errors are precise (line-numbered strings) —
/// the manifest is itself a small untrusted input.
pub fn parse_manifest(text: &str) -> Result<Manifest, String> {
    if text.len() > MAX_MANIFEST_BYTES {
        return Err(format!(
            "manifest exceeds the {MAX_MANIFEST_BYTES}-byte limit"
        ));
    }
    if !text.ends_with('\n') {
        return Err("manifest must end with a final LF newline".to_owned());
    }
    if text.contains('\r') {
        return Err("manifest must use LF line endings".to_owned());
    }

    let mut lines = text[..text.len() - 1].split('\n');
    let header = lines.next().unwrap_or("");
    validate_manifest_line(header, 1)?;
    if header != MANIFEST_HEADER {
        return Err(format!("line 1: expected {MANIFEST_HEADER:?}"));
    }
    let columns = lines.next().unwrap_or("");
    validate_manifest_line(columns, 2)?;
    if columns != MANIFEST_COLUMNS {
        return Err(format!("line 2: expected {MANIFEST_COLUMNS:?}"));
    }

    let mut rows = Vec::new();
    let mut pending = Vec::new();
    let mut seen_targets = std::collections::BTreeSet::new();
    let mut seen_pending = std::collections::BTreeSet::new();
    let mut previous_target = None;
    let mut previous_pending = None;
    let mut rows_started = false;
    for (ix, line) in lines.enumerate() {
        let lineno = ix + 3;
        validate_manifest_line(line, lineno)?;
        if line.is_empty() {
            return Err(format!("line {lineno}: blank lines are not canonical"));
        }
        if let Some(rest) = line.strip_prefix("# pending: ") {
            if rows_started {
                return Err(format!(
                    "line {lineno}: pending records must precede target rows"
                ));
            }
            if pending.len() == MAX_MANIFEST_PENDING {
                return Err(format!(
                    "line {lineno}: manifest exceeds the {MAX_MANIFEST_PENDING}-pending-record limit"
                ));
            }
            let Some((name, note)) = rest.split_once(" — ") else {
                return Err(format!(
                    "line {lineno}: pending record must be '# pending: <target> — <note>'"
                ));
            };
            validate_manifest_label(name, lineno, "pending target", true)?;
            validate_manifest_field(note, lineno, "pending note")?;
            if note.is_empty() || note.trim() != note || note.contains('\t') {
                return Err(format!(
                    "line {lineno}: pending note must be nonempty and unpadded"
                ));
            }
            if !seen_pending.insert(name) {
                return Err(format!("line {lineno}: duplicate pending target"));
            }
            if previous_pending.is_some_and(|previous| previous >= name) {
                return Err(format!(
                    "line {lineno}: pending targets must be strictly sorted"
                ));
            }
            previous_pending = Some(name);
            pending.push((name.to_owned(), note.to_owned()));
            continue;
        }
        if line.starts_with('#') {
            return Err(format!("line {lineno}: unknown manifest comment"));
        }
        rows_started = true;
        if rows.len() == MAX_MANIFEST_ROWS {
            return Err(format!(
                "line {lineno}: manifest exceeds the {MAX_MANIFEST_ROWS}-row limit"
            ));
        }
        let Some(
            [
                target,
                seed,
                ci_cases,
                full_cases,
                max_input_bytes,
                max_output_bytes,
                outcome_classes,
            ],
        ) = split_manifest_row(line)
        else {
            return Err(format!(
                "line {lineno}: expected exactly 7 tab-separated columns"
            ));
        };
        for (field, value) in [
            ("target", target),
            ("seed", seed),
            ("ci_cases", ci_cases),
            ("full_cases", full_cases),
            ("max_input_bytes", max_input_bytes),
            ("max_output_bytes", max_output_bytes),
            ("outcome_classes", outcome_classes),
        ] {
            validate_manifest_field(value, lineno, field)?;
        }
        validate_manifest_label(target, lineno, "target", true)?;
        if !seen_targets.insert(target) {
            return Err(format!("line {lineno}: duplicate target row"));
        }
        if previous_target.is_some_and(|previous| previous >= target) {
            return Err(format!(
                "line {lineno}: target rows must be strictly sorted"
            ));
        }
        if seen_pending.contains(target) {
            return Err(format!(
                "line {lineno}: target cannot be both pending and committed"
            ));
        }
        previous_target = Some(target);
        let max_output_bytes = if max_output_bytes == "-" {
            None
        } else {
            Some(manifest_u64(max_output_bytes, lineno, "max_output_bytes")?)
        };
        let classes = manifest_classes(outcome_classes, lineno)?;
        let ci_cases = u32::try_from(manifest_u64(ci_cases, lineno, "ci_cases")?)
            .map_err(|_| format!("line {lineno}: ci_cases out of range"))?;
        let full_cases = u32::try_from(manifest_u64(full_cases, lineno, "full_cases")?)
            .map_err(|_| format!("line {lineno}: full_cases out of range"))?;
        let max_input_bytes = manifest_u64(max_input_bytes, lineno, "max_input_bytes")?;
        if ci_cases == 0 {
            return Err(format!("line {lineno}: ci_cases must be positive"));
        }
        if full_cases < ci_cases {
            return Err(format!(
                "line {lineno}: full_cases must be at least ci_cases"
            ));
        }
        if max_input_bytes == 0 {
            return Err(format!("line {lineno}: max_input_bytes must be positive"));
        }
        rows.push(ManifestRow {
            target: target.to_owned(),
            seed: manifest_u64(seed, lineno, "seed")?,
            ci_cases,
            full_cases,
            max_input_bytes,
            max_output_bytes,
            classes,
        });
    }
    if rows.is_empty() {
        return Err("manifest records no targets".to_owned());
    }
    Ok(Manifest { rows, pending })
}

#[cfg(test)]
mod tests {
    use super::mutate::{
        corrupt_sfnt_directory, duplicate_chunk, flip_bit, overwrite_byte, overwrite_u32be,
        raw_random, splice_chunk, truncate,
    };
    use super::*;

    #[test]
    fn xorshift_is_deterministic_and_case_independent() {
        let mut a = XorShift64::for_case(42, 7);
        let mut b = XorShift64::for_case(42, 7);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = XorShift64::for_case(42, 8);
        assert_ne!(a.next_u64(), c.next_u64());
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = XorShift64::for_case(1, 0);
        for n in 1..257_u64 {
            for _ in 0..32 {
                assert!(rng.below(n) < n);
            }
        }
    }

    #[test]
    fn mutators_respect_bounds() {
        let mut rng = XorShift64::for_case(7, 0);
        for _ in 0..500 {
            let mut buf = raw_random(&mut rng, 512);
            let cap = 1024_u64;
            match rng.below(7) {
                0 => flip_bit(&mut rng, &mut buf),
                1 => overwrite_byte(&mut rng, &mut buf),
                2 => splice_chunk(&mut rng, &mut buf),
                3 => duplicate_chunk(&mut rng, &mut buf, cap),
                4 => truncate(&mut rng, &mut buf),
                5 => overwrite_u32be(&mut rng, &mut buf),
                _ => corrupt_sfnt_directory(&mut rng, &mut buf),
            }
            assert!(buf.len() as u64 <= cap);
        }
    }

    #[test]
    fn token_ops_keep_utf8_and_bounds() {
        use mutate::{token_soup, token_splice};
        let pool = &["{", "}", "\\frac", "a", " ", "\n", "α"];
        let mut rng = XorShift64::for_case(9, 0);
        for _ in 0..200 {
            let mut s = token_soup(&mut rng, pool, 16);
            assert!(s.len() <= 16 * 8 + 8);
            for _ in 0..4 {
                token_splice(&mut rng, &mut s, pool);
            }
            // Any panic on replace_range/insert_str would fail the test;
            // reaching here means char boundaries were respected.
        }
    }

    #[test]
    fn manifest_round_trips() {
        let rows = vec![
            ManifestRow {
                target: "tex_math".to_owned(),
                seed: 42,
                ci_cases: 200,
                full_cases: 5000,
                max_input_bytes: 8192,
                max_output_bytes: Some(1 << 25),
                classes: vec!["accepted".to_owned(), "malformed".to_owned()],
            },
            ManifestRow {
                target: "canon_deser".to_owned(),
                seed: 7,
                ci_cases: 600,
                full_cases: 20_000,
                max_input_bytes: 262_144,
                max_output_bytes: None,
                classes: vec!["accepted".to_owned(), "unframed".to_owned()],
            },
        ];
        let pending = vec![
            (
                "zeta_future".to_owned(),
                "lands after the codec tranche".to_owned(),
            ),
            (
                "alpha_future".to_owned(),
                "lands with the parser tranche".to_owned(),
            ),
        ];
        let text = render_manifest(&rows, &pending).expect("manifest renders");
        let parsed = parse_manifest(&text).expect("manifest parses");
        assert_eq!(parsed.rows, {
            let mut sorted = rows.clone();
            sorted.sort_by(|a, b| a.target.cmp(&b.target));
            sorted
        });
        assert_eq!(parsed.pending, {
            let mut sorted = pending.clone();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            sorted
        });
        // Rendering is idempotent — the checked-in file is byte-stable.
        assert_eq!(
            render_manifest(&parsed.rows, &parsed.pending).expect("parsed manifest renders"),
            text
        );
    }

    #[test]
    fn manifest_renderer_rejects_noncanonical_authority() {
        let row = |target: &str| ManifestRow {
            target: target.to_owned(),
            seed: 7,
            ci_cases: 1,
            full_cases: 2,
            max_input_bytes: 3,
            max_output_bytes: None,
            classes: vec!["accepted".to_owned(), "malformed".to_owned()],
        };

        let invalid_target = row("Tex_math");
        assert!(
            render_manifest(&[invalid_target], &[])
                .expect_err("invalid target label must be refused")
                .contains("[a-z0-9_-]")
        );

        let mut invalid_classes = row("tex_math");
        invalid_classes.classes.swap(0, 1);
        assert!(
            render_manifest(&[invalid_classes], &[])
                .expect_err("unsorted classes must be refused")
                .contains("strictly sorted")
        );

        let duplicate_rows = [row("tex_math"), row("tex_math")];
        assert!(
            render_manifest(&duplicate_rows, &[])
                .expect_err("duplicate target rows must be refused")
                .contains("duplicate target")
        );

        let duplicate_pending = [
            ("future_target".to_owned(), "first note".to_owned()),
            ("future_target".to_owned(), "second note".to_owned()),
        ];
        assert!(
            render_manifest(&[row("tex_math")], &duplicate_pending)
                .expect_err("duplicate pending targets must be refused")
                .contains("duplicate pending")
        );

        let conflicting_pending = [("tex_math".to_owned(), "not landed".to_owned())];
        assert!(
            render_manifest(&[row("tex_math")], &conflicting_pending)
                .expect_err("pending and committed target conflict must be refused")
                .contains("both pending and committed")
        );

        for note in ["", " padded", "padded ", "tab\tnote", "line\nnote"] {
            let pending = [("future_target".to_owned(), note.to_owned())];
            assert!(
                render_manifest(&[row("tex_math")], &pending).is_err(),
                "pending note {note:?} must be refused"
            );
        }
    }

    #[test]
    fn manifest_renderer_enforces_bounds_before_output_allocation() {
        let row = || ManifestRow {
            target: "tex_math".to_owned(),
            seed: 7,
            ci_cases: 1,
            full_cases: 2,
            max_input_bytes: 3,
            max_output_bytes: None,
            classes: vec!["accepted".to_owned()],
        };

        let excessive_rows = vec![row(); MAX_MANIFEST_ROWS + 1];
        assert!(
            render_manifest(&excessive_rows, &[])
                .expect_err("excessive row count must be refused")
                .contains("row limit")
        );

        let excessive_pending =
            vec![("future_target".to_owned(), "note".to_owned()); MAX_MANIFEST_PENDING + 1];
        assert!(
            render_manifest(&[row()], &excessive_pending)
                .expect_err("excessive pending count must be refused")
                .contains("pending-record limit")
        );

        let mut excessive_classes = row();
        excessive_classes.classes = vec!["accepted".to_owned(); MAX_MANIFEST_CLASSES + 1];
        assert!(
            render_manifest(&[excessive_classes], &[])
                .expect_err("excessive class count must be refused")
                .contains("class limit")
        );

        let oversized_note = [(
            "future_target".to_owned(),
            "n".repeat(MAX_MANIFEST_FIELD_BYTES + 1),
        )];
        assert!(
            render_manifest(&[row()], &oversized_note)
                .expect_err("oversized note must be refused")
                .contains("field limit")
        );

        let document_bomb: Vec<_> = (0..256)
            .map(|index| (format!("p{index:03}"), "n".repeat(MAX_MANIFEST_FIELD_BYTES)))
            .collect();
        assert!(
            render_manifest(&[row()], &document_bomb)
                .expect_err("oversized rendered document must be refused")
                .contains("byte limit")
        );
    }

    #[test]
    fn manifest_rejects_noncanonical_structure() {
        let valid_row =
            |target: &str, classes: &str| format!("{target}\t0\t1\t2\t3\t-\t{classes}\n");
        let document = |body: &str| format!("{MANIFEST_HEADER}\n{MANIFEST_COLUMNS}\n{body}");

        assert!(parse_manifest("").is_err());
        assert!(parse_manifest("# fmn-fuzz-manifest v1\n").is_err());
        assert!(
            parse_manifest(&format!("{MANIFEST_HEADER}\n# columns: wrong\n"))
                .expect_err("altered columns metadata must be refused")
                .contains("line 2")
        );

        let valid = document(&valid_row("tex_math", "accepted,malformed"));
        assert!(parse_manifest(&valid).is_ok());
        assert!(
            parse_manifest(valid.trim_end_matches('\n'))
                .expect_err("missing final LF must be refused")
                .contains("final LF")
        );
        assert!(
            parse_manifest(&valid.replace('\n', "\r\n"))
                .expect_err("CRLF must be refused")
                .contains("LF line endings")
        );
        assert!(
            parse_manifest(&document(&format!(
                "\n{}",
                valid_row("tex_math", "accepted")
            )))
            .expect_err("blank lines must be refused")
            .contains("blank lines")
        );
        assert!(
            parse_manifest(&document(&format!(
                "# arbitrary\n{}",
                valid_row("tex_math", "accepted")
            )))
            .expect_err("unknown comments must be refused")
            .contains("unknown manifest comment")
        );
        assert!(
            parse_manifest(&document(&format!(
                "{}{}",
                valid_row("zeta", "accepted"),
                valid_row("alpha", "accepted")
            )))
            .expect_err("unsorted rows must be refused")
            .contains("strictly sorted")
        );
    }

    #[test]
    fn manifest_rejects_ambiguous_identities_and_values() {
        let valid_row =
            |target: &str, classes: &str| format!("{target}\t0\t1\t2\t3\t-\t{classes}\n");
        let pending = |target: &str, note: &str| format!("# pending: {target} — {note}\n");
        let document = |body: &str| format!("{MANIFEST_HEADER}\n{MANIFEST_COLUMNS}\n{body}");

        let duplicate = document(&format!(
            "{}{}",
            valid_row("tex_math", "accepted,malformed"),
            valid_row("tex_math", "accepted,malformed")
        ));
        assert!(
            parse_manifest(&duplicate)
                .expect_err("duplicate target must be refused")
                .contains("duplicate target")
        );
        assert!(
            parse_manifest(&document(&valid_row("Tex_math", "accepted")))
                .expect_err("noncanonical target label must be refused")
                .contains("[a-z0-9_-]")
        );

        for (classes, expected) in [
            ("malformed,accepted", "strictly sorted"),
            ("accepted,accepted", "duplicates"),
            ("accepted,,malformed", "nonempty"),
            ("accepted, malformed", "whitespace"),
            ("accepted,bad_class", "[a-z0-9-]"),
        ] {
            let error = parse_manifest(&document(&valid_row("tex_math", classes)))
                .expect_err("noncanonical classes must be refused");
            assert!(
                error.contains(expected),
                "{classes:?}: expected {expected:?}, got {error:?}"
            );
        }

        let duplicate_pending = document(&format!(
            "{}{}{}",
            pending("future_target", "first note"),
            pending("future_target", "second note"),
            valid_row("tex_math", "accepted")
        ));
        assert!(
            parse_manifest(&duplicate_pending)
                .expect_err("duplicate pending identity must be refused")
                .contains("duplicate pending target")
        );

        let unsorted_pending = document(&format!(
            "{}{}{}",
            pending("zeta_future", "later"),
            pending("alpha_future", "earlier"),
            valid_row("tex_math", "accepted")
        ));
        assert!(
            parse_manifest(&unsorted_pending)
                .expect_err("unsorted pending identities must be refused")
                .contains("strictly sorted")
        );

        let conflicting = document(&format!(
            "{}{}",
            pending("tex_math", "not landed"),
            valid_row("tex_math", "accepted")
        ));
        assert!(
            parse_manifest(&conflicting)
                .expect_err("pending/committed identity conflict must be refused")
                .contains("both pending and committed")
        );

        let pending_after_row = document(&format!(
            "{}{}",
            valid_row("alpha", "accepted"),
            pending("zeta_future", "not landed")
        ));
        assert!(
            parse_manifest(&pending_after_row)
                .expect_err("pending records after rows must be refused")
                .contains("must precede")
        );
        assert!(
            parse_manifest(&document(&format!(
                "# pending: future_target — \tnote\n{}",
                valid_row("tex_math", "accepted")
            )))
            .expect_err("padded pending note must be refused")
            .contains("unpadded")
        );

        for (row, expected) in [
            ("tex_math\t00\t1\t2\t3\t-\taccepted\n", "canonical"),
            ("tex_math\t+1\t1\t2\t3\t-\taccepted\n", "canonical"),
            ("tex_math\t0\t0\t2\t3\t-\taccepted\n", "positive"),
            ("tex_math\t0\t3\t2\t3\t-\taccepted\n", "at least"),
            ("tex_math\t0\t1\t2\t0\t-\taccepted\n", "positive"),
        ] {
            let error = parse_manifest(&document(row))
                .expect_err("noncanonical campaign values must be refused");
            assert!(
                error.contains(expected),
                "expected {expected:?}, got {error:?}"
            );
        }
    }

    #[test]
    fn manifest_enforces_resource_bounds_and_bounded_diagnostics() {
        let valid_row =
            |target: &str, classes: &str| format!("{target}\t0\t1\t2\t3\t-\t{classes}\n");
        let document = |body: &str| format!("{MANIFEST_HEADER}\n{MANIFEST_COLUMNS}\n{body}");

        let oversized_document = format!("{}\n", "x".repeat(MAX_MANIFEST_BYTES));
        assert!(
            parse_manifest(&oversized_document)
                .expect_err("oversized document must be refused")
                .contains("byte limit")
        );

        let oversized_line = document(&format!(
            "#{}\n{}",
            "x".repeat(MAX_MANIFEST_LINE_BYTES),
            valid_row("tex_math", "accepted")
        ));
        assert!(
            parse_manifest(&oversized_line)
                .expect_err("oversized line must be refused")
                .contains("byte limit")
        );

        let oversized_field = document(&valid_row(
            "tex_math",
            &"a".repeat(MAX_MANIFEST_FIELD_BYTES + 1),
        ));
        assert!(
            parse_manifest(&oversized_field)
                .expect_err("oversized field must be refused")
                .contains("field limit")
        );

        let excessive_classes = (0..=MAX_MANIFEST_CLASSES)
            .map(|ix| format!("c{ix:03}"))
            .collect::<Vec<_>>()
            .join(",");
        assert!(
            parse_manifest(&document(&valid_row("tex_math", &excessive_classes)))
                .expect_err("excessive class count must be refused")
                .contains("class limit")
        );

        let mut excessive_rows = String::new();
        for ix in 0..=MAX_MANIFEST_ROWS {
            excessive_rows.push_str(&valid_row(&format!("t{ix:04}"), "accepted"));
        }
        assert!(
            parse_manifest(&document(&excessive_rows))
                .expect_err("excessive row count must be refused")
                .contains("row limit")
        );

        let mut excessive_pending = String::new();
        for ix in 0..=MAX_MANIFEST_PENDING {
            excessive_pending.push_str(&format!("# pending: p{ix:04} — note\n"));
        }
        excessive_pending.push_str(&valid_row("zeta", "accepted"));
        assert!(
            parse_manifest(&document(&excessive_pending))
                .expect_err("excessive pending count must be refused")
                .contains("pending-record limit")
        );

        let oversized_header = format!("{}\n", "x".repeat(1_000_000));
        let error = parse_manifest(&oversized_header).expect_err("wrong header must be refused");
        assert!(
            error.len() < 256,
            "header diagnostic is {} bytes",
            error.len()
        );

        let excessive_separators = document(&format!("{}\n", "\t".repeat(1_000_000)));
        let error =
            parse_manifest(&excessive_separators).expect_err("separator-heavy row must be refused");
        assert!(
            error.len() < 256,
            "field-count diagnostic is {} bytes",
            error.len()
        );

        let oversized_number = document(&format!(
            "tex_math\t{}\t1\t2\t3\t-\taccepted\n",
            "9".repeat(1_000_000)
        ));
        let error =
            parse_manifest(&oversized_number).expect_err("oversized number must be refused");
        assert!(
            error.len() < 256,
            "numeric diagnostic is {} bytes",
            error.len()
        );
    }

    #[test]
    fn corpus_names_are_safe_and_content_addressed() {
        let a = corpus_file_name(3, "missing-table-head", b"abc");
        let b = corpus_file_name(3, "missing-table-head", b"abd");
        assert_ne!(a, b);
        for name in [a, b] {
            assert!(name.ends_with(".bin"));
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            );
        }
    }

    #[test]
    fn corpus_check_and_bless_converge() {
        let dir = std::env::temp_dir().join(format!(
            "fmn_fuzz_corpus_test_{}_{}",
            std::process::id(),
            sha256(b"corpus_check_and_bless_converge")
        ));
        let mut expected = BTreeMap::new();
        expected.insert("case000000__accepted__aaa.bin".to_owned(), b"one".to_vec());
        expected.insert("case000001__refused__bbb.bin".to_owned(), b"two".to_vec());

        let drift = check_corpus(&dir, &expected, true).expect("check");
        assert_eq!(drift.len(), 2, "everything missing initially: {drift:?}");

        let stale = bless_corpus(&dir, &expected).expect("bless");
        assert!(stale.is_empty());
        assert!(
            check_corpus(&dir, &expected, true)
                .expect("check")
                .is_empty()
        );

        // A file the campaign no longer produces is stale, reported, kept.
        std::fs::write(dir.join("case000009__old__ccc.bin"), b"old").expect("seed stale");
        let drift = check_corpus(&dir, &expected, true).expect("check");
        assert_eq!(
            drift,
            vec![CorpusDrift::Stale("case000009__old__ccc.bin".to_owned())]
        );
        let stale = bless_corpus(&dir, &expected).expect("bless");
        assert_eq!(stale, vec!["case000009__old__ccc.bin".to_owned()]);
        assert!(dir.join("case000009__old__ccc.bin").exists());
    }

    #[test]
    fn corpus_reader_stops_after_expected_length_plus_one() {
        struct RepeatingReader {
            byte: u8,
            bytes_read: usize,
        }

        impl std::io::Read for RepeatingReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                buffer.fill(self.byte);
                self.bytes_read += buffer.len();
                Ok(buffer.len())
            }
        }

        let expected = vec![b'x'; CORPUS_COMPARE_CHUNK_BYTES * 2 + 17];
        let mut reader = RepeatingReader {
            byte: b'x',
            bytes_read: 0,
        };
        assert!(
            !corpus_reader_matches(&mut reader, &expected).expect("comparison succeeds"),
            "a trailing byte must make the artifact different"
        );
        assert_eq!(reader.bytes_read, expected.len() + 1);
    }
}
