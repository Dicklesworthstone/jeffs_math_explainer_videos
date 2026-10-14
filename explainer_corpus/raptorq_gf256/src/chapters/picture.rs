//! The picture the hook sends (01): a sunset over hills and sea, 100 × 100
//! cells, one cell per piece (the model's 10,000 pieces), drawn as row
//! strips (spike S-3: one polyline per row, per-point stroke colours, so the
//! whole picture is 100 mobjects).

use explainer_kit::props::RowStrips;
use explainer_kit::{hex, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use crate::model::HOOK_PIECES;

/// Cells per side: the picture is SIDE × SIDE pieces.
pub const SIDE: usize = 100;

const _: () = assert!(SIDE * SIDE == HOOK_PIECES as usize, "one cell per piece");

fn mix(a: &str, b: &str, t: f64) -> Srgb {
    interpolate_color(hex(a), hex(b), t.clamp(0.0, 1.0))
}

/// The colour of cell (`row`, `col`), row 0 at the top.
pub fn cell(row: usize, col: usize) -> Srgb {
    let (x, y) = (col as f64 / (SIDE - 1) as f64, row as f64 / (SIDE - 1) as f64);
    let horizon = 0.62;
    let (sun_x, sun_y, sun_r) = (0.64, 0.47, 0.11);
    // hills: two ridges, the far one paler, each sloping down into the sea
    // on its right
    let far = 0.50 + 0.05 * (x * 9.0 + 0.6).sin() + 0.03 * (x * 23.0).sin() + 0.9 * (x - 0.72).max(0.0);
    let near = 0.57 + 0.06 * (x * 5.0 + 2.1).sin() + 0.02 * (x * 17.0 + 1.0).sin() + 1.1 * (x - 0.46).max(0.0);
    if y < horizon {
        let d = ((x - sun_x).powi(2) + (y - sun_y).powi(2)).sqrt();
        if y > near {
            return mix("#1C2340", "#141A30", (y - near) * 8.0);
        }
        if y > far {
            return mix("#3A3768", "#2B2B55", (y - far) * 6.0);
        }
        if d < sun_r {
            return mix("#FFF1B8", "#FFC86B", d / sun_r);
        }
        // the sky: indigo above, rose and amber at the horizon, a glow round the sun
        let sky = if y < 0.35 {
            mix("#161B44", "#5B3B7A", y / 0.35)
        } else {
            mix("#5B3B7A", "#F08A5D", (y - 0.35) / (horizon - 0.35))
        };
        return interpolate_color(sky, hex("#FFB36B"), (0.35 - d).max(0.0) * 1.6);
    }
    // the sea: dark water, the sun's path in broken amber lines
    let depth = (y - horizon) / (1.0 - horizon);
    let water = mix("#2A2F5E", "#0F1430", depth);
    let path = (x - sun_x).abs() < 0.05 + 0.10 * depth;
    let ripple = (row + col / 7).is_multiple_of(3);
    if path && ripple {
        mix("#FFC86B", "#C9744A", depth)
    } else {
        water
    }
}

/// Every cell's colour, rows top to bottom.
pub fn colors() -> Vec<Vec<Srgb>> {
    (0..SIDE).map(|r| (0..SIDE).map(|c| cell(r, c)).collect()).collect()
}

/// The picture as `size` × `size` row strips, centred at `at`.
pub fn picture(stage: &mut Stage<'_>, size: f64, at: Vec3) -> fmn::Result<RowStrips> {
    let cell = size / SIDE as f64;
    RowStrips::build(
        stage,
        v(at[0] - size / 2.0, at[1] + size / 2.0),
        (cell, cell),
        &colors(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picture_has_a_sun_a_sky_and_a_sea() {
        let c = colors();
        assert_eq!((c.len(), c[0].len()), (SIDE, SIDE));
        let lum = |s: Srgb| 0.2126 * s.r + 0.7152 * s.g + 0.0722 * s.b;
        // the sun is the brightest thing in the sky; the top of the sky is dark
        assert!(lum(cell(47, 64)) > 0.8, "sun");
        assert!(lum(cell(0, 10)) < 0.15, "night sky");
        assert!(lum(cell(99, 5)) < 0.12, "deep water");
        // the cells are not all one colour: a picture, not a wash
        let distinct: std::collections::BTreeSet<String> = c.iter().flatten().map(|s| s.to_hex()).collect();
        assert!(distinct.len() > 200, "{}", distinct.len());
    }
}
