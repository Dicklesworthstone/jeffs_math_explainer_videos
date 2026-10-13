//! Chapter 07, a die from a coin: the middle group of the row for four
//! tosses holds six patterns, all equally likely (07_1); so each pattern
//! picks a die face, and any other count of heads means four fresh tosses
//! (07_2); the seeded run of 24,000 tosses as a grid of 6,000 blocks and six
//! bars inside a fair die's band (07_3); what a roll costs with a coin this
//! unfair (07_4); and Pascal's triangle again, every entry a fair choice
//! among that many (07_5).

use explainer_kit::props::{RowStrips, predict_pause};
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    block_grey, caption, column, die_face, disc_row, fade_in_all, fade_out_all, fail, flat, gold, group, line_style,
    pause, rgb, say, thrown, thrown_lit,
};
use crate::facts::coin_die_run;
use crate::model::{
    Face, Q, bent, blocks, count_chance, die_from_four_tosses, pascal_row, pattern_chance, patterns, tally,
    two_sigma_band,
};

pub struct ADieFromACoin {
    pub kit: Kit,
}

/// Tosses in a block: the row of Pascal's triangle the die is made from.
const TOSSES: u32 = 4;
/// Heads a block must show: the middle group of that row.
const HEADS: u32 = 2;
/// Blocks of the run played at reading speed before the whole run lands.
const SHOWN: usize = 8;
/// The first and last rows of the triangle that closes the chapter.
const TRIANGLE: (u32, u32) = (2, 4);
/// The trick's own group: the row for two tosses, one head.
const TRICK: (u32, u32) = (2, 1);

// the chapter's fixed words (numbers are built from facts)
const ROW_FOR: &str = "the row for";
const FOUR_TOSSES: &str = "four tosses";
const COUNTS: &str = "counts";
const GROUP_SIZES: &str = "group sizes";
const HEADS_WORD: &str = "heads";
const HEAD_WORD: &str = "head";
const TAILS_WORD: &str = "tails";
const AND: &str = "and";
const PATTERNS: &str = "patterns";
const TOSS_FOUR: &str = "toss four";
const FRESH_ONES: &str = "fresh ones";
const THE_FIRST: &str = "the first";
const BLOCKS_OF: &str = "blocks of";
const TOSSES_WORD: &str = "tosses";
const THROWN_AWAY: &str = "a block of four tosses, thrown away";
const A_ROLL: &str = "a roll";
const ROLLS: &str = "rolls";
const FAIR_DIE: &str = "a fair die:";
const TO: &str = "to";
const MODEL: &str = "(model)";
const OF_TRIES: &str = "of tries";
const CHANCE_OF: &str = "the chance of";
const EACH_COUNT: &str = "each count";
const MODEL_COST: &str = "model:";
const TOSSES_A_ROLL: &str = "tosses a roll";
const THIS_RUN: &str = "this run,";
const SAME_COUNTS: &str = "same counts, same chance";
const FAIR_CHOICE: &str = "each entry: a fair choice among that many";
const THE_TRICK: &str = "the trick";
const A_DIE: &str = "a die";

// the row of boxes: its height on screen, the boxes' pitch and size
const ROW_Y: f64 = -0.9;
const BOX_PITCH: f64 = 1.8;
const BOX_W: f64 = 1.5;
const BOX_H: f64 = 2.3;
const COUNTS_Y: f64 = -2.37;
const SIZES_Y: f64 = -2.92;
/// The left-hand margin's words sit about this line.
const MARGIN_X: f64 = -5.65;
// a pattern inside a box: its discs, and the pitch of patterns down the box
const SMALL_R: f64 = 0.1;
const SMALL_STEP: f64 = 0.21;
const STRIP_PITCH: f64 = 0.32;
// the six patterns lifted out: how much bigger, and where
const LIFT: f64 = 2.0;
const LIFT_Y: f64 = 2.75;
const LIFT_PITCH: f64 = 2.2;
const DICE_Y: f64 = 1.8;
const DIE: f64 = 0.72;
// the run: a grid of blocks on the left, six bars on the right
const GRID_LEFT: f64 = -6.8;
const GRID_TOP: f64 = 2.1;
const GRID_W: f64 = 6.4;
const COLS: usize = 100;
const BLOCKS_Y: f64 = 3.1;
const MARKS_Y: f64 = 2.55;
const BARS_LEFT: f64 = 0.6;
const BAR_PITCH: f64 = 1.0;
const BAR_FOOT: f64 = -1.5;
/// How tall the upper end of the fair die's band stands.
const BAR_MAX: f64 = 3.9;
const FOOT_DIE: f64 = 0.6;
const RISE: f64 = 3.2;
// the triangle
const TRI_TOP: f64 = 1.5;
const TRI_ROW: f64 = 1.05;
const TRI_PITCH: f64 = 1.3;

/// A pattern as a strip of plain discs centred at `at`: one group, detached.
fn strip(stage: &mut Stage<'_>, pattern: &[Face], at: Vec3, radius: f64, step: f64) -> fmn::Result<Mob> {
    let half = (pattern.len() as f64 - 1.0) / 2.0;
    let discs = disc_row(stage, pattern, v(at[0] - half * step, at[1]), radius, step);
    group(stage, &discs)
}

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

/// A bar standing on `foot` whose height follows `tracker` (`unit` scene
/// units for each 1.0), so a bar and its counter rise from the same number.
/// Returns the anchor, which carries the updater and must be in the scene,
/// and the bar; both detached.
fn rising_bar(
    stage: &mut Stage<'_>,
    tracker: Mob,
    foot: Vec3,
    width: f64,
    unit: f64,
    color: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let bar = stage.arena_mut().add(column(foot, width, 0.0, color));
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let value = s.tracker_value(tracker).unwrap_or(0.0);
            if last == Some(value) {
                return;
            }
            let _ = s.set_points(bar, column(foot, width, unit * value, color).points());
            last = Some(value);
        },
        true,
    )?;
    Ok([anchor, bar])
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

/// The box (0 at the left) that holds the patterns with `heads` heads:
/// heads sort first, as they do in the square.
fn box_of(heads: u32) -> usize {
    (TOSSES - heads) as usize
}

fn box_x(b: usize) -> f64 {
    (b as f64 - f64::from(TOSSES) / 2.0) * BOX_PITCH
}

/// Where pattern `j` of `n` sits in box `b`.
fn in_box(b: usize, j: usize, n: usize) -> Vec3 {
    v(box_x(b), ROW_Y + ((n as f64 - 1.0) / 2.0 - j as f64) * STRIP_PITCH)
}

/// Where lifted pattern `i` of `n` sits.
fn lift_at(i: usize, n: usize) -> Vec3 {
    v((i as f64 - (n as f64 - 1.0) / 2.0) * LIFT_PITCH, LIFT_Y)
}

/// The foot of bar `i`: where its die face sits.
fn bar_x(i: usize) -> f64 {
    BARS_LEFT + (i as f64 + 0.5) * BAR_PITCH
}

/// Entry `j` of the triangle's row for `n` tosses.
fn tri_at(n: u32, j: usize) -> Vec3 {
    v(
        (j as f64 - f64::from(n) / 2.0) * TRI_PITCH,
        TRI_TOP - f64::from(n - TRIANGLE.0) * TRI_ROW,
    )
}

/// The row for four tosses: five boxes, each holding its group's patterns
/// as strips of discs, with the group's counts and size beneath.
struct Row {
    backs: Vec<Mob>,
    /// Per box, its patterns.
    strips: Vec<Vec<Mob>>,
    counts: Vec<Mob>,
    sizes: Vec<Mob>,
    words: Vec<Mob>,
}

impl Row {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let sizes_row = pascal_row(TOSSES);
        let (mut backs, mut strips, mut counts, mut sizes) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for heads in (0..=TOSSES).rev() {
            let b = box_of(heads);
            let x = box_x(b);
            let back = Rectangle::new()
                .width(BOX_W)
                .height(BOX_H)
                .corner_radius(0.1)
                .style(
                    Style::default()
                        .fill(block_grey(), 1.0)
                        .stroke(Palette::faint(), 1.5, 1.0),
                )
                .build()?
                .moved_to(v(x, ROW_Y));
            backs.push(stage.arena_mut().add(back));
            let group_patterns = patterns(TOSSES, heads);
            if group_patterns.len() as u128 != sizes_row[heads as usize] {
                return Err(fail(format!(
                    "the group with {heads} heads is not its entry of the row"
                )));
            }
            let mut column = Vec::new();
            for (j, pattern) in group_patterns.iter().enumerate() {
                let at = in_box(b, j, group_patterns.len());
                column.push(strip(stage, pattern, at, SMALL_R, SMALL_STEP)?);
            }
            strips.push(column);
            let word = if heads == 1 { HEAD_WORD } else { HEADS_WORD };
            let count = format!("{heads} {word}");
            counts.push(kit.text(stage, &count, 22.0, Palette::dim(), v(x, COUNTS_Y))?);
            let size = sizes_row[heads as usize].to_string();
            sizes.push(kit.text(stage, &size, 32.0, Palette::ink(), v(x, SIZES_Y))?);
        }
        let words = vec![
            kit.text(stage, ROW_FOR, 24.0, Palette::dim(), v(MARGIN_X, ROW_Y + 0.2))?,
            kit.text(stage, FOUR_TOSSES, 24.0, Palette::dim(), v(MARGIN_X, ROW_Y - 0.2))?,
            kit.text(stage, COUNTS, 22.0, Palette::dim(), v(MARGIN_X, COUNTS_Y))?,
            kit.text(stage, GROUP_SIZES, 22.0, Palette::dim(), v(MARGIN_X, SIZES_Y))?,
        ];
        Ok(Self {
            backs,
            strips,
            counts,
            sizes,
            words,
        })
    }

    /// Everything but the group sizes, back to front.
    fn body(&self) -> Vec<Mob> {
        let mut out = self.backs.clone();
        out.extend(self.strips.iter().flatten());
        out.extend(&self.counts);
        out.extend(&self.words);
        out
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
        let six = patterns(TOSSES, HEADS);
        let faces = six.len();
        let mid = box_of(HEADS);

        // the row for four tosses, as chapter 06 left it
        let row = Row::build(&kit, stage)?;
        let mut row_mobs = row.body();
        row_mobs.extend(&row.sizes);
        fade_in_all(stage, &row_mobs, 0.6)?;

        // 07_1: the middle group lifts out: six patterns, all equally likely
        let line = say(&kit, stage, "07_1")?;
        let ring = outline(v(box_x(mid), ROW_Y), BOX_W + 0.14, BOX_H + 0.14, 0.14, gold(), 3.5)?;
        let ring = stage.arena_mut().add(ring);
        play!(stage; show_creation(ring).rt(0.7));
        let mut lifted = Vec::new();
        for (j, pattern) in six.iter().enumerate() {
            let copy = strip(stage, pattern, in_box(mid, j, faces), SMALL_R, SMALL_STEP)?;
            stage.add_to_scene(copy)?;
            lifted.push(copy);
        }
        let mut rises: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &copy) in lifted.iter().enumerate() {
            rises.push(stage.prepare(glide(copy, LIFT, lift_at(i, faces), 1.0)?)?);
        }
        let rise = AnimationGroup::with_lag_ratio(stage.arena_mut(), rises, 0.2)?.with_run_time(1.6);
        stage.play(rise)?;
        let mut letters = Vec::new();
        for (i, pattern) in six.iter().enumerate() {
            let c = lift_at(i, faces);
            let half = (pattern.len() as f64 - 1.0) / 2.0;
            for (j, f) in pattern.iter().enumerate() {
                let at = v(c[0] + (j as f64 - half) * SMALL_STEP * LIFT, c[1]);
                letters.push(kit.text(stage, &f.letter().to_string(), 23.0, dark, at)?);
            }
        }
        let heading = format!(
            "{HEADS} {HEADS_WORD} {AND} {} {TAILS_WORD}: {faces} {PATTERNS}",
            TOSSES - HEADS
        );
        let heading = kit.text(stage, &heading, 26.0, Palette::dim(), v(0.0, LIFT_Y + 0.7))?;
        let mut named = letters.clone();
        named.push(heading);
        fade_in_all(stage, &named, 0.5)?;
        line.wait_for(stage, "Six patterns")?;
        let each = format!("{}%", pattern_chance(p, TOSSES, HEADS).percent(2));
        let mut percents = Vec::new();
        let mut shows: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..faces {
            let at = v(lift_at(i, faces)[0], LIFT_Y - 0.5);
            let m = kit.text(stage, &each, 22.0, Palette::dim(), at)?;
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            shows.push(stage.prepare(a)?);
            percents.push(m);
        }
        let shows = AnimationGroup::with_lag_ratio(stage.arena_mut(), shows, 0.3)?.with_run_time(1.2);
        stage.play(shows)?;
        // six of them, the same counts, the same chance: one gold frame
        line.wait_for(stage, "What else has six")?;
        let span = LIFT_PITCH * (faces as f64 - 1.0) + 2.1;
        let six_frame = outline(v(0.0, LIFT_Y - 0.22), span, 1.15, 0.16, gold(), 3.0)?;
        let six_frame = stage.arena_mut().add(six_frame);
        play!(stage; show_creation(six_frame).rt(0.9));
        let q = predict_pause(&kit, stage, v(0.0, 1.1))?;

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
        play!(stage; fmn::animation::indicate(stage.arena_mut(), ring, 1.08, Some(rgb(gold())))?.rt(0.9));
        line.wait_for(stage, "the pattern picks the face")?;
        pulse_each(stage, &dice, 1.2, 1.5)?;
        line.wait_for(stage, "If not")?;
        let loop_at = v(5.65, ROW_Y + 0.55);
        let again = curved_arrow(
            v(loop_at[0] - 0.42, loop_at[1] - 0.3),
            v(loop_at[0] + 0.42, loop_at[1] - 0.3),
            -1.5 * PI,
            line_style(Palette::dim(), 4.0),
        )?;
        let again = stage.arena_mut().add(again);
        let fresh = [
            kit.text(stage, TOSS_FOUR, 24.0, Palette::dim(), v(loop_at[0], ROW_Y - 0.45))?,
            kit.text(stage, FRESH_ONES, 24.0, Palette::dim(), v(loop_at[0], ROW_Y - 0.8))?,
        ];
        let mut greys: Vec<Box<dyn Animation>> = Vec::new();
        for b in (0..row.backs.len()).filter(|&b| b != mid) {
            greys.push(stage.prepare(refill(row.backs[b], thrown(), 0.7)?)?);
            for &s in &row.strips[b] {
                greys.push(stage.prepare(refill(s, thrown_lit(), 0.7)?)?);
            }
        }
        for m in [again, fresh[0], fresh[1]] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            greys.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(greys)?;
        // the row as it now stands (it returns at 07_4)
        let mut rule = row.body();
        rule.extend([ring, again, fresh[0], fresh[1]]);

        // 07_3: the run: every block of four as a cell, and the rolls it made
        let line = say(&kit, stage, "07_3")?;
        let run = coin_die_run();
        let block = TOSSES as usize;
        let made = blocks(&run, block, die_from_four_tosses);
        let counts = tally(&made, faces);
        let cells: Vec<Option<usize>> = run.chunks_exact(block).map(die_from_four_tosses).collect();
        // the row and the patterns make room; the six faces go to the feet of six bars
        let mut clears: Vec<Box<dyn Animation>> = Vec::new();
        let gone: Vec<Mob> = rule
            .iter()
            .chain(&row.sizes)
            .chain(&lifted)
            .chain(&letters)
            .chain([&heading])
            .copied()
            .collect();
        for &m in &gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            clears.push(stage.prepare(a.rt(0.4))?);
        }
        for (i, &die) in dice.iter().enumerate() {
            clears.push(stage.prepare(glide(die, FOOT_DIE / DIE, v(bar_x(i), BAR_FOOT - 0.45), 0.7)?)?);
        }
        stage.play_prepared(clears)?;
        // the first eight blocks at reading speed: a grey cell for a block
        // thrown away, a gold-edged face for a roll
        let first = format!("{THE_FIRST} {SHOWN} {BLOCKS_OF} {block} {TOSSES_WORD}");
        let first = kit.text_left(stage, &first, 22.0, Palette::dim(), v(GRID_LEFT, BLOCKS_Y + 0.48))?;
        let mut early = vec![first];
        let mut plays: Vec<Box<dyn Animation>> = Vec::new();
        let a = fade_in(stage.arena_mut(), first, ORIGIN, 1.0)?;
        plays.push(stage.prepare(a)?);
        for (i, tosses) in run.chunks_exact(block).take(SHOWN).enumerate() {
            let x = GRID_LEFT + (i as f64 + 0.5) * GRID_W / SHOWN as f64;
            let shown = strip(stage, tosses, v(x, BLOCKS_Y), 0.07, 0.155)?;
            let mark = match cells[i] {
                Some(face) => {
                    let die = die_face(stage, face + 1, v(x, MARKS_Y), 0.4, Palette::ink(), dark)?;
                    let edge = outline(v(x, MARKS_Y), 0.5, 0.5, 0.1, gold(), 2.5)?;
                    let edge = stage.arena_mut().add(edge);
                    group(stage, &[die, edge])?
                }
                None => {
                    let cell = Rectangle::new()
                        .width(0.26)
                        .height(0.26)
                        .style(flat(thrown(), 1.0))
                        .build()?
                        .moved_to(v(x, MARKS_Y));
                    stage.arena_mut().add(cell)
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
        // then every block of the run: one cell each, in reading order
        line.wait_for(stage, "made nine hundred")?;
        let pitch = GRID_W / COLS as f64;
        let thick = 0.72 * pitch;
        let rows: Vec<&[Option<usize>]> = cells.chunks(COLS).collect();
        let is_roll = |r: &[Option<usize>], c: usize| r.get(c).copied().flatten().is_some();
        let edges: Vec<Vec<Srgb>> = rows
            .iter()
            .map(|r| (0..COLS).map(|c| if is_roll(r, c) { gold() } else { dark }).collect())
            .collect();
        let under = RowStrips::build(stage, v(GRID_LEFT, GRID_TOP), (pitch, pitch), &edges)?;
        let mut grid = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            // a cell is three quarters of its pitch wide; the gap beside a
            // roll is gold, so a roll has an edge all the way round
            let mut colors = Vec::with_capacity(4 * r.len());
            for c in 0..r.len() {
                let fill = if is_roll(r, c) { Palette::ink() } else { thrown() };
                let gap = if is_roll(r, c) || is_roll(r, c + 1) {
                    gold()
                } else {
                    dark
                };
                colors.extend([fill, fill, fill, gap]);
            }
            let y = GRID_TOP - pitch * (i as f64 + 0.5);
            let top = RowStrips::build(
                stage,
                v(GRID_LEFT + pitch / 8.0, y + thick / 2.0),
                (pitch / 4.0, thick),
                &[colors],
            )?;
            grid.extend([under.rows[i], top.rows[0]]);
        }
        let grid_bottom = GRID_TOP - pitch * rows.len() as f64;
        // six bars, one a face, under where a fair die's counts would land
        let (lo, hi) = two_sigma_band(made.len() as u32, Q::new(1, faces as i128));
        let unit = BAR_MAX / f64::from(hi);
        let mut bars = Vec::new();
        let mut tallies = Vec::new();
        for (i, &count) in counts.iter().enumerate() {
            let t = stage.add_value_tracker(0.0);
            bars.extend(rising_bar(stage, t, v(bar_x(i), BAR_FOOT), 0.6, unit, Palette::dim())?);
            tallies.push((t, count as f64));
        }
        let cap = format!("{} {TOSSES_WORD}, {} {ROLLS}", thousands(run.len() as u64), made.len());
        let cap = caption(&kit, stage, &cap)?;
        let legend_y = grid_bottom - 0.4;
        let swatch = |at: Vec3, color: Srgb| -> fmn::Result<VMobject> {
            Ok(Rectangle::new()
                .width(0.2)
                .height(0.2)
                .style(flat(color, 1.0))
                .build()?
                .moved_to(at))
        };
        let grey_cell = stage.arena_mut().add(swatch(v(GRID_LEFT + 0.12, legend_y), thrown())?);
        let grey_word = kit.text_left(stage, THROWN_AWAY, 22.0, Palette::dim(), v(GRID_LEFT + 0.36, legend_y))?;
        let roll_x = GRID_LEFT + 4.9;
        let roll_cell = stage.arena_mut().add(swatch(v(roll_x, legend_y), Palette::ink())?);
        let roll_edge = outline(v(roll_x, legend_y), 0.27, 0.27, 0.0, gold(), 2.5)?;
        let roll_edge = stage.arena_mut().add(roll_edge);
        let roll_word = kit.text_left(stage, A_ROLL, 22.0, Palette::dim(), v(roll_x + 0.26, legend_y))?;
        let legend = [grey_cell, grey_word, roll_cell, roll_edge, roll_word];
        let mut lands: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &grid {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            lands.push(stage.prepare(a)?);
        }
        let lands = AnimationGroup::with_lag_ratio(stage.arena_mut(), lands, 0.12)?.with_run_time(RISE);
        let mut all: Vec<Box<dyn Animation>> = vec![Box::new(lands)];
        for &bar in &bars {
            stage.add_to_scene(bar)?;
        }
        let mut live = Vec::new();
        for (i, &(t, count)) in tallies.iter().enumerate() {
            let at = v(bar_x(i), BAR_FOOT - 1.05);
            live.push(kit.live_number(stage, t, 0, false, 24.0, Palette::ink(), at, ORIGIN)?);
            all.push(stage.prepare(tween(t, count, RISE)?)?);
        }
        for m in legend.into_iter().chain([cap]) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            all.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(all)?;
        line.wait_for(stage, "Each face came up")?;
        pulse_each(stage, &dice, 1.25, 1.3)?;
        line.wait_for(stage, "about as often")?;
        let (x0, x1) = (BARS_LEFT, BARS_LEFT + BAR_PITCH * faces as f64);
        let height = |k: u32| BAR_FOOT + unit * f64::from(k);
        let band = Polygon::new([
            v(x0, height(lo)),
            v(x1, height(lo)),
            v(x1, height(hi)),
            v(x0, height(hi)),
        ])
        .style(flat(Palette::ink(), 0.14))
        .build();
        let mut fair = vec![stage.arena_mut().add(band)];
        for k in [lo, hi] {
            let edge = polyline(vec![v(x0, height(k)), v(x1, height(k))])
                .style(line_style(Palette::ink(), 1.5))
                .build();
            fair.push(stage.arena_mut().add(edge));
        }
        let fair_words = format!("{FAIR_DIE} {lo} {TO} {hi} {MODEL}");
        fair.push(kit.text(
            stage,
            &fair_words,
            24.0,
            Palette::dim(),
            v((x0 + x1) / 2.0, height(hi) + 0.32),
        )?);
        fade_in_all(stage, &fair, 0.7)?;

        // 07_4: it isn't cheap: how seldom four tosses show exactly two heads
        let line = say(&kit, stage, "07_4")?;
        let mut run_panel = early.clone();
        run_panel.extend(&grid);
        run_panel.extend(&bars);
        run_panel.extend(&live);
        run_panel.extend(&dice);
        run_panel.extend(&fair);
        run_panel.extend(legend);
        run_panel.push(cap);
        fade_out_all(stage, &run_panel, 0.5)?;
        let mut back = rule.clone();
        back.extend(&row.sizes);
        fade_in_all(stage, &back, 0.6)?;
        line.wait_for(stage, "A coin this unfair")?;
        let chance_foot = ROW_Y + BOX_H / 2.0 + 0.2;
        let chance_unit = 1.5 / count_chance(p, TOSSES, TOSSES).f64();
        let mut chances = vec![
            kit.text(stage, CHANCE_OF, 22.0, Palette::dim(), v(MARGIN_X, chance_foot + 0.75))?,
            kit.text(stage, EACH_COUNT, 22.0, Palette::dim(), v(MARGIN_X, chance_foot + 0.42))?,
        ];
        let mut grows: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &chances {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            grows.push(stage.prepare(a.rt(0.6))?);
        }
        let mut works_label = None;
        for heads in (0..=TOSSES).rev() {
            let chance = count_chance(p, TOSSES, heads);
            let x = box_x(box_of(heads));
            let h = chance_unit * chance.f64();
            let lit = heads == HEADS;
            let color = if lit { Palette::ink() } else { Palette::dim() };
            let t = stage.add_value_tracker(0.0);
            let bar = rising_bar(stage, t, v(x, chance_foot), 0.9, chance_unit, color)?;
            for m in bar {
                stage.add_to_scene(m)?;
            }
            grows.push(stage.prepare(tween(t, chance.f64(), 1.0)?)?);
            chances.extend(bar);
            let at = v(x, chance_foot + h + 0.28);
            let percent = format!("{}%", chance.percent(1));
            if lit {
                let words = format!("{percent} {OF_TRIES}");
                works_label = Some(kit.text(stage, &words, 26.0, Palette::ink(), at)?);
            } else {
                let m = kit.text(stage, &percent, 22.0, Palette::dim(), at)?;
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                grows.push(stage.prepare(a.rt(1.0))?);
                chances.push(m);
            }
        }
        stage.play_prepared(grows)?;
        let works_label = works_label.ok_or_else(|| fail("the row has no middle group".into()))?;
        fade_in_all(stage, &[works_label], 0.5)?;
        chances.push(works_label);
        line.wait_for(stage, "A roll costs")?;
        let works = count_chance(p, TOSSES, HEADS);
        let model_cost = format!(
            "{MODEL_COST} {} {TOSSES_A_ROLL}",
            (Q::int(i128::from(TOSSES)) / works).decimal(1)
        );
        let run_cost = format!(
            "{THIS_RUN} {} {TOSSES_WORD}: {}",
            thousands(run.len() as u64),
            Q::new(run.len() as i128, made.len() as i128).decimal(1)
        );
        let cost = [
            kit.text(stage, &model_cost, 32.0, Palette::ink(), v(0.0, 3.4))?,
            kit.text(stage, &run_cost, 24.0, Palette::dim(), v(0.0, 2.88))?,
        ];
        fade_in_all(stage, &cost[..1], 0.6)?;
        pause(stage, 0.5)?;
        fade_in_all(stage, &cost[1..], 0.6)?;

        // 07_5: the triangle again: every entry a fair choice among that many
        let line = say(&kit, stage, "07_5")?;
        let mut clears: Vec<Box<dyn Animation>> = Vec::new();
        for &m in rule.iter().chain(&chances).chain(&cost) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            clears.push(stage.prepare(a.rt(0.5))?);
        }
        // the row's group sizes become the triangle's last row
        for (j, &size) in row.sizes.iter().enumerate() {
            clears.push(stage.prepare(glide(size, 44.0 / 32.0, tri_at(TOSSES, j), 0.9)?)?);
        }
        stage.play_prepared(clears)?;
        let mut entries: Vec<(u32, usize, Mob)> = Vec::new();
        let mut arrives = Vec::new();
        for n in TRIANGLE.0..=TRIANGLE.1 {
            let label = format!("{n} {TOSSES_WORD}");
            let at = v(tri_at(n, 0)[0] - 2.0, tri_at(n, 0)[1]);
            arrives.push(kit.text(stage, &label, 22.0, Palette::dim(), at)?);
            for (j, size) in pascal_row(n).iter().enumerate() {
                let m = if n == TOSSES {
                    row.sizes[j]
                } else {
                    let m = kit.text(stage, &size.to_string(), 44.0, Palette::ink(), tri_at(n, j))?;
                    arrives.push(m);
                    m
                };
                entries.push((n, j, m));
            }
        }
        fade_in_all(stage, &arrives, 0.7)?;
        line.wait_for(stage, "Any group of patterns")?;
        let same = kit.text(stage, SAME_COUNTS, 34.0, gold(), v(0.0, TRI_TOP + 1.15))?;
        let choice = kit.text(
            stage,
            FAIR_CHOICE,
            28.0,
            Palette::ink(),
            v(0.0, tri_at(TRIANGLE.1, 0)[1] - 1.25),
        )?;
        fade_in_all(stage, &[same, choice], 0.6)?;
        // each entry glows in turn; the two this video has used are named
        let mut glows: Vec<Box<dyn Animation>> = Vec::new();
        for &(n, j, m) in &entries {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.4, Some(rgb(gold())))?;
            glows.push(stage.prepare(a)?);
            // (tosses, heads): entry `tosses - heads` of its row, as in the boxes
            let used = [(TRICK, THE_TRICK), ((TOSSES, HEADS), A_DIE)];
            for ((tosses, heads), name) in used {
                if (n, j) == (tosses, (tosses - heads) as usize) {
                    let at = v(tri_at(n, j)[0], tri_at(n, j)[1] - 0.47);
                    let tag = kit.text(stage, name, 22.0, Palette::dim(), at)?;
                    let a = fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?;
                    glows.push(stage.prepare(a)?);
                }
            }
        }
        let glows = AnimationGroup::with_lag_ratio(stage.arena_mut(), glows, 0.6)?.with_run_time(4.2);
        stage.play(glows)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
