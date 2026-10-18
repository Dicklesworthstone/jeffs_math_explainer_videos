//! Chapter 06, a triangle of points: the last centres, the middle square
//! the four points bound (06_1), cut along its diagonal, one tile long,
//! into two triangles with sides at most one. Must a tile centred in one
//! cover a corner of it? (06_2, a four-second think.) Suppose a tile missed
//! all three: each corner would lie past one of its four sides (06_3). Two
//! past opposite sides are over a tile apart, so the triangle has a side
//! longer than one (06_4, the middle square grown to 3/4); otherwise all
//! lie past two neighbouring sides, beyond one diagonal of the tile, and the
//! triangle misses the centre (06_5, a centre outside the triangle). With
//! short sides and the centre in the triangle neither can happen, so the
//! tile covers a corner, perhaps only on its edge (06_6). Every tile in the
//! box covers a point, but where five fit each point sits on the edges of
//! two tiles: counting needs one more step (06_7).
//!
//! Friedman's Lemma 3 (DS7), proved by the separating line of `model.rs`
//! § the middle. Every frame is a tile from the model (`middle_wander`, the
//! two dodges `middle_too_large` and `centre_outside`, and the turned tile
//! where five fit); the rings' colours are the points' depths in that tile.

use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, corner, dim, dot, fact, fade_to, fail, faint, flat, ink, line_style, path, polygon, redot,
    reshape, retrace, ring, say, stand, swap, together, touch, turned, warn,
};
use super::the_corner_lemma::{CENTRE_R, EDGE_BAND, LAYER_MARKS, LAYER_RING, LAYER_UNDER, POINT_R, RING_R_POINT};
use super::the_turned_tile::{BOARD, RING_AT, RING_R, THINK_S};
use crate::model::{self, MIDDLE, MIDDLE_TRIANGLES, Point, Surd, Tile};

pub struct ATriangleOfPoints {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["06_1", "06_2", "06_3", "06_4", "06_5", "06_6", "06_7"];

// the chapter's fixed words
pub const ONE: &str = "1";
pub const MISSED: &str = "if a tile missed all three corners,";
pub const PAST: &str = "each would lie past one of its sides";
pub const LONG: &str = "a side longer than one";
pub const APART: &str = "past opposite sides: over a tile apart";
pub const OUT: &str = "the centre outside the triangle";
pub const ONLY: &str = "past two neighbouring sides only";
pub const CROSSING: &str = "crossing the diagonal: past one of the other two";
pub const MISSES: &str = "all beyond it: the triangle misses the centre";
pub const KEPT: &str = "sides at most one, the centre in the triangle";
pub const COVERS: &str = "covers: inside or on the edge";
pub const SETTLED: &str = "every tile covers one of the four points";
pub const SHARED: &str = "each point on two tiles' edges";
pub const NEXT: &str = "next: from covering to counting";

// ---------------------------------------------------------------- layout

/// The middle view: the box of 2.71 at 2.3 scene units a tile, centred on
/// the box's centre, which is the middle square's.
pub const MID: Board = Board {
    at: [-3.3, -0.15, 0.0],
    k: 2.3,
};
/// The left edge of the panel's text beside the middle view, and beside the
/// box view of 06_1 and 06_7.
pub const TEXT_X: f64 = 0.6;
pub const BOX_TEXT_X: f64 = 1.7;
/// The panel's rows of captions.
pub const ROWS: [f64; 6] = [2.6, 2.1, 1.2, 0.7, 0.2, -0.3];
/// How far the tile's side lines run either way from a side's middle, how
/// far out from the centre 06_5's region of corners reaches along the two
/// sides' normals, and half the length of the diagonal it draws (tiles).
pub const SIDE_LINE: f64 = 1.0;
pub const L_REACH: f64 = 1.1;
pub const DIAGONAL_HALF: f64 = 0.85;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    side: f64,
    gap: f64,
    gap_2dp: String,
}

fn reads() -> fmn::Result<Reads> {
    for (key, want) in [
        ("middle.lemma", "yes"),
        ("middle.diagonal2", "1"),
        ("middle.sides2", "1/2,1/2,1|1,1/2,1/2"),
        ("middle.edge.holds", "edge,edge,edge,edge"),
        ("middle.packing.covers", "0,4|2,4|1,4|3,4"),
        ("middle.packing.inside", "none"),
        ("middle.box.left", "0"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    if !fact("middle.fail.large")?.starts_with("apart:") {
        return Err(fail(
            "middle.fail.large: the long side does not part two corners".into(),
        ));
    }
    let out = fact("middle.fail.out")?;
    if !out.starts_with("beside:") || !out.contains("|outside|outside,outside,outside|") {
        return Err(fail(format!("middle.fail.out is {out:?}")));
    }
    Ok(Reads {
        side: model::turned_side().to_f64(),
        gap: model::turned_gap().to_f64(),
        gap_2dp: fact("turned.gap.2dp")?,
    })
}

/// The four points of a middle square `size` wide from (1, 1), in
/// [`model::corner_points`]' order.
fn square_points(size: f64) -> [Point<f64>; 4] {
    let (one, far) = (1.0, 1.0 + size);
    [(one, one), (one, far), (far, one), (far, far)]
}

/// What a ring says about its point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ring {
    /// Strictly inside the tile: yellow.
    Inside,
    /// On its edge: green.
    Edge,
    /// Missed, while a corner of the triangle in view is covered, or not a
    /// corner of it: faint.
    Missed,
    /// A corner of the triangle in view, missed with all its others: red.
    Dodged,
}

impl Ring {
    /// The rings for the four points' depths inside the tile, `focus` the
    /// points that are corners of the triangle in view.
    fn of(depths: [f64; 4], focus: [bool; 4]) -> [Self; 4] {
        let dodged = (0..4).filter(|&k| focus[k]).all(|k| depths[k] < -EDGE_BAND);
        std::array::from_fn(|k| {
            if depths[k] > EDGE_BAND {
                Ring::Inside
            } else if depths[k] >= -EDGE_BAND {
                Ring::Edge
            } else if dodged && focus[k] {
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

/// A frame: the tile, the middle square's size, and the triangle in view
/// (0 the lower right, 1 the upper left).
#[derive(Clone, Copy, Debug)]
struct Frame {
    tile: Tile<f64>,
    size: f64,
    focus: usize,
}

/// A pose the motions run between.
#[derive(Clone, Copy, Debug)]
struct Key {
    centre: Point<f64>,
    degrees: f64,
    size: f64,
}

impl Key {
    /// An exact tile of the model as a pose, the middle square `size` wide.
    fn of(t: &Tile<Surd>, size: f64) -> Self {
        let (c, s) = (t.cos.to_f64(), t.sin.to_f64());
        Self {
            centre: (t.centre.0.to_f64(), t.centre.1.to_f64()),
            degrees: s.atan2(c).to_degrees(),
            size,
        }
    }
}

/// The frame `u` of the way from `a` to `b`, every part straight-line, the
/// lower right triangle in view.
fn blend(a: Key, b: Key, u: f64) -> Frame {
    let mix = |p: f64, q: f64| p + (q - p) * u;
    let th = mix(a.degrees, b.degrees).to_radians();
    Frame {
        tile: Tile {
            centre: (mix(a.centre.0, b.centre.0), mix(a.centre.1, b.centre.1)),
            cos: th.cos(),
            sin: th.sin(),
        },
        size: mix(a.size, b.size),
        focus: 0,
    }
}

/// The ends of tile `t`'s side line `k` (in [`model::SIDES`]' order): the
/// line through the middle of that side, along it, [`SIDE_LINE`] either
/// way.
fn side_line(t: &Tile<f64>, k: usize) -> [Point<f64>; 2] {
    let [e1, e2] = t.axes();
    let normals = [e1, e2, (-e1.0, -e1.1), (-e2.0, -e2.1)];
    let (n, along) = (normals[k], normals[(k + 1) % 4]);
    let mid = (t.centre.0 + 0.5 * n.0, t.centre.1 + 0.5 * n.1);
    [
        (mid.0 - SIDE_LINE * along.0, mid.1 - SIDE_LINE * along.1),
        (mid.0 + SIDE_LINE * along.0, mid.1 + SIDE_LINE * along.1),
    ]
}

/// What 06_5 draws for corners past tile `t`'s sides `k` and k + 1 only:
/// the diagonal between those sides (the line through the centre and the
/// two tile corners where each meets one of the other two sides), and the
/// L-shaped region such corners are confined to, outside the tile but past
/// neither of the other two sides (their lines are the fences), cut to
/// [`L_REACH`]. The region touches the diagonal only at those two tile
/// corners, so a corner in it is strictly beyond the diagonal.
fn beside_region(t: &Tile<f64>, k: usize) -> ([Point<f64>; 2], [Point<f64>; 6]) {
    let [e1, e2] = t.axes();
    let normals = [e1, e2, (-e1.0, -e1.1), (-e2.0, -e2.1)];
    let (na, nb) = (normals[k], normals[(k + 1) % 4]);
    let c = t.centre;
    // a point a out along side k's normal and b along side k + 1's
    let p = |a: f64, b: f64| (c.0 + a * na.0 + b * nb.0, c.1 + a * na.1 + b * nb.1);
    let (h, r, s) = (0.5, L_REACH, DIAGONAL_HALF);
    (
        [p(s, -s), p(-s, s)],
        [p(h, -h), p(r, -h), p(r, r), p(-h, r), p(-h, h), p(h, h)],
    )
}

/// The middle view's parts every frame redraws.
#[derive(Clone, Copy)]
struct Parts {
    side: f64,
    tile: Mob,
    centre: Mob,
    focus: Mob,
    square: Mob,
    diagonal: Mob,
    sides: [Mob; 4],
    points: [Mob; 4],
    rings: [Mob; 4],
}

/// Redraw frame `f`.
fn draw(s: &mut fmn::mobject::Stage, parts: &Parts, f: &Frame) {
    let at = |q: Point<f64>| MID.p(parts.side, q);
    let t = &f.tile;
    reshape(s, parts.tile, MID.tile(parts.side, t));
    redot(s, parts.centre, at(t.centre), CENTRE_R);
    let pts = square_points(f.size);
    let tri = MIDDLE_TRIANGLES[f.focus];
    reshape(s, parts.focus, tri.iter().map(|&i| at(pts[i])).collect());
    retrace(s, parts.square, [0, 2, 3, 1, 0].iter().map(|&i| at(pts[i])).collect());
    retrace(s, parts.diagonal, vec![at(pts[0]), at(pts[3])]);
    for (k, &m) in parts.sides.iter().enumerate() {
        retrace(s, m, side_line(t, k).iter().map(|&q| at(q)).collect());
    }
    let focus: [bool; 4] = std::array::from_fn(|k| tri.contains(&k));
    let says = Ring::of(pts.map(|p| model::depth_inside(t, p)), focus);
    for (k, p) in pts.into_iter().enumerate() {
        redot(s, parts.points[k], at(p), POINT_R);
        redot(s, parts.rings[k], at(p), RING_R_POINT);
        s.set_stroke(parts.rings[k], Some(says[k].colour()), None, None, None, true);
    }
}

/// One motion of the tile, drawn as it goes from `frame` (one updater on an
/// anchor in the scene).
struct Motion {
    tracker: Mob,
    anchor: Mob,
}

impl Motion {
    fn install(stage: &mut Stage<'_>, parts: Parts, frame: Box<dyn Fn(f64) -> Frame>) -> fmn::Result<Self> {
        let tracker = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let now = s.tracker_value(tracker).unwrap_or(0.0);
                if now.to_bits() == last.to_bits() {
                    return;
                }
                last = now;
                draw(s, &parts, &frame(now));
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

/// Play `frame` from 0 to 1 over `rt` seconds, then stop it.
fn run(stage: &mut Stage<'_>, parts: Parts, frame: Box<dyn Fn(f64) -> Frame>, rt: f64) -> fmn::Result<()> {
    let m = Motion::install(stage, parts, frame)?;
    stage.play(tween(m.tracker, 1.0, rt)?)?;
    m.stop(stage);
    Ok(())
}

/// Play the motion from pose `a` to pose `b` over `rt` seconds.
fn go(stage: &mut Stage<'_>, parts: Parts, a: Key, b: Key, rt: f64) -> fmn::Result<()> {
    run(stage, parts, Box::new(move |u| blend(a, b, u)), rt)
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
    let at = |q: Point<f64>| BOARD.p(side, q);
    let [x0, y0, x1, y1] = r;
    polygon(
        stage,
        vec![at((x0, y0)), at((x1, y0)), at((x1, y1)), at((x0, y1))],
        flat(c, o),
    )
}

/// The box of 2.71 as 05 left it, but for its middle square: the corner
/// blocks and the side strips green, the outline, and Friedman's four
/// points. Detached, in drawing order (the middle square, drawn by the
/// caller, goes under them).
fn box_view(stage: &mut Stage<'_>, side: f64) -> Vec<Mob> {
    let at = |q: Point<f64>| BOARD.p(side, q);
    let (lo, hi, near, far) = (0.5, side - 0.5, 1.0, side - 1.0);
    let mut out = Vec::new();
    for r in [
        [lo, lo, near, near],
        [far, lo, hi, near],
        [lo, far, near, hi],
        [far, far, hi, hi],
        [near, lo, far, near],
        [near, far, far, hi],
        [lo, near, near, far],
        [far, near, hi, far],
    ] {
        out.push(box_rect(stage, side, r, touch(), 0.6));
    }
    out.push(path(stage, BOARD.outline(side), line_style(ink(), 4.0)));
    for p in model::corner_points(model::turned_side()) {
        out.push(dot(stage, at((p.0.to_f64(), p.1.to_f64())), POINT_R, Palette::purple()));
    }
    out
}

/// The box's middle square of centres, in colour `c`. Detached.
fn middle_square(stage: &mut Stage<'_>, side: f64, c: Srgb, o: f64) -> Mob {
    box_rect(stage, side, [1.0, 1.0, side - 1.0, side - 1.0], c, o)
}

impl SceneConstruct for ATriangleOfPoints {
    fn name(&self) -> &str {
        "06_a_triangle_of_points"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let (side, gap) = (r.side, r.gap);
        let at = |q: Point<f64>| MID.p(side, q);
        let home = square_points(gap)[0];

        // filled parts enter the scene now, unseen, layered; strokes stay
        // detached and fade in when their beat comes
        let hidden = || vec![at(home); 3];
        let focus = polygon(stage, hidden(), flat(Palette::purple(), 0.18));
        let tile = polygon(stage, hidden(), flat(corner(), 0.85));
        let centre = dot(stage, at(home), CENTRE_R, ink());
        let points: [Mob; 4] = std::array::from_fn(|_| dot(stage, at(home), POINT_R, Palette::purple()));
        stage.set_z_index(focus, LAYER_UNDER, true);
        stage.set_z_index(centre, LAYER_MARKS, true);
        for &m in &points {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        let mut filled = vec![focus, tile, centre];
        filled.extend(points);
        for &m in &filled {
            stage.add_to_scene(m)?;
            stage.set_opacity(m, 0.0, true);
        }
        let stroke = |stage: &mut Stage<'_>, c: Srgb, w: f64| path(stage, vec![at(home), at(home)], line_style(c, w));
        let square = stroke(stage, dim(), 3.0);
        let diagonal = stroke(stage, ink(), 4.0);
        let sides: [Mob; 4] = std::array::from_fn(|_| stroke(stage, faint(), 2.0));
        let rings: [Mob; 4] = std::array::from_fn(|_| ring(stage, at(home), RING_R_POINT, faint(), 5.0));
        let mut marks = vec![square, diagonal];
        marks.extend(sides);
        for &m in &marks {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        for &m in &rings {
            stage.set_z_index(m, LAYER_RING, true);
        }
        let parts = Parts {
            side,
            tile,
            centre,
            focus,
            square,
            diagonal,
            sides,
            points,
            rings,
        };
        let caption = |stage: &mut Stage<'_>, s: &str, c: Srgb, row: usize| {
            kit.text_left(stage, s, 24.0, c, v(TEXT_X, ROWS[row]))
        };

        // the poses: the wander's rest (the lower right triangle's centroid,
        // straight), the two dodges and the tile where five fit
        let rest = model::middle_wander(1.0);
        let centroid = Key {
            centre: rest.centre,
            degrees: rest.sin.atan2(rest.cos).to_degrees(),
            size: gap,
        };
        let (big, large_tile) = model::middle_too_large();
        let large = Key::of(&large_tile, (big[3].0 - big[0].0).to_f64());
        let outside_tile = model::centre_outside();
        let outside = Key::of(&outside_tile, gap);
        let five = model::five_turned();
        let fitted = Key::of(&five.tiles[MIDDLE], gap);

        // ---------------------------------------------------------- 06_1
        // the box as 05 left it, its middle grey; the middle close up, cut
        // along its diagonal, one tile long, into two triangles
        let line_1 = say(&kit, stage, LINES[0])?;
        let grey = middle_square(stage, side, faint(), 1.0);
        let boxed = box_view(stage, side);
        let mut all = vec![grey];
        all.extend(boxed.iter().copied());
        swap(stage, &[], &all, 0.6)?;
        let lit = middle_square(stage, side, Palette::purple(), 0.7);
        swap(stage, &[grey], &[lit], 0.4)?;
        let mut leaving = boxed;
        leaving.push(lit);
        let show = Motion::install(stage, parts, Box::new(move |_| blend(centroid, centroid, 0.0)))?;
        swap(stage, &leaving, &[square], 0.6)?;
        reveal(stage, &points.map(|m| (m, 1.0)), 0.4)?;
        swap(stage, &[], &rings, 0.3)?;
        line_1.wait_for(stage, "Cut it along")?;
        let label = |stage: &mut Stage<'_>, s: &str, q: Point<f64>| kit.text(stage, s, 28.0, dim(), at(q));
        let (lo, hi) = (1.0, 1.0 + gap);
        let labels = [
            label(stage, ONE, (0.5 * (lo + hi) - 0.12, 0.5 * (lo + hi) + 0.12))?,
            label(stage, &r.gap_2dp, (0.5 * (lo + hi), lo - 0.14))?,
            label(stage, &r.gap_2dp, (hi + 0.2, 0.5 * (lo + hi)))?,
        ];
        let mut cut_in = vec![diagonal];
        cut_in.extend(labels);
        swap(stage, &[], &cut_in, 0.5)?;
        show.stop(stage);

        // ---------------------------------------------------------- 06_2
        // a tile centred in the lower right triangle wanders (examples); the
        // question; four seconds to think
        let line_2 = say(&kit, stage, LINES[1])?;
        reveal(stage, &[(focus, 0.18), (tile, 0.85), (centre, 1.0)], 0.4)?;
        let ask = line_2.at("Must a tile")?;
        let rt = (ask - stage.scene().time().to_f64() - 0.1).max(2.0);
        run(
            stage,
            parts,
            Box::new(move |t| Frame {
                tile: model::middle_wander(t),
                size: gap,
                focus: 0,
            }),
            rt,
        )?;
        line_2.wait_for(stage, "Must a tile")?;
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

        // ---------------------------------------------------------- 06_3
        // suppose every corner were missed: each would lie past a side line
        let line_3 = say(&kit, stage, LINES[2])?;
        let mut first = vec![track, arc];
        first.extend(labels);
        let missed = caption(stage, MISSED, ink(), 0)?;
        swap(stage, &first, &[missed], 0.4)?;
        line_3.wait_for(stage, "past one of")?;
        let past = caption(stage, PAST, dim(), 1)?;
        let mut lines_in = sides.to_vec();
        lines_in.push(past);
        swap(stage, &[], &lines_in, 0.5)?;

        // ---------------------------------------------------------- 06_4
        // a side longer than one: the square grown to 3/4, the eighth-turned
        // tile at its centre misses all four; two corners past opposite
        // sides, more than a tile apart
        let line_4 = say(&kit, stage, LINES[3])?;
        let long = caption(stage, LONG, warn(), 2)?;
        swap(stage, &[], &[long], 0.3)?;
        let rt = (line_4.at("over a tile apart")? - stage.scene().time().to_f64() - 0.1).max(1.4);
        go(stage, parts, centroid, large, rt)?;
        let end = blend(large, large, 0.0);
        let big_tri = MIDDLE_TRIANGLES[0].map(|i| big[i]);
        let model::Meets::Apart(i, j, k) = model::meets(&large_tile, &big_tri) else {
            return Err(fail("the long side does not part two corners".into()));
        };
        let opposite = [k, (k + 2) % 4].map(|s| {
            path(
                stage,
                side_line(&end.tile, s).iter().map(|&q| at(q)).collect(),
                line_style(ink(), 4.0),
            )
        });
        let pair = [big_tri[i], big_tri[j]].map(|p| (p.0.to_f64(), p.1.to_f64()));
        let long_side = path(stage, pair.iter().map(|&q| at(q)).collect(), line_style(warn(), 4.0));
        let (dx, dy) = (big_tri[i].0 - big_tri[j].0, big_tri[i].1 - big_tri[j].1);
        let apart_by = format!("these two: {:.2} apart", (dx * dx + dy * dy).to_f64().sqrt());
        let apart = caption(stage, APART, ink(), 3)?;
        let apart_by = caption(stage, &apart_by, warn(), 4)?;
        let mut shown = opposite.to_vec();
        shown.extend([long_side, apart, apart_by]);
        swap(stage, &[], &shown, 0.5)?;

        // ---------------------------------------------------------- 06_5
        // the centre outside the triangle: the corners past two neighbouring
        // sides only, the other two sides' lines fencing them in; crossing
        // the diagonal between the two would take a corner past a fence, so
        // all lie in the L beyond it, and the triangle misses the centre
        let line_5 = say(&kit, stage, LINES[4])?;
        let out_caption = caption(stage, OUT, warn(), 2)?;
        swap(stage, &shown, &[], 0.3)?;
        swap(stage, &[long], &[out_caption], 0.3)?;
        let rt = (line_5.at("Crossing the diagonal")? - stage.scene().time().to_f64() - 0.6).max(1.4);
        go(stage, parts, large, outside, rt)?;
        let end = blend(outside, outside, 0.0);
        let model::Meets::Beside(k) = model::meets(&outside_tile, &model::middle_triangles(model::turned_side())[0])
        else {
            return Err(fail("the centre outside does not lie beside its triangle".into()));
        };
        let fences = [(k + 2) % 4, (k + 3) % 4].map(|s| {
            path(
                stage,
                side_line(&end.tile, s).iter().map(|&q| at(q)).collect(),
                line_style(ink(), 4.0),
            )
        });
        let only = caption(stage, ONLY, ink(), 3)?;
        swap(stage, &[], &[fences[0], fences[1], only], 0.4)?;
        line_5.wait_for(stage, "Crossing the diagonal")?;
        let (cross_line, region) = beside_region(&end.tile, k);
        let shade = polygon(stage, region.iter().map(|&q| at(q)).collect(), flat(warn(), 0.15));
        stage.set_z_index(shade, LAYER_UNDER, true);
        let diagonal_line = path(
            stage,
            cross_line.iter().map(|&q| at(q)).collect(),
            line_style(warn(), 4.0),
        );
        let crossing = caption(stage, CROSSING, ink(), 4)?;
        swap(stage, &[], &[shade, diagonal_line, crossing], 0.5)?;
        line_5.wait_for(stage, "misses the centre")?;
        let misses = caption(stage, MISSES, warn(), 5)?;
        swap(stage, &[], &[misses], 0.4)?;

        // ---------------------------------------------------------- 06_6
        // with short sides and the centre in the triangle, neither: the tile
        // where five fit covers every corner, on its edges
        let line_6 = say(&kit, stage, LINES[5])?;
        let kept = caption(stage, KEPT, touch(), 2)?;
        let mut done = vec![shade, diagonal_line, only, crossing, misses, out_caption, missed, past];
        done.extend(fences);
        done.extend(sides);
        swap(stage, &done, &[kept], 0.5)?;
        let rt = (line_6.at("perhaps only on its edge")? - stage.scene().time().to_f64() - 0.1).max(1.4);
        go(stage, parts, outside, fitted, rt)?;
        let covers = caption(stage, COVERS, touch(), 3)?;
        swap(stage, &[], &[covers], 0.4)?;

        // ---------------------------------------------------------- 06_7
        // every tile in the box covers a point; where five fit, each point is
        // on two tiles' edges: counting needs one more step
        let line_7 = say(&kit, stage, LINES[6])?;
        let mut outs: Vec<Mob> = filled.clone();
        outs.extend([square, diagonal, kept, covers]);
        outs.extend(rings);
        swap(stage, &outs, &[], 0.5)?;
        let mut settled_in = vec![middle_square(stage, side, touch(), 0.6)];
        settled_in.extend(box_view(stage, side));
        swap(stage, &[], &settled_in, 0.6)?;
        line_7.wait_for(stage, "covers a point")?;
        let settled = kit.text_left(stage, SETTLED, 24.0, touch(), v(BOX_TEXT_X, 1.0))?;
        swap(stage, &[], &[settled], 0.4)?;
        line_7.wait_for(stage, "But where five")?;
        let five_tiles: Vec<Mob> = five
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let c = if i == MIDDLE { turned() } else { corner() };
                let pose = Tile {
                    centre: (t.centre.0.to_f64(), t.centre.1.to_f64()),
                    cos: t.cos.to_f64(),
                    sin: t.sin.to_f64(),
                };
                polygon(stage, BOARD.tile(side, &pose), flat(c, 0.75))
            })
            .collect();
        let shared_rings: Vec<Mob> = model::corner_points(model::turned_side())
            .iter()
            .map(|p| {
                let q = (p.0.to_f64(), p.1.to_f64());
                let on = five
                    .tiles
                    .iter()
                    .filter(|t| model::depth_inside(t, *p) == Surd::ZERO)
                    .count();
                let c = if on == 2 { touch() } else { warn() };
                ring(stage, BOARD.p(side, q), RING_R_POINT * 0.8, c, 4.0)
            })
            .collect();
        let shared = kit.text_left(stage, SHARED, 24.0, touch(), v(BOX_TEXT_X, 0.5))?;
        let mut packing_in = five_tiles;
        packing_in.extend(shared_rings);
        packing_in.push(shared);
        swap(stage, &[], &packing_in, 0.6)?;
        line_7.wait_for(stage, "counting needs")?;
        let next = kit.text_left(stage, NEXT, 24.0, dim(), v(BOX_TEXT_X, 0.0))?;
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

    /// The chapter's lines are its script's, and every phrase a beat waits
    /// for or times a move by is in the line that beat speaks, exactly (case
    /// included), in the order the beat uses them.
    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("06_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
        let source = include_str!("a_triangle_of_points.rs");
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
        assert!(cues >= 11, "only {cues} cues found");
    }

    /// The middle view stays in the frame and left of the panel at every
    /// pose the chapter takes, side lines and the half-plane included; the
    /// box view clears its captions.
    #[test]
    fn the_layout_fits_the_frame() {
        let side = model::turned_side().to_f64();
        let gap = model::turned_gap().to_f64();
        let (big, large_tile) = model::middle_too_large();
        let poses = [
            Key::of(&large_tile, (big[3].0 - big[0].0).to_f64()),
            Key::of(&model::centre_outside(), gap),
            Key::of(&model::five_turned().tiles[MIDDLE], gap),
        ];
        let mut pts: Vec<Point<f64>> = Vec::new();
        for k in 0..=100 {
            let t = model::middle_wander(f64::from(k) / 100.0);
            pts.extend(t.corners());
            for s in 0..4 {
                pts.extend(side_line(&t, s));
            }
        }
        for p in poses {
            let f = blend(p, p, 0.0);
            pts.extend(f.tile.corners());
            pts.extend(square_points(f.size));
            for s in 0..4 {
                pts.extend(side_line(&f.tile, s));
            }
        }
        // the one L drawn: beside the outside centre's diagonal
        let out = model::centre_outside();
        let model::Meets::Beside(k) = model::meets(&out, &model::middle_triangles(model::turned_side())[0]) else {
            panic!("the centre outside is not beside its triangle");
        };
        let (l, region) = beside_region(&blend(poses[1], poses[1], 0.0).tile, k);
        pts.extend(l);
        pts.extend(region);
        for q in pts {
            let p = MID.p(side, q);
            assert!(
                p[0].abs() < FRAME_X_RADIUS - 0.2 && p[1].abs() < FRAME_Y_RADIUS - 0.2,
                "{q:?} at {p:?}"
            );
            assert!(p[0] < TEXT_X - 0.3, "{q:?} reaches the panel");
        }
        assert!(
            BOARD.p(side, (side, side))[0] < BOX_TEXT_X - 0.8,
            "the box clears the panel"
        );
        assert!(ROWS.windows(2).all(|r| r[0] > r[1]));
    }

    /// The poses end on the model's tiles, and the rings say what the
    /// chapter says of them: the long side's tile misses all four points;
    /// the centre outside misses the lower right triangle's three corners
    /// (red) and holds the upper left's (yellow); the tile where five fit
    /// has all four on its edge (green).
    #[test]
    fn the_poses_ring_as_the_proof_says() {
        let gap = model::turned_gap().to_f64();
        let ring_of = |f: &Frame| {
            let tri = MIDDLE_TRIANGLES[f.focus];
            let focus: [bool; 4] = std::array::from_fn(|k| tri.contains(&k));
            Ring::of(square_points(f.size).map(|p| model::depth_inside(&f.tile, p)), focus)
        };
        let (big, large_tile) = model::middle_too_large();
        let large_key = Key::of(&large_tile, (big[3].0 - big[0].0).to_f64());
        let large = blend(large_key, large_key, 0.0);
        assert_eq!(square_points(large.size), big.map(|p| (p.0.to_f64(), p.1.to_f64())));
        assert_eq!(
            ring_of(&large),
            [Ring::Dodged, Ring::Missed, Ring::Dodged, Ring::Dodged]
        );
        let out = blend(
            Key::of(&model::centre_outside(), gap),
            Key::of(&model::centre_outside(), gap),
            0.0,
        );
        assert_eq!(ring_of(&out), [Ring::Dodged, Ring::Inside, Ring::Dodged, Ring::Dodged]);
        let fit = Key::of(&model::five_turned().tiles[MIDDLE], gap);
        assert_eq!(ring_of(&blend(fit, fit, 0.0)), [Ring::Edge; 4]);
        let (rest, start) = (model::middle_wander(1.0), model::middle_wander(0.0));
        assert!((rest.centre.0 - start.centre.0).abs() < 1e-12 && (rest.centre.1 - start.centre.1).abs() < 1e-12);
    }

    /// The side lines run through the middles of the tile's sides, along
    /// them. 06_5's L is what it says: every corner of it is past neither
    /// fence (the other two sides), it touches the diagonal only at the two
    /// tile corners on it, and is beyond the diagonal everywhere else, as is
    /// the tile corner where the two sides meet; the diagonal drawn runs
    /// through the centre and those two tile corners.
    #[test]
    fn the_side_lines_and_the_fenced_region_are_the_tiles() {
        let t = Tile {
            centre: (1.3, 1.2),
            cos: 0.8,
            sin: 0.6,
        };
        for k in 0..4 {
            let [a, b] = side_line(&t, k);
            let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            assert!((model::reach(&t, mid, k) - 0.5).abs() < 1e-12, "{k}");
            assert!((model::reach(&t, a, k) - 0.5).abs() < 1e-12, "{k}");
            let (line, region) = beside_region(&t, k);
            for q in line {
                assert!(model::beyond(&t, q, k).abs() < 1e-12, "{k}");
            }
            assert!(model::beyond(&t, t.centre, k).abs() < 1e-12, "{k}");
            let mut on_diagonal = 0;
            for q in region {
                for fence in [(k + 2) % 4, (k + 3) % 4] {
                    assert!(model::reach(&t, q, fence) <= 0.5 + 1e-12, "{k}: {q:?} past a fence");
                }
                let past = model::beyond(&t, q, k);
                assert!(past > -1e-12, "{k}: {q:?} short of the diagonal");
                if past.abs() < 1e-12 {
                    on_diagonal += 1;
                    assert!(
                        model::depth_inside(&t, q).abs() < 1e-12,
                        "{k}: {q:?} is not a tile corner"
                    );
                }
            }
            assert_eq!(on_diagonal, 2, "{k}");
            let corner_k = t.corners()[(k + 2) % 4];
            assert!(model::beyond(&t, corner_k, k) > 0.0, "{k}: holds its corner");
        }
    }
}
