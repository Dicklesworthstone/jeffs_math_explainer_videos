//! Chapter 13, can it feed itself: what people want, made, uses things up, so
//! make those too, round after round, each seven tenths the size, drawn on
//! the arrow plane (13_1); the rounds add up to eighty of each, twenty-four
//! divided by three tenths (13_3); a lopsided order of thirty and ten is
//! eighteen of one pattern and four of the other, each shrinking by its own
//! eigenvalue, so the rounds settle onto equal amounts (13_4); a hungrier
//! economy, make ten of each and eleven is used up, what happens to the
//! rounds (13_5); and they grow, so the test is one number, the largest
//! eigenvalue (13_7).
//!
//! Every number is a fact or the model's: the demand `K-E3.demand` and its
//! rounds `K-E3.rounds`, the total `K-E3.total`, the lopsided order
//! `K-E5.*`, the hungry economy `K-E4.*`, the test `K-X7.*` and the cited
//! years `K-S.*`. The rounds beyond the facts' first few are the model's
//! (`model::economy`), exact fractions drawn as floating point.

use explainer_kit::props::{lower_third, names_to_search, precisely};
use explainer_kit::{Kit, Palette, fade_all, play, v};
use fmn::prelude::*;

use super::an_economy::{table_entries, table_of};
use super::arrows::PlaneArrow;
use super::draw::{
    MatrixGrid, Plane, arrow_points, cross_fade, cross_mark, fact, fade_in_all, fade_in_lagged, fade_out_all,
    faded_line, fail, flat, fraction_value, good, items, matrix, minus, moves, numbers, own, panel, pause,
    predict_hold, pushes, run, say, show, tween_all, warn, whole,
};
use crate::model::economy::{self, Table};

pub struct CanItFeedItself {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The plane of the rounds: food across, tools up, in units of goods (so
/// the totals of 80 and 75, 50 fit).
const PLANE: Plane = Plane {
    centre: [-5.9, -2.2, 0.0],
    unit: 0.048,
};
/// How far the axes reach, in units of goods.
const REACH: f64 = 90.0;
/// Half the width of the cross that marks a total (scene units).
const CROSS_HALF: f64 = 0.22;
/// Where the total's name stands from the cross's centre (scene units): its
/// left edge past the cross's lower right arm, its middle below that arm's
/// tip, so that no arm runs through a digit.
const CROSS_NAME_AT: (f64, f64) = (CROSS_HALF + 0.2, -0.3);
/// The stack of what has been made, its left and its base, and its width.
const STACK_X: f64 = 1.5;
const STACK_W: f64 = 0.9;
const STACK_BASE: f64 = -2.4;
/// The right column where equations and words are put.
const COLUMN_X: f64 = 1.0;

// ---------------------------------------------------------------- helpers

/// The amounts made in the first `n + 1` rounds from `demand`, as floats.
fn model_rounds(table: &Table, demand: [f64; 2], n: usize) -> Vec<[f64; 2]> {
    let start = [
        economy::Frac::whole(demand[0].round() as i128),
        economy::Frac::whole(demand[1].round() as i128),
    ];
    table
        .rounds(start, n)
        .iter()
        .map(|r| [r[0].to_f64(), r[1].to_f64()])
        .collect()
}

/// The name of a total in 13_4 ("75, 50"), detached, in gold beside the
/// cross that marks it.
fn name_the_total(kit: &Kit, stage: &mut Stage<'_>, total: &[f64]) -> fmn::Result<Mob> {
    let c = PLANE.p(total[0], total[1]);
    kit.text_left(
        stage,
        &format!("{}, {}", whole(total[0]), whole(total[1])),
        28.0,
        own(),
        v(c[0] + CROSS_NAME_AT.0, c[1] + CROSS_NAME_AT.1),
    )
}

/// A table in tenths (`6,5;4,7`) as a 2 by 2 map of its values.
fn table_from_tenths(key: &str) -> fmn::Result<super::draw::Map2> {
    let t = matrix(key)?;
    if t.len() != 2 || t.iter().any(|r| r.len() != 2) {
        return Err(fail(format!("fact {key} is not a 2 by 2 table")));
    }
    Ok([[t[0][0] / 10.0, t[0][1] / 10.0], [t[1][0] / 10.0, t[1][1] / 10.0]])
}

/// The two axes as faint lines from the origin, reaching `REACH` units of
/// goods each way (the first quarter only: nothing here is negative).
fn corner_axes(stage: &mut Stage<'_>) -> Vec<Mob> {
    let style = faded_line(Palette::faint(), 3.0, 1.0);
    let across = polyline(vec![PLANE.p(0.0, 0.0), PLANE.p(REACH, 0.0)])
        .style(style)
        .build();
    let up = polyline(vec![PLANE.p(0.0, 0.0), PLANE.p(0.0, REACH)])
        .style(style)
        .build();
    vec![stage.arena_mut().add(across), stage.arena_mut().add(up)]
}

/// The words at the axes' ends, detached.
fn axis_names(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    Ok(vec![
        kit.text(stage, "food", 26.0, dim, PLANE.p(REACH + 8.0, 0.0))?,
        kit.text(stage, "tools", 26.0, dim, PLANE.p(0.0, REACH + 7.0))?,
    ])
}

/// A flat arrow on the plane from `a` to `b` (goods), detached.
fn link(stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], color: Srgb, width: f64) -> Mob {
    stage.arena_mut().add(
        polyline(arrow_points(PLANE.p(a[0], a[1]), PLANE.p(b[0], b[1]), 0.2))
            .style(faded_line(color, width, 1.0))
            .build(),
    )
}

/// A column of made amounts stacked on one another, drawn from `base`
/// upwards at the plane's scale: one rectangle per amount, alternately
/// bright and dim, detached. Returns the rectangles and each one's centre.
fn stack(stage: &mut Stage<'_>, amounts: &[f64], color: Srgb) -> fmn::Result<Vec<(Mob, Vec3)>> {
    let mut out = Vec::new();
    let mut y = STACK_BASE;
    for (i, &a) in amounts.iter().enumerate() {
        let h = (a * PLANE.unit - 0.03).max(0.02);
        let centre = v(STACK_X + STACK_W / 2.0, y + h / 2.0);
        let r = Rectangle::new()
            .width(STACK_W)
            .height(h)
            .style(flat(color, if i % 2 == 0 { 0.95 } else { 0.62 }))
            .build()?
            .moved_to(centre);
        out.push((stage.arena_mut().add(r), centre));
        y += a * PLANE.unit;
    }
    Ok(out)
}

/// A small house, `r` wide, in line, detached.
fn house(stage: &mut Stage<'_>, at: Vec3, r: f64) -> Mob {
    let ink = Palette::ink();
    let (x, y) = (at[0], at[1]);
    let body = polyline(vec![
        v(x - r, y - r),
        v(x - r, y + 0.3 * r),
        v(x, y + r),
        v(x + r, y + 0.3 * r),
        v(x + r, y - r),
        v(x - r, y - r),
    ])
    .style(faded_line(ink, 3.0, 1.0))
    .build();
    stage.arena_mut().add(body)
}

/// A square grid of `n` by `n` cells as two zig-zag lines (the sides the
/// zigzags run along are grid lines themselves), `side` across, centred at
/// `at`, detached.
fn lattice(stage: &mut Stage<'_>, n: usize, side: f64, at: Vec3, color: Srgb) -> Vec<Mob> {
    let (x0, y0) = (at[0] - side / 2.0, at[1] - side / 2.0);
    let step = side / n as f64;
    let mut across = Vec::new();
    let mut up = Vec::new();
    for k in 0..=n {
        let t = step * k as f64;
        // horizontal lines, alternately left to right and back
        let (a, b) = (x0, x0 + side);
        let (a, b) = if k % 2 == 0 { (a, b) } else { (b, a) };
        across.push(v(a, y0 + t));
        across.push(v(b, y0 + t));
        let (c, d) = (y0, y0 + side);
        let (c, d) = if k % 2 == 0 { (c, d) } else { (d, c) };
        up.push(v(x0 + t, c));
        up.push(v(x0 + t, d));
    }
    let style = faded_line(color, 1.5, 1.0);
    vec![
        stage.arena_mut().add(polyline(across).style(style).build()),
        stage.arena_mut().add(polyline(up).style(style).build()),
    ]
}

// -------------------------------------------------------------- the scene

impl SceneConstruct for CanItFeedItself {
    fn name(&self) -> &str {
        "13_can_it_feed_itself"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange) = (moves(), pushes());
        let table = table_of("K-E1.table")?;
        let thrifty = economy::thrifty();

        // ---- 13_1: people want twenty-four of each; the rounds on the plane
        let line = say(&kit, stage, "13_1")?;
        let demand = numbers("K-E3.demand")?;
        let rounds = numbers("K-E3.rounds")?;
        let shown_rounds = items("K-E3.rounds")?;
        let home = house(stage, v(-6.25, 3.2), 0.28);
        let wants = kit.text_left(
            stage,
            &format!("wants {} of food, {} of tools", whole(demand[0]), whole(demand[1])),
            28.0,
            ink,
            v(-5.7, 3.2),
        )?;
        let mut plane_in = corner_axes(stage);
        plane_in.extend(axis_names(&kit, stage)?);
        let b0 = PlaneArrow::build(stage, PLANE, blue, blue, 8.0, 0.26, [0.0, 0.0], [0.0, 0.0])?;
        fade_in_all(stage, &[home, wants], 0.6)?;
        fade_in_all(stage, &plane_in, 0.6)?;
        b0.start(stage)?;
        tween_all(stage, &[(b0.on, 1.0)], 0.2)?;
        tween_all(stage, &[(b0.tip[0], demand[0]), (b0.tip[1], demand[1])], 0.9)?;
        // the equation, on screen only
        let made_line = kit.text_left(stage, "made = used up + delivered", 30.0, dim, v(COLUMN_X, 3.35))?;
        let equation = kit.tex(stage, r"x = Cx + d", 56.0, ink, &[], v(COLUMN_X + 1.5, 2.55))?;
        line.wait_for(stage, "Making twenty-four of each")?;
        fade_in_all(stage, &[made_line], 0.5)?;
        fade_in_all(stage, &[equation], 0.5)?;
        // each round: what making the last one used up (orange, from the
        // origin) slides to the tip and turns blue; the stack adds it
        let heap = stack(stage, &rounds, blue)?;
        let mut sums = vec![0.0_f64];
        for r in &rounds {
            sums.push(sums[sums.len() - 1] + r);
        }
        let mut made_links: Vec<PlaneArrow> = Vec::new();
        let used_label = kit.text(stage, "used up", 28.0, orange, PLANE.p(52.0, 4.0))?;
        let again_label = kit.text(stage, "so make those\nas well", 26.0, blue, PLANE.p(24.0, 76.0))?;
        let stack_words: Vec<Mob> = {
            let mut out = Vec::new();
            for (i, text) in shown_rounds.iter().take(3).enumerate() {
                out.push(kit.text_left(stage, text, 26.0, ink, v(STACK_X + STACK_W + 0.2, heap[i].1[1]))?);
            }
            out
        };
        fade_in_all(stage, &[heap[0].0, stack_words[0]], 0.4)?;
        for round in 1..=3 {
            let d = rounds[round];
            let orange_arrow = PlaneArrow::build(stage, PLANE, orange, blue, 6.0, 0.22, [0.0, 0.0], [d, d])?;
            orange_arrow.start(stage)?;
            if round == 1 {
                line.wait_for(stage, "uses up sixteen point eight")?;
                fade_in_all(stage, &[used_label], 0.3)?;
            } else if round == 2 {
                line.wait_for(stage, "round after round")?;
            }
            tween_all(stage, &[(orange_arrow.on, 1.0)], 0.4)?;
            if round == 1 {
                line.wait_for(stage, "so make those too")?;
            }
            let at = sums[round];
            tween_all(
                stage,
                &[
                    (orange_arrow.tail[0], at),
                    (orange_arrow.tail[1], at),
                    (orange_arrow.tip[0], at + d),
                    (orange_arrow.tip[1], at + d),
                    (orange_arrow.mix, 1.0),
                ],
                1.0,
            )?;
            if round == 1 {
                fade_out_all(stage, &[used_label], 0.3)?;
                fade_in_all(stage, &[again_label], 0.4)?;
            }
            // the stack grows by this round; the first three are labelled
            let mut ins = vec![heap[round].0];
            if let Some(&word) = stack_words.get(round) {
                ins.push(word);
            }
            fade_in_all(stage, &ins, 0.3)?;
            made_links.push(orange_arrow);
        }
        line.wait_end(stage)?;

        // ---- 13_3: the rounds add up: twenty-four divided by three tenths
        let line = say(&kit, stage, "13_3")?;
        fade_out_all(stage, &[made_line, equation, again_label], 0.5)?;
        let largest = numbers("K-E2.largest")?[0];
        let kept = 1.0 - largest;
        let total = numbers("K-E3.total")?;
        let keeps = kit.text_left(
            stage,
            &format!("keeps {} of equal amounts", show(kept, 1)?),
            30.0,
            ink,
            v(COLUMN_X + 2.0, 2.9),
        )?;
        let divide = kit.tex(
            stage,
            &format!(r"{} \div {} = {}", whole(demand[0]), show(kept, 1)?, whole(total[0])),
            56.0,
            ink,
            &[],
            v(COLUMN_X + 4.0, 1.8),
        )?;
        let used = total[0] * largest;
        let check = kit.text_left(
            stage,
            &format!(
                "check: make {},\nuse up {}, {} left",
                whole(total[0]),
                whole(used),
                whole(total[0] - used)
            ),
            28.0,
            ink,
            v(COLUMN_X + 3.0, 0.45),
        )?;
        fade_in_all(stage, &[keeps], 0.5)?;
        // the rest of the chain walks on (tip to tail, along the line)
        let long = model_rounds(&thrifty, [demand[0], demand[1]], 12);
        let mut tails = Vec::new();
        let mut at = 0.0;
        for r in &long {
            tails.push(at);
            at += r[0];
        }
        let mut extra_links = Vec::new();
        let mut extra_stack = Vec::new();
        for k in 4..long.len() {
            extra_links.push(link(
                stage,
                [tails[k], tails[k]],
                [tails[k] + long[k][0], tails[k] + long[k][1]],
                blue,
                5.0,
            ));
        }
        let remaining: Vec<f64> = long[4..].iter().map(|r| r[0]).collect();
        let mut y = STACK_BASE + sums[4] * PLANE.unit;
        for (i, &a) in remaining.iter().enumerate() {
            let h = (a * PLANE.unit - 0.03).max(0.02);
            let centre = v(STACK_X + STACK_W / 2.0, y + h / 2.0);
            let r = Rectangle::new()
                .width(STACK_W)
                .height(h)
                .style(flat(blue, if (i + 4) % 2 == 0 { 0.95 } else { 0.62 }))
                .build()?
                .moved_to(centre);
            extra_stack.push(stage.arena_mut().add(r));
            y += a * PLANE.unit;
        }
        line.wait_for(stage, "make twenty-four divided by three tenths")?;
        fade_in_all(stage, &[divide], 0.6)?;
        line.wait_for(stage, "Eighty of each")?;
        let cross = cross_mark(stage, PLANE.p(total[0], total[1]), CROSS_HALF, own(), 6.0)?;
        let mut walk = extra_links.clone();
        walk.extend(&extra_stack);
        fade_in_lagged(stage, &walk, 1.6, 0.12)?;
        fade_in_all(stage, &[cross, check], 0.5)?;
        let search = names_to_search(&kit, stage, &["Leontief inverse", "Neumann series"])?;
        fade_in_all(stage, &[search], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 1.0)?;

        // ---- 13_4: a lopsided order settles onto equal amounts
        let line = say(&kit, stage, "13_4")?;
        let mut leaving = vec![wants, keeps, divide, check, cross, search, home];
        leaving.extend(&extra_links);
        leaving.extend(&extra_stack);
        leaving.extend(heap.iter().map(|(m, _)| *m));
        leaving.extend(&stack_words);
        for a in made_links.iter().chain(std::iter::once(&b0)) {
            a.stop(stage);
            leaving.push(a.mob);
        }
        fade_out_all(stage, &leaving, 0.6)?;
        let other = numbers("K-E5.demand")?;
        let shares = numbers("K-E5.shares")?;
        let pattern_mix = numbers("K-E2.mix")?;
        let pattern_other = numbers("K-E2.other")?;
        let by_pattern = numbers("K-E5.totals_by_pattern")?;
        let cross_total = numbers("K-E5.total")?;
        let wants2 = kit.text_left(
            stage,
            &format!("wants {} of food, {} of tools", whole(other[0]), whole(other[1])),
            28.0,
            ink,
            v(-5.7, 3.2),
        )?;
        let house2 = house(stage, v(-6.25, 3.2), 0.28);
        // the rounds from the lopsided order, tip to tail: the first link is
        // the order itself
        let lopsided = model_rounds(&thrifty, [other[0], other[1]], 12);
        let mut at = [0.0, 0.0];
        let mut links = Vec::new();
        for (i, r) in lopsided.iter().enumerate() {
            let width = if i == 0 { 8.0 } else { 5.0 };
            links.push(link(stage, at, [at[0] + r[0], at[1] + r[1]], blue, width));
            at = [at[0] + r[0], at[1] + r[1]];
        }
        fade_in_all(stage, &[house2, wants2], 0.5)?;
        fade_in_all(stage, &links[..1], 0.6)?;
        // the demand as a mix of the two own patterns, and how it was found
        let split = kit.text_left(
            stage,
            &format!(
                "{}, {} = {} of {}, {} + {} of {}, {}",
                whole(other[0]),
                whole(other[1]),
                whole(shares[0]),
                whole(pattern_mix[0]),
                whole(pattern_mix[1]),
                whole(shares[1]),
                whole(pattern_other[0]),
                minus(&whole(pattern_other[1])),
            ),
            26.0,
            ink,
            v(COLUMN_X, 3.3),
        )?;
        let solved = kit.text_left(
            stage,
            &format!(
                "{} + {} \u{d7} {} = {}   and   {} {} {} \u{d7} {} = {}",
                whole(shares[0]),
                whole(pattern_other[0]),
                whole(shares[1]),
                whole(other[0]),
                whole(shares[0]),
                minus("-"),
                whole(-pattern_other[1]),
                whole(shares[1]),
                whole(other[1]),
            ),
            26.0,
            ink,
            v(COLUMN_X, 2.6),
        )?;
        let not_square = kit.text_left(
            stage,
            "these two patterns are not at right angles,\nso no dot-product recipe",
            24.0,
            dim,
            v(COLUMN_X, 1.85),
        )?;
        let keeps_a = kit.text_left(
            stage,
            &format!(
                "a part that keeps {} of itself: {} \u{f7} {} = {}",
                show(kept, 1)?,
                whole(shares[0]),
                show(kept, 1)?,
                whole(by_pattern[0]),
            ),
            26.0,
            ink,
            v(COLUMN_X, 0.9),
        )?;
        // the other pattern keeps one less its eigenvalue, 0.2
        let second = fraction_value(items("K-E2.eigenvalues")?[1])?;
        let keeps_b = kit.text_left(
            stage,
            &format!(
                "the other keeps {} = 1 {} {}: {} \u{f7} {} = {}",
                show(1.0 - second, 1)?,
                minus("-"),
                show(second, 1)?,
                whole(shares[1]),
                show(1.0 - second, 1)?,
                whole(by_pattern[1]),
            ),
            26.0,
            ink,
            v(COLUMN_X, 0.2),
        )?;
        line.wait_for(stage, "It is eighteen of one pattern")?;
        fade_in_all(stage, &[split, solved, not_square], 0.6)?;
        line.wait_for(stage, "each round shrinks each part")?;
        fade_in_all(stage, &[keeps_a, keeps_b], 0.6)?;
        // the rest of the rounds: askew first, then onto the line
        let settle = &links[1..];
        let gold_line = stage.arena_mut().add(
            polyline(vec![PLANE.p(0.0, 0.0), PLANE.p(REACH * 0.9, REACH * 0.9)])
                .style(faded_line(own(), 3.0, 0.0))
                .build(),
        );
        let end = cross_mark(stage, PLANE.p(cross_total[0], cross_total[1]), CROSS_HALF, own(), 6.0)?;
        let end_name = name_the_total(&kit, stage, &cross_total)?;
        line.wait_for(stage, "So the rounds settle")?;
        fade_in_lagged(stage, settle, 2.2, 0.2)?;
        play!(stage; gold_line.animate().set_anim_args(run(0.6))?.set_opacity(0.8)?);
        fade_in_all(stage, &[end, end_name], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, 1.0)?;

        // ---- 13_5: a hungrier economy: make ten of each and eleven is used
        // up; what happens to the rounds?
        say(&kit, stage, "13_5")?;
        let mut leaving = vec![
            wants2, house2, split, solved, not_square, keeps_a, keeps_b, gold_line, end, end_name,
        ];
        leaving.extend(&links);
        leaving.extend(&plane_in);
        fade_out_all(stage, &leaving, 0.6)?;
        let hungry = table_from_tenths("K-E4.table_tenths")?;
        let made = crate::facts::shown::EVEN_MIX;
        let eaten = numbers("K-E4.uses_even")?;
        let bar = |stage: &mut Stage<'_>, x: f64, amount: f64, color: Srgb| -> fmn::Result<Mob> {
            let h = amount * 0.2;
            let r = Rectangle::new()
                .width(0.8)
                .height(h)
                .style(flat(color, 0.9))
                .build()?
                .moved_to(v(x, -1.2 + h / 2.0));
            Ok(stage.arena_mut().add(r))
        };
        let mut hungry_pic = Vec::new();
        for (x, amount, color, name) in [
            (-4.2, made[0] as f64, blue, "food"),
            (-3.0, made[1] as f64, blue, "tools"),
            (3.0, eaten[0], orange, "food"),
            (4.2, eaten[1], orange, "tools"),
        ] {
            hungry_pic.push(bar(stage, x, amount, color)?);
            hungry_pic.push(kit.text(stage, &whole(amount), 34.0, ink, v(x, -1.2 + amount * 0.2 + 0.35))?);
            hungry_pic.push(kit.text(stage, name, 24.0, dim, v(x, -1.55))?);
        }
        hungry_pic.push(kit.text(stage, "made", 32.0, blue, v(-3.6, 1.75))?);
        hungry_pic.push(kit.text(stage, "used up", 32.0, orange, v(3.6, 1.75))?);
        let grid = MatrixGrid::build(&kit, stage, &table_entries(&hungry)?, 44.0, ink, v(0.0, 0.2))?;
        let again = kit.text(stage, "?", 90.0, ink, v(6.0, 0.4))?;
        let next = kit.text(stage, "next round", 26.0, dim, v(6.0, -0.5))?;
        hungry_pic.extend([grid.group, again, next]);
        fade_in_all(stage, &hungry_pic, 0.8)?;
        predict_hold(&kit, stage)?;

        // ---- 13_7: they grow; the test is one number
        // the picture keeps up with the voice (D-16: no closing hold; a card
        // is read after the line only where the storyboard keeps it, here the
        // Leontief card): the chain grows as "They grow" is said, is out by
        // "So the test is one number", and the card with the test is up
        // before the voice reaches "below one"
        let line = say(&kit, stage, "13_7")?;
        fade_out_all(stage, &hungry_pic, 0.3)?;
        let grown = model_rounds(&economy::hungry(), [demand[0], demand[1]], 6);
        let mut plane_again = corner_axes(stage);
        plane_again.extend(axis_names(&kit, stage)?);
        let mut red = Vec::new();
        let mut at = 0.0;
        for r in &grown {
            red.push(link(stage, [at, at], [at + r[0], at + r[1]], warn(), 5.0));
            at += r[0];
        }
        let hungry_numbers = items("K-E4.rounds")?;
        let counts = kit.text_left(
            stage,
            &format!("{}, \u{2026}", hungry_numbers.join(", ")),
            34.0,
            warn(),
            v(COLUMN_X + 1.5, 3.0),
        )?;
        let cannot = kit.text(stage, "that economy cannot\nfeed itself", 28.0, warn(), v(3.8, 1.5))?;
        fade_in_all(stage, &plane_again, 0.3)?;
        line.wait_for(stage, "They grow")?;
        fade_in_lagged(stage, &red, 2.2, 0.35)?;
        fade_in_all(stage, &[counts, cannot], 0.4)?;
        let formula = fact("K-E4.formula_says")?;
        let programmer = lower_third(
            &kit,
            stage,
            "for programmers:",
            &format!(
                "the formula still answers, {} \u{f7} (1 \u{2013} {}) = {} of each; check the eigenvalue, or that the plan has no negative entry",
                whole(demand[0]),
                fact("K-E4.largest")?,
                minus(formula),
            ),
        )?;
        fade_in_all(stage, &[programmer], 0.4)?;
        pause(stage, 0.9)?;
        let mut cleared = red.clone();
        cleared.extend(&plane_again);
        cleared.extend([counts, cannot, programmer]);
        fade_out_all(stage, &cleared, 0.5)?;

        // the test: the largest eigenvalue against one
        line.wait_for(stage, "So the test is one number")?;
        let card = Rectangle::new()
            .width(10.4)
            .height(3.7)
            .corner_radius(0.18)
            .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(v(0.0, 1.2));
        let card = stage.arena_mut().add(card);
        let test = kit.text(
            stage,
            "largest eigenvalue < 1: every demand can be met",
            34.0,
            ink,
            v(0.0, 2.45),
        )?;
        let (x_of, base) = (|e: f64| -3.6 + 4.4 * e, 1.3);
        let axis = stage.arena_mut().add(
            polyline(vec![v(x_of(0.0), base), v(x_of(1.6), base)])
                .style(faded_line(dim, 3.0, 1.0))
                .build(),
        );
        let small = numbers("K-E2.largest")?[0];
        let big = numbers("K-E4.largest")?[0];
        let marks = [(small, good()), (1.0, ink), (big, warn())];
        let mut ticks = Vec::new();
        for &(e, c) in &marks {
            ticks.push(
                stage.arena_mut().add(
                    polyline(vec![v(x_of(e), base - 0.22), v(x_of(e), base + 0.22)])
                        .style(faded_line(c, 6.0, 1.0))
                        .build(),
                ),
            );
        }
        let names = [
            kit.text(stage, &show(small, 1)?, 34.0, good(), v(x_of(small), base + 0.7))?,
            kit.text(stage, "1", 34.0, ink, v(x_of(1.0), base + 0.7))?,
            kit.text(stage, &show(big, 1)?, 34.0, warn(), v(x_of(big), base + 0.7))?,
        ];
        let cards = [card, test, axis];
        fade_in_all(stage, &cards, 0.5)?;
        line.wait_for(stage, "With the largest eigenvalue below one")?;
        fade_in_all(stage, &[ticks[0], names[0], ticks[1], names[1]], 0.5)?;
        let below = precisely(
            &kit,
            stage,
            &format!(
                "for a table with no negative entry and a demand with none: below one, every demand can be met (Hawkins and Simon, {}), given the workers, land and machines that are outside the table",
                fact("K-S.hawkins_simon_year")?
            ),
        )?;
        fade_in_all(stage, &[below], 0.5)?;
        line.wait_for(stage, "At one or more")?;
        fade_in_all(stage, &[ticks[2], names[2]], 0.5)?;
        fade_out_all(stage, &[below], 0.3)?;
        let above = precisely(
            &kit,
            stage,
            "at one or more, some demand fails, and every demand when each industry feeds every other, directly or indirectly",
        )?;
        fade_in_all(stage, &[above], 0.5)?;
        line.wait_end(stage)?;

        // the same kind of table, 44 sectors wide: the Leontief card is read
        // after the line (D-16 keeps it); the test card has been up since
        // "below one", so it goes as the line ends
        let mut shown_card = vec![card, test, axis, above];
        shown_card.extend(&ticks);
        shown_card.extend(names);
        fade_out_all(stage, &shown_card, 0.5)?;
        let small_grid = MatrixGrid::build(&kit, stage, &table_entries(&table)?, 44.0, ink, v(-3.4, 0.9))?;
        let sectors = fact("K-S.leontief.sectors")?;
        let lattice_at = v(2.6, 0.9);
        let big_grid = lattice(
            stage,
            sectors.parse::<usize>().map_err(|e| fail(e.to_string()))?,
            3.0,
            lattice_at,
            dim,
        );
        let wide = kit.text(stage, &format!("{sectors} sectors wide"), 28.0, dim, v(2.6, -0.95))?;
        let mut grids = vec![small_grid.group, wide];
        grids.extend(&big_grid);
        let leontief = lower_third(
            &kit,
            stage,
            "Wassily Leontief:",
            &format!(
                "built such tables for the American economy, {sectors} sectors wide (his {} book, for 1919 and 1929)",
                fact("K-S.leontief.book_year")?
            ),
        )?;
        let prize = lower_third(
            &kit,
            stage,
            "Leontief:",
            &format!(
                "Nobel prize in economics, {}, for the development of the input-output method and for its application to important economic problems",
                fact("K-S.leontief.prize_year")?
            ),
        )?;
        grids.push(leontief);
        fade_in_all(stage, &grids, 0.6)?;
        pause(stage, 1.3)?;
        cross_fade(stage, &[leontief], &[prize], 0.4)?;
        pause(stage, 0.8)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// 13_1 and 13_3: the rounds the picture walks are the model's, each
    /// seven tenths of the last, and they add to the total of the facts.
    #[test]
    fn the_rounds_are_seven_tenths_of_the_last_and_add_to_eighty() {
        let demand = numbers("K-E3.demand").unwrap();
        let rounds = numbers("K-E3.rounds").unwrap();
        assert_eq!(demand, [24.0, 24.0]);
        assert_eq!(rounds, [24.0, 16.8, 11.76, 8.232]);
        let chain = model_rounds(&economy::thrifty(), [demand[0], demand[1]], 12);
        assert_eq!(chain.len(), 13);
        for (i, r) in rounds.iter().enumerate() {
            assert!(
                (chain[i][0] - r).abs() < 1e-9 && (chain[i][1] - r).abs() < 1e-9,
                "round {i}"
            );
        }
        for w in chain.windows(2) {
            assert!((w[1][0] / w[0][0] - 0.7).abs() < 1e-12);
        }
        let total = numbers("K-E3.total").unwrap();
        assert_eq!(total, [80.0, 80.0]);
        // the rounds added so far have reached 99% of it by the thirteenth
        let sum: f64 = chain.iter().map(|r| r[0]).sum();
        assert!(sum > 0.99 * total[0] && sum < total[0], "{sum}");
        assert_eq!(fact("K-E3.rounds_for_99_percent").unwrap(), "13");
        let partial: f64 = chain[..12].iter().map(|r| r[0]).sum();
        assert!(partial < 0.99 * total[0], "{partial}");
        // twenty-four divided by three tenths is eighty; made 80, used up
        // 56, 24 left
        let kept = 1.0 - numbers("K-E2.largest").unwrap()[0];
        assert_eq!(show(kept, 1).unwrap(), "0.3");
        assert_eq!(whole(demand[0] / kept), "80");
        let used = total[0] * numbers("K-E2.largest").unwrap()[0];
        assert_eq!(whole(used), "56");
        assert_eq!(whole(total[0] - used), whole(demand[0]));
        // the stack at the right is at the plane's scale and stays on screen
        assert!(STACK_BASE + total[0] * PLANE.unit < 3.0);
        // the demand arrow's tip and the cross are inside the plane's reach
        assert!(total[0] < REACH);
    }

    /// 13_4: the lopsided order is eighteen of one own pattern and four of
    /// the other, each part kept by its eigenvalue's complement, and the
    /// rounds do settle onto the gold line and the cross at 75, 50.
    #[test]
    fn the_lopsided_order_splits_and_its_rounds_settle_on_the_cross() {
        let other = numbers("K-E5.demand").unwrap();
        let shares = numbers("K-E5.shares").unwrap();
        let (mix, rest) = (numbers("K-E2.mix").unwrap(), numbers("K-E2.other").unwrap());
        assert_eq!(other, [30.0, 10.0]);
        assert_eq!(shares, [18.0, 4.0]);
        // 18 + 3 * 4 = 30 and 18 - 2 * 4 = 10
        assert_eq!(shares[0] * mix[0] + shares[1] * rest[0], other[0]);
        assert_eq!(shares[0] * mix[1] + shares[1] * rest[1], other[1]);
        // each part divided by what the economy keeps of it
        let by = numbers("K-E5.totals_by_pattern").unwrap();
        assert_eq!(by, [60.0, 5.0]);
        assert!((shares[0] / 0.3 - by[0]).abs() < 1e-12);
        assert!((shares[1] / 0.8 - by[1]).abs() < 1e-12);
        // 60 of (1, 1) and 5 of (3, -2) is the total
        let total = numbers("K-E5.total").unwrap();
        assert_eq!(total, [75.0, 50.0]);
        assert_eq!(by[0] * mix[0] + by[1] * rest[0], total[0]);
        assert_eq!(by[0] * mix[1] + by[1] * rest[1], total[1]);
        // the rounds in the facts, and the model's, which keep going to the
        // total
        let chain = model_rounds(&economy::thrifty(), [other[0], other[1]], 25);
        let words: Vec<String> = chain[..4]
            .iter()
            .map(|r| {
                format!(
                    "{};{}",
                    show(r[0], 2).unwrap().trim_end_matches('0').trim_end_matches('.'),
                    show(r[1], 2).unwrap().trim_end_matches('0').trim_end_matches('.')
                )
            })
            .collect();
        assert_eq!(words.join(" "), fact("K-E5.rounds").unwrap());
        let (mut x, mut y) = (0.0, 0.0);
        for r in &chain {
            x += r[0];
            y += r[1];
        }
        assert!((x - total[0]).abs() < 0.5 && (y - total[1]).abs() < 0.5, "{x} {y}");
        // the links turn toward equal amounts: the angle to the even line
        // shrinks every round
        let off = |r: &[f64; 2]| (r[1] / r[0] - 1.0).abs();
        assert!(chain.windows(2).all(|w| off(&w[1]) < off(&w[0])));
    }

    /// 13_4: "75, 50" is clear of both arms of the cross it names, of the
    /// gold line and of the chain of rounds. The cross's lower right arm ran
    /// through its "7".
    #[test]
    fn the_name_of_the_total_stands_clear_of_the_cross_and_the_chain() {
        use super::super::draw::testing::Room;
        super::super::carts::probe::run_scene(|stage| {
            let kit = Kit::new()?;
            let total = numbers("K-E5.total")?;
            let name = name_the_total(&kit, stage, &total)?;
            let room = Room::of(stage, name);
            let c = PLANE.p(total[0], total[1]);
            // the cross's two arms and the gold line, a stroke's half width
            // (6 pt) round each
            let arm = |dx: f64, dy: f64| (v(c[0] - dx, c[1] - dy), v(c[0] + dx, c[1] + dy));
            let arms = [arm(CROSS_HALF, -CROSS_HALF), arm(CROSS_HALF, CROSS_HALF)];
            let line = (PLANE.p(0.0, 0.0), PLANE.p(REACH * 0.9, REACH * 0.9));
            let pad = 0.06;
            for (a, b) in arms.into_iter().chain([line]) {
                assert!(!room.grown(pad).crosses(a, b), "{room:?} is crossed by {a:?}-{b:?}");
            }
            // the chain of rounds from the lopsided order, tip to tail
            let other = numbers("K-E5.demand")?;
            let mut at = [0.0, 0.0];
            for r in model_rounds(&economy::thrifty(), [other[0], other[1]], 12) {
                let next = [at[0] + r[0], at[1] + r[1]];
                let (a, b) = (PLANE.p(at[0], at[1]), PLANE.p(next[0], next[1]));
                assert!(
                    !room.grown(pad).crosses(a, b),
                    "{room:?} is crossed by the chain {at:?}-{next:?}"
                );
                at = next;
            }
            // beside the cross, not far from it
            assert!(room.x0 > c[0] + CROSS_HALF && room.x0 - c[0] < 0.7, "{room:?}");
            Ok(())
        });
    }

    /// 13_5 and 13_7: the hungrier economy's rounds grow, its eigenvalue is
    /// above one, the formula gives a negative total, and the lattice is as
    /// wide as Leontief's tables.
    #[test]
    fn the_hungry_rounds_grow_and_the_test_is_the_largest_eigenvalue() {
        let t = table_from_tenths("K-E4.table_tenths").unwrap();
        assert_eq!(t, [[0.6, 0.5], [0.4, 0.7]]);
        let eaten = numbers("K-E4.uses_even").unwrap();
        assert_eq!(eaten, [11.0, 11.0]);
        let grown = model_rounds(&economy::hungry(), [24.0, 24.0], 6);
        let facts = numbers("K-E4.rounds").unwrap();
        for (i, f) in facts.iter().enumerate() {
            assert!((grown[i][0] - f).abs() < 1e-9, "round {i}");
        }
        assert!(grown.windows(2).all(|w| w[1][0] > w[0][0]), "they grow");
        assert!(numbers("K-E4.largest").unwrap()[0] > 1.0);
        assert!(numbers("K-E2.largest").unwrap()[0] < 1.0);
        assert_eq!(fact("K-E4.feeds_itself").unwrap(), "false");
        assert_eq!(fact("K-E4.formula_says").unwrap(), "-240");
        let h = numbers("K-E4.largest").unwrap()[0];
        assert_eq!(whole(24.0 / (1.0 - h)), "-240");
        // the chain leaves the frame at the top within six links, which is
        // what the picture shows
        let leaves = grown.iter().scan(0.0, |s, r| {
            *s += r[1];
            Some(-2.2 + PLANE.unit * *s)
        });
        assert!(leaves.into_iter().any(|y| y > 4.0));
        assert_eq!(fact("K-S.leontief.sectors").unwrap(), "44");
        assert_eq!(fact("K-S.hawkins_simon_year").unwrap(), "1949");
        assert_eq!(fact("K-S.leontief.book_year").unwrap(), "1941");
        assert_eq!(fact("K-S.leontief.prize_year").unwrap(), "1973");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("13_can_it_feed_itself").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "wants 24 of food, 24 of tools",
            "made = used up + delivered",
            "x = Cx + d",
            "so make those\nas well",
            "24 \\div 0.3 = 80",
            "check: make 80,\nuse up 56, 24 left",
            "wants 30 of food, 10 of tools",
            "30, 10 = 18 of 1, 1 + 4 of 3,",
            "18 \u{f7} 0.3 = 60",
            "4 \u{f7} 0.8 = 5",
            "that economy cannot\nfeed itself",
            "largest eigenvalue < 1: every demand can be met",
            "= \u{2013}240 of each",
            "44 sectors wide",
            "Hawkins and Simon, 1949",
            "1973",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // no lower-third was cut short
        assert!(
            words.iter().all(|w| !w.contains('\u{2026}') || w.starts_with("24")),
            "{words:?}"
        );
        assert!(ran.seconds > 40.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }

    /// 13_7: the chain, the test card and the Leontief card keep up with the voice; the Leontief card read after the line is the authored tail (D-16 keeps it) and ends under the corpus cue check's 5 s.
    #[test]
    fn the_picture_after_the_last_line_stays_under_the_cue_checks_tail() {
        let voiced = testing::run_voiced("13_can_it_feed_itself").unwrap();
        let tail = voiced.tail_seconds();
        eprintln!("TAIL 13_can_it_feed_itself: {tail:.2} s");
        assert!(
            tail <= 5.0,
            "13_can_it_feed_itself: {tail:.2} s of picture after the last line"
        );
    }
}
