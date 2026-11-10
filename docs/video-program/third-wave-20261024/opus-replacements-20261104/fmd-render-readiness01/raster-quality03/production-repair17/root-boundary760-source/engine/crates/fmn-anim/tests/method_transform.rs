//! Native .animate uses the complete Transform data plane, including after
//! host-side geometry materialization and under out-of-order reconstruction.
use fmn_anim::purity::{Purity, reconstruct_pure_frame};
use fmn_anim::{Animation, MethodAnimation, RationalFrameClock, play_segment, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{BuiltAnimate, Mob, Mobject, Stage};

fn square(stage: &mut Stage) -> Mob {
    stage.add(Mobject::from_points(&[
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ]))
}

fn close(actual: [f64; 3], expected: [f64; 3]) {
    for axis in 0..3 {
        assert!(
            (actual[axis] - expected[axis]).abs() < 1e-6,
            "{actual:?} != {expected:?}"
        );
    }
}

#[test]
fn explicit_axis_curves_the_native_builder_in_the_xz_plane() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let revision = stage.get(mob).unwrap().buffer.field_revision("point");
    let builder = mob
        .animate()
        .set_anim_args(AnimateArgs {
            path_arc: Some(std::f64::consts::PI),
            path_arc_axis: Some([0.0, 1.0, 0.0]),
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .unwrap()
        .shift([2.0, 0.0, 0.0])
        .unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    close(stage.get_center(mob), [1.0, 0.0, 1.0]);
    assert_eq!(
        stage.get(mob).unwrap().buffer.field_revision("point"),
        revision
    );
    animation.finish(&mut stage);
    close(stage.get_center(mob), [2.0, 0.0, 0.0]);
}

#[test]
fn a_released_host_view_does_not_compound_an_intermediate_placement() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let start = stage.get_points(mob).unwrap();
    let builder = mob
        .animate()
        .set_anim_args(AnimateArgs {
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .unwrap()
        .shift([2.0, 0.0, 0.0])
        .unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    // Record access can bake the placement, then release its view before the
    // next frame. It must not become a new interpolation origin.
    stage.bake_placement(mob).unwrap();
    animation.interpolate(&mut stage, 0.75);
    for (actual, original) in stage.get_points(mob).unwrap().into_iter().zip(&start) {
        close(actual, [original[0] + 1.5, original[1], original[2]]);
    }
    animation.finish(&mut stage);
    close(stage.get_center(mob), [2.0, 0.0, 0.0]);
}

#[test]
fn different_local_geometry_is_baked_before_interpolation() {
    let mut stage = Stage::new();
    let source = square(&mut stage);
    let target = square(&mut stage);
    stage
        .set_points(target, &[[4.0, -1.0, 0.0], [8.0, 1.0, 0.0]])
        .unwrap();
    stage.shift(source, [3.0, 2.0, 0.0]);
    stage.shift(target, [7.0, -4.0, 0.0]);
    let target_before = stage.get_points(target).unwrap();
    let mut animation = MethodAnimation::new(BuiltAnimate {
        source,
        target,
        overridden: None,
        args: AnimateArgs {
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        },
    })
    .unwrap();
    animation.begin(&mut stage).unwrap();
    let rows = animation.all_mobjects();
    let a = stage.get_points(rows[1]).unwrap();
    let b = stage.get_points(rows[2]).unwrap();
    assert_eq!(a.len(), b.len());
    animation.interpolate(&mut stage, 0.25);
    for ((actual, a), b) in stage.get_points(source).unwrap().into_iter().zip(a).zip(b) {
        close(
            actual,
            std::array::from_fn(|axis| 0.75 * a[axis] + 0.25 * b[axis]),
        );
    }
    animation.finish(&mut stage);
    assert_eq!(
        stage.get_points(target).unwrap(),
        target_before,
        "user target is immutable"
    );
}

#[test]
fn method_animations_interpolate_trackers_and_uniforms_and_unlock_on_abort() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(2.0);
    let target = stage.add_value_tracker(10.0);
    stage.get_mut(target).unwrap().uniforms_mut().shading = [0.4, 0.8, 0.2];
    let initial_shading = stage.get(source).unwrap().uniforms().shading;
    let mut animation = MethodAnimation::new(BuiltAnimate {
        source,
        target,
        overridden: None,
        args: AnimateArgs {
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        },
    })
    .unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.25);
    assert_eq!(stage.tracker_value(source), Some(4.0));
    let actual = stage.get(source).unwrap().uniforms().shading;
    let target_shading = [0.4, 0.8, 0.2];
    close(
        actual,
        std::array::from_fn(|i| 0.75 * initial_shading[i] + 0.25 * target_shading[i]),
    );
    animation.abort(&mut stage);
    assert_eq!(
        stage.tracker_value(source),
        Some(4.0),
        "abort must not land the endpoint"
    );

    let source = square(&mut stage);
    let builder = source.animate().shift([1.0, 0.0, 0.0]).unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    assert!(stage.get(source).unwrap().buffer.is_locked("point"));
    animation.abort(&mut stage);
    assert!(!stage.get(source).unwrap().buffer.is_locked("point"));
}

#[test]
fn curved_method_frames_reconstruct_bit_exactly_out_of_order() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).unwrap();
    let builder = mob
        .animate()
        .set_anim_args(AnimateArgs {
            path_arc: Some(-std::f64::consts::FRAC_PI_2),
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .unwrap()
        .shift([2.0, 1.0, 0.0])
        .unwrap()
        .set_opacity(0.25)
        .unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let root = RngRoot::from_seed(42);
    let mut clock = RationalFrameClock::new(8).unwrap();
    let mut serial = Vec::new();
    let report = play_segment(
        &mut stage,
        &mut clock,
        &root,
        &mut animations,
        false,
        &mut |packet| serial.push(packet),
    )
    .unwrap();
    assert_eq!(report.purity, Purity::Pure);
    let samples: Vec<_> = RationalFrameClock::new(8)
        .unwrap()
        .segment(report.run_time)
        .unwrap()
        .samples()
        .collect();
    for index in [7, 0, 4, 1, 5, 2, 6, 3] {
        let packet =
            reconstruct_pure_frame(&mut stage, &mut animations, &report, &root, &samples[index])
                .unwrap();
        let expected = serial[index].materialize_stage();
        let actual = packet.materialize_stage();
        assert_eq!(
            actual.snapshot().to_bytes().unwrap(),
            expected.snapshot().to_bytes().unwrap()
        );
    }
}
