//! Chapter 01, the bridge that swayed: the cold open. London, June 2000: a
//! new footbridge sways sideways by about seven centimetres under nothing but
//! footsteps, and two days later it is closed; and it had been checked
//! (01_1). A bridge has ways of moving that are all its own, so does the air
//! in a room, and an economy has a mix of its own: one calculation finds
//! them (01_2). The title (01_3). The designers had checked how it would
//! bounce; the code said nothing of walkers pushing a bridge sideways (01_4).
//! So what does a bridge have, besides strength, that footsteps can find
//! (01_5)? A way of moving all its own, sideways between half a swing and
//! one swing a second, and a walker sways about once a second, never quite
//! in step (01_6).
//!
//! Every number is a fact: the dates `K-B2.opened` and `K-B2.closed`, the
//! sway `K-B2.sway_cm`, the crowd `K-B2.first_day`, the rates
//! `K-B2.lateral_hz` and `K-B2.walker_lateral_hz`; the three small pictures
//! of 01_2 are drawn from the model (the truss's softest pattern, the
//! lowest mode of a chain of air slices, the economy's even mix and its
//! eigenvalue).

use std::f64::consts::TAU;

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::deck::{Deck, DeckSpec, edge, oscillate};
use super::draw::{
    arrow_points, chip, corners, fact, fade_in_all, fade_in_anim, fade_out_all, fade_out_anim, faded_line, fail, flat,
    good, group, numbers, own, panel, predict_hold, say, tick_mark, tween_all, tween_linear, warn, whole,
};
use super::marks::mark;
use super::truss::{Layout, TrussPic};
use crate::model::carts;
use crate::model::linalg::jacobi_eigh;
use crate::model::truss::Truss;

// ------------------------------------------------------------------ choices

/// The deck of 01_1 and 01_4: where it sits and how big.
const DECK: DeckSpec = DeckSpec {
    centre: [0.0, 0.3, 0.0],
    length: 11.0,
    width: 1.0,
};
/// How far the deck sways at midspan on screen (scene units): the picture's
/// size for "about seven centimetres", not a scale.
const SWAY: f64 = 0.5;
/// The deck's walkers.
const WALKERS: usize = 22;
/// The three panels of 01_2: centres, and the air's slices.
const PANELS_X: [f64; 3] = [-4.4, 0.0, 4.4];
const PANEL_Y: f64 = 0.35;
const SLICES: usize = 14;
/// 01_6: the gold curve's place and size, the walker's place, and the
/// column the counters stand in.
const CURVE: DeckSpec = DeckSpec {
    centre: [-0.6, 2.1, 0.0],
    length: 8.0,
    width: 0.0,
};
const CURVE_AMP: f64 = 0.6;
const WALKER_AT: Vec3 = [-0.6, -0.9, 0.0];
const WALKER_SWAY: f64 = 0.55;
const COUNTERS_X: f64 = 5.5;
/// 01_1: how far along the deck from its middle the sway's bracket stands,
/// and half the reading's size (30 pt, "7 cm"), scene units.
const BRACKET_DX: f64 = 1.1;
const READING_HALF: (f64, f64) = (0.45, 0.2);

// ------------------------------------------------------------- the geometry

/// An ISO date `2000-06-10` as it is written on screen: `10 June 2000`.
pub fn date_text(iso: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let mut parts = iso.split('-');
    let year: u32 = parts.next()?.parse().ok()?;
    let month: usize = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    Some(format!("{day} {} {year}", MONTHS.get(month.checked_sub(1)?)?))
}

/// The date without its year (`12 June`): the stamp of 01_1.
pub fn day_month(iso: &str) -> Option<String> {
    let full = date_text(iso)?;
    full.rsplit_once(' ').map(|(d, _)| d.to_owned())
}

/// The lowest mode of `n` slices of air held between two walls, as the
/// carts' chain finds it: the shape of the air sloshing, scaled so that its
/// largest swing is 1 and pointing the same way at midspan.
pub fn lowest_air_mode(n: usize) -> Vec<f64> {
    let e = jacobi_eigh(&carts::chain(n));
    let mode = &e.vectors[0];
    let top = mode.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    let sign = mode[n / 2].signum();
    mode.iter().map(|x| sign * x / top).collect()
}

/// Where 01_1 sets the sway's reading: above the highest the deck's top edge
/// gets in the span of the words, at the full sway, so that no edge line ever
/// crosses them (it stood on the deck and the lower edge went through it).
pub fn reading_at(spec: &DeckSpec) -> Vec3 {
    let x = spec.centre[0] + BRACKET_DX;
    let top = edge(spec, SWAY, 1.0, 0.0, 1.0)
        .iter()
        .filter(|p| (p[0] - x).abs() <= READING_HALF.0 + 0.25)
        .fold(f64::NEG_INFINITY, |m, p| m.max(p[1]));
    v(x, top + READING_HALF.1 + 0.12)
}

/// How many whole swings (or sways) a counter that began at clock reading
/// `zero` has counted at reading `now`, at `hz` a second: it opens at 0.
pub fn counted(now: f64, zero: f64, hz: f64) -> f64 {
    ((now - zero) * hz).floor().max(0.0)
}

/// The two ends of a range written as the facts write it, `80000-100000`.
pub fn crowd_range(text: &str) -> Option<(u64, u64)> {
    let (a, b) = text.split_once('-')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Lines of a card, centred, on a dark plate edged in `edge`, detached.
pub fn question_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    lines: &[&str],
    size: f64,
    edge_colour: Srgb,
    at: Vec3,
) -> fmn::Result<Mob> {
    let pitch = 0.62;
    let mut texts = Vec::new();
    let mut widest = 0.0_f64;
    for (k, l) in lines.iter().enumerate() {
        let y = at[1] + pitch * ((lines.len() as f64 - 1.0) / 2.0 - k as f64);
        let t = kit.text(stage, l, size, Palette::ink(), v(at[0], y))?;
        widest = widest.max(stage.get_width(t));
        texts.push(t);
    }
    let back = Rectangle::new()
        .width(widest + 0.8)
        .height(pitch * lines.len() as f64 + 0.5)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.96).stroke(edge_colour, 2.5, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let mut all = vec![back];
    all.extend(texts);
    group(stage, &all)
}

/// A walker seen from behind: a head over a body, centred at `at`,
/// detached, as one group.
fn walker_icon(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let ink = Palette::ink();
    let head = Circle::new()
        .radius(0.2)
        .arc_center(v(at[0], at[1] + 0.55))
        .style(flat(ink, 1.0))
        .build();
    let body = Rectangle::new()
        .width(0.5)
        .height(0.9)
        .corner_radius(0.18)
        .style(flat(ink, 1.0))
        .build()?
        .moved_to(v(at[0], at[1] - 0.1));
    let (head, body) = (stage.arena_mut().add(head), stage.arena_mut().add(body));
    group(stage, &[head, body])
}

/// A short double-headed arrow along `(dx, dy)` through `at`, detached.
fn double_arrow(stage: &mut Stage<'_>, at: Vec3, dx: f64, dy: f64) -> Vec<Mob> {
    let ink = Palette::ink();
    let (a, b) = (v(at[0] - dx, at[1] - dy), v(at[0] + dx, at[1] + dy));
    let towards_b = polyline(arrow_points(at, b, 0.18))
        .style(faded_line(ink, 4.0, 1.0))
        .build();
    let towards_a = polyline(arrow_points(at, a, 0.18))
        .style(faded_line(ink, 4.0, 1.0))
        .build();
    vec![stage.arena_mut().add(towards_b), stage.arena_mut().add(towards_a)]
}

// -------------------------------------------------------------- the scene

pub struct TheBridgeThatSwayed {
    pub kit: Kit,
}

impl SceneConstruct for TheBridgeThatSwayed {
    fn name(&self) -> &str {
        "01_the_bridge_that_swayed"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim, gold) = (Palette::ink(), Palette::dim(), own());
        let lateral = numbers("K-B2.lateral_hz")?;
        // the sideways mode nearest a second, and the walker's own rate
        let deck_hz = lateral[2];
        let walker_hz = numbers("K-B2.walker_lateral_hz")?[0];

        // ---- 01_1: the deck from above, already swinging under walkers
        let deck = Deck::build(stage, DECK, WALKERS)?;
        deck.start(stage)?;
        stage.set_tracker_value(deck.swings, deck_hz)?;
        let opened = date_text(fact("K-B2.opened")?).ok_or_else(|| fail("K-B2.opened is not a date".into()))?;
        let closed = day_month(fact("K-B2.closed")?).ok_or_else(|| fail("K-B2.closed is not a date".into()))?;
        let date = kit.text_left(stage, &opened, 36.0, ink, v(-6.7, 3.3))?;
        // the sway, as a bracket at midspan with its reading
        let mid = DECK.centre;
        let ruler = {
            let x = mid[0] + 1.1;
            let l = polyline(vec![v(x, mid[1] - SWAY), v(x, mid[1] + SWAY)])
                .style(faded_line(ink, 3.0, 1.0))
                .build();
            let l = stage.arena_mut().add(l);
            let mut parts = vec![l];
            for y in [mid[1] - SWAY, mid[1] + SWAY] {
                let cap = polyline(vec![v(x - 0.12, y), v(x + 0.12, y)])
                    .style(faded_line(ink, 3.0, 1.0))
                    .build();
                parts.push(stage.arena_mut().add(cap));
            }
            group(stage, &parts)?
        };
        let sway_reading = kit.text(
            stage,
            &format!("{} cm", fact("K-B2.sway_cm")?),
            30.0,
            ink,
            reading_at(&DECK),
        )?;
        let stamp_at = v(3.9, 2.3);
        let stamp = chip(&kit, stage, &format!("closed {closed}"), 36.0, warn(), stamp_at)?;
        stage.rotate(stamp, -0.14, OUT, Some(stamp_at), None);
        let (low, high) =
            crowd_range(fact("K-B2.first_day")?).ok_or_else(|| fail("K-B2.first_day is not a range".into()))?;
        let crowd = lower_third(
            &kit,
            stage,
            "the first day:",
            &format!("{} to {} people", thousands(low), thousands(high)),
        )?;
        // frame one is the swinging deck (R1 C4), whole before the date has
        // faded in; it had come in over 1.6 s after the date, behind a black
        // first second
        stage.set_tracker_value(deck.edges_on, 1.0)?;
        stage.set_tracker_value(deck.walkers_on, 1.0)?;
        stage.set_tracker_value(deck.sway, SWAY)?;
        mark(&kit, stage, "01_1.deck_whole");
        // (said only now: `say` holds the first 0.6 s before its line begins,
        // and the deck is to be there through them)
        let line = say(&kit, stage, "01_1")?;
        fade_in_all(stage, &[date], 0.7)?;
        line.wait_for(stage, "sways sideways by about seven centimetres")?;
        fade_in_all(stage, &[ruler, sway_reading], 0.5)?;
        line.wait_for(stage, "Two days later it is closed")?;
        fade_in_all(stage, &[stamp, crowd], 0.5)?;

        // ---- 01_2: the bridge, the air and the economy each have their own
        let line = say(&kit, stage, "01_2")?;
        tween_all(stage, &[(deck.walkers_on, 0.0), (deck.edges_on, 0.0)], 0.5)?;
        fade_out_all(stage, &[date, ruler, sway_reading, stamp, crowd], 0.6)?;
        // a truss sagging and springing back, in gold
        let t = Truss::pratt();
        let softest = t.all_moves(&t.patterns().vectors[0]);
        let biggest = softest.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        let truss_layout = Layout {
            origin: v(PANELS_X[0] - 1.8, PANEL_Y - 0.2),
            scale: 0.15,
        };
        let truss_pic = TrussPic::build(
            stage,
            &t,
            truss_layout,
            [softest.clone(), softest],
            [0.9 / biggest, 0.9 / biggest],
            vec![0.0; t.members.len()],
        )?;
        truss_pic.start(stage)?;
        stage.set_tracker_value(truss_pic.tint, 1.0)?;
        // the air in a box, sloshing: slices of air moved by the lowest mode
        let mode = lowest_air_mode(SLICES);
        let (box_w, box_h) = (3.6, 1.5);
        let air_box = Rectangle::new()
            .width(box_w)
            .height(box_h)
            .corner_radius(0.08)
            .style(faded_line(ink, 3.0, 1.0))
            .build()?
            .moved_to(v(PANELS_X[1], PANEL_Y));
        let air_box = stage.arena_mut().add(air_box);
        let step = box_w / (SLICES as f64 + 1.0);
        let slice_x = |k: usize| PANELS_X[1] - box_w / 2.0 + step * (k as f64 + 1.0);
        let (y0, y1) = (PANEL_Y - box_h / 2.0 + 0.1, PANEL_Y + box_h / 2.0 - 0.1);
        let slices: Vec<Mob> = (0..SLICES)
            .map(|k| {
                let l = polyline(vec![v(slice_x(k), y0), v(slice_x(k), y1)])
                    .style(faded_line(gold, 4.0, 1.0))
                    .build();
                stage.arena_mut().add(l)
            })
            .collect();
        let air_anchor = stage.arena_mut().add(VMobject::new());
        {
            let (slices_m, mode_m) = (slices.clone(), mode.clone());
            let clock = deck.clock;
            let xs: Vec<f64> = (0..SLICES).map(slice_x).collect();
            let mut last = f64::NAN;
            stage.add_updater(
                air_anchor,
                move |s, _| {
                    let t = s.tracker_value(clock).unwrap_or(0.0);
                    if t.to_bits() == last.to_bits() {
                        return;
                    }
                    last = t;
                    let swing = (TAU * t / 2.4).sin();
                    for (k, &m) in slices_m.iter().enumerate() {
                        let x = xs[k] + 0.5 * step * mode_m[k] * swing;
                        let _ = s.set_points(m, &corners(&[v(x, y0), v(x, y1)]));
                    }
                },
                true,
            )?;
        }
        // two bars of an economy in a proportion that reproduces itself,
        // round after round: the even mix, seven tenths of itself each time
        let mix = numbers("K-E2.mix")?;
        let largest = numbers("K-E2.largest")?[0];
        let top = mix.iter().fold(0.0_f64, |m, x| m.max(*x));
        let mut rounds: Vec<Vec<Mob>> = Vec::new();
        for round in 0..3 {
            let mut pair = Vec::new();
            for (k, &share) in mix.iter().enumerate() {
                let h = 1.3 * share / top * largest.powi(round);
                let x = PANELS_X[2] - 1.2 + 1.2 * f64::from(round) + 0.38 * k as f64;
                let bar = Rectangle::new()
                    .width(0.3)
                    .height(h)
                    .style(flat(gold, 1.0))
                    .build()?
                    .moved_to(v(x, PANEL_Y - 0.65 + h / 2.0));
                pair.push(stage.arena_mut().add(bar));
            }
            rounds.push(pair);
        }
        let captions = [
            kit.text(stage, "a bridge", 28.0, dim, v(PANELS_X[0], -1.0))?,
            kit.text(stage, "the air in a room", 28.0, dim, v(PANELS_X[1], -1.0))?,
            kit.text(stage, "an economy", 28.0, dim, v(PANELS_X[2], -1.0))?,
        ];
        let swing_anchor = oscillate(stage, truss_pic.amp, deck.clock, 2.4, 0.0, 1.0)?;
        stage.add_to_scene(air_anchor)?;
        line.wait_for(stage, "A bridge has ways of moving")?;
        play_all(stage, vec![tween_linear(truss_pic.drawn, t.members.len() as f64, 1.2)?])?;
        fade_in_all(stage, &[captions[0]], 0.5)?;
        line.wait_for(stage, "So does the air in a room")?;
        let mut air = vec![air_box];
        air.extend(&slices);
        air.push(captions[1]);
        fade_in_all(stage, &air, 0.7)?;
        line.wait_for(stage, "an economy has a mix of its own")?;
        fade_in_all(stage, &[rounds[0][0], rounds[0][1], captions[2]], 0.5)?;
        fade_in_all(stage, &[rounds[1][0], rounds[1][1]], 0.5)?;
        fade_in_all(stage, &[rounds[2][0], rounds[2][1]], 0.5)?;

        // ---- 01_3: the title, and the motif
        stage.clear_updaters(swing_anchor, true);
        stage.clear_updaters(air_anchor, true);
        let mut off = air.clone();
        off.extend([captions[0], captions[2]]);
        off.extend(rounds.iter().flatten());
        play_all(stage, vec![tween_linear(truss_pic.drawn, 0.0, 0.6)?])?;
        truss_pic.stop(stage);
        fade_out_all(stage, &off, 0.6)?;
        let title = [
            kit.text(stage, "The One Idea Behind", 72.0, ink, v(0.0, 0.7))?,
            kit.text(stage, "Bridges, Concert Halls and Economies", 56.0, gold, v(0.0, -0.45))?,
        ];
        fade_in_all(stage, &title, 0.8)?;
        kit.listen(stage, 2.5, &format!("{}:01_3", kit.chapter))?;
        fade_out_all(stage, &title, 0.5)?;

        // ---- 01_4: it had been checked for a bounce, not for a sway
        let line = say(&kit, stage, "01_4")?;
        tween_all(stage, &[(deck.edges_on, 1.0), (deck.walkers_on, 1.0)], 0.5)?;
        let mut bounce = {
            let base = polyline(vec![v(-4.2, -2.6), v(-2.2, -2.6)])
                .style(faded_line(ink, 4.0, 1.0))
                .build();
            vec![stage.arena_mut().add(base)]
        };
        bounce.extend(double_arrow(stage, v(-3.2, -2.1), 0.0, 0.65));
        let bounce_icon = group(stage, &bounce)?;
        let tick = tick_mark(stage, v(-1.6, -2.2), 0.7, good(), 6.0)?;
        let bounce_caption = kit.text(stage, "up and down", 26.0, dim, v(-3.2, -3.35))?;
        let mut sway = {
            let base = polyline(vec![v(2.2, -2.6), v(4.2, -2.6)])
                .style(faded_line(ink, 4.0, 1.0))
                .build();
            vec![stage.arena_mut().add(base)]
        };
        sway.extend(double_arrow(stage, v(3.2, -2.1), 0.7, 0.0));
        let sway_icon = group(stage, &sway)?;
        let question = kit.text(stage, "?", 72.0, ink, v(1.5, -2.2))?;
        let sway_caption = kit.text(stage, "side to side", 26.0, dim, v(3.2, -3.35))?;
        line.wait_for(stage, "Its designers had checked how it would bounce")?;
        fade_in_all(stage, &[bounce_icon, tick, bounce_caption], 0.6)?;
        line.wait_for(stage, "said nothing of walkers pushing a bridge sideways")?;
        fade_in_all(stage, &[sway_icon, question, sway_caption], 0.6)?;

        // ---- 01_5: so what does a bridge have, besides strength?
        let line = say(&kit, stage, "01_5")?;
        tween_all(stage, &[(deck.sway, 0.0)], 1.0)?;
        let ask = question_card(
            &kit,
            stage,
            &["what does a bridge have, besides strength,", "that footsteps can find?"],
            36.0,
            gold,
            v(0.0, 2.3),
        )?;
        line.wait_for(stage, "So what does a bridge have")?;
        fade_in_all(stage, &[ask], 0.6)?;
        predict_hold(&kit, stage)?;

        // ---- 01_6: a way of moving all its own, and a walker not in step
        let line = say(&kit, stage, "01_6")?;
        // the deck's sideways swing as a gold curve at the rate of one mode
        // (built visible and not yet on the scene: fading it in brings it in
        // at the opacity it has)
        let curve = stage.arena_mut().add(
            polyline(edge(&CURVE, CURVE_AMP, deck_hz, 0.0, 1.0))
                .style(faded_line(gold, 6.0, 1.0))
                .build(),
        );
        let curve_anchor = stage.arena_mut().add(VMobject::new());
        let figure = walker_icon(stage, WALKER_AT)?;
        let t0 = stage.tracker_value(deck.clock).unwrap_or(0.0);
        let clock = deck.clock;
        // the clock is the deck's: keep it running while the deck is stopped
        let tick_anchor = stage.arena_mut().add(VMobject::new());
        {
            let mut seen = t0;
            stage.add_dt_updater(
                tick_anchor,
                move |s, _, dt| {
                    if dt > 0.0 {
                        seen += dt;
                        let _ = s.set_tracker_value(clock, seen);
                    }
                },
                false,
            )?;
            stage.add_to_scene(tick_anchor)?;
        }
        {
            let mut last = f64::NAN;
            let mut placed = 0.0_f64;
            stage.add_updater(
                curve_anchor,
                move |s, _| {
                    let t = s.tracker_value(clock).unwrap_or(0.0);
                    if t.to_bits() == last.to_bits() {
                        return;
                    }
                    last = t;
                    let _ = s.set_points(curve, &corners(&edge(&CURVE, CURVE_AMP, deck_hz, t, 1.0)));
                    // the walker, seen from behind, left, right and back
                    let x = WALKER_SWAY * (TAU * walker_hz * (t - t0)).sin();
                    s.shift(figure, v(x - placed, 0.0));
                    placed = x;
                },
                true,
            )?;
            stage.add_to_scene(curve_anchor)?;
        }
        let lowest = fact("K-B2.lateral_lowest_hz")?;
        let deck_rate = kit.text(
            stage,
            &format!("about {lowest} to {} swing a second", whole(walker_hz)),
            30.0,
            gold,
            v(CURVE.centre[0], 3.35),
        )?;
        let walker_rate = kit.text(
            stage,
            &format!("about {} a second", whole(walker_hz)),
            30.0,
            ink,
            v(WALKER_AT[0], -2.3),
        )?;
        let deck_caption = kit.text(stage, "swings", 24.0, dim, v(COUNTERS_X, 1.45))?;
        let walker_caption = kit.text(stage, "sways", 24.0, dim, v(COUNTERS_X, -1.55))?;
        line.wait_for(stage, "A way of moving all its own")?;
        // the deck and its icons go as the gold curve comes, in one change:
        // fading them out first left the screen bare for a second under the
        // line's first words
        let mut change: Vec<Box<dyn Animation>> = Vec::new();
        for m in [
            bounce_icon,
            tick,
            bounce_caption,
            sway_icon,
            question,
            sway_caption,
            ask,
        ] {
            change.push(fade_out_anim(stage, m, 0.35)?);
        }
        change.push(stage.prepare(tween_linear(deck.walkers_on, 0.0, 0.35)?)?);
        change.push(stage.prepare(tween_linear(deck.edges_on, 0.0, 0.35)?)?);
        for m in [curve, deck_rate] {
            change.push(fade_in_anim(stage, m, 0.6)?);
        }
        stage.play_prepared(change)?;
        deck.stop(stage);
        mark(&kit, stage, "01_6.curve_in");
        line.wait_for(stage, "Its sideways ones swing")?;
        fade_in_all(stage, &[deck_caption], 0.4)?;
        let zero_deck = stage.tracker_value(clock).unwrap_or(0.0);
        let deck_count = kit.live_value(
            stage,
            move |s| counted(s.tracker_value(clock).unwrap_or(0.0), zero_deck, deck_hz),
            0,
            false,
            64.0,
            gold,
            v(COUNTERS_X, 2.15),
            ORIGIN,
        )?;
        line.wait_for(stage, "And a walker sways left, right and back")?;
        fade_in_all(stage, &[figure, walker_rate, walker_caption], 0.6)?;
        let zero_walker = stage.tracker_value(clock).unwrap_or(0.0);
        let walker_count = kit.live_value(
            stage,
            move |s| counted(s.tracker_value(clock).unwrap_or(0.0), zero_walker, walker_hz),
            0,
            false,
            64.0,
            ink,
            v(COUNTERS_X, -0.85),
            ORIGIN,
        )?;
        kit.hold(stage)?;
        stage.clear_updaters(curve_anchor, true);
        stage.clear_updaters(tick_anchor, true);
        fade_out_all(stage, &[deck_count, walker_count], 0.4)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// Play several animations together (a helper so the chapter reads in the
/// order it is told).
fn play_all(stage: &mut Stage<'_>, anims: Vec<AnimBuilder>) -> fmn::Result<()> {
    let mut prepared: Vec<Box<dyn Animation>> = Vec::new();
    for a in anims {
        prepared.push(stage.prepare(a)?);
    }
    stage.play_prepared(prepared)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::super::marks;
    use super::*;

    /// 01_1: the dates are the facts' and read as the storyboard writes
    /// them; the closing is two days after the opening.
    #[test]
    fn the_dates_read_as_written_and_the_deck_closes_two_days_later() {
        assert_eq!(date_text("2000-06-10").as_deref(), Some("10 June 2000"));
        assert_eq!(day_month("2000-06-12").as_deref(), Some("12 June"));
        assert_eq!(date_text("2000-13-01"), None);
        assert_eq!(date_text("nonsense"), None);
        assert_eq!(date_text(fact("K-B2.opened").unwrap()).as_deref(), Some("10 June 2000"));
        assert_eq!(day_month(fact("K-B2.closed").unwrap()).as_deref(), Some("12 June"));
        assert_eq!(fact("K-B2.days_to_close").unwrap(), "2");
        assert_eq!(fact("K-B2.sway_cm").unwrap(), "7");
        assert_eq!(crowd_range(fact("K-B2.first_day").unwrap()), Some((80_000, 100_000)));
        assert_eq!(crowd_range("80000"), None);
    }

    /// 01_6: the rates are the measured sideways modes' (the one the deck
    /// swings at is among them) and the walker's; they are not equal, so
    /// the two counters drift apart and never lock.
    #[test]
    fn the_two_counters_have_different_rates_so_they_drift() {
        let lateral = numbers("K-B2.lateral_hz").unwrap();
        assert_eq!(lateral, [0.48, 0.8, 0.95, 1.03]);
        let walker = numbers("K-B2.walker_lateral_hz").unwrap()[0];
        assert_eq!(walker, 1.0);
        assert!(lateral[2] != walker);
        // half a swing out of step after 1 / (2 |f - g|) seconds: inside the line
        let drift = 1.0 / (2.0 * (walker - lateral[2]).abs());
        assert!((5.0..=15.0).contains(&drift), "{drift} s");
        // "between half a swing and one swing a second": the lowest is 0.5 and the walker 1
        assert_eq!(fact("K-B2.lateral_lowest_hz").unwrap(), "0.5");
    }

    /// 01_1: frame one is the whole swinging deck (R1 C4), not a black second
    /// and a deck that comes in behind the date; 01_6: the deck and the gold
    /// curve change places in one 0.6 s change, so the first words of the
    /// line are not said over a bare screen.
    #[test]
    fn frame_one_is_the_deck_and_the_curve_comes_as_the_deck_goes() {
        let voiced = testing::run_voiced("01_the_bridge_that_swayed").unwrap();
        let tag = &voiced.tag;
        assert_eq!(marks::at(tag, "01_1.deck_whole"), 0.0);
        let (_, start, _) = voiced.lines.iter().find(|(id, ..)| id == "01_6").cloned().unwrap();
        let curve = marks::at(tag, "01_6.curve_in");
        assert!(
            curve - start <= 0.8,
            "the curve is whole {:.2} s into 01_6",
            curve - start
        );
    }

    /// 01_1: the sway's reading stands above the highest the deck's top edge
    /// reaches under it, at every moment of every measured swing, and above
    /// the bracket it belongs to; the lower edge never goes through it.
    #[test]
    fn the_sway_reading_stands_clear_of_the_deck_at_every_sway() {
        let at = reading_at(&DECK);
        let bottom = at[1] - READING_HALF.1;
        assert!(bottom > DECK.centre[1] + SWAY, "{bottom}: above the bracket's top");
        for hz in numbers("K-B2.lateral_hz").unwrap() {
            for step in 0..400 {
                let t = f64::from(step) * 0.01 / hz;
                for side in [1.0, -1.0] {
                    let top = edge(&DECK, SWAY, hz, t, side)
                        .iter()
                        .filter(|p| (p[0] - at[0]).abs() <= READING_HALF.0)
                        .fold(f64::NEG_INFINITY, |m, p| m.max(p[1]));
                    assert!(
                        top < bottom,
                        "edge {side} at {top:.2} reaches the reading ({bottom:.2}) at t = {t:.2}"
                    );
                }
            }
        }
    }

    /// 01_6: each counter opens at 0 at the moment it appears and adds one
    /// for every swing (sway) after that, whatever the clock read before.
    #[test]
    fn the_counters_open_at_zero_and_count_their_own_rate() {
        assert_eq!(counted(41.2, 41.2, 0.95), 0.0);
        assert_eq!(counted(41.2 + 1.0, 41.2, 0.95), 0.0);
        assert_eq!(counted(41.2 + 1.1, 41.2, 0.95), 1.0);
        assert_eq!(counted(41.2 + 10.0, 41.2, 1.0), 10.0);
        // a clock that reads earlier than the zero never counts below it
        assert_eq!(counted(3.0, 5.0, 1.0), 0.0);
    }

    /// 01_2: the air's lowest mode is one hump, as large at midspan as it
    /// gets, symmetric about the middle and with no sign change.
    #[test]
    fn the_air_sloshes_in_one_hump() {
        let m = lowest_air_mode(SLICES);
        assert_eq!(m.len(), SLICES);
        let peak = m.iter().fold(0.0_f64, |a, x| a.max(x.abs()));
        assert!((peak - 1.0).abs() < 1e-12);
        assert!(m.iter().all(|&x| x > 0.0), "one hump, no sign change");
        for k in 0..SLICES / 2 {
            assert!((m[k] - m[SLICES - 1 - k]).abs() < 1e-9, "symmetric at {k}");
        }
        assert!(m[SLICES / 2] > m[0], "largest in the middle");
    }

    /// The economy's even mix is the same proportion every round, seven
    /// tenths of itself.
    #[test]
    fn the_economys_mix_comes_back_in_the_same_proportion() {
        let mix = numbers("K-E2.mix").unwrap();
        assert_eq!(mix, [1.0, 1.0]);
        assert_eq!(numbers("K-E2.largest").unwrap(), [0.7]);
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("01_the_bridge_that_swayed").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "10 June 2000",
            "7 cm",
            "closed 12 June",
            "the first day:",
            "80,000 to 100,000 people",
            "a bridge",
            "the air in a room",
            "an economy",
            "The One Idea Behind",
            "Bridges, Concert Halls and Economies",
            "up and down",
            "side to side",
            "what does a bridge have, besides strength,",
            "about 0.5 to 1 swing a second",
            "about 1 a second",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // the length that counts is the one with the scratch voice placed
        let voiced = testing::run_voiced("01_the_bridge_that_swayed").unwrap();
        let s = voiced.ran.seconds;
        eprintln!("01_the_bridge_that_swayed: {s:.1} s on the scratch voice");
        assert!(s > 45.0 && s < 85.0, "{s} s");
    }
}
