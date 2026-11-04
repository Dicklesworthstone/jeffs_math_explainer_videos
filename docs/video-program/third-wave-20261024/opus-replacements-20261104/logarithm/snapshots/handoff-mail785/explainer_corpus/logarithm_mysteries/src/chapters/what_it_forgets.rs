//! Chapter 12, what the exponential forgets: four small panels recalled in
//! the order of the film. On the positive numbers nothing is forgotten
//! (one input per output; ratios become equal steps) (12_1); the dial
//! forgets the decade, the complex exponential forgets the laps, a branch
//! cut decides them and the product rule counts them (12_2); flows forget
//! even more, and some places no single steady real flow reaches; one
//! question for every panel (12_3); the question to take away: a solid
//! given a half turn about a fixed axis, and how many steady spins reach it
//! in one unit of time (12_4).
//!
//! The take-away's answer is not shown (storyboard §12 keeps it for the
//! description): the spins about that same axis, either way, by an odd
//! number of half turns, π, 3π, 5π … radians per unit time; the two
//! slowest, a half turn either way, tie, so no principal choice exists.
//! The tests below check that answer.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Decade, answer_c, arc_points, dot, fact, fact_f64, fade_in_all, fade_in_lagged, fade_out_all, flat, lap_c,
    line_style, number_c, path, position_c, predict, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model::{self, M2};

pub struct WhatItForgets {
    pub kit: Kit,
}

/// The four panels' centres (A: reals, B: dial, C: complex, D: matrices)
/// and their common half extents.
const PANELS: [[f64; 2]; 4] = [[-3.55, 1.45], [3.55, 1.45], [-3.55, -1.65], [3.55, -1.65]];
const HALF_W: f64 = 3.2;
const HALF_H: f64 = 1.25;

/// Panel A: the exponential's graph (origin, x and y scale) and its window.
const GRAPH_O: [f64; 2] = [-5.4, 0.85];
const GRAPH_SX: f64 = 0.55;
const GRAPH_SY: f64 = 0.25;
const GRAPH_X: (f64, f64) = (-2.0, 1.95);
/// Panel B: the dial.
const DIAL_C: [f64; 2] = [2.3, 1.45];
const DIAL_R: f64 = 0.95;
/// Panel C: the plane's zero, its scale, the column of log(−1)'s x and
/// its height per π.
const PLANE_C: [f64; 2] = [-5.6, -1.55];
const PLANE_S: f64 = 0.55;
const COLUMN_X: f64 = -3.9;
const COLUMN_PER_PI: f64 = 0.28;
/// Panel D: the plane's zero and scene units per unit.
const FLOWS_C: [f64; 2] = [2.0, -1.55];
const FLOWS_U: f64 = 0.32;
/// The take-away: the cube's centre, its half edge in scene units and the
/// fixed axis.
const CUBE_C: [f64; 2] = [-2.4, -0.4];
const CUBE_HALF: f64 = 0.95;
const AXIS: [f64; 3] = [0.25, 1.0, 0.35];

type P3 = [f64; 3];

fn unit(a: P3) -> P3 {
    let n = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    [a[0] / n, a[1] / n, a[2] / n]
}

fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A turn by `angle` about the unit axis `n` (Rodrigues' formula).
fn turn(n: P3, angle: f64, p: P3) -> P3 {
    let (s, c) = angle.sin_cos();
    let k = cross(n, p);
    let along = dot3(n, p) * (1.0 - c);
    [
        p[0] * c + k[0] * s + n[0] * along,
        p[1] * c + k[1] * s + n[1] * along,
        p[2] * c + k[2] * s + n[2] * along,
    ]
}

/// A fixed oblique view: yaw about the vertical, then pitch toward us.
fn project(p: P3) -> Vec3 {
    let (yaw, pitch) = (35f64.to_radians(), 20f64.to_radians());
    let (sy, cy) = yaw.sin_cos();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let (sp, cp) = pitch.sin_cos();
    let y = p[1] * cp - z * sp;
    v(CUBE_C[0] + CUBE_HALF * x, CUBE_C[1] + CUBE_HALF * y)
}

const CORNERS: [P3; 8] = [
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
];
const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];
/// The top face (+y) and the front face (+z), coloured so a turn shows.
const TOP: [usize; 4] = [3, 2, 6, 7];
const FRONT: [usize; 4] = [4, 5, 6, 7];

/// The cube turned by `angle` about the fixed axis, as one redraw.
fn cube(s: &mut Arena, angle: f64, faint: bool) -> Mob {
    let n = unit(AXIS);
    let pts: Vec<Vec3> = CORNERS.iter().map(|&p| project(turn(n, angle, p))).collect();
    let g = s.add(VMobject::new());
    let (o_face, o_edge) = if faint { (0.12, 0.35) } else { (0.45, 1.0) };
    for (face, c) in [(TOP, slide_c()), (FRONT, position_c())] {
        let f = s.add(
            Polygon::new(face.iter().map(|&i| pts[i]))
                .style(flat(c, o_face))
                .build(),
        );
        let _ = s.attach(g, f);
    }
    for (a, b) in EDGES {
        let style = Style::default()
            .fill(Palette::ink(), 0.0)
            .stroke(Palette::ink(), 2.5, o_edge);
        let e = s.add(polyline(vec![pts[a], pts[b]]).style(style).build());
        let _ = s.attach(g, e);
    }
    g
}

/// The ellipses of chapter 10 from their fact ("a,b,tilt°;…").
fn ellipses(spec: &str) -> fmn::Result<Vec<(f64, f64, f64)>> {
    spec.split(';')
        .map(|e| {
            let n: Vec<f64> = e.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            match n.as_slice() {
                [a, b, t] => Ok((*a, *b, t.to_radians())),
                _ => Err(super::draw::fail(format!("matrix.ellipses: bad entry {e}"))),
            }
        })
        .collect()
}

/// Where the flow of the ellipse (a, b, tilt) carries its start P·(1, 0)
/// after time `t`: e^{t·2πPJP⁻¹} applied to it.
fn orbit_point(a: f64, b: f64, tilt: f64, t: f64) -> [f64; 2] {
    let p: M2 = model::mmul(model::rotation(tilt), [[a, 0.0], [0.0, b]]);
    let start = model::apply(p, [1.0, 0.0]);
    model::apply(model::flow(model::ellipse_generator(a, b, tilt), t), start)
}

fn flows_screen(q: [f64; 2]) -> Vec3 {
    v(FLOWS_C[0] + FLOWS_U * q[0], FLOWS_C[1] + FLOWS_U * q[1])
}

fn graph_screen(x: f64, y: f64) -> Vec3 {
    v(GRAPH_O[0] + GRAPH_SX * x, GRAPH_O[1] + GRAPH_SY * y)
}

/// The angle of a dial mark at `laps` of a turn, clockwise from the top.
fn dial_at(laps: f64, r: f64) -> Vec3 {
    let a = PI / 2.0 - TAU * laps;
    v(DIAL_C[0] + r * a.cos(), DIAL_C[1] + r * a.sin())
}

fn column_at(k: i32) -> Vec3 {
    // the logarithms of −1 have heights (2k + 1)π
    v(COLUMN_X, PLANE_C[1] + COLUMN_PER_PI * f64::from(2 * k + 1))
}

fn frame(stage: &mut Stage<'_>, c: [f64; 2]) -> Mob {
    let (w, h) = (HALF_W, HALF_H);
    let pts = vec![
        v(c[0] - w, c[1] - h),
        v(c[0] + w, c[1] - h),
        v(c[0] + w, c[1] + h),
        v(c[0] - w, c[1] + h),
        v(c[0] - w, c[1] - h),
    ];
    path(stage, pts, Palette::faint(), 2.0)
}

impl SceneConstruct for WhatItForgets {
    fn name(&self) -> &str {
        "12_what_it_forgets"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple, red) = (number_c(), position_c(), answer_c(), lap_c(), warn_c());

        // 12_1: look back; on the positive numbers nothing is forgotten
        kit.say(stage, "12_1")?;
        let heading = title(&kit, stage, "What the exponential forgets")?;
        let frames: Vec<Mob> = PANELS.iter().map(|&c| frame(stage, c)).collect();
        fade_in_all(stage, &[heading], 0.5)?;
        fade_in_lagged(stage, &frames, 1.2, 0.2)?;
        kit.until_sentence(stage, "12_1", 1)?;
        let x_axis = seg(
            stage,
            graph_screen(GRAPH_X.0, 0.0),
            graph_screen(GRAPH_X.1, 0.0),
            Palette::faint(),
            2.0,
        );
        let curve_pts: Vec<Vec3> = (0..=60)
            .map(|k| {
                let x = GRAPH_X.0 + (GRAPH_X.1 - GRAPH_X.0) * f64::from(k) / 60.0;
                graph_screen(x, x.exp())
            })
            .collect();
        let curve = path(stage, curve_pts, green, 3.5);
        let level = 3.0_f64;
        let probe = seg(
            stage,
            graph_screen(GRAPH_X.0, level),
            graph_screen(GRAPH_X.1, level),
            blue,
            2.0,
        );
        let hit = dot(stage, graph_screen(level.ln(), level), 0.07, gold);
        let cap_a = kit.text_left(
            stage,
            "one input for every output",
            20.0,
            Palette::ink(),
            v(-6.55, 0.45),
        )?;
        fade_in_all(stage, &[x_axis, curve], 0.5)?;
        fade_in_all(stage, &[probe, hit, cap_a], 0.5)?;
        kit.until_sentence(stage, "12_1", 2)?;
        let ruler = Decade {
            one: v(-3.6, 1.2),
            length: 3.0,
            side: 1.0,
        };
        let scale = ruler.build(&kit, stage, blue, 14.0)?;
        let mut hops = Vec::new();
        for k in 0..3 {
            let (a, b) = (ruler.x(2f64.powi(k)), ruler.x(2f64.powi(k + 1)));
            let (mid, r) = ((a + b) / 2.0, (b - a) / 2.0);
            let pts: Vec<Vec3> = arc_points(v(mid, 1.95), r, 0.0, PI, 16).into_iter().rev().collect();
            hops.push(path(stage, pts, green, 2.5));
        }
        let cap_a2 = kit.text_left(stage, "×2 is always the same step", 20.0, green, v(-3.6, 0.85))?;
        fade_in_all(stage, &[scale], 0.5)?;
        fade_in_lagged(stage, &hops, 1.0, 0.3)?;
        fade_in_all(stage, &[cap_a2], 0.4)?;

        // 12_2: the dial forgets the decade; the complex exponential the laps
        kit.say(stage, "12_2")?;
        let dial = ring(stage, v(DIAL_C[0], DIAL_C[1]), DIAL_R, Palette::dim(), 2.5);
        let mut dial_marks = Vec::new();
        for k in 1..=9 {
            let laps = f64::from(k).log10();
            dial_marks.push(seg(
                stage,
                dial_at(laps, DIAL_R),
                dial_at(laps, DIAL_R - 0.14),
                blue,
                2.0,
            ));
            dial_marks.push(kit.text(stage, &k.to_string(), 14.0, blue, dial_at(laps, DIAL_R + 0.2))?);
        }
        let mark = fact_f64("dial.mark")?;
        let pointer = seg(stage, v(DIAL_C[0], DIAL_C[1]), dial_at(mark, DIAL_R - 0.05), gold, 3.0);
        let values = fact("dial.values")?.replace(',', ", ");
        let same = kit.text_left(stage, &values, 20.0, gold, v(3.6, 1.85))?;
        let one_mark = kit.text_left(stage, "all land on one mark", 20.0, gold, v(3.6, 1.4))?;
        let cap_b = kit.text_left(stage, "the dial forgets the decade", 20.0, purple, v(3.6, 0.75))?;
        fade_in_all(stage, &[dial], 0.4)?;
        fade_in_all(stage, &dial_marks, 0.5)?;
        fade_in_all(stage, &[pointer, same, one_mark, cap_b], 0.5)?;
        kit.until_sentence(stage, "12_2", 1)?;
        let axes = {
            let (c, w) = (PLANE_C, 0.95);
            let x = seg(stage, v(c[0] - w, c[1]), v(c[0] + w, c[1]), Palette::faint(), 2.0);
            let y = seg(stage, v(c[0], c[1] - w), v(c[0], c[1] + w), Palette::faint(), 2.0);
            [x, y]
        };
        let circle = ring(stage, v(PLANE_C[0], PLANE_C[1]), PLANE_S, Palette::dim(), 2.0);
        let minus_one = v(PLANE_C[0] - PLANE_S, PLANE_C[1]);
        let target = dot(stage, minus_one, 0.07, green);
        let mut column = Vec::new();
        for k in -2..=1 {
            let p = column_at(k);
            column.push(seg(stage, p, minus_one, Palette::faint(), 1.5));
            column.push(dot(stage, p, 0.06, purple));
            column.push(kit.text_left(
                stage,
                &format!("{}i", super::draw::pi_label(i64::from(2 * k + 1))),
                16.0,
                purple,
                v(COLUMN_X + 0.2, p[1]),
            )?);
        }
        let cap_c = kit.text_left(
            stage,
            "the complex exponential forgets the laps",
            20.0,
            purple,
            v(-6.55, -2.65),
        )?;
        fade_in_all(stage, &[axes[0], axes[1], circle, target], 0.4)?;
        fade_in_lagged(stage, &column, 1.2, 0.08)?;
        fade_in_all(stage, &[cap_c], 0.4)?;
        kit.until_sentence(stage, "12_2", 2)?;
        let cut = seg(
            stage,
            v(PLANE_C[0], PLANE_C[1]),
            v(PLANE_C[0] - 0.95, PLANE_C[1]),
            red,
            4.0,
        );
        let cut_word = kit.text_left(stage, "a cut chooses the laps", 18.0, red, v(-2.85, -1.2))?;
        let rule = kit.text_left(stage, "Log zw = Log z + Log w + 2πik", 18.0, purple, v(-2.85, -1.7))?;
        fade_in_all(stage, &[cut, cut_word], 0.4)?;
        fade_in_all(stage, &[rule], 0.4)?;

        // 12_3: flows forget more; some places no single steady flow reaches
        kit.say(stage, "12_3")?;
        let specs = ellipses(fact("matrix.ellipses")?)?;
        let mut orbits = Vec::new();
        for &(a, b, tilt) in std::iter::once(&(1.0, 1.0, 0.0)).chain(specs.iter()) {
            let pts: Vec<Vec3> = (0..=48)
                .map(|k| flows_screen(orbit_point(a, b, tilt, f64::from(k) / 48.0)))
                .collect();
            orbits.push(path(stage, pts, Palette::dim(), 2.0));
        }
        let home = dot(stage, flows_screen([0.0, 0.0]), 0.04, Palette::dim());
        let clock = stage.add_value_tracker(0.0);
        let all: Vec<(f64, f64, f64)> = std::iter::once((1.0, 1.0, 0.0)).chain(specs.iter().copied()).collect();
        let movers = stage.always_redraw(move |s: &mut Arena| {
            let t = value(s, clock);
            let g = s.add(VMobject::new());
            for &(a, b, tilt) in &all {
                let d = s.add(
                    Dot::new()
                        .point(flows_screen(orbit_point(a, b, tilt, t)))
                        .radius(0.07)
                        .style(flat(green, 1.0))
                        .build(),
                );
                let _ = s.attach(g, d);
            }
            g
        });
        let cap_d = kit.text_left(
            stage,
            "many flows, one place: all home at t = 1",
            20.0,
            green,
            v(0.55, -2.65),
        )?;
        fade_in_all(stage, &orbits, 0.4)?;
        fade_in_all(stage, &[home], 0.2)?;
        stage.add_to_scene(movers)?;
        play!(stage; tween(clock, 1.0, 2.6)?);
        fade_in_all(stage, &[cap_d], 0.4)?;
        kit.until_sentence(stage, "12_3", 1)?;
        let unreached = kit.text_left(stage, "diag(−2, −½):", 20.0, red, v(3.4, -1.0))?;
        let no_flow = kit.text_left(stage, "no single steady", 20.0, red, v(3.4, -1.45))?;
        let no_flow2 = kit.text_left(stage, "real flow ends here", 20.0, red, v(3.4, -1.85))?;
        fade_in_all(stage, &[unreached, no_flow, no_flow2], 0.5)?;
        kit.until_sentence(stage, "12_3", 2)?;
        let marks: Vec<Mob> = PANELS
            .iter()
            .map(|c| kit.text(stage, "?", 30.0, gold, v(c[0] + HALF_W - 0.3, c[1] + HALF_H - 0.3)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &marks, 1.0, 0.2)?;
        kit.until_sentence(stage, "12_3", 3)?;
        let mut panels = frames.clone();
        panels.extend([
            x_axis, curve, probe, hit, cap_a, scale, cap_a2, dial, pointer, same, one_mark, cap_b,
        ]);
        panels.extend(hops);
        panels.extend(dial_marks);
        panels.extend([
            axes[0], axes[1], circle, target, cap_c, cut, cut_word, rule, home, movers, cap_d,
        ]);
        panels.extend(column);
        panels.extend(orbits);
        panels.extend([unreached, no_flow, no_flow2]);
        panels.extend(marks);
        fade_out_all(stage, &panels, 0.6)?;
        let question = kit.text(stage, "Which steady motion got us here?", 44.0, gold, v(0.0, 0.2))?;
        fade_in_all(stage, &[question], 0.6)?;

        // 12_4: the take-away: a half turn about a fixed axis in space
        kit.say(stage, "12_4")?;
        let heading2 = title(&kit, stage, "One to take with you")?;
        swap(stage, &[heading, question], &[heading2], 0.5)?;
        kit.until_sentence(stage, "12_4", 1)?;
        let n = unit(AXIS);
        let ends = [-2.1, 2.1].map(|k| project([n[0] * k, n[1] * k, n[2] * k]));
        let axis = stage
            .arena_mut()
            .add(polyline(vec![ends[0], ends[1]]).style(line_style(gold, 3.5)).build());
        let ghost = cube(stage.arena_mut(), 0.0, true);
        let spin = stage.add_value_tracker(0.0);
        let solid = stage.always_redraw(move |s: &mut Arena| cube(s, PI * value(s, spin), false));
        fade_in_all(stage, &[ghost, axis], 0.5)?;
        stage.add_to_scene(solid)?;
        play!(stage; tween(spin, 1.0, 3.0)?);
        let half = kit.text_left(stage, "a half turn about this axis", 22.0, gold, v(0.6, 2.2))?;
        fade_in_all(stage, &[half], 0.4)?;
        kit.until_sentence(stage, "12_4", 2)?;
        let ask = kit.text_left(
            stage,
            "How many steady spins end here",
            28.0,
            Palette::ink(),
            v(0.6, 1.3),
        )?;
        let ask2 = kit.text_left(stage, "after one unit of time?", 28.0, Palette::ink(), v(0.6, 0.8))?;
        fade_in_all(stage, &[ask, ask2], 0.5)?;
        kit.until_sentence(stage, "12_4", 3)?;
        let hints = [
            kit.text_left(stage, "the axis", 24.0, gold, v(0.9, -0.3))?,
            kit.text_left(stage, "the direction of spin", 24.0, blue, v(0.9, -0.85))?,
            kit.text_left(stage, "the laps", 24.0, purple, v(0.9, -1.4))?,
        ];
        fade_in_lagged(stage, &hints, 2.4, 0.9)?;
        predict(&kit, stage, v(2.6, -2.2), 3.4)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close3(a: P3, b: P3) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-12)
    }

    #[test]
    fn a_half_turn_is_reached_by_odd_half_turns_either_way_about_the_same_axis() {
        let n = unit(AXIS);
        let minus = [-n[0], -n[1], -n[2]];
        for &p in &CORNERS {
            let target = turn(n, PI, p);
            for k in -3..=3 {
                let odd = PI * f64::from(2 * k + 1);
                assert!(close3(turn(n, odd, p), target), "{k}");
                assert!(close3(turn(minus, odd, p), target), "{k}");
            }
            // an even number of half turns is a whole number of laps: home
            assert!(close3(turn(n, TAU, p), p));
        }
    }

    #[test]
    fn the_two_slowest_spins_tie_so_no_principal_choice_exists() {
        let n = unit(AXIS);
        let p = [0.3, -0.7, 1.0];
        assert!(close3(turn(n, PI, p), turn(n, -PI, p)));
        // and nothing slower than a half turn gets there
        for k in 1..=99 {
            let a = PI * f64::from(k) / 100.0;
            assert!(!close3(turn(n, a, p), turn(n, PI, p)), "{a}");
        }
    }

    #[test]
    fn a_spin_about_any_other_axis_never_lands_on_the_half_turn() {
        let n = unit(AXIS);
        for m in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], unit([0.25, 1.0, 0.3])] {
            let m = unit(m);
            for k in 1..400 {
                let a = 4.0 * PI * f64::from(k) / 400.0;
                let lands = CORNERS.iter().all(|&p| close3(turn(m, a, p), turn(n, PI, p)));
                assert!(!lands, "{m:?} {a}");
            }
        }
    }

    #[test]
    fn every_ellipse_flow_comes_home_at_one_and_stays_on_its_ellipse() {
        let specs = ellipses("2,1,0;1.6,0.7,30").unwrap();
        for (a, b, tilt) in std::iter::once((1.0, 1.0, 0.0)).chain(specs) {
            let p = model::mmul(model::rotation(tilt), [[a, 0.0], [0.0, b]]);
            let start = orbit_point(a, b, tilt, 0.0);
            let end = orbit_point(a, b, tilt, 1.0);
            assert!((start[0] - end[0]).abs() < 1e-9 && (start[1] - end[1]).abs() < 1e-9);
            let mid = orbit_point(a, b, tilt, 0.3);
            let on = model::apply(p, [(TAU * 0.3).cos(), (TAU * 0.3).sin()]);
            assert!((mid[0] - on[0]).abs() < 1e-9 && (mid[1] - on[1]).abs() < 1e-9);
        }
    }

    #[test]
    fn the_panels_hold_their_pictures() {
        let inside = |p: Vec3, c: [f64; 2]| (p[0] - c[0]).abs() < HALF_W && (p[1] - c[1]).abs() < HALF_H;
        assert!(inside(graph_screen(GRAPH_X.1, GRAPH_X.1.exp()), PANELS[0]));
        assert!(inside(graph_screen(GRAPH_X.0, 0.0), PANELS[0]));
        assert!(inside(dial_at(0.25, DIAL_R + 0.3), PANELS[1]) && inside(dial_at(0.75, DIAL_R + 0.3), PANELS[1]));
        for k in -2..=1 {
            assert!(inside(column_at(k), PANELS[2]));
        }
        for (a, b, tilt) in [(2.0, 1.0, 0.0), (1.6, 0.7, 30f64.to_radians())] {
            for j in 0..32 {
                assert!(inside(
                    flows_screen(orbit_point(a, b, tilt, f64::from(j) / 32.0)),
                    PANELS[3]
                ));
            }
        }
        // the cube and its axis stay left of the text column at x 0.6
        let n = unit(AXIS);
        for angle in [0.0, 0.5, 1.3, PI] {
            for &p in &CORNERS {
                let q = project(turn(n, angle, p));
                assert!(q[0] < 0.4 && q[1].abs() < 3.0, "{q:?}");
            }
        }
        assert!(project([n[0] * 2.1, n[1] * 2.1, n[2] * 2.1])[1] < 3.2);
        // the panels clear the title and the frame edge
        assert!(PANELS[0][1] + HALF_H < 3.0 && PANELS[2][1] - HALF_H > -3.4);
        assert!(PANELS[1][0] + HALF_W < 7.0);
    }

    #[test]
    fn the_dial_mark_is_the_fractional_lap_of_two_point_four() {
        assert!((2.4f64.log10() - 0.380).abs() < 5e-4);
        for x in [0.24f64, 2.4, 24.0, 240.0] {
            let lap = x.log10();
            assert!((lap - lap.floor() - 2.4f64.log10()).abs() < 1e-12);
        }
    }
}
