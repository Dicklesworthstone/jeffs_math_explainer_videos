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
use explainer_kit::{Facts, Frame2, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::mobject::{ImageColorSpace, ImageResource, ImageSampler, ImageWrap, ShapeTag, Stage as Arena};
use fmn::prelude::*;

use crate::model::{self, C, Func};
use crate::plate_layout;

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

/// The image records in `m`'s family, `m` included. A raster plate is one image quad
/// (`Picture::build`, raster branch) whose opacity is its own one-lane `opacity` field, which
/// the camera renderer multiplies into every texel; the engine's fades write only the alpha of
/// `*rgba` fields, so on their own they leave a plate at full strength for the whole fade and
/// the fade out's remover takes it away at the end (cc2-plate-fade-investigation246).
fn image_records(stage: &Stage<'_>, m: Mob) -> Vec<Mob> {
    stage
        .family(m)
        .into_iter()
        .filter(|&k| stage.get(k).is_some_and(|e| e.image_resource().is_some()))
        .collect()
}

/// A plate's own fade, played in the same `play` as the engine fade of `owner` and listed
/// after it, so on every frame it writes last: the same run time (the caller's `.rt`) and the
/// same rate (`AnimConfig::default`'s smooth, as the fades use), so it adds no frame and moves
/// no cue. `rising` goes 0 to 1. A fade out ends, as the engine's gives its alphas back, with
/// the lane back at 1 in the same finish pass where the remover takes the plate off the scene.
fn plate_fade(owner: Mob, images: Vec<Mob>, rising: bool) -> fmn::animation::UpdateFromFunc {
    fmn::animation::UpdateFromFunc::new_alpha(owner, move |s: &mut Arena, _: Mob, a: f64| {
        let shown = (if rising { a } else { 1.0 - a }).clamp(0.0, 1.0);
        for &k in &images {
            s.set_scalar_lanes(k, "opacity", shown, false);
        }
    })
    .with_config(AnimConfig {
        name: "PlateFade".to_owned(),
        final_alpha_value: if rising { 1.0 } else { 0.0 },
        ..AnimConfig::default()
    })
}

pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
        let images = image_records(stage, m);
        if !images.is_empty() {
            anims.push(stage.prepare(plate_fade(m, images, true).rt(rt))?);
        }
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
        let images = image_records(stage, m);
        if !images.is_empty() {
            anims.push(stage.prepare(plate_fade(m, images, false).rt(rt))?);
        }
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
        .style(Style::default().fill(panel(), 1.0).stroke(edge, 2.0, 1.0))
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

    #[cfg(test)]
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

/// "Re" and "Im" just outside `plane`'s window, level with whichever axes cross it, so no
/// stroke runs under them and they read beside a colour picture. Detached.
pub fn axis_names(kit: &Kit, stage: &mut Stage<'_>, plane: &Plane) -> fmn::Result<Vec<Mob>> {
    let f = plane.frame;
    let mut out = Vec::new();
    if f.y.0 <= 0.0 && f.y.1 >= 0.0 {
        let y = plane.p(C::new(f.x.1, 0.0))[1];
        out.push(kit.text(stage, "Re", 20.0, Palette::dim(), v(f.right() + 0.24, y))?);
    }
    if f.x.0 <= 0.0 && f.x.1 >= 0.0 {
        let x = plane.p(C::new(0.0, f.y.1))[0];
        out.push(kit.text(stage, "Im", 20.0, Palette::dim(), v(x, f.top() + 0.18))?);
    }
    Ok(out)
}

/// The corners of a solid arrowhead with its point at `tip`, coming from `from`, `size` long
/// and `size` wide: the end of a curved mark, whose last chord is too short for an `Arrow`.
pub fn arrow_head_points(tip: Vec3, from: Vec3, size: f64) -> Vec<Vec3> {
    let (dx, dy) = (tip[0] - from[0], tip[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let back = v(tip[0] - ux * size, tip[1] - uy * size);
    let half = size / 2.0;
    vec![
        v(tip[0], tip[1]),
        v(back[0] - uy * half, back[1] + ux * half),
        v(back[0] + uy * half, back[1] - ux * half),
    ]
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

/// How a colour picture is laid out and drawn. [`PLATE`] is what the film uses; the others are
/// there for `examples/plate_lab.rs` to compare on real renders, and for root to choose from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlateStyle {
    /// Use a native image quad with two-dimensional texture filtering.
    pub raster: bool,
    /// The height of a row and the width of a column this style aims for, in scene units.
    pub row: f64,
    pub col: f64,
    /// The renderer's antialiasing band on each strip, in pixels. Not scene units.
    pub aa_px: f64,
    /// How much wider than its row each strip's stroke is, in scene units.
    pub overlap: f64,
    /// Samples per axis in a cell, whose encoded colours are averaged; 1 is the cell's centre.
    pub samples: u32,
    /// The colour runs from cell centre to cell centre along a row ([`plate_layout`]), in place
    /// of the kit's flat cell with a jump at each edge.
    pub blend: bool,
}

/// The pictures as they are: rows exactly a cell high with a half-pixel band, flat cells.
pub const PLATE_NOW: PlateStyle = PlateStyle {
    raster: false,
    row: CELL,
    col: CELL,
    aa_px: 0.5,
    overlap: 0.0,
    samples: 1,
    blend: false,
};
/// How much wider than its row a tiling strip is: enough to swallow the f32 rounding of a row's
/// height and width, about 0.03 px at 4K, so no sample can fall in a hole between two rows.
pub const ROW_GUARD: f64 = 1e-4;
/// Rows that tile: no band, so a row's coverage is all or nothing, and at each pixel height one
/// row wins. No background shows at a boundary, at any opacity.
#[allow(dead_code)] // for examples/plate_lab.rs, until root chooses
pub const PLATE_TILE: PlateStyle = PlateStyle {
    aa_px: 0.0,
    overlap: ROW_GUARD,
    ..PLATE_NOW
};
/// Tiling rows, colours running along each row.
#[allow(dead_code)] // for examples/plate_lab.rs, until root chooses
pub const PLATE_BLEND: PlateStyle = PlateStyle {
    blend: true,
    ..PLATE_TILE
};
/// Those, with rows half as high. Costs twice the model evaluations and no more per row.
#[allow(dead_code)] // for examples/plate_lab.rs, until root chooses
pub const PLATE_ROWS2: PlateStyle = PlateStyle {
    row: CELL / 2.0,
    ..PLATE_BLEND
};
/// Those, with cells half as high and half as wide: four times the evaluations, about twice the
/// curves in a row, and so about twice the raster work per pixel.
#[allow(dead_code)] // for examples/plate_lab.rs, until root chooses
pub const PLATE_CELLS2: PlateStyle = PlateStyle {
    row: CELL / 2.0,
    col: CELL / 2.0,
    ..PLATE_BLEND
};
/// Cells a quarter the size: sixteen times the evaluations, about four times the raster work per
/// pixel. A bracket for what finer cells cost, not a proposal.
pub const PLATE_CELLS4: PlateStyle = PlateStyle {
    row: CELL / 4.0,
    col: CELL / 4.0,
    ..PLATE_BLEND
};
/// A mathematically evaluated raster, finer than a display pixel at 4K
/// (360 samples per scene unit versus 270 pixels per unit for an eight-unit
/// camera height). Native filtering acts in both axes. Clamp the edges so
/// the opposite side of the complex plane cannot bleed into this window.
pub const PLATE_RASTER: PlateStyle = PlateStyle {
    raster: true,
    row: 1.0 / 360.0,
    col: 1.0 / 360.0,
    ..PLATE_CELLS4
};
/// The style the film's pictures use. Actual rendered quality is reviewed separately.
pub const PLATE: PlateStyle = PLATE_RASTER;

/// Rows that meet as `style` says. At the engine's default band of 1.5 px the background shows
/// through at each boundary between rows, which reads as fine stripes across the picture; a
/// half-pixel band narrows those stripes and does not remove them (the same fix as
/// `raptorq_gf256`'s `crisp`). `style.aa_px` and `style.overlap` are what [`PlateStyle`] compares.
fn crisp(stage: &mut Stage<'_>, rows: &[Mob], cell_h: f64, style: &PlateStyle) {
    for &m in rows {
        stage.set_anti_alias_width(m, style.aa_px, false);
        if style.overlap != 0.0 {
            // stroke width in Reference units (0.01 scene units each), every record the same
            stage
                .arena_mut()
                .set_scalar_lanes(m, "stroke_width", (cell_h + style.overlap) / 0.01, false);
        }
    }
}

/// A colour picture's mobjects: one strip per row, recoloured in place.
pub struct Picture {
    pub group: Mob,
    painted: Painted,
}

enum Painted {
    /// A retained, immutable RGBA image resource, replaced only on repaint.
    Raster { mobs: [Mob; 1], cols: usize, rows: usize },
    /// The kit's flat cells.
    Steps(RowStrips),
    /// Colours running along each row.
    Blend { rows: Vec<Mob>, cols: usize },
}

impl Picture {
    /// Build the picture with its top-left corner at `top_left`, cells `(width, height)`,
    /// `colors[row][col]`, as `style` says. Every row the same, nonzero length.
    pub fn build(
        stage: &mut Stage<'_>,
        top_left: Vec3,
        (cell_w, cell_h): (f64, f64),
        colors: &[Vec<Srgb>],
        style: &PlateStyle,
    ) -> fmn::Result<Self> {
        if style.raster {
            let resource = picture_resource(colors)?;
            let (cols, rows) = (resource.width() as usize, resource.height() as usize);
            let height = cell_h * rows as f64;
            let image = ImageMobject::from_resource(resource)
                .map_err(|e| fail(e.to_string()))?
                .with_height(height);
            let image_width = image.scene_width();
            let mut object = Mobject::from(image);
            let mut points = object
                .buffer
                .read_column("point")
                .ok_or_else(|| fail("image points missing".into()))?;
            for point in points.as_chunks_mut::<3>().0 {
                point[0] = (f64::from(point[0]) * cell_w * cols as f64 / image_width
                    + top_left[0]
                    + cell_w * cols as f64 / 2.0) as f32;
                point[1] += (top_left[1] - height / 2.0) as f32;
            }
            if !object.buffer.write_range("point", 0, &points) {
                return Err(fail("image geometry update failed".into()));
            }
            let mob = stage.arena_mut().add(object);
            return Ok(Self {
                group: group(stage, &[mob])?,
                painted: Painted::Raster {
                    mobs: [mob],
                    cols,
                    rows,
                },
            });
        }
        let (group_mob, painted) = if style.blend {
            let (rows, cols) = blended_rows(stage, top_left, (cell_w, cell_h), colors)?;
            (group(stage, &rows)?, Painted::Blend { rows, cols })
        } else {
            let strips = RowStrips::build(stage, top_left, (cell_w, cell_h), colors)?;
            (strips.group, Painted::Steps(strips))
        };
        let picture = Self {
            group: group_mob,
            painted,
        };
        crisp(stage, picture.rows(), cell_h, style);
        Ok(picture)
    }

    /// The row strips, top to bottom.
    pub fn rows(&self) -> &[Mob] {
        match &self.painted {
            Painted::Raster { mobs, .. } => mobs,
            Painted::Steps(strips) => &strips.rows,
            Painted::Blend { rows, .. } => rows,
        }
    }

    /// Recolour every cell in place (callable from an updater). `false` if the colours do not
    /// fit the picture or a row is gone.
    pub fn recolor(&self, arena: &mut Arena, colors: &[Vec<Srgb>]) -> bool {
        match &self.painted {
            Painted::Raster { mobs, cols, rows } => {
                if colors.len() != *rows || colors.iter().any(|r| r.len() != *cols) {
                    return false;
                }
                match picture_resource(colors) {
                    Ok(resource) => arena.set_image_resource(mobs[0], Some(resource)).is_ok(),
                    Err(_) => false,
                }
            }
            Painted::Steps(strips) => strips.recolor(arena, colors),
            Painted::Blend { rows, cols } => {
                if colors.len() != rows.len() || colors.iter().any(|r| r.len() != *cols) {
                    return false;
                }
                rows.iter().zip(colors).all(|(&row, cs)| {
                    arena
                        .get_mut(row)
                        .is_some_and(|e| e.buffer.write_range("stroke_rgba", 0, blended_rgba(cs).as_flattened()))
                })
            }
        }
    }
}

fn picture_resource(colors: &[Vec<Srgb>]) -> fmn::Result<ImageResource> {
    let cols = colors.first().map_or(0, Vec::len);
    if cols == 0 || colors.iter().any(|row| row.len() != cols) {
        return Err(fail("a raster needs nonempty equal-width rows".into()));
    }
    let width = u32::try_from(cols).map_err(|e| fail(e.to_string()))?;
    let height = u32::try_from(colors.len()).map_err(|e| fail(e.to_string()))?;
    let mut rgba = Vec::with_capacity(cols * colors.len() * 4);
    for row in colors {
        for c in row {
            rgba.extend([c.r, c.g, c.b].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
            rgba.push(255);
        }
    }
    ImageResource::rgba8(
        width,
        height,
        rgba,
        ImageColorSpace::Srgb,
        ImageSampler {
            wrap_u: ImageWrap::ClampToEdge,
            wrap_v: ImageWrap::ClampToEdge,
        },
    )
    .map_err(|e| fail(e.to_string()))
}

fn blended_rgba(row: &[Srgb]) -> Vec<[f32; 4]> {
    let encoded: Vec<[f64; 3]> = row.iter().map(|c| [c.r, c.g, c.b]).collect();
    plate_layout::blend_rgba(&encoded)
}

/// One blended strip per row, built as `RowStrips::build` builds its strips.
fn blended_rows(
    stage: &mut Stage<'_>,
    top_left: Vec3,
    (cell_w, cell_h): (f64, f64),
    colors: &[Vec<Srgb>],
) -> fmn::Result<(Vec<Mob>, usize)> {
    let cols = colors.first().map_or(0, Vec::len);
    if cols == 0 || colors.iter().any(|r| r.len() != cols) {
        return Err(fail("a picture needs rows of the same, nonzero length".to_owned()));
    }
    let mut rows = Vec::with_capacity(colors.len());
    for (i, row) in colors.iter().enumerate() {
        let y = top_left[1] - cell_h * (i as f64 + 0.5);
        let points = plate_layout::blend_points(top_left[0], y, cell_w, cell_h, cols);
        let vertices = points.len().div_ceil(2);
        let style = Style::default().fill(row[0], 0.0).stroke(row[0], cell_h / 0.01, 1.0);
        let vm = VMobject::from_points(points)
            .with_style(style)
            .with_shape(ShapeTag::Polyline {
                vertices,
                closed: false,
            });
        let mut m = Mobject::from(vm);
        let rgba = blended_rgba(row);
        if m.buffer.len() != rgba.len() || !m.buffer.write_range("stroke_rgba", 0, rgba.as_flattened()) {
            return Err(fail(
                "a blended strip's colour lane does not match its points".to_owned(),
            ));
        }
        rows.push(stage.arena_mut().add(m));
    }
    Ok((rows, cols))
}

/// The colour of each cell, as [`portrait_colors`], with `samples` per axis averaged in the
/// encoded values. Above one sample an edge between bands softens by a cell instead of stepping
/// by one.
fn plate_colors(
    f: impl Fn(C) -> Option<C>,
    plane: &Plane,
    cols: usize,
    rows: usize,
    enhance: f64,
    samples: u32,
) -> Vec<Vec<Srgb>> {
    if samples <= 1 {
        return portrait_colors(f, plane, cols, rows, enhance);
    }
    let (x, y) = (plane.frame.x, plane.frame.y);
    let (dx, dy) = ((x.1 - x.0) / cols as f64, (y.1 - y.0) / rows as f64);
    let n = f64::from(samples);
    let offset = |k: u32| (f64::from(k) + 0.5) / n - 0.5;
    (0..rows)
        .map(|r| {
            (0..cols)
                .map(|c| {
                    let z = model::cell_input(x, y, cols, rows, r, c);
                    let mut mean = Srgb { r: 0.0, g: 0.0, b: 0.0 };
                    for i in 0..samples {
                        for j in 0..samples {
                            let at = C::new(z.re + dx * offset(i), z.im + dy * offset(j));
                            let one = model::portrait(f(at), enhance).map(rgb).unwrap_or_else(ground);
                            mean.r += one.r / (n * n);
                            mean.g += one.g / (n * n);
                            mean.b += one.b / (n * n);
                        }
                    }
                    mean
                })
                .collect()
        })
        .collect()
}

/// The colour picture of `f` over `plane` as row strips.
pub fn portrait(stage: &mut Stage<'_>, f: Func, plane: &Plane, enhance: f64) -> fmn::Result<Picture> {
    portrait_styled(stage, f, plane, enhance, &PLATE)
}

/// [`portrait`] as `style` says.
pub fn portrait_styled(
    stage: &mut Stage<'_>,
    f: Func,
    plane: &Plane,
    enhance: f64,
    style: &PlateStyle,
) -> fmn::Result<Picture> {
    let (cols, rows) = picture_cells(plane, style);
    let colors = plate_colors(|z| f.eval(z), plane, cols, rows, enhance, style.samples);
    let cell = (plane.frame.width / cols as f64, plane.frame.height / rows as f64);
    Picture::build(stage, v(plane.frame.left(), plane.frame.top()), cell, &colors, style)
}

/// How many cells the picture over `plane` has under `style`.
pub fn picture_cells(plane: &Plane, style: &PlateStyle) -> (usize, usize) {
    let cols = (plane.frame.width / style.col).round().max(8.0) as usize;
    let rows = (plane.frame.height / style.row).round().max(8.0) as usize;
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

/// A colour picture with continuous shading and crossfades. Cache each function
/// at both enhancement endpoints; interpolate cached colours at the actual
/// tracker value rather than quantizing motion or reevaluating special functions.
fn portrait_mix(a0: Srgb, a1: Srgb, b0: Srgb, b1: Srgb, enhancement: f64, blend: f64) -> Srgb {
    mix(mix(a0, a1, enhancement), mix(b0, b1, enhancement), blend)
}

type PortraitEndpoints = (Option<Func>, [Vec<Vec<Srgb>>; 2]);

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
        Self::build_styled(stage, f, plane, enhance0, PLATE)
    }

    /// [`Painter::build`] as `style` says.
    pub fn build_styled(
        stage: &mut Stage<'_>,
        f: Option<Func>,
        plane: Plane,
        enhance0: f64,
        style: PlateStyle,
    ) -> fmn::Result<Self> {
        let (cols, rows) = picture_cells(&plane, &style);
        // copies of the window and sizes only, so it is used here and moved into the updater
        let paint = move |g: Option<Func>, e: f64| -> Vec<Vec<Srgb>> {
            match g {
                Some(g) => plate_colors(|z| g.eval(z), &plane, cols, rows, e, style.samples),
                None => vec![vec![ground(); cols]; rows],
            }
        };
        let cell = (plane.frame.width / cols as f64, plane.frame.height / rows as f64);
        let picture = Picture::build(
            stage,
            v(plane.frame.left(), plane.frame.top()),
            cell,
            &paint(f, enhance0),
            &style,
        )?;
        let group = picture.group;
        let enhance = stage.add_value_tracker(enhance0);
        let blend = stage.add_value_tracker(1.0);
        let funcs = std::rc::Rc::new(std::cell::RefCell::new((f, f)));
        let shared = std::rc::Rc::clone(&funcs);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: (f64, f64, Option<Func>, Option<Func>) = (f64::NAN, f64::NAN, f, f);
        let mut cache: Vec<PortraitEndpoints> = Vec::new();
        stage.add_updater(
            anchor,
            move |s, _| {
                let e = value(s, enhance).clamp(0.0, 1.0);
                let b = value(s, blend).clamp(0.0, 1.0);
                let (from, to) = *shared.borrow();
                let now = (e, b, from, to);
                if now == last {
                    return;
                }
                // Keep at most the two currently needed functions. Enhancement
                // is affine in model::portrait, and supersampling is linear, so
                // endpoint interpolation preserves the original colour formula.
                cache.retain(|(g, _)| *g == from || *g == to);
                for g in [from, to] {
                    if !cache.iter().any(|(key, _)| *key == g) {
                        cache.push((g, [paint(g, 0.0), paint(g, 1.0)]));
                    }
                }
                let a = &cache.iter().find(|(g, _)| *g == from).unwrap().1;
                let c = &cache.iter().find(|(g, _)| *g == to).unwrap().1;
                let colors = (0..rows)
                    .map(|row| {
                        (0..cols)
                            .map(|col| {
                                portrait_mix(a[0][row][col], a[1][row][col], c[0][row][col], c[1][row][col], e, b)
                            })
                            .collect()
                    })
                    .collect::<Vec<Vec<Srgb>>>();
                let _ = picture.recolor(s, &colors);
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
        stage.play(tween(self.blend, 1.0, rt)?)?;
        Ok(())
    }

    /// Blend towards the enhanced portrait (`e` = 1) or plain hue (0).
    pub fn enhance_to(&self, stage: &mut Stage<'_>, e: f64, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.enhance, e, rt)?)?;
        Ok(())
    }

    /// Stop repainting and fade out.
    pub fn leave(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.anchor);
        fade_out_all(stage, &[self.group], rt)
    }
}

/// The colour wheel legend: twelve wedges of output direction, each centred on its anchor so
/// that its edges fall where the picture's phase lines do, with the real axis's two ends
/// named, centred at `at`, radius `r`.
pub fn wheel_legend(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    let n = model::WHEEL.len();
    for k in 0..n {
        let edge = |j: usize| TAU * (j as f64 - model::BAND_OFFSET) / n as f64;
        let (a0, a1) = (edge(k), edge(k + 1));
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
        stage.play(tween_linear(self.walked, 1.0, rt)?)?;
        Ok(())
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

/// A reading window (`chapters::READING_WINDOWS`): the cue finishes, then the kit's breath,
/// then the picture holds still for the authored seconds, so the card on screen can be read
/// with nothing else being said. It draws and clears nothing; the next cue, or at a chapter's
/// last cue the closing fade after its music breath, clears the card. Waiting for
/// `line.end()` first is what holds a render with no narrator, where `kit.hold` does nothing.
/// The wait is a plain one: no music window, and no silence in any take.
pub fn reading_window(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<()> {
    let seconds = super::READING_WINDOWS
        .iter()
        .find(|(id, _)| *id == line.id)
        .map(|&(_, s)| s)
        .ok_or_else(|| fail(format!("{} is not in chapters::READING_WINDOWS", line.id)))?;
    wait_until(stage, line.end())?;
    kit.hold(stage)?;
    pause(stage, seconds)
}

/// A lower third ("precisely:") brought in now; returns it.
pub fn precisely_card(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    let m = explainer_kit::props::precisely(kit, stage, text)?;
    // The shared lower third's first child is its background rectangle.
    // Keep plot strokes and colour fields from showing through settled text.
    let back = stage
        .get(m)
        .ok_or_else(|| fail("missing lower third".into()))?
        .submobjects()
        .first()
        .copied()
        .ok_or_else(|| fail("missing lower-third background".into()))?;
    stage.arena_mut().set_fill(back, None, Some(1.0), None, false);
    Ok(m)
}

pub fn precisely_now(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    let m = precisely_card(kit, stage, text)?;
    fade_in_all(stage, &[m], 0.4)?;
    Ok(m)
}

/// A lower third brought in now that holds the first part of the taller `whole`, which is
/// built but not yet shown. Bands sit on the foot of the frame, so this shorter one is lifted
/// until it starts where `whole` will: its first line does not move when `whole` replaces it.
pub fn precisely_lifted(kit: &Kit, stage: &mut Stage<'_>, text: &str, whole: Mob) -> fmn::Result<Mob> {
    let m = precisely_card(kit, stage, text)?;
    let lift = stage.get_height(whole) - stage.get_height(m);
    let at = stage.get_center(m);
    stage.move_to(m, v(at[0], at[1] + lift), ORIGIN);
    fade_in_all(stage, &[m], 0.4)?;
    Ok(m)
}

/// `whole` comes in over `said`, and then `said` goes: a lower third's card is nearly opaque,
/// so `said` leaves behind it. In all 0.4 s, as long as [`precisely_now`] takes.
pub fn precisely_over(stage: &mut Stage<'_>, said: Mob, whole: Mob) -> fmn::Result<()> {
    fade_in_all(stage, &[whole], 0.3)?;
    fade_out_all(stage, &[said], 0.1)
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
        // buff 0: `Arrow::new` otherwise trims 0.25 from each end, and a chain's arrows are
        // shorter than that, so they would not touch, or would point backwards
        let arrow = Arrow::new(a, b).thickness(3.0).color(color).buff(0.0).build()?;
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
    #[test]
    fn portrait_crossfade_resolves_sub_twelfth_frame_changes() {
        let black = super::rgb([0.0; 3]);
        let white = super::rgb([1.0; 3]);
        let a = super::portrait_mix(black, black, white, white, 0.5, 0.500);
        let b = super::portrait_mix(black, black, white, white, 0.5, 0.501);
        assert!((a.r - 0.500).abs() < 1e-12);
        assert!((b.r - 0.501).abs() < 1e-12);
        assert!(b.r > a.r);
        let shade = super::portrait_mix(black, white, black, white, 0.501, 0.2);
        assert!((shade.r - 0.501).abs() < 1e-12);
    }

    #[test]
    fn cached_endpoints_match_direct_function_colours() {
        let functions = [
            Func::Identity,
            Func::Square,
            Func::Recip,
            Func::Bump,
            Func::Cubic,
            Func::ZeroDoublePole,
            Func::Exp,
            Func::Sin,
            Func::Tan,
            Func::Gamma,
            Func::Zeta,
            Func::Moebius,
            Func::Quintic,
            Func::AbsSquare,
            Func::SinRecip,
            Func::Lacunary,
        ];
        let points = [ZERO, I, C::new(0.3, 0.2), C::new(-1.2, 0.7), C::new(2.1, -0.4)];
        for f in functions {
            for z in points {
                let at = |e| model::portrait(f.eval(z), e).map(rgb).unwrap_or_else(ground);
                for e in [0.0, 0.001, 0.37, 0.501, 0.999, 1.0] {
                    let want = at(e);
                    let got = portrait_mix(at(0.0), at(1.0), at(0.0), at(1.0), e, 0.413);
                    for (a, b) in [(want.r, got.r), (want.g, got.g), (want.b, got.b)] {
                        assert!((a - b).abs() < 1e-12, "{f:?} at {z:?}, enhancement={e}");
                    }
                }
            }
        }
    }

    use super::*;
    use crate::model::{I, ZERO};

    /// The window waits for the voice, then the breath, then its seconds, and touches
    /// nothing on screen. Read from this file, because no render can run here.
    #[test]
    fn the_reading_window_waits_in_order_and_clears_nothing() {
        let source = include_str!("draw.rs");
        let head = source.find("pub fn reading_window(").unwrap();
        let body = &source[head..head + source[head..].find("\n}\n").unwrap()];
        let voice = body.find("wait_until(stage, line.end())?;").unwrap();
        let breath = body.find("kit.hold(stage)?;").unwrap();
        let window = body.find("pause(stage, seconds)").unwrap();
        assert!(voice < breath && breath < window);
        assert!(body.trim_end().ends_with("pause(stage, seconds)"));
        for touch in [
            "fade",
            "remove_from_scene",
            "clear_updaters",
            "kit.mark",
            "kit.listen",
            "music_breath",
        ] {
            assert!(!body.contains(touch), "{touch}");
        }
    }

    /// A card that grows brings the whole in before the first part goes, so the band is never
    /// missing. Read from this file, because no render can run here.
    #[test]
    fn a_growing_card_covers_before_it_clears() {
        let source = include_str!("draw.rs");
        let head = source.find("pub fn precisely_over(").unwrap();
        let body = &source[head..head + source[head..].find("\n}\n").unwrap()];
        let comes = body.find("fade_in_all(stage, &[whole], 0.3)?;").unwrap();
        let goes = body.find("fade_out_all(stage, &[said], 0.1)").unwrap();
        assert!(comes < goes);
    }

    /// Every picture the film builds goes through `Picture::build`, and that through `crisp`.
    /// Read from this file; what it does to the pixels is for the pictures to show.
    #[test]
    fn every_colour_picture_is_crisp() {
        let source = include_str!("draw.rs");
        let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
        assert_eq!(code.matches("RowStrips::build(stage").count(), 1);
        assert_eq!(code.matches("blended_rows(stage").count(), 1);
        assert_eq!(code.matches("Picture::build(").count(), 2);
        assert_eq!(code.matches("crisp(stage, picture.rows(), cell_h, style);").count(), 1);
    }

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn scene(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = fmn::scene::RuntimeConfig {
            fps: 10,
            ..fmn::scene::RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut fmn::scene::NullSceneSink).expect("the probe scene runs");
    }

    /// A grid whose cells all differ, so a record in the wrong place shows.
    fn grid(rows: usize, cols: usize, shift: f64) -> Vec<Vec<Srgb>> {
        (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| Srgb {
                        r: 0.1 + 0.8 * (c as f64 + 1.0) / (cols as f64 + 1.0),
                        g: 0.9 - 0.8 * (r as f64 + 1.0) / (rows as f64 + 1.0),
                        b: (0.3 + 0.05 * (r * cols + c) as f64 + shift).fract(),
                    })
                    .collect()
            })
            .collect()
    }

    fn lane(stage: &Stage<'_>, m: Mob, field: &str) -> Vec<f32> {
        stage.get(m).unwrap().buffer.read_column(field).unwrap()
    }

    fn band(stage: &Stage<'_>, m: Mob) -> f64 {
        stage.uniforms(m).unwrap().anti_alias_width
    }

    /// The share of its full opacity a colour lane is showing, if every record still holds its
    /// own colour and all of them are scaled alike. `None` if any colour moved or any alpha
    /// left the common share: what a flattening or a dropped vertex colour would look like.
    fn share_of(full: &[f32], seen: &[f32]) -> Option<f32> {
        if full.len() != seen.len() {
            return None;
        }
        let mut share = None;
        for (a, b) in full.as_chunks::<4>().0.iter().zip(seen.as_chunks::<4>().0) {
            if (0..3).any(|i| (a[i] - b[i]).abs() > 1e-6) {
                return None;
            }
            if a[3] == 0.0 {
                if b[3] != 0.0 {
                    return None;
                }
                continue;
            }
            let here = b[3] / a[3];
            if share.is_some_and(|s: f32| (s - here).abs() > 1e-5) {
                return None;
            }
            share = Some(here);
        }
        share
    }

    const CELLS: (f64, f64) = (0.3, 0.2);

    #[test]
    fn plate_now_writes_exactly_what_the_kit_writes() {
        scene(|stage| {
            let colors = grid(3, 4, 0.0);
            let kit = RowStrips::build(stage, v(-1.0, 1.0), CELLS, &colors)?;
            for &m in &kit.rows {
                stage.set_anti_alias_width(m, 0.5, false);
            }
            let ours = Picture::build(stage, v(-1.0, 1.0), CELLS, &colors, &PLATE_NOW)?;
            assert_eq!(ours.rows().len(), kit.rows.len());
            for (&a, &b) in kit.rows.iter().zip(ours.rows()) {
                for field in ["point", "stroke_rgba", "stroke_width", "fill_rgba"] {
                    assert_eq!(lane(stage, a, field), lane(stage, b, field), "{field}");
                }
                assert_eq!(band(stage, a), band(stage, b));
                assert_eq!(band(stage, b), 0.5);
            }
            Ok(())
        });
    }

    #[test]
    fn tiling_changes_the_band_and_the_width_and_nothing_else() {
        scene(|stage| {
            let colors = grid(3, 4, 0.0);
            let now = Picture::build(stage, v(-1.0, 1.0), CELLS, &colors, &PLATE_NOW)?;
            let tile = Picture::build(stage, v(-1.0, 1.0), CELLS, &colors, &PLATE_TILE)?;
            for (&a, &b) in now.rows().iter().zip(tile.rows()) {
                assert_eq!(lane(stage, a, "stroke_rgba"), lane(stage, b, "stroke_rgba"));
                assert_eq!(lane(stage, a, "point"), lane(stage, b, "point"));
                assert_eq!(band(stage, b), 0.0);
                let want = ((CELLS.1 + ROW_GUARD) / 0.01) as f32;
                assert!(lane(stage, b, "stroke_width").iter().all(|&w| w == want));
            }
            Ok(())
        });
    }

    #[test]
    fn a_blended_row_carries_its_layout_and_recolours_in_place() {
        scene(|stage| {
            let colors = grid(3, 4, 0.0);
            let pic = Picture::build(stage, v(-1.0, 1.0), CELLS, &colors, &PLATE_BLEND)?;
            assert_eq!(band(stage, pic.rows()[0]), 0.0);
            for (&m, row) in pic.rows().iter().zip(&colors) {
                assert_eq!(lane(stage, m, "stroke_rgba"), blended_rgba(row).as_flattened());
            }
            let next = grid(3, 4, 0.2);
            assert!(pic.recolor(stage.arena_mut(), &next));
            for (&m, row) in pic.rows().iter().zip(&next) {
                assert_eq!(lane(stage, m, "stroke_rgba"), blended_rgba(row).as_flattened());
            }
            assert!(!pic.recolor(stage.arena_mut(), &grid(2, 4, 0.0)), "too few rows");
            assert!(!pic.recolor(stage.arena_mut(), &grid(3, 5, 0.0)), "too many columns");
            Ok(())
        });
    }

    /// A fade in and a fade out, with the lanes read at every frame: each record keeps its own
    /// colour, every alpha scales alike, and the clear lead-in stays clear. This reads the
    /// records the renderer is given; `measure_lab.py` asks the same of a real render's pixels.
    #[test]
    fn fades_keep_every_vertex_colour() {
        for style in [PLATE_NOW, PLATE_TILE, PLATE_BLEND] {
            let frames: std::rc::Rc<std::cell::RefCell<Vec<Vec<f32>>>> = std::rc::Rc::default();
            let log = std::rc::Rc::clone(&frames);
            let mut full = Vec::new();
            let mut settled = Vec::new();
            let mut gone = Vec::new();
            scene(|stage| {
                let colors = grid(3, 4, 0.0);
                let pic = Picture::build(stage, v(-1.0, 1.0), CELLS, &colors, &style)?;
                let row = pic.rows()[1];
                full = lane(stage, row, "stroke_rgba");
                let watch = stage.arena_mut().add(VMobject::new());
                let seen = std::rc::Rc::clone(&log);
                stage.add_updater(
                    watch,
                    move |s, _| {
                        seen.borrow_mut()
                            .push(s.get(row).unwrap().buffer.read_column("stroke_rgba").unwrap());
                    },
                    true,
                )?;
                stage.add_to_scene(watch)?;
                fade_in_all(stage, &[pic.group], 1.0)?;
                settled = lane(stage, row, "stroke_rgba");
                assert_eq!(band(stage, row), style.aa_px);
                fade_out_all(stage, &[pic.group], 1.0)?;
                gone = lane(stage, row, "stroke_rgba");
                assert_eq!(band(stage, row), style.aa_px);
                Ok(())
            });
            let end = share_of(&full, &settled);
            assert!(
                end.is_some_and(|s| (s - 1.0).abs() < 1e-5),
                "{style:?}: after the fade in: {end:?}"
            );
            // the fade's remover may leave the records at zero or give them back
            let end = share_of(&full, &gone);
            assert!(
                end.is_some_and(|s| s.abs() < 1e-5 || (s - 1.0).abs() < 1e-5),
                "{style:?}: after the fade out: {end:?}"
            );
            let frames = frames.borrow();
            let shares: Vec<f32> = frames
                .iter()
                .map(|f| share_of(&full, f).unwrap_or_else(|| panic!("{style:?}: a colour moved mid-fade")))
                .collect();
            assert!(
                shares.iter().filter(|s| **s > 0.05 && **s < 0.95).count() >= 4,
                "{style:?}: the probe has to see the fade: {shares:?}"
            );
        }
    }

    /// `share_of` has to fail on what it is there to catch.
    #[test]
    fn control_a_flattened_or_uneven_lane_is_caught() {
        let full: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 1.0], [0.7, 0.2, 0.1, 1.0]].concat();
        assert_eq!(share_of(&full, &full), Some(1.0));
        let half: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.5]].concat();
        assert_eq!(share_of(&full, &half), Some(0.5));
        let flat: Vec<f32> = [[0.4, 0.5, 0.6, 0.0], [0.4, 0.5, 0.6, 1.0], [0.4, 0.5, 0.6, 1.0]].concat();
        assert_eq!(share_of(&full, &flat), None, "flattened colours");
        let uneven: Vec<f32> = [[0.1, 0.2, 0.3, 0.0], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.6]].concat();
        assert_eq!(share_of(&full, &uneven), None, "alphas not scaled alike");
        let leak: Vec<f32> = [[0.1, 0.2, 0.3, 0.1], [0.4, 0.5, 0.6, 0.5], [0.7, 0.2, 0.1, 0.5]].concat();
        assert_eq!(share_of(&full, &leak), None, "the clear lead-in painted");
    }

    /// Choosing a picture for the film is a decision: this changes with it.
    #[test]
    fn the_film_uses_the_native_two_dimensional_raster() {
        assert_eq!(PLATE, PLATE_RASTER);
    }

    #[test]
    fn every_rung_keeps_a_full_picture_under_the_group_limit() {
        let p = Plane::new(v(0.0, 0.0), 6.4, (-2.0, 2.0), (-2.0, 2.0));
        for style in [
            PLATE_NOW,
            PLATE_TILE,
            PLATE_BLEND,
            PLATE_ROWS2,
            PLATE_CELLS2,
            PLATE_CELLS4,
        ] {
            let (_, rows) = picture_cells(&p, &style);
            assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT, "{style:?}: {rows}");
        }
    }

    #[test]
    fn four_samples_are_the_cells_mean_and_one_sample_is_its_centre() {
        let p = Plane::new(v(0.0, 0.0), 6.4, (-2.0, 2.0), (-2.0, 2.0));
        let f = |z: C| Some(z);
        assert_eq!(portrait_colors(f, &p, 8, 8, 1.0), plate_colors(f, &p, 8, 8, 1.0, 1));
        let four = plate_colors(f, &p, 8, 8, 1.0, 2);
        let (dx, dy) = (4.0 / 8.0 / 4.0, 4.0 / 8.0 / 4.0);
        let z = model::cell_input(p.frame.x, p.frame.y, 8, 8, 2, 5);
        let mut mean = [0.0; 3];
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let one = model::portrait(Some(C::new(z.re + sx * dx, z.im + sy * dy)), 1.0).unwrap();
            for i in 0..3 {
                mean[i] += one[i] / 4.0;
            }
        }
        let got = four[2][5];
        assert!((got.r - mean[0]).abs() < 1e-12 && (got.g - mean[1]).abs() < 1e-12 && (got.b - mean[2]).abs() < 1e-12);
    }

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
    fn a_full_raster_has_subpixel_samples_and_bounded_storage() {
        // the widest pictures the chapters paint (05's sin and Γ, 8.4 wide) and the tallest
        // (04's plane and 05's exp, 6.4 high)
        for p in [
            super::super::gallery::trig_plane(),
            super::super::gallery::gamma_plane(),
            super::super::reading_color::plane(),
            super::super::gallery::exp_plane(),
        ] {
            let (cols, rows) = picture_cells(&p, &PLATE);
            assert!(rows <= 2304 && cols <= 3024, "{cols} × {rows}");
            assert!(cols * rows * 4 <= 32 * 1024 * 1024);
            assert!(p.frame.width / cols as f64 * 270.0 <= 1.0);
            assert!(p.frame.height / rows as f64 * 270.0 <= 1.0);
        }
    }

    #[test]
    fn raster_repaints_without_moving_or_flattening_pixels() {
        scene(|stage| {
            let original = grid(3, 4, 0.0);
            let pic = Picture::build(stage, v(-1.0, 1.0), CELLS, &original, &PLATE_RASTER)?;
            assert_eq!(pic.rows().len(), 1);
            let mob = pic.rows()[0];
            let points = lane(stage, mob, "point");
            let before = stage.get(mob).unwrap().image_resource().unwrap().pixels().to_vec();
            assert_eq!(before, picture_resource(&original)?.pixels());
            let next = grid(3, 4, 0.2);
            assert!(pic.recolor(stage.arena_mut(), &next));
            assert_eq!(lane(stage, mob, "point"), points);
            assert_ne!(stage.get(mob).unwrap().image_resource().unwrap().pixels(), before);
            assert_eq!(
                stage.get(mob).unwrap().image_resource().unwrap().pixels(),
                picture_resource(&next)?.pixels()
            );
            assert!(!pic.recolor(stage.arena_mut(), &grid(2, 4, 0.0)));
            fade_in_all(stage, &[pic.group], 1.0)?;
            assert!(lane(stage, mob, "opacity").iter().all(|x| (*x - 1.0).abs() < 1e-5));
            assert_eq!(
                stage.get(mob).unwrap().image_resource().unwrap().pixels(),
                picture_resource(&next)?.pixels()
            );
            fade_out_all(stage, &[pic.group], 1.0)?;
            assert_eq!(
                stage.get(mob).unwrap().image_resource().unwrap().pixels(),
                picture_resource(&next)?.pixels()
            );
            Ok(())
        });
    }

    #[test]
    fn the_ring_drains_from_the_top() {
        let full = ring_points(v(0.0, 0.0), 1.0, 1.0);
        assert!((full[0][1] - 1.0).abs() < 1e-12);
        let half = ring_points(v(0.0, 0.0), 1.0, 0.5);
        let end = half.last().unwrap();
        assert!((end[1] + 1.0).abs() < 1e-9);
    }

    /// The film's plate (`PLATE`, one raster quad) fades through its image's `opacity` lane.
    /// Read on every frame the plate is up (scene updaters with a frame's `dt`), the six records
    /// agree and follow the fades' own smooth alpha up and back down; the lane is given back at
    /// 1 only once the plate is off the scene. Production19 held it at 1 throughout (review241
    /// M1). This reads the records the renderer is given; `plate_fade_probe` reads pixels.
    #[test]
    fn a_raster_plate_fades_through_its_opacity_lane() {
        let log: std::rc::Rc<std::cell::RefCell<Vec<Vec<f32>>>> = std::rc::Rc::default();
        let mut cuts = [0usize; 2];
        let mut took = [0i64; 2];
        let mut after = Vec::new();
        let mut stays = true;
        scene(|stage| {
            let pic = Picture::build(stage, v(-1.0, 1.0), CELLS, &grid(3, 4, 0.0), &PLATE)?;
            let (group, image) = (pic.group, pic.rows()[0]);
            assert_eq!(image_records(stage, group), vec![image], "the plate's one image record");
            let watch = stage.arena_mut().add(VMobject::new());
            let seen = std::rc::Rc::clone(&log);
            stage.add_updater(
                watch,
                move |s, dt| {
                    if dt > 0.0 && s.roots().contains(&group) {
                        seen.borrow_mut()
                            .push(s.get(image).unwrap().buffer.read_column("opacity").unwrap());
                    }
                },
                true,
            )?;
            stage.add_to_scene(watch)?;
            let t0 = stage.scene().time().frames();
            fade_in_all(stage, &[group], 1.0)?;
            cuts[0] = log.borrow().len();
            let t1 = stage.scene().time().frames();
            fade_out_all(stage, &[group], 1.0)?;
            cuts[1] = log.borrow().len();
            took = [t1 - t0, stage.scene().time().frames() - t1];
            after = lane(stage, image, "opacity");
            stays = stage.scene().mobjects().contains(&group);
            Ok(())
        });
        assert_eq!(took, [10, 10], "a 1 s fade is 10 frames at 10 fps, in or out");
        assert_eq!(
            cuts,
            [10, 20],
            "one reading per frame the plate is up, none after it left"
        );
        for (k, lanes) in log.borrow().iter().enumerate() {
            assert_eq!(lanes.len(), 6, "frame {k}");
            assert!(
                lanes.iter().all(|&o| o == lanes[0]),
                "frame {k}: records disagree: {lanes:?}"
            );
            let a = fmn::core::rate::smooth((k % 10 + 1) as f64 / 10.0);
            let want = if k < 10 { a } else { 1.0 - a };
            assert!(
                (f64::from(lanes[0]) - want).abs() < 1e-6,
                "frame {k}: {} against {want}",
                lanes[0]
            );
        }
        assert!(after.iter().all(|&o| o == 1.0), "given back at 1: {after:?}");
        assert!(!stays, "the fade out took the plate off");
    }

    /// At the film's 60 fps a plate's fade takes exactly the frames a vector's fade takes, in
    /// and out, for the run times the film gives plate fades: the ramp rides in the same play.
    #[test]
    fn a_plate_fade_takes_the_frames_a_vector_fade_takes() {
        struct At60<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for At60<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = fmn::scene::RuntimeConfig {
            fps: 60,
            ..fmn::scene::RuntimeConfig::default()
        };
        let mut took = Vec::new();
        let mut run = At60(|stage: &mut Stage<'_>| {
            let now = |s: &Stage<'_>| s.scene().time().frames();
            for rt in [0.1, 0.25, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0, 1.2] {
                let pic = Picture::build(stage, v(-1.0, 1.0), CELLS, &grid(3, 4, 0.0), &PLATE)?;
                let pts = vec![v(1.0, 1.0), v(2.0, 1.0), v(2.0, 0.0), v(1.0, 0.0)];
                let shape = stage
                    .arena_mut()
                    .add(Polygon::new(pts).style(flat(ground(), 1.0)).build());
                let t0 = now(stage);
                fade_in_all(stage, &[pic.group], rt)?;
                let t1 = now(stage);
                fade_in_all(stage, &[shape], rt)?;
                let t2 = now(stage);
                fade_out_all(stage, &[pic.group], rt)?;
                let t3 = now(stage);
                fade_out_all(stage, &[shape], rt)?;
                took.push((rt, [t1 - t0, t2 - t1, t3 - t2, now(stage) - t3]));
            }
            Ok(())
        });
        fmn::run_scene(&mut run, config, 7, &mut fmn::scene::NullSceneSink).expect("the probe scene runs");
        assert_eq!(took.len(), 9);
        for (rt, [plate_in, shape_in, plate_out, shape_out]) in took {
            assert!(plate_in > 0, "{rt} s");
            assert_eq!([plate_in, plate_out, shape_out], [shape_in; 3], "{rt} s");
        }
    }

    /// One file's reviewed engine fades: the file, its engine fade lines as written, and
    /// `(text, count)` pins on the bindings of what they fade.
    type ReviewedFades = (&'static str, &'static [&'static str], &'static [(&'static str, usize)]);

    /// Engine fades outside `fade_in_all`/`fade_out_all` that were read and found vector-only
    /// (cc2-plate-fade-correction260, after CC3's integration finding): each file's engine fade
    /// lines exactly as written, then the one vector constructor of every mobject they fade. A
    /// new or changed engine fade, a second binding of a target, or any plate word in such a file
    /// fails `fade_gate`: it needs a fresh review, not a wider list.
    const VECTOR_ONLY_FADES: &[ReviewedFades] = &[
        (
            "loop_sums",
            &[
                "let c_aside = fade_out(stage.arena_mut(), centre_name, ORIGIN, 1.0)?;",
                "let d_aside = fade_out(stage.arena_mut(), diagonal_name, ORIGIN, 1.0)?;",
            ],
            &[
                ("let centre_name", 1),
                ("let centre_name = kit.tex(stage, \"c\", ", 1),
                ("let diagonal_name", 1),
                ("let diagonal_name = kit.tex(stage, \"D_n\", ", 1),
            ],
        ),
        (
            "one_derivative",
            &[
                "let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;",
                "let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;",
            ],
            &[
                // `hand_over`: its definition and its one call, on three text mobjects
                ("hand_over(", 2),
                ("hand_over(stage, &[scale_note], &zoomed, 0.3)?;", 1),
                ("let scale_note", 1),
                ("let scale_note = kit.text(", 1),
                ("let zoomed", 1),
                (
                    "let zoomed = [\n            kit.text_left(stage, ZOOMED, 18.0, Palette::dim(), v(2.55, -2.35))?,\n            kit.text_left(stage, ZOOMED_WHY, 18.0, Palette::dim(), v(2.55, -2.62))?,\n        ];",
                    1,
                ),
            ],
        ),
    ];

    /// Words only plate code uses: none may appear outside comments in a file whose engine fades
    /// are listed as vector-only.
    const PLATE_WORDS: &[&str] = &[
        "Painter",
        "Picture",
        "portrait",
        "portrait_styled",
        "PlateStyle",
        "PLATE",
        "PLATE_RASTER",
        "RowStrips",
        "ImageMobject",
        "ImageResource",
        "image_resource",
        "set_image_resource",
        "blended_rows",
        "plate_colors",
    ];

    /// The fade gate for one registered chapter file's non-test code; `Err` says what is wrong.
    /// draw.rs holds the two ramped engine fades; a `VECTOR_ONLY_FADES` file holds exactly its
    /// listed ones on its listed text targets; every other file holds none.
    fn fade_gate(name: &str, code: &str) -> Result<(), String> {
        let mut fades: Vec<&str> = code
            .lines()
            .map(str::trim)
            .filter(|l| ["fade_in(", "fade_out(", "fade_all("].iter().any(|p| l.contains(p)))
            .collect();
        fades.sort_unstable();
        let listed = VECTOR_ONLY_FADES.iter().find(|(file, _, _)| *file == name);
        let mut want: Vec<&str> = match (name, listed) {
            ("draw", _) => vec![
                "let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;",
                "let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;",
            ],
            (_, Some((_, lines, _))) => lines.to_vec(),
            (_, None) => Vec::new(),
        };
        want.sort_unstable();
        if fades != want {
            return Err(format!("{name}: engine fades {fades:?}, reviewed {want:?}"));
        }
        if name == "draw" {
            for ramp in [
                "plate_fade(m, images, true).rt(rt)",
                "plate_fade(m, images, false).rt(rt)",
            ] {
                if code.matches(ramp).count() != 1 {
                    return Err(format!("draw: `{ramp}` is not there exactly once"));
                }
            }
        }
        if let Some((_, _, targets)) = listed {
            for &(text, count) in *targets {
                let seen = code.matches(text).count();
                if seen != count {
                    return Err(format!("{name}: {seen} of {text:?}, reviewed {count}"));
                }
            }
            let words: std::collections::HashSet<&str> = code
                .lines()
                .map(|l| l.split("//").next().unwrap_or(""))
                .flat_map(|l| l.split(|c: char| !(c.is_alphanumeric() || c == '_')))
                .collect();
            if let Some(w) = PLATE_WORDS.iter().find(|w| words.contains(**w)) {
                return Err(format!(
                    "{name}: `{w}` in a file whose engine fades are listed as vector-only"
                ));
            }
        }
        Ok(())
    }

    /// Every engine fade in the film is one of draw.rs's two, each with the plate's ramp beside
    /// it, or one reviewed as vector-only in `VECTOR_ONLY_FADES`, so no plate fades without the
    /// ramp. Read from the registered chapter files (`mod.rs`); the unregistered drafts beside
    /// them are not compiled.
    #[test]
    fn every_engine_fade_in_the_film_carries_the_plate_ramp() {
        let registry = include_str!("mod.rs");
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/chapters");
        let mut read = Vec::new();
        for line in registry.lines().map(str::trim) {
            let Some(name) = line
                .strip_prefix("pub mod ")
                .or_else(|| line.strip_prefix("mod "))
                .and_then(|rest| rest.strip_suffix(';'))
            else {
                continue;
            };
            let source = std::fs::read_to_string(format!("{dir}/{name}.rs")).unwrap_or_else(|e| panic!("{name}: {e}"));
            let code = source.split("\n#[cfg(test)]\nmod tests {").next().unwrap();
            fade_gate(name, code).unwrap_or_else(|e| panic!("{e}"));
            read.push(name.to_owned());
        }
        assert_eq!(
            read.len(),
            17,
            "the registered chapter files, draw and thumbnail included: {read:?}"
        );
        for (file, _, _) in VECTOR_ONLY_FADES {
            assert!(read.iter().any(|n| n == file), "{file} is listed but not registered");
        }
    }

    /// `fade_gate` has to refuse what it is there to refuse, on the real files.
    #[test]
    fn control_the_fade_gate_refuses_unreviewed_fades() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/chapters");
        let real = |name: &str| {
            let source = std::fs::read_to_string(format!("{dir}/{name}.rs")).unwrap();
            source.split("\n#[cfg(test)]\nmod tests {").next().unwrap().to_owned()
        };
        let bypass = "        let gone = fade_out(stage.arena_mut(), painter.group, ORIGIN, 1.0)?;\n";
        let (loop_sums, one_derivative, draw) = (real("loop_sums"), real("one_derivative"), real("draw"));
        assert!(fade_gate("loop_sums", &loop_sums).is_ok());
        assert!(fade_gate("one_derivative", &one_derivative).is_ok());
        assert!(fade_gate("draw", &draw).is_ok());
        assert!(fade_gate("residues", bypass).is_err(), "a fade in an unlisted file");
        assert!(
            fade_gate("loop_sums", &format!("{loop_sums}{bypass}")).is_err(),
            "an extra fade in a listed file"
        );
        assert!(
            fade_gate(
                "one_derivative",
                &format!("{one_derivative}\nlet p = Painter::build(stage, None, plane, 1.0)?;\n")
            )
            .is_err(),
            "a plate constructor in a listed file"
        );
        assert!(
            fade_gate(
                "one_derivative",
                &one_derivative.replacen("let scale_note = kit.text(", "let scale_note = (", 1)
            )
            .is_err(),
            "a target no longer built as text"
        );
        assert!(
            fade_gate(
                "loop_sums",
                &loop_sums.replacen(
                    "let centre_name",
                    "let centre_name = painter.group;\n    let centre_name",
                    1
                )
            )
            .is_err(),
            "a second binding of a target"
        );
        assert!(
            fade_gate(
                "draw",
                &draw.replacen("plate_fade(m, images, false).rt(rt)", "plate_fade(m, images, false)", 1)
            )
            .is_err(),
            "draw without its falling ramp"
        );
    }
}

/// The rendered regression for the plate fade (cc2-plate-fade-probe246.rs, installed beside this
/// file): Y4M frames through the film's own camera route, run on purpose with `--ignored`.
#[cfg(test)]
#[path = "plate_fade_probe.rs"]
mod plate_fade_probe;
