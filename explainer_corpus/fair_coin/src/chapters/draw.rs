//! Video E's look, shared by every chapter (storyboard §3, colour roles):
//! the coin's two faces as discs, Alice's and Bob's colours, the unit square
//! cut by the chances (the one picture of the video), the same picture with
//! a third side (a cube of boxes, drawn by projection on the fast route), die
//! faces, bars, tags and cards.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, small groups. The square and the cube are
//! drawn from value trackers by one updater each, which rewrites a handful
//! of polygons and only when a tracker has moved.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, hex, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use crate::model::Face;

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

/// A line being spoken: when it began and how long it runs, so a picture
/// can land on the phrase it illustrates. A phrase's moment is its share of
/// the line's characters (the voice reads at a nearly even rate); the cue
/// sheets of the narrated draft check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
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
        let i = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let before = self.text[..i].chars().count() as f64;
        let all = self.text.chars().count().max(1) as f64;
        Ok(self.start + self.len * before / all)
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

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown).
pub const THINK_S: f64 = 4.0;
const THINK_R: f64 = 0.62;
const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause with think time, in place of the kit's two-second
/// one: let the question finish, mark the moment for the score, bring in a
/// large "?" at `at` inside a gold ring that drains over [`THINK_S`] with
/// "pause for more time" under it; at zero the ring and the note go.
/// Returns the "?", which the scene clears with the answer.
pub fn predict_pause(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 90.0, Palette::hero(), at)?;
    // beside the ring, not under it: under it sits on the coin (01_6) or
    // the die (08_2)
    let note = kit.text_left(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0] + THINK_R + 0.2, at[1]))?;
    // the ring: what is left of the think time, clockwise from the top
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + THINK_R * a.cos(), at[1] + THINK_R * a.sin())
            })
            .collect()
    };
    let left = stage.add_value_tracker(1.0);
    let ring = stage
        .arena_mut()
        .add(polyline(arc(1.0)).style(line_style(gold(), 4.0)).build());
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
    fade_in_all(stage, &[anchor, ring, q, note], 0.3)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.25)?;
    Ok(q)
}

// ------------------------------------------------------------- colour roles

/// Heads; for a die, the stretch of a bigger roll (drawn nearer the corner).
pub fn heads() -> Srgb {
    hex("#58C4DD")
}

/// Tails.
pub fn tails() -> Srgb {
    hex("#EA87B3")
}

/// Alice, and the block that wins for her: above the diagonal, always.
pub fn alice() -> Srgb {
    hex("#8BD96F")
}

/// Bob, and his block: below the diagonal, always.
pub fn bob() -> Srgb {
    hex("#994DE5")
}

/// The mirror line and "same counts, same chance": strokes, glows and words
/// only, never a fill.
pub fn gold() -> Srgb {
    Palette::hero()
}

/// The part of a block that no longer fits its mirror image.
pub fn warn() -> Srgb {
    Palette::red()
}

/// A thrown-away pair or block.
pub fn thrown() -> Srgb {
    Palette::faint()
}

/// A thrown-away block while it is the subject (chapter 04).
pub fn thrown_lit() -> Srgb {
    hex("#5E6878")
}

/// A card's or panel's back.
pub fn panel() -> Srgb {
    hex("#1A2130")
}

/// A face's colour.
pub fn face_color(f: Face) -> Srgb {
    match f {
        Face::H => heads(),
        Face::T => tails(),
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

/// An `Srgb` as the `[f32; 3]` that `indicate` and its kin take.
pub fn rgb(c: Srgb) -> [f32; 3] {
    [c.r as f32, c.g as f32, c.b as f32]
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

/// Fade out every top-level mobject except `keep`, `rt` seconds: how a
/// chapter ends when the next one opens on the same picture, so the picture
/// stays through the cut. `keep` must be top-level mobjects themselves (a
/// kept member of a group that fades would fade with it).
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
        anims.push(stage.prepare(explainer_kit::tween(t, value, rt)?)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

// ------------------------------------------------------------ coin and pairs

/// One result of the coin as a disc in its face's colour, centred at `at`.
pub fn disc(f: Face, at: Vec3, radius: f64) -> VMobject {
    Circle::new()
        .radius(radius)
        .style(flat(face_color(f), 1.0))
        .build()
        .moved_to(at)
}

/// A disc with its letter (H or T) on it, for the few results a viewer is
/// asked to read; `size` is the letter's font size. Returns the disc and the
/// letter, detached.
pub fn lettered_disc(
    kit: &Kit,
    stage: &mut Stage<'_>,
    f: Face,
    at: Vec3,
    radius: f64,
    size: f64,
) -> fmn::Result<[Mob; 2]> {
    let d = stage.arena_mut().add(disc(f, at, radius));
    let l = kit.text(stage, &f.letter().to_string(), size, hex("#0E1116"), at)?;
    Ok([d, l])
}

/// Two results side by side on a small card: one pair. Detached parts, the
/// card first.
pub fn pair_tile(kit: &Kit, stage: &mut Stage<'_>, pair: (Face, Face), at: Vec3, scale: f64) -> fmn::Result<Vec<Mob>> {
    let (r, gap) = (0.26 * scale, 0.32 * scale);
    let back = Rectangle::new()
        .width(4.0 * r + 0.34 * scale)
        .height(2.0 * r + 0.3 * scale)
        .corner_radius(0.1 * scale)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut out = vec![stage.arena_mut().add(back)];
    for (f, dx) in [(pair.0, -gap), (pair.1, gap)] {
        out.extend(lettered_disc(kit, stage, f, v(at[0] + dx, at[1]), r, 30.0 * scale)?);
    }
    Ok(out)
}

/// A row of small discs for a run of results, left to right from `left`.
pub fn disc_row(stage: &mut Stage<'_>, run: &[Face], left: Vec3, radius: f64, step: f64) -> Vec<Mob> {
    run.iter()
        .enumerate()
        .map(|(i, &f)| {
            let d = disc(f, v(left[0] + step * i as f64, left[1]), radius);
            stage.arena_mut().add(d)
        })
        .collect()
}

/// A small tag: a few words on a rounded back, for "a model coin" and the
/// like (smaller than a lower-third, and it sits by the thing it names).
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

/// The one caption slot under the stage (y = −3.35): detached text.
pub fn caption(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(stage, words, 26.0, Palette::dim(), v(0.0, CAPTION_Y))
}

/// The caption slot's height.
pub const CAPTION_Y: f64 = -3.35;

/// Swap the caption: fade `old` (if any) out and `new` in, in one move.
pub fn swap_caption(stage: &mut Stage<'_>, old: Option<Mob>, new: Mob, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    if let Some(o) = old {
        let a = fade_out(stage.arena_mut(), o, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    let a = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
    anims.push(stage.prepare(a.rt(rt))?);
    stage.play_prepared(anims)?;
    Ok(())
}

// --------------------------------------------------------------------- bars

/// A horizontal bar in two colours whose split follows a tracker: `left`
/// share on the left, the rest on the right. For the coin's own tally and
/// the game's.
pub struct SplitBar {
    /// 0 to 1: the left colour's share.
    pub share: Mob,
    pub parts: [Mob; 2],
    pub anchor: Mob,
}

impl SplitBar {
    /// A bar `w` by `h` with its left end at `left` (its vertical middle),
    /// starting at `share`. Detached: add [`SplitBar::mobs`] to the scene.
    pub fn build(
        stage: &mut Stage<'_>,
        left: Vec3,
        (w, h): (f64, f64),
        colors: (Srgb, Srgb),
        share: f64,
    ) -> fmn::Result<Self> {
        let share_t = stage.add_value_tracker(share);
        let a = stage
            .arena_mut()
            .add(Polygon::new(unit_quad()).style(flat(colors.0, 1.0)).build());
        let b = stage
            .arena_mut()
            .add(Polygon::new(unit_quad()).style(flat(colors.1, 1.0)).build());
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let f = s.tracker_value(share_t).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(f) {
                    return;
                }
                let cut = left[0] + w * f.clamp(0.002, 0.998);
                let (y0, y1) = (left[1] - h / 2.0, left[1] + h / 2.0);
                let quads = [
                    [v(left[0], y0), v(cut, y0), v(cut, y1), v(left[0], y1)],
                    [v(cut, y0), v(left[0] + w, y0), v(left[0] + w, y1), v(cut, y1)],
                ];
                for (m, q) in [a, b].into_iter().zip(quads) {
                    let _ = s.set_points(m, Polygon::new(q).build().points());
                }
                last = Some(f);
            },
            true,
        )?;
        Ok(Self {
            share: share_t,
            parts: [a, b],
            anchor,
        })
    }

    /// Everything to add to the scene (the anchor carries the updater).
    pub fn mobs(&self) -> [Mob; 3] {
        [self.anchor, self.parts[0], self.parts[1]]
    }
}

fn unit_quad() -> [Vec3; 4] {
    [v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)]
}

/// A plain filled bar rising from `foot` (its bottom centre) to `height`.
pub fn column(foot: Vec3, width: f64, height: f64, color: Srgb) -> VMobject {
    let h = height.max(0.004);
    Polygon::new([
        v(foot[0] - width / 2.0, foot[1]),
        v(foot[0] + width / 2.0, foot[1]),
        v(foot[0] + width / 2.0, foot[1] + h),
        v(foot[0] - width / 2.0, foot[1] + h),
    ])
    .style(flat(color, 1.0))
    .build()
}

// --------------------------------------------------------------- the square

/// Which of the square's four blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    /// Two heads: bottom-left.
    HH,
    /// Heads then tails: top-left, above the diagonal. Alice's.
    HT,
    /// Tails then heads: bottom-right, below the diagonal. Bob's.
    TH,
    /// Two tails: top-right.
    TT,
}

pub const BLOCKS: [Block; 4] = [Block::HH, Block::HT, Block::TH, Block::TT];

/// The square's trackers: where the first toss cuts it (left to right), where
/// the second toss cuts the heads column and the tails column (bottom to
/// top), and where the square sits. For one coin the three cuts agree; a
/// second coin moves the last two together; a memory moves them apart.
#[derive(Clone, Copy, Debug)]
pub struct SquareView {
    pub x: Mob,
    pub y_h: Mob,
    pub y_t: Mob,
    /// The lower-left corner and the side, in scene units.
    pub left: Mob,
    pub bottom: Mob,
    pub side: Mob,
}

/// The square's values in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SquarePose {
    pub x: f64,
    pub y_h: f64,
    pub y_t: f64,
    pub left: f64,
    pub bottom: f64,
    pub side: f64,
}

impl SquarePose {
    /// The point at (`u` across, `w` up), both 0 to 1.
    pub fn point(&self, u: f64, w: f64) -> Vec3 {
        v(self.left + self.side * u, self.bottom + self.side * w)
    }

    /// A block's corners in unit coordinates: (u0, w0, u1, w1).
    pub fn unit_rect(&self, b: Block) -> (f64, f64, f64, f64) {
        match b {
            Block::HH => (0.0, 0.0, self.x, self.y_h),
            Block::HT => (0.0, self.y_h, self.x, 1.0),
            Block::TH => (self.x, 0.0, 1.0, self.y_t),
            Block::TT => (self.x, self.y_t, 1.0, 1.0),
        }
    }

    /// A block's share of the square: its pair's chance.
    pub fn share(&self, b: Block) -> f64 {
        let (u0, w0, u1, w1) = self.unit_rect(b);
        (u1 - u0) * (w1 - w0)
    }

    /// A block's centre on screen.
    pub fn center(&self, b: Block) -> Vec3 {
        let (u0, w0, u1, w1) = self.unit_rect(b);
        self.point((u0 + u1) / 2.0, (w0 + w1) / 2.0)
    }

    /// A block's four corners on screen (never thinner than a hairline, so
    /// a coin at 99% still has two blocks to look at).
    fn quad(&self, b: Block) -> [Vec3; 4] {
        let (u0, w0, u1, w1) = self.unit_rect(b);
        let e = 0.0015;
        let (u1, w1) = (u1.max(u0 + e), w1.max(w0 + e));
        [
            self.point(u0, w0),
            self.point(u1, w0),
            self.point(u1, w1),
            self.point(u0, w1),
        ]
    }
}

impl SquareView {
    /// This frame's values (from an updater or `always_redraw` closure).
    pub fn pose(&self, s: &fmn::mobject::Stage) -> SquarePose {
        let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
        SquarePose {
            x: val(self.x),
            y_h: val(self.y_h),
            y_t: val(self.y_t),
            left: val(self.left),
            bottom: val(self.bottom),
            side: val(self.side),
        }
    }
}

/// The unit square cut by the chances: four blocks, the cuts, and a thin
/// frame, all drawn from [`SquareView`]'s trackers.
pub struct Square {
    pub view: SquareView,
    /// HH, HT, TH, TT (as [`BLOCKS`]).
    pub blocks: [Mob; 4],
    /// The first toss's cut, then the second toss's in the heads column and
    /// in the tails column.
    pub cuts: [Mob; 3],
    pub frame: Mob,
    /// Carries the updater: it must be in the scene for the square to move.
    pub anchor: Mob,
}

/// The neutral fill of a block before anyone owns it.
pub fn block_grey() -> Srgb {
    hex("#2B3445")
}

impl Square {
    /// A square with its lower-left corner at `corner` and side `side`, cut
    /// for one coin with chance `p` of heads. Detached; blocks neutral.
    pub fn build(stage: &mut Stage<'_>, corner: Vec3, side: f64, p: f64) -> fmn::Result<Self> {
        let view = SquareView {
            x: stage.add_value_tracker(p),
            y_h: stage.add_value_tracker(p),
            y_t: stage.add_value_tracker(p),
            left: stage.add_value_tracker(corner[0]),
            bottom: stage.add_value_tracker(corner[1]),
            side: stage.add_value_tracker(side),
        };
        let blocks: [Mob; 4] = std::array::from_fn(|_| {
            stage
                .arena_mut()
                .add(Polygon::new(unit_quad()).style(flat(block_grey(), 1.0)).build())
        });
        let cut_style = line_style(hex("#0E1116"), 3.0);
        let cuts: [Mob; 3] = std::array::from_fn(|_| {
            stage
                .arena_mut()
                .add(polyline(vec![v(0.0, 0.0), v(1.0, 0.0)]).style(cut_style).build())
        });
        let frame = stage.arena_mut().add(
            Polygon::new(unit_quad())
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: Option<SquarePose> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let pose = view.pose(s);
                if last == Some(pose) {
                    return;
                }
                for (&m, b) in blocks.iter().zip(BLOCKS) {
                    let _ = s.set_points(m, Polygon::new(pose.quad(b)).build().points());
                }
                let lines = [
                    [pose.point(pose.x, 0.0), pose.point(pose.x, 1.0)],
                    [pose.point(0.0, pose.y_h), pose.point(pose.x, pose.y_h)],
                    [pose.point(pose.x, pose.y_t), pose.point(1.0, pose.y_t)],
                ];
                for (&m, l) in cuts.iter().zip(lines) {
                    let _ = s.set_points(m, polyline(l.to_vec()).build().points());
                }
                let corners = [
                    pose.point(0.0, 0.0),
                    pose.point(1.0, 0.0),
                    pose.point(1.0, 1.0),
                    pose.point(0.0, 1.0),
                ];
                let _ = s.set_points(frame, Polygon::new(corners).build().points());
                last = Some(pose);
            },
            true,
        )?;
        Ok(Self {
            view,
            blocks,
            cuts,
            frame,
            anchor,
        })
    }

    pub fn block(&self, b: Block) -> Mob {
        self.blocks[BLOCKS.iter().position(|&x| x == b).unwrap_or(0)]
    }

    /// Everything to bring into the scene, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.blocks);
        out.extend(self.cuts);
        out.push(self.frame);
        out
    }

    /// The pose right now.
    pub fn pose(&self, stage: &Stage<'_>) -> SquarePose {
        self.view.pose(stage.arena())
    }

    /// Move all three cuts to chance `p` (one coin), over `rt` seconds.
    pub fn set_coin(&self, stage: &mut Stage<'_>, p: f64, rt: f64) -> fmn::Result<()> {
        tween_all(stage, &[(self.view.x, p), (self.view.y_h, p), (self.view.y_t, p)], rt)
    }

    /// A block's colour, set at once.
    pub fn paint(&self, stage: &mut Stage<'_>, b: Block, color: Srgb, opacity: f64) {
        stage.set_fill(self.block(b), Some(color), Some(opacity), None, false);
    }

    /// A block's colour, as an animation.
    pub fn repaint(&self, b: Block, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
        Ok(self
            .block(b)
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(color), Some(1.0))?)
    }

    /// A live percentage for block `b`, its right edge at `at` ("16%").
    pub fn share_readout(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        b: Block,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let view = self.view;
        let book = std::rc::Rc::clone(&kit.book);
        let m = stage.always_redraw(move |s| {
            let value = 100.0 * view.pose(s).share(b);
            // one decimal only where a whole percent would hide the block (99% heads)
            let places = usize::from(value < 3.0);
            // the engine's number cuts off the digits it does not show, so
            // round here (a share of 15.999… must read 16%)
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
            s.move_to(m, at, RIGHT);
            m
        });
        Ok(m)
    }

    /// A still copy of block `b` as it stands now, for a flip across the
    /// diagonal (the real block stays where it is): the copy, and the same
    /// shape mirrored in the line from the square's lower-left corner to its
    /// upper-right. `Flip::between(&[(copy, mirror)])` is the flip (a linear
    /// `Transform` between them would fold the copy flat on the diagonal).
    pub fn ghost_and_mirror(&self, stage: &mut Stage<'_>, b: Block, color: Srgb) -> [Mob; 2] {
        let pose = self.pose(stage);
        let quad = pose.quad(b);
        let mirror = quad.map(|p| {
            let (u, w) = ((p[0] - pose.left) / pose.side, (p[1] - pose.bottom) / pose.side);
            pose.point(w, u)
        });
        let style = Style::default().fill(color, 0.9).stroke(gold(), 2.5, 1.0);
        let ghost = Polygon::new(quad).style(style).build();
        // the mirror image's corners in the same order, so each corner
        // travels to its own reflection and the shape turns over
        let mirrored = Polygon::new(mirror).style(style).build();
        [stage.arena_mut().add(ghost), stage.arena_mut().add(mirrored)]
    }

    /// The diagonal the two fair blocks mirror in, detached.
    pub fn diagonal(&self, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let pose = self.pose(stage);
        let line = DashedLine::new(pose.point(0.0, 0.0), pose.point(1.0, 1.0))
            .dash_length(0.16)
            .style(line_style(gold(), 3.0))
            .build()
            .map_err(|e| fail(format!("the square's diagonal: {e}")))?;
        Ok(stage.arena_mut().add(line))
    }
}

/// The share of a flip spent lifting the copy off its block.
pub const LIFT_SHARE: f64 = 0.15;
/// How far the copy lifts, away from the diagonal.
pub const LIFT_BY: f64 = 0.14;
/// A frame this close to exactly edge-on (halfway over) is not drawn.
pub const EDGE_ON: f64 = 0.01;

/// How far a flip has lifted and how far it has turned at `t` (0 to 1): the
/// copy lifts in the first [`LIFT_SHARE`], then turns over while it settles.
fn flip_phase(t: f64) -> (f64, f64) {
    let t = t.clamp(0.0, 1.0);
    let lift = smooth((t / LIFT_SHARE).min(1.0));
    let turn = smooth(((t - LIFT_SHARE) / (1.0 - LIFT_SHARE)).clamp(0.0, 1.0));
    (lift, turn)
}

/// Where a point of a flip is at `t`: straight to its own mirror image `b`,
/// lifted up and to the left (away from the diagonal) by up to `lift_by`
/// until it lands.
pub fn flipped_point(a: Vec3, b: Vec3, t: f64, lift_by: f64) -> Vec3 {
    let (lift, e) = flip_phase(t);
    let off = lift_by / 2f64.sqrt() * lift * (1.0 - e);
    v(a[0] + (b[0] - a[0]) * e - off, a[1] + (b[1] - a[1]) * e + off)
}

/// A shape's points at `t` of a flip from `from` to its mirror image `to`
/// (the same corners in the same order), or `None` for the frame that is
/// edge-on. Halfway over, every point of the copy is on the diagonal and its
/// area is nil (it is `|1 - 2e|` of the shape's own), so a polygon set from
/// that frame has nothing to fill: the last shape stays instead.
pub fn flipped(from: &[Vec3], to: &[Vec3], t: f64, lift_by: f64) -> Option<Vec<Vec3>> {
    let (_, e) = flip_phase(t);
    if (e - 0.5).abs() < EDGE_ON {
        return None;
    }
    Some(
        from.iter()
            .zip(to)
            .map(|(&a, &b)| flipped_point(a, b, t, lift_by))
            .collect(),
    )
}

/// A shape that turns over in a flip: the mobject, and its points before
/// and after.
pub type Turning = (Mob, Vec<Vec3>, Vec<Vec3>);
/// A label that travels with a flip and stays upright: the mobject, and its
/// centre before and after.
pub type Riding = (Mob, Vec3, Vec3);

/// One flip across the diagonal (chapters 03, 05 and 08): copies of blocks,
/// and whatever rides with them, drawn from one tracker. Every point travels
/// straight to its own mirror image, so the copy lifts off its block, turns
/// over on the diagonal and lands on the mirror image.
pub struct Flip {
    /// 0: on its own block. 1: on the mirror image.
    turn: Mob,
    /// Carries the updater.
    anchor: Mob,
    /// The shapes first, then what rides with them.
    parts: Vec<Mob>,
}

impl Flip {
    /// The flip of `shapes` (they turn over) and `carried` (they ride along,
    /// upright), lifting by up to `lift_by`. Nothing is shown yet: the flip's
    /// mobjects are `all()`, for `show` to bring in.
    pub fn new(stage: &mut Stage<'_>, shapes: Vec<Turning>, carried: Vec<Riding>, lift_by: f64) -> fmn::Result<Self> {
        let mut parts: Vec<Mob> = shapes.iter().map(|s| s.0).collect();
        parts.extend(carried.iter().map(|c| c.0));
        let turn = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: Option<f64> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(turn).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(t) {
                    return;
                }
                for (m, from, to) in &shapes {
                    if let Some(now) = flipped(from, to, t, lift_by) {
                        let _ = s.set_points(*m, &now);
                    }
                }
                for &(m, from, to) in &carried {
                    s.move_to(m, flipped_point(from, to, t, lift_by), ORIGIN);
                }
                last = Some(t);
            },
            true,
        )?;
        Ok(Self { turn, anchor, parts })
    }

    /// The flip of shapes already on stage onto their mirror images: each
    /// pair is the shape and the same shape mirrored, corner for corner (the
    /// second as `Square::ghost_and_mirror` makes it: detached, read for its
    /// points and never shown). On stage already, the flip is ready to `play`.
    pub fn between(stage: &mut Stage<'_>, pairs: &[(Mob, Mob)], lift_by: f64) -> fmn::Result<Self> {
        let points = |stage: &Stage<'_>, m: Mob| {
            stage
                .get_points(m)
                .ok_or_else(|| fail("a flipped shape has no points".into()))
        };
        let mut shapes = Vec::new();
        for &(copy, mirror) in pairs {
            let (from, to) = (points(stage, copy)?, points(stage, mirror)?);
            if from.len() != to.len() {
                return Err(fail("a shape and its mirror image do not have the same corners".into()));
            }
            shapes.push((copy, from, to));
        }
        let flip = Self::new(stage, shapes, Vec::new(), lift_by)?;
        stage.scene_mut().add(&[flip.anchor])?;
        Ok(flip)
    }

    /// The flip's anchor and everything that moves with it.
    pub fn all(&self) -> Vec<Mob> {
        let mut all = vec![self.anchor];
        all.extend(&self.parts);
        all
    }

    /// Bring the copy in, on its block.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &self.all(), rt)
    }

    /// Lift it, turn it over and land it, `rt` seconds.
    pub fn play(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let a = self
            .turn
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(linear),
                ..AnimateArgs::default()
            })?
            .set_value(1.0)?;
        stage.play(a)?;
        Ok(())
    }

    /// The same flip played by the clock, through whatever comes next (a
    /// listen window has no animation of its own): it starts `delay` seconds
    /// from now and takes `rt` seconds.
    pub fn play_by_the_clock(&self, stage: &mut Stage<'_>, delay: f64, rt: f64) -> fmn::Result<()> {
        let turn = self.turn;
        let mut clock = 0.0;
        stage.add_dt_updater(
            self.anchor,
            move |s, _, dt| {
                clock += dt;
                let _ = s.set_tracker_value(turn, ((clock - delay) / rt).clamp(0.0, 1.0));
            },
            false,
        )?;
        Ok(())
    }

    pub fn hide(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_out_all(stage, &self.all(), rt)
    }
}

/// Where the square sits in the chapters that keep a right-hand column for
/// tiles, counters and bars (02 to 05, 08, 11): its lower-left corner and
/// its side.
pub const SQUARE_AT: Vec3 = [-6.0, -1.55, 0.0];
pub const SQUARE_SIDE: f64 = 4.3;
/// The right-hand column: its left edge and its middle.
pub const COLUMN_LEFT: f64 = -0.9;
pub const COLUMN_MID: f64 = 3.0;

/// The coloured stretches along the square's bottom and left edges (heads
/// blue, tails pink), which follow the cuts, and their words, which are
/// placed for the cuts as they stand when the edges are built.
pub struct Edges {
    /// Carries the updater.
    pub anchor: Mob,
    /// Bottom heads, bottom tails, left heads, left tails.
    pub stretches: [Mob; 4],
    /// "heads" and "tails" under the bottom edge, then beside the left edge.
    pub words: [Mob; 4],
}

impl Edges {
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.stretches);
        out.extend(self.words);
        out
    }
}

impl Square {
    /// The edges for this square as it stands (call it once the cuts are
    /// where the words should sit). Detached.
    pub fn edges(&self, kit: &Kit, stage: &mut Stage<'_>, head: &str, tail: &str) -> fmn::Result<Edges> {
        let view = self.view;
        let colors = [heads(), tails(), heads(), tails()];
        let stretches: [Mob; 4] = std::array::from_fn(|i| {
            stage.arena_mut().add(
                polyline(vec![v(0.0, 0.0), v(1.0, 0.0)])
                    .style(line_style(colors[i], 7.0))
                    .build(),
            )
        });
        let anchor = stage.arena_mut().add(VMobject::new());
        let gap = 0.1;
        let mut last: Option<SquarePose> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let pose = view.pose(s);
                if last == Some(pose) {
                    return;
                }
                let (below, beside) = (pose.bottom - gap, pose.left - gap);
                let x_cut = pose.point(pose.x, 0.0)[0];
                let y_cut = pose.point(0.0, pose.y_h)[1];
                let lines = [
                    [v(pose.left, below), v(x_cut, below)],
                    [v(x_cut, below), v(pose.left + pose.side, below)],
                    [v(beside, pose.bottom), v(beside, y_cut)],
                    [v(beside, y_cut), v(beside, pose.bottom + pose.side)],
                ];
                for (&m, l) in stretches.iter().zip(lines) {
                    // a stretch with no length (a cut at an end) is a dot, not an error
                    let far = if (l[0][0] - l[1][0]).abs() + (l[0][1] - l[1][1]).abs() < 1e-4 {
                        v(l[1][0] + 1e-4, l[1][1] + 1e-4)
                    } else {
                        l[1]
                    };
                    let _ = s.set_points(m, polyline(vec![l[0], far]).build().points());
                }
                last = Some(pose);
            },
            true,
        )?;
        let pose = self.pose(stage);
        let (below, beside) = (pose.bottom - 0.38, pose.left - 0.55);
        let words = [
            kit.text(stage, head, 24.0, heads(), v(pose.point(pose.x / 2.0, 0.0)[0], below))?,
            kit.text(
                stage,
                tail,
                24.0,
                tails(),
                v(pose.point((1.0 + pose.x) / 2.0, 0.0)[0], below),
            )?,
            kit.text(
                stage,
                head,
                24.0,
                heads(),
                v(beside, pose.point(0.0, pose.y_h / 2.0)[1]),
            )?,
            kit.text(
                stage,
                tail,
                24.0,
                tails(),
                v(beside, pose.point(0.0, (1.0 + pose.y_h) / 2.0)[1]),
            )?,
        ];
        Ok(Edges {
            anchor,
            stretches,
            words,
        })
    }

    /// Faint lines at every `1/n` across and up: the small squares a viewer
    /// counts. Detached, as one group.
    pub fn small_squares(&self, stage: &mut Stage<'_>, n: usize) -> fmn::Result<Mob> {
        let pose = self.pose(stage);
        let style = faded_line(Palette::ink(), 1.2, 0.28);
        let mut lines = Vec::new();
        for k in 1..n {
            let t = k as f64 / n as f64;
            for ends in [
                [pose.point(t, 0.0), pose.point(t, 1.0)],
                [pose.point(0.0, t), pose.point(1.0, t)],
            ] {
                lines.push(stage.arena_mut().add(polyline(ends.to_vec()).style(style).build()));
            }
        }
        group(stage, &lines)
    }
}

// ------------------------------------------------------- dots in the square

/// Many small dots as few mobjects: `points` in batches, each batch one
/// group of dots of one colour, detached. A batch is what a reveal brings in
/// at once, so thousands of pairs can land in the square without thousands
/// of animations.
pub fn dot_batches(stage: &mut Stage<'_>, points: &[Vec3], color: Srgb, radius: f64, batches: usize) -> Vec<Mob> {
    let size = points.len().div_ceil(batches.max(1)).max(1);
    points
        .chunks(size)
        .map(|chunk| {
            let dots = chunk
                .iter()
                .map(|&p| Dot::new().point(p).radius(radius).style(flat(color, 0.9)).build());
            stage.arena_mut().add(v_group(dots))
        })
        .collect()
}

// ----------------------------------------------------------------- the cube

/// A box in the cube, in unit coordinates: its low corner and its size along
/// the first, second and third toss (or roll).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxSpec {
    pub low: [f64; 3],
    pub size: [f64; 3],
}

impl BoxSpec {
    pub fn center(&self) -> [f64; 3] {
        [0, 1, 2].map(|i| self.low[i] + self.size[i] / 2.0)
    }
}

/// How the cube is seen: a parallel projection in which the first two
/// tosses stay flat on the screen, exactly as in the square, and the third
/// runs back and up to the right by `depth` (0 shows the square; 1 the whole
/// cube). Parallel, so equal boxes look equal wherever they stand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubePose {
    pub left: f64,
    pub bottom: f64,
    pub side: f64,
    pub depth: f64,
}

/// Where the third side points on screen at full depth, per unit of length.
const DEPTH_DIR: (f64, f64) = (0.46, 0.34);

impl CubePose {
    pub fn project(&self, p: [f64; 3]) -> Vec3 {
        v(
            self.left + self.side * (p[0] + self.depth * DEPTH_DIR.0 * p[2]),
            self.bottom + self.side * (p[1] + self.depth * DEPTH_DIR.1 * p[2]),
        )
    }

    /// How far back a point is (bigger is further from the viewer).
    fn far(&self, p: [f64; 3]) -> f64 {
        p[2] - self.depth * (DEPTH_DIR.0 * p[0] + DEPTH_DIR.1 * p[1])
    }
}

/// One box of the cube on stage: three trackers move it (a shift in unit
/// coordinates, for sliding it out), one turns it about its own centre, two
/// light it and show it, and its six faces are placed by the cube's updater.
#[derive(Clone, Copy, Debug)]
pub struct CubeBox {
    pub spec: BoxSpec,
    pub shift: [Mob; 3],
    /// 0 to 1: how far the box has turned towards `turned` (its sides
    /// permuted), about its own centre, by the smallest rotation that does
    /// it (a box never folds flat on the way).
    pub turn: Mob,
    /// Which of the box's own sides lies along the first, second and third
    /// direction once it has turned (identity: no turn).
    pub turned: [usize; 3],
    /// 0 to 1: from its plain colour to the lit one with a gold edge.
    pub light: Mob,
    /// 0 to 1: how present the box is (1 solid, 0 gone).
    pub shown: Mob,
    /// Its own colour (the cube's `tint` shows it).
    pub color: Srgb,
    pub faces: [Mob; 6],
}

/// The cube's trackers.
#[derive(Clone, Copy, Debug)]
pub struct CubeView {
    pub left: Mob,
    pub bottom: Mob,
    pub side: Mob,
    pub depth: Mob,
}

/// Solid boxes drawn in a parallel projection from trackers: the square with
/// a third side. A face is opaque: its shade is drawn as colour (the box's
/// colour darkened towards the background), so nothing shows through a box.
pub struct Cube {
    pub view: CubeView,
    pub boxes: Vec<CubeBox>,
    pub anchor: Mob,
    /// 1: every box in its own colour; 0: every box in `base`.
    pub tint: Mob,
    /// 1: a lit box wears a gold edge; 0: no gold anywhere.
    pub rim: Mob,
    pub base: Srgb,
    pub lit: Srgb,
}

/// A piece of a cube nobody has named yet, and one that has.
pub fn piece_grey() -> Srgb {
    hex("#3E4A60")
}

pub fn piece_lit() -> Srgb {
    hex("#8191AE")
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

/// The background, for shading towards.
fn ground() -> Srgb {
    hex("#0E1116")
}

/// A unit box's eight corners, bit `i` of the index choosing the high end
/// along direction `i`.
fn corner(low: [f64; 3], size: [f64; 3], k: usize) -> [f64; 3] {
    [0, 1, 2].map(|i| low[i] + if k >> i & 1 == 1 { size[i] } else { 0.0 })
}

/// The six faces as corner indices (counter-clockwise from outside) and the
/// direction each one faces: (axis, +1 or −1).
const FACES: [([usize; 4], (usize, f64)); 6] = [
    ([0, 2, 6, 4], (0, -1.0)),
    ([1, 5, 7, 3], (0, 1.0)),
    ([0, 4, 5, 1], (1, -1.0)),
    ([2, 3, 7, 6], (1, 1.0)),
    ([0, 1, 3, 2], (2, -1.0)),
    ([4, 6, 7, 5], (2, 1.0)),
];

/// How bright a face is drawn, by the direction it faces: the front (as the
/// square was) full, the top a little lighter, the right side darker.
fn face_shade(normal: [f64; 3]) -> f64 {
    (0.8 - 0.22 * normal[0] + 0.2 * normal[1] - 0.12 * normal[2]).clamp(0.45, 1.0)
}

impl Cube {
    /// A cube whose front face is the square at `corner` with side `side`,
    /// holding `specs` (each with its fill colour). Detached. Boxes are drawn
    /// back to front in the order they stand in the cube, so add
    /// [`Cube::mobs`] in the order returned.
    pub fn build(
        stage: &mut Stage<'_>,
        corner_at: Vec3,
        side: f64,
        depth: f64,
        specs: &[(BoxSpec, Srgb)],
    ) -> fmn::Result<Self> {
        let view = CubeView {
            left: stage.add_value_tracker(corner_at[0]),
            bottom: stage.add_value_tracker(corner_at[1]),
            side: stage.add_value_tracker(side),
            depth: stage.add_value_tracker(depth),
        };
        // furthest first: with this projection a box further back, lower or
        // further left is hidden by its neighbours
        let reference = CubePose {
            left: 0.0,
            bottom: 0.0,
            side: 1.0,
            depth: 1.0,
        };
        let mut order: Vec<usize> = (0..specs.len()).collect();
        order.sort_by(|&a, &b| {
            reference
                .far(specs[b].0.center())
                .total_cmp(&reference.far(specs[a].0.center()))
        });
        let mut boxes = Vec::with_capacity(specs.len());
        for &i in &order {
            let (spec, color) = specs[i];
            let faces: [Mob; 6] = std::array::from_fn(|_| {
                stage.arena_mut().add(
                    Polygon::new(unit_quad())
                        .style(Style::default().fill(color, 1.0).stroke(ground(), 1.6, 0.9))
                        .build(),
                )
            });
            boxes.push(CubeBox {
                spec,
                shift: [0, 1, 2].map(|_| stage.add_value_tracker(0.0)),
                turn: stage.add_value_tracker(0.0),
                turned: [0, 1, 2],
                light: stage.add_value_tracker(0.0),
                shown: stage.add_value_tracker(1.0),
                color,
                faces,
            });
        }
        let anchor = stage.arena_mut().add(VMobject::new());
        Ok(Self {
            view,
            boxes,
            anchor,
            tint: stage.add_value_tracker(1.0),
            rim: stage.add_value_tracker(1.0),
            base: piece_grey(),
            lit: piece_lit(),
        })
    }

    /// The plain and the lit colour of every box (default `piece_grey` and
    /// `piece_lit`). Call before [`Cube::install`].
    pub fn set_look(&mut self, base: Srgb, lit: Srgb) {
        self.base = base;
        self.lit = lit;
    }

    /// The box built from `spec` (boxes are stored back to front, not in
    /// the order given).
    pub fn find(&self, spec: BoxSpec) -> Option<usize> {
        self.boxes.iter().position(|b| b.spec == spec)
    }

    /// Say how box `i` ends up when its `turn` tracker reaches 1: which of
    /// its own sides lies along each direction. Call before [`Cube::install`].
    pub fn set_turn(&mut self, i: usize, turned: [usize; 3]) {
        self.boxes[i].turned = turned;
    }

    /// Start drawing: one updater on the anchor places every face from the
    /// trackers, and only when one has moved.
    pub fn install(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let view = self.view;
        let boxes = self.boxes.clone();
        let rotations: Vec<Rotation> = boxes.iter().map(|b| Rotation::of(b.turned)).collect();
        let (tint, rim, base, lit) = (self.tint, self.rim, self.base, self.lit);
        let mut last: Option<Vec<f64>> = None;
        stage.add_updater(
            self.anchor,
            move |s, _| {
                let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let mut now = vec![
                    val(view.left),
                    val(view.bottom),
                    val(view.side),
                    val(view.depth),
                    val(tint),
                    val(rim),
                ];
                for b in &boxes {
                    now.extend(b.shift.map(val));
                    now.extend([val(b.turn), val(b.light), val(b.shown)]);
                }
                if last.as_ref() == Some(&now) {
                    return;
                }
                let pose = CubePose {
                    left: now[0],
                    bottom: now[1],
                    side: now[2],
                    depth: now[3],
                };
                let (tint, rim) = (now[4], now[5].clamp(0.0, 1.0));
                for (k, b) in boxes.iter().enumerate() {
                    let at = 6 + 6 * k;
                    let shift = [now[at], now[at + 1], now[at + 2]];
                    let (turn, light, shown) = (now[at + 3], now[at + 4].clamp(0.0, 1.0), now[at + 5]);
                    let look = Look {
                        paint: mix(mix(base, b.color, tint), lit, light),
                        edge: mix(ground(), gold(), light * rim),
                        width: 1.6 + 1.4 * light * rim,
                        shown: shown.clamp(0.0, 1.0),
                    };
                    place_box(s, &pose, b, &rotations[k], shift, turn, look);
                }
                last = Some(now);
            },
            true,
        )?;
        Ok(())
    }

    /// Everything to add to the scene, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        for b in &self.boxes {
            out.extend(b.faces);
        }
        out
    }

    /// The faces of box `i`.
    pub fn faces(&self, i: usize) -> [Mob; 6] {
        self.boxes[i].faces
    }
}

/// How one box is painted this frame.
#[derive(Clone, Copy)]
struct Look {
    paint: Srgb,
    edge: Srgb,
    width: f64,
    shown: f64,
}

/// The turn that takes a box from its place in the cube to `turned`, as a
/// rotation: a unit quaternion (w, x, y, z). A box looks the same with any of
/// its sides reversed, so of the rotations that land each side on its new
/// direction (either way along it) this is the one through the smallest
/// angle; a turn interpolated along it keeps the box a box (a side-by-side
/// slerp would fold it flat halfway through a swap).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rotation {
    q: [f64; 4],
}

impl Rotation {
    fn of(turned: [usize; 3]) -> Self {
        // column `own` of the matrix: where the box's own side goes
        let dest = |own: usize| turned.iter().position(|&x| x == own).unwrap_or(own);
        let mut best: Option<([[f64; 3]; 3], f64)> = None;
        for signs in 0..8u32 {
            let mut m = [[0.0; 3]; 3];
            for own in 0..3 {
                m[dest(own)][own] = if signs >> own & 1 == 1 { -1.0 } else { 1.0 };
            }
            let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
            if det < 0.0 {
                continue;
            }
            let trace = m[0][0] + m[1][1] + m[2][2];
            if best.is_none_or(|(_, t)| trace > t + 1e-9) {
                best = Some((m, trace));
            }
        }
        let (m, _) = best.unwrap_or(([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], 3.0));
        Self { q: quaternion(m) }
    }

    /// `v` turned `t` of the way (0 to 1).
    fn apply(&self, v: [f64; 3], t: f64) -> [f64; 3] {
        let [w, x, y, z] = self.q;
        let angle = 2.0 * w.clamp(-1.0, 1.0).acos();
        let s = (angle / 2.0).sin();
        if s.abs() < 1e-9 {
            return v;
        }
        let axis = [x / s, y / s, z / s];
        let half = angle * t.clamp(0.0, 1.0) / 2.0;
        let (qw, sh) = (half.cos(), half.sin());
        let a = axis.map(|c| c * sh);
        let cross = |p: [f64; 3], q: [f64; 3]| {
            [
                p[1] * q[2] - p[2] * q[1],
                p[2] * q[0] - p[0] * q[2],
                p[0] * q[1] - p[1] * q[0],
            ]
        };
        let av = cross(a, v);
        let aav = cross(a, av);
        [0, 1, 2].map(|i| v[i] + 2.0 * qw * av[i] + 2.0 * aav[i])
    }
}

/// A rotation matrix (columns are images of the axes) as a unit quaternion
/// with w ≥ 0 (Shepperd's method).
fn quaternion(m: [[f64; 3]; 3]) -> [f64; 4] {
    let tr = m[0][0] + m[1][1] + m[2][2];
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [
            0.25 * s,
            (m[2][1] - m[1][2]) / s,
            (m[0][2] - m[2][0]) / s,
            (m[1][0] - m[0][1]) / s,
        ]
    } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
        let s = (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 2.0;
        [
            (m[2][1] - m[1][2]) / s,
            0.25 * s,
            (m[0][1] + m[1][0]) / s,
            (m[0][2] + m[2][0]) / s,
        ]
    } else if m[1][1] > m[2][2] {
        let s = (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt() * 2.0;
        [
            (m[0][2] - m[2][0]) / s,
            (m[0][1] + m[1][0]) / s,
            0.25 * s,
            (m[1][2] + m[2][1]) / s,
        ]
    } else {
        let s = (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt() * 2.0;
        [
            (m[1][0] - m[0][1]) / s,
            (m[0][2] + m[2][0]) / s,
            (m[1][2] + m[2][1]) / s,
            0.25 * s,
        ]
    };
    if q[0] < 0.0 { q.map(|c| -c) } else { q }
}

/// Place one box's faces: its corners, turned `turn` of the way towards its
/// final orientation about its own centre, shifted, projected, and painted
/// solid; faces that point away from the viewer are hidden.
fn place_box(
    s: &mut fmn::mobject::Stage,
    pose: &CubePose,
    b: &CubeBox,
    rotation: &Rotation,
    shift: [f64; 3],
    turn: f64,
    look: Look,
) {
    let c = b.spec.center();
    let half = b.spec.size.map(|x| x / 2.0);
    // the box's own three half-sides, as vectors, turned together
    let mut sides = [[0.0; 3]; 3];
    for (own, side) in sides.iter_mut().enumerate() {
        let mut dir = [0.0; 3];
        dir[own] = half[own];
        *side = rotation.apply(dir, turn);
    }
    let point = |k: usize| -> [f64; 3] {
        let mut p = [c[0] + shift[0], c[1] + shift[1], c[2] + shift[2]];
        for (own, side) in sides.iter().enumerate() {
            let sign = if k >> own & 1 == 1 { 1.0 } else { -1.0 };
            for i in 0..3 {
                p[i] += sign * side[i];
            }
        }
        p
    };
    for (&m, (corners, (axis, sign))) in b.faces.iter().zip(FACES) {
        // the face's outward direction: its own side's vector
        let n = sides[axis].map(|x| x * sign);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
        let n = n.map(|x| x / len);
        // it faces the viewer when it points against the line of sight
        let away = n[2] - pose.depth * (DEPTH_DIR.0 * n[0] + DEPTH_DIR.1 * n[1]);
        let seen = away < -1e-6;
        let quad = corners.map(|k| pose.project(point(k)));
        let _ = s.set_points(m, Polygon::new(quad).build().points());
        // shading grows with the depth: a cube with no depth is the square
        let shade = 1.0 - pose.depth.clamp(0.0, 1.0) * (1.0 - face_shade(n));
        let fill = mix(ground(), look.paint, shade);
        let (fill_o, line_o) = if seen {
            (look.shown, 0.9 * look.shown)
        } else {
            (0.0, 0.0)
        };
        s.set_fill(m, Some(fill), Some(fill_o), None, false);
        s.set_stroke(m, Some(look.edge), Some(look.width), Some(line_o), None, false);
    }
}

/// From unit vector `a` to unit vector `b` along the great circle, `t` of
/// the way (the two are equal or at right angles here).
fn slerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    if dot > 0.999 {
        return a;
    }
    let angle = dot.clamp(-1.0, 1.0).acos() * t.clamp(0.0, 1.0);
    // b's part at right angles to a
    let mut w = [b[0] - dot * a[0], b[1] - dot * a[1], b[2] - dot * a[2]];
    let len = (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]).sqrt().max(1e-9);
    w = w.map(|x| x / len);
    [0, 1, 2].map(|i| a[i] * angle.cos() + w[i] * angle.sin())
}

/// The boxes of a cube cut at `cuts` along each direction (each list the
/// running totals between 0 and 1, both ends included), with each box's
/// place: (index along the first, second, third direction).
pub fn cut_boxes(cuts: [&[f64]; 3]) -> Vec<(BoxSpec, [usize; 3])> {
    let mut out = Vec::new();
    for i in 0..cuts[0].len() - 1 {
        for j in 0..cuts[1].len() - 1 {
            for k in 0..cuts[2].len() - 1 {
                out.push((
                    BoxSpec {
                        low: [cuts[0][i], cuts[1][j], cuts[2][k]],
                        size: [
                            cuts[0][i + 1] - cuts[0][i],
                            cuts[1][j + 1] - cuts[1][j],
                            cuts[2][k + 1] - cuts[2][k],
                        ],
                    },
                    [i, j, k],
                ));
            }
        }
    }
    out
}

// --------------------------------------------- the glass box and its edges

/// The side of the cube once it stands (06 and 09): its front face's
/// lower-left corner stays at the square's home, `SQUARE_AT`.
pub const CUBE_SIDE: f64 = 3.0;

/// Where a cube's third edge lies on screen, away from the cube: at right
/// angles to the third direction, down and to the right.
fn third_out() -> (f64, f64) {
    let len = DEPTH_DIR.0.hypot(DEPTH_DIR.1);
    (DEPTH_DIR.1 / len, -DEPTH_DIR.0 / len)
}

fn cube_pose(s: &fmn::mobject::Stage, view: CubeView) -> CubePose {
    let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
    CubePose {
        left: val(view.left),
        bottom: val(view.bottom),
        side: val(view.side),
        depth: val(view.depth),
    }
}

impl CubeView {
    /// This frame's pose.
    pub fn pose(&self, stage: &Stage<'_>) -> CubePose {
        cube_pose(stage.arena(), *self)
    }
}

/// The glass box a cube's pieces stand in: its three far faces (the back,
/// the left side and the floor) as dark panels carrying the cuts, their
/// edges, and the three edges nearest the viewer drawn faintly, all
/// following the cube's trackers. Add it before the cube's pieces.
pub struct CubeRoom {
    pub anchor: Mob,
    /// The back, the left side, the floor.
    pub walls: [Mob; 3],
    pub cuts: Vec<Mob>,
    pub edges: Vec<Mob>,
    pub near: Vec<Mob>,
}

impl CubeRoom {
    /// The room for a cube on `view`, cut at `cuts` along each direction (the
    /// running totals from 0 to 1, as for [`cut_boxes`]). Detached.
    pub fn build(stage: &mut Stage<'_>, view: CubeView, cuts: [&[f64]; 3]) -> fmn::Result<Self> {
        let wall_quads = [
            [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
            [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        ];
        let tones = ["#1B2332", "#161D29", "#222C3D"];
        let walls: [Mob; 3] = std::array::from_fn(|i| {
            stage
                .arena_mut()
                .add(Polygon::new(unit_quad()).style(flat(hex(tones[i]), 1.0)).build())
        });
        let inner = |a: usize| cuts[a].get(1..cuts[a].len().saturating_sub(1)).unwrap_or(&[]).to_vec();
        let mut cut_ends: Vec<[[f64; 3]; 2]> = Vec::new();
        for c in inner(0) {
            cut_ends.push([[c, 0.0, 1.0], [c, 1.0, 1.0]]);
            cut_ends.push([[c, 0.0, 0.0], [c, 0.0, 1.0]]);
        }
        for c in inner(1) {
            cut_ends.push([[0.0, c, 1.0], [1.0, c, 1.0]]);
            cut_ends.push([[0.0, c, 0.0], [0.0, c, 1.0]]);
        }
        for c in inner(2) {
            cut_ends.push([[0.0, 0.0, c], [0.0, 1.0, c]]);
            cut_ends.push([[0.0, 0.0, c], [1.0, 0.0, c]]);
        }
        // the edges of the three far faces, then the three nearest the viewer
        let corner = |k: usize| [0, 1, 2].map(|a| f64::from(u8::from(k >> a & 1 == 1)));
        let (mut edge_ends, mut near_ends) = (Vec::new(), Vec::new());
        for k in 0..8usize {
            for a in 0..3 {
                if k >> a & 1 == 1 {
                    continue;
                }
                let ends = [corner(k), corner(k | 1 << a)];
                let far = ends.iter().all(|p| p[2] == 1.0)
                    || ends.iter().all(|p| p[0] == 0.0)
                    || ends.iter().all(|p| p[1] == 0.0);
                if far {
                    edge_ends.push(ends);
                } else {
                    near_ends.push(ends);
                }
            }
        }
        let mut lines = |n: usize, style: Style| -> Vec<Mob> {
            (0..n)
                .map(|_| {
                    stage
                        .arena_mut()
                        .add(polyline(vec![v(0.0, 0.0), v(1.0, 0.0)]).style(style).build())
                })
                .collect()
        };
        let cut_mobs = lines(cut_ends.len(), line_style(hex("#465269"), 1.5));
        let edge_mobs = lines(edge_ends.len(), faded_line(Palette::dim(), 2.0, 0.85));
        let near_mobs = lines(near_ends.len(), faded_line(Palette::dim(), 1.5, 0.35));
        let all_lines: Vec<(Mob, [[f64; 3]; 2])> = cut_mobs
            .iter()
            .chain(&edge_mobs)
            .chain(&near_mobs)
            .copied()
            .zip(cut_ends.into_iter().chain(edge_ends).chain(near_ends))
            .collect();
        let quads: Vec<(Mob, [[f64; 3]; 4])> = walls.iter().copied().zip(wall_quads).collect();
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last: Option<CubePose> = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let mut pose = cube_pose(s, view);
                // never quite flat: a face with no width is not a shape
                pose.depth = pose.depth.max(0.004);
                if last == Some(pose) {
                    return;
                }
                for &(m, quad) in &quads {
                    let _ = s.set_points(m, Polygon::new(quad.map(|p| pose.project(p))).build().points());
                }
                for &(m, ends) in &all_lines {
                    let _ = s.set_points(m, polyline(ends.map(|p| pose.project(p)).to_vec()).build().points());
                }
                last = Some(pose);
            },
            true,
        )?;
        Ok(Self {
            anchor,
            walls,
            cuts: cut_mobs,
            edges: edge_mobs,
            near: near_mobs,
        })
    }

    /// Everything, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.walls);
        out.extend(self.cuts.iter().chain(&self.edges).chain(&self.near));
        out
    }
}

/// One edge of a [`CubeRuler`]: its stretches (from, to, colour) along the
/// direction from the front corner, a word for each stretch (or none), and
/// the edge's name.
pub struct RulerEdge {
    pub stretches: Vec<(f64, f64, Srgb)>,
    pub words: Vec<Option<String>>,
    /// The words' colour; `None` paints each in its stretch's colour.
    pub word_color: Option<Srgb>,
    pub word_size: f64,
    /// How far beside the second edge its words sit (the square's own
    /// "heads" and "tails" sit 0.55 out; digits 0.38).
    pub word_out: f64,
    pub name: Option<String>,
}

/// Coloured stretches just outside a cube's three nearest edges, as the
/// square has along two (the same 7-point bar, 0.1 out): under the front's
/// bottom edge (the first toss or roll), beside its left edge (the second),
/// and along the floor's right edge going back (the third), which grows as
/// the cube gains depth. Words and names follow the cube's trackers.
pub struct RulerParts {
    pub anchor: Mob,
    pub stretches: [Vec<Mob>; 3],
    pub words: [Vec<Mob>; 3],
    pub names: Vec<Mob>,
}

impl RulerParts {
    /// Everything, back to front.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.stretches.iter().flatten());
        out.extend(self.words.iter().flatten());
        out.extend(&self.names);
        out
    }
}

/// Build the ruler for a cube on `view`; `gap` (scene units) is left at each
/// end of every stretch (0: one unbroken bar, as the square's). Detached.
pub fn cube_ruler(
    kit: &Kit,
    stage: &mut Stage<'_>,
    view: CubeView,
    edges: [RulerEdge; 3],
    gap: f64,
) -> fmn::Result<RulerParts> {
    let mut stretches: [Vec<Mob>; 3] = Default::default();
    let mut words: [Vec<Mob>; 3] = Default::default();
    let mut names = Vec::new();
    // (mob, edge, from, to) for the bars; (mob, edge, at along, align) for text
    let mut bar_spots: Vec<(Mob, usize, f64, f64)> = Vec::new();
    let mut text_spots: Vec<(Mob, usize, f64, f64, bool)> = Vec::new();
    for (a, edge) in edges.iter().enumerate() {
        for (i, &(from, to, color)) in edge.stretches.iter().enumerate() {
            let bar = polyline(vec![v(0.0, 0.0), v(1.0, 0.0)])
                .style(line_style(color, 7.0))
                .build();
            let bar = stage.arena_mut().add(bar);
            stretches[a].push(bar);
            bar_spots.push((bar, a, from, to));
            if let Some(Some(word)) = edge.words.get(i) {
                let ink = edge.word_color.unwrap_or(color);
                let m = kit.text(stage, word, edge.word_size, ink, ORIGIN)?;
                words[a].push(m);
                text_spots.push((m, a, (from + to) / 2.0, edge.word_out, false));
            }
        }
        if let Some(name) = &edge.name {
            let m = kit.text(stage, name, 22.0, Palette::dim(), ORIGIN)?;
            names.push(m);
            text_spots.push((m, a, 0.5, -1.0, true));
        }
    }
    let anchor = stage.arena_mut().add(VMobject::new());
    let (ox, oy) = third_out();
    let mut last: Option<CubePose> = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let pose = cube_pose(s, view);
            // the bars only when the cube has moved; the words every frame,
            // since a fade that brings one in would otherwise leave it where
            // the fade began
            let moved = last != Some(pose);
            // a point along edge `a`, `t` of the way from the front corner,
            // and the way out from that edge (unit)
            let along = |a: usize, t: f64| -> Vec3 {
                match a {
                    0 => pose.project([t, 0.0, 0.0]),
                    1 => pose.project([0.0, t, 0.0]),
                    _ => pose.project([1.0, 0.0, t]),
                }
            };
            let out = |a: usize| -> (f64, f64) {
                match a {
                    0 => (0.0, -1.0),
                    1 => (-1.0, 0.0),
                    _ => (ox, oy),
                }
            };
            for &(m, a, from, to) in bar_spots.iter().filter(|_| moved) {
                let (p, q) = (along(a, from), along(a, to));
                let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
                let len = dx.hypot(dy);
                let trim = if len > 2.0 * gap + 1e-3 { gap / len } else { 0.0 };
                let (u, w) = out(a);
                let end = |f: f64| v(p[0] + dx * f + 0.1 * u, p[1] + dy * f + 0.1 * w);
                let (start, mut finish) = (end(trim), end(1.0 - trim));
                // a stretch with no length (the third edge of a flat cube) is a dot
                if len < 1e-4 {
                    finish = v(start[0] + 1e-4, start[1] + 1e-4);
                }
                let _ = s.set_points(m, polyline(vec![start, finish]).build().points());
            }
            for &(m, a, t, word_out, is_name) in &text_spots {
                let p = along(a, t);
                let (u, w) = out(a);
                let (at, edge) = match (a, is_name) {
                    (0, false) => (v(p[0], pose.bottom - 0.38), ORIGIN),
                    (0, true) => (v(p[0], pose.bottom - 0.8), ORIGIN),
                    (1, false) => (v(pose.left - word_out, p[1]), ORIGIN),
                    (1, true) => (v(pose.left - 0.95, pose.project([0.0, 1.0, 0.0])[1] + 0.32), LEFT),
                    (_, false) => (v(p[0] + 0.45 * u, p[1] + 0.45 * w), ORIGIN),
                    (_, true) => (v(p[0] + 0.72 * u, p[1] + 0.72 * w), LEFT),
                };
                s.move_to(m, at, edge);
            }
            last = Some(pose);
        },
        true,
    )?;
    Ok(RulerParts {
        anchor,
        stretches,
        words,
        names,
    })
}

// ------------------------------------------------- the row for four tosses

/// From one group's centre to the next in a row of groups, and a group's box.
pub const ROW_PITCH: f64 = 2.6;
pub const ROW_BOX_W: f64 = 2.5;
/// The row for four tosses (06_6 to 07_4): the height of its boxes' centres,
/// their height, where the counts ("4 heads") sit over them and the group
/// sizes under them.
pub const FOUR_ROW_Y: f64 = -1.65;
pub const FOUR_ROW_BOX_H: f64 = 1.2;
pub const FOUR_ROW_COUNT_Y: f64 = -0.82;
pub const FOUR_ROW_SIZE_Y: f64 = -2.55;
/// The bars over the row: their foot, their width, and how tall a chance of
/// one would be.
pub const FOUR_ROW_BAR_FOOT: f64 = -0.55;
pub const FOUR_ROW_BAR_W: f64 = 1.3;
pub const FOUR_ROW_BAR_UNIT: f64 = 7.4;

const HEADS_WORD: &str = "heads";
const HEAD_WORD: &str = "head";
const TOSSES_WORD: &str = "tosses";

/// The x of group `k` (how many tails; all heads on the left) in the row for
/// `n` tosses, the rows centred on x = 0.
pub fn row_x(n: usize, k: usize) -> f64 {
    ROW_PITCH * (k as f64 - n as f64 / 2.0)
}

/// "3 heads", "1 head", "0 heads".
pub fn heads_count(heads: usize) -> String {
    format!("{heads} {}", if heads == 1 { HEAD_WORD } else { HEADS_WORD })
}

/// A group's box: a rounded panel, drawn behind whatever stands in it.
/// Detached.
pub fn group_box(stage: &mut Stage<'_>, at: Vec3, height: f64) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(ROW_BOX_W)
        .height(height)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(back))
}

/// A pattern of four tosses as a small tile: its discs on a back, centred at
/// `at`. Detached, one group (the back first).
pub fn pattern_tile(stage: &mut Stage<'_>, pattern: &[Face], at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(1.04)
        .height(0.3)
        .corner_radius(0.07)
        .style(flat(hex("#2A3447"), 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    let step = 0.23;
    let left = v(at[0] - step * (pattern.len() as f64 - 1.0) / 2.0, at[1]);
    parts.extend(disc_row(stage, pattern, left, 0.09, step));
    group(stage, &parts)
}

/// The row for four tosses, one drawing for chapters 06 and 07: five boxes
/// side by side, one per count of heads (four heads on the left), each
/// holding its patterns as tiles (two to a line); the counts over the boxes,
/// the group sizes (Pascal's row, `model::pascal_row`) under them, and "4
/// tosses" to the left of the sizes. Index `k` is the number of tails.
pub struct FourRow {
    pub boxes: [Mob; 5],
    /// Each group's patterns, in `model::patterns` order.
    pub tiles: [Vec<Mob>; 5],
    pub patterns: [Vec<Vec<Face>>; 5],
    /// "4 heads" … "0 heads".
    pub counts: [Mob; 5],
    /// 1, 4, 6, 4, 1.
    pub sizes: [Mob; 5],
    /// "4 tosses".
    pub name: Mob,
}

impl FourRow {
    /// Build the row at its one place (`FOUR_ROW_*`). Detached: bring in
    /// [`FourRow::mobs`] (back to front), or any part of it.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let n = 4usize;
        let sizes_q = crate::model::pascal_row(n as u32);
        let mut boxes = Vec::new();
        let mut tiles: [Vec<Mob>; 5] = Default::default();
        let mut patterns: [Vec<Vec<Face>>; 5] = Default::default();
        let mut counts = Vec::new();
        let mut sizes = Vec::new();
        for k in 0..=n {
            let at = Self::center(k);
            boxes.push(group_box(stage, at, FOUR_ROW_BOX_H)?);
            let group_patterns = crate::model::patterns(n as u32, (n - k) as u32);
            let cols = group_patterns.len().min(2);
            let lines = group_patterns.len().div_ceil(cols);
            for (i, pattern) in group_patterns.iter().enumerate() {
                let dx = 1.12 * ((i % cols) as f64 - (cols as f64 - 1.0) / 2.0);
                let dy = 0.36 * ((lines as f64 - 1.0) / 2.0 - (i / cols) as f64);
                tiles[k].push(pattern_tile(stage, pattern, v(at[0] + dx, at[1] + dy))?);
            }
            patterns[k] = group_patterns;
            let count = heads_count(n - k);
            counts.push(kit.text(stage, &count, 20.0, Palette::dim(), v(at[0], FOUR_ROW_COUNT_Y))?);
            let size = sizes_q[n - k].to_string();
            sizes.push(kit.text(stage, &size, 40.0, Palette::ink(), v(at[0], FOUR_ROW_SIZE_Y))?);
        }
        let name = format!("{n} {TOSSES_WORD}");
        let name = kit.text(stage, &name, 22.0, Palette::dim(), v(0.0, FOUR_ROW_SIZE_Y))?;
        stage.move_to(name, v(row_x(n, 0) - 0.45, FOUR_ROW_SIZE_Y), RIGHT);
        let five = |v: Vec<Mob>| -> fmn::Result<[Mob; 5]> {
            v.try_into()
                .map_err(|_| fail("the row for four tosses has five groups".into()))
        };
        Ok(Self {
            boxes: five(boxes)?,
            tiles,
            patterns,
            counts: five(counts)?,
            sizes: five(sizes)?,
            name,
        })
    }

    /// The centre of box `k` on screen.
    pub fn center(k: usize) -> Vec3 {
        v(row_x(4, k), FOUR_ROW_Y)
    }

    /// The height of the top edge of every box.
    pub fn top() -> f64 {
        FOUR_ROW_Y + FOUR_ROW_BOX_H / 2.0
    }

    /// Every tile, group by group.
    pub fn all_tiles(&self) -> Vec<Mob> {
        self.tiles.iter().flatten().copied().collect()
    }

    /// Everything, back to front: boxes, tiles, counts, sizes, name.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = self.boxes.to_vec();
        out.extend(self.all_tiles());
        out.extend(self.counts);
        out.extend(self.sizes);
        out.push(self.name);
        out
    }

    /// The bars over the boxes for a coin whose chance of heads is the
    /// tracker `chance`: bar `k` is the chance of landing in group `k`, and
    /// its label prints it to two decimals ("40.96%"), computed exactly by
    /// the model at the tracker's value taken to a whole percent (the chance
    /// a slider beside it shows), never read off the bar. Detached; see
    /// [`RowBars`].
    pub fn bars(&self, kit: &Kit, stage: &mut Stage<'_>, chance: Mob) -> fmn::Result<RowBars> {
        let bars: [Mob; 5] = std::array::from_fn(|k| {
            let bar = column(v(row_x(4, k), FOUR_ROW_BAR_FOOT), FOUR_ROW_BAR_W, 0.004, Palette::dim());
            stage.arena_mut().add(bar)
        });
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let c = s.tracker_value(chance).unwrap_or(0.0);
                if last == Some(c) {
                    return;
                }
                for (k, &m) in bars.iter().enumerate() {
                    let h = FOUR_ROW_BAR_UNIT * group_chance(c, k).f64();
                    let bar = column(v(row_x(4, k), FOUR_ROW_BAR_FOOT), FOUR_ROW_BAR_W, h, Palette::dim());
                    let _ = s.set_points(m, bar.points());
                }
                last = Some(c);
            },
            true,
        )?;
        let labels: [Mob; 5] = std::array::from_fn(|k| {
            let book = std::rc::Rc::clone(&kit.book);
            stage.always_redraw(move |s| {
                let c = s.tracker_value(chance).unwrap_or(0.0);
                let q = group_chance(c, k);
                // the printed chance is the model's at the whole percent a
                // slider shows, so 50% reads 6.25% even as the slider leaves it
                let shown = group_chance((c * 100.0).round() / 100.0, k);
                let value: f64 = shown.percent(2).parse().unwrap_or(0.0);
                let d = DecimalNumber::new(value)
                    .num_decimal_places(2)
                    .unit("%")
                    .font_size(22.0)
                    .color(Palette::ink())
                    .build(&book)
                    .expect("bundled digits always lay out");
                let m = s.add(d.into_vmob());
                let top = FOUR_ROW_BAR_FOOT + FOUR_ROW_BAR_UNIT * q.f64();
                s.move_to(m, v(row_x(4, k), top + 0.1), DOWN);
                m
            })
        });
        Ok(RowBars {
            chance,
            anchor,
            bars,
            labels,
        })
    }
}

/// The chance of landing in group `k` (k tails in four tosses) for a coin
/// with chance `c` of heads, exactly, with `c` taken to a hundredth of a
/// percent.
pub fn group_chance(c: f64, k: usize) -> crate::model::Q {
    let p = crate::model::Q::new((c.clamp(0.0, 1.0) * 10_000.0).round() as i128, 10_000);
    crate::model::count_chance(p, 4, (4 - k.min(4)) as u32)
}

/// The bars over a [`FourRow`]: one updater (on `anchor`) sizes all five from
/// the `chance` tracker. Bring in [`RowBars::mobs`] with a fade; the labels
/// are live readouts, so add them with `stage.add_to_scene` when they should
/// show (they appear at once), and fade them with `fade_out_all` (which
/// clears their updaters first).
pub struct RowBars {
    pub chance: Mob,
    pub anchor: Mob,
    pub bars: [Mob; 5],
    pub labels: [Mob; 5],
}

impl RowBars {
    /// The anchor and the bars, back to front (not the labels).
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(self.bars);
        out
    }

    /// The top of bar `k` on screen at chance `c` of heads.
    pub fn top(k: usize, c: f64) -> f64 {
        FOUR_ROW_BAR_FOOT + FOUR_ROW_BAR_UNIT * group_chance(c, k).f64()
    }
}

// ---------------------------------------------------------------------- dice

/// A die face: a rounded square with its pips, centred at `at`, `size`
/// across. Detached, as one group.
pub fn die_face(stage: &mut Stage<'_>, value: usize, at: Vec3, size: f64, back: Srgb, pip: Srgb) -> fmn::Result<Mob> {
    let body = Rectangle::new()
        .width(size)
        .height(size)
        .corner_radius(size * 0.18)
        .style(flat(back, 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(body)];
    let d = size * 0.27;
    let spots: &[(f64, f64)] = match value {
        1 => &[(0.0, 0.0)],
        2 => &[(-1.0, 1.0), (1.0, -1.0)],
        3 => &[(-1.0, 1.0), (0.0, 0.0), (1.0, -1.0)],
        4 => &[(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)],
        5 => &[(-1.0, 1.0), (1.0, 1.0), (0.0, 0.0), (-1.0, -1.0), (1.0, -1.0)],
        _ => &[
            (-1.0, 1.0),
            (1.0, 1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, -1.0),
            (1.0, -1.0),
        ],
    };
    for &(dx, dy) in spots {
        let p = Dot::new()
            .point(v(at[0] + d * dx, at[1] + d * dy))
            .radius(size * 0.085)
            .style(flat(pip, 1.0))
            .build();
        parts.push(stage.arena_mut().add(p));
    }
    group(stage, &parts)
}

// ------------------------------------------------------------------- quotes

/// A verbatim quote on a card, wrapped to `chars` a line, with its source
/// beneath on up to two lines. It never drops words: text that does not fit
/// in `max_lines` is an error, not an ellipsis. Returns the card's back, the
/// quote's lines and the source's lines, all detached.
pub struct QuoteCard {
    pub back: Mob,
    pub lines: Vec<Mob>,
    pub source: Vec<Mob>,
}

impl QuoteCard {
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        words: &str,
        source: &str,
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
        let src = wrap(source, chars + 14, 3);
        if src.len() > 2 {
            return Err(fail(format!("the source line does not fit two lines: {source}")));
        }
        let (line_h, src_h) = (0.44, 0.32);
        let h = 0.5 + line_h * lines.len() as f64 + 0.2 + src_h * src.len() as f64;
        let top = at[1] + h / 2.0 - 0.3 - line_h / 2.0;
        let mut quote = Vec::new();
        let mut widest: f64 = 0.0;
        for (i, l) in lines.iter().enumerate() {
            let m = kit.text_styled(
                stage,
                l,
                28.0,
                Palette::ink(),
                v(at[0], top - line_h * i as f64),
                false,
                true,
            )?;
            widest = widest.max(stage.get_width(m));
            quote.push(m);
        }
        let src_top = top - line_h * lines.len() as f64 - 0.05;
        let mut source_mobs = Vec::new();
        for (i, l) in src.iter().enumerate() {
            let m = kit.text(stage, l, 20.0, Palette::dim(), v(at[0], src_top - src_h * i as f64))?;
            widest = widest.max(stage.get_width(m));
            source_mobs.push(m);
        }
        let back = Rectangle::new()
            .width(widest + 0.8)
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
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    /// Storyboard §3: this video's four hues and the corpus's gold, red, ink,
    /// dim and faint are pairwise at least ΔE2000 20 apart.
    #[test]
    fn the_role_colours_are_told_apart() {
        let roles = [
            ("heads", heads()),
            ("tails", tails()),
            ("Alice", alice()),
            ("Bob", bob()),
            ("gold", gold()),
            ("red", warn()),
            ("ink", Palette::ink()),
            ("dim", Palette::dim()),
            ("faint", Palette::faint()),
        ];
        let own = 4;
        for (i, (na, a)) in roles.iter().enumerate() {
            for (nb, b) in &roles[i + 1..] {
                if i >= own {
                    continue; // the corpus's own roles against each other are not ours to hold
                }
                let d = de2000(lab(*a), lab(*b));
                assert!(d >= 20.0, "{na} and {nb}: ΔE2000 {d:.1}");
            }
        }
    }

    #[test]
    fn the_square_puts_alice_above_the_diagonal_and_gives_each_block_its_chance() {
        let pose = SquarePose {
            x: 0.8,
            y_h: 0.8,
            y_t: 0.8,
            left: 0.0,
            bottom: 0.0,
            side: 1.0,
        };
        for (b, want) in BLOCKS.into_iter().zip([0.64, 0.16, 0.16, 0.04]) {
            assert!((pose.share(b) - want).abs() < 1e-12, "{b:?}");
        }
        let ht = pose.center(Block::HT);
        let th = pose.center(Block::TH);
        assert!(ht[1] > ht[0], "heads then tails sits above the diagonal");
        assert!(th[1] < th[0], "tails then heads sits below it");
        // the mirror image of heads-then-tails is exactly tails-then-heads
        let (u0, w0, u1, w1) = pose.unit_rect(Block::HT);
        assert_eq!((w0, u0, w1, u1), pose.unit_rect(Block::TH));
        // a second coin breaks the mirror
        let two = SquarePose {
            y_h: 0.5,
            y_t: 0.5,
            ..pose
        };
        assert!((two.share(Block::HT) - 0.4).abs() < 1e-12 && (two.share(Block::TH) - 0.1).abs() < 1e-12);
        // a memory puts a step in the second cut
        let sticky = SquarePose {
            y_h: 0.8,
            y_t: 0.2,
            ..pose
        };
        assert!((sticky.share(Block::HT) - 0.16).abs() < 1e-12 && (sticky.share(Block::TH) - 0.04).abs() < 1e-12);
    }

    #[test]
    fn the_cube_keeps_the_square_as_its_front_and_equal_boxes_equal() {
        let flat = CubePose {
            left: 0.0,
            bottom: 0.0,
            side: 2.0,
            depth: 0.0,
        };
        // with no depth the third toss is invisible: the cube is the square
        assert_eq!(flat.project([0.8, 0.8, 0.3]), v(1.6, 1.6));
        let deep = CubePose { depth: 1.0, ..flat };
        // parallel: the same box anywhere projects to the same shape
        let a = [deep.project([0.0, 0.0, 0.0]), deep.project([0.2, 0.3, 0.4])];
        let b = [deep.project([0.5, 0.1, 0.3]), deep.project([0.7, 0.4, 0.7])];
        for i in 0..2 {
            assert!(((a[1][i] - a[0][i]) - (b[1][i] - b[0][i])).abs() < 1e-12);
        }
        // three tosses at four fifths: eight boxes in groups of 1, 3, 3, 1 by how many thin sides
        let cuts = [0.0, 0.8, 1.0];
        let boxes = cut_boxes([&cuts, &cuts, &cuts]);
        assert_eq!(boxes.len(), 8);
        let mut by_tails = [0; 4];
        for (_, place) in &boxes {
            by_tails[place.iter().sum::<usize>()] += 1;
        }
        assert_eq!(by_tails, [1, 3, 3, 1]);
        // a quarter turn takes one direction to another
        let half = slerp([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5);
        assert!((half[0] - half[2]).abs() < 1e-12 && half[1].abs() < 1e-12);
        assert_eq!(slerp([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], 0.7), [0.0, 1.0, 0.0]);
    }

    /// A turning box stays a box: its three sides stay at right angles and
    /// keep their lengths all the way, and it lands with each side along its
    /// new direction (either way along it), for every one of the six orders.
    #[test]
    fn a_turning_box_never_folds_flat() {
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let orders = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
        for turned in orders {
            let r = Rotation::of(turned);
            for step in 0..=10 {
                let t = f64::from(step) / 10.0;
                let sides = [0, 1, 2].map(|own| {
                    let mut d = [0.0; 3];
                    d[own] = 1.0;
                    r.apply(d, t)
                });
                for i in 0..3 {
                    assert!((dot(sides[i], sides[i]) - 1.0).abs() < 1e-9, "{turned:?} at {t}");
                    for j in i + 1..3 {
                        assert!(dot(sides[i], sides[j]).abs() < 1e-9, "{turned:?} folds at {t}");
                    }
                }
                if step == 10 {
                    for (dir, &own) in turned.iter().enumerate() {
                        assert!(
                            (sides[own][dir].abs() - 1.0).abs() < 1e-9,
                            "{turned:?}: side {own} not along {dir}"
                        );
                    }
                }
            }
        }
        // a swap of two sides is a quarter turn, not a half
        let w = Rotation::of([2, 1, 0]).q[0];
        assert!((w - (std::f64::consts::FRAC_PI_4).cos()).abs() < 1e-9, "{w}");
    }

    /// The row's labels print the model's chances to two decimals.
    #[test]
    fn the_row_prints_two_decimals_from_the_model() {
        let at = |c: f64| (0..5).map(|k| group_chance(c, k).percent(2)).collect::<Vec<_>>();
        assert_eq!(at(0.8), ["40.96", "40.96", "15.36", "2.56", "0.16"]);
        assert_eq!(at(0.5), ["6.25", "25.00", "37.50", "25.00", "6.25"]);
    }

    /// The heads-then-tails block of a square of side 3 (across 0 to 2.4, up
    /// 2.4 to 3) and the same four corners mirrored in the diagonal, in the
    /// same order.
    fn block_and_mirror() -> (Vec<Vec3>, Vec<Vec3>) {
        let from: Vec<Vec3> = [(0.0, 2.4), (2.4, 2.4), (2.4, 3.0), (0.0, 3.0)]
            .into_iter()
            .map(|(x, y)| v(x, y))
            .collect();
        let to = from.iter().map(|p| v(p[1], p[0])).collect();
        (from, to)
    }

    /// The shoelace area of a polygon's corners.
    fn area(corners: &[Vec3]) -> f64 {
        let n = corners.len();
        let twice: f64 = (0..n)
            .map(|i| corners[i][0] * corners[(i + 1) % n][1] - corners[(i + 1) % n][0] * corners[i][1])
            .sum();
        twice.abs() / 2.0
    }

    /// The turn at which a flip is exactly edge-on, and a flip drawn without
    /// the guard: every point straight to its mirror image (a linear
    /// `Transform` of the copy onto the mirror, which 05_2 and 08_4 used).
    fn edge_on_turn() -> f64 {
        LIFT_SHARE + (1.0 - LIFT_SHARE) * 0.5
    }

    #[test]
    fn a_flip_starts_on_its_block_and_lands_on_the_mirror_image() {
        let (from, to) = block_and_mirror();
        for (got, want) in flipped(&from, &to, 0.0, LIFT_BY).unwrap().iter().zip(&from) {
            assert_eq!(got, want);
        }
        for (got, want) in flipped(&from, &to, 1.0, LIFT_BY).unwrap().iter().zip(&to) {
            assert!(
                (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
                "{got:?} {want:?}"
            );
        }
    }

    /// A mirror flip scales the shape's area by `|1 - 2e|` (e: how far it has
    /// turned) however far it is lifted: it is nil exactly halfway over, and
    /// no path of straight lines to the mirror image can keep it away from
    /// nil. So the flip does not draw the frames that are edge-on, and every
    /// frame it does draw is a shape with room to fill.
    #[test]
    fn a_flip_draws_no_frame_that_is_flat() {
        let (from, to) = block_and_mirror();
        let whole = area(&from);
        let mut skipped = 0;
        for k in 0..=10_000 {
            let t = f64::from(k) / 10_000.0;
            let (_, e) = flip_phase(t);
            match flipped(&from, &to, t, LIFT_BY) {
                Some(now) => {
                    let a = area(&now);
                    assert!(a >= 0.0199 * whole, "t {t}: area {a} of {whole}");
                    assert!((a - (1.0 - 2.0 * e).abs() * whole).abs() < 1e-9, "t {t}");
                }
                None => {
                    skipped += 1;
                    assert!((e - 0.5).abs() < EDGE_ON, "t {t} skipped at e {e}");
                }
            }
        }
        assert!(skipped > 0, "the edge-on frames are reached");
        // the same on the hundred turns a 100-frame flip draws
        for k in 0..100 {
            if let Some(now) = flipped(&from, &to, f64::from(k) / 99.0, LIFT_BY) {
                assert!(area(&now) > 0.019 * whole, "frame {k}");
            }
        }
    }

    /// What the guard is for: a linear `Transform` of the copy onto its
    /// mirror image (every point to its own reflection) folds it flat on the
    /// diagonal halfway over, a polygon with no area to fill.
    #[test]
    fn without_the_guard_a_flip_folds_flat_halfway_over() {
        let (from, to) = block_and_mirror();
        let t = edge_on_turn();
        assert!(flipped(&from, &to, t, LIFT_BY).is_none());
        let unguarded: Vec<Vec3> = from
            .iter()
            .zip(&to)
            .map(|(&a, &b)| flipped_point(a, b, t, LIFT_BY))
            .collect();
        assert!(area(&unguarded) < 1e-9 * area(&from), "{}", area(&unguarded));
        let linear: Vec<Vec3> = from
            .iter()
            .zip(&to)
            .map(|(a, b)| v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0))
            .collect();
        assert!(area(&linear) < 1e-9, "{}", area(&linear));
    }

    /// The copy lifts up and to the left, away from the diagonal, by
    /// `LIFT_BY` before it turns, and has landed (no lift) at the end.
    #[test]
    fn the_copy_lifts_away_from_the_diagonal_and_lands_flat() {
        let (from, to) = block_and_mirror();
        let up = flipped(&from, &to, LIFT_SHARE, LIFT_BY).unwrap();
        let side = LIFT_BY / 2f64.sqrt();
        for (now, was) in up.iter().zip(&from) {
            assert!((now[0] - (was[0] - side)).abs() < 1e-12 && (now[1] - (was[1] + side)).abs() < 1e-12);
        }
        // a rider travels the same way, and a label on the diagonal stays on it
        let rider = flipped_point(v(1.0, 1.0), v(1.0, 1.0), 0.5, LIFT_BY);
        assert!(rider[1] > rider[0], "lifted off the diagonal mid-flip");
        assert_eq!(flipped_point(v(1.0, 1.0), v(1.0, 1.0), 1.0, LIFT_BY), v(1.0, 1.0));
    }

    /// A scene that is only what a function puts on the stage.
    struct Bare<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>>(F);

    impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Bare<F> {
        fn name(&self) -> &str {
            "bare"
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.0)(stage)?;
            pause(stage, 0.2)
        }
    }

    /// Run `build` on a stage of its own, 64 by 36 at 10 frames a second.
    fn bare(build: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) -> Result<(), String> {
        static RUNS: AtomicUsize = AtomicUsize::new(0);
        let n = RUNS.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("fair_coin_bare_{}_{n}", std::process::id()));
        let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).map_err(|e| e.to_string())?;
        opts.config.camera.resolution = (64, 36);
        opts.config.camera.fps = 10;
        let mut scene = Bare(build);
        fmn::render(&mut scene, opts).map_err(|e| format!("bare: {e}"))?;
        Ok(())
    }

    /// `Flip::between` on a stage: the copy of the block turns over frame by
    /// frame (20 s at 10 frames a second, so frames fall inside the edge-on
    /// window), every frame the shape has is one with room to fill, and it
    /// lands on the mirror image.
    #[test]
    fn between_turns_a_shape_over_on_stage_and_never_draws_it_flat() {
        let (from, to) = block_and_mirror();
        let whole = area(&from);
        let seen: Rc<RefCell<Vec<Vec<Vec3>>>> = Rc::default();
        let log = Rc::clone(&seen);
        bare(move |stage| {
            let style = Style::default().fill(alice(), 0.9).stroke(gold(), 2.5, 1.0);
            let copy = stage.add(Polygon::new(from.clone()).style(style).build())?;
            let mirror = stage.arena_mut().add(Polygon::new(to.clone()).style(style).build());
            let flip = Flip::between(stage, &[(copy, mirror)], LIFT_BY)?;
            let recorder = stage.add(VMobject::new())?;
            let frames = Rc::clone(&log);
            stage.add_updater(
                recorder,
                move |s, _| {
                    if let Some(p) = s.get_points(copy) {
                        frames.borrow_mut().push(p.to_vec());
                    }
                },
                true,
            )?;
            flip.play(stage, 20.0)?;
            let (landed, wanted) = (stage.get_points(copy), stage.get_points(mirror));
            match (landed, wanted) {
                (Some(l), Some(w)) if l.len() == w.len() => {
                    for (a, b) in l.iter().zip(w.iter()) {
                        if (a[0] - b[0]).abs() > 1e-9 || (a[1] - b[1]).abs() > 1e-9 {
                            return Err(fail(format!("the copy landed at {a:?}, not at {b:?}")));
                        }
                    }
                    Ok(())
                }
                _ => Err(fail("the copy or its mirror image lost its points".into())),
            }
        })
        .unwrap();
        let frames = seen.borrow();
        assert!(frames.len() >= 150, "{} frames recorded", frames.len());
        // a polygon's path is quadratic: its four corners are every second point
        let areas: Vec<f64> = frames
            .iter()
            .map(|p| area(&p.iter().step_by(2).take(4).copied().collect::<Vec<_>>()))
            .collect();
        for (i, a) in areas.iter().enumerate() {
            assert!(*a >= 0.0199 * whole, "frame {i}: area {a} of {whole}");
        }
        let least = areas.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(
            least < 0.2 * whole,
            "the shape turns through nearly edge-on (least {least})"
        );
    }

    /// The three flips of this video are one: chapters 05 and 08 turn their
    /// copies with `Flip::between`, and no chapter morphs a copy onto its
    /// mirror image with a linear `Transform` (which folds it flat).
    #[test]
    fn every_chapter_flip_goes_through_the_shared_flip() {
        let sources = [
            ("every_coin", include_str!("every_coin.rs")),
            ("where_it_breaks", include_str!("where_it_breaks.rs")),
            ("loaded_dice", include_str!("loaded_dice.rs")),
        ];
        for (name, source) in sources {
            for planted in ["Transform::new(ghost, mirror)", "Transform::new(copy, mirror)"] {
                assert!(!source.contains(planted), "{name} turns a copy over with {planted}");
            }
        }
        assert_eq!(
            include_str!("where_it_breaks.rs")
                .matches("Flip::between(stage,")
                .count(),
            1
        );
        assert_eq!(
            include_str!("loaded_dice.rs").matches("Flip::between(stage,").count(),
            2
        );
        assert!(include_str!("every_coin.rs").matches("flip_of_block(").count() >= 3);
    }
}
