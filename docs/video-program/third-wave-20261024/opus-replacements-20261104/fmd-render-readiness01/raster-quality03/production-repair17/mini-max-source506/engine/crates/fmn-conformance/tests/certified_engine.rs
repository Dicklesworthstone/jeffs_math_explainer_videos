//! The certified CPU engine's self-goldens and PG-5's per-commit sweep (fm-ig3).
//!
//! `self_goldens.rs` locks geometry and stage lifecycles and says, at its top,
//! "frame hashes join these artifacts once Lumen exists". Lumen exists; this is
//! that file.
//!
//! ## Why these locks are `Scope::Certified`
//!
//! `docs/INPUT_CLOSURE.md` §6: since §5 is frozen and the arithmetic is
//! portable, artifacts on the certified path get **one** lock shared by the
//! whole matrix rather than three per-platform ones. The consequence is the
//! point — a cross-platform lock **fails on whichever machine breaks it**,
//! instead of passing everywhere and waiting for someone to re-run a
//! three-platform sweep. G0-6 demonstrated the shape with a single frame
//! constant; this is the shape applied to a corpus.
//!
//! The raw frame remains the renderer's defining lock: §16.7's certified
//! artifact kinds start there. Each raw lock now has a canonical PNG companion
//! produced by the table-only `Rgba16F -> Rgba8` conversion and the owned
//! deterministic encoder. The companion does not replace the engine oracle;
//! it makes the exact locked result directly reviewable in the Look Gallery.
//!
//! ## The three corpora, and who asked for them
//!
//! - **`caps_and_joins.v1`** — fm-oac's owed acceptance item. §10.3 asks for
//!   "cap/join golden fixtures (every combination, including miter-limit escapes
//!   to bevel)". The *geometry* of every combination is tested in
//!   `fmn_render::stroke`; a golden is a rasterized frame, and there was no
//!   engine to rasterize one, so fm-oac closed by handing the item here.
//! - **`fills.v1`** — the same for §10.2: flat and gradient fills, a winding
//!   hole, the hinted disc and rectangle routes, the general fill path,
//!   a self-intersecting pentagram, and the inner border.
//! - **`composite.v1`** — the frame that stands where G0-6's does. It carries
//!   what fm-zn9 named — gradient fills, a winding hole, joins, a tapered
//!   stroke, a sub-AA hairline, alpha compositing throughout — in the engine's
//!   own terms rather than the spike's IR. It is not the *same* frame: the spike
//!   is not a workspace member (ADR-0003's non-shipped tier), so its frame is not
//!   reachable from this build. What transfers is the property, not the digest.
//!
//! Re-blessing: `UPDATE_GOLDENS=1 cargo test -p fmn-conformance --test
//! certified_engine`, then commit the lock diff. GOVERNANCE §5 applies with full
//! force here — **a drift is a finding to adjudicate, never a number to
//! re-bless**. These bytes are the product promise.

use fmn_conformance::gallery::canonical_png_panel;
use fmn_conformance::golden::{GoldenStore, Scope};
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, GREEN_B, MAROON_C, RED_C, TEAL_B, WHITE, YELLOW_C};
#[cfg(feature = "metal")]
use fmn_frame::{ChromaSiting, ColorRange};
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_library::style::VStyle;
use fmn_library::{Circle, Dot, Line, Polygon, Rectangle};
use fmn_mobject::{JointType, Mob, Stage};
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{
    AaPolicy, EngineIdentity, FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
    FAST_VISUAL_BUDGET_V1_MIN_SSIM, FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR, FrameConfig, FrameJob,
    Tier, encode_frame, journal_digest,
};
use fmn_render::fill::{FillKernel, MonoTable, instance_translation};
#[cfg(feature = "metal")]
use fmn_render::metal::{
    METAL_RGBA8_TRANSFER_V1_MAX_CODE_ERROR, METAL_THREE_D_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
    METAL_THREE_D_VISUAL_BUDGET_V1_MIN_SSIM, METAL_THREE_D_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
    METAL_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR, METAL_VISUAL_BUDGET_V1_MIN_SSIM,
    METAL_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR, MetalRenderer, MetalReport,
};
use fmn_render::plan::RenderPlan;
#[cfg(feature = "metal")]
use fmn_render::{
    Camera, CameraConfig, SurfaceDraw, SurfaceMesh, SurfaceVertex, ThreeDDraw, ThreeDJob,
    TrueDotDraw,
};
use std::path::PathBuf;

/// The frame every corpus renders into.
///
/// Small on purpose, for G0-6's reason: a golden nobody can afford to re-run is
/// a golden nobody re-runs. Bit-identity over a quarter of a million components
/// either holds or it does not.
const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;

/// The declared certified configuration's tile dimensions (C10).
const TILING: Tiling = Tiling {
    macro_tile: 128,
    fine_tile: 16,
};

/// Pixels per scene unit. The Reference's is 135 at 1080p; this is the same
/// order, chosen so a default `Dot` takes the hinted radial kernel and a
/// two-unit `Circle` does not — the boundary §10.2 draws.
const SCALE: f64 = 60.0;

fn store() -> GoldenStore {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens");
    GoldenStore::new(dir, "certified_engine", Scope::Certified).expect("store")
}

fn config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        ScreenMap {
            scale: SCALE,
            // Object-space origin at the frame centre, +Y up the frame like
            // every scene front door (fm-sq8.9 retired the pre-camera +y-down
            // convention that rendered this corpus mirrored).
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: true,
        },
        // The Reference's #333333, decoded once here rather than per pixel.
        Srgb::from_rgb8(0x33, 0x33, 0x33).to_linear(1.0),
    )
}

/// Render a scene at an explicit tier and thread count.
///
/// The tier is threaded through [`FrameJob::with_identity`] rather than ignored,
/// so when fm-4wt makes the engine dispatch on it this harness needs no change:
/// the sweep below already renders the corpus once per registered tier and holds
/// every one of them to the scalar definition.
fn render(stage: &Stage, tier: Tier, threads: usize) -> Vec<u8> {
    render_with_aa(stage, tier, threads, AaPolicy::Adaptive)
}

/// [`render`] with an explicit A/B policy.
fn render_with_aa(stage: &Stage, tier: Tier, threads: usize, aa: AaPolicy) -> Vec<u8> {
    let identity = EngineIdentity {
        tier,
        ..EngineIdentity::certified()
    };
    let frame = render_frame(stage, identity, threads, aa);
    encode_frame(&frame).expect("the frame encodes into its canonical document")
}

/// Render a raw frame through one explicitly journaled engine identity.
fn render_frame(
    stage: &Stage,
    identity: EngineIdentity,
    threads: usize,
    aa: AaPolicy,
) -> FrameBuffer {
    let cfg = config().with_aa_policy(aa);
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0).expect("valid certified-engine fixture");
    let mono = MonoTable::build(&plan, cfg.map).expect("bounded certified monotone table");
    let mut binning = Binning::build(&plan, cfg.viewport, TILING, cfg.map)
        .expect("bounded certified-engine binning");
    binning.prune_occluded(&plan).expect("matching plan");
    let job = FrameJob::with_identity(&plan, &mono, &binning, cfg, identity)
        .expect("matching frame artifacts");
    job.render(threads).expect("the engine renders the frame")
}

/// The scalar definition of a scene's frame, single-threaded.
fn definition(stage: &Stage) -> Vec<u8> {
    encode_frame(&definition_frame(stage)).expect("the frame encodes into its canonical document")
}

/// The scalar definition before canonical-document serialization.
fn definition_frame(stage: &Stage) -> FrameBuffer {
    render_frame(stage, EngineIdentity::certified(), 1, AaPolicy::Adaptive)
}

/// Linear-channel and perceptual divergence between two raw frames.
#[derive(Debug, Clone, Copy)]
struct Divergence {
    maximum: f64,
    maximum_at: (u32, u32, usize),
    reference_at_maximum: f64,
    candidate_at_maximum: f64,
    rms: f64,
    ssim: f64,
}

/// Compare raw linear-light frames under §16.3's two-part engine budget.
fn divergence(reference: &FrameBuffer, candidate: &FrameBuffer) -> Divergence {
    assert_eq!(
        reference.layout(),
        candidate.layout(),
        "engine-equivalence layouts differ"
    );
    let mut maximum = 0.0f64;
    let mut maximum_at = (0, 0, 0);
    let mut reference_at_maximum = 0.0;
    let mut candidate_at_maximum = 0.0;
    let mut squared = 0.0;
    let mut channels = 0u64;
    for y in 0..reference.layout().height() {
        for x in 0..reference.layout().width() {
            for (channel, (a, b)) in read_pixel(reference, x, y)
                .into_iter()
                .zip(read_pixel(candidate, x, y))
                .enumerate()
            {
                let error = (a - b).abs();
                if error > maximum {
                    maximum = error;
                    maximum_at = (x, y, channel);
                    reference_at_maximum = a;
                    candidate_at_maximum = b;
                }
                squared += error * error;
                channels += 1;
            }
        }
    }
    assert!(channels > 0, "a frame comparison must contain channels");
    Divergence {
        maximum,
        maximum_at,
        reference_at_maximum,
        candidate_at_maximum,
        rms: (squared / channels as f64).sqrt(),
        ssim: ssim_luma(reference, candidate),
    }
}

/// Decode one `Rgba16F` pixel into exact `f64` values.
fn read_pixel(frame: &FrameBuffer, x: u32, y: u32) -> [f64; 4] {
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

/// Global SSIM over the canonical sRGB8 Rec. 709 luma plane.
///
/// The spike deliberately selected the global form as a stable smoke alarm
/// rather than shipping an unreviewed windowed metric. The production
/// engine-equivalence lane keeps that exact ruling and pairs it with the hard
/// linear-channel bounds above.
fn ssim_luma(reference: &FrameBuffer, candidate: &FrameBuffer) -> f64 {
    let luma = |frame: &FrameBuffer| {
        let layout = FrameLayout::tight(
            PixelFormat::Rgba8,
            frame.layout().width(),
            frame.layout().height(),
        )
        .expect("the comparison layout is valid");
        let mut encoded = FrameBuffer::new(layout);
        fmn_frame::convert::rgba16f_to_rgba8(frame, &mut encoded)
            .expect("the raw frame converts canonically");
        let mut values =
            Vec::with_capacity(frame.layout().width() as usize * frame.layout().height() as usize);
        let width_bytes = frame.layout().width() as usize * 4;
        for y in 0..frame.layout().height() as usize {
            let row = &encoded.plane(0)
                [y * encoded.layout().stride(0)..y * encoded.layout().stride(0) + width_bytes];
            values.extend(row.as_chunks::<4>().0.iter().map(|pixel| {
                0.2126 * f64::from(pixel[0])
                    + 0.7152 * f64::from(pixel[1])
                    + 0.0722 * f64::from(pixel[2])
            }));
        }
        values
    };
    let reference = luma(reference);
    let candidate = luma(candidate);
    assert_eq!(
        reference.len(),
        candidate.len(),
        "SSIM planes differ in length"
    );
    assert!(!reference.is_empty(), "SSIM requires at least one pixel");
    let count = reference.len() as f64;
    let reference_mean = reference.iter().sum::<f64>() / count;
    let candidate_mean = candidate.iter().sum::<f64>() / count;
    let mut reference_variance = 0.0;
    let mut candidate_variance = 0.0;
    let mut covariance = 0.0;
    for (&reference, &candidate) in reference.iter().zip(&candidate) {
        reference_variance += (reference - reference_mean) * (reference - reference_mean);
        candidate_variance += (candidate - candidate_mean) * (candidate - candidate_mean);
        covariance += (reference - reference_mean) * (candidate - candidate_mean);
    }
    let divisor = (count - 1.0).max(1.0);
    reference_variance /= divisor;
    candidate_variance /= divisor;
    covariance /= divisor;

    let c1 = (0.01 * 255.0f64).powi(2);
    let c2 = (0.03 * 255.0f64).powi(2);
    ((2.0 * reference_mean * candidate_mean + c1) * (2.0 * covariance + c2))
        / ((reference_mean * reference_mean + candidate_mean * candidate_mean + c1)
            * (reference_variance + candidate_variance + c2))
}

fn add(stage: &mut Stage, m: impl Into<fmn_mobject::Mobject>) -> Mob {
    let h = stage.add(m);
    stage.add_to_scene(h).expect("live");
    h
}

// ------------------------------------------------------------- the corpora

/// §10.3's cap/join corpus: every joint setting over three corner severities,
/// plus the open ends that carry round caps.
///
/// The three severities are chosen so the settings actually diverge. A shallow
/// corner puts bevel, miter and round within a pixel of one another; a sharp one
/// separates them; and the near-reversal is past `MITER_LIMIT = √10`, so a miter
/// must fall back to a bevel there rather than growing a spike of unbounded
/// length. A corpus without the third would lock the limit's absence.
fn caps_and_joins() -> Stage {
    let mut stage = Stage::new();
    let joints = [
        JointType::Auto,
        JointType::NoJoint,
        JointType::Bevel,
        JointType::Miter,
    ];
    // Three corner severities, given as the arms' half-separation over the
    // triangle's 0.4 vertical extent — deliberately as offsets rather than
    // angles, so the corpus needs no trigonometry and therefore cannot smuggle a
    // platform libm into a certified golden. The apex half-angles are
    // `atan(dx / 0.4)` = 40.4°, 15.4° and 2.9°, so the miter lengths `1/sin(θ/2)`
    // are **1.54, 3.77 and 20.02** half-widths. That straddles
    // `MITER_LIMIT = √10 ≈ 3.1623`: the first miters and the other two escape to
    // a bevel. A corpus without that third column would lock the limit's absence
    // rather than its behaviour.
    let severities = [0.34_f64, 0.11, 0.02];
    // The shapes are closed triangles rather than open Vs, which is more
    // coverage and not less: a closed outline has a **wrap join** at the point
    // where its last segment meets its first, and `join_wedges` names that as
    // the one corner a naive implementation leaves round under a miter setting.
    for (row, joint) in joints.into_iter().enumerate() {
        for (col, dx) in severities.into_iter().enumerate() {
            let x = -2.1 + 1.4 * col as f64;
            let y = -1.2 + 0.62 * row as f64;
            let m = add(
                &mut stage,
                Polygon::new([
                    [x - dx, y - 0.2, 0.0],
                    [x, y + 0.2, 0.0],
                    [x + dx, y - 0.2, 0.0],
                ])
                .color(WHITE),
            );
            // 14 width units is 8.4 px at this scale, so a miter extension of up
            // to `MITER_LIMIT` half-widths reaches ~13 px past the corner — a
            // difference a reviewer can see and a golden can lose. At the 6
            // this fixture started with, all four settings sat inside the AA
            // band and the corpus locked a picture the joint could not move.
            stage.set_stroke(m, Some(WHITE), Some(14.0), Some(1.0), None, true);
            stage.set_fill(m, None, Some(0.0), None, true);
            stage.uniforms_mut(m).expect("live").joint_type = joint;
        }
    }
    // Open ends, whose distance to a pixel is radial and whose level set is
    // therefore a semicircle — round caps, with nobody asking for one. Three
    // widths, because a cap is only visible at the width it is drawn at.
    for (i, w) in [3.0_f64, 10.0, 24.0].into_iter().enumerate() {
        let x = -2.1 + 1.4 * i as f64;
        let m = add(
            &mut stage,
            Line::new([x - 0.42, 1.25, 0.0], [x + 0.42, 1.25, 0.0])
                .color(TEAL_B)
                .build()
                .expect("the cap fixture line is straight"),
        );
        stage.set_stroke(m, Some(TEAL_B), Some(w), Some(1.0), None, true);
    }
    stage
}

/// §10.2's fill corpus: flat, gradient, a winding hole, both hinted routes, and
/// the inner border.
fn fills() -> Stage {
    let mut stage = Stage::new();

    // A flat opaque disc, and a translucent one overlapping it: the composite.
    let a = add(
        &mut stage,
        Circle::new().radius(0.62).arc_center([-1.85, -0.5, 0.0]),
    );
    stage.set_fill(a, Some(BLUE_C), Some(1.0), Some(0.0), true);
    stage.set_stroke(a, None, Some(0.0), Some(0.0), None, true);
    let b = add(
        &mut stage,
        Circle::new().radius(0.52).arc_center([-1.45, -0.2, 0.0]),
    );
    stage.set_fill(b, Some(RED_C), Some(0.55), Some(0.0), true);
    stage.set_stroke(b, None, Some(0.0), Some(0.0), None, true);

    // A gradient fill: the ramp's two ends differ, so the mean-value field runs.
    let g = add(
        &mut stage,
        Circle::new().radius(0.7).arc_center([-0.35, 0.35, 0.0]),
    );
    stage.set_fill(g, Some(YELLOW_C), Some(0.9), Some(0.0), true);
    stage.set_stroke(g, None, Some(0.0), Some(0.0), None, true);
    gradient(&mut stage, g, RED_C);

    // The same, with an inner border: the seam goes crisp inside the band and
    // the silhouette does not move (ADR-0011).
    let bordered = add(
        &mut stage,
        Circle::new().radius(0.7).arc_center([1.1, 0.35, 0.0]),
    );
    stage.set_fill(bordered, Some(YELLOW_C), Some(0.9), Some(2.0), true);
    stage.set_stroke(bordered, None, Some(0.0), Some(0.0), None, true);
    gradient(&mut stage, bordered, RED_C);

    // The hinted routes: a Dot takes the radial kernel, a Rectangle the box one.
    let d = add(
        &mut stage,
        Dot::new().point([-0.35, -0.9, 0.0]).radius(0.16),
    );
    stage.set_fill(d, Some(GREEN_B), Some(1.0), Some(0.0), true);
    let r = add(
        &mut stage,
        Rectangle::new()
            .width(1.1)
            .height(0.55)
            .color(MAROON_C)
            .build()
            .expect("the fill fixture rectangle is unrounded"),
    );
    stage.shift(r, [1.15, -0.85, 0.0]);
    stage.set_fill(r, Some(MAROON_C), Some(1.0), Some(0.0), true);
    stage.set_stroke(r, None, Some(0.0), Some(0.0), None, true);

    // A **self-intersecting** fill through the general quadratic machinery.
    //
    // Without this the corpus barely exercises §10.2's centrepiece: at 60 px per
    // unit every `Circle` here is inside `HINT_BUDGET_PX`, so it takes the disc
    // kernel, and the general path was left to the annuli alone. A pentagram is
    // the right shape for the gap — its centre pentagon has winding **2**, so it
    // fills under the nonzero rule and would be a hole under even-odd, which
    // makes it a discriminator for the rule rather than merely a workout for the
    // evaluator. §10.2 names "winding-consistent self-intersection behavior"
    // explicitly, and this is the only fixture that can fail it.
    //
    // The vertices are literals rather than a trig loop, for the reason the
    // cap/join grid gives: a certified golden must not compute its own geometry
    // with a platform libm.
    let star = add(
        &mut stage,
        Polygon::new([
            [-1.85, 1.27, 0.0],
            [-2.173282, 0.275041, 0.0],
            [-1.326919, 0.889959, 0.0],
            [-2.373081, 0.889959, 0.0],
            [-1.526718, 0.275041, 0.0],
        ])
        .color(GREEN_B),
    );
    stage.set_fill(star, Some(GREEN_B), Some(0.85), Some(0.0), true);
    stage.set_stroke(star, None, Some(0.0), Some(0.0), None, true);

    // A winding hole. It has to be **one** mobject with two counter-wound
    // subpaths — an `Annulus`, which is exactly that — because two overlapping
    // mobjects test compositing rather than winding, and a second shape filled
    // at zero opacity tests nothing at all. A crossing-order or orientation bug
    // shows up here as a solid disc.
    let ring = add(
        &mut stage,
        fmn_library::Annulus::new()
            .inner_radius(0.3)
            .outer_radius(0.62)
            .center([2.1, -0.2, 0.0]),
    );
    stage.set_fill(ring, Some(TEAL_B), Some(1.0), Some(0.0), true);
    stage.set_stroke(ring, None, Some(0.0), Some(0.0), None, true);

    stage
}

/// Give a mobject a fill ramp by writing the last record's colour.
///
/// The IR carries the ramp's two ends, which is how the Reference expresses a
/// gradient along a path; setting only the last record is what makes the two
/// ends differ and therefore what turns the interior field on.
fn gradient(stage: &mut Stage, mob: Mob, end: Srgb) {
    let Some(entry) = stage.get_mut(mob) else {
        return;
    };
    let n = entry.buffer.len();
    if n == 0 {
        return;
    }
    entry.buffer.write(
        n - 1,
        "fill_rgba",
        &[end.r as f32, end.g as f32, end.b as f32, 0.9],
    );
}

/// The frame that stands where G0-6's does: everything fm-zn9 named, in the
/// engine's own terms.
fn composite() -> Stage {
    let mut stage = Stage::new();

    // A gradient-filled disc.
    let disc = add(
        &mut stage,
        Circle::new().radius(0.85).arc_center([-1.6, -0.15, 0.0]),
    );
    stage.set_fill(disc, Some(BLUE_C), Some(0.85), Some(0.0), true);
    stage.set_stroke(disc, None, Some(0.0), Some(0.0), None, true);
    gradient(&mut stage, disc, TEAL_B);

    // A gradient-filled ring whose hole depends on the winding rule — the same
    // structure G0-6's frame carried, so an orientation bug moves this hash.
    let ring = add(
        &mut stage,
        fmn_library::Annulus::new()
            .inner_radius(0.28)
            .outer_radius(0.75)
            .center([1.4, 0.1, 0.0]),
    );
    stage.set_fill(ring, Some(YELLOW_C), Some(0.9), Some(0.0), true);
    stage.set_stroke(ring, None, Some(0.0), Some(0.0), None, true);
    gradient(&mut stage, ring, RED_C);

    // A sharp zigzag with a real joint override, so joins reach pixels.
    let zig = add(
        &mut stage,
        Polygon::new([
            [-2.2, 1.05, 0.0],
            [-1.6, 0.55, 0.0],
            [-1.0, 1.05, 0.0],
            [-0.4, 0.55, 0.0],
            [0.2, 1.05, 0.0],
        ])
        .color(MAROON_C),
    );
    stage.set_fill(zig, None, Some(0.0), None, true);
    stage.set_stroke(zig, Some(MAROON_C), Some(7.0), Some(1.0), None, true);
    stage.uniforms_mut(zig).expect("live").joint_type = JointType::Miter;

    // A tapered gradient stroke over the fills, exercising the arc-length ramp.
    let taper = add(
        &mut stage,
        Line::new([-2.3, -1.15, 0.0], [2.3, -0.85, 0.0])
            .color(GREEN_B)
            .build()
            .expect("the taper fixture line is straight"),
    );
    stage.set_stroke(taper, Some(GREEN_B), Some(13.0), Some(0.95), None, true);
    taper_stroke(&mut stage, taper, BLUE_C, 1.0);

    // A stroke drawn behind its own translucent fill (R-5's other branch).
    let behind = add(
        &mut stage,
        Circle::new().radius(0.5).arc_center([0.0, -0.1, 0.0]),
    );
    stage.set_fill(behind, Some(WHITE), Some(0.45), Some(0.0), true);
    stage.set_stroke(behind, Some(RED_C), Some(14.0), Some(1.0), Some(true), true);

    // A hairline finer than the AA band, so the sub-pixel regime is in the hash.
    let hair = add(
        &mut stage,
        Line::new([-2.4, 1.35, 0.0], [2.4, 1.28, 0.0])
            .color(WHITE)
            .build()
            .expect("the hairline fixture is straight"),
    );
    stage.set_stroke(hair, Some(WHITE), Some(0.45), Some(1.0), None, true);

    stage
}

/// Give a stroke a width and colour taper by writing the last record.
fn taper_stroke(stage: &mut Stage, mob: Mob, end: Srgb, end_width: f64) {
    let Some(entry) = stage.get_mut(mob) else {
        return;
    };
    let n = entry.buffer.len();
    if n == 0 {
        return;
    }
    entry.buffer.write(
        n - 1,
        "stroke_rgba",
        &[end.r as f32, end.g as f32, end.b as f32, 0.4],
    );
    entry
        .buffer
        .write(n - 1, "stroke_width", &[end_width as f32]);
}

/// The corpus, by locked artifact name.
fn corpus() -> Vec<(&'static str, Stage)> {
    vec![
        ("caps_and_joins.v1", caps_and_joins()),
        ("fills.v1", fills()),
        ("composite.v1", composite()),
    ]
}

// ---------------------------------------------------------------- the gates

#[cfg(feature = "metal")]
#[test]
fn metal_annex_stays_inside_budget_and_reuses_its_surfaces() {
    if !MetalRenderer::is_available() {
        return;
    }
    let mut renderer =
        MetalRenderer::new().expect("an available Metal device must compile the production shader");

    for (name, stage) in corpus() {
        let certified = render_frame(&stage, EngineIdentity::certified(), 1, AaPolicy::Adaptive);
        let cfg = config();
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .expect("valid certified-engine fixture");
        let mono = MonoTable::build(&plan, cfg.map).expect("bounded certified monotone table");
        let mut binning = Binning::build(&plan, cfg.viewport, TILING, cfg.map)
            .expect("bounded certified-engine binning");
        binning.prune_occluded(&plan).expect("matching plan");

        let (first, first_report) = renderer
            .render_raw(&plan, &mono, &binning, cfg)
            .expect("Metal renders the conformance frame");
        let (second, second_report) = renderer
            .render_raw(&plan, &mono, &binning, cfg)
            .expect("Metal repeats the conformance frame");
        let measured = divergence(&certified, &first);

        assert_eq!(first_report.identity, EngineIdentity::metal());
        assert_eq!(first_report.output_format, PixelFormat::Rgba16F);
        assert_eq!(
            first_report.threads_per_threadgroup,
            usize::try_from(TILING.fine_tile * TILING.fine_tile)
                .expect("the declared tile size fits usize")
        );
        assert!(first_report.max_threads_per_threadgroup >= first_report.threads_per_threadgroup);
        assert!(first_report.upload_bytes > 0);
        assert_eq!(first_report.readback_bytes, first.as_bytes().len());
        assert!(first_report.frames_per_second().is_some());
        assert!(second_report.raw_surface_reused);
        assert_eq!(
            first_report.backend_digest(),
            second_report.backend_digest()
        );
        assert_eq!(
            first.as_bytes(),
            second.as_bytes(),
            "{name} Metal output changed between identical dispatches"
        );
        assert!(
            measured.maximum <= METAL_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
            "{name} Metal max={} at {:?} (certified {}, Metal {}) exceeds {}",
            measured.maximum,
            measured.maximum_at,
            measured.reference_at_maximum,
            measured.candidate_at_maximum,
            METAL_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR
        );
        assert!(
            measured.rms <= METAL_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
            "{name} Metal rms={} exceeds {}",
            measured.rms,
            METAL_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR
        );
        assert!(
            measured.ssim >= METAL_VISUAL_BUDGET_V1_MIN_SSIM,
            "{name} Metal ssim={} is below {}",
            measured.ssim,
            METAL_VISUAL_BUDGET_V1_MIN_SSIM
        );

        if name == "composite.v1" {
            let mut expected = FrameBuffer::new(
                FrameLayout::tight(PixelFormat::Rgba8, WIDTH, HEIGHT)
                    .expect("RGBA8 comparison layout"),
            );
            fmn_frame::convert::rgba16f_to_rgba8(&first, &mut expected)
                .expect("canonical transfer");
            let (gpu_first, gpu_first_report) = renderer
                .render_rgba8(&plan, &mono, &binning, cfg)
                .expect("Metal transfers the preview surface");
            let (gpu_second, gpu_second_report) = renderer
                .render_rgba8(&plan, &mono, &binning, cfg)
                .expect("Metal repeats the preview transfer");
            assert_eq!(gpu_first_report.output_format, PixelFormat::Rgba8);
            assert!(gpu_second_report.raw_surface_reused);
            assert!(gpu_second_report.output_surface_reused);
            assert_eq!(gpu_first.as_bytes(), gpu_second.as_bytes());
            assert_eq!(
                maximum_rgba8_code_error(&expected, &gpu_first),
                METAL_RGBA8_TRANSFER_V1_MAX_CODE_ERROR
            );
            let rgba_transfer_digest = gpu_first_report.transfer_table_digest;
            let rgba_readback_bytes = expected.as_bytes().len();

            for (format, range, siting) in [
                (PixelFormat::Nv12, ColorRange::Limited, ChromaSiting::Left),
                (PixelFormat::Nv12, ColorRange::Limited, ChromaSiting::Center),
                (PixelFormat::Nv12, ColorRange::Full, ChromaSiting::Left),
                (PixelFormat::Nv12, ColorRange::Full, ChromaSiting::Center),
                (PixelFormat::P010, ColorRange::Limited, ChromaSiting::Left),
                (PixelFormat::P010, ColorRange::Limited, ChromaSiting::Center),
            ] {
                let layout = negotiated_yuv_layout(format);
                let mut cpu = FrameBuffer::new(layout.clone());
                match format {
                    PixelFormat::Nv12 => {
                        fmn_frame::convert::rgba_to_nv12(&expected, &mut cpu, range, siting)
                            .expect("CPU NV12 oracle");
                    }
                    PixelFormat::P010 => {
                        fmn_frame::convert::rgba_to_p010(&expected, &mut cpu, range, siting)
                            .expect("CPU P010 oracle");
                    }
                    _ => unreachable!("the case table contains only YUV420 formats"),
                }
                let (gpu_first, yuv_first_report) = match format {
                    PixelFormat::Nv12 => renderer
                        .render_nv12(&plan, &mono, &binning, cfg, layout.clone(), range, siting)
                        .expect("Metal transfers NV12 before readback"),
                    PixelFormat::P010 => renderer
                        .render_p010(&plan, &mono, &binning, cfg, layout.clone(), range, siting)
                        .expect("Metal transfers P010 before readback"),
                    _ => unreachable!("the case table contains only YUV420 formats"),
                };
                let (gpu_second, yuv_second_report) = match format {
                    PixelFormat::Nv12 => renderer
                        .render_nv12(&plan, &mono, &binning, cfg, layout.clone(), range, siting)
                        .expect("Metal repeats NV12 transfer"),
                    PixelFormat::P010 => renderer
                        .render_p010(&plan, &mono, &binning, cfg, layout.clone(), range, siting)
                        .expect("Metal repeats P010 transfer"),
                    _ => unreachable!("the case table contains only YUV420 formats"),
                };

                assert_eq!(gpu_first.layout(), &layout);
                assert_eq!(yuv_first_report.output_format, format);
                assert_eq!(yuv_first_report.color_range, Some(range));
                assert_eq!(yuv_first_report.chroma_siting, Some(siting));
                assert_eq!(yuv_first_report.transfer_table_digest, rgba_transfer_digest);
                assert_eq!(yuv_first_report.readback_bytes, layout.total_bytes());
                assert!(
                    yuv_first_report.readback_bytes < rgba_readback_bytes,
                    "{format:?} did not reduce the RGBA8 readback"
                );
                assert!(yuv_second_report.raw_surface_reused);
                assert!(yuv_second_report.output_surface_reused);
                assert_eq!(gpu_first.as_bytes(), gpu_second.as_bytes());
                assert_eq!(
                    cpu.as_bytes(),
                    gpu_first.as_bytes(),
                    "{format:?} {range:?} {siting:?} diverged from Reel's CPU oracle"
                );
            }
        }
    }
}

#[cfg(feature = "metal")]
struct MetalThreeDCorpus {
    camera: Camera,
    surface: SurfaceMesh,
    dots: Vec<TrueDotDraw>,
}

#[cfg(feature = "metal")]
impl MetalThreeDCorpus {
    fn new() -> Self {
        let background = Srgb::from_rgb8(0x18, 0x1a, 0x20).to_linear(1.0);
        let camera = Camera::new(CameraConfig {
            resolution: (WIDTH, HEIGHT),
            samples: 2,
            background,
            ..CameraConfig::default()
        })
        .expect("3D camera");
        let nu = 6u32;
        let nv = 5u32;
        let mut vertices = Vec::with_capacity((nu * nv) as usize);
        for u in 0..nu {
            for v in 0..nv {
                let x = -3.2 + 6.4 * f64::from(u) / f64::from(nu - 1);
                let y = -1.65 + 3.3 * f64::from(v) / f64::from(nv - 1);
                let z = -0.7 + 0.08 * x * y;
                let color = Srgb {
                    r: 0.18 + 0.07 * f64::from(u),
                    g: 0.24 + 0.08 * f64::from(v),
                    b: 0.68 - 0.04 * f64::from(u),
                }
                .to_linear(0.88);
                vertices.push(SurfaceVertex::colored(
                    [x, y, z],
                    [-0.08 * y, -0.08 * x, 1.0],
                    color,
                ));
            }
        }
        let surface = SurfaceMesh::from_uv_grid(vertices, (nu, nv)).expect("fixed UV surface");
        let mut dots = Vec::new();
        for row in 0..5u32 {
            for column in 0..12u32 {
                let x = -3.3 + 0.6 * f64::from(column);
                let y = -1.35 + 0.68 * f64::from(row);
                let z = if (row + column).is_multiple_of(3) {
                    -1.1
                } else {
                    0.25
                };
                let color = if (row + column).is_multiple_of(2) {
                    Srgb::from_rgb8(0xff, 0x78, 0x30).to_linear(0.68)
                } else {
                    Srgb::from_rgb8(0x42, 0xc9, 0xff).to_linear(0.62)
                };
                let mut dot = if column.is_multiple_of(3) {
                    TrueDotDraw::new([x, y, z], 0.22, color)
                } else {
                    TrueDotDraw::glow([x, y, z], 0.34, color)
                };
                dot.depth_test = row.is_multiple_of(2);
                if row == 0 && column == 0 {
                    dot.shading = [0.3, 0.2, 0.4];
                }
                dots.push(dot);
            }
        }
        Self {
            camera,
            surface,
            dots,
        }
    }

    fn with_job<R>(&self, run: impl FnOnce(&ThreeDJob<'_>) -> R) -> R {
        let mut surface_draw = SurfaceDraw::new(&self.surface);
        surface_draw.depth_test = true;
        let mut draws = Vec::with_capacity(self.dots.len() + 1);
        draws.push(ThreeDDraw::Surface(surface_draw));
        draws.extend(self.dots.iter().copied().map(ThreeDDraw::TrueDot));
        let job = ThreeDJob::new(&self.camera, &draws, TILING).expect("prepared 3D corpus");
        run(&job)
    }
}

#[cfg(feature = "metal")]
#[test]
fn metal_annex_keeps_glow_dots_and_lit_surfaces_inside_the_three_d_budget() {
    if !MetalRenderer::is_available() {
        return;
    }
    MetalThreeDCorpus::new().with_job(|job| {
        let cpu = job.render(1).expect("CPU 3D oracle");
        let mut renderer = MetalRenderer::new()
            .expect("an available Metal device must compile the production shader");
        let (first, first_report) = renderer
            .render_three_d_raw(job)
            .expect("Metal renders the prepared 3D corpus");
        let (second, second_report) = renderer
            .render_three_d_raw(job)
            .expect("Metal repeats the prepared 3D corpus");
        let measured = divergence(&cpu, &first);
        println!(
            "{{\"schema\":\"fmn.metal_3d_equivalence.v1\",\
             \"maximum\":{},\"rms\":{},\"ssim\":{},\
             \"upload_bytes\":{},\"readback_bytes\":{}}}",
            measured.maximum,
            measured.rms,
            measured.ssim,
            first_report.upload_bytes,
            first_report.readback_bytes,
        );

        assert_eq!(first_report.identity, EngineIdentity::metal());
        assert_eq!(first_report.output_format, PixelFormat::Rgba16F);
        assert!(first_report.upload_bytes > 0);
        assert_eq!(first_report.readback_bytes, first.as_bytes().len());
        assert!(second_report.raw_surface_reused);
        assert_eq!(first.as_bytes(), second.as_bytes());
        assert!(
            measured.maximum <= METAL_THREE_D_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
            "3D Metal max={} at {:?} (CPU {}, Metal {}) exceeds {}",
            measured.maximum,
            measured.maximum_at,
            measured.reference_at_maximum,
            measured.candidate_at_maximum,
            METAL_THREE_D_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR
        );
        assert!(
            measured.rms <= METAL_THREE_D_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
            "3D Metal rms={} exceeds {}",
            measured.rms,
            METAL_THREE_D_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR
        );
        assert!(
            measured.ssim >= METAL_THREE_D_VISUAL_BUDGET_V1_MIN_SSIM,
            "3D Metal ssim={} is below {}",
            measured.ssim,
            METAL_THREE_D_VISUAL_BUDGET_V1_MIN_SSIM
        );

        let mut expected = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, WIDTH, HEIGHT).expect("RGBA8 comparison layout"),
        );
        fmn_frame::convert::rgba16f_to_rgba8(&first, &mut expected).expect("canonical transfer");
        let (rgba_first, rgba_first_report) = renderer
            .render_three_d_rgba8(job)
            .expect("Metal transfers the 3D preview");
        let (rgba_second, rgba_second_report) = renderer
            .render_three_d_rgba8(job)
            .expect("Metal repeats the 3D preview transfer");
        assert_eq!(rgba_first_report.output_format, PixelFormat::Rgba8);
        assert!(rgba_first_report.raw_surface_reused);
        assert!(rgba_second_report.output_surface_reused);
        assert_eq!(rgba_first.as_bytes(), rgba_second.as_bytes());
        assert_eq!(
            maximum_rgba8_code_error(&expected, &rgba_first),
            METAL_RGBA8_TRANSFER_V1_MAX_CODE_ERROR
        );
    });
}

#[cfg(feature = "metal")]
fn negotiated_yuv_layout(format: PixelFormat) -> FrameLayout {
    let width = WIDTH as usize;
    let strides = match format {
        PixelFormat::Nv12 => [width + 8, width + 24],
        PixelFormat::P010 => [width * 2 + 16, width * 2 + 32],
        _ => unreachable!("YUV layout requested for {format:?}"),
    };
    FrameLayout::with_strides(format, WIDTH, HEIGHT, &strides).expect("valid padded YUV layout")
}

#[cfg(feature = "metal")]
fn maximum_rgba8_code_error(reference: &FrameBuffer, candidate: &FrameBuffer) -> u8 {
    assert_eq!(reference.layout(), candidate.layout());
    reference
        .as_bytes()
        .iter()
        .zip(candidate.as_bytes())
        .map(|(&a, &b)| a.abs_diff(b))
        .max()
        .unwrap_or(0)
}

#[cfg(feature = "metal")]
fn render_pg_a_format(
    renderer: &mut MetalRenderer,
    plan: &RenderPlan,
    mono: &MonoTable,
    binning: &Binning,
    config: FrameConfig,
    format: PixelFormat,
) -> MetalReport {
    match format {
        PixelFormat::Rgba8 => {
            renderer
                .render_rgba8(plan, mono, binning, config)
                .expect("PG-A RGBA8 frame")
                .1
        }
        PixelFormat::Nv12 => {
            renderer
                .render_nv12(
                    plan,
                    mono,
                    binning,
                    config,
                    FrameLayout::tight(PixelFormat::Nv12, WIDTH, HEIGHT).expect("PG-A NV12 layout"),
                    ColorRange::Limited,
                    ChromaSiting::Center,
                )
                .expect("PG-A NV12 frame")
                .1
        }
        PixelFormat::P010 => {
            renderer
                .render_p010(
                    plan,
                    mono,
                    binning,
                    config,
                    FrameLayout::tight(PixelFormat::P010, WIDTH, HEIGHT).expect("PG-A P010 layout"),
                    ColorRange::Limited,
                    ChromaSiting::Center,
                )
                .expect("PG-A P010 frame")
                .1
        }
        _ => unreachable!("unsupported PG-A format {format:?}"),
    }
}

#[cfg(feature = "metal")]
#[test]
#[ignore = "PG-A runs explicitly on the pinned Apple profile"]
fn metal_annex_pg_a_reports_the_empty_floor_beside_the_corpus() {
    if !MetalRenderer::is_available() {
        return;
    }
    const MEASURED_FRAMES: usize = 33;
    let mut renderer =
        MetalRenderer::new().expect("an available Metal device must compile the production shader");

    for (case, stage) in [("empty", Stage::new()), ("composite", composite())] {
        let cfg = config();
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .expect("valid certified-engine fixture");
        let mono = MonoTable::build(&plan, cfg.map).expect("bounded certified monotone table");
        let mut binning = Binning::build(&plan, cfg.viewport, TILING, cfg.map)
            .expect("bounded certified-engine binning");
        binning.prune_occluded(&plan).expect("matching plan");

        for format in [PixelFormat::Rgba8, PixelFormat::Nv12, PixelFormat::P010] {
            render_pg_a_format(&mut renderer, &plan, &mono, &binning, cfg, format);
            let mut elapsed = Vec::with_capacity(MEASURED_FRAMES);
            let mut last = None;
            for _ in 0..MEASURED_FRAMES {
                let report = render_pg_a_format(&mut renderer, &plan, &mono, &binning, cfg, format);
                assert!(report.raw_surface_reused);
                assert!(report.output_surface_reused);
                elapsed.push(report.elapsed);
                last = Some(report);
            }
            elapsed.sort_unstable();
            let median = elapsed[MEASURED_FRAMES / 2];
            let report = last.expect("the measurement count is nonzero");
            let fps = 1.0 / median.as_secs_f64();
            assert!(fps.is_finite() && fps > 0.0);
            assert!(report.upload_bytes > 0);
            assert_eq!(
                report.readback_bytes,
                FrameLayout::tight(format, WIDTH, HEIGHT)
                    .expect("PG-A output layout")
                    .total_bytes()
            );

            println!(
                "{{\"schema\":\"fmn.pg_a.v1\",\"case\":\"{case}\",\
                 \"format\":\"{}\",\"device\":{:?},\"unified_memory\":{},\
                 \"frames\":{MEASURED_FRAMES},\"median_ns\":{},\"fps\":{fps},\
                 \"upload_bytes\":{},\"readback_bytes\":{},\
                 \"threads_per_threadgroup\":{},\"max_threads_per_threadgroup\":{},\
                 \"thread_execution_width\":{}}}",
                match format {
                    PixelFormat::Rgba8 => "rgba8",
                    PixelFormat::Nv12 => "nv12",
                    PixelFormat::P010 => "p010",
                    _ => unreachable!(),
                },
                report.device,
                report.unified_memory,
                median.as_nanos(),
                report.upload_bytes,
                report.readback_bytes,
                report.threads_per_threadgroup,
                report.max_threads_per_threadgroup,
                report.thread_execution_width,
            );
        }
    }
}

#[cfg(feature = "metal")]
#[test]
#[ignore = "PG-A runs explicitly on the pinned Apple profile"]
fn metal_three_d_pg_a_profiles_glow_and_surface_work() {
    if !MetalRenderer::is_available() {
        return;
    }
    const MEASURED_FRAMES: usize = 33;
    MetalThreeDCorpus::new().with_job(|job| {
        let mut renderer = MetalRenderer::new()
            .expect("an available Metal device must compile the production shader");
        renderer
            .render_three_d_rgba8(job)
            .expect("PG-A 3D warmup frame");
        let mut elapsed = Vec::with_capacity(MEASURED_FRAMES);
        let mut last = None;
        for _ in 0..MEASURED_FRAMES {
            let report = renderer.render_three_d_rgba8(job).expect("PG-A 3D frame").1;
            assert!(report.raw_surface_reused);
            assert!(report.output_surface_reused);
            elapsed.push(report.elapsed);
            last = Some(report);
        }
        elapsed.sort_unstable();
        let median = elapsed[MEASURED_FRAMES / 2];
        let report = last.expect("the measurement count is nonzero");
        let fps = 1.0 / median.as_secs_f64();
        assert!(fps.is_finite() && fps > 0.0);
        assert!(report.upload_bytes > 0);
        assert_eq!(
            report.readback_bytes,
            FrameLayout::tight(PixelFormat::Rgba8, WIDTH, HEIGHT)
                .expect("PG-A 3D output layout")
                .total_bytes()
        );
        println!(
            "{{\"schema\":\"fmn.pg_a.v1\",\"case\":\"three_d_glow_surface\",\
             \"format\":\"rgba8\",\"device\":{:?},\"unified_memory\":{},\
             \"frames\":{MEASURED_FRAMES},\"median_ns\":{},\"fps\":{fps},\
             \"upload_bytes\":{},\"readback_bytes\":{},\
             \"threads_per_threadgroup\":{},\"max_threads_per_threadgroup\":{},\
             \"thread_execution_width\":{}}}",
            report.device,
            report.unified_memory,
            median.as_nanos(),
            report.upload_bytes,
            report.readback_bytes,
            report.threads_per_threadgroup,
            report.max_threads_per_threadgroup,
            report.thread_execution_width,
        );
    });
}

#[test]
fn the_corpus_is_bit_locked_across_the_certified_matrix() {
    let mut failures = Vec::new();
    for (name, stage) in corpus() {
        let frame = definition_frame(&stage);
        let doc = encode_frame(&frame).expect("the frame encodes into its canonical document");
        let png = canonical_png_panel(&frame).expect("the certified frame has a canonical PNG");
        assert!(
            doc.len() > 1000,
            "{name} encoded to {} bytes — that is not a frame",
            doc.len()
        );
        assert!(
            png.starts_with(b"\x89PNG\r\n\x1a\n"),
            "{name} did not encode as a PNG"
        );
        let decoded = fmn_codec::decode_png(&png, &fmn_codec::PngLimits::default())
            .expect("the canonical panel decodes through the owned codec");
        assert_eq!((decoded.width, decoded.height), (WIDTH, HEIGHT));
        let mut expected_rgba8 = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, WIDTH, HEIGHT)
                .expect("the corpus dimensions form a tight RGBA8 frame"),
        );
        fmn_frame::convert::rgba16f_to_rgba8(&frame, &mut expected_rgba8)
            .expect("the certified frame converts canonically");
        assert_eq!(
            decoded.rgba,
            expected_rgba8.plane(0),
            "{name} PNG did not preserve the canonical RGBA8 projection"
        );
        dump(name, &doc, &png).expect("the requested certified artifacts are written");
        // Every entry is checked before any is reported. A loop that panicked on
        // the first drift would hide the shape of a regression — one frame moving
        // is a bug in one kernel, three moving is a bug in the composite — and it
        // would write only the first `.actual` sidecar, which is the one a
        // reviewer needs least.
        if let Err(e) = store().check(name, &doc) {
            failures.push(e.to_string());
        }
        if let Err(e) = store().check(&format!("{name}.png"), &png) {
            failures.push(e.to_string());
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Write a frame's canonical document and PNG panel to `$FMN_DUMP_FRAMES`.
///
/// The bytes the golden locks are the bytes a reviewer needs when it moves, and
/// `.actual` sidecars only appear on drift. This makes both the renderer oracle
/// and its directly reviewable projection available on demand — for a Look
/// Gallery panel, a repro bundle, or an eyeball before a bless — without a
/// second, private path to the pixels.
fn dump(name: &str, doc: &[u8], png: &[u8]) -> std::io::Result<()> {
    let Ok(dir) = std::env::var("FMN_DUMP_FRAMES") else {
        return Ok(());
    };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{name}.frame")), doc)?;
    std::fs::write(dir.join(format!("{name}.png")), png)
}

#[test]
fn every_locked_frame_is_thread_count_invariant() {
    // PG-5, per commit: "bit-identical raw frames across runs, {1,4,16} threads".
    // Thread count is one of the things §16.7 declares *outside* the input
    // closure — "proven inert under §10.5" — and this is the proof.
    for (name, stage) in corpus() {
        let one = definition(&stage);
        for threads in [4usize, 16] {
            assert!(
                render(&stage, Tier::Scalar, threads) == one,
                "{name} moved at {threads} threads"
            );
        }
    }
}

#[test]
fn every_certified_frame_ignores_adaptive_and_forced_aa_selection() {
    // §10.4: classification, forced 2× and forced 4× are standard-quality
    // choices. A certified run executes the canonical analytic path under every
    // requested A/B policy and therefore produces the same raw-frame document.
    for (name, stage) in corpus() {
        let analytic = definition(&stage);
        for aa in [AaPolicy::Adaptive, AaPolicy::Ssaa2x, AaPolicy::Ssaa4x] {
            assert_eq!(
                render_with_aa(&stage, Tier::Scalar, 4, aa),
                analytic,
                "{name} moved under certified {aa:?}"
            );
        }
    }
}

#[test]
fn every_tier_reproduces_the_scalar_definition() {
    // §10.5: "within the certified engine the scalar path is the definition and
    // every SIMD tier must match it bit-for-bit."
    //
    // `Tier::ALL` contains the scalar oracle and exactly the tier selected by
    // this artifact's SUITE.lock-governed crate flags. This renders the whole
    // corpus through both; a tier whose arithmetic diverges fails here rather
    // than in a Look Gallery review. ADR-0010 names lane width as the remaining
    // risk; this is the tripwire it asked for.
    assert_eq!(
        Tier::ALL.first(),
        Some(&Tier::Scalar),
        "the scalar tier must be first — it is the definition, not a member"
    );
    for (name, stage) in corpus() {
        let scalar = definition(&stage);
        for &tier in Tier::ALL {
            assert!(
                render(&stage, tier, 1) == scalar,
                "{name} differs between the scalar definition and the {} tier",
                tier.name()
            );
        }
    }
}

#[test]
fn every_fast_tier_stays_inside_the_certified_corpus_budget() {
    // §10.1 and §16.3: the fast engine shares Lumen's semantics but not its
    // arithmetic width. Every artifact therefore runs both its scalar and
    // compiled fast routes against the certified reference, under the
    // versioned max/RMS bounds and the perceptual SSIM smoke alarm.
    //
    // Thread count is not part of an engine identity. Holding each fast route
    // byte-exact at {1,4,16} proves that scheduling cannot silently spend more
    // of the visual budget.
    for (name, stage) in corpus() {
        let certified = render_frame(&stage, EngineIdentity::certified(), 1, AaPolicy::Adaptive);
        for &tier in Tier::ALL {
            let identity = EngineIdentity {
                tier,
                ..EngineIdentity::fast()
            };
            let one = render_frame(&stage, identity, 1, AaPolicy::Adaptive);
            let measured = divergence(&certified, &one);
            assert!(
                measured.maximum <= FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
                "{name} {} max={} at {:?} (certified {}, fast {}) exceeds {}",
                tier.name(),
                measured.maximum,
                measured.maximum_at,
                measured.reference_at_maximum,
                measured.candidate_at_maximum,
                FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR
            );
            assert!(
                measured.rms <= FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
                "{name} {} rms={} exceeds {}",
                tier.name(),
                measured.rms,
                FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR
            );
            assert!(
                measured.ssim >= FAST_VISUAL_BUDGET_V1_MIN_SSIM,
                "{name} {} ssim={} is below {}",
                tier.name(),
                measured.ssim,
                FAST_VISUAL_BUDGET_V1_MIN_SSIM
            );
            for threads in [4usize, 16] {
                let parallel = render_frame(&stage, identity, threads, AaPolicy::Adaptive);
                assert!(
                    parallel.as_bytes() == one.as_bytes(),
                    "{name} {} fast output moved at {threads} threads",
                    tier.name()
                );
            }
        }
    }
}

#[test]
fn a_frame_is_reproducible_within_a_run() {
    // If this fails, the engine has in-process nondeterminism and every
    // cross-platform claim above it is meaningless.
    for (name, stage) in corpus() {
        assert!(
            definition(&stage) == definition(&stage),
            "{name} is unstable"
        );
    }
}

#[test]
fn the_corpus_actually_exercises_what_it_claims() {
    // A golden corpus that quietly stopped drawing half its content would keep
    // passing forever. Each entry is checked for the structure its name promises.
    let cfg = config();

    let mut plan = RenderPlan::new();
    plan.sync(&caps_and_joins(), 0)
        .expect("valid caps-and-joins fixture");
    assert_eq!(
        plan.shapes().instances().len(),
        15,
        "four joint settings x three severities, plus three round-capped ends"
    );
    let joints: std::collections::BTreeSet<i64> = plan
        .styles()
        .rows()
        .iter()
        .map(|s| s.joint_type.to_code() as i64)
        .collect();
    assert_eq!(joints.len(), 4, "not every joint setting is in the corpus");

    // Present in the style table is not the same as visible in the frame. If the
    // corners were too shallow or the stroke too thin, all four settings would
    // render inside the antialiasing band and the golden would be locking a
    // picture no joint setting could move — a fixture that passes forever and
    // guards nothing. So: force every joint to `Auto` (the round field the
    // distance function already produces) and require the frame to change.
    let all_joints = |joint: JointType| {
        let mut s = caps_and_joins();
        let mobs: Vec<Mob> = s.roots().to_vec();
        for m in mobs {
            if let Some(u) = s.uniforms_mut(m) {
                u.joint_type = joint;
            }
        }
        definition(&s)
    };
    assert_ne!(
        definition(&caps_and_joins()),
        all_joints(JointType::Auto),
        "no joint setting reaches a pixel: the cap/join corpus guards nothing"
    );

    // ADR-0012's ruling, at the frame level: `Auto` and `NoJoint` both render the
    // round join the distance field already produces, so `join_wedges` returns
    // nothing for either and the two must be **byte**-identical — not similar.
    // `fmn_render::stroke` asserts this per pixel on one corner; here it is the
    // whole corpus, including the wrap joins of twelve closed outlines.
    assert_eq!(
        all_joints(JointType::Auto),
        all_joints(JointType::NoJoint),
        "the round settings diverged, so the join machinery is not inert for them"
    );
    // And the two real overrides each move the picture, in different ways.
    assert_ne!(all_joints(JointType::Auto), all_joints(JointType::Bevel));
    assert_ne!(all_joints(JointType::Auto), all_joints(JointType::Miter));
    assert_ne!(all_joints(JointType::Bevel), all_joints(JointType::Miter));

    let mut plan = RenderPlan::new();
    plan.sync(&fills(), 0).expect("valid fills fixture");
    assert!(
        plan.styles()
            .rows()
            .iter()
            .any(|s| !fmn_render::fill::fill_is_flat(s)),
        "no gradient fill, so the interior field is untested"
    );
    assert!(
        plan.styles()
            .rows()
            .iter()
            .any(|s| s.fill_border_width > 0.0),
        "no inner border, so ADR-0011's band is untested"
    );
    // Measure the route the frame job actually takes, rather than treating
    // every retained shape hint as a specialized fill kernel. In particular, a
    // closed polyline is a useful durable classification for binning and
    // strokes, but `FillKernel::select` correctly sends its fill through the
    // general winding solver. fm-7if made this distinction observable by
    // preserving a translated rectangle's `Rect` hint instead of rewriting its
    // points and accidentally demoting it to `General`.
    let routes: Vec<_> = plan
        .shapes()
        .instances()
        .iter()
        .map(|instance| {
            let shape = plan
                .shapes()
                .shape(instance.shape)
                .expect("instance names a compiled shape");
            let first = shape.first_segment as usize;
            let end = first + shape.segment_count as usize;
            let segments = &plan.segments()[first..end];
            let kernel = if instance.hint_unsafe || !instance.placement.is_translation() {
                FillKernel::General
            } else {
                FillKernel::select(
                    shape,
                    segments,
                    cfg.map,
                    instance_translation(instance, cfg.map),
                )
            };
            (shape.hint, kernel)
        })
        .collect();
    let specialized = routes
        .iter()
        .filter(|(_, kernel)| !matches!(kernel, FillKernel::General))
        .count();
    assert!(
        specialized >= 2,
        "the specialized fill routes are not exercised: {routes:?}"
    );
    // And the general machinery, which is easy to lose: at this scale every
    // `Circle` is inside the hint budget. The closed-polyline pentagram and the
    // multi-subpath annulus must therefore both exercise §10.2's centrepiece.
    let general = routes
        .iter()
        .filter(|(_, kernel)| matches!(kernel, FillKernel::General))
        .count();
    assert!(
        general >= 2,
        "the general quadratic fill is barely exercised: {general} instances; \
         retained hint/kernel routes: {routes:?}"
    );

    let mut plan = RenderPlan::new();
    plan.sync(&composite(), 0).expect("valid composite fixture");
    assert!(
        plan.styles().rows().iter().any(|s| s.stroke_behind),
        "no backstroke, so R-5's other branch is untested"
    );
    assert!(
        plan.styles()
            .rows()
            .iter()
            .any(|s| s.stroke_width != s.stroke_width_end),
        "no width taper, so the arc-length ramp is untested"
    );
    // And every corpus frame must be binned somewhere: a scene that misses the
    // viewport renders a background nobody would notice was empty.
    for (name, stage) in corpus() {
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .expect("valid certified-engine corpus fixture");
        let mono = MonoTable::build(&plan, cfg.map).expect("bounded corpus monotone table");
        let binning = Binning::build(&plan, cfg.viewport, TILING, cfg.map)
            .expect("bounded certified-engine binning");
        assert!(!binning.draws().is_empty(), "{name} binned to nothing");
        assert!(!mono.is_empty(), "{name} compiled no fill geometry");
    }
}

#[test]
fn the_engine_identity_reaches_the_input_closure() {
    // §10.5(f): "every engine/backend identity is journaled into the input
    // closure". C7 is what that becomes, and this is the value it contributes.
    let cfg = config();
    let certified = journal_digest(EngineIdentity::certified(), &cfg, TILING);
    assert_eq!(
        certified,
        journal_digest(EngineIdentity::certified(), &cfg, TILING),
        "the identity digest is not stable"
    );
    assert!(EngineIdentity::certified().engine.certifiable());

    // The declared configuration is in it, so a run that changed the tile
    // dimensions cannot present a manifest claiming the same inputs.
    let other = Tiling {
        macro_tile: 128,
        fine_tile: 32,
    };
    assert_ne!(
        certified,
        journal_digest(EngineIdentity::certified(), &cfg, other)
    );

    // A frame's job reports the same value the free function does, so nothing
    // downstream can journal an identity the render did not use.
    let stage = fills();
    let mut plan = RenderPlan::new();
    plan.sync(&stage, 0).expect("valid fills fixture");
    let mono = MonoTable::build(&plan, cfg.map).expect("bounded fills monotone table");
    let binning = Binning::build(&plan, cfg.viewport, TILING, cfg.map)
        .expect("bounded certified-engine binning");
    let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
    assert_eq!(job.journal_digest(), certified);
    assert_eq!(job.identity(), EngineIdentity::certified());
}

#[test]
fn the_background_is_the_references_and_is_decoded_once() {
    // The corpus renders on #333333, and 0x33 is emphatically not 0.2 in linear
    // light. Getting this wrong is the failure mode BN-04's decode step exists
    // to prevent, and it would be invisible in a golden that had never been
    // right.
    let bg = config().background;
    let expected = fmn_core::color::srgb_eotf(f64::from(0x33u8) / 255.0);
    assert!((bg.r - expected).abs() < 1e-15);
    assert!(
        bg.r < 0.05,
        "0x33 decoded to {} — that is an encoded value, not linear light",
        bg.r
    );
    assert!(
        (bg.a - 1.0).abs() < 1e-15,
        "the corpus background is opaque"
    );
}
