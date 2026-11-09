//! Correctness-oracle assertions (§16.3 plane 1, fm-t1v).
//!
//! The measurements live in `fmn_conformance::oracles` (whose module docs
//! file every oracle under §16.3's taxonomy); this target owns the
//! pass/fail bars, plus the two oracle families that need this crate's
//! dev-only `fmn-frame` edge: the color-model round-trips and the
//! frame-level integer-pixel translation equivariance law.

use fmn_conformance::oracles::{
    FixtureCorpus, REFERENCE_QUAD, TexAppendixG, WindingProbe, boolean_de_morgan_area_error,
    boolean_partition_area_error, boolean_subdivision_area_error,
    boolean_union_commutes_area_error, circle_arc_length_rel_error, path_arc_length_vs_quadrature,
    probe_winding, quadratic_arc_length_vs_quadrature, reference_quad_point,
    stateless_resampling_max_error, tex_appendix_g_errors, winding_probes_max_error,
    winding_translation_max_error,
};
use fmn_conformance::tolerance::{NanPolicy, check_points_abs};
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, TAU, TEAL_B, WHITE};
use fmn_core::types::Vec3;
use fmn_frame::transfer::{quantize8, srgb_decode, srgb_encode};
use fmn_geom::QuadPath;
use fmn_geom::arclength::ArcLengthTable;
use fmn_geom::bezier::{integer_interpolate, partial_quadratic, quadratic_points_for_arc};
use fmn_geom::boolean::{BooleanOperation, BooleanOptions};
use fmn_library::style::Style;
use fmn_library::{Circle, Rectangle};
use fmn_mobject::{Mob, Stage};
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{EngineIdentity, FrameConfig, FrameJob};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use std::path::PathBuf;

/// §16.4's loose cross-engine bar for structural fixtures (both sides
/// f64, different op order) — the same bar `tests/npy_interchange.rs`
/// pins for the interchange corpus.
const FIXTURE_TOL: f64 = 1e-6;

fn fixtures() -> FixtureCorpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/npy");
    FixtureCorpus::load(&root).expect("fixture manifest loads")
}

fn fixture_manifest_scratch() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("oracle_manifest_strictness");
    std::fs::create_dir_all(&root).expect("fixture manifest scratch directory");
    root
}

// =====================================================================
// 1. Analytic ground truths — arc length vs closed forms
// =====================================================================

#[test]
fn circle_arc_length_matches_the_analytic_circumference() {
    for radius in [0.5, 1.0, 2.75] {
        let error = circle_arc_length_rel_error(radius);
        // The sixteen-quadratic circle model's own approximation
        // shortfall, measured at 1.006e-4 relative; the bar pins the
        // model density, not rounding (rounding here is ~1e-15).
        assert!(
            error < 1.5e-4,
            "radius {radius}: circle model arc length drifted to {error:e} relative"
        );
    }
}

#[test]
fn quadratic_closed_form_matches_an_independent_integral() {
    // Curves exercising the general antiderivative branch: bent,
    // asymmetric, and one nearly straight (which skirts the linear-speed
    // branch without entering it).
    let curves: [([f64; 3], [f64; 3], [f64; 3]); 3] = [
        ([-1.0, 0.5, 0.25], [0.75, 2.0, -0.5], [2.0, -1.0, 1.0]),
        ([0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [2.0, 0.0, 0.0]),
        ([0.0, 0.0, 0.0], [1.0, 1.0 + 1e-7, 0.0], [2.0, 2.0, 0.0]),
    ];
    for (a0, h, a1) in curves {
        let error = quadratic_arc_length_vs_quadrature(a0, h, a1, 4096);
        // Measured 3.6e-15 (the quadrature's own truncation at 4096
        // panels dominates); the bound is 4 orders of margin.
        assert!(
            error < 1e-10,
            "{a0:?} {h:?} {a1:?}: closed form vs quadrature {error:e}"
        );
    }
}

#[test]
fn ellipse_model_arc_length_matches_an_independent_integral() {
    // The ellipse row: scale the sixteen-quadratic unit circle model to a
    // 2×1 ellipse. There is no elementary circumference to compare
    // against; the analytic reference for the model's length is the
    // independent quadrature.
    let circle = QuadPath::try_arc(0.0, TAU, 1.0, [0.0; 3], None)
        .expect("the oracle's fixed circle is valid");
    let points: Vec<Vec3> = circle
        .points()
        .iter()
        .map(|p| [p[0], 0.5 * p[1], p[2]])
        .collect();
    let ellipse = QuadPath::from_points(points).expect("a valid path");
    let error = path_arc_length_vs_quadrature(&ellipse, 1024);
    // Measured 1.1e-15 relative.
    assert!(
        error < 1e-10,
        "ellipse model: table total vs quadrature {error:e} relative"
    );
}

// =====================================================================
// 1. Analytic ground truths — path-boolean identities
// =====================================================================

/// A corner rectangle as a closed QuadPath.
fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> QuadPath {
    let corners: Vec<Vec3> = vec![[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]];
    let mut path = QuadPath::new();
    path.set_points_as_corners(&corners)
        .expect("corners are valid points");
    path
}

/// Overlapping rectangles with fractional edges (nothing axis-degenerate)
/// inside a strict universe.
fn boolean_world() -> (QuadPath, QuadPath, QuadPath) {
    let a = rect(0.0, 0.0, 3.0, 2.0);
    let b = rect(1.25, 0.5, 4.0, 2.75);
    let universe = rect(-1.0, -1.0, 5.0, 4.0);
    (a, b, universe)
}

#[test]
fn boolean_union_area_commutes() {
    let (a, b, _) = boolean_world();
    let error = boolean_union_commutes_area_error(&a, &b, BooleanOptions::default())
        .expect("rectangles boolean cleanly");
    // Measured exactly 0.0: the arrangement is exact on polygonal inputs.
    assert!(error < 1e-12, "A∪B vs B∪A: {error:e}");
}

#[test]
fn boolean_intersection_and_difference_partition_the_subject() {
    let (a, b, _) = boolean_world();
    let error = boolean_partition_area_error(&a, &b, BooleanOptions::default())
        .expect("rectangles boolean cleanly");
    // Measured exactly 0.0.
    assert!(error < 1e-12, "(A∩B)+(A−B) vs A: {error:e}");
}

#[test]
fn boolean_de_morgan_holds_through_a_universe() {
    let (a, b, universe) = boolean_world();
    let error = boolean_de_morgan_area_error(&a, &b, &universe, BooleanOptions::default())
        .expect("rectangles boolean cleanly");
    // Measured exactly 0.0.
    assert!(error < 1e-12, "A∪B vs U−((U−A)∩(U−B)): {error:e}");
}

// =====================================================================
// 1. Analytic ground truths — winding invariants
// =====================================================================

/// A CCW square centered at `center` with the given `radius`.
fn square_ring(center: [f64; 2], radius: f64) -> Vec<Vec3> {
    let [cx, cy] = center;
    vec![
        [cx + radius, cy + radius, 0.0],
        [cx - radius, cy + radius, 0.0],
        [cx - radius, cy - radius, 0.0],
        [cx + radius, cy - radius, 0.0],
    ]
}

fn reversed(ring: &[Vec3]) -> Vec<Vec3> {
    ring.iter().rev().copied().collect()
}

#[test]
fn integer_translations_preserve_winding_numbers() {
    let convex = square_ring([0.25, -0.5], 1.5);
    let concave: Vec<Vec3> = vec![
        [2.0, 0.0, 0.0],
        [0.5, 0.5, 0.0],
        [0.0, 2.0, 0.0],
        [-0.5, 0.5, 0.0],
        [-2.0, 0.0, 0.0],
        [-0.5, -0.5, 0.0],
        [0.0, -2.0, 0.0],
        [0.5, -0.5, 0.0],
    ];
    let polygons: [&[Vec3]; 2] = [&convex, &concave];
    let queries: [Vec3; 3] = [[0.25, -0.5, 0.0], [10.0, 10.0, 0.0], [-3.0, 1.0, 0.0]];
    // Integer translations, including one far enough to exercise
    // large-coordinate angle arithmetic.
    let translations: [Vec3; 3] = [[3.0, -2.0, 0.0], [17.0, 41.0, 0.0], [1000.0, -777.0, 0.0]];
    let error = winding_translation_max_error(&polygons, &queries, &translations);
    // Measured exactly 0.0, including the 1000-unit translation.
    assert!(
        error < 1e-12,
        "winding under integer translation drifted {error:e}"
    );
}

#[test]
fn nested_containment_flips_parity_per_rule() {
    let outer = square_ring([0.0, 0.0], 2.0);
    let inner_ccw = square_ring([0.0, 0.0], 1.0);
    let inner_cw = reversed(&inner_ccw);
    let outside = [10.0, 0.0, 0.0];
    let between = [1.5, 0.0, 0.0];
    let inside = [0.0, 0.0, 0.0];

    // Opposite-wound hole: the winding numbers are 0 / +1 / 0 — a hole
    // reads empty under BOTH rules.
    let holed: [&[Vec3]; 2] = [&outer, &inner_cw];
    let probes = [
        probe_winding(&holed, outside),
        probe_winding(&holed, between),
        probe_winding(&holed, inside),
    ];
    let expected = [0, 1, 0];
    let probes: Vec<WindingProbe> = probes
        .into_iter()
        .zip(expected)
        .map(|(mut p, e)| {
            p.expected = e;
            p
        })
        .collect();
    let error = winding_probes_max_error(&probes);
    // Measured 1.4e-16.
    assert!(error < 1e-12, "opposite-wound nesting: {error:e}");
    for (probe, _) in probes.iter().zip(expected) {
        let nonzero_filled = probe.expected != 0;
        let even_odd_filled = probe.expected % 2 != 0;
        assert_eq!(
            nonzero_filled, even_odd_filled,
            "a hole must read empty under both rules at {:?}",
            probe.query
        );
    }

    // Same-wound nesting: 0 / +1 / +2 — the doubly-wound interior is
    // filled under NonZero and EMPTY under EvenOdd: the parity flip.
    let doubled: [&[Vec3]; 2] = [&outer, &inner_ccw];
    let probes = [
        probe_winding(&doubled, outside),
        probe_winding(&doubled, between),
        probe_winding(&doubled, inside),
    ];
    let expected = [0, 1, 2];
    let probes: Vec<WindingProbe> = probes
        .into_iter()
        .zip(expected)
        .map(|(mut p, e)| {
            p.expected = e;
            p
        })
        .collect();
    let error = winding_probes_max_error(&probes);
    assert!(error < 1e-12, "same-wound nesting: {error:e}");
    let inner_probe = &probes[2];
    assert!(
        inner_probe.expected != 0,
        "NonZero fills the doubly-wound interior"
    );
    assert_eq!(
        inner_probe.expected % 2,
        0,
        "EvenOdd empties the doubly-wound interior: the parity flip"
    );
}

// =====================================================================
// 1. Analytic ground truths — color-model round-trips
// =====================================================================

#[test]
fn srgb_transfer_round_trips_within_deterministic_pow_error() {
    // The continuous law srgb_decode ∘ srgb_encode ≈ id over [0, 1],
    // sampled densely and AT the piecewise knee (0.0031308 / 0.04045),
    // where an implementation seam would show. fmn-frame's own unit test
    // covers 101 points at 1e-12; this oracle's denser grid finds the
    // deterministic pow's true worst case: measured 2.21e-9.
    let mut worst = 0.0f64;
    for i in 0..=4096 {
        let x = f64::from(i) / 4096.0;
        worst = worst.max((srgb_decode(srgb_encode(x)) - x).abs()); // ubs:ignore — sRGB transfer, not JWT
    }
    for knee in [0.003_130_8, 0.040_45] {
        for offset in [-1e-12, 0.0, 1e-12] {
            let x = knee + offset;
            worst = worst.max((srgb_decode(srgb_encode(x)) - x).abs()); // ubs:ignore — sRGB transfer, not JWT
        }
    }
    assert!(worst < 1e-8, "srgb round-trip worst error {worst:e}");
}

#[test]
fn srgb_byte_pipeline_round_trips_every_byte_exactly() {
    // The pipeline law: every 8-bit code survives
    // byte → decode → encode → quantize8 unchanged. This is what makes
    // the encode-once doctrine (§14.1, D-23) safe at the byte boundary.
    for byte in 0..=255u8 {
        let round_tripped = quantize8(srgb_encode(srgb_decode(f64::from(byte) / 255.0))); // ubs:ignore — sRGB transfer, not JWT
        assert_eq!(round_tripped, byte, "byte {byte} did not survive");
    }
}

// =====================================================================
// 1. Analytic ground truths — TeX Appendix-G placement parameters
// =====================================================================

#[test]
fn tex_placement_uses_the_published_appendix_g_parameters() {
    let m: TexAppendixG = tex_appendix_g_errors().expect("the bundled engine typesets the probes");
    // Exact-parameter rows: short glyphs, clearances non-binding, so the
    // published σ is the exact expected coordinate; 1e-9 ems is layout
    // rounding, not a parameter change.
    const EXACT: f64 = 1e-9;
    assert!(
        m.display_superscript_shift < EXACT,
        "σ13 display sup shift: {:e}",
        m.display_superscript_shift
    );
    assert!(
        m.text_superscript_shift < EXACT,
        "σ14 text sup shift: {:e}",
        m.text_superscript_shift
    );
    assert!(
        m.cramped_superscript_shift < EXACT,
        "σ15 cramped sup shift: {:e}",
        m.cramped_superscript_shift
    );
    assert!(
        m.lone_subscript_shift < EXACT,
        "σ16 lone sub shift: {:e}",
        m.lone_subscript_shift
    );
    assert!(
        m.radical_rule_thickness < EXACT,
        "ξ8 radical rule: {:e}",
        m.radical_rule_thickness
    );
    // Clearance-margin rows: the rules require these to be positive.
    assert!(
        m.script_separation_over_4theta > 0.0,
        "rule 18: sup/sub separation must exceed 4θ: {:e}",
        m.script_separation_over_4theta
    );
    assert!(
        m.radical_clearance_over_psi >= -EXACT,
        "rule 11: overbar clearance must be at least ψ: {:e}",
        m.radical_clearance_over_psi
    );
}

// =====================================================================
// 2. Metamorphic laws — stateless resampling, subdivision invariance
// =====================================================================

/// Arc-length fractions kept interior: the law is not claimed at the
/// degenerate endpoints.
const PROBE_ALPHAS: [f64; 9] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

#[test]
fn resampled_path_stays_on_the_original_curve() {
    // A path with real curvature: an arc past the trivial quadrant, plus
    // a smooth tail so two curve shapes participate.
    let mut path = QuadPath::try_arc(0.3, 4.2, 1.5, [0.1, -0.2, 0.0], None)
        .expect("the resampling fixture arc is valid");
    let end = path.last_point().expect("arc has points");
    let _ = path
        .add_smooth_curve_to([end[0] + 1.0, end[1] - 0.5, 0.0])
        .expect("smooth tail is valid");
    let original = ArcLengthTable::for_path(&path).total();
    let error = stateless_resampling_max_error(&path, 3, &PROBE_ALPHAS);
    // Measured 3.1e-16 on a 7.42-unit path — subdivision rounding only.
    assert!(
        error < 1e-12 * original.max(1.0),
        "resampled points off the original curve by {error:e} (path length {original})"
    );
}

#[test]
fn boolean_results_are_subdivision_invariant_in_area() {
    // Curved subject (the circle model), polygonal clip: resampling the
    // subject must not move the boolean's area beyond flatten-induced
    // drift. Measured exactly 0.0 for all three operations; the bound
    // sits far under the flatten tolerance's own area scale
    // (perimeter × tolerance ≈ 6e-2).
    let subject = QuadPath::try_arc(0.0, TAU, 1.0, [0.0; 3], None)
        .expect("the subdivision fixture circle is valid");
    let clip = rect(0.1, -0.6, 1.7, 0.9);
    let options = BooleanOptions::default();
    let mut worst = 0.0f64;
    for operation in [
        BooleanOperation::Union,
        BooleanOperation::Intersection,
        BooleanOperation::Difference,
    ] {
        let error = boolean_subdivision_area_error(&subject, &clip, operation, 3, options)
            .expect("curved boolean succeeds");
        worst = worst.max(error);
    }
    // Measured exactly 0.0 for all three operations on this scene; the
    // bound keeps the assertion honest about flatten arithmetic without
    // re-opening the drift budget the law's docs disclaim.
    assert!(
        worst < 1e-12,
        "subdivision moved the boolean area by {worst:e}"
    );
}

// =====================================================================
// 2. Metamorphic laws — integer-pixel translation equivariance
// =====================================================================

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const SCALE: f64 = 60.0;
const TILING: Tiling = Tiling {
    macro_tile: 128,
    fine_tile: 16,
};

fn frame_config() -> FrameConfig {
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

/// A small filled-and-stroked scene near the frame centre.
fn translation_scene() -> (Stage, Vec<Mob>) {
    let mut stage = Stage::new();
    let circle = stage.add(
        Circle::new()
            .radius(0.8)
            .style(Style::default().fill(TEAL_B, 0.8).stroke(WHITE, 4.0, 1.0)),
    );
    stage.add_to_scene(circle).expect("live");
    let rectangle = stage.add(
        Rectangle::new()
            .width(1.4)
            .height(0.9)
            .style(Style::default().fill(BLUE_C, 0.9).stroke(WHITE, 2.0, 1.0))
            .build()
            .expect("the translation fixture rectangle is unrounded"),
    );
    stage.add_to_scene(rectangle).expect("live");
    // Overlap the two so compositing participates, off-center so the
    // translated copy still clears the frame edge.
    stage.shift(rectangle, [-0.9, 0.35, 0.0]);
    (stage, vec![circle, rectangle])
}

fn render(stage: &Stage) -> fmn_frame::FrameBuffer {
    let cfg = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0).expect("valid rendering-oracle fixture");
    let mono = MonoTable::build(&plan, cfg.map).expect("bounded rendering-oracle monotone table");
    let binning =
        Binning::build(&plan, cfg.viewport, TILING, cfg.map).expect("bounded conformance binning");
    FrameJob::with_identity(&plan, &mono, &binning, cfg, EngineIdentity::certified())
        .expect("matching frame artifacts")
        .render(1)
        .expect("the certified engine renders the frame")
}

fn read_pixel(frame: &fmn_frame::FrameBuffer, x: u32, y: u32) -> [f64; 4] {
    let base = y as usize * frame.layout().stride(0) + x as usize * 8;
    let pixel = &frame.plane(0)[base..base + 8];
    let mut decoded = [0.0; 4];
    for (channel, value) in decoded.iter_mut().enumerate() {
        *value = fmn_frame::half::f16_to_f64(u16::from_le_bytes([
            pixel[channel * 2],
            pixel[channel * 2 + 1],
        ]));
    }
    decoded
}

#[test]
fn integer_pixel_translation_translates_the_frame() {
    let (stage, mobs) = translation_scene();
    let base = render(&stage);
    let mut shifted_stage = stage;
    // +1.0, −0.5 object units at 60 px/unit is EXACTLY 60 and 30 screen
    // pixels — an integer-pixel translation with no rounding anywhere. The
    // map decides the direction (+Y is up the frame, fm-sq8.9); the law is
    // that the frame translates by exactly that many pixels.
    let shift = [1.0, -0.5];
    for mob in &mobs {
        shifted_stage.shift(*mob, [shift[0], shift[1], 0.0]);
    }
    let shifted = render(&shifted_stage);

    let moved = frame_config().map.delta_to_pixel(shift[0], shift[1]);
    assert_eq!(moved, [60.0, 30.0], "an integer-pixel translation");
    let (dx, dy) = (-(moved[0] as i64), -(moved[1] as i64));
    let mut differing = 0u64;
    let mut worst = 0.0f64;
    // Stay 8 px inside the frame: binning clips at the viewport, and the
    // law is about the rasterization, not the clip boundary.
    for y in 8..HEIGHT - 8 {
        for x in 8..WIDTH - 8 {
            let sx = i64::from(x) + dx;
            let sy = i64::from(y) + dy;
            if sx < 0 || sy < 0 || sx >= i64::from(WIDTH) || sy >= i64::from(HEIGHT) {
                continue;
            }
            let a = read_pixel(&base, sx as u32, sy as u32);
            let b = read_pixel(&shifted, x, y);
            for channel in 0..4 {
                let error = (a[channel] - b[channel]).abs();
                if error > 0.0 {
                    differing += 1;
                }
                worst = worst.max(error);
            }
        }
    }
    // The law, where it holds: the certified engine evaluates coverage
    // analytically in f64 with the instance translation folded into the
    // polynomial constant term; an integer-pixel translation changes only
    // that term by an exact integer, so every pixel evaluates the same
    // arithmetic at the same relative position and the frame translates
    // identically — bit for bit.
    assert_eq!(
        differing, 0,
        "{differing} channel(s) differ under an integer-pixel translation (worst {worst:e})"
    );
}

// =====================================================================
// 3. Structural fixtures against the Reference
// =====================================================================

#[test]
fn fixture_manifest_is_canonical_and_shape_bound() {
    const PARTIAL_HASH: &str = "acdf740923ae14f4f8a6ac7b161be06e7b6d023a8376ea205d38205f05cb7a1f";
    const QUAD_EVAL_HASH: &str = "033d1c456aa1f3488f9af1eb014e261928ce758f69f938f1c7e59096edbde4aa";
    const FIRST_FORMULA: &str = "quadratic_bezier_points_for_arc(TAU/4, n_components=4)";

    let root = fixture_manifest_scratch();
    let manifest_path = root.join("MANIFEST.tsv");
    let committed = include_str!("../fixtures/npy/MANIFEST.tsv");
    let mut reordered_lines: Vec<&str> = committed.lines().collect();
    reordered_lines.swap(4, 5);
    let reordered = format!("{}\n", reordered_lines.join("\n"));
    let mut missing_lines: Vec<&str> = committed.lines().collect();
    missing_lines.pop();
    let missing_row = format!("{}\n", missing_lines.join("\n"));
    let first_row = committed
        .lines()
        .nth(4)
        .expect("committed manifest has its first generator row");
    let extra_row = format!("{committed}{first_row}\n");
    let malformed = vec![
        (
            "missing final line feed",
            committed
                .strip_suffix('\n')
                .expect("committed manifest has its canonical final line feed")
                .to_string(),
        ),
        (
            "carriage-return separators",
            committed.replace('\n', "\r\n"),
        ),
        (
            "reference drift",
            committed.replacen(
                "# reference: 3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13",
                "# reference: 3b1b/manim @ unknown",
                1,
            ),
        ),
        (
            "generator drift",
            committed.replacen("(numpy 2.2.4)", "(numpy unknown)", 1),
        ),
        (
            "column drift",
            committed.replacen("# columns: file\t", "# columns: path\t", 1),
        ),
        (
            "comment after the prelude",
            committed.replacen(
                "# columns: file\tdtype\tshape\tsha256\tformula\n",
                "# columns: file\tdtype\tshape\tsha256\tformula\n# ungoverned comment\n",
                1,
            ),
        ),
        (
            "blank row",
            committed.replacen("\narc_full", "\n\narc_full", 1),
        ),
        (
            "extra field",
            committed.replacen(FIRST_FORMULA, &format!("{FIRST_FORMULA}\textra"), 1),
        ),
        (
            "duplicate fixture",
            committed.replacen("arc_full_n8.npy", "arc_quarter_n4.npy", 1),
        ),
        (
            "noncanonical fixture name",
            committed.replacen("arc_quarter_n4.npy", "../arc_quarter_n4.npy", 1),
        ),
        (
            "unsupported dtype",
            committed.replacen("arc_quarter_n4.npy\t<f8", "arc_quarter_n4.npy\t>f8", 1),
        ),
        (
            "noncanonical shape",
            committed.replacen(
                "arc_quarter_n4.npy\t<f8\t9x3",
                "arc_quarter_n4.npy\t<f8\t09x3",
                1,
            ),
        ),
        (
            "non-lowercase digest",
            committed.replacen(
                "5d1019d7270ec7e7576d0f4d7ac968a5185134793075be689c13fbcbff5e2c29",
                "5D1019d7270ec7e7576d0f4d7ac968a5185134793075be689c13fbcbff5e2c29",
                1,
            ),
        ),
        (
            "formula drift",
            committed.replacen(
                FIRST_FORMULA,
                "quadratic_bezier_points_for_arc(TAU/4, 4)",
                1,
            ),
        ),
        (
            "formula field limit",
            committed.replacen(FIRST_FORMULA, &"x".repeat(257), 1),
        ),
        ("generator row order", reordered),
        ("missing generator row", missing_row),
        ("extra generator row", extra_row),
    ];
    for (case, text) in malformed {
        std::fs::write(&manifest_path, text).expect("write malformed manifest");
        let error = FixtureCorpus::load(&root).expect_err(case);
        let diagnostic = error.to_string();
        assert!(
            diagnostic.len() < 256,
            "{case}: diagnostic must stay bounded, got {} bytes",
            diagnostic.len()
        );
    }

    let mut excessive_separators = committed.to_string();
    excessive_separators.extend(std::iter::repeat_n('\t', 1_000_000));
    std::fs::write(&manifest_path, excessive_separators)
        .expect("write noncanonical separator-heavy row");
    let diagnostic = FixtureCorpus::load(&root)
        .expect_err("separator-heavy row must be refused")
        .to_string();
    assert!(
        diagnostic.len() < 256,
        "separator-heavy diagnostic must stay bounded, got {} bytes",
        diagnostic.len()
    );

    let committed_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/npy");
    let fixture_bytes = std::fs::read(committed_root.join("quad_eval.npy"))
        .expect("committed shape-mismatch fixture bytes");
    std::fs::write(root.join("partial_quad.npy"), fixture_bytes).expect("write scratch fixture");
    std::fs::write(
        &manifest_path,
        committed.replacen(PARTIAL_HASH, QUAD_EVAL_HASH, 1),
    )
    .expect("write mismatched-shape manifest");
    let error = FixtureCorpus::load(&root)
        .expect("well-formed manifest")
        .array("partial_quad.npy")
        .expect_err("declared shape must bind the decoded fixture");
    assert!(
        error.to_string().contains("decoded shape [9, 3]"),
        "shape mismatch must name the decoded shape: {error}"
    );
}

#[test]
fn arc_half_fixture_matches_fmn_geom() {
    let corpus = fixtures();
    let reference = corpus.points("arc_half_n8.npy").expect("fixture decodes");
    let ours = quadratic_points_for_arc(TAU / 2.0, 8).expect("valid arc");
    check_points_abs(&reference, &ours, FIXTURE_TOL, NanPolicy::Reject)
        .expect("arc_half_n8: reference vs fmn-geom");
}

#[test]
fn partial_quad_half_fixtures_match_fmn_geom() {
    let corpus = fixtures();
    for (name, a, b) in [
        ("partial_quad_first_half.npy", 0.0, 0.5),
        ("partial_quad_second_half.npy", 0.5, 1.0),
    ] {
        let reference = corpus.points(name).expect("fixture decodes");
        let ours = partial_quadratic(&REFERENCE_QUAD, a, b);
        check_points_abs(&reference, &ours, FIXTURE_TOL, NanPolicy::Reject)
            .expect("partial-quad half: reference vs fmn-geom");
    }
}

#[test]
fn quad_eval_fixture_matches_fmn_geom() {
    let corpus = fixtures();
    let reference = corpus.points("quad_eval.npy").expect("fixture decodes");
    let ours: Vec<Vec3> = (0..=8)
        .map(|i| reference_quad_point(f64::from(i) / 8.0))
        .collect();
    check_points_abs(&reference, &ours, FIXTURE_TOL, NanPolicy::Reject)
        .expect("quad_eval: reference vs fmn-geom");
}

#[test]
fn integer_interpolate_fixture_matches_fmn_geom() {
    let corpus = fixtures();
    let reference = corpus
        .rows("integer_interpolate.npy")
        .expect("fixture decodes");
    let alphas = [0.05, 0.25, 0.5, 0.75, 0.95];
    assert_eq!(reference.len(), alphas.len(), "fixture row count");
    for (row, alpha) in reference.iter().zip(alphas) {
        let (index, residue) =
            integer_interpolate(0, 10, alpha).expect("fixture alphas are finite and in range");
        let ours = [index as f64, residue];
        let error = (row[0] - ours[0]).abs().max((row[1] - ours[1]).abs());
        assert!(
            error < FIXTURE_TOL,
            "integer_interpolate(0, 10, {alpha}): reference {row:?} vs ours {ours:?}"
        );
    }
}

#[test]
fn the_fixture_corpus_covers_the_taxonomy_rows() {
    // The corpus guard: every positional row the taxonomy asks for is
    // present and hash-verified (family shapes live in fmn-library's
    // geometry parity fixtures — see the oracles module docs).
    let corpus = fixtures();
    assert!(corpus.len() >= 9, "the full corpus is present");
    for name in [
        "arc_quarter_n4.npy",
        "arc_full_n8.npy",
        "arc_neg_third_n2.npy",
        "arc_half_n8.npy",
        "partial_quad.npy",
        "partial_quad_first_half.npy",
        "partial_quad_second_half.npy",
        "quad_eval.npy",
        "integer_interpolate.npy",
    ] {
        corpus.array(name).expect("row present and hash-verified");
    }
}
