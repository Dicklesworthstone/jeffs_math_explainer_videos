//! FrankenManim's first-class Rust API.
//!
//! This crate is the public composition root for native Rust scenes. It keeps
//! the subsystem crates available under named modules, while [`prelude`]
//! exposes the small set of names normally needed to write a scene.
//!
//! The facade delegates construction and playback to Proscenium, Choreo, and
//! Marionette. It does not carry a second scene loop or a simplified animation
//! engine.
//!
//! ```
//! use fmn::prelude::*;
//!
//! #[derive(Default)]
//! struct CircleShift;
//!
//! impl SceneConstruct for CircleShift {
//!     fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
//!         let circle = stage.add(Circle::new().radius(0.8).color(BLUE))?;
//!         let movement = circle
//!             .animate()
//!             .set_anim_args(AnimateArgs {
//!                 run_time: Some(0.1),
//!                 ..AnimateArgs::default()
//!             })?
//!             .shift(RIGHT)?;
//!         stage.play(movement)?;
//!         Ok(())
//!     }
//! }
//!
//! let mut sink = NullSceneSink;
//! let completed = run_scene(
//!     &mut CircleShift,
//!     RuntimeConfig::default(),
//!     7,
//!     &mut sink,
//! )?;
//! assert_eq!(completed.report().play_count, 1);
//! # Ok::<(), fmn::Error>(())
//! ```
// Keep the README's native example in the ordinary rustdoc test gate.
#![doc = include_str!("../../../README.md")]
#![forbid(unsafe_code)]

use std::fmt;
use std::num::NonZeroUsize;

use fmn_anim::{
    AnimError, Animation, IntoAnimation, IntoAnimations, SegmentReport, prepare_animation,
    prepare_animations,
};
use fmn_config::ConfigError;
use fmn_geom::{GeomError, SpaceOpsError};
use fmn_library::{
    CoordsError, DashError, DataMobjectError, FieldError, GraphError, ImageError, MatrixError,
    MeshError, NetworkGraphError, NeuralNetworkError, ObjError, ProbabilityError, SamplingError,
    SliderError, SpanCollectorError, TexMobjectError, TextMatchingError, TextMobjectError,
};
use fmn_mobject::{AnimateError, Mob, Mobject, Stage as MobjectStage, StageError};
use fmn_platform::fetch::FetchError;
use fmn_platform::fs::FsError;
use fmn_platform::process::{FfmpegLocatorError, ProcessError};
use fmn_platform::topology::TopologyError;
use fmn_scene::{
    CaptureReason, IntegrationError, LifecycleEvent, LifecyclePhase, PlayOverrides, RuntimeConfig,
    Scene, SceneError, SceneProgram, SceneRunReport, SceneSink,
};
use fmn_tex::{TexEngine, TexError, TexSession, TypesetRequest, TypesetSessionReport};

pub mod exporting;
pub mod prelude;
pub mod rendering;
mod typesetting;
pub use typesetting::TexPreflightError;

pub use exporting::{
    BundleExportError, BundleExportOptions, BundleExportReport, SceneBundleExport, export_bundle,
    export_bundle_bytes, export_bundle_with_fs,
};

pub use rendering::{
    FfmpegCapability, RenderArtifact, RenderError, RenderFormat, RenderOptions, RenderReport,
    SoundtrackReport, render, render_with_fs,
};

/// Built-in native scenes shipped with the standalone binary.
///
/// The primitive corpus is both a useful installation smoke test and G1's
/// public, permissively licensed end-to-end corpus. Keeping its scene
/// definitions here means the CLI and the certified conformance target execute
/// exactly the same native programs.
pub mod builtins {
    use crate::prelude::*;

    /// Stable names in the public G1 primitive corpus.
    pub const PRIMITIVE_SCENE_NAMES: [&str; 25] = [
        "circle_shift.v1",
        "rectangle_shift.v1",
        "triangle_shift.v1",
        "pentagon_shift.v1",
        "arc_shift.v1",
        "dot_shift.v1",
        "ellipse_shift.v1",
        "annulus_shift.v1",
        "line_shift.v1",
        "dashed_line_shift.v1",
        "arrow_shift.v1",
        "circle_scale.v1",
        "rectangle_scale.v1",
        "triangle_scale.v1",
        "hexagon_scale.v1",
        "arc_scale.v1",
        "dot_scale.v1",
        "ellipse_scale.v1",
        "annulus_scale.v1",
        "line_scale.v1",
        "dashed_line_scale.v1",
        "arrow_scale.v1",
        "rounded_rectangle.v1",
        "arc_between_points.v1",
        "layered_polygon.v1",
    ];

    const COLORS: [Srgb; 7] = [BLUE_C, GREEN_B, MAROON_C, RED_C, TEAL_B, YELLOW_C, WHITE];

    // Match Scene::add_mobject exactly without requiring a playback context.
    // Live programs use these constructors before their first stepped frame.
    fn add_builtin_mobject(
        stage: &mut crate::mobject::Stage,
        mobject: impl Into<Mobject>,
    ) -> crate::Result<Mob> {
        let mob = stage.add(mobject);
        stage.add_to_scene(mob)?;
        Ok(mob)
    }

    /// One native program from [`PRIMITIVE_SCENE_NAMES`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PrimitiveScene {
        index: usize,
        name: &'static str,
    }

    impl PrimitiveScene {
        /// Duration of the authored movement before the closing hold.
        pub const PLAY_DURATION: f64 = 0.25;
        /// Duration of the closing hold, rounded as its own native segment.
        pub const WAIT_DURATION: f64 = 0.125;

        /// Construct and root the original primitive family without playback.
        /// Returns the main object whose animation follows this construction.
        ///
        /// # Errors
        /// Preserves the geometry and scene-rooting errors of `construct`.
        pub fn populate(&self, stage: &mut crate::mobject::Stage) -> crate::Result<Mob> {
            let color = COLORS[self.index % COLORS.len()];
            let main = add_builtin_mobject(stage, primitive(self.index, color)?)?;
            stage.set_fill(
                main,
                Some(color),
                Some(0.28 + 0.08 * (self.index % 6) as f64),
                Some(0.0),
                true,
            );
            stage.set_stroke(
                main,
                Some(WHITE),
                Some(1.2 + 0.25 * (self.index % 5) as f64),
                Some(0.95),
                None,
                true,
            );

            // A quiet back layer makes painter ordering and alpha composition
            // part of every sequence, not only the layered fixture.
            let back = add_builtin_mobject(
                stage,
                Circle::new()
                    .radius(0.32 + 0.015 * self.index as f64)
                    .arc_center([-0.55, 0.28, 0.0]),
            )?;
            stage.set_fill(
                back,
                Some(COLORS[(self.index + 3) % COLORS.len()]),
                Some(0.32),
                Some(0.0),
                true,
            );
            stage.set_stroke(back, None, Some(0.0), Some(0.0), None, true);
            stage.set_z_index(back, -1, true);
            stage.add_to_scene(back)?;

            let start_x = -0.72 + 0.06 * (self.index % 5) as f64;
            let start_y = -0.32 + 0.11 * (self.index % 7) as f64;
            stage.shift(main, [start_x, start_y, 0.0]);
            Ok(main)
        }

        /// Build the same fluent movement used by offline playback.
        /// Prepare it at its segment boundary so edits to `main` remain visible.
        ///
        /// # Errors
        /// Preserves the fluent builder's argument-validation errors.
        pub fn animation(
            &self,
            main: Mob,
        ) -> Result<crate::mobject::AnimBuilder, crate::mobject::AnimateError> {
            let builder = main.animate().set_anim_args(AnimateArgs {
                run_time: Some(Self::PLAY_DURATION),
                rate_func: Some(crate::core::rate::linear),
                ..AnimateArgs::default()
            })?;
            if self.index < 11 {
                builder.shift([
                    1.0 + 0.04 * (self.index % 4) as f64,
                    0.18 - 0.03 * (self.index % 3) as f64,
                    0.0,
                ])
            } else if self.index < 22 {
                builder.scale(0.82 + 0.035 * (self.index % 6) as f64)
            } else {
                builder
                    .shift([0.76, 0.16, 0.0])
                    .and_then(|builder| builder.scale(0.9))
            }
        }
    }

    impl SceneConstruct for PrimitiveScene {
        fn name(&self) -> &str {
            self.name
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            let main = self.populate(stage.arena_mut())?;
            stage.play(self.animation(main)?)?;
            stage.wait(Self::WAIT_DURATION)?;
            Ok(())
        }
    }

    /// Resolve one built-in primitive scene by its stable name.
    #[must_use]
    pub fn primitive_scene(name: &str) -> Option<PrimitiveScene> {
        PRIMITIVE_SCENE_NAMES
            .iter()
            .position(|candidate| *candidate == name)
            .map(|index| PrimitiveScene {
                index,
                name: PRIMITIVE_SCENE_NAMES[index],
            })
    }
    /// Stable name of the sound-cue scene outside the pinned G1 corpus.
    ///
    /// The 25-name primitive corpus is digest-pinned by the conformance
    /// scene-runtime goldens, so the audio corpus rides beside it: this scene
    /// is selected by explicit name (never `--write_all`) and exercises the
    /// `Scene.add_sound` request boundary end to end.
    pub const SOUND_CUE_SCENE_NAME: &str = "sound_cue.v1";

    /// The sound-cue scene's asset, resolved by the composition root.
    ///
    /// The path is relative: the caller resolves it against the process
    /// working directory (a checked-in deterministic copy ships under
    /// `crates/fmn/assets/`).
    pub const SOUND_CUE_ASSET_NAME: &str = "sound_cue_tone.v1.wav";

    /// The native sound-cue scene: one shape, one `add_sound` request.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SoundCueScene {
        name: &'static str,
    }

    impl SoundCueScene {
        /// The initial hold before the authored sound request.
        pub const BEFORE_SOUND_DURATION: f64 = 0.5;
        /// The hold after the sound request, rounded independently.
        pub const AFTER_SOUND_DURATION: f64 = 0.5;

        /// Construct the sound scene's visual content without advancing time
        /// or submitting the sound request. The request belongs between the
        /// two holds, at the actual completed native frame time.
        ///
        /// # Errors
        /// Preserves scene-rooting failures.
        pub fn populate(&self, stage: &mut crate::mobject::Stage) -> crate::Result<Mob> {
            let main = add_builtin_mobject(stage, Circle::new().radius(0.6).color(WHITE))?;
            stage.set_fill(main, Some(TEAL_B), Some(0.3), Some(0.0), true);
            stage.set_stroke(main, Some(WHITE), Some(1.2), Some(0.95), None, true);
            Ok(main)
        }
    }

    impl SceneConstruct for SoundCueScene {
        fn name(&self) -> &str {
            self.name
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            self.populate(stage.arena_mut())?;
            stage.wait(Self::BEFORE_SOUND_DURATION)?;
            stage
                .scene_mut()
                .add_sound(SOUND_CUE_ASSET_NAME, 0.0, None, None)
                .map(|_| ())?;
            stage.wait(Self::AFTER_SOUND_DURATION)?;
            Ok(())
        }
    }

    /// Resolve the built-in sound-cue scene by its stable name.
    #[must_use]
    pub fn sound_scene(name: &str) -> Option<SoundCueScene> {
        (name == SOUND_CUE_SCENE_NAME).then_some(SoundCueScene {
            name: SOUND_CUE_SCENE_NAME,
        })
    }

    /// Stable name of the tex-span scene outside the pinned G1 corpus.
    ///
    /// Like the audio corpus, this scene rides beside the digest-pinned
    /// 25-name primitive corpus and is selected by explicit name (never
    /// `--write_all`). It exercises the Studio span-map seam end to end:
    /// Scribe math and text build through
    /// [`crate::library::add_with_spans`] into a
    /// [`crate::library::SpanCollector`] the composition root harvests
    /// into worker state, so the shipped `GET /api/inspect` serves real
    /// typeset span maps instead of an empty registry.
    pub const TEX_SPAN_SCENE_NAME: &str = "tex_span.v1";

    /// The native tex-span scene: one formula with a fraction bar, one
    /// text, both recorded through the span collector.
    #[derive(Clone, Debug)]
    pub struct TexSpanScene {
        name: &'static str,
        collector: crate::library::SpanCollector,
    }

    impl TexSpanScene {
        /// The authored hold after text and mathematics have been constructed.
        pub const WAIT_DURATION: f64 = 0.25;

        /// Consume the scene into the span records gathered at
        /// construct time, in collection order.
        #[must_use]
        pub fn into_span_records(self) -> Vec<crate::library::SpanRecord> {
            self.collector.into_records()
        }

        /// Construct and root both span-mapped values without playback.
        /// Returns the math and text roots, in their original draw-list order;
        /// `into_span_records` retains their native source maps for inspection.
        ///
        /// # Errors
        /// Preserves typesetting, scene-rooting, and span-collection failures.
        pub fn populate(
            &mut self,
            stage: &mut crate::mobject::Stage,
            engine: &crate::tex::TexEngine,
        ) -> crate::Result<[Mob; 2]> {
            let tex = Tex::new(r"\frac{x}{y}").build(engine)?;
            let text = Text::new("hello")
                .build(&FontBook::bundled().map_err(crate::library::TextMobjectError::Text)?)?;
            let (tex_data, text_data) = (tex.span_map(), text.span_map());
            // Record order is the span table's order: root ordinal 0 is
            // the formula, 1 is the text. Both roots share the default
            // z-index, so the stable draw-list sort keeps insertion order
            // and the recorded ordinals hold for every reconstructed
            // frame stage.
            let tex_root = add_builtin_mobject(stage, tex.vmob)?;
            self.collector.record(stage, tex_root, tex_data)?;
            let text_root = add_builtin_mobject(stage, text.vmob)?;
            self.collector.record(stage, text_root, text_data)?;
            Ok([tex_root, text_root])
        }
    }

    impl SceneConstruct for TexSpanScene {
        fn name(&self) -> &str {
            self.name
        }

        fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
            vec![TypesetRequest::math(r"\frac{x}{y}")]
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            self.populate(stage.scene.stage_mut(), stage.typesetting.engine()?)?;
            stage.wait(Self::WAIT_DURATION)?;
            Ok(())
        }
    }

    /// Resolve the built-in tex-span scene by its stable name.
    #[must_use]
    pub fn tex_span_scene(name: &str) -> Option<TexSpanScene> {
        (name == TEX_SPAN_SCENE_NAME).then_some(TexSpanScene {
            name: TEX_SPAN_SCENE_NAME,
            collector: crate::library::SpanCollector::default(),
        })
    }

    /// Stable name of the formula-sheet scene outside the pinned G1 corpus.
    ///
    /// Twenty static display formulas of the kind an explainer typesets on
    /// every draft render. The scene declares all of them as its
    /// [`SceneConstruct::tex_preflight`] manifest, so the front door typesets
    /// them on the worker pool before construction, and every frame comes
    /// after the last layout. A second render with the same persistent typeset
    /// cache serves all twenty from disk. Selected by explicit name only.
    pub const FORMULA_SHEET_SCENE_NAME: &str = "formula_sheet.v1";

    /// The formula sheet's sources, in grid order (four columns, five rows).
    pub const FORMULA_SHEET: [&str; 20] = [
        r"e^{i\pi} + 1 = 0",
        r"a^2 + b^2 = c^2",
        r"\frac{a}{b} + \frac{c}{d} = \frac{ad + bc}{bd}",
        r"x_{1,2} = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}",
        r"\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}",
        r"\int_0^1 x^2 \, dx = \frac{1}{3}",
        r"\prod_{k=1}^{n} k = n!",
        r"\lim_{h \to 0} \frac{f(x+h) - f(x)}{h}",
        r"\binom{n}{k} = \frac{n!}{k!\,(n-k)!}",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
        r"f(x) = \begin{cases} x & x > 0 \\ -x & x \le 0 \end{cases}",
        r"\sqrt[3]{x + 1}",
        r"\nabla \cdot \mathbf{E} = \frac{\rho}{\varepsilon_0}",
        r"\mathbb{E}[X] = \sum_x x \, p(x)",
        r"\sigma^2 = \mathbb{E}\left[(X - \mu)^2\right]",
        r"P(A \mid B) = \frac{P(B \mid A)\, P(A)}{P(B)}",
        r"\left| \sum_i a_i b_i \right| \le \sqrt{\sum_i a_i^2} \sqrt{\sum_i b_i^2}",
        r"\hat{x} + \overline{AB}",
        r"\bar{X}_n = \frac{1}{n} \sum_{i=1}^{n} X_i",
        r"P\left(|\bar{X}_n - \mu| \ge t\right) \le 2 e^{-2 n t^2}",
    ];

    /// The native formula-sheet scene: twenty preflighted formulas, one play.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FormulaSheetScene {
        name: &'static str,
    }

    impl FormulaSheetScene {
        /// Duration of the first formula's movement.
        pub const PLAY_DURATION: f64 = 0.25;
        /// Duration of the closing hold, rounded as its own native segment.
        pub const WAIT_DURATION: f64 = 0.125;

        /// Build the original formula grid without playback. Returns the first
        /// formula to animate and each formula's native source-span record.
        /// The caller supplies the resolved, preflighted native math engine.
        ///
        /// # Errors
        /// Preserves typesetting, scene-rooting, and span-collection failures.
        pub fn populate(
            &self,
            stage: &mut crate::mobject::Stage,
            engine: &crate::tex::TexEngine,
        ) -> crate::Result<(Option<Mob>, Vec<crate::library::SpanRecord>)> {
            const COLUMNS: usize = 4;
            const CELL_WIDTH: f64 = 3.4;
            const CELL_HEIGHT: f64 = 1.45;
            let mut first = None;
            let mut collector = crate::library::SpanCollector::default();
            for (index, source) in FORMULA_SHEET.iter().enumerate() {
                let formula = Tex::new(source).font_size(26.0).build(engine)?;
                let spans = formula.span_map();
                let formula = add_builtin_mobject(stage, formula.vmob)?;
                collector.record(stage, formula, spans)?;
                let width = stage.get_width(formula);
                if width > CELL_WIDTH - 0.3 {
                    stage.scale(formula, (CELL_WIDTH - 0.3) / width);
                }
                let (column, row) = ((index % COLUMNS) as f64, (index / COLUMNS) as f64);
                let center = [(column - 1.5) * CELL_WIDTH, (2.0 - row) * CELL_HEIGHT, 0.0];
                stage.move_to(formula, center, ORIGIN);
                stage.set_fill(
                    formula,
                    Some(COLORS[index % COLORS.len()]),
                    Some(1.0),
                    None,
                    true,
                );
                first.get_or_insert(formula);
            }
            Ok((first, collector.into_records()))
        }

        /// Build the original first-formula movement at its segment boundary.
        ///
        /// # Errors
        /// Preserves the fluent builder's argument-validation errors.
        pub fn animation(
            first: Mob,
        ) -> Result<crate::mobject::AnimBuilder, crate::mobject::AnimateError> {
            first
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(Self::PLAY_DURATION),
                    rate_func: Some(crate::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .shift([0.0, 0.2, 0.0])
        }
    }

    impl SceneConstruct for FormulaSheetScene {
        fn name(&self) -> &str {
            self.name
        }

        fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
            FORMULA_SHEET
                .iter()
                .map(|source| TypesetRequest::math(source))
                .collect()
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            let (first, _) = self.populate(stage.scene.stage_mut(), stage.typesetting.engine()?)?;
            if let Some(first) = first {
                stage.play(Self::animation(first)?)?;
            }
            stage.wait(Self::WAIT_DURATION)?;
            Ok(())
        }
    }

    /// Resolve the built-in formula-sheet scene by its stable name.
    #[must_use]
    pub fn formula_sheet_scene(name: &str) -> Option<FormulaSheetScene> {
        (name == FORMULA_SHEET_SCENE_NAME).then_some(FormulaSheetScene {
            name: FORMULA_SHEET_SCENE_NAME,
        })
    }

    /// Stable name of the semantic-witness scene outside the pinned G1
    /// corpus (fm-5wq.46).
    ///
    /// A bit-locked golden proves a frame unchanged, not right: G1 closed
    /// while every native 2D frame was vertically mirrored (fm-sq8.9). This
    /// static frame is the fixture the semantic sanity oracles
    /// (`fmn-conformance`'s `semantic` module) read on every
    /// golden-producing route. Every element is asymmetric under a mirror
    /// and painted in its own exact colour, so an oracle classifies pixels
    /// without knowing which renderer drew them.
    pub const SEMANTIC_WITNESS_SCENE_NAME: &str = "semantic_witness.v1";

    /// The witness's elements: exact sRGB8 colours, pairwise far apart so an
    /// anti-aliased edge never classifies as another element, and their
    /// scene-space placement in the default 8-unit-high frame.
    pub mod witness {
        use crate::prelude::Vec3;

        /// A filled right triangle in the upper-left quadrant, its right
        /// angle at the top-left corner.
        pub const TRIANGLE: [u8; 3] = [255, 0, 0];
        /// The triangle's vertices.
        pub const TRIANGLE_VERTICES: [Vec3; 3] =
            [[-6.6, 3.6, 0.0], [-4.2, 3.6, 0.0], [-6.6, 1.4, 0.0]];
        /// An F-shaped polygon in the upper-right quadrant: the stem on the
        /// left, the full-width bar on top.
        pub const F_SHAPE: [u8; 3] = [255, 255, 255];
        /// The F's outline.
        pub const F_VERTICES: [Vec3; 10] = [
            [3.2, 0.8, 0.0],
            [3.7, 0.8, 0.0],
            [3.7, 2.0, 0.0],
            [5.0, 2.0, 0.0],
            [5.0, 2.5, 0.0],
            [3.7, 2.5, 0.0],
            [3.7, 3.1, 0.0],
            [5.6, 3.1, 0.0],
            [5.6, 3.6, 0.0],
            [3.2, 3.6, 0.0],
        ];
        /// A dot at `UP * 3`.
        pub const UP_DOT: [u8; 3] = [0, 255, 0];
        /// Its centre.
        pub const UP_DOT_CENTER: Vec3 = [0.0, 3.0, 0.0];
        /// A dot at `LEFT * 5`.
        pub const LEFT_DOT: [u8; 3] = [0, 0, 255];
        /// Its centre.
        pub const LEFT_DOT_CENTER: Vec3 = [-5.0, 0.0, 0.0];
        /// Both dots' radius.
        pub const DOT_RADIUS: f64 = 0.25;
        /// `Text("AB")`, its two glyphs in their own colours.
        pub const TEXT: &str = "AB";
        /// The A.
        pub const TEXT_A: [u8; 3] = [255, 255, 0];
        /// The B.
        pub const TEXT_B: [u8; 3] = [0, 255, 255];
        /// The text's centre.
        pub const TEXT_CENTER: Vec3 = [3.6, -2.2, 0.0];
        /// `Tex("x^2")`, base and superscript in their own colours.
        pub const TEX: &str = "x^2";
        /// The base x.
        pub const TEX_BASE: [u8; 3] = [255, 0, 255];
        /// The superscript 2.
        pub const TEX_SUPERSCRIPT: [u8; 3] = [255, 128, 0];
        /// The formula's centre.
        pub const TEX_CENTER: Vec3 = [-2.6, -2.4, 0.0];
        /// Text and formula size.
        pub const FONT_SIZE: f64 = 96.0;
    }

    /// The native semantic-witness scene: one static frame.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SemanticWitnessScene {
        name: &'static str,
    }

    impl SemanticWitnessScene {
        /// The witness under its stable name.
        #[must_use]
        pub const fn new() -> Self {
            Self {
                name: SEMANTIC_WITNESS_SCENE_NAME,
            }
        }
    }

    impl Default for SemanticWitnessScene {
        fn default() -> Self {
            Self::new()
        }
    }

    fn witness_color(color: [u8; 3]) -> Srgb {
        Srgb::from_rgb8(color[0], color[1], color[2])
    }

    /// Fill `mob` (and its family when `family`) with one opaque colour and
    /// no stroke, so its pixels carry exactly that colour.
    fn paint_solid(stage: &mut crate::mobject::Stage, mob: Mob, color: [u8; 3], family: bool) {
        stage.set_fill(mob, Some(witness_color(color)), Some(1.0), None, family);
        stage.set_stroke(mob, None, Some(0.0), None, None, family);
    }

    /// Paint the point-bearing members of `root`, in family order, one
    /// colour each; the member count must match.
    fn paint_glyphs(
        stage: &mut crate::mobject::Stage,
        root: Mob,
        colors: &[[u8; 3]],
    ) -> crate::Result<()> {
        let glyphs: Vec<Mob> = stage
            .family(root)
            .into_iter()
            .filter(|&member| stage.get_points(member).is_some_and(|p| !p.is_empty()))
            .collect();
        if glyphs.len() != colors.len() {
            return Err(SceneError::Integration(IntegrationError::new(
                "semantic witness",
                format!("expected {} glyphs, found {}", colors.len(), glyphs.len()),
            ))
            .into());
        }
        for (glyph, color) in glyphs.into_iter().zip(colors) {
            paint_solid(stage, glyph, *color, false);
        }
        Ok(())
    }

    impl SemanticWitnessScene {
        /// The authored hold after the semantic witness has been constructed.
        pub const WAIT_DURATION: f64 = 0.25;

        /// Build the original asymmetric geometry and text without playback.
        /// Returns native source-span records for the text and math roots.
        ///
        /// # Errors
        /// Preserves geometry, typesetting, glyph-count, rooting, and span errors.
        pub fn populate(
            &self,
            stage: &mut crate::mobject::Stage,
            engine: &crate::tex::TexEngine,
        ) -> crate::Result<Vec<crate::library::SpanRecord>> {
            let mut collector = crate::library::SpanCollector::default();
            let triangle = add_builtin_mobject(stage, Polygon::new(witness::TRIANGLE_VERTICES))?;
            paint_solid(stage, triangle, witness::TRIANGLE, true);
            let f_shape = add_builtin_mobject(stage, Polygon::new(witness::F_VERTICES))?;
            paint_solid(stage, f_shape, witness::F_SHAPE, true);
            for (center, color) in [
                (witness::UP_DOT_CENTER, witness::UP_DOT),
                (witness::LEFT_DOT_CENTER, witness::LEFT_DOT),
            ] {
                let dot = add_builtin_mobject(
                    stage,
                    Dot::new().point(center).radius(witness::DOT_RADIUS),
                )?;
                paint_solid(stage, dot, color, true);
            }
            let text = Text::new(witness::TEXT)
                .font_size(witness::FONT_SIZE)
                .build(&FontBook::bundled().map_err(crate::library::TextMobjectError::Text)?)?;
            let text_spans = text.span_map();
            let text = add_builtin_mobject(stage, text.vmob)?;
            collector.record(stage, text, text_spans)?;
            stage.move_to(text, witness::TEXT_CENTER, ORIGIN);
            paint_glyphs(stage, text, &[witness::TEXT_A, witness::TEXT_B])?;
            let tex = Tex::new(witness::TEX)
                .font_size(witness::FONT_SIZE)
                .build(engine)?;
            let tex_spans = tex.span_map();
            let tex = add_builtin_mobject(stage, tex.vmob)?;
            collector.record(stage, tex, tex_spans)?;
            stage.move_to(tex, witness::TEX_CENTER, ORIGIN);
            paint_glyphs(stage, tex, &[witness::TEX_BASE, witness::TEX_SUPERSCRIPT])?;
            Ok(collector.into_records())
        }
    }

    impl SceneConstruct for SemanticWitnessScene {
        fn name(&self) -> &str {
            self.name
        }

        fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
            vec![TypesetRequest::math(witness::TEX)]
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            self.populate(stage.scene.stage_mut(), stage.typesetting.engine()?)?;
            // A short hold so every route emits real frames: the CLI refuses
            // a generation with none.
            stage.wait(Self::WAIT_DURATION)?;
            Ok(())
        }
    }

    /// Resolve the built-in semantic-witness scene by its stable name.
    #[must_use]
    pub fn semantic_witness_scene(name: &str) -> Option<SemanticWitnessScene> {
        (name == SEMANTIC_WITNESS_SCENE_NAME).then_some(SemanticWitnessScene::new())
    }

    pub const INTERACTIVE_SCENE_NAME: &str = "interactive.v1";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct InteractiveCanvas;

    impl InteractiveCanvas {
        /// Duration of the original canvas's authored hold.
        pub const WAIT_DURATION: f64 = 2.0;
    }

    impl SceneConstruct for InteractiveCanvas {
        fn name(&self) -> &str {
            INTERACTIVE_SCENE_NAME
        }

        fn construct(&mut self, stage: &mut Stage<'_>) -> crate::Result<()> {
            populate_interactive_canvas(stage.arena_mut())?;
            stage.wait(Self::WAIT_DURATION)?;
            Ok(())
        }
    }

    /// Populate the live canvas without advancing its native scene clock.
    /// Both the eager SceneConstruct and Studio's stepped program use this owner.
    pub fn populate_interactive_canvas(stage: &mut crate::mobject::Stage) -> crate::Result<()> {
        let child = stage.add(
            Circle::new()
                .radius(0.6)
                .arc_center([-1.5, 0.0, 0.0])
                .color(BLUE),
        );
        let group = stage.add(Mobject::new());
        stage.add_to_scene(group)?;
        stage.attach(group, child)?;
        stage.set_fill(child, Some(BLUE), Some(1.0), Some(0.0), true);
        stage.set_stroke(child, None, Some(0.0), Some(0.0), None, true);
        let swatch = stage.add(Square::new().side_length(1.0).color(RED));
        stage.add_to_scene(swatch)?;
        stage.shift(swatch, [1.5, 0.0, 0.0]);
        stage.set_fill(swatch, Some(RED), Some(1.0), Some(0.0), true);
        stage.set_stroke(swatch, None, Some(0.0), Some(0.0), None, true);
        Ok(())
    }

    #[must_use]
    pub fn interactive_scene(name: &str) -> Option<InteractiveCanvas> {
        (name == INTERACTIVE_SCENE_NAME).then_some(InteractiveCanvas)
    }

    fn primitive(index: usize, color: Srgb) -> crate::Result<Mobject> {
        Ok(match index {
            0 | 11 => Circle::new()
                .radius(0.48 + 0.02 * (index % 4) as f64)
                .color(color)
                .into(),
            1 | 12 => Rectangle::new()
                .width(1.1)
                .height(0.62 + 0.03 * (index % 3) as f64)
                .color(color)
                .build()?
                .into(),
            2 | 13 => Mobject::try_from(RegularPolygon::triangle().radius(0.6).color(color))?,
            3 => Mobject::try_from(RegularPolygon::new(5).radius(0.56).color(color))?,
            14 => Mobject::try_from(RegularPolygon::new(6).radius(0.56).color(color))?,
            4 | 15 => Arc::new()
                .start_angle(-0.4)
                .angle(4.2)
                .radius(0.58)
                .color(color)
                .build()?
                .into(),
            5 | 16 => Dot::new().radius(0.23).color(color).into(),
            6 | 17 => Ellipse::new().width(1.15).height(0.58).color(color).into(),
            7 | 18 => Annulus::new()
                .inner_radius(0.22)
                .outer_radius(0.52)
                .color(color)
                .into(),
            8 | 19 => Line::new([-0.58, -0.25, 0.0], [0.58, 0.25, 0.0])
                .path_arc(0.18)
                .color(color)
                .build()?
                .into(),
            9 | 20 => DashedLine::new([-0.62, 0.0, 0.0], [0.62, 0.0, 0.0])
                .dash_length(0.16)
                .positive_space_ratio(0.55)
                .color(color)
                .build()?
                .into(),
            10 | 21 => Arrow::new([-0.58, 0.0, 0.0], [0.58, 0.0, 0.0])
                .buff(0.0)
                .color(color)
                .build()?
                .into(),
            22 => Rectangle::new()
                .width(1.1)
                .height(0.66)
                .corner_radius(0.16)
                .color(color)
                .build()?
                .into(),
            23 => ArcBetweenPoints::new([-0.58, -0.18, 0.0], [0.58, 0.18, 0.0])
                .angle(1.2)
                .color(color)
                .build()?
                .into(),
            // The only remaining corpus index is 24, the layered polygon.
            _ => Mobject::try_from(RegularPolygon::new(7).radius(0.56).color(color))?,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn native_builtin_population_has_no_playback_side_effects() {
            for name in PRIMITIVE_SCENE_NAMES {
                let builtin = primitive_scene(name).unwrap();
                let mut scene = Scene::new(RuntimeConfig::default(), 42).unwrap();
                let main = builtin.populate(scene.stage_mut()).unwrap();
                assert_eq!(scene.time().frames(), 0, "{name}");
                assert_eq!(scene.play_count(), 0, "{name}");
                assert_eq!(scene.stage().time(), 0.0, "{name}");
                assert_eq!(scene.mobjects().len(), 2, "{name}");
                assert_eq!(scene.mobjects()[1], main, "{name}");
                assert_eq!(scene.stage().z_index(scene.mobjects()[0]), -1, "{name}");

                let before = scene.stage().snapshot().to_bytes().unwrap();
                let _builder = builtin.animation(main).unwrap();
                assert_eq!(
                    scene.stage().snapshot().to_bytes().unwrap(),
                    before,
                    "{name}"
                );
            }

            let mut scene = Scene::new(RuntimeConfig::default(), 42).unwrap();
            let sound = sound_scene(SOUND_CUE_SCENE_NAME).unwrap();
            let main = sound.populate(scene.stage_mut()).unwrap();
            assert_eq!(scene.mobjects(), &[main]);
            assert_eq!(scene.time().frames(), 0);
            assert_eq!(scene.play_count(), 0);
            assert!(scene.sound_requests().is_empty());
        }

        #[test]
        fn native_builtin_animation_prepares_from_the_current_edited_geometry() {
            let builtin = primitive_scene("circle_shift.v1").unwrap();
            let mut stage = crate::mobject::Stage::new();
            let main = builtin.populate(&mut stage).unwrap();
            let builder = builtin.animation(main).unwrap();
            // An editor action after registration must survive preparation.
            stage.shift(main, [3.25, -1.5, 0.0]);
            let edited = stage.get_center(main);
            let mut animation = prepare_animation(builder, &mut stage).unwrap();
            animation.begin(&mut stage).unwrap();
            for (actual, expected) in stage.get_center(main).into_iter().zip(edited) {
                assert!((actual - expected).abs() < 1e-6);
            }
            animation.finish(&mut stage);
            let final_center = stage.get_center(main);
            assert!((final_center[0] - edited[0] - 1.0).abs() < 1e-6);
            assert!((final_center[1] - edited[1] - 0.18).abs() < 1e-6);
            assert!((final_center[2] - edited[2]).abs() < 1e-6);
        }

        fn check_spans(stage: &crate::mobject::Stage, records: &[crate::library::SpanRecord]) {
            for record in records {
                let root = stage.roots()[record.root_ordinal];
                let children = stage.get(root).unwrap().submobjects();
                assert_eq!(record.entries.len(), children.len());
                assert!(!record.entries.is_empty());
                for span in &record.entries {
                    assert!(
                        record
                            .source
                            .get(span.start..span.end)
                            .is_some_and(|s| !s.is_empty())
                    );
                }
            }
        }

        fn stage_with_existing_root() -> crate::mobject::Stage {
            let mut stage = crate::mobject::Stage::new();
            let root = stage.add(Mobject::new());
            stage.add_to_scene(root).unwrap();
            stage
        }

        #[test]
        fn native_builtin_span_records_bind_actual_roots_and_child_order() {
            let session = TexSession::default();
            let engine = session.engine().unwrap();

            let mut stage = stage_with_existing_root();
            let mut text = tex_span_scene(TEX_SPAN_SCENE_NAME).unwrap();
            let roots = text.populate(&mut stage, engine).unwrap();
            let records = text.into_span_records();
            assert_eq!(&stage.roots()[1..], &roots);
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].root_ordinal, 1);
            assert_eq!(records[1].root_ordinal, 2);
            assert_eq!(&*records[0].source, r"\frac{x}{y}");
            assert_eq!(&*records[1].source, "hello");
            check_spans(&stage, &records);

            let mut stage = stage_with_existing_root();
            let formula = formula_sheet_scene(FORMULA_SHEET_SCENE_NAME).unwrap();
            let (first, records) = formula.populate(&mut stage, engine).unwrap();
            assert_eq!(first, Some(stage.roots()[1]));
            assert_eq!(stage.roots().len(), 21);
            assert_eq!(records.len(), 20);
            for (index, record) in records.iter().enumerate() {
                assert_eq!(record.root_ordinal, index + 1);
                assert_eq!(&*record.source, FORMULA_SHEET[index]);
            }
            check_spans(&stage, &records);

            let mut stage = stage_with_existing_root();
            let records = SemanticWitnessScene::new()
                .populate(&mut stage, engine)
                .unwrap();
            assert_eq!(stage.roots().len(), 7);
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].root_ordinal, 5);
            assert_eq!(records[1].root_ordinal, 6);
            assert_eq!(&*records[0].source, "AB");
            assert_eq!(&*records[1].source, "x^2");
            check_spans(&stage, &records);
        }
    }
}

/// Substrate constants, colors, rates, deterministic RNG, and value types.
pub mod core {
    pub use fmn_core::*;
}

/// Typed configuration and preamble-pack selection.
pub mod config {
    pub use fmn_config::*;
}

/// Host capability traits and typed capability failures.
pub mod platform {
    pub use fmn_platform::*;
}

/// Chisel geometry types and operations.
pub mod geometry {
    pub use fmn_geom::*;
}

/// Marionette's arena, handles, record buffers, and fluent builders.
pub mod mobject {
    pub use fmn_mobject::*;
}

/// Choreo animations, clocks, and timeline machinery.
pub mod animation {
    pub use fmn_anim::*;
}

/// Scribe text layout and bundled font book.
pub mod text {
    pub use fmn_text::*;
}

/// Scribe mathematics typesetting.
pub mod tex {
    pub use fmn_tex::*;
}

/// Menagerie and Atlas mobjects.
pub mod library {
    pub use fmn_library::*;
}

/// Proscenium scene runtime and event surface.
pub mod scene {
    pub use fmn_scene::*;
}

/// Top-level result type for native Rust scenes.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable front-door error category.
///
/// The numeric values deliberately match the corresponding generated CLI
/// exit-code rows. `usage`, `cancelled`, and `internal` are CLI/host concerns,
/// so native scene construction does not manufacture them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorKind {
    /// Invalid configuration (`fmn` exit 3).
    Config = 3,
    /// A required host capability is unavailable (`fmn` exit 4).
    Capability = 4,
    /// Scene construction or execution failed (`fmn` exit 5).
    Scene = 5,
    /// Host I/O, process execution, or publication failed (`fmn` exit 6).
    Render = 6,
    /// A declared resource budget was exhausted (`fmn` exit 8).
    Budget = 8,
}

impl ErrorKind {
    /// Stable schema spelling shared with the CLI.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Capability => "capability",
            Self::Scene => "scene",
            Self::Render => "render",
            Self::Budget => "budget",
        }
    }

    /// Numeric process status shared with the CLI.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// Errors crossing the native Rust front door.
#[derive(Debug)]
pub enum Error {
    /// Configuration parsing or typed extraction.
    Config(ConfigError),
    /// A geometry constructor or operation.
    Geometry(GeomError),
    /// A bounded public space-ops construction.
    SpaceOps(SpaceOpsError),
    /// A dashed-path construction.
    Dash(DashError),
    /// Arena ownership, family, or record semantics.
    Stage(StageError),
    /// Animation preparation or playback.
    Animation(AnimError),
    /// Scene lifecycle or integration.
    Scene(SceneError),
    /// Native text construction.
    Text(TextMobjectError),
    /// Native math-mobject construction.
    Typesetting(TexMobjectError),
    /// Math-engine initialization or layout.
    TexEngine(TexError),
    /// Declared typesetting preflight admission or outcome-storage budget.
    TexPreflight(TexPreflightError),
    /// Span-record bookkeeping for the Studio span-map seam.
    Span(SpanCollectorError),
    /// Filesystem capability I/O.
    FileSystem(FsError),
    /// Host-provided asset fetch.
    AssetFetch(FetchError),
    /// The ffmpeg-only executable locator.
    FfmpegLocator(FfmpegLocatorError),
    /// The ffmpeg-only process boundary.
    Process(ProcessError),
    /// Host topology discovery.
    Topology(TopologyError),
    /// A native library class (Atlas/Menagerie) refused construction.
    Library(LibraryError),
}

/// The native library's constructor refusals, one variant per class family,
/// so a scene's `construct` can `?` any of them into [`Error`].
#[derive(Debug)]
pub enum LibraryError {
    /// `Axes`, `NumberLine`, labels and calculus helpers.
    Coordinates(CoordsError),
    /// `FunctionGraph`, `ParametricCurve`, `ImplicitFunction`.
    Graph(GraphError),
    /// Range sampling shared by coordinate systems and fields.
    Sampling(SamplingError),
    /// `Matrix` and its integer/decimal/tex variants.
    Matrix(MatrixError),
    /// `BarChart`, `Table`-style data mobjects.
    DataMobject(DataMobjectError),
    /// `SampleSpace` and the probability plane.
    Probability(ProbabilityError),
    /// `VectorField`, `StreamLines`.
    Field(FieldError),
    /// `ImageMobject`.
    Image(ImageError),
    /// `Markdown` documents.
    Markdown(fmn_library::markdown::MarkdownError),
    /// `Union`, `Difference`, `Intersection`, `Exclusion`.
    Boolean(fmn_library::boolean_ops::BooleanMobjectError),
    /// `TransformMatchingTex`/`TransformMatchingStrings` planning.
    TextMatching(TextMatchingError),
    /// `NetworkGraph`.
    NetworkGraph(NetworkGraphError),
    /// `NeuralNetworkMobject`.
    NeuralNetwork(NeuralNetworkError),
    /// `Slider` and the control widgets.
    Slider(SliderError),
    /// Drawing classes that take user-supplied art.
    Drawings(fmn_library::drawings::DrawingsAssetError),
    /// `ThreeDModel` OBJ ingestion.
    Obj(ObjError),
    /// Surface and textured-mesh construction.
    Mesh(MeshError),
    /// Sampled surface meshes.
    SurfaceMesh(fmn_library::solids::SurfaceMeshError),
}

impl LibraryError {
    /// Whether the refusal is a declared resource budget (exit 8), not a
    /// scene defect.
    #[must_use]
    pub const fn is_budget(&self) -> bool {
        const fn sampling(error: &SamplingError) -> bool {
            matches!(
                error,
                SamplingError::LimitExceeded { .. }
                    | SamplingError::CapacityOverflow { .. }
                    | SamplingError::AllocationFailed { .. }
            )
        }
        match self {
            Self::Sampling(error)
            | Self::Coordinates(CoordsError::Sampling(error))
            | Self::Graph(GraphError::Sampling(error)) => sampling(error),
            Self::TextMatching(TextMatchingError::BudgetExceeded { .. }) => true,
            _ => false,
        }
    }

    fn source_error(&self) -> &(dyn std::error::Error + 'static) {
        match self {
            Self::Coordinates(error) => error,
            Self::Graph(error) => error,
            Self::Sampling(error) => error,
            Self::Matrix(error) => error,
            Self::DataMobject(error) => error,
            Self::Probability(error) => error,
            Self::Field(error) => error,
            Self::Image(error) => error,
            Self::Markdown(error) => error,
            Self::Boolean(error) => error,
            Self::TextMatching(error) => error,
            Self::NetworkGraph(error) => error,
            Self::NeuralNetwork(error) => error,
            Self::Slider(error) => error,
            Self::Drawings(error) => error,
            Self::Obj(error) => error,
            Self::Mesh(error) => error,
            Self::SurfaceMesh(error) => error,
        }
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source_error())
    }
}

impl std::error::Error for LibraryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source_error())
    }
}

macro_rules! library_error_from {
    ($source:ty, $variant:ident) => {
        impl From<$source> for LibraryError {
            fn from(error: $source) -> Self {
                Self::$variant(error)
            }
        }

        impl From<$source> for Error {
            fn from(error: $source) -> Self {
                Self::Library(LibraryError::$variant(error))
            }
        }
    };
}

library_error_from!(CoordsError, Coordinates);
library_error_from!(GraphError, Graph);
library_error_from!(SamplingError, Sampling);
library_error_from!(MatrixError, Matrix);
library_error_from!(DataMobjectError, DataMobject);
library_error_from!(ProbabilityError, Probability);
library_error_from!(FieldError, Field);
library_error_from!(ImageError, Image);
library_error_from!(fmn_library::markdown::MarkdownError, Markdown);
library_error_from!(fmn_library::boolean_ops::BooleanMobjectError, Boolean);
library_error_from!(TextMatchingError, TextMatching);
library_error_from!(NetworkGraphError, NetworkGraph);
library_error_from!(NeuralNetworkError, NeuralNetwork);
library_error_from!(SliderError, Slider);
library_error_from!(fmn_library::drawings::DrawingsAssetError, Drawings);
library_error_from!(ObjError, Obj);
library_error_from!(MeshError, Mesh);
library_error_from!(fmn_library::solids::SurfaceMeshError, SurfaceMesh);

impl From<LibraryError> for Error {
    fn from(error: LibraryError) -> Self {
        Self::Library(error)
    }
}

impl Error {
    /// Stable category used by robot integrations and process adapters.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        match self {
            Self::Config(_) => ErrorKind::Config,
            Self::Geometry(
                GeomError::SmoothingSizeOverflow { .. }
                | GeomError::ClosedSmoothingBudgetExceeded { .. }
                | GeomError::SubdivisionBudgetExceeded { .. }
                | GeomError::ToleranceUnreachable { .. }
                | GeomError::ArcComponentOverflow { .. }
                | GeomError::ArcComponentsAboveBudget { .. },
            )
            | Self::SpaceOps(_)
            | Self::TexPreflight(_)
            | Self::Dash(DashError::DashCountOverflow | DashError::TooManyDashes { .. })
            | Self::Stage(StageError::SubmobjectBudgetExceeded { .. })
            | Self::Text(
                TextMobjectError::ResourceLimit { .. }
                | TextMobjectError::CapacityOverflow { .. }
                | TextMobjectError::AllocationFailed { .. },
            )
            | Self::FileSystem(FsError::TooLarge { .. } | FsError::TooManyEntries { .. })
            | Self::AssetFetch(FetchError::TooLarge { .. })
            | Self::FfmpegLocator(
                FfmpegLocatorError::SearchPathLimit { .. }
                | FfmpegLocatorError::ExecutableSizeLimit { .. },
            )
            | Self::Process(
                ProcessError::StdinChunkLimit { .. } | ProcessError::StdinTotalLimit { .. },
            ) => ErrorKind::Budget,
            Self::Library(error) if error.is_budget() => ErrorKind::Budget,
            Self::AssetFetch(_)
            | Self::FfmpegLocator(_)
            | Self::Process(ProcessError::CapabilityAbsent { .. })
            | Self::Topology(_) => ErrorKind::Capability,
            Self::Geometry(_)
            | Self::Dash(_)
            | Self::Stage(_)
            | Self::Animation(_)
            | Self::Scene(
                SceneError::InvalidConfig(_)
                | SceneError::InvalidLifecycle(_)
                | SceneError::InvalidState(_)
                | SceneError::UnboundUpdaters
                | SceneError::Stage(_)
                | SceneError::Animation(_)
                | SceneError::Persist(_)
                | SceneError::Serialize(_)
                | SceneError::Event(_)
                | SceneError::EndScene(_),
            )
            | Self::Text(_)
            | Self::Typesetting(_)
            | Self::TexEngine(_)
            | Self::Span(_)
            | Self::Library(_) => ErrorKind::Scene,
            Self::Scene(SceneError::Camera(_) | SceneError::Integration(_))
            | Self::FileSystem(_)
            | Self::Process(_) => ErrorKind::Render,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "configuration failed: {error}"),
            Self::Geometry(error) => write!(f, "geometry failed: {error}"),
            Self::SpaceOps(error) => write!(f, "space operation failed: {error}"),
            Self::Dash(error) => write!(f, "dash construction failed: {error}"),
            Self::Stage(error) => write!(f, "stage operation failed: {error}"),
            Self::Animation(error) => write!(f, "animation failed: {error}"),
            Self::Scene(error) => write!(f, "{error}"),
            Self::Text(error) => write!(f, "text construction failed: {error}"),
            Self::Typesetting(error) => write!(f, "math construction failed: {error}"),
            Self::TexEngine(error) => write!(f, "math engine failed: {error}"),
            Self::TexPreflight(error) => write!(f, "{error}"),
            Self::Span(error) => write!(f, "span records failed: {error}"),
            Self::FileSystem(error) => write!(f, "{error}"),
            Self::AssetFetch(error) => write!(f, "{error}"),
            Self::FfmpegLocator(error) => write!(f, "{error}"),
            Self::Process(error) => write!(f, "{error}"),
            Self::Topology(error) => write!(f, "{error}"),
            Self::Library(error) => write!(f, "library construction failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Config(error) => error,
            Self::Geometry(error) => error,
            Self::SpaceOps(error) => error,
            Self::Dash(error) => error,
            Self::Stage(error) => error,
            Self::Animation(error) => error,
            Self::Scene(error) => error,
            Self::Text(error) => error,
            Self::Typesetting(error) => error,
            Self::TexEngine(error) => error,
            Self::TexPreflight(error) => error,
            Self::Span(error) => error,
            Self::FileSystem(error) => error,
            Self::AssetFetch(error) => error,
            Self::FfmpegLocator(error) => error,
            Self::Process(error) => error,
            Self::Topology(error) => error,
            Self::Library(error) => error,
        })
    }
}

macro_rules! error_from {
    ($source:ty, $variant:ident) => {
        impl From<$source> for Error {
            fn from(error: $source) -> Self {
                Self::$variant(error)
            }
        }
    };
}

error_from!(ConfigError, Config);
error_from!(GeomError, Geometry);
error_from!(SpaceOpsError, SpaceOps);
error_from!(DashError, Dash);
error_from!(StageError, Stage);
error_from!(AnimError, Animation);
error_from!(TextMobjectError, Text);
error_from!(TexMobjectError, Typesetting);
error_from!(TexError, TexEngine);
error_from!(TexPreflightError, TexPreflight);
error_from!(SpanCollectorError, Span);
error_from!(FsError, FileSystem);
error_from!(FetchError, AssetFetch);
error_from!(FfmpegLocatorError, FfmpegLocator);
error_from!(ProcessError, Process);
error_from!(TopologyError, Topology);

impl From<AnimateError> for Error {
    fn from(error: AnimateError) -> Self {
        Self::Animation(error.into())
    }
}

impl From<SceneError> for Error {
    fn from(error: SceneError) -> Self {
        match error {
            SceneError::Stage(error) => Self::Stage(error),
            SceneError::Animation(error) => Self::Animation(error),
            error => Self::Scene(error),
        }
    }
}

/// Scoped construction and playback context for an ordinary Rust scene.
///
/// `Stage` roots newly added mobjects in the real [`Scene`], prepares fluent
/// `.animate` recordings through Choreo, and forwards captures to the host's
/// [`SceneSink`]. It dereferences to Marionette's arena for positional and
/// style operations; the raw arena type remains available as
/// [`crate::mobject::Stage`].
pub struct Stage<'a> {
    scene: &'a mut Scene,
    sink: &'a mut dyn SceneSink,
    typesetting: &'a TexSession,
}

impl Stage<'_> {
    /// The run's shared native engine, including declared preflight results.
    ///
    /// Use `Tex::new(source).build(stage.tex_engine()?)` rather than creating
    /// another engine inside `construct`. The render host supplies the resolved
    /// template and cache; filesystem-free scene runs use a memory-only session.
    ///
    /// # Errors
    /// Preserves template and bundled-font failures as [`Error::TexEngine`].
    pub fn tex_engine(&self) -> Result<&TexEngine> {
        self.typesetting.engine().map_err(Error::from)
    }

    /// Add a detached mobject to the arena and root it in the scene.
    pub fn add(&mut self, mobject: impl Into<Mobject>) -> Result<Mob> {
        self.scene.add_mobject(mobject).map_err(Error::from)
    }

    /// Prepare and play one animation-like value or a simultaneous pair.
    pub fn play(&mut self, animations: impl IntoAnimations) -> Result<SegmentReport> {
        let animations = prepare_animations(animations, self.scene.stage_mut())?;
        self.play_prepared(animations)?.ok_or({
            Error::Scene(SceneError::InvalidLifecycle(
                "a typed play input unexpectedly became empty",
            ))
        })
    }

    /// Prepare an animation without playing it, for a simultaneous group.
    pub fn prepare(&mut self, animation: impl IntoAnimation) -> Result<Box<dyn Animation>> {
        prepare_animation(animation, self.scene.stage_mut()).map_err(Error::from)
    }

    /// Play already-prepared animations simultaneously with default overrides.
    pub fn play_prepared(
        &mut self,
        animations: Vec<Box<dyn Animation>>,
    ) -> Result<Option<SegmentReport>> {
        self.play_prepared_with(animations, PlayOverrides::default())
    }

    /// Play already-prepared animations with explicit play-level overrides.
    pub fn play_prepared_with(
        &mut self,
        animations: Vec<Box<dyn Animation>>,
        overrides: PlayOverrides,
    ) -> Result<Option<SegmentReport>> {
        self.scene
            .play(animations, overrides, self.sink)
            .map_err(Error::from)
    }

    /// Wait for an explicit number of seconds.
    pub fn wait(&mut self, duration: f64) -> Result<SegmentReport> {
        self.scene
            .wait(Some(duration), self.sink)
            .map_err(Error::from)
    }

    /// Wait for the configured default duration.
    pub fn wait_default(&mut self) -> Result<SegmentReport> {
        self.scene.wait(None, self.sink).map_err(Error::from)
    }

    /// End construction through Proscenium's normal early-termination path.
    pub fn end<T>(&mut self) -> Result<T> {
        self.scene.end().map_err(Error::from)
    }

    /// The underlying Proscenium scene for advanced lifecycle operations.
    #[must_use]
    pub fn scene(&self) -> &Scene {
        self.scene
    }

    /// Mutable access to the underlying Proscenium scene.
    pub fn scene_mut(&mut self) -> &mut Scene {
        self.scene
    }

    /// The underlying Marionette arena.
    #[must_use]
    pub fn arena(&self) -> &MobjectStage {
        self.scene.stage()
    }

    /// Mutable access to the underlying Marionette arena.
    pub fn arena_mut(&mut self) -> &mut MobjectStage {
        self.scene.stage_mut()
    }
}

impl std::ops::Deref for Stage<'_> {
    type Target = MobjectStage;

    fn deref(&self) -> &Self::Target {
        self.scene.stage()
    }
}

impl std::ops::DerefMut for Stage<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.scene.stage_mut()
    }
}

/// The compact lifecycle expected by ordinary native Rust scenes.
///
/// Advanced programs can implement [`SceneProgram`] directly. This trait is
/// the common one-hook front door and returns [`Error`] so geometry,
/// typesetting, and animation failures retain their original typed source.
pub trait SceneConstruct {
    /// Stable scene name used in diagnostics and output naming.
    fn name(&self) -> &str {
        "Scene"
    }

    /// Declare static Tex/TexText requests to warm before `construct` runs.
    ///
    /// Use the same mode, source, preamble and alignment as the constructors.
    /// This is an explicit manifest, not discovery of arbitrary dynamic strings.
    /// Empty manifests do no typesetting or cache I/O. Admission is bounded to
    /// 4096 requests and 4 MiB of combined source/preamble bytes; malformed
    /// formulas preserve their native error and prevent construction/playback.
    fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
        Vec::new()
    }

    /// Construct and play the scene against the real Proscenium runtime.
    fn construct(&mut self, stage: &mut Stage<'_>) -> Result<()>;
}

struct ProgramAdapter<'a, P: ?Sized> {
    program: &'a mut P,
    front_door_error: Option<Error>,
    typesetting: &'a TexSession,
    preflight_workers: NonZeroUsize,
}

impl<P> SceneProgram for ProgramAdapter<'_, P>
where
    P: SceneConstruct + ?Sized,
{
    fn name(&self) -> &str {
        self.program.name()
    }

    fn construct(
        &mut self,
        scene: &mut Scene,
        sink: &mut dyn SceneSink,
    ) -> std::result::Result<(), SceneError> {
        let construction = (|| -> Result<()> {
            typesetting::preflight(self.program, self.typesetting, self.preflight_workers)?;
            self.typesetting.note_construct_begin();
            let mut stage = Stage {
                scene,
                sink,
                typesetting: self.typesetting,
            };
            self.program.construct(&mut stage)
        })();
        if let Err(error) = construction {
            match error {
                Error::Scene(SceneError::EndScene(signal)) => {
                    return Err(SceneError::EndScene(signal));
                }
                error => {
                    let message = error.to_string();
                    self.front_door_error = Some(error);
                    return Err(IntegrationError::new("rust-api", message).into());
                }
            }
        }
        Ok(())
    }
}

/// Forwards every event and capture unchanged, noting the runtime's frame
/// and segment boundaries on the typesetting session so its report can show
/// whether any typesetting happened inside `play()`.
pub(crate) struct SegmentAccountingSink<'a> {
    pub(crate) inner: &'a mut dyn SceneSink,
    pub(crate) typesetting: &'a TexSession,
}

impl SceneSink for SegmentAccountingSink<'_> {
    fn event(&mut self, event: LifecycleEvent) -> std::result::Result<(), IntegrationError> {
        match event.phase {
            // The one-shot preflight point precedes the first capture.
            LifecyclePhase::Preflight => self.typesetting.note_first_frame(),
            LifecyclePhase::PrePlay => self.typesetting.note_segment_begin(),
            LifecyclePhase::PostPlay => self.typesetting.note_segment_end(),
            _ => {}
        }
        self.inner.event(event)
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: fmn_anim::FramePacket,
    ) -> std::result::Result<(), IntegrationError> {
        self.inner.capture(reason, packet)
    }
}

/// A successfully completed native scene.
pub struct CompletedScene {
    scene: Scene,
    report: SceneRunReport,
    typesetting: TypesetSessionReport,
}

impl CompletedScene {
    /// Runtime report from the real Proscenium lifecycle.
    #[must_use]
    pub const fn report(&self) -> &SceneRunReport {
        &self.report
    }

    /// Observed typesetting work at completion; not certified scene identity.
    #[must_use]
    pub const fn typesetting_report(&self) -> &TypesetSessionReport {
        &self.typesetting
    }

    /// Final scene state for inspection, persistence, or another host action.
    #[must_use]
    pub const fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Consume the result and take ownership of the final scene state.
    #[must_use]
    pub fn into_scene(self) -> Scene {
        self.scene
    }
}

/// Run a native scene through the real Proscenium lifecycle.
///
/// The returned value retains the final [`Scene`] rather than discarding the
/// arena after execution. Errors from [`SceneConstruct`] are returned in
/// their original variant; lifecycle failures become [`Error::Scene`].
pub fn run_scene<P>(
    program: &mut P,
    config: RuntimeConfig,
    seed: u64,
    sink: &mut dyn SceneSink,
) -> Result<CompletedScene>
where
    P: SceneConstruct + ?Sized,
{
    run_scene_with_typesetting(
        program,
        config,
        seed,
        sink,
        &TexSession::default(),
        typesetting::DEFAULT_PREFLIGHT_WORKERS,
    )
}

/// Run with an explicitly owned typesetting session shared by preflight and
/// constructors. Unlike [`run_scene`]'s memory-only default, the caller may
/// bind a persistent cache using [`TexSession::with_cache`].
///
/// The worker argument is a ceiling, further bounded to 64 and the native
/// engine's available parallelism. It does not change layout or frame timing.
///
/// # Errors
/// As [`run_scene`], plus bounded preflight admission and native formula errors.
pub fn run_scene_with_typesetting<P>(
    program: &mut P,
    config: RuntimeConfig,
    seed: u64,
    sink: &mut dyn SceneSink,
    typesetting: &TexSession,
    preflight_workers: NonZeroUsize,
) -> Result<CompletedScene>
where
    P: SceneConstruct + ?Sized,
{
    let mut scene = Scene::new(config, seed)?;
    let mut adapter = ProgramAdapter {
        program,
        front_door_error: None,
        typesetting,
        preflight_workers,
    };
    let mut sink = SegmentAccountingSink {
        inner: sink,
        typesetting,
    };
    let run = scene.run(&mut adapter, &mut sink);
    if let Some(error) = adapter.front_door_error {
        return Err(error);
    }
    let report = run?;
    Ok(CompletedScene {
        scene,
        report,
        typesetting: typesetting.report(),
    })
}
