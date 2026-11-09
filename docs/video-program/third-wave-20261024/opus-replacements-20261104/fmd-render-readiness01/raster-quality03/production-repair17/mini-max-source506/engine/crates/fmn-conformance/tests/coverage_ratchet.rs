//! The coverage ratchet's CI teeth (fm-mol).
//!
//! Always-on (every checkout):
//! - the committed baseline must name the CURRENT `SUITE.lock`
//!   franken_markdown pin — a pin bump without a ratchet re-run fails here;
//! - the dashboard's headline must match the baseline (no drift between
//!   the published page and the enforced counts);
//! - the per-construct named-error audit: every non-tier-1 construct of
//!   the G0-4 table must fail with its precise, tier-tagged error — no
//!   construct ever fails silently.
//!
//! Corpus-bearing environments (`FMN_TEX_CORPUS` set, or the default
//! `corpus/tex_corpus.jsonl` present): recompute all four counts against
//! the frozen denominator, fail on any decrease, and bless deliberate
//! advances with `RATCHET_UPDATE=1` (regenerating baseline, trend, and
//! dashboard in one stroke).

use fmn_conformance::ratchet::{
    Baseline, ORACLE_FIXTURE, OracleSummary, Pending, corpus_digest, oracle_error,
    parse_corpus_entry, parse_oracle_fixture, parse_trend_tsv, ratchet_violations,
    render_dashboard, run_layout_oracle, summarize_oracle,
};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_SUITE_LOCK_BYTES: u64 = 1024 * 1024;
const MAX_BASELINE_BYTES: u64 = 16 * 1024;
const MAX_TREND_BYTES: u64 = 1024 * 1024;
const MAX_DASHBOARD_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CONSTRUCT_TABLE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CORPUS_BYTES: u64 = 64 * 1024 * 1024;

fn repo_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name)
}

fn read_utf8_bounded(reader: impl Read, label: &str, max_bytes: u64) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {label}: {error}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(format!(
            "{label} exceeds the {max_bytes}-byte resource limit"
        ));
    }
    String::from_utf8(bytes).map_err(|error| format!("{label} is not UTF-8: {error}"))
}

fn file_utf8_bounded(path: &Path, label: &str, max_bytes: u64) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("cannot read {label}: {error}"))?;
    read_utf8_bounded(file, label, max_bytes)
}

fn repo_file(name: &str, max_bytes: u64) -> Result<String, String> {
    let path = repo_path(name);
    file_utf8_bounded(&path, name, max_bytes)
}

fn suite_lock_franken_markdown_rev() -> Result<String, String> {
    let lock = repo_file("SUITE.lock", MAX_SUITE_LOCK_BYTES)?;
    for line in lock.lines() {
        let line = line.trim();
        if let Some((rev, _)) = line
            .strip_prefix("franken_markdown\t")
            .and_then(|rest| rest.split_once('\t'))
        {
            return Ok(rev.trim().to_owned());
        }
    }
    Err("SUITE.lock must pin franken_markdown".to_owned())
}

fn committed_baseline() -> Result<Baseline, String> {
    Baseline::from_tsv(&repo_file("docs/ratchet/baseline.tsv", MAX_BASELINE_BYTES)?)
        .map_err(|error| format!("docs/ratchet/baseline.tsv is malformed: {error}"))
}

// ── Always-on: the pin coupling ─────────────────────────────────────────

#[test]
fn baseline_names_the_current_franken_markdown_pin() -> Result<(), String> {
    let baseline = committed_baseline()?;
    let pin = suite_lock_franken_markdown_rev()?;
    assert_eq!(
        baseline.franken_markdown_rev, pin,
        "SUITE.lock moved franken_markdown without a ratchet re-run: \
         recompute with the corpus and RATCHET_UPDATE=1 (docs/ratchet/dashboard.md)"
    );
    Ok(())
}

#[test]
fn dashboard_headline_matches_the_baseline() -> Result<(), String> {
    let baseline = committed_baseline()?;
    let dashboard = repo_file("docs/ratchet/dashboard.md", MAX_DASHBOARD_BYTES)?;
    let [po, pu, lo, lu] = baseline.percentages();
    for needle in [
        format!("| **Parse** | {po:.3} % | {pu:.3} % |"),
        format!("| **Parse + typeset returned Ok** | {lo:.3} % | {lu:.3} % |"),
        // The oracle row's numbers need the corpus; tex_corpus_boxes checks
        // them where it is present.
        "| **Layout within 5% of real TeX (oracle)** | ".to_owned(),
        baseline.corpus_hash.clone(),
    ] {
        assert!(
            dashboard.contains(&needle),
            "dashboard drifted from the baseline (missing `{needle}`); re-bless"
        );
    }
    Ok(())
}

#[test]
fn trend_is_monotone_and_ends_at_the_baseline() -> Result<(), String> {
    let baseline = committed_baseline()?;
    let trend = parse_trend_tsv(&repo_file("docs/ratchet/trend.tsv", MAX_TREND_BYTES)?)
        .map_err(|error| format!("docs/ratchet/trend.tsv is malformed: {error}"))?;
    assert!(
        !trend.is_empty(),
        "the trend log carries at least the first bless"
    );
    for pair in trend.windows(2) {
        assert!(
            ratchet_violations(&pair[0], &pair[1]).is_empty(),
            "the committed trend itself must be monotone"
        );
    }
    let last = trend
        .last()
        .ok_or_else(|| "the trend log must carry at least the first bless".to_owned())?;
    assert_eq!(last, &baseline, "the trend's last row is the baseline");
    Ok(())
}

// ── Always-on: the per-construct named-error audit ──────────────────────

#[test]
fn every_non_tier1_construct_fails_with_its_named_tiered_error() -> Result<(), String> {
    let table = repo_file(
        "docs/g0/g0-4-corpus/construct_table.tsv",
        MAX_CONSTRUCT_TABLE_BYTES,
    )?;
    let mut audited = 0_usize;
    for line in table.lines() {
        if line.starts_with('#') || line.trim().is_empty() || line.starts_with("rank\t") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(fields.len() >= 9, "short row: {line}");
        let construct = fields[1];
        let tier = fields[7];
        if tier != "T2" || construct.starts_with("char:") {
            // char: rows are layout-coverage surface (parse-transparent);
            // T1 support is asserted by fmn-tex's tier-table cross-check.
            continue;
        }
        let probes = probe_sources(construct);
        // A construct that parses in some mode on some probe has GRADUATED
        // from the pending vocabulary (the ratchet's recomputed counts
        // carry the advance; tier-1 cross-checks own its correctness).
        // Everything still failing everywhere must fail with the named,
        // tier-tagged error — never silence, never garbage.
        let mut errors = Vec::new();
        let mut graduated = false;
        for probe in &probes {
            let math = fmd_math::parse(probe);
            let text = fmd_math::parse_text(probe);
            if math.is_ok() || text.is_ok() {
                graduated = true;
                break;
            }
            errors.extend(math.err());
            errors.extend(text.err());
        }
        if graduated {
            audited += 1;
            continue;
        }
        let hit = errors.iter().find(|e| {
            e.unsupported_construct() == Some(construct)
                && e.to_string().contains("tier T2")
                && e.to_string().contains("fm-j5t")
        });
        assert!(
            hit.is_some(),
            "`{construct}` must fail with its named tier-T2 error in some mode \
             (probes {probes:?}; errors: {:?})",
            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        );
        audited += 1;
    }
    assert!(
        audited >= 25,
        "the T2 command audit covers the table ({audited})"
    );
    Ok(())
}

/// A minimal source string exercising a construct from the table.
fn probe_sources(construct: &str) -> Vec<String> {
    if let Some(env) = construct.strip_prefix("env:") {
        return vec![format!("\\begin{{{env}}} x \\end{{{env}}}")];
    }
    // A still-pending construct errors at lookup before any argument is
    // read, so the bare command captures its named error. A GRADUATED
    // construct may demand an argument — a letter for the general takers,
    // a number for `\\ding`'s pifont slot — so graduation registers as
    // success on ANY probe rather than a missing-argument refusal.
    vec![
        construct.to_owned(),
        format!("{construct}{{x}}"),
        format!("{construct}{{51}}"),
        // The text accents compose only over accentable bases.
        format!("{construct}{{e}}"),
    ]
}

// ── Corpus-bearing environments: recompute + ratchet + bless ────────────

const TRACK_T2: &str = "franken_manim fm-j5t";
const TRACK_EXT: &str = "franken_manim fm-kg9";
const TRACK_FONTS: &str = "franken_markdown br-…-4vjj (Noto math-alphanumeric subset)";

#[test]
fn recompute_and_enforce_the_ratchet() -> Result<(), String> {
    let corpus_path = std::env::var("FMN_TEX_CORPUS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_path("corpus/tex_corpus.jsonl"));
    let corpus_file = match File::open(&corpus_path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // The pin-coupling test still enforces re-runs at pin bumps.
            fmn_core::test_inputs::skip_or_fail(
                "fmn-conformance::recompute_and_enforce_the_ratchet",
                &format!("TeX corpus at {}", corpus_path.display()),
            );
            return Ok(());
        }
        Err(error) => {
            return Err(format!(
                "cannot read corpus at {}: {error}",
                corpus_path.display()
            ));
        }
    };
    let data = read_utf8_bounded(corpus_file, "coverage corpus", MAX_CORPUS_BYTES)?;
    let engine = fmd_math::Engine::bundled().map_err(|error| format!("bundled faces: {error}"))?;
    let committed = committed_baseline()?;
    let mut current = Baseline {
        rules_version: committed.rules_version,
        corpus_hash: committed.corpus_hash.clone(),
        franken_markdown_rev: suite_lock_franken_markdown_rev()?,
        unique_total: 0,
        occurrence_total: 0,
        parse_unique: 0,
        parse_occurrences: 0,
        layout_unique: 0,
        layout_occurrences: 0,
    };
    let mut parse_pending: BTreeMap<String, u64> = BTreeMap::new();
    let mut layout_pending: BTreeMap<String, u64> = BTreeMap::new();
    for (lineno, line) in data.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry = parse_corpus_entry(line)
            .ok_or_else(|| format!("corpus line {}: bad JSON", lineno + 1))?;
        current.unique_total += 1;
        current.occurrence_total += entry.count;
        let parse_result = if entry.mode == "text" {
            fmd_math::parse_text(&entry.text).map(|_| ())
        } else {
            fmd_math::parse(&entry.text).map(|_| ())
        };
        match parse_result {
            Ok(()) => {
                current.parse_unique += 1;
                current.parse_occurrences += entry.count;
            }
            Err(e) => {
                let name = e
                    .unsupported_construct()
                    .unwrap_or("(structural)")
                    .to_owned();
                *parse_pending.entry(name).or_insert(0) += entry.count;
                continue;
            }
        }
        let laid = if entry.mode == "text" {
            engine.typeset_text(&entry.text).map(|_| ())
        } else {
            engine
                .typeset(&entry.text, fmd_math::Style::Display)
                .map(|_| ())
        };
        match laid {
            Ok(()) => {
                current.layout_unique += 1;
                current.layout_occurrences += entry.count;
            }
            Err(fmd_math::MathError::UnmappedChar { ch, .. }) => {
                *layout_pending
                    .entry(format!("char:U+{:04X}", ch as u32))
                    .or_insert(0) += entry.count;
            }
            Err(e) => {
                let name = e
                    .unsupported_construct()
                    .unwrap_or("(structural)")
                    .to_owned();
                *layout_pending.entry(name).or_insert(0) += entry.count;
            }
        }
    }
    if committed.unique_total != 0 {
        // The zero-count form exists only as the pre-first-bless seed.
        assert_eq!(
            current.unique_total, committed.unique_total,
            "the denominator is frozen (G0-4)"
        );
        assert_eq!(current.occurrence_total, committed.occurrence_total);
    }
    let violations = ratchet_violations(&committed, &current);
    assert!(
        violations.is_empty(),
        "THE RATCHET: coverage regressed — a deliberate tier change updates \
         the baseline with a written note, an accident gets fixed:\n{}",
        violations.join("\n")
    );
    let advanced = current != committed;
    if std::env::var("RATCHET_UPDATE").is_ok() {
        // The dashboard publishes the TeX box oracle's counts beside the
        // coverage (tests/tex_corpus_boxes.rs checks they stay current).
        let texts: HashMap<String, String> = data
            .lines()
            .filter_map(parse_corpus_entry)
            .filter(|entry| entry.mode == "math")
            .map(|entry| (corpus_digest("math", &entry.text), entry.text))
            .collect();
        let rows = parse_oracle_fixture(ORACLE_FIXTURE)?;
        let laid = run_layout_oracle(&rows, &texts)?;
        let errors: Vec<f64> = rows
            .iter()
            .zip(&laid)
            .map(|(row, &laid)| oracle_error(row, laid))
            .collect();
        let oracle = summarize_oracle(&rows, &errors);
        bless(&current, &oracle, &parse_pending, &layout_pending)?;
        eprintln!("ratchet blessed at {}", current.franken_markdown_rev);
    } else if advanced {
        return Err(
            "coverage advanced (or the pin moved) — bless deliberately with \
             RATCHET_UPDATE=1 so the public artifacts move in the same commit"
                .to_owned(),
        );
    }
    let [po, pu, lo, lu] = current.percentages();
    eprintln!("ratchet: parse {po:.3}%/{pu:.3}% · layout {lo:.3}%/{lu:.3}%");
    Ok(())
}

fn bless(
    current: &Baseline,
    oracle: &OracleSummary,
    parse_pending: &BTreeMap<String, u64>,
    layout_pending: &BTreeMap<String, u64>,
) -> Result<(), String> {
    let dir = repo_path("docs/ratchet");
    std::fs::create_dir_all(&dir).map_err(|error| format!("mkdir: {error}"))?;
    // Trend: append (or start) — keyed by rev; re-blessing the same rev
    // replaces its row.
    let trend_path = dir.join("trend.tsv");
    let mut trend = match File::open(&trend_path) {
        Ok(file) => parse_trend_tsv(&read_utf8_bounded(
            file,
            "existing coverage trend",
            MAX_TREND_BYTES,
        )?)
        .map_err(|error| format!("existing trend is malformed: {error}"))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(format!("cannot read existing trend: {error}")),
    };
    trend.retain(|b| b.franken_markdown_rev != current.franken_markdown_rev);
    trend.push(current.clone());
    let mut trend_out = String::from(
        "# ratchet trend (fm-mol): one row per bless, keyed by franken_markdown rev\n\
         # rev\trules\thash\tuniq_total\tocc_total\tparse_uniq\tparse_occ\tlayout_uniq\tlayout_occ\n",
    );
    for b in &trend {
        trend_out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            b.franken_markdown_rev,
            b.rules_version,
            b.corpus_hash,
            b.unique_total,
            b.occurrence_total,
            b.parse_unique,
            b.parse_occurrences,
            b.layout_unique,
            b.layout_occurrences,
        ));
    }
    std::fs::write(&trend_path, trend_out).map_err(|error| format!("trend: {error}"))?;
    std::fs::write(dir.join("baseline.tsv"), current.to_tsv())
        .map_err(|error| format!("baseline: {error}"))?;
    let to_pending = |m: &BTreeMap<String, u64>| -> Vec<Pending> {
        let mut v: Vec<Pending> = m
            .iter()
            .map(|(construct, occ)| Pending {
                construct: construct.clone(),
                occurrences: *occ,
                tracked: track_of(construct).to_owned(),
            })
            .collect();
        v.sort_by(|a, b| {
            b.occurrences
                .cmp(&a.occurrences)
                .then(a.construct.cmp(&b.construct))
        });
        v
    };
    let dashboard = render_dashboard(
        current,
        oracle,
        &to_pending(parse_pending),
        &to_pending(layout_pending),
        &trend,
    );
    std::fs::write(dir.join("dashboard.md"), dashboard)
        .map_err(|error| format!("dashboard: {error}"))?;
    Ok(())
}

#[test]
fn authority_reader_checks_the_limit_before_text_validation() {
    let error = read_utf8_bounded(
        std::io::Cursor::new(vec![0xff_u8; 9]),
        "fixture authority",
        8,
    )
    .expect_err("limit-plus-one input must be refused");
    assert_eq!(error, "fixture authority exceeds the 8-byte resource limit");
}

fn track_of(construct: &str) -> &'static str {
    if construct.starts_with("char:") {
        TRACK_FONTS
    } else if construct.starts_with("env:")
        || matches!(
            construct,
            "\\overbrace"
                | "\\underbrace"
                | "\\overrightarrow"
                | "\\overleftarrow"
                | "\\widehat"
                | "\\widetilde"
        )
    {
        TRACK_EXT
    } else {
        TRACK_T2
    }
}
