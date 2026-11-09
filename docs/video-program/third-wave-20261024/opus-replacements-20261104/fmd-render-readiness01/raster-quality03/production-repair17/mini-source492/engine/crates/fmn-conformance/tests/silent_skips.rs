//! No test passes silently when its input is absent (fm-5wq.48).
//!
//! A test that prints "skipping" and returns success proves nothing on a
//! machine without its input, and nothing counts or reports the skip. Missing
//! inputs go through `fmn_core::test_inputs::skip_or_fail`, which records a
//! `SKIPPED` ledger line and fails under `FMN_REQUIRE_FULL_INPUTS=1`. This
//! test rejects the bare pattern, a print about a skip or an absent input
//! directly followed by `return`, anywhere under `crates/`. Deliberate modes
//! are not missing inputs: a print guarded by an environment switch, or one
//! naming its switch (`set FMN_…=1`), is allowed.
#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

/// Words that mark a message as reporting a skip or an absent input.
const SKIP_WORDS: [&str; 6] = [
    "skip",
    "not present",
    "absent",
    "not built",
    "built yet",
    "missing",
];

/// Each bare print-then-return site in `source` as `(line, message)`.
fn bare_skips(source: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        if !(trimmed.starts_with("eprintln!(") || trimmed.starts_with("println!(")) {
            index += 1;
            continue;
        }
        let start = index;
        let mut call = String::new();
        while index < lines.len() {
            call.push_str(lines[index].trim());
            call.push(' ');
            if lines[index].trim_end().ends_with(");") {
                break;
            }
            index += 1;
        }
        index += 1;
        let next = lines[index.min(lines.len())..]
            .iter()
            .map(|line| line.trim())
            .find(|line| !line.is_empty() && !line.starts_with("//"));
        let lower = call.to_lowercase();
        let reports_skip = SKIP_WORDS.iter().any(|word| lower.contains(word));
        let guarded_by_switch = lines[start.saturating_sub(2)..start]
            .iter()
            .any(|line| line.contains("env::var"));
        let opt_in_tier = lower.contains("set fmn_") || guarded_by_switch;
        if next.is_some_and(|line| line.starts_with("return")) && reports_skip && !opt_in_tier {
            found.push((start + 1, call.trim().to_owned()));
        }
    }
    found
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("read crate directory");
    for entry in entries {
        let path = entry.expect("directory entry").path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && !name.starts_with('.') {
                rust_files(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

/// The detector catches the three shapes fm-5wq.48 found in the tree.
#[test]
fn the_detector_flags_the_patterns_it_replaced() {
    let planted = [
        "let Ok(x) = read() else {\n    eprintln!(\"skipping: pinned Reference checkout not present\");\n    return;\n};",
        "Err(e) => {\n    eprintln!(\n        \"corpus not present at {} — recompute skipped\",\n        path.display()\n    );\n    return Ok(());\n}",
        "if !artifact.exists() {\n    eprintln!(\n        \"size_budget: no wasm artifact built yet\"\n    );\n    // Building is deliberate.\n    return;\n}",
    ];
    for sample in planted {
        assert_eq!(bare_skips(sample).len(), 1, "missed: {sample}");
    }
    let allowed = [
        "eprintln!(\"full campaign skipped (set FMN_FUZZ_FULL=1)\");\nreturn;",
        "if std::env::var_os(\"FMN_FUZZ_BLESS\").is_some() {\n    eprintln!(\"bless mode: check skipped\");\n    return;\n}",
        "eprintln!(\"skipping\");\nlet y = 1;",
        "eprintln!(\"rendered {n} frames\");\nreturn;",
    ];
    for sample in allowed {
        assert!(bare_skips(sample).is_empty(), "false positive: {sample}");
    }
}

#[test]
fn no_test_passes_silently_without_its_input() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf();
    let mut files = Vec::new();
    rust_files(&crates, &mut files);
    files.sort();
    assert!(files.len() > 100, "scanned only {} files", files.len());
    let mut offenders = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file).expect("read source");
        for (line, message) in bare_skips(&source) {
            let relative = file.strip_prefix(&crates).unwrap_or(file);
            offenders.push(format!("crates/{}:{line}: {message}", relative.display()));
        }
    }
    assert!(
        offenders.is_empty(),
        "tests that pass silently without their input; call skip_or_fail instead:\n{}",
        offenders.join("\n")
    );
}
