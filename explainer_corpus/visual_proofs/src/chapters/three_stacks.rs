//! Chapter 07, three stacks: the owner's fourth picture, Siu's. Four square
//! slabs, one tile to four tiles on a side, lift off the board and stack in
//! a corner (07_1); the quiz, how few of these stacks, turned, make a solid
//! box if the cubes that end up on top may be sliced (07_2); three copies,
//! the second and the third turned and slid in (07_3); the locked block taken
//! apart slab by slab, the L-shaped layer each slab sheds being the one the
//! slab above is given by the other two stacks (07_4); and put back, with its
//! braces (07_5).
//!
//! The stacks are `model::stacks::placed(4, 0..3)`, cube for cube: the second
//! and the third are turned by the model's own turns (as quarter turns, found
//! by search and checked), and the scene fails if a stack does not end on the
//! cells the model gives it. The camera never moves; the pictures stand where
//! `siu::VIEW` puts them.

use explainer_kit::{Kit, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    ASIDE_AT, QuizCard, QuizLook, Words, aside, boundary, dim, fact, fact_ints, fail, fit_width, flat, group, hero,
    ink, line_style, panel, pause, say, size,
};
use super::iso::{Anchor, Iso, P3, Solid, box_cells};
use super::kinetics::{Moves, arrive, leave, play, prepared};
use super::siu::{ROW, VIEW, WAIT, colour, edge_brace, legend, level, min_corner, placed, quarters_for};
use super::sound;
use crate::facts::shown;
use crate::model::{layers, stacks};

pub struct ThreeStacks {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["07_1", "07_2", "07_3", "07_4", "07_5"];

// the chapter's fixed words
const QUESTION_CARD: &str = "? stacks make a box";
/// Both rules of the question, kept under the card for the whole countdown
/// (R2: the quiz is read once, then seven silent seconds).
const RULES_CARD: [&str; 2] = [
    "fewest stacks, each turned to fit;",
    "cubes that end up on top may be sliced in half",
];
/// The tag at 07_3, once the three sit together: why three is the fewest.
const FEWEST_TAG: &str = "why three is the fewest";
const LEGEND_CAPTION: &str = "copies";
const BLOCK_WORDS: [&str; 3] = ["4", "5", "4"];
const STEP_WORDS: &str = "staircase";
const TOP_WORDS: &str = "top face";
const CUBES_WORD: &str = "cubes";
const SQUARES_WORD: &str = "squares";

// ---------------------------------------------------------------- geometry

/// Where the four slabs lie on the board before they stack (the stack's
/// base is where its first stack stands), biggest first: they sit along the
/// screen's own horizontal, each a little further back.
pub const SLAB_AT: [P3; 4] = [[0.0, 0.0, 0.0], [5.0, 3.0, 0.0], [9.0, 5.0, 0.0], [12.0, 7.0, 0.0]];
/// How high a slab hovers over its place before it comes down onto it (the
/// slab goes to level 1, 2, 3 of the stack).
pub const HOVER: f64 = 1.0;
/// Where the counter and the legend are on screen.
pub const COUNTER_AT: Vec3 = [4.4, 1.6, 0.0];
pub const LEGEND_AT: Vec3 = [4.4, 3.2, 0.0];

/// The point of the screen under lattice point `p` a little: where a label
/// that names a slab stands, below its front edge.
fn below(iso: &Iso, stage: &Stage<'_>, p: P3, drop: f64) -> Vec3 {
    let s = iso.project(stage, p);
    v(s[0], s[1] - drop)
}

/// 07_3's beats (seconds): a quarter turn, the lining up and the slide in.
/// The first draft's were 0.8, 0.5 and 0.9: the two copies took 7 s from the
/// line's start to be seated and the voice (5.4 s) was done 3 s before, so the
/// next line waited 3.7 s on the picture. The turns are still seen one by one
/// (0.65 s is a quarter turn at an easy pace).
const TURN_S: f64 = 0.65;
const ALIGN_S: f64 = 0.35;
const SLIDE_S: f64 = 0.7;

/// Moves of the cube kit and arrivals together, started `lag` apart.
fn with(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>, extra: Moves, lag: f64) -> fmn::Result<()> {
    let mut all = prepared(stage, moves)?;
    all.extend(extra);
    play(stage, all, lag)
}

/// A solid of the stack of `n` levels in colour `c`, hidden, standing at
/// `at`.
fn stack(iso: &Iso, stage: &mut Stage<'_>, n: i64, c: Srgb, at: P3) -> fmn::Result<Solid> {
    iso.solid(stage, &stacks::standard(n), c, at)
}

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    layers: Vec<i64>,
    stack: i64,
    block: Vec<i64>,
    top: i64,
    top_layer: i64,
    answer: String,
}

fn reads() -> fmn::Result<Reads> {
    let n = shown::STACK;
    let want = |key: &str, got: i64, model: i64| -> fmn::Result<i64> {
        if got == model {
            Ok(got)
        } else {
            Err(fail(format!("{key} is {got} but the model has {model}")))
        }
    };
    let layers_f = fact_ints("F-S.layers")?;
    let model_layers: Vec<i64> = (0..n).map(|l| (n - l) * (n - l)).collect();
    if layers_f != model_layers {
        return Err(fail(format!(
            "F-S.layers is {layers_f:?}, the stack's levels {model_layers:?}"
        )));
    }
    let stack_f = want(
        "F-S.stack",
        super::draw::fact_int("F-S.stack")?,
        stacks::standard(n).len() as i64,
    )?;
    let block = fact_ints("F-S.block")?;
    let top = want(
        "F-S.top",
        super::draw::fact_int("F-S.top")?,
        stacks::top(n).len() as i64,
    )?;
    let top_layer = want("F-S.top.layer", super::draw::fact_int("F-S.top.layer")?, n * (n + 1))?;
    let answer = fact("F-Q.stacks.answer")?;
    Ok(Reads {
        layers: layers_f,
        stack: stack_f,
        block,
        top,
        top_layer,
        answer,
    })
}

/// A small legend chip: the caption and one swatch with its word for each of
/// the three copies, on a panel. One group, detached.
fn legend_chip(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let mut parts = Vec::new();
    let caption = kit.text_left(stage, LEGEND_CAPTION, 24.0, dim(), v(at[0] - 2.1, at[1]))?;
    parts.push(caption);
    let mut x = at[0] - 1.2;
    for (word, c) in legend() {
        let swatch = stage.arena_mut().add(
            Polygon::new(vec![
                v(x, at[1] - 0.12),
                v(x + 0.24, at[1] - 0.12),
                v(x + 0.24, at[1] + 0.12),
                v(x, at[1] + 0.12),
            ])
            .style(flat(c, 1.0))
            .build(),
        );
        let w = kit.text_left(stage, word, 24.0, ink(), v(x + 0.34, at[1]))?;
        x += 0.34 + stage.get_width(w) + 0.3;
        parts.push(swatch);
        parts.push(w);
    }
    let back = stage.arena_mut().add(
        Rectangle::new()
            .width(4.6)
            .height(0.62)
            .corner_radius(0.1)
            .style(Style::default().fill(panel(), 0.94).stroke(dim(), 1.5, 1.0))
            .build()?
            .moved_to(v(at[0], at[1])),
    );
    let mut all = vec![back];
    all.extend(parts);
    group(stage, &all)
}

/// A flat marker over a face of the lattice: the polygon of the cells'
/// outline at height `z`, shifted by `(dx, dy)`, projected by the view.
fn marker(
    iso: &Iso,
    stage: &mut Stage<'_>,
    cells: &[(i64, i64)],
    z: f64,
    shift: (f64, f64),
    color: Srgb,
) -> fmn::Result<Mob> {
    let loops = boundary(cells);
    let corners: Vec<Vec3> = loops
        .first()
        .ok_or_else(|| fail("a marker needs cells".into()))?
        .iter()
        .map(|&(x, y)| iso.project(stage, [x as f64 + shift.0, y as f64 + shift.1, z]))
        .collect();
    Ok(stage.arena_mut().add(
        Polygon::new(corners)
            .style(Style::default().fill(color, 0.55).stroke(color, 2.5, 1.0))
            .build(),
    ))
}

impl SceneConstruct for ThreeStacks {
    fn name(&self) -> &str {
        "07_three_stacks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        super::draw::shelf_so_far(&kit, stage, 7)?;
        let r = reads()?;
        let n = shown::STACK;
        let iso = Iso::new(stage, VIEW)?;

        // ---------------------------------------------------------- 07_1
        let line = say(&kit, stage, LINES[0])?;
        // the four slabs on the board, one tile to four on a side, each with
        // its count under its front edge
        let sizes = [4_i64, 3, 2, 1];
        let colours = [size(4), size(3), size(2), size(1)];
        let mut slabs: Vec<Solid> = Vec::new();
        let mut counts: Vec<Mob> = Vec::new();
        for (i, &k) in sizes.iter().enumerate() {
            slabs.push(iso.solid(stage, &box_cells(k, k, 1), colours[i], SLAB_AT[i])?);
            let label = kit.text(stage, &r.layers[i].to_string(), 34.0, ink(), ORIGIN)?;
            let at = below(&iso, stage, [SLAB_AT[i][0] + k as f64 / 2.0, SLAB_AT[i][1], 0.0], 0.3);
            stage.move_to(label, at, ORIGIN);
            counts.push(label);
        }
        // "Now add up the squares": the four squares come up, the smallest
        // first (the frame was empty for the first four seconds of the line)
        line.wait_for(stage, "Now add up the squares")?;
        #[cfg(test)]
        super::draw::quiz_log::record(&kit.chapter, "07_1", "slabs_in", stage.scene().time().to_f64());
        let mut coming = Vec::new();
        for i in [3usize, 2, 1, 0] {
            coming.push(iso.fade(slabs[i], 1.0, 0.5)?);
        }
        iso.play(stage, coming, 0.25)?;
        // one, four, nine, sixteen: each square's count as it is said
        for (word, i) in [("One,", 3usize), ("four", 2), ("nine", 1), ("sixteen", 0)] {
            line.wait_for(stage, word)?;
            #[cfg(test)]
            if i == 3 {
                super::draw::quiz_log::record(&kit.chapter, "07_1", "first_count", stage.scene().time().to_f64());
            }
            let label = arrive(stage, counts[i], 0.5)?;
            play(stage, vec![label], 0.0)?;
        }
        // stack them, biggest at the bottom, every level in the same corner
        line.wait_sentence(stage, 3)?;
        let counter = stage.add_value_tracker(r.layers[0] as f64);
        let count_mob = super::draw::live_counter(&kit, stage, counter, 96.0, ink(), COUNTER_AT)?;
        let mut running = r.layers[0];
        for (i, &z) in [1.0, 2.0, 3.0].iter().enumerate() {
            let slab = slabs[i + 1];
            let up = iso.slide_by(stage, slab, [0.0, 0.0, z + HOVER], 0.35)?;
            iso.play(stage, vec![up], 0.0)?;
            let over = iso.slide(stage, slab, [0.0, 0.0, z + HOVER], 0.55)?;
            iso.play(stage, vec![over], 0.0)?;
            running += r.layers[i + 1];
            let down = iso.slide_by(stage, slab, [0.0, 0.0, -HOVER], 0.3)?;
            let count = tween(counter, running as f64, 0.3)?;
            let glow = iso.glow(slab, colour(0), 1.0, 0.4)?;
            iso.play(stage, vec![down, count, glow], 0.0)?;
        }
        if running != r.stack {
            return Err(fail(format!("the levels add to {running}, the stack has {}", r.stack)));
        }
        let base_glow = iso.glow(slabs[0], colour(0), 1.0, 0.3)?;
        iso.play(stage, vec![base_glow], 0.0)?;
        // the four slabs are one stack now: the same cubes, in the stack's colour
        let stack_a = stack(&iso, stage, n, colour(0), WAIT[0])?;
        iso.show(stage, stack_a)?;
        let mut hide = Vec::new();
        for &s in &slabs {
            hide.push(iso.fade(s, 0.0, 0.05)?);
        }
        iso.play(stage, hide, 0.0)?;
        line.wait_end(stage)?;
        let mut gone: Moves = Vec::new();
        for &m in counts.iter().chain([&count_mob]) {
            gone.push(leave(stage, m, 0.4)?);
        }
        play(stage, gone, 0.0)?;

        // ---------------------------------------------------------- 07_2
        let line = say(&kit, stage, LINES[1])?;
        let ghost = stack(&iso, stage, n, dim(), WAIT[1])?;
        let card = QuizCard::build(
            &kit,
            stage,
            "stacks",
            Words::Text(QUESTION_CARD),
            Words::Text(&r.answer),
            QuizLook::BESIDE,
        )?;
        // the two rules, under the card, up for the whole countdown
        let (_, card_h) = QuizLook::BESIDE.size();
        let rules_top = QuizLook::BESIDE.at[1] - card_h / 2.0 - 0.45;
        let mut rule_lines = Vec::new();
        for (i, words) in RULES_CARD.iter().enumerate() {
            let m = kit.text(
                stage,
                words,
                22.0,
                ink(),
                v(QuizLook::BESIDE.at[0], rules_top - 0.36 * i as f64),
            )?;
            fit_width(stage, m, 5.0);
            rule_lines.push(m);
        }
        let rules = group(stage, &rule_lines)?;
        line.wait_sentence(stage, 3)?;
        let ghost_in = iso.fade(ghost, 0.3, 0.6)?;
        iso.play(stage, vec![ghost_in], 0.0)?;
        line.wait_last(stage)?;
        card.show(stage, 0.5)?;
        let rules_in = arrive(stage, rules, 0.5)?;
        play(stage, vec![rules_in], 0.0)?;
        card.count(&kit, stage)?;

        // ---------------------------------------------------------- 07_3
        let line = say(&kit, stage, LINES[2])?;
        // the card goes; the ghost becomes the second copy, and the third
        // arrives; a legend chip names them
        let stack_c = stack(&iso, stage, n, colour(2), WAIT[2])?;
        iso.recolour(ghost, colour(1));
        let chip = legend_chip(&kit, stage, LEGEND_AT)?;
        let card_mobs = card.mobs();
        let mut outs: Moves = Vec::new();
        for &m in &card_mobs {
            outs.push(leave(stage, m, 0.4)?);
        }
        outs.push(leave(stage, rules, 0.4)?);
        outs.push(arrive(stage, chip, 0.5)?);
        let fade_b = iso.fade(ghost, 1.0, 0.5)?;
        let fade_c = iso.fade(stack_c, 1.0, 0.5)?;
        with(stage, vec![fade_b, fade_c], outs, 0.0)?;
        // a gold dot on each stack's one-cube apex (the cube at (0, 0, 3))
        let mut dots: Vec<Mob> = Vec::new();
        for &s in &[stack_a, ghost, stack_c] {
            let dot = stage.arena_mut().add(
                Circle::new()
                    .radius(0.1)
                    .arc_center(ORIGIN)
                    .style(flat(hero(), 1.0))
                    .build(),
            );
            iso.pin(stage, dot, Anchor::On(s, [0.5, 0.5, n as f64]), [0.0, 0.0, 0.0])?;
            dots.push(dot);
        }
        let dots_in: Moves = dots
            .iter()
            .map(|&d| arrive(stage, d, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, dots_in, 0.0)?;
        // the second, then the third: turned in place, lined up, slid in
        for (which, solid) in [(1usize, ghost), (2, stack_c)] {
            let seq = quarters_for(stacks::TURNS[which])
                .ok_or_else(|| fail(format!("no quarter turns for stack {which}")))?;
            let wait = WAIT[which];
            let pivot = [
                wait[0] + n as f64 / 2.0,
                wait[1] + n as f64 / 2.0,
                wait[2] + n as f64 / 2.0,
            ];
            for (axis, q) in seq {
                let m = iso.turn(stage, solid, axis, q, pivot, TURN_S)?;
                iso.play(stage, vec![m], 0.0)?;
            }
            let target = min_corner(&placed(n, which));
            let align = iso.land(stage, solid, [wait[0], target[1], target[2]], ALIGN_S)?;
            iso.play(stage, vec![align], 0.0)?;
            let slide_in = iso.land(stage, solid, target, SLIDE_S)?;
            iso.play(stage, vec![slide_in], 0.0)?;
            let mut want = placed(n, which);
            want.sort_unstable();
            if iso.cells(stage, solid).as_deref() != Some(want.as_slice()) {
                return Err(fail(format!("stack {which} did not end on the model's cells")));
            }
        }
        // R2 (Dev D5): "fewest" was shown by nothing; with the three seated, a
        // tag says why one and two stacks cannot (the arithmetic is in the
        // description: F-S.fewest)
        let why = aside(
            &kit,
            stage,
            FEWEST_TAG,
            &format!(
                "one or two stacks, {} or {} cubes: no such box. The arithmetic is in the description.",
                r.stack,
                2 * r.stack
            ),
            ASIDE_AT,
        )?;
        let why_in = arrive(stage, why, 0.5)?;
        play(stage, vec![why_in], 0.0)?;
        line.wait_end(stage)?;
        // the three stacks sit together: the arrangement is a finished chord
        sound::bloom(&kit, stage, "stacks")?;
        let mut dots_out: Moves = dots
            .iter()
            .map(|&d| leave(stage, d, 0.3))
            .collect::<fmn::Result<Moves>>()?;
        dots_out.push(leave(stage, why, 0.3)?);
        play(stage, dots_out, 0.0)?;
        for &d in &dots {
            iso.unpin(d);
        }

        // ---------------------------------------------------------- 07_4
        let line = say(&kit, stage, LINES[3])?;
        // the stacks are cut into the five slabs of the block, one solid a
        // slab and a colour, in the same places
        let mut level_solids: Vec<Vec<(usize, Solid)>> = Vec::new();
        for z in 0..=n {
            let mut here = Vec::new();
            for which in 0..3 {
                let cells = level(n, which, z);
                if cells.is_empty() {
                    continue;
                }
                let s = iso.solid(stage, &cells, colour(which), [0.0; 3])?;
                iso.show(stage, s)?;
                here.push((which, s));
            }
            level_solids.push(here);
        }
        let mut hide = Vec::new();
        for s in [stack_a, ghost, stack_c] {
            hide.push(iso.fade(s, 0.0, 0.05)?);
        }
        iso.play(stage, hide, 0.0)?;
        // four gold seams on the block's front and right faces
        let mut seams: Vec<Mob> = Vec::new();
        for z in 1..=n {
            let zf = z as f64;
            for (a, b) in [
                ([0.0, 0.0, zf], [n as f64, 0.0, zf]),
                ([n as f64, 0.0, zf], [n as f64, (n + 1) as f64, zf]),
            ] {
                let (pa, pb) = (iso.project(stage, a), iso.project(stage, b));
                seams.push(
                    stage
                        .arena_mut()
                        .add(polyline(vec![pa, pb]).style(line_style(hero(), 3.0)).build()),
                );
            }
        }
        let seams_in: Moves = seams
            .iter()
            .map(|&m| arrive(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, seams_in, 0.0)?;
        // the step lifts and slides, then the slabs 3, 2, 1: a row, face up
        let seams_out: Moves = seams
            .iter()
            .map(|&m| leave(stage, m, 0.3))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, seams_out, 0.0)?;
        for z in (1..=n as usize).rev() {
            let (rx, ry) = ROW[z];
            let mut across = Vec::new();
            for &(_, s) in &level_solids[z] {
                across.push(iso.slide_by(stage, s, [rx, ry, 0.0], 0.6)?);
            }
            iso.play(stage, across, 0.0)?;
            let mut down = Vec::new();
            for &(_, s) in &level_solids[z] {
                down.push(iso.slide_by(stage, s, [0.0, 0.0, -(z as f64)], 0.4)?);
            }
            iso.play(stage, down, 0.0)?;
        }
        line.wait_sentence(stage, 1)?;
        // the counts of each slab, and the blue's under them
        let mut tops: Vec<Mob> = Vec::new();
        for (z, &(rx, ry)) in ROW.iter().enumerate().take(n as usize + 1) {
            let cells = if z == n as usize { r.top } else { r.top_layer };
            let label = kit.text(stage, &cells.to_string(), 36.0, ink(), ORIGIN)?;
            let at = below(&iso, stage, [rx + 2.0, ry, 0.0], 0.32);
            stage.move_to(label, at, ORIGIN);
            tops.push(label);
        }
        let tops_in: Moves = tops
            .iter()
            .map(|&m| arrive(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, tops_in, 0.0)?;
        line.wait_sentence(stage, 2)?;
        // each slab's blue square gives up an L-shaped layer, and the slab
        // above has the other two stacks in it
        let mut pulses: Vec<Mob> = Vec::new();
        for z in 1..=n as usize {
            let side = n - z as i64 + 1;
            let l: Vec<(i64, i64)> = layers::layer(side);
            let below_slab = marker(&iso, stage, &l, 1.0, ROW[z - 1], hero())?;
            let above_slab = marker(&iso, stage, &l, 1.0, ROW[z], hero())?;
            let moves: Moves = vec![arrive(stage, below_slab, 0.35)?, arrive(stage, above_slab, 0.35)?];
            play(stage, moves, 0.0)?;
            pause_short(stage)?;
            let off: Moves = vec![leave(stage, below_slab, 0.3)?, leave(stage, above_slab, 0.3)?];
            play(stage, off, 0.0)?;
            pulses.extend([below_slab, above_slab]);
        }
        // three tallies: thirty, thirty, thirty
        let mut tallies: Vec<Mob> = Vec::new();
        for which in 0..3 {
            let t = kit.text(
                stage,
                &r.stack.to_string(),
                54.0,
                colour(which),
                v(COUNTER_AT[0] + 0.9 * which as f64 - 0.9, 0.2),
            )?;
            tallies.push(t);
        }
        let tallies_in: Moves = tallies
            .iter()
            .map(|&m| arrive(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, tallies_in, 0.3)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 07_5
        let line = say(&kit, stage, LINES[4])?;
        let away: Moves = tops
            .iter()
            .chain(&tallies)
            .map(|&m| leave(stage, m, 0.3))
            .collect::<fmn::Result<Moves>>()?;
        play(stage, away, 0.0)?;
        // the slabs go back, the lowest first
        for z in 1..=n as usize {
            let (rx, ry) = ROW[z];
            let mut up = Vec::new();
            for &(_, s) in &level_solids[z] {
                up.push(iso.slide_by(stage, s, [0.0, 0.0, z as f64], 0.35)?);
            }
            iso.play(stage, up, 0.0)?;
            let mut back = Vec::new();
            for &(_, s) in &level_solids[z] {
                back.push(iso.slide_by(stage, s, [-rx, -ry, 0.0], 0.55)?);
            }
            iso.play(stage, back, 0.0)?;
        }
        // braces: 4 wide, 5 deep, 4 tall
        let sb = r.block.clone();
        let (w, d, h) = (sb[0] as f64, sb[1] as f64, sb[2] as f64);
        let front = [iso.project(stage, [0.0, 0.0, 0.0]), iso.project(stage, [w, 0.0, 0.0])];
        let side = [iso.project(stage, [w, 0.0, 0.0]), iso.project(stage, [w, d, 0.0])];
        let tall = [iso.project(stage, [0.0, 0.0, h]), iso.project(stage, [0.0, 0.0, 0.0])];
        let mut braces: Vec<Mob> = Vec::new();
        for (edge, out, word) in [
            (front, v(0.0, -1.0), BLOCK_WORDS[0]),
            (side, v(1.0, -0.6), BLOCK_WORDS[1]),
            (tall, v(-1.0, 0.0), BLOCK_WORDS[2]),
        ] {
            let (b, reach) = edge_brace(stage, edge[0], edge[1], out, ink())?;
            let mid = v((edge[0][0] + edge[1][0]) / 2.0, (edge[0][1] + edge[1][1]) / 2.0);
            let len = (out[0] * out[0] + out[1] * out[1]).sqrt();
            let label = kit.text(
                stage,
                word,
                36.0,
                ink(),
                v(
                    mid[0] + (reach + 0.3) * out[0] / len,
                    mid[1] + (reach + 0.3) * out[1] / len,
                ),
            )?;
            braces.extend([b, label]);
        }
        let braces_in: Moves = braces
            .iter()
            .map(|&m| arrive(stage, m, 0.4))
            .collect::<fmn::Result<Moves>>()?;
        line.wait_for(stage, "Four wide")?;
        play(stage, braces_in, 0.25)?;
        // the staircase on top, ten of the twenty places
        line.wait_for(stage, "with a staircase")?;
        let step_cells: Vec<(i64, i64)> = stacks::top(n);
        let step = marker(&iso, stage, &step_cells, (n + 1) as f64, (0.0, 0.0), hero())?;
        let step_words = kit.text(
            stage,
            &format!("{STEP_WORDS}: {} {CUBES_WORD}", r.top),
            30.0,
            ink(),
            v(4.2, 0.9),
        )?;
        let face_words = kit.text(
            stage,
            &format!("{TOP_WORDS}: {} {SQUARES_WORD}", r.top_layer),
            30.0,
            dim(),
            v(4.2, 0.3),
        )?;
        let mut shown_now: Moves = Vec::new();
        for &m in &[step, step_words, face_words] {
            shown_now.push(arrive(stage, m, 0.5)?);
        }
        play(stage, shown_now, 0.0)?;
        line.wait_end(stage)?;
        pause(stage, 0.8)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// A short stillness while a pulse is read.
fn pause_short(stage: &mut Stage<'_>) -> fmn::Result<()> {
    pause(stage, 0.5)
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::super::siu::max_corner;
    use super::*;

    /// P5 verify (07_1): the four squares come up as "Now add up the
    /// squares" is said, before the first count, so the frame is not empty
    /// for the first four seconds of the line (it showed the shelf alone
    /// until "One,").
    #[test]
    fn the_squares_come_up_with_the_words_that_name_them() {
        let voiced = testing::run_voiced("07_three_stacks").unwrap();
        let events = super::super::draw::quiz_log::take(&voiced.tag, "07_1");
        let at = |k: &str| {
            events
                .iter()
                .find(|e| e.0 == k)
                .unwrap_or_else(|| panic!("no {k}: {events:?}"))
                .1
        };
        let (_, start, _) = voiced.lines.iter().find(|l| l.0 == "07_1").cloned().unwrap();
        let (slabs, first) = (at("slabs_in"), at("first_count"));
        assert!(
            slabs - start <= 3.5,
            "the squares come up {:.1} s into the line",
            slabs - start
        );
        assert!(
            first - slabs >= 0.8,
            "{:.1} s between the squares and the first count",
            first - slabs
        );
    }

    /// P5 verify (07_3): the picture keeps up with the voice: the next line
    /// waits no more than two and a half seconds on the turns (it waited 3.7 s), and
    /// each quarter turn still takes the half second a viewer needs to see
    /// it.
    #[test]
    fn the_turns_keep_up_with_the_voice() {
        let v = testing::run_voiced("07_three_stacks").unwrap();
        let gap = super::super::sound::gap_between(&v, "07_3", "07_4");
        eprintln!("07_3 to 07_4: {gap:.2} s");
        assert!(gap <= 2.5, "{gap:.1} s of silence between 07_3 and 07_4");
        const {
            assert!(TURN_S >= 0.5 && SLIDE_S >= 0.6);
        }
    }

    /// The chapter speaks exactly the script's 07_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("07_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The facts the chapter shows are the model's, and the quiz's answer is
    /// three.
    #[test]
    fn the_facts_are_the_models() {
        let r = reads().unwrap();
        assert_eq!(r.layers, [16, 9, 4, 1]);
        assert_eq!((r.stack, r.top, r.top_layer), (30, 10, 20));
        assert_eq!(r.block, [4, 5, 4]);
        assert_eq!(r.answer, "3");
        // the block's box: the slabs are full boards of 4 by 5
        let all: Vec<_> = (0..3).flat_map(|w| placed(4, w)).collect();
        assert_eq!(min_corner(&all), [0.0, 0.0, 0.0]);
        assert_eq!(max_corner(&all), [4.0, 5.0, 5.0]);
        for z in 0..4 {
            assert_eq!(all.iter().filter(|c| c.2 == z).count() as i64, r.top_layer);
        }
    }

    /// Everything stands on the screen: the slabs' places, the waiting
    /// copies, the block and the five slabs of the row, in the frame and
    /// (the part left of the shelf's last slot) under the shelf's floor.
    #[test]
    fn the_pictures_stand_on_the_screen() {
        let view = VIEW;
        let check = |lo: P3, hi: P3, what: &str| {
            for m in 0..8usize {
                let p = [
                    if m & 1 == 1 { hi[0] } else { lo[0] },
                    if m & 2 == 2 { hi[1] } else { lo[1] },
                    if m & 4 == 4 { hi[2] } else { lo[2] },
                ];
                let s = view.project(p);
                assert!(
                    s[0] > -FRAME_X_RADIUS + 0.2 && s[0] < FRAME_X_RADIUS - 0.2,
                    "{what}: x {} out of the frame",
                    s[0]
                );
                assert!(
                    s[1] > -FRAME_Y_RADIUS + 0.15 && s[1] < super::super::draw::SHELF_FLOOR,
                    "{what}: y {} out of the frame",
                    s[1]
                );
            }
        };
        for (i, k) in [4.0, 3.0, 2.0, 1.0].iter().enumerate() {
            let at = SLAB_AT[i];
            check(at, [at[0] + k, at[1] + k, 1.0], "a slab on the board");
            // hovering above the stack, at its centre
            check([0.0, 0.0, 0.0], [4.0, 4.0, 4.0 + HOVER], "a slab hovering");
        }
        for w in WAIT {
            check(w, [w[0] + 4.0, w[1] + 4.0, w[2] + 4.0], "a waiting copy");
        }
        check([0.0; 3], [4.0, 5.0, 5.0], "the block");
        for (rx, ry) in ROW {
            check([rx, ry, 0.0], [rx + 4.0, ry + 5.0, 1.0], "a slab of the row");
        }
    }

    /// The slabs of the row do not overlap, and the row is where the model's
    /// levels are: each level's cells moved by the row's offset are 20 cells
    /// (the step 10) standing on the ground.
    #[test]
    fn the_row_is_five_separate_boards() {
        for (i, &a) in ROW.iter().enumerate() {
            for (j, &b) in ROW.iter().enumerate().skip(i + 1) {
                let apart = a.0 + 4.0 <= b.0 || b.0 + 4.0 <= a.0 || a.1 + 5.0 <= b.1 || b.1 + 5.0 <= a.1;
                assert!(apart, "slabs {i} and {j} overlap: {a:?} {b:?}");
            }
        }
        let n = 4;
        for z in 0..=n {
            let count: usize = (0..3).map(|w| level(n, w, z).len()).sum();
            assert_eq!(count as i64, if z == n { 10 } else { 20 }, "level {z}");
        }
    }

    /// The L-shaped layer a blue square gives up to the slab above is the
    /// difference of the squares, and the slab above has no blue in it.
    #[test]
    fn the_layer_the_blue_square_gives_up_is_the_difference_of_squares() {
        let n = 4;
        for z in 1..=n {
            let big = level(n, 0, z - 1).len() as i64;
            let small = level(n, 0, z).len() as i64;
            let l = layers::layer(n - z + 1).len() as i64;
            assert_eq!(big - small, l, "slab {z}");
            // in the slab above, the cells of that L belong to the other two
            let (side, above) = (n - z + 1, level(n, 0, z));
            for (x, y) in layers::layer(side) {
                assert!(!above.iter().any(|c| (c.0, c.1) == (x, y)), "({x}, {y}) is blue above");
            }
        }
    }

    /// The scene constructs end to end: the second and third stacks end on
    /// the model's cells (the scene fails otherwise), every phrase is in its
    /// line, the layout lint finds nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("07_three_stacks").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 40.0 && ran.seconds < 90.0, "{} s", ran.seconds);
    }

    /// The drop order is a real one: the stacks can come in from the side,
    /// first then second then third (the facts: stack 0 stands, stack 1 slides
    /// in along x from the right, stack 2 too).
    #[test]
    fn the_second_and_third_slide_in_from_the_right() {
        use std::collections::HashSet;
        let n = 4;
        let a: HashSet<_> = placed(n, 0).into_iter().collect();
        let b: HashSet<_> = placed(n, 1).into_iter().collect();
        let c = placed(n, 2);
        let free = |cells: &[(i64, i64, i64)], others: &HashSet<(i64, i64, i64)>| {
            cells
                .iter()
                .all(|&(x, y, z)| (1..32).all(|t| !others.contains(&(x + t, y, z))))
        };
        assert!(free(&placed(n, 1), &a), "the second slides in along x");
        let ab: HashSet<_> = a.union(&b).copied().collect();
        assert!(free(&c, &ab), "the third slides in along x");
        // and the waiting places are clear of the block and of one another
        for w in WAIT {
            assert!(w[0] == 0.0 || w[0] >= 4.0);
        }
    }
}
