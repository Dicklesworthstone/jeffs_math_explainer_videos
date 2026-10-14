//! Drawing shared by Video A's chapters: the colour roles (storyboard §3),
//! the flat-fill style rules (one flat fill per mobject, no outlines on big
//! shapes; `explainer_kit::kit`), the fish glyphs, and how each glyph's
//! parts are painted so a fade can scale them.

use explainer_kit::{Palette, hex, v};
use fmn::prelude::*;

/// The storyboard's colour roles (§3). Every chapter paints an idea with its
/// role, never with a raw palette colour, so the video keeps one look.
pub struct Role;

impl Role {
    /// Marked (the first catch) and "caught twice": the hero gold.
    pub fn marked() -> Srgb {
        Palette::hero()
    }
    /// Unmarked fish.
    pub fn unmarked() -> Srgb {
        Palette::blue()
    }
    /// The second catch: net rings, the mosaic's top row.
    pub fn second() -> Srgb {
        Palette::green()
    }
    /// The dark cell, the lake size, the estimate.
    pub fn dark() -> Srgb {
        Palette::teal()
    }
    /// Likelihood, "score": curves and the landscape's slice.
    pub fn score() -> Srgb {
        Palette::purple()
    }
    /// Probability over k: histogram bars.
    pub fn chance() -> Srgb {
        hex("#8A93A3")
    }
    /// Bias and failure (never the marks).
    pub fn bias() -> Srgb {
        Palette::red()
    }
    /// A net that is neither catch: the dip of chapter 01, the ladle of 03.
    pub fn net() -> Srgb {
        Palette::ink()
    }
    /// The lake's water, and the deep hole's three shades below it.
    pub fn water() -> Srgb {
        hex("#13253A")
    }
    pub fn deep() -> [Srgb; 3] {
        [hex("#10202F"), hex("#0D1A28"), hex("#0A1520")]
    }
    /// The bank round the water.
    pub fn bank() -> Srgb {
        hex("#0A141E")
    }
}

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// How one part of a glyph is painted, at full strength: what an
/// opacity-only fade scales (`lake::Shade`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Fill(f64),
    Stroke(f64),
}

/// A glyph built at the origin pointing east, and its parts' paints in
/// child order.
pub struct Glyph {
    pub mob: VMobject,
    pub paints: Vec<Paint>,
}

/// A fish's body: a lens from the tail joint to the nose, fuller in front.
fn body(color: Srgb) -> VMobject {
    let (tail, nose, half) = (-0.062, 0.104, 0.034);
    let n = 9;
    let edge = |i: usize| {
        let u = i as f64 / n as f64;
        let x = tail + (nose - tail) * u;
        // fullest a third of the way back from the nose
        let h = half * (std::f64::consts::PI * u.powf(0.75)).sin().max(0.0).powf(0.85);
        (x, h.max(0.006))
    };
    let mut pts: Vec<Vec3> = (0..=n)
        .map(|i| {
            let (x, h) = edge(i);
            v(x, if i == n { 0.0 } else { h })
        })
        .collect();
    pts.extend((0..n).rev().map(|i| {
        let (x, h) = edge(i);
        v(x, -h)
    }));
    Polygon::new(pts).style(flat(color, 1.0)).build()
}

/// A forked tail.
fn tail(color: Srgb) -> VMobject {
    Polygon::new([v(-0.052, 0.0), v(-0.118, 0.042), v(-0.098, 0.0), v(-0.118, -0.042)])
        .style(flat(color, 1.0))
        .build()
}

/// A fish pointing east, flat fills: body and tail.
pub fn fish(color: Srgb) -> Glyph {
    Glyph {
        mob: v_group([body(color), tail(color)]),
        paints: vec![Paint::Fill(1.0), Paint::Fill(1.0)],
    }
}

/// A marked fish: gold, with a soft two-step halo (filled discs: cheaper
/// than stroked glow layers on a hundred moving things; spike S-2).
pub fn marked_fish() -> Glyph {
    let c = Role::marked();
    let outer = Circle::new().radius(0.15).style(flat(c, 0.09)).build();
    let inner = Circle::new().radius(0.095).style(flat(c, 0.2)).build();
    Glyph {
        mob: v_group([outer, inner, body(c), tail(c)]),
        paints: vec![Paint::Fill(0.09), Paint::Fill(0.2), Paint::Fill(1.0), Paint::Fill(1.0)],
    }
}

/// A red-spotted fish: Feller's mark (02_2), before it becomes the glow.
pub fn spotted_fish() -> VMobject {
    let spot = Circle::new()
        .radius(0.017)
        .arc_center(v(0.042, 0.004))
        .style(flat(Palette::red(), 1.0))
        .build();
    v_group([body(Role::unmarked()), tail(Role::unmarked()), spot])
}

/// The second net's mark: a green ring round a fish.
pub fn ring() -> Glyph {
    Glyph {
        mob: Circle::new()
            .radius(0.15)
            .style(line_style(Role::second(), 2.5))
            .build(),
        paints: vec![Paint::Stroke(1.0)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_names_a_paint_for_each_part() {
        for g in [fish(Role::unmarked()), marked_fish()] {
            assert_eq!(g.mob.children().len(), g.paints.len());
        }
        // the ring is one stroked circle with no parts of its own
        let r = ring();
        assert_eq!(r.paints, vec![Paint::Stroke(1.0)]);
        assert!(r.mob.children().is_empty());
    }

    #[test]
    fn the_fish_points_east_and_fits_its_halo() {
        let g = fish(Role::unmarked());
        let (lo, hi) = g.mob.extent().expect("a fish has points");
        assert!(hi[0] > 0.1 && lo[0] < -0.11, "nose east, tail west: {lo:?} {hi:?}");
        assert!(hi[1] <= 0.045 && lo[1] >= -0.045);
        // the halo covers the whole fish
        assert!(hi[0].max(-lo[0]) < 0.15);
    }
}
