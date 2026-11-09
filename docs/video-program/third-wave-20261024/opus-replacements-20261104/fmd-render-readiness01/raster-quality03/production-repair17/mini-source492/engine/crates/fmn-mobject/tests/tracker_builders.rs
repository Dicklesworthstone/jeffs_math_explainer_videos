//! Deferred tracker targets use Stage's existing f64 scalar/log/complex model.
use fmn_mobject::animate::{AnimateArgs, AnimateError, OverrideAnimation};
use fmn_mobject::{Mobject, Stage};

#[test]
fn scalar_increments_resolve_at_build_time_and_leave_the_source_untouched() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(1.0);
    let builder = source
        .animate()
        .increment_value(2.0)
        .unwrap()
        .shift([1.0, 2.0, 0.0])
        .unwrap()
        .increment_value(4.0)
        .unwrap();
    stage.set_tracker_value(source, 10.0).unwrap();
    let built = builder.build(&mut stage).unwrap();
    assert_eq!(stage.tracker_value(source), Some(10.0));
    assert_eq!(stage.tracker_value(built.target), Some(16.0));
    assert_eq!(
        stage.placement(built.target).unwrap().translation(),
        [1.0, 2.0, 0.0]
    );
    assert!(
        stage.roots().is_empty(),
        "target construction must not add scene roots"
    );
}

#[test]
fn scalar_set_and_increment_follow_recording_order_and_keep_double_precision() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(0.0);
    let value = 1.0 + f64::EPSILON;
    let built = source
        .animate()
        .set_value(3.0)
        .unwrap()
        .increment_value(5.0)
        .unwrap()
        .set_value(value)
        .unwrap()
        .build(&mut stage)
        .unwrap();
    assert_eq!(stage.tracker_value(built.target), Some(value));
    assert_eq!(stage.tracker_value(source), Some(0.0));
}

#[test]
fn exponential_commands_increment_decoded_values_not_logarithmic_lanes() {
    let mut stage = Stage::new();
    let source = stage.add_exponential_value_tracker(2.0);
    let built = source
        .animate()
        .set_value(4.0)
        .unwrap()
        .increment_value(12.0)
        .unwrap()
        .build(&mut stage)
        .unwrap();
    assert!((stage.tracker_value(built.target).unwrap() - 16.0).abs() < 1e-12);
    assert!((stage.tracker_value(source).unwrap() - 2.0).abs() < 1e-12);
}

#[test]
fn complex_commands_keep_both_lanes_and_defer_increments() {
    let mut stage = Stage::new();
    let source = stage.add_complex_value_tracker(1.0, -2.0);
    let builder = source.animate().increment_complex_value(3.0, 4.0).unwrap();
    stage.set_tracker_complex_value(source, 5.0, -6.0).unwrap();
    let built = builder.build(&mut stage).unwrap();
    assert_eq!(stage.tracker_complex_value(built.target), Some((8.0, -2.0)));
    let built = source
        .animate()
        .set_complex_value(2.0, -3.0)
        .unwrap()
        .increment_complex_value(4.0, 1.0)
        .unwrap()
        .build(&mut stage)
        .unwrap();
    assert_eq!(stage.tracker_complex_value(built.target), Some((6.0, -2.0)));
    assert_eq!(stage.tracker_complex_value(source), Some((5.0, -6.0)));
}

#[test]
fn incompatible_tracker_commands_refuse_the_whole_chain_before_allocating() {
    let mut stage = Stage::new();
    let ordinary = stage.add(Mobject::from_points(&[[0.0; 3]]));
    let scalar = stage.add_value_tracker(1.0);
    let complex = stage.add_complex_value_tracker(2.0, 3.0);
    for builder in [
        ordinary
            .animate()
            .shift([1.0; 3])
            .unwrap()
            .set_value(4.0)
            .unwrap(),
        scalar
            .animate()
            .set_value(5.0)
            .unwrap()
            .set_complex_value(1.0, 2.0)
            .unwrap(),
        complex.animate().increment_value(3.0).unwrap(),
        ordinary
            .animate()
            .increment_complex_value(1.0, 2.0)
            .unwrap(),
    ] {
        let before = stage.snapshot().to_bytes().unwrap();
        assert!(matches!(
            builder.build(&mut stage),
            Err(AnimateError::TrackerKindMismatch { .. })
        ));
        assert_eq!(stage.snapshot().to_bytes().unwrap(), before);
    }
}

#[test]
fn tracker_recordings_obey_the_same_override_and_argument_rules() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(1.0);
    let override_animation = OverrideAnimation {
        name: "tracker override",
    };
    assert_eq!(
        source
            .animate()
            .with_override(override_animation)
            .unwrap()
            .set_value(2.0),
        Err(AnimateError::OverrideNotChainable)
    );
    assert_eq!(
        source
            .animate()
            .increment_value(2.0)
            .unwrap()
            .with_override(override_animation),
        Err(AnimateError::OverrideNotChainable)
    );
    assert_eq!(
        source
            .animate()
            .set_anim_args(AnimateArgs::default())
            .unwrap()
            .set_value(2.0)
            .unwrap()
            .set_anim_args(AnimateArgs::default()),
        Err(AnimateError::ArgsAlreadySet)
    );
}
