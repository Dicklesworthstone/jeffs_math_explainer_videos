//! fm-ntp fuzz target: `fmd_math::parse` (re-exported through `fmn_tex::Engine`).
//!
//! Resource-budget contract (§16.5/R14): for ANY input the parser must
//! either refuse with a typed `MathError` or produce a `Node` — never
//! hang, never overallocate. The TeX grammar is ambiguous by design;
//! fmd-math's ratchet governs coverage.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };
    // The bare parse surface is the strictest: it does not bind
    // fmd-math's bundled font pack. The Engine surface (fmn_tex::Engine)
    // has a fuller contract; for fuzzer purposes parse's error-class
    // shape is what we probe.
    let _ = fmd_math::parse(s);
});
