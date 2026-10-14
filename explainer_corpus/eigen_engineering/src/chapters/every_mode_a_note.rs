//! Chapter 07, every mode a note: give every joint its mass and K's own
//! patterns shift, a little at the low end, into modes, each with a
//! frequency of its own (07_1); the lowest mode swinging at 5.9 Hz, drawn
//! slowed six times, with a clock and a count (07_2); modes 2 and 3 beside
//! it at their true relative rates, and the three as tones (07_3); the
//! quiz, twice the deck's mass, faster, slower or the same (07_4); 4.32 Hz
//! against 5.91, and not 4.18 (07_5); and a messy shake of the truss
//! resolving into its modes (07_6).
//!
//! The swinging is real: each picture is the truss moved by the model's
//! modes, `Σ φₖ sin(2π fₖ t)`, on one clock that runs a sixth as fast as the
//! screen's. Every number is a fact (`K-V1.*`, `K-V2.*`, `K-Q.07_4.answer`).

use explainer_kit::props::{names_to_search, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    QuizCard, chip, cross_fade, fact, fade_in_all, fade_out_all, faded_line, flat, items, own, pause, say, wait_until,
};
use super::marks::mark;
use super::sound;
use super::the_bridge_that_swayed::counted;
use super::truss::{Layout, outline, supports};
use super::twenty_one_patterns::{corners, deformed, sag_oriented, scaled_to};
use crate::model::linalg::Eigen;
use crate::model::truss::{DECK_KG, Truss};

pub struct EveryModeANote {
    pub kit: Kit,
}

/// How many times slower the picture runs than the bridge does.
pub const SLOW: f64 = 6.0;
/// The four tonnes of 07_2 and the eight of the quiz, kilograms.
pub const HEAVY_KG: f64 = 2.0 * DECK_KG;
/// 07_6: the three clean modes' trusses (their feet, scale, and the largest
/// bend in metres) and the heights of the two captions around them. The mix
/// caption stood at -0.9, 1-2 px over the middle truss's apex; the plane
/// caption was 13 px off the lower-third.
pub const CLEAN_Y: f64 = -1.9;
pub const CLEAN_SCALE: f64 = 0.16;
pub const CLEAN_BEND_M: f64 = 1.3;
pub const MIX_Y: f64 = -0.6;
pub const PLANE_Y: f64 = -2.55;
/// Where the "slow motion × 6" tag stands (at 3.7 it was 0.19 from the
/// frame's top edge, under the 0.25 a text keeps).
const SLOW_TAG_AT: Vec3 = [-5.5, 3.55, 0.0];

// ------------------------------------------------------------ the model

/// The modes of the truss with `deck_kg` at each interior deck joint: the
/// mode shapes as moves of the free directions, and their frequencies in
/// hertz, lowest first.
pub fn modes_of(t: &Truss, deck_kg: f64) -> (Eigen, Vec<f64>) {
    (t.modes(deck_kg), t.hertz(deck_kg))
}

/// Mode `k` as a sag-oriented field of largest move `amp` metres.
pub fn shaped(t: &Truss, e: &Eigen, k: usize, amp: f64) -> Vec<f64> {
    scaled_to(&sag_oriented(t, &e.vectors[k]), amp)
}

/// The moves at clock `clock` seconds of the sum of `parts`, each a field
/// swinging at its own hertz: `Σ field · sin(2π f t)`.
pub fn swung(parts: &[(Vec<f64>, f64)], clock: f64) -> Vec<f64> {
    let n = parts.first().map_or(0, |p| p.0.len());
    let mut out = vec![0.0; n];
    for (field, hz) in parts {
        let s = (std::f64::consts::TAU * hz * clock).sin();
        for (o, x) in out.iter_mut().zip(field) {
            *o += s * x;
        }
    }
    out
}

/// A disc's radius for a joint of `kg`: the root of its mass in tonnes.
pub fn disc_radius(kg: f64) -> f64 {
    0.05 * (kg / 1000.0).sqrt()
}

// ------------------------------------------------------------- the picture

/// A truss that swings: one polyline per member, rewritten from the clock
/// tracker whenever it changes.
struct Swinger {
    members: Vec<Mob>,
    anchor: Mob,
}

/// A swinging truss on `layout`, moved by `parts` (field in metres, hertz),
/// drawn in `colour` at stroke `width`, reading `clock` (seconds of the
/// bridge's time). Detached; add `anchor` to the scene to start it.
fn swinger(
    stage: &mut Stage<'_>,
    t: &Truss,
    layout: Layout,
    parts: Vec<(Vec<f64>, f64)>,
    clock: Mob,
    colour: Srgb,
    width: f64,
) -> fmn::Result<Swinger> {
    let rest = deformed(t, layout, &vec![0.0; t.free().len()], 1.0);
    let members: Vec<Mob> = rest
        .iter()
        .map(|&(a, b)| {
            stage
                .arena_mut()
                .add(polyline(vec![a, b]).style(faded_line(colour, width, 1.0)).build())
        })
        .collect();
    let anchor = stage.arena_mut().add(VMobject::new());
    {
        let (members, tt) = (members.clone(), t.clone());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let c = s.tracker_value(clock).unwrap_or(0.0);
                if c.to_bits() == last.to_bits() {
                    return;
                }
                last = c;
                let segs = deformed(&tt, layout, &swung(&parts, c), 1.0);
                for (&m, (a, b)) in members.iter().zip(segs) {
                    let _ = s.set_points(m, &corners(&[a, b]));
                }
            },
            true,
        )?;
    }
    Ok(Swinger { members, anchor })
}

/// Light row `on` of the three modes' rows and put the others back to a third
/// (all lit for `None`): the picture of which tone is sounding.
fn light_row(stage: &mut Stage<'_>, rows: &[Vec<Mob>], on: Option<usize>) {
    for (k, parts) in rows.iter().enumerate() {
        let opacity = if on.is_none_or(|i| i == k) { 1.0 } else { 0.35 };
        for &m in parts {
            stage.set_opacity(m, opacity, true);
        }
    }
}

/// The joints' mass as discs: dim, radius by the joint's mass, detached.
fn mass_discs(stage: &mut Stage<'_>, t: &Truss, layout: Layout, deck_kg: f64) -> Vec<Mob> {
    t.joint_kg(deck_kg)
        .iter()
        .enumerate()
        .map(|(j, &kg)| {
            stage.arena_mut().add(
                Dot::new()
                    .point(layout.joint(t, j))
                    .radius(disc_radius(kg))
                    .style(flat(Palette::dim(), 0.85))
                    .build(),
            )
        })
        .collect()
}

/// The hero truss of 07_1 and 07_2.
const HERO: Layout = Layout {
    origin: [-5.4, -0.5, 0.0],
    scale: 0.45,
};

impl SceneConstruct for EveryModeANote {
    fn name(&self) -> &str {
        "07_every_mode_a_note"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();
        let t = Truss::pratt();
        let (light, hz) = modes_of(&t, DECK_KG);
        let (heavy, heavy_hz) = modes_of(&t, HEAVY_KG);
        let patterns = t.patterns();

        // the one clock: seconds of the bridge's own time, a sixth as fast as the screen's
        let clock = stage.add_value_tracker(0.0);
        let running = stage.add_value_tracker(0.0);
        let ticker = stage.arena_mut().add(VMobject::new());
        stage.add_dt_updater(
            ticker,
            move |s, _, dt| {
                if dt > 0.0 && s.tracker_value(running).unwrap_or(0.0) > 0.5 {
                    let c = s.tracker_value(clock).unwrap_or(0.0);
                    let _ = s.set_tracker_value(clock, c + dt / SLOW);
                }
            },
            true,
        )?;
        stage.add_to_scene(ticker)?;

        // ---- 07_1: give every joint its mass: modes
        let line = say(&kit, stage, "07_1")?;
        let frame = outline(stage, &t, HERO, Palette::faint(), 1.0);
        let feet = supports(stage, &t, HERO);
        let discs = mass_discs(stage, &t, HERO, DECK_KG);
        // K's own pattern 1, ghosted under the first mode
        let ghost_field = shaped(&t, &patterns, 0, 1.4);
        let ghost: Vec<Mob> = deformed(&t, HERO, &ghost_field, 1.0)
            .iter()
            .map(|&(a, b)| {
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(faded_line(own(), 5.0, 0.28)).build())
            })
            .collect();
        let first = swinger(
            stage,
            &t,
            HERO,
            vec![(shaped(&t, &light, 0, 1.4), hz[0])],
            clock,
            own(),
            5.0,
        )?;
        let equation = kit.tex(stage, r"K\varphi = \omega^2 M\varphi", 56.0, ink, &[], v(0.0, 3.1))?;
        let right_angles = kit.tex(stage, r"\varphi_i^T M \varphi_j = 0", 44.0, dim, &[], v(0.0, 2.2))?;
        let overlap = |i: usize| fact(&format!("K-V1.overlap.{i}"));
        let exact = precisely(
            &kit,
            stage,
            &format!(
                "the eigenvalue is \u{3c9}\u{b2}, hertz = \u{3c9} \u{f7} 2\u{3c0}; modes are at right angles once mass is counted; the lowest few modes are close to K's own patterns (overlap {}, {}, {} for the first three, {} by the sixth, {} by the seventh), the higher ones are not",
                overlap(1)?,
                overlap(2)?,
                overlap(3)?,
                overlap(6)?,
                overlap(7)?
            ),
        )?;
        let mut ins: Vec<Mob> = frame.clone();
        ins.extend(&feet);
        ins.extend(&discs);
        fade_in_all(stage, &ins, 0.7)?;
        stage.set_tracker_value(running, 1.0)?;
        stage.add_to_scene(first.anchor)?;
        let mut ins: Vec<Mob> = ghost.clone();
        ins.extend(&first.members);
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "Now give every joint its mass")?;
        fade_in_all(stage, &[equation], 0.6)?;
        line.wait_for(stage, "At the low end")?;
        fade_in_all(stage, &[right_angles, exact], 0.6)?;
        kit.hold(stage)?;

        // ---- 07_2: four tonnes at each deck joint: 5.9 hertz
        let line = say(&kit, stage, "07_2")?;
        fade_out_all(stage, &[equation, right_angles, exact], 0.4)?;
        fade_out_all(stage, &ghost, 0.4)?;
        let slow_tag = kit.text(
            stage,
            &format!("slow motion \u{d7} {}", SLOW as u32),
            30.0,
            dim,
            SLOW_TAG_AT,
        )?;
        // both counters open at 0 when they come in (they counted from 07_1's
        // first word and arrived at "13 swings, 2.3 seconds")
        let zero = stage.tracker_value(clock).unwrap_or(0.0);
        let swings = kit.live_value(
            stage,
            {
                let f1 = hz[0];
                move |s| counted(s.tracker_value(clock).unwrap_or(0.0), zero, f1)
            },
            0,
            false,
            60.0,
            own(),
            v(4.2, 2.9),
            RIGHT,
        )?;
        let swings_words = kit.text(stage, "swings", 28.0, dim, v(5.3, 2.9))?;
        let seconds = kit.live_value(
            stage,
            move |s| (s.tracker_value(clock).unwrap_or(0.0) - zero).max(0.0),
            1,
            false,
            44.0,
            ink,
            v(4.2, 2.0),
            RIGHT,
        )?;
        let seconds_words = kit.text(stage, "seconds", 28.0, dim, v(5.4, 2.0))?;
        let steel = kit.text(
            stage,
            &format!(
                "the two end joints carry {} t each",
                crate::chapters::draw::whole(DECK_KG / 2000.0)
            ),
            26.0,
            dim,
            v(0.0, -2.2),
        )?;
        let plus = kit.text(
            stage,
            &format!(
                "plus the steel's own {} tonnes, spread over the joints",
                fact("K-V1.steel_tonnes")?
            ),
            26.0,
            dim,
            v(0.0, -2.8),
        )?;
        let big_hz = kit.text(stage, &format!("{} Hz", fact("K-V1.hz.1")?), 72.0, own(), v(0.0, -1.3))?;
        let deck_tag = kit.text(
            stage,
            &format!("{} tonnes at each of the five deck joints", fact("K-V1.deck_tonnes")?),
            28.0,
            ink,
            v(0.0, 3.0),
        )?;
        fade_in_all(
            stage,
            &[slow_tag, swings_words, seconds_words, deck_tag, steel, plus],
            0.6,
        )?;
        line.wait_for(stage, "Five point nine hertz")?;
        fade_in_all(stage, &[big_hz], 0.5)?;
        kit.hold(stage)?;

        // ---- 07_3: the next two, beside it at their true relative rates; then as tones
        let line = say(&kit, stage, "07_3")?;
        fade_out_all(
            stage,
            &[
                big_hz,
                deck_tag,
                steel,
                plus,
                swings,
                swings_words,
                seconds,
                seconds_words,
            ],
            0.4,
        )?;
        // the hero steps aside: three rows
        fade_out_all(stage, &frame, 0.3)?;
        fade_out_all(stage, &feet, 0.3)?;
        fade_out_all(stage, &discs, 0.3)?;
        fade_out_all(stage, &first.members, 0.3)?;
        stage.remove_from_scene(first.anchor);
        let rows = [2.0, 0.1, -1.8];
        let mut trio = Vec::new();
        let mut row_extras: Vec<Mob> = Vec::new();
        for (k, &y) in rows.iter().enumerate() {
            let layout = Layout {
                origin: [-6.4, y - 0.2, 0.0],
                scale: 0.3,
            };
            row_extras.extend(outline(stage, &t, layout, Palette::faint(), 1.0));
            row_extras.extend(supports(stage, &t, layout));
            trio.push(swinger(
                stage,
                &t,
                layout,
                vec![(shaped(&t, &light, k, 1.2), hz[k])],
                clock,
                own(),
                4.0,
            )?);
        }
        let hz_words: Vec<String> = (1..=3)
            .map(|k| Ok(format!("{} Hz", fact(&format!("K-V1.hz.{k}"))?)))
            .collect::<fmn::Result<_>>()?;
        let notes = [
            "the lowest: the whole span sags",
            "mostly slides the deck along its length",
            "mostly bends it into an S",
        ];
        let mut labels = Vec::new();
        for (k, &y) in rows.iter().enumerate() {
            labels.push(kit.text_left(stage, &hz_words[k], 44.0, own(), v(2.3, y + 0.35))?);
            labels.push(kit.text_left(stage, notes[k], 24.0, dim, v(2.3, y - 0.2))?);
        }
        // each row's own parts (its swinging members and its two labels), so
        // that the row whose tone is sounding can be the one that is lit
        let row_parts: Vec<Vec<Mob>> = (0..3)
            .map(|k| {
                let mut p = trio[k].members.clone();
                p.extend(&labels[2 * k..2 * k + 2]);
                p
            })
            .collect();
        let mut ins = row_extras.clone();
        for s in &trio {
            ins.extend(&s.members);
        }
        ins.extend(&labels);
        for s in &trio {
            stage.add_to_scene(s.anchor)?;
        }
        fade_in_all(stage, &ins, 0.7)?;
        let ratio = format!("1 : {} : {}", fact("K-V1.ratio.2")?, fact("K-V1.ratio.3")?);
        let tones = kit.text(
            stage,
            &format!("as tones, one at a time and then together: {ratio}"),
            28.0,
            ink,
            v(0.0, -3.2),
        )?;
        let faster = kit.text(stage, "sound: the same ratios, faster", 24.0, dim, v(0.0, -3.7))?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[tones, faster], 0.5)?;
        // the three modes as three struck tones in their true ratios, one at
        // a time and then together, in a listen window of the same 3 s; the
        // three swing on through it
        let wav = sound::bridge(&Truss::pratt().hertz(DECK_KG))?;
        kit.sonify(stage, sound::BRIDGE_S, &format!("{}:07_3", kit.chapter), &wav)?;
        let start = stage.scene().time().to_f64();
        mark(&kit, stage, "07_3.window_start");
        // the row whose tone sounds is lit and the others stand back, mode
        // by mode as the tones come, then all three for the chord
        for k in 0..3 {
            wait_until(stage, start + sound::BRIDGE_STEP_S * k as f64)?;
            light_row(stage, &row_parts, Some(k));
            mark(&kit, stage, ["07_3.lit_0", "07_3.lit_1", "07_3.lit_2"][k]);
        }
        wait_until(stage, start + sound::BRIDGE_TOGETHER_AT)?;
        light_row(stage, &row_parts, None);
        mark(&kit, stage, "07_3.lit_all");
        wait_until(stage, start + sound::BRIDGE_S)?;
        kit.hold(stage)?;

        // ---- 07_4: your turn: twice as heavy; faster, slower or the same?
        let line = say(&kit, stage, "07_4")?;
        fade_out_all(stage, &[slow_tag], 0.3)?;
        let mut away = row_extras.clone();
        for s in &trio {
            away.extend(&s.members);
        }
        away.extend(&labels);
        away.extend([tones, faster]);
        fade_out_all(stage, &away, 0.5)?;
        for s in &trio {
            stage.remove_from_scene(s.anchor);
        }
        stage.set_tracker_value(running, 0.0)?;
        let card = QuizCard::build(&kit, stage, "07_4", v(0.0, -0.1), (11.0, 5.6))?;
        let (centre, _, _) = card.content();
        let small = Layout {
            origin: [centre[0] - 4.9, centre[1] - 0.3, 0.0],
            scale: 0.17,
        };
        let big = Layout {
            origin: [centre[0] + 1.0, centre[1] - 0.3, 0.0],
            scale: 0.17,
        };
        let mut shown: Vec<Mob> = Vec::new();
        shown.extend(outline(stage, &t, small, ink, 1.0));
        shown.extend(outline(stage, &t, big, ink, 1.0));
        shown.extend(mass_discs(stage, &t, small, DECK_KG));
        shown.extend(mass_discs(stage, &t, big, HEAVY_KG));
        shown.push(kit.text(
            stage,
            "4 t at each deck joint",
            26.0,
            dim,
            v(small.origin[0] + 2.0, centre[1] + 0.9),
        )?);
        shown.push(kit.text(
            stage,
            "8 t at each deck joint",
            26.0,
            ink,
            v(big.origin[0] + 2.0, centre[1] + 0.9),
        )?);
        shown.push(kit.text(
            stage,
            "twice as heavy: does the lowest mode get...",
            30.0,
            ink,
            v(centre[0], centre[1] + 1.55),
        )?);
        let choices = ["faster", "slower", "the same"];
        let mut chips = Vec::new();
        for (k, c) in choices.iter().enumerate() {
            chips.push(chip(
                &kit,
                stage,
                c,
                30.0,
                dim,
                v(centre[0] - 2.6 + 2.6 * k as f64, centre[1] - 1.45),
            )?);
        }
        let mut ins = shown.clone();
        ins.extend(&chips);
        card.show_frame(stage, &ins, 0.6)?;
        line.wait_last(stage)?;
        card.arm(stage)?;
        card.count(&kit, stage)?;

        // ---- 07_5: slower: 4.32 Hz against 5.91, and not 4.18
        let line = say(&kit, stage, "07_5")?;
        let answer = kit.text(
            stage,
            fact("K-Q.07_4.answer")?,
            34.0,
            own(),
            v(centre[0] - 3.5, card.ring_at()[1]),
        )?;
        card.land(stage, &[answer], 0.3)?;
        let mut away = card.mobs();
        away.extend(&shown);
        away.extend(&chips);
        away.push(answer);
        line.wait_for(stage, "Slower")?;
        fade_out_all(stage, &away, 0.5)?;
        stage.remove_from_scene(card.ring);
        let rows2 = [1.7, -0.9];
        let mut pair = Vec::new();
        let mut frames2: Vec<Mob> = Vec::new();
        for (k, &y) in rows2.iter().enumerate() {
            let layout = Layout {
                origin: [-6.4, y - 0.2, 0.0],
                scale: 0.34,
            };
            frames2.extend(outline(stage, &t, layout, Palette::faint(), 1.0));
            frames2.extend(supports(stage, &t, layout));
            let (e, f) = if k == 0 { (&light, hz[0]) } else { (&heavy, heavy_hz[0]) };
            frames2.extend(mass_discs(stage, &t, layout, if k == 0 { DECK_KG } else { HEAVY_KG }));
            pair.push(swinger(
                stage,
                &t,
                layout,
                vec![(shaped(&t, e, 0, 1.3), f)],
                clock,
                own(),
                5.0,
            )?);
        }
        let two_dp = items("K-V1.hz_2dp")?;
        let light_words = kit.text_left(
            stage,
            &format!("{} Hz", two_dp[0].trim()),
            56.0,
            own(),
            v(3.2, rows2[0] + 0.3),
        )?;
        let heavy_words = kit.text_left(
            stage,
            &format!("{} Hz", fact("K-V2.hz_2dp")?),
            56.0,
            own(),
            v(3.2, rows2[1] + 0.3),
        )?;
        let light_tag = kit.text_left(stage, "4 t at each deck joint", 24.0, dim, v(3.2, rows2[0] - 0.4))?;
        let heavy_tag = kit.text_left(stage, "8 t at each deck joint", 24.0, dim, v(3.2, rows2[1] - 0.4))?;
        let naive = kit.text(
            stage,
            &format!(
                "not {}, which is {} \u{f7} \u{221a}2: the steel did not get heavier",
                fact("K-V2.naive_hz_2dp")?,
                two_dp[0].trim()
            ),
            26.0,
            ink,
            v(0.0, -2.2),
        )?;
        let solver = precisely(
            &kit,
            stage,
            "frequency goes as \u{221a}(stiffness \u{f7} mass); the solver: eigenvalues of M^(\u{2013}1/2) K M^(\u{2013}1/2), numpy.linalg.eigh",
        )?;
        let mut ins = frames2.clone();
        for s in &pair {
            ins.extend(&s.members);
        }
        ins.extend([light_words, heavy_words, light_tag, heavy_tag]);
        for s in &pair {
            stage.add_to_scene(s.anchor)?;
        }
        stage.set_tracker_value(clock, 0.0)?;
        stage.set_tracker_value(running, 1.0)?;
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "with more mass on the same springs")?;
        // the slowed picture says so again (the first tag left with the quiz)
        let slow_again = kit.text(
            stage,
            &format!("slow motion \u{d7} {}", SLOW as u32),
            30.0,
            dim,
            SLOW_TAG_AT,
        )?;
        fade_in_all(stage, &[naive, solver, slow_again], 0.6)?;
        kit.hold(stage)?;

        // ---- 07_6: any small motion is a mix of its modes
        let line = say(&kit, stage, "07_6")?;
        let mut away = frames2.clone();
        for s in &pair {
            away.extend(&s.members);
        }
        away.extend([light_words, heavy_words, light_tag, heavy_tag, naive, solver]);
        fade_out_all(stage, &away, 0.5)?;
        for s in &pair {
            stage.remove_from_scene(s.anchor);
        }
        // the shake on top: the first three modes at once; the clean modes under it
        let top = Layout {
            origin: [-3.6, 1.0, 0.0],
            scale: 0.3,
        };
        let shake_parts: Vec<(Vec<f64>, f64)> = (0..3).map(|k| (shaped(&t, &light, k, 0.9), hz[k])).collect();
        let shake = swinger(stage, &t, top, shake_parts, clock, ink, 5.0)?;
        let mut still: Vec<Mob> = outline(stage, &t, top, Palette::faint(), 1.0);
        still.extend(supports(stage, &t, top));
        let mut bottom = Vec::new();
        let mut bottom_still: Vec<Mob> = Vec::new();
        for (k, &f) in hz.iter().enumerate().take(3) {
            let layout = Layout {
                origin: [-6.4 + 4.6 * k as f64, CLEAN_Y, 0.0],
                scale: CLEAN_SCALE,
            };
            bottom_still.extend(outline(stage, &t, layout, Palette::faint(), 1.0));
            bottom_still.extend(supports(stage, &t, layout));
            bottom.push(swinger(
                stage,
                &t,
                layout,
                vec![(shaped(&t, &light, k, CLEAN_BEND_M), f)],
                clock,
                own(),
                4.0,
            )?);
        }
        let shake_words = kit.text(stage, "a messy shake of the bridge", 28.0, ink, v(0.0, 3.0))?;
        let mix_words = kit.text(stage, "is a mix of three clean modes", 28.0, dim, v(0.0, MIX_Y))?;
        let plane = kit.text(
            stage,
            "this model moves only in its own plane",
            24.0,
            dim,
            v(0.0, PLANE_Y),
        )?;
        let finder = names_to_search(
            &kit,
            stage,
            &[
                "modal analysis",
                "the Lanczos algorithm, with shift-and-invert, finds the lowest few modes of a huge table without the rest",
            ],
        )?;
        let mut ins: Vec<Mob> = still.clone();
        ins.extend(&shake.members);
        ins.extend(&bottom_still);
        for s in &bottom {
            ins.extend(&s.members);
        }
        ins.extend([shake_words]);
        stage.add_to_scene(shake.anchor)?;
        for s in &bottom {
            stage.add_to_scene(s.anchor)?;
        }
        fade_in_all(stage, &ins, 0.6)?;
        line.wait_for(stage, "each keeping its own time")?;
        fade_in_all(stage, &[mix_words, plane], 0.5)?;
        line.wait_for(stage, "So an engineer finds the lowest few")?;
        // the scope note under the pictures goes as the two-line band comes
        // (it stood 0.12 from the band's top, under 0.25)
        cross_fade(stage, &[plane], &[finder], 0.5)?;
        pause(stage, 1.0)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{show, testing};
    use super::super::marks;
    use super::*;
    use crate::model::truss::DEPTH_M;

    /// 07_2 and 07_3: with four tonnes at each deck joint the modes are the
    /// facts' 5.9, 11.8, 14.9 hertz (5.91, 11.78, 14.86 to two places), in
    /// the facts' ratios, and the first is a sag, the second mostly along the
    /// deck and the third an S.
    #[test]
    fn the_modes_are_the_facts_and_the_swing_is_their_sum() {
        let t = Truss::pratt();
        let (e, hz) = modes_of(&t, DECK_KG);
        for (k, &f) in hz.iter().enumerate().take(3) {
            assert_eq!(show(f, 1).unwrap(), fact(&format!("K-V1.hz.{}", k + 1)).unwrap());
        }
        let two = items("K-V1.hz_2dp").unwrap();
        for k in 0..4 {
            assert_eq!(show(hz[k], 2).unwrap(), two[k].trim());
        }
        assert_eq!(show(hz[1] / hz[0], 2).unwrap(), fact("K-V1.ratio.2").unwrap());
        assert_eq!(show(hz[2] / hz[0], 2).unwrap(), fact("K-V1.ratio.3").unwrap());
        // the swing: at the start nothing has moved; a quarter period of mode 1 on, it is at its peak
        let parts = vec![(shaped(&t, &e, 0, 1.4), hz[0])];
        assert!(swung(&parts, 0.0).iter().all(|x| x.abs() < 1e-12));
        let quarter = swung(&parts, 0.25 / hz[0]);
        let top = quarter.iter().fold(0.0f64, |s, x| s.max(x.abs()));
        assert!((top - 1.4).abs() < 1e-9, "{top}");
        // a field and its period: the swing repeats
        let (a, b) = (swung(&parts, 0.123), swung(&parts, 0.123 + 1.0 / hz[0]));
        assert!(a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-9));
        // slowed six times, one second of the screen is a sixth of the bridge's
        assert_eq!(SLOW, 6.0);
    }

    /// 07_1: a pattern turned into a mode by the masses is close to it only
    /// at the low end: the squared overlaps are the facts, and the first
    /// three modes are within a hair of K's patterns.
    #[test]
    fn the_modes_are_close_to_the_patterns_at_the_low_end_only() {
        let t = Truss::pratt();
        let (e, _) = modes_of(&t, DECK_KG);
        let p = t.patterns();
        for k in 0..8 {
            let (phi, q) = (&e.vectors[k], &p.vectors[k]);
            let c = crate::model::linalg::dot(phi, q);
            let overlap = c * c / (crate::model::linalg::dot(phi, phi) * crate::model::linalg::dot(q, q));
            assert_eq!(
                show(overlap, 2).unwrap(),
                fact(&format!("K-V1.overlap.{}", k + 1)).unwrap(),
                "{k}"
            );
        }
        // the discs: the deck joints are the heaviest, the top joints the lightest
        let kg = t.joint_kg(DECK_KG);
        assert!(kg[2] > kg[0] && kg[0] > kg[8]);
        assert!(disc_radius(kg[2]) > disc_radius(kg[8]));
    }

    /// 07_4 and 07_5: twice the deck is slower, 4.32 Hz, and not 5.91 over
    /// the root of two, 4.18, since the steel did not get heavier.
    #[test]
    fn twice_the_deck_is_slower_and_not_by_the_root_of_two() {
        let t = Truss::pratt();
        let (_, light) = modes_of(&t, DECK_KG);
        let (_, heavy) = modes_of(&t, HEAVY_KG);
        assert!(heavy[0] < light[0]);
        assert_eq!(fact("K-Q.07_4.answer").unwrap(), "slower");
        assert_eq!(show(heavy[0], 2).unwrap(), fact("K-V2.hz_2dp").unwrap());
        assert_eq!(
            show(light[0] / 2f64.sqrt(), 2).unwrap(),
            fact("K-V2.naive_hz_2dp").unwrap()
        );
        assert!(
            heavy[0] > light[0] / 2f64.sqrt(),
            "the steel is not doubled, so it is not as slow as the root of two"
        );
        assert_eq!(HEAVY_KG, 8000.0);
        assert_eq!(fact("K-V2.deck_kg").unwrap(), "8000");
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("07_every_mode_a_note").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "K\\varphi = \\omega^2 M\\varphi",
            "slow motion \u{d7} 6",
            "the two end joints carry 2 t each",
            "plus the steel's own 3.4 tonnes, spread over the joints",
            "5.9 Hz",
            "11.8 Hz",
            "14.9 Hz",
            "mostly slides the deck along its length",
            "mostly bends it into an S",
            "4 t at each deck joint",
            "8 t at each deck joint",
            "5.91 Hz",
            "4.32 Hz",
            "not 4.18, which is 5.91 \u{f7} \u{221a}2: the steel did not get heavier",
            "this model moves only in its own plane",
            "modal analysis",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never sets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 35.0 && ran.seconds < 120.0, "{} s", ran.seconds);
    }

    /// 07_3: the listen window lights the mode that is sounding, one by one
    /// at the tones' own times (`sound::BRIDGE_STEP_S` apart), then all three
    /// for the chord at `BRIDGE_TOGETHER_AT`.
    #[test]
    fn the_window_lights_each_mode_as_its_tone_sounds() {
        let voiced = testing::run_voiced("07_every_mode_a_note").unwrap();
        let tag = &voiced.tag;
        let start = marks::at(tag, "07_3.window_start");
        for (k, what) in ["07_3.lit_0", "07_3.lit_1", "07_3.lit_2"].into_iter().enumerate() {
            let want = sound::BRIDGE_STEP_S * k as f64;
            let at = marks::at(tag, what) - start;
            assert!(
                (at - want).abs() <= 0.11,
                "{what}: {at:.2} s into the window, wanted {want:.2}"
            );
        }
        let all = marks::at(tag, "07_3.lit_all") - start;
        assert!((all - sound::BRIDGE_TOGETHER_AT).abs() <= 0.11, "{all:.2} s");
        // each tone sounds inside its slot: the lit moment is not after its end
        const { assert!(sound::BRIDGE_STEP_S * 2.0 + sound::BRIDGE_SOLO_S <= sound::BRIDGE_TOGETHER_AT) };
    }

    /// 07_2: the swings and the seconds open at 0 when they come in.
    #[test]
    fn the_swing_counter_opens_at_zero() {
        let hz = Truss::pratt().hertz(DECK_KG)[0];
        let c0 = 2.7; // what the clock read when 07_1 had run for 16 s
        assert_eq!(counted(c0, c0, hz), 0.0);
        assert_eq!(counted(c0 + 1.0 / hz, c0, hz), 1.0);
    }

    /// 07_6: the words around the three clean modes keep a gap from the
    /// trusses (the mix words from the highest the trusses reach, the plane
    /// words from the supports under them).
    #[test]
    fn the_captions_of_the_mix_keep_off_the_clean_trusses() {
        // 28 pt words are about 0.34 high, 24 pt about 0.3; supports go 0.35
        // below the feet; the shake's lowest point is its foot (1.0) less its
        // largest bend (0.9 m at 0.3 a metre)
        const TOP: f64 = CLEAN_Y + DEPTH_M * CLEAN_SCALE + CLEAN_BEND_M * CLEAN_SCALE;
        const { assert!(MIX_Y - 0.17 >= TOP + 0.1, "mix words clear the trusses' apex") };
        const { assert!(PLANE_Y + 0.15 <= CLEAN_Y - 0.35 - 0.1, "plane words clear the supports") };
        const { assert!(MIX_Y + 0.17 < 1.0 - 0.9 * 0.3, "mix words under the shake") };
    }

    #[test]
    fn the_quiz_is_asked_and_counted_down_as_section_five_asks() {
        let a = testing::audit_quiz("07_4").unwrap();
        assert!((a.window.1 - a.window.0 - a.seconds).abs() <= 0.15, "{a:?}");
        assert_eq!(a.seconds, 4.0);
        assert!(a.card_shown <= a.question_ends, "{a:?}");
        assert!(a.ring_armed <= a.question_ends, "{a:?}");
        assert!(a.window.0 >= a.question_ends - 1e-9, "{a:?}");
        assert!(a.spoken_inside.is_empty(), "{a:?}");
        assert!(a.answer_lands >= a.window.1 - 1e-9, "{a:?}");
        if let Some((next, at)) = &a.next_line {
            assert!(*at >= a.window.1 - 0.05, "{next} starts inside the countdown");
        }
    }
}
