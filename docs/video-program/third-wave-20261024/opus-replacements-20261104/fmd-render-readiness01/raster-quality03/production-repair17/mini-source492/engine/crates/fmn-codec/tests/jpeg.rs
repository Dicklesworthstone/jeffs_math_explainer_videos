//! fm-17m acceptance, slice C: JPEG decode conformance — baseline +
//! progressive, 4:4:4/4:2:2/4:2:0/4:4:0, restart markers, EXIF
//! orientation, the CMYK policy refusal, and hostile-input behavior.
//!
//! References are ImageMagick/libjpeg-turbo decodes of the committed
//! fixtures. JPEG decode is not bit-specified, but fmn-codec's islow
//! IDCT + triangle upsampling family tracks libjpeg within a small
//! bound, asserted here per fixture.

use fmn_codec::{JpegError, JpegLimits, decode_jpeg};

macro_rules! jfix {
    ($name:literal) => {
        (
            $name,
            include_bytes!(concat!("fixtures/jpeg/", $name, ".jpg")) as &[u8],
            include_bytes!(concat!("fixtures/jpeg/", $name, ".rgb")) as &[u8],
        )
    };
}

/// (name, jpeg bytes, reference RGB, max per-sample deviation).
const CORPUS: [(&str, &[u8], &[u8]); 10] = [
    jfix!("baseline_444"),
    jfix!("baseline_420"),
    jfix!("baseline_422"),
    jfix!("baseline_440"),
    jfix!("progressive_444"),
    jfix!("progressive_420"),
    jfix!("gray"),
    jfix!("restart_420"),
    jfix!("orient6_le"),
    jfix!("orient3_be"),
];

fn assert_close(name: &str, rgba: &[u8], reference_rgb: &[u8], tolerance: i32) {
    assert_eq!(rgba.len() / 4, reference_rgb.len() / 3, "{name}: size");
    let mut worst = 0i32;
    let mut worst_at = 0usize;
    for (i, (px, r)) in rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(reference_rgb.as_chunks::<3>().0)
        .enumerate()
    {
        assert_eq!(px[3], 255, "{name}: alpha");
        for ch in 0..3 {
            let d = (i32::from(px[ch]) - i32::from(r[ch])).abs();
            if d > worst {
                worst = d;
                worst_at = i;
            }
        }
    }
    assert!(
        worst <= tolerance,
        "{name}: worst deviation {worst} at pixel {worst_at} (tolerance {tolerance})"
    );
}

#[test]
fn decode_conformance_against_libjpeg_references() {
    let limits = JpegLimits::default();
    for (name, jpg, reference) in CORPUS {
        let d = decode_jpeg(jpg, &limits)
            .map_err(|e| format!("{name}: {e}"))
            .expect("corpus fixture decodes");
        // Subsampled chroma tolerates slightly more at sharp edges.
        let tolerance = if name.contains("444") || name.contains("gray") {
            2
        } else {
            4
        };
        assert_close(name, &d.rgba, reference, tolerance);
    }
}

#[test]
fn structure_is_reported() {
    let limits = JpegLimits::default();
    let d = decode_jpeg(CORPUS[0].1, &limits).unwrap();
    assert_eq!((d.width, d.height), (64, 48));
    assert_eq!(d.components, 3);
    assert!(!d.progressive);
    assert_eq!(d.orientation, 1);

    let d = decode_jpeg(CORPUS[5].1, &limits).unwrap();
    assert!(d.progressive, "progressive_420 flagged sequential");

    let d = decode_jpeg(CORPUS[6].1, &limits).unwrap();
    assert_eq!(d.components, 1);
}

#[test]
fn exif_orientation_is_honored_both_endians() {
    let limits = JpegLimits::default();
    // Orientation 6 (rotate 90° CW): dimensions swap.
    let d = decode_jpeg(CORPUS[8].1, &limits).unwrap();
    assert_eq!(d.orientation, 6);
    assert_eq!((d.width, d.height), (48, 64));
    // Orientation 3 (rotate 180°): dimensions keep.
    let d = decode_jpeg(CORPUS[9].1, &limits).unwrap();
    assert_eq!(d.orientation, 3);
    assert_eq!((d.width, d.height), (64, 48));
}

#[test]
fn cmyk_is_a_named_policy_refusal() {
    let cmyk = include_bytes!("fixtures/jpeg/cmyk.jpg");
    assert_eq!(
        decode_jpeg(cmyk, &JpegLimits::default()),
        Err(JpegError::CmykUnsupported)
    );
}

#[test]
fn pixel_budget_is_checked_at_sof() {
    // Patch the SOF dimensions of a valid fixture to 65535×65535.
    let mut jpg = CORPUS[0].1.to_vec();
    let sof = jpg
        .windows(2)
        .position(|w| w == [0xff, 0xc0])
        .expect("SOF0");
    // SOF: marker(2) len(2) precision(1) height(2) width(2) ...
    jpg[sof + 5..sof + 9].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    let limits = JpegLimits::default();
    assert_eq!(
        decode_jpeg(&jpg, &limits),
        Err(JpegError::TooLarge {
            max_pixels: limits.max_pixels
        })
    );
}

#[test]
fn oversampling_is_a_named_refusal() {
    // Patch the first component's sampling factors to 4x1.
    let mut jpg = CORPUS[0].1.to_vec();
    let sof = jpg
        .windows(2)
        .position(|w| w == [0xff, 0xc0])
        .expect("SOF0");
    jpg[sof + 11] = 0x41;
    assert_eq!(
        decode_jpeg(&jpg, &JpegLimits::default()),
        Err(JpegError::UnsupportedSampling)
    );
}

#[test]
fn hostile_inputs_are_typed_refusals() {
    let limits = JpegLimits::default();
    assert_eq!(decode_jpeg(b"", &limits), Err(JpegError::NotJpeg));
    assert_eq!(
        decode_jpeg(b"\x89PNG\r\n", &limits),
        Err(JpegError::NotJpeg)
    );

    // Truncations at every early boundary parse-fail, never hang.
    let good = CORPUS[0].1;
    for cut in [2usize, 4, 20, 100, good.len() / 2, good.len() - 3] {
        assert!(
            decode_jpeg(&good[..cut], &limits).is_err(),
            "prefix {cut} accepted"
        );
    }

    // Byte soup behind a valid SOI.
    let mut soup = vec![0xff, 0xd8];
    let mut state = 12345u32;
    for _ in 0..4096 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        soup.push((state >> 24) as u8);
    }
    let _ = decode_jpeg(&soup, &limits);

    // Bit flips across the entropy region must error or decode — never
    // panic, never hang (a cheap deterministic fuzz sweep).
    for i in (good.len() / 2..good.len()).step_by(97) {
        let mut mutated = good.to_vec();
        mutated[i] ^= 0x40;
        let _ = decode_jpeg(&mutated, &limits);
    }
}

/// Regression for the W10 fuzz campaign's first catch (fm-t1v): hostile
/// table selectors. DHT/DQT validate their own ids, but the SOS scan
/// component selectors and the SOF quantization selector indexed the
/// four-slot table arrays unchecked — an out-of-range nibble was an
/// out-of-bounds read, not a typed refusal.
#[test]
fn hostile_table_selectors_are_typed_refusals() {
    let limits = JpegLimits::default();
    let good = CORPUS[0].1;

    // Find a marker's segment payload start (past marker + length).
    let find_segment = |marker: u8| -> Option<usize> {
        let mut pos = 2usize;
        while pos + 4 <= good.len() {
            if good[pos] != 0xFF {
                return None;
            }
            let m = good[pos + 1];
            let len = usize::from(u16::from_be_bytes([good[pos + 2], good[pos + 3]]));
            if m == marker {
                return Some(pos + 4);
            }
            if m == 0xDA {
                return None; // SOS: scan data follows, stop walking
            }
            pos += 2 + len;
        }
        None
    };

    // SOF: first component's quantization selector → 14.
    let sof = find_segment(0xC0).expect("baseline fixture has SOF0");
    // payload: precision(1) height(2) width(2) ncomp(1) then id/hv/tq…
    let tq_at = sof + 6 + 2;
    let mut mutated = good.to_vec();
    mutated[tq_at] = 0x0E;
    assert_eq!(
        decode_jpeg(&mutated, &limits),
        Err(JpegError::BadTable("frame quantization selector"))
    );

    // SOS: first scan component's DC/AC selectors → 14/14.
    let sos = find_segment(0xDA).expect("baseline fixture has SOS");
    // payload: ns(1) then id/tables per component…
    let tables_at = sos + 2;
    let mut mutated = good.to_vec();
    mutated[tables_at] = 0xEE;
    assert_eq!(
        decode_jpeg(&mutated, &limits),
        Err(JpegError::BadTable("scan table selector"))
    );
}
