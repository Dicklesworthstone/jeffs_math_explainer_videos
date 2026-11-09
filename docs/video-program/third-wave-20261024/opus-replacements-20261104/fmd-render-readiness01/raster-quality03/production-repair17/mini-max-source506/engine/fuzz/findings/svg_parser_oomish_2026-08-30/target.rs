//! fm-ntp fuzz target: `fmn_geom::svg::SvgDocument::parse_with_limits`.
//!
//! Resource-budget contract (§16.5/R14): for ANY input the parser must
//! either refuse with a typed `SvgError` or succeed within the declared
//! `SvgLimits` — never hang, never overallocate. Hangs are caught by the
//! runner's per-input `-timeout`; over-allocation by the limits.
//!
//! The fmn-geom SVG parser is sovereign (no svgelements dependency), so
//! the test surface covers the public typestate (defs/uses) including
//! `viewBox`, transforms, paths, and use/defs.
#![no_main]

use fmn_geom::svg::{SvgDocument, SvgLimits};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The parser is total: it must always return Err or Ok, never panic.
    let _ = SvgDocument::parse_with_limits(data, &SvgLimits::default());
});
