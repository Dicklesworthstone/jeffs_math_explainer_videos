//! Chapter 01, one plus one is one: the slate caught mid-stroke (01_1), the
//! question and its date (01_2), the title (01_3), two programs adding one
//! to a shared scoreboard on space-time lanes (01_4–01_6), every order as a
//! staircase in the progress square (01_7–01_8), and why counting orders is
//! not measuring chances (01_9). The orders, their final scores and the
//! values on every chip come from `model::scoreboard_orders` (oracle C-F1).

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, choose, fade_all, play, v};
use explainer_score::rng::SplitMix64;
use fmn::prelude::*;

use super::draw::{
    Figure, Lattice, access_arrow, chalk, chalk_number, customer, fade_in_all, fade_out_all, faded_line, fail, fan,
    figure, flat, flicker, flicker_numerals, lane, line_style, quote_card, room_red, say, scoreboard, slate, step_dot,
};
use super::glyphs as g;
use crate::facts::cited;
use crate::model::{Op, Order, SCORE_START, apply, scoreboard_orders, scoreboard_program};

pub struct OnePlusOne {
    pub kit: Kit,
}

/// The lanes of 01_4–01_6: Blue above the scoreboard's track, Green below.
const BLUE_Y: f64 = 1.55;
const GREEN_Y: f64 = -1.55;
const HEAD_X: f64 = -5.0;
const LANE_X0: f64 = -4.4;
const LANE_X1: f64 = 6.6;
/// The scoreboard's height.
const BOARD: f64 = 1.05;
/// Where the four steps of the unlucky order happen, left to right.
const SLOTS: [f64; 4] = [-2.6, -0.6, 1.4, 3.4];

/// How far apart the six staircases fan (along the anti-diagonal).
const FAN: f64 = 1.0;

/// The progress square of 01_7–01_8.
const SQUARE: Lattice = Lattice {
    center: [-1.9, -0.25, 0.0],
    side: 4.2,
    steps: 2,
};

impl SceneConstruct for OnePlusOne {
    fn name(&self) -> &str {
        "01_one_plus_one_is_one"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        torn_slate(&kit, stage)?;
        let t = title(&kit, stage)?;
        let race = race(&kit, stage, t)?;
        let square = staircases(&kit, stage, race)?;
        strip(&kit, stage, square)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;
        kit.finish(stage)
    }
}

// --------------------------------------------------------------- 01_1–01_2

/// An eye looking right: the glance.
fn eye(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let white = Ellipse::new()
        .width(1.05)
        .height(0.58)
        .style(flat(chalk(), 1.0))
        .build()
        .moved_to(at);
    let iris = Circle::new()
        .radius(0.21)
        .arc_center(v(at[0] + 0.2, at[1]))
        .style(flat(Palette::dim(), 1.0))
        .build();
    let pupil = Circle::new()
        .radius(0.11)
        .arc_center(v(at[0] + 0.24, at[1]))
        .style(flat(super::draw::board(), 1.0))
        .build();
    Ok(stage.arena_mut().add(v_group([white, iris, pupil])))
}

/// 01_1–01_2: a slate reading 9 rewritten as 10, caught mid-stroke; what a
/// glance can return (Lamport's 2496); two programs reaching for it; 1973.
fn torn_slate(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let at = v(1.3, 0.75);
    let size = 2.5;
    let s = slate(kit, stage, cited::TORN_FROM, at, size)?;
    let line = say(kit, stage, "01_1")?;
    play!(stage;
        fade_in(stage.arena_mut(), s.board, ORIGIN, 1.0)?.rt(0.8),
        write(stage.arena(), s.value).rt(1.0),
    );
    // "Someone is rewriting it as ten": the nine rubbed out, the ten half drawn
    line.wait_for(stage, "Someone is rewriting")?;
    let ten = chalk_number(kit, stage, cited::TORN_TO, at, size)?;
    let mut half = write(stage.arena(), ten);
    half.update_rate_info(Some(1.8), Some(RateFunc::smooth().not_quite_there(0.5)), None);
    let rub = s
        .value
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.8),
            ..AnimateArgs::default()
        })?
        .set_opacity(0.35)?;
    play!(stage; half, rub);
    // the glance
    line.wait_for(stage, "Glance at it")?;
    let eye_at = v(-4.7, 0.75);
    let the_eye = eye(stage, eye_at)?;
    let mut sight = Vec::new();
    for dy in [0.45, -0.45] {
        let l = DashedLine::new(v(eye_at[0] + 0.62, eye_at[1]), v(at[0] - 0.7 * size - 0.12, at[1] + dy))
            .dash_length(0.12)
            .style(faded_line(chalk(), 2.0, 0.55))
            .build()?;
        sight.push(stage.arena_mut().add(l));
    }
    let mut glance = vec![the_eye];
    glance.extend(&sight);
    fade_in_all(stage, &glance, 0.7)?;
    // what the glance returns: nine, ten, or 2496
    let read_at = v(eye_at[0], -1.0);
    let reading = slate(kit, stage, cited::TORN_FROM, read_at, 1.1)?;
    let values = [cited::TORN_FROM, cited::TORN_TO, cited::TORN_VALUE];
    let numerals = flicker_numerals(kit, stage, &values, read_at, 1.1)?;
    line.wait_for(stage, "nine, or ten")?;
    play!(stage; fade_in(stage.arena_mut(), reading.board, ORIGIN, 1.0)?.rt(0.4));
    flicker(stage, &numerals, 0.5, 14.0, 0, 0x0101)?;
    line.wait_for(stage, "ten, or, Lamport")?;
    flicker(stage, &numerals, 0.6, 14.0, 1, 0x0102)?;
    line.wait_for(stage, "two thousand")?;
    flicker(stage, &numerals, 0.9, 16.0, 2, 0x0103)?;
    let words = format!(
        "“{} {} {} {} {} {}.”",
        g::TORN_A,
        cited::TORN_FROM,
        g::TORN_B,
        cited::TORN_TO,
        g::TORN_C,
        cited::TORN_VALUE
    );
    let quote = quote_card(kit, stage, &words, g::TURING_SOURCE, v(0.0, -2.75), 66)?;
    play!(stage; fade_in(stage.arena_mut(), quote, v(0.0, 0.15), 1.0)?.rt(0.8));
    kit.hold(stage)?;

    // 01_2: two programs reach for the same slate; 1973
    let mut out = vec![the_eye, reading.board];
    out.extend(&sight);
    out.extend(&numerals);
    let line = say(kit, stage, "01_2")?;
    fade_out_all(stage, &out, 0.5)?;
    let centre = v(0.0, at[1]);
    let shift = v(centre[0] - at[0], 0.0);
    play!(stage;
        s.board.animate().shift(shift)?,
        s.value.animate().shift(shift)?,
        ten.animate().shift(shift)?,
    );
    let blue = figure(kit, stage, 1, v(-5.4, at[1]), 1.3)?;
    let green = figure(kit, stage, 2, v(5.4, at[1]), 1.3)?;
    for f in [&blue, &green] {
        stage.set_opacity(f.badge, 0.0, true); // badges come at 03_1
    }
    fade_in_all(stage, &[blue.group, green.group], 0.6)?;
    let half_w = 0.7 * size;
    let reach_b = stage
        .arena_mut()
        .add(access_arrow(v(-4.8, at[1]), v(-half_w - 0.05, at[1]), 1)?);
    let reach_g = stage
        .arena_mut()
        .add(access_arrow(v(4.8, at[1]), v(half_w + 0.05, at[1]), 2)?);
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), reach_b)?.rt(0.9),
        fmn::animation::grow_arrow(stage.arena_mut(), reach_g)?.rt(0.9),
    );
    line.wait_for(stage, "In nineteen seventy-three")?;
    let words = format!(
        "“{} {}{}”",
        g::IMPOSSIBLE_A,
        cited::CONSIDERED_IMPOSSIBLE_YEAR,
        g::IMPOSSIBLE_B
    );
    let card = quote_card(kit, stage, &words, g::TURING_SOURCE, v(0.0, -2.6), 66)?;
    play!(stage;
        fade_out(stage.arena_mut(), quote, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), card, v(0.0, 0.15), 1.0)?.rt(0.8),
    );
    kit.hold(stage)?;
    Ok(())
}

// --------------------------------------------------------------------- 01_3

/// 01_3: the title writes itself in as the cold open fades (one move, no
/// gap), then holds through the "take a number" motif; 01_4's picture
/// fades it out as it comes in.
fn title(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let t = kit.text_styled(stage, g::TITLE, 60.0, Palette::hero(), v(0.0, 0.2), true, false)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for m in stage.scene().mobjects().to_vec() {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
    }
    let w = write(stage.arena(), t);
    anims.push(stage.prepare(w.rt(1.2))?);
    stage.play_prepared(anims)?;
    kit.listen(stage, 3.0, "01_3")?;
    Ok(t)
}

// --------------------------------------------------------------- 01_4–01_6

/// What 01_7 morphs into the progress square.
struct Race {
    blue: Figure,
    green: Figure,
    blue_lane: Mob,
    green_lane: Mob,
    /// Everything else on screen, faded at 01_7.
    rest: Vec<Mob>,
    /// The unlucky order the lanes showed.
    shown: Order,
}

/// The values a two-step order shows: for each step, (who, the value read
/// or written), replayed on the scoreboard's program from 41.
fn replay_scores(order: &Order) -> Vec<(usize, Op, u32)> {
    let prog = scoreboard_program();
    let mut mem = vec![SCORE_START];
    let mut pcs = [0usize; 2];
    let mut ms = [0u32; 2];
    let mut out = Vec::new();
    for &i in &order.moves {
        let op = prog[pcs[i]];
        if let Some((m2, priv2)) = apply(op, i, 2, &mem, ms[i]) {
            mem = m2;
            ms[i] = priv2;
        }
        pcs[i] += 1;
        let shown = if op == Op::ReadScore { ms[i] } else { mem[0] };
        out.push((i, op, shown));
    }
    out
}

/// 01_4–01_6: two programs share a scoreboard; x = x + 1 is a read and a
/// write; one unlucky order loses a point.
fn race(kit: &Kit, stage: &mut Stage<'_>, title: Mob) -> fmn::Result<Race> {
    let mut rest = Vec::new();
    // 01_4: two lanes, the scoreboard, x, the code, the caption
    let line = say(kit, stage, "01_4")?;
    let blue = figure(kit, stage, 1, v(HEAD_X, BLUE_Y), 1.0)?;
    let green = figure(kit, stage, 2, v(HEAD_X, GREEN_Y), 1.0)?;
    for f in [&blue, &green] {
        stage.set_opacity(f.badge, 0.0, true);
    }
    let blue_lane = stage.arena_mut().add(lane(LANE_X0, LANE_X1, BLUE_Y, 1)?);
    let green_lane = stage.arena_mut().add(lane(LANE_X0, LANE_X1, GREEN_Y, 2)?);
    play!(stage;
        fade_out(stage.arena_mut(), title, v(0.0, 0.3), 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), blue.group, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), green.group, ORIGIN, 1.0)?.rt(0.6),
    );
    play!(stage; show_creation(blue_lane).rt(1.0), show_creation(green_lane).rt(1.0));
    line.wait_for(stage, "share a scoreboard")?;
    let mut board = scoreboard(kit, stage, SCORE_START, v(HEAD_X, 0.0), BOARD)?;
    let track = Line::new(v(LANE_X0, 0.0), v(LANE_X1, 0.0))
        .style(faded_line(Palette::dim(), 3.0, 0.7))
        .build()?;
    let track = stage.arena_mut().add(track);
    play!(stage;
        fade_in(stage.arena_mut(), board.panel, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), board.value, ORIGIN, 1.0)?.rt(0.6),
        show_creation(track).rt(1.0),
    );
    line.wait_for(stage, "call it x")?;
    let x = kit.mono(stage, g::X, 52.0, Palette::ink(), v(HEAD_X - 0.85 * BOARD - 0.45, 0.0))?;
    play!(stage; write(stage.arena(), x).rt(0.6));
    line.wait_for(stage, "write x equals")?;
    let code = kit.mono(stage, g::CODE, 46.0, Palette::ink(), v(1.0, 3.3))?;
    play!(stage; write(stage.arena(), code).rt(1.0));
    line.wait_for(stage, "It means")?;
    let means = kit.text(stage, g::MAKE_BIGGER, 28.0, Palette::dim(), v(1.0, 2.65))?;
    play!(stage; fade_in(stage.arena_mut(), means, v(0.0, 0.1), 1.0)?.rt(0.6));
    let caption = kit.text(
        stage,
        &format!("{}   ·   {}", g::TIME_RUNS, g::ARROWS),
        24.0,
        Palette::dim(),
        v(0.4, -3.45),
    )?;
    play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.6));
    rest.extend([board.panel, track, x, code, caption]);

    // 01_5: two steps, read then write
    let line = say(kit, stage, "01_5")?;
    play!(stage; fade_out(stage.arena_mut(), means, ORIGIN, 1.0)?.rt(0.4));
    line.wait_for(stage, "Read the score")?;
    let read = kit.text(stage, g::READ_X, 32.0, Palette::ink(), v(-0.4, 2.6))?;
    play!(stage; fade_in(stage.arena_mut(), read, v(0.0, 0.1), 1.0)?.rt(0.5));
    line.wait_for(stage, "then write")?;
    let then = Arrow::new(v(0.55, 2.6), v(1.35, 2.6))
        .buff(0.0)
        .thickness(3.0)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 3.0, 1.0),
        )
        .build()?;
    let then = stage.arena_mut().add(then);
    let write_x = kit.text(stage, g::WRITE_X, 32.0, Palette::ink(), v(2.6, 2.6))?;
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), then)?.rt(0.5),
        fade_in(stage.arena_mut(), write_x, v(0.0, 0.1), 1.0)?.rt(0.5),
    );
    rest.extend([read, then, write_x]);

    // 01_6: one unlucky order: both read 41, both write 42
    let orders = scoreboard_orders();
    let shown = orders
        .iter()
        .find(|o| o.moves == [0, 1, 0, 1])
        .cloned()
        .ok_or_else(|| fail("the scoreboard has no read-read-write-write order".into()))?;
    let steps = replay_scores(&shown);
    let line = say(kit, stage, "01_6")?;
    let lane_y = |i: usize| if i == 0 { BLUE_Y } else { GREEN_Y };
    let mut write_arrows = Vec::new();
    let mut plus = Vec::new();
    for (k, &(i, op, value)) in steps.iter().enumerate() {
        if k == 0 {
            line.wait_for(stage, "Each reads")?;
        } else if k == 2 {
            line.wait_for(stage, "each writes")?;
        }
        let x = SLOTS[k];
        let y = lane_y(i);
        let side = if i == 0 { 1.0 } else { -1.0 };
        let dot = stage.arena_mut().add(step_dot(v(x, y), i + 1));
        let (from, to) = if op == Op::ReadScore {
            (v(x, 0.0), v(x, y))
        } else {
            (v(x, y), v(x, 0.0))
        };
        let arrow = stage.arena_mut().add(access_arrow(from, to, i + 1)?);
        let chip = kit.text(
            stage,
            &value.to_string(),
            30.0,
            chalk(),
            v(x + 0.42, y / 2.0 + 0.12 * side),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), dot, ORIGIN, 1.0)?.rt(0.3),
            fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(0.6),
            fade_in(stage.arena_mut(), chip, ORIGIN, 1.0)?.rt(0.6),
        );
        rest.extend([dot, arrow, chip]);
        if op == Op::WriteScore {
            let [a, b] = board.show(kit, stage, value)?;
            stage.play_prepared(vec![a, b])?;
            write_arrows.push((i, x, y));
        }
        stage.wait(0.35)?;
    }
    rest.push(board.value);
    line.wait_for(stage, "Two points scored")?;
    for &(i, x, y) in &write_arrows {
        let p = kit.text(stage, g::PLUS_ONE, 30.0, customer(i + 1), v(x - 0.45, y / 2.0))?;
        play!(stage; fade_in(stage.arena_mut(), p, v(0.0, 0.1), 1.0)?.rt(0.4));
        plus.push(p);
    }
    line.wait_for(stage, "one counted")?;
    // the scoreboard counted one: the second +1 is lost
    let end = steps.last().map_or(SCORE_START, |s| s.2);
    let counted = (end - SCORE_START) as usize;
    let lost: Vec<Mob> = plus.iter().skip(counted).copied().collect();
    let ring = Rectangle::new()
        .width(1.7 * BOARD + 0.2)
        .height(BOARD + 0.2)
        .corner_radius(0.12)
        .style(line_style(room_red(), 4.0))
        .build()?
        .moved_to(v(HEAD_X, 0.0));
    let ring = stage.arena_mut().add(ring);
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &lost {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .set_color(room_red())?;
        anims.push(stage.prepare(a)?);
    }
    let a = fade_in(stage.arena_mut(), ring, ORIGIN, 1.2)?;
    anims.push(stage.prepare(a.rt(0.6))?);
    stage.play_prepared(anims)?;
    rest.extend(plus);
    rest.push(ring);
    kit.hold(stage)?;
    Ok(Race {
        blue,
        green,
        blue_lane,
        green_lane,
        rest,
        shown,
    })
}

// --------------------------------------------------------------- 01_7–01_8

/// What 01_9 sweeps away.
struct SquareMobs {
    mobs: Vec<Mob>,
}

/// 01_7–01_8: the lanes become the sides of the progress square; every order
/// is a staircase; the four through the middle lose a point.
fn staircases(kit: &Kit, stage: &mut Stage<'_>, race: Race) -> fmn::Result<SquareMobs> {
    let orders = scoreboard_orders();
    let n = orders.len();
    let lost: Vec<bool> = orders.iter().map(|o| o.end[0] == SCORE_START + 1).collect();
    let n_lost = lost.iter().filter(|&&l| l).count();
    let steps = scoreboard_program().len();
    let l = SQUARE;
    let nf = steps as f64;
    let line = say(kit, stage, "01_7")?;
    fade_out_all(stage, &race.rest, 0.6)?;
    // the lanes swing into the square's sides
    let bottom = stage
        .arena_mut()
        .add(lane(l.p(0.0, 0.0)[0], l.p(nf, 0.0)[0], l.p(0.0, 0.0)[1], 1)?);
    let left = Line::new(l.p(0.0, 0.0), l.p(0.0, nf))
        .style(faded_line(customer(2), 4.0, 0.6))
        .build()?;
    let left = stage.arena_mut().add(left);
    let blue_to = v(l.p(nf, 0.0)[0] + 0.7, l.p(0.0, 0.0)[1]);
    let green_to = v(l.p(0.0, 0.0)[0], l.p(0.0, nf)[1] + 0.6);
    play!(stage, 1.6;
        Transform::new(race.blue_lane, bottom),
        Transform::new(race.green_lane, left).with_path_arc(-std::f64::consts::FRAC_PI_2, OUT),
        race.blue.group.animate().move_to(blue_to, ORIGIN)?.scale(0.8)?,
        race.green.group.animate().move_to(green_to, ORIGIN)?.scale(0.8)?,
    );
    let grid = stage.arena_mut().add(l.grid()?);
    let mut labels = Vec::new();
    for (k, word) in [g::READ, g::WRITE].into_iter().enumerate() {
        let c = k as f64 + 0.5;
        let below = v(l.p(c, 0.0)[0], l.p(0.0, 0.0)[1] - 0.66);
        labels.push(kit.text(stage, word, 24.0, customer(1), below)?);
        let beside = v(l.p(0.0, 0.0)[0] - 0.95, l.p(0.0, c)[1]);
        labels.push(kit.text(stage, word, 24.0, customer(2), beside)?);
    }
    let mut ins = vec![grid];
    ins.extend(&labels);
    fade_in_all(stage, &ins, 0.6)?;
    // the order just shown is the first staircase; then the other five
    let style = Style::default().fill(chalk(), 0.0).stroke(chalk(), 3.5, 0.9);
    let first = orders.iter().position(|o| *o == race.shown).unwrap_or(0);
    let stairs: Vec<Mob> = orders
        .iter()
        .enumerate()
        .map(|(r, o)| stage.arena_mut().add(l.staircase(o, fan(r, n, FAN), style)))
        .collect();
    let [start, end] = l.ends(FAN);
    let ends = [stage.arena_mut().add(start), stage.arena_mut().add(end)];
    fade_in_all(stage, &ends, 0.4)?;
    play!(stage; show_creation(stairs[first]).rt(1.4));
    stage.scene_mut().bring_to_front(&ends)?;
    line.wait_for(stage, "There are six")?;
    let others: Vec<Box<dyn Animation>> = (0..n)
        .filter(|&r| r != first)
        .map(|r| Box::new(show_creation(stairs[r])) as Box<dyn Animation>)
        .collect();
    let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), others, 0.25)?.with_run_time(1.8);
    stage.play(group)?;
    stage.scene_mut().bring_to_front(&ends)?;
    let count = kit.text_left(stage, &format!("{n} {}", g::ORDERS), 40.0, Palette::ink(), v(1.5, 1.1))?;
    play!(stage; write(stage.arena(), count).rt(0.6));
    let total = steps * 2;
    let why = format!(
        "{n} = C({total}, {steps}): {} {steps} {} {total} {}",
        g::PICK_A,
        g::PICK_B,
        g::PICK_C
    );
    if choose(total as u64, steps as u64) != n as u128 {
        return Err(fail(format!(
            "{n} orders, but C({total}, {steps}) = {}",
            choose(total as u64, steps as u64)
        )));
    }
    let lower = lower_third(kit, stage, g::WHY_SIX, &why)?;
    play!(stage; fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.6));
    let q = predict_pause(kit, stage, v(4.6, -0.6))?;

    // 01_8: four lose a point: exactly the ones through the middle
    let line = say(kit, stage, "01_8")?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (r, &is_lost) in lost.iter().enumerate() {
        if is_lost {
            let a = stairs[r]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.8),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(room_red()), Some(4.5), Some(1.0))?;
            anims.push(stage.prepare(a)?);
        }
    }
    let a = fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?;
    anims.push(stage.prepare(a.rt(0.3))?);
    stage.play_prepared(anims)?;
    let bad = kit.text_left(
        stage,
        &format!("{n_lost} {}", g::LOSE_A_POINT),
        34.0,
        room_red(),
        v(1.5, 0.45),
    )?;
    play!(stage; write(stage.arena(), bad).rt(0.6));
    line.wait_for(stage, "through the middle")?;
    let mid = l.p(1.0, 1.0);
    let note_1 = kit.text_left(stage, g::MIDDLE_1, 26.0, Palette::ink(), v(1.5, -1.05))?;
    let note_2 = kit.text_left(stage, g::MIDDLE_2, 26.0, Palette::ink(), v(1.5, -1.45))?;
    let pointer = Line::new(v(1.35, -1.25), v(mid[0] + 0.32, mid[1] - 0.12))
        .style(faded_line(Palette::dim(), 2.0, 0.8))
        .build()?;
    let pointer = stage.arena_mut().add(pointer);
    fade_in_all(stage, &[note_1, note_2, pointer], 0.6)?;
    line.wait_for(stage, "wall off the middle")?;
    let wall = stage.arena_mut().add(l.mark(1, 1, room_red()));
    play!(stage; fmn::animation::grow_from_center(stage.arena_mut(), wall, None)?.rt(0.8));
    kit.hold(stage)?;
    let mut mobs = vec![
        grid,
        race.blue_lane,
        race.green_lane,
        race.blue.group,
        race.green.group,
        count,
        bad,
        note_1,
        note_2,
        pointer,
        wall,
        lower,
    ];
    mobs.extend(labels);
    mobs.extend(stairs);
    mobs.extend(ends);
    Ok(SquareMobs { mobs })
}

// --------------------------------------------------------------------- 01_9

/// One tick of the strip: who took the step, and whether it is part of an
/// interleaving (a bad order's middle).
fn strip_ticks(seed: u64, count: usize) -> Vec<(usize, bool)> {
    let mut rng = SplitMix64::new(seed);
    let mut out = Vec::with_capacity(count + 4);
    while out.len() < count {
        let who = (rng.next_u64() % 2) as usize;
        if rng.next_u64() % 100 < 5 {
            // the unlucky timing: read, read, write, write
            out.extend([(who, true), (1 - who, true), (who, true), (1 - who, true)]);
        } else {
            out.extend([(who, false), (who, false)]);
        }
    }
    out
}

/// The strip laid out along a line: each tick's position and colour, and
/// where the red flecks sit (above each interleaving).
struct StripLayout {
    ticks: Vec<(f64, usize)>,
    flecks: Vec<f64>,
}

fn strip_layout(ticks: &[(usize, bool)], pitch: f64) -> StripLayout {
    let mut out = StripLayout {
        ticks: Vec::with_capacity(ticks.len()),
        flecks: Vec::new(),
    };
    let mut x = 0.0;
    let mut k = 0;
    while k < ticks.len() {
        let bad = ticks[k].1;
        let run = if bad { 4 } else { 2 };
        if bad {
            out.flecks.push(x + 1.5 * pitch);
        }
        for &(w, _) in ticks.iter().skip(k).take(run) {
            out.ticks.push((x, w));
            x += pitch;
        }
        k += run;
        x += 0.6 * pitch; // a breath between visits
    }
    out
}

/// 01_9: the orders shrink into one long strip of steps racing past; red
/// flecks where two programs' steps interleave. A fixed pool of ticks is
/// recycled through a window that fades at both ends, so nothing leaves the
/// frame and every frame moves only a few dozen small shapes.
fn strip(kit: &Kit, stage: &mut Stage<'_>, square: SquareMobs) -> fmn::Result<()> {
    let orders = scoreboard_orders();
    let n = orders.len();
    let n_lost = orders.iter().filter(|o| o.end[0] == SCORE_START + 1).count();
    let line = say(kit, stage, "01_9")?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &square.mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, v(0.0, -0.4), 0.3)?;
        anims.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(anims)?;
    let (y, pitch, speed) = (0.4, 0.2, 2.6);
    let (x0, x1, fade) = (-6.7, 6.7, 1.2);
    let layout = strip_layout(&strip_ticks(0x0109, 900), pitch);
    let slots = ((x1 - x0) / pitch) as usize + 4;
    let pool: Vec<Mob> = (0..slots)
        .map(|_| tick(0, 0.0, y).map(|t| stage.arena_mut().add(t)))
        .collect::<fmn::Result<_>>()?;
    let fleck_pool: Vec<Mob> = (0..8)
        .map(|_| {
            let f = Polygon::new([v(-0.13, 0.52), v(0.13, 0.52), v(0.0, 0.32)])
                .style(flat(room_red(), 1.0))
                .build();
            stage.arena_mut().add(f)
        })
        .collect();
    let anchor = stage.add(VMobject::new())?;
    for &m in pool.iter().chain(&fleck_pool) {
        stage.set_opacity(m, 0.0, true);
        stage.add_to_scene(m)?;
    }
    let (pool_u, flecks_u) = (pool.clone(), fleck_pool.clone());
    let mut travelled = 0.0f64;
    let mut shown_as: Vec<Option<usize>> = vec![None; pool.len()];
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            travelled += speed * dt;
            // strip coordinate at the window's left edge: the strip enters
            // from the right edge as the chapter's beat begins
            let left = travelled - (x1 - x0);
            let screen = |sx: f64| x0 + sx - left;
            let alpha = |x: f64| ((x - x0).min(x1 - x) / fade).clamp(0.0, 1.0);
            let first = layout.ticks.partition_point(|&(sx, _)| sx < left);
            for (p, &m) in pool_u.iter().enumerate() {
                match layout.ticks.get(first + p) {
                    Some(&(sx, who)) if sx <= left + (x1 - x0) => {
                        let x = screen(sx);
                        s.move_to(m, v(x, y), ORIGIN);
                        if shown_as[p] != Some(who) {
                            s.set_color(m, customer(who + 1), None, true);
                            shown_as[p] = Some(who);
                        }
                        s.set_opacity(m, 0.95 * alpha(x), true);
                    }
                    _ => {
                        s.set_opacity(m, 0.0, true);
                    }
                }
            }
            let f0 = layout.flecks.partition_point(|&fx| fx < left);
            for (p, &m) in flecks_u.iter().enumerate() {
                match layout.flecks.get(f0 + p) {
                    Some(&fx) if fx <= left + (x1 - x0) => {
                        let x = screen(fx);
                        s.move_to(m, v(x, y + 0.42), ORIGIN);
                        s.set_opacity(m, alpha(x), true);
                    }
                    _ => {
                        s.set_opacity(m, 0.0, true);
                    }
                }
            }
        },
        true,
    )?;
    let note = kit.text(stage, g::STRIP_NOTE, 22.0, Palette::dim(), v(0.0, -0.45))?;
    play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.8));
    line.wait_for(stage, "Each program usually")?;
    let lower = lower_third(
        kit,
        stage,
        g::PRECISELY,
        &format!("{n} {}, {n_lost} {}", g::ORDERS, g::A_COUNT),
    )?;
    play!(stage; fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;
    stage.clear_updaters(anchor, true);
    Ok(())
}

/// One step on the strip, in its program's colour.
fn tick(who: usize, x: f64, y: f64) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(0.13)
        .height(0.42)
        .corner_radius(0.04)
        .style(flat(customer(who + 1), 0.95))
        .build()?
        .moved_to(v(x, y)))
}
