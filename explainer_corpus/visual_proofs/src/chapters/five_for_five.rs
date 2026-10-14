//! Chapter 02, five for five: the hook (storyboard 02). The cubes are added
//! one at a time and every total is a square, five times (02_1, 02_2); that
//! is not a reason (02_3); points on a circle joined by lines cut it into 1,
//! 2, 4, 8, 16 regions (02_4); a guess for six (02_5, a quiz); and 31, not
//! 32, counted (02_6); five for five proves nothing about the sixth (02_7).
//!
//! The cubes and their totals come from `F-K.cubes`, `F-C.cube_totals` and
//! `F-C.cube_roots`, and are checked against each other and the model before
//! anything is shown. The circle's pieces are `model::circle::faces` of
//! `model::circle::drawn(k)`, one convex polygon a region, and their number
//! is checked against `F-C.regions` (computed from the drawing's crossings,
//! not from the closed form) each time a point lands.

use explainer_kit::{Kit, Timed, fade_all, play, v};
use fmn::library::{checkmark, exmark, line_brace};
use fmn::prelude::*;

use super::draw::{
    ANSWER_PT, ASIDE_AT, QuizCard, QuizLook, Words, aside, copy, dim, fact_int, fact_ints, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flat, fly, group, hero, ink, line_style, live_counter, lower_third, pause, question_card,
    quiz_named, say, size, swap, tween_linear, warn,
};
use super::iso::{Iso, Solid, ViewAt, box_cells};
use crate::facts::shown;
use crate::model::circle::{self, Corner};

pub struct FiveForFive {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["02_1", "02_2", "02_3", "02_4", "02_5", "02_6", "02_7"];

// the chapter's fixed words (numbers are built from facts)
const TOTAL_WORD: &str = "total";
const POINTS_WORD: &str = "points";
const REGIONS_WORD: &str = "regions";
const CUBE_TOTALS_WORD: &str = "cube totals";
const FIVE_FOR_FIVE: &str = "five for five";
const PRECISELY: &str = "precisely:";
const FOR_PROGRAMMERS: &str = "for programmers";
const TESTS_ASIDE: &str = "five passing tests are not a proof either";
/// The quiz this chapter asks (`model::QUIZZES`).
const QUIZ: &str = "regions";

// ------------------------------------------------------------------ geometry

/// The ledger of 02_1 and 02_2: the first row's height and the rows' pitch;
/// the right edges of the "+ cube" column and of the totals; the left edge of
/// the "= r × r" column.
const LEDGER_TOP: f64 = 0.95;
const LEDGER_PITCH: f64 = 0.78;
const ADDS_RIGHT: f64 = -1.1;
const TOTALS_RIGHT: f64 = 1.0;
const SQUARES_LEFT: f64 = 1.45;
const LEDGER_PT: f64 = 44.0;
/// The running count of 02_1, and its word.
const RUNNING_AT: Vec3 = [5.6, 0.35, 0.0];
const RUNNING_PT: f64 = 110.0;
/// The cubes' row along the top.
const ROW_CENTRE: (f64, f64) = (0.0, 3.0);
const ROW_ROOM: (f64, f64) = (10.6, 1.9);
const ROW_GAP: f64 = 0.8;

/// How long the statement of 02_6 stays after the line and its breath (it
/// is two lines of about twenty-five words) before 02_7 begins; it is on
/// the screen through the breath and the first second of 02_7 too, so about
/// five seconds of reading in all.
const THIRD_READ_S: f64 = 0.6;
/// The circle of 02_4 to 02_6: centre and radius.
const CIRCLE_AT: Vec3 = [-3.4, 1.15, 0.0];
const CIRCLE_R: f64 = 2.0;
/// The small table under it: the first column's centre, the columns' pitch,
/// the rows' heights, the labels' left edge.
const SMALL_X: f64 = -4.9;
const SMALL_PITCH: f64 = 0.8;
const SMALL_ROWS: (f64, f64) = (-1.75, -2.35);
const SMALL_LABEL_LEFT: f64 = -6.75;
const SMALL_PT: f64 = 32.0;
/// A region is drawn this share of its size about its centre, so the dark
/// seams between regions are there to be counted.
const REGION_SHARE: f64 = 0.9;
const REGION_OPACITY: f64 = 0.35;
/// The big table of 02_7: the columns' centres, the rows' heights.
const BIG_X: f64 = -2.0;
const BIG_PITCH: f64 = 1.5;
const BIG_ROWS: (f64, f64, f64) = (1.9, 0.9, -0.9);
const BIG_LABEL_LEFT: f64 = -6.3;
const BIG_PT: f64 = 42.0;

/// Where the circle's point `p` (on the unit circle) is on the screen.
fn on_circle(p: Corner) -> Vec3 {
    v(CIRCLE_AT[0] + CIRCLE_R * p.0, CIRCLE_AT[1] + CIRCLE_R * p.1)
}

/// The centre of the ledger's row `k` (from 0).
fn ledger_y(k: usize) -> f64 {
    LEDGER_TOP - LEDGER_PITCH * k as f64
}

/// The small table's column `k` (from 0).
fn small_x(k: usize) -> f64 {
    SMALL_X + SMALL_PITCH * k as f64
}

/// The big table's column `k` (from 0).
fn big_x(k: usize) -> f64 {
    BIG_X + BIG_PITCH * k as f64
}

/// Everything the chapter shows of the cubes and the circle, as the facts
/// give it and checked against the model.
struct Hook {
    cubes: Vec<i64>,
    totals: Vec<i64>,
    roots: Vec<i64>,
    regions: Vec<i64>,
    doubling: Vec<i64>,
    answer: i64,
    /// Regions of six evenly spaced points (`F-C.evenly_spaced.regions`),
    /// the number the 02_6 lower-third gives beside 31.
    evenly_spaced: i64,
}

impl Hook {
    fn read() -> fmn::Result<Self> {
        let cubes = fact_ints("F-K.cubes")?;
        let totals = fact_ints("F-C.cube_totals")?;
        let roots = fact_ints("F-C.cube_roots")?;
        let regions = fact_ints("F-C.regions")?;
        let doubling = fact_ints("F-C.doubling")?;
        let n = cubes.len();
        let running: Vec<i64> = cubes
            .iter()
            .scan(0, |sum, c| {
                *sum += c;
                Some(*sum)
            })
            .collect();
        if n != shown::CUBES as usize || totals != running {
            return Err(fail("F-C.cube_totals are not the running sum of F-K.cubes".into()));
        }
        if roots.iter().zip(&totals).any(|(r, t)| r * r != *t) || roots.len() != n {
            return Err(fail("a cube total is not the square of its root".into()));
        }
        // the circle: five counted from the drawing, and the sixth is the quiz
        let drawn: Vec<i64> = (1..=shown::POINTS)
            .map(|k| circle::regions(&circle::drawn(k)) as i64)
            .collect();
        if regions != drawn {
            return Err(fail("F-C.regions are not the regions of the drawn points".into()));
        }
        let answer = fact_int("F-Q.regions.answer")?;
        if drawn.last() != Some(&answer) || doubling.last() != Some(&(2 * regions[regions.len() - 2])) {
            return Err(fail("the quiz's answer is not the sixth region count".into()));
        }
        for k in 1..=shown::POINTS {
            let faces = circle::faces(&circle::drawn(k)).len() as i64;
            if faces != drawn[k - 1] {
                return Err(fail(format!("{k} points draw {faces} faces, not {}", drawn[k - 1])));
            }
        }
        Ok(Self {
            cubes,
            totals,
            roots,
            regions: drawn,
            doubling,
            answer,
            evenly_spaced: fact_int("F-C.evenly_spaced.regions")?,
        })
    }
}

/// Text with its right edge at `at`. Detached.
fn text_right(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, color, at)?;
    stage.move_to(m, at, RIGHT);
    Ok(m)
}

/// A tick or a cross at `at`, `side` high. Detached.
fn mark_at(stage: &mut Stage<'_>, ok: bool, at: Vec3, side: f64) -> Mob {
    let shape = if ok { checkmark(ink()) } else { exmark(warn()) };
    stage.arena_mut().add(shape.with_height(side, false).moved_to(at))
}

/// A region as a polygon on the circle, drawn `share` of its size about its
/// centre.
fn region_polygon(stage: &mut Stage<'_>, face: &[Corner], share: f64, style: Style) -> Mob {
    let n = face.len() as f64;
    let (cx, cy) = face.iter().fold((0.0, 0.0), |a, c| (a.0 + c.0 / n, a.1 + c.1 / n));
    let corners: Vec<Vec3> = face
        .iter()
        .map(|&(x, y)| on_circle((cx + (x - cx) * share, cy + (y - cy) * share)))
        .collect();
    stage.arena_mut().add(Polygon::new(corners).style(style).build())
}

/// The regions of `points` as tinted polygons, one group. Detached.
fn tinted_regions(stage: &mut Stage<'_>, points: &[circle::Point]) -> fmn::Result<(Mob, Vec<Vec<Corner>>)> {
    let faces = circle::faces(points);
    let polys: Vec<Mob> = faces
        .iter()
        .map(|f| region_polygon(stage, f, REGION_SHARE, flat(copy(0), REGION_OPACITY)))
        .collect();
    Ok((group(stage, &polys)?, faces))
}

/// One row of a table: its label, left edge at `left`, and one entry in
/// each of `entries`' columns. Detached; the entries are returned apart so
/// they can arrive one at a time.
#[allow(clippy::too_many_arguments)]
fn table_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    label: &str,
    left: f64,
    y: f64,
    entries: &[(String, Srgb)],
    column: impl Fn(usize) -> f64,
    pt: f64,
) -> fmn::Result<(Mob, Vec<Mob>)> {
    let head = kit.text_left(stage, label, pt * 0.8, dim(), v(left, y))?;
    let mut cells = Vec::with_capacity(entries.len());
    for (k, (s, color)) in entries.iter().enumerate() {
        cells.push(kit.text(stage, s, pt, *color, v(column(k), y))?);
    }
    Ok((head, cells))
}

/// The cubes' row of the first beats: five small cubes along the top.
struct CubeRow {
    iso: Iso,
    solids: Vec<Solid>,
}

impl CubeRow {
    fn build(stage: &mut Stage<'_>, n: i64) -> fmn::Result<Self> {
        let xs: Vec<f64> = (1..=n).map(|k| (1..k).map(|i| i as f64 + ROW_GAP).sum()).collect();
        let width = xs[xs.len() - 1] + n as f64;
        let view =
            ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0, 0.0, 0.0], [width, n as f64, n as f64], ROW_CENTRE, ROW_ROOM);
        let iso = Iso::new(stage, view)?;
        let mut solids = Vec::new();
        for k in 1..=n {
            let s = iso.solid(
                stage,
                &box_cells(k, k, k),
                size(k as usize),
                [xs[k as usize - 1], 0.0, 0.0],
            )?;
            solids.push(s);
        }
        Ok(Self { iso, solids })
    }

    /// Where cube `k` (from 0) is on the screen: its middle.
    fn middle(&self, stage: &Stage<'_>, k: usize) -> Vec3 {
        let (lo, hi) = self.iso.bounds(stage, self.solids[k]);
        self.iso.project(
            stage,
            [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0],
        )
    }
}

impl SceneConstruct for FiveForFive {
    fn name(&self) -> &str {
        "02_five_for_five"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let hook = Hook::read()?;
        let n = hook.cubes.len();
        let points = circle::drawn(shown::POINTS);

        // ---------------------------------------------------------- 02_1
        // the cubes one at a time, and the total they keep
        let line = say(&kit, stage, LINES[0])?;
        let row = CubeRow::build(stage, n as i64)?;
        let mut ins = Vec::new();
        for &s in &row.solids {
            ins.push(row.iso.fade(s, 1.0, 0.5)?);
        }
        row.iso.play(stage, ins, 0.12)?;
        let running = stage.add_value_tracker(0.0);
        let counter = live_counter(&kit, stage, running, RUNNING_PT, ink(), RUNNING_AT)?;
        let total_word = kit.text(stage, TOTAL_WORD, 36.0, dim(), v(RUNNING_AT[0], RUNNING_AT[1] + 1.1))?;
        fade_in_all(stage, &[total_word], 0.4)?;
        let mut adds: Vec<Mob> = Vec::new();
        let mut totals: Vec<Mob> = Vec::new();
        for k in 0..n {
            let y = ledger_y(k);
            let plus = text_right(
                &kit,
                stage,
                &format!("+ {}", hook.cubes[k]),
                LEDGER_PT,
                size(k + 1),
                v(ADDS_RIGHT, y),
            )?;
            let total = text_right(
                &kit,
                stage,
                &hook.totals[k].to_string(),
                LEDGER_PT + 6.0,
                ink(),
                v(TOTALS_RIGHT, y),
            )?;
            // the cube lights and its number goes down to its row of the
            // ledger; the total is kept and the running count catches up
            let from = row.middle(stage, k);
            let half = stage.get_width(plus) / 2.0;
            stage.move_to(plus, from, ORIGIN);
            stage.add_to_scene(plus)?;
            play!(
                stage;
                row.iso.glow(row.solids[k], hero(), 1.0, 0.2)?,
                fly(plus, v(ADDS_RIGHT - half, y), 1.0, 0.55)?,
                tween_to(running, hook.totals[k] as f64, 0.55)?,
                fade_in(stage.arena_mut(), total, ORIGIN, 1.0)?.rt(0.4),
            );
            // the storyboard's woodblock tick on each total: a counter's tick
            // lands on its mark, under the voice, and the score keeps it soft
            kit.mark(stage, "tick:total");
            play!(stage; row.iso.glow(row.solids[k], hero(), 0.0, 0.3)?);
            adds.push(plus);
            totals.push(total);
        }
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 02_2
        // every total is a square
        let line = say(&kit, stage, LINES[1])?;
        let mut squares: Vec<Mob> = Vec::new();
        let mut ticks: Vec<Mob> = Vec::new();
        for (k, &r) in hook.roots.iter().enumerate() {
            squares.push(kit.text_left(
                stage,
                &format!("= {r} × {r}"),
                LEDGER_PT,
                size(k + 1),
                v(SQUARES_LEFT, ledger_y(k)),
            )?);
            let right = SQUARES_LEFT + 2.3;
            ticks.push(mark_at(stage, true, v(right, ledger_y(k)), 0.34));
        }
        let cues = ["One,", "three,", "six,", "ten", "fifteen"];
        for (k, cue) in cues.iter().enumerate().take(n) {
            line.wait_for(stage, cue)?;
            fade_in_all(stage, &[squares[k]], 0.35)?;
        }
        line.wait_for(stage, "Five checks")?;
        fade_in_lagged(stage, &ticks, 1.2, 0.6)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 02_3
        // so is it always true? the sixth slot; and a pattern can break
        let line = say(&kit, stage, LINES[2])?;
        let sixth_y = ledger_y(n);
        let sixth_plus = text_right(&kit, stage, "+ ?", LEDGER_PT, hero(), v(ADDS_RIGHT, sixth_y))?;
        let sixth_total = text_right(&kit, stage, "?", LEDGER_PT + 6.0, hero(), v(TOTALS_RIGHT, sixth_y))?;
        fade_in_all(stage, &[sixth_plus, sixth_total], 0.5)?;
        line.wait_last(stage)?;
        let breaks = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("02_3 has no sentences".into()))?;
        let card = question_card(&kit, stage, &breaks, v(0.0, 0.2))?;
        // the ledger goes up and out; the cubes' row and the counter with it
        let mut ledger = vec![total_word, counter, sixth_plus, sixth_total];
        ledger.extend(&adds);
        ledger.extend(&totals);
        ledger.extend(&squares);
        ledger.extend(&ticks);
        let to_dim = row.iso.fade_all(0.0, 0.6)?;
        let mut away: Vec<Box<dyn Animation>> = vec![stage.prepare(to_dim)?];
        for &m in &ledger {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, v(0.0, 0.6), 1.0)?;
            away.push(stage.prepare(a.rt(0.6))?);
        }
        let a = fade_in(stage.arena_mut(), card, ORIGIN, 1.0)?;
        away.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(away)?;
        row.iso.freeze(stage);
        kit.hold(stage)?;
        fade_out_all(stage, &[card], 0.4)?;

        // ---------------------------------------------------------- 02_4
        // points on a circle, joined, no three lines through one place, the
        // pieces; then the same picture built again a point at a time, and
        // the regions counted
        let line = say(&kit, stage, LINES[3])?;
        let rim = stage.arena_mut().add(
            Circle::new()
                .radius(CIRCLE_R)
                .arc_center(CIRCLE_AT)
                .style(line_style(dim(), 3.0))
                .build(),
        );
        line.wait_for(stage, "Put points")?;
        play!(stage; show_creation(rim).rt(0.8));
        let five = shown::POINTS - 1;
        let place = |stage: &mut Stage<'_>, k: usize, color: Srgb| -> Mob {
            let p = on_circle(circle::xy(&points[k]));
            stage
                .arena_mut()
                .add(Dot::new().point(p).radius(0.09).style(flat(color, 1.0)).build())
        };
        let demo_dots: Vec<Mob> = (0..five).map(|k| place(stage, k, ink())).collect();
        fade_in_lagged(stage, &demo_dots, 1.0, 0.5)?;
        line.wait_for(stage, "join every pair")?;
        let mut demo_chords: Vec<Mob> = Vec::new();
        for (i, j) in circle::lines(five) {
            let (a, b) = (on_circle(circle::xy(&points[i])), on_circle(circle::xy(&points[j])));
            demo_chords.push(
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(line_style(dim(), 2.5)).build()),
            );
        }
        let mut drawing: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &demo_chords {
            drawing.push(stage.prepare(show_creation(m).rt(0.6))?);
        }
        let drawn = AnimationGroup::with_lag_ratio(stage.arena_mut(), drawing, 0.2)?.with_run_time(1.4);
        stage.play(drawn)?;
        stage.scene_mut().bring_to_front(&demo_dots)?;
        // no three lines cross at one place: the five crossings, each of two
        line.wait_for(stage, "Place the points")?;
        let crossings = circle::crossings(&points[..five]);
        if crossings.len() as u64 != crate::model::choose(five as u64, 4) {
            return Err(fail(
                "the five points' lines do not cross once for every four points".into(),
            ));
        }
        let crossing_dots: Vec<Mob> = crossings
            .iter()
            .map(|c| {
                stage.arena_mut().add(
                    Dot::new()
                        .point(on_circle(circle::xy(c)))
                        .radius(0.08)
                        .style(flat(hero(), 1.0))
                        .build(),
                )
            })
            .collect();
        fade_in_lagged(stage, &crossing_dots, 1.0, 0.4)?;
        // and the pieces they cut: each is a region
        line.wait_for(stage, "The lines cut")?;
        let (demo_faces, _) = tinted_regions(stage, &points[..five])?;
        play!(stage; fade_in(stage.arena_mut(), demo_faces, ORIGIN, 1.0)?.rt(0.6));
        stage.scene_mut().bring_to_front(&demo_dots)?;
        stage.scene_mut().bring_to_front(&crossing_dots)?;
        // the counting begins from one point: everything else is cleared
        line.wait_for(stage, "One point")?;
        let mut cleared = vec![demo_faces];
        cleared.extend(&demo_chords);
        cleared.extend(&crossing_dots);
        cleared.extend(&demo_dots);
        let head_y = SMALL_ROWS.0;
        let body_y = SMALL_ROWS.1;
        let first_five: Vec<(String, Srgb)> = (1..=five).map(|k| (k.to_string(), ink())).collect();
        let (points_label, point_cells) = table_row(
            &kit,
            stage,
            POINTS_WORD,
            SMALL_LABEL_LEFT,
            head_y,
            &first_five,
            small_x,
            SMALL_PT,
        )?;
        let region_entries: Vec<(String, Srgb)> = hook.regions[..five].iter().map(|r| (r.to_string(), ink())).collect();
        let (regions_label, region_cells) = table_row(
            &kit,
            stage,
            REGIONS_WORD,
            SMALL_LABEL_LEFT,
            body_y,
            &region_entries,
            small_x,
            SMALL_PT,
        )?;
        swap(stage, &cleared, &[points_label, regions_label], 0.4)?;
        let counted = ["One point", "Two points", "Three points", "Four points", "Five points"];
        let numbers = ["one region.", "two.", "four.", "eight.", "sixteen."];
        let mut dots: Vec<Mob> = Vec::new();
        let mut chords: Vec<Mob> = Vec::new();
        let mut tint: Option<Mob> = None;
        for k in 1..=five {
            if k > 1 {
                line.wait_for(stage, counted[k - 1])?;
            }
            let dot = place(stage, k - 1, hero());
            dots.push(dot);
            let p = on_circle(circle::xy(&points[k - 1]));
            let mut new_chords: Vec<Mob> = Vec::new();
            for earlier in &points[..k - 1] {
                let a = on_circle(circle::xy(earlier));
                new_chords.push(
                    stage
                        .arena_mut()
                        .add(polyline(vec![a, p]).style(line_style(dim(), 2.5)).build()),
                );
            }
            let (faces, _) = tinted_regions(stage, &points[..k])?;
            let mut ins = vec![dot, faces];
            ins.extend(&new_chords);
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &ins {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.5))?);
            }
            if let Some(old) = tint.take() {
                stage.clear_updaters(old, true);
                let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.5))?);
            }
            let a = fade_in(stage.arena_mut(), point_cells[k - 1], ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
            stage.play_prepared(anims)?;
            stage.scene_mut().bring_to_front(&dots)?;
            // the count is said after the points: its number comes with it
            line.wait_for(stage, numbers[k - 1])?;
            fade_in_all(stage, &[region_cells[k - 1]], 0.4)?;
            // the older point sits quietly in ink once the next one lands
            if k >= 2 {
                stage.set_fill(dots[k - 2], Some(ink()), Some(1.0), None, true);
            }
            chords.extend(new_chords);
            tint = Some(faces);
        }
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 02_5
        // your turn: how many regions will six points make?
        let quiz = quiz_named(QUIZ)?;
        let line = say(&kit, stage, quiz.line)?;
        let sixth_point = on_circle(circle::xy(&points[shown::POINTS - 1]));
        let sixth_dot = stage.arena_mut().add(
            Dot::new()
                .point(sixth_point)
                .radius(0.09)
                .style(flat(hero(), 1.0))
                .build(),
        );
        let unknown = kit.text(stage, "?", SMALL_PT, hero(), v(small_x(shown::POINTS - 1), body_y))?;
        let six = kit.text(
            stage,
            &shown::POINTS.to_string(),
            SMALL_PT,
            ink(),
            v(small_x(shown::POINTS - 1), head_y),
        )?;
        play!(
            stage;
            fade_in(stage.arena_mut(), sixth_dot, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), six, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), unknown, ORIGIN, 1.0)?.rt(0.5),
        );
        stage.scene_mut().bring_to_front(&[sixth_dot])?;
        let question = format!("{} {POINTS_WORD}: ? {REGIONS_WORD}", shown::POINTS);
        let answer = hook.answer.to_string();
        let card = QuizCard::build(
            &kit,
            stage,
            QUIZ,
            Words::Text(&question),
            Words::Text(&answer),
            QuizLook::BESIDE,
        )?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ---------------------------------------------------------- 02_6
        // thirty-one: the lines draw, the regions light, the counter runs
        kit.mark(stage, "silence:thirty_one");
        let line = say(&kit, stage, LINES[5])?;
        let mut last_lines: Vec<Mob> = Vec::new();
        for earlier in &points[..shown::POINTS - 1] {
            let a = on_circle(circle::xy(earlier));
            last_lines.push(
                stage
                    .arena_mut()
                    .add(polyline(vec![a, sixth_point]).style(line_style(dim(), 2.5)).build()),
            );
        }
        fade_out_all(stage, &[card.back, card.label, card.question], 0.3)?;
        let mut drawing: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &last_lines {
            drawing.push(stage.prepare(show_creation(m).rt(0.5))?);
        }
        let draw = AnimationGroup::with_lag_ratio(stage.arena_mut(), drawing, 0.3)?.with_run_time(1.0);
        stage.play(draw)?;
        stage.scene_mut().bring_to_front(&dots)?;
        stage.scene_mut().bring_to_front(&[sixth_dot])?;
        // the regions of the whole drawing replace the sixteen
        let (full, faces) = tinted_regions(stage, &points)?;
        if faces.len() as i64 != hook.answer {
            return Err(fail("the six points' faces are not the quiz's answer".into()));
        }
        let lit: Vec<Mob> = faces
            .iter()
            .map(|f| region_polygon(stage, f, REGION_SHARE, flat(hero(), 0.8)))
            .collect();
        let swap_out: Vec<Mob> = tint.take().into_iter().collect();
        swap(stage, &swap_out, &[full], 0.3)?;
        stage.scene_mut().bring_to_front(&dots)?;
        stage.scene_mut().bring_to_front(&[sixth_dot])?;
        // "Count them": the answer's gold gives way to a counter in its place
        line.wait_for(stage, "Count them")?;
        let tally = stage.add_value_tracker(0.0);
        let scale = card.look.scale;
        let counter = live_counter(&kit, stage, tally, ANSWER_PT * scale, hero(), card.answer_at())?;
        let regions_word = kit.text(
            stage,
            REGIONS_WORD,
            36.0,
            dim(),
            v(card.answer_at()[0], card.answer_at()[1] + 1.0),
        )?;
        fade_out_all(stage, &[card.answer], 0.2)?;
        fade_in_all(stage, &[regions_word], 0.3)?;
        let quick = faces.len() - 1;
        let mut lighting: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &lit[..quick] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            lighting.push(stage.prepare(a)?);
        }
        let first_pass = AnimationGroup::with_lag_ratio(stage.arena_mut(), lighting, 0.6)?.with_run_time(1.5);
        let counting = tween_to(tally, quick as f64, 1.5)?;
        play!(stage; first_pass, counting);
        // a pause, and then the last one
        pause(stage, 0.4)?;
        play!(
            stage;
            fade_in(stage.arena_mut(), lit[quick], ORIGIN, 1.0)?.rt(0.3),
            tween_to(tally, faces.len() as f64, 0.3)?,
        );
        // the doubling's thirty-two, said, and struck out; thirty-one beside it
        let at = v(small_x(shown::POINTS - 1), body_y);
        let doubled = kit.text(
            stage,
            &hook.doubling[shown::POINTS - 1].to_string(),
            SMALL_PT,
            warn(),
            at,
        )?;
        line.wait_for(stage, "The doubling pattern")?;
        play!(
            stage;
            fade_out(stage.arena_mut(), unknown, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), doubled, ORIGIN, 1.0)?.rt(0.4),
        );
        let strike = stage.arena_mut().add(
            polyline(vec![v(at[0] - 0.3, at[1] - 0.16), v(at[0] + 0.3, at[1] + 0.16)])
                .style(line_style(warn(), 4.0))
                .build(),
        );
        let found = kit.text(stage, &hook.answer.to_string(), SMALL_PT, hero(), v(at[0] + 0.8, at[1]))?;
        line.wait_for(stage, "and the pattern was wrong")?;
        play!(
            stage;
            fade_in(stage.arena_mut(), strike, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), found, ORIGIN, 1.0)?.rt(0.4),
        );
        // the exact statement, with the time to read it
        let moser = fact_ints("F-X.moser")?;
        let year = moser
            .first()
            .copied()
            .ok_or_else(|| fail("F-X.moser has no year".into()))?;
        let statement = format!(
            "six points make {} regions when no three lines cross at one point inside the circle (six evenly spaced points give {}). Leo Moser, “On the Danger of Induction”, {}",
            hook.answer, hook.evenly_spaced, year
        );
        let third = lower_third(&kit, stage, PRECISELY, &statement)?;
        fade_in_all(stage, &[third], 0.5)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        pause(stage, THIRD_READ_S)?;

        // ---------------------------------------------------------- 02_7
        // five checks in a row prove nothing about the sixth
        let line = say(&kit, stage, LINES[6])?;
        let mut leaving: Vec<Mob> = vec![rim, third, counter, regions_word, full, points_label, regions_label];
        leaving.extend(&point_cells);
        leaving.extend(&region_cells);
        leaving.extend(&dots);
        leaving.extend(&chords);
        leaving.extend(&last_lines);
        leaving.extend(&lit);
        leaving.extend([sixth_dot, six, doubled, strike, found]);
        fade_out_all(stage, &leaving, 0.6)?;
        // two rows, one above the other: the circle's and the cubes'
        let head: Vec<(String, Srgb)> = (1..=shown::POINTS).map(|k| (k.to_string(), dim())).collect();
        let mut number_cells: Vec<Mob> = Vec::new();
        for (k, (s, color)) in head.iter().enumerate() {
            number_cells.push(kit.text(stage, s, BIG_PT * 0.8, *color, v(big_x(k), BIG_ROWS.0))?);
        }
        let how_many = kit.text_left(stage, POINTS_WORD, BIG_PT * 0.8, dim(), v(BIG_LABEL_LEFT, BIG_ROWS.0))?;
        let mut circle_entries: Vec<(String, Srgb)> = hook.regions[..shown::POINTS - 1]
            .iter()
            .map(|r| (r.to_string(), ink()))
            .collect();
        circle_entries.push((hook.answer.to_string(), warn()));
        let (circle_label, circle_cells) = table_row(
            &kit,
            stage,
            REGIONS_WORD,
            BIG_LABEL_LEFT,
            BIG_ROWS.1,
            &circle_entries,
            big_x,
            BIG_PT,
        )?;
        let mut cube_entries: Vec<(String, Srgb)> = hook.totals.iter().map(|t| (t.to_string(), ink())).collect();
        cube_entries.push(("?".to_owned(), hero()));
        let (cube_label, cube_cells) = table_row(
            &kit,
            stage,
            CUBE_TOTALS_WORD,
            BIG_LABEL_LEFT,
            BIG_ROWS.2,
            &cube_entries,
            big_x,
            BIG_PT,
        )?;
        let mut table: Vec<Mob> = number_cells;
        table.extend([how_many, circle_label, cube_label]);
        fade_in_all(stage, &table, 0.5)?;
        line.wait_for(stage, "Five checks in a row")?;
        let mut rows_in: Vec<Mob> = circle_cells[..shown::POINTS - 1].to_vec();
        rows_in.extend(&cube_cells[..n]);
        fade_in_all(stage, &rows_in, 0.6)?;
        // "five for five", under each row, as a brace
        let mut braces: Vec<Mob> = Vec::new();
        for y in [BIG_ROWS.1, BIG_ROWS.2] {
            let (a, b) = (v(big_x(0) - 0.4, y - 0.4), v(big_x(n - 1) + 0.4, y - 0.4));
            let brace = line_brace(a, b, DOWN).buff(0.05).color(dim());
            let depth = brace.depth();
            let brace = stage.arena_mut().add(brace.build());
            let words = kit.text(
                stage,
                FIVE_FOR_FIVE,
                28.0,
                dim(),
                v((a[0] + b[0]) / 2.0, y - 0.4 - depth - 0.3),
            )?;
            braces.extend([brace, words]);
        }
        fade_in_all(stage, &braces, 0.5)?;
        line.wait_for(stage, "prove nothing about the sixth")?;
        let cross = mark_at(stage, false, v(big_x(shown::POINTS - 1) + 0.75, BIG_ROWS.1), 0.36);
        fade_in_all(stage, &[circle_cells[shown::POINTS - 1], cube_cells[n], cross], 0.6)?;
        line.wait_for(stage, "Such a reason")?;
        let tag = aside(&kit, stage, FOR_PROGRAMMERS, TESTS_ASIDE, ASIDE_AT)?;
        fade_in_all(stage, &[tag], 0.5)?;
        // (no breath after this line: its table and aside have been on the
        // screen through it, and the storyboard has no hold here)
        line.wait_end(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// A tracker to `value`, at an even rate, `rt` seconds.
fn tween_to(tracker: Mob, value: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    tween_linear(tracker, value, rt)
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// The scene constructs end to end (every phrase it waits for is in its
    /// line, every fact it reads is there, every string typesets) and the
    /// layout lint of the kit finds nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("02_five_for_five").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 20.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }

    /// P5 trims (storyboard: 02_7 has no hold, 02_6's is its breath): the
    /// chapter ends a fade after its last line, with no breath of silence
    /// over the table, and 02_6's statement gets its breath and a short read,
    /// not the 5.1 s the first draft left before 02_7.
    #[test]
    fn the_chapter_ends_with_its_line_and_the_statement_has_its_breath_and_no_more() {
        use super::super::sound::{after_the_last_line, gap_between};
        let v = testing::run_voiced("02_five_for_five").unwrap();
        let tail = after_the_last_line(&v);
        assert!(tail <= 1.4, "{tail:.1} s of picture after the last line");
        let gap = gap_between(&v, "02_6", "02_7");
        let most = 0.6 + explainer_kit::animatic::HOLD_S + THIRD_READ_S + 0.6;
        assert!(gap <= most, "{gap:.1} s between 02_6 and 02_7, at most {most:.1}");
        assert!(
            gap >= explainer_kit::animatic::HOLD_S,
            "the breath is still there: {gap:.1} s"
        );
    }

    /// The running total of 02_1 to 02_6 stands half a unit or more clear
    /// of the ledger's ticks (it stood 0.3 from them at "225 ✓") and inside
    /// the frame.
    #[test]
    fn the_running_total_stands_clear_of_the_ticks() {
        use fmn::scene::{NullSceneSink, RuntimeConfig};
        struct Probe;
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let kit = Kit::new()?;
                let total = kit.text(stage, "225", RUNNING_PT, ink(), v(RUNNING_AT[0], RUNNING_AT[1]))?;
                let tick_right = SQUARES_LEFT + 2.3 + 0.34 / 2.0;
                let (left, right) = (stage.get_left(total)[0], stage.get_right(total)[0]);
                assert!(
                    left - tick_right >= 0.5,
                    "the total starts {:.2} from the ticks",
                    left - tick_right
                );
                // inside the frame's margin (the frame is ±7.11, the safe area 0.15 in)
                assert!(right <= 7.11 - 0.15, "the total ends at {right:.2}");
                Ok(())
            }
        }
        fmn::run_scene(&mut Probe, RuntimeConfig::default(), 7, &mut NullSceneSink).expect("the probe runs");
    }

    /// The chapter speaks exactly the script's 02_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("02_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The hook's numbers are the facts' and agree with each other: the
    /// cubes add to the totals, the totals are the squares of the roots,
    /// the regions are counted from the drawing, and the doubling's sixth
    /// is 32 against the drawing's 31.
    #[test]
    fn the_hooks_numbers_are_the_facts_and_agree() {
        let h = Hook::read().unwrap();
        assert_eq!(h.cubes, [1, 8, 27, 64, 125]);
        assert_eq!(h.totals, [1, 9, 36, 100, 225]);
        assert_eq!(h.roots, [1, 3, 6, 10, 15]);
        assert_eq!(h.regions, [1, 2, 4, 8, 16, 31]);
        assert_eq!(h.doubling, [1, 2, 4, 8, 16, 32]);
        assert_eq!((h.answer, h.evenly_spaced), (31, 30));
        // the cubes' totals are squares, five times, and the sixth cube's is
        // a square too: the hook is that five is not a proof, not that the
        // pattern fails
        let mut sum = 0i64;
        for k in 1..=6 {
            sum += k * k * k;
            let r = (sum as f64).sqrt().round() as i64;
            assert_eq!(r * r, sum, "{k}");
        }
        assert_eq!(crate::facts::cited::MOSER.0, 1949);
    }

    /// The faces drawn at each point are as many as the regions counted,
    /// and the one set the doubling pattern needs (six points, 32) is not
    /// there: no set of the drawn points has a region count of 32.
    #[test]
    fn each_arrival_draws_the_regions_counted() {
        let drawn = circle::drawn(shown::POINTS);
        for k in 1..=shown::POINTS {
            assert_eq!(
                circle::faces(&drawn[..k]).len() as u64,
                circle::regions(&drawn[..k]),
                "{k} points"
            );
        }
        assert_ne!(circle::faces(&drawn).len(), 32);
    }

    /// Everything the chapter sets down stands inside the frame, clear of
    /// the lower-third's band and the quiz card's place.
    #[test]
    fn the_layout_stands_in_the_frame() {
        // a character of text is about this many units a point wide
        const CHAR: f64 = 0.0045;
        // the ledger's last row (the sixth slot) is above the frame's foot
        assert!(ledger_y(shown::CUBES as usize) - 0.35 > -FRAME_Y_RADIUS + 0.2);
        // the cubes' row is above the ledger's first row
        assert!(ROW_CENTRE.1 - ROW_ROOM.1 / 2.0 > ledger_y(0) + 0.4);
        // the circle and its table stand left of the quiz card (x from 0.8),
        // the circle under the frame's top and above its table
        const {
            assert!(CIRCLE_AT[0] + CIRCLE_R < 0.8 - 0.5);
            assert!(CIRCLE_AT[1] + CIRCLE_R < FRAME_Y_RADIUS - 0.3);
            assert!(CIRCLE_AT[1] - CIRCLE_R > SMALL_ROWS.0 + 0.4);
            assert!(BIG_LABEL_LEFT > -FRAME_X_RADIUS + 0.2);
        }
        assert!(small_x(shown::POINTS) < 0.8);
        // the table's rows end above the lower-third's band (top near -3.1)
        assert!(SMALL_ROWS.1 - 0.3 > -3.1 + 0.2 - 0.9);
        // the big table: columns and labels inside the frame, rows apart
        let last = big_x(shown::POINTS - 1) + 0.75 + 0.3;
        assert!(last < FRAME_X_RADIUS - 0.2, "{last}");
        assert!(BIG_ROWS.0 - BIG_ROWS.1 >= 0.8 && BIG_ROWS.1 - BIG_ROWS.2 >= 1.0);
        // the label "cube totals" ends before the first column
        let width = CUBE_TOTALS_WORD.chars().count() as f64 * BIG_PT * 0.8 * CHAR;
        assert!(BIG_LABEL_LEFT + width < big_x(0) - 0.5, "{width}");
    }
}
