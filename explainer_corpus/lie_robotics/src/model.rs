//! The mathematics of Video G, in closed forms: turns by Rodrigues' formula,
//! poses and twists with the screw exponential and logarithm of Lynch and
//! Park (*Modern Robotics*, 2017: eqs. 3.51, 3.88, 3.91 and 3.92), the arm
//! as a product of exponentials (eq. 4.14) and its space Jacobian (eq.
//! 5.11). Twists are six numbers, spin first and slide second, as in that
//! book. `oracle.py` reaches every number another way (the URDF chain, a
//! generic series exponential and logarithm, finite differences).
//!
//! The arm is the KUKA LBR iiwa 14 R820. Its joint origins are the ones the
//! owner's cmaes_explainer vendors (`vendor/iiwa14.xacro`, which is
//! aica-technology/kuka-iiwa-description v0.1.0, GPL-3.0: cited, never
//! copied); see docs/fact-sheet.md.

use std::f64::consts::PI;

pub type V3 = [f64; 3];
pub type M3 = [[f64; 3]; 3];
/// A twist: spin (rad/s) first, slide (m/s) second.
pub type V6 = [f64; 6];
/// Joint angles, radians.
pub type Q = [f64; N];
/// A Jacobian: six rows, one column per joint.
pub type Jac = [[f64; N]; 6];

pub const N: usize = 7;
pub const DEG: f64 = PI / 180.0;
pub const I3: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
pub const X: V3 = [1.0, 0.0, 0.0];
pub const Y: V3 = [0.0, 1.0, 0.0];
pub const Z: V3 = [0.0, 0.0, 1.0];

// ---------------------------------------------------------------- vectors

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn norm(a: V3) -> f64 {
    dot(a, a).sqrt()
}

pub fn unit(a: V3) -> V3 {
    scale(a, 1.0 / norm(a))
}

// ---------------------------------------------------------------- 3 by 3

pub fn mul(a: &M3, b: &M3) -> M3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    out
}

pub fn transpose(a: &M3) -> M3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[j][i];
        }
    }
    out
}

pub fn apply(a: &M3, v: V3) -> V3 {
    [dot(a[0], v), dot(a[1], v), dot(a[2], v)]
}

pub fn det(a: &M3) -> f64 {
    dot(a[0], cross(a[1], a[2]))
}

/// The spin matrix of an arrow: `hat(w) v = w × v`.
pub fn hat(w: V3) -> M3 {
    [[0.0, -w[2], w[1]], [w[2], 0.0, -w[0]], [-w[1], w[0], 0.0]]
}

fn lin(a: &M3, sa: f64, b: &M3, sb: f64) -> M3 {
    let mut out = [[0.0; 3]; 3];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = sa * a[i][j] + sb * b[i][j];
        }
    }
    out
}

/// The largest difference between two matrices' entries.
pub fn gap(a: &M3, b: &M3) -> f64 {
    (0..3)
        .flat_map(|i| (0..3).map(move |j| (i, j)))
        .map(|(i, j)| (a[i][j] - b[i][j]).abs())
        .fold(0.0, f64::max)
}

/// A turn of `angle` radians about the unit `axis`: Rodrigues' formula,
/// I + sin(a) K + (1 − cos(a)) K².
pub fn turn(axis: V3, angle: f64) -> M3 {
    let k = hat(axis);
    let k2 = mul(&k, &k);
    lin(&lin(&I3, 1.0, &k, angle.sin()), 1.0, &k2, 1.0 - angle.cos())
}

/// The angle (radians, 0 to π) and unit axis of a turn. An exact half turn
/// has two axes and no turn has none: the axis is then zero.
pub fn angle_axis(r: &M3) -> (f64, V3) {
    let v = [r[2][1] - r[1][2], r[0][2] - r[2][0], r[1][0] - r[0][1]];
    let s = norm(v);
    let angle = s.atan2(r[0][0] + r[1][1] + r[2][2] - 1.0);
    if s < 1e-12 {
        (angle, [0.0; 3])
    } else {
        (angle, scale(v, 1.0 / s))
    }
}

// ---------------------------------------------------------------- poses

/// A pose: an orientation and a position (a 4 by 4 matrix's two blocks).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub r: M3,
    pub p: V3,
}

impl Pose {
    pub const NOTHING: Pose = Pose { r: I3, p: [0.0; 3] };

    /// `self`, then `other` read in `self`'s frame (the matrix product).
    pub fn then(&self, other: &Pose) -> Pose {
        Pose {
            r: mul(&self.r, &other.r),
            p: add(apply(&self.r, other.p), self.p),
        }
    }

    pub fn undo(&self) -> Pose {
        let rt = transpose(&self.r);
        Pose {
            r: rt,
            p: scale(apply(&rt, self.p), -1.0),
        }
    }

    pub fn at(&self, point: V3) -> V3 {
        add(apply(&self.r, point), self.p)
    }

    pub fn gap(&self, other: &Pose) -> f64 {
        gap(&self.r, &other.r).max((0..3).map(|i| (self.p[i] - other.p[i]).abs()).fold(0.0, f64::max))
    }
}

pub fn spin_of(t: V6) -> V3 {
    [t[0], t[1], t[2]]
}

pub fn slide_of(t: V6) -> V3 {
    [t[3], t[4], t[5]]
}

pub fn twist(spin: V3, slide: V3) -> V6 {
    [spin[0], spin[1], spin[2], slide[0], slide[1], slide[2]]
}

/// A twist held for one second (Lynch and Park eq. 3.88).
pub fn exp6(t: V6) -> Pose {
    let (w, v) = (spin_of(t), slide_of(t));
    let angle = norm(w);
    if angle < 1e-12 {
        return Pose { r: I3, p: v };
    }
    let axis = scale(w, 1.0 / angle);
    let v = scale(v, 1.0 / angle);
    let k = hat(axis);
    let k2 = mul(&k, &k);
    let g = lin(&lin(&I3, angle, &k, 1.0 - angle.cos()), 1.0, &k2, angle - angle.sin());
    Pose {
        r: turn(axis, angle),
        p: apply(&g, v),
    }
}

/// The twist that reaches `t` in one second (eqs. 3.91 and 3.92). Not for
/// an exact half turn.
pub fn log6(t: &Pose) -> V6 {
    let (angle, axis) = angle_axis(&t.r);
    if angle < 1e-12 {
        return twist([0.0; 3], t.p);
    }
    let k = hat(axis);
    let k2 = mul(&k, &k);
    let g_inv = lin(
        &lin(&I3, 1.0 / angle, &k, -0.5),
        1.0,
        &k2,
        1.0 / angle - 0.5 / (angle / 2.0).tan(),
    );
    twist(scale(axis, angle), scale(apply(&g_inv, t.p), angle))
}

/// How a twist reads in another frame (the adjoint of a pose).
pub fn carry(t: &Pose, v: V6) -> V6 {
    let w = apply(&t.r, spin_of(v));
    twist(w, add(cross(t.p, w), apply(&t.r, slide_of(v))))
}

/// A pose as one screw motion: the turn, its axis, a point on the axis and
/// the slide along it. With no turn the screw is a pure slide along `axis`.
#[derive(Clone, Copy, Debug)]
pub struct Screw {
    pub angle: f64,
    pub axis: V3,
    pub point: V3,
    pub slide: f64,
}

pub fn screw_of(t: &Pose) -> Screw {
    let v6 = log6(t);
    let angle = norm(spin_of(v6));
    if angle < 1e-12 {
        let d = norm(t.p);
        return Screw {
            angle: 0.0,
            axis: if d > 0.0 { scale(t.p, 1.0 / d) } else { [0.0; 3] },
            point: [0.0; 3],
            slide: d,
        };
    }
    let w = scale(spin_of(v6), 1.0 / angle);
    let v = scale(slide_of(v6), 1.0 / angle);
    Screw {
        angle,
        axis: w,
        point: cross(w, v),
        slide: dot(w, v) * angle,
    }
}

/// The pose a screw makes, run for the fraction `f` of the whole motion.
pub fn along_screw(s: &Screw, f: f64) -> Pose {
    if s.angle == 0.0 {
        return Pose {
            r: I3,
            p: scale(s.axis, s.slide * f),
        };
    }
    let r = turn(s.axis, s.angle * f);
    Pose {
        r,
        p: add(sub(s.point, apply(&r, s.point)), scale(s.axis, s.slide * f)),
    }
}

// ---------------------------------------------------------------- three angles

/// Yaw about z (the outer ring), pitch about y (the middle ring), roll about
/// x (the inner ring), in radians.
pub fn rings(yaw: f64, pitch: f64, roll: f64) -> M3 {
    mul(&mul(&turn(Z, yaw), &turn(Y, pitch)), &turn(X, roll))
}

/// How much of the third direction the three ring axes still span.
pub fn rings_left(pitch: f64) -> f64 {
    pitch.cos().abs()
}

/// The volume three rings' axes span, for any order of rings: the outer
/// about `a`, the middle about `b` turned by `middle`, the inner about `c`.
pub fn rings_span(a: V3, b: V3, c: V3, middle: f64) -> f64 {
    let inner = apply(&turn(b, middle), c);
    det(&[[a[0], b[0], inner[0]], [a[1], b[1], inner[1]], [a[2], b[2], inner[2]]]).abs()
}

/// How far the nearest turn to a blend has turned: two turns that differ by
/// `angle` about one axis, mixed with weights `1 − s` and `s`.
pub fn blend_turned(angle: f64, s: f64) -> f64 {
    (s * angle.sin()).atan2(1.0 - s + s * angle.cos())
}

/// An angle in degrees brought into (−180, 180].
pub fn wrap_deg(a: f64) -> f64 {
    let w = (a + 180.0).rem_euclid(360.0) - 180.0;
    if w == -180.0 { 180.0 } else { w }
}

/// Halfway between two headings, along the shorter way round.
pub fn halfway_deg(a: f64, b: f64) -> f64 {
    wrap_deg(a + wrap_deg(b - a) / 2.0)
}

// ---------------------------------------------------------------- the floor

/// Where a base ends after holding a body velocity (forward, sideways,
/// spin) for `t` seconds: (x, y, heading).
pub fn planar_exp(vx: f64, vy: f64, spin: f64, t: f64) -> (f64, f64, f64) {
    if spin == 0.0 {
        return (vx * t, vy * t, 0.0);
    }
    let a = spin * t;
    (
        (vx * a.sin() - vy * (1.0 - a.cos())) / spin,
        (vx * (1.0 - a.cos()) + vy * a.sin()) / spin,
        a,
    )
}

/// The fixed point a spinning base circles.
pub fn planar_centre(vx: f64, vy: f64, spin: f64) -> (f64, f64) {
    (-vy / spin, vx / spin)
}

/// Mecanum wheel speeds, front left, front right, rear left, rear right
/// (Lynch and Park eq. 13.10, whose wheels are numbered round the chassis:
/// these are its wheels 1, 2, 4 and 3): `a` and `b` are the half length
/// and half track between wheel centres, `r` the wheel radius.
pub fn wheel_speeds(vx: f64, vy: f64, spin: f64, a: f64, b: f64, r: f64) -> [f64; 4] {
    let k = (a + b) * spin;
    [
        (vx - vy - k) / r,
        (vx + vy + k) / r,
        (vx + vy - k) / r,
        (vx - vy + k) / r,
    ]
}

// ---------------------------------------------------------------- the arm

/// Joint origins in the parent link's frame: metres, then roll, pitch, yaw
/// (`vendor/iiwa14.xacro` lines 48, 81, 114, 147, 180, 213 and 246). Every
/// joint turns about its own +z.
pub const JOINT_ORIGINS: [(V3, V3); N] = [
    ([0.0, 0.0, 0.1575], [0.0, 0.0, 0.0]),
    ([0.0, 0.0, 0.2025], [PI / 2.0, 0.0, PI]),
    ([0.0, 0.2045, 0.0], [PI / 2.0, 0.0, PI]),
    ([0.0, 0.0, 0.2155], [PI / 2.0, 0.0, 0.0]),
    ([0.0, 0.1845, 0.0], [-PI / 2.0, PI, 0.0]),
    ([0.0, 0.0, 0.2155], [PI / 2.0, 0.0, 0.0]),
    ([0.0, 0.081, 0.0], [-PI / 2.0, PI, 0.0]),
];
/// The flange beyond joint 7 (line 279).
pub const FLANGE: V3 = [0.0, 0.0, 0.045];
/// KUKA's data sheet for the LBR iiwa 14 R820: motion range and speed with
/// rated payload, axes A1 to A7 (degrees, degrees per second).
pub const LIMITS_DEG: [f64; N] = [170.0, 120.0, 170.0, 120.0, 170.0, 120.0, 175.0];
pub const SPEED_DEG_S: [f64; N] = [85.0, 85.0, 100.0, 75.0, 130.0, 135.0, 135.0];

/// Joints by what a person would call them.
pub const SHOULDER: usize = 1;
pub const ELBOW: usize = 3;
pub const WRIST: usize = 5;

/// The arm with every joint at zero: where each joint's axis is, and the
/// tip's pose. Everything else is built from these.
#[derive(Clone, Debug)]
pub struct Arm {
    /// A point on each joint's axis, at home.
    pub origin: [V3; N],
    /// Each joint's axis, at home.
    pub axis: [V3; N],
    /// The tip's pose at home (the M of the product of exponentials).
    pub home: Pose,
}

impl Arm {
    /// Walk the joint origins once, at zero.
    pub fn iiwa14() -> Arm {
        let mut t = Pose::NOTHING;
        let mut origin = [[0.0; 3]; N];
        let mut axis = [[0.0; 3]; N];
        for (i, (xyz, [roll, pitch, yaw])) in JOINT_ORIGINS.iter().enumerate() {
            t = t.then(&Pose {
                r: rings(*yaw, *pitch, *roll),
                p: *xyz,
            });
            origin[i] = t.p;
            axis[i] = apply(&t.r, Z);
        }
        Arm {
            origin,
            axis,
            home: t.then(&Pose { r: I3, p: FLANGE }),
        }
    }

    /// Joint `i`'s screw: a turn about its axis with no slide.
    pub fn screw(&self, i: usize) -> V6 {
        twist(self.axis[i], scale(cross(self.axis[i], self.origin[i]), -1.0))
    }

    /// The motion the first `count` joints give everything beyond them.
    pub fn carried(&self, q: &Q, count: usize) -> Pose {
        (0..count).fold(Pose::NOTHING, |t, i| t.then(&exp6(self.screw(i).map(|x| x * q[i]))))
    }

    /// The tip's pose: the product of exponentials, then home.
    pub fn tip(&self, q: &Q) -> Pose {
        self.carried(q, N).then(&self.home)
    }

    /// Where joint `i`'s axis is now: a point on it and its direction.
    pub fn joint(&self, q: &Q, i: usize) -> (V3, V3) {
        let t = self.carried(q, i);
        (t.at(self.origin[i]), apply(&t.r, self.axis[i]))
    }

    /// The points to draw the arm through: the base, each joint, the tip.
    pub fn skeleton(&self, q: &Q) -> [V3; N + 2] {
        let mut out = [[0.0; 3]; N + 2];
        for i in 0..N {
            out[i + 1] = self.joint(q, i).0;
        }
        out[N + 1] = self.tip(q).p;
        out
    }

    /// The space Jacobian: column `i` is joint `i`'s screw where it is now.
    pub fn jacobian(&self, q: &Q) -> Jac {
        let mut j = [[0.0; N]; 6];
        for i in 0..N {
            let col = carry(&self.carried(q, i), self.screw(i));
            for (row, x) in j.iter_mut().zip(col) {
                row[i] = x;
            }
        }
        j
    }

    /// The Jacobian read in the tip's own frame.
    pub fn body_jacobian(&self, q: &Q) -> Jac {
        let back = self.tip(q).undo();
        let js = self.jacobian(q);
        let mut j = [[0.0; N]; 6];
        for i in 0..N {
            let col = carry(&back, column(&js, i));
            for (row, x) in j.iter_mut().zip(col) {
                row[i] = x;
            }
        }
        j
    }

    /// How far the tip is from joint `i`'s axis: the tip's speed in metres
    /// per second when that joint alone turns at one radian per second.
    pub fn lever(&self, q: &Q, i: usize) -> f64 {
        let col = column(&self.jacobian(q), i);
        norm(add(slide_of(col), cross(spin_of(col), self.tip(q).p)))
    }

    /// The most any one joint, turning at one radian a second, moves the
    /// wrist along the line from shoulder to wrist (m/s). Zero means the
    /// gripper cannot slide along the arm, out or back.
    pub fn push_along_arm(&self, q: &Q) -> f64 {
        let (s, w) = (self.joint(q, SHOULDER).0, self.joint(q, WRIST).0);
        let along = unit(sub(w, s));
        (0..WRIST)
            .map(|i| {
                let (point, axis) = self.joint(q, i);
                dot(cross(axis, sub(w, point)), along).abs()
            })
            .fold(0.0, f64::max)
    }

    /// The radius of the circle the elbow can swing on while the tip holds
    /// still: its distance from the line from shoulder to wrist.
    pub fn elbow_radius(&self, q: &Q) -> f64 {
        let (s, e, w) = (
            self.joint(q, SHOULDER).0,
            self.joint(q, ELBOW).0,
            self.joint(q, WRIST).0,
        );
        let line = unit(sub(w, s));
        let r = sub(e, s);
        norm(sub(r, scale(line, dot(r, line))))
    }

    /// The elbow's speed, degrees per second, for the tip to pull straight
    /// back toward the shoulder at `pull` m/s without turning, with the
    /// elbow bent `bend` degrees from straight.
    pub fn elbow_speed_to_pull(&self, bend: f64, pull: f64) -> f64 {
        let q = degrees([0.0, 90.0, 0.0, bend, 0.0, 90.0, 0.0]);
        let (s, w) = (self.joint(&q, SHOULDER).0, self.joint(&q, WRIST).0);
        let want = twist([0.0; 3], scale(unit(sub(w, s)), -pull));
        least_speeds(&self.jacobian(&q), want)[ELBOW].abs() / DEG
    }

    /// One Newton step toward `goal`: the error as a twist in the tip's own
    /// frame, and the joint angles after stepping with the Jacobian.
    pub fn step_toward(&self, q: &Q, goal: &Pose) -> (V6, Q) {
        let error = log6(&self.tip(q).undo().then(goal));
        let dq = least_speeds(&self.body_jacobian(q), error);
        let mut next = *q;
        for (x, d) in next.iter_mut().zip(dq) {
            *x += d;
        }
        (error, next)
    }

    /// Newton's method on the group (Lynch and Park §6.2.2), stopping when
    /// the error twist's spin is under `tol_spin` radians and its slide
    /// under `tol_slide` metres.
    pub fn solve(&self, start: &Q, goal: &Pose, tol_spin: f64, tol_slide: f64) -> Solve {
        let mut q = *start;
        let mut run = Solve {
            q,
            dist_mm: Vec::new(),
            angle_deg: Vec::new(),
            path: vec![q],
        };
        for _ in 0..50 {
            let (error, next) = self.step_toward(&q, goal);
            run.dist_mm.push(norm(sub(self.tip(&q).p, goal.p)) * 1000.0);
            run.angle_deg.push(norm(spin_of(error)) / DEG);
            if norm(spin_of(error)) < tol_spin && norm(slide_of(error)) < tol_slide {
                break;
            }
            q = next;
            run.path.push(q);
        }
        run.q = q;
        run
    }

    /// Joint angles along the motion that swings the elbow while the tip
    /// holds its pose: small steps along the joint speeds that add up to no
    /// twist, each corrected back onto the pose, until a joint limit.
    pub fn swing(&self, start: &Q, step: f64, direction: f64, max_steps: usize) -> Vec<Q> {
        let hold = self.tip(start);
        let mut q = *start;
        let mut out = vec![q];
        let mut last: Option<Q> = None;
        for _ in 0..max_steps {
            let mut n = still_speeds(&self.jacobian(&q));
            let facing = match last {
                Some(prev) => n.iter().zip(prev).map(|(a, b)| a * b).sum::<f64>(),
                None => direction * n[2],
            };
            if facing < 0.0 {
                n = n.map(|x| -x);
            }
            last = Some(n);
            for (x, d) in q.iter_mut().zip(n) {
                *x += step * d;
            }
            for _ in 0..3 {
                q = self.step_toward(&q, &hold).1;
            }
            if q.iter().zip(LIMITS_DEG).any(|(x, lim)| x.abs() / DEG > lim) {
                break;
            }
            out.push(q);
        }
        out
    }
}

/// A run of Newton's method: the tip's distance from the goal (mm) and the
/// error twist's angle (degrees) before each step, and where it stopped.
#[derive(Clone, Debug)]
pub struct Solve {
    pub q: Q,
    pub dist_mm: Vec<f64>,
    pub angle_deg: Vec<f64>,
    /// The joint angles before each step, and the last ones.
    pub path: Vec<Q>,
}

impl Solve {
    pub fn steps(&self) -> usize {
        self.dist_mm.len() - 1
    }
}

pub fn degrees(q: [f64; N]) -> Q {
    q.map(|x| x * DEG)
}

pub fn column(j: &Jac, i: usize) -> V6 {
    [j[0][i], j[1][i], j[2][i], j[3][i], j[4][i], j[5][i]]
}

/// The Jacobian's columns made perpendicular one by one; a column that
/// brings nothing new is dropped.
fn independent(j: &Jac) -> Vec<V6> {
    let mut basis: Vec<V6> = Vec::new();
    for i in 0..N {
        let mut c = column(j, i);
        for _ in 0..2 {
            for b in &basis {
                let d: f64 = c.iter().zip(b).map(|(x, y)| x * y).sum();
                for (x, y) in c.iter_mut().zip(b) {
                    *x -= d * y;
                }
            }
        }
        let len = c.iter().map(|x| x * x).sum::<f64>().sqrt();
        if len > 1e-6 {
            basis.push(c.map(|x| x / len));
        }
    }
    basis
}

/// How many independent ways the tip can move.
pub fn rank(j: &Jac) -> usize {
    independent(j).len()
}

/// Solve a square system by elimination with row pivoting.
fn solve6(mut a: [[f64; 6]; 6], mut b: V6) -> V6 {
    for c in 0..6 {
        let pivot = (c..6)
            .max_by(|&i, &k| a[i][c].abs().total_cmp(&a[k][c].abs()))
            .unwrap_or(c);
        a.swap(c, pivot);
        b.swap(c, pivot);
        for r in c + 1..6 {
            let f = a[r][c] / a[c][c];
            for k in c..6 {
                a[r][k] -= f * a[c][k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = [0.0; 6];
    for r in (0..6).rev() {
        let s: f64 = (r + 1..6).map(|k| a[r][k] * x[k]).sum();
        x[r] = (b[r] - s) / a[r][r];
    }
    x
}

/// The smallest joint speeds that produce the twist `want` (the Jacobian
/// must reach all six directions): Jᵀ (J Jᵀ)⁻¹ want.
pub fn least_speeds(j: &Jac, want: V6) -> Q {
    let mut jjt = [[0.0; 6]; 6];
    for (r, row) in jjt.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (0..N).map(|k| j[r][k] * j[c][k]).sum();
        }
    }
    let y = solve6(jjt, want);
    let mut out = [0.0; N];
    for (i, x) in out.iter_mut().enumerate() {
        *x = (0..6).map(|r| j[r][i] * y[r]).sum();
    }
    out
}

fn det6(mut a: [[f64; 6]; 6]) -> f64 {
    let mut d = 1.0;
    for c in 0..6 {
        let pivot = (c..6)
            .max_by(|&i, &k| a[i][c].abs().total_cmp(&a[k][c].abs()))
            .unwrap_or(c);
        if a[pivot][c] == 0.0 {
            return 0.0;
        }
        if pivot != c {
            a.swap(c, pivot);
            d = -d;
        }
        d *= a[c][c];
        for r in c + 1..6 {
            let f = a[r][c] / a[c][c];
            for k in c..6 {
                a[r][k] -= f * a[c][k];
            }
        }
    }
    d
}

/// Unit joint speeds that add up to no twist at all (one spare joint): entry
/// `i` is, up to sign, the determinant of the Jacobian without column `i`.
pub fn still_speeds(j: &Jac) -> Q {
    let mut n = [0.0; N];
    for (i, x) in n.iter_mut().enumerate() {
        let mut minor = [[0.0; 6]; 6];
        for (r, row) in minor.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                *cell = j[r][if c < i { c } else { c + 1 }];
            }
        }
        *x = if i % 2 == 0 { det6(minor) } else { -det6(minor) };
    }
    let len = n.iter().map(|x| x * x).sum::<f64>().sqrt();
    n.map(|x| x / len)
}

/// The twist a set of joint speeds produces.
pub fn twist_from(j: &Jac, speeds: &Q) -> V6 {
    let mut out = [0.0; 6];
    for (r, x) in out.iter_mut().enumerate() {
        *x = (0..N).map(|i| j[r][i] * speeds[i]).sum();
    }
    out
}

// ---------------------------------------------------------------- the story's examples

/// The named poses the video uses, joint angles in degrees.
pub mod poses {
    use super::N;
    /// Straight up.
    pub const HOME: [f64; N] = [0.0; N];
    /// Shoulder and elbow a quarter turn each: the L shape.
    pub const ELL: [f64; N] = [0.0, 90.0, 0.0, 90.0, 0.0, 0.0, 0.0];
    /// Only the elbow bent, the arm otherwise straight up.
    pub const ELBOW_ONLY: [f64; N] = [0.0, 0.0, 0.0, 90.0, 0.0, 0.0, 0.0];
    /// The L shape with joint 5 a quarter turn round and the wrist straight:
    /// here the wrist's two aligned axes do cost a direction.
    pub const ELL_WRIST_TURNED: [f64; N] = [0.0, 90.0, 0.0, 90.0, 90.0, 0.0, 0.0];
    /// The L shape swung a quarter turn about the base.
    pub const QUARTER: [f64; N] = [90.0, 90.0, 0.0, 90.0, 0.0, 0.0, 0.0];
    /// Stretched straight out.
    pub const STRETCH: [f64; N] = [0.0, 90.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    /// The elbow straight, shoulder and wrist bent.
    pub const ELBOW_STRAIGHT: [f64; N] = [0.0, 60.0, 0.0, 0.0, 0.0, 60.0, 0.0];
    /// The L shape with the wrist bent: the pose whose elbow swings.
    pub const SWING: [f64; N] = [0.0, 90.0, 0.0, 90.0, 0.0, 60.0, 0.0];
    /// Over the table, the gripper pointing down.
    pub const START: [f64; N] = [0.0, 30.0, 0.0, -60.0, 0.0, 90.0, 0.0];
}

/// Where the gripper should end up: a quarter turn about the vertical from
/// the start's orientation, at this point (metres).
pub const GOAL_P: V3 = [0.40, 0.30, 0.25];
pub const GOAL_TURN_DEG: f64 = 90.0;
/// Newton's stopping tolerances (Lynch and Park, Example 6.1).
pub const TOL_SPIN: f64 = 1e-3;
pub const TOL_SLIDE: f64 = 1e-4;

pub fn goal(arm: &Arm) -> Pose {
    let start = arm.tip(&degrees(poses::START));
    Pose {
        r: mul(&turn(Z, GOAL_TURN_DEG * DEG), &start.r),
        p: GOAL_P,
    }
}

/// The book's two turns: tilt (a quarter turn about x, the far edge rising)
/// and roll (a quarter turn about y, clockwise for the viewer). Turns are
/// made about the room's axes (two lines on the table), so the later one
/// multiplies on the left.
pub fn tilt_turn(angle: f64) -> M3 {
    turn(X, angle)
}

pub fn roll_turn(angle: f64) -> M3 {
    turn(Y, angle)
}

pub fn tilt_then_roll(angle: f64) -> M3 {
    mul(&roll_turn(angle), &tilt_turn(angle))
}

pub fn roll_then_tilt(angle: f64) -> M3 {
    mul(&tilt_turn(angle), &roll_turn(angle))
}

/// The turn that separates the two orders' results.
pub fn leftover(angle: f64) -> M3 {
    mul(&tilt_then_roll(angle), &transpose(&roll_then_tilt(angle)))
}

/// The thirteen axes through a cube's faces, edges and corners.
pub const CUBE_AXES: [V3; 13] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
];
/// The angles (degrees) the edge tests sweep.
pub const SWEEP_DEG: [f64; 6] = [0.0, 30.0, 60.0, 90.0, 120.0, 150.0];

/// The 24 turns that carry a cube onto itself, grown from the book's two
/// quarter turns.
pub fn cube_turns() -> Vec<[[i32; 3]; 3]> {
    let whole = |m: &M3| m.map(|row| row.map(|x| x.round() as i32));
    let times = |a: &[[i32; 3]; 3], b: &[[i32; 3]; 3]| {
        let mut out = [[0; 3]; 3];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| a[i][k] * b[k][j]).sum();
            }
        }
        out
    };
    let makers = [whole(&tilt_turn(PI / 2.0)), whole(&roll_turn(PI / 2.0))];
    let mut all = vec![whole(&I3)];
    let mut at = 0;
    while at < all.len() {
        for g in &makers {
            let c = times(g, &all[at]);
            if !all.contains(&c) {
                all.push(c);
            }
        }
        at += 1;
    }
    all
}

/// One quiz: the storyboard line that asks it and its countdown in seconds.
pub struct Quiz {
    pub line: &'static str,
    pub seconds: u32,
}

const fn quiz(line: &'static str, seconds: u32) -> Quiz {
    Quiz { line, seconds }
}

pub const QUIZZES: &[Quiz] = &[
    quiz("01_1", 5),
    quiz("02_2", 5),
    quiz("03_2", 4),
    quiz("04_2", 5),
    quiz("05_4", 5),
    quiz("06_2", 6),
    quiz("07_1", 4),
    quiz("08_3", 4),
    quiz("09_4", 4),
    quiz("10_4", 5),
    quiz("11_2", 4),
    quiz("12_2", 6),
    quiz("13_5", 5),
    quiz("14_2", 4),
    quiz("15_2", 4),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn rodrigues_turns_are_the_elementary_ones() {
        let (c, s) = ((0.7f64).cos(), (0.7f64).sin());
        assert!(gap(&turn(Z, 0.7), &[[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]) < 1e-15);
        assert!(gap(&turn(X, 0.7), &[[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]) < 1e-15);
        assert!(gap(&turn(Y, 0.7), &[[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]) < 1e-15);
    }

    #[test]
    fn the_logarithm_undoes_the_exponential() {
        for axis in CUBE_AXES {
            for deg in [0.0, 1.0, 30.0, 90.0, 150.0, 179.0] {
                for slide in [[0.0, 0.0, 0.0], [0.3, -0.2, 0.5]] {
                    let t = twist(scale(unit(axis), deg * DEG), slide);
                    let back = log6(&exp6(t));
                    assert!(t.iter().zip(back).all(|(a, b)| (a - b).abs() < 1e-9), "{t:?} {back:?}");
                }
            }
        }
    }

    #[test]
    fn a_pose_then_its_undoing_is_nothing() {
        let t = exp6([0.3, -1.1, 0.4, 0.2, 0.5, -0.7]);
        assert!(t.then(&t.undo()).gap(&Pose::NOTHING) < 1e-12);
        assert!(t.undo().then(&t).gap(&Pose::NOTHING) < 1e-12);
    }

    #[test]
    fn the_arm_at_home_stands_straight_up() {
        let arm = Arm::iiwa14();
        assert!(
            arm.home.gap(&Pose {
                r: I3,
                p: [0.0, 0.0, 1.306]
            }) < 1e-12
        );
        let want = [Z, Y, Z, [0.0, -1.0, 0.0], Z, Y, Z];
        for (have, want) in arm.axis.iter().zip(want) {
            assert!(norm(sub(*have, want)) < 1e-12, "{have:?}");
        }
        assert!(arm.origin.iter().all(|o| o[0].abs() < 1e-12 && o[1].abs() < 1e-12));
    }

    /// The product of exponentials against the chain walked joint by joint.
    #[test]
    fn the_product_of_exponentials_is_the_chain() {
        let arm = Arm::iiwa14();
        for q in [
            poses::ELL,
            poses::SWING,
            poses::START,
            [30.0, 50.0, -20.0, -80.0, 40.0, 60.0, 10.0],
        ] {
            let q = degrees(q);
            let mut t = Pose::NOTHING;
            for (i, (xyz, [roll, pitch, yaw])) in JOINT_ORIGINS.iter().enumerate() {
                t = t
                    .then(&Pose {
                        r: rings(*yaw, *pitch, *roll),
                        p: *xyz,
                    })
                    .then(&Pose {
                        r: turn(Z, q[i]),
                        p: [0.0; 3],
                    });
            }
            t = t.then(&Pose { r: I3, p: FLANGE });
            assert!(arm.tip(&q).gap(&t) < 1e-12);
        }
    }

    /// The Jacobian against a small step of each joint.
    #[test]
    fn the_jacobian_is_the_tip_s_twist_per_joint_speed() {
        let arm = Arm::iiwa14();
        let q = degrees([30.0, 50.0, -20.0, -80.0, 40.0, 60.0, 10.0]);
        let j = arm.jacobian(&q);
        let h = 1e-6;
        for i in 0..N {
            let mut up = q;
            up[i] += h;
            let moved = log6(&arm.tip(&up).then(&arm.tip(&q).undo())).map(|x| x / h);
            assert!(
                moved.iter().zip(column(&j, i)).all(|(a, b)| (a - b).abs() < 1e-5),
                "joint {i}"
            );
        }
    }

    #[test]
    fn the_spare_joint_s_speeds_move_nothing() {
        let arm = Arm::iiwa14();
        let q = degrees(poses::SWING);
        let j = arm.jacobian(&q);
        let n = still_speeds(&j);
        assert!(twist_from(&j, &n).iter().all(|x| x.abs() < 1e-12));
        let path = arm.swing(&q, 0.01, 1.0, 2000);
        assert!(path.len() > 100, "{}", path.len());
        let hold = arm.tip(&q);
        assert!(path.iter().all(|p| arm.tip(p).gap(&hold) < 1e-9));
        let r = arm.elbow_radius(&q);
        assert!(path.iter().all(|p| close(arm.elbow_radius(p), r)));
    }

    #[test]
    fn least_speeds_produce_the_twist_asked_for() {
        let arm = Arm::iiwa14();
        let j = arm.jacobian(&degrees(poses::SWING));
        let want = [0.1, -0.2, 0.05, 0.3, 0.0, -0.1];
        let got = twist_from(&j, &least_speeds(&j, want));
        assert!(want.iter().zip(got).all(|(a, b)| close(*a, b)));
    }

    #[test]
    fn a_base_s_arc_is_its_small_steps() {
        let (v, w, t) = (0.5, 30.0 * DEG, 6.0);
        let (mut x, mut y, mut heading) = (0.0f64, 0.0f64, 0.0f64);
        let n = 60_000;
        let dt = t / f64::from(n);
        for _ in 0..n {
            x += v * (heading + w * dt / 2.0).cos() * dt;
            y += v * (heading + w * dt / 2.0).sin() * dt;
            heading += w * dt;
        }
        let end = planar_exp(v, 0.0, w, t);
        assert!((x - end.0).abs() < 1e-6 && (y - end.1).abs() < 1e-6);
    }

    #[test]
    fn headings_wrap() {
        assert!(close(wrap_deg(190.0), -170.0));
        assert!(close(wrap_deg(-180.0), 180.0));
        assert!(close(halfway_deg(170.0, -170.0), 180.0));
        assert!(close(halfway_deg(10.0, 50.0), 30.0));
    }

    #[test]
    fn the_cube_has_twenty_four_turns() {
        assert_eq!(cube_turns().len(), 24);
    }
}
