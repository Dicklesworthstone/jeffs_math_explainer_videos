//! Drawing shared by Video A's chapters: the flat-fill style rules (one flat
//! fill per mobject, no outlines on big shapes; `explainer_kit::kit`) and
//! the fish, marked and unmarked.

use explainer_kit::{Palette, v};
use fmn::prelude::*;

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// A fish pointing east: a body and a forked tail, flat fills.
pub fn fish_parts(color: Srgb) -> [VMobject; 2] {
    let body = Polygon::new([
        v(0.10, 0.0),
        v(0.04, 0.032),
        v(-0.04, 0.026),
        v(-0.065, 0.0),
        v(-0.04, -0.026),
        v(0.04, -0.032),
    ])
    .style(flat(color, 1.0))
    .build();
    let tail = Polygon::new([v(-0.055, 0.0), v(-0.11, 0.036), v(-0.11, -0.036)])
        .style(flat(color, 1.0))
        .build();
    [body, tail]
}

pub fn fish(color: Srgb) -> VMobject {
    v_group(fish_parts(color))
}

/// A marked fish: gold, with a soft filled halo (the S-2 "halo", cheaper
/// than stroked glow layers on a thousand moving things).
pub fn marked_fish() -> VMobject {
    let [b, t] = fish_parts(Palette::gold());
    let halo = Circle::new().radius(0.12).style(flat(Palette::gold(), 0.22)).build();
    v_group([halo, b, t])
}

/// The second net's mark: a green ring round a fish.
pub fn ring() -> VMobject {
    Circle::new()
        .radius(0.14)
        .style(line_style(Palette::green(), 2.5))
        .build()
}
