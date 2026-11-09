// Cargo exposes only the *base* profile (`PROFILE` = `debug`/`release`) to
// build scripts, never the selected profile's name, so `release-perf` must be
// recovered from the layout of `OUT_DIR`. Two layouts exist:
//
//   legacy:     <profile-dir>/build/<package>-<hash>/out
//   build-dir:  <profile-dir>/build/<package>/<hash>/out   (nightly-2026-08-31)
//
// Counting a fixed number of ancestors picks the wrong component on one of
// them (the build-dir layout yielded `build`, so every release-perf producer
// refused to run and manifests recorded `cargo_profile = "build"`).
//
// This file is `include!`d by the fmn-cli, fmn-conformance and fmn-python
// build scripts and unit-tested as a module of fmn-cli; keep it free of
// crate-relative paths.

/// The Cargo profile name selected for the artifact whose `OUT_DIR` is given.
///
/// `base_profile` is Cargo's `PROFILE` build-script variable, used to reject a
/// derivation that contradicts what Cargo reported.
fn cargo_profile_from_out_dir(
    out_dir: &std::path::Path,
    base_profile: &str,
) -> Result<String, String> {
    let names: Vec<&str> = out_dir
        .ancestors()
        .take(5)
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
        })
        .collect();
    let at = |index: usize| names.get(index).copied().unwrap_or("");
    let is_hash = |value: &str| !value.is_empty() && value.bytes().all(|b| b.is_ascii_hexdigit());
    if at(0) != "out" {
        return Err(format!("OUT_DIR {out_dir:?} does not end in `out`"));
    }
    let directory = if is_hash(at(1)) && at(3) == "build" {
        at(4)
    } else if at(2) == "build"
        && at(1)
            .rsplit_once('-')
            .is_some_and(|(package, hash)| !package.is_empty() && is_hash(hash))
    {
        at(3)
    } else {
        return Err(format!(
            "OUT_DIR {out_dir:?} matches neither Cargo build-directory layout"
        ));
    };
    let profile = if directory == "debug" {
        "dev"
    } else {
        directory
    };
    if profile.is_empty()
        || profile == "build"
        || !profile.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(format!(
            "Cargo selected a non-portable profile directory {directory:?}"
        ));
    }
    let consistent = match profile {
        "dev" | "test" => base_profile == "debug",
        "release" | "bench" => base_profile == "release",
        _ => matches!(base_profile, "debug" | "release"),
    };
    if !consistent {
        return Err(format!(
            "derived profile {profile:?} contradicts Cargo's base profile {base_profile:?}"
        ));
    }
    Ok(profile.to_owned())
}

#[cfg(test)]
mod tests {
    use super::cargo_profile_from_out_dir;
    use std::path::Path;

    fn derive(path: &str, base: &str) -> Result<String, String> {
        cargo_profile_from_out_dir(Path::new(path), base)
    }

    #[test]
    fn both_cargo_layouts_yield_the_selected_profile() {
        let cases = [
            // build-dir layout (the pinned nightly)
            (
                "/t/debug/build/fmn-cli/60dcb5aa321d1121/out",
                "debug",
                "dev",
            ),
            (
                "/t/release/build/fmn-cli/8516584693fdffd5/out",
                "release",
                "release",
            ),
            (
                "/t/release-perf/build/fmn-conformance/0123456789abcdef/out",
                "release",
                "release-perf",
            ),
            (
                "/t/x86_64-unknown-linux-gnu/release-perf/build/fmn-python/a0d3a36eb0de3858/out",
                "release",
                "release-perf",
            ),
            // legacy layout
            (
                "/t/debug/build/fmn-cli-60dcb5aa321d1121/out",
                "debug",
                "dev",
            ),
            (
                "/t/release/build/fmn-conformance-8516584693fdffd5/out",
                "release",
                "release",
            ),
            (
                "/t/release-perf/build/fmn-python-a0d3a36eb0de3858/out",
                "release",
                "release-perf",
            ),
        ];
        for (out_dir, base, expected) in cases {
            assert_eq!(derive(out_dir, base).as_deref(), Ok(expected), "{out_dir}");
        }
    }

    #[test]
    fn the_old_fixed_ancestor_count_is_rejected_by_these_cases() {
        // Negative control: the replaced derivation (`ancestors().nth(3)`)
        // returns `build` on the build-dir layout. The table above must not
        // be satisfiable by it.
        let old = |path: &str| {
            Path::new(path)
                .ancestors()
                .nth(3)
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        };
        let out_dir = "/t/release-perf/build/fmn-conformance/0123456789abcdef/out";
        assert_eq!(old(out_dir).as_deref(), Some("build"));
        assert_eq!(derive(out_dir, "release").as_deref(), Ok("release-perf"));
    }

    #[test]
    fn unrecognized_or_contradictory_layouts_refuse() {
        for (out_dir, base) in [
            ("/t/release/build/fmn-cli/8516584693fdffd5", "release"),
            ("/t/release/fmn-cli/8516584693fdffd5/out", "release"),
            ("/t/release/build/fmn-cli/not-a-hash/out", "release"),
            ("/t/Release/build/fmn-cli/8516584693fdffd5/out", "release"),
            ("/t/build/build/fmn-cli/8516584693fdffd5/out", "release"),
            ("/t/release/build/fmn-cli/8516584693fdffd5/out", "debug"),
            ("/t/debug/build/fmn-cli/8516584693fdffd5/out", "release"),
            ("out", "debug"),
        ] {
            assert!(derive(out_dir, base).is_err(), "{out_dir} with {base}");
        }
    }
}
