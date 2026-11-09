//! Arc method animations must not be mislabeled as straight FMTL segments.
use fmn_anim::{PathFunc, Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

#[test]
fn curved_method_animation_exports_actual_frames_instead_of_a_chord() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]]));
    stage.add_to_scene(mob).expect("root");
    let start = stage.get_center(mob);
    let animation = mob
        .animate()
        .set_anim_args(AnimateArgs {
            path_arc: Some(std::f64::consts::PI),
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .and_then(|b| b.shift([2.0, 0.0, 0.0]))
        .expect("record");
    let animation = prepare_animation(animation, &mut stage).expect("prepare");
    let mut timeline = Timeline::new(8).expect("fps");
    timeline.play(vec![animation]).expect("play");
    let bytes =
        export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0)).expect("export");
    let bundle = TimelineBundle::from_bytes(&bytes).expect("reader");
    assert_eq!(bundle.frame_count(), 8);
    assert_eq!(bundle.segment_kind(0), Some(BundleSegmentKind::Stateful));
    for index in [7, 0, 3, 1, 6, 2, 5, 4] {
        let frame = bundle.stage_at(index).expect("seek");
        let alpha = f64::from(index + 1) / 8.0;
        let expected = PathFunc::from_path_arc(std::f64::consts::PI, [0.0, 0.0, 1.0]).eval(
            start,
            [start[0] + 2.0, start[1], start[2]],
            alpha,
        );
        let actual = frame.get_center(frame.roots()[0]);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1e-6);
        }
    }
}
