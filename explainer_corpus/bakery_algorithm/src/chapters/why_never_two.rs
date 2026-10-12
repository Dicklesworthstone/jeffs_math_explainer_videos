//! Chapter 08, why never two: the proof, on two lanes. Blue is in the room
//! and Green holds a ticket (08_1). Fact one: whoever starts choosing after
//! you finished takes a bigger ticket, because she saw yours (08_2). Fact
//! two: everyone else holding a ticket has a bigger pair than the customer
//! in the room (08_3). Why: take Blue's last look at Green's flag, the one
//! that let her through, and slide Green's choosing along the time axis
//! against it (08_4). The look lands before, inside or after it: before
//! gives fact one (08_5), inside is impossible (08_6), after means Blue read
//! Green's real ticket (08_7). Two in the room would each hold the smaller
//! pair: impossible (08_8). The search agrees for small cases (08_9).
//!
//! The picture is schematic (no trace): it shows the order of events the
//! proof reasons about, never a number. The exact statement is in the
//! description, "Precisely" 08 (memo §4, corrected at R2).

use std::rc::Rc;

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, tween, v};
use fmn::animation::indicate;
use fmn::library::{TransformMatchingTex, checkmark, exmark};
use fmn::prelude::*;

use crate::facts::recorded_sweep;

use super::draw::{
    NAMES, access_arrow, badge, chalk, customer, faded_line, flag_amber, flat, group, lane, line_style, order_gold,
    pictogram, rgb, room_red, rule_tick, say,
};
use super::glyphs;

// ------------------------------------------------------- the proof picture

/// Where the lanes begin (after the badges) and end ("now": Blue is still
/// in the room).
pub const X0: f64 = -5.6;
pub const X1: f64 = 6.7;
/// Blue's lane and Green's.
pub const Y_BLUE: f64 = 0.85;
pub const Y_GREEN: f64 = -0.95;
/// Blue's choosing: from raising her flag to lowering it, her ticket written
/// just before it comes down.
pub const BLUE_CHOOSING: (f64, f64) = (-5.0, -3.0);
/// Blue's last look at Green's flag (the one that let her through), her last
/// glance at Green's slate, and the moment she walks in.
pub const X_CHECK: f64 = -0.35;
pub const X_GLANCE: f64 = 0.45;
pub const X_ROOM: f64 = 1.25;
/// How long Green's choosing lasts on screen.
pub const GREEN_W: f64 = 2.1;

/// Where Green might have been at Blue's last look at her flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// She had not started choosing yet.
    NotYet,
    /// She was in the middle of choosing: her flag was up.
    Choosing,
    /// She had finished choosing: her ticket was written.
    Done,
}

impl Place {
    pub const ALL: [Self; 3] = [Self::NotYet, Self::Choosing, Self::Done];

    /// Green's choosing starts here (scene x) in this case.
    pub fn green_start(self) -> f64 {
        match self {
            Self::NotYet => 1.6,
            Self::Choosing => -1.4,
            Self::Done => -2.75,
        }
    }

    /// The place of a look at `x` against choosing that starts at `start`.
    pub fn of(x: f64, start: f64) -> Self {
        if x < start {
            Self::NotYet
        } else if x <= start + GREEN_W {
            Self::Choosing
        } else {
            Self::Done
        }
    }

    pub fn words(self) -> &'static str {
        match self {
            Self::NotYet => glyphs::NOT_YET,
            Self::Choosing => glyphs::CHOOSING,
            Self::Done => glyphs::DONE_CHOOSING,
        }
    }

    fn color(self) -> Srgb {
        match self {
            Self::NotYet => Palette::dim(),
            Self::Choosing => flag_amber(),
            Self::Done => chalk(),
        }
    }
}

/// A band along a lane: choosing (amber), the room (red).
fn band(x0: f64, x1: f64, y: f64, color: Srgb, o: f64) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(x1 - x0)
        .height(0.3)
        .corner_radius(0.08)
        .style(flat(color, o))
        .build()?
        .moved_to(v((x0 + x1) / 2.0, y)))
}

/// The thin chalk line of a held ticket, from its write to `x1`.
fn holding(x0: f64, x1: f64, y: f64, o: f64) -> fmn::Result<VMobject> {
    Ok(Line::new(v(x0, y - 0.24), v(x1, y - 0.24))
        .style(faded_line(chalk(), 2.0, o))
        .build()?)
}

/// A ticket write: a short chalk stroke across the lane.
fn ticket_tick(x: f64, y: f64, o: f64) -> fmn::Result<VMobject> {
    Ok(Line::new(v(x, y - 0.26), v(x, y + 0.26))
        .style(faded_line(chalk(), 4.0, o))
        .build()?)
}

/// The proof picture of 08 (09 overlays it, same layout and scale): the two
/// lanes and what never moves, plus Green's choosing, drawn each frame from
/// two trackers (where it starts, how strongly it shows) so it can slide.
pub struct ProofLanes {
    /// Lanes, badges, Blue's choosing, ticket and room, the look at
    /// Green's flag and its guide line, the glance at her slate.
    pub lanes: Mob,
    pub blue_choosing: Mob,
    pub blue_room: Mob,
    pub look: Mob,
    pub look_label: Mob,
    pub glance: Mob,
    /// Green's choosing, redrawn from `green_start` and `green_alpha`.
    pub green: Mob,
    pub green_start: Mob,
    pub green_alpha: Mob,
    /// Green's state at the look, redrawn from the same trackers; it shows
    /// as strongly as `probe_alpha`.
    pub probe: Mob,
    pub probe_alpha: Mob,
}

impl ProofLanes {
    /// Build the picture (detached, except the two redrawn parts, which are
    /// in the scene but clear until `green_alpha` rises).
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut parts = Vec::new();
        for (k, y) in [(1, Y_BLUE), (2, Y_GREEN)] {
            parts.push(stage.arena_mut().add(lane(X0, X1, y, k)?));
            parts.push(badge(kit, stage, k, 0.2, v(X0 - 0.45, y))?);
            let name = kit.text_left(stage, NAMES[k - 1], 22.0, customer(k), v(X0 - 0.75, y - 0.42))?;
            parts.push(name);
        }
        let arrow = Arrow::new(v(X1 - 1.4, -2.05), v(X1, -2.05))
            .buff(0.0)
            .thickness(2.0)
            .style(
                Style::default()
                    .fill(Palette::faint(), 1.0)
                    .stroke(Palette::faint(), 2.0, 1.0),
            )
            .build()?;
        parts.push(stage.arena_mut().add(arrow));
        parts.push(kit.text(stage, glyphs::TIME, 20.0, Palette::dim(), v(X1 - 0.7, -1.8))?);
        let lanes = group(stage, &parts)?;

        // Blue: choosing, her ticket written and held, then the room
        let (b0, b1) = BLUE_CHOOSING;
        let bc = stage.arena_mut().add(band(b0, b1, Y_BLUE, flag_amber(), 0.85)?);
        let bc_label = kit.text(
            stage,
            glyphs::BLUE_CHOOSING,
            20.0,
            flag_amber(),
            v((b0 + b1) / 2.0, Y_BLUE + 0.4),
        )?;
        let tick = stage.arena_mut().add(ticket_tick(b1 - 0.25, Y_BLUE, 1.0)?);
        let held = stage.arena_mut().add(holding(b1 - 0.25, X1, Y_BLUE, 0.55)?);
        let blue_choosing = group(stage, &[bc, bc_label, tick, held])?;
        let room = stage.arena_mut().add(band(X_ROOM, X1, Y_BLUE, room_red(), 0.75)?);
        let room_label = kit.text(
            stage,
            glyphs::BLUE_IN_ROOM,
            22.0,
            chalk(),
            v((X_ROOM + X1) / 2.0, Y_BLUE),
        )?;
        let blue_room = group(stage, &[room, room_label])?;

        // Blue's last look at Green's flag: an arrow in, from Green's lane,
        // and a faint guide through both lanes at that moment
        let guide = Line::new(v(X_CHECK, Y_BLUE + 0.25), v(X_CHECK, Y_GREEN - 0.3))
            .style(faded_line(order_gold(), 2.0, 0.45))
            .build()?;
        let guide = stage.arena_mut().add(guide);
        let arrow = stage
            .arena_mut()
            .add(access_arrow(v(X_CHECK, Y_GREEN + 0.2), v(X_CHECK, Y_BLUE - 0.2), 1)?);
        let look = group(stage, &[guide, arrow])?;
        let look_label = kit.text(
            stage,
            glyphs::LAST_LOOK,
            20.0,
            customer(1),
            v(X_CHECK - 0.6, Y_BLUE + 0.95),
        )?;
        let glance = stage
            .arena_mut()
            .add(access_arrow(v(X_GLANCE, Y_GREEN + 0.2), v(X_GLANCE, Y_BLUE - 0.2), 1)?);

        // Green's choosing, and her state at the look, from two trackers
        let green_start = stage.add_value_tracker(Place::NotYet.green_start());
        let green_alpha = stage.add_value_tracker(0.0);
        let book = Rc::clone(&kit.book);
        let green = stage.always_redraw(move |s| {
            let x = s.tracker_value(green_start).unwrap_or(0.0);
            let o = s.tracker_value(green_alpha).unwrap_or(0.0).clamp(0.0, 1.0);
            let mut ms = Vec::new();
            if let Ok(b) = band(x, x + GREEN_W, Y_GREEN, flag_amber(), 0.85 * o) {
                ms.push(b);
            }
            if let Ok(t) = ticket_tick(x + GREEN_W - 0.25, Y_GREEN, o) {
                ms.push(t);
            }
            if let Ok(h) = holding(x + GREEN_W - 0.25, X1, Y_GREEN, 0.55 * o) {
                ms.push(h);
            }
            explainer_kit::glyphs::record(explainer_kit::TextKind::Text, glyphs::GREEN_CHOOSING);
            let label = Text::new(glyphs::GREEN_CHOOSING)
                .font_size(20.0)
                .style(Style::default().fill(flag_amber(), o).stroke(flag_amber(), 0.0, o))
                .build(&book)
                .expect("bundled glyphs");
            ms.push(label.vmob.moved_to(v(x + GREEN_W / 2.0, Y_GREEN - 0.42)));
            s.add(v_group(ms))
        });
        let book = Rc::clone(&kit.book);
        let probe_alpha = stage.add_value_tracker(0.0);
        let probe = stage.always_redraw(move |s| {
            let x = s.tracker_value(green_start).unwrap_or(0.0);
            let o = s.tracker_value(probe_alpha).unwrap_or(0.0).clamp(0.0, 1.0);
            let place = Place::of(X_CHECK, x);
            let c = place.color();
            let dot = Dot::new()
                .point(v(X_CHECK, Y_GREEN))
                .radius(0.09)
                .color(c)
                .build()
                .with_style(flat(c, o));
            let source = format!("{} {}", glyphs::GREEN_AT, place.words());
            explainer_kit::glyphs::record(explainer_kit::TextKind::Text, &source);
            let words = Text::new(&source)
                .font_size(20.0)
                .style(Style::default().fill(c, o).stroke(c, 0.0, o))
                .build(&book)
                .expect("bundled glyphs");
            let words = words.vmob.moved_to(v(X_CHECK, Y_GREEN - 0.8));
            s.add(v_group([dot, words]))
        });
        // redrawn every frame, so in the scene from the start (clear until
        // `green_alpha` rises)
        stage.add_to_scene(green)?;
        stage.add_to_scene(probe)?;
        Ok(Self {
            lanes,
            blue_choosing,
            blue_room,
            look,
            look_label,
            glance,
            green,
            green_start,
            green_alpha,
            probe,
            probe_alpha,
        })
    }

    /// Slide Green's choosing to start at `x` over `run_time`.
    pub fn slide(&self, x: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
        tween(self.green_start, x, run_time)
    }

    /// Green's read of Blue's slate while she chooses (case "not yet").
    pub fn green_read_x(&self, place: Place) -> f64 {
        place.green_start() + 0.55 * GREEN_W
    }
}

/// A fact card at the top of the frame: its name in gold and the fact in
/// words, customers in their colours. `left` picks the left or right slot.
pub fn fact_card(kit: &Kit, stage: &mut Stage<'_>, name: &str, fact: &str, left: bool) -> fmn::Result<Mob> {
    let (w, h) = (6.6, 1.25);
    let at = v(if left { -3.55 } else { 3.55 }, 3.05);
    let card = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(explainer_kit::hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at);
    let card = stage.arena_mut().add(card);
    let title = kit.text_left(stage, name, 26.0, order_gold(), v(at[0] - w / 2.0 + 0.25, at[1] + 0.36))?;
    let names = [(NAMES[0], customer(1)), (NAMES[1], customer(2))];
    let body = kit.text_t2c(stage, fact, 25.0, chalk(), &names, v(at[0], at[1] - 0.17))?;
    stage.move_to(body, v(at[0] - w / 2.0 + 0.25, at[1] - 0.17), LEFT);
    group(stage, &[card, title, body])
}

/// A drawn tick or cross (no bundled face has ✓ or ✗), `size` tall.
pub fn verdict_mark(ok: bool, size: f64, at: Vec3) -> VMobject {
    let mark = if ok {
        checkmark(Palette::green())
    } else {
        exmark(room_red())
    };
    let h = mark.extent().map_or(1.0, |(lo, hi)| (hi[1] - lo[1]).max(1e-9));
    mark.scaled_about(size / h, ORIGIN).moved_to(at)
}

// ------------------------------------------------------------- chapter 08

/// The search of 08_9: customers and the largest ticket (the key the facts
/// pin names them, so a wrong pair here fails `facts-check`).
const SEARCH: (usize, u32) = (4, 4);

pub struct WhyNeverTwo {
    pub kit: Kit,
}

impl SceneConstruct for WhyNeverTwo {
    fn name(&self) -> &str {
        "08_why_never_two"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = ProofLanes::build(&kit, stage)?;

        // 08_1: Blue inside, Green holding a ticket; what the proof counts on
        kit.say(stage, "08_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), pic.lanes, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), pic.blue_room, ORIGIN, 1.0)?.rt(1.2),
        );
        let counts_on = lower_third(&kit, stage, glyphs::COUNTS_ON, glyphs::COUNTS_ON_TEXT)?;
        play!(stage;
            tween(pic.green_alpha, 1.0, 0.8)?,
            fade_in(stage.arena_mut(), counts_on, v(0.0, 0.15), 1.0)?.rt(0.8),
        );

        // 08_2: fact one. Blue finished choosing before Green started, so
        // Green's look at Blue's slate saw Blue's ticket
        let line = say(&kit, stage, "08_2")?;
        let one = fact_card(&kit, stage, glyphs::FACT_ONE, glyphs::FACT_ONE_TEXT, true)?;
        play!(stage; fade_in(stage.arena_mut(), one, v(0.0, -0.2), 1.0)?.rt(0.9));
        line.wait_for(stage, "If Blue finished")?;
        play!(stage; fade_in(stage.arena_mut(), pic.blue_choosing, ORIGIN, 1.0)?.rt(1.0));
        let xr = pic.green_read_x(Place::NotYet);
        let read = stage
            .arena_mut()
            .add(access_arrow(v(xr, Y_BLUE - 0.2), v(xr, Y_GREEN + 0.2), 2)?);
        let saw = kit.text_t2c(
            stage,
            glyphs::SEES_TICKET,
            20.0,
            chalk(),
            &[("Green", customer(2)), ("Blue", customer(1))],
            v(xr + 1.55, 0.0),
        )?;
        line.wait_for(stage, "Green saw")?;
        play!(stage; show_creation(read).rt(0.8), fade_in(stage.arena_mut(), saw, ORIGIN, 1.0)?.rt(0.8));
        kit.hold(stage)?;

        // 08_3: fact two, and the question
        let line = say(&kit, stage, "08_3")?;
        play!(stage;
            fade_out(stage.arena_mut(), counts_on, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), read, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), saw, ORIGIN, 1.0)?.rt(0.6),
        );
        let two = fact_card(&kit, stage, glyphs::FACT_TWO, glyphs::FACT_TWO_TEXT, false)?;
        play!(stage; fade_in(stage.arena_mut(), two, v(0.0, -0.2), 1.0)?.rt(0.9));
        let why = kit.text(stage, glyphs::WHY, 26.0, Palette::hero(), v(6.35, 2.6))?;
        line.wait_for(stage, "Why is that")?;
        play!(stage; fade_in(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.5));

        // 08_4: Blue's last look at Green's flag; slide Green's choosing past it
        let line = say(&kit, stage, "08_4")?;
        play!(stage;
            show_creation(pic.look).rt(0.9),
            fade_in(stage.arena_mut(), pic.look_label, ORIGIN, 1.0)?.rt(0.9),
            tween(pic.probe_alpha, 1.0, 0.9)?,
        );
        line.wait_for(stage, "the check that let")?;
        for (x, rt) in [(-3.4, 2.0), (2.2, 2.2)] {
            stage.play(pic.slide(x, rt)?)?;
        }
        let q = predict_pause(&kit, stage, v(5.6, -1.6))?;

        // 08_5: three places. Not yet: Blue checks only after choosing, so
        // she had finished, and fact one applies
        let line = say(&kit, stage, "08_5")?;
        let legend = places_legend(&kit, stage)?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), legend.group, v(0.0, 0.1), 1.0)?.rt(0.8),
        );
        line.wait_for(stage, "Not yet choosing")?;
        stage.play(pic.slide(Place::NotYet.green_start(), 1.0)?)?;
        let lit_edge = Rectangle::new()
            .width(BLUE_CHOOSING.1 - BLUE_CHOOSING.0 + 0.2)
            .height(0.5)
            .corner_radius(0.1)
            .style(line_style(order_gold(), 3.0))
            .build()?
            .moved_to(v((BLUE_CHOOSING.0 + BLUE_CHOOSING.1) / 2.0, Y_BLUE));
        let lit_edge = stage.arena_mut().add(lit_edge);
        // her check comes after her choosing: that's why she'd finished
        // (R3, maya-09)
        line.wait_for(stage, "Blue only checks")?;
        play!(stage; indicate(stage.arena_mut(), pic.look, 1.15, Some(rgb(order_gold())))?.rt(0.8));
        line.wait_for(stage, "after she's chosen")?;
        play!(stage; show_creation(lit_edge).rt(0.8));
        let xr = pic.green_read_x(Place::NotYet);
        let trusted = stage
            .arena_mut()
            .add(gold_arrow(v(xr, Y_BLUE - 0.2), v(xr, Y_GREEN + 0.2))?);
        let bigger = kit.text_t2c(
            stage,
            glyphs::GREEN_BIGGER,
            20.0,
            chalk(),
            &[("Green", customer(2)), ("Blue", customer(1))],
            v(4.6, Y_GREEN - 0.55),
        )?;
        line.wait_for(stage, "by fact one")?;
        play!(stage;
            show_creation(trusted).rt(0.7),
            indicate(stage.arena_mut(), one, 1.06, None)?.rt(1.0),
        );
        line.wait_for(stage, "Green's ticket will")?;
        let ok_not_yet = stage.arena_mut().add(verdict_mark(true, 0.3, legend.mark_at[0]));
        play!(stage;
            fade_in(stage.arena_mut(), bigger, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), ok_not_yet, ORIGIN, 1.4)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 08_6: in the middle of choosing: her flag was up at the look,
        // so that look would have made Blue wait
        let line = say(&kit, stage, "08_6")?;
        play!(stage;
            fade_out(stage.arena_mut(), lit_edge, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), trusted, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), bigger, ORIGIN, 1.0)?.rt(0.4),
        );
        stage.play(pic.slide(Place::Choosing.green_start(), 1.2)?)?;
        let up = kit.text(stage, glyphs::FLAG_UP, 20.0, room_red(), v(X_CHECK - 2.2, 0.0))?;
        let no = stage
            .arena_mut()
            .add(verdict_mark(false, 0.42, v(X_CHECK + 0.4, -0.05)));
        line.wait_for(stage, "Then Green's flag")?;
        play!(stage; fade_in(stage.arena_mut(), up, ORIGIN, 1.0)?.rt(0.6));
        line.wait_for(stage, "wouldn't have")?;
        let no_choosing = stage.arena_mut().add(verdict_mark(false, 0.3, legend.mark_at[1]));
        play!(stage;
            fade_in(stage.arena_mut(), no, ORIGIN, 1.4)?.rt(0.5),
            fade_in(stage.arena_mut(), no_choosing, ORIGIN, 1.4)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 08_7: finished choosing: Blue's later glance read Green's real
        // ticket, and passed only because Blue's pair was smaller
        let line = say(&kit, stage, "08_7")?;
        play!(stage;
            fade_out(stage.arena_mut(), no, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), up, ORIGIN, 1.0)?.rt(0.4),
        );
        stage.play(pic.slide(Place::Done.green_start(), 1.2)?)?;
        line.wait_for(stage, "Then Blue had read")?;
        play!(stage; show_creation(pic.glance).rt(0.5));
        let glance = stage
            .arena_mut()
            .add(gold_arrow(v(X_GLANCE, Y_GREEN + 0.2), v(X_GLANCE, Y_BLUE - 0.2))?);
        let real = kit.text_t2c(
            stage,
            glyphs::REAL_TICKET,
            20.0,
            chalk(),
            &[("Green", customer(2)), ("Blue", customer(1))],
            v(X_GLANCE + 2.05, 0.15),
        )?;
        let smaller = kit.text_t2c(
            stage,
            glyphs::PAIR_SMALLER,
            20.0,
            chalk(),
            &[("Green", customer(2)), ("Blue", customer(1))],
            v(X_GLANCE + 2.05, -0.2),
        )?;
        play!(stage;
            show_creation(glance).rt(0.7),
            fade_in(stage.arena_mut(), real, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "only because")?;
        play!(stage; fade_in(stage.arena_mut(), smaller, ORIGIN, 1.0)?.rt(0.6));
        line.wait_for(stage, "Either way")?;
        let ok_done = stage.arena_mut().add(verdict_mark(true, 0.3, legend.mark_at[2]));
        play!(stage; fade_in(stage.arena_mut(), ok_done, ORIGIN, 1.4)?.rt(0.5));
        // either way, Green's pair is bigger: fact two
        let proved = stage.arena_mut().add(verdict_mark(true, 0.42, v(6.35, 2.6)));
        line.wait_for(stage, "And that's fact two")?;
        play!(stage;
            fade_out(stage.arena_mut(), why, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), proved, ORIGIN, 1.4)?.rt(0.5),
            indicate(stage.arena_mut(), two, 1.06, None)?.rt(0.7),
        );
        kit.hold(stage)?;

        // 08_8: both in the room: fact two, both ways round
        let line = say(&kit, stage, "08_8")?;
        // first the picture: suppose Green were in the room too
        let keep = [one, two, proved, pic.lanes, pic.blue_room, pic.blue_choosing, pic.green];
        stage.clear_updaters(pic.probe, true);
        let mobs: Vec<Mob> = stage.scene().mobjects().to_vec();
        let mut away: Vec<Box<dyn Animation>> = Vec::new();
        for m in mobs.into_iter().filter(|m| !keep.contains(m)) {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
            away.push(stage.prepare(a)?);
        }
        stage.play_prepared(away)?;
        let g_room = stage
            .arena_mut()
            .add(band(X_ROOM + 0.9, X1, Y_GREEN, room_red(), 0.75)?);
        let g_label = kit.text(
            stage,
            glyphs::GREEN_IN_ROOM,
            22.0,
            chalk(),
            v((X_ROOM + 0.9 + X1) / 2.0, Y_GREEN),
        )?;
        let suppose = kit.text(stage, glyphs::SUPPOSE_BOTH, 24.0, room_red(), v(3.95, -0.05))?;
        play!(stage;
            fade_in(stage.arena_mut(), g_room, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), g_label, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), suppose, ORIGIN, 1.0)?.rt(0.7),
        );
        // then fact two, both ways round
        line.wait_for(stage, "Then fact two says")?;
        stage.clear_updaters(pic.green, true);
        let lanes_away = [
            pic.lanes,
            pic.blue_room,
            pic.blue_choosing,
            pic.green,
            g_room,
            g_label,
            suppose,
        ];
        let mut away: Vec<Box<dyn Animation>> = Vec::new();
        for m in lanes_away {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
            away.push(stage.prepare(a)?);
        }
        stage.play_prepared(away)?;
        let colours = [
            ("n_B", customer(1)),
            ("Blue", customer(1)),
            ("n_G", customer(2)),
            ("Green", customer(2)),
        ];
        let (for_blue, for_blue_spans) =
            kit.tex_spans(stage, glyphs::PAIRS_FOR_BLUE, 44.0, chalk(), &colours, v(0.0, 0.9))?;
        let (for_green, _) = kit.tex_spans(stage, glyphs::PAIRS_FOR_GREEN, 44.0, chalk(), &colours, v(0.0, -0.3))?;
        let tag_b = kit.text(stage, glyphs::FOR_BLUE, 20.0, Palette::dim(), v(-4.9, 0.9))?;
        let tag_g = kit.text(stage, glyphs::FOR_GREEN, 20.0, Palette::dim(), v(-4.9, -0.3))?;
        // what n_B and n_G stand for (R3, maya-10)
        let (key, _) = kit.tex_spans(stage, glyphs::PAIRS_KEY, 30.0, Palette::dim(), &colours, v(0.0, 1.75))?;
        play!(stage;
            write(stage.arena(), for_blue).rt(1.0),
            fade_in(stage.arena_mut(), tag_b, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage;
            write(stage.arena(), for_green).rt(1.0),
            fade_in(stage.arena_mut(), tag_g, ORIGIN, 1.0)?.rt(0.6),
        );
        // one chain: a pair smaller than itself
        line.wait_for(stage, "Like me being")?;
        let (chain, chain_spans) = kit.tex_spans(stage, glyphs::PAIRS_CHAIN, 44.0, chalk(), &colours, v(0.0, 0.3))?;
        let morph = TransformMatchingTex::new(stage.arena_mut(), for_blue, chain, &for_blue_spans, &chain_spans)?;
        play!(stage;
            morph.rt(1.4),
            fade_out(stage.arena_mut(), for_green, v(0.0, 0.4), 1.0)?.rt(1.0),
            fade_out(stage.arena_mut(), tag_b, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), tag_g, ORIGIN, 1.0)?.rt(0.6),
        );
        let ends = SurroundingRectangle::from_extent(Some((stage.get_corner(chain, DL), stage.get_corner(chain, UR))))
            .buff(0.15)
            .color(room_red())
            .build();
        let ends = stage.arena_mut().add(ends);
        let impossible = kit.text(stage, glyphs::IMPOSSIBLE_PAIR, 26.0, room_red(), v(0.0, -0.75))?;
        line.wait_for(stage, "Impossible")?;
        // Dijkstra's rule 1 (02_5's pictogram), proved: ticked
        let (at, size) = (v(-2.2, -2.2), 1.2);
        let rule = pictogram(&kit, stage, 1, at, size)?;
        let words = kit.text_left(stage, glyphs::RULE_1, 28.0, chalk(), v(-1.3, -2.2))?;
        let tick = stage.arena_mut().add(rule_tick(at, size));
        play!(stage;
            show_creation(ends).rt(0.6),
            fade_in(stage.arena_mut(), impossible, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), rule, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), words, ORIGIN, 1.0)?.rt(0.6),
        );
        play!(stage; fade_in(stage.arena_mut(), tick, ORIGIN, 1.5)?.rt(0.5));
        kit.hold(stage)?;

        // 08_9: the search agrees, within its bounds; the proof covers all
        let line = say(&kit, stage, "08_9")?;
        let (n, k) = SEARCH;
        // the recorded search the facts pin (C-F8.n4_k4_atomic.*)
        let (states, clean) = recorded_sweep(n, k, false).ok_or(fmn::Error::Scene(SceneError::InvalidConfig(
            "08: no recorded search for its card",
        )))?;
        let verdict = if clean {
            glyphs::NO_VIOLATION
        } else {
            glyphs::A_VIOLATION
        };
        let card = search_card(
            &kit,
            stage,
            &format!(
                "{} {n} {} {k} {} {k} {}",
                glyphs::SEARCH_A,
                glyphs::SEARCH_B,
                glyphs::SEARCH_C,
                glyphs::SEARCH_D
            ),
            &format!(
                "{} {} {} {verdict}",
                glyphs::SEARCH_E,
                thousands(states as u64),
                glyphs::SEARCH_F
            ),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), impossible, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), ends, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), chain, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), card, v(0.0, 0.2), 1.0)?.rt(0.9),
        );
        let every = kit.text_t2c(
            stage,
            glyphs::EVERY_CASE,
            26.0,
            chalk(),
            &[(glyphs::THE_PROOF, order_gold())],
            v(0.0, -0.95),
        )?;
        line.wait_for(stage, "Only the proof")?;
        play!(stage; fade_in(stage.arena_mut(), every, ORIGIN, 1.0)?.rt(0.8));
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// A gold arrow: a glance the proof trusts.
fn gold_arrow(from: Vec3, to: Vec3) -> fmn::Result<VMobject> {
    Ok(Arrow::new(from, to)
        .buff(0.08)
        .thickness(4.5)
        .style(Style::default().fill(order_gold(), 1.0).stroke(order_gold(), 4.5, 1.0))
        .build()?)
}

/// The three places Green can be at the look, in a row under the lanes,
/// each with a spot for its verdict mark.
struct Legend {
    group: Mob,
    mark_at: [Vec3; 3],
}

fn places_legend(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Legend> {
    let xs = [-3.6, -0.1, 3.4];
    let y = -2.35;
    let mut parts = Vec::new();
    let mut mark_at = [v(0.0, 0.0); 3];
    for (k, (place, x)) in Place::ALL.into_iter().zip(xs).enumerate() {
        parts.push(kit.text_left(stage, place.words(), 24.0, place.color(), v(x, y))?);
        mark_at[k] = v(x - 0.35, y);
    }
    Ok(Legend {
        group: group(stage, &parts)?,
        mark_at,
    })
}

/// The search card of 08_9.
fn search_card(kit: &Kit, stage: &mut Stage<'_>, line1: &str, line2: &str) -> fmn::Result<Mob> {
    let card = Rectangle::new()
        .width(12.6)
        .height(1.3)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(explainer_kit::hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(v(0.0, 0.45));
    let card = stage.arena_mut().add(card);
    let a = kit.text(stage, line1, 22.0, chalk(), v(0.0, 0.67))?;
    let b = kit.text(stage, line2, 22.0, chalk(), v(0.0, 0.23))?;
    group(stage, &[card, a, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_places_are_where_the_picture_puts_them() {
        // each case's slide puts Blue's look in that case's place, and the
        // "not yet" case starts after Blue finished choosing (fact one)
        for place in Place::ALL {
            assert_eq!(Place::of(X_CHECK, place.green_start()), place, "{place:?}");
        }
        assert!(Place::NotYet.green_start() > BLUE_CHOOSING.1);
        // Blue: choosing, then the look, then the glance, then the room
        assert!(BLUE_CHOOSING.1 < X_CHECK && X_CHECK < X_GLANCE && X_GLANCE < X_ROOM && X_ROOM < X1);
        // in "done" Green's ticket is written before the look (so before the
        // later glance at her slate too, which then overlaps no write)
        let done_write = Place::Done.green_start() + GREEN_W - 0.25;
        assert!(done_write < X_CHECK);
        // the slides of 08_4 pass through all three places
        let seen: Vec<Place> = [-3.4, 2.2].into_iter().map(|x| Place::of(X_CHECK, x)).collect();
        assert_eq!(seen, [Place::Done, Place::NotYet]);
        assert_eq!(Place::of(X_CHECK, X_CHECK - 0.5 * GREEN_W), Place::Choosing);
    }

    #[test]
    fn the_search_card_names_a_recorded_search() {
        let (n, k) = SEARCH;
        // the card shows the search the facts pin, under the keys that name it
        let (states, clean) = recorded_sweep(n, k, false).expect("a recorded search");
        let f = crate::facts::facts().expect("facts");
        assert_eq!(
            f.get(&format!("C-F8.n{n}_k{k}_atomic.states.recorded")),
            Some(states.to_string().as_str())
        );
        assert_eq!(f.get(&format!("C-F8.n{n}_k{k}_atomic.violation_found")), Some("0"));
        assert!(clean && states == 3_027_633);
        // 08_9 says "up to four customers"
        let script = include_str!("../../script.tsv");
        let line = script.lines().find(|l| l.starts_with("08_9\t")).expect("08_9");
        assert!(line.contains("up to four customers") && n == 4);
    }
}
