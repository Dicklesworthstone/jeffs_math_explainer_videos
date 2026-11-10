//! The `fmn-manifest-compare` verifier binary (fm-certified-closure-integrity-4fei):
//! its exit codes and JSON line over real FMNP files.

use fmn_hash::sha256;
use fmn_output::{
    ClosureItem, ManifestIdentity, ManifestMode, ManifestOutput, ProvenanceManifest,
    StructuralField,
};
use std::path::PathBuf;
use std::process::Command;

/// A certified manifest as one platform writes it (C3 split into the
/// neutral toolchain and the `platform/target` record).
fn manifest(mode: ManifestMode, triple: &str, config: u64, png: &[u8]) -> ProvenanceManifest {
    let build_id = "git:0123456789abcdef";
    let mut items = vec![
        ClosureItem::structural(1, "C1", &[StructuralField::U64(1)]).expect("C1"),
        ClosureItem::byte_input(2, "franken_manim.build", build_id.as_bytes(), "build")
            .expect("build"),
        ClosureItem::byte_input(2, "SUITE.lock", b"suite", "suite lock").expect("suite"),
        ClosureItem::structural(3, "toolchain", &[StructuralField::Text("nightly-test")])
            .expect("C3"),
        ClosureItem::structural_at(
            3,
            "platform/target",
            "target",
            &[StructuralField::Text(triple)],
        )
        .expect("C3 platform"),
        ClosureItem::structural(4, "C4", &[StructuralField::U64(config)]).expect("C4"),
    ];
    for id in 5..=10 {
        items.push(
            ClosureItem::structural(id, format!("C{id}"), &[StructuralField::U64(u64::from(id))])
                .expect("structural item"),
        );
    }
    let declared_config_digest = items
        .iter()
        .find(|item| item.item_id == 10)
        .expect("C10")
        .digest;
    ProvenanceManifest::new(
        mode,
        items,
        ManifestIdentity {
            build_id: build_id.to_owned(),
            suite_lock_digest: sha256(b"suite"),
            toolchain: "nightly-test".to_owned(),
            target_triple: triple.to_owned(),
            target_features: "baseline".to_owned(),
            engine: "certified-cpu:scalar:1".to_owned(),
            simd_tier: "portable".to_owned(),
            declared_config_digest,
        },
        vec![ManifestOutput {
            virtual_path: "scene.png".to_owned(),
            kind: "canonical_png".to_owned(),
            digest: sha256(png),
            certified: mode == ManifestMode::Certified,
        }],
        None,
    )
    .expect("manifest")
}

fn write(dir: &std::path::Path, name: &str, manifest: &ProvenanceManifest) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, manifest.to_bytes().expect("encode")).expect("write manifest");
    path
}

fn compare(left: &std::path::Path, right: &std::path::Path) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fmn-manifest-compare"))
        .arg(left)
        .arg(right)
        .output()
        .expect("run fmn-manifest-compare");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn the_verifier_names_agreement_determinism_failure_and_incomparable_inputs() {
    let dir = std::env::temp_dir().join(format!("fmn-manifest-compare-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let x86 = write(
        &dir,
        "x86.fmnp",
        &manifest(
            ManifestMode::Certified,
            "x86_64-unknown-linux-gnu",
            7,
            b"png",
        ),
    );
    let arm = write(
        &dir,
        "arm.fmnp",
        &manifest(ManifestMode::Certified, "aarch64-apple-darwin", 7, b"png"),
    );
    let drifted = write(
        &dir,
        "drift.fmnp",
        &manifest(ManifestMode::Certified, "aarch64-apple-darwin", 7, b"other"),
    );
    let other = write(
        &dir,
        "other.fmnp",
        &manifest(ManifestMode::Certified, "aarch64-apple-darwin", 8, b"png"),
    );
    let standard = write(
        &dir,
        "standard.fmnp",
        &manifest(ManifestMode::Standard, "aarch64-apple-darwin", 7, b"png"),
    );

    let (code, line) = compare(&x86, &arm);
    assert_eq!(code, Some(0), "{line}");
    assert!(
        line.contains("\"verdict\":\"agree\"") && line.contains("\"closure_equal\":false"),
        "{line}"
    );

    let (code, line) = compare(&x86, &drifted);
    assert_eq!(code, Some(1), "{line}");
    assert!(
        line.contains("\"verdict\":\"determinism-failure\"")
            && line.contains("\"differing_outputs\":[\"scene.png\"]"),
        "{line}"
    );

    let (code, line) = compare(&x86, &other);
    assert_eq!(code, Some(2), "{line}");
    assert!(line.contains("\"semantic_equal\":false"), "{line}");

    // A standard manifest carries no certified promise to check.
    let (code, line) = compare(&x86, &standard);
    assert_eq!(code, Some(2), "{line}");
    assert!(line.contains("\"certified_both\":false"), "{line}");

    let (code, _) = compare(&x86, &dir.join("missing.fmnp"));
    assert_eq!(code, Some(64));
    let usage = Command::new(env!("CARGO_BIN_EXE_fmn-manifest-compare"))
        .output()
        .expect("run without arguments");
    assert_eq!(usage.status.code(), Some(64));
}
