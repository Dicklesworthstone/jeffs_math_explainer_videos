//! fm-ntp fuzz target: `fmn_codec::decode_png`.
//!
//! Resource-budget contract (§16.5/R14): for ANY input the decoder must
//! either refuse with a typed `PngError` or succeed within the declared
//! `PngLimits` — never hang, never overallocate. An accepted image must
//! be exactly `width × height` pixels of RGBA8, inside the pixel budget.
//! Hangs are caught by the runner's per-input `-timeout`; over-allocation
//! by the assertions below (and by libFuzzer's `-rss_limit_mb`).
#![no_main]

use fmn_codec::PngLimits;
use libfuzzer_sys::fuzz_target;

/// Tight campaign budgets: 4 megapixels and 512 chunks — far below the
/// decoder's production defaults, so bomb steering stays cheap while the
/// whole chunk machinery (including Adam7 and 16-bit paths) is in reach.
const LIMITS: PngLimits = PngLimits {
    max_pixels: 1 << 22,
    max_chunks: 512,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(png) = fmn_codec::decode_png(data, &LIMITS) {
        let pixels = u64::from(png.width) * u64::from(png.height);
        assert!(
            pixels <= LIMITS.max_pixels,
            "decode_png accepted {}x{} = {pixels} pixels, over the declared {}-pixel budget",
            png.width,
            png.height,
            LIMITS.max_pixels
        );
        assert_eq!(
            png.rgba.len() as u64,
            pixels * 4,
            "decode_png RGBA buffer is {} bytes, expected width*height*4 = {}",
            png.rgba.len(),
            pixels * 4
        );
    }
    // Err(_): a precise typed refusal — the contract's other allowed outcome.
});
