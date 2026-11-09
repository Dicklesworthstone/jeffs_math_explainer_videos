//! Self-goldens at scale (§16.3 plane 2, D-16, fm-t1v W10): the ~25-scene
//! primitive-and-feature corpus, bit-locked at two lifecycle points.
//!
//! `tests/self_goldens.rs` locks one geometry lifecycle and one stage
//! lifecycle; `tests/certified_engine.rs` locks three engine corpora. This
//! module is the corpus between them: one deterministic [`Stage`]
//! composition per scene over the landed class families — arc, line, poly,
//! brace, matchers, text, tex, numbers, matrix, special tex, controls,
//! coords, graphs — rendered by the **certified CPU engine only** and locked
//! through the [`crate::golden`] rig.
//!
//! ## What one artifact locks
//!
//! Every scene serializes one canonical document ([`CORPUS_SCHEMA`], the
//! §6.7 durable form — defined field order, float canonicalization, trailing
//! checksum) holding:
//!
//! 1. the **post-construct** geometry snapshot — every family member's
//!    world-space records at `f32` record precision, the same form
//!    `tests/self_goldens.rs` established;
//! 2. the **post-construct** certified frame, encoded into its canonical
//!    document by the test target's renderer (`fmn-frame` is a documented
//!    dev-only edge of this crate, so the frame itself is produced in the
//!    test target and injected into [`artifact`]);
//! 3. the **post-transform** geometry snapshot, after
//!    [`apply_lifecycle_transform`] (a per-scene-index deterministic shift +
//!    scale, the static stand-in for "mid-animation": the same positional
//!    pipeline an animation would drive, with no clock involved);
//! 4. the **post-transform** certified frame.
//!
//! The lock entry's SHA-256 is therefore simultaneously the frame hash and
//! the geometry hash — a drift in either fails the same check.
//!
//! ## Why `Scope::Certified`
//!
//! The convention `tests/certified_engine.rs` states: since §5 is frozen
//! and the certified arithmetic is portable, rendered artifacts get **one
//! lock shared by the whole matrix** rather than per-platform ones — a
//! shared lock fails on whichever machine breaks it instead of passing
//! everywhere until someone re-runs a sweep. Text and TeX shaping are
//! sovereign (bundled faces, own shaper), so they ride the same promise.
//!
//! ## Determinism
//!
//! No RNG, no wall clock, no filesystem reads beyond the bundled faces:
//! every scene is a pure function of [`Corpus`] (the bundled `FontBook` and
//! `TexEngine`, built once per process) and its own index. Blessing is the
//! rig's flow: `UPDATE_GOLDENS=1 cargo test -p fmn-conformance --test
//! scene_goldens`, review the lock diff, commit it. GOVERNANCE §5 applies —
//! a drift is a finding to adjudicate, never a number to re-bless.
//!
//! The frame geometry matches `tests/certified_engine.rs` exactly
//! (320×180 @ 60 px/unit, the declared certified tiling, the Reference's
//! `#333333` background) so a reviewer can diff panels across suites without
//! rescaling.

use fmn_core::color::Srgb;
use fmn_core::constants::{
    BLUE_C, DOWN, GOLD_C, GREEN_B, MAROON_C, PURPLE_B, RED_C, TAU, TEAL_B, WHITE, YELLOW_C,
};
use fmn_hash::{Digest, Schema, Writer, sha256};
use fmn_library::style::Style;
use fmn_library::vmobject::{VMobject, v_group};
use fmn_library::{
    AnnularSector, Annulus, Arc, ArcBetweenPoints, Arrow, Axes, BraceLabel, BulletedList, Button,
    Checkbox, Circle, CubicBezier, DashedLine, DecimalMatrix, DecimalNumber, Dot, Elbow, Ellipse,
    EnableDisableButton, FunctionGraph, ImplicitFunction, Integer, IntegerMatrix, Line, NumberLine,
    ParametricCurve, Polygon, Rectangle, RegularPolygon, StrokeArrow, SurroundingRectangle, Tex,
    Text, Title, checkmark, cross, exmark, underline,
};
use fmn_mobject::{Mob, Stage};
use fmn_render::bin::{ScreenMap, Tiling, Viewport};
use fmn_render::engine::FrameConfig;
use fmn_tex::TexEngine;
use fmn_text::FontBook;
use std::collections::BTreeSet;

use crate::golden::{GoldenStore, Scope};

/// The frame every scene renders into: `tests/certified_engine.rs`'s
/// geometry, so panels diff across suites without rescaling.
pub const WIDTH: u32 = 320;
/// Frame height in pixels.
pub const HEIGHT: u32 = 180;
/// Pixels per scene unit (the Reference's 1080p density halved).
pub const SCALE: f64 = 60.0;
/// The declared certified configuration's tile dimensions (C10).
pub const TILING: Tiling = Tiling {
    macro_tile: 128,
    fine_tile: 16,
};

/// Schema family for the scene-corpus artifact documents.
pub const CORPUS_SCHEMA: Schema = Schema::new(*b"FMNS", 21, 1, 0);

/// The lock-file family these artifacts live under.
pub const SUITE: &str = "scene_goldens";

/// Canonical first row of the certified corpus lock.
pub const CERTIFIED_LOCK_HEADER: &str = "# fmn-golden-lock v1 suite=scene_goldens key=certified";

/// The committed lock bytes that bind the corpus accepted by performance
/// producers and certified-matrix tests.
pub const CERTIFIED_LOCK: &str = include_str!("../goldens/scene_goldens.certified.lock");

/// SHA-256 identity of [`CERTIFIED_LOCK`].
#[must_use]
pub fn certified_lock_digest() -> Digest {
    sha256(CERTIFIED_LOCK.as_bytes())
}

/// Validate that the embedded certified lock and compiled corpus name exactly
/// the same bounded scene set.
///
/// This intentionally validates row syntax as well as names: performance
/// definitions bind the complete lock bytes, so admitting a malformed length
/// or digest column would make their workload identity stronger than the
/// artifact the golden rig can actually consume.
///
/// # Errors
/// Returns a line-attributed message for a malformed, missing, duplicate, or
/// stale lock row.
pub fn validate_certified_lock() -> Result<(), String> {
    let mut lines = CERTIFIED_LOCK.lines();
    if lines.next() != Some(CERTIFIED_LOCK_HEADER) {
        return Err("scene-golden lock header does not match the certified v1 schema".to_owned());
    }
    let expected: BTreeSet<_> = SCENES.iter().map(|case| case.name).collect();
    let mut actual = BTreeSet::new();
    for (index, line) in lines.enumerate() {
        let mut fields = line.split('\t');
        let name = fields.next().unwrap_or_default();
        let length = fields.next().unwrap_or_default();
        let digest = fields.next().unwrap_or_default();
        if name.is_empty()
            || fields.next().is_some()
            || length.parse::<u64>().is_err()
            || Digest::from_hex(digest).is_err()
        {
            return Err(format!("malformed scene-golden lock row {}", index + 2));
        }
        if !actual.insert(name) {
            return Err(format!("duplicate scene-golden lock row {name:?}"));
        }
    }
    if actual != expected {
        let missing: Vec<_> = expected.difference(&actual).copied().collect();
        let stale: Vec<_> = actual.difference(&expected).copied().collect();
        return Err(format!(
            "scene-golden lock/corpus mismatch: missing {missing:?}, stale {stale:?}"
        ));
    }
    Ok(())
}

/// The golden store for this corpus: one lock shared by the certified
/// matrix (see the module docs).
///
/// # Panics
///
/// The suite name is a compile-time constant that satisfies the rig's
/// character rules, so construction cannot fail.
#[must_use]
pub fn store() -> GoldenStore {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens");
    GoldenStore::new(dir, SUITE, Scope::Certified).expect("scene_goldens store")
}

/// The certified frame configuration: 320×180 @ 60 px/unit, object-space
/// origin at the frame centre, the Reference's `#333333` background.
#[must_use]
pub fn frame_config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        ScreenMap {
            scale: SCALE,
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: true,
        },
        Srgb::from_rgb8(0x33, 0x33, 0x33).to_linear(1.0),
    )
}

/// The deterministic construction context shared by every scene: the
/// bundled text and TeX engines, built once per process. Neither engine
/// reads the filesystem, the clock, or any RNG, so scenes are pure
/// functions of it.
pub struct Corpus {
    /// The bundled sovereign font book (Computer Modern, CM Typewriter,
    /// IBM Plex Sans).
    pub book: FontBook,
    /// The TeX engine over the default bundled macro pack.
    pub tex: TexEngine,
}

impl Corpus {
    fn build() -> Self {
        Self {
            book: FontBook::bundled().expect("bundled faces parse"),
            tex: TexEngine::new("fmd-math/pack/default", None).expect("bundled pack loads"),
        }
    }
}

/// The process-wide corpus: the bundled text and TeX engines, built on
/// first use.
pub fn corpus() -> &'static Corpus {
    static CORPUS: std::sync::LazyLock<Corpus> = std::sync::LazyLock::new(Corpus::build);
    &CORPUS
}

/// A built scene: the stage plus the ordered scene roots the geometry
/// snapshot walks and the lifecycle transform moves.
pub struct Built {
    /// The scene graph, post-construct.
    pub stage: Stage,
    /// Root handles, in painter order.
    pub roots: Vec<Mob>,
}

/// One corpus scene: a lock name and a deterministic constructor.
pub struct SceneCase {
    /// The lock entry name (`[a-z0-9._-]`, `.v1`-suffixed).
    pub name: &'static str,
    /// Build the post-construct stage.
    pub build: fn(&Corpus) -> Built,
}

/// Add one composed VMobject root to a fresh stage.
fn stage_of(root: VMobject) -> Built {
    let mut stage = Stage::new();
    let mob = stage.add(root);
    stage.add_to_scene(mob).expect("the root joins the scene");
    Built {
        stage,
        roots: vec![mob],
    }
}

/// Render a stage through one explicitly journaled engine identity is the
/// *test targets'* job: `fmn-frame` is a documented dev-only edge of this
/// crate (see Cargo.toml), so the conformance library holds everything up to
/// the frame — the corpus, the snapshot form, the transform, the frame
/// configuration — and the suites inject their renderer into [`artifact`].
///
/// Append every family member's world-space records at `f32` record
/// precision: member count, then per member the flattened point count and
/// the point run — the same form `tests/self_goldens.rs` established.
pub fn snapshot_geometry(writer: &mut Writer, stage: &Stage, roots: &[Mob]) {
    let mut members = 0_u64;
    for &root in roots {
        members += stage.family(root).len() as u64;
    }
    writer.put_u64(members);
    for &root in roots {
        for member in stage.family(root) {
            let points = stage.get_points(member).unwrap_or_default();
            writer.put_u64((points.len() * 3) as u64);
            for point in points {
                for component in point {
                    #[allow(clippy::cast_possible_truncation)]
                    writer.put_f32(component as f32);
                }
            }
        }
    }
}

/// The post-transform lifecycle step: a small shift and scale, varied
/// deterministically by scene index (the convention `tests/scene_runtime.rs`
/// established for index-derived variation). Static, seeded by nothing but
/// the corpus order — the positional pipeline an animation would drive,
/// with no clock.
pub fn apply_lifecycle_transform(stage: &mut Stage, roots: &[Mob], scene_index: usize) {
    let dx = -0.2 + 0.05 * (scene_index % 3) as f64;
    let dy = 0.1 - 0.05 * (scene_index % 2) as f64;
    let scale = 0.92 + 0.02 * (scene_index % 3) as f64;
    for &root in roots {
        stage.shift(root, [dx, dy, 0.0]);
        stage.scale(root, scale);
    }
}

/// Serialize one scene's locked artifact: geometry snapshot + certified
/// frame at the post-construct point, then again post-transform (see the
/// module docs for the layout).
///
/// `render` produces one frame's canonical encoded document for a stage —
/// the test target's certified single-threaded renderer. Injecting it keeps
/// this library free of the dev-only `fmn-frame` edge while the locked bytes
/// remain the real engine's.
#[must_use]
pub fn artifact(
    case: &SceneCase,
    corpus: &Corpus,
    scene_index: usize,
    render: &dyn Fn(&Stage) -> Vec<u8>,
) -> Vec<u8> {
    let mut built = (case.build)(corpus);
    let mut writer = Writer::new(CORPUS_SCHEMA);
    writer.put_str(case.name);

    writer.put_str("post_construct");
    snapshot_geometry(&mut writer, &built.stage, &built.roots);
    writer.put_bytes(&render(&built.stage));

    apply_lifecycle_transform(&mut built.stage, &built.roots, scene_index);
    writer.put_str("post_transform");
    snapshot_geometry(&mut writer, &built.stage, &built.roots);
    writer.put_bytes(&render(&built.stage));

    writer.finish().expect("the scene artifact encodes")
}

// ---------------------------------------------------------------------------
// The corpus
// ---------------------------------------------------------------------------

/// A filled outline: the Reference's `color=` plus an explicit fill alpha.
fn filled(color: Srgb, opacity: f64) -> Style {
    Style::default().color(color).fill_opacity(opacity)
}

fn circle_tex_label(corpus: &Corpus) -> Built {
    let circle = Circle::new()
        .radius(0.9)
        .style(filled(BLUE_C, 0.35))
        .build();
    let label = Tex::new(r"\odot")
        .font_size(40.0)
        .build(&corpus.tex)
        .expect("label typesets")
        .vmob;
    stage_of(v_group([circle, label]))
}

fn arc_family(corpus: &Corpus) -> Built {
    let _ = corpus;
    let arc = Arc::new()
        .start_angle(-0.4)
        .angle(2.6)
        .radius(0.55)
        .style(filled(TEAL_B, 0.3))
        .build()
        .expect("the locked arc-family arc is valid")
        .shifted([-1.15, 0.45, 0.0]);
    let between = ArcBetweenPoints::new([-0.55, -0.3, 0.0], [0.55, 0.1, 0.0])
        .angle(1.1)
        .style(filled(GOLD_C, 0.3))
        .build()
        .expect("the locked between-points arc is valid")
        .shifted([0.0, 0.5, 0.0]);
    let annulus = Annulus::new()
        .inner_radius(0.2)
        .outer_radius(0.5)
        .style(filled(MAROON_C, 0.4))
        .build()
        .shifted([1.15, 0.45, 0.0]);
    let sector = AnnularSector::new()
        .inner_radius(0.15)
        .outer_radius(0.55)
        .angle(1.9)
        .start_angle(0.3)
        .style(filled(PURPLE_B, 0.4))
        .build()
        .expect("the locked annular sector is valid")
        .shifted([0.0, -0.75, 0.0]);
    stage_of(v_group([arc, between, annulus, sector]))
}

fn ellipse_and_annulus(corpus: &Corpus) -> Built {
    let _ = corpus;
    let ellipse = Ellipse::new()
        .width(1.6)
        .height(0.8)
        .style(filled(GREEN_B, 0.35))
        .build()
        .shifted([-0.9, 0.0, 0.0]);
    let ring = Annulus::new()
        .inner_radius(0.3)
        .outer_radius(0.62)
        .style(filled(YELLOW_C, 0.3))
        .build()
        .shifted([1.0, 0.0, 0.0]);
    let dot = Dot::new()
        .radius(0.16)
        .style(filled(RED_C, 0.9))
        .build()
        .shifted([1.0, 0.0, 0.0]);
    stage_of(v_group([ellipse, ring, dot]))
}

fn line_family(corpus: &Corpus) -> Built {
    let _ = corpus;
    let plain = Line::new([-1.4, 0.6, 0.0], [-0.2, 0.6, 0.0])
        .path_arc(0.15)
        .color(BLUE_C)
        .build()
        .expect("the locked line-family arc is valid");
    let dashed = DashedLine::new([-1.4, 0.2, 0.0], [-0.2, 0.2, 0.0])
        .dash_length(0.12)
        .positive_space_ratio(0.6)
        .color(TEAL_B)
        .build()
        .expect("the locked line-family dash configuration is valid");
    let arrow = Arrow::new([-1.4, -0.2, 0.0], [-0.2, -0.2, 0.0])
        .buff(0.0)
        .color(YELLOW_C)
        .build()
        .expect("the locked straight arrow is valid");
    let stroke_arrow = StrokeArrow::new([0.2, 0.6, 0.0], [1.4, 0.6, 0.0])
        .color(MAROON_C)
        .build()
        .expect("the locked straight stroke arrow is valid");
    let elbow = Elbow::new()
        .width(0.5)
        .color(GREEN_B)
        .build()
        .shifted([0.8, -0.35, 0.0]);
    stage_of(v_group([plain, dashed, arrow, stroke_arrow, elbow]))
}

fn poly_family(corpus: &Corpus) -> Built {
    let _ = corpus;
    let rect = Rectangle::new()
        .width(1.0)
        .height(0.6)
        .style(filled(BLUE_C, 0.35))
        .build()
        .expect("the locked rectangle is unrounded")
        .shifted([-1.2, 0.0, 0.0]);
    let triangle = RegularPolygon::triangle()
        .radius(0.5)
        .style(filled(RED_C, 0.35))
        .build()
        .expect("three directions are within the public cap")
        .shifted([0.0, 0.0, 0.0]);
    let hexagon = RegularPolygon::new(6)
        .radius(0.5)
        .style(filled(TEAL_B, 0.35))
        .build()
        .expect("six directions are within the public cap")
        .shifted([1.2, 0.0, 0.0]);
    stage_of(v_group([rect, triangle, hexagon]))
}

fn polygon_and_cubic(corpus: &Corpus) -> Built {
    let _ = corpus;
    let polygon = Polygon::new([
        [-1.6, -0.5, 0.0],
        [-0.9, -0.2, 0.0],
        [-0.7, 0.6, 0.0],
        [-1.4, 0.7, 0.0],
        [-1.8, 0.1, 0.0],
    ])
    .style(filled(GOLD_C, 0.35))
    .build();
    let cubic = CubicBezier::new(
        [0.2, -0.6, 0.0],
        [0.5, 0.8, 0.0],
        [1.3, -0.8, 0.0],
        [1.7, 0.6, 0.0],
    )
    .color(PURPLE_B)
    .build()
    .expect("primitive-corpus cubic is finite and within the converter budget");
    stage_of(v_group([polygon, cubic]))
}

fn rounded_rectangle_arrow(corpus: &Corpus) -> Built {
    let _ = corpus;
    let rounded = Rectangle::new()
        .width(1.2)
        .height(0.7)
        .corner_radius(0.18)
        .style(filled(MAROON_C, 0.35))
        .build()
        .expect("the locked rounded rectangle is valid")
        .shifted([-0.9, 0.0, 0.0]);
    let arrow = StrokeArrow::new([0.1, 0.0, 0.0], [1.5, 0.0, 0.0])
        .color(WHITE)
        .build()
        .expect("the locked straight stroke arrow is valid");
    stage_of(v_group([rounded, arrow]))
}

fn layered_alpha_stack(corpus: &Corpus) -> Built {
    let _ = corpus;
    let mut circle = Circle::new()
        .radius(0.7)
        .style(filled(BLUE_C, 0.5))
        .build()
        .shifted([-0.4, 0.0, 0.0]);
    circle = circle.with_z_index(0);
    let mut rect = Rectangle::new()
        .width(1.0)
        .height(1.0)
        .style(filled(YELLOW_C, 0.5))
        .build()
        .expect("the locked alpha-stack rectangle is unrounded")
        .shifted([0.1, 0.1, 0.0]);
    rect = rect.with_z_index(1);
    let mut triangle = RegularPolygon::triangle()
        .radius(0.55)
        .style(filled(RED_C, 0.5))
        .build()
        .expect("three directions are within the public cap")
        .shifted([0.55, -0.15, 0.0]);
    triangle = triangle.with_z_index(2);
    stage_of(v_group([circle, rect, triangle]))
}

fn brace_label(corpus: &Corpus) -> Built {
    let target = Rectangle::new()
        .width(1.6)
        .height(0.7)
        .style(filled(TEAL_B, 0.25))
        .build()
        .expect("the locked brace target is unrounded");
    let label = Text::new("span")
        .font_size(28.0)
        .build(&corpus.book)
        .expect("brace label lays out")
        .vmob;
    let brace = BraceLabel::new(&target, label, DOWN).build();
    stage_of(v_group([target, brace]))
}

fn matchers_marks(corpus: &Corpus) -> Built {
    let phrase = Text::new("match me")
        .font_size(32.0)
        .build(&corpus.book)
        .expect("phrase lays out")
        .vmob
        .shifted([-0.9, 0.35, 0.0]);
    let frame = SurroundingRectangle::new(&phrase).color(YELLOW_C).build();
    let under = underline(&phrase, RED_C, 0.12, 1.0);
    let good = checkmark(GREEN_B)
        .scaled_about(0.5, [0.0, 0.0, 0.0])
        .shifted([1.0, 0.4, 0.0]);
    let bad_target = Rectangle::new()
        .width(0.9)
        .height(0.5)
        .style(filled(BLUE_C, 0.25))
        .build()
        .expect("the locked matcher target is unrounded")
        .shifted([1.0, -0.45, 0.0]);
    let bad = cross(&bad_target, RED_C, 6.0);
    let bang = exmark(GOLD_C)
        .scaled_about(0.5, [0.0, 0.0, 0.0])
        .shifted([-1.9, -0.5, 0.0]);
    stage_of(v_group([phrase, frame, under, good, bad_target, bad, bang]))
}

fn text_basic(corpus: &Corpus) -> Built {
    let text = Text::new("FrankenManim")
        .font_size(36.0)
        .build(&corpus.book)
        .expect("text lays out")
        .vmob;
    stage_of(v_group([text]))
}

fn text_styled_spans(corpus: &Corpus) -> Built {
    let t2c: [(&str, Srgb); 2] = [("alpha", RED_C), ("beta", TEAL_B)];
    let text = Text::new("alpha beta gamma")
        .font_size(36.0)
        .t2c(&t2c)
        .build(&corpus.book)
        .expect("styled text lays out")
        .vmob;
    stage_of(v_group([text]))
}

fn tex_fraction(corpus: &Corpus) -> Built {
    // The corpus's text-style fraction (inline math); tex_display_sum is the
    // display-style case. `Tex::new` itself defaults to display, like the
    // Reference's align* (fm-tex-display-style-nclg).
    let frac = Tex::new(r"\frac{a+b}{c-d}")
        .math_style(fmn_tex::Style::Text)
        .font_size(40.0)
        .build(&corpus.tex)
        .expect("fraction typesets")
        .vmob;
    stage_of(v_group([frac]))
}

fn tex_display_sum(corpus: &Corpus) -> Built {
    let sum = Tex::new(r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}")
        .display()
        .font_size(40.0)
        .build(&corpus.tex)
        .expect("sum typesets")
        .vmob;
    stage_of(v_group([sum]))
}

fn tex_overbrace(corpus: &Corpus) -> Built {
    let over = Tex::new(r"\overbrace{x+x+\cdots+x}^{n\text{ terms}}")
        .font_size(40.0)
        .build(&corpus.tex)
        .expect("overbrace typesets")
        .vmob;
    stage_of(v_group([over]))
}

fn decimal_number_tick(corpus: &Corpus) -> Built {
    let line = NumberLine::new([-3.0, 3.0, 1.0])
        .width(4.0)
        .build()
        .expect("number-line ticks stay within the default budget")
        .into_vmob()
        .shifted([0.0, -0.6, 0.0]);
    let number = DecimalNumber::new(std::f64::consts::PI)
        .num_decimal_places(2)
        .font_size(36.0)
        .build(&corpus.book)
        .expect("decimal number lays out")
        .into_vmob()
        .shifted([0.0, 0.45, 0.0]);
    // The line is 4 units wide over [-3, 3], so π lands at π·(4/6).
    let marker = Dot::new()
        .radius(0.08)
        .style(filled(RED_C, 1.0))
        .build()
        .shifted([std::f64::consts::PI * (4.0 / 6.0), -0.6, 0.0]);
    stage_of(v_group([line, number, marker]))
}

fn integer_counter(corpus: &Corpus) -> Built {
    let integer = Integer::new(-42.0)
        .include_sign(true)
        .font_size(40.0)
        .build(&corpus.book)
        .expect("integer lays out")
        .into_vmob();
    stage_of(v_group([integer]))
}

fn matrix_2x2(corpus: &Corpus) -> Built {
    let matrix = IntegerMatrix::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
        .font_size(36.0)
        .build(&corpus.tex, &corpus.book)
        .expect("integer matrix builds")
        .vmob;
    stage_of(v_group([matrix]))
}

fn decimal_matrix(corpus: &Corpus) -> Built {
    let matrix = DecimalMatrix::new(vec![vec![0.5, -1.25], vec![3.0, 2.75]])
        .num_decimal_places(2)
        .font_size(36.0)
        .build(&corpus.tex, &corpus.book)
        .expect("decimal matrix builds")
        .vmob;
    stage_of(v_group([matrix]))
}

fn bulleted_list(corpus: &Corpus) -> Built {
    let list = BulletedList::new(&["Arc and line", "Polygon", "TeX labels"])
        .font_size(28.0)
        .build(&corpus.book)
        .expect("bulleted list lays out")
        .vmob;
    stage_of(v_group([list]))
}

/// The title goes to the top edge of the full frame (y ~ 3.2 ... 3.75), far
/// outside this corpus's +-1.5-unit viewport, so v1 locked a blank frame and
/// could not show fm-5wq.54's 1.31-unit offset. v2 moves the built title 3.5
/// units down into view; a pure translation keeps its centring visible.
fn title_underlined(corpus: &Corpus) -> Built {
    let title = Title::new(&["The Gauntlet"])
        .font_size(40.0)
        .build(&corpus.book)
        .expect("title lays out")
        .vmob
        .shifted([0.0, -3.5, 0.0]);
    stage_of(v_group([title]))
}

fn controls_panel(corpus: &Corpus) -> Built {
    let mut stage = Stage::new();
    let button = Button::new("Play")
        .font_size(28.0)
        .build(&corpus.book)
        .expect("button lays out")
        .shifted([-1.3, 0.0, 0.0]);
    let button = stage.add(button);
    stage.add_to_scene(button).expect("button joins the scene");

    let checkbox = stage.add(Checkbox::new(true));
    stage.shift(checkbox, [0.1, 0.0, 0.0]);
    stage
        .add_to_scene(checkbox)
        .expect("checkbox joins the scene");

    let mut toggle = EnableDisableButton::new(false);
    toggle.set_value(true);
    let toggle = stage.add(toggle);
    stage.shift(toggle, [1.2, 0.0, 0.0]);
    stage.add_to_scene(toggle).expect("toggle joins the scene");

    Built {
        stage,
        roots: vec![button, checkbox, toggle],
    }
}

fn number_line(corpus: &Corpus) -> Built {
    let line = NumberLine::new([-3.0, 3.0, 1.0])
        .width(4.2)
        .include_numbers(true)
        .build_numbered(&corpus.book)
        .expect("numbered line builds")
        .into_vmob();
    stage_of(v_group([line]))
}

fn axes_basic(corpus: &Corpus) -> Built {
    let axes = Axes::new()
        .x_range([-3.0, 3.0, 1.0])
        .y_range([-2.0, 2.0, 1.0])
        .unit_size(0.7)
        .build(&corpus.book)
        .expect("axes build")
        .into_vmob();
    stage_of(v_group([axes]))
}

fn axes_function_graph(corpus: &Corpus) -> Built {
    let axes = Axes::new()
        .x_range([-3.0, 3.0, 1.0])
        .y_range([-2.0, 2.0, 1.0])
        .unit_size(0.7)
        .build(&corpus.book)
        .expect("axes build")
        .into_vmob();
    let graph = FunctionGraph::new(fmn_dmath::sin)
        .x_range([-3.0, 3.0, 0.05])
        .color(YELLOW_C)
        .build()
        .expect("function-graph sampling stays within the default budget")
        .scaled_about(0.7, [0.0, 0.0, 0.0]);
    stage_of(v_group([axes, graph]))
}

fn implicit_circle(corpus: &Corpus) -> Built {
    let _ = corpus;
    let circle = ImplicitFunction::new(|x: f64, y: f64| x * x + y * y - 1.0)
        .x_range([-1.5, 1.5])
        .y_range([-1.5, 1.5])
        .color(TEAL_B)
        .build()
        .expect("the circle extracts within its budget");
    let center = Dot::new().radius(0.06).style(filled(WHITE, 1.0)).build();
    stage_of(v_group([circle, center]))
}

fn parametric_lissajous(corpus: &Corpus) -> Built {
    let _ = corpus;
    let curve = ParametricCurve::new(|t: f64| {
        [
            1.6 * fmn_dmath::sin(3.0 * t),
            1.1 * fmn_dmath::sin(2.0 * t),
            0.0,
        ]
    })
    .t_range([0.0, TAU, TAU / 400.0])
    .color(MAROON_C)
    .build()
    .expect("parametric sampling stays within the default budget");
    stage_of(v_group([curve]))
}

/// The corpus, in lock order: eight primitive-class scenes, then eighteen
/// feature scenes covering brace, matchers, text, tex, numbers, matrix,
/// special tex, controls, coords, and graphs.
pub const SCENES: &[SceneCase] = &[
    SceneCase {
        name: "circle_tex_label.v1",
        build: circle_tex_label,
    },
    SceneCase {
        name: "arc_family.v1",
        build: arc_family,
    },
    SceneCase {
        name: "ellipse_and_annulus.v1",
        build: ellipse_and_annulus,
    },
    SceneCase {
        name: "line_family.v1",
        build: line_family,
    },
    SceneCase {
        name: "poly_family.v1",
        build: poly_family,
    },
    SceneCase {
        name: "polygon_and_cubic.v1",
        build: polygon_and_cubic,
    },
    SceneCase {
        name: "rounded_rectangle_arrow.v1",
        build: rounded_rectangle_arrow,
    },
    SceneCase {
        name: "layered_alpha_stack.v1",
        build: layered_alpha_stack,
    },
    SceneCase {
        name: "brace_label.v1",
        build: brace_label,
    },
    SceneCase {
        name: "matchers_marks.v1",
        build: matchers_marks,
    },
    SceneCase {
        name: "text_basic.v1",
        build: text_basic,
    },
    SceneCase {
        name: "text_styled_spans.v1",
        build: text_styled_spans,
    },
    SceneCase {
        name: "tex_fraction.v1",
        build: tex_fraction,
    },
    SceneCase {
        name: "tex_display_sum.v1",
        build: tex_display_sum,
    },
    SceneCase {
        name: "tex_overbrace.v1",
        build: tex_overbrace,
    },
    SceneCase {
        name: "decimal_number_tick.v1",
        build: decimal_number_tick,
    },
    SceneCase {
        name: "integer_counter.v1",
        build: integer_counter,
    },
    SceneCase {
        name: "matrix_2x2.v1",
        build: matrix_2x2,
    },
    SceneCase {
        name: "decimal_matrix.v1",
        build: decimal_matrix,
    },
    SceneCase {
        name: "bulleted_list.v1",
        build: bulleted_list,
    },
    SceneCase {
        name: "title_underlined.v2",
        build: title_underlined,
    },
    SceneCase {
        name: "controls_panel.v1",
        build: controls_panel,
    },
    SceneCase {
        name: "number_line.v1",
        build: number_line,
    },
    SceneCase {
        name: "axes_basic.v1",
        build: axes_basic,
    },
    SceneCase {
        name: "axes_function_graph.v1",
        build: axes_function_graph,
    },
    SceneCase {
        name: "implicit_circle.v1",
        build: implicit_circle,
    },
    SceneCase {
        name: "parametric_lissajous.v1",
        build: parametric_lissajous,
    },
];

/// The engine-equivalence subset ([`crate::equivalence`]): the scenes whose
/// pixels stress the engine classes the fast route's arithmetic could
/// plausibly diverge on — fills, strokes, joins, dashes, tips, alpha
/// compositing, occlusion, fine glyph geometry, long curves, ticks, and
/// isolines. Names must appear in [`SCENES`]; the test asserts it.
pub const EQUIVALENCE_SUBSET: &[&str] = &[
    "circle_tex_label.v1",
    "arc_family.v1",
    "line_family.v1",
    "poly_family.v1",
    "layered_alpha_stack.v1",
    "tex_display_sum.v1",
    "decimal_matrix.v1",
    "bulleted_list.v1",
    "axes_function_graph.v1",
    "implicit_circle.v1",
];

/// Look a scene up by lock name.
#[must_use]
pub fn scene_named(name: &str) -> Option<&'static SceneCase> {
    SCENES.iter().find(|case| case.name == name)
}
