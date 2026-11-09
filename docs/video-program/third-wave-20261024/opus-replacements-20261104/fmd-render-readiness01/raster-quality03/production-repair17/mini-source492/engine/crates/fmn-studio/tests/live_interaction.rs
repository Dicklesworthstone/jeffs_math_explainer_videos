//! Live ownership acceptance against actual Proscenium and Marionette.

use std::cell::Cell;
use std::rc::Rc;

use fmn_mobject::{Mob, Mobject};
use fmn_scene::studio_bridge::FramePacket;
use fmn_scene::{
    CaptureReason, EventPayload, IntegrationError, InteractiveScene, Key, Modifiers, MouseButton,
    NullSceneSink, RuntimeConfig, Scene, SceneError, SceneSink,
};
use fmn_studio::{InteractivePreview, InteractivePreviewError};

fn scene_with_point() -> (Scene, Mob) {
    let mut scene = Scene::new(RuntimeConfig::default(), 17).expect("scene");
    let root = scene
        .stage_mut()
        .add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
    scene.stage_mut().add_to_scene(root).expect("root");
    (scene, root)
}

fn select(point: [f64; 3]) -> EventPayload {
    EventPayload::MousePress {
        point,
        button: MouseButton::Left,
        modifiers: Modifiers::PRIMARY,
    }
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
fn live_adoption_keeps_the_original_arena_and_updater_callable() {
    let (mut scene, root) = scene_with_point();
    let calls = Rc::new(Cell::new(0_u32));
    let observed = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_updater(
            root,
            move |stage, target| {
                observed.set(observed.get() + 1);
                stage.shift_many(&[target], [0.01, 0.0, 0.0]);
            },
            false,
        )
        .expect("updater");
    let mut live = InteractivePreview::from_scene(scene).expect("live owner");
    assert_eq!(calls.get(), 0, "adoption must not execute callbacks");
    assert_eq!(live.stage().roots(), &[root]);
    let before = live.scene().time();
    let mut captures = Captures::default();
    // Binary-exact duration: unlike 0.1, 0.5 has no upward rounding at 30 fps.
    live.scene_mut()
        .wait(Some(0.5), &mut captures)
        .expect("same native runtime advances");
    assert!(calls.get() > 0);
    assert!(live.scene().time().frames() > before.frames());
    assert_eq!(captures.0.len(), 15);
    let first = captures.0[0].materialize_stage().get_bounding_box(root);
    let last = captures.0[14].materialize_stage().get_bounding_box(root);
    assert!(last.mid[0] > first.mid[0]);
    assert!(live.stage().contains(root));
}

#[test]
fn adopting_an_interactive_scene_keeps_selection_without_duplicate_listeners() {
    let (scene, root) = scene_with_point();
    let mut interactive = InteractiveScene::new(scene).expect("editor");
    interactive.queue_event(select([0.0; 3])).expect("select");
    interactive.dispatch_pending_events().expect("dispatch");
    let listener_ids = interactive.listener_ids().to_vec();
    let mut live = InteractivePreview::from_interactive(interactive).expect("adopt");
    assert_eq!(live.selection(), vec![root]);
    live.dispatch(EventPayload::KeyPress {
        key: Key::ArrowRight,
        modifiers: Modifiers::NONE,
    })
    .expect("one nudge");
    assert!((live.stage().get_bounding_box(root).mid[0] - 0.05).abs() < 1.0e-9);
    let recovered = live.into_interactive();
    assert_eq!(recovered.listener_ids(), listener_ids.as_slice());
    assert_eq!(recovered.selection(), vec![root]);
}

#[test]
fn pending_native_events_survive_ownership_transfer() {
    let (mut scene, root) = scene_with_point();
    scene
        .queue_event(select([0.0; 3]))
        .expect("queued before adoption");
    let mut live = InteractivePreview::from_scene(scene).expect("live");
    assert!(live.selection().is_empty());
    live.scene_mut().dispatch_pending_events().expect("drain");
    assert_eq!(live.selection(), vec![root]);
}

#[test]
fn the_runtime_clock_and_capture_anchor_have_distinct_meanings() {
    let (mut scene, _) = scene_with_point();
    scene.wait(Some(0.5), &mut NullSceneSink).expect("advance");
    let initial = scene.time();
    let mut live = InteractivePreview::from_scene(scene).expect("live");
    assert_eq!(live.frame_index(), u64::try_from(initial.frames()).unwrap());
    assert_eq!(live.scene().time(), initial);
    live.scene_mut()
        .wait(Some(0.5), &mut NullSceneSink)
        .expect("continue");
    assert_eq!(live.scene().time().frames(), initial.frames() + 15);
    assert_eq!(live.frame_index(), u64::try_from(initial.frames()).unwrap());
}

#[test]
fn a_durable_unbound_updater_cannot_bypass_readiness_by_live_adoption() {
    let (mut scene, root) = scene_with_point();
    scene
        .stage_mut()
        .add_updater(root, |_stage, _target| {}, false)
        .expect("updater");
    let bytes = scene.state_bytes().expect("snapshot");
    scene.restore_state_bytes(&bytes).expect("decode");
    let result = InteractivePreview::from_scene(scene);
    assert!(matches!(
        result,
        Err(InteractivePreviewError::Scene(SceneError::UnboundUpdaters))
    ));
}

#[test]
fn failed_snapshot_reset_does_not_discard_a_live_callback_owner() {
    let (mut scene, root) = scene_with_point();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_updater(
            root,
            move |_stage, _target| observed.set(observed.get() + 1),
            false,
        )
        .expect("updater");
    let source = scene.stage().snapshot().materialize();
    let mut live = InteractivePreview::from_scene(scene).expect("live");
    assert!(live.reset(&source, 30, 17, 0).is_err());
    assert_eq!(live.stage().roots(), &[root]);
    live.scene_mut()
        .wait(Some(0.1), &mut NullSceneSink)
        .expect("retained runtime");
    assert!(calls.get() > 0);
}

#[test]
fn native_history_and_updaters_remain_usable_after_preview_ownership_round_trip() {
    let (mut scene, root) = scene_with_point();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_updater(
            root,
            move |_stage, _target| observed.set(observed.get() + 1),
            false,
        )
        .expect("updater");
    let mut interactive = InteractiveScene::new(scene).expect("editor");
    interactive.queue_event(select([0.0; 3])).expect("select");
    interactive.dispatch_pending_events().expect("dispatch");
    interactive.save_undo_state();
    interactive.stage_mut().shift_many(&[root], [2.0, 0.0, 0.0]);
    let mut live = InteractivePreview::from_interactive(interactive).expect("live");
    live.dispatch(EventPayload::KeyPress {
        key: Key::Character('z'),
        modifiers: Modifiers::PRIMARY,
    })
    .expect("undo");
    assert!(live.stage().get_bounding_box(root).mid[0].abs() < 1.0e-9);
    assert_eq!(calls.get(), 0);
    let mut recovered = live.into_interactive();
    assert_eq!(recovered.history_depths(), (0, 1));
    assert!(recovered.redo());
    assert!((recovered.stage().get_bounding_box(root).mid[0] - 2.0).abs() < 1.0e-9);
    assert_eq!(calls.get(), 0);
    recovered
        .wait(Some(0.1), &mut NullSceneSink)
        .expect("native callback execution continues");
    assert!(calls.get() > 0);
}

#[test]
fn a_live_seek_refuses_before_relabeling_time_or_state() {
    let (scene, root) = scene_with_point();
    let mut live = InteractivePreview::from_scene(scene).expect("live");
    assert!(live.is_live());
    let time = live.scene().time();
    let bounds = live.stage().get_bounding_box(root);
    let result = live.seek_frame(99, 99);
    assert!(matches!(
        result,
        Err(InteractivePreviewError::LiveSeekRequiresReplay)
    ));
    assert_eq!(live.frame_index(), 0);
    assert_eq!(live.scene().time(), time);
    assert_eq!(live.stage().get_bounding_box(root), bounds);
}

#[test]
fn captured_seek_and_explicit_reset_remain_separate_from_live_advancement() {
    let (scene, _) = scene_with_point();
    let source = scene.stage().snapshot().materialize();
    let mut preview = InteractivePreview::from_scene(scene).expect("live");
    preview
        .reset(&source, 30, 17, 4)
        .expect("explicit captured reset");
    assert!(!preview.is_live());
    preview.seek_frame(5, 5).expect("captured relabel");
    assert_eq!(preview.frame_index(), 5);
    assert_eq!(preview.scene().time().frames(), 5);
}
