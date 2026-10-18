//! Chapter 01, five percent of what: the residents' survey as the cold open
//! (01_1, 01_2), the title (01_3), a positive result and the lab's five
//! percent (01_4), the tempting ninety-five as a prediction (01_5); then ten
//! thousand people counted, one row with the disease (01_6), ninety of them
//! caught (01_7), four hundred ninety-five false alarms among the rest
//! (01_8); the positives gathered into one block (01_9), whose two gold rows
//! of thirteen are the answer, about fifteen percent (01_10); and the two
//! fractions side by side, each with its group outlined (01_11).
//!
//! The people are an icon array of row strips (`RowStrips`, one mobject a
//! row, recoloured in place by one updater when a tracker moves); each
//! positive result is a tile of its own laid over its cell, so the same
//! object that lit up is the one that flies into the block.

use std::rc::Rc;

use explainer_kit::props::{RowStrips, precisely};
use explainer_kit::{Kit, Palette, Timed, dot_cloud, play, rect, stroke_style, thousands, tween, v};
use fmn::core::color::color_gradient;
use fmn::prelude::*;

use super::draw::{
    card, chip, disease, disease_dark, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, group, healthy,
    healthy_dark, line_style, person, say, survey, think_pause, vacated, warn,
};
use super::glyphs::TITLE;
use crate::facts::{GRID_COLS, cited};
use crate::model::{IconArray, Q, clinic_a, positives_block};

pub struct FivePercentOfWhat {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const PUBLISHED_IN: &str = "published in JAMA,";
const COLLECTED: &str = "data collected";
const PROGRAMS_TAG: &str = "internal-medicine residency programs, Connecticut";
const ANSWERED: &str = "residents answered";
const CONFIDENT: &str = "fair to complete confidence about p-values";
const KEYED: &str = "picked the answer marked correct, choice";
const AS_REPORTED: &str = "shares of the residents who answered, as reported";
const NO_CHOICE: &str = "none of the four choices is correct";
const RESULT: &str = "test result: POSITIVE";
const THE_LAB: &str = "the lab says:";
const FALSE_ALARM_FOR: &str = "a false alarm for";
const OF_PEOPLE_WITHOUT: &str = "of people without the disease";
pub(super) const PEOPLE_MODEL: &str = "people (model)";
const HAVE_IT: &str = "have the disease,";
const IN: &str = "in";
const OF_THEM_POSITIVE: &str = "of them test positive";
const DONT_HAVE_IT: &str = "don't have the disease";
const FALSE_ALARMS: &str = "test positive anyway: false alarms";
pub(super) const WITH_THE_DISEASE: &str = "with the disease";
pub(super) const WITHOUT_THE_DISEASE: &str = "without the disease";
pub(super) const ROWS_OF: &str = "rows of";
pub(super) const SHARE_A: [&str; 2] = ["positive, among people", "without the disease"];
pub(super) const SHARE_B: [&str; 2] = ["the disease, among people", "who tested positive"];
const NOT_A_P_VALUE: &str = "Neither number is a p-value.";

/// The title card's listen window (storyboard 01_3: listen 2 s).
pub const TITLE_S: f64 = 2.0;
/// The listen label of the title card, the score's open motif.
pub const TITLE_LABEL: &str = "01_five_percent_of_what:01_3";

// --- layout (scene units; the frame is 14.2 by 8)

/// The residents: one dot each, in rows of 23, centred here.
const DOTS_AT: Vec3 = [-3.9, 0.4, 0.0];
const DOTS_COLS: usize = 23;
const DOTS_STEP: f64 = 0.22;
const DOT_R: f64 = 0.065;
/// The survey's bars, left edge and size.
const BAR_LEFT: f64 = 0.7;
const BAR_W: f64 = 5.0;
const BAR_H: f64 = 0.34;
const BAR_Y: [f64; 2] = [1.6, 0.3];
/// The icon array: top-left corner and cell size (100 by 100 cells).
pub(super) const GRID_X0: f64 = -6.7;
pub(super) const GRID_Y0: f64 = 2.8;
pub(super) const GC: f64 = 0.054;
/// The ledger beside the array: numbers right-aligned here, words after.
const LEDGER_NUM: f64 = 1.6;
const LEDGER_WORDS: f64 = 1.85;
const LEDGER_Y: [f64; 4] = [2.4, 1.8, 0.8, 0.2];
/// The block of positives: top-left corner and cell size.
pub(super) const BLOCK_X0: f64 = 0.9;
pub(super) const BLOCK_Y0: f64 = 1.9;
pub(super) const BC: f64 = 0.12;
/// The tempting answer's place once it has been asked, and its size there
/// (64-point type scaled by 0.45).
pub(super) const CORNER: Vec3 = [6.2, 3.35, 0.0];
pub(super) const CORNER_SCALE: f64 = 0.45;
pub(super) const ASK_SIZE: f64 = 64.0;
/// The fractions' row.
pub(super) const FRACTION_Y: f64 = -1.0;
pub(super) const FRACTION_X: [f64; 2] = [1.9, 5.1];
/// How many steps a colour crossfade of the array takes.
pub(super) const RAMP: usize = 17;

fn dot_at(i: usize) -> Vec3 {
    let (row, col) = ((i / DOTS_COLS) as f64, (i % DOTS_COLS) as f64);
    let rows = (cited::RESIDENTS as usize).div_ceil(DOTS_COLS) as f64;
    v(
        DOTS_AT[0] + (col - (DOTS_COLS as f64 - 1.0) / 2.0) * DOTS_STEP,
        DOTS_AT[1] + ((rows - 1.0) / 2.0 - row) * DOTS_STEP,
    )
}

/// The centre of the array's cell (row, col).
fn cell_at(row: usize, col: usize) -> Vec3 {
    v(GRID_X0 + GC * (col as f64 + 0.5), GRID_Y0 - GC * (row as f64 + 0.5))
}

/// The centre of the block's place `j`, row by row.
pub(super) fn block_at(j: u64, cols: u64) -> Vec3 {
    let (row, col) = ((j / cols) as f64, (j % cols) as f64);
    v(BLOCK_X0 + BC * (col + 0.5), BLOCK_Y0 - BC * (row + 0.5))
}

/// An outline around a rectangle of the scene, `(left, top)` to
/// `(right, bottom)`, in the "this group" role.
pub(super) fn outline(stage: &mut Stage<'_>, (l, t): (f64, f64), (r, b): (f64, f64)) -> fmn::Result<Mob> {
    let o = Rectangle::new()
        .width(r - l)
        .height(t - b)
        .style(stroke_style(Palette::ink(), 3.0))
        .build()?
        .moved_to(v((l + r) / 2.0, (t + b) / 2.0));
    Ok(stage.arena_mut().add(o))
}

/// The array's colours: unknown grey crossfading to dark gold (the disease)
/// or dark blue by `(disease step, no-disease step)`; a positive's cell goes
/// dark once its tile has left.
pub(super) fn grid_colors(
    array: &IconArray,
    gold: &[Srgb],
    blue: &[Srgb],
    (d, h, left): (usize, usize, bool),
) -> Vec<Vec<Srgb>> {
    (0..array.rows)
        .map(|r| {
            (0..array.cols)
                .map(|c| {
                    let p = array.at(r, c);
                    if left && p.positive {
                        vacated()
                    } else if p.disease {
                        gold[d]
                    } else {
                        blue[h]
                    }
                })
                .collect()
        })
        .collect()
}

pub(super) fn ramp_step(t: f64) -> usize {
    (t.clamp(0.0, 1.0) * (RAMP - 1) as f64).round() as usize
}

/// A survey bar: its words above, the whole dim, the share growing from the
/// left in teal, the printed value after it.
fn survey_bar(kit: &Kit, stage: &mut Stage<'_>, y: f64, words: &str, share: Q, value: &str) -> fmn::Result<()> {
    let label = kit.text_left(stage, words, 24.0, Palette::ink(), v(BAR_LEFT, y + 0.45))?;
    let dl = v(BAR_LEFT, y - BAR_H / 2.0);
    let whole = stage
        .arena_mut()
        .add(rect(dl, BAR_W, BAR_H, flat(Palette::faint(), 0.8))?);
    let part = stage
        .arena_mut()
        .add(rect(dl, BAR_W * share.f64(), BAR_H, flat(survey(), 0.95))?);
    fade_in_all(stage, &[label, whole], 0.4)?;
    let grow = fmn::animation::grow_from_edge(stage.arena_mut(), part, LEFT, None)?;
    stage.play(grow.rt(1.0))?;
    let value = kit.text_left(stage, value, 28.0, survey(), v(BAR_LEFT + BAR_W + 0.15, y))?;
    fade_in_all(stage, &[value], 0.3)
}

/// A fraction in TeX with its value in text after it (`= 5%`), the pair
/// centred on `x`; one group, detached.
pub(super) fn fraction(
    kit: &Kit,
    stage: &mut Stage<'_>,
    tex: &str,
    top: Srgb,
    value: &str,
    x: f64,
) -> fmn::Result<Mob> {
    let numerator = tex
        .strip_prefix(r"\frac{")
        .and_then(|t| t.split('}').next())
        .ok_or_else(|| fail(format!("{tex} is not a fraction")))?
        .to_owned();
    let f = kit.tex(
        stage,
        tex,
        44.0,
        Palette::ink(),
        &[(numerator.as_str(), top)],
        v(0.0, FRACTION_Y),
    )?;
    let t = kit.text(stage, value, 32.0, Palette::ink(), v(0.0, FRACTION_Y))?;
    let (wf, wt) = (stage.get_width(f), stage.get_width(t));
    let left = x - (wf + 0.15 + wt) / 2.0;
    stage.move_to(f, v(left, FRACTION_Y), LEFT);
    stage.move_to(t, v(left + wf + 0.15, FRACTION_Y), LEFT);
    group(stage, &[f, t])
}

/// Two lines of words under a fraction at `x`.
pub(super) fn under(kit: &Kit, stage: &mut Stage<'_>, lines: [&str; 2], color: Srgb, x: f64) -> fmn::Result<Mob> {
    let a = kit.text(stage, lines[0], 22.0, color, v(x, FRACTION_Y - 0.75))?;
    let b = kit.text(stage, lines[1], 22.0, color, v(x, FRACTION_Y - 1.07))?;
    group(stage, &[a, b])
}

impl FivePercentOfWhat {
    /// 01_1 and 01_2: the survey, its confidence and its keyed answer.
    fn survey(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        use cited::*;
        let kit = &self.kit;
        let line = say(kit, stage, "01_1")?;
        let when = kit.text(
            stage,
            &format!("{PUBLISHED_IN} {PUBLISHED_YEAR} · {COLLECTED} {COLLECTED_MONTHS} {COLLECTED_YEAR}"),
            22.0,
            Palette::dim(),
            v(0.0, 3.55),
        )?;
        let tag = kit.text(
            stage,
            &format!("{PROGRAMS} {PROGRAMS_TAG}"),
            22.0,
            Palette::dim(),
            v(0.0, 3.22),
        )?;
        let everyone: Vec<Vec3> = (0..RESIDENTS as usize).map(dot_at).collect();
        let faint = vec![Palette::faint(); everyone.len()];
        let all = stage.arena_mut().add(dot_cloud(&everyone, &faint, DOT_R));
        fade_in_all(stage, &[when, tag, all], 0.6)?;
        let answered: Vec<Mob> = (0..RESPONDENTS as usize)
            .map(|i| {
                let d = Dot::new()
                    .point(dot_at(i))
                    .radius(DOT_R)
                    .style(flat(survey(), 1.0))
                    .build();
                stage.arena_mut().add(d)
            })
            .collect();
        line.wait_for(stage, "two hundred seventy-seven")?;
        fade_in_lagged(stage, &answered, 2.2, 0.02)?;
        let response = format!(
            "{RESPONDENTS} of {RESIDENTS} {ANSWERED} ({}%)",
            Q::ratio(RESPONDENTS, RESIDENTS).percent(1)
        );
        let count = kit.text(stage, &response, 24.0, Palette::dim(), v(DOTS_AT[0], DOTS_AT[1] - 2.1))?;
        fade_in_all(stage, &[count], 0.5)?;
        line.wait_for(stage, "Eighty-eight percent")?;
        let confident = Q::new(i128::from(CONFIDENT_PERCENT), 100);
        survey_bar(
            kit,
            stage,
            BAR_Y[0],
            CONFIDENT,
            confident,
            &format!("{CONFIDENT_PERCENT}%"),
        )?;

        let line = say(kit, stage, "01_2")?;
        line.wait_for(stage, "fifty-nine percent")?;
        let keyed = Q::new(KEYED_PERMILLE, 1000);
        survey_bar(
            kit,
            stage,
            BAR_Y[1],
            &format!("{KEYED} {KEYED_CHOICE}"),
            keyed,
            &format!("{}%", Q::new(KEYED_PERMILLE, 10).decimal(1)),
        )?;
        let note = kit.text_left(stage, AS_REPORTED, 20.0, Palette::dim(), v(BAR_LEFT, BAR_Y[1] - 0.45))?;
        line.wait_for(stage, "A later paper")?;
        let head = format!("choice {KEYED_CHOICE}, the answer marked correct for {KEYED_FOR}:");
        let foot = format!("as reprinted by Gao {GAO_YEAR}");
        let quote = card(
            kit,
            stage,
            Some(head.as_str()),
            &format!("“{KEYED_TEXT}”"),
            Some(foot.as_str()),
            (22.0, 46),
            Palette::ink(),
            6.4,
            v(3.4, -1.75),
        )?;
        fade_in_all(stage, &[note, quote], 0.6)?;
        line.wait_for(stage, "even that answer")?;
        let dispute = chip(
            kit,
            stage,
            &format!("Gao {GAO_YEAR}: {NO_CHOICE}"),
            22.0,
            warn(),
            v(3.4, -3.2),
        )?;
        fade_in_all(stage, &[dispute], 0.5)?;
        kit.hold(stage)
    }

    /// 01_3: the title, in the score's open motif. The survey crossfades
    /// into it (one 0.6 s move, not a wipe and then a fade).
    fn title(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let leaving: Vec<Mob> = stage.scene().mobjects().to_vec();
        let t = self.kit.text(stage, TITLE, 48.0, Palette::hero(), v(0.0, 0.2))?;
        let mut cross: Vec<Box<dyn Animation>> = Vec::new();
        for m in leaving {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            cross.push(stage.prepare(a.rt(0.6))?);
        }
        let a = fade_in(stage.arena_mut(), t, ORIGIN, 1.0)?;
        cross.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(cross)?;
        self.kit.listen(stage, TITLE_S, TITLE_LABEL)?;
        fade_out_all(stage, &[t], 0.4)
    }
}

impl SceneConstruct for FivePercentOfWhat {
    fn name(&self) -> &str {
        "01_five_percent_of_what"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let clinic = clinic_a();
        let c = clinic.counts().map_err(fail)?;
        let array = IconArray::build(&c, GRID_COLS).map_err(fail)?;
        let block = positives_block(&c).map_err(fail)?;
        let five = clinic.false_alarm.percent(0);

        self.survey(stage)?;
        self.title(stage)?;

        // 01_4: a positive result, and the lab's five percent
        let line = say(&kit, stage, "01_4")?;
        let result = chip(&kit, stage, RESULT, 30.0, Palette::ink(), v(-3.0, 1.4))?;
        fade_in_all(stage, &[result], 0.5)?;
        line.wait_for(stage, "It gives a false alarm")?;
        let rule = card(
            &kit,
            stage,
            Some(THE_LAB),
            &format!("{FALSE_ALARM_FOR} {five}% {OF_PEOPLE_WITHOUT}"),
            None,
            (28.0, 26),
            healthy(),
            5.4,
            v(2.9, 1.4),
        )?;
        fade_in_all(stage, &[rule], 0.5)?;

        // 01_5: the tempting answer, then time to think
        say(&kit, stage, "01_5")?;
        let tempting = clinic.false_alarm.complement();
        let ask_at = v(0.0, -1.0);
        let ask = kit.text(
            stage,
            &format!("{}%?", tempting.percent(0)),
            ASK_SIZE,
            Palette::ink(),
            ask_at,
        )?;
        fade_in_all(stage, &[ask], 0.5)?;
        think_pause(&kit, stage, ask_at, 1.3)?;

        // 01_6: ten thousand people, one row of them with the disease
        let line = say(&kit, stage, "01_6")?;
        let to_corner = ask
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .scale(CORNER_SCALE)?
            .move_to(CORNER, ORIGIN)?;
        let mut moves: Vec<Box<dyn Animation>> = vec![stage.prepare(to_corner)?];
        for m in [result, rule] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(moves)?;
        line.wait_for(stage, "Ten thousand")?;
        let (gold, blue) = (
            color_gradient(&[person(), disease_dark()], RAMP),
            color_gradient(&[person(), healthy_dark()], RAMP),
        );
        let strips = Rc::new(RowStrips::build(
            stage,
            v(GRID_X0, GRID_Y0),
            (GC, GC),
            &grid_colors(&array, &gold, &blue, (0, 0, false)),
        )?);
        let (sick_t, well_t, left_t) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let painter = stage.arena_mut().add(VMobject::new());
        let (paint, cells) = (Rc::clone(&strips), array.clone());
        let mut last = Some((0, 0, false));
        stage.add_updater(
            painter,
            move |s, _| {
                let now = (
                    ramp_step(s.tracker_value(sick_t).unwrap_or(0.0)),
                    ramp_step(s.tracker_value(well_t).unwrap_or(0.0)),
                    s.tracker_value(left_t).unwrap_or(0.0) >= 0.5,
                );
                if last == Some(now) {
                    return;
                }
                if paint.recolor(s, &grid_colors(&cells, &gold, &blue, now)) {
                    last = Some(now);
                }
            },
            true,
        )?;
        let grid_mid = GRID_X0 + GC * GRID_COLS as f64 / 2.0;
        let people_label = kit.text(
            stage,
            &format!("{} {PEOPLE_MODEL}", thousands(c.people)),
            24.0,
            Palette::dim(),
            v(grid_mid, GRID_Y0 + 0.35),
        )?;
        fade_in_all(stage, &[strips.group, painter, people_label], 0.9)?;
        line.wait_for(stage, "one in a hundred")?;
        let n_with = kit.text(stage, &c.with.to_string(), 30.0, disease(), v(0.0, LEDGER_Y[0]))?;
        stage.move_to(n_with, v(LEDGER_NUM, LEDGER_Y[0]), RIGHT);
        let (some, of) = (clinic.prevalence.num(), clinic.prevalence.den());
        let w_with = ledger(&kit, stage, 0, &format!("{HAVE_IT} {some} {IN} {of}"), disease())?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(sick_t, 1.0, 0.8)?)?];
        for m in [n_with, w_with] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;

        // 01_7: the disease row, and the ninety the test catches
        let line = say(&kit, stage, "01_7")?;
        let grid_right = GRID_X0 + GC * array.cols as f64;
        let row = outline(
            stage,
            (GRID_X0 - 0.03, GRID_Y0 + 0.03),
            (grid_right + 0.03, GRID_Y0 - GC - 0.03),
        )?;
        kit.allow_crossing(row);
        fade_in_all(stage, &[row], 0.4)?;
        // the tiles start with "Say the test" so the count reaches ninety as
        // the word is said and the row is lit before the line ends
        line.wait_for(stage, "Say the test")?;
        let mut caught = Vec::new();
        for r in 0..array.disease_rows {
            for col in 0..array.cols {
                if array.at(r, col).positive {
                    caught.push(tile(stage, r, col, disease())?);
                }
            }
        }
        let mut tiles = caught.clone();
        let caught_n = stage.add_value_tracker(0.0);
        let n_caught = kit.live_number(
            stage,
            caught_n,
            0,
            false,
            30.0,
            disease(),
            v(LEDGER_NUM, LEDGER_Y[1]),
            RIGHT,
        )?;
        let w_caught = ledger(&kit, stage, 1, OF_THEM_POSITIVE, disease())?;
        fade_in_all(stage, &[w_caught], 0.3)?;
        let mut both: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(caught_n, c.true_pos as f64, 1.8)?)?];
        both.push(Box::new(lagged(stage, &caught, 1.8, 0.03)?));
        stage.play_prepared(both)?;
        fade_out_all(stage, &[row], 0.3)?;

        // 01_8: the others, and their five percent of false alarms
        let line = say(&kit, stage, "01_8")?;
        let n_without = kit.text(stage, &thousands(c.without), 30.0, healthy(), v(0.0, LEDGER_Y[2]))?;
        stage.move_to(n_without, v(LEDGER_NUM, LEDGER_Y[2]), RIGHT);
        let w_without = ledger(&kit, stage, 2, DONT_HAVE_IT, healthy())?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(well_t, 1.0, 0.8)?)?];
        for m in [n_without, w_without] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "Five percent of them")?;
        let mut alarms = Vec::new();
        for r in array.disease_rows..array.rows {
            for col in 0..array.cols {
                if array.at(r, col).positive {
                    alarms.push(tile(stage, r, col, healthy())?);
                }
            }
        }
        tiles.extend(&alarms);
        let alarm_n = stage.add_value_tracker(0.0);
        let n_alarms = kit.live_number(
            stage,
            alarm_n,
            0,
            false,
            30.0,
            healthy(),
            v(LEDGER_NUM, LEDGER_Y[3]),
            RIGHT,
        )?;
        let w_alarms = ledger(&kit, stage, 3, FALSE_ALARMS, healthy())?;
        fade_in_all(stage, &[w_alarms], 0.3)?;
        let mut both: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(alarm_n, c.false_pos as f64, 2.6)?)?];
        both.push(Box::new(lagged(stage, &alarms, 2.6, 0.01)?));
        stage.play_prepared(both)?;
        if tiles.len() as u64 != c.positives() {
            return Err(fail(format!("{} tiles for {} positives", tiles.len(), c.positives())));
        }

        // 01_9: every positive leaves the array for one block
        let line = say(&kit, stage, "01_9")?;
        let ledger_mobs = [
            n_with, w_with, n_caught, w_caught, n_without, w_without, n_alarms, w_alarms,
        ];
        fade_out_all(stage, &ledger_mobs, 0.4)?;
        // the cells under the tiles go dark now, so the tiles leave holes
        play!(stage; tween(left_t, 1.0, 0.05)?);
        line.wait_for(stage, "gather")?;
        stage.scene_mut().bring_to_front(&tiles)?;
        let grow = BC * 0.86 / GC;
        let mut flights: Vec<Box<dyn Animation>> = Vec::new();
        for (j, &piece) in tiles.iter().enumerate() {
            let a = piece
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .scale(grow)?
                .move_to(block_at(j as u64, block.cols), ORIGIN)?;
            flights.push(stage.prepare(a)?);
        }
        let flight = AnimationGroup::with_lag_ratio(stage.arena_mut(), flights, 0.004)?.with_run_time(2.4);
        stage.play(flight)?;

        // 01_10: two rows of thirteen have the disease: about fifteen percent
        let line = say(&kit, stage, "01_10")?;
        line.wait_for(stage, "Ninety have")?;
        let block_right = BLOCK_X0 + BC * block.cols as f64;
        let block_bottom = BLOCK_Y0 - BC * block.rows as f64;
        let with_words = kit.text_left(
            stage,
            &format!("{} {WITH_THE_DISEASE}", c.true_pos),
            24.0,
            disease(),
            v(BLOCK_X0, BLOCK_Y0 + 0.3),
        )?;
        fade_in_all(stage, &[with_words], 0.5)?;
        line.wait_for(stage, "four hundred ninety-five")?;
        let without_words = kit.text_left(
            stage,
            &format!("{} {WITHOUT_THE_DISEASE}", c.false_pos),
            24.0,
            healthy(),
            v(BLOCK_X0, block_bottom - 0.3),
        )?;
        fade_in_all(stage, &[without_words], 0.5)?;
        line.wait_for(stage, "only about fifteen percent")?;
        let ppv = Q::ratio(c.true_pos, c.positives());
        let centre = (BLOCK_X0 + block_right) / 2.0;
        let answer = fraction(
            &kit,
            stage,
            &format!(r"\frac{{{}}}{{{}}}", c.true_pos, c.positives()),
            disease(),
            &format!("≈ {}%", ppv.percent(0)),
            centre,
        )?;
        let rows_note = kit.text(
            stage,
            &format!("{} {ROWS_OF} {}", block.disease_rows, block.rows),
            22.0,
            Palette::dim(),
            v(centre, FRACTION_Y - 0.75),
        )?;
        let w = stage.get_width(ask);
        let strike = polyline(vec![
            v(CORNER[0] - w / 2.0 - 0.08, CORNER[1]),
            v(CORNER[0] + w / 2.0 + 0.08, CORNER[1]),
        ])
        .style(line_style(warn(), 4.0))
        .build();
        let strike = stage.arena_mut().add(strike);
        kit.allow_crossing(strike);
        fade_in_all(stage, &[answer, rows_note, strike], 0.6)?;

        // 01_11: the two fractions, each with its group
        let line = say(&kit, stage, "01_11")?;
        let slide = answer
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .move_to(v(FRACTION_X[1], FRACTION_Y), ORIGIN)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(slide)?];
        let gone_note = fade_out(stage.arena_mut(), rows_note, ORIGIN, 1.0)?;
        anims.push(stage.prepare(gone_note.rt(0.4))?);
        stage.play_prepared(anims)?;
        let first = fraction(
            &kit,
            stage,
            &format!(
                r"\frac{{{}}}{{{}}}",
                c.false_pos,
                thousands(c.without).replace(',', "{,}")
            ),
            healthy(),
            &format!("= {five}%"),
            FRACTION_X[0],
        )?;
        let first_words = under(&kit, stage, SHARE_A, healthy(), FRACTION_X[0])?;
        let below_disease = BLOCK_Y0 - BC * block.disease_rows as f64;
        let groups_a = [
            outline(
                stage,
                (GRID_X0 - 0.03, GRID_Y0 - GC),
                (grid_right + 0.03, GRID_Y0 - GC * array.rows as f64 - 0.03),
            )?,
            outline(
                stage,
                (BLOCK_X0 - 0.05, below_disease),
                (block_right + 0.05, block_bottom - 0.05),
            )?,
        ];
        for &g in &groups_a {
            kit.allow_crossing(g);
        }
        let mut ins = vec![first, first_words];
        ins.extend(groups_a);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "Your question")?;
        let second_words = under(&kit, stage, SHARE_B, disease(), FRACTION_X[1])?;
        let group_b = outline(
            stage,
            (BLOCK_X0 - 0.05, BLOCK_Y0 + 0.05),
            (block_right + 0.05, block_bottom - 0.05),
        )?;
        kit.allow_crossing(group_b);
        // the exact version arrives with the second group, so it is read
        // while the line finishes rather than after it
        let exact = format!(
            "in this model clinic, P(positive | no disease) = {} and P(disease | positive) = {} ≈ {}%. {NOT_A_P_VALUE}",
            clinic.false_alarm,
            ppv,
            ppv.percent(2)
        );
        let lower = precisely(&kit, stage, &exact)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &g in &groups_a {
            let a = fade_out(stage.arena_mut(), g, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        for m in [second_words, group_b, lower] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        // chapter 02 opens on this picture as it rests: only the lower-third goes
        fade_out_all(stage, &[lower], 0.4)?;
        kit.finish(stage)
    }
}

/// One line of the ledger beside the array: its words, after the number.
fn ledger(kit: &Kit, stage: &mut Stage<'_>, i: usize, words: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.text_left(stage, words, 24.0, color, v(LEDGER_WORDS, LEDGER_Y[i]))
}

/// A positive result: a tile of its own over the array's cell (row, col),
/// detached, that will later fly into the block.
fn tile(stage: &mut Stage<'_>, row: usize, col: usize, color: Srgb) -> fmn::Result<Mob> {
    let at = cell_at(row, col);
    let t = rect(v(at[0] - GC / 2.0, at[1] - GC / 2.0), GC, GC, flat(color, 1.0))?;
    Ok(stage.arena_mut().add(t))
}

/// Prepared fade-ins of `mobs` as one lagged group of `rt` seconds, to play
/// beside another animation.
fn lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<AnimationGroup> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    Ok(AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout's numbers are the facts': the dots are the residents, the
    /// array is the clinic, the block holds every positive.
    #[test]
    fn the_layout_holds_the_counts() {
        let c = clinic_a().counts().unwrap();
        let array = IconArray::build(&c, GRID_COLS).unwrap();
        let block = positives_block(&c).unwrap();
        assert_eq!(array.cols * array.rows, 10_000);
        assert_eq!(block.cols * block.rows, c.positives());
        assert_eq!(cited::RESIDENTS, 367, "dot_at lays out 367 dots");
        // the dots stay inside the left half and above the count beneath
        for i in 0..cited::RESIDENTS as usize {
            let p = dot_at(i);
            assert!(
                p[0] > -7.0 && p[0] < -0.6 && p[1] > DOTS_AT[1] - 1.8 && p[1] < 2.3,
                "{i}: {p:?}"
            );
        }
        // the array and the block do not overlap, and the block clears the fractions
        let grid_right = GRID_X0 + GC * array.cols as f64;
        assert!(grid_right < BLOCK_X0);
        assert!(BLOCK_X0 + BC * block.cols as f64 <= 6.9);
        assert!(BLOCK_Y0 - BC * block.rows as f64 - 0.3 > FRACTION_Y + 0.6);
        // the array sits above the lower-third's band
        assert!(GRID_Y0 - GC * array.rows as f64 > -2.75);
    }

    #[test]
    fn the_title_window_is_the_storyboards() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let beat = board[0].beats.iter().find(|b| b.id == "01_3").unwrap();
        assert_eq!(beat.listen_s(), Some(TITLE_S));
        assert_eq!(TITLE_LABEL, format!("{}:{}", board[0].key, beat.id));
    }
}
