//! Perspective and mixed-primitive export through the public native API.

use std::path::Path;
use std::sync::Arc;

use fmn::prelude::*;
use fmn_codec::{PngLimits, decode_png};
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_render::RetainedFrameRendererError;
use fmn_runtime::ExecutionEngine;

fn options(path: &str) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.config.camera.resolution = (96, 64);
    options.config.camera.fps = 10;
    options.config.sizes.frame_height = 4.0;
    options.config.render.threads = ThreadPolicy::Fixed(1);
    options
}

struct MixedScene;

impl SceneConstruct for MixedScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Cube::new(1.4).color(BLUE))?;
        let circle = stage.add(Circle::new().radius(0.3).color(WHITE))?;
        stage.set_fill(circle, Some(WHITE), Some(1.0), None, true);
        stage.shift(circle, [-1.5, 0.0, 0.0]);
        Ok(())
    }
}

fn pixels(fs: &VirtualFs, directory: &str, sequence: u64) -> Vec<u8> {
    let bytes = fs
        .read(&Path::new(directory).join(format!("frame_{sequence:06}.png")))
        .unwrap();
    decode_png(&bytes, &PngLimits::default()).unwrap().rgba
}

#[test]
fn mixed_surface_and_vector_scene_uses_the_real_camera_pose() {
    let fs = Arc::new(VirtualFs::new());
    let mut images = Vec::new();
    for (directory, theta, phi) in [("/front", 0.0, 0.0), ("/oblique", 0.6, 0.8)] {
        let mut options = options(directory);
        // Standard mode must still report the certified CPU camera engine,
        // not the fast 2D engine that it bypasses.
        options.config.determinism.mode = DeterminismMode::Standard;
        let mut camera = options.camera_config().unwrap();
        assert_eq!(camera.frame.height(), 4.0);
        assert_eq!(camera.frame.width(), 6.0);
        camera
            .frame
            .set_euler_angles(Some(theta), Some(phi), None)
            .unwrap();
        options.camera = Some(camera);
        let report = render_with_fs(&mut MixedScene, options, fs.clone()).unwrap();
        assert_eq!(report.execution_plan.engine, ExecutionEngine::CertifiedCpu);
        assert_eq!(report.artifact.frame_count, 1);
        let image = pixels(&fs, directory, 0);
        let colored = image
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[2] > 30 && pixel[2] > pixel[0])
            .count();
        let white = image
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| pixel[0] > 180 && pixel[1] > 180 && pixel[2] > 180)
            .count();
        assert!(colored > 20, "surface pixels reached the output");
        assert!(white > 3, "vector pixels share the camera painter sequence");
        images.push(image);
    }
    assert_ne!(
        images[0], images[1],
        "camera pose changes the rendered scene"
    );
}

#[test]
fn non_vector_content_without_a_camera_is_not_silently_dropped() {
    let fs = Arc::new(VirtualFs::new());
    let error = render_with_fs(&mut MixedScene, options("/refused"), fs.clone()).unwrap_err();
    assert!(matches!(
        error,
        RenderError::Renderer(RetainedFrameRendererError::CameraRequired { .. })
    ));
    assert!(!fs.exists(Path::new("/refused")));
}

#[test]
fn camera_route_animation_frames_follow_the_same_scene_clock() {
    struct MovingCube;
    impl SceneConstruct for MovingCube {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            let cube = stage.add(Cube::new(1.2))?;
            stage.play(
                cube.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.2),
                        rate_func: Some(fmn::core::rate::linear),
                        ..AnimateArgs::default()
                    })?
                    .shift(RIGHT)?,
            )?;
            Ok(())
        }
    }
    let fs = Arc::new(VirtualFs::new());
    let mut options = options("/moving");
    let mut camera = options.camera_config().unwrap();
    camera
        .frame
        .set_euler_angles(Some(0.4), Some(0.7), None)
        .unwrap();
    options.camera = Some(camera);
    let report = render_with_fs(&mut MovingCube, options, fs.clone()).unwrap();
    assert_eq!(report.artifact.frame_count, 3);
    assert_eq!(report.scene.play_count, 1);
    assert_ne!(pixels(&fs, "/moving", 0), pixels(&fs, "/moving", 1));
}

#[test]
fn mismatched_and_invalid_cameras_are_refused_before_construction() {
    struct MustNotRun;
    impl SceneConstruct for MustNotRun {
        fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
            panic!("camera validation must precede scene execution");
        }
    }
    for case in 0..4 {
        let fs = Arc::new(VirtualFs::new());
        let mut options = options("/invalid");
        let mut camera = options.camera_config().unwrap();
        match case {
            0 => camera.resolution.0 += 2,
            1 => camera.fps += 1,
            2 => camera.background.a = 0.25,
            _ => camera.light_source_position[0] = f64::NAN,
        }
        options.camera = Some(camera);
        assert!(render_with_fs(&mut MustNotRun, options, fs.clone()).is_err());
        assert!(!fs.exists(Path::new("/invalid")));
    }
}
