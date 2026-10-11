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
    LaneLayout, LaneSpec, Slots, TraceLanes, chalk, customer, fade_in_all, fade_out_all, figure, flag_amber, flicker,
    flicker_numerals, line_style, play_timed, quote_card, rgb, room_red, say, slate, static_burst,
};
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
            let label = kit.text(stage, &format!("step {s}"), 22.0, customer(who), v(x, AXIS_Y - 0.32))?;
            play!(stage;
                show_creation(g).rt(0.45),
                fade_in(stage.arena_mut(), label, v(0.0, 0.1), 1.0)?.rt(0.45),
            );
            guides.extend([g, label]);
        }
        line.wait_for(stage, "That's fact one")?;
        let fact = fact_one_swapped(&kit, stage, v(0.0, 2.75))?;
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
            (
                "I discovered that the bakery algorithm had this property after writing a proof of its correctness",
                "Leslie Lamport, publications page".to_owned(),
            ),
            (
                "The read may return any arbitrary value!",
                format!(
                    "Lamport, {}, {}, p. {}",
                    cited::bakery_journal(),
                    cited::BAKERY_YEAR,
                    cited::ARBITRARY_VALUE_PAGE
                ),
            ),
            (
                "He refused to believe it could be true.",
                "Lamport on Anatol Holt, publications page".to_owned(),
            ),
        ];
        let line = say(&kit, stage, "10_3")?;
        fade_out_all(stage, &old, 0.7)?;
        let chips = lower_third(
            &kit,
            stage,
            "precisely:",
            "today's chips don't tear an aligned word; the proof just doesn't need that",
        )?;
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
            "t = 0",
            "for k in others:",
            "    if number[k] > t:",
            "        t = number[k]",
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
        let reads = [(2usize, "compare"), (3usize, "copy")];
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
        let df = TraceLanes::build(&kit, stage, double_fetch_layout(), &failure, &[])?;
        let mut away = code_mobs.clone();
        away.extend(tags);
        away.push(fx);
        fade_out_all(stage, &away, 0.5)?;
        fade_in_all(stage, &df.base(), 0.5)?;
        if let Some(a) = df.reveal(stage, 1..=failure.len(), 3.0)? {
            stage.play_prepared(vec![a])?;
        }
        let model_note = kit.text(
            stage,
            &format!(
                "model: {} customers, flickering slates, read twice: two in the room after {} steps",
                df.layout.lanes.len(),
                failure.len()
            ),
            24.0,
            Palette::dim(),
            v(0.0, -2.55),
        )?;
        let cite = kit.text(
            stage,
            &format!(
                "Lamport, Reasoning About Nonatomic Operations, POPL {}",
                cited::NONATOMIC_OPERATIONS_YEAR
            ),
            24.0,
            Palette::dim(),
            v(0.0, 2.6),
        )?;
        let both: Vec<Mob> = df.parts[failure.len() - 1].bands.clone();
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in both {
            let a = fmn::animation::indicate(stage.arena_mut(), m, 1.2, Some(rgb(room_red())))?;
            anims.push(stage.prepare(a.rt(0.9))?);
        }
        for m in [model_note, cite] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        // two in the room at once: the clash gets its own listen window
        kit.listen(stage, 3.0, "10_the_old_zero:10_4")?;
        kit.hold(stage)?;

        // 10_5: the opening question, answered
        let line = say(&kit, stage, "10_5")?;
        fade_all(stage, 0.6)?;
        let board = slate(&kit, stage, cited::TORN_FROM, v(-2.4, 0.4), 1.5)?;
        let blue = figure(&kit, stage, 1, v(1.4, 0.3), 1.6)?;
        let ask = kit.text(
            stage,
            "Could computers take turns safely with memory like that?",
            36.0,
            Palette::ink(),
            v(0.0, 2.6),
        )?;
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
        let yes = kit.text_styled(stage, "Yes.", 64.0, Palette::green(), v(0.0, -2.2), true, false)?;
        play!(stage; fade_in(stage.arena_mut(), yes, v(0.0, 0.2), 1.1)?.rt(0.5));
        kit.music_breath(stage, HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// Fact one with Blue and Green swapped (08_2's card, M6): Green finished
/// choosing before Blue started, so Blue's ticket is bigger.
fn fact_one_swapped(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let parts = [
        ("Fact one, the other way round:", Palette::dim()),
        ("Green", customer(2)),
        ("finished choosing before", chalk()),
        ("Blue", customer(1)),
        ("started, so", chalk()),
        ("Blue's", customer(1)),
        ("ticket is bigger", chalk()),
    ];
    let mut mobs = Vec::new();
    let mut x = 0.0;
    for (s, c) in parts {
        let m = kit.text(stage, s, 28.0, c, ORIGIN)?;
        let w = stage.get_width(m);
        stage.move_to(m, v(x + w / 2.0, at[1]), ORIGIN);
        x += w + 0.16;
        mobs.push(m);
    }
    let shift = at[0] - (x - 0.16) / 2.0;
    for &m in &mobs {
        stage.shift(m, v(shift, 0.0));
    }
    super::draw::group(stage, &mobs)
}
