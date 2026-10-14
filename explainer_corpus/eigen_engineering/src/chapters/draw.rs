//! Video K's look, shared by every chapter (storyboard §3 "Color roles"):
//! the eight role colours, line timing against the voice, the facts as the
//! scenes read them, the plane and its arrows, the grid of numbers, the "your
//! turn" card with its countdown, and the harness that constructs a real
//! chapter in a test.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no outline
//! on big filled shapes, small groups, and what moves is rewritten by one
//! updater on one anchor, only when a tracker has moved.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::f64::consts::{FRAC_PI_2, TAU};
use std::sync::OnceLock;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Facts, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::prelude::*;

use crate::model::linalg::Mat;
use crate::model::{QUIZZES, Quiz};

pub use explainer_kit::animatic::HOLD_S;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------------- facts

/// Every fact of the video, computed once per process (the edge tests behind
/// them are not cheap).
fn all_facts() -> fmn::Result<&'static Facts> {
    static FACTS: OnceLock<Result<Facts, String>> = OnceLock::new();
    FACTS
        .get_or_init(crate::facts::facts)
        .as_ref()
        .map_err(|e| fail(format!("the facts do not compute: {e}")))
}

/// A fact as it is printed for the oracle.
pub fn fact(key: &str) -> fmn::Result<&'static str> {
    all_facts()?
        .get(key)
        .ok_or_else(|| fail(format!("no fact {key} (docs/fact-sheet.md)")))
}

fn parse_number(key: &str, s: &str) -> fmn::Result<f64> {
    s.parse::<f64>()
        .map_err(|e| fail(format!("fact {key}: {s:?} is not a number: {e}")))
}

/// A fact that is a row of numbers (`2,1`).
pub fn numbers(key: &str) -> fmn::Result<Vec<f64>> {
    fact(key)?.split(',').map(|s| parse_number(key, s)).collect()
}

/// A fact that is a row of words or exact fractions (`1/2,1/6`), as written.
pub fn items(key: &str) -> fmn::Result<Vec<&'static str>> {
    Ok(fact(key)?.split(',').collect())
}

/// A fact that is a matrix (`2,-1;-1,2`): its rows.
pub fn matrix(key: &str) -> fmn::Result<Vec<Vec<f64>>> {
    fact(key)?
        .split(';')
        .map(|row| {
            row.split(',')
                .map(|s| parse_number(key, s))
                .collect::<fmn::Result<Vec<f64>>>()
        })
        .collect()
}

/// `x` to `places` decimals, as `facts::fixed` prints it.
pub fn show(x: f64, places: usize) -> fmn::Result<String> {
    crate::facts::fixed(x, places).map_err(fail)
}

/// A whole number as TeX prints it (no `-0`).
pub fn whole(x: f64) -> String {
    let r = x.round();
    if r == 0.0 {
        "0".to_owned()
    } else {
        format!("{}", r as i64)
    }
}

/// The entries of a matrix fact as TeX strings: whole numbers as they are,
/// the others to `places` decimals.
pub fn entries(key: &str, places: Option<usize>) -> fmn::Result<Vec<Vec<String>>> {
    matrix(key)?
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|x| match places {
                    Some(p) => show(x, p),
                    None => Ok(whole(x)),
                })
                .collect::<fmn::Result<Vec<String>>>()
        })
        .collect()
}

/// The minus a number is printed with in plain text: the engine's live
/// numbers (`DecimalNumber`) show a negative with an en dash at mid-digit
/// height, and the text face has no U+2212 of its own (it falls back to a
/// heavier face), so a static number uses the same character and sits beside
/// a live one alike. TeX sets its own minus from `-`.
pub const MINUS: char = '\u{2013}';

/// `s` with each ASCII hyphen-minus written as the engine's minus
/// ([`MINUS`]), for plain text.
pub fn minus(s: &str) -> String {
    s.replace('-', &MINUS.to_string())
}

/// An exact fraction as a fact prints it (`1/6`, `-1/2`, `2`) in TeX:
/// `\tfrac{1}{6}`, `-\tfrac{1}{2}`, `2`.
pub fn fraction_tex(s: &str) -> String {
    match s.split_once('/') {
        Some((n, d)) => match n.strip_prefix('-') {
            Some(n) => format!("-\\tfrac{{{n}}}{{{d}}}"),
            None => format!("\\tfrac{{{n}}}{{{d}}}"),
        },
        None => s.to_owned(),
    }
}

/// The value of a fraction fact item (`1/6`, `2/3`, `1`).
pub fn fraction_value(s: &str) -> fmn::Result<f64> {
    match s.split_once('/') {
        Some((n, d)) => {
            let (n, d) = (parse_number(s, n)?, parse_number(s, d)?);
            Ok(n / d)
        }
        None => parse_number(s, s),
    }
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the words it illustrates. A
/// phrase's moment is its share of its sentence's characters (the voice reads
/// at a nearly even rate); the cue sheets of the narrated draft check the
/// result.
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

/// How long a prediction pause without a ring holds the picture (the kit's).
pub const PREDICT_HOLD_S: f64 = explainer_kit::props::PREDICT_HOLD_S;

/// A prediction pause with no ring (the storyboard's eight plain pauses):
/// let the question finish, mark the moment for the score, hold the picture.
pub fn predict_hold(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    pause(stage, PREDICT_HOLD_S)
}

/// A `hold` beat: the music alone for [`HOLD_S`] after the line.
pub fn music_hold(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.music_breath(stage, HOLD_S)
}

// ------------------------------------------------------------ colour roles

/// What goes into the rule: a move of carts or joints, a mix of goods, the
/// amounts made.
pub fn moves() -> Srgb {
    Palette::blue()
}

/// What the rule gives back: the pushes, what making things uses up.
pub fn pushes() -> Srgb {
    hex("#F2994A")
}

/// An own pattern, where the move and its push line up: an eigenvector, a
/// mode (the corpus's hero gold).
pub fn own() -> Srgb {
    Palette::hero()
}

/// Squeezed: a member in compression, air pressed together.
pub fn squeezed() -> Srgb {
    hex("#E0408F")
}

/// Stretched: a member in tension, air thinned out.
pub fn stretched() -> Srgb {
    hex("#26867C")
}

/// The "your turn" card and its countdown. A card's fill, never a line or a
/// word beside `dim` text (15.4 from it).
pub fn quiz() -> Srgb {
    hex("#B189D6")
}

/// Holds, allowed, can feed itself.
pub fn good() -> Srgb {
    Palette::green()
}

/// Fails, not allowed.
pub fn warn() -> Srgb {
    Palette::red()
}

/// A card's or panel's back.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// The picture's background.
pub fn ground() -> Srgb {
    hex(BACKGROUND)
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

/// An empty parent in the arena holding mobjects already there.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
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
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade every top-level mobject except `keep` out, `rt` seconds.
pub fn fade_all_but(stage: &mut Stage<'_>, keep: &[Mob], rt: f64) -> fmn::Result<()> {
    let leaving: Vec<Mob> = stage
        .scene()
        .mobjects()
        .iter()
        .copied()
        .filter(|m| !keep.contains(m))
        .collect();
    fade_out_all(stage, &leaving, rt)
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

/// A fade-in as an animation to play with others (`stage.prepare` of
/// `fade_in(stage.arena_mut(), ..)` is a double borrow).
pub fn fade_in_anim(stage: &mut Stage<'_>, m: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// A fade-out as an animation to play with others, its updaters stopped.
pub fn fade_out_anim(stage: &mut Stage<'_>, m: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    stage.clear_updaters(m, true);
    let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
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

/// A tracker animated at a constant rate (a clock, a ring).
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

/// `rt` seconds, for an `animate()` chain.
pub fn run(rt: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    }
}

// ----------------------------------------------------------- text and math

/// A line of mathematics in ink, centred at `at`.
pub fn eq(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, at: Vec3) -> fmn::Result<Mob> {
    kit.tex(stage, src, size, Palette::ink(), &[], at)
}

/// A line of mathematics in `color`, centred at `at`.
pub fn tex_c(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    kit.tex(stage, src, size, color, &[], at)
}

/// Shrink `mob` about its centre until it is at most `room` wide.
pub fn fit_width(stage: &mut Stage<'_>, mob: Mob, room: f64) {
    if stage.get_width(mob) > room {
        let at = stage.get_center(mob);
        stage.set_width(mob, room, false);
        stage.move_to(mob, at, ORIGIN);
    }
}

/// A small label in the dim role, centred at `at`.
pub fn label(kit: &Kit, stage: &mut Stage<'_>, words: &str, at: Vec3) -> fmn::Result<Mob> {
    kit.text(stage, words, 28.0, Palette::dim(), at)
}

/// Words on a dark plate edged in `edge`, so they read over a picture
/// (detached, one group), centred at `at`.
pub fn chip(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, edge: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let text = kit.text(stage, words, size, Palette::ink(), at)?;
    let (w, h) = (stage.get_width(text), stage.get_height(text));
    let back = Rectangle::new()
        .width(w + 0.4)
        .height(h + 0.3)
        .corner_radius(0.1)
        .style(Style::default().fill(ground(), 0.92).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group(stage, &[back, text])
}

// ------------------------------------------------------------- the plane

/// Maths coordinates on the screen: `unit` scene units to one unit of the
/// plane, the origin at `centre`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub centre: Vec3,
    pub unit: f64,
}

impl Plane {
    /// Where the point `(x, y)` of the plane sits on screen.
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        v(self.centre[0] + self.unit * x, self.centre[1] + self.unit * y)
    }

    /// The two axes as faint lines reaching `half_w` and `half_h` plane units
    /// either way, detached.
    pub fn axes(&self, stage: &mut Stage<'_>, half_w: f64, half_h: f64) -> Vec<Mob> {
        let style = faded_line(Palette::faint(), 2.0, 1.0);
        let across = polyline(vec![self.p(-half_w, 0.0), self.p(half_w, 0.0)])
            .style(style)
            .build();
        let up = polyline(vec![self.p(0.0, -half_h), self.p(0.0, half_h)])
            .style(style)
            .build();
        vec![stage.arena_mut().add(across), stage.arena_mut().add(up)]
    }
}

/// A 2 by 2 table as a map of the plane, rows.
pub type Map2 = [[f64; 2]; 2];

/// A model matrix (2 by 2) as a map.
pub fn map2(m: &Mat) -> Map2 {
    [[m.at(0, 0), m.at(0, 1)], [m.at(1, 0), m.at(1, 1)]]
}

/// The map applied to a point of the plane.
pub fn apply(m: &Map2, p: [f64; 2]) -> [f64; 2] {
    [m[0][0] * p[0] + m[0][1] * p[1], m[1][0] * p[0] + m[1][1] * p[1]]
}

/// An arrow as one open polyline: the shaft, then the two barbs of the head
/// (tail, tip, left barb, tip, right barb). The head is at most `head` long
/// and at most 45% of the arrow; a zero-length arrow is a speck.
pub fn arrow_points(tail: Vec3, tip: Vec3, head: f64) -> Vec<Vec3> {
    let (dx, dy) = (tip[0] - tail[0], tip[1] - tail[1]);
    let len = dx.hypot(dy);
    if len < 1e-6 {
        return vec![tail, v(tail[0] + 1e-4, tail[1])];
    }
    let (ux, uy) = (dx / len, dy / len);
    let h = head.min(0.45 * len);
    let back = [tip[0] - ux * h, tip[1] - uy * h];
    let (nx, ny) = (-uy * h * 0.45, ux * h * 0.45);
    vec![
        tail,
        tip,
        v(back[0] + nx, back[1] + ny),
        tip,
        v(back[0] - nx, back[1] - ny),
    ]
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

/// A static arrow with a filled head, detached.
pub fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, thickness: f64) -> fmn::Result<Mob> {
    let a = Arrow::new(from, to)
        .buff(0.0)
        .thickness(thickness)
        .style(Style::default().fill(color, 1.0).stroke(color, thickness, 1.0))
        .build()?;
    Ok(stage.arena_mut().add(a))
}

// ------------------------------------------------------- the grid prop

/// A grid of numbers with its brackets drawn: one TeX string per cell, so it
/// is the table of chapter 02 at a large size and sits in a corner at a small
/// one.
pub struct MatrixGrid {
    /// Cells and brackets, one group.
    pub group: Mob,
    /// The cells, row by row.
    pub cells: Vec<Vec<Mob>>,
    pub brackets: [Mob; 2],
    /// The grid's centre when it was built.
    pub at: Vec3,
    /// How far apart the rows are.
    pub pitch: f64,
    /// Width and height, brackets included.
    pub size: (f64, f64),
}

impl MatrixGrid {
    /// A grid of TeX `entries` (rows of equal length) at font `size`, in
    /// `color`, centred at `at`. Detached.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        entries: &[Vec<String>],
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Self> {
        let rows = entries.len();
        let cols = entries.first().map_or(0, Vec::len);
        if rows == 0 || cols == 0 || entries.iter().any(|r| r.len() != cols) {
            return Err(fail("a grid needs rows of one length".into()));
        }
        // a digit's height at this size, and the grid's rhythm from it
        let digit = 0.33 * size / 48.0;
        let pitch = digit * 2.4;
        let gap = digit * 1.7;
        let mut cells: Vec<Vec<Mob>> = Vec::new();
        let mut widths = vec![0.0_f64; cols];
        for row in entries {
            let mut mobs = Vec::new();
            for (c, s) in row.iter().enumerate() {
                let m = kit.tex(stage, s, size, color, &[], ORIGIN)?;
                widths[c] = widths[c].max(stage.get_width(m));
                mobs.push(m);
            }
            cells.push(mobs);
        }
        let total = widths.iter().sum::<f64>() + gap * (cols as f64 - 1.0);
        let left = at[0] - total / 2.0;
        let mut x = left;
        for (c, w) in widths.iter().enumerate() {
            for (r, row) in cells.iter().enumerate() {
                let y = at[1] + pitch * ((rows as f64 - 1.0) / 2.0 - r as f64);
                stage.move_to(row[c], v(x + w / 2.0, y), ORIGIN);
            }
            x += w + gap;
        }
        let half = pitch * rows as f64 / 2.0;
        let pad = gap * 0.55;
        let serif = digit * 0.7;
        let (xl, xr) = (left - pad, left + total + pad);
        let (top, bottom) = (at[1] + half, at[1] - half);
        let style = || faded_line(color, 4.0, 1.0);
        let lb = polyline(vec![
            v(xl + serif, top),
            v(xl, top),
            v(xl, bottom),
            v(xl + serif, bottom),
        ])
        .style(style())
        .build();
        let rb = polyline(vec![
            v(xr - serif, top),
            v(xr, top),
            v(xr, bottom),
            v(xr - serif, bottom),
        ])
        .style(style())
        .build();
        let (lb, rb) = (stage.arena_mut().add(lb), stage.arena_mut().add(rb));
        let mut all: Vec<Mob> = cells.iter().flatten().copied().collect();
        all.extend([lb, rb]);
        let g = group(stage, &all)?;
        Ok(Self {
            group: g,
            cells,
            brackets: [lb, rb],
            at,
            pitch,
            size: (xr - xl, 2.0 * half),
        })
    }

    /// The scene point of the cell at `row`, `col` right now.
    pub fn cell_at(&self, stage: &Stage<'_>, row: usize, col: usize) -> Vec3 {
        stage.get_center(self.cells[row][col])
    }

    /// A translucent band over row `row` (lit while that row is worked),
    /// detached.
    pub fn row_band(&self, stage: &mut Stage<'_>, row: usize, color: Srgb) -> fmn::Result<Mob> {
        let y = stage.get_center(self.cells[row][0])[1];
        let x = stage.get_center(self.group)[0];
        let band = Rectangle::new()
            .width(self.size.0 + 0.2)
            .height(self.pitch * 0.92)
            .corner_radius(0.08)
            .style(flat(color, 0.2))
            .build()?
            .moved_to(v(x, y));
        Ok(stage.arena_mut().add(band))
    }

    /// A translucent band over column `col`, detached.
    pub fn column_band(&self, stage: &mut Stage<'_>, col: usize, color: Srgb) -> fmn::Result<Mob> {
        let x = stage.get_center(self.cells[0][col])[0];
        let y = stage.get_center(self.group)[1];
        let band = Rectangle::new()
            .width(stage.get_width(self.cells[0][col]) + 0.35)
            .height(self.size.1 + 0.1)
            .corner_radius(0.08)
            .style(flat(color, 0.2))
            .build()?
            .moved_to(v(x, y));
        Ok(stage.arena_mut().add(band))
    }

    /// A rounded outline round the whole grid, in `color`, detached.
    pub fn outline(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        let at = stage.get_center(self.group);
        let frame = Rectangle::new()
            .width(self.size.0 + 0.3)
            .height(self.size.1 + 0.3)
            .corner_radius(0.12)
            .style(faded_line(color, 3.0, 1.0))
            .build()?
            .moved_to(at);
        Ok(stage.arena_mut().add(frame))
    }
}

/// A cross (the engine's faces have no ✗): two strokes of `color` across a
/// square `half` either way of `at`, detached, one group.
pub fn cross_mark(stage: &mut Stage<'_>, at: Vec3, half: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let style = || faded_line(color, width, 1.0);
    let a = polyline(vec![v(at[0] - half, at[1] + half), v(at[0] + half, at[1] - half)])
        .style(style())
        .build();
    let b = polyline(vec![v(at[0] - half, at[1] - half), v(at[0] + half, at[1] + half)])
        .style(style())
        .build();
    let (a, b) = (stage.arena_mut().add(a), stage.arena_mut().add(b));
    group(stage, &[a, b])
}

/// A tick (the engine's faces have no ✔): a short stroke down and a long one
/// up, `size` across, in `color`, detached.
pub fn tick_mark(stage: &mut Stage<'_>, at: Vec3, size: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let t = polyline(vec![
        v(at[0] - 0.5 * size, at[1] - 0.05 * size),
        v(at[0] - 0.15 * size, at[1] - 0.4 * size),
        v(at[0] + 0.5 * size, at[1] + 0.45 * size),
    ])
    .style(faded_line(color, width, 1.0))
    .build();
    Ok(stage.arena_mut().add(t))
}

// ------------------------------------------------------------ the quiz card

/// How a quiz card is dressed.
pub const YOUR_TURN: &str = "your turn";
const PAUSE_LINE: &str = "pause for more time";
/// The ring's radius.
const RING_R: f64 = 0.5;
/// The strip along the card's foot that holds the ring and its note.
const FOOT: f64 = 1.5;

/// The quiz asked on line `id`.
pub fn quiz_at(id: &str) -> fmn::Result<&'static Quiz> {
    QUIZZES
        .iter()
        .find(|q| q.line == id)
        .ok_or_else(|| fail(format!("line {id} asks no quiz in model::QUIZZES")))
}

/// What is left of a circle of radius `r` at `c`, drained clockwise from the
/// top.
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

/// The "your turn" card (storyboard, "The 'your turn' device"): a violet
/// card with the problem above, along its foot "pause for more time" and a
/// ring that drains over the quiz's seconds with the seconds left in its
/// middle, the narration silent and the music alone; at zero the ring goes
/// and the answer lands. The problem is the chapter's own picture, placed in
/// [`QuizCard::content`]; the answer is the `K-Q.<line>.*` facts, built by
/// the chapter and handed to [`QuizCard::land`].
pub struct QuizCard {
    pub quiz: &'static Quiz,
    pub at: Vec3,
    pub size: (f64, f64),
    pub back: Mob,
    pub label: Mob,
    /// The ring's faint track.
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub pause_line: Mob,
    ring_at: Vec3,
    /// Seconds left, whether the countdown is running, and how much of the
    /// ring and its digit shows.
    left: Mob,
    running: Mob,
    shown: Mob,
    anchor: Mob,
    /// The chapter key the card was built under, to tell the runs of the
    /// section 5 audit apart (tests only).
    #[cfg(test)]
    chapter: String,
}

impl QuizCard {
    /// Note when this card did `kind` (tests only: the section 5 audit).
    #[cfg(test)]
    fn log(&self, kind: &'static str, stage: &Stage<'_>) {
        quiz_log::record(&self.chapter, self.quiz.line, kind, stage.scene().time().to_f64());
    }

    /// The card for the quiz asked on line `id`, centred at `at`. Detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, id: &str, at: Vec3, size: (f64, f64)) -> fmn::Result<Self> {
        let quiz = quiz_at(id)?;
        let (w, h) = size;
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18)
            .style(Style::default().fill(panel(), 0.96).stroke(self::quiz(), 3.0, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let label = kit.text(stage, YOUR_TURN, 30.0, self::quiz(), v(at[0], at[1] + h / 2.0 - 0.42))?;
        let ring_at = v(at[0] + w / 2.0 - 0.95, at[1] - h / 2.0 + FOOT / 2.0);
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(RING_R)
                .arc_center(ring_at)
                .style(faded_line(Palette::faint(), 8.0, 1.0))
                .build(),
        );
        let n = f64::from(quiz.seconds);
        let ring = stage.arena_mut().add(
            polyline(ring_points(ring_at, RING_R, 1.0))
                .style(faded_line(self::quiz(), 9.0, 0.0))
                .build(),
        );
        let mut digits = Vec::new();
        for second in 1..=quiz.seconds {
            let d = kit.text(stage, &second.to_string(), 48.0, Palette::ink(), ring_at)?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let pause_line = kit.text_left(
            stage,
            PAUSE_LINE,
            26.0,
            Palette::dim(),
            v(at[0] - w / 2.0 + 0.45, ring_at[1]),
        )?;
        let left = stage.add_value_tracker(n);
        let running = stage.add_value_tracker(0.0);
        let shown = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let seen = digits.clone();
        let mut last = (f64::NAN, f64::NAN);
        stage.add_dt_updater(
            anchor,
            move |s, _, dt| {
                let mut l = s.tracker_value(left).unwrap_or(n);
                if dt > 0.0 && s.tracker_value(running).unwrap_or(0.0) > 0.5 {
                    l = (l - dt).max(0.0);
                    let _ = s.set_tracker_value(left, l);
                }
                let sh = s.tracker_value(shown).unwrap_or(0.0);
                if (l, sh) == last {
                    return;
                }
                let _ = s.set_points(ring, &corners(&ring_points(ring_at, RING_R, l / n)));
                s.set_stroke(ring, None, None, Some(sh), None, false);
                let current = l.ceil() as usize;
                for (k, &d) in seen.iter().enumerate() {
                    let on = l > 0.0 && current == k + 1;
                    s.set_opacity(d, if on { sh } else { 0.0 }, true);
                }
                last = (l, sh);
            },
            true,
        )?;
        Ok(Self {
            quiz,
            at,
            size,
            back,
            label,
            track,
            ring,
            digits,
            pause_line,
            ring_at,
            left,
            running,
            shown,
            anchor,
            #[cfg(test)]
            chapter: kit.chapter.clone(),
        })
    }

    /// The middle of the card above its foot: where the problem is placed,
    /// and the room it has.
    pub fn content(&self) -> (Vec3, f64, f64) {
        let (w, h) = self.size;
        let top = self.at[1] + h / 2.0 - 0.8;
        let bottom = self.at[1] - h / 2.0 + FOOT;
        (v(self.at[0], (top + bottom) / 2.0), w - 0.8, top - bottom)
    }

    /// Where the ring sits (and the answer, if it lands there).
    pub fn ring_at(&self) -> Vec3 {
        self.ring_at
    }

    /// Bring the card in with its full ring, `rt` seconds; `extra` (the
    /// problem's own mobjects) fade in with it. A card whose problem is
    /// still being spoken shows its frame first ([`QuizCard::show_frame`])
    /// and its ring when the question comes ([`QuizCard::arm`]).
    pub fn show(&self, stage: &mut Stage<'_>, extra: &[Mob], rt: f64) -> fmn::Result<()> {
        self.show_frame(stage, extra, rt)?;
        self.arm(stage)
    }

    /// The card without its ring: the violet frame, "your turn", the note
    /// and the ring's faint track, with `extra` fading in beside them.
    pub fn show_frame(&self, stage: &mut Stage<'_>, extra: &[Mob], rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        let mut ins = vec![self.back, self.label, self.track, self.pause_line];
        ins.extend(extra);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in ins {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        stage.play_prepared(anims)?;
        #[cfg(test)]
        self.log("frame", stage);
        Ok(())
    }

    /// The ring and its seconds, full, as the question is asked.
    pub fn arm(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        stage.play(tween(self.shown, 1.0, 0.25)?)?;
        #[cfg(test)]
        self.log("ring", stage);
        Ok(())
    }

    /// The countdown: let the question finish, then a listen window of the
    /// quiz's seconds (the narration silent, the music alone) while the ring
    /// drains from the scene's own clock.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.hold(stage)?;
        stage.set_tracker_value(self.running, 1.0)?;
        #[cfg(test)]
        self.log("window_start", stage);
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.listen(stage, f64::from(self.quiz.seconds), &label)?;
        stage.set_tracker_value(self.running, 0.0)?;
        stage.set_tracker_value(self.left, 0.0)?;
        #[cfg(test)]
        self.log("window_end", stage);
        Ok(())
    }

    /// At zero: the ring, its track and its note go, `answer` (detached,
    /// built by the chapter) lands. The next beat works it in the same place.
    pub fn land(&self, stage: &mut Stage<'_>, answer: &[Mob], rt: f64) -> fmn::Result<()> {
        #[cfg(test)]
        self.log("land", stage);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(tween(self.shown, 0.0, rt)?)?);
        for m in [self.track, self.pause_line] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        for &m in answer {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.12), 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        stage.play_prepared(anims)?;
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.ring);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        Ok(())
    }

    /// The card's own mobjects once the countdown is over (the back and the
    /// label; the chapter holds the problem and the answer).
    pub fn mobs(&self) -> Vec<Mob> {
        vec![self.back, self.label]
    }
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
        Ok(Voiced {
            ran,
            tag,
            lines,
            listens,
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
        let dir = std::env::temp_dir().join(format!("eigen_engineering_{key}_{}_{n}", std::process::id()));
        let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).map_err(|e| e.to_string())?;
        opts.config.camera.resolution = (64, 36);
        opts.config.camera.fps = 10;
        opts.config.camera.background_color = super::BACKGROUND.to_owned();
        let report = fmn::render(scene.as_mut(), opts).map_err(|e| format!("{key}: {e}"))?;
        let frames = report.artifact.frame_count as u64;
        let ran = Ran {
            frames,
            seconds: frames as f64 / 10.0,
            lint: lint.report(),
            lint_count: lint.count(),
            registered: kit.registered(),
        };
        Ok((ran, kit))
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
    }

    /// What the quiz asked on one line did, measured (scene seconds).
    #[derive(Debug)]
    pub struct QuizAudit {
        pub line: String,
        /// The quiz's countdown, as `model::QUIZZES` has it.
        pub seconds: f64,
        /// When the narrator finishes the question (the line's end).
        pub question_ends: f64,
        /// When the card, with its problem, was fully shown.
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
    }

    /// Run the chapter of the quiz asked on `line` with the scratch voice and
    /// measure the quiz's three times (REVIEW-AND-CONTINUITY section 5).
    pub fn audit_quiz(line: &str) -> Result<QuizAudit, String> {
        let quiz = super::quiz_at(line).map_err(|e| e.to_string())?;
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
        if (listen.1 - window.0).abs() > 0.11 || (listen.2 - f64::from(quiz.seconds)).abs() > 1e-9 {
            return Err(format!(
                "{line}: the listen window ({:.2} s for {:.1} s) is not the ring's ({:.2} s)",
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
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Storyboard §3: "every pair of these, and each against `ink`, is at
    /// least ΔE2000 20 apart (least 20.9, orange against gold), by a test".
    /// This is that test, with the background added: the stretched teal is
    /// the darkest role and must still be told from it.
    #[test]
    fn the_role_colours_are_told_apart() {
        let roles = [
            ("moves", moves()),
            ("pushes", pushes()),
            ("own", own()),
            ("squeezed", squeezed()),
            ("stretched", stretched()),
            ("quiz", quiz()),
            ("good", good()),
            ("warn", warn()),
            ("ink", Palette::ink()),
            ("background", ground()),
        ];
        let mut least = (f64::INFINITY, "", "");
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 20.0, "{na} and {nb}: ΔE2000 {d:.1}");
                if d < least.0 {
                    least = (d, na, nb);
                }
            }
        }
        // the storyboard's own figure for the closest pair, orange and gold
        let orange_gold = de2000(lab(pushes()), lab(own()));
        assert!((orange_gold - 20.9).abs() < 0.1, "orange against gold {orange_gold:.2}");
        assert!(least.0 >= 20.0, "{least:?}");
    }

    #[test]
    fn the_role_colours_are_the_storyboards_hex() {
        let hex_of = |c: Srgb| {
            format!(
                "#{:02X}{:02X}{:02X}",
                (c.r * 255.0).round() as u8,
                (c.g * 255.0).round() as u8,
                (c.b * 255.0).round() as u8
            )
        };
        assert_eq!(hex_of(moves()), "#58C4DD");
        assert_eq!(hex_of(pushes()), "#F2994A");
        assert_eq!(hex_of(own()), "#FFD866");
        assert_eq!(hex_of(squeezed()), "#E0408F");
        assert_eq!(hex_of(stretched()), "#26867C");
        assert_eq!(hex_of(quiz()), "#B189D6");
        assert_eq!(hex_of(good()), "#83C167");
        assert_eq!(hex_of(warn()), "#FC6255");
    }

    #[test]
    fn an_arrow_is_a_shaft_and_two_barbs_that_stay_inside_it() {
        let pts = arrow_points(v(0.0, 0.0), v(2.0, 0.0), 0.3);
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[1], pts[3], "both barbs leave the tip");
        // barbs fall back along the shaft, one to each side
        assert!(pts[2][0] < 2.0 && pts[4][0] < 2.0);
        assert!((pts[2][1] + pts[4][1]).abs() < 1e-12 && pts[2][1] != 0.0);
        // a short arrow's head is at most 45% of it
        let short = arrow_points(v(0.0, 0.0), v(0.2, 0.0), 0.3);
        assert!((short[2][0] - (0.2 - 0.09)).abs() < 1e-12, "{:?}", short[2]);
        // a zero-length arrow is a speck, not NaN
        assert!(
            arrow_points(v(1.0, 1.0), v(1.0, 1.0), 0.3)
                .iter()
                .all(|p| p[0].is_finite())
        );
    }

    /// What `set_points` is given is a quad path: an odd run, anchors at
    /// the even places and a handle between each pair.
    #[test]
    fn a_vertex_list_becomes_an_odd_run_with_handles_at_the_midpoints() {
        for n in 1..8 {
            let vertices: Vec<Vec3> = (0..n).map(|i| v(f64::from(i), f64::from(i * i))).collect();
            let run = corners(&vertices);
            assert_eq!(run.len(), 2 * n as usize - 1, "{n} vertices");
            assert_eq!(run.len() % 2, 1, "an odd count, as the engine requires");
            for (i, p) in vertices.iter().enumerate() {
                assert_eq!(run[2 * i], *p, "anchor {i}");
            }
            for i in 0..vertices.len().saturating_sub(1) {
                let mid = [
                    (vertices[i][0] + vertices[i + 1][0]) / 2.0,
                    (vertices[i][1] + vertices[i + 1][1]) / 2.0,
                    0.0,
                ];
                assert_eq!(run[2 * i + 1], mid, "handle {i}");
            }
        }
        assert!(corners(&[]).is_empty());
        // the arrow's five vertices become nine points
        assert_eq!(corners(&arrow_points(v(0.0, 0.0), v(2.0, 0.0), 0.3)).len(), 9);
    }

    #[test]
    fn rings_drain_from_the_top_and_end_where_they_began_only_when_full() {
        let c = v(1.0, 2.0);
        let full = ring_points(c, 0.5, 1.0);
        assert!(
            (full[0][0] - 1.0).abs() < 1e-9 && (full[0][1] - 2.5).abs() < 1e-9,
            "starts at the top"
        );
        let (first, last) = (full[0], full[full.len() - 1]);
        assert!((first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9);
        let half = ring_points(c, 0.5, 0.5);
        let end = half[half.len() - 1];
        assert!(
            (end[0] - 1.0).abs() < 1e-9 && (end[1] - 1.5).abs() < 1e-9,
            "half a ring ends at the bottom"
        );
        assert_eq!(ring_points(c, 0.5, 0.0).len(), 2, "an empty ring is a speck");
    }

    #[test]
    fn fractions_print_as_tex_and_read_back() {
        assert_eq!(fraction_tex("1/6"), r"\tfrac{1}{6}");
        assert_eq!(fraction_tex("-1/2"), r"-\tfrac{1}{2}");
        assert_eq!(fraction_tex("2"), "2");
        assert!((fraction_value("2/3").unwrap() - 2.0 / 3.0).abs() < 1e-15);
        assert_eq!(fraction_value("-3").unwrap(), -3.0);
        assert!(fraction_value("x/3").is_err());
    }

    /// The numbers the scenes read are the model's, in the shape the picture
    /// needs: the carts' table, the exact shares, the quiz answers.
    #[test]
    fn the_facts_the_scenes_read_are_the_models() {
        assert_eq!(
            entries("K-A1.table", None).unwrap(),
            vec![vec!["2", "-1"], vec!["-1", "2"]]
        );
        assert_eq!(items("K-A3.shares").unwrap(), ["1/2", "1/6"]);
        assert_eq!(numbers("K-A2.eigenvalues").unwrap(), [1.0, 3.0]);
        assert_eq!(items("K-Q.04_5.answer").unwrap(), ["1/3", "2/3"]);
        assert_eq!(numbers("K-Q.02_6.answer").unwrap(), [1.0, 1.0]);
        assert_eq!(minus("-1"), "\u{2013}1");
        assert_eq!(whole(-0.0), "0");
        assert!(fact("K-nope").is_err());
    }

    #[test]
    fn every_quiz_of_the_model_is_found_by_its_line() {
        for q in QUIZZES {
            assert_eq!(quiz_at(q.line).unwrap().line, q.line);
            assert!((3..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
        }
        assert!(quiz_at("01_5").is_err(), "a plain prediction has no countdown");
    }

    /// REVIEW-AND-CONTINUITY section 5, measured on a scratch-voice run of
    /// the quiz's chapter: the card and its problem are readable no later
    /// than the narrator's last word of the question; the viewer then has
    /// the quiz's 4 to 6 seconds with the narration silent and a ring
    /// counting them; and the answer lands only after the countdown ended
    /// (the answer's line starts then too). What stands on the screen as a
    /// possible hint is read in each chapter's code (decisions.md); this is
    /// the part that is a number.
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
            (4.0..=6.0).contains(&a.seconds),
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
        quiz_02_6_follows_section_5: "02_6",
        quiz_04_5_follows_section_5: "04_5",
        quiz_11_3_follows_section_5: "11_3",
        quiz_12_2_follows_section_5: "12_2",
    }
}
