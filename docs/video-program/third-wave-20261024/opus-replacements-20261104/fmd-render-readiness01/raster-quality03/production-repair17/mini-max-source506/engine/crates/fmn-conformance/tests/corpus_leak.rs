//! The no-corpus-leak release gate (fm-aef acceptance 4, §15.3): the
//! distributable surface — the git-tracked set plus `dist/` and the
//! wheel/npm staging trees — must carry no CC BY-NC-SA private fixture:
//! no harvested corpus string, no Reference-capture bytes, no path under a
//! private-fixture directory, and `.gitignore` must actually exclude those
//! directories. The negative tests (planted leaks) prove the teeth bite;
//! the gate test proves the real tree is clean. Any finding fails LOUDLY.

use fmn_conformance::corpus_leak::{
    DENOMINATOR_PATH, PRIVATE_FIXTURE_DIRS, content_leaks, corpus_hash, git_tracked_files,
    gitignore_violations, parse_denominator, path_violations, run,
};
use fmn_hash::Digest;
use std::collections::HashSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn release_surface_carries_no_private_fixtures() {
    let root = repo_root();
    let leaks = run(&root).unwrap_or_else(|e| {
        std::panic::panic_any(format!("the corpus-leak gate could not run: {e}"))
    });
    if !leaks.is_empty() {
        let report = leaks
            .iter()
            .map(|l| format!("  - {l}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::panic::panic_any(format!(
            "no-corpus-leak gate FAILED: {} finding(s) on the distributable surface (§15.3 — \
             CC BY-NC-SA fixtures never ship):\n{report}",
            leaks.len()
        ));
    }
}

#[test]
fn private_fixture_dirs_are_actually_gitignored() {
    let root = repo_root();
    let leaks = gitignore_violations(&root)
        .unwrap_or_else(|e| std::panic::panic_any(format!("git check-ignore unavailable: {e}")));
    assert!(
        leaks.is_empty(),
        ".gitignore must exclude every private-fixture directory: {leaks:?}"
    );
}

#[test]
fn git_tracked_set_carries_no_private_paths() {
    let root = repo_root();
    let tracked = git_tracked_files(&root)
        .unwrap_or_else(|e| std::panic::panic_any(format!("git ls-files unavailable: {e}")));
    assert!(
        !tracked.is_empty(),
        "git ls-files returned no files — the gate requires a real checkout"
    );
    let leaks = path_violations(&tracked);
    assert!(
        leaks.is_empty(),
        "private fixtures are tracked by git: {leaks:?}"
    );
}

#[test]
fn denominator_is_whole_and_well_formed() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(DENOMINATOR_PATH))
        .unwrap_or_else(|e| std::panic::panic_any(format!("reading {DENOMINATOR_PATH}: {e}")));
    let digests =
        parse_denominator(&text).unwrap_or_else(|e| std::panic::panic_any(format!("{e}")));
    assert!(
        !digests.is_empty(),
        "the denominator carries no corpus hashes"
    );
    let distinct: HashSet<Digest> = digests.iter().copied().collect();
    assert_eq!(
        distinct.len(),
        digests.len(),
        "the denominator repeats a corpus hash — the leak oracle must be whole and distinct"
    );
    // Sanity: the oracle covers the documented harvest scale (G0-4:
    // thousands of distinct strings), not a token sample.
    assert!(
        digests.len() > 1_000,
        "the denominator holds only {} hashes — the content tooth would be near-blind",
        digests.len()
    );
}

#[test]
fn a_planted_corpus_string_is_caught() {
    // Negative proof against the REAL denominator: pick a real corpus
    // hash, then assert a file line whose bytes hash to it is flagged.
    // The preimage is unavailable by design (hashes ship, strings don't),
    // so construct the planted leak from the convention itself: the
    // synthetic string is admitted into a synthetic corpus set, and the
    // content tooth must flag it in each realistic embedding form.
    let root = repo_root();
    let text = std::fs::read_to_string(root.join(DENOMINATOR_PATH))
        .unwrap_or_else(|e| std::panic::panic_any(format!("reading {DENOMINATOR_PATH}: {e}")));
    let real = parse_denominator(&text).unwrap_or_else(|e| std::panic::panic_any(format!("{e}")));
    // A non-corpus line must NOT collide with any real denominator hash
    // (guards the tooth against false-positive constructs).
    let real_set: HashSet<Digest> = real.into_iter().collect();
    let empty = HashSet::new();
    assert!(
        content_leaks("clean.rs", b"let x = Mobject::new();\n", &real_set, &empty).is_empty(),
        "ordinary engine source must never trip the corpus tooth"
    );

    let corpus: HashSet<Digest> = [corpus_hash("text", b"Some private Reference string")]
        .into_iter()
        .collect();
    let planted = b"# fixtures\nSome private Reference string\n";
    let leaks = content_leaks(
        "crates/fmn-conformance/fixtures/planted.txt",
        planted,
        &corpus,
        &empty,
    );
    assert_eq!(
        leaks.len(),
        1,
        "a planted corpus string must be flagged: {leaks:?}"
    );
}

#[test]
fn private_dir_prefixes_are_exactly_the_policy_set() {
    // The gate's private-dir list is the §15.3 policy set — assert it
    // names the four governed locations so a fifth private tree can't
    // appear without this test being re-read.
    assert_eq!(
        PRIVATE_FIXTURE_DIRS,
        &[
            "corpus/",
            "gallery/reference_captures/",
            "scripts/manim_ref/",
            "scripts/videos_ref/"
        ]
    );
}

#[test]
fn planted_reference_asset_basenames_are_caught() {
    // ADR-0020's basename tooth: every realistic leak form of the
    // Reference-derived drawings assets trips, wherever it hides.
    let leaks = path_violations(&[
        "assets/lightbulb.svg".to_owned(),
        "npm/pkg/Bubbles_speech.svg".to_owned(),
        "dist/earth".to_owned(),
        "wasm-smoke/pkg/VIDEO_ICON.SVG".to_owned(),
    ]);
    assert_eq!(
        leaks.len(),
        4,
        "every drawings-asset basename form must trip: {leaks:?}"
    );
    assert!(
        leaks.iter().all(|leak| leak.detail.contains("ADR-0020")),
        "each finding must name its policy: {leaks:?}"
    );

    // Names that merely share a substring or stem must not trip.
    let clean = path_violations(&[
        "docs/adr/0020-reference-drawings-assets-stay-unbundled.md".to_owned(),
        "crates/fmn-library/src/drawings.rs".to_owned(),
    ]);
    assert!(
        clean.is_empty(),
        "ordinary source must never trip the asset tooth: {clean:?}"
    );
}
