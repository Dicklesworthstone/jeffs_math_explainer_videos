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
    /// upper-right. `Transform::new(copy, mirror)` is the flip.
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
/// coordinates, for sliding it out), one turns it about its own centre, and
/// its six faces are placed by the cube's updater.
#[derive(Clone, Copy, Debug)]
pub struct CubeBox {
    pub spec: BoxSpec,
    pub shift: [Mob; 3],
    /// 0 to 1: how far the box has turned towards `turned` (its sides
    /// permuted), about its own centre.
    pub turn: Mob,
    /// Which of the box's own sides lies along the first, second and third
    /// direction once it has turned (identity: no turn).
    pub turned: [usize; 3],
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

/// Boxes drawn in a parallel projection from trackers: the square with a
/// third side.
pub struct Cube {
    pub view: CubeView,
    pub boxes: Vec<CubeBox>,
    pub anchor: Mob,
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
        let mut colors = Vec::with_capacity(specs.len());
        for &i in &order {
            let (spec, color) = specs[i];
            let faces: [Mob; 6] = std::array::from_fn(|_| {
                stage.arena_mut().add(
                    Polygon::new(unit_quad())
                        .style(Style::default().fill(color, 1.0).stroke(hex("#0E1116"), 1.6, 0.9))
                        .build(),
                )
            });
            boxes.push(CubeBox {
                spec,
                shift: [0, 1, 2].map(|_| stage.add_value_tracker(0.0)),
                turn: stage.add_value_tracker(0.0),
                turned: [0, 1, 2],
                faces,
            });
            colors.push(color);
        }
        let anchor = stage.arena_mut().add(VMobject::new());
        Ok(Self { view, boxes, anchor })
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
        let mut last: Option<Vec<f64>> = None;
        stage.add_updater(
            self.anchor,
            move |s, _| {
                let val = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let mut now = vec![val(view.left), val(view.bottom), val(view.side), val(view.depth)];
                for b in &boxes {
                    now.extend(b.shift.map(val));
                    now.push(val(b.turn));
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
                for (k, b) in boxes.iter().enumerate() {
                    let at = 4 + 4 * k;
                    let (shift, turn) = ([now[at], now[at + 1], now[at + 2]], now[at + 3]);
                    place_box(s, &pose, b, shift, turn);
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

/// Place one box's faces: its corners, turned `turn` of the way towards its
/// final orientation about its own centre, shifted, projected; faces that
/// point away from the viewer are hidden.
fn place_box(s: &mut fmn::mobject::Stage, pose: &CubePose, b: &CubeBox, shift: [f64; 3], turn: f64) {
    let c = b.spec.center();
    let half = b.spec.size.map(|x| x / 2.0);
    // the box's own three half-sides, as vectors: unturned they lie along the
    // directions; fully turned, side `turned[i]` lies along direction i
    let mut sides = [[0.0; 3]; 3];
    for (own, side) in sides.iter_mut().enumerate() {
        let to = b.turned.iter().position(|&x| x == own).unwrap_or(own);
        let mut from_dir = [0.0; 3];
        from_dir[own] = 1.0;
        let mut to_dir = [0.0; 3];
        to_dir[to] = 1.0;
        *side = slerp(from_dir, to_dir, turn).map(|x| x * half[own]);
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
        let shade = face_shade(n);
        s.set_fill(m, None, Some(if seen { shade } else { 0.0 }), None, false);
        s.set_stroke(m, None, None, Some(if seen { 0.9 } else { 0.0 }), None, false);
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
}
