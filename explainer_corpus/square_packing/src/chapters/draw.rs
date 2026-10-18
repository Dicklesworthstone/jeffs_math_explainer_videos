//! The five-tile video's look, shared by its chapters (storyboard §2): the
//! colour roles, the board (the box drawn at a fixed scale about a fixed
//! centre, however wide it is), tiles drawn from `model` poses, the overlap
//! patches, line timing, and the plays the chapters use.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation or `add_to_scene`. A
//! tile on screen is its model pose, pulled in by [`SEAM`] on every side so
//! that two touching tiles show a hairline between them; the box outline is
//! drawn exactly. Every number on screen comes from [`fact`] or from the
//! model's own geometry; this file types none.

use std::cell::OnceCell;

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Facts, Kit, Palette, Script, Timed, v};
use fmn::prelude::*;

use crate::model::{Point, Tile};

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
pub fn fact_int(key: &str) -> fmn::Result<u32> {
    let value = fact(key)?;
    value
        .parse()
        .map_err(|_| fail(format!("fact {key} = {value:?} is not a whole number")))
}

// ------------------------------------------------------------- colour roles

/// The turned tile, all video long.
pub fn turned() -> Srgb {
    Palette::hero()
}

/// The four straight tiles in the corners.
pub fn corner() -> Srgb {
    Palette::blue()
}

/// The box.
pub fn ink() -> Srgb {
    Palette::ink()
}

/// Room, reach and ghosts: empty cells, the circle, the old box.
pub fn faint() -> Srgb {
    Palette::faint()
}

/// A touch (a dot, never a tint alone).
pub fn touch() -> Srgb {
    Palette::green()
}

/// What fails: an overlap.
pub fn warn() -> Srgb {
    Palette::red()
}

/// Measures and small print.
pub fn dim() -> Srgb {
    Palette::dim()
}

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

// -------------------------------------------------------------------- board

/// How far a drawn tile is pulled in from its pose on every side (tiles):
/// a seam, so two tiles that touch read as two.
pub const SEAM: f64 = 0.012;

/// The box on screen: its centre stays at `at` and a tile is `k` scene units
/// wide, so a squeeze shrinks the box about its centre.
#[derive(Clone, Copy, Debug)]
pub struct Board {
    pub at: Vec3,
    pub k: f64,
}

impl Board {
    /// The scene point of box point `q` when the box is `side` wide.
    pub fn p(&self, side: f64, q: Point<f64>) -> Vec3 {
        v(
            self.at[0] + self.k * (q.0 - side / 2.0),
            self.at[1] + self.k * (q.1 - side / 2.0),
        )
    }

    /// A tile's corners on screen, pulled in by [`SEAM`].
    pub fn tile(&self, side: f64, t: &Tile<f64>) -> Vec<Vec3> {
        let pulled = Tile {
            centre: t.centre,
            cos: t.cos * (1.0 - 2.0 * SEAM),
            sin: t.sin * (1.0 - 2.0 * SEAM),
        };
        pulled.corners().iter().map(|&q| self.p(side, q)).collect()
    }

    /// The box's outline, closed (its first corner repeated).
    pub fn outline(&self, side: f64) -> Vec<Vec3> {
        [(0.0, 0.0), (side, 0.0), (side, side), (0.0, side), (0.0, 0.0)]
            .iter()
            .map(|&q| self.p(side, q))
            .collect()
    }
}

/// A filled polygon through `pts`. Detached.
pub fn polygon(stage: &mut Stage<'_>, pts: Vec<Vec3>, style: Style) -> Mob {
    stage.arena_mut().add(Polygon::new(pts).style(style).build())
}

/// An open path through `pts`. Detached.
pub fn path(stage: &mut Stage<'_>, pts: Vec<Vec3>, style: Style) -> Mob {
    stage.arena_mut().add(polyline(pts).style(style).build())
}

/// Replace a polygon's outline with `pts` (in an updater).
pub fn reshape(s: &mut fmn::mobject::Stage, m: Mob, pts: Vec<Vec3>) {
    let fresh = Polygon::new(pts).build();
    let _ = s.set_points(m, fresh.points());
}

/// Replace a path's points with `pts` (in an updater).
pub fn retrace(s: &mut fmn::mobject::Stage, m: Mob, pts: Vec<Vec3>) {
    let fresh = polyline(pts).build();
    let _ = s.set_points(m, fresh.points());
}

/// A dot of radius `r` at `at`. Detached.
pub fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(Circle::new().radius(r).arc_center(at).style(flat(c, 1.0)).build())
}

/// Move a dot made by [`dot`] so it is centred at `at` (in an updater).
pub fn redot(s: &mut fmn::mobject::Stage, m: Mob, at: Vec3, r: f64) {
    let fresh = Circle::new().radius(r).arc_center(at).build();
    let _ = s.set_points(m, fresh.points());
}

/// A mobject already in the scene brought to `opacity`, `rt` seconds (the
/// board's parts enter the scene once, in drawing order, and are revealed
/// this way, so the pins stay above the tiles).
pub fn fade_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

/// A circle outline of radius `r` about `at`. Detached.
pub fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, c: Srgb, w: f64) -> Mob {
    stage
        .arena_mut()
        .add(Circle::new().radius(r).arc_center(at).style(line_style(c, w)).build())
}

/// The points of an arc of radius `r` about `at`, from the top, clockwise,
/// covering `share` of the circle (a draining ring), at least two points.
pub fn arc_points(at: Vec3, r: f64, share: f64) -> Vec<Vec3> {
    let n = 48;
    let sweep = std::f64::consts::TAU * share.clamp(0.0, 1.0);
    (0..=n)
        .map(|i| {
            let a = std::f64::consts::FRAC_PI_2 - sweep * f64::from(i) / f64::from(n);
            v(at[0] + r * a.cos(), at[1] + r * a.sin())
        })
        .collect()
}

/// The common part of two convex counter-clockwise polygons (Sutherland and
/// Hodgman), for drawing where two tiles overlap; empty when they do not.
/// Drawing only: the checks are `model::clearance` and `oracle.py`.
pub fn common(a: &[Point<f64>], b: &[Point<f64>]) -> Vec<Point<f64>> {
    let cross = |p: Point<f64>, q: Point<f64>, r: Point<f64>| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
    let mut out: Vec<Point<f64>> = a.to_vec();
    for (i, &p) in b.iter().enumerate() {
        let q = b[(i + 1) % b.len()];
        let pts = std::mem::take(&mut out);
        for (j, &s) in pts.iter().enumerate() {
            let e = pts[(j + 1) % pts.len()];
            let (ds, de) = (cross(p, q, s), cross(p, q, e));
            if ds >= 0.0 {
                out.push(s);
            }
            if (ds > 0.0 && de < 0.0) || (ds < 0.0 && de > 0.0) {
                let t = ds / (ds - de);
                out.push((s.0 + t * (e.0 - s.0), s.1 + t * (e.1 - s.1)));
            }
        }
        if out.is_empty() {
            break;
        }
    }
    out
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began and how long it runs, so a picture can
/// land on the words it illustrates. A phrase's moment is its share of the
/// line's characters (the voice reads at a nearly even rate); the cue sheets
/// of the narrated draft check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
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
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
    })
}

impl Spoken {
    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let byte = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let i = self.text[..byte].chars().count() as f64;
        let all = self.text.chars().count().max(1) as f64;
        Ok(self.start + self.len * i / all)
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

    /// The seconds left of the line from now, at least `floor`: how long a
    /// move may take and still end with the words.
    pub fn left(&self, stage: &Stage<'_>, floor: f64) -> f64 {
        (self.end() - stage.scene().time().to_f64()).max(floor)
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

// ------------------------------------------------------------------- plays

/// Fade `out` away and `ins` in, in one move of `rt` seconds.
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

/// Bring detached mobjects in one after another, the whole taking `rt`.
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

/// Move a mobject so its centre is at `to`, `rt` seconds.
pub fn fly(mob: Mob, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// Scale a mobject about its centre by `by`, `rt` seconds.
pub fn grow(mob: Mob, by: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(by)?)
}

/// Builders played together, `rt` seconds each (as set on them).
pub fn together(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(moves.len());
    for m in moves {
        anims.push(stage.prepare(m)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_common_part_of_two_tiles() {
        // a tile and its copy moved half a tile right and up: a quarter
        let a = Tile::<f64>::straight(0.5, 0.5).corners();
        let b = Tile::<f64>::straight(1.0, 1.0).corners();
        let c = common(&a, &b);
        let area: f64 = (0..c.len())
            .map(|i| {
                let (p, q) = (c[i], c[(i + 1) % c.len()]);
                p.0 * q.1 - q.0 * p.1
            })
            .sum::<f64>()
            / 2.0;
        assert!((area - 0.25).abs() < 1e-12, "{area}");
        // apart: nothing in common
        let far = Tile::<f64>::straight(3.0, 0.5).corners();
        assert!(common(&a, &far).is_empty());
    }

    #[test]
    fn the_board_keeps_its_centre() {
        let b = Board {
            at: v(1.0, -1.0),
            k: 2.0,
        };
        for side in [3.0, 2.7] {
            let c = b.p(side, (side / 2.0, side / 2.0));
            assert!((c[0] - 1.0).abs() < 1e-12 && (c[1] + 1.0).abs() < 1e-12);
        }
        assert_eq!(b.outline(3.0).len(), 5);
        assert!((b.p(3.0, (3.0, 0.0))[0] - 4.0).abs() < 1e-12);
    }
}
