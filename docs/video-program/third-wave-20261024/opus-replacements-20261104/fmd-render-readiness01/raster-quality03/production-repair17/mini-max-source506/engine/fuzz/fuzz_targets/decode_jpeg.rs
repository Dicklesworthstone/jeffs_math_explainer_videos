//! fm-ntp fuzz target: `fmn_codec::decode_jpeg`.
//!
//! Resource-budget contract (§16.5/R14): for ANY input the decoder must
//! either refuse with a typed `JpegError` or succeed within the declared
//! `JpegLimits` — never hang, never overallocate. An accepted image must
//! be exactly `width × height` pixels of RGBA8, inside the pixel budget.
//! Hangs are caught by the runner's per-input `-timeout`; over-allocation
//! by the assertions below (and by libFuzzer's `-rss_limit_mb`).
#![no_main]

use fmn_codec::JpegLimits;
use libfuzzer_sys::fuzz_target;

/// Tight campaign budget: 4 megapixels — far below the decoder's
/// production default, so bomb steering stays cheap while baseline and
/// progressive scans, restarts, and EXIF orientation stay in reach.
const LIMITS: JpegLimits = JpegLimits {
    max_pixels: 1 << 22,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(jpeg) = fmn_codec::decode_jpeg(data, &LIMITS) {
        let pixels = u64::from(jpeg.width) * u64::from(jpeg.height);
        assert!(
            pixels <= LIMITS.max_pixels,
            "decode_jpeg accepted {}x{} = {pixels} pixels, over the declared {}-pixel budget",
            jpeg.width,
            jpeg.height,
            LIMITS.max_pixels
        );
        assert_eq!(
            jpeg.rgba.len() as u64,
            pixels * 4,
            "decode_jpeg RGBA buffer is {} bytes, expected width*height*4 = {}",
            jpeg.rgba.len(),
            pixels * 4
        );
    }
    // Err(_): a precise typed refusal — the contract's other allowed outcome.
});
