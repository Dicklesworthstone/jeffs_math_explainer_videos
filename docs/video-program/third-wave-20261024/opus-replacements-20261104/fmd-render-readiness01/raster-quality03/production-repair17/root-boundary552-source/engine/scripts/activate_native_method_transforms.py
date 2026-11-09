#!/usr/bin/env python3
"""Activate complete native method transforms with exact source guards.

All edits are validated before writing. The new carrier uses Transform's
existing data plane; no second field interpolator or frame scheduler is added.
"""
from pathlib import Path
import hashlib

pending = {}

def read(name):
    return pending.setdefault(name, Path(name).read_text())

def replace(name, old, new):
    text = read(name)
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit(f"changed source anchor in {name}; refusing to overwrite")
    pending[name] = text.replace(old, new, 1)

def between(name, start, end, digest, new):
    text = read(name)
    if start not in text and new in text:
        return
    if text.count(start) != 1:
        raise SystemExit(f"changed method carrier in {name}")
    a = text.index(start)
    z = text.index(end, a)
    if hashlib.sha256(text[a:z].encode()).hexdigest() != digest:
        raise SystemExit(f"method carrier changed in {name}; refusing to overwrite")
    pending[name] = text[:a] + new + text[z:]

between('crates/fmn-anim/src/animation.rs', '/// The concrete animation a built `.animate` chain becomes (the\n/// Referen', '// ----------------------------------------------------- prepare_animation', 'c7c67b4a8ca56e75ce4dbe7690bc1f138fc94c152f6ca3e49048836950f38859', '#[path = "method_animation.rs"]\nmod method_animation;\npub use method_animation::MethodAnimation;\n\n')
replace('crates/fmn-anim/src/animation.rs', "//!   chains (the Reference's `_MethodAnimation`): source → target record\n//!   lerp over families that are structurally aligned *by construction*\n//!   (the target is a build-time copy of the source). Heterogeneous-pair\n//!   alignment (`align_data`) and `path_arc` arcs are the Transform\n//!   family's, arriving with fm-cye; until then a path-arc request or a\n//!   structurally diverged pair is a precise, named error — never garbage.\n", "//!   chains (the Reference's `_MethodAnimation`), delegated to the same\n//!   Transform mechanism as explicit transforms: heterogeneous family\n//!   alignment, curved point paths, numeric uniforms, matching-data locks,\n//!   and host-view materialization all have one implementation.\n")
replace('crates/fmn-anim/src/animation.rs', 'use fmn_mobject::{AnimBuilder, BuiltAnimate, Mob, Placement, Stage, StageError};\n', 'use fmn_mobject::{AnimBuilder, BuiltAnimate, Mob, Stage, StageError};\n')
replace('crates/fmn-anim/src/animation.rs', "    /// A `.animate` pair whose source and target have structurally\n    /// diverged since build (family shape or record length). Alignment of\n    /// heterogeneous pairs is `align_data` — the Transform family's\n    /// mechanism, arriving with fm-cye.\n", "    /// Legacy alignment refusal retained for source compatibility.\n    /// Method animations now use Transform's heterogeneous alignment.\n")
replace('crates/fmn-anim/src/animation.rs', "    /// `path_arc` was recorded on the chain: arc paths are the Transform\n    /// family's `path_func` mechanism, arriving with fm-cye. Named, never\n    /// silently a straight line.\n", "    /// Legacy arc-path refusal retained for source compatibility.\n    /// Method animations now support Transform's native arc paths.\n")
replace('crates/fmn-anim/src/animation.rs', '    /// not part of this surface — [`MethodAnimation::new`] rejects it by\n    /// name until fm-cye.\n', '    /// not part of this surface — [`MethodAnimation::new`] passes it to\n    /// the shared Transform path mechanism, together with its axis.\n')
replace('crates/fmn-mobject/src/animate.rs', '    pub path_arc: Option<f64>,\n', '    pub path_arc: Option<f64>,\n    /// Axis for the arc path; defaults to OUT. A zero vector also means OUT.\n    pub path_arc_axis: Option<Vec3>,\n')
replace('crates/fmn-mobject/src/animate.rs', '            && self.path_arc == other.path_arc\n', '            && self.path_arc == other.path_arc\n            && self.path_arc_axis == other.path_arc_axis\n')
replace('crates/fmn-scene/src/timeline_bundle.rs', '/// catalog rate function. The path is nominated [`PathFunc::Straight`]:\n/// every `.animate` method animation is straight by construction\n/// (`path_arc` is a named refusal there), and anything that is secretly\n/// not straight fails the proof and falls back to kind 1.\n', '/// catalog rate function. The path is nominated [`PathFunc::Straight`].\n/// Curved method animations are pure for native reconstruction, but only\n/// qualify for this serialized law when the frame-by-frame proof succeeds;\n/// otherwise their actual captured frames are preserved as kind 1.\n')

between('crates/fmn-anim/tests/animation.rs', '#[test]\nfn path_arc_on_a_built_chain_is_a_named_error() {\n    let mut stag', '#[test]\nfn stale_target_at_begin_is_a_named_error()', 'ee1ac1ec048d74c9de76f09d9c76b8b7bb0d03a3aa52427670fee63a0bf85082', r'''#[test]
fn path_arc_on_a_built_chain_uses_transform_motion() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let built = mob
        .animate()
        .set_anim_args(AnimateArgs {
            path_arc: Some(std::f64::consts::PI),
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .and_then(|b| b.shift([2.0, 0.0, 0.0]))
        .and_then(|b| b.build(&mut stage))
        .expect("builds");
    let mut anim = MethodAnimation::new(built).expect("wraps");
    anim.begin(&mut stage).expect("begin");
    anim.interpolate(&mut stage, 0.5);
    let center = stage.get_center(mob);
    assert!((center[0] - 1.0).abs() < 1e-9);
    assert!((center[1] + 1.0).abs() < 1e-9);
    anim.finish(&mut stage);
    assert!((stage.get_center(mob)[0] - 2.0).abs() < 1e-9);
}

#[test]
fn structurally_diverged_pair_is_aligned_without_mutating_the_target() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let built = mob
        .animate()
        .shift([1.0, 0.0, 0.0])
        .and_then(|b| b.build(&mut stage))
        .expect("builds");
    let target = built.target;
    let target_points = stage.get_points(target).expect("points");
    let extra = square(&mut stage);
    stage.attach(mob, extra).expect("attach");

    let mut anim = MethodAnimation::new(built).expect("wraps");
    anim.begin(&mut stage).expect("heterogeneous alignment");
    assert_eq!(stage.family(target), vec![target]);
    assert_eq!(stage.get_points(target).unwrap(), target_points);
    anim.finish(&mut stage);
    assert_eq!(stage.get_points(mob).unwrap(), target_points);
    assert_eq!(stage.family(mob).len(), 2);
}

''')

new_files = {
'crates/fmn-anim/src/method_animation.rs': r'''//! Complete native method animations over the shared Transform mechanism.
use super::{AnimConfig, AnimError, AnimState, Animation, AnimationSignature};
use crate::transform::Transform;
use fmn_mobject::{BuiltAnimate, Mob, Stage};

/// A built `.animate` recording using the complete Transform mechanism.
///
/// The recording still resolves its target at build time. At begin, Transform
/// aligns a private target when necessary and freezes the starting family.
/// Sharing these hooks also gives method animations point-path interpolation,
/// numeric uniforms/trackers, matching-field locks, and correct host-view
/// materialization without a second, reduced implementation of Transform.
#[derive(Debug, Clone)]
pub struct MethodAnimation {
    transform: Transform,
    target: Mob,
}

impl MethodAnimation {
    /// Wrap a built recording, retaining its path and lifecycle arguments.
    /// Handle and schema validation occurs at prepare/begin, as for Transform.
    pub fn new(built: BuiltAnimate) -> Result<Self, AnimError> {
        let mut config = AnimConfig::from_animate_args(&built.args);
        if config.name.is_empty() {
            config.name = "MethodAnimation".to_owned();
        }
        let transform = Transform::new(built.source, built.target)
            .with_config(config)
            .with_path_arc(
                built.args.path_arc.unwrap_or(0.0),
                built.args.path_arc_axis.unwrap_or(fmn_core::constants::OUT),
            );
        Ok(Self {
            transform,
            target: built.target,
        })
    }

    /// The original build-time target; alignment never mutates this family.
    #[must_use]
    pub fn target(&self) -> Mob {
        self.target
    }
}

impl Animation for MethodAnimation {
    fn state(&self) -> &AnimState {
        self.transform.state()
    }

    fn state_mut(&mut self) -> &mut AnimState {
        self.transform.state_mut()
    }

    fn effect_signature(&self) -> AnimationSignature {
        self.transform.effect_signature()
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        self.transform.setup(stage)
    }

    fn create_starting_mobject(&self, stage: &mut Stage) -> Result<Mob, AnimError> {
        self.transform.create_starting_mobject(stage)
    }

    fn after_begin(&mut self, stage: &mut Stage) {
        self.transform.after_begin(stage);
    }

    fn teardown(&mut self, stage: &mut Stage) {
        self.transform.teardown(stage);
    }

    fn clean_up_from_scene(&mut self, stage: &mut Stage) {
        self.transform.clean_up_from_scene(stage);
    }

    fn all_mobjects(&self) -> Vec<Mob> {
        self.transform.all_mobjects()
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        self.transform.preflight_mobjects()
    }

    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        self.transform.interpolate_submobject(stage, mobs, sub_alpha);
    }
}

''',
'crates/fmn-anim/tests/method_transform.rs': r'''//! Native .animate uses the complete Transform data plane, including after
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
        assert!((actual[axis] - expected[axis]).abs() < 1e-6, "{actual:?} != {expected:?}");
    }
}

#[test]
fn explicit_axis_curves_the_native_builder_in_the_xz_plane() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let revision = stage.get(mob).unwrap().buffer.field_revision("point");
    let builder = mob.animate().set_anim_args(AnimateArgs {
        path_arc: Some(std::f64::consts::PI),
        path_arc_axis: Some([0.0, 1.0, 0.0]),
        rate_func: Some(rate::linear),
        ..AnimateArgs::default()
    }).unwrap().shift([2.0, 0.0, 0.0]).unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    close(stage.get_center(mob), [1.0, 0.0, 1.0]);
    assert_eq!(stage.get(mob).unwrap().buffer.field_revision("point"), revision);
    animation.finish(&mut stage);
    close(stage.get_center(mob), [2.0, 0.0, 0.0]);
}

#[test]
fn a_released_host_view_does_not_compound_an_intermediate_placement() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let start = stage.get_points(mob).unwrap();
    let builder = mob.animate().set_anim_args(AnimateArgs {
        rate_func: Some(rate::linear),
        ..AnimateArgs::default()
    }).unwrap().shift([2.0, 0.0, 0.0]).unwrap();
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
    stage.set_points(target, &[[4.0, -1.0, 0.0], [8.0, 1.0, 0.0]]).unwrap();
    stage.shift(source, [3.0, 2.0, 0.0]);
    stage.shift(target, [7.0, -4.0, 0.0]);
    let target_before = stage.get_points(target).unwrap();
    let mut animation = MethodAnimation::new(BuiltAnimate {
        source, target, overridden: None,
        args: AnimateArgs { rate_func: Some(rate::linear), ..AnimateArgs::default() },
    }).unwrap();
    animation.begin(&mut stage).unwrap();
    let rows = animation.all_mobjects();
    let a = stage.get_points(rows[1]).unwrap();
    let b = stage.get_points(rows[2]).unwrap();
    assert_eq!(a.len(), b.len());
    animation.interpolate(&mut stage, 0.25);
    for ((actual, a), b) in stage.get_points(source).unwrap().into_iter().zip(a).zip(b) {
        close(actual, std::array::from_fn(|axis| 0.75 * a[axis] + 0.25 * b[axis]));
    }
    animation.finish(&mut stage);
    assert_eq!(stage.get_points(target).unwrap(), target_before, "user target is immutable");
}

#[test]
fn method_animations_interpolate_trackers_and_uniforms_and_unlock_on_abort() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(2.0);
    let target = stage.add_value_tracker(10.0);
    stage.get_mut(target).unwrap().uniforms_mut().shading = [0.4, 0.8, 0.2];
    let initial_shading = stage.get(source).unwrap().uniforms().shading;
    let mut animation = MethodAnimation::new(BuiltAnimate {
        source, target, overridden: None,
        args: AnimateArgs { rate_func: Some(rate::linear), ..AnimateArgs::default() },
    }).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.25);
    assert_eq!(stage.tracker_value(source), Some(4.0));
    let actual = stage.get(source).unwrap().uniforms().shading;
    let target_shading = [0.4, 0.8, 0.2];
    close(actual, std::array::from_fn(|i| 0.75 * initial_shading[i] + 0.25 * target_shading[i]));
    animation.abort(&mut stage);
    assert_eq!(stage.tracker_value(source), Some(4.0), "abort must not land the endpoint");

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
    let builder = mob.animate().set_anim_args(AnimateArgs {
        path_arc: Some(-std::f64::consts::FRAC_PI_2),
        rate_func: Some(rate::linear),
        ..AnimateArgs::default()
    }).unwrap().shift([2.0, 1.0, 0.0]).unwrap().set_opacity(0.25).unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let root = RngRoot::from_seed(42);
    let mut clock = RationalFrameClock::new(8).unwrap();
    let mut serial = Vec::new();
    let report = play_segment(&mut stage, &mut clock, &root, &mut animations, false,
        &mut |packet| serial.push(packet)).unwrap();
    assert_eq!(report.purity, Purity::Pure);
    let samples: Vec<_> = RationalFrameClock::new(8).unwrap()
        .segment(report.run_time).unwrap().samples().collect();
    for index in [7, 0, 4, 1, 5, 2, 6, 3] {
        let packet = reconstruct_pure_frame(&mut stage, &mut animations, &report, &root,
            &samples[index]).unwrap();
        let expected = serial[index].materialize_stage();
        let actual = packet.materialize_stage();
        assert_eq!(actual.snapshot().to_bytes().unwrap(), expected.snapshot().to_bytes().unwrap());
    }
}
''',
'crates/fmn-scene/tests/method_bundle.rs': r'''//! Arc method animations must not be mislabeled as straight FMTL segments.
use fmn_anim::{PathFunc, Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

#[test]
fn curved_method_animation_exports_actual_frames_instead_of_a_chord() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[
        [-0.5, -0.5, 0.0], [0.5, 0.5, 0.0],
    ]));
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
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0))
        .expect("export");
    let bundle = TimelineBundle::from_bytes(&bytes).expect("reader");
    assert_eq!(bundle.frame_count(), 8);
    assert_eq!(bundle.segment_kind(0), Some(BundleSegmentKind::Stateful));
    for index in [7, 0, 3, 1, 6, 2, 5, 4] {
        let frame = bundle.stage_at(index).expect("seek");
        let alpha = f64::from(index + 1) / 8.0;
        let expected = PathFunc::from_path_arc(std::f64::consts::PI, [0.0, 0.0, 1.0])
            .eval(start, [start[0] + 2.0, start[1], start[2]], alpha);
        let actual = frame.get_center(frame.roots()[0]);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1e-6);
        }
    }
}

''',
}
for name, content in new_files.items():
    path = Path(name)
    if path.exists() and path.read_text() != content:
        raise SystemExit(f"new source already exists with different content: {name}")
    pending[name] = content
for name, text in pending.items():
    if not Path(name).exists() or Path(name).read_text() != text:
        Path(name).parent.mkdir(parents=True, exist_ok=True)
        Path(name).write_text(text)
