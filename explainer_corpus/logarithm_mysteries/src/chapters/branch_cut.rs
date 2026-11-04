//! Chapter 07, giving something up: a cut from zero to infinity that no
//! walk may cross, so no loop winds around zero (07_1); the principal
//! logarithm Log on the plane with zero and the negative numbers removed,
//! angles strictly between −π and π, derivative 1/z, worked at ±i: Log i =
//! πi/2, Log(−i) = −πi/2 (07_2); the cut plane unrolled into the strip
//! −π < Im < π, circles to vertical segments, rays to horizontal lines; then
//! the cut's two sides are marked on the cut plane and the viewer predicts
//! where they go (07_3); they go to the strip's two edges, angles near π and
//! near −π, so crossing jumps 2π (07_4); other cuts and the branch point
//! (07_5); the convention on the negative axis and the one-sided inverse
//! (07_6).
//!
//! Domains, as screen text states them (DLMF §4.2): Log is holomorphic on
//! C \ (−∞, 0] with values in −π < Im < π; the principal value extends to
//! the negative axis with Arg = π, so pointwise Im Log ∈ (−π, π];
//! exp(Log z) = z for all z ≠ 0; Log(e^w) = w exactly when −π < Im w ≤ π.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Plane, answer_c, arc_points, dashed, dot, fact_list, fade_in_all, fade_out_all, faded_line, flat, lap_c,
    line_style, number_c, position_c, precisely, predict, ring, seg, swap, title, value, warn_c,
};
use crate::model::C;

pub struct BranchCut {
    pub kit: Kit,
}

const Z: Plane = Plane {
    centre: [-3.4, -0.3, 0.0],
    scale: 1.0,
    half_w: 2.9,
    half_h: 2.7,
};
/// Where Log's strip is drawn when the cut plane unrolls in place: the same
/// centre, `W_SCALE` scene units per unit, so ±π sits at ±2.5.
const W_SCALE: f64 = 0.8;
const RADII: [f64; 5] = [0.4, 0.7, 1.0, 1.5, 2.3];
const R_MIN: f64 = 0.3;
const R_MAX: f64 = 2.6;
const RAYS: [i32; 7] = [-3, -2, -1, 0, 1, 2, 3];
/// How close to the cut the drawn circles stop, on either side.
const SIDE: f64 = 0.004;

/// The screen point of z (polar r, θ with −π < θ < π), part way `m` from
/// the cut plane (m = 0) to Log's strip (m = 1).
fn unroll(r: f64, th: f64, m: f64) -> Vec3 {
    let a = Z.p(C::polar(r, th));
    let b = v(Z.centre[0] + W_SCALE * r.ln(), Z.centre[1] + W_SCALE * th);
    v(a[0] + (b[0] - a[0]) * m, a[1] + (b[1] - a[1]) * m)
}

/// How far from the cut its two marked sides are drawn (07_3–07_4), so the
/// two stay apart on screen: just above (gold) and just below (purple).
const MARKED: f64 = 0.07;

/// The cut's stroke on its own, and thinner once its two sides are marked, so
/// gold and purple keep the line between them.
const CUT_W: f64 = 6.0;
const CUT_BETWEEN_SIDES_W: f64 = 2.5;
/// The ring round the removed zero.
const ZERO_RING: f64 = 0.13;
const ZERO_RING_W: f64 = 3.5;
/// How far into the unrolling the cut and the zero ring have gone. They mark
/// points of the plane, and the strip has no such points: its w = 0 is the
/// image of z = 1, and z → 0 runs off to the left.
const PLANE_ONLY: f64 = 0.08;

/// How strongly the plane's own marks show, part way `m` to the strip.
fn plane_marks(m: f64) -> f64 {
    (1.0 - m / PLANE_ONLY).clamp(0.0, 1.0)
}

/// The strip's two edges on screen.
const STRIP_TOP: f64 = Z.centre[1] + W_SCALE * PI;
const STRIP_BOTTOM: f64 = Z.centre[1] - W_SCALE * PI;
/// Where the words beside the strip start, and the right-hand column.
const BESIDE_STRIP: f64 = Z.centre[0] + 1.65;
const COLUMN: f64 = 0.9;
/// Where 07_6's live angle starts, just past the words that name it.
const ANGLE_VALUE_X: f64 = COLUMN + 2.2;

/// 07_3's question and 07_4's answer, as screen text.
const ASK: &str = "where do the two sides go?";
const TO_TOP: &str = "just above → top edge";
const TO_BOTTOM: &str = "just below → bottom edge";
const NEAR: &str = "angle just above the cut: near π; just below: near −π";
const CIRCLES: &str = "circles |z| = r  →  vertical segments at ln r";
const RAYS_TO_LINES: &str = "rays at angle θ  →  horizontal lines at height θ";
const OFF_LEFT: &str = "z → 0: off to the left";
const ANY_CURVE: &str = "any curve from 0 out to infinity that never crosses itself";
const ANGLE_WORDS: &str = "principal angle, in π:";
const TOP_KEPT: &str = "top edge: included";
const BOTTOM_LEFT_OUT: &str = "bottom edge: left out";

/// The cut plane part way `m` to Log's strip, at opacity `alpha`; `sides` from
/// 0 to 1 brings in the cut's two marked sides (07_3 keeps them hidden until
/// it asks where they go). The cut and the zero ring are drawn here so that
/// they can only ever stand on the plane.
fn grid(s: &mut Arena, m: f64, sides: f64, alpha: f64) -> Mob {
    let g = s.add(VMobject::new());
    for &r in &RADII {
        let pts: Vec<Vec3> = (0..=72)
            .map(|k| unroll(r, -PI + SIDE + (TAU - 2.0 * SIDE) * f64::from(k) / 72.0, m))
            .collect();
        let c = s.add(polyline(pts).style(faded_line(position_c(), 2.5, alpha)).build());
        let _ = s.attach(g, c);
    }
    for &k in &RAYS {
        let th = f64::from(k) * PI / 4.0;
        let pts: Vec<Vec3> = (0..=40)
            .map(|j| unroll(R_MIN + (R_MAX - R_MIN) * f64::from(j) / 40.0, th, m))
            .collect();
        let c = s.add(polyline(pts).style(faded_line(number_c(), 2.5, alpha)).build());
        let _ = s.attach(g, c);
    }
    if sides > 0.0 {
        for (th, c) in [(PI - MARKED, answer_c()), (-PI + MARKED, lap_c())] {
            let pts: Vec<Vec3> = (0..=40)
                .map(|j| unroll(R_MIN + (R_MAX - R_MIN) * f64::from(j) / 40.0, th, m))
                .collect();
            let l = s.add(polyline(pts).style(faded_line(c, 5.0, alpha * sides)).build());
            let _ = s.attach(g, l);
        }
    }
    let marks = alpha * plane_marks(m);
    if marks > 0.0 {
        let width = CUT_W + (CUT_BETWEEN_SIDES_W - CUT_W) * sides;
        let cut = s.add(
            polyline(vec![Z.p(C::new(0.0, 0.0)), Z.p(C::new(-Z.half_w, 0.0))])
                .style(faded_line(warn_c(), width, marks))
                .build(),
        );
        let zero = s.add(
            polyline(arc_points(Z.centre, ZERO_RING, 0.0, TAU, 40))
                .style(faded_line(warn_c(), ZERO_RING_W, marks))
                .build(),
        );
        let _ = s.attach(g, cut);
        let _ = s.attach(g, zero);
    }
    g
}

/// "blocked", clear of the unit circle it stops at.
fn blocked_word(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text(stage, "blocked", 20.0, warn_c(), Z.p(C::new(-1.3, 0.75)))
}

/// The words for the cut's two sides, outside the outer circle so that no
/// circle runs through them, and above and below the marked lines.
fn side_words(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    Ok([
        kit.text(stage, "just above", 20.0, answer_c(), Z.p(C::new(-2.95, 0.45)))?,
        kit.text(stage, "just below", 20.0, lap_c(), Z.p(C::new(-2.95, -0.45)))?,
    ])
}

/// 07_4's answer, beside the strip.
fn edge_labels(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 2]> {
    Ok([
        kit.text_left(stage, TO_TOP, 20.0, answer_c(), v(BESIDE_STRIP, STRIP_TOP - 0.4))?,
        kit.text_left(stage, TO_BOTTOM, 20.0, lap_c(), v(BESIDE_STRIP, STRIP_BOTTOM + 0.4))?,
    ])
}

/// What the strip says about zero in place of the ring: its image has run off
/// to the left of the strip.
fn off_left_note(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    kit.text_left(stage, OFF_LEFT, 18.0, Palette::dim(), v(-6.7, Z.centre[1]))
}

/// The dashed path a short walk across the cut would need, on the strip: from
/// the gold line near the top edge to the purple line near the bottom edge.
fn leap_ends() -> (Vec3, Vec3) {
    (unroll(1.2, PI - MARKED, 1.0), unroll(1.2, -PI + MARKED, 1.0))
}

/// 07_6's last sentence as a picture: the strip again, its top edge solid gold
/// because the convention adds it, its bottom edge dashed because it stays out.
fn closing_strip(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Mob; 5]> {
    let (from, to) = (Z.centre[0] - 1.5, Z.centre[0] + 1.5);
    let plane = grid(stage.arena_mut(), 1.0, 0.0, 1.0);
    let top = seg(stage, v(from, STRIP_TOP), v(to, STRIP_TOP), answer_c(), 4.0);
    let bottom = dashed(
        stage,
        v(from, STRIP_BOTTOM),
        v(to, STRIP_BOTTOM),
        Palette::dim(),
        2.0,
        12,
    )?;
    let kept = kit.text_left(stage, TOP_KEPT, 20.0, answer_c(), v(BESIDE_STRIP, STRIP_TOP))?;
    let left_out = kit.text_left(
        stage,
        BOTTOM_LEFT_OUT,
        20.0,
        Palette::dim(),
        v(BESIDE_STRIP, STRIP_BOTTOM),
    )?;
    Ok([plane, top, bottom, kept, left_out])
}

/// Log's imaginary part in units of π, ±½, as screen text: "πi/2", "−πi/2".
fn half_pi_i(x: f64) -> fmn::Result<String> {
    if (x.abs() - 0.5).abs() > 1e-12 {
        return Err(super::draw::fail(format!("branch.log_i_pi: {x} is not ±½")));
    }
    Ok(if x < 0.0 {
        "\u{2212}πi/2".into()
    } else {
        "πi/2".into()
    })
}

/// The spiral cut's growth per radian, and where its drawing starts. The
/// drawing stops at radius `Z.half_h`, inside the frame; only the card says
/// that the cut goes on to infinity.
const SPIRAL_K: f64 = 0.18;
const SPIRAL_START: f64 = -5.0 * PI;

/// A logarithmic spiral from (near) zero out to the panel's edge: a cut
/// whose radius only grows, so it never crosses itself, and which winds
/// ever more tightly toward zero.
fn spiral_cut() -> Vec<Vec3> {
    let end = (Z.half_h / 0.5).ln() / SPIRAL_K;
    (0..=300)
        .map(|k| {
            let th = SPIRAL_START + (end - SPIRAL_START) * f64::from(k) / 300.0;
            Z.p(C::polar(0.5 * (SPIRAL_K * th).exp(), th))
        })
        .collect()
}

impl SceneConstruct for BranchCut {
    fn name(&self) -> &str {
        "07_branch_cut"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());
        let right = COLUMN;

        // 07_1: forbid one line from zero to infinity
        kit.say(stage, "07_1")?;
        let heading = title(&kit, stage, "Give something up: a cut")?;
        let axes = Z.axes(stage)?;
        let circle = Z.unit_circle(stage);
        fade_in_all(stage, &[heading, axes, circle], 0.6)?;
        kit.until_sentence(stage, "07_1", 1)?;
        let cut = seg(stage, Z.p(C::new(0.0, 0.0)), Z.p(C::new(-Z.half_w, 0.0)), red, CUT_W);
        let cut_word = kit.text(stage, "no crossing", 22.0, red, Z.p(C::new(-1.9, 0.35)))?;
        fade_in_all(stage, &[cut, cut_word], 0.6)?;
        kit.until_sentence(stage, "07_1", 2)?;
        // a walker on the unit circle runs into the cut and must turn back
        let walk = stage.add_value_tracker(0.0);
        let walker = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, walk);
            let g = s.add(VMobject::new());
            if t.abs() > 1e-3 {
                let n = ((t.abs() * 160.0).ceil() as usize).max(2);
                let pts: Vec<Vec3> = (0..=n)
                    .map(|k| Z.p(C::polar(1.0, TAU * t * k as f64 / n as f64)))
                    .collect();
                let tr = s.add(polyline(pts).style(line_style(green, 3.0)).build());
                let _ = s.attach(g, tr);
            }
            let d = s.add(
                Dot::new()
                    .point(Z.p(C::polar(1.0, TAU * t)))
                    .radius(0.1)
                    .style(flat(green, 1.0))
                    .build(),
            );
            let _ = s.attach(g, d);
            g
        });
        stage.add_to_scene(walker)?;
        play!(stage; tween(walk, 0.47, 2.2)?);
        let stop = blocked_word(&kit, stage)?;
        fade_in_all(stage, &[stop], 0.3)?;
        play!(stage; tween(walk, -0.47, 3.0)?);
        let possible = kit.text_left(
            stage,
            "no loop can go around 0 now",
            22.0,
            Palette::ink(),
            v(right, 1.9),
        )?;
        fade_in_all(stage, &[possible], 0.4)?;
        kit.until_sentence(stage, "07_1", 3)?;
        let name = kit.text(stage, "branch cut", 26.0, red, Z.p(C::new(-1.9, -0.4)))?;
        swap(stage, &[cut_word, stop], &[name], 0.4)?;

        // 07_2: the usual cut; the principal logarithm Log
        kit.say(stage, "07_2")?;
        fade_out_all(stage, &[walker], 0.3)?;
        let removed = kit.tex(
            stage,
            r"\text{remove } (-\infty, 0]:\quad -\pi < \operatorname{Arg} z < \pi",
            30.0,
            Palette::ink(),
            &[],
            v(3.9, 1.25),
        )?;
        let zero = ring(stage, Z.centre, ZERO_RING, red, ZERO_RING_W);
        fade_in_all(stage, &[removed, zero], 0.5)?;
        kit.until_sentence(stage, "07_2", 1)?;
        let log = kit.tex(
            stage,
            r"\operatorname{Log} z = \ln|z| + i \operatorname{Arg} z",
            34.0,
            gold,
            &[],
            v(3.9, 0.4),
        )?;
        fade_in_all(stage, &[log], 0.5)?;
        kit.until_sentence(stage, "07_2", 2)?;
        let deriv = kit.tex(
            stage,
            r"\frac{d}{dz} \operatorname{Log} z = \frac{1}{z}",
            32.0,
            blue,
            &[],
            v(3.9, -0.6),
        )?;
        let where_ = kit.text(stage, "on the cut plane", 20.0, Palette::dim(), v(3.9, -1.25))?;
        fade_in_all(stage, &[deriv, where_], 0.5)?;
        kit.until_sentence(stage, "07_2", 3)?;
        // worked: i is a quarter turn up, −i a quarter turn down
        let logs = fact_list("branch.log_i_pi")?;
        let o = Z.centre;
        let worked = [
            super::draw::path(stage, arc_points(o, 0.35, 0.0, PI * logs[0], 16), gold, 3.0),
            super::draw::path(stage, arc_points(o, 0.35, 0.0, PI * logs[1], 16), purple, 3.0),
            dot(stage, Z.p(C::new(0.0, 1.0)), 0.09, gold),
            dot(stage, Z.p(C::new(0.0, -1.0)), 0.09, purple),
            kit.text_left(
                stage,
                &format!("Log i = {}", half_pi_i(logs[0])?),
                22.0,
                gold,
                Z.p(C::new(0.2, 1.25)),
            )?,
            kit.text_left(
                stage,
                &format!("Log(−i) = {}", half_pi_i(logs[1])?),
                22.0,
                purple,
                Z.p(C::new(0.2, -1.25)),
            )?,
        ];
        fade_in_all(stage, &worked, 0.5)?;

        // 07_3: the cut plane unrolls into the strip −π < Im < π; then a
        // prediction: where do the cut's two sides go?
        kit.say(stage, "07_3")?;
        fade_out_all(stage, &[possible, circle, axes], 0.4)?;
        fade_out_all(stage, &worked, 0.3)?;
        let morph = stage.add_value_tracker(0.0);
        let sides = stage.add_value_tracker(0.0);
        let grid_alpha = stage.add_value_tracker(0.0);
        let unrolled =
            stage.always_redraw(move |s: &mut Arena| grid(s, value(s, morph), value(s, sides), value(s, grid_alpha)));
        stage.add_to_scene(unrolled)?;
        fade_out_all(stage, &[name], 0.3)?;
        // the grid fades in with its own cut and zero ring over the old ones,
        // which then leave unseen; from here the grid decides when they show
        play!(stage; tween(grid_alpha, 1.0, 0.4)?);
        fade_out_all(stage, &[cut, zero], 0.1)?;
        kit.until_sentence(stage, "07_3", 1)?;
        play!(stage; tween(morph, 1.0, 3.6)?);
        let top = dashed(
            stage,
            v(Z.centre[0] - 1.5, STRIP_TOP),
            v(Z.centre[0] + 1.5, STRIP_TOP),
            Palette::dim(),
            2.0,
            12,
        )?;
        let bottom = dashed(
            stage,
            v(Z.centre[0] - 1.5, STRIP_BOTTOM),
            v(Z.centre[0] + 1.5, STRIP_BOTTOM),
            Palette::dim(),
            2.0,
            12,
        )?;
        let top_word = kit.text_left(stage, "height π", 20.0, Palette::dim(), v(BESIDE_STRIP, STRIP_TOP))?;
        let bottom_word = kit.text_left(stage, "height −π", 20.0, Palette::dim(), v(BESIDE_STRIP, STRIP_BOTTOM))?;
        let off_left = off_left_note(&kit, stage)?;
        fade_in_all(stage, &[top, bottom, top_word, bottom_word, off_left], 0.5)?;
        kit.until_sentence(stage, "07_3", 2)?;
        let circles = kit.text_left(stage, CIRCLES, 22.0, blue, v(right, -1.95))?;
        fade_in_all(stage, &[circles], 0.4)?;
        kit.until_sentence(stage, "07_3", 3)?;
        let rays = kit.text_left(stage, RAYS_TO_LINES, 22.0, green, v(right, -2.45))?;
        fade_in_all(stage, &[rays], 0.4)?;
        kit.until_sentence(stage, "07_3", 4)?;
        // back to the cut plane, and mark the cut's two sides (the strip's
        // height marks leave meanwhile: fade_out restores them for 07_4)
        fade_out_all(stage, &[top, bottom, top_word, bottom_word, off_left], 0.3)?;
        play!(stage; tween(morph, 0.0, 2.0)?);
        play!(stage; tween(sides, 1.0, 0.4)?);
        let [above_word, below_word] = side_words(&kit, stage)?;
        fade_in_all(stage, &[above_word, below_word], 0.4)?;
        kit.until_sentence(stage, "07_3", 5)?;
        let ask = kit.text_left(stage, ASK, 24.0, gold, v(right, 2.3))?;
        fade_in_all(stage, &[ask], 0.4)?;
        predict(&kit, stage, v(right + 1.7, 1.95), 3.0)?;

        // 07_4: the two sides go to the strip's two edges; crossing jumps 2π
        kit.say(stage, "07_4")?;
        fade_out_all(stage, &[above_word, below_word], 0.3)?;
        play!(stage; tween(morph, 1.0, 3.0)?);
        let [to_top, to_bottom] = edge_labels(&kit, stage)?;
        swap(
            stage,
            &[ask],
            &[top, bottom, top_word, bottom_word, off_left, to_top, to_bottom],
            0.4,
        )?;
        kit.until_sentence(stage, "07_4", 1)?;
        let near = kit.text_left(stage, NEAR, 22.0, Palette::ink(), v(right, -2.95))?;
        fade_in_all(stage, &[near], 0.4)?;
        kit.until_sentence(stage, "07_4", 2)?;
        // a short path across the cut: on the strip it would leap 2π
        let (above, below) = leap_ends();
        let x = above[0];
        let leap = dashed(stage, above, below, red, 3.0, 14)?;
        let ends = [dot(stage, above, 0.09, red), dot(stage, below, 0.09, red)];
        let leap_word = kit.text(
            stage,
            "crossing the cut: a jump of 2π",
            22.0,
            red,
            v(x + 2.6, Z.centre[1] + 0.35),
        )?;
        fade_in_all(stage, &ends, 0.3)?;
        fade_in_all(stage, &[leap, leap_word], 0.5)?;

        // 07_5: other cuts; the branch point
        kit.say(stage, "07_5")?;
        fade_out_all(
            stage,
            &[
                leap,
                ends[0],
                ends[1],
                leap_word,
                top,
                bottom,
                top_word,
                bottom_word,
                off_left,
                circles,
                rays,
                to_top,
                to_bottom,
                near,
                removed,
                log,
                deriv,
                where_,
            ],
            0.4,
        )?;
        play!(stage; tween(morph, 0.0, 2.4)?);
        let axes2 = Z.axes(stage)?;
        let usual = seg(stage, Z.p(C::new(0.0, 0.0)), Z.p(C::new(-Z.half_w, 0.0)), red, 5.0);
        swap(stage, &[unrolled], &[axes2, usual, zero], 0.4)?;
        let choice = kit.text_left(stage, "a choice, not a crack", 24.0, Palette::ink(), v(right, -1.95))?;
        fade_in_all(stage, &[choice], 0.4)?;
        kit.until_sentence(stage, "07_5", 1)?;
        let upward = seg(stage, Z.p(C::new(0.0, 0.0)), Z.p(C::new(0.0, Z.half_h)), red, 5.0);
        let window = kit.tex(
            stage,
            r"\text{cut upward:}\quad -\tfrac{3\pi}{2} < \text{angle} < \tfrac{\pi}{2}",
            28.0,
            red,
            &[],
            v(3.9, -2.55),
        )?;
        swap(stage, &[usual], &[upward], 0.6)?;
        fade_in_all(stage, &[window], 0.5)?;
        kit.until_sentence(stage, "07_5", 2)?;
        let spiral = super::draw::path(stage, spiral_cut(), red, 4.0);
        swap(stage, &[upward, window], &[spiral], 0.8)?;
        kit.until_sentence(stage, "07_5", 3)?;
        let any = kit.text_left(stage, ANY_CURVE, 22.0, Palette::ink(), v(right, -2.55))?;
        fade_in_all(stage, &[any], 0.4)?;
        kit.until_sentence(stage, "07_5", 4)?;
        let point = ring(stage, Z.centre, 0.2, gold, 4.0);
        fade_out_all(stage, &[zero], 0.2)?;
        fade_in_all(stage, &[point], 0.4)?;
        kit.until_sentence(stage, "07_5", 5)?;
        let branch = kit.text_left(stage, "branch point", 26.0, gold, v(right, -3.05))?;
        fade_in_all(stage, &[branch], 0.4)?;

        // 07_6: two cautions — the convention on the cut, the one-sided inverse
        kit.say(stage, "07_6")?;
        fade_out_all(stage, &[spiral, choice, any, branch], 0.5)?;
        let heading2 = title(&kit, stage, "Two cautions")?;
        swap(stage, &[heading], &[heading2], 0.4)?;
        let usual2 = seg(stage, Z.p(C::new(0.0, 0.0)), Z.p(C::new(-Z.half_w, 0.0)), red, 5.0);
        let circle2 = Z.unit_circle(stage);
        fade_in_all(stage, &[usual2, circle2], 0.4)?;
        kit.until_sentence(stage, "07_6", 1)?;
        let on_cut = dot(stage, Z.p(C::new(-1.0, 0.0)), 0.1, gold);
        let conv = kit.tex(
            stage,
            r"\operatorname{Arg}(-1) = \pi \ \ (\text{convention})",
            28.0,
            gold,
            &[],
            v(3.9, 1.6),
        )?;
        fade_in_all(stage, &[on_cut, conv], 0.5)?;
        kit.until_sentence(stage, "07_6", 2)?;
        // a point slides down across the cut; its principal angle jumps
        let slide = stage.add_value_tracker(0.85);
        let slider = stage.always_redraw(move |s: &mut Arena| {
            let a = PI * value(s, slide);
            s.add(
                Dot::new()
                    .point(Z.p(C::polar(1.0, a)))
                    .radius(0.09)
                    .style(flat(purple, 1.0))
                    .build(),
            )
        });
        stage.add_to_scene(slider)?;
        let angle_word = kit.text_left(stage, ANGLE_WORDS, 22.0, purple, v(right, 0.85))?;
        let angle = kit.live_value(
            stage,
            move |s| crate::model::wrap(PI * value(s, slide)) / PI,
            3,
            true,
            26.0,
            purple,
            v(ANGLE_VALUE_X, 0.85),
            LEFT,
        )?;
        fade_in_all(stage, &[angle_word], 0.3)?;
        play!(stage; tween(slide, 1.15, 3.0)?);
        kit.until_sentence(stage, "07_6", 3)?;
        // the first caution is done: its moving dot and number leave with it
        fade_out_all(stage, &[slider, angle_word, angle], 0.3)?;
        let one_side = kit.text_left(
            stage,
            "Log undoes exp from one side only",
            22.0,
            Palette::ink(),
            v(right, 0.15),
        )?;
        fade_in_all(stage, &[one_side], 0.4)?;
        kit.until_sentence(stage, "07_6", 4)?;
        let always = kit.tex(
            stage,
            r"e^{\operatorname{Log} z} = z \quad (z \neq 0)",
            30.0,
            green,
            &[],
            v(3.9, -0.6),
        )?;
        fade_in_all(stage, &[always], 0.5)?;
        kit.until_sentence(stage, "07_6", 5)?;
        let only = kit.tex(
            stage,
            r"\operatorname{Log}(e^{w}) = w \iff -\pi < \operatorname{Im} w \le \pi",
            30.0,
            blue,
            &[],
            v(3.9, -1.45),
        )?;
        let example = kit.tex(
            stage,
            r"\operatorname{Log}(e^{2\pi i}) = 0 \neq 2\pi i",
            26.0,
            red,
            &[],
            v(3.9, -2.2),
        )?;
        fade_in_all(stage, &[only], 0.5)?;
        fade_in_all(stage, &[example], 0.4)?;
        // the fine print comes with the statement it qualifies, so it has time
        // to be read before the chapter ends
        let fine = precisely(
            &kit,
            stage,
            "Log is holomorphic off (−∞, 0] with −π < Im < π; with Arg(−x) = π for x > 0, Im Log fills −π < Im ≤ π (DLMF 4.2)",
        )?;
        fade_in_all(stage, &[fine], 0.5)?;
        kit.until_sentence(stage, "07_6", 6)?;
        // "That is the strip, with its top edge added by the convention."
        let strip = closing_strip(&kit, stage)?;
        swap(stage, &[axes2, usual2, circle2, on_cut, point], &strip, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

    /// The safe area's half width and half height.
    const SAFE: [f64; 2] = [6.96, 3.85];

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// A mobject's typeset box: left, right, bottom, top.
    fn box_of(stage: &Stage<'_>, m: Mob) -> [f64; 4] {
        [
            stage.get_bounding_box_point(m, LEFT)[0],
            stage.get_bounding_box_point(m, RIGHT)[0],
            stage.get_bounding_box_point(m, DOWN)[1],
            stage.get_bounding_box_point(m, UP)[1],
        ]
    }

    #[test]
    fn unrolling_sends_circles_to_verticals_and_rays_to_horizontals() {
        for r in RADII {
            let a = unroll(r, -2.0, 1.0);
            let b = unroll(r, 2.5, 1.0);
            assert!((a[0] - b[0]).abs() < 1e-12);
        }
        for k in RAYS {
            let th = f64::from(k) * PI / 4.0;
            let a = unroll(R_MIN, th, 1.0);
            let b = unroll(R_MAX, th, 1.0);
            assert!((a[1] - b[1]).abs() < 1e-12);
        }
        // the cut's two sides end at the strip's two edges, 2π apart
        let up = unroll(1.0, PI - SIDE, 1.0);
        let down = unroll(1.0, -PI + SIDE, 1.0);
        assert!(((up[1] - down[1]) / W_SCALE - TAU).abs() < 0.01);
        // and before unrolling they lie together on the negative axis
        let (u0, d0) = (unroll(1.0, PI - SIDE, 0.0), unroll(1.0, -PI + SIDE, 0.0));
        assert!((u0[1] - d0[1]).abs() < 0.01);
    }

    #[test]
    fn the_principal_angle_jumps_across_the_cut_and_takes_pi_on_it() {
        let above = crate::model::wrap(PI - 1e-6);
        let below = crate::model::wrap(PI + 1e-6);
        assert!((above - PI).abs() < 1e-5 && (below + PI).abs() < 1e-5);
        assert_eq!(C::new(-1.0, 0.0).arg(), PI);
    }

    #[test]
    fn log_undoes_exp_exactly_on_the_half_open_strip() {
        for im in [-3.0, 0.0, 2.0, PI] {
            let w = C::new(0.4, im);
            let back = w.exp().log();
            assert!((back.re - w.re).abs() < 1e-12 && (back.im - w.im).abs() < 1e-9, "{im}");
        }
        let w = C::new(0.0, TAU);
        assert!(w.exp().log().im.abs() < 1e-9);
        // just past −π, Log lands near the top edge instead
        let w = C::new(0.0, -PI - 0.01);
        assert!((w.exp().log().im - (PI - 0.01)).abs() < 1e-9);
    }

    #[test]
    fn the_spiral_cut_never_crosses_itself_and_leaves_the_panel() {
        let pts = spiral_cut();
        let r = |p: &Vec3| ((p[0] - Z.centre[0]).powi(2) + (p[1] - Z.centre[1]).powi(2)).sqrt();
        for w in pts.windows(2) {
            assert!(r(&w[1]) > r(&w[0]));
        }
        assert!(r(&pts[0]) < 0.05 && (r(&pts[pts.len() - 1]) - Z.half_h).abs() < 1e-9);
        assert!(pts.iter().all(|p| p[0] > -7.1 && p[1] < 3.0 && p[1] > -3.2));
    }

    #[test]
    fn the_marked_sides_start_apart_on_the_cut_and_end_on_the_strips_two_edges() {
        // on the cut plane: either side of the negative axis, visibly apart
        let (u0, d0) = (unroll(1.0, PI - MARKED, 0.0), unroll(1.0, -PI + MARKED, 0.0));
        assert!(u0[1] > Z.centre[1] && d0[1] < Z.centre[1] && u0[1] - d0[1] > 0.1);
        // unrolled: just inside the top edge (height π) and the bottom edge (−π)
        let top = Z.centre[1] + W_SCALE * PI;
        let bottom = Z.centre[1] - W_SCALE * PI;
        for r in [R_MIN, 1.0, R_MAX] {
            let (u, d) = (unroll(r, PI - MARKED, 1.0), unroll(r, -PI + MARKED, 1.0));
            assert!(u[1] < top && top - u[1] < 0.1, "{r}");
            assert!(d[1] > bottom && d[1] - bottom < 0.1, "{r}");
        }
        // the answer agrees with Log: angle near π above the cut, near −π below
        let above = C::polar(1.0, PI - MARKED).log().im;
        let below = C::polar(1.0, -PI + MARKED).log().im;
        assert!((above - (PI - MARKED)).abs() < 1e-12);
        assert!((below - (-PI + MARKED)).abs() < 1e-12);
    }

    #[test]
    fn log_of_i_and_minus_i_are_a_quarter_turn_either_way() {
        let logs = fact_list("branch.log_i_pi").unwrap();
        assert_eq!(logs, vec![0.5, -0.5]);
        assert_eq!(half_pi_i(logs[0]).unwrap(), "πi/2");
        assert_eq!(half_pi_i(logs[1]).unwrap(), "\u{2212}πi/2");
        assert!(half_pi_i(0.25).is_err());
        // both are on the principal strip, and exponentiate back to ±i
        for (x, im) in [(logs[0], 1.0), (logs[1], -1.0)] {
            let back = C::new(0.0, PI * x).exp();
            assert!(back.re.abs() < 1e-12 && (back.im - im).abs() < 1e-12);
        }
    }

    #[test]
    fn the_labels_beside_the_strip_end_before_the_cards_in_their_rows() {
        run(|stage| {
            let kit = Kit::new()?;
            // the strip itself ends left of the labels
            assert!(unroll(R_MAX, 0.0, 1.0)[0] < BESIDE_STRIP);
            let [to_top, to_bottom] = edge_labels(&kit, stage)?;
            let rays = kit.text_left(stage, RAYS_TO_LINES, 22.0, Palette::ink(), v(COLUMN, -2.45))?;
            let circles = kit.text_left(stage, CIRCLES, 22.0, Palette::ink(), v(COLUMN, -1.95))?;
            let rays_left = box_of(stage, rays)[0];
            assert!(
                box_of(stage, to_bottom)[1] < rays_left - 0.15,
                "the bottom label reaches the rays card"
            );
            assert!(
                box_of(stage, to_top)[1] < COLUMN - 0.15,
                "the top label reaches the column"
            );
            // every card in that column stays on screen
            let ask = kit.text_left(stage, ASK, 24.0, Palette::ink(), v(COLUMN, 2.3))?;
            let near = kit.text_left(stage, NEAR, 22.0, Palette::ink(), v(COLUMN, -2.95))?;
            for card in [rays, circles, ask, near] {
                assert!(box_of(stage, card)[1] < SAFE[0], "a card leaves the safe area");
            }
            Ok(())
        });
    }

    #[test]
    fn the_words_for_the_two_sides_clear_every_circle_and_the_marked_lines() {
        run(|stage| {
            let kit = Kit::new()?;
            let outer = RADII[RADII.len() - 1];
            // how far the marked lines stray from the axis at their outer end
            let line = R_MAX * MARKED.sin();
            for word in side_words(&kit, stage)? {
                let [left, right, bottom, top] = box_of(stage, word);
                let near = (bottom - Z.centre[1]).abs().min((top - Z.centre[1]).abs());
                let outer_left = Z.centre[0] - (outer * outer - near * near).sqrt();
                assert!(right < outer_left - 0.05, "the outer circle crosses a side word");
                assert!(left > -SAFE[0], "a side word leaves the safe area");
                assert!(near > line + 0.05, "a side word sits on its marked line");
            }
            Ok(())
        });
    }

    #[test]
    fn the_blocked_word_clears_the_unit_circle() {
        run(|stage| {
            let kit = Kit::new()?;
            let word = blocked_word(&kit, stage)?;
            let [_, right, bottom, _] = box_of(stage, word);
            // the box's corner nearest the origin stays outside the circle
            let nearest = (right - Z.centre[0]).hypot(bottom - Z.centre[1]);
            assert!(nearest > Z.scale + 0.05, "corner at radius {nearest}");
            Ok(())
        });
    }

    #[test]
    fn the_removed_zero_and_the_cut_stand_on_the_plane_and_never_on_the_strip() {
        // where the ring would sit on the strip is w = 0, the image of z = 1
        let w0 = unroll(1.0, 0.0, 1.0);
        assert!((w0[0] - Z.centre[0]).abs() < 1e-12 && (w0[1] - Z.centre[1]).abs() < 1e-12);
        // while points near zero run off the left of the frame
        assert!(unroll(1e-4, 0.0, 1.0)[0] < -7.11);
        assert_eq!(plane_marks(0.0), 1.0);
        assert_eq!(plane_marks(PLANE_ONLY), 0.0);
        assert_eq!(plane_marks(1.0), 0.0);
        run(|stage| {
            let kit = Kit::new()?;
            let lines = RADII.len() + RAYS.len();
            let on_plane = grid(stage.arena_mut(), 0.0, 0.0, 1.0);
            let leaving = grid(stage.arena_mut(), PLANE_ONLY, 0.0, 1.0);
            let on_strip = grid(stage.arena_mut(), 1.0, 0.0, 1.0);
            // the parent, the lines, and on the plane the cut and the ring
            assert_eq!(stage.family(on_plane).len(), 1 + lines + 2);
            assert_eq!(stage.family(leaving).len(), 1 + lines);
            assert_eq!(stage.family(on_strip).len(), 1 + lines);
            // the marked sides do not change that
            let sides_on_strip = grid(stage.arena_mut(), 1.0, 1.0, 1.0);
            assert_eq!(stage.family(sides_on_strip).len(), 1 + lines + 2);
            let sides_on_plane = grid(stage.arena_mut(), 0.0, 1.0, 1.0);
            assert_eq!(stage.family(sides_on_plane).len(), 1 + lines + 2 + 2);
            // only the cut reaches the panel's left edge, and it goes with the plane
            assert!((box_of(stage, on_plane)[0] - (Z.centre[0] - Z.half_w)).abs() < 1e-6);
            assert!(box_of(stage, leaving)[0] > Z.centre[0] - Z.half_w + 0.1);
            // the note in the ring's place clears the strip and the frame
            let note = off_left_note(&kit, stage)?;
            let [left, right, _, _] = box_of(stage, note);
            assert!(left > -SAFE[0], "the zero note leaves the safe area");
            assert!(
                right < unroll(R_MIN, 0.0, 1.0)[0] - 0.1,
                "the zero note reaches the strip"
            );
            Ok(())
        });
    }

    #[test]
    fn the_leap_runs_between_the_two_marked_lines() {
        let (above, below) = leap_ends();
        // both ends stand at one x, on the marked lines just inside the edges
        assert!((above[0] - below[0]).abs() < 1e-12);
        assert!((above[1] - (STRIP_TOP - W_SCALE * MARKED)).abs() < 1e-9);
        assert!((below[1] - (STRIP_BOTTOM + W_SCALE * MARKED)).abs() < 1e-9);
        // so the drawn leap is the words' 2π, less the drawn offset each side
        assert!(((above[1] - below[1]) / W_SCALE - (TAU - 2.0 * MARKED)).abs() < 1e-12);
    }

    #[test]
    fn the_branch_point_word_clears_the_spiral_and_the_card_above_it() {
        run(|stage| {
            let kit = Kit::new()?;
            let word = kit.text_left(stage, "branch point", 26.0, Palette::ink(), v(COLUMN, -3.05))?;
            let any = kit.text_left(stage, ANY_CURVE, 22.0, Palette::ink(), v(COLUMN, -2.55))?;
            let [left, right, bottom, top] = box_of(stage, word);
            assert!(top < box_of(stage, any)[2], "the word reaches the card above it");
            assert!(bottom > -SAFE[1] && right < SAFE[0], "the word leaves the safe area");
            for p in spiral_cut() {
                let clear = p[0] < left - 0.05 || p[0] > right + 0.05 || p[1] < bottom - 0.05 || p[1] > top + 0.05;
                assert!(clear, "the spiral crosses the word at {p:?}");
            }
            Ok(())
        });
    }

    #[test]
    fn the_closing_strip_shows_the_two_edges_two_pi_apart() {
        run(|stage| {
            let kit = Kit::new()?;
            let [plane, top, bottom, kept, left_out] = closing_strip(&kit, stage)?;
            let (t, b) = (stage.get_center(top)[1], stage.get_center(bottom)[1]);
            assert!(((t - b) / W_SCALE - TAU).abs() < 1e-6, "edges {t} and {b}");
            // no ring and no cut on this strip, and the lines stay between its edges
            assert_eq!(stage.family(plane).len(), 1 + RADII.len() + RAYS.len());
            let [_, _, grid_bottom, grid_top] = box_of(stage, plane);
            assert!(grid_top < t && grid_bottom > b);
            // each edge's word sits on its row and ends before the column
            for (word, edge) in [(kept, t), (left_out, b)] {
                let [left, right, _, _] = box_of(stage, word);
                assert!(left > unroll(R_MAX, 0.0, 1.0)[0], "a word reaches the strip");
                assert!(right < COLUMN - 0.15, "a word reaches the column");
                assert!((stage.get_center(word)[1] - edge).abs() < 1e-6);
            }
            // the top edge is the one Log reaches: Arg(−1) = π
            assert!((C::new(-1.0, 0.0).log().im - PI).abs() < 1e-12);
            let mathematical_top = unroll(1.0, PI, 1.0)[1];
            assert!((mathematical_top - STRIP_TOP).abs() < 1e-12);
            // Native mobject records store positions as f32; get_center
            // promotes those records back to f64 (fmn-mobject/src/bbox.rs).
            // Check exact storage rounding separately from the f64 model.
            assert_eq!(t, f64::from(mathematical_top as f32));
            Ok(())
        });
    }

    #[test]
    fn the_live_angle_starts_just_past_the_words_that_name_it() {
        run(|stage| {
            let kit = Kit::new()?;
            let words = kit.text_left(stage, ANGLE_WORDS, 22.0, Palette::ink(), v(COLUMN, 0.85))?;
            let right = box_of(stage, words)[1];
            assert!(
                right + 0.1 < ANGLE_VALUE_X && ANGLE_VALUE_X - right < 0.5,
                "gap from {right}"
            );
            Ok(())
        });
    }
}
