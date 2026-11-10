//! The bundle-manifest drift gate and license-completeness check (fm-aef,
//! acceptance 1+2): the committed `dist/FONT_BUNDLE.json` must regenerate
//! byte-for-byte from the actual bundled faces (`fmd_font::bundled::
//! ALL_FACES` at the SUITE.lock pin) through fmn-hash's SHA-256 — the same
//! identity the §16.7 input closure keys against — and every bundled face
//! must ship with its OFL text plus the engine's MIT+rider license.
//! Nothing unlicensed ships; a font change without regeneration fails here.

use fmn_conformance::font_bundle::{
    ENGINE_LICENSE_PATH, FACE_POLICY, LICENSE_SLUGS, build_manifest, ofl_path, render_manifest,
    suite_lock_pin,
};
use fmn_hash::sha256;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path)
        .unwrap_or_else(|e| std::panic::panic_any(format!("reading {}: {e}", path.display())))
}

fn read_string(path: &Path) -> String {
    String::from_utf8(read(path))
        .unwrap_or_else(|e| std::panic::panic_any(format!("{} is not UTF-8: {e}", path.display())))
}

/// Rebuilds the manifest from ground truth: the bundled faces, the
/// committed `dist/licenses/` inventory, the repository LICENSE, and the
/// SUITE.lock pin.
fn ground_truth_manifest(root: &Path) -> fmn_conformance::font_bundle::BundleManifest {
    let suite_lock = read_string(&root.join("SUITE.lock"));
    let rev = suite_lock_pin(&suite_lock, "franken_markdown")
        .unwrap_or_else(|| std::panic::panic_any("SUITE.lock pins no franken_markdown row"));
    let ofl_texts: Vec<(&str, Vec<u8>)> = LICENSE_SLUGS
        .iter()
        .map(|slug| (*slug, read(&root.join("dist").join(ofl_path(slug)))))
        .collect();
    let ofl_refs: Vec<(&str, &[u8])> = ofl_texts
        .iter()
        .map(|(slug, bytes)| (*slug, bytes.as_slice()))
        .collect();
    let engine_license = read(&root.join("LICENSE"));
    let faces: Vec<(&str, &[u8])> = fmd_font::bundled::ALL_FACES.to_vec();
    build_manifest(&rev, &faces, &ofl_refs, &engine_license).unwrap_or_else(|e| {
        std::panic::panic_any(format!("building the ground-truth manifest: {e}"))
    })
}

#[test]
fn manifest_regenerates_byte_for_byte_from_the_bundled_faces() {
    let root = repo_root();
    let expected = render_manifest(&ground_truth_manifest(&root));
    let committed_path = root.join("dist/FONT_BUNDLE.json");
    let committed = read_string(&committed_path);
    if committed != expected {
        let actual_path = root.join("dist/FONT_BUNDLE.json.actual");
        if let Err(e) = std::fs::write(&actual_path, &expected) {
            std::panic::panic_any(format!("writing {}: {e}", actual_path.display()));
        }
        std::panic::panic_any(
            "dist/FONT_BUNDLE.json drifted from the bundled faces at the SUITE.lock pin \
             (expected form written to dist/FONT_BUNDLE.json.actual). A font or license change \
             must regenerate the manifest: cargo run -p fmn-conformance --bin gen_font_manifest",
        );
    }
}

#[test]
fn face_hashes_are_the_input_closure_identity() {
    let root = repo_root();
    let manifest = ground_truth_manifest(&root);
    assert_eq!(manifest.faces.len(), fmd_font::bundled::ALL_FACES.len());
    for ((name, bytes), face) in fmd_font::bundled::ALL_FACES
        .iter()
        .zip(manifest.faces.iter())
    {
        assert_eq!(&face.name, name, "registry order must match ALL_FACES");
        assert_eq!(face.byte_len as usize, bytes.len(), "{name}: byte length");
        assert_eq!(
            face.sha256_hex,
            sha256(bytes).to_hex(),
            "{name}: the manifest hash must be fmn-hash's SHA-256 over the exact bundled bytes \
             (the §16.7 input-closure identity)"
        );
        assert_eq!(
            face.sha256_hex.len(),
            64,
            "{name}: lowercase-hex digest rendering"
        );
    }
}

#[test]
fn every_bundled_face_ships_with_its_ofl_text() {
    let root = repo_root();
    let manifest = ground_truth_manifest(&root);
    for face in &manifest.faces {
        let ofl = read(&root.join("dist").join(&face.license));
        assert!(!ofl.is_empty(), "{}: empty OFL text", face.name);
        assert!(
            ofl.windows(b"OPEN FONT LICENSE".len())
                .any(|w| w == b"OPEN FONT LICENSE"),
            "{}: {} is not an SIL OFL text",
            face.name,
            face.license
        );
    }
    // Every face's family has its license set, and every license set on
    // disk is claimed by at least one face (no orphan, unlicensed-by-
    // omission assets in the bundle).
    for policy in FACE_POLICY {
        assert!(
            manifest
                .licenses
                .iter()
                .any(|l| l.path == ofl_path(policy.license_slug)),
            "{}: license set '{}' does not ship",
            policy.name,
            policy.license_slug
        );
    }
    let fonts_dir = root.join("dist/licenses/fonts");
    let entries: Vec<String> = std::fs::read_dir(&fonts_dir)
        .unwrap_or_else(|e| std::panic::panic_any(format!("reading {}: {e}", fonts_dir.display())))
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    for file in &entries {
        let rel = format!("licenses/fonts/{file}");
        assert!(
            manifest.licenses.iter().any(|l| l.path == rel),
            "dist/licenses/fonts/{file} ships unlisted in the manifest: no unlicensed asset ships"
        );
    }
    assert_eq!(
        entries.len(),
        LICENSE_SLUGS.len(),
        "the shipped font-license inventory must be exactly the {} license sets",
        LICENSE_SLUGS.len()
    );
}

#[test]
fn the_engine_license_ships_in_the_bundle() {
    let root = repo_root();
    let manifest = ground_truth_manifest(&root);
    let engine = manifest
        .licenses
        .iter()
        .find(|l| l.path == ENGINE_LICENSE_PATH)
        .unwrap_or_else(|| {
            std::panic::panic_any("the manifest ships no engine license (MIT+rider)")
        });
    let repo_license = read(&root.join("LICENSE"));
    let shipped = read(&root.join("dist").join(ENGINE_LICENSE_PATH));
    assert_eq!(
        shipped, repo_license,
        "dist/{ENGINE_LICENSE_PATH} must equal the repo LICENSE"
    );
    assert_eq!(engine.sha256_hex, sha256(&repo_license).to_hex());
    let text = String::from_utf8_lossy(&repo_license);
    assert!(
        text.contains("MIT License"),
        "engine license must be the MIT text"
    );
    assert!(
        text.contains("RIDER"),
        "engine license must carry the rider"
    );
}

#[test]
fn manifest_pin_matches_suite_lock() {
    let root = repo_root();
    let manifest = ground_truth_manifest(&root);
    let suite_lock = read_string(&root.join("SUITE.lock"));
    let pinned = suite_lock_pin(&suite_lock, "franken_markdown")
        .unwrap_or_else(|| std::panic::panic_any("SUITE.lock pins no franken_markdown row"));
    assert_eq!(
        manifest.pin_rev, pinned,
        "the manifest pins a different franken_markdown rev than SUITE.lock — a pin bump must \
         regenerate the bundle"
    );
}

#[test]
fn manifest_lists_only_public_assets() {
    let root = repo_root();
    let committed = read_string(&root.join("dist/FONT_BUNDLE.json"));
    for marker in [
        "corpus/",
        "reference_captures",
        "manim_ref",
        "videos_ref",
        "CC BY-NC-SA",
    ] {
        assert!(
            !committed.contains(marker),
            "the manifest references the private-fixture marker '{marker}' (§15.3)"
        );
    }
    let manifest = ground_truth_manifest(&root);
    let face_names: Vec<&str> = fmd_font::bundled::ALL_FACES
        .iter()
        .map(|(name, _)| *name)
        .collect();
    for face in &manifest.faces {
        assert!(
            face_names.contains(&face.name.as_str()),
            "manifest face '{}' is not a bundled ALL_FACES face",
            face.name
        );
        assert!(
            face.license.starts_with("licenses/fonts/"),
            "{}: license path '{}' escapes the license inventory",
            face.name,
            face.license
        );
    }
}
