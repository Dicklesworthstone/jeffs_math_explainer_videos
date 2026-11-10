//! Parameter-driven native scenes traverse the actual builder/animation/clock.
use fmn_anim::purity::{Purity, reconstruct_pure_frame};
use fmn_anim::{RationalFrameClock, play_segment, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};

fn linear() -> AnimateArgs {
    AnimateArgs {
        rate_func: Some(rate::linear),
        ..AnimateArgs::default()
    }
}

#[test]
fn scene_updaters_observe_the_new_tracker_value_before_each_capture() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(0.0);
    let dot = stage.add(Mobject::from_points(&[[0.0; 3]]));
    stage.add_to_scene(dot).unwrap();
    stage.add_to_scene(tracker).unwrap();
    stage
        .add_updater(
            dot,
            move |stage, me| {
                let value = stage.tracker_value(tracker).unwrap();
                stage.move_to(me, [value, value * 2.0, 0.0], [0.0; 3]);
            },
            false,
        )
        .unwrap();
    let builder = tracker
        .animate()
        .set_anim_args(linear())
        .unwrap()
        .set_value(8.0)
        .unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let mut values = Vec::new();
    let report = play_segment(
        &mut stage,
        &mut RationalFrameClock::new(4).unwrap(),
        &RngRoot::from_seed(0),
        &mut animations,
        false,
        &mut |packet| {
            let frame = packet.materialize_stage();
            values.push((frame.tracker_value(tracker).unwrap(), frame.get_center(dot)));
        },
    )
    .unwrap();
    assert!(matches!(report.purity, Purity::Stateful(_)));
    assert_eq!(
        values,
        vec![
            (2.0, [2.0, 4.0, 0.0]),
            (4.0, [4.0, 8.0, 0.0]),
            (6.0, [6.0, 12.0, 0.0]),
            (8.0, [8.0, 16.0, 0.0])
        ]
    );
}

#[test]
fn exponential_and_complex_builders_interpolate_their_native_encodings() {
    let mut stage = Stage::new();
    let scalar = stage.add_exponential_value_tracker(4.0);
    let complex = stage.add_complex_value_tracker(1.0, -2.0);
    let builder = scalar
        .animate()
        .set_anim_args(linear())
        .unwrap()
        .set_value(16.0)
        .unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    assert!((stage.tracker_value(scalar).unwrap() - 8.0).abs() < 1e-12);
    animation.finish(&mut stage);
    let builder = complex
        .animate()
        .set_anim_args(linear())
        .unwrap()
        .set_complex_value(5.0, 6.0)
        .unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.25);
    assert_eq!(stage.tracker_complex_value(complex), Some((2.0, 0.0)));
    animation.finish(&mut stage);
    assert_eq!(stage.tracker_complex_value(complex), Some((5.0, 6.0)));
}

#[test]
fn tracker_only_animation_is_pure_and_reconstructs_out_of_order() {
    let mut stage = Stage::new();
    let tracker = stage.add_complex_value_tracker(1.0, 2.0);
    stage.add_to_scene(tracker).unwrap();
    let builder = tracker
        .animate()
        .set_anim_args(linear())
        .unwrap()
        .increment_complex_value(4.0, -8.0)
        .unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let root = RngRoot::from_seed(3);
    let mut serial = Vec::new();
    let report = play_segment(
        &mut stage,
        &mut RationalFrameClock::new(4).unwrap(),
        &root,
        &mut animations,
        false,
        &mut |packet| serial.push(packet),
    )
    .unwrap();
    assert_eq!(report.purity, Purity::Pure);
    let samples: Vec<_> = RationalFrameClock::new(4)
        .unwrap()
        .segment(report.run_time)
        .unwrap()
        .samples()
        .collect();
    for i in [3, 0, 2, 1] {
        let actual =
            reconstruct_pure_frame(&mut stage, &mut animations, &report, &root, &samples[i])
                .unwrap();
        assert_eq!(
            actual.materialize_stage().tracker_complex_value(tracker),
            serial[i].materialize_stage().tracker_complex_value(tracker)
        );
    }
}
