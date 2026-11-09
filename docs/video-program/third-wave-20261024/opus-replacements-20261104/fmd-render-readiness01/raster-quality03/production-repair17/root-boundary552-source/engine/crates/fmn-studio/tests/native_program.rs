//! Native Studio sampling against the ordinary Scene runtime, not a simulator.

use std::cell::Cell;
use std::rc::Rc;

use fmn_mobject::{Mob, Mobject};
use fmn_scene::studio_bridge::FramePacket;
use fmn_scene::{
    CaptureReason, EventPayload, IntegrationError, Key, Modifiers, MouseButton, PlayOverrides,
    RuntimeConfig, Scene, SceneSink,
};
use fmn_studio::WorkerErrorCode;
use fmn_studio::native::{NativeSceneProgram, NativeSegment};

fn source() -> (Scene, Mob, Rc<Cell<u32>>) {
    let mut scene = Scene::new(RuntimeConfig::default(), 71).expect("scene");
    let root = scene.stage_mut().add(Mobject::from_points(&[[0.0; 3]]));
    scene.stage_mut().add_to_scene(root).expect("root");
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
        .expect("updater");
    (scene, root, calls)
}

#[derive(Default)]
struct Captures(Vec<FramePacket>);

impl SceneSink for Captures {
    fn capture(
        &mut self,
        _reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        self.0.push(packet);
        Ok(())
    }
}

#[test]
fn waits_match_every_ordinary_native_capture_and_finish_once() {
    let (mut ordinary, ordinary_root, ordinary_calls) = source();
    let mut expected = Captures::default();
    ordinary.wait(Some(0.1), &mut expected).expect("first wait");
    ordinary
        .wait(Some(0.2), &mut expected)
        .expect("second wait");
    let (scene, root, calls) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Wait {
                duration: Some(0.1),
            },
            NativeSegment::Wait {
                duration: Some(0.2),
            },
        ],
        100,
    )
    .expect("program");
    assert_eq!(calls.get(), 0, "adoption must not run callbacks");
    for frame in &expected.0 {
        let actual = program.next_frame().expect("step").expect("frame");
        assert_eq!(actual.time(), frame.time());
        assert_eq!(actual.alpha(), frame.alpha());
        assert_eq!(actual.segment_frame(), frame.segment_frame());
        assert_eq!(
            actual.materialize_stage().get_bounding_box(root),
            frame.materialize_stage().get_bounding_box(ordinary_root)
        );
    }
    assert!(program.next_frame().expect("end").is_none());
    assert_eq!(program.preview().scene().play_count(), 2);
    assert_eq!(calls.get(), ordinary_calls.get());
    assert_eq!(program.preview().scene().time(), ordinary.time());
}

#[test]
fn input_runs_the_existing_editor_at_the_paused_native_time() {
    let (scene, root, calls) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(1.0),
        }],
        30,
    )
    .expect("program");
    program.advance_to(3).expect("advance");
    let before = program.preview().stage().get_bounding_box(root);
    let time = program.preview().scene().time();
    let count = calls.get();
    program
        .dispatch(EventPayload::MousePress {
            point: before.mid,
            button: MouseButton::Left,
            modifiers: Modifiers::PRIMARY,
        })
        .expect("select");
    program
        .dispatch(EventPayload::KeyPress {
            key: Key::ArrowUp,
            modifiers: Modifiers::NONE,
        })
        .expect("nudge");
    assert_eq!(program.preview().selection(), vec![root]);
    assert!((program.preview().stage().get_bounding_box(root).mid[1] - 0.05).abs() < 1.0e-6);
    assert_eq!(program.preview().scene().time(), time);
    assert_eq!(calls.get(), count, "paused editing must not run updaters");
    program
        .dispatch(EventPayload::KeyPress {
            key: Key::Character('z'),
            modifiers: Modifiers::PRIMARY,
        })
        .expect("undo");
    assert_eq!(program.preview().stage().get_bounding_box(root), before);
    program.advance_to(4).expect("continue native callbacks");
    assert_eq!(calls.get(), count + 1);
}

#[test]
fn clock_jumps_and_budget_overruns_do_not_execute_callbacks() {
    let (scene, _, calls) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(1.0),
        }],
        2,
    )
    .expect("program");
    assert_eq!(
        program.advance_to(3).unwrap_err().code,
        WorkerErrorCode::InvalidRequest
    );
    assert_eq!(calls.get(), 0);
    program.advance_to(2).expect("budget boundary");
    let count = calls.get();
    assert!(program.advance_to(1).is_err());
    assert!(program.next_frame().is_err());
    assert_eq!(calls.get(), count);
    assert_eq!(program.frame_index(), 2);
    assert!(
        program.state_bytes().is_ok(),
        "admission refusal must preserve the cursor"
    );
}

#[test]
fn empty_and_zero_duration_segments_use_the_engine_lifecycle() {
    let (scene, _, _) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: Vec::new(),
                overrides: PlayOverrides::default(),
            },
            NativeSegment::Wait {
                duration: Some(0.0),
            },
            NativeSegment::Wait {
                duration: Some(0.5),
            },
        ],
        20,
    )
    .expect("program");
    program.advance_to(15).expect("skip empty intervals");
    assert!(program.next_frame().expect("end").is_none());
    assert_eq!(program.preview().scene().play_count(), 2);
}

#[test]
fn invalid_segment_poisoning_prevents_reusing_partial_execution() {
    let (scene, _, _) = source();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![NativeSegment::Wait {
            duration: Some(f64::NAN),
        }],
        10,
    )
    .expect("schedule is validated when opened");
    assert!(program.next_frame().is_err());
    assert!(program.state_bytes().is_err());
    assert!(
        program
            .dispatch(EventPayload::KeyPress {
                key: Key::ArrowRight,
                modifiers: Modifiers::NONE,
            })
            .is_err()
    );
}

#[test]
fn skip_range_and_presenter_scenes_refuse_before_adoption() {
    for config in [
        RuntimeConfig {
            skip_animations: true,
            ..RuntimeConfig::default()
        },
        RuntimeConfig {
            start_at_play: Some(1),
            ..RuntimeConfig::default()
        },
        RuntimeConfig {
            presenter_mode: true,
            ..RuntimeConfig::default()
        },
    ] {
        let result = NativeSceneProgram::new(Scene::new(config, 1).unwrap(), Vec::new(), 10);
        assert!(matches!(result, Err(error) if error.code == WorkerErrorCode::InvalidRequest));
    }
}
