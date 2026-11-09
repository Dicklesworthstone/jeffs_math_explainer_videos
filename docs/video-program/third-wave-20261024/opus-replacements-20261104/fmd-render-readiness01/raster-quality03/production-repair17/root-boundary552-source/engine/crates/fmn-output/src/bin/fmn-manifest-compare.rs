//! `fmn-manifest-compare A.fmnp B.fmnp`: the certified-matrix check of
//! docs/INPUT_CLOSURE.md §4. Two FMNP manifests, typically from two
//! certified platforms, are compared by their platform-neutral semantic
//! digests and their certified output digests.
//!
//! Prints one JSON line (`schema` `fmn.manifest-compare`, version 1) and
//! exits 0 when the certified promise holds (same semantic input, both
//! certified, every certified output bit-identical), 1 on a determinism
//! failure (same semantic input, a certified output differs), 2 when the
//! two manifests are not comparable (different input, or not both
//! certified), and 64 on a usage or read error.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use fmn_output::ProvenanceManifest;

/// No manifest a render publishes comes near this; a larger file is not one.
const MAX_MANIFEST_BYTES: u64 = 64 * 1024 * 1024;

fn load(path: &std::ffi::OsStr) -> Result<ProvenanceManifest, String> {
    let shown = std::path::Path::new(path).display().to_string();
    let length = std::fs::metadata(path)
        .map_err(|error| format!("{shown}: {error}"))?
        .len();
    if length > MAX_MANIFEST_BYTES {
        return Err(format!(
            "{shown}: {length} bytes exceeds the manifest bound"
        ));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("{shown}: {error}"))?;
    ProvenanceManifest::from_bytes(&bytes).map_err(|error| format!("{shown}: {error}"))
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ch if u32::from(ch) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(ch))),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [left, right] = args.as_slice() else {
        eprintln!("usage: fmn-manifest-compare A.fmnp B.fmnp");
        return ExitCode::from(64);
    };
    let (left, right) = match (load(left), load(right)) {
        (Ok(left), Ok(right)) => (left, right),
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("fmn-manifest-compare: {error}");
            return ExitCode::from(64);
        }
    };
    let digests = left
        .semantic_digest()
        .and_then(|l| right.semantic_digest().map(|r| (l, r)));
    let (comparison, (left_semantic, right_semantic)) =
        match left.compare(&right).and_then(|c| digests.map(|d| (c, d))) {
            Ok(result) => result,
            Err(error) => {
                eprintln!("fmn-manifest-compare: {error}");
                return ExitCode::from(64);
            }
        };
    let (verdict, code) = if comparison.certified_bits_agree() {
        ("agree", 0)
    } else if comparison.semantic_equal && comparison.certified_both {
        ("determinism-failure", 1)
    } else {
        ("not-comparable", 2)
    };
    let outputs: Vec<String> = comparison
        .differing_outputs
        .iter()
        .map(|path| json_string(path))
        .collect();
    println!(
        "{{\"schema\":\"fmn.manifest-compare\",\"version\":1,\"verdict\":{},\
         \"semantic_equal\":{},\"closure_equal\":{},\"certified_both\":{},\
         \"left_semantic_digest\":{},\"right_semantic_digest\":{},\
         \"differing_outputs\":[{}]}}",
        json_string(verdict),
        comparison.semantic_equal,
        comparison.closure_equal,
        comparison.certified_both,
        json_string(&left_semantic.to_string()),
        json_string(&right_semantic.to_string()),
        outputs.join(",")
    );
    ExitCode::from(code)
}
