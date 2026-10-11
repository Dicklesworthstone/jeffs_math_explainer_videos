//! Chapter 09, reading garbage safely. 01_1's slate comes back: rewritten
//! one digit at a time, a glance between the two digit writes reads 19 or
//! 00 (09_1). Lamport went further: a glance during a write may return any
//! value the slate can hold, even 2496 (09_2), and twelve years later he
//! named memory like that "safe", the weakest of three kinds (09_3). Then
//! 08's proof picture, same layout and scale: every write flickers while it
//! lasts, and a glance that crosses a write turns to white static as Green's
//! choosing slides through the three places; in the listen window a glance
//! that crosses a write smears its note (09_4). The proof trusts only two
//! glances at slates, Green's at Blue's finished ticket ("not yet") and
//! Blue's at Green's finished ticket ("done"); no write overlaps either, and
//! the static only lands on glances it never uses (09_5). Trace e1 is the
//! "done" place with Green's glance caught mid-write: she reads 7, takes 8
//! and waits (09_6); the model check with flickering memory agrees.
//!
//! "Garbage" (description, "Precisely" 09): a read that overlaps a write
//! may return any value the slate can hold; a read that overlaps none
//! returns the last value written. On the picture a write lasts a while (an
//! interval on its writer's lane) and a glance is a moment; a glance
//! crosses a write when its moment falls inside the other customer's write
//! of the slate or flag it reads.

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, tween, v};
use explainer_score::instr::{Note, render, shape_edges};
use explainer_score::score::Identity;
use explainer_score::units::midi_to_hz;
use explainer_score::{FS as SAMPLE_RATE, Stereo};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{
    Spoken, board, chalk, customer, fade_out_all, faded_line, fail, flag_amber, flat, group, hardware, order_gold,
    quote_card, say, static_burst, wait_until,
};
use super::glyphs;
use super::tunes::{self, timbre};
use super::why_never_two::{
    BLUE_CHOOSING, GREEN_W, Place, ProofLanes, X_CHECK, X_GLANCE, X_ROOM, X0, X1, Y_BLUE, Y_GREEN,
};
use crate::facts::{cited, recorded_sweep};
use crate::model::{Step, Trace};

// ------------------------------------------------------------------ 09_1

/// The two digits of `n` as a two-digit slate shows them.
fn digits(n: u32) -> [u32; 2] {
    [n / 10 % 10, n % 10]
}

/// What a glance between the two digit writes reads when the slate goes
/// from `old` to `new` and digit `first` (0 tens, 1 ones) is written first.
fn half_written(old: u32, new: u32, first: usize) -> [u32; 2] {
    let mut d = digits(old);
    d[first] = digits(new)[first];
    d
}

/// A two-digit slate: a board and one chalk numeral per digit.
struct TwoDigit {
    board: Mob,
    digits: [Mob; 2],
}

/// A slate `size` tall at `at` showing `d` as two digits (detached).
fn two_digit(kit: &Kit, stage: &mut Stage<'_>, d: [u32; 2], at: Vec3, size: f64) -> fmn::Result<TwoDigit> {
    let b = Rectangle::new()
        .width(1.4 * size)
        .height(size)
        .corner_radius(0.12 * size)
        .style(Style::default().fill(board(), 1.0).stroke(hardware(), 2.5, 1.0))
        .build()?
        .moved_to(at);
    let board = stage.arena_mut().add(b);
    let mut ms = Vec::new();
    for x in d {
        ms.push(kit.text(stage, &x.to_string(), 62.0 * size, chalk(), at)?);
    }
    let w = ms.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
    let dx = 0.5 * w + 0.07 * size;
    stage.move_to(ms[0], v(at[0] - dx, at[1]), ORIGIN);
    stage.move_to(ms[1], v(at[0] + dx, at[1]), ORIGIN);
    Ok(TwoDigit {
        board,
        digits: [ms[0], ms[1]],
    })
}

/// An eye (01_1's glance), its iris and pupil turned towards `look`.
struct Eye {
    white: Mob,
    iris: Mob,
}

fn eye(stage: &mut Stage<'_>, at: Vec3, look: Vec3) -> fmn::Result<Eye> {
    let white = Ellipse::new()
        .width(0.9)
        .height(0.5)
        .style(flat(chalk(), 1.0))
        .build()
        .moved_to(at);
    let white = stage.arena_mut().add(white);
    let iris = stage.arena_mut().add(iris_at(at, look));
    Ok(Eye { white, iris })
}

/// Iris and pupil of an eye at `at` looking towards `look`.
fn iris_at(at: Vec3, look: Vec3) -> VMobject {
    let (dx, dy) = (look[0] - at[0], look[1] - at[1]);
    let n = dx.hypot(dy).max(1e-9);
    let c = v(at[0] + 0.15 * dx / n, at[1] + 0.08 * dy / n);
    let iris = Circle::new()
        .radius(0.17)
        .arc_center(c)
        .style(flat(Palette::dim(), 1.0))
        .build();
    let pupil = Circle::new()
        .radius(0.09)
        .arc_center(c)
        .style(flat(board(), 1.0))
        .build();
    v_group([iris, pupil])
}

/// A chalk pulse on `m`: it swells and settles, keeping its colour
/// (`indicate`'s default colour would flood a filled shape with yellow).
fn pulse(stage: &mut Stage<'_>, m: Mob) -> fmn::Result<impl Animation + use<>> {
    Ok(indicate(stage.arena_mut(), m, 1.2, Some(super::draw::rgb(chalk())))?)
}

/// A dashed line of sight from an eye to what it reads.
fn sight(from: Vec3, to: Vec3) -> fmn::Result<VMobject> {
    Ok(DashedLine::new(from, to)
        .dash_length(0.12)
        .style(faded_line(chalk(), 2.0, 0.6))
        .build()?)
}

/// Static that crackles: a [`static_burst`] at `at`, redrawn from the
/// chapter's clock about [`FLICKER_HZ`] times a second. In the scene.
fn crackle(stage: &mut Stage<'_>, clock: Mob, at: Vec3, size: f64, seed: u64) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let t = s.tracker_value(clock).unwrap_or(0.0);
        let tick = (t * FLICKER_HZ).floor() as u64;
        s.add(static_burst(at, size, seed.wrapping_mul(1_000).wrapping_add(tick)))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The rows of 09_1: the slate rewritten tens digit first, and ones digit
/// first; the middle slate of each is what a glance in between reads.
const ROW_Y: [f64; 2] = [1.15, -1.45];
const COL_X: [f64; 3] = [-3.6, 0.0, 3.6];
const ROW_SLATE: f64 = 1.1;

/// 09_1: 01_1's slate caught mid-stroke, then the rewrite one digit at a
/// time in both orders, and what a glance in between reads. Returns what
/// stays for 09_2's wipe.
fn digit_by_digit(kit: &Kit, stage: &mut Stage<'_>, clock: Mob) -> fmn::Result<Vec<Mob>> {
    let (from, to) = (cited::TORN_FROM, cited::TORN_TO);
    let line = say(kit, stage, "09_1")?;
    // 01_1's slate: nine rubbed out, ten half drawn
    let at = v(0.0, 0.5);
    let size = 2.0;
    let old = super::draw::slate(kit, stage, from, at, size)?;
    play!(stage;
        fade_in(stage.arena_mut(), old.board, ORIGIN, 1.0)?.rt(0.5),
        write(stage.arena(), old.value).rt(0.6),
    );
    let ten = super::draw::chalk_number(kit, stage, to, at, size)?;
    let mut half = write(stage.arena(), ten);
    half.update_rate_info(Some(1.1), Some(RateFunc::smooth().not_quite_there(0.5)), None);
    let rub = old
        .value
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.1),
            ..AnimateArgs::default()
        })?
        .set_opacity(0.35)?;
    play!(stage; half, rub);

    // one digit at a time, both orders
    line.wait_for(stage, "Rewrite oh-nine")?;
    let mut rows = Vec::new();
    for &y in &ROW_Y {
        rows.push(two_digit(kit, stage, digits(from), v(COL_X[0], y), ROW_SLATE)?);
    }
    let mut first_in: Vec<Box<dyn Animation>> = Vec::new();
    for m in [old.board, old.value, ten] {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.5);
        first_in.push(stage.prepare(a)?);
    }
    for row in &rows {
        for m in [row.board, row.digits[0], row.digits[1]] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
            first_in.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared(first_in)?;
    let mut keep: Vec<Mob> = rows.iter().flat_map(|r| [r.board, r.digits[0], r.digits[1]]).collect();
    let mut middles = Vec::new();
    // step 1 writes digit `first`; step 2 the other one
    for step in 0..2 {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (r, &y) in ROW_Y.iter().enumerate() {
            let first = r;
            let written = if step == 0 { first } else { 1 - first };
            let shown = if step == 0 {
                half_written(from, to, first)
            } else {
                digits(to)
            };
            let (xa, xb) = (COL_X[step], COL_X[step + 1]);
            let w = 0.7 * ROW_SLATE;
            let arrow = Arrow::new(v(xa + w + 0.1, y), v(xb - w - 0.1, y))
                .buff(0.0)
                .thickness(2.5)
                .style(
                    Style::default()
                        .fill(Palette::dim(), 1.0)
                        .stroke(Palette::dim(), 2.5, 1.0),
                )
                .build()?;
            let arrow = stage.arena_mut().add(arrow);
            let label = [glyphs::TENS_DIGIT, glyphs::ONES_DIGIT][written];
            let label = kit.text(stage, label, 20.0, Palette::dim(), v((xa + xb) / 2.0, y + 0.36))?;
            let next = two_digit(kit, stage, shown, v(xb, y), ROW_SLATE)?;
            let grow = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(0.6);
            anims.push(stage.prepare(grow)?);
            for m in [label, next.board, next.digits[1 - written]] {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
                anims.push(stage.prepare(a)?);
            }
            let w = write(stage.arena(), next.digits[written]).rt(0.9);
            anims.push(stage.prepare(w)?);
            keep.extend([arrow, label, next.board, next.digits[0], next.digits[1]]);
            if step == 0 {
                middles.push(next);
            }
        }
        stage.play_prepared(anims)?;
    }

    // a glance in between: nineteen (tens first), or zero zero (ones first)
    let eye_at = v(-1.8, -0.15);
    let targets: Vec<Vec3> = ROW_Y
        .iter()
        .map(|&y| {
            let edge = if y > eye_at[1] { -0.5 } else { 0.5 };
            v(COL_X[1] - 0.45 * ROW_SLATE, y + edge * ROW_SLATE)
        })
        .collect();
    let the_eye = eye(stage, eye_at, targets[0])?;
    play!(stage;
        fade_in(stage.arena_mut(), the_eye.white, ORIGIN, 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), the_eye.iris, ORIGIN, 1.0)?.rt(0.4),
    );
    keep.push(the_eye.white);
    let mut iris = the_eye.iris;
    for (r, (phrase, target)) in ["reads nineteen", "or zero zero"].into_iter().zip(&targets).enumerate() {
        line.wait_for(stage, phrase)?;
        let from_eye = v(eye_at[0] + 0.35, eye_at[1] + if r == 0 { 0.15 } else { -0.15 });
        let look = stage.arena_mut().add(sight(from_eye, *target)?);
        let turned = stage.arena_mut().add(iris_at(eye_at, *target));
        let m = &middles[r];
        play!(stage;
            show_creation(look).rt(0.4),
            fade_out(stage.arena_mut(), iris, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), turned, ORIGIN, 1.0)?.rt(0.3),
            pulse(stage, m.digits[0])?.rt(0.8),
            pulse(stage, m.digits[1])?.rt(0.8),
        );
        iris = turned;
        // the glance crosses the write: static on the line of sight
        let mid = v((from_eye[0] + target[0]) / 2.0, (from_eye[1] + target[1]) / 2.0);
        let fx = crackle(stage, clock, mid, 0.42, 0x0901 + r as u64)?;
        keep.extend([look, fx]);
    }
    keep.push(iris);
    kit.hold(stage)?;
    Ok(keep)
}

// ------------------------------------------------------------------ 09_2

/// The values 09_2's glance flickers through (every one a value the facts
/// hold), ending on Lamport's 2496: the slate's old and new values, the
/// two torn reads of 09_1, trace e1's garbage and the ticket it led to.
fn wild_values(e: &E1) -> Vec<String> {
    let (from, to) = (cited::TORN_FROM, cited::TORN_TO);
    let torn = |first| {
        let d = half_written(from, to, first);
        format!("{}{}", d[0], d[1])
    };
    vec![
        from.to_string(),
        to.to_string(),
        torn(0),
        torn(1),
        e.garbage.to_string(),
        e.green_ticket.to_string(),
        cited::TORN_VALUE.to_string(),
    ]
}

/// 09_2: a glance during a write returns any value the slate can hold,
/// even 2496 (01_1's quote returns).
fn any_value(kit: &Kit, stage: &mut Stage<'_>, clock: Mob, gone: &[Mob], e: &E1) -> fmn::Result<Vec<Mob>> {
    let line = say(kit, stage, "09_2")?;
    fade_out_all(stage, gone, 0.5)?;
    // the slate mid-write: nine rubbed out, ten half drawn
    let at = v(-3.6, 0.85);
    let size = 1.6;
    let s = super::draw::slate(kit, stage, cited::TORN_FROM, at, size)?;
    stage.set_opacity(s.value, 0.35, true);
    let ten = super::draw::chalk_number(kit, stage, cited::TORN_TO, at, size)?;
    let mut half = write(stage.arena(), ten);
    half.update_rate_info(Some(0.9), Some(RateFunc::smooth().not_quite_there(0.5)), None);
    play!(stage;
        fade_in(stage.arena_mut(), s.board, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), s.value, ORIGIN, 1.0)?.rt(0.5),
        half,
    );
    // the glance, crossing the write, and what it reads
    line.wait_for(stage, "A glance during")?;
    let read_at = v(3.0, 0.85);
    let read_w = 3.4;
    let reading = Rectangle::new()
        .width(read_w)
        .height(size)
        .corner_radius(0.12 * size)
        .style(Style::default().fill(board(), 1.0).stroke(hardware(), 2.5, 1.0))
        .build()?
        .moved_to(read_at);
    let reading = stage.arena_mut().add(reading);
    let (a, b) = (
        v(at[0] + 0.7 * size + 0.12, at[1]),
        v(read_at[0] - read_w / 2.0 - 0.12, at[1]),
    );
    let glance = stage.arena_mut().add(sight(a, b)?);
    play!(stage;
        show_creation(glance).rt(0.5),
        fade_in(stage.arena_mut(), reading, ORIGIN, 1.0)?.rt(0.5),
    );
    let fx = crackle(stage, clock, v((a[0] + b[0]) / 2.0, a[1]), 0.6, 0x0902)?;
    let values = wild_values(e);
    let mut numerals = Vec::new();
    for x in &values {
        numerals.push(kit.text(stage, x, 62.0 * size, chalk(), read_at)?);
    }
    let last = values.len() - 1;
    line.wait_for(stage, "any number")?;
    super::draw::flicker(stage, &numerals, 2.6, 9.0, 4, 0x0921)?;
    line.wait_for(stage, "Even two thousand")?;
    super::draw::flicker(stage, &numerals, 0.8, 12.0, last, 0x0922)?;
    let words = format!(
        "“{} {} {} {} {} {}.”",
        glyphs::TORN_A,
        cited::TORN_FROM,
        glyphs::TORN_B,
        cited::TORN_TO,
        glyphs::TORN_C,
        cited::TORN_VALUE
    );
    let quote = quote_card(kit, stage, &words, glyphs::TURING_SOURCE, v(0.0, -2.35), 66)?;
    play!(stage; fade_in(stage.arena_mut(), quote, v(0.0, 0.15), 1.0)?.rt(0.8));
    kit.hold(stage)?;
    let mut out = vec![s.board, s.value, ten, glance, reading, fx, quote];
    out.extend(numerals);
    Ok(out)
}

// ------------------------------------------------------------------ 09_3

/// 09_3: twelve years on, "safe" memory, with regular and atomic above it.
fn three_kinds(kit: &Kit, stage: &mut Stage<'_>, gone: &[Mob]) -> fmn::Result<Vec<Mob>> {
    let line = say(kit, stage, "09_3")?;
    fade_out_all(stage, gone, 0.5)?;
    let y = 3.2;
    let then = kit.text(stage, &cited::BAKERY_YEAR.to_string(), 34.0, Palette::dim(), v(-1.3, y))?;
    let now = kit.text(stage, &cited::INTERPROCESS_YEAR.to_string(), 34.0, chalk(), v(1.3, y))?;
    let arrow = Arrow::new(v(-0.55, y), v(0.55, y))
        .buff(0.0)
        .thickness(2.5)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 2.5, 1.0),
        )
        .build()?;
    let arrow = stage.arena_mut().add(arrow);
    play!(stage;
        fade_in(stage.arena_mut(), then, ORIGIN, 1.0)?.rt(0.5),
        fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(0.6),
        fade_in(stage.arena_mut(), now, ORIGIN, 1.0)?.rt(0.8),
    );

    // the three kinds, strongest on top; safe first
    let rows = [
        (glyphs::ATOMIC_KIND, glyphs::ATOMIC_READ, 1.55),
        (glyphs::REGULAR_KIND, glyphs::REGULAR_READ, 0.6),
        (glyphs::SAFE_KIND, glyphs::SAFE_READ, -0.35),
    ];
    let mut built = Vec::new();
    let mut safe_name = None;
    for (i, &(name, read, ry)) in rows.iter().enumerate() {
        let safe = i == rows.len() - 1;
        let back = Rectangle::new()
            .width(11.7)
            .height(0.74)
            .corner_radius(0.1)
            .style(Style::default().fill(explainer_kit::hex("#1A2130"), 0.94).stroke(
                if safe { chalk() } else { Palette::faint() },
                if safe { 2.5 } else { 1.5 },
                1.0,
            ))
            .build()?
            .moved_to(v(-0.3, ry));
        let back = stage.arena_mut().add(back);
        let n = kit.text_left(
            stage,
            name,
            30.0,
            if safe { chalk() } else { Palette::dim() },
            v(-5.85, ry),
        )?;
        let r = kit.text_left(
            stage,
            read,
            25.0,
            if safe { chalk() } else { Palette::dim() },
            v(-3.75, ry),
        )?;
        if safe {
            safe_name = Some(n);
        }
        built.push(group(stage, &[back, n, r])?);
    }
    let safe_name = safe_name.ok_or_else(|| fail("09_3: no safe row".to_owned()))?;
    let head = kit.text_left(stage, glyphs::OVERLAPPING_READ, 25.0, Palette::dim(), v(-5.85, 2.45))?;
    let every = kit.text_left(stage, glyphs::NO_OVERLAP, 22.0, Palette::dim(), v(-5.85, -1.3))?;
    let source = kit.text_left(
        stage,
        &format!("{} {}", glyphs::INTERPROCESS_SOURCE, cited::INTERPROCESS_YEAR),
        22.0,
        Palette::dim(),
        v(-5.85, -1.85),
    )?;
    line.wait_for(stage, "named memory")?;
    play!(stage;
        fade_in(stage.arena_mut(), built[2], v(0.0, -0.1), 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), head, ORIGIN, 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), every, ORIGIN, 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), source, ORIGIN, 1.0)?.rt(0.7),
    );
    let up = Arrow::new(v(6.15, -0.6), v(6.15, 1.8))
        .buff(0.0)
        .thickness(2.5)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 2.5, 1.0),
        )
        .build()?;
    let up = stage.arena_mut().add(up);
    let stronger = kit.text(stage, glyphs::STRONGER, 20.0, Palette::dim(), v(6.15, 2.1))?;
    line.wait_for(stage, "A funny name")?;
    play!(stage;
        fade_in(stage.arena_mut(), built[1], v(0.0, -0.1), 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), built[0], v(0.0, -0.1), 1.0)?.rt(0.9),
        fmn::animation::grow_arrow(stage.arena_mut(), up)?.rt(0.9),
        fade_in(stage.arena_mut(), stronger, ORIGIN, 1.0)?.rt(0.9),
    );
    line.wait_for(stage, "the least safe")?;
    play!(stage; pulse(stage, safe_name)?.rt(1.0));
    kit.hold(stage)?;
    let mut out = vec![then, now, arrow, head, every, source, up, stronger];
    out.extend(built);
    Ok(out)
}

// ------------------------------------------------------- the proof zones

/// How long a write lasts on the picture (the time axis's units).
const WRITE_W: f64 = 0.3;
/// Blue's glance at Green's slate while she chooses (before her own ticket
/// write, and before Green raises her flag in the "done" place below).
const X_BLUE_MAX: f64 = -4.9;
/// Where Green's choosing parks in "done" for 09_5 and 09_6: her glance at
/// Blue's slate lands inside Blue's ticket write, trace e1's shape.
const DONE_E1: f64 = -4.55;
/// Changes a second of a write's flicker and of static.
const FLICKER_HZ: f64 = 12.0;
/// The listen window's playhead runs from the lanes' start to Blue's
/// walking in, in this long.
const HEAD_S: f64 = 3.0;
/// The listen window runs this long past the sweep; its last part fades.
const LISTEN_TAIL: f64 = 0.6;
const LISTEN_FADE: f64 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Var {
    Slate,
    Flag,
}

/// A write on the picture: whose (1 Blue, 2 Green), what, and when.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Write {
    who: usize,
    var: Var,
    x0: f64,
    x1: f64,
}

/// A glance on the picture: who reads, which of the other's variables, when.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Read {
    who: usize,
    var: Var,
    x: f64,
}

/// Customer `k`'s lane height.
fn lane_y(k: usize) -> f64 {
    if k == 1 { Y_BLUE } else { Y_GREEN }
}

/// Green's glance at Blue's slate while she chooses, her choosing starting
/// at `s` (where 08 puts it: `ProofLanes::green_read_x`).
fn green_read(s: f64) -> f64 {
    s + 0.55 * GREEN_W
}

/// Every write on the picture with Green's choosing starting at `s`: each
/// customer raises her flag just before her choosing band (the band is her
/// flag up), finishes her ticket write at 08's ticket tick, and lowers her
/// flag just after the band.
fn writes(s: f64) -> [Write; 6] {
    let one = |who: usize, start: f64, end: f64| {
        let tick = end - 0.25;
        [
            Write {
                who,
                var: Var::Flag,
                x0: start - WRITE_W,
                x1: start,
            },
            Write {
                who,
                var: Var::Slate,
                x0: tick - WRITE_W,
                x1: tick,
            },
            Write {
                who,
                var: Var::Flag,
                x0: end,
                x1: end + WRITE_W,
            },
        ]
    };
    let [a, b, c] = one(1, BLUE_CHOOSING.0, BLUE_CHOOSING.1);
    let [d, e, f] = one(2, s, s + GREEN_W);
    [a, b, c, d, e, f]
}

/// The glances the picture shows: Blue's at Green's slate while choosing,
/// Green's at Blue's slate while choosing (08_2's), Blue's last look at
/// Green's flag (08_4's) and her later glance at Green's slate (08_7's).
fn reads(s: f64) -> [Read; 4] {
    [
        Read {
            who: 1,
            var: Var::Slate,
            x: X_BLUE_MAX,
        },
        Read {
            who: 2,
            var: Var::Slate,
            x: green_read(s),
        },
        Read {
            who: 1,
            var: Var::Flag,
            x: X_CHECK,
        },
        Read {
            who: 1,
            var: Var::Slate,
            x: X_GLANCE,
        },
    ]
}

/// The write a glance crosses, if any: the other customer's write of what
/// it reads, in progress at the glance.
fn crossed(r: &Read, ws: &[Write]) -> Option<Write> {
    ws.iter()
        .copied()
        .find(|w| w.who != r.who && w.var == r.var && w.x0 <= r.x && r.x <= w.x1)
}

#[cfg(test)]
/// The glance at a slate the proof uses in each place (description,
/// "Precisely" 08–09): in "not yet", Green's at Blue's ticket (fact one);
/// in "done", Blue's later glance at Green's ticket; none in "choosing"
/// (Blue's look at the raised flag would have stopped her).
fn trusted(place: Place, s: f64) -> Option<Read> {
    match place {
        Place::NotYet => Some(reads(s)[1]),
        Place::Choosing => None,
        Place::Done => Some(reads(s)[3]),
    }
}

/// A hashed jitter in [0, 1) for flicker `tick` of mark `i`.
fn jitter(tick: u64, i: u64) -> f64 {
    let mut x = tick.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ i.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    x ^= x >> 29;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 32;
    (x % 1000) as f64 / 1000.0
}

/// An arrow for a glance, `o` opaque.
fn glance_arrow(from: Vec3, to: Vec3, c: Srgb, w: f64, o: f64) -> fmn::Result<VMobject> {
    Ok(Arrow::new(from, to)
        .buff(0.08)
        .thickness(w)
        .style(Style::default().fill(c, o).stroke(c, w, o))
        .build()?)
}

/// A glance's arrow ends: from the lane of what is read to the reader's.
fn glance_ends(r: &Read) -> (Vec3, Vec3) {
    let (yo, yr) = (lane_y(3 - r.who), lane_y(r.who));
    let dir = if yr > yo { 1.0 } else { -1.0 };
    (v(r.x, yo + dir * 0.2), v(r.x, yr - dir * 0.2))
}

/// The overlay on 08's picture, redrawn every frame from trackers: the
/// writes flickering, the two glances 08 draws only in passing (Blue's and
/// Green's while choosing), static on every glance that crosses a write,
/// gold on the glances the proof trusts, and the listen window's playhead.
struct Overlay {
    group: Mob,
    reads: Mob,
    writes: Mob,
    gold_green: Mob,
    gold_blue: Mob,
    head: Mob,
    head_alpha: Mob,
}

fn overlay(stage: &mut Stage<'_>, green_start: Mob, clock: Mob) -> fmn::Result<Overlay> {
    let reads_a = stage.add_value_tracker(0.0);
    let writes_a = stage.add_value_tracker(0.0);
    let gold_green = stage.add_value_tracker(0.0);
    let gold_blue = stage.add_value_tracker(0.0);
    let head = stage.add_value_tracker(X0);
    let head_alpha = stage.add_value_tracker(0.0);
    let group = stage.always_redraw(move |s| {
        let get = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
        let st = get(green_start);
        let tick = (get(clock) * FLICKER_HZ).floor() as u64;
        let (ra, wa) = (get(reads_a).clamp(0.0, 1.0), get(writes_a).clamp(0.0, 1.0));
        let (gg, gb) = (get(gold_green).clamp(0.0, 1.0), get(gold_blue).clamp(0.0, 1.0));
        let (hx, ha) = (get(head), get(head_alpha).clamp(0.0, 1.0));
        let mid = (Y_BLUE + Y_GREEN) / 2.0;
        let ws = writes(st);
        let rs = reads(st);
        let mut ms: Vec<VMobject> = Vec::new();
        if wa > 0.0 {
            for (i, w) in ws.iter().enumerate() {
                let c = match w.var {
                    Var::Slate => chalk(),
                    Var::Flag => flag_amber(),
                };
                let o = wa * (0.2 + 0.6 * jitter(tick, i as u64));
                if let Ok(r) = Rectangle::new()
                    .width(w.x1 - w.x0)
                    .height(0.46)
                    .style(flat(c, o))
                    .build()
                {
                    ms.push(r.moved_to(v((w.x0 + w.x1) / 2.0, lane_y(w.who))));
                }
            }
        }
        for (i, r) in rs.iter().enumerate() {
            let (a, b) = glance_ends(r);
            // the look and the later glance are 08's own arrows
            if i < 2
                && ra > 0.0
                && let Ok(m) = glance_arrow(a, b, customer(r.who), 3.5, ra)
            {
                ms.push(m);
            }
            if ra > 0.5 && crossed(r, &ws).is_some() {
                ms.push(static_burst(
                    v(r.x, mid),
                    0.5,
                    tick.wrapping_mul(8).wrapping_add(i as u64),
                ));
            }
        }
        for (o, r) in [(gg, rs[1]), (gb, rs[3])] {
            let (a, b) = glance_ends(&r);
            if o > 0.0
                && let Ok(m) = glance_arrow(a, b, order_gold(), 4.5, o)
            {
                ms.push(m);
            }
        }
        if ha > 0.0 {
            if let Ok(l) = Line::new(v(hx, Y_BLUE + 0.5), v(hx, Y_GREEN - 0.5))
                .style(faded_line(chalk(), 2.0, 0.55 * ha))
                .build()
            {
                ms.push(l);
            }
            for r in &rs {
                let d = (hx - r.x).abs();
                if d < 0.35 {
                    let c = if crossed(r, &ws).is_some() {
                        chalk()
                    } else {
                        customer(r.who)
                    };
                    ms.push(
                        Circle::new()
                            .radius(0.26)
                            .arc_center(v(r.x, mid))
                            .style(faded_line(c, 3.0, ha * (1.0 - d / 0.35)))
                            .build(),
                    );
                }
            }
        }
        s.add(v_group(ms))
    });
    stage.add_to_scene(group)?;
    Ok(Overlay {
        group,
        reads: reads_a,
        writes: writes_a,
        gold_green,
        gold_blue,
        head,
        head_alpha,
    })
}

// ------------------------------------------------------------- the listen

/// When the playhead reaches `x` (seconds into the listen window).
fn head_t(x: f64) -> f64 {
    HEAD_S * (x - X0) / (X_ROOM - X0)
}

/// A glance's note: Blue's at F, her look at a flag at B♭ (a flag is a yes
/// or no), Green's at D (MIDI, over the bakery's B♭ tonic).
fn read_pitch(r: &Read, tonic: i32) -> i32 {
    match (r.who, r.var) {
        (1, Var::Flag) => tonic + 12,
        (1, Var::Slate) => tonic + 7,
        _ => tonic + 4,
    }
}

/// A note that smears: two slightly detuned plucked partials sagging a
/// whole step from `from` to `to` (MIDI) like slowing tape, `secs` long.
fn smeared(from: f64, to: f64, secs: f64) -> Vec<f64> {
    let n = (secs * SAMPLE_RATE).round() as usize;
    let mut phase = [0.0f64; 6];
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / SAMPLE_RATE;
        let u = (t / secs).clamp(0.0, 1.0);
        let bend = u * u * (3.0 - 2.0 * u);
        let f = midi_to_hz(from + (to - from) * bend);
        let env = (-2.4 * t).exp();
        let mut y = 0.0;
        for (k, (mult, amp)) in [(1.0, 1.0), (2.0, 0.32), (3.0, 0.1)].into_iter().enumerate() {
            // the two voices drift apart as the tape slows: the smear
            for (j, cents) in [(0, -1.0), (1, 1.0)] {
                let detune = 2f64.powf(cents * (2.0 + 10.0 * bend) / 1200.0);
                let p = &mut phase[2 * k + j];
                *p += std::f64::consts::TAU * f * mult * detune / SAMPLE_RATE;
                y += amp * p.sin();
            }
        }
        out.push(env * y / 2.84);
    }
    shape_edges(&mut out, 0.008);
    out
}

/// 09_4's listen window: the playhead's glances in time order, each a soft
/// note in its reader's timbre; the glance that crosses a write smears.
fn listen_sound(s: f64) -> Stereo {
    let tonic = Identity::bakery_algorithm().tonic_midi;
    let ws = writes(s);
    let mut parts = Vec::new();
    for (i, r) in reads(s).iter().enumerate() {
        let at = head_t(r.x);
        let pitch = f64::from(read_pitch(r, tonic));
        let sound = if crossed(r, &ws).is_some() {
            Stereo::from_mono(&smeared(pitch, pitch - 2.0, 1.2))
        } else {
            render(timbre(r.who), &Note::new(pitch, 0.6, 0.3, 0x0940 + i as u64))
        };
        parts.push((sound, at));
    }
    // the sweep and a short tail: the last notes' rings fade out with it
    let n = ((HEAD_S + LISTEN_TAIL) * SAMPLE_RATE).round() as usize;
    let mut out = Stereo::silence(n);
    for (s, t) in &parts {
        out.mix_in(s, (t * SAMPLE_RATE).round() as usize, 1.0);
    }
    let fade = (LISTEN_FADE * SAMPLE_RATE).round() as usize;
    for k in 0..fade {
        let g = 0.5 + 0.5 * (std::f64::consts::PI * (k + 1) as f64 / fade as f64).cos();
        let i = n - fade + k;
        out.l[i] *= g;
        out.r[i] *= g;
    }
    out
}

// --------------------------------------------------------------- trace e1

/// What 09_6 shows of trace e1: the garbage Green's glance returned, Blue's
/// slate before and after the write it crossed, the ticket Green then took,
/// what Blue's later glance read, and whether Green ends up waiting.
#[derive(Clone, Debug, PartialEq, Eq)]
struct E1 {
    garbage: u32,
    blue_old: u32,
    blue_new: u32,
    green_ticket: u32,
    blue_saw: u32,
    green_waits: bool,
}

/// The value a step's label ends with (`…=7`, `…:=8`, `…=8, pass`).
fn value_of(label: &str) -> Option<u32> {
    let (_, rest) = label.rsplit_once('=')?;
    rest.chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// The 0-based index of `who`'s first step starting with `pat`.
fn index_of(steps: &[Step], who: usize, pat: &str) -> fmn::Result<usize> {
    steps
        .iter()
        .position(|s| s.customer == who && s.label.starts_with(pat))
        .ok_or_else(|| fail(format!("09: trace e1 has no step {pat:?} by customer {who}")))
}

/// Read trace e1 the way 09_6 tells it, checking it has the "done" shape:
/// Green's glance at Blue's slate falls inside Blue's ticket write; Green
/// finishes choosing before Blue's last look at her flag; Blue's later
/// glance at Green's slate comes after Green's write ended.
fn e1(steps: &[Step]) -> fmn::Result<E1> {
    let begin = index_of(steps, 1, "begin number[1]:=")?;
    let end = index_of(steps, 1, "end number[1]:=")?;
    let glance = index_of(steps, 2, "read number[1]=")?;
    let green_lowered = index_of(steps, 2, "end choosing:=0")?;
    let green_wrote = index_of(steps, 2, "end number[2]:=")?;
    let look = index_of(steps, 1, "L2: choosing[2]")?;
    let blue_glance = index_of(steps, 1, "L3: number[2]=")?;
    if !(begin < glance && glance < end) {
        return Err(fail("09_6: Green's glance should cross Blue's write".to_owned()));
    }
    if !(green_lowered < look && green_wrote < blue_glance) {
        return Err(fail("09_6: trace e1 should be the \"done\" place".to_owned()));
    }
    let parse =
        |i: usize| value_of(&steps[i].label).ok_or_else(|| fail(format!("09: no value in {:?}", steps[i].label)));
    let green_ticket = steps
        .iter()
        .filter_map(Step::ticket_written)
        .find(|&(p, _)| p == 2)
        .map(|(_, t)| t)
        .ok_or_else(|| fail("09_6: Green takes no ticket in e1".to_owned()))?;
    let green_waits = steps
        .last()
        .is_some_and(|s| s.customer == 2 && s.blocked() && s.label.starts_with("L3: number[1]="));
    Ok(E1 {
        garbage: parse(glance)?,
        blue_old: steps[begin].before.number[0],
        blue_new: parse(begin)?,
        green_ticket,
        blue_saw: parse(blue_glance)?,
        green_waits,
    })
}

/// The model check of 09_6's corner card: customers and the largest ticket
/// (the key the facts pin names them, so a wrong pair fails `facts-check`).
const SAFE_SEARCH: (usize, u32) = (3, 6);

// ------------------------------------------------------------- chapter 09

pub struct ReadingGarbage {
    pub kit: Kit,
}

impl SceneConstruct for ReadingGarbage {
    fn name(&self) -> &str {
        "09_reading_garbage_safely"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let trace = e1(&Trace::E1.steps())?;
        // the chapter's clock, for flicker and static
        let clock = stage.add_value_tracker(0.0);
        let anchor = stage.add(VMobject::new())?;
        let mut t = 0.0;
        stage.add_dt_updater(
            anchor,
            move |s, _, dt| {
                t += dt;
                let _ = s.set_tracker_value(clock, t);
            },
            false,
        )?;

        // 09_1–09_3: torn reads, any value, safe memory
        let gone = digit_by_digit(&kit, stage, clock)?;
        let gone = any_value(&kit, stage, clock, &gone, &trace)?;
        let gone = three_kinds(&kit, stage, &gone)?;

        // 09_4: back to 08's picture; every write flickers, glances that
        // cross a write turn to static as Green's choosing slides
        let line = say(&kit, stage, "09_4")?;
        fade_out_all(stage, &gone, 0.5)?;
        let pic = ProofLanes::build(&kit, stage)?;
        play!(stage;
            fade_in(stage.arena_mut(), pic.lanes, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), pic.blue_choosing, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), pic.blue_room, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), pic.look, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), pic.look_label, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), pic.glance, ORIGIN, 1.0)?.rt(0.9),
            tween(pic.green_alpha, 1.0, 0.9)?,
        );
        let over = overlay(stage, pic.green_start, clock)?;
        play!(stage; tween(over.reads, 1.0, 0.6)?);
        line.wait_for(stage, "Let every slate")?;
        play!(stage; tween(over.writes, 1.0, 0.8)?);
        line.wait_for(stage, "Does the proof")?;
        stage.play(pic.slide(DONE_E1, 4.0)?)?;
        let q = predict_pause(&kit, stage, v(5.2, 2.6))?;
        // listen: the playhead sounds each glance; the crossed one smears
        let (wav, secs) = tunes::write("09_4", listen_sound(DONE_E1)).map_err(|e| fail(e.to_string()))?;
        kit.sonify(stage, secs, "09_4", &wav)?;
        let t0 = stage.scene().time().to_f64();
        play!(stage; tween(over.head_alpha, 1.0, 0.15)?);
        let sweep = tween(over.head, X_ROOM, HEAD_S - 0.15)?;
        let sweep = stage.prepare(sweep)?;
        stage.play_prepared_with(
            vec![sweep],
            PlayOverrides {
                run_time: Some(HEAD_S - 0.15),
                rate_func: Some(RateFunc::linear()),
                ..PlayOverrides::default()
            },
        )?;
        play!(stage; tween(over.head_alpha, 0.0, 0.3)?);
        wait_until(stage, t0 + secs)?;

        // 09_5: never; only two trusted glances, each at a finished ticket
        let line = say(&kit, stage, "09_5")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        // to "not yet" while the voice counts the two, so each gold glance
        // lands on its own words and stays until the next
        line.wait_for(stage, "It trusts")?;
        stage.play(pic.slide(Place::NotYet.green_start(), 1.2)?)?;
        play!(stage; tween(pic.probe_alpha, 1.0, 0.5)?);
        line.wait_for(stage, "Green reading")?;
        play!(stage; tween(over.gold_green, 1.0, 0.5)?);
        line.wait_for(stage, "and Blue reading")?;
        play!(stage;
            tween(pic.probe_alpha, 0.0, 0.3)?,
            tween(over.gold_green, 0.0, 0.3)?,
        );
        stage.play(pic.slide(DONE_E1, 1.2)?)?;
        play!(stage;
            tween(pic.probe_alpha, 1.0, 0.5)?,
            tween(over.gold_blue, 1.0, 0.5)?,
        );
        line.wait_for(stage, "Nobody was writing")?;
        let flags = lower_third(&kit, stage, glyphs::FLAGS_TAG, glyphs::FLAG_GLANCES)?;
        play!(stage; fade_in(stage.arena_mut(), flags, v(0.0, 0.15), 1.0)?.rt(0.7));
        kit.hold(stage)?;

        // 09_6: trace e1 is this place: garbage seven, ticket eight, a wait
        let line = say(&kit, stage, "09_6")?;
        let card = model_card(&kit, stage)?;
        play!(stage;
            fade_out(stage.arena_mut(), flags, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), card, v(0.0, -0.15), 1.0)?.rt(0.8),
        );
        e1_marks(&kit, stage, &line, &trace)?;
        kit.hold(stage)?;
        // time to read the corner card
        kit.music_breath(stage, 1.5)?;
        for m in [over.group, pic.green, pic.probe, anchor] {
            stage.clear_updaters(m, true);
        }
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 09_6: trace e1's values in the "done" place: Green's glance mid-write
/// read garbage, she took one more, Blue's glance read her real ticket, and
/// Green waits.
fn e1_marks(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken, e: &E1) -> fmn::Result<()> {
    let mid = (Y_BLUE + Y_GREEN) / 2.0;
    let xg = green_read(DONE_E1);
    let garbage = kit.text(stage, &e.garbage.to_string(), 28.0, chalk(), v(xg + 0.45, mid))?;
    line.wait_for(stage, "say seven")?;
    play!(stage; fade_in(stage.arena_mut(), garbage, ORIGIN, 1.0)?.rt(0.4));
    play!(stage; pulse(stage, garbage)?.rt(0.8));
    // Green takes one more; Blue's later glance reads that real ticket
    let tick = DONE_E1 + GREEN_W - 0.25;
    let ticket = kit.text(
        stage,
        &format!(":={}", e.green_ticket),
        24.0,
        chalk(),
        v(tick + 0.1, Y_GREEN + 0.45),
    )?;
    let saw = kit.text(stage, &e.blue_saw.to_string(), 24.0, chalk(), v(X_GLANCE + 0.3, mid))?;
    line.wait_for(stage, "take eight")?;
    play!(stage;
        fade_in(stage.arena_mut(), ticket, v(0.0, -0.1), 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), saw, ORIGIN, 1.0)?.rt(0.5),
    );
    let x_wait = X_ROOM - 0.3;
    let band = Rectangle::new()
        .width(X1 - x_wait)
        .height(0.24)
        .style(flat(customer(2), 0.22))
        .build()?
        .moved_to(v((x_wait + X1) / 2.0, Y_GREEN));
    let band = stage.arena_mut().add(band);
    let waits = kit.text(
        stage,
        glyphs::GREEN_WAITS,
        20.0,
        customer(2),
        v((x_wait + X1) / 2.0, Y_GREEN - 0.42),
    )?;
    line.wait_for(stage, "wait longer")?;
    play!(stage;
        fade_in(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), waits, ORIGIN, 1.0)?.rt(0.6),
    );
    Ok(())
}

/// 09_6's corner card: the model check with flickering memory, from the
/// recorded sweep the facts pin (C-F8.n3_k6_safe.*).
fn model_card(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let (n, k) = SAFE_SEARCH;
    let (states, clean) =
        recorded_sweep(n, k, true).ok_or_else(|| fail("09_6: no recorded safe sweep for its card".to_owned()))?;
    if !clean {
        return Err(fail("09_6: the safe sweep found a violation".to_owned()));
    }
    let at = v(3.35, 3.05);
    let a = kit.text(stage, glyphs::MODEL_A, 22.0, chalk(), v(at[0], at[1] + 0.22))?;
    let b = kit.text(
        stage,
        &format!(
            "{n} {} {k}: {} {}",
            glyphs::MODEL_B,
            thousands(states as u64),
            glyphs::MODEL_C
        ),
        22.0,
        chalk(),
        v(at[0], at[1] - 0.22),
    )?;
    let w = stage.get_width(a).max(stage.get_width(b)) + 0.6;
    let back = Rectangle::new()
        .width(w)
        .height(1.05)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(explainer_kit::hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group(stage, &[back, a, b])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::torn_two_digit_reads;

    fn script_line(id: &str) -> String {
        let script = include_str!("../../script.tsv");
        script
            .lines()
            .find(|l| l.starts_with(&format!("{id}\t")))
            .expect("line")
            .to_owned()
    }

    #[test]
    fn what_09_1_and_09_2_say_is_what_a_torn_rewrite_reads() {
        let (from, to) = (cited::TORN_FROM, cited::TORN_TO);
        let show = |d: [u32; 2]| format!("{}{}", d[0], d[1]);
        // tens first reads nineteen, ones first zero zero, in that order
        let torn: Vec<String> = [0, 1].map(|f| show(half_written(from, to, f))).to_vec();
        assert_eq!(torn, ["19", "00"]);
        assert!(script_line("09_1").contains("reads nineteen, or zero zero"));
        // the same two reads the facts pin (C-X1.rewrite_09_as_10)
        let mut sorted = torn.clone();
        sorted.sort();
        assert_eq!(sorted, torn_two_digit_reads(from, to));
        assert_eq!(
            (show(digits(from)), show(digits(to))),
            ("09".to_owned(), "10".to_owned())
        );
        // the wild values end on Lamport's 2496, and every one is a fact
        let e = e1(&Trace::E1.steps()).expect("e1");
        let wild = wild_values(&e);
        assert_eq!(wild.last().map(String::as_str), Some("2496"));
        assert!(script_line("09_2").contains("two thousand four hundred ninety-six"));
        assert_eq!(wild, ["9", "10", "19", "00", "7", "8", "2496"]);
    }

    #[test]
    fn twelve_years_from_the_bakery_to_safe_registers() {
        assert_eq!(cited::INTERPROCESS_YEAR - cited::BAKERY_YEAR, 12);
        assert!(script_line("09_3").starts_with("09_3\tTwelve years later"));
    }

    /// Green's choosing at every position the slides pass through, finely.
    fn slide_positions() -> Vec<f64> {
        let (lo, hi) = (DONE_E1 - 0.2, Place::NotYet.green_start() + 0.4);
        (0..=4000).map(|i| lo + (hi - lo) * f64::from(i) / 4000.0).collect()
    }

    #[test]
    fn the_proof_never_trusts_a_glance_that_crosses_a_write() {
        let mut static_in = [false; 3];
        for s in slide_positions() {
            let place = Place::of(X_CHECK, s);
            let ws = writes(s);
            // 09_5: the trusted glance never crosses a write, in any place
            if let Some(r) = trusted(place, s) {
                assert!(crossed(&r, &ws).is_none(), "{place:?} at {s}: trusted glance crossed");
                assert_eq!(r.var, Var::Slate);
            }
            for r in reads(s) {
                let Some(w) = crossed(&r, &ws) else { continue };
                static_in[Place::ALL.iter().position(|&p| p == place).expect("place")] = true;
                match r.var {
                    // static on a slate glance: never one the proof uses
                    Var::Slate => assert_ne!(Some(r), trusted(place, s)),
                    // a look at a flag mid-raise counts as not yet choosing;
                    // mid-lowering, as finished (the lower-third)
                    Var::Flag => {
                        let raising = (w.x0 - (s - WRITE_W)).abs() < 1e-12;
                        let want = if raising { Place::NotYet } else { Place::Done };
                        assert_eq!(place, want, "look mid-flip at {s}");
                    }
                }
            }
        }
        // static lands in every place along the slide
        assert_eq!(static_in, [true; 3]);
        // where 09_5 parks: not yet (08's), done (e1's); the static there
        // is on Green's own glance at Blue's slate, mid-write
        assert_eq!(Place::of(X_CHECK, Place::NotYet.green_start()), Place::NotYet);
        assert_eq!(Place::of(X_CHECK, DONE_E1), Place::Done);
        let crossed_at_done: Vec<Read> = reads(DONE_E1)
            .into_iter()
            .filter(|r| crossed(r, &writes(DONE_E1)).is_some())
            .collect();
        assert_eq!(crossed_at_done, [reads(DONE_E1)[1]]);
        assert!(
            reads(Place::NotYet.green_start())
                .iter()
                .all(|r| crossed(r, &writes(Place::NotYet.green_start())).is_none())
        );
        // Blue's glance while choosing comes before Green raises her flag
        // there, and before Blue's own ticket write, as in e1
        assert!(X_BLUE_MAX < DONE_E1 - WRITE_W && X_BLUE_MAX < writes(DONE_E1)[1].x0);
        assert!(X_BLUE_MAX > BLUE_CHOOSING.0);
        // the writes sit where 08 draws its ticket ticks
        assert!((writes(0.0)[1].x1 - (BLUE_CHOOSING.1 - 0.25)).abs() < 1e-12);
        assert!((writes(0.0)[4].x1 - (GREEN_W - 0.25)).abs() < 1e-12);
    }

    #[test]
    fn what_09_6_says_is_what_trace_e1_does() {
        let steps = Trace::E1.steps();
        let e = e1(&steps).expect("e1 has the done shape");
        // say seven: true garbage, neither Blue's old value nor her new one
        assert_eq!(e.garbage, 7);
        assert!(e.garbage != e.blue_old && e.garbage != e.blue_new);
        assert!(script_line("09_6").contains("say seven"));
        // take eight: one more than the garbage; Blue's glance reads it
        assert_eq!(e.green_ticket, 8);
        assert_eq!(e.green_ticket, e.garbage + 1);
        assert_eq!(e.blue_saw, e.green_ticket);
        assert!(script_line("09_6").contains("take eight and wait longer"));
        // and wait: Green ends waiting for Blue; never two in the room
        assert!(e.green_waits);
        let model = Trace::E1.model();
        assert!(steps.iter().all(|s| model.in_room(&s.after).len() <= 1));
        // the facts say the same (C-F6.e1.*)
        let f = crate::facts::facts().expect("facts");
        assert_eq!(f.get("C-F6.e1.overlapping_read_value"), Some("7"));
        assert_eq!(f.get("C-F6.e1.ticket_taken_after_garbage"), Some("8"));
        assert_eq!(f.get("C-F6.e1.max_in_room"), Some("1"));
    }

    #[test]
    fn the_corner_card_names_the_recorded_safe_search() {
        let (n, k) = SAFE_SEARCH;
        let (states, clean) = recorded_sweep(n, k, true).expect("a recorded safe search");
        assert!(clean && states == 269_550);
        let f = crate::facts::facts().expect("facts");
        assert_eq!(
            f.get(&format!("C-F8.n{n}_k{k}_safe.states")),
            Some(states.to_string().as_str())
        );
        assert_eq!(f.get(&format!("C-F8.n{n}_k{k}_safe.violation_found")), Some("0"));
    }

    #[test]
    fn the_listen_window_is_in_key_and_smears_only_the_crossed_glance() {
        let tonic = Identity::bakery_algorithm().tonic_midi;
        const IN_KEY: [i32; 6] = [10, 0, 2, 3, 5, 7];
        let ws = writes(DONE_E1);
        for r in reads(DONE_E1) {
            let p = read_pitch(&r, tonic);
            assert!(IN_KEY.contains(&p.rem_euclid(12)) && p <= 76, "pitch {p}");
            if crossed(&r, &ws).is_some() {
                assert!(IN_KEY.contains(&(p - 2).rem_euclid(12)), "the smear lands on {}", p - 2);
            }
        }
        // the playhead meets the glances in time order, inside the window
        let ts: Vec<f64> = reads(DONE_E1).iter().map(|r| head_t(r.x)).collect();
        assert!(ts.windows(2).all(|w| w[0] < w[1]) && ts[0] > 0.0 && ts[3] < HEAD_S);
        let mut s = listen_sound(DONE_E1);
        // the sweep plus its tail, ending in silence (no click at the cut)
        let secs = s.len() as f64 / SAMPLE_RATE;
        assert!(
            s.is_finite() && (secs - (HEAD_S + LISTEN_TAIL)).abs() < 1e-3,
            "{secs} s"
        );
        assert!(s.l.last().is_some_and(|x| x.abs() < 1e-12) && s.r.last().is_some_and(|x| x.abs() < 1e-12));
        let lufs = explainer_score::meter::integrated(&s);
        s.scale(10f64.powf((tunes::LEVEL_LUFS - lufs) / 20.0));
        assert!(s.peak() < 1.0);
    }
}
