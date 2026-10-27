//! Chapter 01, the stop. You take a bus to the station to catch a train,
//! and you never check the timetable (01_1). Two routes, each every twelve
//! minutes exactly, the red bus two minutes after the blue (01_2); the sign
//! is true, ten buses an hour, a bus every six minutes on average (01_3).
//! Your train won't wait: a lucky morning, then on another day an unlucky
//! one, each wait as you arrive a bar that shrinks to the bus, sped up
//! (01_4). The fair-looking guess, half of six;
//! three minutes, more or less? (01_5, a prediction.) More: four minutes
//! twenty with every bus on time (01_6). Nothing is late; what the video
//! will do; the title (01_7).
//!
//! No derivation here (the owner's direction of 2026-10-24): the chapter
//! sets the stage and poses the puzzle. Every number on screen is read from
//! the facts, and the scene refuses to render when a spoken claim stops
//! holding.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, live_bar, rect, tween, v};
use fmn::prelude::*;

use super::draw::{
    LINE_Y, PASSENGER_R, bus_marker, fact, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flash_all,
    flash_lagged, flat, fly, group, hour_line, line_style, marker_top, min_s, now, pause, read_s, say, shrink_to,
    stand, steady, think, wait_until, x_of, you,
};
use super::glyphs::TITLE;
use crate::facts;
use crate::model::{self, HOUR, OFFSET, PERIOD, Q};
use crate::palette;

pub struct TheStop {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are read from the facts)
const HOME: &str = "home";
const STOP: &str = "bus stop";
const STATION: &str = "station";
const SIGN_HEAD: &str = "To the station";
const CHOICE_LESS: &str = "less";
const CHOICE_MORE: &str = "more";
const ON_TIME: &str = "every bus on time";
/// 01_4: a bar is the wait as you arrive; each morning runs sped up by its
/// own amount, on its own day.
const ON_ARRIVAL: &str = "as you arrive";
const ONE_MORNING: &str = "one morning, sped up";
const ANOTHER_MORNING: &str = "another morning, sped up";
/// The exact version of the chapter's claim (K01).
const K01: &str = "average over moments of turning up spread evenly over the hour, with no look at the \
                   timetable; each wait runs to the next bus of either route; every bus exactly on time";

// ------------------------------------------------------------------ layout

/// 01_1: the street, full size, then shrunk to a strip at the top.
const STREET_Y: f64 = 0.0;
const HOME_X: f64 = -5.0;
const STOP_X: f64 = 0.0;
const STATION_X: f64 = 5.0;
const GROUND_HALF: f64 = 6.5;
const STREET_LABEL_DY: f64 = -0.8;
const STREET_LABEL_SIZE: f64 = 20.0;
const STRIP_SCALE: f64 = 0.5;
const STRIP_AT: Vec3 = [0.0, 3.05, 0.0];
/// 01_3: the sign's card, lower left, clear of the bar labels above it and
/// of K01's three-line lower-third (top near y = −2.42) below it.
const SIGN_AT: Vec3 = [-4.3, -1.55, 0.0];
const SIGN_W: f64 = 3.7;
const SIGN_H: f64 = 1.4;
const SIGN_YS: [f64; 3] = [-1.1, -1.55, -1.95];
const SIGN_SIZES: [f64; 2] = [22.0, 20.0];
/// 01_4: your two mornings' bars under the clock labels, one morning at a
/// time, each label under the middle of its bar and the morning's note to
/// the right of the :12 bus, on the bars' row.
const BAR_Y: f64 = LINE_Y - 0.78;
const BAR_H: f64 = 0.16;
const BAR_LABEL_DY: f64 = -0.32;
const NOTE_AT: Vec3 = [-1.4, BAR_Y, 0.0];
const NOTE_SIZE: f64 = 18.0;
const GHOST_DIM: f64 = 0.45;
const GROW_RT: f64 = 0.6;
const LUCKY_RT: f64 = 1.0;
const UNLUCKY_RT: f64 = 2.6;
/// 01_5, 01_6: the guess, the three chips, the answer and the extra time,
/// right of centre, all above K01's lower-third.
const PANEL_X: f64 = 3.7;
const GUESS_Y: f64 = -0.35;
const CHIP_Y: f64 = -0.95;
const CHIP_DX: f64 = 1.35;
const CHIP_W: f64 = 1.15;
const CHIP_H: f64 = 0.42;
const ANSWER_Y: f64 = -1.55;
const EXTRA_Y: f64 = -2.0;
const PANEL_SIZE: f64 = 24.0;
const THINK_AT: Vec3 = [0.2, -1.6, 0.0];
/// 01_6: a green dot over every bus.
const ON_TIME_DY: f64 = 0.18;
const ON_TIME_AT: Vec3 = [0.0, 1.55, 0.0];
/// 01_7: the three icons of what the video will do.
const ICON_XS: [f64; 3] = [-2.2, 0.0, 2.2];
const ICON_Y: f64 = -1.6;
/// The title card's two lines.
const TITLE_YS: [f64; 2] = [0.45, -0.45];
const TITLE_SIZE: f64 = 44.0;
const TITLE_HOLD: f64 = 2.0;

/// The street of 01_1, detached: the ground, the house, the stop with its
/// sign, the station with its train and clock, and their labels.
fn street(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let y = STREET_Y;
    let ground = Line::new(v(-GROUND_HALF, y - 0.4), v(GROUND_HALF, y - 0.4))
        .style(line_style(Palette::faint(), 2.0))
        .build()?;
    let house = rect(v(HOME_X - 0.5, y - 0.4), 1.0, 0.8, flat(Palette::faint(), 1.0))?;
    let roof = Polygon::new([
        v(HOME_X - 0.65, y + 0.4),
        v(HOME_X + 0.65, y + 0.4),
        v(HOME_X, y + 0.95),
    ])
    .style(flat(Palette::dim(), 1.0))
    .build();
    let post = Line::new(v(STOP_X, y - 0.4), v(STOP_X, y + 0.85))
        .style(line_style(Palette::dim(), 3.0))
        .build()?;
    let plate = rect(v(STOP_X - 0.32, y + 0.85), 0.64, 0.36, flat(Palette::dim(), 1.0))?;
    let station = rect(v(STATION_X - 1.0, y - 0.4), 2.0, 1.1, flat(Palette::faint(), 1.0))?;
    let train = rect(v(STATION_X - 0.8, y - 0.32), 1.6, 0.34, flat(Palette::dim(), 1.0))?;
    let clock = Circle::new()
        .radius(0.2)
        .arc_center(v(STATION_X, y + 0.98))
        .style(line_style(Palette::dim(), 2.5))
        .build();
    let hand = Line::new(v(STATION_X, y + 0.98), v(STATION_X, y + 1.12))
        .style(line_style(Palette::dim(), 2.5))
        .build()?;
    let mut parts: Vec<Mob> = [ground, house, roof, post, plate, station, train, clock, hand]
        .into_iter()
        .map(|m| stage.arena_mut().add(m))
        .collect();
    for (word, x) in [(HOME, HOME_X), (STOP, STOP_X), (STATION, STATION_X)] {
        parts.push(kit.text(
            stage,
            word,
            STREET_LABEL_SIZE,
            Palette::dim(),
            v(x, y + STREET_LABEL_DY),
        )?);
    }
    Ok(parts)
}

/// A chip: a word in an outlined box (01_5's three answers).
fn chip(kit: &Kit, stage: &mut Stage<'_>, word: &str, x: f64) -> fmn::Result<[Mob; 2]> {
    let frame = rect(
        v(x - CHIP_W / 2.0, CHIP_Y - CHIP_H / 2.0),
        CHIP_W,
        CHIP_H,
        line_style(Palette::dim(), 2.0),
    )?;
    let frame = stage.arena_mut().add(frame);
    let label = kit.text(stage, word, 22.0, Palette::ink(), v(x, CHIP_Y))?;
    Ok([frame, label])
}

/// 01_7's three icons: a tag over a dot (two ways to count), a gap with its
/// two ends (stretch a gap), a triangle (the waits as a picture).
fn icons(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let [a, b, c] = ICON_XS;
    let y = ICON_Y;
    let tag = rect(v(a - 0.18, y + 0.12), 0.36, 0.22, flat(Palette::dim(), 1.0))?;
    let dot = Dot::new()
        .point(v(a, y - 0.22))
        .radius(PASSENGER_R)
        .style(flat(Palette::dim(), 1.0))
        .build();
    let gap = Line::new(v(b - 0.5, y), v(b + 0.5, y))
        .style(line_style(Palette::dim(), 3.0))
        .build()?;
    let ends = [
        Line::new(v(b - 0.5, y - 0.15), v(b - 0.5, y + 0.15))
            .style(line_style(Palette::dim(), 3.0))
            .build()?,
        Line::new(v(b + 0.5, y - 0.15), v(b + 0.5, y + 0.15))
            .style(line_style(Palette::dim(), 3.0))
            .build()?,
    ];
    let triangle = Polygon::new([v(c - 0.4, y - 0.3), v(c + 0.4, y - 0.3), v(c - 0.4, y + 0.5)])
        .style(flat(Palette::dim(), 1.0))
        .build();
    let [e0, e1] = ends;
    Ok([tag, dot, gap, e0, e1, triangle]
        .into_iter()
        .map(|m| stage.arena_mut().add(m))
        .collect())
}

/// One of your mornings (01_4), drawn on its own: a ghost of you at `t`;
/// the wait as you arrive as a bar that reaches from the next bus back to
/// you, its label under the bar and the morning's `note` beside it; then
/// the bar shrinks to the bus at a steady rate over `rt` seconds, the wait
/// running out, sped up. Each morning is sped up by its own amount (the
/// note says so), so the two bars share no clock. Both mornings fall in a
/// long gap (chapter 04 meets them again as its first and last passengers),
/// so both bars are gold. Returns the ghost (it stays) and what leaves.
fn morning(kit: &Kit, stage: &mut Stage<'_>, t: Q, label: &str, note: &str, rt: f64) -> fmn::Result<(Mob, Vec<Mob>)> {
    let color = palette::long_gap();
    let wait = model::wait_at(OFFSET, t);
    let bus = t + wait;
    let ghost = you(stage, v(x_of(t), LINE_Y))?;
    fade_in_all(stage, &[ghost], 0.3)?;
    // the bar hangs left from the bus: its length is the wait still ahead
    let unit = x_of(Q::int(1)) - x_of(Q::int(0));
    let left = stage.add_value_tracker(0.0);
    let bar = live_bar(stage, left, v(x_of(bus), BAR_Y), unit, BAR_H, color)?;
    stage.play(tween(left, -wait.to_f64(), GROW_RT)?)?;
    let middle = (x_of(t) + x_of(bus)) / 2.0;
    let words = [
        kit.text(stage, label, 20.0, color, v(middle, BAR_Y + BAR_LABEL_DY))?,
        kit.text(stage, note, NOTE_SIZE, Palette::dim(), NOTE_AT)?,
    ];
    fade_in_all(stage, &words, 0.3)?;
    stage.play(steady(left, 0.0, rt)?)?;
    let mut gone = vec![bar];
    gone.extend(words);
    Ok((ghost, gone))
}

/// The most opaque alpha in one of a part's colour columns (`fill_rgba`,
/// `stroke_rgba`), 0 for a part without it.
fn alpha(stage: &Stage<'_>, part: Mob, field: &str) -> f32 {
    stage
        .get(part)
        .and_then(|e| e.buffer.read_column(field))
        .map_or(0.0, |lanes| {
            lanes.iter().skip(3).step_by(4).copied().fold(0.0, f32::max)
        })
}

/// The parts of `mob` that paint: the members of its family with no
/// members of their own.
fn leaves(stage: &Stage<'_>, mob: Mob) -> Vec<Mob> {
    stage
        .family(mob)
        .into_iter()
        .filter(|&m| stage.family(m).len() == 1)
        .collect()
}

/// Dim a ghost of you (01_4) to [`GHOST_DIM`] over `rt` seconds, each part
/// by what it paints with: a filled part (the dot) by its opacity, a part
/// with no fill (the ring) by its stroke alone. `fade_to` on the whole ghost
/// writes every alpha of its family at the pinned fmn, the ring's empty
/// fill included, and the dimmed ring became a gold disc over the markers
/// beside it.
fn dim_ghost(stage: &mut Stage<'_>, ghost: Mob, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for part in leaves(stage, ghost) {
        let dim = if alpha(stage, part, "fill_rgba") > 0.0 {
            fade_to(part, GHOST_DIM, rt)?
        } else {
            part.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .set_stroke(None, None, Some(GHOST_DIM))?
        };
        anims.push(stage.prepare(dim)?);
    }
    if anims.is_empty() {
        return Err(fail("the ghost has no part to dim".into()));
    }
    stage.play_prepared(anims)?;
    Ok(())
}

impl SceneConstruct for TheStop {
    fn name(&self) -> &str {
        "01_the_stop"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts::facts().map_err(fail)?;
        let buses_per_hour = fact(&f, "F-T1.buses_per_hour")?;
        let mean_gap = fact(&f, "F-T1.mean_gap")?;
        let half_gap = fact(&f, "F-T1.half_mean_gap")?;
        let wait = min_s(&fact(&f, "F-W1.mean_wait_clock")?)?;
        let extra = min_s(&fact(&f, "F-W1.extra_clock")?)?;
        // the guess must be below the true average, or "More." is false
        if model::mean_gap(&model::hour_gaps(OFFSET)) / Q::int(2) >= model::mean_wait(&model::hour_gaps(OFFSET)) {
            return Err(fail(
                "the guess of half the average gap is not below the average wait".into(),
            ));
        }

        // 01_1: the street; you walk from home to the stop
        let line = say(&kit, stage, "01_1")?;
        let parts = street(&kit, stage)?;
        fade_in_lagged(stage, &parts, 1.2, 0.15)?;
        let me = you(stage, v(HOME_X, STREET_Y - 0.25))?;
        fade_in_all(stage, &[me], 0.3)?;
        line.wait_for(stage, "You walk to the stop")?;
        stage.play(fly(me, v(STOP_X - 0.35, STREET_Y - 0.25), 2.0)?)?;

        // 01_2: the street becomes a strip at the top, you still at the stop;
        // the hour, then the blue buses and the red
        let line = say(&kit, stage, "01_2")?;
        let mut members = parts.clone();
        members.push(me);
        let strip = group(stage, &members)?;
        stage.play(shrink_to(strip, STRIP_SCALE, STRIP_AT, 1.0)?)?;
        let marks: Vec<i64> = (0..=HOUR).step_by(PERIOD as usize).collect();
        let hour = hour_line(&kit, stage, &marks)?;
        fade_in_all(stage, &hour.mobs(), 0.6)?;
        line.wait_for(stage, "Two bus routes")?;
        let blue: Vec<Mob> = model::route(0)
            .into_iter()
            .map(|t| bus_marker(stage, t, palette::blue_bus()))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &blue, 1.0, 0.25)?;
        line.wait_for(stage, "the red bus")?;
        let red: Vec<Mob> = model::route(OFFSET)
            .into_iter()
            .map(|t| bus_marker(stage, t, palette::red_bus()))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &red, 1.0, 0.25)?;

        // 01_3: the sign's two promises; every bus pulses, left to right
        let line = say(&kit, stage, "01_3")?;
        let card = rect(
            v(SIGN_AT[0] - SIGN_W / 2.0, SIGN_AT[1] - SIGN_H / 2.0),
            SIGN_W,
            SIGN_H,
            line_style(Palette::dim(), 2.0),
        )?;
        let card = stage.arena_mut().add(card);
        let sign = [
            card,
            kit.text(
                stage,
                SIGN_HEAD,
                SIGN_SIZES[0],
                Palette::ink(),
                v(SIGN_AT[0], SIGN_YS[0]),
            )?,
            kit.text(
                stage,
                &format!("{buses_per_hour} buses an hour"),
                SIGN_SIZES[1],
                Palette::ink(),
                v(SIGN_AT[0], SIGN_YS[1]),
            )?,
            kit.text(
                stage,
                &format!("every {mean_gap} minutes on average"),
                SIGN_SIZES[1],
                Palette::ink(),
                v(SIGN_AT[0], SIGN_YS[2]),
            )?,
        ];
        fade_in_all(stage, &sign[..2], 0.4)?;
        line.wait_for(stage, "Ten buses")?;
        fade_in_all(stage, &sign[2..], 0.5)?;
        let mut all_buses: Vec<(i64, Mob)> = model::route(0).into_iter().zip(blue.iter().copied()).collect();
        all_buses.extend(model::route(OFFSET).into_iter().zip(red.iter().copied()));
        all_buses.sort_by_key(|&(t, _)| t);
        let in_order: Vec<Mob> = all_buses.iter().map(|&(_, m)| m).collect();
        if in_order.len().to_string() != buses_per_hour {
            return Err(fail("the markers are not the sign's buses an hour".into()));
        }
        flash_lagged(stage, &in_order, 1.4, 0.15)?;

        // 01_4: your train won't wait; a lucky morning, cleared, then an
        // unlucky one on another day
        let line = say(&kit, stage, "01_4")?;
        line.wait_for(stage, "Some mornings a bus")?;
        let lucky_words = format!("wait {} {ON_ARRIVAL}", fact(&f, "F-M1.lucky_wait_clock")?);
        let (lucky, lucky_gone) = morning(&kit, stage, model::lucky_morning(), &lucky_words, ONE_MORNING, LUCKY_RT)?;
        fade_out_all(stage, &lucky_gone, 0.3)?;
        dim_ghost(stage, lucky, 0.3)?;
        line.wait_for(stage, "Some mornings you've")?;
        let unlucky_words = format!("wait {} {ON_ARRIVAL}", fact(&f, "F-M1.unlucky_wait_clock")?);
        let (unlucky, unlucky_gone) = morning(
            &kit,
            stage,
            model::unlucky_morning(),
            &unlucky_words,
            ANOTHER_MORNING,
            UNLUCKY_RT,
        )?;
        fade_out_all(stage, &unlucky_gone, 0.4)?;
        dim_ghost(stage, unlucky, 0.4)?;

        // 01_5: the guess, the three chips, the prediction
        let line = say(&kit, stage, "01_5")?;
        let guess = kit.text(
            stage,
            &format!("guess: half of {mean_gap} = {half_gap} min"),
            PANEL_SIZE,
            Palette::dim(),
            v(PANEL_X, GUESS_Y),
        )?;
        fade_in_all(stage, &[guess], 0.5)?;
        line.wait_for(stage, "will your average wait")?;
        let chips = [
            chip(&kit, stage, CHOICE_LESS, PANEL_X - CHIP_DX)?,
            chip(&kit, stage, &format!("{half_gap} min"), PANEL_X)?,
            chip(&kit, stage, CHOICE_MORE, PANEL_X + CHIP_DX)?,
        ];
        let chip_mobs: Vec<Mob> = chips.iter().flatten().copied().collect();
        fade_in_lagged(stage, &chip_mobs, 0.6, 0.2)?;
        let ask = think(&kit, stage, THINK_AT)?;

        // 01_6: more; the true average and its exact version; the extra time;
        // a green dot over every bus, all on time
        let line = say(&kit, stage, "01_6")?;
        fade_out_all(stage, &[ask], 0.2)?;
        let lit = rect(
            v(PANEL_X + CHIP_DX - CHIP_W / 2.0, CHIP_Y - CHIP_H / 2.0),
            CHIP_W,
            CHIP_H,
            flat(palette::you(), 0.3),
        )?;
        let lit = stage.arena_mut().add(lit);
        fade_in_all(stage, &[lit], 0.3)?;
        flash_all(stage, &[chips[2][1]], 0.5)?;
        line.wait_for(stage, "Four minutes")?;
        let answer = kit.text(
            stage,
            &format!("average wait: {wait}"),
            PANEL_SIZE,
            palette::verdict(),
            v(PANEL_X, ANSWER_Y),
        )?;
        let k01 = precisely(&kit, stage, K01)?;
        fade_in_all(stage, &[answer, k01], 0.5)?;
        let card_up = now(stage);
        line.wait_for(stage, "exactly on time")?;
        let mut checks: Vec<Mob> = all_buses
            .iter()
            .map(|&(t, _)| {
                let at = marker_top(t);
                stage.arena_mut().add(
                    Dot::new()
                        .point(v(at[0], at[1] + ON_TIME_DY))
                        .radius(PASSENGER_R)
                        .style(flat(palette::verdict(), 1.0))
                        .build(),
                )
            })
            .collect();
        checks.push(kit.text(stage, ON_TIME, 20.0, palette::verdict(), ON_TIME_AT)?);
        fade_in_lagged(stage, &checks, 0.8, 0.1)?;
        line.wait_for(stage, "an extra minute")?;
        let plus = kit.text(
            stage,
            &format!("+ {extra}"),
            PANEL_SIZE,
            palette::you(),
            v(PANEL_X, EXTRA_Y),
        )?;
        fade_in_all(stage, &[plus], 0.4)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(K01))?;

        // 01_7: nothing is late; the sign and the panel leave, the ghosts
        // glow; what the video will do; then the title
        let line = say(&kit, stage, "01_7")?;
        let mut panel = vec![guess, lit, answer, plus, k01];
        panel.extend(&chip_mobs);
        panel.extend(sign);
        fade_out_all(stage, &panel, 0.5)?;
        flash_all(stage, &[lucky, unlucky], 0.8)?;
        line.wait_for(stage, "By the end")?;
        let marks_of_what_comes = icons(stage)?;
        fade_in_lagged(stage, &marks_of_what_comes, 1.2, 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_all(stage, 0.6)?;
        let split = TITLE
            .find(" Than")
            .ok_or_else(|| fail("the title has no second half".into()))?;
        let (first, second) = TITLE.split_at(split);
        let title = [
            kit.text(stage, first, TITLE_SIZE, Palette::ink(), v(0.0, TITLE_YS[0]))?,
            kit.text(
                stage,
                second.trim_start(),
                TITLE_SIZE,
                Palette::ink(),
                v(0.0, TITLE_YS[1]),
            )?,
        ];
        fade_in_all(stage, &title, 0.5)?;
        pause(stage, TITLE_HOLD)?;
        fade_out_all(stage, &title, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use explainer_kit::{Narrator, Script};

    use super::*;

    /// Two ghosts of you, measured: each painted part's (fill, stroke) alpha
    /// once the first ghost is whole, then after `dim_ghost` on it, and
    /// after the old dimming, `fade_to` on the whole ghost, on the second.
    type PartAlphas = Vec<(f32, f32)>;
    type GhostMeasurements = Rc<RefCell<Vec<PartAlphas>>>;

    struct Ghosts {
        seen: GhostMeasurements,
    }

    /// Each painted part of `ghost`: its (fill, stroke) alpha.
    fn parts(stage: &Stage<'_>, ghost: Mob) -> Vec<(f32, f32)> {
        leaves(stage, ghost)
            .into_iter()
            .map(|p| (alpha(stage, p, "fill_rgba"), alpha(stage, p, "stroke_rgba")))
            .collect()
    }

    impl SceneConstruct for Ghosts {
        fn name(&self) -> &str {
            "ghosts"
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            let dimmed = you(stage, v(-1.0, LINE_Y))?;
            let faded = you(stage, v(1.0, LINE_Y))?;
            fade_in_all(stage, &[dimmed, faded], 0.3)?;
            let whole = parts(stage, dimmed);
            dim_ghost(stage, dimmed, 0.3)?;
            stage.play(fade_to(faded, GHOST_DIM, 0.3)?)?;
            *self.seen.borrow_mut() = vec![whole, parts(stage, dimmed), parts(stage, faded)];
            Ok(())
        }
    }

    /// A dimmed ghost of you keeps its shape: the dot's fill and the ring's
    /// stroke dim to GHOST_DIM, and the ring stays an outline with no fill.
    /// The control is the old dimming, `fade_to` on the whole ghost: at the
    /// pinned fmn it gives the ring's empty fill the same alpha, the gold
    /// disc of root's inspection-opening03 render (01_4 on). If the control
    /// fails, family opacity no longer fills an outline, and `dim_ghost` can
    /// go back to `fade_to`.
    #[test]
    fn a_dimmed_ghost_keeps_its_ring_an_outline() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let mut scene = Ghosts { seen: Rc::clone(&seen) };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let seen = seen.borrow();
        let [whole, dimmed, faded] = &seen[..] else {
            panic!("{} measurements, not 3", seen.len());
        };
        let near = |a: f32, b: f64| (f64::from(a) - b).abs() < 1e-6;
        // you, whole: a filled dot, and a ring with a stroke and no fill
        assert_eq!(whole.len(), 2, "{whole:?}");
        let ring = whole
            .iter()
            .position(|&(fill, stroke)| near(fill, 0.0) && near(stroke, 1.0))
            .expect("you have a ring with no fill");
        let dot = 1 - ring;
        assert!(near(whole[dot].0, 1.0), "{whole:?}");
        // dimmed: the dot's fill and the ring's stroke; the ring stays unfilled
        assert!(near(dimmed[dot].0, GHOST_DIM), "{dimmed:?}");
        assert!(near(dimmed[ring].1, GHOST_DIM), "{dimmed:?}");
        assert!(near(dimmed[ring].0, 0.0), "the dimmed ring took a fill: {dimmed:?}");
        // the control: the family fade fills the ring
        assert!(
            near(faded[ring].0, GHOST_DIM),
            "the family fade no longer fills the ring: {faded:?}"
        );
    }

    /// The whole chapter runs headless at 10 fps with the scratch voice:
    /// every phrase it waits for is in its line, every fact it reads exists,
    /// every builder takes its arguments, and its seven lines are said in
    /// order, the answer after the question. It ends with `kit.finish`: the
    /// chapter leaves its timeline, and the timeline runs past the end of
    /// the last line.
    #[test]
    fn the_chapter_runs_and_says_its_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = Narrator::estimate(&script);
        let kit = Kit::new().unwrap().with_narrator(Rc::clone(&narrator));
        let mut scene = TheStop { kit };
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut scene, config, 7, &mut fmn::scene::NullSceneSink).unwrap();
        let said_at = narrator.said();
        let said: Vec<&str> = said_at.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(said, ["01_1", "01_2", "01_3", "01_4", "01_5", "01_6", "01_7"]);
        let (last, at) = said_at.last().unwrap();
        let last_ends = at + narrator.length(last).unwrap();
        let end = scene
            .kit
            .finished
            .borrow()
            .as_ref()
            .expect("no timeline: the chapter must end with kit.finish")
            .seconds();
        assert!(
            end > last_ends,
            "the timeline ends at {end:.2} s, before {last} ends at {last_ends:.2} s"
        );
    }

    /// The card is the storyboard's exact version of the chapter's claim
    /// (K01), word for word.
    #[test]
    fn the_card_is_the_storyboards_exact_version() {
        let board = crate::STORYBOARD.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(board.contains(&format!("\"{K01}\"")), "{K01}");
    }

    /// The chapter's fixed words carry no digits: every number on screen is
    /// read from the facts.
    #[test]
    fn the_fixed_words_carry_no_numbers() {
        for w in [
            HOME,
            STOP,
            STATION,
            SIGN_HEAD,
            CHOICE_LESS,
            CHOICE_MORE,
            ON_TIME,
            ON_ARRIVAL,
            ONE_MORNING,
            ANOTHER_MORNING,
            K01,
        ] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }
}
