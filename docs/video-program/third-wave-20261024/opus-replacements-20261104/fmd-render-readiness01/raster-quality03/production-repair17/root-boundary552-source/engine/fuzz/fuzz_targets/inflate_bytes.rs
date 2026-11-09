//! fm-ntp fuzz target: `fmn_codec::inflate_bytes` (raw DEFLATE).
//!
//! Resource-budget contract (§16.5/R14): for ANY input the parser must
//! either refuse with a typed `InflateError` or succeed within the
//! declared output budget — never hang, never overallocate. Hangs are
//! caught by the runner's per-input `-timeout`; over-allocation by the
//! assertion below (and by libFuzzer's `-rss_limit_mb`).
//!
//! The budget here is deliberately far below the parser's own ceiling so
//! coverage-guided mutation explores bomb-adjacent inputs cheaply.
#![no_main]

use libfuzzer_sys::fuzz_target;

/// The declared decompressed-size cap for this campaign (1 MiB).
const MAX_OUTPUT: usize = 1 << 20;

fuzz_target!(|data: &[u8]| {
    if let Ok(out) = fmn_codec::inflate_bytes(data, MAX_OUTPUT) {
        assert!(
            out.len() <= MAX_OUTPUT,
            "inflate_bytes returned {} bytes, over the declared {MAX_OUTPUT}-byte budget",
            out.len()
        );
    }
    // Err(_): a precise typed refusal — the contract's other allowed outcome.
});
