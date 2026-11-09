//! FMTL must preserve invisible tracker state as well as drawable geometry.
use fmn_anim::{Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::Stage;
use fmn_mobject::animate::AnimateArgs;
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

#[test]
fn pure_tracker_bundles_reconstruct_scalar_logarithmic_and_complex_state() {
    let mut stage = Stage::new();
    let scalar = stage.add_value_tracker(1.0);
    let exponential = stage.add_exponential_value_tracker(4.0);
    let complex = stage.add_complex_value_tracker(2.0, -3.0);
    stage
        .add_many_to_scene(&[scalar, exponential, complex])
        .unwrap();
    let args = AnimateArgs {
        rate_func: Some(rate::linear),
        ..AnimateArgs::default()
    };
    let mut animations = Vec::new();
    for builder in [
        scalar
            .animate()
            .set_anim_args(args)
            .unwrap()
            .set_value(9.0)
            .unwrap(),
        exponential
            .animate()
            .set_anim_args(args)
            .unwrap()
            .set_value(16.0)
            .unwrap(),
        complex
            .animate()
            .set_anim_args(args)
            .unwrap()
            .set_complex_value(6.0, 5.0)
            .unwrap(),
    ] {
        animations.push(prepare_animation(builder, &mut stage).unwrap());
    }
    let mut timeline = Timeline::new(4).unwrap();
    timeline.play(animations).unwrap();
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0)).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.segment_kind(0), Some(BundleSegmentKind::Pure));
    for i in [3, 0, 2, 1] {
        let frame = bundle.stage_at(i).unwrap();
        let alpha = f64::from(i + 1) / 4.0;
        let roots = frame.roots();
        assert_eq!(frame.tracker_value(roots[0]), Some(1.0 + alpha * 8.0));
        let expected = match i {
            0 => 32.0_f64.sqrt(),
            1 => 8.0,
            2 => 128.0_f64.sqrt(),
            _ => 16.0,
        };
        assert!((frame.tracker_value(roots[1]).unwrap() - expected).abs() < 1e-12);
        assert_eq!(
            frame.tracker_complex_value(roots[2]),
            Some((2.0 + 4.0 * alpha, -3.0 + 8.0 * alpha))
        );
    }
}

#[test]
fn out_and_back_tracker_motion_cannot_pass_an_endpoint_only_pure_proof() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(1.0);
    stage.add_to_scene(tracker).unwrap();
    let builder = tracker
        .animate()
        .set_anim_args(AnimateArgs {
            rate_func: Some(rate::there_and_back),
            ..AnimateArgs::default()
        })
        .unwrap()
        .set_value(9.0)
        .unwrap();
    let animation = prepare_animation(builder, &mut stage).unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    timeline.play(vec![animation]).unwrap();
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0)).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.segment_kind(0), Some(BundleSegmentKind::Stateful));
    for i in [7, 0, 3, 2, 5, 1, 4, 6] {
        let frame = bundle.stage_at(i).unwrap();
        let alpha = rate::there_and_back(f64::from(i + 1) / 8.0);
        let expected = (1.0 - alpha) + alpha * 9.0;
        assert_eq!(frame.tracker_value(frame.roots()[0]), Some(expected));
    }
}
