//! Chapter 04, same number: from 03's slates, Purple and Teal read at the
//! same moment and both write 5 (04_1–04_2; `model::purple_and_teal_tie`,
//! oracle C-F7.model.tie.*); comparing tickets alone lets both in, or
//! stalls them bowing forever (04_3; the model's `<` and `≤` variants from
//! the same slates, tested in `model.rs`); the tie-breaker, a pair compared
//! like words in a dictionary (04_4); and every pair on a grid swept column
//! by column, which serves Pink, Green, Blue, Purple, Teal (04_5–04_6;
//! `model::service_orders`).
//!
//! [`PairGrid`] is the video's grid of (ticket, customer number) pairs;
//! 07_6 replays its sweep, 08_8 tears it and 13_3 relabels it.

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    Figure, badge, chalk, customer, fade_in_all, fade_out_all, fail, group, line_style, order_gold, room_red, say,
    scanline, wait_until,
};
use super::glyphs as g;
use super::tickets::{Bakery, LANE_Y, ROOM_C, ROOM_W, SLATE_SIZE, SLATE_X, set_bakery};
use crate::model::{FIVE, State, blue_takes_a_ticket, purple_and_teal_tie, service_orders};

pub struct SameNumber {
    pub kit: Kit,
}

// ------------------------------------------------------------ the pair grid

/// The grid of pairs: ticket across, customer number up; each pair is its
/// customer's badge at (ticket, customer number). Sweeping it column by
/// column, bottom to top, visits the pairs in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairGrid {
    pub center: Vec3,
    pub cell: f64,
    /// Columns: tickets 1 to `tickets`.
    pub tickets: u32,
    /// Rows: customer numbers 1 to `customers`.
    pub customers: usize,
}

impl PairGrid {
    fn width(&self) -> f64 {
        self.cell * f64::from(self.tickets)
    }

    fn height(&self) -> f64 {
        self.cell * self.customers as f64
    }

    /// The centre of the cell for pair (ticket, customer number).
    pub fn p(&self, ticket: u32, k: usize) -> Vec3 {
        v(
            self.center[0] - self.width() / 2.0 + self.cell * (f64::from(ticket) - 0.5),
            self.center[1] - self.height() / 2.0 + self.cell * (k as f64 - 0.5),
        )
    }

    /// The x of the boundary just before column `ticket`.
    pub fn before(&self, ticket: f64) -> f64 {
        self.center[0] - self.width() / 2.0 + self.cell * (ticket - 1.0)
    }

    pub fn bottom(&self) -> f64 {
        self.center[1] - self.height() / 2.0
    }

    pub fn top(&self) -> f64 {
        self.center[1] + self.height() / 2.0
    }

    /// The grid's lines, faint.
    pub fn lines(&self) -> fmn::Result<VMobject> {
        let (l, b) = (self.center[0] - self.width() / 2.0, self.bottom());
        let mut out = Vec::new();
        for c in 0..=self.tickets {
            let x = l + self.cell * f64::from(c);
            out.push(
                Line::new(v(x, b), v(x, self.top()))
                    .style(line_style(Palette::faint(), 1.5))
                    .build()?,
            );
        }
        for r in 0..=self.customers {
            let y = b + self.cell * r as f64;
            out.push(
                Line::new(v(l, y), v(l + self.width(), y))
                    .style(line_style(Palette::faint(), 1.5))
                    .build()?,
            );
        }
        Ok(v_group(out))
    }

    /// The column and row numbers and the two axis titles (`across`, `up`).
    pub fn labels(&self, kit: &Kit, stage: &mut Stage<'_>, across: &str, up: &str) -> fmn::Result<Vec<Mob>> {
        let mut out = Vec::new();
        for t in 1..=self.tickets {
            let at = v(self.p(t, 1)[0], self.bottom() - 0.3);
            out.push(kit.text(stage, &t.to_string(), 24.0, chalk(), at)?);
        }
        for k in 1..=self.customers {
            let at = v(self.center[0] - self.width() / 2.0 - 0.3, self.p(1, k)[1]);
            out.push(kit.text(stage, &k.to_string(), 24.0, customer(k), at)?);
        }
        out.push(kit.text(
            stage,
            across,
            24.0,
            Palette::dim(),
            v(self.center[0], self.bottom() - 0.72),
        )?);
        let side = kit.text(
            stage,
            up,
            24.0,
            Palette::dim(),
            v(self.center[0] - self.width() / 2.0 - 0.75, self.center[1]),
        )?;
        stage.rotate(side, std::f64::consts::FRAC_PI_2, OUT, None, None);
        out.push(side);
        Ok(out)
    }

    /// Pair (ticket, customer number) as the customer's badge in its cell.
    pub fn pair(&self, kit: &Kit, stage: &mut Stage<'_>, ticket: u32, k: usize) -> fmn::Result<Mob> {
        badge(kit, stage, k, 0.32 * self.cell, self.p(ticket, k))
    }

    /// The gold scanline at the boundary before column `ticket`.
    pub fn scanline(&self, ticket: f64) -> fmn::Result<VMobject> {
        scanline(self.before(ticket), self.bottom() - 0.1, self.top() + 0.1)
    }
}

/// A pair as a token: a chalk ticket with its customer's badge clipped to
/// its right edge (04_4; never two bare numerals). Detached, centred at `at`.
pub fn pair_token(kit: &Kit, stage: &mut Stage<'_>, ticket: u32, k: usize, at: Vec3, size: f64) -> fmn::Result<Mob> {
    let stub = Rectangle::new()
        .width(1.1 * size)
        .height(0.8 * size)
        .corner_radius(0.1 * size)
        .style(
            Style::default()
                .fill(super::draw::board(), 1.0)
                .stroke(chalk(), 3.0, 1.0),
        )
        .build()?
        .moved_to(v(at[0] - 0.2 * size, at[1]));
    let stub = stage.arena_mut().add(stub);
    let num = kit.text(
        stage,
        &ticket.to_string(),
        54.0 * size,
        chalk(),
        v(at[0] - 0.28 * size, at[1]),
    )?;
    let clip = badge(kit, stage, k, 0.27 * size, v(at[0] + 0.42 * size, at[1] + 0.22 * size))?;
    group(stage, &[stub, num, clip])
}

// ------------------------------------------------------------------- scene

/// The pairs of 04_5–04_6: every customer holding a ticket, (ticket, k).
fn pairs(st: &State) -> Vec<(u32, usize)> {
    st.number
        .iter()
        .enumerate()
        .filter(|(_, t)| **t > 0)
        .map(|(i, &t)| (t, i + 1))
        .collect()
}

impl SceneConstruct for SameNumber {
    fn name(&self) -> &str {
        "04_same_number"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (mut b, room) = rewind(&kit, stage)?;
        let (st2, chips) = same_ticket(&kit, stage, &mut b)?;
        let words = no_tie_breaker(&kit, stage, &mut b, &st2, &chips)?;
        let tokens = tie_breaker(&kit, stage, &mut b, &room, &st2, words)?;
        sweep(&kit, stage, &b, &st2, tokens)?;
        kit.hold(stage)?;
        fade_all(stage, 0.3)?;
        kit.finish(stage)
    }
}

/// 04_1: back to 03's slates, before anyone was served.
fn rewind(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Bakery, [Mob; 3])> {
    let st1 = blue_takes_a_ticket()
        .last()
        .map(|s| s.after.clone())
        .ok_or_else(|| fail("Blue's doorway has no steps".into()))?;
    let (b, room) = set_bakery(kit, stage, &st1.number)?;
    say(kit, stage, "04_1")?;
    let mut all = b.all();
    all.extend(room);
    // the rewind: everything slides in from the right, as if played backwards
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &all {
        let a = fade_in(stage.arena_mut(), m, v(0.6, 0.0), 1.0)?;
        anims.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(anims)?;
    let q = predict_pause(kit, stage, v(1.6, 0.0))?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    Ok((b, room))
}

/// 04_2: Purple and Teal run their doorways side by side, replayed on the
/// model: the same reads, the same biggest ticket, the same ticket.
fn same_ticket(kit: &Kit, stage: &mut Stage<'_>, b: &mut Bakery) -> fmn::Result<(State, Vec<Mob>)> {
    let line = say(kit, stage, "04_2")?;
    let steps = purple_and_teal_tie();
    let st2 = steps
        .last()
        .map(|s| s.after.clone())
        .ok_or_else(|| fail("the tie has no steps".into()))?;
    line.wait_for(stage, "read at the same moment")?;
    // the reads, two at a time (one per reader), chips on their lanes
    let xs = [-3.3, -2.35, -1.4, -0.45];
    let mut slot = [0usize; 6];
    let mut pending: Vec<Box<dyn Animation>> = Vec::new();
    let mut arrows = Vec::new();
    let mut chips = Vec::new();
    let mut seen: Vec<Vec<u32>> = vec![Vec::new(); 6];
    for s in &steps {
        let Some(rest) = s.label.strip_prefix("read number[") else {
            continue;
        };
        let Some((who, value)) = rest.split_once("]=") else {
            continue;
        };
        let (who, value): (usize, u32) = (
            who.parse().map_err(|_| fail(format!("unreadable read {}", s.label)))?,
            value
                .parse()
                .map_err(|_| fail(format!("unreadable read {}", s.label)))?,
        );
        let reader = s.customer;
        let j = slot[reader];
        slot[reader] += 1;
        seen[reader].push(value);
        let y = LANE_Y[reader - 1];
        let from = v(SLATE_X + 0.7 * SLATE_SIZE + 0.05, LANE_Y[who - 1]);
        let to = v(xs[j], y + 0.05 * if LANE_Y[who - 1] > y { 1.0 } else { -1.0 });
        let arrow = Arrow::new(from, to)
            .buff(0.06)
            .thickness(2.5)
            .style(
                Style::default()
                    .fill(customer(reader), 0.8)
                    .stroke(customer(reader), 2.5, 0.8),
            )
            .build()?;
        let arrow = stage.arena_mut().add(arrow);
        let chip = super::draw::chalk_number(kit, stage, value, v(xs[j], y + 0.3), 0.45)?;
        let a = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?;
        pending.push(stage.prepare(a.rt(0.45))?);
        let c = fade_in(stage.arena_mut(), chip, v(0.0, -0.1), 1.0)?;
        pending.push(stage.prepare(c.rt(0.45))?);
        arrows.push(arrow);
        chips.push(chip);
        if pending.len() == 4 {
            stage.play_prepared(std::mem::take(&mut pending))?;
            stage.wait(0.25)?;
        }
    }
    if !pending.is_empty() {
        stage.play_prepared(pending)?;
    }
    if seen[3] != seen[4] {
        return Err(fail(format!("Purple read {:?} but Teal read {:?}", seen[3], seen[4])));
    }
    line.wait_for(stage, "both write")?;
    fade_out_all(stage, &arrows, 0.3)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    let mut sums = Vec::new();
    for k in [3, 4] {
        let t = st2.number[k - 1];
        let biggest = seen[k].iter().copied().max().unwrap_or(0);
        let sum = kit.text(
            stage,
            &format!("{biggest} + 1 = {t}"),
            26.0,
            chalk(),
            v(0.75, LANE_Y[k - 1] + 0.3),
        )?;
        sums.push(sum);
        let a = write(stage.arena(), sum);
        anims.push(stage.prepare(a.rt(0.6))?);
        let [x, w] = b.slates[k - 1].rewrite(kit, stage, t)?;
        anims.push(x);
        anims.push(w);
    }
    stage.play_prepared(anims)?;
    // the two equal tickets, side by side
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for k in [3, 4] {
        let a = fmn::animation::indicate(stage.arena_mut(), b.slates[k - 1].value, 1.4, Some(rgb_gold()))?;
        anims.push(stage.prepare(a.rt(0.8))?);
    }
    stage.play_prepared(anims)?;
    kit.hold(stage)?;
    // the reads and sums leave as 04_3 begins (no wait of their own)
    chips.extend(sums);
    Ok((st2, chips))
}

fn rgb_gold() -> [f32; 3] {
    [1.0, 0.847, 0.4]
}

/// A rewind mark (two triangles pointing back), chalk, centred at `at`.
fn rewind_mark(at: Vec3) -> fmn::Result<VMobject> {
    let tri = |dx: f64| {
        Polygon::new([
            v(at[0] + dx + 0.22, at[1] + 0.2),
            v(at[0] + dx - 0.12, at[1]),
            v(at[0] + dx + 0.22, at[1] - 0.2),
        ])
        .style(super::draw::flat(chalk(), 0.85))
        .build()
    };
    Ok(v_group([tri(-0.17), tri(0.17)]))
}

/// Everyone with a ticket smaller than Purple and Teal's, in the order the
/// room serves them from 04_2's slates (the model's order).
fn served_before_the_tie(st2: &State) -> fmn::Result<Vec<usize>> {
    let order = service_orders(&FIVE, st2, &[1, 2, 3, 4, 5])
        .into_iter()
        .next()
        .ok_or_else(|| fail("no service order".into()))?;
    let tie = st2.number[2];
    Ok(order.into_iter().filter(|&k| st2.number[k - 1] < tie).collect())
}

/// Walk a figure to `to`.
fn walk(f: &Figure, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(f.group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// 04_3: tickets alone. With "smaller", both end up in the room; with
/// "smaller or equal", both wait, bowing, forever.
/// Returns the rule and the two "after you" labels, which 04_4 clears.
fn no_tie_breaker(
    kit: &Kit,
    stage: &mut Stage<'_>,
    b: &mut Bakery,
    st2: &State,
    chips: &[Mob],
) -> fmn::Result<[Mob; 3]> {
    let line = say(kit, stage, "04_3")?;
    let rule = kit.text(stage, g::RULE_SMALLER, 28.0, Palette::ink(), v(-0.6, 3.3))?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in chips {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    let a = fade_in(stage.arena_mut(), rule, v(0.0, 0.1), 1.0)?;
    anims.push(stage.prepare(a.rt(0.5))?);
    stage.play_prepared(anims)?;
    // under either rule, everyone with a smaller ticket goes first (the
    // model's order from 04_2's slates); then Purple and Teal are left
    let door = ROOM_C[0] - ROOM_W / 2.0;
    for k in served_before_the_tie(st2)? {
        let f = &b.figures[k - 1];
        let home = super::tickets::figure_at(k);
        play!(stage; walk(f, v(ROOM_C[0], 0.0), 0.4)?);
        let [a, w] = b.slates[k - 1].rewrite(kit, stage, 0)?;
        let back = walk(f, home, 0.4)?;
        let back = stage.prepare(back)?;
        stage.play_prepared(vec![a, w, back])?;
    }
    let purple = &b.figures[2];
    let teal = &b.figures[3];
    let inside = [v(ROOM_C[0], 0.45), v(ROOM_C[0], -0.45)];
    play!(stage;
        walk(purple, v(door - 0.7, 0.45), 1.0)?,
        walk(teal, v(door - 0.7, -0.45), 1.0)?,
    );
    play!(stage;
        walk(purple, inside[0], 0.7)?,
        walk(teal, inside[1], 0.7)?,
    );
    let flash = Rectangle::new()
        .width(ROOM_W + 0.15)
        .height(super::tickets::ROOM_H + 0.15)
        .corner_radius(0.26)
        .style(Style::default().fill(room_red(), 0.0).stroke(room_red(), 8.0, 1.0))
        .build()?
        .moved_to(ROOM_C);
    let flash = stage.arena_mut().add(flash);
    let both = kit.text(stage, g::BOTH_INSIDE, 30.0, room_red(), v(ROOM_C[0], -1.6))?;
    play!(stage;
        fmn::animation::indicate(stage.arena_mut(), flash, 1.04, None)?.rt(0.7),
        fade_in(stage.arena_mut(), both, ORIGIN, 1.0)?.rt(0.5),
    );
    stage.remove_from_scene(flash);
    // the other rule: both wait, bowing to each other forever
    line.wait_for(stage, "Change the rule")?;
    let rule2 = kit.text(stage, g::RULE_SMALLER_OR_EQUAL, 28.0, Palette::ink(), v(-0.6, 3.3))?;
    play!(stage;
        fade_out(stage.arena_mut(), rule, v(0.0, 0.2), 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), rule2, v(0.0, -0.2), 1.0)?.rt(0.4),
        fade_out(stage.arena_mut(), both, ORIGIN, 1.0)?.rt(0.4),
        walk(purple, v(door - 0.75, 0.45), 0.8)?,
        walk(teal, v(door - 0.75, -0.45), 0.8)?,
    );
    line.wait_for(stage, "bowing")?;
    let after = kit.text(stage, g::AFTER_YOU, 24.0, Palette::dim(), v(door - 2.2, 0.75))?;
    let no_after = kit.text(stage, g::NO_AFTER_YOU, 24.0, Palette::dim(), v(door - 2.2, -0.75))?;
    fade_in_all(stage, &[after, no_after], 0.4)?;
    // "after you", "no, after you": they trade bows, a note each, forever
    let notes = 4;
    let (wav, secs) =
        super::tunes::write("04_3", super::tunes::after_you(3, 4, notes)).map_err(|e| fail(e.to_string()))?;
    let beat = 60.0 / 104.0;
    kit.sonify(stage, secs, "04_3", &wav)?;
    let t0 = stage.scene().time().to_f64();
    for n in 0..notes {
        let f = if n % 2 == 0 { purple } else { teal };
        wait_until(stage, t0 + n as f64 * beat)?;
        let base = stage.get_center(f.body);
        let pivot = v(base[0], base[1] - 0.35);
        let tilt = if f.k == 3 { -0.3 } else { 0.3 };
        for (rt, angle) in [(0.24, tilt), (0.24, -tilt)] {
            let bow = f
                .group
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .rotate_about(angle, OUT, Some(pivot))?;
            play!(stage; bow);
        }
    }
    wait_until(stage, t0 + secs)?;
    Ok([rule2, after, no_after])
}

/// 04_4: the tie-breaker. (5, Purple 3) before (5, Teal 4), like "cat"
/// before "cow".
fn tie_breaker(
    kit: &Kit,
    stage: &mut Stage<'_>,
    b: &mut Bakery,
    room: &[Mob; 3],
    st2: &State,
    words: [Mob; 3],
) -> fmn::Result<[Mob; 2]> {
    let line = say(kit, stage, "04_4")?;
    fade_out_all(stage, &words, 0.4)?;
    // back to 04_2's slates: the customers 04_3 served hold their
    // tickets again, under a rewind mark (R3 maya-08)
    let rewind = stage.arena_mut().add(rewind_mark(v(0.0, 3.3))?);
    play!(stage; fade_in(stage.arena_mut(), rewind, ORIGIN, 1.0)?.rt(0.3));
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for k in served_before_the_tie(st2)? {
        let [a, w] = b.slates[k - 1].rewrite(kit, stage, st2.number[k - 1])?;
        anims.push(a);
        anims.push(w);
    }
    for k in [3, 4] {
        let a = walk(&b.figures[k - 1], super::tickets::figure_at(k), 0.6)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    play!(stage; fade_out(stage.arena_mut(), rewind, ORIGIN, 1.0)?.rt(0.3));
    let mut out = b.all();
    out.extend(room);
    out.retain(|m| !b.slates.iter().any(|s| s.parts().contains(m)));
    fade_out_all(stage, &out, 0.6)?;
    // the slates stay on screen (M4), the pairs appear beside them
    line.wait_for(stage, "Pair each ticket")?;
    let (t3, t4) = (st2.number[2], st2.number[3]);
    // the two pairs are the picture: large, side by side, the dictionary's
    // two words under them letter for number
    let (x1, x2, y, size) = (-2.3, 3.2, 0.65, 2.0);
    let first = pair_token(kit, stage, t3, 3, v(x1, y), size)?;
    let second = pair_token(kit, stage, t4, 4, v(x2, y), size)?;
    let pair_word = kit.text(stage, g::PAIR, 30.0, Palette::dim(), v(x1 - 0.4, y - 1.1))?;
    play!(stage;
        fade_in(stage.arena_mut(), first, v(0.0, 0.2), 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), pair_word, ORIGIN, 1.0)?.rt(0.6),
    );
    play!(stage; fade_in(stage.arena_mut(), second, v(0.0, 0.2), 1.0)?.rt(0.6));
    // tickets first; only on a tie, customer numbers (a dictionary)
    line.wait_for(stage, "the way a dictionary")?;
    let cat = kit.text(stage, g::CAT, 72.0, Palette::ink(), v(x1, y - 2.1))?;
    let cow = kit.text(stage, g::COW, 72.0, Palette::ink(), v(x2, y - 2.1))?;
    fade_in_all(stage, &[cat, cow], 0.5)?;
    line.wait_for(stage, "Purple is three")?;
    // between the first pair's badge and the second pair's ticket
    let mid = (x1 + x2) / 2.0 - 0.06;
    let before = kit.text(stage, g::COMES_BEFORE, 34.0, order_gold(), v(mid, y + 0.45))?;
    let lt = Arrow::new(v(mid - 0.75, y - 0.05), v(mid + 0.75, y - 0.05))
        .buff(0.0)
        .thickness(3.0)
        .style(Style::default().fill(order_gold(), 1.0).stroke(order_gold(), 3.0, 1.0))
        .build()?;
    let lt = stage.arena_mut().add(lt);
    play!(stage;
        write(stage.arena(), before).rt(0.6),
        fmn::animation::grow_arrow(stage.arena_mut(), lt)?.rt(0.6),
    );
    kit.hold(stage)?;
    fade_out_all(stage, &[cat, cow, before, lt, pair_word], 0.4)?;
    Ok([first, second])
}

/// 04_5–04_6: every pair on the grid; the gold scanline sweeps it.
fn sweep(kit: &Kit, stage: &mut Stage<'_>, b: &Bakery, st2: &State, tokens: [Mob; 2]) -> fmn::Result<()> {
    let line = say(kit, stage, "04_5")?;
    let held = pairs(st2);
    let top = held.iter().map(|p| p.0).max().unwrap_or(1);
    let grid = PairGrid {
        center: v(1.15, 0.35),
        cell: 1.0,
        tickets: top + 1,
        customers: st2.number.len(),
    };
    let lines = stage.arena_mut().add(grid.lines()?);
    let labels = grid.labels(kit, stage, g::TICKET_ACROSS, g::CUSTOMER_NUMBER_UP)?;
    let mut ins = vec![lines];
    ins.extend(&labels);
    fade_out_all(stage, &tokens, 0.4)?;
    fade_in_all(stage, &ins, 0.6)?;
    // each pair flies from its slate to its cell
    let mut badges = Vec::new();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, k) in &held {
        let m = grid.pair(kit, stage, t, k)?;
        let from = b.slates[k - 1].center;
        let to = grid.p(t, k);
        stage.shift(m, v(from[0] - to[0], from[1] - to[1]));
        stage.add_to_scene(m)?;
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
        badges.push((t, k, m));
    }
    let g_fly = fmn::animation::lagged_start(stage.arena_mut(), anims)?;
    stage.play(g_fly)?;
    // the scanline waits before column 2
    line.wait_for(stage, "sweep column by column")?;
    let scan = stage.arena_mut().add(grid.scanline(1.0)?);
    play!(stage; fade_in(stage.arena_mut(), scan, ORIGIN, 1.0)?.rt(0.4));
    play!(stage; scan.animate().set_anim_args(AnimateArgs { run_time: Some(0.6), ..AnimateArgs::default() })?
        .shift(v(grid.cell, 0.0))?);
    let q = predict_pause(kit, stage, v(5.9, 2.4))?;

    // 04_6: the sweep serves Pink, Green, Blue, Purple, Teal
    let line = say(kit, stage, "04_6")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    let order = service_orders(&FIVE, st2, &[1, 2, 3, 4, 5]);
    let order = order
        .iter()
        .next()
        .cloned()
        .ok_or_else(|| fail("no service order".into()))?;
    let mut by_sweep = held.clone();
    by_sweep.sort_unstable();
    if by_sweep.iter().map(|p| p.1).collect::<Vec<_>>() != order {
        return Err(fail(format!(
            "the sweep gives {by_sweep:?}, the model serves {order:?}"
        )));
    }
    let served_head = kit.text(stage, g::SERVED, 26.0, Palette::dim(), v(6.15, 2.35))?;
    play!(stage; fade_in(stage.arena_mut(), served_head, ORIGIN, 1.0)?.rt(0.4));
    line.wait_for(stage, "The customer number only")?;
    let mut x_now = grid.before(2.0);
    for (n, &(t, k)) in by_sweep.iter().enumerate() {
        // the scanline passes this pair's column; then the pair is served
        let x = grid.before(f64::from(t) + 1.0);
        if x > x_now + 1e-9 {
            play!(stage; scan.animate().set_anim_args(AnimateArgs { run_time: Some(0.4), ..AnimateArgs::default() })?
                .shift(v(x - x_now, 0.0))?);
            x_now = x;
        }
        let m = badges
            .iter()
            .find(|b| b.0 == t && b.1 == k)
            .map(|b| b.2)
            .ok_or_else(|| fail(format!("no badge for pair ({t}, {k})")))?;
        let to = v(6.15, 1.75 - 0.62 * n as f64);
        let copy = badge(kit, stage, k, 0.25, grid.p(t, k))?;
        let a = copy
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.45),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        let flash = fmn::animation::indicate(stage.arena_mut(), m, 1.3, Some(rgb_gold()))?;
        stage.add_to_scene(copy)?;
        play!(stage; a, flash.rt(0.45));
    }
    let lower = lower_third(kit, stage, g::PRECISELY, g::TOTALLY_ORDERED)?;
    play!(stage; fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.5));
    kit.hold(stage)?;
    kit.music_breath(stage, 0.4)?;
    Ok(())
}
