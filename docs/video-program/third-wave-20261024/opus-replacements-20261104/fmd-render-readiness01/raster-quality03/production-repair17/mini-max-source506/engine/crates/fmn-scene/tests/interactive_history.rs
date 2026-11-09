//! Native editor history through actual typed input, Stage snapshots and callbacks.

use std::cell::Cell;
use std::rc::Rc;

use fmn_mobject::{Mob, Mobject};
use fmn_scene::{
    EventPayload, InteractiveClipboard, InteractiveScene, Key, Modifiers, MouseButton,
    NullSceneSink,
};

fn dispatch(scene: &mut InteractiveScene, event: EventPayload) {
    scene.queue_event(event).expect("queue");
    scene.dispatch_pending_events().expect("dispatch");
}

fn key(scene: &mut InteractiveScene, key: Key, modifiers: Modifiers) {
    dispatch(scene, EventPayload::KeyPress { key, modifiers });
}

fn release(scene: &mut InteractiveScene, key: Key) {
    dispatch(
        scene,
        EventPayload::KeyRelease {
            key,
            modifiers: Modifiers::NONE,
        },
    );
}

fn motion(scene: &mut InteractiveScene, point: [f64; 3]) {
    dispatch(
        scene,
        EventPayload::MouseMotion {
            point,
            delta: [0.0; 3],
            modifiers: Modifiers::NONE,
        },
    );
}

fn select(scene: &mut InteractiveScene, point: [f64; 3]) {
    dispatch(
        scene,
        EventPayload::MousePress {
            point,
            button: MouseButton::Left,
            modifiers: Modifiers::PRIMARY,
        },
    );
}

fn editor() -> (InteractiveScene, Mob) {
    let mut scene = InteractiveScene::default();
    let root = scene.stage_mut().add(Mobject::from_points(&[[0.0; 3]]));
    scene.stage_mut().add_to_scene(root).expect("root");
    select(&mut scene, [0.0; 3]);
    (scene, root)
}

fn x(scene: &InteractiveScene, root: Mob) -> f64 {
    scene.stage().get_bounding_box(root).mid[0]
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
}

#[test]
fn several_native_nudges_undo_and_redo_in_order() {
    let (mut scene, root) = editor();
    for _ in 0..3 {
        key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    }
    assert_eq!(scene.history_depths(), (3, 0));
    for expected in [0.10, 0.05, 0.0] {
        assert!(scene.undo());
        close(x(&scene, root), expected);
    }
    assert!(!scene.undo());
    assert_eq!(scene.history_depths(), (0, 3));
    for expected in [0.05, 0.10, 0.15] {
        assert!(scene.redo());
        close(x(&scene, root), expected);
    }
    assert!(!scene.redo());
    assert_eq!(scene.selection(), vec![root]);
}

#[test]
fn shifted_primary_z_dispatches_redo_not_another_undo() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    key(&mut scene, Key::Character('z'), Modifiers::PRIMARY);
    close(x(&scene, root), 0.0);
    key(
        &mut scene,
        Key::Character('z'),
        Modifiers::PRIMARY | Modifiers::SHIFT,
    );
    close(x(&scene, root), 0.05);
}

#[test]
fn a_new_edit_discards_the_old_redo_branch() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    assert!(scene.undo());
    key(&mut scene, Key::ArrowLeft, Modifiers::NONE);
    assert_eq!(scene.history_depths(), (2, 0));
    assert!(!scene.redo());
    close(x(&scene, root), 0.0);
    assert!(scene.undo());
    close(x(&scene, root), 0.05);
}

#[test]
fn history_limit_bounds_both_branches_and_retains_nearby_states() {
    let (mut scene, root) = editor();
    scene.set_history_limit(2);
    for _ in 0..4 {
        key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    }
    assert_eq!(scene.history_depths(), (2, 0));
    assert!(scene.undo());
    assert!(scene.undo());
    assert!(!scene.undo());
    close(x(&scene, root), 0.1);
    scene.set_history_limit(1);
    assert_eq!(scene.history_depths(), (0, 1));
    assert!(scene.redo());
    close(x(&scene, root), 0.15);
    assert!(!scene.redo());
}

#[test]
fn zero_limit_disables_capture_and_releases_existing_branches() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    assert!(scene.undo());
    scene.set_history_limit(0);
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    scene.save_undo_state();
    assert_eq!(scene.history_depths(), (0, 0));
    assert!(!scene.undo());
    assert!(!scene.redo());
    close(x(&scene, root), 0.05);
}

#[test]
fn clearing_history_does_not_change_live_geometry() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    scene.clear_history();
    assert_eq!(scene.history_depths(), (0, 0));
    close(x(&scene, root), 0.05);
    assert_eq!(scene.selection(), vec![root]);
}

#[test]
fn native_group_topology_and_original_handles_round_trip() {
    let (mut scene, left) = editor();
    let right = scene
        .stage_mut()
        .add(Mobject::from_points(&[[2.0, 0.0, 0.0]]));
    scene.stage_mut().add_to_scene(right).expect("right");
    select(&mut scene, [2.0, 0.0, 0.0]);
    key(&mut scene, Key::Character('g'), Modifiers::PRIMARY);
    let group = scene.selection()[0];
    assert_eq!(scene.stage().roots(), &[group]);
    assert_eq!(
        scene.stage().get(group).unwrap().submobjects(),
        &[left, right]
    );
    assert!(scene.undo());
    assert_eq!(scene.stage().roots(), &[left, right]);
    assert_eq!(scene.selection(), vec![left, right]);
    assert!(scene.redo());
    assert_eq!(scene.stage().roots(), &[group]);
    assert_eq!(
        scene.stage().get(group).unwrap().submobjects(),
        &[left, right]
    );
    assert_eq!(scene.stage().get(left).unwrap().parents(), &[group]);
}

#[test]
fn deletion_restores_draw_roots_and_selection_in_both_directions() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::Backspace, Modifiers::NONE);
    assert!(scene.stage().roots().is_empty());
    assert!(scene.selection().is_empty());
    assert!(scene.undo());
    assert_eq!(scene.stage().roots(), &[root]);
    assert_eq!(scene.selection(), vec![root]);
    assert!(scene.redo());
    assert!(scene.stage().roots().is_empty());
    assert!(scene.selection().is_empty());
}

#[test]
fn one_grab_gesture_is_one_edit_and_undo_cancels_the_live_gesture() {
    let (mut scene, root) = editor();
    motion(&mut scene, [0.0; 3]);
    key(&mut scene, Key::Character('g'), Modifiers::NONE);
    for point in [1.0, 2.0, 3.0] {
        motion(&mut scene, [point, 0.0, 0.0]);
    }
    assert_eq!(scene.history_depths(), (1, 0));
    assert!(scene.undo());
    close(x(&scene, root), 0.0);
    motion(&mut scene, [4.0, 0.0, 0.0]);
    close(x(&scene, root), 0.0);
    assert!(scene.redo());
    close(x(&scene, root), 3.0);
}

#[test]
fn a_cancelled_or_motionless_gesture_does_not_destroy_redo() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    assert!(scene.undo());
    motion(&mut scene, [0.0; 3]);
    key(&mut scene, Key::Character('g'), Modifiers::NONE);
    motion(&mut scene, [0.0; 3]);
    release(&mut scene, Key::Character('g'));
    key(&mut scene, Key::Character('t'), Modifiers::NONE);
    release(&mut scene, Key::Character('t'));
    key(&mut scene, Key::Character('c'), Modifiers::NONE);
    key(&mut scene, Key::Character('c'), Modifiers::NONE);
    assert_eq!(scene.history_depths(), (0, 1));
    assert!(scene.redo());
    close(x(&scene, root), 0.05);
}

#[test]
fn repeated_resize_motion_is_one_history_transition() {
    let mut scene = InteractiveScene::default();
    let root = scene
        .stage_mut()
        .add(Mobject::from_points(&[[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]]));
    scene.stage_mut().add_to_scene(root).expect("root");
    select(&mut scene, [0.0; 3]);
    motion(&mut scene, [2.0, 0.0, 0.0]);
    key(&mut scene, Key::Character('t'), Modifiers::NONE);
    motion(&mut scene, [3.0, 0.0, 0.0]);
    motion(&mut scene, [4.0, 0.0, 0.0]);
    release(&mut scene, Key::Character('t'));
    assert_eq!(scene.history_depths(), (1, 0));
    close(scene.stage().get_bounding_box(root).width(), 4.0);
    assert!(scene.undo());
    close(scene.stage().get_bounding_box(root).width(), 2.0);
    assert!(scene.redo());
    close(scene.stage().get_bounding_box(root).width(), 4.0);
}

#[test]
fn history_retains_updater_identity_without_running_it_during_restore() {
    let (mut scene, root) = editor();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_updater(
            root,
            move |_stage, _mob| observed.set(observed.get() + 1),
            false,
        )
        .expect("updater");
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    assert!(scene.undo());
    assert!(scene.redo());
    assert_eq!(calls.get(), 0);
    scene.stage_mut().update(0.1);
    assert_eq!(calls.get(), 1);
}

#[test]
fn undo_does_not_rewind_the_native_runtime_clock() {
    let (mut scene, _) = editor();
    key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    scene
        .wait(Some(0.1), &mut NullSceneSink)
        .expect("native wait");
    let time = scene.time();
    assert!(scene.undo());
    assert_eq!(scene.time(), time);
    assert!(scene.redo());
    assert_eq!(scene.time(), time);
}

#[test]
fn an_explicit_native_edit_checkpoint_restores_shared_dag_edges() {
    let mut scene = InteractiveScene::default();
    let child = scene.stage_mut().add(Mobject::from_points(&[[0.0; 3]]));
    let left = scene.stage_mut().add(Mobject::new());
    let right = scene.stage_mut().add(Mobject::new());
    scene.stage_mut().attach(left, child).expect("left");
    scene.stage_mut().attach(right, child).expect("right");
    scene
        .stage_mut()
        .add_many_to_scene(&[left, right])
        .expect("roots");
    scene.save_undo_state();
    scene.stage_mut().detach(left, child);
    assert!(scene.undo());
    assert_eq!(scene.stage().get(child).unwrap().parents().len(), 2);
    assert!(scene.redo());
    assert_eq!(scene.stage().get(child).unwrap().parents(), &[right]);
}

#[test]
fn shrinking_mixed_history_preserves_the_nearest_transition_in_each_direction() {
    let (mut scene, root) = editor();
    for _ in 0..4 {
        key(&mut scene, Key::ArrowRight, Modifiers::NONE);
    }
    assert!(scene.undo());
    assert!(scene.undo());
    assert_eq!(scene.history_depths(), (2, 2));
    scene.set_history_limit(2);
    assert_eq!(scene.history_depths(), (1, 1));
    assert!(scene.undo());
    close(x(&scene, root), 0.05);
    assert!(!scene.undo());
    assert!(scene.redo());
    close(x(&scene, root), 0.10);
    assert!(scene.redo());
    close(x(&scene, root), 0.15);
    assert!(!scene.redo());
}

#[test]
fn pasted_copies_and_their_clipboard_templates_survive_undo_redo() {
    let (mut scene, root) = editor();
    key(&mut scene, Key::Character('c'), Modifiers::PRIMARY);
    let clipboard = scene.clipboard();
    let InteractiveClipboard::Mobjects(templates) = &clipboard else {
        panic!("copy must produce native clipboard templates");
    };
    key(&mut scene, Key::Character('v'), Modifiers::PRIMARY);
    let pasted = scene.selection()[0];
    assert_ne!(root, pasted);
    assert_eq!(scene.stage().roots(), &[root, pasted]);
    assert!(scene.undo());
    assert_eq!(scene.stage().roots(), &[root]);
    assert_eq!(scene.clipboard(), clipboard);
    assert!(templates.iter().all(|mob| scene.stage().contains(*mob)));
    assert!(scene.redo());
    assert_eq!(scene.stage().roots(), &[root, pasted]);
    assert_eq!(scene.selection(), vec![pasted]);
    assert_eq!(scene.clipboard(), clipboard);
}
