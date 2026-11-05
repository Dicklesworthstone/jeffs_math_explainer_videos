//! Not part of either budget patch. The validation runner adds this file to its stage copy, runs it
//! before and after applying the patches, and compares what it writes byte for byte.
//!
//! It renders one small scene through five native routes at default budgets and writes every frame
//! to `$FMN_DEFAULT_PIXELS_OUT/<route>/`. It names no field the patches add, so the same file builds
//! on both trees.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn::prelude::*;
use fmn::rendering::{CameraConfig, render_bundle_with_fs, render_camera_with_fs};
use fmn_anim::{Timeline, prepare_animation};
use fmn_codec::{PngLimits, decode_png};
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_core::rng::RngRoot;
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, RenderPrimitive};
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_scene::CameraRig;
use fmn_scene::timeline_bundle::export_timeline_bundle;

fn options(path: &str) -> RenderOptions {
    let mut options = RenderOptions::new(path).unwrap();
    options.format = RenderFormat::PngSequence;
    options.config.camera.resolution = (96, 64);
    options.config.camera.fps = 8;
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(4);
    options.frames_in_flight = 2;
    options
}

fn tilted(path: &str) -> RenderOptions {
    let mut options = options(path);
    let mut camera = options.camera_config().unwrap();
    camera
        .frame
        .set_euler_angles(Some(0.4), Some(0.7), None)
        .unwrap();
    options.camera = Some(camera);
    options
}

fn outlined(points: &[[f32; 3]], fill_alpha: f32) -> Mobject {
    let mut records = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    for (i, point) in points.iter().enumerate() {
        records.write(i, "point", point);
        records.write(i, "fill_rgba", &[1.0, 0.2, 0.2, fill_alpha]);
        records.write(i, "stroke_rgba", &[0.2, 0.8, 1.0, 1.0]);
        records.write(i, "stroke_width", &[6.0]);
    }
    Mobject::from_buffer(records)
}

fn square() -> Mobject {
    outlined(
        &[
            [-0.8, -0.8, 0.0],
            [0.0, -0.8, 0.0],
            [0.8, -0.8, 0.0],
            [0.8, 0.0, 0.0],
            [0.8, 0.8, 0.0],
            [0.0, 0.8, 0.0],
            [-0.8, 0.8, 0.0],
            [-0.8, 0.0, 0.0],
            [-0.8, -0.8, 0.0],
        ],
        1.0,
    )
}

/// Forty curves in one stroked vector: enough for the camera route to bin them under the default.
fn zigzag() -> Mobject {
    const RECORDS: usize = 81;
    let points: Vec<[f32; 3]> = (0..RECORDS)
        .map(|i| {
            let x = -2.8 + 5.6 * i as f32 / (RECORDS - 1) as f32;
            let y = match i % 4 {
                0 => -1.2,
                2 => 1.2,
                _ => 0.0,
            };
            [x, y, 0.0]
        })
        .collect();
    outlined(&points, 0.0)
}

fn dots() -> Mobject {
    let points: [[f32; 3]; 3] = [[-1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
    let schema = RecordSchema::new(
        &[("point", 3), ("radius", 1), ("rgba", 4), ("glow_factor", 1)],
        &["point"],
        &["point"],
    )
    .unwrap();
    let mut records = RecordBuffer::new(schema, points.len()).unwrap();
    for (i, point) in points.iter().enumerate() {
        records.write(i, "point", point);
        records.write(i, "radius", &[0.2]);
        records.write(i, "rgba", &[0.2, 1.0, 0.2, 1.0]);
        records.write(i, "glow_factor", &[0.0]);
    }
    Mobject::from_buffer(records).with_render_primitive(RenderPrimitive::DotCloud)
}

/// The square comes first: it is the one that moves.
fn mobjects(camera: bool, with_dots: bool) -> Vec<Mobject> {
    let mut list = vec![square(), zigzag()];
    if camera {
        list.push(Cube::new(1.0).color(BLUE).into());
    }
    if with_dots {
        list.push(dots());
    }
    list
}

fn half_second() -> AnimateArgs {
    AnimateArgs {
        run_time: Some(0.5),
        rate_func: Some(fmn::core::rate::linear),
        ..AnimateArgs::default()
    }
}

struct Moving {
    camera: bool,
}

impl SceneConstruct for Moving {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut list = mobjects(self.camera, self.camera).into_iter();
        let moving = stage.add(list.next().unwrap())?;
        for mobject in list {
            stage.add(mobject)?;
        }
        stage.play(
            moving
                .animate()
                .set_anim_args(half_second())?
                .shift([1.0, 0.0, 0.0])?,
        )?;
        Ok(())
    }
}

fn bundle(camera: bool) -> Vec<u8> {
    let mut stage = fmn_mobject::Stage::new();
    let mut list = mobjects(camera, false).into_iter();
    let moving = stage.add(list.next().unwrap());
    stage.add_to_scene(moving).unwrap();
    for mobject in list {
        let mob = stage.add(mobject);
        stage.add_to_scene(mob).unwrap();
    }
    let builder = moving
        .animate()
        .set_anim_args(half_second())
        .unwrap()
        .shift([1.0, 0.0, 0.0])
        .unwrap();
    let animation = prepare_animation(builder, &mut stage).unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    timeline.play(vec![animation]).unwrap();
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(17)).unwrap()
}

struct Orbit {
    rig: CameraRig,
    target: CameraConfig,
}

impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        for mobject in mobjects(true, true) {
            stage.add(mobject)?;
        }
        let animation = self.rig.animate_to(stage.scene_mut(), &self.target)?;
        stage.play_prepared_with(
            vec![Box::new(animation)],
            PlayOverrides {
                run_time: Some(0.5),
                rate_func: Some(RateFunc::linear()),
                ..PlayOverrides::default()
            },
        )?;
        Ok(())
    }
}

fn orbit(scene: &mut Scene, base: &CameraConfig) -> fmn::Result<(Orbit, CameraRig)> {
    let rig = CameraRig::new(scene, base)?;
    let mut target = base.clone();
    target
        .frame
        .set_euler_angles(Some(0.6), Some(0.8), None)
        .unwrap();
    Ok((Orbit { rig, target }, rig))
}

fn publish(route: &str, frames: u64, fs: &VirtualFs) {
    assert!(frames > 1, "{route}: {frames} frames");
    let root = std::env::var("FMN_DEFAULT_PIXELS_OUT").expect("the runner sets this");
    let out = PathBuf::from(root).join(route);
    std::fs::create_dir_all(&out).unwrap();
    for index in 0..frames {
        let name = format!("frame_{index:06}.png");
        let bytes = fs.read(&Path::new("/").join(route).join(&name)).unwrap();
        let image = decode_png(&bytes, &PngLimits::default()).unwrap();
        let (first, rest) = image.rgba.as_chunks::<4>().0.split_first().unwrap();
        assert!(
            rest.iter().any(|pixel| pixel != first),
            "{route}: frame {index} drew nothing to compare"
        );
        std::fs::write(out.join(&name), &bytes).unwrap();
    }
}

#[test]
fn every_native_route_writes_its_default_frames() {
    let fs = Arc::new(VirtualFs::new());
    let report = render_with_fs(
        &mut Moving { camera: false },
        options("/affine-serial"),
        fs.clone(),
    )
    .unwrap();
    publish("affine-serial", report.artifact.frame_count, &fs);
    let report = render_with_fs(
        &mut Moving { camera: true },
        tilted("/camera-serial"),
        fs.clone(),
    )
    .unwrap();
    publish("camera-serial", report.artifact.frame_count, &fs);
    let report =
        render_bundle_with_fs(&bundle(false), options("/affine-worker"), fs.clone()).unwrap();
    publish("affine-worker", report.artifact.frame_count, &fs);
    let report =
        render_bundle_with_fs(&bundle(true), tilted("/camera-worker"), fs.clone()).unwrap();
    publish("camera-worker", report.artifact.frame_count, &fs);
    let report = render_camera_with_fs(orbit, options("/camera-rig"), fs.clone()).unwrap();
    publish("camera-rig", report.artifact.frame_count, &fs);
}
