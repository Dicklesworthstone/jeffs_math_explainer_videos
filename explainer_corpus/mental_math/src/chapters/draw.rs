//! Video F's look, shared by every chapter (storyboard: "Three kinds of
//! question", "The one move", "Numbers on screen", "Color roles"): areas
//! drawn to scale on a board of units, the strip that turns, the cut
//! rectangle, the quiz card with its countdown, the question card, number
//! lines, the six small pictures, the sheet of paper, the calculator, the
//! magnifier and the cards.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, small groups, and a dark cut line over the
//! seam of two fills. What moves is drawn from a value tracker by one
//! updater, which rewrites a handful of points.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::f64::consts::{FRAC_PI_2, PI, SQRT_2, TAU};

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, hex, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::sound;
use crate::model::{Ask, OneMove, QUIZZES, Quiz, Trick, by_parts, ends_in_five, money};

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the words it illustrates. A
/// phrase's moment is its share of its sentence's characters (the voice
/// reads at a nearly even rate); the cue sheets of the narrated draft check
/// the result.
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

/// Speak line `id` now (after the previous line), as `Kit::say`.
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

// ------------------------------------------------------------- colour roles

/// Hundreds: the big block, and any piece that is a whole number of hundreds.
pub fn hundreds() -> Srgb {
    Palette::blue()
}

/// The strips (tens times ones), and the strip that moves.
pub fn strip() -> Srgb {
    hex("#70CCA1")
}

/// The corner (ones times ones): the small square that sticks out.
pub fn corner() -> Srgb {
    hex("#CC811F")
}

/// The move itself: an outline on the moving piece, never a fill.
pub fn mover() -> Srgb {
    Palette::hero()
}

/// A piece taken away, a gap, a wrong reading.
pub fn gone() -> Srgb {
    Palette::red()
}

/// The "your turn" card and its countdown.
pub fn quiz() -> Srgb {
    hex("#B189D6")
}

/// A shape before it has a role (the square before it is cut, the bill).
pub fn slate() -> Srgb {
    hex("#3A465C")
}

/// The fill behind cards.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// The background, for cut lines over fills.
pub fn ground() -> Srgb {
    hex(explainer_kit::BACKGROUND)
}

/// A sheet of paper, and the ink written on it.
pub fn paper() -> Srgb {
    hex("#D9D3C2")
}

pub fn paper_ink() -> Srgb {
    hex("#1B1F27")
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

/// The cut between two fills: a line in the background's colour.
pub fn cut_style() -> Style {
    line_style(ground(), 3.0)
}

/// An `Srgb` as the `[f32; 3]` that `indicate` and its kin take (never call
/// `indicate` with no colour: its default yellow floods a fill).
pub fn rgb(c: Srgb) -> [f32; 3] {
    [c.r as f32, c.g as f32, c.b as f32]
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
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
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

/// Tween several trackers together, `rt` seconds.
pub fn tween_all(stage: &mut Stage<'_>, moves: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, value) in moves {
        anims.push(stage.prepare(tween(t, value, rt)?)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
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
pub fn dim(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

// ------------------------------------------------------- numbers on screen

/// A whole number as it is worked on screen: no thousands separator, so the
/// two-part reading (12 then 25, 23 then 04) stays visible.
pub fn num(n: u64) -> String {
    n.to_string()
}

/// Cents as TeX dollars with their cents: `\$4.80`.
pub fn dollars(cents: u64) -> String {
    format!("\\${}", money(cents))
}

/// A price as a problem states it: whole dollars without their zeros
/// (`\$60`), anything else in full (`\$4.99`).
pub fn price(cents: u64) -> String {
    if cents.is_multiple_of(100) {
        format!("\\${}", cents / 100)
    } else {
        dollars(cents)
    }
}

/// What a quiz asks, as TeX (a bare `%` would be a comment and a bare `$`
/// an error: both are escaped here and nowhere else).
pub fn ask_tex(ask: Ask) -> String {
    match ask {
        Ask::Square(x) => format!("{x}^2"),
        Ask::Times(a, b) => format!("{a} \\times {b}"),
        Ask::PercentOfMoney(p, cents) => format!("{p}\\% \\text{{ of }} {}", price(cents)),
        Ask::TimesMoney(n, cents) => format!("{n} \\times {}", price(cents)),
    }
}

/// The quiz asked on line `id`.
pub fn quiz_at(id: &str) -> fmn::Result<&'static Quiz> {
    QUIZZES
        .iter()
        .find(|q| q.line == id)
        .ok_or_else(|| fail(format!("line {id} asks no quiz in model::QUIZZES")))
}

/// A quiz's answer as TeX: money with its dollar sign, a number plain.
pub fn answer_tex(q: &Quiz) -> fmn::Result<String> {
    let answer = q.worked().map_err(fail)?.answer;
    Ok(match q.ask {
        Ask::PercentOfMoney(..) | Ask::TimesMoney(..) => format!("\\${answer}"),
        Ask::Square(_) | Ask::Times(..) => answer,
    })
}

/// A line of mathematics in ink, centred at `at`.
pub fn eq(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, at: Vec3) -> fmn::Result<Mob> {
    kit.tex(stage, src, size, Palette::ink(), &[], at)
}

/// A small grey line of mathematics: an aside that is true and no easier.
pub fn grey_line(kit: &Kit, stage: &mut Stage<'_>, src: &str, at: Vec3) -> fmn::Result<Mob> {
    kit.tex(stage, src, 34.0, Palette::dim(), &[], at)
}

/// Shrink `mob` about its centre until it is at most `room` wide.
pub fn fit_width(stage: &mut Stage<'_>, mob: Mob, room: f64) {
    if stage.get_width(mob) > room {
        let at = stage.get_center(mob);
        stage.set_width(mob, room, false);
        stage.move_to(mob, at, ORIGIN);
    }
}

/// The glyphs of a typeset formula, in source order (a `Tex`'s children).
pub fn glyphs(stage: &Stage<'_>, tex: Mob) -> Vec<Mob> {
    explainer_kit::children(stage, tex)
}

/// The middle of the box round several mobjects (a few glyphs of a formula).
pub fn middle_of(stage: &Stage<'_>, mobs: &[Mob]) -> fmn::Result<Vec3> {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for &m in mobs {
        let (l, r) = (stage.get_left(m)[0], stage.get_right(m)[0]);
        let (b, t) = (stage.get_bottom(m)[1], stage.get_top(m)[1]);
        lo = [lo[0].min(l), lo[1].min(b)];
        hi = [hi[0].max(r), hi[1].max(t)];
    }
    if mobs.is_empty() {
        return Err(fail("no mobjects to find the middle of".into()));
    }
    Ok(v((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0))
}

// ---------------------------------------------------------------- the board

/// Where the area pictures stand: the lower-left corner of the figure.
pub const FIGURE_AT: Vec3 = [-6.1, -2.6, 0.0];
/// The room the area pictures may take (scene units, wide and tall), the
/// strip's turn included: a square of side s needs about 1.5 s each way
/// while its strip turns.
pub const FIGURE_ROOM: (f64, f64) = (6.5, 6.2);
/// The middle of the column of sums beside the figure (it runs from about
/// x = 0.6 to x = 6.6).
pub const COLUMN_X: f64 = 3.6;
/// The widest a line of the column may be.
pub const COLUMN_ROOM: f64 = 6.0;
/// The size of a side's label, and of a smaller note on a piece.
pub const SIDE_PT: f64 = 36.0;
pub const NOTE_PT: f64 = 30.0;
/// The one caption slot under the stage.
pub const CAPTION_Y: f64 = -3.35;

/// Which side of a rectangle a label stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Below,
    Above,
}

/// Areas drawn to scale: a board of units (ones) with its origin at a scene
/// point. A rectangle is `[x, y, wide, tall]` in units, from its lower-left
/// corner; a rectangle's sides are always said tall by wide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub origin: Vec3,
    /// Scene units per one.
    pub unit: f64,
}

/// The tiles of a rectangle ruled into hundreds: its bright grid and, when
/// asked for, a small numeral in each tile, in reading order.
pub struct Hundreds {
    pub lines: Mob,
    pub numerals: Vec<Mob>,
}

impl Board {
    pub fn new(origin: Vec3, unit: f64) -> Self {
        Self { origin, unit }
    }

    /// The largest board at `origin` on which `wide` by `tall` units fit in
    /// `room` (scene units, wide and tall).
    pub fn fit(origin: Vec3, wide: f64, tall: f64, room: (f64, f64)) -> Self {
        Self {
            origin,
            unit: (room.0 / wide).min(room.1 / tall),
        }
    }

    /// The scene point of (`x`, `y`) units.
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        v(self.origin[0] + x * self.unit, self.origin[1] + y * self.unit)
    }

    /// A rectangle's corners, counter-clockwise from its lower left.
    pub fn quad(&self, r: [f64; 4]) -> [Vec3; 4] {
        let [x, y, w, h] = r;
        [self.p(x, y), self.p(x + w, y), self.p(x + w, y + h), self.p(x, y + h)]
    }

    pub fn centre(&self, r: [f64; 4]) -> Vec3 {
        self.p(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)
    }

    /// A rectangle in `style`. Detached.
    pub fn rect(&self, stage: &mut Stage<'_>, r: [f64; 4], style: Style) -> Mob {
        stage.arena_mut().add(Polygon::new(self.quad(r)).style(style).build())
    }

    /// A straight line between two points given in units. Detached.
    pub fn line(&self, stage: &mut Stage<'_>, a: [f64; 2], b: [f64; 2], style: Style) -> Mob {
        stage.arena_mut().add(
            polyline(vec![self.p(a[0], a[1]), self.p(b[0], b[1])])
                .style(style)
                .build(),
        )
    }

    /// The lines of a grid every `step` units across rectangle `r`, its
    /// border included, as one group. Detached.
    pub fn grid(&self, stage: &mut Stage<'_>, r: [f64; 4], step: f64, style: Style) -> fmn::Result<Mob> {
        let [x, y, w, h] = r;
        let mut lines = Vec::new();
        let mut at = 0.0;
        while at <= w + 1e-9 {
            lines.push(self.line(stage, [x + at, y], [x + at, y + h], style));
            at += step;
        }
        let mut at = 0.0;
        while at <= h + 1e-9 {
            lines.push(self.line(stage, [x, y + at], [x + w, y + at], style));
            at += step;
        }
        group(stage, &lines)
    }

    /// The faint grid of tens the pictures are drawn over.
    pub fn tens(&self, stage: &mut Stage<'_>, wide: f64, tall: f64) -> fmn::Result<Mob> {
        self.grid(
            stage,
            [0.0, 0.0, wide, tall],
            10.0,
            faded_line(Palette::faint(), 1.5, 0.7),
        )
    }

    /// Rectangle `r` (whole tens each way) ruled into squares of a hundred,
    /// with a small numeral in each (1, 2, … in reading order) when
    /// `numbered`. Detached.
    pub fn hundreds(&self, kit: &Kit, stage: &mut Stage<'_>, r: [f64; 4], numbered: bool) -> fmn::Result<Hundreds> {
        let lines = self.grid(stage, r, 10.0, faded_line(Palette::ink(), 2.5, 0.85))?;
        let mut numerals = Vec::new();
        if numbered {
            let (cols, rows) = ((r[2] / 10.0).round() as usize, (r[3] / 10.0).round() as usize);
            let size = (10.0 * self.unit * 34.0).clamp(14.0, 44.0);
            for row in 0..rows {
                for col in 0..cols {
                    let at = self.p(r[0] + 10.0 * col as f64 + 5.0, r[1] + r[3] - 10.0 * row as f64 - 5.0);
                    let n = row * cols + col + 1;
                    numerals.push(kit.text_styled(stage, &n.to_string(), size, paper_ink(), at, true, false)?);
                }
            }
        }
        Ok(Hundreds { lines, numerals })
    }

    /// Words beside rectangle `r`, centred along its `side`. Detached.
    #[allow(clippy::too_many_arguments)]
    pub fn label(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        words: &str,
        r: [f64; 4],
        side: Side,
        size: f64,
        color: Srgb,
    ) -> fmn::Result<Mob> {
        let [x, y, w, h] = r;
        let gap = 0.2;
        let (at, edge) = match side {
            Side::Left => (add2(self.p(x, y + h / 2.0), -gap, 0.0), RIGHT),
            Side::Right => (add2(self.p(x + w, y + h / 2.0), gap, 0.0), LEFT),
            Side::Below => (add2(self.p(x + w / 2.0, y), 0.0, -gap), UP),
            Side::Above => (add2(self.p(x + w / 2.0, y + h), 0.0, gap), DOWN),
        };
        let m = kit.text(stage, words, size, color, at)?;
        stage.move_to(m, at, edge);
        Ok(m)
    }

    /// A rectangle redrawn every frame from `f` (`[x, y, wide, tall]` in
    /// units), for a picture on a tracker. In the scene at once.
    pub fn live_rect(
        &self,
        stage: &mut Stage<'_>,
        f: impl Fn(&Arena) -> [f64; 4] + 'static,
        style: Style,
    ) -> fmn::Result<Mob> {
        let board = *self;
        let m = stage.always_redraw(move |s| {
            let [x, y, w, h] = f(s);
            let quad = board.quad([x, y, w.max(1e-4), h.max(1e-4)]);
            s.add(Polygon::new(quad).style(style).build())
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }
}

fn add2(p: Vec3, dx: f64, dy: f64) -> Vec3 {
    v(p[0] + dx, p[1] + dy)
}

// ------------------------------------------------------------ the one move

/// How long the strip takes to turn, inside the move's two silent seconds.
pub const TURN_S: f64 = 1.6;

/// A strip that turns: `len` long and `deep` thick, lying flat with its
/// centre at `flat` (units) at 0 and standing with its centre at `up` at 1.
/// It is one rigid rectangle the whole way, so its area never changes; it
/// lifts clear of the corner it turns round, a quarter turn clockwise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn {
    pub flat: [f64; 2],
    pub up: [f64; 2],
    pub len: f64,
    pub deep: f64,
    pub lift: f64,
}

impl Turn {
    /// The strip sliced off the top of the square of `m`, and the same
    /// strip stood against the square's right side.
    pub fn of(m: OneMove) -> Self {
        let (side, b) = (m.side as f64, m.strip as f64);
        Self::between([side / 2.0, side - b / 2.0], [side + b / 2.0, side / 2.0], side, b)
    }

    pub fn between(flat: [f64; 2], up: [f64; 2], len: f64, deep: f64) -> Self {
        Self {
            flat,
            up,
            len,
            deep,
            lift: (len / 2.0 - deep).max(0.0) / SQRT_2 + deep / 2.0 + 0.1 * len,
        }
    }

    /// The strip's corners at `t` (0 flat, 1 standing): it lifts first,
    /// turns in the air and comes down standing.
    pub fn quad(&self, t: f64) -> [[f64; 2]; 4] {
        let t = t.clamp(0.0, 1.0);
        let u = ((t - 0.12) / 0.76).clamp(0.0, 1.0);
        let angle = -FRAC_PI_2 * u * u * (3.0 - 2.0 * u);
        let bulge = self.lift * (PI * t).sin() / SQRT_2;
        let c = [
            self.flat[0] + (self.up[0] - self.flat[0]) * t + bulge,
            self.flat[1] + (self.up[1] - self.flat[1]) * t + bulge,
        ];
        let (along, across) = ([angle.cos(), angle.sin()], [-angle.sin(), angle.cos()]);
        let (hl, hd) = (self.len / 2.0, self.deep / 2.0);
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, b)| {
            [
                c[0] + a * hl * along[0] + b * hd * across[0],
                c[1] + a * hl * along[1] + b * hd * across[1],
            ]
        })
    }
}

/// The strip that moves, drawn from a tracker: its fill, and a gold outline
/// that shows only while it is on its way.
pub struct Strip {
    /// 0: lying flat; 1: standing.
    pub t: Mob,
    pub fill: Mob,
    pub edge: Mob,
    /// Carries the updater: it must be in the scene for the strip to move.
    pub anchor: Mob,
    pub turn: Turn,
    pub board: Board,
}

impl Strip {
    /// A strip on `board` in `color`, at `t0` of its turn. Detached.
    pub fn build(stage: &mut Stage<'_>, board: Board, turn: Turn, color: Srgb, t0: f64) -> fmn::Result<Self> {
        let t = stage.add_value_tracker(t0);
        let quad = move |t: f64| turn.quad(t).map(|[x, y]| board.p(x, y));
        let fill = stage
            .arena_mut()
            .add(Polygon::new(quad(t0)).style(flat(color, 1.0)).build());
        let edge = stage
            .arena_mut()
            .add(Polygon::new(quad(t0)).style(faded_line(mover(), 5.0, 0.0)).build());
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = s.tracker_value(t).unwrap_or(t0);
                if now == last {
                    return;
                }
                let shape = Polygon::new(quad(now)).build();
                let _ = s.set_points(fill, shape.points());
                let _ = s.set_points(edge, shape.points());
                let moving = (PI * now.clamp(0.0, 1.0)).sin().max(0.0).sqrt();
                s.set_stroke(edge, None, None, Some(moving), None, false);
                last = now;
            },
            true,
        )?;
        Ok(Self {
            t,
            fill,
            edge,
            anchor,
            turn,
            board,
        })
    }

    /// Everything to bring into the scene, back to front.
    pub fn mobs(&self) -> [Mob; 3] {
        [self.anchor, self.fill, self.edge]
    }

    /// The turn to `to` (1 standing, 0 flat) as an animation, [`TURN_S`]
    /// long: for a move made while the narrator speaks.
    pub fn turn_to(&self, to: f64) -> fmn::Result<AnimBuilder> {
        tween(self.t, to, TURN_S)
    }
}

/// The one move, made the same way every time: the narrator waits, one
/// short plucked sound, two seconds. Every strip in `strips` turns to `to`
/// together (1 standing, 0 flat). `id` is the line the move belongs to.
pub fn the_move(kit: &Kit, stage: &mut Stage<'_>, strips: &[&Strip], to: f64, id: &str) -> fmn::Result<()> {
    let wav = sound::the_move()?;
    kit.sonify(stage, sound::MOVE_S, &format!("{}:{id}", kit.chapter), &wav)?;
    let start = stage.scene().time().to_f64();
    pause(stage, 0.1)?;
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for s in strips {
        stage.scene_mut().bring_to_front(&[s.fill, s.edge])?;
        anims.push(stage.prepare(s.turn_to(to)?)?);
    }
    stage.play_prepared(anims)?;
    wait_until(stage, start + sound::MOVE_S)
}

/// The square whose strip moves (storyboard, "The one move"): a square of
/// side `m.side` with its lower-left corner at the board's origin; a strip
/// `m.strip` deep comes off its top and stands against its right side,
/// leaving a rectangle and, where the end of the strip stands above it, a
/// small square.
pub struct MoveFigure {
    pub board: Board,
    pub m: OneMove,
    /// The square less its top strip.
    pub body: Mob,
    pub strip: Strip,
    /// The cut under the strip while it lies on top.
    pub cut: Mob,
    /// The seam between the body and the strip once it stands.
    pub seam: Mob,
    /// The end of the standing strip above the rectangle, in the corner's
    /// colour, with the cut beneath it.
    pub corner: Mob,
    pub corner_cut: Mob,
    /// The rest of the standing strip in the body's colour: with it the
    /// rectangle is one piece.
    pub merged: Mob,
}

impl MoveFigure {
    /// The figure with its pieces in `body_color` and `strip_color`, the
    /// strip at `t0` of its turn (0 on top, 1 at the side). Detached.
    pub fn build(
        stage: &mut Stage<'_>,
        board: Board,
        m: OneMove,
        body_color: Srgb,
        strip_color: Srgb,
        t0: f64,
    ) -> fmn::Result<Self> {
        let (side, b) = (m.side as f64, m.strip as f64);
        let body = board.rect(stage, [0.0, 0.0, side, side - b], flat(body_color, 1.0));
        let strip = Strip::build(stage, board, Turn::of(m), strip_color, t0)?;
        let cut = board.line(stage, [0.0, side - b], [side, side - b], cut_style());
        let seam = board.line(stage, [side, 0.0], [side, side - b], cut_style());
        let corner = board.rect(stage, [side, side - b, b, b], flat(corner(), 1.0));
        let corner_cut = board.line(stage, [side, side - b], [side + b, side - b], cut_style());
        let merged = board.rect(stage, [side, 0.0, b, side - b], flat(hundreds(), 1.0));
        Ok(Self {
            board,
            m,
            body,
            strip,
            cut,
            seam,
            corner,
            corner_cut,
            merged,
        })
    }

    /// The plain square: the body and the strip, back to front.
    pub fn square(&self) -> Vec<Mob> {
        let mut out = vec![self.body];
        out.extend(self.strip.mobs());
        out
    }

    /// The whole square, `[x, y, wide, tall]`.
    pub fn square_rect(&self) -> [f64; 4] {
        let side = self.m.side as f64;
        [0.0, 0.0, side, side]
    }

    /// The strip where it lies on top.
    pub fn top_rect(&self) -> [f64; 4] {
        let (side, b) = (self.m.side as f64, self.m.strip as f64);
        [0.0, side - b, side, b]
    }

    /// The strip where it stands at the side.
    pub fn side_rect(&self) -> [f64; 4] {
        let (side, b) = (self.m.side as f64, self.m.strip as f64);
        [side, 0.0, b, side]
    }

    /// The rectangle the move makes: `m.tall()` tall by `m.wide()` wide.
    pub fn rectangle_rect(&self) -> [f64; 4] {
        [0.0, 0.0, self.m.wide() as f64, self.m.tall() as f64]
    }

    /// The small square: the end of the strip above the rectangle.
    pub fn corner_rect(&self) -> [f64; 4] {
        let (side, b) = (self.m.side as f64, self.m.strip as f64);
        [side, side - b, b, b]
    }

    /// Slice the strip off: the two pieces take their roles' colours and
    /// the cut shows, `rt` seconds.
    pub fn slice(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(repaint(self.body, hundreds(), 1.0, rt)?)?);
        anims.push(stage.prepare(repaint(self.strip.fill, strip(), 1.0, rt)?)?);
        let a = fade_in(stage.arena_mut(), self.cut, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// The one move, forwards: the strip leaves the top (the cut goes with
    /// it) and stands against the side (the seam shows).
    pub fn play(&self, kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        stage.remove_from_scene(self.cut);
        the_move(kit, stage, &[&self.strip], 1.0, id)?;
        stage.add_to_scene(self.seam)?;
        Ok(())
    }

    /// The end of the standing strip turns into the small square.
    pub fn show_corner(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &[self.corner, self.corner_cut], rt)
    }

    /// The rectangle becomes one piece in the hundreds' colour: the part of
    /// the strip beside the body joins it and the seam goes. Call it after
    /// [`MoveFigure::show_corner`], so the small square stays on top.
    pub fn merge(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        swap(stage, &[self.seam], &[self.merged], rt)?;
        stage.scene_mut().bring_to_front(&[self.corner, self.corner_cut])?;
        Ok(())
    }

    /// Everything of the figure that may be on stage, for a fade-out.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = self.square();
        out.extend([self.cut, self.seam, self.merged, self.corner, self.corner_cut]);
        out
    }
}

// ------------------------------------------------------- the cut rectangle

/// A rectangle `a` tall by `b` wide cut by the parts of its two numbers
/// (storyboard, chapter 04): the big block, the strips, the corner, in the
/// order the video adds them.
pub struct CutRect {
    pub board: Board,
    /// Each piece as `[x, y, wide, tall]`, in `model::by_parts` order.
    pub rects: Vec<[f64; 4]>,
    pub pieces: Vec<Mob>,
    /// The cuts (none, one or two).
    pub cuts: Vec<Mob>,
    pub a: u64,
    pub b: u64,
}

impl CutRect {
    /// The rectangle with every piece in `color` (give the pieces their
    /// roles with [`CutRect::role`]). The tens are at the bottom and at the
    /// left, so the block sits on the board's origin. Detached.
    pub fn build(stage: &mut Stage<'_>, board: Board, a: u64, b: u64, color: Srgb) -> Self {
        let parts = by_parts(a, b);
        let (a_tens, b_tens) = (crate::model::parts(a).0 as f64, crate::model::parts(b).0 as f64);
        let mut rects = Vec::new();
        for p in &parts.pieces {
            let (h, w) = (p.height as f64, p.width as f64);
            let y = if (h - a_tens).abs() < 1e-9 && a_tens > 0.0 {
                0.0
            } else {
                a_tens
            };
            let x = if (w - b_tens).abs() < 1e-9 && b_tens > 0.0 {
                0.0
            } else {
                b_tens
            };
            rects.push([x, y, w, h]);
        }
        let pieces = rects.iter().map(|&r| board.rect(stage, r, flat(color, 1.0))).collect();
        let (af, bf) = (a as f64, b as f64);
        let mut cuts = Vec::new();
        if a_tens > 0.0 && a_tens < af {
            cuts.push(board.line(stage, [0.0, a_tens], [bf, a_tens], cut_style()));
        }
        if b_tens > 0.0 && b_tens < bf {
            cuts.push(board.line(stage, [b_tens, 0.0], [b_tens, af], cut_style()));
        }
        Self {
            board,
            rects,
            pieces,
            cuts,
            a,
            b,
        }
    }

    /// The colour of piece `i`: hundreds for tens times tens, the corner for
    /// ones times ones, a strip otherwise.
    pub fn role(&self, i: usize) -> Srgb {
        let [_, _, w, h] = self.rects[i];
        let whole = |x: f64| x >= 10.0 && (x % 10.0).abs() < 1e-9;
        match (whole(w), whole(h)) {
            (true, true) => hundreds(),
            (false, false) => corner(),
            _ => strip(),
        }
    }

    /// The whole rectangle, `[x, y, wide, tall]`.
    pub fn whole(&self) -> [f64; 4] {
        [0.0, 0.0, self.b as f64, self.a as f64]
    }

    /// The centre of piece `i` (scene point).
    pub fn centre(&self, i: usize) -> Vec3 {
        self.board.centre(self.rects[i])
    }
}

// ------------------------------------------------------------ the quiz card

/// Where a quiz card sits when it has the screen to itself.
pub const QUIZ_AT: Vec3 = [0.0, 0.3, 0.0];
/// The card's size at scale 1 (scene units): it then fills the middle of
/// the frame, with room under it for one line of working.
pub const QUIZ_CARD: (f64, f64) = (7.2, 5.2);
/// The ring's radius at scale 1.
const RING_R: f64 = 0.72;
/// The problem's and the answer's type size at scale 1.
pub const QUIZ_PT: f64 = 120.0;
const PAUSE_LINE: &str = "pause for more time";
/// The label of an ordinary quiz, and of one in the mixed round.
pub const YOUR_TURN: &str = "your turn";
pub const WHICH_TRICK: &str = "which trick?";

/// How a quiz card is dressed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuizLook {
    /// The card's centre.
    pub at: Vec3,
    /// The small label at the card's head (none on the cold open).
    pub label: Option<&'static str>,
    /// "pause for more time" under the ring (not on the cold open's card).
    pub pause_line: bool,
    /// Whether the answer lands at zero (13_1's ring empties into silence).
    pub lands: bool,
    /// The card's size against [`QUIZ_CARD`] (1 when it has the screen to
    /// itself; about 0.7 beside a picture).
    pub scale: f64,
}

impl QuizLook {
    /// The usual card at `at`, at `scale`.
    pub fn at(at: Vec3, scale: f64) -> Self {
        Self {
            at,
            label: Some(YOUR_TURN),
            pause_line: true,
            lands: true,
            scale,
        }
    }

    /// The card's width and height.
    pub fn size(&self) -> (f64, f64) {
        (QUIZ_CARD.0 * self.scale, QUIZ_CARD.1 * self.scale)
    }

    /// The type size of the problem and of the answer.
    pub fn pt(&self) -> f64 {
        QUIZ_PT * self.scale
    }
}

/// The "your turn" card (storyboard, "Three kinds of question"): the
/// problem, a ring that drains over the quiz's seconds with the seconds left
/// in its middle, and, at zero, the answer alone where the ring was.
pub struct QuizCard {
    pub quiz: &'static Quiz,
    pub look: QuizLook,
    pub back: Mob,
    pub label: Option<Mob>,
    pub problem: Mob,
    /// The ring's faint track.
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub pause_line: Option<Mob>,
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
    /// The card for the quiz asked on line `id`. Detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, id: &str, look: QuizLook) -> fmn::Result<Self> {
        let quiz = quiz_at(id)?;
        let (at, k) = (look.at, look.scale);
        let (w, h) = look.size();
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18 * k)
            .style(Style::default().fill(panel(), 0.96).stroke(quiz_color(), 3.0, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let label = match look.label {
            Some(words) => Some(kit.text(
                stage,
                words,
                (30.0 * k).max(22.0),
                quiz_color(),
                v(at[0], at[1] + h / 2.0 - 0.45 * k),
            )?),
            None => None,
        };
        let problem = eq(kit, stage, &ask_tex(quiz.ask), look.pt(), v(at[0], at[1] + 0.95 * k))?;
        let room = w - 1.0 * k;
        fit_width(stage, problem, room);
        let ring_at = Self::ring_at(look);
        let radius = RING_R * k;
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(radius)
                .arc_center(ring_at)
                .style(faded_line(Palette::faint(), 8.0 * k, 1.0))
                .build(),
        );
        let n = f64::from(quiz.seconds);
        let ring = stage.arena_mut().add(
            polyline(ring_points(ring_at, radius, 1.0))
                .style(faded_line(quiz_color(), 9.0 * k, 0.0))
                .build(),
        );
        let mut digits = Vec::new();
        for second in 1..=quiz.seconds {
            let d = kit.text(stage, &second.to_string(), 56.0 * k, Palette::ink(), ring_at)?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let pause_line = if look.pause_line {
            Some(kit.text(
                stage,
                PAUSE_LINE,
                (26.0 * k).max(20.0),
                Palette::dim(),
                v(ring_at[0], ring_at[1] - radius - 0.4 * k),
            )?)
        } else {
            None
        };
        let answer = eq(kit, stage, &answer_tex(quiz)?, look.pt(), ring_at)?;
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
                let left = polyline(ring_points(ring_at, radius, now.0 / n)).build();
                let _ = s.set_points(ring, left.points());
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
            problem,
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

    /// Bring the card in with its problem and its full ring, `rt` seconds.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        let mut ins = vec![self.back, self.problem, self.track];
        ins.extend(self.label);
        ins.extend(self.pause_line);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in ins {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        stage.play_prepared(anims)?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        stage.play(tween(self.shown, 1.0, 0.25)?)?;
        Ok(())
    }

    /// The countdown: the narration silent, one mallet note a second, the
    /// ring draining; at zero the ring goes and the answer lands alone in
    /// its place (unless the look says nothing lands), and has the screen
    /// for [`sound::LAND_S`] before the next line.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = f64::from(self.quiz.seconds);
        let wav = sound::countdown(self.quiz.seconds, self.look.lands)?;
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.sonify(stage, n + sound::LAND_S, &label, &wav)?;
        let start = stage.scene().time().to_f64();
        stage.play(tween_linear(self.left, 0.0, n)?)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(tween(self.shown, 0.0, 0.25)?)?);
        let mut out = vec![self.track];
        out.extend(self.pause_line);
        for m in out {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.25))?);
        }
        if self.look.lands {
            let a = fade_in(stage.arena_mut(), self.answer, v(0.0, -0.12), 1.0)?;
            anims.push(stage.prepare(a.rt(0.25))?);
        }
        stage.play_prepared(anims)?;
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.ring);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        wait_until(stage, start + n + sound::LAND_S)
    }

    /// What the card still shows once the countdown is over: its back, its
    /// label, its problem and (if it landed) its answer.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back, self.problem];
        out.extend(self.label);
        if self.look.lands {
            out.push(self.answer);
        }
        out
    }
}

fn quiz_color() -> Srgb {
    quiz()
}

/// A quiz, start to finish: say line `id`, bring the card in as the
/// question itself begins, count down, land the answer. The next beat works
/// the answer in the same place.
pub fn your_turn(kit: &Kit, stage: &mut Stage<'_>, id: &str, look: QuizLook) -> fmn::Result<QuizCard> {
    let line = say(kit, stage, id)?;
    let card = QuizCard::build(kit, stage, id, look)?;
    line.wait_last(stage)?;
    card.show(stage, 0.5)?;
    card.count(kit, stage)?;
    Ok(card)
}

// -------------------------------------------------------- the question card

/// How long a "why?" or "which?" question holds before its answer.
pub const WHY_HOLD_S: f64 = 2.0;

/// A question for the viewer with no countdown (02_4, 06_6, 08_1, 09_5,
/// 10_4): its words on a card edged in the quiz's colour, centred at `at`.
/// Detached, one group.
pub fn why_card(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3) -> fmn::Result<Mob> {
    let lines = wrap(words, 26, 3);
    let line_h = 0.6;
    let top = at[1] + line_h * (lines.len() as f64 - 1.0) / 2.0;
    let mut texts = Vec::new();
    let mut widest: f64 = 0.0;
    for (i, l) in lines.iter().enumerate() {
        let m = kit.text(stage, l, 40.0, Palette::ink(), v(at[0], top - line_h * i as f64))?;
        widest = widest.max(stage.get_width(m));
        texts.push(m);
    }
    let back = Rectangle::new()
        .width(widest + 0.9)
        .height(line_h * lines.len() as f64 + 0.5)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.96).stroke(quiz(), 2.5, 1.0))
        .build()?
        .moved_to(at);
    let mut all = vec![stage.arena_mut().add(back)];
    all.extend(texts);
    group(stage, &all)
}

/// The pause after a question with no countdown: let the question finish,
/// mark the moment for the score, and hold the picture [`WHY_HOLD_S`].
pub fn why_pause(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    pause(stage, WHY_HOLD_S)
}

// -------------------------------------------------------------- number line

/// A number line from `lo` to `hi` between scene x `x0` and `x1` at height
/// `y`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumberLine1 {
    pub x0: f64,
    pub x1: f64,
    pub y: f64,
    pub lo: f64,
    pub hi: f64,
}

impl NumberLine1 {
    pub fn p(&self, value: f64) -> Vec3 {
        let share = (value - self.lo) / (self.hi - self.lo);
        v(self.x0 + (self.x1 - self.x0) * share, self.y)
    }

    /// The line itself. Detached.
    pub fn axis(&self, stage: &mut Stage<'_>) -> Mob {
        stage.arena_mut().add(
            polyline(vec![v(self.x0, self.y), v(self.x1, self.y)])
                .style(line_style(Palette::dim(), 2.5))
                .build(),
        )
    }

    /// A tick at `value`, `h` tall. Detached.
    pub fn tick(&self, stage: &mut Stage<'_>, value: f64, h: f64, color: Srgb) -> Mob {
        let p = self.p(value);
        stage.arena_mut().add(
            polyline(vec![v(p[0], p[1] - h / 2.0), v(p[0], p[1] + h / 2.0)])
                .style(line_style(color, 2.5))
                .build(),
        )
    }

    /// A dot at `value`. Detached.
    pub fn dot(&self, stage: &mut Stage<'_>, value: f64, radius: f64, color: Srgb) -> Mob {
        stage.arena_mut().add(
            Dot::new()
                .point(self.p(value))
                .radius(radius)
                .style(flat(color, 1.0))
                .build(),
        )
    }

    /// `words` under (or, with `above`, over) the line at `value`. Detached.
    #[allow(clippy::too_many_arguments)]
    pub fn label(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        words: &str,
        value: f64,
        size: f64,
        color: Srgb,
        above: bool,
    ) -> fmn::Result<Mob> {
        let p = self.p(value);
        let dy = if above { 0.42 } else { -0.42 };
        kit.text(stage, words, size, color, v(p[0], p[1] + dy))
    }

    /// A bow over the line from `a` to `b`, `lift` high. Detached.
    pub fn bow(&self, stage: &mut Stage<'_>, a: f64, b: f64, lift: f64, style: Style) -> Mob {
        stage
            .arena_mut()
            .add(polyline(bow(self.p(a), self.p(b), lift)).style(style).build())
    }

    /// The stretch from `a` to `b` as a bar `h` tall on the line. Detached.
    pub fn stretch(&self, stage: &mut Stage<'_>, a: f64, b: f64, h: f64, style: Style) -> Mob {
        let (pa, pb) = (self.p(a), self.p(b));
        let quad = [
            v(pa[0], pa[1] - h / 2.0),
            v(pb[0], pb[1] - h / 2.0),
            v(pb[0], pb[1] + h / 2.0),
            v(pa[0], pa[1] + h / 2.0),
        ];
        stage.arena_mut().add(Polygon::new(quad).style(style).build())
    }
}

/// The points of a bow from `a` to `b` that rises `lift` at its middle.
pub fn bow(a: Vec3, b: Vec3, lift: f64) -> Vec<Vec3> {
    (0..=24)
        .map(|k| {
            let t = k as f64 / 24.0;
            v(
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t + lift * 4.0 * t * (1.0 - t),
            )
        })
        .collect()
}

// --------------------------------------------------- the six small pictures

/// The six small pictures (01_7), which return as the tiles of the mixed
/// round (chapters 12 to 14), in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    /// A square whose strip stands against its side: squares ending in 5.
    Strip,
    /// A rectangle cut into four pieces: any product, by parts.
    Parts,
    /// A bill cut in ten: percents from a tenth.
    Bill,
    /// A rectangle with one cut across it: halve and double, round and fix.
    Halve,
    /// A number line with a middle: two numbers the same distance from it.
    Middle,
    /// A square of fifty: squares near 50.
    Fifty,
}

pub const TILES: [TileKind; 6] = [
    TileKind::Strip,
    TileKind::Parts,
    TileKind::Bill,
    TileKind::Halve,
    TileKind::Middle,
    TileKind::Fifty,
];

impl TileKind {
    /// The picture a trick's quiz lights.
    pub fn of(trick: Trick) -> Self {
        match trick {
            Trick::EndsInFive => TileKind::Strip,
            Trick::ByParts => TileKind::Parts,
            Trick::PercentBlocks => TileKind::Bill,
            Trick::HalveAndDouble | Trick::RoundThenFix => TileKind::Halve,
            Trick::Pair => TileKind::Middle,
            Trick::NearFifty => TileKind::Fifty,
        }
    }
}

/// One small picture in its frame.
pub struct Tile {
    pub kind: TileKind,
    pub at: Vec3,
    pub size: f64,
    pub frame: Mob,
    /// The picture's pieces, back to front.
    pub parts: Vec<Mob>,
}

impl Tile {
    /// The picture `kind` in a frame `size` across, centred at `at`.
    /// Detached.
    pub fn build(stage: &mut Stage<'_>, kind: TileKind, at: Vec3, size: f64) -> fmn::Result<Self> {
        let frame = Rectangle::new()
            .width(size)
            .height(size)
            .corner_radius(0.1)
            .style(Style::default().fill(panel(), 0.9).stroke(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(at);
        let frame = stage.arena_mut().add(frame);
        // the picture's own board: 100 units across the frame's inside
        let inside = size * 0.72;
        let board = Board::new(v(at[0] - inside / 2.0, at[1] - inside / 2.0), inside / 100.0);
        let thin = line_style(ground(), 1.5);
        let mut parts = Vec::new();
        match kind {
            TileKind::Strip => {
                parts.push(board.rect(stage, [0.0, 8.0, 87.5, 75.0], flat(hundreds(), 1.0)));
                parts.push(board.rect(stage, [87.5, 8.0, 12.5, 87.5], flat(strip(), 1.0)));
                parts.push(board.rect(stage, [87.5, 83.0, 12.5, 12.5], flat(corner(), 1.0)));
                parts.push(board.line(stage, [87.5, 8.0], [87.5, 83.0], thin));
            }
            TileKind::Parts => {
                parts.push(board.rect(stage, [0.0, 25.0, 80.0, 50.0], flat(hundreds(), 1.0)));
                parts.push(board.rect(stage, [80.0, 25.0, 20.0, 50.0], flat(strip(), 1.0)));
                parts.push(board.rect(stage, [0.0, 75.0, 80.0, 12.0], flat(strip(), 1.0)));
                parts.push(board.rect(stage, [80.0, 75.0, 20.0, 12.0], flat(corner(), 1.0)));
                parts.push(board.line(stage, [80.0, 25.0], [80.0, 87.0], thin));
                parts.push(board.line(stage, [0.0, 75.0], [100.0, 75.0], thin));
            }
            TileKind::Bill => {
                parts.push(board.rect(stage, [0.0, 38.0, 100.0, 24.0], flat(slate(), 1.0)));
                parts.push(board.rect(stage, [0.0, 38.0, 10.0, 24.0], flat(Palette::ink(), 0.9)));
                for k in 1..10 {
                    let x = 10.0 * f64::from(k);
                    parts.push(board.line(stage, [x, 38.0], [x, 62.0], thin));
                }
            }
            TileKind::Halve => {
                parts.push(board.rect(stage, [10.0, 20.0, 80.0, 30.0], flat(hundreds(), 1.0)));
                parts.push(board.rect(stage, [10.0, 50.0, 80.0, 30.0], flat(hundreds(), 0.45)));
                parts.push(board.line(stage, [10.0, 50.0], [90.0, 50.0], thin));
            }
            TileKind::Middle => {
                let line = NumberLine1 {
                    x0: board.p(0.0, 0.0)[0],
                    x1: board.p(100.0, 0.0)[0],
                    y: board.p(0.0, 42.0)[1],
                    lo: 0.0,
                    hi: 100.0,
                };
                parts.push(line.axis(stage));
                parts.push(line.tick(stage, 50.0, 0.22 * size, Palette::ink()));
                let arc = line_style(Palette::dim(), 2.0);
                parts.push(line.bow(stage, 18.0, 50.0, 0.2 * size, arc));
                parts.push(line.bow(stage, 50.0, 82.0, 0.2 * size, arc));
                parts.push(line.dot(stage, 18.0, 0.045 * size, hundreds()));
                parts.push(line.dot(stage, 82.0, 0.045 * size, hundreds()));
            }
            TileKind::Fifty => {
                parts.push(board.rect(stage, [0.0, 5.0, 90.0, 68.0], flat(hundreds(), 1.0)));
                parts.push(board.rect(stage, [79.0, 5.0, 11.0, 79.0], flat(strip(), 1.0)));
                parts.push(board.rect(stage, [79.0, 73.0, 11.0, 11.0], flat(corner(), 1.0)));
                parts.push(board.line(stage, [79.0, 5.0], [79.0, 73.0], thin));
                let outline = board.quad([0.0, 5.0, 90.0, 90.0]);
                parts.push(
                    stage
                        .arena_mut()
                        .add(Polygon::new(outline).style(line_style(Palette::ink(), 2.0)).build()),
                );
            }
        }
        Ok(Self {
            kind,
            at,
            size,
            frame,
            parts,
        })
    }

    /// The frame and the picture, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.frame];
        out.extend(&self.parts);
        out
    }

    /// Light the tile (`on`) or put it out: its frame brightens or fades
    /// and its picture with it. Animations to play together.
    pub fn light(&self, stage: &mut Stage<'_>, on: bool, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let (edge, opacity) = if on {
            (Palette::ink(), 1.0)
        } else {
            (Palette::faint(), 0.3)
        };
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let frame = self
            .frame
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(edge), None, Some(1.0))?;
        anims.push(stage.prepare(frame)?);
        for &m in &self.parts {
            anims.push(stage.prepare(dim(m, opacity, rt)?)?);
        }
        Ok(anims)
    }
}

/// The six tiles in a row centred on x = 0 at height `y`, each `size`
/// across and `gap` apart. Detached.
pub fn tile_row(stage: &mut Stage<'_>, y: f64, size: f64, gap: f64) -> fmn::Result<Vec<Tile>> {
    let step = size + gap;
    let left = -step * (TILES.len() as f64 - 1.0) / 2.0;
    TILES
        .iter()
        .enumerate()
        .map(|(i, &kind)| Tile::build(stage, kind, v(left + step * i as f64, y), size))
        .collect()
}

/// Where the tiles sit along the top of the mixed round (chapters 12 to
/// 14): the row's height, each tile's size and the gap between them.
pub const TILES_TOP: (f64, f64, f64) = (3.05, 1.35, 0.3);

/// The six tiles along the top of the frame, as the mixed round shows them.
/// Detached.
pub fn tiles_top(stage: &mut Stage<'_>) -> fmn::Result<Vec<Tile>> {
    tile_row(stage, TILES_TOP.0, TILES_TOP.1, TILES_TOP.2)
}

// ------------------------------------------- pictures that cross chapters

/// Scene units per one for the pictures on a square of fifty (chapters 09
/// to 11 and 14): the square of fifty is 4 tall, and a strip of a square of
/// fifty-three turns inside the frame.
pub const FIFTY_UNIT: f64 = 0.08;

/// The board the squares near fifty stand on, the same in every chapter.
pub fn fifty_board() -> Board {
    Board::new(FIGURE_AT, FIFTY_UNIT)
}

/// Where the card that waits for thirteen times fourteen sits (03_7 to
/// 05_7): the top right corner.
pub const WAITING_AT: Vec3 = [5.0, 3.3, 0.0];

/// The card left waiting at 03_7 and filled in at 05_7: the square that
/// needs a product the viewer cannot do yet, with a question mark for it
/// (135²: 13 × 14 = ?). `number` ends in five. Returns the card's back and
/// its line of mathematics, both detached; chapters 04 and 05 draw it again
/// where 03_7 leaves it.
pub fn waiting_card(kit: &Kit, stage: &mut Stage<'_>, number: u64) -> fmn::Result<[Mob; 2]> {
    let five = ends_in_five(number).ok_or_else(|| fail(format!("{number} does not end in five")))?;
    let src = format!(r"{}^2:\quad {} \times {} = \;?", five.number, five.n, five.n + 1);
    let math = eq(kit, stage, &src, 38.0, WAITING_AT)?;
    let (w, h) = (stage.get_width(math), stage.get_height(math));
    let back = Rectangle::new()
        .width(w + 0.5)
        .height(h + 0.4)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(WAITING_AT);
    Ok([stage.arena_mut().add(back), math])
}

// ----------------------------------------------------- what is in your head

pub const IN_YOUR_HEAD: &str = "in your head";

/// "in your head": the words over a faint frame `w` by `h` centred at `at`,
/// in which the numbers being held hang. Returns the frame and the words,
/// detached.
pub fn in_your_head(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64) -> fmn::Result<[Mob; 2]> {
    let frame = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.3)
        .style(Style::default().fill(panel(), 0.5).stroke(Palette::faint(), 2.0, 1.0))
        .build()?
        .moved_to(at);
    let frame = stage.arena_mut().add(frame);
    let words = kit.text(
        stage,
        IN_YOUR_HEAD,
        32.0,
        Palette::dim(),
        v(at[0], at[1] + h / 2.0 + 0.36),
    )?;
    Ok([frame, words])
}

// ----------------------------------------------------------- sheet of paper

/// A sum set out as on a sheet of paper (01_4, 13_4): rows of digits in the
/// typewriter face, right-aligned, with a rule under some rows.
pub struct Sheet {
    pub back: Mob,
    pub rows: Vec<Mob>,
    pub rules: Vec<Mob>,
    /// The right edge the rows align to, each row's height (scene), and
    /// the distance from one row to the next.
    pub right: f64,
    pub row_ys: Vec<f64>,
    pub line_h: f64,
}

impl Sheet {
    /// A sheet centred at `at` with `rows` (their text as written) in type
    /// of `size` points and a rule under each row whose index is in
    /// `rules_under`. Detached.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        at: Vec3,
        rows: &[&str],
        rules_under: &[usize],
        size: f64,
    ) -> fmn::Result<Self> {
        let k = size / 34.0;
        let line_h = 0.56 * k;
        let h = line_h * rows.len() as f64 + 0.6 * k;
        let widest = rows.iter().map(|r| r.chars().count()).max().unwrap_or(1) as f64;
        let w = ((widest * 0.3 + 1.0) * k).max(2.4 * k);
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.06)
            .style(flat(paper(), 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let right = at[0] + w / 2.0 - 0.4 * k;
        let top = at[1] + h / 2.0 - 0.3 * k - line_h / 2.0;
        let mut row_mobs = Vec::new();
        let mut row_ys = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            let y = top - line_h * i as f64;
            let m = kit.mono(stage, r, size, paper_ink(), v(right, y))?;
            stage.move_to(m, v(right, y), RIGHT);
            row_mobs.push(m);
            row_ys.push(y);
        }
        let mut rules = Vec::new();
        for &i in rules_under {
            let y = top - line_h * i as f64 - line_h / 2.0;
            let rule = polyline(vec![v(at[0] - w / 2.0 + 0.3 * k, y), v(right + 0.12 * k, y)])
                .style(line_style(paper_ink(), 2.5 * k))
                .build();
            rules.push(stage.arena_mut().add(rule));
        }
        Ok(Self {
            back,
            rows: row_mobs,
            rules,
            right,
            row_ys,
            line_h,
        })
    }

    /// The sheet, its rows and its rules, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back];
        out.extend(&self.rows);
        out.extend(&self.rules);
        out
    }
}

// ------------------------------------------------------------ the calculator

/// A desk calculator in outline (09_2, 13_4): a body, a result window and a
/// keyboard of round keys. A generic machine: the storyboard names the
/// book's word for it and draws nothing of a real model.
pub struct Calculator {
    pub at: Vec3,
    pub scale: f64,
    /// The body, the window and the keys, back to front.
    pub parts: Vec<Mob>,
}

impl Calculator {
    /// A calculator centred at `at`, about `2.6 × scale` wide. Detached.
    pub fn build(stage: &mut Stage<'_>, at: Vec3, scale: f64) -> fmn::Result<Self> {
        let (w, h) = (2.6 * scale, 3.0 * scale);
        let outline = Style::default().fill(panel(), 0.9).stroke(Palette::dim(), 2.5, 1.0);
        let body = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.14 * scale)
            .style(outline)
            .build()?
            .moved_to(at);
        let window = Rectangle::new()
            .width(w - 0.5 * scale)
            .height(0.62 * scale)
            .corner_radius(0.05 * scale)
            .style(Style::default().fill(ground(), 1.0).stroke(Palette::dim(), 1.5, 1.0))
            .build()?
            .moved_to(Self::window_at(at, scale));
        let mut parts = vec![stage.arena_mut().add(body), stage.arena_mut().add(window)];
        for row in 0..4 {
            for col in 0..5 {
                let key = Circle::new()
                    .radius(0.13 * scale)
                    .arc_center(v(
                        at[0] + (f64::from(col) - 2.0) * 0.44 * scale,
                        at[1] + 0.3 * scale - f64::from(row) * 0.44 * scale,
                    ))
                    .style(faded_line(Palette::dim(), 2.0, 0.9))
                    .build();
                parts.push(stage.arena_mut().add(key));
            }
        }
        Ok(Self { at, scale, parts })
    }

    fn window_at(at: Vec3, scale: f64) -> Vec3 {
        v(at[0], at[1] + 0.98 * scale)
    }

    /// The middle of the result window.
    pub fn window(&self) -> Vec3 {
        Self::window_at(self.at, self.scale)
    }

    /// Digits for the result window, right-aligned in it, in the typewriter
    /// face. Detached.
    pub fn reading(&self, kit: &Kit, stage: &mut Stage<'_>, digits: &str, color: Srgb) -> fmn::Result<Mob> {
        let right = v(self.window()[0] + (1.3 - 0.36) * self.scale, self.window()[1]);
        let m = kit.mono(stage, digits, 34.0 * self.scale, color, right)?;
        stage.move_to(m, right, RIGHT);
        Ok(m)
    }
}

// ------------------------------------------------------------- the magnifier

/// A framed enlargement of a small part of a board (08_4, 10_3, 10_6): the
/// corner is 2 or 3 units in 50, too small to read at scale.
pub struct Lens {
    /// The part enlarged, `[x, y, wide, tall]` in the source board's units.
    pub window: [f64; 4],
    /// The same units, enlarged: draw the pieces again on this board (only
    /// what [`Lens::clip`] returns of each).
    pub board: Board,
    /// A small frame round the part on the source, the two rays to the
    /// enlargement, the enlargement's dark back and its frame.
    pub mark: Mob,
    pub rays: [Mob; 2],
    pub back: Mob,
    pub frame: Mob,
}

impl Lens {
    /// Enlarge `window` of `source` by `zoom`, centred at `at`. Detached.
    pub fn build(stage: &mut Stage<'_>, source: Board, window: [f64; 4], at: Vec3, zoom: f64) -> Self {
        let [x, y, w, h] = window;
        let unit = source.unit * zoom;
        let board = Board::new(v(at[0] - (x + w / 2.0) * unit, at[1] - (y + h / 2.0) * unit), unit);
        let edge = line_style(Palette::ink(), 2.0);
        let mark = source.rect(stage, window, line_style(Palette::ink(), 1.5));
        let back = board.rect(stage, window, flat(ground(), 1.0));
        let frame = board.rect(stage, window, edge);
        let thin = faded_line(Palette::dim(), 1.5, 0.8);
        let near = |a: [f64; 2]| source.p(a[0], a[1]);
        let far = |a: [f64; 2]| board.p(a[0], a[1]);
        // the rays join the corners that face each other
        let (top, bottom) = if at[0] >= source.centre(window)[0] {
            ([x + w, y + h], [x + w, y])
        } else {
            ([x, y + h], [x, y])
        };
        let opposite = |a: [f64; 2]| if a[0] > x { [x, a[1]] } else { [x + w, a[1]] };
        let rays = [top, bottom].map(|a| {
            stage
                .arena_mut()
                .add(polyline(vec![near(a), far(opposite(a))]).style(thin).build())
        });
        Self {
            window,
            board,
            mark,
            rays,
            back,
            frame,
        }
    }

    /// The part of rectangle `r` inside the window, if any.
    pub fn clip(&self, r: [f64; 4]) -> Option<[f64; 4]> {
        let [x, y, w, h] = self.window;
        let (x0, y0) = (r[0].max(x), r[1].max(y));
        let (x1, y1) = ((r[0] + r[2]).min(x + w), (r[1] + r[3]).min(y + h));
        (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
    }

    /// What frames the enlargement, back to front (draw the enlarged pieces
    /// between [`Lens::back`] and [`Lens::frame`]).
    pub fn mobs(&self) -> [Mob; 5] {
        [self.mark, self.rays[0], self.rays[1], self.back, self.frame]
    }
}

// --------------------------------------------------------------------- cards

/// A small tag: a few words on a rounded back, beside the thing it names
/// ("about", "running total"). Detached, one group.
pub fn tag(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3, size: f64) -> fmn::Result<Mob> {
    let t = kit.text(stage, words, size, Palette::dim(), at)?;
    let (w, h) = (stage.get_width(t), stage.get_height(t));
    let back = Rectangle::new()
        .width(w + 0.3)
        .height(h + 0.22)
        .corner_radius(0.09)
        .style(Style::default().fill(panel(), 0.9).stroke(Palette::faint(), 1.2, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group(stage, &[back, t])
}

/// The one caption slot under the stage: detached text.
pub fn caption(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(stage, words, 26.0, Palette::dim(), v(0.0, CAPTION_Y))
}

/// A lower-third whose text is one line of mathematics (the kit's
/// `lower_third` sets plain words): a `label` in the hero role and the TeX
/// beside it, on the same band at the foot of the frame. Detached, one
/// group.
pub fn lower_third_tex(kit: &Kit, stage: &mut Stage<'_>, label: &str, src: &str) -> fmn::Result<Mob> {
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let h = 0.78;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let left = -w / 2.0 + 0.25;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(centre);
    let back = stage.arena_mut().add(back);
    let tag = kit.text_left(stage, label, 26.0, Palette::hero(), v(left, centre[1]))?;
    let indent = left + stage.get_width(tag) + 0.3;
    let math = kit.tex_left(stage, src, 34.0, Palette::ink(), &[], v(indent, centre[1]))?;
    let room = w / 2.0 - 0.25 - indent;
    if stage.get_width(math) > room {
        stage.set_width(math, room, false);
        stage.move_to(math, v(indent, centre[1]), LEFT);
    }
    group(stage, &[back, tag, math])
}

/// A verbatim quote on a card, wrapped to `chars` a line, with its source
/// beneath on up to two lines. It never drops words: text that does not fit
/// in `max_lines` is an error, not an ellipsis. The card's back, the quote's
/// lines and the source's lines, all detached.
pub struct QuoteCard {
    pub back: Mob,
    pub lines: Vec<Mob>,
    pub source: Vec<Mob>,
}

impl QuoteCard {
    /// `source` is one or two lines, given as they are to be set.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        words: &str,
        source: &[&str],
        at: Vec3,
        chars: usize,
        max_lines: usize,
    ) -> fmn::Result<Self> {
        let lines = wrap(words, chars, max_lines + 1);
        if lines.len() > max_lines {
            return Err(fail(format!(
                "a quote of {} characters does not fit {max_lines} lines of {chars}",
                words.chars().count()
            )));
        }
        if source.len() > 2 {
            return Err(fail("a quote's source is at most two lines".into()));
        }
        let (line_h, src_h) = (0.5, 0.34);
        let h = 0.6 + line_h * lines.len() as f64 + 0.25 + src_h * source.len() as f64;
        let top = at[1] + h / 2.0 - 0.35 - line_h / 2.0;
        let mut quote = Vec::new();
        let mut widest: f64 = 0.0;
        for (i, l) in lines.iter().enumerate() {
            let m = kit.text_styled(
                stage,
                l,
                32.0,
                Palette::ink(),
                v(at[0], top - line_h * i as f64),
                false,
                true,
            )?;
            widest = widest.max(stage.get_width(m));
            quote.push(m);
        }
        let src_top = top - line_h * lines.len() as f64 - 0.1;
        let mut source_mobs = Vec::new();
        for (i, l) in source.iter().enumerate() {
            let m = kit.text(stage, l, 20.0, Palette::dim(), v(at[0], src_top - src_h * i as f64))?;
            widest = widest.max(stage.get_width(m));
            source_mobs.push(m);
        }
        let back = Rectangle::new()
            .width(widest + 0.9)
            .height(h)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
            .build()?
            .moved_to(at);
        Ok(Self {
            back: stage.arena_mut().add(back),
            lines: quote,
            source: source_mobs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::one_move;

    // ΔE2000 (Sharma, Wu and Dalal 2005), for the colour-role test.
    fn lab(c: Srgb) -> [f64; 3] {
        let lin = |u: f64| {
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
        let x = (0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b) / 0.950_47;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
        let z = (0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b) / 1.088_83;
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (24389.0 / 27.0 * t + 16.0) / 116.0
            }
        };
        let (fx, fy, fz) = (f(x), f(y), f(z));
        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }

    fn de2000(a: [f64; 3], b: [f64; 3]) -> f64 {
        let (c1, c2) = (a[1].hypot(a[2]), b[1].hypot(b[2]));
        let cb = (c1 + c2) / 2.0;
        let g = 0.5 * (1.0 - (cb.powi(7) / (cb.powi(7) + 25f64.powi(7))).sqrt());
        let (a1, a2) = ((1.0 + g) * a[1], (1.0 + g) * b[1]);
        let (c1p, c2p) = (a1.hypot(a[2]), a2.hypot(b[2]));
        let hue = |y: f64, x: f64| {
            if x == 0.0 && y == 0.0 {
                0.0
            } else {
                y.atan2(x).to_degrees().rem_euclid(360.0)
            }
        };
        let (h1, h2) = (hue(a[2], a1), hue(b[2], a2));
        let (dl, dc) = (b[0] - a[0], c2p - c1p);
        let mut dh = h2 - h1;
        if c1p * c2p == 0.0 {
            dh = 0.0;
        } else if dh > 180.0 {
            dh -= 360.0;
        } else if dh < -180.0 {
            dh += 360.0;
        }
        let dh_big = 2.0 * (c1p * c2p).sqrt() * (dh / 2.0).to_radians().sin();
        let (lb, cbp) = ((a[0] + b[0]) / 2.0, (c1p + c2p) / 2.0);
        let hs = h1 + h2;
        let hb = if c1p * c2p == 0.0 {
            hs
        } else if (h1 - h2).abs() <= 180.0 {
            hs / 2.0
        } else if hs < 360.0 {
            (hs + 360.0) / 2.0
        } else {
            (hs - 360.0) / 2.0
        };
        let t = 1.0 - 0.17 * (hb - 30.0).to_radians().cos()
            + 0.24 * (2.0 * hb).to_radians().cos()
            + 0.32 * (3.0 * hb + 6.0).to_radians().cos()
            - 0.20 * (4.0 * hb - 63.0).to_radians().cos();
        let dth = 30.0 * (-((hb - 275.0) / 25.0).powi(2)).exp();
        let rc = 2.0 * (cbp.powi(7) / (cbp.powi(7) + 25f64.powi(7))).sqrt();
        let sl = 1.0 + 0.015 * (lb - 50.0).powi(2) / (20.0 + (lb - 50.0).powi(2)).sqrt();
        let (sc, sh) = (1.0 + 0.045 * cbp, 1.0 + 0.015 * cbp * t);
        let rt = -(2.0 * dth).to_radians().sin() * rc;
        ((dl / sl).powi(2) + (dc / sc).powi(2) + (dh_big / sh).powi(2) + rt * (dc / sc) * (dh_big / sh)).sqrt()
    }

    #[test]
    fn de2000_matches_sharma_wu_dalal() {
        // the paper's first test pair
        let d = de2000([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485]);
        assert!((d - 2.0425).abs() < 1e-3, "{d}");
    }

    /// Storyboard, "Color roles": every pair of the roles, with the
    /// background, is at least ΔE2000 24 apart.
    #[test]
    fn the_role_colours_are_told_apart() {
        let roles = [
            ("hundreds", hundreds()),
            ("strip", strip()),
            ("corner", corner()),
            ("the move", mover()),
            ("taken away", gone()),
            ("quiz", quiz()),
            ("ink", Palette::ink()),
            ("background", ground()),
        ];
        let mut least = f64::INFINITY;
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 24.0, "{na} and {nb}: ΔE2000 {d:.1}");
                least = least.min(d);
            }
        }
        assert!(
            (least - 24.2).abs() < 0.1,
            "the storyboard says the least is 24.2: {least:.2}"
        );
    }

    fn inside(p: [f64; 2], r: [f64; 4], shrink: f64) -> bool {
        p[0] > r[0] + shrink && p[0] < r[0] + r[2] - shrink && p[1] > r[1] + shrink && p[1] < r[1] + r[3] - shrink
    }

    /// The strip is the same rigid rectangle all the way, it starts and ends
    /// where the picture says, and on the way no part of it passes through
    /// the body it turns round.
    #[test]
    fn the_strip_turns_as_one_rigid_piece_and_clears_the_body() {
        for (side, b) in [(35, 5), (25, 5), (48, 2), (53, 3), (50, 3), (46, 4), (20, 1)] {
            let m = one_move(side, b).unwrap();
            let turn = Turn::of(m);
            let (s, d) = (side as f64, b as f64);
            let body = [0.0, 0.0, s, s - d];
            let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9;
            let flat = turn.quad(0.0);
            assert!(
                near(flat[0], [0.0, s - d]) && near(flat[2], [s, s]),
                "{side}: flat {flat:?}"
            );
            let up = turn.quad(1.0);
            let (xs, ys): (Vec<f64>, Vec<f64>) = up.iter().map(|p| (p[0], p[1])).unzip();
            let (x0, x1) = (
                xs.iter().copied().fold(f64::MAX, f64::min),
                xs.iter().copied().fold(f64::MIN, f64::max),
            );
            let (y0, y1) = (
                ys.iter().copied().fold(f64::MAX, f64::min),
                ys.iter().copied().fold(f64::MIN, f64::max),
            );
            assert!(
                (x0 - s).abs() < 1e-9 && (x1 - s - d).abs() < 1e-9 && y0.abs() < 1e-9 && (y1 - s).abs() < 1e-9,
                "{side}: standing {up:?}"
            );
            for k in 0..=200 {
                let q = turn.quad(f64::from(k) / 200.0);
                for i in 0..4 {
                    let (p, n) = (q[i], q[(i + 1) % 4]);
                    let len = (p[0] - n[0]).hypot(p[1] - n[1]);
                    let want = if i % 2 == 0 { s } else { d };
                    assert!((len - want).abs() < 1e-9, "{side} at {k}: side {i} is {len}");
                    for j in 0..=40 {
                        let u = f64::from(j) / 40.0;
                        let at = [p[0] + (n[0] - p[0]) * u, p[1] + (n[1] - p[1]) * u];
                        assert!(!inside(at, body, 1e-6), "{side} at {k}: {at:?} is inside the body");
                    }
                }
            }
        }
    }

    #[test]
    fn a_problem_never_carries_a_bare_percent_or_dollar() {
        for q in QUIZZES {
            let mut all = ask_tex(q.ask);
            all.push_str(&answer_tex(q).unwrap());
            let chars: Vec<char> = all.chars().collect();
            for (i, &c) in chars.iter().enumerate() {
                if c == '%' || c == '$' {
                    assert!(i > 0 && chars[i - 1] == '\\', "{}: {all}", q.line);
                }
            }
        }
        assert_eq!(ask_tex(Ask::PercentOfMoney(15, 6000)), "15\\% \\text{ of } \\$60");
        assert_eq!(ask_tex(Ask::TimesMoney(3, 499)), "3 \\times \\$4.99");
        assert_eq!(answer_tex(quiz_at("06_4").unwrap()).unwrap(), "\\$9.00");
        assert_eq!(answer_tex(quiz_at("03_3").unwrap()).unwrap(), "11025");
    }

    #[test]
    fn a_phrase_is_timed_inside_its_own_sentence() {
        let text = "One two. Three four five. Six?";
        assert_eq!(sentence_offsets(text), [0, 9, 26]);
        let line = Spoken {
            id: "x".into(),
            start: 10.0,
            len: 6.0,
            text: text.into(),
            sentences: vec![(0, 0.0), (9, 1.0), (26, 5.0)],
        };
        assert!((line.sentence(1).unwrap() - 11.0).abs() < 1e-9);
        // "four" is 6 of the 17 characters of the second sentence, which runs 4 s
        assert!((line.at("four").unwrap() - (11.0 + 4.0 * 6.0 / 17.0)).abs() < 1e-9);
        assert!((line.at("Six").unwrap() - 15.0).abs() < 1e-9);
        assert!(line.at("seven").is_err());
    }

    #[test]
    fn the_cut_rectangle_puts_the_block_on_the_origin_and_gives_each_piece_its_role() {
        let (a, b) = crate::facts::shown::PARTS;
        let parts = by_parts(a, b);
        assert_eq!(parts.pieces.len(), 4);
        // the rectangles the builder would draw (no stage needed for the maths)
        let rects: Vec<[f64; 4]> = vec![
            [0.0, 0.0, 40.0, 20.0],
            [40.0, 0.0, 6.0, 20.0],
            [0.0, 20.0, 40.0, 3.0],
            [40.0, 20.0, 6.0, 3.0],
        ];
        for (p, r) in parts.pieces.iter().zip(&rects) {
            assert_eq!((p.width as f64, p.height as f64), (r[2], r[3]));
        }
        let area: f64 = rects.iter().map(|r| r[2] * r[3]).sum();
        assert_eq!(area as u64, parts.product());
    }

    #[test]
    fn every_trick_has_its_tile() {
        for q in QUIZZES {
            assert!(TILES.contains(&TileKind::of(q.trick)), "{}", q.line);
        }
    }
}
