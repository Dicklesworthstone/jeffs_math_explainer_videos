//! The Behavior Notes register (`docs/behavior_notes/README.md`) is the
//! authority ADR-0009 names, so it must agree with the note files themselves
//! (fm-5wq.49): every `BN-NN-*.md` is linked from its number's row, one row per
//! number, several files under one number only where the row declares a family
//! (rule 2), and every note carries a `**Status:**` of `Draft` or `Final`
//! (rule 4). Drift found on 2026-10-04: five notes had no row, BN-04 had an
//! undeclared second file, and six notes had no status line.
#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn notes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/behavior_notes")
}

/// `BN-NN` of a file name such as `BN-07-frame-marker-constants.md`.
fn note_number(name: &str) -> Option<String> {
    let number = name.strip_prefix("BN-")?.get(..2)?;
    (number.bytes().all(|b| b.is_ascii_digit()) && name.get(5..6) == Some("-"))
        .then(|| format!("BN-{number}"))
}

/// Register rows: number -> (linked note files, whether the row declares a family).
fn register() -> BTreeMap<String, (BTreeSet<String>, bool)> {
    let text = std::fs::read_to_string(notes_dir().join("README.md")).expect("register exists");
    let mut rows = BTreeMap::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('|') else {
            continue;
        };
        let number = rest.split('|').next().unwrap_or("").trim();
        if note_number(&format!("{number}-x.md")).is_none() {
            continue;
        }
        let mut files = BTreeSet::new();
        for piece in line.split("](").skip(1) {
            let target = piece.split(')').next().unwrap_or("");
            if note_number(target).is_some() && target.ends_with(".md") {
                files.insert(target.to_owned());
            }
        }
        let family = line.contains("a family");
        assert!(
            rows.insert(number.to_owned(), (files, family)).is_none(),
            "{number} has more than one register row"
        );
    }
    rows
}

#[test]
fn every_note_file_is_registered_under_its_own_number() {
    let rows = register();
    let mut seen = BTreeSet::new();
    for entry in std::fs::read_dir(notes_dir()).expect("notes directory") {
        let name = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        let Some(number) = note_number(&name) else {
            continue;
        };
        let (files, _) = rows
            .get(&number)
            .unwrap_or_else(|| panic!("{name}: {number} has no register row"));
        assert!(
            files.contains(&name),
            "{name} is not linked from the {number} row"
        );
        seen.insert(name);
    }
    for (number, (files, family)) in &rows {
        for file in files {
            assert!(seen.contains(file), "{number} links missing file {file}");
            assert_eq!(note_number(file).as_deref(), Some(number.as_str()));
        }
        assert!(
            files.len() <= 1 || *family,
            "{number} links {} files without declaring a family (ADR-0009 rule 2)",
            files.len()
        );
    }
}

#[test]
fn every_note_has_a_draft_or_final_status() {
    for entry in std::fs::read_dir(notes_dir()).expect("notes directory") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if note_number(&name).is_none() {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("note readable");
        let status = text
            .lines()
            .find_map(|line| line.split_once("**Status:**").map(|(_, rest)| rest.trim()))
            .unwrap_or_else(|| panic!("{name} has no **Status:** line"));
        let word: String = status
            .chars()
            .filter(|c| *c != '*')
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        assert!(
            word == "Draft" || word == "Final",
            "{name}: status {status:?} is outside the Draft/Final vocabulary (ADR-0009 rule 4)"
        );
    }
}
