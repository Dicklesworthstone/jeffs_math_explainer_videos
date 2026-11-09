//! The engine-equivalence measurement (§16.3).
//!
//! §10.1 holds annex engines "to an explicit, versioned **visual-equivalence
//! budget** against certified reference frames (perceptual metrics plus
//! max-error bounds)". This module computes the numbers that budget would be
//! written in. It deliberately does *not* pick the thresholds: a budget
//! authored from one spike frame on one GPU would be a guess wearing a number,
//! and §16.3's budget is W5's to set once the real corpus exists. What the
//! spike owes is the measurement and its shape.

use crate::cpu::Surface;

/// Per-channel and perceptual divergence between two renders of one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Divergence {
    /// Largest absolute difference over all linear-light components.
    pub max_abs: f64,
    /// Mean absolute difference over all linear-light components.
    pub mean_abs: f64,
    /// Root-mean-square difference over all linear-light components.
    pub rms: f64,
    /// Largest difference after sRGB encoding and 8-bit quantization — the
    /// number a reviewer can actually see, and the one a Look Gallery verdict
    /// would be argued over.
    pub max_u8: u8,
    /// How many 8-bit components differ at all.
    pub differing_u8: usize,
    /// Total 8-bit components compared.
    pub total_u8: usize,
    /// Global SSIM over the sRGB luma plane — the §16.3 "smoke alarm", never a
    /// hard gate.
    pub ssim: f64,
}

impl Divergence {
    /// The fraction of 8-bit components that differ at all.
    pub fn differing_fraction(&self) -> f64 {
        if self.total_u8 == 0 {
            return 0.0;
        }
        self.differing_u8 as f64 / self.total_u8 as f64
    }

    /// A one-line summary for the report and the runner's stdout.
    pub fn summary(&self) -> String {
        format!(
            "max_abs {:.3e}  mean_abs {:.3e}  rms {:.3e}  max_u8 {}  differing {}/{} ({:.4}%)  ssim {:.6}",
            self.max_abs,
            self.mean_abs,
            self.rms,
            self.max_u8,
            self.differing_u8,
            self.total_u8,
            100.0 * self.differing_fraction(),
            self.ssim,
        )
    }
}

/// Compare two surfaces of identical dimensions.
///
/// # Panics
/// If the surfaces differ in size — that is a harness bug, not a divergence,
/// and silently comparing a prefix would hide it.
pub fn diverge(reference: &Surface, other: &Surface) -> Divergence {
    assert_eq!(reference.width, other.width, "surface widths differ");
    assert_eq!(reference.height, other.height, "surface heights differ");
    assert_eq!(reference.pixels.len(), other.pixels.len());

    let mut max_abs = 0.0f64;
    let mut sum_abs = 0.0f64;
    let mut sum_sq = 0.0f64;
    for (a, b) in reference.pixels.iter().zip(other.pixels.iter()) {
        let d = (*a as f64 - *b as f64).abs();
        if d > max_abs {
            max_abs = d;
        }
        sum_abs += d;
        sum_sq += d * d;
    }
    let n = reference.pixels.len().max(1) as f64;

    let ra = reference.to_srgb8();
    let rb = other.to_srgb8();
    let mut max_u8 = 0u8;
    let mut differing = 0usize;
    for (a, b) in ra.iter().zip(rb.iter()) {
        let d = a.abs_diff(*b);
        if d > 0 {
            differing += 1;
        }
        if d > max_u8 {
            max_u8 = d;
        }
    }

    Divergence {
        max_abs,
        mean_abs: sum_abs / n,
        rms: (sum_sq / n).sqrt(),
        max_u8,
        differing_u8: differing,
        total_u8: ra.len(),
        ssim: ssim_luma(
            &ra,
            &rb,
            reference.width as usize,
            reference.height as usize,
        ),
    }
}

/// Global SSIM over the Rec. 709 luma of two sRGB8 RGBA buffers.
///
/// Global rather than windowed: §16.3 wants a smoke alarm, and a spike that
/// implemented an 11×11 Gaussian-windowed SSIM would be shipping an
/// unreviewed perceptual metric into a program that has not yet decided which
/// one it wants. The global form has the right monotonicity for "did something
/// break" and is stated as such.
fn ssim_luma(a: &[u8], b: &[u8], width: usize, height: usize) -> f64 {
    let n = width * height;
    if n == 0 {
        return 1.0;
    }
    let luma = |px: &[u8]| -> Vec<f64> {
        px.as_chunks::<4>()
            .0
            .iter()
            .map(|c| 0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64)
            .collect()
    };
    let la = luma(a);
    let lb = luma(b);

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let ma = mean(&la);
    let mb = mean(&lb);
    let mut va = 0.0;
    let mut vb = 0.0;
    let mut cov = 0.0;
    for (x, y) in la.iter().zip(lb.iter()) {
        va += (x - ma) * (x - ma);
        vb += (y - mb) * (y - mb);
        cov += (x - ma) * (y - mb);
    }
    let d = (n as f64 - 1.0).max(1.0);
    va /= d;
    vb /= d;
    cov /= d;

    // The standard 8-bit stabilizers.
    let c1 = (0.01 * 255.0f64).powi(2);
    let c2 = (0.03 * 255.0f64).powi(2);
    ((2.0 * ma * mb + c1) * (2.0 * cov + c2)) / ((ma * ma + mb * mb + c1) * (va + vb + c2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surf(w: u32, h: u32, f: impl Fn(u32, u32) -> [f32; 4]) -> Surface {
        let mut pixels = Vec::new();
        for y in 0..h {
            for x in 0..w {
                pixels.extend_from_slice(&f(x, y));
            }
        }
        Surface {
            width: w,
            height: h,
            pixels,
        }
    }

    #[test]
    fn identical_surfaces_diverge_by_nothing() {
        let s = surf(8, 8, |x, y| [x as f32 / 8.0, y as f32 / 8.0, 0.5, 1.0]);
        let d = diverge(&s, &s.clone());
        assert_eq!(d.max_abs, 0.0);
        assert_eq!(d.max_u8, 0);
        assert_eq!(d.differing_u8, 0);
        assert!((d.ssim - 1.0).abs() < 1e-9, "ssim {}", d.ssim);
        assert_eq!(d.differing_fraction(), 0.0);
    }

    #[test]
    fn a_one_bit_difference_is_reported_not_swallowed() {
        let a = surf(4, 4, |_, _| [0.5, 0.5, 0.5, 1.0]);
        let mut b = a.clone();
        b.pixels[0] += 0.01;
        let d = diverge(&a, &b);
        assert!(d.max_abs > 0.0);
        assert!(d.differing_u8 >= 1, "an 0.01 linear step must move a byte");
        assert!(d.mean_abs > 0.0 && d.mean_abs < d.max_abs);
    }

    #[test]
    fn ssim_falls_when_the_picture_actually_changes() {
        let a = surf(16, 16, |x, _| {
            if x < 8 {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.0, 0.0, 0.0, 1.0]
            }
        });
        let b = surf(16, 16, |x, _| {
            if x < 8 {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [1.0, 1.0, 1.0, 1.0]
            }
        });
        let d = diverge(&a, &b);
        assert!(d.ssim < 0.5, "inverted image should not score {}", d.ssim);
        assert_eq!(d.max_u8, 255);
    }

    #[test]
    fn the_summary_names_every_number() {
        let s = surf(2, 2, |_, _| [0.0, 0.0, 0.0, 1.0]);
        let text = diverge(&s, &s.clone()).summary();
        for token in ["max_abs", "mean_abs", "rms", "max_u8", "differing", "ssim"] {
            assert!(text.contains(token), "summary missing {token}: {text}");
        }
    }
}
