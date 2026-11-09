//! Direct binary16 output: independent precision oracles and legacy byte parity.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use fmn_frame::convert::{
    rgba_to_nv12, rgba_to_p010, rgba16f_to_bgra8, rgba16f_to_nv12, rgba16f_to_nv12_threaded,
    rgba16f_to_p010, rgba16f_to_p010_threaded, rgba16f_to_rgba8, swap_rb8,
};
use fmn_frame::half::{f16_from_f32, f16_to_f64};
use fmn_frame::transfer::{srgb_decode, srgb_encode};
use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameError, FrameLayout, PixelFormat};

fn frame(format: PixelFormat, width: u32, height: u32, alignment: usize) -> FrameBuffer {
    let mut buffer = FrameBuffer::new(
        FrameLayout::with_row_alignment(format, width, height, alignment).unwrap(),
    );
    buffer.as_bytes_mut().fill(0xA5);
    buffer
}

fn set_pixel(frame: &mut FrameBuffer, x: usize, y: usize, bits: [u16; 4]) {
    let at = y * frame.layout().stride(0) + x * 8;
    for (index, value) in bits.into_iter().enumerate() {
        frame.plane_mut(0)[at + index * 2..at + index * 2 + 2]
            .copy_from_slice(&value.to_le_bytes());
    }
}

fn code(frame: &FrameBuffer, plane: usize, x: usize, y: usize) -> u16 {
    let at = y * frame.layout().stride(plane) + x * 2;
    let bytes = &frame.plane(plane)[at..at + 2];
    let sample = u16::from_le_bytes([bytes[0], bytes[1]]);
    assert_eq!(sample & 63, 0, "P010 low six bits must be zero");
    sample >> 6
}

fn padding_is_untouched(frame: &FrameBuffer) {
    let layout = frame.layout();
    for plane in 0..layout.format().plane_count() {
        let payload = layout
            .format()
            .min_row_bytes(layout.width(), plane)
            .unwrap();
        let stride = layout.stride(plane);
        for row in frame.plane(plane).chunks_exact(stride) {
            assert!(row[payload..].iter().all(|byte| *byte == 0xA5));
        }
    }
}

#[test]
fn nv12_fusion_preserves_legacy_bytes_for_every_binary16_pattern() {
    let mut source = frame(PixelFormat::Rgba16F, 256, 256, 64);
    for bits in 0..=u16::MAX {
        set_pixel(
            &mut source,
            usize::from(bits) % 256,
            usize::from(bits) / 256,
            [bits, bits.rotate_left(5), !bits, bits.rotate_right(3)],
        );
    }
    let before = source.as_bytes().to_vec();
    let mut rgba = frame(PixelFormat::Rgba8, 256, 256, 64);
    rgba16f_to_rgba8(&source, &mut rgba).unwrap();
    for range in [ColorRange::Limited, ColorRange::Full] {
        for siting in [ChromaSiting::Left, ChromaSiting::Center] {
            let mut legacy = frame(PixelFormat::Nv12, 256, 256, 64);
            let mut fused = frame(PixelFormat::Nv12, 256, 256, 64);
            rgba_to_nv12(&rgba, &mut legacy, range, siting).unwrap();
            rgba16f_to_nv12(&source, &mut fused, range, siting).unwrap();
            assert_eq!(legacy.as_bytes(), fused.as_bytes(), "{range:?}, {siting:?}");
        }
    }
    assert_eq!(source.as_bytes(), before);
}

#[test]
fn repeated_quads_convert_exactly_like_isolated_quads() {
    // Runs of byte-identical quads (the converters' reuse path) interleaved with
    // quads that differ from their predecessor in exactly one pixel, one byte —
    // including an alpha-only byte the transfer ignores. Every quad must encode
    // exactly as the same four pixels converted alone in a 2x2 frame, where no
    // predecessor exists to reuse.
    let (width, height) = (24usize, 6usize);
    let base = [
        f16_from_f32(0.25),
        f16_from_f32(0.5),
        f16_from_f32(0.75),
        f16_from_f32(1.0),
    ];
    let mut source = frame(PixelFormat::Rgba16F, width as u32, height as u32, 64);
    for y in 0..height {
        for x in 0..width {
            let mut bits = base;
            let quad = (x / 2, y / 2);
            if quad.0 % 3 == 2 && (x + y) % 4 == 1 {
                // One pixel of every third quad differs in one channel.
                let channel = (quad.0 + quad.1) % 4;
                bits[channel] ^= 1;
            }
            set_pixel(&mut source, x, y, bits);
        }
    }
    let read = |frame: &FrameBuffer, x: usize, y: usize| -> [u16; 4] {
        let at = y * frame.layout().stride(0) + x * 8;
        std::array::from_fn(|k| {
            let bytes = &frame.plane(0)[at + 2 * k..at + 2 * k + 2];
            u16::from_le_bytes([bytes[0], bytes[1]])
        })
    };
    for siting in [ChromaSiting::Left, ChromaSiting::Center] {
        let mut nv12 = frame(PixelFormat::Nv12, width as u32, height as u32, 64);
        let mut p010 = frame(PixelFormat::P010, width as u32, height as u32, 64);
        rgba16f_to_nv12(&source, &mut nv12, ColorRange::Limited, siting).unwrap();
        rgba16f_to_p010(&source, &mut p010, ColorRange::Limited, siting).unwrap();
        for qy in 0..height / 2 {
            for qx in 0..width / 2 {
                let mut alone = frame(PixelFormat::Rgba16F, 2, 2, 64);
                for dy in 0..2 {
                    for dx in 0..2 {
                        set_pixel(&mut alone, dx, dy, read(&source, 2 * qx + dx, 2 * qy + dy));
                    }
                }
                let mut nv12_alone = frame(PixelFormat::Nv12, 2, 2, 64);
                let mut p010_alone = frame(PixelFormat::P010, 2, 2, 64);
                rgba16f_to_nv12(&alone, &mut nv12_alone, ColorRange::Limited, siting).unwrap();
                rgba16f_to_p010(&alone, &mut p010_alone, ColorRange::Limited, siting).unwrap();
                for dy in 0..2 {
                    for dx in 0..2 {
                        let (x, y) = (2 * qx + dx, 2 * qy + dy);
                        assert_eq!(
                            nv12.plane(0)[y * nv12.layout().stride(0) + x],
                            nv12_alone.plane(0)[dy * nv12_alone.layout().stride(0) + dx],
                            "NV12 luma at ({x}, {y}), {siting:?}"
                        );
                        assert_eq!(
                            code(&p010, 0, x, y),
                            code(&p010_alone, 0, dx, dy),
                            "P010 luma at ({x}, {y}), {siting:?}"
                        );
                    }
                }
                for channel in 0..2 {
                    assert_eq!(
                        nv12.plane(1)[qy * nv12.layout().stride(1) + 2 * qx + channel],
                        nv12_alone.plane(1)[channel],
                        "NV12 chroma {channel} of quad ({qx}, {qy}), {siting:?}"
                    );
                    assert_eq!(
                        code(&p010, 1, 2 * qx + channel, qy),
                        code(&p010_alone, 1, channel, 0),
                        "P010 chroma {channel} of quad ({qx}, {qy}), {siting:?}"
                    );
                }
            }
        }
        padding_is_untouched(&nv12);
        padding_is_untouched(&p010);
    }
}

#[test]
fn direct_converters_preserve_unequal_strides_orientation_and_padding() {
    for width in [2, 6, 18] {
        let mut source = frame(PixelFormat::Rgba16F, width, 6, 128);
        for y in 0..6 {
            for x in 0..width as usize {
                set_pixel(
                    &mut source,
                    x,
                    y,
                    [
                        f16_from_f32(x as f32 / width as f32),
                        f16_from_f32(y as f32 / 6.0),
                        f16_from_f32(0.18),
                        f16_from_f32((x + y) as f32 / (width as f32 + 6.0)),
                    ],
                );
            }
        }
        let original = source.as_bytes().to_vec();
        let mut rgba = frame(PixelFormat::Rgba8, width, 6, 64);
        rgba16f_to_rgba8(&source, &mut rgba).unwrap();
        let mut bgra = frame(PixelFormat::Bgra8, width, 6, 64);
        let mut legacy_bgra = frame(PixelFormat::Bgra8, width, 6, 64);
        rgba16f_to_bgra8(&source, &mut bgra).unwrap();
        swap_rb8(&rgba, &mut legacy_bgra).unwrap();
        assert_eq!(bgra.as_bytes(), legacy_bgra.as_bytes());
        padding_is_untouched(&bgra);
        for siting in [ChromaSiting::Center, ChromaSiting::Left] {
            let mut legacy = frame(PixelFormat::Nv12, width, 6, 64);
            let mut nv12 = frame(PixelFormat::Nv12, width, 6, 64);
            rgba_to_nv12(&rgba, &mut legacy, ColorRange::Limited, siting).unwrap();
            rgba16f_to_nv12(&source, &mut nv12, ColorRange::Limited, siting).unwrap();
            assert_eq!(nv12.as_bytes(), legacy.as_bytes());
            padding_is_untouched(&nv12);
            let mut p010 = frame(PixelFormat::P010, width, 6, 64);
            rgba16f_to_p010(&source, &mut p010, ColorRange::Limited, siting).unwrap();
            padding_is_untouched(&p010);
            assert!(code(&p010, 0, 0, 0) < code(&p010, 0, 0, 5));
            assert!(code(&p010, 0, 0, 0) < code(&p010, 0, width as usize - 1, 0));
        }
        assert_eq!(source.as_bytes(), original);
    }
}

#[test]
fn p010_keeps_more_than_eight_bits_of_a_half_float_gray_ramp() {
    let width = 4096usize;
    let mut source = frame(PixelFormat::Rgba16F, width as u32, 2, 64);
    for x in 0..width {
        let bits = f16_from_f32(srgb_decode(x as f64 / (width - 1) as f64) as f32);
        for y in 0..2 {
            set_pixel(&mut source, x, y, [bits, bits, bits, 0x3c00]);
        }
    }
    let mut direct = frame(PixelFormat::P010, width as u32, 2, 64);
    let mut legacy = frame(PixelFormat::P010, width as u32, 2, 64);
    let mut rgba = frame(PixelFormat::Rgba8, width as u32, 2, 64);
    rgba16f_to_rgba8(&source, &mut rgba).unwrap();
    rgba_to_p010(
        &rgba,
        &mut legacy,
        ColorRange::Limited,
        ChromaSiting::Center,
    )
    .unwrap();
    rgba16f_to_p010(
        &source,
        &mut direct,
        ColorRange::Limited,
        ChromaSiting::Center,
    )
    .unwrap();
    let old_levels: BTreeSet<_> = (0..width).map(|x| code(&legacy, 0, x, 0)).collect();
    let new_levels: BTreeSet<_> = (0..width).map(|x| code(&direct, 0, x, 0)).collect();
    assert_eq!(old_levels.len(), 256);
    assert!(
        new_levels.len() > 800,
        "only {} luma levels",
        new_levels.len()
    );
    assert_eq!(
        (code(&direct, 0, 0, 0), code(&direct, 0, width - 1, 0)),
        (64, 940)
    );
    for x in 0..width {
        assert_eq!(code(&direct, 1, x, 0), 512);
        let at = x * 8;
        let bits = u16::from_le_bytes([source.plane(0)[at], source.plane(0)[at + 1]]);
        let back = srgb_decode(f64::from(code(&direct, 0, x, 0) - 64) / 876.0);
        assert!((back - f16_to_f64(bits)).abs() <= 0.0015);
        if x != 0 {
            assert!(code(&direct, 0, x - 1, 0) <= code(&direct, 0, x, 0));
        }
    }
}

#[test]
fn p010_uniform_colors_obey_an_independent_inverse_matrix_error_bound() {
    let mut source = frame(PixelFormat::Rgba16F, 2, 2, 16);
    let mut output = frame(PixelFormat::P010, 2, 2, 16);
    for r in 0..=16 {
        for g in 0..=16 {
            for b in 0..=16 {
                let bits = [r, g, b].map(|v| f16_from_f32(v as f32 / 16.0));
                for y in 0..2 {
                    for x in 0..2 {
                        set_pixel(&mut source, x, y, [bits[0], bits[1], bits[2], 0x3c00]);
                    }
                }
                rgba16f_to_p010(
                    &source,
                    &mut output,
                    ColorRange::Limited,
                    ChromaSiting::Center,
                )
                .unwrap();
                let y = (f64::from(code(&output, 0, 0, 0)) - 64.0) / 876.0;
                let cb = (f64::from(code(&output, 1, 0, 0)) - 512.0) / 896.0;
                let cr = (f64::from(code(&output, 1, 1, 0)) - 512.0) / 896.0;
                let red = y + 1.5748 * cr;
                let blue = y + 1.8556 * cb;
                let green = (y - 0.2126 * red - 0.0722 * blue) / 0.7152;
                for (actual, bits) in [red, green, blue].into_iter().zip(bits) {
                    let expected = srgb_encode(f16_to_f64(bits));
                    assert!(
                        (actual - expected).abs() <= 2.0 / 1023.0,
                        "rgb=({r},{g},{b}): reconstructed {actual}, expected {expected}"
                    );
                }
                assert!((64..=940).contains(&code(&output, 0, 0, 0)));
                assert!((64..=960).contains(&code(&output, 1, 0, 0)));
                assert!((64..=960).contains(&code(&output, 1, 1, 0)));
            }
        }
    }
}

#[test]
fn p010_siting_uses_unquantized_chroma_and_left_ignores_the_right_column() {
    let mut source = frame(PixelFormat::Rgba16F, 2, 2, 16);
    for y in 0..2 {
        set_pixel(&mut source, 0, y, [0, 0, 0, 0x3c00]);
        set_pixel(&mut source, 1, y, [0x3c00, 0, 0, 0x3c00]);
    }
    let mut left = frame(PixelFormat::P010, 2, 2, 16);
    let mut center = frame(PixelFormat::P010, 2, 2, 16);
    rgba16f_to_p010(&source, &mut left, ColorRange::Limited, ChromaSiting::Left).unwrap();
    rgba16f_to_p010(
        &source,
        &mut center,
        ColorRange::Limited,
        ChromaSiting::Center,
    )
    .unwrap();
    assert_eq!(left.plane(0), center.plane(0));
    assert_eq!((code(&left, 1, 0, 0), code(&left, 1, 1, 0)), (512, 512));
    // Average black and red in encoded space: cb = -Kr/(1-Kb)/4,
    // cr = 1/4, independently of the fixed-point implementation.
    let expected_cb = (512.0_f64 - 896.0 * 0.2126 / (1.0 - 0.0722) / 4.0).round();
    assert!((f64::from(code(&center, 1, 0, 0)) - expected_cb).abs() <= 1.0);
    assert_eq!(code(&center, 1, 1, 0), 736);
}

#[test]
fn nonfinite_values_and_coverage_alpha_do_not_pollute_p010_channels() {
    let mut source = frame(PixelFormat::Rgba16F, 2, 2, 16);
    let mut output = frame(PixelFormat::P010, 2, 2, 16);
    for (value, expected) in [
        (0x0000, 64),
        (0x8000, 64),
        (0xbc00, 64),
        (0xfc00, 64),
        (0x7e00, 64),
        (0xfe00, 64),
        (0x7c00, 940),
        (0x4000, 940),
    ] {
        for y in 0..2 {
            for x in 0..2 {
                set_pixel(&mut source, x, y, [value, value, value, 0x7e00]);
            }
        }
        rgba16f_to_p010(
            &source,
            &mut output,
            ColorRange::Limited,
            ChromaSiting::Center,
        )
        .unwrap();
        assert_eq!(code(&output, 0, 0, 0), expected);
        assert_eq!((code(&output, 1, 0, 0), code(&output, 1, 1, 0)), (512, 512));
    }
}

#[test]
fn invalid_layouts_and_full_range_p010_refuse_before_any_write() {
    let source = frame(PixelFormat::Rgba16F, 2, 2, 16);
    let wrong_source = frame(PixelFormat::Rgba8, 2, 2, 16);
    for format in [PixelFormat::Nv12, PixelFormat::P010, PixelFormat::Bgra8] {
        let convert = |src: &FrameBuffer, dst: &mut FrameBuffer| match format {
            PixelFormat::Nv12 => rgba16f_to_nv12(src, dst, ColorRange::Limited, ChromaSiting::Left),
            PixelFormat::P010 => rgba16f_to_p010(src, dst, ColorRange::Limited, ChromaSiting::Left),
            _ => rgba16f_to_bgra8(src, dst),
        };
        let mut target = frame(format, 2, 2, 16);
        let before = target.as_bytes().to_vec();
        assert!(matches!(
            convert(&wrong_source, &mut target),
            Err(FrameError::FormatMismatch { .. })
        ));
        assert_eq!(target.as_bytes(), before);
        let mut target = frame(format, 4, 2, 16);
        let before = target.as_bytes().to_vec();
        assert_eq!(
            convert(&source, &mut target),
            Err(FrameError::DimensionMismatch)
        );
        assert_eq!(target.as_bytes(), before);
        let mut target = frame(PixelFormat::Rgba8, 2, 2, 16);
        let before = target.as_bytes().to_vec();
        assert!(matches!(
            convert(&source, &mut target),
            Err(FrameError::FormatMismatch { .. })
        ));
        assert_eq!(target.as_bytes(), before);
    }
    let mut target = frame(PixelFormat::P010, 2, 2, 16);
    let before = target.as_bytes().to_vec();
    assert!(matches!(
        rgba16f_to_p010(&source, &mut target, ColorRange::Full, ChromaSiting::Left),
        Err(FrameError::UnsupportedConversion(_))
    ));
    assert_eq!(target.as_bytes(), before);
}

#[test]
fn threaded_yuv_conversion_is_byte_identical_at_every_thread_count() {
    // 330 row pairs: bands split unevenly at most counts; padded strides and
    // flat runs (memo hits across what become band seams) are both present.
    let (width, height) = (322, 660);
    let mut source = frame(PixelFormat::Rgba16F, width, height, 64);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let bits = if (y / 7 + x / 13) % 3 == 0 {
                [0x3C00, 0x3800, 0x3400, 0x3C00]
            } else {
                let seed = (x * 7919 + y * 104_729) as u16;
                [seed, seed.rotate_left(5), !seed, 0x3C00]
            };
            set_pixel(&mut source, x, y, bits);
        }
    }
    for siting in [ChromaSiting::Left, ChromaSiting::Center] {
        let mut nv12 = frame(PixelFormat::Nv12, width, height, 64);
        let mut p010 = frame(PixelFormat::P010, width, height, 64);
        rgba16f_to_nv12(&source, &mut nv12, ColorRange::Limited, siting).unwrap();
        rgba16f_to_p010(&source, &mut p010, ColorRange::Limited, siting).unwrap();
        for threads in [0, 1, 2, 3, 4, 7, 10, 16, 64] {
            let mut nv12_threaded = frame(PixelFormat::Nv12, width, height, 64);
            let mut p010_threaded = frame(PixelFormat::P010, width, height, 64);
            rgba16f_to_nv12_threaded(
                &source,
                &mut nv12_threaded,
                ColorRange::Limited,
                siting,
                threads,
            )
            .unwrap();
            rgba16f_to_p010_threaded(
                &source,
                &mut p010_threaded,
                ColorRange::Limited,
                siting,
                threads,
            )
            .unwrap();
            assert_eq!(
                nv12_threaded.as_bytes(),
                nv12.as_bytes(),
                "NV12 at {threads} threads"
            );
            assert_eq!(
                p010_threaded.as_bytes(),
                p010.as_bytes(),
                "P010 at {threads} threads"
            );
            padding_is_untouched(&nv12_threaded);
            padding_is_untouched(&p010_threaded);
        }
    }
}
