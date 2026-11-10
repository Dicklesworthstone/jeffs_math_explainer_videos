//! Deferred authoring through actual native scenes, animations and snapshots.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fmn_anim::{Animation, Transform, rotate};
use fmn_core::constants::PI;
use fmn_mobject::{Mob, Mobject, Stage};
use fmn_scene::studio_bridge::FramePacket;
use fmn_scene::{
    CaptureReason, IntegrationError, PlayOverrides, RuntimeConfig, Scene, SceneError, SceneSink,
};
use fmn_studio::WorkerErrorCode;
use fmn_studio::native::{MAX_NATIVE_SEGMENTS, NativeSceneProgram, NativeSegment};

fn source() -> (Scene, Mob) {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        81,
    )
    .unwrap();
    let root = scene
        .add_mobject(Mobject::from_points(&[
            [-1.0, -0.5, 0.0],
            [1.0, -0.5, 0.0],
            [1.0, 0.5, 0.0],
        ]))
        .unwrap();
    (scene, root)
}

fn timing() -> PlayOverrides {
    PlayOverrides {
        run_time: Some(0.5),
        ..PlayOverrides::default()
    }
}

#[derive(Default)]
struct Captures(Vec<FramePacket>);
impl SceneSink for Captures {
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        assert_eq!(reason, CaptureReason::Segment);
        self.0.push(packet);
        Ok(())
    }
}

fn build_target(stage: &mut Stage, source: Mob) -> Result<Vec<Box<dyn Animation>>, SceneError> {
    let target = stage.copy_family(source)?;
    stage.shift_many(&[target], [3.0, 1.0, 0.0]);
    Ok(vec![Box::new(Transform::new(source, target))])
}

#[test]
fn late_targets_and_edits_match_ordinary_scene_playback_at_every_capture() {
    let (mut ordinary, root) = source();
    let mut expected = Captures::default();
    ordinary
        .play(vec![Box::new(rotate(root, PI))], timing(), &mut expected)
        .unwrap();
    ordinary.stage_mut().shift_many(&[root], [0.0, 2.0, 0.0]);
    let next = build_target(ordinary.stage_mut(), root).unwrap();
    ordinary.play(next, timing(), &mut expected).unwrap();
    ordinary.wait(Some(0.25), &mut expected).unwrap();

    let (scene, root) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(rotate(root, PI))],
                overrides: timing(),
            },
            NativeSegment::edit(move |context| {
                assert_eq!(context.scene().play_count(), 1);
                assert_eq!(context.scene().time().frames(), 4);
                context.stage_mut().shift_many(&[root], [0.0, 2.0, 0.0]);
                Ok(())
            }),
            NativeSegment::play_with(timing(), move |context| {
                build_target(context.stage_mut(), root)
            }),
            NativeSegment::Wait {
                duration: Some(0.25),
            },
        ],
        20,
    )
    .unwrap();
    for expected in &expected.0 {
        let actual = program.next_frame().unwrap().unwrap();
        assert_eq!(actual.time(), expected.time());
        assert_eq!(actual.alpha(), expected.alpha());
        assert_eq!(actual.segment_frame(), expected.segment_frame());
        assert_eq!(
            actual.materialize_stage().snapshot().to_bytes().unwrap(),
            expected.materialize_stage().snapshot().to_bytes().unwrap()
        );
    }
    assert!(program.next_frame().unwrap().is_none());
    assert_eq!(
        program.state_bytes().unwrap(),
        ordinary.state_bytes().unwrap()
    );
}

#[test]
fn deferred_sound_cues_use_the_completed_native_clock_without_extra_frames() {
    let (mut ordinary, _) = source();
    let mut captures = Captures::default();
    ordinary.wait(Some(0.25), &mut captures).unwrap();
    ordinary
        .add_sound("cue.wav", -0.125, Some(-3.0), Some(-9.0))
        .unwrap();
    ordinary.wait(Some(0.5), &mut captures).unwrap();

    let (scene, _) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Wait {
                duration: Some(0.25),
            },
            NativeSegment::edit(|context| {
                context.add_sound("cue.wav", -0.125, Some(-3.0), Some(-9.0))
            }),
            NativeSegment::Wait {
                duration: Some(0.5),
            },
        ],
        7,
    )
    .unwrap();
    assert!(program.preview().scene().sound_requests().is_empty());
    program.advance_to(2).unwrap();
    assert!(
        program.preview().scene().sound_requests().is_empty(),
        "pausing at the last capture must not execute the next authored statement"
    );
    program.advance_to(3).unwrap();
    assert_eq!(
        program.preview().scene().sound_requests(),
        ordinary.sound_requests()
    );
    program.advance_to(6).unwrap();
    assert!(program.next_frame().unwrap().is_none());
    assert_eq!(
        program.preview().scene().play_count(),
        ordinary.play_count()
    );
    assert_eq!(
        program.state_bytes().unwrap(),
        ordinary.state_bytes().unwrap()
    );
}

#[test]
fn invalid_deferred_sound_is_rejected_without_recording_a_cue() {
    let (scene, _) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::edit(|context| {
            context.add_sound("cue.wav", f64::NAN, None, None)
        })],
        1,
    )
    .unwrap();
    assert!(
        program
            .next_frame()
            .err()
            .expect("invalid deferred sound must be rejected")
            .to_string()
            .contains("sound time_offset must be finite")
    );
    assert!(program.preview().scene().sound_requests().is_empty());
}

#[test]
fn construction_waits_for_finish_not_the_last_interpolation_sample() {
    let (scene, root) = source();
    let seen = Rc::new(Cell::new(false));
    let observed = Rc::clone(&seen);
    let mut rotation = rotate(root, PI);
    rotation.state_mut().config.final_alpha_value = 0.0;
    let original = scene.stage().get_bounding_box(root);
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(rotation)],
                overrides: timing(),
            },
            NativeSegment::edit(move |context| {
                assert_eq!(context.stage().get_bounding_box(root), original);
                assert_eq!(context.scene().play_count(), 1);
                observed.set(true);
                Ok(())
            }),
            NativeSegment::Wait {
                duration: Some(0.25),
            },
        ],
        10,
    )
    .unwrap();
    program.advance_to(4).unwrap();
    assert!(
        !seen.get(),
        "paused terminal capture must not run the next constructor"
    );
    program.advance_to(5).unwrap();
    assert!(seen.get());
}

#[test]
fn newly_constructed_roots_do_not_leak_into_earlier_frozen_frames() {
    let (scene, old) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Wait {
                duration: Some(0.25),
            },
            NativeSegment::defer(move |context| {
                let stage = context.stage_mut();
                stage.remove_from_scene(old);
                let new = stage.add(Mobject::from_points(&[[8.0, 0.0, 0.0]]));
                stage.add_to_scene(new)?;
                Ok(vec![NativeSegment::Wait {
                    duration: Some(0.25),
                }])
            }),
        ],
        10,
    )
    .unwrap();
    let early = program.next_frame().unwrap().unwrap();
    let frozen = early.materialize_stage().snapshot().to_bytes().unwrap();
    program.advance_to(3).unwrap();
    assert_eq!(early.materialize_stage().roots(), &[old]);
    assert_eq!(
        early.materialize_stage().snapshot().to_bytes().unwrap(),
        frozen
    );
    assert!(!program.preview().stage().roots().contains(&old));
    assert_eq!(program.preview().stage().roots().len(), 1);
    assert!(
        program.preview().stage().contains(old),
        "scene removal is not arena destruction"
    );
}

#[test]
fn nested_builds_are_depth_first_and_each_one_shot_runs_once() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let outer = Rc::clone(&trace);
    let tail = Rc::clone(&trace);
    let mut program = NativeSceneProgram::new(
        source().0,
        vec![
            NativeSegment::defer(move |_| {
                outer.borrow_mut().push(1);
                let inner = Rc::clone(&outer);
                let after = Rc::clone(&outer);
                Ok(vec![
                    NativeSegment::defer(move |_| {
                        inner.borrow_mut().push(2);
                        Ok(vec![NativeSegment::Wait {
                            duration: Some(0.25),
                        }])
                    }),
                    NativeSegment::edit(move |_| {
                        after.borrow_mut().push(3);
                        Ok(())
                    }),
                ])
            }),
            NativeSegment::edit(move |_| {
                tail.borrow_mut().push(4);
                Ok(())
            }),
            NativeSegment::Wait {
                duration: Some(0.25),
            },
        ],
        10,
    )
    .unwrap();
    program.advance_to(0).unwrap();
    program.state_bytes().unwrap();
    assert!(trace.borrow().is_empty());
    program.advance_to(2).unwrap();
    assert_eq!(*trace.borrow(), vec![1, 2]);
    program.advance_to(2).unwrap();
    program.state_bytes().unwrap();
    assert_eq!(*trace.borrow(), vec![1, 2]);
    program.advance_to(3).unwrap();
    assert_eq!(*trace.borrow(), vec![1, 2, 3, 4]);
}

fn recursive_build(calls: Rc<Cell<usize>>) -> NativeSegment {
    NativeSegment::defer(move |_| {
        calls.set(calls.get() + 1);
        Ok(vec![recursive_build(calls)])
    })
}

#[test]
fn recursive_expansion_spends_a_lifetime_budget_not_just_queue_slots() {
    let calls = Rc::new(Cell::new(0));
    let mut program =
        NativeSceneProgram::new(source().0, vec![recursive_build(Rc::clone(&calls))], 1)
            .unwrap()
            .with_segment_limit(5)
            .unwrap();
    let error = program
        .next_frame()
        .err()
        .expect("recursive expansion must refuse");
    assert_eq!(error.code, WorkerErrorCode::InvalidRequest);
    assert!(error.message.contains("cumulative segment budget"));
    assert_eq!(calls.get(), 5);
    assert!(program.next_frame().is_err());
    assert!(program.state_bytes().is_err());
    assert_eq!(calls.get(), 5, "a failed one-shot cannot be retried");
}

#[test]
fn rejected_expansion_runs_none_of_its_returned_work() {
    let calls = Rc::new(Cell::new(0));
    let later = Rc::clone(&calls);
    let mut program = NativeSceneProgram::new(
        source().0,
        vec![NativeSegment::defer(move |_| {
            Ok(vec![NativeSegment::edit(move |_| {
                later.set(1);
                Ok(())
            })])
        })],
        1,
    )
    .unwrap()
    .with_segment_limit(1)
    .unwrap();
    assert!(program.next_frame().is_err());
    assert_eq!(calls.get(), 0);
}

#[test]
fn frame_budget_refuses_before_any_later_edit_or_finalization() {
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let mut program = NativeSceneProgram::new(
        source().0,
        vec![
            NativeSegment::Wait {
                duration: Some(0.5),
            },
            NativeSegment::edit(move |_| {
                observed.set(1);
                Ok(())
            }),
        ],
        4,
    )
    .unwrap();
    program.advance_to(4).unwrap();
    let state = program.state_bytes().unwrap();
    assert!(program.next_frame().is_err());
    assert_eq!(program.state_bytes().unwrap(), state);
    assert_eq!(calls.get(), 0);
}

#[test]
fn failed_edit_preserves_failure_without_claiming_mutation_rollback() {
    let (scene, root) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::edit(move |context| {
            context.stage_mut().shift_many(&[root], [5.0, 0.0, 0.0]);
            Err(SceneError::InvalidState("authored construction failed"))
        })],
        1,
    )
    .unwrap();
    let error = program
        .next_frame()
        .err()
        .expect("authored failure must propagate");
    assert!(error.message.contains("authored construction failed"));
    assert!(program.preview().stage().get_bounding_box(root).mid[0] > 4.0);
    assert!(program.state_bytes().is_err());
    assert!(program.next_frame().is_err());
}

#[test]
fn catching_a_builder_panic_does_not_make_the_partial_program_reusable() {
    let mut program = NativeSceneProgram::new(
        source().0,
        vec![NativeSegment::edit(|_| {
            panic!("authored construction panic");
        })],
        1,
    )
    .unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| program.next_frame())).is_err()
    );
    assert!(program.state_bytes().is_err());
    assert!(program.advance_to(0).is_err());
}

#[test]
fn invalid_segment_limits_do_not_invoke_authored_work() {
    let calls = Rc::new(Cell::new(0));
    for limit in [0, MAX_NATIVE_SEGMENTS + 1] {
        let observed = Rc::clone(&calls);
        let result = NativeSceneProgram::new(
            source().0,
            vec![NativeSegment::edit(move |_| {
                observed.set(1);
                Ok(())
            })],
            1,
        )
        .unwrap()
        .with_segment_limit(limit);
        assert!(result.is_err());
    }
    assert_eq!(calls.get(), 0);
    let empty = NativeSceneProgram::new(source().0, Vec::new(), 0)
        .unwrap()
        .with_segment_limit(0);
    assert!(empty.is_ok());
}
