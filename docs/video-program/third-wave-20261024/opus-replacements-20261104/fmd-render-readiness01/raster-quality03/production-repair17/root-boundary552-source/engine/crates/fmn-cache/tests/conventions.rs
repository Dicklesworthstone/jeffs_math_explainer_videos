//! fm-aef cache/config convention acceptance: the documented per-platform
//! locations (docs/dist/cache_config_conventions.md) resolve exactly as
//! `resolve_host_cache_root` resolves them, and the documented store layout
//! names the real on-disk markers. This test is the drift gate between the
//! distribution documentation and the code: change either side alone and it
//! fails.

use std::path::Path;

use fmn_cache::{DEFAULT_CACHE_LEAF, resolve_host_cache_root};

/// The packaged copy keeps this test runnable from a published crate and on
/// source-transfer systems that intentionally omit repository distribution
/// outputs. A full workspace additionally proves that its distribution copy
/// is byte-identical below.
fn conventions_doc() -> &'static str {
    include_str!("../docs/cache_config_conventions.md")
}

#[test]
fn packaged_contract_matches_workspace_distribution_copy_when_present() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/dist/cache_config_conventions.md");
    let Ok(distribution_doc) = std::fs::read_to_string(path) else {
        return;
    };
    assert_eq!(
        conventions_doc(),
        distribution_doc,
        "the packaged and workspace cache/config contracts drifted"
    );
}

#[test]
fn documented_leaf_is_the_code_leaf() {
    let doc = conventions_doc();
    assert!(
        doc.contains(DEFAULT_CACHE_LEAF),
        "the doc must name the code's cache leaf {DEFAULT_CACHE_LEAF:?}"
    );
    assert_eq!(DEFAULT_CACHE_LEAF, "franken-manim");
}

#[test]
fn documented_platform_bases_match_the_resolution_rules() {
    let doc = conventions_doc();
    // Every documented resolved shape names the same base the resolver
    // constructs (store.rs's synthetic-environment tests prove resolution;
    // this proves the doc records those shapes).
    for shape in [
        "$XDG_CACHE_HOME/franken-manim",
        "$HOME/.cache/franken-manim",
        "$HOME/Library/Caches/franken-manim",
        "%LOCALAPPDATA%\\franken-manim",
        "%USERPROFILE%\\AppData\\Local\\franken-manim",
    ] {
        assert!(doc.contains(shape), "doc is missing resolved shape {shape}");
    }
    // The no-guessing refusal is part of the documented contract.
    assert!(
        doc.contains("PlatformDefaultUnavailable"),
        "doc must name the no-guessing failure"
    );
}

#[test]
fn documented_store_layout_names_the_real_markers() {
    let doc = conventions_doc();
    for marker in [
        "STORE_OWNER",
        "STORE_FORMAT",
        "fmn-cache 1",
        "ns/<name>/v<version>",
        "purge_stale_versions",
        "TYPESET_FORMAT_VERSION",
    ] {
        assert!(
            doc.contains(marker),
            "doc is missing store-layout marker {marker}"
        );
    }
}

#[test]
fn documented_config_convention_names_the_real_surfaces() {
    let doc = conventions_doc();
    for surface in [
        "$XDG_CONFIG_HOME/franken-manim/config.yml",
        "$HOME/.config/franken-manim/config.yml",
        "$HOME/Library/Application Support/franken-manim/config.yml",
        "%APPDATA%\\franken-manim\\config.yml",
        "custom_config.yml",
        "--config_file",
        "--cache-dir",
        "directories.cache",
    ] {
        assert!(
            doc.contains(surface),
            "doc is missing config surface {surface}"
        );
    }
    assert!(
        !doc.contains("fm-xdg-config-discovery-dqo6"),
        "the landed convention must not retain its implementation tracker"
    );
}

/// On the Linux host (the CI platform), the actual resolver must produce the
/// documented XDG shape. Environment mutation is `unsafe` under
/// `#![forbid(unsafe_code)]`, so this reads the host environment only — the
/// per-platform matrix is proven synthetically in store.rs's unit tests.
#[cfg(target_os = "linux")]
#[test]
fn linux_host_resolution_follows_the_documented_xdg_convention() {
    use std::path::PathBuf;

    let resolved = resolve_host_cache_root("").expect("the CI host has a cache convention");
    assert!(
        resolved.is_absolute(),
        "the platform default is always absolute: {resolved:?}"
    );
    assert_eq!(
        resolved.file_name().map(std::ffi::OsStr::as_encoded_bytes),
        Some(DEFAULT_CACHE_LEAF.as_bytes()),
        "the resolved root ends in the dedicated leaf"
    );
    let parent = resolved.parent().expect("the leaf has a base");
    let xdg = std::env::var_os("XDG_CACHE_HOME").filter(|value| {
        let path = PathBuf::from(value);
        path.is_absolute()
    });
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let documented_base = match (xdg, home) {
        (Some(xdg), _) => Some(PathBuf::from(xdg)),
        (None, Some(home)) if home.is_absolute() => Some(home.join(".cache")),
        _ => None,
    }
    .expect("the CI host provides XDG_CACHE_HOME or an absolute HOME");
    assert_eq!(
        parent, documented_base,
        "host resolution diverged from the documented XDG convention"
    );
}

/// A configured relative root anchors to the current directory, as documented.
#[test]
fn configured_relative_root_is_anchored_not_guessed() {
    let resolved = resolve_host_cache_root("relative/fmn-test-cache")
        .expect("a relative configured root anchors to the cwd");
    let cwd = std::env::current_dir().expect("test cwd");
    assert_eq!(resolved, cwd.join("relative/fmn-test-cache"));
    assert!(resolved.is_absolute());
}
