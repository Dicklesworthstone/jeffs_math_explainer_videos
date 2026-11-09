//! VIDEO_CORPUS.lock integrity — fm-rqc (§15.3-15.4, R13).
//!
//! The committed lock is validated here without the gitignored
//! `scripts/videos_ref` checkout: schema shape, status vocabulary,
//! hash well-formedness, sortedness, attribution provenance, and the
//! pin equality against `SUITE.lock [reference]`. Byte-reproducibility
//! against the pinned tree itself is `scripts/video_corpus.py verify`,
//! which runs wherever the checkout exists (the G0-4 convention).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn read_repo_file(name: &str) -> String {
    std::fs::read_to_string(repo_root().join(name)).unwrap_or_else(|error| {
        panic!("read {name}: {error}");
    })
}

/// Section name -> data lines (comments and blanks stripped).
fn sections(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = trimmed[1..trimmed.len() - 1].to_owned();
            assert!(
                out.insert(name.clone(), Vec::new()).is_none(),
                "duplicate section [{name}]"
            );
            current = Some(name);
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let section = current.as_ref().expect("data line before any section");
        out.get_mut(section)
            .expect("section exists")
            .push(trimmed.to_owned());
    }
    out
}

fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

fn suite_reference_pins() -> BTreeMap<String, String> {
    let mut pins = BTreeMap::new();
    let mut in_ref = false;
    for line in read_repo_file("SUITE.lock").lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_ref = trimmed == "[reference]";
            continue;
        }
        if !in_ref || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut fields = trimmed.split('\t');
        if let (Some(name), Some(rev)) = (fields.next(), fields.next()) {
            pins.insert(name.to_owned(), rev.to_owned());
        }
    }
    pins
}

#[test]
fn the_lock_pins_mirror_suite_lock_reference() {
    let lock = sections(&read_repo_file("VIDEO_CORPUS.lock"));
    let suite = suite_reference_pins();
    let pins = &lock["pins"];
    assert_eq!(pins.len(), 2, "exactly the two reference pins");
    for row in pins {
        let (name, rev) = row.split_once('\t').expect("pin row is name\\trev");
        assert_eq!(
            suite.get(name).map(String::as_str),
            Some(rev),
            "{name} pin must equal SUITE.lock [reference]"
        );
    }
}

#[test]
fn scene_rows_are_well_formed_sorted_and_attributed() {
    let lock = sections(&read_repo_file("VIDEO_CORPUS.lock"));
    let scenes = &lock["scenes"];
    assert!(!scenes.is_empty(), "the seed allowlist is non-empty");

    let videos_pin = suite_reference_pins()["3b1b/videos"].clone();
    let mut previous_scene = String::new();
    for row in scenes {
        let fields: Vec<&str> = row.split('\t').collect();
        assert_eq!(fields.len(), 7, "scene row has 7 fields: {row}");
        let [scene, module, sha, era, status, attribution, note] = [
            fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6],
        ];
        assert!(
            *previous_scene < *scene,
            "scene rows sorted and unique: {previous_scene} !< {scene}"
        );
        previous_scene = scene.to_owned();
        assert!(module.ends_with(".py"), "module is a python path: {module}");
        assert!(is_sha256_hex(sha), "module hash is sha256 hex: {sha}");
        assert!(
            module.starts_with(&format!("_{era}/")),
            "era {era} matches the module tree {module}"
        );
        assert!(
            matches!(
                status,
                "allowlisted" | "pending-with-named-constructs" | "excluded"
            ),
            "status in the lock vocabulary: {status}"
        );
        assert!(
            attribution.starts_with(&format!("3b1b/videos@{}", &videos_pin[..12]))
                && attribution.ends_with(module),
            "attribution carries the pin prefix and in-tree provenance: {attribution}"
        );
        assert!(!note.is_empty(), "curation note present");
    }
}

#[test]
fn shims_are_documented_and_the_import_shim_is_hash_pinned() {
    let lock = sections(&read_repo_file("VIDEO_CORPUS.lock"));
    let shims = &lock["shims"];
    let names: Vec<&str> = shims
        .iter()
        .map(|row| row.split('\t').next().expect("shim name"))
        .collect();
    assert_eq!(
        names,
        [
            "import-virtualization",
            "asset-path-virtualization",
            "fonts",
            "kwarg-vintage"
        ],
        "the §15.3 shim axes plus the R13 kwarg-vintage era shim, in order"
    );
    for row in shims {
        let fields: Vec<&str> = row.split('\t').collect();
        assert_eq!(fields.len(), 3, "shim row has 3 fields: {row}");
        assert!(!fields[1].is_empty(), "shim mechanism documented");
        if fields[0] == "import-virtualization" {
            assert!(
                is_sha256_hex(fields[2]),
                "the era-shim entry blob is hash-pinned: {}",
                fields[2]
            );
        }
    }
}

#[test]
fn exclusions_carry_reasons_and_the_out_of_era_record_exists() {
    let lock = sections(&read_repo_file("VIDEO_CORPUS.lock"));
    let exclusions = &lock["exclusions"];
    assert!(!exclusions.is_empty(), "R13 exclusions are on the record");
    for row in exclusions {
        let (subject, reason) = row.split_once('\t').expect("exclusion row");
        assert!(!subject.is_empty() && !reason.is_empty(), "reasoned: {row}");
    }
    assert!(
        exclusions.iter().any(|row| row.contains("out-of-era")),
        "the R13 out-of-era exclusion is recorded"
    );
}

#[test]
fn every_section_the_format_promises_is_present() {
    let lock = sections(&read_repo_file("VIDEO_CORPUS.lock"));
    for section in ["pins", "shims", "scenes", "assets", "exclusions"] {
        assert!(lock.contains_key(section), "missing section [{section}]");
    }
}
