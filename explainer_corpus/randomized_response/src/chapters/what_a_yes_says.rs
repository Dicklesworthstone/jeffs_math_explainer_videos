//! Chapter 02, what a yes says: every student with all four coin outcomes,
//! eighty equally likely (02_1); one student's four outcomes for each truth
//! give both answers' ratio of three (02_2, 02_3); the prediction, a
//! yes-sayer's chance of having copied (02_4); the class's yes outcomes
//! counted, twelve of twenty-eight, three in seven and not three in four
//! (02_5, 02_6), and why (02_7); a no, four of fifty-two (02_8); the chance
//! moving from one in five along a line (02_9, 02_10); deniable but not
//! meaningless, and the count's accuracy a separate question (02_11).
//!
//! Every square is `model::answer` run on one student and one coin outcome;
//! every count drawn is checked against `model::class_outcomes` before it is
//! labelled.

use explainer_kit::props::{names_to_search, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    arc, disc, face, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, lane, pause, ring, say, square, swap,
    tex_q, think, wait_until,
};
use crate::facts::{self, Average};
use crate::model::{self, COIN_PAIRS, Q};
use crate::palette;

pub struct WhatAYesSays {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const COPIED: &str = "copied";
const DIDNT: &str = "didn't copy";
const YES: &str = "yes";
const NO: &str = "no";
const NAMES: [&str; 4] = [
    "likelihood ratio",
    "posterior probability",
    "randomized response",
    "differential privacy",
];

// ------------------------------------------------------------------ layout

/// The students across the top, the four who copied first, `COL_DX` apart,
/// then a gap and the others; their group names above them.
const COL_DX: f64 = 0.44;
const COPIER_X0: f64 = -5.0;
const GROUP_GAP: f64 = 0.6;
const STUDENT_Y: f64 = 2.55;
const GROUP_Y: f64 = 3.2;
/// The outcome squares under each student, one row per coin outcome, the
/// rows' faces on the left.
const ROW_Y0: f64 = 1.95;
const ROW_DY: f64 = 0.44;
const SIDE: f64 = 0.34;
const FACE_X: f64 = -5.75;
/// Each group's label under the squares; the ratio and the total on the
/// right; the think time above the ratio.
const LABEL_Y: f64 = 0.0;
const EACH_DY: f64 = 0.35;
const RIGHT_X: f64 = 5.4;
const TOTAL_Y: f64 = 1.3;
const THINK_Y: f64 = 1.5;
/// The fraction under the grid, and its notes beside it.
const DERIVE_X: f64 = -2.3;
const DERIVE_Y: f64 = -1.0;
const NOTE_X: f64 = 0.8;
const NOTE_DY: f64 = 0.25;
/// The chance line (02_9): 0 at `LINE_X0`, 1 at `LINE_X0 + LINE_W`; marks,
/// their fractions below, the class's caption under its mark.
const LINE_X0: f64 = -4.5;
const LINE_W: f64 = 9.0;
const LINE_Y: f64 = -1.7;
const TICK: f64 = 0.15;
const MARK_DY: f64 = 0.4;
const CAPTION_DY: f64 = 0.82;
const YES_ARC: f64 = 0.5;
const NO_ARC: f64 = 0.35;

/// The x of the k-th student as drawn, of whom the first `copiers` copied.
fn column_x(k: usize, copiers: usize) -> f64 {
    let gap = if k < copiers { 0.0 } else { GROUP_GAP };
    COPIER_X0 + COL_DX * k as f64 + gap
}

/// The middle of the columns `from..to`.
fn group_x(from: usize, to: usize, copiers: usize) -> f64 {
    (column_x(from, copiers) + column_x(to - 1, copiers)) / 2.0
}

/// The y of outcome row `r`.
fn row_y(r: usize) -> f64 {
    ROW_Y0 - ROW_DY * r as f64
}

/// Where a chance `q` sits on the chance line.
fn chance_x(q: Q) -> f64 {
    LINE_X0 + LINE_W * q.to_f64()
}

/// A ratio as odds are written, "1 : 4".
fn odds_text(q: Q) -> String {
    format!("{} : {}", q.num(), q.den())
}

/// One group's yes or no outcomes per student: "yes: 3 of 4".
fn per_student(said: bool, copied: bool) -> String {
    let word = if said { YES } else { NO };
    format!(
        "{word}: {} of {}",
        model::outcomes_saying(copied, said),
        COIN_PAIRS.len()
    )
}

/// Each group's per-student count for the answer `said`, under the
/// copiers' columns and the others' (at `xs`). Detached.
fn group_labels(kit: &Kit, stage: &mut Stage<'_>, said: bool, xs: [f64; 2]) -> fmn::Result<[Mob; 2]> {
    let color = palette::answer(said);
    Ok([
        kit.text(stage, &per_student(said, true), 22.0, color, v(xs[0], LABEL_Y))?,
        kit.text(stage, &per_student(said, false), 22.0, color, v(xs[1], LABEL_Y))?,
    ])
}

/// 02_6's note beside the fraction: the same count on average.
fn average_note(c: &[[u64; 2]; 2]) -> String {
    let per = COIN_PAIRS.len() as u64;
    let [didnt, copied] = c[1];
    format!(
        "a quarter of the outcomes: {} of chapter 1's {} yes answers",
        copied / per,
        (didnt + copied) / per
    )
}

/// 02_6's note: where the tempting answer comes from.
fn tempting_note() -> String {
    format!("{} is a copier's chance of saying yes", model::p_said(true, true))
}

/// 02_6's exact version: the chance is the class's, and when 3/4 would be
/// right.
fn posterior_card(c: &[[u64; 2]; 2], n: usize, copiers: usize) -> fmn::Result<String> {
    let yes = model::share_who_copied(c, true).ok_or_else(|| fail("no outcome says yes".into()))?;
    let half = n / 2;
    let seats: Vec<usize> = (0..half).collect();
    let class = model::Class::new(n, &seats).map_err(fail)?;
    let at_half = model::share_who_copied(&model::class_outcomes(&class), true)
        .ok_or_else(|| fail("no outcome says yes".into()))?;
    Ok(format!(
        "{yes} is the chance that someone who said yes copied, in this class where {copiers} of {n} did. It \
         would be {at_half} only if half the class, {half}, had copied"
    ))
}

/// 02_10's exact version: the coin multiplies the odds; the class sets them
/// (and in a class of `n` where nobody copied, a yes means nothing to fear).
/// The chances `yes` and `no` themselves are already marked on the chance
/// line, so the card gives only their odds; it stays two lines however far
/// the lower-third narrows its wrap (the tests check 92 down to 72).
fn odds_card(prior: Q, yes: Q, no: Q, n: usize) -> fmn::Result<String> {
    let odds = |q: Q| model::odds(q).ok_or_else(|| fail(format!("{q} has no odds")));
    let nobody = model::Class::new(n, &[]).map_err(fail)?;
    let none = model::share_who_copied(&model::class_outcomes(&nobody), true)
        .ok_or_else(|| fail("no outcome says yes".into()))?;
    Ok(format!(
        "an answer multiplies the odds by {} for a yes, {} for a no: here {} becomes {} or {}. If nobody \
         copied, a yes would mean {none}",
        model::likelihood_ratio(true),
        Q::ONE / model::likelihood_ratio(false),
        odds_text(odds(prior)?),
        odds_text(odds(yes)?),
        odds_text(odds(no)?),
    ))
}

/// One outcome as drawn: whose it is, what it says, its square now.
struct Outcome {
    copied: bool,
    said: bool,
    at: Vec3,
    mob: Mob,
}

/// Redraw the outcomes `which` picks in the colour `color` gives them
/// (fresh squares over the old), together.
fn recolor(
    stage: &mut Stage<'_>,
    outcomes: &mut [Outcome],
    which: impl Fn(&Outcome) -> bool,
    color: impl Fn(&Outcome) -> Srgb,
    rt: f64,
) -> fmn::Result<()> {
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for o in outcomes.iter_mut() {
        if !which(&*o) {
            continue;
        }
        let fresh = square(stage, color(&*o), o.at, SIDE)?;
        old.push(o.mob);
        new.push(fresh);
        o.mob = fresh;
    }
    swap(stage, &old, &new, rt)
}

/// The squares of the outcomes `which` picks.
fn squares(outcomes: &[Outcome], which: impl Fn(&Outcome) -> bool) -> Vec<Mob> {
    outcomes.iter().filter(|&o| which(o)).map(|o| o.mob).collect()
}

/// The drawn outcomes counted as `counts[said][copied]`.
fn tally(outcomes: &[Outcome]) -> [[u64; 2]; 2] {
    let mut c = [[0u64; 2]; 2];
    for o in outcomes {
        c[usize::from(o.said)][usize::from(o.copied)] += 1;
    }
    c
}

/// A mark on the chance line at `q` in `color`, and its fraction below.
fn mark(kit: &Kit, stage: &mut Stage<'_>, q: Q, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let x = chance_x(q);
    let tick = lane(stage, vec![v(x, LINE_Y - TICK), v(x, LINE_Y + TICK)], color);
    let label = kit.tex(stage, &tex_q(q, true), 30.0, color, &[], v(x, LINE_Y - MARK_DY))?;
    Ok([tick, label])
}

impl SceneConstruct for WhatAYesSays {
    fn name(&self) -> &str {
        "02_what_a_yes_says"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let a: Average = facts::average().map_err(fail)?;
        let (n, copiers) = (a.class.size(), a.class.copiers());
        let counted = model::class_outcomes(&a.class);
        let prior = Q::new(copiers as i128, n as i128);
        let given_yes = model::share_who_copied(&counted, true).ok_or_else(|| fail("no yes outcome".into()))?;
        let given_no = model::share_who_copied(&counted, false).ok_or_else(|| fail("no no outcome".into()))?;
        // the students as drawn: those who copied first, then the others
        let drawn: Vec<bool> = (0..n)
            .filter(|&i| a.class.copied(i))
            .chain((0..n).filter(|&i| !a.class.copied(i)))
            .map(|i| a.class.copied(i))
            .collect();
        let copier_x = group_x(0, copiers, copiers);
        let others_x = group_x(copiers, n, copiers);

        // 02_1: every student with every outcome, eighty in all
        let line = say(&kit, stage, "02_1")?;
        let mut top = vec![
            kit.text(stage, COPIED, 24.0, palette::copied(), v(copier_x, GROUP_Y))?,
            kit.text(stage, DIDNT, 24.0, palette::student(), v(others_x, GROUP_Y))?,
        ];
        let mut groups = [Vec::new(), Vec::new()];
        for (k, &copied) in drawn.iter().enumerate() {
            let at = v(column_x(k, copiers), STUDENT_Y);
            let d = disc(stage, palette::student(), at);
            top.push(d);
            groups[usize::from(copied)].push(d);
            if copied {
                top.push(ring(stage, at));
            }
        }
        fade_in_lagged(stage, &top, 1.2, 0.05)?;
        let mut faces = Vec::new();
        let mut outcomes = Vec::with_capacity(n * COIN_PAIRS.len());
        for (r, &(first, second)) in COIN_PAIRS.iter().enumerate() {
            let words = format!("{} {}", face(first), face(second));
            faces.push(kit.text(stage, &words, 20.0, Palette::dim(), v(FACE_X, row_y(r)))?);
            for (k, &copied) in drawn.iter().enumerate() {
                let at = v(column_x(k, copiers), row_y(r));
                outcomes.push(Outcome {
                    copied,
                    said: model::answer(copied, first, second),
                    at,
                    mob: square(stage, palette::ghost(), at, SIDE)?,
                });
            }
        }
        if tally(&outcomes) != counted {
            return Err(fail(format!(
                "the drawn outcomes count {:?}, the model {counted:?}",
                tally(&outcomes)
            )));
        }
        fade_in_all(stage, &faces, 0.4)?;
        fade_in_lagged(stage, &squares(&outcomes, |_| true), 1.6, 0.02)?;
        line.wait_for(stage, "Twenty students")?;
        let total = kit.tex(
            stage,
            &format!(r"{n} \times {} = {}", COIN_PAIRS.len(), outcomes.len()),
            36.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, TOTAL_Y),
        )?;
        fade_in_all(stage, &[total], 0.4)?;

        // 02_2: one student's four outcomes, for each truth
        let line = say(&kit, stage, "02_2")?;
        recolor(stage, &mut outcomes, |o| o.copied, |o| palette::answer(o.said), 0.5)?;
        let xs = [copier_x, others_x];
        let yes_labels = group_labels(&kit, stage, true, xs)?;
        fade_in_all(stage, &yes_labels[..1], 0.4)?;
        line.wait_for(stage, "Someone who didn't")?;
        recolor(stage, &mut outcomes, |o| !o.copied, |o| palette::answer(o.said), 0.5)?;
        fade_in_all(stage, &yes_labels[1..], 0.4)?;
        line.wait_for(stage, "Three times as likely")?;
        let times = kit.tex(
            stage,
            &format!(r"\times {}", model::likelihood_ratio(true)),
            40.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, LABEL_Y),
        )?;
        fade_in_all(stage, &[times], 0.4)?;

        // 02_3: a no, the reverse, three times again
        let _line = say(&kit, stage, "02_3")?;
        let no_labels = group_labels(&kit, stage, false, xs)?;
        swap(stage, &yes_labels, &no_labels, 0.4)?;
        flash_all(stage, &squares(&outcomes, |o| !o.said), 0.6)?;
        if model::likelihood_ratio(false) != model::likelihood_ratio(true) {
            return Err(fail(
                "the two answers' ratios differ; the shared × label would be wrong".into(),
            ));
        }
        flash_all(stage, &[times], 0.4)?;

        // 02_4: the prediction; nothing totals the yes outcomes yet
        let _line = say(&kit, stage, "02_4")?;
        let yes_labels = group_labels(&kit, stage, true, xs)?;
        swap(stage, &no_labels, &yes_labels, 0.3)?;
        fade_out_all(stage, &[total], 0.3)?;
        let q = think(&kit, stage, v(RIGHT_X, THINK_Y))?;

        // 02_5: count the yes outcomes, twelve and sixteen
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[q], 0.2)?;
        recolor(stage, &mut outcomes, |o| !o.said, |_| palette::ghost(), 0.5)?;
        let drawn_counts = tally(&outcomes);
        let [yes_didnt, yes_copied] = drawn_counts[1];
        line.wait_for(stage, "have twelve")?;
        flash_all(stage, &squares(&outcomes, |o| o.said && o.copied), 0.5)?;
        let twelve = kit.text(
            stage,
            &yes_copied.to_string(),
            30.0,
            palette::yes(),
            v(copier_x, LABEL_Y),
        )?;
        swap(stage, &yes_labels[..1], &[twelve], 0.3)?;
        line.wait_for(stage, "the sixteen others")?;
        flash_all(stage, &squares(&outcomes, |o| o.said && !o.copied), 0.5)?;
        let sixteen = kit.text(
            stage,
            &yes_didnt.to_string(),
            30.0,
            palette::yes(),
            v(others_x, LABEL_Y),
        )?;
        swap(stage, &yes_labels[1..], &[sixteen], 0.3)?;

        // 02_6: twelve of twenty-eight; three in seven, not three in four
        let line = say(&kit, stage, "02_6")?;
        let yes_total = Q::int(i128::from(yes_copied + yes_didnt));
        let share = Q::int(i128::from(yes_copied)) / yes_total;
        if share != given_yes {
            return Err(fail(format!("{share} drawn, {given_yes} counted")));
        }
        let raw = format!(r"\frac{{{yes_copied}}}{{{}}}", yes_copied + yes_didnt);
        // 12/28 first, then "= 3/7" beside it. The whole line is built now,
        // so the lone fraction can stand exactly where the line's own will
        // (the same TeX and size, their left edges together); the line then
        // fades in over it and the lone copy goes unseen, so the fraction is
        // never drawn twice side by side (review PinkDuck F3).
        let full = kit.tex(
            stage,
            &format!("{raw} = {}", tex_q(given_yes, false)),
            40.0,
            Palette::ink(),
            &[],
            v(DERIVE_X, DERIVE_Y),
        )?;
        let frac = kit.tex(stage, &raw, 40.0, Palette::ink(), &[], v(DERIVE_X, DERIVE_Y))?;
        let edge = stage.get_left(full);
        stage.move_to(frac, edge, LEFT);
        fade_in_all(stage, &[frac], 0.4)?;
        line.wait_for(stage, "On average")?;
        let avg = kit.text_left(
            stage,
            &average_note(&counted),
            20.0,
            Palette::ink(),
            v(NOTE_X, DERIVE_Y + NOTE_DY),
        )?;
        fade_in_all(stage, &[full, avg], 0.4)?;
        stage.remove_from_scene(frac);
        line.wait_for(stage, "not three in four")?;
        let tempt = kit.text_left(
            stage,
            &tempting_note(),
            20.0,
            Palette::dim(),
            v(NOTE_X, DERIVE_Y - NOTE_DY),
        )?;
        let posterior_text = posterior_card(&counted, n, copiers)?;
        let card = precisely(&kit, stage, &posterior_text)?;
        fade_in_all(stage, &[tempt, card], 0.4)?;
        let posterior_read_until = stage.scene().time().to_f64() + posterior_text.chars().count() as f64 / 15.0;

        // 02_7: three times the yeses each, but outnumbered four to one
        let line = say(&kit, stage, "02_7")?;
        let each = [
            kit.text(
                stage,
                &format!("{} each", model::outcomes_saying(true, true)),
                20.0,
                palette::yes(),
                v(copier_x, LABEL_Y - EACH_DY),
            )?,
            kit.text(
                stage,
                &format!("{} each", model::outcomes_saying(false, true)),
                20.0,
                palette::yes(),
                v(others_x, LABEL_Y - EACH_DY),
            )?,
        ];
        fade_in_all(stage, &each, 0.3)?;
        flash_all(stage, &each, 0.5)?;
        line.wait_for(stage, "outnumber")?;
        let named = [
            kit.text(
                stage,
                &format!("{copiers} {COPIED}"),
                24.0,
                palette::copied(),
                v(copier_x, GROUP_Y),
            )?,
            kit.text(
                stage,
                &format!("{} {DIDNT}", n - copiers),
                24.0,
                palette::student(),
                v(others_x, GROUP_Y),
            )?,
        ];
        swap(stage, &top[..2], &named, 0.3)?;
        flash_all(stage, &groups[1], 0.4)?;
        flash_all(stage, &groups[0], 0.5)?;

        // 02_8: a no, four of fifty-two, one in thirteen
        // Keep the explanation readable through the narrated comparison.
        wait_until(stage, posterior_read_until)?;
        let line = say(&kit, stage, "02_8")?;
        let mut leaving = vec![full, avg, tempt, card, twelve, sixteen];
        leaving.extend(each);
        fade_out_all(stage, &leaving, 0.4)?;
        recolor(stage, &mut outcomes, |o| o.said, |_| palette::ghost(), 0.4)?;
        recolor(stage, &mut outcomes, |o| !o.said, |_| palette::no(), 0.4)?;
        let [no_didnt, no_copied] = tally(&outcomes)[0];
        line.wait_for(stage, "Four of the fifty-two")?;
        flash_all(stage, &squares(&outcomes, |o| !o.said && o.copied), 0.5)?;
        let no_counts = [
            kit.text(stage, &no_copied.to_string(), 30.0, palette::no(), v(copier_x, LABEL_Y))?,
            kit.text(stage, &no_didnt.to_string(), 30.0, palette::no(), v(others_x, LABEL_Y))?,
        ];
        let no_share = Q::int(i128::from(no_copied)) / Q::int(i128::from(no_copied + no_didnt));
        if no_share != given_no {
            return Err(fail(format!("{no_share} drawn, {given_no} counted")));
        }
        let no_frac = kit.tex(
            stage,
            &format!(
                r"\frac{{{no_copied}}}{{{}}} = {}",
                no_copied + no_didnt,
                tex_q(given_no, false)
            ),
            40.0,
            Palette::ink(),
            &[],
            v(DERIVE_X, DERIVE_Y),
        )?;
        let mut shown = no_counts.to_vec();
        shown.push(no_frac);
        fade_in_all(stage, &shown, 0.4)?;

        // 02_9: the chance line, from one in five
        let line = say(&kit, stage, "02_9")?;
        fade_out_all(stage, &shown, 0.3)?;
        let axis = lane(
            stage,
            vec![v(LINE_X0, LINE_Y), v(LINE_X0 + LINE_W, LINE_Y)],
            Palette::dim(),
        );
        let zero = kit.text(
            stage,
            &tex_q(Q::ZERO, true),
            24.0,
            Palette::dim(),
            v(LINE_X0, LINE_Y - MARK_DY),
        )?;
        let one = kit.text(
            stage,
            &tex_q(Q::ONE, true),
            24.0,
            Palette::dim(),
            v(LINE_X0 + LINE_W, LINE_Y - MARK_DY),
        )?;
        let [prior_tick, prior_label] = mark(&kit, stage, prior, palette::copied())?;
        let caption = kit.text(
            stage,
            &format!("the class: {copiers} of {n}"),
            20.0,
            palette::copied(),
            v(chance_x(prior), LINE_Y - CAPTION_DY),
        )?;
        fade_in_all(stage, &[axis, zero, one, prior_tick, prior_label, caption], 0.5)?;
        let start = v(chance_x(prior), LINE_Y + TICK);
        line.wait_for(stage, "three in seven")?;
        let yes_mark = mark(&kit, stage, given_yes, palette::yes())?;
        let yes_arc = arc(
            stage,
            start,
            v(chance_x(given_yes), LINE_Y + TICK),
            YES_ARC,
            palette::yes(),
        );
        fade_in_all(stage, &[yes_arc, yes_mark[0], yes_mark[1]], 0.5)?;
        line.wait_for(stage, "one in thirteen")?;
        let no_mark = mark(&kit, stage, given_no, palette::no())?;
        let no_arc = arc(
            stage,
            start,
            v(chance_x(given_no), LINE_Y + TICK),
            NO_ARC,
            palette::no(),
        );
        fade_in_all(stage, &[no_arc, no_mark[0], no_mark[1]], 0.5)?;

        // 02_10: the coin fixes the factor; the class, where it starts
        let _line = say(&kit, stage, "02_10")?;
        flash_all(stage, &[yes_arc, no_arc], 0.6)?;
        let odds_text = odds_card(prior, given_yes, given_no, n)?;
        let card = precisely(&kit, stage, &odds_text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let odds_read_until = stage.scene().time().to_f64() + odds_text.chars().count() as f64 / 15.0;

        // 02_11: deniable, not meaningless; accuracy is another question
        wait_until(stage, odds_read_until)?;
        let line = say(&kit, stage, "02_11")?;
        line.wait_for(stage, "deniable")?;
        flash_all(stage, &yes_mark, 0.5)?;
        line.wait_for(stage, "not meaningless")?;
        flash_all(stage, &[yes_arc], 0.5)?;
        line.wait_for(stage, "How well")?;
        let names = names_to_search(&kit, stage, &NAMES)?;
        // Separate the two texts without changing the transition's duration.
        fade_out_all(stage, &[card], 0.2)?;
        fade_in_all(stage, &[names], 0.2)?;
        line.wait_end(stage)?;
        pause(stage, 2.0)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "Twenty students"),
            ("02_2", "Someone who didn't"),
            ("02_2", "Three times as likely"),
            ("02_5", "have twelve"),
            ("02_5", "the sixteen others"),
            ("02_6", "On average"),
            ("02_6", "not three in four"),
            ("02_7", "outnumber"),
            ("02_8", "Four of the fifty-two"),
            ("02_9", "three in seven"),
            ("02_9", "one in thirteen"),
            ("02_11", "deniable"),
            ("02_11", "not meaningless"),
            ("02_11", "How well"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen say what the facts say: one student's outcomes
    /// per answer, the average reading, the tempting 3/4 named for what it
    /// is, the half-class edge, and the odds.
    #[test]
    fn the_cards_are_built_from_the_facts() {
        assert_eq!(per_student(true, true), "yes: 3 of 4");
        assert_eq!(per_student(true, false), "yes: 1 of 4");
        assert_eq!(per_student(false, true), "no: 1 of 4");
        assert_eq!(per_student(false, false), "no: 3 of 4");
        let a = facts::average().unwrap();
        let c = model::class_outcomes(&a.class);
        assert_eq!(
            average_note(&c),
            "a quarter of the outcomes: 3 of chapter 1's 7 yes answers"
        );
        assert_eq!(tempting_note(), "3/4 is a copier's chance of saying yes");
        let card = posterior_card(&c, 20, 4).unwrap();
        assert!(
            card.starts_with("3/7 is the chance that someone who said yes copied"),
            "{card}"
        );
        assert!(
            card.contains("It would be 3/4 only if half the class, 10, had copied"),
            "{card}"
        );
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F12.posterior.yes_at_half"), Some("3/4"));
        let odds = odds_card(Q::new(1, 5), Q::new(3, 7), Q::new(1, 13), 20).unwrap();
        assert!(odds.contains("by 3 for a yes, 1/3 for a no"), "{odds}");
        assert!(odds.contains("1 : 4 becomes 3 : 4 or 1 : 12."), "{odds}");
        // the chances those odds are, as the chance line marks them
        assert_eq!(model::odds(Q::new(3, 7)), Some(Q::new(3, 4)));
        assert_eq!(model::odds(Q::new(1, 13)), Some(Q::new(1, 12)));
        assert!(odds.ends_with("a yes would mean 0"), "{odds}");
        assert_eq!(tex_q(Q::new(3, 7), false), r"\frac{3}{7}");
        assert_eq!(tex_q(Q::new(1, 13), true), r"\tfrac{1}{13}");
        assert_eq!(tex_q(Q::int(1), true), "1");
    }

    /// The cards stay two lines at the lower-third's width and at every
    /// narrower wrap it falls back to (it drops 4 characters at a time from
    /// 92 when a line overflows the room beside its label): at 80 the first
    /// odds card wrapped to three lines, the last a lone "0", and a
    /// three-line band would cover the chance line's caption (TanElm, Mail
    /// 3802).
    #[test]
    fn the_cards_are_two_lines() {
        let a = facts::average().unwrap();
        let c = model::class_outcomes(&a.class);
        for text in [
            posterior_card(&c, 20, 4).unwrap(),
            odds_card(Q::new(1, 5), Q::new(3, 7), Q::new(1, 13), 20).unwrap(),
        ] {
            for width in (72..=92).rev().step_by(4) {
                let lines = explainer_kit::animatic::wrap(&text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
        }
    }

    /// The layout keeps its pieces apart and inside the frame.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::RING_R;
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        let (n, copiers) = (model::CLASS_SIZE, model::COPIERS.len());
        // the columns: the groups apart, inside the frame, clear of the faces
        assert!(column_x(copiers, copiers) - column_x(copiers - 1, copiers) > COL_DX + 0.3);
        assert!(column_x(0, copiers) - SIDE / 2.0 > FACE_X + 0.3);
        assert!(column_x(n - 1, copiers) + SIDE / 2.0 < RIGHT_X - 0.9);
        const {
            assert!(
                FACE_X - 0.25 > -FRAME_X_RADIUS + 0.25,
                "the faces inside the title-safe edge"
            )
        };
        // the rows: under the students' rings, over the labels
        const {
            assert!(
                ROW_Y0 + SIDE / 2.0 < STUDENT_Y - RING_R - 0.1,
                "the squares under the rings"
            )
        };
        const { assert!(STUDENT_Y + RING_R < GROUP_Y - 0.25, "the rings under the group names") };
        assert!(row_y(COIN_PAIRS.len() - 1) - SIDE / 2.0 > LABEL_Y + 0.25);
        // squares one apart
        const { assert!(COL_DX > SIDE + 0.05 && ROW_DY > SIDE + 0.05, "squares apart") };
        // the think ring clears the ratio under it and the grid to its left
        assert!(column_x(n - 1, copiers) + SIDE / 2.0 < RIGHT_X - 0.62 - 0.1);
        const { assert!(THINK_Y - 0.62 - 0.45 > LABEL_Y + 0.25, "the note clears the ratio") };
        // the derivation and the chance line clear the band
        const { assert!(DERIVE_Y - 0.45 > BAND_TOP + 0.3, "the fraction over the band") };
        const { assert!(LINE_Y - CAPTION_DY - 0.12 > BAND_TOP, "the caption over the band") };
        const { assert!(LINE_X0 + LINE_W < FRAME_X_RADIUS - 0.5, "the line inside the frame") };
        // the marks' fractions apart on the line: 0, 1/13, 1/5, 3/7, 1
        let xs: Vec<f64> = [Q::ZERO, Q::new(1, 13), Q::new(1, 5), Q::new(3, 7), Q::ONE]
            .iter()
            .map(|&q| chance_x(q))
            .collect();
        for pair in xs.windows(2) {
            assert!(pair[1] - pair[0] > 0.55, "{pair:?}");
        }
    }
}
