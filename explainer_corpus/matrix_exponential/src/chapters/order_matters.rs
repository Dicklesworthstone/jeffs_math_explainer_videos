//! Chapter 14, order matters: for numbers, growing at two shares at once is
//! growing at one after the other, so e to the sum is e to the first times
//! e to the second (14_1). Two matrices, both shears: A pushes the top of a
//! square to the right, B pushes its right side upward (14_2). Your turn:
//! follow A then B, or B then A; is the square in the same place (14_3)? No,
//! they are mirror images, and both rules at once lands between them: three
//! different results (14_4). Multiply the two grids in either order: where
//! AB and BA agree the matrices commute, and for those the rule holds
//! (14_5). The failure can be measured: go round a loop, A, B, undo each; the
//! square does not come back, and what is left over is set by AB minus BA,
//! the commutator (14_6).
//!
//! Every number is a fact: the rules `F-O.A` and `F-O.B`, the three results
//! `F-O.expB_expA`, `F-O.expA_expB` and `F-V.14_4.both_at_once`, the products
//! `F-O.AB` and `F-O.BA`, the loop `F-O.loop.*`, the commutator
//! `F-O.commutator`, the quiz answer `F-Q.14_3.answer`.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, Plane, QuizCard, arrow_points, chip, corners, entries, fade_in_all, fade_out_all, faded_line,
    fit_width, flat, good, music_hold, result, rule, say, second, substance, tween_linear, warn,
};
use super::flow::{fade_in_anim, fade_out_anim};
use crate::facts::{shear_right, shear_up, shown};
use crate::model::{Mat, expm_t};

// ------------------------------------------------------------------ choices

/// The plane the shapes live on: its origin at the lower left, so the
/// parallelograms (up to three units out) stay on the left of the frame.
const PLANE: Plane = Plane {
    centre: [-4.6, -2.2, 0.0],
    unit: 1.25,
};
/// 14_1: the two shares, for the bars (growth factors e^a and e^b).
const SHARE_A: f64 = 0.5;
const SHARE_B: f64 = 0.4;
/// The bars of 14_1: scene units for a factor of one, and where they stand.
const BAR_UNIT: f64 = 1.05;
const BAR_BASE: f64 = -2.4;
const BAR_X: [f64; 2] = [-2.4, 1.6];
/// 14_5: the pair of 3 and 4 full turns, which does not commute and still
/// obeys the rule (their sum is 5 full turns about the 3-4-5 axis).
const TURNS: (u32, u32) = (3, 4);
/// Where the grids sit in the right column.
const GRID_AT: Vec3 = [4.2, 1.2, 0.0];

// ------------------------------------------------------------- the geometry

/// The unit square's corners, counter-clockwise from the origin.
pub fn square() -> [[f64; 2]; 4] {
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

/// The corners of the square under `m`, on screen.
pub fn corners_under(plane: &Plane, m: &Mat) -> Vec<Vec3> {
    square()
        .iter()
        .map(|c| {
            let p = m.apply(c);
            plane.p(p[0], p[1])
        })
        .collect()
}

/// The pose of the square after following `first` and then `second`, each
/// for a time of one, at progress `p` from 0 to 2 (the later motion is
/// written on the left).
pub fn then_pose(first: &Mat, second: &Mat, p: f64) -> Mat {
    let p = p.clamp(0.0, 2.0);
    if p <= 1.0 {
        expm_t(first, p)
    } else {
        expm_t(second, p - 1.0).mul(&expm_t(first, 1.0))
    }
}

/// The pose after following both rules at once (their sum) for a time `p`
/// from 0 to 1: the limit of a tiny step of each, over and over.
pub fn both_pose(a: &Mat, b: &Mat, p: f64) -> Mat {
    expm_t(&a.add(b), p.clamp(0.0, 1.0))
}

/// The pose in the loop of 14_6 at progress `p` from 0 to 4: A for a time
/// `t`, then B, then A undone, then B undone, the later motion on the left.
pub fn loop_pose(t: f64, p: f64) -> Mat {
    let (a, b) = (shear_right(), shear_up());
    let p = p.clamp(0.0, 4.0);
    let stage = (p.floor() as usize).min(3);
    let f = p - stage as f64;
    let after_a = expm_t(&a, t);
    let after_b = expm_t(&b, t).mul(&after_a);
    let after_unda = expm_t(&a, -t).mul(&after_b);
    match stage {
        0 => expm_t(&a, f * t),
        1 => expm_t(&b, f * t).mul(&after_a),
        2 => expm_t(&a, -f * t).mul(&after_b),
        _ => expm_t(&b, -f * t).mul(&after_unda),
    }
}

/// The three bars of 14_1 at progress `v` from 0 to 1: heights, in units of
/// the starting amount, of (the sequential bar's first stretch top, its
/// second stretch top, the parallel bar's top). The sequential bar spends
/// the first half of `v` at share a and the second at share b; the parallel
/// bar grows at a + b all the way.
pub fn bar_tops(v: f64) -> (f64, f64, f64) {
    let v = v.clamp(0.0, 1.0);
    let first = (SHARE_A * (2.0 * v).min(1.0)).exp();
    let second = first * (SHARE_B * (2.0 * v - 1.0).max(0.0)).exp();
    let both = ((SHARE_A + SHARE_B) * v).exp();
    (first, second, both)
}

/// A rectangle's corners standing on `base` at `x`, `w` wide, from `lo` to
/// `hi` scene units above it.
fn column(x: f64, w: f64, base: f64, lo: f64, hi: f64) -> Vec<Vec3> {
    let top = base + hi.max(lo + 0.004);
    // a closed run of vertices, to be set through `corners`
    vec![
        v(x - w / 2.0, base + lo),
        v(x + w / 2.0, base + lo),
        v(x + w / 2.0, top),
        v(x - w / 2.0, top),
        v(x - w / 2.0, base + lo),
    ]
}

// ------------------------------------------------------------ the pictures

/// A square whose pose follows `pose(progress)` of a tracker.
struct Moving {
    poly: Mob,
    progress: Mob,
    anchor: Mob,
}

impl Moving {
    fn build(stage: &mut Stage<'_>, color: Srgb, pose: impl Fn(f64) -> Mat + 'static) -> fmn::Result<Self> {
        let start = corners_under(&PLANE, &pose(0.0));
        let poly = Polygon::new(start)
            .style(Style::default().fill(color, 0.28).stroke(color, 4.0, 1.0))
            .build();
        let poly = stage.arena_mut().add(poly);
        let progress = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let p = s.tracker_value(progress).unwrap_or(0.0);
                if p.to_bits() == last.to_bits() {
                    return;
                }
                last = p;
                let _ = s.set_points(poly, Polygon::new(corners_under(&PLANE, &pose(p))).build().points());
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { poly, progress, anchor })
    }
}

/// The unit square drawn still, as an outline.
fn outline(stage: &mut Stage<'_>, plane: &Plane, color: Srgb) -> Mob {
    let o = Polygon::new(corners_under(plane, &Mat::identity(2)))
        .style(Style::default().fill(color, 0.0).stroke(color, 3.0, 1.0))
        .build();
    stage.arena_mut().add(o)
}

/// A field of arrows of the rule `m` over the unit square and a margin, on
/// `plane`, each `gain` times the rule's velocity there.
fn field(stage: &mut Stage<'_>, plane: &Plane, m: &Mat, color: Srgb, gain: f64) -> Vec<Mob> {
    let mut out = Vec::new();
    for i in 0..5 {
        for j in 0..5 {
            let p = [0.1 + 0.2 * f64::from(i), 0.1 + 0.2 * f64::from(j)];
            let w = m.apply(&p);
            let (a, b) = (plane.p(p[0], p[1]), plane.p(p[0] + gain * w[0], p[1] + gain * w[1]));
            let arrow = polyline(arrow_points(a, b, 0.12))
                .style(faded_line(color, 3.0, 1.0))
                .build();
            out.push(stage.arena_mut().add(arrow));
        }
    }
    out
}

/// The name over the grid of 14_4's three results.
fn result_name(kit: &Kit, stage: &mut Stage<'_>, words: &str, c: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, words, 30.0, c, v(GRID_AT[0] + 0.1, GRID_AT[1] + 1.55))
}

/// A bar of 14_1, standing on the baseline at `x`, as tall as the starting
/// amount to begin with.
fn bar(stage: &mut Stage<'_>, x: f64, c: Srgb) -> Mob {
    let p = Polygon::new(column(x, 1.3, BAR_BASE, 0.0, BAR_UNIT))
        .style(flat(c, 0.85))
        .build();
    stage.arena_mut().add(p)
}

/// Rows of whole numbers from a fact, as grid entries.
fn whole_entries(key: &str) -> fmn::Result<Vec<Vec<String>>> {
    entries(key, None)
}

// -------------------------------------------------------------- the scene

pub struct OrderMatters {
    pub kit: Kit,
}

impl SceneConstruct for OrderMatters {
    fn name(&self) -> &str {
        "14_order_matters"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, rose, gold, amber) = (Palette::ink(), rule(), second(), result(), substance());
        let (a, b) = (shear_right(), shear_up());

        // ---- 14_1: numbers: two shares at once, or one after the other
        let line = say(&kit, stage, "14_1")?;
        let seq_base = bar(stage, BAR_X[0], amber);
        let seq_a = bar(stage, BAR_X[0], teal);
        let seq_b = bar(stage, BAR_X[0], rose);
        let par = bar(stage, BAR_X[1], amber);
        let progress = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let u = s.tracker_value(progress).unwrap_or(0.0);
                if u.to_bits() == last.to_bits() {
                    return;
                }
                last = u;
                let (first, second_top, both) = bar_tops(u);
                let at = |h: f64| h * BAR_UNIT;
                let _ = s.set_points(seq_base, &corners(&column(BAR_X[0], 1.3, BAR_BASE, 0.0, at(1.0))));
                let _ = s.set_points(seq_a, &corners(&column(BAR_X[0], 1.3, BAR_BASE, at(1.0), at(first))));
                let _ = s.set_points(
                    seq_b,
                    &corners(&column(BAR_X[0], 1.3, BAR_BASE, at(first), at(second_top))),
                );
                let _ = s.set_points(par, &corners(&column(BAR_X[1], 1.3, BAR_BASE, 0.0, at(both))));
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let first_name = kit.text(stage, "the first", 30.0, teal, v(BAR_X[0] - 1.9, BAR_BASE + 1.4))?;
        let second_name = kit.text(stage, "the second", 30.0, rose, v(BAR_X[0] - 1.9, BAR_BASE + 2.2))?;
        let seq_title = kit.text(
            stage,
            "one after the other",
            28.0,
            Palette::dim(),
            v(BAR_X[0], BAR_BASE - 0.5),
        )?;
        let par_title = kit.text(stage, "both at once", 28.0, Palette::dim(), v(BAR_X[1], BAR_BASE - 0.5))?;
        let mut ins = vec![seq_base, seq_a, seq_b, par, seq_title, par_title];
        ins.extend([first_name, second_name]);
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "growing at two shares at once")?;
        play!(stage; tween_linear(progress, 1.0, 4.0)?);
        line.wait_for(stage, "E to the sum is e to the first")?;
        let sum = kit.tex(
            stage,
            r"e^{a+b}=e^{a}\cdot e^{b}",
            56.0,
            ink,
            &[("e", gold), ("a", teal), ("b", rose)],
            v(2.4, 2.8),
        )?;
        let tick = chip(&kit, stage, "level", 30.0, good(), v(5.8, 2.8))?;
        fade_in_all(stage, &[sum, tick], 0.6)?;
        kit.hold(stage)?;

        // ---- 14_2: two matrices, both shears
        let line = say(&kit, stage, "14_2")?;
        let mut away = ins.clone();
        away.extend([sum, tick]);
        fade_out_all(stage, &away, 0.5)?;
        let still = outline(stage, &PLANE, Palette::dim());
        let a_field = field(stage, &PLANE, &a, teal, 0.5);
        let b_field = field(stage, &PLANE, &b, rose, 0.5);
        let a_grid = MatrixGrid::build(
            &kit,
            stage,
            &whole_entries("F-O.A")?,
            54.0,
            teal,
            v(GRID_AT[0] - 1.4, GRID_AT[1]),
        )?;
        let b_grid = MatrixGrid::build(
            &kit,
            stage,
            &whole_entries("F-O.B")?,
            54.0,
            rose,
            v(GRID_AT[0] + 1.6, GRID_AT[1]),
        )?;
        let a_name = kit.tex(stage, "A", 48.0, teal, &[], v(GRID_AT[0] - 1.4, GRID_AT[1] + 1.4))?;
        let b_name = kit.tex(stage, "B", 48.0, rose, &[], v(GRID_AT[0] + 1.6, GRID_AT[1] + 1.4))?;
        // "Now two matrices, A and B, both shears": the square and the two
        // matrices are up from the line's first words (the first frame of
        // the line was black until "One pushes the top"); each one's arrows
        // come in as its sentence is said
        let first_in = vec![still, a_grid.group, a_name, b_grid.group, b_name];
        fade_in_all(stage, &first_in, 0.7)?;
        line.wait_for(stage, "One pushes the top of a square to the right")?;
        fade_in_all(stage, &a_field, 0.7)?;
        line.wait_for(stage, "The other pushes its right side upward")?;
        fade_in_all(stage, &b_field, 0.7)?;
        let mut second_in = a_field.clone();
        second_in.extend(&b_field);
        kit.hold(stage)?;

        // ---- 14_3: your turn: A then B, or B then A?
        let line = say(&kit, stage, "14_3")?;
        let mut clear = first_in.clone();
        clear.extend(&second_in);
        fade_out_all(stage, &clear, 0.5)?;
        let small = Plane {
            centre: [-5.3, -0.9, 0.0],
            unit: 1.3,
        };
        let small_right = Plane {
            centre: [-2.3, -0.9, 0.0],
            unit: 1.3,
        };
        let left_square = outline(stage, &small, ink);
        let right_square = outline(stage, &small_right, ink);
        let left_name = kit.text(stage, "A, then B", 30.0, ink, v(small.centre[0] + 0.65, 1.0))?;
        let right_name = kit.text(stage, "B, then A", 30.0, ink, v(small_right.centre[0] + 0.65, 1.0))?;
        let card = QuizCard::build(&kit, stage, "14_3", [3.9, 0.3, 0.0], (5.6, 3.7))?;
        let (middle, room, _) = card.content();
        let problem = kit.text(stage, "the same place?", 44.0, ink, v(middle[0], middle[1] + 0.2))?;
        fit_width(stage, problem, room);
        fade_in_all(stage, &[left_square, right_square, left_name, right_name], 0.6)?;
        line.wait_last(stage)?;
        card.show(stage, &[problem], 0.5)?;
        card.count(&kit, stage)?;
        let answer = kit.text(stage, "no", 64.0, warn(), v(middle[0], middle[1] - 0.5))?;
        card.land(stage, &[answer], 0.4)?;

        // ---- 14_4: three different results
        let line = say(&kit, stage, "14_4")?;
        let mut leave = card.mobs();
        leave.extend([problem, answer, left_square, right_square, left_name, right_name]);
        fade_out_all(stage, &leave, 0.5)?;
        let ghost = outline(stage, &PLANE, Palette::dim());
        let diagonal = polyline(vec![PLANE.p(0.0, 0.0), PLANE.p(3.2, 3.2)])
            .style(faded_line(Palette::faint(), 2.5, 1.0))
            .build();
        let diagonal = stage.arena_mut().add(diagonal);
        let a_then_b = Moving::build(stage, teal, {
            let (a, b) = (a.clone(), b.clone());
            move |p| then_pose(&a, &b, p)
        })?;
        let b_then_a = Moving::build(stage, rose, {
            let (a, b) = (a.clone(), b.clone());
            move |p| then_pose(&b, &a, p)
        })?;
        let at_once = Moving::build(stage, gold, {
            let (a, b) = (a.clone(), b.clone());
            move |p| both_pose(&a, &b, p)
        })?;
        fade_in_all(
            stage,
            &[ghost, diagonal, a_then_b.poly, b_then_a.poly, at_once.poly],
            0.4,
        )?;
        let rows_ab = whole_entries("F-O.expB_expA")?;
        let rows_ba = whole_entries("F-O.expA_expB")?;
        let rows_both = super::draw::entries("F-V.14_4.both_at_once", Some(3))?;
        let g1 = MatrixGrid::build(&kit, stage, &rows_ab, 56.0, teal, v(GRID_AT[0] + 0.1, GRID_AT[1]))?;
        let n1 = result_name(&kit, stage, "A, then B", teal)?;
        line.wait_for(stage, "No. They're mirror images")?;
        let moves = vec![
            stage.prepare(tween_linear(a_then_b.progress, 2.0, 2.0)?)?,
            fade_in_anim(stage, g1.group, 0.6)?,
            fade_in_anim(stage, n1, 0.6)?,
        ];
        stage.play_prepared(moves)?;
        let g2 = MatrixGrid::build(&kit, stage, &rows_ba, 56.0, rose, v(GRID_AT[0] + 0.1, GRID_AT[1]))?;
        let n2 = result_name(&kit, stage, "B, then A", rose)?;
        let swap = vec![
            stage.prepare(tween_linear(b_then_a.progress, 2.0, 2.0)?)?,
            fade_out_anim(stage, g1.group, 0.5)?,
            fade_out_anim(stage, n1, 0.5)?,
            fade_in_anim(stage, g2.group, 0.6)?,
            fade_in_anim(stage, n2, 0.6)?,
        ];
        stage.play_prepared(swap)?;
        line.wait_for(stage, "both rules at once")?;
        let g3 = MatrixGrid::build(&kit, stage, &rows_both, 56.0, gold, v(GRID_AT[0] + 0.1, GRID_AT[1]))?;
        let n3 = result_name(&kit, stage, "both at once", gold)?;
        let swap = vec![
            stage.prepare(tween_linear(at_once.progress, 1.0, 2.0)?)?,
            fade_out_anim(stage, g2.group, 0.5)?,
            fade_out_anim(stage, n2, 0.5)?,
            fade_in_anim(stage, g3.group, 0.6)?,
            fade_in_anim(stage, n3, 0.6)?,
        ];
        stage.play_prepared(swap)?;
        line.wait_for(stage, "Three different results")?;
        // said with no music under it (the score bead: the bare key line)
        kit.mark(stage, "silence:three_results");
        kit.hold(stage)?;

        // ---- 14_5: AB and BA: commuting
        let line = say(&kit, stage, "14_5")?;
        let shapes = vec![
            ghost,
            diagonal,
            a_then_b.poly,
            b_then_a.poly,
            at_once.poly,
            g3.group,
            n3,
        ];
        let ab = MatrixGrid::build(&kit, stage, &whole_entries("F-O.AB")?, 60.0, teal, v(-2.6, 0.9))?;
        let ba = MatrixGrid::build(&kit, stage, &whole_entries("F-O.BA")?, 60.0, rose, v(2.6, 0.9))?;
        let ab_name = kit.tex(stage, r"AB=", 60.0, teal, &[], v(-5.1, 0.9))?;
        let ba_name = kit.tex(stage, r"BA=", 60.0, rose, &[], v(0.1, 0.9))?;
        // the three results stay up for "The rule broke because order
        // matters" (the sentence ran over a black stage); they go as the two
        // products come
        line.wait_for(stage, "Multiply the two grids in either order")?;
        stage.clear_updaters(a_then_b.anchor, true);
        stage.clear_updaters(b_then_a.anchor, true);
        stage.clear_updaters(at_once.anchor, true);
        fade_out_all(stage, &shapes, 0.5)?;
        fade_in_all(stage, &[ab.group, ab_name], 0.6)?;
        fade_in_all(stage, &[ba.group, ba_name], 0.6)?;
        line.wait_for(stage, "When those agree, the matrices commute")?;
        let commute = kit.tex(
            stage,
            r"AB=BA\ \Rightarrow\ e^{t(A+B)}=e^{tA}e^{tB}",
            44.0,
            ink,
            &[("e", gold)],
            v(0.0, -1.0),
        )?;
        fade_in_all(stage, &[commute], 0.6)?;
        let (tx, ty) = TURNS;
        let tz = f64::from(tx * tx + ty * ty).sqrt();
        let exact = lower_third(
            &kit,
            stage,
            "precisely:",
            &format!(
                "exp(t(A+B)) = exp(tA) exp(tB) for every time t exactly when AB = BA. One pair can still obey it \
                 without commuting: {tx} full turns about x, {ty} about y, {tz:.0} about the {tx}-{ty}-{tz:.0} axis"
            ),
        )?;
        fade_in_all(stage, &[exact], 0.6)?;
        kit.hold(stage)?;

        // ---- 14_6: the loop that does not close
        let line = say(&kit, stage, "14_6")?;
        let leaving = vec![ab.group, ba.group, ab_name, ba_name, commute, exact];
        fade_out_all(stage, &leaving, 0.5)?;
        let t = shown::SQUARE_LOOP_TIME;
        let ghost = outline(stage, &PLANE, Palette::dim());
        let walker = Moving::build(stage, ink, move |p| loop_pose(t, p))?;
        let labels = ["A", "B", "undo A", "undo B"];
        let label_colors = [teal, rose, teal, rose];
        let mut stage_names = Vec::new();
        for (words, c) in labels.iter().zip(label_colors) {
            stage_names.push(kit.text(stage, words, 36.0, c, v(-2.6, 2.7))?);
        }
        fade_in_all(stage, &[ghost, walker.poly], 0.5)?;
        line.wait_for(stage, "Go round a loop")?;
        for (i, &n) in stage_names.iter().enumerate() {
            let go = vec![
                fade_in_anim(stage, n, 0.2)?,
                stage.prepare(tween_linear(walker.progress, (i + 1) as f64, 1.2)?)?,
            ];
            stage.play_prepared(go)?;
            let off = fade_out_anim(stage, n, 0.2)?;
            stage.play_prepared(vec![off])?;
        }
        line.wait_for(stage, "The square doesn't come back")?;
        // what is left over is a new rule: a field of stretch and squeeze
        let c = a.commutator(&b);
        let leftover = field(stage, &PLANE, &c, teal, 0.5);
        let c_grid = MatrixGrid::build(
            &kit,
            stage,
            &whole_entries("F-O.commutator")?,
            56.0,
            teal,
            v(GRID_AT[0] + 0.6, GRID_AT[1]),
        )?;
        let c_name = kit.tex(stage, r"AB-BA=", 56.0, teal, &[], v(GRID_AT[0] - 1.9, GRID_AT[1]))?;
        let mut ins = leftover.clone();
        ins.extend([c_grid.group, c_name]);
        fade_in_all(stage, &ins, 0.7)?;
        line.wait_for(stage, "That is the commutator")?;
        let approx = kit.tex(
            stage,
            r"\text{leftover}\approx -t^{2}\,[A,B]",
            40.0,
            ink,
            &[("A", teal), ("B", rose)],
            v(GRID_AT[0], GRID_AT[1] - 1.7),
        )?;
        let bch = lower_third(
            &kit,
            stage,
            "who and when:",
            "the Baker\u{2013}Campbell\u{2013}Hausdorff formula",
        )?;
        fade_in_all(stage, &[approx, bch], 0.6)?;
        music_hold(&kit, stage)?;
        let swap = vec![fade_out_anim(stage, bch, 0.4)?];
        stage.play_prepared(swap)?;
        let dev = lower_third(
            &kit,
            stage,
            "for programmers:",
            "update A\u{2019}s effect, then B\u{2019}s, and this is your error per step",
        )?;
        fade_in_all(stage, &[dev], 0.5)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, testing};
    use super::*;
    use crate::facts::square_loop;
    use crate::model::{expm, hat};

    fn rows(key: &str) -> Vec<Vec<f64>> {
        super::super::draw::matrix(key).unwrap()
    }

    fn near(m: &Mat, key: &str, tol: f64) -> bool {
        let want = rows(key);
        (0..m.n()).all(|i| (0..m.n()).all(|j| (m.get(i, j) - want[i][j]).abs() < tol))
    }

    /// 14_1, an always sentence for numbers: e to the sum is e to the first
    /// times e to the second; the bars end level, whatever the shares.
    #[test]
    fn for_numbers_growing_at_two_shares_at_once_is_one_after_the_other() {
        for (a, b) in [(SHARE_A, SHARE_B), (0.0, 1.3), (-0.7, 0.7), (2.0, -1.5)] {
            assert!(((a + b).exp() - a.exp() * b.exp()).abs() < 1e-12 * (a + b).exp().max(1.0));
        }
        let (first, second_top, both) = bar_tops(1.0);
        assert!((second_top - both).abs() < 1e-12, "they end level");
        assert!((first - SHARE_A.exp()).abs() < 1e-12);
        assert_eq!(bar_tops(0.0), (1.0, 1.0, 1.0));
        // on the way the two bars grow differently (the sequential one spends
        // half the time at share a, the parallel one all of it at a + b);
        // they meet at the end only
        let (_, s, p) = bar_tops(0.25);
        assert!(s > p && (s - p).abs() > 1e-3, "{s} against {p}");
    }

    /// 14_2 and 14_4: the shears are the facts', "A then B" is e^B e^A
    /// (1,1;1,2), "B then A" is e^A e^B (2,1;1,1), mirror images across the
    /// diagonal, and both at once is e^{A+B} between them.
    #[test]
    fn three_different_results_and_two_of_them_mirror_images() {
        let (a, b) = (shear_right(), shear_up());
        assert!(near(&a, "F-O.A", 1e-12) && near(&b, "F-O.B", 1e-12));
        let a_then_b = then_pose(&a, &b, 2.0);
        let b_then_a = then_pose(&b, &a, 2.0);
        let both = both_pose(&a, &b, 1.0);
        assert!(near(&a_then_b, "F-O.expB_expA", 1e-12));
        assert!(near(&b_then_a, "F-O.expA_expB", 1e-12));
        assert!(near(&both, "F-V.14_4.both_at_once", 5e-4));
        // mirror images: swap the two coordinates of every entry
        let swap = Mat::from_rows(&[&[0.0, 1.0], &[1.0, 0.0]]);
        assert!(swap.mul(&a_then_b).mul(&swap).sub(&b_then_a).max_abs() < 1e-12);
        // three different shapes
        assert!(a_then_b.sub(&b_then_a).max_abs() > 0.5);
        assert!(both.sub(&a_then_b).max_abs() > 0.1 && both.sub(&b_then_a).max_abs() > 0.1);
        // the limit of a tiny step of each, over and over
        let alternate = rows("F-O.alternate.n1000");
        assert!((both.get(0, 0) - alternate[0][0]).abs() < 2e-3);
        // poses at the stage boundaries are the finished moves
        assert!(then_pose(&a, &b, 1.0).sub(&expm_t(&a, 1.0)).max_abs() < 1e-12);
        assert!(then_pose(&a, &b, 0.0).sub(&Mat::identity(2)).max_abs() < 1e-12);
        // the quiz's answer is these two grids
        assert_eq!(fact("F-Q.14_3.answer").unwrap(), "no: 1,1;1,2 against 2,1;1,1");
    }

    /// 14_5: AB and BA are the facts' two different grids; a pair of
    /// matrices that does commute obeys the rule; and the pair of 3 and 4
    /// full turns obeys it without commuting.
    #[test]
    fn the_products_differ_commuting_pairs_obey_and_one_pair_obeys_without() {
        let (a, b) = (shear_right(), shear_up());
        assert!(near(&a.mul(&b), "F-O.AB", 1e-12) && near(&b.mul(&a), "F-O.BA", 1e-12));
        assert!(a.mul(&b).sub(&b.mul(&a)).max_abs() > 0.5, "AB and BA disagree");
        // the rule fails for the shears, and holds for a commuting pair
        assert!(expm(&a.add(&b)).sub(&expm(&a).mul(&expm(&b))).max_abs() > 0.1);
        let (c, d) = (
            Mat::from_rows(&[&[1.0, 2.0], &[2.0, 1.0]]),
            Mat::from_rows(&[&[3.0, 1.0], &[1.0, 3.0]]),
        );
        assert_eq!(c.commutator(&d).max_abs(), 0.0);
        assert!(expm(&c.add(&d)).sub(&expm(&c).mul(&expm(&d))).max_abs() < 1e-9 * expm(&c.add(&d)).max_abs());
        // 3 full turns about x and 4 about y: not commuting, and still the rule
        let tau = std::f64::consts::TAU;
        let (tx, ty) = (f64::from(TURNS.0), f64::from(TURNS.1));
        let (x, y) = (hat([tx * tau, 0.0, 0.0]), hat([0.0, ty * tau, 0.0]));
        let id = Mat::identity(3);
        assert!(expm(&x).sub(&id).max_abs() < 1e-9 && expm(&y).sub(&id).max_abs() < 1e-9);
        assert!(
            expm(&x.add(&y)).sub(&id).max_abs() < 1e-9,
            "5 full turns about the 3-4-5 axis"
        );
        assert!(x.commutator(&y).max_abs() > 1.0, "and they do not commute");
        assert_eq!(fact("F-O.turns.commute").unwrap(), "false");
        assert_eq!(fact("F-O.turns.sum_is_identity").unwrap(), "true");
        assert_eq!(f64::from(TURNS.0 * TURNS.0 + TURNS.1 * TURNS.1).sqrt(), 5.0);
    }

    /// 14_6: the loop, as played, ends at the facts' square (squeezed one
    /// way, stretched the other), leaves minus the commutator for short
    /// times, and its stages agree with the model's loop.
    #[test]
    fn the_loop_does_not_close_and_leaves_minus_the_commutator() {
        let t = shown::SQUARE_LOOP_TIME;
        let end = loop_pose(t, 4.0);
        assert!(end.sub(&square_loop(t)).max_abs() < 1e-12);
        assert!(near(&end, "F-O.loop.t0.5", 5e-5));
        assert!(
            end.get(0, 0) < 1.0 && end.get(1, 1) > 1.0,
            "squeezed across, stretched up"
        );
        // continuous from stage to stage
        for k in 1..4 {
            let before = loop_pose(t, f64::from(k) - 1e-9);
            let after = loop_pose(t, f64::from(k));
            assert!(before.sub(&after).max_abs() < 1e-6, "stage {k}");
        }
        // for short times the leftover over t² is minus the commutator
        let (a, b) = (shear_right(), shear_up());
        let small = 0.01;
        let left = loop_pose(small, 4.0)
            .sub(&Mat::identity(2))
            .scale(1.0 / (small * small));
        assert!(left.add(&a.commutator(&b)).max_abs() < 0.02);
        assert!(near(&a.commutator(&b), "F-O.commutator", 1e-12));
        assert_eq!(fact("F-O.loop.is_minus_commutator").unwrap(), "true");
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("14_order_matters").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "the same place?",
            "e^{a+b}=e^{a}\\cdot e^{b}",
            "A, then B",
            "both at once",
            "AB-BA=",
            "3 full turns about x, 4 about y, 5 about the 3-4-5 axis",
            "the Baker\u{2013}Campbell\u{2013}Hausdorff formula",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
