//! Chapter 11, transformations no steady real flow reaches: a mirror,
//! compared before and after (dashes pair each corner with where it
//! lands, no path), and the plain average of the two ends, ½(I + mirror),
//! squashing the plane flat (11_1); the signed area of a unit square under
//! a flow, on a gauge it never crosses at zero, forwards or backwards
//! (11_2); a positive determinant is not enough: diag(−2, −½), with a
//! prediction (11_3); suppose A = e^X: X commutes with its own e^X, seen
//! as two routes of u = (1, 1) that end together and proved term by term
//! from the powers of X (11_4); what commuting with A costs: the routes
//! A-then-X and X-then-A of u end ±1.5 × an off-diagonal entry of X apart
//! in each part, and meet only when those entries are zero (11_5); so X is
//! diagonal, its flow only stretches the axes by positive factors, and a
//! half turn then a stretch needs two flows (11_6); flipping both axes
//! alike is a half turn, why the argument fails there (equal stretches,
//! no gap), the 2×2 rule and Culver's exact condition (11_7); complex
//! entries remove the obstacle, and only flattening matrices have no
//! logarithm (11_8).
//!
//! Screen statements and their sources (checked 2026-11-04; quotes in
//! docs/sources.md): det e^{X} = e^{tr X}, and for a time-dependent linear
//! flow det = e^{∫ tr} (Liouville's formula). The real-logarithm condition
//! (a real A has a real logarithm iff A is nonsingular and has an even
//! number of Jordan blocks of each size for every negative eigenvalue) is
//! quoted from N. J. Higham, "Functions of Matrices", MIMS EPrint 2005.21,
//! the chapter for the Handbook of Linear Algebra (2006), section 4 Fact 3,
//! and agrees with J. Gallier, "Logarithms and Square Roots of Real
//! Matrices", arXiv 0805.0245, Theorem 3.4 (with proof). Both attribute it
//! to W. J. Culver, "On the existence and uniqueness of the real logarithm
//! of a matrix", Proc. Amer. Math. Soc. 17 (1966), 1146–1151, whose own
//! text sits behind the AMS bot check and was not read. The 2×2 rule on
//! screen is that condition for n = 2 (and 11_5's argument). Every
//! invertible complex matrix has a logarithm: Gallier Theorem 3.3 (with
//! proof, via the Jordan form).
//!
//! Screen layout: the plane panel at the left (x −5.8..−1.4), the signed
//! area gauge at x 0.9 with its word over its live number, a text column
//! from x 2.2. Proof captions break inside that column. A label on the panel
//! either fits one grid cell or stands on a plate (`on_plate`); the target F's
//! words are a key in the column, clear of the moving plane. In 11_7, with the
//! gauge gone, the sweep's words stand in the gap from x −1.15, and the block
//! rows run from x −5.2 to the gap, clear of the column.
//!
//! The picture lands on phrases through `until_phrase`: a word mark when the
//! narrator has one, otherwise a provisional share of the measured sentence.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::collections::HashMap;
use std::f64::consts::PI;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Kit, Palette, Script, hex, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, arc_points, arrow_parts, clipped_grid, dot, fact, fact_f64, fact_list, fact_matrix, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, flat, group, lap_c, letter_f, line_style, matrix_card, number_c,
    path, position_c, precisely, predict, ring, seg, shown, slide_c, swap, title, tween_all, value, warn_c,
};
use crate::model::{self, I2, J, M2};

pub struct Unreachable {
    pub kit: Kit,
}

const CENTRE: [f64; 3] = [-3.6, -0.3, 0.0];
const UNIT: f64 = 0.7;
/// Grid lines are kept within this many plane units of the centre.
const CLIP: f64 = 3.2;
/// The signed-area gauge: its x, the screen height of det = 0, and the
/// screen length of one unit of area; it shows −1.3 to 2.2.
const GAUGE_X: f64 = 0.9;
const GAUGE_ZERO: f64 = -0.9;
const GAUGE_PER: f64 = 1.35;
const GAUGE_RANGE: (f64, f64) = (-1.3, 2.2);
/// The gauge's tick numbers and the left end of its red zero line, apart so
/// that the line never strikes the 0.
const GAUGE_LABEL_X: f64 = GAUGE_X - 0.45;
const ZERO_LINE_LEFT: f64 = GAUGE_X - 0.25;
/// The readout over the gauge: its word, then its live number beneath, under
/// the title and above the gauge's top. A three-decimal number is about 0.55
/// wide, so this left edge centres it on the gauge.
const READOUT_WORD_Y: f64 = 2.72;
const READOUT_VALUE_Y: f64 = 2.36;
const READOUT_VALUE_X: f64 = GAUGE_X - 0.28;
/// The key to the target F in 11_6, offsets from the text column: a small copy
/// of the outline (the flipped F reaches 1.8 units left of its centre), then
/// its words.
const KEY_Y: f64 = -0.45;
const KEY_F_UNIT: f64 = 0.45;
const KEY_F_X: f64 = 1.8 * KEY_F_UNIT;
const KEY_WORD_X: f64 = 0.85;
/// The flow's clock runs over this range in 11_2 (forwards, then back).
const FLOW_TIMES: (f64, f64) = (-2.0, 3.0);

/// The words the film speaks, for the phrase hooks below.
const SCRIPT: &str = include_str!("../../script.tsv");
/// Phrases the picture lands on, each the exact words of its line.
const HALFWAY: &str = "a halfway point";
const SQUASHES: &str = "squashes the whole plane";
const ENDS_DIFFER: &str = "the two routes end in different places";
const KEEP_AXES: &str = "so nothing forces a logarithm to keep the axes";
const TURNS_EVERY: &str = "the spin, which turns every axis";
const SOME_DIRECTION: &str = "a matrix that sends some direction";
const EVERY_DIRECTION: &str = "only when it does that to every direction alike";
const HALF_TURN_STRETCH: &str = "a half turn combined with an equal stretch";
const BLOCKS: &str = "the matrix's blocks for each negative stretch";
const MATCHING: &str = "come in matching pairs";

/// 11_1: below this sideways scale the plane has gone flat, and its horizontal
/// lines are single points that no stroke can draw.
const FLAT_SCALE: f64 = 1e-3;
/// 11_7: how far the marked axes reach, in plane units.
const AXES_REACH: f64 = 3.0;
/// 11_7's sweep, in plane units: the length of the white direction, and how far
/// the dashes run straight back from it.
const DIRECTION_LEN: f64 = 1.2;
const BACK_REACH: f64 = 2.8;
/// The sweep checks directions this far apart, a whole number of steps to each axis.
const SWEEP_STEP: f64 = PI / 180.0;
/// The equal stretch of the sweep's last beat. It only shows that c may be any
/// positive number, so the screen never prints it.
const STRETCH_C: f64 = 1.5;
/// The left edge of the sweep's words, in the gap the gauge left.
const GAP_X: f64 = -1.15;
/// 11_7's block rows: the left tile's centre, the distance between tiles, a
/// tile's half height (and a plain tile's half width), a two-by-two block's
/// half width, the first row's height and the distance between rows.
const ROWS_X: f64 = -4.9;
const TILE_STEP: f64 = 0.74;
const TILE_HALF: f64 = 0.31;
const WIDE_TILE_HALF: f64 = 0.49;
const ROWS_TOP: f64 = 1.35;
const ROWS_PITCH: f64 = 1.0;
const BLOCKS_HEAD_Y: f64 = 2.2;
const VERDICT_X: f64 = -2.75;
/// A pair's bracket: where its feet stop and its bar runs, below a row's centre.
const BRACKET_TOP: f64 = 0.38;
const BRACKET_FOOT: f64 = 0.46;

/// A block of a matrix: the stretch on its diagonal, and its size.
type Block = (f64, usize);
/// One row per matrix, for the rule in 11_7, and what the rule says of it. The
/// second row is `matrix.flip_pair`; the others are illustrations, checked below
/// against the rule and the exponential.
const BLOCK_ROWS: [(&[Block], &str); 4] = [
    (&[(-1.0, 1), (-1.0, 1)], "matching pair: real log"),
    (&[(-2.0, 1), (-0.5, 1)], "no partners: no real log"),
    (&[(-2.0, 1), (-2.0, 1), (3.0, 1)], "−2 paired, 3 positive: real log"),
    (&[(-1.0, 2)], "one size-2 block: no real log"),
];

fn gauge(d: f64) -> Vec3 {
    v(GAUGE_X, GAUGE_ZERO + GAUGE_PER * d)
}

fn lerp(a: M2, b: M2, t: f64) -> M2 {
    model::madd(model::mscale(a, 1.0 - t), model::mscale(b, t))
}

/// A number for the screen: halves as fractions, a typographic minus.
fn shown_num(x: f64) -> String {
    if (x - 0.5).abs() < 1e-12 {
        "½".into()
    } else if (x + 0.5).abs() < 1e-12 {
        "\u{2212}½".into()
    } else {
        shown(&format!("{x}"))
    }
}

fn to_screen(q: [f64; 2]) -> Vec3 {
    v(CENTRE[0] + UNIT * q[0], CENTRE[1] + UNIT * q[1])
}

/// Stand a placed label on a plate of the background colour. One grid cell is
/// narrower than these words, so without it a grid line runs through a glyph.
fn on_plate(stage: &mut Stage<'_>, label: Mob) -> fmn::Result<Mob> {
    let plate = Rectangle::new()
        .width(stage.get_width(label) + 0.1)
        .height(stage.get_height(label) + 0.06)
        .style(flat(hex(BACKGROUND), 1.0))
        .build()?
        .moved_to(stage.get_center(label));
    let plate = stage.arena_mut().add(plate);
    group(stage, &[plate, label])
}

/// When `phrase` begins, in seconds from the start of a line with this `text`:
/// the narrator's word mark if it has one for that word, otherwise a share by
/// characters of the measured sentence the phrase sits in. The share is a
/// provisional draft cue, not a heard boundary: fine sync waits for word marks
/// measured on the takes. A phrase the line lacks, or a narrator that splits the
/// line differently, is an error, never a guess.
fn phrase_seconds(
    text: &str,
    phrase: &str,
    marks: &HashMap<usize, f64>,
    starts: &[f64],
    len: f64,
) -> Result<f64, String> {
    let byte = text
        .find(phrase)
        .ok_or_else(|| format!("the line has no phrase {phrase:?}"))?;
    if let Some(&seconds) = marks.get(&byte) {
        return Ok(seconds);
    }
    let mut opens = vec![0];
    for (i, pair) in text.as_bytes().windows(2).enumerate() {
        if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {
            opens.push(i + 2);
        }
    }
    if opens.len() != starts.len() {
        return Err(format!(
            "the line has {} sentences, but the narrator timed {}",
            opens.len(),
            starts.len()
        ));
    }
    let j = opens.iter().rposition(|&open| open <= byte).unwrap_or(0);
    let close = opens.get(j + 1).copied().unwrap_or(text.len());
    let (t0, t1) = (starts[j], starts.get(j + 1).copied().unwrap_or(len));
    let before = text[opens[j]..byte].chars().count() as f64;
    let whole = text[opens[j]..close].trim_end().chars().count().max(1) as f64;
    Ok(t0 + (t1 - t0) * before / whole)
}

/// The scene time `phrase` of line `id` begins at, once the line has been said;
/// `None` without a narrator.
fn phrase_time(kit: &Kit, id: &str, phrase: &str) -> fmn::Result<Option<f64>> {
    let Some(n) = kit.narrator.as_ref() else {
        return Ok(None);
    };
    let starts = n.sentence_starts(id);
    let (Some(&first), Some(said), Some(len)) = (starts.first(), n.sentence_at(id, 0), n.length(id)) else {
        return Ok(None);
    };
    let script = Script::parse(SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?;
    let seconds =
        phrase_seconds(text, phrase, &n.phrase_marks(id), &starts, len).map_err(|e| fail(format!("line {id}: {e}")))?;
    Ok(Some(said - first + seconds))
}

/// Wait until `phrase` of line `id` begins; nothing if that moment has passed.
fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, id: &str, phrase: &str) -> fmn::Result<()> {
    let Some(at) = phrase_time(kit, id, phrase)? else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

/// The scene time sentence `j` of line `id` begins at, once the line has been said.
fn sentence_time(kit: &Kit, id: &str, j: usize) -> Option<f64> {
    kit.narrator.as_ref().and_then(|n| n.sentence_at(id, j))
}

/// The scene time line `id` ends at, once it has been said.
fn line_end(kit: &Kit, id: &str) -> Option<f64> {
    let n = kit.narrator.as_ref()?;
    let first = *n.sentence_starts(id).first()?;
    Some(n.sentence_at(id, 0)? - first + n.length(id)?)
}

/// A run time that ends by scene time `by` if it can, kept between `shortest`
/// and `longest`; `longest` with nothing to end by.
fn run_by(by: Option<f64>, now: f64, shortest: f64, longest: f64) -> f64 {
    by.map_or(longest, |at| (at - now).clamp(shortest, longest))
}

/// A tracker's move at constant speed, for a sweep through directions.
fn steady(tracker: Mob, to: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?)
}

/// The plane under `m`, for a redraw: clipped grid, optionally the unit
/// square (blue, its signed area on the gauge), and the letter F.
fn plane(s: &mut Arena, m: M2, square: bool) -> Mob {
    let g = s.add(VMobject::new());
    let grid = clipped_grid(s, m, CENTRE, UNIT, 4, CLIP, number_c(), 0.6);
    let _ = s.attach(g, grid);
    if square {
        let corners: Vec<Vec3> = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .iter()
            .map(|&p| to_screen(model::apply(m, p)))
            .collect();
        let sq = s.add(Polygon::new(corners).style(flat(position_c(), 0.45)).build());
        let _ = s.attach(g, sq);
    }
    let f = s.add(
        Polygon::new(letter_f(m, CENTRE, UNIT))
            .style(flat(answer_c(), 0.9))
            .build(),
    );
    let _ = s.attach(g, f);
    g
}

/// The two-stage journey to diag(−e^{a}, −e^{b}) at time `t` in [0, 2]: a
/// half-turn spin (generator πJ) for t ≤ 1, then the stretch
/// diag(e^{a(t−1)}, e^{b(t−1)}) applied after −I. Two steady flows in turn.
fn two_stage(t: f64, a: f64, b: f64) -> M2 {
    if t <= 1.0 {
        model::flow(model::mscale(J, PI), t)
    } else {
        let u = t - 1.0;
        model::mmul([[(a * u).exp(), 0.0], [0.0, (b * u).exp()]], model::mscale(I2, -1.0))
    }
}

/// A steady flow that keeps both axes: diag(e^{ta}, e^{tb}).
fn diagonal_flow(a: f64, b: f64, t: f64) -> M2 {
    [[(a * t).exp(), 0.0], [0.0, (b * t).exp()]]
}

/// 11_5's candidate X: `base` with its off-diagonal entries scaled by `k`
/// (k = 1 the full candidate, k = 0 its diagonal part).
fn candidate(base: M2, k: f64) -> M2 {
    [[base[0][0], base[0][1] * k], [base[1][0] * k, base[1][1]]]
}

/// The points of the two routes of `u`: first `p` then `q`, and first `q`
/// then `p`, as (midway, end) for each.
fn route_points(p: M2, q: M2, u: [f64; 2]) -> [([f64; 2], [f64; 2]); 2] {
    let (mid_a, mid_b) = (model::apply(p, u), model::apply(q, u));
    [(mid_a, model::apply(q, mid_a)), (mid_b, model::apply(p, mid_b))]
}

/// The two routes of the probe `u`, for a redraw. The probe is an ink
/// arrow from the origin. Route one, first `p` and then `q`, is purple;
/// route two, first `q` and then `p`, is pink. Each route is two arrows,
/// from where the probe's tip is to where the next matrix sends it, with a
/// dot at the midway point. A red bar joins the two ends when they differ and
/// `show_gap` asks for it, and a gold dot marks them when they meet. The arrows
/// compare where a point lands; nothing travels along them.
fn routes(s: &mut Arena, p: M2, q: M2, u: [f64; 2], show_gap: bool) -> Mob {
    let g = s.add(VMobject::new());
    let probe = arrow_parts(s, to_screen([0.0, 0.0]), to_screen(u), Palette::ink(), 4.0, 0.2);
    let _ = s.attach(g, probe);
    let [(mid_a, end_a), (mid_b, end_b)] = route_points(p, q, u);
    for (mid, end, c) in [(mid_a, end_a, lap_c()), (mid_b, end_b, slide_c())] {
        for (from, to) in [(u, mid), (mid, end)] {
            let leg = arrow_parts(s, to_screen(from), to_screen(to), c, 3.5, 0.18);
            let _ = s.attach(g, leg);
        }
        let stop = s.add(
            Dot::new()
                .point(to_screen(mid))
                .radius(0.07)
                .style(flat(c, 1.0))
                .build(),
        );
        let _ = s.attach(g, stop);
    }
    if (end_a[0] - end_b[0]).hypot(end_a[1] - end_b[1]) > 0.02 {
        if show_gap {
            let bar = s.add(
                polyline(vec![to_screen(end_a), to_screen(end_b)])
                    .style(line_style(warn_c(), 6.0))
                    .build(),
            );
            let _ = s.attach(g, bar);
        }
    } else {
        let meet = s.add(
            Dot::new()
                .point(to_screen(end_a))
                .radius(0.12)
                .style(flat(answer_c(), 1.0))
                .build(),
        );
        let _ = s.attach(g, meet);
    }
    g
}

/// The F's corners used for 11_1's comparison dashes.
const CHORD_CORNERS: [usize; 4] = [0, 3, 7, 9];

/// Dashed chords from a few corners of the F to where `m` sends them: a
/// comparison of before and after, not a path anything travels.
fn chords(s: &mut Arena, m: M2) -> Mob {
    let g = s.add(VMobject::new());
    let (from, to) = (letter_f(I2, CENTRE, UNIT), letter_f(m, CENTRE, UNIT));
    for k in CHORD_CORNERS {
        let (a, b) = (from[k], to[k]);
        let at = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
        for d in 0..6 {
            let t0 = f64::from(d) / 6.0;
            let dash = s.add(
                polyline(vec![at(t0), at(t0 + 0.5 / 6.0)])
                    .style(faded_line(Palette::ink(), 2.5, 0.7))
                    .build(),
            );
            let _ = s.attach(g, dash);
        }
    }
    g
}

/// The middle of each comparison dash: where the plain average sends that corner.
fn chord_midpoints(m: M2) -> [Vec3; 4] {
    let (from, to) = (letter_f(I2, CENTRE, UNIT), letter_f(m, CENTRE, UNIT));
    CHORD_CORNERS.map(|k| v((from[k][0] + to[k][0]) / 2.0, (from[k][1] + to[k][1]) / 2.0))
}

/// 11_1's grid under the straight line from I towards the mirror, for a redraw.
/// Once flat, the whole grid is the one line it has become. The redraw stops at
/// the average: this compares matrices, it is no flow and never reaches the mirror.
fn averaged_grid(s: &mut Arena, mirror: M2, k: f64) -> Mob {
    let m = lerp(I2, mirror, k);
    if m[0][0].abs() > FLAT_SCALE {
        return clipped_grid(s, m, CENTRE, UNIT, 4, CLIP, number_c(), 0.5);
    }
    s.add(
        polyline(vec![to_screen([0.0, -CLIP]), to_screen([0.0, CLIP])])
            .style(faded_line(number_c(), 3.0, 0.5))
            .build(),
    )
}

/// The outline of the F on that same straight line, for a redraw.
fn averaged_outline(s: &mut Arena, mirror: M2, k: f64) -> Mob {
    let mut pts = letter_f(lerp(I2, mirror, k), CENTRE, UNIT);
    pts.push(pts[0]);
    s.add(polyline(pts).style(line_style(warn_c(), 6.0)).build())
}

/// The plane's two axes carried by `m`, at opacity `o`, for a redraw.
fn turning_axes(s: &mut Arena, m: M2, o: f64) -> Mob {
    let g = s.add(VMobject::new());
    for end in [[AXES_REACH, 0.0], [0.0, AXES_REACH]] {
        let q = model::apply(m, end);
        let line = s.add(
            polyline(vec![to_screen([-q[0], -q[1]]), to_screen(q)])
                .style(faded_line(Palette::ink(), 3.0, o))
                .build(),
        );
        let _ = s.attach(g, line);
    }
    g
}

/// Whether `m` sends the direction `d` straight back: to a negative multiple of itself.
fn reverses(m: M2, d: [f64; 2]) -> bool {
    let q = model::apply(m, d);
    (d[0] * q[1] - d[1] * q[0]).abs() < 1e-9 && d[0] * q[0] + d[1] * q[1] < 0.0
}

/// The runs of directions between the angles `from` and `to` that `m` sends
/// straight back. An isolated direction is a run of no width.
fn reversed_spans(m: M2, from: f64, to: f64) -> Vec<(f64, f64)> {
    let first = (from / SWEEP_STEP - 1e-9).ceil() as i32;
    let last = (to / SWEEP_STEP + 1e-9).floor() as i32;
    let mut spans: Vec<(f64, f64)> = Vec::new();
    let mut open = false;
    for j in first..=last {
        let a = f64::from(j) * SWEEP_STEP;
        if reverses(m, [a.cos(), a.sin()]) {
            if !open {
                spans.push((a, a));
            } else if let Some(span) = spans.last_mut() {
                span.1 = a;
            }
            open = true;
        } else {
            open = false;
        }
    }
    spans
}

/// The white direction at `theta`.
fn direction(theta: f64) -> [f64; 2] {
    [DIRECTION_LEN * theta.cos(), DIRECTION_LEN * theta.sin()]
}

/// 11_7's sweep, for a redraw. A white direction turns; a gold arrow shows where
/// the matrix sends it; dashes run straight back from it, so the gold lies on the
/// dashes exactly when the direction is sent straight back; and a purple mark
/// stays on each such direction passed so far. The matrix is `pair` for the first
/// half lap (to π), then moves to −cI by `to_equal`, and the marks after π are
/// those of −I, which reverses what −cI reverses.
fn sweep(s: &mut Arena, pair: M2, theta: f64, to_equal: f64, c: f64) -> Mob {
    let g = s.add(VMobject::new());
    let o = to_screen([0.0, 0.0]);
    let mut spans = reversed_spans(pair, 0.0, theta.min(PI));
    if theta > PI {
        spans.extend(reversed_spans(model::mscale(I2, -1.0), PI, theta));
    }
    for (a, b) in spans {
        let mark = if b - a < 1e-9 {
            s.add(
                Dot::new()
                    .point(to_screen(direction(a)))
                    .radius(0.07)
                    .style(flat(lap_c(), 1.0))
                    .build(),
            )
        } else {
            let n = ((b - a) / 0.1).ceil() as usize + 1;
            s.add(
                polyline(arc_points(o, DIRECTION_LEN * UNIT, a, b, n))
                    .style(line_style(lap_c(), 5.0))
                    .build(),
            )
        };
        let _ = s.attach(g, mark);
    }
    let back = [-BACK_REACH * theta.cos(), -BACK_REACH * theta.sin()];
    for k in 0..7 {
        let (t0, t1) = (f64::from(k) / 7.0, (f64::from(k) + 0.55) / 7.0);
        let dash = s.add(
            polyline(vec![
                to_screen([back[0] * t0, back[1] * t0]),
                to_screen([back[0] * t1, back[1] * t1]),
            ])
            .style(faded_line(Palette::dim(), 2.5, 0.9))
            .build(),
        );
        let _ = s.attach(g, dash);
    }
    let d = direction(theta);
    let m = lerp(pair, model::mscale(I2, -c), to_equal);
    let image = arrow_parts(s, o, to_screen(model::apply(m, d)), answer_c(), 4.0, 0.2);
    let _ = s.attach(g, image);
    let white = arrow_parts(s, o, to_screen(d), Palette::ink(), 4.0, 0.2);
    let _ = s.attach(g, white);
    g
}

/// Whether block `i` has a partner: blocks with its stretch and size come in an
/// even number.
fn paired(blocks: &[Block], i: usize) -> bool {
    let (stretch, size) = blocks[i];
    blocks
        .iter()
        .filter(|&&(s, n)| (s - stretch).abs() < 1e-12 && n == size)
        .count()
        .is_multiple_of(2)
}

/// The rule on screen for an invertible real matrix given by its blocks: for
/// every negative stretch, the blocks of each size occur an even number of times.
fn pairs_up(blocks: &[Block]) -> bool {
    (0..blocks.len()).all(|i| blocks[i].0 > 0.0 || paired(blocks, i))
}

/// A tile's half width for a block of this size.
fn tile_half(size: usize) -> f64 {
    if size > 1 { WIDE_TILE_HALF } else { TILE_HALF }
}

/// The centre x of block `i`'s tile: left edges start together.
fn tile_x(blocks: &[Block], i: usize) -> f64 {
    ROWS_X + TILE_STEP * i as f64 + tile_half(blocks[i].1) - TILE_HALF
}

/// The outline of a tile.
fn tile_box(stage: &mut Stage<'_>, at: Vec3, size: usize, c: Srgb, w: f64) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(2.0 * tile_half(size))
        .height(2.0 * TILE_HALF)
        .style(line_style(c, w))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// A block as a tile: its stretch in a faint box, or a two-by-two block's own
/// entries. A positive stretch is dim, since the rule asks nothing of it.
fn block_tile(kit: &Kit, stage: &mut Stage<'_>, block: Block, at: Vec3) -> fmn::Result<Mob> {
    let (stretch, size) = block;
    if size > 1 {
        let e = stretch.to_string();
        let entries = [[e.as_str(), "1"], ["0", e.as_str()]];
        return matrix_card(kit, stage, entries, at, 20.0, Palette::ink());
    }
    let c = if stretch < 0.0 { Palette::ink() } else { Palette::dim() };
    let frame = tile_box(stage, at, size, Palette::faint(), 2.0)?;
    let word = kit.text(stage, &shown_num(stretch), 24.0, c, at)?;
    group(stage, &[frame, word])
}

/// 11_7's rows, built but not yet shown: the tiles of each row; each row's
/// outlines and verdict, green where paired and red where not; and a gold
/// bracket under each pair.
struct BlockRows {
    tiles: Vec<Mob>,
    verdicts: Vec<Mob>,
    brackets: Vec<Mob>,
}

fn block_rows(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<BlockRows> {
    let mut rows = BlockRows {
        tiles: Vec::new(),
        verdicts: Vec::new(),
        brackets: Vec::new(),
    };
    for (row, (blocks, verdict)) in BLOCK_ROWS.iter().enumerate() {
        let y = ROWS_TOP - ROWS_PITCH * row as f64;
        let (mut tiles, mut says) = (Vec::new(), Vec::new());
        for (i, &block) in blocks.iter().enumerate() {
            let at = v(tile_x(blocks, i), y);
            tiles.push(block_tile(kit, stage, block, at)?);
            if block.0 < 0.0 {
                let c = if paired(blocks, i) { number_c() } else { warn_c() };
                says.push(tile_box(stage, at, block.1, c, 3.0)?);
            }
        }
        rows.tiles.push(group(stage, &tiles)?);
        let c = if pairs_up(blocks) { number_c() } else { warn_c() };
        says.push(kit.text_left(stage, verdict, 22.0, c, v(VERDICT_X, y))?);
        rows.verdicts.push(group(stage, &says)?);
        let partners: Vec<usize> = (0..blocks.len())
            .filter(|&i| blocks[i].0 < 0.0 && paired(blocks, i))
            .collect();
        if let (Some(&a), Some(&b)) = (partners.first(), partners.last()) {
            let (left, right) = (tile_x(blocks, a) - TILE_HALF, tile_x(blocks, b) + TILE_HALF);
            let pts = vec![
                v(left, y - BRACKET_TOP),
                v(left, y - BRACKET_FOOT),
                v(right, y - BRACKET_FOOT),
                v(right, y - BRACKET_TOP),
            ];
            rows.brackets.push(path(stage, pts, answer_c(), 3.0));
        }
    }
    Ok(rows)
}

impl SceneConstruct for Unreachable {
    fn name(&self) -> &str {
        "11_unreachable"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());
        let right = 2.2;
        let mirror: M2 = [[-1.0, 0.0], [0.0, 1.0]];
        let pair = fact_list("matrix.flip_pair")?;
        let flip_pair: M2 = [[pair[0], 0.0], [0.0, pair[1]]];
        // the stretch rates of the flip pair's halves, exact for the motion
        let (la, lb) = (pair[0].abs().ln(), pair[1].abs().ln());
        // 11_4/11_5: the probe vector (two nonzero parts) and the candidate X
        let probe = fact_list("matrix.probe")?;
        let u = [probe[0], probe[1]];
        let base = fact_matrix("matrix.probe_candidate")?;

        // 11_1: a mirror, compared before and after; the plain average of the two ends is flat
        kit.say(stage, "11_1")?;
        let heading = title(&kit, stage, "Places no steady flow reaches")?;
        fade_in_all(stage, &[heading], 0.5)?;
        kit.until_sentence(stage, "11_1", 1)?;
        let m_card = matrix_card(&kit, stage, [["-1", "0"], ["0", "1"]], v(5.4, 1.6), 32.0, red)?;
        let m_word = kit.text(stage, "mirror", 24.0, red, v(5.4, 2.2))?;
        fade_in_all(stage, &[m_card, m_word], 0.5)?;
        kit.until_sentence(stage, "11_1", 2)?;
        let start_grid = clipped_grid(stage.arena_mut(), I2, CENTRE, UNIT, 4, CLIP, green, 0.5);
        let before_f = stage.arena_mut().add(
            Polygon::new(letter_f(I2, CENTRE, UNIT))
                .style(flat(Palette::dim(), 0.65))
                .build(),
        );
        // each word fits one grid cell: centred in it, no grid line touches it
        let before_word = kit.text(stage, "before", 20.0, Palette::dim(), to_screen([0.5, 1.25]))?;
        fade_in_all(stage, &[start_grid, before_f, before_word], 0.5)?;
        kit.until_sentence(stage, "11_1", 3)?;
        let after_f = stage.arena_mut().add(
            Polygon::new(letter_f(mirror, CENTRE, UNIT))
                .style(flat(gold, 0.9))
                .build(),
        );
        let joins = chords(stage.arena_mut(), mirror);
        let after_word = kit.text(stage, "after", 20.0, gold, to_screen([-0.5, 1.25]))?;
        let ends_note = kit.text_left(
            stage,
            "dashes pair each corner with where it lands",
            20.0,
            Palette::dim(),
            v(right, 0.75),
        )?;
        fade_in_all(stage, &[joins, after_f, after_word, ends_note], 0.6)?;
        kit.until_sentence(stage, "11_1", 4)?;
        // the plain average ½(I + mirror) = diag(0, 1): every point onto the vertical axis
        let avg_a = kit.text_left(
            stage,
            &format!(
                "the average ½(I + mirror) has det {}:",
                fact("matrix.mirror_average_det")?
            ),
            22.0,
            red,
            v(right, 0.1),
        )?;
        fade_in_all(stage, &[avg_a], 0.5)?;
        // the average sends each corner to the middle of its dash: all on one line
        until_phrase(&kit, stage, "11_1", HALFWAY)?;
        let mids = chord_midpoints(mirror).map(|p| dot(stage, p, 0.07, red));
        swap(stage, &[before_word, after_word], &mids, 0.4)?;
        // then the whole grid goes there. The dashes leave first: they only pair
        // corners, and the moving corners must not read as travelling along them
        until_phrase(&kit, stage, "11_1", SQUASHES)?;
        fade_out_all(stage, &[joins], 0.3)?;
        let squash = stage.add_value_tracker(0.0);
        let collapsing = stage.always_redraw(move |s: &mut Arena| averaged_grid(s, mirror, value(s, squash)));
        // the static grid's slot in the draw list, under both Fs; at k = 0 they match
        stage.scene_mut().replace(start_grid, &[collapsing])?;
        let flat_f = stage.always_redraw(move |s: &mut Arena| averaged_outline(s, mirror, value(s, squash)));
        stage.add_to_scene(flat_f)?;
        let now = stage.scene().time().to_f64();
        // done with room for the last fade before the line ends
        let by = line_end(&kit, "11_1").map(|end| end - 0.5);
        play!(stage; tween(squash, 0.5, run_by(by, now, 1.2, 2.6))?);
        let whole_line = seg(stage, to_screen([0.0, -CLIP]), to_screen([0.0, CLIP]), red, 3.0);
        let avg_b = kit.text_left(
            stage,
            "it squashes the plane flat onto a line",
            22.0,
            red,
            v(right, -0.35),
        )?;
        fade_in_all(stage, &[whole_line, avg_b], 0.4)?;

        // 11_2: the signed area along a flow starts at 1 and never reaches 0
        kit.say(stage, "11_2")?;
        let mut first_picture = vec![
            collapsing, before_f, after_f, ends_note, flat_f, whole_line, avg_a, avg_b,
        ];
        first_picture.extend(mids);
        fade_out_all(stage, &first_picture, 0.4)?;
        let x = fact_matrix("matrix.flow")?;
        let time = stage.add_value_tracker(0.0);
        let flowing = stage.always_redraw(move |s: &mut Arena| plane(s, model::flow(x, value(s, time)), true));
        stage.add_to_scene(flowing)?;
        let mut scale = vec![seg(
            stage,
            gauge(GAUGE_RANGE.0),
            gauge(GAUGE_RANGE.1),
            Palette::faint(),
            2.5,
        )];
        for k in -1..=2 {
            let y = gauge(f64::from(k))[1];
            scale.push(seg(
                stage,
                v(GAUGE_X - 0.1, y),
                v(GAUGE_X + 0.1, y),
                Palette::dim(),
                2.0,
            ));
            scale.push(kit.text(stage, &shown(&k.to_string()), 18.0, Palette::dim(), v(GAUGE_LABEL_X, y))?);
        }
        let zero_line = seg(
            stage,
            v(ZERO_LINE_LEFT, GAUGE_ZERO),
            v(GAUGE_X + 0.35, GAUGE_ZERO),
            red,
            4.0,
        );
        let area_word = kit.text(
            stage,
            "signed area of the square:",
            22.0,
            blue,
            v(GAUGE_X, READOUT_WORD_Y),
        )?;
        fade_in_all(stage, &scale, 0.4)?;
        fade_in_all(stage, &[zero_line, area_word], 0.4)?;
        let needle = stage.always_redraw(move |s: &mut Arena| {
            let d = model::det(model::flow(x, value(s, time)));
            s.add(Dot::new().point(gauge(d)).radius(0.1).style(flat(blue, 1.0)).build())
        });
        stage.add_to_scene(needle)?;
        let det_val = kit.live_value(
            stage,
            move |s| model::det(model::flow(x, value(s, time))),
            3,
            false,
            24.0,
            blue,
            v(READOUT_VALUE_X, READOUT_VALUE_Y),
            LEFT,
        )?;
        kit.until_sentence(stage, "11_2", 1)?;
        let one = ring(stage, gauge(1.0), 0.16, blue, 2.5);
        fade_in_all(stage, &[one], 0.3)?;
        kit.until_sentence(stage, "11_2", 2)?;
        play!(stage; tween(time, 1.0, 1.6)?);
        let d1 = fact_f64("matrix.flow_det_t1")?;
        let t1 = kit.text_left(
            stage,
            &format!("t = 1: {}", fact("matrix.flow_det_t1")?),
            18.0,
            Palette::dim(),
            v(GAUGE_X + 0.3, gauge(d1)[1]),
        )?;
        fade_in_all(stage, &[t1], 0.3)?;
        play!(stage; tween(time, FLOW_TIMES.1, 2.0)?);
        let d3 = fact_f64("matrix.flow_det_t3")?;
        let t3 = kit.text_left(
            stage,
            &format!("t = 3: {}", fact("matrix.flow_det_t3")?),
            18.0,
            Palette::dim(),
            v(GAUGE_X + 0.3, gauge(d3)[1]),
        )?;
        fade_in_all(stage, &[t3], 0.3)?;
        kit.until_sentence(stage, "11_2", 3)?;
        play!(stage; tween(time, FLOW_TIMES.0, 3.0)?);
        let backwards = kit.text_left(
            stage,
            "run backwards: never flat, never zero",
            22.0,
            Palette::ink(),
            v(right, -1.4),
        )?;
        fade_in_all(stage, &[backwards], 0.4)?;
        kit.until_sentence(stage, "11_2", 4)?;
        let positive = seg(stage, v(GAUGE_X, GAUGE_ZERO + 0.06), gauge(GAUGE_RANGE.1), green, 6.0);
        let stays = kit.text_left(
            stage,
            "starts at 1, never touches 0: stays positive",
            22.0,
            green,
            v(right, -1.9),
        )?;
        fade_in_all(stage, &[positive, stays], 0.5)?;
        play!(stage; tween(time, 0.0, 1.2)?);
        kit.until_sentence(stage, "11_2", 5)?;
        let mirror_det = fact_f64("matrix.mirror_det")?;
        let mirror_mark = ring(stage, gauge(mirror_det), 0.16, red, 3.5);
        let mirror_word = kit.text_left(
            stage,
            &format!("mirror: {}", shown(fact("matrix.mirror_det")?)),
            20.0,
            red,
            v(GAUGE_X + 0.3, gauge(mirror_det)[1]),
        )?;
        let liouville = precisely(
            &kit,
            stage,
            "for any real linear flow, steady or not, det = e^(∫ trace dt) > 0 (Liouville)",
        )?;
        fade_in_all(stage, &[mirror_mark, mirror_word, liouville], 0.5)?;

        // 11_3: det < 0 rules a real logarithm out; det > 0 is not enough
        kit.say(stage, "11_3")?;
        let det_word = kit.text(
            stage,
            "det = signed area factor:",
            22.0,
            gold,
            v(GAUGE_X, READOUT_WORD_Y),
        )?;
        swap(stage, &[area_word, m_card, m_word], &[det_word], 0.4)?;
        kit.until_sentence(stage, "11_3", 1)?;
        let no_neg = kit.text_left(stage, "det < 0: no real logarithm", 24.0, red, v(right, 2.3))?;
        fade_in_all(stage, &[no_neg], 0.4)?;
        kit.until_sentence(stage, "11_3", 2)?;
        let mut gone = scale.clone();
        gone.extend([
            flowing,
            needle,
            det_val,
            det_word,
            zero_line,
            one,
            t1,
            t3,
            backwards,
            positive,
            stays,
            mirror_mark,
            mirror_word,
            liouville,
        ]);
        fade_out_all(stage, &gone, 0.5)?;
        let enough = kit.text_left(stage, "det > 0: enough?", 26.0, gold, v(right, 1.7))?;
        fade_in_all(stage, &[enough], 0.4)?;
        kit.until_sentence(stage, "11_3", 3)?;
        let grid = clipped_grid(stage.arena_mut(), I2, CENTRE, UNIT, 4, CLIP, green, 0.5);
        let before = stage.arena_mut().add(
            Polygon::new(letter_f(I2, CENTRE, UNIT))
                .style(flat(Palette::dim(), 0.65))
                .build(),
        );
        let a_word = kit.text_left(stage, "A =", 28.0, gold, v(right, 1.05))?;
        let (e00, e11) = (shown_num(pair[0]), shown_num(pair[1]));
        let a_card = matrix_card(
            &kit,
            stage,
            [[e00.as_str(), "0"], ["0", e11.as_str()]],
            v(right + 1.45, 1.05),
            30.0,
            gold,
        )?;
        fade_in_all(stage, &[grid, before, a_word, a_card], 0.5)?;
        kit.until_sentence(stage, "11_3", 4)?;
        let o = to_screen([0.0, 0.0]);
        let e1 = arrow_parts(stage.arena_mut(), o, to_screen([pair[0], 0.0]), green, 4.0, 0.2);
        let e1_word = kit.text(
            stage,
            &format!("×{}, flipped", shown_num(-pair[0])),
            20.0,
            green,
            v(to_screen([pair[0], 0.0])[0], CENTRE[1] + 0.35),
        )?;
        let e1_word = on_plate(stage, e1_word)?;
        fade_in_all(stage, &[e1, e1_word], 0.5)?;
        kit.until_sentence(stage, "11_3", 5)?;
        let e2 = arrow_parts(stage.arena_mut(), o, to_screen([0.0, pair[1]]), slide_c(), 4.0, 0.2);
        let e2_word = kit.text_left(
            stage,
            &format!("×{}, flipped", shown_num(-pair[1])),
            20.0,
            slide_c(),
            v(CENTRE[0] + 0.35, to_screen([0.0, pair[1]])[1] - 0.2),
        )?;
        let e2_word = on_plate(stage, e2_word)?;
        let after = stage.arena_mut().add(
            Polygon::new(letter_f(flip_pair, CENTRE, UNIT))
                .style(flat(gold, 0.9))
                .build(),
        );
        // keep the ×2 arrow over the gold F that lands on it: each add below re-sorts
        // the draw list by z index (fmn-mobject stage.rs at pin a399d626)
        stage.set_z_index(e1, 1, true);
        fade_in_all(stage, &[e2, e2_word, after], 0.5)?;
        kit.until_sentence(stage, "11_3", 6)?;
        let area = kit.text_left(
            stage,
            &format!(
                "det = ({}) × ({}) = +{}",
                shown_num(pair[0]),
                shown_num(pair[1]),
                fact("matrix.flip_pair_det")?
            ),
            24.0,
            Palette::ink(),
            v(right, 0.35),
        )?;
        fade_in_all(stage, &[area], 0.4)?;
        kit.until_sentence(stage, "11_3", 7)?;
        let split = kit.text_left(
            stage,
            &format!("A = (−I) × diag({}, {})", shown_num(-pair[0]), shown_num(-pair[1])),
            24.0,
            purple,
            v(right, -0.25),
        )?;
        let split_word = kit.text_left(stage, "a half turn, then a stretch", 20.0, purple, v(right, -0.8))?;
        fade_in_all(stage, &[split, split_word], 0.4)?;
        kit.until_sentence(stage, "11_3", 8)?;
        predict(&kit, stage, v(right + 1.6, -1.4), 3.0)?;

        // 11_4: no. Suppose A = e^X; then X commutes with A, because X
        // commutes with every power of itself
        kit.say(stage, "11_4")?;
        let no = kit.text_left(stage, "det > 0: not enough", 26.0, red, v(right, 1.7))?;
        swap(stage, &[enough], &[no], 0.3)?;
        kit.until_sentence(stage, "11_4", 1)?;
        let suppose = kit.tex(
            stage,
            r"\text{suppose } A = e^{X},\ X \text{ real}",
            28.0,
            blue,
            &[],
            v(4.6, 2.3),
        )?;
        swap(stage, &[no_neg, area, split, split_word], &[suppose], 0.4)?;
        kit.until_sentence(stage, "11_4", 2)?;
        // a sample X with its own exponential: both orders end on one point
        fade_out_all(stage, &[before, after, e1, e1_word, e2, e2_word], 0.4)?;
        let x_any = fact_matrix("matrix.flow")?;
        let e_any = model::flow(x_any, 1.0);
        let both = stage.always_redraw(move |s: &mut Arena| routes(s, x_any, e_any, u, true));
        stage.add_to_scene(both)?;
        let u_word = kit.text(
            stage,
            &format!("u = ({}, {})", shown_num(u[0]), shown_num(u[1])),
            20.0,
            Palette::ink(),
            to_screen([u[0] + 0.8, u[1] + 0.3]),
        )?;
        let u_word = on_plate(stage, u_word)?;
        let order_a = kit.text_left(stage, "first X, then e^X", 22.0, purple, v(right, 0.4))?;
        let order_b = kit.text_left(stage, "first e^X, then X", 22.0, slide_c(), v(right, 0.0))?;
        let same_end = kit.text_left(stage, "same end point", 22.0, gold, v(right + 3.2, 0.2))?;
        let sample = kit.text_left(
            stage,
            "(here X is the sample flow from before)",
            18.0,
            Palette::dim(),
            v(right, -0.35),
        )?;
        fade_in_all(stage, &[u_word, order_a, order_b, same_end, sample], 0.5)?;
        kit.until_sentence(stage, "11_4", 4)?;
        let series = kit.tex(
            stage,
            r"A = e^{X} = I + X + \tfrac{1}{2}X^{2} + \tfrac{1}{6}X^{3} + \text{...}",
            26.0,
            Palette::ink(),
            &[],
            v(4.6, -0.85),
        )?;
        fade_in_all(stage, &[series], 0.5)?;
        kit.until_sentence(stage, "11_4", 5)?;
        let powers = kit.tex(
            stage,
            r"X X^{k} = X^{k+1} = X^{k} X",
            26.0,
            Palette::ink(),
            &[],
            v(4.6, -1.5),
        )?;
        fade_in_all(stage, &[powers], 0.4)?;
        let so = kit.tex(stage, r"\Rightarrow\ XA = AX", 28.0, gold, &[], v(4.6, -2.15))?;
        fade_in_all(stage, &[so], 0.4)?;

        // 11_5: what commuting with A costs: the routes A-then-X and X-then-A of u
        kit.say(stage, "11_5")?;
        fade_out_all(stage, &[both, order_a, order_b, same_end, sample, series, powers], 0.4)?;
        let given = kit.tex(stage, r"A = e^{X}\ \Rightarrow\ XA = AX", 28.0, gold, &[], v(4.6, 2.3))?;
        swap(stage, &[suppose, so], &[given], 0.4)?;
        let x_word = kit.text_left(stage, "X =", 28.0, blue, v(right + 2.7, 1.05))?;
        let x_card = matrix_card(&kit, stage, [["p", "b"], ["c", "q"]], v(right + 3.95, 1.05), 30.0, blue)?;
        fade_in_all(stage, &[x_word, x_card], 0.4)?;
        kit.until_sentence(stage, "11_5", 1)?;
        let off = stage.add_value_tracker(1.0);
        // the bar waits for the words that claim it
        let gap_on = stage.add_value_tracker(0.0);
        let test = stage.always_redraw(move |s: &mut Arena| {
            let xs = candidate(base, value(s, off));
            routes(s, flip_pair, xs, u, value(s, gap_on) > 0.5)
        });
        stage.add_to_scene(test)?;
        let route_a = kit.text_left(stage, "first A, then X", 22.0, purple, v(right, 0.4))?;
        let route_b = kit.text_left(stage, "first X, then A", 22.0, slide_c(), v(right, 0.0))?;
        fade_in_all(stage, &[route_a, route_b], 0.4)?;
        kit.until_sentence(stage, "11_5", 2)?;
        until_phrase(&kit, stage, "11_5", ENDS_DIFFER)?;
        let gap_word = kit.text_left(stage, "gap between the ends:", 22.0, red, v(right, -0.55))?;
        let gap_val = kit.live_value(
            stage,
            move |s| {
                let g = model::apply(model::commutator(candidate(base, value(s, off)), flip_pair), u);
                g[0].hypot(g[1])
            },
            2,
            false,
            22.0,
            red,
            v(right + 2.75, -0.55),
            LEFT,
        )?;
        stage.set_tracker_value(gap_on, 1.0)?;
        fade_in_all(stage, &[gap_word], 0.4)?;
        kit.until_sentence(stage, "11_5", 3)?;
        let gap_factor = fact("matrix.flip_pair_gap")?;
        let why_a = kit.text_left(
            stage,
            &format!("each gap part = ±{gap_factor} × an off-diagonal"),
            22.0,
            Palette::ink(),
            v(right, -1.1),
        )?;
        let why_b = kit.text_left(stage, "entry of X × a part of u,", 22.0, Palette::ink(), v(right, -1.5))?;
        let why_c = kit.text_left(
            stage,
            &format!(
                "where {gap_factor} = ({}) − ({})",
                shown_num(pair[1]),
                shown_num(pair[0])
            ),
            22.0,
            Palette::ink(),
            v(right, -1.9),
        )?;
        fade_in_all(stage, &[why_a, why_b, why_c], 0.5)?;
        kit.until_sentence(stage, "11_5", 4)?;
        play!(stage; tween(off, 0.0, 2.6)?);
        let only_diag = kit.text_left(stage, "the ends meet only when b = c = 0", 24.0, gold, v(right, -2.45))?;
        fade_in_all(stage, &[only_diag], 0.4)?;

        // 11_6: so X is diagonal, and a real exponential never reverses an axis
        kit.say(stage, "11_6")?;
        let x_diag = matrix_card(&kit, stage, [["p", "0"], ["0", "q"]], v(right + 3.95, 1.05), 30.0, blue)?;
        swap(stage, &[x_card], &[x_diag], 0.4)?;
        fade_out_all(
            stage,
            &[
                test, u_word, route_a, route_b, gap_word, gap_val, why_a, why_b, why_c, only_diag,
            ],
            0.4,
        )?;
        let reach = UNIT * CLIP;
        let ax = seg(
            stage,
            v(CENTRE[0] - reach, CENTRE[1]),
            v(CENTRE[0] + reach, CENTRE[1]),
            gold,
            5.0,
        );
        let ay = seg(
            stage,
            v(CENTRE[0], CENTRE[1] - reach),
            v(CENTRE[0], CENTRE[1] + reach),
            gold,
            5.0,
        );
        fade_in_all(stage, &[ax, ay], 0.4)?;
        kit.until_sentence(stage, "11_6", 1)?;
        fade_out_all(stage, &[grid], 0.2)?;
        let diag = stage.add_value_tracker(0.0);
        let stretching =
            stage.always_redraw(move |s: &mut Arena| plane(s, diagonal_flow(la, lb, value(s, diag)), true));
        stage.add_to_scene(stretching)?;
        let axes_word = kit.text_left(
            stage,
            "e^(tX) stretches the axes by e^(tp) and e^(tq)",
            22.0,
            blue,
            v(right, 0.4),
        )?;
        fade_in_all(stage, &[axes_word], 0.4)?;
        play!(stage; tween(diag, 1.0, 2.0)?);
        kit.until_sentence(stage, "11_6", 2)?;
        // where A puts the F: a half turn away from anything this flow reaches
        let mut target_pts = letter_f(flip_pair, CENTRE, UNIT);
        target_pts.push(target_pts[0]);
        let target = stage
            .arena_mut()
            .add(polyline(target_pts).style(faded_line(red, 3.0, 0.9)).build());
        // its words stand in the column: on the panel they sit on grid lines and under the moving plane
        let mut key_pts = letter_f(flip_pair, v(right + KEY_F_X, KEY_Y), KEY_F_UNIT);
        key_pts.push(key_pts[0]);
        let target_key = stage
            .arena_mut()
            .add(polyline(key_pts).style(faded_line(red, 2.0, 0.9)).build());
        let target_word = kit.text_left(stage, "where A puts the F", 20.0, red, v(right + KEY_WORD_X, KEY_Y))?;
        let positive = kit.text_left(
            stage,
            "e^(tp) > 0 for all real t and p: no axis reverses",
            22.0,
            blue,
            v(right, -0.05),
        )?;
        fade_in_all(stage, &[target, target_key, target_word, positive], 0.4)?;
        play!(stage; tween(diag, -1.0, 2.0)?);
        kit.until_sentence(stage, "11_6", 3)?;
        fade_out_all(stage, &[stretching], 0.3)?;
        let journey = stage.add_value_tracker(0.0);
        let two = stage.always_redraw(move |s: &mut Arena| plane(s, two_stage(value(s, journey), la, lb), true));
        stage.add_to_scene(two)?;
        let stage_word = kit.text_left(
            stage,
            "a half turn, then a stretch: two flows in turn",
            22.0,
            purple,
            v(right, -0.85),
        )?;
        fade_in_all(stage, &[stage_word], 0.3)?;
        play!(stage; tween(journey, 1.0, 2.0)?);
        play!(stage; tween(journey, 2.0, 1.8)?);
        kit.until_sentence(stage, "11_6", 4)?;
        let never_one = kit.text_left(stage, "no single steady flow does both", 26.0, red, v(right, -1.45))?;
        fade_in_all(stage, &[never_one], 0.4)?;

        // 11_7: flipping both axes alike is a half turn; why the argument fails there; the exact rule
        kit.say(stage, "11_7")?;
        let away = [
            two,
            stage_word,
            never_one,
            positive,
            axes_word,
            target,
            target_key,
            target_word,
            ax,
            ay,
            a_word,
            a_card,
            x_word,
            x_diag,
            no,
            given,
        ];
        fade_out_all(stage, &away, 0.5)?;
        let alike = kit.text_left(stage, "both axes flipped alike:", 24.0, green, v(right, 2.3))?;
        let minus_word = kit.text_left(stage, "−I =", 28.0, green, v(right, 1.6))?;
        let minus_i = matrix_card(
            &kit,
            stage,
            [["-1", "0"], ["0", "-1"]],
            v(right + 1.45, 1.6),
            30.0,
            green,
        )?;
        fade_in_all(stage, &[alike, minus_word, minus_i], 0.4)?;
        kit.until_sentence(stage, "11_7", 1)?;
        let turn = stage.add_value_tracker(0.0);
        let speed = fact_f64("matrix.half_turn_speed")?;
        let spinning = stage.always_redraw(move |s: &mut Arena| {
            plane(s, model::flow(model::mscale(J, 2.0 * PI * speed), value(s, turn)), true)
        });
        stage.add_to_scene(spinning)?;
        play!(stage; tween(turn, 1.0, 2.4)?);
        let spin = kit.tex(stage, r"e^{\pi J} = -I", 30.0, green, &[], v(right + 1.2, 0.75))?;
        let spin_word = kit.text_left(
            stage,
            &format!("a steady spin, {} lap", fact("matrix.half_turn_speed")?),
            20.0,
            green,
            v(right + 2.6, 0.75),
        )?;
        fade_in_all(stage, &[spin, spin_word], 0.4)?;
        kit.until_sentence(stage, "11_7", 3)?;
        let equal_a = kit.text_left(
            stage,
            "equal stretches: (−1) − (−1) = 0, no gap,",
            20.0,
            gold,
            v(right, 0.1),
        )?;
        let equal_b = kit.text_left(stage, "so every X commutes with −I", 20.0, gold, v(right, -0.3))?;
        fade_in_all(stage, &[equal_a, equal_b], 0.4)?;
        // back to the start with the axes marked, then the same half lap with them
        // turning. No further lap: the caption says ½, and 12_5 asks about the rest
        until_phrase(&kit, stage, "11_7", KEEP_AXES)?;
        let axes_on = stage.add_value_tracker(0.0);
        let turning = stage.always_redraw(move |s: &mut Arena| {
            let m = model::flow(model::mscale(J, 2.0 * PI * speed), value(s, turn));
            turning_axes(s, m, value(s, axes_on))
        });
        stage.add_to_scene(turning)?;
        let now = stage.scene().time().to_f64();
        let by = phrase_time(&kit, "11_7", TURNS_EVERY)?;
        tween_all(stage, &[(turn, 0.0), (axes_on, 1.0)], run_by(by, now, 0.8, 1.8))?;
        until_phrase(&kit, stage, "11_7", TURNS_EVERY)?;
        let now = stage.scene().time().to_f64();
        let by = sentence_time(&kit, "11_7", 4).map(|at| at - 0.2);
        play!(stage; tween(turn, 1.0, run_by(by, now, 1.0, 2.4))?);
        kit.until_sentence(stage, "11_7", 4)?;
        let still_grid = clipped_grid(stage.arena_mut(), I2, CENTRE, UNIT, 4, CLIP, green, 0.3);
        let rim = ring(
            stage,
            to_screen([0.0, 0.0]),
            DIRECTION_LEN * UNIT,
            Palette::faint(),
            2.0,
        );
        let two_a = kit.text_left(
            stage,
            "2×2, a direction sent to a negative multiple:",
            20.0,
            Palette::ink(),
            v(right, -0.8),
        )?;
        let two_b = kit.text_left(
            stage,
            "a real log only if A = −cI, c > 0",
            20.0,
            Palette::ink(),
            v(right, -1.2),
        )?;
        swap(stage, &[spinning, turning], &[still_grid, rim, two_a, two_b], 0.4)?;
        // some directions only: half a lap with A itself, marks on the axes alone
        until_phrase(&kit, stage, "11_7", SOME_DIRECTION)?;
        let angle = stage.add_value_tracker(0.0);
        let to_equal = stage.add_value_tracker(0.0);
        let equal_c = stage.add_value_tracker(1.0);
        let sweeping = stage.always_redraw(move |s: &mut Arena| {
            sweep(s, flip_pair, value(s, angle), value(s, to_equal), value(s, equal_c))
        });
        stage.add_to_scene(sweeping)?;
        let legend = [
            kit.text_left(stage, "white: a direction", 18.0, Palette::ink(), v(GAP_X, 1.7))?,
            kit.text_left(stage, "gold: where A sends it", 18.0, gold, v(GAP_X, 1.35))?,
            kit.text_left(stage, "dashes: straight back", 18.0, Palette::dim(), v(GAP_X, 1.0))?,
        ];
        let some_a = kit.text_left(
            stage,
            &format!("A = diag({}, {})", shown_num(pair[0]), shown_num(pair[1])),
            22.0,
            gold,
            v(GAP_X, 0.3),
        )?;
        let some_b = kit.text_left(
            stage,
            "only the two axes go straight back",
            20.0,
            purple,
            v(GAP_X, -0.1),
        )?;
        let mut words = legend.to_vec();
        words.extend([some_a, some_b]);
        fade_in_all(stage, &words, 0.4)?;
        let now = stage.scene().time().to_f64();
        let by = phrase_time(&kit, "11_7", EVERY_DIRECTION)?;
        play!(stage; steady(angle, PI, run_by(by, now, 2.0, 6.0))?);
        // every direction: −I, a whole lap of marks
        until_phrase(&kit, stage, "11_7", EVERY_DIRECTION)?;
        tween_all(stage, &[(to_equal, 1.0)], 0.3)?;
        let all_a = kit.text_left(stage, "A = −I", 22.0, gold, v(GAP_X, 0.3))?;
        let all_b = kit.text_left(
            stage,
            "every direction goes straight back",
            20.0,
            purple,
            v(GAP_X, -0.1),
        )?;
        swap(stage, &[some_a, some_b], &[all_a, all_b], 0.3)?;
        let now = stage.scene().time().to_f64();
        let by = phrase_time(&kit, "11_7", HALF_TURN_STRETCH)?;
        play!(stage; steady(angle, 3.0 * PI, run_by(by, now, 1.5, 3.0))?);
        // the equal stretch: the gold grows along the dashes, still straight back
        until_phrase(&kit, stage, "11_7", HALF_TURN_STRETCH)?;
        let any_a = kit.text_left(stage, "A = −cI, c > 0", 22.0, gold, v(GAP_X, 0.3))?;
        let any_b = kit.text_left(stage, "half turn, equal stretch", 20.0, purple, v(GAP_X, -0.1))?;
        swap(stage, &[all_a, all_b], &[any_a, any_b], 0.3)?;
        let now = stage.scene().time().to_f64();
        let by = sentence_time(&kit, "11_7", 5).map(|at| at - 0.2);
        play!(stage; tween(equal_c, STRETCH_C, run_by(by, now, 0.8, 1.6))?);
        kit.until_sentence(stage, "11_7", 5)?;
        let cite = precisely(
            &kit,
            stage,
            "Culver, Proc. AMS 17 (1966) 1146–1151, as stated by Higham (Handbook of Linear Algebra) and Gallier (Thm 3.4)",
        )?;
        // higher dimensions: a matrix as its blocks, one row per matrix
        let blocks_head = kit.text_left(
            stage,
            "each row is one matrix, each tile a block and its stretch",
            20.0,
            Palette::dim(),
            v(ROWS_X - TILE_HALF, BLOCKS_HEAD_Y),
        )?;
        let mut sweep_gone = vec![sweeping, still_grid, rim, any_a, any_b];
        sweep_gone.extend(legend);
        swap(stage, &sweep_gone, &[cite, blocks_head], 0.4)?;
        let rows = block_rows(&kit, stage)?;
        let now = stage.scene().time().to_f64();
        let by = phrase_time(&kit, "11_7", BLOCKS)?;
        fade_in_lagged(stage, &rows.tiles, run_by(by, now, 1.2, 4.0), 0.5)?;
        until_phrase(&kit, stage, "11_7", BLOCKS)?;
        let now = stage.scene().time().to_f64();
        let by = phrase_time(&kit, "11_7", MATCHING)?;
        fade_in_lagged(stage, &rows.verdicts, run_by(by, now, 0.8, 2.4), 0.4)?;
        until_phrase(&kit, stage, "11_7", MATCHING)?;
        fade_in_all(stage, &rows.brackets, 0.4)?;
        kit.until_sentence(stage, "11_7", 6)?;
        let rule_a = kit.text_left(
            stage,
            "A real matrix has a real logarithm exactly",
            20.0,
            gold,
            v(right, -1.75),
        )?;
        let rule_b = kit.text_left(
            stage,
            "when it is invertible and, for every negative",
            20.0,
            gold,
            v(right, -2.15),
        )?;
        let rule_c = kit.text_left(
            stage,
            "eigenvalue, its Jordan blocks of each size",
            20.0,
            gold,
            v(right, -2.55),
        )?;
        let rule_d = kit.text_left(stage, "occur an even number of times.", 20.0, gold, v(right, -2.95))?;
        fade_in_all(stage, &[rule_a, rule_b, rule_c, rule_d], 0.6)?;

        // 11_8: complex logarithms; only flattening matrices have none
        kit.say(stage, "11_8")?;
        let mut rule_gone = vec![
            alike,
            minus_word,
            minus_i,
            spin,
            spin_word,
            equal_a,
            equal_b,
            two_a,
            two_b,
            rule_a,
            rule_b,
            rule_c,
            rule_d,
            cite,
            blocks_head,
        ];
        rule_gone.extend(rows.tiles);
        rule_gone.extend(rows.verdicts);
        rule_gone.extend(rows.brackets);
        fade_out_all(stage, &rule_gone, 0.5)?;
        let heading2 = title(&kit, stage, "Allow complex entries")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        kit.until_sentence(stage, "11_8", 1)?;
        let flip = kit.tex(stage, r"e^{i\pi} = -1", 34.0, purple, &[], v(0.0, 2.2))?;
        fade_in_all(stage, &[flip], 0.4)?;
        let big = format!("{}", -pair[0]);
        let complex_log = kit.tex(
            stage,
            &format!(r"\log A = \operatorname{{diag}}(\ln {big} + i\pi,\ -\ln {big} + i\pi)"),
            32.0,
            blue,
            &[],
            v(0.0, 1.3),
        )?;
        let check = kit.tex(
            stage,
            &format!(r"e^{{\ln {big} + i\pi}} = -{big}, \qquad e^{{-\ln {big} + i\pi}} = -\tfrac{{1}}{{{big}}}"),
            28.0,
            Palette::ink(),
            &[],
            v(0.0, 0.45),
        )?;
        let logs = fact_list("matrix.flip_pair_log_real")?;
        let approx = kit.text(
            stage,
            &format!("ln {big} ≈ {}", logs[0]),
            20.0,
            Palette::dim(),
            v(0.0, -0.1),
        )?;
        fade_in_all(stage, &[complex_log, check, approx], 0.6)?;
        kit.until_sentence(stage, "11_8", 2)?;
        let every = kit.text(
            stage,
            "every invertible complex matrix has a complex logarithm",
            26.0,
            gold,
            v(0.0, -0.9),
        )?;
        let src = precisely(
            &kit,
            stage,
            "Gallier, arXiv 0805.0245, Theorem 3.3, proved via the Jordan form",
        )?;
        fade_in_all(stage, &[every, src], 0.5)?;
        kit.until_sentence(stage, "11_8", 3)?;
        let singular = kit.tex(
            stage,
            r"\operatorname{det} A = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0",
            28.0,
            red,
            &[],
            v(0.0, -1.85),
        )?;
        let like_zero = kit.text(
            stage,
            "flattening matrices, like zero, have no logarithm",
            22.0,
            red,
            v(0.0, -2.5),
        )?;
        fade_in_all(stage, &[singular, like_zero], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOW: M2 = [[0.3, -1.0], [0.8, -0.1]];
    const PAIR: (f64, f64) = (-2.0, -0.5);

    /// CMR10 advance widths in em; capitals and ½ take a typical capital's.
    fn em(c: char) -> f64 {
        match c {
            'i' | 'l' | ',' | '.' | ':' | ';' | '!' | '[' | ']' => 0.278,
            'j' | 'f' => 0.306,
            ' ' | '-' => 0.333,
            'I' => 0.361,
            'r' | 's' | 't' | '(' | ')' => 0.39,
            'a' | 'c' | 'e' | 'z' => 0.444,
            'g' | 'o' | '^' | '/' | '0'..='9' => 0.5,
            'k' | 'q' | 'v' | 'x' | 'y' => 0.528,
            'b' | 'd' | 'h' | 'n' | 'p' | 'u' => 0.556,
            'w' => 0.722,
            '=' | '+' | '−' | '×' | '±' | '<' | '>' => 0.778,
            'm' => 0.833,
            _ => 0.75,
        }
    }

    /// Text width in scene units: ems × size × 0.01. Within 2% of the five long
    /// lines the layout lint measured on preview01, within 4% of nine shorter
    /// ones read from its frames. The flat estimate this replaces, 0.55 × size /
    /// 144 per character, ran about 20% short and passed lines clipped on screen.
    fn width(s: &str, size: f64) -> f64 {
        s.chars().map(em).sum::<f64>() * size * 0.01
    }

    /// The lint's `tight` limit for a text's right edge, and room for the
    /// width estimate's error.
    const TEXT_RIGHT: f64 = 7.11 - 0.25;
    const WIDTH_ROOM: f64 = 1.03;

    const MIRROR: M2 = [[-1.0, 0.0], [0.0, 1.0]];
    const PROBE: [f64; 2] = [1.0, 1.0];
    const BASE: M2 = [[0.4, 0.6], [-0.5, -0.3]];

    fn on_panel(q: [f64; 2]) -> bool {
        q[0].abs() < CLIP && q[1].abs() < CLIP
    }

    #[test]
    fn the_plain_average_of_identity_and_mirror_is_flat() {
        let m = lerp(I2, MIRROR, 0.5);
        assert!(model::det(m).abs() < 1e-12);
        assert_eq!(fact("matrix.mirror_average_det").unwrap(), "0");
        // every point lands on the vertical axis: the squashed F is a segment
        for p in letter_f(m, CENTRE, UNIT) {
            assert!((p[0] - CENTRE[0]).abs() < 1e-12);
        }
        // and the sign really changes along the straight line between them
        assert!(model::det(lerp(I2, MIRROR, 0.49)) > 0.0);
        assert!(model::det(lerp(I2, MIRROR, 0.51)) < 0.0);
    }

    #[test]
    fn the_comparison_dashes_join_each_corner_to_its_mirror_image() {
        let (from, to) = (letter_f(I2, CENTRE, UNIT), letter_f(MIRROR, CENTRE, UNIT));
        for k in CHORD_CORNERS {
            // the mirror keeps height and reflects across the vertical axis
            assert!((from[k][1] - to[k][1]).abs() < 1e-12);
            assert!((from[k][0] - CENTRE[0] + to[k][0] - CENTRE[0]).abs() < 1e-12);
        }
    }

    #[test]
    fn the_probe_and_candidate_match_the_facts() {
        assert_eq!(fact_list("matrix.probe").unwrap(), PROBE.to_vec());
        assert!(model::mdist(fact_matrix("matrix.probe_candidate").unwrap(), BASE) < 1e-15);
        assert_eq!(fact_matrix("matrix.flow").unwrap(), FLOW);
    }

    #[test]
    fn x_and_its_own_exponential_end_both_routes_together() {
        let e = model::flow(FLOW, 1.0);
        let [(_, end_a), (_, end_b)] = route_points(FLOW, e, PROBE);
        assert!((end_a[0] - end_b[0]).hypot(end_a[1] - end_b[1]) < 1e-12);
        // term by term: X X^k = X^k X for the powers the series is built from
        let mut power = I2;
        for _ in 0..12 {
            assert!(model::mdist(model::mmul(FLOW, power), model::mmul(power, FLOW)) < 1e-12);
            power = model::mmul(power, FLOW);
        }
        // the routes' points stay on the plane panel
        for (mid, end) in route_points(FLOW, e, PROBE) {
            assert!(on_panel(mid) && on_panel(end) && on_panel(PROBE));
        }
    }

    #[test]
    fn each_part_of_the_gap_is_an_off_diagonal_entry_times_the_stretch_difference() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        for k in [1.0, 0.5, 0.25, 1e-6] {
            let x = candidate(BASE, k);
            let [(_, end_a), (_, end_b)] = route_points(a, x, PROBE);
            let gap = [end_a[0] - end_b[0], end_a[1] - end_b[1]];
            // route one is X(Au), route two A(Xu): the gap is (XA − AX)u
            let c = model::apply(model::commutator(x, a), PROBE);
            assert!((gap[0] - c[0]).abs() < 1e-12 && (gap[1] - c[1]).abs() < 1e-12);
            assert!((gap[0] - x[0][1] * (PAIR.1 - PAIR.0) * PROBE[1]).abs() < 1e-12);
            assert!((gap[1] - x[1][0] * (PAIR.0 - PAIR.1) * PROBE[0]).abs() < 1e-12);
            // the edge: however small the off-diagonal entries, the ends differ
            assert!(gap[0].abs() > 1e-9 && gap[1].abs() > 1e-9, "{k}");
        }
        assert_eq!(fact("matrix.flip_pair_gap").unwrap(), "1.5");
        let full = model::apply(model::commutator(BASE, a), PROBE);
        assert_eq!(
            format!("{:.2}", full[0].hypot(full[1])),
            fact("matrix.probe_gap").unwrap()
        );
    }

    #[test]
    fn the_ends_meet_once_the_off_diagonal_entries_are_zero() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        let [(_, end_a), (_, end_b)] = route_points(a, candidate(BASE, 0.0), PROBE);
        assert!((end_a[0] - end_b[0]).hypot(end_a[1] - end_b[1]) < 1e-15);
        // throughout the tween every point of both routes stays on the panel
        for j in 0..=20 {
            let x = candidate(BASE, f64::from(j) / 20.0);
            for (mid, end) in route_points(a, x, PROBE) {
                assert!(on_panel(mid) && on_panel(end), "{j}");
            }
        }
    }

    #[test]
    fn signed_area_along_a_flow_is_e_to_trace_times_t_at_the_shown_times() {
        let tr = model::trace(FLOW);
        for k in -40..=60 {
            let t = f64::from(k) / 20.0;
            let d = model::det(model::flow(FLOW, t));
            assert!(d > 0.0 && (d - (tr * t).exp()).abs() < 1e-9, "{t}");
        }
        // at the ends of the clock's range, and the gauge holds them
        for t in [FLOW_TIMES.0, FLOW_TIMES.1] {
            let d = model::det(model::flow(FLOW, t));
            assert!(d > 0.0 && d > GAUGE_RANGE.0 && d < GAUGE_RANGE.1, "{t}: {d}");
        }
    }

    #[test]
    fn no_flow_reaches_negative_area_even_with_a_strongly_shrinking_generator() {
        // the edge: a trace far below zero shrinks area hard, never past zero
        for x in [
            [[-6.0, 3.0], [-2.0, -5.0]],
            [[0.0, 9.0], [-9.0, -0.4]],
            [[-0.01, 0.0], [0.0, -20.0]],
        ] {
            for t in [0.5, 2.0, 5.0] {
                assert!(model::det(model::flow(x, t)) > 0.0);
            }
        }
    }

    #[test]
    fn anything_commuting_with_the_flip_pair_is_diagonal() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        // (XA − AX) has off-diagonal entries x01 (a11 − a00) and x10 (a00 − a11)
        for x in [
            [[0.3, 0.7], [-0.2, 0.1]],
            [[1.0, 1e-3], [0.0, 2.0]],
            [[0.0, 0.0], [1e-3, 0.0]],
        ] {
            let c = model::commutator(x, a);
            let off = x[0][1].abs() + x[1][0].abs();
            assert!((c[0][1].abs() + c[1][0].abs() - off * (PAIR.0 - PAIR.1).abs()).abs() < 1e-12);
            assert!(model::mdist(c, [[0.0; 2]; 2]) > 0.0);
        }
        assert!(model::mdist(model::commutator([[0.4, 0.0], [0.0, -3.0]], a), [[0.0; 2]; 2]) < 1e-15);
    }

    #[test]
    fn a_diagonal_flow_never_flips_an_axis() {
        for (a, b) in [(0.4, -1.0), (-3.0, 2.0), (2f64.ln(), -(2f64.ln()))] {
            for t in [-1.0, 0.5, 1.0, 4.0] {
                let e = diagonal_flow(a, b, t);
                assert!(e[0][0] > 0.0 && e[1][1] > 0.0);
            }
        }
    }

    #[test]
    fn two_flows_in_turn_reach_the_flip_pair_continuously() {
        let (la, lb) = (PAIR.0.abs().ln(), PAIR.1.abs().ln());
        let end = two_stage(2.0, la, lb);
        assert!(model::mdist(end, [[PAIR.0, 0.0], [0.0, PAIR.1]]) < 1e-12);
        assert!(model::mdist(two_stage(1.0, la, lb), two_stage(1.0 + 1e-9, la, lb)) < 1e-6);
        // the pair's determinant is +1, so −ln|a| = ln|b| (used on screen)
        assert!((PAIR.0 * PAIR.1 - 1.0).abs() < 1e-15 && (la + lb).abs() < 1e-15);
    }

    #[test]
    fn the_half_turn_spin_reaches_minus_identity() {
        let e = model::flow(model::mscale(J, 2.0 * PI * 0.5), 1.0);
        assert!(model::mdist(e, model::mscale(I2, -1.0)) < 1e-12);
    }

    #[test]
    fn the_complex_logarithm_of_the_flip_pair_exponentiates_back() {
        let (la, lb) = (PAIR.0.abs().ln(), PAIR.1.abs().ln());
        for (l, target) in [(la, PAIR.0), (lb, PAIR.1)] {
            let e = model::C::new(l, PI).exp();
            assert!((e.re - target).abs() < 1e-12 && e.im.abs() < 1e-12);
        }
    }

    #[test]
    fn the_panels_and_text_column_fit() {
        assert!(gauge(GAUGE_RANGE.0)[1] > -3.2 && gauge(GAUGE_RANGE.1)[1] < 2.3);
        const { assert!(CENTRE[0] + UNIT * CLIP < GAUGE_X - 0.6) };
        for (s, size, x) in [
            ("dashes pair each corner with where it lands", 20.0, 2.2),
            ("the average ½(I + mirror) has det 0:", 22.0, 2.2),
            ("it squashes the plane flat onto a line", 22.0, 2.2),
            ("starts at 1, never touches 0: stays positive", 22.0, 2.2),
            ("(here X is the sample flow from before)", 18.0, 2.2),
            ("det < 0: no real logarithm", 24.0, 2.2),
            ("each gap part = ±1.5 × an off-diagonal", 22.0, 2.2),
            ("entry of X × a part of u,", 22.0, 2.2),
            ("where 1.5 = (−½) − (−2)", 22.0, 2.2),
            ("the ends meet only when b = c = 0", 24.0, 2.2),
            ("e^(tX) stretches the axes by e^(tp) and e^(tq)", 22.0, 2.2),
            ("e^(tp) > 0 for all real t and p: no axis reverses", 22.0, 2.2),
            ("where A puts the F", 20.0, 2.2 + KEY_WORD_X),
            ("a half turn, then a stretch: two flows in turn", 22.0, 2.2),
            ("equal stretches: (−1) − (−1) = 0, no gap,", 20.0, 2.2),
            ("so every X commutes with −I", 20.0, 2.2),
            ("2×2, a direction sent to a negative multiple:", 20.0, 2.2),
            ("a real log only if A = −cI, c > 0", 20.0, 2.2),
            ("A real matrix has a real logarithm exactly", 20.0, 2.2),
            ("when it is invertible and, for every negative", 20.0, 2.2),
            ("eigenvalue, its Jordan blocks of each size", 20.0, 2.2),
            ("occur an even number of times.", 20.0, 2.2),
        ] {
            assert!(x + width(s, size) * WIDTH_ROOM < TEXT_RIGHT, "{s}");
        }
        // the X card clears the A card and the screen edge (card half-width 0.0205 × size)
        for (offset, boundary) in [(1.45, 2.2 + 2.7), (3.95, 7.0)] {
            assert!(2.2 + offset + 0.0205 * 30.0 < boundary);
        }
        // the "u = (1, 1)" label sits on the plane panel
        let label = to_screen([PROBE[0] + 0.8, PROBE[1] + 0.3]);
        assert!(label[0] + width("u = (1, 1)", 20.0) / 2.0 < CENTRE[0] + UNIT * CLIP + 0.1);
        assert!(width("every invertible complex matrix has a complex logarithm", 26.0) < 13.6);
        for line in [
            "Culver, Proc. AMS 17 (1966) 1146–1151, as stated by Higham (Handbook of Linear Algebra) and Gallier (Thm 3.4)",
            "Gallier, arXiv 0805.0245, Theorem 3.3, proved via the Jordan form",
        ] {
            assert!(width(&format!("precisely: {line}"), 22.0) < 13.6, "{line}");
        }
    }

    #[test]
    fn the_readout_keeps_its_word_and_number_apart_and_clear_of_the_gauge() {
        // text is about 0.2 high: word over number, under the title (its foot near y 3.28)
        const { assert!(READOUT_WORD_Y - READOUT_VALUE_Y > 0.3) };
        const { assert!(READOUT_WORD_Y + 0.12 < 3.2) };
        assert!(READOUT_VALUE_Y - 0.12 > gauge(GAUGE_RANGE.1)[1]);
        // the number, about 0.55 wide, is centred on the gauge
        const { assert!(READOUT_VALUE_X + 0.275 > GAUGE_X - 0.02 && READOUT_VALUE_X + 0.275 < GAUGE_X + 0.02) };
        // and neither word reaches the text column
        for word in ["signed area of the square:", "det = signed area factor:"] {
            assert!(GAUGE_X + width(word, 22.0) * WIDTH_ROOM / 2.0 < 2.2, "{word}");
        }
    }

    #[test]
    fn the_zero_line_leaves_the_tick_numbers_alone() {
        for tick in ["−1", "0", "1", "2"] {
            assert!(
                GAUGE_LABEL_X + width(tick, 18.0) * WIDTH_ROOM / 2.0 < ZERO_LINE_LEFT - 0.05,
                "{tick}"
            );
        }
        const { assert!(CENTRE[0] + UNIT * CLIP < GAUGE_LABEL_X - 0.3) };
    }

    #[test]
    fn the_target_key_stands_in_the_column_between_its_neighbours() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        for p in letter_f(a, v(2.2 + KEY_F_X, KEY_Y), KEY_F_UNIT) {
            assert!(p[0] > 2.2 - 1e-9 && p[0] < 2.2 + KEY_WORD_X - 0.1, "{}", p[0]);
            // the caption lines above (y −0.05) and below (y −0.85), each about 0.2 high
            assert!(p[1] < -0.05 - 0.12 && p[1] > -0.85 + 0.12, "{}", p[1]);
        }
    }

    #[test]
    fn a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence() {
        let text = "One two. Three four five six. Seven.";
        let starts = [0.2, 2.0, 6.0];
        let none = HashMap::new();
        // eleven of the sentence's twenty characters come before "five"
        let t = phrase_seconds(text, "five", &none, &starts, 8.0).unwrap();
        assert!((t - (2.0 + 4.0 * 11.0 / 20.0)).abs() < 1e-12);
        // the first word of a sentence is that sentence's measured start
        let t = phrase_seconds(text, "Seven", &none, &starts, 8.0).unwrap();
        assert!((t - 6.0).abs() < 1e-12);
        // the line's length closes its last sentence: four of nine characters in
        let t = phrase_seconds("One. Two three", "three", &none, &[0.0, 1.0], 10.0).unwrap();
        assert!((t - (1.0 + 9.0 * 4.0 / 9.0)).abs() < 1e-12);
        // a measured word mark wins, and a mark on another word changes nothing
        let marks = HashMap::from([(text.find("five").unwrap(), 3.25)]);
        let t = phrase_seconds(text, "five", &marks, &starts, 8.0).unwrap();
        assert!((t - 3.25).abs() < 1e-12);
        let t = phrase_seconds(text, "six", &marks, &starts, 8.0).unwrap();
        assert!((t - (2.0 + 4.0 * 16.0 / 20.0)).abs() < 1e-12);
        // never a guess
        assert!(phrase_seconds(text, "nine", &none, &starts, 8.0).is_err());
        assert!(phrase_seconds(text, "five", &none, &starts[..2], 8.0).is_err());
    }

    #[test]
    fn every_phrase_hook_is_in_its_line_once_and_they_come_in_spoken_order() {
        let script = Script::parse(SCRIPT).unwrap();
        for (id, phrases) in [
            ("11_1", &[HALFWAY, SQUASHES][..]),
            ("11_5", &[ENDS_DIFFER][..]),
            (
                "11_7",
                &[
                    KEEP_AXES,
                    TURNS_EVERY,
                    SOME_DIRECTION,
                    EVERY_DIRECTION,
                    HALF_TURN_STRETCH,
                    BLOCKS,
                    MATCHING,
                ][..],
            ),
        ] {
            let text = script.text(id).unwrap();
            let mut last = 0;
            for &phrase in phrases {
                assert_eq!(text.matches(phrase).count(), 1, "{phrase}");
                let at = text.find(phrase).unwrap();
                assert!(at >= last, "{phrase}");
                last = at;
            }
        }
    }

    #[test]
    fn a_run_ends_by_its_moment_within_its_limits() {
        assert!((run_by(Some(12.0), 10.0, 1.0, 3.0) - 2.0).abs() < 1e-12);
        assert!((run_by(Some(20.0), 10.0, 1.0, 3.0) - 3.0).abs() < 1e-12);
        // a moment already past still leaves a visible motion
        assert!((run_by(Some(9.0), 10.0, 1.0, 3.0) - 1.0).abs() < 1e-12);
        assert!((run_by(None, 10.0, 1.0, 3.0) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn the_average_collapses_the_whole_plane_onto_the_axis_without_turning_back() {
        let mut last = f64::INFINITY;
        for j in 0..=50 {
            let m = lerp(I2, MIRROR, 0.5 * f64::from(j) / 50.0);
            // a sideways scale only, shrinking all the way; heights are kept
            assert!(m[0][0] < last && m[0][0] > -1e-12, "{j}");
            assert!((m[1][1] - 1.0).abs() < 1e-12 && m[0][1].abs() < 1e-12 && m[1][0].abs() < 1e-12);
            last = m[0][0];
        }
        // the end of the run is the matrix on screen, and it flattens every point
        let flat_end = lerp(I2, MIRROR, 0.5);
        assert!(model::mdist(flat_end, [[0.0, 0.0], [0.0, 1.0]]) < 1e-12);
        assert!(flat_end[0][0].abs() < FLAT_SCALE);
        for p in [[4.0, 3.0], [-4.0, -2.0], [1.0, 1.0], [-3.5, 0.0]] {
            let q = model::apply(flat_end, p);
            assert!(q[0].abs() < 1e-12 && (q[1] - p[1]).abs() < 1e-12);
        }
        // the grid's last drawn step before it counts as flat is under a pixel wide
        const { assert!(2.0 * 4.0 * FLAT_SCALE * UNIT * 90.0 < 1.0) };
    }

    #[test]
    fn the_midpoints_of_the_dashes_are_where_the_average_sends_the_corners() {
        let to = letter_f(lerp(I2, MIRROR, 0.5), CENTRE, UNIT);
        for (k, mid) in CHORD_CORNERS.into_iter().zip(chord_midpoints(MIRROR)) {
            assert!((mid[0] - to[k][0]).abs() < 1e-12 && (mid[1] - to[k][1]).abs() < 1e-12);
            assert!((mid[0] - CENTRE[0]).abs() < 1e-12);
        }
    }

    #[test]
    fn the_half_lap_turns_each_axis_away_and_lays_it_back_on_itself_reversed() {
        let turn = |t: f64| model::flow(model::mscale(J, PI), t);
        // halfway through, the x axis lies along the y axis
        let q = model::apply(turn(0.5), [1.0, 0.0]);
        assert!(q[0].abs() < 1e-9 && (q[1] - 1.0).abs() < 1e-9);
        // at the end, each axis is on its own line again, reversed
        for d in [[1.0, 0.0], [0.0, 1.0]] {
            let q = model::apply(turn(1.0), d);
            assert!((q[0] + d[0]).abs() < 1e-9 && (q[1] + d[1]).abs() < 1e-9);
        }
        // the marked axes stay on the panel all the way round
        for j in 0..=20 {
            for end in [[AXES_REACH, 0.0], [0.0, AXES_REACH]] {
                assert!(on_panel(model::apply(turn(f64::from(j) / 20.0), end)), "{j}");
            }
        }
    }

    #[test]
    fn the_sweep_marks_exactly_the_directions_sent_straight_back() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        // unequal negative stretches: only the axes
        let spans = reversed_spans(a, 0.0, PI);
        assert_eq!(spans.len(), 3);
        for (span, at) in spans.iter().zip([0.0, PI / 2.0, PI]) {
            assert!((span.0 - at).abs() < 1e-9 && (span.1 - at).abs() < 1e-9, "{at}");
        }
        // part of the way round, only the axes passed so far
        assert_eq!(reversed_spans(a, 0.0, 1.0).len(), 1);
        // −I and −cI: every direction, as one unbroken span of the whole lap
        for c in [1.0, STRETCH_C] {
            let spans = reversed_spans(model::mscale(I2, -c), PI, 3.0 * PI);
            assert_eq!(spans.len(), 1);
            assert!((spans[0].0 - PI).abs() < 1e-9 && (spans[0].1 - 3.0 * PI).abs() < 1e-9);
        }
        // the edges: nothing is sent back by a stretch or a quarter turn, and the
        // mirror reverses its one axis only
        assert!(reversed_spans(I2, 0.0, 2.0 * PI).is_empty());
        assert!(reversed_spans(model::rotation(PI / 2.0), 0.0, 2.0 * PI).is_empty());
        assert_eq!(reversed_spans(MIRROR, 0.0, PI).len(), 2);
    }

    #[test]
    fn the_sweep_stays_on_the_panel() {
        let a = [[PAIR.0, 0.0], [0.0, PAIR.1]];
        const { assert!(BACK_REACH < CLIP && DIRECTION_LEN < CLIP) };
        for j in 0..=72 {
            let d = direction(f64::from(j) * PI / 36.0);
            for w in [0.0, 0.5, 1.0] {
                for c in [1.0, STRETCH_C] {
                    let m = lerp(a, model::mscale(I2, -c), w);
                    assert!(on_panel(model::apply(m, d)), "{j}");
                }
            }
        }
    }

    #[test]
    fn the_block_rows_follow_the_rule_and_each_verdict_says_what_the_rule_says() {
        for (blocks, verdict) in BLOCK_ROWS {
            assert_eq!(pairs_up(blocks), !verdict.contains("no real log"), "{verdict}");
        }
        // the second row is the flip pair the chapter has been using
        assert_eq!(BLOCK_ROWS[1].0, [(PAIR.0, 1), (PAIR.1, 1)]);
        assert_eq!(fact_list("matrix.flip_pair").unwrap(), [PAIR.0, PAIR.1].to_vec());
        // the rule counts blocks of each size, and leaves positive stretches alone
        assert!(pairs_up(&[(-1.0, 2), (-1.0, 2)]));
        assert!(!pairs_up(&[(-1.0, 1), (-1.0, 2)]));
        assert!(!pairs_up(&[(-1.0, 1), (-1.0, 1), (-1.0, 1)]));
        assert!(pairs_up(&[(3.0, 1)]));
        assert!(pairs_up(&[(3.0, 1), (0.5, 2)]));
    }

    #[test]
    fn a_half_turn_with_an_equal_stretch_has_a_real_logarithm_that_commutes_with_every_x() {
        for c in [1.0f64, 2.0, STRETCH_C] {
            // ln c on the diagonal plus the half-turn spin
            let x = model::madd(model::mscale(I2, c.ln()), model::mscale(J, PI));
            let minus_c = model::mscale(I2, -c);
            assert!(model::mdist(model::flow(x, 1.0), minus_c) < 1e-9, "{c}");
            // −cI commutes with anything: no stretches differ, so no gap
            for other in [FLOW, BASE, J, x] {
                assert!(model::mdist(model::commutator(other, minus_c), [[0.0; 2]; 2]) < 1e-12);
            }
        }
    }

    #[test]
    fn a_real_spin_with_any_shear_that_reaches_a_negative_stretch_reaches_it_in_every_direction() {
        // the 2×2 reason, and why a single size-2 block has no real logarithm: a real
        // X whose exponential has a negative eigenvalue has eigenvalues a ± iπ(2k+1),
        // and then e^X = −e^a I, every direction alike, never a Jordan block
        for (a, shear, odd) in [(0.0f64, 1.0, 1.0), (0.4, 2.0, 1.0), (-0.7, 0.5, 1.0), (0.2, 1.5, 3.0)] {
            let spin = odd * PI;
            let x = [[a, -spin * shear], [spin / shear, a]];
            let e = model::flow(x, 1.0);
            assert!(model::mdist(e, model::mscale(I2, -a.exp())) < 1e-8, "{a} {shear}");
        }
    }

    #[test]
    fn the_sweep_words_and_block_rows_stay_out_of_the_text_column() {
        for (s, size) in [
            ("white: a direction", 18.0),
            ("gold: where A sends it", 18.0),
            ("dashes: straight back", 18.0),
            ("A = diag(−2, −½)", 22.0),
            ("only the two axes go straight back", 20.0),
            ("A = −I", 22.0),
            ("every direction goes straight back", 20.0),
            ("A = −cI, c > 0", 22.0),
            ("half turn, equal stretch", 20.0),
        ] {
            assert!(GAP_X + width(s, size) * WIDTH_ROOM < 2.2 - 0.1, "{s}");
        }
        const { assert!(GAP_X > CENTRE[0] + UNIT * CLIP + 0.1) };
        for (blocks, verdict) in BLOCK_ROWS {
            assert!(VERDICT_X + width(verdict, 22.0) * WIDTH_ROOM < 2.2 - 0.1, "{verdict}");
            // each row's last tile ends before its verdict starts
            let last = blocks.len() - 1;
            assert!(tile_x(blocks, last) + tile_half(blocks[last].1) < VERDICT_X - 0.1);
            // and left edges line up
            assert!((tile_x(blocks, 0) - tile_half(blocks[0].1) - (ROWS_X - TILE_HALF)).abs() < 1e-12);
        }
        let head = "each row is one matrix, each tile a block and its stretch";
        assert!(ROWS_X - TILE_HALF + width(head, 20.0) * WIDTH_ROOM < 2.2 - 0.1);
        // under the title, over the rows; a pair's bracket clears the next row's tile
        // and the last row stays above the fine print
        const { assert!(BLOCKS_HEAD_Y + 0.12 < 3.2 && ROWS_TOP + TILE_HALF < BLOCKS_HEAD_Y - 0.12) };
        const { assert!(TILE_HALF < BRACKET_TOP && BRACKET_FOOT < ROWS_PITCH - TILE_HALF) };
        const { assert!(ROWS_TOP - 3.0 * ROWS_PITCH - BRACKET_FOOT > -3.4) };
    }

    #[test]
    fn shown_numbers_use_halves_and_true_minus() {
        assert_eq!(shown_num(-0.5), "\u{2212}½");
        assert_eq!(shown_num(0.5), "½");
        assert_eq!(shown_num(-2.0), "\u{2212}2");
        assert_eq!(shown_num(2.0), "2");
    }
}
