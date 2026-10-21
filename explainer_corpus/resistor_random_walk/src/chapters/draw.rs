//! The video's look, shared by its chapters (storyboard, color roles): the
//! network drawn as junctions and identical wires (a link of conductance k
//! is k wires side by side), the battery, values that travel along wires,
//! the think-time countdown, and line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, short strokes.
//!
//! The playback and line-timing helpers are the Golay chapters' (accepted
//! centrally), kept to the ones this video uses.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, v};
use fmn::prelude::*;

use crate::model::{BOTTOM, JUNCTIONS, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ----------------------------------------------------------------- geometry

/// Where each junction of the opening network sits, from its picture's
/// centre, in the model's order (top, left, right, bottom): a diamond, the
/// top above the bottom, the left and the right between them.
pub const SPOTS: [[f64; 2]; JUNCTIONS] = [[0.0, 2.0], [-1.7, 0.0], [1.7, 0.0], [0.0, -2.0]];
/// A junction's radius.
pub const JUNCTION_R: f64 = 0.17;
/// How far apart two wires side by side bow at their middle.
pub const SPREAD: f64 = 0.2;
/// A wire's stroke width.
pub const WIRE_W: f64 = 4.0;
/// How far a travelling value sits from its wire: a fraction's half height
/// at [`TOKEN_SIZE`] and a margin, so the wire never crosses it.
pub const TOKEN_GAP: f64 = 0.32;
/// How far a middle junction's value sits from it, sideways.
pub const LABEL_GAP: f64 = 0.55;
/// How far the top's and the bottom's labels sit to their left.
pub const END_LABEL_GAP: f64 = 0.95;
/// The battery's wire, right of the picture's centre.
pub const BATTERY_X: f64 = 3.1;
/// Half the battery plates' widths, long and short, and their height off
/// the centre line.
const PLATE_HALF: [f64; 2] = [0.3, 0.15];
const PLATE_Y: f64 = 0.14;
/// An arrowhead's length, and where it sits along its wire from the
/// junction (a fraction of the wire).
const ARROW: f64 = 0.16;
const ARROW_T: f64 = 0.3;
/// A picture's name, above its top junction; its equation and notes, below
/// its bottom junction.
pub const NAME_DY: f64 = 2.8;
pub const EQUATION_DY: f64 = -2.75;

/// The text sizes: a junction's value, a value travelling along a wire, an
/// averaging equation.
pub const LABEL_SIZE: f64 = 38.0;
pub const TOKEN_SIZE: f64 = 30.0;
pub const EQUATION_SIZE: f64 = 40.0;

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// A value as TeX: `\frac{3}{5}`, or the whole number.
pub fn tex(q: Q) -> String {
    match q.whole() {
        Some(n) => n.to_string(),
        None if q.numer() < 0 => format!(r"-\frac{{{}}}{{{}}}", -q.numer(), q.denom()),
        None => format!(r"\frac{{{}}}{{{}}}", q.numer(), q.denom()),
    }
}

/// A value as TeX inside a larger fraction: `\tfrac{2}{5}`.
pub fn small_tex(q: Q) -> String {
    tex(q).replace(r"\frac", r"\tfrac")
}

/// The averaging rule at one junction as TeX: its wires' far-end values
/// over their count, equal to its value.
pub fn equation(ends: &[Q], value: Q) -> String {
    let terms: Vec<String> = ends.iter().map(|&q| small_tex(q)).collect();
    format!(r"\frac{{{}}}{{{}}} = {}", terms.join(" + "), ends.len(), tex(value))
}

/// "1 volt", "0 volts".
pub fn volts_text(q: Q) -> String {
    if q == Q::ONE {
        format!("{q} volt")
    } else {
        format!("{q} volts")
    }
}

// --------------------------------------------------------------- the network

/// One identical wire: the junctions it joins and the side it bows to (0
/// for a wire alone; ±1, in half-spreads, for a pair side by side).
pub struct Strand {
    pub mob: Mob,
    pub a: usize,
    pub b: usize,
    pub side: f64,
}

/// The network drawn around `centre`: its junctions (by index) and its
/// identical wires, in the order `Network::far_ends` lists them.
pub struct Picture {
    pub centre: [f64; 2],
    pub junctions: Vec<Mob>,
    pub strands: Vec<Strand>,
}

/// The unit normal to the left of the straight line from junction `a` to
/// junction `b`.
fn normal(a: usize, b: usize) -> [f64; 2] {
    let (dx, dy) = (SPOTS[b][0] - SPOTS[a][0], SPOTS[b][1] - SPOTS[a][1]);
    let len = dx.hypot(dy).max(1e-9);
    [-dy / len, dx / len]
}

/// `p` moved `r` towards `q`.
fn towards(p: Vec3, q: Vec3, r: f64) -> Vec3 {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let len = dx.hypot(dy).max(1e-9);
    v(p[0] + r * dx / len, p[1] + r * dy / len)
}

impl Picture {
    /// Junction `x`'s centre.
    pub fn spot(&self, x: usize) -> Vec3 {
        v(self.centre[0] + SPOTS[x][0], self.centre[1] + SPOTS[x][1])
    }

    /// The middle of a strand, bowed to its side.
    pub fn mid(&self, s: &Strand) -> Vec3 {
        self.bowed_mid(s.a, s.b, s.side)
    }

    /// The middle of a wire from `a` to `b` bowed `side` half-spreads to
    /// the left of that direction.
    fn bowed_mid(&self, a: usize, b: usize, side: f64) -> Vec3 {
        let (p, q, n) = (self.spot(a), self.spot(b), normal(a, b));
        let bow = side * SPREAD / 2.0;
        v((p[0] + q[0]) / 2.0 + bow * n[0], (p[1] + q[1]) / 2.0 + bow * n[1])
    }

    /// The strands at junction `x`, in the order `Network::far_ends(x)`
    /// lists their far ends.
    pub fn strands_at(&self, x: usize) -> Vec<&Strand> {
        self.strands.iter().filter(|s| s.a == x || s.b == x).collect()
    }

    /// The wires at junction `x`.
    pub fn wires_at(&self, x: usize) -> Vec<Mob> {
        self.strands_at(x).iter().map(|s| s.mob).collect()
    }

    /// The wires that run side by side with another.
    pub fn doubled(&self) -> Vec<Mob> {
        self.strands.iter().filter(|s| s.side != 0.0).map(|s| s.mob).collect()
    }

    /// The first wire joining `x` and `y`.
    pub fn strand_between(&self, x: usize, y: usize) -> fmn::Result<&Strand> {
        self.strands
            .iter()
            .find(|s| (s.a, s.b) == (x, y) || (s.a, s.b) == (y, x))
            .ok_or_else(|| fail(format!("no wire joins junctions {x} and {y}")))
    }

    /// The point beside strand `s` a fraction `t` of the way from junction
    /// `from` to its far end, on the side the strand bows to (a wire alone:
    /// its left, seen from `a` to `b`).
    pub fn beside(&self, s: &Strand, from: usize, t: f64) -> Vec3 {
        let to = if s.a == from { s.b } else { s.a };
        let (p, m, q) = (self.spot(from), self.mid(s), self.spot(to));
        let (a, b, u) = if t <= 0.5 {
            (p, m, 2.0 * t)
        } else {
            (m, q, 2.0 * t - 1.0)
        };
        let n = normal(s.a, s.b);
        let side = if s.side == 0.0 { 1.0 } else { s.side.signum() };
        v(
            a[0] + (b[0] - a[0]) * u + side * TOKEN_GAP * n[0],
            a[1] + (b[1] - a[1]) * u + side * TOKEN_GAP * n[1],
        )
    }

    /// Where junction `x`'s value is written: beside a middle junction, on
    /// its outer side; left of the top and the bottom, `gap` away.
    pub fn label_spot(&self, x: usize, gap: f64) -> Vec3 {
        let p = self.spot(x);
        match x {
            LEFT => v(p[0] - LABEL_GAP, p[1]),
            RIGHT => v(p[0] + LABEL_GAP, p[1]),
            _ => v(p[0] - gap, p[1] + 0.05),
        }
    }

    /// Where the picture's name goes.
    pub fn name_spot(&self) -> Vec3 {
        v(self.centre[0], self.centre[1] + NAME_DY)
    }

    /// Where the picture's averaging equation and notes go.
    pub fn equation_spot(&self) -> Vec3 {
        v(self.centre[0], self.centre[1] + EQUATION_DY)
    }

    /// The junctions and the wires, to move or fade together.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all: Vec<Mob> = self.strands.iter().map(|s| s.mob).collect();
        all.extend(&self.junctions);
        all
    }
}

/// The network drawn around `centre`, junction `x` filled with `colors[x]`;
/// a wire of whole conductance k is k identical wires side by side.
/// Detached.
pub fn picture(stage: &mut Stage<'_>, net: &Network, centre: [f64; 2], colors: &[Srgb]) -> fmn::Result<Picture> {
    if net.size() != JUNCTIONS || colors.len() != JUNCTIONS {
        return Err(fail(format!(
            "the picture draws {JUNCTIONS} junctions, not {} with {} colors",
            net.size(),
            colors.len()
        )));
    }
    let mut pic = Picture {
        centre,
        junctions: Vec::with_capacity(JUNCTIONS),
        strands: Vec::new(),
    };
    for w in net.wires() {
        let k = w
            .conductance
            .whole()
            .filter(|&k| k > 0)
            .ok_or_else(|| fail(format!("the wire {}-{} is not a whole number of wires", w.a, w.b)))?;
        for j in 0..k {
            let side = (2 * j - (k - 1)) as f64;
            let mid = pic.bowed_mid(w.a, w.b, side);
            let (p, q) = (pic.spot(w.a), pic.spot(w.b));
            let path = vec![towards(p, mid, JUNCTION_R), mid, towards(q, mid, JUNCTION_R)];
            let mob = stage
                .arena_mut()
                .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build());
            pic.strands.push(Strand {
                mob,
                a: w.a,
                b: w.b,
                side,
            });
        }
    }
    for (x, &color) in colors.iter().enumerate() {
        let dot = Circle::new()
            .radius(JUNCTION_R)
            .style(flat(color, 1.0))
            .build()
            .moved_to(pic.spot(x));
        pic.junctions.push(stage.arena_mut().add(dot));
    }
    Ok(pic)
}

/// The battery beside a picture: a wire from the top round to the right,
/// down through the two plates (the long one, positive, towards the top),
/// and back to the bottom. Detached.
pub fn battery(stage: &mut Stage<'_>, centre: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let at = |x: f64, y: f64| v(centre[0] + x, centre[1] + y);
    let (top, bottom) = (SPOTS[TOP][1], SPOTS[BOTTOM][1]);
    let paths = [
        vec![at(JUNCTION_R, top), at(BATTERY_X, top), at(BATTERY_X, PLATE_Y)],
        vec![
            at(BATTERY_X - PLATE_HALF[0], PLATE_Y),
            at(BATTERY_X + PLATE_HALF[0], PLATE_Y),
        ],
        vec![
            at(BATTERY_X - PLATE_HALF[1], -PLATE_Y),
            at(BATTERY_X + PLATE_HALF[1], -PLATE_Y),
        ],
        vec![at(BATTERY_X, -PLATE_Y), at(BATTERY_X, bottom), at(JUNCTION_R, bottom)],
    ];
    Ok(paths
        .into_iter()
        .map(|path| {
            stage
                .arena_mut()
                .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build())
        })
        .collect())
}

/// One straight wire from junction `a` to junction `b` of `pic`, `width`
/// wide: a link drawn as one heavier wire (chapter 03's merged pairs,
/// chapter 10's unequal wires). Detached.
pub fn heavy_link(stage: &mut Stage<'_>, pic: &Picture, a: usize, b: usize, width: f64) -> Mob {
    let (p, q) = (pic.spot(a), pic.spot(b));
    let path = vec![towards(p, q, JUNCTION_R), towards(q, p, JUNCTION_R)];
    stage
        .arena_mut()
        .add(polyline(path).style(line_style(palette::wire(), width)).build())
}

/// The opening's shape with each link of `net` drawn as one wire, its
/// stroke heavier in proportion to its conductance (chapters 10 and 11),
/// junction `x` filled with `colors[x]`. Its strands are its links, one
/// each, in wire order. Detached.
pub fn weighted_picture(
    stage: &mut Stage<'_>,
    net: &Network,
    centre: [f64; 2],
    colors: &[Srgb],
) -> fmn::Result<Picture> {
    if net.size() != JUNCTIONS || colors.len() != JUNCTIONS {
        return Err(fail(format!(
            "the picture draws {JUNCTIONS} junctions, not {} with {} colors",
            net.size(),
            colors.len()
        )));
    }
    let mut pic = Picture {
        centre,
        junctions: Vec::with_capacity(JUNCTIONS),
        strands: Vec::new(),
    };
    for w in net.wires() {
        let mob = heavy_link(stage, &pic, w.a, w.b, WIRE_W * w.conductance.f64());
        pic.strands.push(Strand {
            mob,
            a: w.a,
            b: w.b,
            side: 0.0,
        });
    }
    for (x, &color) in colors.iter().enumerate() {
        let dot = Circle::new()
            .radius(JUNCTION_R)
            .style(flat(color, 1.0))
            .build()
            .moved_to(pic.spot(x));
        pic.junctions.push(stage.arena_mut().add(dot));
    }
    Ok(pic)
}

// ---------------------------------------------------------------- the line

/// The line of chapters 01 and 02, standing up: how far apart its junctions
/// are, how far left of a junction its value is written, and how far right
/// its battery's wire runs.
pub const LINE_STEP: f64 = 0.95;
pub const LINE_LABEL_GAP: f64 = 0.6;
pub const LINE_BATTERY_X: f64 = 1.5;

/// A line of identical wires drawn around `centre`, junction 0 (the
/// bottom) at the foot and the last (the top) at the head.
pub struct LinePicture {
    pub centre: [f64; 2],
    pub size: usize,
    pub junctions: Vec<Mob>,
    /// The wire from junction `i` up to junction `i + 1`.
    pub wires: Vec<Mob>,
}

impl LinePicture {
    /// Junction `x`'s centre.
    pub fn spot(&self, x: usize) -> Vec3 {
        line_spot(self.centre, self.size, x)
    }

    /// Where junction `x`'s value is written, left of it.
    pub fn label_spot(&self, x: usize) -> Vec3 {
        let p = self.spot(x);
        v(p[0] - LINE_LABEL_GAP, p[1])
    }

    /// The middle of the wire from junction `i` up to `i + 1`, moved `dx`
    /// to the right.
    pub fn wire_mid(&self, i: usize, dx: f64) -> Vec3 {
        let (p, q) = (self.spot(i), self.spot(i + 1));
        v((p[0] + q[0]) / 2.0 + dx, (p[1] + q[1]) / 2.0)
    }

    /// The wires at junction `x`: the one below it, then the one above it.
    pub fn wires_at(&self, x: usize) -> Vec<Mob> {
        let mut out = Vec::new();
        if x > 0 {
            out.push(self.wires[x - 1]);
        }
        if x < self.wires.len() {
            out.push(self.wires[x]);
        }
        out
    }

    /// Where the line's name goes, above its top.
    pub fn name_spot(&self) -> Vec3 {
        let top = self.spot(self.size - 1);
        v(top[0], top[1] + 0.6)
    }

    /// The junctions and the wires, to move or fade together.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.wires.clone();
        all.extend(&self.junctions);
        all
    }
}

/// Junction `x`'s centre on a line of `size` junctions around `centre`.
pub fn line_spot(centre: [f64; 2], size: usize, x: usize) -> Vec3 {
    let half = size.saturating_sub(1) as f64 / 2.0;
    v(centre[0], centre[1] + LINE_STEP * (x as f64 - half))
}

/// The line `net` (junction `x` joined to `x + 1` by one identical wire
/// each) drawn around `centre`, junction `x` filled with `colors[x]`.
/// Detached.
pub fn line_picture(
    stage: &mut Stage<'_>,
    net: &Network,
    centre: [f64; 2],
    colors: &[Srgb],
) -> fmn::Result<LinePicture> {
    let is_line = net.wires().len() + 1 == net.size()
        && net
            .wires()
            .iter()
            .enumerate()
            .all(|(i, w)| (w.a, w.b) == (i, i + 1) && w.conductance == Q::ONE);
    if !is_line || colors.len() != net.size() {
        return Err(fail(format!(
            "a line of {} junctions with {} colors is not one identical wire from each junction to the next",
            net.size(),
            colors.len()
        )));
    }
    let mut pic = LinePicture {
        centre,
        size: net.size(),
        junctions: Vec::with_capacity(net.size()),
        wires: Vec::with_capacity(net.wires().len()),
    };
    for i in 0..net.wires().len() {
        let (p, q) = (pic.spot(i), pic.spot(i + 1));
        let path = vec![v(p[0], p[1] + JUNCTION_R), v(q[0], q[1] - JUNCTION_R)];
        let mob = stage
            .arena_mut()
            .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build());
        pic.wires.push(mob);
    }
    for (x, &color) in colors.iter().enumerate() {
        let dot = Circle::new()
            .radius(JUNCTION_R)
            .style(flat(color, 1.0))
            .build()
            .moved_to(pic.spot(x));
        pic.junctions.push(stage.arena_mut().add(dot));
    }
    Ok(pic)
}

/// The battery beside a line: a wire from the top out to the right, down
/// through the two plates at the line's middle height (the long one,
/// positive, towards the top), and back in to the bottom. Detached.
pub fn line_battery(stage: &mut Stage<'_>, pic: &LinePicture) -> fmn::Result<Vec<Mob>> {
    let (top, bottom) = (pic.spot(pic.size - 1), pic.spot(0));
    let (x, y) = (pic.centre[0] + LINE_BATTERY_X, pic.centre[1]);
    let paths = [
        vec![v(top[0] + JUNCTION_R, top[1]), v(x, top[1]), v(x, y + PLATE_Y)],
        vec![v(x - PLATE_HALF[0], y + PLATE_Y), v(x + PLATE_HALF[0], y + PLATE_Y)],
        vec![v(x - PLATE_HALF[1], y - PLATE_Y), v(x + PLATE_HALF[1], y - PLATE_Y)],
        vec![v(x, y - PLATE_Y), v(x, bottom[1]), v(bottom[0] + JUNCTION_R, bottom[1])],
    ];
    Ok(paths
        .into_iter()
        .map(|path| {
            stage
                .arena_mut()
                .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build())
        })
        .collect())
}

/// The walker as a dot of radius `r` at `at`, at `opacity` (a ghost is
/// faint). Detached.
pub fn walker_dot(stage: &mut Stage<'_>, at: Vec3, r: f64, opacity: f64) -> Mob {
    let dot = Circle::new()
        .radius(r)
        .style(flat(palette::walker(), opacity))
        .build()
        .moved_to(at);
    stage.arena_mut().add(dot)
}

/// One value per wire at junction `x` (`texts[i]` beside its `i`th wire,
/// in that wire's far end's color), a little past the wire's middle from
/// `x`. Detached.
pub fn tokens(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pic: &Picture,
    x: usize,
    texts: &[(String, Srgb)],
) -> fmn::Result<Vec<Mob>> {
    tokens_at(kit, stage, pic, x, texts, 0.62)
}

/// As [`tokens`], each value a fraction `t` of the way along its wire from
/// `x` (chapter 02 writes a wire's drop at its middle).
pub fn tokens_at(
    kit: &Kit,
    stage: &mut Stage<'_>,
    pic: &Picture,
    x: usize,
    texts: &[(String, Srgb)],
    t: f64,
) -> fmn::Result<Vec<Mob>> {
    let strands = pic.strands_at(x);
    if strands.len() != texts.len() {
        return Err(fail(format!(
            "junction {x} has {} wires but {} values",
            strands.len(),
            texts.len()
        )));
    }
    let mut out = Vec::with_capacity(texts.len());
    for (s, (src, color)) in strands.iter().zip(texts) {
        out.push(kit.tex(stage, src, TOKEN_SIZE, *color, &[], pic.beside(s, x, t))?);
    }
    Ok(out)
}

/// An arrowhead on each wire at junction `x`, a fraction `t` along it from
/// `x`, pointing into `x` where `inward[i]` holds and away from it
/// otherwise (the way each wire's current flows). Stroke-only chevrons, in
/// `color`. Detached.
pub fn arrowheads(
    stage: &mut Stage<'_>,
    pic: &Picture,
    x: usize,
    inward: &[bool],
    color: Srgb,
) -> fmn::Result<Vec<Mob>> {
    let strands = pic.strands_at(x);
    if strands.len() != inward.len() {
        return Err(fail(format!(
            "junction {x} has {} wires but {} directions",
            strands.len(),
            inward.len()
        )));
    }
    let mut out = Vec::with_capacity(strands.len());
    for (s, &towards_x) in strands.iter().zip(inward) {
        // on the first half of the wire, from the junction to the bow
        let (p, m) = (pic.spot(x), pic.mid(s));
        let at = v(
            p[0] + (m[0] - p[0]) * 2.0 * ARROW_T,
            p[1] + (m[1] - p[1]) * 2.0 * ARROW_T,
        );
        let len = (m[0] - p[0]).hypot(m[1] - p[1]).max(1e-9);
        let sign = if towards_x { -1.0 } else { 1.0 };
        let (dx, dy) = (sign * (m[0] - p[0]) / len, sign * (m[1] - p[1]) / len);
        let tip = v(at[0] + dx * ARROW / 2.0, at[1] + dy * ARROW / 2.0);
        let back = |side: f64| {
            v(
                tip[0] - dx * ARROW - side * dy * ARROW * 0.6,
                tip[1] - dy * ARROW + side * dx * ARROW * 0.6,
            )
        };
        let chevron = polyline(vec![back(1.0), tip, back(-1.0)])
            .style(line_style(color, WIRE_W))
            .build();
        out.push(stage.arena_mut().add(chevron));
    }
    Ok(out)
}

/// The point beside the link joining junctions `a` and `b` (a lone wire or
/// a pair side by side), `gap` from its straight middle, on its left seen
/// from `a` (a negative `gap` puts it on the right).
pub fn link_spot(pic: &Picture, a: usize, b: usize, gap: f64) -> Vec3 {
    let (p, q, n) = (pic.spot(a), pic.spot(b), normal(a, b));
    v((p[0] + q[0]) / 2.0 + gap * n[0], (p[1] + q[1]) / 2.0 + gap * n[1])
}

/// Where a label for the link joining `a` and `b` goes: `gap` beside its
/// straight middle, on the side away from the picture's centre (a link
/// through the centre: its left, seen from `a`).
pub fn outside_link_spot(pic: &Picture, a: usize, b: usize, gap: f64) -> Vec3 {
    let (p, q) = (pic.spot(a), pic.spot(b));
    let (mx, my) = ((p[0] + q[0]) / 2.0 - pic.centre[0], (p[1] + q[1]) / 2.0 - pic.centre[1]);
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let outward = -dy * mx + dx * my >= 0.0;
    link_spot(pic, a, b, if outward { gap } else { -gap })
}

/// The values `values` as tokens (each in its value's color).
pub fn value_texts(values: &[Q]) -> Vec<(String, Srgb)> {
    values.iter().map(|&q| (tex(q), palette::value(q))).collect()
}

// --------------------------------------------------------------- playback

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

/// Flash several mobjects (the highlight role), together.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move several mobjects together by `(dx, dy)` over `rt` seconds.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64, rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(v(dx, dy))?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move each mobject to its point, together, over `rt` seconds.
pub fn move_all(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], rt: f64) -> fmn::Result<()> {
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
    stage.play_prepared(anims)?;
    Ok(())
}

/// Turn each mobject to its color, together, over `rt` seconds.
pub fn recolor(stage: &mut Stage<'_>, mobs: &[(Mob, Srgb)], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, color) in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_color(color)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade `old` out and the detached `new` in, in one move.
pub fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in old {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in new {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if anims.is_empty() {
        return Ok(());
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Slide travelling values into junction `to` and let them vanish there.
pub fn slide_in(stage: &mut Stage<'_>, tokens: &[Mob], to: Vec3, rt: f64) -> fmn::Result<()> {
    let moves: Vec<(Mob, Vec3)> = tokens.iter().map(|&m| (m, to)).collect();
    move_all(stage, &moves, rt)?;
    fade_out_all(stage, tokens, 0.2)
}

/// Wait `seconds` (a beat of stillness after a reveal).
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
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
fn sentence_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![0];
    for (i, pair) in chars.windows(2).enumerate() {
        if matches!(pair[0], '.' | '?' | '!') && pair[1] == ' ' && i + 2 < chars.len() {
            out.push(i + 2);
        }
    }
    out
}

/// Line `id`'s sentences, each with its start in seconds into the line:
/// the narrator's `measured` onsets, one per sentence, or with no narrator
/// (an empty list) each sentence's share of `len` by characters. A
/// narrator that splits the line differently is an error, never a guess.
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

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown and "pause for more time").
pub const THINK_S: f64 = 4.0;
pub const THINK_R: f64 = 0.62;
pub const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause: let the question finish, mark the moment for the
/// score, bring in a large "?" at `at` inside a gold ring that drains over
/// [`THINK_S`] with "pause for more time" under it; at zero the ring and the
/// note go. Returns the "?", which the scene clears with the answer.
pub fn think(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 80.0, Palette::hero(), at)?;
    let note = kit.text(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0], at[1] - THINK_R - 0.3))?;
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
    fade_in_all(stage, &[anchor, ring, q, note], 0.2)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.2)?;
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn values_print_as_tex() {
        assert_eq!(tex(Q::new(3, 5)), r"\frac{3}{5}");
        assert_eq!(tex(Q::ONE), "1");
        assert_eq!(tex(Q::new(-1, 2)), r"-\frac{1}{2}");
        assert_eq!(small_tex(Q::new(2, 5)), r"\tfrac{2}{5}");
        assert_eq!(volts_text(Q::ONE), "1 volt");
        assert_eq!(volts_text(Q::ZERO), "0 volts");
    }

    /// The drawn order of a junction's wires is the order the model lists
    /// their far ends in, so a value slides along the wire it belongs to.
    #[test]
    fn the_strands_follow_the_model_order() {
        let net = model::opening().unwrap();
        let mut order = Vec::new();
        for w in net.wires() {
            for _ in 0..w.conductance.whole().unwrap() {
                order.push((w.a, w.b));
            }
        }
        for x in net.middle() {
            let drawn: Vec<usize> = order
                .iter()
                .filter(|&&(a, b)| a == x || b == x)
                .map(|&(a, b)| if a == x { b } else { a })
                .collect();
            assert_eq!(drawn, net.far_ends(x).unwrap(), "junction {x}");
        }
    }

    /// Two wires side by side bow apart, and every wire's ends sit outside
    /// both junctions it joins.
    #[test]
    fn the_diamond_keeps_its_wires_apart() {
        // the spots follow the model's junction order
        assert!(SPOTS[TOP][1] > SPOTS[LEFT][1] && SPOTS[LEFT][1] > SPOTS[BOTTOM][1]);
        assert!(SPOTS[LEFT][0] < SPOTS[TOP][0] && SPOTS[TOP][0] < SPOTS[RIGHT][0]);
        let d = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).hypot(p[1] - q[1]);
        for (a, b) in [(TOP, LEFT), (LEFT, RIGHT), (RIGHT, BOTTOM)] {
            assert!(d(SPOTS[a], SPOTS[b]) > 4.0 * JUNCTION_R + SPREAD);
            let n = normal(a, b);
            assert!((n[0].hypot(n[1]) - 1.0).abs() < 1e-12);
        }
        // a value beside a wire clears the wire next to it in a pair
        const { assert!(TOKEN_GAP > SPREAD / 2.0 + 0.1) };
        // the battery's plates clear the right junction's value
        const { assert!(BATTERY_X - PLATE_HALF[0] > 1.7 + LABEL_GAP + 0.5) };
    }

    /// A link's outside label sits further from the picture's centre than
    /// the link's middle, for every link of the diamond off the centre.
    #[test]
    fn outside_labels_sit_outside() {
        for (a, b) in [(TOP, LEFT), (TOP, RIGHT), (LEFT, BOTTOM), (RIGHT, BOTTOM)] {
            let (p, q) = (SPOTS[a], SPOTS[b]);
            let (mx, my) = ((p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0);
            let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
            let n = normal(a, b);
            let sign = if -dy * mx + dx * my >= 0.0 { 1.0 } else { -1.0 };
            let at = (mx + sign * 0.35 * n[0], my + sign * 0.35 * n[1]);
            assert!(at.0.hypot(at.1) > mx.hypot(my), "the {a}-{b} label sits inside");
        }
    }

    #[test]
    fn sentences_split_like_the_narrator() {
        assert_eq!(sentence_offsets("One. Two? Three."), [0, 5, 10]);
        let timed = timed_sentences("x", "Ab. Cd.", 7.0, &[]).unwrap();
        assert_eq!(timed[0], (0, 0.0));
        assert!(timed_sentences("x", "Ab. Cd.", 7.0, &[0.0]).is_err());
    }
}
