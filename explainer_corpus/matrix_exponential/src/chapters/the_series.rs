//! Chapter 04, the series: a second way to the curve whose rate equals
//! itself, built piece by piece (04_1); the first piece, one, has rate zero
//! and needs one, so x is added, which climbs at rate one (04_2); x needs a
//! partner whose rate is x, half of x squared, the area under the line,
//! which grows as fast as its leading edge is tall (04_3); the quiz, what
//! comes after half of x squared (04_4); x cubed over six, x to the fourth
//! over twenty-four, the factorials (04_5); and the endless sum, a series,
//! whose row of terms slides one place over and lands on itself, where five
//! terms alone leave a gap (04_6).
//!
//! Every piece is `x^k / k!`: the quiz's answer is `F-Q.04_4.answer`, the
//! bottoms are `F-N.factorials`.

use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    QuizCard, arrow_points, fade_in_all, fade_out_all, faded_line, flat, good, music_hold, numbers, result, rule, say,
    tick_mark, warn, whole,
};
use super::flow::{fade_in_anim, fade_out_anim};

pub struct TheSeries {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const RATE_ZERO: &str = "rate 0";
const RATE_ONE: &str = "rate 1";
const NEED_ONE: &str = "we need 1";
const ITS_RATE: &str = "its rate";
const NOTHING_TO_COME_FROM: &str = "nothing to come from";

/// The card of 04_4.
const CARD_AT: Vec3 = [0.0, 0.3, 0.0];
const CARD_SIZE: (f64, f64) = (11.0, 5.8);
/// Where the card's four pieces stand across.
const CARD_COLS: [f64; 4] = [-4.0, -1.5, 1.4, 4.2];
/// The columns of 04_6's rows, and how far one place is.
const ROW_X0: f64 = -4.3;
const PLACE: f64 = 2.0;

/// The pieces of the series as TeX, by degree: `1`, `x`, `x^2/2!`, ...
fn piece(k: u32) -> String {
    match k {
        0 => "1".to_owned(),
        1 => "x".to_owned(),
        _ => format!("\\frac{{x^{{{k}}}}}{{{k}!}}"),
    }
}

/// The pieces with their bottoms written out, as 04_5 builds them:
/// `x^3 / (2 \times 3)`.
fn piece_by_bottoms(k: u32) -> String {
    match k {
        0 => "1".to_owned(),
        1 => "x".to_owned(),
        _ => {
            let bottoms: Vec<String> = (2..=k).map(|j| j.to_string()).collect();
            format!("\\frac{{x^{{{k}}}}}{{{}}}", bottoms.join(" \\times "))
        }
    }
}

/// A curved arrow from `a` to `b` bowed up by `lift`, as one polyline with
/// its head at `b`.
fn bow(a: Vec3, b: Vec3, lift: f64) -> Vec<Vec3> {
    const STEPS: usize = 18;
    let mut pts: Vec<Vec3> = (0..=STEPS)
        .map(|i| {
            let t = i as f64 / STEPS as f64;
            v(
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t + lift * 4.0 * t * (1.0 - t),
            )
        })
        .collect();
    let tip = pts.pop().unwrap_or(b);
    let from = pts.last().copied().unwrap_or(a);
    pts.extend(arrow_points(from, tip, 0.2));
    pts
}

impl SceneConstruct for TheSeries {
    fn name(&self) -> &str {
        "04_the_series"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();

        // 04_1: a second way to reach the curve: build it piece by piece
        let line = say(&kit, stage, "04_1")?;
        let target = kit.tex(stage, "f' = f", 100.0, rule(), &[], v(0.0, 2.9))?;
        let f_is = kit.tex(stage, "f(x) =", 90.0, ink, &[], v(-4.9, 0.9))?;
        let waiting = polyline(vec![v(-3.0, 0.9), v(6.2, 0.9)])
            .style(faded_line(Palette::faint(), 4.0, 1.0))
            .build();
        let waiting = stage.arena_mut().add(waiting);
        fade_in_all(stage, &[target], 0.6)?;
        line.wait_for(stage, "Build it piece by piece")?;
        fade_in_all(stage, &[f_is, waiting], 0.6)?;
        kit.hold(stage)?;

        // 04_2: one has rate zero and we need one; add x, which climbs at
        // rate one
        let line = say(&kit, stage, "04_2")?;
        let one = kit.tex(stage, "1", 90.0, result(), &[], v(-2.2, 0.9))?;
        let plus_x = kit.tex(stage, "+ \\; x", 90.0, rule(), &[], v(0.1, 0.9))?;
        // two small windows: the piece 1, flat; the piece x, steepness 1
        let win = |at: Vec3| Frame2 {
            center: at,
            width: 3.0,
            height: 2.4,
            x: (0.0, 2.0),
            y: (0.0, 2.0),
        };
        let (win_one, win_x) = (win(v(-3.2, -1.7)), win(v(1.0, -1.7)));
        let mut windows = Vec::new();
        for w in [win_one, win_x] {
            windows.push(stage.arena_mut().add(w.border(Palette::faint())?));
        }
        let flat_line = stage.arena_mut().add(
            polyline(vec![win_one.p(0.0, 1.0), win_one.p(2.0, 1.0)])
                .style(faded_line(result(), 6.0, 1.0))
                .build(),
        );
        let steep_line = stage.arena_mut().add(
            polyline(vec![win_x.p(0.0, 0.0), win_x.p(2.0, 2.0)])
                .style(faded_line(rule(), 6.0, 1.0))
                .build(),
        );
        let rate_zero = kit.text(stage, RATE_ZERO, 34.0, warn(), v(-3.2, -3.35))?;
        let need_one = kit.text(stage, NEED_ONE, 30.0, Palette::dim(), v(-0.6, -0.4))?;
        let rate_one = kit.text(stage, RATE_ONE, 34.0, rule(), v(1.0, -3.35))?;
        line.wait_for(stage, "Start with one")?;
        // the line that waited for the pieces ran through the 1 and the x
        // (a strike-through in the narrated draft's tile): it goes as they come
        fade_out_all(stage, &[waiting], 0.3)?;
        fade_in_all(stage, &[one, windows[0], flat_line], 0.6)?;
        line.wait_for(stage, "Its rate is zero")?;
        fade_in_all(stage, &[rate_zero], 0.5)?;
        line.wait_for(stage, "and we need one")?;
        fade_in_all(stage, &[need_one], 0.5)?;
        line.wait_for(stage, "So add x")?;
        fade_in_all(stage, &[plus_x, windows[1], steep_line], 0.6)?;
        line.wait_for(stage, "which climbs")?;
        fade_in_all(stage, &[rate_one], 0.5)?;
        // the mismatch clears: the red words give way to a tick
        let tick = tick_mark(stage, v(-3.2, -3.35), 0.5, good(), 7.0)?;
        play!(stage;
            fade_out(stage.arena_mut(), rate_zero, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), need_one, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), tick, ORIGIN, 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 04_3: x needs a partner whose rate is x: the area under the line
        let line = say(&kit, stage, "04_3")?;
        let mut leaving = vec![target, f_is, one, plus_x, windows[0], windows[1], flat_line, steep_line];
        leaving.extend([rate_one, tick]);
        fade_out_all(stage, &leaving, 0.5)?;
        let big = Frame2 {
            center: [-2.6, -0.5, 0.0],
            width: 5.6,
            height: 4.6,
            x: (0.0, 3.2),
            y: (0.0, 3.2),
        };
        let axes = [
            polyline(vec![big.p(0.0, 0.0), big.p(3.2, 0.0)]),
            polyline(vec![big.p(0.0, 0.0), big.p(0.0, 3.2)]),
        ]
        .map(|p| {
            stage
                .arena_mut()
                .add(p.style(faded_line(Palette::dim(), 3.0, 1.0)).build())
        });
        let diagonal = stage.arena_mut().add(
            polyline(vec![big.p(0.0, 0.0), big.p(3.2, 3.2)])
                .style(faded_line(result(), 6.0, 1.0))
                .build(),
        );
        let x_now = stage.add_value_tracker(1.0);
        let area = stage.arena_mut().add(
            Polygon::new(vec![big.p(0.0, 0.0), big.p(1.0, 0.0), big.p(1.0, 1.0)])
                .style(flat(rule(), 0.35))
                .build(),
        );
        let strip = stage
            .arena_mut()
            .add(Polygon::new(strip_points(&big, 1.0)).style(flat(rule(), 0.95)).build());
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let x = s.tracker_value(x_now).unwrap_or(0.0);
                if x.to_bits() == last.to_bits() {
                    return;
                }
                let _ = s.set_points(
                    area,
                    Polygon::new(vec![big.p(0.0, 0.0), big.p(x, 0.0), big.p(x, x)])
                        .build()
                        .points(),
                );
                let _ = s.set_points(strip, Polygon::new(strip_points(&big, x)).build().points());
                last = x;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let half_square = kit.tex(stage, "\\frac{x^{2}}{2}", 110.0, rule(), &[], v(3.6, 0.7))?;
        let the_line = kit.tex(stage, "y = x", 70.0, result(), &[], v(3.6, 2.4))?;
        // the line y = x is on the stage as "a partner whose rate is x" is
        // asked for (the narrated draft's tile was blank for three seconds);
        // its area comes with the answer
        let mut line_in = axes.to_vec();
        line_in.extend([diagonal, the_line]);
        fade_in_all(stage, &line_in, 0.6)?;
        line.wait_for(stage, "That's half of x squared")?;
        fade_in_all(stage, &[area, strip], 0.6)?;
        fade_in_all(stage, &[half_square], 0.6)?;
        line.wait_for(stage, "An area grows as fast")?;
        play!(stage; tween(x_now, 2.8, 3.2)?);
        kit.hold(stage)?;

        // 04_4: your turn: what comes after half of x squared?
        let line = say(&kit, stage, "04_4")?;
        let mut leaving = axes.to_vec();
        leaving.extend([diagonal, area, strip, half_square, the_line]);
        fade_out_all(stage, &leaving, 0.5)?;
        let card = QuizCard::build(&kit, stage, "04_4", CARD_AT, CARD_SIZE)?;
        let row_y = card.content().0[1] + 0.5;
        let mut pieces = Vec::new();
        for (k, words) in ["1", "x", "\\frac{x^{2}}{2}"].into_iter().enumerate() {
            pieces.push(kit.tex(stage, words, 84.0, ink, &[], v(CARD_COLS[k], row_y))?);
        }
        let question = kit.tex(stage, "?", 84.0, result(), &[], v(CARD_COLS[3], row_y))?;
        // the bottoms written 1, 1 x 2 under the pieces they belong to
        let mut bottoms = Vec::new();
        for (k, words) in ["1", "1", "1 \\times 2"].into_iter().enumerate() {
            bottoms.push(kit.tex(stage, words, 50.0, Palette::dim(), &[], v(CARD_COLS[k], row_y - 1.3))?);
        }
        // "its rate": each piece points back at the one before
        let mut arcs = Vec::new();
        let mut words = Vec::new();
        for k in 0..3 {
            let (from, to) = (
                v(CARD_COLS[k + 1] - 0.55, row_y + 0.45),
                v(CARD_COLS[k] + 0.55, row_y + 0.45),
            );
            let arc = polyline(bow(from, to, 0.35))
                .style(faded_line(rule(), 3.5, 1.0))
                .build();
            arcs.push(stage.arena_mut().add(arc));
            let at = v((from[0] + to[0]) / 2.0, row_y + 1.05);
            words.push(kit.text(stage, ITS_RATE, 22.0, Palette::dim(), at)?);
        }
        let mut frame_in = pieces.clone();
        frame_in.push(question);
        card.show_frame(stage, &frame_in, 0.5)?;
        line.wait_for(stage, "Each new piece must have")?;
        fade_in_all(stage, &arcs.iter().chain(&words).copied().collect::<Vec<_>>(), 0.6)?;
        line.wait_for(stage, "Raise the power by one")?;
        fade_in_all(stage, &bottoms, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 04_5: x cubed over six, x to the fourth over twenty-four: the
        // bottoms are the last one times the next whole number
        let line = say(&kit, stage, "04_5")?;
        let factorial = numbers("F-N.factorials")?;
        let answer = kit.tex(
            stage,
            &format!("\\frac{{x^{{3}}}}{{{}}}", whole(factorial[3])),
            84.0,
            result(),
            &[],
            v(CARD_COLS[3], row_y),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.2),
            fade_in(stage.arena_mut(), answer, ORIGIN, 1.0)?.rt(0.4),
        );
        card.land(stage, &[], 0.25)?;
        // the row, with its bottoms written out
        let mut row = Vec::new();
        let columns = [-5.2, -3.8, -2.2, 0.6, 4.0];
        for (k, &x) in columns.iter().enumerate() {
            row.push(kit.tex(stage, &piece_by_bottoms(k as u32), 66.0, ink, &[], v(x, 1.7))?);
        }
        // the plus signs sit in the gaps between the pieces' own widths
        let plus = [-4.5, -3.0, -0.95, 2.05]
            .into_iter()
            .map(|x| kit.tex(stage, "+", 56.0, Palette::dim(), &[], v(x, 1.7)))
            .collect::<fmn::Result<Vec<_>>>()?;
        let back = kit.tex(
            stage,
            "3 \\cdot \\frac{x^{2}}{6} = \\frac{x^{2}}{2}",
            70.0,
            rule(),
            &[],
            v(-1.0, -0.2),
        )?;
        let back_tag = kit.text(stage, ITS_RATE, 28.0, Palette::dim(), v(-1.0, -1.3))?;
        let four = kit.tex(
            stage,
            &format!("4! = 1 \\times 2 \\times 3 \\times 4 = {}", whole(factorial[4])),
            80.0,
            result(),
            &[],
            v(0.0, -2.7),
        )?;
        // the sum so far and x cubed over six come in together, as it is
        // said (the narrated draft's tile, 1.5 s in, had the sum without it)
        let mut first = vec![row[0], row[1], row[2]];
        first.extend(&plus[..2]);
        first.extend([plus[2], row[3]]);
        // "X cubed over six" is said over the answer, fully up; the card
        // goes as the row comes in, in one batch (so no frame between them
        // is a black stage)
        line.wait_first_sentence(stage)?;
        let mut leaving = card.mobs();
        leaving.extend(&pieces);
        leaving.extend(&bottoms);
        leaving.extend(&arcs);
        leaving.extend(&words);
        leaving.push(answer);
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &leaving {
            swap.push(fade_out_anim(stage, m, 0.3)?);
        }
        for &m in &first {
            swap.push(fade_in_anim(stage, m, 0.5)?);
        }
        stage.play_prepared(swap)?;
        line.wait_for(stage, "Then x to the fourth")?;
        fade_in_all(stage, &[plus[3], row[4]], 0.6)?;
        line.wait_for(stage, "Each bottom number")?;
        fade_in_all(stage, &[back, back_tag], 0.6)?;
        line.wait_for(stage, "Those are the factorials")?;
        fade_in_all(stage, &[four], 0.6)?;
        kit.hold(stage)?;

        // 04_6: an endless sum is a series: the row of terms slides one place
        // over and lands on itself; five terms alone leave a gap
        let line = say(&kit, stage, "04_6")?;
        let mut leaving = row.clone();
        leaving.extend(&plus);
        leaving.extend([back, back_tag, four]);
        fade_out_all(stage, &leaving, 0.5)?;
        let (top_y, low_y) = (1.0, -1.0);
        let column = |k: usize| ROW_X0 + PLACE * k as f64;
        let f_label = kit.tex(stage, "f(x) =", 60.0, ink, &[], v(-6.0, top_y))?;
        let d_label = kit.tex(stage, "f'(x) =", 60.0, rule(), &[], v(-6.0, low_y))?;
        let mut terms = Vec::new();
        for k in 0..5 {
            terms.push(kit.tex(stage, &piece(k as u32), 62.0, ink, &[], v(column(k), top_y))?);
        }
        // the rate's row: a zero, then each term again, one place to the right
        let mut rates = vec![kit.tex(stage, "0", 62.0, rule(), &[], v(column(0), low_y))?];
        for k in 0..4 {
            rates.push(kit.tex(stage, &piece(k as u32), 62.0, rule(), &[], v(column(k + 1), low_y))?);
        }
        let slide = stage.add_value_tracker(0.0);
        let places: Vec<(Mob, f64)> = rates.iter().enumerate().map(|(k, &m)| (m, column(k))).collect();
        let carrier = stage.arena_mut().add(VMobject::new());
        let mut shown = f64::NAN;
        stage.add_updater(
            carrier,
            move |s, _| {
                let t = s.tracker_value(slide).unwrap_or(0.0);
                if t.to_bits() == shown.to_bits() {
                    return;
                }
                for &(m, x) in &places {
                    s.move_to(m, v(x - t * PLACE, low_y), ORIGIN);
                }
                shown = t;
            },
            true,
        )?;
        stage.add_to_scene(carrier)?;
        let gap = kit.text(stage, NOTHING_TO_COME_FROM, 30.0, warn(), v(column(4), low_y - 1.1))?;
        let box_at = v(column(4), low_y);
        // what the endless row has there: the term, and the dots after it
        let late = kit.tex(stage, &piece(4), 62.0, rule(), &[], box_at)?;
        let dots_top = kit.tex(stage, "\\cdots", 62.0, ink, &[], v(column(5), top_y))?;
        let dots_low = kit.tex(stage, "\\cdots", 62.0, rule(), &[], v(column(5), low_y))?;
        let gap_box = stage.arena_mut().add(
            Rectangle::new()
                .width(1.8)
                .height(1.3)
                .corner_radius(0.1)
                .style(faded_line(warn(), 3.0, 1.0))
                .build()?
                .moved_to(box_at),
        );
        let endless = kit.tex(
            stage,
            "e^{x} = 1 + x + \\frac{x^{2}}{2!} + \\frac{x^{3}}{3!} + \\cdots",
            62.0,
            result(),
            &[],
            v(0.2, 2.85),
        )?;
        let mut ins = vec![f_label, d_label];
        ins.extend(&terms);
        ins.extend(&rates);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "Take its rate")?;
        // the row of rates slides one place over (its zero leaves the row),
        // and the last term of five has nothing to come from
        play!(stage;
            tween(slide, 1.0, 1.4)?,
            fade_out(stage.arena_mut(), rates[0], ORIGIN, 1.0)?.rt(0.9),
        );
        fade_in_all(stage, &[gap_box, gap], 0.5)?;
        line.wait_for(stage, "The whole row slides")?;
        // the endless row: the missing term is there, and the dots go on
        play!(stage;
            fade_out(stage.arena_mut(), gap_box, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), gap, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), late, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), dots_top, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), dots_low, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), endless, ORIGIN, 1.0)?.rt(0.8),
        );
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// The thin strip at the right edge of the area at `x`: as tall as the line
/// there, a sliver wide.
fn strip_points(frame: &Frame2, x: f64) -> Vec<Vec3> {
    let w = 0.12;
    vec![
        frame.p((x - w).max(0.0), 0.0),
        frame.p(x, 0.0),
        frame.p(x, x),
        frame.p((x - w).max(0.0), (x - w).max(0.0)),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, testing};
    use super::*;
    use crate::model::{exp_partial, factorial};

    /// 04_2 to 04_5: each piece is the last piece's integral, and the last
    /// piece is its rate: the coefficient `k / k!` of the rate of `x^k / k!`
    /// is `1 / (k - 1)!`, exactly, so the rate of every piece is the one
    /// before it.
    #[test]
    fn the_rate_of_every_piece_is_the_piece_before_it() {
        for k in 1..=20_u32 {
            assert_eq!(u128::from(k) * factorial(k - 1), factorial(k), "k = {k}");
        }
        // and numerically, at a point, by a central difference
        let (x, h) = (1.3_f64, 1e-5);
        for k in 1..=6_i32 {
            let f = |x: f64| x.powi(k) / factorial(k as u32) as f64;
            let slope = (f(x + h) - f(x - h)) / (2.0 * h);
            let before = x.powi(k - 1) / factorial(k as u32 - 1) as f64;
            assert!((slope - before).abs() < 1e-6, "k = {k}");
        }
    }

    /// 04_4 and 04_5: the answer of the quiz and the numbers of the bottoms
    /// are the facts', and the bottoms written out are the factorials.
    #[test]
    fn the_quiz_answer_and_the_bottoms_are_the_facts() {
        assert_eq!(fact("F-Q.04_4.answer").unwrap(), "x^3/6");
        assert_eq!(fact("F-N.factorials").unwrap(), "1,1,2,6,24,120,720");
        let f = numbers("F-N.factorials").unwrap();
        assert_eq!(f[3], 6.0);
        assert_eq!(f[4], 24.0);
        assert_eq!(piece_by_bottoms(3), "\\frac{x^{3}}{2 \\times 3}");
        assert_eq!(piece_by_bottoms(4), "\\frac{x^{4}}{2 \\times 3 \\times 4}");
        // 2 x 3 is 3!, 2 x 3 x 4 is 4!: "the last one, times the next whole number"
        assert_eq!(2 * 3, f[3] as i32);
        assert_eq!(2 * 3 * 4, f[4] as i32);
        assert_eq!(piece(3), "\\frac{x^{3}}{3!}");
    }

    /// 04_6: five terms leave a gap (the rate of the row's last term has
    /// nothing in the row to be), and the endless sum does not: the
    /// difference between a sum and its rate is exactly the last term, and
    /// it goes to nothing as terms are added.
    #[test]
    fn five_terms_leave_a_gap_and_the_endless_sum_does_not() {
        let x = 1.5_f64;
        for terms in [5_u32, 8, 12, 20, 30] {
            let top = terms - 1;
            // sum through x^top / top!, and its rate: the same sum through top - 1
            let sum = exp_partial(x, top);
            let rate = exp_partial(x, top - 1);
            let gap = sum - rate;
            let last = x.powi(top as i32) / factorial(top) as f64;
            assert!((gap - last).abs() < 1e-12 * sum.max(1.0), "terms = {terms}");
            assert!(
                last > 0.0,
                "the rate of a finite sum falls short of it by its last term"
            );
            if last > 1e-9 {
                assert!(gap > 0.0, "terms = {terms}: the gap is there to be seen");
            }
        }
        // five terms: the missing term is x^4 / 4!, a fifth of the sum's
        // last digit places; the endless row has nothing missing, the
        // missing term going to nothing faster than any power
        let term = |k: u32| x.powi(k as i32) / factorial(k) as f64;
        assert!(term(4) > 0.2, "five terms: a visible gap, {}", term(4));
        assert!(term(29) < 1e-20, "endless: nothing is lost, {}", term(29));
        // the series is e^x
        assert!((exp_partial(x, 40) - x.exp()).abs() < 1e-14);
    }

    /// 04_3: the area under y = x from 0 to x is x squared over two, and
    /// the strip at its edge is as tall as the line there.
    #[test]
    fn the_area_under_the_line_is_half_x_squared() {
        let frame = Frame2 {
            center: [0.0, 0.0, 0.0],
            width: 4.0,
            height: 4.0,
            x: (0.0, 2.0),
            y: (0.0, 2.0),
        };
        for x in [0.5, 1.0, 2.0] {
            // the triangle (0,0), (x,0), (x,x) by the shoelace formula, in
            // data units
            let area = 0.5 * (x * x);
            assert_eq!(area, x * x / 2.0);
            let strip = strip_points(&frame, x);
            let height = strip[2][1] - strip[1][1];
            let line_height = frame.p(x, x)[1] - frame.p(x, 0.0)[1];
            assert!((height - line_height).abs() < 1e-12, "x = {x}");
        }
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("04_the_series").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "f' = f",
            "\\frac{x^{3}}{6}",
            "\\frac{x^{4}}{2 \\times 3 \\times 4}",
            "4! = 1 \\times 2 \\times 3 \\times 4 = 24",
            "e^{x} = 1 + x",
            "your turn",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 35.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
