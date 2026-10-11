//! Chapter 07, why not wrap around at 256: the 256-hour clock (07_1), two
//! times 128 as a prediction (07_2), the top bit falling into the gutter
//! (07_3), 3 and 131 doubling to the same 6 (07_4), a table of random byte
//! amounts whose top seven bits fade, leaving the last bit (07_5), and the
//! race: ex-or against wrap-around, both failing about 71% (07_6).

use std::rc::Rc;

use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, stroke_style, thousands, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::clock::Clock;
use super::draw::{Role, badge, card_fill, flat, group_of};
use super::register::Register;
use crate::model::wrap::{self, RACE_PIECES, RACE_ROUNDS, Race};

/// The wrap-around role (storyboard §3).
fn purple() -> Srgb {
    Role::wrap()
}

/// The ex-or role.
fn blue() -> Srgb {
    Role::ex_or()
}

/// A dark slot behind a bit that is zero.
fn slot_fill() -> Srgb {
    hex("#1A2130")
}

/// Bytes on a clock: one hour per byte value.
const HOURS: usize = 256;
/// 07_2's byte and the amount it is multiplied by.
const DOUBLED: u8 = 128;
const TWO: u8 = 2;
/// 07_4: the two bytes that double to the same result.
const SMALL: u8 = 3;
const LARGE: u8 = 131;
/// 07_1's amount to undo.
const THREE: u8 = 3;
/// The listen windows (storyboard: 3 s).
const LISTEN_S: f64 = 3.0;
/// How many race rounds are drawn one at a time before the counters run.
const SHOWN_ROUNDS: usize = 4;

/// A grid of square cells, one per entry, as two flat groups: the lit cells
/// and the dark ones (one fill per mobject).
struct Grid {
    lit: Mob,
    dark: Mob,
    group: Mob,
}

/// `bits[row][col]` as squares `cell` wide, centred at `centre`.
fn grid(
    stage: &mut Stage<'_>,
    bits: &[[bool; RACE_PIECES]; RACE_PIECES],
    cell: f64,
    centre: Vec3,
    color: Srgb,
) -> fmn::Result<Grid> {
    let (mut lit, mut dark) = (Vec::new(), Vec::new());
    let half = RACE_PIECES as f64 / 2.0 - 0.5;
    for (r, row) in bits.iter().enumerate() {
        for (c, &b) in row.iter().enumerate() {
            let at = v(
                centre[0] + (c as f64 - half) * cell,
                centre[1] - (r as f64 - half) * cell,
            );
            let sq = Rectangle::new()
                .width(cell * 0.86)
                .height(cell * 0.86)
                .corner_radius(cell * 0.12)
                .style(if b { flat(color, 0.9) } else { flat(slot_fill(), 1.0) })
                .build()?
                .moved_to(at);
            if b { lit.push(sq) } else { dark.push(sq) }
        }
    }
    let lit = stage.arena_mut().add(v_group(lit));
    let dark = stage.arena_mut().add(v_group(dark));
    let group = group_of(stage, &[dark, lit])?;
    Ok(Grid { lit, dark, group })
}

/// A card with one amount: "2 × (A) = 6", piece A as its badge, on a
/// card's paper.
fn amount_card(kit: &Kit, stage: &mut Stage<'_>, amount: u8, result: u8, at: Vec3) -> fmn::Result<Mob> {
    let h = 0.95;
    let times = kit.tex(
        stage,
        &format!(r"{amount} \times"),
        56.0,
        Palette::ink(),
        &[],
        v(at[0] - 0.95, at[1]),
    )?;
    let a = badge(kit, stage, 0, h * 0.27, v(at[0], at[1]))?;
    let eq = kit.tex(
        stage,
        &format!("= {result}"),
        56.0,
        Palette::ink(),
        &[],
        v(at[0] + 0.95, at[1]),
    )?;
    let back = Rectangle::new()
        .width(3.6)
        .height(h)
        .corner_radius(h * 0.16)
        .style(
            Style::default()
                .fill(card_fill(), 0.96)
                .stroke(Role::card(), h * 6.0, 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group_of(stage, &[back, times, a, eq])
}

/// One side of the race: where its pieces go.
#[derive(Clone, Copy)]
struct Side {
    x: f64,
    color: Srgb,
}

const GRID_Y: f64 = 1.3;
const GRID_CELL: f64 = 0.25;
const VERDICT_Y: f64 = -0.3;
const BAR_Y: f64 = -0.85;
const BAR_W: f64 = 3.6;
const PERCENT_Y: f64 = -1.45;
const COUNT_Y: f64 = -1.95;

/// The verdict under a round's grid.
fn verdict(kit: &Kit, stage: &mut Stage<'_>, fails: bool, x: f64) -> fmn::Result<Mob> {
    let (s, c) = if fails {
        ("fails", Palette::red())
    } else {
        ("pins down every piece", Palette::green())
    };
    kit.text(stage, s, 26.0, c, v(x, VERDICT_Y))
}

/// The failed share of the rounds so far as a bar, a big percentage and a
/// "failed of rounds" line, all following `tracker` (rounds played).
fn counters(
    kit: &Kit,
    stage: &mut Stage<'_>,
    side: Side,
    fails: Rc<Vec<usize>>,
    tracker: Mob,
) -> fmn::Result<Vec<Mob>> {
    let left = side.x - BAR_W / 2.0;
    let whole = Rectangle::new()
        .width(BAR_W)
        .height(0.3)
        .style(flat(Palette::faint(), 0.6))
        .build()?
        .moved_to(v(side.x, BAR_Y));
    let whole = stage.arena_mut().add(whole);
    stage.add_to_scene(whole)?;
    let f = Rc::clone(&fails);
    let bar = stage.always_redraw(move |s| {
        let n = s.tracker_value(tracker).unwrap_or(0.0).round() as usize;
        let share = if n == 0 { 0.0 } else { f[n] as f64 / n as f64 };
        let r = Rectangle::new()
            .width((share * BAR_W).max(0.004))
            .height(0.3)
            .style(flat(Role::lost(), 0.85))
            .build()
            .expect("positive extents");
        let m = s.add(r);
        s.move_to(m, v(left, BAR_Y), LEFT);
        m
    });
    stage.add_to_scene(bar)?;
    let f = Rc::clone(&fails);
    let percent = kit.live_value(
        stage,
        move |s| {
            let n = s.tracker_value(tracker).unwrap_or(0.0).round() as usize;
            if n == 0 { 0.0 } else { 100.0 * f[n] as f64 / n as f64 }
        },
        1,
        false,
        56.0,
        side.color,
        v(side.x + 0.1, PERCENT_Y),
        RIGHT,
    )?;
    let sign = kit.text(stage, "% fail", 36.0, side.color, v(side.x + 0.95, PERCENT_Y))?;
    stage.move_to(sign, v(side.x + 0.2, PERCENT_Y - 0.03), LEFT);
    let book = Rc::clone(&kit.book);
    let f = Rc::clone(&fails);
    let count = stage.always_redraw(move |s| {
        let n = s.tracker_value(tracker).unwrap_or(0.0).round() as usize;
        let t = Text::new(&format!(
            "{} of {} rounds failed",
            thousands(f[n] as u64),
            thousands(n as u64)
        ))
        .font_size(24.0)
        .style(explainer_kit::ink_style(Palette::dim()))
        .build(&book)
        .expect("bundled text always lays out");
        let m = s.add(t);
        s.move_to(m, v(side.x, COUNT_Y), ORIGIN);
        m
    });
    stage.add_to_scene(count)?;
    play!(stage; fade_in(stage.arena_mut(), sign, ORIGIN, 1.0)?.rt(0.4));
    Ok(vec![whole, bar, percent, sign, count])
}

/// The model's failure share as a tick on the bar, labelled.
fn model_tick(kit: &Kit, stage: &mut Stage<'_>, side: Side, share: f64) -> fmn::Result<Mob> {
    let x = side.x - BAR_W / 2.0 + share * BAR_W;
    let tick = Line::new(v(x, BAR_Y - 0.25), v(x, BAR_Y + 0.25))
        .style(stroke_style(Palette::ink(), 3.0))
        .build()?;
    let tick = stage.arena_mut().add(tick);
    let label = kit.text(
        stage,
        &format!("model {}%", (share * 100.0).round()),
        22.0,
        Palette::ink(),
        v(side.x + BAR_W / 2.0 + 0.7, BAR_Y),
    )?;
    group_of(stage, &[tick, label])
}

pub struct Wrap256 {
    pub kit: Kit,
}

impl SceneConstruct for Wrap256 {
    fn name(&self) -> &str {
        "07_why_not_wrap_around_at_256"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let race = Race::draw();

        // 07_1: the 256-hour clock; the hand runs past 255 back to 0
        let clock = Clock::build(
            &kit,
            stage,
            HOURS,
            v(-3.1, -0.15),
            2.7,
            purple(),
            &[0, HOURS / 4, HOURS / 2, 3 * HOURS / 4],
            26.0,
        )?;
        let hour = stage.add_value_tracker((HOURS - 12) as f64);
        let hand = clock.hand(stage, hour, Palette::ink());
        let title = kit.text(stage, &format!("wrap around at {HOURS}"), 40.0, purple(), v(3.7, 2.3))?;
        let modulo = kit.tex(
            stage,
            &format!(r"\text{{mod }} {HOURS}"),
            40.0,
            Palette::dim(),
            &[],
            v(3.7, 1.55),
        )?;
        kit.say(stage, "07_1")?;
        play!(stage;
            show_creation(clock.ring).rt(1.2),
            fade_in(stage.arena_mut(), clock.ticks, ORIGIN, 1.0)?.rt(1.2),
        );
        let mut labels: Vec<Box<dyn Animation>> = Vec::new();
        for &(_, l) in &clock.labels {
            let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
            labels.push(stage.prepare(a)?);
        }
        let labels = explainer_kit::lagged(stage, labels, 0.2)?;
        play!(stage; labels.rt(0.8), fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.8));
        stage.add_to_scene(hand)?;
        let readout = clock.readout(&kit, stage, hour, 64.0, Palette::ink(), clock.centre)?;
        play!(stage; tween(hour, (HOURS + 12) as f64, 3.2)?);
        play!(stage; fade_in(stage.arena_mut(), modulo, ORIGIN, 1.0)?.rt(0.6));
        let times = kit.tex(stage, &format!(r"\times {THREE}"), 64.0, purple(), &[], v(2.35, -0.2))?;
        let undo = Arrow::new(v(3.2, -0.2), v(4.5, -0.2))
            .buff(0.0)
            .thickness(3.0)
            .style(stroke_style(Palette::dim(), 3.0))
            .build()?;
        let undo = stage.arena_mut().add(undo);
        let undo_l = kit.text(stage, "undo", 24.0, Palette::dim(), v(3.85, 0.15))?;
        let divide = kit.tex(
            stage,
            &format!(r"\div {THREE}\;?"),
            64.0,
            Palette::ink(),
            &[],
            v(5.45, -0.2),
        )?;
        play!(stage; write(stage.arena(), times).rt(0.7));
        play!(stage;
            fmn::animation::grow_arrow(stage.arena_mut(), undo)?.rt(0.6),
            fade_in(stage.arena_mut(), undo_l, ORIGIN, 1.0)?.rt(0.6),
        );
        play!(stage; write(stage.arena(), divide).rt(0.7));
        kit.hold(stage)?;

        // 07_2: two times 128, on the bits; the clock moves to the corner
        let small = Clock::build(
            &kit,
            stage,
            HOURS,
            v(-5.3, 2.25),
            1.05,
            purple(),
            &[0, HOURS / 4, HOURS / 2, 3 * HOURS / 4],
            18.0,
        )?;
        stage.clear_updaters(hand, true);
        stage.clear_updaters(readout, true);
        play!(stage;
            Transform::new(clock.group, small.group).rt(1.2),
            fade_out(stage.arena_mut(), hand, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), readout, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), modulo, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), times, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), undo, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), undo_l, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), divide, ORIGIN, 1.0)?.rt(0.5),
        );
        let mut reg = Register::build(&kit, stage, DOUBLED, 0.85, v(0.9, -0.1), purple(), true)?;
        let ask = kit.tex(
            stage,
            &format!(r"{TWO} \times {DOUBLED} ="),
            64.0,
            Palette::ink(),
            &[],
            v(0.5, 2.1),
        )?;
        kit.say(stage, "07_2")?;
        reg.show(stage, 0.8)?;
        play!(stage; write(stage.arena(), ask).rt(0.8));
        let q_at = v(stage.get_right(ask)[0] + 0.55, 2.1);
        let q = predict_pause(&kit, stage, q_at)?;

        // 07_3: the top bit slides off the end and is gone: zero
        kit.say(stage, "07_3")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        let fallen = reg.double(&kit, stage, 1.3)?;
        let product = wrap::mul(TWO, DOUBLED);
        let answer = kit.tex(
            stage,
            &format!(r"{TWO} \times {DOUBLED} \equiv {product} \pmod{{{HOURS}}}"),
            64.0,
            purple(),
            &[],
            v(0.9, 2.1),
        )?;
        let lost = kit.text(
            stage,
            "lost",
            26.0,
            Role::lost(),
            v(reg.gutter_point()[0], reg.centre[1] - 0.95),
        )?;
        play!(stage;
            fmn::animation::replacement_transform(ask, answer).rt(0.9),
            fade_in(stage.arena_mut(), lost, ORIGIN, 1.0)?.rt(0.6),
        );
        // the listen window: the lost bit's tone slides off the top; on the
        // corner clock, 128 hours on from 128 lands on 0
        let wav = super::sound::wav("07_3_overflow", &super::sound::overflow(None, LISTEN_S))?;
        kit.sonify(stage, LISTEN_S, "07_3", &wav)?;
        let lap = stage.add_value_tracker(f64::from(DOUBLED));
        let hand = small.hand(stage, lap, Palette::ink());
        stage.add_to_scene(hand)?;
        play!(stage; tween(lap, f64::from(DOUBLED) * f64::from(TWO), 2.0)?);
        // the live clock is the big one, morphed into the corner
        let zero = clock.labels[0].1;
        play!(stage; indicate(stage.arena_mut(), zero, 1.6, Some([0.94, 0.68, 0.37]))?.rt(1.0));
        stage.clear_updaters(hand, true);

        // 07_4: a card saying 2 × A = 6 can't tell you A: 3 or 131
        let six = wrap::mul(TWO, SMALL);
        assert_eq!(six, wrap::mul(TWO, LARGE), "3 and 131 double to the same byte");
        let everything = reg.everything(stage)?;
        play!(stage;
            fade_out(stage.arena_mut(), everything, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), fallen, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), lost, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), answer, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), hand, ORIGIN, 1.0)?.rt(0.6),
        );
        let card = amount_card(&kit, stage, TWO, six, v(0.6, 2.75))?;
        let mut left = Register::build(&kit, stage, SMALL, 0.62, v(-3.25, 0.3), purple(), true)?;
        let mut right = Register::build(&kit, stage, LARGE, 0.62, v(3.85, 0.3), purple(), true)?;
        let left_v = kit.tex(stage, &SMALL.to_string(), 48.0, Palette::ink(), &[], v(-3.25, 1.45))?;
        let right_v = kit.tex(stage, &LARGE.to_string(), 48.0, Palette::ink(), &[], v(3.85, 1.45))?;
        kit.say(stage, "07_4")?;
        play!(stage; fade_in(stage.arena_mut(), card, v(0.0, -0.2), 1.0)?.rt(0.6));
        left.show(stage, 0.5)?;
        right.show(stage, 0.5)?;
        play!(stage;
            fade_in(stage.arena_mut(), left_v, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), right_v, ORIGIN, 1.0)?.rt(0.4),
        );
        let fallen_l = left.double(&kit, stage, 1.0)?;
        let fallen_r = right.double(&kit, stage, 1.0)?;
        assert_eq!((left.value(), right.value()), (six, six));
        let left_eq = kit.tex(
            stage,
            &format!(r"{TWO} \times {SMALL} = {six}"),
            40.0,
            Palette::ink(),
            &[],
            v(-3.25, -0.75),
        )?;
        let right_eq = kit.tex(
            stage,
            &format!(
                r"{TWO} \times {LARGE} = {} \to {six}",
                u32::from(TWO) * u32::from(LARGE)
            ),
            40.0,
            Palette::ink(),
            &[],
            v(3.85, -0.75),
        )?;
        play!(stage;
            write(stage.arena(), left_eq).rt(0.7),
            write(stage.arena(), right_eq).rt(0.7),
        );
        let either = kit.text(
            stage,
            &format!("A could be {SMALL}, or {LARGE}"),
            36.0,
            Palette::red(),
            v(0.3, -1.9),
        )?;
        play!(stage; fade_in(stage.arena_mut(), either, v(0.0, 0.15), 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 07_5: even amounts lose information; random byte amounts as bits;
        // the top seven bits fade, leaving the last
        let even = kit.text(
            stage,
            &format!("{TWO} is even: two bytes, one result"),
            30.0,
            purple(),
            v(0.3, -2.75),
        )?;
        kit.say(stage, "07_5")?;
        play!(stage; fade_in(stage.arena_mut(), even, ORIGIN, 1.0)?.rt(0.6));
        stage.wait(2.4)?;
        fade_all(stage, 0.7)?;
        let _ = (fallen_l, fallen_r);
        let amounts = &race.wrap[0];
        let header = kit.text(
            stage,
            &format!("{RACE_PIECES} cards, a random byte amount for each of {RACE_PIECES} pieces"),
            30.0,
            Palette::ink(),
            v(0.0, 2.45),
        )?;
        let bytes = ByteTable::build(stage, amounts, v(0.0, -0.25))?;
        play!(stage; fade_in(stage.arena_mut(), header, ORIGIN, 1.0)?.rt(0.5));
        bytes.show(stage, 1.4)?;
        let last = kit.text(stage, "only the last bit: odd or even", 30.0, purple(), v(0.0, -2.75))?;
        play!(stage;
            bytes.high_lit.animate().set_anim_args(AnimateArgs { run_time: Some(1.4), ..AnimateArgs::default() })?
                .set_fill(Some(Palette::faint()), Some(0.9))?,
            fade_in(stage.arena_mut(), last, ORIGIN, 1.0)?.rt(1.0),
        );
        kit.hold(stage)?;

        // 07_6: the race: ex-or on the left, wrap-around on the right
        let left_side = Side { x: -3.5, color: blue() };
        let right_side = Side {
            x: 3.5,
            color: purple(),
        };
        let xor_title = kit.text(stage, "ex-or cards", 36.0, blue(), v(left_side.x, 3.3))?;
        let wrap_title = kit.text(stage, "wrap-around cards", 36.0, purple(), v(right_side.x, 3.3))?;
        let first_xor = grid(stage, &race.xor_bits(0), GRID_CELL, v(left_side.x, GRID_Y), blue())?;
        let first_wrap = grid(stage, &race.wrap_bits(0), GRID_CELL, v(right_side.x, GRID_Y), purple())?;
        kit.say(stage, "07_6")?;
        play!(stage;
            fade_out(stage.arena_mut(), header, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), last, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), bytes.high_lit, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), bytes.high_dark, ORIGIN, 1.0)?.rt(0.5),
        );
        play!(stage;
            Transform::new(bytes.low_lit, first_wrap.lit).rt(1.2),
            Transform::new(bytes.low_dark, first_wrap.dark).rt(1.2),
            fade_in(stage.arena_mut(), first_xor.group, ORIGIN, 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), xor_title, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), wrap_title, ORIGIN, 1.0)?.rt(0.8),
        );
        let (xor_fails, wrap_fails) = (race.xor_fails(), race.wrap_fails());
        let mut shown = vec![first_xor.group, bytes.low_lit, bytes.low_dark];
        let mut verdicts = vec![
            verdict(&kit, stage, xor_fails[0], left_side.x)?,
            verdict(&kit, stage, wrap_fails[0], right_side.x)?,
        ];
        play!(stage;
            fade_in(stage.arena_mut(), verdicts[0], ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), verdicts[1], ORIGIN, 1.0)?.rt(0.4),
        );
        stage.wait(0.5)?;
        for round in 1..SHOWN_ROUNDS {
            let gx = grid(stage, &race.xor_bits(round), GRID_CELL, v(left_side.x, GRID_Y), blue())?;
            let gw = grid(
                stage,
                &race.wrap_bits(round),
                GRID_CELL,
                v(right_side.x, GRID_Y),
                purple(),
            )?;
            let vx = verdict(&kit, stage, xor_fails[round], left_side.x)?;
            let vw = verdict(&kit, stage, wrap_fails[round], right_side.x)?;
            for &m in shown.iter().chain(&verdicts) {
                stage.scene_mut().remove(&[m]);
            }
            stage.add_to_scene(gx.group)?;
            stage.add_to_scene(gw.group)?;
            stage.add_to_scene(vx)?;
            stage.add_to_scene(vw)?;
            shown = vec![gx.group, gw.group];
            verdicts = vec![vx, vw];
            stage.wait(0.45)?;
        }
        // the counters: every round so far, then all of them
        let cum = |fails: &[bool]| -> Rc<Vec<usize>> {
            let mut c = vec![0];
            for &f in fails {
                c.push(c.last().copied().unwrap_or(0) + usize::from(f));
            }
            Rc::new(c)
        };
        let rounds = stage.add_value_tracker(SHOWN_ROUNDS as f64);
        let mut live = counters(&kit, stage, left_side, cum(&xor_fails), rounds)?;
        live.extend(counters(&kit, stage, right_side, cum(&wrap_fails), rounds)?);
        let model = crate::model::p_fail(2, RACE_PIECES as u32, 0);
        let ticks = [
            model_tick(&kit, stage, left_side, model)?,
            model_tick(&kit, stage, right_side, model)?,
        ];
        let mut dim: Vec<Box<dyn Animation>> = Vec::new();
        for &m in shown.iter().chain(&verdicts) {
            let a = m.animate().set_opacity(0.25)?;
            dim.push(stage.prepare(a)?);
        }
        for &t in &ticks {
            let a = fade_in(stage.arena_mut(), t, ORIGIN, 1.0)?;
            dim.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            dim,
            PlayOverrides {
                run_time: Some(0.6),
                ..PlayOverrides::default()
            },
        )?;
        let run = rounds
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(8.0),
                rate_func: Some(fmn::core::rate::rush_into),
                ..AnimateArgs::default()
            })?
            .set_value(RACE_ROUNDS as f64)?;
        stage.play(run)?;
        kit.hold(stage)?;
        let note = precisely(
            &kit,
            stage,
            &format!(
                "a table of amounts wrapped at {HOURS} can be undone exactly when its determinant is odd, exactly when its \
                 last bits alone can be undone; random amounts give random last bits, so the odds are ex-or's"
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        stage.wait(4.5)?;
        let inv3 = wrap::inverse_odd(THREE);
        let big = u32::from(THREE) * u32::from(inv3);
        let dev = lower_third(
            &kit,
            stage,
            "for programmers:",
            &format!(
                "{THREE} × {inv3} = {big} = {} · {HOURS} + {}: multiplying by an odd constant can be undone, which is \
                 why hash mixers multiply by odd numbers",
                big / HOURS as u32,
                big % HOURS as u32
            ),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), dev, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        stage.wait(4.5)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 07_5's table: ten cards of ten byte amounts, each byte drawn as eight
/// thin bars (its bits, top bit first), the last bar set apart. Four flat
/// groups, each in reading order (card by card, piece by piece): lit and
/// dark bars of the top seven bits, and of the last bit. The last bits'
/// groups line up child for child with a [`grid`] of the same table's
/// parities, so one morphs into the other.
struct ByteTable {
    high_lit: Mob,
    high_dark: Mob,
    low_lit: Mob,
    low_dark: Mob,
}

/// Bar size and pitches of the byte table.
const BAR_PITCH: f64 = 0.074;
const ENTRY_PITCH: f64 = 0.8;
const ROW_PITCH: f64 = 0.36;

impl ByteTable {
    fn build(stage: &mut Stage<'_>, amounts: &[[u8; RACE_PIECES]; RACE_PIECES], centre: Vec3) -> fmn::Result<Self> {
        let half = RACE_PIECES as f64 / 2.0 - 0.5;
        let bar = |at: Vec3, on: bool, high: bool| -> fmn::Result<VMobject> {
            let (c, o) = if on { (purple(), 0.95) } else { (slot_fill(), 1.0) };
            let w = if high { BAR_PITCH * 0.78 } else { BAR_PITCH * 1.1 };
            Ok(Rectangle::new()
                .width(w)
                .height(ROW_PITCH * 0.72)
                .style(flat(c, o))
                .build()?
                .moved_to(at))
        };
        let mut classes: [Vec<VMobject>; 4] = Default::default();
        for (r, row) in amounts.iter().enumerate() {
            let y = centre[1] - (r as f64 - half) * ROW_PITCH;
            for (c, &a) in row.iter().enumerate() {
                let x0 = centre[0] + (c as f64 - half) * ENTRY_PITCH - 4.0 * BAR_PITCH;
                for k in 0..8 {
                    let on = a >> (7 - k) & 1 == 1;
                    let high = k < 7;
                    let x = x0 + k as f64 * BAR_PITCH + if high { 0.0 } else { 0.05 };
                    classes[usize::from(!high) * 2 + usize::from(!on)].push(bar(v(x, y), on, high)?);
                }
            }
        }
        let [hl, hd, ll, ld] = classes.map(|m| stage.arena_mut().add(v_group(m)));
        Ok(Self {
            high_lit: hl,
            high_dark: hd,
            low_lit: ll,
            low_dark: ld,
        })
    }

    /// The four groups fading in together.
    fn show(&self, stage: &mut Stage<'_>, run_time: f64) -> fmn::Result<()> {
        play!(stage;
            fade_in(stage.arena_mut(), self.high_dark, v(0.0, 0.1), 1.0)?.rt(run_time),
            fade_in(stage.arena_mut(), self.low_dark, v(0.0, 0.1), 1.0)?.rt(run_time),
            fade_in(stage.arena_mut(), self.high_lit, v(0.0, 0.1), 1.0)?.rt(run_time),
            fade_in(stage.arena_mut(), self.low_lit, v(0.0, 0.1), 1.0)?.rt(run_time),
        );
        Ok(())
    }
}
