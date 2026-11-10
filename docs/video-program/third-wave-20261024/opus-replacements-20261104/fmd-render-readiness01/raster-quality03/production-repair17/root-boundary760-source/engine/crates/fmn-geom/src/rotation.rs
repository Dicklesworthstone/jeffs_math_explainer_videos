//! Quaternions and Euler angles — scipy `Rotation`'s conventions, fixed as
//! FrankenManim semantics (§7.5, §2.2).
//!
//! The Reference delegates every rotation conversion to
//! `scipy.spatial.transform.Rotation` (`manimlib/utils/space_ops.py`), and
//! user camera code is written against what that library *means*: the
//! quaternion element order and sign, the composition order, the
//! `as_euler("zxz")` branch and range choices, and the gimbal-lock
//! degeneracy rule. Those are **semantics, not implementation detail**
//! (D-05), so this module reimplements them exactly rather than picking
//! its own — the algorithms below are ports of scipy's, and every case is
//! locked against recorded scipy output in `fixtures/space_ops.txt`.
//!
//! Two deliberate differences, both refusals of silence rather than
//! changes of meaning:
//!
//! * scipy signals gimbal lock with a `UserWarning`; we return it as data
//!   ([`EulerAngles::gimbal_lock`]), because the Reference's own camera
//!   code has to branch on it (`camera_frame.py:76-82`) and a warning is
//!   not a value.
//! * scipy raises on a zero-norm quaternion; [`normalized`] returns
//!   `None`, and every entry point that needs a unit quaternion says so in
//!   its signature.
//!
//! Everything here is f64 semantic math (§6.1) and routes its
//! transcendentals through `crate::scalar` onto fmn-dmath, so certified
//! renders get bit-stable Euler and axis-angle conversions.

use fmn_core::types::Vec3;

use crate::scalar;
use crate::vec::Mat3;

/// A quaternion in scipy's storage order: `[x, y, z, w]`, **scalar last**.
///
/// The Reference states the convention explicitly
/// (`space_ops.quaternion_mult`: "the real part is the last entry, so as to
/// follow the scipy Rotation conventions") and stores camera orientation in
/// this layout, so it is API surface.
pub type Quat = [f64; 4];

/// The identity rotation, `[0, 0, 0, 1]`.
pub const IDENTITY_QUAT: Quat = [0.0, 0.0, 0.0, 1.0];

/// scipy's small-angle cutoff for the rotation-vector series (`1e-3` rad).
const SMALL_ANGLE: f64 = 1e-3;

/// scipy's gimbal-lock window on the second Euler angle (`1e-7` rad).
const GIMBAL_EPS: f64 = 1e-7;

/// `Rotation.from_quat`'s normalization, with scipy's zero-norm rejection
/// returned as `None` instead of raised.
#[must_use]
pub fn normalized(quat: Quat) -> Option<Quat> {
    let n = (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3]).sqrt();
    if n == 0.0 || !n.is_finite() {
        return None;
    }
    Some([quat[0] / n, quat[1] / n, quat[2] / n, quat[3] / n])
}

/// `Rotation.from_rotvec(rotvec).as_quat()`.
///
/// Uses scipy's series expansion below `1e-3` rad, so near-identity
/// rotations lose no precision to `sin(θ/2)/θ`'s cancellation.
#[must_use]
pub fn quat_from_rotvec(rotvec: Vec3) -> Quat {
    let angle = (rotvec[0] * rotvec[0] + rotvec[1] * rotvec[1] + rotvec[2] * rotvec[2]).sqrt();
    let scale = if angle <= SMALL_ANGLE {
        let a2 = angle * angle;
        0.5 - a2 / 48.0 + a2 * a2 / 3840.0
    } else {
        scalar::sin(angle / 2.0) / angle
    };
    [
        scale * rotvec[0],
        scale * rotvec[1],
        scale * rotvec[2],
        scalar::cos(angle / 2.0),
    ]
}

/// `Rotation.from_quat(quat).as_rotvec()`.
///
/// The double-cover choice is scipy's: a negative real part is negated
/// first, so the returned rotation vector always has magnitude in `[0, π]`.
/// Returns `None` for a zero-norm (non-)quaternion.
#[must_use]
pub fn rotvec_from_quat(quat: Quat) -> Option<Vec3> {
    let mut q = normalized(quat)?;
    if q[3] < 0.0 {
        q = [-q[0], -q[1], -q[2], -q[3]];
    }
    let vec_norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    let angle = 2.0 * scalar::atan2(vec_norm, q[3]);
    let scale = if angle <= SMALL_ANGLE {
        let a2 = angle * angle;
        2.0 + a2 / 12.0 + 7.0 * a2 * a2 / 2880.0
    } else {
        angle / scalar::sin(angle / 2.0)
    };
    Some([scale * q[0], scale * q[1], scale * q[2]])
}

/// The Hamilton product in scipy's storage order — `compose_quat(p, q)`,
/// identical to the Reference's `quaternion_mult(p, q)`.
///
/// Composition order is scipy's: the result applies `q` first, then `p`.
#[must_use]
pub fn compose_quat(p: Quat, q: Quat) -> Quat {
    [
        p[3] * q[0] + q[3] * p[0] + p[1] * q[2] - p[2] * q[1],
        p[3] * q[1] + q[3] * p[1] + p[2] * q[0] - p[0] * q[2],
        p[3] * q[2] + q[3] * p[2] + p[0] * q[1] - p[1] * q[0],
        p[3] * q[3] - p[0] * q[0] - p[1] * q[1] - p[2] * q[2],
    ]
}

/// `Rotation.from_quat(quat).as_matrix()` for an already-unit quaternion.
///
/// Row-major, and — like scipy — the matrix that maps a **column** vector:
/// `m @ v`. The Reference's `rotation_matrix_transpose_from_quaternion`
/// is this, and its `rotation_matrix_from_quaternion` is this transposed
/// (`space_ops.py:132-137`), a naming inversion we keep because scene code
/// calls both.
#[must_use]
pub fn matrix_from_unit_quat(q: Quat) -> Mat3 {
    let [x, y, z, w] = q;
    let (x2, y2, z2, w2) = (x * x, y * y, z * z, w * w);
    let (xy, zw, xz, yw, yz, xw) = (x * y, z * w, x * z, y * w, y * z, x * w);
    [
        [x2 - y2 - z2 + w2, 2.0 * (xy - zw), 2.0 * (xz + yw)],
        [2.0 * (xy + zw), -x2 + y2 - z2 + w2, 2.0 * (yz - xw)],
        [2.0 * (xz - yw), 2.0 * (yz + xw), -x2 - y2 + z2 + w2],
    ]
}

/// `Rotation.from_quat(quat).as_matrix()`, normalizing first (scipy does);
/// `None` for a zero-norm input.
#[must_use]
pub fn matrix_from_quat(quat: Quat) -> Option<Mat3> {
    normalized(quat).map(matrix_from_unit_quat)
}

/// A parsed Euler axis sequence.
///
/// Lowercase letters are **extrinsic** rotations (about the fixed frame's
/// axes) and uppercase are **intrinsic** (about the rotating body's axes) —
/// scipy's spelling, and therefore the Reference's: `CameraFrame` uses the
/// extrinsic `"zxz"` by default and supports `"zxy"`
/// (`camera_frame.py:30, :160-163`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EulerSeq {
    axes: [usize; 3],
    extrinsic: bool,
}

impl EulerSeq {
    /// Parse a three-character sequence such as `"zxz"` or `"ZXY"`.
    ///
    /// Rejects, as scipy does: sequences of the wrong length, mixed case
    /// (an axis sequence is either wholly extrinsic or wholly intrinsic),
    /// characters outside `xyz`/`XYZ`, and consecutive repeated axes
    /// (`"zzx"` is not a rotation sequence).
    #[must_use]
    pub fn parse(seq: &str) -> Option<Self> {
        let chars: Vec<char> = seq.chars().collect();
        if chars.len() != 3 {
            return None;
        }
        let extrinsic = chars[0].is_ascii_lowercase();
        let mut axes = [0usize; 3];
        for (slot, c) in axes.iter_mut().zip(chars.iter()) {
            if c.is_ascii_lowercase() != extrinsic {
                return None;
            }
            *slot = match c.to_ascii_lowercase() {
                'x' => 0,
                'y' => 1,
                'z' => 2,
                _ => return None,
            };
        }
        if axes[0] == axes[1] || axes[1] == axes[2] {
            return None;
        }
        Some(Self { axes, extrinsic })
    }

    /// The three axis indices (`x`=0, `y`=1, `z`=2), in sequence order.
    #[must_use]
    pub fn axes(&self) -> [usize; 3] {
        self.axes
    }

    /// Whether the sequence is extrinsic (lowercase).
    #[must_use]
    pub fn extrinsic(&self) -> bool {
        self.extrinsic
    }

    /// A symmetric (proper-Euler) sequence repeats its first axis third —
    /// `zxz`; an asymmetric (Tait–Bryan) sequence uses all three — `zxy`.
    /// The two families have different second-angle ranges and different
    /// degeneracies.
    #[must_use]
    pub fn symmetric(&self) -> bool {
        self.axes[0] == self.axes[2]
    }
}

/// The result of an Euler decomposition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EulerAngles {
    /// The three angles, in the sequence's own order — for the extrinsic
    /// `"zxz"` the Reference then reverses them into `(theta, phi, gamma)`
    /// (`camera_frame.py:74`).
    pub angles: [f64; 3],
    /// Set when the decomposition hit a degeneracy (the second angle at a
    /// pole), where the first and third rotations act about the same axis
    /// and only their sum or difference is recoverable. In that case the
    /// third angle is zero by construction and the whole rotation is
    /// carried by the first — scipy's choice, kept.
    pub gimbal_lock: bool,
}

/// `Rotation.from_euler(seq, angles).as_quat()`.
///
/// Angles are in radians and in sequence order. The composition order is
/// scipy's: extrinsic sequences pre-multiply each successive elementary
/// rotation, intrinsic ones post-multiply.
#[must_use]
pub fn quat_from_euler(seq: EulerSeq, angles: [f64; 3]) -> Quat {
    let mut result = elementary_quat(seq.axes[0], angles[0]);
    for (axis, angle) in seq.axes.iter().zip(angles.iter()).skip(1) {
        let next = elementary_quat(*axis, *angle);
        result = if seq.extrinsic {
            compose_quat(next, result)
        } else {
            compose_quat(result, next)
        };
    }
    result
}

fn elementary_quat(axis: usize, angle: f64) -> Quat {
    let mut q = [0.0, 0.0, 0.0, scalar::cos(angle / 2.0)];
    q[axis] = scalar::sin(angle / 2.0);
    q
}

/// `Rotation.from_quat(quat).as_euler(seq)`.
///
/// A port of scipy's quaternion-direct decomposition (Bernardes & Viollet,
/// 2022), including its branch structure: the second angle comes from a
/// pair of hypotenuses (so it is never fed to `acos` near a pole), the
/// first and third from the half-sum and half-difference, and a degeneracy
/// inside `GIMBAL_EPS` collapses onto the first angle. Every returned
/// angle is wrapped into `[-π, π]`.
///
/// Returns `None` only for a zero-norm quaternion.
#[must_use]
pub fn euler_from_quat(quat: Quat, seq: EulerSeq) -> Option<EulerAngles> {
    let q = normalized(quat)?;

    // The algorithm is formulated for extrinsic sequences; an intrinsic
    // sequence is the same computation on the reversed axis order, with
    // the first and third angles written back in swapped slots.
    let (slot_first, slot_third) = if seq.extrinsic { (0, 2) } else { (2, 0) };
    let axes = if seq.extrinsic {
        seq.axes
    } else {
        [seq.axes[2], seq.axes[1], seq.axes[0]]
    };
    let (i, j) = (axes[0], axes[1]);
    let symmetric = i == axes[2];
    let k = if symmetric { 3 - i - j } else { axes[2] };

    // +1 for an even permutation of (x, y, z), -1 for an odd one.
    let sign = ((i as i64 - j as i64) * (j as i64 - k as i64) * (k as i64 - i as i64) / 2) as f64;

    let (a, b, c, d) = if symmetric {
        (q[3], q[i], q[j], q[k] * sign)
    } else {
        (
            q[3] - q[j],
            q[i] + q[k] * sign,
            q[j] + q[3],
            q[k] * sign - q[i],
        )
    };

    let mut angles = [0.0f64; 3];
    angles[1] = 2.0 * scalar::atan2(hypot(c, d), hypot(a, b));

    let case = if angles[1].abs() <= GIMBAL_EPS {
        1
    } else if (angles[1] - fmn_core::constants::PI).abs() <= GIMBAL_EPS {
        2
    } else {
        0
    };

    let half_sum = scalar::atan2(b, a);
    let half_diff = scalar::atan2(d, c);

    if case == 0 {
        angles[slot_first] = half_sum - half_diff;
        angles[slot_third] = half_sum + half_diff;
    } else {
        // A degeneracy always parks the recoverable combination in the
        // first slot and zeroes the third — by index, not by role, which
        // is why an intrinsic lock reports it in `angles[0]` too.
        angles[2] = 0.0;
        angles[0] = if case == 1 {
            2.0 * half_sum
        } else if seq.extrinsic {
            -2.0 * half_diff
        } else {
            2.0 * half_diff
        };
    }

    if !symmetric {
        angles[slot_third] *= sign;
        angles[1] -= fmn_core::constants::PI / 2.0;
    }

    for angle in &mut angles {
        if *angle < -fmn_core::constants::PI {
            *angle += fmn_core::constants::TAU;
        } else if *angle > fmn_core::constants::PI {
            *angle -= fmn_core::constants::TAU;
        }
    }

    Some(EulerAngles {
        angles,
        gimbal_lock: case != 0,
    })
}

/// `math.hypot` without the intermediate overflow — the scaled form, so a
/// pair like `(1e200, 1e200)` does not square to infinity.
fn hypot(a: f64, b: f64) -> f64 {
    let (a, b) = (a.abs(), b.abs());
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    if hi == 0.0 {
        return 0.0;
    }
    let r = lo / hi;
    hi * (1.0 + r * r).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::{PI, TAU};

    fn quat_close(a: Quat, b: Quat, tol: f64) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < tol)
    }

    #[test]
    fn identity_round_trips() {
        assert_eq!(quat_from_rotvec([0.0; 3]), IDENTITY_QUAT);
        let e = euler_from_quat(IDENTITY_QUAT, EulerSeq::parse("zxz").unwrap()).unwrap();
        assert_eq!(e.angles, [0.0; 3]);
        assert!(e.gimbal_lock);
    }

    #[test]
    fn rotvec_round_trip() {
        for rotvec in [
            [0.0, 0.0, 1.0],
            [0.3, -0.4, 0.5],
            [PI - 1e-6, 0.0, 0.0],
            [1e-9, 2e-9, 0.0],
        ] {
            let q = quat_from_rotvec(rotvec);
            let back = rotvec_from_quat(q).unwrap();
            for i in 0..3 {
                assert!(
                    (back[i] - rotvec[i]).abs() < 1e-12,
                    "{rotvec:?} -> {back:?}"
                );
            }
        }
    }

    #[test]
    fn rotvec_from_quat_picks_the_short_way_round() {
        // A 3π/2 turn about z is reported as a -π/2 turn: scipy's
        // double-cover choice keeps |angle| <= π.
        let q = quat_from_rotvec([0.0, 0.0, 3.0 * PI / 2.0]);
        let back = rotvec_from_quat(q).unwrap();
        assert!((back[2] + PI / 2.0).abs() < 1e-12, "{back:?}");
    }

    #[test]
    fn zero_quaternion_is_refused_everywhere() {
        assert!(normalized([0.0; 4]).is_none());
        assert!(rotvec_from_quat([0.0; 4]).is_none());
        assert!(matrix_from_quat([0.0; 4]).is_none());
        assert!(euler_from_quat([0.0; 4], EulerSeq::parse("zxz").unwrap()).is_none());
    }

    #[test]
    fn composition_is_not_commutative_and_matches_matrix_product() {
        let p = quat_from_rotvec([0.0, 0.0, 0.7]);
        let q = quat_from_rotvec([0.9, 0.0, 0.0]);
        assert!(!quat_close(compose_quat(p, q), compose_quat(q, p), 1e-9));

        // compose_quat(p, q) applies q first: M(pq) == M(p) M(q).
        let m_pq = matrix_from_unit_quat(compose_quat(p, q));
        let (mp, mq) = (matrix_from_unit_quat(p), matrix_from_unit_quat(q));
        for r in 0..3 {
            for c in 0..3 {
                let expected: f64 = (0..3).map(|k| mp[r][k] * mq[k][c]).sum();
                assert!((m_pq[r][c] - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn euler_sequences_parse_and_refuse() {
        let zxz = EulerSeq::parse("zxz").unwrap();
        assert_eq!(zxz.axes(), [2, 0, 2]);
        assert!(zxz.extrinsic());
        assert!(zxz.symmetric());

        let big = EulerSeq::parse("ZXY").unwrap();
        assert!(!big.extrinsic());
        assert!(!big.symmetric());

        assert!(EulerSeq::parse("zx").is_none(), "too short");
        assert!(EulerSeq::parse("zxzz").is_none(), "too long");
        assert!(EulerSeq::parse("zXz").is_none(), "mixed case");
        assert!(EulerSeq::parse("zqz").is_none(), "bad axis");
        assert!(EulerSeq::parse("zzx").is_none(), "repeated adjacent axis");
    }

    #[test]
    fn euler_round_trip_away_from_poles() {
        for seq_name in ["zxz", "zxy", "xyz", "ZXZ", "ZXY", "XYZ"] {
            let seq = EulerSeq::parse(seq_name).unwrap();
            for angles in [[0.3, 1.1, -0.4], [-2.2, 0.6, 1.9], [1.0, 2.0, -1.0]] {
                let q = quat_from_euler(seq, angles);
                let back = euler_from_quat(q, seq).unwrap();
                assert!(!back.gimbal_lock, "{seq_name} {angles:?}");
                let q2 = quat_from_euler(seq, back.angles);
                // Same rotation, possibly the other double-cover sign.
                let same = quat_close(q, q2, 1e-9)
                    || quat_close(q, [-q2[0], -q2[1], -q2[2], -q2[3]], 1e-9);
                assert!(same, "{seq_name} {angles:?} -> {:?}", back.angles);
            }
        }
    }

    #[test]
    fn gimbal_lock_collapses_onto_the_first_angle() {
        let seq = EulerSeq::parse("zxz").unwrap();
        // Second angle at 0: only the sum of the outer two is recoverable.
        let q = quat_from_euler(seq, [0.4, 0.0, 0.9]);
        let e = euler_from_quat(q, seq).unwrap();
        assert!(e.gimbal_lock);
        assert_eq!(e.angles[2], 0.0);
        assert!((e.angles[0] - 1.3).abs() < 1e-12, "{:?}", e.angles);

        // Second angle at π: only the difference is recoverable.
        let q = quat_from_euler(seq, [0.4, PI, 0.9]);
        let e = euler_from_quat(q, seq).unwrap();
        assert!(e.gimbal_lock);
        assert_eq!(e.angles[2], 0.0);
        assert!((e.angles[0] + 0.5).abs() < 1e-12, "{:?}", e.angles);
    }

    #[test]
    fn just_outside_the_lock_window_is_not_a_lock() {
        let seq = EulerSeq::parse("zxz").unwrap();
        let e = euler_from_quat(quat_from_euler(seq, [0.4, 1e-5, 0.9]), seq).unwrap();
        assert!(!e.gimbal_lock);
        let e = euler_from_quat(quat_from_euler(seq, [0.4, 1e-9, 0.9]), seq).unwrap();
        assert!(e.gimbal_lock);
    }

    #[test]
    fn euler_angles_stay_in_range() {
        let seq = EulerSeq::parse("zxz").unwrap();
        for k in 0..64 {
            let t = -3.0 * TAU + k as f64 * TAU / 7.0;
            let e = euler_from_quat(quat_from_euler(seq, [t, t / 2.0, -t]), seq).unwrap();
            for a in e.angles {
                assert!((-PI..=PI).contains(&a), "{a} out of range");
            }
        }
    }

    #[test]
    fn matrix_from_unit_quat_is_orthonormal() {
        let q = quat_from_rotvec([0.4, -1.2, 0.9]);
        let m = matrix_from_unit_quat(q);
        for r in 0..3 {
            for c in 0..3 {
                let dot: f64 = (0..3).map(|k| m[r][k] * m[c][k]).sum();
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!((dot - expected).abs() < 1e-14);
            }
        }
    }

    #[test]
    fn hypot_survives_overflow_scale() {
        assert!((hypot(3e200, 4e200) - 5e200).abs() < 1e188);
        assert_eq!(hypot(0.0, 0.0), 0.0);
        assert_eq!(hypot(0.0, -2.0), 2.0);
    }
}
