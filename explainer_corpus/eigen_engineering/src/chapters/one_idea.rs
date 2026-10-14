//! Chapter 14, one idea: the split of chapter 04 run forwards on the carts'
//! plane, a move cut along the gold lines, each piece multiplied by its rung
//! and the pieces added, the letters of K = QΛQᵀ arriving under the steps
//! (14_1); why symmetric tables always have enough own patterns, and what the
//! economy's table gives instead (14_3); where the idea stops, a question
//! (14_4) answered by a quarter turn, steel bent past yield and a factory's
//! limits (14_5); the footbridge reopened, its engineers had known how it
//! would move (14_6); the three questions (14_7); and the receipts (14_8).
//!
//! Every number is a fact or the model's: the carts' table `K-A1.table`,
//! their shares `K-A3.*` and rungs `K-A2.eigenvalues`, the economy's own
//! directions `K-E2.*`, the edges `K-X1.*` and `K-X2.*`, the factory
//! `K-L1.*` and `K-L2.*`, the yield `K-T1.yield_mpa`, the bridge `K-B2.*`.

use explainer_kit::props::{lower_third, receipts_card};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::an_economy::{angle_readout, degree_sign, hatched_quarter, table_entries, table_of};
use super::arrows::{ArrowTest, PlaneArrow, band_ends, own_lines, unit_at};
use super::draw::{
    BAND_UP, MatrixGrid, Plane, arrow_points, corners, cross_fade, fact, fade_in_all, fade_out_all, faded_line, fail,
    flat, fraction_tex, fraction_value, good, items, matrix, minus, moves, now, numbers, own, panel, pause,
    predict_hold, pushes, run, say, tween_all, tween_linear, wait_until, warn, whole,
};

pub struct OneIdea {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The carts' plane of 14_1: left cart's move across, right cart's up.
const CARTS: Plane = Plane {
    centre: [-3.4, 0.2, 0.0],
    unit: 1.0,
};
const CARTS_REACH: (f64, f64) = (2.4, 2.0);
/// Where "the move" begins (its left edge), plane units: past the blue
/// arrow's tip at (1, 0), a little above the axis.
const MOVE_NAME_AT: (f64, f64) = (1.25, 0.3);
/// How far right of (and above) the orange arrow's tip the left edge of
/// "the pushes from before" stands, plane units.
const RESULT_NAME_GAP: (f64, f64) = (0.25, 0.0);
/// The right-hand column of 14_1, where the steps are written.
const STEPS_X: f64 = 3.6;
/// The two little planes of 14_3.
const MINI_A: Plane = Plane {
    centre: [-4.4, -0.55, 0.0],
    unit: 0.5,
};
const MINI_B: Plane = Plane {
    centre: [-0.6, -0.55, 0.0],
    unit: 0.5,
};
const MINI_REACH: (f64, f64) = (2.0, 1.5);
/// The quarter turn's plane of 14_5, and the shear's inset.
const TURN: Plane = Plane {
    centre: [-1.9, 0.2, 0.0],
    unit: 0.85,
};
const TURN_REACH: (f64, f64) = (3.2, 2.6);
const SHEAR: Plane = Plane {
    centre: [4.6, 1.3, 0.0],
    unit: 0.45,
};
const SHEAR_REACH: (f64, f64) = (2.0, 1.6);
/// Where the angle readout's number ends (its right edge) in 14_5; the
/// degree sign stands against it.
const READOUT_AT: Vec3 = [5.0, -0.5, 0.0];
/// The factory's plane: bicycles across, scooters up, `unit` scene units a
/// unit, the window `WINDOW` units square.
const FACTORY: Plane = Plane {
    centre: [-4.9, -2.0, 0.0],
    unit: 0.075,
};
const WINDOW: f64 = 70.0;
/// The gauge and the plot of steel past its yield (14_5).
const PLOT_LEFT: f64 = -5.4;
const PLOT_BOTTOM: f64 = -2.1;
const PLOT_W: f64 = 5.6;
const PLOT_H: f64 = 4.8;
/// Where yield sits on the plot, as a share of its height (and how far
/// past it the picture goes).
const YIELD_AT: f64 = 0.55;
const PAST: f64 = 0.35;
/// The bridge's deck: its y, its half length and half width.
const DECK_Y: f64 = 1.7;
const DECK_HALF: f64 = 5.6;
const DECK_WIDTH: f64 = 0.4;
/// How far its sideways pattern swings at the most (scene units), and the
/// pistons beside the deck.
const SWAY: f64 = 0.28;
/// The gold curve stays on the screen, clear of the texts below, at full
/// swing.
const _: () = assert!(DECK_Y + SWAY < 3.0 && DECK_Y - SWAY > 0.0);
const DAMPERS: usize = 9;
/// How many segments the gold curve is drawn in.
const CURVE_STEPS: usize = 48;
/// The breath of 14_7 as strokes: each takes this many seconds to reach this
/// swing (+1 one way, -1 the other, 0 at rest).
const BREATH: [(f64, f64); 3] = [(1.4, 1.0), (2.8, -1.0), (1.4, 0.0)];
/// People on the deck.
const WALKERS: usize = 14;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// ---------------------------------------------------------------- helpers

/// An ISO date (`2002-02-22`) as it is said: `22 February 2002`.
fn long_date(iso: &str) -> fmn::Result<String> {
    let parts: Vec<&str> = iso.split('-').collect();
    let bad = || fail(format!("{iso:?} is not a date written year-month-day"));
    if parts.len() != 3 {
        return Err(bad());
    }
    let month: usize = parts[1].parse().map_err(|_| bad())?;
    let day: u32 = parts[2].parse().map_err(|_| bad())?;
    let name = MONTHS.get(month.wrapping_sub(1)).ok_or_else(bad)?;
    Ok(format!("{day} {name} {}", parts[0]))
}

/// A list of fractions (`1/2,1/6`) as TeX, in order.
fn fractions(key: &str) -> fmn::Result<Vec<String>> {
    Ok(items(key)?.into_iter().map(fraction_tex).collect())
}

/// "the move" in 14_1, detached: above the first axis, just past the blue
/// arrow's tip. Nothing of the picture runs through that corner: the gold
/// arms are at 45 degrees (far above it at this distance across), and the
/// second half's arrow stays on the line x + y = 1, which is left of it at
/// this height.
fn name_the_move(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let at = CARTS.p(MOVE_NAME_AT.0, MOVE_NAME_AT.1);
    kit.text_left(stage, "the move", 24.0, moves(), at)
}

/// "the pushes from before: 2, -1" in 14_1, detached, in three short lines,
/// its left edge a little right of the orange arrow's tip: the room between
/// the plane and the steps, clear of the gold arms and of the arrows.
fn name_the_pushes(kit: &Kit, stage: &mut Stage<'_>, push: &[f64]) -> fmn::Result<Mob> {
    let at = CARTS.p(push[0] + RESULT_NAME_GAP.0, push[1] + RESULT_NAME_GAP.1);
    let words = format!(
        "the pushes\nfrom before:\n{}, {}",
        whole(push[0]),
        minus(&whole(push[1]))
    );
    kit.text_left(stage, &words, 24.0, pushes(), at)
}

/// A static arrow from `a` to `b` (plane units) on `plane`, detached.
fn arrow_on(stage: &mut Stage<'_>, plane: Plane, a: [f64; 2], b: [f64; 2], color: Srgb, width: f64, o: f64) -> Mob {
    stage.arena_mut().add(
        polyline(arrow_points(plane.p(a[0], a[1]), plane.p(b[0], b[1]), 0.24))
            .style(faded_line(color, width, o))
            .build(),
    )
}

/// Where the line `a x + b y = c` crosses the square window `[0, w]²`: its
/// two ends, if it does.
fn line_in_window(a: f64, b: f64, c: f64, w: f64) -> Option<([f64; 2], [f64; 2])> {
    let mut ends: Vec<[f64; 2]> = Vec::new();
    let mut push = |p: [f64; 2]| {
        if p[0] >= -1e-9
            && p[0] <= w + 1e-9
            && p[1] >= -1e-9
            && p[1] <= w + 1e-9
            && !ends.iter().any(|q| (q[0] - p[0]).hypot(q[1] - p[1]) < 1e-9)
        {
            ends.push(p);
        }
    };
    if b != 0.0 {
        push([0.0, c / b]);
        push([w, (c - a * w) / b]);
    }
    if a != 0.0 {
        push([c / a, 0.0]);
        push([(c - b * w) / a, w]);
    }
    (ends.len() >= 2).then(|| (ends[0], ends[1]))
}

/// The corners of a convex polygon in order round it (by angle about their
/// mean).
fn round_the_polygon(corners: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = corners.len() as f64;
    let c = [
        corners.iter().map(|p| p[0]).sum::<f64>() / n,
        corners.iter().map(|p| p[1]).sum::<f64>() / n,
    ];
    let mut sorted = corners.to_vec();
    sorted.sort_by(|p, q| {
        let (a, b) = ((p[1] - c[1]).atan2(p[0] - c[0]), (q[1] - c[1]).atan2(q[0] - c[0]));
        a.total_cmp(&b)
    });
    sorted
}

/// The gold curve's vertices when it swings by `amp` (-1 to 1): lying along
/// the deck at 0, a half wave of height `SWAY` at 1. The curve is BUILT with
/// all of them (straight), so that no rewrite changes its number of points:
/// a mobject that is resized while a fade-in is playing over it ends the
/// fade at the opacity it began at (0.016 here), and built as two vertices
/// this curve was resized in the first frame of its fade-in and was never
/// seen. (`set_points` returns Ok throughout; a rewrite after the fade has
/// ended, or of the same size, is fine.)
fn curve_vertices(amp: f64) -> Vec<Vec3> {
    (0..=CURVE_STEPS)
        .map(|k| {
            let t = k as f64 / CURVE_STEPS as f64;
            v(
                -DECK_HALF + 2.0 * DECK_HALF * t,
                DECK_Y + amp * SWAY * (std::f64::consts::PI * t).sin(),
            )
        })
        .collect()
}

/// The swing `seconds` into the breath of 14_7: out to one side, across to
/// the other and back to rest, each stroke eased at both ends.
fn breath_amplitude(seconds: f64) -> f64 {
    let (mut from, mut start) = (0.0, 0.0);
    for &(length, to) in &BREATH {
        if seconds < start + length {
            let u = (seconds - start) / length;
            return from + (to - from) * u * u * (3.0 - 2.0 * u);
        }
        from = to;
        start += length;
    }
    from
}

/// The bridge seen from above: two edges of the deck, a gold curve lying
/// along it whose sideways swing a tracker sets, small pistons beside it
/// and people walking along it. Everything is rewritten by one updater.
struct Footbridge {
    /// Edges, pistons, the gold curve and the walkers.
    parts: Vec<Mob>,
    /// The gold curve (read by the tests of its state).
    #[cfg(test)]
    curve: Mob,
    /// How far the gold curve swings (-1 to 1; set by the tests, driven by
    /// `breath` in the chapter), whether people walk, and whether it is
    /// breathing (the swing then follows `BREATH` by itself).
    #[cfg(test)]
    swing: Mob,
    walking: Mob,
    breath: Mob,
    anchor: Mob,
}

impl Footbridge {
    fn build(stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let mut parts = Vec::new();
        for s in [-1.0, 1.0] {
            let y = DECK_Y + s * DECK_WIDTH / 2.0;
            parts.push(
                stage.arena_mut().add(
                    polyline(vec![v(-DECK_HALF, y), v(DECK_HALF, y)])
                        .style(faded_line(dim, 4.0, 1.0))
                        .build(),
                ),
            );
        }
        // the pistons: a short thick stroke at each edge, beside the deck
        for i in 0..DAMPERS {
            let x = -DECK_HALF + 2.0 * DECK_HALF * (i as f64 + 0.5) / DAMPERS as f64;
            let s = if i % 2 == 0 { -1.0 } else { 1.0 };
            let y0 = DECK_Y + s * (DECK_WIDTH / 2.0 + 0.04);
            parts.push(
                stage.arena_mut().add(
                    polyline(vec![v(x, y0), v(x, y0 + s * 0.22)])
                        .style(faded_line(ink, 5.0, 1.0))
                        .build(),
                ),
            );
        }
        let curve = stage
            .arena_mut()
            .add(polyline(curve_vertices(0.0)).style(faded_line(own(), 4.0, 1.0)).build());
        parts.push(curve);
        let mut walkers = Vec::new();
        for i in 0..WALKERS {
            let x = -DECK_HALF + 2.0 * DECK_HALF * (i as f64 + 0.5) / WALKERS as f64;
            let y = DECK_Y + if i % 2 == 0 { -0.08 } else { 0.08 };
            let d = stage
                .arena_mut()
                .add(Dot::new().point(v(x, y)).radius(0.05).style(flat(ink, 0.95)).build());
            walkers.push((d, x, y));
            parts.push(d);
        }
        let swing = stage.add_value_tracker(0.0);
        let walking = stage.add_value_tracker(0.0);
        let breath = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = [f64::NAN; 2];
        let mut travelled = 0.0_f64;
        let mut breathed = 0.0_f64;
        let marks: Vec<(Mob, f64, f64)> = walkers;
        let template: Vec<Vec3> = Dot::new().radius(0.05).build().points().to_vec();
        stage.add_dt_updater(
            anchor,
            move |s, _, dt| {
                // the breath runs on the picture's clock, not on a play that
                // would hold the lines of the voice back
                if s.tracker_value(breath).unwrap_or(0.0) > 0.5 && dt > 0.0 {
                    breathed += dt;
                    let _ = s.set_tracker_value(swing, breath_amplitude(breathed));
                }
                let amp = s.tracker_value(swing).unwrap_or(0.0);
                let on = s.tracker_value(walking).unwrap_or(0.0) > 0.5;
                if on && dt > 0.0 {
                    travelled += 0.9 * dt;
                    for &(m, x0, y) in &marks {
                        let span = 2.0 * DECK_HALF;
                        let x = -DECK_HALF + (x0 + DECK_HALF + travelled).rem_euclid(span);
                        let c = v(x, y);
                        let pts: Vec<Vec3> = template.iter().map(|p| [p[0] + c[0], p[1] + c[1], p[2]]).collect();
                        let _ = s.set_points(m, &pts);
                    }
                }
                if amp.to_bits() != last[0].to_bits() {
                    last[0] = amp;
                    let _ = s.set_points(curve, &corners(&curve_vertices(amp)));
                }
            },
            false,
        )?;
        Ok(Self {
            parts,
            #[cfg(test)]
            curve,
            #[cfg(test)]
            swing,
            walking,
            breath,
            anchor,
        })
    }

    /// Let the gold curve breathe once, on the picture's clock.
    fn breathe(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.set_tracker_value(self.breath, 1.0)?;
        Ok(())
    }
}

// -------------------------------------------------------------- the scene

impl SceneConstruct for OneIdea {
    fn name(&self) -> &str {
        "14_one_idea"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange) = (moves(), pushes());

        // ---- 14_1: the split of chapter 04, run forwards
        let line = say(&kit, stage, "14_1")?;
        let shares = items("K-A3.mix")?;
        let half = fraction_value(shares[0])?;
        let rungs = numbers("K-A2.eigenvalues")?;
        let push = numbers("K-A1.push.left")?;
        let (up, down) = ([half, half], [half, -half]);
        let mut axes = CARTS.axes(stage, CARTS_REACH.0, CARTS_REACH.1);
        let own_dirs = own_lines(stage, CARTS, &[[1.0, 1.0], [1.0, -1.0]], CARTS_REACH.0, CARTS_REACH.1);
        let the_move = arrow_on(stage, CARTS, [0.0, 0.0], [1.0, 0.0], blue, 9.0, 1.0);
        let part_a = PlaneArrow::build(stage, CARTS, blue, orange, 6.0, 0.22, [0.0, 0.0], up)?;
        let part_b = PlaneArrow::build(stage, CARTS, blue, orange, 6.0, 0.22, up, [1.0, 0.0])?;
        let result = arrow_on(stage, CARTS, [0.0, 0.0], [push[0], push[1]], orange, 9.0, 1.0);
        let result_name = name_the_pushes(&kit, stage, &push)?;
        let move_name = name_the_move(&kit, stage)?;
        let table = MatrixGrid::build(
            &kit,
            stage,
            &super::draw::entries("K-A1.table", None)?,
            40.0,
            ink,
            v(4.7, 2.75),
        )?;
        let table_name = kit.text(stage, "the rule", 24.0, dim, v(2.4, 2.75))?;
        // the steps, in the right-hand column
        let half_tex = fractions("K-A3.mix")?;
        let (p, q) = (whole(rungs[0]), whole(rungs[1]));
        let step_a = kit.tex(
            stage,
            &format!(r"(1,0) = {}\,(1,1) + {}\,(1,-1)", half_tex[0], half_tex[1]),
            36.0,
            ink,
            &[],
            v(STEPS_X, 1.25),
        )?;
        let step_b = kit.tex(
            stage,
            &format!(r"{}\cdot {p}\,(1,1) + {}\cdot {q}\,(1,-1)", half_tex[0], half_tex[1]),
            36.0,
            ink,
            &[],
            v(STEPS_X, 0.3),
        )?;
        let step_c = kit.tex(
            stage,
            &format!(r"= ({},\,{})", whole(push[0]), whole(push[1])),
            36.0,
            orange,
            &[],
            v(STEPS_X, -0.55),
        )?;
        let letters = kit.tex(stage, r"K = Q\,\Lambda\,Q^{T}", 64.0, ink, &[], v(STEPS_X, -1.85))?;
        let unit_columns = kit.text(stage, "columns of length 1", 26.0, dim, v(STEPS_X, -2.7))?;
        part_a.start(stage)?;
        part_b.start(stage)?;
        axes.push(the_move);
        fade_in_all(stage, &axes, 0.6)?;
        fade_in_all(stage, &[move_name], 0.4)?;
        line.wait_for(stage, "Each is a rule in a table")?;
        fade_in_all(stage, &[table.group, table_name], 0.6)?;
        line.wait_for(stage, "And in its own patterns")?;
        // the gold lines, and the move cut along them into halves
        let mut lines_in: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &m) in own_dirs.iter().enumerate() {
            let o = if i % 2 == 0 { 0.25 } else { 1.0 };
            lines_in.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.set_opacity(o)?)?);
        }
        stage.play_prepared(lines_in)?;
        tween_all(stage, &[(part_a.on, 1.0), (part_b.on, 1.0)], 0.5)?;
        fade_in_all(stage, &[step_a], 0.6)?;
        // each half is multiplied by its rung: 1 leaves it, 3 stretches it
        line.wait_for(stage, "when it has enough of them")?;
        fade_in_all(stage, &[step_b], 0.6)?;
        line.wait_for(stage, "a table only multiplies")?;
        let end_b = [up[0] + rungs[1] * down[0], up[1] + rungs[1] * down[1]];
        tween_all(
            stage,
            &[
                (part_a.mix, 1.0),
                (part_b.mix, 1.0),
                (part_b.tip[0], end_b[0]),
                (part_b.tip[1], end_b[1]),
            ],
            1.4,
        )?;
        fade_in_all(stage, &[result, result_name, step_c], 0.6)?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[letters, unit_columns], 0.8)?;
        pause(stage, 1.6)?;

        // ---- 14_3: symmetric tables always have enough; the economy's
        // splits anyway
        let line = say(&kit, stage, "14_3")?;
        part_a.stop(stage);
        part_b.stop(stage);
        let mut leaving = axes.clone();
        leaving.extend(&own_dirs);
        leaving.extend([
            part_a.mob,
            part_b.mob,
            result,
            result_name,
            move_name,
            table.group,
            table_name,
            step_a,
            step_b,
            step_c,
            letters,
            unit_columns,
        ]);
        fade_out_all(stage, &leaving, 0.6)?;
        let economy = table_of("K-E1.table")?;
        let grid_a = MatrixGrid::build(
            &kit,
            stage,
            &super::draw::entries("K-A1.table", None)?,
            44.0,
            ink,
            v(-4.4, 2.25),
        )?;
        let grid_b = MatrixGrid::build(&kit, stage, &table_entries(&economy)?, 44.0, ink, v(-0.6, 2.25))?;
        let name_a = kit.text(stage, "the bridge's and the room's", 24.0, dim, v(-4.4, 3.45))?;
        let name_b = kit.text(stage, "the economy's", 24.0, dim, v(-0.6, 3.45))?;
        // the diagonal: a mirror line through the carts' table; the two
        // numbers either side of it match, the economy's do not
        let mirror = {
            let a = grid_a.cell_at(stage, 0, 0);
            let b = grid_a.cell_at(stage, 1, 1);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            stage.arena_mut().add(
                DashedLine::new(v(a[0] - 0.5 * dx, a[1] - 0.5 * dy), v(b[0] + 0.5 * dx, b[1] + 0.5 * dy))
                    .dash_length(0.1)
                    .style(faded_line(dim, 2.5, 1.0))
                    .build()?,
            )
        };
        let mut cell_marks = Vec::new();
        for (grid, color) in [(&grid_a, good()), (&grid_b, warn())] {
            for (r, c) in [(0, 1), (1, 0)] {
                let at = grid.cell_at(stage, r, c);
                let w = stage.get_width(grid.cells[r][c]) + 0.3;
                let ring = Rectangle::new()
                    .width(w)
                    .height(0.62)
                    .corner_radius(0.1)
                    .style(faded_line(color, 3.5, 1.0))
                    .build()?
                    .moved_to(at);
                cell_marks.push(stage.arena_mut().add(ring));
            }
        }
        // the two little planes: the carts' own lines at right angles; the
        // economy's, one into the quarter where tools would be negative
        let mut plane_a = MINI_A.axes(stage, MINI_REACH.0, MINI_REACH.1);
        let mut plane_b = MINI_B.axes(stage, MINI_REACH.0, MINI_REACH.1);
        let lines_a = own_lines(stage, MINI_A, &[[1.0, 1.0], [1.0, -1.0]], MINI_REACH.0, MINI_REACH.1);
        let (mix, other) = (numbers("K-E2.mix")?, numbers("K-E2.other")?);
        let lines_b_solid = own_lines(stage, MINI_B, &[[mix[0], mix[1]]], MINI_REACH.0, MINI_REACH.1);
        let (e0, e1) = band_ends([other[0], other[1]], MINI_REACH.0, MINI_REACH.1);
        let dashed = DashedLine::new(MINI_B.p(e0[0], e0[1]), MINI_B.p(e1[0], e1[1]))
            .dash_length(0.12)
            .style(faded_line(own(), 3.0, 0.0))
            .build()?;
        let dashed = stage.arena_mut().add(dashed);
        let hatch = hatched_quarter(stage, MINI_B, MINI_REACH);
        // a square corner where the carts' two lines cross
        let corner = {
            let (u, w) = (unit_at(45.0), unit_at(135.0));
            let a = 0.35;
            let p = |x: f64, y: f64| MINI_A.p(x, y);
            stage.arena_mut().add(
                polyline(vec![
                    p(a * u[0], a * u[1]),
                    p(a * (u[0] + w[0]), a * (u[1] + w[1])),
                    p(a * w[0], a * w[1]),
                ])
                .style(faded_line(ink, 2.5, 1.0))
                .build(),
            )
        };
        let say_a = kit.text(stage, "at right angles", 24.0, dim, v(-4.4, -1.75))?;
        let say_b = kit.text(stage, "not at right angles", 24.0, dim, v(-0.6, -1.75))?;
        let spectral = lower_third(
            &kit,
            stage,
            "the spectral theorem:",
            "a table that is the same on both sides of the diagonal always has a full set of own patterns, at right \
             angles (for motion, right angles counted with mass)",
        )?;
        let split_letters = kit.tex(stage, r"C = V\,\Lambda\,V^{-1}", 64.0, ink, &[], v(4.3, 1.4))?;
        plane_a.push(say_a);
        plane_b.push(say_b);
        let mut first = vec![grid_a.group, grid_b.group, name_a, name_b];
        first.extend(&plane_a);
        first.extend(&plane_b);
        fade_in_all(stage, &first, 0.7)?;
        line.wait_for(stage, "are symmetric")?;
        fade_in_all(stage, &[mirror, cell_marks[0], cell_marks[1]], 0.5)?;
        fade_in_all(stage, &[spectral], 0.5)?;
        line.wait_for(stage, "so they always have enough")?;
        let mut gold_a: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &m) in lines_a.iter().enumerate() {
            let o = if i % 2 == 0 { 0.25 } else { 1.0 };
            gold_a.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.set_opacity(o)?)?);
        }
        stage.play_prepared(gold_a)?;
        fade_in_all(stage, &[corner], 0.4)?;
        line.wait_for(stage, "The economy's is not")?;
        fade_in_all(stage, &[cell_marks[2], cell_marks[3]], 0.5)?;
        line.wait_for(stage, "so it still splits")?;
        let mut gold_b: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &m) in lines_b_solid.iter().enumerate() {
            let o = if i % 2 == 0 { 0.25 } else { 1.0 };
            gold_b.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.set_opacity(o)?)?);
        }
        gold_b.push(stage.prepare(dashed.animate().set_anim_args(run(0.6))?.set_opacity(1.0)?)?);
        stage.play_prepared(gold_b)?;
        fade_in_all(stage, &[split_letters], 0.7)?;
        line.wait_for(stage, "Having nothing negative")?;
        let mut hatching: Vec<Box<dyn Animation>> = Vec::new();
        for &h in &hatch {
            hatching.push(stage.prepare(h.animate().set_anim_args(run(1.2))?.set_opacity(0.55)?)?);
        }
        stage.play_prepared(hatching)?;
        line.wait_end(stage)?;
        pause(stage, 1.2)?;

        // ---- 14_4: where does the idea stop?
        let line = say(&kit, stage, "14_4")?;
        let mut leaving = first.clone();
        leaving.extend([mirror, spectral, corner, split_letters, dashed]);
        leaving.extend(&cell_marks);
        leaving.extend(&lines_a);
        leaving.extend(&lines_b_solid);
        leaving.extend(&hatch);
        fade_out_all(stage, &leaving, 0.6)?;
        let ask_card = Rectangle::new()
            .width(9.0)
            .height(2.2)
            .corner_radius(0.18)
            .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(v(0.0, 0.3));
        let ask_card = stage.arena_mut().add(ask_card);
        let ask = kit.text(stage, "where does the idea stop?", 52.0, ink, v(0.0, 0.3))?;
        fade_in_all(stage, &[ask_card, ask], 0.6)?;
        line.wait_end(stage)?;
        predict_hold(&kit, stage)?;

        // ---- 14_5: a quarter turn, steel past yield, a factory's limits
        let line = say(&kit, stage, "14_5")?;
        fade_out_all(stage, &[ask_card, ask], 0.5)?;
        // the quarter turn: the orange arrow stays at right angles to the
        // blue one all the way round; the readout does not move
        let quarter = [[0.0, -1.0], [1.0, 0.0]];
        let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let spin = ArrowTest::build(stage, TURN, quarter, u, TURN_REACH)?;
        let turn_name = kit.text(stage, "a quarter turn", 30.0, ink, TURN.p(0.0, TURN_REACH.1 + 0.35))?;
        let readout = angle_readout(&kit, stage, &spin, READOUT_AT)?;
        let readout_name = kit.text(stage, "angle between the arrows", 26.0, dim, v(4.9, 0.4))?;
        let degrees = degree_sign(&kit, stage, READOUT_AT)?;
        // the shear: one own line, small
        let shear_map = [[1.0, 1.0], [0.0, 1.0]];
        let s = [stage.add_value_tracker(0.5), stage.add_value_tracker(1.0)];
        let shear = ArrowTest::build(stage, SHEAR, shear_map, s, SHEAR_REACH)?;
        let shear_line = own_lines(stage, SHEAR, &[[1.0, 0.0]], SHEAR_REACH.0, SHEAR_REACH.1);
        let shear_name = kit.text(
            stage,
            "a shear: nothing negative,\nand still only one",
            24.0,
            dim,
            v(4.6, 3.35),
        )?;
        let mut turn_in = spin.axes.clone();
        turn_in.extend([turn_name, readout_name, degrees]);
        spin.start(stage)?;
        stage.set_tracker_value(spin.polar, 1.0)?;
        stage.set_tracker_value(spin.radius, 2.2)?;
        stage.add_to_scene(readout)?;
        fade_in_all(stage, &turn_in, 0.6)?;
        fade_in_all(stage, &[readout], 0.3)?;
        tween_all(stage, &[(spin.blue_on, 1.0), (spin.orange_on, 1.0)], 0.5)?;
        let sweep = stage.prepare(tween_linear(spin.degrees, 360.0, 5.0)?)?;
        stage.play_prepared(vec![sweep])?;
        line.wait_for(stage, "Steel bent too far")?;
        let mut shear_in = shear.axes.clone();
        shear_in.push(shear_name);
        fade_in_all(stage, &shear_in, 0.5)?;
        shear.start(stage)?;
        tween_all(stage, &[(shear.blue_on, 1.0), (shear.orange_on, 1.0)], 0.5)?;
        play!(stage; shear_line[0].animate().set_anim_args(run(0.5))?.set_opacity(0.25)?, shear_line[1].animate().set_anim_args(run(0.5))?.set_opacity(1.0)?);
        pause(stage, 1.0)?;
        // steel past yield: the push no longer follows the move in proportion
        spin.stop(stage);
        shear.stop(stage);
        let mut leaving = turn_in.clone();
        leaving.extend([
            spin.blue,
            spin.orange,
            spin.drops,
            readout,
            shear.blue,
            shear.orange,
            shear.drops,
        ]);
        leaving.extend(&shear_in);
        leaving.extend(&shear_line);
        fade_out_all(stage, &leaving, 0.5)?;
        let yield_mpa = fact("K-T1.yield_mpa")?;
        let top = PLOT_BOTTOM + PLOT_H;
        let at_yield = PLOT_BOTTOM + YIELD_AT * PLOT_H;
        let plot_axes = stage.arena_mut().add(
            polyline(vec![
                v(PLOT_LEFT, top),
                v(PLOT_LEFT, PLOT_BOTTOM),
                v(PLOT_LEFT + PLOT_W, PLOT_BOTTOM),
            ])
            .style(faded_line(Palette::faint(), 3.0, 1.0))
            .build(),
        );
        // the straight line (Hooke's, the table's) and the line the steel
        // follows: straight to yield, then bending over
        let to_yield = YIELD_AT;
        let along = |t: f64| PLOT_LEFT + PLOT_W * t;
        let up = |s: f64| PLOT_BOTTOM + PLOT_H * s;
        let straight = stage.arena_mut().add(
            polyline(vec![v(along(0.0), up(0.0)), v(along(0.96), up(0.96 * 1.0))])
                .style(faded_line(dim, 2.5, 1.0))
                .build(),
        );
        let mut path = vec![v(along(0.0), up(0.0)), v(along(to_yield), up(to_yield))];
        for k in 1..=24 {
            let t = to_yield + (0.96 - to_yield) * f64::from(k) / 24.0;
            let past = t - to_yield;
            // bends over towards a plateau
            let s = to_yield + 0.22 * (1.0 - (-5.0 * past / (0.96 - to_yield)).exp());
            path.push(v(along(t), up(s)));
        }
        let steel = stage
            .arena_mut()
            .add(polyline(path).style(faded_line(ink, 6.0, 1.0)).build());
        let yield_mark = stage.arena_mut().add(
            polyline(vec![v(PLOT_LEFT - 0.1, at_yield), v(PLOT_LEFT + PLOT_W, at_yield)])
                .style(faded_line(warn(), 2.5, 1.0))
                .build(),
        );
        let yield_name = kit.text_left(
            stage,
            &format!("{yield_mpa} N per mm\u{b2}: yield"),
            26.0,
            warn(),
            v(PLOT_LEFT + 0.2, at_yield + 0.3),
        )?;
        let move_axis = kit.text(
            stage,
            "how far its ends move apart",
            24.0,
            dim,
            v(PLOT_LEFT + PLOT_W / 2.0, PLOT_BOTTOM - 0.3),
        )?;
        let push_axis = kit.text(stage, "stress", 24.0, dim, v(PLOT_LEFT + 0.1, top + 0.25))?;
        let gauge_x = 3.4;
        let gauge_h = PLOT_H;
        let gauge_frame = Rectangle::new()
            .width(0.9)
            .height(gauge_h)
            .style(faded_line(ink, 3.5, 1.0))
            .build()?
            .moved_to(v(gauge_x, PLOT_BOTTOM + gauge_h / 2.0));
        let gauge_frame = stage.arena_mut().add(gauge_frame);
        let below = YIELD_AT * gauge_h;
        let past_h = PAST * gauge_h * 0.6;
        let fill_ok = Rectangle::new()
            .width(0.8)
            .height(below)
            .style(flat(good(), 0.9))
            .build()?
            .moved_to(v(gauge_x, PLOT_BOTTOM + below / 2.0));
        let fill_ok = stage.arena_mut().add(fill_ok);
        let fill_past = Rectangle::new()
            .width(0.8)
            .height(past_h)
            .style(flat(warn(), 0.9))
            .build()?
            .moved_to(v(gauge_x, PLOT_BOTTOM + below + past_h / 2.0));
        let fill_past = stage.arena_mut().add(fill_past);
        // (a beat's id is never shown to the viewer: "from before" is what 05_8 was to them)
        let gauge_name = kit.text(stage, "the gauge from before", 24.0, dim, v(gauge_x, top + 0.25))?;
        let mark_at = stage.arena_mut().add(
            polyline(vec![
                v(gauge_x - 0.6, PLOT_BOTTOM + below),
                v(gauge_x + 0.6, PLOT_BOTTOM + below),
            ])
            .style(faded_line(warn(), 4.0, 1.0))
            .build(),
        );
        fade_in_all(
            stage,
            &[plot_axes, straight, yield_mark, yield_name, move_axis, push_axis],
            0.6,
        )?;
        fade_in_all(stage, &[gauge_frame, gauge_name, mark_at], 0.5)?;
        play!(stage; show_creation(steel).rt(1.6));
        fade_in_all(stage, &[fill_ok], 0.5)?;
        fade_in_all(stage, &[fill_past], 0.7)?;
        // the factory: three limits, the allowed region, the line of equal
        // profit slides out to a corner
        line.wait_for(stage, "And some questions about scarce resources")?;
        let leaving = vec![
            plot_axes,
            straight,
            steel,
            yield_mark,
            yield_name,
            move_axis,
            push_axis,
            gauge_frame,
            gauge_name,
            mark_at,
            fill_ok,
            fill_past,
        ];
        fade_out_all(stage, &leaving, 0.5)?;
        let limits = matrix("K-L1.limits")?;
        let profit = numbers("K-L1.profit")?;
        let corner_text = fact("K-L2.corners")?;
        let corner_points: Vec<[f64; 2]> = corner_text
            .split(';')
            .map(|p| {
                let xy: Vec<f64> = p.split(',').filter_map(|s| s.parse().ok()).collect();
                [xy[0], xy[1]]
            })
            .collect();
        let region_pts = round_the_polygon(&corner_points);
        let plane_pt = |p: [f64; 2]| FACTORY.p(p[0], p[1]);
        let region = {
            let pts: Vec<Vec3> = region_pts.iter().map(|&p| plane_pt(p)).collect();
            let poly = Polygon::new(pts)
                .style(Style::default().fill(good(), 0.35).stroke(good(), 3.0, 1.0))
                .build();
            stage.arena_mut().add(poly)
        };
        let fact_axes = stage.arena_mut().add(
            polyline(vec![
                FACTORY.p(0.0, WINDOW),
                FACTORY.p(0.0, 0.0),
                FACTORY.p(WINDOW, 0.0),
            ])
            .style(faded_line(Palette::faint(), 3.0, 1.0))
            .build(),
        );
        let mut limit_mobs = Vec::new();
        let mut limit_names = Vec::new();
        let names = items("K-L2.tight")?;
        for (row, name) in limits.iter().zip(&names) {
            if let Some((a, b)) = line_in_window(row[0], row[1], row[2], WINDOW) {
                limit_mobs.push(
                    stage.arena_mut().add(
                        polyline(vec![plane_pt(a), plane_pt(b)])
                            .style(faded_line(ink, 3.0, 1.0))
                            .build(),
                    ),
                );
                // the name sits beyond the end that is higher (or further
                // right)
                let end = if a[1] > b[1] || (a[1] == b[1] && a[0] > b[0]) {
                    a
                } else {
                    b
                };
                let at = plane_pt(end);
                let (dx, dy) = if end[1] >= WINDOW - 1e-6 {
                    (0.0, 0.3)
                } else if end[0] >= WINDOW - 1e-6 {
                    (0.85, 0.0)
                } else {
                    (-0.7, 0.15)
                };
                limit_names.push(kit.text(stage, name, 24.0, dim, v(at[0] + dx, at[1] + dy))?);
            }
        }
        let slide = stage.add_value_tracker(0.0);
        let best = numbers("K-L2.best")?;
        let best_profit = numbers("K-L2.best_profit")?[0];
        let profit_line = stage.arena_mut().add(
            polyline(vec![plane_pt([0.0, 0.0]), plane_pt([1e-3, 0.0])])
                .style(faded_line(ink, 6.0, 1.0))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let c = s.tracker_value(slide).unwrap_or(0.0);
                if c.to_bits() == last.to_bits() {
                    return;
                }
                last = c;
                if let Some((a, b)) = line_in_window(profit[0], profit[1], c.max(1e-3), WINDOW) {
                    let _ = s.set_points(profit_line, &corners(&[FACTORY.p(a[0], a[1]), FACTORY.p(b[0], b[1])]));
                }
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let corner_dot = stage.arena_mut().add(
            Dot::new()
                .point(plane_pt([best[0], best[1]]))
                .radius(0.12)
                .style(flat(own(), 1.0))
                .build(),
        );
        let best_name = kit.text_left(
            stage,
            &format!("${}: linear programming", whole(best_profit)),
            32.0,
            ink,
            v(
                plane_pt([best[0], best[1]])[0] + 0.5,
                plane_pt([best[0], best[1]])[1] + 0.35,
            ),
        )?;
        let bicycles = kit.text(stage, "bicycles", 24.0, dim, FACTORY.p(WINDOW - 3.0, -4.5))?;
        let scooters = kit.text(stage, "scooters", 24.0, dim, FACTORY.p(-10.0, WINDOW - 2.0))?;
        let mut factory_in = vec![fact_axes, region, bicycles, scooters];
        factory_in.extend(&limit_mobs);
        factory_in.extend(&limit_names);
        fade_in_all(stage, &factory_in, 0.7)?;
        stage.add_to_scene(profit_line)?;
        play!(stage; tween(slide, best_profit, 3.0)?);
        fade_in_all(stage, &[corner_dot, best_name], 0.5)?;
        let powers = kit.tex(stage, r"C^{n} = V\,\Lambda^{n}\,V^{-1}", 52.0, ink, &[], v(4.0, 1.0))?;
        let companion = lower_third(
            &kit,
            stage,
            "where it holds:",
            "the same split gives powers of a table and its exponential; see the companion video on the matrix \
             exponential",
        )?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[powers, companion], 0.6)?;
        // the companion video's band has to be up long enough to read (1.9 s
        // was not), and the beat is a pause besides
        let companion_up = now(stage);
        wait_until(stage, companion_up + BAND_UP)?;

        // ---- 14_6: London's footbridge reopened: the pattern was
        // predicted; what would push along with it was not
        let line = say(&kit, stage, "14_6")?;
        stage.clear_updaters(anchor, true);
        let mut leaving = vec![
            region,
            fact_axes,
            profit_line,
            corner_dot,
            best_name,
            bicycles,
            scooters,
            powers,
            companion,
        ];
        leaving.extend(&limit_mobs);
        leaving.extend(&limit_names);
        fade_out_all(stage, &leaving, 0.6)?;
        let bridge = Footbridge::build(stage)?;
        stage.add_to_scene(bridge.anchor)?;
        fade_in_all(stage, &bridge.parts, 0.8)?;
        stage.set_tracker_value(bridge.walking, 1.0)?;
        let reopened = kit.text(
            stage,
            &format!("reopened {}", long_date(fact("K-B2.reopened")?)?),
            32.0,
            ink,
            v(0.0, 0.55),
        )?;
        let model = kit.text(
            stage,
            &format!(
                "design model: sideways modes within {}% of measured (Dallard et al. {})",
                fact("K-B2.predicted_within_percent")?,
                fact("K-B2.paper_year")?
            ),
            28.0,
            ink,
            v(0.0, -0.25),
        )?;
        let planned = kit.text(
            stage,
            &format!(
                "as planned in {}: {} viscous dampers and {} pairs of sideways tuned mass dampers",
                fact("K-B2.paper_year")?,
                fact("K-B2.dampers_viscous")?,
                fact("K-B2.tmd_lateral_pairs")?
            ),
            28.0,
            ink,
            v(0.0, -1.0),
        )?;
        let fix = kit.text(
            stage,
            "the fix was more damping, not a stronger bridge",
            30.0,
            own(),
            v(0.0, -1.95),
        )?;
        line.wait_for(stage, "London's footbridge reopened")?;
        fade_in_all(stage, &[reopened], 0.5)?;
        line.wait_for(stage, "Its engineers had known")?;
        fade_in_all(stage, &[model], 0.5)?;
        line.wait_for(stage, "What no code had asked")?;
        fade_in_all(stage, &[planned, fix], 0.6)?;
        line.wait_end(stage)?;

        // ---- 14_7: three questions; the gold curve breathes once
        let line = say(&kit, stage, "14_7")?;
        fade_out_all(stage, &[reopened, model, planned, fix], 0.5)?;
        let questions = [
            "How strong is it?",
            "How does it want to move?",
            "And what will push along with it?",
        ];
        let mut asked = Vec::new();
        for (i, q) in questions.iter().enumerate() {
            asked.push(kit.text(stage, q, 40.0, ink, v(0.0, 0.2 - 0.95 * i as f64))?);
        }
        stage.set_tracker_value(bridge.walking, 0.0)?;
        // the gold curve breathes once on the picture's clock (5.6 s inside
        // the line's 6.7): plays here would run to their end before the
        // first question, and the three would arrive after the voice
        bridge.breathe(stage)?;
        line.wait_for(stage, "How strong is it")?;
        fade_in_all(stage, &asked[..1], 0.4)?;
        line.wait_for(stage, "How does it want to move")?;
        fade_in_all(stage, &asked[1..2], 0.4)?;
        line.wait_for(stage, "And what will push along with it")?;
        // the last question is said with a breath of silence before it and
        // no music under it (R1 C20)
        kit.mark(stage, "silence:last_question");
        fade_in_all(stage, &asked[2..], 0.4)?;
        kit.hold(stage)?;

        // ---- 14_8: the receipts, in silence
        let receipts = receipts_card(
            &kit,
            stage,
            &[
                "Every computed number in this video was checked by a second program, written separately.",
                "Every cited number has its source in the description.",
                "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
            ],
        )?;
        let mut leaving = bridge.parts.clone();
        leaving.extend(&asked);
        stage.clear_updaters(bridge.anchor, true);
        // the bridge goes as the card comes (a cross-fade, 0.7 s): the tail
        // is the card, the 2.5 s window in which the music resolves, and
        // the closing fade, under the cue check's 5 s
        cross_fade(stage, &leaving, &[receipts], 0.7)?;
        let label = format!("{}:14_8", kit.chapter);
        kit.listen(stage, 2.5, &label)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::super::draw::testing::Room;
    use super::*;
    use crate::model::factory;

    /// 14_1: the move splits into halves along the own lines, each half is
    /// multiplied by its rung, and the pieces add to the pushes of 02_3.
    #[test]
    fn the_halves_times_their_rungs_add_to_the_carts_pushes() {
        let table = matrix("K-A1.table").unwrap();
        let shares = items("K-A3.mix").unwrap();
        assert_eq!(shares, ["1/2", "1/2"]);
        let half = fraction_value(shares[0]).unwrap();
        let rungs = numbers("K-A2.eigenvalues").unwrap();
        assert_eq!(rungs, [1.0, 3.0]);
        let (up, down) = ([half, half], [half, -half]);
        // the move (1, 0) is the two halves tip to tail
        assert_eq!([up[0] + down[0], up[1] + down[1]], [1.0, 0.0]);
        // each half is an own pattern with its rung
        for (piece, rung) in [(up, rungs[0]), (down, rungs[1])] {
            let pushed = [
                table[0][0] * piece[0] + table[0][1] * piece[1],
                table[1][0] * piece[0] + table[1][1] * piece[1],
            ];
            assert!((pushed[0] - rung * piece[0]).abs() < 1e-12 && (pushed[1] - rung * piece[1]).abs() < 1e-12);
        }
        // multiplied by the rungs and added: the pushes of 02_3
        let end = [
            up[0] * rungs[0] + down[0] * rungs[1],
            up[1] * rungs[0] + down[1] * rungs[1],
        ];
        assert_eq!(end, [2.0, -1.0]);
        assert_eq!(fact("K-A1.push.left").unwrap(), "2,-1");
        // and the picture's second piece starts at the first one's tip and
        // ends where the facts say
        let tip = [up[0] + rungs[1] * down[0], up[1] + rungs[1] * down[1]];
        assert_eq!(tip, end);
        // everything fits the plane's reach
        assert!(end[0].abs() < CARTS_REACH.0 && end[1].abs() < CARTS_REACH.1);
        assert_eq!(fraction_tex("1/2"), r"\tfrac{1}{2}");
    }

    /// 14_3: the carts' table is the same on both sides of its diagonal,
    /// the economy's is not, and the economy's own lines are not at right
    /// angles (the carts' are).
    #[test]
    fn the_carts_table_is_symmetric_the_economys_is_not() {
        let t = matrix("K-A1.table").unwrap();
        assert_eq!(t[0][1], t[1][0]);
        let e = table_of("K-E1.table").unwrap();
        assert!((e[0][1] - e[1][0]).abs() > 0.05, "0.3 and 0.2 do not match");
        assert_eq!((e[0][1], e[1][0]), (0.3, 0.2));
        let dot = |a: &[f64], b: &[f64]| a[0] * b[0] + a[1] * b[1];
        assert_eq!(dot(&[1.0, 1.0], &[1.0, -1.0]), 0.0);
        let (mix, other) = (numbers("K-E2.mix").unwrap(), numbers("K-E2.other").unwrap());
        assert!(dot(&mix, &other).abs() > 0.5, "the economy's are not at right angles");
        assert_eq!(
            fact("K-X1.symmetric.checked").unwrap(),
            fact("K-X1.symmetric.hold").unwrap()
        );
        // the dashed line is the one into the quarter with tools negative
        assert!(other[0] > 0.0 && other[1] < 0.0);
    }

    /// 14_5: a quarter turn never lines up, a shear has one own line, the
    /// readout stays at 90, steel's yield and the factory are the facts.
    #[test]
    fn the_edges_are_the_facts() {
        let quarter = [[0.0, -1.0], [1.0, 0.0]];
        for a in [0.0, 17.0, 90.0, 200.0, 333.0] {
            assert!(
                (super::super::arrows::turn_deg(&quarter, unit_at(a)) - 90.0).abs() < 1e-9,
                "{a}"
            );
        }
        assert_eq!(fact("K-X2.quarter_turn").unwrap(), "0");
        assert_eq!(fact("K-X2.shear").unwrap(), "1");
        let shear = [[1.0, 1.0], [0.0, 1.0]];
        // the one line: along the first axis; anything else turns
        assert!(super::super::arrows::turn_deg(&shear, [1.0, 0.0]) < 1e-9);
        assert!(super::super::arrows::turn_deg(&shear, [0.5, 1.0]) > 5.0);
        assert_eq!(fact("K-T1.yield_mpa").unwrap(), "250");
        // the factory: three limits, four corners, best (40, 20), $460
        let limits = matrix("K-L1.limits").unwrap();
        assert_eq!(limits.len(), 3);
        let profit = numbers("K-L1.profit").unwrap();
        let best = numbers("K-L2.best").unwrap();
        assert_eq!(
            profit[0] * best[0] + profit[1] * best[1],
            numbers("K-L2.best_profit").unwrap()[0]
        );
        let region = round_the_polygon(
            &fact("K-L2.corners")
                .unwrap()
                .split(';')
                .map(|p| {
                    let xy: Vec<f64> = p.split(',').map(|s| s.parse().unwrap()).collect();
                    [xy[0], xy[1]]
                })
                .collect::<Vec<_>>(),
        );
        assert_eq!(region.len(), 4);
        assert_eq!(region[0], [0.0, 0.0]);
        // the corners are where the model puts them, and the best is one
        let model: Vec<[f64; 2]> = factory::LIMITS
            .iter()
            .map(|l| [l.bicycles as f64, l.scooters as f64])
            .collect();
        assert_eq!(model.len(), 3);
        assert!(region.contains(&[best[0], best[1]]));
        // every limit crosses the window, and the equal-profit line at the
        // best sits in it
        for row in &limits {
            assert!(line_in_window(row[0], row[1], row[2], WINDOW).is_some());
        }
        assert!(line_in_window(profit[0], profit[1], numbers("K-L2.best_profit").unwrap()[0], WINDOW).is_some());
        // no corner is outside a limit
        for p in &region {
            for row in &limits {
                assert!(row[0] * p[0] + row[1] * p[1] <= row[2] + 1e-9, "{p:?}");
            }
        }
    }

    /// 14_6: the date as it is said, and the bridge's numbers.
    #[test]
    fn the_bridge_numbers_are_the_facts() {
        assert_eq!(long_date("2002-02-22").unwrap(), "22 February 2002");
        assert_eq!(long_date(fact("K-B2.reopened").unwrap()).unwrap(), "22 February 2002");
        assert!(long_date("22 February 2002").is_err());
        assert_eq!(fact("K-B2.predicted_within_percent").unwrap(), "10");
        assert_eq!(fact("K-B2.paper_year").unwrap(), "2001");
        assert_eq!(fact("K-B2.dampers_viscous").unwrap(), "37");
        assert_eq!(fact("K-B2.tmd_lateral_pairs").unwrap(), "4");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("14_one_idea").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "the pushes\nfrom before:\n2, \u{2013}1",
            "the move",
            r"K = Q\,\Lambda\,Q^{T}",
            "columns of length 1",
            r"C = V\,\Lambda\,V^{-1}",
            "where does the idea stop?",
            "250 N per mm\u{b2}: yield",
            "$460: linear programming",
            r"C^{n} = V\,\Lambda^{n}\,V^{-1}",
            "reopened 22 February 2002",
            "within 10% of measured (Dallard et al. 2001)",
            "37 viscous dampers and 4 pairs",
            "the fix was more damping, not a stronger bridge",
            "How strong is it?",
            "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(
            words.iter().all(|w| !w.contains('\u{2026}')),
            "a lower-third was cut short: {words:?}"
        );
        assert!(ran.seconds > 40.0 && ran.seconds < 95.0, "{} s", ran.seconds);
    }

    /// 14_1: "the move" and "the pushes from before" stand where no stroke
    /// of the picture runs through them: not the axes, not the gold arms,
    /// not any of the arrows (the halves, the stretched half and the pushes
    /// at their ends). Both were crossed.
    #[test]
    fn the_names_of_the_move_and_the_pushes_stand_clear_of_the_strokes() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let push = numbers("K-A1.push.left")?;
            let (move_box, push_box) = (
                {
                    let m = name_the_move(&kit, stage)?;
                    Room::of(stage, m)
                },
                {
                    let m = name_the_pushes(&kit, stage, &push)?;
                    Room::of(stage, m)
                },
            );
            let shares = items("K-A3.mix")?;
            let half = fraction_value(shares[0])?;
            let rungs = numbers("K-A2.eigenvalues")?;
            let (up, down) = ([half, half], [half, -half]);
            let end_b = [up[0] + rungs[1] * down[0], up[1] + rungs[1] * down[1]];
            let (rx, ry) = CARTS_REACH;
            // the strokes, plane units, with the half width each is drawn with
            let mut strokes: Vec<([f64; 2], [f64; 2], f64)> = vec![
                ([-rx, 0.0], [rx, 0.0], 0.04),
                ([0.0, -ry], [0.0, ry], 0.04),
                ([0.0, 0.0], [1.0, 0.0], 0.08),
                ([0.0, 0.0], up, 0.06),
                (up, end_b, 0.06),
                ([0.0, 0.0], [push[0], push[1]], 0.08),
            ];
            for dir in [[1.0, 1.0], [1.0, -1.0]] {
                let (a, b) = band_ends(dir, rx, ry);
                strokes.push((a, b, 0.13));
            }
            for (name, room) in [("the move", move_box), ("the pushes", push_box)] {
                for &(a, b, half_width) in &strokes {
                    assert!(
                        !room.grown(half_width).crosses(CARTS.p(a[0], a[1]), CARTS.p(b[0], b[1])),
                        "{name} {room:?} is crossed by the stroke {a:?} to {b:?}"
                    );
                }
            }
            // they do not stand over each other (the steps beside them are
            // checked by the layout lint on the whole chapter)
            assert!(!move_box.overlaps(&push_box));
            Ok(())
        });
    }

    /// 14_5: the readout's degree sign is against its number (a quarter turn
    /// reads 90 all the way round), as in 12_5.
    #[test]
    fn the_degree_sign_is_against_the_readout() {
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let quarter = [[0.0, -1.0], [1.0, 0.0]];
            let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
            let spin = ArrowTest::build(stage, TURN, quarter, u, TURN_REACH)?;
            let readout = angle_readout(&kit, stage, &spin, READOUT_AT)?;
            let sign = degree_sign(&kit, stage, READOUT_AT)?;
            spin.start(stage)?;
            stage.set_tracker_value(spin.polar, 1.0)?;
            stage.set_tracker_value(spin.radius, 2.2)?;
            stage.add_to_scene(readout)?;
            stage.add_to_scene(sign)?;
            stage.wait(0.2)?;
            let (number, sign) = (Room::of(stage, readout), Room::of(stage, sign));
            assert!((number.x1 - READOUT_AT[0]).abs() < 1e-6, "{number:?}");
            let gap = sign.x0 - number.x1;
            assert!(gap > 0.0 && gap < 0.1, "the sign stands {gap} off the number");
            Ok(())
        });
    }

    /// 14_6 and 14_7: the gold curve is really drawn and really moves, run
    /// as the chapter runs it: the updater on the scene, then the parts faded
    /// in. The curve was built as two vertices and its updater rewrote it as
    /// forty-nine in the first frame of that fade-in; a fade that is playing
    /// when a mobject changes its number of points ends at the opacity it
    /// began at (here 0.016), so the curve was never on the screen and the
    /// deck never swayed (`set_points` itself returned Ok). A test of the
    /// state, not the lint.
    #[test]
    fn the_gold_curve_is_drawn_and_swings_with_the_tracker() {
        super::super::carts::probe::run_scene(|stage| {
            let bridge = Footbridge::build(stage)?;
            stage.add_to_scene(bridge.anchor)?;
            fade_in_all(stage, &bridge.parts, 0.8)?;
            let seen = stage.get_stroke_opacity(bridge.curve).unwrap_or(0.0);
            assert!(seen > 0.99, "the curve ended its fade-in at opacity {seen}");
            let at_rest = stage.get_points(bridge.curve).expect("the curve has points");
            // 49 vertices as a quad path: an odd run
            assert_eq!(at_rest.len(), 2 * (CURVE_STEPS + 1) - 1);
            // (the engine keeps single precision: 1e-5 is the tolerance)
            assert!(
                at_rest.iter().all(|p| (p[1] - DECK_Y).abs() < 1e-5),
                "lies along the deck"
            );
            stage.set_tracker_value(bridge.swing, 1.0)?;
            stage.wait(0.2)?;
            let swung = stage.get_points(bridge.curve).expect("the curve has points");
            assert_eq!(swung.len(), at_rest.len());
            let top = swung.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
            assert!((top - (DECK_Y + SWAY)).abs() < 1e-5, "the swing reached {top}");
            // the ends stay on the deck's ends
            assert!((swung[0][1] - DECK_Y).abs() < 1e-5 && (swung[swung.len() - 1][1] - DECK_Y).abs() < 1e-5);
            let still = stage.get_stroke_opacity(bridge.curve).unwrap_or(0.0);
            assert!(still > 0.99, "the swing left the curve at opacity {still}");
            Ok(())
        });
    }

    /// 14_7: the breath is one stroke each way and back, 5.6 s in all, and
    /// it runs by itself: the tracker is driven from the picture's clock.
    #[test]
    fn the_breath_goes_out_across_and_back_by_itself() {
        let total: f64 = BREATH.iter().map(|&(s, _)| s).sum();
        assert!((total - 5.6).abs() < 1e-9);
        assert_eq!(breath_amplitude(0.0), 0.0);
        let mut at = 0.0;
        for &(length, to) in &BREATH {
            at += length;
            assert!((breath_amplitude(at) - to).abs() < 1e-9, "{at}");
        }
        assert_eq!(breath_amplitude(total + 3.0), 0.0);
        // eased: it never overshoots a stroke's end
        for k in 0..=56 {
            let a = breath_amplitude(f64::from(k) / 10.0);
            assert!(a.abs() <= 1.0 + 1e-12);
        }
        super::super::carts::probe::run_scene(|stage| {
            let bridge = Footbridge::build(stage)?;
            stage.add_to_scene(bridge.anchor)?;
            bridge.breathe(stage)?;
            stage.wait(BREATH[0].0)?;
            let out = stage.tracker_value(bridge.swing).unwrap_or(f64::NAN);
            assert!(out > 0.9, "after the first stroke the swing is {out}");
            stage.wait(BREATH[1].0)?;
            let across = stage.tracker_value(bridge.swing).unwrap_or(f64::NAN);
            assert!(across < -0.9, "after the second stroke the swing is {across}");
            stage.wait(BREATH[2].0 + 0.3)?;
            let rest = stage.tracker_value(bridge.swing).unwrap_or(f64::NAN);
            assert!(rest.abs() < 1e-6, "at rest the swing is {rest}");
            Ok(())
        });
    }

    /// 14_8: the receipts card, the 2.5 s window in which the music resolves and the closing fade are the authored tail; it ends under the corpus cue check's 5 s of picture after the last line.
    #[test]
    fn the_picture_after_the_last_line_stays_under_the_cue_checks_tail() {
        let voiced = testing::run_voiced("14_one_idea").unwrap();
        let tail = voiced.tail_seconds();
        eprintln!("TAIL 14_one_idea: {tail:.2} s");
        assert!(tail <= 5.0, "14_one_idea: {tail:.2} s of picture after the last line");
    }
}
