//! Chapter 12, an economy: a field and a workshop, and what making things
//! uses up, folded into a table (12_1); your turn: make ten of each, how much
//! is used up (12_2); seven of each, the same pattern back, seven tenths the
//! size (12_3); equal amounts of each is an own pattern with eigenvalue seven
//! tenths, so the economy uses up seventy percent of it and keeps thirty
//! (12_4); and the arrow turned as before finds the other own pattern, with
//! eigenvalue two tenths, which needs a negative amount of tools (12_5).
//!
//! Every number is a fact or the model's: the table `K-E1.table`, the seven
//! and seven `K-E1.uses_even` (the quiz answer `K-Q.12_2.answer`), the
//! eigenvalues `K-E2.*`, the directions `K-E2.*_direction_deg`, the cited
//! years `K-S.*`. The orange arrow of the sweep is recomputed from the blue
//! one every frame (`arrows::ArrowTest`).
//!
//! The pieces the next two chapters draw again (the table with its headings,
//! the plane with its words and its hatched quarter, the angle readout) are
//! `pub(super)` here.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::arrows::{ArrowTest, band_ends, own_lines, turn_deg, unit_at};
use super::draw::{
    BAND_UP, Map2, MatrixGrid, Plane, QuizCard, arrow_points, chip, cross_fade, fact, fade_in_all, fade_out_all,
    faded_line, fail, flat, fraction_value, good, ground, group, items, line_style, moves, now, numbers, own, pause,
    pushes, run, say, show, tween_all, wait_until, warn, whole,
};
use crate::facts::shown;

pub struct AnEconomy {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The field and the workshop of 12_1.
const FIELD_AT: Vec3 = [-5.1, 1.2, 0.0];
const WORKSHOP_AT: Vec3 = [-1.5, 1.2, 0.0];
/// Where the table of 12_1 lands, and its type size.
const TABLE_AT: Vec3 = [4.4, 0.55, 0.0];
const TABLE_SIZE: f64 = 56.0;
/// The type size of the four numbers beside the arrows of 12_1.
const NUMBER_SIZE: f64 = 34.0;
/// How long the field, the workshop, their names and arrows take to leave
/// before the four numbers fold into the table (seconds).
const ICONS_GO: f64 = 0.4;
/// How long the numbers take to fly to their cells (seconds).
const FLIGHT: f64 = 1.0;
/// The quiz card, laid out as 02_6's.
const CARD_AT: Vec3 = [0.0, 0.05, 0.0];
const CARD_SIZE: (f64, f64) = (11.6, 6.6);
const CARD_GRID: f64 = 42.0;
/// The two bars of 12_4.
const BAR_W: f64 = 1.5;
const BAR_H: f64 = 3.2;
const BAR_BASE: f64 = -1.7;
/// The plane of 12_5 (and the two chapters after it): food across, tools
/// up, `unit` scene units to one unit of goods drawn.
pub(super) const PLANE: Plane = Plane {
    centre: [-2.3, 0.3, 0.0],
    unit: 0.8,
};
/// How far the plane's axes reach, plane units either way.
pub(super) const REACH: (f64, f64) = (4.0, 3.0);
/// How long the sweep's blue arrow is, plane units.
const SWEEP_LENGTH: f64 = 2.2;
/// Where the angle readout's number ends (its right edge) in 12_5; the
/// degree sign stands against it.
const READOUT_AT: Vec3 = [5.2, 1.9, 0.0];

// --------------------------------------------------------------- the tables

/// A table of fractions (`2/5,3/10;1/5,1/2`) as a 2 by 2 map.
pub(super) fn table_of(key: &str) -> fmn::Result<Map2> {
    let rows: Vec<Vec<f64>> = fact(key)?
        .split(';')
        .map(|row| row.split(',').map(fraction_value).collect::<fmn::Result<Vec<f64>>>())
        .collect::<fmn::Result<_>>()?;
    if rows.len() != 2 || rows.iter().any(|r| r.len() != 2) {
        return Err(fail(format!("fact {key} is not a 2 by 2 table")));
    }
    Ok([[rows[0][0], rows[0][1]], [rows[1][0], rows[1][1]]])
}

/// The entries of a table as the strings it is drawn with: one decimal.
pub(super) fn table_entries(t: &Map2) -> fmn::Result<Vec<Vec<String>>> {
    t.iter()
        .map(|row| row.iter().map(|&x| show(x, 1)).collect::<fmn::Result<Vec<_>>>())
        .collect()
}

/// A list of two numbers as one column, whole numbers.
fn list_entries(values: [f64; 2]) -> Vec<Vec<String>> {
    values.iter().map(|&x| vec![whole(x)]).collect()
}

/// The headings of the economy's table: what a column makes along the top,
/// what a row uses up down the side. Detached, around a grid at `at`.
pub(super) fn table_headings(kit: &Kit, stage: &mut Stage<'_>, grid: &MatrixGrid, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let half = grid.size.0 / 2.0;
    let mut out = Vec::new();
    for (col, words) in ["to make\n1 food", "to make\n1 tools"].into_iter().enumerate() {
        let x = grid.cell_at(stage, 0, col)[0];
        out.push(kit.text(stage, words, 24.0, dim, v(x, at[1] + grid.size.1 / 2.0 + 0.55))?);
    }
    for (row, words) in ["food\nused up", "tools\nused up"].into_iter().enumerate() {
        let y = grid.cell_at(stage, row, 0)[1];
        let m = kit.text(stage, words, 24.0, dim, v(at[0] - half - 0.9, y))?;
        out.push(m);
    }
    Ok(out)
}

// ------------------------------------------------------------ the icons

/// An arc of a circle at `c`, radius `r`, from angle `a0` to `a1` degrees,
/// with an arrow's head at its end: a loop that comes back to where it
/// began.
fn loop_points(c: Vec3, r: f64, a0: f64, a1: f64, head: f64) -> Vec<Vec3> {
    let steps = 24;
    let pts: Vec<Vec3> = (0..=steps)
        .map(|i| {
            let a = (a0 + (a1 - a0) * f64::from(i) / f64::from(steps)).to_radians();
            v(c[0] + r * a.cos(), c[1] + r * a.sin())
        })
        .collect();
    let tip = pts[pts.len() - 1];
    let before = pts[pts.len() - 3];
    let barbs = arrow_points(before, tip, head);
    let mut out = pts;
    out.extend([barbs[2], barbs[3], barbs[4]]);
    out
}

/// The picture of 12_1, built and detached: a field and a workshop drawn in
/// line, their names, "a model", and four arrows of what making uses up,
/// each with its number.
struct Icons {
    field: Mob,
    workshop: Mob,
    names: [Mob; 2],
    tag: Mob,
    /// The four arrows, in the order of the table's cells: food by food (the
    /// field's loop), tools by food (workshop to field), food by tools (field
    /// to workshop), tools by tools (the workshop's loop).
    arrows: [Mob; 4],
    numbers: [Mob; 4],
}

impl Icons {
    fn build(kit: &Kit, stage: &mut Stage<'_>, t: &Map2) -> fmn::Result<Self> {
        let ink = Palette::ink();
        let dim = Palette::dim();
        let outline = |stage: &mut Stage<'_>, pts: Vec<Vec3>, w: f64| {
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(ink, w, 1.0)).build())
        };
        // the field: a plot with three furrows
        let (fx, fy) = (FIELD_AT[0], FIELD_AT[1]);
        let mut parts = vec![outline(
            stage,
            vec![
                v(fx - 0.75, fy - 0.5),
                v(fx + 0.75, fy - 0.5),
                v(fx + 0.75, fy + 0.5),
                v(fx - 0.75, fy + 0.5),
                v(fx - 0.75, fy - 0.5),
            ],
            4.0,
        )];
        for k in 1..=3 {
            let y = fy - 0.5 + 0.25 * f64::from(k);
            parts.push(outline(stage, vec![v(fx - 0.6, y), v(fx + 0.6, y)], 3.0));
        }
        let field = group(stage, &parts)?;
        // the workshop: a house with a door
        let (wx, wy) = (WORKSHOP_AT[0], WORKSHOP_AT[1]);
        let house = outline(
            stage,
            vec![
                v(wx - 0.75, wy - 0.5),
                v(wx - 0.75, wy + 0.2),
                v(wx, wy + 0.7),
                v(wx + 0.75, wy + 0.2),
                v(wx + 0.75, wy - 0.5),
                v(wx - 0.75, wy - 0.5),
            ],
            4.0,
        );
        let door = outline(
            stage,
            vec![
                v(wx - 0.18, wy - 0.5),
                v(wx - 0.18, wy - 0.05),
                v(wx + 0.18, wy - 0.05),
                v(wx + 0.18, wy - 0.5),
            ],
            3.0,
        );
        let workshop = group(stage, &[house, door])?;
        let names = [
            kit.text(stage, "farms: food", 26.0, dim, v(fx, fy - 0.95))?,
            kit.text(stage, "workshops: tools", 26.0, dim, v(wx, wy - 0.95))?,
        ];
        let tag = kit.text(stage, "a model", 26.0, dim, v((fx + wx) / 2.0, fy - 1.8))?;

        let orange = pushes();
        let style = |w: f64| faded_line(orange, w, 1.0);
        let mid = (fx + wx) / 2.0;
        let field_loop = stage.arena_mut().add(
            polyline(loop_points(v(fx, fy + 1.05), 0.36, -60.0, 240.0, 0.2))
                .style(style(4.0))
                .build(),
        );
        let to_field = stage.arena_mut().add(
            polyline(arrow_points(v(wx - 0.95, fy - 0.25), v(fx + 0.95, fy - 0.25), 0.22))
                .style(style(4.0))
                .build(),
        );
        let to_workshop = stage.arena_mut().add(
            polyline(arrow_points(v(fx + 0.95, fy + 0.25), v(wx - 0.95, fy + 0.25), 0.22))
                .style(style(4.0))
                .build(),
        );
        let workshop_loop = stage.arena_mut().add(
            polyline(loop_points(v(wx, wy + 1.05), 0.36, -60.0, 240.0, 0.2))
                .style(style(4.0))
                .build(),
        );
        let arrows = [field_loop, to_field, to_workshop, workshop_loop];
        // the numbers: food by food above the field's loop, tools by food
        // under the lower arrow, food by tools over the upper arrow, tools
        // by tools above the workshop's loop
        let n =
            |kit: &Kit, stage: &mut Stage<'_>, x: f64, at: Vec3| kit.text(stage, &show(x, 1)?, NUMBER_SIZE, orange, at);
        let numbers = [
            n(kit, stage, t[0][0], v(fx, fy + 1.85))?,
            n(kit, stage, t[1][0], v(mid, fy - 0.62))?,
            n(kit, stage, t[0][1], v(mid, fy + 0.62))?,
            n(kit, stage, t[1][1], v(wx, wy + 1.85))?,
        ];
        Ok(Self {
            field,
            workshop,
            names,
            tag,
            arrows,
            numbers,
        })
    }
}

/// The pictures of 12_1 go first and the four numbers fly to their `cells`
/// after, so that no number crosses a house, a name or an arrow still on the
/// screen.
fn fold_into_table(stage: &mut Stage<'_>, icons: &Icons, cells: [Vec3; 4]) -> fmn::Result<()> {
    let mut gone = vec![icons.field, icons.workshop, icons.tag];
    gone.extend(icons.names);
    gone.extend(icons.arrows);
    let mut leave: Vec<Box<dyn Animation>> = Vec::new();
    for g in gone {
        stage.clear_updaters(g, true);
        let a = fade_out(stage.arena_mut(), g, ORIGIN, 1.0)?;
        leave.push(stage.prepare(a.rt(ICONS_GO))?);
    }
    stage.play_prepared(leave)?;
    let mut fold: Vec<Box<dyn Animation>> = Vec::new();
    for (&n, c) in icons.numbers.iter().zip(cells) {
        // each lands as big as the table's entries, so that the table can
        // take its place without two sizes of the same number on the screen
        fold.push(
            stage.prepare(
                n.animate()
                    .set_anim_args(run(FLIGHT))?
                    .move_to(c, ORIGIN)?
                    .scale(TABLE_SIZE / NUMBER_SIZE)?,
            )?,
        );
    }
    stage.play_prepared(fold)?;
    Ok(())
}

// --------------------------------------------------- the plane and its words

/// The words at the ends of the economy's axes, detached: what the plane's
/// across and up are.
pub(super) fn axis_words(kit: &Kit, stage: &mut Stage<'_>, plane: Plane, reach: (f64, f64)) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let across = kit.text(stage, "food", 26.0, dim, plane.p(reach.0 + 0.5, -0.28))?;
    let up = kit.text(stage, "tools", 26.0, dim, plane.p(0.0, reach.1 + 0.3))?;
    Ok(vec![across, up])
}

/// The quarter of the plane where tools would be negative (food positive),
/// hatched in red with diagonal strokes, detached, opacity 0.
pub(super) fn hatched_quarter(stage: &mut Stage<'_>, plane: Plane, reach: (f64, f64)) -> Vec<Mob> {
    let (w, h) = reach;
    let mut out = Vec::new();
    let step = 0.7;
    let mut c = step / 2.0;
    while c < w + h - 0.1 {
        // the stroke y = x - c, kept inside the quarter [0, w] by [-h, 0]
        let x0 = (c - h).max(0.0);
        let x1 = c.min(w);
        let (a, b) = (plane.p(x0, x0 - c), plane.p(x1, x1 - c));
        out.push(
            stage
                .arena_mut()
                .add(polyline(vec![a, b]).style(faded_line(warn(), 3.0, 0.0)).build()),
        );
        c += step;
    }
    out
}

/// How far the degree sign stands from the last digit of the angle readout.
const DEGREE_GAP: f64 = 0.05;

/// The degree sign of the angle readout, detached, its left edge `DEGREE_GAP`
/// to the right of `at`: the readout's number is anchored on its RIGHT edge
/// at the same `at`, so the sign stays against the last digit whether the
/// angle is one digit or three.
pub(super) fn degree_sign(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let sign = kit.text(stage, "\u{b0}", 56.0, Palette::ink(), at)?;
    stage.move_to(sign, v(at[0] + DEGREE_GAP, at[1]), LEFT);
    Ok(sign)
}

/// The live angle between the move and the push it gets back, in degrees: a
/// number rebuilt from the arrow test's move every frame, detached, with its
/// right edge at `at` (the degree sign of [`degree_sign`] goes just past it).
pub(super) fn angle_readout(kit: &Kit, stage: &mut Stage<'_>, test: &ArrowTest, at: Vec3) -> fmn::Result<Mob> {
    let (u, table) = (test.u, test.table);
    let readout = kit.live_value(
        stage,
        move |s| {
            let mv = [
                s.tracker_value(u[0]).unwrap_or(0.0),
                s.tracker_value(u[1]).unwrap_or(0.0),
            ];
            turn_deg(&table, mv).round()
        },
        0,
        false,
        56.0,
        Palette::ink(),
        at,
        RIGHT,
    )?;
    stage.remove_from_scene(readout);
    Ok(readout)
}

// ------------------------------------------------- the words of the sweep

/// How far a written value stands off the end of its line (scene units).
const VALUE_GAP: f64 = 0.15;
/// How far the chip of the negative quarter stands to the right of the
/// plane's last unit (scene units).
const CHIP_GAP: f64 = 0.45;
/// How far down the quarter's height the chip's middle stands.
const CHIP_DROP: f64 = 0.4;

/// The three words 12_5's plane writes, built and detached.
struct SweepWords {
    /// "0.7", past the end of the gold line of the even mix.
    mix: Mob,
    /// "0.2", past the end of the dashed line of the other own pattern.
    other: Mob,
    /// "tools would be negative", beside the hatched quarter.
    negative: Mob,
}

/// Put `label` up and to the right of `end` (a point on the screen, the end
/// of a line that reaches the plane's top or right edge): its lower left
/// corner stands `VALUE_GAP` off the end each way, so nothing of the line,
/// its glow or the hatch passes through the number.
fn past_the_end(stage: &mut Stage<'_>, label: Mob, end: Vec3) {
    stage.move_to(label, v(end[0] + VALUE_GAP, end[1] + VALUE_GAP), v(-1.0, -1.0));
}

fn sweep_words(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<SweepWords> {
    let mix_deg = numbers("K-E2.mix_direction_deg")?[0];
    let other = numbers("K-E2.other")?;
    let largest = numbers("K-E2.largest")?[0];
    let (_, end_mix) = band_ends(unit_at(mix_deg), REACH.0, REACH.1);
    let (_, end_other) = band_ends([other[0], other[1]], REACH.0, REACH.1);
    let eigen_values = items("K-E2.eigenvalues")?;
    let other_value = fraction_value(eigen_values[1])?;
    let mix = kit.text(stage, &show(largest, 1)?, 34.0, own(), ORIGIN)?;
    past_the_end(stage, mix, PLANE.p(end_mix[0], end_mix[1]));
    let other_label = kit.text(stage, &show(other_value, 1)?, 34.0, own(), ORIGIN)?;
    past_the_end(stage, other_label, PLANE.p(end_other[0], end_other[1]));
    // the chip stands outside the plane, beside the upper part of the
    // quarter it names (clear of "food" above it and "0.2" below), so no
    // arrow, line or stroke of the hatch runs through it
    let negative = chip(kit, stage, "tools would be negative", 24.0, warn(), ORIGIN)?;
    let at = PLANE.p(REACH.0, -CHIP_DROP * REACH.1);
    stage.move_to(negative, v(at[0] + CHIP_GAP, at[1]), LEFT);
    Ok(SweepWords {
        mix,
        other: other_label,
        negative,
    })
}

// -------------------------------------------------------------- the scene

impl SceneConstruct for AnEconomy {
    fn name(&self) -> &str {
        "12_an_economy"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange) = (moves(), pushes());
        let table = table_of("K-E1.table")?;

        // ---- 12_1: farms make food, workshops make tools, and making things
        // uses things up; the four numbers fold into a table
        let line = say(&kit, stage, "12_1")?;
        let icons = Icons::build(&kit, stage, &table)?;
        fade_in_all(stage, &[icons.field, icons.workshop], 0.8)?;
        fade_in_all(stage, &[icons.tag], 0.4)?;
        line.wait_for(stage, "Farms make food")?;
        fade_in_all(stage, &icons.names, 0.5)?;
        line.wait_for(stage, "uses things up")?;
        fade_in_all(stage, &icons.arrows, 0.7)?;
        line.wait_for(stage, "four tenths of food")?;
        fade_in_all(stage, &[icons.numbers[0]], 0.4)?;
        line.wait_for(stage, "two tenths of tools")?;
        fade_in_all(stage, &[icons.numbers[1]], 0.4)?;
        line.wait_for(stage, "three tenths of food")?;
        fade_in_all(stage, &[icons.numbers[2]], 0.4)?;
        line.wait_for(stage, "five tenths of tools")?;
        fade_in_all(stage, &[icons.numbers[3]], 0.4)?;
        line.wait_end(stage)?;
        // the numbers fold into the table: food by food, food by tools over
        // tools by food, tools by tools
        let grid = MatrixGrid::build(&kit, stage, &table_entries(&table)?, TABLE_SIZE, ink, TABLE_AT)?;
        let heads = table_headings(&kit, stage, &grid, TABLE_AT)?;
        let cells = [
            grid.cell_at(stage, 0, 0),
            grid.cell_at(stage, 1, 0),
            grid.cell_at(stage, 0, 1),
            grid.cell_at(stage, 1, 1),
        ];
        fold_into_table(stage, &icons, cells)?;
        let mut table_in = vec![grid.group];
        table_in.extend(&heads);
        // the orange numbers give way to the table's as it comes in (the same
        // numbers at the same size), not under it: the table used to fade in
        // beneath them and they printed over its white
        cross_fade(stage, &icons.numbers, &table_in, 0.5)?;

        // ---- 12_2: your turn: make ten of each; how much is used up?
        let line = say(&kit, stage, "12_2")?;
        fade_out_all(stage, &table_in, 0.5)?;
        // the table works like the carts': a list of what is made goes in,
        // a list of what is used up comes out
        let row_y = 0.2;
        let first = MatrixGrid::build(&kit, stage, &table_entries(&table)?, TABLE_SIZE, ink, v(0.0, row_y))?;
        let made_word = kit.text(stage, "made", 34.0, blue, v(-3.9, row_y + 0.95))?;
        let used_word = kit.text(stage, "used up", 34.0, orange, v(3.9, row_y + 0.95))?;
        let into = arrow(stage, v(-2.9, row_y), v(-1.8, row_y), dim);
        let out = arrow(stage, v(1.8, row_y), v(2.9, row_y), dim);
        fade_in_all(stage, &[first.group, made_word, used_word, into, out], 0.7)?;
        line.wait_for(stage, "along each row")?;
        let band_top = first.row_band(stage, 0, own())?;
        fade_in_all(stage, &[band_top], 0.4)?;
        let band_bottom = first.row_band(stage, 1, own())?;
        pause(stage, 0.6)?;
        play!(stage; band_top.animate().set_anim_args(run(0.3))?.set_opacity(0.0)?);
        fade_in_all(stage, &[band_bottom], 0.4)?;
        line.wait_for(stage, "Your turn")?;
        fade_out_all(stage, &[first.group, made_word, used_word, into, out, band_bottom], 0.5)?;
        let card = QuizCard::build(&kit, stage, "12_2", CARD_AT, CARD_SIZE)?;
        let (middle, _, _) = card.content();
        let card_row = middle[1] - 0.1;
        let made = shown::EVEN_MIX;
        let made = [made[0] as f64, made[1] as f64];
        let c_made_word = kit.text(stage, "made", 30.0, blue, v(-3.6, card_row + 0.95))?;
        let c_used_word = kit.text(stage, "used up", 30.0, orange, v(3.6, card_row + 0.95))?;
        let c_made = MatrixGrid::build(&kit, stage, &list_entries(made), CARD_GRID, blue, v(-3.6, card_row))?;
        let c_table = MatrixGrid::build(&kit, stage, &table_entries(&table)?, CARD_GRID, ink, v(0.0, card_row))?;
        let unknown = [
            kit.text(stage, "?", 54.0, orange, v(3.6, card_row + 0.3))?,
            kit.text(stage, "?", 54.0, orange, v(3.6, card_row - 0.3))?,
        ];
        let c_into = arrow(stage, v(-2.75, card_row), v(-1.25, card_row), dim);
        let c_out = arrow(stage, v(1.25, card_row), v(2.75, card_row), dim);
        let problem = vec![
            c_made_word,
            c_used_word,
            c_made.group,
            c_table.group,
            unknown[0],
            unknown[1],
            c_into,
            c_out,
        ];
        card.show_frame(stage, &problem, 0.8)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 12_3: seven of each: the same pattern back, seven tenths the size
        let line = say(&kit, stage, "12_3")?;
        let answer = numbers("K-Q.12_2.answer")?;
        let c_answer = MatrixGrid::build(
            &kit,
            stage,
            &list_entries([answer[0], answer[1]]),
            CARD_GRID,
            orange,
            v(3.6, card_row),
        )?;
        // the question marks give way at once, so the answer lands with the
        // end of the countdown and not after it
        for &m in &unknown {
            stage.set_opacity(m, 0.0, true);
        }
        card.land(stage, &[c_answer.group], 0.5)?;
        let sum = |row: usize| -> fmn::Result<String> {
            Ok(format!(
                r"{} \times {} + {} \times {} = {}",
                show(table[row][0], 1)?,
                whole(made[0]),
                show(table[row][1], 1)?,
                whole(made[1]),
                whole(answer[row]),
            ))
        };
        let sum_a = kit.tex(stage, &sum(0)?, 34.0, ink, &[], v(-2.6, middle[1] - 1.55))?;
        let sum_b = kit.tex(stage, &sum(1)?, 34.0, ink, &[], v(2.6, middle[1] - 1.55))?;
        line.wait_for(stage, "The same pattern comes back")?;
        fade_in_all(stage, &[sum_a, sum_b], 0.6)?;
        // the two lists pulse together in gold
        let mut pulse = Vec::new();
        for g in [c_made.group, c_answer.group] {
            pulse.push(stage.prepare(g.animate().set_anim_args(run(0.5))?.set_color(own())?)?);
        }
        stage.play_prepared(pulse)?;

        // ---- 12_4: equal amounts of each is an own pattern, eigenvalue
        // seven tenths: seventy percent used up, thirty kept
        let line = say(&kit, stage, "12_4")?;
        let mut leaving = card.mobs();
        leaving.extend([
            card.track,
            card.pause_line,
            c_made_word,
            c_used_word,
            c_made.group,
            c_table.group,
            c_answer.group,
            c_into,
            c_out,
            sum_a,
            sum_b,
        ]);
        fade_out_all(stage, &leaving, 0.6)?;
        let largest = numbers("K-E2.largest")?[0];
        let used = numbers("K-E2.largest_percent")?[0];
        let kept = numbers("K-E2.kept_percent")?[0];
        let mut bars = Vec::new();
        let mut words = Vec::new();
        for (x, name) in [(-1.9, "food"), (1.9, "tools")] {
            let centre = BAR_BASE + BAR_H / 2.0;
            let used_h = BAR_H * used / 100.0;
            let kept_h = BAR_H * kept / 100.0;
            let bar = |stage: &mut Stage<'_>, base: f64, h: f64, c: Srgb| -> fmn::Result<Mob> {
                let r = Rectangle::new()
                    .width(BAR_W)
                    .height(h)
                    .style(flat(c, 0.92))
                    .build()?
                    .moved_to(v(x, base + h / 2.0));
                Ok(stage.arena_mut().add(r))
            };
            let used_bar = bar(stage, BAR_BASE, used_h, orange)?;
            let kept_bar = bar(stage, BAR_BASE + used_h, kept_h, good())?;
            let frame = Rectangle::new()
                .width(BAR_W + 0.12)
                .height(BAR_H + 0.12)
                .style(line_style(blue, 5.0))
                .build()?
                .moved_to(v(x, centre));
            let frame = stage.arena_mut().add(frame);
            let used_tag = kit.text(stage, &whole(used), 40.0, ground(), v(x, BAR_BASE + used_h / 2.0))?;
            let kept_tag = kit.text(
                stage,
                &whole(kept),
                40.0,
                ground(),
                v(x, BAR_BASE + used_h + kept_h / 2.0),
            )?;
            let name = kit.text(stage, name, 30.0, dim, v(x, BAR_BASE - 0.4))?;
            // the bars come with their names ("food", "tools"): two bars
            // that nothing names stood on the screen for the voice's first
            // sentence
            bars.extend([frame, used_bar, kept_bar, name]);
            words.extend([used_tag, kept_tag]);
        }
        let eigenvalue = kit.text(
            stage,
            &format!("eigenvalue {}", show(largest, 1)?),
            40.0,
            own(),
            v(0.0, 2.9),
        )?;
        let used_name = kit.text(stage, "used up", 30.0, orange, v(5.0, BAR_BASE + 0.9))?;
        let kept_name = kit.text(stage, "kept", 30.0, good(), v(5.0, BAR_BASE + 2.5))?;
        line.wait_for(stage, "So equal amounts of each")?;
        fade_in_all(stage, &bars, 0.7)?;
        line.wait_for(stage, "eigenvalue seven tenths")?;
        fade_in_all(stage, &[eigenvalue], 0.5)?;
        line.wait_for(stage, "uses up seventy percent")?;
        fade_in_all(stage, &words, 0.5)?;
        fade_in_all(stage, &[used_name], 0.4)?;
        line.wait_for(stage, "and keeps thirty")?;
        fade_in_all(stage, &[kept_name], 0.4)?;

        // ---- 12_5: turn the arrow as before: the other own pattern needs
        // a negative amount of tools
        let line = say(&kit, stage, "12_5")?;
        let mut leaving = bars.clone();
        leaving.extend(&words);
        leaving.extend([eigenvalue, used_name, kept_name]);
        fade_out_all(stage, &leaving, 0.4)?;
        let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let test = ArrowTest::build(stage, PLANE, table, u, REACH)?;
        let across = axis_words(&kit, stage, PLANE, REACH)?;
        let readout = angle_readout(&kit, stage, &test, READOUT_AT)?;
        let readout_name = kit.text(stage, "angle between the arrows", 26.0, dim, v(4.9, 2.7))?;
        let degrees = degree_sign(&kit, stage, READOUT_AT)?;
        let mix_deg = numbers("K-E2.mix_direction_deg")?[0];
        let other_deg = numbers("K-E2.other_direction_deg")?[0];
        let (mix_dir, other_dir) = (numbers("K-E2.mix")?, numbers("K-E2.other")?);
        let lines_mix = own_lines(stage, PLANE, &[[mix_dir[0], mix_dir[1]]], REACH.0, REACH.1);
        let (a, b) = band_ends([other_dir[0], other_dir[1]], REACH.0, REACH.1);
        let dashed = DashedLine::new(PLANE.p(a[0], a[1]), PLANE.p(b[0], b[1]))
            .dash_length(0.16)
            .style(faded_line(own(), 4.0, 0.0))
            .build()?;
        let dashed = stage.arena_mut().add(dashed);
        let hatch = hatched_quarter(stage, PLANE, REACH);
        let sweep = sweep_words(&kit, stage)?;
        let (label_mix, label_other, tag_negative) = (sweep.mix, sweep.other, sweep.negative);
        let cited = citation(&kit, stage)?;

        // the picture keeps up with the voice: "Turn the arrow as before" is
        // 2 s, and the rest of the line (2 to 7.6 s) is the other own pattern
        // and the tools that would be negative. The citation, which the voice
        // does not read, is up from the second turn on, so that the chapter
        // ends when the line does (D-16: no closing hold)
        let mut axes = test.axes.clone();
        axes.extend(&across);
        fade_in_all(stage, &axes, 0.5)?;
        test.start(stage)?;
        stage.set_tracker_value(test.polar, 1.0)?;
        stage.set_tracker_value(test.radius, SWEEP_LENGTH)?;
        stage.set_tracker_value(test.degrees, 0.0)?;
        stage.add_to_scene(readout)?;
        fade_in_all(stage, &[readout_name, degrees, readout], 0.3)?;
        tween_all(stage, &[(test.blue_on, 1.0), (test.orange_on, 1.0)], 0.3)?;
        // the blue arrow turns to the even mix: the two line up, 0.7
        line.wait_for(stage, "Turn the arrow")?;
        play!(stage; tween(test.degrees, mix_deg, 1.4)?);
        play!(stage;
            lines_mix[0].animate().set_anim_args(run(0.3))?.set_opacity(0.25)?,
            lines_mix[1].animate().set_anim_args(run(0.3))?.set_opacity(1.0)?
        );
        fade_in_all(stage, &[label_mix], 0.3)?;
        // on, to the other own direction: the dashed line and 0.2
        line.wait_for(stage, "The other own pattern")?;
        play!(stage; tween(test.degrees, other_deg, 1.8)?);
        play!(stage; dashed.animate().set_anim_args(run(0.3))?.set_opacity(1.0)?);
        fade_in_all(stage, &[label_other, cited], 0.3)?;
        let cited_up = now(stage);
        // and round to where it points into the quarter with tools negative
        line.wait_for(stage, "needs a negative amount of tools")?;
        let into_quarter = other_deg + 180.0;
        let mut fades: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(test.degrees, into_quarter, 2.0)?)?];
        for &h in &hatch {
            fades.push(stage.prepare(h.animate().set_anim_args(run(1.6))?.set_opacity(0.6)?)?);
        }
        stage.play_prepared(fades)?;
        fade_in_all(stage, &[tag_negative], 0.3)?;
        line.wait_end(stage)?;
        // the citation, which the voice does not read, has stood long enough
        // to read before the chapter lets go of it
        wait_until(stage, cited_up + BAND_UP)?;

        kit.hold(stage)?;
        test.stop(stage);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// What Perron and Frobenius showed, and what came of it, as the lower-third
/// of 12_5 says it: short enough for two even lines (at the band's width a
/// longer one left "1998)" alone on a third line).
fn citation_text(brin_page_year: &str) -> String {
    format!(
        "a table with no negative numbers has an own pattern with none, with the largest eigenvalue; that pattern \
         ranked web pages (Brin and Page {brin_page_year})"
    )
}

/// 12_5's lower-third, built and detached: the two names and their years,
/// and what came of it.
fn citation(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    lower_third(
        kit,
        stage,
        &format!(
            "Perron {}, Frobenius {}:",
            fact("K-S.perron_year")?,
            fact("K-S.frobenius_year")?
        ),
        &citation_text(fact("K-S.brin_page_year")?),
    )
}

/// A plain arrow from `a` to `b` in `color`, detached.
fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb) -> Mob {
    stage.arena_mut().add(
        polyline(arrow_points(a, b, 0.22))
            .style(faded_line(color, 4.0, 1.0))
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::super::draw::testing::Room;
    use super::super::draw::{matrix, testing};
    use super::*;
    use crate::model::economy;

    /// 12_1 to 12_4: the tables, the seven and seven of the quiz, the
    /// percentages and the sums are the facts and the model's.
    #[test]
    fn the_table_the_sums_and_the_bars_are_the_facts() {
        let t = table_of("K-E1.table").unwrap();
        assert_eq!(t, [[0.4, 0.3], [0.2, 0.5]]);
        // the same table in tenths
        let tenths = matrix("K-E1.table_tenths").unwrap();
        assert_eq!(tenths, vec![vec![4.0, 3.0], vec![2.0, 5.0]]);
        assert_eq!(table_entries(&t).unwrap(), vec![vec!["0.4", "0.3"], vec!["0.2", "0.5"]]);
        // the model's table is the same one
        let c = economy::thrifty();
        for (i, row) in t.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                assert!((c.c[i][j].to_f64() - x).abs() < 1e-12, "entry {i},{j}");
            }
        }
        // make ten of each, use up seven of each
        let made = shown::EVEN_MIX;
        assert_eq!(made, [10, 10]);
        let used: Vec<f64> = t
            .iter()
            .map(|row| row[0] * made[0] as f64 + row[1] * made[1] as f64)
            .collect();
        assert_eq!(used.iter().map(|&x| whole(x)).collect::<Vec<_>>(), ["7", "7"]);
        assert_eq!(fact("K-E1.uses_even").unwrap(), "7,7");
        assert_eq!(fact("K-Q.12_2.answer").unwrap(), "7,7");
        assert_eq!(super::super::draw::quiz_at("12_2").unwrap().seconds, 6);
        // seven tenths used up, thirty percent kept, and the other own
        // pattern: two tenths, food 3 for tools minus 2
        assert_eq!(numbers("K-E2.largest").unwrap(), [0.7]);
        assert_eq!(numbers("K-E2.largest_percent").unwrap(), [70.0]);
        assert_eq!(numbers("K-E2.kept_percent").unwrap(), [30.0]);
        assert_eq!(items("K-E2.eigenvalues").unwrap(), ["7/10", "1/5"]);
        assert_eq!(numbers("K-E2.other").unwrap(), [3.0, -2.0]);
        assert_eq!(numbers("K-E2.mix").unwrap(), [1.0, 1.0]);
        // the own directions of the table are where the facts say, and the
        // push is the move scaled by the eigenvalue
        for (dir, deg, lambda) in [
            (
                numbers("K-E2.mix").unwrap(),
                numbers("K-E2.mix_direction_deg").unwrap()[0],
                0.7,
            ),
            (
                numbers("K-E2.other").unwrap(),
                numbers("K-E2.other_direction_deg").unwrap()[0],
                0.2,
            ),
        ] {
            let push = apply_map(&t, [dir[0], dir[1]]);
            assert!((push[0] - lambda * dir[0]).abs() < 1e-12 && (push[1] - lambda * dir[1]).abs() < 1e-12);
            assert!(turn_deg(&t, [dir[0], dir[1]]) < 1e-6, "{dir:?} lines up");
            let a = dir[1].atan2(dir[0]).to_degrees().rem_euclid(180.0);
            assert_eq!(show(a, 1).unwrap(), format!("{deg:.1}"));
        }
        // the sweep's turn: nothing turns at either own direction, and it
        // is a real turn in between
        assert!(turn_deg(&t, unit_at(0.0)) > 5.0);
        assert!(turn_deg(&t, unit_at(90.0)) > 5.0);
        // the other pattern needs a negative amount: food 3, tools -2
        let other = numbers("K-E2.other").unwrap();
        assert!(other[0] > 0.0 && other[1] < 0.0);
    }

    fn apply_map(t: &Map2, p: [f64; 2]) -> [f64; 2] {
        super::super::draw::apply(t, p)
    }

    /// The hatched quarter lies wholly where food is positive and tools
    /// negative, inside the plane's reach.
    #[test]
    fn the_hatch_stays_in_the_quarter_where_tools_would_be_negative() {
        let (w, h) = REACH;
        let mut c = 0.35;
        let mut strokes = 0;
        while c < w + h - 0.1 {
            let x0 = (c - h).max(0.0);
            let x1 = c.min(w);
            assert!(x1 > x0, "c = {c}");
            for x in [x0, x1] {
                let y = x - c;
                assert!(x >= -1e-12 && x <= w + 1e-12, "x {x}");
                assert!(y <= 1e-12 && y >= -h - 1e-12, "y {y}");
            }
            strokes += 1;
            c += 0.7;
        }
        assert!(strokes >= 8);
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("12_an_economy").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "a model",
            "to make\n1 food",
            "tools\nused up",
            "0.4 \\times 10 + 0.3 \\times 10 = 7",
            "0.2 \\times 10 + 0.5 \\times 10 = 7",
            "eigenvalue 0.7",
            "tools would be negative",
            "Perron 1907, Frobenius 1912",
            "Brin and Page 1998",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }

    /// 12_5: the picture keeps up with the voice, so the chapter ends when its line does (D-16: no closing hold); it ran 4 to 5 s behind and left 7.6 s of picture after the last line.
    #[test]
    fn the_picture_after_the_last_line_stays_under_the_cue_checks_tail() {
        let voiced = testing::run_voiced("12_an_economy").unwrap();
        let tail = voiced.tail_seconds();
        eprintln!("TAIL 12_an_economy: {tail:.2} s");
        assert!(tail <= 2.5, "12_an_economy: {tail:.2} s of picture after the last line");
    }

    /// 12_5: the two values and the chip stand clear of the lines, the hatch
    /// and the plane. A dense read caught "0.7" under the end of the gold
    /// line, "0.2" on the dashed line inside the hatch, and the chip on the
    /// path of the arrow turning into the quarter.
    #[test]
    fn the_values_and_the_chip_stand_clear_of_the_lines_the_hatch_and_the_arrows() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let words = sweep_words(&kit, stage)?;
            let across = axis_words(&kit, stage, PLANE, REACH)?;
            let (mix, other, negative) = (
                Room::of(stage, words.mix),
                Room::of(stage, words.other),
                Room::of(stage, words.negative),
            );
            let food = Room::of(stage, across[0]);
            let plane = Room::spanning(PLANE.p(-REACH.0, -REACH.1), PLANE.p(REACH.0, REACH.1));
            let hatch = Room::spanning(PLANE.p(0.0, 0.0), PLANE.p(REACH.0, -REACH.1));
            // a gold line is a 26 pt glow round a 3 pt core
            let stroke = 0.12;
            let segment = |dir: [f64; 2]| {
                let (a, b) = band_ends(dir, REACH.0, REACH.1);
                (PLANE.p(a[0], a[1]), PLANE.p(b[0], b[1]))
            };
            let mix_deg = numbers("K-E2.mix_direction_deg")?[0];
            let other_dir = numbers("K-E2.other")?;
            let (m0, m1) = segment(unit_at(mix_deg));
            let (o0, o1) = segment([other_dir[0], other_dir[1]]);
            assert!(
                !mix.grown(stroke).crosses(m0, m1),
                "0.7 stands on the gold line: {mix:?}"
            );
            assert!(!mix.overlaps(&plane), "0.7 is inside the plane: {mix:?}");
            assert!(
                !other.grown(stroke).crosses(o0, o1),
                "0.2 stands on the dashed line: {other:?}"
            );
            assert!(!other.overlaps(&hatch), "0.2 is inside the hatch: {other:?}");
            assert!(!other.overlaps(&plane), "0.2 is inside the plane: {other:?}");
            // the chip is outside the plane, so no arrow of the sweep (at
            // most SWEEP_LENGTH, inside the reach) can run under it, and
            // it names the quarter it stands beside
            assert!(SWEEP_LENGTH < REACH.0 && SWEEP_LENGTH < REACH.1);
            assert!(!negative.overlaps(&plane), "the chip is inside the plane: {negative:?}");
            assert!(negative.x0 - plane.x1 >= CHIP_GAP - 1e-9, "{negative:?}");
            assert!(
                negative.y0 >= hatch.y0 && negative.y1 <= hatch.y1,
                "the chip is not beside the quarter: {negative:?} against {hatch:?}"
            );
            for (name, other_box) in [("0.2", other), ("0.7", mix), ("food", food)] {
                assert!(!negative.overlaps(&other_box), "the chip is over {name}: {negative:?}");
            }
            Ok(())
        });
    }

    /// 12_5: the citation's lower-third is two lines, the second not a
    /// remnant of the first ("1998)" stood alone on a third line).
    #[test]
    fn the_citation_wraps_into_two_even_lines() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let before = kit.registered().len();
            let _band = citation(&kit, stage)?;
            let made: Vec<String> = kit.registered()[before..].iter().map(|r| r.source.clone()).collect();
            // what it settled on is the run of lines at the end of those it
            // laid out whose words, in order, are the whole text
            let text = citation_text(fact("K-S.brin_page_year")?);
            let mut lines: Vec<String> = Vec::new();
            for source in made.iter().rev() {
                lines.insert(0, source.clone());
                if lines.join(" ") == text {
                    break;
                }
            }
            assert_eq!(
                lines.join(" "),
                text,
                "the lines of the band were not found in {made:?}"
            );
            assert_eq!(lines.len(), 2, "{lines:?}");
            let (a, b) = (lines[0].chars().count(), lines[1].chars().count());
            assert!(3 * b >= a, "the second line is a remnant: {lines:?}");
            Ok(())
        });
    }

    /// 12_5 (and 14_5): the degree sign stands against the last digit of the
    /// readout, however many digits it has. It stood 0.5 units off ("25   °").
    #[test]
    fn the_degree_sign_hugs_the_last_digit_at_any_angle() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let table = table_of("K-E1.table")?;
            let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
            let test = ArrowTest::build(stage, PLANE, table, u, REACH)?;
            let readout = angle_readout(&kit, stage, &test, READOUT_AT)?;
            let sign = degree_sign(&kit, stage, READOUT_AT)?;
            test.start(stage)?;
            stage.set_tracker_value(test.polar, 1.0)?;
            stage.set_tracker_value(test.radius, SWEEP_LENGTH)?;
            stage.add_to_scene(readout)?;
            stage.add_to_scene(sign)?;
            let sign_box = Room::of(stage, sign);
            assert!(
                (sign_box.x0 - READOUT_AT[0] - DEGREE_GAP).abs() < 1e-6,
                "the sign's left edge is {} not {}",
                sign_box.x0,
                READOUT_AT[0] + DEGREE_GAP
            );
            // along the even mix nothing turns ("0"), along the first axis
            // the push is turned by about thirty degrees: one digit, then two
            let mix_deg = numbers("K-E2.mix_direction_deg")?[0];
            let mut widths = Vec::new();
            for deg in [mix_deg, 0.0] {
                stage.set_tracker_value(test.degrees, deg)?;
                stage.wait(0.2)?;
                let n = Room::of(stage, readout);
                assert!(
                    (n.x1 - READOUT_AT[0]).abs() < 1e-6,
                    "at {deg} degrees the number ends at {} not {}",
                    n.x1,
                    READOUT_AT[0]
                );
                widths.push(n.x1 - n.x0);
            }
            assert!(
                widths[1] > widths[0] + 0.1,
                "the readout never changed width: {widths:?}"
            );
            Ok(())
        });
    }

    /// 12_1: the pictures are gone before the numbers start to fly, so no
    /// number crosses a house, a name or an arrow (a frame had "0.2" over
    /// "workshops: tools"); and the numbers do land on their cells.
    #[test]
    fn the_numbers_fly_only_after_the_pictures_are_gone() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let icons = Icons::build(&kit, stage, &table_of("K-E1.table")?)?;
            let mut all = vec![icons.field, icons.workshop, icons.tag];
            all.extend(icons.names);
            all.extend(icons.arrows);
            all.extend(icons.numbers);
            for &m in &all {
                stage.add_to_scene(m)?;
            }
            let start: Vec<Vec3> = icons.numbers.iter().map(|&n| stage.get_center(n)).collect();
            let widths: Vec<f64> = icons.numbers.iter().map(|&n| stage.get_width(n)).collect();
            let (clock, moved_at) = (Rc::new(Cell::new(0.0)), Rc::new(Cell::new(f64::NAN)));
            let (c, m, numbers) = (Rc::clone(&clock), Rc::clone(&moved_at), icons.numbers);
            let probe = stage.arena_mut().add(VMobject::new());
            stage.add_dt_updater(
                probe,
                move |s, _, dt| {
                    c.set(c.get() + dt);
                    let moving = numbers.iter().zip(&start).any(|(&n, p)| {
                        let at = s.get_center(n);
                        (at[0] - p[0]).abs() + (at[1] - p[1]).abs() > 1e-6
                    });
                    if moving && m.get().is_nan() {
                        m.set(c.get());
                    }
                },
                false,
            )?;
            stage.add_to_scene(probe)?;
            let cells = [v(3.8, 1.1), v(3.8, 0.0), v(5.0, 1.1), v(5.0, 0.0)];
            fold_into_table(stage, &icons, cells)?;
            assert!(
                moved_at.get() >= ICONS_GO - 1e-6,
                "a number moved at {} s, while the pictures take {ICONS_GO} s to go",
                moved_at.get()
            );
            for ((&n, cell), w0) in icons.numbers.iter().zip(cells).zip(widths) {
                let at = stage.get_center(n);
                assert!(
                    (at[0] - cell[0]).abs() < 1e-6 && (at[1] - cell[1]).abs() < 1e-6,
                    "{at:?} {cell:?}"
                );
                // and as big as the table's own entries
                let grown = stage.get_width(n) / w0;
                assert!((grown - TABLE_SIZE / NUMBER_SIZE).abs() < 1e-6, "{grown}");
            }
            Ok(())
        });
    }
}
