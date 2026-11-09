//! Full native animation lifecycle equivalence, including suspended callbacks.

use std::cell::Cell;
use std::rc::Rc;

use fmn_anim::{FramePacket, rotate};
use fmn_core::constants::PI;
use fmn_mobject::{Mob, Mobject};
use fmn_scene::{CaptureReason, IntegrationError, PlayOverrides, RuntimeConfig, Scene, SceneSink};
use fmn_studio::native::{NativeSceneProgram, NativeSegment};

fn source(fps: u32) -> (Scene, Mob, Rc<Cell<u32>>) {
    let mut scene = Scene::new(
        RuntimeConfig {
            fps,
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
    let calls = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_updater(
            root,
            move |stage, target| {
                counter.set(counter.get() + 1);
                stage.shift_many(&[target], [0.125, 0.0, 0.0]);
            },
            false,
        )
        .unwrap();
    // The follower also verifies that animation interpolation precedes the
    // scene-updater phase; it references an original handle in this arena.
    let follower = scene
        .add_mobject(Mobject::from_points(&[[0.0; 3]]))
        .unwrap();
    scene
        .stage_mut()
        .add_updater(
            follower,
            move |stage, target| {
                stage.set_x(target, stage.get_center(root)[0]);
            },
            false,
        )
        .unwrap();
    (scene, root, calls)
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

#[test]
fn rotation_then_wait_matches_every_ordinary_capture_and_final_runtime_state() {
    for fps in [8, 30] {
        let overrides = PlayOverrides {
            run_time: Some(0.5),
            ..PlayOverrides::default()
        };
        let (mut expected, root, expected_calls) = source(fps);
        let mut captures = Captures::default();
        expected
            .play(
                vec![Box::new(rotate(root, PI))],
                overrides.clone(),
                &mut captures,
            )
            .unwrap();
        expected.wait(Some(0.5), &mut captures).unwrap();
        let (scene, root, calls) = source(fps);
        let mut program = NativeSceneProgram::new(
            scene,
            vec![
                NativeSegment::Play {
                    animations: vec![Box::new(rotate(root, PI))],
                    overrides,
                },
                NativeSegment::Wait {
                    duration: Some(0.5),
                },
            ],
            100,
        )
        .unwrap();
        for expected in &captures.0 {
            let actual = program
                .next_frame()
                .unwrap()
                .expect("native animation capture");
            assert_eq!(actual.time(), expected.time());
            assert_eq!(actual.alpha(), expected.alpha());
            assert_eq!(actual.segment_frame(), expected.segment_frame());
            assert_eq!(
                actual.materialize_stage().snapshot().to_bytes().unwrap(),
                expected.materialize_stage().snapshot().to_bytes().unwrap(),
                "all records, roots and callback registrations must match at frame {}",
                actual.frame_index(),
            );
        }
        assert!(program.next_frame().unwrap().is_none());
        assert_eq!(program.preview().scene().play_count(), 2);
        assert_eq!(calls.get(), expected_calls.get());
        assert_eq!(
            program.state_bytes().unwrap(),
            expected.state_bytes().unwrap()
        );
    }
}

#[test]
fn pausing_on_the_last_animation_capture_does_not_finalize_early() {
    let (scene, root, calls) = source(8);
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(rotate(root, PI))],
                overrides: PlayOverrides {
                    run_time: Some(0.5),
                    ..PlayOverrides::default()
                },
            },
            NativeSegment::Wait {
                duration: Some(0.5),
            },
        ],
        20,
    )
    .unwrap();
    program.advance_to(4).unwrap();
    assert_eq!(program.preview().scene().play_count(), 0);
    let count = calls.get();
    let state = program.state_bytes().unwrap();
    program.advance_to(4).unwrap();
    assert_eq!(program.state_bytes().unwrap(), state);
    assert_eq!(
        calls.get(),
        count,
        "revisiting a paused capture must not run cleanup callbacks"
    );
    program.advance_to(5).unwrap();
    assert_eq!(program.preview().scene().play_count(), 1);
    assert!(
        calls.get() > count,
        "native completion must resume the suspended updater"
    );
}
