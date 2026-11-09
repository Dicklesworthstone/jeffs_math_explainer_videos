//! Permanent FrankenNumPy source-identity and RNG-closure checks.

#![forbid(unsafe_code)]

use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};

const MAX_AUTHORITY_BYTES: u64 = 8 * 1024 * 1024;
const CONSUMED_FNP_PACKAGES: &[&str] = &["fnp-dtype", "fnp-io", "fnp-ndarray", "fnp-random-core"];

type PackageBlocks = BTreeMap<String, Vec<String>>;

fn repo_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name)
}

fn read_repo_file(name: &str) -> String {
    let path = repo_path(name);
    let file = File::open(&path).unwrap_or_else(|error| {
        std::panic::panic_any(format!("opening {}: {error}", path.display()))
    });
    let mut bytes = Vec::new();
    file.take(MAX_AUTHORITY_BYTES + 1)
        .read_to_end(&mut bytes)
        .unwrap_or_else(|error| {
            std::panic::panic_any(format!("reading {}: {error}", path.display()))
        });
    assert!(
        u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= MAX_AUTHORITY_BYTES,
        "{} exceeds the {MAX_AUTHORITY_BYTES}-byte authority limit",
        path.display()
    );
    String::from_utf8(bytes).unwrap_or_else(|error| {
        std::panic::panic_any(format!("{} is not UTF-8: {error}", path.display()))
    })
}

fn suite_pin(document: &str, repo: &str) -> String {
    document
        .lines()
        .find(|line| line.starts_with(&format!("{repo}\t")))
        .and_then(|line| line.split('\t').nth(1))
        .filter(|pin| pin.len() == 40 && pin.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .unwrap_or_else(|| std::panic::panic_any(format!("SUITE.lock must pin {repo} at full SHA")))
        .to_owned()
}

fn package_blocks(lock: &str) -> PackageBlocks {
    let mut packages = PackageBlocks::new();
    for block in lock.split("\n[[package]]\n").skip(1) {
        let name = block
            .lines()
            .find_map(|line| line.strip_prefix("name = \"")?.strip_suffix('"'))
            .unwrap_or_else(|| std::panic::panic_any("Cargo.lock package lacks a canonical name"));
        packages
            .entry(name.to_owned())
            .or_default()
            .push(block.to_owned());
    }
    packages
}

fn unique_package_block<'a>(packages: &'a PackageBlocks, name: &str) -> &'a str {
    let blocks = packages
        .get(name)
        .unwrap_or_else(|| std::panic::panic_any(format!("Cargo.lock lacks {name}")));
    assert_eq!(
        blocks.len(),
        1,
        "Cargo.lock must contain exactly one governed {name} package, found {}",
        blocks.len()
    );
    &blocks[0]
}

fn source_from_block<'a>(name: &str, block: &'a str) -> &'a str {
    block
        .lines()
        .find_map(|line| line.strip_prefix("source = \"")?.strip_suffix('"'))
        .unwrap_or_else(|| std::panic::panic_any(format!("{name} lacks a source in Cargo.lock")))
}

fn dependencies_from_block(block: &str) -> Vec<&str> {
    let mut dependencies = Vec::new();
    let mut in_dependencies = false;
    for line in block.lines() {
        if line == "dependencies = [" {
            in_dependencies = true;
            continue;
        }
        if in_dependencies && line == "]" {
            break;
        }
        if in_dependencies {
            let dependency = line
                .trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix("\","))
                .unwrap_or_else(|| {
                    std::panic::panic_any(format!("malformed Cargo.lock dependency line {line:?}"))
                });
            dependencies.push(dependency);
        }
    }
    dependencies
}

#[test]
fn every_consumed_franken_numpy_package_resolves_to_the_suite_pin() {
    let suite = read_repo_file("SUITE.lock");
    let pin = suite_pin(&suite, "franken_numpy");
    let lock = read_repo_file("Cargo.lock");
    let packages = package_blocks(&lock);

    for package in CONSUMED_FNP_PACKAGES {
        let block = unique_package_block(&packages, package);
        let source = source_from_block(package, block);
        assert!(
            source.starts_with("git+https://github.com/Dicklesworthstone/franken_numpy?rev="),
            "{package} is not sourced through the governed FrankenNumPy repository: {source}"
        );
        assert!(
            source.contains(&format!("?rev={pin}#")) && source.ends_with(&pin),
            "{package} resolves outside the FrankenNumPy SUITE.lock pin {pin}: {source}"
        );
    }
}

#[test]
fn fmn_core_consumes_only_the_dependency_free_rng_primitive() {
    let lock = read_repo_file("Cargo.lock");
    let packages = package_blocks(&lock);
    let core = unique_package_block(&packages, "fmn-core");
    let core_dependencies = dependencies_from_block(core);
    assert!(
        core_dependencies.contains(&"fnp-random-core"),
        "fmn-core does not depend on fnp-random-core: {core_dependencies:?}"
    );
    for forbidden in ["fnp-random", "fnp-ndarray", "rayon", "getrandom 0.4.3"] {
        assert!(
            !core_dependencies.contains(&forbidden),
            "fmn-core directly depends on forbidden RNG-adjacent package {forbidden}: {core_dependencies:?}"
        );
    }
    assert!(
        !packages.contains_key("fnp-random"),
        "full fnp-random entered the native Cargo.lock instead of the dependency-free core"
    );

    let random_core = unique_package_block(&packages, "fnp-random-core");
    assert!(
        dependencies_from_block(random_core).is_empty(),
        "fnp-random-core is no longer dependency-free"
    );
}

#[test]
fn lock_parser_preserves_multiple_versions_without_weakening_governed_uniqueness() {
    let synthetic = concat!(
        "# preamble\n[[package]]\nname = \"shared\"\nversion = \"1.0.0\"\n\n",
        "[[package]]\nname = \"shared\"\nversion = \"2.0.0\"\n\n",
        "[[package]]\nname = \"governed\"\nversion = \"3.0.0\"\n"
    );
    let packages = package_blocks(synthetic);
    assert_eq!(packages.get("shared").map(Vec::len), Some(2));
    assert_eq!(
        unique_package_block(&packages, "governed").lines().next(),
        Some("name = \"governed\"")
    );
}
