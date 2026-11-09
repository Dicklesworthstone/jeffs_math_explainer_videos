//! The §6.3 color pipeline — one model, ours (Behavior Note BN-04).
//!
//! Colors decode to linear light, composite premultiplied, and encode at the
//! output transfer function. Manim's gradient *aesthetic* is deliberately
//! preserved inside that model: [`interpolate_color`] keeps the Reference's
//! `sqrt(lerp(c1², c2², alpha))` form and [`average_color`] its RMS form,
//! both operating on sRGB-encoded components exactly as
//! `manimlib/utils/color.py` does. An Oklab interpolation is offered as an
//! opt-in alternative, never as a silent replacement.

/// An sRGB-encoded color with unit-range components (the Reference's
/// `rgb` triple). This is the *user-facing* color type; rendering decodes
/// it to [`LinearRgba`] before any arithmetic that models light.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Srgb {
    /// Red, sRGB-encoded, in [0, 1].
    pub r: f64,
    /// Green, sRGB-encoded, in [0, 1].
    pub g: f64,
    /// Blue, sRGB-encoded, in [0, 1].
    pub b: f64,
}

/// A linear-light RGBA color, straight (not premultiplied) alpha.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LinearRgba {
    /// Red, linear light.
    pub r: f64,
    /// Green, linear light.
    pub g: f64,
    /// Blue, linear light.
    pub b: f64,
    /// Coverage/opacity in [0, 1].
    pub a: f64,
}

/// A premultiplied linear-light RGBA color — the compositing currency.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PremulRgba {
    /// Red · alpha, linear light.
    pub r: f64,
    /// Green · alpha, linear light.
    pub g: f64,
    /// Blue · alpha, linear light.
    pub b: f64,
    /// Coverage/opacity in [0, 1].
    pub a: f64,
}

/// Error from parsing a hex color string.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HexParseError;

impl std::fmt::Display for HexParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected a color of the form #RRGGBB or #RGB")
    }
}

impl std::error::Error for HexParseError {}

impl Srgb {
    /// Build from 8-bit sRGB components (`x / 255`, the Reference's
    /// `hex_to_rgb` arithmetic).
    #[must_use]
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    /// Parse `#RRGGBB` or `#RGB` (case-insensitive, `#` required).
    pub fn from_hex(s: &str) -> Result<Self, HexParseError> {
        let hex = s.strip_prefix('#').ok_or(HexParseError)?;
        let nib = |c: u8| -> Result<u8, HexParseError> {
            match c {
                b'0'..=b'9' => Ok(c - b'0'),
                b'a'..=b'f' => Ok(c - b'a' + 10),
                b'A'..=b'F' => Ok(c - b'A' + 10),
                _ => Err(HexParseError),
            }
        };
        let by = hex.as_bytes();
        match by.len() {
            6 => Ok(Self::from_rgb8(
                nib(by[0])? * 16 + nib(by[1])?,
                nib(by[2])? * 16 + nib(by[3])?,
                nib(by[4])? * 16 + nib(by[5])?,
            )),
            3 => Ok(Self::from_rgb8(
                nib(by[0])? * 17,
                nib(by[1])? * 17,
                nib(by[2])? * 17,
            )),
            _ => Err(HexParseError),
        }
    }

    /// Quantize to 8-bit components (round-half-away, clamped).
    #[must_use]
    pub fn to_rgb8(self) -> [u8; 3] {
        let q = |x: f64| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
        [q(self.r), q(self.g), q(self.b)]
    }

    /// Format as `#RRGGBB` (uppercase, like the Reference's `rgb_to_hex`).
    #[must_use]
    pub fn to_hex(self) -> String {
        let [r, g, b] = self.to_rgb8();
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    /// Decode to linear light with the given alpha.
    #[must_use]
    pub fn to_linear(self, alpha: f64) -> LinearRgba {
        LinearRgba {
            r: srgb_eotf(self.r),
            g: srgb_eotf(self.g),
            b: srgb_eotf(self.b),
            a: alpha,
        }
    }
}

impl LinearRgba {
    /// Encode back to sRGB, discarding alpha.
    #[must_use]
    pub fn to_srgb(self) -> Srgb {
        Srgb {
            r: srgb_oetf(self.r),
            g: srgb_oetf(self.g),
            b: srgb_oetf(self.b),
        }
    }

    /// Premultiply the color by its alpha.
    #[must_use]
    pub fn premultiply(self) -> PremulRgba {
        PremulRgba {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }
}

impl PremulRgba {
    /// Transparent black: the compositing identity.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Porter–Duff source-over: `self` over `dst`.
    #[must_use]
    pub fn over(self, dst: Self) -> Self {
        let k = 1.0 - self.a;
        Self {
            r: self.r + k * dst.r,
            g: self.g + k * dst.g,
            b: self.b + k * dst.b,
            a: self.a + k * dst.a,
        }
    }

    /// Un-premultiply (straight alpha). Alpha of zero yields black.
    #[must_use]
    pub fn unpremultiply(self) -> LinearRgba {
        if self.a == 0.0 {
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }
        } else {
            LinearRgba {
                r: self.r / self.a,
                g: self.g / self.a,
                b: self.b / self.a,
                a: self.a,
            }
        }
    }
}

/// The sRGB electro-optical transfer function (decode: encoded → linear),
/// per IEC 61966-2-1.
#[must_use]
pub fn srgb_eotf(u: f64) -> f64 {
    if u <= 0.04045 {
        u / 12.92
    } else {
        fmn_dmath::pow((u + 0.055) / 1.055, 2.4)
    }
}

/// The sRGB opto-electronic transfer function (encode: linear → encoded).
#[must_use]
pub fn srgb_oetf(l: f64) -> f64 {
    if l <= 0.003_130_8 {
        12.92 * l
    } else {
        1.055 * fmn_dmath::pow(l, 1.0 / 2.4) - 0.055
    }
}

/// Linear interpolation, the Reference's `interpolate`.
#[must_use]
fn lerp(a: f64, b: f64, alpha: f64) -> f64 {
    (1.0 - alpha) * a + alpha * b
}

/// Manim's gradient interpolation, kept exactly (BN-04): per-channel
/// `sqrt(lerp(c1², c2², alpha))` on sRGB-encoded components.
#[must_use]
pub fn interpolate_color(c1: Srgb, c2: Srgb, alpha: f64) -> Srgb {
    let ch = |x: f64, y: f64| lerp(x * x, y * y, alpha).sqrt();
    Srgb {
        r: ch(c1.r, c2.r),
        g: ch(c1.g, c2.g),
        b: ch(c1.b, c2.b),
    }
}

/// The Reference's opt-in HSL interpolation.  Its `colour.Color` boundary
/// stores HSL as `(hue, saturation, lightness)` in the unit interval and
/// linearly interpolates all three components without shortest-arc hue
/// wrapping before converting back to sRGB.
#[must_use]
pub fn interpolate_color_by_hsl(c1: Srgb, c2: Srgb, alpha: f64) -> Srgb {
    let hsl1 = srgb_to_hsl(c1);
    let hsl2 = srgb_to_hsl(c2);
    hsl_to_srgb([
        lerp(hsl1[0], hsl2[0], alpha),
        lerp(hsl1[1], hsl2[1], alpha),
        lerp(hsl1[2], hsl2[2], alpha),
    ])
}

/// The HSL branch of Manim's `color_gradient`, with the same sample points
/// and final-index correction as [`color_gradient`].
#[must_use]
pub fn color_gradient_by_hsl(reference_colors: &[Srgb], length: usize) -> Vec<Srgb> {
    color_gradient_with(reference_colors, length, interpolate_color_by_hsl)
}

/// Manim's `average_color`, kept exactly (BN-04): per-channel RMS over
/// sRGB-encoded components. An empty slice averages to black.
#[must_use]
pub fn average_color(colors: &[Srgb]) -> Srgb {
    if colors.is_empty() {
        return Srgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
    }
    let n = colors.len() as f64;
    let mean_sq = |f: fn(&Srgb) -> f64| -> f64 {
        (colors.iter().map(|c| f(c) * f(c)).sum::<f64>() / n).sqrt()
    };
    Srgb {
        r: mean_sq(|c| c.r),
        g: mean_sq(|c| c.g),
        b: mean_sq(|c| c.b),
    }
}

/// Manim's `color_gradient`: `length` colors sampled across the reference
/// colors with [`interpolate_color`], floors and end-edge case exactly as
/// the Reference computes them. Returns an empty vector for `length == 0`;
/// requires at least two reference colors otherwise.
#[must_use]
pub fn color_gradient(reference_colors: &[Srgb], length: usize) -> Vec<Srgb> {
    color_gradient_with(reference_colors, length, interpolate_color)
}

fn color_gradient_with(
    reference_colors: &[Srgb],
    length: usize,
    interpolate: fn(Srgb, Srgb, f64) -> Srgb,
) -> Vec<Srgb> {
    if length == 0 {
        return Vec::new();
    }
    assert!(
        reference_colors.len() >= 2,
        "color_gradient needs at least two reference colors"
    );
    let n_ref = reference_colors.len();
    (0..length)
        .map(|j| {
            // np.linspace(0, n_ref - 1, length)
            let alpha = if length == 1 {
                0.0
            } else {
                let step = (n_ref - 1) as f64 / (length - 1) as f64;
                j as f64 * step
            };
            let (floor, alpha_mod1) = if j == length - 1 {
                (n_ref - 2, 1.0)
            } else {
                (alpha as usize, alpha % 1.0)
            };
            interpolate(
                reference_colors[floor],
                reference_colors[floor + 1],
                alpha_mod1,
            )
        })
        .collect()
}

// These are the formulae used by `colour.Color` 0.1.5 at the Reference pin,
// including its near-gray tolerance. Keeping the conversion here preserves
// fmn-core as FrankenManim's one color authority.
fn srgb_to_hsl(color: Srgb) -> [f64; 3] {
    let min = color.r.min(color.g).min(color.b);
    let max = color.r.max(color.g).max(color.b);
    let range = max - min;
    let sum = min + max;
    let lightness = sum / 2.0;
    if range < 0.000_000_5 {
        return [0.0, 0.0, lightness];
    }

    let saturation = if lightness < 0.5 {
        range / sum
    } else {
        range / (2.0 - sum)
    };
    let red_distance = (((max - color.r) / 6.0) + (range / 2.0)) / range;
    let green_distance = (((max - color.g) / 6.0) + (range / 2.0)) / range;
    let blue_distance = (((max - color.b) / 6.0) + (range / 2.0)) / range;
    let mut hue = if color.r == max {
        blue_distance - green_distance
    } else if color.g == max {
        (1.0 / 3.0) + red_distance - blue_distance
    } else {
        (2.0 / 3.0) + green_distance - red_distance
    };
    if hue < 0.0 {
        hue += 1.0;
    }
    if hue > 1.0 {
        hue -= 1.0;
    }
    [hue, saturation, lightness]
}

fn hsl_to_srgb([hue, saturation, lightness]: [f64; 3]) -> Srgb {
    if saturation == 0.0 {
        return Srgb {
            r: lightness,
            g: lightness,
            b: lightness,
        };
    }
    let m2 = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        (lightness + saturation) - (saturation * lightness)
    };
    let m1 = 2.0 * lightness - m2;
    let channel = |hue: f64| {
        let mut hue = hue;
        while hue < 0.0 {
            hue += 1.0;
        }
        while hue > 1.0 {
            hue -= 1.0;
        }
        if 6.0 * hue < 1.0 {
            m1 + (m2 - m1) * 6.0 * hue
        } else if 2.0 * hue < 1.0 {
            m2
        } else if 3.0 * hue < 2.0 {
            m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
        } else {
            m1
        }
    };
    Srgb {
        r: channel(hue + 1.0 / 3.0),
        g: channel(hue),
        b: channel(hue - 1.0 / 3.0),
    }
}

// --- Oklab (the opt-in perceptual interpolation) --------------------------

/// A color in Oklab coordinates (L, a, b).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Oklab {
    /// Perceived lightness.
    pub l: f64,
    /// Green–red axis.
    pub a: f64,
    /// Blue–yellow axis.
    pub b: f64,
}

/// Convert sRGB to Oklab (Björn Ottosson's reference matrices).
#[must_use]
pub fn srgb_to_oklab(c: Srgb) -> Oklab {
    let lin = c.to_linear(1.0);
    let l = 0.412_221_470_8 * lin.r + 0.536_332_536_3 * lin.g + 0.051_445_992_9 * lin.b;
    let m = 0.211_903_498_2 * lin.r + 0.680_699_545_1 * lin.g + 0.107_396_956_6 * lin.b;
    let s = 0.088_302_461_9 * lin.r + 0.281_718_837_6 * lin.g + 0.629_978_700_5 * lin.b;
    let (l_, m_, s_) = (fmn_dmath::cbrt(l), fmn_dmath::cbrt(m), fmn_dmath::cbrt(s));
    Oklab {
        l: 0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_,
        a: 1.977_998_495_1 * l_ - 2.428_592_205_0 * m_ + 0.450_593_709_9 * s_,
        b: 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766_0 * s_,
    }
}

/// Convert Oklab back to sRGB (components may leave [0, 1] for
/// out-of-gamut inputs; callers clamp at quantization).
#[must_use]
pub fn oklab_to_srgb(c: Oklab) -> Srgb {
    let l_ = c.l + 0.396_337_777_4 * c.a + 0.215_803_757_3 * c.b;
    let m_ = c.l - 0.105_561_345_8 * c.a - 0.063_854_172_8 * c.b;
    let s_ = c.l - 0.089_484_177_5 * c.a - 1.291_485_548_0 * c.b;
    let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    LinearRgba {
        r: 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        g: -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        b: -0.004_196_086_3 * l - 0.703_418_614_8 * m + 1.707_614_701_0 * s,
        a: 1.0,
    }
    .to_srgb()
}

/// Perceptual interpolation in Oklab — the documented BN-04 OPTION, never
/// a silent replacement for [`interpolate_color`].
#[must_use]
pub fn interpolate_color_oklab(c1: Srgb, c2: Srgb, alpha: f64) -> Srgb {
    let (a, b) = (srgb_to_oklab(c1), srgb_to_oklab(c2));
    oklab_to_srgb(Oklab {
        l: lerp(a.l, b.l, alpha),
        a: lerp(a.a, b.a, alpha),
        b: lerp(a.b, b.b, alpha),
    })
}

// --- HWB (CSS-compatible helper, shared machinery with fmd) ---------------

/// CSS `hwb()`: hue in degrees, whiteness and blackness in [0, 1].
/// If `w + b >= 1` the result is the achromatic gray `w / (w + b)`.
#[must_use]
pub fn hwb(hue_deg: f64, w: f64, b: f64) -> Srgb {
    if w + b >= 1.0 {
        let gray = w / (w + b);
        return Srgb {
            r: gray,
            g: gray,
            b: gray,
        };
    }
    let pure = hue_to_rgb(hue_deg);
    let scale = 1.0 - w - b;
    Srgb {
        r: pure.r * scale + w,
        g: pure.g * scale + w,
        b: pure.b * scale + w,
    }
}

/// The pure hue wheel color for `hwb()` (HSL with S=100%, L=50%).
fn hue_to_rgb(hue_deg: f64) -> Srgb {
    let h = hue_deg.rem_euclid(360.0) / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    Srgb { r, g, b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parse_and_format_round_trip() {
        for s in ["#1C758A", "#FFFF00", "#000000", "#8B4513"] {
            assert_eq!(Srgb::from_hex(s).unwrap().to_hex(), s);
        }
        assert_eq!(Srgb::from_hex("#FA3").unwrap().to_hex(), "#FFAA33");
        assert!(Srgb::from_hex("1C758A").is_err());
        assert!(Srgb::from_hex("#12345").is_err());
        assert!(Srgb::from_hex("#GGHHII").is_err());
    }

    #[test]
    fn hwb_matches_css_semantics() {
        // hwb(0deg 0% 0%) is pure red; hwb(120deg 0% 0%) pure green.
        assert_eq!(
            hwb(0.0, 0.0, 0.0),
            Srgb {
                r: 1.0,
                g: 0.0,
                b: 0.0
            }
        );
        assert_eq!(
            hwb(120.0, 0.0, 0.0),
            Srgb {
                r: 0.0,
                g: 1.0,
                b: 0.0
            }
        );
        // Fully white + black degenerates to proportional gray.
        let g = hwb(200.0, 0.6, 0.6);
        assert!((g.r - 0.5).abs() < 1e-12 && g.r == g.g && g.g == g.b);
    }
}
