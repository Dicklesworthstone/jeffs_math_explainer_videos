//! Video G's colour roles (the storyboard's table): one colour per idea for
//! the whole video. `hero`, `warn`, `good` and the quiz card's violet are the
//! corpus's own; the other five were chosen so that every pair of the nine
//! is at least ΔE2000 20 apart (the test below computes it; the first
//! draft's palette was not, review R1 finding C7).

use explainer_kit::{Palette, hex};
use fmn::prelude::*;

/// Orientation and turns: the book's cover, a ring, a turn's arc.
pub fn turn() -> Srgb {
    hex("#4A8DF8")
}

/// Position, and a path travelled.
pub fn place() -> Srgb {
    hex("#3FE0E0")
}

/// The flat piece: every velocity arrow, twists, the Jacobian's columns.
pub fn flat() -> Srgb {
    hex("#D48F3F")
}

/// A screw's axis, and the joint in question.
pub fn screw() -> Srgb {
    Palette::hero()
}

/// The goal pose.
pub fn goal() -> Srgb {
    hex("#EC4E97")
}

/// The arm's links and joints at rest.
pub fn steel() -> Srgb {
    hex("#CFD7E5")
}

/// A lost direction, a bad spot, a wrong path (always dashed or crossed).
pub fn warn() -> Srgb {
    Palette::red()
}

/// A confirmed answer (ticks and equals signs only).
pub fn good() -> Srgb {
    Palette::green()
}

/// The quiz card and its countdown (the same card as Videos F and J).
pub fn quiz() -> Srgb {
    hex("#B189D6")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CIE L*a*b* (D65) of an sRGB colour.
    fn lab(c: Srgb) -> [f64; 3] {
        let h = c.to_hex();
        let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map(f64::from).expect("hex") / 255.0;
        let lin = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(ch(1)), lin(ch(3)), lin(ch(5)));
        let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
        let z = 0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b;
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

    /// CIEDE2000 colour difference (Sharma, Wu and Dalal's formulation; the
    /// same code as Video C's test, which checks it against their table).
    fn de2000(a: [f64; 3], b: [f64; 3]) -> f64 {
        let deg = f64::to_radians;
        let [l1, a1, b1] = a;
        let [l2, a2, b2] = b;
        let cb = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
        let g = 0.5 * (1.0 - (cb.powi(7) / (cb.powi(7) + 25f64.powi(7))).sqrt());
        let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
        let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
        let hue = |b: f64, a: f64| b.atan2(a).to_degrees().rem_euclid(360.0);
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
            ("turn", turn()),
            ("place", place()),
            ("flat", flat()),
            ("screw", screw()),
            ("goal", goal()),
            ("steel", steel()),
            ("warn", warn()),
            ("good", good()),
            ("quiz", quiz()),
        ];
        let mut worst = f64::INFINITY;
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 20.0, "{na} and {nb}: ΔE2000 {d:.1}");
                worst = worst.min(d);
            }
        }
        // the storyboard quotes the least: pink against violet
        assert!((worst - 20.7).abs() < 0.05, "least ΔE2000 {worst:.2}");
    }
}
