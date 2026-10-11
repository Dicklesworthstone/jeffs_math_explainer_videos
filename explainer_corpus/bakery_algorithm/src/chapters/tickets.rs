//! Chapter 03, take a number: Lamport's bakery (03_1), why a shared ticket
//! machine would need a room of its own (03_2), a slate per customer
//! (03_3), Blue taking a ticket among the slates 0, 3, 0, 0, 2 and the room
//! serving Pink, Green, then Blue (03_4–03_5; replayed on the model,
//! oracle C-F7.model.customer1_*), one tune at a time (03_6), and nobody
//! writing on anyone else's slate (03_7).

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    Figure, Slate, Spoken, chalk, chalk_number, customer, fade_in_all, fade_out_all, fail, figure, flat, lane,
    line_style, order_gold, room, room_red, say, slate, wait_until,
};
use super::glyphs as g;
use crate::facts::{cited, take_ticket};
use crate::model::{FIVE, SLATES, blue_takes_a_ticket, service_orders};

pub struct TakeANumber {
    pub kit: Kit,
}

/// The five lanes, customer 1 (Blue) on top, evenly spaced: a stave.
pub const LANE_Y: [f64; 5] = [1.9, 0.95, 0.0, -0.95, -1.9];
pub const FIG_X: f64 = -6.25;
pub const SLATE_X: f64 = -5.05;
pub const SLATE_SIZE: f64 = 0.66;
pub const FIG_SCALE: f64 = 0.98;
pub const LANE_X0: f64 = -4.45;
pub const LANE_X1: f64 = 3.7;
/// The room, on the right, the height of the stave.
pub const ROOM_C: Vec3 = [5.35, 0.0, 0.0];
pub const ROOM_W: f64 = 2.6;
pub const ROOM_H: f64 = 4.8;
/// Where Blue's four glances land on her lane, and her ticket's sum.
const READ_X: [f64; 4] = [-3.3, -2.35, -1.4, -0.45];
const SUM_X: f64 = 0.75;
/// The room's turns on the lanes (03_5–03_6): one stretch per customer
/// served, one after another along time.
const TURN_X: [(f64, f64); 3] = [(1.0, 1.85), (1.9, 2.75), (2.8, 3.65)];

impl SceneConstruct for TakeANumber {
    fn name(&self) -> &str {
        "03_take_a_number"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let mut b = bakery(&kit, stage)?;
        machine(&kit, stage)?;
        slates(&kit, stage, &mut b)?;
        let (reads, line) = take_a_ticket(&kit, stage, &mut b)?;
        let turns = served(&kit, stage, &mut b, &line)?;
        tunes(&kit, stage, &reads, &turns)?;
        own_slates(&kit, stage, &b, &reads)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// The bakery on screen: five customers, their lanes and slates, the room.
pub struct Bakery {
    pub figures: Vec<Figure>,
    pub lanes: Vec<Mob>,
    pub slates: Vec<Slate>,
}

pub fn figure_at(k: usize) -> Vec3 {
    v(FIG_X, LANE_Y[k - 1])
}

/// The bakery as 03 left it, built at once and detached (04 and 05 start
/// from it): the five customers with their badges, slates showing
/// `slates`, each slate edged in its owner's colour, the lanes and the
/// room. Returns the bakery and the room's three parts.
pub fn set_bakery(kit: &Kit, stage: &mut Stage<'_>, slates: &[u32]) -> fmn::Result<(Bakery, [Mob; 3])> {
    let mut b = Bakery {
        figures: Vec::new(),
        lanes: Vec::new(),
        slates: Vec::new(),
    };
    for (i, &t) in slates.iter().enumerate() {
        let k = i + 1;
        b.figures.push(figure(kit, stage, k, figure_at(k), FIG_SCALE)?);
        let s = slate(kit, stage, t, v(SLATE_X, LANE_Y[i]), SLATE_SIZE)?;
        stage.set_stroke(s.board, Some(customer(k)), Some(3.0), Some(1.0), None, true);
        b.slates.push(s);
        b.lanes
            .push(stage.arena_mut().add(lane(LANE_X0, LANE_X1, LANE_Y[i], k)?));
    }
    let [tint, edge] = room(ROOM_W, ROOM_H, ROOM_C)?;
    let tint = stage.arena_mut().add(tint);
    let edge = stage.arena_mut().add(edge);
    let label = kit.text(
        stage,
        g::THE_ROOM,
        30.0,
        room_red(),
        v(ROOM_C[0], ROOM_C[1] + ROOM_H / 2.0 + 0.32),
    )?;
    Ok((b, [tint, edge, label]))
}

impl Bakery {
    /// Every mobject of the bakery (for fades).
    pub fn all(&self) -> Vec<Mob> {
        let mut out: Vec<Mob> = self.figures.iter().map(|f| f.group).collect();
        out.extend(&self.lanes);
        out.extend(self.slates.iter().flat_map(Slate::parts));
        out
    }
}

// --------------------------------------------------------------------- 03_1

fn bakery(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Bakery> {
    let line = say(kit, stage, "03_1")?;
    let dated = kit.text(
        stage,
        &format!("{}, {}", g::LAMPORT, cited::BAKERY_YEAR),
        40.0,
        Palette::ink(),
        v(0.0, 3.3),
    )?;
    play!(stage; write(stage.arena(), dated).rt(0.9));
    // Blue and Green, as in 01–02, then three more customers
    let mut figures = Vec::new();
    for k in 1..=5 {
        let f = figure(kit, stage, k, figure_at(k), FIG_SCALE)?;
        stage.set_opacity(f.badge, 0.0, true);
        figures.push(f);
    }
    fade_in_all(stage, &[figures[0].group, figures[1].group], 0.6)?;
    line.wait_for(stage, "picture a bakery")?;
    let [tint, edge] = room(ROOM_W, ROOM_H, ROOM_C)?;
    let tint = stage.arena_mut().add(tint);
    let edge = stage.arena_mut().add(edge);
    let label = kit.text(
        stage,
        g::THE_ROOM,
        30.0,
        room_red(),
        v(ROOM_C[0], ROOM_C[1] + ROOM_H / 2.0 + 0.32),
    )?;
    let joining: Vec<Mob> = figures[2..].iter().map(|f| f.group).collect();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for m in [tint, edge, label] {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.8))?);
    }
    for (i, &m) in joining.iter().enumerate() {
        let a = fade_in(stage.arena_mut(), m, v(-0.4, 0.0), 1.0)?;
        anims.push(stage.prepare(a.rt(0.6 + 0.2 * i as f64))?);
    }
    stage.play_prepared(anims)?;
    // "Take a number": the first chalk tickets beside the counter, taken
    // one after another from an empty bakery (1 + the biggest so far)
    line.wait_for(stage, "Take a number")?;
    let mut held = [0u32; 5];
    let mut tickets = Vec::new();
    for (i, x) in [-0.62, 0.0, 0.62].into_iter().enumerate() {
        held[i] = take_ticket(&held, i + 1);
        tickets.push(ticket(kit, stage, held[i], v(ROOM_C[0] + x, ROOM_C[1] - 1.55), false)?);
    }
    let parts: Vec<Mob> = tickets.iter().flatten().copied().collect();
    fade_in_all(stage, &parts, 0.5)?;
    line.wait_for(stage, "the lowest ticket")?;
    let lowest = tickets[0];
    play!(stage;
        lowest[0].animate().set_anim_args(AnimateArgs { run_time: Some(0.7), ..AnimateArgs::default() })?
            .set_stroke(Some(order_gold()), Some(4.0), Some(1.0))?,
        lowest[1].animate().set_anim_args(AnimateArgs { run_time: Some(0.7), ..AnimateArgs::default() })?
            .set_color(order_gold())?,
    );
    // the badges: each customer's number, with a roll call beside her
    // (R3 maya-08: the numbers are named once, on screen)
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    let mut roll = Vec::new();
    for f in &figures {
        let a = f
            .badge
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        anims.push(stage.prepare(a)?);
        let name = format!("{} {}", super::draw::NAMES[f.k - 1], f.k);
        let tag = kit.text_left(stage, &name, 28.0, customer(f.k), v(FIG_X + 0.5, LANE_Y[f.k - 1]))?;
        let t = fade_in(stage.arena_mut(), tag, v(-0.15, 0.0), 1.0)?;
        anims.push(stage.prepare(t.rt(0.6))?);
        roll.push(tag);
    }
    let g_roll = fmn::animation::lagged_start(stage.arena_mut(), anims)?;
    stage.play(g_roll)?;
    line.wait_for(stage, "From now on")?;
    fade_out_all(stage, &roll, 0.4)?;
    let names = lower_third(kit, stage, g::NAMES, g::PROCESS)?;
    play!(stage;
        fade_in(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.5),
        fade_out(stage.arena_mut(), dated, ORIGIN, 1.0)?.rt(0.5),
    );
    line.wait_for(stage, "going into the room")?;
    play!(stage; fmn::animation::indicate(stage.arena_mut(), label, 1.15, None)?.rt(0.8));
    kit.hold(stage)?;
    let mut out = parts;
    out.push(names);
    fade_out_all(stage, &out, 0.4)?;
    Ok(Bakery {
        figures,
        lanes: Vec::new(),
        slates: Vec::new(),
    })
}

/// A chalk ticket: an outlined stub with its number. Returns [outline, number].
pub fn ticket(kit: &Kit, stage: &mut Stage<'_>, value: u32, at: Vec3, gold: bool) -> fmn::Result<[Mob; 2]> {
    let c = if gold { order_gold() } else { chalk() };
    let stub = Rectangle::new()
        .width(0.52)
        .height(0.36)
        .corner_radius(0.05)
        .style(Style::default().fill(super::draw::board(), 1.0).stroke(c, 2.5, 1.0))
        .build()?
        .moved_to(at);
    let stub = stage.arena_mut().add(stub);
    let num = kit.text(stage, &value.to_string(), 24.0, c, at)?;
    Ok([stub, num])
}

// --------------------------------------------------------------------- 03_2

/// A ticket machine: a grey box with a slot and a ticket in it.
fn machine_shape(at: Vec3) -> fmn::Result<VMobject> {
    let body = Rectangle::new()
        .width(1.3)
        .height(1.7)
        .corner_radius(0.16)
        .style(flat(super::draw::hardware(), 1.0))
        .build()?
        .moved_to(at);
    let slot = Rectangle::new()
        .width(0.8)
        .height(0.09)
        .style(flat(super::draw::board(), 1.0))
        .build()?
        .moved_to(v(at[0], at[1] + 0.25));
    let tongue = Rectangle::new()
        .width(0.55)
        .height(0.32)
        .corner_radius(0.04)
        .style(
            Style::default()
                .fill(super::draw::board(), 1.0)
                .stroke(chalk(), 2.0, 1.0),
        )
        .build()?
        .moved_to(v(at[0], at[1] + 0.5));
    let button = Circle::new()
        .radius(0.14)
        .arc_center(v(at[0], at[1] - 0.4))
        .style(flat(room_red(), 1.0))
        .build();
    Ok(v_group([body, slot, tongue, button]))
}

fn machine(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let line = say(kit, stage, "03_2")?;
    let at = v(-0.2, 0.15);
    let m = stage.arena_mut().add(machine_shape(at)?);
    let label = kit.text(stage, g::TICKET_MACHINE, 26.0, Palette::dim(), v(at[0], at[1] - 1.2))?;
    fade_in_all(stage, &[m, label], 0.6)?;
    // two customers pressing at once: both get the same ticket
    line.wait_for(stage, "Two customers pressing")?;
    let mut presses = Vec::new();
    for k in [3, 4] {
        let from = v(FIG_X + 0.45, LANE_Y[k - 1]);
        let to = v(at[0] - 0.7, at[1] - 0.4 + 0.2 * (k as f64 - 3.5));
        let a = super::draw::access_arrow(from, to, k)?;
        presses.push(stage.arena_mut().add(a));
    }
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), presses[0])?.rt(0.6),
        fmn::animation::grow_arrow(stage.arena_mut(), presses[1])?.rt(0.6),
    );
    line.wait_for(stage, "could get the same")?;
    // the next ticket after 1, 2, 3, given twice
    let next = take_ticket(&[1, 2, 3, 0, 0], 4);
    let a = ticket(kit, stage, next, v(at[0] + 1.25, at[1] + 0.35), false)?;
    let b = ticket(kit, stage, next, v(at[0] + 1.25, at[1] - 0.15), false)?;
    fade_in_all(stage, &[a[0], a[1], b[0], b[1]], 0.5)?;
    line.wait_for(stage, "It would need")?;
    let cross = fmn::library::matchers::exmark(room_red())
        .scaled_about(1.6 / 0.3, ORIGIN)
        .moved_to(at);
    let cross = stage.arena_mut().add(cross);
    let own = Rectangle::new()
        .width(2.0)
        .height(2.3)
        .corner_radius(0.2)
        .style(Style::default().fill(room_red(), 0.1).stroke(room_red(), 2.5, 0.8))
        .build()?
        .moved_to(at);
    let own = stage.arena_mut().add(own);
    play!(stage;
        fade_in(stage.arena_mut(), own, ORIGIN, 1.1)?.rt(0.6),
        fmn::animation::grow_from_center(stage.arena_mut(), cross, None)?.rt(0.6),
    );
    kit.hold(stage)?;
    let mut out = vec![m, label, cross, own, a[0], a[1], b[0], b[1]];
    out.extend(presses);
    fade_out_all(stage, &out, 0.5)?;
    Ok(())
}

// --------------------------------------------------------------------- 03_3

fn slates(kit: &Kit, stage: &mut Stage<'_>, b: &mut Bakery) -> fmn::Result<()> {
    let line = say(kit, stage, "03_3")?;
    for k in 1..=5 {
        b.slates
            .push(slate(kit, stage, 0, v(SLATE_X, LANE_Y[k - 1]), SLATE_SIZE)?);
        b.lanes
            .push(stage.arena_mut().add(lane(LANE_X0, LANE_X1, LANE_Y[k - 1], k)?));
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (i, s) in b.slates.iter().enumerate() {
        for m in s.parts() {
            let a = fade_in(stage.arena_mut(), m, v(-0.2, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.5 + 0.1 * i as f64))?);
        }
    }
    stage.play_prepared(anims)?;
    line.wait_for(stage, "only she writes")?;
    // each customer chalks her own slate: the slate's edge takes her colour
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (i, s) in b.slates.iter().enumerate() {
        let a = s
            .board
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(customer(i + 1)), Some(3.0), Some(1.0))?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    line.wait_for(stage, "A zero means")?;
    let lanes: Vec<Box<dyn Animation>> = b
        .lanes
        .iter()
        .map(|&m| Box::new(show_creation(m)) as Box<dyn Animation>)
        .collect();
    let g_lanes = AnimationGroup::with_lag_ratio(stage.arena_mut(), lanes, 0.1)?.with_run_time(1.0);
    stage.play(g_lanes)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for s in &b.slates {
        let a = fmn::animation::indicate(stage.arena_mut(), s.value, 1.3, None)?;
        anims.push(stage.prepare(a.rt(0.7))?);
    }
    stage.play_prepared(anims)?;
    kit.hold(stage)?;
    Ok(())
}

// --------------------------------------------------------------- 03_4–03_5

/// Blue's four glances, kept for 03_7.
struct Reads {
    arrows: Vec<Mob>,
    chips: Vec<Mob>,
    sum: Vec<Mob>,
}

fn take_a_ticket(kit: &Kit, stage: &mut Stage<'_>, b: &mut Bakery) -> fmn::Result<(Reads, Spoken)> {
    let line = say(kit, stage, "03_4")?;
    // Green and Pink are already in line, with 3 and 2
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &t) in SLATES.iter().enumerate() {
        if t > 0 {
            let [a, w] = b.slates[i].rewrite(kit, stage, t)?;
            anims.push(a);
            anims.push(w);
        }
    }
    stage.play_prepared(anims)?;
    // Blue's doorway, replayed on the model: her reads, then her write
    let steps = blue_takes_a_ticket();
    let reads: Vec<(usize, u32)> = steps
        .iter()
        .filter_map(|s| {
            let rest = s.label.strip_prefix("read number[")?;
            let (who, value) = rest.split_once("]=")?;
            Some((who.parse().ok()?, value.parse().ok()?))
        })
        .collect();
    if reads.len() != READ_X.len() {
        return Err(fail(format!("Blue's doorway reads {} slates", reads.len())));
    }
    line.wait_for(stage, "Blue reads")?;
    let blue_y = LANE_Y[0];
    let mut arrows = Vec::new();
    let mut chips = Vec::new();
    let span = (line.at("What does")? - line.at("Blue reads")?).max(1.6);
    for (j, &(who, value)) in reads.iter().enumerate() {
        let from = v(SLATE_X + 0.7 * SLATE_SIZE + 0.05, LANE_Y[who - 1]);
        let to = v(READ_X[j], blue_y - 0.12);
        let arrow = Arrow::new(from, to)
            .buff(0.06)
            .thickness(2.5)
            .style(Style::default().fill(customer(1), 0.8).stroke(customer(1), 2.5, 0.8))
            .build()?;
        let arrow = stage.arena_mut().add(arrow);
        let chip = chalk_number(kit, stage, value, v(READ_X[j], blue_y + 0.32), 0.5)?;
        play!(stage;
            fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(0.45),
            fade_in(stage.arena_mut(), chip, v(0.0, -0.1), 1.0)?.rt(0.45),
        );
        arrows.push(arrow);
        chips.push(chip);
        stage.wait((span / reads.len() as f64 - 0.45).max(0.05))?;
    }
    // the biggest of what she read, boxed in gold
    let biggest = reads.iter().map(|r| r.1).max().unwrap_or(0);
    let at = reads.iter().position(|r| r.1 == biggest).unwrap_or(0);
    let gold = Rectangle::new()
        .width(0.5)
        .height(0.46)
        .corner_radius(0.06)
        .style(super::draw::line_style(order_gold(), 2.5))
        .build()?
        .moved_to(v(READ_X[at], blue_y + 0.32));
    let gold = stage.arena_mut().add(gold);
    play!(stage; fmn::animation::grow_from_center(stage.arena_mut(), gold, None)?.rt(0.4));
    let q = predict_pause(kit, stage, v(SUM_X, blue_y + 0.32))?;

    // 03_5: four; then she waits for every smaller ticket
    let line = say(kit, stage, "03_5")?;
    let written = steps
        .iter()
        .find_map(|s| s.ticket_written())
        .ok_or_else(|| fail("Blue's doorway writes no ticket".into()))?;
    let sum = kit.text(
        stage,
        &format!("{biggest} + 1 = {}", written.1),
        30.0,
        chalk(),
        v(SUM_X, blue_y + 0.32),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
        write(stage.arena(), sum).rt(0.6),
    );
    let [a, w] = b.slates[0].rewrite(kit, stage, written.1)?;
    stage.play_prepared(vec![a, w])?;
    let reads = Reads {
        arrows,
        chips,
        sum: vec![gold, sum],
    };
    Ok((reads, line))
}

/// 03_5's served turns: who went into the room, in order, and the lane
/// stretch that marks each turn.
struct Turns {
    order: Vec<usize>,
}

fn served(kit: &Kit, stage: &mut Stage<'_>, b: &mut Bakery, line: &Spoken) -> fmn::Result<Turns> {
    let st1 = blue_takes_a_ticket()
        .last()
        .map(|s| s.after.clone())
        .ok_or_else(|| fail("Blue's doorway has no steps".into()))?;
    let orders = service_orders(&FIVE, &st1, &[1, 2, 5]);
    let order = orders
        .iter()
        .next()
        .cloned()
        .ok_or_else(|| fail("no service order".into()))?;
    if orders.len() != 1 {
        return Err(fail(format!("{} service orders, the narration says one", orders.len())));
    }
    // "She waits until nobody with a smaller ticket is still in line"
    let phrases = ["Here that's Pink", "then Green", "then Blue"];
    line.wait_for(stage, "Then she waits")?;
    let wait = kit.text_left(
        stage,
        g::WAITS,
        24.0,
        Palette::dim(),
        v(SUM_X - 0.6, (LANE_Y[0] + LANE_Y[1]) / 2.0),
    )?;
    play!(stage; fade_in(stage.arena_mut(), wait, ORIGIN, 1.0)?.rt(0.4));
    kit.lint_here(stage, "03_5 waits");
    for (turn, &k) in order.iter().enumerate() {
        if let Some(p) = phrases.get(turn) {
            line.wait_for(stage, p)?;
        }
        let f = &b.figures[k - 1];
        let home = figure_at(k);
        let inside = v(ROOM_C[0], ROOM_C[1] + 0.9 - 0.9 * turn as f64);
        play!(stage; f.group.animate().set_anim_args(AnimateArgs { run_time: Some(0.6), ..AnimateArgs::default() })?
            .move_to(inside, ORIGIN)?);
        let (x0, x1) = TURN_X[turn];
        let mark = Line::new(v(x0, LANE_Y[k - 1]), v(x1, LANE_Y[k - 1]))
            .style(line_style(room_red(), 9.0))
            .build()?;
        let mark = stage.arena_mut().add(mark);
        play!(stage; show_creation(mark).rt(0.5));
        // out again: her slate wiped to zero, back to her place
        let [a, w] = b.slates[k - 1].rewrite(kit, stage, 0)?;
        let back = f
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .move_to(home, ORIGIN)?;
        let back = stage.prepare(back)?;
        stage.play_prepared(vec![a, w, back])?;
        if k == 1 {
            play!(stage; fade_out(stage.arena_mut(), wait, ORIGIN, 1.0)?.rt(0.3));
        }
    }
    kit.hold(stage)?;
    Ok(Turns { order })
}

// --------------------------------------------------------------------- 03_6

fn tunes(kit: &Kit, stage: &mut Stage<'_>, reads: &Reads, turns: &Turns) -> fmn::Result<()> {
    say(kit, stage, "03_6")?;
    // Blue's sums leave the stave to the tunes
    let mut done = reads.chips.clone();
    done.extend(&reads.sum);
    fade_out_all(stage, &done, 0.4)?;
    // the room's turns as one sound, each customer's "served" in her own
    // timbre; the lanes become a stave and each note lands as it sounds
    let (sound, onsets) = super::tunes::turns(&turns.order);
    let (wav, secs) = super::tunes::write("03_6", sound).map_err(|e| fail(e.to_string()))?;
    let degrees = super::tunes::served_degrees();
    let mut heads = Vec::new();
    for (turn, &k) in turns.order.iter().enumerate() {
        let (x0, x1) = TURN_X[turn];
        let y = LANE_Y[k - 1];
        for (n, &d) in degrees.iter().enumerate() {
            let x = x0 + (x1 - x0) * (n as f64 + 0.5) / degrees.len() as f64;
            let at = v(x, y + 0.09 * f64::from(d - 1));
            let head = Ellipse::new()
                .width(0.2)
                .height(0.15)
                .style(flat(customer(k), 1.0))
                .build()
                .moved_to(at);
            heads.push((onsets[turn][n], stage.arena_mut().add(head)));
        }
    }
    kit.sonify(stage, secs, "03_6", &wav)?;
    let t0 = stage.scene().time().to_f64();
    for (t, head) in heads {
        wait_until(stage, t0 + t)?;
        play!(stage; fade_in(stage.arena_mut(), head, v(0.0, 0.12), 0.6)?.rt(0.15));
    }
    wait_until(stage, t0 + secs)?;
    Ok(())
}

// --------------------------------------------------------------------- 03_7

fn own_slates(kit: &Kit, stage: &mut Stage<'_>, b: &Bakery, reads: &Reads) -> fmn::Result<()> {
    let line = say(kit, stage, "03_7")?;
    line.wait_for(stage, "Nobody ever writes")?;
    // every slate glows in its owner's colour; every arrow between lanes is
    // a glance (a read), none a write
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (i, s) in b.slates.iter().enumerate() {
        let a = fmn::animation::indicate(stage.arena_mut(), s.board, 1.12, Some(rgb(customer(i + 1))))?;
        anims.push(stage.prepare(a.rt(0.9))?);
    }
    for &m in &reads.arrows {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.0, None)?;
        anims.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(anims)?;
    let lower = lower_third(kit, stage, g::FOR_PROGRAMMERS, g::TICKET_LOCK)?;
    play!(stage; fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;
    kit.music_breath(stage, 2.0)?;
    Ok(())
}

fn rgb(c: Srgb) -> [f32; 3] {
    let h = c.to_hex();
    let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_or(0.0, |x| f32::from(x) / 255.0);
    [ch(1), ch(3), ch(5)]
}
