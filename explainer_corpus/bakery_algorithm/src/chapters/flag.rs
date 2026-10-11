//! Chapter 05, the unwritten ticket: Blue decides on ticket 1 but has not
//! written it, so Green sees 0 and walks in, then Blue does too (05_1–05_3;
//! the checker's trace a, replayed on the model); the flag (05_4); the same
//! order with flags, where Green waits (05_5–05_6; trace a′); every order of
//! two customers without and with flags, 28 with 4 bad and 322 with none
//! (05_7; `model::bakery_orders`, oracle C-F3); choosing time, which is not
//! a lock (05_8); and the thesis: choosing at once is fine (05_9; trace c,
//! equal tickets settled by the pair).

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use explainer_score::rng::SplitMix64;
use fmn::animation::UpdateFromFunc;
use fmn::prelude::*;

use super::draw::{
    Flag, Lattice, chalk_number, customer, fade_in_all, fade_out_all, faded_line, fail, fan, flag, flag_amber,
    line_style, order_gold, room_red, say, sign, wait_until,
};
use super::glyphs as g;
use super::tickets::{Bakery, LANE_Y, ROOM_C, SLATE_SIZE, SLATE_X, figure_at, set_bakery};
use super::ties::pair_token;
use crate::model::{Cmp, Order, Step, Trace, bakery_orders, bakery_program, red_point};

pub struct UnwrittenTicket {
    pub kit: Kit,
}

/// 05_7's progress square: the 7×7 lattice of two customers with flags
/// (06 extrudes it into the cube, so 06 imports this very value).
pub const SQUARE_05: Lattice = Lattice {
    center: [-2.4, -0.1, 0.0],
    side: 5.4,
    steps: 7,
};

/// How far 05_7's 28 staircases fan.
const FAN_4: f64 = 0.6;

/// Where a counted step of trace a′ sits along time (13 slots); trace a's
/// steps sit where the same step of a′ does, so the flag steps slot in.
fn slot_x(k: usize) -> f64 {
    -3.45 + 0.55 * k as f64
}

/// Trace a's counted steps, by their slot in a′ (P1 read, P2 read, P2
/// write, P2 enters, P1 write, P1 enters).
const A_SLOTS: [usize; 6] = [1, 3, 4, 5, 6, 9];

/// A flag's foot on customer `k`'s lane, just after her slate.
fn flag_foot(k: usize) -> Vec3 {
    v(SLATE_X + 0.75, LANE_Y[k - 1] - 0.3)
}

/// Where customer `k` stands in the room.
fn inside(k: usize) -> Vec3 {
    v(ROOM_C[0], 0.9 - 0.9 * (k as f64 - 1.0))
}

/// The two-customer stage of 05: the bakery with everyone's flag, and the
/// marks the replay leaves on the lanes.
struct Stage05 {
    bakery: Bakery,
    room: [Mob; 3],
    flags: Vec<Flag>,
    marks: Vec<Mob>,
}

impl Stage05 {
    fn flag_mobs(&self) -> Vec<Mob> {
        self.flags.iter().flat_map(|f| [f.pole, f.pennant]).collect()
    }
}

/// Draw one replayed step at slot `k`: a read is an arrow from the slate or
/// flag glanced at into the reader's lane with what she saw; a write
/// rewrites her own slate; a flag step raises or lowers her flag; an entry
/// walks her into the room and starts a red band on her lane.
fn draw_step(kit: &Kit, stage: &mut Stage<'_>, s5: &mut Stage05, step: &Step, k: usize) -> fmn::Result<()> {
    let who = step.customer;
    let y = LANE_Y[who - 1];
    let x = slot_x(k);
    let label = step.label.as_str();
    let dot = Dot::new().point(v(x, y)).radius(0.07).color(customer(who)).build();
    let dot = stage.arena_mut().add(dot);
    s5.marks.push(dot);
    let glance = |stage: &mut Stage<'_>, from: Vec3, color: Srgb, text: &str| -> fmn::Result<[Mob; 2]> {
        let arrow = Arrow::new(from, v(x, y + if from[1] > y { 0.06 } else { -0.06 }))
            .buff(0.06)
            .thickness(2.5)
            .style(Style::default().fill(color, 0.85).stroke(color, 2.5, 0.85))
            .build()?;
        let arrow = stage.arena_mut().add(arrow);
        let chip = kit.text(stage, text, 26.0, color, v(x, y + 0.32))?;
        Ok([arrow, chip])
    };
    let mut anims: Vec<Box<dyn Animation>> = vec![{
        let a = fade_in(stage.arena_mut(), dot, ORIGIN, 1.0)?;
        stage.prepare(a.rt(0.25))?
    }];
    if let Some(rest) = label.strip_prefix("read number[") {
        let (q, value) = rest
            .split_once("]=")
            .ok_or_else(|| fail(format!("unreadable {label}")))?;
        let q: usize = q.parse().map_err(|_| fail(format!("unreadable {label}")))?;
        let from = v(SLATE_X + 0.7 * SLATE_SIZE + 0.05, LANE_Y[q - 1]);
        let [arrow, chip] = glance(stage, from, customer(who), value)?;
        let a = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        let c = fade_in(stage.arena_mut(), chip, ORIGIN, 1.0)?;
        anims.push(stage.prepare(c.rt(0.5))?);
        s5.marks.extend([arrow, chip]);
    } else if let Some(rest) = label.strip_prefix("L2: choosing[") {
        let (q, rest) = rest
            .split_once("]=")
            .ok_or_else(|| fail(format!("unreadable {label}")))?;
        let q: usize = q.parse().map_err(|_| fail(format!("unreadable {label}")))?;
        let up = rest.starts_with('1');
        let foot = flag_foot(q);
        let from = v(foot[0] + 0.2, foot[1] + 0.45);
        let word = if up { g::FLAG_UP_WORD } else { g::FLAG_DOWN_WORD };
        let [arrow, chip] = glance(stage, from, flag_amber(), word)?;
        let a = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        let c = fade_in(stage.arena_mut(), chip, ORIGIN, 1.0)?;
        anims.push(stage.prepare(c.rt(0.5))?);
        s5.marks.extend([arrow, chip]);
    } else if let Some(rest) = label.strip_prefix("L3: number[") {
        let (q, rest) = rest
            .split_once("]=")
            .ok_or_else(|| fail(format!("unreadable {label}")))?;
        let q: usize = q.parse().map_err(|_| fail(format!("unreadable {label}")))?;
        let value = rest.split(',').next().unwrap_or("?");
        let from = v(SLATE_X + 0.7 * SLATE_SIZE + 0.05, LANE_Y[q - 1]);
        let [arrow, chip] = glance(stage, from, customer(who), value)?;
        let a = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        let c = fade_in(stage.arena_mut(), chip, ORIGIN, 1.0)?;
        anims.push(stage.prepare(c.rt(0.5))?);
        s5.marks.extend([arrow, chip]);
    } else if label.starts_with("choosing:=") {
        let raised = label.ends_with('1');
        let t = s5.flags[who - 1].set(stage, raised);
        anims.push(Box::new(t.rt(0.5)));
    } else if let Some((w, t)) = step.ticket_written() {
        let [a, b] = s5.bakery.slates[w - 1].rewrite(kit, stage, t)?;
        anims.push(a);
        anims.push(b);
    } else if label.contains(":=0 (exit)") {
        let [a, b] = s5.bakery.slates[who - 1].rewrite(kit, stage, 0)?;
        anims.push(a);
        anims.push(b);
    }
    stage.play_prepared(anims)?;
    if step.enters() {
        let band = Line::new(v(x + 0.1, y), v(slot_x(12) + 0.3, y))
            .style(line_style(room_red(), 8.0))
            .build()?;
        let band = stage.arena_mut().add(band);
        s5.marks.push(band);
        let f = s5.bakery.figures[who - 1].group;
        let walk_in = f
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.7),
                ..AnimateArgs::default()
            })?
            .move_to(inside(who), ORIGIN)?;
        play!(stage; walk_in, show_creation(band).rt(0.7));
    }
    Ok(())
}

impl SceneConstruct for UnwrittenTicket {
    fn name(&self) -> &str {
        "05_the_unwritten_ticket"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let mut s5 = crack(&kit, stage)?;
        the_flag(&kit, stage, &mut s5)?;
        same_order(&kit, stage, &mut s5)?;
        let square = every_order(&kit, stage, &s5)?;
        choosing_time(&kit, stage, &square)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;
        kit.finish(stage)
    }
}

// --------------------------------------------------------------- 05_1–05_3

/// 05_1–05_3: trace a, the crack without flags.
fn crack(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Stage05> {
    // 04's slates, then a fresh start: every slate wiped, three customers dim
    let tie = crate::model::purple_and_teal_tie();
    let st2 = tie
        .last()
        .map(|s| s.after.clone())
        .ok_or_else(|| fail("the tie has no steps".into()))?;
    let (bakery, room) = set_bakery(kit, stage, &st2.number)?;
    let mut s5 = Stage05 {
        bakery,
        room,
        flags: Vec::new(),
        marks: Vec::new(),
    };
    let mut all = s5.bakery.all();
    all.extend(s5.room);
    fade_in_all(stage, &all, 0.6)?;
    let line = say(kit, stage, "05_1")?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for slate in &mut s5.bakery.slates {
        let [a, b] = slate.rewrite(kit, stage, 0)?;
        anims.push(a);
        anims.push(b);
    }
    for k in 3..=5 {
        let mut dim = vec![s5.bakery.figures[k - 1].group, s5.bakery.lanes[k - 1]];
        dim.extend(s5.bakery.slates[k - 1].parts());
        for m in dim {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.8),
                    ..AnimateArgs::default()
                })?
                .set_opacity(0.25)?;
            anims.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared(anims)?;
    // trace a, counted steps only, each at its slot
    let steps: Vec<Step> = Trace::A.steps().into_iter().filter(Step::counted).collect();
    if steps.len() != A_SLOTS.len() {
        return Err(fail(format!("trace a has {} counted steps", steps.len())));
    }
    line.wait_for(stage, "Blue reads")?;
    draw_step(kit, stage, &mut s5, &steps[0], A_SLOTS[0])?;
    // she decides on 1 but has not written it: a ghost over her slate
    line.wait_for(stage, "decides on ticket")?;
    let decided = steps[2..]
        .iter()
        .filter_map(Step::ticket_written)
        .find(|&(w, _)| w == 1)
        .map(|(_, t)| t)
        .ok_or_else(|| fail("Blue never writes in trace a".into()))?;
    let ghost = chalk_number(kit, stage, decided, v(SLATE_X, LANE_Y[0] + 0.62), 0.55)?;
    stage.set_opacity(ghost, 0.45, true);
    play!(stage; fade_in(stage.arena_mut(), ghost, v(0.0, -0.1), 1.0)?.rt(0.5));
    s5.marks.push(ghost);
    // 05_2: Green takes 1, sees Blue's 0, walks in
    let line = say(kit, stage, "05_2")?;
    // her doorway (she reads Blue's 0 and writes 1), then her check of
    // Blue's slate, which lets her in
    for (i, phrase) in [(1, "Green takes"), (2, "Green takes"), (3, "checks Blue's")] {
        line.wait_for(stage, phrase)?;
        draw_step(kit, stage, &mut s5, &steps[i], A_SLOTS[i])?;
    }
    // 05_3: Blue writes her 1, wins the tie, walks in too
    let line = say(kit, stage, "05_3")?;
    line.wait_for(stage, "Blue writes")?;
    play!(stage; fade_out(stage.arena_mut(), ghost, v(0.0, -0.3), 1.0)?.rt(0.4));
    draw_step(kit, stage, &mut s5, &steps[4], A_SLOTS[4])?;
    line.wait_for(stage, "wins it")?;
    draw_step(kit, stage, &mut s5, &steps[5], A_SLOTS[5])?;
    line.wait_for(stage, "Both in the room")?;
    let both = kit.text(stage, g::BOTH_INSIDE, 30.0, room_red(), v(ROOM_C[0], -1.4))?;
    let count = kit.text(
        stage,
        &format!("{} {}", steps.len(), g::STEPS_WORD),
        26.0,
        Palette::dim(),
        v(slot_x(9), LANE_Y[4] - 0.45),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), both, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), count, ORIGIN, 1.0)?.rt(0.5),
    );
    s5.marks.extend([both, count]);
    // two tunes at once: the bug
    let (wav, secs) = super::tunes::write("05_3", super::tunes::clash(1, 2, 2.0)).map_err(|e| fail(e.to_string()))?;
    kit.sonify(stage, secs, "05_3", &wav)?;
    let t0 = stage.scene().time().to_f64();
    let edge = s5.room[1];
    play!(stage; fmn::animation::indicate(stage.arena_mut(), edge, 1.04, None)?.rt(0.8));
    wait_until(stage, t0 + secs)?;
    Ok(s5)
}

// --------------------------------------------------------------------- 05_4

fn the_flag(kit: &Kit, stage: &mut Stage<'_>, s5: &mut Stage05) -> fmn::Result<()> {
    let line = say(kit, stage, "05_4")?;
    // back to the start: both out of the room, the marks cleared
    let mut out = std::mem::take(&mut s5.marks);
    out.retain(|m| stage.scene().mobjects().contains(m));
    fade_out_all(stage, &out, 0.5)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for k in [1, 2] {
        let a = s5.bakery.figures[k - 1]
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .move_to(figure_at(k), ORIGIN)?;
        anims.push(stage.prepare(a)?);
        let [x, w] = s5.bakery.slates[k - 1].rewrite(kit, stage, 0)?;
        anims.push(x);
        anims.push(w);
    }
    stage.play_prepared(anims)?;
    // a flag on every lane: "I'm choosing"
    line.wait_for(stage, "a flag")?;
    for k in 1..=5 {
        s5.flags.push(flag(stage, flag_foot(k), 0.62, false)?);
    }
    let mobs = s5.flag_mobs();
    fade_in_all(stage, &mobs, 0.5)?;
    for k in 3..=5 {
        let f = &s5.flags[k - 1];
        stage.set_opacity(f.pole, 0.25, true);
        stage.set_opacity(f.pennant, 0.25, true);
    }
    let rule = kit.text(stage, g::FLAG_RULE_1, 28.0, Palette::ink(), v(0.0, 3.35))?;
    play!(stage; fade_in(stage.arena_mut(), rule, ORIGIN, 1.0)?.rt(0.4));
    line.wait_for(stage, "raise it before")?;
    let t = s5.flags[0].set(stage, true);
    stage.play(t.rt(0.5))?;
    let rule2 = kit.text(stage, g::FLAG_RULE_2, 28.0, Palette::ink(), v(0.0, 3.35))?;
    play!(stage;
        fade_out(stage.arena_mut(), rule, ORIGIN, 1.0)?.rt(0.3),
        fade_in(stage.arena_mut(), rule2, ORIGIN, 1.0)?.rt(0.3),
    );
    line.wait_for(stage, "lower it")?;
    let t = s5.flags[0].set(stage, false);
    stage.play(t.rt(0.5))?;
    line.wait_for(stage, "Before comparing")?;
    let rule3 = kit.text(stage, g::FLAG_RULE_3, 28.0, Palette::ink(), v(0.0, 3.35))?;
    play!(stage;
        fade_out(stage.arena_mut(), rule2, ORIGIN, 1.0)?.rt(0.3),
        fade_in(stage.arena_mut(), rule3, ORIGIN, 1.0)?.rt(0.3),
    );
    s5.marks.push(rule3);
    Ok(())
}

// --------------------------------------------------------------- 05_5–05_6

/// 05_5–05_6: the same order, with flags: trace a′ until Green waits.
fn same_order(kit: &Kit, stage: &mut Stage<'_>, s5: &mut Stage05) -> fmn::Result<()> {
    say(kit, stage, "05_5")?;
    if let Some(rule) = s5.marks.pop() {
        play!(stage; fade_out(stage.arena_mut(), rule, ORIGIN, 1.0)?.rt(0.3));
    }
    let steps = Trace::APrime.steps();
    let stop = steps
        .iter()
        .position(|s| s.customer == 2 && s.label.starts_with("L2: choosing[1]=1, wait"))
        .ok_or_else(|| fail("in trace a′ Green never waits on Blue's flag".into()))?;
    // the counted steps before Green's check, each at its slot; the flag
    // steps (amber) fall between trace a's
    let before: Vec<&Step> = steps[..stop].iter().filter(|s| s.counted()).collect();
    for (k, step) in before.iter().enumerate() {
        draw_step(kit, stage, s5, step, k)?;
        stage.wait(0.15)?;
    }
    let q = predict_pause(kit, stage, v(2.4, 2.9))?;
    s5.marks.push(q);
    // 05_6: she sees Blue's flag up, and waits
    let line = say(kit, stage, "05_6")?;
    if let Some(q) = s5.marks.pop() {
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    }
    draw_step(kit, stage, s5, &steps[stop], before.len())?;
    line.wait_for(stage, "and waits")?;
    let wait = Line::new(
        v(slot_x(before.len()) + 0.15, LANE_Y[1]),
        v(slot_x(before.len() + 3), LANE_Y[1]),
    )
    .style(faded_line(flag_amber(), 8.0, 0.45))
    .build()?;
    let wait = stage.arena_mut().add(wait);
    let waits = kit.text(
        stage,
        g::WAITS_WORD,
        26.0,
        flag_amber(),
        v(slot_x(before.len() + 2), LANE_Y[1] - 0.32),
    )?;
    play!(stage;
        show_creation(wait).rt(0.5),
        fade_in(stage.arena_mut(), waits, ORIGIN, 1.0)?.rt(0.4),
    );
    s5.marks.extend([wait, waits]);
    kit.hold(stage)?;
    Ok(())
}

// --------------------------------------------------------------------- 05_7

/// The flag set as edge ribbons that thicken as the staircases arrive (spike
/// S-6): each lattice edge one rectangle, stretched by placement to its
/// running multiplicity, shown when its first staircase arrives. Returns the
/// ribbons once they have all grown over `run_time` seconds.
fn ribbons(
    stage: &mut Stage<'_>,
    l: Lattice,
    orders: &[Order],
    width: f64,
    run_time: f64,
    count: Mob,
) -> fmn::Result<Vec<Mob>> {
    let n = orders.len();
    let mut arrival: Vec<usize> = (0..n).collect();
    let mut rng = SplitMix64::new(0x5EED_0507);
    for i in (1..n).rev() {
        arrival.swap(i, (rng.next_u64() % (i as u64 + 1)) as usize);
    }
    let mut users: std::collections::BTreeMap<((usize, usize), bool), Vec<usize>> = Default::default();
    for (r, o) in orders.iter().enumerate() {
        for w in o.lattice().windows(2) {
            users.entry((w[0], w[1].0 > w[0].0)).or_default().push(arrival[r]);
        }
    }
    let m_max = users.values().map(Vec::len).max().unwrap_or(1);
    let thick = move |m: usize| (width * m as f64 / m_max as f64).max(0.014);
    let cell = l.cell();
    struct Ribbon {
        mob: Mob,
        center: Vec3,
        dim: usize,
        full: f64,
        ranks: Vec<usize>,
        factor: f64,
        shown: bool,
    }
    let amber = flag_amber();
    let mut all = Vec::with_capacity(users.len());
    for (((i, j), horizontal), mut ranks) in users {
        ranks.sort_unstable();
        let a = l.p(i as f64, j as f64);
        let (center, dim, w, h) = if horizontal {
            (v(a[0] + cell / 2.0, a[1]), 1, cell, thick(ranks.len()))
        } else {
            (v(a[0], a[1] + cell / 2.0), 0, thick(ranks.len()), cell)
        };
        let r = Rectangle::new()
            .width(w)
            .height(h)
            .style(Style::default().fill(amber, 0.8).stroke(amber, 0.0, 0.0))
            .build()?
            .moved_to(center);
        let full = thick(ranks.len());
        let mob = stage.arena_mut().add(r);
        let factor = thick(1) / full;
        stage.stretch_about(mob, factor, dim, Some(center), None);
        all.push(Ribbon {
            mob,
            center,
            dim,
            full,
            ranks,
            factor,
            shown: false,
        });
    }
    let mobs: Vec<Mob> = all.iter().map(|r| r.mob).collect();
    let anchor = stage.add(VMobject::new())?;
    let grow = UpdateFromFunc::new_alpha(anchor, move |s, _, alpha| {
        let c = (alpha * n as f64).round() as usize;
        s.set_tracker_value(count, c as f64).ok();
        for r in &mut all {
            let m = r.ranks.partition_point(|&k| k < c);
            if m == 0 {
                continue;
            }
            if !r.shown {
                s.add_to_scene(r.mob).ok();
                r.shown = true;
            }
            let target = thick(m) / r.full;
            if target != r.factor {
                s.stretch_about(r.mob, target / r.factor, r.dim, Some(r.center), None);
                r.factor = target;
            }
        }
    });
    stage.play_prepared_with(
        vec![Box::new(grow)],
        PlayOverrides {
            run_time: Some(run_time),
            rate_func: Some(RateFunc::linear()),
            ..PlayOverrides::default()
        },
    )?;
    stage.remove_from_scene(anchor);
    Ok(mobs)
}

/// The lattice's lines for `steps` steps, placed in `l`'s square at the
/// fractions `at` (so 4-step lines can sit where 7-step lines will be).
fn grid_lines(l: &Lattice, at: &[f64]) -> fmn::Result<Vec<VMobject>> {
    let n = l.steps as f64;
    let mut out = Vec::new();
    for (idx, &f) in at.iter().enumerate() {
        let edge = idx == 0 || idx == at.len() - 1;
        let (w, c) = if edge {
            (2.5, Palette::dim())
        } else {
            (1.5, Palette::faint())
        };
        out.push(Line::new(l.p(f, 0.0), l.p(f, n)).style(line_style(c, w)).build()?);
        out.push(Line::new(l.p(0.0, f), l.p(n, f)).style(line_style(c, w)).build()?);
    }
    Ok(out)
}

/// Returns the square's mobjects, which 05_8 clears as it begins.
fn every_order(kit: &Kit, stage: &mut Stage<'_>, s5: &Stage05) -> fmn::Result<Vec<Mob>> {
    let line = say(kit, stage, "05_7")?;
    let mut out: Vec<Mob> = s5.bakery.all();
    out.extend(s5.room);
    out.extend(s5.flag_mobs());
    out.extend(&s5.marks);
    out.retain(|m| stage.scene().mobjects().contains(m));
    fade_out_all(stage, &out, 0.5)?;
    // without flags: 28 orders on the 4×4 lattice, drawn in the 7×7's frame
    let plain = bakery_orders(false, Cmp::Lex);
    let flagged = bakery_orders(true, Cmp::Lex);
    let n4 = bakery_program(2, 0, false, Cmp::Lex).len();
    let n7 = bakery_program(2, 0, true, Cmp::Lex).len();
    // where each no-flag step count sits among the flag program's: the flag
    // steps are slotted in (raise before the read, lower and wait after the
    // write)
    let embed: [usize; 5] = [0, 2, 3, 6, 7];
    if embed.len() != n4 + 1 || embed[n4] != n7 {
        return Err(fail(format!("embedding {embed:?} does not map {n4} steps into {n7}")));
    }
    let l7 = SQUARE_05;
    let l4 = Lattice { steps: n4, ..SQUARE_05 };
    let uniform: Vec<f64> = (0..=n4).map(|i| i as f64).collect();
    let lines4: Vec<Mob> = grid_lines(&l4, &uniform)?
        .into_iter()
        .map(|m| stage.arena_mut().add(m))
        .collect();
    let red4 = red_point(n4);
    let mark = stage.arena_mut().add(l4.mark(red4.0, red4.1, room_red()));
    let [start, end] = l4.ends(FAN_4);
    let ends4 = [stage.arena_mut().add(start), stage.arena_mut().add(end)];
    let mut ins = lines4.clone();
    ins.push(mark);
    ins.extend(ends4);
    fade_in_all(stage, &ins, 0.5)?;
    let hot = Style::default().fill(room_red(), 0.0).stroke(room_red(), 3.5, 1.0);
    let calm = Style::default().fill(customer(1), 0.0).stroke(customer(1), 2.5, 0.85);
    let (bad, good): (Vec<_>, Vec<_>) = plain.iter().enumerate().partition(|(_, o)| o.both_inside);
    let mut strands = Vec::new();
    let mut grow: Vec<Box<dyn Animation>> = Vec::new();
    for &(r, o) in good.iter().chain(&bad) {
        let style = if o.both_inside { hot } else { calm };
        let m = stage
            .arena_mut()
            .add(l4.staircase(o, fan(r, plain.len(), FAN_4), style));
        grow.push(Box::new(show_creation(m)));
        strands.push(m);
    }
    let g_grow = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, 0.06)?.with_run_time(2.0);
    stage.play(g_grow)?;
    stage.scene_mut().bring_to_front(&ends4)?;
    let panel_x = 1.6;
    let head_a = kit.text_left(stage, g::WITHOUT_FLAGS, 30.0, Palette::ink(), v(panel_x, 1.75))?;
    let count_a = kit.text_left(
        stage,
        &format!("{} {}", thousands(plain.len() as u64), g::ORDERS),
        36.0,
        customer(1),
        v(panel_x, 1.2),
    )?;
    let bad_a = kit.text_left(
        stage,
        &format!("{} {}", bad.len(), g::PUT_BOTH_IN),
        30.0,
        room_red(),
        v(panel_x, 0.68),
    )?;
    play!(stage, 0.5; write(stage.arena(), head_a), write(stage.arena(), count_a), write(stage.arena(), bad_a));
    // with flags: the lattice regrows to 7×7 and the 322 arrive as ribbons
    line.wait_for(stage, "With flags")?;
    let mut out_a: Vec<Box<dyn Animation>> = Vec::new();
    for &m in strands.iter().chain(&ends4) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out_a.push(stage.prepare(a.rt(0.5))?);
    }
    stage.play_prepared(out_a)?;
    // the four steps slide to their places among the seven; the three flag
    // steps' lines appear; the both-inside point moves with them
    let mut morph: Vec<Box<dyn Animation>> = Vec::new();
    let targets = grid_lines(&l7, &embed.iter().map(|&e| e as f64).collect::<Vec<_>>())?;
    for (&m, target) in lines4.iter().zip(targets) {
        let t = stage.arena_mut().add(target);
        morph.push(Box::new(Transform::new(m, t).rt(1.0)));
    }
    let red7 = red_point(n7);
    let target = stage.arena_mut().add(l7.mark(red7.0, red7.1, room_red()));
    morph.push(Box::new(Transform::new(mark, target).rt(1.0)));
    let new_lines: Vec<Mob> = grid_lines(
        &l7,
        &(0..=n7)
            .filter(|i| !embed.contains(i))
            .map(|i| i as f64)
            .collect::<Vec<_>>(),
    )?
    .into_iter()
    .map(|m| stage.arena_mut().add(m))
    .collect();
    for &m in &new_lines {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        morph.push(stage.prepare(a.rt(1.0))?);
    }
    stage.play_prepared(morph)?;
    let head_b = kit.text_left(stage, g::WITH_FLAGS, 30.0, Palette::ink(), v(panel_x, -0.55))?;
    play!(stage; write(stage.arena(), head_b).rt(0.4));
    let tracker = stage.add_value_tracker(0.0);
    let live = kit.live_count(stage, tracker, 36.0, flag_amber(), v(panel_x + 0.45, -1.1))?;
    let orders_word = kit.text_left(stage, g::ORDERS, 36.0, flag_amber(), v(panel_x + 1.2, -1.1))?;
    play!(stage; fade_in(stage.arena_mut(), orders_word, ORIGIN, 1.0)?.rt(0.3));
    let ribbons = ribbons(stage, l7, &flagged, 0.3, 1.5, tracker)?;
    stage.set_tracker_value(tracker, flagged.len() as f64)?;
    let none = flagged.iter().filter(|o| o.both_inside).count();
    let bad_b = kit.text_left(
        stage,
        &format!("{none} {}", g::PUT_BOTH_IN),
        30.0,
        Palette::ink(),
        v(panel_x, -1.62),
    )?;
    play!(stage; write(stage.arena(), bad_b).rt(0.5));
    // the both-inside point stays empty: it pulses on top
    stage.scene_mut().bring_to_front(&[mark])?;
    let pulse = fmn::animation::indicate(stage.arena_mut(), mark, 1.4, None)?;
    // the note fades in while the point pulses
    let note = precisely(kit, stage, g::ONE_VISIT_EACH)?;
    let note_in = fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?;
    play!(stage; pulse.rt(0.5), note_in.rt(0.4));
    kit.hold(stage)?;
    let mut out: Vec<Mob> = lines4;
    out.extend(new_lines);
    out.extend(ribbons);
    out.extend([mark, head_a, count_a, bad_a, head_b, live, orders_word, bad_b, note]);
    out.retain(|m| stage.scene().mobjects().contains(m));
    Ok(out)
}

// --------------------------------------------------------------- 05_8–05_9

/// 05_8–05_9: choosing time is not a lock; choosing at once is fine.
fn choosing_time(kit: &Kit, stage: &mut Stage<'_>, square: &[Mob]) -> fmn::Result<()> {
    let line = say(kit, stage, "05_8")?;
    fade_out_all(stage, square, 0.5)?;
    // Blue's and Green's choosing in trace a′, shaded on their lanes: the
    // steps from raising the flag to lowering it
    let steps = Trace::APrime.steps();
    let counted: Vec<&Step> = steps.iter().filter(|s| s.counted()).collect();
    let (bakery, room) = set_bakery(kit, stage, &[0; 5])?;
    let mut flags = Vec::new();
    for k in 1..=5 {
        flags.push(flag(stage, flag_foot(k), 0.62, false)?);
    }
    let mut ins = bakery.all();
    ins.extend(room);
    ins.extend(flags.iter().flat_map(|f| [f.pole, f.pennant]));
    fade_in_all(stage, &ins, 0.6)?;
    let mut bands = Vec::new();
    for k in [1usize, 2] {
        let up = counted
            .iter()
            .position(|s| s.customer == k && s.label.ends_with("choosing:=1"));
        let down = counted
            .iter()
            .position(|s| s.customer == k && s.label.ends_with("choosing:=0"));
        let (Some(a), Some(b)) = (up, down) else {
            return Err(fail(format!("customer {k} never chooses in trace a′")));
        };
        let band = Rectangle::new()
            .width(slot_x(b) - slot_x(a))
            .height(0.3)
            .corner_radius(0.08)
            .style(super::draw::flat(flag_amber(), 0.14))
            .build()?
            .moved_to(v((slot_x(a) + slot_x(b)) / 2.0, LANE_Y[k - 1]));
        bands.push(stage.arena_mut().add(band));
    }
    line.wait_for(stage, "choosing time")?;
    fade_in_all(stage, &bands, 0.6)?;
    let label = kit.text(
        stage,
        g::CHOOSING_TIME,
        26.0,
        flag_amber(),
        v(slot_x(3), LANE_Y[0] + 0.42),
    )?;
    let doorway = kit.text(stage, g::DOORWAY, 22.0, Palette::dim(), v(slot_x(3), LANE_Y[1] - 0.42))?;
    fade_in_all(stage, &[label, doorway], 0.5)?;
    // the door sign returns beside the flag: a lock; the flag is not one
    line.wait_for(stage, "Unlike the door sign")?;
    let door = sign(kit, stage, true, v(2.0, 3.05), 1.0, false)?;
    let lock = kit.text_left(stage, g::A_LOCK, 24.0, room_red(), v(3.15, 3.05))?;
    let mut ins = door.parts().to_vec();
    ins.push(lock);
    fade_in_all(stage, &ins, 0.5)?;
    // many flags up at once (C-X6: all flags up is reachable)
    line.wait_for(stage, "many can be up")?;
    let ups: Vec<Box<dyn Animation>> = flags
        .iter_mut()
        .take(3)
        .map(|f| Box::new(f.set(stage, true).rt(0.5)) as Box<dyn Animation>)
        .collect();
    stage.play_prepared(ups)?;
    kit.hold(stage)?;

    // 05_9: two customers choosing at once take equal tickets (trace c);
    // the pair settles it
    let line = say(kit, stage, "05_9")?;
    let mut out = vec![label, doorway, lock];
    out.extend(door.parts());
    out.extend(bands);
    let t = flags[2].set(stage, false);
    stage.play(t.rt(0.4))?;
    fade_out_all(stage, &out, 0.4)?;
    let c = Trace::C.steps();
    let tickets = c
        .last()
        .map(|s| s.after.number.clone())
        .ok_or_else(|| fail("trace c has no steps".into()))?;
    line.wait_for(stage, "The flag just shows")?;
    let mut bakery = bakery;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for k in [1usize, 2] {
        let [a, b] = bakery.slates[k - 1].rewrite(kit, stage, tickets[k - 1])?;
        anims.push(a);
        anims.push(b);
    }
    stage.play_prepared(anims)?;
    let thesis = kit.text(stage, g::THESIS, 28.0, Palette::ink(), v(0.0, 3.35))?;
    play!(stage; fade_in(stage.arena_mut(), thesis, v(0.0, 0.1), 1.0)?.rt(0.5));
    line.wait_for(stage, "the tie-breaker")?;
    let card = Rectangle::new()
        .width(5.2)
        .height(1.9)
        .corner_radius(0.16)
        .style(
            Style::default()
                .fill(explainer_kit::hex("#1A2130"), 0.96)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(v(1.7, -0.55));
    let card = stage.arena_mut().add(card);
    let first = pair_token(kit, stage, tickets[0], 1, v(0.4, -0.75), 1.2)?;
    let second = pair_token(kit, stage, tickets[1], 2, v(3.0, -0.75), 1.2)?;
    let before = kit.text(stage, g::COMES_BEFORE, 26.0, order_gold(), v(1.7, 0.05))?;
    play!(stage; fade_in(stage.arena_mut(), card, ORIGIN, 1.0)?.rt(0.4));
    play!(stage;
        fade_in(stage.arena_mut(), first, v(0.0, 0.2), 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), second, v(0.0, 0.2), 1.0)?.rt(0.5),
        write(stage.arena(), before).rt(0.6),
    );
    Ok(())
}
