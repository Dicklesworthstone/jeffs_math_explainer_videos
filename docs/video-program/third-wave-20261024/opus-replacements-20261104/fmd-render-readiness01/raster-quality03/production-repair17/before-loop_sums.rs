//! 08_loop_sums: the loop sum as a chain of product arrows laid tip to tail
//! (storyboard 08_1–08_5).
//!
//! - One step's product: the value f(z_k) times the step Δz_k, drawn as
//!   arrows from one origin (lengths multiply, angles add).
//! - f(z) = z: the 24 products curl into a closed polygon, walked twice.
//! - 1/z: every product points straight up; they stack into the 2πi column,
//!   and a bigger circle gives the very same column.
//! - The other whole powers close up; only z⁻¹ survives.
//! - Cauchy's theorem by tiling. The loop here is a square, so the small
//!   squares fill it exactly: each small loop integrates a constant plus a
//!   stretch and turn to zero, shared edges cancel, the outer loop is left;
//!   Goursat's lemma is the precise statement.

use std::f64::consts::TAU;

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Plane, arrow_head_points, axis_names, chain_arrows, chain_points, dot, fact, fade_in_all, fade_out_all, fit_chain,
    flat, hero, input, lay_down, line_style, output, precisely_now, reading_window, say, tween_linear, value,
    wait_until,
};
// the residues chapter's chain totals, the turning chapter's rule, 1/z's formula
use super::residues::TOTALS;
use super::turning::RULE;
use super::two_planes::FN_TEX;
use crate::model::{self, C, ZERO};

pub const SUM_TEX: &str = r"\oint f(z)\,dz \approx \sum_k f(z_k)\,\Delta z_k";
pub const SHRINK: &str = "as the steps shrink, ≈ becomes =";
pub const PART_TEX: [&str; 4] = [r"z_k", r"f(z_k)", r"\Delta z_k", r"f(z_k)\,\Delta z_k"];
pub const IDENTITY_TEX: &str = r"f(z) = z";
pub const UNIT_CIRCLE: &str = "unit circle";
pub const DISTANCE: &str = "distance from the start:";
/// The chain of 08_2 goes round twice: each product turns twice as fast as the walk.
pub const LAPS: [&str; 2] = ["the chain's first lap", "second lap, over the first"];
pub const CLOSED: &str = "|total| = ";
pub const UP: &str = "straight up";
pub const HEIGHT: &str = "column height:";
pub const STEPS_SAID: &str = " steps: ";
pub const LIMIT: &str = "limit: 2πi ≈ ";
pub const BIGGER: [&str; 2] = ["a bigger circle:", "the same column"];
pub const POWER_TEX: [&str; 6] = [r"z^{-3}", r"z^{-2}", r"z^{-1}", r"z^{0}", r"z^{1}", r"z^{2}"];
pub const SURVIVES: &str = "only 1/z survives";
pub const CAUCHY_TEX: &str = r"\oint_{\gamma} f(z)\,dz = 0";
pub const GAMMA_TEX: &str = r"\gamma";
pub const HYPOTHESIS: &str = "f holomorphic on the loop and inside it";
pub const LINEAR_TEX: &str = r"f(z) \approx a + b\,(z - z_{\,0})";
pub const EACH_TEX: &str = r"\oint (a + b\,(z - z_{\,0}))\,dz = 0";
pub const EACH: &str = "around each small square";
pub const CANCEL: [&str; 2] = ["shared edges cancel:", "only the outer loop is left"];
pub const GOURSAT: &str =
    "by Goursat's lemma each square's error shrinks faster than its area; no continuity of f' is assumed";

/// Steps around the loop (the facts' loop sums use the same count; the
/// tests hold the two together), and the finer cut of 08_1.
pub const STEPS: usize = 24;
pub const FINER: usize = 96;
/// One finer step as a share of one step of 08_1.
pub const FINER_STEP: f64 = STEPS as f64 / FINER as f64;
/// The step whose product is drawn in 08_1 and 08_3: its value, step and
/// product point in well-separated directions.
pub const K: usize = 3;
/// The circle's radius, and the bigger one of 08_3.
pub const R: f64 = 1.0;
pub const BIG_R: f64 = 2.0;
/// The powers of 08_4, in the storyboard's order.
pub const POWERS: [i32; 6] = [-3, -2, -1, 0, 1, 2];

/// The loop's plane (08_1–08_3): unit 1, room for radius 2.
pub fn circle_plane() -> Plane {
    Plane::new(v(-3.6, -0.4), 5.0, (-2.5, 2.5), (-2.5, 2.5))
}
/// One product's arrows share this origin, `PRODUCT_SCALE` scene units per unit.
pub const PRODUCT_AT: (f64, f64) = (2.2, -0.6);
pub const PRODUCT_SCALE: f64 = 2.4;
/// Where the chains of 08_2–08_3 are fitted: centre, box, largest scale.
pub const CHAIN_AT: (f64, f64) = (2.4, -0.4);
pub const CHAIN_BOX: (f64, f64) = (3.4, 4.8);
pub const CHAIN_MAX: f64 = 3.0;
/// The readout column on the right.
pub const SIDE_X: f64 = 5.3;
/// The row of 08_4: one centre per power, one common scale.
pub const ROW_X: [f64; 6] = [-5.25, -3.15, -1.05, 1.05, 3.15, 5.25];
pub const ROW_Y: f64 = -0.2;
pub const ROW_BOX: (f64, f64) = (1.8, 3.0);
pub const ROW_SCALE: f64 = 0.45;
/// The square loop of 08_5 and its tiling.
pub const SQUARE_CENTRE: (f64, f64) = (-2.6, -0.1);
pub const SQUARE_SIDE: f64 = 4.0;
pub const CELLS: usize = 4;
/// The flashed shared edge: between cell (col, row) and the cell to its right.
pub const SHARED: (usize, usize) = (1, 1);

/// Step `k` of `n` around |z| = `r`: its start, its end, and the arc's
/// midpoint, where f is read (as `model::loop_products` reads it).
pub fn step(r: f64, k: usize, n: usize) -> (C, C, C) {
    let pts = model::circle(ZERO, r, n);
    (pts[k], pts[(k + 1) % n], C::polar(r, TAU * (k as f64 + 0.5) / n as f64))
}

fn identity(z: C) -> C {
    z
}

fn inverse(z: C) -> C {
    z.recip()
}

/// 08_2's readout: how far the chain's end is from its start.
fn distance_of(z: C) -> f64 {
    z.abs()
}

/// 08_3's readout: how high the column has climbed.
fn height_of(z: C) -> f64 {
    z.im
}

/// f(z_k), Δz_k and their product, for step `k` of `STEPS` around |z| = `r`.
pub fn parts(f: fn(C) -> C, r: f64, k: usize) -> [C; 3] {
    let (a, b, mid) = step(r, k, STEPS);
    let w = f(mid);
    [w, b - a, w * (b - a)]
}

/// The tip-to-tail chain of `f` around |z| = `r`.
pub fn chain(f: fn(C) -> C, r: f64) -> Vec<C> {
    chain_points(&model::loop_products(|z| Some(f(z)), ZERO, r, STEPS).unwrap_or_default())
}

/// The chain of z^`p` around the unit circle.
pub fn power_chain(p: i32) -> Vec<C> {
    chain_points(&model::loop_products(|z| Some(z.powi(p)), ZERO, R, STEPS).unwrap_or_default())
}

/// A scene point of the product diagram.
fn product_point(w: C) -> Vec3 {
    v(PRODUCT_AT.0 + PRODUCT_SCALE * w.re, PRODUCT_AT.1 + PRODUCT_SCALE * w.im)
}

/// Where a product-diagram arrow's name sits: half a unit beyond its tip.
pub fn label_point(w: C) -> Vec3 {
    let tip = product_point(w);
    let d = w.scale(0.5 / w.abs());
    v(tip[0] + d.re, tip[1] + d.im)
}

/// One step's product as arrows from a common origin: the value, the
/// step, the product, each with its name.
fn product_diagram(kit: &Kit, stage: &mut Stage<'_>, f: fn(C) -> C) -> fmn::Result<Vec<Mob>> {
    let colors = [output(), input(), hero()];
    let mut out = Vec::new();
    for (j, (&w, &c)) in parts(f, R, K).iter().zip(&colors).enumerate() {
        let arrow = Arrow::new(product_point(ZERO), product_point(w))
            .thickness(4.0)
            .color(c)
            .buff(0.0)
            .build()?;
        let arrow = stage.arena_mut().add(arrow);
        let name = kit.tex(stage, PART_TEX[j + 1], 22.0, c, &[], label_point(w))?;
        fade_in_all(stage, &[arrow, name], 0.4)?;
        out.extend([arrow, name]);
    }
    Ok(out)
}

/// Each step of the loop as a lit chord, detached.
fn chords(stage: &mut Stage<'_>, p: &Plane, r: f64) -> fmn::Result<Vec<Mob>> {
    (0..STEPS)
        .map(|k| {
            let (a, b, _) = step(r, k, STEPS);
            p.path(stage, &[a, b], hero(), 5.0)
        })
        .collect()
}

/// The `n` step points as small dots, detached.
fn step_dots(stage: &mut Stage<'_>, p: &Plane, r: f64, n: usize, size: f64) -> Vec<Mob> {
    model::circle(ZERO, r, n)
        .into_iter()
        .map(|z| dot(stage, p.p(z), size, Palette::ink()))
        .collect()
}

/// Lay a chain's arrows one by one, each with its step lit on the circle,
/// `each` seconds apiece; `tracker` takes `readout` of the chain's corner
/// as each arrow lands.
fn lay_in_step(
    stage: &mut Stage<'_>,
    arrows: &[Mob],
    lit: &[Mob],
    corners: &[C],
    (tracker, readout): (Mob, fn(C) -> f64),
    each: f64,
) -> fmn::Result<()> {
    for (k, (&a, &l)) in arrows.iter().zip(lit).enumerate() {
        fade_in_all(stage, &[a, l], each)?;
        stage.set_tracker_value(tracker, readout(corners[k + 1]))?;
    }
    Ok(())
}

/// Where a circulation mark starts and stops, in radians, going anticlockwise.
const CIRCULATION: (f64, f64) = (0.4, 5.4);
/// A circulation head's length and width. It is narrower than long, so the tip is its one point;
/// a head as wide as it is long reads as a triangle aimed at whichever corner is lowest.
const CIRCULATION_HEAD: (f64, f64) = (0.16, 0.11);

/// The corners of a circulation mark's head, pointing on round the way it turns. A solid
/// head, since a chord of an arc this small is shorter than an `Arrow` can point. The middle of
/// its base lies on the arc, so the arc runs into the head along its axis, not along one edge.
pub fn circulation_head(at: Vec3, r: f64) -> Vec<Vec3> {
    let (length, width) = CIRCULATION_HEAD;
    let point = |a: f64| v(at[0] + r * a.cos(), at[1] + r * a.sin());
    let back = 2.0 * (length / (2.0 * r)).min(1.0).asin();
    let (tip, base) = (point(CIRCULATION.1), point(CIRCULATION.1 - back));
    let (ux, uy) = ((tip[0] - base[0]) / length, (tip[1] - base[1]) / length);
    let half = width / 2.0;
    vec![
        tip,
        v(base[0] - uy * half, base[1] + ux * half),
        v(base[0] + uy * half, base[1] - ux * half),
    ]
}

/// A small anticlockwise circulation arrow around `at`, detached.
fn circulation(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Vec<Mob> {
    let (a0, a1) = CIRCULATION;
    let point = |a: f64| v(at[0] + r * a.cos(), at[1] + r * a.sin());
    let arc: Vec<Vec3> = (0..=40).map(|j| point(a0 + (a1 - a0) * f64::from(j) / 40.0)).collect();
    let arc = stage
        .arena_mut()
        .add(polyline(arc).style(line_style(color, 3.0)).build());
    let tip = Polygon::new(circulation_head(at, r)).style(flat(color, 1.0)).build();
    vec![arc, stage.arena_mut().add(tip)]
}

/// The square loop's corners, anticlockwise from the bottom left, closed.
pub fn square_corners() -> [Vec3; 5] {
    let (cx, cy) = SQUARE_CENTRE;
    let h = SQUARE_SIDE / 2.0;
    [
        v(cx - h, cy - h),
        v(cx + h, cy - h),
        v(cx + h, cy + h),
        v(cx - h, cy + h),
        v(cx - h, cy - h),
    ]
}

/// The tiling's inner edges: the grid lines strictly inside the square.
pub fn inner_edges() -> Vec<[Vec3; 2]> {
    let [bl, _, tr, _, _] = square_corners();
    let cell = SQUARE_SIDE / CELLS as f64;
    let mut out = Vec::new();
    for i in 1..CELLS {
        let t = i as f64 * cell;
        out.push([v(bl[0] + t, bl[1]), v(bl[0] + t, tr[1])]);
        out.push([v(bl[0], bl[1] + t), v(tr[0], bl[1] + t)]);
    }
    out
}

/// The centre of cell (`col`, `row`), counted from the bottom left.
pub fn cell_centre(col: usize, row: usize) -> Vec3 {
    let [bl, ..] = square_corners();
    let cell = SQUARE_SIDE / CELLS as f64;
    v(bl[0] + (col as f64 + 0.5) * cell, bl[1] + (row as f64 + 0.5) * cell)
}

/// The two walks of the flashed shared edge, each a little off the edge on
/// its own square's side: up for the left square, down for the right one.
pub fn shared_walks() -> [[Vec3; 2]; 2] {
    let cell = SQUARE_SIDE / CELLS as f64;
    let c = cell_centre(SHARED.0, SHARED.1);
    let x = c[0] + cell / 2.0;
    let (y0, y1) = (c[1] - 0.4 * cell, c[1] + 0.4 * cell);
    [[v(x - 0.1, y0), v(x - 0.1, y1)], [v(x + 0.1, y1), v(x + 0.1, y0)]]
}

pub struct LoopSums {
    pub kit: Kit,
}

impl SceneConstruct for LoopSums {
    fn name(&self) -> &str {
        "08_loop_sums"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = circle_plane();
        let side = |y: f64| v(SIDE_X, y);

        // 08_1: walk the loop in steps; one step's product; the sum
        let line = say(&kit, stage, "08_1")?;
        let mut frame = vec![p.border(stage, Palette::faint())?];
        frame.extend(p.axes(stage)?);
        frame.extend(axis_names(&kit, stage, &p)?);
        let ring = p.circle(stage, ZERO, R, hero(), 3.0)?;
        // which circle, and which way round the walk goes
        let ring_name = kit.text(stage, UNIT_CIRCLE, 20.0, hero(), p.p(C::polar(1.35, -1.1)))?;
        let ring_head = stage.arena_mut().add(
            Polygon::new(arrow_head_points(p.p(C::polar(R, 2.5)), p.p(C::polar(R, 2.3)), 0.24))
                .style(flat(hero(), 1.0))
                .build(),
        );
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &[ring, ring_head, ring_name], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let dots = step_dots(stage, &p, R, STEPS, 0.05);
        lay_down(stage, &dots, 0.05)?;
        line.wait_sentence(stage, 2)?;
        let (a, b, mid) = step(R, K, STEPS);
        let chord = Arrow::new(p.p(a), p.p(b))
            .thickness(4.0)
            .color(input())
            .buff(0.0)
            .build()?;
        let chord = stage.arena_mut().add(chord);
        let at_mid = dot(stage, p.p(mid), 0.07, output());
        let mid_name = kit.tex(stage, PART_TEX[0], 22.0, output(), &[], p.p(mid.scale(1.35)))?;
        fade_in_all(stage, &[chord, at_mid, mid_name], 0.4)?;
        let product = product_diagram(&kit, stage, identity)?;
        let rule = kit.text(stage, RULE, 20.0, Palette::dim(), v(PRODUCT_AT.0, -3.35))?;
        fade_in_all(stage, &[rule], 0.3)?;
        line.wait_sentence(stage, 3)?;
        let sum = kit.tex(stage, SUM_TEX, 32.0, Palette::ink(), &[], v(0.0, 3.15))?;
        fade_in_all(stage, &[sum], 0.5)?;
        line.wait_sentence(stage, 4)?;
        let fine = step_dots(stage, &p, R, FINER, 0.03);
        fade_out_all(stage, &dots, 0.3)?;
        fade_in_all(stage, &fine, 0.5)?;
        // the one step drawn shrinks with the rest, to the finer cut's size, its product with it
        let [_, dz, prod] = parts(identity, R, K);
        let finer = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let bits = product.clone();
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = value(s, finer);
                let (centre, half) = ((a + b).scale(0.5), (b - a).scale(0.5 * t));
                if let Ok(m) = Arrow::new(p.p(centre - half), p.p(centre + half))
                    .thickness(4.0)
                    .buff(0.0)
                    .build()
                {
                    let _ = s.set_points(chord, m.points());
                }
                for (j, w) in [(2, dz), (4, prod)] {
                    let tip = w.scale(t);
                    if let Ok(m) = Arrow::new(product_point(ZERO), product_point(tip))
                        .thickness(4.0)
                        .buff(0.0)
                        .build()
                    {
                        let _ = s.set_points(bits[j], m.points());
                    }
                    s.move_to(bits[j + 1], label_point(tip), ORIGIN);
                }
            },
            true,
        )?;
        stage.play(tween_linear(finer, FINER_STEP, 0.9)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        let shrink = kit.text(stage, SHRINK, 20.0, Palette::dim(), v(0.0, 2.55))?;
        fade_in_all(stage, &[shrink], 0.4)?;
        wait_until(stage, line.end())?;

        // 08_2: f(z) = z: the products laid tip to tail curl round and close
        kit.hold(stage)?;
        let line = say(&kit, stage, "08_2")?;
        let mut gone = vec![chord, at_mid, mid_name, rule, shrink];
        gone.extend(&product);
        gone.extend(&fine);
        fade_out_all(stage, &gone, 0.4)?;
        let dots = step_dots(stage, &p, R, STEPS, 0.05);
        fade_in_all(stage, &dots, 0.3)?;
        let named = kit.tex(stage, IDENTITY_TEX, 30.0, Palette::ink(), &[], side(1.8))?;
        fade_in_all(stage, &[named], 0.4)?;
        let corners = chain(identity, R);
        let (origin, scale) = fit_chain(&corners, v(CHAIN_AT.0, CHAIN_AT.1), CHAIN_BOX.0, CHAIN_BOX.1, CHAIN_MAX);
        // each product turns twice as fast as the walk, so the chain goes round twice: the
        // second lap is laid over the first in its own colour, and the picture keeps moving
        let half = STEPS / 2;
        let first_lap = chain_arrows(stage, origin, scale, &corners[..=half], hero())?;
        let second_lap = chain_arrows(stage, origin, scale, &corners[half..], Palette::ink())?;
        let laps = [
            kit.text(stage, LAPS[0], 20.0, hero(), side(-0.4))?,
            kit.text(stage, LAPS[1], 20.0, Palette::ink(), side(-0.4))?,
        ];
        let lit = chords(stage, &p, R)?;
        let distance = stage.add_value_tracker(0.0);
        let label = kit.text(stage, DISTANCE, 20.0, Palette::dim(), side(0.9))?;
        fade_in_all(stage, &[label], 0.3)?;
        let live = kit.live_value(
            stage,
            move |s| value(s, distance),
            2,
            false,
            28.0,
            hero(),
            side(0.3),
            ORIGIN,
        )?;
        // the lap captions' fades come out of the time the chain has
        let each = ((line.until(stage, "The total is zero", 2.4)? - 0.35) / STEPS as f64).clamp(0.08, 0.3);
        fade_in_all(stage, &[laps[0]], 0.15)?;
        lay_in_step(stage, &first_lap, &lit[..half], &corners, (distance, distance_of), each)?;
        fade_out_all(stage, &[laps[0]], 0.1)?;
        fade_in_all(stage, &[laps[1]], 0.1)?;
        lay_in_step(
            stage,
            &second_lap,
            &lit[half..],
            &corners[half..],
            (distance, distance_of),
            each,
        )?;
        let mut arrows = first_lap;
        arrows.extend(second_lap);
        line.wait_sentence(stage, 2)?;
        // the total replaces the running distance, and the distance's label goes with it
        fade_out_all(stage, &[label, laps[1]], 0.2)?;
        stage.clear_updaters(live, true);
        let total = kit.text(
            stage,
            &format!("{CLOSED}{}", fact("loop.z.total")?),
            26.0,
            hero(),
            side(0.3),
        )?;
        stage.remove_from_scene(live);
        stage.add_to_scene(total)?;
        let start = dot(stage, origin, 0.08, Palette::ink());
        fade_in_all(stage, &[start], 0.3)?;
        wait_until(stage, line.end())?;

        // 08_3: 1/z: every product points straight up; they stack into 2πi
        kit.hold(stage)?;
        let line = say(&kit, stage, "08_3")?;
        // the circle is about to stop being the unit circle
        let mut gone = vec![named, total, start, ring_name, ring_head];
        gone.extend(&arrows);
        gone.extend(&lit);
        fade_out_all(stage, &gone, 0.4)?;
        let named = kit.tex(stage, FN_TEX[1], 30.0, Palette::ink(), &[], side(1.8))?;
        fade_in_all(stage, &[named], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let product = product_diagram(&kit, stage, inverse)?;
        let [_, _, up] = parts(inverse, R, K);
        let up_at = product_point(up);
        let up_says = kit.text(stage, UP, 20.0, hero(), v(up_at[0] + 1.1, up_at[1]))?;
        // the position whose turn the voice says is undone: 08_1's dot and name on the same step,
        // in this fade so no beat moves, and gone before the circle grows away from them
        let position = dot(stage, p.p(mid), 0.07, output());
        let position_name = kit.tex(stage, PART_TEX[0], 22.0, output(), &[], p.p(mid.scale(1.35)))?;
        fade_in_all(stage, &[up_says, position, position_name], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let mut gone = product.clone();
        gone.extend([up_says, position, position_name]);
        fade_out_all(stage, &gone, 0.3)?;
        let corners = chain(inverse, R);
        let (origin, scale) = fit_chain(&corners, v(CHAIN_AT.0, CHAIN_AT.1), CHAIN_BOX.0, CHAIN_BOX.1, CHAIN_MAX);
        let arrows = chain_arrows(stage, origin, scale, &corners, hero())?;
        let lit = chords(stage, &p, R)?;
        let height = stage.add_value_tracker(0.0);
        let label = kit.text(stage, HEIGHT, 20.0, Palette::dim(), side(0.9))?;
        fade_in_all(stage, &[label], 0.3)?;
        let live = kit.live_value(
            stage,
            move |s| value(s, height),
            2,
            false,
            28.0,
            hero(),
            side(0.3),
            ORIGIN,
        )?;
        let each = (line.until(stage, "As the steps shrink", 2.4)? / STEPS as f64).clamp(0.06, 0.25);
        lay_in_step(stage, &arrows, &lit, &corners, (height, height_of), each)?;
        line.wait_sentence(stage, 3)?;
        stage.clear_updaters(live, true);
        let total = kit.text(
            stage,
            &format!("{STEPS}{STEPS_SAID}{}i", fact("loop.inv.total24")?),
            24.0,
            hero(),
            side(0.3),
        )?;
        stage.remove_from_scene(live);
        stage.add_to_scene(total)?;
        let limit = kit.text(
            stage,
            &format!("{LIMIT}{}i", fact("loop.two_pi")?),
            24.0,
            hero(),
            side(-0.4),
        )?;
        fade_in_all(stage, &[limit], 0.4)?;
        line.wait_for(stage, "whatever the size")?;
        // a bigger circle: the products, and so the column, do not change
        let grow = stage.add_value_tracker(R);
        let moving_dots = dots.clone();
        let moving_lit = lit.clone();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, grow);
                let mut round: Vec<Vec3> = model::circle(ZERO, r, 240).into_iter().map(|z| p.p(z)).collect();
                round.push(round[0]);
                let _ = a.set_points(ring, polyline(round).build().points());
                for (k, (&d, &l)) in moving_dots.iter().zip(&moving_lit).enumerate() {
                    let (s, e, _) = step(r, k, STEPS);
                    a.move_to(d, p.p(s), ORIGIN);
                    let _ = a.set_points(l, polyline(vec![p.p(s), p.p(e)]).build().points());
                }
            },
            true,
        )?;
        stage.play(tween(grow, BIG_R, 1.6)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        let bigger = [
            kit.text(stage, BIGGER[0], 20.0, Palette::dim(), side(-1.3))?,
            kit.text(stage, BIGGER[1], 20.0, Palette::dim(), side(-1.7))?,
        ];
        fade_in_all(stage, &bigger, 0.4)?;
        wait_until(stage, line.end())?;

        // 08_4: every other whole power closes up; only z⁻¹ survives
        kit.hold(stage)?;
        let line = say(&kit, stage, "08_4")?;
        let mut gone = vec![ring, named, label, total, limit];
        gone.extend(&frame);
        gone.extend(&dots);
        gone.extend(&arrows);
        gone.extend(&lit);
        gone.extend(bigger);
        fade_out_all(stage, &gone, 0.4)?;
        let closed: Vec<i32> = fact("loop.powers.closed")?
            .split(',')
            .map(|s| {
                s.parse::<i32>()
                    .map_err(|e| super::draw::fail(format!("loop.powers.closed: {e}")))
            })
            .collect::<fmn::Result<_>>()?;
        let mut row = Vec::new();
        let mut names = Vec::new();
        for (j, &x) in ROW_X.iter().enumerate() {
            let n = kit.tex(stage, POWER_TEX[j], 28.0, Palette::ink(), &[], v(x, 1.9))?;
            names.push(n);
        }
        fade_in_all(stage, &names, 0.4)?;
        row.extend(&names);
        let mut survivor = None;
        for (j, (&x, &pw)) in ROW_X.iter().zip(&POWERS).enumerate() {
            let corners = power_chain(pw);
            let (origin, scale) = fit_chain(&corners, v(x, ROW_Y), ROW_BOX.0, ROW_BOX.1, ROW_SCALE);
            if closed.contains(&pw) {
                let arrows = chain_arrows(stage, origin, scale, &corners, Palette::ink())?;
                lay_down(stage, &arrows, 0.03)?;
                let says = kit.text(stage, TOTALS[0], 20.0, Palette::dim(), v(x, -2.25))?;
                fade_in_all(stage, &[says], 0.2)?;
                row.extend(arrows);
                row.push(says);
                if j == 1 {
                    line.wait_sentence(stage, 1)?;
                }
            } else {
                survivor = Some((x, origin, scale, corners));
            }
        }
        line.wait_sentence(stage, 2)?;
        if let Some((x, origin, scale, corners)) = survivor {
            let arrows = chain_arrows(stage, origin, scale, &corners, hero())?;
            lay_down(stage, &arrows, 0.04)?;
            let says = kit.text(stage, TOTALS[1], 24.0, hero(), v(x, -2.25))?;
            let (w, h) = (ROW_BOX.0, 4.8);
            let boxed: Vec<Vec3> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)]
                .into_iter()
                .map(|(sx, sy)| v(x + sx * w / 2.0, -0.15 + sy * h / 2.0))
                .collect();
            let boxed = stage
                .arena_mut()
                .add(polyline(boxed).style(line_style(hero(), 4.0)).build());
            let survives = kit.text(stage, SURVIVES, 22.0, hero(), v(x, -3.0))?;
            fade_in_all(stage, &[says, boxed], 0.4)?;
            fade_in_all(stage, &[survives], 0.4)?;
            row.extend(arrows);
            row.extend([says, boxed, survives]);
        }
        wait_until(stage, line.end())?;

        // 08_5: Cauchy's theorem: cut the inside into squares; inner edges cancel
        kit.hold(stage)?;
        let line = say(&kit, stage, "08_5")?;
        row.push(sum);
        fade_out_all(stage, &row, 0.5)?;
        let right = 3.6;
        let cauchy = kit.tex(stage, CAUCHY_TEX, 34.0, Palette::ink(), &[], v(right, 1.6))?;
        fade_in_all(stage, &[cauchy], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let corners = square_corners();
        let outer = stage
            .arena_mut()
            .add(polyline(corners.to_vec()).style(line_style(hero(), 5.0)).build());
        let gamma = kit.tex(
            stage,
            GAMMA_TEX,
            28.0,
            hero(),
            &[],
            v(corners[1][0] + 0.3, corners[1][1] + 0.3),
        )?;
        let walk_on = Arrow::new(
            v(SQUARE_CENTRE.0 - 0.4, corners[0][1]),
            v(SQUARE_CENTRE.0 + 0.4, corners[0][1]),
        )
        .thickness(5.0)
        .color(hero())
        .buff(0.0)
        .build()?;
        let walk_on = stage.arena_mut().add(walk_on);
        let hypothesis = kit.text(stage, HYPOTHESIS, 20.0, Palette::dim(), v(right, 0.95))?;
        fade_in_all(stage, &[outer, gamma, walk_on], 0.5)?;
        fade_in_all(stage, &[hypothesis], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let inner: Vec<Mob> = inner_edges()
            .into_iter()
            .map(|[a, b]| {
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(line_style(Palette::dim(), 3.0)).build())
            })
            .collect();
        fade_in_all(stage, &inner, 0.6)?;
        line.wait_sentence(stage, 4)?;
        let mut turning = Vec::new();
        for row_k in 0..CELLS {
            for col in 0..CELLS {
                turning.extend(circulation(stage, cell_centre(col, row_k), 0.26, input()));
            }
        }
        fade_in_all(stage, &turning, 0.6)?;
        let linear = kit.tex(stage, LINEAR_TEX, 28.0, Palette::ink(), &[], v(right, 0.0))?;
        let each_zero = kit.tex(stage, EACH_TEX, 28.0, Palette::ink(), &[], v(right, -0.7))?;
        let each = kit.text(stage, EACH, 20.0, Palette::dim(), v(right, -1.25))?;
        fade_in_all(stage, &[linear], 0.4)?;
        fade_in_all(stage, &[each_zero, each], 0.4)?;
        line.wait_sentence(stage, 5)?;
        let colors = [input(), output()];
        let mut walks = Vec::new();
        for ([a, b], &c) in shared_walks().into_iter().zip(&colors) {
            let w = Arrow::new(a, b).thickness(4.0).color(c).buff(0.0).build()?;
            walks.push(stage.arena_mut().add(w));
        }
        fade_in_all(stage, &walks, 0.4)?;
        line.wait_for(stage, "so the inner edges cancel")?;
        let mut cancel = walks.clone();
        cancel.extend(&inner);
        cancel.extend(&turning);
        fade_out_all(stage, &cancel, 0.8)?;
        line.wait_for(stage, "only the outer loop is left")?;
        let left = [
            kit.text(stage, CANCEL[0], 20.0, hero(), v(right, -1.9))?,
            kit.text(stage, CANCEL[1], 20.0, hero(), v(right, -2.3))?,
        ];
        let lit_outer = stage
            .arena_mut()
            .add(polyline(corners.to_vec()).style(line_style(hero(), 9.0)).build());
        fade_in_all(stage, &[lit_outer], 0.4)?;
        fade_in_all(stage, &left, 0.4)?;
        wait_until(stage, line.end())?;
        // the lemma behind "almost": no voice names it, and from "Neighbouring squares" on
        // the voice and the picture are both on the shared edges, so it comes when they stop
        let exact = precisely_now(&kit, stage, GOURSAT)?;
        reading_window(&kit, stage, &line)?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            cauchy, outer, gamma, walk_on, hypothesis, linear, each_zero, each, exact, lit_outer,
        ];
        all.extend(left);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::fixed;

    fn get(key: &str) -> &'static str {
        fact(key).unwrap()
    }

    #[test]
    fn the_drawn_products_are_the_loop_sums_products() {
        for f in [identity as fn(C) -> C, inverse] {
            let products = model::loop_products(|z| Some(f(z)), ZERO, R, STEPS).unwrap();
            for (k, &want) in products.iter().enumerate() {
                let [w, dz, got] = parts(f, R, k);
                assert!((got - want).abs() < 1e-15 && (w * dz - got).abs() < 1e-15);
            }
        }
        // the drawn step: lengths multiply, angles add
        let [w, dz, prod] = parts(identity, R, K);
        assert!((prod.abs() - w.abs() * dz.abs()).abs() < 1e-12);
        let turn = (prod.arg() - w.arg() - dz.arg()).rem_euclid(TAU);
        assert!(turn < 1e-12 || TAU - turn < 1e-12);
        // 1/z's drawn product points straight up
        let [_, _, up] = parts(inverse, R, K);
        assert!(up.re.abs() < 1e-12 && up.im > 0.0);
        // the three directions are well apart, so their names do not collide
        for f in [identity as fn(C) -> C, inverse] {
            let names: Vec<Vec3> = parts(f, R, K).into_iter().map(label_point).collect();
            for (i, a) in names.iter().enumerate() {
                for (j, b) in names.iter().enumerate().skip(i + 1) {
                    assert!((a[0] - b[0]).hypot(a[1] - b[1]) > 0.8, "{i} {j}");
                }
            }
        }
    }

    #[test]
    fn the_chains_say_what_the_facts_say() {
        let z = chain(identity, R);
        assert_eq!(fixed(z[STEPS].abs(), 6), get("loop.z.total"));
        let inv = chain(inverse, R);
        assert_eq!(fixed(inv[STEPS].im, 2), get("loop.inv.total24"));
        assert_eq!(get("loop.inv.all_up"), "true");
        // the bigger circle's column is the same column
        let big = chain(inverse, BIG_R);
        for (a, b) in inv.iter().zip(&big) {
            assert!((*a - *b).abs() < 1e-12);
        }
        // the closed powers are the ones the facts list; only z⁻¹ is left
        let closed: Vec<i32> = get("loop.powers.closed")
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        for p in POWERS {
            let end = power_chain(p)[STEPS];
            assert_eq!(closed.contains(&p), end.abs() < 1e-12, "z^{p}");
        }
        assert_eq!(
            POWERS.iter().filter(|p| !closed.contains(p)).collect::<Vec<_>>(),
            vec![&-1]
        );
        assert!((TAU - get("loop.two_pi").parse::<f64>().unwrap()).abs() < 5e-3);
    }

    #[test]
    fn the_chain_of_z_is_back_at_its_start_after_half_the_steps() {
        let pts = chain(identity, R);
        assert_eq!(pts.len(), STEPS + 1);
        assert!((pts[STEPS / 2] - pts[0]).abs() < 1e-9);
        // and so the second lap lies over the first
        for k in 0..=STEPS / 2 {
            assert!((pts[k + STEPS / 2] - pts[k]).abs() < 1e-9);
        }
    }

    #[test]
    fn a_circulation_mark_points_the_way_it_turns() {
        let at = v(1.0, 2.0);
        let head = circulation_head(at, 0.26);
        let (tip, back) = (
            head[0],
            [(head[1][0] + head[2][0]) / 2.0, (head[1][1] + head[2][1]) / 2.0],
        );
        let (rx, ry) = (tip[0] - at[0], tip[1] - at[1]);
        let (dx, dy) = (tip[0] - back[0], tip[1] - back[1]);
        assert!(rx * dy - ry * dx > 0.0, "the head points clockwise");
    }

    /// Picture review 08-N2: the sign was right and the head still read as aimed downward,
    /// because it was as wide as long and the arc entered it along one edge.
    #[test]
    fn a_circulation_head_has_one_point_and_sits_on_its_arc() {
        let (at, r) = (v(1.0, 2.0), 0.26);
        let head = circulation_head(at, r);
        let [tip, left, right] = [head[0], head[1], head[2]];
        let angle_at = |q: Vec3, a: Vec3, b: Vec3| {
            let (ax, ay, bx, by) = (a[0] - q[0], a[1] - q[1], b[0] - q[0], b[1] - q[1]);
            (ax * by - ay * bx).abs().atan2(ax * bx + ay * by).to_degrees()
        };
        let (sharp, blunt) = (angle_at(tip, left, right), angle_at(left, tip, right));
        assert!(sharp < 40.0 && blunt > 65.0, "tip {sharp}, base {blunt}");
        let on_arc = |x: f64, y: f64| ((x - at[0]).hypot(y - at[1]) - r).abs() < 1e-9;
        let (base_x, base_y) = ((left[0] + right[0]) / 2.0, (left[1] + right[1]) / 2.0);
        assert!(on_arc(tip[0], tip[1]) && on_arc(base_x, base_y));
        // and it keeps its size: the base's middle is one head length back from the tip
        assert!(((tip[0] - base_x).hypot(tip[1] - base_y) - CIRCULATION_HEAD.0).abs() < 1e-9);
    }

    /// Read from the source, because no render can run here. Picture review 08-N1: the voice
    /// says dividing by z undoes the turn of the step's position, so that position is on the
    /// ring for those words. It leaves before the ring grows away from it.
    #[test]
    fn the_position_is_on_the_ring_while_the_voice_undoes_its_turn() {
        let source = include_str!("loop_sums.rs");
        let cue = source.find("let line = say(&kit, stage, \"08_3\")?;").unwrap();
        let scene = &source[cue..];
        let said = scene.find("line.wait_sentence(stage, 1)?;").unwrap();
        let arrives = scene
            .find("fade_in_all(stage, &[up_says, position, position_name], 0.3)?;")
            .unwrap();
        let held = scene.find("line.wait_sentence(stage, 2)?;").unwrap();
        let leaves = scene.find("gone.extend([up_says, position, position_name]);").unwrap();
        let grows = scene.find("stage.play(tween(grow, BIG_R, 1.6)?)?;").unwrap();
        assert!(said < arrives && arrives < held && held < leaves && leaves < grows);
        // and what it shows is true: the step runs a quarter turn ahead of the position, f's
        // arrow is the position's turn undone, and what is left points straight up
        let (_, _, mid) = step(R, K, STEPS);
        let [w, dz, up] = parts(inverse, R, K);
        assert!((mid.abs() - R).abs() < 1e-12);
        assert!((w.arg() + mid.arg()).abs() < 1e-12);
        assert!((dz.arg() - mid.arg() - TAU / 4.0).abs() < 1e-12);
        assert!((up.arg() - TAU / 4.0).abs() < 1e-12);
        assert!(circle_plane().contains(mid.scale(1.35)));
    }

    #[test]
    fn the_pictures_fit_and_no_arrow_is_dropped() {
        let p = circle_plane();
        assert!(p.contains(C::real(BIG_R)) && p.contains(C::new(0.0, -BIG_R)));
        assert!(p.frame.top() < 2.3 && p.frame.bottom() > -3.0 && p.frame.center[0] + p.frame.width / 2.0 < 0.0);
        let inside = |q: Vec3| q[0].abs() < 6.9 && q[1].abs() < 3.8;
        // the product diagrams stay right of the plane, under the formula
        for f in [identity as fn(C) -> C, inverse] {
            for w in parts(f, R, K) {
                let (tip, name) = (product_point(w), label_point(w));
                assert!(tip[0] > -0.8 && name[0] > -0.3 && name[1] < 2.3 && inside(name));
            }
        }
        // the chains: every arrow long enough to draw, inside their box
        for (f, max) in [(identity as fn(C) -> C, CHAIN_MAX), (inverse, 1.0)] {
            let pts = chain(f, R);
            let (o, s) = fit_chain(&pts, v(CHAIN_AT.0, CHAIN_AT.1), CHAIN_BOX.0, CHAIN_BOX.1, CHAIN_MAX);
            assert!(s <= max);
            for w in pts.windows(2) {
                assert!((w[1] - w[0]).abs() * s > 0.05);
            }
            for z in &pts {
                let q = super::super::draw::chain_at(o, s, *z);
                assert!(q[1] < 2.2 && q[1] > -3.0 && q[0] > 0.0 && q[0] < SIDE_X - 1.3);
            }
        }
        // the row: one common scale, each chain in its own column
        for (j, p) in POWERS.into_iter().enumerate() {
            let pts = power_chain(p);
            let (o, s) = fit_chain(&pts, v(ROW_X[j], ROW_Y), ROW_BOX.0, ROW_BOX.1, ROW_SCALE);
            assert_eq!(s, ROW_SCALE);
            for z in &pts {
                let q = super::super::draw::chain_at(o, s, *z);
                assert!((q[0] - ROW_X[j]).abs() < 0.9 && q[1] < 1.6 && q[1] > -2.0);
            }
            for w in pts.windows(2) {
                assert!((w[1] - w[0]).abs() * s > 1e-3);
            }
        }
        // the tiling: inside the frame, clear of a two-line lower third
        for q in square_corners() {
            assert!(inside(q) && q[1] > -2.7 && q[0] < 0.0);
        }
        assert_eq!(inner_edges().len(), 2 * (CELLS - 1));
        let [up, down] = shared_walks();
        assert!(up[1][1] > up[0][1] && down[1][1] < down[0][1] && up[0][0] < down[0][0]);
        assert!(SHARED.0 + 1 < CELLS && SHARED.1 < CELLS);
    }
}
