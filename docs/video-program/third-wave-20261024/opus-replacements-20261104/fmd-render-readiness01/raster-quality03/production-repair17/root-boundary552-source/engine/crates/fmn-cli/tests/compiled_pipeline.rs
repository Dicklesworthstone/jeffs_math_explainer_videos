//! The shipping compiled-scene path must reconstruct on its render teams.
#![forbid(unsafe_code)]
#![cfg(all(feature = "cli", feature = "batch"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use fmn::animation::{Timeline, prepare_animation};
use fmn::mobject::animate::AnimateArgs;
use fmn::mobject::{Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_codec::{PngLimits, decode_png};
use fmn_core::{rate, rng::RngRoot};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap,
    Tiling, Viewport,
};
use fmn_scene::timeline_bundle::{BundleSegmentKind, TimelineBundle, export_timeline_bundle};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "fmn-compiled-pipeline-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

fn bundle(stateful: bool, zero_segment: bool) -> Vec<u8> {
    let mut records = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    records.write_range(
        "point",
        0,
        &[-2.0, -1.0, 0.0, -0.5, 1.0, 0.0, 1.0, -1.0, 0.0],
    );
    records.write_range("fill_rgba", 0, &[0.2, 0.6, 1.0, 1.0].repeat(3));
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(records));
    stage.add_to_scene(mob).unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    if stateful {
        // Captures a non-Send value. Compiled playback must consume the stored
        // snapshots, not rerun this callback or guess a pure endpoint law.
        let calls = std::rc::Rc::new(std::cell::Cell::new(0_u32));
        stage
            .add_dt_updater(
                mob,
                move |stage, mob, dt| {
                    calls.set(calls.get() + 1);
                    stage.shift(mob, [dt, dt * 0.5, 0.0]);
                },
                false,
            )
            .unwrap();
        timeline.wait(0.5).unwrap();
    } else {
        let builder = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                rate_func: Some(rate::linear),
                ..AnimateArgs::default()
            })
            .unwrap()
            .shift([1.0, 0.0, 0.0])
            .unwrap();
        timeline
            .play(vec![prepare_animation(builder, &mut stage).unwrap()])
            .unwrap();
    }
    timeline
        .wait(if zero_segment { 0.0 } else { 0.25 })
        .unwrap();
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(17)).unwrap()
}

fn invoke(source: &Path, output: &Path, format: &str, threads: &str, extra: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    command.current_dir(source.parent().unwrap());
    if format != "gif" {
        command.arg("--reproducible");
    }
    command
        .args([
            "--robot",
            "--format",
            format,
            "--resolution",
            "32x24",
            "--fps",
            "8",
            "--threads",
            threads,
            "--video_dir",
        ])
        .arg(output)
        .args(extra)
        .arg(source)
        .arg("Compiled")
        .env("PATH", "")
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME");
    command.output().unwrap()
}

fn success(output: Output) -> String {
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("\"route\":\"compiled-cpu\""), "{stdout}");
    assert!(stdout.contains("\"outstanding_slots\":0"), "{stdout}");
    stdout
}

fn pngs(directory: &Path) -> Vec<Vec<u8>> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect()
}

fn serial_pixels(bundle: &TimelineBundle) -> Vec<Vec<u8>> {
    let config = fmn_config::Config::resolve(&[], None).unwrap().config;
    let frame = FrameConfig::new(
        Viewport {
            width: 32,
            height: 24,
        },
        ScreenMap {
            scale: 24.0 / config.sizes.frame_height,
            origin: [16.0, 12.0],
            y_up: true,
        },
        fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .unwrap()
            .to_linear(config.camera.background_opacity),
    )
    .with_aa_policy(config.render.aa);
    (0..bundle.frame_count())
        .map(|index| {
            // Original serial reader and a fresh renderer are the oracle. Neither
            // SharedTimelineBundle nor NativeFramePipeline contributes expected bits.
            let stage = bundle.stage_at(index).unwrap();
            let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
                frame,
                tiling: Tiling::default(),
                engine: EngineIdentity::certified(),
                threads: 1,
            })
            .unwrap();
            renderer.render(&stage, 0).unwrap();
            let mut pixels =
                FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap());
            rgba16f_to_rgba8(renderer.frame(), &mut pixels).unwrap();
            pixels.as_bytes().to_vec()
        })
        .collect()
}

#[test]
fn compiled_pure_and_recorded_cli_frames_match_serial_pixels_at_1_4_16_threads() {
    for stateful in [false, true] {
        let root = root();
        let source = root.join("scene.fmtl");
        let bytes = bundle(stateful, false);
        std::fs::write(&source, &bytes).unwrap();
        let reader = TimelineBundle::from_bytes(&bytes).unwrap();
        assert_eq!(reader.frame_count(), 6);
        assert_eq!(
            reader.segment_kind(0),
            Some(if stateful {
                BundleSegmentKind::Stateful
            } else {
                BundleSegmentKind::Pure
            })
        );
        let expected = serial_pixels(&reader);
        assert_ne!(expected[0], expected[3], "fixture must visibly animate");
        let mut first = None;
        for threads in ["1", "4", "16"] {
            let output = root.join(threads);
            let stdout = success(invoke(&source, &output, "png_sequence", threads, &[]));
            assert!(stdout.contains("\"submitted\":6,\"emitted\":6"), "{stdout}");
            let pngs = pngs(&output.join("Compiled"));
            assert_eq!(pngs.len(), expected.len());
            for (png, pixels) in pngs.iter().zip(&expected) {
                assert_eq!(
                    &decode_png(png, &PngLimits::default()).unwrap().rgba,
                    pixels
                );
            }
            if let Some(first) = &first {
                assert_eq!(&pngs, first);
            } else {
                first = Some(pngs);
            }
        }
    }
}

#[test]
fn compiled_stills_subdivisions_and_native_streams_keep_original_frame_selection() {
    let root = root();
    let source = root.join("scene.fmtl");
    std::fs::write(&source, bundle(false, false)).unwrap();
    let all = root.join("all");
    success(invoke(&source, &all, "png_sequence", "1", &[]));
    let expected = pngs(&all.join("Compiled"));
    let still = root.join("still");
    let stdout = success(invoke(&source, &still, "png", "4", &[]));
    assert!(stdout.contains("\"submitted\":1,\"emitted\":1"), "{stdout}");
    assert_eq!(
        std::fs::read(still.join("Compiled.png")).unwrap(),
        expected[5]
    );
    let partial = root.join("partial");
    let stdout = success(invoke(
        &source,
        &partial,
        "png_sequence",
        "4",
        &["--subdivide", "--prerun"],
    ));
    assert!(
        stdout.contains("\"subdivision\":0") && stdout.contains("\"subdivision\":1"),
        "{stdout}"
    );
    // Derive the authored naming rule rather than duplicating it in a test.
    let naming = fmn_scene::OutputNaming {
        output_directory: partial,
        ..Default::default()
    };
    let directory = naming.partial_directory("Compiled");
    assert_eq!(pngs(&directory.join("00000")), expected[..4]);
    assert_eq!(pngs(&directory.join("00001")), expected[4..]);
    for format in ["gif", "y4m"] {
        let mut first = None;
        for threads in ["1", "4", "16"] {
            let output = root.join(format!("{format}-{threads}"));
            success(invoke(&source, &output, format, threads, &[]));
            let bytes = std::fs::read(output.join(format!("Compiled.{format}"))).unwrap();
            if let Some(first) = &first {
                assert_eq!(&bytes, first);
            } else {
                first = Some(bytes);
            }
        }
    }
}

#[test]
fn zero_frame_subdivision_refuses_before_publishing_any_generation() {
    let root = root();
    let source = root.join("scene.fmtl");
    std::fs::write(&source, bundle(false, true)).unwrap();
    let reader = TimelineBundle::from_bytes(&std::fs::read(&source).unwrap()).unwrap();
    assert_eq!(reader.segment_frame_range(1), Some(4..4));
    let output = root.join("output");
    std::fs::create_dir(&output).unwrap();
    let result = invoke(&source, &output, "png", "4", &["--subdivide"]);
    assert!(!result.status.success());
    assert!(
        String::from_utf8(result.stdout)
            .unwrap()
            .contains("zero-frame segment")
    );
    assert_eq!(std::fs::read_dir(output).unwrap().count(), 0);
}
