//! Exercise the public scene-to-artifact path, not a separately assembled sink.

use std::path::Path;
use std::sync::Arc;

use fmn::mobject::{Mobject, RecordBuffer, RecordSchema};
use fmn::prelude::*;
use fmn::rendering::{RenderError, RenderPlanLimits};
use fmn_codec::{PngLimits, decode_png, decode_y4m};
use fmn_config::config::{DeterminismMode, Engine, ThreadPolicy};
use fmn_platform::fs::{ATOMIC_DIRECTORY_COMPLETE_LEAF, FileSystem, VirtualFs};

fn options(path: &str, format: RenderFormat, threads: u32) -> RenderOptions {
    let mut options = RenderOptions::new(path).expect("bundled defaults");
    options.format = format;
    options.config.camera.resolution = (96, 64);
    options.config.camera.fps = 10;
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(threads);
    options
}

struct MovingCircle;

impl SceneConstruct for MovingCircle {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let circle = stage.add(Circle::new().radius(0.6).color(WHITE))?;
        stage.set_fill(circle, Some(WHITE), Some(1.0), None, true);
        stage.play(
            circle
                .animate()
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

struct ApexUp;

impl SceneConstruct for ApexUp {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let triangle = stage.add(
            Polygon::new([[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]]).color(WHITE),
        )?;
        stage.set_fill(triangle, Some(WHITE), Some(1.0), None, true);
        stage.set_stroke(triangle, None, Some(0.0), None, None, true);
        Ok(())
    }
}

fn png(fs: &VirtualFs, path: &str) -> fmn_codec::DecodedPng {
    decode_png(
        &fs.read(Path::new(path)).expect("published PNG"),
        &PngLimits::default(),
    )
    .expect("native PNG decodes")
}

fn lit_pixels_on_row(image: &fmn_codec::DecodedPng, row: usize) -> usize {
    let width = image.width as usize;
    image.rgba[row * width * 4..(row + 1) * width * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[0] > 128)
        .count()
}

#[test]
fn static_scene_produces_one_nonuniform_upright_png() {
    let fs = Arc::new(VirtualFs::new());
    let report = render_with_fs(
        &mut ApexUp,
        options("/static", RenderFormat::PngSequence, 1),
        fs.clone(),
    )
    .expect("static scene exports");
    assert_eq!(report.artifact.frame_count, 1);
    assert_eq!(report.scene.play_count, 0);
    assert!(fs.exists(&Path::new("/static").join(ATOMIC_DIRECTORY_COMPLETE_LEAF)));
    let image = png(&fs, "/static/frame_000000.png");
    assert_eq!((image.width, image.height), (96, 64));
    let upper = lit_pixels_on_row(&image, 22);
    let lower = lit_pixels_on_row(&image, 42);
    assert!(upper > 0, "apex reaches the upper half");
    assert!(
        lower > upper * 2,
        "object +Y is image-up, not mirrored: {upper}/{lower}"
    );
    assert!(
        lower < image.width as usize,
        "the background is distinct from the triangle"
    );
}

#[test]
fn animation_has_only_nominal_samples_and_changes_pixels() {
    let fs = Arc::new(VirtualFs::new());
    let report = render_with_fs(
        &mut MovingCircle,
        options("/moving", RenderFormat::PngSequence, 1),
        fs.clone(),
    )
    .expect("animated scene exports");
    assert_eq!(
        report.artifact.frame_count, 3,
        "exact f64 0.2 is above 1/5: upward rounding covers it with three samples"
    );
    assert_eq!(report.scene.play_count, 1);
    assert_eq!(report.emission.stats.emitted, 3);
    let first = png(&fs, "/moving/frame_000000.png");
    let second = png(&fs, "/moving/frame_000001.png");
    assert_ne!(first.rgba, second.rgba);
    let centroid = |image: &fmn_codec::DecodedPng| {
        let mut sum = 0usize;
        let mut count = 0usize;
        for (index, pixel) in image.rgba.as_chunks::<4>().0.iter().enumerate() {
            if pixel[0] > 128 {
                sum += index % image.width as usize;
                count += 1;
            }
        }
        assert!(count > 0);
        sum as f64 / count as f64
    };
    assert!(centroid(&second) > centroid(&first) + 4.0);
}

#[test]
fn certified_png_bytes_are_invariant_under_thread_caps() {
    let mut results = Vec::new();
    for threads in [1, 4, 16] {
        let fs = Arc::new(VirtualFs::new());
        let report = render_with_fs(
            &mut MovingCircle,
            options("/moving", RenderFormat::PngSequence, threads),
            fs.clone(),
        )
        .expect("certified export");
        let frames = (0..report.artifact.frame_count)
            .map(|index| {
                fs.read(&Path::new("/moving").join(format!("frame_{index:06}.png")))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        results.push((report.artifact.digest, frames));
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], results[2]);
}

#[test]
fn native_gif_and_y4m_are_complete_real_streams() {
    for (path, format) in [
        ("/clip.gif", RenderFormat::Gif),
        ("/clip.y4m", RenderFormat::Y4m),
    ] {
        let fs = Arc::new(VirtualFs::new());
        let report = render_with_fs(&mut MovingCircle, options(path, format, 1), fs.clone())
            .expect("native stream");
        assert_eq!(report.artifact.frame_count, 3);
        let bytes = fs.read(Path::new(path)).unwrap();
        assert_eq!(report.artifact.bytes, bytes.len() as u64);
        if format == RenderFormat::Y4m {
            let decoded = decode_y4m(&bytes).expect("decodable native Y4M");
            assert_eq!((decoded.width, decoded.height), (96, 64));
            assert_eq!(decoded.fps, (10, 1));
            assert_eq!(decoded.frames.len(), 3);
            assert_ne!(decoded.frames[0], decoded.frames[1]);
        } else {
            assert!(bytes.starts_with(b"GIF89a"));
            assert_eq!(bytes.last(), Some(&0x3b), "GIF trailer was finalized");
        }
    }
}

struct FailingScene;

impl SceneConstruct for FailingScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Circle::new())?;
        stage.wait(0.2)?;
        Err(SceneError::InvalidConfig("scene's original failure").into())
    }
}

#[test]
fn scene_error_preserves_source_and_aborts_all_formats_before_return() {
    for format in [
        RenderFormat::PngSequence,
        RenderFormat::Gif,
        RenderFormat::Y4m,
    ] {
        let fs = Arc::new(VirtualFs::new());
        let error = render_with_fs(&mut FailingScene, options("/failed", format, 1), fs.clone())
            .expect_err("scene error must abort export");
        assert!(matches!(
            error,
            RenderError::Scene(fmn::Error::Scene(SceneError::InvalidConfig(
                "scene's original failure"
            )))
        ));
        assert!(!fs.exists(Path::new("/failed")));
        // No background abort races the next writer to the same destination.
        render_with_fs(&mut ApexUp, options("/failed", format, 1), fs.clone())
            .expect("immediate retry after joined cancellation");
    }
}

#[test]
fn a_declared_plan_budget_reaches_the_renderer_and_refuses_by_name() {
    let fs = Arc::new(VirtualFs::new());
    let mut config = options("/limited", RenderFormat::PngSequence, 1);
    config.plan_limits = RenderPlanLimits {
        max_retained_segments: 1,
        ..RenderPlanLimits::default()
    };
    let Err(error) = render_with_fs(&mut MovingCircle, config, fs.clone()) else {
        panic!("a circle needs more than one retained segment");
    };
    assert!(
        format!("{error:?}").contains("\"retained segments\""),
        "{error:?}"
    );
    assert!(!fs.exists(Path::new("/limited")));
}

const WIDE_ROWS: usize = 8;
const WIDE_RECORDS: usize = 131_209;

/// One frame of per-record paint: 8 × 131,209 = 1,049,672 knots, just past the default of
/// 1,048,576, with every other part of the plan far under its limit.
struct WideRows;

impl SceneConstruct for WideRows {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        for row in 0..WIDE_ROWS {
            let y = 1.4 - 0.4 * row as f32;
            let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), WIDE_RECORDS).unwrap();
            for i in 0..WIDE_RECORDS {
                let x = -1.5 + 3.0 * i as f32 / (WIDE_RECORDS - 1) as f32;
                let (red, green) = ((row + 1) as f32 / 16.0, (i % 4096) as f32 / 4096.0);
                buffer.write(i, "point", &[x, y, 0.0]);
                buffer.write(i, "stroke_width", &[20.0]);
                buffer.write(i, "stroke_rgba", &[red, green, 0.5, 1.0]);
            }
            let mob = stage.arena_mut().add(Mobject::from_buffer(buffer));
            stage.add_to_scene(mob)?;
        }
        stage.wait(0.1)?;
        Ok(())
    }
}

/// The other budget tests lower a limit, which a clamp to the default would also pass. This one
/// raises it, through the same public route a film takes.
#[test]
fn a_declared_plan_budget_above_the_default_is_admitted_and_the_default_still_refuses() {
    let fs = Arc::new(VirtualFs::new());
    let default = options("/refused", RenderFormat::PngSequence, 4);
    let Err(error) = render_with_fs(&mut WideRows, default, fs.clone()) else {
        panic!("1,049,672 knots should not fit the default budget");
    };
    let words = format!("{error:?}");
    assert!(
        words.contains("resource: \"retained profile knots\"") && words.contains("limit: 1048576"),
        "{words}"
    );
    assert!(!fs.exists(Path::new("/refused")));

    let mut declared = options("/admitted", RenderFormat::PngSequence, 4);
    declared.plan_limits = RenderPlanLimits {
        max_retained_profile_knots: 1 << 21,
        ..RenderPlanLimits::default()
    };
    let report = render_with_fs(&mut WideRows, declared, fs.clone()).unwrap();
    // Binary f64 0.1 lies just above 1/10: the exact nominal clock emits two frames.
    assert_eq!(report.artifact.frame_count, 2);
    assert_eq!(report.emission.stats.emitted, 2);
    assert!(fs.exists(Path::new("/admitted/frame_000000.png")));
    assert!(fs.exists(Path::new("/admitted/frame_000001.png")));
    assert!(!fs.exists(Path::new("/admitted/frame_000002.png")));
    assert_eq!(
        png(&fs, "/admitted/frame_000000.png").rgba,
        png(&fs, "/admitted/frame_000001.png").rgba
    );
}

#[test]
fn frame_budget_failure_does_not_publish_a_truncated_animation() {
    let fs = Arc::new(VirtualFs::new());
    let mut config = options("/limited", RenderFormat::PngSequence, 1);
    config.max_frames = 1;
    assert!(matches!(
        render_with_fs(&mut MovingCircle, config, fs.clone()),
        Err(RenderError::InvalidOptions("scene exceeded max_frames"))
    ));
    assert!(!fs.exists(Path::new("/limited")));
}

#[test]
fn existing_artifacts_are_not_clobbered() {
    let fs = Arc::new(VirtualFs::new());
    fs.insert("/keep.y4m", b"user-owned artifact".to_vec());
    assert!(
        render_with_fs(
            &mut MovingCircle,
            options("/keep.y4m", RenderFormat::Y4m, 1),
            fs.clone()
        )
        .is_err()
    );
    assert_eq!(
        fs.read(Path::new("/keep.y4m")).unwrap(),
        b"user-owned artifact"
    );
}

#[test]
fn invalid_options_fail_without_publishing_or_running_the_scene() {
    struct MustNotRun;
    impl SceneConstruct for MustNotRun {
        fn construct(&mut self, _: &mut Stage<'_>) -> fmn::Result<()> {
            panic!("validation must precede scene execution");
        }
    }
    for case in 0..8 {
        let fs = Arc::new(VirtualFs::new());
        let mut config = options("/invalid", RenderFormat::Y4m, 1);
        match case {
            0 => config.config.camera.fps = 0,
            1 => config.config.camera.resolution.0 = 95,
            2 => config.config.sizes.frame_height = f64::NAN,
            3 => config.frames_in_flight = 0,
            4 => config.max_frames = 0,
            5 => config.max_resident_bytes = 1,
            6 => config.config.render.engine = Engine::Metal,
            _ => config.config.render.threads = ThreadPolicy::Fixed(0),
        }
        assert!(
            render_with_fs(&mut MustNotRun, config, fs.clone()).is_err(),
            "case {case}"
        );
        assert!(!fs.exists(Path::new("/invalid")));
    }
}

#[test]
fn unwinding_scene_panic_aborts_and_joins_output() {
    struct Panicking;
    impl SceneConstruct for Panicking {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            stage.wait(0.1)?;
            panic!("scene panic after a capture");
        }
    }
    let fs = Arc::new(VirtualFs::new());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = render_with_fs(
            &mut Panicking,
            options("/panic", RenderFormat::PngSequence, 1),
            fs.clone(),
        );
    }));
    assert!(result.is_err());
    assert!(!fs.exists(Path::new("/panic")));
    render_with_fs(
        &mut ApexUp,
        options("/panic", RenderFormat::PngSequence, 1),
        fs,
    )
    .expect("panic cleanup completed before control returned");
}

#[test]
fn adjacent_float_durations_straddle_the_exact_frame_boundary() {
    struct DurationScene(f64);
    impl SceneConstruct for DurationScene {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            let circle = stage.add(Circle::new())?;
            stage.play(
                circle
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(self.0),
                        rate_func: Some(fmn::core::rate::linear),
                        ..AnimateArgs::default()
                    })?
                    .shift(RIGHT)?,
            )?;
            Ok(())
        }
    }
    for (duration, count) in [(f64::from_bits(0.2f64.to_bits() - 1), 2), (0.2, 3)] {
        let fs = Arc::new(VirtualFs::new());
        let report = render_with_fs(
            &mut DurationScene(duration),
            options("/boundary", RenderFormat::PngSequence, 1),
            fs.clone(),
        )
        .unwrap();
        assert_eq!(report.artifact.frame_count, count);
        assert_eq!(report.emission.stats.emitted, count);
        assert!(fs.exists(&Path::new("/boundary").join(format!("frame_{:06}.png", count - 1))));
        assert!(!fs.exists(&Path::new("/boundary").join(format!("frame_{count:06}.png"))));
    }
}

/// A native calculus scene written the way a user writes one: every library
/// constructor (`Axes`, graphs, labels, Riemann rectangles) propagates with a
/// plain `?` into `fmn::Result`.
struct LabelledAxes;

impl SceneConstruct for LabelledAxes {
    fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
        vec![
            TypesetRequest::math("x"),
            TypesetRequest::math("y"),
            TypesetRequest::math("f(x)"),
        ]
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let book = FontBook::bundled().map_err(fmn::library::TextMobjectError::Text)?;
        let axes = Axes::new()
            .x_range([-1.0, 4.0, 1.0])
            .y_range([-1.0, 2.0, 1.0])
            .build(&book)?;
        let f = |x: f64| 0.5 * x;
        let graph = axes.get_graph(f).color(YELLOW).build()?;
        let labels = axes.get_axis_labels(stage.tex_engine()?, "x", "y")?;
        let label =
            axes.get_graph_label_tex(stage.tex_engine()?, &f, &graph, "f(x)", None, None)?;
        let area = axes.get_riemann_rectangles(&f, Some([0.0, 2.0]), Some(0.5), "left")?;
        let v_line = axes.get_v_line_to_graph(1.0, &f)?;
        stage.add(axes.into_vmob())?;
        for mobject in [graph, labels, label, area, v_line] {
            stage.add(mobject)?;
        }
        Ok(())
    }
}

#[test]
fn a_labelled_axes_scene_renders_through_the_front_door() {
    let fs = Arc::new(VirtualFs::new());
    let mut options = options("/axes", RenderFormat::PngSequence, 1);
    options.config.camera.resolution = (160, 90);
    options.config.sizes.frame_height = 8.0;
    let report = render_with_fs(&mut LabelledAxes, options, fs.clone()).expect("axes render");
    assert_eq!(report.artifact.frame_count, 1);
    let image = png(&fs, "/axes/frame_000000.png");
    let lit: usize = (0..image.height as usize)
        .map(|row| lit_pixels_on_row(&image, row))
        .sum();
    assert!(lit > 100, "axes, graph and labels are drawn: {lit}");
    // The Riemann rectangles' BLUE -> GREEN gradient: green well above red.
    let area = image
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[1] > 100 && pixel[1] > pixel[0].saturating_add(40))
        .count();
    assert!(area > 50, "the area under the graph is drawn: {area}");
}

struct BadSampleType;

impl SceneConstruct for BadSampleType {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let book = FontBook::bundled().map_err(fmn::library::TextMobjectError::Text)?;
        let axes = Axes::new().build(&book)?;
        let area = axes.get_riemann_rectangles(&|x| x, None, None, "middle")?;
        stage.add(area)?;
        Ok(())
    }
}

#[test]
fn library_refusals_cross_the_front_door_typed_and_classified() {
    let fs = Arc::new(VirtualFs::new());
    let error = render_with_fs(
        &mut BadSampleType,
        options("/refused", RenderFormat::PngSequence, 1),
        fs.clone(),
    )
    .expect_err("an invalid sample type refuses the scene");
    let RenderError::Scene(error) = error else {
        panic!("a scene-side refusal: {error:?}")
    };
    assert!(matches!(
        &error,
        fmn::Error::Library(LibraryError::Coordinates(
            fmn::library::CoordsError::InvalidSampleType(kind)
        )) if kind == "middle"
    ));
    assert_eq!(error.kind(), fmn::ErrorKind::Scene);
    assert!(!fs.exists(Path::new("/refused")));

    let budget = fmn::Error::from(fmn::library::CoordsError::Sampling(
        fmn::library::SamplingError::LimitExceeded {
            context: "axis ticks",
            max_samples: 16,
        },
    ));
    assert_eq!(budget.kind(), fmn::ErrorKind::Budget);
    assert!(std::error::Error::source(&budget).is_some());
}

struct LitSphere;

impl SceneConstruct for LitSphere {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(Sphere::new(1.0))?;
        Ok(())
    }
}

#[test]
fn surface_content_names_the_camera_route_and_renders_on_it() {
    let fs = Arc::new(VirtualFs::new());
    let error = render_with_fs(
        &mut LitSphere,
        options("/sphere", RenderFormat::PngSequence, 1),
        fs.clone(),
    )
    .expect_err("a surface cannot take the planar vector route");
    let message = error.to_string();
    assert!(
        message.contains("camera route") && message.contains("RenderOptions::camera"),
        "{message}"
    );
    assert!(!fs.exists(Path::new("/sphere")));

    let mut options = options("/sphere", RenderFormat::PngSequence, 1);
    options.camera = Some(options.camera_config().expect("camera config"));
    let report = render_with_fs(&mut LitSphere, options, fs.clone()).expect("camera route");
    assert_eq!(report.artifact.frame_count, 1);
    let image = png(&fs, "/sphere/frame_000000.png");
    let shaded = (0..image.height as usize)
        .map(|row| lit_pixels_on_row(&image, row))
        .sum::<usize>();
    assert!(shaded > 0, "the sphere is drawn");
}
