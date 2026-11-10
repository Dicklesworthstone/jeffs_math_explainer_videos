//! An external host may complete step 4, but capture/clock remain Choreo-owned.
use std::cell::RefCell;
use std::rc::Rc;

use fmn_anim::{FramePacket, ImpureEffect, Purity, prepare_animation};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::{CaptureReason, IntegrationError, PlayOverrides, RuntimeConfig, Scene, SceneSink};

#[derive(Default)]
struct Sink(Vec<FramePacket>);
impl SceneSink for Sink {
    fn capture(&mut self, _: CaptureReason, frame: FramePacket) -> Result<(), IntegrationError> {
        self.0.push(frame);
        Ok(())
    }
}

fn scene() -> Scene {
    Scene::new(
        RuntimeConfig {
            fps: 4,
            ..RuntimeConfig::default()
        },
        42,
    )
    .unwrap()
}

fn observer(scene: &mut Scene) -> (fmn_mobject::Mob, Rc<RefCell<Vec<f64>>>) {
    let mob = scene
        .add_mobject(Mobject::from_points(&[[0.0; 3]]))
        .unwrap();
    let values = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&values);
    scene
        .stage_mut()
        .add_dt_updater(
            mob,
            move |stage: &mut Stage, target, dt| {
                if target == mob {
                    output.borrow_mut().push(dt);
                    stage.shift(target, [dt, 0.0, 0.0]);
                }
            },
            false,
        )
        .unwrap();
    (mob, values)
}

#[test]
fn external_wait_updates_are_consumed_once_and_captured_on_the_rational_clock() {
    let mut scene = scene();
    let (mob, ticks) = observer(&mut scene);
    let mut sink = Sink::default();
    scene.stage_mut().update(0.0); // the host's complete prologue
    let mut wait = scene
        .begin_stepped_wait_after_updaters(Some(0.5), &mut sink)
        .unwrap();
    assert_eq!(*ticks.borrow(), [0.0]);
    assert!(wait.mark_scene_updaters_complete().is_err());
    while let Some(boundary) = scene.prepare_stepped_wait_frame(&mut wait).unwrap() {
        assert_eq!(scene.time(), boundary.time);
        scene
            .stage_mut()
            .update_at_time(boundary.dt, boundary.time.to_f64());
        wait.mark_scene_updaters_complete().unwrap();
        assert!(wait.mark_scene_updaters_complete().is_err());
        scene
            .complete_stepped_wait_frame(&mut wait, &mut sink)
            .unwrap();
        assert!(wait.mark_scene_updaters_complete().is_err());
    }
    let report = scene.finish_stepped_wait(wait, &mut sink).unwrap();
    assert_eq!(*ticks.borrow(), [0.0, 0.25, 0.25]);
    assert_eq!(report.n_frames, 2);
    assert!(!report.purity.is_pure());
    assert!(report.begin_state.is_none());
    for (index, packet) in sink.0.iter().enumerate() {
        let expected = (index + 1) as f64 / 4.0;
        assert_eq!(packet.time().to_f64(), expected);
        assert_eq!(packet.materialize_stage().get_center(mob)[0], expected);
    }
    assert_eq!(sink.0.len(), 2);
}

#[test]
fn unmarked_native_wait_is_unchanged_and_aborted_marks_cannot_leak() {
    let mut scene = scene();
    let (_, ticks) = observer(&mut scene);
    let mut sink = Sink::default();
    let mut first = scene.begin_stepped_wait(Some(0.25), &mut sink).unwrap();
    scene
        .prepare_stepped_wait_frame(&mut first)
        .unwrap()
        .unwrap();
    first.mark_scene_updaters_complete().unwrap();
    scene.abort_stepped_wait(first, &mut sink);
    let mut next = scene.begin_stepped_wait(Some(0.25), &mut sink).unwrap();
    scene
        .prepare_stepped_wait_frame(&mut next)
        .unwrap()
        .unwrap();
    scene
        .complete_stepped_wait_frame(&mut next, &mut sink)
        .unwrap();
    scene.finish_stepped_wait(next, &mut sink).unwrap();
    assert_eq!(*ticks.borrow(), [0.0, 0.0, 0.25]);
    assert_eq!(sink.0.len(), 1);
}

#[test]
fn external_play_updates_and_final_zero_dt_do_not_repeat_native_slots() {
    let mut scene = scene();
    let (_, ticks) = observer(&mut scene);
    let animated = scene
        .add_mobject(Mobject::from_points(&[[0.0; 3]]))
        .unwrap();
    let builder = animated
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.5),
            ..AnimateArgs::default()
        })
        .unwrap()
        .shift([1.0, 0.0, 0.0])
        .unwrap();
    let animation = prepare_animation(builder, scene.stage_mut()).unwrap();
    let mut sink = Sink::default();
    let mut play = scene
        .begin_stepped_play(vec![animation], PlayOverrides::default(), &mut sink)
        .unwrap()
        .unwrap();
    assert!(play.mark_scene_updaters_complete().is_err());
    assert!(play.mark_final_scene_updaters_complete().is_err());
    while let Some(boundary) = scene.prepare_stepped_play_frame(&mut play).unwrap() {
        scene
            .stage_mut()
            .update_at_time(boundary.dt, boundary.time.to_f64());
        play.mark_scene_updaters_complete().unwrap();
        assert!(play.mark_scene_updaters_complete().is_err());
        assert!(play.mark_final_scene_updaters_complete().is_err());
        scene
            .complete_stepped_play_frame(&mut play, &mut sink)
            .unwrap();
    }
    scene.finish_stepped_play_animations(&mut play).unwrap();
    assert!(scene.prepare_stepped_play_frame(&mut play).is_err());
    scene.stage_mut().update(0.0);
    play.mark_final_scene_updaters_complete().unwrap();
    assert!(play.mark_final_scene_updaters_complete().is_err());
    let report = scene.finish_stepped_play(play, &mut sink).unwrap();
    assert_eq!(*ticks.borrow(), [0.25, 0.25, 0.0]);
    assert!(!report.purity.is_pure());
    assert_eq!(sink.0.len(), 2);
    assert_eq!(scene.play_count(), 1);
}

#[test]
fn external_callbacks_demote_a_slot_free_wait_including_skip_mode() {
    for skip in [false, true] {
        let mut scene = Scene::new(
            RuntimeConfig {
                fps: 4,
                skip_animations: skip,
                ..RuntimeConfig::default()
            },
            42,
        )
        .unwrap();
        let mob = scene
            .add_mobject(Mobject::from_points(&[[0.0; 3]]))
            .unwrap();
        let mut sink = Sink::default();
        let mut wait = scene
            .begin_stepped_wait_after_updaters(Some(0.5), &mut sink)
            .unwrap();
        while let Some(boundary) = scene.prepare_stepped_wait_frame(&mut wait).unwrap() {
            scene.stage_mut().shift(mob, [boundary.dt, 0.0, 0.0]);
            wait.mark_scene_updaters_complete().unwrap();
            scene
                .complete_stepped_wait_frame(&mut wait, &mut sink)
                .unwrap();
        }
        let report = scene.finish_stepped_wait(wait, &mut sink).unwrap();
        assert_eq!(
            report.purity,
            Purity::Stateful(vec![ImpureEffect::SceneUpdater])
        );
        assert!(report.begin_state.is_none());
        assert_eq!(scene.stage().get_center(mob)[0], 0.5);
        assert_eq!(scene.time().to_f64(), 0.5);
        assert_eq!(sink.0.len(), if skip { 0 } else { 2 });
    }
}
