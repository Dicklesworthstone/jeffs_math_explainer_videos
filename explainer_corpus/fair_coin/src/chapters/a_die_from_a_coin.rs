//! Chapter 07, a die from a coin. It opens on the row for four tosses just
//! as chapter 06 leaves it (`draw::FourRow`, in its one place) and everything
//! happens around it: the middle group's six patterns, all equally likely,
//! rise out of their box (07_1); each picks a die face, and any other count
//! of heads means four fresh tosses (07_2); the seeded run of 24,000 tosses
//! lands above the row as 6,000 tries, every count a tally of what has
//! landed, the six faces inside a fair die's band (07_3); the row's bars
//! show what a roll costs with a coin this unfair (07_4); and the group sizes
//! under the row grow into Pascal's triangle, every entry a fair choice
//! among that many (07_5).

use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::props::RowStrips;
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, children, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    FOUR_ROW_BOX_H, FOUR_ROW_SIZE_Y, FourRow, ROW_BOX_W, RowBars, caption, die_face, disc_row, fade_in_all,
    fade_out_all, fail, flat, gold, group, line_style, pattern_tile, pause, rgb, row_x, say, thrown, thrown_lit,
};
use crate::facts::coin_die_run;
use crate::model::{Face, Q, bent, count_chance, die_from_four_tosses, pascal_row, pattern_chance, two_sigma_band};

pub struct ADieFromACoin {
    pub kit: Kit,
}

/// Tosses in a try: the row of Pascal's triangle the die is made from.
const TOSSES: u32 = 4;
/// Heads a try must show: the middle group of that row.
const HEADS: u32 = 2;
/// Tries of the run played at reading speed before the whole run lands.
const SHOWN: usize = 8;
/// The first and last rows of the triangle that closes the chapter.
const TRIANGLE: (u32, u32) = (2, 4);
/// The trick's own group: the row for two tosses, one head.
const TRICK: (u32, u32) = (2, 1);

// the chapter's fixed words (numbers are built from facts)
const HEADS_WORD: &str = "heads";
const TAILS_WORD: &str = "tails";
const AND: &str = "and";
const PATTERNS: &str = "patterns";
const FRESH: &str = "any other count: toss four fresh ones";
const THE_FIRST: &str = "the first";
const TRIES: &str = "tries,";
const TOSSES_EACH: &str = "tosses each";
const TOSSES_WORD: &str = "tosses";
const THROWN_AWAY: &str = "a try thrown away";
const A_ROLL: &str = "a roll";
const ROLLS: &str = "rolls";
const EACH_FACE: &str = "each face:";
const TO: &str = "to";
const NINETEEN_IN_TWENTY: &str = "about 19 times in 20 (model)";
const OF_TRIES: &str = "of tries";
const DIVIDED: &str = "\u{f7}";
const ON_AVERAGE: &str = "on average (model)";
const THIS_RUN: &str = "this run:";
const SAME_COUNTS: &str = "same counts, same chance";
const FAIR_CHOICE: &str = "each entry: a fair choice among that many";
const THE_TRICK: &str = "the trick";
const A_DIE: &str = "a die";

/// The middle group of the row: how many tails its patterns have.
const MID: usize = (TOSSES - HEADS) as usize;

// 07_1 and 07_2: the six patterns lifted out above the row, their dice under them
const LIFT: f64 = 1.8;
const LIFT_Y: f64 = 2.85;
const LIFT_PITCH: f64 = 2.2;
const HEADING_Y: f64 = 3.55;
const PERCENT_Y: f64 = 2.35;
const DICE_Y: f64 = 1.35;
const DIE: f64 = 0.72;
/// A tile's discs: their pitch at the row's size (`draw::pattern_tile`).
const TILE_STEP: f64 = 0.23;

// 07_3: the first eight tries across the top, each with its mark under it
// (the legend on their heading's line), the grid of every try under them on
// the left, the six faces' bars on the right; all above the row
const TRIES_HEAD_Y: f64 = 3.66;
const TRY_Y: f64 = 3.22;
const MARK_Y: f64 = 2.74;
const TRY_PITCH: f64 = 1.6;
const TRY_R: f64 = 0.14;
const TRY_STEP: f64 = 0.3;
const GRID_LEFT: f64 = -6.8;
const GRID_TOP: f64 = 2.05;
const GRID_COLS: usize = 120;
const GRID_H: f64 = 2.45;
const LEGEND_X: [f64; 2] = [0.9, 4.3];
const BAND_WORDS_Y: f64 = 2.22;
const FACE_X: f64 = 1.2;
const FACE: f64 = 0.34;
const BAR_X0: f64 = 1.55;
const BAR_THICK: f64 = 0.24;
/// How long the bar of the fair die's band's upper count would be.
const BAR_LEN: f64 = 4.1;
const TALLY_X: f64 = 6.7;
const BAR_Y0: f64 = 1.8;
const BAR_PITCH: f64 = 0.38;
/// How long the whole run takes to land.
const LAND_S: f64 = 3.2;

// 07_4: the cost, over the row's bars
const SUM_Y: f64 = 3.45;
const RUN_Y: f64 = 3.0;

// 07_5: the triangle grows up from the row's group sizes
const TRI_STEP: f64 = 1.15;
const SAME_Y: f64 = 1.75;
const CHOICE_Y: f64 = 1.05;

/// A rounded outline (no fill) centred at `at`.
fn outline(at: Vec3, w: f64, h: f64, corner: f64, color: Srgb, width: f64) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(corner)
        .style(line_style(color, width))
        .build()?
        .moved_to(at))
}

/// A square cell of colour centred at `at`.
fn cell(at: Vec3, side: f64, color: Srgb) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(side)
        .height(side)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(at))
}

/// A pattern as a strip of plain discs centred at `at`: one group, detached.
fn strip(stage: &mut Stage<'_>, pattern: &[Face], at: Vec3, radius: f64, step: f64) -> fmn::Result<Mob> {
    let half = (pattern.len() as f64 - 1.0) / 2.0;
    let discs = disc_row(stage, pattern, v(at[0] - half * step, at[1]), radius, step);
    group(stage, &discs)
}

/// Grow or shrink `mob` by `scale` on its way to `to`.
fn glide(mob: Mob, scale: f64, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(scale)?
        .move_to(to, ORIGIN)?)
}

/// Repaint a mobject's whole family.
fn refill(mob: Mob, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_fill(Some(color), Some(1.0))?)
}

/// One after another, each of `mobs` swells by `scale` and settles back;
/// the whole takes `rt` seconds. (A die face has two fills, so a colour
/// flash would flood its pips.)
fn pulse_each(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64, rt: f64) -> fmn::Result<()> {
    let mut pulses: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                rate_func: Some(fmn::core::rate::there_and_back),
                ..AnimateArgs::default()
            })?
            .scale(scale)?;
        pulses.push(stage.prepare(a)?);
    }
    let pulses = AnimationGroup::with_lag_ratio(stage.arena_mut(), pulses, 0.5)?.with_run_time(rt);
    stage.play(pulses)?;
    Ok(())
}

/// Where lifted pattern `i` of `n` sits.
fn lift_at(i: usize, n: usize) -> Vec3 {
    v((i as f64 - (n as f64 - 1.0) / 2.0) * LIFT_PITCH, LIFT_Y)
}

/// Where try `i` of the first eight sits, and its mark under it.
fn try_at(i: usize) -> (Vec3, Vec3) {
    let x = TRY_PITCH * (i as f64 - (SHOWN as f64 - 1.0) / 2.0);
    (v(x, TRY_Y), v(x, MARK_Y))
}

/// The height of face `f`'s bar (0 for a one).
fn bar_y(f: usize) -> f64 {
    BAR_Y0 - BAR_PITCH * f as f64
}

/// A horizontal bar from `left` (its left end, at its middle height).
fn hbar(left: Vec3, len: f64, color: Srgb) -> VMobject {
    let (x0, x1) = (left[0], left[0] + len.max(0.004));
    let (y0, y1) = (left[1] - BAR_THICK / 2.0, left[1] + BAR_THICK / 2.0);
    Polygon::new([v(x0, y0), v(x1, y0), v(x1, y1), v(x0, y1)])
        .style(flat(color, 1.0))
        .build()
}

/// The entry `k` (from the left) of the triangle's row for `n` tosses.
fn tri_at(n: u32, k: usize) -> Vec3 {
    v(
        row_x(n as usize, k),
        FOUR_ROW_SIZE_Y + TRI_STEP * f64::from(TRIANGLE.1 - n),
    )
}

/// The grid of every try of the run, in reading order, as two layers of
/// row strips (one mobject per row each): under, a gold cell behind every
/// roll; over, each cell's core (ink for a roll, grey for a try thrown away)
/// with the gaps between them, gold beside a roll. So a roll is an ink cell
/// with a gold edge. One tracker says how many tries have landed: a try not
/// yet landed is the background colour.
struct TryGrid {
    /// Each row strip's group, to bring in and fade.
    groups: Vec<Mob>,
    /// Carries the updater that recolours the cells.
    driver: Mob,
}

impl TryGrid {
    fn build(stage: &mut Stage<'_>, cells: &[Option<usize>], landed: Mob) -> fmn::Result<Self> {
        let rows = cells.len().div_ceil(GRID_COLS);
        let pitch = GRID_H / rows as f64;
        let dark = hex(BACKGROUND);
        // the cell core is four fifths of the pitch; the fifth part between
        // two cores is the gap
        let colors = move |cells: &[Option<usize>], n: usize| -> (Vec<Vec<Srgb>>, Vec<Vec<Srgb>>) {
            let roll = |i: usize| i < n && cells.get(i).copied().flatten().is_some();
            let mut under = Vec::with_capacity(rows);
            let mut over = Vec::with_capacity(rows);
            for r in 0..rows {
                let mut u = Vec::with_capacity(GRID_COLS);
                let mut o = Vec::with_capacity(5 * GRID_COLS);
                for c in 0..GRID_COLS {
                    let i = r * GRID_COLS + c;
                    u.push(if roll(i) { gold() } else { dark });
                    let core = if i >= n || i >= cells.len() {
                        dark
                    } else if roll(i) {
                        Palette::ink()
                    } else {
                        thrown()
                    };
                    let gap = if roll(i) || (c + 1 < GRID_COLS && roll(i + 1)) {
                        gold()
                    } else {
                        dark
                    };
                    o.extend([core, core, core, core, gap]);
                }
                under.push(u);
                over.push(o);
            }
            (under, over)
        };
        let n0 = SHOWN.min(cells.len());
        let (under_c, over_c) = colors(cells, n0);
        let under = RowStrips::build(stage, v(GRID_LEFT, GRID_TOP), (pitch, pitch), &under_c)?;
        let mut overs = Vec::with_capacity(rows);
        let mut groups = vec![under.group];
        for (r, row) in over_c.iter().enumerate() {
            let y = GRID_TOP - pitch * (r as f64 + 0.5);
            let thick = 0.8 * pitch;
            let strip = RowStrips::build(
                stage,
                v(GRID_LEFT + pitch / 10.0, y + thick / 2.0),
                (pitch / 5.0, thick),
                std::slice::from_ref(row),
            )?;
            groups.push(strip.group);
            overs.push(strip);
        }
        let driver = stage.arena_mut().add(VMobject::new());
        let cells = cells.to_vec();
        let mut last = Some(n0);
        stage.add_updater(
            driver,
            move |s, _| {
                let n = (s.tracker_value(landed).unwrap_or(0.0).floor().max(0.0) as usize).min(cells.len());
                if last == Some(n) {
                    return;
                }
                let (u, o) = colors(&cells, n);
                under.recolor(s, &u);
                for (strip, row) in overs.iter().zip(&o) {
                    strip.recolor(s, std::slice::from_ref(row));
                }
                last = Some(n);
            },
            false,
        )?;
        Ok(Self { groups, driver })
    }
}

impl SceneConstruct for ADieFromACoin {
    fn name(&self) -> &str {
        "07_a_die_from_a_coin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent();
        let dark = hex(BACKGROUND);

        // the row for four tosses, on stage at frame 0 exactly as 06 leaves it
        let row = FourRow::build(&kit, stage)?;
        for m in row.mobs() {
            stage.add_to_scene(m)?;
        }
        let six = row.patterns[MID].clone();
        let faces = six.len();
        if faces as u128 != pascal_row(TOSSES)[HEADS as usize] {
            return Err(fail("the middle group is not its entry of Pascal's row".into()));
        }

        // 07_1: the middle group's six patterns rise out of their box
        let line = say(&kit, stage, "07_1")?;
        let mid_at = FourRow::center(MID);
        let ring = outline(mid_at, ROW_BOX_W + 0.14, FOUR_ROW_BOX_H + 0.14, 0.16, gold(), 3.5)?;
        let ring = stage.arena_mut().add(ring);
        play!(stage; show_creation(ring).rt(0.7));
        line.wait_for(stage, "Six patterns")?;
        let mut lifted = Vec::new();
        let mut rises: Vec<Box<dyn Animation>> = Vec::new();
        for (j, pattern) in six.iter().enumerate() {
            // a copy over the tile, which stays in its box
            let from = stage.get_center(row.tiles[MID][j]);
            let copy = pattern_tile(stage, pattern, from)?;
            stage.add_to_scene(copy)?;
            rises.push(stage.prepare(glide(copy, LIFT, lift_at(j, faces), 1.0)?)?);
            lifted.push(copy);
        }
        let rise = AnimationGroup::with_lag_ratio(stage.arena_mut(), rises, 0.2)?.with_run_time(1.6);
        stage.play(rise)?;
        let mut letters = Vec::new();
        for (i, pattern) in six.iter().enumerate() {
            let c = lift_at(i, faces);
            let half = (pattern.len() as f64 - 1.0) / 2.0;
            for (j, f) in pattern.iter().enumerate() {
                let at = v(c[0] + (j as f64 - half) * TILE_STEP * LIFT, c[1]);
                letters.push(kit.text(stage, &f.letter().to_string(), 20.0, dark, at)?);
            }
        }
        let heading = format!(
            "{HEADS} {HEADS_WORD} {AND} {} {TAILS_WORD}: {faces} {PATTERNS}",
            TOSSES - HEADS
        );
        let heading = kit.text(stage, &heading, 26.0, Palette::dim(), v(0.0, HEADING_Y))?;
        let mut named = letters.clone();
        named.push(heading);
        fade_in_all(stage, &named, 0.5)?;
        line.wait_for(stage, "all equally likely")?;
        let each = format!("{}%", pattern_chance(p, TOSSES, HEADS).percent(2));
        let mut percents = Vec::new();
        for i in 0..faces {
            let at = v(lift_at(i, faces)[0], PERCENT_Y);
            percents.push(kit.text(stage, &each, 22.0, Palette::dim(), at)?);
        }
        let mut shows: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &percents {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            shows.push(stage.prepare(a)?);
        }
        let shows = AnimationGroup::with_lag_ratio(stage.arena_mut(), shows, 0.3)?.with_run_time(1.2);
        stage.play(shows)?;
        // six of them, the same counts, the same chance: one gold frame
        line.wait_for(stage, "What else has six")?;
        let span = LIFT_PITCH * (faces as f64 - 1.0) + 2.15;
        let frame_at = v(0.0, (LIFT_Y + PERCENT_Y) / 2.0);
        let six_frame = outline(frame_at, span, 1.25, 0.16, gold(), 3.0)?;
        let six_frame = stage.arena_mut().add(six_frame);
        play!(stage; show_creation(six_frame).rt(0.9));
        let q = predict_pause(&kit, stage, v(0.0, 0.6))?;

        // 07_2: a die: each pattern picks a face; any other count, toss four fresh ones
        let line = say(&kit, stage, "07_2")?;
        let mut asked = percents.clone();
        asked.extend([q, six_frame]);
        fade_out_all(stage, &asked, 0.3)?;
        // one at a time, so a copy on its way never hides its neighbours' letters
        let mut dice = Vec::new();
        for (i, &pattern) in lifted.iter().enumerate() {
            let at = v(lift_at(i, faces)[0], DICE_Y);
            let die = die_face(stage, i + 1, at, DIE, Palette::ink(), dark)?;
            play!(stage; fmn::animation::transform_from_copy(stage.arena_mut(), pattern, die)?.rt(0.3));
            dice.push(die);
        }
        line.wait_for(stage, "If exactly two")?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), ring, 1.06, Some(rgb(gold())))?.rt(0.9));
        line.wait_for(stage, "the pattern picks the face")?;
        pulse_each(stage, &dice, 1.2, 1.5)?;
        line.wait_for(stage, "If not")?;
        // the other four groups' patterns turn grey, in their boxes
        let mut greys: Vec<Box<dyn Animation>> = Vec::new();
        for k in (0..row.tiles.len()).filter(|&k| k != MID) {
            for &tile in &row.tiles[k] {
                for disc in children(stage, tile).into_iter().skip(1) {
                    greys.push(stage.prepare(refill(disc, thrown_lit(), 0.7)?)?);
                }
            }
        }
        let fresh = caption(&kit, stage, FRESH)?;
        let loop_x = -stage.get_width(fresh) / 2.0 - 0.8;
        let again = curved_arrow(
            v(loop_x - 0.2, super::draw::CAPTION_Y - 0.12),
            v(loop_x + 0.2, super::draw::CAPTION_Y - 0.12),
            -1.5 * PI,
            line_style(Palette::dim(), 3.5),
        )?;
        let again = stage.arena_mut().add(again);
        for m in [again, fresh] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            greys.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(greys)?;

        // 07_3: the run, landing above the row: every try a cell, every
        // count a tally of the tries that have landed
        let line = say(&kit, stage, "07_3")?;
        let run = coin_die_run();
        let block = TOSSES as usize;
        let cells: Vec<Option<usize>> = run.chunks_exact(block).map(die_from_four_tosses).collect();
        let made: Vec<usize> = cells.iter().flatten().copied().collect();
        // the tally of each face after the first n tries, for every n
        let mut sofar = vec![[0usize; 6]];
        for c in &cells {
            let mut next = *sofar.last().unwrap_or(&[0; 6]);
            if let Some(face) = c {
                next[*face] += 1;
            }
            sofar.push(next);
        }
        let sofar = Rc::new(sofar);
        let finals = sofar.last().copied().unwrap_or([0; 6]);
        // the lifted patterns make room; the six faces go to the heads of six bars
        let mut clears: Vec<Box<dyn Animation>> = Vec::new();
        let gone: Vec<Mob> = lifted
            .iter()
            .chain(&letters)
            .chain([&heading, &again])
            .copied()
            .collect();
        for &m in &gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            clears.push(stage.prepare(a.rt(0.4))?);
        }
        for (i, &die) in dice.iter().enumerate() {
            clears.push(stage.prepare(glide(die, FACE / DIE, v(FACE_X, bar_y(i)), 0.7)?)?);
        }
        let counted = caption(
            &kit,
            stage,
            &format!("{} {TOSSES_WORD}, {} {ROLLS}", thousands(run.len() as u64), made.len()),
        )?;
        let a = fade_out(stage.arena_mut(), fresh, ORIGIN, 1.0)?;
        clears.push(stage.prepare(a.rt(0.4))?);
        let a = fade_in(stage.arena_mut(), counted, ORIGIN, 1.0)?;
        clears.push(stage.prepare(a.rt(0.7))?);
        stage.play_prepared(clears)?;
        // the first eight tries at reading speed: a grey cell for a try
        // thrown away, a die face for a roll
        let first = format!("{THE_FIRST} {SHOWN} {TRIES} {block} {TOSSES_EACH}");
        let first = kit.text_left(stage, &first, 22.0, Palette::dim(), v(GRID_LEFT, TRIES_HEAD_Y))?;
        let mut early = vec![first];
        let mut plays: Vec<Box<dyn Animation>> = Vec::new();
        let a = fade_in(stage.arena_mut(), first, ORIGIN, 1.0)?;
        plays.push(stage.prepare(a)?);
        for (i, tosses) in run.chunks_exact(block).take(SHOWN).enumerate() {
            let (at, mark_at) = try_at(i);
            let shown = strip(stage, tosses, at, TRY_R, TRY_STEP)?;
            let mark = match cells[i] {
                Some(face) => {
                    let die = die_face(stage, face + 1, mark_at, 0.38, Palette::ink(), dark)?;
                    let edge = outline(mark_at, 0.47, 0.47, 0.1, gold(), 2.5)?;
                    let edge = stage.arena_mut().add(edge);
                    group(stage, &[die, edge])?
                }
                None => {
                    let c = cell(mark_at, 0.28, thrown())?;
                    stage.arena_mut().add(c)
                }
            };
            let a = fade_in(stage.arena_mut(), shown, ORIGIN, 1.0)?;
            plays.push(stage.prepare(a)?);
            let a = fade_in(stage.arena_mut(), mark, v(0.0, -0.2), 1.0)?;
            plays.push(stage.prepare(a)?);
            early.extend([shown, mark]);
        }
        let plays = AnimationGroup::with_lag_ratio(stage.arena_mut(), plays, 0.5)?.with_run_time(2.6);
        stage.play(plays)?;
        // then every try of the run lands, in reading order
        line.wait_for(stage, "made nine hundred")?;
        let landed = stage.add_value_tracker(SHOWN as f64);
        let grid = TryGrid::build(stage, &cells, landed)?;
        let (lo, hi) = two_sigma_band(made.len() as u32, Q::new(1, faces as i128));
        let unit = BAR_LEN / f64::from(hi);
        // the six bars and their tallies, read off the same tracker
        let bars: Vec<Mob> = (0..faces)
            .map(|f| {
                let start = sofar[SHOWN][f] as f64 * unit;
                stage.arena_mut().add(hbar(v(BAR_X0, bar_y(f)), start, Palette::dim()))
            })
            .collect();
        let bar_driver = stage.arena_mut().add(VMobject::new());
        {
            let (bars, sofar) = (bars.clone(), Rc::clone(&sofar));
            let mut last = None;
            stage.add_updater(
                bar_driver,
                move |s, _| {
                    let n = (s.tracker_value(landed).unwrap_or(0.0).floor().max(0.0) as usize).min(sofar.len() - 1);
                    if last == Some(n) {
                        return;
                    }
                    for (f, &m) in bars.iter().enumerate() {
                        let len = sofar[n][f] as f64 * unit;
                        let _ = s.set_points(m, hbar(v(BAR_X0, bar_y(f)), len, Palette::dim()).points());
                    }
                    last = Some(n);
                },
                false,
            )?;
        }
        let (grey_at, roll_at) = (v(LEGEND_X[0], TRIES_HEAD_Y), v(LEGEND_X[1], TRIES_HEAD_Y));
        let legend = [
            stage.arena_mut().add(cell(grey_at, 0.22, thrown())?),
            kit.text_left(
                stage,
                THROWN_AWAY,
                22.0,
                Palette::dim(),
                v(grey_at[0] + 0.28, grey_at[1]),
            )?,
            stage.arena_mut().add(cell(roll_at, 0.22, Palette::ink())?),
            stage.arena_mut().add(outline(roll_at, 0.28, 0.28, 0.0, gold(), 2.5)?),
            kit.text_left(stage, A_ROLL, 22.0, Palette::dim(), v(roll_at[0] + 0.28, roll_at[1]))?,
        ];
        let mut ins = grid.groups.clone();
        ins.extend([grid.driver, bar_driver]);
        ins.extend(&bars);
        ins.extend(legend);
        fade_in_all(stage, &ins, 0.4)?;
        let mut tallies = Vec::new();
        for f in 0..faces {
            let sofar = Rc::clone(&sofar);
            let tally = move |s: &fmn::mobject::Stage| {
                let n = (s.tracker_value(landed).unwrap_or(0.0).floor().max(0.0) as usize).min(sofar.len() - 1);
                sofar[n][f] as f64
            };
            let at = v(TALLY_X, bar_y(f));
            tallies.push(kit.live_value(stage, tally, 0, false, 22.0, Palette::ink(), at, RIGHT)?);
        }
        play!(stage; tween(landed, cells.len() as f64, LAND_S)?);
        if sofar[cells.len()] != finals || finals.iter().sum::<usize>() != made.len() {
            return Err(fail("the tallies do not end at the run's counts".into()));
        }
        line.wait_for(stage, "Each face came up")?;
        pulse_each(stage, &dice, 1.25, 1.3)?;
        // where a fair die's count of one face lands about 19 times in 20
        line.wait_for(stage, "about as often")?;
        let (x_lo, x_hi) = (BAR_X0 + unit * f64::from(lo), BAR_X0 + unit * f64::from(hi));
        let (y0, y1) = (bar_y(faces - 1) - 0.21, bar_y(0) + 0.21);
        let tint = Polygon::new([v(x_lo, y0), v(x_hi, y0), v(x_hi, y1), v(x_lo, y1)])
            .style(flat(Palette::ink(), 0.05))
            .build();
        let mut band = vec![stage.arena_mut().add(tint)];
        for x in [x_lo, x_hi] {
            let rule = polyline(vec![v(x, y0), v(x, y1)])
                .style(super::draw::faded_line(Palette::ink(), 1.5, 0.75))
                .build();
            band.push(stage.arena_mut().add(rule));
        }
        let band_words = format!("{EACH_FACE} {lo} {TO} {hi}, {NINETEEN_IN_TWENTY}");
        let band_words = kit.text(stage, &band_words, 22.0, Palette::dim(), v(0.0, BAND_WORDS_Y))?;
        let w = stage.get_width(band_words);
        stage.move_to(band_words, v(TALLY_X - w / 2.0, BAND_WORDS_Y), ORIGIN);
        band.push(band_words);
        fade_in_all(stage, &band, 0.7)?;
        stage.scene_mut().bring_to_front(&bars)?;

        // 07_4: it isn't cheap: the row's bars, the cost from chapter 04's sum
        let line = say(&kit, stage, "07_4")?;
        let mut run_panel = early.clone();
        run_panel.extend(&ins);
        run_panel.extend(&tallies);
        run_panel.extend(&dice);
        run_panel.extend(&band);
        run_panel.push(counted);
        fade_out_all(stage, &run_panel, 0.5)?;
        line.wait_for(stage, "A coin this unfair")?;
        let chance = stage.add_value_tracker(p.f64());
        let row_bars = row.bars(&kit, stage, chance)?;
        stage.add_to_scene(row_bars.anchor)?;
        let mut grows: Vec<Box<dyn Animation>> = Vec::new();
        for &bar in &row_bars.bars {
            let a = fmn::animation::grow_from_edge(stage.arena_mut(), bar, DOWN, None)?;
            grows.push(stage.prepare(a.rt(0.9))?);
        }
        stage.play_prepared(grows)?;
        // the bars' chances, the middle one as a share of tries
        let works = count_chance(p, TOSSES, HEADS);
        for (k, &label) in row_bars.labels.iter().enumerate() {
            if k != MID {
                stage.add_to_scene(label)?;
            }
        }
        let tries = format!("{}% {OF_TRIES}", works.percent(2));
        let tries = kit.text(stage, &tries, 22.0, Palette::ink(), v(0.0, 0.0))?;
        stage.move_to(tries, v(row_x(4, MID), RowBars::top(MID, p.f64()) + 0.1), DOWN);
        play!(stage;
            refill(row_bars.bars[MID], Palette::ink(), 0.5)?,
            fade_in(stage.arena_mut(), tries, ORIGIN, 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "A roll costs")?;
        let tosses = Q::int(i128::from(TOSSES));
        let sum_left = format!("{TOSSES} {TOSSES_WORD} {DIVIDED} {} =", works.decimal(4));
        let sum_right = format!("{}, {ON_AVERAGE}", (tosses / works).decimal(2));
        let sum_left = kit.text(stage, &sum_left, 30.0, Palette::ink(), v(0.0, SUM_Y))?;
        let sum_right = kit.text(stage, &sum_right, 30.0, Palette::ink(), v(0.0, SUM_Y))?;
        let (wl, wr) = (stage.get_width(sum_left), stage.get_width(sum_right));
        let x0 = -(wl + 0.15 + wr) / 2.0;
        stage.move_to(sum_left, v(x0, SUM_Y), LEFT);
        stage.move_to(sum_right, v(x0 + wl + 0.15, SUM_Y), LEFT);
        let run_sum = format!(
            "{THIS_RUN} {} {TOSSES_WORD} {DIVIDED} {} {ROLLS} = {}",
            thousands(run.len() as u64),
            made.len(),
            Q::new(run.len() as i128, made.len() as i128).decimal(1)
        );
        let run_sum = kit.text(stage, &run_sum, 24.0, Palette::dim(), v(0.0, RUN_Y))?;
        fade_in_all(stage, &[sum_left], 0.4)?;
        // the result snaps in
        pause(stage, 0.3)?;
        stage.add_to_scene(sum_right)?;
        pause(stage, 0.8)?;
        fade_in_all(stage, &[run_sum], 0.4)?;

        // 07_5: the group sizes under the row grow into Pascal's triangle
        let line = say(&kit, stage, "07_5")?;
        let mut leaving: Vec<Mob> = row.boxes.to_vec();
        leaving.extend(row.all_tiles());
        leaving.extend(row.counts);
        leaving.extend([ring, tries, sum_left, sum_right, run_sum]);
        leaving.extend(row_bars.mobs());
        leaving.extend(
            row_bars
                .labels
                .iter()
                .enumerate()
                .filter(|&(k, _)| k != MID)
                .map(|(_, &l)| l),
        );
        fade_out_all(stage, &leaving, 0.5)?;
        let mut entries: Vec<(u32, usize, Mob)> = Vec::new();
        let mut arrives = Vec::new();
        for n in TRIANGLE.0..=TRIANGLE.1 {
            for (k, size) in pascal_row(n).iter().rev().enumerate() {
                let m = if n == TOSSES {
                    row.sizes[k]
                } else {
                    let m = kit.text(stage, &size.to_string(), 40.0, Palette::ink(), tri_at(n, k))?;
                    arrives.push(m);
                    m
                };
                entries.push((n, k, m));
            }
            if n != TOSSES {
                let name = format!("{n} {TOSSES_WORD}");
                let name = kit.text(stage, &name, 22.0, Palette::dim(), tri_at(n, 0))?;
                stage.move_to(name, v(tri_at(n, 0)[0] - 0.45, tri_at(n, 0)[1]), RIGHT);
                arrives.push(name);
            }
        }
        fade_in_all(stage, &arrives, 0.6)?;
        line.wait_for(stage, "with the same counts")?;
        let same = kit.text(stage, SAME_COUNTS, 34.0, gold(), v(0.0, SAME_Y))?;
        let choice = kit.text(stage, FAIR_CHOICE, 28.0, Palette::ink(), v(0.0, CHOICE_Y))?;
        fade_in_all(stage, &[same, choice], 0.5)?;
        // each entry glows in turn; the two this video has used are named
        let mut glows: Vec<Box<dyn Animation>> = Vec::new();
        for &(n, k, m) in &entries {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.4, Some(rgb(gold())))?;
            glows.push(stage.prepare(a)?);
            // (tosses, heads): entry `tosses - heads` of its row, as in the boxes
            for ((tosses, heads), name) in [(TRICK, THE_TRICK), ((TOSSES, HEADS), A_DIE)] {
                if (n, k) == (tosses, (tosses - heads) as usize) {
                    let at = v(tri_at(n, k)[0], tri_at(n, k)[1] - 0.47);
                    let tag = kit.text(stage, name, 22.0, Palette::dim(), at)?;
                    let a = fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?;
                    glows.push(stage.prepare(a)?);
                }
            }
        }
        let glows = AnimationGroup::with_lag_ratio(stage.arena_mut(), glows, 0.6)?.with_run_time(4.0);
        stage.play(glows)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
