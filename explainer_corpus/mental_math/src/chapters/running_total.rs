//! Chapter 05, the running total: four numbers are too many to hold, so
//! hold one (05_1); the pour: the pieces of twenty-three by forty-six land
//! one at a time in a pile ruled in hundreds while one total steps up
//! (05_2); between steps one number is held, and each total is a floor
//! (05_3); a quiz on six times forty-three (05_4), worked on its two pieces
//! (05_5); and the product left waiting at 03_7, thirteen times fourteen
//! (05_6), worked on its four pieces and written into the waiting card:
//! one hundred thirty-five squared (05_7).
//!
//! The chapter starts where chapter 04 ends (`by_parts.rs`): the cut
//! rectangle on its grid, the four numbers in the head, the waiting card.

use std::f64::consts::{FRAC_PI_2, PI};

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::animation::{Transform, indicate};
use fmn::prelude::*;

use super::draw::{
    Board, CutRect, NOTE_PT, NumberLine1, QuizCard, QuizLook, SIDE_PT, Side, corner, eq, fade_in_all, fade_out_all,
    faded_line, fail, flat, fly, glyphs, hundreds, in_your_head, middle_of, num, panel, paper_ink, pause, rgb, say,
    swap, tag, waiting_card,
};
use crate::facts::shown;
use crate::model::{ByParts, by_parts, ends_in_five, parts};

pub struct RunningTotal {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const RUNNING_TOTAL: &str = "running total";
const BETWEEN_STEPS: &str = "between steps:";
const AFTER_BLOCK: &str = "after the block: between";
const AND: &str = "and";
const FOR_PROGRAMMERS: &str = "for programmers:";
/// The music alone after 05_3, to read its stretch and its note.
const NOTE_READ_S: f64 = 1.5;
const ANYTIME: &str = "an anytime answer: stop after any step and you have a floor that only rises. The price: a later piece can change a digit you already said";
const PIECES_ARE: &str = "pieces = digits × digits:";
const HERE: &str = "here,";
const BEFORE: &str = "before";

/// What chapter 04 leaves (the same numbers as `by_parts.rs`): its board,
/// the grid behind it, the head in the column and the four numbers in it.
const UNIT_04: f64 = 0.14;
const BOARD_04: Vec3 = [-6.1, -1.9, 0.0];
const GRID: (f64, f64) = (50.0, 30.0);
const HEAD_AT: Vec3 = [3.9, -0.35, 0.0];
const HEAD_SIZE: (f64, f64) = (5.6, 4.0);
const HELD_AT: [Vec3; 4] = [
    [2.55, 0.75, 0.0],
    [5.15, 0.55, 0.0],
    [2.85, -1.35, 0.0],
    [5.25, -1.15, 0.0],
];
const HELD_PT: f64 = 84.0;
const WAVER: f64 = 0.06;
/// The running total held for the whole pour: one number.
const HELD_TOTAL: usize = 1;

/// The pour (05_1 to 05_2): the rectangle smaller at the left; the head
/// across the rest, with the pile at its left and the total at its right.
const UNIT: f64 = 0.1;
const BOARD_AT: Vec3 = [-6.25, -0.45, 0.0];
const POUR_HEAD_AT: Vec3 = [2.9, -0.3, 0.0];
const POUR_HEAD_SIZE: (f64, f64) = (7.6, 4.4);
const PILE_AT: Vec3 = [-0.45, -2.15, 0.0];
/// The pile's ruled frame, in units: four hundreds wide, three high.
const PILE: [f64; 4] = [0.0, 0.0, 40.0, 30.0];
const TOTAL_RIGHT: Vec3 = [6.35, 0.15, 0.0];
const TOTAL_PT: f64 = 110.0;
const TAG_PT: f64 = 36.0;
/// 05_3: the head shrinks round the total; the number line beneath.
const SMALL_HEAD_AT: Vec3 = [3.9, 1.05, 0.0];
const SMALL_HEAD_SIZE: (f64, f64) = (4.6, 1.9);
const STEPS_TAG_AT: Vec3 = [-2.6, 1.05, 0.0];
const LINE_Y: f64 = -1.25;
const LINE_X: (f64, f64) = (-6.1, 6.1);
/// The number line's labelled ticks are this far apart.
const LINE_STEP: u64 = 400;
/// The quizzes' cards, at the right, with the working at the left.
const QUIZ_RIGHT: Vec3 = [3.9, -0.3, 0.0];
const QUIZ_SCALE: f64 = 0.7;
const WORK_PT: f64 = 96.0;
/// 05_5: six by forty-three.
const SIX_UNIT: f64 = 0.15;
const SIX_AT: Vec3 = [-6.3, 0.1, 0.0];
const SIX_TOTAL_RIGHT: Vec3 = [-1.4, -1.45, 0.0];
/// 05_6 to 05_7: the waiting card beside the quiz card, and thirteen by
/// fourteen under it.
const WAITING_BESIDE: Vec3 = [-2.4, 1.35, 0.0];
const WAITING_SCALE: f64 = 1.3;
/// The waiting card's type size (`draw::waiting_card`).
const WAITING_PT: f64 = 38.0;
const HARD_UNIT: f64 = 0.2;
const HARD_AT: Vec3 = [-6.0, -2.55, 0.0];
const HARD_TOTAL_RIGHT: Vec3 = [0.2, -1.2, 0.0];

/// Text or mathematics, typeset on its own at the origin. Detached.
fn typeset(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, tex: bool) -> fmn::Result<Mob> {
    if tex {
        kit.tex(stage, src, size, color, &[], ORIGIN)
    } else {
        kit.text(stage, src, size, color, ORIGIN)
    }
}

/// `whole` (already in place, never shown) cut into `parts` that move on
/// their own, each laid over its own glyphs in the whole (as in
/// `by_parts.rs`). Detached.
fn cut_into(
    kit: &Kit,
    stage: &mut Stage<'_>,
    whole: Mob,
    parts: &[&str],
    size: f64,
    color: Srgb,
) -> fmn::Result<Vec<Mob>> {
    let all = glyphs(stage, whole);
    let mut out = Vec::new();
    let mut k = 0;
    for &part in parts {
        let m = typeset(kit, stage, part, size, color, true)?;
        let n = glyphs(stage, m).len();
        if n == 0 || k + n > all.len() {
            return Err(fail(format!("{part:?} is not where it should be in {parts:?}")));
        }
        let at = middle_of(stage, &all[k..k + n])?;
        stage.move_to(m, at, ORIGIN);
        out.push(m);
        k += n;
    }
    if k != all.len() {
        return Err(fail(format!("{parts:?} leave glyphs over")));
    }
    Ok(out)
}

/// Move `mob`'s centre to `to`, scaling it by `scale` and recolouring it on
/// the way, `rt` seconds.
fn fly_paint(mob: Mob, to: Vec3, scale: f64, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(scale)?
        .move_to(to, ORIGIN)?
        .set_color(color)?)
}

/// The numbers held in the head waver as much as the tracker `amount` says
/// (as in `by_parts.rs`). The returned anchor carries the updater.
fn waver(stage: &mut Stage<'_>, mobs: &[Mob], amount: Mob) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let homes: Vec<Vec3> = mobs.iter().map(|&m| stage.get_center(m)).collect();
    let mobs = mobs.to_vec();
    let mut clock = 0.0;
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            clock += dt;
            let k = s.tracker_value(amount).unwrap_or(0.0);
            for (i, (&m, home)) in mobs.iter().zip(&homes).enumerate() {
                let phase = 1.9 * i as f64;
                let dx = WAVER * k * (2.1 * clock + phase).sin();
                let dy = 0.7 * WAVER * k * (2.9 * clock + 2.0 * phase).sin();
                s.move_to(m, v(home[0] + dx, home[1] + dy), ORIGIN);
                let flicker = 0.5 + 0.5 * (1.7 * clock + phase).sin();
                s.set_opacity(m, 1.0 - 0.35 * k * flicker, true);
            }
        },
        false,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

/// A piece carried rigidly from where it lies to where it lands: `w` by `h`
/// (scene units) centred at `from`, turned by `turn` on the way (a quarter
/// turn swaps its sides), lifted `lift` at the middle of the flight. It is
/// the same rectangle all the way, so its area never changes.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Carry {
    from: Vec3,
    to: Vec3,
    w: f64,
    h: f64,
    turn: f64,
    lift: f64,
}

impl Carry {
    fn quad(&self, t: f64) -> [Vec3; 4] {
        let t = t.clamp(0.0, 1.0);
        let u = ((t - 0.15) / 0.7).clamp(0.0, 1.0);
        let angle = self.turn * u * u * (3.0 - 2.0 * u);
        let c = [
            self.from[0] + (self.to[0] - self.from[0]) * t,
            self.from[1] + (self.to[1] - self.from[1]) * t + self.lift * (PI * t).sin(),
        ];
        let (cos, sin) = (angle.cos(), angle.sin());
        let (hw, hh) = (self.w / 2.0, self.h / 2.0);
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, b): (f64, f64)| {
            let (x, y) = (a * hw, b * hh);
            v(c[0] + x * cos - y * sin, c[1] + x * sin + y * cos)
        })
    }
}

/// Carry `piece` (a polygon of four corners) by `carry` as its tracker runs
/// from 0 to 1. Returns the tracker and the anchor that carries the updater
/// (in the scene).
fn carry(stage: &mut Stage<'_>, piece: Mob, carry: Carry) -> fmn::Result<(Mob, Mob)> {
    let t = stage.add_value_tracker(0.0);
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = f64::NAN;
    stage.add_updater(
        anchor,
        move |s, _| {
            let now = s.tracker_value(t).unwrap_or(0.0);
            if now == last {
                return;
            }
            let shape = Polygon::new(carry.quad(now)).build();
            let _ = s.set_points(piece, shape.points());
            last = now;
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok((t, anchor))
}

/// The running total, stepped from one fixed number to the next (a number
/// being worked is never printed with a separator): every value typeset at
/// once, right-aligned at `right`, with its tag beneath.
struct Total {
    values: Vec<Mob>,
    tag: Mob,
    shown: Option<usize>,
}

impl Total {
    fn build(kit: &Kit, stage: &mut Stage<'_>, totals: &[u64], right: Vec3, size: f64) -> fmn::Result<Self> {
        let mut values = Vec::new();
        for &t in totals {
            let m = eq(kit, stage, &num(t), size, right)?;
            stage.move_to(m, right, RIGHT);
            values.push(m);
        }
        let widest = values.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
        let tag_at = v(right[0] - widest / 2.0, right[1] - size / 110.0 - 0.15);
        let tag = tag(kit, stage, RUNNING_TOTAL, tag_at, TAG_PT)?;
        Ok(Self {
            values,
            tag,
            shown: None,
        })
    }

    /// Show value `k` in place of the one shown, `rt` seconds.
    fn step(&mut self, stage: &mut Stage<'_>, k: usize, rt: f64) -> fmn::Result<()> {
        let new = *self
            .values
            .get(k)
            .ok_or_else(|| fail(format!("the total has no step {k}")))?;
        let old: Vec<Mob> = self.shown.map(|i| self.values[i]).into_iter().collect();
        swap(stage, &old, &[new], rt)?;
        self.shown = Some(k);
        Ok(())
    }

    fn current(&self) -> fmn::Result<Mob> {
        self.shown
            .map(|i| self.values[i])
            .ok_or_else(|| fail("the total shows nothing yet".into()))
    }

    /// What is on stage, for a fade.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.tag];
        out.extend(self.shown.map(|i| self.values[i]));
        out
    }
}

/// A small cut rectangle for a quiz's working: its pieces in their roles'
/// colours, the cuts, and the side labels (tall at the left, wide below,
/// one label per part). Detached.
struct Working {
    cut: CutRect,
    labels: Vec<Mob>,
}

impl Working {
    fn build(kit: &Kit, stage: &mut Stage<'_>, board: Board, a: u64, b: u64) -> fmn::Result<Self> {
        let cut = CutRect::build(stage, board, a, b, Palette::ink());
        for (i, &m) in cut.pieces.iter().enumerate() {
            stage.set_color(m, cut.role(i), None, true);
        }
        let p = by_parts(a, b);
        let mut labels = Vec::new();
        let mut seen_h = Vec::new();
        let mut seen_w = Vec::new();
        for (piece, &r) in p.pieces.iter().zip(&cut.rects) {
            if !seen_h.contains(&piece.height) {
                seen_h.push(piece.height);
                let ink = Palette::ink();
                labels.push(board.label(kit, stage, &num(piece.height), r, Side::Left, SIDE_PT, ink)?);
            }
            if !seen_w.contains(&piece.width) {
                seen_w.push(piece.width);
                let ink = Palette::ink();
                labels.push(board.label(kit, stage, &num(piece.width), r, Side::Below, SIDE_PT, ink)?);
            }
        }
        Ok(Self { cut, labels })
    }

    fn mobs(&self) -> Vec<Mob> {
        let mut out = self.cut.pieces.clone();
        out.extend(&self.cut.cuts);
        out.extend(&self.labels);
        out
    }

    /// Piece `i`'s number, on the piece, or above it when the piece is too
    /// small to hold it. Detached.
    fn number(&self, kit: &Kit, stage: &mut Stage<'_>, i: usize, n: u64) -> fmn::Result<Mob> {
        let r = self.cut.rects[i];
        let at = self.cut.centre(i);
        let m = eq(kit, stage, &num(n), NOTE_PT, at)?;
        let unit = self.cut.board.unit;
        let fits = stage.get_width(m) < r[2] * unit - 0.08 && stage.get_height(m) < r[3] * unit - 0.08;
        if fits {
            stage.set_color(m, paper_ink(), None, true);
        } else {
            let top = self.cut.board.p(r[0] + r[2] / 2.0, r[1] + r[3]);
            stage.move_to(m, v(top[0], top[1] + 0.12), DOWN);
        }
        Ok(m)
    }
}

/// Work a product by parts beside its quiz card: the small cut rectangle
/// appears, then piece by piece (piece `i` on sentence `i`) its number lands
/// on it, and the total steps when the line says it (`totals[i]`). Returns
/// the working, the numbers on the pieces and the total.
fn work(
    kit: &Kit,
    stage: &mut Stage<'_>,
    line: &super::draw::Spoken,
    p: &ByParts,
    board: Board,
    right: Vec3,
    totals: &[&str],
) -> fmn::Result<(Working, Vec<Mob>, Total)> {
    if totals.len() != p.pieces.len() {
        return Err(fail(format!(
            "{} totals said for {} pieces",
            totals.len(),
            p.pieces.len()
        )));
    }
    let w = Working::build(kit, stage, board, p.a, p.b)?;
    let mut total = Total::build(kit, stage, &p.running, right, WORK_PT)?;
    let mut ins = w.mobs();
    ins.push(total.tag);
    fade_in_all(stage, &ins, 0.5)?;
    let mut numbers = Vec::new();
    for (i, piece) in p.pieces.iter().enumerate() {
        line.wait_sentence(stage, i)?;
        let n = w.number(kit, stage, i, piece.area())?;
        let mut anims = Vec::new();
        let lit = indicate(stage.arena_mut(), w.cut.pieces[i], 1.0, Some(rgb(Palette::ink())))?;
        anims.push(stage.prepare(lit.rt(0.6))?);
        let a = fade_in(stage.arena_mut(), n, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, totals[i])?;
        total.step(stage, i, 0.35)?;
        numbers.push(n);
    }
    Ok((w, numbers, total))
}

/// The last total flies into the answer already on the card, which lights.
fn into_the_card(kit: &Kit, stage: &mut Stage<'_>, total: &Total, card: &QuizCard) -> fmn::Result<()> {
    let from = stage.get_center(total.current()?);
    let copy = eq(kit, stage, &card.quiz.worked().map_err(fail)?.answer, WORK_PT, from)?;
    stage.add_to_scene(copy)?;
    let to = stage.get_center(card.answer);
    play!(stage; fly(copy, to, card.look.pt() / WORK_PT, 0.8)?);
    stage.remove_from_scene(copy);
    play!(stage; indicate(stage.arena_mut(), card.answer, 1.15, Some(rgb(Palette::ink())))?.rt(0.6));
    Ok(())
}

impl SceneConstruct for RunningTotal {
    fn name(&self) -> &str {
        "05_running_total"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let (a, b) = shown::PARTS;
        let p = by_parts(a, b);
        if p.pieces.len() != 4 {
            return Err(fail(format!("{a} × {b} is drawn as four pieces: {p:?}")));
        }

        // where chapter 04 left off: the cut rectangle on its grid, the four
        // numbers in the head, the card waiting in its corner
        let waiting = waiting_card(&kit, stage, shown::FIVE_HARD)?;
        let board04 = Board::new(BOARD_04, UNIT_04);
        let grid = board04.tens(stage, GRID.0, GRID.1)?;
        let cut = CutRect::build(stage, board04, a, b, ink);
        for (i, &m) in cut.pieces.iter().enumerate() {
            stage.set_color(m, cut.role(i), None, true);
        }
        let mut labels = Vec::new();
        let mut label_sides = Vec::new();
        {
            let (a_tens, a_ones) = parts(a);
            let (b_tens, b_ones) = parts(b);
            let rect_of = |h: u64, w: u64| -> fmn::Result<[f64; 4]> {
                p.pieces
                    .iter()
                    .position(|q| q.height == h && q.width == w)
                    .map(|i| cut.rects[i])
                    .ok_or_else(|| fail(format!("{a} × {b} has no piece {h} by {w}")))
            };
            for (x, r, side) in [
                (a_tens, rect_of(a_tens, b_tens)?, Side::Left),
                (a_ones, rect_of(a_ones, b_tens)?, Side::Left),
                (b_tens, rect_of(a_tens, b_tens)?, Side::Below),
                (b_ones, rect_of(a_tens, b_ones)?, Side::Below),
            ] {
                labels.push(board04.label(&kit, stage, &num(x), r, side, SIDE_PT, ink)?);
                label_sides.push((x, r, side));
            }
        }
        let head = in_your_head(&kit, stage, HEAD_AT, HEAD_SIZE.0, HEAD_SIZE.1)?;
        let mut held = Vec::new();
        for (piece, &at) in p.pieces.iter().zip(&HELD_AT) {
            held.push(eq(&kit, stage, &num(piece.area()), HELD_PT, at)?);
        }
        let mut first = vec![grid];
        first.extend(&cut.pieces);
        first.extend(&cut.cuts);
        first.extend(&labels);
        first.extend(head);
        first.extend(&held);
        first.extend(waiting);
        for &m in &first {
            stage.add_to_scene(m)?;
        }

        // 05_1: four numbers is a lot to hold, so don't hold them: the head
        // empties, and one running total takes their place beside the
        // rectangle
        let line = say(&kit, stage, "05_1")?;
        let amount = stage.add_value_tracker(0.0);
        let shaking = waver(stage, &held, amount)?;
        stage.play(tween(amount, 1.0, 1.0)?)?;
        line.wait_sentence(stage, 1)?;
        stage.clear_updaters(shaking, true);
        let mut anims = Vec::new();
        for &m in &held {
            let a = fade_out(stage.arena_mut(), m, v(0.0, -0.4), 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 2)?;
        // the rectangle shrinks to the left and the head widens to take the
        // pile and the total
        let board = Board::new(BOARD_AT, UNIT);
        let k = UNIT / UNIT_04;
        let to_new = |p: Vec3| {
            v(
                BOARD_AT[0] + (p[0] - BOARD_04[0]) * k,
                BOARD_AT[1] + (p[1] - BOARD_04[1]) * k,
            )
        };
        let mut anims = Vec::new();
        for &m in cut.pieces.iter().chain(&cut.cuts) {
            let c = to_new(stage.get_center(m));
            anims.push(stage.prepare(fly(m, c, k, 1.0)?)?);
        }
        for (&m, &(x, r, side)) in labels.iter().zip(&label_sides) {
            let goal = board.label(&kit, stage, &num(x), r, side, SIDE_PT, ink)?;
            let to = stage.get_center(goal);
            anims.push(stage.prepare(fly(m, to, 1.0, 1.0)?)?);
        }
        let grid_out = fade_out(stage.arena_mut(), grid, ORIGIN, 1.0)?;
        anims.push(stage.prepare(grid_out.rt(0.6))?);
        let wide_head = in_your_head(&kit, stage, POUR_HEAD_AT, POUR_HEAD_SIZE.0, POUR_HEAD_SIZE.1)?;
        anims.push(stage.prepare(Transform::new(head[0], wide_head[0]).rt(1.0))?);
        let words_to = stage.get_center(wide_head[1]);
        anims.push(stage.prepare(fly(head[1], words_to, 1.0, 1.0)?)?);
        stage.play_prepared(anims)?;
        // one number, large, tagged; the pile it stands for beside it
        let mut steps = vec![p.running[0] - p.pieces[0].area()];
        steps.extend(&p.running);
        let mut total = Total::build(&kit, stage, &steps, TOTAL_RIGHT, TOTAL_PT)?;
        line.wait_for(stage, "running total")?;
        total.step(stage, 0, 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), total.tag, v(0.0, 0.1), 1.0)?.rt(0.4));
        line.wait_for(stage, "add each piece")?;
        let pile = Board::new(PILE_AT, UNIT);
        let ruled = pile.grid(stage, PILE, 10.0, faded_line(ink, 2.0, 0.45))?;
        let ghosts: Vec<Mob> = cut
            .rects
            .iter()
            .map(|&r| board.rect(stage, r, faded_line(Palette::faint(), 2.0, 1.0)))
            .collect();
        let mut ins = ghosts.clone();
        ins.push(ruled);
        fade_in_all(stage, &ins, 0.6)?;
        stage.scene_mut().bring_to_front(&cut.pieces)?;

        // 05_2: the pour: block, strip, strip, corner, each lifts off the
        // rectangle and lands in the pile as the total steps
        let line = say(&kit, stage, "05_2")?;
        let mut lit = Vec::new();
        for &m in &cut.pieces {
            let a = indicate(stage.arena_mut(), m, 1.0, Some(rgb(ink)))?;
            lit.push(stage.prepare(a)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), lit, 0.5)?.with_run_time(1.8);
        stage.play(g)?;
        // where each piece lands in the pile (units): the block as it is;
        // the strip at the side cut down its middle, each half turned a
        // quarter and laid along the top; the strip along the top as it is,
        // above them; the corner on top at the left
        let block_h = p.pieces[0].height as f64;
        let side = p.pieces[1];
        let top = p.pieces[2];
        let (sh, sw) = (side.height as f64, side.width as f64);
        // the two halves lie end to end as wide as the block, as does the
        // strip along the top (forty-six is twice twenty-three), and the
        // whole pile stays inside its ruled frame
        let pile_h = block_h + sw / 2.0 + (top.height + p.pieces[3].height) as f64;
        if 2 * side.height != p.pieces[0].width || top.width != p.pieces[0].width || pile_h > PILE[3] {
            return Err(fail(format!("the pile is laid out for {a} × {b}: {p:?}")));
        }
        let half_w = sw / 2.0;
        // a half turned a quarter is as tall as it was wide
        let band = half_w;
        let landing = [
            vec![[0.0, 0.0, p.pieces[0].width as f64, block_h]],
            vec![[0.0, block_h, sh, band], [sh, block_h, sh, band]],
            vec![[0.0, block_h + band, top.width as f64, top.height as f64]],
            vec![[
                0.0,
                block_h + band + top.height as f64,
                p.pieces[3].width as f64,
                p.pieces[3].height as f64,
            ]],
        ];
        // every piece keeps its area where it lands
        for (piece, lands) in p.pieces.iter().zip(&landing) {
            let area: f64 = lands.iter().map(|r| r[2] * r[3]).sum();
            if (area - piece.area() as f64).abs() > 1e-9 {
                return Err(fail(format!("{piece:?} lands as {area} units")));
            }
        }
        let says = [
            "eight hundred",
            "nine hundred twenty",
            "one thousand forty",
            "one thousand fifty-eight",
        ];
        let mut in_pile = Vec::new();
        for (i, lands) in landing.iter().enumerate() {
            line.wait_sentence(stage, i + 1)?;
            let r = cut.rects[i];
            // the moving pieces, and how each is carried
            let mut moving = Vec::new();
            if lands.len() == 1 {
                let c = board.centre(r);
                let to = pile.centre(lands[0]);
                let carry = Carry {
                    from: c,
                    to,
                    w: r[2] * UNIT,
                    h: r[3] * UNIT,
                    turn: 0.0,
                    lift: 0.6,
                };
                moving.push((cut.pieces[i], carry));
            } else {
                // cut down the middle: two halves in place of the strip
                let halves = [[r[0], r[1], half_w, r[3]], [r[0] + half_w, r[1], half_w, r[3]]];
                let seam = board.line(
                    stage,
                    [r[0] + half_w, r[1]],
                    [r[0] + half_w, r[1] + r[3]],
                    super::draw::cut_style(),
                );
                let mut made = Vec::new();
                for (h, &land) in halves.iter().zip(lands) {
                    let m = board.rect(stage, *h, flat(cut.role(i), 1.0));
                    stage.add_to_scene(m)?;
                    made.push(m);
                    let carry = Carry {
                        from: board.centre(*h),
                        to: pile.centre(land),
                        w: h[2] * UNIT,
                        h: h[3] * UNIT,
                        turn: -FRAC_PI_2,
                        lift: 0.8,
                    };
                    moving.push((m, carry));
                }
                stage.remove_from_scene(cut.pieces[i]);
                stage.add_to_scene(seam)?;
                pause(stage, 0.2)?;
                stage.remove_from_scene(seam);
            }
            if i == 0 {
                fade_out_all(stage, &cut.cuts, 0.3)?;
            }
            let mut trackers = Vec::new();
            let mut anchors = Vec::new();
            for &(m, c) in &moving {
                stage.scene_mut().bring_to_front(&[m])?;
                let (t, anchor) = carry(stage, m, c)?;
                trackers.push((t, 1.0));
                anchors.push(anchor);
                in_pile.push(m);
            }
            super::draw::tween_all(stage, &trackers, 1.1)?;
            for anchor in anchors {
                stage.clear_updaters(anchor, true);
                stage.remove_from_scene(anchor);
            }
            stage.scene_mut().bring_to_front(&[ruled])?;
            line.wait_for(stage, says[i])?;
            total.step(stage, i + 1, 0.35)?;
        }

        // 05_3: one number held between steps; each total a floor on the
        // number line, and the stretch the answer must lie in after the block
        let line = say(&kit, stage, "05_3")?;
        let mut gone = ghosts.clone();
        gone.extend(&labels);
        gone.extend(&in_pile);
        gone.push(ruled);
        let small_head = in_your_head(&kit, stage, SMALL_HEAD_AT, SMALL_HEAD_SIZE.0, SMALL_HEAD_SIZE.1)?;
        let mut anims = Vec::new();
        for &m in &gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        anims.push(stage.prepare(Transform::new(head[0], small_head[0]).rt(0.9))?);
        let words_to = stage.get_center(small_head[1]);
        anims.push(stage.prepare(fly(head[1], words_to, 1.0, 0.9)?)?);
        let held_now = total.current()?;
        anims.push(stage.prepare(fly(held_now, SMALL_HEAD_AT, 1.0, 0.9)?)?);
        let tag_out = fade_out(stage.arena_mut(), total.tag, ORIGIN, 1.0)?;
        anims.push(stage.prepare(tag_out.rt(0.4))?);
        stage.play_prepared(anims)?;
        let four = tag(
            &kit,
            stage,
            &format!("{BETWEEN_STEPS} {}", p.pieces.len()),
            STEPS_TAG_AT,
            40.0,
        )?;
        let one = tag(
            &kit,
            stage,
            &format!("{BETWEEN_STEPS} {HELD_TOTAL}"),
            STEPS_TAG_AT,
            40.0,
        )?;
        play!(stage; fade_in(stage.arena_mut(), four, ORIGIN, 1.0)?.rt(0.4));
        // the number line, 0 up past the most the answer could be
        let (low, high) = p.bounds();
        let top = (high / LINE_STEP + 1) * LINE_STEP;
        let nl = NumberLine1 {
            x0: LINE_X.0,
            x1: LINE_X.1,
            y: LINE_Y,
            lo: 0.0,
            hi: top as f64,
        };
        let mut axis = vec![nl.axis(stage)];
        for k in 0..=top / LINE_STEP {
            let value = (k * LINE_STEP) as f64;
            axis.push(nl.tick(stage, value, 0.22, Palette::dim()));
            axis.push(nl.label(&kit, stage, &num(k * LINE_STEP), value, 30.0, Palette::dim(), false)?);
        }
        let marker = nl.dot(stage, 0.0, 0.09, ink);
        axis.push(marker);
        fade_in_all(stage, &axis, 0.5)?;
        axis.pop();
        // the totals step up the line, one number at a time; each leaves a
        // mark, a floor
        let mut at_label: Option<Mob> = None;
        let mut marks = Vec::new();
        let mut was = 0.0;
        for &t in &p.running {
            let value = t as f64;
            let bow = nl.bow(
                stage,
                was,
                value,
                0.35 + 0.0004 * (value - was),
                faded_line(ink, 2.0, 0.6),
            );
            let label = nl.label(&kit, stage, &num(t), value, 34.0, ink, true)?;
            stage.move_to(label, v(nl.p(value)[0], LINE_Y + 0.5), DOWN);
            let mark = nl.tick(stage, value, 0.3, ink);
            let mut anims = Vec::new();
            let a = fade_in(stage.arena_mut(), bow, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.35))?);
            anims.push(stage.prepare(fly(marker, nl.p(value), 1.0, 0.35)?)?);
            let a = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.35))?);
            if let Some(old) = at_label {
                let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.25))?);
            }
            stage.play_prepared(anims)?;
            stage.add_to_scene(mark)?;
            marks.push(mark);
            marks.push(bow);
            at_label = Some(label);
            was = value;
        }
        line.wait_for(stage, "one number")?;
        swap(stage, &[four], &[one], 0.5)?;
        // after the block the answer cannot leave a stretch; the totals
        // only climb inside it
        line.wait_sentence(stage, 1)?;
        let stretch = nl.stretch(stage, low as f64, high as f64, 0.34, flat(hundreds(), 0.35));
        let (at, bt) = (parts(a).0, parts(b).0);
        if at * bt != low || (at + 10) * (bt + 10) != high {
            return Err(fail(format!(
                "the bounds {low}, {high} are not the tens of {a} and {b}"
            )));
        }
        let between = format!("{AFTER_BLOCK} {at} × {bt} {AND} {} × {}", at + 10, bt + 10);
        let mid = (low + high) as f64 / 2.0;
        let stretch_label = kit.text(stage, &between, 30.0, hundreds(), v(nl.p(mid)[0], LINE_Y - 0.95))?;
        play!(stage;
            fade_in(stage.arena_mut(), stretch, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), stretch_label, v(0.0, 0.1), 1.0)?.rt(0.5),
        );
        stage.scene_mut().bring_to_front(&marks)?;
        stage.scene_mut().bring_to_front(&[marker])?;
        // the lower-third stays until the next quiz card comes in
        let anytime = format!("{ANYTIME} ({} → {})", p.running[1], p.running[2]);
        let note = lower_third(&kit, stage, FOR_PROGRAMMERS, &anytime)?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.15), 1.0)?.rt(0.5));
        // a breath of music to read the stretch and the note before the quiz
        // (R2, M8: too much to read in the line alone)
        kit.music_breath(stage, NOTE_READ_S)?;

        // 05_4: your turn: six times forty-three; the lower-third stays
        // while "your turn" is said
        let line = say(&kit, stage, "05_4")?;
        let mut gone = axis.clone();
        gone.extend(&marks);
        gone.extend([marker, stretch, stretch_label, one, head[0], head[1], held_now]);
        gone.extend(at_label);
        fade_out_all(stage, &gone, 0.5)?;
        let look = QuizLook::at(QUIZ_RIGHT, QUIZ_SCALE);
        let card = QuizCard::build(&kit, stage, "05_4", look)?;
        line.wait_last(stage)?;
        play!(stage; fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.3));
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 05_5: six times forty, then six times three, on a rectangle of
        // two pieces, the total ending on the answer already on the card
        let line = say(&kit, stage, "05_5")?;
        let (x, y) = shown::TWO_PIECES;
        let six = by_parts(x, y);
        let six_board = Board::new(SIX_AT, SIX_UNIT);
        let said = ["two hundred forty", "Two hundred fifty-eight"];
        let (six_work, six_numbers, six_total) = work(&kit, stage, &line, &six, six_board, SIX_TOTAL_RIGHT, &said)?;
        line.wait_sentence(stage, six.pieces.len())?;
        // how many pieces: digits times digits (it stays until the next
        // quiz card comes in)
        let digits = |n: u64| num(n).len();
        if digits(x) * digits(y) != six.pieces.len() || digits(a) * digits(b) != p.pieces.len() {
            return Err(fail("the pieces are not digits times digits".into()));
        }
        let counts = format!(
            "{} × {} {HERE} {} × {} {BEFORE}",
            digits(x),
            digits(y),
            digits(a),
            digits(b)
        );
        let note = lower_third(&kit, stage, PIECES_ARE, &counts)?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.15), 1.0)?.rt(0.5));
        into_the_card(&kit, stage, &six_total, &card)?;

        // 05_6: the one we left: the waiting card comes back beside the
        // quiz card; thirteen times fourteen
        let line = say(&kit, stage, "05_6")?;
        let mut gone = six_work.mobs();
        gone.extend(six_numbers);
        gone.extend(six_total.mobs());
        gone.extend(card.mobs());
        let mut anims = Vec::new();
        for &m in &gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        for &m in &waiting {
            anims.push(stage.prepare(fly(m, WAITING_BESIDE, WAITING_SCALE, 1.0)?)?);
        }
        stage.play_prepared(anims)?;
        let card = QuizCard::build(&kit, stage, "05_6", look)?;
        line.wait_last(stage)?;
        play!(stage; fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.3));
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // 05_7: ten times ten, ten times four, three times ten, three times
        // four, the total ending on the card's 182; then the 182 and a 25
        // fill in the waiting card: 135² = 18225
        let line = say(&kit, stage, "05_7")?;
        let five =
            ends_in_five(shown::FIVE_HARD).ok_or_else(|| fail(format!("{} does not end in five", shown::FIVE_HARD)))?;
        let hard = by_parts(five.n, five.n + 1);
        if hard.product() != five.hundreds || hard.pieces.len() != 4 {
            return Err(fail(format!("{} × {} is not {}", five.n, five.n + 1, five.hundreds)));
        }
        let hard_board = Board::new(HARD_AT, HARD_UNIT);
        let said = [
            "one hundred.",
            "one hundred forty",
            "one hundred seventy",
            "one hundred eighty-two",
        ];
        let (_, _, hard_total) = work(&kit, stage, &line, &hard, hard_board, HARD_TOTAL_RIGHT, &said)?;
        into_the_card(&kit, stage, &hard_total, &card)?;
        line.wait_sentence(stage, hard.pieces.len())?;
        let size = WAITING_PT * WAITING_SCALE;
        let corner_n = five.square - 100 * five.hundreds;
        // the waiting line, cut where it changes: its square stays put, the
        // product and its question mark go (the source as `waiting_card`
        // sets it; the cut checks every glyph)
        let square_src = format!("{}^2", five.number);
        let asked_src = format!(r":\quad {} \times {} = \;?", five.n, five.n + 1);
        let old = cut_into(&kit, stage, waiting[1], &[&square_src, &asked_src], size, ink)?;
        for &m in &old {
            stage.add_to_scene(m)?;
        }
        stage.remove_from_scene(waiting[1]);
        // the filled line, its square over the square already there
        let filled_src = format!("{square_src} = {}", five.square);
        let filled = eq(&kit, stage, &filled_src, size, ORIGIN)?;
        let filled_glyphs = glyphs(stage, filled);
        let square_n = glyphs(stage, old[0]).len();
        let square_glyphs = filled_glyphs
            .get(..square_n)
            .ok_or_else(|| fail(format!("{filled_src} is shorter than {square_src}")))?;
        let square_at = middle_of(stage, square_glyphs)?;
        let goal = stage.get_center(old[0]);
        stage.shift(filled, v(goal[0] - square_at[0], goal[1] - square_at[1]));
        let filled_parts = cut_into(
            &kit,
            stage,
            filled,
            &[&square_src, "=", &num(five.hundreds), &num(corner_n)],
            size,
            ink,
        )?;
        let lead_n = num(five.hundreds).len();
        let tail = filled_glyphs.len() - num(corner_n).len();
        for &g in &filled_glyphs[tail - lead_n..tail] {
            stage.set_color(g, hundreds(), None, true);
        }
        for &g in &filled_glyphs[tail..] {
            stage.set_color(g, corner(), None, true);
        }
        let answer_at = stage.get_center(card.answer);
        let lead = eq(&kit, stage, &num(five.hundreds), card.look.pt(), answer_at)?;
        stage.add_to_scene(lead)?;
        let tail_at = stage.get_center(filled_parts[3]);
        let then = kit.tex(stage, &num(corner_n), size, corner(), &[], tail_at)?;
        let lead_to = stage.get_center(filled_parts[2]);
        play!(stage;
            fade_out(stage.arena_mut(), old[1], ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), filled_parts[1], ORIGIN, 1.0)?.rt(0.5),
            fly_paint(lead, lead_to, size / card.look.pt(), hundreds(), 1.0)?,
        );
        play!(stage; fade_in(stage.arena_mut(), then, v(0.4, 0.0), 1.0)?.rt(0.5));
        // the card is filled in: one line, and its card fitted round it with
        // its edge lit
        stage.add_to_scene(filled)?;
        for m in [old[0], filled_parts[1], lead, then] {
            stage.remove_from_scene(m);
        }
        line.wait_sentence(stage, hard.pieces.len() + 1)?;
        let fitted = Rectangle::new()
            .width(stage.get_width(filled) + 0.5 * WAITING_SCALE)
            .height(stage.get_height(filled) + 0.4 * WAITING_SCALE)
            .corner_radius(0.1 * WAITING_SCALE)
            .style(Style::default().fill(panel(), 0.94).stroke(ink, 1.5, 1.0))
            .build()?
            .moved_to(stage.get_center(filled));
        let fitted = stage.arena_mut().add(fitted);
        play!(stage;
            Transform::new(waiting[0], fitted).rt(0.7),
            indicate(stage.arena_mut(), filled, 1.08, Some(rgb(ink)))?.rt(0.8),
        );
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
