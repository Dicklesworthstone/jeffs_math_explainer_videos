//! Chapter 08, the shadow rule. From 07's end (the stick lying across the
//! first placement's lines, its three crossings and count, the ruler, the
//! light and the full 6 cm shadow, 07's two rows) the stick tilts to 30°
//! from the lines and its shadow shrinks to 3 cm (08_1). It crosses two
//! lines; held still while the sheet slides up one whole gap, will the
//! count stay at two? (08_2, a prediction.) In silence the sheet slides,
//! the count follows, and a step plot under the coast keeps it: two, one,
//! two, half the slide at one and half at two, so one and a half on
//! average over every slide (08_3). Cut the shadow one gap above its lower
//! end and slide again: the whole gap always holds one line, the
//! centimetre left over holds one half the time (08_4, 08_5). So the
//! average over every slide is the shadow divided by the gap (08_6).
//! Tested on a 4.5 cm shadow (08_7, a prediction): two and a quarter
//! (08_8); and 07's three and zero agree (08_9). The stick, its shadow, its
//! crossings, their heights on the ruler, the lines and the count are drawn
//! every frame from two trackers, the stick's direction and the sheet's
//! placement, through `straight_sticks`'s model, and the plot from this
//! file's runs of the same model. [`Bench`] builds it all for chapter 09
//! too. Starts where chapter 07 ends. Not registered yet: root wires it in
//! (`docs/candidate-08-09.md`).

use std::f64::consts::PI;
use std::rc::Rc;

use explainer_kit::{Kit, Palette, TextKind, ink_style, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::draw::{
    EVERY, card_panel, card_point, dashed, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now, path, say,
    stand, think,
};
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, Move, Placement, Pose, Run, SLIDE_NAME, WORD_GAP, at_once, faded_line, glide, tracked,
};
use super::sheet::{self, COUNT_SIZE, DOT_R, LABEL_SIZE, Sheet, THINK_AT};
use super::straight_sticks::{ACROSS, ALONG, HOME, LENGTH, crossings, ends, shadow};
use crate::facts;
use crate::model::{self, Curve, GAP, P2};
use crate::palette;

pub struct TheShadowRule {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the model

/// How far 08_1 tilts the stick from the lines, radians (`stick.tilt`, 30°),
/// and its direction then: on from across through the upper left, the way
/// 07 turned it, so its upper end stays clear of the count.
pub const TILT: f64 = PI / 6.0;
pub const TILTED: f64 = ALONG - TILT;
/// The shadow 08_7 tests the rule on, cm (`stick.test_shadow`).
pub const TEST_SHADOW: f64 = 4.5;

/// The direction, on from across through the upper left, at which the
/// stick's shadow is `reach` cm (no more than its length).
pub fn tilted_to(reach: f64) -> f64 {
    ALONG - (reach / LENGTH).asin()
}

/// The stick's shadow at `theta`, cm.
pub fn reach(theta: f64) -> f64 {
    let [lo, hi] = shadow(HOME, theta);
    hi - lo
}

/// The steps a slide through one gap is read at, as the oracle reads its
/// own (block 3's `slide_steps`).
pub const SLIDE_STEPS: usize = 20000;

/// The counts the stick at `theta` passes as the sheet at direction 0
/// slides one gap up from `from`, in order, each with the share of the
/// slide it lasts. The slide is read at the middle of each of
/// [`SLIDE_STEPS`] equal steps, as the oracle reads it, so no reading falls
/// on a change; a run's ends are the ends of its steps.
pub fn slide_runs(theta: f64, from: f64) -> Vec<Run> {
    let n = SLIDE_STEPS as f64;
    let mut out: Vec<Run> = Vec::new();
    for i in 0..SLIDE_STEPS {
        let (u0, u1) = (i as f64 / n, (i + 1) as f64 / n);
        let count = crossings(HOME, theta, from + GAP * (u0 + u1) / 2.0).len();
        match out.last_mut() {
            Some(r) if r.count == count => r.to = u1,
            _ => out.push(Run {
                count,
                from: u0,
                to: u1,
            }),
        }
    }
    out
}

/// The share of the slide the runs spend at `count`.
pub fn share(runs: &[Run], count: usize) -> f64 {
    runs.iter().filter(|r| r.count == count).map(|r| r.to - r.from).sum()
}

/// The average count over the slide.
pub fn mean(runs: &[Run]) -> f64 {
    runs.iter().map(|r| r.count as f64 * (r.to - r.from)).sum()
}

/// How many lines of the sheet slid by `slide` lie at heights in (lo, hi]:
/// the floor rule the oracle counts the cut shadow's pieces by.
pub fn lines_in(lo: f64, hi: f64, slide: f64) -> i64 {
    ((hi - slide) / GAP).floor() as i64 - ((lo - slide) / GAP).floor() as i64
}

/// 08_4's cut of the shadow at `theta`: one whole gap up from its lower
/// end, then the part left over above it, heights cm.
pub fn pieces(theta: f64) -> [[f64; 2]; 2] {
    let [lo, hi] = shadow(HOME, theta);
    [[lo, lo + GAP], [lo + GAP, hi]]
}

/// What the two pieces hold as the sheet slides one gap up from `from`,
/// read at the middle of each step: every count the whole gap holds,
/// smallest first, and the share of the slide the part left over holds a
/// line.
pub fn cut_counts(theta: f64, from: f64) -> (Vec<i64>, f64) {
    let [[lo, mid], [_, hi]] = pieces(theta);
    let mut whole: Vec<i64> = Vec::new();
    let mut held = 0;
    for i in 0..SLIDE_STEPS {
        let s = from + GAP * (i as f64 + 0.5) / SLIDE_STEPS as f64;
        let n = lines_in(lo, mid, s);
        if !whole.contains(&n) {
            whole.push(n);
        }
        held += lines_in(mid, hi, s);
    }
    whole.sort_unstable();
    (whole, held as f64 / SLIDE_STEPS as f64)
}

/// How far the nearest line lies from a height, cm.
fn off_lines(h: f64, slide: f64) -> f64 {
    let t = (h - slide) / GAP;
    (t - t.round()).abs() * GAP
}

/// What the voice says the model shows: the counts each slide passes
/// (08_3, 08_8), half the slide at one and half at two (08_3), a quarter
/// at three (08_8), and the part left over holding a line half the time
/// (08_5). No still pose may put an end nearer a line than [`CLEAR`].
const SAID_TILTED: [usize; 3] = [2, 1, 2];
const SAID_TESTED: [usize; 3] = [2, 3, 2];
const SAID_HALF: f64 = 0.5;
const SAID_QUARTER: f64 = 0.25;
const CLEAR: f64 = 0.2;

/// 08_6's rule, in words.
pub const RULE: &str = r"\text{average over every slide} = \text{shadow} \div \text{gap}";

/// The chapter's words that carry numbers, each from the facts, and the
/// runs the plot draws. Refused unless the model, at the first placement
/// (`slide`), gives what the voice says: the counts above, both averages
/// the shadow over the gap, the cut's whole gap holding one line at every
/// step, and every quotient on screen what its shown operands give.
pub struct Words {
    /// 08_1's angle, `30^\circ`.
    pub tilt: String,
    /// 07's two rows and its last label, as 07 left them.
    pub old_rows: [String; 2],
    pub old_shadow: String,
    /// "shadow 3 cm", "shadow 4.5 cm".
    pub shadows: [String; 2],
    /// The cut's pieces, "2 cm" and "1 cm".
    pub pieces: [String; 2],
    /// The plot's slide at its right end, one gap, "2 cm".
    pub gap: String,
    /// "average 1.5", "average 2.25", and the averages the marks are drawn at.
    pub averages: [String; 2],
    pub means: [f64; 2],
    /// The rule's four uses: the two slides, then 07's across and along.
    pub uses: [String; 4],
    /// The two slides' runs, the tilt's then the test's.
    pub runs: [Vec<Run>; 2],
}

pub fn words(slide: f64) -> Result<Words, String> {
    words_for(slide, TILT, TEST_SHADOW)
}

fn words_for(slide: f64, tilt: f64, test_shadow: f64) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;
    let agree = |key: &str, got: String| -> Result<String, String> {
        let want = get(key)?;
        if want == got {
            Ok(want)
        } else {
            Err(format!("{key}: the model gives {got}, the facts {want}"))
        }
    };

    // the model first, so a tilt or test the voice does not describe is
    // refused by what the stick does there, not only by the facts
    let (tilted, tested) = (ALONG - tilt, tilted_to(test_shadow));
    for theta in [tilted, tested] {
        let [lo, hi] = shadow(HOME, theta);
        let near = off_lines(lo, slide).min(off_lines(hi, slide));
        if near < CLEAR {
            return Err(format!(
                "at {:.2}° the stick's ends come {near:.3} cm from a line",
                theta.to_degrees()
            ));
        }
    }
    if (reach(tested) - test_shadow).abs() > 1e-9 {
        return Err(format!("08_7's shadow is {} cm, not {test_shadow}", reach(tested)));
    }

    // the two slides: the counts they pass, from the count shown at rest
    let runs = [slide_runs(tilted, slide), slide_runs(tested, slide)];
    for (said, got, key, first_key, theta) in [
        (
            SAID_TILTED,
            &runs[0],
            "stick.tilted_counts",
            "stick.tilted_first",
            tilted,
        ),
        (SAID_TESTED, &runs[1], "stick.test_counts", "stick.test_first", tested),
    ] {
        let seen: Vec<usize> = got.iter().map(|r| r.count).collect();
        let still = crossings(HOME, theta, slide).len();
        if seen != said || still != said[0] {
            return Err(format!(
                "the voice says the slide passes {said:?} from {}, the model {seen:?} from {still}",
                said[0]
            ));
        }
        agree(key, seen.iter().map(usize::to_string).collect::<Vec<_>>().join(","))?;
        agree(first_key, still.to_string())?;
    }
    let gap = agree("sheet.gap", fixed(GAP, 0))?;
    let degrees = agree("stick.tilt", fixed(tilt.to_degrees(), 0))?;
    let half = fixed(SAID_HALF, 2);
    let one = agree("stick.tilted_one_share", fixed(share(&runs[0], 1), 2))?;
    let two = agree("stick.tilted_two_share", fixed(share(&runs[0], 2), 2))?;
    let three = agree("stick.test_three_share", fixed(share(&runs[1], 3), 2))?;
    if one != half || two != half || three != fixed(SAID_QUARTER, 2) {
        return Err(format!(
            "the voice says half at one, half at two and a quarter at three: the model {one}, {two}, {three}"
        ));
    }
    let means = [mean(&runs[0]), mean(&runs[1])];
    for (m, theta) in means.iter().zip([tilted, tested]) {
        if (m - reach(theta) / GAP).abs() > 1e-3 {
            return Err(format!(
                "a slide averages {m:.4}, its shadow over the gap {:.4}",
                reach(theta) / GAP
            ));
        }
    }
    let first_mean = agree("stick.at30_mean", fixed(means[0], 1))?;
    let test_mean = agree("stick.test_mean", fixed(means[1], 2))?;
    let short = agree("stick.reach30", fixed(reach(tilted), 0))?;
    let long = agree("stick.test_shadow", fixed(test_shadow, 1))?;

    // the cut: the whole gap holds one line at every step, the part left
    // over a line for its share of the slide
    let (whole, held) = cut_counts(tilted, slide);
    agree(
        "stick.gap_piece_counts",
        whole.iter().map(i64::to_string).collect::<Vec<_>>().join(","),
    )?;
    let left_share = agree("stick.leftover_share", fixed(held, 2))?;
    let left = agree("stick.tilted_leftover", fixed(reach(tilted) - GAP, 0))?;
    if whole != [1] || left_share != half || (held - (reach(tilted) - GAP) / GAP).abs() > 1e-3 {
        return Err(format!(
            "08_5 says the whole gap held one line and the {left} cm left over one half the time: the model {whole:?} and {held:.4}"
        ));
    }

    // 07's readings, and the rule's quotients from their shown operands
    let (across, along) = (
        agree("stick.shadow_across", fixed(reach(ACROSS), 0))?,
        agree("stick.shadow_along", fixed(reach(ALONG), 0))?,
    );
    let across_rule = agree("stick.across_rule", fixed(reach(ACROSS) / GAP, 0))?;
    let along_rule = agree("stick.along_rule", fixed(reach(ALONG) / GAP, 0))?;
    let quotient = |a: &str, r: &str| -> Result<(), String> {
        let x: f64 = a.parse().map_err(|e| format!("{a}: {e}"))?;
        let g: f64 = gap.parse().map_err(|e| format!("{gap}: {e}"))?;
        let places = r.split('.').nth(1).map_or(0, str::len);
        if fixed(x / g, places) == r {
            Ok(())
        } else {
            Err(format!("the screen shows {a} ÷ {gap} = {r}"))
        }
    };
    for (a, r) in [
        (&short, &first_mean),
        (&long, &test_mean),
        (&across, &across_rule),
        (&along, &along_rule),
    ] {
        quotient(a, r)?;
    }
    let (n_across, n_along) = (get("stick.home_across")?, get("stick.home_along")?);
    let (estimate, none) = (get("stick.across_estimate")?, get("stick.along_estimate")?);
    let lying = [ACROSS, ALONG].map(|theta| crossings(HOME, theta, slide).len().to_string());
    if lying != [n_across.clone(), n_along.clone()] {
        return Err(format!(
            "07 left {n_across} and {n_along} crossings, the model gives {lying:?}"
        ));
    }

    Ok(Words {
        tilt: format!(r"{degrees}^\circ"),
        old_rows: [
            format!(r"\text{{across}}\quad {n_across} \times \pi\text{{ cm}} \approx {estimate}\text{{ cm}}"),
            format!(r"\text{{along}}\quad {n_along} \times \pi\text{{ cm}} = {none}\text{{ cm}}"),
        ],
        old_shadow: format!("shadow {across} cm"),
        shadows: [format!("shadow {short} cm"), format!("shadow {long} cm")],
        pieces: [format!("{gap} cm"), format!("{left} cm")],
        gap: format!("{gap} cm"),
        averages: [format!("average {first_mean}"), format!("average {test_mean}")],
        means,
        uses: [
            format!(r"{short} \div {gap} = {first_mean}"),
            format!(r"{long} \div {gap} = {test_mean}"),
            format!(r"\text{{across}}\quad {across} \div {gap} = {across_rule}"),
            format!(r"\text{{along}}\quad {along} \div {gap} = {along_rule}"),
        ],
        runs,
    })
}

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
/// What 06 left the card and coast dimmed to.
const DIMMED: f64 = 0.25;
/// 07's stick, shadow and light, as 07 draws them.
const STICK_W: f64 = 6.0;
const SHADOW_W: f64 = 7.0;
const LIGHT_W: f64 = 1.5;
const LIGHT_OPACITY: f64 = 0.7;
const DASH: f64 = 0.1;
const DASH_GAP: f64 = 0.08;
const POINT_R: f64 = 0.06;
/// 07's ruler, cm: upright just beyond the lines' left ends, from under
/// the stick's lowest reach to over its highest, a tick each centimetre
/// from the stick's centre height.
pub const RULER_X: f64 = -7.87;
pub const RULER_Y: [f64; 2] = [HOME[1] - LENGTH / 2.0 - 0.2, HOME[1] + LENGTH / 2.0 + 0.2];
const RULER_TICK: f64 = 0.12;
const RULER_W: f64 = 2.5;
/// Where 07 left "shadow 6 cm", and where this chapter's shadow labels go,
/// left-aligned, cm: 0.55 cm higher, clear of the stick's upper end at
/// every tilt 08 makes, on its own backing.
const SHADOW_07: P2 = [-7.55, 3.45];
pub const SHADOW_AT: P2 = [-7.55, 4.0];
/// 07's two rows, left-aligned in the band under the coast, frame.
const EQUATION_07: f64 = 28.0;
const ROWS_07: [Vec3; 2] = [[0.4, sheet::LOW_Y + 0.22, 0.0], [0.4, sheet::LOW_Y - 0.22, 0.0]];

/// 08_1's angle at the stick's centre, cm: a dashed reference along the
/// lines, the arc from it to the stick, and "30°" inside the wedge.
const ANGLE_REF: f64 = 2.6;
const ANGLE_R: f64 = 1.2;
const ANGLE_LABEL: (f64, f64) = (2.1, 168.0);

/// 08_4's cut: a mark across the ruler where the whole gap ends, and the
/// pieces' words just right of the ruler, left-aligned, at heights inside
/// each piece clear of the lines at rest and of the cut, cm.
const CHOP_X: [f64; 2] = [-7.98, -7.76];
const CHOP_W: f64 = 3.0;
const PIECE_X: f64 = -7.55;
const PIECE_Y: [f64; 2] = [1.0, 2.05];
const PIECE_SIZE: f64 = 20.0;

/// The opaque panels that keep the sliding sheet's lines behind words,
/// frame (centre, width, height): the count's; the shadow label's; the
/// console, the band under the coast from the card's bottom up, that the
/// plot and the rows stand on; and the two pieces' words'.
pub const COUNT_BACK: (Vec3, f64, f64) = ([-2.15, sheet::TOP_Y, 0.0], 2.54, 0.56);
pub const LABEL_BACK: (Vec3, f64, f64) = ([-5.035, 3.45, 0.0], 2.17, 0.34);
pub const CONSOLE: (Vec3, f64, f64) = ([0.0, -2.455, 0.0], 12.6, 1.81);
const PIECE_BACK: [f64; 2] = [0.68, 0.28];
const PIECE_BACK_INSET: f64 = 0.08;

/// The step plot, frame: the slide across, 0 to one gap; the count up, a
/// level every 0.3 to 3; its words; the averages' words at its right.
const PLOT_X: [f64; 2] = [-5.7, -2.7];
const PLOT_Y0: f64 = -2.9;
const PLOT_K: f64 = 0.3;
const PLOT_TOP: usize = 3;
const PLOT_TICK: f64 = 0.08;
const PLOT_SIZE: f64 = 18.0;
const PLOT_LABEL_Y: f64 = -3.12;
const PLOT_NUMBER_X: f64 = -5.9;
const AXIS_W: f64 = 2.0;
const TRACE_W: f64 = 4.0;
const MEAN_W: f64 = 2.0;
const MEAN_X: f64 = -2.55;
const MEAN_SIZE: f64 = 20.0;
/// The rule and its uses, left-aligned on the console, frame.
pub const ROW_SIZE: f64 = 24.0;
pub const RULE_AT: Vec3 = [-0.6, -1.85, 0.0];
pub const USES_AT: [Vec3; 4] = [[-0.6, -2.4, 0.0], [-0.6, -2.9, 0.0], [2.8, -2.4, 0.0], [2.8, -2.9, 0.0]];

pub fn plot_x(u: f64) -> f64 {
    PLOT_X[0] + (PLOT_X[1] - PLOT_X[0]) * u
}

pub fn plot_y(count: f64) -> f64 {
    PLOT_Y0 + PLOT_K * count
}

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const HALO_S: f64 = 0.3;
/// 08_1's tilt, 60° (its count changes once, at 113.6°), and 08_7's, 18.6°.
const TILT_S: f64 = 2.5;
const TEST_S: f64 = 2.0;
/// Every slide is one gap at a steady 0.25 cm a second: the tilt's counts
/// show for 1, 4 and 3 s, the test's for 2, 2 and 4.
const GAP_S: f64 = 8.0;

// --------------------------------------------------------- what is drawn

/// The stick's direction, as its tracker holds it.
pub fn direction(s: &fmn::mobject::Stage, turn: Mob) -> f64 {
    s.tracker_value(turn).unwrap_or(ACROSS)
}

/// The stick at its direction, at the opacity `alpha` holds. In the scene.
fn live_stick(stage: &mut Stage<'_>, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let [a, b] = ends(HOME, direction(s, turn));
        let stick = polyline(vec![card_point(a), card_point(b)])
            .style(faded_line(palette::stick(), STICK_W, tracked(s, alpha)))
            .build();
        s.add(v_group(vec![stick]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A gold dot, as [`sheet::crossing_dot`] draws one, at the frame point
/// `at` and the opacity `o`.
fn gold_dot(at: Vec3, o: f64) -> VMobject {
    Circle::new()
        .radius(DOT_R)
        .arc_center(at)
        .style(flat(palette::touch(), o))
        .build()
}

/// The stick's crossings with the sheet at the placement shown, at the
/// opacity `alpha` holds. In the scene.
fn live_dots(stage: &mut Stage<'_>, place: Placement, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let dots: Vec<VMobject> = crossings(HOME, direction(s, turn), place.pose(s).slide)
            .iter()
            .map(|&(_, at)| gold_dot(card_point(at), o))
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Where the lines meet the shadow: a gold dot on the ruler at the height
/// of each crossing, at the placement shown, at the opacity `alpha` holds.
/// In the scene.
fn live_meets(stage: &mut Stage<'_>, place: Placement, turn: Mob, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let dots: Vec<VMobject> = crossings(HOME, direction(s, turn), place.pose(s).slide)
            .iter()
            .map(|&(_, at)| gold_dot(card_point([RULER_X, at[1]]), o))
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// "crossing" for a count of one, "crossings" otherwise.
fn noun(n: usize) -> &'static str {
    if n == 1 { "crossing" } else { CROSSINGS }
}

/// The count's word for `count(s)`, redrawn every frame, its left edge at
/// `at`, at the opacity `alpha` holds, as 07 draws it. In the scene.
fn live_noun(
    kit: &Kit,
    stage: &mut Stage<'_>,
    count: impl Fn(&fmn::mobject::Stage) -> usize + 'static,
    alpha: Mob,
    at: Vec3,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let (o, word) = (tracked(s, alpha), noun(count(s)));
        let t = Text::new(word)
            .font_size(COUNT_SIZE)
            .style(ink_style(palette::touch()))
            .build(&book)
            .expect("the bundled face sets every word on screen");
        explainer_kit::glyphs::record(TextKind::Text, word);
        let m = s.add(t);
        s.move_to(m, at, LEFT);
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Dashes along the frame segment from `a` to `b`.
fn dashes(a: Vec3, b: Vec3) -> Vec<Vec<Vec3>> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    let mut out = Vec::new();
    let mut from = 0.0;
    while from < length {
        let to = (from + DASH).min(length);
        let at = |d: f64| v(a[0] + dx * d / length, a[1] + dy * d / length);
        out.push(vec![at(from), at(to)]);
        from = to + DASH_GAP;
    }
    out
}

/// The stick's shadow on the ruler (a point where its ends are level) and
/// the light from each end along the lines to the ruler, dashed, at the
/// opacities `alpha` and `light` hold, as 07 draws them. In the scene.
fn live_shadow(stage: &mut Stage<'_>, turn: Mob, alpha: Mob, light: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, l, theta) = (tracked(s, alpha), tracked(s, light), direction(s, turn));
        let [lo, hi] = shadow(HOME, theta);
        let mut parts: Vec<VMobject> = Vec::new();
        for end in ends(HOME, theta) {
            for pts in dashes(card_point(end), card_point([RULER_X, end[1]])) {
                parts.push(
                    polyline(pts)
                        .style(faded_line(Palette::dim(), LIGHT_W, LIGHT_OPACITY * l))
                        .build(),
                );
            }
        }
        if hi - lo > 1e-6 {
            parts.push(
                polyline(vec![card_point([RULER_X, lo]), card_point([RULER_X, hi])])
                    .style(faded_line(palette::stick(), SHADOW_W, o))
                    .build(),
            );
        } else {
            parts.push(
                Circle::new()
                    .radius(POINT_R)
                    .arc_center(card_point([RULER_X, lo]))
                    .style(flat(palette::stick(), o))
                    .build(),
            );
        }
        s.add(v_group(parts))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The ruler and its ticks, as 07 draws them. Detached.
fn ruler(stage: &mut Stage<'_>) -> Vec<Mob> {
    let mut out = vec![path(
        stage,
        &[[RULER_X, RULER_Y[0]], [RULER_X, RULER_Y[1]]],
        Palette::ink(),
        RULER_W,
    )];
    let half = (LENGTH / 2.0) as i64;
    for k in -half..=half {
        let y = HOME[1] + k as f64;
        out.push(path(
            stage,
            &[[RULER_X, y], [RULER_X + RULER_TICK, y]],
            Palette::ink(),
            RULER_W,
        ));
    }
    out
}

/// An opaque panel in the faint ink, as 04's backings: an overlay that
/// keeps the sliding lines behind words, not a cut in the sheet. In the
/// scene, unseen until brought up by opacity ([`fade_to`]), so it stays
/// behind every word made later.
pub fn hidden_back(stage: &mut Stage<'_>, (at, width, height): (Vec3, f64, f64)) -> fmn::Result<Mob> {
    let panel = Rectangle::new()
        .width(width)
        .height(height)
        .corner_radius(0.08)
        .style(flat(Palette::faint(), 1.0))
        .build()?
        .moved_to(at);
    let m = stage.arena_mut().add(panel);
    stage.set_family_opacity(m, 0.0);
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Where piece `i`'s words begin, frame, and their backing.
fn piece_at(i: usize) -> Vec3 {
    card_point([PIECE_X, PIECE_Y[i]])
}

fn piece_back(i: usize) -> (Vec3, f64, f64) {
    let at = piece_at(i);
    (
        v(at[0] - PIECE_BACK_INSET + PIECE_BACK[0] / 2.0, at[1]),
        PIECE_BACK[0],
        PIECE_BACK[1],
    )
}

/// The backings, unseen.
pub struct Backs {
    pub count: Mob,
    pub label: Mob,
    pub console: Mob,
    pub pieces: [Mob; 2],
}

/// Everything chapters 08 and 09 draw the stick's experiment with, in the
/// scene from the back: the dimmed card and coast; the sheet, live at its
/// placement, and the footprints of its first placement's lines (unseen);
/// the stick's shadow and light, the stick, its crossings, the ruler and
/// the crossings' heights on it; the backings (unseen); the live count and
/// its word. Each live part has its own opacity tracker.
pub struct Bench {
    pub place: Placement,
    pub turn: Mob,
    pub footprints: Vec<Mob>,
    pub dots_alpha: Mob,
    pub meets_alpha: Mob,
    pub count_alpha: Mob,
    pub backs: Backs,
}

impl Bench {
    /// The bench with the sheet at `first` and the stick at `theta`, as 07
    /// ends: the stick, shadow, light, crossings and count shown, the
    /// heights on the ruler, the footprints and the backings not.
    pub fn new(kit: &Kit, stage: &mut Stage<'_>, coast: &Curve, first: Pose, theta: f64) -> fmn::Result<Self> {
        let card = card_panel(stage)?;
        stage.set_family_opacity(card, DIMMED);
        stage.add_to_scene(card)?;
        for p in &coast.parts {
            let m = path(
                stage,
                &coast.between(p.start, p.end(), EVERY),
                palette::coast(),
                COAST_W,
            );
            stage.set_color_lanes(m, "stroke_rgba", None, Some(DIMMED), true);
            stage.add_to_scene(m)?;
        }
        let panel = Sheet::new(stage, first.slide, 0.0)?.panel;
        stage.add_to_scene(panel)?;
        let place = Placement::new(stage, first);
        let lines_alpha = stage.add_value_tracker(1.0);
        placement::live_lines(stage, place, lines_alpha)?;
        let footprints = placement::footprints(stage, first)?;
        for &m in &footprints {
            stage.set_family_opacity(m, 0.0);
            stage.add_to_scene(m)?;
        }
        let turn = stage.add_value_tracker(theta);
        let [
            shadow_alpha,
            light_alpha,
            stick_alpha,
            dots_alpha,
            meets_alpha,
            count_alpha,
        ] = [1.0, 1.0, 1.0, 1.0, 0.0, 1.0].map(|o| stage.add_value_tracker(o));
        live_shadow(stage, turn, shadow_alpha, light_alpha)?;
        live_stick(stage, turn, stick_alpha)?;
        live_dots(stage, place, turn, dots_alpha)?;
        for m in ruler(stage) {
            stage.add_to_scene(m)?;
        }
        live_meets(stage, place, turn, meets_alpha)?;
        let backs = Backs {
            count: hidden_back(stage, COUNT_BACK)?,
            label: hidden_back(stage, LABEL_BACK)?,
            console: hidden_back(stage, CONSOLE)?,
            pieces: [hidden_back(stage, piece_back(0))?, hidden_back(stage, piece_back(1))?],
        };
        let counted = move |s: &fmn::mobject::Stage| crossings(HOME, direction(s, turn), place.pose(s).slide).len();
        placement::live_count(kit, stage, counted, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        live_noun(
            kit,
            stage,
            counted,
            count_alpha,
            v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]),
        )?;
        Ok(Self {
            place,
            turn,
            footprints,
            dots_alpha,
            meets_alpha,
            count_alpha,
            backs,
        })
    }
}

/// Turn the stick to `to`, `rt` seconds at `rate`; it ends exactly there.
pub fn turn_to(stage: &mut Stage<'_>, turn: Mob, to: f64, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<()> {
    stage.play(glide(turn, to, rt, rate)?)?;
    stage.set_tracker_value(turn, to)?;
    Ok(())
}

/// A detached mobject, its left edge at the frame point `at`.
pub fn left_at(stage: &mut Stage<'_>, m: Mob, at: Vec3) -> Mob {
    stage.move_to(m, at, LEFT);
    m
}

/// A row of TeX on the console in `color`, its left edge at `at`. Detached.
pub fn row(kit: &Kit, stage: &mut Stage<'_>, tex: &str, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let m = kit.tex(stage, tex, ROW_SIZE, color, &[], at)?;
    Ok(left_at(stage, m, at))
}

/// A shadow's length over the ruler's top, on its backing. Detached.
pub fn shadow_label(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    let at = card_point(SHADOW_AT);
    let m = kit.text(stage, words, LABEL_SIZE, palette::stick(), at)?;
    Ok(left_at(stage, m, at))
}

/// Frame points as one open polyline. Detached.
fn frame_path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}

/// The step plot's axes, ticks and words. Detached.
pub fn plot_axes(kit: &Kit, stage: &mut Stage<'_>, gap: &str) -> fmn::Result<Vec<Mob>> {
    let top = plot_y(PLOT_TOP as f64) + PLOT_TICK;
    let mut out = vec![
        frame_path(
            stage,
            vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[1], PLOT_Y0)],
            Palette::dim(),
            AXIS_W,
        ),
        frame_path(
            stage,
            vec![v(PLOT_X[0], PLOT_Y0), v(PLOT_X[0], top)],
            Palette::dim(),
            AXIS_W,
        ),
        frame_path(
            stage,
            vec![v(PLOT_X[1], PLOT_Y0), v(PLOT_X[1], PLOT_Y0 - PLOT_TICK)],
            Palette::dim(),
            AXIS_W,
        ),
    ];
    for k in 1..=PLOT_TOP {
        let y = plot_y(k as f64);
        out.push(frame_path(
            stage,
            vec![v(PLOT_X[0] - PLOT_TICK, y), v(PLOT_X[0], y)],
            Palette::dim(),
            AXIS_W,
        ));
        out.push(kit.text(stage, &k.to_string(), PLOT_SIZE, Palette::dim(), v(PLOT_NUMBER_X, y))?);
    }
    for (word, u) in [("0", 0.0), (SLIDE_NAME, 0.5), (gap, 1.0)] {
        out.push(kit.text(stage, word, PLOT_SIZE, Palette::dim(), v(plot_x(u), PLOT_LABEL_Y))?);
    }
    Ok(out)
}

/// The trace of `runs` from the slide's start to the share `upto`, frame
/// points: level along each run at its count, upright at each change.
pub fn trace_points(runs: &[Run], upto: f64) -> Vec<Vec3> {
    let mut pts = Vec::new();
    for r in runs {
        if r.from >= upto {
            break;
        }
        let y = plot_y(r.count as f64);
        pts.push(v(plot_x(r.from), y));
        pts.push(v(plot_x(r.to.min(upto)), y));
    }
    pts
}

/// The plot's trace of `runs`, gold, drawn as far as the share `upto(s)`,
/// at the opacity `alpha` holds. In the scene.
pub fn live_trace(
    stage: &mut Stage<'_>,
    runs: Vec<Run>,
    upto: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let pts = trace_points(&runs, upto(s));
        let trace: Vec<VMobject> = (pts.len() >= 2)
            .then(|| {
                polyline(pts)
                    .style(faded_line(palette::touch(), TRACE_W, tracked(s, alpha)))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(trace))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// How far a trace is drawn: the slide under way while `drawing` is on,
/// then the share `held` keeps (the whole slide, once it is done).
fn drawn(s: &fmn::mobject::Stage, place: Placement, drawing: Mob, held: Mob) -> f64 {
    if tracked(s, drawing) > 0.5 {
        place.share(s)
    } else {
        tracked(s, held)
    }
}

/// A dashed line across the plot at the average `mean`, and its words at
/// the plot's right. Detached.
pub fn mean_mark(kit: &Kit, stage: &mut Stage<'_>, mean: f64, words: &str) -> fmn::Result<Vec<Mob>> {
    let y = plot_y(mean);
    let line = DashedLine::new(v(PLOT_X[0], y), v(PLOT_X[1], y))
        .dash_length(0.1)
        .style(line_style(Palette::ink(), MEAN_W))
        .build()?;
    let line = stage.arena_mut().add(line);
    let label = kit.text(stage, words, MEAN_SIZE, Palette::ink(), v(MEAN_X, y))?;
    Ok(vec![line, left_at(stage, label, v(MEAN_X, y))])
}

/// A halo along each of the trace's runs at `count`. Detached.
fn halos(stage: &mut Stage<'_>, runs: &[Run], count: usize) -> Vec<Mob> {
    runs.iter()
        .filter(|r| r.count == count)
        .map(|r| {
            let y = plot_y(count as f64);
            placement::halo_along(stage, vec![v(plot_x(r.from), y), v(plot_x(r.to), y)])
        })
        .collect()
}

/// A halo along the ruler from the height `a` to `b`. Detached.
fn ruler_halo(stage: &mut Stage<'_>, a: f64, b: f64) -> Mob {
    placement::halo_along(stage, vec![card_point([RULER_X, a]), card_point([RULER_X, b])])
}

/// 08_1's angle: the reference, the arc and its words. Detached.
fn angle_mark(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Vec<Mob>> {
    let reference = dashed(stage, HOME, [HOME[0] - ANGLE_REF, HOME[1]], Palette::ink())?;
    let arc: Vec<Vec3> = (0..=24)
        .map(|i| {
            let a = TILTED + TILT * f64::from(i) / 24.0;
            card_point([HOME[0] + ANGLE_R * a.cos(), HOME[1] + ANGLE_R * a.sin()])
        })
        .collect();
    let arc = frame_path(stage, arc, Palette::ink(), 2.5);
    let (r, deg) = ANGLE_LABEL;
    let at = card_point([
        HOME[0] + r * deg.to_radians().cos(),
        HOME[1] + r * deg.to_radians().sin(),
    ]);
    let label = kit.tex(stage, words, LABEL_SIZE, Palette::ink(), &[], at)?;
    Ok(vec![reference, arc, label])
}

/// A piece's words, left-aligned just right of the ruler. Detached.
fn piece_label(kit: &Kit, stage: &mut Stage<'_>, words: &str, i: usize) -> fmn::Result<Mob> {
    let m = kit.text(stage, words, PIECE_SIZE, palette::stick(), piece_at(i))?;
    Ok(left_at(stage, m, piece_at(i)))
}

impl SceneConstruct for TheShadowRule {
    fn name(&self) -> &str {
        "08_the_shadow_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let words = words(first.slide).map_err(fail)?;
        let tested = tilted_to(TEST_SHADOW);
        let gap_slide = Move::slide(first, GAP);

        // where 07 ended: the bench with the stick across, 07's two rows
        // and "shadow 6 cm"
        let bench = Bench::new(&kit, stage, &coast, first, ACROSS)?;
        let place = bench.place;
        let mut old = Vec::new();
        for (tex, at) in words.old_rows.iter().zip(ROWS_07) {
            let m = kit.tex(stage, tex, EQUATION_07, Palette::ink(), &[], at)?;
            old.push(left_at(stage, m, at));
        }
        let full = kit.text(
            stage,
            &words.old_shadow,
            LABEL_SIZE,
            palette::stick(),
            card_point(SHADOW_07),
        )?;
        old.push(left_at(stage, full, card_point(SHADOW_07)));
        for &m in &old {
            stage.add_to_scene(m)?;
        }

        // 08_1: 07's words go; the stick tilts to 30° from the lines, its
        // count, crossings and shadow following; the angle, the new shadow
        let line = say(&kit, stage, "08_1")?;
        line.wait_for(stage, "Now tilt the stick")?;
        fade_out_all(stage, &old, FADE_S)?;
        turn_to(stage, bench.turn, TILTED, TILT_S, smooth)?;
        line.wait_for(stage, "thirty degrees from the lines")?;
        let angle = angle_mark(&kit, stage, &words.tilt)?;
        fade_in_all(stage, &angle, FADE_S)?;
        line.wait_for(stage, "shrinks to three centimetres")?;
        let short = shadow_label(&kit, stage, &words.shadows[0])?;
        at_once(
            stage,
            &[short],
            &[],
            vec![fade_to(bench.backs.label, 1.0, FADE_S)?],
            FADE_S,
        )?;

        // 08_2: two crossings at this placement; the angle goes, the
        // console, the count's backing, the empty plot and the footprints
        // come; will the count stay at two?
        let line = say(&kit, stage, "08_2")?;
        line.wait_for(stage, "slide the sheet up through one whole gap")?;
        let axes = plot_axes(&kit, stage, &words.gap)?;
        let [drawing, held, first_alpha] = [1.0, 0.0, 1.0].map(|x| stage.add_value_tracker(x));
        live_trace(
            stage,
            words.runs[0].clone(),
            move |s| drawn(s, place, drawing, held),
            first_alpha,
        )?;
        let mut reveal = vec![
            fade_to(bench.backs.console, 1.0, FADE_S)?,
            fade_to(bench.backs.count, 1.0, FADE_S)?,
        ];
        for &m in &bench.footprints {
            reveal.push(fade_to(m, 1.0, FADE_S)?);
        }
        at_once(stage, &axes, &angle, reveal, FADE_S)?;
        line.wait_for(stage, "Will the count stay at two")?;
        let q = think(&kit, stage, THINK_AT)?;

        // the slide through one gap, in silence: the count, the crossings
        // and the trace follow the sheet; the lines land on their footprints
        fade_out_all(stage, &[q], 0.3)?;
        place.make(stage, &gap_slide, &[(1.0, GAP_S)], linear)?;
        stage.set_tracker_value(held, 1.0)?;
        stage.set_tracker_value(drawing, 0.0)?;
        place.settle(stage)?;

        // 08_3: two, one, two; half at each; the average over every slide
        let line = say(&kit, stage, "08_3")?;
        line.wait_for(stage, "half of them at one crossing")?;
        let ones = halos(stage, &words.runs[0], 1);
        fade_in_all(stage, &ones, HALO_S)?;
        line.wait_for(stage, "and half at two")?;
        let twos = halos(stage, &words.runs[0], 2);
        at_once(stage, &twos, &ones, vec![], HALO_S)?;
        line.wait_for(stage, "one and a half lines")?;
        let mut first_mean = mean_mark(&kit, stage, words.means[0], &words.averages[0])?;
        at_once(stage, &first_mean, &twos, vec![], FADE_S)?;

        // 08_4: the cut, the pieces' words, the crossings' heights on the
        // ruler; then the same slide again, in silence
        let line = say(&kit, stage, "08_4")?;
        let [[lo, mid], [_, hi]] = pieces(TILTED);
        line.wait_for(stage, "Cut the shadow")?;
        let chop = path(stage, &[[CHOP_X[0], mid], [CHOP_X[1], mid]], Palette::ink(), CHOP_W);
        fade_in_all(stage, &[chop], FADE_S)?;
        line.wait_for(stage, "one whole gap, two centimetres")?;
        let whole = piece_label(&kit, stage, &words.pieces[0], 0)?;
        at_once(
            stage,
            &[whole],
            &[],
            vec![fade_to(bench.backs.pieces[0], 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "the one centimetre left over")?;
        let left = piece_label(&kit, stage, &words.pieces[1], 1)?;
        at_once(
            stage,
            &[left],
            &[],
            vec![fade_to(bench.backs.pieces[1], 1.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "watch where the lines meet the shadow")?;
        stage.play(tween(bench.meets_alpha, 1.0, FADE_S)?)?;
        kit.hold(stage)?;
        place.make(stage, &gap_slide, &[(1.0, GAP_S)], linear)?;
        place.settle(stage)?;

        // 08_5: the whole gap always held one line; the part left over,
        // half the time
        let line = say(&kit, stage, "08_5")?;
        line.wait_for(stage, "The whole gap always held one line")?;
        let low = ruler_halo(stage, lo, mid);
        fade_in_all(stage, &[low], HALO_S)?;
        line.wait_for(stage, "The leftover centimetre")?;
        let high = ruler_halo(stage, mid, hi);
        at_once(stage, &[high], &[low], vec![], HALO_S)?;
        line.wait_for(stage, "One, plus a half")?;
        fade_out_all(stage, &[high], HALO_S)?;

        // 08_6: 3 ÷ 2 = 1.5; the cut goes; the rule
        let line = say(&kit, stage, "08_6")?;
        let uses = words
            .uses
            .iter()
            .zip(USES_AT)
            .map(|(tex, at)| row(&kit, stage, tex, Palette::ink(), at))
            .collect::<fmn::Result<Vec<Mob>>>()?;
        line.wait_for(stage, "divided by the gap, two")?;
        fade_in_all(stage, &uses[..1], FADE_S)?;
        line.wait_for(stage, "That works for any shadow")?;
        at_once(
            stage,
            &[],
            &[chop, whole, left],
            vec![
                fade_to(bench.backs.pieces[0], 0.0, FADE_S)?,
                fade_to(bench.backs.pieces[1], 0.0, FADE_S)?,
                tween(bench.meets_alpha, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        line.wait_for(stage, "the average count over every slide")?;
        let rule = row(&kit, stage, RULE, Palette::ink(), RULE_AT)?;
        fade_in_all(stage, &[rule], FADE_S)?;

        // 08_7: the first trace, its average and the 3 cm label go; the
        // stick turns part of the way back, against 08_1's tilt, to a 4.5 cm
        // shadow; what should the average be?
        let line = say(&kit, stage, "08_7")?;
        line.wait_for(stage, "Turn the stick part of the way back")?;
        first_mean.push(short);
        at_once(stage, &[], &first_mean, vec![tween(first_alpha, 0.0, FADE_S)?], FADE_S)?;
        turn_to(stage, bench.turn, tested, TEST_S, smooth)?;
        line.wait_for(stage, "four and a half centimetres")?;
        let long = shadow_label(&kit, stage, &words.shadows[1])?;
        fade_in_all(stage, &[long], FADE_S)?;
        line.wait_for(stage, "What should its average count be now")?;
        let [drawing, held, test_alpha] = [1.0, 0.0, 1.0].map(|x| stage.add_value_tracker(x));
        live_trace(
            stage,
            words.runs[1].clone(),
            move |s| drawn(s, place, drawing, held),
            test_alpha,
        )?;
        let q = think(&kit, stage, THINK_AT)?;

        // the test slide, in silence
        fade_out_all(stage, &[q], 0.3)?;
        place.make(stage, &gap_slide, &[(1.0, GAP_S)], linear)?;
        stage.set_tracker_value(held, 1.0)?;
        stage.set_tracker_value(drawing, 0.0)?;
        place.settle(stage)?;

        // 08_8: two and a quarter; three for a quarter of the slide
        let line = say(&kit, stage, "08_8")?;
        line.wait_for(stage, "Two and a quarter")?;
        let test_mean = mean_mark(&kit, stage, words.means[1], &words.averages[1])?;
        fade_in_all(stage, &test_mean, FADE_S)?;
        line.wait_for(stage, "three lines for a quarter of the slide")?;
        let threes = halos(stage, &words.runs[1], 3);
        fade_in_all(stage, &threes, HALO_S)?;
        line.wait_for(stage, "four and a half divided by two")?;
        at_once(stage, &uses[1..2], &threes, vec![], FADE_S)?;

        // 08_9: 07's two readings by the rule
        let line = say(&kit, stage, "08_9")?;
        line.wait_for(stage, "six over two is three")?;
        fade_in_all(stage, &uses[2..3], FADE_S)?;
        line.wait_for(stage, "zero over two is zero")?;
        fade_in_all(stage, &uses[3..], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::THINK_R;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Part, Seg};

    fn first_slide() -> f64 {
        sheet::shown_slide(&model::coast(), 0.0)
    }

    /// The stick as a one-piece curve, for `sheet::crossings`.
    fn as_curve(theta: f64) -> Curve {
        let [from, to] = ends(HOME, theta);
        Curve {
            parts: vec![Part {
                seg: Seg::Line { from, to },
                start: 0.0,
            }],
            length: LENGTH,
        }
    }

    /// A text `size` points tall is taken as at most size/75 frame units
    /// high, size/150 a character wide, as the other chapters' layout
    /// tests take it.
    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    /// The frame box (x0, x1, y0, y1) of a backing.
    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    /// How far the frame point `p` lies outside the box `b` (0 inside).
    fn outside(p: Vec3, b: [f64; 4]) -> f64 {
        let dx = (b[0] - p[0]).max(p[0] - b[1]).max(0.0);
        let dy = (b[2] - p[1]).max(p[1] - b[3]).max(0.0);
        dx.hypot(dy)
    }

    /// Tilted 30° from the lines the shadow runs from −0.3 to 2.7 cm, 3 cm,
    /// two crossings at rest, each end at least a quarter centimetre from
    /// the lines; at 08_7's tilt, 48.59° from the lines, from −1.05 to
    /// 3.45 cm, 4.5 cm, two crossings, the ends half a centimetre off.
    #[test]
    fn the_tilts_show_what_the_voice_says() {
        let s = first_slide();
        let [lo, hi] = shadow(HOME, TILTED);
        assert!((lo + 0.3).abs() < 1e-12 && (hi - 2.7).abs() < 1e-12, "{lo} {hi}");
        assert!((reach(TILTED) - 3.0).abs() < 1e-12);
        assert_eq!(crossings(HOME, TILTED, s).len(), 2);
        assert!((off_lines(lo, s).min(off_lines(hi, s)) - 0.25).abs() < 1e-9);
        let tested = tilted_to(TEST_SHADOW);
        let [lo, hi] = shadow(HOME, tested);
        assert!((lo + 1.05).abs() < 1e-9 && (hi - 3.45).abs() < 1e-9, "{lo} {hi}");
        assert_eq!(facts::fixed(180.0 - tested.to_degrees(), 2), "48.59");
        assert_eq!(crossings(HOME, tested, s).len(), 2);
        assert!((off_lines(lo, s).min(off_lines(hi, s)) - 0.5).abs() < 1e-9);
        // the tilt passes 3 then 2 (at 113.56°), the test stays at 2
        let seen = |a: f64, b: f64| -> Vec<usize> {
            let mut out: Vec<usize> = Vec::new();
            for i in 0..=6000 {
                let n = crossings(HOME, a + (b - a) * f64::from(i) / 6000.0, s).len();
                if out.last() != Some(&n) {
                    out.push(n);
                }
            }
            out
        };
        assert_eq!(seen(ACROSS, TILTED), [3, 2]);
        assert_eq!(seen(TILTED, tested), [2]);
    }

    /// The tilt's slide passes 2, 1, 2 for 1/8, 1/2 and 3/8 of the gap, the
    /// test's 2, 3, 2 for 1/4, 1/4 and 1/2: averages 1.5 and 2.25, each its
    /// shadow over the gap. At every step's middle the count agrees with
    /// `sheet::crossings` on the stick as a one-piece curve and with the
    /// oracle's floor rule, the difference of the two ends' line numbers.
    #[test]
    fn the_slides_pass_the_counts_the_voice_names() {
        let s = first_slide();
        for (theta, counts, lasts, avg) in [
            (TILTED, [2, 1, 2], [0.125, 0.5, 0.375], 1.5),
            (tilted_to(TEST_SHADOW), [2, 3, 2], [0.25, 0.25, 0.5], 2.25),
        ] {
            let runs = slide_runs(theta, s);
            assert_eq!(runs.iter().map(|r| r.count).collect::<Vec<_>>(), counts);
            for (r, want) in runs.iter().zip(lasts) {
                assert!((r.to - r.from - want).abs() < 1e-9, "{r:?}");
            }
            assert!((runs[0].from).abs() < 1e-15 && (runs[2].to - 1.0).abs() < 1e-15);
            for w in runs.windows(2) {
                assert_eq!(w[0].to, w[1].from);
            }
            assert!((mean(&runs) - avg).abs() < 1e-9 && (avg - reach(theta) / GAP).abs() < 1e-9);
            let curve = as_curve(theta);
            let [a, b] = ends(HOME, theta);
            for i in 0..SLIDE_STEPS {
                let slid = s + GAP * (i as f64 + 0.5) / SLIDE_STEPS as f64;
                let n = crossings(HOME, theta, slid).len();
                assert_eq!(n, sheet::crossings(&curve, 0.0, slid).len());
                let line_of = |p: P2| ((p[1] - slid) / GAP).floor() as i64;
                assert_eq!((line_of(a) - line_of(b)).unsigned_abs() as usize, n);
            }
        }
        // 08_8's "a quarter of the slide at three", 08_3's halves
        let tilt = slide_runs(TILTED, s);
        assert!((share(&tilt, 1) - 0.5).abs() < 1e-9 && (share(&tilt, 2) - 0.5).abs() < 1e-9);
        assert!((share(&slide_runs(tilted_to(TEST_SHADOW), s), 3) - 0.25).abs() < 1e-9);
    }

    /// The cut: from the shadow's lower end one whole gap holds exactly one
    /// line at every slide, whatever the shadow and the slide (its lines
    /// leave at the top as the next comes in at the bottom); the part left
    /// over holds one for its share of the gap; together they hold the
    /// stick's count at every step of the tilt's slide.
    #[test]
    fn the_whole_gap_always_holds_one_line() {
        for i in 0..400 {
            for j in 0..50 {
                let lo = -3.0 + 0.0173 * f64::from(i);
                let slide = 0.0371 * f64::from(j);
                assert_eq!(lines_in(lo, lo + GAP, slide), 1, "{lo} {slide}");
            }
        }
        let s = first_slide();
        let [[lo, mid], [_, hi]] = pieces(TILTED);
        assert!((mid - 1.7).abs() < 1e-12 && (hi - mid - 1.0).abs() < 1e-12);
        let (whole, held) = cut_counts(TILTED, s);
        assert_eq!(whole, [1]);
        assert!((held - 0.5).abs() < 1e-9);
        for i in 0..SLIDE_STEPS {
            let slid = s + GAP * (i as f64 + 0.5) / SLIDE_STEPS as f64;
            let n = crossings(HOME, TILTED, slid).len() as i64;
            assert_eq!(lines_in(lo, mid, slid) + lines_in(mid, hi, slid), n);
        }
    }

    /// "For a straight stick at any angle, the average count over every
    /// slide is its shadow divided by the gap": at every whole degree of
    /// half a turn, within the reading's step. Lying across, the shadow is
    /// three whole gaps and the average three, though at the one slide
    /// that puts both ends on lines the count is two, which is why the
    /// voice claims only the average there; lying along it is zero.
    #[test]
    fn the_rule_holds_at_every_degree() {
        let s = first_slide();
        for d in 0..=180 {
            let theta = f64::from(d).to_radians();
            let got = mean(&slide_runs(theta, s));
            assert!(
                (got - reach(theta) / GAP).abs() < 2.0 / SLIDE_STEPS as f64,
                "at {d}°: {got}"
            );
        }
        assert!((mean(&slide_runs(ACROSS, s)) - 3.0).abs() < 1e-12);
        assert_eq!(mean(&slide_runs(ALONG, s)), 0.0);
        // both ends on lines: lying across with the lowest end on one
        let [lo, _] = shadow(HOME, ACROSS);
        assert_eq!(crossings(HOME, ACROSS, lo).len(), 2);
    }

    /// The words on screen say what the facts and the model say, and 07's
    /// rebuilt words are 07's own.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words(first_slide()).unwrap();
        assert_eq!(w.tilt, r"30^\circ");
        assert_eq!(
            w.old_rows,
            [
                r"\text{across}\quad 3 \times \pi\text{ cm} \approx 9.4\text{ cm}",
                r"\text{along}\quad 0 \times \pi\text{ cm} = 0\text{ cm}",
            ]
        );
        assert_eq!(w.old_shadow, "shadow 6 cm");
        assert_eq!(w.shadows, ["shadow 3 cm", "shadow 4.5 cm"]);
        assert_eq!(w.pieces, ["2 cm", "1 cm"]);
        assert_eq!(w.gap, "2 cm");
        assert_eq!(w.averages, ["average 1.5", "average 2.25"]);
        assert_eq!(
            w.uses,
            [
                r"3 \div 2 = 1.5",
                r"4.5 \div 2 = 2.25",
                r"\text{across}\quad 6 \div 2 = 3",
                r"\text{along}\quad 0 \div 2 = 0",
            ]
        );
    }

    /// Tilts and tests the voice does not describe are refused, each by the
    /// model's own check before any fact is read: 25° (an end 0.018 cm from
    /// a line), 45° (the slide passes 2, 3, 2), a 4 cm shadow (two whole
    /// gaps, the count never changes) and a 5.4 cm one (an end 0.05 cm from
    /// a line). So is a sheet slid a quarter centimetre (an end on a line).
    #[test]
    fn a_tilt_the_voice_does_not_describe_is_refused() {
        let s = first_slide();
        assert!(words_for(s, TILT, TEST_SHADOW).is_ok());
        let refusal = |r: Result<Words, String>| r.err().unwrap_or_default();
        for (tilt, test, slide, why) in [
            (25.0_f64.to_radians(), TEST_SHADOW, s, "from a line"),
            (45.0_f64.to_radians(), TEST_SHADOW, s, "the slide passes"),
            (TILT, 4.0, s, "the slide passes"),
            (TILT, 5.4, s, "from a line"),
            (TILT, TEST_SHADOW, s + 0.25, "from a line"),
        ] {
            let got = refusal(words_for(slide, tilt, test));
            assert!(got.contains(why), "{tilt} {test} {slide}: {got:?}");
        }
    }

    /// The plot is to scale: each run's level is its count, its length its
    /// share of the gap, and each average's mark its height.
    #[test]
    fn the_plot_is_to_scale() {
        let w = words(first_slide()).unwrap();
        let pts = trace_points(&w.runs[0], 1.0);
        assert_eq!(pts.len(), 6);
        let span = PLOT_X[1] - PLOT_X[0];
        assert!((pts[0][0] - PLOT_X[0]).abs() < 1e-12 && (pts[5][0] - PLOT_X[1]).abs() < 1e-12);
        assert!((pts[3][0] - pts[2][0] - span / 2.0).abs() < 1e-9);
        assert!((pts[2][1] - plot_y(1.0)).abs() < 1e-12 && (pts[0][1] - plot_y(2.0)).abs() < 1e-12);
        assert!(trace_points(&w.runs[0], 0.0).is_empty());
        assert_eq!(trace_points(&w.runs[0], 0.3).len(), 4);
        for (m, avg) in w.means.iter().zip([1.5, 2.25]) {
            assert!((plot_y(*m) - plot_y(avg)).abs() < 1e-6);
        }
        assert!((plot_y(1.5) - PLOT_Y0 - 1.5 * PLOT_K).abs() < 1e-12);
    }

    /// At every tenth of a degree of 08_1's tilt the stick keeps clear of
    /// the count and its backing, and of 08_7's tilt clear of the shadow
    /// label's backing too (shown then); the console stands under the
    /// stick at every direction and under the coast; the angle's words sit
    /// inside the wedge; the pieces' words inside their pieces, clear of
    /// the cut and of the lines at rest, their backings clear of the stick;
    /// the labels fit their backings; the plot and the rows fit the
    /// console; the think ring clears the stick.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let coast = model::coast();
        let s = first_slide();
        let count = bounds(COUNT_BACK);
        let label = bounds(LABEL_BACK);
        let console = bounds(CONSOLE);
        let tested = tilted_to(TEST_SHADOW);
        let tilt: Vec<f64> = (0..=600)
            .map(|i| ACROSS + (TILTED - ACROSS) * f64::from(i) / 600.0)
            .collect();
        let test: Vec<f64> = (0..=200)
            .map(|i| TILTED + (tested - TILTED) * f64::from(i) / 200.0)
            .collect();
        for &theta in tilt.iter().chain(&test) {
            for e in ends(HOME, theta) {
                assert!(outside(card_point(e), count) > 0.15, "{e:?} at {theta}");
            }
        }
        for &theta in &test {
            for e in ends(HOME, theta) {
                assert!(outside(card_point(e), label) > 0.15, "{e:?} at {theta}");
            }
        }
        // the console: under every direction's lower end, under the coast,
        // across every line's whole width, down to the card's bottom
        for i in 0..=1800 {
            let theta = PI * f64::from(i) / 1800.0;
            for e in ends(HOME, theta) {
                assert!(card_point(e)[1] > console[3] + 0.3, "{e:?}");
            }
        }
        let n = (coast.length / 0.02).ceil() as usize;
        let lowest = (0..=n)
            .map(|i| card_point(coast.at(coast.length * i as f64 / n as f64))[1])
            .fold(f64::INFINITY, f64::min);
        assert!(lowest > console[3] + 0.15, "{lowest}");
        let line_end = card_point([CARD_W / 2.0 - sheet::LINE_INSET, 0.0])[0];
        assert!(console[0] < -line_end && console[1] > line_end);
        assert!(console[2] <= CARD_CENTRE[1] - CARD_UNIT * CARD_H / 2.0);
        // the angle's words, "30°" (at most 0.45 wide, 0.3 high), inside the
        // wedge between the reference and the stick
        let (r, deg) = ANGLE_LABEL;
        let at = [
            HOME[0] + r * deg.to_radians().cos(),
            HOME[1] + r * deg.to_radians().sin(),
        ];
        let (hw, hh) = (0.45 / 2.0 / CARD_UNIT, half_height(LABEL_SIZE) / CARD_UNIT);
        for x in [at[0] - hw, at[0] + hw] {
            let stick_y = HOME[1] + (x - HOME[0]) * TILTED.tan();
            assert!(stick_y - (at[1] + hh) > 0.15 && at[1] - hh - HOME[1] > 0.15, "{x}");
        }
        assert!(at[0] - hw > HOME[0] - ANGLE_REF && at[0] + hw < HOME[0] - ANGLE_R);
        // the pieces' words: inside their pieces, clear of the cut, of the
        // lines at rest and of the light from the upper end; their
        // backings clear of the stick and of the ruler
        let [[lo, mid], [_, hi]] = pieces(TILTED);
        let hh = half_height(PIECE_SIZE) / CARD_UNIT;
        let [p, q] = ends(HOME, TILTED).map(card_point);
        for (i, (a, b)) in [(lo, mid), (mid, hi)].into_iter().enumerate() {
            let y = PIECE_Y[i];
            assert!(y - hh > a + 0.15 && y + hh < b - 0.15, "{i}");
            assert!(off_lines(y, s) - hh > 0.15, "{i}");
            let back = bounds(piece_back(i));
            assert!(back[0] > card_point([RULER_X + RULER_TICK, 0.0])[0] + 0.05);
            assert!(back[1] > piece_at(i)[0] + width("2 cm", PIECE_SIZE) + 0.05);
            assert!(back[2] < piece_at(i)[1] - half_height(PIECE_SIZE));
            assert!(back[3] > piece_at(i)[1] + half_height(PIECE_SIZE));
            for k in 0..=100 {
                let t = f64::from(k) / 100.0;
                let on = v(p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1]));
                assert!(outside(on, back) > 0.18, "{i} {on:?}");
            }
        }
        // the labels in their backings
        let at = card_point(SHADOW_AT);
        assert!(at[0] - 0.05 > label[0] && at[0] + width("shadow 4.5 cm", LABEL_SIZE) < label[1]);
        assert!(at[1] - half_height(LABEL_SIZE) > label[2] && at[1] + half_height(LABEL_SIZE) < label[3]);
        assert!(label[0] > card_point([RULER_X + RULER_TICK, 0.0])[0] + 0.05);
        assert!(COUNT_RIGHT[0] - 0.2 > count[0] && COUNT_RIGHT[0] + WORD_GAP + width(CROSSINGS, COUNT_SIZE) < count[1]);
        assert!(
            COUNT_RIGHT[1] - half_height(COUNT_SIZE) > count[2] && COUNT_RIGHT[1] + half_height(COUNT_SIZE) < count[3]
        );
        // backings over the sheet at rest hide no line but the console's
        for b in [count, label] {
            for h in sheet::lines_on_card(s) {
                let y = CARD_CENTRE[1] + CARD_UNIT * h;
                assert!(!(b[2]..=b[3]).contains(&y), "{h}");
            }
        }
        // the plot and its words on the console, the averages' words left
        // of the rows, the rows inside the console
        let pad = 0.05;
        assert!(PLOT_NUMBER_X - 0.1 > console[0] + pad);
        assert!(PLOT_LABEL_Y - half_height(PLOT_SIZE) > console[2] + pad);
        assert!(plot_y(PLOT_TOP as f64) + PLOT_TICK < console[3] - pad);
        assert!(MEAN_X + width("average 2.25", MEAN_SIZE) < USES_AT[0][0] - 0.1);
        assert!(RULE_AT[1] + half_height(ROW_SIZE) < console[3] - pad);
        assert!(USES_AT[1][1] - half_height(ROW_SIZE) > console[2] + pad);
        // the rule (40 characters by the estimate) and the across row
        assert!(RULE_AT[0] + 40.0 * ROW_SIZE / 150.0 < console[1]);
        assert!(USES_AT[2][0] + 18.0 * ROW_SIZE / 150.0 < console[1]);
        const {
            assert!(MEAN_X > PLOT_X[1] + 0.1);
            assert!(USES_AT[0][0] + 12.0 * ROW_SIZE / 150.0 < USES_AT[2][0] - 0.2);
        }
        // the think ring and its note, from both still tilts
        for theta in [TILTED, tested] {
            for e in ends(HOME, theta) {
                let f = card_point(e);
                assert!((f[0] - THINK_AT[0]).hypot(f[1] - THINK_AT[1]) > THINK_R + 0.5);
            }
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("08_1", "Now tilt the stick"),
            ("08_1", "thirty degrees from the lines"),
            ("08_1", "shrinks to three centimetres"),
            ("08_2", "slide the sheet up through one whole gap"),
            ("08_2", "Will the count stay at two"),
            ("08_3", "half of them at one crossing"),
            ("08_3", "and half at two"),
            ("08_3", "one and a half lines"),
            ("08_4", "Cut the shadow"),
            ("08_4", "one whole gap, two centimetres"),
            ("08_4", "the one centimetre left over"),
            ("08_4", "watch where the lines meet the shadow"),
            ("08_5", "The whole gap always held one line"),
            ("08_5", "The leftover centimetre"),
            ("08_5", "One, plus a half"),
            ("08_6", "divided by the gap, two"),
            ("08_6", "That works for any shadow"),
            ("08_6", "the average count over every slide"),
            ("08_7", "Turn the stick part of the way back"),
            ("08_7", "four and a half centimetres"),
            ("08_7", "What should its average count be now"),
            ("08_8", "Two and a quarter"),
            ("08_8", "three lines for a quarter of the slide"),
            ("08_8", "four and a half divided by two"),
            ("08_9", "six over two is three"),
            ("08_9", "zero over two is zero"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
