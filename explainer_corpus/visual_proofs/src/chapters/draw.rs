//! Video I's flat look, shared by every chapter (storyboard: "Two kinds of
//! question", "The card", "Color roles"): the colour roles, narration with
//! phrase timing, the "your turn" card with its countdown, the question card,
//! THE CARD (Shown / Assumed), the lower-third, the source line, the aside,
//! and the tile board (tiles, outlines, dimensions, a live count, formulas
//! that morph).
//!
//! Where the devices came from: line timing ([`say`], [`Spoken`]), the plays
//! ([`fade_in_all`] and its kin), the quiz card with its ring and its mallet
//! countdown, and the question card are ported from Video F's
//! `mental_math/src/chapters/draw.rs` and `sound.rs` (the newest house form),
//! changed where this video's storyboard differs: the question and the answer
//! are the caller's words (text or TeX, the answer built from `facts`), the
//! card is edged in `ink` and its answer lands in `hero` gold, and the
//! countdown walks down F major to this video's tonic. The card, the tile
//! board, the outline tracer and the dimension labels are new here.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`, unless their doc says "in the scene": bring them in
//! with an animation, or `add_to_scene`. Engine rules (`explainer_kit::kit`):
//! one flat fill per mobject, no outline on big filled shapes, small groups.
//! A piece of many tiles is ONE mobject (one closed path per tile), so a
//! layer of eleven tiles slides, fades and flashes as one thing.
//!
//! Every number on screen comes from [`fact`] (or is counted from `model`);
//! this file types none.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::f64::consts::{FRAC_PI_2, TAU};
use std::ops::Range;

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Facts, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::geometry::QuadPath;
use fmn::library::{SpanMapData, TransformMatchingTex, checkmark, exmark};
use fmn::prelude::*;

use crate::model::{QUIZZES, Quiz, Rect};

/// A cell of the board: its column and its row, from the lower left.
pub type Cell = (i64, i64);

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// -------------------------------------------------------------------- facts

thread_local! {
    static FACTS: OnceCell<Result<Facts, String>> = const { OnceCell::new() };
}

/// The value of fact `key` (`crate::facts::facts()`, computed once a thread).
pub fn fact(key: &str) -> fmn::Result<String> {
    FACTS.with(|f| match f.get_or_init(crate::facts::facts) {
        Ok(facts) => facts
            .get(key)
            .map(str::to_owned)
            .ok_or_else(|| fail(format!("no fact {key}"))),
        Err(e) => Err(fail(format!("facts: {e}"))),
    })
}

/// Fact `key` as a whole number.
pub fn fact_int(key: &str) -> fmn::Result<i64> {
    let value = fact(key)?;
    value
        .parse()
        .map_err(|_| fail(format!("fact {key} = {value:?} is not a whole number")))
}

/// Fact `key` as its whole numbers: a list (`1,3,5,7,9`) or a size (`5x5`).
pub fn fact_ints(key: &str) -> fmn::Result<Vec<i64>> {
    let value = fact(key)?;
    value
        .split([',', 'x'])
        .map(|part| {
            part.trim()
                .parse()
                .map_err(|_| fail(format!("fact {key} = {value:?} is not a list of whole numbers")))
        })
        .collect()
}

// ------------------------------------------------------------- colour roles

/// The five sizes (storyboard §3), `k` from 1: pink, copper, green, blue,
/// violet. The first layer, the one cube, a block with no short side is
/// `size(1)`. There is no sixth size colour: a size beyond the five is the
/// next size, which wears [`hero`] (storyboard §3), so `size(6)` is gold.
pub fn size(k: usize) -> Srgb {
    match k {
        1 => hex("#EA87B3"),
        2 => hex("#CD853F"),
        3 => Palette::green(),
        4 => Palette::blue(),
        5 => hex("#994DE5"),
        _ => hero(),
    }
}

/// A copy or a share where nothing is being sized (04, 07 to 11), `i` from
/// 0: blue, copper, pink. A fourth copy would be `dim`.
pub fn copy(i: usize) -> Srgb {
    match i {
        0 => size(4),
        1 => size(2),
        2 => size(1),
        _ => dim(),
    }
}

/// The piece moving now, an answer as it lands, the next size. Never a
/// resting fill.
pub fn hero() -> Srgb {
    Palette::hero()
}

/// What does not fit: the wrong guess, the bent edge, a failed tick.
pub fn warn() -> Srgb {
    Palette::red()
}

/// Pieces that only fill; a ghost of the last size; small print.
pub fn dim() -> Srgb {
    Palette::dim()
}

/// The faintest role: a ring's track, an outline of what is not there yet.
pub fn faint() -> Srgb {
    Palette::faint()
}

/// The quiz card, its ring, the card's text, outlines.
pub fn ink() -> Srgb {
    Palette::ink()
}

/// The fill behind cards.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// The background.
pub fn ground() -> Srgb {
    hex(explainer_kit::BACKGROUND)
}

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// A stroke at an opacity, and no fill.
pub fn faded_line(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// An `Srgb` as the `[f32; 3]` that `indicate` and its kin take (never call
/// `indicate` with no colour: its default yellow floods a fill).
pub fn rgb(c: Srgb) -> [f32; 3] {
    [c.r as f32, c.g as f32, c.b as f32]
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the words it illustrates. A
/// phrase's moment is its share of its sentence's characters (the voice
/// reads at a nearly even rate); the cue sheets of the narrated draft check
/// the result. (Video F's.)
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    /// Each sentence's first character and its start, seconds into the line.
    sentences: Vec<(usize, f64)>,
}

/// The character each sentence of `text` starts at.
fn sentence_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![0];
    for i in 0..chars.len().saturating_sub(2) {
        if matches!(chars[i], '.' | '?' | '!') && chars[i + 1] == ' ' {
            out.push(i + 2);
        }
    }
    out
}

/// Speak line `id` now (after the previous line has finished), as
/// `Kit::say`, and return its timing. Call it BEFORE the beat's animations.
pub fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Spoken> {
    kit.say(stage, id)?;
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?
        .to_owned();
    let words = text.split_whitespace().count() as f64;
    let len = kit
        .narrator
        .as_ref()
        .and_then(|n| n.length(id))
        .unwrap_or(words / SILENT_WPS + 0.4);
    let offsets = sentence_offsets(&text);
    let all = text.chars().count().max(1) as f64;
    let measured = kit.sentence_starts(id);
    let sentences = offsets
        .iter()
        .enumerate()
        .map(|(j, &at)| {
            let share = len * at as f64 / all;
            let start = if measured.len() == offsets.len() {
                measured[j]
            } else {
                share
            };
            (at, start)
        })
        .collect();
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
        sentences,
    })
}

impl Spoken {
    /// How many sentences the line has.
    pub fn sentences(&self) -> usize {
        self.sentences.len()
    }

    /// When sentence `j` (from 0) begins (scene seconds).
    pub fn sentence(&self, j: usize) -> fmn::Result<f64> {
        self.sentences
            .get(j)
            .map(|&(_, s)| self.start + s)
            .ok_or_else(|| fail(format!("line {} has no sentence {j}", self.id)))
    }

    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let byte = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let i = self.text[..byte].chars().count();
        let all = self.text.chars().count().max(1);
        let j = self.sentences.iter().rposition(|&(at, _)| at <= i).unwrap_or(0);
        let (from, t0) = self.sentences[j];
        let (to, t1) = self.sentences.get(j + 1).copied().unwrap_or((all, self.len));
        let share = (i - from) as f64 / (to - from).max(1) as f64;
        Ok(self.start + t0 + (t1 - t0) * share)
    }

    /// When the line ends.
    pub fn end(&self) -> f64 {
        self.start + self.len
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }

    /// Wait (if need be) until sentence `j` (from 0) begins.
    pub fn wait_sentence(&self, stage: &mut Stage<'_>, j: usize) -> fmn::Result<()> {
        let t = self.sentence(j)?;
        wait_until(stage, t)
    }

    /// Wait (if need be) until the line's last sentence begins (a quiz's
    /// question).
    pub fn wait_last(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.wait_sentence(stage, self.sentences.len() - 1)
    }

    /// Wait (if need be) until the line has been spoken.
    pub fn wait_end(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        wait_until(stage, self.end())
    }
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

/// Wait `seconds` (a beat of stillness after a reveal).
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

/// Let the current line finish, plus a breath (call it before every wipe).
pub fn hold(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)
}

/// A storyboard `hold`: the line finishes, then the music has the picture
/// to itself for the corpus's authored breath.
pub fn breath(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.music_breath(stage, explainer_kit::animatic::HOLD_S)
}

// --------------------------------------------------------- groups and plays

/// An empty parent in the arena holding mobjects already there. Animate the
/// group or its members, never both: animating a member takes it out.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, &[], mobs, rt)
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    swap(stage, mobs, &[], rt)
}

/// Fade `out` away and `ins` in, in one move.
pub fn swap(stage: &mut Stage<'_>, out: &[Mob], ins: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in ins {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Bring detached mobjects in one after another (a lagged reveal), the whole
/// taking `rt` seconds.
pub fn fade_in_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// A detached piece arriving: it fades in while sliding `by` (so it starts
/// `by` short of its place), `rt` seconds. Play it.
pub fn slide_in(stage: &mut Stage<'_>, mob: Mob, by: Vec3, rt: f64) -> fmn::Result<impl Animation + use<>> {
    Ok(fade_in(stage.arena_mut(), mob, by, 1.0)?.rt(rt))
}

/// A tracker to `value` at an even rate (a clock, a sweep), `rt` seconds.
pub fn tween_linear(tracker: Mob, value: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// Move a mobject so its centre is at `to`, scaling it by `scale` on the
/// way, `rt` seconds.
pub fn fly(mob: Mob, to: Vec3, scale: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    let a = mob.animate().set_anim_args(AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    })?;
    let a = if (scale - 1.0).abs() > 1e-9 { a.scale(scale)? } else { a };
    Ok(a.move_to(to, ORIGIN)?)
}

/// Slide a mobject by `by` (motion by placement), `rt` seconds.
pub fn slide(mob: Mob, by: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .shift(by)?)
}

/// A fill's colour and opacity, as an animation.
pub fn repaint(mob: Mob, color: Srgb, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_fill(Some(color), Some(opacity))?)
}

/// A mobject's opacity (fill and stroke), as an animation: dim what has
/// said its piece without taking it away.
pub fn fade_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

/// Flash mobjects already in the scene: each swells to `scale` and back,
/// tinted towards `color` on the way (always an explicit colour: the
/// engine's default yellow floods a fill). Never in the same play as the
/// mobject's own fade-in.
pub fn flash(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb, scale: f64, rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, scale, Some(rgb(color)))?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

// ----------------------------------------------------------- words on screen

/// One line of words: plain text (Computer Modern roman) or TeX. Fractions,
/// letters that stand for numbers and formulas are TeX.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Words<'a> {
    Text(&'a str),
    Tex(&'a str),
}

impl Words<'_> {
    /// The words, centred at `at`. Detached.
    pub fn set(self, kit: &Kit, stage: &mut Stage<'_>, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        match self {
            Words::Text(s) => kit.text(stage, s, size, color, at),
            Words::Tex(s) => kit.tex(stage, s, size, color, &[], at),
        }
    }

    /// The words, their left edge at `at`. Detached.
    pub fn set_left(self, kit: &Kit, stage: &mut Stage<'_>, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let m = self.set(kit, stage, size, color, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }
}

/// Shrink `mob` about its centre until it is at most `room` wide.
pub fn fit_width(stage: &mut Stage<'_>, mob: Mob, room: f64) {
    if stage.get_width(mob) > room {
        let at = stage.get_center(mob);
        stage.set_width(mob, room, false);
        stage.move_to(mob, at, ORIGIN);
    }
}

/// Shrink `mob` until it is at most `room` wide, keeping its left edge.
pub fn fit_width_left(stage: &mut Stage<'_>, mob: Mob, room: f64) {
    if stage.get_width(mob) > room {
        let at = stage.get_left(mob);
        stage.set_width(mob, room, false);
        stage.move_to(mob, at, LEFT);
    }
}

/// The glyphs of a typeset formula, in source order (a `Tex`'s children).
pub fn glyphs(stage: &Stage<'_>, tex: Mob) -> Vec<Mob> {
    explainer_kit::children(stage, tex)
}

/// The middle of the box round several mobjects (a few glyphs of a formula).
pub fn middle_of(stage: &Stage<'_>, mobs: &[Mob]) -> fmn::Result<Vec3> {
    if mobs.is_empty() {
        return Err(fail("no mobjects to find the middle of".into()));
    }
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for &m in mobs {
        let (l, r) = (stage.get_left(m)[0], stage.get_right(m)[0]);
        let (b, t) = (stage.get_bottom(m)[1], stage.get_top(m)[1]);
        lo = [lo[0].min(l), lo[1].min(b)];
        hi = [hi[0].max(r), hi[1].max(t)];
    }
    Ok(v((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0))
}

// ------------------------------------------------------------ the quiz card

/// Where a quiz card sits when it has the screen to itself.
pub const QUIZ_AT: Vec3 = [0.0, 0.3, 0.0];
/// The card's size at scale 1 (scene units).
pub const QUIZ_CARD: (f64, f64) = (7.2, 5.2);
/// The ring's radius at scale 1.
const RING_R: f64 = 0.72;
/// The question's type size at scale 1 (it shrinks to the card's width).
pub const QUIZ_PT: f64 = 76.0;
/// The answer's type size at scale 1.
pub const ANSWER_PT: f64 = 120.0;
/// The card's fixed words.
pub const YOUR_TURN: &str = "your turn";
pub const PAUSE_LINE: &str = "pause for more time";
/// How long the answer has the screen after the countdown's last second,
/// before the next line (the last note rings here).
pub const LAND_S: f64 = sound::LAND_S;

/// Where a quiz card sits and how big it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuizLook {
    /// The card's centre.
    pub at: Vec3,
    /// The card's size against [`QUIZ_CARD`] (1 when it has the screen to
    /// itself; about 0.72 beside a picture).
    pub scale: f64,
}

impl QuizLook {
    /// The card alone in the middle of the frame.
    pub const ALONE: Self = Self {
        at: QUIZ_AT,
        scale: 1.0,
    };
    /// The card in the right half, beside a picture in the left half.
    pub const BESIDE: Self = Self {
        at: [3.4, 0.55, 0.0],
        scale: 0.72,
    };

    /// The card's width and height.
    pub fn size(&self) -> (f64, f64) {
        (QUIZ_CARD.0 * self.scale, QUIZ_CARD.1 * self.scale)
    }
}

/// The quiz named `name` in `model::QUIZZES` (its seconds and its line).
pub fn quiz_named(name: &str) -> fmn::Result<&'static Quiz> {
    QUIZZES
        .iter()
        .find(|q| q.name == name)
        .ok_or_else(|| fail(format!("no quiz named {name} in model::QUIZZES")))
}

/// The "your turn" card (storyboard, "Two kinds of question"): the question
/// in ink, a ring that drains over the quiz's seconds with the seconds left
/// in its middle, "pause for more time" under it, and, at zero, the answer in
/// gold where the ring was.
pub struct QuizCard {
    pub quiz: &'static Quiz,
    pub look: QuizLook,
    pub back: Mob,
    pub label: Mob,
    pub question: Mob,
    /// The ring's faint track.
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub pause_line: Mob,
    /// The answer, where the ring was (detached until it lands).
    pub answer: Mob,
    /// Seconds left.
    pub left: Mob,
    /// 0 to 1: how much of the ring and its digit shows.
    pub shown: Mob,
    /// Carries the updater.
    pub anchor: Mob,
}

/// The ring: what is left of a circle of radius `r` at `c`, drained
/// clockwise from the top.
fn ring_points(c: Vec3, r: f64, share: f64) -> Vec<Vec3> {
    let share = share.clamp(0.0, 1.0);
    let steps = ((64.0 * share).ceil() as usize).max(1);
    (0..=steps)
        .map(|k| {
            // what is left runs counter-clockwise from the top
            let a = FRAC_PI_2 + TAU * share * k as f64 / steps as f64;
            v(c[0] + r * a.cos(), c[1] + r * a.sin())
        })
        .collect()
}

impl QuizCard {
    /// The card for the quiz named `name` (its seconds come from the model),
    /// asking `question` and holding `answer` (build it from `facts`).
    /// Detached.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        name: &str,
        question: Words<'_>,
        answer: Words<'_>,
        look: QuizLook,
    ) -> fmn::Result<Self> {
        let quiz = quiz_named(name)?;
        let (at, k) = (look.at, look.scale);
        let (w, h) = look.size();
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18 * k)
            .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.5, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let label = kit.text(
            stage,
            YOUR_TURN,
            (30.0 * k).max(22.0),
            dim(),
            v(at[0], at[1] + h / 2.0 - 0.45 * k),
        )?;
        let room = w - 1.0 * k;
        let question = question.set(kit, stage, QUIZ_PT * k, ink(), v(at[0], at[1] + 0.95 * k))?;
        fit_width(stage, question, room);
        let ring_at = Self::ring_at(look);
        let radius = RING_R * k;
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(radius)
                .arc_center(ring_at)
                .style(faded_line(faint(), 8.0 * k, 1.0))
                .build(),
        );
        let n = f64::from(quiz.seconds);
        let ring = stage.arena_mut().add(
            polyline(ring_points(ring_at, radius, 1.0))
                .style(faded_line(ink(), 9.0 * k, 0.0))
                .build(),
        );
        let mut digits = Vec::new();
        for second in 1..=quiz.seconds {
            let d = kit.text(stage, &second.to_string(), 56.0 * k, ink(), ring_at)?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let pause_line = kit.text(
            stage,
            PAUSE_LINE,
            (26.0 * k).max(20.0),
            dim(),
            v(ring_at[0], ring_at[1] - radius - 0.4 * k),
        )?;
        let answer = answer.set(kit, stage, ANSWER_PT * k, hero(), ring_at)?;
        fit_width(stage, answer, room);
        let left = stage.add_value_tracker(n);
        let shown = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let seen = digits.clone();
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = (
                    s.tracker_value(left).unwrap_or(n),
                    s.tracker_value(shown).unwrap_or(0.0),
                );
                if now == last {
                    return;
                }
                let rest = polyline(ring_points(ring_at, radius, now.0 / n)).build();
                let _ = s.set_points(ring, rest.points());
                s.set_stroke(ring, None, None, Some(now.1), None, false);
                let current = now.0.ceil() as usize;
                for (k, &d) in seen.iter().enumerate() {
                    let on = now.0 > 0.0 && current == k + 1;
                    s.set_opacity(d, if on { now.1 } else { 0.0 }, true);
                }
                last = now;
            },
            true,
        )?;
        Ok(Self {
            quiz,
            look,
            back,
            label,
            question,
            track,
            ring,
            digits,
            pause_line,
            answer,
            left,
            shown,
            anchor,
        })
    }

    fn ring_at(look: QuizLook) -> Vec3 {
        v(look.at[0], look.at[1] - 0.85 * look.scale)
    }

    /// Where the answer lands (and the ring sat).
    pub fn answer_at(&self) -> Vec3 {
        Self::ring_at(self.look)
    }

    /// Bring the card in with its question and its full ring, `rt` seconds.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        fade_in_all(
            stage,
            &[self.back, self.label, self.question, self.track, self.pause_line],
            rt,
        )?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        stage.play(tween(self.shown, 1.0, 0.25)?)?;
        Ok(())
    }

    /// The countdown: it opens once the question has been spoken, the
    /// narration silent (a listen window), one mallet note a second, the ring
    /// draining; at zero the ring goes and the answer lands in gold in its
    /// place, and has the screen for [`LAND_S`] before the next line.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = f64::from(self.quiz.seconds);
        let wav = sound::countdown(self.quiz.seconds)?;
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.sonify(stage, n + LAND_S, &label, &wav)?;
        let start = stage.scene().time().to_f64();
        stage.play(tween_linear(self.left, 0.0, n)?)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(tween(self.shown, 0.0, 0.25)?)?);
        for m in [self.track, self.pause_line] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.25))?);
        }
        let a = fade_in(stage.arena_mut(), self.answer, v(0.0, -0.12), 1.0)?;
        anims.push(stage.prepare(a.rt(0.25))?);
        stage.play_prepared(anims)?;
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.ring);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        wait_until(stage, start + n + LAND_S)
    }

    /// What the card still shows once the countdown is over: its back, its
    /// label, its question and its answer (fade these to clear it).
    pub fn mobs(&self) -> Vec<Mob> {
        vec![self.back, self.label, self.question, self.answer]
    }
}

/// A quiz, start to finish: say the quiz's line, bring the card in as the
/// question itself (the line's last sentence) begins, count down, land the
/// answer. The next beat works the answer on the same picture; clear the
/// card with `fade_out_all(stage, &card.mobs(), ..)`.
pub fn your_turn(
    kit: &Kit,
    stage: &mut Stage<'_>,
    name: &str,
    question: Words<'_>,
    answer: Words<'_>,
    look: QuizLook,
) -> fmn::Result<QuizCard> {
    let quiz = quiz_named(name)?;
    let line = say(kit, stage, quiz.line)?;
    let card = QuizCard::build(kit, stage, name, question, answer, look)?;
    line.wait_last(stage)?;
    card.show(stage, 0.5)?;
    card.count(kit, stage)?;
    Ok(card)
}

/// The countdown's sound (Video F's `sound.rs`, in this video's key): one
/// mallet note a second, walking down F major and ending A, G, F, so the
/// last three seconds always sound the same and the answer lands on the
/// tonic. Written at about the narration's level; the score re-levels it.
/// Not auditioned yet: the score bead tunes it.
mod sound {
    use std::path::PathBuf;

    use explainer_score::instr::{Instrument, Note, render};
    use explainer_score::{FS, Stereo};

    use super::fail;

    /// A sonification's integrated loudness as written (the narration's).
    const LEVEL_LUFS: f64 = -23.0;
    /// The fade every sound ends with, seconds.
    const TAIL_S: f64 = 0.25;
    /// F major from the tonic up a ninth, low to high: the only notes a
    /// countdown plays. A countdown of n seconds takes the n + 1 lowest and
    /// walks down them, so the last three are always A, G, F; the longest
    /// quiz is 8 s, which uses all nine.
    const SCALE: [f64; 9] = [77.0, 79.0, 81.0, 82.0, 84.0, 86.0, 88.0, 89.0, 91.0];
    /// See [`super::LAND_S`].
    pub const LAND_S: f64 = 1.0;

    fn frames(seconds: f64) -> usize {
        (seconds * FS).round() as usize
    }

    /// The notes of a countdown of `seconds`, one a second from its first
    /// second to the moment the answer lands.
    pub fn countdown_pitches(seconds: u32) -> Vec<f64> {
        let n = (seconds as usize + 1).min(SCALE.len());
        SCALE[..n].iter().rev().copied().collect()
    }

    /// A quiz's countdown as a WAV: one mallet note a second, and the last
    /// one as the answer lands.
    pub fn countdown(seconds: u32) -> fmn::Result<PathBuf> {
        let total = f64::from(seconds) + LAND_S;
        let mut out = Stereo::silence(frames(total + 4.0));
        let pitches = countdown_pitches(seconds);
        let last = pitches.len() - 1;
        for (k, &pitch) in pitches.iter().enumerate() {
            let velocity = if k == last { 0.8 } else { 0.55 };
            let note = Note::new(pitch, velocity, 0.6, k as u64 + 1);
            out.mix_in(&render(Instrument::Marimba, &note), frames(k as f64), 1.0);
        }
        // cut to length, fade the tail to an exact zero, set the level
        let n = frames(total);
        out.l.resize(n, 0.0);
        out.r.resize(n, 0.0);
        let tail = frames(TAIL_S).min(n);
        for k in 0..tail {
            let g = 0.5 - 0.5 * (std::f64::consts::PI * k as f64 / tail as f64).cos();
            out.l[n - 1 - k] *= g;
            out.r[n - 1 - k] *= g;
        }
        let lufs = explainer_score::meter::integrated(&out);
        if lufs.is_finite() {
            out.scale(10f64.powf((LEVEL_LUFS - lufs) / 20.0));
        }
        let label = format!("proofs_countdown_{seconds}");
        explainer_kit::sound_file(&label, &out).map_err(|e| fail(format!("sonification {label}: {e}")))
    }
}

// -------------------------------------------------------- the question card

/// How long a question to carry holds before its answer.
pub const QUESTION_HOLD_S: f64 = 2.0;

/// A question to carry (01_2, 06_1, 08_1, 09_1, 10_3, 15_2): its words on a
/// card with no ring, centred at `at`, wrapped to about 26 characters a
/// line. Detached, one group.
pub fn question_card(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3) -> fmn::Result<Mob> {
    let lines = wrap(words, 26, 3);
    let line_h = 0.6;
    let top = at[1] + line_h * (lines.len() as f64 - 1.0) / 2.0;
    let mut texts = Vec::new();
    let mut widest: f64 = 0.0;
    for (i, l) in lines.iter().enumerate() {
        let m = kit.text(stage, l, 40.0, ink(), v(at[0], top - line_h * i as f64))?;
        widest = widest.max(stage.get_width(m));
        texts.push(m);
    }
    let back = Rectangle::new()
        .width(widest + 0.9)
        .height(line_h * lines.len() as f64 + 0.5)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.5, 1.0))
        .build()?
        .moved_to(at);
    let mut all = vec![stage.arena_mut().add(back)];
    all.extend(texts);
    group(stage, &all)
}

/// The pause after a question to carry: let the question finish, mark the
/// moment for the score, and hold the picture [`QUESTION_HOLD_S`].
pub fn question_pause(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    pause(stage, QUESTION_HOLD_S)
}

// ------------------------------------------------------------------ the card

/// The card's labels.
pub const SHOWN: &str = "Shown";
pub const ASSUMED: &str = "Assumed";
pub const COUNTED: &str = "Counted, not drawn";

/// The card's top edge, its width, and its type size, at scale 1: a band
/// across the foot of the frame, the same in every chapter. Keep the picture
/// above `CARD_TOP` when the card comes. Two rows end at y = −3.2, three at
/// −3.76 (the lower-third's band is in the way of a third row: fade it).
pub const CARD_TOP: f64 = -1.6;
pub const CARD_W: f64 = 12.8;
pub const CARD_PT: f64 = 30.0;
const CARD_REASON_PT: f64 = 26.0;
const CARD_PAD: f64 = 0.24;
const CARD_ROW: f64 = 0.56;
/// From the labels' left edge to the statements' left edge.
const CARD_LABEL_COL: f64 = 1.75;
/// The room an Assumed line keeps on its right for its mark and its reason.
const CARD_MARK_ROOM: f64 = 3.6;

/// One line of the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardRow<'a> {
    /// The exact statement, with its range.
    Shown(Words<'a>),
    /// What the picture took for granted: it can be ticked or crossed.
    Assumed(Words<'a>),
    /// Chapter 12's third line: what was counted and never drawn.
    Counted(Words<'a>),
}

/// Where the card sits: every chapter uses [`CardLook::HOME`]; chapter 15
/// reads two cards back, enlarged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLook {
    /// The x of the card's middle and the y of its top edge.
    pub x: f64,
    pub top: f64,
    pub scale: f64,
}

impl CardLook {
    pub const HOME: Self = Self {
        x: 0.0,
        top: CARD_TOP,
        scale: 1.0,
    };
}

/// One built line of the card.
pub struct CardLine {
    pub label: Mob,
    pub words: Mob,
    /// The tick or the cross, once the line is checked.
    pub mark: Option<Mob>,
    /// The short reason beside the mark.
    pub reason: Option<Mob>,
    assumed: bool,
    y: f64,
}

/// THE CARD (storyboard header): **Shown** and **Assumed**, always in the
/// same place. Build it with its lines, show it, then tick (or cross) each
/// Assumed line as the narration checks it.
pub struct ProofCard {
    pub back: Mob,
    pub lines: Vec<CardLine>,
    pub look: CardLook,
    back_shown: bool,
    /// The right edge of the card's writing.
    right: f64,
}

impl ProofCard {
    /// The card with `rows`, top to bottom, at its home. Detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, rows: &[CardRow<'_>]) -> fmn::Result<Self> {
        Self::build_at(kit, stage, rows, CardLook::HOME)
    }

    /// As [`ProofCard::build`], somewhere else or at another size.
    pub fn build_at(kit: &Kit, stage: &mut Stage<'_>, rows: &[CardRow<'_>], look: CardLook) -> fmn::Result<Self> {
        if rows.is_empty() {
            return Err(fail("the card needs at least one line".into()));
        }
        let k = look.scale;
        let (w, h) = (CARD_W * k, (2.0 * CARD_PAD + CARD_ROW * rows.len() as f64) * k);
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.14 * k)
            .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.0, 1.0))
            .build()?
            .moved_to(v(look.x, look.top - h / 2.0));
        let back = stage.arena_mut().add(back);
        let left = look.x - w / 2.0 + 0.4 * k;
        let right = look.x + w / 2.0 - 0.4 * k;
        let mut lines = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let y = look.top - (CARD_PAD + CARD_ROW * (i as f64 + 0.5)) * k;
            let (name, words, assumed) = match *row {
                CardRow::Shown(w) => (SHOWN, w, false),
                CardRow::Assumed(w) => (ASSUMED, w, true),
                CardRow::Counted(w) => (COUNTED, w, false),
            };
            let label = kit.text_styled(stage, name, CARD_PT * k, ink(), v(left, y), true, false)?;
            stage.move_to(label, v(left, y), LEFT);
            let from = left + (CARD_LABEL_COL * k).max(stage.get_width(label) + 0.35 * k);
            let words = words.set_left(kit, stage, CARD_PT * k, ink(), v(from, y))?;
            let room = right - from - if assumed { CARD_MARK_ROOM * k } else { 0.0 };
            fit_width_left(stage, words, room);
            lines.push(CardLine {
                label,
                words,
                mark: None,
                reason: None,
                assumed,
                y,
            });
        }
        Ok(Self {
            back,
            lines,
            look,
            back_shown: false,
            right,
        })
    }

    /// Bring the whole card in, `rt` seconds.
    pub fn show(&mut self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        self.show_lines(stage, 0..self.lines.len(), rt)
    }

    /// Bring in some of the card's lines (and its back, the first time), so
    /// each line can arrive with its words in the narration.
    pub fn show_lines(&mut self, stage: &mut Stage<'_>, lines: Range<usize>, rt: f64) -> fmn::Result<()> {
        let mut ins = Vec::new();
        if !self.back_shown {
            ins.push(self.back);
            self.back_shown = true;
        }
        for line in self.lines.get(lines).unwrap_or_default() {
            ins.extend([line.label, line.words]);
        }
        fade_in_all(stage, &ins, rt)
    }

    /// Tick Assumed line `line` (counted from the card's top, from 0): a tick
    /// in ink (never green: green is a size) and the short `reason` beside it.
    pub fn tick(&mut self, kit: &Kit, stage: &mut Stage<'_>, line: usize, reason: &str) -> fmn::Result<()> {
        self.mark(kit, stage, line, reason, true)
    }

    /// Cross Assumed line `line` in red, with the short `reason` beside it
    /// (chapter 14: the card fails).
    pub fn cross(&mut self, kit: &Kit, stage: &mut Stage<'_>, line: usize, reason: &str) -> fmn::Result<()> {
        self.mark(kit, stage, line, reason, false)
    }

    fn mark(&mut self, kit: &Kit, stage: &mut Stage<'_>, line: usize, reason: &str, ok: bool) -> fmn::Result<()> {
        let k = self.look.scale;
        let right = self.right;
        let Some(row) = self.lines.get_mut(line) else {
            return Err(fail(format!("the card has no line {line}")));
        };
        if !row.assumed || row.mark.is_some() {
            return Err(fail(format!("the card's line {line} is not an unchecked Assumed line")));
        }
        let color = if ok { ink() } else { warn() };
        let x = stage.get_right(row.words)[0] + 0.45 * k;
        let shape = if ok { checkmark(color) } else { exmark(color) };
        let side = 0.3 * k;
        let mark = stage
            .arena_mut()
            .add(shape.with_height(side, false).moved_to(v(x + side / 2.0, row.y)));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = fmn::animation::grow_from_center(stage.arena_mut(), mark, None)?;
        anims.push(stage.prepare(a.rt(0.4))?);
        row.mark = Some(mark);
        if !reason.is_empty() {
            let from = x + side + 0.22 * k;
            let color = if ok { dim() } else { warn() };
            let words = kit.text_left(stage, reason, CARD_REASON_PT * k, color, v(from, row.y))?;
            fit_width_left(stage, words, right - from);
            let a = fade_in(stage.arena_mut(), words, v(0.15, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
            row.reason = Some(words);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// Everything of the card that is on screen (fade these to clear it).
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back];
        for line in &self.lines {
            out.extend([line.label, line.words]);
            out.extend(line.mark);
            out.extend(line.reason);
        }
        out
    }
}

/// The card in one call: build it with `rows` at its home and bring it in.
/// Tick with [`ProofCard::tick`].
pub fn the_card(kit: &Kit, stage: &mut Stage<'_>, rows: &[CardRow<'_>]) -> fmn::Result<ProofCard> {
    let mut card = ProofCard::build(kit, stage, rows)?;
    card.show(stage, 0.6)?;
    Ok(card)
}

// --------------------------------------------- lower-third, source and aside

/// The lower-third band at the foot of the frame (the kit's
/// `props::lower_third`): a `label` in gold ("gnomon:", "precisely:") and
/// the wrapped `text` beside it. It shares the foot of the frame with the
/// card: fade one before the other comes. Detached, one group.
pub fn lower_third(kit: &Kit, stage: &mut Stage<'_>, label: &str, text: &str) -> fmn::Result<Mob> {
    explainer_kit::props::lower_third(kit, stage, label, text)
}

/// A source line: a citation in small dim type at the bottom left. With
/// `raised` it sits just above the lower-third's band instead of in it.
/// Detached.
pub fn source_line(kit: &Kit, stage: &mut Stage<'_>, words: &str, raised: bool) -> fmn::Result<Mob> {
    let y = if raised { -2.92 } else { -3.6 };
    kit.text_left(stage, words, 22.0, dim(), v(-FRAME_X_RADIUS + 0.45, y))
}

/// Where an aside sits unless the chapter says otherwise: bottom right.
pub const ASIDE_AT: Vec3 = [4.3, -3.05, 0.0];

/// A small tagged aside ("for programmers"): the `tag` in dim type over the
/// `words` in ink, wrapped to about 34 characters, on a small card centred
/// at `at`. Detached, one group.
pub fn aside(kit: &Kit, stage: &mut Stage<'_>, tag: &str, words: &str, at: Vec3) -> fmn::Result<Mob> {
    let lines = wrap(words, 34, 3);
    let line_h = 0.36;
    let rows = lines.len() as f64 + 1.0;
    let top = at[1] + line_h * (rows - 1.0) / 2.0;
    let head = kit.text_styled(stage, tag, 22.0, dim(), v(at[0], top), true, false)?;
    let mut widest = stage.get_width(head);
    let mut texts = vec![head];
    for (i, l) in lines.iter().enumerate() {
        let m = kit.text(stage, l, 24.0, ink(), v(at[0], top - line_h * (i as f64 + 1.0)))?;
        widest = widest.max(stage.get_width(m));
        texts.push(m);
    }
    let back = Rectangle::new()
        .width(widest + 0.5)
        .height(line_h * rows + 0.3)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.92).stroke(faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut all = vec![stage.arena_mut().add(back)];
    all.extend(texts);
    group(stage, &all)
}

// ---------------------------------------------------------------- the board

/// The share of a cell left dark between two tiles (half on each side).
pub const TILE_GAP: f64 = 0.07;

/// Which side of a rectangle a label or a dimension stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Below,
    Above,
}

impl Side {
    /// The direction pointing away from the rectangle on this side.
    pub fn out(self) -> Vec3 {
        match self {
            Side::Left => LEFT,
            Side::Right => RIGHT,
            Side::Below => DOWN,
            Side::Above => UP,
        }
    }
}

/// A side's length with its brace: see [`Board::dimension`].
#[derive(Clone, Copy, Debug)]
pub struct Dimension {
    pub brace: Mob,
    pub label: Mob,
}

impl Dimension {
    pub fn mobs(&self) -> [Mob; 2] {
        [self.brace, self.label]
    }
}

/// The tile board: a grid of unit cells with its origin (the lower-left
/// corner of cell (0, 0)) at a scene point. Never in perspective.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub origin: Vec3,
    /// Scene units per cell.
    pub cell: f64,
}

/// The corners of the loops round a set of cells, each loop walked with the
/// cells on its left, corners on a straight run left out. A set with a hole
/// has a second loop round the hole; two pieces have a loop each.
pub fn boundary(cells: &[Cell]) -> Vec<Vec<Cell>> {
    let set: HashSet<Cell> = cells.iter().copied().collect();
    let mut sorted: Vec<Cell> = set.iter().copied().collect();
    sorted.sort_unstable();
    let mut next: HashMap<Cell, Vec<Cell>> = HashMap::new();
    for &(x, y) in &sorted {
        let sides = [
            ((x, y - 1), (x, y), (x + 1, y)),
            ((x + 1, y), (x + 1, y), (x + 1, y + 1)),
            ((x, y + 1), (x + 1, y + 1), (x, y + 1)),
            ((x - 1, y), (x, y + 1), (x, y)),
        ];
        for (neighbour, from, to) in sides {
            if !set.contains(&neighbour) {
                next.entry(from).or_default().push(to);
            }
        }
    }
    let mut starts: Vec<Cell> = next.keys().copied().collect();
    starts.sort_unstable();
    let mut loops = Vec::new();
    for start in starts {
        while let Some(first) = next.get_mut(&start).and_then(Vec::pop) {
            let mut corners = vec![start];
            let mut at = first;
            while at != start {
                corners.push(at);
                match next.get_mut(&at).and_then(Vec::pop) {
                    Some(to) => at = to,
                    None => break,
                }
            }
            let n = corners.len();
            let turns: Vec<Cell> = (0..n)
                .filter(|&i| {
                    let (a, b, c) = (corners[(i + n - 1) % n], corners[i], corners[(i + 1) % n]);
                    (b.0 - a.0) * (c.1 - b.1) != (b.1 - a.1) * (c.0 - b.0)
                })
                .map(|i| corners[i])
                .collect();
            loops.push(turns);
        }
    }
    loops
}

impl Board {
    pub fn new(origin: Vec3, cell: f64) -> Self {
        Self { origin, cell }
    }

    /// The largest board at `origin` on which `wide` by `tall` cells fit in
    /// `room` (scene units, wide and tall).
    pub fn fit(origin: Vec3, wide: i64, tall: i64, room: (f64, f64)) -> Self {
        Self {
            origin,
            cell: (room.0 / wide as f64).min(room.1 / tall as f64),
        }
    }

    /// The scene point of grid point (`x`, `y`), in cells (fractions allowed).
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        v(self.origin[0] + x * self.cell, self.origin[1] + y * self.cell)
    }

    /// The middle of a cell.
    pub fn centre(&self, (x, y): Cell) -> Vec3 {
        self.p(x as f64 + 0.5, y as f64 + 0.5)
    }

    /// The middle of a rectangle.
    pub fn rect_centre(&self, r: Rect) -> Vec3 {
        self.p(r.x as f64 + r.w as f64 / 2.0, r.y as f64 + r.h as f64 / 2.0)
    }

    /// The scene displacement of (`dx`, `dy`) cells (for `slide`, `slide_in`).
    pub fn step(&self, dx: f64, dy: f64) -> Vec3 {
        v(dx * self.cell, dy * self.cell)
    }

    fn tile_path(&self, cells: &[Cell]) -> fmn::Result<QuadPath> {
        if cells.is_empty() {
            return Err(fail("a piece needs at least one tile".into()));
        }
        let g = self.cell * TILE_GAP / 2.0;
        let mut path = QuadPath::new();
        for &(x, y) in cells {
            let (a, b) = (self.p(x as f64, y as f64), self.p(x as f64 + 1.0, y as f64 + 1.0));
            let quad = [
                v(a[0] + g, a[1] + g),
                v(b[0] - g, a[1] + g),
                v(b[0] - g, b[1] - g),
                v(a[0] + g, b[1] - g),
            ];
            path.add_subpath(Polygon::new(quad).build().points())
                .map_err(|e| fail(format!("a tile's path: {e:?}")))?;
        }
        Ok(path)
    }

    /// A piece: `cells` as unit tiles of one colour with thin dark gaps, as
    /// ONE mobject (one closed path a tile). Detached.
    pub fn tiles(&self, stage: &mut Stage<'_>, cells: &[Cell], color: Srgb) -> fmn::Result<Mob> {
        self.tiles_styled(stage, cells, flat(color, 1.0))
    }

    /// As [`Board::tiles`], in any style (a ghost at an opacity).
    pub fn tiles_styled(&self, stage: &mut Stage<'_>, cells: &[Cell], style: Style) -> fmn::Result<Mob> {
        let path = self.tile_path(cells)?;
        Ok(stage.arena_mut().add(VMobject::from_path(&path).with_style(style)))
    }

    /// A rectangle of the model as tiles of one colour, one mobject. Detached.
    pub fn rect_tiles(&self, stage: &mut Stage<'_>, r: Rect, color: Srgb) -> fmn::Result<Mob> {
        let cells: Vec<Cell> = r.cells().collect();
        self.tiles(stage, &cells, color)
    }

    /// A rectangle of the model as one solid piece with no tiles ruled on it
    /// (a slab seen whole), inset by the tiles' gap so neighbours stay apart.
    /// Detached.
    pub fn slab(&self, stage: &mut Stage<'_>, r: Rect, color: Srgb) -> Mob {
        let g = self.cell * TILE_GAP / 2.0;
        let (a, b) = (
            self.p(r.x as f64, r.y as f64),
            self.p((r.x + r.w) as f64, (r.y + r.h) as f64),
        );
        let quad = [
            v(a[0] + g, a[1] + g),
            v(b[0] - g, a[1] + g),
            v(b[0] - g, b[1] - g),
            v(a[0] + g, b[1] - g),
        ];
        stage
            .arena_mut()
            .add(Polygon::new(quad).style(flat(color, 1.0)).build())
    }

    /// The outline of a region (the edge of `cells` taken together), as one
    /// mobject in `style` (a stroke: [`line_style`], [`faded_line`]).
    /// Detached.
    pub fn outline(&self, stage: &mut Stage<'_>, cells: &[Cell], style: Style) -> fmn::Result<Mob> {
        let loops = boundary(cells);
        let corners = |l: &[Cell]| -> Vec<Vec3> { l.iter().map(|&(x, y)| self.p(x as f64, y as f64)).collect() };
        let shape = match loops.as_slice() {
            [] => return Err(fail("an outline needs at least one cell".into())),
            [one] => Polygon::new(corners(one)).style(style).build(),
            many => {
                let mut path = QuadPath::new();
                for l in many {
                    path.add_subpath(Polygon::new(corners(l)).build().points())
                        .map_err(|e| fail(format!("an outline's path: {e:?}")))?;
                }
                VMobject::from_path(&path).with_style(style)
            }
        };
        Ok(stage.arena_mut().add(shape))
    }

    /// The outline of a rectangle of the model. Detached.
    pub fn rect_outline(&self, stage: &mut Stage<'_>, r: Rect, style: Style) -> fmn::Result<Mob> {
        let cells: Vec<Cell> = r.cells().collect();
        self.outline(stage, &cells, style)
    }

    /// Words beside rectangle `r`, centred along its `side`. Detached.
    #[allow(clippy::too_many_arguments)]
    pub fn label(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        words: Words<'_>,
        r: Rect,
        side: Side,
        size: f64,
        color: Srgb,
    ) -> fmn::Result<Mob> {
        let (mid, out) = self.side_of(r, side);
        let at = v(mid[0] + 0.2 * out[0], mid[1] + 0.2 * out[1]);
        let m = words.set(kit, stage, size, color, at)?;
        stage.move_to(m, at, [-out[0], -out[1], 0.0]);
        Ok(m)
    }

    /// A dimension: a brace along the `side` of rectangle `r` with `words`
    /// (the side's length, from `facts`) at its tip. Detached.
    #[allow(clippy::too_many_arguments)]
    pub fn dimension(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        r: Rect,
        side: Side,
        words: Words<'_>,
        size: f64,
        color: Srgb,
    ) -> fmn::Result<Dimension> {
        let (x0, y0) = (r.x as f64, r.y as f64);
        let (x1, y1) = (x0 + r.w as f64, y0 + r.h as f64);
        // the brace is built along a flat line and then turned with it, so
        // its direction is given in the flat line's frame
        let (a, b, flat_dir) = match side {
            Side::Below => (self.p(x0, y0), self.p(x1, y0), DOWN),
            Side::Above => (self.p(x0, y1), self.p(x1, y1), UP),
            Side::Left => (self.p(x0, y0), self.p(x0, y1), UP),
            Side::Right => (self.p(x1, y0), self.p(x1, y1), DOWN),
        };
        let buff = 0.1;
        let brace = fmn::library::line_brace(a, b, flat_dir).buff(buff).color(color);
        let depth = brace.depth();
        let brace = stage.arena_mut().add(brace.build());
        let (mid, out) = self.side_of(r, side);
        let reach = buff + depth + 0.14;
        let at = v(mid[0] + reach * out[0], mid[1] + reach * out[1]);
        let label = words.set(kit, stage, size, color, at)?;
        stage.move_to(label, at, [-out[0], -out[1], 0.0]);
        Ok(Dimension { brace, label })
    }

    /// The middle of a side of `r`, and the direction away from it.
    fn side_of(&self, r: Rect, side: Side) -> (Vec3, Vec3) {
        let (x0, y0) = (r.x as f64, r.y as f64);
        let (x1, y1) = (x0 + r.w as f64, y0 + r.h as f64);
        let mid = match side {
            Side::Left => self.p(x0, (y0 + y1) / 2.0),
            Side::Right => self.p(x1, (y0 + y1) / 2.0),
            Side::Below => self.p((x0 + x1) / 2.0, y0),
            Side::Above => self.p((x0 + x1) / 2.0, y1),
        };
        (mid, side.out())
    }
}

// ------------------------------------------------------- counts and formulas

/// A live whole number driven by `tracker` (rounded, so a tween never shows
/// a truncated count), centred at `at`. In the scene at once; clear its
/// updaters before fading it (`fade_out_all` does).
pub fn live_counter(
    kit: &Kit,
    stage: &mut Stage<'_>,
    tracker: Mob,
    size: f64,
    color: Srgb,
    at: Vec3,
) -> fmn::Result<Mob> {
    kit.live_count(stage, tracker, size, color, at)
}

/// A formula in TeX with what `morph` needs to pair its glyphs.
pub struct Formula {
    pub mob: Mob,
    pub spans: SpanMapData,
}

/// Typeset `src` centred at `at`. Detached.
pub fn formula(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Formula> {
    let (mob, spans) = kit.tex_spans(stage, src, size, color, &[], at)?;
    Ok(Formula { mob, spans })
}

/// Write a detached formula onto the screen, `rt` seconds.
pub fn write_formula(stage: &mut Stage<'_>, f: &Formula, rt: f64) -> fmn::Result<()> {
    let a = stage.prepare(write(stage.arena(), f.mob).rt(rt))?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

/// Morph formula `from` (on screen) into `to` (detached, already placed):
/// the pieces of TeX source the two share glide, the rest fades. Afterwards
/// `to.mob` is the one on screen.
pub fn morph(stage: &mut Stage<'_>, from: &Formula, to: &Formula, rt: f64) -> fmn::Result<()> {
    let m = TransformMatchingTex::new(stage.arena_mut(), from.mob, to.mob, &from.spans, &to.spans)?;
    let a = stage.prepare(m.rt(rt))?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CIE L*a*b* (D65) of an sRGB colour.
    fn lab(c: Srgb) -> [f64; 3] {
        let lin = |u: f64| {
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
        let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
        let z = 0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b;
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (24389.0 / 27.0 * t + 16.0) / 116.0
            }
        };
        let (fx, fy, fz) = (f(x / 0.95047), f(y), f(z / 1.08883));
        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }

    /// The CIEDE2000 colour difference of two L*a*b* colours (Sharma, Wu and
    /// Dalal 2005, with unit weights).
    fn ciede2000(p: [f64; 3], q: [f64; 3]) -> f64 {
        let ([l1, a1, b1], [l2, a2, b2]) = (p, q);
        let pow7 = |c: f64| c.powi(7) / (c.powi(7) + 25f64.powi(7));
        let c_bar = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
        let g = 0.5 * (1.0 - pow7(c_bar).sqrt());
        let (a1, a2) = ((1.0 + g) * a1, (1.0 + g) * a2);
        let (c1, c2) = (a1.hypot(b1), a2.hypot(b2));
        let hue = |b: f64, a: f64| {
            if a == 0.0 && b == 0.0 {
                0.0
            } else {
                b.atan2(a).to_degrees().rem_euclid(360.0)
            }
        };
        let (h1, h2) = (hue(b1, a1), hue(b2, a2));
        let (dl, dc) = (l2 - l1, c2 - c1);
        let grey = c1 * c2 == 0.0;
        let dh = if grey {
            0.0
        } else if (h2 - h1).abs() <= 180.0 {
            h2 - h1
        } else if h2 - h1 > 180.0 {
            h2 - h1 - 360.0
        } else {
            h2 - h1 + 360.0
        };
        let dh = 2.0 * (c1 * c2).sqrt() * (dh.to_radians() / 2.0).sin();
        let (l_bar, c_bar) = ((l1 + l2) / 2.0, (c1 + c2) / 2.0);
        let h_bar = if grey {
            h1 + h2
        } else if (h1 - h2).abs() <= 180.0 {
            (h1 + h2) / 2.0
        } else if h1 + h2 < 360.0 {
            (h1 + h2 + 360.0) / 2.0
        } else {
            (h1 + h2 - 360.0) / 2.0
        };
        let cos = |deg: f64| deg.to_radians().cos();
        let t = 1.0 - 0.17 * cos(h_bar - 30.0) + 0.24 * cos(2.0 * h_bar) + 0.32 * cos(3.0 * h_bar + 6.0)
            - 0.20 * cos(4.0 * h_bar - 63.0);
        let turn = 30.0 * (-((h_bar - 275.0) / 25.0).powi(2)).exp();
        let rc = 2.0 * pow7(c_bar).sqrt();
        let sl = 1.0 + 0.015 * (l_bar - 50.0).powi(2) / (20.0 + (l_bar - 50.0).powi(2)).sqrt();
        let (sc, sh) = (1.0 + 0.045 * c_bar, 1.0 + 0.015 * c_bar * t);
        let rt = -(2.0 * turn).to_radians().sin() * rc;
        ((dl / sl).powi(2) + (dc / sc).powi(2) + (dh / sh).powi(2) + rt * (dc / sc) * (dh / sh)).sqrt()
    }

    #[test]
    fn ciede2000_matches_the_published_test_pairs() {
        // Sharma, Wu and Dalal's table 1, pairs 1, 4 and 17
        for (p, q, want) in [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0),
            ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
        ] {
            assert!((ciede2000(p, q) - want).abs() < 1e-4, "{p:?} {q:?}");
        }
    }

    /// Storyboard §3: every pair among the five sizes, hero, warn, dim and
    /// ink is at least CIEDE2000 20 apart, and the least is 21.6, blue
    /// against dim.
    #[test]
    fn the_colour_roles_are_the_storyboard_s_and_stay_apart() {
        let roles = [
            ("pink", size(1), "#EA87B3"),
            ("copper", size(2), "#CD853F"),
            ("green", size(3), "#83C167"),
            ("blue", size(4), "#58C4DD"),
            ("violet", size(5), "#994DE5"),
            ("hero", hero(), "#FFD866"),
            ("warn", warn(), "#FC6255"),
            ("dim", dim(), "#8A93A3"),
            ("ink", ink(), "#ECEFF4"),
        ];
        for (name, role, literal) in roles {
            assert_eq!(role, hex(literal), "{name}");
        }
        assert_eq!(faint(), hex("#3B4352"));
        assert_eq!([copy(0), copy(1), copy(2)], [size(4), size(2), size(1)]);
        let mut least = (f64::INFINITY, "", "");
        for (i, &(a, ca, _)) in roles.iter().enumerate() {
            for &(b, cb, _) in &roles[i + 1..] {
                let d = ciede2000(lab(ca), lab(cb));
                assert!(d >= 20.0, "{a} and {b} are only {d:.1} apart");
                if d < least.0 {
                    least = (d, a, b);
                }
            }
        }
        assert_eq!((least.1, least.2), ("blue", "dim"));
        assert!((least.0 - 21.6).abs() < 0.05, "the least distance is {:.2}", least.0);
    }

    /// There is no sixth size colour and no fourth copy: the next size is
    /// the hero's gold, and an extra copy is dim.
    #[test]
    fn the_sizes_past_the_ramp_are_the_next_size() {
        assert_eq!(size(6), hero());
        assert_eq!(size(0), hero());
        assert_eq!(copy(3), dim());
    }

    #[test]
    fn a_region_s_boundary_is_its_corners_in_order() {
        // one cell: its four corners, the cell on the left of the walk
        assert_eq!(boundary(&[(0, 0)]), [vec![(0, 0), (1, 0), (1, 1), (0, 1)]]);
        // a layer of the model is one loop of six corners, however long
        for k in 2..=9 {
            let loops = boundary(&crate::model::layers::layer(k));
            assert_eq!(loops.len(), 1, "layer {k}");
            assert_eq!(loops[0].len(), 6, "layer {k}");
        }
        // a rectangle has four corners; a ring has a second loop round its hole
        let r = Rect::new(2, 1, 4, 3);
        assert_eq!(boundary(&r.cells().collect::<Vec<_>>())[0].len(), 4);
        let ring: Vec<Cell> = Rect::new(0, 0, 3, 3).cells().filter(|&c| c != (1, 1)).collect();
        assert_eq!(boundary(&ring).len(), 2);
        // two pieces apart: a loop each
        assert_eq!(boundary(&[(0, 0), (5, 5)]).len(), 2);
    }

    #[test]
    fn sentences_start_after_their_stops() {
        assert_eq!(sentence_offsets("One. Two? Three."), [0, 5, 10]);
        assert_eq!(sentence_offsets("No stop"), [0]);
    }

    #[test]
    fn every_quiz_has_a_name_and_a_countdown_that_lands_on_the_tonic() {
        for q in QUIZZES {
            assert_eq!(quiz_named(q.name).unwrap().line, q.line);
            let p = sound::countdown_pitches(q.seconds);
            assert_eq!(
                p.len(),
                q.seconds as usize + 1,
                "{}: a note a second and one more",
                q.name
            );
            assert_eq!(p[p.len() - 3..], [81.0, 79.0, 77.0], "{}: A, G, F", q.name);
            assert!(p.windows(2).all(|w| w[0] > w[1]), "{}: it only walks down", q.name);
        }
        assert!(quiz_named("no such quiz").is_err());
    }

    #[test]
    fn facts_are_read_by_key() {
        assert_eq!(fact_ints("F-O.layers").unwrap(), [1, 3, 5, 7, 9]);
        assert_eq!(fact_ints("F-O.square").unwrap(), [5, 5]);
        assert_eq!(fact_int("F-Q.next_layer.answer").unwrap(), 11);
        assert!(fact("F-no.such").is_err());
    }
}
