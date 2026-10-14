//! Chapter 04, the split: the carts' table run backwards. In real life the
//! push is known and the moves are not (04_1): two equations with both
//! unknowns in each. The push is a mix of the two patterns (04_2); each
//! pattern answers by itself, its half divided by its stiffness, one and
//! three (04_3); the answers add tip to tail and the carts move there
//! (04_4). Your turn: push the right cart instead (04_5); the same answer
//! turned round, one third and two thirds (04_6). Two tangled equations
//! have become two divisions (04_7).
//!
//! Every number is a fact: the mix `K-A3.mix`, the shares `K-A3.shares`,
//! the moves `K-A3.moves` and `K-A3.moves_right`, the stiffnesses
//! `K-A2.eigenvalues`, the quiz answer `K-Q.04_5.answer`. The fractions
//! are exact (the model's `Frac`), and the picture's arrows are built from
//! them, never from decimals.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::arrows::{PlaneArrow, own_lines};
use super::carts::{CartsPic, CartsSpec, NEWTON};
use super::draw::{
    MatrixGrid, Plane, QuizCard, arrow_points, chip, entries, fade_in_all, fade_out_all, faded_line, fraction_tex,
    fraction_value, items, matrix, moves, numbers, own, predict_hold, pushes, run, say, tween_all, whole,
};
use super::marks;
use super::two_carts::column_entries;

// ------------------------------------------------------------------ choices

/// The carts, large, in 04_1; and small, top right, from 04_2 on.
const BIG_AT: Vec3 = [0.0, 2.5, 0.0];
const BIG_SCALE: f64 = 0.75;
const SMALL_AT: Vec3 = [3.7, 2.7, 0.0];
const SMALL_SCALE: f64 = 0.5;
/// The plane of 04_2 to 04_4: its origin and scene units to a plane unit,
/// and how far the axes reach (plane units).
const PLANE_AT: Vec3 = [-3.1, -0.5, 0.0];
const PLANE_UNIT: f64 = 1.7;
const REACH: (f64, f64) = (1.5, 1.35);
/// The ladder of 04_3: its two rails and the rung of each pattern.
const LADDER_X: (f64, f64) = (0.8, 2.6);
const RUNGS_Y: (f64, f64) = (1.0, -0.25);

// ------------------------------------------------------------- the geometry

/// `c a + d b` as TeX with the variables written the way the storyboard
/// does: a unit coefficient is left out, a negative second term is a
/// subtraction (`2a - b`, `-a + 2b`).
pub fn linear_tex(c: [f64; 2], vars: [&str; 2]) -> String {
    let term = |x: f64, var: &str| {
        let a = x.abs();
        if (a - 1.0).abs() < 1e-12 {
            var.to_owned()
        } else {
            format!("{}{var}", whole(a))
        }
    };
    let first = if c[0] < 0.0 {
        format!("-{}", term(c[0], vars[0]))
    } else {
        term(c[0], vars[0])
    };
    let second = if c[1] < 0.0 {
        format!(" - {}", term(c[1], vars[1]))
    } else {
        format!(" + {}", term(c[1], vars[1]))
    };
    format!("{first}{second}")
}

/// `1, 0 = \tfrac12 (1, 1) + \tfrac12 (1, -1)`: a push as the mix of the
/// two patterns, with the fractions as the facts print them.
pub fn mix_tex(push: [f64; 2], mix: &[&str], soft: [f64; 2], stiff: [f64; 2]) -> String {
    let pair = |p: [f64; 2]| format!(r"({},\,{})", whole(p[0]), whole(p[1]));
    let sign = |s: &str| if s.starts_with('-') { " - " } else { " + " };
    let magnitude = |s: &str| fraction_tex(s.trim_start_matches('-'));
    format!(
        r"{},\,{} = {}\,{}{}{}\,{}",
        whole(push[0]),
        whole(push[1]),
        fraction_tex(mix[0]),
        pair(soft),
        sign(mix[1]),
        magnitude(mix[1]),
        pair(stiff)
    )
}

/// `n` times the pattern `p`, a piece of the split in plane units.
pub fn piece(n: f64, p: [f64; 2]) -> [f64; 2] {
    [n * p[0], n * p[1]]
}

/// The y at which the name of a list (`points` pt, one line) sits so that its
/// words clear the list's brackets by a gap: above their top by 0.12 and half
/// the words' height (about 0.0125 a point). The names touched the brackets
/// when they were set a fixed 0.95 above the list's middle.
fn name_above(list: &MatrixGrid, points: f64) -> f64 {
    list.at[1] + list.size.1 / 2.0 + 0.12 + points * 0.0125 / 2.0
}

/// How far the bottom of `name` stands above the top of `list`'s brackets,
/// as laid out (negative: the words are on the brackets).
fn name_gap(stage: &Stage<'_>, name: Mob, list: &MatrixGrid) -> f64 {
    (stage.get_center(name)[1] - stage.get_height(name) / 2.0) - (list.at[1] + list.size.1 / 2.0)
}

/// A one-column grid of exact fractions as the facts print them.
fn fraction_column(items: &[&str]) -> Vec<Vec<String>> {
    items.iter().map(|s| vec![fraction_tex(s)]).collect()
}

/// A static push arrow of `newtons` from `x`, at height `y` (the carts'
/// hands' height), at the picture scale `scale`, detached.
fn given_push(stage: &mut Stage<'_>, x: f64, y: f64, scale: f64, newtons: f64) -> Mob {
    let a = polyline(arrow_points(v(x, y), v(x + scale * NEWTON * newtons, y), 0.22 * scale))
        .style(faded_line(pushes(), 6.0, 1.0))
        .build();
    stage.arena_mut().add(a)
}

/// A straight stroke from `a` to `b`, detached.
fn stroke(stage: &mut Stage<'_>, a: Vec3, b: Vec3, colour: Srgb, width: f64) -> Mob {
    let l = polyline(vec![a, b]).style(faded_line(colour, width, 1.0)).build();
    stage.arena_mut().add(l)
}

/// The table's two rows set equal to the push, as the two equations with
/// both unknowns in each (`2a - b = 1`, `-a + 2b = 0`), the first centred at
/// `at` and the second `step` below it, detached.
fn tangled_equations(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rows: &[Vec<f64>],
    push: [f64; 2],
    at: Vec3,
    size: f64,
    step: f64,
) -> fmn::Result<[Mob; 2]> {
    let mut made = Vec::with_capacity(2);
    for row in 0..2 {
        let src = format!(
            "{} = {}",
            linear_tex([rows[row][0], rows[row][1]], ["a", "b"]),
            whole(push[row])
        );
        made.push(kit.tex(
            stage,
            &src,
            size,
            Palette::ink(),
            &[],
            v(at[0], at[1] - step * row as f64),
        )?);
    }
    Ok([made[0], made[1]])
}

// -------------------------------------------------------------- the scene

pub struct TheSplit {
    pub kit: Kit,
}

impl SceneConstruct for TheSplit {
    fn name(&self) -> &str {
        "04_the_split"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange, gold) = (moves(), pushes(), own());

        // the facts the chapter is made of, as exact fractions
        let rows = matrix("K-A1.table")?;
        let soft = numbers("K-A2.pattern.soft")?;
        let stiff = numbers("K-A2.pattern.stiff")?;
        let (soft, stiff) = ([soft[0], soft[1]], [stiff[0], stiff[1]]);
        let mix_items = items("K-A3.mix")?;
        let share_items = items("K-A3.shares")?;
        let move_items = items("K-A3.moves")?;
        let right_items = items("K-A3.moves_right")?;
        let stiffness = numbers("K-A2.eigenvalues")?;
        let mix = [fraction_value(mix_items[0])?, fraction_value(mix_items[1])?];
        let shares = [fraction_value(share_items[0])?, fraction_value(share_items[1])?];
        let moved = [fraction_value(move_items[0])?, fraction_value(move_items[1])?];
        let moved_right = [fraction_value(right_items[0])?, fraction_value(right_items[1])?];
        let push_left = [1.0, 0.0];

        // ---- 04_1: the push is known, the moves are not
        let line = say(&kit, stage, "04_1")?;
        let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let big = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: BIG_AT,
                scale: BIG_SCALE,
            },
            u,
        )?;
        big.start(stage)?;
        fade_in_all(stage, &big.base(), 0.7)?;
        // one newton on the left cart: its hand and its arrow
        let hand_y = BIG_AT[1] + BIG_SCALE * (0.425 + 0.5);
        let push_arrow = given_push(stage, big.cart_at(0, 0.0)[0], hand_y, BIG_SCALE, push_left[0]);
        let pushes_list = MatrixGrid::build(&kit, stage, &column_entries(push_left), 52.0, orange, v(-4.6, -0.4))?;
        let name_y = name_above(&pushes_list, 30.0);
        let pushes_label = kit.text(stage, "pushes", 30.0, orange, v(-4.6, name_y))?;
        let moves_label = kit.text(stage, "moves", 30.0, blue, v(-1.9, name_y))?;
        marks::note(&kit, "04_1.name_gap", name_gap(stage, pushes_label, &pushes_list));
        let unknown = [
            kit.text(stage, "?", 60.0, blue, v(-1.9, -0.1))?,
            kit.text(stage, "?", 60.0, blue, v(-1.9, -0.7))?,
        ];
        // the two equations, each with both unknowns in it
        let tangled = tangled_equations(&kit, stage, &rows, push_left, v(3.6, -0.1), 62.0, 1.2)?;
        line.wait_for(stage, "Push the left cart with one newton")?;
        tween_all(stage, &[(big.hands_on[0], 1.0)], 0.5)?;
        fade_in_all(stage, &[push_arrow, pushes_label, pushes_list.group], 0.6)?;
        line.wait_for(stage, "Where do the carts end up")?;
        fade_in_all(
            stage,
            &[moves_label, unknown[0], unknown[1], tangled[0], tangled[1]],
            0.7,
        )?;
        predict_hold(&kit, stage)?;

        // ---- 04_2: the push is a mix of the two patterns
        let line = say(&kit, stage, "04_2")?;
        big.stop(stage);
        let mut leaving = big.base();
        leaving.extend(big.hands);
        leaving.extend(big.push_arrows);
        leaving.extend(big.move_arrows);
        leaving.extend([
            push_arrow,
            pushes_label,
            moves_label,
            pushes_list.group,
            unknown[0],
            unknown[1],
        ]);
        leaving.extend(tangled);
        fade_out_all(stage, &leaving, 0.6)?;
        let plane = Plane {
            centre: PLANE_AT,
            unit: PLANE_UNIT,
        };
        let axes = plane.axes(stage, REACH.0, REACH.1);
        let across = kit.text(stage, "left cart", 24.0, dim, plane.p(REACH.0 - 0.15, -0.22))?;
        let up = kit.text(stage, "right cart", 24.0, dim, plane.p(0.0, REACH.1 + 0.22))?;
        let bands = own_lines(stage, plane, &[soft, stiff], REACH.0, REACH.1);
        stage.add_many_to_scene(&bands)?;
        // the push, and two copies of it that will leave along the gold lines
        let push_plane = PlaneArrow::build(stage, plane, orange, blue, 8.0, 0.3, [0.0, 0.0], push_left)?;
        let a = PlaneArrow::build(stage, plane, orange, blue, 6.0, 0.26, [0.0, 0.0], push_left)?;
        let b = PlaneArrow::build(stage, plane, orange, blue, 6.0, 0.26, [0.0, 0.0], push_left)?;
        for p in [&push_plane, &a, &b] {
            p.start(stage)?;
        }
        let mut plane_mobs = axes.clone();
        plane_mobs.extend([across, up]);
        fade_in_all(stage, &plane_mobs, 0.6)?;
        let mut bands_in = Vec::new();
        for (k, &m) in bands.iter().enumerate() {
            let opacity = if k % 2 == 0 { 0.22 } else { 0.9 };
            bands_in.push(stage.prepare(m.animate().set_anim_args(run(0.6))?.set_opacity(opacity)?)?);
        }
        stage.play_prepared(bands_in)?;
        tween_all(stage, &[(push_plane.on, 1.0)], 0.5)?;
        let small_u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let small = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: SMALL_AT,
                scale: SMALL_SCALE,
            },
            small_u,
        )?;
        small.start(stage)?;
        fade_in_all(stage, &small.base(), 0.6)?;
        let mix_eq = kit.tex(
            stage,
            &mix_tex(push_left, &mix_items, soft, stiff),
            44.0,
            ink,
            &[],
            v(3.5, -2.35),
        )?;
        line.wait_for(stage, "Half of together")?;
        // the split: two shorter arrows along the gold lines, the push beneath
        let (together_half, against_half) = (piece(mix[0], soft), piece(mix[1], stiff));
        tween_all(
            stage,
            &[
                (a.tip[0], together_half[0]),
                (a.tip[1], together_half[1]),
                (b.tip[0], against_half[0]),
                (b.tip[1], against_half[1]),
                (a.on, 1.0),
                (b.on, 1.0),
                (push_plane.on, 0.35),
            ],
            1.4,
        )?;
        fade_in_all(stage, &[mix_eq], 0.7)?;

        // ---- 04_3: each pattern answers by itself, its half over its stiffness
        let line = say(&kit, stage, "04_3")?;
        let rails = [
            stroke(stage, v(LADDER_X.0, 1.75), v(LADDER_X.0, -1.05), dim, 3.0),
            stroke(stage, v(LADDER_X.1, 1.75), v(LADDER_X.1, -1.05), dim, 3.0),
        ];
        let rungs = [
            stroke(stage, v(LADDER_X.0, RUNGS_Y.0), v(LADDER_X.1, RUNGS_Y.0), gold, 5.0),
            stroke(stage, v(LADDER_X.0, RUNGS_Y.1), v(LADDER_X.1, RUNGS_Y.1), gold, 5.0),
        ];
        let middle_x = (LADDER_X.0 + LADDER_X.1) / 2.0;
        let names = [
            kit.text(stage, "together", 24.0, gold, v(middle_x, RUNGS_Y.0 + 0.3))?,
            kit.text(stage, "against", 24.0, gold, v(middle_x, RUNGS_Y.1 + 0.3))?,
        ];
        let divisions = [
            kit.tex(
                stage,
                &format!(r"\div\,{}", whole(stiffness[0])),
                44.0,
                ink,
                &[],
                v(3.35, RUNGS_Y.0),
            )?,
            kit.tex(
                stage,
                &format!(r"\div\,{}", whole(stiffness[1])),
                44.0,
                ink,
                &[],
                v(3.35, RUNGS_Y.1),
            )?,
        ];
        let mut works = Vec::new();
        for (k, y) in [RUNGS_Y.0, RUNGS_Y.1].into_iter().enumerate() {
            works.push(kit.tex(
                stage,
                &format!(
                    r"{} \div {} = {}",
                    fraction_tex(mix_items[k]),
                    whole(stiffness[k]),
                    fraction_tex(share_items[k])
                ),
                38.0,
                ink,
                &[],
                v(5.35, y),
            )?);
        }
        fade_out_all(stage, &[mix_eq], 0.4)?;
        let mut ladder = rails.to_vec();
        ladder.extend(rungs);
        ladder.extend(names);
        fade_in_all(stage, &ladder, 0.7)?;
        line.wait_for(stage, "Together has stiffness one")?;
        fade_in_all(stage, &[divisions[0], works[0]], 0.6)?;
        line.wait_for(stage, "Against has stiffness three")?;
        fade_in_all(stage, &[divisions[1], works[1]], 0.6)?;
        // the pieces answer: orange turns blue, the stiff one a third as long
        let first_answer = piece(shares[0], soft);
        let second_answer = piece(shares[1], stiff);
        tween_all(
            stage,
            &[
                (a.mix, 1.0),
                (b.mix, 1.0),
                (a.tip[0], first_answer[0]),
                (a.tip[1], first_answer[1]),
                (b.tip[0], second_answer[0]),
                (b.tip[1], second_answer[1]),
            ],
            1.4,
        )?;

        // ---- 04_4: add the answers, and the carts move there
        let line = say(&kit, stage, "04_4")?;
        let mut done = ladder.clone();
        done.extend(divisions);
        done.extend(works);
        fade_out_all(stage, &done, 0.5)?;
        let sum_arrow = PlaneArrow::build(stage, plane, blue, blue, 9.0, 0.3, [0.0, 0.0], moved)?;
        sum_arrow.start(stage)?;
        line.wait_for(stage, "Add the answers")?;
        // the second answer slides to the first one's tip, tip to tail
        tween_all(
            stage,
            &[
                (b.tail[0], first_answer[0]),
                (b.tail[1], first_answer[1]),
                (b.tip[0], first_answer[0] + second_answer[0]),
                (b.tip[1], first_answer[1] + second_answer[1]),
            ],
            1.3,
        )?;
        tween_all(stage, &[(sum_arrow.on, 1.0), (push_plane.on, 0.0)], 0.6)?;
        let sums = [
            kit.tex(
                stage,
                &format!(
                    r"{} + {} = {}",
                    fraction_tex(share_items[0]),
                    fraction_tex(share_items[1]),
                    fraction_tex(move_items[0])
                ),
                40.0,
                ink,
                &[],
                v(3.5, 0.45),
            )?,
            kit.tex(
                stage,
                &format!(
                    r"{} - {} = {}",
                    fraction_tex(share_items[0]),
                    fraction_tex(share_items[1]),
                    fraction_tex(move_items[1])
                ),
                40.0,
                ink,
                &[],
                v(3.5, -0.5),
            )?,
        ];
        line.wait_for(stage, "The left cart moves two thirds")?;
        fade_in_all(stage, &sums, 0.6)?;
        tween_all(stage, &[(small.u[0], moved[0]), (small.u[1], moved[1])], 1.3)?;
        // the check, on screen only: the table turns the moves back into the push
        let check_table = MatrixGrid::build(&kit, stage, &entries("K-A1.table", None)?, 38.0, ink, v(1.5, -2.3))?;
        let check_moves = MatrixGrid::build(&kit, stage, &fraction_column(&move_items), 38.0, blue, v(3.6, -2.3))?;
        let check_equals = kit.tex(stage, "=", 44.0, ink, &[], v(4.65, -2.3))?;
        let check_push = MatrixGrid::build(&kit, stage, &column_entries(push_left), 38.0, orange, v(5.7, -2.3))?;
        line.wait_for(stage, "one third")?;
        fade_in_all(
            stage,
            &[check_table.group, check_moves.group, check_equals, check_push.group],
            0.7,
        )?;

        // ---- 04_5: your turn, push the right cart instead
        let line = say(&kit, stage, "04_5")?;
        small.stop(stage);
        for p in [&a, &b, &push_plane, &sum_arrow] {
            p.stop(stage);
        }
        let mut gone = plane_mobs.clone();
        gone.extend(&bands);
        gone.extend([a.mob, b.mob, push_plane.mob, sum_arrow.mob]);
        gone.extend(sums);
        gone.extend([check_table.group, check_moves.group, check_equals, check_push.group]);
        gone.extend(small.base());
        gone.extend(small.move_arrows);
        gone.extend(small.push_arrows);
        gone.extend(small.hands);
        fade_out_all(stage, &gone, 0.6)?;
        let card = QuizCard::build(&kit, stage, "04_5", v(0.0, 0.05), (12.4, 6.6))?;
        let (middle, _, _) = card.content();
        let w = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let card_carts = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: v(-3.2, middle[1] + 1.05),
                scale: 0.5,
            },
            w,
        )?;
        card_carts.start(stage)?;
        let card_hand_y = middle[1] + 1.05 + 0.5 * (0.425 + 0.5);
        let right_push = given_push(stage, card_carts.cart_at(1, 0.0)[0], card_hand_y, 0.5, 1.0);
        stage.set_tracker_value(card_carts.hands_on[1], 1.0)?;
        // lower than the carts' ruler by a hand, so the names fit between it and the brackets
        let row_y = middle[1] - 0.85;
        let right_only = [0.0, 1.0];
        let card_pushes = MatrixGrid::build(&kit, stage, &column_entries(right_only), 42.0, orange, v(-4.5, row_y))?;
        let card_name_y = name_above(&card_pushes, 28.0);
        let card_pushes_label = kit.text(stage, "pushes", 28.0, orange, v(-4.5, card_name_y))?;
        let card_moves_label = kit.text(stage, "moves", 28.0, blue, v(-1.9, card_name_y))?;
        marks::note(&kit, "04_5.name_gap", name_gap(stage, card_pushes_label, &card_pushes));
        let card_unknown = [
            kit.text(stage, "?", 54.0, blue, v(-1.9, row_y + 0.3))?,
            kit.text(stage, "?", 54.0, blue, v(-1.9, row_y - 0.3))?,
        ];
        let card_flow = {
            let l = polyline(arrow_points(v(-3.7, row_y), v(-2.7, row_y), 0.2))
                .style(faded_line(dim, 4.0, 1.0))
                .build();
            stage.arena_mut().add(l)
        };
        let mut problem = card_carts.base();
        problem.extend([
            right_push,
            card_pushes_label,
            card_moves_label,
            card_pushes.group,
            card_unknown[0],
            card_unknown[1],
            card_flow,
        ]);
        card.show_frame(stage, &problem, 0.8)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 04_6: one third and two thirds, the same answer turned round
        let line = say(&kit, stage, "04_6")?;
        let answer = items("K-Q.04_5.answer")?;
        let answer_grid = MatrixGrid::build(&kit, stage, &fraction_column(&answer), 42.0, blue, v(-1.9, row_y))?;
        fade_out_all(stage, &card_unknown, 0.2)?;
        card.land(stage, &[answer_grid.group], 0.5)?;
        line.wait_for(stage, "One third and two thirds")?;
        tween_all(stage, &[(w[0], moved_right[0]), (w[1], moved_right[1])], 1.2)?;
        // the plane again, mirrored across the together line: (0, 1) is half
        // of together and minus half of against, each over its stiffness
        let mini = Plane {
            centre: v(3.3, middle[1] + 0.2),
            unit: 1.0,
        };
        let mini_axes = mini.axes(stage, 1.5, 1.5);
        let mini_bands = own_lines(stage, mini, &[soft, stiff], 1.5, 1.5);
        stage.add_many_to_scene(&mini_bands)?;
        for (k, &m) in mini_bands.iter().enumerate() {
            stage.set_opacity(m, if k % 2 == 0 { 0.22 } else { 0.9 }, true);
        }
        let first_piece = piece(shares[0], soft);
        let second_piece = piece(-shares[1], stiff);
        let mini_a = PlaneArrow::build(stage, mini, blue, blue, 6.0, 0.22, [0.0, 0.0], first_piece)?;
        let mini_b = PlaneArrow::build(
            stage,
            mini,
            blue,
            blue,
            6.0,
            0.22,
            first_piece,
            [first_piece[0] + second_piece[0], first_piece[1] + second_piece[1]],
        )?;
        let mini_sum = PlaneArrow::build(stage, mini, blue, blue, 8.0, 0.26, [0.0, 0.0], moved_right)?;
        for p in [&mini_a, &mini_b, &mini_sum] {
            p.start(stage)?;
        }
        fade_in_all(stage, &mini_axes, 0.5)?;
        tween_all(stage, &[(mini_a.on, 1.0), (mini_b.on, 1.0)], 0.6)?;
        tween_all(stage, &[(mini_sum.on, 1.0)], 0.5)?;

        // ---- 04_7: two tangled equations have become two divisions
        let line = say(&kit, stage, "04_7")?;
        for p in [&mini_a, &mini_b, &mini_sum] {
            p.stop(stage);
        }
        card_carts.stop(stage);
        let mut away = card_carts.base();
        away.extend(card_carts.move_arrows);
        away.extend(card_carts.push_arrows);
        away.extend(card_carts.hands);
        away.extend([
            right_push,
            card_pushes_label,
            card_moves_label,
            card_pushes.group,
            answer_grid.group,
            card_flow,
            mini_a.mob,
            mini_b.mob,
            mini_sum.mob,
        ]);
        away.extend(&mini_axes);
        away.extend(&mini_bands);
        away.extend(card.mobs());
        fade_out_all(stage, &away, 0.6)?;
        let left_eqs = tangled_equations(&kit, stage, &rows, push_left, v(-3.4, 0.9), 58.0, 1.2)?;
        let mut right_divs = Vec::new();
        for (k, y) in [0.9, -0.3].into_iter().enumerate() {
            right_divs.push(kit.tex(
                stage,
                &format!(r"{} \div {}", fraction_tex(mix_items[k]), whole(stiffness[k])),
                58.0,
                ink,
                &[],
                v(3.6, y),
            )?);
        }
        let tangled_word = kit.text(stage, "tangled", 26.0, dim, v(-3.4, 1.9))?;
        let separate_word = kit.text(stage, "separate", 26.0, dim, v(3.6, 1.9))?;
        let sentence = chip(&kit, stage, "pieces that ignore each other", 30.0, gold, v(0.0, -1.55))?;
        line.wait_for(stage, "Two tangled equations")?;
        fade_in_all(stage, &[left_eqs[0], left_eqs[1], tangled_word], 0.6)?;
        line.wait_for(stage, "two divisions")?;
        fade_in_all(stage, &[right_divs[0], right_divs[1], separate_word], 0.6)?;
        // the act's closing sentence is said with a breath of silence before
        // it and no music under it (R1 C20)
        line.wait_for(stage, "Seen in its own patterns")?;
        kit.mark(stage, "silence:closing_sentence");
        line.wait_for(stage, "falls apart into pieces")?;
        fade_in_all(stage, &[sentence], 0.6)?;
        let strip = lower_third(
            &kit,
            stage,
            "names to search:",
            "eigendecomposition, diagonalization, superposition; for programmers: the split is this simple because \
             these patterns are at right angles",
        )?;
        fade_in_all(stage, &[strip], 0.5)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{fact, testing};
    use super::*;
    use crate::model::carts::moves as moves_for;

    /// 04_1 and 04_5: the names "pushes" and "moves" stand clear above the
    /// brackets of their lists, as laid out (they stood on them).
    #[test]
    fn the_names_clear_the_brackets_of_their_lists() {
        let voiced = testing::run_voiced("04_the_split").unwrap();
        for what in ["04_1.name_gap", "04_5.name_gap"] {
            let gap = marks::at(&voiced.tag, what);
            assert!(
                gap >= 0.05,
                "{what}: the name's bottom is {gap:.3} above the bracket's top"
            );
        }
    }

    /// 04_1: the two equations are the table's rows set equal to the push,
    /// written the way the storyboard writes them.
    #[test]
    fn the_equations_are_the_tables_rows() {
        assert_eq!(linear_tex([2.0, -1.0], ["a", "b"]), "2a - b");
        assert_eq!(linear_tex([-1.0, 2.0], ["a", "b"]), "-a + 2b");
        assert_eq!(linear_tex([1.0, 1.0], ["a", "b"]), "a + b");
        let rows = matrix("K-A1.table").unwrap();
        assert_eq!(linear_tex([rows[0][0], rows[0][1]], ["a", "b"]), "2a - b");
        assert_eq!(linear_tex([rows[1][0], rows[1][1]], ["a", "b"]), "-a + 2b");
    }

    /// 04_2 to 04_4: the exact fractions of the facts are the split of the
    /// push (1, 0) over the two patterns: its mix, each half over its
    /// stiffness, and the carts' moves, which solve the two equations.
    #[test]
    fn the_split_is_the_facts_fractions_and_solves_the_equations() {
        let soft = numbers("K-A2.pattern.soft").unwrap();
        let stiff = numbers("K-A2.pattern.stiff").unwrap();
        let (soft, stiff) = ([soft[0], soft[1]], [stiff[0], stiff[1]]);
        let mix_items = items("K-A3.mix").unwrap();
        let shares_items = items("K-A3.shares").unwrap();
        let moves_items = items("K-A3.moves").unwrap();
        let mix: Vec<f64> = mix_items.iter().map(|s| fraction_value(s).unwrap()).collect();
        let shares: Vec<f64> = shares_items.iter().map(|s| fraction_value(s).unwrap()).collect();
        let eig = numbers("K-A2.eigenvalues").unwrap();
        // the mix gives the push back: (1, 0) = 1/2 (1, 1) + 1/2 (1, -1)
        let (p, q) = (piece(mix[0], soft), piece(mix[1], stiff));
        assert!((p[0] + q[0] - 1.0).abs() < 1e-12 && (p[1] + q[1]).abs() < 1e-12);
        // each half over its stiffness
        for k in 0..2 {
            assert!((shares[k] - mix[k] / eig[k]).abs() < 1e-12, "share {k}");
        }
        // the answers add to the facts' moves, and the moves are the model's solution
        let (a, b) = (piece(shares[0], soft), piece(shares[1], stiff));
        let moved: Vec<f64> = moves_items.iter().map(|s| fraction_value(s).unwrap()).collect();
        assert!((a[0] + b[0] - moved[0]).abs() < 1e-12 && (a[1] + b[1] - moved[1]).abs() < 1e-12);
        let direct = moves_for([1.0, 0.0]);
        assert!((direct[0] - moved[0]).abs() < 1e-12 && (direct[1] - moved[1]).abs() < 1e-12);
        // the lower-third's picture: the sums on screen
        assert_eq!(fact("K-A3.mix").unwrap(), "1/2,1/2");
        assert_eq!(fact("K-A3.shares").unwrap(), "1/2,1/6");
        assert_eq!(fact("K-A3.moves").unwrap(), "2/3,1/3");
    }

    /// 04_5 and 04_6: pushing the right cart is the mirror image: half of
    /// together and minus half of against, each over its stiffness, and
    /// the carts end at the facts' one third and two thirds.
    #[test]
    fn the_right_cart_is_the_mirror_image() {
        let soft = numbers("K-A2.pattern.soft").unwrap();
        let stiff = numbers("K-A2.pattern.stiff").unwrap();
        let shares: Vec<f64> = items("K-A3.shares")
            .unwrap()
            .iter()
            .map(|s| fraction_value(s).unwrap())
            .collect();
        let first = piece(shares[0], [soft[0], soft[1]]);
        let second = piece(-shares[1], [stiff[0], stiff[1]]);
        let end = [first[0] + second[0], first[1] + second[1]];
        let right: Vec<f64> = items("K-A3.moves_right")
            .unwrap()
            .iter()
            .map(|s| fraction_value(s).unwrap())
            .collect();
        assert!(
            (end[0] - right[0]).abs() < 1e-12 && (end[1] - right[1]).abs() < 1e-12,
            "{end:?}"
        );
        let direct = moves_for([0.0, 1.0]);
        assert!((direct[0] - right[0]).abs() < 1e-12 && (direct[1] - right[1]).abs() < 1e-12);
        // the quiz's answer is the same fact, in the same order as the picture
        assert_eq!(items("K-Q.04_5.answer").unwrap(), items("K-A3.moves_right").unwrap());
        // and it is the first answer with the two numbers turned round
        let left = items("K-A3.moves").unwrap();
        assert_eq!(left[0], "2/3");
        assert!((right[0] - 1.0 / 3.0).abs() < 1e-12 && (right[1] - 2.0 / 3.0).abs() < 1e-12);
    }

    /// The mix as the storyboard writes it.
    #[test]
    fn the_mix_reads_as_written() {
        assert_eq!(
            mix_tex([1.0, 0.0], &["1/2", "1/2"], [1.0, 1.0], [1.0, -1.0]),
            r"1,\,0 = \tfrac{1}{2}\,(1,\,1) + \tfrac{1}{2}\,(1,\,-1)"
        );
        // a negative share is a subtraction
        assert!(mix_tex([0.0, 1.0], &["1/2", "-1/2"], [1.0, 1.0], [1.0, -1.0]).contains(" - "));
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("04_the_split").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "2a - b = 1",
            "-a + 2b = 0",
            r"\tfrac{1}{2} \div 1 = \tfrac{1}{2}",
            r"\tfrac{1}{2} \div 3 = \tfrac{1}{6}",
            r"\tfrac{1}{2} + \tfrac{1}{6} = \tfrac{2}{3}",
            r"\tfrac{1}{2} - \tfrac{1}{6} = \tfrac{1}{3}",
            "your turn",
            "names to search:",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // the length that counts is the one with the scratch voice placed
        let s = testing::run_voiced("04_the_split").unwrap().ran.seconds;
        eprintln!("04_the_split: {s:.1} s on the scratch voice");
        assert!(s > 45.0 && s < 85.0, "{s} s");
    }
}
