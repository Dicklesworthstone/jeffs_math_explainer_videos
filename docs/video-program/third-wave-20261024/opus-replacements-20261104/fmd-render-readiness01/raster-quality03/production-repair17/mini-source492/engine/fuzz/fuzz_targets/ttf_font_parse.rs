//! fm-ntp fuzz target: `fmd_font::Font::parse`.
//!
//! Resource-budget contract (§16.5/R14): for ANY input the font parser
//! must either refuse with a typed `FontError` or produce a `Font` — never
//! panic, never hang. fmd-font is the sovereign parser; this target
//! exercises the public typestate (sfnt header, table directory, head/glyf
//! parsing, and the rejected-malformed-input path).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The parser is total: Err or Ok, never panic. `parse` consumes the
    // full byte buffer (it owns the table directory); the typed refusal
    // contract shapes what fmd-font refuses for malformed TTF.
    let _ = fmd_font::Font::parse(data.to_vec());
});
