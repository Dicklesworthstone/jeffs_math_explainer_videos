//! Real CLI camera selection, mixed-resource output, and pre-publication refusal.
#![forbid(unsafe_code)]
#![cfg(all(feature = "cli", feature = "batch"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use fmn::animation::{Timeline, prepare_animation};
use fmn::mobject::animate::AnimateArgs;
use fmn::mobject::{Mob, Mobject, Stage};
use fmn_codec::{PngLimits, decode_png};
use fmn_core::{constants::*, rng::RngRoot};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_library::{Circle, Cube, DotCloud, ImageMobject};
use fmn_render::{
    Camera, CameraConfig, CameraFrame, EngineIdentity, FrameConfig, RetainedFrameRenderer,
    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
};
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fmn-camera-cli-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).unwrap();
    root
}
fn run(
    source: &Path,
    output: &Path,
    name: &str,
    threads: &str,
    format: &str,
    extra: &[&str],
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    // GIF is deliberately outside the certified artifact set.
    if format != "gif" && !extra.contains(&"--engine") {
        command.arg("--reproducible");
    }
    command
        .args([
            "--robot",
            "--resolution",
            "64x48",
            "--fps",
            "8",
            "--threads",
            threads,
            "--format",
            format,
            "--video_dir",
        ])
        .arg(output)
        .args(extra)
        .arg(source)
        .arg(name)
        .env("PATH", "")
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME");
    command.output().unwrap()
}
fn ok(output: Output, route: &str) -> String {
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{text}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(text.contains(&format!("\"route\":\"{route}\"")), "{text}");
    assert!(text.contains("certified-cpu"), "{text}");
    assert!(text.contains("\"outstanding_slots\":0"), "{text}");
    text
}
fn pngs(directory: &Path) -> Vec<Vec<u8>> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "png"))
        .collect();
    paths.sort();
    paths.iter().map(|p| std::fs::read(p).unwrap()).collect()
}
fn add(stage: &mut Stage, object: impl Into<Mobject>) -> Mob {
    let mob = stage.add(object);
    stage.add_to_scene(mob).unwrap();
    mob
}
fn image() -> ImageMobject {
    ImageMobject::from_rgba8(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    )
    .unwrap()
    .with_height(2.5)
}
fn mixed_bundle(recorded: bool) -> Vec<u8> {
    let mut stage = Stage::new();
    let vector = add(&mut stage, Circle::new().radius(0.4).color(WHITE));
    stage.shift(vector, [0.0, 1.2, 1.0]);
    let cube = if recorded {
        stage.add(Mobject::from(Cube::new(1.4).color(BLUE)))
    } else {
        add(&mut stage, Cube::new(1.4).color(BLUE))
    };
    stage.shift(cube, [-1.4, 0.0, 0.0]);
    if recorded {
        // Appears only in middle recorded frames. Projection must be chosen
        // for the complete artifact, not late or from its first frame alone.
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        stage
            .add_dt_updater(
                vector,
                move |stage, _, dt| {
                    // Wait preparation invokes a zero-dt updater pass. Count
                    // only sampled ticks so the cube first appears in frame 1.
                    if dt == 0.0 {
                        return;
                    }
                    let n = calls.get() + 1;
                    calls.set(n);
                    if n == 2 {
                        stage.add_to_scene(cube).unwrap();
                    }
                    if n == 4 {
                        stage.remove_from_scene(cube);
                    }
                },
                false,
            )
            .unwrap();
    } else {
        let image = add(&mut stage, image());
        stage.shift(image, [1.7, 0.0, 0.0]);
        add(
            &mut stage,
            DotCloud::new([[0.0, -1.0, 0.5], [0.5, -0.6, 1.0]])
                .colored(YELLOW, 1.0)
                .with_radius(0.2)
                .make_3d(),
        );
    }
    let mut timeline = Timeline::new(8).unwrap();
    if recorded {
        timeline.wait(0.5).unwrap();
    } else {
        let anim = cube
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                rate_func: Some(fmn_core::rate::linear),
                ..Default::default()
            })
            .unwrap()
            .shift([0.5, 0.0, 0.0])
            .unwrap();
        timeline
            .play(vec![prepare_animation(anim, &mut stage).unwrap()])
            .unwrap();
    }
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(5)).unwrap()
}
fn serial(bundle: &TimelineBundle) -> Vec<Vec<u8>> {
    let config = fmn_config::Config::resolve(&[], None).unwrap().config;
    let mut frame = CameraFrame::default();
    frame
        .set_shape([
            config.sizes.frame_height * 64.0 / 48.0,
            config.sizes.frame_height,
        ])
        .unwrap();
    let camera = Camera::new(CameraConfig {
        resolution: (64, 48),
        fps: 8,
        frame,
        background: fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .unwrap()
            .to_linear(config.camera.background_opacity),
        ..Default::default()
    })
    .unwrap();
    let frame = FrameConfig::new(
        Viewport {
            width: 64,
            height: 48,
        },
        ScreenMap {
            scale: 48.0 / config.sizes.frame_height,
            origin: [32.0, 24.0],
            y_up: true,
        },
        camera.background(),
    );
    (0..bundle.frame_count())
        .map(|index| {
            let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
                frame,
                tiling: Tiling::default(),
                engine: EngineIdentity::certified(),
                threads: 1,
            })
            .unwrap();
            renderer
                .render_with_camera(&bundle.stage_at(index).unwrap(), &camera)
                .unwrap();
            let mut output =
                FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 64, 48).unwrap());
            rgba16f_to_rgba8(renderer.frame(), &mut output).unwrap();
            output.as_bytes().to_vec()
        })
        .collect()
}

#[test]
fn every_camera_registration_reaches_the_real_binary_at_all_thread_caps() {
    for name in [
        "surface_cube.v1",
        "dot_cloud_depth.v1",
        "image_quad.v1",
        "mixed_camera.v1",
    ] {
        let root = root();
        let mut baseline = None;
        for threads in ["1", "4", "16"] {
            let output = root.join(threads);
            ok(
                run(
                    Path::new("@builtin"),
                    &output,
                    name,
                    threads,
                    "png_sequence",
                    &[],
                ),
                "camera-cpu",
            );
            let frames = pngs(&output.join(name));
            assert_eq!(frames.len(), 2);
            assert_ne!(
                frames[0], frames[1],
                "registration must animate visible content"
            );
            let rgba = decode_png(&frames[0], &PngLimits::default()).unwrap().rgba;
            assert!(
                rgba.as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
            );
            if let Some(baseline) = &baseline {
                assert_eq!(&frames, baseline);
            } else {
                baseline = Some(frames);
            }
        }
    }
}

#[test]
fn pure_and_recorded_mixed_bundles_match_serial_camera_for_the_whole_artifact() {
    for recorded in [false, true] {
        let root = root();
        let bytes = mixed_bundle(recorded);
        let source = root.join("scene.fmtl");
        std::fs::write(&source, &bytes).unwrap();
        let reader = TimelineBundle::from_bytes(&bytes).unwrap();
        assert_eq!(
            reader.segment_kind(0),
            Some(if recorded {
                BundleSegmentKind::Stateful
            } else {
                BundleSegmentKind::Pure
            })
        );
        assert!(reader.requires_camera());
        let expected = serial(&reader);
        assert_ne!(expected[0], expected[1]);
        if recorded {
            let visible: Vec<_> = (0..reader.frame_count())
                .map(|index| {
                    reader
                        .stage_at(index)
                        .unwrap()
                        .draw_plan()
                        .items()
                        .iter()
                        .any(|item| item.key.program != fmn::mobject::ProgramKind::Vector)
                })
                .collect();
            assert_eq!(visible, [false, true, true, false]);
        }
        for threads in ["1", "4", "16"] {
            let output = root.join(threads);
            let text = ok(
                run(&source, &output, "Mixed", threads, "png_sequence", &[]),
                "compiled-camera-cpu",
            );
            assert!(text.contains("\"submitted\":4,\"emitted\":4"));
            let frames = pngs(&output.join("Mixed"));
            assert_eq!(frames.len(), expected.len());
            for (png, want) in frames.iter().zip(&expected) {
                assert_eq!(&decode_png(png, &PngLimits::default()).unwrap().rgba, want);
            }
        }
    }
}

#[test]
fn detached_targets_do_not_force_perspective_but_depth_tested_vectors_do() {
    for depth in [false, true] {
        let mut stage = Stage::new();
        let vector = add(&mut stage, Circle::new());
        stage.add(Mobject::from(Cube::new(1.0)));
        stage.get_mut(vector).unwrap().uniforms_mut().depth_test = depth;
        let mut timeline = Timeline::new(8).unwrap();
        timeline.wait(0.125).unwrap();
        let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(1)).unwrap();
        assert_eq!(
            TimelineBundle::from_bytes(&bytes)
                .unwrap()
                .requires_camera(),
            depth
        );
    }
}

#[test]
fn camera_native_streams_subdivisions_and_no_clobber_share_the_output_contract() {
    let root = root();
    let source = root.join("scene.fmtl");
    std::fs::write(&source, mixed_bundle(false)).unwrap();
    for format in ["gif", "y4m"] {
        let mut expected = None;
        for threads in ["1", "4", "16"] {
            let output = root.join(format!("{format}-{threads}"));
            ok(
                run(&source, &output, "Mixed", threads, format, &[]),
                "compiled-camera-cpu",
            );
            let bytes = std::fs::read(output.join(format!("Mixed.{format}"))).unwrap();
            if let Some(expected) = &expected {
                assert_eq!(&bytes, expected);
            } else {
                expected = Some(bytes.clone());
            }
            assert!(
                !run(&source, &output, "Mixed", threads, format, &[])
                    .status
                    .success()
            );
            assert_eq!(
                std::fs::read(output.join(format!("Mixed.{format}"))).unwrap(),
                bytes
            );
        }
    }
    let output = root.join("partial");
    ok(
        run(
            Path::new("@builtin"),
            &output,
            "mixed_camera.v1",
            "4",
            "png_sequence",
            &["--subdivide"],
        ),
        "camera-cpu",
    );
    let naming = fmn_scene::OutputNaming {
        output_directory: output,
        ..Default::default()
    };
    assert_eq!(
        pngs(&naming.partial_directory("mixed_camera.v1").join("00000")).len(),
        2
    );
    let output = root.join("refused");
    std::fs::create_dir(&output).unwrap();
    let rejected = run(
        &source,
        &output,
        "Mixed",
        "4",
        "png_sequence",
        &["--engine", "metal"],
    );
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8(rejected.stdout)
            .unwrap()
            .contains("requires the CPU renderer")
    );
    assert_eq!(std::fs::read_dir(output).unwrap().count(), 0);
}
