#!/usr/bin/env python3
"""Wire deferred tracker commands to the Stage tracker authority and FMTL."""
from pathlib import Path
name = 'crates/fmn-mobject/src/animate.rs'
text = Path(name).read_text()
def replace(old, new):
    global text
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit('tracker builder source changed: ' + old[:90])
    text = text.replace(old, new, 1)
replace('    SetHeight(f64, bool),\n', '''    SetHeight(f64, bool),
    /// Set a scalar tracker (plain or exponential) through its encoding.
    SetValue(f64),
    /// Increment a scalar tracker's decoded value at build time.
    IncrementValue(f64),
    /// Set both lanes of a complex tracker.
    SetComplexValue(f64, f64),
    /// Increment a complex tracker's real and imaginary components.
    IncrementComplexValue(f64, f64),
''')
replace('    StaleHandle(Mob),\n', '''    StaleHandle(Mob),
    /// A live source lacks the tracker encoding required by a command.
    TrackerKindMismatch {
        /// The live, but incompatible, source.
        source: Mob,
        /// The required decoded value type (scalar or complex).
        expected: &'static str,
    },
''')
replace('            Self::StaleHandle(_) => write!(f, "animate target handle is stale"),\n', '''            Self::StaleHandle(_) => write!(f, "animate target handle is stale"),
            Self::TrackerKindMismatch { expected, .. } => {
                write!(f, "animate command requires a {expected} value tracker")
            }
''')
replace('    /// Realize the recording: generate the target copy NOW (dynamic target\n', '''    /// Record `set_value` for a plain or exponential ValueTracker.
    /// The source is untouched; the target receives the decoded value at build.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override. The tracker
    /// kind is validated when the recording is built, before Stage mutation.
    pub fn set_value(self, value: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetValue(value))
    }

    /// Record a scalar increment, resolved against the target's decoded value
    /// at build time rather than the source's value when this call is recorded.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn increment_value(self, delta: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::IncrementValue(delta))
    }

    /// Record `set_value` for a complex tracker, as real and imaginary lanes.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_complex_value(self, real: f64, imaginary: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetComplexValue(real, imaginary))
    }

    /// Record an increment of a complex tracker's decoded components.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn increment_complex_value(self, real: f64, imaginary: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::IncrementComplexValue(real, imaginary))
    }

    /// Realize the recording: generate the target copy NOW (dynamic target
''')
replace('    /// inside a command.\n', '    /// inside a command; [`AnimateError::TrackerKindMismatch`] for a scalar\n    /// command on a non-scalar source or a complex command on a non-complex one.\n')
replace('        for command in &self.commands {\n            if let AnimateCommand::MoveTo', '''        for command in &self.commands {
            let expected = match command {
                AnimateCommand::SetValue(_) | AnimateCommand::IncrementValue(_)
                    if stage.tracker_value(self.source).is_none() => Some("scalar"),
                AnimateCommand::SetComplexValue(_, _) | AnimateCommand::IncrementComplexValue(_, _)
                    if stage.tracker_complex_value(self.source).is_none() => Some("complex"),
                _ => None,
            };
            if let Some(expected) = expected {
                return Err(AnimateError::TrackerKindMismatch { source: self.source, expected });
            }
            if let AnimateCommand::MoveTo''')
replace('            apply(stage, target, *command);\n', '            apply(stage, target, *command)?;\n')
replace('fn apply(stage: &mut Stage, target: Mob, command: AnimateCommand) {\n', 'fn apply(stage: &mut Stage, target: Mob, command: AnimateCommand) -> Result<(), AnimateError> {\n')
replace('''        AnimateCommand::SetHeight(h, s) => {
            stage.set_height(target, h, s);
        }
    }
}
''', '''        AnimateCommand::SetHeight(h, s) => {
            stage.set_height(target, h, s);
        }
        AnimateCommand::SetValue(value) => {
            stage.set_tracker_value(target, value)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::IncrementValue(delta) => {
            stage.increment_tracker_value(target, delta)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::SetComplexValue(real, imaginary) => {
            stage.set_tracker_complex_value(target, real, imaginary)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::IncrementComplexValue(real, imaginary) => {
            stage.increment_tracker_complex_value(target, real, imaginary)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
    }
    Ok(())
}
''')
pending = {name: text}
def patch_file(name, old, new):
    text = pending.get(name, Path(name).read_text())
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit('tracker integration source changed: ' + name)
    pending[name] = text.replace(old, new, 1)
patch_file('crates/fmn-mobject/src/dynamics.rs', '    // ---------------------------------------------- closure-clock binding\n', '''    /// Increment a complex tracker's decoded real and imaginary components.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] for a dead or non-complex tracker.
    pub fn increment_tracker_complex_value(
        &mut self,
        mob: Mob,
        real: f64,
        imaginary: f64,
    ) -> Result<(), StageError> {
        let (re, im) = self.tracker_complex_value(mob).ok_or(StageError::StaleHandle)?;
        self.set_tracker_complex_value(mob, re + real, im + imaginary)
    }

    // ---------------------------------------------- closure-clock binding
''')
patch_file('crates/fmn-anim/src/bundle.rs', '    if a.uniforms() != b.uniforms() {\n', '''    if stage.tracker(mob) != end_stage.tracker(mob) {
        return false;
    }
    if a.uniforms() != b.uniforms() {
''')
patch_file('crates/fmn-scene/src/timeline_bundle.rs', '            let left_fields = left_entry.buffer.schema().fields();\n', '''            // Invisible ValueTrackers still carry observable scene state.
            // Omitting them lets an out-and-back parameter animation pass a
            // false pure proof and disappear from the exported timeline.
            match (left.tracker(left_mob), right.tracker(right_mob)) {
                (None, None) => {}
                (Some(a), Some(b)) if a.kind == b.kind
                    && a.lanes.into_iter().zip(b.lanes)
                        .all(|(a, b)| canonical_f64_equal(a, b)) => {}
                _ => return false,
            }
            let left_fields = left_entry.buffer.schema().fields();
''')
new_files = {
'crates/fmn-mobject/tests/tracker_builders.rs': r'''//! Deferred tracker targets use Stage's existing f64 scalar/log/complex model.
use fmn_mobject::animate::{AnimateArgs, AnimateError, OverrideAnimation};
use fmn_mobject::{Mobject, Stage};

#[test]
fn scalar_increments_resolve_at_build_time_and_leave_the_source_untouched() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(1.0);
    let builder = source.animate().increment_value(2.0).unwrap()
        .shift([1.0, 2.0, 0.0]).unwrap().increment_value(4.0).unwrap();
    stage.set_tracker_value(source, 10.0).unwrap();
    let built = builder.build(&mut stage).unwrap();
    assert_eq!(stage.tracker_value(source), Some(10.0));
    assert_eq!(stage.tracker_value(built.target), Some(16.0));
    assert_eq!(stage.placement(built.target).unwrap().translation(), [1.0, 2.0, 0.0]);
    assert!(stage.roots().is_empty(), "target construction must not add scene roots");
}

#[test]
fn scalar_set_and_increment_follow_recording_order_and_keep_double_precision() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(0.0);
    let value = 1.0 + f64::EPSILON;
    let built = source.animate().set_value(3.0).unwrap().increment_value(5.0).unwrap()
        .set_value(value).unwrap().build(&mut stage).unwrap();
    assert_eq!(stage.tracker_value(built.target), Some(value));
    assert_eq!(stage.tracker_value(source), Some(0.0));
}

#[test]
fn exponential_commands_increment_decoded_values_not_logarithmic_lanes() {
    let mut stage = Stage::new();
    let source = stage.add_exponential_value_tracker(2.0);
    let built = source.animate().set_value(4.0).unwrap().increment_value(12.0).unwrap()
        .build(&mut stage).unwrap();
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
    let built = source.animate().set_complex_value(2.0, -3.0).unwrap()
        .increment_complex_value(4.0, 1.0).unwrap().build(&mut stage).unwrap();
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
        ordinary.animate().shift([1.0; 3]).unwrap().set_value(4.0).unwrap(),
        scalar.animate().set_value(5.0).unwrap().set_complex_value(1.0, 2.0).unwrap(),
        complex.animate().increment_value(3.0).unwrap(),
        ordinary.animate().increment_complex_value(1.0, 2.0).unwrap(),
    ] {
        let before = stage.snapshot().to_bytes().unwrap();
        assert!(matches!(builder.build(&mut stage), Err(AnimateError::TrackerKindMismatch { .. })));
        assert_eq!(stage.snapshot().to_bytes().unwrap(), before);
    }
}

#[test]
fn tracker_recordings_obey_the_same_override_and_argument_rules() {
    let mut stage = Stage::new();
    let source = stage.add_value_tracker(1.0);
    let override_animation = OverrideAnimation { name: "tracker override" };
    assert_eq!(source.animate().with_override(override_animation).unwrap().set_value(2.0),
        Err(AnimateError::OverrideNotChainable));
    assert_eq!(source.animate().increment_value(2.0).unwrap().with_override(override_animation),
        Err(AnimateError::OverrideNotChainable));
    assert_eq!(source.animate().set_anim_args(AnimateArgs::default()).unwrap()
        .set_value(2.0).unwrap().set_anim_args(AnimateArgs::default()), Err(AnimateError::ArgsAlreadySet));
}
''',
'crates/fmn-anim/tests/tracker_animation.rs': r'''//! Parameter-driven native scenes traverse the actual builder/animation/clock.
use fmn_anim::purity::{Purity, reconstruct_pure_frame};
use fmn_anim::{RationalFrameClock, play_segment, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};

fn linear() -> AnimateArgs {
    AnimateArgs { rate_func: Some(rate::linear), ..AnimateArgs::default() }
}

#[test]
fn scene_updaters_observe_the_new_tracker_value_before_each_capture() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(0.0);
    let dot = stage.add(Mobject::from_points(&[[0.0; 3]]));
    stage.add_to_scene(dot).unwrap();
    stage.add_to_scene(tracker).unwrap();
    stage.add_updater(dot, move |stage, me| {
        let value = stage.tracker_value(tracker).unwrap();
        stage.move_to(me, [value, value * 2.0, 0.0], [0.0; 3]);
    }, false).unwrap();
    let builder = tracker.animate().set_anim_args(linear()).unwrap().set_value(8.0).unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let mut values = Vec::new();
    let report = play_segment(&mut stage, &mut RationalFrameClock::new(4).unwrap(),
        &RngRoot::from_seed(0), &mut animations, false, &mut |packet| {
            let frame = packet.materialize_stage();
            values.push((frame.tracker_value(tracker).unwrap(), frame.get_center(dot)));
        }).unwrap();
    assert!(matches!(report.purity, Purity::Stateful(_)));
    assert_eq!(values, vec![(2.0, [2.0, 4.0, 0.0]), (4.0, [4.0, 8.0, 0.0]),
        (6.0, [6.0, 12.0, 0.0]), (8.0, [8.0, 16.0, 0.0])]);
}

#[test]
fn exponential_and_complex_builders_interpolate_their_native_encodings() {
    let mut stage = Stage::new();
    let scalar = stage.add_exponential_value_tracker(4.0);
    let complex = stage.add_complex_value_tracker(1.0, -2.0);
    let builder = scalar.animate().set_anim_args(linear()).unwrap().set_value(16.0).unwrap();
    let mut animation = prepare_animation(builder, &mut stage).unwrap();
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    assert!((stage.tracker_value(scalar).unwrap() - 8.0).abs() < 1e-12);
    animation.finish(&mut stage);
    let builder = complex.animate().set_anim_args(linear()).unwrap().set_complex_value(5.0, 6.0).unwrap();
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
    let builder = tracker.animate().set_anim_args(linear()).unwrap().increment_complex_value(4.0, -8.0).unwrap();
    let mut animations = vec![prepare_animation(builder, &mut stage).unwrap()];
    let root = RngRoot::from_seed(3);
    let mut serial = Vec::new();
    let report = play_segment(&mut stage, &mut RationalFrameClock::new(4).unwrap(),
        &root, &mut animations, false, &mut |packet| serial.push(packet)).unwrap();
    assert_eq!(report.purity, Purity::Pure);
    let samples: Vec<_> = RationalFrameClock::new(4).unwrap().segment(report.run_time).unwrap().samples().collect();
    for i in [3, 0, 2, 1] {
        let actual = reconstruct_pure_frame(&mut stage, &mut animations, &report, &root, &samples[i]).unwrap();
        assert_eq!(actual.materialize_stage().tracker_complex_value(tracker),
            serial[i].materialize_stage().tracker_complex_value(tracker));
    }
}
''',
'crates/fmn-scene/tests/tracker_bundle.rs': r'''//! FMTL must preserve invisible tracker state as well as drawable geometry.
use fmn_anim::{Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::Stage;
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

#[test]
fn pure_tracker_bundles_reconstruct_scalar_logarithmic_and_complex_state() {
    let mut stage = Stage::new();
    let scalar = stage.add_value_tracker(1.0);
    let exponential = stage.add_exponential_value_tracker(4.0);
    let complex = stage.add_complex_value_tracker(2.0, -3.0);
    stage.add_many_to_scene(&[scalar, exponential, complex]).unwrap();
    let args = AnimateArgs { rate_func: Some(rate::linear), ..AnimateArgs::default() };
    let mut animations = Vec::new();
    for builder in [
        scalar.animate().set_anim_args(args).unwrap().set_value(9.0).unwrap(),
        exponential.animate().set_anim_args(args).unwrap().set_value(16.0).unwrap(),
        complex.animate().set_anim_args(args).unwrap().set_complex_value(6.0, 5.0).unwrap(),
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
        let expected = match i { 0 => 32.0_f64.sqrt(), 1 => 8.0, 2 => 128.0_f64.sqrt(), _ => 16.0 };
        assert!((frame.tracker_value(roots[1]).unwrap() - expected).abs() < 1e-12);
        assert_eq!(frame.tracker_complex_value(roots[2]), Some((2.0 + 4.0 * alpha, -3.0 + 8.0 * alpha)));
    }
}

#[test]
fn out_and_back_tracker_motion_cannot_pass_an_endpoint_only_pure_proof() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(1.0);
    stage.add_to_scene(tracker).unwrap();
    let builder = tracker.animate().set_anim_args(AnimateArgs {
        rate_func: Some(rate::there_and_back), ..AnimateArgs::default()
    }).unwrap().set_value(9.0).unwrap();
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
''',
}
for name, text in new_files.items():
    if Path(name).exists() and Path(name).read_text() != text:
        raise SystemExit('new source already exists with different contents: ' + name)
    pending[name] = text
for name, text in pending.items():
    Path(name).write_text(text)
