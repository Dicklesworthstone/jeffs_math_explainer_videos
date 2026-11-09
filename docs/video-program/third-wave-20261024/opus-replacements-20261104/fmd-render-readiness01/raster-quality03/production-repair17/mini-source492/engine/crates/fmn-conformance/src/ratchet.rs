//! The public coverage ratchet (§11.5, §18; R1's public mitigation).
//!
//! With no LaTeX fallback, coverage *discipline* replaces fallback
//! discipline — public, precise, monotone. The four published numbers
//! (occurrence-weighted and unique-string coverage, parse vs layout split
//! out) are computed against G0-4's **frozen** corpus denominator and
//! recorded in `docs/ratchet/baseline.tsv`; the human-facing dashboard is
//! `docs/ratchet/dashboard.md`.
//!
//! **The pin-coupling insight that gives CI teeth without the private
//! corpus:** coverage is a pure function of (frozen corpus, fmd-math
//! pin). Between `SUITE.lock` bumps of `franken_markdown` the numbers
//! *cannot move*, so an always-on CI test simply asserts the baseline
//! names the current pin — any pin bump without a ratchet re-run fails —
//! while the corpus-bearing environments (dev boxes, the pin-bump ritual)
//! recompute, enforce monotonicity, and bless with
//! `RATCHET_UPDATE=1`. The 3b1b-authored strings never leave the private
//! fixture (§15.3): the committed artifacts carry numbers, construct
//! names, and hashes only.

use std::collections::BTreeSet;
use std::fmt::Write as _;

const BASELINE_FIELDS: [&str; 9] = [
    "rules_version",
    "corpus_hash",
    "franken_markdown_rev",
    "unique_total",
    "occurrence_total",
    "parse_unique",
    "parse_occurrences",
    "layout_unique",
    "layout_occurrences",
];
const MAX_BASELINE_TSV_BYTES: usize = 16 * 1024;
const MAX_TREND_TSV_BYTES: usize = 1024 * 1024;
const MAX_TREND_ROWS: usize = 4_096;
const MAX_FIELD_BYTES: usize = 128;

/// The exact counts behind the four published numbers, plus identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Baseline {
    /// The harvest lexer/extraction rules version (G0-4).
    pub rules_version: u32,
    /// The corpus identity hash (G0-4's denominator).
    pub corpus_hash: String,
    /// The franken_markdown commit the numbers were computed against.
    pub franken_markdown_rev: String,
    /// Distinct (mode, string) pairs in the denominator.
    pub unique_total: u64,
    /// Occurrence-weighted denominator.
    pub occurrence_total: u64,
    /// Distinct strings that parse end-to-end.
    pub parse_unique: u64,
    /// Occurrences that parse end-to-end.
    pub parse_occurrences: u64,
    /// Distinct strings that parse and lay out.
    pub layout_unique: u64,
    /// Occurrences that parse and lay out.
    pub layout_occurrences: u64,
}

impl Baseline {
    /// Serialize as the committed TSV (std-parseable, no YAML/TOML).
    #[must_use]
    pub fn to_tsv(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# The coverage-ratchet baseline (fm-mol). Counts are exact; the\n\
             # four public percentages derive from them. Regenerate with\n\
             # RATCHET_UPDATE=1 (needs the private corpus); the always-on CI\n\
             # test pins this file to the SUITE.lock franken_markdown rev."
        );
        let _ = writeln!(out, "rules_version\t{}", self.rules_version);
        let _ = writeln!(out, "corpus_hash\t{}", self.corpus_hash);
        let _ = writeln!(out, "franken_markdown_rev\t{}", self.franken_markdown_rev);
        let _ = writeln!(out, "unique_total\t{}", self.unique_total);
        let _ = writeln!(out, "occurrence_total\t{}", self.occurrence_total);
        let _ = writeln!(out, "parse_unique\t{}", self.parse_unique);
        let _ = writeln!(out, "parse_occurrences\t{}", self.parse_occurrences);
        let _ = writeln!(out, "layout_unique\t{}", self.layout_unique);
        let _ = writeln!(out, "layout_occurrences\t{}", self.layout_occurrences);
        out
    }

    /// Parse the committed TSV in the exact order emitted by [`Self::to_tsv`].
    ///
    /// # Errors
    ///
    /// Returns a bounded diagnostic for oversized input, non-canonical rows,
    /// missing/duplicate/unknown fields, invalid identities or numbers, and
    /// count relationships that cannot describe a corpus coverage snapshot.
    pub fn from_tsv(text: &str) -> Result<Self, String> {
        validate_text_envelope(text, MAX_BASELINE_TSV_BYTES, "baseline")?;
        let mut values = [""; BASELINE_FIELDS.len()];
        let mut field_index = 0_usize;
        for (line_index, line) in text.lines().enumerate() {
            let line_number = line_index + 1;
            if line.starts_with('#') {
                continue;
            }
            validate_data_line(line, line_number, "baseline")?;
            let mut parts = line.split('\t');
            let key = parts.next().unwrap_or_default();
            let value = parts.next().ok_or_else(|| {
                format!("baseline line {line_number}: expected one tab separator")
            })?;
            if parts.next().is_some() {
                return Err(format!(
                    "baseline line {line_number}: expected exactly two tab-separated fields"
                ));
            }
            validate_field_width(key, line_number, "baseline", "field name")?;
            validate_field_width(value, line_number, "baseline", "field value")?;
            let expected = BASELINE_FIELDS
                .get(field_index)
                .ok_or_else(|| format!("baseline line {line_number}: unexpected extra data row"))?;
            if key != *expected {
                return Err(format!(
                    "baseline line {line_number}: expected field `{expected}`, found `{key}`"
                ));
            }
            values[field_index] = value;
            field_index += 1;
        }
        if let Some(missing) = BASELINE_FIELDS.get(field_index) {
            return Err(format!("baseline: missing field `{missing}`"));
        }
        Self::from_fields(values, "baseline")
    }

    fn from_fields(fields: [&str; 9], context: &str) -> Result<Self, String> {
        validate_lower_hex(fields[1], 64, "corpus_hash", context)?;
        validate_lower_hex(fields[2], 40, "franken_markdown_rev", context)?;
        let baseline = Self {
            rules_version: parse_u32(fields[0], "rules_version", context)?,
            corpus_hash: fields[1].to_owned(),
            franken_markdown_rev: fields[2].to_owned(),
            unique_total: parse_u64(fields[3], "unique_total", context)?,
            occurrence_total: parse_u64(fields[4], "occurrence_total", context)?,
            parse_unique: parse_u64(fields[5], "parse_unique", context)?,
            parse_occurrences: parse_u64(fields[6], "parse_occurrences", context)?,
            layout_unique: parse_u64(fields[7], "layout_unique", context)?,
            layout_occurrences: parse_u64(fields[8], "layout_occurrences", context)?,
        };
        baseline.validate_counts(context)?;
        Ok(baseline)
    }

    fn validate_counts(&self, context: &str) -> Result<(), String> {
        for (subset_name, subset, superset_name, superset) in [
            (
                "unique_total",
                self.unique_total,
                "occurrence_total",
                self.occurrence_total,
            ),
            (
                "parse_unique",
                self.parse_unique,
                "unique_total",
                self.unique_total,
            ),
            (
                "layout_unique",
                self.layout_unique,
                "parse_unique",
                self.parse_unique,
            ),
            (
                "parse_occurrences",
                self.parse_occurrences,
                "occurrence_total",
                self.occurrence_total,
            ),
            (
                "layout_occurrences",
                self.layout_occurrences,
                "parse_occurrences",
                self.parse_occurrences,
            ),
            (
                "parse_unique",
                self.parse_unique,
                "parse_occurrences",
                self.parse_occurrences,
            ),
            (
                "layout_unique",
                self.layout_unique,
                "layout_occurrences",
                self.layout_occurrences,
            ),
        ] {
            if subset > superset {
                return Err(format!(
                    "{context}: `{subset_name}` ({subset}) exceeds `{superset_name}` ({superset})"
                ));
            }
        }
        for (unique_name, unique, occurrences_name, occurrences) in [
            (
                "unique_total",
                self.unique_total,
                "occurrence_total",
                self.occurrence_total,
            ),
            (
                "parse_unique",
                self.parse_unique,
                "parse_occurrences",
                self.parse_occurrences,
            ),
            (
                "layout_unique",
                self.layout_unique,
                "layout_occurrences",
                self.layout_occurrences,
            ),
        ] {
            if (unique == 0) != (occurrences == 0) {
                return Err(format!(
                    "{context}: `{unique_name}` and `{occurrences_name}` must either both be zero or both be positive"
                ));
            }
        }
        Ok(())
    }

    /// The four public percentages.
    #[must_use]
    pub fn percentages(&self) -> [f64; 4] {
        let pct = |n: u64, d: u64| {
            if d == 0 {
                0.0
            } else {
                100.0 * n as f64 / d as f64
            }
        };
        [
            pct(self.parse_occurrences, self.occurrence_total),
            pct(self.parse_unique, self.unique_total),
            pct(self.layout_occurrences, self.occurrence_total),
            pct(self.layout_unique, self.unique_total),
        ]
    }
}

fn validate_text_envelope(text: &str, max_bytes: usize, kind: &str) -> Result<(), String> {
    if text.len() > max_bytes {
        return Err(format!(
            "{kind}: input exceeds the {max_bytes}-byte format limit"
        ));
    }
    if text.as_bytes().contains(&b'\r') {
        return Err(format!("{kind}: CR line endings are not canonical"));
    }
    Ok(())
}

fn validate_data_line(line: &str, line_number: usize, kind: &str) -> Result<(), String> {
    if line.is_empty() {
        return Err(format!(
            "{kind} line {line_number}: blank rows are not canonical"
        ));
    }
    if line.trim() != line {
        return Err(format!(
            "{kind} line {line_number}: surrounding whitespace is not canonical"
        ));
    }
    Ok(())
}

fn validate_field_width(
    value: &str,
    line_number: usize,
    kind: &str,
    field_kind: &str,
) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{kind} line {line_number}: {field_kind} is empty"));
    }
    if value.len() > MAX_FIELD_BYTES {
        return Err(format!(
            "{kind} line {line_number}: {field_kind} exceeds {MAX_FIELD_BYTES} bytes"
        ));
    }
    Ok(())
}

fn parse_u64(value: &str, field: &str, context: &str) -> Result<u64, String> {
    if !is_canonical_decimal(value) {
        return Err(format!(
            "{context}: `{field}` must be canonical unsigned decimal"
        ));
    }
    value
        .parse()
        .map_err(|_| format!("{context}: `{field}` exceeds u64"))
}

fn parse_u32(value: &str, field: &str, context: &str) -> Result<u32, String> {
    let value = parse_u64(value, field, context)?;
    u32::try_from(value).map_err(|_| format!("{context}: `{field}` exceeds u32"))
}

fn is_canonical_decimal(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn validate_lower_hex(
    value: &str,
    expected_bytes: usize,
    field: &str,
    context: &str,
) -> Result<(), String> {
    if value.len() != expected_bytes
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{context}: `{field}` must be exactly {expected_bytes} lowercase hexadecimal bytes"
        ));
    }
    Ok(())
}

/// Parse the committed trend TSV without dropping malformed rows.
///
/// # Errors
///
/// Returns a bounded diagnostic for oversized input, non-canonical or
/// non-nine-field rows, invalid baseline snapshots, too many rows, or a
/// duplicate `franken_markdown` revision.
pub fn parse_trend_tsv(text: &str) -> Result<Vec<Baseline>, String> {
    validate_text_envelope(text, MAX_TREND_TSV_BYTES, "trend")?;
    let mut trend = Vec::new();
    let mut revisions = BTreeSet::new();
    for (line_index, line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        if line.starts_with('#') {
            continue;
        }
        validate_data_line(line, line_number, "trend")?;
        if trend.len() == MAX_TREND_ROWS {
            return Err(format!(
                "trend line {line_number}: row count exceeds {MAX_TREND_ROWS}"
            ));
        }
        let mut parts = line.split('\t');
        let mut fields = [""; 9];
        for (field_index, field) in fields.iter_mut().enumerate() {
            *field = parts.next().ok_or_else(|| {
                format!(
                    "trend line {line_number}: expected exactly 9 tab-separated fields, found {field_index}"
                )
            })?;
            validate_field_width(field, line_number, "trend", "field value")?;
        }
        if parts.next().is_some() {
            return Err(format!(
                "trend line {line_number}: expected exactly 9 tab-separated fields"
            ));
        }
        let baseline = Baseline::from_fields(
            [
                fields[1], fields[2], fields[0], fields[3], fields[4], fields[5], fields[6],
                fields[7], fields[8],
            ],
            &format!("trend line {line_number}"),
        )?;
        if !revisions.insert(baseline.franken_markdown_rev.clone()) {
            return Err(format!(
                "trend line {line_number}: duplicate franken_markdown revision"
            ));
        }
        trend.push(baseline);
    }
    Ok(trend)
}

/// The ratchet rule: within a `rules_version` and denominator, none of the
/// four counts may decrease. Returns the violations (empty = pass).
#[must_use]
pub fn ratchet_violations(baseline: &Baseline, current: &Baseline) -> Vec<String> {
    let mut violations = Vec::new();
    if baseline.rules_version != current.rules_version
        || baseline.corpus_hash != current.corpus_hash
    {
        // A rules/denominator change restates the corpus (G0-4 rule 5);
        // comparability resets and the boundary is annotated on the chart.
        return violations;
    }
    let mut check = |name: &str, before: u64, now: u64| {
        if now < before {
            violations.push(format!("{name} regressed: {before} → {now}"));
        }
    };
    check(
        "parse_occurrences",
        baseline.parse_occurrences,
        current.parse_occurrences,
    );
    check("parse_unique", baseline.parse_unique, current.parse_unique);
    check(
        "layout_occurrences",
        baseline.layout_occurrences,
        current.layout_occurrences,
    );
    check(
        "layout_unique",
        baseline.layout_unique,
        current.layout_unique,
    );
    violations
}

/// One pending construct for the dashboard: a named blocker with its
/// occurrence mass and where it is tracked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    /// Construct name, table scheme.
    pub construct: String,
    /// Occurrences blocked on it.
    pub occurrences: u64,
    /// Where it is tracked.
    pub tracked: String,
}

/// Render the public dashboard (numbers, construct names, hashes — never
/// the corpus strings; §15.3) plus the trend log and the R1 escalation
/// path.
#[must_use]
pub fn render_dashboard(
    baseline: &Baseline,
    oracle: &OracleSummary,
    parse_pending: &[Pending],
    layout_pending: &[Pending],
    trend: &[Baseline],
) -> String {
    let [po, pu, lo, lu] = baseline.percentages();
    let share = |part: u64, whole: u64| {
        if whole == 0 {
            0.0
        } else {
            100.0 * part as f64 / whole as f64
        }
    };
    let oracle_occ = share(oracle.occurrences_within_5, baseline.occurrence_total);
    let oracle_uniq = share(oracle.within_5 as u64, baseline.unique_total);
    let mut out = String::new();
    let _ = writeln!(out, "# The fmd-math coverage ratchet\n");
    let _ = writeln!(
        out,
        "The headline metric of the no-LaTeX pivot (§11.5): what fraction of the\n\
         real 3b1b formula corpus typesets natively. The denominator is **frozen**\n\
         (G0-4: `{}` distinct strings, `{}` occurrences, corpus hash\n\
         `{}`, rules_version {}); the numbers may only rise.\n",
        baseline.unique_total,
        baseline.occurrence_total,
        baseline.corpus_hash,
        baseline.rules_version
    );
    let _ = writeln!(out, "**Computed against franken_markdown `{}`.**\n", {
        &baseline.franken_markdown_rev[..12.min(baseline.franken_markdown_rev.len())]
    });
    let _ = writeln!(out, "| Plane | Occurrence-weighted | Unique-string |");
    let _ = writeln!(out, "|---|---|---|");
    let _ = writeln!(out, "| **Parse** | {po:.3} % | {pu:.3} % |");
    let _ = writeln!(
        out,
        "| **Parse + typeset returned Ok** | {lo:.3} % | {lu:.3} % |"
    );
    let _ = writeln!(
        out,
        "| **Layout within 5% of real TeX (oracle)** | {oracle_occ:.3} % | {oracle_uniq:.3} % |"
    );
    let _ = writeln!(
        out,
        "\n\"Typeset returned Ok\" means fmd-math produced a layout without an \
         error. It does not show that the layout is right. {} The oracle row \
         counts only those verified strings against the whole corpus; \
         unsampled strings count as unverified.",
        oracle_dashboard_line(oracle)
    );
    let _ = writeln!(out, "\n## Pending constructs (parse plane)\n");
    let _ = writeln!(out, "| Construct | Occurrences blocked | Tracked at |");
    let _ = writeln!(out, "|---|---|---|");
    for p in parse_pending {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            p.construct, p.occurrences, p.tracked
        );
    }
    let _ = writeln!(out, "\n## Pending at layout (parse succeeds)\n");
    let _ = writeln!(out, "| Construct | Occurrences blocked | Tracked at |");
    let _ = writeln!(out, "|---|---|---|");
    for p in layout_pending {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            p.construct, p.occurrences, p.tracked
        );
    }
    let _ = writeln!(out, "\n## Trend (by franken_markdown rev)\n");
    let _ = writeln!(
        out,
        "| Rev | Parse occ. % | Parse uniq. % | Layout occ. % | Layout uniq. % |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    for b in trend {
        let [tpo, tpu, tlo, tlu] = b.percentages();
        let _ = writeln!(
            out,
            "| `{}` | {tpo:.3} | {tpu:.3} | {tlo:.3} | {tlu:.3} |",
            &b.franken_markdown_rev[..12.min(b.franken_markdown_rev.len())]
        );
    }
    let _ = writeln!(
        out,
        "\n## How this is enforced\n\n\
         - Coverage is a pure function of (frozen corpus, fmd-math pin), so the\n\
           numbers can only move when `SUITE.lock`'s `franken_markdown` row moves.\n\
           An always-on CI test requires `baseline.tsv` to name the current pin:\n\
           **a pin bump without a ratchet re-run fails CI.**\n\
         - On corpus-bearing machines the ratchet test recomputes all four\n\
           counts and fails on any decrease; `RATCHET_UPDATE=1` blesses a\n\
           deliberate advance (with this dashboard regenerated in the same\n\
           commit).\n\
         - Every non-tier-1 construct must fail with its precise, named,\n\
           tier-tagged error — audited construct-by-construct against the G0-4\n\
           table in always-on CI. Nothing ever fails silently.\n\n\
         ## The escalation path (R1, the G2 checkpoint)\n\n\
         If coverage misses a gate's criteria, the response is a **public\n\
         amendment with a construct-sprint plan** — never a silent slip: the\n\
         gap is named construct-by-construct above, each with its tracked\n\
         bead, and the gate review adjudicates the sprint scope in the open.\n\n\
         ## Licensing (§15.3)\n\n\
         The corpus strings are 3b1b-authored course material and stay in the\n\
         private fixture; this dashboard publishes **numbers, construct names,\n\
         and hashes only**. Anyone with the pinned trees can reproduce the\n\
         denominator byte-for-byte via `scripts/harvest_tex_corpus.py`."
    );
    out
}

// ── The corpus TeX box oracle (fm-tex-layout-oracle-bkbc) ───────────────

/// The oracle's committed fixture: real TeX's boxes for a stratified corpus
/// sample, captured by `scripts/capture_tex_corpus_boxes.py`. Rows name
/// their strings by digest only (§15.3).
pub const ORACLE_FIXTURE: &str = include_str!("../fixtures/tex_corpus_boxes.v1.tsv");

/// One fixture row: real TeX's box for one sampled corpus string, in ems.
#[derive(Clone, Debug, PartialEq)]
pub struct OracleRow {
    /// The string's public digest, `sha256(mode + NUL + string)`.
    pub digest: String,
    /// TeX's `\wd`.
    pub width: f64,
    /// TeX's `\ht`.
    pub height: f64,
    /// TeX's `\dp`.
    pub depth: f64,
    /// Occurrences across the corpus.
    pub count: u64,
    /// Construct classes the string exercises.
    pub classes: Vec<String>,
}

/// Parse the oracle fixture. Comment lines (`#`) and blank lines are
/// skipped.
///
/// # Errors
///
/// A row without six tab-separated fields, or with an unparsable number.
pub fn parse_oracle_fixture(text: &str) -> Result<Vec<OracleRow>, String> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.starts_with('#') && !line.trim().is_empty())
        .map(|(i, line)| {
            let bad = || format!("oracle fixture line {}: {line:?}", i + 1);
            let fields: Vec<&str> = line.split('\t').collect();
            let [digest, width, height, depth, count, classes] = fields.as_slice() else {
                return Err(bad());
            };
            let number = |field: &str| field.parse::<f64>().map_err(|_| bad());
            Ok(OracleRow {
                digest: (*digest).to_owned(),
                width: number(width)?,
                height: number(height)?,
                depth: number(depth)?,
                count: count.parse().map_err(|_| bad())?,
                // `|` appears in no construct name (`\,` and `\ ` do).
                classes: classes.split('|').map(str::to_owned).collect(),
            })
        })
        .collect()
}

/// fmd-math's box (width, height, depth in ems) for one sampled string, or
/// `None` where fmd-math refused it.
pub type OracleBox = Option<(f64, f64, f64)>;

/// Lay out every sampled string as the Tex surface does: display style,
/// the default macro pack. `texts` maps digests to the private corpus
/// strings.
///
/// # Errors
///
/// A fixture row whose digest `texts` lacks, or a missing face or pack.
pub fn run_layout_oracle(
    rows: &[OracleRow],
    texts: &std::collections::HashMap<String, String>,
) -> Result<Vec<OracleBox>, String> {
    let engine = fmd_math::Engine::bundled().map_err(|error| format!("bundled faces: {error}"))?;
    let pack = fmd_math::MacroSet::pack("fmd-math/pack/default")
        .ok_or_else(|| "the default macro pack is missing".to_owned())?;
    rows.iter()
        .map(|row| {
            let text = texts
                .get(&row.digest)
                .ok_or_else(|| format!("fixture row {} is not in the corpus", row.digest))?;
            Ok(engine
                .typeset_with_macros(text, fmd_math::Style::Display, &pack)
                .ok()
                .map(|layout| (layout.width, layout.height, layout.depth)))
        })
        .collect()
}

/// How far fmd-math's box is from TeX's: the larger of the relative width
/// error and the height and depth errors relative to TeX's total height,
/// so a misplaced baseline counts too. Multi-line strings keep their first
/// row's baseline on both sides, as the Reference's align* rows do.
/// Infinite where fmd-math refused the string.
#[must_use]
pub fn oracle_error(row: &OracleRow, laid: OracleBox) -> f64 {
    let Some((width, height, depth)) = laid else {
        return f64::INFINITY;
    };
    let total = row.height + row.depth;
    let w = (width / row.width - 1.0).abs();
    let h = (height - row.height).abs() / total;
    let d = (depth - row.depth).abs() / total;
    w.max(h).max(d)
}

/// The oracle's counts over its sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OracleSummary {
    /// Sampled strings.
    pub rows: usize,
    /// Strings within 5% of TeX's box.
    pub within_5: usize,
    /// Within 10%.
    pub within_10: usize,
    /// Strings fmd-math refused.
    pub refused: usize,
    /// Corpus occurrences of the sampled strings.
    pub occurrences: u64,
    /// Occurrences of the strings within 5%.
    pub occurrences_within_5: u64,
}

/// Tally per-row errors into the oracle's counts.
#[must_use]
pub fn summarize_oracle(rows: &[OracleRow], errors: &[f64]) -> OracleSummary {
    let mut s = OracleSummary::default();
    for (row, &error) in rows.iter().zip(errors) {
        s.rows += 1;
        s.within_5 += usize::from(error <= 0.05);
        s.within_10 += usize::from(error <= 0.10);
        s.refused += usize::from(error.is_infinite());
        s.occurrences += row.count;
        if error <= 0.05 {
            s.occurrences_within_5 += row.count;
        }
    }
    s
}

/// The sentence the dashboard publishes about the oracle. The oracle test
/// requires the committed dashboard to carry it verbatim, so the published
/// numbers cannot drift from what the oracle measures.
#[must_use]
pub fn oracle_dashboard_line(s: &OracleSummary) -> String {
    format!(
        "The corpus TeX box oracle (fm-tex-layout-oracle-bkbc) lays out {} \
         strings, sampled by construct class and occurrence ({} occurrences), \
         and compares each box with real TeX's: {} are within 5% and {} within \
         10%, covering {} occurrences within 5%.",
        s.rows, s.occurrences, s.within_5, s.within_10, s.occurrences_within_5
    )
}

const ORACLE_SCHEMA: &str = "fmn.tex-corpus-boxes.v1";

/// A JSON number with five decimals; `null` for a refusal's infinite error.
fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.5}")
    } else {
        "null".to_owned()
    }
}

/// The oracle run as bounded, deterministic NDJSON (schema
/// `fmn.tex-corpus-boxes.v1`): one `row` record per sampled string in
/// fixture order (digest, occurrences, TeX's and fmd-math's boxes, error),
/// one `class` record per construct class with the lowest within-5% share
/// first, and a closing `summary` with the floors and the decision. Rows
/// carry digests, never strings (§15.3).
#[must_use]
pub fn oracle_ndjson(
    rows: &[OracleRow],
    laid: &[OracleBox],
    errors: &[f64],
    floors: (usize, usize),
) -> String {
    #[derive(Default)]
    struct Class<'a> {
        rows: usize,
        within_5: usize,
        within_10: usize,
        worst: f64,
        worst_digest: &'a str,
    }
    let mut out = String::new();
    let mut classes: std::collections::BTreeMap<&str, Class<'_>> =
        std::collections::BTreeMap::new();
    for ((row, laid), &error) in rows.iter().zip(laid).zip(errors) {
        let native = laid.map_or_else(
            || "null".to_owned(),
            |(w, h, d)| format!("[{w:.5},{h:.5},{d:.5}]"),
        );
        let _ = writeln!(
            out,
            "{{\"schema\":\"{ORACLE_SCHEMA}\",\"kind\":\"row\",\"digest\":\"{}\",\"count\":{},\
             \"tex\":[{:.5},{:.5},{:.5}],\"native\":{native},\"error\":{}}}",
            row.digest,
            row.count,
            row.width,
            row.height,
            row.depth,
            json_number(error)
        );
        for class in &row.classes {
            let tally = classes.entry(class).or_default();
            tally.rows += 1;
            tally.within_5 += usize::from(error <= 0.05);
            tally.within_10 += usize::from(error <= 0.10);
            if error >= tally.worst {
                tally.worst = error;
                tally.worst_digest = &row.digest;
            }
        }
    }
    let mut table: Vec<(&&str, &Class<'_>)> = classes.iter().collect();
    let share = |c: &Class<'_>| c.within_5 as f64 / c.rows as f64;
    table.sort_by(|a, b| {
        share(a.1)
            .total_cmp(&share(b.1))
            .then(b.1.rows.cmp(&a.1.rows))
    });
    for (name, c) in table {
        let mut class = String::new();
        crate::e2e::escape_json_string(name, &mut class);
        let _ = writeln!(
            out,
            "{{\"schema\":\"{ORACLE_SCHEMA}\",\"kind\":\"class\",\"class\":{class},\"rows\":{},\
             \"within_5\":{},\"within_10\":{},\"worst\":{},\"worst_digest\":\"{}\"}}",
            c.rows,
            c.within_5,
            c.within_10,
            json_number(c.worst),
            c.worst_digest
        );
    }
    let s = summarize_oracle(rows, errors);
    let decision = if s.within_5 >= floors.0 && s.within_10 >= floors.1 {
        "pass"
    } else {
        "below_floor"
    };
    let _ = writeln!(
        out,
        "{{\"schema\":\"{ORACLE_SCHEMA}\",\"kind\":\"summary\",\"rows\":{},\"within_5\":{},\
         \"within_10\":{},\"refused\":{},\"occurrences\":{},\"occurrences_within_5\":{},\
         \"floor_5\":{},\"floor_10\":{},\"decision\":\"{decision}\"}}",
        s.rows,
        s.within_5,
        s.within_10,
        s.refused,
        s.occurrences,
        s.occurrences_within_5,
        floors.0,
        floors.1
    );
    out
}

// ── A minimal JSON-object reader for the corpus lines (governed closure:
//    no serde) ──────────────────────────────────────────────────────────

/// One private-corpus record: harvest mode (`math` or `text`), the string,
/// and its occurrence count. Other fields are skipped.
pub struct CorpusEntry {
    /// `math` or `text`.
    pub mode: String,
    /// The TeX string.
    pub text: String,
    /// Occurrences across the corpus.
    pub count: u64,
}

/// The public digest naming a corpus string, `sha256(mode + NUL + string)`
/// in lowercase hex: the convention of `docs/g0/g0-4-corpus/denominator.tsv`
/// and of the corpus records' own `sha256` field. Committed fixtures name
/// strings this way because the strings themselves never ship (§15.3).
#[must_use]
pub fn corpus_digest(mode: &str, text: &str) -> String {
    let mut material = Vec::with_capacity(mode.len() + 1 + text.len());
    material.extend_from_slice(mode.as_bytes());
    material.push(0);
    material.extend_from_slice(text.as_bytes());
    fmn_hash::sha256(&material).to_hex()
}

/// Parse one corpus JSONL line; `None` for anything malformed.
#[must_use]
pub fn parse_corpus_entry(line: &str) -> Option<CorpusEntry> {
    let mut mode = None;
    let mut text = None;
    let mut count = None;
    let bytes = line.as_bytes();
    let mut i = skip_ws(bytes, 0);
    if bytes.get(i) != Some(&b'{') {
        return None;
    }
    i += 1;
    loop {
        i = skip_ws(bytes, i);
        match bytes.get(i) {
            Some(b'}') => break,
            Some(b',') => {
                i += 1;
                continue;
            }
            Some(b'"') => {}
            _ => return None,
        }
        let (key, ni) = read_string(line, i)?;
        i = skip_ws(bytes, ni);
        if bytes.get(i) != Some(&b':') {
            return None;
        }
        i = skip_ws(bytes, i + 1);
        match key.as_str() {
            "mode" => {
                let (v, ni) = read_string(line, i)?;
                mode = Some(v);
                i = ni;
            }
            "text" => {
                let (v, ni) = read_string(line, i)?;
                text = Some(v);
                i = ni;
            }
            "count" => {
                let (v, ni) = read_number(bytes, i)?;
                count = Some(v);
                i = ni;
            }
            _ => i = skip_value(line, i)?,
        }
    }
    Some(CorpusEntry {
        mode: mode?,
        text: text?,
        count: count?,
    })
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        i += 1;
    }
    i
}

fn read_string(s: &str, start: usize) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut out = String::new();
    let mut i = start + 1;
    loop {
        let rest = s.get(i..)?;
        let mut chars = rest.char_indices();
        let (_, c) = chars.next()?;
        match c {
            '"' => return Some((out, i + 1)),
            '\\' => {
                let (_, esc) = chars.next()?;
                i += 1 + esc.len_utf8();
                match esc {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{0008}'),
                    'f' => out.push('\u{000C}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let hex = s.get(i..i + 4)?;
                        let cp = u32::from_str_radix(hex, 16).ok()?;
                        i += 4;
                        if (0xD800..0xDC00).contains(&cp) {
                            if s.get(i..i + 2)? != "\\u" {
                                return None;
                            }
                            let lo = u32::from_str_radix(s.get(i + 2..i + 6)?, 16).ok()?;
                            i += 6;
                            let combined =
                                0x10000 + ((cp - 0xD800) << 10) + lo.checked_sub(0xDC00)?;
                            out.push(char::from_u32(combined)?);
                        } else {
                            out.push(char::from_u32(cp)?);
                        }
                    }
                    _ => return None,
                }
            }
            other => {
                out.push(other);
                i += other.len_utf8();
            }
        }
    }
}

fn read_number(bytes: &[u8], start: usize) -> Option<(u64, usize)> {
    let mut i = start;
    let mut val: u64 = 0;
    let mut any = false;
    while let Some(d) = bytes.get(i).copied().filter(u8::is_ascii_digit) {
        val = val.checked_mul(10)?.checked_add(u64::from(d - b'0'))?;
        i += 1;
        any = true;
    }
    any.then_some((val, i))
}

fn skip_value(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    match bytes.get(start)? {
        b'"' => read_string(s, start).map(|(_, i)| i),
        b'[' => {
            let mut depth = 0_usize;
            let mut i = start;
            loop {
                match bytes.get(i)? {
                    b'"' => i = read_string(s, i)?.1,
                    b'[' => {
                        depth += 1;
                        i += 1;
                    }
                    b']' => {
                        depth -= 1;
                        i += 1;
                        if depth == 0 {
                            return Some(i);
                        }
                    }
                    _ => i += 1,
                }
            }
        }
        _ => {
            let mut i = start;
            while !matches!(bytes.get(i), None | Some(b',' | b'}' | b']')) {
                i += 1;
            }
            Some(i)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Baseline {
        Baseline {
            rules_version: 1,
            corpus_hash: "a".repeat(64),
            franken_markdown_rev: "d".repeat(40),
            unique_total: 100,
            occurrence_total: 1000,
            parse_unique: 90,
            parse_occurrences: 950,
            layout_unique: 85,
            layout_occurrences: 920,
        }
    }

    #[test]
    fn tsv_round_trips() {
        let b = base();
        assert_eq!(Baseline::from_tsv(&b.to_tsv()), Ok(b));
    }

    #[test]
    fn baseline_parser_rejects_ambiguous_or_noncanonical_rows() {
        let canonical = base().to_tsv();
        for (name, malformed, needle) in [
            (
                "duplicate",
                canonical.replacen("corpus_hash\t", "rules_version\t1\ncorpus_hash\t", 1),
                "expected field `corpus_hash`, found `rules_version`",
            ),
            (
                "unknown",
                canonical.replacen("corpus_hash\t", "corpus_digest\t", 1),
                "expected field `corpus_hash`, found `corpus_digest`",
            ),
            (
                "extra column",
                canonical.replacen("rules_version\t1", "rules_version\t1\textra", 1),
                "exactly two tab-separated fields",
            ),
            (
                "surrounding whitespace",
                canonical.replacen("rules_version\t1", " rules_version\t1", 1),
                "surrounding whitespace",
            ),
            (
                "noncanonical number",
                canonical.replacen("unique_total\t100", "unique_total\t0100", 1),
                "canonical unsigned decimal",
            ),
        ] {
            let error = Baseline::from_tsv(&malformed).unwrap_err();
            assert!(error.contains(needle), "{name}: {error}");
        }
    }

    #[test]
    fn baseline_parser_rejects_invalid_identity_and_count_lattice() {
        let canonical = base().to_tsv();
        let bad_hash = canonical.replacen(&"a".repeat(64), "xyz", 1);
        assert!(
            Baseline::from_tsv(&bad_hash)
                .unwrap_err()
                .contains("64 lowercase hexadecimal bytes")
        );
        let impossible = canonical.replacen("parse_unique\t90", "parse_unique\t101", 1);
        assert!(
            Baseline::from_tsv(&impossible)
                .unwrap_err()
                .contains("`parse_unique` (101) exceeds `unique_total` (100)")
        );
        let mismatched_zero = canonical
            .replacen("layout_unique\t85", "layout_unique\t0", 1)
            .replacen("layout_occurrences\t920", "layout_occurrences\t1", 1);
        assert!(
            Baseline::from_tsv(&mismatched_zero)
                .unwrap_err()
                .contains("must either both be zero or both be positive")
        );
    }

    fn trend_row(baseline: &Baseline) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            baseline.franken_markdown_rev,
            baseline.rules_version,
            baseline.corpus_hash,
            baseline.unique_total,
            baseline.occurrence_total,
            baseline.parse_unique,
            baseline.parse_occurrences,
            baseline.layout_unique,
            baseline.layout_occurrences,
        )
    }

    #[test]
    fn trend_parser_preserves_every_valid_row() {
        let first = base();
        let mut second = base();
        second.franken_markdown_rev = "e".repeat(40);
        second.layout_unique += 1;
        second.layout_occurrences += 1;
        let text = format!("# trend\n{}{}", trend_row(&first), trend_row(&second));
        assert_eq!(parse_trend_tsv(&text), Ok(vec![first, second]));
    }

    #[test]
    fn trend_parser_rejects_malformed_middle_rows_and_duplicate_revisions() {
        let row = trend_row(&base());
        let malformed = format!("{row}not\tenough\tfields\n{row}");
        assert!(
            parse_trend_tsv(&malformed)
                .unwrap_err()
                .contains("trend line 2: expected exactly 9 tab-separated fields")
        );
        let duplicate = format!("{row}{row}");
        assert!(
            parse_trend_tsv(&duplicate)
                .unwrap_err()
                .contains("trend line 2: duplicate franken_markdown revision")
        );
    }

    #[test]
    fn the_ratchet_fails_on_any_decrease() {
        // The deliberate-regression negative test: removing a construct
        // (coverage drops) must fail.
        let b = base();
        let mut worse = base();
        worse.parse_occurrences -= 1;
        let v = ratchet_violations(&b, &worse);
        assert_eq!(v.len(), 1);
        assert!(v[0].contains("parse_occurrences regressed"));
        let mut worse = base();
        worse.layout_unique -= 5;
        assert_eq!(ratchet_violations(&b, &worse).len(), 1);
    }

    #[test]
    fn the_ratchet_passes_on_advance_or_equality() {
        let b = base();
        assert!(ratchet_violations(&b, &b).is_empty());
        let mut better = base();
        better.layout_occurrences += 10;
        assert!(ratchet_violations(&b, &better).is_empty());
    }

    #[test]
    fn a_rules_version_change_resets_comparability() {
        let b = base();
        let mut restated = base();
        restated.rules_version = 2;
        restated.parse_occurrences = 0;
        assert!(
            ratchet_violations(&b, &restated).is_empty(),
            "a restated denominator annotates the chart instead of failing"
        );
    }

    #[test]
    fn percentages_derive_from_counts() {
        let [po, pu, lo, lu] = base().percentages();
        assert!((po - 95.0).abs() < 1e-9);
        assert!((pu - 90.0).abs() < 1e-9);
        assert!((lo - 92.0).abs() < 1e-9);
        assert!((lu - 85.0).abs() < 1e-9);
    }

    #[test]
    fn dashboard_renders_numbers_and_never_strings() {
        let oracle = OracleSummary {
            rows: 20,
            within_5: 8,
            within_10: 12,
            refused: 0,
            occurrences: 400,
            occurrences_within_5: 250,
        };
        let d = render_dashboard(
            &base(),
            &oracle,
            &[Pending {
                construct: "\\substack".to_owned(),
                occurrences: 5,
                tracked: "fm-j5t".to_owned(),
            }],
            &[],
            &[base()],
        );
        assert!(d.contains("95.000 %"));
        assert!(d.contains("\\substack"));
        assert!(d.contains("escalation path"));
        // 250 of the corpus's 1000 occurrences and 8 of its 100 strings are
        // verified within 5%.
        assert!(d.contains("| 25.000 % | 8.000 % |"));
        assert!(d.contains(&oracle_dashboard_line(&oracle)));
    }

    #[test]
    fn oracle_fixture_rows_parse_and_errors_tally() {
        let rows = parse_oracle_fixture(
            "# header\n\
             aa\t1.00000\t0.60000\t0.40000\t3\t\\frac|script:sup\n\
             bb\t2.00000\t0.50000\t0.00000\t1\t(plain)\n",
        )
        .expect("well-formed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].classes, ["\\frac", "script:sup"]);
        // Width 2% wide, height 3% of the total high: within 5%.
        let near = oracle_error(&rows[0], Some((1.02, 0.63, 0.40)));
        assert!((near - 0.03).abs() < 1e-12, "{near}");
        let refused = oracle_error(&rows[1], None);
        assert!(refused.is_infinite());
        let s = summarize_oracle(&rows, &[near, refused]);
        assert_eq!((s.rows, s.within_5, s.within_10, s.refused), (2, 1, 1, 1));
        assert_eq!((s.occurrences, s.occurrences_within_5), (4, 3));
        let log = oracle_ndjson(
            &rows,
            &[Some((1.02, 0.63, 0.40)), None],
            &[near, refused],
            (1, 1),
        );
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines.len(), 2 + 3 + 1, "two rows, three classes, a summary");
        assert!(lines[1].contains("\"native\":null,\"error\":null"));
        assert!(
            lines[2].contains("\"class\":\"(plain)\""),
            "worst class first"
        );
        assert!(lines[5].ends_with("\"decision\":\"pass\"}"));
        assert!(parse_oracle_fixture("aa\t1.0\t0.5\n").is_err());
    }
}
