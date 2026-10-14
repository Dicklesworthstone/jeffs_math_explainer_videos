//! Video H's look, shared by every chapter (storyboard §3 "Color roles" and
//! "The dots"): the dots, a cell's blue heat recounted from the dots still
//! lit, the 1968 lattice with its searches, the teal strip, the "your turn"
//! frame with its ring, and the timing helpers that key a picture to the
//! words it illustrates.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, never transform a group of more than
//! `GROUP_TRANSFORM_LIMIT` members, and an `animate()` copies the whole
//! family, so a dot goes out by a slate twin fading in over it (never by
//! recolouring a group), and a map dims under one veil.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use std::cell::{Cell, RefCell};
use std::f64::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::prelude::*;

use super::sound;
use crate::model::{Q, QUIZZES, Quiz, y1968};

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

    /// The line's sentences as written, so what is set on screen is what
    /// is said (never retyped).
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

/// How long a question to carry (no ring) holds before the next beat.
pub const CARRY_HOLD_S: f64 = 2.0;

/// The pause after a question to carry: let the question finish, mark the
/// moment for the score and hold the picture [`CARRY_HOLD_S`].
pub fn carry_pause(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    pause(stage, CARRY_HOLD_S)
}

// ------------------------------------------------------------- colour roles

/// The map: a dot still lit, and a cell's heat.
pub fn map() -> Srgb {
    hex("#58C4DD")
}

/// The search: the searcher, its track and strip, the ring on a dot it
/// would see, and the chance of seeing.
pub fn search() -> Srgb {
    hex("#17806D")
}

/// The chance of a find, and the cell the rule chooses: nothing else.
pub fn find() -> Srgb {
    Palette::hero()
}

/// A dot a search has put out.
pub fn out() -> Srgb {
    hex("#566074")
}

/// The true place of a wreck, once the viewer is told it (a hollow ring).
pub fn true_place() -> Srgb {
    Palette::ink()
}

/// A last known position (not the true place): a small open diamond.
pub fn last_known() -> Srgb {
    Palette::dim()
}

/// A living person found (11_6 and 14_6 only).
pub fn found() -> Srgb {
    Palette::green()
}

/// An estimate that turned out wrong (11_3, 13_4, 14_3): never danger and
/// never the dead.
pub fn wrong() -> Srgb {
    Palette::red()
}

/// The "your turn" frame and its countdown.
pub fn quiz() -> Srgb {
    hex("#B189D6")
}

/// A card's or panel's back.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// The corpus background.
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

/// Fade every top-level mobject but `keep` out together, `rt` seconds: the
/// end of a chapter whose next chapter starts from what is kept.
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

/// Shrink `mob` about its centre until it is at most `room` wide.
pub fn fit_width(stage: &mut Stage<'_>, mob: Mob, room: f64) {
    if stage.get_width(mob) > room {
        let at = stage.get_center(mob);
        stage.set_width(mob, room, false);
        stage.move_to(mob, at, ORIGIN);
    }
}

/// Move several mobjects (each by its centre) to their places together,
/// one after another with `lag` of the move between starts, the whole
/// taking `rt` seconds. Each mover must be in the scene already.
pub fn fly_all(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], rt: f64, lag: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, to) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// One veil in the background's colour over the whole frame, detached and
/// at no opacity: fade it in to dim everything under it, in one animation
/// (an `animate()` on a map of ten thousand dots would copy them all).
pub fn veil(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(2.0 * FRAME_X_RADIUS)
        .height(2.0 * FRAME_Y_RADIUS)
        .style(flat(ground(), VEIL_OPACITY))
        .build()?;
    Ok(stage.arena_mut().add(r))
}

/// How far a veil dims what is under it.
pub const VEIL_OPACITY: f64 = 0.7;

/// Bring up what is already in the scene from the dark, `rt` seconds: a
/// veil in the background's colour over the whole frame fades away. It
/// works on pictures redrawn every frame (live percents), which a fade-in
/// cannot reach.
pub fn fade_from_black(stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
    let r = Rectangle::new()
        .width(2.0 * FRAME_X_RADIUS)
        .height(2.0 * FRAME_Y_RADIUS)
        .style(flat(ground(), 1.0))
        .build()?;
    let cover = stage.arena_mut().add(r);
    stage.add_to_scene(cover)?;
    stage.scene_mut().bring_to_front(&[cover])?;
    let a = fade_out(stage.arena_mut(), cover, ORIGIN, 1.0)?;
    stage.play(a.rt(rt))?;
    stage.remove_from_scene(cover);
    Ok(())
}

// ------------------------------------------------------------- the stage

/// The one caption slot under the stage (y = −3.35): detached text.
pub fn caption(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(stage, words, 26.0, Palette::dim(), v(0.0, CAPTION_Y))
}

/// The caption slot's height.
pub const CAPTION_Y: f64 = -3.35;

/// A small tag: a few words on a rounded back ("a made-up sea"), smaller
/// than a lower-third; it sits by the thing it names. One group, detached.
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

// ------------------------------------------------------------- seeded dots

/// A small seeded generator (splitmix64): the same dots on every machine.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// `n` places in a `w` by `h` rectangle centred on the origin, as a jittered
/// grid: no two closer than 0.4 of a grid step, and the same on every call.
/// A dot is one equally likely place, so the cloud must read as a scatter
/// with nothing in it to read (no rows, no clumps).
pub fn scatter(n: usize, w: f64, h: f64, seed: u64) -> Vec<(f64, f64)> {
    if n == 0 {
        return Vec::new();
    }
    let mut rng = Rng::new(seed);
    let cols = ((n as f64 * w / h).sqrt().ceil() as usize).max(1);
    let rows = n.div_ceil(cols).max(1);
    let (sx, sy) = (w / cols as f64, h / rows as f64);
    // the slots, in a seeded shuffle: the first n are used
    let mut slots: Vec<usize> = (0..cols * rows).collect();
    for i in (1..slots.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        slots.swap(i, j);
    }
    slots
        .into_iter()
        .take(n)
        .map(|s| {
            let (c, r) = (s % cols, s / cols);
            let jx = (rng.unit() - 0.5) * 0.6 * sx;
            let jy = (rng.unit() - 0.5) * 0.6 * sy;
            (
                -w / 2.0 + sx * (c as f64 + 0.5) + jx,
                -h / 2.0 + sy * (r as f64 + 0.5) + jy,
            )
        })
        .collect()
}

// -------------------------------------------------------------------- heat

/// How a cell's blue deepens with its share of the lit dots: linear up to
/// `full` (the share that is the deepest blue) and no deeper. One scale per
/// map, so two cells on a map are always told apart by their share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heat {
    pub full: f64,
}

/// The deepest a cell's blue goes: dots lit in the same blue must still
/// read over it.
pub const HEAT_MAX: f64 = 0.42;

impl Heat {
    /// The 1968 map: its hottest cell holds one dot in eight.
    pub const LATTICE: Heat = Heat { full: 0.125 };
    /// The made-up sea: its likeliest cell holds two dots in five, and a
    /// half would be the deepest.
    pub const SEA: Heat = Heat { full: 0.5 };

    /// The blue's opacity for a cell holding `share` (0 to 1) of the lit dots.
    pub fn opacity(&self, share: f64) -> f64 {
        (HEAT_MAX * share / self.full).clamp(0.0, HEAT_MAX)
    }
}

/// Every cell's blue for the lit counts `lit` (a cell with no dots has none).
pub fn heat_state(lit: &[u32], heat: Heat) -> Vec<f64> {
    let total: u32 = lit.iter().sum();
    lit.iter()
        .map(|&n| {
            if total == 0 {
                0.0
            } else {
                heat.opacity(f64::from(n) / f64::from(total))
            }
        })
        .collect()
}

// ------------------------------------------------------------ the strip

/// A strip sweeping a rectangle west to east: a teal wash grows behind a
/// narrow bar, both drawn from one tracker. Detached; bring in
/// [`Strip::mobs`] and tween `phase` from 0 to 1.
pub struct Strip {
    pub phase: Mob,
    pub wash: Mob,
    pub bar: Mob,
    /// Carries the updater: it must be in the scene.
    pub anchor: Mob,
}

impl Strip {
    /// A strip over the rectangle `(x0, y0, x1, y1)`, its bar `bar_w` wide.
    pub fn build(stage: &mut Stage<'_>, rect: (f64, f64, f64, f64), bar_w: f64) -> fmn::Result<Self> {
        let (x0, y0, x1, y1) = rect;
        let phase = stage.add_value_tracker(0.0);
        let quad = move |a: f64, b: f64| -> Vec<Vec3> { vec![v(a, y0), v(b, y0), v(b, y1), v(a, y1)] };
        let wash = stage
            .arena_mut()
            .add(Polygon::new(quad(x0, x0 + 0.01)).style(flat(search(), 0.32)).build());
        let bar = stage
            .arena_mut()
            .add(Polygon::new(quad(x0, x0 + bar_w)).style(flat(search(), 1.0)).build());
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let p = s.tracker_value(phase).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(p) {
                    return;
                }
                let x = x0 + (x1 - x0) * p;
                let _ = s.set_points(wash, Polygon::new(quad(x0, x.max(x0 + 0.01))).build().points());
                let lo = (x - bar_w / 2.0).clamp(x0, x1 - bar_w);
                let _ = s.set_points(bar, Polygon::new(quad(lo, lo + bar_w)).build().points());
                last = Some(p);
            },
            true,
        )?;
        Ok(Self {
            phase,
            wash,
            bar,
            anchor,
        })
    }

    pub fn mobs(&self) -> [Mob; 3] {
        [self.anchor, self.wash, self.bar]
    }
}

// ------------------------------------------------------ the 1968 lattice

/// The printed lattice: 20 columns (A to O in the middle) by 16 rows.
pub const LAT_COLS: usize = 20;
pub const LAT_ROWS: usize = 16;

/// One search of a cell of the lattice, in the order the scene makes them:
/// it puts out `seeing` of the dots still lit in that cell, in whole dots
/// (rounded half up).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Probe {
    pub row: usize,
    pub col: usize,
    pub seeing: Q,
}

/// Lit dots per cell (row-major, [`LAT_COLS`] wide) before any search and
/// after each of `probes`, and the dots each search puts out.
#[derive(Clone, Debug, PartialEq)]
pub struct Counts {
    pub lit: Vec<Vec<u32>>,
    pub out: Vec<u32>,
}

/// Work the searches of `probes` over `points` (the printed counts).
pub fn counts(points: &[[u16; LAT_COLS]; LAT_ROWS], probes: &[Probe]) -> Counts {
    let mut now: Vec<u32> = points.iter().flatten().map(|&n| u32::from(n)).collect();
    let mut lit = vec![now.clone()];
    let mut out = Vec::with_capacity(probes.len());
    for p in probes {
        let i = p.row * LAT_COLS + p.col;
        let gone = (Q::int(i128::from(now[i])) * p.seeing)
            .fixed(0)
            .parse::<u32>()
            .unwrap_or(0)
            .min(now[i]);
        now[i] -= gone;
        out.push(gone);
        lit.push(now.clone());
    }
    Counts { lit, out }
}

/// The map of 1968 as a picture: grid lines, a blue heat under every cell
/// that holds dots, the dots themselves (one group a cell, in blue), and for
/// each search a slate twin of the dots it puts out. Nothing here moves but
/// the heat; a search is one call, [`Lattice::search`].
pub struct Lattice {
    pub centre: Vec3,
    pub cell_w: f64,
    pub cell_h: f64,
    /// 21 vertical and 17 horizontal single strokes.
    pub lines: Vec<Mob>,
    /// One rectangle under each cell that holds dots, and the updater's anchor.
    pub heat: Vec<Mob>,
    pub anchor: Mob,
    /// One group of blue dots per cell that holds dots, the middle first.
    pub dots: Vec<Mob>,
    /// Search `k`'s dots in slate, detached until it comes back empty.
    pub twins: Vec<Mob>,
    pub counts: Counts,
    /// The searches the lattice was built for, in order.
    pub probes: Vec<Probe>,
    heats: Heat,
    phase: Mob,
    from: Rc<RefCell<Vec<f64>>>,
    to: Rc<RefCell<Vec<f64>>>,
    version: Rc<Cell<u64>>,
}

/// The dot's radius against the cell's height.
const DOT_R: f64 = 0.026;
/// A strip's bar against its cell's width.
const STRIP_BAR_SHARE: f64 = 0.22;

impl Lattice {
    /// The lattice centred at `centre` with cells `cell_h` tall (and 0.84 of
    /// that wide: a printed cell is a mile north to south and 0.84 of one
    /// east to west), holding the dots of [`y1968::POINTS`] and the
    /// searches `probes`. Detached.
    pub fn build(stage: &mut Stage<'_>, centre: Vec3, cell_h: f64, probes: &[Probe], heat: Heat) -> fmn::Result<Self> {
        let cell_w = cell_h * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64();
        let counts = counts(&y1968::POINTS, probes);
        let place = |row: usize, col: usize| -> Vec3 {
            v(
                centre[0] + (col as f64 - (LAT_COLS as f64 - 1.0) / 2.0) * cell_w,
                centre[1] + ((LAT_ROWS as f64 - 1.0) / 2.0 - row as f64) * cell_h,
            )
        };

        // the lines
        let style = faded_line(Palette::faint(), 1.2, 0.9);
        let (half_w, half_h) = (LAT_COLS as f64 * cell_w / 2.0, LAT_ROWS as f64 * cell_h / 2.0);
        let mut lines = Vec::new();
        for c in 0..=LAT_COLS {
            let x = centre[0] - half_w + c as f64 * cell_w;
            let l = polyline(vec![v(x, centre[1] - half_h), v(x, centre[1] + half_h)]).style(style);
            lines.push(stage.arena_mut().add(l.build()));
        }
        for r in 0..=LAT_ROWS {
            let y = centre[1] - half_h + r as f64 * cell_h;
            let l = polyline(vec![v(centre[0] - half_w, y), v(centre[0] + half_w, y)]).style(style);
            lines.push(stage.arena_mut().add(l.build()));
        }

        // the heat under every cell that holds dots
        let before = heat_state(&counts.lit[0], heat);
        let mut rects: Vec<(Mob, usize)> = Vec::new();
        for (i, &n) in counts.lit[0].iter().enumerate() {
            if n == 0 {
                continue;
            }
            let r = Rectangle::new()
                .width(cell_w)
                .height(cell_h)
                .style(flat(map(), before[i]))
                .build()?
                .moved_to(place(i / LAT_COLS, i % LAT_COLS));
            rects.push((stage.arena_mut().add(r), i));
        }
        let phase = stage.add_value_tracker(1.0);
        let from = Rc::new(RefCell::new(before.clone()));
        let to = Rc::new(RefCell::new(before));
        let version = Rc::new(Cell::new(0u64));
        let anchor = stage.arena_mut().add(VMobject::new());
        {
            let (from, to, version) = (Rc::clone(&from), Rc::clone(&to), Rc::clone(&version));
            let members = rects.clone();
            let mut last: Option<(f64, u64)> = None;
            stage.add_updater(
                anchor,
                move |s, _| {
                    let t = s.tracker_value(phase).unwrap_or(1.0);
                    let key = (t, version.get());
                    if last == Some(key) {
                        return;
                    }
                    let (a, b) = (from.borrow(), to.borrow());
                    for &(m, i) in &members {
                        s.set_fill(m, Some(map()), Some(a[i] + (b[i] - a[i]) * t), None, false);
                    }
                    last = Some(key);
                },
                true,
            )?;
        }

        // the dots: each cell's, then each search's twin of the first of them
        let mut by_cell: Vec<(usize, Mob)> = Vec::new();
        let mut twins: Vec<Mob> = Vec::with_capacity(probes.len());
        // dots of a cell already put out by earlier searches
        let mut taken = vec![0usize; LAT_COLS * LAT_ROWS];
        let radius = DOT_R * cell_h;
        let mut spots: Vec<Option<Vec<(f64, f64)>>> = vec![None; LAT_COLS * LAT_ROWS];
        for (i, &n) in counts.lit[0].iter().enumerate() {
            if n == 0 {
                continue;
            }
            let (row, col) = (i / LAT_COLS, i % LAT_COLS);
            let at = place(row, col);
            let pts = scatter(n as usize, 0.86 * cell_w, 0.86 * cell_h, 0x1968_0000 + i as u64);
            let dots = pts.iter().map(|&(dx, dy)| {
                Dot::new()
                    .point(v(at[0] + dx, at[1] + dy))
                    .radius(radius)
                    .style(flat(map(), 1.0))
                    .build()
            });
            by_cell.push((i, stage.arena_mut().add(v_group(dots))));
            spots[i] = Some(pts);
        }
        for (k, p) in probes.iter().enumerate() {
            let i = p.row * LAT_COLS + p.col;
            let at = place(p.row, p.col);
            let gone = counts.out[k] as usize;
            let pts = spots[i].as_deref().unwrap_or(&[]);
            let take = &pts[taken[i].min(pts.len())..(taken[i] + gone).min(pts.len())];
            taken[i] += gone;
            let twin = take.iter().map(|&(dx, dy)| {
                Dot::new()
                    .point(v(at[0] + dx, at[1] + dy))
                    .radius(radius * 1.12)
                    .style(flat(out(), 1.0))
                    .build()
            });
            twins.push(stage.arena_mut().add(v_group(twin)));
        }
        // the middle first, so a lagged reveal runs outward
        by_cell.sort_by(|a, b| {
            let d = |i: usize| {
                let (r, c) = (i / LAT_COLS, i % LAT_COLS);
                let p = place(r, c);
                (p[0] - centre[0]).hypot(p[1] - centre[1])
            };
            d(a.0).total_cmp(&d(b.0))
        });
        Ok(Self {
            centre,
            cell_w,
            cell_h,
            lines,
            heat: rects.into_iter().map(|(m, _)| m).collect(),
            anchor,
            dots: by_cell.into_iter().map(|(_, m)| m).collect(),
            twins,
            counts,
            probes: probes.to_vec(),
            heats: heat,
            phase,
            from,
            to,
            version,
        })
    }

    /// The scene rectangle of a cell: (x0, y0, x1, y1).
    pub fn cell_rect(&self, row: usize, col: usize) -> (f64, f64, f64, f64) {
        let x = self.centre[0] + (col as f64 - (LAT_COLS as f64 - 1.0) / 2.0) * self.cell_w;
        let y = self.centre[1] + ((LAT_ROWS as f64 - 1.0) / 2.0 - row as f64) * self.cell_h;
        (
            x - self.cell_w / 2.0,
            y - self.cell_h / 2.0,
            x + self.cell_w / 2.0,
            y + self.cell_h / 2.0,
        )
    }

    /// The whole picture's width and height.
    pub fn size(&self) -> (f64, f64) {
        (LAT_COLS as f64 * self.cell_w, LAT_ROWS as f64 * self.cell_h)
    }

    /// Everything in the picture but the twins, back to front: lines, heat
    /// (with its anchor), dots.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = self.lines.clone();
        out.push(self.anchor);
        out.extend(&self.heat);
        out.extend(&self.dots);
        out
    }

    /// The printed number of every cell that holds dots, centred in its
    /// cell, one text each, detached (04_4's "the map they printed"), the
    /// middle first so a lagged reveal ripples outward like the dots. A
    /// cell is a third of a unit wide, so each is shrunk to fit it.
    pub fn numbers(&self, kit: &Kit, stage: &mut Stage<'_>, size: f64) -> fmn::Result<Vec<Mob>> {
        let mut placed: Vec<(f64, Mob)> = Vec::new();
        for (i, &n) in self.counts.lit[0].iter().enumerate() {
            if n == 0 {
                continue;
            }
            let (x0, y0, x1, y1) = self.cell_rect(i / LAT_COLS, i % LAT_COLS);
            let at = v((x0 + x1) / 2.0, (y0 + y1) / 2.0);
            let m = kit.text(stage, &n.to_string(), size, Palette::ink(), at)?;
            fit_width(stage, m, 0.88 * self.cell_w);
            placed.push(((at[0] - self.centre[0]).hypot(at[1] - self.centre[1]), m));
        }
        placed.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(placed.into_iter().map(|(_, m)| m).collect())
    }

    /// Search number `k` of the plan, the whole move: the search's mark for
    /// the score, the strip crossing its cell in `cross_s` seconds (the
    /// dots untouched), then, in `move_s` seconds and in the same instant,
    /// the dots it would have seen going out (their slate twins fade in
    /// over them), every cell's blue recounted from the dots still lit, and
    /// the strip leaving.
    pub fn search(&self, kit: &Kit, stage: &mut Stage<'_>, k: usize, cross_s: f64, move_s: f64) -> fmn::Result<()> {
        kit.mark(stage, sound::SEARCH_MARK);
        self.search_silent(stage, k, cross_s, move_s)
    }

    /// [`Lattice::search`] without the search's mark: the searches after the
    /// first of a run that sounds once (08_4's four: the search sound has
    /// one meaning and is heard on the first move only).
    pub fn search_silent(&self, stage: &mut Stage<'_>, k: usize, cross_s: f64, move_s: f64) -> fmn::Result<()> {
        let p = *self.probes.get(k).ok_or_else(|| {
            fail(format!(
                "the lattice plans {} searches, not {}",
                self.probes.len(),
                k + 1
            ))
        })?;
        let strip = Strip::build(stage, self.cell_rect(p.row, p.col), STRIP_BAR_SHARE * self.cell_w)?;
        for m in strip.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(strip.phase, 1.0, cross_s)?)?;
        let mut together: Vec<Box<dyn Animation>> = vec![self.heat_move(stage, k + 1, move_s)?];
        let twin = fade_in(stage.arena_mut(), self.twins[k], ORIGIN, 1.0)?;
        together.push(stage.prepare(twin.rt(move_s))?);
        for m in [strip.wash, strip.bar] {
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            together.push(stage.prepare(gone.rt(0.5))?);
        }
        stage.play_prepared(together)?;
        Ok(())
    }

    /// Move every cell's blue from where it is to its value after `k`
    /// searches, `rt` seconds, as the dots stand (not an animation of the
    /// dots: the heat is recounted from them).
    pub fn heat_to(&self, stage: &mut Stage<'_>, k: usize, rt: f64) -> fmn::Result<()> {
        let a = self.heat_move(stage, k, rt)?;
        stage.play_prepared(vec![a])?;
        Ok(())
    }

    /// As [`Lattice::heat_to`], returned as a prepared animation, so it
    /// runs in the same move as the dots going out.
    pub fn heat_move(&self, stage: &mut Stage<'_>, k: usize, rt: f64) -> fmn::Result<Box<dyn Animation>> {
        let lit = self
            .counts
            .lit
            .get(k)
            .ok_or_else(|| fail(format!("the lattice has no state after {k} searches")))?
            .clone();
        self.heat_toward(stage, &lit, rt)
    }

    /// Move every cell's blue to what `lit` (row-major lit dots per cell,
    /// [`LAT_COLS`] wide) says, for a state the plan does not name. Only
    /// cells that held dots at the start have a blue to move.
    pub fn heat_toward(&self, stage: &mut Stage<'_>, lit: &[u32], rt: f64) -> fmn::Result<Box<dyn Animation>> {
        if lit.len() != LAT_COLS * LAT_ROWS {
            return Err(fail(format!(
                "a lattice state has {} cells, not {}",
                lit.len(),
                LAT_COLS * LAT_ROWS
            )));
        }
        let want = heat_state(lit, self.heats);
        let t = stage.tracker_value(self.phase).unwrap_or(1.0);
        {
            let now: Vec<f64> = {
                let (a, b) = (self.from.borrow(), self.to.borrow());
                a.iter().zip(b.iter()).map(|(&x, &y)| x + (y - x) * t).collect()
            };
            *self.from.borrow_mut() = now;
            *self.to.borrow_mut() = want;
        }
        self.version.set(self.version.get() + 1);
        stage
            .arena_mut()
            .set_tracker_value(self.phase, 0.0)
            .map_err(|e| fail(format!("the lattice's heat tracker: {e:?}")))?;
        stage.prepare(tween(self.phase, 1.0, rt)?)
    }
}

// -------------------------------------------------------- the "your turn"

/// The frame's half width and half height: the stage inset a little.
pub const QUIZ_FRAME: (f64, f64) = (6.9, 3.8);
/// The ring's centre and radius: top right, inside the frame.
pub const QUIZ_RING_AT: Vec3 = [6.1, 3.05, 0.0];
pub const QUIZ_RING_R: f64 = 0.5;
/// Where the question's lines and the note sit in the caption band.
pub const QUIZ_LINES_Y: [f64; 2] = [-2.85, -3.25];
pub const QUIZ_NOTE_Y: f64 = -3.6;
/// How far a line's text reaches from its centre (26 pt and 20 pt).
const QUIZ_LINE_REACH: f64 = 0.2;
const QUIZ_NOTE_REACH: f64 = 0.15;
/// How wide a character of the text face runs, per point of type size, in
/// stage units: the face measures 20.6 px a character at 34 pt on the glyph
/// pages' 1,920 px, 14.22-unit frame, which is 0.0045 of a unit a point.
pub const CHAR_UNITS_PER_POINT: f64 = 0.0045;
/// A question's type size, and how many characters of it make a line 9.8
/// units wide (84 at 26 pt, of the frame's 13.8).
const QUIZ_PT: f64 = 26.0;
pub const QUIZ_FULL_CHARS: usize = 84;
/// The most a question line may run to: a longer line is set smaller so it
/// is never wider than a full one (the longest question, 09_5, is 179
/// characters: two lines of 90, at 24 pt).
pub const QUIZ_LINE_CHARS: usize = 100;
const PAUSE_NOTE: &str = "pause for more time";

/// The type size for a question set in `lines`: full size, or smaller so
/// that its longest line is no wider than [`QUIZ_FULL_CHARS`] at full size.
pub fn question_pt(lines: &[String]) -> f64 {
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    if longest <= QUIZ_FULL_CHARS {
        QUIZ_PT
    } else {
        QUIZ_PT * QUIZ_FULL_CHARS as f64 / longest as f64
    }
}

/// What a line break costs when it falls inside a sentence, against how
/// unbalanced the two lines are: a sentence's end wins unless that leaves
/// the lines more than this many characters apart.
const MID_SENTENCE_COST: usize = 60;
/// What a break costs that strands a one-letter word ("E | five") at the end
/// of a line.
const STRANDED_COST: usize = 200;

/// `text` as one line if it fits `max_chars`, otherwise as two lines broken
/// where it reads best: at the end of a sentence if that keeps the lines
/// within [`MID_SENTENCE_COST`] of balanced, otherwise at the most balanced
/// word, and never after a one-letter word. Every word is kept (a text too
/// long for two lines wraps on to a third rather than losing its end, which
/// is what the kit's `wrap` does to it).
pub fn balanced_lines(text: &str, max_chars: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let whole = words.join(" ");
    if whole.chars().count() <= max_chars {
        return vec![whole];
    }
    let mut best: Option<(usize, usize)> = None;
    for k in 1..words.len() {
        let (a, b) = (words[..k].join(" "), words[k..].join(" "));
        let (la, lb) = (a.chars().count(), b.chars().count());
        if la > max_chars || lb > max_chars {
            continue;
        }
        let ends_sentence = a.ends_with(['.', '?', '!']);
        let mut cost = la.abs_diff(lb);
        if !ends_sentence {
            cost += MID_SENTENCE_COST;
            if words[k - 1].chars().count() == 1 {
                cost += STRANDED_COST;
            }
        }
        if best.is_none_or(|(c, _)| cost < c) {
            best = Some((cost, k));
        }
    }
    match best {
        Some((_, k)) => vec![words[..k].join(" "), words[k..].join(" ")],
        None => wrap(&whole, max_chars, words.len().max(1)),
    }
}

/// `lines` as separate texts one under another, centred on `x`, the first
/// line's middle at `top` and `step` between them: one group, detached.
#[allow(clippy::too_many_arguments)]
pub fn text_lines(
    kit: &Kit,
    stage: &mut Stage<'_>,
    lines: &[String],
    size: f64,
    color: Srgb,
    x: f64,
    top: f64,
    step: f64,
) -> fmn::Result<Mob> {
    let mut texts = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        texts.push(kit.text(stage, l, size, color, v(x, top - step * i as f64))?);
    }
    group(stage, &texts)
}

/// A lower-third that wraps the way [`balanced_lines`] does (the kit's breaks
/// a long quotation at its last word: "… a subjective / one."): the `label`
/// in the hero role at the left, the text beside it in one or two balanced
/// lines, on a panel at the foot of the frame. One group, detached.
pub fn quote_card(kit: &Kit, stage: &mut Stage<'_>, label: &str, text: &str) -> fmn::Result<Mob> {
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let left = -w / 2.0 + 0.25;
    let tag = kit.text_left(stage, label, 26.0, Palette::hero(), v(left, 0.0))?;
    let indent = left + stage.get_width(tag) + 0.2;
    let room = w / 2.0 - 0.25 - indent;
    let chars = (room / (CHAR_UNITS_PER_POINT * 26.0)).floor() as usize;
    let lines = balanced_lines(text, chars);
    if lines.len() > 3 {
        return Err(fail(format!("the card for {label:?} needs {} lines", lines.len())));
    }
    let line_h = 0.36;
    let h = 0.3 + line_h * lines.len() as f64;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(centre);
    let back = stage.arena_mut().add(back);
    let top = centre[1] + h / 2.0 - 0.15 - line_h / 2.0;
    stage.move_to(tag, v(left, top), LEFT);
    let mut all = vec![back, tag];
    for (i, l) in lines.iter().enumerate() {
        all.push(kit.text_left(stage, l, 26.0, Palette::ink(), v(indent, top - line_h * i as f64))?);
    }
    group(stage, &all)
}

/// What is left of a circle of radius `r` at `c`, drained clockwise from
/// the top.
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

/// A ring of radius `r` at `at`, `share` of the way round (clockwise from
/// the top; 1 is a whole ring), as a detached stroke: a third of a dot is
/// shown as a third of a ring (07_4's "thirteen and a third").
pub fn arc_ring(stage: &mut Stage<'_>, at: Vec3, r: f64, share: f64, color: Srgb, width: f64) -> Mob {
    // ring_points runs counter-clockwise from the top; mirror it so a
    // partial ring grows clockwise like the quiz's drains
    let pts: Vec<Vec3> = ring_points(at, r, share)
        .into_iter()
        .map(|p| v(2.0 * at[0] - p[0], p[1]))
        .collect();
    stage
        .arena_mut()
        .add(polyline(pts).style(line_style(color, width)).build())
}

/// A "your turn" (storyboard, "Two kinds of question"): the stage stays, a
/// violet frame comes on around the live picture, a ring at top right
/// drains evenly over the quiz's seconds with the seconds left in its
/// middle, and the question as spoken sets in the caption band with "pause
/// for more time". The narration is silent and the music alone. At zero the
/// ring goes; the chapter lands the answer in its picture and lifts the
/// frame ([`QuizFrame::lift`]) in the same move.
pub struct QuizFrame {
    pub quiz: &'static Quiz,
    pub edges: [Mob; 4],
    pub track: Mob,
    pub ring: Mob,
    pub digits: Vec<Mob>,
    pub note: Mob,
    pub question: Vec<Mob>,
    /// Seconds left.
    pub left: Mob,
    /// 0 to 1: how much of the ring and its digit shows.
    pub shown: Mob,
    /// Carries the updater.
    pub anchor: Mob,
}

impl QuizFrame {
    /// The frame for the quiz asked on line `id`. Detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Self> {
        let quiz = QUIZZES
            .iter()
            .find(|q| q.line == id)
            .ok_or_else(|| fail(format!("{id} is not a quiz of Video H")))?;
        let script = Script::parse(crate::SCRIPT).map_err(fail)?;
        let words = script
            .text(id)
            .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?;
        let (fx, fy) = QUIZ_FRAME;
        let sides = [
            [v(-fx, fy), v(fx, fy)],
            [v(-fx, -fy), v(fx, -fy)],
            [v(-fx, -fy), v(-fx, fy)],
            [v(fx, -fy), v(fx, fy)],
        ];
        let edges: [Mob; 4] = std::array::from_fn(|i| {
            stage
                .arena_mut()
                .add(polyline(sides[i].to_vec()).style(line_style(quiz_role(), 5.0)).build())
        });

        let lines = balanced_lines(words, QUIZ_LINE_CHARS);
        if lines.len() > QUIZ_LINES_Y.len() {
            return Err(fail(format!(
                "the question of {id} needs {} lines, the caption band has {}",
                lines.len(),
                QUIZ_LINES_Y.len()
            )));
        }
        let pt = question_pt(&lines);
        let mut question = Vec::new();
        for (l, &y) in lines.iter().zip(&QUIZ_LINES_Y) {
            question.push(kit.text(stage, l, pt, Palette::ink(), v(0.0, y))?);
        }
        let note = kit.text(stage, PAUSE_NOTE, 20.0, Palette::dim(), v(0.0, QUIZ_NOTE_Y))?;

        let n = f64::from(quiz.seconds);
        let at = QUIZ_RING_AT;
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(QUIZ_RING_R)
                .arc_center(at)
                .style(faded_line(Palette::faint(), 7.0, 1.0))
                .build(),
        );
        let ring = stage.arena_mut().add(
            polyline(ring_points(at, QUIZ_RING_R, 1.0))
                .style(faded_line(quiz_role(), 8.0, 0.0))
                .build(),
        );
        let mut digits = Vec::new();
        for second in 1..=quiz.seconds {
            let d = kit.text(stage, &second.to_string(), 44.0, Palette::ink(), at)?;
            stage.set_opacity(d, 0.0, true);
            digits.push(d);
        }
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
                let drained = polyline(ring_points(at, QUIZ_RING_R, now.0 / n)).build();
                let _ = s.set_points(ring, drained.points());
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
            edges,
            track,
            ring,
            digits,
            note,
            question,
            left,
            shown,
            anchor,
        })
    }

    /// Bring the frame, the question and the full ring in, `rt` seconds. Call
    /// it as the question's last sentence begins: it must be readable by
    /// the narrator's last word of it.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        let mut ins: Vec<Mob> = self.edges.to_vec();
        ins.push(self.track);
        ins.push(self.note);
        ins.extend(&self.question);
        fade_in_all(stage, &ins, rt)?;
        stage.add_to_scene(self.ring)?;
        for &d in &self.digits {
            stage.add_to_scene(d)?;
        }
        stage.play(tween(self.shown, 1.0, 0.25)?)?;
        Ok(())
    }

    /// The countdown: the narration silent, one mallet note a second, the
    /// ring draining and its digit counting 6, 5, ..., 1; at zero the ring
    /// goes (a quarter second). The listen window runs [`sound::LAND_S`]
    /// past zero for the last note, so the answer the chapter lands next
    /// fills the rest of it (about `LAND_S` minus the quarter second) and
    /// the next line cannot start before it ends.
    pub fn count(&self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = f64::from(self.quiz.seconds);
        let wav = sound::countdown(self.quiz.seconds)?;
        let label = format!("{}:{}", kit.chapter, self.quiz.line);
        kit.sonify(stage, n + sound::LAND_S, &label, &wav)?;
        stage.play(tween_linear(self.left, 0.0, n)?)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        anims.push(stage.prepare(tween(self.shown, 0.0, 0.25)?)?);
        let track_gone = fade_out(stage.arena_mut(), self.track, ORIGIN, 1.0)?;
        anims.push(stage.prepare(track_gone.rt(0.25))?);
        stage.play_prepared(anims)?;
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.ring);
        for &d in &self.digits {
            stage.remove_from_scene(d);
        }
        Ok(())
    }

    /// The whole beat for the quiz asked on `line` (already said): the
    /// frame comes in as the question's last sentence begins, the ring
    /// counts down with the narration silent, and the frame is returned
    /// ringless and still up, for the chapter to land the answer in its
    /// picture and play [`QuizFrame::lift`] in the same move.
    pub fn your_turn(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<QuizFrame> {
        let frame = QuizFrame::build(kit, stage, &line.id)?;
        line.wait_last(stage)?;
        frame.show(stage, 0.5)?;
        frame.count(kit, stage)?;
        Ok(frame)
    }

    /// The frame and the question lifting, as prepared animations the
    /// chapter plays with the answer's own, so the answer lands as the
    /// frame lifts.
    pub fn lift(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let mut gone: Vec<Mob> = self.edges.to_vec();
        gone.push(self.note);
        gone.extend(&self.question);
        for m in gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        Ok(anims)
    }
}

fn quiz_role() -> Srgb {
    quiz()
}

/// A question to carry (storyboard, "Two kinds of question"): the line's
/// words as one or two lines in the caption band, no ring, one group,
/// detached. Bring it in as the question's last sentence begins, then
/// [`carry_pause`], then fade it with what follows.
pub fn carry_question(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Mob> {
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let words = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?;
    let lines = balanced_lines(words, QUIZ_LINE_CHARS);
    if lines.len() > QUIZ_LINES_Y.len() {
        return Err(fail(format!(
            "the question of {id} needs {} lines, the caption band has {}",
            lines.len(),
            QUIZ_LINES_Y.len()
        )));
    }
    let pt = question_pt(&lines);
    let mut texts = Vec::new();
    for (l, &y) in lines.iter().zip(&QUIZ_LINES_Y) {
        texts.push(kit.text(stage, l, pt, Palette::ink(), v(0.0, y))?);
    }
    group(stage, &texts)
}

// ------------------------------------------------------ the made-up sea

/// The sea's four cells, A and B above C and D: centres, in the left half of
/// the stage (the right half is for story cards, the table and the sums).
pub const SEA_CENTRES: [Vec3; 4] = [[-5.1, 2.4, 0.0], [-1.8, 2.4, 0.0], [-5.1, -0.2, 0.0], [-1.8, -0.2, 0.0]];
/// A cell's width and height.
pub const SEA_CELL: (f64, f64) = (2.8, 1.6);
/// A dot's radius in the sea, and a ring's.
pub const SEA_DOT_R: f64 = 0.085;
pub const SEA_RING_R: f64 = 0.17;
/// Rows of the small sum under a cell (blue, teal, gold), top of the first
/// below the cell's bottom edge.
const SLOT_FIRST: f64 = 0.22;
const SLOT_STEP: f64 = 0.27;

/// The dots of a hundred in the made-up sea by cell and story, as the two
/// stories make them: story one (weight 60) puts 30, 20, 10, 0; story two
/// (weight 40) 10 in each (`model::sea`).
pub fn sea_split() -> [[usize; 4]; 2] {
    use crate::model::sea;
    let dots = |s: &sea::Story| -> [usize; 4] { sea::story_dots(s).map(|q| q.fixed(0).parse().unwrap_or(0)) };
    let stories = sea::stories();
    [dots(&stories[0]), dots(&stories[1])]
}

/// The made-up sea's cells: a blue heat under each, a faint border, its
/// letter, the tag, and the canyon of A. The heat is recounted every frame
/// from `lit`, one tracker of lit dots per cell.
pub struct Sea {
    pub blocks: [Mob; 4],
    pub borders: [Mob; 4],
    pub letters: [Mob; 4],
    pub tag: Mob,
    /// Four smooth contour lines in A (05_2), as one group, detached.
    pub canyon: Mob,
    /// Lit dots per cell: tween them to move the heat.
    pub lit: [Mob; 4],
    /// Carries the updater: it must be in the scene.
    pub anchor: Mob,
}

/// A cell's rectangle: (x0, y0, x1, y1).
pub fn sea_rect(cell: usize) -> (f64, f64, f64, f64) {
    let c = SEA_CENTRES[cell];
    (
        c[0] - SEA_CELL.0 / 2.0,
        c[1] - SEA_CELL.1 / 2.0,
        c[0] + SEA_CELL.0 / 2.0,
        c[1] + SEA_CELL.1 / 2.0,
    )
}

/// Where row `k` (0 blue, 1 teal, 2 gold) of the small sum under a cell is
/// centred.
pub fn sea_slot(cell: usize, k: usize) -> Vec3 {
    let (_, y0, _, _) = sea_rect(cell);
    v(SEA_CENTRES[cell][0], y0 - SLOT_FIRST - SLOT_STEP * k as f64)
}

/// The gold slot's type size, and how far a mixed fraction ("13⅓%") stands
/// below that slot's centre. Stacked it is two lines tall, taller than the
/// slots' pitch: centred on its slot at 30 pt its numerator touched the teal
/// "2 in 3" above it (the narrated draft's 05_6, 06 and 07_3). Every gold
/// number is set at [`GOLD_PT`], so a row of them reads alike.
pub const GOLD_PT: f64 = 26.0;
pub const TALL_DROP: f64 = 0.12;
/// Half the height of a mixed fraction at [`GOLD_PT`].
pub const TALL_HALF_H: f64 = 0.25;

/// A chance of a find as TeX (`percent_tex`, `mixed_tex`) in the gold slot
/// under `cell`, detached. A stacked mixed fraction stands [`TALL_DROP`]
/// lower so that it clears the teal row above it and, in 06, the row of dots
/// below; there is no room for one under the top row's cells (the bottom
/// row's cells start 0.24 under their slot), so asking for one there fails
/// the build: set it as text.
pub fn gold_tex(kit: &Kit, stage: &mut Stage<'_>, tex: &str, color: Srgb, cell: usize) -> fmn::Result<Mob> {
    let mut at = sea_slot(cell, 2);
    if tex.contains("\\frac") {
        if cell < 2 {
            return Err(fail(format!(
                "a stacked fraction ({tex}) does not fit under cell {cell}: the cell below starts 0.24 under its slot"
            )));
        }
        at[1] -= TALL_DROP;
    }
    kit.tex(stage, tex, GOLD_PT, color, &[], at)
}

/// The tag every appearance of the sea carries.
pub const SEA_TAG: &str = "a made-up sea";

impl Sea {
    /// The sea with `lit` dots in each cell (the heat follows their shares).
    /// Detached.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, lit: [u32; 4]) -> fmn::Result<Self> {
        let (w, h) = SEA_CELL;
        let share = |l: [f64; 4], i: usize| -> f64 {
            let t: f64 = l.iter().sum();
            if t > 0.0 { l[i] / t } else { 0.0 }
        };
        let start = lit.map(f64::from);
        let mut blocks = Vec::new();
        let mut borders = Vec::new();
        for (i, c) in SEA_CENTRES.iter().enumerate() {
            let fill = Rectangle::new()
                .width(w)
                .height(h)
                .style(flat(map(), Heat::SEA.opacity(share(start, i))))
                .build()?
                .moved_to(*c);
            blocks.push(stage.arena_mut().add(fill));
            let edge = Rectangle::new()
                .width(w)
                .height(h)
                .style(faded_line(Palette::faint(), 2.0, 1.0))
                .build()?
                .moved_to(*c);
            borders.push(stage.arena_mut().add(edge));
        }
        let mut letters = Vec::new();
        for (i, name) in crate::model::sea::CELLS.iter().enumerate() {
            let c = SEA_CENTRES[i];
            letters.push(kit.text(
                stage,
                name,
                26.0,
                Palette::dim(),
                v(c[0] - w / 2.0 + 0.28, c[1] + h / 2.0 - 0.28),
            )?);
        }
        let top = SEA_CENTRES[0][1] + h / 2.0 + 0.32;
        let mid = (SEA_CENTRES[0][0] + SEA_CENTRES[1][0]) / 2.0;
        let tag = tag(kit, stage, SEA_TAG, v(mid, top), 22.0)?;

        // the canyon: four nested smooth contours, a little off the middle
        let a = SEA_CENTRES[0];
        let mut rings = Vec::new();
        for k in 1..=4 {
            let s = f64::from(k) / 4.0;
            let (rx, ry) = (0.44 * w * s, 0.36 * h * s);
            let pts: Vec<Vec3> = (0..=48)
                .map(|j| {
                    let t = TAU * f64::from(j) / 48.0;
                    // a gentle bend so it reads as terrain, not a target
                    v(
                        a[0] + 0.1 * w * (1.0 - s) + rx * t.cos(),
                        a[1] - 0.05 * h + ry * t.sin() * (1.0 + 0.12 * (2.0 * t).cos()),
                    )
                })
                .collect();
            rings.push(
                stage
                    .arena_mut()
                    .add(polyline(pts).style(faded_line(Palette::dim(), 2.0, 0.7)).build()),
            );
        }
        let canyon = group(stage, &rings)?;

        let lit_t: [Mob; 4] = std::array::from_fn(|i| stage.add_value_tracker(start[i]));
        let anchor = stage.arena_mut().add(VMobject::new());
        let fills = [blocks[0], blocks[1], blocks[2], blocks[3]];
        let mut last: Option<[f64; 4]> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let now: [f64; 4] = std::array::from_fn(|i| s.tracker_value(lit_t[i]).unwrap_or(0.0));
                if last == Some(now) {
                    return;
                }
                for (i, &m) in fills.iter().enumerate() {
                    s.set_fill(m, Some(map()), Some(Heat::SEA.opacity(share(now, i))), None, false);
                }
                last = Some(now);
            },
            true,
        )?;
        Ok(Self {
            blocks: fills,
            borders: [borders[0], borders[1], borders[2], borders[3]],
            letters: [letters[0], letters[1], letters[2], letters[3]],
            tag,
            canyon,
            lit: lit_t,
            anchor,
        })
    }

    /// Everything but the tag and the canyon, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.blocks);
        out.extend(self.borders);
        out.extend(self.letters);
        out
    }

    /// A cell's centre.
    pub fn centre(&self, cell: usize) -> Vec3 {
        SEA_CENTRES[cell]
    }

    /// A cell's rectangle: (x0, y0, x1, y1).
    pub fn rect(&self, cell: usize) -> (f64, f64, f64, f64) {
        sea_rect(cell)
    }

    /// Where row `k` (0 blue, 1 teal, 2 gold) of the small sum under a cell
    /// is centred.
    pub fn slot(&self, cell: usize, k: usize) -> Vec3 {
        sea_slot(cell, k)
    }

    /// A live percent of the lit dots in `cell` (50, 12.5, 25, ...), its
    /// centre at `at`: one decimal only where the value needs it.
    pub fn percent_readout(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        cell: usize,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> Mob {
        let lit = self.lit;
        let book = Rc::clone(&kit.book);
        stage.always_redraw(move |s| {
            let now: [f64; 4] = std::array::from_fn(|i| s.tracker_value(lit[i]).unwrap_or(0.0));
            let total: f64 = now.iter().sum();
            let value = if total > 0.0 { 100.0 * now[cell] / total } else { 0.0 };
            let places = usize::from(((value * 10.0).round() as i64) % 10 != 0);
            let scale = if places == 1 { 10.0 } else { 1.0 };
            let value = (value * scale).round() / scale;
            let d = DecimalNumber::new(value)
                .num_decimal_places(places)
                .unit("%")
                .font_size(size)
                .color(color)
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, at, ORIGIN);
            m
        })
    }
}

/// One of the sea's hundred dots: where it settles, and its three looks
/// (blue lit, its slate twin once a search has put it out, the teal ring a
/// search would see it by), the twin and the ring detached until needed.
#[derive(Clone, Copy, Debug)]
pub struct SeaDot {
    pub cell: usize,
    /// 0 for the story that kept to its route, 1 for the one that strayed.
    pub story: usize,
    pub at: Vec3,
    pub dot: Mob,
    pub twin: Mob,
    pub ring: Mob,
}

/// The sea's dots as `split` makes them (`[story][cell]`), settled in their
/// cells: a cell's first dots (in its seeded order) are story one's, the
/// rest story two's. In reading order by cell, then by that order.
pub fn sea_dots(stage: &mut Stage<'_>, split: [[usize; 4]; 2]) -> Vec<SeaDot> {
    let mut out = Vec::new();
    for cell in 0..4 {
        let n = split[0][cell] + split[1][cell];
        let c = SEA_CENTRES[cell];
        let pts = scatter(n, 0.84 * SEA_CELL.0, 0.74 * SEA_CELL.1, 0x5EA0 + cell as u64);
        for (j, &(dx, dy)) in pts.iter().enumerate() {
            let at = v(c[0] + dx, c[1] + dy - 0.06);
            let mk = |stage: &mut Stage<'_>, d: VMobject| stage.arena_mut().add(d);
            let dot = mk(
                stage,
                Dot::new().point(at).radius(SEA_DOT_R).style(flat(map(), 1.0)).build(),
            );
            let twin = mk(
                stage,
                Dot::new()
                    .point(at)
                    .radius(SEA_DOT_R * 1.1)
                    .style(flat(out_color(), 1.0))
                    .build(),
            );
            let ring = mk(
                stage,
                Circle::new()
                    .radius(SEA_RING_R)
                    .style(line_style(search(), 2.5))
                    .build()
                    .moved_to(at),
            );
            out.push(SeaDot {
                cell,
                story: usize::from(j >= split[0][cell]),
                at,
                dot,
                twin,
                ring,
            });
        }
    }
    out
}

fn out_color() -> Srgb {
    out()
}

/// The dots of `cell`, in the order they go out.
pub fn in_cell(dots: &[SeaDot], cell: usize) -> Vec<SeaDot> {
    dots.iter().copied().filter(|d| d.cell == cell).collect()
}

/// A chance as TeX: `n/d` as `\frac{n}{d}`, a whole number as itself. A
/// third is a real fraction on screen, never a font's ⅓ (which may be
/// missing from the face).
pub fn frac_tex(q: Q) -> String {
    let s = q.to_string();
    match s.split_once('/') {
        Some((n, d)) => format!("\\frac{{{n}}}{{{d}}}"),
        None => s,
    }
}

/// A percent as TeX for a chance whose percent is not always whole: 40, 12.5
/// as decimals where they end, and a third of a percent as a mixed number
/// (13⅓ is `13\frac{1}{3}`), then the sign.
pub fn percent_tex(q: Q) -> String {
    let hundred = q * Q::int(100);
    let s = hundred.to_string();
    match s.split_once('/') {
        None => format!("{s}\\%"),
        Some((n, d)) => {
            let (n, d): (i128, i128) = (n.parse().unwrap_or(0), d.parse().unwrap_or(1));
            // a decimal that ends: the fewest places k with d dividing 10^k
            let places = (1..=6u32).find(|&k| 10i128.pow(k) % d == 0);
            match places {
                Some(k) => format!("{}\\%", hundred.fixed(k)),
                None => format!("{}\\frac{{{}}}{{{d}}}\\%", n / d, n % d),
            }
        }
    }
}

// ------------------------------------------------------- the Navy's grid

/// The search grid of 1968: 12 rows by 15 columns of lettered cells.
pub const NAVY_ROWS: usize = 12;
pub const NAVY_COLS: usize = 15;
/// Where chapters 02 and 03 stand it, and a row's height there.
pub const NAVY_CENTRE: Vec3 = [-2.6, 0.1, 0.0];
pub const NAVY_ROW_H: f64 = 0.42;

/// The cells in the order the sled paints them: row by row from the north,
/// west to east and then east to west.
pub fn navy_paint_order() -> Vec<(usize, usize)> {
    (0..NAVY_ROWS)
        .flat_map(|r| {
            (0..NAVY_COLS).map(move |k| {
                let c = if r % 2 == 0 { k } else { NAVY_COLS - 1 - k };
                (r, c)
            })
        })
        .collect()
}

/// The Navy's grid, empty: its lines, and one teal wash per cell in paint
/// order (detached, for painting in as the strip passes). Cells are 0.84 of
/// a row's height wide, as the printed ones are.
pub struct NavyGrid {
    pub centre: Vec3,
    pub cell_w: f64,
    pub cell_h: f64,
    pub lines: Vec<Mob>,
    /// One per cell, in [`navy_paint_order`].
    pub washes: Vec<Mob>,
}

impl NavyGrid {
    pub fn build(stage: &mut Stage<'_>, centre: Vec3, cell_h: f64) -> fmn::Result<Self> {
        let cell_w = cell_h * y1968::CELL_MILES.1.f64() / y1968::CELL_MILES.0.f64();
        let (half_w, half_h) = (NAVY_COLS as f64 * cell_w / 2.0, NAVY_ROWS as f64 * cell_h / 2.0);
        let style = faded_line(Palette::dim(), 1.5, 0.8);
        let mut lines = Vec::new();
        for c in 0..=NAVY_COLS {
            let x = centre[0] - half_w + c as f64 * cell_w;
            let l = polyline(vec![v(x, centre[1] - half_h), v(x, centre[1] + half_h)]).style(style);
            lines.push(stage.arena_mut().add(l.build()));
        }
        for r in 0..=NAVY_ROWS {
            let y = centre[1] - half_h + r as f64 * cell_h;
            let l = polyline(vec![v(centre[0] - half_w, y), v(centre[0] + half_w, y)]).style(style);
            lines.push(stage.arena_mut().add(l.build()));
        }
        let mut washes = Vec::new();
        for (r, c) in navy_paint_order() {
            let at = v(
                centre[0] + (c as f64 - (NAVY_COLS as f64 - 1.0) / 2.0) * cell_w,
                centre[1] + ((NAVY_ROWS as f64 - 1.0) / 2.0 - r as f64) * cell_h,
            );
            let w = Rectangle::new()
                .width(cell_w)
                .height(cell_h)
                .style(flat(search(), 0.45))
                .build()?
                .moved_to(at);
            washes.push(stage.arena_mut().add(w));
        }
        Ok(Self {
            centre,
            cell_w,
            cell_h,
            lines,
            washes,
        })
    }

    /// The whole grid's width and height.
    pub fn size(&self) -> (f64, f64) {
        (NAVY_COLS as f64 * self.cell_w, NAVY_ROWS as f64 * self.cell_h)
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

    fn between(a: Srgb, b: Srgb) -> f64 {
        de2000(lab(a), lab(b))
    }

    #[test]
    fn de2000_matches_sharma_wu_dalal() {
        // the paper's first test pair
        let d = de2000([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485]);
        assert!((d - 2.0425).abs() < 1e-3, "{d}");
    }

    /// Storyboard §3 says every pair of its roles, with the background, is
    /// at least ΔE2000 24 apart. Measured, that holds for every pair that
    /// does not involve the corpus's `dim` grey (the closest of them is
    /// ink and blue, 24.2). `dim` is closer to violet (15.4), to slate
    /// (20.2) and to blue (21.6): the last-known-position diamond and the
    /// tags are never on a frame with the quiz's violet, and each of them is
    /// a different shape from the dots, so the test pins the true floors
    /// for `dim` rather than the promised 24 (reported to the orchestrator
    /// in the bead comment: a storyboard figure to correct, or a colour).
    #[test]
    fn the_role_colours_are_told_apart() {
        let roles = [
            ("blue map", map()),
            ("teal search", search()),
            ("gold find", find()),
            ("slate out", out()),
            ("ink true place", true_place()),
            ("good", found()),
            ("warn", wrong()),
            ("violet quiz", quiz()),
            ("ground", ground()),
        ];
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                let d = between(*a, *b);
                assert!(d >= 24.0, "{na} and {nb}: ΔE2000 {d:.1}");
            }
        }
        // the corpus's dim grey, against every role: never closer than 15
        for (name, c) in &roles {
            let d = between(last_known(), *c);
            assert!(d >= 15.0, "dim and {name}: ΔE2000 {d:.1}");
        }
        assert!(
            between(last_known(), quiz()) < 24.0,
            "if dim now clears 24, say so in the test"
        );
    }

    #[test]
    fn scatter_keeps_every_dot_inside_and_apart_and_repeats() {
        for n in [1usize, 2, 7, 30, 100, 417, 1250] {
            let (w, h) = (0.86 * 0.84, 0.86);
            let pts = scatter(n, w, h, 0xB0A7 + n as u64);
            assert_eq!(pts.len(), n);
            for &(x, y) in &pts {
                assert!(x.abs() <= w / 2.0 && y.abs() <= h / 2.0, "n = {n}: ({x}, {y})");
            }
            // the same dots every time
            assert_eq!(pts, scatter(n, w, h, 0xB0A7 + n as u64));
            // never closer than 0.4 of a grid step
            let cols = ((n as f64 * w / h).sqrt().ceil() as usize).max(1);
            let rows = n.div_ceil(cols).max(1);
            let step = (w / cols as f64).min(h / rows as f64);
            for (i, a) in pts.iter().enumerate() {
                for b in &pts[i + 1..] {
                    let d = (a.0 - b.0).hypot(a.1 - b.1);
                    assert!(d >= 0.4 * step - 1e-12, "n = {n}: {d} < {}", 0.4 * step);
                }
            }
        }
        assert!(scatter(0, 1.0, 1.0, 7).is_empty());
    }

    #[test]
    fn heat_is_nothing_at_nothing_grows_with_share_and_stops_at_its_cap() {
        for heat in [Heat::LATTICE, Heat::SEA] {
            assert_eq!(heat.opacity(0.0), 0.0);
            let mut last = 0.0;
            for k in 1..=100 {
                let o = heat.opacity(f64::from(k) / 100.0);
                assert!(o >= last, "monotone");
                assert!(o <= HEAT_MAX + 1e-12);
                last = o;
            }
            assert_eq!(heat.opacity(1.0), HEAT_MAX);
        }
        // the lattice's hottest cell, one dot in eight, is the deepest blue;
        // the sea's likeliest, two in five, is not (a half would be)
        assert!((Heat::LATTICE.opacity(1250.0 / 10_000.0) - HEAT_MAX).abs() < 1e-12);
        assert!(Heat::SEA.opacity(0.4) < HEAT_MAX && Heat::SEA.opacity(0.4) > Heat::SEA.opacity(0.3));
    }

    fn e5() -> (usize, usize) {
        y1968::at("E5").unwrap()
    }

    /// Storyboard 01_2: a search of the hottest cell that sees two in
    /// three puts out whole dots, 833 of its 1,250.
    #[test]
    fn the_cold_opens_search_puts_out_833_of_1250() {
        let (row, col) = e5();
        let probe = Probe {
            row,
            col,
            seeing: Q::lowest(2, 3),
        };
        let c = counts(&y1968::POINTS, &[probe]);
        let i = row * LAT_COLS + col;
        assert_eq!(c.lit[0][i], 1250);
        assert_eq!(c.out, vec![833]);
        assert_eq!(c.lit[1][i], 417);
        assert_eq!(c.lit[0].iter().sum::<u32>(), 10_000);
        assert_eq!(c.lit[1].iter().sum::<u32>(), 10_000 - 833);
    }

    /// The "always" the cold open draws: after an empty search every other
    /// cell that holds dots is a deeper blue, and the searched cell a
    /// cooler one. Tested at its edges: a search that sees nothing changes
    /// nothing; one that sees everything empties its cell and deepens the
    /// rest by exactly 1 / (1 − its share); a cell with no dots stays
    /// colourless; the two cells that hold the fewest dots still deepen.
    #[test]
    fn an_empty_search_cools_its_cell_and_deepens_every_other_that_holds_dots() {
        let (row, col) = e5();
        let i = row * LAT_COLS + col;
        let before = heat_state(&counts(&y1968::POINTS, &[]).lit[0], Heat::LATTICE);
        let run = |seeing: Q| {
            let c = counts(&y1968::POINTS, &[Probe { row, col, seeing }]);
            heat_state(&c.lit[1], Heat::LATTICE)
        };

        // sees two in three
        let after = run(Q::lowest(2, 3));
        assert!(after[i] < before[i], "the searched cell cools");
        let mut deeper = 0;
        for (j, (&b, &a)) in before.iter().zip(&after).enumerate() {
            if j == i {
                continue;
            }
            if y1968::POINTS[j / LAT_COLS][j % LAT_COLS] == 0 {
                assert_eq!(a, 0.0, "a cell with no dots stays colourless");
            } else if b < HEAT_MAX {
                assert!(a > b, "cell {j} must deepen: {b} to {a}");
                deeper += 1;
            } else {
                // a cell already at the deepest blue cannot go further
                assert!(a <= HEAT_MAX + 1e-12);
            }
        }
        assert!(deeper > 100, "{deeper} cells deepened");

        // sees nothing: nothing moves
        assert_eq!(run(Q::ZERO), before);

        // sees everything: the cell is empty and every other is scaled by
        // 1 / (1 − 1250/10000), until the cap
        let all = run(Q::ONE);
        assert_eq!(all[i], 0.0);
        let scale = 1.0 / (1.0 - 1250.0 / 10_000.0);
        for (j, (&b, &a)) in before.iter().zip(&all).enumerate() {
            if j != i {
                assert!((a - (b * scale).min(HEAT_MAX)).abs() < 1e-12, "cell {j}");
            }
        }
    }

    #[test]
    fn a_second_search_of_a_cell_takes_what_the_first_left() {
        let (row, col) = e5();
        let p = Probe {
            row,
            col,
            seeing: Q::lowest(2, 3),
        };
        let c = counts(&y1968::POINTS, &[p, p]);
        // 1250, then 417 left, then two thirds of 417 (278) put out
        assert_eq!(c.out, vec![833, 278]);
        assert_eq!(c.lit[2][row * LAT_COLS + col], 139);
    }

    #[test]
    fn the_lattice_and_the_quiz_frame_sit_on_the_stage() {
        // the cold open's lattice, 0.46 a row
        let h = 0.46;
        let w = h * 21.0 / 25.0;
        assert!(LAT_ROWS as f64 * h <= 2.0 * FRAME_Y_RADIUS - 0.5);
        assert!(LAT_COLS as f64 * w <= 2.0 * FRAME_X_RADIUS - 1.0);
        // the frame is inside the stage, the ring inside the frame, the
        // question and its note inside the frame
        assert!(QUIZ_FRAME.0 < FRAME_X_RADIUS && QUIZ_FRAME.1 < FRAME_Y_RADIUS);
        assert!(QUIZ_RING_AT[0] + QUIZ_RING_R < QUIZ_FRAME.0);
        assert!(QUIZ_RING_AT[1] + QUIZ_RING_R < QUIZ_FRAME.1);
        for y in QUIZ_LINES_Y {
            assert!(
                y.abs() + QUIZ_LINE_REACH < QUIZ_FRAME.1,
                "line at {y} touches the frame"
            );
        }
        assert!(
            QUIZ_NOTE_Y.abs() + QUIZ_NOTE_REACH < QUIZ_FRAME.1,
            "the note at {QUIZ_NOTE_Y} touches the frame"
        );
        // the note's top clears the second line's foot, and the lines do not
        // run into each other (constants, so checked as the crate builds)
        const {
            assert!(QUIZ_NOTE_Y + QUIZ_NOTE_REACH <= QUIZ_LINES_Y[1] - QUIZ_LINE_REACH + 1e-9);
            assert!(QUIZ_LINES_Y[0] - QUIZ_LINES_Y[1] >= 2.0 * QUIZ_LINE_REACH - 1e-9);
        }
    }

    #[test]
    fn every_quiz_is_a_line_of_the_script() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for q in QUIZZES {
            assert!(script.text(q.line).is_some(), "{}", q.line);
            assert!((4..=6).contains(&q.seconds));
        }
    }

    #[test]
    fn the_sea_is_two_by_two_with_room_for_its_sums_and_the_question() {
        use crate::model::sea;
        // A and B above C and D, no cell on another
        for (i, a) in (0..4).map(sea_rect).enumerate() {
            for (j, b) in (0..4).map(sea_rect).enumerate() {
                if i < j {
                    let apart = a.2 <= b.0 || b.2 <= a.0 || a.3 <= b.1 || b.3 <= a.1;
                    assert!(apart, "cells {i} and {j} overlap");
                }
            }
        }
        assert!(SEA_CENTRES[0][1] > SEA_CENTRES[2][1] && SEA_CENTRES[0][0] < SEA_CENTRES[1][0]);
        // the sum's three rows sit under their cell, above the next row's
        // cell (or the question's lines), and inside the frame's left half
        for cell in 0..4 {
            for k in 0..3 {
                let at = sea_slot(cell, k);
                assert!(at[1] < sea_rect(cell).1, "row {k} of {cell} is under its cell");
                // a row's text reaches about 0.15 below its centre: it must
                // clear the cell under it, or the question's first line
                let floor = if cell < 2 {
                    sea_rect(cell + 2).3
                } else {
                    QUIZ_LINES_Y[0] + 0.2
                };
                assert!(at[1] - 0.15 > floor, "row {k} of {cell} runs into what is under it");
                assert!(at[0].abs() < QUIZ_FRAME.0);
            }
        }
        // the sea's cells keep to the left half, leaving the right for cards
        assert!(sea_rect(1).2 < 0.0);
        // the two stories make A's 40, B's 30, C's 20 and D's 10 dots
        let split = sea_split();
        assert_eq!(split, [[30, 20, 10, 0], [10, 10, 10, 10]]);
        let per_cell: Vec<usize> = (0..4).map(|c| split[0][c] + split[1][c]).collect();
        assert_eq!(per_cell, [40, 30, 20, 10]);
        assert_eq!(per_cell.iter().sum::<usize>() as i128, sea::DOTS);
    }

    #[test]
    fn a_ring_is_whole_at_one_and_a_third_at_a_third() {
        let at = v(0.0, 0.0);
        let whole = ring_points(at, 1.0, 1.0);
        let (first, last) = (whole[0], whole[whole.len() - 1]);
        assert!(
            (first[0] - last[0]).abs() < 1e-9 && (first[1] - last[1]).abs() < 1e-9,
            "closes"
        );
        let third = ring_points(at, 1.0, 1.0 / 3.0);
        let end = third[third.len() - 1];
        // a third of a turn from the top is 120 degrees round
        let a = (end[1].atan2(end[0]) - FRAC_PI_2).rem_euclid(TAU);
        assert!((a - TAU / 3.0).abs() < 1e-6, "{a}");
        assert!(third.iter().all(|p| (p[0].hypot(p[1]) - 1.0).abs() < 1e-9));
        // it never has fewer than two points, even at nothing
        assert!(ring_points(at, 1.0, 0.0).len() >= 2);
    }

    #[test]
    fn frac_and_percent_tex_print_thirds_as_fractions_and_eighths_as_decimals() {
        assert_eq!(frac_tex(Q::lowest(1, 2)), "\\frac{1}{2}");
        assert_eq!(frac_tex(Q::ZERO), "0");
        assert_eq!(frac_tex(Q::ONE), "1");
        assert_eq!(percent_tex(Q::lowest(2, 5)), "40\\%");
        assert_eq!(percent_tex(Q::lowest(1, 8)), "12.5\\%");
        assert_eq!(percent_tex(Q::lowest(2, 15)), "13\\frac{1}{3}\\%");
        assert_eq!(percent_tex(Q::lowest(1, 15)), "6\\frac{2}{3}\\%");
        assert_eq!(percent_tex(Q::lowest(1, 16)), "6.25\\%");
    }

    #[test]
    fn the_navys_grid_is_painted_row_by_row_back_and_forth() {
        let order = navy_paint_order();
        assert_eq!(order.len(), NAVY_ROWS * NAVY_COLS);
        let mut seen = order.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 180, "every cell once");
        assert_eq!(order[0], (0, 0));
        assert_eq!(order[NAVY_COLS - 1], (0, NAVY_COLS - 1));
        assert_eq!(order[NAVY_COLS], (1, NAVY_COLS - 1), "the second row comes back");
        // the painted forty are two whole rows and ten cells of the third
        let forty = &order[..40];
        assert_eq!(forty.iter().filter(|&&(r, _)| r < 2).count(), 30);
        assert_eq!(forty.iter().filter(|&&(r, _)| r == 2).count(), 10);
    }

    /// The glyph pages showed the kit's greedy wrap cutting a question off
    /// with " …" ("… in cell …", "… A or …") and splitting "E | five". Every
    /// question the video asks, a quiz or a carried one, now fits two lines
    /// with all its words, breaks at the end of a sentence where it can, and
    /// never strands a one-letter word.
    #[test]
    fn every_question_fits_two_lines_whole() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        // the quizzes, and the questions carried with no ring (storyboard,
        // "Two kinds of question")
        let mut ids: Vec<&str> = QUIZZES.iter().map(|q| q.line).collect();
        ids.extend(["02_7", "04_5", "08_1", "10_2", "11_4", "12_4", "13_3", "14_3"]);
        for id in ids {
            let text = script.text(id);
            assert!(text.is_some(), "no line {id}");
            let text = text.unwrap();
            let lines = balanced_lines(text, QUIZ_LINE_CHARS);
            assert!(lines.len() <= QUIZ_LINES_Y.len(), "{id}: {} lines", lines.len());
            assert_eq!(
                lines.join(" "),
                text.split_whitespace().collect::<Vec<_>>().join(" "),
                "{id}: words lost"
            );
            assert!(!lines.iter().any(|l| l.contains('…')), "{id} is cut off");
            for l in &lines {
                assert!(l.chars().count() <= QUIZ_LINE_CHARS, "{id}: {l:?}");
            }
            if let [first, _] = lines.as_slice() {
                let last = first.split_whitespace().last().unwrap_or("");
                assert!(
                    last.chars().count() > 1 || last.ends_with(['.', '?', '!']),
                    "{id} strands {last:?} at the end of a line"
                );
            }
        }
    }

    #[test]
    fn balanced_lines_break_at_a_sentence_and_never_cut_a_name() {
        // at the width of a full-size line
        let n = QUIZ_FULL_CHARS;
        // short: one line
        assert_eq!(
            balanced_lines("So where would you look first?", n),
            ["So where would you look first?"]
        );
        // the cell E five stays whole, the break is at the sentence's end
        assert_eq!(
            balanced_lines(
                "One cell stands out, with one dot in every eight. They called it E five. Was she there?",
                n
            ),
            [
                "One cell stands out, with one dot in every eight.",
                "They called it E five. Was she there?"
            ]
        );
        // the question and its facts apart
        assert_eq!(
            balanced_lines(
                "Forty percent, seen one time in four. Thirty percent, seen two times in three. \
                 Which cell gives the better chance of a find, A or B?",
                n
            ),
            [
                "Forty percent, seen one time in four. Thirty percent, seen two times in three.",
                "Which cell gives the better chance of a find, A or B?"
            ]
        );
        // a text too long for two lines runs on to a third, whole
        let long = "word ".repeat(60);
        let lines = balanced_lines(&long, n);
        assert!(lines.len() >= 3);
        assert_eq!(lines.join(" "), long.trim());
        // the face's measured width: a full line fits the frame with room
        const {
            assert!(QUIZ_FULL_CHARS as f64 * QUIZ_PT * CHAR_UNITS_PER_POINT < 2.0 * QUIZ_FRAME.0 - 2.0);
        }
    }

    /// The longest question (09_5, 179 characters) takes two lines of about
    /// 90 and is set smaller, so no question's line is wider than a full
    /// one, and none is set under 22 pt.
    #[test]
    fn a_long_question_is_set_smaller_never_cut_and_never_wider() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let long = script.text("09_5").unwrap();
        let lines = balanced_lines(long, QUIZ_LINE_CHARS);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= QUIZ_LINE_CHARS));
        let pt = question_pt(&lines);
        assert!((22.0..QUIZ_PT).contains(&pt), "{pt}");
        // a short question is full size
        assert_eq!(question_pt(&["So where would you look first?".to_owned()]), QUIZ_PT);
        // and every question's widest line, at its size, is within a full line
        let mut ids: Vec<&str> = QUIZZES.iter().map(|q| q.line).collect();
        ids.extend(["02_7", "04_5", "08_1", "10_2", "11_4", "12_4", "13_3", "14_3"]);
        for id in ids {
            let lines = balanced_lines(script.text(id).unwrap(), QUIZ_LINE_CHARS);
            let pt = question_pt(&lines);
            assert!(pt >= 22.0, "{id}: {pt} pt");
            let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64;
            assert!(
                widest * pt * CHAR_UNITS_PER_POINT <= QUIZ_FULL_CHARS as f64 * QUIZ_PT * CHAR_UNITS_PER_POINT + 1e-9,
                "{id} is wider than a full line"
            );
        }
    }

    #[test]
    fn a_line_gives_back_its_sentences_as_written() {
        let text = "A nuclear sub has gone missing. Men are trapped, and time is running out. \
                    How can we possibly find them in time?";
        let spoken = Spoken {
            id: "01_1".into(),
            start: 0.0,
            len: 9.0,
            text: text.into(),
            sentences: sentence_offsets(text).into_iter().map(|at| (at, 0.0)).collect(),
        };
        assert_eq!(
            spoken.sentence_texts(),
            [
                "A nuclear sub has gone missing.",
                "Men are trapped, and time is running out.",
                "How can we possibly find them in time?"
            ]
        );
    }

    #[test]
    fn sentences_split_where_the_voice_starts_a_new_one() {
        let text = "Watch a search that finds nothing. It has still told you something.";
        assert_eq!(sentence_offsets(text), vec![0, 35]);
        assert_eq!(sentence_offsets("One."), vec![0]);
    }

    /// The narrated draft's 05_6, 06 and 07_3 had "13⅓%" through the teal
    /// "2 in 3" above it. Standing TALL_DROP under its slot, a mixed
    /// fraction's top edge is under the teal digits' bottom edge; its foot
    /// is above the question's band under the bottom row. The top row's
    /// cells have 0.24 under their slot before the cell below starts: no
    /// stacked fraction fits there.
    #[test]
    fn a_mixed_fraction_in_the_gold_slot_clears_the_teal_row_above_it() {
        // a teal 26 pt digit reaches 0.1 either side of its slot's centre
        let teal_bottom = sea_slot(2, 1)[1] - 0.1;
        let tall_top = sea_slot(2, 2)[1] - TALL_DROP + TALL_HALF_H;
        assert!(tall_top < teal_bottom, "{tall_top} against {teal_bottom}");
        let tall_foot = sea_slot(2, 2)[1] - TALL_DROP - TALL_HALF_H;
        assert!(tall_foot > QUIZ_LINES_Y[0] + 0.2, "the sums reach the question's band");
        // the top row has no room for one
        let room_below = sea_slot(0, 2)[1] - sea_rect(2).3;
        assert!((room_below - 0.24).abs() < 1e-9, "{room_below}");
        assert!(room_below < TALL_DROP + TALL_HALF_H);
    }
}
