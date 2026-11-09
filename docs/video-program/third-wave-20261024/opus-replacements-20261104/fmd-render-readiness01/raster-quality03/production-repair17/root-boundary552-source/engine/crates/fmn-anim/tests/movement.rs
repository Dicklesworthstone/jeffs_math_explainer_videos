//! Functional-map, rotation, and update-driven semantics (fm-cye, §9.4
//! family 5 + rotation.py + update.py): Homotopy's match-then-map rule,
//! PhaseFlow's deliberate path dependence, MoveAlongPath's true-arclength
//! constant speed (BN-03), Rotating's absolute per-frame pose, and the
//! live-stage update closures.

use fmn_anim::animation::Animation;
use fmn_anim::{
    AnimError, Homotopy, MaintainPositionRelativeTo, MoveAlongPath, PhaseFlow, RateFunc,
    UpdateFromFunc, complex_homotopy, rotate, smoothed_homotopy,
};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

fn vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    mob
}

/// A 2-curve straight-line vmobject with x = [0, 1, 2, 3, 4].
fn line5(stage: &mut Stage) -> Mob {
    vmob(
        stage,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ],
    )
}

fn coords(stage: &Stage, mob: Mob, lane: usize) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation)]
    stage
        .get_points(mob)
        .unwrap()
        .into_iter()
        .map(|point| point[lane] as f32)
        .collect()
}

fn assert_close(actual: &[f32], expected: &[f32], tol: f32) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= tol, "{actual:?} !~ {expected:?}");
    }
}

// -------------------------------------------------------------- Homotopy

#[test]
fn homotopy_maps_points_from_the_start_each_frame() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = Homotopy::new(|x, y, z, t| [x, y + t * x, z], mob);
    assert_eq!(anim.state().config.run_time, 3.0);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, mob, 1), &[0.0, 0.5, 1.0, 1.5, 2.0], 1e-6);
    // Absolute per-frame semantics: re-interpolating the same alpha does
    // not accumulate (match_points restores from the start first).
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, mob, 1), &[0.0, 0.5, 1.0, 1.5, 2.0], 1e-6);
    anim.finish(&mut stage);
    assert_close(&coords(&stage, mob, 1), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-6);
}

#[test]
fn complex_homotopy_maps_the_plane_and_carries_z() {
    let mut stage = Stage::new();
    let mob = vmob(&mut stage, &[[1.0, 2.0, 7.0]; 3]);
    let mut anim = complex_homotopy(|re, im, t| (re + 3.0 * t, im), mob);
    assert_eq!(anim.state().config.name, "ComplexHomotopy");
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 1.0);
    assert_close(&coords(&stage, mob, 0), &[4.0; 3], 1e-6);
    assert_close(&coords(&stage, mob, 1), &[2.0; 3], 1e-6);
    assert_close(&coords(&stage, mob, 2), &[7.0; 3], 1e-6);
}

#[test]
fn smoothed_homotopy_carries_the_reference_name() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let anim = smoothed_homotopy(|x, y, z, _| [x, y, z], mob);
    assert_eq!(anim.state().config.name, "SmoothedVectorizedHomotopy");
}

// ------------------------------------------------------------- PhaseFlow

#[test]
fn phase_flow_integrates_forward_euler() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = PhaseFlow::new(|_| [1.0, 0.0, 0.0], mob, None);
    assert_eq!(anim.state().config.run_time, 3.0);
    anim.begin(&mut stage).unwrap();
    // begin's interpolate(0) only records last_alpha.
    assert_close(&coords(&stage, mob, 0), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-6);
    // dt = virtual_time · Δα = 3 · 0.5 = 1.5.
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, mob, 0), &[1.5, 2.5, 3.5, 4.5, 5.5], 1e-5);
    // Path-dependent by design: the next step advects from where it is.
    anim.interpolate(&mut stage, 1.0);
    assert_close(&coords(&stage, mob, 0), &[3.0, 4.0, 5.0, 6.0, 7.0], 1e-5);
}

// ---------------------------------------------------------- MoveAlongPath

#[test]
fn move_along_path_is_constant_speed_by_true_arc_length() {
    let mut stage = Stage::new();
    // Two curves of very different lengths: 0→1 (length 1), 1→5 (length 4).
    let path = vmob(
        &mut stage,
        &[
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ],
    );
    let dot = vmob(&mut stage, &[[9.0, 9.0, 0.0]]);
    let mut anim = MoveAlongPath::new(dot, path);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    assert_close(&coords(&stage, dot, 0), &[0.0], 1e-6);
    // Half the *arc length* is 2.5 — the Reference's
    // quick_point_from_proportion would sit at the curve boundary (x = 1).
    // Constant speed under the original name is BN-03.
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, dot, 0), &[2.5], 1e-4);
    anim.finish(&mut stage);
    assert_close(&coords(&stage, dot, 0), &[5.0], 1e-5);
}

#[test]
fn move_along_empty_path_is_the_named_error() {
    let mut stage = Stage::new();
    let dot = vmob(&mut stage, &[[0.0; 3]]);
    let path = vmob(&mut stage, &[]);
    let mut anim = MoveAlongPath::new(dot, path);
    assert!(matches!(
        anim.begin(&mut stage),
        Err(AnimError::EmptyMobject)
    ));
}

// -------------------------------------------------------------- Rotating

#[test]
fn rotate_lands_the_absolute_pose_without_accumulating() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = rotate(mob, std::f64::consts::FRAC_PI_2);
    assert_eq!(anim.state().config.run_time, 1.0);
    anim.begin(&mut stage).unwrap();
    // Same alpha twice: the pose is absolute (restore-then-rotate), so
    // nothing accumulates.
    anim.interpolate(&mut stage, 0.5);
    let mid = coords(&stage, mob, 1);
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, mob, 1), &mid, 1e-6);
    // Full turn: 90° about the center (x = 2) sends x ∈ [0,4] to
    // y ∈ [−2,2] at x = 2.
    anim.finish(&mut stage);
    assert_close(&coords(&stage, mob, 0), &[2.0; 5], 1e-5);
    assert_close(&coords(&stage, mob, 1), &[-2.0, -1.0, 0.0, 1.0, 2.0], 1e-5);
}

// ------------------------------------------------------------ update.py

#[test]
fn update_from_func_runs_live_each_frame() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = UpdateFromFunc::new(mob, |stage, mob| {
        stage.shift(mob, [1.0, 0.0, 0.0]);
    });
    anim.begin(&mut stage).unwrap(); // interpolate(0) → one call
    anim.interpolate(&mut stage, 0.5);
    anim.interpolate(&mut stage, 0.9);
    // Three calls, three shifts — live semantics, deliberately impure.
    assert_close(&coords(&stage, mob, 0), &[3.0, 4.0, 5.0, 6.0, 7.0], 1e-6);
}

#[test]
fn update_from_alpha_func_receives_the_true_alpha() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = UpdateFromFunc::new_alpha(mob, |stage, mob, alpha| {
        stage.set_x(mob, 10.0 * alpha);
    });
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.3);
    let center = stage.get_center(mob);
    assert!((center[0] - 3.0).abs() < 1e-5);
}

#[test]
fn maintain_position_holds_the_construction_offset() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage); // center x = 2
    let tracked = vmob(&mut stage, &[[0.0, 0.0, 0.0]]);
    let mut anim = MaintainPositionRelativeTo::new(&stage, mob, tracked);
    anim.begin(&mut stage).unwrap();
    stage.shift(tracked, [1.0, 1.0, 0.0]);
    anim.interpolate(&mut stage, 0.5);
    let center = stage.get_center(mob);
    assert!((center[0] - 3.0).abs() < 1e-5, "kept diff of +2 in x");
    assert!((center[1] - 1.0).abs() < 1e-5, "followed in y");
}
