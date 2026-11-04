//! The film's look, shared by every chapter (storyboard "Colour roles"):
//! line timing, fades, the two planes, the colour picture of a function
//! (kit `RowStrips`, one strip per row, recoloured in place), zero and pole
//! marks, a loop with a walker whose image is followed around zero, and
//! the prediction card with its draining ring.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`. Engine rules (`explainer_kit::kit`): one flat fill per
//! mobject, `always_redraw` only for small readouts, groups well under
//! `GROUP_TRANSFORM_LIMIT`, updaters cleared before a fade.

use std::f64::consts::TAU;
use std::sync::OnceLock;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::props::RowStrips;
use explainer_kit::{Facts, Frame2, Kit, Palette, Script, hex, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::model::{self, C, Func};

/// A scene error carrying `message`.
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------------- facts

/// Every fact of the film, computed once; scenes read numbers from here.
pub fn facts() -> fmn::Result<&'static Facts> {
    static FACTS: OnceLock<Result<Facts, String>> = OnceLock::new();
    FACTS
        .get_or_init(crate::facts::facts)
        .as_ref()
        .map_err(|e| fail(format!("facts: {e}")))
}

/// One fact's value, as printed.
pub fn fact(key: &str) -> fmn::Result<&'static str> {
    facts()?.get(key).ok_or_else(|| fail(format!("no fact {key}")))
}

/// One fact that is a number.
pub fn fact_f64(key: &str) -> fmn::Result<f64> {
    let s = fact(key)?;
    s.parse().map_err(|_| fail(format!("fact {key} is not a number: {s}")))
}

/// One fact that is a comma-separated list of numbers.
pub fn fact_list(key: &str) -> fmn::Result<Vec<f64>> {
    fact(key)?
        .split(',')
        .map(|x| {
            x.parse::<f64>()
                .map_err(|_| fail(format!("fact {key} has a non-number {x:?}")))
        })
        .collect()
}

// ------------------------------------------------------------ line timing

/// Words a second the scratch timing assumes without a narrator.
const SILENT_WPS: f64 = 2.6;

/// A narration line being spoken, with where its sentences and phrases
/// start (measured sentence starts when the narrator has them; otherwise a
/// share of the line by characters). Phrase moments are estimates until the
/// Oliver takes are aligned: the cue sheets of the narrated draft check them.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    sentences: Vec<(usize, f64)>,
    phrase_marks: std::collections::HashMap<usize, f64>,
}

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
            let start = if measured.len() == offsets.len() {
                measured[j]
            } else {
                len * at as f64 / all
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
        phrase_marks: kit.narrator.as_ref().map(|n| n.phrase_marks(id)).unwrap_or_default(),
    })
}

impl Spoken {
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
        if let Some(time) = self.phrase_marks.get(&byte) {
            return Ok(self.start + time);
        }
        let i = self.text[..byte].chars().count();
        let all = self.text.chars().count().max(1);
        let j = self.sentences.iter().rposition(|&(at, _)| at <= i).unwrap_or(0);
        let (from, t0) = self.sentences[j];
        let (to, t1) = self.sentences.get(j + 1).copied().unwrap_or((all, self.len));
        let share = (i - from) as f64 / (to - from).max(1) as f64;
        Ok(self.start + t0 + (t1 - t0) * share)
    }

    pub fn end(&self) -> f64 {
        self.start + self.len
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }

    /// Wait (if need be) until sentence `j` begins.
    pub fn wait_sentence(&self, stage: &mut Stage<'_>, j: usize) -> fmn::Result<()> {
        let t = self.sentence(j)?;
        wait_until(stage, t)
    }

    /// Wait (if need be) until the last sentence begins (a question).
    pub fn wait_last(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.wait_sentence(stage, self.sentences.len() - 1)
    }

    /// Seconds from now until `phrase`, at least `floor`: the run time that
    /// lets an animation land on the words.
    pub fn until(&self, stage: &Stage<'_>, phrase: &str, floor: f64) -> fmn::Result<f64> {
        Ok((self.at(phrase)? - stage.scene().time().to_f64()).max(floor))
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

/// Wait `seconds`.
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

// ------------------------------------------------------------ colour roles

/// Input points and the input plane.
pub fn input() -> Srgb {
    Palette::blue()
}
/// Outputs and the output plane.
pub fn output() -> Srgb {
    Palette::green()
}
/// The recurring hero: loops, recipes, the bump.
pub fn hero() -> Srgb {
    Palette::hero()
}
/// Poles.
pub fn pole() -> Srgb {
    hex("#EC4E97")
}
/// The derivative's stretch and turn.
pub fn derivative() -> Srgb {
    Palette::teal()
}
/// Failure, refusal.
pub fn warn() -> Srgb {
    Palette::red()
}
/// Prediction cards.
pub fn quiz() -> Srgb {
    hex("#B189D6")
}
/// The background, for unpainted cells.
pub fn ground() -> Srgb {
    hex(explainer_kit::BACKGROUND)
}
/// Card fill.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

pub fn rgb([r, g, b]: [f64; 3]) -> Srgb {
    Srgb { r, g, b }
}

// ------------------------------------------------------------------ styles

pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

pub fn faded_line(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// An empty parent holding mobjects already in the arena.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

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

/// Tween a tracker at a steady rate (a walker, a draining ring).
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

/// Shift several mobjects together by `(dx, dy)`, `rt` seconds.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift([dx, dy, 0.0])?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Restroke `m` (and its family) in `color` at opacity `o`.
pub fn restroke(stage: &mut Stage<'_>, m: Mob, color: Srgb, o: f64) {
    stage.arena_mut().set_stroke(m, Some(color), None, Some(o), None, true);
}

/// The value of tracker `t` inside an updater.
pub fn value(s: &Arena, t: Mob) -> f64 {
    s.tracker_value(t).unwrap_or(0.0)
}

/// A rounded card behind `w` × `h` of content centred at `at`, outlined in `edge`.
pub fn card(stage: &mut Stage<'_>, w: f64, h: f64, at: Vec3, edge: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// Text on a card: returns `[card, text]`.
pub fn carded(
    kit: &Kit,
    stage: &mut Stage<'_>,
    s: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let t = kit.text(stage, s, size, color, at)?;
    let (w, h) = (stage.get_width(t), stage.get_height(t));
    let back = card(stage, w + 0.5, h + 0.35, at, edge)?;
    Ok([back, t])
}

/// Lines of text stacked on one card centred at `at`: the card first.
pub fn carded_lines(
    kit: &Kit,
    stage: &mut Stage<'_>,
    lines: &[&str],
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Srgb,
) -> fmn::Result<Vec<Mob>> {
    let pitch = size / 48.0 * 0.62;
    let top = at[1] + pitch * (lines.len() as f64 - 1.0) / 2.0;
    let mut texts = Vec::with_capacity(lines.len());
    let mut w: f64 = 0.0;
    for (i, l) in lines.iter().enumerate() {
        let t = kit.text(stage, l, size, color, v(at[0], top - pitch * i as f64))?;
        w = w.max(stage.get_width(t));
        texts.push(t);
    }
    let h = pitch * lines.len() as f64 + 0.3;
    let mut out = vec![card(stage, w + 0.5, h, at, edge)?];
    out.extend(texts);
    Ok(out)
}

// --------------------------------------------------------------- the plane

/// A window onto part of the complex plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub frame: Frame2,
}

impl Plane {
    /// `x × y` of the plane shown `width` wide, centred at `center` (height
    /// follows, so a unit is the same length both ways).
    pub fn new(center: Vec3, width: f64, x: (f64, f64), y: (f64, f64)) -> Self {
        let height = width * (y.1 - y.0) / (x.1 - x.0);
        Self {
            frame: Frame2 {
                center,
                width,
                height,
                x,
                y,
            },
        }
    }

    /// The scene point of `z`.
    pub fn p(&self, z: C) -> Vec3 {
        self.frame.p(z.re, z.im)
    }

    /// Scene units per unit of the plane.
    pub fn unit(&self) -> f64 {
        self.frame.width / (self.frame.x.1 - self.frame.x.0)
    }

    pub fn contains(&self, z: C) -> bool {
        z.re >= self.frame.x.0 && z.re <= self.frame.x.1 && z.im >= self.frame.y.0 && z.im <= self.frame.y.1
    }

    /// The two axes through 0 (where 0's lines cross the window), dim.
    pub fn axes(&self, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
        let f = self.frame;
        let mut out = Vec::new();
        if f.y.0 <= 0.0 && f.y.1 >= 0.0 {
            let l = polyline(vec![f.p(f.x.0, 0.0), f.p(f.x.1, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.8))
                .build();
            out.push(stage.arena_mut().add(l));
        }
        if f.x.0 <= 0.0 && f.x.1 >= 0.0 {
            let l = polyline(vec![f.p(0.0, f.y.0), f.p(0.0, f.y.1)])
                .style(faded_line(Palette::dim(), 2.0, 0.8))
                .build();
            out.push(stage.arena_mut().add(l));
        }
        Ok(out)
    }

    /// The window's frame in `color`.
    pub fn border(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        let b = self.frame.border(color)?;
        Ok(stage.arena_mut().add(b))
    }

    /// A polyline through the images of `zs`.
    pub fn path(&self, stage: &mut Stage<'_>, zs: &[C], color: Srgb, width: f64) -> fmn::Result<Mob> {
        let pts: Vec<Vec3> = zs.iter().map(|&z| self.p(z)).collect();
        Ok(stage
            .arena_mut()
            .add(polyline(pts).style(line_style(color, width)).build()))
    }

    /// The circle `|z − centre| = r`, closed, `n` segments.
    pub fn circle(&self, stage: &mut Stage<'_>, centre: C, r: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
        let mut zs = model::circle(centre, r, 240);
        zs.push(zs[0]);
        self.path(stage, &zs, color, width)
    }
}

/// A dot of `color` at `at`.
pub fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let d = Circle::new().radius(r).style(flat(color, 1.0)).build().moved_to(at);
    stage.arena_mut().add(d)
}

/// A zero: a white dot on a dark backing, so it reads on any colour.
pub fn zero_mark(stage: &mut Stage<'_>, at: Vec3) -> [Mob; 2] {
    [dot(stage, at, 0.11, ground()), dot(stage, at, 0.07, Palette::ink())]
}

/// A pole: a pink ring on a dark backing.
pub fn pole_mark(stage: &mut Stage<'_>, at: Vec3) -> [Mob; 2] {
    let back = dot(stage, at, 0.13, ground());
    let ring = Circle::new()
        .radius(0.085)
        .style(line_style(pole(), 5.0))
        .build()
        .moved_to(at);
    [back, stage.arena_mut().add(ring)]
}

/// Every exact zero and pole of `f` inside the plane, marked.
pub fn marks(stage: &mut Stage<'_>, f: Func, plane: &Plane) -> Vec<Mob> {
    let (x, y) = (plane.frame.x, plane.frame.y);
    let mut out = Vec::new();
    for z in f.zeros(x, y) {
        out.extend(zero_mark(stage, plane.p(z)));
    }
    for (p, _) in f.poles(x, y) {
        out.extend(pole_mark(stage, plane.p(p)));
    }
    out
}

// --------------------------------------------------------- colour pictures

/// The colour of each cell of the picture of `f` over `plane`, `cols` × `rows`
/// (row 0 at the top); unpainted (the background) where f is undefined.
/// `enhance` 0 is hue alone, 1 the full enhanced portrait.
pub fn portrait_colors(
    f: impl Fn(C) -> Option<C>,
    plane: &Plane,
    cols: usize,
    rows: usize,
    enhance: f64,
) -> Vec<Vec<Srgb>> {
    let (x, y) = (plane.frame.x, plane.frame.y);
    (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    let z = model::cell_input(x, y, cols, rows, r, c);
                    model::portrait(f(z), enhance).map(rgb).unwrap_or_else(ground)
                })
                .collect()
        })
        .collect()
}

/// Cells across a full-height picture: about 0.035 scene units each, fine
/// enough for the phase lines, and `rows` strips however many cells.
pub const CELL: f64 = 0.035;

/// The colour picture of `f` over `plane` as row strips.
pub fn portrait(stage: &mut Stage<'_>, f: Func, plane: &Plane, enhance: f64) -> fmn::Result<RowStrips> {
    let (cols, rows) = picture_cells(plane);
    let colors = portrait_colors(|z| f.eval(z), plane, cols, rows, enhance);
    let cell = (plane.frame.width / cols as f64, plane.frame.height / rows as f64);
    RowStrips::build(stage, v(plane.frame.left(), plane.frame.top()), cell, &colors)
}

/// How many cells the picture over `plane` has, each about [`CELL`].
pub fn picture_cells(plane: &Plane) -> (usize, usize) {
    let cols = (plane.frame.width / CELL).round().max(8.0) as usize;
    let rows = (plane.frame.height / CELL).round().max(8.0) as usize;
    (cols, rows)
}

/// `a` mixed towards `b` by `t` (0 to 1), channel by channel.
pub fn mix(a: Srgb, b: Srgb, t: f64) -> Srgb {
    let t = t.clamp(0.0, 1.0);
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

/// A colour picture that repaints in place (never rebuilt): `enhance`
/// blends plain hue (0) into the enhanced portrait (1), and [`Painter::to`]
/// crossfades the picture to another function, or to nothing (`None`, the
/// background). One updater, which repaints only when a tracker has moved,
/// in twentieths of `enhance` and twelfths of a crossfade (a repaint is one
/// evaluation per cell, two during a crossfade).
pub struct Painter {
    pub group: Mob,
    pub anchor: Mob,
    pub enhance: Mob,
    pub blend: Mob,
    funcs: std::rc::Rc<std::cell::RefCell<(Option<Func>, Option<Func>)>>,
}

impl Painter {
    /// The picture of `f` (or a blank plane) over `plane`, detached; its
    /// updater starts with [`Painter::show`].
    pub fn build(stage: &mut Stage<'_>, f: Option<Func>, plane: Plane, enhance0: f64) -> fmn::Result<Self> {
        let (cols, rows) = picture_cells(&plane);
        // copies of the window and sizes only, so it is used here and moved into the updater
        let paint = move |g: Option<Func>, e: f64| -> Vec<Vec<Srgb>> {
            match g {
                Some(g) => portrait_colors(|z| g.eval(z), &plane, cols, rows, e),
                None => vec![vec![ground(); cols]; rows],
            }
        };
        let cell = (plane.frame.width / cols as f64, plane.frame.height / rows as f64);
        let strips = RowStrips::build(
            stage,
            v(plane.frame.left(), plane.frame.top()),
            cell,
            &paint(f, enhance0),
        )?;
        let group = strips.group;
        let enhance = stage.add_value_tracker(enhance0);
        let blend = stage.add_value_tracker(1.0);
        let funcs = std::rc::Rc::new(std::cell::RefCell::new((f, f)));
        let shared = std::rc::Rc::clone(&funcs);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: (f64, f64, Option<Func>, Option<Func>) = (f64::NAN, f64::NAN, f, f);
        stage.add_updater(
            anchor,
            move |s, _| {
                let e = (value(s, enhance).clamp(0.0, 1.0) * 20.0).round() / 20.0;
                let b = (value(s, blend).clamp(0.0, 1.0) * 12.0).round() / 12.0;
                let (from, to) = *shared.borrow();
                let now = (e, b, from, to);
                if now == last {
                    return;
                }
                let colors = if b >= 1.0 || from == to {
                    paint(to, e)
                } else {
                    let (a, c) = (paint(from, e), paint(to, e));
                    a.iter()
                        .zip(&c)
                        .map(|(ra, rc)| ra.iter().zip(rc).map(|(&x, &y)| mix(x, y, b)).collect())
                        .collect()
                };
                let _ = strips.recolor(s, &colors);
                last = now;
            },
            true,
        )?;
        Ok(Self {
            group,
            anchor,
            enhance,
            blend,
            funcs,
        })
    }

    /// Put the picture on screen (and start its updater), fading in over `rt`.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        fade_in_all(stage, &[self.group], rt)
    }

    /// Crossfade the picture to `f` (or blank, `None`) in place, `rt` seconds.
    pub fn to(&self, stage: &mut Stage<'_>, f: Option<Func>, rt: f64) -> fmn::Result<()> {
        let current = self.funcs.borrow().1;
        *self.funcs.borrow_mut() = (current, f);
        stage.set_tracker_value(self.blend, 0.0)?;
        stage.play(tween(self.blend, 1.0, rt)?)
    }

    /// Blend towards the enhanced portrait (`e` = 1) or plain hue (0).
    pub fn enhance_to(&self, stage: &mut Stage<'_>, e: f64, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.enhance, e, rt)?)
    }

    /// Stop repainting and fade out.
    pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.anchor);
        fade_out_all(stage, &[self.group], rt)
    }
}

/// The colour wheel legend: twelve wedges of output direction, with the
/// real axis's two ends named, centred at `at`, radius `r`.
pub fn wheel_legend(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    let n = model::WHEEL.len();
    for k in 0..n {
        let (a0, a1) = (TAU * k as f64 / n as f64, TAU * (k + 1) as f64 / n as f64);
        let mut pts = vec![at];
        for j in 0..=6 {
            let a = a0 + (a1 - a0) * f64::from(j) / 6.0;
            pts.push(v(at[0] + r * a.cos(), at[1] + r * a.sin()));
        }
        let c = rgb(model::wheel((a0 + a1) / 2.0));
        out.push(stage.arena_mut().add(Polygon::new(pts).style(flat(c, 1.0)).build()));
    }
    out.push(kit.text(stage, "positive", 20.0, Palette::dim(), v(at[0] + r + 0.55, at[1]))?);
    out.push(kit.text(stage, "negative", 20.0, Palette::dim(), v(at[0] - r - 0.6, at[1]))?);
    Ok(out)
}

// ------------------------------------------------------- walking a loop

/// Where the walker's output is drawn: same direction as w, radius
/// R·|w|/(|w| + s), so every output fits the window and every turn around
/// zero is exactly the output's. The lower third says so where it is used.
pub fn squash(w: C, s: f64, radius: f64) -> C {
    let m = w.abs();
    if m == 0.0 { w } else { w.scale(radius / (m + s)) }
}

/// A walker on the circle `|z − centre| = r` in the input plane, and its
/// image f(z) in the output plane (squashed by `scale`, drawn within
/// `radius`): the input dot, the image dot, the image's trail and the arrow
/// from 0 to it, all following tracker `walked` (0 to 1 of the loop).
pub struct Walker {
    pub walked: Mob,
    pub anchor: Mob,
    pub mobs: Vec<Mob>,
    pub f: Func,
    pub centre: C,
    pub r: f64,
}

/// Samples per loop for the walker's trail and its turn count.
pub const WALK_SAMPLES: usize = 720;

#[allow(clippy::too_many_arguments)]
pub fn walker(
    stage: &mut Stage<'_>,
    f: Func,
    centre: C,
    r: f64,
    inp: Plane,
    out: Plane,
    scale: f64,
    radius: f64,
) -> fmn::Result<Walker> {
    let walked = stage.add_value_tracker(0.0);
    let z0 = centre + C::polar(r, 0.0);
    let w0 = f.eval(z0).map_or(model::ZERO, |w| squash(w, scale, radius));
    let in_dot = dot(stage, inp.p(z0), 0.08, input());
    let out_dot = dot(stage, out.p(w0), 0.08, output());
    let trail = stage.arena_mut().add(
        polyline(vec![out.p(w0), out.p(w0)])
            .style(line_style(output(), 4.0))
            .build(),
    );
    let arrow = stage.arena_mut().add(
        polyline(vec![out.p(model::ZERO), out.p(w0)])
            .style(line_style(Palette::ink(), 3.0))
            .build(),
    );
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = f64::NAN;
    stage.add_updater(
        anchor,
        move |s, _| {
            let t = value(s, walked).clamp(0.0, 1.0);
            if t == last {
                return;
            }
            let k = ((t * WALK_SAMPLES as f64).round() as usize).max(1);
            let pts: Vec<Vec3> = (0..=k)
                .filter_map(|j| {
                    let z = centre + C::polar(r, TAU * j as f64 / WALK_SAMPLES as f64);
                    f.eval(z).map(|w| out.p(squash(w, scale, radius)))
                })
                .collect();
            let z = centre + C::polar(r, TAU * t);
            s.move_to(in_dot, inp.p(z), ORIGIN);
            if let Some(&end) = pts.last() {
                s.move_to(out_dot, end, ORIGIN);
                let _ = s.set_points(arrow, polyline(vec![out.p(model::ZERO), end]).build().points());
            }
            if pts.len() >= 2 {
                let _ = s.set_points(trail, polyline(pts).build().points());
            }
            last = t;
        },
        true,
    )?;
    Ok(Walker {
        walked,
        anchor,
        mobs: vec![trail, arrow, in_dot, out_dot],
        f,
        centre,
        r,
    })
}

impl Walker {
    /// The net turns of the image so far (the sampled walk, for display: the
    /// counts the film states come from `facts`, certified).
    pub fn turns_so_far(f: Func, centre: C, r: f64, t: f64) -> f64 {
        let k = (t.clamp(0.0, 1.0) * WALK_SAMPLES as f64).round() as usize;
        let mut total = 0.0;
        let mut prev = f.eval(centre + C::polar(r, 0.0));
        for j in 1..=k {
            let w = f.eval(centre + C::polar(r, TAU * j as f64 / WALK_SAMPLES as f64));
            if let (Some(a), Some(b)) = (prev, w) {
                total += (b / a).arg();
            }
            prev = w;
        }
        total / TAU
    }

    /// A live turn counter for this walker, one decimal while it walks.
    pub fn counter(&self, kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
        let (f, centre, r, walked) = (self.f, self.centre, self.r, self.walked);
        kit.live_value(
            stage,
            move |s| Walker::turns_so_far(f, centre, r, value(s, walked)),
            1,
            true,
            34.0,
            hero(),
            at,
            ORIGIN,
        )
    }

    /// Put the walker on screen (its updater runs from now).
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        fade_in_all(stage, &self.mobs, rt)
    }

    /// Walk the whole loop in `rt` seconds at a steady pace.
    pub fn walk(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.play(tween_linear(self.walked, 1.0, rt)?)
    }

    /// Everything the walker drew, for a fade (updater cleared first).
    pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.clear_updaters(self.anchor, true);
        fade_out_all(stage, &self.mobs, rt)
    }
}

// ------------------------------------------------------------ predictions

/// Points of a ring at `at`, radius `r`, the first `share` of a turn from the top.
fn ring_points(at: Vec3, r: f64, share: f64) -> Vec<Vec3> {
    let n = ((96.0 * share.clamp(0.0, 1.0)).ceil() as usize).max(2);
    (0..=n)
        .map(|k| {
            let a = TAU / 4.0 - TAU * share.clamp(0.0, 1.0) * k as f64 / n as f64;
            v(at[0] + r * a.cos(), at[1] + r * a.sin())
        })
        .collect()
}

/// The prediction (⟨predict⟩): as the question's last sentence begins, the
/// violet card with the question's `lines`; once it is spoken, the
/// "predict" mark for the score and a ring draining over the storyboard's
/// listen window (`seconds`, which `chapters::PREDICTS` pins to the
/// storyboard). The card stays (the caller clears it with the answer).
pub fn predict(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken, lines: &[&str], at: Vec3) -> fmn::Result<Vec<Mob>> {
    let seconds = super::PREDICTS
        .iter()
        .find(|(id, _)| *id == line.id)
        .map(|&(_, s)| s)
        .ok_or_else(|| fail(format!("{} is not a prediction in chapters::PREDICTS", line.id)))?;
    line.wait_last(stage)?;
    let shown = carded_lines(kit, stage, lines, 28.0, quiz(), at, quiz())?;
    fade_in_all(stage, &shown, 0.4)?;
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let h = stage.get_height(shown[0]);
    let ring_at = v(at[0], at[1] - h / 2.0 - 0.45);
    let track = Circle::new()
        .radius(0.26)
        .style(line_style(Palette::faint(), 6.0))
        .build()
        .moved_to(ring_at);
    let track = stage.arena_mut().add(track);
    let ring = stage.arena_mut().add(
        polyline(ring_points(ring_at, 0.26, 1.0))
            .style(line_style(quiz(), 7.0))
            .build(),
    );
    stage.add_to_scene(track)?;
    stage.add_to_scene(ring)?;
    // a separate clock drives the ring, as the quiz countdowns do
    let clock = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(clock)?;
    let mut elapsed = 0.0;
    stage.add_dt_updater(
        clock,
        move |s, _, dt| {
            elapsed += dt;
            let share = (1.0 - elapsed / seconds).max(0.0);
            let _ = s.set_points(ring, polyline(ring_points(ring_at, 0.26, share)).build().points());
        },
        true,
    )?;
    let start = stage.scene().time().to_f64();
    kit.listen(stage, seconds, &format!("{}:{}", kit.chapter, line.id))?;
    wait_until(stage, start + seconds)?;
    stage.clear_updaters(clock, true);
    stage.remove_from_scene(clock);
    fade_out_all(stage, &[ring, track], 0.25)?;
    Ok(shown)
}

/// A lower third ("precisely:") brought in now; returns it.
pub fn precisely_now(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    let m = explainer_kit::props::precisely(kit, stage, text)?;
    fade_in_all(stage, &[m], 0.4)?;
    Ok(m)
}

/// "names to search:" brought in now; returns it.
pub fn names_now(kit: &Kit, stage: &mut Stage<'_>, names: &[&str]) -> fmn::Result<Mob> {
    let m = explainer_kit::props::names_to_search(kit, stage, names)?;
    fade_in_all(stage, &[m], 0.4)?;
    Ok(m)
}

// ------------------------------------------------ loop sums, tip to tail

/// The running sums of a loop sum's products, 0 first: the corners of the
/// tip-to-tail chain (its last point is the loop sum).
pub fn chain_points(products: &[C]) -> Vec<C> {
    let mut out = Vec::with_capacity(products.len() + 1);
    let mut at = model::ZERO;
    out.push(at);
    for &p in products {
        at = at + p;
        out.push(at);
    }
    out
}

/// The scene point of chain corner 0 and the scene units per unit that fit
/// `pts` inside a `w` × `h` box centred at `at`, at most `max_scale`.
pub fn fit_chain(pts: &[C], at: Vec3, w: f64, h: f64, max_scale: f64) -> (Vec3, f64) {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for z in pts {
        x0 = x0.min(z.re);
        x1 = x1.max(z.re);
        y0 = y0.min(z.im);
        y1 = y1.max(z.im);
    }
    let scale = (w / (x1 - x0).max(1e-9)).min(h / (y1 - y0).max(1e-9)).min(max_scale);
    let mid = C::new((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    (v(at[0] - scale * mid.re, at[1] - scale * mid.im), scale)
}

/// Where chain point `z` is drawn.
pub fn chain_at(origin: Vec3, scale: f64, z: C) -> Vec3 {
    v(origin[0] + scale * z.re, origin[1] + scale * z.im)
}

/// One arrow per product, laid tip to tail from `origin`, detached.
pub fn chain_arrows(stage: &mut Stage<'_>, origin: Vec3, scale: f64, pts: &[C], color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::with_capacity(pts.len());
    for w in pts.windows(2) {
        let (a, b) = (chain_at(origin, scale, w[0]), chain_at(origin, scale, w[1]));
        if ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt() < 1e-3 {
            continue;
        }
        let arrow = Arrow::new(a, b).thickness(3.0).color(color).build()?;
        out.push(stage.arena_mut().add(arrow));
    }
    Ok(out)
}

/// Lay `arrows` down one after another, `each` seconds apiece.
pub fn lay_down(stage: &mut Stage<'_>, arrows: &[Mob], each: f64) -> fmn::Result<()> {
    for &m in arrows {
        fade_in_all(stage, &[m], each)?;
    }
    Ok(())
}

/// A dashed segment from `a` to `b` (18 dashes), detached.
pub fn dashed(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let n = 18;
    let at = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
    let mut dashes = Vec::with_capacity(n);
    for k in 0..n {
        let t0 = k as f64 / n as f64;
        let t1 = (k as f64 + 0.55) / n as f64;
        dashes.push(
            stage
                .arena_mut()
                .add(polyline(vec![at(t0), at(t1)]).style(line_style(color, 3.0)).build()),
        );
    }
    group(stage, &dashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{I, ZERO};

    #[test]
    fn squashing_keeps_direction_and_fits() {
        for w in [C::new(3.0, 4.0), C::new(-1e6, 2.0), C::new(1e-9, -1e-9)] {
            let s = squash(w, 1.0, 1.4);
            assert!(s.abs() < 1.4);
            assert!((s.arg() - w.arg()).abs() < 1e-12);
        }
        assert_eq!(squash(ZERO, 1.0, 1.4), ZERO);
    }

    #[test]
    fn the_walkers_display_count_agrees_with_the_facts() {
        // the displayed count at the end of the walk is the certified one
        assert!((Walker::turns_so_far(Func::Bump, I, 0.5, 1.0) + 1.0).abs() < 1e-9);
        assert!((Walker::turns_so_far(Func::Quintic, ZERO, 1.0, 1.0) - 1.0).abs() < 1e-9);
        assert!((Walker::turns_so_far(Func::Quintic, ZERO, 3.0, 1.0) - 5.0).abs() < 1e-9);
        assert!((Walker::turns_so_far(Func::Moebius, ZERO, 1.7, 1.0)).abs() < 1e-9);
    }

    #[test]
    fn a_plane_keeps_units_square_and_maps_its_corners() {
        let p = Plane::new(v(0.0, 0.0), 6.0, (-2.0, 2.0), (-1.0, 1.0));
        assert!((p.frame.height - 3.0).abs() < 1e-12);
        let c = p.p(C::new(2.0, 1.0));
        assert!((c[0] - 3.0).abs() < 1e-12 && (c[1] - 1.5).abs() < 1e-12);
        assert!((p.unit() - 1.5).abs() < 1e-12);
    }

    #[test]
    fn a_full_picture_stays_within_the_strip_budget() {
        // the widest picture the chapters draw: 6.4 × 6.4 scene units
        let p = Plane::new(v(0.0, 0.0), 6.4, (-2.0, 2.0), (-2.0, 2.0));
        let (cols, rows) = picture_cells(&p);
        assert!(rows <= 200 && cols <= 200, "{cols} × {rows}");
        assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT);
    }

    #[test]
    fn the_ring_drains_from_the_top() {
        let full = ring_points(v(0.0, 0.0), 1.0, 1.0);
        assert!((full[0][1] - 1.0).abs() < 1e-12);
        let half = ring_points(v(0.0, 0.0), 1.0, 0.5);
        let end = half.last().unwrap();
        assert!((end[1] + 1.0).abs() < 1e-9);
    }
}
