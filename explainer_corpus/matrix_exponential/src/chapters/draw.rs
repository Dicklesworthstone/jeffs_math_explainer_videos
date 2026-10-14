//! Video J's look, shared by every chapter (storyboard §3 "Color roles" and
//! §4 "Showcase"): the seven role colours, line timing against the voice, the
//! facts as the scenes read them, the grid of numbers that sits in an
//! exponent, the plane of dots carried along their arrows (the one picture of
//! the video), the "your turn" card with its countdown, and the harness that
//! constructs a real chapter in a test.
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

use super::sound;
use crate::model::{Mat, QUIZZES, Quiz};

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

/// A fact that is a matrix (`0,-1;1,0`): its rows.
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

/// `x` to `places` decimals, as `facts::dec` prints it.
pub fn show(x: f64, places: usize) -> fmn::Result<String> {
    crate::facts::dec(x, places).map_err(fail)
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

/// The true minus sign (U+2212) for plain text; TeX sets its own.
pub fn minus(s: &str) -> String {
    s.replace('-', "\u{2212}")
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

    /// Wait (if need be) until the line's first sentence has been said (or
    /// the line is over, if it has one): the answer of a quiz stays fully up
    /// until then, in the line that follows the countdown.
    pub fn wait_first_sentence(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if self.sentences.len() > 1 {
            self.wait_sentence(stage, 1)
        } else {
            self.wait_end(stage)
        }
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

/// A prediction pause with no ring (storyboard: "five further prediction
/// pauses hold for a moment without a ring"): let the question finish, mark
/// the moment for the score, hold the picture.
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

/// The rule: a rate, a tiny step, the matrix in the exponent, its powers and
/// terms, velocity arrows, spins.
pub fn rule() -> Srgb {
    hex("#5CD0B3")
}

/// The result: e^{tA}, the running total, a path traced, where a point ends
/// up, a rotation reached (the corpus's hero gold).
pub fn result() -> Srgb {
    Palette::hero()
}

/// The act of turning drawn as geometry: the fan of arrows, the dial of
/// powers, an arc. (A little more azure than the kit's blue: the storyboard's
/// teal and blue were 19.6 apart in ΔE2000, under its own gate of 20.)
pub fn turning() -> Srgb {
    hex("#54C4E5")
}

/// A substance in a container or on a node: money, salt, heat, chance. (A
/// deeper orange than the kit's amber, which sat 14.9 from the result's gold.)
pub fn substance() -> Srgb {
    hex("#E8903B")
}

/// The second rule B, where order matters.
pub fn second() -> Srgb {
    hex("#E48BB5")
}

/// The guess that fails, a broken rule, digits lost.
pub fn warn() -> Srgb {
    Palette::red()
}

/// A confirmed answer. (More yellow-green than the kit's: 17.3 from the
/// rule's teal.)
pub fn good() -> Srgb {
    hex("#83B943")
}

/// The "your turn" card and its countdown. (Bluer than the storyboard's
/// `#B189D6`, which sat 15.7 from the second rule's rose.)
pub fn quiz() -> Srgb {
    hex("#9985E6")
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

/// Words on a dark plate edged in `edge`, so they read over a field of dots
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

/// A 2 by 2 map of the plane, rows.
pub type Map2 = [[f64; 2]; 2];

/// A model matrix as a 2 by 2 map.
pub fn map2(m: &Mat) -> Map2 {
    [[m.get(0, 0), m.get(0, 1)], [m.get(1, 0), m.get(1, 1)]]
}

/// The map applied to a point of the plane.
pub fn apply(m: &Map2, p: [f64; 2]) -> [f64; 2] {
    [m[0][0] * p[0] + m[0][1] * p[1], m[1][0] * p[0] + m[1][1] * p[1]]
}

/// The turn by `t` radians.
pub fn rotation_map(t: f64) -> Map2 {
    [[t.cos(), -t.sin()], [t.sin(), t.cos()]]
}

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

/// The points `(i·spacing, j·spacing)` within `radius` of the origin, row by
/// row from the top.
pub fn lattice(spacing: f64, radius: f64) -> Vec<[f64; 2]> {
    // a hair over, so that 3.3 / 0.55 is 6 and not 5.999...
    let n = (radius / spacing + 1e-9).floor() as i32;
    let mut pts = Vec::new();
    for j in (-n..=n).rev() {
        for i in -n..=n {
            let p = [f64::from(i) * spacing, f64::from(j) * spacing];
            if p[0].hypot(p[1]) <= radius + 1e-9 {
                pts.push(p);
            }
        }
    }
    pts
}

/// The index of the point of `pts` nearest `target`.
pub fn nearest(pts: &[[f64; 2]], target: [f64; 2]) -> usize {
    pts.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a[0] - target[0]).hypot(a[1] - target[1]);
            let db = (b[0] - target[0]).hypot(b[1] - target[1]);
            da.total_cmp(&db)
        })
        .map_or(0, |(i, _)| i)
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

/// What a dot of a [`Flow`] carries: the rule's arrow at its current place.
#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    pub rule: Map2,
    /// Plane units of arrow per plane unit of speed.
    pub scale: f64,
    pub color: Srgb,
    pub width: f64,
    /// The head's length on screen.
    pub head: f64,
}

/// What a [`Flow`] is made of.
#[derive(Clone, Debug)]
pub struct FlowSpec {
    pub plane: Plane,
    pub starts: Vec<[f64; 2]>,
    pub dot_radius: f64,
    pub dot_color: Srgb,
    /// How bright the dots are to begin with (0 to 1).
    pub dot_level: f64,
    pub velocity: Option<Velocity>,
    /// Which starts leave a trail behind them.
    pub trails: Vec<usize>,
    pub trail_color: Srgb,
    pub trail_width: f64,
}

/// The central picture (storyboard §4: "a dot field placed every frame at
/// e^{tA} applied to each start, with arrows and trails"): dots, each with
/// the rule's arrow where it stands, and trails behind a few, all rewritten
/// by one updater from one tracker.
pub struct Flow {
    /// Dots, then arrows, then trails: add this to the scene to show it.
    pub group: Mob,
    pub dots: Vec<Mob>,
    pub arrows: Vec<Mob>,
    pub trails: Vec<Mob>,
    /// How bright the dots, the arrows and the trails are (0 to 1).
    pub levels: [Mob; 3],
    anchor: Mob,
    spec: FlowSpec,
}

/// How many samples a trail is drawn from.
const TRAIL_SAMPLES: usize = 40;

impl Flow {
    /// The dots, arrows and trails at the start (time 0), detached; arrows
    /// and trails begin invisible (level 0).
    pub fn build(stage: &mut Stage<'_>, spec: FlowSpec) -> fmn::Result<Self> {
        let mut members = Vec::new();
        let mut dots = Vec::new();
        for &p in &spec.starts {
            let c = spec.plane.p(p[0], p[1]);
            let d = Dot::new()
                .point(c)
                .radius(spec.dot_radius)
                .style(flat(spec.dot_color, spec.dot_level))
                .build();
            let m = stage.arena_mut().add(d);
            dots.push(m);
            members.push(m);
        }
        let mut arrows = Vec::new();
        if let Some(vel) = &spec.velocity {
            for &p in &spec.starts {
                let tail = spec.plane.p(p[0], p[1]);
                let w = apply(&vel.rule, p);
                let tip = spec.plane.p(p[0] + vel.scale * w[0], p[1] + vel.scale * w[1]);
                let a = polyline(arrow_points(tail, tip, vel.head))
                    .style(faded_line(vel.color, vel.width, 0.0))
                    .build();
                let m = stage.arena_mut().add(a);
                arrows.push(m);
                members.push(m);
            }
        }
        let mut trails = Vec::new();
        for &i in &spec.trails {
            let c = spec.plane.p(spec.starts[i][0], spec.starts[i][1]);
            let t = polyline(vec![c, v(c[0] + 1e-4, c[1])])
                .style(faded_line(spec.trail_color, spec.trail_width, 0.0))
                .build();
            let m = stage.arena_mut().add(t);
            trails.push(m);
            members.push(m);
        }
        let g = group(stage, &members)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        let levels = [
            stage.add_value_tracker(spec.dot_level),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        ];
        Ok(Self {
            group: g,
            dots,
            arrows,
            trails,
            levels,
            anchor,
            spec,
        })
    }

    /// Every dot, arrow and trail.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.dots.clone();
        all.extend(&self.arrows);
        all.extend(&self.trails);
        all
    }

    /// Place everything from now on at `map(t)` applied to each start, `t`
    /// read from `tracker` every frame (one updater, working only when the
    /// tracker or a level has moved).
    pub fn drive(&self, stage: &mut Stage<'_>, tracker: Mob, map: impl Fn(f64) -> Map2 + 'static) -> fmn::Result<()> {
        let plane = self.spec.plane;
        let starts = self.spec.starts.clone();
        let (dots, arrows, trails) = (self.dots.clone(), self.arrows.clone(), self.trails.clone());
        let trail_of = self.spec.trails.clone();
        let velocity = self.spec.velocity.map(|vel| (vel.rule, vel.scale, vel.head));
        let template: Vec<Vec3> = Dot::new().radius(self.spec.dot_radius).build().points().to_vec();
        let levels = self.levels;
        let mut last = (f64::NAN, [f64::NAN; 3]);
        stage.add_updater(
            self.anchor,
            move |s, _| {
                let t = s.tracker_value(tracker).unwrap_or(0.0);
                let lv = levels.map(|m| s.tracker_value(m).unwrap_or(0.0));
                if t.to_bits() != last.0.to_bits() {
                    let m = map(t);
                    for (&d, &p) in dots.iter().zip(&starts) {
                        let q = apply(&m, p);
                        let c = plane.p(q[0], q[1]);
                        let pts: Vec<Vec3> = template.iter().map(|p| [p[0] + c[0], p[1] + c[1], p[2]]).collect();
                        let _ = s.set_points(d, &pts);
                    }
                    if let Some((rule, k, head)) = velocity {
                        for (&a, &p) in arrows.iter().zip(&starts) {
                            let q = apply(&m, p);
                            let w = apply(&rule, q);
                            let tail = plane.p(q[0], q[1]);
                            let tip = plane.p(q[0] + k * w[0], q[1] + k * w[1]);
                            let _ = s.set_points(a, &corners(&arrow_points(tail, tip, head)));
                        }
                    }
                    for (&tr, &i) in trails.iter().zip(&trail_of) {
                        let mut pts: Vec<Vec3> = (0..=TRAIL_SAMPLES)
                            .map(|k| {
                                let q = apply(&map(t * k as f64 / TRAIL_SAMPLES as f64), starts[i]);
                                plane.p(q[0], q[1])
                            })
                            .collect();
                        if t.abs() < 1e-3 {
                            pts.truncate(1);
                            pts.push(v(pts[0][0] + 1e-4, pts[0][1]));
                        }
                        let _ = s.set_points(tr, &corners(&pts));
                    }
                }
                if lv != last.1 {
                    for &d in &dots {
                        s.set_opacity(d, lv[0], false);
                    }
                    // arrows and trails are strokes: their fill stays at zero
                    // (`set_opacity` would fill an open trail, and the chord
                    // of a long arc becomes a petal)
                    for &a in &arrows {
                        s.set_stroke(a, None, None, Some(lv[1]), None, false);
                    }
                    for &tr in &trails {
                        s.set_stroke(tr, None, None, Some(lv[2]), None, false);
                    }
                }
                last = (t, lv);
            },
            true,
        )?;
        stage.add_to_scene(self.anchor)?;
        Ok(())
    }

    /// Stop the updater (before the flow fades out, or the chapter ends).
    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// A slow wobble of `tracker` about 0 while `on` reads above one half
/// (amplitude `amp`, `rate` radians a second): the plane "stirs". Stops, and
/// leaves the tracker where it is, when `on` is turned off.
pub fn stir(stage: &mut Stage<'_>, tracker: Mob, on: Mob, amp: f64, rate: f64) -> fmn::Result<Mob> {
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut phase = 0.0_f64;
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            if dt > 0.0 && s.tracker_value(on).unwrap_or(0.0) > 0.5 {
                phase += rate * dt;
                let _ = s.set_tracker_value(tracker, amp * phase.sin());
            }
        },
        false,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

// ------------------------------------------------------- the grid prop

/// A grid of numbers with its brackets drawn (storyboard §4: "the shared
/// grid prop, which can sit in an exponent, tick and be pointed at"): one
/// TeX string per cell, so it sits in an exponent at a small size and is the
/// matrix of the chapter at a large one.
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

    /// A rounded outline round the whole grid, in `color` (the guess, "outlined
    /// in red"), detached.
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
/// [`QuizCard::content`]; the answer is the `F-Q.<line>.*` facts, built by
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
    /// A mobject whose updater watches the card's frame and notes when,
    /// fully shown, it begins to go (tests only: the audit of how long the
    /// answer stays up).
    #[cfg(test)]
    watch: Mob,
    /// The scene time the watch was put on the scene at (tests only).
    #[cfg(test)]
    since: std::rc::Rc<std::cell::Cell<f64>>,
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
            .style(Style::default().fill(panel(), 0.96).stroke(quiz_color(), 3.0, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let label = kit.text(stage, YOUR_TURN, 30.0, quiz_color(), v(at[0], at[1] + h / 2.0 - 0.42))?;
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
                .style(faded_line(quiz_color(), 9.0, 0.0))
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
        #[cfg(test)]
        let (watch, since) = {
            let watch = stage.arena_mut().add(VMobject::new());
            let (chapter, line) = (kit.chapter.clone(), quiz.line);
            // the scene time the watch was put on the scene at, and the
            // seconds it has seen since (the updater is given only dt)
            let since = std::rc::Rc::new(std::cell::Cell::new(0.0));
            let origin = std::rc::Rc::clone(&since);
            let (mut full, mut gone, mut seen) = (false, false, 0.0);
            stage.add_dt_updater(
                watch,
                move |s, _, dt| {
                    seen += dt;
                    // (called once as registered, with the frame at its
                    // full opacity before the card fades in: not a sighting)
                    if gone || seen == 0.0 {
                        return;
                    }
                    let o = s.get_fill_opacity(back).unwrap_or(0.0);
                    if o >= 0.9 {
                        full = true;
                    } else if full {
                        gone = true;
                        quiz_log::record(&chapter, line, "leave", origin.get() + seen);
                    }
                },
                true,
            )?;
            (watch, since)
        };
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
            #[cfg(test)]
            watch,
            #[cfg(test)]
            since,
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
        #[cfg(test)]
        {
            self.since.set(stage.scene().time().to_f64());
            stage.add_to_scene(self.watch)?;
        }
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
    /// quiz's seconds (the narration silent, one soft note a second under the
    /// ring) while the ring drains from the scene's own clock; at zero the
    /// answer's note rings for [`sound::LAND_S`] before the next line, which
    /// is where the chapter lands the answer.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.hold(stage)?;
        let n = f64::from(self.quiz.seconds);
        let wav = sound::countdown(self.quiz.seconds)?;
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.sonify(stage, n + sound::LAND_S, &label, &wav)?;
        let start = stage.scene().time().to_f64();
        stage.set_tracker_value(self.running, 1.0)?;
        #[cfg(test)]
        self.log("window_start", stage);
        wait_until(stage, start + n)?;
        stage.set_tracker_value(self.running, 0.0)?;
        stage.set_tracker_value(self.left, 0.0)?;
        #[cfg(test)]
        self.log("window_end", stage);
        wait_until(stage, start + n + sound::LAND_S)
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

fn quiz_color() -> Srgb {
    quiz()
}

/// Tests only: have `mob` watched through the run, noting in `quiz_log`
/// under the chapter's name and `key` each time it is fully seen ("up": on
/// the scene at nearly the opacity it was made with) and each time it has
/// all but gone ("down": off the scene, or nearly transparent), scene
/// seconds. A fade-out takes the mobject off the scene and gives it its
/// opacity back, so both are read. The first thing seen is a state, not an
/// event. Call it where the mobject is made, before anything fades it; how
/// long a credit stands, or whether a picture is out of the way of a card,
/// is then a number a test can read.
#[cfg(test)]
pub fn watch_dwell(kit: &Kit, stage: &mut Stage<'_>, mob: Mob, key: &'static str) -> fmn::Result<()> {
    let watch = stage.arena_mut().add(VMobject::new());
    let chapter = kit.chapter.clone();
    let origin = stage.scene().time().to_f64();
    let (mut base, mut seen, mut state) = (f64::NAN, 0.0, None::<bool>);
    // the most opaque of the family (a group has no colour of its own)
    let opacity = move |s: &fmn::mobject::Stage| {
        s.family(mob).into_iter().fold(0.0, |o: f64, m| {
            o.max(s.get_fill_opacity(m).unwrap_or(0.0))
                .max(s.get_stroke_opacity(m).unwrap_or(0.0))
        })
    };
    let on_scene = move |s: &fmn::mobject::Stage| s.roots().iter().any(|&r| r == mob || s.family(r).contains(&mob));
    stage.add_dt_updater(
        watch,
        move |s, _, dt| {
            seen += dt;
            let o = opacity(s);
            if base.is_nan() {
                // as registered: the mobject's own opacity, before any fade,
                // and whether it is up now (the state the first change is
                // measured from)
                base = o.max(1e-6);
                state = Some(on_scene(s) && o >= 0.9 * base);
                return;
            }
            let now = if on_scene(s) && o >= 0.9 * base {
                Some(true)
            } else if !on_scene(s) || o <= 0.1 * base {
                Some(false)
            } else {
                state
            };
            if now != state {
                if state.is_some() {
                    let kind = if now == Some(true) { "up" } else { "down" };
                    quiz_log::record(&chapter, key, kind, origin + seen);
                }
                state = now;
            }
        },
        true,
    )?;
    stage.add_to_scene(watch)?;
    Ok(())
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
        /// The folder of the run's frames (one PNG a frame, in order).
        pub frames_dir: std::path::PathBuf,
    }

    /// The sizes of a run's frame files, in frame order.
    pub fn frame_sizes(dir: &std::path::Path) -> Result<Vec<u64>, String> {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "png"))
            .collect();
        files.sort();
        files
            .iter()
            .map(|p| std::fs::metadata(p).map(|m| m.len()).map_err(|e| e.to_string()))
            .collect()
    }

    /// Whether frame `i` of a run (frame file sizes `sizes`) shows anything:
    /// a frame with no picture is the background alone, the smallest file of
    /// the run, and the same size as every other such frame.
    pub fn has_picture(sizes: &[u64], i: usize) -> bool {
        let blank = sizes.iter().copied().min().unwrap_or(0);
        sizes.get(i).is_some_and(|&s| s > blank + 16)
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
        run_voiced_with(key, false)
    }

    /// The finished narration's own timing (`narration/out/report.json` of
    /// the scored draft of 2026-10-14: each sentence's onset and the line's
    /// length, seconds) of the ten lines that follow a quiz's countdown. The
    /// scratch voice puts a first sentence by its share of the characters,
    /// so "No." ends at 0.3 s there and at 0.82 s in the real read; a card
    /// that leaves on a fixed beat passes on the one and goes early on the
    /// other.
    // (06_4's third sentence starts at 3.14 s: seconds, not pi)
    #[allow(clippy::approx_constant)]
    pub const REAL_TIMING: &[(&str, &[f64], f64)] = &[
        ("03_4", &[0.0, 0.82, 2.06, 5.44, 9.12], 10.35),
        ("04_5", &[0.0, 2.08, 4.75, 9.06], 10.62),
        ("06_4", &[0.0, 1.14, 3.14, 4.72, 9.445, 13.945], 15.76),
        ("07_4", &[0.0, 1.1, 2.98, 6.16], 8.57),
        ("08_4", &[0.0, 1.24, 5.42, 8.53], 10.69),
        ("10_6", &[0.0, 1.82, 4.33], 7.91),
        ("11_4", &[0.0, 0.73, 4.17], 5.87),
        ("14_4", &[0.0, 0.82, 3.12, 9.69], 11.71),
        ("15_3", &[0.0, 0.82, 5.04], 7.95),
        ("16_4", &[0.0, 2.63, 7.39, 12.57], 13.58),
    ];

    /// `run_voiced` with the lines of `REAL_TIMING` placed as the finished
    /// narration says them.
    pub fn run_voiced_real(key: &str) -> Result<Voiced, String> {
        run_voiced_with(key, true)
    }

    fn run_voiced_with(key: &str, real: bool) -> Result<Voiced, String> {
        let script = Script::parse(crate::SCRIPT)?;
        let narrator = if real {
            Narrator::estimate_timed(&script, REAL_TIMING)?
        } else {
            Narrator::estimate(&script)
        };
        // a name of its own, so the events of this run are not mixed with
        // another test's run of the same chapter
        let tag = format!("{key}#{}", if real { "real" } else { "voiced" });
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
        let sentences = narrator
            .said()
            .into_iter()
            .map(|(id, _)| {
                let starts = (0..).map_while(|j| narrator.sentence_at(&id, j)).collect();
                (id, starts)
            })
            .collect();
        Ok(Voiced {
            ran,
            tag,
            lines,
            listens,
            marks,
            sentences,
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
        let dir = std::env::temp_dir().join(format!("matrix_exponential_{key}_{}_{n}", std::process::id()));
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
            frames_dir: dir,
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
        /// Every mark the chapter made ("silence:...", "reveal:..."): its
        /// label and when, seconds.
        pub marks: Vec<(String, f64)>,
        /// Every line said: its id and where each of its sentences starts
        /// (scene seconds).
        pub sentences: Vec<(String, Vec<f64>)>,
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
        /// When the card, with the answer on it, begins to go (None if it
        /// stays to the chapter's end).
        pub answer_leaves: Option<f64>,
        /// When the first sentence of the next line has been said: the
        /// answer stays fully up until then.
        pub answer_said: Option<f64>,
        /// When the whole of the next line has been said.
        pub next_ends: Option<f64>,
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
        audit_quiz_with(line, false)
    }

    /// `audit_quiz` with the lines after the countdown placed as the finished
    /// narration says them (`REAL_TIMING`).
    pub fn audit_quiz_real(line: &str) -> Result<QuizAudit, String> {
        audit_quiz_with(line, true)
    }

    fn audit_quiz_with(line: &str, real: bool) -> Result<QuizAudit, String> {
        let quiz = super::quiz_at(line).map_err(|e| e.to_string())?;
        let prefix = format!("{}_", line.split('_').next().unwrap_or(line));
        let key = crate::chapters::registry()
            .into_iter()
            .map(|c| c.key)
            .find(|k| k.starts_with(&prefix))
            .ok_or_else(|| format!("no chapter for line {line}"))?;
        let voiced = run_voiced_with(key, real)?;
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
        // the window is the ring's seconds and the answer's beat (the note of
        // the answer rings alone before the next line)
        let wanted = f64::from(quiz.seconds) + super::sound::LAND_S;
        if (listen.1 - window.0).abs() > 0.11 || (listen.2 - wanted).abs() > 1e-9 {
            return Err(format!(
                "{line}: the listen window ({:.2} s for {:.1} s, wanted {wanted:.1} s) is not the ring's ({:.2} s)",
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
        // the first sentence of the next line ends where its second begins,
        // or where the line does if it has one sentence
        let answer_said = next_line.as_ref().and_then(|(id, at)| {
            let starts = voiced.sentences.iter().find(|(s, _)| s == id).map(|(_, st)| st)?;
            match starts.get(1) {
                Some(&second) => Some(second),
                None => {
                    let len = voiced.lines.iter().find(|(l, ..)| l == id).map(|&(_, _, len)| len)?;
                    Some(at + len)
                }
            }
        });
        let next_ends = next_line.as_ref().and_then(|(id, at)| {
            let len = voiced.lines.iter().find(|(l, ..)| l == id).map(|&(_, _, len)| len)?;
            Some(at + len)
        });
        let answer_leaves = events.iter().find(|(k, _)| *k == "leave").map(|&(_, t)| t);
        Ok(QuizAudit {
            line: line.to_owned(),
            seconds: f64::from(quiz.seconds),
            question_ends: start + len,
            card_shown: at("frame")?,
            ring_armed: at("ring")?,
            window,
            answer_lands: at("land")?,
            answer_leaves,
            answer_said,
            next_ends,
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
    use crate::model::{expm, expm_t, quarter_turn};

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

    /// Storyboard §3: the colour roles are checked pairwise for contrast
    /// (ΔE2000 at least 20) before drafts. As the storyboard first set them,
    /// four pairs were under it (gold and amber 14.9, rose and violet 15.7,
    /// teal and green 17.3, teal and blue 19.6); the roles here are nudged
    /// the least that clears it (docs/decisions.md).
    #[test]
    fn the_role_colours_are_told_apart() {
        let roles = [
            ("rule", rule()),
            ("result", result()),
            ("turning", turning()),
            ("substance", substance()),
            ("second rule", second()),
            ("warn", warn()),
            ("good", good()),
            ("quiz", quiz()),
            ("ink", Palette::ink()),
            ("background", ground()),
        ];
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 20.0, "{na} and {nb}: ΔE2000 {d:.1}");
            }
        }
    }

    /// Run `construct` on a scratch stage with no sink (the scene's mobjects
    /// are really built, updaters really run, nothing is rendered).
    fn probe(construct: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        fmn::run_scene(
            &mut Probe(construct),
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("the probe scene runs");
    }

    /// The flow's arrows and trails are strokes: raising their levels to full
    /// opacity (and following the rule far enough that a trail is a long arc)
    /// leaves their fill at nothing, both in `Flow::drive` and in the rule
    /// driver. With a fill, an open trail is a petal: the chord of its arc
    /// becomes the other edge of a yellow shape (09_5 and 09_6 showed it).
    #[test]
    fn arrows_and_trails_stay_strokes_as_their_levels_rise() {
        use crate::chapters::flow::{RuleTrackers, drive_rule};
        probe(|stage| {
            let j = quarter_turn();
            let spec = FlowSpec {
                plane: Plane {
                    centre: [0.0, 0.0, 0.0],
                    unit: 1.0,
                },
                starts: vec![[1.0, 0.0], [0.0, 2.0], [-1.5, 0.5]],
                dot_radius: 0.05,
                dot_color: Palette::ink(),
                dot_level: 0.0,
                velocity: Some(Velocity {
                    rule: map2(&j),
                    scale: 0.2,
                    color: rule(),
                    width: 3.0,
                    head: 0.1,
                }),
                trails: vec![0, 1, 2],
                trail_color: result(),
                trail_width: 3.0,
            };
            // the flow with a fixed rule
            let flow = Flow::build(stage, spec.clone())?;
            let clock = stage.add_value_tracker(0.0);
            let rotate = {
                let j = j.clone();
                move |t: f64| map2(&expm_t(&j, t))
            };
            flow.drive(stage, clock, rotate)?;
            stage.add_to_scene(flow.group)?;
            // the flow driven by a rule's entries
            let rule_flow = Flow::build(stage, spec.clone())?;
            let trackers = RuleTrackers::new(stage, &j);
            drive_rule(stage, &rule_flow, &spec, trackers)?;
            stage.add_to_scene(rule_flow.group)?;
            for f in [&flow, &rule_flow] {
                stage.set_tracker_value(f.levels[1], 1.0)?;
                stage.set_tracker_value(f.levels[2], 0.8)?;
            }
            stage.set_tracker_value(clock, 2.0)?;
            stage.set_tracker_value(trackers.time, 2.0)?;
            pause(stage, 0.3)?;
            for f in [&flow, &rule_flow] {
                for &m in f.trails.iter().chain(&f.arrows) {
                    assert_eq!(stage.arena().get_fill_opacity(m), Some(0.0), "a stroke has no fill");
                }
                for &m in &f.trails {
                    let o = stage.arena().get_stroke_opacity(m).unwrap_or(0.0);
                    assert!((o - 0.8).abs() < 1e-6, "the trail's stroke is at its level: {o}");
                }
            }
            Ok(())
        });
    }

    /// The cue sheets of a narrated draft show the frame 1.5 s into each line
    /// (the middle of a shorter one): a line must not begin over a black
    /// stage. Measured on the scratch-voice run of every chapter, at the
    /// frame that moment falls on (a frame with no picture is the background
    /// alone). The narrated draft had seven such lines in chapters 09 to 16
    /// (11_1, 12_4, 13_5, 14_2, 14_5, 16_2 and 16_3; 16_1 and 13_4 were all
    /// but black), and a handful in 01 to 08 that are fixed.
    #[test]
    fn no_line_begins_over_a_black_stage() {
        let mut black = Vec::new();
        for key in crate::chapters::registry().into_iter().map(|c| c.key) {
            let voiced = testing::run_voiced(key).unwrap();
            let sizes = testing::frame_sizes(&voiced.ran.frames_dir).unwrap();
            for (id, start, len) in &voiced.lines {
                let at = start + 1.5_f64.min(len / 2.0);
                let frame = (at * 10.0).floor() as usize;
                if !testing::has_picture(&sizes, frame) {
                    black.push(format!("{id} at {at:.1} s"));
                }
            }
        }
        assert!(black.is_empty(), "black stages: {black:?}");
    }

    #[test]
    fn the_lattice_is_a_disc_of_about_a_hundred_and_twenty_dots() {
        let pts = lattice(0.55, 3.3);
        assert!((100..=140).contains(&pts.len()), "{}", pts.len());
        assert!(pts.contains(&[0.0, 0.0]), "the centre is one of them");
        assert!(pts.iter().all(|p| p[0].hypot(p[1]) <= 3.3 + 1e-9));
        // symmetric: every point's mirror is there
        for p in &pts {
            assert!(
                pts.iter()
                    .any(|q| (q[0] + p[0]).abs() < 1e-9 && (q[1] + p[1]).abs() < 1e-9)
            );
        }
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
        // the arrow's five vertices become nine points, and a trail's
        // forty-one eighty-one: never an even count
        assert_eq!(corners(&arrow_points(v(0.0, 0.0), v(2.0, 0.0), 0.3)).len(), 9);
        assert_eq!(corners(&[v(0.0, 0.0); 41]).len(), 81);
        assert_eq!(corners(&[v(0.0, 0.0); 2]).len(), 3);
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
    fn the_facts_the_scenes_read_are_the_models() {
        // the cold open's answer, to the three places it is shown, is the
        // number 07_6 shows
        let shown = entries("F-H.J.true", Some(3)).unwrap();
        assert_eq!(shown, entries("F-V.07_6.at_one", Some(3)).unwrap());
        let want = expm(&quarter_turn());
        for (i, row) in shown.iter().enumerate() {
            for (j, s) in row.iter().enumerate() {
                assert_eq!(*s, show(want.get(i, j), 3).unwrap(), "entry {i},{j}");
            }
        }
        // the quarter turn itself, as whole numbers
        assert_eq!(
            entries("F-J.power.k1", None).unwrap(),
            vec![vec!["0", "-1"], vec!["1", "0"]]
        );
        assert_eq!(minus("-0.841"), "\u{2212}0.841");
        assert_eq!(whole(-0.0), "0");
    }

    #[test]
    fn every_quiz_of_the_model_is_found_by_its_line() {
        for q in QUIZZES {
            assert_eq!(quiz_at(q.line).unwrap().line, q.line);
            assert!((3..=6).contains(&q.seconds), "{}: {} s", q.line, q.seconds);
        }
        assert!(quiz_at("01_1").is_err(), "the opening question has no countdown");
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
                *at >= a.window.1 + sound::LAND_S - 0.05,
                "{line}: {next} starts inside the countdown or the answer's beat\n{said}"
            );
        }
        // the answer stays fully up until the voice has said the first
        // sentence after it (the producer's call on the dense read of J: a
        // card that fades under the answer's own words is an answer not seen)
        eprintln!(
            "{line}: answer on screen from {:.1}s, card goes {:?}, first sentence after is said {:?}",
            a.answer_lands, a.answer_leaves, a.answer_said
        );
        if let (Some(leaves), Some(said_at)) = (a.answer_leaves, a.answer_said) {
            assert!(
                leaves >= said_at - 0.05,
                "{line}: the card goes at {leaves:.2} s, the first sentence after the answer is said at {said_at:.2} s\n{said}"
            );
        }
    }

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
        quiz_03_3_follows_section_5: "03_3",
        quiz_04_4_follows_section_5: "04_4",
        quiz_05_5_follows_section_5: "05_5",
        quiz_06_3_follows_section_5: "06_3",
        quiz_07_3_follows_section_5: "07_3",
        quiz_08_3_follows_section_5: "08_3",
        quiz_10_5_follows_section_5: "10_5",
        quiz_11_3_follows_section_5: "11_3",
        quiz_14_3_follows_section_5: "14_3",
        quiz_15_2_follows_section_5: "15_2",
        quiz_16_3_follows_section_5: "16_3",
    }

    /// The rule of the answer on the finished narration's own sentences
    /// (REAL_TIMING from the scored draft's report.json): the card stands
    /// fully up until the second sentence of the next line begins, which
    /// is when the first has been said. The scratch voice puts that
    /// moment by shares of the characters and cannot see a card that
    /// leaves on a fixed beat (03_4's "No." is 0.3 s there and 0.82 s in
    /// the read; 07_4's "A half turn." 0.2 and 1.1; 14_4's "No." and
    /// 15_3's "No." 0.8; the late leavers pass on both).
    fn on_the_real_voice(line: &str) {
        let a = testing::audit_quiz_real(line).unwrap();
        let (leaves, said_at) = (
            a.answer_leaves.expect("the card goes"),
            a.answer_said.expect("a line follows"),
        );
        eprintln!(
            "{line}: on the finished read the card goes at {leaves:.2} s, the first sentence after it is said at {said_at:.2} s"
        );
        assert!(
            leaves >= said_at - 0.05,
            "{line}: on the finished read the card goes at {leaves:.2} s, the first sentence after the answer is said at {said_at:.2} s\n{a:#?}"
        );
    }

    macro_rules! real_voice_tests {
        ($($name:ident: $line:literal),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    on_the_real_voice($line);
                }
            )*
        };
    }

    real_voice_tests! {
        quiz_03_3_leaves_on_the_real_first_sentence: "03_3",
        quiz_04_4_leaves_on_the_real_first_sentence: "04_4",
        quiz_06_3_leaves_on_the_real_first_sentence: "06_3",
        quiz_07_3_leaves_on_the_real_first_sentence: "07_3",
        quiz_08_3_leaves_on_the_real_first_sentence: "08_3",
        quiz_10_5_leaves_on_the_real_first_sentence: "10_5",
        quiz_11_3_leaves_on_the_real_first_sentence: "11_3",
        quiz_16_3_leaves_on_the_real_first_sentence: "16_3",
    }

    /// 05_5 (the dense read of J's first half): the card the answer is worked
    /// on goes once the line that works it is said, leaving the chart a
    /// while with the marker's reading; it had stood eleven seconds, to the
    /// chapter's last.
    #[test]
    fn quiz_05_5_card_goes_once_its_working_is_said() {
        let a = testing::audit_quiz("05_5").unwrap();
        let leaves = a.answer_leaves.expect("the card goes");
        let ends = a.next_ends.expect("a line follows");
        assert!(
            leaves >= ends - 0.05,
            "the card goes at {leaves:.1} s, the working's line ends {ends:.1} s"
        );
        assert!(
            leaves <= a.chapter_seconds - 1.5,
            "the card goes at {leaves:.1} s of a chapter of {:.1} s",
            a.chapter_seconds
        );
    }
}
