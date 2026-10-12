//! Chapter 11, fair at a price: first come, first served on the lanes, and
//! either order when choosing overlaps (11_1); once your ticket is written,
//! each other customer gets ahead of you at most once (11_2); tickets climb
//! on an odometer while the bakery never empties (11_3); Lamport's sum, a
//! year of milliseconds against 2^35, and the 32-bit slate rolling over
//! (11_4–11_5); 64 bits, and the assumption under the sum (11_6); the costs:
//! twelve reads alone, a frozen chooser stalling everyone, the costs card
//! (11_7). Every lane shows a run replayed on the model.

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    LaneLayout, LaneSpec, Slots, TraceLanes, board, chalk, customer, fade_in_all, fade_out_all, fail, group, hardware,
    line_style, play_timed, quote_card, room_red, say, static_burst,
};
use super::glyphs;
use crate::facts::{self, cited, fact, fact_u64};
use crate::model::{Bakery, State, Step, Trace, Variant, replay};

/// Two customers, one visit each, atomic slates (as 05's 322 orders).
const TWO: Bakery = Bakery::new(2, 5, Variant::Correct, false, false);
/// Three customers, atomic slates, tickets up to 6 (11_2, 11_7).
const THREE: Bakery = Bakery::new(3, 6, Variant::Correct, false, false);
/// Five customers alone at the counter (11_7's twelve reads).
const FIVE_ALONE: Bakery = Bakery::new(5, 6, Variant::Correct, false, false);

/// 11_1: Blue finishes choosing before Green starts; Blue is served first.
pub const DISJOINT: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (1, "read number[2]=0"),
    (1, "number[1]:=1"),
    (1, "choosing:=0"),
    (2, "choosing:=1"),
    (2, "read number[1]=1"),
    (2, "number[2]:=2"),
    (2, "choosing:=0"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=2, pass -> ENTER CS"),
    (1, "number[1]:=0 (exit)"),
    (2, "L3: number[1]=0, pass -> ENTER CS"),
];

/// 11_1: they choose at once; Blue started first, yet Green goes first.
pub const OVERLAP_GREEN_FIRST: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (2, "choosing:=1"),
    (2, "read number[1]=0"),
    (2, "number[2]:=1"),
    (1, "read number[2]=1"),
    (1, "number[1]:=2"),
    (2, "choosing:=0"),
    (1, "choosing:=0"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=2, pass -> ENTER CS"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, wait"),
    (2, "number[2]:=0 (exit)"),
    (1, "L3: number[2]=0, pass -> ENTER CS"),
];

/// 11_1: they choose at once, both take 1, and the tie goes to Blue.
pub const OVERLAP_BLUE_FIRST: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (2, "choosing:=1"),
    (1, "read number[2]=0"),
    (2, "read number[1]=0"),
    (1, "number[1]:=1"),
    (2, "number[2]:=1"),
    (1, "choosing:=0"),
    (2, "choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, pass -> ENTER CS"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
    (1, "number[1]:=0 (exit)"),
    (2, "L3: number[1]=0, pass -> ENTER CS"),
];

/// 11_2: Green and Purple hold 1 and 2 when Blue takes 3; each gets in
/// once before her, comes back with a bigger ticket, and Blue goes in.
pub const BYPASS: &[(usize, &str)] = &[
    (2, "choosing:=1"),
    (2, "read number[1]=0"),
    (2, "read number[3]=0"),
    (2, "number[2]:=1"),
    (2, "choosing:=0"),
    (3, "choosing:=1"),
    (3, "read number[1]=0"),
    (3, "read number[2]=1"),
    (3, "number[3]:=2"),
    (3, "choosing:=0"),
    (1, "choosing:=1"),
    (1, "read number[2]=1"),
    (1, "read number[3]=2"),
    (1, "number[1]:=3"),
    (1, "choosing:=0"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=3, pass"),
    (2, "L2: choosing[3]=0, pass"),
    (2, "L3: number[3]=2, pass -> ENTER CS"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=1, wait"),
    (2, "number[2]:=0 (exit)"),
    (2, "choosing:=1"),
    (2, "read number[1]=3"),
    (2, "read number[3]=2"),
    (2, "number[2]:=4"),
    (2, "choosing:=0"),
    (1, "L3: number[2]=4, pass"),
    (1, "L2: choosing[3]=0, pass"),
    (1, "L3: number[3]=2, wait"),
    (3, "L2: choosing[1]=0, pass"),
    (3, "L3: number[1]=3, pass"),
    (3, "L2: choosing[2]=0, pass"),
    (3, "L3: number[2]=4, pass -> ENTER CS"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=3, wait"),
    (3, "number[3]:=0 (exit)"),
    (3, "choosing:=1"),
    (3, "read number[1]=3"),
    (3, "read number[2]=4"),
    (3, "number[3]:=5"),
    (3, "choosing:=0"),
    (1, "L3: number[3]=5, pass -> ENTER CS"),
];

/// 11_3: the leapfrog (tickets 1 to 6) ends with Green in the room holding
/// 6; she leaves, the bakery is empty, and the next ticket is 1 again.
pub const EMPTIES: &[(usize, &str)] = &[(2, "number[2]:=0 (exit)")];

/// 11_7: Blue, alone among five, takes a ticket and goes in: three reads
/// of every other customer (the max, the flag, the slate).
pub fn alone() -> Vec<(usize, String)> {
    let mut s: Vec<(usize, String)> = vec![(1, "choosing:=1".into())];
    for j in 2..=5 {
        s.push((1, format!("read number[{j}]=0")));
    }
    s.push((1, "number[1]:=1".into()));
    s.push((1, "choosing:=0".into()));
    for j in 2..=5 {
        s.push((1, format!("L2: choosing[{j}]=0, pass")));
        s.push((1, format!("L3: number[{j}]=0, pass")));
    }
    s
}

/// 11_7: Blue goes in; Purple raises her flag and freezes; Green and Blue
/// each manage at most one more step forward, then stall behind her.
pub const FROZEN: &[(usize, &str)] = &[
    (1, "choosing:=1"),
    (1, "read number[2]=0"),
    (1, "read number[3]=0"),
    (1, "number[1]:=1"),
    (1, "choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=0, pass"),
    (1, "L2: choosing[3]=0, pass"),
    (1, "L3: number[3]=0, pass -> ENTER CS"),
    (3, "choosing:=1"),
    (2, "choosing:=1"),
    (2, "read number[1]=1"),
    (2, "read number[3]=0"),
    (2, "number[2]:=2"),
    (2, "choosing:=0"),
    (2, "L2: choosing[1]=0, pass"),
    (2, "L3: number[1]=1, wait"),
    (1, "number[1]:=0 (exit)"),
    (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=1, wait"),
    (1, "choosing:=1"),
    (1, "read number[2]=2"),
    (1, "read number[3]=0"),
    (1, "number[1]:=3"),
    (1, "choosing:=0"),
    (1, "L2: choosing[2]=0, pass"),
    (1, "L3: number[2]=2, wait"),
];

/// Replay a schedule from the model's start; an error if a step is not
/// exactly one enabled move (the tests replay every schedule).
pub fn run(model: &Bakery, schedule: &[(usize, &str)]) -> fmn::Result<Vec<Step>> {
    let s: Vec<(usize, String)> = schedule.iter().map(|&(p, l)| (p, l.to_owned())).collect();
    replay(model, model.initial(), &s).map_err(fail)
}

/// The first customer (1-based) to enter the room in `steps`.
pub fn first_in(steps: &[Step]) -> Option<usize> {
    steps.iter().find(|s| s.enters()).map(|s| s.customer)
}

// ---------------------------------------------------------------- odometer

/// An odometer: one wheel per digit, a value shown by sliding each changed
/// digit up and out while the new one slides in from below (no clipping:
/// each wheel is a small dark window, the digits fade at its edges).
struct Odometer {
    wheels: Vec<Mob>,
    digits: Vec<Mob>,
    value: u64,
    centers: Vec<Vec3>,
    size: f64,
}

impl Odometer {
    fn build(kit: &Kit, stage: &mut Stage<'_>, n: usize, value: u64, at: Vec3, size: f64) -> fmn::Result<Self> {
        let w = 0.62 * size;
        let mut wheels = Vec::new();
        let mut centers = Vec::new();
        for i in 0..n {
            let c = v(at[0] + (i as f64 - (n as f64 - 1.0) / 2.0) * (w + 0.06 * size), at[1]);
            let r = Rectangle::new()
                .width(w)
                .height(size)
                .corner_radius(0.08 * size)
                .style(Style::default().fill(board(), 1.0).stroke(hardware(), 2.0, 1.0))
                .build()?
                .moved_to(c);
            wheels.push(stage.arena_mut().add(r));
            centers.push(c);
        }
        let mut o = Self {
            wheels,
            digits: Vec::new(),
            value,
            centers,
            size,
        };
        let ds = o.digit_chars(value);
        for (i, ch) in ds.iter().enumerate() {
            let d = kit.text(stage, &ch.to_string(), 60.0 * size, chalk(), o.centers[i])?;
            o.digits.push(d);
        }
        Ok(o)
    }

    fn digit_chars(&self, value: u64) -> Vec<char> {
        format!("{value:0width$}", width = self.centers.len()).chars().collect()
    }

    fn parts(&self) -> Vec<Mob> {
        self.wheels.iter().chain(&self.digits).copied().collect()
    }

    /// Roll to `value`: every changed digit slides up and out, the new one
    /// in from below.
    fn roll_to(&mut self, kit: &Kit, stage: &mut Stage<'_>, value: u64, rt: f64) -> fmn::Result<()> {
        let old = self.digit_chars(self.value);
        let new = self.digit_chars(value);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let dy = 0.55 * self.size;
        for i in 0..new.len() {
            if old[i] == new[i] {
                continue;
            }
            let c = self.centers[i];
            let d = kit.text(
                stage,
                &new[i].to_string(),
                60.0 * self.size,
                chalk(),
                v(c[0], c[1] - dy),
            )?;
            let out = fade_out(stage.arena_mut(), self.digits[i], v(0.0, dy), 1.0)?;
            anims.push(stage.prepare(out.rt(rt))?);
            let inn = fade_in(stage.arena_mut(), d, v(0.0, -dy), 1.0)?;
            stage.move_to(d, c, ORIGIN);
            anims.push(stage.prepare(inn.rt(rt))?);
            self.digits[i] = d;
        }
        self.value = value;
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
    }
}

/// The biggest ticket held in `st`.
fn biggest(st: &State) -> u64 {
    u64::from(st.number.iter().copied().max().unwrap_or(0))
}

fn lanes(ys: &[(usize, f64)], x0: f64, x1: f64, slate: f64) -> LaneLayout {
    LaneLayout {
        head_x: x0 - 2.1,
        x0,
        x1,
        lanes: ys
            .iter()
            .map(|&(customer, y)| LaneSpec {
                customer,
                y,
                idle: false,
            })
            .collect(),
        slots: Slots::EveryStep,
        slate_size: slate,
    }
}

/// Reveal `steps` of `l` with the heads following, `rt` seconds a step.
fn walk(
    kit: &Kit,
    stage: &mut Stage<'_>,
    l: &mut TraceLanes,
    range: std::ops::RangeInclusive<usize>,
    rt: f64,
) -> fmn::Result<()> {
    for s in range {
        let mut all = Vec::new();
        if let Some(a) = l.reveal(stage, s..=s, rt)? {
            all.push(a);
        }
        all.extend(l.heads_to(kit, stage, s)?);
        play_timed(stage, all, rt)?;
    }
    Ok(())
}

/// Everything a lane diagram has on screen.
fn lane_mobs(l: &TraceLanes) -> Vec<Mob> {
    let mut m = l.base();
    m.extend(l.parts.iter().flat_map(|p| p.all()));
    m
}

/// A card: a title in the hero role and lines beneath, left-aligned.
fn card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    title: &str,
    lines: &[&str],
    left: f64,
    top: f64,
    w: f64,
) -> fmn::Result<Mob> {
    let line_h = 0.4;
    let h = 0.85 + line_h * lines.len() as f64;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(board(), 0.95).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(v(left + w / 2.0, top - h / 2.0));
    let mut parts = vec![stage.arena_mut().add(back)];
    parts.push(kit.text_left(stage, title, 22.0, Palette::dim(), v(left + 0.3, top - 0.36))?);
    for (i, l) in lines.iter().enumerate() {
        parts.push(kit.text_left(
            stage,
            l,
            22.0,
            Palette::ink(),
            v(left + 0.3, top - 0.86 - line_h * i as f64),
        )?);
    }
    group(stage, &parts)
}

pub struct FairAtAPrice {
    pub kit: Kit,
}

impl SceneConstruct for FairAtAPrice {
    fn name(&self) -> &str {
        "11_fair_at_a_price"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 11_1: first come, first served; overlapping choosing, either order
        let fcfs = facts::choosing_order(&crate::model::bakery_orders(true, crate::model::Cmp::Lex));
        let line = say(&kit, stage, "11_1")?;
        let steps = run(&TWO, DISJOINT)?;
        let mut d = TraceLanes::build(
            &kit,
            stage,
            lanes(&[(1, 1.15), (2, -0.95)], -3.6, 5.6, 0.5),
            &steps,
            &[],
        )?;
        let tag_a = kit.text(stage, glyphs::BLUE_FINISHES_FIRST, 26.0, customer(1), v(1.0, 2.35))?;
        fade_in_all(stage, &d.base(), 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), tag_a, ORIGIN, 1.0)?.rt(0.5));
        walk(&kit, stage, &mut d, 1..=steps.len(), 0.22)?;
        let served = kit.text(stage, glyphs::BLUE_SERVED_FIRST, 26.0, customer(1), v(1.0, -2.05))?;
        play!(stage; fade_in(stage.arena_mut(), served, v(0.0, 0.1), 1.0)?.rt(0.5));
        line.wait_for(stage, "If we choose at the same time")?;
        let mut old = lane_mobs(&d);
        old.extend([tag_a, served]);
        fade_out_all(stage, &old, 0.5)?;
        let g = run(&TWO, OVERLAP_GREEN_FIRST)?;
        let b = run(&TWO, OVERLAP_BLUE_FIRST)?;
        let mut top = TraceLanes::build(&kit, stage, lanes(&[(1, 2.3), (2, 0.95)], -3.6, 4.4, 0.42), &g, &[])?;
        let mut bot = TraceLanes::build(&kit, stage, lanes(&[(1, -0.35), (2, -1.7)], -3.6, 4.4, 0.42), &b, &[])?;
        let mut both = top.base();
        both.extend(bot.base());
        fade_in_all(stage, &both, 0.5)?;
        for (l, n) in [(&mut top, g.len()), (&mut bot, b.len())] {
            walk(&kit, stage, l, 1..=n, 0.12)?;
        }
        let first = |k: usize| format!("{} {}", super::draw::NAMES[k - 1], glyphs::FIRST);
        let (gk, bk) = (
            first_in(&g).ok_or_else(|| fail("someone enters".into()))?,
            first_in(&b).ok_or_else(|| fail("someone enters".into()))?,
        );
        let t1 = kit.text(stage, &first(gk), 26.0, customer(gk), v(5.75, 1.62))?;
        let t2 = kit.text(stage, &first(bk), 26.0, customer(bk), v(5.75, -1.02))?;
        let note = lower_third(
            &kit,
            stage,
            glyphs::PRECISELY,
            &format!(
                "{} {} {} {}{}",
                glyphs::FCFS_A,
                fcfs.disjoint,
                glyphs::FCFS_B,
                fcfs.overlap,
                glyphs::FCFS_C
            ),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), t1, v(-0.1, 0.0), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), t2, v(-0.1, 0.0), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;

        // 11_2: once Blue's ticket is written, Green and Purple each get
        // ahead of her once, come back behind her, and Blue goes in
        let line = say(&kit, stage, "11_2")?;
        let mut old = lane_mobs(&top);
        old.extend(lane_mobs(&bot));
        old.extend([t1, t2, note]);
        fade_out_all(stage, &old, 0.5)?;
        let steps = run(&THREE, BYPASS)?;
        let mut l = TraceLanes::build(
            &kit,
            stage,
            lanes(&[(1, 1.75), (2, 0.25), (3, -1.25)], -3.9, 6.3, 0.46),
            &steps,
            &[],
        )?;
        fade_in_all(stage, &l.base(), 0.5)?;
        let ticket = steps
            .iter()
            .position(|s| s.customer == 1 && s.ticket_written().is_some())
            .ok_or_else(|| fail("Blue writes a ticket".into()))?
            + 1;
        walk(&kit, stage, &mut l, 1..=ticket, 0.12)?;
        let tx = l
            .x(ticket)
            .ok_or_else(|| fail("Blue's ticket step has a slot".into()))?;
        let mark = stage.arena_mut().add(
            DashedLine::new(v(tx, 2.25), v(tx, -1.75))
                .dash_length(0.08)
                .style(line_style(customer(1), 2.0))
                .build()?,
        );
        let mark_t = kit.text(stage, glyphs::BLUES_TICKET_WRITTEN, 22.0, customer(1), v(tx, 2.55))?;
        play!(stage;
            show_creation(mark).rt(0.4),
            fade_in(stage.arena_mut(), mark_t, ORIGIN, 1.0)?.rt(0.4),
        );
        line.wait_for(stage, "Once you've taken your ticket")?;
        let mut ahead = Vec::new();
        for s in ticket + 1..=steps.len() {
            walk(&kit, stage, &mut l, s..=s, 0.17)?;
            let st = &steps[s - 1];
            if st.enters() && st.customer != 1 {
                let x = l.x(s).ok_or_else(|| fail("entry has a slot".into()))?;
                let y = l.y(st.customer).ok_or_else(|| fail("lane".into()))?;
                let once = kit.text(stage, glyphs::ONCE, 22.0, customer(st.customer), v(x + 0.15, y - 0.42))?;
                play!(stage; fade_in(stage.arena_mut(), once, v(0.0, 0.1), 1.0)?.rt(0.35));
                ahead.push(once);
            }
        }
        line.wait_for(stage, "So nobody waits forever")?;
        let tick = kit.text(
            stage,
            &format!("{} {}", glyphs::AHEAD_AT_MOST, facts::bypass_worst()),
            26.0,
            Palette::green(),
            v(1.2, -2.45),
        )?;
        play!(stage; fade_in(stage.arena_mut(), tick, v(0.0, 0.15), 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 11_3: the leapfrog: tickets climb while the bakery never empties;
        // they reset only when it does
        let line = say(&kit, stage, "11_3")?;
        let mut old = lane_mobs(&l);
        old.extend(ahead);
        old.extend([mark, mark_t, tick]);
        fade_out_all(stage, &old, 0.5)?;
        let leap = Trace::Leapfrog.steps();
        let last = leap.last().ok_or_else(|| fail("leapfrog steps".into()))?.after.clone();
        let extra: Vec<(usize, String)> = EMPTIES.iter().map(|&(p, s)| (p, s.to_owned())).collect();
        let tail = replay(&Trace::Leapfrog.model(), last, &extra).map_err(fail)?;
        let mut all_steps = leap.clone();
        all_steps.extend(tail);
        let mut lf = TraceLanes::build(
            &kit,
            stage,
            lanes(&[(1, 1.2), (2, -0.9)], -4.4, 3.2, 0.5),
            &all_steps,
            &[],
        )?;
        let mut odo = Odometer::build(&kit, stage, 1, 0, v(5.4, 0.15), 1.4)?;
        let odo_t = kit.text(stage, glyphs::BIGGEST_TICKET, 24.0, Palette::dim(), v(5.4, 1.4))?;
        let mut intro = lf.base();
        intro.extend(odo.parts());
        intro.push(odo_t);
        fade_in_all(stage, &intro, 0.5)?;
        let rt = (line.end() - stage.scene().time().to_f64() - 1.2).max(4.0) / all_steps.len() as f64;
        for s in 1..=all_steps.len() {
            walk(&kit, stage, &mut lf, s..=s, rt)?;
            let b = biggest(&all_steps[s - 1].after);
            if b != odo.value {
                odo.roll_to(&kit, stage, b, 0.3)?;
            }
        }
        kit.hold(stage)?;

        // 11_4: Lamport's sum
        say(&kit, stage, "11_4")?;
        let mut old = lane_mobs(&lf);
        old.extend(odo.parts());
        old.push(odo_t);
        fade_out_all(stage, &old, 0.5)?;
        let sum = kit.tex(
            stage,
            &format!(
                r"\text{{{}}} \times \text{{{}}}\ ({}\ \text{{{}}})",
                glyphs::ONE_TICKET_A_MS,
                glyphs::ONE_YEAR,
                fact("C-F10.days_per_year"),
                glyphs::DAYS
            ),
            44.0,
            Palette::ink(),
            &[],
            v(0.0, 2.2),
        )?;
        // the sum is written as Lamport "did the sum"; the question follows
        play!(stage; write(stage.arena(), sum).rt(1.4));
        let q = predict_pause(&kit, stage, v(0.0, 0.7))?;

        // 11_5: about 31.5 billion, under Lamport's 2^35; too big for 32 bits
        let line = say(&kit, stage, "11_5")?;
        // said "thirty-one and a half": shown as the same mixed number, not as a
        // decimal that reads as 31.5 against 31.56 (R3 ruth-02)
        let billions = fact("C-F10.ms_per_year_billions_to_nearest_half");
        let billions = match billions.strip_suffix(".5") {
            Some(whole) => format!(r"{whole}\tfrac{{1}}{{2}}"),
            None => billions.to_owned(),
        };
        let answer = kit.tex(
            stage,
            &format!(
                r"= {}\ \text{{{}}} \approx {}\ \text{{{}}}",
                tex_thousands(fact_u64("C-F10.ms_per_year")),
                glyphs::TICKETS,
                billions,
                glyphs::BILLION
            ),
            44.0,
            Palette::hero(),
            &[],
            v(0.0, 1.2),
        )?;
        let p35 = fact_u64("C-F10.two_pow_35");
        let bound = kit.tex(
            stage,
            &format!(
                r"< 2^{{{}}} = {}\quad\text{{{}}}",
                p35.trailing_zeros(),
                tex_thousands(p35),
                glyphs::LAMPORTS_BOUND
            ),
            36.0,
            Palette::ink(),
            &[],
            v(0.0, 0.35),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            write(stage.arena(), answer).rt(1.2),
        );
        play!(stage; fade_in(stage.arena_mut(), bound, v(0.0, 0.1), 1.0)?.rt(0.7));
        line.wait_for(stage, "A thirty-two bit slate")?;
        // a 32-bit slate's top, its digits one wheel each
        let top32 = fact_u64("C-F10.u32_max");
        let bits32 = (top32 + 1).trailing_zeros();
        let mut odo32 = Odometer::build(&kit, stage, top32.to_string().len(), top32, v(0.0, -1.35), 0.62)?;
        let label32 = kit.text(
            stage,
            &format!("{} {bits32}{}", glyphs::BIT_SLATE_A, glyphs::BIT_SLATE_B),
            24.0,
            Palette::dim(),
            v(0.0, -0.7),
        )?;
        let mut intro = odo32.parts();
        intro.push(label32);
        fade_in_all(stage, &intro, 0.5)?;
        line.wait_for(stage, "then rolls back")?;
        odo32.roll_to(&kit, stage, 0, 0.6)?;
        let wrap = kit.tex(
            stage,
            &format!(
                r"2^{{{}}}\ \text{{{}}} = {}\ \text{{{}}}\quad (2^{{{}}}: {},\ \text{{{}}})",
                bits32,
                glyphs::MS,
                fact("C-F10.u32_wrap_days_at_1_per_ms_1dp"),
                glyphs::DAYS,
                bits32 - 1,
                fact("C-F10.i32_wrap_days_at_1_per_ms_1dp"),
                glyphs::SIGNED_JAVA_INT
            ),
            30.0,
            Palette::ink(),
            &[],
            v(0.0, -2.35),
        )?;
        play!(stage; fade_in(stage.arena_mut(), wrap, v(-0.15, 0.0), 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 11_6: 64 bits last centuries, if no glance returns more than was
        // written; one huge glance makes every later ticket huge
        let line = say(&kit, stage, "11_6")?;
        let mut old = odo32.parts();
        old.extend([label32, wrap, sum, answer, bound]);
        fade_out_all(stage, &old, 0.5)?;
        let glance = fact_u64("C-F11.huge_glance");
        let bits64 = glance.trailing_zeros() + 1;
        let years = kit.tex(
            stage,
            &format!(
                r"2^{{{}}}\ \text{{{} }} {}\ \text{{{}}}\quad (2^{{{}}}: {},\ \text{{{}}})",
                bits64,
                glyphs::BILLION_A_SECOND,
                fact("C-F11.u64_years_at_1e9_per_s_1dp"),
                glyphs::YEARS,
                bits64 - 1,
                fact("C-F11.i64_years_at_1e9_per_s_1dp"),
                glyphs::JAVA_LONG
            ),
            36.0,
            Palette::ink(),
            &[],
            v(0.0, 2.2),
        )?;
        play!(stage; write(stage.arena(), years).rt(1.2));
        line.wait_for(stage, "That sum assumes")?;
        let quote = quote_card(
            &kit,
            stage,
            glyphs::QUOTE_NEVER_LARGER,
            &format!(
                "{}, {}, {}, {} {}",
                glyphs::LAMPORT_SURNAME,
                cited::bakery_journal(),
                cited::BAKERY_YEAR,
                glyphs::PAGE,
                cited::NEVER_LARGER_PAGE
            ),
            v(0.0, -2.55),
            64,
        )?;
        play!(stage; fade_in(stage.arena_mut(), quote, v(0.0, 0.2), 1.0)?.rt(0.6));
        line.wait_for(stage, "One huge glance")?;
        // a slate caught mid-write: static, then a glance that reads huge,
        // and the ticket taken after it is huge too
        let well = stage.arena_mut().add(
            Rectangle::new()
                .width(8.6)
                .height(0.9)
                .corner_radius(0.1)
                .style(Style::default().fill(board(), 1.0).stroke(hardware(), 2.5, 1.0))
                .build()?
                .moved_to(v(0.0, 0.1)),
        );
        let slate_t = kit.text(stage, glyphs::GLANCE_MID_WRITE, 24.0, Palette::dim(), v(0.0, 0.9))?;
        fade_in_all(stage, &[well, slate_t], 0.4)?;
        let fx = stage.arena_mut().add(static_burst(v(0.0, 0.1), 1.1, 6));
        play!(stage; fade_in(stage.arena_mut(), fx, ORIGIN, 1.2)?.rt(0.3));
        let huge = kit.text(stage, &thousands(glance), 40.0, room_red(), v(0.0, 0.1))?;
        let next = kit.text(
            stage,
            &format!(
                "{} {}",
                glyphs::NEXT_TICKET,
                thousands(fact_u64("C-F11.ticket_after_huge_glance"))
            ),
            32.0,
            Palette::ink(),
            v(0.0, -0.8),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), fx, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), huge, ORIGIN, 0.8)?.rt(0.5),
        );
        // where a huge glance comes from on real hardware (R3 dev-07): a wide
        // ticket written in two halves (JLS §17.7; fact sheet C-S22)
        let bits64 = fact_u64("C-F11.huge_glance").trailing_zeros() + 1;
        let real_case = kit.text(
            stage,
            &format!(
                "{} {bits64}{} {}{}",
                glyphs::REAL_CASE_A,
                glyphs::REAL_CASE_B,
                bits64 / 2,
                glyphs::REAL_CASE_C
            ),
            22.0,
            Palette::dim(),
            v(0.0, -1.3),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), next, v(0.0, 0.15), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), real_case, v(0.0, 0.1), 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 11_7: the costs: twelve reads alone; a frozen chooser stalls
        // everyone; the card stays through the listen window
        let line = say(&kit, stage, "11_7")?;
        fade_out_all(stage, &[years, quote, well, slate_t, huge, next, real_case], 0.5)?;
        let wide_tickets = format!("{bits64}{}", glyphs::WIDE_TICKETS);
        let costs = card(
            &kit,
            stage,
            glyphs::FOR_PROGRAMMERS_TITLE,
            &[
                glyphs::COST_FIXED,
                glyphs::COST_PAUSED,
                glyphs::COST_DEAD,
                glyphs::COST_RESET,
                glyphs::COST_PAD,
                &wide_tickets,
            ],
            1.75,
            2.6,
            5.1,
        )?;
        // slides in from the right as it fades in
        play!(stage; fade_in(stage.arena_mut(), costs, v(-1.2, 0.0), 1.0)?.rt(0.9));
        let alone_steps = {
            let s = alone();
            replay(&FIVE_ALONE, FIVE_ALONE.initial(), &s).map_err(fail)?
        };
        let ys: Vec<(usize, f64)> = (1..=5).map(|k| (k, 2.3 - 0.95 * (k as f64 - 1.0))).collect();
        let mut five = TraceLanes::build(&kit, stage, lanes(&ys, -4.4, 1.3, 0.38), &alone_steps, &[])?;
        fade_in_all(stage, &five.base(), 0.5)?;
        line.wait_for(stage, "three reads")?;
        walk(&kit, stage, &mut five, 1..=alone_steps.len(), 0.11)?;
        let reads = alone_steps
            .iter()
            .filter(|s| s.customer == 1 && is_read(&s.label))
            .count();
        let count = kit.tex(
            stage,
            &format!(
                r"{} = 3 \times ({} - 1)\ \text{{{}}}",
                reads,
                five.layout.lanes.len(),
                glyphs::READS
            ),
            34.0,
            Palette::hero(),
            &[],
            v(-1.6, -2.6),
        )?;
        play!(stage; fade_in(stage.arena_mut(), count, v(0.0, 0.1), 1.0)?.rt(0.5));
        line.wait_for(stage, "One customer frozen")?;
        let mut old = lane_mobs(&five);
        old.push(count);
        fade_out_all(stage, &old, 0.4)?;
        let frozen = run(&THREE, FROZEN)?;
        let mut fl = TraceLanes::build(
            &kit,
            stage,
            // the lanes and their spinning rings end left of the costs card,
            // which starts at x = 1.75 (R3 dev-06)
            lanes(&[(1, 1.6), (2, 0.4), (3, -0.8)], -4.4, FROZEN_X1, 0.46),
            &frozen,
            &[],
        )?;
        fade_in_all(stage, &fl.base(), 0.4)?;
        let freeze_at = frozen
            .iter()
            .position(|s| s.customer == 3)
            .ok_or_else(|| fail("Purple moves once".into()))?
            + 1;
        walk(&kit, stage, &mut fl, 1..=freeze_at, 0.08)?;
        let pause = kit.text(stage, glyphs::FROZEN_TAG, 24.0, customer(3), v(-5.2, -1.3))?;
        play!(stage; fade_in(stage.arena_mut(), pause, ORIGIN, 1.0)?.rt(0.4));
        walk(&kit, stage, &mut fl, freeze_at + 1..=frozen.len(), 0.1)?;
        let stall = kit.text(
            stage,
            &format!("{} {} {}", glyphs::STALL_A, facts::frozen_worst(), glyphs::STALL_B),
            24.0,
            Palette::ink(),
            v(-1.6, -2.0),
        )?;
        play!(stage; fade_in(stage.arena_mut(), stall, v(0.0, 0.1), 1.0)?.rt(0.5));
        // spinning: the stalled lanes glance again and again; then the listen
        // window holds on them
        let spin_y = [fl.y(1), fl.y(2)];
        let mut rings: Vec<Box<dyn Animation>> = Vec::new();
        for k in 0..18 {
            let y = spin_y[k % 2].unwrap_or(0.0);
            let x = FROZEN_X1 + 0.12 + 0.1 * (k / 2) as f64;
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.05)
                    .arc_center(v(x, y))
                    .style(line_style(customer(k % 2 + 1), 2.0))
                    .build(),
            );
            let a = fade_in(stage.arena_mut(), ring, ORIGIN, 0.5)?;
            rings.push(stage.prepare(a)?);
        }
        let mut spin = AnimationGroup::with_lag_ratio(stage.arena_mut(), rings, 0.5)?;
        spin.update_rate_info(Some(1.1), None, None);
        stage.play_prepared(vec![Box::new(spin)])?;
        kit.listen(stage, 3.0, "11_fair_at_a_price:11_7")?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// A read of another customer: the max's reads and both waits.
fn is_read(label: &str) -> bool {
    label.starts_with("read number[") || label.starts_with("L2: ") || label.starts_with("L3: ")
}

/// A number with TeX-safe thousands separators.
/// Where 11_7's frozen lanes end: their nine spinning rings (0.1 apart)
/// stop at 1.57, left of the costs card's edge at 1.75.
const FROZEN_X1: f64 = 0.6;
const _: () = assert!(FROZEN_X1 + 0.12 + 0.1 * 8.0 + 0.05 < 1.75 - 0.1);

fn tex_thousands(n: u64) -> String {
    thousands(n).replace(',', "{,}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_11_5_line_rounds_the_32_bit_top_as_the_fact_does() {
        // "stops at about four billion": 4,294,967,295 to the nearest billion
        let top = fact_u64("C-F10.u32_max");
        assert_eq!((top as f64 / 1e9).round() as u64, 4);
        let row = crate::STORYBOARD
            .lines()
            .find(|l| l.starts_with("| 11_5 |"))
            .expect("the storyboard has 11_5");
        assert!(row.contains("A thirty-two bit slate stops at about four billion"));
        assert_eq!((top + 1).trailing_zeros(), 32);
    }

    #[test]
    fn every_schedule_replays_and_does_what_the_voice_says() {
        // 11_1: finish choosing first, go in first; overlapping, either
        let d = run(&TWO, DISJOINT).unwrap();
        assert_eq!(first_in(&d), Some(1));
        assert_eq!(first_in(&run(&TWO, OVERLAP_GREEN_FIRST).unwrap()), Some(2));
        assert_eq!(first_in(&run(&TWO, OVERLAP_BLUE_FIRST).unwrap()), Some(1));
        // the overlap runs really overlap: both flags up at once
        for s in [OVERLAP_GREEN_FIRST, OVERLAP_BLUE_FIRST] {
            assert!(run(&TWO, s).unwrap().iter().any(|st| st.after.choosing == [1, 1]));
        }
        assert!(!d.iter().any(|st| st.after.choosing == [1, 1]));
        // 11_2: after Blue's ticket, Green and Purple enter once each, then Blue
        let b = run(&THREE, BYPASS).unwrap();
        let t = b
            .iter()
            .position(|s| s.customer == 1 && s.ticket_written().is_some())
            .unwrap();
        let entries: Vec<usize> = b[t..].iter().filter(|s| s.enters()).map(|s| s.customer).collect();
        assert_eq!(entries, [2, 3, 1]);
        assert!(b.iter().all(|s| THREE.in_room(&s.after).len() <= 1));
        // 11_3: the leapfrog's biggest ticket climbs 1..6, then the bakery empties
        let leap = Trace::Leapfrog.steps();
        let last = leap.last().unwrap().after.clone();
        let extra: Vec<(usize, String)> = EMPTIES.iter().map(|&(p, s)| (p, s.to_owned())).collect();
        let tail = replay(&Trace::Leapfrog.model(), last, &extra).unwrap();
        assert_eq!(tail.last().unwrap().after.number, [0, 0]);
        let mut seen = vec![];
        for s in leap.iter().chain(&tail) {
            let x = biggest(&s.after);
            if seen.last() != Some(&x) {
                seen.push(x);
            }
        }
        // empty at the start, climbing while someone is always in line,
        // back to empty only when the bakery empties
        assert_eq!(seen, [0, 1, 2, 3, 4, 5, 6, 0]);
        // 11_7: twelve reads alone; the frozen chooser stalls the others
        let a = replay(&FIVE_ALONE, FIVE_ALONE.initial(), &alone()).unwrap();
        assert_eq!(a.iter().filter(|s| is_read(&s.label)).count(), 12);
        assert!(a.last().unwrap().enters());
        let f = run(&THREE, FROZEN).unwrap();
        let end = &f.last().unwrap().after;
        assert_eq!(end.choosing[2], 1, "Purple's flag stays up");
        assert!(THREE.in_room(end).is_empty());
        let after_freeze = f.iter().position(|s| s.customer == 3).unwrap();
        assert!(f[after_freeze..].iter().filter(|s| s.enters()).count() <= 2);
    }

    #[test]
    fn the_odometer_digits_pad_to_the_wheels() {
        let top = fact_u64("C-F10.u32_max");
        assert_eq!(format!("{top:0width$}", width = top.to_string().len()), "4294967295");
        assert_eq!((top + 1).trailing_zeros(), 32);
        assert_eq!(fact_u64("C-F11.huge_glance").trailing_zeros(), 63);
        assert_eq!(tex_thousands(1u64 << 35), "34{,}359{,}738{,}368");
    }
}
