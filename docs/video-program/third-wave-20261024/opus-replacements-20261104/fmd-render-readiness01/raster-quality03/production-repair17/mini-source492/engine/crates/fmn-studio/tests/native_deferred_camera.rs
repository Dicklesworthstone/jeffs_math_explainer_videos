//! Late camera construction uses the current pose, not the initial pose.

use std::cell::Cell;
use std::rc::Rc;

use fmn_render::CameraConfig;
use fmn_scene::{CameraRig, PlayOverrides, RuntimeConfig, Scene};
use fmn_studio::native::{NativeSceneProgram, NativeSegment};

fn timing() -> PlayOverrides {
    PlayOverrides {
        run_time: Some(0.5),
        ..PlayOverrides::default()
    }
}

#[test]
fn deferred_camera_transition_chooses_the_hemisphere_from_the_completed_predecessor() {
    let base = CameraConfig::default();
    let mut scene = Scene::new(
        RuntimeConfig {
            fps: 8,
            ..RuntimeConfig::default()
        },
        23,
    )
    .unwrap();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let mut first = base.clone();
    first.frame.set_orientation([0.0, 0.0, -0.8, 0.6]).unwrap();
    let mut second = base.clone();
    second.frame.set_orientation([0.0, 0.0, 0.8, 0.6]).unwrap();
    let first_animation = rig.animate_to(&mut scene, &first).unwrap();
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::Play {
                animations: vec![Box::new(first_animation)],
                overrides: timing(),
            },
            NativeSegment::play_with(timing(), move |context| {
                assert_eq!(context.scene().time().frames(), 4);
                Ok(vec![context.camera_to(rig, &second)?])
            }),
        ],
        8,
    )
    .unwrap()
    .with_camera_rig(rig)
    .unwrap();
    program.advance_to(6).unwrap();
    let halfway = rig
        .sample(program.preview().stage(), &base)
        .unwrap()
        .frame
        .orientation();
    // Rebuilding the second target at frame zero picks the opposite hemisphere
    // and incorrectly takes the long route through identity. The late target
    // must instead pass through a half-turn between these two orientations.
    assert!((halfway[2].abs() - 1.0).abs() < 1.0e-12);
    assert!(halfway[3].abs() < 1.0e-12);
    program.advance_to(8).unwrap();
    let end = rig
        .sample(program.preview().stage(), &base)
        .unwrap()
        .frame
        .orientation();
    assert!((end[2] + 0.8).abs() < 1.0e-12);
    assert!((end[3] + 0.6).abs() < 1.0e-12);
}

#[test]
fn invalid_camera_edit_refuses_before_invoking_its_continuation() {
    let base = CameraConfig::default();
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &base).unwrap();
    let width = rig.width();
    let ran = Rc::new(Cell::new(false));
    let continuation = Rc::clone(&ran);
    let mut program = NativeSceneProgram::new(
        scene,
        vec![
            NativeSegment::edit(move |context| {
                context.stage_mut().set_tracker_value(width, 0.0)?;
                Ok(())
            }),
            NativeSegment::edit(move |_| {
                continuation.set(true);
                Ok(())
            }),
        ],
        1,
    )
    .unwrap()
    .with_camera_rig(rig)
    .unwrap();
    assert!(program.next_frame().is_err());
    assert!(!ran.get());
    assert!(program.state_bytes().is_err());
}

#[test]
fn zero_frame_edits_still_prevent_late_camera_rebinding() {
    let mut scene = Scene::default();
    let rig = CameraRig::new(&mut scene, &CameraConfig::default()).unwrap();
    let mut program =
        NativeSceneProgram::new(scene, vec![NativeSegment::edit(|_| Ok(()))], 1).unwrap();
    assert!(program.next_frame().unwrap().is_none());
    assert_eq!(program.frame_index(), 0);
    assert!(program.with_camera_rig(rig).is_err());
}
