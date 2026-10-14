//! The projected-cube kit: every solid of Video I, drawn by the scene itself
//! on the fast 2D route (storyboard, "The cubes"). A fixed viewpoint, unit
//! cubes projected to the screen, each face in one of three shades of its
//! solid's colour with thin dark edges, painted back to front.
//!
//! # How it is drawn (and what a frame costs)
//!
//! A face never has its points rewritten. Every face is one unit square in
//! the arena, and what puts it on screen is its *placement* (the engine's
//! object-to-world affine map, `Arena::set_placement`): the projection of a
//! parallelogram is an affine image of the unit square, so a slide, a turn
//! and a tilt of the view are all twelve numbers per face. A solid at rest
//! costs nothing: one updater compares a handful of trackers and returns.
//! A solid that moves costs one placement per visible face, plus one sort of
//! the blocks on stage (below). Colours are rewritten only while a solid
//! turns, fades or is tinted.
//!
//! # Painter's order
//!
//! All faces of one [`Iso`] are the children of one group, and the group's
//! child order is the paint order; the updater re-sorts it whenever anything
//! has moved (`Arena::replace_children`, only if the order really changed).
//! The unit of sorting is a block (a unit cube, or a bigger box): a block's
//! own visible faces never overlap each other. Two blocks are ordered by the
//! exact rule for boxes that share axes: if a plane across one of the three
//! axes separates them, the one on the viewer's side of it is painted later.
//! That rule is used
//!
//! - between two blocks of the same solid, in the solid's own frame (so it
//!   is exact at every instant of a turn), and
//! - between blocks of two solids that are both square to the lattice (at
//!   rest or sliding, at any position, of any size).
//!
//! The pairs are then sorted topologically, farthest first among the free.
//!
//! **Where the order can go wrong.**
//! - A solid in the middle of a turn against *another* solid: no separating
//!   axis is shared, so those pairs fall back to the depth of the blocks'
//!   centres. Keep a turning solid clear of the others on screen (lift it
//!   out, turn it, slide it in), or accept a flicker where they overlap.
//! - Two solids that pass *through* each other (their blocks overlap in
//!   space): also centre depth. Choreograph moves that do not collide.
//! - Three or more boxes of different sizes can overlap in a cycle (each in
//!   front of the next). The sort then releases the farthest one first. Unit
//!   cubes on one lattice never do this.
//! - Faces are not culled between two different solids, so a translucent
//!   solid (mid-fade) shows the faces of what stands behind it.
//!
//! # The view
//!
//! Lattice x runs to the right, y away from the viewer, z up. The viewer
//! stands in front (−y), a little to the right and above: the top, the front
//! and the right side of every solid are seen ([`ViewAt::classic`]). The
//! view is five trackers, so it can be tweened: [`ViewAt::top_down`] looks
//! straight down, and there lattice (x, y) is screen (x, y), a flat board.
//! Tilting costs one placement per visible face per frame, like a slide of
//! everything at once.

#![allow(dead_code, reason = "the kit lands before the chapters that use the rest of it")]

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::f64::consts::FRAC_PI_2;
use std::rc::Rc;

use explainer_kit::{lagged, tween, v};
use fmn::core::color::interpolate_color;
use fmn::mobject::{Placement, Stage as Arena};
use fmn::prelude::*;

/// A unit cell of the lattice, as `model` gives them.
pub type Cell = (i64, i64, i64);
/// A point or a vector in lattice units.
pub type P3 = [f64; 3];
type M3 = [[f64; 3]; 3];
type Turn = [[i64; 3]; 3];

/// How bright a face is: the top, the faces across y (front and back) and
/// the faces across x (the sides).
const TOP: f64 = 1.0;
const FRONT: f64 = 0.74;
const SIDE: f64 = 0.52;
/// The edges: the solid's colour this far towards black, and their width.
const EDGE_DARK: f64 = 0.74;
const EDGE_WIDTH: f64 = 1.6;
/// A face is drawn when its normal leans at least this far to the viewer.
const FACING: f64 = 1e-6;
/// Two boxes this close along an axis touch.
const TOUCH: f64 = 1e-7;
/// A motion's tracker has arrived.
const DONE: f64 = 1.0 - 1e-9;

const IDENTITY: Turn = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];

fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: P3, b: P3) -> P3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn times(a: P3, k: f64) -> P3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn apply(m: &M3, p: P3) -> P3 {
    [dot(m[0], p), dot(m[1], p), dot(m[2], p)]
}

/// `mᵀ p`: a world direction in the frame of a solid turned by `m`.
fn apply_back(m: &M3, p: P3) -> P3 {
    [
        m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2],
        m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2],
        m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2],
    ]
}

fn product(a: &M3, b: &M3) -> M3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    out
}

fn real(m: Turn) -> M3 {
    m.map(|row| row.map(|x| x as f64))
}

/// A lattice axis to turn about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// The turn by `angle` about `axis` (counter-clockwise seen from the axis's
/// positive end).
fn rotation(axis: Axis, angle: f64) -> M3 {
    let (s, c) = angle.sin_cos();
    match axis {
        Axis::X => [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        Axis::Y => [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        Axis::Z => [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
    }
}

/// `quarters` quarter turns about `axis`, exactly (whole numbers).
pub const fn quarter_turns(axis: Axis, quarters: i32) -> Turn {
    let (c, s) = match quarters.rem_euclid(4) {
        0 => (1, 0),
        1 => (0, 1),
        2 => (-1, 0),
        _ => (0, -1),
    };
    match axis {
        Axis::X => [[1, 0, 0], [0, c, -s], [0, s, c]],
        Axis::Y => [[c, 0, s], [0, 1, 0], [-s, 0, c]],
        Axis::Z => [[c, -s, 0], [s, c, 0], [0, 0, 1]],
    }
}

/// `a` after `b`, for two exact turns.
pub const fn compose(a: Turn, b: Turn) -> Turn {
    let mut out = [[0; 3]; 3];
    let mut i = 0;
    while i < 3 {
        let mut j = 0;
        while j < 3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
            j += 1;
        }
        i += 1;
    }
    out
}

/// The cells of a box `w` by `d` by `h` with its corner at the origin (a
/// slab is `box_cells(k, k, 1)`).
pub fn box_cells(w: i64, d: i64, h: i64) -> Vec<Cell> {
    (0..h)
        .flat_map(move |k| (0..d).flat_map(move |j| (0..w).map(move |i| (i, j, k))))
        .collect()
}

/// Where the lattice sits on screen and from where it is seen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewAt {
    /// The screen point of the lattice origin (scene units).
    pub origin: (f64, f64),
    /// Scene units per lattice unit.
    pub scale: f64,
    /// How far the viewer stands to the right of straight in front (radians).
    pub yaw: f64,
    /// How far above the horizon the viewer looks from (radians; a quarter
    /// turn is straight down).
    pub elevation: f64,
}

impl ViewAt {
    /// The video's fixed viewpoint.
    pub const YAW: f64 = 0.50;
    pub const ELEVATION: f64 = 0.62;

    /// The classic "proof without words" view: from in front, a little to
    /// the right and above, so the top, the front and the right side show.
    pub const fn classic(origin: (f64, f64), scale: f64) -> Self {
        Self {
            origin,
            scale,
            yaw: Self::YAW,
            elevation: Self::ELEVATION,
        }
    }

    /// Straight down: lattice (x, y) is screen (x, y), a flat board.
    pub const fn top_down(origin: (f64, f64), scale: f64) -> Self {
        Self {
            origin,
            scale,
            yaw: 0.0,
            elevation: FRAC_PI_2,
        }
    }

    /// The lattice directions of screen right and screen up.
    fn axes(&self) -> (P3, P3) {
        let (sy, cy) = self.yaw.sin_cos();
        let (se, ce) = self.elevation.sin_cos();
        ([cy, sy, 0.0], [-sy * se, cy * se, ce])
    }

    /// The direction from the scene to the viewer.
    pub fn toward_viewer(&self) -> P3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (se, ce) = self.elevation.sin_cos();
        [sy * ce, -cy * ce, se]
    }

    /// Lattice point `p` on screen.
    pub fn project(&self, p: P3) -> Vec3 {
        let (right, up) = self.axes();
        v(
            self.origin.0 + self.scale * dot(right, p),
            self.origin.1 + self.scale * dot(up, p),
        )
    }

    /// The screen box of the lattice box `lo..hi`: (left, right, bottom, top).
    fn extent(&self, lo: P3, hi: P3) -> (f64, f64, f64, f64) {
        let mut out = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for m in 0..8usize {
            let corner = [0, 1, 2].map(|k| if m >> k & 1 == 1 { hi[k] } else { lo[k] });
            let s = self.project(corner);
            out = (out.0.min(s[0]), out.1.max(s[0]), out.2.min(s[1]), out.3.max(s[1]));
        }
        out
    }

    /// The same viewpoint moved so the lattice box `lo..hi` is centred at
    /// the screen point `centre` (the scale is kept).
    #[must_use]
    pub fn centred(self, lo: P3, hi: P3, centre: (f64, f64)) -> Self {
        let (l, r, b, t) = self.extent(lo, hi);
        Self {
            origin: (
                self.origin.0 + centre.0 - (l + r) / 2.0,
                self.origin.1 + centre.1 - (b + t) / 2.0,
            ),
            ..self
        }
    }

    /// The same viewpoint scaled and moved so the lattice box `lo..hi` just
    /// fits a screen room `room` (width, height) centred at `centre`.
    #[must_use]
    pub fn fitted(self, lo: P3, hi: P3, centre: (f64, f64), room: (f64, f64)) -> Self {
        let unit = Self { scale: 1.0, ..self };
        let (l, r, b, t) = unit.extent(lo, hi);
        let scale = (room.0 / (r - l).max(1e-9)).min(room.1 / (t - b).max(1e-9));
        Self { scale, ..self }.centred(lo, hi, centre)
    }
}

/// One box of a solid, in the solid's own lattice coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub at: P3,
    pub size: P3,
}

impl Block {
    /// The unit cube of a cell.
    pub fn unit((i, j, k): Cell) -> Self {
        Self {
            at: [i as f64, j as f64, k as f64],
            size: [1.0; 3],
        }
    }

    /// A cell of a lattice whose cells are `size` big (chapter 08's half
    /// heights are `Block::sized(cell, [1.0, 1.0, 0.5])`).
    pub fn sized((i, j, k): Cell, size: P3) -> Self {
        Self {
            at: [i as f64 * size[0], j as f64 * size[1], k as f64 * size[2]],
            size,
        }
    }

    fn end(&self) -> P3 {
        add(self.at, self.size)
    }
}

/// A solid on stage (a handle; copy it freely).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Solid(usize);

/// What a label is pinned to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    /// A point of the lattice.
    World(P3),
    /// A point of a solid, in the solid's own coordinates: the label rides
    /// with the solid.
    On(Solid, P3),
}

struct Face {
    mob: Mob,
    /// Outward, in the solid's frame.
    normal: P3,
    corner: P3,
    a: P3,
    b: P3,
    visible: bool,
}

struct Motion {
    by: P3,
    turn: Option<(Axis, i32, P3)>,
}

struct Body {
    blocks: Vec<Block>,
    faces: Vec<Face>,
    /// Each block's faces, as a range of `faces`.
    ranges: Vec<(usize, usize)>,
    colour: Srgb,
    tint: Srgb,
    lines: bool,
    rot: Turn,
    pos: P3,
    motion: Option<Motion>,
    /// 0 to 1 along `motion`.
    u: Mob,
    opacity: Mob,
    /// 0 to 1 from `colour` to `tint`.
    glow: Mob,
    /// Bumped whenever the rest pose, the motion or the colours change.
    stamp: u64,
    /// What the last drawn frame was drawn from: stamp, u, opacity, glow.
    seen: Option<(u64, f64, f64, f64)>,
    // the frame now on screen
    m: M3,
    t: P3,
    square: bool,
    shown: bool,
    posed: bool,
}

impl Body {
    /// The solid's turn and its origin's place, `u` of the way along its
    /// motion.
    fn frame(&self, u: f64) -> (M3, P3) {
        let rest = real(self.rot);
        match &self.motion {
            None => (rest, self.pos),
            Some(Motion { by, turn: None }) => (rest, add(self.pos, times(*by, u))),
            Some(Motion {
                by,
                turn: Some((axis, quarters, pivot)),
            }) => {
                let q = rotation(*axis, FRAC_PI_2 * f64::from(*quarters) * u);
                let at = add(apply(&q, sub(self.pos, *pivot)), *pivot);
                (product(&q, &rest), add(at, times(*by, u)))
            }
        }
    }

    /// Where the motion ends, exactly.
    fn arrival(&self, motion: &Motion) -> (Turn, P3) {
        let (rot, mut pos) = match motion.turn {
            None => (self.rot, self.pos),
            Some((axis, quarters, pivot)) => {
                let q = quarter_turns(axis, quarters);
                (compose(q, self.rot), add(apply(&real(q), sub(self.pos, pivot)), pivot))
            }
        };
        pos = add(pos, motion.by);
        // back onto the half lattice, past the last bit of rounding
        for x in &mut pos {
            let half = (*x * 2.0).round() / 2.0;
            if (*x - half).abs() < 1e-6 {
                *x = half;
            }
        }
        (rot, pos)
    }

    fn settle(&mut self) {
        if let Some(motion) = self.motion.take() {
            (self.rot, self.pos) = self.arrival(&motion);
            self.stamp += 1;
        }
    }

    /// The solid's bounds in the lattice for a pose.
    fn bounds(&self, rot: Turn, pos: P3) -> (P3, P3) {
        let m = real(rot);
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        for b in &self.blocks {
            for p in [add(apply(&m, b.at), pos), add(apply(&m, b.end()), pos)] {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
        (lo, hi)
    }
}

struct Pin {
    mob: Mob,
    anchor: Anchor,
    offset: Vec3,
    placed: bool,
}

/// How two boxes with shared axes stand to the viewer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Order {
    /// The first hides the second where they overlap on screen.
    FirstInFront,
    SecondInFront,
    /// They cannot overlap on screen.
    Free,
    /// They overlap in space.
    Unknown,
}

fn order(a: (P3, P3), b: (P3, P3), d: P3) -> Order {
    let (mut first, mut second) = (false, false);
    for k in 0..3 {
        let a_past_b = a.0[k] >= b.1[k] - TOUCH;
        let b_past_a = b.0[k] >= a.1[k] - TOUCH;
        if d[k] > FACING {
            first |= a_past_b;
            second |= b_past_a;
        } else if d[k] < -FACING {
            first |= b_past_a;
            second |= a_past_b;
        } else if a_past_b || b_past_a {
            return Order::Free;
        }
    }
    match (first, second) {
        (true, false) => Order::FirstInFront,
        (false, true) => Order::SecondInFront,
        (true, true) => Order::Free,
        (false, false) => Order::Unknown,
    }
}

struct Item {
    body: usize,
    block: usize,
    depth: f64,
    x: (f64, f64),
    y: (f64, f64),
    bounds: (P3, P3),
}

struct World {
    bodies: Vec<Body>,
    pins: Vec<Pin>,
    group: Mob,
    /// Origin x and y, scale, yaw, elevation, and every solid's opacity.
    view: [Mob; 6],
    seen: Option<[f64; 6]>,
    painted: Vec<Mob>,
    sorts: u64,
}

impl World {
    fn view_now(&self, s: &Arena) -> ([f64; 6], ViewAt) {
        let vals = self.view.map(|m| s.tracker_value(m).unwrap_or(0.0));
        let view = ViewAt {
            origin: (vals[0], vals[1]),
            scale: vals[2],
            yaw: vals[3],
            elevation: vals[4],
        };
        (vals, view)
    }

    /// One frame: place what moved, recolour what changed, repaint the
    /// order if anything moved.
    fn draw(&mut self, s: &mut Arena) {
        let (vals, view) = self.view_now(s);
        let view_moved = self.seen.is_none_or(|w| w[..5] != vals[..5]);
        let all_changed = self.seen.is_none_or(|w| w[5] != vals[5]);
        self.seen = Some(vals);
        let (right, up) = view.axes();
        let d = view.toward_viewer();
        let mut resort = view_moved;
        for body in &mut self.bodies {
            let mut u = s.tracker_value(body.u).unwrap_or(0.0).clamp(0.0, 1.0);
            if body.motion.is_some() && u >= DONE {
                body.settle();
            }
            if body.motion.is_none() {
                u = 0.0;
            }
            let own = s.tracker_value(body.opacity).unwrap_or(0.0).clamp(0.0, 1.0);
            let glow = s.tracker_value(body.glow).unwrap_or(0.0).clamp(0.0, 1.0);
            let now = (body.stamp, u, own, glow);
            let turning = u > 0.0 && body.motion.as_ref().is_some_and(|m| m.turn.is_some());
            let posed = view_moved || body.seen.is_none_or(|x| x.0 != now.0 || x.1 != now.1);
            let styled = all_changed
                || (posed && turning)
                || body.seen.is_none_or(|x| x.0 != now.0 || x.2 != own || x.3 != glow);
            let opacity = own * vals[5].clamp(0.0, 1.0);
            let shown = opacity > 0.0;
            resort |= shown != body.shown;
            body.shown = shown;
            body.seen = Some(now);
            body.posed = posed;
            if !(posed || styled) {
                continue;
            }
            let (m, t) = body.frame(u);
            if posed {
                resort = true;
                body.m = m;
                body.t = t;
                body.square = !turning;
                for f in &mut body.faces {
                    f.visible = dot(apply(&m, f.normal), d) > FACING;
                    if !f.visible {
                        continue;
                    }
                    let (a, b) = (apply(&m, f.a), apply(&m, f.b));
                    let c = add(apply(&m, f.corner), t);
                    let k = view.scale;
                    let linear = [
                        [k * dot(right, a), k * dot(right, b), 0.0],
                        [k * dot(up, a), k * dot(up, b), 0.0],
                        [0.0, 0.0, 1.0],
                    ];
                    let at = [view.origin.0 + k * dot(right, c), view.origin.1 + k * dot(up, c), 0.0];
                    let _ = s.set_placement(f.mob, Placement::new(linear, at));
                }
            }
            if styled {
                let base = if glow > 0.0 {
                    interpolate_color(body.colour, body.tint, glow)
                } else {
                    body.colour
                };
                let black = Srgb::from_rgb8(0, 0, 0);
                let edge = interpolate_color(base, black, EDGE_DARK);
                for f in &body.faces {
                    let n = apply(&m, f.normal);
                    let light = TOP * n[2] * n[2] + FRONT * n[1] * n[1] + SIDE * n[0] * n[0];
                    let fill = interpolate_color(base, black, 1.0 - light);
                    s.set_fill(f.mob, Some(fill), Some(opacity), None, false);
                    let line = if body.lines { edge } else { fill };
                    s.set_stroke(f.mob, Some(line), None, Some(opacity), None, false);
                }
            }
        }
        for pin in &mut self.pins {
            let (p, moved) = match pin.anchor {
                Anchor::World(p) => (p, view_moved),
                Anchor::On(Solid(i), p) => {
                    let body = &self.bodies[i];
                    (add(apply(&body.m, p), body.t), body.posed)
                }
            };
            if moved || !pin.placed {
                let at = view.project(p);
                s.move_to(pin.mob, [at[0] + pin.offset[0], at[1] + pin.offset[1], 0.0], ORIGIN);
                pin.placed = true;
            }
        }
        if resort {
            self.sort(s, &view);
        }
    }

    /// Repaint order: every block on stage, back to front (module doc).
    fn sort(&mut self, s: &mut Arena, view: &ViewAt) {
        let (right, up) = view.axes();
        let d = view.toward_viewer();
        let mut items: Vec<Item> = Vec::new();
        let mut local: Vec<P3> = Vec::with_capacity(self.bodies.len());
        for (bi, body) in self.bodies.iter().enumerate() {
            local.push(apply_back(&body.m, d));
            if !body.shown {
                continue;
            }
            // the screen directions of the solid's three axes
            let spans = [0, 1, 2].map(|k| {
                let e = [body.m[0][k], body.m[1][k], body.m[2][k]];
                (dot(right, e).abs(), dot(up, e).abs())
            });
            for (ki, blk) in body.blocks.iter().enumerate() {
                let (from, to) = body.ranges[ki];
                if !body.faces[from..to].iter().any(|f| f.visible) {
                    continue;
                }
                let c = add(apply(&body.m, add(blk.at, times(blk.size, 0.5))), body.t);
                let (cx, cy) = (dot(right, c), dot(up, c));
                let hx: f64 = (0..3).map(|k| spans[k].0 * blk.size[k] / 2.0).sum();
                let hy: f64 = (0..3).map(|k| spans[k].1 * blk.size[k] / 2.0).sum();
                let (p, q) = (
                    add(apply(&body.m, blk.at), body.t),
                    add(apply(&body.m, blk.end()), body.t),
                );
                let lo = [p[0].min(q[0]), p[1].min(q[1]), p[2].min(q[2])];
                let hi = [p[0].max(q[0]), p[1].max(q[1]), p[2].max(q[2])];
                items.push(Item {
                    body: bi,
                    block: ki,
                    depth: dot(d, c),
                    x: (cx - hx, cx + hx),
                    y: (cy - hy, cy + hy),
                    bounds: (lo, hi),
                });
            }
        }
        let n = items.len();
        let mut later: Vec<Vec<u32>> = vec![Vec::new(); n];
        let mut waits = vec![0u32; n];
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (&items[i], &items[j]);
                if a.x.1 <= b.x.0 + TOUCH || b.x.1 <= a.x.0 + TOUCH || a.y.1 <= b.y.0 + TOUCH || b.y.1 <= a.y.0 + TOUCH
                {
                    continue;
                }
                let found = if a.body == b.body {
                    let blocks = &self.bodies[a.body].blocks;
                    let (p, q) = (&blocks[a.block], &blocks[b.block]);
                    order((p.at, p.end()), (q.at, q.end()), local[a.body])
                } else if self.bodies[a.body].square && self.bodies[b.body].square {
                    order(a.bounds, b.bounds, d)
                } else {
                    Order::Unknown
                };
                let first_in_front = match found {
                    Order::FirstInFront => true,
                    Order::SecondInFront => false,
                    Order::Free => continue,
                    Order::Unknown => a.depth > b.depth,
                };
                let (back, front) = if first_in_front { (j, i) } else { (i, j) };
                later[back].push(front as u32);
                waits[front] += 1;
            }
        }
        // farthest first among the blocks nothing is waiting behind
        let key = |i: usize| ((items[i].depth * 1e6).round() as i64, i);
        let mut ready: BinaryHeap<Reverse<(i64, usize)>> =
            (0..n).filter(|&i| waits[i] == 0).map(|i| Reverse(key(i))).collect();
        let mut done = vec![false; n];
        let mut painted: Vec<Mob> = Vec::with_capacity(self.painted.len());
        let mut left = n;
        while left > 0 {
            let i = match ready.pop() {
                Some(Reverse((_, i))) => i,
                // a cycle: release the farthest block still waiting
                None => match (0..n).filter(|&i| !done[i]).min_by_key(|&i| key(i)) {
                    Some(i) => i,
                    None => break,
                },
            };
            if done[i] {
                continue;
            }
            done[i] = true;
            left -= 1;
            let body = &self.bodies[items[i].body];
            let (from, to) = body.ranges[items[i].block];
            painted.extend(body.faces[from..to].iter().filter(|f| f.visible).map(|f| f.mob));
            for &j in &later[i] {
                let j = j as usize;
                if done[j] {
                    continue;
                }
                waits[j] = waits[j].saturating_sub(1);
                if waits[j] == 0 {
                    ready.push(Reverse(key(j)));
                }
            }
        }
        if painted != self.painted {
            let _ = s.replace_children(self.group, &painted);
            self.painted = painted;
            self.sorts += 1;
        }
    }
}

/// One lattice of solids on stage, with its view. Build it once per scene
/// (or once per picture; two can stand side by side, each paints as one
/// layer in the order they were made).
pub struct Iso {
    world: Rc<RefCell<World>>,
    anchor: Mob,
}

impl Iso {
    /// An empty lattice seen from `view`, on stage from now on. Everything
    /// added to the scene later (text, cards) is drawn over its solids.
    pub fn new(stage: &mut Stage<'_>, view: ViewAt) -> fmn::Result<Self> {
        let group = stage.arena_mut().add(Mobject::new());
        stage.add_to_scene(group)?;
        // the anchor carries the updater and draws nothing
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let vals = [view.origin.0, view.origin.1, view.scale, view.yaw, view.elevation, 1.0];
        let trackers = vals.map(|x| stage.add_value_tracker(x));
        let world = Rc::new(RefCell::new(World {
            bodies: Vec::new(),
            pins: Vec::new(),
            group,
            view: trackers,
            seen: None,
            painted: Vec::new(),
            sorts: 0,
        }));
        let shared = Rc::clone(&world);
        stage.add_updater(anchor, move |s, _| shared.borrow_mut().draw(s), false)?;
        Ok(Self { world, anchor })
    }

    // ---------------------------------------------------------------- solids

    /// A solid of unit cubes (`cells` in its own coordinates, as `model`
    /// gives them) in `colour`, its own origin at lattice point `at`, with
    /// the unit grid drawn on its faces. Hidden until [`Iso::show`] or
    /// [`Iso::fade`].
    pub fn solid(&self, stage: &mut Stage<'_>, cells: &[Cell], colour: Srgb, at: P3) -> fmn::Result<Solid> {
        let blocks: Vec<Block> = cells.iter().map(|&c| Block::unit(c)).collect();
        self.solid_of(stage, &blocks, colour, at, true)
    }

    /// One plain box `size` big with no grid on it (chapter 12's blocks).
    pub fn block(&self, stage: &mut Stage<'_>, size: P3, colour: Srgb, at: P3) -> fmn::Result<Solid> {
        self.solid_of(stage, &[Block { at: [0.0; 3], size }], colour, at, true)
    }

    /// A solid of boxes of any size (chapter 08's half-height cubes are
    /// `Block::sized(cell, [1.0, 1.0, 0.5])`). Each box's faces are outlined
    /// when `lines` is on; with it off the boxes melt into one plain shape.
    /// Faces between two boxes of the solid are never drawn (the hidden one
    /// must be covered whole by one neighbour).
    pub fn solid_of(
        &self,
        stage: &mut Stage<'_>,
        blocks: &[Block],
        colour: Srgb,
        at: P3,
        lines: bool,
    ) -> fmn::Result<Solid> {
        let black = Srgb::from_rgb8(0, 0, 0);
        let style =
            Style::default()
                .fill(colour, 0.0)
                .stroke(interpolate_color(colour, black, EDGE_DARK), EDGE_WIDTH, 0.0);
        let mut faces = Vec::new();
        let mut ranges = Vec::with_capacity(blocks.len());
        for (bi, blk) in blocks.iter().enumerate() {
            let from = faces.len();
            for axis in 0..3 {
                for positive in [false, true] {
                    let plane = if positive { blk.end()[axis] } else { blk.at[axis] };
                    let (ka, kb) = ((axis + 1) % 3, (axis + 2) % 3);
                    let covered = blocks.iter().enumerate().any(|(oi, o)| {
                        let touching = if positive { o.at[axis] } else { o.end()[axis] };
                        oi != bi
                            && (touching - plane).abs() < TOUCH
                            && [ka, kb]
                                .iter()
                                .all(|&k| o.at[k] <= blk.at[k] + TOUCH && o.end()[k] >= blk.end()[k] - TOUCH)
                    });
                    if covered {
                        continue;
                    }
                    let mut corner = blk.at;
                    corner[axis] = plane;
                    let (mut a, mut b, mut normal) = ([0.0; 3], [0.0; 3], [0.0; 3]);
                    a[ka] = blk.size[ka];
                    b[kb] = blk.size[kb];
                    normal[axis] = if positive { 1.0 } else { -1.0 };
                    let quad = Polygon::new([v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)])
                        .style(style)
                        .build();
                    faces.push(Face {
                        mob: stage.arena_mut().add(quad),
                        normal,
                        corner,
                        a,
                        b,
                        visible: false,
                    });
                }
            }
            ranges.push((from, faces.len()));
        }
        let body = Body {
            blocks: blocks.to_vec(),
            faces,
            ranges,
            colour,
            tint: colour,
            lines,
            rot: IDENTITY,
            pos: at,
            motion: None,
            u: stage.add_value_tracker(0.0),
            opacity: stage.add_value_tracker(0.0),
            glow: stage.add_value_tracker(0.0),
            stamp: 0,
            seen: None,
            m: real(IDENTITY),
            t: at,
            square: true,
            shown: false,
            posed: false,
        };
        let mut world = self.world.borrow_mut();
        world.bodies.push(body);
        Ok(Solid(world.bodies.len() - 1))
    }

    /// Show a solid now, at full strength.
    pub fn show(&self, stage: &mut Stage<'_>, solid: Solid) -> fmn::Result<()> {
        let m = self.world.borrow().bodies[solid.0].opacity;
        Ok(stage.set_tracker_value(m, 1.0)?)
    }

    /// Fade a solid to `opacity` (0 hides it, 1 is solid). While it is
    /// translucent, what stands behind it shows through.
    pub fn fade(&self, solid: Solid, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        tween(self.world.borrow().bodies[solid.0].opacity, opacity, rt)
    }

    /// Fade every solid of this lattice together (a multiplier on each
    /// solid's own opacity): a picture that dims under a card.
    pub fn fade_all(&self, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        tween(self.world.borrow().view[5], opacity, rt)
    }

    /// Repaint a solid in another colour, now.
    pub fn recolour(&self, solid: Solid, colour: Srgb) {
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        body.colour = colour;
        body.stamp += 1;
    }

    /// Tween a solid `amount` of the way (0 to 1) from its own colour to
    /// `tint` (the piece moving now, in gold; back with amount 0).
    pub fn glow(&self, solid: Solid, tint: Srgb, amount: f64, rt: f64) -> fmn::Result<AnimBuilder> {
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        body.tint = tint;
        body.stamp += 1;
        tween(body.glow, amount, rt)
    }

    // ---------------------------------------------------------------- motion

    /// The solid's rest pose, with a motion that has finished playing taken
    /// in (and one that never played dropped).
    fn rest(&self, stage: &mut Stage<'_>, solid: Solid) -> fmn::Result<()> {
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        if body.motion.is_some() {
            if stage.tracker_value(body.u).unwrap_or(0.0) >= DONE {
                body.settle();
            } else {
                body.motion = None;
                body.stamp += 1;
            }
        }
        stage.set_tracker_value(body.u, 0.0)?;
        Ok(())
    }

    fn go(&self, stage: &mut Stage<'_>, solid: Solid, motion: Motion, rt: f64) -> fmn::Result<AnimBuilder> {
        self.rest(stage, solid)?;
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        body.motion = Some(motion);
        body.stamp += 1;
        tween(body.u, 1.0, rt)
    }

    /// Put a solid's origin at `at`, now.
    pub fn place(&self, stage: &mut Stage<'_>, solid: Solid, at: P3) -> fmn::Result<()> {
        self.rest(stage, solid)?;
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        body.pos = at;
        body.stamp += 1;
        Ok(())
    }

    /// Slide a solid, in a straight line, until its origin is at `to`.
    pub fn slide(&self, stage: &mut Stage<'_>, solid: Solid, to: P3, rt: f64) -> fmn::Result<AnimBuilder> {
        self.rest(stage, solid)?;
        let by = sub(to, self.world.borrow().bodies[solid.0].pos);
        self.go(stage, solid, Motion { by, turn: None }, rt)
    }

    /// Slide a solid by `by`.
    pub fn slide_by(&self, stage: &mut Stage<'_>, solid: Solid, by: P3, rt: f64) -> fmn::Result<AnimBuilder> {
        self.go(stage, solid, Motion { by, turn: None }, rt)
    }

    /// Slide a solid until the lowest corner of its bounds (least x, y and
    /// z, whatever way it is turned) is at `corner`.
    pub fn land(&self, stage: &mut Stage<'_>, solid: Solid, corner: P3, rt: f64) -> fmn::Result<AnimBuilder> {
        self.rest(stage, solid)?;
        let by = {
            let world = self.world.borrow();
            let body = &world.bodies[solid.0];
            sub(corner, body.bounds(body.rot, body.pos).0)
        };
        self.go(stage, solid, Motion { by, turn: None }, rt)
    }

    /// Turn a solid `quarters` quarter turns about the line along `axis`
    /// through lattice point `pivot` (positive is counter-clockwise seen
    /// from the axis's positive end; 2 is a half turn), as a real turn. It
    /// ends exactly on the lattice when the pivot is a lattice point.
    pub fn turn(
        &self,
        stage: &mut Stage<'_>,
        solid: Solid,
        axis: Axis,
        quarters: i32,
        pivot: P3,
        rt: f64,
    ) -> fmn::Result<AnimBuilder> {
        let turn = Some((axis, quarters, pivot));
        self.go(stage, solid, Motion { by: [0.0; 3], turn }, rt)
    }

    /// Turn a solid `quarters` quarter turns about `axis` through its own
    /// middle while it glides, so that it ends with the lowest corner of its
    /// bounds exactly at `corner`.
    pub fn turn_landing(
        &self,
        stage: &mut Stage<'_>,
        solid: Solid,
        axis: Axis,
        quarters: i32,
        corner: P3,
        rt: f64,
    ) -> fmn::Result<AnimBuilder> {
        self.rest(stage, solid)?;
        let motion = {
            let world = self.world.borrow();
            let body = &world.bodies[solid.0];
            let (lo, hi) = body.bounds(body.rot, body.pos);
            let pivot = times(add(lo, hi), 0.5);
            let mut motion = Motion {
                by: [0.0; 3],
                turn: Some((axis, quarters, pivot)),
            };
            let (rot, pos) = body.arrival(&motion);
            motion.by = sub(corner, body.bounds(rot, pos).0);
            motion
        };
        self.go(stage, solid, motion, rt)
    }

    /// Play several moves (slides, turns, fades, view moves) in one play,
    /// each starting `lag` of a move's length after the one before (0 plays
    /// them together).
    pub fn play(&self, stage: &mut Stage<'_>, moves: Vec<AnimBuilder>, lag: f64) -> fmn::Result<()> {
        let mut prepared: Vec<Box<dyn Animation>> = Vec::with_capacity(moves.len());
        for m in moves {
            prepared.push(stage.prepare(m)?);
        }
        if prepared.is_empty() {
            return Ok(());
        }
        if lag > 0.0 {
            let group = lagged(stage, prepared, lag)?;
            stage.play(group)?;
        } else {
            stage.play_prepared(prepared)?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------ view

    /// Tween the view to `to` (five trackers: play them together, e.g. with
    /// [`Iso::play`] at lag 0). To [`ViewAt::top_down`] this is the tilt
    /// from the solid to the flat board.
    pub fn view_to(&self, to: ViewAt, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
        let t = self.world.borrow().view;
        let vals = [to.origin.0, to.origin.1, to.scale, to.yaw, to.elevation];
        (0..5).map(|k| tween(t[k], vals[k], rt)).collect()
    }

    /// Set the view, now.
    pub fn set_view(&self, stage: &mut Stage<'_>, to: ViewAt) -> fmn::Result<()> {
        let t = self.world.borrow().view;
        let vals = [to.origin.0, to.origin.1, to.scale, to.yaw, to.elevation];
        for k in 0..5 {
            stage.set_tracker_value(t[k], vals[k])?;
        }
        Ok(())
    }

    /// The view as it is now.
    pub fn view(&self, stage: &Stage<'_>) -> ViewAt {
        self.world.borrow().view_now(stage.arena()).1
    }

    /// Lattice point `p` on screen, in the view as it is now.
    pub fn project(&self, stage: &Stage<'_>, p: P3) -> Vec3 {
        self.view(stage).project(p)
    }

    // ---------------------------------------------------------------- labels

    /// Pin a text or TeX mobject (built with the kit, so the lint sees it)
    /// with its centre `offset` (scene units) from the screen place of
    /// `anchor`. It is moved there now and follows the view and, for
    /// [`Anchor::On`], its solid. Bring it in yourself (`fade_in`, `write`).
    pub fn pin(&self, stage: &mut Stage<'_>, mob: Mob, anchor: Anchor, offset: Vec3) -> fmn::Result<()> {
        if let Anchor::On(solid, _) = anchor {
            self.rest_keeping(stage, solid);
        }
        let p = match anchor {
            Anchor::World(p) => p,
            Anchor::On(solid, p) => {
                let world = self.world.borrow();
                let body = &world.bodies[solid.0];
                add(apply(&real(body.rot), p), body.pos)
            }
        };
        let at = self.project(stage, p);
        stage.move_to(mob, [at[0] + offset[0], at[1] + offset[1], 0.0], ORIGIN);
        self.world.borrow_mut().pins.push(Pin {
            mob,
            anchor,
            offset,
            placed: true,
        });
        Ok(())
    }

    /// Stop moving a pinned label (before fading it out, or to leave it).
    pub fn unpin(&self, mob: Mob) {
        self.world.borrow_mut().pins.retain(|p| p.mob != mob);
    }

    // --------------------------------------------------------------- reading

    /// Take in a finished motion without touching one still to play.
    fn rest_keeping(&self, stage: &Stage<'_>, solid: Solid) {
        let mut world = self.world.borrow_mut();
        let body = &mut world.bodies[solid.0];
        if body.motion.is_some() && stage.tracker_value(body.u).unwrap_or(0.0) >= DONE {
            body.settle();
        }
    }

    /// Where a solid's origin rests.
    pub fn position(&self, stage: &Stage<'_>, solid: Solid) -> P3 {
        self.rest_keeping(stage, solid);
        self.world.borrow().bodies[solid.0].pos
    }

    /// How a solid is turned at rest: where its own three axes point.
    pub fn orientation(&self, stage: &Stage<'_>, solid: Solid) -> Turn {
        self.rest_keeping(stage, solid);
        self.world.borrow().bodies[solid.0].rot
    }

    /// The solid's bounds at rest, in the lattice: its lowest and highest
    /// corners.
    pub fn bounds(&self, stage: &Stage<'_>, solid: Solid) -> (P3, P3) {
        self.rest_keeping(stage, solid);
        let world = self.world.borrow();
        let body = &world.bodies[solid.0];
        body.bounds(body.rot, body.pos)
    }

    /// The lattice cells a solid of unit cubes fills at rest, sorted; `None`
    /// if a cube is off the lattice (so a scene can check a move against
    /// `model`).
    pub fn cells(&self, stage: &Stage<'_>, solid: Solid) -> Option<Vec<Cell>> {
        self.rest_keeping(stage, solid);
        let world = self.world.borrow();
        let body = &world.bodies[solid.0];
        let m = real(body.rot);
        let mut out = Vec::with_capacity(body.blocks.len());
        for b in &body.blocks {
            let (p, q) = (add(apply(&m, b.at), body.pos), add(apply(&m, b.end()), body.pos));
            let lo = [p[0].min(q[0]), p[1].min(q[1]), p[2].min(q[2])];
            if lo.iter().any(|x| (x - x.round()).abs() > 1e-9) {
                return None;
            }
            out.push((lo[0].round() as i64, lo[1].round() as i64, lo[2].round() as i64));
        }
        out.sort_unstable();
        Some(out)
    }

    /// How many faces are painted now, and how many times the paint order
    /// has changed (for cost reports).
    pub fn load(&self) -> (usize, u64) {
        let world = self.world.borrow();
        (world.painted.len(), world.sorts)
    }

    /// Stop drawing from the trackers (before `fade_all` it is not needed:
    /// that clears every updater itself).
    pub fn freeze(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stacks;

    #[test]
    fn straight_down_the_lattice_is_the_board() {
        let view = ViewAt::top_down((1.0, -2.0), 0.4);
        for (x, y, z) in [(0.0, 0.0, 0.0), (3.0, 7.0, 0.0), (15.0, 15.0, 4.0)] {
            let p = view.project([x, y, z]);
            assert!((p[0] - (1.0 + 0.4 * x)).abs() < 1e-12 && (p[1] - (-2.0 + 0.4 * y)).abs() < 1e-12);
        }
        let d = view.toward_viewer();
        assert!(d[0].abs() < 1e-12 && d[1].abs() < 1e-12 && (d[2] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_classic_view_shows_the_top_the_front_and_the_right_side() {
        let d = ViewAt::classic((0.0, 0.0), 1.0).toward_viewer();
        assert!(d[0] > 0.1 && d[1] < -0.1 && d[2] > 0.1, "{d:?}");
        let fitted = ViewAt::classic((0.0, 0.0), 1.0).fitted([0.0; 3], [5.0, 3.0, 2.0], (1.0, 1.0), (4.0, 3.0));
        let (l, r, b, t) = fitted.extent([0.0; 3], [5.0, 3.0, 2.0]);
        assert!(((l + r) / 2.0 - 1.0).abs() < 1e-9 && ((b + t) / 2.0 - 1.0).abs() < 1e-9);
        assert!(r - l <= 4.0 + 1e-9 && t - b <= 3.0 + 1e-9);
    }

    #[test]
    fn two_quarter_turns_make_the_second_stacks_turn() {
        // the probe's two turns: a quarter about y, then a quarter back about z
        let turn = compose(quarter_turns(Axis::Z, -1), quarter_turns(Axis::Y, 1));
        assert_eq!(turn, stacks::TURNS[1]);
        assert_eq!(stacks::determinant(turn), 1);
        // and the real turn ends on the exact one
        let q = rotation(Axis::Y, FRAC_PI_2);
        let exact = real(quarter_turns(Axis::Y, 1));
        for i in 0..3 {
            for j in 0..3 {
                assert!((q[i][j] - exact[i][j]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn boxes_are_ordered_by_the_plane_between_them() {
        let d = ViewAt::classic((0.0, 0.0), 1.0).toward_viewer();
        let unit = |x: f64, y: f64, z: f64| ([x, y, z], [x + 1.0, y + 1.0, z + 1.0]);
        // above, to the right and in front are all nearer the viewer
        assert_eq!(order(unit(0.0, 0.0, 1.0), unit(0.0, 0.0, 0.0), d), Order::FirstInFront);
        assert_eq!(order(unit(1.0, 0.0, 0.0), unit(0.0, 0.0, 0.0), d), Order::FirstInFront);
        assert_eq!(order(unit(0.0, 1.0, 0.0), unit(0.0, 0.0, 0.0), d), Order::SecondInFront);
        // a big slab under a small cube that centre depth would get wrong
        let slab = ([0.0, 0.0, 0.0], [9.0, 9.0, 1.0]);
        assert_eq!(order(unit(0.0, 8.0, 1.0), slab, d), Order::FirstInFront);
        // in front on one axis and behind on another: they cannot overlap
        assert_eq!(order(unit(1.0, 1.0, 0.0), unit(0.0, 0.0, 0.0), d), Order::Free);
        assert_eq!(order(unit(0.5, 0.5, 0.5), unit(0.0, 0.0, 0.0), d), Order::Unknown);
    }
}
