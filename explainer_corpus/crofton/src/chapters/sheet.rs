//! The ruled sheet laid over the card (chapters 02 and 03): where its lines
//! go, where a curve crosses them in the order the curve meets them, the
//! beach redrawn as half circles (03_2), and the sheet and its crossings as
//! drawn, with the places both chapters use so that 02 ends where 03
//! begins. Card centimetres unless a name says frame. Lines run at `beta`
//! radians, a line's height measured along the normal [−sin β, cos β] as
//! [`Curve::meetings`] measures it: at β = 0 the lines run left to right and
//! a height is a y.

use std::f64::consts::{PI, TAU};

use explainer_kit::Palette;
use fmn::prelude::*;

use super::draw::{card_point, dot, flat, path};
use crate::model::{BEACH, CARD_H, CARD_UNIT, CARD_W, Curve, GAP, P2, Part, Seg};
use crate::palette;

// ------------------------------------------------------------ the geometry

/// Shown sheets slide in steps of a twentieth of a centimetre (the
/// oracle's `SLIDE_STEP`).
const STEPS_PER_CM: f64 = 20.0;

fn normal(beta: f64) -> P2 {
    [-beta.sin(), beta.cos()]
}

fn dot2(a: P2, b: P2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

/// Every height across lines at `beta` where a count can change: each
/// piece's ends, and each arc's tangent points inside it (`turning_heights`
/// in `oracle.py`).
pub fn turning_heights(curve: &Curve, beta: f64) -> Vec<f64> {
    let n = normal(beta);
    let psi = n[1].atan2(n[0]);
    let mut out = Vec::new();
    for part in &curve.parts {
        out.extend([dot2(part.seg.point(0.0), n), dot2(part.seg.point(1.0), n)]);
        if let Seg::Arc {
            centre,
            radius,
            from,
            to,
        } = part.seg
        {
            let (lo, span) = (from.min(to), (to - from).abs());
            // the arc faces straight along the normal at psi, against it at psi + π
            for (k, sign) in [(0.0, 1.0), (1.0, -1.0)] {
                let u = (psi + k * PI - lo).rem_euclid(TAU);
                if u > 0.0 && u < span {
                    out.push(dot2(centre, n) + sign * radius);
                }
            }
        }
    }
    out
}

/// How close any turning height comes to a line of the sheet slid by
/// `slide`, cm.
pub fn clearance(curve: &Curve, beta: f64, slide: f64) -> f64 {
    turning_heights(curve, beta)
        .into_iter()
        .map(|h| {
            let t = (h - slide) / GAP;
            (t - t.round()).abs() * GAP
        })
        .fold(f64::INFINITY, f64::min)
}

/// The placement every sheet shown on screen uses (`shown_slide` in
/// `oracle.py`): of the slides 0, 0.05, … short of one gap, the one whose
/// lines keep clearest of the turning heights (to 1e-9 cm), the smaller on
/// a tie.
pub fn shown_slide(curve: &Curve, beta: f64) -> f64 {
    let steps = (GAP * STEPS_PER_CM).round() as usize;
    let mut best = (f64::NEG_INFINITY, 0.0);
    for k in 0..steps {
        let slide = k as f64 / STEPS_PER_CM;
        let clear = (clearance(curve, beta, slide) * 1e9).round();
        if clear > best.0 {
            best = (clear, slide);
        }
    }
    best.1
}

/// The heights of a sheet's lines across the card (β = 0) slid by `slide`,
/// lowest first.
pub fn lines_on_card(slide: f64) -> Vec<f64> {
    let half = CARD_H / 2.0;
    let first = ((-half - slide) / GAP).ceil() as i64;
    (first..)
        .map(|k| slide + k as f64 * GAP)
        .take_while(|&h| h < half)
        .filter(|&h| h > -half)
        .collect()
}

/// Where a curve meets a line of the sheet.
#[derive(Clone, Copy, Debug)]
pub struct Crossing {
    pub at: P2,
    /// How far along the curve, cm.
    pub along: f64,
    /// Its piece, an index into the curve's parts.
    pub piece: usize,
    /// Its line: the one at `slide + line × GAP`.
    pub line: i64,
}

/// The lines `slide + k × GAP` from at or below `lo` to at or above `hi`.
fn lines_spanning(lo: f64, hi: f64, slide: f64) -> impl Iterator<Item = (i64, f64)> {
    let (k0, k1) = (((lo - slide) / GAP).floor() as i64, ((hi - slide) / GAP).ceil() as i64);
    (k0..=k1).map(move |k| (k, slide + k as f64 * GAP))
}

/// Every crossing of `curve` with the lines at `beta` slid by `slide`, in
/// the order the curve meets them. A straight run crosses a line its ends
/// lie strictly either side of, an arc wherever the line meets its circle
/// at an angle strictly inside it, as [`Curve::meetings`] counts.
pub fn crossings(curve: &Curve, beta: f64, slide: f64) -> Vec<Crossing> {
    let n = normal(beta);
    let psi = n[1].atan2(n[0]);
    let mut out = Vec::new();
    for (piece, part) in curve.parts.iter().enumerate() {
        let mut here = Vec::new();
        match part.seg {
            Seg::Line { from, to } => {
                let (a, b) = (dot2(from, n), dot2(to, n));
                for (line, h) in lines_spanning(a.min(b), a.max(b), slide) {
                    if (a - h) * (b - h) < 0.0 {
                        let t = (h - a) / (b - a);
                        here.push(Crossing {
                            at: part.seg.point(t),
                            along: part.start + t * part.seg.len(),
                            piece,
                            line,
                        });
                    }
                }
            }
            Seg::Arc {
                centre,
                radius,
                from,
                to,
            } => {
                let c = dot2(centre, n);
                let (lo, span) = (from.min(to), (to - from).abs());
                for (line, h) in lines_spanning(c - radius, c + radius, slide) {
                    let q = (h - c) / radius;
                    if q.abs() >= 1.0 {
                        continue;
                    }
                    for theta in [psi + q.acos(), psi - q.acos()] {
                        let u = (theta - lo).rem_euclid(TAU);
                        if u > 0.0 && u < span {
                            // how far round from the arc's start
                            let w = if to > from { u } else { span - u };
                            here.push(Crossing {
                                at: part.seg.point(w / span),
                                along: part.start + radius * w,
                                piece,
                                line,
                            });
                        }
                    }
                }
            }
        }
        here.sort_by(|a, b| a.along.total_cmp(&b.along));
        out.extend(here);
    }
    out
}

/// How many crossings each line holds, lowest line first, lines with none
/// left out: (line, count).
pub fn per_line(crossings: &[Crossing]) -> Vec<(i64, usize)> {
    let mut lines: Vec<(i64, usize)> = Vec::new();
    for c in crossings {
        match lines.iter_mut().find(|(k, _)| *k == c.line) {
            Some((_, n)) => *n += 1,
            None => lines.push((c.line, 1)),
        }
    }
    lines.sort_by_key(|&(k, _)| k);
    lines
}

/// `curve` with its beach, a straight run, redrawn as `count` half circles
/// side by side, each bulging to the left of the beach's direction (above
/// it, on this coast) and meeting the next at a corner: the beach keeps its
/// ends and every other piece stays where it was (`rippled_coast` in
/// `oracle.py`). Whatever the count, the half circles are π/2 times the
/// beach.
pub fn rippled(curve: &Curve, count: usize) -> Result<Curve, String> {
    if count == 0 {
        return Err("a beach needs at least one half circle".into());
    }
    let Seg::Line { from, to } = curve.parts[BEACH].seg else {
        return Err("the beach must be a straight run".into());
    };
    let len = curve.parts[BEACH].seg.len();
    let d = [(to[0] - from[0]) / len, (to[1] - from[1]) / len];
    let heading = d[1].atan2(d[0]);
    let radius = len / (2.0 * count as f64);
    let ripples = (0..count).map(|k| {
        let along = (2 * k + 1) as f64 * radius;
        Seg::Arc {
            centre: [from[0] + along * d[0], from[1] + along * d[1]],
            radius,
            from: heading + PI,
            to: heading,
        }
    });
    let segs: Vec<Seg> = curve.parts[..BEACH]
        .iter()
        .map(|p| p.seg)
        .chain(ripples)
        .chain(curve.parts[BEACH + 1..].iter().map(|p| p.seg))
        .collect();
    let mut parts = Vec::with_capacity(segs.len());
    let mut start = 0.0;
    for seg in segs {
        parts.push(Part { seg, start });
        start += seg.len();
    }
    Ok(Curve { parts, length: start })
}

// ---------------------------------------------------------- the sheet drawn

/// The sheet's lines stop this far inside the card's sides, cm, clear of
/// its rounded corners.
pub const LINE_INSET: f64 = 0.25;
pub const LINE_W: f64 = 2.0;
/// A line picked out (02_7): brighter and wider, its fill still empty.
pub const LINE_PICKED_W: f64 = 3.5;
/// The clear sheet's tint.
const PANEL_OPACITY: f64 = 0.06;
/// A crossing's dot, frame units.
pub const DOT_R: f64 = 0.08;

/// The sheet as drawn: a clear panel the card's size and its lines across
/// the card, lowest first, with their heights. Detached, `shift` frame units
/// to the right of where it lies on the card (0 to lie on it).
pub struct Sheet {
    pub panel: Mob,
    pub lines: Vec<Mob>,
    pub heights: Vec<f64>,
    /// The line index (as [`Crossing::line`] counts) of the lowest line.
    pub lowest: i64,
}

impl Sheet {
    pub fn new(stage: &mut Stage<'_>, slide: f64, shift: f64) -> fmn::Result<Self> {
        let panel = Rectangle::new()
            .width(CARD_W * CARD_UNIT)
            .height(CARD_H * CARD_UNIT)
            .corner_radius(0.12)
            .style(flat(Palette::ink(), PANEL_OPACITY))
            .build()?
            .moved_to(card_point([0.0, 0.0]));
        let panel = stage.arena_mut().add(panel);
        let heights = lines_on_card(slide);
        let x = CARD_W / 2.0 - LINE_INSET;
        let lines = heights
            .iter()
            .map(|&h| path(stage, &[[-x, h], [x, h]], Palette::dim(), LINE_W))
            .collect();
        let lowest = ((heights[0] - slide) / GAP).round() as i64;
        let sheet = Self {
            panel,
            lines,
            heights,
            lowest,
        };
        if shift != 0.0 {
            for m in sheet.all() {
                stage.shift(m, [shift, 0.0, 0.0]);
            }
        }
        Ok(sheet)
    }

    /// The panel, then the lines (the order they are drawn in).
    pub fn all(&self) -> Vec<Mob> {
        let mut all = vec![self.panel];
        all.extend(&self.lines);
        all
    }

    /// The drawn line a crossing is on.
    pub fn line(&self, c: &Crossing) -> Option<Mob> {
        usize::try_from(c.line - self.lowest)
            .ok()
            .and_then(|i| self.lines.get(i).copied())
    }
}

/// A crossing's dot, gold like 01's touch point. Detached.
pub fn crossing_dot(stage: &mut Stage<'_>, at: P2) -> Mob {
    dot(stage, at, DOT_R, palette::touch())
}

/// "7 crossings".
pub fn count_words(n: usize) -> String {
    format!("{n} crossing{}", if n == 1 { "" } else { "s" })
}

// ------------------------------------------- places chapters 02 and 03 share

/// The band over the coast, between the shown sheet's two top lines (frame
/// y 2.21 and 3.81): the question, the count, the envelope.
pub const TOP_Y: f64 = 3.0;
/// The count, where 02_5's question stood.
pub const COUNT_AT: Vec3 = [-2.6, TOP_Y, 0.0];
pub const COUNT_SIZE: f64 = 30.0;
/// The think ring, in the band's right half; its note falls just under the
/// band's lower line, over empty card above the bay.
pub const THINK_AT: Vec3 = [3.6, 2.9, 0.0];
/// The band under the coast, between its lowest point and the sheet's
/// lowest line (frame y −1.37 and −2.59).
pub const LOW_Y: f64 = -2.0;
/// The "2 cm" bracket between the sheet's two lowest lines (02_3, 03_4),
/// near the card's right edge where the coast stays above them, cm; its
/// label to its left.
pub const BRACKET_X: f64 = 7.2;
const BRACKET_TICK: f64 = 0.2;
const GAP_LABEL_OFF: f64 = 0.75;
pub const LABEL_SIZE: f64 = 22.0;

/// The bracket's corners, cm, for the lines at `heights` (lowest first).
pub fn gap_bracket(heights: &[f64]) -> [P2; 4] {
    let (lo, hi) = (heights[0], heights[1]);
    [
        [BRACKET_X - BRACKET_TICK, lo],
        [BRACKET_X, lo],
        [BRACKET_X, hi],
        [BRACKET_X - BRACKET_TICK, hi],
    ]
}

/// Where the bracket's "2 cm" goes, cm.
pub fn gap_label_at(heights: &[f64]) -> P2 {
    [BRACKET_X - GAP_LABEL_OFF, (heights[0] + heights[1]) / 2.0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::fixed;
    use crate::model::{self, CARD_CENTRE, HEADLAND, PIECE_NAMES};
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    fn frame_y(cm: f64) -> f64 {
        CARD_CENTRE[1] + CARD_UNIT * cm
    }

    /// The shown sheet lies where the oracle puts it (`open.slide`), its
    /// lines 0.397 cm clear of every turning height, five of them on the
    /// card; a slide through the beach's level would have no clearance.
    #[test]
    fn the_shown_sheet_lies_where_the_oracle_puts_it() {
        let coast = model::coast();
        let slide = shown_slide(&coast, 0.0);
        assert_eq!(fixed(slide, 2), "0.45");
        assert_eq!(fixed(clearance(&coast, 0.0, slide), 3), "0.397");
        let lines = lines_on_card(slide);
        let want = [-3.55, -1.55, 0.45, 2.45, 4.45];
        assert_eq!(lines.len(), want.len());
        assert!(lines.iter().zip(want).all(|(h, w)| (h - w).abs() < 1e-12), "{lines:?}");
        let Seg::Line { from, .. } = coast.parts[BEACH].seg else {
            panic!("the beach is a straight run")
        };
        assert!(clearance(&coast, 0.0, from[1].rem_euclid(GAP)) < 1e-12);
    }

    /// Seven crossings at the shown placement, met in the oracle's order
    /// (`open.crossing_pieces`, `open.crossing_lines`): four on the lower
    /// line, three on the upper (`open.per_line`), two lines in all
    /// (`open.lines_crossed`). Each lies on its line and on the coast, in
    /// order along it, and the model's own meetings agree line by line.
    #[test]
    fn the_shown_sheet_crosses_seven_times_in_walking_order() {
        let coast = model::coast();
        let slide = shown_slide(&coast, 0.0);
        let found = crossings(&coast, 0.0, slide);
        let names: Vec<&str> = found.iter().map(|c| PIECE_NAMES[c.piece]).collect();
        assert_eq!(names.join(","), "cove,climb,rise,fall,bay,bay,end");
        let lowest = found.iter().map(|c| c.line).min().unwrap();
        let lines: Vec<String> = found.iter().map(|c| (c.line - lowest).to_string()).collect();
        assert_eq!(lines.join(","), "0,0,1,1,0,0,1");
        let tally: Vec<usize> = per_line(&found).into_iter().map(|(_, n)| n).collect();
        assert_eq!(tally, [4, 3]);
        for w in found.windows(2) {
            assert!(w[1].along > w[0].along);
        }
        for c in &found {
            let h = slide + c.line as f64 * GAP;
            assert!((c.at[1] - h).abs() < 1e-9, "{c:?}");
            assert!(dist(coast.at(c.along), c.at) < 1e-9, "{c:?}");
            let part = coast.parts[c.piece];
            assert!(c.along > part.start && c.along < part.end(), "{c:?}");
        }
        for (line, n) in per_line(&found) {
            assert_eq!(coast.meetings(0.0, slide + line as f64 * GAP), n);
        }
        // the pieces either side of the headland are the rise and the fall
        assert_eq!((PIECE_NAMES[HEADLAND - 1], PIECE_NAMES[HEADLAND + 1]), ("rise", "fall"));
    }

    /// At placements all over (8 directions by 10 slides, each clear of
    /// the turning heights by more than 0.01 cm) the crossings are the
    /// model's meetings line by line, and the sign changes along the drawn
    /// coast at 0.002 cm apart, a count that knows nothing of arcs.
    #[test]
    fn crossings_agree_with_the_models_meetings_and_the_drawn_coast() {
        let coast = model::coast();
        let pts = coast.upto(coast.length, 0.002);
        let mut checked = 0;
        for i in 0..8 {
            let beta = PI * (i as f64 + 0.3) / 8.0;
            let n = normal(beta);
            for j in 0..10 {
                let slide = GAP * (j as f64 + 0.5) / 10.0;
                if clearance(&coast, beta, slide) <= 0.01 {
                    continue;
                }
                let found = crossings(&coast, beta, slide);
                let heights: Vec<f64> = pts.iter().map(|&p| dot2(p, n)).collect();
                let (lo, hi) = heights
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &h| (a.min(h), b.max(h)));
                let mut by_sign = 0;
                for (line, h) in lines_spanning(lo, hi, slide) {
                    let mine = found.iter().filter(|c| c.line == line).count();
                    assert_eq!(mine, coast.meetings(beta, h), "beta {beta}, slide {slide}, line {line}");
                    by_sign += heights.windows(2).filter(|w| (w[0] - h) * (w[1] - h) < 0.0).count();
                }
                assert_eq!(found.len(), by_sign, "beta {beta}, slide {slide}");
                for c in &found {
                    assert!((dot2(c.at, n) - (slide + c.line as f64 * GAP)).abs() < 1e-9);
                    assert!(dist(coast.at(c.along), c.at) < 1e-9);
                }
                checked += 1;
            }
        }
        assert!(checked >= 60, "only {checked} placements were clear");
    }

    /// 03_2's computed illustration. One big half circle on the beach makes
    /// the coast 1.998 cm longer and meets the lower line twice more: nine
    /// crossings. Seven small ones make it exactly as much longer and meet
    /// no line: seven crossings, at the same placement. Neither drawing
    /// meets itself or moves the coast's ends. More length alone does not
    /// mean more crossings at one placement.
    #[test]
    fn equal_extra_length_brings_two_more_crossings_or_none() {
        let coast = model::coast();
        let slide = shown_slide(&coast, 0.0);
        let beach = coast.parts[BEACH].seg.len();
        let Seg::Line { from, .. } = coast.parts[BEACH].seg else {
            panic!("the beach is a straight run")
        };
        let (one, seven) = (rippled(&coast, 1).unwrap(), rippled(&coast, 7).unwrap());
        for c in [&one, &seven] {
            assert!((c.length - coast.length - (PI / 2.0 - 1.0) * beach).abs() < 1e-12);
            assert!(!model::crosses_itself(c));
            let (a, b) = (c.ends(), coast.ends());
            assert!(dist(a.0, b.0) < 1e-12 && dist(a.1, b.1) < 1e-12);
        }
        assert_eq!(fixed(one.length - coast.length, 3), "1.998");
        assert_eq!(fixed(one.length - coast.length, 0), "2");
        let (a, b) = (crossings(&one, 0.0, slide), crossings(&seven, 0.0, slide));
        assert_eq!((a.len(), b.len()), (9, 7));
        let new: Vec<&Crossing> = a.iter().filter(|c| c.piece == BEACH).collect();
        assert_eq!(new.len(), 2);
        for (c, x) in new.iter().zip([-5.314, -1.922]) {
            assert_eq!((fixed(c.at[0], 3), fixed(c.at[1], 2)), (fixed(x, 3), "-1.55".into()));
        }
        assert!(b.iter().all(|c| !(BEACH..BEACH + 7).contains(&c.piece)));
        // the small tops stay 0.18 cm under the lower line; neither count is
        // borderline at this placement
        let top = from[1] + beach / 14.0;
        assert_eq!(fixed(-1.55 - top, 2), "0.18");
        assert_eq!(fixed(clearance(&one, 0.0, slide), 3), "0.397");
        assert_eq!(fixed(clearance(&seven, 0.0, slide), 3), "0.182");
    }

    /// The count depends on the placement, not only on the drawing: with
    /// the sheet slid so a line runs through the small half circles, they
    /// add fourteen crossings. Over every slide, at this direction, one big
    /// half circle and seven small ones add the same on average: each rises
    /// and falls as far in all as the beach is long (`Curve::travel`, in
    /// degrees), 1.75 crossings at 2 cm apart.
    #[test]
    fn the_count_depends_on_where_the_sheet_lies() {
        let coast = model::coast();
        let seven = rippled(&coast, 7).unwrap();
        let one = rippled(&coast, 1).unwrap();
        let beach = coast.parts[BEACH].seg.len();
        let Seg::Line { from, .. } = coast.parts[BEACH].seg else {
            panic!("the beach is a straight run")
        };
        let through = (from[1] + 0.15).rem_euclid(GAP);
        assert!(clearance(&seven, 0.0, through) > 0.05);
        let gained = crossings(&seven, 0.0, through).len() - crossings(&coast, 0.0, through).len();
        assert_eq!(gained, 14);
        for c in [&one, &seven] {
            let extra = c.travel(0.0) - coast.travel(0.0);
            assert!((extra - beach).abs() < 1e-12, "{extra}");
        }
        assert_eq!(fixed(beach / GAP, 2), "1.75");
    }

    #[test]
    fn a_beach_of_no_half_circles_is_refused() {
        assert!(rippled(&model::coast(), 0).is_err());
    }

    /// The places 02 and 03 share keep their rooms: the top band holds no
    /// coast and lies between the sheet's two top lines; the think ring
    /// stays inside it and its note over empty card; the low band lies
    /// under the coast and over the lowest line.
    #[test]
    fn the_shared_places_keep_their_rooms() {
        use super::super::draw::THINK_R;
        let coast = model::coast();
        let lines: Vec<f64> = lines_on_card(shown_slide(&coast, 0.0))
            .into_iter()
            .map(frame_y)
            .collect();
        let pts: Vec<Vec3> = coast.upto(coast.length, 0.02).into_iter().map(card_point).collect();
        let high = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        let low = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let (band_lo, band_hi) = (lines[3], lines[4]);
        assert!(high < band_lo - 0.3, "the coast reaches {high}");
        assert!(TOP_Y - 0.2 > band_lo && TOP_Y + 0.2 < band_hi);
        assert!(THINK_AT[1] - THINK_R > band_lo + 0.05 && THINK_AT[1] + THINK_R < band_hi - 0.05);
        let note_y = THINK_AT[1] - THINK_R - 0.3;
        assert!(note_y + 0.1 < band_lo, "the note sits under the band's lower line");
        for p in &pts {
            if (p[0] - THINK_AT[0]).abs() < 1.3 {
                assert!(p[1] < note_y - 0.35, "the coast at {p:?} reaches the think note");
            }
        }
        assert!(LOW_Y + 0.2 < low - 0.15 && LOW_Y - 0.2 > lines[0] + 0.15, "{low}");
        // the bracket and its label, where the coast does not reach
        let heights = lines_on_card(shown_slide(&coast, 0.0));
        let (box_lo, box_hi) = (
            [gap_label_at(&heights)[0] - 0.6, heights[0]],
            [BRACKET_X + 0.1, heights[1] + 0.3],
        );
        for p in coast.upto(coast.length, 0.02) {
            let inside = (box_lo[0]..=box_hi[0]).contains(&p[0]) && (box_lo[1]..=box_hi[1]).contains(&p[1]);
            assert!(!inside, "the coast at {p:?} meets the bracket");
        }
        const {
            assert!(BRACKET_X < CARD_W / 2.0 - LINE_INSET);
        }
        assert!(COUNT_AT[0].abs() + 1.2 < CARD_W * CARD_UNIT / 2.0);
        assert!(band_hi < FRAME_Y_RADIUS && CARD_UNIT * CARD_W / 2.0 < FRAME_X_RADIUS);
    }
}
