//! Chapter 02, the same test in another clinic: chapter 01's last picture
//! gathers into the left half as clinic A while clinic B, where one person
//! in ten has the disease, appears on the right (02_1); the test's two rates
//! are the same in both (02_2); a prediction (02_3); clinic B counted, a
//! thousand with the disease and nine hundred caught (02_4), four hundred
//! fifty false alarms among the other nine thousand (02_5); its positives
//! gathered into a block as wide as clinic A's, twenty rows of thirty, two
//! in three (02_6); the two answers side by side (02_7) and why they differ
//! (02_8); the bridge on a cleared stage, a false-alarm rate is not the
//! chance a positive result is right and neither is a p-value (02_9); and
//! the study of the next chapters, twenty pairs and no result yet (02_10).
//!
//! Both clinics are the model's (`model::clinic_a`, `clinic_b`), counted
//! person by person; every number on screen is built from those counts.

use std::rc::Rc;

use explainer_kit::props::RowStrips;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, rect, thousands, tween, v};
use fmn::core::color::color_gradient;
use fmn::prelude::*;

use super::draw::{
    card, chip, disease, disease_dark, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, group, healthy,
    healthy_dark, line_style, panel, person, say, think_pause, warn,
};
use super::five_percent::{
    ASK_SIZE, BC, BLOCK_X0, BLOCK_Y0, CORNER, CORNER_SCALE, FRACTION_X, GC, GRID_X0, GRID_Y0, PEOPLE_MODEL, RAMP,
    ROWS_OF, SHARE_A, SHARE_B, WITH_THE_DISEASE, WITHOUT_THE_DISEASE, block_at, fraction, grid_colors, outline,
    ramp_step, under,
};
use crate::facts::{BLOCK_COLS, GRID_COLS};
use crate::model::{Clinic, Counts, IconArray, PositivesBlock, Q, clinic_a, clinic_b, positives_block_of, study};

pub struct TheSameTest {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const CLINIC: &str = "clinic";
const HAS_IT: &str = "has the disease";
const IN: &str = "in";
const THE_SAME_TEST: &str = "the same test in both clinics (assumed): catches";
const WITH_IT: &str = "with the disease · a false alarm for";
const WITHOUT_IT: &str = "of people without it";
const LEDGER_WORDS: [&str; 4] = [
    "have the disease",
    "of them test positive",
    "don't have the disease",
    "false alarms",
];
const FALSE_ALARM_RATE: &str = "false-alarm rate";
const POSITIVES_WITH_IT: &str = "positives with the disease";
const TIMES: &str = "×";
const PPV: &str = "positive predictive value";
const P_FALSE_ALARM: &str = "P(positive | no disease) =";
const P_PPV: &str = "P(disease | positive) =";
const IN_CLINIC: &str = "in clinic";
const SAME_IN_BOTH: &str = "the same in both clinics";
const MOVES: &str = "moves with the clinic";
const NEITHER: &str = "Neither is a p-value.";
pub(super) const STUDY_A: &str = "a synthetic study:";
pub(super) const STUDY_B: &str = "pairs of patients, a coin in each pair picks who is treated (model)";
pub(super) const NULL_WORLD: &str = "imagine the treatment changed no one's recovery";

// what 02_8 and 02_9's pictures land on: 02_8 asks its whole question, then
// answers it (the test below holds each phrase to its line and sentence)
const ALARMS_AT: &str = "The false alarms";
const TEN_TIMES_AT: &str = "ten times as many";
const NEITHER_AT: &str = "neither one";

// --- layout (scene units; the frame is 14.2 by 8)

/// The two clinics' halves: their left edges.
const PANEL_X0: [f64; 2] = [-6.9, 0.2];
/// The clinic's name above each half.
const HEADER_Y: f64 = 3.45;
/// Each half's icon array: its top and cell size (100 by 100 cells).
const ARRAY_Y0: f64 = 3.0;
const AC: f64 = 0.028;
/// Each half's block of positives: right of its array, its top level with
/// the array's, cells this wide (45 a row in both).
const BLOCK_GAP: f64 = 0.3;
const PC: f64 = 0.062;
/// The test's card across the middle.
const CARD_Y: f64 = -0.15;
/// Each half's ledger: numbers right-aligned at the first offset from the
/// half's left edge, words at the second, one line each.
const LEDGER_NUM: f64 = 1.15;
const LEDGER_TEXT: f64 = 1.35;
const LEDGER_Y: [f64; 4] = [-0.65, -1.05, -1.45, -1.85];
/// Each half's two fractions, false-alarm rate then PPV, and their labels.
const FRACTION_Y2: f64 = -2.55;
const FRACTION_DX: [f64; 2] = [1.5, 4.8];
const LABEL_Y: f64 = -3.3;
/// The ring of the prediction pause, where clinic B's block will be.
const ASK_AT: Vec3 = [4.2, 2.1, 0.0];
const ASK_R: f64 = 0.55;
/// The study's pairs: five to a row, four rows.
pub(super) const PAIR_COLS: usize = 5;
pub(super) const PAIR_DX: f64 = 2.1;
pub(super) const PAIR_DY: f64 = 1.05;
pub(super) const PAIR_TOP: f64 = 1.9;
/// A pair's box, and its two patients (A on the left, B on the right).
pub(super) const PAIR_W: f64 = 1.5;
pub(super) const PAIR_H: f64 = 0.75;
pub(super) const DISC_R: f64 = 0.2;
pub(super) const DISC_DX: f64 = 0.33;
/// The study's tag above the pairs, and 02_10's caption beneath them.
pub(super) const STUDY_TAG_Y: f64 = 3.2;
pub(super) const NULL_Y: f64 = -2.5;

/// Where pair `i` of the study sits: five to a row, four rows.
pub(super) fn pair_at(i: usize) -> Vec3 {
    let (row, col) = ((i / PAIR_COLS) as f64, (i % PAIR_COLS) as f64);
    v(
        (col - (PAIR_COLS as f64 - 1.0) / 2.0) * PAIR_DX,
        PAIR_TOP - row * PAIR_DY,
    )
}

/// One pair of the study: its box and its two patients as plain discs,
/// detached, as one group (02_10; chapter 03 opens on them).
pub(super) fn study_pair(stage: &mut Stage<'_>, i: usize) -> fmn::Result<Mob> {
    let at = pair_at(i);
    let back = Rectangle::new()
        .width(PAIR_W)
        .height(PAIR_H)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let mut parts = vec![back];
    for dx in [-DISC_DX, DISC_DX] {
        let d = Circle::new()
            .radius(DISC_R)
            .style(flat(person(), 1.0))
            .build()
            .moved_to(v(at[0] + dx, at[1]));
        parts.push(stage.arena_mut().add(d));
    }
    group(stage, &parts)
}

/// The centre of cell (row, col) of half `p`'s array.
fn cell_in(p: usize, row: usize, col: usize) -> Vec3 {
    v(
        PANEL_X0[p] + AC * (col as f64 + 0.5),
        ARRAY_Y0 - AC * (row as f64 + 0.5),
    )
}

/// The left edge of half `p`'s block of positives.
fn block_x0(p: usize) -> f64 {
    PANEL_X0[p] + AC * GRID_COLS as f64 + BLOCK_GAP
}

/// The centre of place `j` of half `p`'s block, row by row.
fn place_in(p: usize, j: u64) -> Vec3 {
    let (row, col) = ((j / BLOCK_COLS) as f64, (j % BLOCK_COLS) as f64);
    v(block_x0(p) + PC * (col + 0.5), ARRAY_Y0 - PC * (row + 0.5))
}

/// Half `p`'s block: left, top, right, bottom.
fn block_box(p: usize, block: &PositivesBlock) -> (f64, f64, f64, f64) {
    let l = block_x0(p);
    (
        l,
        ARRAY_Y0,
        l + PC * block.cols as f64,
        ARRAY_Y0 - PC * block.rows as f64,
    )
}

/// A clinic's test, read off its counted people: the share caught among
/// people with the disease, and the false-alarm rate among people without it
/// (what the 02_2 card and the 02_9 card print).
fn counted_rates(c: &Counts) -> (Q, Q) {
    (Q::ratio(c.true_pos, c.with), Q::ratio(c.false_pos, c.without))
}

/// A fraction's TeX, thousands written the TeX way (`9{,}900`).
fn tex_fraction(top: u64, bottom: u64) -> String {
    let tex = |n: u64| thousands(n).replace(',', "{,}");
    format!(r"\frac{{{}}}{{{}}}", tex(top), tex(bottom))
}

/// A fraction in TeX with its value in text after it, the pair centred at
/// `(x, y)`, in this chapter's smaller sizes; one group, detached.
pub(super) fn small_fraction(
    kit: &Kit,
    stage: &mut Stage<'_>,
    (top, bottom): (u64, u64),
    color: Srgb,
    value: &str,
    (x, y): (f64, f64),
) -> fmn::Result<Mob> {
    let top_tex = thousands(top).replace(',', "{,}");
    let f = kit.tex(
        stage,
        &tex_fraction(top, bottom),
        36.0,
        Palette::ink(),
        &[(top_tex.as_str(), color)],
        v(0.0, y),
    )?;
    let t = kit.text(stage, value, 26.0, Palette::ink(), v(0.0, y))?;
    let (wf, wt) = (stage.get_width(f), stage.get_width(t));
    let left = x - (wf + 0.12 + wt) / 2.0;
    stage.move_to(f, v(left, y), LEFT);
    stage.move_to(t, v(left + wf + 0.12, y), LEFT);
    group(stage, &[f, t])
}

/// An outline around a mobject already placed, `pad` outside its box.
fn around(stage: &mut Stage<'_>, m: Mob, centre: Vec3, pad: f64) -> fmn::Result<Mob> {
    let (w, h) = (stage.get_width(m), stage.get_height(m));
    outline(
        stage,
        (centre[0] - w / 2.0 - pad, centre[1] + h / 2.0 + pad),
        (centre[0] + w / 2.0 + pad, centre[1] - h / 2.0 - pad),
    )
}

/// The outline of ledger line `i` of half `p`.
fn ledger_box(stage: &mut Stage<'_>, p: usize, i: usize) -> fmn::Result<Mob> {
    outline(
        stage,
        (PANEL_X0[p] + 0.4, LEDGER_Y[i] + 0.2),
        (PANEL_X0[p] + 3.75, LEDGER_Y[i] - 0.2),
    )
}

/// A ledger number of half `p`, its right edge at the number column.
fn ledger_number(kit: &Kit, stage: &mut Stage<'_>, p: usize, i: usize, n: u64, color: Srgb) -> fmn::Result<Mob> {
    let at = v(PANEL_X0[p] + LEDGER_NUM, LEDGER_Y[i]);
    let m = kit.text(stage, &thousands(n), 22.0, color, at)?;
    stage.move_to(m, at, RIGHT);
    Ok(m)
}

/// A ledger line's words, in half `p`.
fn ledger_words(kit: &Kit, stage: &mut Stage<'_>, p: usize, i: usize, color: Srgb) -> fmn::Result<Mob> {
    kit.text_left(
        stage,
        LEDGER_WORDS[i],
        22.0,
        color,
        v(PANEL_X0[p] + LEDGER_TEXT, LEDGER_Y[i]),
    )
}

/// "clinic A · 1 in 100 has the disease", over half `p`.
fn header(kit: &Kit, stage: &mut Stage<'_>, p: usize, name: &str, clinic: &Clinic) -> fmn::Result<Mob> {
    let (some, of) = (clinic.prevalence.num(), clinic.prevalence.den());
    let words = format!("{CLINIC} {name} · {some} {IN} {of} {HAS_IT}");
    let centre = PANEL_X0[p] + 3.35;
    kit.text(stage, &words, 22.0, Palette::ink(), v(centre, HEADER_Y))
}

/// A positive result of half `p`'s array as a tile of its own over its cell.
fn tile_in(stage: &mut Stage<'_>, p: usize, row: usize, col: usize, color: Srgb) -> fmn::Result<Mob> {
    let at = cell_in(p, row, col);
    let t = rect(v(at[0] - AC / 2.0, at[1] - AC / 2.0), AC, AC, flat(color, 1.0))?;
    Ok(stage.arena_mut().add(t))
}

/// Prepared fade-ins of `mobs` as one lagged group of `rt` seconds.
fn lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<AnimationGroup> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(anim)?);
    }
    Ok(AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt))
}

/// Chapter 01's last picture, rebuilt still (its own builders and layout):
/// what this chapter opens on.
struct Opening {
    grid: Mob,
    block: Mob,
    false_alarm: Mob,
    ppv: Mob,
    leaving: Vec<Mob>,
}

fn opening(kit: &Kit, stage: &mut Stage<'_>, c: &Counts, array: &IconArray) -> fmn::Result<Opening> {
    let (gold, blue) = (
        color_gradient(&[person(), disease_dark()], RAMP),
        color_gradient(&[person(), healthy_dark()], RAMP),
    );
    let strips = RowStrips::build(
        stage,
        v(GRID_X0, GRID_Y0),
        (GC, GC),
        &grid_colors(array, &gold, &blue, (RAMP - 1, RAMP - 1, true)),
    )?;
    let block = positives_block_of(c, BLOCK_COLS).map_err(fail)?;
    // chapter 01's tiles at rest: 0.86 of a block cell, centred on its place
    let side = BC * 0.86;
    let mut tiles = Vec::with_capacity(c.positives() as usize);
    for j in 0..c.positives() {
        let color = if j < c.true_pos { disease() } else { healthy() };
        let at = block_at(j, block.cols);
        tiles.push(rect(
            v(at[0] - side / 2.0, at[1] - side / 2.0),
            side,
            side,
            flat(color, 1.0),
        )?);
    }
    let block_mob = stage.arena_mut().add(v_group(tiles));
    let grid_mid = GRID_X0 + GC * GRID_COLS as f64 / 2.0;
    let people = kit.text(
        stage,
        &format!("{} {PEOPLE_MODEL}", thousands(c.people)),
        24.0,
        Palette::dim(),
        v(grid_mid, GRID_Y0 + 0.35),
    )?;
    let block_right = BLOCK_X0 + BC * block.cols as f64;
    let block_bottom = BLOCK_Y0 - BC * block.rows as f64;
    let with_words = kit.text_left(
        stage,
        &format!("{} {WITH_THE_DISEASE}", c.true_pos),
        24.0,
        disease(),
        v(BLOCK_X0, BLOCK_Y0 + 0.3),
    )?;
    let without_words = kit.text_left(
        stage,
        &format!("{} {WITHOUT_THE_DISEASE}", c.false_pos),
        24.0,
        healthy(),
        v(BLOCK_X0, block_bottom - 0.3),
    )?;
    let five = Q::ratio(c.false_pos, c.without).percent(0);
    let ppv = Q::ratio(c.true_pos, c.positives());
    let false_alarm = fraction(
        kit,
        stage,
        &tex_fraction(c.false_pos, c.without),
        healthy(),
        &format!("= {five}%"),
        FRACTION_X[0],
    )?;
    let first_words = under(kit, stage, SHARE_A, healthy(), FRACTION_X[0])?;
    let ppv_mob = fraction(
        kit,
        stage,
        &tex_fraction(c.true_pos, c.positives()),
        disease(),
        &format!("≈ {}%", ppv.percent(0)),
        FRACTION_X[1],
    )?;
    let second_words = under(kit, stage, SHARE_B, disease(), FRACTION_X[1])?;
    let group_b = outline(
        stage,
        (BLOCK_X0 - 0.05, BLOCK_Y0 + 0.05),
        (block_right + 0.05, block_bottom - 0.05),
    )?;
    kit.allow_crossing(group_b);
    let tempting = Q::ratio(c.false_pos, c.without).complement();
    let ask = kit.text(
        stage,
        &format!("{}%?", tempting.percent(0)),
        ASK_SIZE * CORNER_SCALE,
        Palette::ink(),
        CORNER,
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
    let leaving = vec![
        people,
        with_words,
        without_words,
        first_words,
        second_words,
        group_b,
        ask,
        strike,
    ];
    for &m in [strips.group, block_mob, false_alarm, ppv_mob].iter().chain(&leaving) {
        stage.add_to_scene(m)?;
    }
    Ok(Opening {
        grid: strips.group,
        block: block_mob,
        false_alarm,
        ppv: ppv_mob,
        leaving,
    })
}

impl SceneConstruct for TheSameTest {
    fn name(&self) -> &str {
        "02_the_same_test"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (model_a, model_b) = (clinic_a(), clinic_b());
        let (ca, cb) = (model_a.counts().map_err(fail)?, model_b.counts().map_err(fail)?);
        let (array_a, array_b) = (
            IconArray::build(&ca, GRID_COLS).map_err(fail)?,
            IconArray::build(&cb, GRID_COLS).map_err(fail)?,
        );
        let (block_a, block_b) = (
            positives_block_of(&ca, BLOCK_COLS).map_err(fail)?,
            positives_block_of(&cb, BLOCK_COLS).map_err(fail)?,
        );
        let (ppv_a, ppv_b) = (
            Q::ratio(ca.true_pos, ca.positives()),
            Q::ratio(cb.true_pos, cb.positives()),
        );
        // the test's two rates as counted in each clinic's people: the cards
        // read these, and the chapter refuses clinics whose counts disagree
        let ((caught_a, alarm_a), (caught_b, alarm_b)) = (counted_rates(&ca), counted_rates(&cb));
        if caught_a != caught_b || alarm_a != alarm_b {
            return Err(fail(format!(
                "chapter 02 needs the same test in both clinics: catches {caught_a} and {caught_b}, \
                 false alarms {alarm_a} and {alarm_b}"
            )));
        }
        let start = opening(&kit, stage, &ca, &array_a)?;
        let (gold, blue) = (
            color_gradient(&[person(), disease_dark()], RAMP),
            color_gradient(&[person(), healthy_dark()], RAMP),
        );

        // 02_1: chapter 01's picture gathers into clinic A; clinic B appears
        let line = say(&kit, stage, "02_1")?;
        let small_a = RowStrips::build(
            stage,
            v(PANEL_X0[0], ARRAY_Y0),
            (AC, AC),
            &grid_colors(&array_a, &gold, &blue, (RAMP - 1, RAMP - 1, true)),
        )?;
        let (al, at, ar, ab) = block_box(0, &block_a);
        let header_a = header(&kit, stage, 0, "A", &model_a)?;
        let rows_a = kit.text(
            stage,
            &format!("{} {ROWS_OF} {}", block_a.disease_rows, block_a.rows),
            20.0,
            Palette::dim(),
            v((al + ar) / 2.0, ab - 0.25),
        )?;
        let mut ledger_a = Vec::new();
        for (i, (n, color)) in [
            (ca.with, disease()),
            (ca.true_pos, disease()),
            (ca.without, healthy()),
            (ca.false_pos, healthy()),
        ]
        .into_iter()
        .enumerate()
        {
            ledger_a.push(ledger_number(&kit, stage, 0, i, n, color)?);
            ledger_a.push(ledger_words(&kit, stage, 0, i, color)?);
        }
        let labels_a = [
            kit.text(
                stage,
                FALSE_ALARM_RATE,
                20.0,
                Palette::dim(),
                v(PANEL_X0[0] + FRACTION_DX[0], LABEL_Y),
            )?,
            kit.text(
                stage,
                POSITIVES_WITH_IT,
                20.0,
                Palette::dim(),
                v(PANEL_X0[0] + FRACTION_DX[1], LABEL_Y),
            )?,
        ];
        let scale = 36.0 / 44.0;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        let to_block = start
            .block
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.1),
                ..AnimateArgs::default()
            })?
            .scale(PC / BC)?
            .move_to(v((al + ar) / 2.0, (at + ab) / 2.0), ORIGIN)?;
        moves.push(stage.prepare(to_block)?);
        for (m, k) in [(start.false_alarm, 0), (start.ppv, 1)] {
            let slide = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.1),
                    ..AnimateArgs::default()
                })?
                .scale(scale)?
                .move_to(v(PANEL_X0[0] + FRACTION_DX[k], FRACTION_Y2), ORIGIN)?;
            moves.push(stage.prepare(slide)?);
        }
        let mut out = start.leaving.clone();
        out.push(start.grid);
        for m in out {
            let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(anim.rt(0.8))?);
        }
        let mut ins = vec![small_a.group, header_a, rows_a];
        ins.extend(&ledger_a);
        ins.extend(labels_a);
        for m in ins {
            let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(anim.rt(0.9))?);
        }
        stage.play_prepared(moves)?;
        line.wait_for(stage, "a clinic where")?;
        let strips_b = Rc::new(RowStrips::build(
            stage,
            v(PANEL_X0[1], ARRAY_Y0),
            (AC, AC),
            &grid_colors(&array_b, &gold, &blue, (0, 0, false)),
        )?);
        let (sick_t, well_t, left_t) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let painter = stage.arena_mut().add(VMobject::new());
        let (paint, cells) = (Rc::clone(&strips_b), array_b.clone());
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
        fade_in_all(stage, &[strips_b.group, painter], 0.6)?;
        line.wait_for(stage, "one person in ten")?;
        let header_b = header(&kit, stage, 1, "B", &model_b)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(sick_t, 1.0, 0.8)?)?];
        let anim = fade_in(stage.arena_mut(), header_b, ORIGIN, 1.0)?;
        anims.push(stage.prepare(anim.rt(0.6))?);
        stage.play_prepared(anims)?;

        // 02_2: the test has not changed
        let line = say(&kit, stage, "02_2")?;
        let words = format!(
            "{THE_SAME_TEST} {} {IN} {} {WITH_IT} {}% {WITHOUT_IT}",
            caught_b.num(),
            caught_b.den(),
            alarm_b.percent(0)
        );
        let test_card = card(
            &kit,
            stage,
            None,
            &words,
            None,
            (20.0, 140),
            Palette::ink(),
            12.4,
            v(0.0, CARD_Y),
        )?;
        fade_in_all(stage, &[test_card], 0.6)?;
        line.wait_for(stage, "five percent of the rest")?;
        let fa_a = around(
            stage,
            start.false_alarm,
            v(PANEL_X0[0] + FRACTION_DX[0], FRACTION_Y2),
            0.1,
        )?;
        kit.allow_crossing(fa_a);
        fade_in_all(stage, &[fa_a], 0.4)?;

        // 02_3: will a positive result mean more here, or less?
        say(&kit, stage, "02_3")?;
        fade_out_all(stage, &[fa_a], 0.3)?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), ASK_AT)?;
        fade_in_all(stage, &[ask], 0.4)?;
        think_pause(&kit, stage, ASK_AT, ASK_R)?;

        // 02_4: a thousand have the disease; nine hundred are caught
        let line = say(&kit, stage, "02_4")?;
        let n_with = ledger_number(&kit, stage, 1, 0, cb.with, disease())?;
        let w_with = ledger_words(&kit, stage, 1, 0, disease())?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let anim = fade_out(stage.arena_mut(), ask, ORIGIN, 1.0)?;
        anims.push(stage.prepare(anim.rt(0.4))?);
        for m in [n_with, w_with] {
            let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(anim.rt(0.5))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "and nine hundred")?;
        let mut caught = Vec::new();
        for r in 0..array_b.disease_rows {
            for col in 0..array_b.cols {
                if array_b.at(r, col).positive {
                    caught.push(tile_in(stage, 1, r, col, disease())?);
                }
            }
        }
        let caught_n = stage.add_value_tracker(0.0);
        kit.live_number(
            stage,
            caught_n,
            0,
            false,
            22.0,
            disease(),
            v(PANEL_X0[1] + LEDGER_NUM, LEDGER_Y[1]),
            RIGHT,
        )?;
        let w_caught = ledger_words(&kit, stage, 1, 1, disease())?;
        fade_in_all(stage, &[w_caught], 0.3)?;
        let mut both: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(caught_n, cb.true_pos as f64, 1.6)?)?];
        both.push(Box::new(lagged(stage, &caught, 1.6, 0.01)?));
        stage.play_prepared(both)?;

        // 02_5: of the other nine thousand, four hundred fifty false alarms
        let line = say(&kit, stage, "02_5")?;
        let n_without = ledger_number(&kit, stage, 1, 2, cb.without, healthy())?;
        let w_without = ledger_words(&kit, stage, 1, 2, healthy())?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(well_t, 1.0, 0.6)?)?];
        for m in [n_without, w_without] {
            let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(anim.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "five percent")?;
        let mut alarms = Vec::new();
        for r in array_b.disease_rows..array_b.rows {
            for col in 0..array_b.cols {
                if array_b.at(r, col).positive {
                    alarms.push(tile_in(stage, 1, r, col, healthy())?);
                }
            }
        }
        let alarm_n = stage.add_value_tracker(0.0);
        kit.live_number(
            stage,
            alarm_n,
            0,
            false,
            22.0,
            healthy(),
            v(PANEL_X0[1] + LEDGER_NUM, LEDGER_Y[3]),
            RIGHT,
        )?;
        let w_alarms = ledger_words(&kit, stage, 1, 3, healthy())?;
        fade_in_all(stage, &[w_alarms], 0.3)?;
        let mut both: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(alarm_n, cb.false_pos as f64, 1.6)?)?];
        both.push(Box::new(lagged(stage, &alarms, 1.6, 0.01)?));
        stage.play_prepared(both)?;
        let fa_b = small_fraction(
            &kit,
            stage,
            (cb.false_pos, cb.without),
            healthy(),
            &format!("= {}%", Q::ratio(cb.false_pos, cb.without).percent(0)),
            (PANEL_X0[1] + FRACTION_DX[0], FRACTION_Y2),
        )?;
        let fa_label_b = kit.text(
            stage,
            FALSE_ALARM_RATE,
            20.0,
            Palette::dim(),
            v(PANEL_X0[1] + FRACTION_DX[0], LABEL_Y),
        )?;
        fade_in_all(stage, &[fa_b, fa_label_b], 0.5)?;
        let mut tiles = caught.clone();
        tiles.extend(&alarms);
        if tiles.len() as u64 != cb.positives() {
            return Err(fail(format!("{} tiles for {} positives", tiles.len(), cb.positives())));
        }

        // 02_6: gather clinic B's positives; nine hundred of 1,350, two in three
        let line = say(&kit, stage, "02_6")?;
        play!(stage; tween(left_t, 1.0, 0.05)?);
        stage.scene_mut().bring_to_front(&tiles)?;
        let grow = PC * 0.86 / AC;
        let mut flights: Vec<Box<dyn Animation>> = Vec::new();
        for (j, &piece) in tiles.iter().enumerate() {
            let anim = piece
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .scale(grow)?
                .move_to(place_in(1, j as u64), ORIGIN)?;
            flights.push(stage.prepare(anim)?);
        }
        let flight = AnimationGroup::with_lag_ratio(stage.arena_mut(), flights, 0.0008)?.with_run_time(2.0);
        stage.play(flight)?;
        let (bl, _, br, bb) = block_box(1, &block_b);
        let rows_b = kit.text(
            stage,
            &format!("{} {ROWS_OF} {}", block_b.disease_rows, block_b.rows),
            20.0,
            Palette::dim(),
            v((bl + br) / 2.0, bb - 0.25),
        )?;
        fade_in_all(stage, &[rows_b], 0.4)?;
        line.wait_for(stage, "two in three")?;
        let ppv_b_mob = small_fraction(
            &kit,
            stage,
            (cb.true_pos, cb.positives()),
            disease(),
            &format!("≈ {}%", ppv_b.percent(0)),
            (PANEL_X0[1] + FRACTION_DX[1], FRACTION_Y2),
        )?;
        let ppv_label_b = kit.text(
            stage,
            POSITIVES_WITH_IT,
            20.0,
            Palette::dim(),
            v(PANEL_X0[1] + FRACTION_DX[1], LABEL_Y),
        )?;
        fade_in_all(stage, &[ppv_b_mob, ppv_label_b], 0.5)?;

        // 02_7: the two answers, each with its group
        let line = say(&kit, stage, "02_7")?;
        let mut groups = Vec::new();
        for (p, block) in [(0, &block_a), (1, &block_b)] {
            let (l, t, r, bot) = block_box(p, block);
            let g = outline(stage, (l - 0.05, t + 0.05), (r + 0.05, bot - 0.05))?;
            kit.allow_crossing(g);
            groups.push(g);
        }
        fade_in_all(stage, &groups, 0.5)?;
        line.wait_for(stage, "sixty-seven percent")?;
        let ring_b = around(stage, ppv_b_mob, v(PANEL_X0[1] + FRACTION_DX[1], FRACTION_Y2), 0.1)?;
        kit.allow_crossing(ring_b);
        fade_in_all(stage, &[ring_b], 0.4)?;
        line.wait_for(stage, "not fifteen")?;
        let ring_a = around(stage, start.ppv, v(PANEL_X0[0] + FRACTION_DX[1], FRACTION_Y2), 0.1)?;
        kit.allow_crossing(ring_a);
        fade_in_all(stage, &[ring_a], 0.4)?;

        // 02_8: why a positive means more here, asked whole and then answered:
        // the false alarms barely moved; ten times as many caught. During the
        // question the two answers stay, the outlines go
        let line = say(&kit, stage, "02_8")?;
        let mut old = groups.clone();
        old.extend([ring_a, ring_b]);
        fade_out_all(stage, &old, 0.4)?;
        line.wait_for(stage, ALARMS_AT)?;
        let mut alarms_lit = vec![ledger_box(stage, 0, 3)?, ledger_box(stage, 1, 3)?];
        alarms_lit.push(around(
            stage,
            start.false_alarm,
            v(PANEL_X0[0] + FRACTION_DX[0], FRACTION_Y2),
            0.1,
        )?);
        alarms_lit.push(around(stage, fa_b, v(PANEL_X0[1] + FRACTION_DX[0], FRACTION_Y2), 0.1)?);
        for &m in &alarms_lit {
            kit.allow_crossing(m);
        }
        fade_in_all(stage, &alarms_lit, 0.5)?;
        line.wait_for(stage, TEN_TIMES_AT)?;
        let caught_lit = [ledger_box(stage, 0, 1)?, ledger_box(stage, 1, 1)?];
        for &m in &caught_lit {
            kit.allow_crossing(m);
        }
        let ten = Q::ratio(cb.true_pos, ca.true_pos);
        let times = chip(
            &kit,
            stage,
            &format!("{TIMES} {ten}"),
            26.0,
            disease(),
            v(-1.2, LEDGER_Y[1]),
        )?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &alarms_lit {
            let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(anim.rt(0.4))?);
        }
        for m in caught_lit.into_iter().chain([times]) {
            let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(anim.rt(0.5))?);
        }
        stage.play_prepared(anims)?;

        // 02_9: on a cleared stage, the two rates apart, and neither a p-value
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        let line = say(&kit, stage, "02_9")?;
        let left = card(
            &kit,
            stage,
            Some(FALSE_ALARM_RATE),
            &format!("{P_FALSE_ALARM} {alarm_a}"),
            Some(SAME_IN_BOTH),
            (28.0, 22),
            healthy(),
            5.8,
            v(-3.45, 1.0),
        )?;
        let right = card(
            &kit,
            stage,
            Some(PPV),
            &format!("{P_PPV} {ppv_a} {IN_CLINIC} A, {ppv_b} {IN_CLINIC} B"),
            Some(MOVES),
            (28.0, 22),
            disease(),
            5.8,
            v(3.45, 1.0),
        )?;
        let not_equal = kit.tex(stage, r"\neq", 60.0, Palette::ink(), &[], v(0.0, 1.0))?;
        fade_in_lagged(stage, &[left, not_equal, right], 1.2, 0.4)?;
        line.wait_for(stage, NEITHER_AT)?;
        let neither = kit.text(stage, NEITHER, 40.0, Palette::hero(), v(0.0, -1.6))?;
        fade_in_all(stage, &[neither], 0.5)?;

        // 02_10: a study, and the imaginary world it needs; no result yet
        let line = say(&kit, stage, "02_10")?;
        fade_out_all(stage, &[left, not_equal, right, neither], 0.5)?;
        let pairs = study().pairs as usize;
        let tag = kit.text(
            stage,
            &format!("{STUDY_A} {pairs} {STUDY_B}"),
            22.0,
            Palette::dim(),
            v(0.0, STUDY_TAG_Y),
        )?;
        let mut boxes = Vec::new();
        for i in 0..pairs {
            boxes.push(study_pair(stage, i)?);
        }
        fade_in_all(stage, &[tag], 0.4)?;
        fade_in_lagged(stage, &boxes, 1.6, 0.15)?;
        line.wait_for(stage, "imaginary world")?;
        let null = kit.text(stage, NULL_WORLD, 28.0, Palette::ink(), v(0.0, NULL_Y))?;
        fade_in_all(stage, &[null], 0.6)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 1.0)?;
        // chapter 03 opens on the pairs and their question, as they rest
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 02_8 asks a whole question before its answer (not a bare "Why?") and
    /// its pictures land in the answer; 02_9 opens a new paragraph without a
    /// connective. Each phrase is said once, in the sentence the narrator
    /// times it in.
    #[test]
    fn the_why_is_a_whole_question_then_its_answer() {
        use crate::chapters::draw::sentence_offsets;
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let why = text("02_8");
        let starts = sentence_offsets(why);
        assert_eq!(starts.len(), 2);
        let question = &why[..starts[1] - 1];
        assert!(
            question.ends_with('?') && question.split_whitespace().count() >= 6,
            "{question:?}"
        );
        assert!(question.contains("positive result") && question.contains("here"));
        assert!(!text("02_9").starts_with("So "));
        for (id, phrase, sentence, opens) in [
            ("02_8", ALARMS_AT, 1, true),
            ("02_8", TEN_TIMES_AT, 1, false),
            ("02_9", NEITHER_AT, 2, false),
        ] {
            let t = text(id);
            assert_eq!(t.matches(phrase).count(), 1, "{id} says {phrase:?} once");
            let at = t[..t.find(phrase).unwrap()].chars().count();
            let starts = sentence_offsets(t);
            assert_eq!(
                starts.iter().rposition(|&s| s <= at),
                Some(sentence),
                "{id}: {phrase:?}"
            );
            assert_eq!(starts.contains(&at), opens, "{id}: {phrase:?}");
        }
    }

    /// The layout holds the counts: both arrays are the clinics', both
    /// blocks are 45 wide and fit their halves, nothing crosses the middle.
    #[test]
    fn the_two_halves_hold_the_two_clinics() {
        let (ca, cb) = (clinic_a().counts().unwrap(), clinic_b().counts().unwrap());
        let (ba, bb) = (
            positives_block_of(&ca, BLOCK_COLS).unwrap(),
            positives_block_of(&cb, BLOCK_COLS).unwrap(),
        );
        assert_eq!((ba.rows, ba.disease_rows, bb.rows, bb.disease_rows), (13, 2, 30, 20));
        for (p, block) in [(0, &ba), (1, &bb)] {
            let (l, t, r, b) = block_box(p, block);
            assert!(l > PANEL_X0[p] + AC * GRID_COLS as f64, "block {p} clears its array");
            let right_edge = if p == 0 { PANEL_X0[1] - 0.2 } else { 6.9 };
            assert!(r <= right_edge, "block {p} ends at {r}");
            assert!(t <= HEADER_Y - 0.2 && b > LEDGER_Y[0] + 0.5, "block {p} spans {t}..{b}");
        }
        // the last place of clinic B's block is its bottom-right cell
        let last = place_in(1, cb.positives() - 1);
        let (_, _, r, b) = block_box(1, &bb);
        assert!((last[0] - (r - PC / 2.0)).abs() < 1e-9 && (last[1] - (b + PC / 2.0)).abs() < 1e-9);
        // the arrays end above the test's card, the fractions above the labels
        const {
            assert!(ARRAY_Y0 - AC * GRID_COLS as f64 > CARD_Y + 0.3);
            assert!(FRACTION_Y2 - 0.45 > LABEL_Y + 0.15);
        }
        // the think ring and its note sit inside clinic B's block area
        let (l, t, _, b) = block_box(1, &bb);
        assert!(ASK_AT[0] - ASK_R > l && ASK_AT[1] + ASK_R < t && ASK_AT[1] - ASK_R > b);
    }

    /// The cards print the rates counted in the clinics' people, and those
    /// are the facts both clinics register: the same in A and B, so the 02_9
    /// card's "the same in both clinics" is a computed statement.
    #[test]
    fn the_cards_read_the_counted_rates_which_are_the_facts() {
        let f = crate::facts::facts().unwrap();
        let (ca, cb) = (clinic_a().counts().unwrap(), clinic_b().counts().unwrap());
        let (rates_a, rates_b) = (counted_rates(&ca), counted_rates(&cb));
        assert_eq!(rates_a, rates_b);
        for prefix in ["M-F1", "M-F2"] {
            let caught = f.get(&format!("{prefix}.sensitivity.exact")).unwrap();
            let alarm = f.get(&format!("{prefix}.false_alarm.exact")).unwrap();
            assert_eq!(
                (rates_a.0.to_string(), rates_a.1.to_string()),
                (caught.to_owned(), alarm.to_owned())
            );
        }
        // a clinic whose test caught fewer people would be refused by the comparison
        let other = Counts {
            true_pos: cb.true_pos - 100,
            false_neg: cb.false_neg + 100,
            ..cb
        };
        assert_ne!(counted_rates(&other), rates_a);
    }

    #[test]
    fn the_fractions_are_written_the_tex_way() {
        assert_eq!(tex_fraction(495, 9_900), r"\frac{495}{9{,}900}");
        assert_eq!(tex_fraction(900, 1_350), r"\frac{900}{1{,}350}");
        assert_eq!(tex_fraction(90, 585), r"\frac{90}{585}");
    }
}
