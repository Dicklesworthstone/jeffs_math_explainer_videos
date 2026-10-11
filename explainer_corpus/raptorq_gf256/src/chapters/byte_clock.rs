//! Chapter 11, the byte clock: doubling from 1 (11_1), 29 after 128 and
//! every nonzero byte visited once in 255 steps, a tangled star in byte
//! order (11_2); the bytes slide to the step that reached them and the star
//! untangles into a circle (11_3); the byte clock, 255 hours, and its lap
//! (11_4); multiplying is adding clicks, read on the clicks-and-bytes ruler
//! (11_5); the slide rule, 150 × 3 = 167, the logarithm (11_6); the table
//! lookup and the mirror pairs (11_7); and the two clocks side by side (11_8).
//!
//! [`ByteClock`] and its [`ByteClock::mirror_pairs`] are pub for 17_7.

use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, stroke_style, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::clock::Clock;
use super::draw::{Role, group_of};
use super::fold::{CodePanel, Seg, fade_list, math_lower_third};
use crate::model::{AES_FOLD, exps, logs, order};

/// The byte clock's hours: the order of 2 under RaptorQ's fold.
pub const CLICKS: usize = 255;
/// The wrong clock's hours: one per byte.
const BYTE_HOURS: usize = 256;
const LISTEN_S: f64 = 3.0;
/// 11_5 and 11_6's products, and 11_1's doubling.
const TWO: u8 = 2;
const FOUR: u8 = 4;
const THREE: u8 = 3;
const ONE_FIFTY: u8 = 150;
/// The clicks the ruler of 11_5 shows: 0 … 12.
const RULER: usize = 13;
/// The doubling steps 11_1 names before asking what comes next.
const NAMED: usize = 8;

const C: Vec3 = [-2.7, -0.15, 0.0];
const R: f64 = 2.75;

/// The point at angle `turns` (0 at the top, clockwise) on a circle.
fn at_turn(centre: Vec3, radius: f64, turns: f64) -> Vec3 {
    let a = FRAC_PI_2 - TAU * turns;
    v(centre[0] + radius * a.cos(), centre[1] + radius * a.sin())
}

/// Byte order: byte `b` at hour `b` of a 256-hour clock (07's).
fn byte_point(centre: Vec3, radius: f64, b: u8) -> Vec3 {
    at_turn(centre, radius, f64::from(b) / BYTE_HOURS as f64)
}

/// The byte clock, 255 hours: click `k` (the byte 2ᵏ).
pub fn click_point(centre: Vec3, radius: f64, k: f64) -> Vec3 {
    at_turn(centre, radius, k / CLICKS as f64)
}

/// The byte clock: a ring, a dot per hour (click k is the byte 2ᵏ), and
/// zero, which has no click, at the centre.
pub struct ByteClock {
    pub centre: Vec3,
    pub radius: f64,
    pub ring: Mob,
    pub dots: Mob,
    pub zero: Mob,
    pub group: Mob,
}

impl ByteClock {
    pub fn build(stage: &mut Stage<'_>, centre: Vec3, radius: f64) -> fmn::Result<Self> {
        let ring = Circle::new()
            .radius(radius)
            .arc_center(centre)
            .style(stroke_style(Role::byte(), 2.5))
            .build();
        let ring = stage.arena_mut().add(ring);
        let dots: Vec<VMobject> = (0..CLICKS)
            .map(|k| {
                Dot::new()
                    .point(click_point(centre, radius, k as f64))
                    .radius(0.03)
                    .color(Role::byte())
                    .build()
            })
            .collect();
        let dots = stage.arena_mut().add(v_group(dots));
        let zero = Dot::new().point(centre).radius(0.06).color(Palette::dim()).build();
        let zero = stage.arena_mut().add(zero);
        let group = group_of(stage, &[ring, dots, zero])?;
        Ok(Self {
            centre,
            radius,
            ring,
            dots,
            zero,
            group,
        })
    }

    /// The 127 mirror pairs of this clock (see [`mirror_pairs`]).
    pub fn mirror_pairs(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        mirror_pairs(stage, self.centre, self.radius, color)
    }
}

/// The 127 mirror pairs (11_7, 17_7) on a byte clock at `centre`: byte a
/// and its partner a⁻¹ sit at clicks k and 255 − k, mirror images across
/// the top, joined by a chord. 1, at the top, is its own partner; zero has
/// none. Not yet on screen.
pub fn mirror_pairs(stage: &mut Stage<'_>, centre: Vec3, radius: f64, color: Srgb) -> fmn::Result<Mob> {
    let chords: Vec<VMobject> = (1..=CLICKS / 2)
        .map(|k| {
            Line::new(
                click_point(centre, radius, k as f64),
                click_point(centre, radius, (CLICKS - k) as f64),
            )
            .style(Style::default().fill(color, 0.0).stroke(color, 1.0, 0.35))
            .build()
        })
        .collect::<Result<_, _>>()?;
    Ok(stage.arena_mut().add(v_group(chords)))
}

/// A closed path through the bytes 2⁰, 2¹, …, 2²⁵⁴ in step order, each at
/// `place(step, byte)`.
fn path(place: impl Fn(usize, u8) -> Vec3) -> VMobject {
    let exp = exps();
    let points: Vec<Vec3> = exp.iter().enumerate().map(|(k, &b)| place(k, b)).collect();
    Polygon::new(points)
        .style(Style::default().fill(Role::byte(), 0.0).stroke(Role::byte(), 1.4, 0.6))
        .build()
}

pub struct TheByteClock {
    pub kit: Kit,
}

impl SceneConstruct for TheByteClock {
    fn name(&self) -> &str {
        "11_the_byte_clock"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let exp = exps();
        let log = logs();
        assert_eq!(
            order(TWO, crate::model::FOLD) as usize,
            CLICKS,
            "2 visits every nonzero byte"
        );

        // 11_1: doubling from 1 on the bytes in byte order; what comes next?
        let ring = Circle::new()
            .radius(R)
            .arc_center(C)
            .style(stroke_style(Palette::faint(), 1.5))
            .build();
        let ring = stage.arena_mut().add(ring);
        let byte_dots: Vec<VMobject> = (1..=CLICKS as u16)
            .map(|b| {
                Dot::new()
                    .point(byte_point(C, R, b as u8))
                    .radius(0.03)
                    .color(Role::byte())
                    .build()
            })
            .collect();
        // child i is byte i + 1; the log-order target below keeps that order
        let dots = stage.arena_mut().add(v_group(byte_dots));
        let one = kit.text(stage, "1", 28.0, Role::byte(), byte_point(C, R + 0.35, 1))?;
        let row1: Vec<String> = exp[..5].iter().map(u8::to_string).collect();
        let row2: Vec<String> = exp[5..NAMED].iter().map(u8::to_string).collect();
        let seq1 = kit.tex(stage, &row1.join(r" \to "), 48.0, Palette::ink(), &[], v(3.9, 2.3))?;
        let seq2 = kit.tex(
            stage,
            &format!(r"\to {} \to", row2.join(r" \to ")),
            48.0,
            Palette::ink(),
            &[],
            v(3.6, 1.45),
        )?;
        kit.say(stage, "11_1")?;
        play!(stage;
            show_creation(ring).rt(0.8),
            fade_in(stage.arena_mut(), dots, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), one, ORIGIN, 1.0)?.rt(0.8),
        );
        let mut chords = Vec::new();
        play!(stage; fade_in(stage.arena_mut(), seq1, v(-0.2, 0.0), 1.0)?.rt(0.6));
        for k in 0..NAMED - 1 {
            if k == 4 {
                play!(stage; fade_in(stage.arena_mut(), seq2, v(-0.2, 0.0), 1.0)?.rt(0.4));
            }
            let chord = Line::new(byte_point(C, R, exp[k]), byte_point(C, R, exp[k + 1]))
                .style(stroke_style(Role::byte(), 2.5))
                .build()?;
            let chord = stage.arena_mut().add(chord);
            play!(stage; show_creation(chord).rt(0.45));
            chords.push(chord);
        }
        let last = exp[NAMED - 1];
        let label_last = kit.text(
            stage,
            &last.to_string(),
            28.0,
            Role::byte(),
            byte_point(C, R + 0.45, last),
        )?;
        play!(stage; fade_in(stage.arena_mut(), label_last, ORIGIN, 1.0)?.rt(0.3));
        let q_at = v(stage.get_right(seq2)[0] + 0.45, 1.45);
        let q = predict_pause(&kit, stage, q_at)?;

        // 11_2: 29; every nonzero byte once, 255 steps: a tangled star
        let next = exp[NAMED];
        assert_eq!(next, crate::model::mul(TWO, last, crate::model::FOLD));
        let q_c = stage.get_center(q);
        let answer = kit.tex(stage, &next.to_string(), 48.0, Role::fold(), &[], q_c)?;
        let fold_chord = Line::new(byte_point(C, R, last), byte_point(C, R, next))
            .style(stroke_style(Role::fold(), 3.0))
            .build()?;
        let fold_chord = stage.arena_mut().add(fold_chord);
        let label_next = kit.text(
            stage,
            &next.to_string(),
            28.0,
            Role::fold(),
            byte_point(C, R + 0.4, next),
        )?;
        kit.say(stage, "11_2")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), answer, ORIGIN, 1.0)?.rt(0.3),
        );
        play!(stage;
            show_creation(fold_chord).rt(0.6),
            fade_in(stage.arena_mut(), label_next, ORIGIN, 1.0)?.rt(0.6),
        );
        // the rest of the walk, from 29 round to 1
        let rest_points: Vec<Vec3> = (NAMED..=CLICKS).map(|k| byte_point(C, R, exp[k % CLICKS])).collect();
        let rest = Polygon::polyline(rest_points)
            .style(Style::default().fill(Role::byte(), 0.0).stroke(Role::byte(), 1.4, 0.6))
            .build();
        let rest = stage.arena_mut().add(rest);
        let steps = stage.add_value_tracker(NAMED as f64 + 1.0);
        let counter = kit.live_value(
            stage,
            move |s| s.tracker_value(steps).unwrap_or(0.0).round(),
            0,
            false,
            44.0,
            Role::byte(),
            v(4.3, 0.2),
            RIGHT,
        )?;
        let steps_l = kit.text(stage, "steps", 30.0, Palette::dim(), v(5.0, 0.18))?;
        stage.move_to(steps_l, v(4.45, 0.18), LEFT);
        play!(stage; fade_in(stage.arena_mut(), steps_l, ORIGIN, 1.0)?.rt(0.3));
        let walk = show_creation(rest).rt(4.0);
        let count = steps
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(4.0),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(CLICKS as f64)?;
        play!(stage; walk, count);
        let aes = order(TWO, AES_FOLD);
        let note = precisely(
            &kit,
            stage,
            &format!(
                "under AES's polynomial, times two comes back to 1 after {aes} steps (its hand is {}); every finite \
                 field has such a clock",
                (2..=255u8)
                    .find(|&g| order(g, AES_FOLD) as usize == CLICKS)
                    .expect("AES's field has a generator")
            ),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), label_last, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 11_3: every byte slides to the step that reached it; the star untangles
        let star = stage.arena_mut().add(path(|_, b| byte_point(C, R, b)));
        stage.add_to_scene(star)?;
        let mut pieces = chords.clone();
        pieces.extend([fold_chord, rest]);
        stage.scene_mut().remove(&pieces);
        let circle = stage.arena_mut().add(path(|k, _| click_point(C, R, k as f64)));
        let target: Vec<VMobject> = (1..=CLICKS as u16)
            .map(|b| {
                Dot::new()
                    .point(click_point(C, R, f64::from(log[b as usize])))
                    .radius(0.03)
                    .color(Role::byte())
                    .build()
            })
            .collect();
        let target = stage.arena_mut().add(v_group(target));
        let zero = kit.text(stage, "0", 30.0, Palette::dim(), v(C[0], C[1] + 0.3))?;
        // the byte clock itself: its dots sit where the untangled bytes land
        let clock = ByteClock::build(stage, C, R)?;
        let zero_dot = clock.zero;
        let off = kit.text(stage, "off the clock", 24.0, Palette::dim(), v(C[0], C[1] - 0.35))?;
        kit.say(stage, "11_3")?;
        stage.clear_updaters(counter, true);
        play!(stage;
            fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), label_next, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), seq1, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), seq2, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), answer, ORIGIN, 1.0)?.rt(0.4),
        );
        stage.wait(0.4)?;
        play!(stage;
            Transform::new(star, circle).rt(3.2),
            Transform::new(dots, target).rt(3.2),
        );
        play!(stage;
            fade_in(stage.arena_mut(), zero_dot, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), zero, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), off, ORIGIN, 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 11_4: one click around; 255 hours; the byte clock; its lap
        let title = kit.text_styled(stage, "the byte clock", 54.0, Role::byte(), v(4.0, 2.6), true, false)?;
        let hours = kit.text(stage, &format!("{CLICKS} hours"), 34.0, Palette::dim(), v(4.0, 1.85))?;
        let click = stage.add_value_tracker(0.0);
        let hand = byte_hand(stage, click, Palette::ink());
        let times_two = kit.tex(
            stage,
            r"\times 2 = \text{one click}",
            44.0,
            Palette::ink(),
            &[],
            v(4.0, 0.7),
        )?;
        kit.say(stage, "11_4")?;
        play!(stage;
            fade_out(stage.arena_mut(), steps_l, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), counter, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), off, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), zero, ORIGIN, 1.0)?.rt(0.4),
        );
        // the untangled dots are exactly the clock's: swap them in place
        stage.scene_mut().remove(&[dots]);
        stage.add_to_scene(clock.dots)?;
        stage.add_to_scene(hand)?;
        play!(stage; tween(click, 1.0, 0.7)?, fade_in(stage.arena_mut(), times_two, ORIGIN, 1.0)?.rt(0.7));
        // the untangled 255-gon is already the clock's ring to the eye: swap
        // them in place, and the faint byte-order ring of 11_1 goes
        stage.scene_mut().remove(&[star]);
        stage.add_to_scene(clock.ring)?;
        play!(stage;
            fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.6),
            write(stage.arena(), title).rt(1.0),
        );
        play!(stage; fade_in(stage.arena_mut(), hours, ORIGIN, 1.0)?.rt(0.5));
        // the lap: the gold dot once round, 255 clicks, climbing an octave
        let wav = super::sound::wav("11_4_lap", &super::sound::lap(CLICKS, LISTEN_S))?;
        kit.sonify(stage, LISTEN_S, "11_4", &wav)?;
        let lap = click
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(LISTEN_S - 0.6),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(1.0 + CLICKS as f64)?;
        stage.play(lap)?;
        let home = Dot::new()
            .point(click_point(C, R, 0.0))
            .radius(0.09)
            .color(Role::byte())
            .build();
        let home = stage.arena_mut().add(home);
        play!(stage; fade_in(stage.arena_mut(), home, ORIGIN, 2.0)?.rt(0.3));
        play!(stage; indicate(stage.arena_mut(), home, 1.8, Some([1.0, 0.85, 0.4]))?.rt(0.3));
        let raid = lower_third(
            &kit,
            stage,
            "for programmers:",
            &format!(
                "Linux RAID-6 gives drive i the weight 2^i, i clicks round the byte clock; with AES's polynomial only \
                 {aes} of those weights differ"
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), raid, v(0.0, 0.2), 1.0)?.rt(0.5));
        kit.hold(stage)?;

        // 11_5: multiplying is adding clicks, read on the ruler of clicks 0 … 12
        let ruler = Ruler::build(&kit, stage, &exp)?;
        let powers = kit.tex(
            stage,
            r"2^{3} \cdot 2^{4} = 2^{3 + 4} = 2^{7}",
            48.0,
            Palette::ink(),
            &[],
            v(4.1, 3.2),
        )?;
        let ints = kit.tex(
            stage,
            &format!(r"{} \times {} = {}", 1u32 << 3, 1u32 << 4, (1u32 << 3) * (1u32 << 4)),
            40.0,
            Palette::dim(),
            &[],
            v(4.1, 2.55),
        )?;
        let (a, b) = (exp[NAMED - 1], FOUR);
        let (ka, kb) = (usize::from(log[a as usize]), usize::from(log[b as usize]));
        let product = exp[(ka + kb) % CLICKS];
        assert_eq!(product, crate::model::mul(a, b, crate::model::FOLD));
        kit.say(stage, "11_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), raid, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), hours, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), times_two, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), home, ORIGIN, 1.0)?.rt(0.4),
            tween(click, 0.0, 0.4)?,
        );
        play!(stage; write(stage.arena(), powers).rt(0.9), fade_in(stage.arena_mut(), ints, ORIGIN, 1.0)?.rt(0.9));
        ruler.show(stage)?;
        let ask = kit.tex(
            stage,
            &format!(r"{a} \times {b}"),
            52.0,
            Palette::ink(),
            &[],
            v(4.1, -1.15),
        )?;
        play!(stage; write(stage.arena(), ask).rt(0.6));
        let hop1 = ruler.hop(&kit, stage, 0, ka, &format!("{ka} clicks"))?;
        play!(stage; tween(click, ka as f64, 1.0)?, show_creation(hop1).rt(1.0));
        let hop2 = ruler.hop(&kit, stage, ka, ka + kb, &format!("+{kb}"))?;
        play!(stage; tween(click, (ka + kb) as f64, 0.7)?, show_creation(hop2).rt(0.7));
        play!(stage; indicate(stage.arena_mut(), ruler.bytes[ka + kb], 1.5, Some([1.0, 0.85, 0.4]))?.rt(0.8));
        let sum = kit.tex(
            stage,
            &format!(r"{ka} + {kb} = {} \text{{ clicks}} \to {product}", ka + kb),
            44.0,
            Role::byte(),
            &[],
            v(4.1, -1.9),
        )?;
        play!(stage; write(stage.arena(), sum).rt(0.8));
        let seven = 7u8;
        let ls = usize::from(log[seven as usize]);
        let twice = 2 * ls;
        let squared = crate::model::mul(seven, seven, crate::model::FOLD);
        let by_hand = crate::model::poly::poly(squared);
        let dd = math_lower_third(
            &kit,
            stage,
            "precisely:",
            &format!(
                r"{seven} \times {seven}:\; {ls} + {ls} = {twice} \to {} \text{{ clicks}} \to {squared};\quad \text{{by hand }} (x^{{2}} + x + 1)^{{2}} = {by_hand} = {squared}",
                twice % CLICKS
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), dd, v(0.0, 0.2), 1.0)?.rt(0.5));
        kit.hold(stage)?;

        // 11_6: the slide rule: 150 × 3, 180 + 25 = 205 clicks, 167; the logarithm
        let (k150, k3) = (usize::from(log[ONE_FIFTY as usize]), usize::from(log[THREE as usize]));
        let k_sum = k150 + k3;
        let p = exp[k_sum % CLICKS];
        assert_eq!(p, crate::model::mul(ONE_FIFTY, THREE, crate::model::FOLD));
        let outer = |kit: &Kit, stage: &mut Stage<'_>, byte: u8| -> fmn::Result<Mob> {
            let k = f64::from(log[byte as usize]);
            kit.text(
                stage,
                &byte.to_string(),
                30.0,
                Role::byte(),
                click_point(C, R + 0.38, k),
            )
        };
        let o3 = outer(&kit, stage, THREE)?;
        let o150 = outer(&kit, stage, ONE_FIFTY)?;
        let o167 = outer(&kit, stage, p)?;
        let r_in = R - 0.55;
        let inner_ring = Circle::new()
            .radius(r_in)
            .arc_center(C)
            .style(stroke_style(Palette::ink(), 2.0))
            .build();
        let inner_ring = stage.arena_mut().add(inner_ring);
        let turn = stage.add_value_tracker(0.0);
        let inner_labels = inner_marks(&kit, stage, turn, &[(1u8, 0usize), (ONE_FIFTY, k150)], r_in);
        let ask2 = kit.tex(
            stage,
            &format!(r"{ONE_FIFTY} \times {THREE}"),
            52.0,
            Palette::ink(),
            &[],
            v(4.1, 2.4),
        )?;
        let clicks_line = kit.tex(
            stage,
            &format!(r"{k150} + {k3} = {k_sum} \text{{ clicks}}"),
            44.0,
            Palette::ink(),
            &[],
            v(4.1, 1.4),
        )?;
        let gives = kit.tex(stage, &format!(r"\to {p}"), 52.0, Role::byte(), &[], v(4.1, 0.55))?;
        let logname = kit.text_styled(stage, "logarithm", 48.0, Role::byte(), v(4.1, -0.7), true, false)?;
        let logline = kit.tex(
            stage,
            &format!(r"\log_{{2}} {ONE_FIFTY} = {k150}"),
            44.0,
            Palette::ink(),
            &[],
            v(4.1, -1.5),
        )?;
        kit.say(stage, "11_6")?;
        let mut gone = ruler.all.clone();
        gone.extend([powers, ints, ask, hop1, hop2, sum, dd]);
        fade_list(stage, &gone, 0.5)?;
        play!(stage; write(stage.arena(), ask2).rt(0.6), tween(click, 0.0, 0.5)?);
        play!(stage;
            fade_in(stage.arena_mut(), o150, ORIGIN, 1.0)?.rt(0.5),
            tween(click, k150 as f64, 1.0)?,
        );
        play!(stage;
            fade_in(stage.arena_mut(), o3, ORIGIN, 1.0)?.rt(0.5),
            show_creation(inner_ring).rt(0.6),
        );
        stage.add_to_scene(inner_labels)?;
        play!(stage; write(stage.arena(), clicks_line).rt(0.8));
        // the inner ring turns 25 clicks: its 1 under the outer 3, its 150 under 167
        play!(stage;
            tween(turn, k3 as f64, 1.6)?,
            tween(click, k_sum as f64, 1.6)?,
        );
        play!(stage;
            fade_in(stage.arena_mut(), o167, ORIGIN, 1.0)?.rt(0.5),
            write(stage.arena(), gives).rt(0.5),
        );
        play!(stage; write(stage.arena(), logname).rt(0.7));
        play!(stage; fade_in(stage.arena_mut(), logline, ORIGIN, 1.0)?.rt(0.5));
        kit.hold(stage)?;

        // 11_7: the standard's lookup; every partner at its mirror image
        let code = CodePanel::build(&kit, stage, &table_code(), 16.0, v(6.95, 3.6))?;
        let pairs = clock.mirror_pairs(stage, Role::byte())?;
        let top = click_point(C, R + 0.35, 0.0);
        let own = kit.text(stage, "its own partner", 24.0, Role::byte(), v(top[0] + 1.25, top[1]))?;
        let none = kit.text(stage, "0: none", 24.0, Palette::dim(), v(C[0], C[1] - 0.35))?;
        let wraps = crate::model::wrap::u8_log_sum_wraps(&log);
        let caption = kit.text(
            stage,
            &format!(
                "a u8 sum wraps for {} of {} pairs",
                explainer_kit::thousands(wraps.0 as u64),
                explainer_kit::thousands(wraps.1 as u64)
            ),
            22.0,
            Palette::dim(),
            v(4.65, 0.85),
        )?;
        kit.say(stage, "11_7")?;
        stage.clear_updaters(inner_labels, true);
        fade_list(
            stage,
            &[
                o3,
                o150,
                o167,
                inner_ring,
                inner_labels,
                ask2,
                clicks_line,
                gives,
                logname,
                logline,
            ],
            0.5,
        )?;
        play!(stage; fade_in(stage.arena_mut(), code.group, v(-1.0, 0.0), 1.0)?.rt(0.7));
        play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.5));
        stage.wait(1.4)?;
        play!(stage; fade_in(stage.arena_mut(), pairs, ORIGIN, 1.0)?.rt(2.0).lag(0.03));
        play!(stage;
            fade_in(stage.arena_mut(), own, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), none, ORIGIN, 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 11_8: the wrong clock counted bytes; the right one counts clicks
        stage.clear_updaters(hand, true);
        fade_list(stage, &[code.group, caption, pairs, own, none, hand, one], 0.5)?;
        let wrong = Clock::build(
            &kit,
            stage,
            BYTE_HOURS,
            v(-3.6, 0.2),
            2.3,
            Role::wrap(),
            &[0, BYTE_HOURS / 4, BYTE_HOURS / 2, 3 * BYTE_HOURS / 4],
            22.0,
        )?;
        let bytes_l = kit.text(stage, "bytes", 40.0, Role::wrap(), v(-3.6, -2.75))?;
        let clicks_l = kit.text(stage, "clicks", 40.0, Role::byte(), v(3.6, -2.75))?;
        kit.say(stage, "11_8")?;
        // the clock as one group (its ring, dots and zero), moved aside
        stage.add_to_scene(clock.group)?;
        play!(stage; clock.group.animate().scale(2.3 / R)?.move_to(v(3.6, 0.2), ORIGIN)?);
        play!(stage;
            show_creation(wrong.ring).rt(0.8),
            fade_in(stage.arena_mut(), wrong.ticks, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), bytes_l, ORIGIN, 1.0)?.rt(0.8),
        );
        let mut labels: Vec<Box<dyn Animation>> = Vec::new();
        for &(_, l) in &wrong.labels {
            let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
            labels.push(stage.prepare(a)?);
        }
        let labels = explainer_kit::lagged(stage, labels, 0.15)?;
        play!(stage; labels.rt(0.6));
        let broke = kit.tex(stage, r"2 \times 128 \equiv 0", 40.0, Role::lost(), &[], v(-3.6, -3.35))?;
        play!(stage; fade_in(stage.arena_mut(), broke, ORIGIN, 1.0)?.rt(0.5));
        stage.wait(1.5)?;
        play!(stage; fade_in(stage.arena_mut(), clicks_l, ORIGIN, 1.0)?.rt(0.6));
        let lap_l = kit.tex(
            stage,
            &format!(r"{CLICKS} \text{{ clicks}}"),
            40.0,
            Role::byte(),
            &[],
            v(3.6, -3.35),
        )?;
        play!(stage; fade_in(stage.arena_mut(), lap_l, ORIGIN, 1.0)?.rt(0.5));
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// The hand of the byte clock at click `tracker`, rebuilt each frame.
fn byte_hand(stage: &mut Stage<'_>, tracker: Mob, color: Srgb) -> Mob {
    stage.always_redraw(move |s| {
        let k = s.tracker_value(tracker).unwrap_or(0.0);
        let tip = click_point(C, R, k);
        let stem = click_point(C, R * 0.88, k);
        let line = Line::new(C, stem)
            .style(stroke_style(color, 3.0))
            .build()
            .expect("a hand has length");
        let dot = Dot::new().point(tip).radius(0.08).color(Role::byte()).build();
        s.add(v_group([line, dot]))
    })
}

/// The inner ring's labels, turned `tracker` clicks clockwise and kept
/// upright: (byte, click) each.
fn inner_marks(kit: &Kit, stage: &mut Stage<'_>, tracker: Mob, marks: &[(u8, usize)], radius: f64) -> Mob {
    let book = Rc::clone(&kit.book);
    let marks = marks.to_vec();
    stage.always_redraw(move |s| {
        let t = s.tracker_value(tracker).unwrap_or(0.0);
        let mut parts = Vec::new();
        for &(b, k) in &marks {
            let p = click_point(C, radius - 0.3, k as f64 + t);
            let tick = Line::new(
                click_point(C, radius - 0.12, k as f64 + t),
                click_point(C, radius + 0.12, k as f64 + t),
            )
            .style(stroke_style(Palette::ink(), 3.0))
            .build()
            .expect("a tick has length");
            let label = Text::new(&b.to_string())
                .font_size(28.0)
                .style(explainer_kit::ink_style(Palette::ink()))
                .build(&book)
                .expect("bundled text always lays out");
            let tick = s.add(tick);
            let label = s.add(label);
            s.move_to(label, p, ORIGIN);
            parts.push(tick);
            parts.push(label);
        }
        let g = s.add(VMobject::new());
        for m in parts {
            s.attach(g, m).expect("fresh mobjects attach");
        }
        g
    })
}

/// 11_5's ruler: clicks 0 … 12 over the bytes they reach (OCT_EXP's first
/// thirteen entries), magnifying the top of the clock.
struct Ruler {
    bytes: Vec<Mob>,
    all: Vec<Mob>,
    x0: f64,
    step: f64,
}

const RULER_Y: f64 = 0.7;

impl Ruler {
    fn build(kit: &Kit, stage: &mut Stage<'_>, exp: &[u8; 255]) -> fmn::Result<Self> {
        let (x0, step) = (1.45, 0.43);
        let (mut clicks, mut bytes, mut all) = (Vec::new(), Vec::new(), Vec::new());
        for (k, byte) in exp.iter().take(RULER).enumerate() {
            let x = x0 + step * k as f64;
            clicks.push(kit.text(stage, &k.to_string(), 22.0, Palette::dim(), v(x, RULER_Y + 0.3))?);
            bytes.push(kit.text(stage, &byte.to_string(), 22.0, Role::byte(), v(x, RULER_Y - 0.3))?);
        }
        let line = Line::new(v(x0 - 0.25, RULER_Y), v(x0 + step * (RULER - 1) as f64 + 0.25, RULER_Y))
            .style(stroke_style(Palette::faint(), 2.0))
            .build()?;
        let line = stage.arena_mut().add(line);
        let head_c = kit.text(stage, "clicks", 22.0, Palette::dim(), v(x0 + step * 6.0, RULER_Y + 0.7))?;
        let head_b = kit.text(stage, "bytes", 22.0, Role::byte(), v(x0 + step * 6.0, RULER_Y - 0.7))?;
        // the magnifier: the clock's first clicks fan out to the ruler
        let lens_a = Line::new(click_point(C, R, 0.0), v(x0 - 0.25, RULER_Y))
            .style(stroke_style(Palette::faint(), 1.5))
            .build()?;
        let lens_b = Line::new(
            click_point(C, R, (RULER - 1) as f64),
            v(x0 + step * (RULER - 1) as f64 + 0.25, RULER_Y),
        )
        .style(stroke_style(Palette::faint(), 1.5))
        .build()?;
        let (lens_a, lens_b) = (stage.arena_mut().add(lens_a), stage.arena_mut().add(lens_b));
        all.extend(&clicks);
        all.extend(&bytes);
        all.extend([line, head_c, head_b, lens_a, lens_b]);
        Ok(Self { bytes, all, x0, step })
    }

    fn show(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &self.all {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(0.8),
                ..PlayOverrides::default()
            },
        )?;
        Ok(())
    }

    /// An arc over the ruler from click `from` to `to`, labelled.
    fn hop(&self, kit: &Kit, stage: &mut Stage<'_>, from: usize, to: usize, label: &str) -> fmn::Result<Mob> {
        let (a, b) = (self.x0 + self.step * from as f64, self.x0 + self.step * to as f64);
        let y = RULER_Y + 0.55;
        let arc = Line::new(v(a, y), v(b, y))
            .path_arc(-1.4)
            .style(stroke_style(Role::byte(), 3.0))
            .build()?;
        let arc = stage.arena_mut().add(arc);
        // the arc's height over its chord (sagitta) for a 1.4 rad arc
        let rise = (b - a) / 2.0 * (1.4f64 / 4.0).tan();
        let l = kit.text(stage, label, 22.0, Role::byte(), v((a + b) / 2.0, y + rise + 0.2))?;
        group_of(stage, &[arc, l])
    }
}

/// 11_7's for-programmers panel: multiplying by table lookup, the tables'
/// sizes from the field (EXP needs index 254 + 254; LOG sums need u16).
fn table_code() -> Vec<Vec<Seg>> {
    let top = 2 * (CLICKS - 1);
    let plain = |s: String| -> Vec<Seg> { vec![(s, None)] };
    vec![
        plain(format!("static EXP: [u8; {}] = [/* 2^0 … 2^{} */];", top + 2, top + 1)),
        plain(format!(
            "// largest index {} + {} = {top}: no % {CLICKS}",
            CLICKS - 1,
            CLICKS - 1
        )),
        plain("static LOG: [u16; 256] = [/* LOG[0] unused */];".to_owned()),
        plain("// u16: a u8 sum wraps at 256 (chapter 7's bug)".to_owned()),
        plain("fn mul(a: u8, b: u8) -> u8 {".to_owned()),
        plain("    if a == 0 || b == 0 { return 0 }".to_owned()),
        plain("    EXP[(LOG[a as usize] + LOG[b as usize]) as usize]".to_owned()),
        plain("}".to_owned()),
    ]
}
