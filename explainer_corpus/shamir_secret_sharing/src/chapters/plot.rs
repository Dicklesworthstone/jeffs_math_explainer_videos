//! Chapter 07's paper plot (storyboard-draft, "07_a_line_through_the_secret"):
//! positions 0 to 5 across, ordinary heights up the side, nothing wrapped.
//! Each trustee's disc, numbered as on the rail, sits under its own
//! position, and the secret's gold disc under position 0. Points, lines and
//! the fan of lines through one point are drawn from data, so what a picture
//! shows is what the model says.
//!
//! Builders return detached mobjects, as draw.rs's. The plot takes the left
//! half over the rail's card frames; the strip keeps the right.

use explainer_kit::{BACKGROUND, Kit, Palette, hex, v};
use fmn::prelude::*;

use super::draw::{TRUSTEE_R, bundle, disc, fade_in_all, stroke};
use super::envelope::{PAD, outline};
use crate::palette;

// ------------------------------------------------------------------ layout

/// Position 0's place across and one step of position; height 0's place up
/// and one unit of height.
pub const PLOT_X0: f64 = -5.6;
pub const PLOT_DX: f64 = 0.85;
pub const PLOT_Y0: f64 = -0.3;
pub const PLOT_DY: f64 = 0.14;
/// The last position, and the tallest height the side is labelled to.
pub const LAST: u32 = 5;
pub const TALLEST: i64 = 25;
/// How far the axes run on past the last position and the tallest height.
pub const RUN_ON_X: f64 = 0.45;
pub const RUN_ON_Y: f64 = 0.25;
/// The labelled heights' step, their labels' size and how far left of the
/// axis they sit, and a tick's length.
const HEIGHT_STEP: usize = 5;
pub const LABEL_SIZE: f64 = 18.0;
pub const HEIGHT_LABEL_DX: f64 = 0.32;
const TICK: f64 = 0.08;
const AXIS_LINE: f64 = 2.5;
/// The axes' titles: over the side's top, and after the bottom's end.
pub const HEIGHT_TITLE: &str = "height";
pub const POSITION_TITLE: &str = "position";
pub const TITLE_GAP: f64 = 0.22;
// Keep the rendered height title clear of the top title-safe boundary.
pub const HEIGHT_TITLE_GAP: f64 = 0.14;
/// Where the position discs sit under the axis (the rail's size, so a disc
/// that lifts off the rail is the same disc).
pub const MARKER_DY: f64 = 0.42;
const MARKER_NUMBER_SIZE: f64 = 24.0;
/// The secret's name, left of position 0's disc.
pub const SECRET_NAME: &str = "secret";
pub const SECRET_NAME_SIZE: f64 = 16.0;
pub const SECRET_NAME_GAP: f64 = 0.1;
/// A point's radius; its label's size and place, up and to the left of the
/// point, clear of a line that climbs to the right.
pub const POINT_R: f64 = 0.08;
pub const POINT_LABEL_SIZE: f64 = 22.0;
pub const POINT_LABEL_DX: f64 = -0.26;
pub const POINT_LABEL_DY: f64 = 0.26;
/// A plotted line's width.
pub const LINE_W: f64 = 4.0;
/// The width of a band's outline round the side.
pub const BAND_W: f64 = 0.34;

/// The scene point at position `x`, height `h`.
pub fn at(x: f64, h: f64) -> Vec3 {
    v(PLOT_X0 + PLOT_DX * x, PLOT_Y0 + PLOT_DY * h)
}

/// The centre of position `x`'s disc, under the axis.
pub fn marker_at(x: u32) -> Vec3 {
    v(PLOT_X0 + PLOT_DX * f64::from(x), PLOT_Y0 - MARKER_DY)
}

/// Where the bottom axis ends, and the side's top.
pub fn axis_end() -> f64 {
    at(f64::from(LAST), 0.0)[0] + RUN_ON_X
}

pub fn axis_top() -> f64 {
    at(0.0, TALLEST as f64)[1] + RUN_ON_Y
}

/// The labelled heights: 0, 5, … up to the tallest.
pub fn labelled_heights() -> Vec<i64> {
    (0..=TALLEST).step_by(HEIGHT_STEP).collect()
}

// -------------------------------------------------------------------- data

/// The part, from position 0 to `x`, of the line through `(x, h)` that
/// climbs `slope` a step, kept to heights 0 to the tallest: its left and
/// right ends as (position, height). `None` when no part is inside. A line
/// whose height at 0 is below 0 stops where it meets the bottom axis.
pub fn through(x: f64, h: f64, slope: f64) -> Option<[(f64, f64); 2]> {
    let height = |t: f64| h + slope * (t - x);
    let top = TALLEST as f64;
    let (lo, hi) = if slope == 0.0 {
        if !(0.0..=top).contains(&h) {
            return None;
        }
        (0.0, x)
    } else {
        // where the line meets height 0 and the tallest height
        let (t0, t1) = (x - h / slope, x + (top - h) / slope);
        let (enter, leave) = if slope > 0.0 { (t0, t1) } else { (t1, t0) };
        (enter.max(0.0), leave.min(x))
    };
    (lo <= hi).then(|| [(lo, height(lo)), (hi, height(hi))])
}

/// The height at position 0 of the line through `(x, h)` climbing `slope` a
/// step: on paper, the secret that line would start from.
pub fn at_zero(x: i64, h: i64, slope: i64) -> i64 {
    h - slope * x
}

// ---------------------------------------------------------------- the plot

/// The axes: the bottom one with a tick under each position, the side with
/// a tick and a label at every fifth height, and their titles.
pub struct Plot {
    parts: Vec<Mob>,
    /// The side's labels, by height.
    labels: Vec<(i64, Mob)>,
}

impl Plot {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let origin = at(0.0, 0.0);
        let mut parts = vec![
            stroke(stage, vec![origin, v(axis_end(), PLOT_Y0)], Palette::dim(), AXIS_LINE),
            stroke(stage, vec![origin, v(PLOT_X0, axis_top())], Palette::dim(), AXIS_LINE),
        ];
        for x in 1..=LAST {
            let p = at(f64::from(x), 0.0);
            parts.push(stroke(stage, vec![p, v(p[0], p[1] - TICK)], Palette::dim(), AXIS_LINE));
        }
        let mut labels = Vec::new();
        for h in labelled_heights() {
            let p = at(0.0, h as f64);
            if h > 0 {
                parts.push(stroke(stage, vec![p, v(p[0] - TICK, p[1])], Palette::dim(), AXIS_LINE));
            }
            let label = kit.text(
                stage,
                &h.to_string(),
                LABEL_SIZE,
                Palette::dim(),
                v(PLOT_X0 - HEIGHT_LABEL_DX, p[1]),
            )?;
            parts.push(label);
            labels.push((h, label));
        }
        parts.push(kit.text(
            stage,
            HEIGHT_TITLE,
            LABEL_SIZE,
            Palette::dim(),
            v(PLOT_X0, axis_top() + HEIGHT_TITLE_GAP),
        )?);
        parts.push(kit.text_left(
            stage,
            POSITION_TITLE,
            LABEL_SIZE,
            Palette::dim(),
            v(axis_end() + TITLE_GAP / 2.0, PLOT_Y0),
        )?);
        Ok(Self { parts, labels })
    }

    /// Everything the axes are drawn with.
    pub fn parts(&self) -> &[Mob] {
        &self.parts
    }

    /// The side's label for height `h`, if it has one.
    pub fn label(&self, h: i64) -> Option<Mob> {
        self.labels.iter().find(|&&(k, _)| k == h).map(|&(_, m)| m)
    }

    /// Bring the axes in, over `rt`.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, self.parts(), rt)
    }
}

/// A numbered disc in `color` at `at`, drawn as the rail draws a trustee:
/// position `x`'s marker once it sits under the axis.
pub struct Marker {
    /// Disc and number together, to move as one.
    pub whole: Mob,
    /// The disc alone, the shape that changes color: 08_5 greys the other
    /// trustees' discs while their numbers stay on them.
    pub disc: Mob,
}

impl Marker {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, x: u32, color: Srgb, at: Vec3) -> fmn::Result<Self> {
        let d = disc(stage, color, at, TRUSTEE_R);
        let n = kit.text(stage, &x.to_string(), MARKER_NUMBER_SIZE, hex(BACKGROUND), at)?;
        Ok(Self {
            whole: bundle(stage, &[d, n])?,
            disc: d,
        })
    }
}

/// The secret's name, in its color, its right edge just left of position
/// 0's disc. Detached.
pub fn secret_name(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let right = v(PLOT_X0 - TRUSTEE_R - SECRET_NAME_GAP, PLOT_Y0 - MARKER_DY);
    let m = kit.text(stage, SECRET_NAME, SECRET_NAME_SIZE, palette::secret(), right)?;
    stage.move_to(m, right, RIGHT);
    Ok(m)
}

/// A point at position `x`, height `h`, in `color`. Detached.
pub fn point(stage: &mut Stage<'_>, x: f64, h: f64, color: Srgb) -> Mob {
    disc(stage, color, at(x, h), POINT_R)
}

/// Where a point's label sits.
pub fn label_at(x: f64, h: f64) -> Vec3 {
    let p = at(x, h);
    v(p[0] + POINT_LABEL_DX, p[1] + POINT_LABEL_DY)
}

/// The height `h` written by its point at position `x`, in `color`.
/// Detached.
pub fn point_label(kit: &Kit, stage: &mut Stage<'_>, x: f64, h: i64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, &h.to_string(), POINT_LABEL_SIZE, color, label_at(x, h as f64))
}

/// A straight line on the plot from `from` to `to` (position, height), in
/// `color`, `w` wide; it grows from `from`. Detached.
pub fn segment(stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), color: Srgb, w: f64) -> Mob {
    stroke(stage, vec![at(from.0, from.1), at(to.0, to.1)], color, w)
}

/// An outline round the side from height `lo` to `hi`, in `color`: how a
/// range of heights is pointed at. Detached.
pub fn band(stage: &mut Stage<'_>, lo: i64, hi: i64, color: Srgb) -> fmn::Result<Mob> {
    let (bottom, top) = (at(0.0, lo as f64)[1], at(0.0, hi as f64)[1]);
    outline(
        stage,
        v(PLOT_X0, (bottom + top) / 2.0),
        BAND_W,
        top - bottom + 2.0 * PAD,
        color,
    )
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{FRAME_H, FRAME_Y, STRIP_LEFT};
    use super::*;

    /// Generous text sizes, as the scenes' tests: 0.0055 units a point a
    /// character wide and 0.0065 a point tall.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// The fan through trustee 5's (5, 25), one line per spin 0 to 10:
    /// each runs from (5, 25) to position 0 at 25 − 5 × spin while that is
    /// 0 or more, and otherwise stops on the bottom axis at 5 − 25 / spin.
    #[test]
    fn the_fan_is_clipped_to_the_plot() {
        for spin in 0..=10_i64 {
            let ends = through(5.0, 25.0, spin as f64).expect("every fan line has a part inside");
            let [(x0, h0), (x1, h1)] = ends;
            assert!(
                (x1 - 5.0).abs() < 1e-12 && (h1 - 25.0).abs() < 1e-12,
                "{spin}: {ends:?}"
            );
            let zero = at_zero(5, 25, spin);
            if zero >= 0 {
                assert!(x0.abs() < 1e-12 && (h0 - zero as f64).abs() < 1e-12, "{spin}: {ends:?}");
            } else {
                let meets = 5.0 - 25.0 / spin as f64;
                assert!((x0 - meets).abs() < 1e-12 && h0.abs() < 1e-12, "{spin}: {ends:?}");
            }
        }
        // spin 10 meets the bottom at position 2.5; spin 0 is flat at 25
        assert_eq!(through(5.0, 25.0, 10.0), Some([(2.5, 0.0), (5.0, 25.0)]));
        assert_eq!(through(5.0, 25.0, 0.0), Some([(0.0, 25.0), (5.0, 25.0)]));
    }

    /// Lines that leave by the top, or never come in, are clipped too.
    #[test]
    fn lines_outside_are_clipped_or_absent() {
        // a line falling 10 a step to (5, 0) is 25 high at position 2.5,
        // and is cut there
        assert_eq!(through(5.0, 0.0, -10.0), Some([(2.5, 25.0), (5.0, 0.0)]));
        assert_eq!(through(5.0, 30.0, 0.0), None);
        assert_eq!(through(5.0, -1.0, 0.0), None);
        // a line through (2, −5) climbing 1: below 0 everywhere up to 2
        assert_eq!(through(2.0, -5.0, 1.0), None);
        assert_eq!(at_zero(3, 17, 4), 5);
        assert_eq!(at_zero(5, 25, 4), 5);
    }

    /// The plot keeps to the left half: its height labels and the
    /// secret's name inside the title-safe margin, its discs over the
    /// rail's card frames, its top under the frame's top, and the position
    /// title left of the strip.
    #[test]
    fn the_plot_keeps_inside_its_half() {
        const FRAMES_TOP: f64 = FRAME_Y + FRAME_H / 2.0;
        const {
            assert!(
                PLOT_Y0 - MARKER_DY - TRUSTEE_R > FRAMES_TOP + 0.2,
                "the position discs over the frames"
            );
            assert!(PLOT_DX > 2.0 * TRUSTEE_R + 0.2, "the position discs apart");
        }
        let widest = labelled_heights()
            .iter()
            .map(|h| text_w(&h.to_string(), LABEL_SIZE))
            .fold(0.0, f64::max);
        assert!(PLOT_X0 - HEIGHT_LABEL_DX - widest / 2.0 > -FRAME_X_RADIUS + 0.25);
        assert!(PLOT_X0 - HEIGHT_LABEL_DX + widest / 2.0 < PLOT_X0 - TICK - 0.05);
        let name_left = PLOT_X0 - TRUSTEE_R - SECRET_NAME_GAP - text_w(SECRET_NAME, SECRET_NAME_SIZE);
        assert!(name_left > -FRAME_X_RADIUS + 0.25, "{name_left}");
        // the secret's name sits between the side's 0 label and the frames
        let name_y = PLOT_Y0 - MARKER_DY;
        assert!(name_y + text_h(SECRET_NAME_SIZE) / 2.0 < PLOT_Y0 - text_h(LABEL_SIZE) / 2.0 - 0.1);
        assert!(name_y - text_h(SECRET_NAME_SIZE) / 2.0 > FRAMES_TOP + 0.2);
        let title_top = axis_top() + HEIGHT_TITLE_GAP + text_h(LABEL_SIZE) / 2.0;
        assert!(title_top < FRAME_Y_RADIUS - 0.3, "{title_top}");
        let tallest_label = label_at(f64::from(LAST), TALLEST as f64)[1] + text_h(POINT_LABEL_SIZE) / 2.0;
        assert!(tallest_label < FRAME_Y_RADIUS - 0.25, "{tallest_label}");
        let title_right = axis_end() + TITLE_GAP / 2.0 + text_w(POSITION_TITLE, LABEL_SIZE);
        assert!(title_right < STRIP_LEFT - 0.3, "{title_right}");
    }

    /// Data and scene agree: position 0 at height 0 is the corner, a step
    /// and a unit move by the constants, and a disc sits under each
    /// position.
    #[test]
    fn data_maps_onto_the_plot() {
        assert_eq!(at(0.0, 0.0), v(PLOT_X0, PLOT_Y0));
        assert!((at(1.0, 4.0)[0] - (PLOT_X0 + PLOT_DX)).abs() < 1e-12);
        assert!((at(1.0, 4.0)[1] - (PLOT_Y0 + 4.0 * PLOT_DY)).abs() < 1e-12);
        for x in 0..=LAST {
            assert!((marker_at(x)[0] - at(f64::from(x), 0.0)[0]).abs() < 1e-12);
        }
        assert_eq!(labelled_heights(), vec![0, 5, 10, 15, 20, 25]);
    }
}
