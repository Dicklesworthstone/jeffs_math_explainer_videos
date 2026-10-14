//! Chapter 03, growth: one dollar at a hundred percent a year becomes two
//! (03_1); split the year in two and the second step earns on the first
//! (03_2); the quiz, whether finer and finer steps grow without limit (03_3);
//! no, they settle, at e (03_4); every step followed one rule, so the rate
//! equals the amount (03_5); and the one curve that starts at one with rate
//! equal to amount is e to the x (03_6).
//!
//! The chart is one data window (`CHART`: years across, dollars up). Every
//! number is a fact: the totals of the staircase and of the quiz are
//! `F-N.compound.n1` and `F-V.03_*`, the daily and million-step totals
//! `F-V.03_4.*`, Bernoulli's year `F-B.bernoulli.year`, the answer of the
//! quiz `F-Q.03_3.answer`.

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, rect, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    QuizCard, corners, fact, fade_in_all, fade_out_all, faded_line, fit_width, flat, music_hold, result, rule, say,
    substance, whole,
};

pub struct Growth {
    pub kit: Kit,
}

/// Years across, dollars up (2.2 years, 8 dollars: e squared is 7.39).
const CHART: Frame2 = Frame2 {
    center: [-2.6, 0.0, 0.0],
    width: 6.6,
    height: 4.8,
    x: (0.0, 2.2),
    y: (0.0, 8.0),
};
/// The right panel's centre.
const PANEL_X: f64 = 3.9;
/// The quiz card.
const CARD_AT: Vec3 = [0.0, 0.3, 0.0];
const CARD_SIZE: (f64, f64) = (9.6, 6.2);
/// The totals the quiz keeps in view: how many steps in the year.
const QUIZ_STEPS: [u32; 4] = [1, 2, 4, 12];
/// The card's rows: their type size, where the first stands above the
/// content's middle and the pitch down; and the question the voice asks, a
/// row's worth under them (the card kept the totals and not the question).
const ROW_PT: f64 = 52.0;
const ROW_TOP: f64 = 1.5;
const ROW_PITCH: f64 = 0.75;
const QUESTION_DY: f64 = -1.5;
const QUESTION: &str = "does the total grow without limit?";

// the chapter's fixed words (numbers are built from facts)
const YEARS: &str = "years";
const RATE_MEANS: &str = "f\u{2032} means: the rate of f";
const PRECISELY_RULE: &str = "f\u{2032} = f, and f(0) = 1";
const WHO_AND_WHEN: &str = "who and when:";
const AMOUNT: &str = "height: the amount";
const RATE: &str = "rise in a year: the rate";
const RATE_EQUALS_AMOUNT: &str = "rate = amount";
const ONE_STEP: &str = "one step";
const STEPS: &str = "steps";

/// One dollar after each of `n` steps of a year at a hundred percent a year,
/// `(1 + 1/n)^j` for `j = 0..=n`.
pub fn totals(n: u32) -> Vec<f64> {
    let step = 1.0 + 1.0 / f64::from(n);
    (0..=n).map(|j| step.powi(j as i32)).collect()
}

/// The largest x the chart's curve is drawn to: where e to the x leaves the
/// top of the window.
fn curve_end() -> f64 {
    CHART.y.1.ln().min(CHART.x.1)
}

/// The staircase of `n` steps: for each step the dollars it began with (an
/// amber bar) and, on top of it, what the step earned (a teal slice).
fn staircase(stage: &mut Stage<'_>, n: u32) -> fmn::Result<Vec<Mob>> {
    let t = totals(n);
    let w = 1.0 / f64::from(n);
    let floor = CHART.p(0.0, 0.0)[1];
    let mut out = Vec::new();
    for j in 0..n as usize {
        let (x0, x1) = (j as f64 * w, (j + 1) as f64 * w);
        let (left, right) = (CHART.p(x0, 0.0)[0], CHART.p(x1, 0.0)[0]);
        let gap = 0.03 * (right - left).min(1.0);
        let width = right - left - 2.0 * gap;
        let (base, top) = (CHART.p(0.0, t[j])[1], CHART.p(0.0, t[j + 1])[1]);
        let held = rect(v(left + gap, floor), width, base - floor, flat(substance(), 0.92))?;
        let gain = rect(v(left + gap, base), width, top - base, flat(rule(), 0.95))?;
        out.push(stage.arena_mut().add(held));
        out.push(stage.arena_mut().add(gain));
    }
    Ok(out)
}

/// The axes: years across, dollars up, with their numbers (detached).
fn axes(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let style = || faded_line(Palette::dim(), 3.0, 1.0);
    let across = polyline(vec![CHART.p(0.0, 0.0), CHART.p(CHART.x.1, 0.0)])
        .style(style())
        .build();
    let up = polyline(vec![CHART.p(0.0, 0.0), CHART.p(0.0, CHART.y.1)])
        .style(style())
        .build();
    let mut out = vec![stage.arena_mut().add(across), stage.arena_mut().add(up)];
    for year in [0.0, 1.0, 2.0] {
        let at = CHART.p(year, 0.0);
        out.push(kit.text(stage, &whole(year), 24.0, Palette::dim(), v(at[0], at[1] - 0.3))?);
    }
    let end = CHART.p(CHART.x.1, 0.0);
    out.push(kit.text(stage, YEARS, 24.0, Palette::dim(), v(end[0] - 0.1, end[1] - 0.3))?);
    for dollars in [1.0, 2.0, 4.0, 8.0] {
        let at = CHART.p(0.0, dollars);
        out.push(kit.text(
            stage,
            &format!("${}", whole(dollars)),
            24.0,
            Palette::dim(),
            v(at[0] - 0.45, at[1]),
        )?);
    }
    Ok(out)
}

/// e to the x from 0 to `x1`, as one polyline, detached.
fn curve(stage: &mut Stage<'_>, x1: f64, color: Srgb, width: f64) -> Mob {
    const SAMPLES: usize = 120;
    let pts: Vec<Vec3> = (0..=SAMPLES)
        .map(|k| {
            let x = x1 * k as f64 / SAMPLES as f64;
            CHART.p(x, x.exp())
        })
        .collect();
    stage
        .arena_mut()
        .add(polyline(pts).style(faded_line(color, width, 1.0)).build())
}

/// 03_3's rows, the totals of one, two, four and twelve steps, and under them
/// the question the voice asks (detached): the card's content has `centre`
/// as its middle and `room` as its width.
fn quiz_rows(kit: &Kit, stage: &mut Stage<'_>, centre: Vec3, room: f64) -> fmn::Result<(Vec<Mob>, Mob)> {
    let mut rows = Vec::new();
    for (k, n) in QUIZ_STEPS.into_iter().enumerate() {
        let value = match n {
            1 => whole(
                fact("F-N.compound.n1")?
                    .parse::<f64>()
                    .map_err(|e| super::draw::fail(e.to_string()))?,
            ),
            2 => fact("F-V.03_2.two_steps")?.to_owned(),
            4 => fact("F-V.03_3.four_steps")?.to_owned(),
            _ => fact("F-V.03_3.twelve_steps")?.to_owned(),
        };
        let src = format!("(1 + 1/{n})^{{{n}}} = {value}");
        let y = centre[1] + ROW_TOP - ROW_PITCH * k as f64;
        rows.push(kit.tex_left(
            stage,
            &src,
            ROW_PT,
            Palette::ink(),
            &[],
            v(centre[0] - room / 2.0 + 1.3, y),
        )?);
    }
    let question = kit.text(
        stage,
        QUESTION,
        36.0,
        Palette::ink(),
        v(centre[0], centre[1] + QUESTION_DY),
    )?;
    fit_width(stage, question, room);
    Ok((rows, question))
}

/// The slope construction at `a` years: the height of the curve there (the
/// amount), and the rise of its tangent over the next year (the rate), the
/// two as vertical segments, with the point.
fn slope_points(a: f64) -> [Vec<Vec3>; 4] {
    let h = a.exp();
    [
        vec![CHART.p(a, 0.0), CHART.p(a, h)],
        vec![CHART.p(a, h), CHART.p(a + 1.0, h)],
        vec![CHART.p(a + 1.0, h), CHART.p(a + 1.0, 2.0 * h)],
        vec![CHART.p(a, h), CHART.p(a + 1.0, 2.0 * h)],
    ]
}

impl SceneConstruct for Growth {
    fn name(&self) -> &str {
        "03_growth"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let axes = axes(&kit, stage)?;

        // 03_1: one dollar, a hundred percent a year, becomes two
        let line = say(&kit, stage, "03_1")?;
        fade_in_all(stage, &axes, 0.6)?;
        let one_step = staircase(stage, 1)?;
        let one_label = kit.text(stage, ONE_STEP, 30.0, Palette::dim(), v(PANEL_X, 2.3))?;
        line.wait_for(stage, "One dollar")?;
        fade_in_all(stage, &one_step[..1], 0.5)?;
        line.wait_for(stage, "becomes two")?;
        fade_in_all(stage, &[one_step[1], one_label], 0.7)?;
        kit.hold(stage)?;

        // 03_2: split the year in two; the second step earns on the first
        let line = say(&kit, stage, "03_2")?;
        let two_steps = staircase(stage, 2)?;
        let eq_two = {
            let value = fact("F-V.03_2.two_steps")?;
            kit.tex(
                stage,
                &format!("(1 + 1/2)^{{2}} = {value}"),
                78.0,
                ink,
                &[],
                v(PANEL_X, 0.4),
            )?
        };
        let mut gone = one_step.clone();
        gone.push(one_label);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_for(stage, "Fifty percent, twice")?;
        fade_in_all(stage, &two_steps[..2], 0.5)?;
        line.wait_for(stage, "The second step earns on the first")?;
        fade_in_all(stage, &two_steps[2..], 0.6)?;
        line.wait_for(stage, "so you end with")?;
        fade_in_all(stage, &[eq_two], 0.6)?;
        kit.hold(stage)?;

        // 03_3: your turn: finer and finer steps; the totals stay in view
        let line = say(&kit, stage, "03_3")?;
        let mut leaving = axes.clone();
        leaving.extend(&two_steps);
        leaving.push(eq_two);
        fade_out_all(stage, &leaving, 0.5)?;
        let card = QuizCard::build(&kit, stage, "03_3", CARD_AT, CARD_SIZE)?;
        let (centre, room, _) = card.content();
        let (rows, question) = quiz_rows(&kit, stage, centre, room)?;
        card.show_frame(stage, &rows[..2], 0.5)?;
        line.wait_for(stage, "Four steps give")?;
        fade_in_all(stage, &rows[2..3], 0.5)?;
        line.wait_for(stage, "Twelve give")?;
        fade_in_all(stage, &rows[3..], 0.5)?;
        line.wait_last(stage)?;
        // the question comes up as it is asked, with the ring
        fade_in_all(stage, &[question], 0.4)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // 03_4: no: it settles, at e
        let line = say(&kit, stage, "03_4")?;
        let answer = kit.text(
            stage,
            fact("F-Q.03_3.answer")?,
            40.0,
            result(),
            v(card.at[0], card.ring_at()[1]),
        )?;
        card.land(stage, &[answer], 0.3)?;
        let mut card_mobs = card.mobs();
        card_mobs.extend(&rows);
        card_mobs.extend([question, answer]);
        // the card gives the screen back to the chart, with the curve the
        // steps settle on
        let shown_curve = curve(stage, curve_end(), result(), 6.0);
        let daily = format!("365 {STEPS}: {}", fact("F-V.03_4.daily")?);
        let million = format!("{} {STEPS}: {}", thousands(1_000_000_u64), fact("F-V.03_4.million")?);
        let daily = kit.text(stage, &daily, 40.0, ink, v(PANEL_X, 2.5))?;
        let million = kit.text(stage, &million, 40.0, ink, v(PANEL_X, 1.7))?;
        let e_value = kit.text(
            stage,
            &format!("e = {}\u{2026}", fact("F-V.03_4.e")?),
            60.0,
            result(),
            v(PANEL_X, 0.5),
        )?;
        // (fresh axes: a faded-out mobject is gone for good)
        let chart_again = self::axes(&kit, stage)?;
        // the answer stays fully up until "No." has been said (the finished
        // read puts that at 0.82 s; the card went at 0.3 s)
        line.wait_first_sentence(stage)?;
        fade_out_all(stage, &card_mobs, 0.5)?;
        fade_in_all(stage, &chart_again, 0.5)?;
        let who = lower_third(
            &kit,
            stage,
            WHO_AND_WHEN,
            &format!("Jacob Bernoulli asked this in {}", fact("F-B.bernoulli.year")?),
        )?;
        #[cfg(test)]
        super::draw::watch_dwell(&kit, stage, who, "03_4:credit")?;
        line.wait_for(stage, "A step every day")?;
        // the credit for the question the card just asked comes up with the
        // first of the numbers that answer it, and stays through the line
        // (it came with the last sentence and stood two seconds)
        fade_in_all(stage, &[daily, who], 0.5)?;
        line.wait_for(stage, "A million steps")?;
        play!(stage;
            fade_in(stage.arena_mut(), million, ORIGIN, 1.0)?.rt(0.5),
            show_creation(shown_curve).rt(1.6),
        );
        line.wait_for(stage, "That number is e")?;
        fade_in_all(stage, &[e_value], 0.6)?;
        kit.hold(stage)?;

        // 03_5: every step followed one rule: the rate equals the amount.
        // A point slides along the curve; the height there and the rise of
        // its tangent over a year stay the same length
        let line = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[daily, million, e_value, who], 0.4)?;
        let a = stage.add_value_tracker(0.0);
        let [height, run, rise, tangent] = slope_points(0.0);
        let mk = |stage: &mut Stage<'_>, pts: Vec<Vec3>, color: Srgb, w: f64, o: f64| {
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(color, w, o)).build())
        };
        let m_height = mk(stage, height, result(), 7.0, 1.0);
        let m_run = mk(stage, run, rule(), 3.0, 0.7);
        let m_rise = mk(stage, rise, rule(), 7.0, 1.0);
        let m_tangent = mk(stage, tangent, rule(), 3.0, 0.8);
        let dot_template: Vec<Vec3> = Dot::new().radius(0.11).build().points().to_vec();
        let m_dot = stage.arena_mut().add(
            Dot::new()
                .point(CHART.p(0.0, 1.0))
                .radius(0.11)
                .style(flat(result(), 1.0))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let parts = [m_height, m_run, m_rise, m_tangent];
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(a).unwrap_or(0.0);
                if t.to_bits() == last.to_bits() {
                    return;
                }
                for (&m, pts) in parts.iter().zip(slope_points(t)) {
                    let _ = s.set_points(m, &corners(&pts));
                }
                let c = CHART.p(t, t.exp());
                let moved: Vec<Vec3> = dot_template.iter().map(|p| [p[0] + c[0], p[1] + c[1], p[2]]).collect();
                let _ = s.set_points(m_dot, &moved);
                last = t;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let amount = kit.text_left(stage, AMOUNT, 30.0, result(), v(PANEL_X - 2.2, 2.6))?;
        let rate = kit.text_left(stage, RATE, 30.0, rule(), v(PANEL_X - 2.2, 2.0))?;
        let rate_means = kit.text(stage, RATE_MEANS, 36.0, ink, v(PANEL_X, 0.7))?;
        line.wait_for(stage, "In a thousandth of a year")?;
        fade_in_all(stage, &[m_height, m_run, m_rise, m_tangent, m_dot, amount, rate], 0.6)?;
        line.wait_for(stage, "So your gain per year")?;
        play!(stage; tween(a, 1.2, 4.0)?);
        line.wait_for(stage, "your rate")?;
        fade_in_all(stage, &[rate_means], 0.5)?;
        let exact = precisely(&kit, stage, PRECISELY_RULE)?;
        fade_in_all(stage, &[exact], 0.5)?;
        kit.hold(stage)?;

        // 03_6: the one curve that starts at one, with rate equal to amount
        let line = say(&kit, stage, "03_6")?;
        fade_out_all(
            stage,
            &[
                m_height, m_run, m_rise, m_tangent, m_dot, amount, rate, rate_means, exact,
            ],
            0.5,
        )?;
        // a mark at one year, at two and at "x" years, each as its words come
        let mut marks = Vec::new();
        for (year, words, phrase) in [
            (1.0, "e", "for a year"),
            (2.0, "e \\cdot e", "Two years"),
            (1.55, "e^{x}", "X years"),
        ] {
            let at = CHART.p(year, f64::exp(year));
            let dot = stage
                .arena_mut()
                .add(Dot::new().point(at).radius(0.1).style(flat(result(), 1.0)).build());
            let label = kit.tex(stage, words, 56.0, ink, &[], v(at[0] - 0.65, at[1] + 0.35))?;
            marks.push((phrase, dot, label));
        }
        let eq = kit.text(stage, RATE_EQUALS_AMOUNT, 56.0, rule(), v(PANEL_X, 1.2))?;
        for (phrase, dot, label) in &marks {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &[*dot, *label], 0.5)?;
        }
        line.wait_for(stage, "The one curve")?;
        fade_in_all(stage, &[eq], 0.6)?;
        music_hold(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{quiz_at, quiz_log, show, testing};
    use super::*;
    use crate::model::compound;

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// 03_3 (the dense read of J's first half): the card keeps the totals
    /// and now the question the voice asks, in the voice's own words, under
    /// them and inside the card's content.
    #[test]
    fn the_card_asks_the_question_the_voice_asks() {
        let said = crate::SCRIPT
            .lines()
            .find_map(|l| l.strip_prefix("03_3\t"))
            .expect("03_3 is in the script");
        let asked = said.rsplit(". ").next().unwrap();
        assert_eq!(asked.to_lowercase(), QUESTION, "the voice asks {asked:?}");
        run(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, "03_3", CARD_AT, CARD_SIZE)?;
            let (centre, room, height) = card.content();
            let (rows, question) = quiz_rows(&kit, stage, centre, room)?;
            assert!(stage.get_width(question) <= room + 1e-6);
            let (top, bottom) = (centre[1] + height / 2.0, centre[1] - height / 2.0);
            assert!(stage.get_top(rows[0])[1] <= top + 0.02, "the first row pokes out");
            assert!(
                stage.get_bottom(question)[1] >= bottom - 0.05,
                "the question sinks into the foot"
            );
            let last = *rows.last().unwrap();
            assert!(
                stage.get_bottom(last)[1] > stage.get_top(question)[1] + 0.1,
                "the last row {:.2} over the question {:.2}",
                stage.get_bottom(last)[1],
                stage.get_top(question)[1]
            );
            for pair in rows.windows(2) {
                assert!(stage.get_bottom(pair[0])[1] > stage.get_top(pair[1])[1], "rows overlap");
            }
            Ok(())
        });
    }

    /// 03_4: the credit for Bernoulli's question stands from the first of
    /// the answering numbers to the end of the line, not the two seconds
    /// after its last words that it stood.
    #[test]
    fn the_credit_stands_through_the_answer() {
        let voiced = testing::run_voiced("03_growth").unwrap();
        let events = quiz_log::take(&voiced.tag, "03_4:credit");
        let when = |kind: &str| events.iter().find(|(k, _)| *k == kind).map(|e| e.1);
        let (up, down) = (when("up").expect("it comes up"), when("down").expect("it goes"));
        let (_, start, len) = voiced.lines.iter().find(|(id, ..)| id == "03_4").cloned().unwrap();
        assert!(
            up > start && up < start + len * 0.6,
            "up at {up:.1} s in a line from {start:.1} s"
        );
        assert!(
            down >= start + len - 0.1,
            "it goes at {down:.1} s, the line ends {:.1} s",
            start + len
        );
        assert!(down - up >= 4.0, "it stood {:.1} s", down - up);
    }

    /// 03_1 to 03_3: the staircase is the compound-interest totals, exactly,
    /// and the numbers on the screen are the facts that say them.
    #[test]
    fn the_staircase_ends_where_the_facts_say() {
        for (n, key, places) in [
            (1_u32, "F-N.compound.n1", 5),
            (2, "F-V.03_2.two_steps", 2),
            (4, "F-V.03_3.four_steps", 2),
            (12, "F-V.03_3.twelve_steps", 2),
        ] {
            let t = totals(n);
            assert_eq!(t.len(), n as usize + 1);
            assert_eq!(t[0], 1.0);
            assert!((t[n as usize] - compound(1.0, n)).abs() < 1e-12, "n = {n}");
            assert_eq!(show(t[n as usize], places).unwrap(), fact(key).unwrap(), "n = {n}");
            // every step earns its share of what it began with
            for j in 0..n as usize {
                assert!((t[j + 1] - t[j] * (1.0 + 1.0 / f64::from(n))).abs() < 1e-12);
            }
        }
        // 03_2's own sentence: the second step earns on the first
        let t = totals(2);
        assert_eq!(t, vec![1.0, 1.5, 2.25]);
    }

    /// 03_3: "the jumps shrink": each doubling of the steps adds less, and
    /// the totals never pass e. The always sentence behind "it settles".
    #[test]
    fn finer_steps_add_less_and_never_pass_e() {
        let mut last = compound(1.0, 1);
        let mut gain = f64::INFINITY;
        for n in [2_u32, 4, 8, 16, 32, 64, 128, 365, 1_000_000] {
            let total = compound(1.0, n);
            assert!(total > last && total < std::f64::consts::E, "n = {n}: {total}");
            let step = total - last;
            if n <= 128 {
                assert!(step < gain, "n = {n}");
                gain = step;
            }
            last = total;
        }
        assert_eq!(show(compound(1.0, 365), 3).unwrap(), fact("F-V.03_4.daily").unwrap());
        assert_eq!(
            show(compound(1.0, 1_000_000), 5).unwrap(),
            fact("F-V.03_4.million").unwrap()
        );
        assert_eq!(show(std::f64::consts::E, 5).unwrap(), fact("F-V.03_4.e").unwrap());
        assert_eq!(quiz_at("03_3").unwrap().seconds, 4);
    }

    /// 03_5: at every point of the curve the height and the rise of the
    /// tangent over one year are the same length: rate equals amount.
    #[test]
    fn the_height_and_the_rise_stay_the_same_length() {
        for k in 0..=24 {
            let a = 1.2 * f64::from(k) / 24.0;
            let [height, run, rise, tangent] = slope_points(a);
            let len = |p: &Vec<Vec3>| (p[1][1] - p[0][1]).abs();
            assert!((len(&height) - len(&rise)).abs() < 1e-12, "a = {a}");
            // the run is exactly one year across
            let year = CHART.p(1.0, 0.0)[0] - CHART.p(0.0, 0.0)[0];
            assert!((run[1][0] - run[0][0] - year).abs() < 1e-12);
            // and the tangent joins the point to the top of the rise
            assert_eq!(tangent[0], run[0]);
            assert_eq!(tangent[1], rise[1]);
            // all of it stays inside the chart
            for p in [&height, &run, &rise, &tangent].into_iter().flatten() {
                assert!(
                    p[0] <= CHART.right() + 1e-9 && p[1] <= CHART.top() + 1e-9,
                    "a = {a}: {p:?}"
                );
            }
        }
        // the curve's slope at x is its height there: the facts' own rule
        let h = 1e-6_f64;
        for x in [0.0_f64, 0.5, 1.0, 1.7] {
            let slope = ((x + h).exp() - (x - h).exp()) / (2.0 * h);
            assert!((slope - x.exp()).abs() < 1e-6);
        }
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("03_growth").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "(1 + 1/2)^{2} = 2.25",
            "(1 + 1/4)^{4} = 2.44",
            "(1 + 1/12)^{12} = 2.61",
            "365 steps: 2.715",
            "1,000,000 steps: 2.71828",
            "e = 2.71828\u{2026}",
            "Jacob Bernoulli asked this in 1690",
            PRECISELY_RULE,
            "your turn",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 30.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
