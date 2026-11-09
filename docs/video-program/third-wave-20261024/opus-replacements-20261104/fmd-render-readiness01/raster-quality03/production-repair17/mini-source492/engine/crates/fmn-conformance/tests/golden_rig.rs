//! End-to-end demonstration of the self-golden rig (fm-xb3 acceptance):
//! create → lock → match → mutate → CI-fail (with `.actual` sidecar) → bless.
//!
//! Runs against a scratch store under `CARGO_TARGET_TMPDIR`, so the committed
//! goldens are untouched; `tests/self_goldens.rs` is the rig's live use.
//! Modes are passed explicitly (never via the environment) so the tests are
//! parallel-safe.

use fmn_conformance::golden::{
    GoldenError, GoldenStore, LockEntry, Mode, REBLESS_ARTEFACT_DIR, Scope, Verdict, platform_key,
    rebless_artefact_name, rebless_violations, side_by_side_png,
};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("golden_rig_{name}"));
    // A fresh subdirectory per test; stale files from a previous run are
    // overwritten by the rig itself (bless), so no cleanup pass is needed.
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

#[test]
fn path_diagnostics_are_bounded_and_keep_the_artifact_leaf() {
    let path = PathBuf::from(format!("/{}/bounded.certified.lock", "é".repeat(200)));
    let entry = LockEntry {
        len: 1,
        sha256_hex: "00".repeat(32),
    };
    let errors = [
        GoldenError::Io {
            path: path.clone(),
            err: std::io::Error::other("probe"),
        },
        GoldenError::Corrupt {
            path: path.clone(),
            line: 1,
            detail: "probe".to_string(),
        },
        GoldenError::LockTooLarge {
            path: path.clone(),
            limit: 1024,
        },
        GoldenError::Drift {
            name: "probe".to_string(),
            expected: Some(entry.clone()),
            actual: entry,
            sidecar: path,
        },
    ];

    for error in errors {
        let diagnostic = error.to_string();
        assert!(diagnostic.len() < 512, "unbounded diagnostic: {diagnostic}");
        assert!(
            diagnostic.contains("bounded.certified.lock"),
            "diagnostic lost its useful leaf: {diagnostic}"
        );
    }
}

#[test]
fn full_lifecycle_create_lock_drift_bless() {
    let dir = scratch("lifecycle");
    let store = GoldenStore::new(&dir, "demo", Scope::PerPlatform).expect("store");
    // Locks are per-platform: the file name carries the platform key.
    let lock = store.lock_path();
    assert!(
        lock.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains(&platform_key()),
        "lock path {lock:?} must embed the platform key"
    );
    let _ = std::fs::remove_file(&lock); // reset from any previous run (scratch only)

    // 1. CREATE: an unlocked artifact fails in check mode and writes a sidecar.
    let v1 = b"artifact bytes, version 1".to_vec();
    let err = store
        .check_with_mode("trivial", &v1, Mode::Check)
        .expect_err("unlocked artifact must fail in check mode");
    assert!(
        matches!(&err, GoldenError::Drift { expected: None, .. }),
        "expected no-entry drift, got: {err}"
    );
    let GoldenError::Drift {
        name,
        expected: None,
        sidecar,
        ..
    } = err
    else {
        return;
    };
    assert_eq!(name, "trivial");
    assert_eq!(std::fs::read(&sidecar).expect("sidecar"), v1);

    // 2. LOCK: bless writes the lock entry; the rig never commits anything.
    let verdict = store
        .check_with_mode("trivial", &v1, Mode::Bless)
        .expect("bless");
    assert_eq!(verdict, Verdict::Blessed { previous: None });
    assert!(lock.is_file(), "bless must materialize the lock file");

    // 3. MATCH: the same bytes now pass in check mode.
    assert_eq!(
        store
            .check_with_mode("trivial", &v1, Mode::Check)
            .expect("relock match"),
        Verdict::Match
    );

    // 4. MUTATE → CI-FAIL: changed bytes drift, with both entries reported.
    let v2 = b"artifact bytes, version 2 (drifted)".to_vec();
    let err = store
        .check_with_mode("trivial", &v2, Mode::Check)
        .expect_err("drifted artifact must fail in check mode");
    assert!(
        matches!(
            &err,
            GoldenError::Drift {
                expected: Some(_),
                ..
            }
        ),
        "expected drift with previous entry, got: {err}"
    );
    let GoldenError::Drift {
        expected: Some(expected),
        actual,
        sidecar,
        ..
    } = err
    else {
        return;
    };
    assert_eq!(expected.len, v1.len() as u64);
    assert_eq!(actual.len, v2.len() as u64);
    assert_ne!(expected.sha256_hex, actual.sha256_hex);
    assert_eq!(std::fs::read(sidecar).expect("sidecar"), v2);

    // 5. BLESS: deliberate re-lock accepts the new bytes and reports what it
    //    replaced; a subsequent check passes.
    let verdict = store
        .check_with_mode("trivial", &v2, Mode::Bless)
        .expect("re-bless");
    assert!(
        matches!(&verdict, Verdict::Blessed { previous: Some(_) }),
        "expected replacing bless, got {verdict:?}"
    );
    let Verdict::Blessed { previous: Some(p) } = verdict else {
        return;
    };
    assert_eq!(p.len, v1.len() as u64);
    assert_eq!(
        store
            .check_with_mode("trivial", &v2, Mode::Check)
            .expect("post-bless match"),
        Verdict::Match
    );
}

#[test]
fn lock_file_bytes_are_deterministic() {
    let dir = scratch("deterministic");
    let store = GoldenStore::new(&dir, "det", Scope::Certified).expect("store");
    let _ = std::fs::remove_file(store.lock_path());
    // Bless in one order…
    store
        .check_with_mode("b-second", b"bb", Mode::Bless)
        .expect("bless b");
    store
        .check_with_mode("a-first", b"aa", Mode::Bless)
        .expect("bless a");
    let one = std::fs::read_to_string(store.lock_path()).expect("lock");
    // …then re-bless the identical content in the opposite order; the file
    // must be byte-identical (sorted rows, versioned header).
    let _ = std::fs::remove_file(store.lock_path());
    store
        .check_with_mode("a-first", b"aa", Mode::Bless)
        .expect("bless a");
    store
        .check_with_mode("b-second", b"bb", Mode::Bless)
        .expect("bless b");
    let two = std::fs::read_to_string(store.lock_path()).expect("lock");
    assert_eq!(one, two, "lock bytes must not depend on bless order");
    assert!(one.starts_with("# fmn-golden-lock v1 suite=det key=certified\n"));
    // Certified scope shares one lock file across the whole matrix.
    assert!(store.lock_path().ends_with("det.certified.lock"));
}

#[test]
fn names_are_path_components_never_traversal() {
    let dir = scratch("names");
    let store = GoldenStore::new(&dir, "names", Scope::PerPlatform).expect("store");
    for bad in ["../escape", "a/b", "", ".hidden", "UPPER", "sp ace"] {
        assert!(
            matches!(
                store.check_with_mode(bad, b"x", Mode::Check),
                Err(GoldenError::InvalidName { .. })
            ),
            "name {bad:?} must be refused"
        );
    }
    assert!(GoldenStore::new(&dir, "../sneaky", Scope::PerPlatform).is_err());

    let oversized = "a".repeat(129);
    let error = store
        .check_with_mode(&oversized, b"x", Mode::Check)
        .expect_err("an oversized artifact name must be refused before ownership");
    assert!(matches!(error, GoldenError::InvalidName { bytes: 129 }));
    assert!(error.to_string().len() < 200);
    assert!(matches!(
        GoldenStore::new(&dir, &oversized, Scope::PerPlatform),
        Err(GoldenError::InvalidName { bytes: 129 })
    ));
}

#[test]
fn corrupt_lock_is_a_named_error_not_a_pass() {
    let dir = scratch("corrupt");
    let store = GoldenStore::new(&dir, "corrupt", Scope::PerPlatform).expect("store");
    std::fs::write(store.lock_path(), "not a lock header\n").expect("write");
    let result = store.check_with_mode("x", b"x", Mode::Check);
    assert!(
        matches!(&result, Err(GoldenError::Corrupt { line: 1, .. })),
        "expected corrupt-lock error, got {result:?}"
    );
}

#[test]
fn lock_identity_and_rows_are_strictly_parsed_with_bounded_refusals() {
    let dir = scratch("strict_format");
    let store = GoldenStore::new(&dir, "strict", Scope::Certified).expect("store");
    let lock = store.lock_path();
    let exact_header = "# fmn-golden-lock v1 suite=strict key=certified";
    let sha = "00".repeat(32);

    let assert_corrupt = |text: &str, expected_line: usize, needle: &str| {
        std::fs::write(&lock, text).expect("write malformed lock");
        let result = store.load_entries();
        assert!(
            matches!(&result, Err(GoldenError::Corrupt { .. })),
            "expected corrupt-lock error, got {result:?}"
        );
        let Err(GoldenError::Corrupt { line, detail, .. }) = result else {
            return;
        };
        assert_eq!(line, expected_line, "unexpected refusal: {detail}");
        assert!(
            detail.contains(needle),
            "expected {needle:?} in refusal: {detail}"
        );
        assert!(
            detail.len() < 200,
            "refusal copied malformed input: {detail}"
        );
    };

    assert_corrupt(
        "# fmn-golden-lock v10 suite=strict key=certified\n",
        1,
        "expected header",
    );
    assert_corrupt(
        "# fmn-golden-lock v1 suite=other key=certified\n",
        1,
        "expected header",
    );
    assert_corrupt(
        "# fmn-golden-lock v1 suite=strict key=other\n",
        1,
        "expected header",
    );
    assert_corrupt(
        &format!("{exact_header}\nartifact\t1\t{sha}\t\n"),
        2,
        "expected 3 tab-separated fields, found 4",
    );
    assert_corrupt(
        &format!("{exact_header}\nartifact\t1\t{sha} \n"),
        2,
        "invalid sha256 field",
    );
    assert_corrupt(
        &format!("{exact_header}\r\nartifact\t1\t{sha}\r\n"),
        1,
        "carriage returns",
    );
    assert_corrupt(
        &format!("{exact_header}\nartifact\t1\t{sha}"),
        2,
        "end with exactly one LF",
    );
    assert_corrupt(&format!("{exact_header}\n\n"), 2, "blank data row");
    assert_corrupt(
        &format!("{exact_header}\n# extra metadata\n"),
        2,
        "comment data row",
    );
    assert_corrupt(
        &format!("{exact_header}\nartifact\t01\t{sha}\n"),
        2,
        "noncanonical length field",
    );
    assert_corrupt(
        &format!("{exact_header}\nz-last\t1\t{sha}\na-first\t1\t{sha}\n"),
        3,
        "not strictly increasing",
    );
    assert_corrupt(
        &format!("{exact_header}\nartifact\t1\t{sha}\nartifact\t1\t{sha}\n"),
        3,
        "duplicate artifact name",
    );

    let mut delimiter_heavy = format!("{exact_header}\nartifact\t1\t{sha}");
    delimiter_heavy.extend(std::iter::repeat_n('\t', 1_000_000));
    delimiter_heavy.push('\n');
    assert_corrupt(
        &delimiter_heavy,
        2,
        "expected 3 tab-separated fields, found 1000003",
    );

    let canonical = format!("{exact_header}\nartifact-a\t1\t{sha}\nartifact-b\t2\t{sha}\n");
    std::fs::write(&lock, &canonical).expect("write canonical lock");
    let entries = store.load_entries().expect("canonical lock parses");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries["artifact-a"].len, 1);
    assert_eq!(entries["artifact-b"].len, 2);
    assert_eq!(entries["artifact-a"].sha256_hex, sha);
    assert_eq!(
        std::fs::read_to_string(&lock).expect("read canonical lock"),
        canonical,
        "accepted lock bytes must already match the canonical writer"
    );
}

#[test]
fn lock_document_envelope_bounds_reads_and_refuses_oversized_bless_atomically() {
    const LIMIT: usize = 1024 * 1024;

    let dir = scratch("document_envelope");
    let store = GoldenStore::new(&dir, "bounded", Scope::Certified).expect("store");
    let lock = store.lock_path();
    std::fs::write(&lock, vec![b'x'; LIMIT + 1]).expect("write oversized lock");
    let error = store
        .load_entries()
        .expect_err("an oversized lock must be refused");
    assert!(matches!(
        error,
        GoldenError::LockTooLarge { limit: LIMIT, .. }
    ));
    assert!(error.to_string().len() < 200);

    let sha = "00".repeat(32);
    let mut canonical = String::from("# fmn-golden-lock v1 suite=bounded key=certified\n");
    for index in 0_u32.. {
        let row = format!("entry-{index:05}\t1\t{sha}\n");
        if canonical.len() + row.len() > LIMIT {
            break;
        }
        canonical.push_str(&row);
    }
    let new_name = "z".repeat(128);
    let new_row_len = new_name.len() + 1 + 1 + 1 + 64 + 1;
    assert!(
        LIMIT - canonical.len() < new_row_len,
        "the canonical fixture must leave less than one maximal row"
    );
    std::fs::write(&lock, canonical.as_bytes()).expect("write near-limit canonical lock");

    let error = store
        .check_with_mode(&new_name, b"x", Mode::Bless)
        .expect_err("a bless that crosses the format envelope must be refused");
    assert!(matches!(
        error,
        GoldenError::LockTooLarge { limit: LIMIT, .. }
    ));
    assert_eq!(
        std::fs::read(&lock).expect("read unchanged lock"),
        canonical.as_bytes(),
        "a refused oversized bless must not rewrite the lock"
    );
}

/// Every committed golden lock, as `(file name, SHA-256 hex)`.
fn committed_locks() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens");
    let mut locks: Vec<_> = std::fs::read_dir(&dir)
        .expect("list goldens")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "lock"))
        .map(|path| {
            let bytes = std::fs::read(&path).expect("read lock");
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("UTF-8 lock name")
                .to_owned();
            (name, fmn_hash::sha256(&bytes).to_hex())
        })
        .collect();
    locks.sort();
    assert!(locks.len() >= 6, "the golden locks went missing: {locks:?}");
    locks
}

fn committed_artefact(name: &str) -> Option<Vec<u8>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read(root.join(REBLESS_ARTEFACT_DIR).join(name)).ok()
}

/// fm-5wq.46: every golden lock at its committed digest is either the
/// protocol's launch baseline or carries its before/after artefact.
#[test]
fn every_golden_lock_carries_its_rebless_artefact() {
    let violations = rebless_violations(&committed_locks(), &committed_artefact);
    assert!(
        violations.is_empty(),
        "re-bless protocol (docs/GOVERNANCE.md §4): {violations:#?}"
    );
}

/// The planted negatives: a lock changed without an artefact, and an
/// artefact that is not a PNG, both fail the protocol.
#[test]
fn a_rebless_without_its_side_by_side_fails_the_protocol() {
    let mut locks = committed_locks();
    let (lock, digest) = locks[0].clone();
    let reblessed = fmn_hash::sha256(format!("{digest} re-blessed").as_bytes()).to_hex();
    locks[0].1.clone_from(&reblessed);
    let violations = rebless_violations(&locks, &committed_artefact);
    assert_eq!(violations.len(), 1, "{violations:#?}");
    assert!(
        violations[0].contains(&lock)
            && violations[0].contains(&rebless_artefact_name(&lock, &reblessed)),
        "{violations:#?}"
    );

    let expected = rebless_artefact_name(&lock, &reblessed);
    let not_png = |name: &str| (name == expected).then(|| b"not a png".to_vec());
    let violations = rebless_violations(&locks, &|name| {
        not_png(name).or_else(|| committed_artefact(name))
    });
    assert_eq!(violations.len(), 1, "{violations:#?}");
    assert!(violations[0].contains("is not a PNG"), "{violations:#?}");

    let panel = side_by_side_png(2, 1, &[1; 8], &[2; 8]).expect("panel");
    let accepted = |name: &str| (name == expected).then(|| panel.clone());
    assert!(
        rebless_violations(&locks, &|name| accepted(name)
            .or_else(|| committed_artefact(name)))
        .is_empty(),
        "an artefact under the digest's name satisfies the protocol"
    );
}

#[test]
fn side_by_side_panels_put_before_left_and_after_right() {
    let before = [10_u8, 20, 30, 255].repeat(3 * 2);
    let after = [40_u8, 50, 60, 255].repeat(3 * 2);
    let png = side_by_side_png(3, 2, &before, &after).expect("panel");
    let decoded =
        fmn_codec::decode_png(&png, &fmn_codec::PngLimits::default()).expect("panel decodes");
    assert_eq!((decoded.width, decoded.height), (3 + 4 + 3, 2));
    let pixel = |x: usize, y: usize| &decoded.rgba[(y * 10 + x) * 4..(y * 10 + x) * 4 + 4];
    assert_eq!(pixel(0, 1), [10, 20, 30, 255]);
    assert_eq!(pixel(4, 0), [255, 0, 255, 255]);
    assert_eq!(pixel(9, 1), [40, 50, 60, 255]);
    assert!(side_by_side_png(3, 2, &before, &after[4..]).is_err());
}
