//! Chapter 05, one function: the series at x equals one, its running total
//! climbing to the bank's number (05_1); the staircase of compound interest
//! and the growing sum both close in on one curve, because only one curve
//! has rate equal to amount from that start (05_2); loosen the rule to a
//! fixed share, five percent a year, and the answer is the same curve
//! stretched (05_3); turn the dial and one function handles savings,
//! bacteria, decay and a cooling cup (05_4); the quiz, five percent for
//! twenty years (05_5); and its answer, e to the one (05_6).
//!
//! Every number is a fact: the running totals are `F-N.series.k*` and
//! `F-V.05_1.*`, the bank's number `F-V.03_4.e`, the share `F-V.05_3.share`,
//! the product and the factor `F-V.05_6.*`, the quiz's answer
//! `F-Q.05_5.answer`, the rate and years of the quiz `F-N.growth.*`.

use super::draw::precisely;
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    QuizCard, corners, fact, fade_in_all, fade_out_all, faded_line, flat, good, music_hold, numbers, result, rule, say,
    substance, turning, whole,
};
use crate::model::exp_partial;

pub struct OneFunction {
    pub kit: Kit,
}

/// The window of 05_2: e to the x on its own scale (x up to 2.2). Both
/// windows stand with their unit label (a window's bottom, a tick row and
/// the label's row under it) clear of the one-line lower third that comes up
/// under them at 05_2 and 05_4: at the old height "x" and "years" stood in
/// the lower third's box.
const UNIT_WINDOW: Frame2 = Frame2 {
    center: [-2.6, -0.1, 0.0],
    width: 6.4,
    height: 4.2,
    x: (0.0, 2.2),
    y: (0.0, 8.0),
};
/// The window of 05_3 onwards: years across (forty), the amount up.
const YEARS_WINDOW: Frame2 = Frame2 {
    center: [-2.6, -0.1, 0.0],
    width: 6.4,
    height: 4.2,
    x: (0.0, 40.0),
    y: (0.0, 8.0),
};
/// The dial: where it stands, how big, and the share at its ends.
const DIAL_AT: Vec3 = [-5.3, 2.35, 0.0];
const DIAL_R: f64 = 0.8;
const R_MAX: f64 = 0.1;
/// The quiz card.
const CARD_AT: Vec3 = [4.2, -0.2, 0.0];
const CARD_SIZE: (f64, f64) = (5.2, 4.8);
/// The right panel's centre.
const PANEL_X: f64 = 4.3;

// the chapter's fixed words (numbers are built from facts)
const RUNNING_TOTAL: &str = "running total";
const BANKS_NUMBER: &str = "the bank\u{2019}s number";
const STAIRCASE: &str = "steps of a year";
const PIECES: &str = "pieces of the series";
const PRECISELY_SAME: &str = "if g\u{2032} = g too, then g(x) \u{b7} f(\u{2212}x) never changes, so g = f";
const WHATEVER: &str = "times whatever you start with";
const PRECISELY_ODE: &str = "a differential equation: a rule for the rate";
const STOPS: [(&str, f64); 4] = [
    ("savings", 0.05),
    ("bacteria", 0.1),
    ("radioactive decay", -0.05),
    ("coffee: its difference from the room", -0.1),
];
const FIVE_PERCENT: &str = "a year, growing all the time";

/// The staircase of compound interest with `steps` steps a year, as a
/// function of time: `(1 + 1/steps)^{floor(steps x)}`.
pub fn staircase(steps: u32, x: f64) -> f64 {
    let n = f64::from(steps);
    (1.0 + 1.0 / n).powf((n * x + 1e-9).floor())
}

/// The sum of the series through degree `d`, the growing sum of 05_2.
pub fn series_through(d: u32, x: f64) -> f64 {
    exp_partial(x, d)
}

/// The polyline of `f` over `[0, x1]` in `frame`, cut where it leaves the
/// top of the window (`samples` + 1 points).
fn curve_points(frame: &Frame2, x1: f64, samples: usize, f: &dyn Fn(f64) -> f64) -> Vec<Vec3> {
    let mut pts = Vec::with_capacity(samples + 1);
    for k in 0..=samples {
        let x = x1 * k as f64 / samples as f64;
        let y = f(x);
        if y > frame.y.1 {
            break;
        }
        pts.push(frame.p(x, y));
    }
    if pts.len() < 2 {
        pts.push(frame.p(0.0, f(0.0)));
        pts.push(frame.p(x1 / samples as f64, f(0.0)));
    }
    pts
}

/// The axes of a window with tick numbers at `ticks`, and `unit` at the
/// right end of the x axis (detached).
fn axes(kit: &Kit, stage: &mut Stage<'_>, frame: &Frame2, ticks: &[f64], unit: &str) -> fmn::Result<Vec<Mob>> {
    let style = || faded_line(Palette::dim(), 3.0, 1.0);
    let across = polyline(vec![frame.p(0.0, 0.0), frame.p(frame.x.1, 0.0)])
        .style(style())
        .build();
    let up = polyline(vec![frame.p(0.0, 0.0), frame.p(0.0, frame.y.1)])
        .style(style())
        .build();
    let mut out = vec![stage.arena_mut().add(across), stage.arena_mut().add(up)];
    for &t in ticks {
        let at = frame.p(t, 0.0);
        out.push(kit.text(stage, &whole(t), 24.0, Palette::dim(), v(at[0], at[1] - 0.3))?);
    }
    let end = frame.p(frame.x.1, 0.0);
    out.push(kit.text(stage, unit, 24.0, Palette::dim(), v(end[0] - 0.2, end[1] - 0.62))?);
    Ok(out)
}

/// The dial of the share: a half circle, its ticks, a needle that follows
/// `r` and a readout. Built detached; the needle is placed by an updater.
struct Dial {
    parts: Vec<Mob>,
}

impl Dial {
    fn build(kit: &Kit, stage: &mut Stage<'_>, r: Mob) -> fmn::Result<Self> {
        let at = DIAL_AT;
        let arc: Vec<Vec3> = (0..=40)
            .map(|i| {
                let phi = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * f64::from(i) / 40.0;
                v(at[0] + DIAL_R * phi.sin(), at[1] + DIAL_R * phi.cos())
            })
            .collect();
        let arc = stage
            .arena_mut()
            .add(polyline(arc).style(faded_line(Palette::dim(), 4.0, 1.0)).build());
        let mut parts = vec![arc];
        for tick in [-R_MAX, 0.0, R_MAX] {
            let phi = tick / R_MAX * std::f64::consts::FRAC_PI_2;
            let (s, c) = (phi.sin(), phi.cos());
            let t = polyline(vec![
                v(at[0] + 0.9 * DIAL_R * s, at[1] + 0.9 * DIAL_R * c),
                v(at[0] + 1.08 * DIAL_R * s, at[1] + 1.08 * DIAL_R * c),
            ])
            .style(faded_line(Palette::dim(), 3.0, 1.0))
            .build();
            parts.push(stage.arena_mut().add(t));
        }
        let needle = stage.arena_mut().add(
            polyline(vec![at, v(at[0], at[1] + 0.75 * DIAL_R)])
                .style(faded_line(result(), 6.0, 1.0))
                .build(),
        );
        let hub = stage
            .arena_mut()
            .add(Dot::new().point(at).radius(0.07).style(flat(result(), 1.0)).build());
        let name = kit.text(stage, "r", 40.0, result(), v(at[0] - 1.25, at[1] + 0.1))?;
        parts.extend([needle, hub, name]);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let share = s.tracker_value(r).unwrap_or(0.0);
                if share.to_bits() == last.to_bits() {
                    return;
                }
                let phi = (share / R_MAX).clamp(-1.0, 1.0) * std::f64::consts::FRAC_PI_2 * 0.92;
                let tip = v(at[0] + 0.75 * DIAL_R * phi.sin(), at[1] + 0.75 * DIAL_R * phi.cos());
                let _ = s.set_points(needle, polyline(vec![at, tip]).build().points());
                last = share;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let readout = kit.live_value(
            stage,
            move |s| s.tracker_value(r).unwrap_or(0.0),
            2,
            false,
            40.0,
            Palette::ink(),
            v(at[0] + 1.35, at[1] - 0.05),
            LEFT,
        )?;
        stage.remove_from_scene(readout);
        parts.push(readout);
        Ok(Self { parts })
    }
}

impl SceneConstruct for OneFunction {
    fn name(&self) -> &str {
        "05_one_function"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();

        // 05_1: the series at x = 1: its running total climbs to the bank's
        // number
        let line = say(&kit, stage, "05_1")?;
        let columns = [-5.0, -3.4, -1.9, -0.2, 1.7];
        let words = ["1", "1", "\\frac{1}{2}", "\\frac{1}{6}", "\\frac{1}{24}"];
        let mut terms = Vec::new();
        for (x, w) in columns.iter().zip(words) {
            terms.push(kit.tex(stage, w, 76.0, ink, &[], v(*x, 2.3))?);
        }
        let plus: Vec<Mob> = [-4.2, -2.65, -1.0, 0.75, 2.7]
            .iter()
            .map(|&x| {
                kit.tex(
                    stage,
                    if x > 2.0 { "+ \\cdots" } else { "+" },
                    60.0,
                    Palette::dim(),
                    &[],
                    v(x, 2.3),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let k = stage.add_value_tracker(0.0);
        let total_label = kit.text(stage, RUNNING_TOTAL, 32.0, Palette::dim(), v(-2.4, 0.5))?;
        let total = kit.live_value(
            stage,
            move |s| exp_partial(1.0, s.tracker_value(k).unwrap_or(0.0).round().max(0.0) as u32),
            5,
            false,
            100.0,
            result(),
            v(-2.4, -0.7),
            ORIGIN,
        )?;
        stage.remove_from_scene(total);
        let bank_label = kit.text(stage, BANKS_NUMBER, 32.0, Palette::dim(), v(3.9, 0.5))?;
        let bank = kit.text(stage, fact("F-V.03_4.e")?, 80.0, ink, v(3.9, -0.7))?;
        // "x = 1" is on the stage as "Try the series at x equals one" is said
        // (the narrated draft's tile was a blank stage for two seconds)
        let at_one = kit.tex(stage, "x = 1", 80.0, ink, &[], v(-4.9, 3.4))?;
        fade_in_all(stage, &[at_one], 0.5)?;
        line.wait_for(stage, "One, plus one")?;
        fade_in_all(stage, &[terms[0], terms[1], plus[0], total_label, total], 0.6)?;
        play!(stage; tween(k, 1.0, 0.5)?);
        line.wait_for(stage, "plus a half")?;
        fade_in_all(stage, &[plus[1], terms[2]], 0.5)?;
        play!(stage; tween(k, 2.0, 0.5)?);
        line.wait_for(stage, "plus a sixth")?;
        fade_in_all(stage, &[plus[2], terms[3]], 0.5)?;
        play!(stage; tween(k, 3.0, 0.5)?);
        line.wait_for(stage, "The running total climbs")?;
        fade_in_all(stage, &[plus[3], terms[4], plus[4]], 0.5)?;
        play!(stage; tween(k, 9.0, 3.0)?);
        line.wait_for(stage, "The bank's number")?;
        fade_in_all(stage, &[bank_label, bank], 0.6)?;
        kit.hold(stage)?;

        // 05_2: both obey rate equals amount from the same start: the
        // staircase and the growing sum close in on one curve
        let line = say(&kit, stage, "05_2")?;
        let mut leaving = terms.clone();
        leaving.extend(&plus);
        leaving.extend([at_one, total_label, total, bank_label, bank]);
        fade_out_all(stage, &leaving, 0.5)?;
        let window = UNIT_WINDOW;
        let chart = axes(&kit, stage, &window, &[0.0, 1.0, 2.0], "x")?;
        let the_curve = stage.arena_mut().add(
            polyline(curve_points(&window, 2.2, 120, &f64::exp))
                .style(faded_line(result(), 7.0, 1.0))
                .build(),
        );
        let steps_curve = stage.arena_mut().add(
            polyline(curve_points(&window, 2.0, 120, &|x| staircase(2, x)))
                .style(faded_line(substance(), 5.0, 1.0))
                .build(),
        );
        let sum_curve = stage.arena_mut().add(
            polyline(curve_points(&window, 2.0, 120, &|x| series_through(2, x)))
                .style(faded_line(rule(), 5.0, 1.0))
                .build(),
        );
        let level = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let m = s.tracker_value(level).unwrap_or(0.0).clamp(0.0, 4.0);
                if m.to_bits() == last.to_bits() {
                    return;
                }
                let (lo, f) = (m.floor().min(3.0), m - m.floor().min(3.0));
                let lo = lo as u32;
                // level L: 2^(L+1) steps a year, and the series through degree L + 2
                let steps = |l: u32| 1_u32 << (l + 1);
                let blend = |at: f64, a: f64, b: f64| at.mul_add(0.0, (1.0 - f) * a + f * b);
                let stair = |x: f64| blend(x, staircase(steps(lo), x), staircase(steps(lo + 1), x));
                let sum = |x: f64| blend(x, series_through(lo + 2, x), series_through(lo + 3, x));
                let _ = s.set_points(steps_curve, &corners(&curve_points(&window, 2.0, 120, &stair)));
                let _ = s.set_points(sum_curve, &corners(&curve_points(&window, 2.0, 120, &sum)));
                last = m;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let key_steps = kit.text(stage, STAIRCASE, 28.0, substance(), v(PANEL_X, 1.9))?;
        let key_sum = kit.text(stage, PIECES, 28.0, rule(), v(PANEL_X, 1.2))?;
        let key_curve = kit.tex(stage, "e^{x}", 70.0, result(), &[], v(PANEL_X, 0.1))?;
        // the chart comes in as the sum goes, not at "both obey" (it came
        // 0.6 s after the sum had gone: a bare stage under "That's because")
        let mut ins = chart.clone();
        ins.extend([the_curve, steps_curve, sum_curve, key_steps, key_sum]);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "and only one curve can")?;
        // the argument for it comes with the sentence, before the two
        // approximations close in (it came after them, with the line's end,
        // and stood 0.5 s before 05_3 took it)
        let same = precisely(&kit, stage, PRECISELY_SAME)?;
        fade_in_all(stage, &[same], 0.5)?;
        play!(stage; tween(level, 4.0, 3.2)?);
        fade_in_all(stage, &[key_curve], 0.5)?;
        kit.hold(stage)?;

        // 05_3: loosen the rule: a fixed share, five percent a year: the
        // same curve, stretched
        let line = say(&kit, stage, "05_3")?;
        let mut leaving = chart.clone();
        leaving.extend([the_curve, steps_curve, sum_curve, key_steps, key_sum, key_curve, same]);
        fade_out_all(stage, &leaving, 0.5)?;
        let share = fact("F-V.05_3.share")?;
        let rate = stage.add_value_tracker(0.05);
        let years = YEARS_WINDOW;
        let chart = axes(&kit, stage, &years, &[0.0, 10.0, 20.0, 30.0, 40.0], "years")?;
        let growth = stage.arena_mut().add(
            polyline(curve_points(&years, 40.0, 80, &|t| (0.05 * t).exp()))
                .style(faded_line(result(), 7.0, 1.0))
                .build(),
        );
        let grow_anchor = stage.arena_mut().add(VMobject::new());
        let mut grown = f64::NAN;
        stage.add_updater(
            grow_anchor,
            move |s, _| {
                let r = s.tracker_value(rate).unwrap_or(0.0);
                if r.to_bits() == grown.to_bits() {
                    return;
                }
                let _ = s.set_points(growth, &corners(&curve_points(&years, 40.0, 80, &|t| (r * t).exp())));
                grown = r;
            },
            true,
        )?;
        stage.add_to_scene(grow_anchor)?;
        let dial = Dial::build(&kit, stage, rate)?;
        let rule_now = kit.tex(stage, &format!("y' = {share}\\, y"), 70.0, rule(), &[], v(PANEL_X, 1.7))?;
        let result_now = kit.tex(
            stage,
            &format!("y = e^{{{share}\\, t}}"),
            70.0,
            result(),
            &[],
            v(PANEL_X, 0.5),
        )?;
        let letters_rule = kit.tex(stage, "y' = r\\, y", 70.0, rule(), &[], v(PANEL_X, -0.9))?;
        let letters_result = kit.tex(stage, "y = y(0)\\, e^{rt}", 70.0, result(), &[], v(PANEL_X, -2.0))?;
        let whatever = kit.text(stage, WHATEVER, 26.0, Palette::dim(), v(PANEL_X, -2.75))?;
        let mut ins = chart.clone();
        ins.extend(&dial.parts);
        ins.push(growth);
        line.wait_for(stage, "Now loosen the rule")?;
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "say five percent a year")?;
        fade_in_all(stage, &[rule_now], 0.6)?;
        line.wait_for(stage, "The answer is the same curve")?;
        fade_in_all(stage, &[result_now], 0.6)?;
        line.wait_for(stage, "E to the zero point zero five t")?;
        fade_in_all(stage, &[letters_rule, letters_result], 0.6)?;
        fade_in_all(stage, &[whatever], 0.4)?;
        kit.hold(stage)?;

        // 05_4: turn the dial: one function handles them all
        let line = say(&kit, stage, "05_4")?;
        fade_out_all(
            stage,
            &[rule_now, result_now, letters_rule, letters_result, whatever],
            0.4,
        )?;
        let name_at = v(-2.2, 2.35);
        let mut name = kit.text_left(stage, STOPS[0].0, 34.0, ink, name_at)?;
        let mut seen_names = vec![name];
        fade_in_all(stage, &[name], 0.3)?;
        line.wait_for(stage, "Savings")?;
        play!(stage; tween(rate, STOPS[0].1, 0.4)?);
        for (phrase, (label, share)) in ["bacteria", "radioactive decay", "a cooling cup of coffee"]
            .into_iter()
            .zip(&STOPS[1..])
        {
            line.wait_for(stage, phrase)?;
            let next = kit.text_left(stage, label, 34.0, ink, name_at)?;
            play!(stage;
                tween(rate, *share, 1.2)?,
                fade_out(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.3),
                fade_in(stage.arena_mut(), next, ORIGIN, 1.0)?.rt(0.5),
            );
            name = next;
            seen_names.push(next);
        }
        line.wait_for(stage, "One function handles them all")?;
        let ode = precisely(&kit, stage, PRECISELY_ODE)?;
        fade_in_all(stage, &[ode], 0.5)?;
        kit.hold(stage)?;

        // 05_5: your turn: five percent a year for twenty years
        let line = say(&kit, stage, "05_5")?;
        fade_out_all(stage, &[name, ode], 0.4)?;
        play!(stage; tween(rate, 0.05, 0.8)?);
        let rate_5 = numbers("F-N.growth.rate")?[0];
        let years_20 = numbers("F-N.growth.years")?[0];
        let card = QuizCard::build(&kit, stage, "05_5", CARD_AT, CARD_SIZE)?;
        let (centre, _, _) = card.content();
        let ask_a = kit.text(
            stage,
            &format!("{}% {FIVE_PERCENT}", whole(rate_5 * 100.0)),
            30.0,
            ink,
            v(centre[0], centre[1] + 0.6),
        )?;
        let ask_b = kit.text(
            stage,
            &format!("for {} years", whole(years_20)),
            34.0,
            ink,
            v(centre[0], centre[1] - 0.1),
        )?;
        let ask_c = kit.text(
            stage,
            "how many times bigger?",
            30.0,
            Palette::dim(),
            v(centre[0], centre[1] - 0.8),
        )?;
        // the marker at twenty years: where it stands, not what it reads
        let at = years.p(years_20, (rate_5 * years_20).exp());
        let marker = stage
            .arena_mut()
            .add(Dot::new().point(at).radius(0.11).style(flat(good(), 1.0)).build());
        let drop = stage.arena_mut().add(
            polyline(vec![at, years.p(years_20, 0.0)])
                .style(faded_line(good(), 3.0, 0.8))
                .build(),
        );
        let marker_name = kit.text(
            stage,
            &format!("{} years", whole(years_20)),
            26.0,
            good(),
            v(at[0], years.p(0.0, 0.0)[1] - 0.62),
        )?;
        card.show_frame(stage, &[ask_a, ask_b, ask_c, marker, drop, marker_name], 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 05_6: five percent times twenty is one: it grows by e to the one
        let line = say(&kit, stage, "05_6")?;
        let answer = kit.tex(
            stage,
            &format!("\\times\\, {}", fact("F-Q.05_5.answer")?),
            80.0,
            result(),
            &[],
            v(CARD_AT[0], card.ring_at()[1]),
        )?;
        card.land(stage, &[answer], 0.3)?;
        let product = kit.tex(
            stage,
            &format!(
                "{share} \\times {} = {}",
                whole(years_20),
                fact("F-V.05_6.rate_times_time")?
            ),
            56.0,
            ink,
            &[],
            v(CARD_AT[0], centre[1] + 0.6),
        )?;
        let power = kit.tex(
            stage,
            &format!(
                "e^{{{}}} = {}",
                fact("F-V.05_6.rate_times_time")?,
                fact("F-V.05_6.factor")?
            ),
            56.0,
            result(),
            &[],
            v(CARD_AT[0], centre[1] - 0.55),
        )?;
        let reading = kit.text(
            stage,
            &format!("\u{d7} {}", fact("F-V.05_6.factor")?),
            34.0,
            good(),
            v(at[0] - 0.9, at[1] + 0.35),
        )?;
        fade_out_all(stage, &[ask_a, ask_b, ask_c], 0.4)?;
        line.wait_for(stage, "Five percent times twenty")?;
        fade_in_all(stage, &[product], 0.6)?;
        line.wait_for(stage, "So it grows by")?;
        fade_in_all(stage, &[power], 0.6)?;
        line.wait_for(stage, "Two point seven")?;
        fade_in_all(stage, &[reading], 0.5)?;
        // the working has done its work once the line is said: the card goes
        // and the chart is left a while with the marker's reading (the card
        // stood on to the chapter's last second, eleven of them)
        line.wait_end(stage)?;
        let mut worked = card.mobs();
        worked.extend([product, power, answer]);
        fade_out_all(stage, &worked, 0.6)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        let _ = (seen_names, turning());
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{quiz_at, show, testing};
    use super::*;

    /// 05_1: the running totals of the series at x = 1 are the facts, and
    /// the bank's number is where they settle.
    #[test]
    fn the_running_total_climbs_to_the_banks_number() {
        let running = numbers("F-V.05_1.running").unwrap();
        assert_eq!(running.len(), 6);
        for (i, total) in running.iter().enumerate() {
            let k = i as u32 + 1;
            assert_eq!(show(exp_partial(1.0, k), 3).unwrap(), format!("{total:.3}"), "k = {k}");
        }
        assert_eq!(show(exp_partial(1.0, 9), 5).unwrap(), fact("F-V.05_1.settles").unwrap());
        assert_eq!(fact("F-V.05_1.settles").unwrap(), fact("F-V.03_4.e").unwrap());
        // "One, plus one, plus a half, plus a sixth": the terms are 1/k!
        let terms: Vec<f64> = (0..4).map(|k| 1.0 / crate::model::factorial(k) as f64).collect();
        assert_eq!(terms, vec![1.0, 1.0, 0.5, 1.0 / 6.0]);
    }

    /// 05_2: both approximations close in on the one curve, from the same
    /// start, and neither passes it: the staircase falls short of e to the
    /// x at every x, and so does every partial sum.
    #[test]
    fn the_staircase_and_the_sum_close_in_on_e_to_the_x() {
        for x in [0.3, 0.9, 1.4, 2.0] {
            let mut last_stair = 0.0_f64;
            let mut last_sum = 0.0_f64;
            for l in 0..4_u32 {
                let (stair, sum) = (staircase(1 << (l + 1), x), series_through(l + 2, x));
                assert!(stair <= x.exp() + 1e-12 && sum <= x.exp() + 1e-12, "x = {x}, l = {l}");
                assert!(stair >= last_stair - 1e-12 && sum >= last_sum, "x = {x}, l = {l}");
                (last_stair, last_sum) = (stair, sum);
            }
            // the start is shared
            assert_eq!(staircase(8, 0.0), 1.0);
            assert_eq!(series_through(3, 0.0), 1.0);
        }
        // the remainder after degree d at x is below x^(d+1) / (d+1)!
        assert!((series_through(20, 2.0) - 2.0_f64.exp()).abs() < 1e-12);
        assert!((staircase(1 << 20, 1.0) - std::f64::consts::E).abs() < 1e-5);
    }

    /// 05_3 to 05_6: the share, the product and the factor are the facts,
    /// and e to 0.05 t at twenty years is e to the one.
    #[test]
    fn five_percent_for_twenty_years_is_e_to_the_one() {
        assert_eq!(fact("F-V.05_3.share").unwrap(), "0.05");
        assert_eq!(fact("F-V.05_6.rate_times_time").unwrap(), "1");
        assert_eq!(fact("F-V.05_6.factor").unwrap(), "2.718");
        assert_eq!(fact("F-Q.05_5.answer").unwrap(), "2.718");
        let (r, t) = (
            numbers("F-N.growth.rate").unwrap()[0],
            numbers("F-N.growth.years").unwrap()[0],
        );
        assert_eq!((r, t), (0.05, 20.0));
        assert_eq!(show((r * t).exp(), 3).unwrap(), "2.718");
        // growing all the time is not 5% added once a year
        assert!(1.05_f64.powi(20) < (r * t).exp());
        assert_eq!(quiz_at("05_5").unwrap().seconds, 5);
        // the dial's stops are inside its range and the curve stays in its
        // window at twenty years for the quiz's share
        assert!(STOPS.iter().all(|(_, s)| s.abs() <= R_MAX + 1e-12));
        assert!(YEARS_WINDOW.y.1 > (r * t).exp());
    }

    /// 05_4: the one curve at every stop of the dial: it grows for a
    /// positive share, is flat at zero and decays below, always at the
    /// rate `r` times its amount.
    #[test]
    fn one_function_at_every_stop_of_the_dial() {
        for (_, r) in STOPS {
            for t in [0.0, 5.0, 17.0, 33.0] {
                let h = 1e-6;
                let y = |t: f64| (r * t).exp();
                let slope = (y(t + h) - y(t - h)) / (2.0 * h);
                assert!((slope - r * y(t)).abs() < 1e-6, "r = {r}, t = {t}");
            }
            let up = (r * 40.0).exp() > 1.0;
            assert_eq!(up, r > 0.0);
        }
        assert_eq!((0.0_f64 * 40.0).exp(), 1.0);
    }

    /// 05_2 and 05_4, the dense read of the scored draft: the one-line lower
    /// third that comes up under each window (its box tops out at -3.14) is
    /// clear of the window's tick numbers and its unit label; at the old
    /// height "x" and "years" stood inside the box.
    #[test]
    fn the_windows_stand_clear_of_the_lower_third_under_them() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        // props::lower_third: 0.2 over the frame's foot, 0.3 and a line of 0.36
        let band_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36;
        for (name, w) in [("unit", UNIT_WINDOW), ("years", YEARS_WINDOW)] {
            let ticks = w.bottom() - 0.3 - 0.15;
            let label = w.bottom() - 0.62 - 0.15;
            assert!(
                ticks > band_top + 0.05,
                "{name}: tick numbers reach {ticks:.2}, the band's top is {band_top:.2}"
            );
            assert!(
                label > band_top + 0.05,
                "{name}: the unit label reaches {label:.2}, the band's top is {band_top:.2}"
            );
            // the marker's own label at 05_5 stands on the same row
            assert!(w.bottom() - 0.62 - 0.17 > band_top);
        }
        // and the window's top is under the dial's base (2.35)
        assert!(YEARS_WINDOW.top() < DIAL_AT[1] - 0.2, "{}", YEARS_WINDOW.top());
        assert!(UNIT_WINDOW.top() < DIAL_AT[1] - 0.2);
    }

    /// The longest run of frames with no picture among frames `from..to`.
    fn longest_blank_run(sizes: &[u64], from: usize, to: usize) -> usize {
        let (mut run, mut longest) = (0, 0);
        for i in from..to {
            if testing::has_picture(sizes, i) {
                run = 0;
            } else {
                run += 1;
                longest = longest.max(run);
            }
        }
        longest
    }

    /// 05_2 (the dense read of the scored draft): the sum of 05_1 goes as the
    /// line begins and the chart comes in at once, not at "both obey rate
    /// equals amount": the whole frame was empty for 0.6 s under "That's
    /// because". At 10 frames a second no run of bare frames is longer than
    /// three, from the line's start to the end of its first sentence's
    /// beginning (3 s in).
    #[test]
    fn the_chart_comes_in_as_the_sum_goes_not_after_a_bare_stage() {
        let voiced = testing::run_voiced("05_one_function").unwrap();
        let sizes = testing::frame_sizes(&voiced.ran.frames_dir).unwrap();
        let (_, start, _) = voiced.lines.iter().find(|(id, ..)| id == "05_2").cloned().unwrap();
        let (from, to) = ((start * 10.0).floor() as usize, ((start + 3.0) * 10.0).ceil() as usize);
        let longest = longest_blank_run(&sizes, from, to);
        assert!(
            longest <= 3,
            "{longest} bare frames in a row between {start:.1} s and {:.1} s",
            start + 3.0
        );
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("05_one_function").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "2.71828",
            "y' = 0.05\\, y",
            "y = y(0)\\, e^{rt}",
            "5% a year, growing all the time",
            "for 20 years",
            "0.05 \\times 20 = 1",
            "e^{1} = 2.718",
            "savings",
            "coffee: its difference from the room",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 35.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
