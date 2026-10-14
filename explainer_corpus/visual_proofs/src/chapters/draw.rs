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

/// A vertex list as the run of points a mobject keeps: the vertices as
/// anchors with a handle at each midpoint (what `polyline` builds), always
/// an odd count. `Stage::set_points` takes THAT, not a vertex list: an even
/// count is refused and the mobject silently never changes, an odd one is
/// drawn as curves with every second vertex a control point.
pub fn corners(vertices: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::with_capacity(2 * vertices.len());
    for w in vertices.windows(2) {
        out.push(w[0]);
        out.push([
            (w[0][0] + w[1][0]) / 2.0,
            (w[0][1] + w[1][1]) / 2.0,
            (w[0][2] + w[1][2]) / 2.0,
        ]);
    }
    if let Some(&last) = vertices.last() {
        out.push(last);
    }
    out
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

    /// The line's sentences as written, so that what is set on screen is
    /// what is said (never retyped).
    pub fn sentence_texts(&self) -> Vec<String> {
        let chars: Vec<char> = self.text.chars().collect();
        let mut starts: Vec<usize> = self.sentences.iter().map(|&(at, _)| at).collect();
        starts.push(chars.len() + 1);
        starts
            .windows(2)
            .map(|w| {
                let end = (w[1] - 1).min(chars.len());
                chars[w[0]..end.max(w[0])].iter().collect::<String>().trim().to_owned()
            })
            .collect()
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
    /// The chapter key the card was built under, to tell the runs of the
    /// section 5 audit apart (tests only).
    #[cfg(test)]
    chapter: String,
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
            #[cfg(test)]
            chapter: kit.chapter.clone(),
        })
    }

    /// Note when this card did `kind` (tests only: the section 5 audit).
    #[cfg(test)]
    fn log(&self, kind: &'static str, stage: &Stage<'_>) {
        quiz_log::record(&self.chapter, self.quiz.line, kind, stage.scene().time().to_f64());
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
        #[cfg(test)]
        {
            self.log("frame", stage);
            self.log("ring", stage);
        }
        Ok(())
    }

    /// The countdown: it opens once the question has been spoken, the
    /// narration silent (a listen window), one mallet note a second, the ring
    /// draining; at zero the ring goes and the answer lands in gold in its
    /// place, and has the screen for [`LAND_S`] before the next line.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = f64::from(self.quiz.seconds);
        let wav = sound::countdown(self.quiz)?;
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.sonify(stage, n + LAND_S, &label, &wav)?;
        let start = stage.scene().time().to_f64();
        #[cfg(test)]
        self.log("window_start", stage);
        stage.play(tween_linear(self.left, 0.0, n)?)?;
        #[cfg(test)]
        {
            self.log("window_end", stage);
            self.log("land", stage);
        }
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

/// The video's teaching sounds (`chapters::sound`, the sound plan of the
/// storyboard), re-exported here for the chapters that already reach them
/// through the kit.
pub use super::sound;

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
/// above `CARD_TOP` when the card comes. Two rows end at y = −2.88, three at
/// −3.38, and a statement of two lines is 0.34 more (the card then rises, to
/// keep its foot inside the frame's safe area; the lower-third's band is in
/// the way of a third row: fade it).
pub const CARD_TOP: f64 = -1.6;
pub const CARD_W: f64 = 12.8;
pub const CARD_PT: f64 = 30.0;
const CARD_REASON_PT: f64 = 26.0;
const CARD_PAD: f64 = 0.14;
const CARD_ROW: f64 = 0.5;
/// The pitch of a statement's second line, and of a reason's two lines.
const CARD_LINE: f64 = 0.34;
const CARD_REASON_LINE: f64 = 0.25;
/// The size of a reason of two lines: with its pitch, two adjacent rows'
/// reasons stand as one block of evenly spaced lines.
const CARD_REASON_TWO_PT: f64 = 22.0;
/// A statement or reason that would be set smaller than this share of its
/// size takes two lines.
pub const CARD_FLOOR: f64 = 0.8;
/// From the labels' left edge to the statements' left edge.
const CARD_LABEL_COL: f64 = 1.75;
/// The room an Assumed line keeps on its right for its mark and its reason.
const CARD_MARK_ROOM: f64 = 3.6;

/// The source of a statement.
fn source_of<'a>(words: Words<'a>) -> &'a str {
    match words {
        Words::Text(s) | Words::Tex(s) => s,
    }
}

/// `s` as the same kind of words as `words`.
fn like<'a>(words: Words<'_>, s: &'a str) -> Words<'a> {
    match words {
        Words::Text(_) => Words::Text(s),
        Words::Tex(_) => Words::Tex(s),
    }
}

/// `words` broken into two lines of about the same length at a space: in
/// plain text any space; in TeX a space of the top level or of a `\text{}`
/// group (the group is closed on the first line and opened again on the
/// second). `None` if there is no such space.
fn two_lines(words: Words<'_>) -> Option<(String, String)> {
    match words {
        Words::Text(s) => {
            let chars: Vec<char> = s.chars().collect();
            let mut best: Option<(usize, usize)> = None;
            for (i, &c) in chars.iter().enumerate() {
                if c == ' ' && i > 0 && i + 1 < chars.len() {
                    let gap = i.abs_diff(chars.len() - i - 1);
                    if best.is_none_or(|(g, _)| gap < g) {
                        best = Some((gap, i));
                    }
                }
            }
            let (_, i) = best?;
            Some((chars[..i].iter().collect(), chars[i + 1..].iter().collect()))
        }
        Words::Tex(s) => tex_two_lines(s),
    }
}

/// [`two_lines`] for TeX.
fn tex_two_lines(src: &str) -> Option<(String, String)> {
    // the groups open at each point: true for a `\text{}` group, false for
    // any other
    let mut open: Vec<bool> = Vec::new();
    // what is shown before the point: letters, digits, symbols, and in a
    // text group its spaces
    let mut shown = 0usize;
    let mut spaces: Vec<(usize, bool, usize)> = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let rest = &src[i..];
        if rest.starts_with("\\text{") {
            open.push(true);
            i += "\\text{".len();
            continue;
        }
        let c = rest.chars().next()?;
        match c {
            '\\' => {
                let after = &rest[1..];
                let name = after.chars().take_while(char::is_ascii_alphabetic).count();
                i += 1 + if name > 0 {
                    name
                } else {
                    after.chars().next().map_or(0, char::len_utf8)
                };
                shown += 1;
            }
            '{' => {
                open.push(false);
                i += 1;
            }
            '}' => {
                open.pop();
                i += 1;
            }
            '^' | '_' => i += 1,
            ' ' => {
                if open.iter().all(|&t| t) {
                    spaces.push((i, !open.is_empty(), shown));
                }
                if open.last() == Some(&true) {
                    shown += 1;
                }
                i += 1;
            }
            _ => {
                shown += 1;
                i += c.len_utf8();
            }
        }
    }
    let (at, in_text, _) = spaces
        .into_iter()
        .filter(|&(i, in_text, _)| {
            i > 0 && !(in_text && (src[..i].ends_with("\\text{") || src[i + 1..].starts_with('}')))
        })
        .min_by_key(|&(_, _, before)| (2 * before).abs_diff(shown))?;
    if in_text {
        Some((format!("{}}}", &src[..at]), format!("\\text{{{}", &src[at + 1..])))
    } else {
        Some((src[..at].to_owned(), src[at + 1..].to_owned()))
    }
}

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
    /// The statement's last line, which the mark stands after, and its y.
    tail: Mob,
    tail_y: f64,
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

    /// As [`ProofCard::build`], somewhere else or at another size. A
    /// statement that would be set smaller than [`CARD_FLOOR`] of its size
    /// takes two lines, and the card grows to hold them (it rises, so that
    /// its foot stays inside the frame's safe area).
    pub fn build_at(kit: &Kit, stage: &mut Stage<'_>, rows: &[CardRow<'_>], look: CardLook) -> fmn::Result<Self> {
        if rows.is_empty() {
            return Err(fail("the card needs at least one line".into()));
        }
        let k = look.scale;
        let w = CARD_W * k;
        let left = look.x - w / 2.0 + 0.4 * k;
        let right = look.x + w / 2.0 - 0.4 * k;
        // each row's label and statement (detached, at the origin's height)
        let mut built: Vec<(Mob, Vec<Mob>, bool, f64, f64)> = Vec::new();
        for row in rows {
            let (name, words, assumed) = match *row {
                CardRow::Shown(w) => (SHOWN, w, false),
                CardRow::Assumed(w) => (ASSUMED, w, true),
                CardRow::Counted(w) => (COUNTED, w, false),
            };
            let label = kit.text_styled(stage, name, CARD_PT * k, ink(), v(left, 0.0), true, false)?;
            let from = left + (CARD_LABEL_COL * k).max(stage.get_width(label) + 0.35 * k);
            let room = right - from - if assumed { CARD_MARK_ROOM * k } else { 0.0 };
            let one = words.set_left(kit, stage, CARD_PT * k, ink(), v(from, 0.0))?;
            let mut lines = vec![(one, source_of(words).to_owned())];
            if stage.get_width(one) * CARD_FLOOR > room
                && let Some((a, b)) = two_lines(words)
            {
                let first = like(words, &a).set_left(kit, stage, CARD_PT * k, ink(), v(from, 0.0))?;
                let second = like(words, &b).set_left(kit, stage, CARD_PT * k, ink(), v(from, 0.0))?;
                lines = vec![(first, a), (second, b)];
            }
            let mut mobs = Vec::new();
            for (m, source) in lines {
                #[cfg(test)]
                fit_log::row(&source, stage.get_width(m), room);
                let _ = &source;
                fit_width_left(stage, m, room);
                mobs.push(m);
            }
            // two lines are as far apart as their heights need (a fraction
            // is taller than a letter), and no nearer than a text line
            let pitch = match mobs[..] {
                [a, b] => ((stage.get_height(a) + stage.get_height(b)) / 2.0 + 0.04 * k).max(CARD_LINE * k),
                _ => 0.0,
            };
            built.push((label, mobs, assumed, from, pitch));
        }
        // the card's height, and its top: a taller card rises
        let heights: Vec<f64> = built.iter().map(|(.., pitch)| CARD_ROW * k + pitch).collect();
        let h = 2.0 * CARD_PAD * k + heights.iter().sum::<f64>();
        let top = look.top.max(-FRAME_Y_RADIUS + 0.15 + h);
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.14 * k)
            .style(Style::default().fill(panel(), 0.96).stroke(ink(), 2.0, 1.0))
            .build()?
            .moved_to(v(look.x, top - h / 2.0));
        let back = stage.arena_mut().add(back);
        let mut lines = Vec::new();
        let mut row_top = top - CARD_PAD * k;
        for ((label, mobs, assumed, from, pitch), height) in built.into_iter().zip(heights) {
            // the row's lines stand about its middle, the label beside it
            let middle = row_top - height / 2.0;
            let first_y = middle + pitch / 2.0;
            let tail_y = first_y - pitch * (mobs.len() - 1) as f64;
            stage.move_to(label, v(left, middle), LEFT);
            for (j, &m) in mobs.iter().enumerate() {
                stage.move_to(m, v(from, first_y - pitch * j as f64), LEFT);
            }
            let tail = mobs[mobs.len() - 1];
            let words = if mobs.len() == 1 { mobs[0] } else { group(stage, &mobs)? };
            lines.push(CardLine {
                label,
                words,
                mark: None,
                reason: None,
                assumed,
                tail,
                tail_y,
            });
            row_top -= height;
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
        // the mark stands after the statement's last line
        let x = stage.get_right(row.tail)[0] + 0.45 * k;
        let shape = if ok { checkmark(color) } else { exmark(color) };
        let side = 0.3 * k;
        let mark = stage
            .arena_mut()
            .add(shape.with_height(side, false).moved_to(v(x + side / 2.0, row.tail_y)));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = fmn::animation::grow_from_center(stage.arena_mut(), mark, None)?;
        anims.push(stage.prepare(a.rt(0.4))?);
        row.mark = Some(mark);
        if !reason.is_empty() {
            let from = x + side + 0.22 * k;
            let room = right - from;
            let color = if ok { dim() } else { warn() };
            let at = row.tail_y;
            let one = kit.text_left(stage, reason, CARD_REASON_PT * k, color, v(from, at))?;
            // a reason that would be set below the floor takes two lines,
            // one over the other beside the mark, a size smaller so that the
            // lines of two rows' reasons keep apart (a row is 0.5 high)
            let split = if stage.get_width(one) * CARD_FLOOR > room {
                two_lines(Words::Text(reason))
            } else {
                None
            };
            let (parts, pt) = match split {
                Some((a, b)) => {
                    let up = at + CARD_REASON_LINE * k / 2.0;
                    let down = at - CARD_REASON_LINE * k / 2.0;
                    let first = kit.text_left(stage, &a, CARD_REASON_TWO_PT * k, color, v(from, up))?;
                    let second = kit.text_left(stage, &b, CARD_REASON_TWO_PT * k, color, v(from, down))?;
                    (vec![(first, a), (second, b)], CARD_REASON_TWO_PT)
                }
                None => (vec![(one, reason.to_owned())], CARD_REASON_PT),
            };
            let mut mobs = Vec::new();
            for (m, source) in parts {
                // (logged as it would be at the full reason size)
                #[cfg(test)]
                fit_log::row(&source, stage.get_width(m) * CARD_REASON_PT / pt, room);
                let _ = (&source, pt);
                fit_width_left(stage, m, room);
                mobs.push(m);
            }
            let words = if mobs.len() == 1 { mobs[0] } else { group(stage, &mobs)? };
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

// ----------------------------------------- terms of a line, each its colour

/// An exact fraction as a fact prints it (`21/64`, `1/3`, `2`) in TeX:
/// `\frac{21}{64}`, `\frac{1}{3}`, `2`.
pub fn frac_tex(s: &str) -> String {
    match s.split_once('/') {
        Some((n, d)) => format!("\\frac{{{n}}}{{{d}}}"),
        None => s.to_owned(),
    }
}

/// Terms of one line of mathematics, each in its own colour, laid left to
/// right with `gap` between them and centred at `centre`; shrunk together
/// if the whole is wider than `room` (text-per-term colour by one mobject
/// a term, never `t2c`: a repeated term would share one colour). Detached.
pub fn tex_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[(&str, Srgb)],
    size: f64,
    centre: Vec3,
    gap: f64,
    room: f64,
) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for &(src, color) in parts {
        mobs.push(kit.tex(stage, src, size, color, &[], ORIGIN)?);
    }
    let widths: Vec<f64> = mobs.iter().map(|&m| stage.get_width(m)).collect();
    let total = widths.iter().sum::<f64>() + gap * (parts.len() as f64 - 1.0).max(0.0);
    let f = if total > room { room / total } else { 1.0 };
    let mut x = centre[0] - total * f / 2.0;
    for (&m, &w) in mobs.iter().zip(&widths) {
        if f < 1.0 {
            stage.set_width(m, w * f, false);
        }
        stage.move_to(m, v(x + w * f / 2.0, centre[1]), ORIGIN);
        x += (w + gap) * f;
    }
    Ok(mobs)
}

/// A lower-third that mixes words and formulas ("the mixed formula-and-words
/// lower-third as two mobjects", build note E10): the `label` in gold, then
/// each of `parts` in turn, on the kit's band at the foot of the frame, shrunk
/// to fit. Detached, one group.
pub fn formula_third(kit: &Kit, stage: &mut Stage<'_>, label: &str, parts: &[Words<'_>]) -> fmn::Result<Mob> {
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let h = 0.9;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(faint(), 1.5, 1.0))
        .build()?
        .moved_to(centre);
    let back = stage.arena_mut().add(back);
    let tag = kit.text_left(stage, label, 26.0, hero(), v(0.0, centre[1]))?;
    let mut mobs = vec![tag];
    for part in parts {
        mobs.push(part.set(kit, stage, 26.0, ink(), v(0.0, centre[1]))?);
    }
    let gap = 0.2;
    let widths: Vec<f64> = mobs.iter().map(|&m| stage.get_width(m)).collect();
    let total = widths.iter().sum::<f64>() + gap * (mobs.len() as f64 - 1.0);
    let room = w - 0.6;
    let f = if total > room { room / total } else { 1.0 };
    let mut x = centre[0] - total * f / 2.0;
    for (&m, &mw) in mobs.iter().zip(&widths) {
        if f < 1.0 {
            stage.set_width(m, mw * f, false);
        }
        stage.move_to(m, v(x + mw * f / 2.0, centre[1]), ORIGIN);
        x += (mw + gap) * f;
    }
    let mut all = vec![back];
    all.extend(mobs);
    group(stage, &all)
}

// ---------------------------------------------------- the shelf and the lens

/// The shelf along the top edge (storyboard, "The card"): when a card has
/// passed, its picture shrinks to a thumbnail in the next slot, left to
/// right. Slot `i` is `i`-th in the order the cards pass: 03, 04, 06, 09, 10,
/// 11, then 12 and 13; the lying picture leaves its slot empty. A picture
/// above the shelf's row keeps below [`SHELF_FLOOR`].
pub const SHELF_LEFT: f64 = -6.0;
pub const SHELF_Y: f64 = 3.4;
pub const SHELF_PITCH: f64 = 1.3;
/// The side of a thumbnail, scene units.
pub const THUMB: f64 = 0.8;
/// A picture that shares the screen with the shelf stays below this y.
pub const SHELF_FLOOR: f64 = 2.95;

/// The middle of shelf slot `i` (from 0).
pub fn shelf_slot(i: usize) -> Vec3 {
    v(SHELF_LEFT + SHELF_PITCH * i as f64, SHELF_Y)
}

/// File a picture on the shelf: every mobject of `mobs` (already in the
/// scene, not grouped) shrinks about `from` by `THUMB / size` and lands so
/// that the picture, `size` scene units across and centred at `from`, is a
/// thumbnail centred in slot `slot`. `rt` seconds, all together.
pub fn file_on_shelf(
    stage: &mut Stage<'_>,
    mobs: &[Mob],
    from: Vec3,
    size: f64,
    slot: usize,
    rt: f64,
) -> fmn::Result<()> {
    let k = THUMB / size;
    let to = shelf_slot(slot);
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let c = stage.get_center(m);
        let target = v(to[0] + k * (c[0] - from[0]), to[1] + k * (c[1] - from[1]));
        stage.clear_updaters(m, true);
        anims.push(stage.prepare(fly(m, target, k, rt)?)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    #[cfg(test)]
    shelf_log::filed(stage, slot, mobs);
    Ok(())
}

/// A filed picture as it stands on the shelf, rebuilt by its chapter: the
/// mobjects it files, at thumbnail scale in its slot (detached, as
/// [`shelve`] leaves them).
pub type ThumbFn = fn(&Kit, &mut Stage<'_>) -> fmn::Result<Vec<Mob>>;

/// One chapter's place on the shelf (the table is `chapters::SHELF`): the
/// chapter whose card files a picture, the slot it takes, and the function
/// that rebuilds the filed thumbnail for the chapters after it.
#[derive(Clone, Copy)]
pub struct Filed {
    pub chapter: usize,
    pub slot: usize,
    pub thumb: ThumbFn,
}

/// `mobs` (detached, as they stood when their card passed) moved to slot
/// `slot` at once: the move [`file_on_shelf`] plays, without the animation.
/// Each mobject is scaled by `THUMB / size` about its own centre and
/// carried so that the picture, `size` scene units across and centred at
/// `from`, is a thumbnail centred in the slot.
pub fn shelve(stage: &mut Stage<'_>, mobs: &[Mob], from: Vec3, size: f64, slot: usize) {
    let k = THUMB / size;
    let to = shelf_slot(slot);
    for &m in mobs {
        let c = stage.get_center(m);
        let target = v(to[0] + k * (c[0] - from[0]), to[1] + k * (c[1] - from[1]));
        stage.scale(m, k);
        stage.move_to(m, target, ORIGIN);
    }
}

/// The shelf as it stands when chapter `chapter` (from 1) opens: every
/// thumbnail filed by the chapters before it, in its slot, on the stage at
/// once with no animation (each chapter is its own scene, so a chapter that
/// shares the screen with the shelf begins with this call). A chapter whose
/// card refused to file leaves its empty slot (`chapters::SHELF`). The
/// pictures a chapter draws beside the shelf keep under [`SHELF_FLOOR`].
pub fn shelf_so_far(kit: &Kit, stage: &mut Stage<'_>, chapter: usize) -> fmn::Result<Vec<Mob>> {
    let mut all = Vec::new();
    for filed in super::SHELF.iter().filter(|f| f.chapter < chapter) {
        let mobs = (filed.thumb)(kit, stage)?;
        for &m in &mobs {
            stage.add_to_scene(m)?;
        }
        #[cfg(test)]
        shelf_log::opened(stage, filed.slot, &mobs);
        all.extend(mobs);
    }
    #[cfg(test)]
    shelf_log::watch(stage, &all)?;
    Ok(all)
}

/// The thumbnail filed in slot `slot`, built again and carried to `at`,
/// `size` across (the small pictures "from the shelf" of 15_3). Detached.
pub fn shelf_copy(kit: &Kit, stage: &mut Stage<'_>, slot: usize, at: Vec3, size: f64) -> fmn::Result<Vec<Mob>> {
    let filed = super::SHELF
        .iter()
        .find(|f| f.slot == slot)
        .ok_or_else(|| fail(format!("nothing is filed in slot {slot}")))?;
    let mobs = (filed.thumb)(kit, stage)?;
    let (k, from) = (size / THUMB, shelf_slot(slot));
    for &m in &mobs {
        let c = stage.get_center(m);
        let target = v(at[0] + k * (c[0] - from[0]), at[1] + k * (c[1] - from[1]));
        stage.scale(m, k);
        stage.move_to(m, target, ORIGIN);
    }
    Ok(mobs)
}

/// The empty outline a card that refused to file leaves in its slot
/// (storyboard 14_7: the shelf keeps the gap). Detached.
pub fn empty_slot(stage: &mut Stage<'_>, slot: usize) -> fmn::Result<Mob> {
    let m = stage.arena_mut().add(
        Rectangle::new()
            .width(THUMB)
            .height(THUMB)
            .corner_radius(0.08)
            .style(faded_line(faint(), 2.0, 1.0))
            .build()?
            .moved_to(shelf_slot(slot)),
    );
    Ok(m)
}

/// Where the magnifier ring sits (storyboard, "The card": the Assumed line is
/// a close-up of the fit it names, on the same ring every time) and its
/// radius.
pub const LENS_AT: Vec3 = [2.7, 0.7, 0.0];
pub const LENS_R: f64 = 1.05;

/// The magnifier ring, in ink, centred at [`LENS_AT`]. Detached; the
/// chapter draws the close-up inside it.
pub fn lens(stage: &mut Stage<'_>) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(LENS_R)
            .arc_center(LENS_AT)
            .style(faded_line(ink(), 4.0, 1.0))
            .build(),
    )
}

/// A tick (or, with `ok` false, a cross) drawn over the close-up, `size`
/// across, centred at `at`: ink for a tick, `warn` for a cross. Detached.
pub fn lens_mark(stage: &mut Stage<'_>, at: Vec3, size: f64, ok: bool) -> Mob {
    let color = if ok { ink() } else { warn() };
    let shape = if ok { checkmark(color) } else { exmark(color) };
    stage.arena_mut().add(shape.with_height(size, false).moved_to(at))
}

// ------------------------------------------------------------ the harness

/// Construct a real chapter in a test: render it to a tiny PNG sequence
/// through franken_manim's facade with the kit's layout lint on, so every
/// `wait_for` phrase, every TeX string, every tracker and updater runs.
#[cfg(test)]
pub mod testing {
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use explainer_kit::{Kit, Linter, Narrator, Registered, Script};
    use fmn::prelude::*;

    /// What a constructed chapter left behind.
    pub struct Ran {
        pub frames: u64,
        /// The scene's length, seconds (at the test's frame rate).
        pub seconds: f64,
        /// The layout lint's findings (empty when clean).
        pub lint: String,
        pub lint_count: usize,
        /// Every text-like mobject the chapter built, with its source.
        pub registered: Vec<Registered>,
        /// Every card row and tick reason of the run that did not fit its room
        /// at full size: its words and the share of the full size it was set
        /// at (a row that fits is not here).
        pub squeezed: Vec<(String, f64)>,
    }

    /// Run chapter `key` of the registry at 64 by 36 and 10 frames a second,
    /// visual-only (no narrator: a line lasts its scratch-voice estimate).
    pub fn run(key: &str) -> Result<Ran, String> {
        run_as(key, key, None).map(|(ran, _)| ran)
    }

    /// The same run with the scratch voice placed: every line starts when
    /// the voice would be free and lasts its estimated length, as in a
    /// scratch-voice draft, so the timing of a quiz can be measured.
    pub fn run_voiced(key: &str) -> Result<Voiced, String> {
        let script = Script::parse(crate::SCRIPT)?;
        let narrator = Narrator::estimate(&script);
        // a name of its own, so the events of this run are not mixed with
        // another test's run of the same chapter
        let tag = format!("{key}#voiced");
        let (ran, kit) = run_as(key, &tag, Some(Rc::clone(&narrator)))?;
        let timeline = kit
            .finished
            .borrow()
            .clone()
            .ok_or_else(|| format!("{key}: the chapter left no timeline"))?;
        let fps = f64::from(timeline.fps);
        let lines = narrator
            .said()
            .into_iter()
            .map(|(id, at)| {
                let len = narrator.length(&id).unwrap_or(0.0);
                (id, at, len)
            })
            .collect();
        let listens = timeline
            .listens
            .iter()
            .map(|w| (w.label.clone(), w.frame as f64 / fps, w.seconds))
            .collect();
        let marks = timeline
            .marks
            .iter()
            .map(|m| (m.label.clone(), m.frame as f64 / fps))
            .collect();
        Ok(Voiced {
            ran,
            tag,
            lines,
            listens,
            marks,
        })
    }

    /// Run chapter `key` with its kit named `name`, and a narrator if given.
    fn run_as(key: &str, name: &str, narrator: Option<Rc<Narrator>>) -> Result<(Ran, Kit), String> {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let chapter = crate::chapters::registry()
            .into_iter()
            .find(|c| c.key == key)
            .ok_or_else(|| format!("no chapter {key} in the registry"))?;
        let lint = Rc::new(Linter::new());
        let mut kit = Kit::new().map_err(|e| e.to_string())?.with_lint(Rc::clone(&lint));
        if let Some(n) = narrator {
            kit = kit.with_narrator(n);
        }
        kit.chapter = name.to_owned();
        let mut scene = (chapter.make)(kit.clone());
        let n = RUNS.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("visual_proofs_{key}_{}_{n}", std::process::id()));
        let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).map_err(|e| e.to_string())?;
        opts.config.camera.resolution = (64, 36);
        opts.config.camera.fps = 10;
        opts.config.camera.background_color = super::ground_hex().to_owned();
        let report = fmn::render(scene.as_mut(), opts).map_err(|e| format!("{key}: {e}"))?;
        let frames = report.artifact.frame_count as u64;
        shelf_checks(key)?;
        let squeezed = super::fit_log::take();
        if let Some((text, scale)) = squeezed.iter().find(|(_, s)| *s < super::CARD_FLOOR - 1e-9) {
            return Err(format!(
                "{key}: a card row is set at {scale:.2} of its size, under the floor {}: {text:?}",
                super::CARD_FLOOR
            ));
        }
        let ran = Ran {
            frames,
            seconds: frames as f64 / 10.0,
            lint: lint.report(),
            lint_count: lint.count(),
            registered: kit.registered(),
            squeezed,
        };
        Ok((ran, kit))
    }

    /// Chapters from 04 on whose scene does not yet open with
    /// `shelf_so_far`: a lane that lands the call takes its number off.
    const NOT_YET_OPENING: &[usize] = &[];
    /// Chapters whose card files a picture and that have no `chapters::SHELF`
    /// entry yet: a lane that lands its `thumb` takes its number off.
    const NOT_YET_FILING: &[usize] = &[];
    /// How many updates (frames, at the tests' 10 a second) a picture may be
    /// over the shelf: a move across it is not a place.
    const STAYS_FRAMES: u32 = 15;

    /// What the run of chapter `key` did with the shelf, against the table
    /// (`chapters::SHELF`): (a) it opened with exactly the slots the
    /// chapters before it file; (c) no picture of it reached above the
    /// shelf's floor over them at any frame; (b) the picture it files stands
    /// where its `thumb` has it (the same box, to a hundredth of a unit).
    fn shelf_checks(key: &str) -> Result<(), String> {
        let n: usize = key
            .split('_')
            .next()
            .and_then(|d| d.parse().ok())
            .ok_or_else(|| format!("{key} has no chapter number"))?;
        let log = super::shelf_log::take();
        let shelf = crate::chapters::SHELF;
        // (a) the opening
        let mut got: Vec<usize> = log.opened.iter().map(|&(s, _)| s).collect();
        got.sort_unstable();
        let mut want: Vec<usize> = shelf.iter().filter(|f| f.chapter < n).map(|f| f.slot).collect();
        want.sort_unstable();
        if NOT_YET_OPENING.contains(&n) {
            if !got.is_empty() {
                return Err(format!("{key} opens with the shelf: take {n} off NOT_YET_OPENING"));
            }
        } else if got != want {
            return Err(format!(
                "{key} opens with slots {got:?}; the chapters before it file {want:?} (open with draw::shelf_so_far(stage, {n}))"
            ));
        }
        // (c) nothing over the shelf
        // (a picture sliding across the thumbnails for a moment is a move, not
        // a place: it has to stay over them for more than half a second)
        let stays: Vec<String> = log
            .strays
            .iter()
            .filter(|st| st.frames >= STAYS_FRAMES)
            .map(|st| format!("from update {} for {}: {}", st.from, st.frames, st.note))
            .collect();
        if !stays.is_empty() {
            return Err(format!("{key}: pictures over the shelf:\n{}", stays.join("\n")));
        }
        // (b) what it files, and the thumbnail of it
        let own = shelf.iter().find(|f| f.chapter == n);
        match own {
            None if NOT_YET_FILING.contains(&n) => {}
            None => {
                if !log.filed.is_empty() {
                    return Err(format!(
                        "{key} files slot {:?} and has no entry in chapters::SHELF",
                        log.filed.iter().map(|(s, _)| s).collect::<Vec<_>>()
                    ));
                }
            }
            Some(entry) => {
                if NOT_YET_FILING.contains(&n) {
                    return Err(format!("{key} has a SHELF entry: take {n} off NOT_YET_FILING"));
                }
                let filed: Vec<_> = log.filed.iter().filter(|(s, _)| *s == entry.slot).collect();
                if filed.len() != 1 || log.filed.len() != 1 {
                    return Err(format!(
                        "{key} files slots {:?}; SHELF has it take slot {} once",
                        log.filed.iter().map(|(s, _)| s).collect::<Vec<_>>(),
                        entry.slot
                    ));
                }
                let want = filed[0].1;
                let mut got = None;
                bare(|kit, stage| {
                    let mobs = (entry.thumb)(kit, stage)?;
                    got = super::shelf_log::union_box(stage, &mobs);
                    Ok(())
                })?;
                let got = got.ok_or_else(|| format!("{key}: its thumbnail shows nothing"))?;
                for (g, w, what) in [
                    (got.0[0], want.0[0], "left"),
                    (got.0[1], want.0[1], "bottom"),
                    (got.1[0], want.1[0], "right"),
                    (got.1[1], want.1[1], "top"),
                ] {
                    if (g - w).abs() >= 0.01 {
                        return Err(format!(
                            "{key}: the thumbnail's {what} is {g:.3}, the filed picture's {w:.3}"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// A scene that is only what a function puts on the stage, rendered tiny
    /// (the shelf's thumbnails are built from a stage and need no chapter).
    struct Bare<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>>(Kit, F);

    impl<F: FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Bare<F> {
        fn name(&self) -> &str {
            "bare"
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.1)(&self.0, stage)?;
            super::pause(stage, 0.2)
        }
    }

    /// Run `build` on a stage of its own, 64 by 36, and report an error it
    /// returns.
    pub fn bare(build: impl FnMut(&Kit, &mut Stage<'_>) -> fmn::Result<()>) -> Result<(), String> {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let n = RUNS.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("visual_proofs_bare_{}_{n}", std::process::id()));
        let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).map_err(|e| e.to_string())?;
        opts.config.camera.resolution = (64, 36);
        opts.config.camera.fps = 10;
        opts.config.camera.background_color = super::ground_hex().to_owned();
        let mut scene = Bare(Kit::new().map_err(|e| e.to_string())?, build);
        fmn::render(&mut scene, opts).map_err(|e| format!("bare: {e}"))?;
        Ok(())
    }

    /// A chapter run with the scratch voice placed.
    pub struct Voiced {
        pub ran: Ran,
        /// The kit's name in this run (the key of its quiz events).
        pub tag: String,
        /// Every line said: its id, its start and its length, seconds.
        pub lines: Vec<(String, f64, f64)>,
        /// Every listen window: its label, its start and its length, seconds.
        pub listens: Vec<(String, f64, f64)>,
        /// Every authored mark (`kit.mark`): its label and its time, seconds.
        pub marks: Vec<(String, f64)>,
    }

    /// What the quiz asked on one line did, measured (scene seconds).
    #[derive(Debug)]
    pub struct QuizAudit {
        pub line: String,
        /// The quiz's countdown, as `model::QUIZZES` has it.
        pub seconds: f64,
        /// When the narrator finishes the question (the line's end).
        pub question_ends: f64,
        /// When the card, with its question, was fully shown.
        pub card_shown: f64,
        /// When the ring and its seconds were full on screen.
        pub ring_armed: f64,
        /// The silent countdown: its start and its end.
        pub window: (f64, f64),
        /// When the answer began to land.
        pub answer_lands: f64,
        /// Lines whose start falls inside the countdown (there must be none).
        pub spoken_inside: Vec<String>,
        /// The next line and when it starts (None for the chapter's last).
        pub next_line: Option<(String, f64)>,
        /// The chapter's length with the scratch voice.
        pub chapter_seconds: f64,
        /// The listen window's own length, which holds the answer for
        /// `LAND_S` after the countdown.
        pub listen_seconds: f64,
    }

    /// Run the chapter of the quiz asked on `line` with the scratch voice and
    /// measure the quiz's times (REVIEW-AND-CONTINUITY section 5).
    pub fn audit_quiz(line: &str) -> Result<QuizAudit, String> {
        let quiz = crate::model::QUIZZES
            .iter()
            .find(|q| q.line == line)
            .ok_or_else(|| format!("line {line} asks no quiz in model::QUIZZES"))?;
        let prefix = format!("{}_", line.split('_').next().unwrap_or(line));
        let key = crate::chapters::registry()
            .into_iter()
            .map(|c| c.key)
            .find(|k| k.starts_with(&prefix))
            .ok_or_else(|| format!("no chapter for line {line}"))?;
        let voiced = run_voiced(key)?;
        let events = super::quiz_log::take(&voiced.tag, line);
        let at = |kind: &str| {
            events
                .iter()
                .find(|(k, _)| *k == kind)
                .map(|&(_, t)| t)
                .ok_or_else(|| format!("{line}: the card never did {kind:?} ({events:?})"))
        };
        let (_, start, len) = voiced
            .lines
            .iter()
            .find(|(id, ..)| id == line)
            .cloned()
            .ok_or_else(|| format!("{line} was never said"))?;
        let window = (at("window_start")?, at("window_end")?);
        let listen = voiced
            .listens
            .iter()
            .find(|(label, ..)| label.ends_with(&format!(":{line}")))
            .ok_or_else(|| format!("{line}: no listen window was opened"))?;
        let held = f64::from(quiz.seconds) + super::LAND_S;
        if (listen.1 - window.0).abs() > 0.11 || (listen.2 - held).abs() > 1e-9 {
            return Err(format!(
                "{line}: the listen window ({:.2} s for {:.1} s) is not the ring's ({:.2} s for {held:.1} s)",
                listen.1, listen.2, window.0
            ));
        }
        let spoken_inside = voiced
            .lines
            .iter()
            .filter(|(id, at, _)| id != line && *at > window.0 - 0.05 && *at < window.1 - 0.05)
            .map(|(id, ..)| id.clone())
            .collect();
        let next_line = voiced
            .lines
            .iter()
            .filter(|(_, at, _)| *at > start)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, at, _)| (id.clone(), *at));
        Ok(QuizAudit {
            line: line.to_owned(),
            seconds: f64::from(quiz.seconds),
            question_ends: start + len,
            card_shown: at("frame")?,
            ring_armed: at("ring")?,
            window,
            answer_lands: at("land")?,
            spoken_inside,
            next_line,
            chapter_seconds: voiced.ran.seconds,
            listen_seconds: listen.2,
        })
    }
}

/// The background as the engine's config wants it (a `#RRGGBB` string).
#[cfg(test)]
fn ground_hex() -> &'static str {
    explainer_kit::BACKGROUND
}

/// Events of the quiz cards during a test run, to measure them (the section
/// 5 audit): chapter name, quiz line, what happened, scene seconds.
#[cfg(test)]
pub mod quiz_log {
    use std::sync::Mutex;

    static EVENTS: Mutex<Vec<(String, String, &'static str, f64)>> = Mutex::new(Vec::new());

    pub fn record(chapter: &str, line: &str, kind: &'static str, at: f64) {
        if let Ok(mut events) = EVENTS.lock() {
            events.push((chapter.to_owned(), line.to_owned(), kind, at));
        }
    }

    /// What card `line` did in the run named `chapter`, in order.
    pub fn take(chapter: &str, line: &str) -> Vec<(&'static str, f64)> {
        EVENTS
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter(|(c, l, ..)| c == chapter && l == line)
                    .map(|&(_, _, kind, at)| (kind, at))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// What the shelf did in a test run on this thread: the boxes of what
/// `file_on_shelf` filed, of what `shelf_so_far` put up, and every picture
/// seen above the shelf's floor beside it.
#[cfg(test)]
pub mod shelf_log {
    use std::cell::RefCell;
    use std::collections::HashSet;

    use explainer_kit::layout::{Box2, visible_box};
    use fmn::prelude::*;

    use super::{SHELF_FLOOR, SHELF_Y, THUMB, shelf_slot};

    #[derive(Default)]
    pub struct Log {
        /// (slot, box) of each picture filed, as it landed.
        pub filed: Vec<(usize, Box2)>,
        /// (slot, box) of each thumbnail the chapter opened with.
        pub opened: Vec<(usize, Box2)>,
        /// Pictures that reached the shelf's row over occupied slots: what
        /// and where, the update it began at, and for how many updates.
        pub strays: Vec<Stray>,
    }

    pub struct Stray {
        pub mob: Mob,
        pub note: String,
        pub from: u32,
        pub frames: u32,
    }

    thread_local! {
        static LOG: RefCell<Log> = RefCell::new(Log::default());
    }

    /// Forget everything (a test starts a run with this).
    pub fn clear() {
        LOG.with(|l| *l.borrow_mut() = Log::default());
    }

    /// Take what the run logged.
    pub fn take() -> Log {
        LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
    }

    /// The box of what `mobs` show.
    pub fn union_box(stage: &Stage<'_>, mobs: &[Mob]) -> Option<Box2> {
        mobs.iter()
            .filter_map(|&m| visible_box(stage.arena(), m))
            .reduce(|(a, b), (c, d)| ([a[0].min(c[0]), a[1].min(c[1])], [b[0].max(d[0]), b[1].max(d[1])]))
    }

    pub fn filed(stage: &Stage<'_>, slot: usize, mobs: &[Mob]) {
        if let Some(b) = union_box(stage, mobs) {
            LOG.with(|l| l.borrow_mut().filed.push((slot, b)));
        }
    }

    pub fn opened(stage: &Stage<'_>, slot: usize, mobs: &[Mob]) {
        if let Some(b) = union_box(stage, mobs) {
            LOG.with(|l| l.borrow_mut().opened.push((slot, b)));
        }
    }

    /// Watch every frame: a visible mobject that reaches above the shelf's
    /// floor over the slots that hold something is a stray. `thumbs` are
    /// the shelf's own.
    pub fn watch(stage: &mut Stage<'_>, thumbs: &[Mob]) -> fmn::Result<()> {
        let slots: Vec<usize> = LOG.with(|l| l.borrow().opened.iter().map(|&(s, _)| s).collect());
        let Some(&last) = slots.iter().max() else {
            return Ok(());
        };
        let band = (shelf_slot(0)[0] - THUMB / 2.0, shelf_slot(last)[0] + THUMB / 2.0);
        let own: HashSet<Mob> = thumbs.iter().flat_map(|&m| stage.arena().family(m)).collect();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let mut frame = 0u32;
        stage.add_updater(
            anchor,
            move |s, _| {
                frame += 1;
                let found: Vec<(Mob, Box2)> = s
                    .roots()
                    .iter()
                    .filter(|m| **m != anchor && !own.contains(m))
                    .filter_map(|&m| visible_box(s, m).map(|b| (m, b)))
                    .filter(|(_, (lo, hi))| {
                        // above the thumbnails' bottom edge (the floor has a margin
                        // of 0.05 under it), over a slot that holds one; a veil over
                        // the whole frame is not a picture
                        hi[1] > SHELF_Y - THUMB / 2.0 + 0.015
                            && lo[0] < band.1
                            && hi[0] > band.0
                            && hi[0] - lo[0] < 2.0 * FRAME_X_RADIUS
                    })
                    .collect();
                for (m, (lo, hi)) in found {
                    LOG.with(|l| {
                        let mut l = l.borrow_mut();
                        match l.strays.iter_mut().find(|st| st.mob == m) {
                            Some(st) => st.frames += 1,
                            None => l.strays.push(Stray {
                                mob: m,
                                note: format!(
                                    "{m:?} spans x {:.2}..{:.2}, y {:.2}..{:.2}: reaches the shelf's row (floor {SHELF_FLOOR}) over x {:.2}..{:.2}",
                                    lo[0], hi[0], lo[1], hi[1], band.0, band.1
                                ),
                                from: frame,
                                frames: 1,
                            }),
                        }
                    });
                }
            },
            false,
        )?;
        Ok(())
    }
}

/// How far the card rows and reasons of a test run were squeezed into their
/// room: a row wider than its room is shrunk to it, and a row shrunk to half
/// is hard to read at a viewer's distance.
#[cfg(test)]
pub mod fit_log {
    use std::cell::RefCell;

    thread_local! {
        static ROWS: RefCell<Vec<(String, f64)>> = const { RefCell::new(Vec::new()) };
    }

    /// A row of `text`, `natural` wide at full size, in a room `room` wide.
    pub fn row(text: &str, natural: f64, room: f64) {
        if natural > room && natural > 0.0 {
            ROWS.with(|r| r.borrow_mut().push((text.to_owned(), room / natural)));
        }
    }

    /// Take what the run logged.
    pub fn take() -> Vec<(String, f64)> {
        ROWS.with(|r| std::mem::take(&mut *r.borrow_mut()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REVIEW-AND-CONTINUITY section 5, measured on a scratch-voice run of
    /// the quiz's chapter: the card and its question are readable no later
    /// than the narrator's last word of the question; the viewer then has
    /// the quiz's seconds (4 to 8, as `model::QUIZZES` has them) with the
    /// narration silent and a ring counting them; and the answer lands only
    /// after the countdown ended and is held for `LAND_S` before the next
    /// line. What stands on the screen as a possible hint is read in each
    /// chapter's code; this is the part that is a number.
    fn section_five(line: &str) {
        let a = testing::audit_quiz(line).unwrap();
        let said = format!("{a:#?}");
        // (shown with --nocapture: what the numbers are, not only that they pass)
        eprintln!(
            "{line}: question ends {:.1}s, card {:.1}s, ring {:.1}s, countdown {:.1}..{:.1}s ({} s), answer lands {:.1}s, next {:?}, chapter {:.1}s",
            a.question_ends,
            a.card_shown,
            a.ring_armed,
            a.window.0,
            a.window.1,
            a.seconds,
            a.answer_lands,
            a.next_line,
            a.chapter_seconds
        );
        // the viewer's thinking time
        assert!(
            (4.0..=8.0).contains(&a.seconds),
            "{line}: a {} s countdown\n{said}",
            a.seconds
        );
        assert!(
            (a.window.1 - a.window.0 - a.seconds).abs() <= 0.15,
            "{line}: the ring ran {:.2} s, not {} s\n{said}",
            a.window.1 - a.window.0,
            a.seconds
        );
        // readable no later than the narrator's last word of the question
        assert!(a.card_shown <= a.question_ends, "{line}: the card is late\n{said}");
        assert!(a.ring_armed <= a.question_ends, "{line}: the ring is late\n{said}");
        // the countdown starts when the question has finished, and nobody
        // speaks inside it
        assert!(
            a.window.0 >= a.question_ends - 1e-9,
            "{line}: counts before the question ends\n{said}"
        );
        assert!(
            a.spoken_inside.is_empty(),
            "{line}: a line starts inside the countdown\n{said}"
        );
        // the answer is shown only after it, and the answer's own line too
        assert!(
            a.answer_lands >= a.window.1 - 1e-9,
            "{line}: the answer lands early\n{said}"
        );
        if let Some((next, at)) = &a.next_line {
            assert!(
                *at >= a.window.1 - 0.05,
                "{line}: {next} starts inside the countdown\n{said}"
            );
        }
    }

    /// One audit test per quiz whose chapter is a real scene (the others
    /// are animatics until their lanes land them: a lane adds its line here
    /// with its chapter).
    macro_rules! section_five_tests {
        ($($name:ident: $line:literal),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    section_five($line);
                }
            )*
        };
    }

    section_five_tests! {
        quiz_02_5_follows_section_5: "02_5",
        quiz_03_3_follows_section_5: "03_3",
        quiz_04_3_follows_section_5: "04_3",
        quiz_05_4_follows_section_5: "05_4",
        quiz_11_3_follows_section_5: "11_3",
        quiz_12_4_follows_section_5: "12_4",
        quiz_13_4_follows_section_5: "13_4",
        quiz_14_4_follows_section_5: "14_4",
    }

    // ------------------------------------------------------- the shelf

    /// The shelf table is in the order the cards pass, one slot to a
    /// chapter, left to right. (What each chapter does with it, opening and
    /// filing, is checked on every run of a chapter: `testing::run`.)
    #[test]
    fn the_shelf_table_is_in_the_order_the_cards_pass() {
        let shelf = crate::chapters::SHELF;
        assert!(
            shelf.windows(2).all(|w| w[0].chapter < w[1].chapter),
            "chapters in order"
        );
        assert!(shelf.windows(2).all(|w| w[0].slot < w[1].slot), "slots left to right");
    }

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
            assert_eq!(p[p.len() - 3..], [76.0, 74.0, 72.0], "{}: E, D, C", q.name);
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
