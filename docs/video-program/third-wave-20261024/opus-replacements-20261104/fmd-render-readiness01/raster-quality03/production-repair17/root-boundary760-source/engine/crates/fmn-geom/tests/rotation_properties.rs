//! Rotation-convention properties and singularities (fm-ngx, §7.5, §2.2).
//!
//! `space_ops_parity.rs` locks *what* the conventions produce, case by
//! case, against the Reference. This file locks the *laws* they have to
//! obey everywhere in between — composition associativity, the
//! double cover, Euler round-trips within the branch conventions — and the
//! behavior at the hard points the plan names: gimbal lock, quaternion
//! sign at ±π, near-identity and near-π rotations, axis-angle degeneracies.
//!
//! The standing rule for every degenerate input here: **a defined output,
//! never a NaN**. A NaN escaping this layer would propagate into a
//! bounding box, then a layout, then a frame, and be diagnosed nowhere
//! near where it was born.

use fmn_core::constants::{DOWN, LEFT, ORIGIN, OUT, PI, RIGHT, TAU, UP};
use fmn_core::types::Vec3;
use fmn_geom::rotation::{self, EulerSeq, IDENTITY_QUAT, Quat};
use fmn_geom::space_ops as so;

const SEQS: [&str; 6] = ["zxz", "zxy", "xyz", "ZXZ", "ZXY", "XYZ"];

/// A deterministic spread of rotation vectors: cardinal axes, general
/// axes, and magnitudes from denormal-adjacent to several full turns.
fn sample_rotvecs() -> Vec<Vec3> {
    let axes: [Vec3; 6] = [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.3, -1.7, 2.2],
        [-1.0, 0.4, 0.0],
    ];
    let angles = [
        0.0,
        1e-15,
        1e-9,
        1e-3,
        0.5,
        PI / 2.0,
        PI - 1e-9,
        PI,
        PI + 0.4,
        TAU - 1e-6,
        3.0 * TAU + 1.1,
    ];
    let mut out = Vec::new();
    for axis in axes {
        let unit = so::normalize(axis);
        for angle in angles {
            out.push([unit[0] * angle, unit[1] * angle, unit[2] * angle]);
        }
    }
    out
}

fn quat_norm(q: Quat) -> f64 {
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt()
}

fn same_rotation(a: Quat, b: Quat, tol: f64) -> bool {
    let direct = (0..4).all(|i| (a[i] - b[i]).abs() < tol);
    let flipped = (0..4).all(|i| (a[i] + b[i]).abs() < tol);
    direct || flipped
}

fn matrices_close(a: [[f64; 3]; 3], b: [[f64; 3]; 3], tol: f64) -> bool {
    (0..3).all(|r| (0..3).all(|c| (a[r][c] - b[r][c]).abs() < tol))
}

fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, slot) in row.iter_mut().enumerate() {
            *slot = (0..3).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

fn finite(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite())
}

#[test]
fn every_produced_quaternion_is_a_unit_quaternion() {
    for rotvec in sample_rotvecs() {
        let q = rotation::quat_from_rotvec(rotvec);
        assert!(finite(&q), "{rotvec:?} produced {q:?}");
        assert!(
            (quat_norm(q) - 1.0).abs() < 1e-15,
            "{rotvec:?} -> norm {}",
            quat_norm(q)
        );
    }
    for seq_name in SEQS {
        let seq = EulerSeq::parse(seq_name).unwrap();
        for k in 0..40 {
            let t = -TAU + k as f64 * TAU / 13.0;
            let q = rotation::quat_from_euler(seq, [t, t / 3.0, -2.0 * t]);
            assert!((quat_norm(q) - 1.0).abs() < 1e-14, "{seq_name} at {t}");
        }
    }
}

#[test]
fn composition_is_associative() {
    let vecs = sample_rotvecs();
    for i in (0..vecs.len()).step_by(7) {
        for j in (0..vecs.len()).step_by(11) {
            for k in (0..vecs.len()).step_by(13) {
                let (p, q, r) = (
                    rotation::quat_from_rotvec(vecs[i]),
                    rotation::quat_from_rotvec(vecs[j]),
                    rotation::quat_from_rotvec(vecs[k]),
                );
                let left = rotation::compose_quat(rotation::compose_quat(p, q), r);
                let right = rotation::compose_quat(p, rotation::compose_quat(q, r));
                assert!(
                    same_rotation(left, right, 1e-13),
                    "({i},{j},{k}): {left:?} vs {right:?}"
                );
            }
        }
    }
}

#[test]
fn composition_matches_matrix_multiplication_in_the_same_order() {
    let vecs = sample_rotvecs();
    for i in (0..vecs.len()).step_by(5) {
        for j in (0..vecs.len()).step_by(9) {
            let (p, q) = (
                rotation::quat_from_rotvec(vecs[i]),
                rotation::quat_from_rotvec(vecs[j]),
            );
            let composed = rotation::matrix_from_unit_quat(rotation::compose_quat(p, q));
            let product = mat_mul(
                rotation::matrix_from_unit_quat(p),
                rotation::matrix_from_unit_quat(q),
            );
            assert!(matrices_close(composed, product, 1e-13), "({i},{j})");
        }
    }
}

#[test]
fn the_double_cover_is_a_double_cover() {
    for rotvec in sample_rotvecs() {
        let q = rotation::quat_from_rotvec(rotvec);
        let neg = [-q[0], -q[1], -q[2], -q[3]];
        // q and -q are the same rotation: same matrix, same Euler angles.
        assert!(
            matrices_close(
                rotation::matrix_from_unit_quat(q),
                rotation::matrix_from_unit_quat(neg),
                1e-15
            ),
            "{rotvec:?}"
        );
        for seq_name in SEQS {
            let seq = EulerSeq::parse(seq_name).unwrap();
            let a = rotation::euler_from_quat(q, seq).unwrap();
            let b = rotation::euler_from_quat(neg, seq).unwrap();
            assert_eq!(a.gimbal_lock, b.gimbal_lock, "{rotvec:?} {seq_name}");
            for k in 0..3 {
                let diff = (a.angles[k] - b.angles[k] + PI).rem_euclid(TAU) - PI;
                assert!(diff.abs() < 1e-12, "{rotvec:?} {seq_name} angle {k}");
            }
        }
    }
}

#[test]
fn rotvec_round_trips_through_the_quaternion() {
    for rotvec in sample_rotvecs() {
        let angle = so::get_norm(rotvec);
        let back = rotation::rotvec_from_quat(rotation::quat_from_rotvec(rotvec)).unwrap();
        assert!(finite(&back), "{rotvec:?} -> {back:?}");
        // Beyond a half turn the short-way-round representative differs by
        // construction, so compare the rotations, not the vectors.
        if angle <= PI {
            for k in 0..3 {
                assert!((back[k] - rotvec[k]).abs() < 1e-9, "{rotvec:?} -> {back:?}");
            }
        }
        assert!(
            matrices_close(
                rotation::matrix_from_unit_quat(rotation::quat_from_rotvec(rotvec)),
                rotation::matrix_from_unit_quat(rotation::quat_from_rotvec(back)),
                1e-9
            ),
            "{rotvec:?}"
        );
        assert!(
            so::get_norm(back) <= PI + 1e-12,
            "{rotvec:?} came back as {} rad",
            so::get_norm(back)
        );
    }
}

#[test]
fn euler_round_trips_within_the_branch_conventions() {
    for seq_name in SEQS {
        let seq = EulerSeq::parse(seq_name).unwrap();
        for rotvec in sample_rotvecs() {
            let q = rotation::quat_from_rotvec(rotvec);
            let decomposed = rotation::euler_from_quat(q, seq).unwrap();
            assert!(finite(&decomposed.angles), "{seq_name} {rotvec:?}");
            let recomposed = rotation::quat_from_euler(seq, decomposed.angles);
            assert!(
                same_rotation(q, recomposed, 1e-9),
                "{seq_name} {rotvec:?}: {:?} did not rebuild the rotation",
                decomposed.angles
            );
        }
    }
}

#[test]
fn the_second_euler_angle_stays_in_its_family_range() {
    for seq_name in SEQS {
        let seq = EulerSeq::parse(seq_name).unwrap();
        for rotvec in sample_rotvecs() {
            let e = rotation::euler_from_quat(rotation::quat_from_rotvec(rotvec), seq).unwrap();
            let mid = e.angles[1];
            if seq.symmetric() {
                // Proper Euler: the middle angle is a polar angle, [0, π].
                assert!(
                    (-1e-12..=PI + 1e-12).contains(&mid),
                    "{seq_name} {rotvec:?}: middle angle {mid}"
                );
            } else {
                // Tait–Bryan: the middle angle is a pitch, [-π/2, π/2].
                assert!(
                    (-PI / 2.0 - 1e-12..=PI / 2.0 + 1e-12).contains(&mid),
                    "{seq_name} {rotvec:?}: middle angle {mid}"
                );
            }
            for a in e.angles {
                assert!((-PI - 1e-12..=PI + 1e-12).contains(&a), "{seq_name}: {a}");
            }
        }
    }
}

#[test]
fn gimbal_lock_is_reported_exactly_where_the_pole_is() {
    let symmetric = EulerSeq::parse("zxz").unwrap();
    // Symmetric family: poles at the middle angle 0 and π.
    for (middle, expect_lock) in [
        (0.0, true),
        (1e-9, true),
        (1e-8, true),
        (1e-5, false),
        (0.5, false),
        (PI - 1e-9, true),
        (PI, true),
        (PI - 1e-5, false),
    ] {
        let q = rotation::quat_from_euler(symmetric, [0.4, middle, 0.9]);
        let e = rotation::euler_from_quat(q, symmetric).unwrap();
        assert_eq!(e.gimbal_lock, expect_lock, "zxz middle {middle}");
        if expect_lock {
            assert_eq!(e.angles[2], 0.0, "a lock parks everything in angle 0");
            // The rotation rebuilds to within the lock window: at the pole
            // exactly, and inside the window to about its own width, since
            // that is precisely how far the two co-axial rotations can be
            // from actually sharing an axis.
            assert!(
                same_rotation(q, rotation::quat_from_euler(symmetric, e.angles), 1e-6),
                "zxz middle {middle} did not rebuild"
            );
        }
    }

    // Tait–Bryan family: the pole is at the middle angle ±π/2.
    let tb = EulerSeq::parse("zxy").unwrap();
    for (middle, expect_lock) in [
        (PI / 2.0, true),
        (-PI / 2.0, true),
        (PI / 2.0 - 1e-9, true),
        (PI / 2.0 - 1e-4, false),
        (0.0, false),
    ] {
        let q = rotation::quat_from_euler(tb, [0.4, middle, 0.9]);
        let e = rotation::euler_from_quat(q, tb).unwrap();
        assert_eq!(e.gimbal_lock, expect_lock, "zxy middle {middle}");
        assert!(
            same_rotation(q, rotation::quat_from_euler(tb, e.angles), 1e-6),
            "zxy middle {middle} did not rebuild"
        );
    }
}

#[test]
fn near_identity_rotations_keep_their_precision() {
    // The naive sin(θ/2)/θ loses everything here; the series does not.
    for angle in [1e-16, 1e-12, 1e-9, 1e-6, 1e-4, 1e-3, 2e-3] {
        let q = rotation::quat_from_rotvec([0.0, 0.0, angle]);
        // cos(θ/2) = 1 − θ²/8 + …: the real part may only drift by that.
        assert!(
            (q[3] - 1.0).abs() <= angle * angle / 8.0 + 1e-16,
            "w drifted at {angle}: {}",
            q[3]
        );
        // sin(θ/2) = θ/2 − θ³/48 + …: the vector lane may only drift by
        // that much, which is the whole point of the series — a naive
        // sin(θ/2)/θ would lose relative precision entirely down here.
        assert!(
            (q[2] - angle / 2.0).abs() <= angle * angle * angle / 48.0 + 1e-18,
            "{angle}: z lane {} vs {}",
            q[2],
            angle / 2.0
        );
        let back = rotation::rotvec_from_quat(q).unwrap();
        assert!(
            (back[2] - angle).abs() <= 1e-15 * angle,
            "{angle} -> {} (relative error {})",
            back[2],
            (back[2] - angle).abs() / angle
        );
    }
}

#[test]
fn near_pi_rotations_are_stable() {
    let axis = so::normalize([1.0, 2.0, -0.5]);
    for delta in [1e-12, 1e-9, 1e-6, 1e-3] {
        // Just under a half turn: recovered as itself.
        let angle = PI - delta;
        let q = so::quaternion_from_angle_axis(angle, axis);
        let (back_angle, back_axis) = so::angle_axis_from_quaternion(q).unwrap();
        assert!((back_angle - angle).abs() < 1e-9, "{angle}");
        for k in 0..3 {
            assert!((back_axis[k] - axis[k]).abs() < 1e-6, "{angle} axis");
        }

        // Just over: the same rotation the short way round, i.e. the
        // complementary angle about the *negated* axis. That flip is the
        // double cover doing its job, not an instability.
        let over = PI + delta;
        let q = so::quaternion_from_angle_axis(over, axis);
        let (back_angle, back_axis) = so::angle_axis_from_quaternion(q).unwrap();
        assert!((back_angle - (TAU - over)).abs() < 1e-9, "{over}");
        for k in 0..3 {
            assert!((back_axis[k] + axis[k]).abs() < 1e-6, "{over} axis");
        }
    }
    // Exactly π: the axis sign is the double cover's free choice, but the
    // rotation must still be exactly a half turn about that line.
    let q = so::quaternion_from_angle_axis(PI, OUT);
    let m = so::rotation_matrix(PI, OUT);
    assert!((q[3].abs()) < 1e-15, "half turn has a zero real part");
    assert!(matrices_close(
        m,
        [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]],
        1e-15
    ));
}

#[test]
fn axis_angle_degeneracies_are_defined_not_nan() {
    // A zero axis is not a rotation: the Reference's normalize gives the
    // zero vector, so the result is the identity.
    assert_eq!(so::quaternion_from_angle_axis(1.5, ORIGIN), IDENTITY_QUAT);
    assert_eq!(
        so::rotation_matrix(1.5, ORIGIN),
        so::rotation_matrix(0.0, OUT)
    );
    // A zero angle about a real axis is likewise the identity.
    assert_eq!(so::quaternion_from_angle_axis(0.0, OUT), IDENTITY_QUAT);
    // The identity has no axis to report, and says so.
    assert!(so::angle_axis_from_quaternion(IDENTITY_QUAT).is_none());
    // A zero quaternion is not a rotation at all.
    assert!(rotation::normalized([0.0; 4]).is_none());
    assert!(rotation::rotvec_from_quat([0.0; 4]).is_none());
    assert!(rotation::matrix_from_quat([0.0; 4]).is_none());
    for seq_name in SEQS {
        let seq = EulerSeq::parse(seq_name).unwrap();
        assert!(rotation::euler_from_quat([0.0; 4], seq).is_none());
        // The identity decomposes to all zeros in every sequence.
        let e = rotation::euler_from_quat(IDENTITY_QUAT, seq).unwrap();
        assert_eq!(e.angles, [0.0; 3], "{seq_name}");
    }
}

#[test]
fn rotation_matrices_are_orthonormal_and_right_handed() {
    for rotvec in sample_rotvecs() {
        let m = rotation::matrix_from_unit_quat(rotation::quat_from_rotvec(rotvec));
        let product = mat_mul(
            m,
            [
                [m[0][0], m[1][0], m[2][0]],
                [m[0][1], m[1][1], m[2][1]],
                [m[0][2], m[1][2], m[2][2]],
            ],
        );
        assert!(
            matrices_close(
                product,
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                1e-14
            ),
            "{rotvec:?} is not orthonormal"
        );
        let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        assert!(
            (det - 1.0).abs() < 1e-13,
            "{rotvec:?} has determinant {det}"
        );
    }
}

#[test]
fn rotate_vector_preserves_length_and_composes() {
    let v: Vec3 = [1.0, -2.0, 0.5];
    let len = so::get_norm(v);
    for rotvec in sample_rotvecs() {
        let angle = so::get_norm(rotvec);
        let axis = if angle == 0.0 { OUT } else { rotvec };
        let rotated = so::rotate_vector(v, angle, axis);
        assert!((so::get_norm(rotated) - len).abs() < 1e-13, "{rotvec:?}");
        // Rotating by θ then −θ about the same axis is the identity.
        let back = so::rotate_vector(rotated, -angle, axis);
        for k in 0..3 {
            assert!((back[k] - v[k]).abs() < 1e-12, "{rotvec:?}: {back:?}");
        }
    }
}

#[test]
fn the_cardinal_rotations_are_what_scene_code_expects() {
    // These four are the ones every manim scene relies on by eye.
    let close = |a: Vec3, b: Vec3| (0..3).all(|k| (a[k] - b[k]).abs() < 1e-15);
    assert!(close(so::rotate_vector(RIGHT, PI / 2.0, OUT), UP));
    assert!(close(so::rotate_vector(UP, PI / 2.0, OUT), LEFT));
    assert!(close(so::rotate_vector(LEFT, PI / 2.0, OUT), DOWN));
    assert!(close(so::rotate_vector(DOWN, PI / 2.0, OUT), RIGHT));
    // A quarter turn about RIGHT takes UP into OUT (right-handed).
    assert!(close(so::rotate_vector(UP, PI / 2.0, RIGHT), OUT));
}

#[test]
fn no_space_op_emits_a_nan_on_a_degenerate_input() {
    let degenerate: [Vec3; 5] = [
        ORIGIN,
        [1e-300, 0.0, 0.0],
        [f64::MIN_POSITIVE, f64::MIN_POSITIVE, 0.0],
        OUT,
        [-0.0, -0.0, -0.0],
    ];
    for a in degenerate {
        for b in degenerate {
            assert!(finite(&so::normalize(a)), "normalize {a:?}");
            assert!(finite(&so::cross(a, b)), "cross {a:?} {b:?}");
            assert!(so::cross2d(a, b).is_finite());
            assert!(so::get_norm(a).is_finite());
            assert!(so::get_dist(a, b).is_finite());
            assert!(so::angle_of_vector(a).is_finite());
            assert!(
                so::angle_between_vectors(a, b).is_finite(),
                "angle_between {a:?} {b:?}"
            );
            assert!(finite(&so::get_unit_normal(
                a,
                b,
                so::DEFAULT_UNIT_NORMAL_TOL
            )));
            assert!(finite(&so::midpoint(a, b)));
            assert!(finite(&so::get_closest_point_on_line(
                a,
                b,
                [1.0, 1.0, 1.0]
            )));
            assert!(finite(&so::find_intersection(
                a,
                b,
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                so::DEFAULT_INTERSECTION_THRESHOLD
            )));
            assert!(so::tri_area(a, b, ORIGIN).is_finite());
            assert!(finite(&so::project_along_vector(a, so::normalize(b))));
            assert!(so::get_winding_number(&[a, b, ORIGIN]).is_finite());
            for m in [
                so::rotation_matrix(1.0, a),
                so::rotation_between_vectors(a, b),
                so::z_to_vector(a),
            ] {
                assert!(m.iter().all(|row| finite(row)), "matrix from {a:?} {b:?}");
            }
        }
    }
    assert!(so::get_winding_number(&[]).is_finite());
    assert!(so::poly_line_length(&[]).is_finite());
    assert!(finite(&so::center_of_mass(&[])));
}

#[test]
fn euler_sequences_that_are_not_rotations_are_refused() {
    for bad in [
        "", "z", "zx", "zxzz", "zXz", "Zxz", "abc", "zzz", "zzx", "xzz", "111",
    ] {
        assert!(EulerSeq::parse(bad).is_none(), "`{bad}` should be refused");
    }
    for good in SEQS {
        assert!(EulerSeq::parse(good).is_some(), "`{good}` should parse");
    }
    // Non-adjacent repeats are proper-Euler sequences and must parse.
    for good in ["xyx", "yzy", "zyz", "XZX"] {
        let seq = EulerSeq::parse(good).unwrap();
        assert!(seq.symmetric(), "{good} is a proper-Euler sequence");
    }
}
