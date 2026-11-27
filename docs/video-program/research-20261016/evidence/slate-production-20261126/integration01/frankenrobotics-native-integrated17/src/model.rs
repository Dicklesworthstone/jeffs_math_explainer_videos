//! Exact algebraic and kinematic models for The Math Behind FrankenRobotics.
//!
//! Pure `std` Rust with zero external dependencies so every scene calculation,
//! group law, permutation matrix, Reynolds projection, snake holonomy loop,
//! corridor clearance, stance load balance, and docking update is computed from
//! first principles and independently cross-checked against `oracle.py`.

use std::f64::consts::{PI, TAU};

/// A planar rigid transform in SE(2): rotation by `theta` and translation `(x, y)`.
/// Matrix representation on column vectors:
/// `[[cos(theta), -sin(theta), x], [sin(theta), cos(theta), y], [0, 0, 1]]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Se2 {
    pub theta: f64,
    pub x: f64,
    pub y: f64,
}

impl Se2 {
    pub const IDENTITY: Self = Self {
        theta: 0.0,
        x: 0.0,
        y: 0.0,
    };

    pub const fn rotation(theta: f64) -> Self {
        Self { theta, x: 0.0, y: 0.0 }
    }

    pub const fn translation(x: f64, y: f64) -> Self {
        Self { theta: 0.0, x, y }
    }

    /// Semidirect product `(R, p)(S, q) = (R S, p + R q)`.
    pub fn compose(self, right: Self) -> Self {
        let (c, s) = (self.theta.cos(), self.theta.sin());
        Self {
            theta: self.theta + right.theta,
            x: self.x + c * right.x - s * right.y,
            y: self.y + s * right.x + c * right.y,
        }
    }

    /// Action on a planar point `(px, py)`.
    pub fn apply(self, point: [f64; 2]) -> [f64; 2] {
        let (c, s) = (self.theta.cos(), self.theta.sin());
        [
            self.x + c * point[0] - s * point[1],
            self.y + s * point[0] + c * point[1],
        ]
    }

    /// Group inverse `(R^T, -R^T p)`.
    pub fn inverse(self) -> Self {
        let (c, s) = (self.theta.cos(), self.theta.sin());
        Self {
            theta: -self.theta,
            x: -(c * self.x + s * self.y),
            y: -(-s * self.x + c * self.y),
        }
    }

    /// Exact four-move rover commutator `R(theta) T(a) R(-theta) T(-a)`.
    pub fn rover_commutator(theta: f64, a: f64) -> Self {
        Self::rotation(theta)
            .compose(Self::translation(a, 0.0))
            .compose(Self::rotation(-theta))
            .compose(Self::translation(-a, 0.0))
    }

    /// Piecewise-smooth rover pose along the four-move commutator for `progress` in `[0, 4]`.
    /// Each unit interval executes one local move:
    /// `[0, 1]`: turn `+theta`, `[1, 2]`: drive `+a`,
    /// `[2, 3]`: turn `-theta`, `[3, 4]`: drive `-a`.
    pub fn rover_commutator_at(theta: f64, a: f64, progress: f64) -> Self {
        let u = progress.clamp(0.0, 4.0);
        if u <= 1.0 {
            Self::rotation(theta * u)
        } else if u <= 2.0 {
            Self::rotation(theta).compose(Self::translation(a * (u - 1.0), 0.0))
        } else if u <= 3.0 {
            Self::rotation(theta)
                .compose(Self::translation(a, 0.0))
                .compose(Self::rotation(-theta * (u - 2.0)))
        } else {
            Self::rotation(theta)
                .compose(Self::translation(a, 0.0))
                .compose(Self::rotation(-theta))
                .compose(Self::translation(-a * (u - 3.0), 0.0))
        }
    }
}

/// Planar two-link revolute arm on `SO(2) x SO(2)`.
#[derive(Clone, Copy, Debug)]
pub struct Arm2R {
    pub l1: f64,
    pub l2: f64,
}

impl Arm2R {
    pub const UNIT: Self = Self { l1: 1.0, l2: 1.0 };
    pub const BENCH: Self = Self { l1: 0.4, l2: 0.3 };

    pub fn elbow(self, q1: f64) -> [f64; 2] {
        Se2::rotation(q1).compose(Se2::translation(self.l1, 0.0)).apply([0.0, 0.0])
    }

    pub fn hand(self, q1: f64, q2: f64) -> [f64; 2] {
        Se2::rotation(q1)
            .compose(Se2::translation(self.l1, 0.0))
            .compose(Se2::rotation(q2))
            .compose(Se2::translation(self.l2, 0.0))
            .apply([0.0, 0.0])
    }

    /// Translational Jacobian `J(q1, q2)` mapping `[q1_dot, q2_dot]` to `[vx, vy]`.
    pub fn jacobian(self, q1: f64, q2: f64) -> [[f64; 2]; 2] {
        let (s1, c1) = (q1.sin(), q1.cos());
        let (s12, c12) = ((q1 + q2).sin(), (q1 + q2).cos());
        [
            [-self.l1 * s1 - self.l2 * s12, -self.l2 * s12],
            [self.l1 * c1 + self.l2 * c12, self.l2 * c12],
        ]
    }

    pub fn torques(self, q1: f64, q2: f64, tip_force: [f64; 2]) -> [f64; 2] {
        let j = self.jacobian(q1, q2);
        [
            j[0][0] * tip_force[0] + j[1][0] * tip_force[1],
            j[0][1] * tip_force[0] + j[1][1] * tip_force[1],
        ]
    }

    pub fn tip_velocity(self, q1: f64, q2: f64, rates: [f64; 2]) -> [f64; 2] {
        let j = self.jacobian(q1, q2);
        [
            j[0][0] * rates[0] + j[0][1] * rates[1],
            j[1][0] * rates[0] + j[1][1] * rates[1],
        ]
    }
}

/// Ideal three-wheel omnidirectional triangle (`omni-3`, wheels at 0, 120, 240 deg, unit radius).
pub struct Omni3;

impl Omni3 {
    /// Wheel rolling tangents `(0, 1)`, `(-sqrt(3)/2, -1/2)`, `(sqrt(3)/2, -1/2)`.
    pub fn tangents() -> [[f64; 2]; 3] {
        let s = 3.0_f64.sqrt() / 2.0;
        [[0.0, 1.0], [-s, -0.5], [s, -0.5]]
    }

    /// Signed rim speeds `[u1, u2, u3]` for body velocity `[vx, vy, omega]`.
    pub fn rim_speeds(vx: f64, vy: f64, omega: f64) -> [f64; 3] {
        let t = Self::tangents();
        [
            t[0][0] * vx + t[0][1] * vy + omega,
            t[1][0] * vx + t[1][1] * vy + omega,
            t[2][0] * vx + t[2][1] * vy + omega,
        ]
    }

    pub fn unscaled_determinant() -> f64 {
        1.5 * 3.0_f64.sqrt()
    }
}

/// Ideal rigid track contact samples at `+a` and `-a` ahead/behind center (`a = 0.5`).
pub fn track_lateral_velocities(vy: f64, omega: f64, half_length: f64) -> [f64; 2] {
    [vy + half_length * omega, vy - half_length * omega]
}

/// GB-05 minimizes travel per wheel within the four-move family for small lateral shifts.
pub fn four_move_wheel_travel(d: f64, b: f64) -> (f64, f64) {
    let alpha = 2.0 * (d / (4.0 * b)).sqrt().asin();
    let travel = 2.0 * alpha * b + d / (alpha / 2.0).tan();
    (alpha, travel)
}

/// Four-foot rectangular quadruped symmetry group `C_2 x C_2` in channel order
/// `[FL, FR, BL, BR] = [0, 1, 2, 3]`, matching `scene-facts.py` and `treatment.md`.
pub const QUAD_PERMS: [[usize; 4]; 4] = [
    [0, 1, 2, 3], // identity
    [1, 0, 3, 2], // left-right reflection
    [2, 3, 0, 1], // front-back reflection
    [3, 2, 1, 0], // half turn
];

/// Orthogonal character sign patterns of `C_2 x C_2`: heave, pitch, roll, diagonal warp.
pub const QUAD_MODES: [[i64; 4]; 4] = [
    [1, 1, 1, 1],
    [1, 1, -1, -1],
    [1, -1, 1, -1],
    [1, -1, -1, 1],
];

/// Teaching coupling matrix `K` with first row `(3, 1, 2, 0)` (self, same row, same side, opposite).
pub const QUAD_COUPLING: [[i64; 4]; 4] = [
    [3, 1, 2, 0],
    [1, 3, 0, 2],
    [2, 0, 3, 1],
    [0, 2, 1, 3],
];

/// Surviving symmetry subgroup indices in `QUAD_PERMS` given hardware/sensor decorations.
pub fn quad_stabilizer(has_lidar: bool, has_off_axis_payload: bool, swapped_gearbox: bool) -> Vec<usize> {
    if swapped_gearbox || has_off_axis_payload {
        vec![0]
    } else if has_lidar {
        vec![0, 1]
    } else {
        vec![0, 1, 2, 3]
    }
}

/// Exact integer mode gains `[6, 2, 4, 0]` of `QUAD_COUPLING` on `QUAD_MODES`.
pub fn quad_mode_gains() -> [i64; 4] {
    let mut gains = [0_i64; 4];
    for (k, mode) in QUAD_MODES.iter().enumerate() {
        let mut row0 = 0_i64;
        for j in 0..4 {
            row0 += QUAD_COUPLING[0][j] * mode[j];
        }
        gains[k] = row0;
    }
    gains
}

/// Reynolds projection of the front-left `+4` diagonal perturbation over a chosen subgroup of `QUAD_PERMS`.
/// Returns the resulting diagonal entries (all off-diagonals are zero for a diagonal input).
pub fn reynolds_project_fl4(subgroup: &[usize]) -> [i64; 4] {
    let count = subgroup.len() as i64;
    let mut diag = [0_i64; 4];
    for &g in subgroup {
        let p = QUAD_PERMS[g];
        // Conjugating e_{0,0} by P_g places 4 at (i, i) where p[i] == 0.
        for i in 0..4 {
            if p[i] == 0 {
                diag[i] += 4;
            }
        }
    }
    for d in &mut diag {
        assert_eq!(*d % count, 0, "exact integer projection");
        *d /= count;
    }
    diag
}

/// Radial hexapod `D_6` permutation actions on indices `0..6`.
pub fn hexapod_d6_elements() -> Vec<[usize; 6]> {
    let mut elems = Vec::with_capacity(12);
    for rot in 0..6 {
        let mut r = [0_usize; 6];
        let mut sr = [0_usize; 6];
        for i in 0..6 {
            let turned = (i + rot) % 6;
            r[i] = turned;
            sr[i] = (6 - turned) % 6;
        }
        elems.push(r);
        elems.push(sr);
    }
    elems
}

/// Burnside orbit count for `k` leg types across 6 slots under `D_6`.
pub const fn hexapod_d6_orbits(k: u64) -> u64 {
    let num = k.pow(6) + 2 * k + 2 * k.pow(2) + k.pow(3) + 3 * k.pow(4) + 3 * k.pow(3);
    num / 12
}

/// Planar 3-link kinematic wheeled snake local connection `A = (A1, A2)` (links `L = 1`, `h = 0.5`).
pub fn snake3_connection(a1: f64, a2: f64) -> ([f64; 3], [f64; 3], f64) {
    let h = 0.5;
    let (s1, c1) = (a1.sin(), a1.cos());
    let (s2, c2) = (a2.sin(), a2.cos());
    let denom = s1 * (1.0 + c2) + s2 * (1.0 + c1);
    let col1 = [h * (1.0 + c2) / denom, 0.0, s2 / denom];
    let col2 = [-h * (1.0 + c1) / denom, 0.0, s1 / denom];
    (col1, col2, denom)
}

/// Integrate the 3-link wheeled snake around a closed circle in `(a1, a2)` using RK4.
pub fn snake3_integrate_loop(center: [f64; 2], radius: f64, clockwise: bool, steps: usize) -> Se2 {
    let sign = if clockwise { -1.0 } else { 1.0 };
    let dt = TAU / steps as f64;
    let mut state = [0.0_f64; 3]; // x, y, theta

    let deriv = |t: f64, s: [f64; 3]| -> [f64; 3] {
        let a1 = center[0] + radius * t.cos();
        let a2 = center[1] + sign * radius * t.sin();
        let da1 = -radius * t.sin();
        let da2 = sign * radius * t.cos();
        let (c1, c2, _) = snake3_connection(a1, a2);
        let vx = -(c1[0] * da1 + c2[0] * da2);
        let om = -(c1[2] * da1 + c2[2] * da2);
        [vx * s[2].cos(), vx * s[2].sin(), om]
    };

    for k in 0..steps {
        let t0 = k as f64 * dt;
        let k1 = deriv(t0, state);
        let k2 = deriv(t0 + 0.5 * dt, [state[0] + 0.5 * dt * k1[0], state[1] + 0.5 * dt * k1[1], state[2] + 0.5 * dt * k1[2]]);
        let k3 = deriv(t0 + 0.5 * dt, [state[0] + 0.5 * dt * k2[0], state[1] + 0.5 * dt * k2[1], state[2] + 0.5 * dt * k2[2]]);
        let k4 = deriv(t0 + dt, [state[0] + dt * k3[0], state[1] + dt * k3[1], state[2] + dt * k3[2]]);
        for i in 0..3 {
            state[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
    }
    Se2 {
        theta: state[2],
        x: state[0],
        y: state[1],
    }
}

/// Exact longest rectangle of width `b` that turns an L-shaped corridor of width `a`.
pub fn corridor_exact_limit(a: f64, b: f64) -> f64 {
    2.0 * 2.0_f64.sqrt() * a - 2.0 * b
}

/// Balanced-clearance SE(2) pose turning through the L-corridor at angle `phi` in `[0, pi/2]`.
pub fn corridor_balanced_pose(length: f64, width: f64, corridor_w: f64, phi: f64) -> (Se2, f64) {
    let (c, s) = (phi.cos(), phi.sin());
    let margin = (corridor_w * (s + c) - length * s * c - width) / (1.0 + s + c);
    let ex = 0.5 * length * c + 0.5 * width * s;
    let ey = 0.5 * length * s + 0.5 * width * c;
    (
        Se2 {
            theta: PI - phi,
            x: ex + margin,
            y: ey + margin,
        },
        margin,
    )
}

pub const STAIR_SUN_TEETH: u32 = 24;
pub const STAIR_IDLER_TEETH: u32 = 12;
pub const STAIR_WHEEL_TEETH: u32 = 20;
pub const STAIR_AXLE_OFFSET_M: f64 = 0.17;
pub const STAIR_TIRE_RADIUS_M: f64 = 0.095;

/// Idler angle in the chassis, wheel angle in the carrier, and wheel angle in
/// the chassis. Each of the two external meshes reverses the relative spin.
pub fn stair_cluster_angles(carrier: f64, sun: f64) -> (f64, f64, f64) {
    let sun_relative = sun - carrier;
    let idler_relative = -f64::from(STAIR_SUN_TEETH) / f64::from(STAIR_IDLER_TEETH) * sun_relative;
    let wheel_relative = -f64::from(STAIR_IDLER_TEETH) / f64::from(STAIR_WHEEL_TEETH) * idler_relative;
    (carrier + idler_relative, wheel_relative, carrier + wheel_relative)
}

/// Positive pitch is counterclockwise in the side view, tipping the torso back.
pub fn balancer_wheel_relative(advance_in_radii: f64, pitch: f64) -> f64 {
    -advance_in_radii - pitch
}

/// Four-foot static stance allocation (website corner order `++, +-, -+, --`, `W = 400 N`, `a = 1/3 m`).
pub fn stance_equal_springs(half_width_m: f64) -> [f64; 4] {
    let sx = (60.0 / 4.0) / (1.0 / 3.0); // 45 N
    let sy = (30.0 / 4.0) / half_width_m; // 30 N at b = 1/4 m
    [100.0 + sx + sy, 100.0 + sx - sy, 100.0 - sx + sy, 100.0 - sx - sy]
}

/// Apply diagonal warp shift `delta * (+1, -1, -1, +1)` without altering total weight or moments.
pub fn stance_apply_diagonal_shift(forces: [f64; 4], delta: f64) -> [f64; 4] {
    [
        forces[0] + delta,
        forces[1] - delta,
        forces[2] - delta,
        forces[3] + delta,
    ]
}

/// Three-step docking cart affine trajectory from `(0, 0)` for gain `g` and first push `p`.
pub fn dock_cart_ride(gain: f64, first_push: f64) -> ([f64; 3], [f64; 2]) {
    let pushes = [first_push, 1.0 / gain - 2.0 * first_push, first_push - 1.0 / gain];
    let (mut x, mut v) = (0.0, 0.0);
    for u in pushes {
        x += v + 0.5 * gain * u;
        v += gain * u;
    }
    (pushes, [x, v])
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;

    #[test]
    fn arm_and_se2_order_match_oracle() {
        let hand = Arm2R::UNIT.hand(0.0, FRAC_PI_2);
        assert!((hand[0] - 1.0).abs() < 1e-12 && (hand[1] - 1.0).abs() < 1e-12);

        let rt = Se2::rotation(FRAC_PI_2).compose(Se2::translation(1.0, 0.0)).apply([0.0, 0.0]);
        let tr = Se2::translation(1.0, 0.0).compose(Se2::rotation(FRAC_PI_2)).apply([0.0, 0.0]);
        assert!((rt[0] - 0.0).abs() < 1e-12 && (rt[1] - 1.0).abs() < 1e-12);
        assert!((tr[0] - 1.0).abs() < 1e-12 && (tr[1] - 0.0).abs() < 1e-12);

        let tau = Arm2R::BENCH.torques(0.0, FRAC_PI_2, [10.0, 0.0]);
        let vel = Arm2R::BENCH.tip_velocity(0.0, FRAC_PI_2, [0.2, -0.1]);
        assert!((tau[0] + 3.0).abs() < 1e-12 && (tau[1] + 3.0).abs() < 1e-12);
        assert!((vel[0] + 0.03).abs() < 1e-12 && (vel[1] - 0.08).abs() < 1e-12);
    }

    #[test]
    fn drives_and_commutator_match_oracle() {
        let rim = Omni3::rim_speeds(0.0, 2.0, 0.0);
        assert!((rim[0] - 2.0).abs() < 1e-12 && (rim[1] + 1.0).abs() < 1e-12 && (rim[2] + 1.0).abs() < 1e-12);

        let track = track_lateral_velocities(0.0, 1.0, 0.5);
        assert_eq!(track, [0.5, -0.5]);

        let c = Se2::rover_commutator(FRAC_PI_2, 1.0);
        assert!(c.theta.abs() < 1e-12 && (c.x + 1.0).abs() < 1e-12 && (c.y - 1.0).abs() < 1e-12);

        let end = Se2::rover_commutator_at(FRAC_PI_2, 1.0, 4.0);
        assert!((end.x - c.x).abs() < 1e-12 && (end.y - c.y).abs() < 1e-12 && end.theta.abs() < 1e-12);

        let (alpha, cost) = four_move_wheel_travel(0.10, 0.20);
        assert!((alpha - 0.7227).abs() < 1e-3 && (cost - 0.5537).abs() < 1e-3);
    }

    #[test]
    fn symmetries_modes_and_reynolds_match_oracle() {
        assert_eq!(quad_mode_gains(), [6, 2, 4, 0]);
        assert_eq!(reynolds_project_fl4(&quad_stabilizer(false, false, false)), [1, 1, 1, 1]);
        assert_eq!(reynolds_project_fl4(&quad_stabilizer(true, false, false)), [2, 2, 0, 0]);
        assert_eq!(reynolds_project_fl4(&quad_stabilizer(true, true, false)), [4, 0, 0, 0]);
        assert_eq!(hexapod_d6_elements().len(), 12);
        assert_eq!(hexapod_d6_orbits(10), 86_185);
    }

    #[test]
    fn snake_corridor_stance_and_dock_match_oracle() {
        let s3 = snake3_integrate_loop([0.9, 0.9], 0.4, true, 4000);
        assert!((s3.x - 0.202930846).abs() < 1e-6);
        assert!((s3.y - 0.127232429).abs() < 1e-6);
        assert!(s3.theta.abs() < 1e-9);

        assert!((corridor_exact_limit(0.90, 0.60) - 1.3455844).abs() < 1e-5);
        let (_, m130) = corridor_balanced_pose(1.30, 0.60, 0.90, FRAC_PI_2 / 2.0);
        let (_, m140) = corridor_balanced_pose(1.40, 0.60, 0.90, FRAC_PI_2 / 2.0);
        assert!((m130 * 1000.0 - 9.44).abs() < 0.05);
        assert!((m140 * 1000.0 + 11.27).abs() < 0.05);

        let springs = stance_equal_springs(0.25);
        let active = stance_apply_diagonal_shift(springs, -20.0);
        assert_eq!(springs, [175.0, 115.0, 85.0, 25.0]);
        assert_eq!(active, [155.0, 135.0, 105.0, 5.0]);

        for g in [1.0, 2.0] {
            let (_, end) = dock_cart_ride(g, 0.25);
            assert!((end[0] - 1.0).abs() < 1e-12 && end[1].abs() < 1e-12);
        }
    }
}
