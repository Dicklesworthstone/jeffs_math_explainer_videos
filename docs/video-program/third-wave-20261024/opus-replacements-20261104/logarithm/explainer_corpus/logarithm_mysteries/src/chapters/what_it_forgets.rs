//! Chapter 12, what the exponential forgets: four small panels recalled in
//! the order of the film. On the positive numbers nothing is forgotten
//! (one input per output; ratios become equal steps) (12_1); the dial
//! forgets the decade, the complex exponential forgets the laps, a branch
//! cut decides them and the product rule counts them (12_2); flows forget
//! even more, and some places no single steady real flow reaches; one
//! question for every panel (12_3); the take-away, taught before it is
//! asked: a solid cube and a marked rod through the centres of its top and
//! bottom faces, each corner's circle about the rod, one gold corner
//! followed at half a lap per unit time to the half turn (12_4); the
//! puzzle, how many steady spins end there at time 1, with three hints and
//! a pause (12_5); the answer, each hint run on the cube: 1½ and 2½ laps
//! land the same, so does ½ lap the other way, and a spin about another
//! axis drags the top face's centre off the rod; so infinitely many spins,
//! all about the rod, the slowest two turning ±π, on the open edge of
//! 10_3's window: a tie, so no principal logarithm (12_6).
//!
//! Mail 803 repair 3: depth is unambiguous. Only the faces turned toward
//! the viewer are drawn, opaque, and the rod, the other axis, the circles,
//! the traces and the marks are drawn only where the solid does not hide
//! them (`hidden`, a ray-box test). Statement on screen: a half turn has
//! eigenvalue −1, so Higham's principal-logarithm condition (MIMS EPrint
//! 2005.21, section 4 Fact 1) fails; Log(−1) = iπ is the complex
//! convention of chapter 07.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened.

use std::f64::consts::{PI, SQRT_2, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Decade, answer_c, arc_points, dot, fact, fact_f64, fact_list, fade_in_all, fade_in_lagged, fade_out_all,
    faded_line, flat, lap_c, number_c, path, position_c, precisely, predict, ring, seg, slide_c, swap, title, value,
    warn_c,
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
/// The take-away: the cube's centre and half edge in scene units. The
/// marked axis, the rod, is vertical through the centres of the top and
/// bottom faces, drawn `ROD_REACH` half edges each way from the centre.
const CUBE_C: [f64; 2] = [-3.0, -0.55];
const CUBE_HALF: f64 = 1.05;
const ROD: P3 = [0.0, 1.0, 0.0];
const ROD_REACH: f64 = 2.0;
/// 12_6's other axis (normalised where used), tilted 45° from the rod
/// toward the viewer's side, so the top face's centre ends in view.
const OTHER_AXIS: P3 = [-1.0, 1.0, 0.0];
/// The corner followed round its circle, and the top face's centre.
const CORNER: P3 = [1.0, 1.0, 1.0];
const TOP_CENTRE: P3 = [0.0, 1.0, 0.0];
/// The fixed view: yaw about the vertical, then pitch toward us, degrees.
const YAW_DEG: f64 = 35.0;
const PITCH_DEG: f64 = 20.0;
/// 12_6's turning line: the centre's x and y, and scene units per π.
const TURNING_X: f64 = 3.9;
const TURNING_Y: f64 = -2.0;
const TURNING_PER_PI: f64 = 0.62;

fn turning_at(angle: f64) -> Vec3 {
    v(TURNING_X + TURNING_PER_PI * angle / PI, TURNING_Y)
}

type P3 = [f64; 3];
/// A pose of the cube: turned by an angle about a unit axis.
type Pose = (P3, f64);

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

/// The view's (sin, cos) of yaw and (sin, cos) of pitch.
fn view() -> (f64, f64, f64, f64) {
    let (sy, cy) = YAW_DEG.to_radians().sin_cos();
    let (sp, cp) = PITCH_DEG.to_radians().sin_cos();
    (sy, cy, sp, cp)
}

/// A fixed oblique view: yaw about the vertical, then pitch toward us.
fn project(p: P3) -> Vec3 {
    let (sy, cy, sp, cp) = view();
    let (x, z) = (p[0] * cy + p[2] * sy, -p[0] * sy + p[2] * cy);
    let y = p[1] * cp - z * sp;
    v(CUBE_C[0] + CUBE_HALF * x, CUBE_C[1] + CUBE_HALF * y)
}

/// The unit direction toward the viewer: at right angles to both screen
/// directions of `project` (screen x cross screen y).
fn toward_viewer() -> P3 {
    let (sy, cy, sp, cp) = view();
    [-sy * cp, sp, cy * cp]
}

/// Whether the point `p` (cube units, centre at 0) is hidden by the solid
/// cube in `pose`: the ray from `p` toward the viewer passes through it (a
/// slab test in the cube's own frame). Points inside the cube are hidden;
/// points on a face turned toward the viewer are not.
fn hidden(p: P3, pose: Pose) -> bool {
    let (axis, angle) = pose;
    let q = turn(axis, -angle, p);
    let d = turn(axis, -angle, toward_viewer());
    let (mut lo, mut hi) = (0.0_f64, f64::INFINITY);
    for (qi, di) in q.into_iter().zip(d) {
        if di.abs() < 1e-12 {
            if qi.abs() > 1.0 {
                return false;
            }
        } else {
            let (a, b) = ((-1.0 - qi) / di, (1.0 - qi) / di);
            lo = lo.max(a.min(b));
            hi = hi.min(a.max(b));
        }
    }
    hi > lo + 1e-9 && hi > 1e-6
}

/// The stretches of the space curve through `pts` that the cube in `pose`
/// leaves in view, as screen polylines (each piece tested at its middle).
fn unhidden_runs(pts: &[P3], pose: Pose) -> Vec<Vec<Vec3>> {
    let mut runs: Vec<Vec<Vec3>> = vec![Vec::new()];
    for pair in pts.windows(2) {
        let mid = [0usize, 1, 2].map(|i| (pair[0][i] + pair[1][i]) / 2.0);
        if hidden(mid, pose) {
            if runs.last().is_some_and(|r| !r.is_empty()) {
                runs.push(Vec::new());
            }
        } else if let Some(run) = runs.last_mut() {
            if run.is_empty() {
                run.push(project(pair[0]));
            }
            run.push(project(pair[1]));
        }
    }
    runs.retain(|r| r.len() >= 2);
    runs
}

/// Strokes for `runs`, attached to the group `g`.
fn add_runs(s: &mut Arena, g: Mob, runs: Vec<Vec<Vec3>>, c: Srgb, w: f64, o: f64) {
    for run in runs {
        let m = s.add(polyline(run).style(faded_line(c, w, o)).build());
        let _ = s.attach(g, m);
    }
}

/// A dot, or with `ring` an unfilled circle, at the space point `p`,
/// attached to `g`, unless the cube in `pose` hides it.
fn add_mark(s: &mut Arena, g: Mob, p: P3, pose: Pose, r: f64, c: Srgb, ring: bool) {
    if hidden(p, pose) {
        return;
    }
    let m = if ring {
        s.add(
            Circle::new()
                .radius(r)
                .arc_center(project(p))
                .style(faded_line(c, 2.5, 1.0))
                .build(),
        )
    } else {
        s.add(Dot::new().point(project(p)).radius(r).style(flat(c, 1.0)).build())
    };
    let _ = s.attach(g, m);
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
/// The six faces: outward normal and corners in order round the face
/// (indices into `CORNERS`): top, bottom, +z, −z, +x, −x.
const FACES: [(P3, [usize; 4]); 6] = [
    ([0.0, 1.0, 0.0], [3, 2, 6, 7]),
    ([0.0, -1.0, 0.0], [0, 1, 5, 4]),
    ([0.0, 0.0, 1.0], [4, 5, 6, 7]),
    ([0.0, 0.0, -1.0], [0, 1, 2, 3]),
    ([1.0, 0.0, 0.0], [1, 2, 6, 5]),
    ([-1.0, 0.0, 0.0], [0, 3, 7, 4]),
];

/// Each face's colour. The faces seen at the start (+z blue, −x green)
/// differ from those seen after a half turn about the rod (−z teal, +x
/// purple), so the turn shows; the top stays grey under the gold rod.
fn face_colour(k: usize) -> Srgb {
    match k {
        0 => Palette::dim(),
        1 => Palette::faint(),
        2 => position_c(),
        3 => slide_c(),
        4 => lap_c(),
        _ => number_c(),
    }
}

/// The solid cube in `pose`: only the faces turned toward the viewer, each
/// filled opaque and outlined, so no hidden edge shows through and the
/// picture cannot be read inside out.
fn cube(s: &mut Arena, pose: Pose) -> Mob {
    let (axis, angle) = pose;
    let w = toward_viewer();
    let pts: Vec<Vec3> = CORNERS.iter().map(|&p| project(turn(axis, angle, p))).collect();
    let g = s.add(VMobject::new());
    for (k, &(normal, idx)) in FACES.iter().enumerate() {
        if dot3(turn(axis, angle, normal), w) > 1e-9 {
            let style = Style::default()
                .fill(face_colour(k), 1.0)
                .stroke(Palette::ink(), 2.5, 1.0);
            let f = s.add(Polygon::new(idx.iter().map(|&i| pts[i])).style(style).build());
            let _ = s.attach(g, f);
        }
    }
    g
}

/// A count of half laps for the screen: "½", "1½", "2½" (or a whole number).
fn half_laps(rate: f64) -> String {
    let whole = rate.abs().floor();
    let n = whole as i64;
    if (rate.abs() - whole - 0.5).abs() < 1e-9 {
        if n == 0 { "½".to_string() } else { format!("{n}½") }
    } else {
        n.to_string()
    }
}

/// A spin rate for the screen: "½ lap", "1½ laps", "½ lap the other way".
fn laps_text(rate: f64) -> String {
    let unit = if rate.abs() > 1.0 { "laps" } else { "lap" };
    let way = if rate < 0.0 { " the other way" } else { "" };
    format!("{} {unit}{way}", half_laps(rate))
}

/// 12_4–12_6's take-away as trackers for one redraw: the clock (0 to 1),
/// the spin rate in laps per unit time, which axis turns (0 the rod, 1 the
/// other axis), and switches for the corners' circles and the followed
/// corner with its start and end rings.
#[derive(Clone, Copy)]
struct Puzzle {
    time: Mob,
    rate: Mob,
    other: Mob,
    circles: Mob,
    follow: Mob,
}

impl Puzzle {
    fn on_rod(self, s: &Arena) -> bool {
        value(s, self.other) < 0.5
    }

    /// The pose at `time` of the steady spin now on screen.
    fn pose_at(self, s: &Arena, time: f64) -> Pose {
        let axis = if self.on_rod(s) { ROD } else { unit(OTHER_AXIS) };
        (axis, TAU * value(s, self.rate) * time)
    }

    /// Where the point `p` of the cube has been from time 0 to `now`.
    fn path(self, s: &Arena, p: P3, now: f64) -> Vec<P3> {
        let laps = (value(s, self.rate) * now).abs();
        if laps < 1e-6 {
            return Vec::new();
        }
        let n = ((laps * 96.0).ceil() as usize).max(2);
        (0..=n)
            .map(|k| {
                let (axis, angle) = self.pose_at(s, now * k as f64 / n as f64);
                turn(axis, angle, p)
            })
            .collect()
    }

    /// The whole picture, back to front: the visible faces, then whatever
    /// of the circles, the rod, the other axis and the traces the solid
    /// leaves in view, then the marks.
    fn draw(self, s: &mut Arena) -> Mob {
        let now = value(s, self.time);
        let pose = self.pose_at(s, now);
        let (gold, red) = (answer_c(), warn_c());
        let g = s.add(VMobject::new());
        let solid = cube(s, pose);
        let _ = s.attach(g, solid);
        let on_rod = self.on_rod(s);
        if on_rod && value(s, self.circles) > 0.5 {
            for h in [1.0, -1.0] {
                let ring: Vec<P3> = (0..=72)
                    .map(|k| {
                        let a = TAU * f64::from(k) / 72.0;
                        [SQRT_2 * a.cos(), h, SQRT_2 * a.sin()]
                    })
                    .collect();
                add_runs(s, g, unhidden_runs(&ring, pose), Palette::ink(), 2.0, 0.55);
            }
        }
        let line = |a: P3| -> Vec<P3> {
            (0..=80)
                .map(|k| {
                    let t = ROD_REACH * (f64::from(k) / 40.0 - 1.0);
                    [a[0] * t, a[1] * t, a[2] * t]
                })
                .collect()
        };
        add_runs(s, g, unhidden_runs(&line(ROD), pose), gold, 5.0, 1.0);
        if !on_rod {
            add_runs(s, g, unhidden_runs(&line(unit(OTHER_AXIS)), pose), red, 4.0, 1.0);
            let dragged = self.path(s, TOP_CENTRE, now);
            add_runs(s, g, unhidden_runs(&dragged, pose), red, 3.0, 0.9);
        }
        if on_rod && value(s, self.follow) > 0.5 {
            add_mark(s, g, CORNER, pose, 0.13, gold, true);
            add_mark(s, g, turn(ROD, PI, CORNER), pose, 0.2, gold, true);
            let trace = self.path(s, CORNER, now);
            add_runs(s, g, unhidden_runs(&trace, pose), gold, 3.0, 0.9);
            add_mark(s, g, turn(pose.0, pose.1, CORNER), pose, 0.09, gold, false);
        }
        add_mark(s, g, turn(pose.0, pose.1, TOP_CENTRE), pose, 0.07, gold, false);
        g
    }
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

        // 12_4: a solid cube and a marked rod through its top and bottom face
        // centres; a spin carries each corner round a circle about the rod;
        // half a lap per unit time ends, at time 1, in a half turn
        kit.say(stage, "12_4")?;
        let heading2 = title(&kit, stage, "One to take with you")?;
        swap(stage, &[heading, question], &[heading2], 0.5)?;
        kit.until_sentence(stage, "12_4", 1)?;
        let rates = fact_list("puzzle.spin_rates")?;
        let &[half_rate, lap_and_half, two_and_half, back_rate] = rates.as_slice() else {
            return Err(super::draw::fail("puzzle.spin_rates: want four rates".into()));
        };
        let pz = Puzzle {
            time: stage.add_value_tracker(0.0),
            rate: stage.add_value_tracker(half_rate),
            other: stage.add_value_tracker(0.0),
            circles: stage.add_value_tracker(0.0),
            follow: stage.add_value_tracker(0.0),
        };
        let solid = stage.always_redraw(move |s: &mut Arena| pz.draw(s));
        stage.add_to_scene(solid)?;
        let rod_word = kit.text(
            stage,
            "the rod: our axis",
            20.0,
            gold,
            project([0.0, ROD_REACH + 0.3, 0.0]),
        )?;
        fade_in_all(stage, &[rod_word], 0.4)?;
        kit.until_sentence(stage, "12_4", 2)?;
        stage.set_tracker_value(pz.circles, 1.0)?;
        let circle_word = kit.text_left(
            stage,
            "each corner keeps to a circle about the rod",
            20.0,
            Palette::dim(),
            v(0.6, 2.3),
        )?;
        fade_in_all(stage, &[circle_word], 0.4)?;
        // a quarter lap and back: the corners slide round, the rod stays put
        play!(stage; tween(pz.time, 0.5, 1.8)?);
        play!(stage; tween(pz.time, 0.0, 1.2)?);
        kit.until_sentence(stage, "12_4", 3)?;
        stage.set_tracker_value(pz.follow, 1.0)?;
        let spin_word = |r: f64| format!("spin: {} per unit of time", laps_text(r));
        let rate_word = kit.text_left(stage, &spin_word(half_rate), 22.0, purple, v(0.6, 2.3))?;
        swap(stage, &[circle_word], &[rate_word], 0.3)?;
        let time_word = kit.text_left(stage, "time", 20.0, Palette::dim(), v(0.6, 1.85))?;
        let _time_val = kit.live_value(
            stage,
            move |s| value(s, pz.time),
            2,
            false,
            20.0,
            Palette::ink(),
            v(1.25, 1.85),
            LEFT,
        )?;
        let laps_word = kit.text_left(stage, "laps so far", 20.0, Palette::dim(), v(2.5, 1.85))?;
        let _laps_val = kit.live_value(
            stage,
            move |s| value(s, pz.rate) * value(s, pz.time),
            2,
            false,
            20.0,
            purple,
            v(3.75, 1.85),
            LEFT,
        )?;
        fade_in_all(stage, &[time_word, laps_word], 0.3)?;
        play!(stage; tween(pz.time, 1.0, 3.0)?);
        kit.until_sentence(stage, "12_4", 4)?;
        let half = kit.text_left(
            stage,
            "every corner halfway round: a half turn about the rod",
            22.0,
            gold,
            v(0.6, 1.3),
        )?;
        fade_in_all(stage, &[half], 0.4)?;

        // 12_5: the puzzle, three hints, and time to think
        kit.say(stage, "12_5")?;
        kit.until_sentence(stage, "12_5", 1)?;
        let ask = kit.text_left(
            stage,
            "How many steady spins end at this",
            26.0,
            Palette::ink(),
            v(0.6, 0.6),
        )?;
        let ask2 = kit.text_left(
            stage,
            "half turn after one unit of time?",
            26.0,
            Palette::ink(),
            v(0.6, 0.15),
        )?;
        fade_in_all(stage, &[ask, ask2], 0.5)?;
        kit.until_sentence(stage, "12_5", 2)?;
        let hints = [
            kit.text_left(stage, "the laps", 22.0, purple, v(0.9, -0.45))?,
            kit.text_left(stage, "the direction of spin", 22.0, blue, v(0.9, -0.9))?,
            kit.text_left(stage, "the axis", 22.0, gold, v(0.9, -1.35))?,
        ];
        fade_in_lagged(stage, &hints, 2.4, 0.9)?;
        predict(&kit, stage, v(2.6, -2.1), 3.4)?;

        // 12_6: the answer, one hint at a time, each run on the cube
        kit.say(stage, "12_6")?;
        fade_out_all(stage, &[ask, ask2, half], 0.4)?;
        fade_out_all(stage, &hints, 0.3)?;
        kit.until_sentence(stage, "12_6", 1)?;
        // the laps: one and a half, then two and a half, land in the same place
        let mut rate_shown = rate_word;
        for r in [lap_and_half, two_and_half] {
            stage.set_tracker_value(pz.time, 0.0)?;
            stage.set_tracker_value(pz.rate, r)?;
            let word = kit.text_left(stage, &spin_word(r), 22.0, purple, v(0.6, 2.3))?;
            swap(stage, &[rate_shown], &[word], 0.3)?;
            rate_shown = word;
            play!(stage; tween(pz.time, 1.0, 2.4 + 0.4 * r)?);
        }
        let laps_chip = kit.text_left(
            stage,
            &format!(
                "laps: {}, {}, {}, … all end here",
                half_laps(half_rate),
                half_laps(lap_and_half),
                half_laps(two_and_half)
            ),
            22.0,
            purple,
            v(0.6, 1.3),
        )?;
        fade_in_all(stage, &[laps_chip], 0.4)?;
        kit.until_sentence(stage, "12_6", 2)?;
        // the direction: half a lap the other way
        stage.set_tracker_value(pz.time, 0.0)?;
        stage.set_tracker_value(pz.rate, back_rate)?;
        let word = kit.text_left(stage, &spin_word(back_rate), 22.0, blue, v(0.6, 2.3))?;
        swap(stage, &[rate_shown], &[word], 0.3)?;
        rate_shown = word;
        play!(stage; tween(pz.time, 1.0, 2.6)?);
        let way_chip = kit.text_left(
            stage,
            &format!("direction: {} the other way too", laps_text(half_rate)),
            22.0,
            blue,
            v(0.6, 0.85),
        )?;
        fade_in_all(stage, &[way_chip], 0.4)?;
        kit.until_sentence(stage, "12_6", 3)?;
        // the axis: half a lap about another line drags the top face's
        // centre off the rod, while that line's own points stay put
        stage.set_tracker_value(pz.time, 0.0)?;
        stage.set_tracker_value(pz.rate, half_rate)?;
        stage.set_tracker_value(pz.other, 1.0)?;
        let word = kit.text_left(
            stage,
            &format!("spin: {} about another axis", laps_text(half_rate)),
            22.0,
            red,
            v(0.6, 2.3),
        )?;
        swap(stage, &[rate_shown], &[word], 0.3)?;
        play!(stage; tween(pz.time, 1.0, 2.8)?);
        let axis_chip = kit.text_left(
            stage,
            "axis: only the rod; another axis drags the top off it",
            22.0,
            gold,
            v(0.6, 0.4),
        )?;
        fade_in_all(stage, &[axis_chip], 0.4)?;
        stage.wait(0.6)?;
        // back to the half turn about the rod
        stage.set_tracker_value(pz.other, 0.0)?;
        let back = kit.text_left(stage, &spin_word(half_rate), 22.0, purple, v(0.6, 2.3))?;
        swap(stage, &[word], &[back], 0.3)?;
        kit.until_sentence(stage, "12_6", 4)?;
        let so = kit.text_left(
            stage,
            "so: infinitely many spins, all about the rod",
            24.0,
            gold,
            v(0.6, -0.2),
        )?;
        fade_in_all(stage, &[so], 0.4)?;
        kit.until_sentence(stage, "12_6", 5)?;
        // the turning line of 10_3: these spins turn by ±π, ±3π, …; the
        // two slowest sit on the open ends of the principal window
        let mut marks = vec![seg(
            stage,
            turning_at(-3.4 * PI),
            turning_at(3.4 * PI),
            Palette::faint(),
            2.5,
        )];
        for k in -3..=3_i64 {
            let p = turning_at(PI * k as f64);
            marks.push(seg(
                stage,
                v(p[0], p[1] - 0.08),
                v(p[0], p[1] + 0.08),
                Palette::dim(),
                2.0,
            ));
            marks.push(kit.text(
                stage,
                &super::draw::pi_label(k),
                16.0,
                Palette::dim(),
                v(p[0], p[1] - 0.3),
            )?);
        }
        let window = seg(stage, turning_at(-PI), turning_at(PI), green, 6.0);
        let mut ends = Vec::new();
        for k in [-3.0, -1.0, 1.0, 3.0] {
            ends.push(dot(stage, turning_at(PI * k), 0.08, purple));
        }
        for k in [-1.0, 1.0] {
            ends.push(ring(stage, turning_at(PI * k), 0.15, red, 2.5));
        }
        let line_word = kit.text_left(
            stage,
            "how far each of these spins turns in one unit of time:",
            20.0,
            Palette::ink(),
            v(1.4, -1.5),
        )?;
        let tie = kit.text_left(
            stage,
            "the slowest two, ±π, tie on the window's open edge",
            20.0,
            gold,
            v(1.4, -2.7),
        )?;
        fade_in_all(stage, &marks, 0.4)?;
        fade_in_all(stage, &[window, line_word], 0.4)?;
        fade_in_all(stage, &ends, 0.5)?;
        fade_in_all(stage, &[tie], 0.4)?;
        let source = precisely(
            &kit,
            stage,
            &format!(
                "spins ending here: ±{}, ±{}, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)",
                half_laps(half_rate),
                half_laps(lap_and_half)
            ),
        )?;
        fade_in_all(stage, &[source], 0.4)?;
        kit.until_sentence(stage, "12_6", 6)?;
        let convention = kit.text_left(
            stage,
            "for the number −1, Log(−1) = iπ by convention",
            18.0,
            Palette::dim(),
            v(1.4, -3.1),
        )?;
        fade_in_all(stage, &[convention], 0.4)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close3(a: P3, b: P3) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-12)
    }

    fn pose(axis: P3, rate: f64, time: f64) -> Pose {
        (unit(axis), TAU * rate * time)
    }

    #[test]
    fn a_half_turn_is_reached_by_odd_half_turns_either_way_about_the_rod() {
        let minus = [-ROD[0], -ROD[1], -ROD[2]];
        for &p in &CORNERS {
            let target = turn(ROD, PI, p);
            for k in -3..=3 {
                let odd = PI * f64::from(2 * k + 1);
                assert!(close3(turn(ROD, odd, p), target), "{k}");
                assert!(close3(turn(minus, odd, p), target), "{k}");
            }
            // an even number of half turns is a whole number of laps: home
            assert!(close3(turn(ROD, TAU, p), p));
        }
    }

    #[test]
    fn the_shown_rates_all_end_in_the_half_turn() {
        let rates = super::super::draw::fact_list("puzzle.spin_rates").unwrap();
        assert_eq!(rates, vec![0.5, 1.5, 2.5, -0.5]);
        for r in rates {
            let (axis, angle) = pose(ROD, r, 1.0);
            for &p in &CORNERS {
                assert!(close3(turn(axis, angle, p), turn(ROD, PI, p)), "{r}");
            }
        }
        assert_eq!(laps_text(0.5), "½ lap");
        assert_eq!(laps_text(1.5), "1½ laps");
        assert_eq!(laps_text(2.5), "2½ laps");
        assert_eq!(laps_text(-0.5), "½ lap the other way");
        assert_eq!(half_laps(1.0), "1");
    }

    #[test]
    fn the_two_slowest_spins_tie_so_no_principal_choice_exists() {
        let p = [0.3, -0.7, 1.0];
        assert!(close3(turn(ROD, PI, p), turn(ROD, -PI, p)));
        // and nothing slower than a half turn gets there
        for k in 1..=99 {
            let a = PI * f64::from(k) / 100.0;
            assert!(!close3(turn(ROD, a, p), turn(ROD, PI, p)), "{a}");
        }
    }

    #[test]
    fn a_spin_about_any_other_axis_never_lands_on_the_half_turn() {
        for m in [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.25, 1.0, 0.3], OTHER_AXIS] {
            let m = unit(m);
            for k in 1..400 {
                let a = 4.0 * PI * f64::from(k) / 400.0;
                let lands = CORNERS.iter().all(|&p| close3(turn(m, a, p), turn(ROD, PI, p)));
                assert!(!lands, "{m:?} {a}");
            }
        }
        // the reason shown on screen: the top face's centre, on the rod, is
        // left in place by the half turn and dragged off the rod by the other
        let top = turn(unit(OTHER_AXIS), PI, TOP_CENTRE);
        assert!(close3(turn(ROD, PI, TOP_CENTRE), TOP_CENTRE));
        assert!(top[0].hypot(top[2]) > 0.9, "{top:?}");
        // and it ends on a face turned toward the viewer, so the drag shows
        assert!(close3(top, [-1.0, 0.0, 0.0]) && !hidden(top, pose(OTHER_AXIS, 0.5, 1.0)));
    }

    #[test]
    fn the_viewer_sees_three_faces_and_never_a_face_from_behind() {
        let w = toward_viewer();
        let (sy, cy, sp, cp) = view();
        // at right angles to both screen directions, pointing out of the screen
        assert!(dot3(w, [cy, 0.0, sy]).abs() < 1e-12 && dot3(w, [sy * sp, cp, -cy * sp]).abs() < 1e-12);
        assert!(close3(cross([cy, 0.0, sy], [sy * sp, cp, -cy * sp]), w));
        for (axis, rate) in [(ROD, 2.5), (OTHER_AXIS, 0.5)] {
            for j in 0..=60 {
                let (a, angle) = pose(axis, rate, f64::from(j) / 60.0);
                let seen = FACES
                    .iter()
                    .filter(|&&(n, _)| dot3(turn(a, angle, n), w) > 1e-9)
                    .count();
                assert!((1..=3).contains(&seen), "{seen}");
                // a face's centre is hidden exactly when the face looks away
                for &(n, _) in &FACES {
                    let facing = dot3(turn(a, angle, n), w);
                    if facing.abs() > 1e-6 {
                        assert_eq!(hidden(turn(a, angle, n), (a, angle)), facing < 0.0, "{n:?} {j}");
                    }
                }
            }
        }
        // at the start: the top, the front (+z) and the left (−x) face
        let start: Vec<usize> = (0..6).filter(|&k| dot3(FACES[k].0, w) > 0.0).collect();
        assert_eq!(start, vec![0, 2, 5]);
        // after a half turn about the rod: the top, −z and +x, other colours
        let after: Vec<usize> = (0..6).filter(|&k| dot3(turn(ROD, PI, FACES[k].0), w) > 0.0).collect();
        assert_eq!(after, vec![0, 3, 4]);
    }

    #[test]
    fn the_rod_shows_only_outside_the_cube_and_the_marks_stay_in_view() {
        for j in 0..=50 {
            let p = pose(ROD, 2.5, f64::from(j) / 50.0);
            // inside the solid: hidden; above the top face: always in view
            for y in [-0.9, 0.0, 0.9] {
                assert!(hidden([0.0, y, 0.0], p));
            }
            for y in [1.05, 1.5, ROD_REACH] {
                assert!(!hidden([0.0, y, 0.0], p));
            }
            // the rod's bottom end clears the cube; just under the bottom face it is behind it
            assert!(!hidden([0.0, -ROD_REACH, 0.0], p) && hidden([0.0, -1.1, 0.0], p));
            // the start and end rings, the corner and the top-face centre
            assert!(!hidden(CORNER, p) && !hidden(turn(ROD, PI, CORNER), p));
            assert!(!hidden(turn(p.0, p.1, CORNER), p) && !hidden(turn(p.0, p.1, TOP_CENTRE), p));
        }
        // the corners' lower circle passes behind the cube for part of its way
        let lower: Vec<P3> = (0..=72)
            .map(|k| {
                let a = TAU * f64::from(k) / 72.0;
                [SQRT_2 * a.cos(), -1.0, SQRT_2 * a.sin()]
            })
            .collect();
        let shown: usize = unhidden_runs(&lower, pose(ROD, 0.5, 0.0)).iter().map(Vec::len).sum();
        assert!(shown > 20 && shown < 73, "{shown}");
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
        // the cube, the rod and the other axis stay left of the text column
        // at x 0.6 and between the title and the bottom line
        for (axis, rate) in [(ROD, 2.5), (OTHER_AXIS, 0.5)] {
            for j in 0..=40 {
                let (a, angle) = pose(axis, rate, f64::from(j) / 40.0);
                for &p in &CORNERS {
                    let q = project(turn(a, angle, p));
                    assert!(q[0] < -0.6 && q[1] > -3.0 && q[1] < 2.9, "{q:?}");
                }
            }
            let n = unit(axis);
            for k in [-ROD_REACH, ROD_REACH] {
                let q = project([n[0] * k, n[1] * k, n[2] * k]);
                assert!(q[0] < -0.6 && q[1] > -3.0 && q[1] < 2.9, "{q:?}");
            }
        }
        assert!(project([0.0, ROD_REACH + 0.3, 0.0])[1] < 3.0);
        // the turning line and its labels fit between x 1.4 and the edge
        assert!(turning_at(-3.4 * PI)[0] > 1.4 && turning_at(3.4 * PI)[0] < 6.6);
        let width = |s: &str, size: f64| s.chars().count() as f64 * 0.55 * size / 144.0;
        for (s, size, x) in [
            ("each corner keeps to a circle about the rod", 20.0, 0.6),
            ("spin: ½ lap the other way per unit of time", 22.0, 0.6),
            ("every corner halfway round: a half turn about the rod", 22.0, 0.6),
            ("half turn after one unit of time?", 26.0, 0.6),
            ("axis: only the rod; another axis drags the top off it", 22.0, 0.6),
            ("so: infinitely many spins, all about the rod", 24.0, 0.6),
            ("how far each of these spins turns in one unit of time:", 20.0, 1.4),
            ("the slowest two, ±π, tie on the window's open edge", 20.0, 1.4),
        ] {
            assert!(x + width(s, size) < 6.9, "{s}");
        }
        let fine = "precisely: spins ending here: ±½, ±1½, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)";
        assert!(width(fine, 22.0) < 13.6);
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
