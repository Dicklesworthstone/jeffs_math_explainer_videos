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
//! 10_3's window: a tie, so no principal logarithm (12_6, 12_20, 12_21).
//!
//! Mail 803 repair 3: depth is unambiguous. Only the faces turned toward
//! the viewer are drawn, opaque, and the rod, the other axis, the circles,
//! the traces and the marks are drawn only where the solid does not hide
//! them (`hidden`, a ray-box test). Statement on screen: a half turn has
//! eigenvalue −1, so Higham's principal-logarithm condition (MIMS EPrint
//! 2005.21, section 4 Fact 1) fails; Log(−1) = iπ is the complex
//! convention of chapter 07.
//!
//! Picture repair after the independent review of this chapter (F1–F7). A
//! point of a face stays inside the solid's inscribed ball, so its path can
//! never be seen; the drag is shown by the pin, a copy of the rod's upper stub
//! carried with the cube, whose tip stays outside the solid in every pose. The
//! spins run at constant speed, and their length comes from the sentence they
//! illustrate.
//!
//! The red spin turns about (0, 1, 1), whose line and arcs keep clear of the two
//! rings. A pose never cuts in plain view: a background veil dips over the picture
//! while the trackers jump (`redeal`). Panel B's ruler really folds into the dial,
//! and the family ends on its words: a convention for −1, the whole family here.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`.

use std::collections::HashMap;
use std::f64::consts::{PI, SQRT_2, TAU};

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Kit, Palette, Script, fade_all, hex, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    answer_c, arc_points, dot, fact, fact_f64, fact_list, fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail,
    flat, group, lap_c, number_c, path, position_c, predict, ring, seg, slide_c, swap, title, value, warn_c,
};
use crate::model;

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
/// Panel A: the ruler for ratios, the 1 and its length to the 10; its ticks and
/// its numbers above the spine; where the hops stand.
const RULER_ONE: [f64; 2] = [-3.6, 1.3];
const RULER_LEN: f64 = 3.0;
const RULER_TICK: f64 = 0.32;
const RULER_LABEL: f64 = 0.58;
const HOP_Y: f64 = 2.05;
/// Panel A: the keyboard under the ruler, two octaves of white keys from A to A.
const KEYS_LEFT: f64 = -3.6;
const KEYS_RIGHT: f64 = -0.6;
const KEYS_TOP: f64 = 0.72;
const KEYS_H: f64 = 0.38;
const KEYS_WHITE: i32 = 15;
/// Panel B: the dial, and how far its ticks reach in from the rim.
const DIAL_C: [f64; 2] = [2.3, 1.45];
const DIAL_R: f64 = 0.95;
const DIAL_TICK: f64 = 0.14;
/// Panel C: the plane's zero, its scale, the reach of its axes, the column of
/// log(−1)'s x and its height per π. The zero stands a little high and the axes
/// short, so the lowest label and the axis's foot keep clear of the sentence.
const PLANE_C: [f64; 2] = [-5.6, -1.45];
const PLANE_S: f64 = 0.55;
const PLANE_AXIS: f64 = 0.9;
const COLUMN_X: f64 = -3.9;
const COLUMN_PER_PI: f64 = 0.28;
/// Panel D: the plane's zero and scene units per unit.
const FLOWS_C: [f64; 2] = [2.0, -1.55];
const FLOWS_U: f64 = 0.32;
/// The one place every flow of panel D leaves from and comes home to, and
/// the ring that marks it.
const SHARED_START: [f64; 2] = [1.6, 0.0];
const SHARED_RING_R: f64 = 0.13;
/// Panel C: the left edge of the two lines beside its column of logarithms.
const PLANE_WORDS_X: f64 = -3.1;
/// Baseline of the final family label, shared with its clearance check.
const FAMILY_LABEL_Y: f64 = -2.9;
/// The take-away: the cube's centre and half edge in scene units. The
/// marked axis, the rod, is vertical through the centres of the top and
/// bottom faces, drawn `ROD_REACH` half edges each way from the centre.
const CUBE_C: [f64; 2] = [-3.0, -0.55];
const CUBE_HALF: f64 = 1.05;
const ROD: P3 = [0.0, 1.0, 0.0];
const ROD_REACH: f64 = 2.0;
/// 12_20's other axis (normalised where used), tilted 45° from the rod through
/// the middle of an edge, so the cube lands on itself. Of the edge axes this one
/// keeps its line, the pin and the arcs clear of both rings, and the top face's
/// centre ends in view on the front face.
const OTHER_AXIS: P3 = [0.0, 1.0, 1.0];
/// The corner followed round its circle, and the top face's centre.
const CORNER: P3 = [1.0, 1.0, 1.0];
const TOP_CENTRE: P3 = [0.0, 1.0, 0.0];
/// The rings at the corner's start and at the half turn's end.
const START_RING_R: f64 = 0.13;
const END_RING_R: f64 = 0.2;
/// A dip: how long the veil takes to come and go, and how far it reaches from the
/// cube's centre. It covers all of the picture and none of the words.
const DIP: f64 = 0.4;
const VEIL_HALF: (f64, f64) = (1.8, 2.1);
/// The tip of 12_20's pin, the rod's upper stub carried with the cube by a spin
/// about another axis. A point of the top face stays within radius 1, inside
/// the inscribed ball, so the solid hides its whole path in every pose. This
/// tip is at radius `ROD_REACH`, beyond the corners' √3, so it never enters
/// the solid and its path can be drawn through the same depth test.
const PIN_TIP: P3 = [0.0, ROD_REACH, 0.0];
/// How far the other axis is drawn each way from the centre: both ends leave
/// the cube's outline, so it reads as a line through the solid.
const OTHER_REACH: f64 = 3.0;
/// The fixed view: yaw about the vertical, then pitch toward us, degrees.
const YAW_DEG: f64 = 35.0;
const PITCH_DEG: f64 = 20.0;
/// 12_21's turning line: the centre's x and y, and scene units per π.
const TURNING_X: f64 = 3.9;
const TURNING_Y: f64 = -1.4;
const TURNING_PER_PI: f64 = 0.62;
/// 12_20's second sentence as shares of its length, from even word spacing
/// over its 34 words (a proposal until fine sync): the red spin ends with "where
/// it was", holds through "and our half turn leaves", goes back for "only the
/// rod's points", and the chip comes with "so any other axis is ruled out".
const AXIS_SPIN_END: f64 = 0.45;
const AXIS_RESET: f64 = 0.60;
const AXIS_CHIP: f64 = 0.79;
/// 12_21's first sentence: the tie appears as the voice reaches "the very edge".
const TIE_AT: f64 = 0.45;
/// The shortest a spin may run when its sentence leaves little room, seconds.
const MIN_RUN: f64 = 1.0;
/// The words the film speaks, for the phrase hooks below.
const SCRIPT: &str = include_str!("../../script.tsv");
/// 12_6's second sentence: the 1½-lap run lands by the first phrase and stays
/// landed until the second, when the 2½-lap run begins.
const ALSO_ENDS_HERE: &str = "also ends here";
const AND_TWO_AND_A_HALF: &str = "and so does two and a half";
/// 12_1: the keys come when the line names them.
const THE_KEYBOARD: &str = "the keyboard and the slide rule";
/// 12_21's last sentence: the number's convention, then the family lit up.
const CONVENTION: &str = "a convention broke the same tie";
const WHOLE_FAMILY: &str = "the honest answer is the whole family";
/// However the take falls, the 1½-lap arrival holds at least this long, seconds:
/// a chosen floor for the eye, not a timing of any voice.
const ARRIVAL_HOLD: f64 = 0.5;
/// What the 2½-lap run leaves before its line ends: the laps chip's fade and
/// a little air.
const LAPS_CHIP_ROOM: f64 = 0.5;

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

/// Where the point `p` of the cube has been from time 0 to `now` in the steady
/// spin of `rate` laps per unit time about the unit axis `axis`.
fn spin_path(axis: P3, rate: f64, p: P3, now: f64) -> Vec<P3> {
    let laps = (rate * now).abs();
    if laps < 1e-6 {
        return Vec::new();
    }
    let n = ((laps * 96.0).ceil() as usize).max(2);
    (0..=n)
        .map(|k| turn(axis, TAU * rate * now * k as f64 / n as f64, p))
        .collect()
}

/// The line along the unit vector `a`, `reach` half edges each way from the
/// centre, in pieces short enough for the depth test.
fn axis_points(a: P3, reach: f64) -> Vec<P3> {
    let n = (reach * 40.0).ceil() as i32;
    (0..=n)
        .map(|k| {
            let t = reach * (2.0 * f64::from(k) / f64::from(n) - 1.0);
            [a[0] * t, a[1] * t, a[2] * t]
        })
        .collect()
}

/// The pin in `pose`, from the top face's centre to its tip, in pieces short
/// enough for the depth test.
fn pin_points(pose: Pose) -> Vec<P3> {
    let (a, b) = (turn(pose.0, pose.1, TOP_CENTRE), turn(pose.0, pose.1, PIN_TIP));
    (0..=24)
        .map(|k| {
            let f = f64::from(k) / 24.0;
            [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ]
        })
        .collect()
}

/// 12_4–12_21's take-away as trackers for one redraw: the clock (0 to 1),
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
        let (axis, _) = self.pose_at(s, now);
        spin_path(axis, value(s, self.rate), p, now)
    }

    /// The whole picture, back to front: the visible faces, then whatever
    /// of the circles, the pin, the rod, the other axis and the traces the
    /// solid leaves in view, then the marks.
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
        if !on_rod {
            // under the gold rod: at the spin's start the rod lies over its copy
            add_runs(s, g, unhidden_runs(&pin_points(pose), pose), red, 3.5, 1.0);
        }
        add_runs(s, g, unhidden_runs(&axis_points(ROD, ROD_REACH), pose), gold, 5.0, 1.0);
        if !on_rod {
            let other = axis_points(unit(OTHER_AXIS), OTHER_REACH);
            add_runs(s, g, unhidden_runs(&other, pose), red, 4.0, 1.0);
            let dragged = self.path(s, PIN_TIP, now);
            add_runs(s, g, unhidden_runs(&dragged, pose), red, 3.0, 0.9);
        }
        if value(s, self.follow) > 0.5 {
            // the rings mark the half turn's start and end whatever now spins, so
            // a spin about another axis shows its corner missing the end ring
            add_mark(s, g, CORNER, pose, START_RING_R, gold, true);
            add_mark(s, g, turn(ROD, PI, CORNER), pose, END_RING_R, gold, true);
            let moving = if on_rod { gold } else { red };
            let trace = self.path(s, CORNER, now);
            add_runs(s, g, unhidden_runs(&trace, pose), moving, 3.0, 0.9);
            add_mark(s, g, turn(pose.0, pose.1, CORNER), pose, 0.09, moving, false);
        }
        add_mark(s, g, turn(pose.0, pose.1, TOP_CENTRE), pose, 0.07, gold, false);
        if !on_rod {
            add_mark(s, g, turn(pose.0, pose.1, PIN_TIP), pose, 0.07, red, false);
        }
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

/// Where the flow of the ellipse (a, b, tilt) carries the shared start after
/// time `t`: e^{t·2πPJP⁻¹} applied to it. All the flows leave one place, part,
/// and come home together at t = 1. They also meet once at t = 1/2, at the
/// opposite point, because each half-time flow is −I.
fn orbit_point(a: f64, b: f64, tilt: f64, t: f64) -> [f64; 2] {
    model::apply(model::flow(model::ellipse_generator(a, b, tilt), t), SHARED_START)
}

/// A tracker's move at constant speed: this chapter's word is "steady", and
/// `tween` takes the engine's default easing.
fn steady(tracker: Mob, to: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(to)?)
}

/// How long sentence `j` of a line lasts, from the line's sentence starts;
/// `None` for its last sentence, or without a narrator.
fn sentence_len(starts: &[f64], j: usize) -> Option<f64> {
    Some(starts.get(j + 1)? - starts.get(j)?)
}

/// A run time for a motion that begins `used` seconds into sentence `j` and
/// must end `margin` seconds before the next sentence, so that the next
/// `until_sentence` still has something to wait for. `fallback` without
/// sentence times.
fn room(starts: &[f64], j: usize, used: f64, margin: f64, fallback: f64) -> f64 {
    sentence_len(starts, j).map_or(fallback, |len| (len - used - margin).max(MIN_RUN))
}

/// Wait until `share` of sentence `j` of line `id` has gone by, the sentence
/// having begun at scene time `began`; nothing if that moment has passed, or
/// without sentence times.
fn until_share(kit: &Kit, stage: &mut Stage<'_>, id: &str, j: usize, began: f64, share: f64) -> fmn::Result<()> {
    let Some(len) = sentence_len(&kit.sentence_starts(id), j) else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    let at = began + share * len;
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

/// When `phrase` begins, in seconds from the start of a line with this `text`:
/// the narrator's word mark if it has one for that word, otherwise a share by
/// characters of the measured sentence the phrase sits in. The share is a
/// provisional draft cue, not a heard boundary: fine sync waits for word marks
/// measured on the takes. A phrase the line lacks, or a narrator that splits the
/// line differently, is an error, never a guess.
fn phrase_seconds(
    text: &str,
    phrase: &str,
    marks: &HashMap<usize, f64>,
    starts: &[f64],
    len: f64,
) -> Result<f64, String> {
    let byte = text
        .find(phrase)
        .ok_or_else(|| format!("the line has no phrase {phrase:?}"))?;
    if let Some(&seconds) = marks.get(&byte) {
        return Ok(seconds);
    }
    let mut opens = vec![0];
    for (i, pair) in text.as_bytes().windows(2).enumerate() {
        if matches!(pair[0], b'.' | b'?' | b'!') && pair[1] == b' ' {
            opens.push(i + 2);
        }
    }
    if opens.len() != starts.len() {
        return Err(format!(
            "the line has {} sentences, but the narrator timed {}",
            opens.len(),
            starts.len()
        ));
    }
    let j = opens.iter().rposition(|&open| open <= byte).unwrap_or(0);
    let close = opens.get(j + 1).copied().unwrap_or(text.len());
    let (t0, t1) = (starts[j], starts.get(j + 1).copied().unwrap_or(len));
    let before = text[opens[j]..byte].chars().count() as f64;
    let whole = text[opens[j]..close].trim_end().chars().count().max(1) as f64;
    Ok(t0 + (t1 - t0) * before / whole)
}

/// The scene time `phrase` of line `id` begins at, once the line has been said;
/// `None` without a narrator.
fn phrase_time(kit: &Kit, id: &str, phrase: &str) -> fmn::Result<Option<f64>> {
    let Some(n) = kit.narrator.as_ref() else {
        return Ok(None);
    };
    let starts = n.sentence_starts(id);
    let (Some(&first), Some(said), Some(len)) = (starts.first(), n.sentence_at(id, 0), n.length(id)) else {
        return Ok(None);
    };
    let script = Script::parse(SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?;
    let seconds =
        phrase_seconds(text, phrase, &n.phrase_marks(id), &starts, len).map_err(|e| fail(format!("line {id}: {e}")))?;
    Ok(Some(said - first + seconds))
}

/// The scene time line `id` ends at, once it has been said.
fn line_end(kit: &Kit, id: &str) -> Option<f64> {
    let n = kit.narrator.as_ref()?;
    let first = *n.sentence_starts(id).first()?;
    Some(n.sentence_at(id, 0)? - first + n.length(id)?)
}

/// Wait until `phrase` of line `id` begins; nothing if that moment has passed.
fn until_phrase(kit: &Kit, stage: &mut Stage<'_>, id: &str, phrase: &str) -> fmn::Result<()> {
    let Some(at) = phrase_time(kit, id, phrase)? else {
        return Ok(());
    };
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}

/// A spin's run time: `longest`, or less if it must end by scene time `by`, but
/// never under `MIN_RUN`.
fn run_by(by: Option<f64>, now: f64, longest: f64) -> f64 {
    by.map_or(longest, |at| (at - now).clamp(MIN_RUN, longest))
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

/// Words give way to words in the same place: the old ones leave first, so the
/// two never print over each other.
fn reword(stage: &mut Stage<'_>, out: &[Mob], inn: &[Mob], rt: f64) -> fmn::Result<()> {
    fade_out_all(stage, out, rt / 2.0)?;
    fade_in_all(stage, inn, rt / 2.0)
}

/// A new pose for the cube, never a jump in plain view: a veil the colour of the
/// background comes over the picture as `out` leaves, the trackers move under it,
/// and it lifts as `inn` arrives.
fn redeal(stage: &mut Stage<'_>, set: &[(Mob, f64)], out: &[Mob], inn: &[Mob]) -> fmn::Result<()> {
    let r = Rectangle::new()
        .width(2.0 * VEIL_HALF.0)
        .height(2.0 * VEIL_HALF.1)
        .style(flat(hex(BACKGROUND), 1.0))
        .build()?
        .moved_to(v(CUBE_C[0], CUBE_C[1]));
    let veil = stage.arena_mut().add(r);
    swap(stage, out, &[veil], DIP / 2.0)?;
    for &(tracker, to) in set {
        stage.set_tracker_value(tracker, to)?;
    }
    swap(stage, &[veil], inn, DIP / 2.0)
}

/// Where panel A's ruler puts `value`.
fn ruler_x(value: f64) -> f64 {
    RULER_ONE[0] + RULER_LEN * value.log10()
}

/// Panel A's ruler for ratios: a tick at each whole number from 1 to 10, and
/// numbers only at the doublings the hops land on.
fn ratio_ruler(kit: &Kit, stage: &mut Stage<'_>, c: Srgb) -> fmn::Result<Mob> {
    let y = RULER_ONE[1];
    let mut parts = vec![seg(stage, v(ruler_x(1.0), y), v(ruler_x(10.0), y), c, 3.0)];
    for n in 1..=10 {
        let x = ruler_x(f64::from(n));
        parts.push(seg(stage, v(x, y), v(x, y + RULER_TICK), c, 2.5));
    }
    for n in [1, 2, 4, 8] {
        let at = v(ruler_x(f64::from(n)), y + RULER_LABEL);
        parts.push(kit.text(stage, &n.to_string(), 14.0, c, at)?);
    }
    group(stage, &parts)
}

/// Panel A's keyboard: two octaves of white keys from A to A, the black keys cut
/// in the background colour, and a gold dot on each A, every one a doubling.
fn mini_keyboard(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let w = (KEYS_RIGHT - KEYS_LEFT) / f64::from(KEYS_WHITE);
    let key_x = |j: i32| KEYS_LEFT + w * (f64::from(j) + 0.5);
    let mut parts = Vec::new();
    for j in 0..KEYS_WHITE {
        let key = Rectangle::new()
            .width(w * 0.9)
            .height(KEYS_H)
            .style(flat(Palette::faint(), 1.0))
            .build()?
            .moved_to(v(key_x(j), KEYS_TOP - KEYS_H / 2.0));
        parts.push(stage.arena_mut().add(key));
    }
    // from A, a black key follows A, C, D, F and G
    for j in (0..KEYS_WHITE - 1).filter(|j| matches!(j % 7, 0 | 2 | 3 | 5 | 6)) {
        let key = Rectangle::new()
            .width(w * 0.6)
            .height(KEYS_H * 0.6)
            .style(flat(hex(BACKGROUND), 1.0))
            .build()?
            .moved_to(v(key_x(j) + w / 2.0, KEYS_TOP - KEYS_H * 0.3));
        parts.push(stage.arena_mut().add(key));
    }
    for j in [0, 7, 14] {
        parts.push(dot(stage, v(key_x(j), KEYS_TOP - KEYS_H * 0.78), 0.05, answer_c()));
    }
    group(stage, &parts)
}

/// A point of the ruler as it folds into the dial: `laps` along it (0 at the 1, 1
/// at the 10) and `inset` in from its rim. `fold` closes it from flat across panel
/// B to a circle with its ends at the bottom, keeping its middle at the top of the
/// bend and its length fixed; `turned` then brings the 1 round to the top, where it
/// is exactly `dial_at`.
fn folded_at(fold: f64, turned: f64, laps: f64, inset: f64) -> Vec3 {
    let top = [PANELS[1][0] + (DIAL_C[0] - PANELS[1][0]) * fold, DIAL_C[1] + DIAL_R];
    if fold < 1e-4 {
        return v(top[0] + TAU * DIAL_R * (laps - 0.5), top[1] - inset);
    }
    let r = DIAL_R / fold;
    let a = PI * fold * (1.0 - turned) - TAU * fold * laps;
    v(top[0] - (r - inset) * a.sin(), top[1] - r + (r - inset) * a.cos())
}

/// The ruler part way to a dial: its rim and a tick at each whole number. At the
/// end the 10 lands on the 1, which is what the dial forgets.
fn folded_ruler(s: &mut Arena, fold: f64, turned: f64) -> Mob {
    let g = s.add(VMobject::new());
    let rim: Vec<Vec3> = (0..=72)
        .map(|k| folded_at(fold, turned, f64::from(k) / 72.0, 0.0))
        .collect();
    let spine = s.add(polyline(rim).style(faded_line(Palette::dim(), 2.5, 1.0)).build());
    let _ = s.attach(g, spine);
    for k in 1..=10 {
        let laps = f64::from(k).log10();
        let a = folded_at(fold, turned, laps, 0.0);
        let b = folded_at(fold, turned, laps, DIAL_TICK);
        let tick = s.add(polyline(vec![a, b]).style(faded_line(position_c(), 2.0, 1.0)).build());
        let _ = s.attach(g, tick);
    }
    g
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
        let scale = ratio_ruler(&kit, stage, blue)?;
        let mut hops = Vec::new();
        for k in 0..3 {
            let (a, b) = (ruler_x(2f64.powi(k)), ruler_x(2f64.powi(k + 1)));
            let (mid, r) = ((a + b) / 2.0, (b - a) / 2.0);
            let pts: Vec<Vec3> = arc_points(v(mid, HOP_Y), r, 0.0, PI, 16).into_iter().rev().collect();
            hops.push(path(stage, pts, green, 2.5));
        }
        let cap_a2 = kit.text_left(stage, "×2 is always the same step", 20.0, green, v(KEYS_LEFT, 0.95))?;
        fade_in_all(stage, &[scale], 0.5)?;
        fade_in_lagged(stage, &hops, 1.0, 0.3)?;
        fade_in_all(stage, &[cap_a2], 0.4)?;
        until_phrase(&kit, stage, "12_1", THE_KEYBOARD)?;
        let keys = mini_keyboard(stage)?;
        fade_in_all(stage, &[keys], 0.4)?;

        // 12_2: the dial forgets the decade; the complex exponential the laps
        kit.say(stage, "12_2")?;
        // the ruler itself, flat across panel B, folds shut and turns its 1 to the top
        let flat_ruler = folded_ruler(stage.arena_mut(), 0.0, 0.0);
        fade_in_all(stage, &[flat_ruler], 0.3)?;
        let (fold, turned) = (stage.add_value_tracker(0.0), stage.add_value_tracker(0.0));
        let dial = stage.always_redraw(move |s: &mut Arena| {
            let (f, t) = (value(s, fold), value(s, turned));
            folded_ruler(s, f, t)
        });
        stage.scene_mut().replace(flat_ruler, &[dial])?;
        play!(stage; tween(fold, 1.0, 1.2)?);
        play!(stage; tween(turned, 1.0, 0.5)?);
        let mut dial_marks = Vec::new();
        for k in 1..=9 {
            let laps = f64::from(k).log10();
            dial_marks.push(kit.text(stage, &k.to_string(), 14.0, blue, dial_at(laps, DIAL_R + 0.2))?);
        }
        let mark = fact_f64("dial.mark")?;
        let pointer = seg(stage, v(DIAL_C[0], DIAL_C[1]), dial_at(mark, DIAL_R - 0.05), gold, 3.0);
        // the one mark the caption names: on the rim, with its value outside it
        let mark_dot = dot(stage, dial_at(mark, DIAL_R), 0.05, gold);
        let mark_word = kit.text(
            stage,
            &format!("{:.1}", 10f64.powf(mark)),
            14.0,
            gold,
            dial_at(mark, DIAL_R + 0.2),
        )?;
        let values = fact("dial.values")?.replace(',', ", ");
        let same = kit.text_left(stage, &values, 20.0, gold, v(3.6, 1.85))?;
        let one_mark = kit.text_left(stage, "all land on one mark", 20.0, gold, v(3.6, 1.4))?;
        let cap_b = kit.text_left(stage, "the dial forgets the decade", 20.0, purple, v(3.6, 0.75))?;
        fade_in_all(stage, &dial_marks, 0.3)?;
        fade_in_all(stage, &[pointer, mark_dot, mark_word, same, one_mark, cap_b], 0.5)?;
        kit.until_sentence(stage, "12_2", 1)?;
        let axes = {
            let (c, w) = (PLANE_C, PLANE_AXIS);
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
                Palette::ink(),
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
        // −1 stays on top of the leaders and of the cut, which runs through it
        stage.set_z_index(target, 1, true);
        fade_in_lagged(stage, &column, 1.2, 0.08)?;
        fade_in_all(stage, &[cap_c], 0.4)?;
        kit.until_sentence(stage, "12_2", 2)?;
        let cut = seg(
            stage,
            v(PLANE_C[0], PLANE_C[1]),
            v(PLANE_C[0] - PLANE_AXIS, PLANE_C[1]),
            red,
            4.0,
        );
        let cut_word = kit.text_left(stage, "a cut chooses the laps", 18.0, red, v(PLANE_WORDS_X, -1.2))?;
        let rule = kit.text_left(
            stage,
            "Log zw = Log z + Log w + 2πik",
            18.0,
            purple,
            v(PLANE_WORDS_X, -1.7),
        )?;
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
        let home = ring(stage, flows_screen(SHARED_START), SHARED_RING_R, Palette::ink(), 2.5);
        let clock = stage.add_value_tracker(0.0);
        let movers_on = stage.add_value_tracker(0.0);
        let all: Vec<(f64, f64, f64)> = std::iter::once((1.0, 1.0, 0.0)).chain(specs.iter().copied()).collect();
        let movers = stage.always_redraw(move |s: &mut Arena| {
            let (t, on) = (value(s, clock), value(s, movers_on));
            let g = s.add(VMobject::new());
            for &(a, b, tilt) in &all {
                let d = s.add(
                    Dot::new()
                        .point(flows_screen(orbit_point(a, b, tilt, t)))
                        .radius(0.07)
                        .style(flat(green, on))
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
        play!(stage; tween(movers_on, 1.0, 0.3)?);
        play!(stage; steady(clock, 1.0, 2.6)?);
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
            x_axis, curve, probe, hit, cap_a, scale, cap_a2, keys, dial, pointer, mark_dot, mark_word, same, one_mark,
            cap_b,
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
        reword(stage, &[heading, question], &[heading2], 0.5)?;
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
        // one steady lap across the sentence: the corners sweep their circles and
        // the rod stays put; a whole lap ends where it began, so the reset is unseen
        let lap = room(&kit.sentence_starts("12_4"), 2, 0.4, 0.6, 3.0);
        play!(stage; steady(pz.time, 1.0 / half_rate, lap)?);
        stage.set_tracker_value(pz.time, 0.0)?;
        kit.until_sentence(stage, "12_4", 3)?;
        stage.set_tracker_value(pz.follow, 1.0)?;
        let spin_word = |r: f64| format!("spin: {} per unit of time", laps_text(r));
        let rate_word = kit.text_left(stage, &spin_word(half_rate), 22.0, purple, v(0.6, 2.3))?;
        reword(stage, &[circle_word], &[rate_word], 0.3)?;
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
        // the corner arrives as the sentence says it does: the swap and the fade
        // above take 0.6 s of it
        let to_half_way = room(&kit.sentence_starts("12_4"), 3, 0.6, 0.8, 3.0);
        play!(stage; steady(pz.time, 1.0, to_half_way)?);
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
        // the laps: one and a half, then two and a half, land in the same place. The
        // first lands by "also ends here" and stays landed until "and so does two and
        // a half"; the second must leave the chip its fade before the line ends
        let mut rate_shown = rate_word;
        let chip_by = line_end(&kit, "12_6").map(|at| at - LAPS_CHIP_ROOM);
        for (r, by, hold) in [
            (lap_and_half, phrase_time(&kit, "12_6", ALSO_ENDS_HERE)?, true),
            (two_and_half, chip_by, false),
        ] {
            let word = kit.text_left(stage, &spin_word(r), 22.0, purple, v(0.6, 2.3))?;
            redeal(stage, &[(pz.time, 0.0), (pz.rate, r)], &[rate_shown], &[word])?;
            rate_shown = word;
            let now = stage.scene().time().to_f64();
            play!(stage; steady(pz.time, 1.0, run_by(by, now, 2.4 + 0.4 * r))?);
            if hold {
                stage.wait(ARRIVAL_HOLD)?;
                until_phrase(&kit, stage, "12_6", AND_TWO_AND_A_HALF)?;
            }
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
        // 12_20: the answer goes on, split at the narration's cap of 100 words
        kit.say(stage, "12_20")?;
        // the direction: half a lap the other way
        let word = kit.text_left(stage, &spin_word(back_rate), 22.0, blue, v(0.6, 2.3))?;
        redeal(stage, &[(pz.time, 0.0), (pz.rate, back_rate)], &[rate_shown], &[word])?;
        rate_shown = word;
        play!(stage; steady(pz.time, 1.0, 2.6)?);
        let way_chip = kit.text_left(
            stage,
            &format!("direction: {} the other way too", laps_text(half_rate)),
            22.0,
            blue,
            v(0.6, 0.85),
        )?;
        fade_in_all(stage, &[way_chip], 0.4)?;
        kit.until_sentence(stage, "12_20", 1)?;
        let axis_began = stage.scene().time().to_f64();
        // the axis: half a lap about another line drags the top face's
        // centre off the rod, while that line's own points stay put
        let word = kit.text_left(
            stage,
            &format!("spin: {} about another axis", laps_text(half_rate)),
            22.0,
            red,
            v(0.6, 2.3),
        )?;
        redeal(
            stage,
            &[(pz.time, 0.0), (pz.rate, half_rate), (pz.other, 1.0)],
            &[rate_shown],
            &[word],
        )?;
        let red_spin =
            sentence_len(&kit.sentence_starts("12_20"), 1).map_or(2.8, |len| (AXIS_SPIN_END * len - DIP).max(MIN_RUN));
        play!(stage; steady(pz.time, 1.0, red_spin)?);
        // hold the miss, then back to the half turn about the rod, its top
        // face's centre on the rod in plain view for "only the rod's points"
        until_share(&kit, stage, "12_20", 1, axis_began, AXIS_RESET)?;
        let back = kit.text_left(stage, &spin_word(half_rate), 22.0, purple, v(0.6, 2.3))?;
        redeal(stage, &[(pz.other, 0.0)], &[word], &[back])?;
        until_share(&kit, stage, "12_20", 1, axis_began, AXIS_CHIP)?;
        let axis_chip = kit.text_left(
            stage,
            "axis: only the rod; another axis drags the top off it",
            22.0,
            gold,
            v(0.6, 0.4),
        )?;
        fade_in_all(stage, &[axis_chip], 0.4)?;
        kit.until_sentence(stage, "12_20", 2)?;
        let so = kit.text_left(
            stage,
            "so: infinitely many spins, all about the rod",
            24.0,
            gold,
            v(0.6, -0.2),
        )?;
        fade_in_all(stage, &[so], 0.4)?;
        // 12_21: the tie and the family, split the same way
        kit.say(stage, "12_21")?;
        let tie_began = stage.scene().time().to_f64();
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
                20.0,
                Palette::ink(),
                v(p[0], p[1] - 0.32),
            )?);
        }
        // the family goes on past the last mark, both ways
        for side in [-1.0, 1.0] {
            marks.push(kit.text(stage, "…", 20.0, Palette::ink(), turning_at(side * 3.75 * PI))?);
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
            v(1.4, -0.9),
        )?;
        let tie = kit.text_left(
            stage,
            "the slowest two, ±π, tie on the window's open edge",
            20.0,
            gold,
            v(1.4, -2.1),
        )?;
        fade_in_all(stage, &marks, 0.4)?;
        fade_in_all(stage, &[window, line_word], 0.4)?;
        fade_in_all(stage, &ends, 0.5)?;
        // the tie as the voice reaches the window's edge, and the conclusion only
        // once the voice has said it
        until_share(&kit, stage, "12_21", 0, tie_began, TIE_AT)?;
        fade_in_all(stage, &[tie], 0.4)?;
        kit.until_sentence(stage, "12_21", 1)?;
        let source = footer(
            &kit,
            stage,
            &format!(
                "spins ending here: ±{}, ±{}, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)",
                half_laps(half_rate),
                half_laps(lap_and_half)
            ),
        )?;
        fade_in_all(stage, &[source], 0.4)?;
        until_phrase(&kit, stage, "12_21", CONVENTION)?;
        let convention = kit.text_left(
            stage,
            "for the number −1, Log(−1) = iπ by convention",
            20.0,
            Palette::ink(),
            v(1.4, -2.5),
        )?;
        fade_in_all(stage, &[convention], 0.4)?;
        // here no convention picks one: every spin on the line lights up
        until_phrase(&kit, stage, "12_21", WHOLE_FAMILY)?;
        let mut family = vec![kit.text_left(stage, "here: the whole family", 20.0, gold, v(1.4, FAMILY_LABEL_Y))?];
        for k in [-3.0, -1.0, 1.0, 3.0] {
            family.push(dot(stage, turning_at(PI * k), 0.1, gold));
        }
        fade_in_all(stage, &family, 0.4)?;
        // the film's end: the last line finishes first, then the card is left to be read, and only
        // then does everything fade to the background. No line follows
        kit.music_breath(stage, CLOSING_HOLD)?;
        fade_all(stage, CLOSING_FADE)?;
        kit.finish(stage)
    }
}

/// The closing footer: `draw::precisely`'s words, in ink and clear of the frame's edge. It stands after the film's
/// code so that no held line moves.
const FOOTER_Y: f64 = -3.4;

/// After the last word the finished card stays this long for reading, in a window of its own,
/// then fades this long, in seconds. Chosen for the eye, not timed to any voice.
const CLOSING_HOLD: f64 = 5.5;
const CLOSING_FADE: f64 = 1.5;

fn footer(kit: &Kit, stage: &mut Stage<'_>, words: &str) -> fmn::Result<Mob> {
    kit.text(
        stage,
        &format!("precisely: {words}"),
        22.0,
        Palette::ink(),
        v(0.0, FOOTER_Y),
    )
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

    /// The screen length of the stretches a depth test keeps.
    fn drawn(runs: &[Vec<Vec3>]) -> f64 {
        runs.iter()
            .flat_map(|r| r.windows(2))
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .sum()
    }

    /// Text advances: CMR10 widths, em × size × 0.01 scene units, within 2%
    /// of lint-measured lines; unlisted characters count as wide.
    fn em(c: char) -> f64 {
        match c {
            'i' | 'l' | ',' | '.' | ':' | ';' | '!' | '[' | ']' => 0.278,
            'j' | 'f' => 0.306,
            ' ' | '-' => 0.333,
            'I' => 0.361,
            'r' | 's' | 't' | '(' | ')' => 0.39,
            'a' | 'c' | 'e' | 'z' => 0.444,
            'g' | 'o' | '^' | '/' | '0'..='9' => 0.5,
            'k' | 'q' | 'v' | 'x' | 'y' => 0.528,
            'b' | 'd' | 'h' | 'n' | 'p' | 'u' => 0.556,
            'L' => 0.625,
            'w' => 0.722,
            '=' | '+' | '−' | '×' | '±' | '<' | '>' => 0.778,
            'm' => 0.833,
            _ => 0.75,
        }
    }

    fn width(s: &str, size: f64) -> f64 {
        s.chars().map(em).sum::<f64>() * size * 0.01
    }

    /// The layout lint's limit for text, and a little room for the table's error.
    const TEXT_RIGHT: f64 = 7.11 - 0.25;
    const WIDTH_ROOM: f64 = 1.03;

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
        // and it ends on a face turned toward the viewer, so its dot ends in view
        assert!(close3(top, [0.0, 0.0, 1.0]) && !hidden(top, pose(OTHER_AXIS, 0.5, 1.0)));
    }

    #[test]
    fn a_spin_about_another_axis_draws_the_drag_and_the_miss() {
        let n = unit(OTHER_AXIS);
        for j in 1..=40 {
            let t = f64::from(j) / 40.0;
            let p = pose(OTHER_AXIS, 0.5, t);
            // why the pin exists: the depth test takes every bit of the top
            // centre's path, which stays inside the solid
            assert!(
                drawn(&unhidden_runs(&spin_path(n, 0.5, TOP_CENTRE, t), p)) < 1e-12,
                "{t}"
            );
            // the pin's tip is outside the solid, and in this view never behind it
            let arc = spin_path(n, 0.5, PIN_TIP, t);
            let whole: Vec<Vec3> = arc.iter().map(|&q| project(q)).collect();
            let shown = drawn(&unhidden_runs(&arc, p));
            assert!((shown - drawn(&[whole])).abs() < 1e-9, "{t}: {shown}");
            assert!(!hidden(turn(p.0, p.1, PIN_TIP), p), "{t}");
            // the corner's start and end rings stay in view for the whole spin
            assert!(!hidden(CORNER, p) && !hidden(turn(ROD, PI, CORNER), p), "{t}");
        }
        // a guard on what is drawn, not on where things end
        for t in [0.25, 0.5, 0.75, 1.0] {
            let p = pose(OTHER_AXIS, 0.5, t);
            let corner = drawn(&unhidden_runs(&spin_path(n, 0.5, CORNER, t), p));
            assert!(corner > 0.8, "{t}: {corner}");
            assert!(drawn(&unhidden_runs(&spin_path(n, 0.5, PIN_TIP, t), p)) > 0.85, "{t}");
        }
        // the pin points nearly at the viewer through the middle of the spin, when its
        // tip and arc carry the drag; it has real length at both ends
        for (t, least) in [(0.025, 0.9), (1.0, 0.6)] {
            let p = pose(OTHER_AXIS, 0.5, t);
            assert!(drawn(&unhidden_runs(&pin_points(p), p)) > least, "{t}");
        }
        // the miss: the corner lands in view, one edge from its end ring, and so
        // does every corner, though the outline is the same cube
        let end = pose(OTHER_AXIS, 0.5, 1.0);
        assert!(close3(turn(n, PI, CORNER), [-1.0, 1.0, 1.0]) && !hidden(turn(n, PI, CORNER), end));
        for &c in &CORNERS {
            let (a, b) = (turn(n, PI, c), turn(ROD, PI, c));
            let gap = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            assert!((gap - 2.0).abs() < 1e-9, "{c:?}: {gap}");
            assert!(CORNERS.iter().any(|&q| close3(a, q)), "{c:?}");
        }
        // the other axis shows well beyond the solid at both ends of the spin
        for t in [0.0, 1.0] {
            let p = pose(OTHER_AXIS, 0.5, t);
            assert!(drawn(&unhidden_runs(&axis_points(n, OTHER_REACH), p)) > 1.4, "{t}");
        }
    }

    /// The least screen distance from `at` to the points of `runs`.
    fn nearest(at: Vec3, runs: &[Vec<Vec3>]) -> f64 {
        runs.iter()
            .flatten()
            .map(|q| (q[0] - at[0]).hypot(q[1] - at[1]))
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn the_red_spin_keeps_clear_of_both_rings() {
        let n = unit(OTHER_AXIS);
        let (start, end) = (project(CORNER), project(turn(ROD, PI, CORNER)));
        for j in 0..=60 {
            let t = f64::from(j) / 60.0;
            let p = pose(OTHER_AXIS, 0.5, t);
            let line = unhidden_runs(&axis_points(n, OTHER_REACH), p);
            let pin = unhidden_runs(&pin_points(p), p);
            let arc = unhidden_runs(&spin_path(n, 0.5, PIN_TIP, t), p);
            for runs in [&line, &pin, &arc] {
                assert!(nearest(start, runs) > START_RING_R + 0.3, "{t}");
                assert!(nearest(end, runs) > END_RING_R + 0.4, "{t}");
            }
            // the corner leaves the start ring, and its trace never nears the end ring
            let trace = unhidden_runs(&spin_path(n, 0.5, CORNER, t), p);
            assert!(nearest(end, &trace) > END_RING_R + 0.8, "{t}");
        }
        // the miss still shows: the red corner lands well outside the end ring
        let landed = project(turn(n, PI, CORNER));
        assert!((landed[0] - end[0]).hypot(landed[1] - end[1]) > 1.3);
    }

    #[test]
    fn the_veil_covers_every_pose_of_the_picture_and_none_of_the_words() {
        let inside = |q: Vec3| (q[0] - CUBE_C[0]).abs() < VEIL_HALF.0 && (q[1] - CUBE_C[1]).abs() < VEIL_HALF.1;
        for (axis, rate, reach) in [
            (ROD, 2.5, ROD_REACH),
            (OTHER_AXIS, 0.5, OTHER_REACH),
            (ROD, -0.5, ROD_REACH),
        ] {
            let n = unit(axis);
            for j in 0..=120 {
                let (a, angle) = pose(axis, rate, f64::from(j) / 120.0);
                for &c in &CORNERS {
                    assert!(inside(project(turn(a, angle, c))));
                }
                assert!(inside(project(turn(a, angle, PIN_TIP))));
            }
            assert!(inside(project([n[0] * reach, n[1] * reach, n[2] * reach])));
            assert!(inside(project([-n[0] * reach, -n[1] * reach, -n[2] * reach])));
        }
        // the corners' circles reach √2 from the rod, at the top and bottom faces
        for k in 0..72 {
            let a = TAU * f64::from(k) / 72.0;
            for h in [1.0, -1.0] {
                assert!(inside(project([SQRT_2 * a.cos(), h, SQRT_2 * a.sin()])));
            }
        }
        // the rod's name stands above it, and the words start right of it
        assert!(project([0.0, ROD_REACH + 0.3, 0.0])[1] - 0.12 > CUBE_C[1] + VEIL_HALF.1);
        const { assert!(CUBE_C[0] + VEIL_HALF.0 < 0.6) };
    }

    #[test]
    fn the_ruler_folds_inside_panel_b_and_closes_exactly_on_the_dial() {
        let (c, w, h) = (PANELS[1], HALF_W, HALF_H);
        for j in 0..=100 {
            let fold = f64::from(j) / 100.0;
            for k in 0..=72 {
                let q = folded_at(fold, 0.0, f64::from(k) / 72.0, 0.0);
                assert!((q[0] - c[0]).abs() < w && (q[1] - c[1]).abs() < h, "{fold} {q:?}");
            }
        }
        for k in 0..=40 {
            let laps = f64::from(k) / 40.0;
            // shut and turned, it is the dial; and folding starts from flat, with no jump
            let (shut, mark) = (folded_at(1.0, 1.0, laps, DIAL_TICK), dial_at(laps, DIAL_R - DIAL_TICK));
            assert!((shut[0] - mark[0]).abs() < 1e-12 && (shut[1] - mark[1]).abs() < 1e-12);
            let (a, b) = (folded_at(2e-4, 0.0, laps, 0.1), folded_at(0.0, 0.0, laps, 0.1));
            assert!((a[0] - b[0]).hypot(a[1] - b[1]) < 2e-3);
            for t in [0.25, 0.5, 0.75] {
                let q = folded_at(1.0, t, laps, 0.0);
                let r = (q[0] - DIAL_C[0]).hypot(q[1] - DIAL_C[1]);
                assert!((r - DIAL_R).abs() < 1e-12);
            }
        }
        // and the 10 lands on the 1: what the dial forgets
        let (one, ten) = (folded_at(1.0, 1.0, 0.0, 0.0), folded_at(1.0, 1.0, 1.0, 0.0));
        assert!((one[0] - ten[0]).abs() < 1e-12 && (one[1] - ten[1]).abs() < 1e-12);
    }

    #[test]
    fn a_motion_ends_before_the_next_sentence_starts() {
        let starts = [0.0, 3.0, 13.0, 21.0];
        assert_eq!(sentence_len(&starts, 1), Some(10.0));
        assert_eq!(sentence_len(&starts, 3), None);
        assert_eq!(sentence_len(&[], 0), None);
        // what comes before, the run and the margin fill the sentence exactly
        let run = room(&starts, 1, 0.4, 0.6, 3.0);
        assert!((0.4 + run + 0.6 - 10.0).abs() < 1e-12);
        // no sentence times: the fallback; hardly any room: still a visible motion
        assert!((room(&[], 1, 0.4, 0.6, 3.0) - 3.0).abs() < 1e-12);
        assert!((room(&[0.0, 0.5], 0, 0.4, 0.6, 3.0) - MIN_RUN).abs() < 1e-12);
        const { assert!(0.0 < AXIS_SPIN_END && AXIS_SPIN_END < AXIS_RESET) };
        const { assert!(AXIS_RESET < AXIS_CHIP && AXIS_CHIP < 1.0 && TIE_AT < 1.0) };
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
    fn every_ellipse_flow_leaves_one_place_parts_and_comes_home_there() {
        let specs = ellipses("2,1,0;1.6,0.7,30").unwrap();
        let near = |p: [f64; 2], q: [f64; 2]| (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9;
        let mut parted = Vec::new();
        for (a, b, tilt) in std::iter::once((1.0, 1.0, 0.0)).chain(specs) {
            assert!(near(orbit_point(a, b, tilt, 0.0), SHARED_START));
            assert!(near(orbit_point(a, b, tilt, 1.0), SHARED_START));
            // they meet once more, at the opposite point: each half-time flow is −I
            let opposite = [-SHARED_START[0], -SHARED_START[1]];
            assert!(near(orbit_point(a, b, tilt, 0.5), opposite));
            // and each stays on its own ellipse: P·R(2πt)·P⁻¹ applied to the start
            let p = model::mmul(model::rotation(tilt), [[a, 0.0], [0.0, b]]);
            let turned = model::mmul(model::mmul(p, model::rotation(TAU * 0.3)), model::inverse(p));
            let mid = orbit_point(a, b, tilt, 0.3);
            assert!(near(mid, model::apply(turned, SHARED_START)));
            parted.push(mid);
        }
        // the dots really are in different places between the meetings
        for (k, p) in parted.iter().enumerate() {
            for q in &parted[k + 1..] {
                assert!((p[0] - q[0]).hypot(p[1] - q[1]) > 0.3, "{p:?} {q:?}");
            }
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
        for (a, b, tilt) in [(1.0, 1.0, 0.0), (2.0, 1.0, 0.0), (1.6, 0.7, 30f64.to_radians())] {
            for j in 0..32 {
                let q = flows_screen(orbit_point(a, b, tilt, f64::from(j) / 32.0));
                assert!(inside(q, PANELS[3]));
                // clear of the words at x 3.4 and the caption at y −2.65
                assert!(q[0] < 3.2 && q[1] > -2.4, "{q:?}");
            }
        }
        // the two lines beside panel C's column stay inside its right border
        let border = PANELS[2][0] + HALF_W;
        for (s, size) in [
            ("a cut chooses the laps", 18.0),
            ("Log zw = Log z + Log w + 2πik", 18.0),
        ] {
            assert!(PLANE_WORDS_X + width(s, size) * WIDTH_ROOM < border, "{s}");
        }
        // the cube, the rod and the other axis stay left of the text column
        // at x 0.6 and between the title and the bottom line
        for (axis, rate, reach) in [(ROD, 2.5, ROD_REACH), (OTHER_AXIS, 0.5, OTHER_REACH)] {
            for j in 0..=40 {
                let (a, angle) = pose(axis, rate, f64::from(j) / 40.0);
                for &p in &CORNERS {
                    let q = project(turn(a, angle, p));
                    assert!(q[0] < -0.6 && q[1] > -3.0 && q[1] < 2.9, "{q:?}");
                }
            }
            let n = unit(axis);
            for k in [-reach, reach] {
                let q = project([n[0] * k, n[1] * k, n[2] * k]);
                assert!(q[0] < -0.6 && q[1] > -3.0 && q[1] < 2.9, "{q:?}");
            }
        }
        let rod_word_y = project([0.0, ROD_REACH + 0.3, 0.0])[1];
        assert!(rod_word_y < 3.0);
        // the pin's whole sweep stays on the picture, left of the text column
        // and below the rod's name
        for &p in &spin_path(unit(OTHER_AXIS), 0.5, PIN_TIP, 1.0) {
            let q = project(p);
            assert!(
                q[0] > -6.8 && q[0] < -0.6 && q[1] > -3.0 && q[1] < rod_word_y - 0.18,
                "{q:?}"
            );
        }
        // the turning line and its labels fit between x 1.4 and the edge, with the
        // family's dots past each end
        assert!(turning_at(-3.4 * PI)[0] > 1.4 && turning_at(3.4 * PI)[0] < 6.6);
        let dots = 0.11;
        assert!(turning_at(-3.75 * PI)[0] - dots > 1.4 && turning_at(3.75 * PI)[0] + dots < TEXT_RIGHT);
        assert!(turning_at(-3.75 * PI)[0] + dots < turning_at(-3.4 * PI)[0]);
        // and the words stack clear of it and of the fine print
        const { assert!(-0.9 - 0.11 > TURNING_Y + 0.15 && TURNING_Y - 0.32 - 0.09 > -2.1 + 0.11) };
        const { assert!(FAMILY_LABEL_Y - 0.11 > FOOTER_Y + 0.13 + 0.15) };
        // panel A: the ruler's hops under the frame, its keys inside it under the caption
        let tallest = (ruler_x(2.0) - ruler_x(1.0)) / 2.0;
        assert!(HOP_Y + tallest < PANELS[0][1] + HALF_H);
        const { assert!(RULER_ONE[1] + RULER_LABEL + 0.08 < HOP_Y) };
        const { assert!(KEYS_TOP - KEYS_H > PANELS[0][1] - HALF_H && KEYS_TOP < 0.95 - 0.1) };
        const { assert!(KEYS_RIGHT < PANELS[0][0] + HALF_W) };
        assert!(-6.55 + width("one input for every output", 20.0) * WIDTH_ROOM < KEYS_LEFT - 0.1);
        // the caption over the keys stays inside the panel
        assert!(KEYS_LEFT + width("×2 is always the same step", 20.0) * WIDTH_ROOM < PANELS[0][0] + HALF_W);
        // panel C: the lowest label and the axis's foot clear the sentence, the top the frame
        let sentence_top = -2.65 + 0.11;
        assert!(column_at(-2)[1] - 0.09 > sentence_top + 0.15);
        const { assert!(PLANE_C[1] - PLANE_AXIS > -2.65 + 0.11 + 0.15) };
        const { assert!(PLANE_C[1] + PLANE_AXIS < PANELS[2][1] + HALF_H) };
        assert!(column_at(1)[1] + 0.09 < PANELS[2][1] + HALF_H);
        for (s, size, x) in [
            ("each corner keeps to a circle about the rod", 20.0, 0.6),
            ("spin: ½ lap the other way per unit of time", 22.0, 0.6),
            ("every corner halfway round: a half turn about the rod", 22.0, 0.6),
            ("half turn after one unit of time?", 26.0, 0.6),
            ("axis: only the rod; another axis drags the top off it", 22.0, 0.6),
            ("so: infinitely many spins, all about the rod", 24.0, 0.6),
            ("how far each of these spins turns in one unit of time:", 20.0, 1.4),
            ("the slowest two, ±π, tie on the window's open edge", 20.0, 1.4),
            ("for the number −1, Log(−1) = iπ by convention", 20.0, 1.4),
            ("here: the whole family", 20.0, 1.4),
        ] {
            assert!(x + width(s, size) * WIDTH_ROOM < TEXT_RIGHT, "{s}");
        }
        let fine = "precisely: spins ending here: ±½, ±1½, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)";
        assert!(width(fine, 22.0) * WIDTH_ROOM < 2.0 * TEXT_RIGHT);
        // the panels clear the title and the frame edge
        assert!(PANELS[0][1] + HALF_H < 3.0 && PANELS[2][1] - HALF_H > -3.4);
        assert!(PANELS[1][0] + HALF_W < 7.0);
    }

    /// The film ends on the finished card: the last line runs out, the card is held for
    /// reading, and only then does everything fade. Nothing is said after the last line.
    #[test]
    fn the_closing_card_is_held_after_the_last_line_and_then_fades() {
        let film = include_str!("what_it_forgets.rs").split("#[cfg(test)]").next().unwrap();
        let order = [
            r#"kit.say(stage, "12_21")?;"#,
            "fade_in_all(stage, &family, 0.4)?;",
            "kit.music_breath(stage, CLOSING_HOLD)?;",
            "fade_all(stage, CLOSING_FADE)?;",
            "kit.finish(stage)",
        ];
        let at = order.map(|step| {
            assert_eq!(film.matches(step).count(), 1, "{step}");
            film.find(step).unwrap()
        });
        assert!(at.windows(2).all(|w| w[0] < w[1]));
        assert!(!film[at[0] + order[0].len()..].contains("kit.say("));
    }

    #[test]
    fn the_closing_words_are_in_ink_and_clear_of_the_edge_and_each_other() {
        let film = include_str!("what_it_forgets.rs").split("#[cfg(test)]").next().unwrap();
        assert!(film.contains("pi_label(k),\n                20.0,\n                Palette::ink(),"));
        assert!(film.contains("\"…\", 20.0, Palette::ink(), turning_at"));
        assert!(film.contains("by convention\",\n            20.0,\n            Palette::ink(),"));
        assert_eq!(film.matches(" = footer(").count(), 1);
        assert!(!film.contains(" = precisely("));
        assert!(film.contains("Palette::ink(),\n        v(0.0, FOOTER_Y),"));
        super::super::draw::probe_scene(|stage| {
            let kit = Kit::new()?;
            let m = footer(
                &kit,
                stage,
                "spins ending here: ±½, ±1½, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)",
            )?;
            // off the frame's edge, inside the text limit, and clear of the family's row
            assert!(stage.get_bounding_box_point(m, DOWN)[1] > -3.6);
            assert!(stage.get_bounding_box_point(m, UP)[1] < FAMILY_LABEL_Y - 0.1 - 0.2);
            assert!(stage.get_bounding_box_point(m, RIGHT)[0] < TEXT_RIGHT);
            // the rows take the room above: the red rings and the tie row bracket the labels
            let mut reach = f64::NEG_INFINITY;
            for k in -3..=3_i64 {
                let at = turning_at(PI * k as f64);
                let label = super::super::draw::pi_label(k);
                let m = kit.text(stage, &label, 20.0, Palette::ink(), v(at[0], at[1] - 0.32))?;
                assert!(
                    stage.get_bounding_box_point(m, UP)[1] < TURNING_Y - 0.15 - 0.05,
                    "{label}"
                );
                assert!(stage.get_bounding_box_point(m, DOWN)[1] > -2.1 + 0.1 + 0.15, "{label}");
                // and side by side, under marks 0.62 apart, they do not touch
                assert!(stage.get_bounding_box_point(m, LEFT)[0] > reach + 0.15, "{label}");
                reach = stage.get_bounding_box_point(m, RIGHT)[0];
            }
            Ok(())
        });
    }

    #[test]
    fn a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence() {
        let text = "One two. Three four five six. Seven.";
        let starts = [0.2, 2.0, 6.0];
        let none = HashMap::new();
        // eleven of the sentence's twenty characters come before "five"
        let t = phrase_seconds(text, "five", &none, &starts, 8.0).unwrap();
        assert!((t - (2.0 + 4.0 * 11.0 / 20.0)).abs() < 1e-12);
        // a measured word mark wins
        let marks = HashMap::from([(text.find("five").unwrap(), 3.25)]);
        let t = phrase_seconds(text, "five", &marks, &starts, 8.0).unwrap();
        assert!((t - 3.25).abs() < 1e-12);
        // never a guess
        assert!(phrase_seconds(text, "nine", &none, &starts, 8.0).is_err());
        assert!(phrase_seconds(text, "five", &none, &starts[..2], 8.0).is_err());
    }

    #[test]
    fn the_lap_hooks_are_in_the_second_sentence_once_and_in_spoken_order() {
        let script = Script::parse(SCRIPT).unwrap();
        let text = script.text("12_6").unwrap();
        for phrase in [ALSO_ENDS_HERE, AND_TWO_AND_A_HALF] {
            assert_eq!(text.matches(phrase).count(), 1, "{phrase}");
        }
        let second = text.find("Laps first:").unwrap();
        let (a, b) = (
            text.find(ALSO_ENDS_HERE).unwrap(),
            text.find(AND_TWO_AND_A_HALF).unwrap(),
        );
        assert!(second < a && a < b);
        // the line ends with that sentence: the direction has its own line
        assert!(text.ends_with("any odd number of half laps."));
    }

    #[test]
    fn the_answer_is_three_lines_within_the_narrations_cap() {
        let script = Script::parse(SCRIPT).unwrap();
        let parts = ["12_6", "12_20", "12_21"].map(|id| script.text(id).unwrap());
        assert!(parts[0].starts_with("Here is the answer, one hint at a time."));
        assert!(parts[1].starts_with("Direction next:"));
        assert!(parts[1].ends_with("all about this one rod."));
        assert!(parts[2].starts_with("The two slowest,"));
        assert!(parts[2].ends_with("the whole family."));
        for part in parts {
            assert!(part.split_whitespace().count() <= 100, "{part}");
        }
    }

    #[test]
    fn the_new_hooks_are_in_their_lines_once_and_in_spoken_order() {
        let script = Script::parse(SCRIPT).unwrap();
        let line = script.text("12_1").unwrap();
        assert_eq!(line.matches(THE_KEYBOARD).count(), 1);
        let text = script.text("12_21").unwrap();
        let last = text.find("For the number minus one").unwrap();
        for phrase in [CONVENTION, WHOLE_FAMILY] {
            assert_eq!(text.matches(phrase).count(), 1, "{phrase}");
        }
        let (a, b) = (text.find(CONVENTION).unwrap(), text.find(WHOLE_FAMILY).unwrap());
        assert!(last < a && a < b);
    }

    #[test]
    fn a_spin_ends_by_its_moment_within_its_limits() {
        assert!((run_by(Some(12.0), 10.0, 3.0) - 2.0).abs() < 1e-12);
        assert!((run_by(Some(20.0), 10.0, 3.0) - 3.0).abs() < 1e-12);
        assert!((run_by(Some(9.0), 10.0, 3.0) - MIN_RUN).abs() < 1e-12);
        assert!((run_by(None, 10.0, 3.0) - 3.0).abs() < 1e-12);
    }

    #[test]
    fn the_one_mark_is_on_the_rim_labelled_and_clear_of_its_neighbours() {
        let mark = fact_f64("dial.mark").unwrap();
        let label = format!("{:.1}", 10f64.powf(mark));
        // the printed label is the value the caption lists, on the unit decade
        assert_eq!(Some(label.as_str()), fact("dial.values").unwrap().split(',').nth(1));
        let at = dial_at(mark, DIAL_R + 0.2);
        for digit in [2.0f64, 3.0] {
            let p = dial_at(digit.log10(), DIAL_R + 0.2);
            assert!((at[0] - p[0]).hypot(at[1] - p[1]) > 0.4, "{digit}");
        }
        // clear of the caption at x 3.6, inside panel B
        assert!(at[0] + width(&label, 14.0) * WIDTH_ROOM / 2.0 < 3.6 - 0.1);
        assert!(at[1] - 0.1 > PANELS[1][1] - HALF_H);
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
