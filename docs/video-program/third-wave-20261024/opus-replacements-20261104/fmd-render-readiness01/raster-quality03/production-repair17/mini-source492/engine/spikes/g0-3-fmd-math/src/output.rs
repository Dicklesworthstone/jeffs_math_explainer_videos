//! The spike's output model: positioned glyphs (each naming its face —
//! multi-face layout is structural, see the ratification note), rules,
//! and drawn paths, every one carrying §11.3 source-span provenance.
//! Coordinates are ems, y-up, baseline at 0. `to_svg` resolves glyph
//! outlines through fmd-font for the visual-review artifacts.

use core::fmt::Write as _;
use core::ops::Range;

use fmd_font::Font;
use fmd_font::outline::{Contour, Segment};

/// Which bundled face a glyph resolves through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    /// Computer Modern Unicode roman.
    CmRegular,
    /// The Noto Sans Math symbol-fallback subset.
    NotoMath,
}

/// A positioned glyph: `(x, y)` is its origin on the baseline, `size`
/// the em scale factor (1.0 = text size).
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedGlyph {
    /// The face resolving this glyph.
    pub face: Face,
    /// Glyph id within the face.
    pub gid: u16,
    /// The source character (for review/debugging; gid is authoritative).
    pub ch: char,
    /// X of the glyph origin, ems.
    pub x: f64,
    /// Y of the baseline, ems (y-up).
    pub y: f64,
    /// Em scale factor.
    pub size: f64,
    /// Source-span provenance (§11.3).
    pub span: Range<usize>,
}

/// A positioned rule (fraction bars, radical overbars, drawn strokes):
/// `(x, y)` is the bottom-left corner, ems, y-up.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedRule {
    /// Left edge.
    pub x: f64,
    /// Bottom edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height (thickness for horizontal rules).
    pub h: f64,
}

/// A drawn-path element (the OQ-2 delimiter mainline): closed quadratic
/// contours in em units, positioned at `(x, y)`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedPath {
    /// Closed quadratic contours (em units, y-up, relative to `(x, y)`).
    pub contours: Vec<Contour>,
    /// X offset, ems.
    pub x: f64,
    /// Y offset, ems.
    pub y: f64,
    /// Source-span provenance.
    pub span: Range<usize>,
}

/// A finished layout: positioned content plus overall box dimensions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    /// Positioned glyphs.
    pub glyphs: Vec<PlacedGlyph>,
    /// Positioned rules.
    pub rules: Vec<PlacedRule>,
    /// Drawn paths.
    pub paths: Vec<PlacedPath>,
    /// Total advance width, ems.
    pub width: f64,
    /// Height above the baseline, ems.
    pub height: f64,
    /// Depth below the baseline, ems.
    pub depth: f64,
}

impl Layout {
    /// Translate every element.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        for g in &mut self.glyphs {
            g.x += dx;
            g.y += dy;
        }
        for r in &mut self.rules {
            r.x += dx;
            r.y += dy;
        }
        for p in &mut self.paths {
            p.x += dx;
            p.y += dy;
        }
    }

    /// Absorb another fragment's elements.
    pub fn absorb(&mut self, other: Layout) {
        self.glyphs.extend(other.glyphs);
        self.rules.extend(other.rules);
        self.paths.extend(other.paths);
    }

    /// Render to a standalone SVG (100 px/em) by resolving glyph
    /// outlines through the given faces — the visual-review artifact.
    #[must_use]
    pub fn to_svg(&self, cm: &Font, noto: &Font) -> String {
        const PX: f64 = 100.0;
        let pad = 0.25;
        let w = (self.width + 2.0 * pad) * PX;
        let h = (self.height + self.depth + 2.0 * pad) * PX;
        let baseline_y = (self.height + pad) * PX;
        let x0 = pad * PX;
        let mut svg = String::new();
        let _ = write!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" \
             viewBox=\"0 0 {w:.1} {h:.1}\">\n\
             <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n\
             <line x1=\"0\" y1=\"{baseline_y:.1}\" x2=\"{w:.1}\" y2=\"{baseline_y:.1}\" \
             stroke=\"#d0e0ff\" stroke-width=\"1\"/>\n"
        );
        for g in &self.glyphs {
            let font = match g.face {
                Face::CmRegular => cm,
                Face::NotoMath => noto,
            };
            let Ok(outline) = font.glyph_outline(g.gid) else {
                continue;
            };
            let upm = f64::from(font.units_per_em);
            let scale = g.size / upm * PX;
            let ox = x0 + g.x * PX;
            let oy = baseline_y - g.y * PX;
            let mut d = String::new();
            for c in &outline.contours {
                emit_contour(&mut d, c, |x, y| (ox + x * scale, oy - y * scale));
            }
            let _ = writeln!(svg, "<path d=\"{d}\" fill=\"black\"/>");
        }
        for r in &self.rules {
            let _ = writeln!(
                svg,
                "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"black\"/>",
                x0 + r.x * PX,
                baseline_y - (r.y + r.h) * PX,
                r.w * PX,
                r.h * PX
            );
        }
        for p in &self.paths {
            let ox = x0 + p.x * PX;
            let oy = baseline_y - p.y * PX;
            let mut d = String::new();
            for c in &p.contours {
                emit_contour(&mut d, c, |x, y| (ox + x * PX, oy - y * PX));
            }
            let _ = writeln!(svg, "<path d=\"{d}\" fill=\"black\"/>");
        }
        svg.push_str("</svg>\n");
        svg
    }
}

fn emit_contour(d: &mut String, c: &Contour, map: impl Fn(f64, f64) -> (f64, f64)) {
    let (sx, sy) = map(c.start.x, c.start.y);
    let _ = write!(d, "M{sx:.2} {sy:.2} ");
    for seg in &c.segments {
        match seg {
            Segment::Line { to } => {
                let (x, y) = map(to.x, to.y);
                let _ = write!(d, "L{x:.2} {y:.2} ");
            }
            Segment::Quad { ctrl, to } => {
                let (cx, cy) = map(ctrl.x, ctrl.y);
                let (x, y) = map(to.x, to.y);
                let _ = write!(d, "Q{cx:.2} {cy:.2} {x:.2} {y:.2} ");
            }
        }
    }
    d.push_str("Z ");
}
