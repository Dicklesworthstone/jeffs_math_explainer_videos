//! Chapter 05, the triangles. The wait for every moment of chapter 04's
//! stretch, drawn over its line with a minute up as long as a minute
//! across:
//! - 05_1: the line becomes the plot's bottom axis; up is the wait.
//! - 05_2: you ride the axis from one blue bus to the next, and a point
//!   above you traces your wait, falling steadily to nothing at each bus
//!   and jumping back up to the whole next gap.
//! - 05_3: the trace closes into two triangles, each as wide as its gap
//!   and as tall.
//! - 05_4: each passenger's bar widens to its minute; the corner it puts
//!   above the slant equals the corner it leaves under it, so the bars hold
//!   exactly the triangles' area.
//! - 05_5: the two square-minute areas represent chapter 04's passenger
//!   waiting when multiplied by one passenger per minute.
//! - 05_6: five times as wide, how many times the waiting? (a prediction;
//!   no tiles on screen.)
//! - 05_7: copies of the small triangle fly into the large one and tile
//!   it, upright and upside down: five times as wide and five times as
//!   tall.
//! - 05_8: the waiting grows with the square of the gap: a triangle half as
//!   wide fits four times into the long one.
//!
//! Every number on screen is read from the facts, and
//! `twelve_minutes::numbers` checks each against the stretch; the tiles are
//! counted here and checked against those numbers before the chapter
//! starts.

use std::rc::Rc;

use explainer_kit::{Kit, Palette, TextKind, Timed, fade_all, ink_style, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{
    fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all, flash_lagged, flat, line_style, now, say,
    stand, steady, think, you,
};
use super::timetable::{self, Gap};
use super::twelve_minutes::{
    self, AXIS_Y, BAR_W, Numbers, ORIGIN_X, PANEL_X, Passenger, at, equal_scale, fade_in_flashing, frame, in_gap, pick,
    row,
};
use crate::facts;
use crate::model::{self, PER_MINUTE, Q};
use crate::palette;

pub struct Triangles {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const WAIT_NAME: &str = "wait (min)";
const MOMENT_NAME: &str = "moment you turn up (min)";
const WAIT_WORD: &str = "wait:";
const TIMES: &str = "×";
const ASK: &str = "how many times the waiting?";
const GAP_WORD: &str = "gap";
const WAITING: &str = "waiting";
const BECOMES: &str = "→";
const AREA_UNIT: &str = "min²";

// ------------------------------------------------------------------ layout

/// 05_1: the wait's axis up from the stretch's first moment, a little past
/// the longest wait, with a tick at each gap's length; its name over it, and
/// the moment's name right of the line's end.
const Y_OVERHANG: f64 = 0.05;
const Y_TICK_W: f64 = 0.1;
const Y_LABEL_DX: f64 = 0.3;
const WAIT_NAME_DY: f64 = 0.2;
const MOMENT_NAME_DX: f64 = 0.3;
const SMALL: f64 = 18.0;
/// 05_2: you ride at this many seconds a minute, the point above you
/// tracing the wait; the readout left-aligned in the panel's top row. The
/// short ride stops a hair before the red bus, so that the jump comes with
/// its words ("then jumps back up"), as the long ride starts.
const RIDE_S_PER_MIN: f64 = 0.5;
const JUST_BEFORE: f64 = 1e-6;
const TRACE_W: f64 = 4.0;
const POINT_R: f64 = 0.07;
const READOUT_X: f64 = 3.0;
/// Drawing order over the stretch (bars 1, passengers 2): the trace, the
/// corners and the tiles, then you and your point.
const TRACE_Z: i32 = 3;
const CORNER_Z: i32 = 3;
const TILE_Z: i32 = 1;
const YOU_Z: i32 = 4;
const POINT_Z: i32 = 5;
/// 05_3: a triangle's fill under its outline; a pulse brightens it.
const TRI_FILL: f64 = 0.35;
const TRI_W: f64 = 3.0;
const PULSE_FILL: f64 = 0.7;
const PULSE_RT: f64 = 0.8;
/// 05_4: the bars' widening, and the corners' fill.
const WIDEN_RT: f64 = 0.8;
const CORNER_FILL: f64 = 0.85;
/// The panel (`twelve_minutes::row`): the areas in rows 0 to 2, the
/// question and then the answer in row 3, the think ring under it, the
/// tiles' count in row 4 and the closing card in row 5.
const PANEL_SIZE: f64 = 24.0;
const BIG_SIZE: f64 = 36.0;
const AREA_ROWS: [usize; 3] = [0, 1, 2];
const ASK_ROW: usize = 3;
const THINK_AT: Vec3 = [PANEL_X, -0.45, 0.0];
const COUNT_ROW: usize = 4;
const CARD_ROW: usize = 5;
/// 05_6: an arrow under the brackets from the short gap's length to the
/// long gap's, its label under it.
const ARROW_Y: f64 = -2.75;
const ARROW_INSET: f64 = 0.3;
const RATIO_Y: f64 = -3.05;
const RATIO_SIZE: f64 = 20.0;
/// 05_7: each tile a little smaller than its place, so that every copy
/// shows; their flight; the time to look at the finished tiling.
const TILE_K: f64 = 0.9;
const TILE_FILL: f64 = 0.6;
const TILE_RT: f64 = 2.2;
const TILE_LAG: f64 = 0.15;
const LOOK_S: f64 = 1.5;

// ------------------------------------------------------------- the waiting

/// The wait of someone who turns up at moment `t` of the stretch, whole or
/// not: until the stretch's next bus after `t` (a bus at `t` itself has
/// just gone, as `model::wait_at` has it). At the stretch's end there is no
/// next bus in it, and the trace ends: nothing.
pub fn wait_in(t: f64) -> f64 {
    twelve_minutes::buses()
        .into_iter()
        .map(|b| b as f64)
        .find(|&b| b > t)
        .map_or(0.0, |b| b - t)
}

/// What the point above you has traced once you have ridden to moment `t`,
/// as (moment, wait) corners: down each gap's slant from the whole gap,
/// and up the jump at each bus passed, to where you are now. One corner
/// alone before you move.
pub fn trace(t: f64) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for g in twelve_minutes::gaps() {
        let (a, b) = (g.from as f64, g.to as f64);
        if t < a {
            break;
        }
        let end = t.min(b);
        for corner in [(a, b - a), (end, b - end)] {
            if out.last() != Some(&corner) {
                out.push(corner);
            }
        }
    }
    out
}

/// A gap's triangle as (moment, wait) corners: at its first bus, from the
/// line up the whole gap; across to its next bus.
pub fn triangle(g: &Gap) -> [(f64, f64); 3] {
    let (a, b) = (g.from as f64, g.to as f64);
    [(a, 0.0), (a, b - a), (b, 0.0)]
}

/// Where a passenger's bar, widened to its whole minute, and its gap's
/// slant part: the corner the bar puts above the slant (right of its
/// middle), and the corner it leaves empty under the slant (left of its
/// middle). The slant touches the bar's top at its middle, so the two are
/// the same right triangle, half a minute each way.
pub fn corners(p: &Passenger) -> [[(f64, f64); 3]; 2] {
    let (t, w) = (p.t.to_f64(), p.wait.to_f64());
    let h = 1.0 / (2 * PER_MINUTE) as f64;
    [
        [(t, w), (t + h, w), (t + h, w - h)],
        [(t - h, w), (t - h, w + h), (t, w)],
    ]
}

/// One copy of a small triangle inside a large one, as (moment, wait)
/// corners, its right angle first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    pub upright: bool,
    pub corners: [(f64, f64); 3],
}

/// The copies of a `small`-minute gap's triangle that tile `gap`'s triangle
/// (its length a whole number of times `small`): in each square of a
/// `small` grid under the slant, the lower left half upright, and the
/// upper right half upside down wherever it fits under the slant. Upright
/// first, each by rows from the line up.
///
/// An upright tile's corners run right angle, top, right, as the small
/// triangle's do; an upside-down tile's run right angle, top left, bottom
/// right, so a copy of the small triangle flown onto it vertex by vertex
/// folds over its long side as it goes.
pub fn tiles(gap: &Gap, small: i64) -> Result<Vec<Tile>, String> {
    if small <= 0 || gap.minutes() % small != 0 {
        return Err(format!(
            "a {small}-minute triangle cannot tile the {}-minute one",
            gap.minutes()
        ));
    }
    let n = gap.minutes() / small;
    let s = small as f64;
    let mut out = Vec::new();
    for upright in [true, false] {
        for j in 0..n {
            for i in 0..n - j {
                let (x, y) = (gap.from as f64 + s * i as f64, s * j as f64);
                if upright {
                    out.push(Tile {
                        upright,
                        corners: [(x, y), (x, y + s), (x + s, y)],
                    });
                } else if i + j + 1 < n {
                    out.push(Tile {
                        upright,
                        corners: [(x + s, y + s), (x, y + s), (x + s, y)],
                    });
                }
            }
        }
    }
    Ok(out)
}

/// `corners` scaled by `k` about their centroid.
pub fn shrunk(corners: [(f64, f64); 3], k: f64) -> [(f64, f64); 3] {
    let cx = corners.iter().map(|c| c.0).sum::<f64>() / 3.0;
    let cy = corners.iter().map(|c| c.1).sum::<f64>() / 3.0;
    corners.map(|(x, y)| (cx + k * (x - cx), cy + k * (y - cy)))
}

/// The readout under you in 05_2: "wait: 9:30", rounded to whole seconds
/// (it changes faster than it can be read).
pub fn readout(minutes: f64) -> Result<String, String> {
    let thousandths = Q::new((minutes * 60_000.0).round() as i64, 60_000);
    Ok(format!("{WAIT_WORD} {}", model::clock_rounded(thousandths)?))
}

// ----------------------------------------------------------------- picture

/// (moment, wait) corners on screen.
fn on_screen(corners: &[(f64, f64)]) -> Vec<Vec3> {
    corners.iter().map(|&(t, w)| at(t, w)).collect()
}

/// A filled triangle, no outline, detached.
fn filled(stage: &mut Stage<'_>, corners: [(f64, f64); 3], color: Srgb, fill: f64) -> Mob {
    stage
        .arena_mut()
        .add(Polygon::new(on_screen(&corners)).style(flat(color, fill)).build())
}

/// A triangle's outline alone, closed, detached.
fn outline(stage: &mut Stage<'_>, corners: [(f64, f64); 3], color: Srgb, w: f64) -> Mob {
    let mut closed = corners.to_vec();
    closed.push(corners[0]);
    stage
        .arena_mut()
        .add(polyline(on_screen(&closed)).style(line_style(color, w)).build())
}

/// The wait's axis, detached: its line, a tick and a label at each gap's
/// length, and its name.
struct WaitAxis {
    line: Mob,
    ticks: Vec<Mob>,
    labels: Vec<Mob>,
    name: Mob,
}

fn wait_axis(kit: &Kit, stage: &mut Stage<'_>, n: &Numbers, gaps: &[Gap]) -> fmn::Result<WaitAxis> {
    let top = at(0.0, frame().y.1);
    let line = stage.arena_mut().add(
        Line::new(at(0.0, 0.0), v(top[0], top[1] + Y_OVERHANG))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let mut ticks = Vec::new();
    let mut labels = Vec::new();
    for (g, label) in gaps.iter().zip(&n.gap_minutes) {
        if *label != g.minutes().to_string() {
            return Err(fail(format!("the tick at {} min is labelled {label}", g.minutes())));
        }
        let y = at(0.0, g.minutes() as f64)[1];
        ticks.push(
            stage.arena_mut().add(
                Line::new(v(ORIGIN_X - Y_TICK_W, y), v(ORIGIN_X, y))
                    .style(line_style(Palette::dim(), 2.0))
                    .build()?,
            ),
        );
        labels.push(kit.text(stage, label, SMALL, Palette::dim(), v(ORIGIN_X - Y_LABEL_DX, y))?);
    }
    let name = kit.text(
        stage,
        WAIT_NAME,
        SMALL,
        Palette::dim(),
        v(ORIGIN_X, top[1] + Y_OVERHANG + WAIT_NAME_DY),
    )?;
    Ok(WaitAxis {
        line,
        ticks,
        labels,
        name,
    })
}

/// An updater riding you along the line with the tracker `ride` (a moment
/// of the stretch), the point above you at your wait, and the trace
/// following. Returns its anchor, already in the scene.
fn rider(stage: &mut Stage<'_>, ride: Mob, me: Mob, point: Mob, path: Mob) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = s.tracker_value(ride).unwrap_or(0.0);
            s.move_to(me, at(t, 0.0), ORIGIN);
            s.move_to(point, at(t, wait_in(t)), ORIGIN);
            let corners = trace(t);
            if corners.len() > 1 {
                let _ = s.set_points(path, polyline(on_screen(&corners)).build().points());
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// The readout following the tracker `ride`, left-aligned at `left` as
/// `Kit::text_left` places a text, so it can stand in for one. Detached
/// (out of the scene) until added.
fn live_readout(kit: &Kit, stage: &mut Stage<'_>, ride: Mob, left: Vec3) -> Mob {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let t = s.tracker_value(ride).unwrap_or(0.0);
        let text = readout(wait_in(t)).expect("a wait in the stretch is never negative");
        explainer_kit::glyphs::record(TextKind::Text, &text);
        let shown = Text::new(&text)
            .font_size(PANEL_SIZE)
            .style(ink_style(palette::you()))
            .build(&book)
            .expect("the bundled face lays out digits");
        let m = s.add(shown);
        s.move_to(m, left, LEFT);
        m
    });
    stage.remove_from_scene(m);
    m
}

// --------------------------------------------------------------- playback

/// Mobjects on screen fade out while detached ones fade in, together over
/// `rt` seconds.
fn cross_fade(stage: &mut Stage<'_>, leaving: &[Mob], coming: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in leaving {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in coming {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Builder animations together, each made with its own run time.
fn play_together(stage: &mut Stage<'_>, builders: Vec<AnimBuilder>) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for b in builders {
        anims.push(stage.prepare(b)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Brighten filled triangles and let them settle back: an area pointed at.
/// (A flash would scale a triangle off its axes.)
fn pulse(stage: &mut Stage<'_>, fills: &[Mob]) -> fmn::Result<()> {
    for opacity in [PULSE_FILL, TRI_FILL] {
        let builders = fills
            .iter()
            .map(|&m| fade_to(m, opacity, PULSE_RT / 2.0))
            .collect::<fmn::Result<Vec<_>>>()?;
        play_together(stage, builders)?;
    }
    Ok(())
}

/// Copies of the triangle at `from` flying onto `tiles` one after another,
/// each a little smaller than its place: the flight, to play, and the
/// tiles it leaves in the scene.
fn tile_flight(
    stage: &mut Stage<'_>,
    from: [(f64, f64); 3],
    tiles: &[Tile],
    color: Srgb,
) -> fmn::Result<(AnimationGroup, Vec<Mob>)> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    let mut placed = Vec::new();
    for tile in tiles {
        let copy = filled(stage, shrunk(from, TILE_K), color, TILE_FILL);
        let target = filled(stage, shrunk(tile.corners, TILE_K), color, TILE_FILL);
        stage.set_z_index(target, TILE_Z, true);
        anims.push(stage.prepare(replacement_transform(copy, target))?);
        placed.push(target);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, TILE_LAG)?.with_run_time(TILE_RT);
    Ok((g, placed))
}

impl SceneConstruct for Triangles {
    fn name(&self) -> &str {
        "05_triangles"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let n = twelve_minutes::numbers(&f).map_err(fail)?;
        equal_scale(&frame()).map_err(fail)?;
        let people = twelve_minutes::passengers().map_err(fail)?;
        let gs = twelve_minutes::gaps();
        let (short_gap, long_gap) = (gs[0], gs[1]);
        let long = in_gap(&people, 1);
        let (teal, gold) = (timetable::short_gap(), palette::long_gap());
        let colors = [teal, gold];
        // the tiles, counted here, must be the facts' counts
        let pieces = tiles(&long_gap, short_gap.minutes()).map_err(fail)?;
        let upright = pieces.iter().filter(|t| t.upright).count();
        if [upright.to_string(), (pieces.len() - upright).to_string()] != n.tiles {
            return Err(fail(format!(
                "{upright} upright and {} upside-down tiles, not {:?}",
                pieces.len() - upright,
                n.tiles
            )));
        }
        let k: i64 = n.double.parse().map_err(|e| fail(format!("F-A1.double: {e}")))?;
        let halves = tiles(&long_gap, long_gap.minutes() / k).map_err(fail)?;
        if halves.len().to_string() != n.double_area_ratio {
            return Err(fail(format!(
                "{} half-size triangles fill the long one, not {}",
                halves.len(),
                n.double_area_ratio
            )));
        }

        // 05_1: the stretch as chapter 04 left it, its bars thin and
        // unlabelled, before the line; the wait's axis rises from the
        // line's start
        let st = twelve_minutes::stretch(&kit, stage, &n, &people)?;
        let bars = twelve_minutes::bars(stage, &people)?;
        let mut opening = st.mobs();
        opening.extend(&bars);
        fade_in_all(stage, &opening, 0.8)?;
        let line = say(&kit, stage, "05_1")?;
        line.wait_for(stage, "Now draw the wait")?;
        let axis = wait_axis(&kit, stage, &n, &gs)?;
        let rise = fmn::animation::grow_from_edge(stage.arena_mut(), axis.line, DOWN, None)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(rise.rt(0.8))?];
        for &m in axis.ticks.iter().chain(&axis.labels) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.8))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "Across is the moment")?;
        let end = at(model::PERIOD as f64, 0.0);
        let moment_name = kit.text_left(
            stage,
            MOMENT_NAME,
            SMALL,
            Palette::dim(),
            v(end[0] + MOMENT_NAME_DX, AXIS_Y),
        )?;
        fade_in_flashing(stage, &[moment_name], &st.clocks, 0.6)?;
        line.wait_for(stage, "Up is how long")?;
        fade_in_flashing(stage, &[axis.name], &axis.labels, 0.6)?;

        // 05_2: you ride the line; the point above you traces your wait,
        // the readout follows it
        let line = say(&kit, stage, "05_2")?;
        let ride = stage.add_value_tracker(0.0);
        let me = you(stage, at(0.0, 0.0))?;
        stage.set_z_index(me, YOU_Z, true);
        let point = stage.arena_mut().add(
            Dot::new()
                .point(at(0.0, wait_in(0.0)))
                .radius(POINT_R)
                .style(flat(palette::you(), 1.0))
                .build(),
        );
        stage.set_z_index(point, POINT_Z, true);
        let left = v(READOUT_X, row(0));
        let still = kit.text_left(
            stage,
            &readout(wait_in(0.0)).map_err(fail)?,
            PANEL_SIZE,
            palette::you(),
            left,
        )?;
        fade_in_all(stage, &[me, point, still], 0.5)?;
        line.wait_for(stage, "wait the whole gap")?;
        flash_all(stage, &[point, still, st.brackets[0].label], 0.6)?;
        let path = stage.arena_mut().add(
            polyline(on_screen(&[(0.0, wait_in(0.0)), (0.01, wait_in(0.01))]))
                .style(line_style(palette::you(), TRACE_W))
                .build(),
        );
        stage.set_z_index(path, TRACE_Z, true);
        let anchor = rider(stage, ride, me, point, path)?;
        stage.add_to_scene(path)?;
        let live = live_readout(&kit, stage, ride, left);
        stage.remove_from_scene(still);
        stage.add_to_scene(live)?;
        line.wait_for(stage, "shrinks steadily")?;
        let short_ride = RIDE_S_PER_MIN * short_gap.minutes() as f64;
        stage.play(steady(ride, short_gap.to as f64 - JUST_BEFORE, short_ride)?)?;
        line.wait_for(stage, "then jumps back up")?;
        let long_ride = RIDE_S_PER_MIN * long_gap.minutes() as f64;
        stage.play(steady(ride, long_gap.to as f64, long_ride)?)?;
        stage.clear_updaters(anchor, true);

        // 05_3: the trace closes into the two triangles; you leave, and
        // the buses (each triangle's edge stands where its bus stood); as
        // wide as the gap, and as tall
        let line = say(&kit, stage, "05_3")?;
        line.wait_for(stage, "each gap draws a triangle")?;
        let mut fills = Vec::new();
        let mut shapes = Vec::new();
        for (g, &color) in gs.iter().zip(&colors) {
            let fill = filled(stage, triangle(g), color, TRI_FILL);
            fills.push(fill);
            shapes.extend([fill, outline(stage, triangle(g), color, TRI_W)]);
        }
        let mut leaving = vec![me, point, path, live];
        leaving.extend(&st.markers);
        cross_fade(stage, &leaving, &shapes, 0.8)?;
        line.wait_for(stage, "as wide as the gap")?;
        let gap_labels: Vec<Mob> = st.brackets.iter().map(|b| b.label).collect();
        flash_all(stage, &gap_labels, 0.7)?;
        line.wait_for(stage, "just as tall")?;
        let mut levels = Vec::new();
        for g in gs.iter().filter(|g| g.from > 0) {
            let apex = at(g.from as f64, g.minutes() as f64);
            levels.push(
                stage.arena_mut().add(
                    Line::new(at(0.0, g.minutes() as f64), apex)
                        .style(line_style(Palette::faint(), 2.0))
                        .build()?,
                ),
            );
        }
        fade_in_flashing(stage, &levels, &axis.labels, 0.7)?;

        // 05_4: each bar widens to its minute; the corners it puts above
        // the slant and the equal corners it leaves under it; the bars go,
        // and the triangles hold the waiting
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, "a bar one minute wide")?;
        let minute = at(1.0 / PER_MINUTE as f64, 0.0)[0] - at(0.0, 0.0)[0];
        let widen = bars
            .iter()
            .map(|&b| -> fmn::Result<AnimBuilder> {
                Ok(b.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(WIDEN_RT),
                        ..AnimateArgs::default()
                    })?
                    .stretch(minute / BAR_W, 0)?)
            })
            .collect::<fmn::Result<Vec<_>>>()?;
        play_together(stage, widen)?;
        line.wait_for(stage, "The bars hold exactly")?;
        let mut bits = Vec::new();
        for p in &people {
            for c in corners(p) {
                let m = filled(stage, c, palette::you(), CORNER_FILL);
                stage.set_z_index(m, CORNER_Z, true);
                bits.push(m);
            }
        }
        fade_in_all(stage, &bits, 0.5)?;
        line.wait_for(stage, "So the total waiting")?;
        let mut gone = bits.clone();
        gone.extend(&bars);
        fade_out_all(stage, &gone, 0.6)?;
        pulse(stage, &fills)?;

        // 05_5: each triangle's area as it is said, then both: chapter 04's
        // minutes of waiting
        let line = say(&kit, stage, "05_5")?;
        let mut areas = Vec::new();
        for (i, phrase) in ["half of two times two", "half of ten times ten"]
            .into_iter()
            .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let g = &n.gap_minutes[i];
            let card = kit.text(
                stage,
                &format!("{} {TIMES} {g} {TIMES} {g} = {} {AREA_UNIT}", n.half, n.areas[i]),
                PANEL_SIZE,
                colors[i],
                v(PANEL_X, row(AREA_ROWS[i])),
            )?;
            fade_in_all(stage, &[card], 0.5)?;
            pulse(stage, &[fills[i]])?;
            areas.push(card);
        }
        line.wait_for(stage, "the fifty-two minutes")?;
        let both = kit.text(
            stage,
            &format!("{} + {} = {} {AREA_UNIT}", n.areas[0], n.areas[1], n.total_area),
            PANEL_SIZE,
            Palette::ink(),
            v(PANEL_X, row(AREA_ROWS[2])),
        )?;
        fade_in_flashing(stage, &[both], &areas, 0.6)?;

        // 05_6: five times as wide, under the brackets; the question; think
        // time; no tiles on screen
        let line = say(&kit, stage, "05_6")?;
        line.wait_for(stage, "five times as wide")?;
        let (a, b) = (
            at((short_gap.from + short_gap.to) as f64 / 2.0, 0.0)[0],
            at((long_gap.from + long_gap.to) as f64 / 2.0, 0.0)[0],
        );
        let arrow = stage.arena_mut().add(
            Arrow::new(v(a + ARROW_INSET, ARROW_Y), v(b - ARROW_INSET, ARROW_Y))
                .buff(0.0)
                .thickness(3.0)
                .style(
                    Style::default()
                        .fill(Palette::ink(), 1.0)
                        .stroke(Palette::ink(), 3.0, 1.0),
                )
                .build()?,
        );
        let ratio = kit.text(
            stage,
            &format!("{TIMES} {}", n.width_ratio),
            RATIO_SIZE,
            Palette::ink(),
            v((a + b) / 2.0, RATIO_Y),
        )?;
        fade_in_flashing(stage, &[arrow, ratio], &gap_labels, 0.6)?;
        line.wait_for(stage, "How many times")?;
        let ask = kit.text(stage, ASK, PANEL_SIZE, Palette::ink(), v(PANEL_X, row(ASK_ROW)))?;
        fade_in_all(stage, &[ask], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 05_7: twenty-five times; copies of the small triangle tile the
        // long one; five times as wide, as tall, as many passengers
        let line = say(&kit, stage, "05_7")?;
        fade_out_all(stage, &[q, ask], 0.3)?;
        let answer = kit.text(
            stage,
            &format!("{TIMES} {}", n.area_ratio),
            BIG_SIZE,
            gold,
            v(PANEL_X, row(ASK_ROW)),
        )?;
        let (flight, placed) = tile_flight(stage, triangle(&short_gap), &pieces, teal)?;
        let shown = fade_in(stage.arena_mut(), answer, ORIGIN, 1.0)?;
        let anims = vec![stage.prepare(flight)?, stage.prepare(shown.rt(0.4))?];
        stage.play_prepared(anims)?;
        line.wait_for(stage, "five times as wide")?;
        flash_all(stage, &[ratio], 0.6)?;
        line.wait_for(stage, "five times as tall")?;
        flash_all(stage, &axis.labels, 0.6)?;
        line.wait_for(stage, "Five times as many passengers")?;
        flash_lagged(stage, &pick(&st.dots, &long), 1.0, 0.1)?;
        let count = kit.text(
            stage,
            &format!("{} + {} = {}", n.tiles[0], n.tiles[1], n.area_ratio),
            PANEL_SIZE,
            Palette::dim(),
            v(PANEL_X, row(COUNT_ROW)),
        )?;
        fade_in_all(stage, &[count], 0.4)?;
        kit.hold(stage)?;
        kit.music_breath(stage, LOOK_S)?;

        // 05_8: the tiles go; the square of the gap; half the long gap's
        // triangle, and the three more that fill the long one
        let line = say(&kit, stage, "05_8")?;
        let mut gone = placed;
        gone.push(count);
        fade_out_all(stage, &gone, 0.5)?;
        line.wait_for(stage, "the square of the gap")?;
        flash_all(stage, &[ratio, answer], 0.7)?;
        line.wait_for(stage, "Double a gap")?;
        // The previous 2-to-10 comparison is over. Show the new 5-to-10
        // comparison explicitly, so the surviving ×5/×25 cannot be read
        // as part of the doubling claim.
        fade_out_all(stage, &[arrow, ratio, answer], 0.3)?;
        let half_minutes = (long_gap.to - long_gap.from) / k;
        let half_end = long_gap.from + half_minutes;
        let left = at(long_gap.from as f64, 0.0)[0];
        let right = at(half_end as f64, 0.0)[0];
        let mut half_bracket = Vec::new();
        for (a, b) in [
            (v(left, ARROW_Y), v(right, ARROW_Y)),
            (v(left, ARROW_Y), v(left, ARROW_Y + 0.12)),
            (v(right, ARROW_Y), v(right, ARROW_Y + 0.12)),
        ] {
            half_bracket.push(
                stage
                    .arena_mut()
                    .add(Line::new(a, b).style(line_style(palette::verdict(), 2.0)).build()?),
            );
        }
        let half_label = kit.text(
            stage,
            &format!("{half_minutes} min"),
            RATIO_SIZE,
            palette::verdict(),
            v((left + right) / 2.0, RATIO_Y),
        )?;
        let doubling = kit.text(
            stage,
            &format!("{half_minutes} {TIMES} {} = {} min", n.double, n.gap_minutes[1]),
            PANEL_SIZE,
            palette::verdict(),
            v(PANEL_X, row(ASK_ROW)),
        )?;
        let mut comparison = half_bracket;
        comparison.extend([half_label, doubling]);
        fade_in_all(stage, &comparison, 0.4)?;
        let quarters: Vec<Mob> = halves
            .iter()
            .map(|t| outline(stage, t.corners, Palette::dim(), TRI_W))
            .collect();
        fade_in_all(stage, &quarters[..1], 0.5)?;
        fade_in_lagged(stage, &quarters[1..], 0.9, 0.3)?;
        line.wait_for(stage, "four times the waiting")?;
        let card = kit.text(
            stage,
            &format!(
                "{GAP_WORD} {TIMES} {} {BECOMES} {WAITING} {TIMES} {}",
                n.double, n.double_area_ratio
            ),
            PANEL_SIZE,
            Palette::ink(),
            v(PANEL_X, row(CARD_ROW)),
        )?;
        fade_in_all(stage, &[card], 0.5)?;
        pulse(stage, &fills[1..])?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::{Narrator, Script};

    use super::*;
    use crate::model::{OFFSET, PERIOD};

    /// Twice a triangle's signed area (positive counterclockwise).
    fn twice_area(c: [(f64, f64); 3]) -> f64 {
        (c[1].0 - c[0].0) * (c[2].1 - c[0].1) - (c[2].0 - c[0].0) * (c[1].1 - c[0].1)
    }

    /// Needs chapter 05's lines in script.tsv (the integration step copies
    /// them from the draft). The whole chapter runs headless at 10 fps with
    /// the scratch voice: every phrase it waits for is in its line, the
    /// numbers and the tiles match the facts, and its eight lines are said
    /// in order. It ends with `kit.finish`, whose timeline runs past the
    /// end of the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = Triangles { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6", "05_7", "05_8"]);
        let (last, at) = said_at.last().unwrap();
        let last_ends = at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        assert!(
            end > last_ends,
            "the timeline ends at {end:.2} s, before {last} ends at {last_ends:.2} s"
        );
    }

    /// The traced wait is the model's at every quarter minute of the
    /// stretch: falling a minute a minute inside each gap, jumping to the
    /// whole next gap at the red bus, nothing at the stretch's end.
    #[test]
    fn the_wait_falls_steadily_and_jumps_at_each_bus() {
        for quarter in 0..4 * PERIOD {
            let t = Q::new(quarter, 4);
            assert!(
                (wait_in(t.to_f64()) - model::wait_at(OFFSET, t).to_f64()).abs() < 1e-12,
                "{t}"
            );
        }
        assert_eq!(wait_in(PERIOD as f64), 0.0);
        let (short, long) = facts::short_and_long();
        assert_eq!(wait_in(0.0), short as f64);
        assert_eq!(wait_in(OFFSET as f64), long as f64);
    }

    /// The finished trace is the sawtooth: down the short gap, up the jump,
    /// down the long gap. Mid-ride it ends where you are, at your wait, and
    /// every edge is a slant falling a minute a minute or a jump at a bus.
    #[test]
    fn the_trace_is_the_sawtooth() {
        let red = twelve_minutes::gaps()[0].to as f64;
        // 05_2's short ride stops a hair before the red bus: still on the
        // short gap's slant, the wait all but nothing, the jump left for
        // the long ride; at the bus itself the jump is drawn
        let c = trace(red - JUST_BEFORE);
        assert_eq!(c.len(), 2);
        assert!((c[1].0 - red).abs() < 1e-5 && c[1].1 > 0.0 && c[1].1 < 1e-5);
        assert!(wait_in(red - JUST_BEFORE) < 1e-5);
        assert_eq!(trace(red).len(), 3);
        let (short, long) = facts::short_and_long();
        let (s, l, o, p) = (short as f64, long as f64, OFFSET as f64, PERIOD as f64);
        assert_eq!(trace(p), vec![(0.0, s), (o, 0.0), (o, l), (p, 0.0)]);
        assert_eq!(trace(0.0).len(), 1);
        for hundredth in 1..1200 {
            let t = f64::from(hundredth) / 100.0;
            let c = trace(t);
            let &(x, w) = c.last().unwrap();
            assert!((x - t).abs() < 1e-12 && (w - wait_in(t)).abs() < 1e-12, "{t}");
            for e in c.windows(2) {
                let (dx, dw) = (e[1].0 - e[0].0, e[1].1 - e[0].1);
                let slant = (dx + dw).abs() < 1e-12 && dx > 0.0;
                let jump = dx == 0.0 && twelve_minutes::buses().contains(&(e[0].0 as i64));
                assert!(slant || jump, "{t}: {e:?}");
            }
        }
    }

    /// Each gap's triangle is as wide as its gap and as tall, its area is
    /// the model's, and it holds exactly its passengers' waiting.
    #[test]
    fn each_triangle_holds_its_gaps_waiting() {
        let people = twelve_minutes::passengers().unwrap();
        for (g, gap) in twelve_minutes::gaps().iter().enumerate() {
            let c = triangle(gap);
            let m = gap.minutes() as f64;
            assert_eq!((c[2].0 - c[0].0, c[1].1 - c[0].1), (m, m));
            let area = twice_area(c).abs() / 2.0;
            assert!((area - model::triangle_area(gap.minutes()).to_f64()).abs() < 1e-12);
            let waits: f64 = in_gap(&people, g).iter().map(|&i| people[i].wait.to_f64()).sum();
            assert!((waits - area).abs() < 1e-12, "the {m}-minute gap");
        }
    }

    /// The corner a widened bar puts above its gap's slant is the corner it
    /// leaves empty under it: two right triangles of the same area, each
    /// with its long side on the slant, so a bar holds exactly the
    /// triangle's slice over its minute.
    #[test]
    fn the_corners_a_bar_adds_are_the_corners_it_leaves() {
        let people = twelve_minutes::passengers().unwrap();
        let gaps = twelve_minutes::gaps();
        for p in &people {
            let slant = |t: f64| gaps[p.gap].to as f64 - t;
            let [above, under] = corners(p);
            assert!((twice_area(above).abs() - twice_area(under).abs()).abs() < 1e-12);
            assert!(twice_area(above).abs() > 0.0);
            // above: two corners on the slant, the third over it
            assert!((above[0].1 - slant(above[0].0)).abs() < 1e-12);
            assert!((above[2].1 - slant(above[2].0)).abs() < 1e-12);
            assert!(above[1].1 > slant(above[1].0));
            // under: two corners on the slant, the third under it
            assert!((under[1].1 - slant(under[1].0)).abs() < 1e-12);
            assert!((under[2].1 - slant(under[2].0)).abs() < 1e-12);
            assert!(under[0].1 < slant(under[0].0));
            // the bar over its minute, and the slant's slice over it
            let h = 1.0 / (2 * PER_MINUTE) as f64;
            let t = p.t.to_f64();
            let slice = (slant(t - h) + slant(t + h)) / 2.0 * 2.0 * h;
            assert!((p.wait.to_f64() * 2.0 * h - slice).abs() < 1e-12, "{}", p.t);
        }
    }

    /// The tiles fill the long triangle exactly: the model's counts and the
    /// facts', each the small triangle's area, all inside, no two in one
    /// place, the right angle first; an upside-down tile is the small
    /// triangle turned half round. The half-size triangles fill it four
    /// times.
    #[test]
    fn the_tiles_fill_the_long_triangle() {
        let f = facts::facts().unwrap();
        let n = twelve_minutes::numbers(&f).unwrap();
        let gaps = twelve_minutes::gaps();
        let (short, long) = (gaps[0], gaps[1]);
        let pieces = tiles(&long, short.minutes()).unwrap();
        let (up, down) = model::tiling((long.minutes() / short.minutes()) as u64);
        let upright = pieces.iter().filter(|t| t.upright).count() as u64;
        assert_eq!((upright, pieces.len() as u64 - upright), (up, down));
        assert_eq!([up.to_string(), down.to_string()], n.tiles);
        let small = twice_area(triangle(&short)).abs();
        let mut centroids: Vec<(i64, i64)> = Vec::new();
        for t in &pieces {
            assert!((twice_area(t.corners).abs() - small).abs() < 1e-12);
            let [r, a, b] = t.corners;
            let dot = (a.0 - r.0) * (b.0 - r.0) + (a.1 - r.1) * (b.1 - r.1);
            assert!(dot.abs() < 1e-12, "the right angle comes first: {t:?}");
            for (x, w) in t.corners {
                assert!(x >= long.from as f64 - 1e-12 && w >= -1e-12);
                assert!(x - long.from as f64 + w <= long.minutes() as f64 + 1e-12, "{t:?}");
            }
            if !t.upright {
                // the same corners, listed in the fold's order
                let turned = triangle(&short).map(|(x, w)| (-x, -w));
                let shift = (t.corners[0].0 - turned[0].0, t.corners[0].1 - turned[0].1);
                let by_place = |c: [(f64, f64); 3]| {
                    let mut c = c.to_vec();
                    c.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    c
                };
                let moved = turned.map(|(x, w)| (x + shift.0, w + shift.1));
                assert_eq!(
                    by_place(moved),
                    by_place(t.corners),
                    "an upside-down tile is the small one turned"
                );
            }
            let c = shrunk(t.corners, 0.0)[0];
            centroids.push(((c.0 * 3.0).round() as i64, (c.1 * 3.0).round() as i64));
        }
        let all = centroids.len();
        centroids.sort_unstable();
        centroids.dedup();
        assert_eq!(centroids.len(), all, "two tiles share a place");
        let total: f64 = pieces.iter().map(|t| twice_area(t.corners).abs()).sum();
        assert!((total - twice_area(triangle(&long)).abs()).abs() < 1e-9);
        let k: i64 = n.double.parse().unwrap();
        let halves = tiles(&long, long.minutes() / k).unwrap();
        assert_eq!(halves.len().to_string(), n.double_area_ratio);
        assert!(tiles(&long, 3).is_err());
    }

    /// A shrunk tile keeps its centroid and its corners' order.
    #[test]
    fn a_shrunk_tile_keeps_its_place() {
        let c = [(2.0, 0.0), (2.0, 2.0), (4.0, 0.0)];
        let s = shrunk(c, TILE_K);
        let centre = |c: [(f64, f64); 3]| ((c[0].0 + c[1].0 + c[2].0) / 3.0, (c[0].1 + c[1].1 + c[2].1) / 3.0);
        let (a, b) = (centre(c), centre(s));
        assert!((a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);
        assert!((twice_area(s) - TILE_K * TILE_K * twice_area(c)).abs() < 1e-12);
    }

    /// The readout rounds to whole seconds and writes minutes as a clock.
    #[test]
    fn the_readout_writes_the_wait_as_a_clock() {
        assert_eq!(readout(2.0).unwrap(), format!("{WAIT_WORD} 2:00"));
        assert_eq!(readout(9.5).unwrap(), format!("{WAIT_WORD} 9:30"));
        assert_eq!(readout(13.0 / 3.0).unwrap(), format!("{WAIT_WORD} 4:20"));
        assert_eq!(readout(0.0).unwrap(), format!("{WAIT_WORD} 0:00"));
        assert_eq!(readout(1.0 / 120.0).unwrap(), format!("{WAIT_WORD} 0:01"));
        assert!(readout(-1.0).is_err());
    }

    /// The panel, the wait's axis and names, the readout, the arrow and its
    /// label stay apart and on screen: the panel right of the plot, the
    /// axis labels left of it, the arrow under the brackets' lengths.
    /// Widths are estimated, not measured: 0.0045 units a point a
    /// character, a line 0.6 of its size tall.
    #[test]
    fn the_picture_stays_apart_and_on_screen() {
        let f = facts::facts().unwrap();
        let n = twelve_minutes::numbers(&f).unwrap();
        let w = |s: &str, pt: f64| s.chars().count() as f64 * pt * 0.0045;
        let h = |pt: f64| pt * 0.0045 / 0.55 * 0.6;
        type Box4 = (f64, f64, f64, f64);
        let centred = |s: &str, pt: f64, at: Vec3| -> Box4 {
            (
                at[0] - w(s, pt) / 2.0,
                at[0] + w(s, pt) / 2.0,
                at[1] - h(pt),
                at[1] + h(pt),
            )
        };
        let left = |s: &str, pt: f64, at: Vec3| -> Box4 { (at[0], at[0] + w(s, pt), at[1] - h(pt), at[1] + h(pt)) };
        let apart = |a: Box4, b: Box4| a.1 < b.0 || b.1 < a.0 || a.3 < b.2 || b.3 < a.2;
        let on_screen = |a: Box4| a.0 > -7.1 && a.1 < 7.1 && a.2 > -4.0 && a.3 < 4.0;
        let fr = frame();
        let p = |i: usize| v(PANEL_X, row(i));
        let g = &n.gap_minutes;
        let readout_box = left(&readout(9.5).unwrap(), PANEL_SIZE, v(READOUT_X, row(0)));
        let ring = (
            THINK_AT[0] - 0.62,
            THINK_AT[0] + 0.62,
            THINK_AT[1] - 0.62,
            THINK_AT[1] + 0.62,
        );
        let note = centred("pause for more time", 18.0, v(THINK_AT[0], THINK_AT[1] - 0.62 - 0.3));
        let card = format!(
            "{GAP_WORD} {TIMES} {} {BECOMES} {WAITING} {TIMES} {}",
            n.double, n.double_area_ratio
        );
        // up together at some moment: the areas with the question and its
        // ring, then with the answer, the count and the card
        let areas = [
            centred(
                &format!(
                    "{} {TIMES} {} {TIMES} {} = {} {AREA_UNIT}",
                    n.half, g[0], g[0], n.areas[0]
                ),
                PANEL_SIZE,
                p(AREA_ROWS[0]),
            ),
            centred(
                &format!(
                    "{} {TIMES} {} {TIMES} {} = {} {AREA_UNIT}",
                    n.half, g[1], g[1], n.areas[1]
                ),
                PANEL_SIZE,
                p(AREA_ROWS[1]),
            ),
            centred(
                &format!("{} + {} = {} {AREA_UNIT}", n.areas[0], n.areas[1], n.total_area),
                PANEL_SIZE,
                p(AREA_ROWS[2]),
            ),
        ];
        let asking = [centred(ASK, PANEL_SIZE, p(ASK_ROW)), ring, note];
        let answering = [
            centred(&format!("{TIMES} {}", n.area_ratio), BIG_SIZE, p(ASK_ROW)),
            centred(
                &format!("{} + {} = {}", n.tiles[0], n.tiles[1], n.area_ratio),
                PANEL_SIZE,
                p(COUNT_ROW),
            ),
            centred(&card, PANEL_SIZE, p(CARD_ROW)),
        ];
        for group in [
            areas.iter().chain(&asking).copied().collect::<Vec<_>>(),
            areas.iter().chain(&answering).copied().collect::<Vec<_>>(),
        ] {
            for (i, &a) in group.iter().enumerate() {
                assert!(on_screen(a) && a.0 > fr.right(), "{a:?}");
                for &b in &group[i + 1..] {
                    assert!(apart(a, b), "{a:?} {b:?}");
                }
            }
        }
        // the readout, alone in the panel during the ride
        assert!(on_screen(readout_box) && readout_box.0 > fr.right());
        // the wait's axis: its labels left of the line, its name over it
        let top = at(0.0, fr.y.1)[1] + Y_OVERHANG;
        let name = centred(WAIT_NAME, SMALL, v(ORIGIN_X, top + WAIT_NAME_DY));
        assert!(on_screen(name) && name.2 > top);
        for label in g {
            let m: f64 = label.parse().unwrap();
            let b = centred(label, SMALL, v(ORIGIN_X - Y_LABEL_DX, at(0.0, m)[1]));
            assert!(on_screen(b) && b.1 < ORIGIN_X - Y_TICK_W, "{label}");
        }
        // the moment's name right of the last bus's marker (half width
        // 0.07), on the line's row above the clock labels
        let moment = left(MOMENT_NAME, SMALL, v(fr.right() + MOMENT_NAME_DX, AXIS_Y));
        assert!(on_screen(moment) && moment.0 > fr.right() + 0.07);
        // the arrow and its label under the brackets' lengths (20 pt, 0.82
        // under the line), and on screen
        let gaps = twelve_minutes::gaps();
        let mid = |g: &Gap| at((g.from + g.to) as f64 / 2.0, 0.0)[0];
        let (a, b) = (mid(&gaps[0]) + ARROW_INSET, mid(&gaps[1]) - ARROW_INSET);
        assert!(a < b);
        let ratio = centred(
            &format!("{TIMES} {}", n.width_ratio),
            RATIO_SIZE,
            v((a + b) / 2.0, RATIO_Y),
        );
        assert!(on_screen(ratio));
        let lengths_bottom = AXIS_Y - 0.82 - h(20.0);
        assert!(ARROW_Y < lengths_bottom && ratio.3 < ARROW_Y);
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            WAIT_NAME,
            MOMENT_NAME,
            WAIT_WORD,
            TIMES,
            ASK,
            GAP_WORD,
            WAITING,
            BECOMES,
            AREA_UNIT,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
