//! Camera controls are ordinary native state, not a renderer-side callback.

use std::cell::Cell;
use std::rc::Rc;

use fmn_render::{Camera, CameraConfig};
use fmn_scene::{CameraRig, NullSceneSink, Scene};

#[test]
fn initial_camera_and_all_live_channels_are_sampled_without_mutation() {
    let mut base = CameraConfig::default();
    base.frame
        .set_euler_angles(Some(0.7), Some(0.004), Some(-0.3))
        .unwrap();
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let original = Camera::new(base.clone()).unwrap();
    let before = scene.state_bytes().unwrap();
    let first = rig.sample(scene.stage(), &base).unwrap();
    assert_eq!(first.frame.orientation(), original.frame().orientation());
    assert_eq!(first.frame.shape(), original.frame().shape());
    assert_eq!(scene.state_bytes().unwrap(), before);
    for (handle, value) in rig.center().into_iter().zip([1.0, 2.0, 3.0]) {
        scene.stage_mut().set_tracker_value(handle, value).unwrap();
    }
    scene
        .stage_mut()
        .set_tracker_value(rig.width(), 8.0)
        .unwrap();
    scene
        .stage_mut()
        .set_tracker_value(rig.field_of_view(), 0.9)
        .unwrap();
    for (handle, value) in rig.light().into_iter().zip([3.0, 4.0, 5.0]) {
        scene.stage_mut().set_tracker_value(handle, value).unwrap();
    }
    let current = rig.sample(scene.stage(), &base).unwrap();
    assert_eq!(current.frame.center(), [1.0, 2.0, 3.0]);
    assert_eq!(current.frame.width(), 8.0);
    assert_eq!(current.frame.field_of_view(), 0.9);
    assert_eq!(current.light_source_position, [3.0, 4.0, 5.0]);
}

#[test]
fn in_memory_snapshots_freeze_camera_values_but_retain_original_handles() {
    let base = CameraConfig::default();
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let snapshot = scene.stage().snapshot();
    scene
        .stage_mut()
        .set_tracker_value(rig.center()[0], 7.0)
        .unwrap();
    assert_eq!(
        rig.sample(scene.stage(), &base).unwrap().frame.center()[0],
        7.0
    );
    assert_eq!(
        rig.sample(&snapshot.materialize(), &base)
            .unwrap()
            .frame
            .center()[0],
        0.0
    );
    scene.stage_mut().restore(&snapshot);
    assert_eq!(
        rig.sample(scene.stage(), &base).unwrap().frame.center()[0],
        0.0
    );
}

#[test]
fn native_updaters_advance_the_camera_on_scene_time_and_sampling_never_ticks_them() {
    let base = CameraConfig::default();
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    scene
        .stage_mut()
        .add_dt_updater(
            rig.center()[0],
            move |stage, target, dt| {
                seen.set(seen.get() + 1);
                let current = stage.tracker_value(target).unwrap();
                stage.set_tracker_value(target, current + dt).unwrap();
            },
            false,
        )
        .unwrap();
    scene.wait(Some(0.5), &mut NullSceneSink).unwrap();
    let camera = rig.sample(scene.stage(), &base).unwrap();
    assert!((camera.frame.center()[0] - 0.5).abs() < 1e-6);
    let count = calls.get();
    let bytes = scene.state_bytes().unwrap();
    rig.sample(scene.stage(), &base).unwrap();
    assert_eq!(calls.get(), count);
    assert_eq!(scene.state_bytes().unwrap(), bytes);
}

#[test]
fn invalid_projection_or_channel_structure_is_not_silently_replaced() {
    let base = CameraConfig::default();
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let good = scene.stage().snapshot();
    for (handle, value) in [(rig.width(), 0.0), (rig.field_of_view(), 4.0)] {
        scene.stage_mut().set_tracker_value(handle, value).unwrap();
        assert!(rig.sample(scene.stage(), &base).is_err());
        scene.stage_mut().restore(&good);
    }
    for handle in rig.orientation() {
        scene.stage_mut().set_tracker_value(handle, 0.0).unwrap();
    }
    assert!(rig.sample(scene.stage(), &base).is_err());
    scene.stage_mut().restore(&good);
    scene.stage_mut().detach(rig.root(), rig.width());
    assert!(rig.sample(scene.stage(), &base).is_err());
    assert!(rig.sample(Scene::default().stage(), &base).is_err());
}
