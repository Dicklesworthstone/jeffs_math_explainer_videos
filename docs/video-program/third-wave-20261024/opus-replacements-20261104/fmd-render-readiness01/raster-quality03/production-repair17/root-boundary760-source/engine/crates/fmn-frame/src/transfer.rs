//! Transfer functions, applied once, natively, at the defined point
//! (§14.1, D-23).
//!
//! The renderer works in linear light; sinks receive encoded bytes. The
//! encode happens exactly once — here — so the ffmpeg boundary's
//! "no obligatory `eq`" rule holds and no double-encode can exist.
//!
//! Certified arithmetic: fmn-core's [`fmn_core::color::srgb_oetf`]
//! rides `std` `powf`, which defers to platform libm and may differ
//! across glibc/macOS/WASM. Canonical RGBA bytes are part of the
//! certified artifact set, so this module's functions ride
//! [`fmn_dmath::pow`] instead (D-17), and the per-sample quantization
//! is a fixed round-half-up. On top of that, the binary16 → byte path
//! is a 65536-entry table built once from those functions: conversion
//! becomes pure table lookup — bit-exact on every platform, on every
//! thread count, with defined behavior for every bit pattern including
//! negatives, infinities, and NaN.

use std::sync::OnceLock;

use crate::half::f16_to_f64;

/// The sRGB opto-electronic transfer function (encode: linear →
/// encoded), per IEC 61966-2-1, over deterministic `fmn_dmath::pow`.
#[must_use]
pub fn srgb_encode(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * fmn_dmath::pow(linear, 1.0 / 2.4) - 0.055
    }
}

/// The sRGB electro-optical transfer function (decode: encoded →
/// linear), per IEC 61966-2-1, over deterministic `fmn_dmath::pow`.
#[must_use]
pub fn srgb_decode(encoded: f64) -> f64 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        fmn_dmath::pow((encoded + 0.055) / 1.055, 2.4)
    }
}

/// Quantize a nominal-range value to a byte: clamp to [0, 1] (NaN → 0),
/// then round half up. The one quantization rule of the certified
/// conversion path.
#[must_use]
pub fn quantize8(x: f64) -> u8 {
    let clamped = if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) };
    (clamped * 255.0 + 0.5).floor() as u8
}

/// The binary16 → byte conversion tables, indexed by raw f16 bits.
#[derive(Debug)]
pub struct TransferTables {
    srgb8: Box<[u8; 65536]>,
    linear8: Box<[u8; 65536]>,
}

impl TransferTables {
    /// Build both tables from the deterministic transfer functions.
    /// Every one of the 65536 bit patterns gets a defined byte.
    #[must_use]
    pub fn build() -> Self {
        let mut srgb8 = Box::new([0u8; 65536]);
        let mut linear8 = Box::new([0u8; 65536]);
        for bits in 0..=u16::MAX {
            let v = f16_to_f64(bits);
            let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
            srgb8[bits as usize] = quantize8(srgb_encode(v));
            linear8[bits as usize] = quantize8(v);
        }
        Self { srgb8, linear8 }
    }

    /// Linear-light f16 bits → sRGB-encoded byte (color channels).
    #[must_use]
    pub fn srgb8_from_f16(&self, bits: u16) -> u8 {
        self.srgb8[bits as usize]
    }

    /// Linear f16 bits → linearly scaled byte (the alpha channel:
    /// alpha is coverage, never gamma-encoded).
    #[must_use]
    pub fn linear8_from_f16(&self, bits: u16) -> u8 {
        self.linear8[bits as usize]
    }
}

/// The process-wide tables, built once on first use.
#[must_use]
pub fn tables() -> &'static TransferTables {
    static TABLES: OnceLock<TransferTables> = OnceLock::new();
    TABLES.get_or_init(TransferTables::build)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::half::f16_from_f32;

    #[test]
    fn oetf_eotf_round_trip() {
        for i in 0..=100 {
            let l = f64::from(i) / 100.0;
            let back = srgb_decode(srgb_encode(l));
            assert!((back - l).abs() < 1e-12, "l={l}: {back}");
        }
    }

    #[test]
    fn quantize_rounds_half_up_and_scrubs_nan() {
        assert_eq!(quantize8(0.0), 0);
        assert_eq!(quantize8(1.0), 255);
        assert_eq!(quantize8(0.5), 128); // 127.5 rounds up
        assert_eq!(quantize8(-3.0), 0);
        assert_eq!(quantize8(7.0), 255);
        assert_eq!(quantize8(f64::NAN), 0);
    }

    #[test]
    fn table_anchors() {
        let t = tables();
        assert_eq!(t.srgb8_from_f16(f16_from_f32(0.0)), 0);
        assert_eq!(t.srgb8_from_f16(f16_from_f32(1.0)), 255);
        assert_eq!(t.srgb8_from_f16(f16_from_f32(-1.0)), 0);
        assert_eq!(t.srgb8_from_f16(f16_from_f32(f32::INFINITY)), 255);
        assert_eq!(t.linear8_from_f16(f16_from_f32(0.5)), 128);
        // NaN bit patterns are defined: 0.
        assert_eq!(t.srgb8_from_f16(0x7e00), 0);
        assert_eq!(t.linear8_from_f16(0x7e00), 0);
    }
}
