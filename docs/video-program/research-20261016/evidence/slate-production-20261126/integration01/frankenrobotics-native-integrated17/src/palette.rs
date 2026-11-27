//! Persistent color roles for The Math Behind FrankenRobotics.
//!
//! Every recurring mathematical concept keeps one color across all ten
//! chapters. Pairwise CIEDE2000 separation is pinned >= 20.0 (minimum pair
//! 20.70, symmetry pink against quiz violet), matching `oracle.py` and the
//! corpus convention in `explainer_corpus/lie_robotics/src/palette.rs`.

use explainer_kit::{Palette, hex};
use fmn::prelude::*;

/// Rotation, joint angle, circle factor SO(2), and turn arcs.
pub fn rotation() -> Srgb {
    hex("#4A8DF8")
}

/// Translation, position vector, and floor path travelled.
pub fn translation() -> Srgb {
    hex("#3FE0E0")
}

/// Velocity, twist, wheel tangent arrows, and rim speeds.
pub fn velocity() -> Srgb {
    hex("#D48F3F")
}

/// Active joint, Lie bracket leftover, and key emphasis.
pub fn hero() -> Srgb {
    Palette::hero()
}

/// Symmetry mirrors, permutation actions, and orthogonal mode patterns.
pub fn symmetry() -> Srgb {
    hex("#EC4E97")
}

/// Robot chassis, rigid links, and wheels at rest.
pub fn steel() -> Srgb {
    hex("#CFD7E5")
}

/// Blocked cells, forbidden slip, overloaded feet, and broken mirrors.
pub fn warn() -> Srgb {
    Palette::red()
}

/// Confirmed answer, cleared path, balanced stance, and surviving mirror.
pub fn good() -> Srgb {
    Palette::green()
}

/// Prediction quiz card and countdown ring.
pub fn quiz() -> Srgb {
    hex("#B189D6")
}

/// Dark panel fill behind cards and robot silhouettes.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// Flat fill with no stroke (engine rule: avoid outline on filled shapes).
pub fn flat(color: Srgb, opacity: f64) -> Style {
    Style::default().fill(color, opacity).stroke(color, 0.0, 0.0)
}

/// Stroke only with full opacity.
pub fn line_style(color: Srgb, width: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, 1.0)
}

/// Stroke only with custom opacity.
pub fn faded_line(color: Srgb, width: f64, opacity: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, opacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lab(c: Srgb) -> [f64; 3] {
        let h = c.to_hex();
        let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map(f64::from).expect("hex") / 255.0;
        let lin = |x: f64| {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(ch(1)), lin(ch(3)), lin(ch(5)));
        let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
        let z = 0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b;
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (24389.0 / 27.0 * t + 16.0) / 116.0
            }
        };
        let (fx, fy, fz) = (f(x / 0.950_47), f(y), f(z / 1.088_83));
        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }

    fn de2000(a: [f64; 3], b: [f64; 3]) -> f64 {
        let deg = f64::to_radians;
        let [l1, a1, b1] = a;
        let [l2, a2, b2] = b;
        let cb = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
        let g = 0.5 * (1.0 - (cb.powi(7) / (cb.powi(7) + 25f64.powi(7))).sqrt());
        let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
        let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
        let hue = |bv: f64, av: f64| bv.atan2(av).to_degrees().rem_euclid(360.0);
        let (h1p, h2p) = (hue(b1, a1p), hue(b2, a2p));
        let dlp = l2 - l1;
        let dcp = c2p - c1p;
        let mut dhp = h2p - h1p;
        if c1p * c2p == 0.0 {
            dhp = 0.0;
        } else if dhp > 180.0 {
            dhp -= 360.0;
        } else if dhp < -180.0 {
            dhp += 360.0;
        }
        let dhh = 2.0 * (c1p * c2p).sqrt() * deg(dhp / 2.0).sin();
        let lbp = (l1 + l2) / 2.0;
        let cbp = (c1p + c2p) / 2.0;
        let hbp = if c1p * c2p == 0.0 {
            h1p + h2p
        } else if (h1p - h2p).abs() <= 180.0 {
            (h1p + h2p) / 2.0
        } else if h1p + h2p < 360.0 {
            (h1p + h2p + 360.0) / 2.0
        } else {
            (h1p + h2p - 360.0) / 2.0
        };
        let t = 1.0 - 0.17 * deg(hbp - 30.0).cos() + 0.24 * deg(2.0 * hbp).cos() + 0.32 * deg(3.0 * hbp + 6.0).cos()
            - 0.20 * deg(4.0 * hbp - 63.0).cos();
        let dth = 30.0 * (-((hbp - 275.0) / 25.0).powi(2)).exp();
        let rc = 2.0 * (cbp.powi(7) / (cbp.powi(7) + 25f64.powi(7))).sqrt();
        let sl = 1.0 + 0.015 * (lbp - 50.0).powi(2) / (20.0 + (lbp - 50.0).powi(2)).sqrt();
        let sc = 1.0 + 0.045 * cbp;
        let sh = 1.0 + 0.015 * cbp * t;
        let rt = -deg(2.0 * dth).sin() * rc;
        ((dlp / sl).powi(2) + (dcp / sc).powi(2) + (dhh / sh).powi(2) + rt * (dcp / sc) * (dhh / sh)).sqrt()
    }

    #[test]
    fn every_pair_of_roles_is_at_least_twenty_apart() {
        let roles = [
            ("rotation", rotation()),
            ("translation", translation()),
            ("velocity", velocity()),
            ("hero", hero()),
            ("symmetry", symmetry()),
            ("steel", steel()),
            ("warn", warn()),
            ("good", good()),
            ("quiz", quiz()),
        ];
        let mut worst = f64::INFINITY;
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 20.0, "{na} and {nb}: dE2000 {d:.2}");
                worst = worst.min(d);
            }
        }
        assert!((worst - 20.70).abs() < 0.05, "least dE2000 {worst:.2}");
    }
}
