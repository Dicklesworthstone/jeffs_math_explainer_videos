//! Chapter 08, does it settle: an endless sum of matrices has an answer only
//! if the running total settles in every entry, which is what converges
//! means (08_1); a worrying case, e to five J, whose terms swell past
//! twenty-six before the sum lands five radians round the circle (08_2); the
//! quiz, what the terms do from there (08_3); they collapse, the factorial
//! wins, and the total spirals home (08_4); and why it holds for every square
//! matrix, the k-th term at most the size to the k over k factorial (08_5).
//!
//! Every number is a fact: the running total of e to the J ticks through
//! `series` and stops on `F-V.07_6.at_one`; the sizes of the terms of 5J are
//! `F-S.terms` (`F-V.08_2.bars` as spoken), the term at k = 20 `F-S.term.k20`,
//! where the sum lands `F-V.08_4.home`, the ceilings of the second matrix
//! `F-S.strict.*`, and the bound `F-V.08_5.bound`.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Frame2, Kit, Palette, fade_all, play, rect, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::a_matrix_in::spiral_corners;
use super::draw::{
    Plane, QuizCard, arrow, corners as quad_run, fact, fade_in_all, fade_in_lagged, fade_out_all, faded_line, flat,
    good, minus, music_hold, numbers, result, rule, say, substance, tween_linear,
};
use super::flow::LiveMatrix;
use crate::model::{Mat, quarter_turn, series};

pub struct DoesItSettle {
    pub kit: Kit,
}

/// The plane of the spiral and the inset, at the left.
const LEFT_AT: Vec3 = [-3.6, -0.3, 0.0];
/// How far from the centre the spiral may swell on screen.
const REACH: f64 = 2.4;
/// The terms of e^{tJ} the spiral is drawn from.
const TERMS: u32 = 30;
/// The bar chart at the right: bars of the terms' sizes, k across.
const BARS: Frame2 = Frame2 {
    center: [3.3, -0.2, 0.0],
    width: 6.6,
    height: 4.2,
    x: (-0.5, 20.5),
    y: (0.0, 28.0),
};
/// The quiz card, over the spiral.
const CARD_AT: Vec3 = [-3.6, 0.1, 0.0];
const CARD_SIZE: (f64, f64) = (6.2, 4.8);
/// The inset's side (scene units) and how much closer it looks.
const INSET_SIDE: f64 = 4.2;
const ZOOM: f64 = 15.0;
/// How many bars the chart shows in 08_5 (k from 0 to this).
const CEILINGS: usize = 12;
/// The second matrix of 08_5 (three times a shear) and its ellipse's scale.
const ELLIPSE_UNIT: f64 = 0.42;

// the chapter's fixed words (numbers are built from facts)
const RUNNING: &str = "the running total, term by term";
const TERMS_ADDED: &str = "terms added:";
const CONVERGES: &str = "the series converges";
const NEXT_TERM: &str = "\u{d7} 5 \u{f7} k";
const RATIO_IS: &str = "5 \u{f7} k =";
const YOUR_TURN_LINES: [&str; 3] = [
    "each new term",
    "multiplies by 5 again",
    "and divides by the next whole number",
];
const SHRINK: &str = "they shrink";
const SIZE: &str = "size";
const TOUCHES: &str = "5J: every bar touches its ceiling";
const LOWER: &str = "a second matrix: its bars sit lower";
const ZOOMED: &str = "15 times closer";
const PRECISELY_NORM: &str = "the operator norm, \u{2016}AB\u{2016} \u{2264} \u{2016}A\u{2016} \u{2016}B\u{2016}";
const PROGRAMMERS: &str = "for programmers:";
const PROOF: &str = "np.linalg.norm(A, 2); the proof: \u{2016}A^k/k!\u{2016} \u{2264} \u{2016}A\u{2016}^k/k!, so the tail is smaller than the tail of e^\u{2016}A\u{2016}";

/// The sizes of the terms of e^{5J}: the facts' own list, k from 0.
fn term_sizes() -> fmn::Result<Vec<f64>> {
    numbers("F-S.terms")
}

/// A bar of the chart at `k`, `height` high (data units), flat in `style`.
fn bar(stage: &mut Stage<'_>, frame: &Frame2, k: usize, height: f64, style: Style) -> fmn::Result<Mob> {
    let (left, right) = (frame.p(k as f64 - 0.4, 0.0), frame.p(k as f64 + 0.4, 0.0));
    let h = frame.p(0.0, height)[1] - frame.p(0.0, 0.0)[1];
    let r = rect(v(left[0], left[1]), right[0] - left[0], h, style)?;
    Ok(stage.arena_mut().add(r))
}

/// How far from the centre the spiral swells: the largest distance of any
/// partial sum (never less than 1, the circle it is heading for).
fn farthest(corners: &[[f64; 2]]) -> f64 {
    corners.iter().map(|c| c[0].hypot(c[1])).fold(1.0, f64::max)
}

impl SceneConstruct for DoesItSettle {
    fn name(&self) -> &str {
        "08_does_it_settle"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let mk_line = |stage: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, width: f64, o: f64| {
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(color, width, o)).build())
        };

        // 08_1: an endless sum of matrices: the running total has to settle,
        // in every entry
        let line = say(&kit, stage, "08_1")?;
        let totals: Vec<[f64; 4]> = series(&quarter_turn(), 30)
            .iter()
            .map(|s| {
                [
                    s.total.get(0, 0),
                    s.total.get(0, 1),
                    s.total.get(1, 0),
                    s.total.get(1, 1),
                ]
            })
            .collect();
        let k = stage.add_value_tracker(0.0);
        let entry = move |i: usize, c: usize| -> Box<dyn Fn(&Arena) -> f64> {
            let totals = totals.clone();
            Box::new(move |s: &Arena| {
                let at = s.tracker_value(k).unwrap_or(0.0).round().max(0.0) as usize;
                totals[at.min(30)][i * 2 + c]
            })
        };
        let heading = kit.text(stage, RUNNING, 34.0, Palette::dim(), v(0.0, 2.7))?;
        let name = kit.tex(stage, "e^{J} =", 80.0, result(), &[], v(-3.2, 0.4))?;
        let converges = kit.text(stage, CONVERGES, 40.0, rule(), v(0.0, -2.8))?;
        let added = kit.text(stage, TERMS_ADDED, 30.0, Palette::dim(), v(-0.6, -1.6))?;
        fade_in_all(stage, &[heading, name], 0.6)?;
        let live = LiveMatrix::build(&kit, stage, v(0.4, 0.4), 2, 2, (2.4, 1.1), 76.0, 3, result(), entry)?;
        let count = kit.live_value(
            stage,
            move |s| s.tracker_value(k).unwrap_or(0.0).round(),
            0,
            false,
            48.0,
            ink,
            v(0.4, -1.6),
            LEFT,
        )?;
        stage.remove_from_scene(count);
        let mut ins = vec![added, count, live.brackets];
        ins.extend(&live.cells);
        line.wait_for(stage, "the running total has to settle")?;
        fade_in_all(stage, &ins, 0.6)?;
        play!(stage; tween_linear(k, 12.0, 4.2)?);
        line.wait_for(stage, "Mathematicians say")?;
        fade_in_all(stage, &[converges], 0.5)?;
        kit.hold(stage)?;

        // 08_2: a worrying case: e to five J: the spiral swells past
        // twenty, then its steps stand up as bars
        let line = say(&kit, stage, "08_2")?;
        let mut leaving = vec![heading, name, converges, added, count, live.brackets];
        leaving.extend(&live.cells);
        fade_out_all(stage, &leaving, 0.5)?;
        let t = stage.add_value_tracker(1.0);
        let unit_ring = mk_line(
            stage,
            vec![LEFT_AT, v(LEFT_AT[0] + 1e-4, LEFT_AT[1])],
            Palette::dim(),
            3.0,
            1.0,
        );
        let path = mk_line(stage, vec![LEFT_AT, v(LEFT_AT[0] + 1e-4, LEFT_AT[1])], rule(), 4.0, 1.0);
        let dot_template: Vec<Vec3> = Dot::new().radius(0.1).build().points().to_vec();
        let lands = stage
            .arena_mut()
            .add(Dot::new().point(LEFT_AT).radius(0.1).style(flat(result(), 1.0)).build());
        let drawer = stage.arena_mut().add(VMobject::new());
        let mut drawn = f64::NAN;
        stage.add_updater(
            drawer,
            move |s, _| {
                let time = s.tracker_value(t).unwrap_or(1.0);
                if time.to_bits() == drawn.to_bits() {
                    return;
                }
                // the picture keeps its reach: the unit circle shrinks as the
                // spiral swells
                let corners = spiral_corners(time, TERMS as usize);
                let far = farthest(&corners);
                let plane = Plane {
                    centre: LEFT_AT,
                    unit: REACH / far,
                };
                let ring: Vec<Vec3> = (0..=64)
                    .map(|i| {
                        let a = std::f64::consts::TAU * f64::from(i) / 64.0;
                        plane.p(a.cos(), a.sin())
                    })
                    .collect();
                let _ = s.set_points(unit_ring, &quad_run(&ring));
                let pts: Vec<Vec3> = corners.iter().map(|c| plane.p(c[0], c[1])).collect();
                let _ = s.set_points(path, &quad_run(&pts));
                let at = plane.p(time.cos(), time.sin());
                let moved: Vec<Vec3> = dot_template
                    .iter()
                    .map(|d| [d[0] + at[0], d[1] + at[1], d[2]])
                    .collect();
                let _ = s.set_points(lands, &moved);
                drawn = time;
            },
            true,
        )?;
        stage.add_to_scene(drawer)?;
        let five_j = kit.tex(stage, "e^{5J}", 90.0, result(), &[], v(3.3, 3.0))?;
        // the bars of the first six terms, with their sizes under the voice's
        // "twenty-six"
        let sizes = term_sizes()?;
        let said = fact("F-V.08_2.bars")?;
        let mut axis = vec![mk_line(
            stage,
            vec![BARS.p(-0.5, 0.0), BARS.p(20.5, 0.0)],
            Palette::dim(),
            3.0,
            1.0,
        )];
        let mut bars: Vec<Mob> = Vec::new();
        for (i, &size) in sizes.iter().enumerate() {
            bars.push(bar(stage, &BARS, i, size, flat(rule(), 0.95))?);
        }
        let mut under = Vec::new();
        for i in (0..=20).step_by(5) {
            let at = BARS.p(i as f64, 0.0);
            under.push(kit.text(stage, &i.to_string(), 24.0, Palette::dim(), v(at[0], at[1] - 0.3))?);
        }
        let k_name = kit.text(
            stage,
            "k",
            28.0,
            Palette::dim(),
            v(BARS.right() + 0.05, BARS.p(0.0, 0.0)[1] - 0.3),
        )?;
        let mut values = Vec::new();
        for (i, s) in said.split(',').enumerate() {
            // neighbouring bars are closer than their numbers are wide: every
            // other number stands a line higher
            let raise = if i % 2 == 1 { 0.65 } else { 0.3 };
            let at = BARS.p(i as f64, sizes[i]);
            // a hair left of the bar's middle: the bars rise to the right, and
            // a number as wide as the pitch ran into the next, taller bar
            // ("12.5" and "20.8" in the narrated draft's 08_3 tile)
            values.push(kit.text(stage, s, 24.0, ink, v(at[0] - 0.1, at[1] + raise))?);
        }
        line.wait_for(stage, "Take a worrying case")?;
        fade_in_all(stage, &[unit_ring, path, lands], 0.6)?;
        line.wait_for(stage, "E to five J")?;
        fade_in_all(stage, &[five_j], 0.5)?;
        play!(stage; tween(t, 5.0, 3.6)?);
        line.wait_for(stage, "yet two of its terms")?;
        axis.extend(&under);
        axis.push(k_name);
        axis.extend(&bars[..6]);
        axis.extend(&values);
        // the steps stand up one after another
        fade_in_lagged(stage, &axis, 1.6, 0.18)?;
        kit.hold(stage)?;

        // 08_3: your turn: each new term multiplies by five again and
        // divides by the next whole number
        let line = say(&kit, stage, "08_3")?;
        let card = QuizCard::build(&kit, stage, "08_3", CARD_AT, CARD_SIZE)?;
        let (centre, _, _) = card.content();
        let mut said_lines = Vec::new();
        for (i, words) in YOUR_TURN_LINES.iter().enumerate() {
            said_lines.push(kit.text(stage, words, 32.0, ink, v(centre[0], centre[1] + 0.8 - 0.65 * i as f64))?);
        }
        let next_gap = BARS.p(6.0, 0.0);
        let ratio = kit.text(stage, NEXT_TERM, 34.0, result(), v(next_gap[0], next_gap[1] - 0.85))?;
        let mut frame_in = said_lines.clone();
        frame_in.push(ratio);
        card.show_frame(stage, &frame_in, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 08_4: they collapse: the factorial wins, and the total spirals home
        let line = say(&kit, stage, "08_4")?;
        let verdict = match fact("F-Q.08_3.answer")? {
            "they shrink: true" => SHRINK,
            other => {
                return Err(super::draw::fail(format!(
                    "08_3's answer is {other:?}, not 'they shrink: true'"
                )));
            }
        };
        let answer = kit.text(stage, verdict, 40.0, result(), v(card.at[0], card.ring_at()[1]))?;
        card.land(stage, &[answer], 0.3)?;
        // the bars from six on appear as the ratio drops below one
        let progress = stage.add_value_tracker(5.0);
        let live_ratio = kit.live_value(
            stage,
            move |s| 5.0 / s.tracker_value(progress).unwrap_or(5.0).round().max(1.0),
            2,
            false,
            38.0,
            result(),
            v(next_gap[0] + 0.7, next_gap[1] - 0.85),
            LEFT,
        )?;
        stage.remove_from_scene(live_ratio);
        let ratio_is = kit.text(stage, RATIO_IS, 30.0, ink, v(next_gap[0] - 0.4, next_gap[1] - 0.85))?;
        let rest: Vec<Mob> = bars[6..].to_vec();
        let end_label = kit.text(
            stage,
            &format!("k = 20: {}", fact("F-S.term.k20")?),
            28.0,
            good(),
            v(BARS.p(17.0, 0.0)[0], BARS.p(0.0, 0.0)[1] + 1.25),
        )?;
        let mut card_gone = card.mobs();
        card_gone.extend(&said_lines);
        card_gone.push(answer);
        fade_out_all(stage, &card_gone, 0.5)?;
        fade_in_all(stage, &[ratio_is, live_ratio], 0.4)?;
        fade_out_all(stage, &[ratio], 0.2)?;
        line.wait_for(stage, "Once you're dividing by six")?;
        // bars k = 6 .. 20 in one lagged reveal, the ratio running with them
        let reveal = rest.clone();
        play!(stage; tween_linear(progress, 20.0, 3.2)?);
        fade_in_lagged(stage, &reveal, 0.2, 0.1)?;
        line.wait_for(stage, "By k equals twenty")?;
        fade_in_all(stage, &[end_label], 0.5)?;
        // the spiral spirals home: an inset, fifteen times closer
        line.wait_for(stage, "the running total spirals home")?;
        fade_out_all(stage, &[unit_ring, path, lands, five_j], 0.4)?;
        let home = numbers("F-V.08_4.home")?;
        let end = [home[0], home[1]];
        let corners = spiral_corners(5.0, TERMS as usize);
        let scale_main = REACH;
        let inset = Plane {
            centre: LEFT_AT,
            unit: scale_main * ZOOM,
        };
        // the inset shows the plane within half a side of the landing point
        let shifted = |c: [f64; 2]| inset.p(c[0] - end[0], c[1] - end[1]);
        let half = INSET_SIDE / 2.0;
        let inside = |p: Vec3| (p[0] - LEFT_AT[0]).abs() <= half && (p[1] - LEFT_AT[1]).abs() <= half;
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(INSET_SIDE)
                .height(INSET_SIDE)
                .corner_radius(0.1)
                .style(faded_line(Palette::dim(), 3.0, 1.0))
                .build()?
                .moved_to(LEFT_AT),
        );
        let near: Vec<Vec3> = corners.iter().map(|c| shifted(*c)).filter(|p| inside(*p)).collect();
        let angle = end[1].atan2(end[0]);
        let circle: Vec<Vec3> = (-40..=40)
            .map(|i| {
                let a = angle + f64::from(i) * 0.0021;
                shifted([a.cos(), a.sin()])
            })
            .filter(|p| inside(*p))
            .collect();
        let inset_path = mk_line(stage, near.clone(), rule(), 4.0, 1.0);
        let inset_circle = mk_line(stage, circle, result(), 6.0, 0.8);
        let landed = stage
            .arena_mut()
            .add(Dot::new().point(LEFT_AT).radius(0.12).style(flat(good(), 1.0)).build());
        let zoomed = kit.text(
            stage,
            ZOOMED,
            30.0,
            Palette::dim(),
            v(LEFT_AT[0], LEFT_AT[1] + half + 0.35),
        )?;
        let home_words = kit.text(
            stage,
            &minus(&format!("lands at {}", fact("F-V.08_4.home")?.replace(',', ", "))),
            30.0,
            good(),
            v(LEFT_AT[0], LEFT_AT[1] - half - 0.35),
        )?;
        fade_in_all(
            stage,
            &[frame, inset_circle, inset_path, landed, zoomed, home_words],
            0.7,
        )?;
        kit.hold(stage)?;

        // 08_5: it works for every square matrix: size, ceilings, and the
        // factorial always wins
        let line = say(&kit, stage, "08_5")?;
        let mut leaving = vec![
            frame,
            inset_circle,
            inset_path,
            landed,
            zoomed,
            home_words,
            ratio_is,
            live_ratio,
            end_label,
        ];
        // (the first six bars left with the axis; the rest were revealed alone)
        leaving.extend(&axis);
        leaving.extend(&bars[6..]);
        fade_out_all(stage, &leaving, 0.6)?;
        // a matrix takes the unit circle to an ellipse; its longest arrow is
        // its size
        let strict = Mat::from_rows(&[&[3.0, 3.0], &[0.0, 3.0]]);
        let ellipse_plane = Plane {
            centre: LEFT_AT,
            unit: ELLIPSE_UNIT,
        };
        let circle_pts: Vec<Vec3> = (0..=96)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / 96.0;
                ellipse_plane.p(a.cos(), a.sin())
            })
            .collect();
        let ellipse_pts: Vec<Vec3> = (0..=96)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / 96.0;
                let q = strict.apply(&[a.cos(), a.sin()]);
                ellipse_plane.p(q[0], q[1])
            })
            .collect();
        let unit_circle = mk_line(stage, circle_pts, Palette::dim(), 3.0, 1.0);
        let ellipse = mk_line(stage, ellipse_pts, rule(), 5.0, 1.0);
        let (long, _) = longest_stretch(&strict);
        let size_arrow = arrow(stage, LEFT_AT, ellipse_plane.p(long[0], long[1]), result(), 5.0)?;
        let size_name = kit.text(
            stage,
            &format!("{SIZE} {}", fact("F-S.strict.size")?),
            30.0,
            result(),
            v(LEFT_AT[0] + 1.2, LEFT_AT[1] + 2.1),
        )?;
        // ceilings size^k / k!, as outlines, and the bars of each matrix
        let ceilings = numbers("F-S.strict.ceilings")?;
        let strict_terms = numbers("F-S.strict.terms")?;
        let five_terms: Vec<f64> = term_sizes()?.into_iter().take(CEILINGS + 1).collect();
        let outline = |c: Srgb| faded_line(c, 3.0, 1.0);
        let mut ceiling_five = Vec::new();
        let mut filled_five = Vec::new();
        let mut ceiling_strict = Vec::new();
        let mut filled_strict = Vec::new();
        for i in 0..=CEILINGS {
            ceiling_five.push(bar(stage, &BARS, i, five_terms[i], outline(Palette::ink()))?);
            filled_five.push(bar(stage, &BARS, i, five_terms[i], flat(rule(), 0.95))?);
            ceiling_strict.push(bar(stage, &BARS, i, ceilings[i], outline(Palette::ink()))?);
            filled_strict.push(bar(stage, &BARS, i, strict_terms[i], flat(substance(), 0.95))?);
        }
        let floor = mk_line(
            stage,
            vec![BARS.p(-0.5, 0.0), BARS.p(12.5, 0.0)],
            Palette::dim(),
            3.0,
            1.0,
        );
        let touches = kit.text(stage, TOUCHES, 28.0, rule(), v(3.3, 2.55))?;
        let lower = kit.text(stage, LOWER, 28.0, substance(), v(3.3, 2.55))?;
        let bound = kit.tex(
            stage,
            &format!(
                "\\sum_k \\frac{{5^{{k}}}}{{k!}} = e^{{5}} = {}",
                fact("F-V.08_5.bound")?
            ),
            56.0,
            good(),
            &[],
            v(3.3, 1.7),
        )?;
        let norm = precisely(&kit, stage, PRECISELY_NORM)?;
        let coder = lower_third(&kit, stage, PROGRAMMERS, PROOF)?;
        // "It works for every square matrix": the circle and what a matrix
        // makes of it are on the stage as that is said (the narrated draft's
        // tile was blank for the first three seconds)
        fade_in_all(stage, &[unit_circle, ellipse], 0.7)?;
        line.wait_for(stage, "Call the most a matrix can stretch")?;
        fade_in_all(stage, &[size_arrow, size_name], 0.6)?;
        fade_in_all(stage, &[norm], 0.5)?;
        line.wait_for(stage, "The k-th term is at most")?;
        let mut first = ceiling_five.clone();
        first.push(floor);
        fade_in_all(stage, &first, 0.6)?;
        let mut filled = filled_five.clone();
        filled.push(touches);
        fade_in_all(stage, &filled, 0.6)?;
        line.wait_for(stage, "and the factorial always wins")?;
        let mut swap = filled_five.clone();
        swap.extend(&ceiling_five);
        swap.push(touches);
        fade_out_all(stage, &swap, 0.4)?;
        fade_out_all(stage, &[norm], 0.3)?;
        let mut second = ceiling_strict.clone();
        second.extend(&filled_strict);
        second.extend([lower, coder]);
        fade_in_all(stage, &second, 0.7)?;
        line.wait_for(stage, "So what's still to come")?;
        fade_in_all(stage, &[bound], 0.6)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// The point of the unit circle the matrix stretches most, mapped: where
/// its image is, and how long (the matrix's size, to the sampling).
fn longest_stretch(m: &Mat) -> ([f64; 2], f64) {
    let mut best = ([0.0, 0.0], 0.0_f64);
    for i in 0..3600 {
        let a = std::f64::consts::TAU * f64::from(i) / 3600.0;
        let q = m.apply(&[a.cos(), a.sin()]);
        let len = q[0].hypot(q[1]);
        if len > best.1 {
            best = ([q[0], q[1]], len);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::super::draw::{quiz_at, show, testing, whole};
    use super::*;
    use crate::model::{expm_t, factorial_f64};

    /// 08_1: the running total of e to the J ticks and stops on the matrix
    /// the facts print for t = 1; 08_2 and 08_4: the terms of 5J are 5^k / k!
    /// in size, two of them near twenty-six, and the sum lands where the
    /// facts say.
    #[test]
    fn the_terms_of_five_j_swell_and_collapse_as_the_facts_say() {
        let steps = series(&quarter_turn(), 12);
        let total = &steps[steps.len() - 1].total;
        for (i, row) in super::super::draw::entries("F-V.07_6.at_one", Some(3))
            .unwrap()
            .iter()
            .enumerate()
        {
            for (c, s) in row.iter().enumerate() {
                assert_eq!(&show(total.get(i, c), 3).unwrap(), s, "entry {i},{c}");
            }
        }
        let sizes = term_sizes().unwrap();
        assert_eq!(sizes.len(), 21);
        for (k, s) in sizes.iter().enumerate() {
            let want = 5.0_f64.powi(k as i32) / factorial_f64(k as u32);
            assert!((s - want).abs() < 1e-5 * want.max(1.0), "k = {k}");
        }
        // two terms near twenty-six, and the bars as the voice says them
        assert_eq!(fact("F-V.08_2.bars").unwrap(), "1.0,5.0,12.5,20.8,26.0,26.0");
        assert_eq!(whole(sizes[4]), "26");
        assert_eq!(whole(sizes[5]), "26");
        assert_eq!(fact("F-V.08_2.near").unwrap(), "26");
        // swells past twenty-one, and a factorial later is forty millionths
        let reach = farthest(&spiral_corners(5.0, TERMS as usize));
        assert!(reach > 21.0 && reach < 22.0, "{reach}");
        assert_eq!(whole(reach.floor()), fact("F-V.08_2.swings_past").unwrap());
        let k20 = 5.0_f64.powi(20) / factorial_f64(20);
        assert!(k20 < 4.0e-5 && k20 > 3.9e-5, "{k20}");
        assert_eq!(fact("F-S.term.k20").unwrap(), show(k20, 7).unwrap());
        assert_eq!(fact("F-S.term.k20").unwrap(), "0.0000392");
        // from k = 6 on every term is smaller than the one before: the
        // quiz's answer, and the fact that holds it
        assert!(sizes[5..].windows(2).all(|w| w[1] < w[0]));
        assert_eq!(fact("F-Q.08_3.answer").unwrap(), "they shrink: true");
        assert_eq!(quiz_at("08_3").unwrap().seconds, 4);
        // the ratio of the voice: each term is 5 / k times the last, and
        // it drops below one at k = 6, from where every term shrinks
        for k in 1..=20_u32 {
            let (now, before) = (
                5.0_f64.powi(k as i32) / factorial_f64(k),
                5.0_f64.powi(k as i32 - 1) / factorial_f64(k - 1),
            );
            assert!((now / before - 5.0 / f64::from(k)).abs() < 1e-12, "k = {k}");
            assert_eq!(now < before, k > 5, "k = {k}");
        }
        // where the sum lands
        let home = expm_t(&quarter_turn(), 5.0).column(0);
        assert_eq!(
            fact("F-V.08_4.home").unwrap(),
            format!("{},{}", show(home[0], 3).unwrap(), show(home[1], 3).unwrap())
        );
    }

    /// 08_5: the k-th term is at most size^k / k!, for J scaled and for the
    /// second matrix; 5J touches its ceiling and the second sits strictly
    /// under from k = 2; the ceilings add to e to the size.
    #[test]
    fn the_terms_sit_under_their_ceilings_and_the_ceilings_add_to_e_to_the_size() {
        let (strict, five_j) = (Mat::from_rows(&[&[3.0, 3.0], &[0.0, 3.0]]), quarter_turn().scale(5.0));
        for (name, m) in [("strict", &strict), ("5J", &five_j)] {
            let size = m.size();
            for (k, step) in series(m, 12).iter().enumerate() {
                let ceiling = size.powi(k as i32) / factorial_f64(k as u32);
                assert!(step.term.size() <= ceiling * (1.0 + 1e-9), "{name}, k = {k}");
                if name == "5J" {
                    assert!(
                        (step.term.size() - ceiling).abs() < 1e-9 * ceiling.max(1.0),
                        "5J touches, k = {k}"
                    );
                } else if k >= 2 {
                    assert!(step.term.size() < ceiling * 0.999, "strict is under, k = {k}");
                }
            }
        }
        assert_eq!(fact("F-S.strict.below_from_k2").unwrap(), "true");
        assert_eq!(fact("F-S.strict.size").unwrap(), show(strict.size(), 3).unwrap());
        // the picture's longest arrow is the matrix's size
        let (_, long) = longest_stretch(&strict);
        assert!((long - strict.size()).abs() < 1e-5, "{long} {}", strict.size());
        // the ceilings of 5J add to e^5
        let sum: f64 = (0..=40).map(|k| 5.0_f64.powi(k) / factorial_f64(k as u32)).sum();
        assert_eq!(show(sum, 1).unwrap(), fact("F-V.08_5.bound").unwrap());
        assert_eq!(fact("F-V.08_5.bound").unwrap(), "148.4");
    }

    /// 08_4: the inset's window holds the last steps and the circle: the
    /// partial sums from about k = 14 are inside it, fifteen times closer.
    #[test]
    fn the_inset_holds_the_last_steps_of_the_spiral() {
        let end = numbers("F-V.08_4.home").unwrap();
        let corners = spiral_corners(5.0, TERMS as usize);
        let window = INSET_SIDE / 2.0 / (REACH * ZOOM);
        let inside = |c: &[f64; 2]| (c[0] - end[0]).abs() <= window && (c[1] - end[1]).abs() <= window;
        let first_inside = corners.iter().position(inside).unwrap();
        assert!(
            (14..=18).contains(&first_inside),
            "the first corner inside is {first_inside}"
        );
        assert!(corners[first_inside..].iter().all(inside), "and it stays inside");
        let last = corners[corners.len() - 1];
        assert!((last[0].hypot(last[1]) - 1.0).abs() < 1e-9, "home is on the circle");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("08_does_it_settle").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "e^{5J}",
            "the series converges",
            "they shrink",
            "k = 20: 0.0000392",
            "15 times closer",
            "e^{5} = 148.4",
            "size 4.8",
            "26.0",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
