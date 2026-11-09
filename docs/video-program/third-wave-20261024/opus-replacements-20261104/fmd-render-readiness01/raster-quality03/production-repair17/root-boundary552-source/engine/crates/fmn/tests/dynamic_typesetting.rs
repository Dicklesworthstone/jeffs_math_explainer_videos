//! Data-dependent formula batches use the production session before playback.
#![forbid(unsafe_code)]

use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

use fmn::prelude::*;
use fmn::tex::{Mode, Style as MathStyle, TexError};
use fmn_config::config::{DeterminismMode, ThreadPolicy};
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::{FileSystem, VirtualFs};

const PREAMBLE: &str = r"\newcommand{\datum}{x}";

fn cache_root() -> &'static str {
    if cfg!(windows) {
        r"C:\dynamic-typesetting"
    } else {
        "/dynamic-typesetting"
    }
}

struct DataScene {
    workers: NonZeroUsize,
    layouts_at_build: u64,
    layouts_after_play: u64,
}

impl DataScene {
    fn new(workers: usize) -> Self {
        Self {
            workers: NonZeroUsize::new(workers).unwrap(),
            layouts_at_build: 0,
            layouts_after_play: 0,
        }
    }
}

impl SceneConstruct for DataScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // These strings are discovered inside construct, not its static
        // manifest. Each macro invocation has a distinct native layout key.
        let sources: Vec<_> = (1..=24)
            .map(|n| format!(r"\frac{{\datum+{n}}}{{{n}}}"))
            .collect();
        let requests: Vec<_> = sources
            .iter()
            .map(|source| {
                let mut request = TypesetRequest::math(source);
                request.preamble = PREAMBLE;
                request
            })
            .collect();
        let before = stage.scene().time();
        stage.preflight_tex(&requests, self.workers)?;
        assert_eq!(stage.scene().time(), before);
        self.layouts_at_build = stage.tex_engine()?.layout_computations();
        for (index, source) in sources.iter().enumerate() {
            let object = Tex::new(source)
                .preamble(PREAMBLE)
                .build(stage.tex_engine()?)?;
            let mob = stage.add(object)?;
            stage.scale(mob, 0.3);
            stage.shift(
                mob,
                [
                    (index % 6) as f64 * 0.7 - 1.75,
                    1.2 - (index / 6) as f64 * 0.8,
                    0.0,
                ],
            );
        }
        // Constructor scale calibration must already be warm, too.
        assert_eq!(
            stage.tex_engine()?.layout_computations(),
            self.layouts_at_build
        );
        stage.wait(0.25)?;
        self.layouts_after_play = stage.tex_engine()?.layout_computations();
        Ok(())
    }
}

fn options(output: &str, persistent: bool) -> RenderOptions {
    let mut options = RenderOptions::new(output).unwrap();
    options.config.camera.resolution = (96, 64);
    options.config.camera.fps = 8;
    options.config.sizes.frame_height = 4.0;
    options.config.determinism.mode = DeterminismMode::Certified;
    options.config.render.threads = ThreadPolicy::Fixed(1);
    options.config.directories.cache = cache_root().to_owned();
    options.typeset_cache = persistent;
    options
}

fn frames(fs: &VirtualFs, output: &str) -> Vec<Vec<u8>> {
    (0..2)
        .map(|n| {
            fs.read(&Path::new(output).join(format!("frame_{n:06}.png")))
                .unwrap()
        })
        .collect()
}

#[test]
fn twenty_four_dynamic_formulas_warm_across_sessions_without_layout_inside_play() {
    let fs = Arc::new(VirtualFs::new());
    let mut cold_scene = DataScene::new(1);
    let cold = render_with_fs(&mut cold_scene, options("/cold", true), fs.clone()).unwrap();
    assert!(cold_scene.layouts_at_build >= 24);
    assert_eq!(cold_scene.layouts_at_build, cold_scene.layouts_after_play);
    assert!(cold.typesetting.persistent);
    assert!(cold.typesetting.memory.hits >= 24);
    assert_eq!(cold.artifact.frame_count, 2);

    let mut warm_scene = DataScene::new(4);
    let warm = render_with_fs(&mut warm_scene, options("/warm", true), fs.clone()).unwrap();
    assert_eq!(warm.typesetting.layout_computations, 0);
    assert!(warm.typesetting.persistent_hits >= 25);
    assert_eq!(warm_scene.layouts_after_play, 0);
    assert_eq!(frames(&fs, "/cold"), frames(&fs, "/warm"));

    let memory_fs = Arc::new(VirtualFs::new());
    let mut memory_scene = DataScene::new(4);
    let memory = render_with_fs(
        &mut memory_scene,
        options("/memory", false),
        memory_fs.clone(),
    )
    .unwrap();
    assert!(!memory.typesetting.persistent);
    assert_eq!(
        memory_scene.layouts_at_build,
        memory_scene.layouts_after_play
    );
    assert!(!memory_fs.exists(Path::new(cache_root())));
    let reference = frames(&fs, "/cold");
    assert_eq!(reference, frames(&memory_fs, "/memory"));
    let image = fmn_codec::decode_png(&reference[0], &fmn_codec::PngLimits::default()).unwrap();
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[0] > 128)
    );
}

struct RejectedBatch {
    source: String,
    count: usize,
    preamble: String,
}

impl SceneConstruct for RejectedBatch {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut request = TypesetRequest::math(&self.source);
        request.preamble = &self.preamble;
        stage.preflight_tex(&vec![request; self.count], NonZeroUsize::MIN)?;
        panic!("an over-budget dynamic batch must be refused")
    }
}

#[test]
fn dynamic_admission_refuses_count_source_preamble_and_total_bytes_before_cache_io() {
    for (length, count, preamble) in [
        (1, 4097, 0),
        (262_145, 1, 0),
        (1, 1, 262_145),
        (262_144, 17, 0),
    ] {
        let fs = Arc::new(VirtualFs::new());
        let config = options("/unused", true).config;
        let session = TexSession::with_cache(&config, fs.clone(), Arc::new(FakeClock::new()));
        let mut program = RejectedBatch {
            source: "x".repeat(length),
            count,
            preamble: "x".repeat(preamble),
        };
        let result = run_scene_with_typesetting(
            &mut program,
            RuntimeConfig::default(),
            0,
            &mut NullSceneSink,
            &session,
            NonZeroUsize::MIN,
        );
        let error = result.err().expect("dynamic batch admission must fail");
        assert_eq!(error.kind(), ErrorKind::Budget);
        assert_eq!(session.report(), TypesetSessionReport::default());
        assert!(!fs.exists(Path::new(cache_root())));
    }
}

struct RecoverWithinConstruction;

impl SceneConstruct for RecoverWithinConstruction {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.wait(0.125)?;
        let time = stage.scene().time();
        let result = stage.preflight_tex(
            &[
                TypesetRequest::math(r"\fmnUnknownConstruct"),
                TypesetRequest::math("x+1"),
            ],
            NonZeroUsize::new(4).unwrap(),
        );
        assert!(matches!(result, Err(Error::TexEngine(TexError::Math(_)))));
        assert_eq!(stage.scene().time(), time);
        let engine = stage.tex_engine()?;
        let layouts = engine.layout_computations();
        engine.typeset(Mode::Math(MathStyle::Display), "x+1")?;
        assert_eq!(engine.layout_computations(), layouts);
        Ok(())
    }
}

#[test]
fn a_caught_dynamic_formula_failure_preserves_time_and_later_successful_cache_entries() {
    run_scene(
        &mut RecoverWithinConstruction,
        RuntimeConfig::default(),
        0,
        &mut NullSceneSink,
    )
    .unwrap();
}

struct EmptyBatch;

impl SceneConstruct for EmptyBatch {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.preflight_tex(&[], NonZeroUsize::MIN)?;
        stage.add(Circle::new())?;
        Ok(())
    }
}

#[test]
fn empty_dynamic_batches_leave_the_typesetting_session_lazy() {
    let fs = Arc::new(VirtualFs::new());
    let mut options = options("/empty", true);
    options.config.tex.template = "unused-template".to_owned();
    let report = render_with_fs(&mut EmptyBatch, options, fs.clone()).unwrap();
    assert_eq!(report.typesetting, TypesetSessionReport::default());
    assert!(!fs.exists(Path::new(cache_root())));
}
