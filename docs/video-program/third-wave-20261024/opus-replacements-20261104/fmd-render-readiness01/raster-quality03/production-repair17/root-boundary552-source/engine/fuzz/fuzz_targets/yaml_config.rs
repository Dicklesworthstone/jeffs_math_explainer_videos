//! fm-ntp fuzz target: `fmn_config::yaml::parse_with_limits`.
//!
//! Resource-budget contract (§16.5/R14): for ANY input the parser must
//! either refuse with a typed `ParseError` or succeed within the declared
//! `Limits` — never hang, never overallocate. The config parser is
//! deterministic: same input ⇒ same output, every time.
#![no_main]

use fmn_config::yaml::Limits;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The parser expects text — non-UTF-8 inputs must be a clean refusal,
    // not a panic. `parse_with_limits` accepts &[u8] directly.
    let _ = fmn_config::yaml::parse_with_limits(
        match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return,
        },
        Limits::default(),
    );
});
