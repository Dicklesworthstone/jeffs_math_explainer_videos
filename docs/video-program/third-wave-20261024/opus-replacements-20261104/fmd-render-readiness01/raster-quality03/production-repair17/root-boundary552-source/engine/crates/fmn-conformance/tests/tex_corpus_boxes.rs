//! Corpus-wide TeX box oracle for fmd-math layout (fm-tex-layout-oracle-bkbc).
//!
//! `fixtures/tex_corpus_boxes.v1.tsv` holds real TeX's box (width, height and
//! depth in ems, display style, under the pinned Reference's preamble) for a
//! stratified sample of the private TeX corpus. Each string is named only by
//! its public digest (plan §15.3); `scripts/capture_tex_corpus_boxes.py`
//! captured them. The evaluation lives in `fmn_conformance::ratchet`, which
//! the coverage ratchet's dashboard shares.
//!
//! Where the corpus is present (`FMN_TEX_CORPUS`, or `corpus/tex_corpus.jsonl`),
//! each sampled string is laid out natively. A string agrees when its width,
//! and its height and depth relative to the total, are within the tolerance.
//! The counts within 5% and 10% may only rise, and `docs/ratchet/dashboard.md`
//! must publish the measured counts. The run logs bounded NDJSON (schema
//! `fmn.tex-corpus-boxes.v1`): a record per sampled row with TeX's and the
//! native box, a record per construct class with the worst first, and a
//! summary with the decision. Without the corpus, only the fixture's shape is
//! checked.
#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic, clippy::print_stderr)]

use fmn_conformance::ratchet::{
    ORACLE_FIXTURE, OracleRow, corpus_digest, oracle_dashboard_line, oracle_error, oracle_ndjson,
    parse_corpus_entry, parse_oracle_fixture, run_layout_oracle, summarize_oracle,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Sampled strings within 5% of TeX's box: 188 of 563 when this oracle was
/// first measured (franken_markdown a8aab0d), 189 once the Tex surface's rows
/// opened up by \jot (d664c13), 197 of 585 once the capture set strings as
/// the Reference's align* does (stripped, braced, multi-line rows on their
/// first baseline), 233 once fractions carried TeX's null delimiters and
/// \vdots, \ddots and trailing `\\` rows were TeX's (e4e8e2e), 235 once the
/// Tex surface's rows were align* rows (798de11), 252 with TeX's accent
/// geometry, \big sizes and array struts (bd00a24), 279 with TeX's
/// \scriptspace, cmsy's prime character and LaTeX's brace fills (3ed1d33).
/// Raise it when layout improves; never lower it to land a change.
const WITHIN_5_PERCENT_FLOOR: usize = 279;
/// The same at 10%: 315, 316, 330, 365, 367, 385, then 421.
const WITHIN_10_PERCENT_FLOOR: usize = 421;

fn rows() -> Vec<OracleRow> {
    parse_oracle_fixture(ORACLE_FIXTURE).expect("the oracle fixture parses")
}

#[test]
fn fixture_names_corpus_strings_only_by_digest() {
    let rows = rows();
    assert!(
        rows.len() >= 500,
        "the sample thinned out: {} rows",
        rows.len()
    );
    let mut seen = HashSet::new();
    for row in &rows {
        assert!(
            row.digest.len() == 64
                && row
                    .digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "not a lowercase sha256: {}",
            row.digest
        );
        assert!(seen.insert(&row.digest), "duplicate row {}", row.digest);
        assert!(row.width > 0.0 && row.height + row.depth > 0.0 && row.count > 0);
        assert!(row.height >= 0.0 && row.depth >= 0.0);
        assert!(!row.classes.is_empty() && row.classes.iter().all(|c| !c.is_empty()));
    }
}

#[test]
fn corpus_boxes_track_tex_and_never_regress() {
    let path = std::env::var("FMN_TEX_CORPUS").map_or_else(
        |_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/tex_corpus.jsonl"),
        PathBuf::from,
    );
    let Ok(corpus) = std::fs::read_to_string(&path) else {
        fmn_core::test_inputs::skip_or_fail(
            "fmn-conformance::corpus_boxes_track_tex_and_never_regress",
            &format!("TeX corpus at {}", path.display()),
        );
        return;
    };
    let texts: HashMap<String, String> = corpus
        .lines()
        .filter_map(parse_corpus_entry)
        .filter(|entry| entry.mode == "math")
        .map(|entry| (corpus_digest("math", &entry.text), entry.text))
        .collect();
    let rows = rows();
    let laid = run_layout_oracle(&rows, &texts).expect("every sampled string is in the corpus");
    let errors: Vec<f64> = rows
        .iter()
        .zip(&laid)
        .map(|(row, &laid)| oracle_error(row, laid))
        .collect();
    eprint!(
        "{}",
        oracle_ndjson(
            &rows,
            &laid,
            &errors,
            (WITHIN_5_PERCENT_FLOOR, WITHIN_10_PERCENT_FLOOR)
        )
    );
    let summary = summarize_oracle(&rows, &errors);
    assert!(
        summary.within_5 >= WITHIN_5_PERCENT_FLOOR,
        "{} sampled strings within 5% of TeX, below the floor {WITHIN_5_PERCENT_FLOOR}",
        summary.within_5
    );
    assert!(
        summary.within_10 >= WITHIN_10_PERCENT_FLOOR,
        "{} sampled strings within 10% of TeX, below the floor {WITHIN_10_PERCENT_FLOOR}",
        summary.within_10
    );
    let dashboard = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/ratchet/dashboard.md"),
    )
    .expect("the ratchet dashboard");
    assert!(
        dashboard.contains(&oracle_dashboard_line(&summary)),
        "docs/ratchet/dashboard.md does not publish the oracle's counts \
         ({summary:?}); re-bless it with RATCHET_UPDATE=1 cargo test -p \
         fmn-conformance --test coverage_ratchet"
    );
}
