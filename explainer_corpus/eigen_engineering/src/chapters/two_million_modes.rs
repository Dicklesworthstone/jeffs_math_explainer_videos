//! Chapter 10, two million modes: a room has three directions, so a mode
//! counts humps along the length, the width and the height, and one formula
//! gives every tone of a box with hard walls, drawn as the pressure on the
//! floor and on one wall swinging between squeezed and thinned out (10_1);
//! a box the size of a concert hall, grown from a bathroom until the bathroom
//! is a speck beside it (10_2); the question, how many modes below a thousand
//! hertz (10_3); the count, swept out as a quarter circle over the floor's
//! dots to 2,246 at 100 Hz and 1,964,040 at 1,000 Hz (10_4); a single mode
//! blurred about a hertz wide by a decay of 1.9 s (10_5); and the modes of the
//! hall crowding together until they merge above about 20 Hz, the Schroeder
//! frequency (10_6).
//!
//! Every number is a fact: the hall's sides, volume and lowest tone
//! `K-H1.*`, its counts `K-H2.count.*`, its blur `K-H2.mode_width_hz` and
//! decay `K-H1.t60_s`, the crossover `K-H3.schroeder_hz`; the model
//! (`model::rooms`) counts them again exactly, in whole numbers.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::props::{RowStrips, lower_third, predict_pause};
use explainer_kit::{Kit, Palette, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    chip, fact, fade_in_all, fade_out_all, faded_line, flat, ground, group, own, pause, say, show, squeezed, stretched,
    tween_linear, wait_until,
};
use super::sound;
use super::the_air_in_a_room::{GLOW_HZ, PITCH_HI, PITCH_LO};
use crate::model::rooms::{BATHROOM, HALL, HALL_T60_S, SOUND_M_PER_S};

pub struct TwoMillionModes {
    pub kit: Kit,
}

// ------------------------------------------------------- the maps (10_1)

const COLS: usize = 24;
const PLAN_ROWS: usize = 16;
const SIDE_ROWS: usize = 12;
const CELL: f64 = 0.21;
const MAP_TOP: f64 = 3.58;
const PLAN_LEFT: f64 = -6.42;
const SIDE_LEFT: f64 = 1.38;
/// Seconds for one full swing of the pressure.
const SWING_S: f64 = 2.4;
/// The counts the maps are shown at, one change at a time.
const STEPS: [[u32; 3]; 4] = [[1, 0, 0], [2, 0, 0], [2, 1, 0], [2, 1, 1]];

/// The pressure of mode `n` at the point `(u, w, z)` of the box, each from 0
/// to 1 along its side, at the top of a swing: a cosine in each direction,
/// largest at the walls and corners and zero half way between.
pub fn pressure(n: [u32; 3], u: f64, w: f64, z: f64) -> f64 {
    let pi = std::f64::consts::PI;
    (f64::from(n[0]) * pi * u).cos() * (f64::from(n[1]) * pi * w).cos() * (f64::from(n[2]) * pi * z).cos()
}

/// What a pressure from -1 to 1 looks like: squeezed magenta above zero,
/// thinned-out teal below, the background at rest.
pub fn paint(p: f64) -> Srgb {
    let (from, to) = (ground(), if p >= 0.0 { squeezed() } else { stretched() });
    let k = p.abs().clamp(0.0, 1.0);
    Srgb {
        r: from.r * (1.0 - k) + to.r * k,
        g: from.g * (1.0 - k) + to.g * k,
        b: from.b * (1.0 - k) + to.b * k,
    }
}

/// The floor seen from above, one colour per cell: rows from the far wall,
/// the pressure at the floor (`z = 0`) at `swing` of the way through a swing
/// (its cosine).
fn plan_colours(n: [u32; 3], swing: f64) -> Vec<Vec<Srgb>> {
    (0..PLAN_ROWS)
        .map(|j| {
            (0..COLS)
                .map(|i| {
                    let (u, w) = ((i as f64 + 0.5) / COLS as f64, (j as f64 + 0.5) / PLAN_ROWS as f64);
                    paint(pressure(n, u, w, 0.0) * swing)
                })
                .collect()
        })
        .collect()
}

/// One long wall seen from the side (`w = 0`): rows from the ceiling.
fn side_colours(n: [u32; 3], swing: f64) -> Vec<Vec<Srgb>> {
    (0..SIDE_ROWS)
        .map(|k| {
            (0..COLS)
                .map(|i| {
                    let (u, z) = (
                        (i as f64 + 0.5) / COLS as f64,
                        1.0 - (k as f64 + 0.5) / SIDE_ROWS as f64,
                    );
                    paint(pressure(n, u, 0.0, z) * swing)
                })
                .collect()
        })
        .collect()
}

// ------------------------------------------------------- the hall (10_2)

const HALL_AT: Vec3 = [1.0, 1.3, 0.0];
/// Scene units to a metre in the picture of the hall.
const HALL_SCALE: f64 = 0.2;

/// The box's sides in metres after `s` of its growth from the bathroom
/// (0) to the hall (1), and its volume.
pub fn grown(s: f64) -> ([f64; 3], f64) {
    let (small, big) = (BATHROOM.sides_m(), HALL.sides_m());
    let sides = [0, 1, 2].map(|i| small[i] + (big[i] - small[i]) * s);
    (sides, sides.iter().product())
}

/// `n` with a comma between thousands.
pub fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

// ------------------------------------------------------ the count (10_4)

const ORIGIN_AT: Vec3 = [-5.0, -1.55, 0.0];
/// The quarter circle's radius on screen, for the largest frequency shown.
const REACH: f64 = 4.2;

/// The counts the counter passes: exact for every hertz to 100, then every
/// 10 Hz to 1,000 (the count of modes at or below that frequency).
pub fn count_table() -> Vec<(f64, u64)> {
    let mut table = Vec::new();
    for f in 0..=100u64 {
        table.push((f as f64, if f == 0 { 0 } else { HALL.count_below(f) }));
    }
    for f in (110..=1000u64).step_by(10) {
        table.push((f as f64, HALL.count_below(f)));
    }
    table
}

/// The count at `hz` from `table`, between its exact points in a straight line.
pub fn count_at(table: &[(f64, u64)], hz: f64) -> f64 {
    let i = table.partition_point(|&(f, _)| f <= hz);
    if i == 0 {
        return 0.0;
    }
    if i >= table.len() {
        return table[table.len() - 1].1 as f64;
    }
    let ((f0, c0), (f1, c1)) = (table[i - 1], table[i]);
    c0 as f64 + (c1 as f64 - c0 as f64) * (hz - f0) / (f1 - f0)
}

/// The modes of the hall that sit on its floor (no humps in the height),
/// with their two counts and their tone, up to `limit` Hz.
pub fn floor_modes(limit: f64) -> Vec<([u32; 2], f64)> {
    let mut out = Vec::new();
    let [lx, ly, _] = HALL.sides_m();
    let top = (2.0 * lx.max(ly) * limit / SOUND_M_PER_S).ceil() as u32 + 1;
    for nx in 0..=top {
        for ny in 0..=top {
            if (nx, ny) == (0, 0) {
                continue;
            }
            let hz = HALL.mode_hz([nx, ny, 0]);
            if hz <= limit {
                out.push(([nx, ny], hz));
            }
        }
    }
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out
}

/// A quarter circle's vertices: from the origin out along the x axis, round
/// to the y axis, and back, for radius `r` on screen.
fn sector_vertices(r: f64) -> Vec<Vec3> {
    let r = r.max(1e-3);
    let mut pts = vec![v(ORIGIN_AT[0], ORIGIN_AT[1])];
    for k in 0..=24 {
        let a = FRAC_PI_2 * k as f64 / 24.0;
        pts.push(v(ORIGIN_AT[0] + r * a.cos(), ORIGIN_AT[1] + r * a.sin()));
    }
    pts.push(pts[0]);
    pts
}

// ----------------------------------------------- the blur and the band

/// How much a resonance of half-power width `width` Hz responds at `df` Hz
/// from its tone (a Lorentzian: 1 at the tone, one half at half the width).
pub fn response(df: f64, width: f64) -> f64 {
    1.0 / (1.0 + (2.0 * df / width).powi(2))
}

/// The width in hertz a decay of `t60` seconds gives each mode: 2.2 / T.
pub fn blur_hz(t60: f64) -> f64 {
    2.2 / t60
}

/// The sampled sum of every mode's response at `hz`, squashed to 0..1
/// (`s / (s + 2)`) so that sparse modes stand as peaks and a crowd fills in
/// as a band.
pub fn crowd(tones: &[f64], hz: f64, width: f64) -> f64 {
    let sum: f64 = tones.iter().map(|&t| response(hz - t, width)).sum();
    sum / (sum + 2.0)
}

const SPEC_X0: f64 = -5.8;
const SPEC_X1: f64 = 5.8;
const SPEC_Y: f64 = -0.6;
const SPEC_H: f64 = 2.6;
const SPEC_HZ: f64 = 60.0;
const SPEC_STEP: f64 = 0.1;

fn spec_x(hz: f64) -> f64 {
    SPEC_X0 + (SPEC_X1 - SPEC_X0) * hz / SPEC_HZ
}

/// A vertex list as the run of points a mobject keeps (and `set_points`
/// wants): a handle at the midpoint of each pair of vertices, as `polyline`
/// builds.
fn corners(vertices: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(2 * vertices.len());
    for w in vertices.windows(2) {
        out.push(w[0]);
        out.push([
            (w[0][0] + w[1][0]) / 2.0,
            (w[0][1] + w[1][1]) / 2.0,
            (w[0][2] + w[1][2]) / 2.0,
        ]);
    }
    if let Some(&last) = vertices.last() {
        out.push(last);
    }
    out
}

/// The crowd curve from 0 Hz up to `upto` Hz, as screen vertices.
fn spectrum_points(tones: &[f64], width: f64, upto: f64) -> Vec<Vec3> {
    let n = ((upto / SPEC_STEP).round() as usize).max(1);
    (0..=n)
        .map(|k| {
            let hz = k as f64 * SPEC_STEP;
            v(spec_x(hz), SPEC_Y + SPEC_H * crowd(tones, hz, width))
        })
        .collect()
}

impl SceneConstruct for TwoMillionModes {
    fn name(&self) -> &str {
        "10_two_million_modes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();

        // ---- 10_1: three directions, a mode counts humps along each
        let line = say(&kit, stage, "10_1")?;
        let nx = stage.add_value_tracker(1.0);
        let ny = stage.add_value_tracker(0.0);
        let nz = stage.add_value_tracker(0.0);
        let swing = stage.add_value_tracker(0.0);
        let plan = RowStrips::build(stage, v(PLAN_LEFT, MAP_TOP), (CELL, CELL), &plan_colours(STEPS[0], 1.0))?;
        let side = RowStrips::build(stage, v(SIDE_LEFT, MAP_TOP), (CELL, CELL), &side_colours(STEPS[0], 1.0))?;
        let (plan_group, side_group) = (plan.group, side.group);
        let maps = stage.arena_mut().add(VMobject::new());
        stage.add_dt_updater(
            maps,
            move |s, _, dt| {
                let mut phase = s.tracker_value(swing).unwrap_or(0.0);
                if dt > 0.0 {
                    phase = (phase + TAU * dt / SWING_S) % TAU;
                    let _ = s.set_tracker_value(swing, phase);
                }
                let n = [
                    s.tracker_value(nx).unwrap_or(1.0).round() as u32,
                    s.tracker_value(ny).unwrap_or(0.0).round() as u32,
                    s.tracker_value(nz).unwrap_or(0.0).round() as u32,
                ];
                // the swing's cosine: a half swing later every squeezed cell is thinned out
                let k = phase.cos();
                plan.recolor(s, &plan_colours(n, k));
                side.recolor(s, &side_colours(n, k));
            },
            true,
        )?;
        stage.add_to_scene(maps)?;
        let counters = [("n_x", nx, 3.0), ("n_y", ny, 2.2), ("n_z", nz, 1.4)];
        let mut counter_mobs = Vec::new();
        for (name, tracker, y) in counters {
            let label = kit.tex(stage, name, 46.0, ink, &[], v(-0.45, y))?;
            counter_mobs.push(label);
            counter_mobs.push(kit.live_number(stage, tracker, 0, false, 46.0, ink, v(0.15, y), LEFT)?);
        }
        let plan_words = kit.text(stage, "the floor, seen from above", 24.0, dim, v(-3.9, 0.0))?;
        let side_words = kit.text(stage, "one long wall", 24.0, dim, v(3.9, 0.75))?;
        let least = kit.text(
            stage,
            "where the air moves least, at the walls, it is squeezed most",
            24.0,
            dim,
            v(-0.4, -0.55),
        )?;
        let formula = kit.tex(
            stage,
            r"\text{tone} = \frac{c}{2}\sqrt{\left(\frac{n_x}{L_x}\right)^2 + \left(\frac{n_y}{L_y}\right)^2 + \left(\frac{n_z}{L_z}\right)^2}",
            42.0,
            ink,
            &[],
            v(0.0, -1.45),
        )?;
        let sides = kit.tex(
            stage,
            r"L_x = \text{length},\quad L_y = \text{width},\quad L_z = \text{height}",
            32.0,
            dim,
            &[],
            v(0.0, -2.45),
        )?;
        fade_in_all(stage, &[plan_group, side_group, plan_words, side_words], 0.7)?;
        // the counters are in the scene as they are built; the labels fade in
        let labels: Vec<Mob> = counter_mobs.iter().step_by(2).copied().collect();
        fade_in_all(stage, &labels, 0.5)?;
        line.wait_for(stage, "along the length")?;
        play!(stage; tween(nx, 2.0, 0.01)?);
        line.wait_for(stage, "the width")?;
        play!(stage; tween(ny, 1.0, 0.01)?);
        line.wait_for(stage, "the height")?;
        play!(stage; tween(nz, 1.0, 0.01)?);
        fade_in_all(stage, &[least], 0.5)?;
        line.wait_for(stage, "One formula")?;
        fade_in_all(stage, &[formula], 0.7)?;
        fade_in_all(stage, &[sides], 0.5)?;
        let from = lower_third(&kit, stage, "FrankenSim:", "rectangular_cavity_modes")?;
        fade_in_all(stage, &[from], 0.5)?;
        kit.hold(stage)?;

        // ---- 10_2: a hall, grown from a bathroom
        let line = say(&kit, stage, "10_2")?;
        let mut leaving = vec![
            plan_group, side_group, plan_words, side_words, least, formula, sides, from,
        ];
        leaving.extend(counter_mobs.iter().copied());
        fade_out_all(stage, &leaving, 0.5)?;
        stage.clear_updaters(maps, true);
        stage.remove_from_scene(maps);
        let grow = stage.add_value_tracker(0.0);
        let unit = Rectangle::new().width(1.0).height(1.0).build()?.points().to_vec();
        let hall_box = stage.arena_mut().add(
            Rectangle::new()
                .width(HALL_SCALE * BATHROOM.sides_m()[0])
                .height(HALL_SCALE * BATHROOM.sides_m()[1])
                .style(faded_line(ink, 4.0, 1.0))
                .build()?
                .moved_to(v(HALL_AT[0], HALL_AT[1])),
        );
        let boxer = stage.arena_mut().add(VMobject::new());
        {
            let mut last = f64::NAN;
            stage.add_updater(
                boxer,
                move |s, _| {
                    let g = s.tracker_value(grow).unwrap_or(0.0);
                    if g.to_bits() == last.to_bits() {
                        return;
                    }
                    last = g;
                    let (sides, _) = grown(g);
                    let (w, h) = (HALL_SCALE * sides[0], HALL_SCALE * sides[1]);
                    let pts: Vec<Vec3> = unit
                        .iter()
                        .map(|p| [HALL_AT[0] + p[0] * w, HALL_AT[1] + p[1] * h, p[2]])
                        .collect();
                    let _ = s.set_points(hall_box, &pts);
                },
                true,
            )?;
        }
        stage.add_to_scene(boxer)?;
        let speck = stage.arena_mut().add(
            Rectangle::new()
                .width(HALL_SCALE * BATHROOM.sides_m()[0])
                .height(HALL_SCALE * BATHROOM.sides_m()[1])
                .style(faded_line(dim, 3.0, 1.0))
                .build()?
                .moved_to(v(-5.6, 0.6)),
        );
        let speck_words = kit.text(stage, "the bathroom", 24.0, dim, v(-5.6, 0.1))?;
        let hall_sides = HALL.sides_m();
        let width_words = kit.text(
            stage,
            &format!("{} m", show(hall_sides[0], 0)?),
            28.0,
            dim,
            v(HALL_AT[0], HALL_AT[1] - HALL_SCALE * hall_sides[1] / 2.0 - 0.3),
        )?;
        let depth_words = kit.text(
            stage,
            &format!("{} m", show(hall_sides[1], 0)?),
            28.0,
            dim,
            v(HALL_AT[0] + HALL_SCALE * hall_sides[0] / 2.0 + 0.7, HALL_AT[1]),
        )?;
        let volume = kit.text(
            stage,
            &format!(
                "{} \u{d7} {} \u{d7} {} m = {} m\u{b3}",
                show(hall_sides[0], 0)?,
                show(hall_sides[1], 0)?,
                show(hall_sides[2], 0)?,
                grouped(fact("K-H1.volume_m3")?.parse::<u64>().unwrap_or(0))
            ),
            34.0,
            ink,
            v(0.0, -2.25),
        )?;
        let model_tag = kit.text(stage, "a model of a hall", 26.0, dim, v(5.2, -2.25))?;
        let boston = kit.text(
            stage,
            &format!(
                "Boston Symphony Hall: {} m\u{b3}",
                grouped(fact("K-H1.boston_m3")?.parse::<u64>().unwrap_or(0))
            ),
            30.0,
            ink,
            v(0.0, -2.85),
        )?;
        let lowest = kit.text(
            stage,
            &format!(
                "lowest mode: {} \u{f7} (2 \u{d7} {}) = {} Hz, far below hearing",
                fact("K-R0.sound_m_per_s")?,
                show(hall_sides[0], 0)?,
                fact("K-H1.lowest_hz")?
            ),
            28.0,
            dim,
            v(0.0, -3.45),
        )?;
        fade_in_all(stage, &[hall_box, speck, speck_words], 0.6)?;
        line.wait_for(stage, "a concert hall")?;
        play!(stage; tween_linear(grow, 1.0, 2.2)?);
        fade_in_all(stage, &[width_words, depth_words, volume, model_tag], 0.6)?;
        line.wait_for(stage, "Boston's Symphony Hall")?;
        fade_in_all(stage, &[boston], 0.5)?;
        fade_in_all(stage, &[lowest], 0.5)?;
        kit.hold(stage)?;

        // ---- 10_3: how many modes below a thousand hertz?
        say(&kit, stage, "10_3")?;
        fade_out_all(
            stage,
            &[
                speck,
                speck_words,
                width_words,
                depth_words,
                volume,
                model_tag,
                boston,
                lowest,
            ],
            0.4,
        )?;
        stage.remove_from_scene(boxer);
        // the hall stays behind the card, faint
        stage.set_stroke(hall_box, None, None, Some(0.2), None, false);
        let ask = chip(&kit, stage, "How many modes below 1,000 Hz?", 40.0, own(), v(0.0, 2.3))?;
        let axis_y = -1.6;
        let axis = stage.arena_mut().add(
            polyline(vec![v(-5.8, axis_y), v(5.8, axis_y)])
                .style(faded_line(dim, 4.0, 1.0))
                .build(),
        );
        let mut axis_mobs = vec![axis];
        for (hz, x) in [(0u32, -5.8), (250, -2.9), (500, 0.0), (750, 2.9), (1000, 5.8)] {
            let tick = stage.arena_mut().add(
                polyline(vec![v(x, axis_y - 0.15), v(x, axis_y + 0.15)])
                    .style(faded_line(dim, 4.0, 1.0))
                    .build(),
            );
            axis_mobs.push(tick);
            axis_mobs.push(kit.text(stage, &grouped(u64::from(hz)), 26.0, dim, v(x, axis_y - 0.55))?);
        }
        let hz_words = kit.text(stage, "hertz", 26.0, dim, v(5.8, axis_y - 1.05))?;
        axis_mobs.push(hz_words);
        let mut ins = vec![ask];
        ins.extend(&axis_mobs);
        fade_in_all(stage, &ins, 0.6)?;
        let q = predict_pause(&kit, stage, v(0.0, 0.3))?;

        // ---- 10_4: nearly two million
        let line = say(&kit, stage, "10_4")?;
        fade_out_all(stage, &[q, ask, hall_box], 0.3)?;
        fade_out_all(stage, &axis_mobs, 0.4)?;
        let table = count_table();
        let sweep = stage.add_value_tracker(0.0);
        let reach = stage.add_value_tracker(100.0);
        let counted = stage.add_value_tracker(0.0);
        let dots_data = floor_modes(100.0);
        let template: Vec<Vec3> = Dot::new().radius(0.05).build().points().to_vec();
        let mut dots = Vec::new();
        for _ in &dots_data {
            dots.push(
                stage.arena_mut().add(
                    Dot::new()
                        .point(v(ORIGIN_AT[0], ORIGIN_AT[1]))
                        .radius(0.05)
                        .style(flat(dim, 1.0))
                        .build(),
                ),
            );
        }
        let dot_group = group(stage, &dots)?;
        let fill = stage
            .arena_mut()
            .add(Polygon::new(sector_vertices(1e-3)).style(flat(own(), 0.0)).build());
        stage.add_to_scene(fill)?;
        let rim = stage.arena_mut().add(
            polyline((0..=24).map(|k| {
                let a = FRAC_PI_2 * k as f64 / 24.0;
                v(ORIGIN_AT[0] + 1e-3 * a.cos(), ORIGIN_AT[1] + 1e-3 * a.sin())
            }))
            .style(faded_line(own(), 5.0, 1.0))
            .build(),
        );
        let x_axis = stage.arena_mut().add(
            polyline(vec![
                v(ORIGIN_AT[0], ORIGIN_AT[1]),
                v(ORIGIN_AT[0] + REACH + 0.2, ORIGIN_AT[1]),
            ])
            .style(faded_line(dim, 3.0, 1.0))
            .build(),
        );
        let y_axis = stage.arena_mut().add(
            polyline(vec![
                v(ORIGIN_AT[0], ORIGIN_AT[1]),
                v(ORIGIN_AT[0], ORIGIN_AT[1] + REACH + 0.2),
            ])
            .style(faded_line(dim, 3.0, 1.0))
            .build(),
        );
        let to_screen = move |hz: [f64; 2], axis_hz: f64| -> Vec3 {
            v(
                ORIGIN_AT[0] + REACH * hz[0] / axis_hz,
                ORIGIN_AT[1] + REACH * hz[1] / axis_hz,
            )
        };
        let [lx, ly, _] = HALL.sides_m();
        let spots: Vec<([f64; 2], f64)> = dots_data
            .iter()
            .map(|&([a, b], hz)| {
                (
                    [
                        SOUND_M_PER_S / 2.0 * f64::from(a) / lx,
                        SOUND_M_PER_S / 2.0 * f64::from(b) / ly,
                    ],
                    hz,
                )
            })
            .collect();
        let sweeper = stage.arena_mut().add(VMobject::new());
        {
            let (dots, spots, template) = (dots.clone(), spots.clone(), template.clone());
            let mut lit = vec![false; dots.len()];
            let mut last = (f64::NAN, f64::NAN);
            stage.add_updater(
                sweeper,
                move |s, _| {
                    let (f, a) = (
                        s.tracker_value(sweep).unwrap_or(0.0),
                        s.tracker_value(reach).unwrap_or(100.0),
                    );
                    if (f, a) == last {
                        return;
                    }
                    last = (f, a);
                    let _ = s.set_tracker_value(counted, count_at(&table, f));
                    let r = REACH * f / a;
                    let _ = s.set_points(
                        rim,
                        &corners(
                            &(0..=24)
                                .map(|k| {
                                    let ang = FRAC_PI_2 * f64::from(k) / 24.0;
                                    v(
                                        ORIGIN_AT[0] + r.max(1e-3) * ang.cos(),
                                        ORIGIN_AT[1] + r.max(1e-3) * ang.sin(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        ),
                    );
                    let _ = s.set_points(fill, &corners(&sector_vertices(r)));
                    // the quarter circle fills in as the dots crowd into the corner
                    let crowded = ((a - 100.0) / 300.0).clamp(0.0, 1.0);
                    s.set_fill(fill, None, Some(0.55 * crowded), None, false);
                    s.set_opacity(dot_group, 1.0 - crowded, true);
                    for (i, (&d, &(hz2, tone))) in dots.iter().zip(&spots).enumerate() {
                        let at = to_screen(hz2, a);
                        let moved: Vec<Vec3> = template.iter().map(|p| [p[0] + at[0], p[1] + at[1], p[2]]).collect();
                        let _ = s.set_points(d, &moved);
                        if !lit[i] && tone <= f {
                            lit[i] = true;
                            s.set_fill(d, Some(own()), Some(1.0), None, false);
                        }
                    }
                },
                true,
            )?;
        }
        stage.add_to_scene(sweeper)?;
        let reading = kit.live_count(stage, counted, 72.0, own(), v(3.4, 1.0))?;
        let modes_words = kit.text(stage, "modes at or below", 30.0, dim, v(3.4, 2.1))?;
        let hz_now = kit.live_number(stage, sweep, 0, false, 44.0, ink, v(2.9, 0.0), RIGHT)?;
        let hz_unit = kit.text(stage, "hertz", 30.0, dim, v(3.9, 0.0))?;
        let lo = kit.text(stage, "100 Hz", 26.0, dim, v(ORIGIN_AT[0] + REACH, ORIGIN_AT[1] - 0.35))?;
        let wide = kit.text(
            stage,
            "1,000 Hz",
            26.0,
            dim,
            v(ORIGIN_AT[0] + REACH, ORIGIN_AT[1] - 0.35),
        )?;
        fade_in_all(stage, &[dot_group, x_axis, y_axis, rim, modes_words, hz_unit, lo], 0.6)?;
        line.wait_for(stage, "Nearly two million")?;
        play!(stage; tween_linear(sweep, 100.0, 3.0)?);
        // out to a thousand hertz: the dots crowd into the corner and the
        // sweep's filled quarter circle takes over from them
        fade_out_all(stage, &[lo], 0.2)?;
        let note = lower_third(
            &kit,
            stage,
            "precisely:",
            "the count grows as the cube of the frequency, and depends on the volume, hardly on the shape",
        )?;
        fade_in_all(stage, &[wide, note], 0.4)?;
        play!(stage;
            tween_linear(reach, 1000.0, 3.2)?,
            tween_linear(sweep, 1000.0, 3.2)?,
        );
        kit.hold(stage)?;
        pause(stage, 1.2)?;

        // ---- 10_5: every mode is blurred about a hertz wide
        let line = say(&kit, stage, "10_5")?;
        let width = blur_hz(HALL_T60_S);
        fade_out_all(
            stage,
            &[
                dot_group,
                x_axis,
                y_axis,
                fill,
                rim,
                modes_words,
                hz_unit,
                wide,
                note,
                reading,
                hz_now,
            ],
            0.5,
        )?;
        stage.clear_updaters(sweeper, true);
        stage.remove_from_scene(sweeper);
        // left: sound dying away, its level in decibels falling in a straight
        // line to 60 down in T seconds; right: one mode as a resonance peak,
        // its width marked at half height
        let (dx, dy, dw, dh) = (-5.7, -1.0, 4.8, 2.6);
        let t_end = 2.4;
        let decay_pts: Vec<Vec3> = [0.0, HALL_T60_S]
            .iter()
            .map(|&t| v(dx + dw * t / t_end, dy + dh * (1.0 - t / HALL_T60_S)))
            .collect();
        let decay = stage
            .arena_mut()
            .add(polyline(decay_pts).style(faded_line(ink, 6.0, 1.0)).build());
        let decay_axis = stage.arena_mut().add(
            polyline(vec![v(dx, dy), v(dx + dw, dy)])
                .style(faded_line(dim, 3.0, 1.0))
                .build(),
        );
        let t_x = dx + dw * HALL_T60_S / t_end;
        let t_tick = stage.arena_mut().add(
            polyline(vec![v(t_x, dy - 0.15), v(t_x, dy + 0.15)])
                .style(faded_line(ink, 4.0, 1.0))
                .build(),
        );
        let decay_words = kit.text(stage, "sound dying away", 28.0, dim, v(dx + dw / 2.0, dy + dh + 0.5))?;
        let t_words = kit.text(
            stage,
            &format!("{} s", fact("K-H1.t60_s")?),
            34.0,
            ink,
            v(t_x, dy - 0.55),
        )?;
        let level_top = kit.text(stage, "level: 0", 24.0, dim, v(dx + 0.7, dy + dh + 0.15))?;
        let level_end = kit.text(stage, "to fall 60 decibels", 24.0, dim, v(t_x, dy - 1.05))?;
        let (px, py, pw, ph) = (2.9, -1.0, 0.8, 2.6);
        let peak_pts: Vec<Vec3> = (0..=120)
            .map(|k| {
                let df = -3.6 + 7.2 * f64::from(k) / 120.0;
                v(px + pw * df, py + ph * response(df, width))
            })
            .collect();
        let peak = stage
            .arena_mut()
            .add(polyline(peak_pts).style(faded_line(own(), 6.0, 1.0)).build());
        let peak_axis = stage.arena_mut().add(
            polyline(vec![v(px - 3.1, py), v(px + 3.1, py)])
                .style(faded_line(dim, 3.0, 1.0))
                .build(),
        );
        let peak_words = kit.text(stage, "one mode, on a line of tones", 28.0, dim, v(px, py + ph + 0.5))?;
        let half = py + ph / 2.0;
        let bracket = stage.arena_mut().add(
            polyline(vec![
                v(px - pw * width / 2.0, half - 0.15),
                v(px - pw * width / 2.0, half + 0.15),
                v(px - pw * width / 2.0, half),
                v(px + pw * width / 2.0, half),
                v(px + pw * width / 2.0, half + 0.15),
                v(px + pw * width / 2.0, half - 0.15),
            ])
            .style(faded_line(ink, 4.0, 1.0))
            .build(),
        );
        let width_words = kit.text(
            stage,
            &format!("about {} Hz wide", fact("K-H2.mode_width_hz")?),
            32.0,
            ink,
            v(px + 2.2, half + 0.5),
        )?;
        fade_in_all(stage, &[decay_axis, decay, decay_words, level_top], 0.7)?;
        line.wait_for(stage, "about two seconds")?;
        fade_in_all(stage, &[t_tick, t_words, level_end], 0.5)?;
        line.wait_for(stage, "no mode is one exact pitch")?;
        fade_in_all(stage, &[peak_axis, peak, peak_words], 0.7)?;
        line.wait_for(stage, "blurred about a hertz wide")?;
        fade_in_all(stage, &[bracket, width_words], 0.5)?;
        kit.hold(stage)?;

        // ---- 10_6: where the blurs crowd together they merge
        let line = say(&kit, stage, "10_6")?;
        fade_out_all(
            stage,
            &[
                decay,
                decay_axis,
                t_tick,
                decay_words,
                t_words,
                level_top,
                level_end,
                peak,
                peak_axis,
                peak_words,
                bracket,
                width_words,
            ],
            0.5,
        )?;
        let tones: Vec<f64> = HALL.modes_below(SPEC_HZ as u64).iter().map(|m| m.1).collect();
        let upto = stage.add_value_tracker(0.2);
        let spec = stage.arena_mut().add(
            polyline(spectrum_points(&tones, width, 0.2))
                .style(faded_line(own(), 4.0, 1.0))
                .build(),
        );
        let reveal = stage.arena_mut().add(VMobject::new());
        {
            let tones = tones.clone();
            let mut last = f64::NAN;
            stage.add_updater(
                reveal,
                move |s, _| {
                    let hz = s.tracker_value(upto).unwrap_or(0.2);
                    if hz.to_bits() == last.to_bits() {
                        return;
                    }
                    last = hz;
                    let _ = s.set_points(spec, &corners(&spectrum_points(&tones, width, hz)));
                },
                true,
            )?;
        }
        stage.add_to_scene(reveal)?;
        let baseline = stage.arena_mut().add(
            polyline(vec![v(SPEC_X0, SPEC_Y), v(SPEC_X1, SPEC_Y)])
                .style(faded_line(dim, 3.0, 1.0))
                .build(),
        );
        let mut scale_mobs = vec![baseline];
        for hz in [0.0, 20.0, 40.0, 60.0] {
            let tick = stage.arena_mut().add(
                polyline(vec![v(spec_x(hz), SPEC_Y - 0.12), v(spec_x(hz), SPEC_Y)])
                    .style(faded_line(dim, 3.0, 1.0))
                    .build(),
            );
            scale_mobs.push(tick);
            scale_mobs.push(kit.text(stage, &format!("{hz:.0}"), 24.0, dim, v(spec_x(hz), SPEC_Y - 0.4))?);
        }
        scale_mobs.push(kit.text(stage, "hertz", 24.0, dim, v(spec_x(60.0) - 0.4, SPEC_Y - 0.85))?);
        let crossover = HALL.schroeder_hz(HALL_T60_S);
        let band_pts: Vec<Vec3> = {
            let from = (crossover / SPEC_STEP).round() as usize;
            let upto_n = (SPEC_HZ / SPEC_STEP).round() as usize;
            (from..=upto_n)
                .map(|k| {
                    let hz = k as f64 * SPEC_STEP;
                    v(spec_x(hz), SPEC_Y + SPEC_H * crowd(&tones, hz, width))
                })
                .collect()
        };
        let band = stage
            .arena_mut()
            .add(polyline(band_pts).style(faded_line(dim, 6.0, 1.0)).build());
        let marker = stage.arena_mut().add(
            polyline(vec![
                v(spec_x(crossover), SPEC_Y),
                v(spec_x(crossover), SPEC_Y + SPEC_H + 0.3),
            ])
            .style(faded_line(ink, 3.0, 1.0))
            .build(),
        );
        let marker_words = kit.text(
            stage,
            &format!("{} Hz", fact("K-H3.schroeder_hz")?),
            40.0,
            ink,
            v(spec_x(crossover), SPEC_Y + SPEC_H + 0.7),
        )?;
        let edge_words = kit.text(
            stage,
            "the bottom edge of hearing",
            26.0,
            dim,
            v(spec_x(crossover) + 2.4, SPEC_Y + SPEC_H + 0.7),
        )?;
        let rule = lower_third(
            &kit,
            stage,
            "precisely:",
            "Schroeder frequency \u{2248} 2000 \u{d7} \u{221a}(decay time in seconds \u{f7} volume in cubic metres)",
        )?;
        let mut ins = vec![spec];
        ins.extend(&scale_mobs);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "Where three or more")?;
        play!(stage; tween_linear(upto, SPEC_HZ, 5.0)?);
        line.wait_for(stage, "The crossover")?;
        fade_in_all(stage, &[marker, marker_words], 0.5)?;
        line.wait_for(stage, "about twenty hertz")?;
        fade_in_all(stage, &[band, edge_words], 0.6)?;
        play!(stage; tween(upto, SPEC_HZ, 0.01)?);
        fade_in_all(stage, &[rule], 0.5)?;
        line.wait_end(stage)?;
        // the same hum in the hall: an even tail, no answer (the slide of
        // 09_5, 60 to 150 Hz, over so many modes that the hall is always on
        // one, and the hall's own 1.9 s decay)
        let mut modes: Vec<f64> = HALL.modes_below(PITCH_HI as u64).iter().map(|m| m.1).collect();
        modes.sort_by(f64::total_cmp);
        let wav = sound::hall_hum(PITCH_LO, PITCH_HI, &modes, GLOW_HZ, HALL_T60_S)?;
        kit.sonify(stage, sound::HALL_S, &format!("{}:10_6", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        wait_until(stage, start + sound::HALL_S)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{numbers, testing};
    use super::*;

    /// 10_1: the mode with one hump along the length is most squeezed at
    /// one end wall and most thinned out at the other, and at rest in the
    /// middle; the walls and corners are where the swing is largest.
    #[test]
    fn pressure_is_largest_at_the_walls_and_zero_between() {
        assert_eq!(pressure([1, 0, 0], 0.0, 0.5, 0.5), 1.0);
        assert_eq!(pressure([1, 0, 0], 1.0, 0.5, 0.5), -1.0);
        assert!(pressure([1, 0, 0], 0.5, 0.3, 0.7).abs() < 1e-12);
        // a corner of a mode with a hump along each side
        assert!((pressure([1, 1, 1], 0.0, 0.0, 0.0) - 1.0).abs() < 1e-12);
        assert!((pressure([1, 1, 1], 1.0, 0.0, 0.0) + 1.0).abs() < 1e-12);
        // the colours: rest is the background, squeezed and thinned out are the roles
        assert_eq!(paint(0.0), ground());
        assert_eq!(paint(1.0), squeezed());
        assert_eq!(paint(-1.0), stretched());
        // the shown steps add one count at a time
        for w in STEPS.windows(2) {
            let changed = (0..3).filter(|&i| w[0][i] != w[1][i]).count();
            assert_eq!(changed, 1, "{w:?}");
        }
        // the maps have the shape the strips are built with
        assert_eq!(plan_colours(STEPS[0], 1.0).len(), PLAN_ROWS);
        assert_eq!(side_colours(STEPS[0], 1.0).len(), SIDE_ROWS);
        assert!(plan_colours(STEPS[3], 0.3).iter().all(|r| r.len() == COLS));
    }

    /// 10_2: the grown box is a bathroom at the start and the hall at the
    /// end, and its volume the fact's.
    #[test]
    fn the_box_grows_from_the_bathroom_to_the_hall() {
        let (sides, volume) = grown(0.0);
        assert_eq!(sides, BATHROOM.sides_m());
        assert!((volume - BATHROOM.volume_m3()).abs() < 1e-9);
        let (sides, volume) = grown(1.0);
        assert_eq!(sides, HALL.sides_m());
        assert_eq!(grouped(volume.round() as u64), "18,630");
        assert_eq!(volume.round().to_string(), fact("K-H1.volume_m3").unwrap());
        let mut last = 0.0;
        for k in 0..=10 {
            let v = grown(f64::from(k) / 10.0).1;
            assert!(v >= last);
            last = v;
        }
        assert_eq!(fact("K-H1.boston_m3").unwrap(), "18750");
        assert_eq!(grouped(1_964_040), "1,964,040");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        let sides = numbers("K-H1.sides_m").unwrap();
        assert_eq!(sides, HALL.sides_m());
    }

    /// 10_4: the counter reads exact counts at the frequencies it passes,
    /// the facts at 20, 100 and 1,000 Hz, and the floor's dots are modes of
    /// the hall with no humps in the height.
    #[test]
    fn the_counter_passes_the_exact_counts_and_the_dots_are_modes() {
        let table = count_table();
        let at = |hz: f64| count_at(&table, hz);
        assert_eq!(at(20.0).to_string(), fact("K-H2.count.20").unwrap());
        assert_eq!(at(100.0).to_string(), fact("K-H2.count.100").unwrap());
        assert_eq!(at(1000.0).to_string(), fact("K-H2.count.1000").unwrap());
        assert_eq!(at(0.0), 0.0);
        assert!(at(5000.0) >= at(1000.0));
        // between two exact points, a straight line, never going down
        let mut last = 0.0;
        for k in 0..=1000 {
            let c = at(f64::from(k));
            assert!(c >= last, "{k}");
            last = c;
        }
        let dots = floor_modes(100.0);
        assert!(!dots.is_empty() && dots.len() < 400, "{}", dots.len());
        for &([a, b], hz) in &dots {
            assert!(hz <= 100.0);
            assert!((hz - HALL.mode_hz([a, b, 0])).abs() < 1e-12);
        }
        assert!(dots.windows(2).all(|w| w[0].1 <= w[1].1));
        // the lowest is the length's first: 343 / 90 = 3.8 Hz
        assert_eq!(show(dots[0].1, 1).unwrap(), fact("K-H1.lowest_hz").unwrap());
        // the sector is closed: the origin, the arc, the origin
        let s = sector_vertices(2.0);
        assert_eq!(s.first(), s.last());
        assert!((s[1][0] - (ORIGIN_AT[0] + 2.0)).abs() < 1e-9);
    }

    /// 10_5 and 10_6: a decay of 1.9 s blurs each mode to 1.2 Hz; below the
    /// crossover the modes stand as separate peaks and above it they fill in
    /// as a band; the crossover is the fact's 20 Hz.
    #[test]
    fn the_blur_is_one_point_two_hertz_and_the_crowd_merges_near_twenty() {
        assert_eq!(
            show(blur_hz(HALL_T60_S), 1).unwrap(),
            fact("K-H2.mode_width_hz").unwrap()
        );
        assert!((response(0.0, 1.2) - 1.0).abs() < 1e-12);
        assert!((response(0.6, 1.2) - 0.5).abs() < 1e-12, "half power at half the width");
        assert_eq!(
            show(HALL.schroeder_hz(HALL_T60_S), 0).unwrap(),
            fact("K-H3.schroeder_hz").unwrap()
        );
        let tones: Vec<f64> = HALL.modes_below(60).iter().map(|m| m.1).collect();
        assert_eq!(tones.len() as u64, HALL.count_below(60));
        let width = blur_hz(HALL_T60_S);
        // the lowest mode stands alone: a high peak with a deep valley after it
        let first = tones[0];
        assert!(crowd(&tones, first, width) > 0.3);
        let valley = (first * 10.0).ceil() / 10.0 + 1.5;
        assert!(crowd(&tones, valley.min(tones[1] - 0.6), width) < crowd(&tones, first, width));
        // up in the crowd every sample is nearly full: a band, not peaks
        let low = (500..=600)
            .map(|k| crowd(&tones, f64::from(k) / 10.0, width))
            .fold(f64::INFINITY, f64::min);
        assert!(low > 0.85, "{low}");
        // below the crossover the deepest valley is much deeper
        let sparse = (0..=100)
            .map(|k| crowd(&tones, f64::from(k) / 10.0, width))
            .fold(f64::INFINITY, f64::min);
        assert!(sparse < 0.2, "{sparse}");
        // every sample is inside the picture's band
        let pts = spectrum_points(&tones, width, SPEC_HZ);
        assert!(pts.iter().all(|p| p[0] >= SPEC_X0 - 1e-9 && p[0] <= SPEC_X1 + 1e-9));
        assert!(
            pts.iter()
                .all(|p| p[1] >= SPEC_Y - 1e-9 && p[1] <= SPEC_Y + SPEC_H + 1e-9)
        );
        // a vertex list becomes a quad path: an odd run with handles at midpoints
        let quad = corners(&pts);
        assert_eq!(quad.len(), 2 * pts.len() - 1);
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("10_two_million_modes").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "n_x",
            "n_y",
            "n_z",
            "where the air moves least, at the walls, it is squeezed most",
            "FrankenSim:",
            "rectangular_cavity_modes",
            "45 \u{d7} 23 \u{d7} 18 m = 18,630 m\u{b3}",
            "Boston Symphony Hall: 18,750 m\u{b3}",
            "lowest mode: 343 \u{f7} (2 \u{d7} 45) = 3.8 Hz, far below hearing",
            "How many modes below 1,000 Hz?",
            "1.9 s",
            "about 1.2 Hz wide",
            "20 Hz",
            "the bottom edge of hearing",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never sets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 110.0, "{} s", ran.seconds);
    }
}
