//! Chapter 05, near a side: the second of the points no tile can dodge.
//! Past a corner block one point is not enough, so the strip between two
//! corner blocks takes two, a gap apart and one tile up; a tile centred
//! between them, within one tile of the floor, covers one (05_1). Could an
//! eighth-turned tile, its shadow wider, slip up between them? (05_2, a
//! four-second think.) No: the points are at most half a tile above its
//! centre and a tile reaches that far straight up (05_3), so it cuts their
//! line there, and dodging both means that cut fits between them; lower is
//! narrower, so the tile rests on the floor (05_4). Resting, it stands one
//! shadow s tall; its corner above the points' line splits at its top into
//! two right triangles of height d = s − y whose bases, d cot θ and
//! d tan θ, add to the cut, d / (sin θ cos θ): shorter than the gap only if
//! the points stand above h = s − gap (s² − 1)/2 (05_5). h bends down as the shadow
//! grows, so it is least at an end: exactly one, straight, where the points
//! stand, so nothing slips through (05_6). Drop the floor, raise the points
//! or widen the gap, each its own sentence and escape, and a tile slips
//! through (05_7). Turned to every side
//! of the box of 2.71, this settles the centres near a side; the middle is
//! left (05_8).
//!
//! Friedman's Lemma 2 (DS7), proved as in `model.rs` § the side strip, which
//! also records the error in his sketch (the least is not at 45°); the
//! screen says only which end is lower. Every frame is a tile from the model
//! (`strip_wander`, tiles resting on the floor, and the dodges of
//! `model::strip_dodges`); the rings' colours are the points' depths inside
//! that tile, and every readout reads the frame drawn ([`pose_of`]).

use std::cell::Cell;
use std::convert::identity;
use std::rc::Rc;

use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, common, corner, dim, dot, fact, fade_to, fail, faint, flat, ink, line_style, path, polygon,
    redot, reshape, retrace, ring, say, stand, swap, together, touch, turned, warn,
};
use super::the_corner_lemma::{
    CENTRE_R, EDGE_BAND, LAYER_MARKS, LAYER_RING, LAYER_UNDER, POINT_R, RING_R_POINT, SHADOW_Y,
};
use super::the_turned_tile::{BOARD, DEGREE, RING_AT, RING_R, THINK_S};
use crate::model::{self, Surd, Tile};

pub struct NearASide {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6", "05_7", "05_8"];

// the chapter's fixed words
pub const TURN: &str = "turn";
pub const SHADOW: &str = "shadow s";
pub const GAP: &str = "gap";
pub const CUT: &str = "cut";
pub const UP: &str = "the points: at most half a tile above the centre";
pub const REACH: &str = "a tile: half a tile straight up";
pub const D_IS: &str = "y: the points' height, d = s - y";
pub const D_TEX: &str = "d";
pub const THETA: &str = r"\theta";
pub const D_TAN: &str = r"d\tan\theta";
pub const D_COT: &str = r"d\cot\theta";
pub const CUT_IS: &str = r"\text{cut} = d\tan\theta + d\cot\theta = \frac{d}{\sin\theta\,\cos\theta}";
pub const BEATS: &str = r"\text{cut} < \text{gap}\ \text{only if}\ y > s - \text{gap}\,\sin\theta\cos\theta";
pub const SLIP: &str = r"y > s - \text{gap}\cdot\tfrac{s^2-1}{2} = h";
pub const IDENTITY: &str = r"s^2 = 1 + 2\sin\theta\cos\theta";
pub const STRAIGHT: &str = "straight";
pub const EIGHTH: &str = "an eighth";
pub const ONE: &str = "1";
pub const LOWEST: &str = "lowest at the straight end, not the eighth";
pub const EDGE: &str = "straight, on the floor: on the edge";
pub const DROPPED: &str = "an assumption dropped";
pub const NO_FLOOR: &str = "no floor";
pub const HIGH: &str = "points above one tile";
pub const WIDE: &str = "gap a whole tile";
pub const SETTLED: &str = "settled: centres near a corner or a side";
pub const NEXT: &str = "next: the middle";

// ---------------------------------------------------------------- layout

/// The side view: the floor of the box of 2.71 at 2.2 scene units a tile,
/// centred on the box's centre as [`Board`] does.
pub const SIDE: Board = Board {
    at: [-3.75, 0.03, 0.0],
    k: 2.2,
};
/// How high the side view's walls rise (tiles).
pub const WALL_TOP: f64 = 1.6;
/// The left edge of the panel's text beside the side view, and beside the
/// box view of 05_8; where the panel's formulas centre.
pub const TEXT_X: f64 = 0.2;
pub const BOX_TEXT_X: f64 = 1.7;
pub const FORMULA_X: f64 = 3.55;
/// The panel's rows: the four readouts, then the derivation and its note.
pub const ROWS: [f64; 4] = [3.0, 2.45, 1.9, 1.35];
pub const DERIVE_Y: (f64, f64) = (0.75, 0.2);
/// The plot of the slip height against the shadow: its corners on screen
/// (x0, y0, x1, y1) and the heights its bottom and top stand for.
pub const PLOT: [f64; 4] = [0.9, -2.85, 5.4, -0.6];
pub const PLOT_H: (f64, f64) = (0.98, 1.08);
/// The "lowest at the straight end" caption: above the plot, under the
/// derivation's note (below the plot it sat 33 px from the frame's foot).
pub const LOWEST_Y: f64 = -0.25;
/// The bar beside a tile resting on the floor that shows it one shadow
/// tall, left of its shadow (tiles).
pub const TALL_X: (f64, f64) = (-0.14, -0.07);
/// How high 05_3 lifts the eighth-turned tile's centre (tiles): inside the
/// strip, which reaches one tile up.
pub const LIFT: f64 = 0.95;
/// The turn 05_5 splits the corner at (degrees): not an eighth, so its two
/// right triangles differ (tan 30° ≠ cot 30°).
pub const SPLIT_DEG: f64 = 30.0;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    side: f64,
    gap: f64,
    eighth: String,
    dodges: Vec<model::Dodge>,
}

fn reads() -> fmn::Result<Reads> {
    for (key, want) in [
        ("strip.lemma", "yes"),
        ("strip.assumed", "yes"),
        ("strip.least", "straight"),
        ("strip.height.straight", "1"),
        ("strip.turned", "inside"),
        ("strip.one", "outside,edge"),
        ("strip.edges", "edge,outside|edge,edge|outside,edge"),
        ("strip.fail.floor", "outside,outside|through a wall|quadrant"),
        ("strip.fail.high", "outside,outside|inside the walls|0<y<=1"),
        ("strip.fail.wide", "outside,outside|inside the walls|x+2y<2sqrt2"),
        ("strip.box.left", "1/2"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    let gap = model::turned_gap();
    let eighth = fact("strip.height.eighth.2dp")?;
    if eighth != model::slip_height(Surd::sqrt2(), gap).decimals(2) {
        return Err(fail("strip.height.eighth.2dp is not the model's".into()));
    }
    Ok(Reads {
        side: model::turned_side().to_f64(),
        gap: gap.to_f64(),
        eighth,
        dodges: model::strip_dodges(),
    })
}

/// A point of the plot: shadow `s` across, slip height `h` up.
fn plot_p(s: f64, h: f64) -> Vec3 {
    let [x0, y0, x1, y1] = PLOT;
    v(
        x0 + (x1 - x0) * (s - 1.0) / (std::f64::consts::SQRT_2 - 1.0),
        y0 + (y1 - y0) * (h - PLOT_H.0) / (PLOT_H.1 - PLOT_H.0),
    )
}

/// What a ring says about its point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ring {
    /// Strictly inside the tile: yellow.
    Inside,
    /// On its edge: green.
    Edge,
    /// Missed while the other point is covered: faint.
    Missed,
    /// Missed with the other: both dodged, red.
    Dodged,
}

impl Ring {
    /// The rings for the points' depths inside the tile.
    fn of(depths: [f64; 2]) -> [Self; 2] {
        let both = depths.iter().all(|&d| d < -EDGE_BAND);
        depths.map(|d| {
            if d > EDGE_BAND {
                Ring::Inside
            } else if d >= -EDGE_BAND {
                Ring::Edge
            } else if both {
                Ring::Dodged
            } else {
                Ring::Missed
            }
        })
    }

    fn colour(self) -> Srgb {
        match self {
            Ring::Inside => Palette::yellow(),
            Ring::Edge => touch(),
            Ring::Missed => faint(),
            Ring::Dodged => warn(),
        }
    }
}

/// A frame: the tile and the strip's points, `gap` apart, `height` up.
#[derive(Clone, Copy, Debug)]
struct Frame {
    tile: Tile<f64>,
    gap: f64,
    height: f64,
}

/// What the readouts read off a frame: the centre, the turn in [0, 90°),
/// the cut's length along the points' line, the gap between the points and
/// the shadow.
fn pose_of(f: &Frame) -> [f64; 6] {
    let t = &f.tile;
    let theta = t.sin.atan2(t.cos).rem_euclid(std::f64::consts::FRAC_PI_2);
    [
        t.centre.0,
        t.centre.1,
        theta,
        model::cut_length(t, f.height),
        f.gap,
        model::shadow_width((t.cos, t.sin)),
    ]
}

/// The turn `theta` (radians, in [0, 90°)) as the readout shows it: whole
/// degrees, rounded to the nearest, with 90 shown as 0 (a quarter turn is the
/// same tile). The readout's formatter truncates toward zero when it shows no
/// decimals, so the drawn turn must reach it already whole: an atan2 a hair
/// under 30° would otherwise read 29.
fn shown_degrees(theta: f64) -> f64 {
    let d = theta.to_degrees().round();
    if d >= 90.0 { d - 90.0 } else { d }
}

/// A pose the motions run between.
#[derive(Clone, Copy, Debug)]
struct Key {
    across: f64,
    /// The centre's height; None: resting on the floor.
    up: Option<f64>,
    degrees: f64,
    gap: f64,
    height: f64,
}

/// A tile turned `degrees`, resting on the floor, `across` from the wall,
/// with the box's strip (the points `gap` apart, one tile up).
fn resting(across: f64, degrees: f64, gap: f64) -> Key {
    Key {
        across,
        up: None,
        degrees,
        gap,
        height: 1.0,
    }
}

/// The frame `u` of the way from `a` to `b`: every part straight-line,
/// except that a tile resting at both ends rests on the floor throughout.
fn blend(a: Key, b: Key, u: f64) -> Frame {
    let mix = |p: f64, q: f64| p + (q - p) * u;
    let turn_of = |deg: f64| {
        let r = deg.to_radians();
        (r.cos(), r.sin())
    };
    let turn = turn_of(mix(a.degrees, b.degrees));
    let low = |k: Key| k.up.unwrap_or(0.5 * model::shadow_width(turn_of(k.degrees)));
    let up = match (a.up, b.up) {
        (None, None) => 0.5 * model::shadow_width(turn),
        _ => mix(low(a), low(b)),
    };
    Frame {
        tile: Tile {
            centre: (mix(a.across, b.across), up),
            cos: turn.0,
            sin: turn.1,
        },
        gap: mix(a.gap, b.gap),
        height: mix(a.height, b.height),
    }
}

/// A dodge's tile and strip as a pose.
fn dodge_key(d: &model::Dodge) -> Key {
    let (c, s) = (d.tile.cos.to_f64(), d.tile.sin.to_f64());
    Key {
        across: d.tile.centre.0.to_f64(),
        up: Some(d.tile.centre.1.to_f64()),
        degrees: s.atan2(c).to_degrees(),
        gap: d.x.to_f64(),
        height: d.y.to_f64(),
    }
}

/// The side view's parts every frame redraws.
#[derive(Clone, Copy)]
struct Parts {
    side: f64,
    tile: Mob,
    centre: Mob,
    allowed: Mob,
    shadow: Mob,
    tall: Mob,
    /// The corner above the points' line, split at its top: left, right.
    cap: [Mob; 2],
    /// Its height d, the vertical from the line to the top corner.
    rise: Mob,
    line: Mob,
    points: [Mob; 2],
    rings: [Mob; 2],
    up: Mob,
    reach: Mob,
    cut: Mob,
    marker: Mob,
}

/// The corner of a tile above a line, split by the vertical through its top
/// corner ([`corner_split`]).
struct SplitCorner {
    /// The tile's top corner.
    top: (f64, f64),
    /// The part of the tile above the line left of that vertical, and right
    /// of it (outlines, counter-clockwise; empty when the line misses it).
    halves: [Vec<(f64, f64)>; 2],
}

/// Tile `t`'s top corner, and the part of the tile above the line `y` cut
/// by the vertical through that corner: left and right. While the line
/// passes over both side corners each is a right triangle of height d (the
/// corner's height over the line); the right one's angle at the top is the
/// turn θ, so its base is d tan θ, and the left one's is d cot θ.
fn corner_split(t: &Tile<f64>, y: f64, side: f64) -> SplitCorner {
    let corners = t.corners();
    let top = corners
        .iter()
        .copied()
        .fold(corners[0], |m, c| if c.1 > m.1 { c } else { m });
    let (lo, hi, sky) = (-1.0, side + 1.0, 9.0);
    let cap = common(&corners, &[(lo, y), (hi, y), (hi, sky), (lo, sky)]);
    let left = common(&cap, &[(lo, y), (top.0, y), (top.0, sky), (lo, sky)]);
    let right = common(&cap, &[(top.0, y), (hi, y), (hi, sky), (top.0, sky)]);
    SplitCorner {
        top,
        halves: [left, right],
    }
}

/// The labels of 05_5's split corner for frame `f`: d beside the vertical,
/// the turn θ at the top corner inside the right triangle, and the two
/// bases under the points' line. Detached.
fn split_labels(kit: &Kit, stage: &mut Stage<'_>, f: &Frame, side: f64) -> fmn::Result<[Mob; 4]> {
    let at = |q: (f64, f64)| SIDE.p(side, q);
    let y = f.height;
    let top = corner_split(&f.tile, y, side).top;
    let (l, r) = model::cut_at(&f.tile, y).unwrap_or((top.0, top.0));
    Ok([
        kit.tex(stage, D_TEX, 26.0, ink(), &[], at((top.0 - 0.07, 0.5 * (y + top.1))))?,
        kit.tex(stage, THETA, 24.0, ink(), &[], at((top.0 + 0.05, top.1 - 0.2)))?,
        kit.tex(stage, D_COT, 22.0, ink(), &[], at((0.5 * (l + top.0), y - 0.2)))?,
        kit.tex(stage, D_TAN, 22.0, ink(), &[], at((0.5 * (top.0 + r), y - 0.2)))?,
    ])
}

/// Redraw frame `f`, and record what the readouts read ([`pose_of`]).
fn draw(s: &mut fmn::mobject::Stage, parts: &Parts, pose: &Cell<[f64; 6]>, f: &Frame) {
    let at = |q: (f64, f64)| SIDE.p(parts.side, q);
    let t = &f.tile;
    let (cx, cy) = t.centre;
    reshape(s, parts.tile, SIDE.tile(parts.side, t));
    redot(s, parts.centre, at(t.centre), CENTRE_R);
    let w = model::shadow_width((t.cos, t.sin));
    let (a0, a1, low) = (1.0, 1.0 + f.gap, 0.5 * w);
    let rect = |x: (f64, f64), y: (f64, f64)| vec![at((x.0, y.0)), at((x.1, y.0)), at((x.1, y.1)), at((x.0, y.1))];
    reshape(s, parts.allowed, rect((a0, a1), (low, f.height)));
    reshape(s, parts.shadow, rect((cx - low, cx + low), SHADOW_Y));
    let x = cx - low;
    reshape(s, parts.tall, rect((x + TALL_X.0, x + TALL_X.1), (cy - low, cy + low)));
    // the part of the tile above the points' line, split at its top corner:
    // two right triangles whose bases make the cut
    let SplitCorner { top, halves } = corner_split(t, f.height, parts.side);
    for (m, half) in parts.cap.into_iter().zip(halves) {
        let pts = if half.len() < 3 {
            vec![at((cx, f.height)); 3]
        } else {
            half.iter().map(|&q| at(q)).collect()
        };
        reshape(s, m, pts);
    }
    retrace(
        s,
        parts.rise,
        vec![at((top.0, f.height)), at((top.0, top.1.max(f.height)))],
    );
    retrace(s, parts.line, vec![at((0.0, f.height)), at((parts.side, f.height))]);
    let pts = model::strip_points(f.gap, f.height);
    let says = Ring::of(pts.map(|p| model::depth_inside(t, p)));
    for (k, p) in pts.into_iter().enumerate() {
        redot(s, parts.points[k], at(p), POINT_R);
        redot(s, parts.rings[k], at(p), RING_R_POINT);
        s.set_stroke(parts.rings[k], Some(says[k].colour()), None, None, None, true);
    }
    retrace(s, parts.up, vec![at(t.centre), at((cx, f.height))]);
    retrace(s, parts.reach, vec![at(t.centre), at((cx, cy + 0.5))]);
    let (l, r) = model::cut_at(t, f.height).unwrap_or((cx, cx));
    retrace(s, parts.cut, vec![at((l, f.height)), at((r, f.height))]);
    redot(s, parts.marker, plot_p(w, model::slip_height(w, f.gap)), POINT_R);
    pose.set(pose_of(f));
}

/// One motion of the tile, drawn as it goes from `frame` (one updater on an
/// anchor in the scene).
struct Motion {
    tracker: Mob,
    anchor: Mob,
}

impl Motion {
    fn install(
        stage: &mut Stage<'_>,
        parts: Parts,
        pose: &Rc<Cell<[f64; 6]>>,
        start: f64,
        frame: Box<dyn Fn(f64) -> Frame>,
    ) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(start);
        let anchor = stage.arena_mut().add(VMobject::new());
        let pose = Rc::clone(pose);
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = s.tracker_value(tracker).unwrap_or(start);
                if now.to_bits() == last.to_bits() {
                    return;
                }
                last = now;
                draw(s, &parts, &pose, &frame(now));
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        Ok(Self { tracker, anchor })
    }

    fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

/// Tween `tracker` to `value` over `rt` seconds, played.
fn drive(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(tween(tracker, value, rt)?)?;
    Ok(())
}

/// Play the motion from pose `a` to pose `b` over `rt` seconds, then stop
/// it.
fn go(stage: &mut Stage<'_>, parts: Parts, pose: &Rc<Cell<[f64; 6]>>, a: Key, b: Key, rt: f64) -> fmn::Result<()> {
    let m = Motion::install(stage, parts, pose, 0.0, Box::new(move |u| blend(a, b, u)))?;
    drive(stage, m.tracker, 1.0, rt)?;
    m.stop(stage);
    Ok(())
}

/// Bring filled parts already in the scene to `opacity` together.
fn reveal(stage: &mut Stage<'_>, mobs: &[(Mob, f64)], rt: f64) -> fmn::Result<()> {
    let moves = mobs
        .iter()
        .map(|&(m, o)| fade_to(m, o, rt))
        .collect::<fmn::Result<Vec<_>>>()?;
    together(stage, moves)
}

/// A filled rectangle [x0, x1] × [y0, y1] of the box at chapter 01's scale.
/// Detached.
fn box_rect(stage: &mut Stage<'_>, side: f64, r: [f64; 4], c: Srgb, o: f64) -> Mob {
    let at = |q: (f64, f64)| BOARD.p(side, q);
    let [x0, y0, x1, y1] = r;
    polygon(
        stage,
        vec![at((x0, y0)), at((x1, y0)), at((x1, y1)), at((x0, y1))],
        flat(c, o),
    )
}

/// The four side strips of centres in the box `side` wide, each between two
/// corner blocks: bottom, top, left, right, as [x0, y0, x1, y1].
fn strips(side: f64) -> [[f64; 4]; 4] {
    let (lo, hi, near, far) = (0.5, side - 0.5, 1.0, side - 1.0);
    [
        [near, lo, far, near],
        [near, far, far, hi],
        [lo, near, near, far],
        [far, near, hi, far],
    ]
}

/// The box of 2.71 as 04 left it: the middle square and the strips grey,
/// the four corner blocks green, the outline, the centre region and
/// Friedman's four points. Detached, in drawing order.
fn box_view(stage: &mut Stage<'_>, side: f64) -> Vec<Mob> {
    let at = |q: (f64, f64)| BOARD.p(side, q);
    let (lo, hi, near, far) = (0.5, side - 0.5, 1.0, side - 1.0);
    let mut out = vec![box_rect(stage, side, [near, near, far, far], faint(), 1.0)];
    for r in strips(side) {
        out.push(box_rect(stage, side, r, faint(), 1.0));
    }
    for (x0, x1) in [(lo, near), (far, hi)] {
        for (y0, y1) in [(lo, near), (far, hi)] {
            out.push(box_rect(stage, side, [x0, y0, x1, y1], touch(), 0.6));
        }
    }
    out.push(path(stage, BOARD.outline(side), line_style(ink(), 4.0)));
    out.push(path(
        stage,
        vec![at((lo, lo)), at((hi, lo)), at((hi, hi)), at((lo, hi)), at((lo, lo))],
        line_style(dim(), 2.0),
    ));
    for p in model::corner_points(model::turned_side()) {
        out.push(dot(stage, at((p.0.to_f64(), p.1.to_f64())), POINT_R, Palette::purple()));
    }
    out
}

impl SceneConstruct for NearASide {
    fn name(&self) -> &str {
        "05_near_a_side"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let (side, gap) = (r.side, r.gap);
        let at = |q: (f64, f64)| SIDE.p(side, q);
        let mid = 1.0 + 0.5 * gap;
        let home = model::strip_points(gap, 1.0)[0];

        // filled parts enter the scene now, unseen, layered; strokes stay
        // detached and fade in when their beat comes
        let hidden = || vec![at(home); 3];
        let allowed = polygon(stage, hidden(), flat(Palette::purple(), 0.3));
        let block = |stage: &mut Stage<'_>, x0: f64, x1: f64| {
            polygon(
                stage,
                vec![at((x0, 0.5)), at((x1, 0.5)), at((x1, 1.0)), at((x0, 1.0))],
                flat(touch(), 0.35),
            )
        };
        let blocks = [block(stage, 0.5, 1.0), block(stage, side - 1.0, side - 0.5)];
        let tile = polygon(stage, hidden(), flat(corner(), 1.0));
        let caps = [
            polygon(stage, hidden(), flat(turned(), 0.65)),
            polygon(stage, hidden(), flat(turned(), 0.35)),
        ];
        let shadow = polygon(stage, hidden(), flat(Palette::purple(), 0.85));
        let tall = polygon(stage, hidden(), flat(Palette::purple(), 0.85));
        let centre = dot(stage, at(home), CENTRE_R, ink());
        let points = [
            dot(stage, at(home), POINT_R, Palette::purple()),
            dot(stage, at(home), POINT_R, Palette::purple()),
        ];
        let marker = dot(stage, plot_p(1.0, 1.0), POINT_R, turned());
        for &m in &[allowed, blocks[0], blocks[1]] {
            stage.set_z_index(m, LAYER_UNDER, true);
        }
        for &m in &[centre, points[0], points[1], marker] {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        let filled = [
            allowed, blocks[0], blocks[1], tile, caps[0], caps[1], shadow, tall, centre, points[0], points[1], marker,
        ];
        for &m in &filled {
            stage.add_to_scene(m)?;
            stage.set_opacity(m, 0.0, true);
        }
        let stroke = |stage: &mut Stage<'_>, c: Srgb, w: f64| path(stage, vec![at(home), at(home)], line_style(c, w));
        let line = stroke(stage, dim(), 2.0);
        let up = stroke(stage, dim(), 7.0);
        let reach = stroke(stage, ink(), 2.5);
        let cut = stroke(stage, ink(), 6.0);
        let rise = stroke(stage, ink(), 3.0);
        let rings = [
            ring(stage, at(home), RING_R_POINT, faint(), 5.0),
            ring(stage, at(home), RING_R_POINT, faint(), 5.0),
        ];
        for &m in &[line, up, reach, cut, rise] {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        for &m in &rings {
            stage.set_z_index(m, LAYER_RING, true);
        }
        let parts = Parts {
            side,
            tile,
            centre,
            allowed,
            shadow,
            tall,
            cap: caps,
            rise,
            line,
            points,
            rings,
            up,
            reach,
            cut,
            marker,
        };
        let pose = Rc::new(Cell::new([1.6, 0.5, 0.0, 0.0, gap, 1.0]));
        let walls = [
            path(stage, vec![at((0.0, 0.0)), at((side, 0.0))], line_style(ink(), 6.0)),
            path(stage, vec![at((0.0, 0.0)), at((0.0, WALL_TOP))], line_style(ink(), 6.0)),
            path(
                stage,
                vec![at((side, 0.0)), at((side, WALL_TOP))],
                line_style(ink(), 6.0),
            ),
        ];
        for &m in &walls {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        // a readout of the frame drawn: `k` of [`pose_of`], as `show` gives it
        let readout = |stage: &mut Stage<'_>, k: usize, show: fn(f64) -> f64, places: usize, c: Srgb, y: f64| {
            let p = Rc::clone(&pose);
            kit.live_value(
                stage,
                move |_: &fmn::mobject::Stage| show(p.get()[k]),
                places,
                false,
                34.0,
                c,
                v(TEXT_X + 2.4, y),
                RIGHT,
            )
        };

        // ---------------------------------------------------------- 05_1
        // the box as 04 left it, its bottom strip; the floor close up: 04's
        // stray tile misses the corner's point, a second point catches it;
        // the tile wanders through the strip
        let wander = Motion::install(
            stage,
            parts,
            &pose,
            0.0,
            Box::new(move |t| Frame {
                tile: model::strip_wander(t),
                gap,
                height: 1.0,
            }),
        )?;
        let line_1 = say(&kit, stage, LINES[0])?;
        let boxed = box_view(stage, side);
        swap(stage, &[], &boxed, 0.6)?;
        let bottom = box_rect(stage, side, strips(side)[0], Palette::purple(), 0.7);
        swap(stage, &[], &[bottom], 0.4)?;
        line_1.wait_for(stage, "one point is not enough")?;
        let mut leaving = boxed;
        leaving.push(bottom);
        let mut arriving = walls.to_vec();
        arriving.push(line);
        swap(stage, &leaving, &arriving, 0.6)?;
        reveal(
            stage,
            &[
                (blocks[0], 0.35),
                (blocks[1], 0.35),
                (tile, 1.0),
                (centre, 1.0),
                (points[0], 1.0),
            ],
            0.5,
        )?;
        swap(stage, &[], &[rings[0]], 0.3)?;
        line_1.wait_for(stage, "so take two")?;
        reveal(stage, &[(points[1], 1.0)], 0.3)?;
        swap(stage, &[], &[rings[1]], 0.3)?;
        line_1.wait_for(stage, "centred between them")?;
        reveal(stage, &[(allowed, 0.3)], 0.4)?;
        let rt = line_1.left(stage, 2.5) - 0.2;
        drive(stage, wander.tracker, 0.5, rt)?;
        wander.stop(stage);

        // ---------------------------------------------------------- 05_2
        // its shadow s on the floor; rolled on the floor to an eighth
        // between the points; the question; four seconds to think
        let rest = model::strip_wander(0.5);
        let from = resting(rest.centre.0, rest.sin.atan2(rest.cos).to_degrees(), gap);
        let eighth = resting(mid, 45.0, gap);
        let line_2 = say(&kit, stage, LINES[1])?;
        let turn_label = kit.text_left(stage, TURN, 30.0, dim(), v(TEXT_X, ROWS[0]))?;
        let degree = kit.text_left(stage, DEGREE, 34.0, turned(), v(TEXT_X + 2.46, ROWS[0] + 0.1))?;
        let turn_value = readout(stage, 2, shown_degrees, 0, turned(), ROWS[0])?;
        let shadow_label = kit.text_left(stage, SHADOW, 30.0, dim(), v(TEXT_X, ROWS[1]))?;
        let shadow_value = readout(stage, 5, identity, 2, Palette::purple(), ROWS[1])?;
        reveal(stage, &[(shadow, 0.85)], 0.4)?;
        swap(
            stage,
            &[],
            &[turn_label, degree, turn_value, shadow_label, shadow_value],
            0.4,
        )?;
        let ask = line_2.at("Could it slip")?;
        let rt = (ask - stage.scene().time().to_f64() - 0.1).max(1.5);
        go(stage, parts, &pose, from, eighth, rt)?;
        line_2.wait_for(stage, "Could it slip")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        kit.hold(stage)?;
        let mut elapsed = 0.0;
        stage.add_dt_updater(
            arc,
            move |s: &mut fmn::mobject::Stage, m, dt| {
                elapsed += dt;
                retrace(s, m, arc_points(RING_AT, RING_R, 1.0 - elapsed / THINK_S));
            },
            false,
        )?;
        kit.listen(stage, THINK_S, LINES[1])?;

        // ---------------------------------------------------------- 05_3
        // straight up: the points at most half a tile above the centre, a
        // tile's reach half a tile; lifted, the tile keeps both
        let lifted = Key {
            up: Some(LIFT),
            ..eighth
        };
        let line_3 = say(&kit, stage, LINES[2])?;
        let up_caption = kit.text_left(stage, UP, 24.0, dim(), v(TEXT_X, DERIVE_Y.0))?;
        swap(stage, &[track, arc], &[up, up_caption], 0.5)?;
        line_3.wait_for(stage, "a tile reaches")?;
        let reach_caption = kit.text_left(stage, REACH, 24.0, ink(), v(TEXT_X, DERIVE_Y.1))?;
        swap(stage, &[], &[reach, reach_caption], 0.4)?;
        let rt = line_3.left(stage, 1.2) - 0.1;
        go(stage, parts, &pose, eighth, lifted, rt)?;

        // ---------------------------------------------------------- 05_4
        // the cut along the points' line, against the gap (both read off
        // the frame); lower is narrower, so the tile comes back to the floor
        let line_4 = say(&kit, stage, LINES[3])?;
        let gap_label = kit.text_left(stage, GAP, 30.0, dim(), v(TEXT_X, ROWS[2]))?;
        let gap_value = readout(stage, 4, identity, 2, dim(), ROWS[2])?;
        let cut_label = kit.text_left(stage, CUT, 30.0, ink(), v(TEXT_X, ROWS[3]))?;
        let cut_value = readout(stage, 3, identity, 2, ink(), ROWS[3])?;
        swap(stage, &[], &[cut, gap_label, gap_value, cut_label, cut_value], 0.5)?;
        line_4.wait_for(stage, "Lower is narrower")?;
        let rt = line_4.left(stage, 1.2) - 0.1;
        go(stage, parts, &pose, lifted, eighth, rt)?;

        // ---------------------------------------------------------- 05_5
        // resting, one shadow tall; rolled to 30° so the two right triangles
        // differ; the corner above the points' line split at its top: its
        // height d = s − y, the turn θ at the top, bases d cot θ and d tan θ
        // adding to the cut, d / (sin θ cos θ); the cut against the gap gives
        // the slip height. Each equation replaces the one before
        let split = resting(mid, SPLIT_DEG, gap);
        let line_5 = say(&kit, stage, LINES[4])?;
        swap(stage, &[up_caption, reach_caption, up, reach], &[], 0.4)?;
        reveal(stage, &[(tall, 0.85)], 0.4)?;
        let ask = line_5.at("Split its corner")?;
        let rt = (ask - stage.scene().time().to_f64() - 0.1).max(0.8);
        go(stage, parts, &pose, eighth, split, rt)?;
        line_5.wait_for(stage, "two right triangles")?;
        let labels = split_labels(&kit, stage, &blend(split, split, 0.0), side)?;
        let d_is = kit.text_left(stage, D_IS, 24.0, dim(), v(TEXT_X, DERIVE_Y.1))?;
        reveal(stage, &[(caps[0], 0.65), (caps[1], 0.35)], 0.4)?;
        let mut shown = labels.to_vec();
        shown.extend([rise, d_is]);
        swap(stage, &[], &shown, 0.4)?;
        line_5.wait_for(stage, "add to the cut")?;
        let cut_is = kit.tex(stage, CUT_IS, 28.0, ink(), &[], v(FORMULA_X, DERIVE_Y.0))?;
        swap(stage, &[], &[cut_is], 0.4)?;
        line_5.wait_for(stage, "shorter than the gap")?;
        let beats = kit.tex(stage, BEATS, 28.0, ink(), &[], v(FORMULA_X, DERIVE_Y.0))?;
        swap(stage, &[cut_is], &[beats], 0.5)?;
        line_5.wait_for(stage, "higher than this")?;
        let slip = kit.tex(stage, SLIP, 32.0, ink(), &[], v(FORMULA_X, DERIVE_Y.0))?;
        let identity = kit.tex(stage, IDENTITY, 26.0, dim(), &[], v(FORMULA_X, DERIVE_Y.1))?;
        swap(stage, &[beats, d_is], &[slip, identity], 0.5)?;

        // ---------------------------------------------------------- 05_6
        // h against the shadow: bends down, above its chord, least at an end;
        // the tile rolls to straight on the floor, the marker runs along the
        // curve; one straight, the points' height; straight on the floor, on
        // the edge
        let line_6 = say(&kit, stage, LINES[5])?;
        let [x0, y0, x1, y1] = PLOT;
        let axes = path(stage, vec![v(x0, y1), v(x0, y0), v(x1, y0)], line_style(dim(), 2.0));
        let root = std::f64::consts::SQRT_2;
        let curve = path(
            stage,
            (0..=40)
                .map(|i| {
                    let s = 1.0 + (root - 1.0) * f64::from(i) / 40.0;
                    plot_p(s, model::slip_height(s, gap))
                })
                .collect(),
            line_style(ink(), 4.0),
        );
        let level = path(
            stage,
            vec![plot_p(1.0, 1.0), plot_p(root, 1.0)],
            line_style(Palette::purple(), 3.0),
        );
        let chord = path(
            stage,
            vec![
                plot_p(1.0, model::slip_height(1.0, gap)),
                plot_p(root, model::slip_height(root, gap)),
            ],
            line_style(dim(), 2.0),
        );
        let straight_tick = kit.text(stage, STRAIGHT, 24.0, dim(), v(x0, y0 - 0.35))?;
        let eighth_tick = kit.text(stage, EIGHTH, 24.0, dim(), v(x1, y0 - 0.35))?;
        let mut done = labels.to_vec();
        done.push(rise);
        swap(
            stage,
            &done,
            &[axes, curve, level, chord, straight_tick, eighth_tick],
            0.5,
        )?;
        reveal(stage, &[(marker, 1.0)], 0.3)?;
        let high_end = plot_p(root, model::slip_height(root, gap));
        let eighth_end = kit.text(stage, &r.eighth, 30.0, ink(), v(high_end[0] - 0.1, high_end[1] + 0.35))?;
        swap(stage, &[], &[eighth_end], 0.3)?;
        let straight = resting(mid, 0.0, gap);
        let ends = line_6.at("exactly one")?;
        let rt = (ends - stage.scene().time().to_f64() - 0.1).max(1.2);
        go(stage, parts, &pose, split, straight, rt)?;
        let one_end = plot_p(1.0, 1.0);
        let one = kit.text(stage, ONE, 30.0, ink(), v(one_end[0] - 0.3, one_end[1] + 0.3))?;
        let lowest = kit.text_left(stage, LOWEST, 24.0, touch(), v(TEXT_X, LOWEST_Y))?;
        swap(stage, &[], &[one, lowest], 0.4)?;
        line_6.wait_for(stage, "nothing slips through")?;
        let edge = kit.text_left(stage, EDGE, 24.0, touch(), v(TEXT_X, DERIVE_Y.1))?;
        swap(stage, &[identity], &[edge], 0.4)?;

        // ---------------------------------------------------------- 05_7
        // drop an assumption and a tile slips through: no floor, the points
        // raised past one, the gap a whole tile. The bound, its plot and the
        // edge status hold only under the assumptions, so they leave first,
        // and only then does the header arrive in the bound's row; each move
        // starts from the straight tile on the floor, waits for the sentence
        // that names it and ends on one of the model's dodges; the gap and
        // cut readouts follow
        let line_7 = say(&kit, stage, LINES[6])?;
        swap(
            stage,
            &[
                slip,
                axes,
                curve,
                level,
                chord,
                straight_tick,
                eighth_tick,
                one,
                eighth_end,
                lowest,
                edge,
            ],
            &[],
            0.4,
        )?;
        let dropped = kit.text_left(stage, DROPPED, 26.0, warn(), v(TEXT_X, DERIVE_Y.0))?;
        swap(stage, &[], &[dropped], 0.3)?;
        reveal(stage, &[(marker, 0.0), (tall, 0.0)], 0.2)?;
        let [floor_dodge, high_dodge, wide_dodge] = [0, 1, 2].map(|k| dodge_key(&r.dodges[k]));
        let caption = |stage: &mut Stage<'_>, s: &str| kit.text_left(stage, s, 24.0, warn(), v(TEXT_X, DERIVE_Y.1));
        line_7.wait_for(stage, "Drop the floor")?;
        let no_floor = caption(stage, NO_FLOOR)?;
        swap(stage, &[], &[no_floor], 0.3)?;
        go(stage, parts, &pose, straight, floor_dodge, 1.4)?;
        go(stage, parts, &pose, floor_dodge, straight, 0.4)?;
        line_7.wait_for(stage, "Raise the points")?;
        let high = caption(stage, HIGH)?;
        swap(stage, &[no_floor], &[high], 0.3)?;
        go(stage, parts, &pose, straight, high_dodge, 1.1)?;
        go(stage, parts, &pose, high_dodge, straight, 0.4)?;
        line_7.wait_for(stage, "Widen the gap")?;
        let wide = caption(stage, WIDE)?;
        swap(stage, &[high], &[wide], 0.3)?;
        let rt = (line_7.left(stage, 1.2) - 0.1).max(1.2);
        go(stage, parts, &pose, straight, wide_dodge, rt)?;

        // ---------------------------------------------------------- 05_8
        // the box of 2.71: every side strip settled, the middle next
        let line_8 = say(&kit, stage, LINES[7])?;
        let mut outs: Vec<Mob> = filled.to_vec();
        outs.extend([line, cut, rings[0], rings[1]]);
        outs.extend(walls);
        outs.extend([
            dropped,
            wide,
            turn_label,
            degree,
            turn_value,
            shadow_label,
            shadow_value,
            gap_label,
            gap_value,
            cut_label,
            cut_value,
        ]);
        swap(stage, &outs, &[], 0.5)?;
        let boxed = box_view(stage, side);
        swap(stage, &[], &boxed, 0.6)?;
        line_8.wait_for(stage, "this settles")?;
        let mut ins: Vec<Mob> = strips(side)
            .into_iter()
            .map(|r| box_rect(stage, side, r, touch(), 0.6))
            .collect();
        ins.push(kit.text_left(stage, SETTLED, 24.0, touch(), v(BOX_TEXT_X, 1.0))?);
        swap(stage, &[], &ins, 0.5)?;
        line_8.wait_for(stage, "Only the middle")?;
        let next = kit.text_left(stage, NEXT, 24.0, dim(), v(BOX_TEXT_X, 0.5))?;
        swap(stage, &[], &[next], 0.4)?;

        // the line and its breath, and the closing reveal stands before the
        // wipe (it comes on the line's last words)
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("05_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
        // every phrase a beat waits for or times a move by is in the line
        // that beat speaks, exactly (case included), in the order the beat
        // uses them: the scene's cues read from its own source
        let source = include_str!("near_a_side.rs");
        let mut from = [0usize; LINES.len()];
        let mut cues = 0;
        for (k, _) in source.match_indices("line_") {
            let rest = &source[k + "line_".len()..];
            let Some(n) = rest.chars().next().and_then(|c| c.to_digit(10)) else {
                continue;
            };
            let call = &rest[1..];
            let Some(phrase) = [".wait_for(stage, \"", ".at(\""]
                .iter()
                .find_map(|p| call.strip_prefix(p))
                .and_then(|s| s.split('"').next())
            else {
                continue;
            };
            let i = n as usize - 1;
            let text = script.text(LINES[i]).unwrap();
            let at = text[from[i]..]
                .find(phrase)
                .unwrap_or_else(|| panic!("{} has no {phrase:?} at or after its earlier cues", LINES[i]));
            from[i] += at;
            cues += 1;
        }
        assert!(cues >= 16, "only {cues} cues found");
    }

    /// The side view stays in the frame and left of the panel, with room
    /// under the floor for a tile through it and over the raised points; the
    /// plot and its labels sit in the panel; the box view clears its
    /// captions.
    #[test]
    fn the_layout_fits_the_frame() {
        let side = model::turned_side().to_f64();
        let (lo, hi) = (SIDE.p(side, (0.0, -0.3)), SIDE.p(side, (side, 1.75)));
        assert!(lo[0] > -FRAME_X_RADIUS + 0.3 && lo[1] > -FRAME_Y_RADIUS + 0.3, "{lo:?}");
        assert!(hi[1] < FRAME_Y_RADIUS - 0.3, "{hi:?}");
        assert!(hi[0] < TEXT_X - 0.6, "the side view clears the panel");
        let [x0, y0, x1, y1] = PLOT;
        assert!(x0 > TEXT_X && x1 < FRAME_X_RADIUS - 0.5, "{PLOT:?}");
        assert!(y0 - 0.35 - 0.3 > -FRAME_Y_RADIUS && y1 < DERIVE_Y.1 - 0.4, "{PLOT:?}");
        let root = std::f64::consts::SQRT_2;
        let gap = model::turned_gap().to_f64();
        for i in 0..=40 {
            let s = 1.0 + (root - 1.0) * f64::from(i) / 40.0;
            let p = plot_p(s, model::slip_height(s, gap));
            assert!(p[1] > y0 && p[1] < y1, "the curve stays in the plot at {s}");
        }
        assert!(plot_p(root, model::slip_height(root, gap))[1] + 0.35 < DERIVE_Y.1 - 0.2);
        assert!(
            LOWEST_Y > y1 + 0.3 && LOWEST_Y < DERIVE_Y.1 - 0.4,
            "the 'lowest' caption sits between the plot and the derivation's note"
        );
        assert!(ROWS.windows(2).all(|r| r[0] - r[1] >= 0.5) && ROWS[3] - DERIVE_Y.0 >= 0.5);
        assert!(
            BOARD.p(side, (side, side))[0] < BOX_TEXT_X - 0.8,
            "the box clears the panel"
        );
    }

    /// Every motion between poses keeps a resting tile on the floor and a
    /// point covered, and the lifted tile's cut is longer than its floor
    /// cut.
    #[test]
    fn the_rolls_rest_on_the_floor_and_cover_a_point() {
        let gap = model::turned_gap().to_f64();
        let rest = model::strip_wander(0.5);
        let from = resting(rest.centre.0, rest.sin.atan2(rest.cos).to_degrees(), gap);
        let mid = 1.0 + 0.5 * gap;
        for (a, b) in [
            (from, resting(mid, 45.0, gap)),
            (resting(mid, 45.0, gap), resting(mid, SPLIT_DEG, gap)),
            (resting(mid, SPLIT_DEG, gap), resting(mid, 0.0, gap)),
        ] {
            for k in 0..=200 {
                let f = blend(a, b, f64::from(k) / 200.0);
                let low = f.tile.corners().iter().fold(f64::INFINITY, |m, c| m.min(c.1));
                assert!(low.abs() < 1e-9, "{k}: rests on the floor");
                let best = model::strip_points(f.gap, f.height)
                    .iter()
                    .map(|&p| model::depth_inside(&f.tile, p))
                    .fold(f64::NEG_INFINITY, f64::max);
                assert!(best >= -EDGE_BAND, "{k}: a point covered");
            }
        }
        let eighth = resting(mid, 45.0, gap);
        let lifted = blend(
            eighth,
            Key {
                up: Some(LIFT),
                ..eighth
            },
            1.0,
        );
        let turn = (lifted.tile.cos, lifted.tile.sin);
        assert!(lifted.tile.centre.1 <= 1.0);
        assert!(model::cut_length(&lifted.tile, 1.0) > model::floor_cut(turn, 1.0));
    }

    /// 05_5's picture: for a turned tile resting on the floor, the corner
    /// above the points' line splits at its top into a left part from the
    /// cut's left end and a right part to its right end, each as tall as the
    /// corner's height d over the line; the bases are d cot θ and d tan θ and
    /// add to the cut, d / (sin θ cos θ).
    #[test]
    fn the_corner_splits_into_two_right_triangles() {
        let gap = model::turned_gap().to_f64();
        let side = model::turned_side().to_f64();
        for deg in [10.0, SPLIT_DEG, 45.0, 60.0, 80.0] {
            let f = blend(resting(1.3, deg, gap), resting(1.3, deg, gap), 0.0);
            let SplitCorner {
                top,
                halves: [left, right],
            } = corner_split(&f.tile, f.height, side);
            let (l, r) = model::cut_at(&f.tile, f.height).unwrap();
            let d = top.1 - f.height;
            let th = f64::to_radians(deg);
            let span = |p: &[(f64, f64)]| {
                let xs = p.iter().map(|q| q.0);
                let ys = p.iter().map(|q| q.1);
                (
                    xs.clone().fold(f64::INFINITY, f64::min),
                    xs.fold(f64::NEG_INFINITY, f64::max),
                    ys.fold(f64::NEG_INFINITY, f64::max),
                )
            };
            let (l0, l1, lt) = span(&left);
            let (r0, r1, rt) = span(&right);
            assert!((l0 - l).abs() < 1e-9 && (l1 - top.0).abs() < 1e-9, "{deg}");
            assert!((r0 - top.0).abs() < 1e-9 && (r1 - r).abs() < 1e-9, "{deg}");
            assert!((lt - top.1).abs() < 1e-9 && (rt - top.1).abs() < 1e-9, "{deg}");
            assert!((top.0 - l - d / th.tan()).abs() < 1e-9, "{deg}: d cot θ");
            assert!((r - top.0 - d * th.tan()).abs() < 1e-9, "{deg}: d tan θ");
            assert!((r - l - d / (th.sin() * th.cos())).abs() < 1e-9, "{deg}");
        }
    }

    /// The moves of 05_7 start from the straight tile on the floor, which
    /// touches both points, and end on the model's dodges, each breaking
    /// exactly its assumption with both points missed; the readouts read the
    /// frame, so at the whole-tile gap they say 1 and a cut shorter than it.
    #[test]
    fn the_dodges_end_on_the_models() {
        let gap = model::turned_gap().to_f64();
        let mid = 1.0 + 0.5 * gap;
        let start = blend(resting(mid, 0.0, gap), resting(mid, 0.0, gap), 0.0);
        let held = model::strip_points(gap, 1.0).map(|p| model::depth_inside(&start.tile, p));
        assert!(held.iter().all(|d| d.abs() < 1e-9), "{held:?}");
        assert_eq!(pose_of(&start)[4], gap);
        let dodges = model::strip_dodges();
        assert_eq!(
            dodges.iter().take(3).map(|d| d.breaks).collect::<Vec<_>>(),
            ["quadrant", "0<y<=1", "x+2y<2sqrt2"]
        );
        for d in dodges.iter().take(3) {
            let end = blend(resting(mid, 0.0, gap), dodge_key(d), 1.0);
            let (cx, cy) = end.tile.centre;
            assert!((cx - d.tile.centre.0.to_f64()).abs() < 1e-9, "{}", d.key);
            assert!((cy - d.tile.centre.1.to_f64()).abs() < 1e-9, "{}", d.key);
            assert!((end.tile.cos - d.tile.cos.to_f64()).abs() < 1e-9, "{}", d.key);
            assert!((end.tile.sin - d.tile.sin.to_f64()).abs() < 1e-9, "{}", d.key);
            assert!((end.gap - d.x.to_f64()).abs() < 1e-12 && (end.height - d.y.to_f64()).abs() < 1e-12);
            let depths = model::strip_points(end.gap, end.height).map(|p| model::depth_inside(&end.tile, p));
            assert!(depths.iter().all(|&dp| dp < -EDGE_BAND), "{}: {depths:?}", d.key);
            assert_eq!(Ring::of(depths), [Ring::Dodged; 2], "{}", d.key);
            let pose = pose_of(&end);
            assert!((pose[4] - d.x.to_f64()).abs() < 1e-12, "{}: the gap readout", d.key);
            assert!(pose[3] < pose[4], "{}: the cut readout under the gap", d.key);
        }
        let wide = pose_of(&blend(resting(mid, 0.0, gap), dodge_key(&dodges[2]), 1.0));
        assert!((wide[4] - 1.0).abs() < 1e-12, "{wide:?}");
        assert!(
            (wide[3] - (2.0 * std::f64::consts::SQRT_2 - 2.0)).abs() < 1e-9,
            "{wide:?}"
        );
    }

    /// The readouts' shadow is the tile's height resting on the floor, and
    /// the cut over the top corner is (s − y)/(sin θ cos θ), as 05_5 writes.
    #[test]
    fn the_cut_is_the_base_of_the_corner() {
        let gap = model::turned_gap().to_f64();
        for deg in [5.0, 20.0, 45.0, 70.0, 85.0] {
            let f = blend(resting(1.3, deg, gap), resting(1.3, deg, gap), 0.0);
            let [_, cy, theta, cut, _, s] = pose_of(&f);
            let top = f.tile.corners().iter().fold(f64::NEG_INFINITY, |m, c| m.max(c.1));
            assert!((top - s).abs() < 1e-12 && (cy - 0.5 * s).abs() < 1e-12, "{deg}");
            let (sn, cs) = theta.sin_cos();
            assert!((cut - (s - f.height) / (sn * cs)).abs() < 1e-9, "{deg}");
        }
    }

    /// The turn readout shows the drawn turn in whole degrees: the settled
    /// split of 05_5 reads SPLIT_DEG and the eighth-turned dodges of 05_7
    /// read 45, from the frames the scene draws, whatever hair atan2 lands
    /// under; either side of a rounding boundary goes to the nearer degree; a
    /// quarter turn reads 0, the same tile; and every value is whole, so the
    /// formatter's truncation shows it as it is.
    #[test]
    fn the_turn_readout_shows_whole_degrees() {
        let gap = model::turned_gap().to_f64();
        let mid = 1.0 + 0.5 * gap;
        let split = resting(mid, SPLIT_DEG, gap);
        assert_eq!(shown_degrees(pose_of(&blend(split, split, 1.0))[2]), SPLIT_DEG);
        let eighth = resting(mid, 45.0, gap);
        let from = blend(eighth, resting(mid, SPLIT_DEG, gap), 0.0);
        assert_eq!(shown_degrees(pose_of(&from)[2]), 45.0);
        let dodges = model::strip_dodges();
        for d in [&dodges[0], &dodges[2]] {
            let end = blend(resting(mid, 0.0, gap), dodge_key(d), 1.0);
            assert_eq!(shown_degrees(pose_of(&end)[2]), 45.0, "{}", d.key);
        }
        let deg = |x: f64| shown_degrees(x.to_radians());
        assert_eq!(deg(29.999_999_999), 30.0);
        assert_eq!(deg(30.000_000_001), 30.0);
        assert_eq!(deg(29.49), 29.0);
        assert_eq!(deg(29.51), 30.0);
        assert_eq!(deg(44.6), 45.0);
        assert_eq!(deg(0.0), 0.0);
        assert_eq!(deg(0.4), 0.0);
        assert_eq!(deg(89.4), 89.0);
        assert_eq!(deg(89.6), 0.0);
        let quarter = std::f64::consts::FRAC_PI_2;
        assert_eq!(shown_degrees((-1e-15_f64).rem_euclid(quarter)), 0.0);
        for k in 0..=900 {
            let shown = deg(f64::from(k) / 10.0);
            assert_eq!(shown, shown.trunc(), "{k}");
            assert!((0.0..90.0).contains(&shown), "{k}");
        }
    }

    /// The rings: inside, on the edge, one miss beside a covered point, and
    /// both dodged.
    #[test]
    fn the_rings_say_which_point_is_covered() {
        assert_eq!(Ring::of([0.1, -0.1]), [Ring::Inside, Ring::Missed]);
        assert_eq!(Ring::of([0.0, -0.1]), [Ring::Edge, Ring::Missed]);
        assert_eq!(Ring::of([-0.1, -0.2]), [Ring::Dodged; 2]);
    }
}
