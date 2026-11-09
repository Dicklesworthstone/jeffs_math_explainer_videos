//! fm-tdp fuzz target: namespace-name validation and digest → object-path
//! derivation.
//!
//! Invariants (store.rs module docs): an accepted name is 1..=64 bytes of
//! `[a-z0-9_-]` starting alphanumeric — no dots, no separators, so no path
//! traversal; and `object_relative_path` derives ONLY from lowercase hex of
//! the digest, always rooted at `objects`, exactly two components deep below
//! it. Both hold for arbitrary input or the call refuses/is independent of
//! it — never a panic, never a path outside the managed tree.
#![no_main]

use libfuzzer_sys::fuzz_target;

fn assert_hex(component: &std::ffi::OsStr, len: usize) {
    let s = component
        .to_str()
        .unwrap_or_else(|| panic!("path component is not UTF-8"));
    assert_eq!(s.len(), len, "hex component length");
    assert!(
        s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "path component {s} is not lowercase hex"
    );
}

fuzz_target!(|data: &[u8]| {
    // Namespace-name validation: pure accept/refuse over arbitrary bytes.
    if let Ok(name) = std::str::from_utf8(data) {
        let accepted = fmn_cache::validate_namespace_name(name).is_ok();
        if accepted {
            assert!(!name.is_empty());
            assert!(name.len() <= 64);
            let first = name.as_bytes()[0];
            assert!(first.is_ascii_lowercase() || first.is_ascii_digit());
            assert!(
                name.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
            );
            assert!(!name.contains(['.', '/', '\\']));
        }
    }

    // Object-path derivation: a function of digest hex and nothing else.
    let digest = fmn_hash::sha256(data);
    let path = fmn_cache::object_relative_path(&digest);
    let mut components = path.components();
    let high = components.next().expect("high nibble component");
    let low = components.next().expect("low nibble component");
    assert_hex(high.as_os_str(), 2);
    assert_hex(low.as_os_str(), 62);
    assert!(components.next().is_none(), "no trailing components");

    // The two components concatenate back to the exact digest hex the path
    // was derived from — nothing entered the path but the digest.
    let joined = format!(
        "{}{}",
        high.as_os_str().to_string_lossy(),
        low.as_os_str().to_string_lossy()
    );
    assert_eq!(joined, digest.to_hex());
});
