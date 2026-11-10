//! The one API schema and its generated artifacts (fm-vn6, §16.2, §16.1).
//!
//! The schema lives in two committed files at the repo root — `API_SCHEMA.tsv`
//! (extracted from the pinned Reference by `scripts/gen_api_schema.py`) and
//! `API_OVERLAY.tsv` (authored). Everything generated from them is also
//! committed, and this suite is the reason that is safe: each artifact is
//! regenerated in memory and compared byte-for-byte with the committed file.
//!
//! **This is the "drift between the front doors is a build error" mechanism.**
//! Edit a generated signature by hand and the comparison fails; add a config
//! key without binding it and the coverage check fails; let the overlay go
//! stale against a re-extracted Reference and the merge fails naming the
//! dangling key.
//!
//! Regenerate deliberately with:
//!
//! ```text
//! UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance --test api_schema
//! ```
//!
//! which rewrites the artifacts in the working tree and never commits them —
//! the same bless discipline as `UPDATE_GOLDENS=1` and `RATCHET_UPDATE=1`.

#![allow(clippy::expect_used)]

use std::io::Read;
use std::path::{Path, PathBuf};

use fmn_conformance::schema::{
    Schema, Status, SymbolKind, generate_cli_rs, generate_cli_table_md, generate_config_rs,
    generate_coverage_badge_svg, generate_docs_md, generate_ledger_tsv, generate_out_of_tier_tsv,
};

/// File-read envelope shared by `API_SCHEMA.tsv`, `API_OVERLAY.tsv`, and
/// `SUITE.lock`. The first two are parsed under the schema module's matching
/// 8 MiB document limit; the much smaller lock rides the same authority cap.
const MAX_SCHEMA_AUTHORITY_BYTES: usize = 8 * 1024 * 1024;

/// Repo-root-relative path (`CARGO_MANIFEST_DIR` is `crates/fmn-conformance`).
fn repo_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name)
}

fn read_utf8_with_limit(reader: impl Read, max_bytes: usize, name: &str) -> Result<String, String> {
    let read_limit = max_bytes
        .checked_add(1)
        .ok_or_else(|| format!("{name} byte limit cannot be represented"))?;
    let read_limit = u64::try_from(read_limit)
        .map_err(|_| format!("{name} byte limit does not fit the reader"))?;
    let mut bytes = Vec::new();
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("reading {name}: {error}"))?;
    if bytes.len() > max_bytes {
        return Err(format!("{name} exceeds the {max_bytes}-byte limit"));
    }
    String::from_utf8(bytes).map_err(|error| format!("{name} is not UTF-8: {error}"))
}

fn read_bounded_utf8(path: &Path, max_bytes: usize, name: &str) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {name}: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("{name} is not a regular file"));
    }
    let max_bytes_u64 =
        u64::try_from(max_bytes).map_err(|_| format!("{name} byte limit does not fit metadata"))?;
    if metadata.len() > max_bytes_u64 {
        return Err(format!(
            "{name} is {} bytes, exceeding the {max_bytes}-byte limit",
            metadata.len()
        ));
    }
    let file = std::fs::File::open(path).map_err(|error| format!("opening {name}: {error}"))?;
    read_utf8_with_limit(file, max_bytes, name)
}

fn repo_file(name: &str) -> String {
    read_bounded_utf8(&repo_path(name), MAX_SCHEMA_AUTHORITY_BYTES, name)
        .expect("repository authority must be a bounded UTF-8 regular file")
}

fn schema() -> Schema {
    Schema::parse(&repo_file("API_SCHEMA.tsv"), &repo_file("API_OVERLAY.tsv"))
        .expect("the canonical API schema and overlay must parse")
}

/// Whether this run rewrites artifacts instead of checking them.
fn blessing() -> bool {
    std::env::var("UPDATE_API_ARTIFACTS").is_ok_and(|v| v == "1")
}

/// Compare a generated artifact with its committed copy, or rewrite it.
#[track_caller]
fn artifact(name: &str, generated: &str) {
    let path = repo_path(name);
    if blessing() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("creating parent for {name}: {error}"))
                .expect("generated artifact parent must be creatable");
        }
        std::fs::write(&path, generated)
            .map_err(|error| format!("writing {name}: {error}"))
            .expect("generated artifact must be writable");
        return;
    }
    let committed = read_bounded_utf8(&path, generated.len(), name)
        .map_err(|error| {
            format!(
                "{error}. Generate the artifact with:\n  \
                 UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance --test api_schema"
            )
        })
        .expect("committed generated artifact must be bounded UTF-8");
    if committed == generated {
        return;
    }
    let (line, before, after) = first_difference(&committed, generated);
    assert!(
        committed == generated,
        "{name} has drifted from the schema at line {line}.\n\
         \x20 committed:  {before}\n\
         \x20 schema says: {after}\n\n\
         The schema is the source of truth: either the hand edit belongs in \
         API_SCHEMA.tsv/API_OVERLAY.tsv, or the artifact needs regenerating with\n  \
         UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance --test api_schema"
    );
}

/// First differing line, 1-based, with both sides — a diff a human can act on
/// without reading two thousand rows.
fn first_difference(a: &str, b: &str) -> (usize, String, String) {
    for (index, (left, right)) in a.lines().zip(b.lines()).enumerate() {
        if left != right {
            return (index + 1, left.to_owned(), right.to_owned());
        }
    }
    let shared = a.lines().count().min(b.lines().count());
    let tail = |s: &str| s.lines().nth(shared).unwrap_or("<end of file>").to_owned();
    (shared + 1, tail(a), tail(b))
}

#[test]
fn bounded_utf8_reader_accepts_the_limit_and_checks_size_before_encoding() {
    const LIMIT: usize = 8;

    let exact = read_utf8_with_limit(
        std::io::repeat(b'a').take(u64::try_from(LIMIT).expect("small test limit fits u64")),
        LIMIT,
        "test authority",
    )
    .expect("the exact byte limit must be accepted");
    assert_eq!(exact, "a".repeat(LIMIT));

    let mut oversized_invalid = vec![b'a'; LIMIT];
    oversized_invalid.push(0xff);
    let error = read_utf8_with_limit(&oversized_invalid[..], LIMIT, "test authority")
        .expect_err("one byte beyond the limit must be refused");
    assert_eq!(error, "test authority exceeds the 8-byte limit");

    let error = read_utf8_with_limit(&[0xff][..], LIMIT, "test authority")
        .expect_err("invalid UTF-8 within the envelope must be named");
    assert!(error.contains("is not UTF-8"), "wrong UTF-8 error: {error}");
}

#[test]
fn bounded_utf8_file_preflights_exact_size_oversize_and_file_type() {
    let path = repo_path("Cargo.toml");
    let size = usize::try_from(
        std::fs::metadata(&path)
            .expect("workspace manifest metadata")
            .len(),
    )
    .expect("workspace manifest length fits usize");
    let text = read_bounded_utf8(&path, size, "workspace manifest")
        .expect("a regular UTF-8 file at the exact limit must be accepted");
    assert_eq!(text.len(), size);

    let error = read_bounded_utf8(
        &path,
        size.checked_sub(1)
            .expect("workspace manifest is not empty"),
        "workspace manifest",
    )
    .expect_err("the metadata preflight must refuse an oversized file");
    assert!(error.contains("exceeding"), "wrong size error: {error}");

    let error = read_bounded_utf8(&repo_path("."), size, "workspace root")
        .expect_err("a directory must not be consumed as an authority");
    assert_eq!(error, "workspace root is not a regular file");
}

// ---------------------------------------------------------------------------
// The schema itself
// ---------------------------------------------------------------------------

#[test]
fn the_schema_parses_and_names_the_pinned_reference() {
    let schema = schema();
    let suite = repo_file("SUITE.lock");
    let pinned = suite
        .lines()
        .find_map(|l| l.strip_prefix("3b1b/manim\t"))
        .and_then(|rest| rest.split('\t').next())
        .expect("SUITE.lock [reference] pins 3b1b/manim");
    assert_eq!(
        schema.reference_commit(),
        pinned,
        "API_SCHEMA.tsv was extracted from a different Reference commit than \
         SUITE.lock pins — rerun scripts/gen_api_schema.py"
    );
}

#[test]
fn native_error_categories_match_the_generated_cli_schema() {
    use fmn::ErrorKind;

    for kind in [
        ErrorKind::Config,
        ErrorKind::Capability,
        ErrorKind::Scene,
        ErrorKind::Render,
        ErrorKind::Budget,
    ] {
        let generated = fmn_cli::EXIT_CODE_SPECS
            .iter()
            .find(|spec| spec.name == kind.name())
            .expect("every native error category has a generated CLI schema row");
        assert_eq!(
            generated.code,
            kind.code(),
            "native `{}` category drifted from API_OVERLAY.tsv",
            kind.name()
        );
    }
}

#[test]
fn the_class_census_matches_the_plans_verified_count() {
    // Appendix A: "the complete Reference census (257 classes, verified
    // class-by-class against the pin)". The schema is extracted independently
    // of that audit, so agreement is a real cross-check on both — and a
    // Reference pin bump that changes the class count will say so here.
    let schema = schema();
    assert_eq!(
        schema.of_kind(SymbolKind::Class).len(),
        257,
        "the extracted class count no longer matches Appendix A's verified census"
    );
}

#[test]
fn the_wildcard_surface_is_enumerated_not_assumed() {
    // §1.6: `from manimlib import *` has no authoritative `__all__`, so the
    // export set is a computed closure. The extractor records its own count in
    // [meta]; this asserts the rows agree with it, which catches an extractor
    // that walked the closure one way and emitted rows another.
    let schema = schema();
    let declared: usize = schema
        .meta
        .get("wildcard_exports")
        .expect("[meta] wildcard_exports")
        .parse()
        .expect("a count");
    let exported = schema.exported();
    let names: std::collections::BTreeSet<&str> =
        exported.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names.len(),
        declared,
        "the exported rows and [meta] wildcard_exports disagree"
    );
    assert_eq!(declared, 663, "the installed-wheel contract drifted");
    assert_eq!(
        exported.len(),
        665,
        "the wildcard-exported schema-row inventory drifted"
    );

    let mut name_counts = std::collections::BTreeMap::new();
    for symbol in exported {
        *name_counts.entry(symbol.name.as_str()).or_insert(0_usize) += 1;
    }
    let duplicates: Vec<(&str, usize)> = name_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();
    assert_eq!(
        duplicates,
        vec![("DEFAULT_DOT_RADIUS", 2), ("EPSILON", 2)],
        "duplicate wildcard rows changed; update the generated explanation"
    );

    let docs = generate_docs_md(&schema);
    assert!(
        docs.contains(
            "`from manimlib import *` binds 663 unique names from 665 wildcard-exported schema rows."
        ),
        "generated docs must distinguish unique bindings from schema rows"
    );
    assert!(
        docs.contains("The 2 duplicate rows are `DEFAULT_DOT_RADIUS`, `EPSILON`."),
        "generated docs must identify the duplicate names"
    );

    // The closure genuinely leaks third-party names — that is the finding, not
    // a bug, and the Ledger has to carry them.
    assert!(
        names.contains("np"),
        "`from manimlib import *` really does bind numpy as `np`; losing that \
         row means the extractor stopped modelling CPython's rules"
    );
    // ...while type-checking-only imports must NOT be in it.
    assert!(
        !names.contains("Vect3"),
        "`Vect3` is imported under `if TYPE_CHECKING:` and is not bound at \
         runtime; the extractor must skip those bodies"
    );
}

#[test]
fn every_class_of_the_census_carries_its_constructor_parameters() {
    let schema = schema();
    let classes: Vec<&str> = schema
        .of_kind(SymbolKind::Class)
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    // A class that declares `__init__` must have its parameters recorded;
    // otherwise the Python front door has nothing to reproduce.
    let inits: std::collections::BTreeSet<&str> = schema
        .of_kind(SymbolKind::Method)
        .iter()
        .filter(|s| s.name.ends_with(".__init__"))
        .map(|s| s.name.trim_end_matches(".__init__"))
        .collect();
    for class in &inits {
        assert!(
            classes.contains(class),
            "`{class}.__init__` has no class row"
        );
        let owner_suffix = format!(":{class}.__init__");
        assert!(
            schema
                .params
                .iter()
                .any(|p| p.owner.ends_with(&owner_suffix)),
            "`{class}.__init__` has no recorded parameters"
        );
    }
    assert!(inits.len() > 100, "suspiciously few constructors recorded");
}

#[test]
fn defaults_are_recorded_as_source_expressions() {
    // `Arc(angle=TAU / 4, arc_center=ORIGIN)`: resolving those would need the
    // import the extractor deliberately refuses, and the expression text is
    // what the ledger and the Python signatures both want.
    let schema = schema();
    let arc: Vec<(&str, Option<&str>)> = schema
        .params
        .iter()
        .filter(|p| p.owner == "manimlib.mobject.geometry:Arc.__init__")
        .map(|p| (p.name.as_str(), p.default.as_deref()))
        .collect();
    assert!(
        arc.contains(&("angle", Some("TAU / 4"))),
        "Arc.angle default lost its expression: {arc:?}"
    );
    assert!(
        arc.contains(&("arc_center", Some("ORIGIN"))),
        "Arc.arc_center default lost its expression: {arc:?}"
    );
}

// ---------------------------------------------------------------------------
// C-9 — canonical names, exact-name aliases
// ---------------------------------------------------------------------------

#[test]
fn every_public_surface_typo_has_a_canonical_name() {
    // Appendix C, C-9. The plan names three examples; the extracted surface
    // shows the real extent. Whatever the extractor finds must be ruled on —
    // a partial rename would be worse than none, because callers could not
    // predict which spelling any given member takes.
    let schema = schema();
    let misspellings = ["listner", "cahced", "tickness"];
    let mut unruled = Vec::new();
    for symbol in &schema.symbols {
        let leaf = symbol.name.rsplit('.').next().unwrap_or(&symbol.name);
        if misspellings.iter().any(|m| leaf.contains(m))
            && !schema.renames.contains_key(&symbol.key())
        {
            unruled.push(symbol.key());
        }
    }
    assert!(
        unruled.is_empty(),
        "these public-surface typos have no canonical name in API_OVERLAY.tsv \
         [canonical]: {unruled:?}"
    );
    assert!(
        schema.renames.len() >= 26,
        "C-9 covers 26 symbols at this pin, found {}",
        schema.renames.len()
    );
}

#[test]
fn a_canonical_name_fixes_the_typo_and_keeps_the_owner() {
    let schema = schema();
    let listener = schema
        .symbols
        .iter()
        .find(|s| s.name == "Mobject.add_event_listner")
        .expect("the Reference's Mobject.add_event_listner");
    assert_eq!(
        schema.canonical_name(listener),
        "Mobject.add_event_listener"
    );

    let attribute = schema
        .symbols
        .iter()
        .find(|s| s.name == "Arrow.tickness_multiplier")
        .expect("the Reference's Arrow.tickness_multiplier");
    assert_eq!(
        schema.canonical_name(attribute),
        "Arrow.thickness_multiplier"
    );

    // Every ruling must actually change the spelling, and must only fix the
    // typo — a "canonical" name identical to the Reference's, or one that
    // renamed the concept, is a mistake in the overlay.
    for rename in schema.renames.values() {
        let leaf = rename
            .symbol
            .rsplit(['.', ':'])
            .next()
            .expect("a leaf name");
        assert_ne!(
            leaf, rename.canonical,
            "{} is listed as a rename but does not change the name",
            rename.symbol
        );
        // C-9 fixes spelling, it does not rename concepts. Every ruling must
        // be within two edits of the Reference's own name — enough for a
        // dropped letter (`listner`), a transposition (`cahced`), or a
        // doubled fix, and not enough to smuggle in an API redesign under a
        // typo ruling.
        let distance = edit_distance(leaf, &rename.canonical);
        assert!(
            distance <= 2,
            "{leaf} -> {} is {distance} edits apart; that is a rename, not a \
             typo fix, and needs its own ruling rather than a C-9 row",
            rename.canonical
        );
    }
}

/// Levenshtein distance, for holding C-9 rulings to actual typo repairs.
fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[test]
fn the_reference_spelling_survives_as_the_python_alias() {
    // The alias is not a deprecation shim: it is the Reference's public
    // surface, and AGENTS.md's no-shims rule names the manim API as its
    // standing exception. fmn-python binds both names; nothing here may
    // suggest the old one goes away.
    let schema = schema();
    for rename in schema.renames.values() {
        let leaf = rename.symbol.rsplit(['.', ':']).next().expect("a leaf");
        assert!(
            schema
                .symbols
                .iter()
                .any(|s| s.key() == rename.symbol && s.name.ends_with(leaf)),
            "the Reference spelling `{leaf}` must remain in the schema as the alias"
        );
    }
}

// ---------------------------------------------------------------------------
// Config keys
// ---------------------------------------------------------------------------

#[test]
fn every_config_key_is_bound_exactly_once() {
    schema()
        .check_config_coverage()
        .expect("every config key must have exactly one authored binding");
}

#[test]
fn the_shipped_config_covers_the_references_key_surface() {
    // The Reference's own default_config.yml is the parity check, not the
    // source: we ship its key shapes so existing custom_config.yml files
    // overlay onto familiar names, plus native sections it never had.
    let schema = schema();
    let shared = schema.config.iter().filter(|c| c.in_reference).count();
    assert!(
        shared > 90,
        "only {shared} keys are shared with the Reference — the shipped \
         defaults have drifted off its key surface"
    );
    for native in ["determinism.mode", "determinism.seed", "render.engine"] {
        let key = schema
            .config
            .iter()
            .find(|c| c.path == native)
            .expect("the native config-key census must be complete");
        assert!(
            !key.in_reference,
            "`{native}` is FrankenManim-native and must not claim Reference provenance"
        );
    }
}

// ---------------------------------------------------------------------------
// The generated artifacts — the drift gate
// ---------------------------------------------------------------------------

#[test]
fn the_generated_config_extraction_matches_the_schema() {
    artifact(
        "crates/fmn-config/src/generated.rs",
        &generate_config_rs(&schema()),
    );
}

#[test]
fn the_generated_ledger_matches_the_schema() {
    artifact("docs/api/ledger.tsv", &generate_ledger_tsv(&schema()));
}

#[test]
fn the_generated_out_of_tier_ledger_matches_the_schema() {
    artifact(
        "docs/api/out_of_tier.tsv",
        &generate_out_of_tier_tsv(&schema()),
    );
}

#[test]
fn the_generated_coverage_badge_matches_the_schema() {
    artifact(
        "docs/api/parity_coverage.svg",
        &generate_coverage_badge_svg(&schema()),
    );
}

#[test]
fn the_generated_cli_table_matches_the_schema() {
    artifact("docs/api/cli_flags.md", &generate_cli_table_md(&schema()));
}

#[test]
fn the_generated_cli_parser_contract_matches_the_schema() {
    artifact(
        "crates/fmn-cli/src/generated.rs",
        &generate_cli_rs(&schema()),
    );
}

#[test]
fn the_generated_docs_match_the_schema() {
    artifact("docs/api/schema.md", &generate_docs_md(&schema()));
}

#[test]
fn a_hand_edited_signature_fails_the_drift_gate() {
    // The negative test the acceptance criteria ask for: mutate a generated
    // binding and prove the comparison catches it. Generated in memory rather
    // than read from the tree, so it neither races the blessing run nor
    // depends on the artifact already existing.
    let generated = generate_config_rs(&schema());
    let mutated = generated.replace(
        "fps: cx.u32(\"camera.fps\")?,",
        "fps: cx.u64(\"camera.fps\")?,",
    );
    assert_ne!(
        generated, mutated,
        "the fixture line vanished from the generated extraction; update this test"
    );
    let (line, before, after) = first_difference(&mutated, &generated);
    assert!(line > 0, "a mutated artifact must differ somewhere");
    assert!(
        before.contains("u64") && after.contains("u32"),
        "the drift report must name both sides: {before} / {after}"
    );

    // And the same for a hand-edited *config key*: the accessor is only half
    // the contract, the path is the other half.
    let repathed = generated.replace("\"camera.fps\"", "\"camera.framerate\"");
    let (_, before, after) = first_difference(&repathed, &generated);
    assert!(
        before.contains("camera.framerate") && after.contains("camera.fps"),
        "{before} / {after}"
    );
}

#[test]
fn the_ledger_carries_every_surface_and_its_review_contract() {
    let schema = schema();
    let ledger = generate_ledger_tsv(&schema);
    let rows = ledger
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        schema.symbols.len()
            + schema.flags.len()
            + schema.native_flags.len()
            + schema.subcommands.len()
            + schema.config.len(),
        "the Ledger must carry every Python, CLI, and config surface"
    );
    assert!(
        rows.iter().all(|row| row.len() == 10),
        "every Ledger row must carry signature/defaults, status, evidence, tests, and notes"
    );
    for kind in ["cli_flag", "cli_command", "config_key"] {
        assert!(
            rows.iter().any(|row| row[3] == kind),
            "the Ledger lost its {kind} surface"
        );
    }
    assert!(
        ledger.contains("\timproved\tBN-11"),
        "the seeded BN-11 rulings must reach the Ledger"
    );

    let note_ids = repo_file("docs/behavior_notes/README.md")
        .lines()
        .filter_map(|line| {
            let first = line.strip_prefix('|')?.split('|').next()?.trim();
            first.starts_with("BN-").then(|| first.to_owned())
        })
        .collect::<std::collections::BTreeSet<_>>();
    for row in &rows {
        let status = row[6];
        let evidence = row[7];
        let tests = row[8];
        let notes = row[9];
        if status == Status::Improved.as_str() {
            assert!(
                note_ids.contains(evidence),
                "improved {}:{} cites missing Behavior Note {evidence}",
                row[0],
                row[1]
            );
        }
        if matches!(status, "tiered" | "excluded") {
            let ruling = schema.out_of_tier.get(evidence);
            assert!(
                ruling.is_some(),
                "{status} {}:{} cites missing out-of-tier row {evidence}",
                row[0],
                row[1]
            );
            if let Some(ruling) = ruling {
                assert_eq!(ruling.status.as_str(), status);
            }
        }
        if status == Status::Unreviewed.as_str() {
            assert_eq!((evidence, tests, notes), ("-", "-", "-"));
        } else {
            assert_ne!(tests, "-", "reviewed {}:{} lost tests", row[0], row[1]);
            assert_ne!(notes, "-", "reviewed {}:{} lost notes", row[0], row[1]);
        }
    }

    let status_total = [
        Status::Same,
        Status::Improved,
        Status::Tiered,
        Status::Excluded,
        Status::Unreviewed,
    ]
    .into_iter()
    .map(|status| schema.ledger_status_count(status))
    .sum::<usize>();
    assert_eq!(
        status_total,
        schema.ledger_row_count(),
        "every Ledger row must belong to exactly one semantic tier"
    );
}
