//! The video's look, shared by its chapters (storyboard §3, colour roles):
//! the plot of the curves x^n on [0, 1] and their limit, the fixed point
//! and the moving one, and the timing helpers that land a picture on the
//! phrase it illustrates.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, small groups, and no long strokes under
//! moving things (each curve is short pieces).

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Facts, Frame2, Kit, Palette, Script, Timed, hex, v};
use fmn::prelude::*;

use crate::model::{Ratio, curve, power};

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// A fact by key, or a scene error naming it.
pub fn fact(f: &Facts, key: &str) -> fmn::Result<String> {
    f.get(key)
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("no fact {key}")))
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the phrase it illustrates. A
/// phrase's moment is its sentence's start, as the narrator measured it,
/// plus its share of that sentence's characters (the voice reads at a
/// nearly even rate inside a sentence; the pauses fall between them); the
/// narrated draft's cue sheets check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    /// Each sentence's first character and its start, seconds into the line.
    sentences: Vec<(usize, f64)>,
}

/// The character each sentence of `text` starts at: after a `.`, `?` or
/// `!` and its space, as the narrator splits a line.
pub(super) fn sentence_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![0];
    for (i, pair) in chars.windows(2).enumerate() {
        if matches!(pair[0], '.' | '?' | '!') && pair[1] == ' ' && i + 2 < chars.len() {
            out.push(i + 2);
        }
    }
    out
}

/// Line `id`'s sentences, each with its start in seconds into the line: the
/// narrator's `measured` onsets, one per sentence, or with no narrator (an
/// empty list) each sentence's share of `len` by characters. A narrator
/// that splits the line differently is an error, never a guess.
fn timed_sentences(id: &str, text: &str, len: f64, measured: &[f64]) -> fmn::Result<Vec<(usize, f64)>> {
    let offsets = sentence_offsets(text);
    if measured.is_empty() {
        let all = text.chars().count().max(1) as f64;
        return Ok(offsets.into_iter().map(|at| (at, len * at as f64 / all)).collect());
    }
    if measured.len() != offsets.len() {
        return Err(fail(format!(
            "line {id} has {} sentences, but the narrator timed {}",
            offsets.len(),
            measured.len()
        )));
    }
    Ok(offsets.into_iter().zip(measured.iter().copied()).collect())
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
    let sentences = timed_sentences(id, &text, len, &kit.sentence_starts(id))?;
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
        sentences,
    })
}

impl Spoken {
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

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
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

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown; the answer stays hidden until it ends).
pub const THINK_S: f64 = 4.0;
pub(super) const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause with think time: let the question finish, mark the
/// moment for the score, draw a ring of radius `r` around `at` that drains
/// clockwise over [`THINK_S`], with "pause for more time" under it; at zero
/// the ring and the note go.
pub fn think_pause(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let note = kit.text(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0], at[1] - r - 0.3))?;
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + r * a.cos(), at[1] + r * a.sin())
            })
            .collect()
    };
    let left = stage.add_value_tracker(1.0);
    let ring = stage
        .arena_mut()
        .add(polyline(arc(1.0)).style(line_style(Palette::hero(), 4.0)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(left).unwrap_or(0.0);
            if last == Some(f) {
                return;
            }
            let _ = s.set_points(ring, polyline(arc(f)).build().points());
            last = Some(f);
        },
        true,
    )?;
    fade_in_all(stage, &[anchor, ring, note], 0.3)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.25)
}

// ------------------------------------------------------------- colour roles

/// The curve being looked at, x^n for the current n.
pub fn curve_color() -> Srgb {
    Palette::blue()
}

/// The curves already drawn, kept behind the current one.
pub fn ghost() -> Srgb {
    hex("#2C5566")
}

/// The limit f: flat at zero, a single point at height one.
pub fn limit_color() -> Srgb {
    Palette::gold()
}

/// The fixed point x = 0.9 and its heights.
pub fn fixed() -> Srgb {
    Palette::teal()
}

/// The moving point x_n = 2^(−1/n) and the half-height line it sits on.
pub fn moving() -> Srgb {
    hex("#C59BF2")
}

/// The gap that survives at the moving point.
pub fn gap_color() -> Srgb {
    Palette::red()
}

/// A chord's slope (chapter 03): the chord on the curve, and the graph of
/// its slope against the increment h.
pub fn slope_color() -> Srgb {
    Palette::green()
}

/// The step that fails (chapter 03): h set to zero, where the slope is 0/0.
/// Red, as the gap and the jump.
pub fn fails() -> Srgb {
    Palette::red()
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

/// Bring detached mobjects in one after another (a lagged reveal), the
/// whole taking `rt` seconds.
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

/// A move of `mob` to `to` over `rt` seconds, to prepare beside others.
pub fn move_to(mob: Mob, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

// -------------------------------------------------------------------- plot

/// Segments per piece of a drawn curve: long strokes under moving things
/// are cut into short pieces (kit engine rules, spike S-2).
const PIECE: usize = 20;

/// Segments of a curve drawn across a zoom window ([`Plot::curve_window`]).
pub const WINDOW_SEGMENTS: usize = 120;

/// A polyline through `pts` cut into pieces of [`PIECE`] segments sharing
/// their ends, as one group (chapter 10 draws the square root with it).
pub(super) fn pieces(pts: &[Vec3], color: Srgb, width: f64) -> VMobject {
    v_group(
        pts.windows(PIECE + 1)
            .step_by(PIECE)
            .map(|w| polyline(w.to_vec()).style(line_style(color, width)).build()),
    )
}

/// The bright curve's stroke width, the dim copies'.
pub const BRIGHT_W: f64 = 5.0;
pub const GHOST_W: f64 = 3.0;

/// The label of the n-th curve: y = x, y = x², …, at the plot's top left.
pub fn curve_label(kit: &Kit, stage: &mut Stage<'_>, n: u32) -> fmn::Result<Mob> {
    let src = if n == 1 {
        "y = x".to_owned()
    } else {
        format!("y = x^{{{n}}}")
    };
    kit.tex_left(stage, &src, 34.0, curve_color(), &[], PLOT.p(0.04, 0.92))
}

/// The bright curve and its label, and the dim copies it has left behind.
pub struct Bright {
    pub curve: Mob,
    pub label: Mob,
    pub n: u32,
    pub copies: Vec<Mob>,
}

impl Bright {
    pub fn new(curve: Mob, label: Mob, n: u32) -> Self {
        Self {
            curve,
            label,
            n,
            copies: Vec::new(),
        }
    }
}

/// Morph the bright curve into x^to over `rt` seconds, leaving a dim copy
/// of the old one behind and swapping the label, with `with` (prepared
/// animations) playing alongside; the new curve ends on top.
pub fn step(
    kit: &Kit,
    stage: &mut Stage<'_>,
    b: &mut Bright,
    to: u32,
    rt: f64,
    with: Vec<Box<dyn Animation>>,
) -> fmn::Result<()> {
    let copy = PLOT.curve(stage, b.n, ghost(), GHOST_W)?;
    stage.add_to_scene(copy)?;
    b.copies.push(copy);
    let next = PLOT.curve(stage, to, curve_color(), BRIGHT_W)?;
    let next_label = curve_label(kit, stage, to)?;
    let morph = fmn::animation::replacement_transform(b.curve, next).rt(rt);
    let mut anims = with;
    anims.push(stage.prepare(morph)?);
    let out = fade_out(stage.arena_mut(), b.label, ORIGIN, 1.0)?;
    anims.push(stage.prepare(out.rt(rt * 0.5))?);
    let inn = fade_in(stage.arena_mut(), next_label, ORIGIN, 1.0)?;
    anims.push(stage.prepare(inn.rt(rt))?);
    stage.play_prepared(anims)?;
    stage.scene_mut().bring_to_front(&[next])?;
    b.curve = next;
    b.label = next_label;
    b.n = to;
    Ok(())
}

/// The plot of [0, 1] × [0, 1] the chapters draw on: its window, its axes
/// and the shapes of the curves and the limit in scene coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Plot {
    pub frame: Frame2,
}

/// Where the plot sits (scene units; the frame is 14.2 by 8): left of
/// centre, its foot clear of a three-line lower-third.
pub const PLOT: Plot = Plot {
    frame: Frame2 {
        center: [-3.4, 0.6, 0.0],
        width: 4.8,
        height: 4.8,
        x: (0.0, 1.0),
        y: (0.0, 1.0),
    },
};

/// The zoom window (chapter 02) is exactly as wide as the plot, so showing
/// [1 − 1/m, 1] across it stretches that stretch of x exactly m times
/// sideways. Its heights are drawn at this share of the plot's, and its
/// label says so.
pub const LENS_HEIGHTS: Ratio = Ratio { num: 1, den: 3 };
/// The zoom window's panel: the plot's width, [`LENS_HEIGHTS`] of its height.
pub const LENS_W: f64 = PLOT.frame.width;
pub const LENS_H: f64 = PLOT.frame.height * LENS_HEIGHTS.num as f64 / LENS_HEIGHTS.den as f64;

/// How far the axes run past 1, and how far under the x axis its numbers sit.
const AXIS_PAST: f64 = 0.3;
pub(super) const NUMBER_DROP: f64 = 0.32;

/// The radius of the limit's two end marks (the hollow one at (1, 0) and
/// the filled one at (1, 1)).
pub const END_R: f64 = 0.065;

impl Plot {
    /// Data → scene.
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        self.frame.p(x, y)
    }

    /// The axes, their ticks at 1 and their labels, detached: x along the
    /// foot, y up the left side, a little past 1 each.
    pub fn axes(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
        let (o, x1, y1) = (self.p(0.0, 0.0), self.p(1.0, 0.0), self.p(0.0, 1.0));
        let dim = Palette::dim();
        let mut out = Vec::new();
        for (a, b) in [
            (v(o[0] - 0.2, o[1]), v(x1[0] + AXIS_PAST, o[1])),
            (v(o[0], o[1] - 0.2), v(o[0], y1[1] + AXIS_PAST)),
            (v(x1[0], o[1] - 0.09), v(x1[0], o[1] + 0.09)),
            (v(o[0] - 0.09, y1[1]), v(o[0] + 0.09, y1[1])),
        ] {
            let l = Line::new(a, b).style(line_style(dim, 2.5)).build()?;
            out.push(stage.arena_mut().add(l));
        }
        out.push(kit.text(stage, "0", 24.0, dim, v(o[0] - 0.22, o[1] - NUMBER_DROP))?);
        out.push(kit.text(stage, "1", 24.0, dim, v(x1[0], o[1] - NUMBER_DROP))?);
        out.push(kit.text(stage, "1", 24.0, dim, v(o[0] - 0.3, y1[1]))?);
        out.push(kit.tex(stage, "x", 30.0, dim, &[], v(x1[0] + AXIS_PAST + 0.22, o[1]))?);
        out.push(kit.tex(stage, "y", 30.0, dim, &[], v(o[0] - 0.3, y1[1] + AXIS_PAST))?);
        Ok(out)
    }

    /// The n-th curve as short pieces of one group, every piece with the
    /// same number of points, so one curve morphs into another point by
    /// point (each point straight down: the samples share their x).
    pub fn curve_shape(&self, n: u32, color: Srgb, width: f64) -> fmn::Result<VMobject> {
        let pts: Vec<Vec3> = curve(n).map_err(fail)?.into_iter().map(|(x, y)| self.p(x, y)).collect();
        Ok(pieces(&pts, color, width))
    }

    /// The n-th curve across this plot's own x range (a zoom window), evenly
    /// sampled in [`WINDOW_SEGMENTS`] segments, detached: every window's curve
    /// has the same pieces, so one window's curve morphs into another's.
    pub fn curve_window(&self, stage: &mut Stage<'_>, n: u32, color: Srgb, width: f64) -> fmn::Result<Mob> {
        let (lo, hi) = self.frame.x;
        let pts = (0..=WINDOW_SEGMENTS)
            .map(|k| {
                // the last sample is the window's right end exactly (x = 1
                // must not round past the domain)
                let x = if k == WINDOW_SEGMENTS {
                    hi
                } else {
                    lo + (hi - lo) * k as f64 / WINDOW_SEGMENTS as f64
                };
                Ok(self.p(x, power(x, n).map_err(fail)?))
            })
            .collect::<fmn::Result<Vec<Vec3>>>()?;
        Ok(stage.arena_mut().add(pieces(&pts, color, width)))
    }

    /// The n-th curve, detached.
    pub fn curve(&self, stage: &mut Stage<'_>, n: u32, color: Srgb, width: f64) -> fmn::Result<Mob> {
        let shape = self.curve_shape(n, color, width)?;
        Ok(stage.arena_mut().add(shape))
    }

    /// The limit, detached, as three mobjects: the flat stretch along y = 0
    /// from the plot's left edge (x = 0, or a zoom window's left end) up to
    /// the hollow mark at x = 1 (it stops at the mark's edge: f(1) is not 0),
    /// the hollow mark, and the filled point (1, 1).
    pub fn limit(&self, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 3]> {
        let c = limit_color();
        let (o, x1, top) = (self.p(self.frame.x.0, 0.0), self.p(1.0, 0.0), self.p(1.0, 1.0));
        let flat_part = Line::new(o, v(x1[0] - END_R, x1[1]))
            .style(line_style(c, 7.0))
            .build()?;
        let hollow = Circle::new()
            .radius(END_R)
            .arc_center(x1)
            .style(Style::default().fill(hex(BACKGROUND), 1.0).stroke(c, 3.5, 1.0))
            .build();
        let point = Dot::new().point(top).radius(END_R).style(flat(c, 1.0)).build();
        Ok([
            stage.arena_mut().add(flat_part),
            stage.arena_mut().add(hollow),
            stage.arena_mut().add(point),
        ])
    }

    /// A dot on the plot at data (x, y), detached.
    pub fn dot(&self, stage: &mut Stage<'_>, x: f64, y: f64, color: Srgb) -> Mob {
        let d = Dot::new()
            .point(self.p(x, y))
            .radius(0.07)
            .style(flat(color, 1.0))
            .build();
        stage.arena_mut().add(d)
    }

    /// A dashed line between data points, detached.
    pub fn dashed(&self, stage: &mut Stage<'_>, a: (f64, f64), b: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
        let l = DashedLine::new(self.p(a.0, a.1), self.p(b.0, b.1))
            .dash_length(0.1)
            .style(line_style(color, 2.5))
            .build()?;
        Ok(stage.arena_mut().add(l))
    }

    /// A solid line between data points, detached.
    pub fn segment(
        &self,
        stage: &mut Stage<'_>,
        a: (f64, f64),
        b: (f64, f64),
        color: Srgb,
        w: f64,
    ) -> fmn::Result<Mob> {
        let l = Line::new(self.p(a.0, a.1), self.p(b.0, b.1))
            .style(line_style(color, w))
            .build()?;
        Ok(stage.arena_mut().add(l))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A phrase that opens a sentence lands on the narrator's measured
    /// onset; inside a sentence it keeps its share of that sentence.
    #[test]
    fn a_phrase_lands_with_its_measured_sentence() {
        let text = "One two. Three four.";
        let sentences = timed_sentences("x", text, 6.0, &[0.0, 4.0]).unwrap();
        assert_eq!(sentences.iter().map(|&(at, _)| at).collect::<Vec<_>>(), [0, 9]);
        let line = Spoken {
            id: "x".to_owned(),
            start: 10.0,
            len: 6.0,
            text: text.to_owned(),
            sentences,
        };
        let close = |phrase: &str, t: f64| (line.at(phrase).unwrap() - t).abs() < 1e-9;
        assert!(close("Three", 14.0));
        assert!(close("two", 10.0 + 4.0 * 4.0 / 9.0));
        assert!(close("four", 14.0 + 2.0 * 6.0 / 11.0));
        assert!(line.at("five").is_err(), "a phrase the line does not say");
        assert!(
            timed_sentences("x", text, 6.0, &[0.0]).is_err(),
            "a narrator's other split"
        );
    }

    /// Every line splits into the sentences the kit's narrator times, so a
    /// measured line never meets the error above.
    #[test]
    fn the_script_splits_as_the_narrator_does() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let narrator = explainer_kit::Narrator::estimate(&script);
        for (id, text) in script.lines() {
            assert_eq!(sentence_offsets(text).len(), narrator.sentence_starts(id).len(), "{id}");
        }
    }

    /// The plot sits inside the frame, clear of a three-line lower-third,
    /// and leaves the right half for the panels.
    #[test]
    fn the_plot_fits_its_place() {
        let f = PLOT.frame;
        assert!(f.left() - 0.6 > -7.1 && f.right() + AXIS_PAST + 0.4 < 0.0);
        assert!(f.top() + AXIS_PAST + 0.3 < 4.0);
        // a 3-line lower-third's top is at -4 + 0.2 + 0.3 + 3 × 0.36 = -2.42
        assert!(f.bottom() - NUMBER_DROP - 0.2 > -2.42);
        assert_eq!(PLOT.p(0.0, 0.0), [f.left(), f.bottom(), 0.0]);
        assert_eq!(PLOT.p(1.0, 1.0), [f.right(), f.top(), 0.0]);
    }

    /// Each curve is the same number of pieces with the same number of
    /// points, and the pieces join end to end.
    #[test]
    fn every_curve_is_the_same_pieces() {
        let pieces = crate::model::SEGMENTS / PIECE;
        assert_eq!(pieces * PIECE, crate::model::SEGMENTS, "the pieces cover every segment");
        let pts: Vec<Vec3> = curve(7).unwrap().into_iter().map(|(x, y)| PLOT.p(x, y)).collect();
        let chunks: Vec<&[Vec3]> = pts.windows(PIECE + 1).step_by(PIECE).collect();
        assert_eq!(chunks.len(), pieces);
        for w in chunks.windows(2) {
            assert_eq!(w[0][PIECE], w[1][0]);
        }
        assert_eq!(*chunks[pieces - 1].last().unwrap(), PLOT.p(1.0, 1.0));
    }
}
