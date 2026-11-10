//! 10_rigidity: the identity theorem and Liouville's theorem (storyboard
//! 10_1–10_6). The mechanism is drawn, not just named. Where f and g agree
//! on a little curve, every derivative of f − g vanishes, so f − g is 0 on
//! the largest disk around the point that fits in the region. A new centre
//! inside that disk starts the next disk, and the agreement walks across the
//! connected region disk by disk. Then sin² + cos² = 1 spreads off the real
//! line to 3i. Next comes the contrast of a smooth real function that is 0
//! on a half line yet is not 0, and sin(1/z), whose zeros pile up at a point
//! outside its region. The chapter ends on the bounded-entire-means-constant
//! limit, with sine growing off the real line.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, axis_names, card, carded, dot, fact, fade_in_all, fade_out_all, faded_line, flat, ground, group,
    hero, line_style, names_now, output, precisely_now, predict, reading_window, say, wait_until, warn,
};
use super::one_point::{BLOB_CENTRE, blob};
use crate::model::{C, Func, ZERO};

pub const THEOREM: [&str; 3] = ["f = g on a little curve", "⇒ f = g on the whole", "connected region"];
pub const WHY: [&str; 2] = [
    "on the curve, every derivative of f − g is 0,",
    "so f − g = 0 on a disk; step on, disk by disk",
];
pub const PILING: &str = "points piling up at a point inside U: enough";
pub const IDENTITY_TEX: &str = r"\sin^2 x + \cos^2 x = 1";
pub const REAL_LINE: &str = "they agree on the real line";
pub const SPREADS: &str = "so on the whole plane";
pub const AT_3I: [&str; 2] = [r"\sin^2(3i) \approx ", r"\cos^2(3i) \approx "];
pub const SIN_3I: [&str; 2] = ["sin(3i) ≈ ", " i"];
pub const SUM: &str = "sum = ";
pub const SMOOTH_TEX: &str = r"e^{-1/x}\ (x > 0), \qquad 0\ (x \le 0)";
pub const DERIVS: &str = "every derivative at 0 is 0";
pub const RECIPE_AT: &str = "so its recipe at 0 is";
pub const ZERO_RECIPE_TEX: &str = r"0 + 0\,x + 0\,x^2 + \cdots = 0";
pub const ZEROS: [&str; 2] = ["zeros at 1/π, 1/(2π), 1/(3π), …", "= "];
pub const ASK: [&str; 1] = ["forced to be 0?"];
pub const NOT_IN: &str = "0 is not in the region: sin(1/z) is undefined there";
pub const CARD: [&str; 3] = [
    "identity theorem: agree on points",
    "piling up at a point inside the region",
    "⇒ agree on the whole connected region",
];
pub const EXACT: &str = "identity theorem: if f and g are holomorphic on a connected open set U and agree on a set with a limit point in U, then f = g on U";
pub const LIOUVILLE: [&str; 2] = ["holomorphic on the whole plane", "and bounded ⇒ constant"];
pub const REAL_BOUND: &str = "on the real line: between −1 and 1";
pub const AGREE_HERE: &str = "agree here";
pub const ZERO_HERE: &str = "zero here";
pub const NOT_ZERO_HERE: &str = "not zero here";
pub const SIN_RECIP_TEX: &str = r"\sin(1/z)";
pub const SIN_Z_TEX: &str = r"\sin z";
pub const FIRST_ZERO: &str = "1/π";
pub const SIZE: &str = "|sin z| = ";
/// How high the dot of 10_6 climbs the imaginary axis: to 3i, the reading of 10_2.
pub const CLIMB: f64 = 3.0;

/// The little curve where f and g are given to agree (scene points inside U).
pub fn curve() -> Vec<Vec3> {
    (0..=24)
        .map(|k| {
            let t = f64::from(k) / 24.0;
            v(
                BLOB_CENTRE[0] - 1.25 + 0.5 * t,
                BLOB_CENTRE[1] + 0.35 + 0.18 * (PI * t).sin(),
            )
        })
        .collect()
}

/// Distance from `at` to the edge of U.
pub fn edge_distance(at: Vec3) -> f64 {
    blob(720)
        .iter()
        .map(|q| ((q[0] - at[0]).powi(2) + (q[1] - at[1]).powi(2)).sqrt())
        .fold(f64::INFINITY, f64::min)
}

/// Where the chain of disks heads, in turn (offsets from U's centre).
pub const ROUTE: [[f64; 2]; 5] = [[-0.2, 0.95], [0.85, 0.35], [0.7, -0.75], [-0.3, -1.2], [-1.1, -0.55]];

/// The chain of disks that carries the agreement: each disk is 0.85 of the
/// way to U's edge, and each next centre lies inside the disk before it.
pub fn chain() -> Vec<(Vec3, f64)> {
    let start = curve()[12];
    let mut out = vec![(start, 0.85 * edge_distance(start))];
    for target in ROUTE {
        let goal = v(BLOB_CENTRE[0] + target[0], BLOB_CENTRE[1] + target[1]);
        for _ in 0..12 {
            let (c, r) = *out.last().unwrap_or(&(start, 0.1));
            let (dx, dy) = (goal[0] - c[0], goal[1] - c[1]);
            let d = (dx * dx + dy * dy).sqrt();
            if d < 0.05 {
                break;
            }
            let step = (0.7 * r).min(d);
            let next = v(c[0] + dx / d * step, c[1] + dy / d * step);
            out.push((next, 0.85 * edge_distance(next)));
        }
    }
    out
}

/// The point inside U where the points of 10_1 pile up.
pub fn limit_point() -> Vec3 {
    v(BLOB_CENTRE[0] + 0.6, BLOB_CENTRE[1] - 0.2)
}

/// The points that pile up at [`limit_point`] q: q + t(1, 0.35), with t = 0.9 · 0.62^k.
pub fn piling_points() -> Vec<Vec3> {
    let q = limit_point();
    (0..9)
        .map(|k| {
            let t = 0.9 * 0.62f64.powi(k);
            v(q[0] + t, q[1] + 0.35 * t)
        })
        .collect()
}

/// Radius of each piling dot.
pub const PILE_DOT: f64 = 0.06;
/// Radius of the gold dot at q.
pub const Q_DOT: f64 = 0.08;
/// Radius of q's dark backing: 0.03 wider than the dot, as for the zeros of 10_4. Without it
/// the gold q sits on U's gold fill at 1.48:1.
pub const Q_BACKING: f64 = Q_DOT + 0.03;

/// A disk outline and its translucent fill.
fn disk(stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Mob> {
    let pts: Vec<Vec3> = (0..72)
        .map(|j| {
            let th = TAU * f64::from(j) / 72.0;
            v(at[0] + r * th.cos(), at[1] + r * th.sin())
        })
        .collect();
    let fill = stage
        .arena_mut()
        .add(Polygon::new(pts.clone()).style(flat(hero(), 0.16)).build());
    let mut closed = pts;
    closed.push(closed[0]);
    let rim = stage
        .arena_mut()
        .add(polyline(closed).style(line_style(hero(), 2.5)).build());
    group(stage, &[fill, rim])
}

/// The plane of 10_2, wide enough for 3i.
pub fn trig_plane() -> Plane {
    Plane::new(v(-3.0, -0.3), 5.4, (-3.6, 3.6), (-3.6, 3.6))
}

/// The graph of 10_3.
pub fn smooth_graph() -> Frame2 {
    Frame2 {
        center: v(-1.4, -0.6),
        width: 8.0,
        height: 4.0,
        x: (-3.0, 4.0),
        y: (-0.25, 1.0),
    }
}

/// e^{−1/x} for x > 0, 0 otherwise: smooth, with every derivative 0 at 0.
pub fn flat_bump(x: f64) -> f64 {
    if x > 0.0 { (-1.0 / x).exp() } else { 0.0 }
}

/// The window of sin(1/z) in 10_4–10_5.
pub fn sin_recip_plane() -> Plane {
    // 0.1 above centre, so that its foot and frame stop above where 10_5's two-line lower third
    // starts (centred, the foot ran 0.04 into that band); moved, not resized, so the zeros,
    // cards and cross keep their places on the plate
    Plane::new(v(-2.8, 0.1), 5.6, (-0.42, 0.42), (-0.42, 0.42))
}

/// The window of sine in 10_6.
pub fn sine_plane() -> Plane {
    Plane::new(v(-2.8, -0.3), 6.0, (-4.0, 4.0), (-3.5, 3.5))
}

pub struct Rigidity {
    pub kit: Kit,
}

impl SceneConstruct for Rigidity {
    fn name(&self) -> &str {
        "10_rigidity"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let right = 4.3;

        // 10_1: agreement on a little curve spreads disk by disk
        let line = say(&kit, stage, "10_1")?;
        let edge = blob(96);
        let fill = stage
            .arena_mut()
            .add(Polygon::new(edge.clone()).style(flat(output(), 0.10)).build());
        let mut closed = edge.clone();
        closed.push(edge[0]);
        let rim = stage
            .arena_mut()
            .add(polyline(closed).style(line_style(output(), 3.0)).build());
        let u_name = kit.tex(
            stage,
            "U",
            34.0,
            output(),
            &[],
            v(BLOB_CENTRE[0] - 1.55, BLOB_CENTRE[1] + 1.75),
        )?;
        fade_in_all(stage, &[fill, rim, u_name], 0.6)?;
        line.wait_sentence(stage, 2)?;
        let little = stage
            .arena_mut()
            .add(polyline(curve()).style(line_style(hero(), 7.0)).build());
        fade_in_all(stage, &[little], 0.4)?;
        let theorem = super::draw::carded_lines(&kit, stage, &THEOREM, 24.0, Palette::ink(), v(right, 2.3), hero())?;
        fade_in_all(stage, &theorem, 0.4)?;
        let why = [
            kit.text(stage, WHY[0], 20.0, Palette::dim(), v(right, 0.9))?,
            kit.text(stage, WHY[1], 20.0, Palette::dim(), v(right, 0.5))?,
        ];
        fade_in_all(stage, &why, 0.4)?;
        let mut disks = Vec::new();
        for (at, r) in chain() {
            let d = disk(stage, at, r)?;
            fade_in_all(stage, &[d], 0.18)?;
            disks.push(d);
        }
        // "they agree everywhere": all of U, margin and all
        let whole = stage
            .arena_mut()
            .add(Polygon::new(edge.clone()).style(flat(hero(), 0.18)).build());
        fade_in_all(stage, &[whole], 0.3)?;
        line.wait_sentence(stage, 3)?;
        let q = limit_point();
        let mut piling = Vec::new();
        for p in piling_points() {
            piling.push(dot(stage, p, PILE_DOT, Palette::ink()));
        }
        // q on a dark backing, as the zeros of 10_4 are, so the gold point reads on U's gold fill
        let q_marks = [dot(stage, q, Q_BACKING, ground()), dot(stage, q, Q_DOT, hero())];
        let pile_says = kit.text(stage, PILING, 20.0, Palette::ink(), v(right, -0.6))?;
        for m in &piling {
            fade_in_all(stage, &[*m], 0.12)?;
        }
        // both in the one 0.12 s step that q alone took, so the timeline is unchanged
        fade_in_all(stage, &q_marks, 0.12)?;
        piling.extend(q_marks);
        fade_in_all(stage, &[pile_says], 0.4)?;
        let search = names_now(&kit, stage, &["identity theorem"])?;
        wait_until(stage, line.end())?;

        // 10_2: sin² + cos² = 1 leaves the real line
        kit.hold(stage)?;
        let line = say(&kit, stage, "10_2")?;
        let mut gone = vec![fill, whole, rim, u_name, little, pile_says, search, why[0], why[1]];
        gone.extend(theorem);
        gone.extend(disks);
        gone.extend(piling);
        fade_out_all(stage, &gone, 0.5)?;
        let tp = trig_plane();
        let mut frame = vec![tp.border(stage, Palette::faint())?];
        frame.extend(tp.axes(stage)?);
        frame.extend(axis_names(&kit, stage, &tp)?);
        let identity = kit.tex(stage, IDENTITY_TEX, 34.0, Palette::ink(), &[], v(right, 2.6))?;
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &[identity], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let real = stage.arena_mut().add(
            polyline(vec![tp.p(C::real(-3.6)), tp.p(C::real(3.6))])
                .style(line_style(hero(), 8.0))
                .build(),
        );
        let real_says = kit.text(stage, REAL_LINE, 22.0, hero(), v(right, 1.8))?;
        let agree_here = carded(
            &kit,
            stage,
            AGREE_HERE,
            20.0,
            Palette::ink(),
            tp.p(C::new(2.1, -0.5)),
            hero(),
        )?;
        fade_in_all(stage, &[real, real_says, agree_here[0], agree_here[1]], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let spread = stage.add_value_tracker(0.0);
        let sheet = stage
            .arena_mut()
            .add(Polygon::new(sheet_points(&tp, 0.01)).style(flat(hero(), 0.16)).build());
        stage.add_to_scene(sheet)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let h = super::draw::value(a, spread).max(0.01);
                let _ = a.set_points(sheet, Polygon::new(sheet_points(&tp, h)).build().points());
            },
            true,
        )?;
        let spread_says = kit.text(stage, SPREADS, 22.0, hero(), v(right, 1.3))?;
        fade_in_all(stage, &[spread_says], 0.3)?;
        stage.play(super::draw::tween_linear(spread, 3.6, 2.0)?)?;
        stage.clear_updaters(anchor, true);
        line.wait_sentence(stage, 3)?;
        let at = dot(stage, tp.p(C::new(0.0, 3.0)), 0.09, Palette::ink());
        let at_name = kit.tex(stage, "3i", 26.0, Palette::ink(), &[], tp.p(C::new(0.55, 3.0)))?;
        fade_in_all(stage, &[at, at_name], 0.4)?;
        let vals: Vec<&str> = fact("rigid.trig")?.split(',').collect();
        let rows = [
            kit.tex(
                stage,
                &format!("{}{}", AT_3I[0], vals[0]),
                28.0,
                Palette::ink(),
                &[],
                v(right, 0.3),
            )?,
            kit.tex(
                stage,
                &format!("{}{}", AT_3I[1], vals[1]),
                28.0,
                Palette::ink(),
                &[],
                v(right, -0.45),
            )?,
            kit.text(
                stage,
                &format!("{SUM}{}", fact("rigid.trig.sum")?),
                28.0,
                hero(),
                v(right, -1.3),
            )?,
        ];
        for r in rows {
            fade_in_all(stage, &[r], 0.35)?;
        }
        wait_until(stage, line.end())?;

        // 10_3: a smooth real function zero on a half line, yet not zero
        kit.hold(stage)?;
        let line = say(&kit, stage, "10_3")?;
        let mut gone = vec![
            identity,
            real,
            real_says,
            agree_here[0],
            agree_here[1],
            sheet,
            spread_says,
            at,
            at_name,
        ];
        gone.extend(frame);
        gone.extend(rows);
        fade_out_all(stage, &gone, 0.5)?;
        let g = smooth_graph();
        let x_axis = stage.arena_mut().add(
            polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let y_axis = stage.arena_mut().add(
            polyline(vec![g.p(0.0, g.y.0), g.p(0.0, g.y.1)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let pts: Vec<Vec3> = (0..=700)
            .map(|k| {
                let x = g.x.0 + (g.x.1 - g.x.0) * f64::from(k) / 700.0;
                g.p(x, flat_bump(x))
            })
            .collect();
        let curve_m = stage
            .arena_mut()
            .add(polyline(pts).style(line_style(Palette::ink(), 5.0)).build());
        let smooth_tex = kit.tex(stage, SMOOTH_TEX, 30.0, Palette::ink(), &[], v(-1.4, 2.6))?;
        let axis_end = g.p(g.x.1, 0.0);
        let x_name = kit.text(stage, "x", 22.0, Palette::dim(), v(axis_end[0] + 0.18, axis_end[1]))?;
        fade_in_all(stage, &[x_axis, y_axis, x_name, smooth_tex], 0.4)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[curve_m], 0.8)?;
        line.wait_sentence(stage, 2)?;
        let half_line = stage.arena_mut().add(
            polyline(vec![g.p(g.x.0, 0.0), g.p(0.0, 0.0)])
                .style(faded_line(hero(), 10.0, 0.6))
                .build(),
        );
        // the half line and the part that is not on it, each named on its own words
        let zero_here = kit.text(stage, ZERO_HERE, 20.0, hero(), g.p(-1.5, 0.1))?;
        let not_zero_here = kit.text(stage, NOT_ZERO_HERE, 20.0, Palette::ink(), g.p(2.2, 0.82))?;
        fade_in_all(stage, &[half_line, zero_here], 0.4)?;
        line.wait_for(stage, "yet it is not zero")?;
        fade_in_all(stage, &[not_zero_here], 0.3)?;
        line.wait_sentence(stage, 3)?;
        let derivs = kit.text(stage, DERIVS, 22.0, Palette::ink(), v(5.1, 1.2))?;
        let recipe_at = kit.text(stage, RECIPE_AT, 22.0, Palette::ink(), v(5.1, 0.6))?;
        let recipe_tex = kit.tex(stage, ZERO_RECIPE_TEX, 28.0, hero(), &[], v(5.1, 0.0))?;
        let zero_recipe = stage.arena_mut().add(
            polyline(vec![g.p(0.0, 0.0), g.p(g.x.1, 0.0)])
                .style(line_style(hero(), 5.0))
                .build(),
        );
        let mut gaps = Vec::new();
        for k in 1..=7 {
            let x = 0.5 * f64::from(k);
            gaps.push(
                stage.arena_mut().add(
                    polyline(vec![g.p(x, 0.0), g.p(x, flat_bump(x))])
                        .style(line_style(warn(), 4.0))
                        .build(),
                ),
            );
        }
        fade_in_all(stage, &[derivs, recipe_at, recipe_tex, zero_recipe], 0.4)?;
        fade_in_all(stage, &gaps, 0.5)?;
        wait_until(stage, line.end())?;

        // 10_4: sin(1/z): zeros piling up at 0. Forced to be 0?
        kit.hold(stage)?;
        let line = say(&kit, stage, "10_4")?;
        let mut gone = vec![
            x_axis,
            y_axis,
            x_name,
            curve_m,
            smooth_tex,
            half_line,
            zero_here,
            not_zero_here,
            derivs,
            recipe_at,
            recipe_tex,
            zero_recipe,
        ];
        gone.extend(gaps);
        fade_out_all(stage, &gone, 0.5)?;
        let sp = sin_recip_plane();
        let painter = Painter::build(stage, Some(Func::SinRecip), sp, 1.0)?;
        let mut sp_frame = sp.axes(stage)?;
        sp_frame.extend(axis_names(&kit, stage, &sp)?);
        sp_frame.push(sp.border(stage, Palette::faint())?);
        sp_frame.push(kit.tex(
            stage,
            SIN_RECIP_TEX,
            30.0,
            Palette::ink(),
            &[],
            v(sp.frame.left() + 0.9, sp.frame.top() + 0.3),
        )?);
        painter.show(stage, 1.0)?;
        fade_in_all(stage, &sp_frame, 0.3)?;
        let mut zeros = Vec::new();
        for k in 1..=12 {
            for s in [1.0, -1.0] {
                let z = C::real(s / (PI * f64::from(k)));
                let r = if k <= 3 { 0.07 } else { 0.045 };
                zeros.push(dot(stage, sp.p(z), r + 0.03, ground()));
                zeros.push(dot(stage, sp.p(z), r, Palette::ink()));
            }
        }
        let zs: Vec<&str> = fact("rigid.sin_recip.zeros")?.split(',').collect();
        let listed = [
            kit.text(stage, ZEROS[0], 20.0, Palette::ink(), v(right, 2.6))?,
            kit.text(
                stage,
                &format!("{}{}, {}, {}, … → 0", ZEROS[1], zs[0], zs[1], zs[2]),
                20.0,
                Palette::ink(),
                v(right, 2.2),
            )?,
        ];
        // the first three zeros as the voice says each, a pair at a time, then all the rest
        // on "and so on"; the question then has the pile-up to look at
        let first_zero = carded(
            &kit,
            stage,
            FIRST_ZERO,
            18.0,
            Palette::ink(),
            sp.p(C::new(1.0 / PI, -0.07)),
            Palette::faint(),
        )?;
        line.wait_for(stage, "one over pi")?;
        let mut first = zeros[..4].to_vec();
        first.extend([first_zero[0], first_zero[1], listed[0]]);
        fade_in_all(stage, &first, 0.2)?;
        line.wait_for(stage, "one over two pi")?;
        fade_in_all(stage, &zeros[4..8], 0.2)?;
        line.wait_for(stage, "one over three pi")?;
        fade_in_all(stage, &zeros[8..12], 0.2)?;
        line.wait_for(stage, "and so on")?;
        fade_in_all(stage, &zeros[12..], 0.5)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[listed[1]], 0.4)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right, 0.9))?;

        // 10_5: no: the pile-up point 0 is not in the region
        let line = say(&kit, stage, "10_5")?;
        fade_out_all(stage, &asked, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let mut cross = red_cross(stage, sp.p(ZERO))?;
        cross.extend(carded(
            &kit,
            stage,
            "0",
            20.0,
            Palette::ink(),
            sp.p(C::new(0.0, 0.07)),
            warn(),
        )?);
        let not_in = kit.text(stage, NOT_IN, 18.0, warn(), v(right, 1.6))?;
        fade_in_all(stage, &cross, 0.3)?;
        fade_in_all(stage, &[not_in], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let theorem_card = condition_card(&kit, stage, v(right, -0.2))?;
        fade_in_all(stage, &theorem_card, 0.5)?;
        let exact = precisely_now(&kit, stage, EXACT)?;
        wait_until(stage, line.end())?;

        // 10_6: Liouville: bounded and entire means constant; sine must grow
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "10_6")?;
        let mut gone = vec![listed[0], listed[1], first_zero[0], first_zero[1], not_in, exact];
        gone.extend(sp_frame);
        gone.extend(zeros);
        gone.extend(cross);
        gone.extend(theorem_card);
        fade_out_all(stage, &gone, 0.4)?;
        painter.leave(stage, 0.5)?;
        let sn = sine_plane();
        let sine = Painter::build(stage, Some(Func::Sin), sn, 1.0)?;
        let mut sn_frame = sn.axes(stage)?;
        sn_frame.extend(axis_names(&kit, stage, &sn)?);
        sn_frame.push(sn.border(stage, Palette::faint())?);
        sn_frame.push(kit.tex(
            stage,
            SIN_Z_TEX,
            30.0,
            Palette::ink(),
            &[],
            v(sn.frame.left() + 0.7, sn.frame.top() + 0.3),
        )?);
        // the plate's pinwheels are its zeros: name the three in view
        for (x, s) in [(-PI, "−π"), (0.0, "0"), (PI, "π")] {
            sn_frame.extend(carded(
                &kit,
                stage,
                s,
                20.0,
                Palette::ink(),
                sn.p(C::new(x, -0.62)),
                Palette::faint(),
            )?);
        }
        sine.show(stage, 1.0)?;
        fade_in_all(stage, &sn_frame, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let liouville = [
            kit.text(stage, LIOUVILLE[0], 22.0, hero(), v(right, 2.6))?,
            kit.text(stage, LIOUVILLE[1], 22.0, hero(), v(right, 2.2))?,
        ];
        fade_in_all(stage, &liouville, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["Liouville's theorem"])?;
        line.wait_sentence(stage, 3)?;
        let real_line = stage.arena_mut().add(
            polyline(vec![sn.p(C::real(-4.0)), sn.p(C::real(4.0))])
                .style(line_style(Palette::ink(), 4.0))
                .build(),
        );
        let bound = kit.text(stage, REAL_BOUND, 22.0, Palette::ink(), v(right, 1.4))?;
        fade_in_all(stage, &[real_line, bound], 0.4)?;
        // "must grow huge": a dot climbs from 0 to 3i, its size read as it goes
        line.wait_for(stage, "must grow huge")?;
        let s3: Vec<&str> = fact("gallery.sin.at_3i")?.split(',').collect();
        let climb = stage.add_value_tracker(0.0);
        let at = dot(stage, sn.p(ZERO), 0.09, Palette::ink());
        let trail = sn.path(stage, &[ZERO, C::new(0.0, 1e-3)], hero(), 4.0)?;
        let size_says = kit.text_left(stage, SIZE, 24.0, hero(), v(2.7, 0.6))?;
        fade_in_all(stage, &[trail, at, size_says], 0.2)?;
        let size = kit.live_value(
            stage,
            move |s| C::new(0.0, super::draw::value(s, climb)).sin().abs(),
            2,
            false,
            24.0,
            hero(),
            v(3.95, 0.6),
            LEFT,
        )?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let y = super::draw::value(a, climb).max(1e-3);
                a.move_to(at, sn.p(C::new(0.0, y)), ORIGIN);
                let _ = a.set_points(trail, polyline(vec![sn.p(ZERO), sn.p(C::new(0.0, y))]).build().points());
            },
            true,
        )?;
        stage.play(super::draw::tween_linear(climb, CLIMB, 1.2)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        stage.clear_updaters(size, true);
        stage.remove_from_scene(size);
        stage.remove_from_scene(size_says);
        let at_name = carded(
            &kit,
            stage,
            "3i",
            22.0,
            Palette::ink(),
            sn.p(C::new(0.85, CLIMB)),
            Palette::faint(),
        )?;
        let big = kit.text(
            stage,
            &format!("{}{}{}", SIN_3I[0], s3[1], SIN_3I[1]),
            24.0,
            hero(),
            v(right, 0.6),
        )?;
        fade_in_all(stage, &[at_name[0], at_name[1], big], 0.3)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            liouville[0],
            liouville[1],
            search,
            real_line,
            bound,
            at,
            trail,
            at_name[0],
            at_name[1],
            big,
        ];
        all.extend(sn_frame);
        fade_out_all(stage, &all, 0.5)?;
        sine.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

/// The band |Im z| ≤ h of the plane `p` (clipped to its window), as a polygon.
pub fn sheet_points(p: &Plane, h: f64) -> Vec<Vec3> {
    let h = h.min(p.frame.y.1);
    vec![
        p.p(C::new(p.frame.x.0, -h)),
        p.p(C::new(p.frame.x.1, -h)),
        p.p(C::new(p.frame.x.1, h)),
        p.p(C::new(p.frame.x.0, h)),
    ]
}

/// The identity theorem's card, its condition in gold.
fn condition_card(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let pitch = 0.42;
    let texts = [
        kit.text(stage, CARD[0], 22.0, Palette::ink(), v(at[0], at[1] + pitch))?,
        kit.text(stage, CARD[1], 22.0, hero(), v(at[0], at[1]))?,
        kit.text(stage, CARD[2], 22.0, Palette::ink(), v(at[0], at[1] - pitch))?,
    ];
    let w = texts.iter().map(|&t| stage.get_width(t)).fold(0.0, f64::max);
    let mut out = vec![card(stage, w + 0.5, 3.0 * pitch + 0.35, at, hero())?];
    out.extend(texts);
    Ok(out)
}

/// How far a cross's arms reach: short of the innermost zero drawn, 1/(12π), on `sin_recip_plane`.
pub const CROSS_ARM: f64 = 0.12;

/// A red cross at `at`.
fn red_cross(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Vec<Mob>> {
    let d = CROSS_ARM;
    let a = polyline(vec![v(at[0] - d, at[1] - d), v(at[0] + d, at[1] + d)])
        .style(line_style(warn(), 5.0))
        .build();
    let b = polyline(vec![v(at[0] - d, at[1] + d), v(at[0] + d, at[1] - d)])
        .style(line_style(warn(), 5.0))
        .build();
    Ok(vec![stage.arena_mut().add(a), stage.arena_mut().add(b)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{self};

    fn inside_u(p: Vec3) -> bool {
        let (dx, dy) = (p[0] - BLOB_CENTRE[0], p[1] - BLOB_CENTRE[1]);
        dx.hypot(dy) < super::super::one_point::blob_radius(dy.atan2(dx))
    }

    #[test]
    fn the_chain_of_disks_is_the_identity_theorems_argument() {
        let disks = chain();
        assert!(disks.len() >= 8, "a visible chain");
        for (k, &(c, r)) in disks.iter().enumerate() {
            // every disk lies in U
            assert!(inside_u(c) && r > 0.1 && r < edge_distance(c));
            // every next centre is inside the disk before it, where f − g is already 0
            if k > 0 {
                let (p, rp) = disks[k - 1];
                assert!(((c[0] - p[0]).powi(2) + (c[1] - p[1]).powi(2)).sqrt() < rp);
            }
        }
        // the first disk is centred on the little curve, which lies in U
        assert!(curve().iter().all(|&p| inside_u(p)));
        // and the chain reaches every part of the route
        for t in ROUTE {
            let goal = v(BLOB_CENTRE[0] + t[0], BLOB_CENTRE[1] + t[1]);
            assert!(
                disks
                    .iter()
                    .any(|&(c, r)| ((c[0] - goal[0]).powi(2) + (c[1] - goal[1]).powi(2)).sqrt() < r)
            );
        }
    }

    #[test]
    fn the_points_pile_up_inside_u_at_a_marked_point_inside_u() {
        let q = limit_point();
        let pts = piling_points();
        // the condition the picture shows: the points and the point they pile up at are in U
        assert!(inside_u(q) && edge_distance(q) > Q_BACKING);
        let d: Vec<f64> = pts.iter().map(|p| (p[0] - q[0]).hypot(p[1] - q[1])).collect();
        for (k, &p) in pts.iter().enumerate() {
            assert!(inside_u(p) && edge_distance(p) > PILE_DOT);
            // each point is closer to q by the same ratio: they pile up at q and nowhere else
            if k > 0 {
                assert!((d[k] / d[k - 1] - 0.62).abs() < 1e-12);
            }
        }
        assert!(d[8] < Q_DOT, "the last points reach under q");
        // q's backing covers only points already at q: the first four stay whole, and the
        // first six still show at least 0.03 past it
        for (k, &dk) in d.iter().enumerate() {
            if k < 4 {
                assert!(dk - PILE_DOT > Q_BACKING);
            }
            if k < 6 {
                assert!(dk + PILE_DOT - Q_BACKING > 0.03);
            }
        }
    }

    #[test]
    fn the_sin_recip_plate_stops_above_the_two_line_lower_third() {
        use fmn::core::constants::FRAME_Y_RADIUS;
        // explainer_kit::props::lower_third: lines wrap at 92 characters; the band is 0.3
        // plus 0.36 a line high, its foot 0.2 above the frame's, its stroke 1.5 wide. A plot
        // frame's border strokes 2.0. A stroke w is w / 100 scene units wide.
        let lines = explainer_kit::animatic::wrap(EXACT, 92, 3);
        assert_eq!(lines.len(), 2, "10_5's precisely card is two lines");
        let band_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * lines.len() as f64 + 0.015 / 2.0;
        let sp = sin_recip_plane();
        let foot = sp.frame.bottom() - 0.02 / 2.0;
        // 0.05 is 13.5 px at 4K; the centred plate ran 0.0375 into the band
        assert!(foot - band_top > 0.05, "plate foot {foot} against band top {band_top}");
        // its title sits 0.3 above the top; the title stays well inside the frame
        assert!(sp.frame.top() + 0.3 + 0.3 < FRAME_Y_RADIUS);
    }

    #[test]
    fn the_trig_values_drawn_are_the_facts_and_the_identity_holds_off_the_line() {
        let z = C::new(0.0, 3.0);
        let (s, c) = (z.sin(), z.cos());
        let vals: Vec<f64> = crate::facts::facts()
            .unwrap()
            .get("rigid.trig")
            .unwrap()
            .split(',')
            .map(|x| x.parse().unwrap())
            .collect();
        assert!(((s * s).re - vals[0]).abs() < 0.005 && ((c * c).re - vals[1]).abs() < 0.005);
        assert!((s * s + c * c - C::real(1.0)).abs() < 1e-9);
        assert!(trig_plane().contains(z));
    }

    #[test]
    fn the_flat_function_is_zero_on_the_half_line_and_not_elsewhere() {
        assert!((-300..=0).all(|k| flat_bump(f64::from(k) / 100.0) == 0.0));
        assert!(flat_bump(0.5) > 0.1 && flat_bump(4.0) < smooth_graph().y.1);
        // its slope at 0 from the right is 0 to every order we can see
        for h in [0.1, 0.05, 0.02] {
            assert!(flat_bump(h) / h.powi(3) < 0.05);
        }
    }

    #[test]
    fn the_climb_ends_on_the_reading_and_the_cross_leaves_the_zeros_showing() {
        let facts = crate::facts::facts().unwrap();
        let displayed = facts.get("gallery.sin.at_3i").unwrap().split(',').nth(1).unwrap();
        let endpoint = C::new(0.0, CLIMB).sin();
        // Check the exact endpoint separately from the one-decimal caption.
        assert_eq!(CLIMB, 3.0);
        assert!(endpoint.re.abs() < 1e-12);
        assert!((endpoint.im - (3.0_f64.exp() - (-3.0_f64).exp()) / 2.0).abs() < 1e-12);
        assert_eq!(format!("{:.1}", endpoint.im), displayed);
        assert!(sine_plane().contains(C::new(0.85, CLIMB)));
        // the innermost zero drawn sits outside the cross, in scene units along the real axis
        let sp = sin_recip_plane();
        let inner = sp.p(C::real(1.0 / (12.0 * PI)))[0] - sp.p(ZERO)[0];
        assert!(inner > CROSS_ARM);
    }

    #[test]
    fn the_zeros_drawn_are_zeros_and_pile_up_at_zero() {
        let sp = sin_recip_plane();
        for k in 1..=12 {
            let z = C::real(1.0 / (PI * f64::from(k)));
            assert!(sp.contains(z));
            assert!(Func::SinRecip.eval(z).unwrap().abs() < 1e-9);
        }
        assert!(Func::SinRecip.eval(ZERO).is_none());
        assert_eq!(
            crate::facts::facts().unwrap().get("rigid.sin_recip.zeros"),
            Some("0.318,0.159,0.106")
        );
        // sine is 10i at 3i (the readout) and inside its window
        let s3 = model::C::new(0.0, 3.0).sin();
        assert!((s3.im - 10.0).abs() < 0.05 && sine_plane().contains(C::new(0.0, 3.0)));
    }
}
