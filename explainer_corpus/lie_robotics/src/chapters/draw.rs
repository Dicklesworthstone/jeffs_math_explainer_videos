//! Video G's look, shared by every chapter (storyboard §3 "Color roles", "The
//! quiz device", "Three-dimensional pictures"): line timing, the fades,
//! the fixed projection and the one redrawn world every 3D picture is made
//! of (the book, the rings, the arm), the arm's rig and the quiz card with
//! its countdown.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, small groups. A 3D world is a pool of
//! polygons that one updater re-points from trackers each frame (faces
//! culled by their normals, parts sorted back to front), and only when a
//! tracker has moved.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::rc::Rc;
use std::sync::OnceLock;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Facts, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::sound;
use crate::model::{self, Arm, DEG, N, Pose, Q, Screw, V3, add, cross, dot, norm, scale, sub, unit};
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------------- facts

/// Every fact of the video, computed once (the scenes read their numbers
/// from here, never from a literal).
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
    fact(key)?
        .parse()
        .map_err(|_| fail(format!("fact {key} is not a number: {}", fact(key).unwrap_or("?"))))
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

/// The winter Lie began his theory of continuous groups, as years the facts
/// give: `1873–74` (R2 B-09; 05_7's name card and 08_6's lower third).
pub fn lie_winter() -> fmn::Result<String> {
    let begun = fact_f64("G-L.lie.year")? as u32;
    Ok(format!("{begun}–{:02}", (begun + 1) % 100))
}

/// A whole number from zero to twenty in words (`3` is `three`), for on-screen
/// sentences whose numbers are read from facts and said as the script says
/// them.
pub fn number_word(n: u32) -> fmn::Result<&'static str> {
    const WORDS: [&str; 21] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
        "twenty",
    ];
    WORDS
        .get(n as usize)
        .copied()
        .ok_or_else(|| fail(format!("no word for {n}: the sentences spell zero to twenty")))
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

/// The fill behind cards.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// The background, for cut lines over fills and for shading towards.
pub fn ground() -> Srgb {
    hex(explainer_kit::BACKGROUND)
}

/// A book's pages.
pub fn paper() -> Srgb {
    hex("#D9D3C2")
}

/// What a part that is not the subject is drawn in (a greyed copy).
pub fn greyed() -> Srgb {
    hex("#6C7686")
}

/// A table top, a floor: a surface to stand things on.
pub fn table_top() -> Srgb {
    hex("#222B3B")
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

/// Take things out together, `rt` seconds: fade `mobs` out and tween every
/// tracker in `to_zero` (a world's `shown`, a part's) to 0.
pub fn leave(stage: &mut Stage<'_>, mobs: &[Mob], to_zero: &[Mob], rt: f64) -> fmn::Result<()> {
    hand_over(stage, mobs, to_zero, &[], rt)
}

/// One picture gives way to the next in one beat, `rt` seconds: fade `mobs`
/// out, tween the trackers in `to_zero` to 0 and those in `to_one` (a
/// world's `shown`) to 1, all together. A fade out and then a fade in leaves
/// the frame empty between them (the background, for half a second); this
/// crossfades, so the two always sum to a whole picture.
pub fn hand_over(stage: &mut Stage<'_>, mobs: &[Mob], to_zero: &[Mob], to_one: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &t in to_zero {
        anims.push(stage.prepare(tween(t, 0.0, rt)?)?);
    }
    for &t in to_one {
        anims.push(stage.prepare(tween(t, 1.0, rt)?)?);
    }
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// A tag for a band: `words` on a small back standing on the band's upper
/// left corner (`band` a lower third, built, not yet shown), so a note under
/// the band's own label reads as the band's: a tab, not a label of its own.
pub fn tab_on(kit: &Kit, stage: &mut Stage<'_>, band: Mob, words: &str, size: f64) -> fmn::Result<Mob> {
    let chip = tag(kit, stage, words, v(0.0, 0.0), size)?;
    let (w, h) = (stage.get_width(chip), stage.get_height(chip));
    let (left, top) = (stage.get_left(band)[0], stage.get_top(band)[1]);
    stage.move_to(chip, v(left + w / 2.0, top + h / 2.0 + TAB_GAP), ORIGIN);
    Ok(chip)
}

/// How far a tab stands above its band (scene units).
pub const TAB_GAP: f64 = 0.02;

/// Animate a tracker to `value` at a steady rate over `rt` seconds (a clock,
/// a ring draining).
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

// -------------------------------------------------------- the four words

/// The four words, one card (storyboard, "Four words, one card"): a turn is
/// how far, turning only; a spin is how fast, turning only; a screw is how
/// far, turning and sliding; a twist is how fast, turning and sliding. Shown
/// at 07_3 and brought back small at 10_2 and 13_3 (`scale` below 1). All the
/// parts, detached, back first.
pub fn four_words(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, scale: f64) -> fmn::Result<Vec<Mob>> {
    let (w, h) = (5.8 * scale, 3.7 * scale);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.16 * scale)
        .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
        .build()?
        .moved_to(at);
    let mut out = vec![stage.arena_mut().add(back)];
    let word = (40.0 * scale).max(22.0);
    let note = (22.0 * scale).max(15.0);
    let (dx, dy) = (1.5 * scale, 0.95 * scale);
    // each cell: the word, and under it what it measures
    let cells = [
        ("turn", "how far,\nturning only", palette::turn(), -1.0, 1.0),
        ("spin", "how fast,\nturning only", palette::flat(), 1.0, 1.0),
        ("screw", "how far,\nturning and sliding", palette::screw(), -1.0, -1.0),
        ("twist", "how fast,\nturning and sliding", palette::flat(), 1.0, -1.0),
    ];
    for (words, what, color, cx, cy) in cells {
        let (x, y) = (at[0] + cx * dx, at[1] + cy * dy);
        out.push(kit.text(stage, words, word, color, v(x, y + 0.3 * scale))?);
        out.push(kit.text(stage, what, note, Palette::dim(), v(x, y - 0.35 * scale))?);
    }
    Ok(out)
}

// ---------------------------------------------------------------- the badge

/// Where the badge sits in a corner whenever the one move is being made
/// (storyboard, "The badge"): chapters 05, 06, 10, 13 and 14.
/// (The badge is 1.24 wide and 0.6 tall: at (6.3, 3.2) it ends 0.04 inside the
/// safe area; at (6.35, 3.25) it touched the safe area's edge.)
pub const BADGE_CORNER: Vec3 = [6.3, 3.2, 0.0];

/// The badge: a small circle (the curved space) with an orange line touching
/// it (the flat piece), and an arrow that wraps the answer back from the line
/// onto the circle. Detached; the arrow is separate so that it can be lit when
/// the move is made ("the exponential" at 05_3).
#[derive(Clone, Copy, Debug)]
pub struct Badge {
    /// The circle and the line.
    pub body: Mob,
    pub circle: Mob,
    pub line: Mob,
    /// The wrap-back arrow, from the line's far end down onto the circle.
    pub arrow: Mob,
    pub at: Vec3,
    pub scale: f64,
}

impl Badge {
    /// The badge centred at `at`, `scale` times the corner size.
    pub fn build(stage: &mut Stage<'_>, at: Vec3, scale: f64) -> fmn::Result<Self> {
        let r = 0.3 * scale;
        let circle = stage.arena_mut().add(
            Circle::new()
                .radius(r)
                .arc_center(at)
                .style(line_style(palette::turn(), 4.0 * scale.sqrt()))
                .build(),
        );
        let top = at[1] + r;
        let line = stage.arena_mut().add(
            polyline(vec![v(at[0] - 0.62 * scale, top), v(at[0] + 0.62 * scale, top)])
                .style(line_style(palette::flat(), 4.0 * scale.sqrt()))
                .build(),
        );
        // the arrow: from the line's right end, round the circle's right
        // side and onto it, with a head
        let rr = r + 0.13 * scale;
        let (a0, a1) = (62.0_f64.to_radians(), -22.0_f64.to_radians());
        let pts: Vec<Vec3> = (0..=10)
            .map(|k| {
                let a = a0 + (a1 - a0) * k as f64 / 10.0;
                v(at[0] + rr * a.cos(), at[1] + rr * a.sin())
            })
            .collect();
        let tip = pts[10];
        let before = pts[9];
        let (dx, dy) = (tip[0] - before[0], tip[1] - before[1]);
        let len = dx.hypot(dy).max(1e-9);
        let (ux, uy) = (dx / len, dy / len);
        let h = 0.13 * scale;
        let ink_width = 3.5 * scale.sqrt();
        let head = polyline(vec![
            v(tip[0] - h * ux + 0.6 * h * uy, tip[1] - h * uy - 0.6 * h * ux),
            tip,
            v(tip[0] - h * ux - 0.6 * h * uy, tip[1] - h * uy + 0.6 * h * ux),
        ])
        .style(line_style(Palette::ink(), ink_width))
        .build();
        let shaft = polyline(pts).style(line_style(Palette::ink(), ink_width)).build();
        let arrow = stage.arena_mut().add(v_group([shaft, head]));
        let body = group(stage, &[circle, line])?;
        Ok(Self {
            body,
            circle,
            line,
            arrow,
            at,
            scale,
        })
    }

    /// What to add: the body and the arrow.
    pub fn mobs(&self) -> [Mob; 2] {
        [self.body, self.arrow]
    }

    /// Shrink or grow about its centre by `factor` and move to `to`, `rt`
    /// seconds: the animations for body and arrow.
    pub fn move_to(
        &self,
        stage: &mut Stage<'_>,
        factor: f64,
        to: Vec3,
        rt: f64,
    ) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        let shift = [to[0] - self.at[0], to[1] - self.at[1], 0.0];
        for m in self.mobs() {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .scale_about(factor, Some(self.at), None)?
                .shift(shift)?;
            out.push(stage.prepare(a)?);
        }
        Ok(out)
    }
}

/// A horizontal bar whose length is `f` (read from the arena each frame) in
/// units of `unit` scene units, growing right from `origin`: the kit's
/// `live_bar`, for a value that is a function of trackers rather than one.
pub fn live_bar_of(
    stage: &mut Stage<'_>,
    f: impl Fn(&Arena) -> f64 + 'static,
    origin: Vec3,
    unit: f64,
    height: f64,
    color: Srgb,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let w = (f(s).abs() * unit).max(0.004);
        let bar = Rectangle::new()
            .width(w)
            .height(height)
            .style(Style::default().fill(color, 0.85).stroke(color, 0.0, 1.0))
            .build()
            .expect("positive extents");
        let m = s.add(bar);
        s.move_to(m, origin, LEFT);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A dashed rectangle outline of `dashes` dashes a side-pair, centred at
/// `at`: a bad spot, a wrong path (always dashed or crossed). One group,
/// detached.
pub fn dashed_rect(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let (x0, x1, y0, y1) = (at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0);
    let mut dashes = Vec::new();
    let sides = [
        (v(x0, y0), v(x1, y0)),
        (v(x1, y0), v(x1, y1)),
        (v(x1, y1), v(x0, y1)),
        (v(x0, y1), v(x0, y0)),
    ];
    for (a, b) in sides {
        let len = (b[0] - a[0]).hypot(b[1] - a[1]);
        let n = (len / 0.28).round().max(2.0) as usize;
        for k in 0..n {
            let (t0, t1) = ((k as f64 + 0.1) / n as f64, (k as f64 + 0.65) / n as f64);
            let p = |t: f64| v(a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
            let dash = polyline(vec![p(t0), p(t1)]).style(line_style(color, width)).build();
            dashes.push(stage.arena_mut().add(dash));
        }
    }
    group(stage, &dashes)
}

/// A small tag: a few words on a rounded back, for "for programmers" and the
/// like (smaller than a lower third, and it sits by the thing it names).
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

// ----------------------------------------------------------- the projection

/// How far the fixed view is turned round the vertical, and raised
/// (degrees): from the front right and above (storyboard, "Three-dimensional
/// pictures"). Axes: x to the right, y away from the viewer, z up.
pub const VIEW_AZIMUTH_DEG: f64 = 32.0;
pub const VIEW_ELEVATION_DEG: f64 = 22.0;

/// The one fixed view: a parallel projection, `scale` scene units per unit of
/// the world, the world's origin at the scene point `at`. A close-up is the
/// same view with a larger scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub scale: f64,
    pub at: Vec3,
}

impl View {
    pub fn new(scale: f64, at: Vec3) -> Self {
        Self { scale, at }
    }

    /// The screen's right, up and toward-the-viewer directions in world
    /// coordinates (a right-handed, orthonormal triple).
    pub fn basis() -> (V3, V3, V3) {
        let (a, e) = (VIEW_AZIMUTH_DEG.to_radians(), VIEW_ELEVATION_DEG.to_radians());
        let right = [a.cos(), a.sin(), 0.0];
        let up = [-a.sin() * e.sin(), a.cos() * e.sin(), e.cos()];
        let toward = [a.sin() * e.cos(), -a.cos() * e.cos(), e.sin()];
        (right, up, toward)
    }

    /// Where world point `p` lands on the screen.
    pub fn project(&self, p: V3) -> Vec3 {
        let (right, up, _) = Self::basis();
        v(
            self.at[0] + self.scale * dot(p, right),
            self.at[1] + self.scale * dot(p, up),
        )
    }

    /// How near the viewer `p` is (bigger is nearer).
    pub fn depth(p: V3) -> f64 {
        dot(p, Self::basis().2)
    }

    /// The box on the screen (lower-left, upper-right) that the world points
    /// `pts` project into.
    #[cfg(test)]
    pub fn box_of(&self, pts: impl IntoIterator<Item = V3>) -> ([f64; 2], [f64; 2]) {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in pts {
            let s = self.project(p);
            for k in 0..2 {
                lo[k] = lo[k].min(s[k]);
                hi[k] = hi[k].max(s[k]);
            }
        }
        (lo, hi)
    }
}

/// The half width and half height of the safe area the layout lint holds the
/// picture to (the frame, ±7.11 by ±4, less its 0.15 margin).
#[cfg(test)]
pub const SAFE: [f64; 2] = [6.96, 3.85];
/// The top edge of a one-line lower third, rounded up a little (it is at
/// -3.14); a two-line one reaches -2.78, a three-line one -2.42.
#[cfg(test)]
pub const LOWER_THIRD_TOP: f64 = -3.1;

// ---------------------------------------------------------------- the world

/// One drawn thing, in the coordinates of the part it belongs to.
#[derive(Clone, Debug)]
pub enum Shape {
    /// A flat polygon, its corners counter-clockwise as seen from outside:
    /// drawn only while it faces the viewer. `shaded` darkens it by its
    /// direction; `bias` pulls it towards the viewer in the sort (a decal on
    /// a face).
    Face {
        pts: Vec<V3>,
        fill: Srgb,
        edge: Option<(Srgb, f64)>,
        shaded: bool,
        bias: f64,
    },
    /// A path through points, open or closed.
    Path {
        pts: Vec<V3>,
        color: Srgb,
        width: f64,
        closed: bool,
        bias: f64,
    },
    /// A straight line of `dashes` dashes.
    Dashed {
        a: V3,
        b: V3,
        color: Srgb,
        width: f64,
        dashes: usize,
    },
    /// A straight arrow whose head is drawn flat on the screen, `head` long.
    Arrow {
        from: V3,
        to: V3,
        color: Srgb,
        width: f64,
        head: f64,
    },
    /// A dot of screen radius `radius`.
    Dot { at: V3, color: Srgb, radius: f64 },
}

impl Shape {
    /// The polygons this shape needs in the pool.
    fn slots(&self) -> usize {
        match self {
            Shape::Dashed { dashes, .. } => *dashes,
            Shape::Arrow { .. } => 2,
            _ => 1,
        }
    }

    /// Every point that places the shape: corners, path points, ends, centre.
    #[cfg(test)]
    pub fn points(&self) -> Vec<V3> {
        match self {
            Shape::Face { pts, .. } | Shape::Path { pts, .. } => pts.clone(),
            Shape::Dashed { a, b, .. } => vec![*a, *b],
            Shape::Arrow { from, to, .. } => vec![*from, *to],
            Shape::Dot { at, .. } => vec![*at],
        }
    }

    /// A straight line.
    pub fn line(a: V3, b: V3, color: Srgb, width: f64) -> Shape {
        Shape::Path {
            pts: vec![a, b],
            color,
            width,
            closed: false,
            bias: 0.0,
        }
    }

    /// A circle of `radius` about `centre` in the plane perpendicular to
    /// `normal`, as `n` points.
    pub fn circle(centre: V3, normal: V3, radius: f64, n: usize, color: Srgb, width: f64) -> Shape {
        Shape::Path {
            pts: circle_points(centre, normal, radius, 0.0, TAU, n),
            color,
            width,
            closed: true,
            bias: 0.0,
        }
    }
}

/// Two unit vectors at right angles to each other and to `normal`, in that
/// order a right-handed turn about it: where angle 0 and a quarter turn point
/// in a circle about `normal`.
pub fn perp_basis(normal: V3) -> (V3, V3) {
    let nrm = unit(normal);
    let helper = if nrm[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let e1 = unit(cross(helper, nrm));
    (e1, cross(nrm, e1))
}

/// `n + 1` points of a circle's arc from angle `from` to `to` (radians,
/// counter-clockwise about `normal`), in the plane through `centre`.
pub fn circle_points(centre: V3, normal: V3, radius: f64, from: f64, to: f64, n: usize) -> Vec<V3> {
    let (e1, e2) = perp_basis(normal);
    (0..=n)
        .map(|k| {
            let a = from + (to - from) * k as f64 / n.max(1) as f64;
            add(centre, add(scale(e1, radius * a.cos()), scale(e2, radius * a.sin())))
        })
        .collect()
}

/// A box's six faces about `at` (half sizes `half`), counter-clockwise from
/// outside, coloured `[x−, x+, y−, y+, z−, z+]`.
pub fn cuboid(at: V3, half: V3, colors: [Srgb; 6], edge: Option<(Srgb, f64)>) -> Vec<Shape> {
    let mut out = Vec::new();
    for axis in 0..3 {
        for (k, sign) in [-1.0, 1.0].into_iter().enumerate() {
            let (u, w) = ((axis + 1) % 3, (axis + 2) % 3);
            let mut quad: Vec<V3> = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                .iter()
                .map(|&(su, sw)| {
                    let mut p = [0.0; 3];
                    p[axis] = sign * half[axis];
                    p[u] = su * half[u];
                    p[w] = sw * half[w];
                    add(at, p)
                })
                .collect();
            if sign < 0.0 {
                quad.reverse();
            }
            out.push(Shape::Face {
                pts: quad,
                fill: colors[2 * axis + k],
                edge,
                shaded: true,
                bias: 0.0,
            });
        }
    }
    out
}

/// A rectangle in the plane `z = height` with its corners at `lo` and `hi`
/// (x, y), facing up: a decal, a table top, a floor.
pub fn plate(lo: [f64; 2], hi: [f64; 2], height: f64, fill: Srgb, bias: f64) -> Shape {
    Shape::Face {
        pts: vec![
            [lo[0], lo[1], height],
            [hi[0], lo[1], height],
            [hi[0], hi[1], height],
            [lo[0], hi[1], height],
        ],
        fill,
        edge: None,
        shaded: false,
        bias,
    }
}

/// A screw, drawn the one way (storyboard, "The screw, drawn one way"): a gold
/// axis that runs `reach` out of the point at both ends, a curled arrow for
/// the turn (radius `curl`) and a straight one for the slide, starting where
/// the curl does, and the thread faintly past both ends of the motion. A
/// pure slide has the axis and the straight arrow only.
pub fn screw_shapes(s: &Screw, reach: f64, curl: f64) -> Vec<Shape> {
    let gold = palette::screw();
    let axis = unit(s.axis);
    let mut out = vec![Shape::line(
        sub(s.point, scale(axis, reach)),
        add(s.point, scale(axis, reach)),
        gold,
        5.0,
    )];
    let (e1, e2) = perp_basis(axis);
    let on_thread = |f: f64| {
        let a = s.angle * f;
        add(
            add(s.point, scale(axis, s.slide * f)),
            add(scale(e1, curl * a.cos()), scale(e2, curl * a.sin())),
        )
    };
    if s.angle.abs() > 1e-9 {
        let n = 28;
        let arc: Vec<V3> = (0..=n).map(|k| on_thread(k as f64 / n as f64)).collect();
        out.push(Shape::Path {
            pts: arc[..n].to_vec(),
            color: gold,
            width: 4.0,
            closed: false,
            bias: 0.0,
        });
        out.push(Shape::Arrow {
            from: arc[n - 1],
            to: arc[n],
            color: gold,
            width: 4.0,
            head: 0.22,
        });
        // the thread, faint, a quarter of the motion past each end
        let faint = mix(gold, ground(), 0.55);
        let thread: Vec<V3> = (0..=2 * n)
            .map(|k| on_thread(-0.25 + 1.5 * k as f64 / (2 * n) as f64))
            .collect();
        out.push(Shape::Path {
            pts: thread,
            color: faint,
            width: 2.2,
            closed: false,
            bias: -0.01,
        });
    }
    if s.slide.abs() > 1e-9 {
        out.push(Shape::Arrow {
            from: on_thread(0.0),
            to: add(on_thread(0.0), scale(axis, s.slide)),
            color: gold,
            width: 4.0,
            head: 0.22,
        });
    }
    out
}

/// A spin, drawn the one way: an orange arrow from `from` along `axis`,
/// `len` long (its length is its speed), and a curl round its tip for the
/// sense of the turning (radius `curl`, three quarters of a turn). The flat
/// piece's arrows are always this colour.
pub fn spin_arrow(from: V3, axis: V3, len: f64, curl: f64) -> Vec<Shape> {
    let orange = palette::flat();
    let a = unit(axis);
    let tip = add(from, scale(a, len));
    let n = 18;
    let arc = circle_points(add(from, scale(a, 0.8 * len)), a, curl, 0.0, 1.5 * PI, n);
    vec![
        Shape::Arrow {
            from,
            to: tip,
            color: orange,
            width: 5.0,
            head: 0.26,
        },
        Shape::Path {
            pts: arc[..n].to_vec(),
            color: orange,
            width: 4.0,
            closed: false,
            bias: 0.0,
        },
        Shape::Arrow {
            from: arc[n - 1],
            to: arc[n],
            color: orange,
            width: 4.0,
            head: 0.22,
        },
    ]
}

/// How a part is placed this frame (read from trackers and other mobjects).
pub type PoseFn = Rc<dyn Fn(&Arena) -> Pose>;
/// A world point read this frame.
pub type PointFn = Rc<dyn Fn(&Arena) -> V3>;

/// A pose that never changes.
pub fn still(pose: Pose) -> PoseFn {
    Rc::new(move |_: &Arena| pose)
}

/// A pose computed from the arena each frame.
pub fn pose_fn(f: impl Fn(&Arena) -> Pose + 'static) -> PoseFn {
    Rc::new(f)
}

/// A world point computed from the arena each frame.
pub fn point_fn(f: impl Fn(&Arena) -> V3 + 'static) -> PointFn {
    Rc::new(f)
}

/// A tracker's value now.
pub fn value(s: &Arena, tracker: Mob) -> f64 {
    s.tracker_value(tracker).unwrap_or(0.0)
}

/// Turns about fixed lines through `centre`, one after another: step `k`
/// turns `angle_k × tracker_k` about `axis_k`, and a later step multiplies on
/// the left (the lines are the room's, not the thing's). The shapes of the
/// part are drawn about the origin and carried to `centre`.
pub fn turned_about(centre: V3, steps: Vec<(V3, Mob, f64)>) -> PoseFn {
    Rc::new(move |s: &Arena| {
        let turns: Vec<(V3, f64)> = steps
            .iter()
            .map(|&(axis, tracker, angle)| (axis, angle * value(s, tracker)))
            .collect();
        Pose {
            r: turns_in_order(&turns),
            p: centre,
        }
    })
}

/// Turns about fixed lines, first to last: each later turn multiplies on the
/// left, because the lines are the room's and not the thing's.
pub fn turns_in_order(turns: &[(V3, f64)]) -> model::M3 {
    turns
        .iter()
        .fold(model::I3, |r, &(axis, angle)| model::mul(&model::turn(axis, angle), &r))
}

/// One part of a world: shapes in its own coordinates, where it is, and
/// which layer it is drawn in (layers go in ascending order; inside a layer
/// the farther shape goes first).
#[derive(Clone)]
pub struct PartSpec {
    pub shapes: Vec<Shape>,
    pub pose: PoseFn,
    pub layer: i8,
}

impl PartSpec {
    pub fn new(shapes: Vec<Shape>, pose: PoseFn) -> Self {
        Self { shapes, pose, layer: 0 }
    }

    pub fn layer(mut self, layer: i8) -> Self {
        self.layer = layer;
        self
    }
}

/// A label that follows a world point: `mob` is moved so that its centre sits
/// at the point's projection plus `shift`, every frame.
#[derive(Clone)]
pub struct Pin {
    pub mob: Mob,
    pub at: PointFn,
    pub shift: Vec3,
}

/// The two trackers every part has: 0 to 1 how present it is, and 0 to 1 how
/// far it has gone grey.
#[derive(Clone, Copy, Debug)]
pub struct PartCtl {
    pub shown: Mob,
    pub grey: Mob,
}

/// A 3D picture: parts drawn through the fixed [`View`] into a pool of
/// polygons. Add [`World::mobs`] to the scene, then bring it in by tweening
/// `shown`.
pub struct World {
    pub view: View,
    /// 0 to 1: how present the whole world is (it starts at 0).
    pub shown: Mob,
    pub parts: Vec<PartCtl>,
    pub anchor: Mob,
    pool: Vec<Mob>,
}

/// The light, from above, the front and the left.
const LIGHT: V3 = [-0.35, -0.55, 0.76];
/// A greyed part's colour weight.
const GREY_MIX: f64 = 0.8;

/// A draw call: one polygon of the pool this frame.
struct Draw {
    layer: i8,
    depth: f64,
    pts: Vec<Vec3>,
    closed: bool,
    fill: Option<(Srgb, f64)>,
    stroke: Option<(Srgb, f64, f64)>,
}

impl World {
    /// Build the world for `view`. `radius` is how far from the origin
    /// anything goes (farther parts are dimmed by up to `dim` of the way to
    /// the background); `pins` are labels that follow world points.
    pub fn build(
        stage: &mut Stage<'_>,
        view: View,
        radius: f64,
        dim: f64,
        specs: Vec<PartSpec>,
        pins: Vec<Pin>,
    ) -> fmn::Result<World> {
        let shown = stage.add_value_tracker(0.0);
        let parts: Vec<PartCtl> = specs
            .iter()
            .map(|_| PartCtl {
                shown: stage.add_value_tracker(1.0),
                grey: stage.add_value_tracker(0.0),
            })
            .collect();
        let slots: usize = specs.iter().flat_map(|p| &p.shapes).map(Shape::slots).sum();
        let pool: Vec<Mob> = (0..slots)
            .map(|_| {
                let quad = [[0.0, 0.0, 0.0], [0.001, 0.0, 0.0], [0.001, 0.001, 0.0]];
                stage.arena_mut().add(
                    Polygon::new(quad)
                        .style(
                            Style::default()
                                .fill(Palette::ink(), 0.0)
                                .stroke(Palette::ink(), 0.0, 0.0),
                        )
                        .build(),
                )
            })
            .collect();
        let anchor = stage.arena_mut().add(VMobject::new());
        let (right, up, toward) = View::basis();
        let draw_pool = pool.clone();
        let ctl = parts.clone();
        let mut last: Option<Vec<f64>> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let world_shown = value(s, shown).clamp(0.0, 1.0);
                let mut key = vec![world_shown];
                let mut posed: Vec<(Pose, f64, f64)> = Vec::with_capacity(specs.len());
                for (spec, c) in specs.iter().zip(&ctl) {
                    let pose = (spec.pose)(s);
                    let (sh, gr) = (value(s, c.shown).clamp(0.0, 1.0), value(s, c.grey).clamp(0.0, 1.0));
                    key.extend(pose.r.iter().flatten().copied());
                    key.extend(pose.p);
                    key.extend([sh, gr]);
                    posed.push((pose, sh, gr));
                }
                let mut pinned: Vec<V3> = Vec::with_capacity(pins.len());
                for pin in &pins {
                    let p = (pin.at)(s);
                    key.extend(p);
                    pinned.push(p);
                }
                if last.as_ref() == Some(&key) {
                    return;
                }
                let mut draws: Vec<Draw> = Vec::with_capacity(draw_pool.len());
                for (spec, &(pose, sh, gr)) in specs.iter().zip(&posed) {
                    let opacity = world_shown * sh;
                    let ink = |c: Srgb, shade: f64, depth: f64| -> Srgb {
                        let c = mix(c, greyed(), gr * GREY_MIX);
                        let c = mix(ground(), c, shade);
                        let far = ((radius - depth) / (2.0 * radius).max(1e-9)).clamp(0.0, 1.0);
                        mix(c, ground(), dim * far)
                    };
                    for shape in &spec.shapes {
                        match shape {
                            Shape::Face {
                                pts,
                                fill,
                                edge,
                                shaded,
                                bias,
                            } => {
                                let w: Vec<V3> = pts.iter().map(|&p| pose.at(p)).collect();
                                if w.len() < 3 {
                                    continue;
                                }
                                let n = cross(sub(w[1], w[0]), sub(w[2], w[0]));
                                if norm(n) < 1e-12 {
                                    continue;
                                }
                                let n = unit(n);
                                if dot(n, toward) <= 1e-6 {
                                    continue;
                                }
                                let centroid = scale(w.iter().fold([0.0; 3], |a, &p| add(a, p)), 1.0 / w.len() as f64);
                                let depth = dot(centroid, toward);
                                let shade = if *shaded {
                                    0.55 + 0.45 * dot(n, LIGHT).max(0.0)
                                } else {
                                    1.0
                                };
                                let fill_c = ink(*fill, shade, depth);
                                let stroke = edge.as_ref().map(|&(c, wd)| (ink(c, 1.0, depth), wd, 0.9 * opacity));
                                draws.push(Draw {
                                    layer: spec.layer,
                                    depth: depth + bias,
                                    pts: w.iter().map(|&p| project(&view, p, right, up)).collect(),
                                    closed: true,
                                    fill: Some((fill_c, opacity)),
                                    stroke,
                                });
                            }
                            Shape::Path {
                                pts,
                                color,
                                width,
                                closed,
                                bias,
                            } => {
                                if pts.len() < 2 {
                                    continue;
                                }
                                let w: Vec<V3> = pts.iter().map(|&p| pose.at(p)).collect();
                                let centroid = scale(w.iter().fold([0.0; 3], |a, &p| add(a, p)), 1.0 / w.len() as f64);
                                let depth = dot(centroid, toward);
                                draws.push(Draw {
                                    layer: spec.layer,
                                    depth: depth + bias,
                                    pts: w.iter().map(|&p| project(&view, p, right, up)).collect(),
                                    closed: *closed,
                                    fill: None,
                                    stroke: Some((ink(*color, 1.0, depth), *width, opacity)),
                                });
                            }
                            Shape::Dashed {
                                a,
                                b,
                                color,
                                width,
                                dashes,
                            } => {
                                let (a, b) = (pose.at(*a), pose.at(*b));
                                for k in 0..*dashes {
                                    let (t0, t1) =
                                        ((k as f64 + 0.15) / *dashes as f64, (k as f64 + 0.65) / *dashes as f64);
                                    let at = |t: f64| add(a, scale(sub(b, a), t));
                                    let (p0, p1) = (at(t0), at(t1));
                                    let depth = dot(scale(add(p0, p1), 0.5), toward);
                                    draws.push(Draw {
                                        layer: spec.layer,
                                        depth,
                                        pts: vec![project(&view, p0, right, up), project(&view, p1, right, up)],
                                        closed: false,
                                        fill: None,
                                        stroke: Some((ink(*color, 1.0, depth), *width, opacity)),
                                    });
                                }
                            }
                            Shape::Arrow {
                                from,
                                to,
                                color,
                                width,
                                head,
                            } => {
                                let (a, b) = (pose.at(*from), pose.at(*to));
                                let depth = dot(scale(add(a, b), 0.5), toward);
                                let (pa, pb) = (project(&view, a, right, up), project(&view, b, right, up));
                                let (dx, dy) = (pb[0] - pa[0], pb[1] - pa[1]);
                                let len = dx.hypot(dy).max(1e-9);
                                let (ux, uy) = (dx / len, dy / len);
                                let h = head.min(len * 0.6);
                                let base = v(pb[0] - ux * h, pb[1] - uy * h);
                                let c = ink(*color, 1.0, depth);
                                draws.push(Draw {
                                    layer: spec.layer,
                                    depth,
                                    pts: vec![pa, base],
                                    closed: false,
                                    fill: None,
                                    stroke: Some((c, *width, opacity)),
                                });
                                let half = h * 0.42;
                                draws.push(Draw {
                                    layer: spec.layer,
                                    depth: depth + 1e-6,
                                    pts: vec![
                                        pb,
                                        v(base[0] - uy * half, base[1] + ux * half),
                                        v(base[0] + uy * half, base[1] - ux * half),
                                    ],
                                    closed: true,
                                    fill: Some((c, opacity)),
                                    stroke: None,
                                });
                            }
                            Shape::Dot { at, color, radius: r } => {
                                let p = pose.at(*at);
                                let depth = dot(p, toward);
                                let c = project(&view, p, right, up);
                                let ring: Vec<Vec3> = (0..14)
                                    .map(|k| {
                                        let a = TAU * k as f64 / 14.0;
                                        v(c[0] + r * a.cos(), c[1] + r * a.sin())
                                    })
                                    .collect();
                                draws.push(Draw {
                                    layer: spec.layer,
                                    depth,
                                    pts: ring,
                                    closed: true,
                                    fill: Some((ink(*color, 1.0, depth), opacity)),
                                    stroke: None,
                                });
                            }
                        }
                    }
                }
                draws.sort_by(|a, b| a.layer.cmp(&b.layer).then(a.depth.total_cmp(&b.depth)));
                for (&slot, d) in draw_pool.iter().zip(&draws) {
                    let outline = if d.closed {
                        Polygon::new(d.pts.clone()).build()
                    } else {
                        polyline(d.pts.clone()).build()
                    };
                    let _ = s.set_points(slot, outline.points());
                    match d.fill {
                        Some((c, o)) => {
                            s.set_fill(slot, Some(c), Some(o), None, false);
                        }
                        None => {
                            s.set_fill(slot, None, Some(0.0), None, false);
                        }
                    }
                    match d.stroke {
                        Some((c, w, o)) => {
                            s.set_stroke(slot, Some(c), Some(w), Some(o), None, false);
                        }
                        None => {
                            s.set_stroke(slot, None, None, Some(0.0), None, false);
                        }
                    }
                }
                for &slot in draw_pool.iter().skip(draws.len()) {
                    s.set_fill(slot, None, Some(0.0), None, false);
                    s.set_stroke(slot, None, None, Some(0.0), None, false);
                }
                for (pin, p) in pins.iter().zip(&pinned) {
                    let at = project(&view, *p, right, up);
                    s.move_to(pin.mob, v(at[0] + pin.shift[0], at[1] + pin.shift[1]), ORIGIN);
                }
                last = Some(key);
            },
            true,
        )?;
        Ok(World {
            view,
            shown,
            parts,
            anchor,
            pool,
        })
    }

    /// Everything to add to the scene (the pool back to front is the
    /// updater's business, not the scene's).
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(&self.pool);
        out
    }

    /// Put the world in the scene (still invisible: `shown` is 0).
    pub fn add_to_scene(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        for m in self.mobs() {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    /// Bring the whole world in, `rt` seconds.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.shown, 1.0, rt)?)?;
        Ok(())
    }

    /// Take the whole world out, `rt` seconds.
    pub fn hide(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.shown, 0.0, rt)?)?;
        Ok(())
    }
}

/// A world point on the screen.
fn project(view: &View, p: V3, right: V3, up: V3) -> Vec3 {
    v(
        view.at[0] + view.scale * dot(p, right),
        view.at[1] + view.scale * dot(p, up),
    )
}

/// The three arrows along a body's own edges (x, y, z): the orientation's
/// blue in three weights, pale, full and deep, so that "each edge lands where
/// the next one was" can be followed.
pub fn edge_colors() -> [Srgb; 3] {
    [hex("#A9C9FF"), palette::turn(), hex("#2C58A8")]
}

/// Three equal arrows from the body's origin along its own x, y and z.
pub fn edge_arrows(len: f64) -> Vec<Shape> {
    edge_colors()
        .iter()
        .enumerate()
        .map(|(k, &color)| {
            let mut to = [0.0; 3];
            to[k] = len;
            Shape::Arrow {
                from: [0.0; 3],
                to,
                color,
                width: 5.0,
                head: 0.26,
            }
        })
        .collect()
}

/// `n + 1` points of the shorter arc from direction `u` to direction `w`
/// (unit vectors that are not opposite) on the sphere of radius `r` about the
/// origin.
pub fn arc_between(u: V3, w: V3, r: f64, n: usize) -> Vec<V3> {
    let phi = dot(u, w).clamp(-1.0, 1.0).acos();
    let s = phi.sin();
    (0..=n)
        .map(|k| {
            let t = k as f64 / n.max(1) as f64;
            if s < 1e-9 {
                return scale(u, r);
            }
            let p = add(scale(u, ((1.0 - t) * phi).sin() / s), scale(w, (t * phi).sin() / s));
            scale(p, r)
        })
        .collect()
}

// ------------------------------------------------------------- matrix grids

/// Where the nine cells of a 3 by 3 grid centred at `at` sit, `pitch` apart
/// across and down: row `i`, column `j`.
pub fn grid_spots(at: Vec3, pitch: (f64, f64)) -> [[Vec3; 3]; 3] {
    std::array::from_fn(|i| {
        std::array::from_fn(|j| v(at[0] + pitch.0 * (j as f64 - 1.0), at[1] - pitch.1 * (i as f64 - 1.0)))
    })
}

/// The two square brackets round a grid at `at` (detached strokes).
pub fn grid_brackets(stage: &mut Stage<'_>, at: Vec3, pitch: (f64, f64), color: Srgb) -> [Mob; 2] {
    let (hw, hh, hook) = (1.62 * pitch.0, 1.5 * pitch.1, 0.22 * pitch.0);
    [-1.0, 1.0].map(|side| {
        let x = at[0] + side * hw;
        let bracket = polyline(vec![
            v(x - side * hook, at[1] + hh),
            v(x, at[1] + hh),
            v(x, at[1] - hh),
            v(x - side * hook, at[1] - hh),
        ])
        .style(line_style(color, 3.0))
        .build();
        stage.arena_mut().add(bracket)
    })
}

/// A matrix entry as printed: whole numbers plain, anything else to two
/// places, a true minus sign (U+2212), and never "−0". On screen beside live
/// numbers use [`screen_entry`].
pub fn entry(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0 + 0.0;
    let s = if (r - r.round()).abs() < 1e-9 {
        format!("{}", r.round() as i64)
    } else {
        format!("{r:.2}")
    };
    s.replace('-', "−")
}

/// The minus a number is printed with on screen: the engine's live numbers
/// (`DecimalNumber`) show a negative with an en dash at mid-digit height, and
/// the text face has no U+2212 of its own (it falls back to a heavier face),
/// so a static number uses the same character and sits beside a live one
/// alike.
pub const MINUS: char = '\u{2013}';

/// [`entry`] with the engine's own minus ([`MINUS`]): a matrix cell or a
/// label that appears next to live numbers.
pub fn screen_entry(x: f64) -> String {
    entry(x).replace('−', &MINUS.to_string())
}

/// A 3 by 3 grid of fixed numbers, column `j` in `colors[j]`: nine cells and
/// the two brackets, detached, cells in row-major order then the brackets.
pub fn static_grid(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rows: [[String; 3]; 3],
    at: Vec3,
    pitch: (f64, f64),
    size: f64,
    colors: [Srgb; 3],
) -> fmn::Result<Vec<Mob>> {
    let spots = grid_spots(at, pitch);
    let mut out = Vec::with_capacity(11);
    for (i, row) in rows.iter().enumerate() {
        for (j, text) in row.iter().enumerate() {
            out.push(kit.text(stage, text, size, colors[j], spots[i][j])?);
        }
    }
    out.extend(grid_brackets(stage, at, pitch, Palette::dim()));
    Ok(out)
}

// ---------------------------------------------------------- book and table

/// The book's half sizes (x across, y deep, z thick): about 1.5 by 2.2 and
/// thin, in world units.
pub const BOOK_HALF: V3 = [0.78, 1.08, 0.15];

/// A book about its own centre: the cover up (blue, with a lighter plate set
/// toward the near edge so every turn reads), the spine on the left (darker),
/// pages on the other three sides, the back cover below.
pub fn book(half: V3) -> Vec<Shape> {
    let cover = palette::turn();
    let spine = mix(cover, ground(), 0.4);
    let back = mix(cover, ground(), 0.55);
    let pages = paper();
    let edge = Some((ground(), 1.6));
    let mut out = cuboid([0.0; 3], half, [spine, pages, pages, pages, back, cover], edge);
    out.push(plate(
        [-0.3 * half[0], -0.8 * half[1]],
        [0.62 * half[0], -0.28 * half[1]],
        half[2] + 0.003,
        mix(cover, Palette::ink(), 0.55),
        0.02,
    ));
    out
}

/// A table: a top at height `z` from `lo` to `hi` (x, y), with a fine grid
/// of lines every `step` so that depth reads.
pub fn table(lo: [f64; 2], hi: [f64; 2], z: f64, step: f64) -> Vec<Shape> {
    let mut out = vec![plate(lo, hi, z, table_top(), -1.0)];
    let line = mix(table_top(), Palette::faint(), 0.5);
    let mut x = (lo[0] / step).ceil() * step;
    while x <= hi[0] + 1e-9 {
        out.push(Shape::line([x, lo[1], z], [x, hi[1], z], line, 1.2));
        x += step;
    }
    let mut y = (lo[1] / step).ceil() * step;
    while y <= hi[1] + 1e-9 {
        out.push(Shape::line([lo[0], y, z], [hi[0], y, z], line, 1.2));
        y += step;
    }
    out
}

// ---------------------------------------------------------------- the arm

/// Scene units per metre in the arm's pictures.
pub const ARM_SCALE: f64 = 2.5;

/// Where the arm's joint angles come from: seven trackers (radians), or one
/// tracker walking a table of poses (a solved path).
#[derive(Clone)]
pub enum QSource {
    Joints([Mob; N]),
    Path { keys: Rc<Vec<Q>>, at: Mob },
}

/// The arm, as the pictures draw it: its model and where its angles come from.
#[derive(Clone)]
pub struct ArmRig {
    pub arm: Arm,
    pub source: QSource,
}

impl ArmRig {
    /// Which of [`ArmRig::parts`]' parts are the links (the first, the
    /// fixed one, to the last), the steel collars and the gold ones.
    pub const LINKS: std::ops::Range<usize> = 1..9;
    pub const STEEL_COLLARS: std::ops::Range<usize> = 9..16;
    pub const GOLD_COLLARS: std::ops::Range<usize> = 16..23;

    /// An arm held at `q_deg` by seven trackers.
    pub fn joints(stage: &mut Stage<'_>, arm: Arm, q_deg: [f64; N]) -> Self {
        let trackers = std::array::from_fn(|i| stage.add_value_tracker(q_deg[i] * DEG));
        Self {
            arm,
            source: QSource::Joints(trackers),
        }
    }

    /// An arm walking `keys` (joint angles in radians) as one tracker goes
    /// from 0 to 1; the tracker comes back with it.
    pub fn along(stage: &mut Stage<'_>, arm: Arm, keys: Vec<Q>) -> (Self, Mob) {
        let at = stage.add_value_tracker(0.0);
        let rig = Self {
            arm,
            source: QSource::Path {
                keys: Rc::new(keys),
                at,
            },
        };
        (rig, at)
    }

    /// The joint angles now (radians).
    pub fn q(&self, s: &Arena) -> Q {
        match &self.source {
            QSource::Joints(t) => std::array::from_fn(|i| value(s, t[i])),
            QSource::Path { keys, at } => {
                let f = value(s, *at).clamp(0.0, 1.0) * (keys.len() - 1) as f64;
                let (k, rest) = (f.floor() as usize, f - f.floor());
                let next = (k + 1).min(keys.len() - 1);
                std::array::from_fn(|i| keys[k][i] + (keys[next][i] - keys[k][i]) * rest)
            }
        }
    }

    /// The pose of everything past the first `count` joints.
    pub fn carried(&self, count: usize) -> PoseFn {
        let rig = self.clone();
        Rc::new(move |s: &Arena| rig.arm.carried(&rig.q(s), count))
    }

    /// The tip's pose now.
    pub fn tip(&self) -> PoseFn {
        let rig = self.clone();
        Rc::new(move |s: &Arena| rig.arm.tip(&rig.q(s)))
    }

    /// Joint `i`'s axis now: a point on it and its direction.
    pub fn axis_now(&self, i: usize) -> (PointFn, PointFn) {
        let (a, b) = (self.clone(), self.clone());
        (
            Rc::new(move |s: &Arena| a.arm.joint(&a.q(s), i).0),
            Rc::new(move |s: &Arena| b.arm.joint(&b.q(s), i).1),
        )
    }

    /// The arm's parts, in this order: a foot, then the eight links (the
    /// first is fixed, link `k` rides joints `0..k`), then a steel collar at
    /// every joint, then a gold one (shown 0): `1 + 8 + 7 + 7 = 23` parts.
    /// `steel` is the links' colour. Metres in, the view carries the scale.
    pub fn parts(&self, steel: Srgb) -> Vec<PartSpec> {
        let arm = &self.arm;
        let edge = mix(steel, ground(), 0.7);
        let mut out = Vec::new();
        // the foot: a short thick post and a collar on the floor
        out.push(PartSpec::new(
            vec![
                Shape::line([0.0; 3], [0.0, 0.0, 0.06], edge, 30.0),
                Shape::line([0.0; 3], [0.0, 0.0, 0.06], steel, 22.0),
            ],
            still(Pose::NOTHING),
        ));
        // the links: home positions from the joints' origins, carried by the
        // joints before them
        let mut ends: Vec<V3> = vec![[0.0; 3]];
        ends.extend(arm.origin);
        ends.push(arm.home.p);
        for k in 0..ends.len() - 1 {
            let (a, b) = (ends[k], ends[k + 1]);
            out.push(PartSpec::new(
                vec![Shape::line(a, b, edge, 17.0), Shape::line(a, b, steel, 12.0)],
                self.carried(k.min(N)),
            ));
        }
        // the collars, steel then gold: a ring about each joint's axis
        for gold in [false, true] {
            for i in 0..N {
                let ring = |color: Srgb, width: f64| Shape::circle(arm.origin[i], arm.axis[i], 0.058, 20, color, width);
                let color = if gold {
                    palette::screw()
                } else {
                    mix(steel, Palette::ink(), 0.5)
                };
                let mut part = PartSpec::new(vec![ring(color, if gold { 6.0 } else { 4.0 })], self.carried(i));
                part.layer = 1;
                out.push(part);
            }
        }
        out
    }
}

/// Joint angles (radians) that carry the tip along the screw from `start` to
/// the pose `screw` leads to, one Newton solve per sample, each warm-started
/// from the last: `samples + 1` poses of the arm, the first being `q0`.
pub fn along_a_screw(arm: &Arm, q0: &Q, start: &Pose, screw: &Screw, samples: usize) -> Vec<Q> {
    let mut keys = vec![*q0];
    let mut q = *q0;
    for k in 1..=samples {
        let goal = model::along_screw(screw, k as f64 / samples as f64).then(start);
        q = arm.solve(&q, &goal, model::TOL_SPIN, model::TOL_SLIDE).q;
        keys.push(q);
    }
    keys
}

// ----------------------------------------------------------- the quiz card

/// One choice of a quiz: its `key` is what `G-Q.<line>.answer` says when it
/// is the right one; its `label` is typeset as TeX when it holds a
/// backslash, as plain text otherwise.
#[derive(Clone, Copy, Debug)]
pub struct Choice {
    pub key: &'static str,
    pub label: &'static str,
}

/// What a quiz card asks: the line that asks it (a row of `model::QUIZZES`),
/// the question on the card (`\n` breaks it) and the choices in order.
#[derive(Clone, Copy, Debug)]
pub struct Ask {
    pub line: &'static str,
    pub question: &'static str,
    pub choices: &'static [Choice],
}

impl Ask {
    /// The countdown's seconds, from `model::QUIZZES`.
    pub fn seconds(&self) -> fmn::Result<u32> {
        model::QUIZZES
            .iter()
            .find(|q| q.line == self.line)
            .map(|q| q.seconds)
            .ok_or_else(|| fail(format!("line {} asks no quiz in model::QUIZZES", self.line)))
    }

    /// The index of the right choice: the one whose key is the computed
    /// answer (`G-Q.<line>.answer`).
    pub fn answer_index(&self) -> fmn::Result<usize> {
        let answer = fact(&format!("G-Q.{}.answer", self.line))?;
        let hits: Vec<usize> = self
            .choices
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == answer)
            .map(|(i, _)| i)
            .collect();
        match hits.as_slice() {
            [i] => Ok(*i),
            _ => Err(fail(format!(
                "quiz {}: the answer {answer:?} matches {} of its choices",
                self.line,
                hits.len()
            ))),
        }
    }
}

/// How a quiz card sits: its centre and its size against the full card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuizLook {
    pub at: Vec3,
    pub scale: f64,
}

impl QuizLook {
    pub fn at(at: Vec3, scale: f64) -> Self {
        Self { at, scale }
    }
}

const PAUSE_LINE: &str = "pause for more time";
const CARD_LABEL: &str = "your turn";
/// The card's width at scale 1 (scene units).
pub const CARD_W: f64 = 5.4;
/// Where a choice's words stop, from the card's right edge: the tick of the
/// right choice stands in that strip.
const TICK_ROOM: f64 = 1.0;
const RING_R: f64 = 0.62;

/// The ring: what is left of a circle of radius `r` at `c`, drained
/// clockwise from the top.
fn ring_points(c: Vec3, r: f64, share: f64) -> Vec<Vec3> {
    let share = share.clamp(0.0, 1.0);
    let steps = ((64.0 * share).ceil() as usize).max(1);
    (0..=steps)
        .map(|k| {
            let a = FRAC_PI_2 + TAU * share * k as f64 / steps as f64;
            v(c[0] + r * a.cos(), c[1] + r * a.sin())
        })
        .collect()
}

/// The violet card of the quiz device: the question and its choices, a ring
/// that drains over the quiz's seconds with the seconds left in its middle
/// and "pause for more time" under it; at zero the right choice is ticked
/// where it stands and the ring goes. The answer is computed, never typed.
pub struct QuizCard {
    pub ask: Ask,
    pub look: QuizLook,
    pub seconds: u32,
    pub back: Mob,
    pub label: Mob,
    pub question: Mob,
    pub rows: Vec<Mob>,
    pub dots: Vec<Mob>,
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub pause_line: Mob,
    /// The right row's outline and tick (detached until the answer lands).
    pub verdict: Vec<Mob>,
    /// Seconds left (driven by [`QuizCard::count`]).
    pub left: Mob,
    /// 0 to 1: how much of the ring and its digit shows.
    pub shown: Mob,
    pub anchor: Mob,
    clock: Mob,
}

impl QuizCard {
    /// The distance between two choices' centres at scale `k`: a fraction's
    /// two storeys need more room than a line of words.
    fn pitch(ask: &Ask, k: f64) -> f64 {
        if ask.choices.iter().any(|c| c.label.contains("\\frac")) {
            0.95 * k
        } else {
            0.55 * k
        }
    }

    /// The card's height at scale `k` for a question of `lines` lines and
    /// the choices of `ask`.
    fn height(lines: usize, ask: &Ask, k: f64) -> f64 {
        k * (1.0 + 0.5 * lines as f64 + 2.1) + Self::pitch(ask, k) * ask.choices.len() as f64
    }

    /// The card for `ask`, detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, ask: Ask, look: QuizLook) -> fmn::Result<Self> {
        let seconds = ask.seconds()?;
        let right = ask.answer_index()?;
        let (at, k) = (look.at, look.scale);
        let lines = ask.question.lines().count().max(1);
        let pitch = Self::pitch(&ask, k);
        let (w, h) = (CARD_W * k, Self::height(lines, &ask, k));
        let top = at[1] + h / 2.0;
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.18 * k)
            .style(Style::default().fill(panel(), 0.96).stroke(palette::quiz(), 3.0, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let label = kit.text(
            stage,
            CARD_LABEL,
            (28.0 * k).max(24.0),
            palette::quiz(),
            v(at[0], top - 0.42 * k),
        )?;
        let q_y = top - (0.95 + 0.25 * (lines as f64 - 1.0)) * k;
        let question = kit.text(stage, ask.question, 40.0 * k, Palette::ink(), v(at[0], q_y))?;
        let room = w - 0.7 * k;
        let width = stage.get_width(question);
        if width > room {
            stage.scale(question, room / width);
        }
        let first_row = q_y - (0.55 + 0.25 * (lines as f64 - 1.0)) * k - 0.2 * k;
        let (mut rows, mut dots) = (Vec::new(), Vec::new());
        let left_x = at[0] - w / 2.0 + 0.95 * k;
        for (i, choice) in ask.choices.iter().enumerate() {
            let y = first_row - pitch * i as f64;
            let row = if choice.label.contains('\\') {
                kit.tex_left(stage, choice.label, 38.0 * k, Palette::ink(), &[], v(left_x, y))?
            } else {
                kit.text_left(stage, choice.label, 36.0 * k, Palette::ink(), v(left_x, y))?
            };
            rows.push(row);
            let dot = Circle::new()
                .radius(0.075 * k)
                .arc_center(v(left_x - 0.4 * k, y))
                .style(flat(palette::quiz(), 1.0))
                .build();
            dots.push(stage.arena_mut().add(dot));
        }
        // a choice longer than the card is wide would run out of it and under
        // the tick: all the choices shrink together until the longest fits
        let room = at[0] + w / 2.0 - TICK_ROOM * k - left_x;
        let widest = rows.iter().map(|&r| stage.get_width(r)).fold(0.0, f64::max);
        if widest > room {
            for (i, &row) in rows.iter().enumerate() {
                stage.scale(row, room / widest);
                stage.move_to(row, v(left_x, first_row - pitch * i as f64), LEFT);
            }
        }
        let last_row = first_row - pitch * (ask.choices.len().saturating_sub(1)) as f64;
        let ring_at = v(at[0], last_row - (pitch + 0.45 * k));
        let radius = RING_R * k;
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(radius)
                .arc_center(ring_at)
                .style(faded_line(Palette::faint(), 8.0 * k, 1.0))
                .build(),
        );
        let n = f64::from(seconds);
        let ring = stage.arena_mut().add(
            polyline(ring_points(ring_at, radius, 1.0))
                .style(faded_line(palette::quiz(), 9.0 * k, 0.0))
                .build(),
        );
        let mut digits = Vec::new();
        for second in 1..=seconds {
            let d = kit.text(stage, &second.to_string(), 52.0 * k, Palette::ink(), ring_at)?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
        let pause_line = kit.text(
            stage,
            PAUSE_LINE,
            (26.0 * k).max(22.0),
            Palette::dim(),
            v(at[0], ring_at[1] - radius - 0.4 * k),
        )?;
        // the verdict: an outline round the right row and a tick before it
        let ry = first_row - pitch * right as f64;
        let outline = Rectangle::new()
            .width(w - 0.5 * k)
            .height(pitch - 0.05 * k)
            .corner_radius(0.1 * k)
            .style(
                Style::default()
                    .fill(palette::good(), 0.0)
                    .stroke(palette::good(), 3.0, 1.0),
            )
            .build()?
            .moved_to(v(at[0], ry));
        let outline = stage.arena_mut().add(outline);
        let tx = at[0] + w / 2.0 - 0.55 * k;
        let tick = polyline(vec![
            v(tx - 0.16 * k, ry),
            v(tx - 0.04 * k, ry - 0.13 * k),
            v(tx + 0.18 * k, ry + 0.14 * k),
        ])
        .style(line_style(palette::good(), 6.0 * k))
        .build();
        let tick = stage.arena_mut().add(tick);
        let left = stage.add_value_tracker(n);
        let shown = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let clock = stage.arena_mut().add(VMobject::new());
        let seen = digits.clone();
        let mut last = (f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = (value(s, left), value(s, shown));
                if now == last {
                    return;
                }
                let drained = polyline(ring_points(ring_at, radius, now.0 / n)).build();
                let _ = s.set_points(ring, drained.points());
                s.set_stroke(ring, None, None, Some(now.1), None, false);
                let current = now.0.ceil() as usize;
                for (j, &d) in seen.iter().enumerate() {
                    let on = now.0 > 0.0 && current == j + 1;
                    s.set_opacity(d, if on { now.1 } else { 0.0 }, true);
                }
                last = now;
            },
            true,
        )?;
        Ok(Self {
            ask,
            look,
            seconds,
            back,
            label,
            question,
            rows,
            dots,
            track,
            ring,
            digits,
            pause_line,
            verdict: vec![outline, tick],
            left,
            shown,
            anchor,
            clock,
        })
    }

    /// Bring the card in with its question, choices and full ring, `rt`
    /// seconds.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        let mut ins = vec![self.back, self.label, self.question, self.track, self.pause_line];
        ins.extend(&self.rows);
        ins.extend(&self.dots);
        fade_in_all(stage, &ins, rt)?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        stage.play(tween(self.shown, 1.0, 0.25)?)?;
        Ok(())
    }

    /// The countdown: the narration silent, one soft note a second climbing
    /// the motif and left open (a listen window of the quiz's seconds and the
    /// answer's beat), the ring draining with the clock; at zero the ring goes,
    /// the right choice is outlined and ticked and the resolution winds down
    /// for [`sound::LAND_S`] before the next line.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = f64::from(self.seconds);
        // the question must be over before the window opens, or the clock
        // would run through the end of the line
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        let (left, mut elapsed) = (self.left, 0.0);
        stage.add_to_scene(self.clock)?;
        stage.add_dt_updater(
            self.clock,
            move |s, _, dt| {
                elapsed += dt;
                let _ = s.set_tracker_value(left, (n - elapsed).max(0.0));
            },
            true,
        )?;
        let label = format!("{}:{}", kit.chapter, self.ask.line);
        let wav = sound::countdown(self.seconds)?;
        kit.sonify(stage, n + sound::LAND_S, &label, &wav)?;
        let start = stage.scene().time().to_f64();
        wait_until(stage, start + n)?;
        stage.clear_updaters(self.clock, true);
        let _ = stage.set_tracker_value(self.left, 0.0);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(tween(self.shown, 0.0, 0.25)?)?);
        for m in [self.track, self.pause_line] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.25))?);
        }
        for &m in &self.verdict {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.25))?);
        }
        stage.play_prepared(anims)?;
        stage.remove_from_scene(self.ring);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        wait_until(stage, start + n + sound::LAND_S)
    }

    /// What the card still shows once the countdown is over.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back, self.label, self.question];
        out.extend(&self.rows);
        out.extend(&self.dots);
        out.extend(&self.verdict);
        out
    }
}

/// A quiz, start to finish: say line `ask.line`, bring the card in as the
/// question itself begins, count down, tick the answer. The next beat works
/// the answer in the same place.
pub fn your_turn(kit: &Kit, stage: &mut Stage<'_>, ask: Ask, look: QuizLook) -> fmn::Result<QuizCard> {
    let line = say(kit, stage, ask.line)?;
    let card = QuizCard::build(kit, stage, ask, look)?;
    line.wait_last(stage)?;
    card.show(stage, 0.5)?;
    card.count(kit, stage)?;
    Ok(card)
}

/// Run `construct` on a scratch stage with no sink: the scene's mobjects are
/// really built and measured, nothing is rendered.
#[cfg(test)]
pub fn probe(construct: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
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

/// Run a whole chapter's scene headless: every phrase it waits for is in its
/// line, and every label, tag and picture is built and placed.
#[cfg(test)]
pub fn run_chapter(scene: &mut impl SceneConstruct) {
    fmn::run_scene(
        scene,
        fmn::scene::RuntimeConfig::default(),
        7,
        &mut fmn::scene::NullSceneSink,
    )
    .expect("the chapter's scene runs");
}

/// The texts the kit built that a lower third cut to fit its three lines
/// (the kit's wrap ends such a line with " …" and drops the rest): an exact
/// version that is too long loses its tail without a word, so a test asserts
/// that there are none.
#[cfg(test)]
pub fn cut_texts(kit: &Kit) -> Vec<String> {
    kit.registered()
        .into_iter()
        .map(|r| r.source)
        .filter(|s| s.ends_with(" …"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use super::*;
    use crate::model::{X, Y, Z};

    fn close(a: V3, b: V3) -> bool {
        norm(sub(a, b)) < 1e-12
    }

    /// Numbers in sentences are spelled as the script spells them, and a
    /// number past twenty is an error, not a wrong word.
    #[test]
    fn numbers_are_spelled_to_twenty() {
        assert_eq!(number_word(3).unwrap(), "three");
        assert_eq!(number_word(9).unwrap(), "nine");
        assert_eq!(number_word(4).unwrap(), "four");
        assert_eq!(number_word(20).unwrap(), "twenty");
        assert!(number_word(21).is_err());
    }

    #[test]
    fn the_view_is_a_right_handed_orthonormal_triple() {
        let (right, up, toward) = View::basis();
        for u in [right, up, toward] {
            assert!((norm(u) - 1.0).abs() < 1e-12);
        }
        assert!(dot(right, up).abs() < 1e-12);
        assert!(dot(right, toward).abs() < 1e-12);
        assert!(dot(up, toward).abs() < 1e-12);
        assert!(close(cross(right, up), toward), "right x up is toward the viewer");
        // from the front right and above: the viewer's side of +x, -y and +z
        assert!(toward[0] > 0.0 && toward[1] < 0.0 && toward[2] > 0.0);
    }

    #[test]
    fn the_viewer_sees_the_cover_the_front_and_the_right_of_a_box() {
        let (_, _, toward) = View::basis();
        let shapes = cuboid([0.0; 3], BOOK_HALF, [Palette::ink(); 6], None);
        let faces: Vec<&Vec<V3>> = shapes
            .iter()
            .filter_map(|s| match s {
                Shape::Face { pts, .. } => Some(pts),
                _ => None,
            })
            .collect();
        assert_eq!(faces.len(), 6, "a box is six faces");
        let mut seen = Vec::new();
        for pts in faces {
            let n = unit(cross(sub(pts[1], pts[0]), sub(pts[2], pts[0])));
            // the normal points out of the box: away from its centre
            let mid = scale(pts.iter().fold([0.0; 3], |a, &p| add(a, p)), 0.25);
            assert!(dot(n, mid) > 0.0, "a face points away from the box's centre");
            if dot(n, toward) > 1e-6 {
                seen.push(n.map(|x| x.round() as i32));
            }
        }
        seen.sort();
        assert_eq!(seen, vec![[0, -1, 0], [0, 0, 1], [1, 0, 0]]);
    }

    #[test]
    fn a_pair_of_turns_about_the_rooms_lines_is_the_books_matrix() {
        let (a, b) = (
            turns_in_order(&[(X, FRAC_PI_2), (Y, FRAC_PI_2)]),
            model::tilt_then_roll(FRAC_PI_2),
        );
        assert!(model::gap(&a, &b) < 1e-12, "tilt, then roll");
        let (a, b) = (
            turns_in_order(&[(Y, FRAC_PI_2), (X, FRAC_PI_2)]),
            model::roll_then_tilt(FRAC_PI_2),
        );
        assert!(model::gap(&a, &b) < 1e-12, "roll, then tilt");
        // halfway through the first turn is half of it
        let half = turns_in_order(&[(X, FRAC_PI_2 / 2.0)]);
        assert!(model::gap(&half, &model::turn(X, PI / 4.0)) < 1e-12);
        // no turns, no change
        assert!(model::gap(&turns_in_order(&[]), &model::I3) < 1e-15);
    }

    #[test]
    fn a_circle_stays_on_its_circle_and_in_its_plane() {
        let (centre, normal) = ([0.3, -0.2, 1.1], unit([1.0, 2.0, -0.5]));
        for p in circle_points(centre, normal, 0.7, 0.0, TAU, 40) {
            let d = sub(p, centre);
            assert!((norm(d) - 0.7).abs() < 1e-12);
            assert!(dot(d, normal).abs() < 1e-12);
        }
        let (e1, e2) = perp_basis(normal);
        assert!(close(cross(e1, e2), normal), "a right-handed turn about the normal");
    }

    #[test]
    fn a_screw_is_one_axis_one_curl_one_slide_and_a_thread() {
        let s = Screw {
            angle: FRAC_PI_2,
            axis: Z,
            point: [0.0, 1.0, 0.0],
            slide: 0.5,
        };
        // axis, the arc, its arrow head, the thread, the slide arrow
        assert_eq!(screw_shapes(&s, 2.0, 0.4).len(), 5);
        let slide = Screw { angle: 0.0, ..s };
        assert_eq!(screw_shapes(&slide, 2.0, 0.4).len(), 2, "axis and slide only");
        let turn_only = Screw { slide: 0.0, ..s };
        assert_eq!(screw_shapes(&turn_only, 2.0, 0.4).len(), 4, "axis, curl and thread");
    }

    #[test]
    fn a_world_has_a_polygon_for_every_shape_it_can_draw() {
        let s = Screw {
            angle: 1.0,
            axis: Z,
            point: [0.0; 3],
            slide: 0.2,
        };
        let shapes = [
            screw_shapes(&s, 1.0, 0.3),
            book(BOOK_HALF),
            table([-1.0, -1.0], [1.0, 1.0], 0.0, 0.5),
        ]
        .concat();
        // an arrow is a shaft and a head; a dashed line is its dashes
        let want: usize = shapes
            .iter()
            .map(|sh| match sh {
                Shape::Arrow { .. } => 2,
                Shape::Dashed { dashes, .. } => *dashes,
                _ => 1,
            })
            .sum();
        assert_eq!(shapes.iter().map(Shape::slots).sum::<usize>(), want);
    }
}
