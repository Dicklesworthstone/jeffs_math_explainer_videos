//! Chapter 10, the old zero: trace e3 on two lanes, where Green's glance
//! catches Blue's old zero in the middle of Blue's write and Green walks in
//! (10_1); why that's safe, read off the lanes: Green finished choosing
//! (step 14) and saw Blue's flag down (17) before Blue raised it (21), so
//! fact one runs the other way and Blue's ticket is bigger (10_2);
//! Lamport's words, one card at a time (10_3); the double-fetch caution,
//! drawn from the model's own shortest failure (10_4); and the opening
//! question answered (10_5).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    LaneLayout, LaneSpec, Slots, TraceLanes, customer, fade_in_all, fade_out_all, fail, figure, flag_amber, flicker,
    flicker_numerals, line_style, play_timed, quote_card, rgb, room_red, say, slate, static_burst, wait_until,
};
use super::glyphs;
use super::tunes;
use super::why_never_two::fact_card;
use crate::facts::{cited, e3_moments};
use crate::model::{Bakery, Trace, Variant, explore};

const BLUE_Y: f64 = 1.45;
const GREEN_Y: f64 = -0.7;
/// The time axis under the lanes, where step numbers are marked.
const AXIS_Y: f64 = -1.7;

fn e3_layout() -> LaneLayout {
    LaneLayout {
        head_x: -6.45,
        x0: -4.35,
        x1: 6.45,
        lanes: vec![
            LaneSpec {
                customer: 1,
                y: BLUE_Y,
                idle: false,
            },
            LaneSpec {
                customer: 2,
                y: GREEN_Y,
                idle: false,
            },
        ],
        slots: Slots::EveryStep,
        slate_size: 0.5,
    }
}

/// The double-fetch failure's lanes: three customers, every step a slot.
fn double_fetch_layout() -> LaneLayout {
    LaneLayout {
        head_x: -6.45,
        x0: -4.35,
        x1: 6.45,
        lanes: (1..=3)
            .map(|k| LaneSpec {
                customer: k,
                y: 1.25 - 1.25 * (k as f64 - 1.0),
                idle: false,
            })
            .collect(),
        slots: Slots::EveryStep,
        slate_size: 0.42,
    }
}

/// The double-fetch variant on flickering slates, three customers, tickets
/// up to 3 (the checker's `df_trace.py`), and the shortest run it finds
/// that puts two in the room.
pub fn double_fetch_failure() -> Vec<crate::model::Step> {
    let m = Bakery::new(3, 3, Variant::DoubleFetch, true, true);
    explore(&m)
        .violation
        .expect("the double-fetch max lets two in with three customers and flickering slates")
}

/// 10_4's two tunes at once (s), as 12_4's.
const CLASH_S: f64 = 3.0;

/// The two customers (1-based) in the room as the failure ends.
fn in_room_at_end(failure: &[crate::model::Step]) -> fmn::Result<[usize; 2]> {
    let m = Bakery::new(3, 3, Variant::DoubleFetch, true, true);
    let last = failure
        .last()
        .ok_or_else(|| fail("the double-fetch run is empty".to_owned()))?;
    let inside: Vec<usize> = m.in_room(&last.after).into_iter().map(|i| i + 1).collect();
    inside
        .try_into()
        .map_err(|v: Vec<usize>| fail(format!("10_4 needs two in the room, the model has {v:?}")))
}

pub struct OldZero {
    pub kit: Kit,
}

impl SceneConstruct for OldZero {
    fn name(&self) -> &str {
        "10_the_old_zero"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let steps = Trace::E3.steps();
        let at = e3_moments(&steps);
        let mut lanes = TraceLanes::build(&kit, stage, e3_layout(), &steps, &[])?;

        // 10_1: the run so far, fast; then Blue leaves, comes back, and
        // starts writing 9; Green's glance lands inside that write, reads
        // Blue's old zero, and Green walks in
        let line = say(&kit, stage, "10_1")?;
        fade_in_all(stage, &lanes.base(), 0.7)?;
        if let Some(a) = lanes.reveal(stage, 1..=at.glance - 7, 2.2)? {
            let mut all = vec![a];
            all.extend(lanes.heads_to(&kit, stage, at.glance - 7)?);
            play_timed(stage, all, 2.2)?;
        }
        line.wait_for(stage, "a glance")?;
        for s in at.glance - 6..at.glance {
            if let Some(a) = lanes.reveal(stage, s..=s, 0.32)? {
                let mut all = vec![a];
                all.extend(lanes.heads_to(&kit, stage, s)?);
                play_timed(stage, all, 0.32)?;
            }
        }
        line.wait_for(stage, "lets Green straight in")?;
        if let Some(a) = lanes.reveal(stage, at.glance..=at.glance, 0.9)? {
            stage.play_prepared(vec![a])?;
        }
        let q = predict_pause(&kit, stage, v(5.7, 2.6))?;

        // 10_2: read the lanes
        let line = say(&kit, stage, "10_2")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        line.wait_for(stage, "Blue's flag is up")?;
        let blue_band: Vec<Mob> = (at.blue_raises_again..=at.glance)
            .flat_map(|s| lanes.parts[s - 1].bands.clone())
            .filter(|&m| stage.get_center(m)[1] > (BLUE_Y + GREEN_Y) / 2.0)
            .collect();
        let mut hi: Vec<Box<dyn Animation>> = Vec::new();
        for m in blue_band.iter().copied().chain([lanes.heads[0].flag.pennant]) {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.15, Some(rgb(flag_amber())))?;
            hi.push(stage.prepare(a.rt(1.0))?);
        }
        stage.play_prepared(hi)?;
        line.wait_for(stage, "Green had already seen it down")?;
        let saw: Vec<Mob> = lanes.step_mobs(at.green_saw_flag_down);
        let mut hi: Vec<Box<dyn Animation>> = Vec::new();
        for m in saw {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.3, None)?;
            hi.push(stage.prepare(a.rt(1.0))?);
        }
        stage.play_prepared(hi)?;
        // the three moments, in order along the time axis
        line.wait_for(stage, "So Blue raised it")?;
        let axis = stage.arena_mut().add(
            Line::new(v(e3_layout().x0, AXIS_Y), v(e3_layout().x1, AXIS_Y))
                .style(line_style(Palette::faint(), 2.0))
                .build()?,
        );
        play!(stage; show_creation(axis).rt(0.5));
        let mut guides = vec![axis];
        for (s, who) in [
            (at.green_done_choosing, 2),
            (at.green_saw_flag_down, 2),
            (at.blue_raises_again, 1),
        ] {
            let x = lanes
                .x(s)
                .ok_or_else(|| super::draw::fail(format!("step {s} has a slot")))?;
            let guide = DashedLine::new(v(x, BLUE_Y + 0.45), v(x, AXIS_Y))
                .dash_length(0.08)
                .style(line_style(customer(who), 2.0))
                .build()?;
            let g = stage.arena_mut().add(guide);
            let label = kit.text(
                stage,
                &format!("{} {s}", glyphs::STEP),
                22.0,
                customer(who),
                v(x, AXIS_Y - 0.32),
            )?;
            play!(stage;
                show_creation(g).rt(0.45),
                fade_in(stage.arena_mut(), label, v(0.0, 0.1), 1.0)?.rt(0.45),
            );
            guides.extend([g, label]);
        }
        line.wait_for(stage, "That's fact one")?;
        let fact = fact_card(
            &kit,
            stage,
            glyphs::FACT_ONE_SWAPPED,
            glyphs::FACT_ONE_SWAPPED_TEXT,
            true,
        )?;
        play!(stage; fade_in(stage.arena_mut(), fact, v(0.0, -0.15), 1.0)?.rt(0.7));
        // Blue's write lands: 9; her flag comes down; she checks Green and
        // waits behind Green's smaller pair
        line.wait_for(stage, "so Blue's ticket will be bigger")?;
        for s in at.glance + 1..=steps.len() {
            if let Some(a) = lanes.reveal(stage, s..=s, 0.4)? {
                let mut all = vec![a];
                all.extend(lanes.heads_to(&kit, stage, s)?);
                play_timed(stage, all, 0.4)?;
            }
        }
        let end = &steps[steps.len() - 1].after;
        let pairs = kit.tex(
            stage,
            &format!(r"({},\ 2) < ({},\ 1)", end.number[1], end.number[0]),
            40.0,
            Palette::ink(),
            &[],
            v(4.6, -2.6),
        )?;
        play!(stage; fade_in(stage.arena_mut(), pairs, v(0.0, 0.15), 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 10_3: Lamport's words, one card at a time
        let mut old = lanes.base();
        old.extend(lanes.parts.iter().flat_map(|p| p.all()));
        old.extend(guides);
        old.extend([fact, pairs]);
        let cards = [
            (glyphs::QUOTE_DISCOVERED, glyphs::PUBLICATIONS_SOURCE.to_owned()),
            (
                glyphs::QUOTE_ARBITRARY,
                format!(
                    "{}, {}, {}, {} {}",
                    glyphs::LAMPORT_SURNAME,
                    cited::bakery_journal(),
                    cited::BAKERY_YEAR,
                    glyphs::PAGE,
                    cited::ARBITRARY_VALUE_PAGE
                ),
            ),
            (glyphs::QUOTE_REFUSED, glyphs::HOLT_SOURCE.to_owned()),
        ];
        let line = say(&kit, stage, "10_3")?;
        fade_out_all(stage, &old, 0.7)?;
        let chips = lower_third(&kit, stage, glyphs::PRECISELY, glyphs::CHIPS_DONT_TEAR)?;
        let mut shown: Option<Mob> = None;
        for (i, (words, source)) in cards.iter().enumerate() {
            if i == 1 {
                line.wait_for(stage, "In his words")?;
            } else if i == 2 {
                line.wait_for(stage, "His colleague")?;
            }
            let card = quote_card(&kit, stage, words, source, v(0.0, 0.5), 46)?;
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            if let Some(s) = shown.take() {
                let a = fade_out(stage.arena_mut(), s, v(0.0, 0.3), 1.0)?;
                anims.push(stage.prepare(a.rt(0.5))?);
            }
            let a = fade_in(stage.arena_mut(), card, v(0.0, 0.3), 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
            if i == 0 {
                let c = fade_in(stage.arena_mut(), chips, v(0.0, 0.2), 1.0)?;
                anims.push(stage.prepare(c.rt(0.7))?);
            }
            stage.play_prepared(anims)?;
            shown = Some(card);
        }
        kit.hold(stage)?;

        // 10_4: the max that reads a slate twice, then only the outcome: the
        // model's shortest failure, two in the room at once
        let code = [
            glyphs::MAX_CODE_1,
            glyphs::MAX_CODE_2,
            glyphs::MAX_CODE_3,
            glyphs::MAX_CODE_4,
        ];
        let line = say(&kit, stage, "10_4")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in shown.into_iter().chain([chips]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        stage.play_prepared(anims)?;
        let mut code_mobs = Vec::new();
        for (i, l) in code.iter().enumerate() {
            let at_ = v(-2.9, 1.0 - 0.62 * i as f64);
            let m = kit.mono(stage, l, 40.0, Palette::ink(), at_)?;
            stage.move_to(m, at_, LEFT);
            code_mobs.push(m);
        }
        line.wait_for(stage, "Find the biggest")?;
        fade_in_all(stage, &code_mobs, 0.6)?;
        line.wait_for(stage, "The obvious way")?;
        // the two reads of number[k]: compare, then copy
        let reads = [(2usize, glyphs::COMPARE), (3usize, glyphs::COPY)];
        let mut tags = Vec::new();
        for (row, word) in reads {
            let m = code_mobs[row];
            let r = stage.get_right(m);
            let tag = kit.text(stage, word, 26.0, flag_amber(), v(r[0] + 0.25, r[1]))?;
            stage.move_to(tag, v(r[0] + 0.25, r[1]), LEFT);
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.05, Some(rgb(flag_amber())))?;
            play!(stage; a.rt(0.7), fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5));
            tags.push(tag);
        }
        line.wait_for(stage, "a flicker between them")?;
        let between = {
            let a = stage.get_left(code_mobs[2]);
            let b = stage.get_left(code_mobs[3]);
            v(a[0] - 0.55, (a[1] + b[1]) / 2.0)
        };
        let fx = stage.arena_mut().add(static_burst(between, 0.7, 41));
        play!(stage; fade_in(stage.arena_mut(), fx, ORIGIN, 1.2)?.rt(0.3));
        // the outcome, on three lanes, fast
        let failure = double_fetch_failure();
        let mut df = TraceLanes::build(&kit, stage, double_fetch_layout(), &failure, &[])?;
        let mut away = code_mobs.clone();
        away.extend(tags);
        away.push(fx);
        fade_out_all(stage, &away, 0.5)?;
        fade_in_all(stage, &df.base(), 0.5)?;
        // the run in four stretches, the slates and flags at the lane heads
        // following it, so the heads end on the model's final state
        let n = failure.len();
        let cuts = [n / 4, n / 2, 3 * n / 4, n];
        let mut from = 1;
        for &to in &cuts {
            let mut all: Vec<Box<dyn Animation>> = df.reveal(stage, from..=to, 0.75)?.into_iter().collect();
            all.extend(df.heads_to(&kit, stage, to)?);
            play_timed(stage, all, 0.75)?;
            from = to + 1;
        }
        let model_note = kit.text(
            stage,
            &format!(
                "{} {} {} {} {}",
                glyphs::MODEL_NOTE_A,
                df.layout.lanes.len(),
                glyphs::MODEL_NOTE_B,
                failure.len(),
                glyphs::MODEL_STEPS
            ),
            24.0,
            Palette::dim(),
            v(0.0, -2.55),
        )?;
        let cite = kit.text(
            stage,
            &format!("{} {}", glyphs::NONATOMIC_SOURCE, cited::NONATOMIC_OPERATIONS_YEAR),
            24.0,
            Palette::dim(),
            v(0.0, 2.6),
        )?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in [model_note, cite] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        // two in the room at once: the two inside play their tunes together
        // in their own window, as their room bands pulse
        let [a, b] = in_room_at_end(&failure)?;
        let (wav, secs) = tunes::write("10_4", tunes::clash(a, b, CLASH_S)).map_err(|e| fail(e.to_string()))?;
        kit.sonify(stage, secs, "10_the_old_zero:10_4", &wav)?;
        let t0 = stage.scene().time().to_f64();
        let both: Vec<Mob> = df.parts[failure.len() - 1].bands.clone();
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in both {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.2, Some(rgb(room_red())))?;
            anims.push(stage.prepare(a.rt(0.9))?);
        }
        stage.play_prepared(anims)?;
        wait_until(stage, t0 + secs)?;
        kit.hold(stage)?;

        // 10_5: the opening question, answered
        let line = say(&kit, stage, "10_5")?;
        fade_all(stage, 0.6)?;
        let board = slate(&kit, stage, cited::TORN_FROM, v(-2.4, 0.4), 1.5)?;
        let blue = figure(&kit, stage, 1, v(1.4, 0.3), 1.6)?;
        let ask = kit.text(stage, glyphs::COULD_COMPUTERS, 36.0, Palette::ink(), v(0.0, 2.6))?;
        play!(stage;
            fade_in(stage.arena_mut(), board.board, ORIGIN, 0.9)?.rt(0.6),
            fade_in(stage.arena_mut(), blue.group, v(0.0, -0.1), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ask, v(0.0, -0.15), 1.0)?.rt(0.6),
        );
        let digits = flicker_numerals(
            &kit,
            stage,
            &[cited::TORN_FROM, cited::TORN_VALUE, cited::TORN_TO],
            v(-2.4, 0.4),
            1.5,
        )?;
        // while the question is asked, a glance at the slate flickers through
        // garbage; as the answer comes, the write lands and she still waits
        let until_yes = line.at("Yes")? - stage.scene().time().to_f64();
        flicker(stage, &digits, until_yes.max(1.0), 9.0, 2, u64::from(cited::TORN_VALUE))?;
        let yes = kit.text_styled(stage, glyphs::YES, 64.0, Palette::green(), v(0.0, -2.2), true, false)?;
        play!(stage; fade_in(stage.arena_mut(), yes, v(0.0, 0.2), 1.1)?.rt(0.5));
        kit.music_breath(stage, HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_double_fetch_failure_ends_with_two_in_the_room() {
        // C-F6.double_fetch.n3_k3_safe.shortest_violation_steps = 33
        let f = double_fetch_failure();
        assert_eq!(f.len(), 33);
        let m = Bakery::new(3, 3, Variant::DoubleFetch, true, true);
        let last = &f.last().unwrap().after;
        assert_eq!(m.in_room(last).len(), 2);
        // the clash plays those two
        let [a, b] = in_room_at_end(&f).unwrap();
        assert_eq!(m.in_room(last), vec![a - 1, b - 1]);
        // no step before the last has two inside
        assert!(f[..f.len() - 1].iter().all(|s| m.in_room(&s.after).len() <= 1));
    }

    #[test]
    fn the_e3_moments_come_in_the_order_the_voice_gives() {
        let at = e3_moments(&Trace::E3.steps());
        assert!(at.green_done_choosing < at.green_saw_flag_down);
        assert!(at.green_saw_flag_down < at.blue_raises_again);
        assert!(at.blue_raises_again < at.glance);
    }
}
