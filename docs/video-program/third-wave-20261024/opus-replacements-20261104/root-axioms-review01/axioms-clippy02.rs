//! Chapter 06: the group requirements, tested on clock moves and then used on
//! the marked square. Every law is shown as a motion rather than only as a
//! formula: move chips and sweeping arcs on the clock, a sum that wraps past
//! twelve, associativity as packaging (two clocks, then a replay of s, r, s on
//! two tiles), full laps dropped from an unrolled tape, and every equation
//! step paired with a tile that is moved and undone. The narration is the
//! recorded 06 text, unchanged; pictures wait for its sentences, and for
//! phrases inside a sentence by their share of its characters
//! (`until_phrase`).
use crate::d4_group::{self as d4g, ALL, E, Elem, R, S};
use crate::tile_actor;
use crate::tile_model::{self, Matrix};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, play, stroke_style, tween, v};
use fmn::prelude::*;
use std::f64::consts::{FRAC_PI_2, TAU};

pub struct Axioms {
    pub kit: Kit,
}

/// The clock and the examples the narration and the pictures use.
pub(crate) const MODULUS: i32 = 12;
/// 06_2: "three hours followed by four hours".
pub(crate) const CLOSURE: (i32, i32) = (3, 4);
/// 06_2: a shown sum that passes twelve ("wrap it around the clock").
pub(crate) const WRAP: (i32, i32) = (8, 7);
/// 06_3: "three plus four, then plus five".
pub(crate) const GROUPED: [i32; 3] = [3, 4, 5];
/// 06_3: three shown moves totalling whole laps, for "taking a remainder
/// before or after adding".
pub(crate) const LAPS: [i32; 3] = [8, 7, 9];
/// 06_4: the move whose identity and inverse are shown.
pub(crate) const START: i32 = 3;

pub(crate) fn clock_add(a: i32, b: i32) -> i32 {
    (a + b).rem_euclid(MODULUS)
}

pub(crate) fn clock_inverse(a: i32) -> i32 {
    (MODULUS - a).rem_euclid(MODULUS)
}

/// One stage of a tile's motion: perform a move, or undo it by running the
/// same path backwards (a quarter-turn is undone by turning back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Do(Elem),
    Undo(Elem),
}

impl Step {
    pub(crate) fn elem(self) -> Elem {
        match self {
            Step::Do(g) => g,
            Step::Undo(g) => g.inv(),
        }
    }
}

/// 06_6: s, then r, then s, in time order (a palindrome, so it also reads
/// s r s written rightmost first). Both groupings replay these steps.
pub(crate) const REGROUP: [Step; 3] = [Step::Do(S), Step::Do(R), Step::Do(S)];
/// 06_6: the same moves with the packaged pair exchanged.
pub(crate) const SWAPPED: [Step; 3] = [REGROUP[0], REGROUP[2], REGROUP[1]];
/// 06_7: a x = b with a the quarter-turn and b the flip. The tile starts at
/// b (after its first step), undoes a to show x, then redoes a.
pub(crate) const SOLVE_A: Elem = R;
pub(crate) const SOLVE_B: Elem = S;
pub(crate) const SOLVE: [Step; 3] = [Step::Do(SOLVE_B), Step::Undo(SOLVE_A), Step::Do(SOLVE_A)];
/// 06_7: undoing a on the wrong side, b a⁻¹, then doing a.
pub(crate) const WRONG_SIDE: [Step; 3] = [Step::Undo(SOLVE_A), Step::Do(SOLVE_B), Step::Do(SOLVE_A)];
/// 06_11: a b with a the quarter-turn and b the flip (b happens first).
pub(crate) const SEQ_A: Elem = R;
pub(crate) const SEQ_B: Elem = S;
pub(crate) const REVERSE_UNDO: [Step; 4] = [Step::Do(SEQ_B), Step::Do(SEQ_A), Step::Undo(SEQ_A), Step::Undo(SEQ_B)];
pub(crate) const SAME_ORDER_UNDO: [Step; 4] = [Step::Do(SEQ_B), Step::Do(SEQ_A), Step::Undo(SEQ_B), Step::Undo(SEQ_A)];

/// The single move equal to `steps` performed in order (each later step
/// multiplies on the left: rightmost first).
pub(crate) fn steps_product(steps: &[Step]) -> Elem {
    steps.iter().fold(E, |acc, &step| step.elem().mul(acc))
}

fn unit(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

/// Undoing `g` as a motion: `d4g::motion(g, ·)` with its direction reversed.
fn undo_motion(g: Elem, t: f64) -> Matrix {
    let t = unit(t);
    match g.mirror() {
        None => tile_model::turn(-f64::from(g.turns) * t),
        Some(eighths) => {
            let half = f64::from(eighths) / 2.0;
            tile_model::compose(
                tile_model::turn(half),
                tile_model::compose(tile_model::flip(-t), tile_model::turn(-half)),
            )
        }
    }
}

/// The tile pose along a tracker: 0 to 1 performs the first step, 1 to 2 the
/// second on the result, and so on.
fn steps_pose(steps: &[Step], t: f64) -> Matrix {
    steps
        .iter()
        .enumerate()
        .fold(tile_model::turn(0.0), |pose, (i, &step)| {
            let local = t - i as f64;
            let motion = match step {
                Step::Do(g) => d4g::motion(g, local),
                Step::Undo(g) => undo_motion(g, local),
            };
            tile_model::compose(motion, pose)
        })
}

fn regroup_pose(t: f64) -> Matrix {
    steps_pose(&REGROUP, t)
}

fn swapped_pose(t: f64) -> Matrix {
    steps_pose(&SWAPPED, t)
}

fn solve_pose(t: f64) -> Matrix {
    steps_pose(&SOLVE, t)
}

fn wrong_side_pose(t: f64) -> Matrix {
    steps_pose(&WRONG_SIDE, t)
}

fn reverse_undo_pose(t: f64) -> Matrix {
    steps_pose(&REVERSE_UNDO, t)
}

fn same_order_undo_pose(t: f64) -> Matrix {
    steps_pose(&SAME_ORDER_UNDO, t)
}

// ---- timing inside a recorded sentence --------------------------------------

const SCRIPT: &str = include_str!("../script.tsv");

/// The recorded text of `cue`.
fn line_text(cue: &str) -> Option<&'static str> {
    SCRIPT.lines().find_map(|l| l.strip_prefix(cue)?.strip_prefix('\t'))
}

/// `text[from..to]` without its surrounding whitespace, as a byte range.
fn trimmed(text: &str, from: usize, to: usize) -> Option<(usize, usize)> {
    let s = &text[from..to];
    let start = from + (s.len() - s.trim_start().len());
    let end = from + s.trim_end().len();
    (start < end).then_some((start, end))
}

/// The byte ranges of a line's sentences, split exactly as the kit splits
/// them (after `.`, `?` or `!` followed by a space or the end).
fn sentence_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    let b = text.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        if matches!(c, b'.' | b'?' | b'!') && (i + 1 == b.len() || b[i + 1] == b' ') {
            out.extend(trimmed(text, from, i + 1));
            from = i + 1;
        }
    }
    out.extend(trimmed(text, from, text.len()));
    out
}

/// When `phrase`, inside sentence `j` of `cue`, begins (scene seconds): the
/// narration's phrase mark at that byte when one exists, otherwise the
/// phrase's share of the sentence's characters between this sentence's start
/// and the next one's (the house rule, as in the other films' `Spoken::at`).
fn phrase_time(kit: &Kit, cue: &str, j: usize, phrase: &str) -> Option<f64> {
    let narrator = kit.narrator.as_ref()?;
    let text = line_text(cue)?;
    let &(from, to) = sentence_ranges(text).get(j)?;
    let byte = from + text[from..to].find(phrase)?;
    let said = narrator.sentence_at(cue, 0)? - narrator.sentence_starts(cue).first().copied()?;
    if let Some(&seconds) = narrator.phrase_marks(cue).get(&byte) {
        return Some(said + seconds);
    }
    let start = narrator.sentence_at(cue, j)?;
    let end = match narrator.sentence_at(cue, j + 1) {
        Some(next) => next,
        None => said + narrator.length(cue)?,
    };
    let share = text[from..byte].chars().count() as f64 / text[from..to].chars().count() as f64;
    Some(start + (end - start) * share)
}

/// Wait until sentence `j` of `cue`, then until `phrase` inside it. Without a
/// narrator, or if the phrase is missing (a test guards every phrase used),
/// only the sentence wait applies; nothing waits once the moment has passed.
fn until_phrase(stage: &mut Stage<'_>, kit: &Kit, cue: &str, j: usize, phrase: &str) -> fmn::Result<()> {
    kit.until_sentence(stage, cue, j)?;
    let Some(at) = phrase_time(kit, cue, j, phrase) else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

/// Phrases the pictures wait for, by cue and sentence.
const REQUIREMENT_PHRASES: [&str; 4] = ["Closure", "associativity", "an identity", "an inverse"];
const RULE_PHRASE: &str = "combining them";
const RESULT_PHRASE: &str = "as three plus";
const RIGHT_UNDO_PHRASE: &str = "and c undo it";

/// One row of the 06_10 derivation.
struct Link {
    /// The expression (TeX).
    piece: &'static str,
    /// The rule that justifies the step (TeX); empty for the first row.
    reason: &'static str,
    /// Whether the rule is one of the group laws (identity, associativity)
    /// rather than an assumption about b or c: sentence 3 lights these.
    law: bool,
    /// The words of sentence 2 the row waits for.
    phrase: Option<&'static str>,
}

/// 06_10 sentence 2, one row per link. The spoken chain skips "e c", so that
/// row follows "finally c" straight into the last one.
const CHAIN: [Link; 6] = [
    Link {
        piece: "b",
        reason: "",
        law: false,
        phrase: None,
    },
    Link {
        piece: "=be",
        reason: r"\text{identity}",
        law: true,
        phrase: Some("b times the identity"),
    },
    Link {
        piece: "=b(ac)",
        reason: "ac=e",
        law: false,
        phrase: Some("then b times a c"),
    },
    Link {
        piece: "=(ba)c",
        reason: r"\text{regroup}",
        law: true,
        phrase: Some("then b a times c"),
    },
    Link {
        piece: "=ec",
        reason: "ba=e",
        law: false,
        phrase: Some("finally c"),
    },
    Link {
        piece: "=c",
        reason: r"\text{identity}",
        law: true,
        phrase: None,
    },
];

/// 06_11 sentence 2: the combined expression, then each meeting cancels.
const CANCELLING: [(&str, Option<&str>); 4] = [
    (r"(ab)(b^{-1}a^{-1})", None),
    (r"a(bb^{-1})a^{-1}", Some("b meets b inverse")),
    (r"aea^{-1}=aa^{-1}", None),
    (r"aa^{-1}=e", Some("then a meets a inverse")),
];

// ---- clock pictures ------------------------------------------------------------

const CLOCK_RADIUS: f64 = 1.55;
const NUMERAL_RADIUS: f64 = 1.05;
/// 06_1's tour: seconds per hour of the sweep, and each chip's pulse.
const TOUR_HOUR: f64 = 0.3;
const TOUR_PULSE: f64 = 0.5;

fn rim(center: (f64, f64), hours: f64, radius: f64) -> Vec3 {
    let angle = FRAC_PI_2 - TAU * hours / f64::from(MODULUS);
    v(center.0 + radius * angle.cos(), center.1 + radius * angle.sin())
}

fn clock_face(stage: &mut Stage<'_>, kit: &Kit, center: (f64, f64)) -> fmn::Result<Vec<Mob>> {
    let ring = stage.arena_mut().add(
        Circle::new()
            .radius(CLOCK_RADIUS)
            .style(stroke_style(Palette::dim(), 3.0))
            .build()
            .moved_to(v(center.0, center.1)),
    );
    let mut parts = vec![ring];
    for hour in 0..MODULUS {
        let at = rim(center, f64::from(hour), NUMERAL_RADIUS);
        parts.push(kit.text(stage, &hour.to_string(), 24.0, Palette::ink(), at)?);
    }
    Ok(parts)
}

/// An advance drawn on a clock: an arc from `from` hours that grows with its
/// tracker (the hours advanced so far), with a dot riding at its tip.
struct Sweep {
    tracker: Mob,
    arc: Mob,
    dot: Mob,
}

impl Sweep {
    fn new(stage: &mut Stage<'_>, center: (f64, f64), from: f64, color: Srgb) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(0.0);
        let arc = stage.always_redraw(move |arena| {
            let hours = arena.tracker_value(tracker).unwrap_or(0.0);
            if hours.abs() < 1e-3 {
                return arena.add(VMobject::new());
            }
            let steps = (hours.abs() * 8.0).ceil() as usize;
            let points: Vec<Vec3> = (0..=steps)
                .map(|i| rim(center, from + hours * i as f64 / steps as f64, CLOCK_RADIUS))
                .collect();
            arena.add(Polygon::polyline(points).style(stroke_style(color, 8.0)).build())
        });
        stage.add_to_scene(arc)?;
        let dot = stage.always_redraw(move |arena| {
            let hours = arena.tracker_value(tracker).unwrap_or(0.0);
            arena.add(Circle::new().radius(0.15).style(ink_style(color)).build().moved_to(rim(
                center,
                from + hours,
                CLOCK_RADIUS,
            )))
        });
        stage.add_to_scene(dot)?;
        Ok(Self { tracker, arc, dot })
    }

    fn parts(&self) -> [Mob; 2] {
        [self.arc, self.dot]
    }

    fn advance(&self, stage: &mut Stage<'_>, hours: i32, run_time: f64) -> fmn::Result<()> {
        play!(stage; tween(self.tracker, f64::from(hours), run_time)?);
        Ok(())
    }
}

/// A move token: a rounded frame with its label.
fn chip(stage: &mut Stage<'_>, kit: &Kit, label: &str, at: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(0.86)
        .height(0.56)
        .corner_radius(0.12)
        .style(stroke_style(color, 3.0))
        .build()?
        .moved_to(v(at.0, at.1));
    let frame = stage.arena_mut().add(frame);
    let text = kit.text(stage, label, 24.0, color, v(at.0, at.1))?;
    d4g::group_of(stage, &[frame, text])
}

/// A move token labelled in TeX (for a, b, inverses).
fn tex_chip(stage: &mut Stage<'_>, kit: &Kit, src: &str, at: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(0.86)
        .height(0.56)
        .corner_radius(0.12)
        .style(stroke_style(color, 3.0))
        .build()?
        .moved_to(v(at.0, at.1));
    let frame = stage.arena_mut().add(frame);
    let text = kit.tex(stage, src, 26.0, color, &[], v(at.0, at.1))?;
    d4g::group_of(stage, &[frame, text])
}

fn bracket(stage: &mut Stage<'_>, center: (f64, f64), width: f64, color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(width)
        .height(0.78)
        .corner_radius(0.16)
        .style(stroke_style(color, 3.0))
        .build()?
        .moved_to(v(center.0, center.1));
    Ok(stage.arena_mut().add(frame))
}

fn tex(stage: &mut Stage<'_>, kit: &Kit, src: &str, size: f64, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    kit.tex(stage, src, size, color, &[], v(at.0, at.1))
}

fn ring(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(0.32)
            .style(stroke_style(color, 5.0))
            .build()
            .moved_to(at),
    )
}

/// A tick (true) or a cross (false), drawn from strokes.
fn mark(stage: &mut Stage<'_>, at: (f64, f64), ok: bool) -> fmn::Result<Mob> {
    let (x, y) = at;
    let (color, strokes) = if ok {
        (
            Palette::green(),
            [
                (v(x - 0.17, y + 0.01), v(x - 0.05, y - 0.12)),
                (v(x - 0.05, y - 0.12), v(x + 0.19, y + 0.16)),
            ],
        )
    } else {
        (
            Palette::gold(),
            [
                (v(x - 0.13, y - 0.13), v(x + 0.13, y + 0.13)),
                (v(x - 0.13, y + 0.13), v(x + 0.13, y - 0.13)),
            ],
        )
    };
    let mut parts = Vec::new();
    for (a, b) in strokes {
        parts.push(
            stage
                .arena_mut()
                .add(Line::new(a, b).style(stroke_style(color, 5.0)).build()?),
        );
    }
    d4g::group_of(stage, &parts)
}

fn arrow(stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(v(from.0, from.1), v(to.0, to.1))
            .buff(0.08)
            .thickness(3.0)
            .style(Style::default().fill(color, 0.9).stroke(color, 3.0, 0.9))
            .build()?,
    ))
}

fn pulse(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), mob, 1.25, None)?.rt(0.6);
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn recolor(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb, run_time: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(run_time),
                ..AnimateArgs::default()
            })?
            .set_color(color)?;
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn morph(stage: &mut Stage<'_>, from: Mob, to: Mob, run_time: f64) -> fmn::Result<()> {
    play!(stage; fmn::animation::replacement_transform(from, to).rt(run_time));
    Ok(())
}

/// The canonical marker that steps hour by hour (kept for 06_4, where the
/// stepping identity and inverse demonstrations were judged clear).
fn step_to(stage: &mut Stage<'_>, marker: Mob, center: (f64, f64), hour: i32) -> fmn::Result<()> {
    let motion = marker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.35),
            ..AnimateArgs::default()
        })?
        .move_to(rim(center, f64::from(hour), CLOCK_RADIUS), ORIGIN)?;
    play!(stage; motion);
    Ok(())
}

// ---- the unrolled tape (06_3, remainder before or after adding) --------------------

const TAPE_UNIT: f64 = 0.4;
const TAPE_X0: f64 = -4.8;

fn tape_x(hours: f64) -> f64 {
    TAPE_X0 + TAPE_UNIT * hours
}

fn bar(stage: &mut Stage<'_>, kit: &Kit, from: i32, to: i32, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let rect = Rectangle::new()
        .width(TAPE_UNIT * f64::from(to - from))
        .height(0.42)
        .style(Style::default().fill(color, 0.8).stroke(color, 2.0, 1.0))
        .build()?
        .moved_to(v(tape_x(f64::from(from + to) / 2.0), y));
    let rect = stage.arena_mut().add(rect);
    let label = kit.text(
        stage,
        &(to - from).to_string(),
        22.0,
        hex("#0E1116"),
        v(tape_x(f64::from(from + to) / 2.0), y),
    )?;
    d4g::group_of(stage, &[rect, label])
}

/// An axis with a tick and a numeral at every full lap.
fn tape(stage: &mut Stage<'_>, kit: &Kit, y: f64, laps: i32) -> fmn::Result<Vec<Mob>> {
    let end = tape_x(f64::from(laps * MODULUS));
    let mut parts = vec![
        stage.arena_mut().add(
            Line::new(v(TAPE_X0, y), v(end, y))
                .style(stroke_style(Palette::dim(), 3.0))
                .build()?,
        ),
    ];
    for lap in 0..=laps {
        let x = tape_x(f64::from(lap * MODULUS));
        parts.push(
            stage.arena_mut().add(
                Line::new(v(x, y - 0.45), v(x, y + 0.45))
                    .style(stroke_style(Palette::faint(), 3.0))
                    .build()?,
            ),
        );
        parts.push(kit.text(
            stage,
            &(lap * MODULUS).to_string(),
            22.0,
            Palette::dim(),
            v(x, y - 0.68),
        )?);
    }
    Ok(parts)
}

impl SceneConstruct for Axioms {
    fn name(&self) -> &str {
        "06_axioms"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();

        // ---- 06_1: from the square's moves to a collection of clock moves ------
        kit.say(stage, "06_1")?;
        let title = kit.text(stage, "What makes a group?", 36.0, ink, v(0.0, 3.0))?;
        let mut rack = Vec::new();
        for (i, g) in ALL.into_iter().enumerate() {
            rack.push(d4g::thumb(stage, &kit, g, (-4.55 + 1.3 * i as f64, 1.0), 0.75)?);
        }
        let source = kit.text(stage, "The square's eight moves", 26.0, Palette::dim(), v(0.0, -0.25))?;
        d4g::show(stage, &[title], 0.4)?;
        d4g::show(stage, &rack, 0.6)?;
        d4g::show(stage, &[source], 0.3)?;
        kit.until_sentence(stage, "06_1", 1)?;
        let mut leaving = rack.clone();
        leaving.push(source);
        d4g::hide(stage, &leaving, 0.5)?;
        let center = (-3.4, -0.35);
        let face = clock_face(stage, &kit, center)?;
        let context = kit.text(stage, "Clock moves: add modulo 12", 29.0, ink, v(0.0, 2.25))?;
        d4g::show(stage, &face, 0.6)?;
        d4g::show(stage, &[context], 0.4)?;
        kit.until_sentence(stage, "06_1", 2)?;
        let moves_label = kit.text(stage, "The moves", 26.0, Palette::dim(), v(3.8, 1.5))?;
        let chip_at = |k: i32| (1.35 + 0.98 * f64::from(k % 6), 0.8 - 0.75 * f64::from(k / 6));
        let mut chips = Vec::new();
        for k in 0..MODULUS {
            chips.push(chip(stage, &kit, &format!("+{k}"), chip_at(k), Palette::blue())?);
        }
        let rule_label = kit.text(
            stage,
            "The rule: combine two moves",
            26.0,
            Palette::dim(),
            v(3.8, -1.05),
        )?;
        d4g::show(stage, &[moves_label], 0.3)?;
        d4g::show(stage, &chips, 0.6)?;
        d4g::show(stage, &[rule_label], 0.3)?;
        kit.until_sentence(stage, "06_1", 3)?;
        // Each advance lands where its chip says: one sweep through all twelve
        // positions, each chip lighting as the tip reaches its hour.
        let tour = Sweep::new(stage, center, 0.0, Palette::blue())?;
        let mut lights: Vec<Box<dyn Animation>> = Vec::new();
        for &c in &chips {
            let a = fmn::animation::indicate(stage.arena_mut(), c, 1.25, None)?.rt(TOUR_PULSE);
            lights.push(stage.prepare(a)?);
        }
        let lights = explainer_kit::lagged(stage, lights, TOUR_HOUR / TOUR_PULSE)?;
        let sweep = tour
            .tracker
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(TOUR_HOUR * f64::from(MODULUS - 1)),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(f64::from(MODULUS - 1))?;
        play!(stage; sweep, lights);
        until_phrase(stage, &kit, "06_1", 3, RULE_PHRASE)?;
        let rule = tex(stage, &kit, r"(a,b)\longmapsto a+b\pmod{12}", 32.0, ink, (3.8, -1.8))?;
        d4g::show(stage, &[rule], 0.4)?;
        d4g::hide(stage, &tour.parts(), 0.3)?;

        // ---- 06_2: closure, then a sum that wraps -------------------------------
        kit.say(stage, "06_2")?;
        let (a, b) = CLOSURE;
        let closure = kit.text(
            stage,
            &format!("Closure: {a} + {b} = {}", clock_add(a, b)),
            30.0,
            Palette::blue(),
            v(0.0, 3.0),
        )?;
        d4g::hide(stage, &[title], 0.3)?;
        d4g::show(stage, &[closure], 0.4)?;
        kit.until_sentence(stage, "06_2", 1)?;
        let first = Sweep::new(stage, center, 0.0, Palette::blue())?;
        first.advance(stage, a, 0.9)?;
        let second = Sweep::new(stage, center, f64::from(a), Palette::gold())?;
        second.advance(stage, b, 1.0)?;
        // The result is one of the twelve chips: the collection is closed.
        let landed = chips[clock_add(a, b) as usize];
        recolor(stage, &[landed], Palette::green(), 0.3)?;
        pulse(stage, &[landed])?;
        kit.until_sentence(stage, "06_2", 2)?;
        let mut old = first.parts().to_vec();
        old.extend(second.parts());
        d4g::hide(stage, &old, 0.3)?;
        let (p, q) = WRAP;
        let wrap_first = Sweep::new(stage, center, 0.0, Palette::blue())?;
        wrap_first.advance(stage, p, 0.8)?;
        let wrap_second = Sweep::new(stage, center, f64::from(p), Palette::gold())?;
        let top = ring(stage, rim(center, 0.0, CLOCK_RADIUS), Palette::hero());
        d4g::show(stage, &[top], 0.2)?;
        wrap_second.advance(stage, q, 1.0)?;
        let wrap_sum = tex(
            stage,
            &kit,
            &format!(r"{p}+{q}={}\rightarrow {}", p + q, clock_add(p, q)),
            32.0,
            Palette::gold(),
            (3.8, -2.6),
        )?;
        d4g::show(stage, &[wrap_sum], 0.3)?;
        let wrapped = chips[clock_add(p, q) as usize];
        recolor(stage, &[wrapped], Palette::green(), 0.3)?;
        pulse(stage, &[wrapped])?;

        // ---- 06_3: associativity as packaging, then remainders on a tape ------------
        kit.say(stage, "06_3")?;
        let mut old = wrap_first.parts().to_vec();
        old.extend(wrap_second.parts());
        old.extend([top, wrap_sum, moves_label, rule_label, rule, closure, context]);
        old.extend(chips.iter().copied());
        old.extend(face.iter().copied());
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(stage, "Associativity: package, do not reorder", 32.0, ink, v(0.0, 3.0))?;
        let centers = [(-3.4, 0.6), (3.4, 0.6)];
        let mut faces = Vec::new();
        for &c in &centers {
            faces.extend(clock_face(stage, &kit, c)?);
        }
        let [x, y, z] = GROUPED;
        let colors = [Palette::blue(), Palette::gold(), Palette::green()];
        let mut rows = Vec::new();
        for &(cx, _) in &centers {
            for (i, (&n, &color)) in GROUPED.iter().zip(&colors).enumerate() {
                rows.push(chip(
                    stage,
                    &kit,
                    &format!("+{n}"),
                    (cx - 1.0 + i as f64, -1.75),
                    color,
                )?);
            }
        }
        let left_box = bracket(stage, (centers[0].0 - 0.5, -1.75), 2.05, Palette::hero())?;
        let right_box = bracket(stage, (centers[1].0 + 0.5, -1.75), 2.05, Palette::hero())?;
        let left_name = kit.text(
            stage,
            &format!("({x} + {y}) + {z}"),
            28.0,
            Palette::blue(),
            v(centers[0].0, -2.6),
        )?;
        let right_name = kit.text(
            stage,
            &format!("{x} + ({y} + {z})"),
            28.0,
            Palette::gold(),
            v(centers[1].0, -2.6),
        )?;
        d4g::show(stage, &[heading], 0.4)?;
        d4g::show(stage, &faces, 0.5)?;
        d4g::show(stage, &rows, 0.4)?;
        d4g::show(stage, &[left_box, right_box, left_name, right_name], 0.4)?;
        kit.until_sentence(stage, "06_3", 1)?;
        // Left: the package (x + y) runs as one sweep, then z.
        let l1 = Sweep::new(stage, centers[0], 0.0, colors[0])?;
        l1.advance(stage, x, 0.7)?;
        let l2 = Sweep::new(stage, centers[0], f64::from(x), colors[1])?;
        l2.advance(stage, y, 0.8)?;
        let left_partial = kit.text(
            stage,
            &format!("= {}", clock_add(x, y)),
            24.0,
            Palette::hero(),
            v(centers[0].0 - 0.5, -1.15),
        )?;
        d4g::show(stage, &[left_partial], 0.25)?;
        stage.wait(0.3)?;
        let l3 = Sweep::new(stage, centers[0], f64::from(x + y), colors[2])?;
        l3.advance(stage, z, 0.8)?;
        until_phrase(stage, &kit, "06_3", 1, RESULT_PHRASE)?;
        // Right: x alone, then the package (y + z) as one sweep.
        let r1 = Sweep::new(stage, centers[1], 0.0, colors[0])?;
        r1.advance(stage, x, 0.7)?;
        stage.wait(0.3)?;
        let r2 = Sweep::new(stage, centers[1], f64::from(x), colors[1])?;
        r2.advance(stage, y, 0.8)?;
        let r3 = Sweep::new(stage, centers[1], f64::from(x + y), colors[2])?;
        r3.advance(stage, z, 0.8)?;
        let right_partial = kit.text(
            stage,
            &format!("= {}", clock_add(y, z)),
            24.0,
            Palette::hero(),
            v(centers[1].0 + 0.5, -1.15),
        )?;
        d4g::show(stage, &[right_partial], 0.25)?;
        kit.until_sentence(stage, "06_3", 2)?;
        let total = clock_add(clock_add(x, y), z);
        let goals = [
            ring(stage, rim(centers[0], f64::from(total), CLOCK_RADIUS), Palette::green()),
            ring(stage, rim(centers[1], f64::from(total), CLOCK_RADIUS), Palette::green()),
        ];
        let destination = kit.text(
            stage,
            &format!("Same destination: {total}"),
            28.0,
            Palette::green(),
            v(0.0, 2.35),
        )?;
        let zero_move = tex(
            stage,
            &kit,
            &format!(r"{x}+{y}+{z}={}\rightarrow {total}", x + y + z),
            30.0,
            Palette::green(),
            (0.0, -3.3),
        )?;
        d4g::show(stage, &goals, 0.3)?;
        d4g::show(stage, &[destination, zero_move], 0.3)?;
        kit.until_sentence(stage, "06_3", 3)?;
        let caution = kit.text(
            stage,
            "One example, not a proof for every choice",
            26.0,
            Palette::dim(),
            v(0.0, 2.35),
        )?;
        morph(stage, destination, caution, 0.5)?;
        kit.until_sentence(stage, "06_3", 4)?;
        let mut old = vec![
            left_box,
            right_box,
            left_name,
            right_name,
            left_partial,
            right_partial,
            caution,
            zero_move,
        ];
        for sweep in [&l1, &l2, &l3, &r1, &r2, &r3] {
            old.extend(sweep.parts());
        }
        old.extend(goals);
        old.extend(faces.iter().copied());
        old.extend(rows.iter().copied());
        d4g::hide(stage, &old, 0.5)?;
        // Unroll the clock: add first and drop whole laps, or drop each lap
        // as soon as it fills. Both rows stop at the same clock position.
        let [p, q, r] = LAPS;
        let laps = (p + q + r) / MODULUS;
        let (upper, lower) = (1.35, -1.45);
        let mut tapes = tape(stage, &kit, upper, laps)?;
        tapes.extend(tape(stage, &kit, lower, laps)?);
        let after = kit.text_left(stage, "Add, then drop full laps", 24.0, ink, v(TAPE_X0, upper + 0.75))?;
        let before = kit.text_left(stage, "Drop each full lap at once", 24.0, ink, v(TAPE_X0, lower + 0.75))?;
        d4g::show(stage, &tapes, 0.4)?;
        d4g::show(stage, &[after, before], 0.3)?;
        let top_bars = [
            bar(stage, &kit, 0, p, upper, colors[0])?,
            bar(stage, &kit, p, p + q, upper, colors[1])?,
            bar(stage, &kit, p + q, p + q + r, upper, colors[2])?,
        ];
        let low_first = bar(stage, &kit, 0, p, lower, colors[0])?;
        let low_lap = bar(stage, &kit, p, MODULUS, lower, colors[1])?;
        let low_rest = bar(stage, &kit, MODULUS, p + q, lower, colors[1])?;
        d4g::show(stage, &top_bars, 0.5)?;
        d4g::show(stage, &[low_first, low_lap, low_rest], 0.5)?;
        // Upper row: the total is whole laps, so it drops to zero.
        let upper_sum = tex(
            stage,
            &kit,
            &format!(r"{p}+{q}+{r}={}\rightarrow {}", p + q + r, clock_add(p + q, r)),
            28.0,
            ink,
            (4.9, upper + 0.75),
        )?;
        d4g::show(stage, &[upper_sum], 0.3)?;
        d4g::hide(stage, &top_bars, 0.5)?;
        let upper_end = ring(stage, v(tape_x(0.0), upper), Palette::green());
        d4g::show(stage, &[upper_end], 0.25)?;
        // Lower row: drop the first full lap, keep the remainder, add the last move.
        let lower_sum = tex(
            stage,
            &kit,
            &format!(r"{p}+{q}={}\rightarrow {}", p + q, clock_add(p, q)),
            28.0,
            ink,
            (4.9, lower + 0.75),
        )?;
        d4g::show(stage, &[lower_sum], 0.3)?;
        d4g::hide(stage, &[low_first, low_lap], 0.4)?;
        let remainder = clock_add(p, q);
        d4g::glide(stage, low_rest, (tape_x(f64::from(remainder) / 2.0), lower), 0.6)?;
        let low_last = bar(stage, &kit, remainder, remainder + r, lower, colors[2])?;
        d4g::show(stage, &[low_last], 0.4)?;
        let lower_next = tex(
            stage,
            &kit,
            &format!(
                r"{remainder}+{r}={}\rightarrow {}",
                remainder + r,
                clock_add(remainder, r)
            ),
            28.0,
            ink,
            (4.9, lower + 0.75),
        )?;
        morph(stage, lower_sum, lower_next, 0.4)?;
        d4g::hide(stage, &[low_rest, low_last], 0.4)?;
        let lower_end = ring(stage, v(tape_x(0.0), lower), Palette::green());
        d4g::show(stage, &[lower_end], 0.25)?;
        let same_place = kit.text(stage, "Same clock position", 28.0, Palette::green(), v(0.0, -3.1))?;
        d4g::show(stage, &[same_place], 0.3)?;

        // ---- 06_4: identity and inverse (the stepping marker, kept) -----------------
        kit.say(stage, "06_4")?;
        let mut old = tapes.clone();
        old.extend([
            after, before, upper_sum, upper_end, lower_next, lower_end, same_place, heading,
        ]);
        d4g::hide(stage, &old, 0.5)?;
        let center = (-3.0, 0.0);
        let face = clock_face(stage, &kit, center)?;
        let marker = stage.arena_mut().add(
            Circle::new()
                .radius(0.15)
                .style(ink_style(Palette::blue()))
                .build()
                .moved_to(rim(center, 0.0, CLOCK_RADIUS)),
        );
        let laws = kit.text(stage, "Identity and inverse", 32.0, ink, v(0.0, 3.0))?;
        d4g::show(stage, &face, 0.4)?;
        d4g::show(stage, &[marker, laws], 0.3)?;
        for hour in 1..=START {
            step_to(stage, marker, center, hour)?;
        }
        let identity = kit.text(
            stage,
            &format!("{START} + 0 = {START}"),
            32.0,
            Palette::green(),
            v(2.8, 0.6),
        )?;
        d4g::show(stage, &[identity], 0.4)?;
        pulse(stage, &[marker])?;
        kit.until_sentence(stage, "06_4", 3)?;
        let inverse_hours = clock_inverse(START);
        let inverse = kit.text(
            stage,
            &format!(
                "{START} + {inverse_hours} = {} (mod {MODULUS})",
                clock_add(START, inverse_hours)
            ),
            30.0,
            Palette::gold(),
            v(2.8, -0.6),
        )?;
        d4g::show(stage, &[inverse], 0.4)?;
        for hour in START + 1..=START + inverse_hours {
            step_to(stage, marker, center, hour)?;
        }

        // ---- 06_5: the list builds as it is read; what is missing ------------------
        kit.say(stage, "06_5")?;
        d4g::hide(stage, &[identity, inverse], 0.3)?;
        let (a, b) = CLOSURE;
        let items = [
            ("Closure", format!("{a}+{b}={}", clock_add(a, b))),
            ("Associativity", format!("({x}+{y})+{z}={x}+({y}+{z})")),
            ("Identity", format!("{START}+0={START}")),
            ("Inverse", format!(r"{START}+{inverse_hours}\equiv 0")),
        ];
        let mut list = Vec::new();
        for (i, ((name, formula), phrase)) in items.iter().zip(REQUIREMENT_PHRASES).enumerate() {
            until_phrase(stage, &kit, "06_5", 0, phrase)?;
            let row_y = 1.75 - 0.85 * i as f64;
            let tick = mark(stage, (-0.35, row_y), true)?;
            let label = kit.text_left(stage, name, 28.0, ink, v(0.1, row_y))?;
            let shown = tex(stage, &kit, formula, 28.0, Palette::blue(), (4.75, row_y))?;
            d4g::show(stage, &[tick, label, shown], 0.3)?;
            list.extend([tick, label, shown]);
        }
        kit.until_sentence(stage, "06_5", 1)?;
        let missing_y = 1.75 - 0.85 * 4.0;
        let missing = kit.text_left(stage, "Commutativity?", 28.0, Palette::dim(), v(0.1, missing_y))?;
        d4g::show(stage, &[missing], 0.4)?;
        kit.until_sentence(stage, "06_5", 2)?;
        let strike = stage.arena_mut().add(
            Line::new(v(0.0, missing_y), v(3.0, missing_y))
                .style(stroke_style(Palette::gold(), 4.0))
                .build()?,
        );
        let optional = kit.text(
            stage,
            "Commutativity is optional",
            26.0,
            Palette::gold(),
            v(4.6, missing_y - 0.65),
        )?;
        d4g::show(stage, &[strike, optional], 0.4)?;
        kit.until_sentence(stage, "06_5", 3)?;
        let mut old = face.clone();
        old.extend([marker, laws, missing, strike, optional]);
        old.extend(list.iter().copied());
        d4g::hide(stage, &old, 0.5)?;
        // The opening experiment, replayed (judged clear at review: kept).
        let order_title = kit.text(stage, "Same two moves; different order", 34.0, ink, v(0.0, 2.8))?;
        let first = stage.add_value_tracker(0.0);
        let second = stage.add_value_tracker(0.0);
        let turn_flip = tile_actor::add_tile(stage, &kit, first, -3.0, 0.0, 1.25, |t| {
            if t <= 1.0 {
                tile_model::turn(t)
            } else {
                tile_model::compose(tile_model::flip(t - 1.0), tile_model::turn(1.0))
            }
        })?;
        let flip_turn = tile_actor::add_tile(stage, &kit, second, 3.0, 0.0, 1.25, |t| {
            if t <= 1.0 {
                tile_model::flip(t)
            } else {
                tile_model::compose(tile_model::turn(t - 1.0), tile_model::flip(1.0))
            }
        })?;
        let tf = kit.text(stage, "Turn, then flip", 28.0, Palette::blue(), v(-3.0, -2.1))?;
        let ft = kit.text(stage, "Flip, then turn", 28.0, Palette::gold(), v(3.0, -2.1))?;
        d4g::show(stage, &[order_title, tf, ft], 0.4)?;
        for destination in [1.0, 2.0] {
            play!(stage; tween(first, destination, 1.4)?, tween(second, destination, 1.4)?);
            stage.wait(0.4)?;
        }

        // ---- 06_6: regrouping replays the same steps; swapping does not ------------
        kit.say(stage, "06_6")?;
        let mut old = turn_flip.clone();
        old.extend(flip_turn.iter().copied());
        old.extend([order_title, tf, ft]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(stage, "Regrouping keeps the sequence", 34.0, ink, v(0.0, 2.8))?;
        let sides = [-3.4, 3.4];
        let trackers = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let mut tiles = tile_actor::add_tile(stage, &kit, trackers[0], sides[0], 0.55, 0.95, regroup_pose)?;
        let right_tile = tile_actor::add_tile(stage, &kit, trackers[1], sides[1], 0.55, 0.95, regroup_pose)?;
        let mut timeline = [Vec::new(), Vec::new()];
        for (side, row) in sides.iter().zip(timeline.iter_mut()) {
            for (i, step) in REGROUP.iter().enumerate() {
                row.push(chip(
                    stage,
                    &kit,
                    step.elem().name(),
                    (side - 1.0 + i as f64, -1.3),
                    ink,
                )?);
            }
        }
        let groups = [
            bracket(stage, (sides[0] + 0.5, -1.3), 2.05, Palette::hero())?,
            bracket(stage, (sides[1] - 0.5, -1.3), 2.05, Palette::hero())?,
        ];
        let names = [
            tex(stage, &kit, "(sr)s", 34.0, Palette::blue(), (sides[0], -2.15))?,
            tex(stage, &kit, "s(rs)", 34.0, Palette::gold(), (sides[1], -2.15))?,
        ];
        d4g::show(stage, &[heading], 0.4)?;
        d4g::show(stage, &timeline.concat(), 0.4)?;
        d4g::show(stage, &groups, 0.3)?;
        d4g::show(stage, &names, 0.3)?;
        kit.until_sentence(stage, "06_6", 1)?;
        // (sr)s: s alone, then the package r-then-s in one motion; s(rs): the
        // package s-then-r, then s alone. The pause marks where the package
        // boundary falls; the moves and their order never change.
        for (side, cut) in [(0, 1), (1, 2)] {
            recolor(stage, &timeline[side][..cut], Palette::hero(), 0.15)?;
            play!(stage; tween(trackers[side], cut as f64, 0.65 * cut as f64)?);
            stage.wait(0.25)?;
            recolor(stage, &timeline[side][cut..], Palette::hero(), 0.15)?;
            play!(stage; tween(trackers[side], 3.0, 0.65 * (3 - cut) as f64)?);
        }
        let agree = tex(stage, &kit, "=", 60.0, Palette::green(), (0.0, 0.55))?;
        let regrouped = tex(
            stage,
            &kit,
            &format!("(sr)s=s(rs)={}", steps_product(&REGROUP).tex()),
            36.0,
            Palette::green(),
            (0.0, -3.1),
        )?;
        d4g::show(stage, &[agree, regrouped], 0.4)?;
        kit.until_sentence(stage, "06_6", 2)?;
        // Exchange the last two moves on the right: a different order, a different tile.
        d4g::hide(stage, &[groups[1], names[1], agree], 0.3)?;
        let (p1, p2) = (timeline[1][1], timeline[1][2]);
        d4g::glide_all(stage, &[(p1, (sides[1] + 1.0, -1.3)), (p2, (sides[1], -1.3))], 0.6)?;
        d4g::hide(stage, &right_tile, 0.3)?;
        let swap_tracker = stage.add_value_tracker(0.0);
        let swapped = tile_actor::add_tile(stage, &kit, swap_tracker, sides[1], 0.55, 0.95, swapped_pose)?;
        play!(stage; tween(swap_tracker, 3.0, 1.8)?);
        let differs = mark(stage, (0.0, 0.55), false)?;
        let swapped_name = tex(
            stage,
            &kit,
            &format!("rss={}", steps_product(&SWAPPED).tex()),
            34.0,
            Palette::gold(),
            (sides[1], -2.15),
        )?;
        d4g::show(stage, &[differs, swapped_name], 0.3)?;
        kit.until_sentence(stage, "06_6", 3)?;
        let distinction = kit.text(
            stage,
            "Changing parentheses is not swapping moves",
            28.0,
            Palette::hero(),
            v(0.0, 2.05),
        )?;
        d4g::show(stage, &[distinction], 0.4)?;

        // ---- 06_7: solve a x = b by undoing a, on the tile and in symbols -----------
        kit.say(stage, "06_7")?;
        tiles.extend(swapped.iter().copied());
        tiles.extend(timeline.concat());
        tiles.extend([
            groups[0],
            names[0],
            regrouped,
            differs,
            swapped_name,
            distinction,
            heading,
        ]);
        d4g::hide(stage, &tiles, 0.5)?;
        let heading = kit.text(stage, "Undo the same move on the same side", 34.0, ink, v(0.0, 2.8))?;
        let question = tex(stage, &kit, "ax=b", 44.0, Palette::hero(), (-3.4, 1.55))?;
        d4g::show(stage, &[heading, question], 0.4)?;
        kit.until_sentence(stage, "06_7", 1)?;
        // One example on the marked square: the tile performs b, then a is undone.
        let instance = tex(
            stage,
            &kit,
            &format!(r"\text{{Example: }}a={},\quad b={}", SOLVE_A.tex(), SOLVE_B.tex()),
            28.0,
            ink,
            (3.9, 2.2),
        )?;
        let target = d4g::thumb(stage, &kit, SOLVE_B, (2.4, 0.75), 0.9)?;
        let target_name = tex(stage, &kit, "b", 30.0, ink, (2.4, -0.05))?;
        d4g::show(stage, &[instance, target, target_name], 0.4)?;
        let solve_tracker = stage.add_value_tracker(0.0);
        let solver = tile_actor::add_tile(stage, &kit, solve_tracker, 5.3, 0.75, 0.75, solve_pose)?;
        play!(stage; tween(solve_tracker, 1.0, 1.0)?);
        let undo_eq = tex(stage, &kit, r"a^{-1}(ax)=a^{-1}b", 36.0, Palette::blue(), (-3.4, 0.55))?;
        let undo_arrow = arrow(stage, (3.0, 1.45), (4.0, 1.45), Palette::blue())?;
        let undo_name = tex(stage, &kit, r"\text{undo }a", 24.0, Palette::blue(), (3.5, 1.75))?;
        d4g::show(stage, &[undo_eq, undo_arrow, undo_name], 0.4)?;
        play!(stage; tween(solve_tracker, 2.0, 1.4)?);
        let x = steps_product(&SOLVE[..2]);
        let x_name = tex(
            stage,
            &kit,
            &format!("x=a^{{-1}}b={}", x.tex()),
            28.0,
            Palette::green(),
            (5.3, -0.4),
        )?;
        d4g::show(stage, &[x_name], 0.3)?;
        kit.until_sentence(stage, "06_7", 2)?;
        let answer = tex(
            stage,
            &kit,
            r"(a^{-1}a)x=ex=x=a^{-1}b",
            30.0,
            Palette::green(),
            (-3.4, -0.45),
        )?;
        d4g::show(stage, &[answer], 0.4)?;
        // Check: x, then a, reaches b.
        play!(stage; tween(solve_tracker, 3.0, 1.2)?);
        let reached = d4g::slot_frame(stage, (2.4, 0.75), 0.9, Palette::green())?;
        let reached_mark = mark(stage, (6.45, 0.75), true)?;
        d4g::show(stage, &[reached, reached_mark], 0.3)?;
        kit.until_sentence(stage, "06_7", 3)?;
        let wrong_eq = tex(stage, &kit, r"(ax)a^{-1}=ba^{-1}", 30.0, Palette::gold(), (-3.4, -1.45))?;
        d4g::show(stage, &[wrong_eq], 0.3)?;
        let wrong_tracker = stage.add_value_tracker(0.0);
        let wrong_tile = tile_actor::add_tile(stage, &kit, wrong_tracker, 5.3, -2.1, 0.75, wrong_side_pose)?;
        play!(stage; tween(wrong_tracker, 2.0, 1.6)?);
        let wrong = steps_product(&WRONG_SIDE[..2]);
        let wrong_name = tex(
            stage,
            &kit,
            &format!("ba^{{-1}}={}", wrong.tex()),
            28.0,
            Palette::gold(),
            (2.6, -2.1),
        )?;
        d4g::show(stage, &[wrong_name], 0.3)?;
        play!(stage; tween(wrong_tracker, 3.0, 1.0)?);
        let missed = mark(stage, (6.45, -2.1), false)?;
        let missed_name = kit.text(stage, "then a: misses b", 24.0, Palette::gold(), v(2.6, -2.75))?;
        d4g::show(stage, &[missed, missed_name], 0.3)?;
        kit.until_sentence(stage, "06_7", 4)?;
        let side = kit.text(stage, "The side matters", 28.0, Palette::gold(), v(-3.4, -2.6))?;
        d4g::show(stage, &[side], 0.3)?;

        // ---- 06_8: cancellation, symbolic then on two tiles -----------------------------
        kit.say(stage, "06_8")?;
        let mut old = solver.clone();
        old.extend(wrong_tile.iter().copied());
        old.extend([
            question,
            instance,
            target,
            target_name,
            undo_eq,
            undo_arrow,
            undo_name,
            x_name,
            answer,
            reached,
            reached_mark,
            wrong_eq,
            wrong_name,
            missed,
            missed_name,
            side,
        ]);
        d4g::hide(stage, &old, 0.5)?;
        let equal = tex(stage, &kit, "ax=ay", 42.0, Palette::hero(), (0.0, 1.65))?;
        let la = tex(stage, &kit, "a", 48.0, Palette::gold(), (-3.0, 0.2))?;
        let lx = tex(stage, &kit, "x", 48.0, Palette::green(), (-2.0, 0.2))?;
        let ra = tex(stage, &kit, "a", 48.0, Palette::gold(), (2.0, 0.2))?;
        let ry = tex(stage, &kit, "y", 48.0, Palette::green(), (3.0, 0.2))?;
        let balance = tex(stage, &kit, "=", 44.0, ink, (0.0, 0.2))?;
        d4g::show(stage, &[equal, la, lx, ra, ry, balance], 0.4)?;
        kit.until_sentence(stage, "06_8", 1)?;
        let li = tex(stage, &kit, r"a^{-1}", 44.0, Palette::blue(), (-5.3, 0.2))?;
        let ri = tex(stage, &kit, r"a^{-1}", 44.0, Palette::blue(), (0.1, -1.3))?;
        d4g::show(stage, &[li, ri], 0.4)?;
        d4g::glide_all(stage, &[(li, (-4.0, 0.2)), (ri, (1.0, 0.2))], 0.8)?;
        let cancel = tex(
            stage,
            &kit,
            r"a^{-1}(ax)=a^{-1}(ay)",
            34.0,
            Palette::blue(),
            (0.0, -1.5),
        )?;
        d4g::show(stage, &[cancel], 0.4)?;
        kit.until_sentence(stage, "06_8", 2)?;
        // Collapse adjacent inverse pairs without moving a past x or y.
        let le = tex(stage, &kit, "e", 48.0, Palette::blue(), (-3.5, 0.2))?;
        let re = tex(stage, &kit, "e", 48.0, Palette::blue(), (1.5, 0.2))?;
        d4g::hide(stage, &[li, la, ri, ra], 0.4)?;
        d4g::show(stage, &[le, re], 0.4)?;
        stage.wait(0.6)?;
        d4g::hide(stage, &[le, re, cancel], 0.4)?;
        d4g::glide_all(stage, &[(lx, (-0.9, 0.2)), (ry, (0.9, 0.2))], 0.8)?;
        kit.until_sentence(stage, "06_8", 3)?;
        let law = tex(
            stage,
            &kit,
            r"ax=ay\ \Rightarrow\ x=y",
            36.0,
            Palette::green(),
            (0.0, 2.0),
        )?;
        morph(stage, equal, law, 0.5)?;
        kit.until_sentence(stage, "06_8", 4)?;
        // Two proposed solutions of a x = b: both tiles show b; undo a on both.
        d4g::glide_all(
            stage,
            &[(lx, (-0.9, 1.15)), (balance, (0.0, 1.15)), (ry, (0.9, 1.15))],
            0.5,
        )?;
        let cancel_tracker = stage.add_value_tracker(1.0);
        let mut pair = tile_actor::add_tile(stage, &kit, cancel_tracker, -2.6, -0.95, 0.75, solve_pose)?;
        pair.extend(tile_actor::add_tile(
            stage,
            &kit,
            cancel_tracker,
            2.6,
            -0.95,
            0.75,
            solve_pose,
        )?);
        let before_names = [
            tex(stage, &kit, "ax=b", 28.0, ink, (-2.6, -2.35))?,
            tex(stage, &kit, "ay=b", 28.0, ink, (2.6, -2.35))?,
        ];
        d4g::show(stage, &before_names, 0.3)?;
        play!(stage; tween(cancel_tracker, 2.0, 1.4)?);
        let after_names = [
            tex(stage, &kit, "x", 32.0, Palette::green(), (-2.6, -2.35))?,
            tex(stage, &kit, "y", 32.0, Palette::green(), (2.6, -2.35))?,
        ];
        morph(stage, before_names[0], after_names[0], 0.4)?;
        morph(stage, before_names[1], after_names[1], 0.4)?;
        let only = tex(
            stage,
            &kit,
            &format!("x=y=a^{{-1}}b={}", x.tex()),
            30.0,
            Palette::green(),
            (0.0, -0.95),
        )?;
        d4g::show(stage, &[only], 0.4)?;
        kit.until_sentence(stage, "06_8", 5)?;
        let right_law = tex(
            stage,
            &kit,
            r"xa=ya\ \Rightarrow\ x=y",
            32.0,
            Palette::gold(),
            (0.0, -3.2),
        )?;
        d4g::show(stage, &[right_law], 0.4)?;

        // ---- 06_9: one product, two readings ----------------------------------------------
        kit.say(stage, "06_9")?;
        let mut old = pair.clone();
        old.extend(after_names);
        old.extend([law, lx, balance, ry, only, right_law, heading]);
        d4g::hide(stage, &old, 0.5)?;
        let unique = kit.text(stage, "Could there be two identities?", 34.0, ink, v(0.0, 2.8))?;
        let candidates = [
            tex_chip(stage, &kit, "e", (-2.4, 1.7), Palette::blue())?,
            tex_chip(stage, &kit, "f", (2.4, 1.7), Palette::gold())?,
        ];
        let does_nothing = [
            kit.text(stage, "does nothing?", 22.0, Palette::dim(), v(-2.4, 1.1))?,
            kit.text(stage, "does nothing?", 22.0, Palette::dim(), v(2.4, 1.1))?,
        ];
        d4g::show(stage, &[unique], 0.4)?;
        d4g::show(stage, &candidates, 0.3)?;
        d4g::show(stage, &does_nothing, 0.3)?;
        kit.until_sentence(stage, "06_9", 1)?;
        let pe = tex(stage, &kit, "e", 48.0, Palette::blue(), (-0.2, 0.25))?;
        let pf = tex(stage, &kit, "f", 48.0, Palette::gold(), (0.2, 0.25))?;
        d4g::show(stage, &[pe, pf], 0.4)?;
        let to_left = arrow(stage, (-0.4, -0.15), (-2.4, -1.15), Palette::blue())?;
        let left_reason = kit.text(
            stage,
            "e does nothing on the left",
            22.0,
            Palette::blue(),
            v(-3.6, -0.35),
        )?;
        d4g::show(stage, &[to_left, left_reason], 0.3)?;
        pulse(stage, &[pe])?;
        let left_value = tex(stage, &kit, "ef=f", 42.0, Palette::blue(), (-2.6, -1.55))?;
        play!(stage; fmn::animation::transform_from_copy(stage.arena_mut(), pf, left_value)?.rt(0.6));
        kit.until_sentence(stage, "06_9", 2)?;
        let to_right = arrow(stage, (0.4, -0.15), (2.4, -1.15), Palette::gold())?;
        let right_reason = kit.text(
            stage,
            "f does nothing on the right",
            22.0,
            Palette::gold(),
            v(3.6, -0.35),
        )?;
        d4g::show(stage, &[to_right, right_reason], 0.3)?;
        pulse(stage, &[pf])?;
        let right_value = tex(stage, &kit, "ef=e", 42.0, Palette::gold(), (2.6, -1.55))?;
        play!(stage; fmn::animation::transform_from_copy(stage.arena_mut(), pe, right_value)?.rt(0.6));
        kit.until_sentence(stage, "06_9", 3)?;
        // The same product, read twice: both readings light, then they join.
        pulse(stage, &[pe, pf, left_value, right_value])?;
        let one_id = tex(stage, &kit, "e=ef=f", 44.0, Palette::green(), (0.0, -2.6))?;
        d4g::show(stage, &[one_id], 0.5)?;
        kit.until_sentence(stage, "06_9", 4)?;
        let unique_id = kit.text(stage, "The identity is unique", 28.0, Palette::green(), v(0.0, -3.4))?;
        d4g::show(stage, &[unique_id], 0.3)?;

        // ---- 06_10: the chain, one justified step at a time -------------------------------
        kit.say(stage, "06_10")?;
        let mut old = candidates.to_vec();
        old.extend(does_nothing);
        old.extend([
            unique,
            pe,
            pf,
            to_left,
            left_reason,
            left_value,
            to_right,
            right_reason,
            right_value,
            one_id,
            unique_id,
        ]);
        d4g::hide(stage, &old, 0.5)?;
        let inverse_title = kit.text(stage, "Could a move have two inverses?", 34.0, ink, v(0.0, 2.8))?;
        d4g::show(stage, &[inverse_title], 0.4)?;
        kit.until_sentence(stage, "06_10", 1)?;
        let left_undo = tex(stage, &kit, "ba=e", 40.0, Palette::blue(), (-2.6, 1.75))?;
        let left_undo_name = kit.text(stage, "b undoes a on the left", 22.0, Palette::blue(), v(-2.6, 1.15))?;
        d4g::show(stage, &[left_undo, left_undo_name], 0.4)?;
        until_phrase(stage, &kit, "06_10", 1, RIGHT_UNDO_PHRASE)?;
        let right_undo = tex(stage, &kit, "ac=e", 40.0, Palette::gold(), (2.6, 1.75))?;
        let right_undo_name = kit.text(stage, "c undoes a on the right", 22.0, Palette::gold(), v(2.6, 1.15))?;
        d4g::show(stage, &[right_undo, right_undo_name], 0.4)?;
        kit.until_sentence(stage, "06_10", 2)?;
        // The chain as a derivation: each row arrives with its words, the rule
        // it uses beside it; an assumption lights the line it came from.
        let mut chain = Vec::new();
        let mut reasons = Vec::new();
        for (i, link) in CHAIN.iter().enumerate() {
            if let Some(phrase) = link.phrase {
                until_phrase(stage, &kit, "06_10", 2, phrase)?;
            }
            let row_y = 0.6 - 0.58 * i as f64;
            let x = if i == 0 { -2.75 } else { -2.3 };
            let shown = kit.tex_left(stage, link.piece, 36.0, Palette::green(), &[], v(x, row_y))?;
            chain.push(shown);
            let mut arriving = vec![shown];
            if !link.reason.is_empty() {
                let color = if link.law { Palette::dim() } else { Palette::hero() };
                let note = kit.tex_left(stage, link.reason, 24.0, color, &[], v(0.1, row_y))?;
                arriving.push(note);
                reasons.push((link.law, note));
            }
            d4g::show(stage, &arriving, 0.3)?;
            match link.reason {
                "ac=e" => pulse(stage, &[right_undo])?,
                "ba=e" => pulse(stage, &[left_undo])?,
                _ => {}
            }
        }
        kit.until_sentence(stage, "06_10", 3)?;
        let laws: Vec<Mob> = reasons.iter().filter(|&&(law, _)| law).map(|&(_, note)| note).collect();
        recolor(stage, &laws, Palette::green(), 0.3)?;
        pulse(stage, &laws)?;
        kit.until_sentence(stage, "06_10", 4)?;
        pulse(stage, &[chain[0], chain[chain.len() - 1]])?;
        let coincide = tex(stage, &kit, "b=c", 44.0, Palette::green(), (3.7, -0.55))?;
        let unique_inverse = kit.text(stage, "The inverse is unique", 28.0, Palette::green(), v(3.7, -1.3))?;
        d4g::show(stage, &[coincide, unique_inverse], 0.4)?;

        // ---- 06_11: undo a sequence: reverse order works, same order does not -------------
        kit.say(stage, "06_11")?;
        let mut old = chain.clone();
        old.extend(reasons.iter().map(|&(_, note)| note));
        old.extend([
            inverse_title,
            left_undo,
            left_undo_name,
            right_undo,
            right_undo_name,
            coincide,
            unique_inverse,
        ]);
        d4g::hide(stage, &old, 0.5)?;
        let reverse_title = kit.text(stage, "Undo a sequence in reverse order", 34.0, ink, v(0.0, 2.8))?;
        let instance = tex(
            stage,
            &kit,
            &format!(r"a={},\quad b={}", SEQ_A.tex(), SEQ_B.tex()),
            28.0,
            ink,
            (0.0, 2.2),
        )?;
        let sides = [-4.0, 4.0];
        let undo_trackers = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let mut undo_tiles =
            tile_actor::add_tile(stage, &kit, undo_trackers[0], sides[0], -0.45, 0.8, reverse_undo_pose)?;
        undo_tiles.extend(tile_actor::add_tile(
            stage,
            &kit,
            undo_trackers[1],
            sides[1],
            -0.45,
            0.8,
            same_order_undo_pose,
        )?);
        let tex_name = |step: Step| match step {
            Step::Do(g) if g == SEQ_A => "a",
            Step::Do(_) => "b",
            Step::Undo(g) if g == SEQ_A => "a^{-1}",
            Step::Undo(_) => "b^{-1}",
        };
        let mut lines = [Vec::new(), Vec::new()];
        for ((side, steps), line) in sides.iter().zip([REVERSE_UNDO, SAME_ORDER_UNDO]).zip(lines.iter_mut()) {
            for (i, &step) in steps.iter().enumerate() {
                line.push(tex_chip(
                    stage,
                    &kit,
                    tex_name(step),
                    (side - 1.5 + i as f64, -2.1),
                    ink,
                )?);
            }
        }
        let captions = [
            kit.text(
                stage,
                "Undo in reverse order",
                24.0,
                Palette::green(),
                v(sides[0], 1.05),
            )?,
            kit.text(
                stage,
                "Undo in the same order",
                24.0,
                Palette::gold(),
                v(sides[1], 1.05),
            )?,
        ];
        let mut setup = vec![reverse_title, instance];
        setup.extend(lines.concat());
        setup.extend(captions);
        d4g::show(stage, &setup, 0.4)?;
        // Forward on both tiles: b, then a.
        for k in 0..2 {
            recolor(stage, &[lines[0][k], lines[1][k]], Palette::hero(), 0.15)?;
            play!(stage; tween(undo_trackers[0], (k + 1) as f64, 0.6)?, tween(undo_trackers[1], (k + 1) as f64, 0.6)?);
        }
        // Left: undo a first, then b, and the tile comes home.
        for (k, &line) in lines[0].iter().enumerate().take(4).skip(2) {
            recolor(stage, &[line], Palette::hero(), 0.15)?;
            play!(stage; tween(undo_trackers[0], (k + 1) as f64, 0.6)?);
        }
        let home = mark(stage, (sides[0], -2.85), true)?;
        let home_eq = tex(
            stage,
            &kit,
            &format!("b^{{-1}}a^{{-1}}ab={}", steps_product(&REVERSE_UNDO).tex()),
            28.0,
            Palette::green(),
            (sides[0], -3.45),
        )?;
        d4g::show(stage, &[home, home_eq], 0.3)?;
        kit.until_sentence(stage, "06_11", 1)?;
        let product = tex(
            stage,
            &kit,
            "(ab)^{-1}=b^{-1}a^{-1}",
            34.0,
            Palette::blue(),
            (0.0, 1.55),
        )?;
        d4g::show(stage, &[product], 0.4)?;
        // The cancellation, one meeting at a time, each with its words.
        kit.until_sentence(stage, "06_11", 2)?;
        let mut current = None;
        for &(src, phrase) in &CANCELLING {
            if let Some(phrase) = phrase {
                until_phrase(stage, &kit, "06_11", 2, phrase)?;
            }
            let next = tex(stage, &kit, src, 32.0, Palette::green(), (0.0, 0.55))?;
            match current {
                None => d4g::show(stage, &[next], 0.3)?,
                Some(previous) => morph(stage, previous, next, 0.5)?,
            }
            current = Some(next);
        }
        kit.until_sentence(stage, "06_11", 3)?;
        // Right: undo b first, then a, and the tile does not come home.
        for (k, &line) in lines[1].iter().enumerate().take(4).skip(2) {
            recolor(stage, &[line], Palette::hero(), 0.15)?;
            play!(stage; tween(undo_trackers[1], (k + 1) as f64, 0.7)?);
        }
        let stuck = mark(stage, (sides[1], -2.85), false)?;
        let stuck_eq = tex(
            stage,
            &kit,
            &format!("a^{{-1}}b^{{-1}}ab={}", steps_product(&SAME_ORDER_UNDO).tex()),
            28.0,
            Palette::gold(),
            (sides[1], -3.45),
        )?;
        d4g::show(stage, &[stuck, stuck_eq], 0.3)?;
        kit.until_sentence(stage, "06_11", 4)?;
        let every = kit.text(
            stage,
            "These deductions apply to every group",
            26.0,
            Palette::green(),
            v(0.0, -0.1),
        )?;
        d4g::show(stage, &[every], 0.4)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::d4_group::{R2, R2S, R3, R3S, RS};

    fn same(a: Matrix, b: Matrix) -> bool {
        tile_model::CORNERS.iter().chain([&[0.0, 0.0, 1.0]]).all(|&p| {
            tile_model::apply(a, p)
                .iter()
                .zip(tile_model::apply(b, p))
                .all(|(x, y)| (x - y).abs() < 1e-9)
        })
    }

    #[test]
    fn every_tile_motion_ends_at_the_product_of_its_steps() {
        for steps in [
            &REGROUP[..],
            &SWAPPED,
            &SOLVE,
            &WRONG_SIDE,
            &REVERSE_UNDO,
            &SAME_ORDER_UNDO,
        ] {
            for k in 0..=steps.len() {
                assert!(
                    same(steps_pose(steps, k as f64), steps_product(&steps[..k]).matrix()),
                    "{steps:?} at {k}"
                );
            }
        }
    }

    #[test]
    fn regrouping_agrees_and_swapping_differs() {
        assert_eq!(steps_product(&REGROUP), R3);
        assert_eq!(S.mul(R).mul(S), S.mul(R.mul(S)));
        assert_eq!(steps_product(&SWAPPED), R);
        assert_ne!(steps_product(&SWAPPED), steps_product(&REGROUP));
    }

    #[test]
    fn solving_on_the_correct_side_reaches_b_and_the_wrong_side_misses() {
        let x = steps_product(&SOLVE[..2]);
        assert_eq!(x, SOLVE_A.inv().mul(SOLVE_B));
        assert_eq!(x, R3S);
        assert_eq!(SOLVE_A.mul(x), SOLVE_B);
        assert_eq!(steps_product(&SOLVE), SOLVE_B);
        let wrong = steps_product(&WRONG_SIDE[..2]);
        assert_eq!(wrong, SOLVE_B.mul(SOLVE_A.inv()));
        assert_eq!(wrong, RS);
        assert_ne!(wrong, x);
        assert_eq!(steps_product(&WRONG_SIDE), R2S);
        assert_ne!(steps_product(&WRONG_SIDE), SOLVE_B);
    }

    #[test]
    fn cancellation_holds_in_the_square_group() {
        for a in ALL {
            for x in ALL {
                for y in ALL {
                    assert_eq!(a.mul(x) == a.mul(y), x == y);
                    assert_eq!(x.mul(a) == y.mul(a), x == y);
                }
            }
        }
    }

    #[test]
    fn identity_and_inverses_are_unique_in_the_square_group() {
        let identities: Vec<Elem> = ALL
            .into_iter()
            .filter(|&f| ALL.iter().all(|&g| f.mul(g) == g && g.mul(f) == g))
            .collect();
        assert_eq!(identities, vec![E]);
        for a in ALL {
            let left: Vec<Elem> = ALL.into_iter().filter(|&b| b.mul(a) == E).collect();
            let right: Vec<Elem> = ALL.into_iter().filter(|&c| a.mul(c) == E).collect();
            assert_eq!(left, vec![a.inv()]);
            assert_eq!(right, vec![a.inv()]);
        }
    }

    #[test]
    fn undoing_in_reverse_order_comes_home_and_same_order_does_not() {
        assert_eq!(steps_product(&REVERSE_UNDO), E);
        assert_eq!(steps_product(&SAME_ORDER_UNDO), R2);
        for a in ALL {
            for b in ALL {
                assert_eq!(a.mul(b).inv(), b.inv().mul(a.inv()));
            }
        }
    }

    #[test]
    fn clock_examples_match_the_narration_and_the_pictures() {
        assert_eq!(clock_add(CLOSURE.0, CLOSURE.1), 7);
        let (p, q) = WRAP;
        assert!(p + q > MODULUS);
        assert_eq!((p + q, clock_add(p, q)), (15, 3));
        let [x, y, z] = GROUPED;
        assert_eq!((x + y, y + z, x + y + z), (7, 9, 12));
        assert_eq!(clock_add(clock_add(x, y), z), clock_add(x, clock_add(y, z)));
        assert_eq!(clock_add(clock_add(x, y), z), 0);
        let [a, b, c] = LAPS;
        assert_eq!((a + b + c) % MODULUS, 0, "the lap example totals whole laps");
        assert_eq!(
            ((a + b + c) / MODULUS, clock_add(a, b), clock_add(a, b) + c),
            (2, 3, 12)
        );
        assert_eq!(clock_add(START, 0), START);
        assert_eq!((clock_inverse(START), clock_add(START, clock_inverse(START))), (9, 0));
        // Remainder before or after adding gives the same position, for every triple.
        for a in 0..MODULUS {
            for b in 0..MODULUS {
                for c in 0..MODULUS {
                    assert_eq!(clock_add(clock_add(a, b), c), (a + b + c).rem_euclid(MODULUS));
                }
            }
        }
    }

    fn sentence(cue: &str, j: usize) -> &'static str {
        let text = line_text(cue).unwrap_or_else(|| panic!("{cue} is not in the script"));
        let (from, to) = *sentence_ranges(text)
            .get(j)
            .unwrap_or_else(|| panic!("{cue} has no sentence {j}"));
        &text[from..to]
    }

    #[test]
    fn the_recorded_cues_split_into_the_sentences_the_pictures_wait_for() {
        // The highest `until_sentence` index each cue uses is below these.
        let counts = [
            ("06_1", 4),
            ("06_2", 3),
            ("06_3", 5),
            ("06_4", 4),
            ("06_5", 4),
            ("06_6", 4),
            ("06_7", 5),
            ("06_8", 6),
            ("06_9", 5),
            ("06_10", 5),
            ("06_11", 6),
        ];
        for (cue, count) in counts {
            let text = line_text(cue).unwrap_or_else(|| panic!("{cue} is not in the script"));
            assert_eq!(sentence_ranges(text).len(), count, "{cue}");
        }
        assert_eq!(sentence("06_6", 2), "Commutativity exchanges the order of two moves.");
        assert_eq!(
            sentence("06_11", 5),
            "Now we can ask when a smaller collection of moves forms a group of its own."
        );
    }

    #[test]
    fn every_phrase_the_pictures_wait_for_is_in_its_recorded_sentence_in_order() {
        let chain: Vec<&str> = CHAIN.iter().filter_map(|link| link.phrase).collect();
        let cancelling: Vec<&str> = CANCELLING.iter().filter_map(|&(_, phrase)| phrase).collect();
        assert_eq!((chain.len(), cancelling.len()), (4, 2));
        let uses: [(&str, usize, &[&str]); 6] = [
            ("06_1", 3, &[RULE_PHRASE]),
            ("06_3", 1, &[RESULT_PHRASE]),
            ("06_5", 0, &REQUIREMENT_PHRASES),
            ("06_10", 1, &[RIGHT_UNDO_PHRASE]),
            ("06_10", 2, &chain),
            ("06_11", 2, &cancelling),
        ];
        for (cue, j, phrases) in uses {
            let text = sentence(cue, j);
            let mut last: Option<usize> = None;
            for phrase in phrases {
                let at = text
                    .find(phrase)
                    .unwrap_or_else(|| panic!("{phrase:?} not in {cue}/{j}: {text}"));
                assert!(
                    last.is_none_or(|before| at > before),
                    "{phrase:?} out of order in {cue}/{j}"
                );
                last = Some(at);
            }
        }
    }

    #[test]
    fn the_derivation_uses_each_assumption_once_and_otherwise_only_group_laws() {
        let assumptions: Vec<&str> = CHAIN
            .iter()
            .filter(|l| !l.law && !l.reason.is_empty())
            .map(|l| l.reason)
            .collect();
        assert_eq!(assumptions, ["ac=e", "ba=e"]);
        assert!(CHAIN.iter().filter(|l| l.law).all(|l| l.reason.starts_with(r"\text{")));
        assert_eq!((CHAIN[0].piece, CHAIN[CHAIN.len() - 1].piece), ("b", "=c"));
    }
}
