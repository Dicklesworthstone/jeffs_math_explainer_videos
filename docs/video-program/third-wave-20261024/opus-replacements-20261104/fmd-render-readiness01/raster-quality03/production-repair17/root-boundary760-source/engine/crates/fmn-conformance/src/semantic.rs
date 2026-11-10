//! Semantic sanity oracles over rendered frames (fm-5wq.46).
//!
//! A bit-locked self-golden proves a frame is *unchanged*, never that it is
//! *right*: G1 closed on 2026-08-20 while every native 2D render was
//! vertically mirrored, and the certified goldens had locked the mirrored
//! frames for a month (fm-sq8.9). These oracles are the independent check.
//! They read one frame of the built-in semantic witness
//! ([`fmn::builtins::SEMANTIC_WITNESS_SCENE_NAME`]) on whatever route drew
//! it, and ask questions a mirrored, misplaced or recoloured frame cannot
//! answer correctly:
//!
//! - **orientation**: the red right triangle sits in the upper-left quadrant
//!   with its full-width edge on top and its right angle on the left. The
//!   F's full-width bar is on top and its stem on the left.
//! - **placement**: the dot at `UP * 3` lands in the top band at its mapped
//!   pixel, and the dot at `LEFT * 5` in the left band.
//! - **colour**: the triangle's interior decodes to `#FF0000`, and the
//!   frame corners to the configured background.
//! - **reading order** (routes that typeset): `Text("AB")` puts A left of B,
//!   and `Tex("x^2")` puts the superscript above and right of its base.
//!
//! Every element of the witness has its own exact colour, so an oracle
//! classifies pixels by colour alone and needs nothing from the renderer.
//! [`flipped_rows`] plants the regression the oracles exist for: a test runs
//! each route's real frame and its vertical mirror, and requires the first
//! to pass and the second to fail.

use fmn::builtins::witness;

/// One rendered frame: tightly packed RGBA8, sRGB, top row first.
#[derive(Clone, Copy, Debug)]
pub struct Frame<'a> {
    /// Pixels per row.
    pub width: usize,
    /// Rows.
    pub height: usize,
    /// `width * height * 4` bytes.
    pub rgba: &'a [u8],
}

/// The scene-space extent the frame shows, centred on the origin: the
/// default camera is 8 units high at the frame's aspect ratio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameShape {
    /// Scene units across the frame.
    pub width_units: f64,
    /// Scene units down the frame.
    pub height_units: f64,
}

impl FrameShape {
    /// The default 8-unit-high frame at `width x height` pixels.
    #[must_use]
    pub fn default_for(width: usize, height: usize) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let aspect = width as f64 / height as f64;
        Self {
            width_units: 8.0 * aspect,
            height_units: 8.0,
        }
    }
}

/// One oracle's verdict on one frame, with what it expected and what it
/// measured, for the per-oracle log line.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// The oracle, e.g. `orientation.triangle`.
    pub oracle: &'static str,
    /// The witness element it read.
    pub fixture: &'static str,
    /// The region or relation it required.
    pub expected: String,
    /// What it measured: mass, shares, ratios or centroids.
    pub measured: String,
    /// Whether the frame satisfied it.
    pub pass: bool,
}

/// Which oracles apply on a route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Orientation, placement and colour: every route.
    Geometry,
    /// Also reading order, on routes that typeset `Text` and `Tex`.
    WithReadingOrder,
}

/// Per-channel tolerance for classifying a pixel as one witness colour. An
/// anti-aliased edge blends toward the background by more than this, so only
/// an element's interior counts; the witness colours are at least 127 apart
/// on some channel.
const CLASSIFY_TOLERANCE: u8 = 40;
/// Per-channel tolerance for an exact colour check (fill, background).
const EXACT_TOLERANCE: u8 = 3;

fn near(a: u8, b: u8, tolerance: u8) -> bool {
    a.abs_diff(b) <= tolerance
}

fn matches(pixel: &[u8; 4], color: [u8; 3], tolerance: u8) -> bool {
    near(pixel[0], color[0], tolerance)
        && near(pixel[1], color[1], tolerance)
        && near(pixel[2], color[2], tolerance)
}

/// The pixels of one witness colour, as `(column, row)` pairs.
fn pixels_of(frame: Frame<'_>, color: [u8; 3]) -> Vec<(usize, usize)> {
    frame
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(_, pixel)| matches(pixel, color, CLASSIFY_TOLERANCE))
        .map(|(index, _)| (index % frame.width, index / frame.width))
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn centroid(pixels: &[(usize, usize)]) -> Option<(f64, f64)> {
    if pixels.is_empty() {
        return None;
    }
    let n = pixels.len() as f64;
    let (sx, sy) = pixels.iter().fold((0.0, 0.0), |(sx, sy), &(x, y)| {
        (sx + x as f64, sy + y as f64)
    });
    Some((sx / n, sy / n))
}

/// Mass in the first and last third of the pixels' bounding box, along rows
/// (`vertical`) or columns.
fn thirds(pixels: &[(usize, usize)], vertical: bool) -> (usize, usize) {
    let coord = |&(x, y): &(usize, usize)| if vertical { y } else { x };
    let (Some(lo), Some(hi)) = (
        pixels.iter().map(coord).min(),
        pixels.iter().map(coord).max(),
    ) else {
        return (0, 0);
    };
    let span = hi - lo + 1;
    let first = pixels.iter().filter(|p| (coord(p) - lo) * 3 < span).count();
    let last = pixels
        .iter()
        .filter(|p| (coord(p) - lo) * 3 >= 2 * span)
        .count();
    (first, last)
}

/// A scene point's pixel position: scene +y is image up, row 0 is the top.
fn to_pixel(frame: Frame<'_>, shape: FrameShape, point: [f64; 3]) -> (f64, f64) {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (frame.width as f64, frame.height as f64);
    (
        (point[0] / shape.width_units + 0.5) * w,
        (0.5 - point[1] / shape.height_units) * h,
    )
}

fn pixel_at(frame: Frame<'_>, x: f64, y: f64) -> Option<[u8; 4]> {
    if !(x >= 0.0 && y >= 0.0) {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (column, row) = (x as usize, y as usize);
    if column >= frame.width || row >= frame.height {
        return None;
    }
    let at = (row * frame.width + column) * 4;
    frame.rgba.get(at..at + 4).map(|p| [p[0], p[1], p[2], p[3]])
}

/// "At least twice": an asymmetric shape's heavy third against its light
/// one. Both witness shapes are about 5:1 and 3:1, so 2:1 has margin both
/// ways and a mirror inverts it.
fn heavier(a: usize, b: usize) -> bool {
    a > 0 && a >= 2 * b
}

fn orientation_triangle(frame: Frame<'_>) -> Reading {
    let red = pixels_of(frame, witness::TRIANGLE);
    let upper_left = red
        .iter()
        .filter(|&&(x, y)| 2 * x < frame.width && 2 * y < frame.height)
        .count();
    let (top, bottom) = thirds(&red, true);
    let (left, right) = thirds(&red, false);
    let share = if red.is_empty() {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        let share = upper_left as f64 / red.len() as f64;
        share
    };
    Reading {
        oracle: "orientation.triangle",
        fixture: "red right triangle",
        expected:
            "upper-left quadrant share >= 0.95; top third >= 2x bottom; left third >= 2x right"
                .to_owned(),
        measured: format!(
            "mass {} upper_left_share {share:.3} top {top} bottom {bottom} left {left} right {right}",
            red.len()
        ),
        pass: share >= 0.95 && heavier(top, bottom) && heavier(left, right),
    }
}

fn orientation_f(frame: Frame<'_>) -> Reading {
    let white = pixels_of(frame, witness::F_SHAPE);
    let (top, bottom) = thirds(&white, true);
    let (left, right) = thirds(&white, false);
    let upper_right = white
        .iter()
        .filter(|&&(x, y)| 2 * x >= frame.width && 2 * y < frame.height)
        .count();
    Reading {
        oracle: "orientation.f_shape",
        fixture: "white F",
        expected: "upper-right quadrant; top third >= 2x bottom (bar on top); left third >= 2x right (stem left)"
            .to_owned(),
        measured: format!(
            "mass {} upper_right {upper_right} top {top} bottom {bottom} left {left} right {right}",
            white.len()
        ),
        pass: !white.is_empty()
            && upper_right == white.len()
            && heavier(top, bottom)
            && heavier(left, right),
    }
}

/// The frame band a placed dot must land in.
#[derive(Clone, Copy)]
enum Band {
    /// The top quarter of the rows.
    Top,
    /// The left quarter of the columns.
    Left,
}

impl Band {
    const fn describe(self) -> &'static str {
        match self {
            Self::Top => "in the top quarter",
            Self::Left => "in the left quarter",
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn contains(self, x: f64, y: f64, frame: Frame<'_>) -> bool {
        match self {
            Self::Top => y < frame.height as f64 / 4.0,
            Self::Left => x < frame.width as f64 / 4.0,
        }
    }
}

/// A dot's centroid must land within 2% of the frame of its mapped centre,
/// and inside its band.
fn placement(
    frame: Frame<'_>,
    shape: FrameShape,
    oracle: &'static str,
    fixture: &'static str,
    color: [u8; 3],
    center: [f64; 3],
    band: Band,
) -> Reading {
    let dots = pixels_of(frame, color);
    let (ex, ey) = to_pixel(frame, shape, center);
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (frame.width as f64, frame.height as f64);
    let (measured, pass) = match centroid(&dots) {
        Some((cx, cy)) => (
            format!("mass {} centroid ({cx:.1}, {cy:.1})", dots.len()),
            (cx - ex).abs() <= 0.02 * w
                && (cy - ey).abs() <= 0.02 * h
                && band.contains(cx, cy, frame),
        ),
        None => ("mass 0".to_owned(), false),
    };
    Reading {
        oracle,
        fixture,
        expected: format!(
            "centroid within 2% of ({ex:.1}, {ey:.1}); {}",
            band.describe()
        ),
        measured,
        pass,
    }
}

fn color_fill(frame: Frame<'_>, shape: FrameShape) -> Reading {
    let [a, b, c] = witness::TRIANGLE_VERTICES;
    let inside = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0, 0.0];
    let (x, y) = to_pixel(frame, shape, inside);
    let sample = pixel_at(frame, x, y);
    Reading {
        oracle: "colour.fill",
        fixture: "red right triangle",
        expected: format!("pixel ({x:.0}, {y:.0}) is #FF0000 within {EXACT_TOLERANCE}"),
        measured: format!("{sample:?}"),
        pass: sample.is_some_and(|p| matches(&p, witness::TRIANGLE, EXACT_TOLERANCE)),
    }
}

fn color_background(frame: Frame<'_>, background: [u8; 3]) -> Reading {
    let (w, h) = (frame.width, frame.height);
    #[allow(clippy::cast_precision_loss)]
    let corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
        .map(|(x, y)| pixel_at(frame, x as f64, y as f64));
    Reading {
        oracle: "colour.background",
        fixture: "frame corners",
        expected: format!("all four corners are {background:?} within {EXACT_TOLERANCE}"),
        measured: format!("{corners:?}"),
        pass: corners
            .iter()
            .all(|p| p.is_some_and(|p| matches(&p, background, EXACT_TOLERANCE))),
    }
}

fn reading_order_text(frame: Frame<'_>) -> Reading {
    let a = centroid(&pixels_of(frame, witness::TEXT_A));
    let b = centroid(&pixels_of(frame, witness::TEXT_B));
    #[allow(clippy::cast_precision_loss)]
    let row_tolerance = 0.05 * frame.height as f64;
    Reading {
        oracle: "reading_order.text",
        fixture: "Text(\"AB\")",
        expected: "A's centroid left of B's, on the same line".to_owned(),
        measured: format!("A {a:?} B {b:?}"),
        pass: matches!((a, b), (Some(a), Some(b)) if a.0 < b.0 && (a.1 - b.1).abs() <= row_tolerance),
    }
}

fn reading_order_tex(frame: Frame<'_>) -> Reading {
    let base = centroid(&pixels_of(frame, witness::TEX_BASE));
    let sup = centroid(&pixels_of(frame, witness::TEX_SUPERSCRIPT));
    Reading {
        oracle: "reading_order.tex",
        fixture: "Tex(\"x^2\")",
        expected: "the superscript's centroid right of and above the base's".to_owned(),
        measured: format!("base {base:?} superscript {sup:?}"),
        pass: matches!((base, sup), (Some(b), Some(s)) if s.0 > b.0 && s.1 < b.1),
    }
}

/// Run the oracles that `coverage` selects on one frame of the semantic
/// witness drawn into `shape` over `background`.
#[must_use]
pub fn read_witness(
    frame: Frame<'_>,
    shape: FrameShape,
    background: [u8; 3],
    coverage: Coverage,
) -> Vec<Reading> {
    let mut readings = vec![
        orientation_triangle(frame),
        orientation_f(frame),
        placement(
            frame,
            shape,
            "placement.up_dot",
            "dot at UP*3",
            witness::UP_DOT,
            witness::UP_DOT_CENTER,
            Band::Top,
        ),
        placement(
            frame,
            shape,
            "placement.left_dot",
            "dot at LEFT*5",
            witness::LEFT_DOT,
            witness::LEFT_DOT_CENTER,
            Band::Left,
        ),
        color_fill(frame, shape),
        color_background(frame, background),
    ];
    if coverage == Coverage::WithReadingOrder {
        readings.push(reading_order_text(frame));
        readings.push(reading_order_tex(frame));
    }
    readings
}

/// The frame with its rows in reverse order: the vertical mirror fm-sq8.9
/// shipped, planted as a negative control.
#[must_use]
pub fn flipped_rows(frame: Frame<'_>) -> Vec<u8> {
    let stride = frame.width * 4;
    frame
        .rgba
        .chunks_exact(stride)
        .rev()
        .flatten()
        .copied()
        .collect()
}

/// The oracles that failed, for an assertion message.
#[must_use]
pub fn failures(readings: &[Reading]) -> Vec<&Reading> {
    readings.iter().filter(|reading| !reading.pass).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blank_frame_fails_every_oracle_but_background() {
        let rgba = [0_u8, 0, 0, 255].repeat(64 * 36);
        let frame = Frame {
            width: 64,
            height: 36,
            rgba: &rgba,
        };
        let readings = read_witness(
            frame,
            FrameShape::default_for(64, 36),
            [0, 0, 0],
            Coverage::WithReadingOrder,
        );
        let failed: Vec<_> = failures(&readings).iter().map(|r| r.oracle).collect();
        assert_eq!(readings.len(), 8);
        assert_eq!(failed.len(), 7, "{failed:?}");
        assert!(!failed.contains(&"colour.background"));
    }

    #[test]
    fn flipping_twice_is_the_identity() {
        let rgba: Vec<u8> = (0..4 * 3 * 2).map(|v| v as u8).collect();
        let frame = Frame {
            width: 3,
            height: 2,
            rgba: &rgba,
        };
        let once = flipped_rows(frame);
        assert_eq!(&once[..12], &rgba[12..]);
        let twice = flipped_rows(Frame {
            rgba: &once,
            ..frame
        });
        assert_eq!(twice, rgba);
    }

    #[test]
    fn scene_points_map_y_up_onto_top_row_first_pixels() {
        let rgba = vec![0; 160 * 90 * 4];
        let frame = Frame {
            width: 160,
            height: 90,
            rgba: &rgba,
        };
        let shape = FrameShape::default_for(160, 90);
        let (x, y) = to_pixel(frame, shape, [0.0, 4.0, 0.0]);
        assert!(
            (x - 80.0).abs() < 1e-9 && y.abs() < 1e-9,
            "UP*4 is the top edge"
        );
        let (x, y) = to_pixel(frame, shape, [-shape.width_units / 2.0, -4.0, 0.0]);
        assert!(x.abs() < 1e-9 && (y - 90.0).abs() < 1e-9);
    }
}
