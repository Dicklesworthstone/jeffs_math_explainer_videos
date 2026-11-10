//! The Look Gallery plane (§16.3 plane 3, fm-t1v): metric sanity, manifest
//! mechanics, verdict transitions, and the smoke-alarm demo over the real
//! captured pairs.
//!
//! The metrics are smoke alarms, never gates: the demo test reports numbers
//! and fails only on manifest corruption or a missing committed render. The
//! Reference captures are private §15.3 fixtures (gitignored); a checkout
//! without them simply skips the measurement and says so.

use fmn_conformance::gallery::{
    GalleryError, GalleryManifest, OwnerVerdicts, PairMetrics, RgbaView, Verdict,
    canonical_png_panel, compare_pair, edge_distance_luma, render_pairs,
};
use fmn_frame::{FrameBuffer, FrameError, FrameLayout, PixelFormat};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

// ------------------------------------------------------------ image helpers

/// Build a tight RGBA8 image from a per-pixel function.
fn image(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&f(x, y));
        }
    }
    pixels
}

/// A 96×96 dark field with a filled bright square at (`ox`, `oy`) — a shape
/// with real Sobel edges, so the edge metric has something to measure.
fn square_image(ox: u32, oy: u32) -> Vec<u8> {
    image(96, 96, |x, y| {
        let inside = (ox..ox + 40).contains(&x) && (oy..oy + 40).contains(&y);
        if inside {
            [200, 180, 160, 255]
        } else {
            [20, 24, 32, 255]
        }
    })
}

fn view(width: u32, height: u32, pixels: &[u8]) -> RgbaView<'_> {
    RgbaView::new(width, height, pixels).expect("the synthetic image is valid")
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("look_gallery_{name}"));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn committed_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/look_gallery.tsv")
}

// ----------------------------------------------------------- metric sanity

#[test]
fn identical_images_have_perfect_metrics() {
    let pixels = image(64, 64, |x, y| {
        [
            ((x * 3 + y) % 256) as u8,
            ((x + y * 5) % 256) as u8,
            ((x * 7 + y * 11) % 256) as u8,
            255,
        ]
    });
    let a = view(64, 64, &pixels);
    let b = view(64, 64, &pixels);
    let m: PairMetrics = compare_pair(&a, &b).expect("same dimensions");
    assert!(
        (m.ssim - 1.0).abs() < 1e-12,
        "identical images must score SSIM 1.0, got {}",
        m.ssim
    );
    assert!(
        m.edge.symmetric.abs() < 1e-12,
        "identical images must have zero edge distance, got {}",
        m.edge.symmetric
    );
    for value in [m.error.p50, m.error.p95, m.error.p99, m.error.max] {
        assert!(
            value.abs() < 1e-12,
            "identical images must have zero error at every percentile, got {value}"
        );
    }
}

#[test]
fn canonical_panel_refuses_a_non_certified_source_format() {
    let layout = FrameLayout::tight(PixelFormat::Rgba8, 2, 2).expect("valid small frame");
    let frame = FrameBuffer::new(layout);
    assert_eq!(
        canonical_png_panel(&frame),
        Err(FrameError::FormatMismatch {
            expected: "Rgba16F source",
            got: PixelFormat::Rgba8,
        })
    );
}

#[test]
fn shifted_edges_move_the_metrics_in_the_right_direction() {
    let base = square_image(10, 10);
    let near_pixels = square_image(14, 10);
    let far_pixels = square_image(30, 10);
    let reference = view(96, 96, &base);
    let near = view(96, 96, &near_pixels);
    let far = view(96, 96, &far_pixels);

    let near_metrics = compare_pair(&reference, &near).expect("same dimensions");
    let far_metrics = compare_pair(&reference, &far).expect("same dimensions");

    assert!(
        near_metrics.edge.symmetric > 0.0,
        "a shifted edge must have positive edge distance"
    );
    // A pure 4 px horizontal translation moves the square's vertical edges
    // by 4 px, but the horizontal edges partially overlap the original (36
    // of 40 px at distance 0), so the symmetric mean lands strictly below 4.
    assert!(
        near_metrics.edge.symmetric > 1.0 && near_metrics.edge.symmetric < 4.0,
        "a 4 px shift should read as single-digit edge distance, got {}",
        near_metrics.edge.symmetric
    );
    assert!(
        far_metrics.edge.symmetric > 2.0 * near_metrics.edge.symmetric,
        "a 20 px shift must outdistance a 4 px shift by a wide margin: {} vs {}",
        far_metrics.edge.symmetric,
        near_metrics.edge.symmetric
    );
    assert!(
        near_metrics.ssim < 1.0 && far_metrics.ssim < near_metrics.ssim,
        "SSIM must fall as the shift grows: near {}, far {}",
        near_metrics.ssim,
        far_metrics.ssim
    );
    // A 4 px shift touches ~3.5% of pixels (p95 still zero); a 20 px shift
    // touches ~17%, pushing the p95 off zero.
    assert!(
        far_metrics.error.p95 > near_metrics.error.p95,
        "the bigger shift touches more pixels, so the p95 error must rise: {} vs {}",
        far_metrics.error.p95,
        near_metrics.error.p95
    );
    // The red channel jumps 180 codes (200 vs 20): 180/255 ≈ 0.706.
    assert!(
        near_metrics.error.max > 0.7,
        "a 180-code channel jump must register near full scale, got {}",
        near_metrics.error.max
    );
}

#[test]
fn impulse_noise_moves_the_tail_not_the_median() {
    let base = square_image(10, 10);
    let mut noisy = base.clone();
    // One percent of pixels, deterministic positions, flipped to white.
    let total = 96 * 96;
    for i in 0..total {
        if i % 100 == 0 {
            noisy[i * 4] = 255;
            noisy[i * 4 + 1] = 255;
            noisy[i * 4 + 2] = 255;
        }
    }
    let reference = view(96, 96, &base);
    let candidate = view(96, 96, &noisy);
    let m = compare_pair(&reference, &candidate).expect("same dimensions");
    assert!(
        m.error.p50.abs() < 1e-12 && m.error.p95.abs() < 1e-12,
        "1% impulse noise must not move the median or the p95, got p50 {} p95 {}",
        m.error.p50,
        m.error.p95
    );
    // Nearest-rank p99 catches the smallest of the 93 nonzero errors: a flip
    // inside the square (a 95-code jump), well above the untouched p95.
    assert!(
        m.error.p99 > 0.3,
        "1% impulse noise must light up the p99 tail, got {}",
        m.error.p99
    );
    assert!(
        m.error.max > 0.9,
        "the darkest flipped pixel is 32 codes off white, got {}",
        m.error.max
    );
    assert!(
        m.ssim < 1.0,
        "impulse noise must register on SSIM, got {}",
        m.ssim
    );
}

#[test]
fn flat_images_have_no_edges_and_exact_error() {
    let dark = image(32, 32, |_, _| [0, 0, 0, 255]);
    let grey = image(32, 32, |_, _| [51, 51, 51, 255]);
    let m = compare_pair(&view(32, 32, &dark), &view(32, 32, &grey)).expect("same dimensions");
    assert_eq!(m.edge.reference_edges, 0);
    assert_eq!(m.edge.candidate_edges, 0);
    assert!(
        m.edge.symmetric.abs() < 1e-12,
        "two edge-free images have zero edge distance"
    );
    assert!(
        (m.error.max - 0.2).abs() < 1e-9,
        "a uniform 51-code jump is exactly 0.2 normalized, got {}",
        m.error.max
    );
}

#[test]
fn one_sided_edges_report_the_frame_diagonal() {
    let flat = image(96, 96, |_, _| [20, 24, 32, 255]);
    let shaped = square_image(10, 10);
    let m = compare_pair(&view(96, 96, &flat), &view(96, 96, &shaped)).expect("same dimensions");
    let diagonal = (96.0_f64 * 96.0 + 96.0 * 96.0).sqrt();
    assert!(
        m.edge.reference_to_candidate.abs() < 1e-12,
        "the edge-free reference contributes zero in its direction"
    );
    assert!(
        (m.edge.candidate_to_reference - diagonal).abs() < 1e-9,
        "edges with nothing to match read as the frame diagonal: {} vs {diagonal}",
        m.edge.candidate_to_reference
    );
}

#[test]
fn mismatched_dimensions_are_a_named_error() {
    let small = image(32, 32, |_, _| [0, 0, 0, 255]);
    let large = image(64, 64, |_, _| [0, 0, 0, 255]);
    let err = compare_pair(&view(32, 32, &small), &view(64, 64, &large))
        .expect_err("different sizes must refuse");
    assert!(
        err.to_string().contains("dimension mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn raw_luma_metrics_reject_inconsistent_geometry() {
    let panic_case = edge_distance_luma(&[0.0], &[0.0], 3, 3)
        .expect_err("short 3x3 planes must refuse before Sobel indexing");
    assert!(
        panic_case.to_string().contains("reference has 1 samples")
            && panic_case.to_string().contains("expected 9"),
        "unexpected panic-case refusal: {panic_case}"
    );

    let silent_case = edge_distance_luma(&[0.0], &[0.0], 2, 2)
        .expect_err("short sub-kernel planes must not alias a valid zero score");
    assert!(
        silent_case.to_string().contains("reference has 1 samples")
            && silent_case.to_string().contains("expected 4"),
        "unexpected silent-case refusal: {silent_case}"
    );

    let candidate_case = edge_distance_luma(&[0.0; 4], &[0.0], 2, 2)
        .expect_err("candidate geometry is independently bound");
    assert!(
        candidate_case
            .to_string()
            .contains("candidate has 1 samples"),
        "unexpected candidate refusal: {candidate_case}"
    );

    let exact = edge_distance_luma(&[0.0; 4], &[0.0; 4], 2, 2)
        .expect("exact raw-luma geometry remains measurable");
    assert_eq!(exact.reference_edges, 0);
    assert_eq!(exact.candidate_edges, 0);
    assert_eq!(exact.symmetric, 0.0);
}

#[test]
fn image_views_refuse_dimension_products_that_overflow_rgba8_length() {
    let error = RgbaView::new(1 << 31, 1 << 31, &[])
        .expect_err("overflowing dimensions must fail without inspecting a wrapped length");
    assert!(
        error
            .to_string()
            .contains("overflow the addressable RGBA8 byte length"),
        "unexpected error: {error}"
    );

    let error = RgbaView::new(2, 3, &[0; 23])
        .expect_err("ordinary wrong lengths must retain their named refusal");
    assert!(error.to_string().contains("expected 24"));

    assert!(RgbaView::new(2, 3, &[0; 24]).is_ok());
}

// ---------------------------------------------------------- manifest rules

const RELEASE: &str = "git:0123456789abcdef0123456789abcdef01234567";
const DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const COLUMNS: &str = "# columns: panel\tsource\treference\treference_sha256\trender\t\
     render_sha256\tbuild_id\tadvisory_verdict\tchanged";

/// A v2 prelude at `revision`.
fn prelude(revision: &str) -> String {
    format!(
        "# fmn-look-gallery v2\n# revision: {revision}\n# release: {RELEASE}\n\
         # reference: 3b1b/manim@6199a00d4c1b1127ebe45cb629c3f22538b10e13\n{COLUMNS}\n"
    )
}

/// A well-formed v2 row for `panel` with the given advisory verdict and note.
fn row(panel: &str, verdict: &str, note: &str) -> String {
    format!(
        "{panel}\tgallery/scenes/s.py:Scene\ta/{panel}.png\t{DIGEST}\tc/{panel}.png\t{DIGEST}\t\
         {RELEASE}\t{verdict}\t{note}\n"
    )
}

#[test]
fn committed_manifest_parses_and_round_trips_byte_for_byte() {
    let path = committed_manifest_path();
    let text = std::fs::read_to_string(&path).expect("committed manifest");
    let manifest = GalleryManifest::parse(&text).expect("the committed manifest parses");
    assert!(
        manifest.revision >= 4,
        "regeneration advanced the v1 revision 3"
    );
    assert_eq!(
        manifest.to_text().expect("canonical manifest serializes"),
        text,
        "the committed manifest must be in canonical form"
    );
    // fm-5wq.50 coverage: the seven primitives, a 50-formula math sheet in two
    // parts, and twenty README-class scenes, each from a committed scene file.
    let count = |prefix: &str| {
        manifest
            .rows
            .iter()
            .filter(|row| row.panel.starts_with(prefix))
            .count()
    };
    assert_eq!(count("math."), 2, "{:?}", manifest.rows);
    assert_eq!(count("readme."), 20, "{:?}", manifest.rows);
    assert_eq!(manifest.rows.len(), 29);
    for name in [
        "glow",
        "gradient_fills",
        "joints_and_caps",
        "lighting_3d",
        "math_formula",
        "self_intersections",
        "text_sample",
    ] {
        let row = manifest.row(name).expect("every primitive panel is kept");
        assert!(row.source.starts_with("gallery/scenes/primitives.py:"));
    }
    for row in &manifest.rows {
        let (file, _) = row.source.split_once(':').expect("source names a scene");
        assert!(
            repo_root().join(file).is_file(),
            "{} names a missing scene file",
            row.panel
        );
    }
}

#[test]
fn every_committed_panel_is_this_release_and_unaltered() {
    let manifest = GalleryManifest::load(&committed_manifest_path()).expect("manifest parses");
    let pairs =
        render_pairs(&manifest, &repo_root()).expect("panels match the release under review");
    assert_eq!(pairs.len(), manifest.rows.len());
}

#[test]
fn a_planted_stale_or_altered_panel_is_refused() {
    let manifest = GalleryManifest::load(&committed_manifest_path()).expect("manifest parses");
    let mut stale = manifest.clone();
    stale.rows[0].build_id = "git:ffffffffffffffffffffffffffffffffffffffff".to_string();
    let error = render_pairs(&stale, &repo_root()).expect_err("a stale panel must refuse");
    assert!(
        matches!(&error, GalleryError::StalePanel { panel, .. } if *panel == stale.rows[0].panel),
        "{error}"
    );

    let mut altered = manifest.clone();
    altered.rows[1].render_sha256 = DIGEST.to_string();
    let error = render_pairs(&altered, &repo_root()).expect_err("an altered render must refuse");
    assert!(
        matches!(&error, GalleryError::DigestMismatch { panel, .. } if *panel == altered.rows[1].panel),
        "{error}"
    );
}

#[test]
fn corrupt_manifests_are_named_errors() {
    let with_row = |line: String| format!("{}{line}", prelude("1"));
    let mut swapped = row("panel", "regression", "note");
    swapped = swapped.replacen(DIGEST, "not-a-digest", 2);
    let cases: Vec<(String, &str)> = vec![
        (String::new(), "empty manifest"),
        ("# fmn-look-gallery v2\n".to_string(), "revision"),
        (
            "# fmn-look-gallery v1\n# revision: 1\n".to_string(),
            "first line must be",
        ),
        (
            "# fmn-look-gallery v2\n# revision: soon\n".to_string(),
            "not a non-negative integer",
        ),
        (
            "# fmn-look-gallery v2\n# revision: 1\n# columns: x\n".to_string(),
            "must start with",
        ),
        (
            "# fmn-look-gallery v2\n# revision: 1\n# release: has space\n".to_string(),
            "invalid release identity",
        ),
        (
            with_row("panel\tonly-two\n".to_string()),
            "expected 9 tab-separated fields",
        ),
        (
            with_row(row("Panel", "regression", "note")),
            "invalid panel id",
        ),
        (
            with_row(row("panel", "regression", "note").replace("a/panel.png", "../escape.png")),
            "invalid reference path",
        ),
        (
            with_row(row("panel", "looks-good", "note")),
            "unknown verdict",
        ),
        (
            with_row(
                row("panel", "regression", "note").replace("gallery/scenes/s.py:Scene", "s.py"),
            ),
            "invalid source",
        ),
        (with_row(swapped), "invalid reference_sha256"),
        (
            with_row(row("panel", "regression", "note").replace(RELEASE, "git dirty")),
            "invalid build_id",
        ),
        (
            with_row(row("panel", "at-least-as-good", "note").replacen(
                &format!("\t{DIGEST}\tc/"),
                "\t-\tc/",
                1,
            )),
            "records no Reference capture",
        ),
    ];
    for (text, needle) in cases {
        let err = GalleryManifest::parse(&text).expect_err("corrupt input must refuse");
        assert!(
            err.to_string().contains(needle),
            "expected {needle:?} in: {err}"
        );
    }
    let duplicate = format!(
        "{}{}{}",
        prelude("1"),
        row("panel", "regression", "note"),
        row("panel", "regression", "note")
    );
    let err = GalleryManifest::parse(&duplicate).expect_err("duplicate panel must refuse");
    assert!(err.to_string().contains("duplicate panel"), "{err}");
    let empty_note = format!("{}{}", prelude("1"), row("panel", "regression", ""));
    let err = GalleryManifest::parse(&empty_note).expect_err("empty change note must refuse");
    assert!(err.to_string().contains("empty change note"), "{err}");

    let duplicate_revision = format!("{}# revision: 2\n", prelude("1"));
    let err = GalleryManifest::parse(&duplicate_revision)
        .expect_err("a manifest must have one authoritative revision line");
    assert!(
        matches!(
            err,
            GalleryError::Corrupt { line: 6, ref detail }
                if detail.contains("duplicate revision line")
        ),
        "unexpected duplicate-revision refusal: {err}"
    );

    let mut over_fielded = format!("{}{}", prelude("1"), row("panel", "regression", "note"));
    over_fielded.pop();
    over_fielded.extend(std::iter::repeat_n('\t', 1_000_000));
    over_fielded.push('\n');
    let err = GalleryManifest::parse(&over_fielded)
        .expect_err("a delimiter-heavy malformed row must be refused");
    assert!(
        err.to_string()
            .contains("expected 9 tab-separated fields, found 1000009"),
        "unexpected over-fielded refusal: {err}"
    );
    assert!(
        err.to_string().len() < 200,
        "the refusal must not copy the malformed row into its diagnostic"
    );
}

#[test]
fn noncanonical_manifest_spellings_are_refused() {
    let canonical = format!("{}{}", prelude("1"), row("alpha", "unreviewed", "note"));
    GalleryManifest::parse(&canonical).expect("canonical spelling parses");

    let cases = [
        (
            format!(
                "{}{}",
                prelude("1").replace(&format!("{COLUMNS}\n"), ""),
                row("alpha", "unreviewed", "note")
            ),
            "fifth line must be",
        ),
        (prelude("01"), "canonical unsigned-decimal"),
        (prelude("1 "), "not a non-negative integer"),
        (prelude("1").replace('\n', "\r\n"), "carriage returns"),
        (prelude("1").trim_end_matches('\n').to_string(), "final LF"),
        (format!("{}\n", prelude("1")), "blank lines"),
        (
            format!("{}# extra metadata\n", prelude("1")),
            "unexpected comment line",
        ),
        (
            format!(
                "{}{}{}",
                prelude("1"),
                row("beta", "unreviewed", "note"),
                row("alpha", "unreviewed", "note")
            ),
            "out of canonical order",
        ),
    ];
    for (text, needle) in cases {
        let error = GalleryManifest::parse(&text).expect_err("noncanonical spelling must refuse");
        assert!(
            error.to_string().contains(needle),
            "expected {needle:?} in: {error}"
        );
    }
}

#[test]
fn oversized_manifest_documents_are_refused_by_parse_and_load() {
    let oversized = "x".repeat(1024 * 1024 + 1);
    let parse_error = GalleryManifest::parse(&oversized)
        .expect_err("an oversized in-memory manifest must be refused");
    assert!(
        matches!(
            parse_error,
            GalleryError::ManifestTooLarge { limit: 1_048_576 }
        ),
        "unexpected parse refusal: {parse_error}"
    );

    let path = scratch("oversized_manifest").join("look_gallery.tsv");
    std::fs::write(&path, oversized.as_bytes()).expect("write oversized manifest fixture");
    let load_error = GalleryManifest::load(&path)
        .expect_err("an oversized file-backed manifest must be refused");
    assert!(
        matches!(
            load_error,
            GalleryError::ManifestTooLarge { limit: 1_048_576 }
        ),
        "unexpected load refusal: {load_error}"
    );

    let valid = format!(
        "{}{}",
        prelude("1"),
        row("panel", "at-least-as-good", "note")
    );
    let mut updated = GalleryManifest::parse(&valid).expect("valid manifest");
    let before = updated.clone();
    let update_error = updated
        .record_verdict("panel", Verdict::Regression, &oversized)
        .expect_err("an oversized verdict update must be refused");
    assert!(matches!(
        update_error,
        GalleryError::ManifestTooLarge { limit: 1_048_576 }
    ));
    assert_eq!(updated, before, "a refused update must be atomic");

    updated.rows[0].changed = oversized;
    let refused_path = scratch("oversized_save").join("look_gallery.tsv");
    let save_error = updated
        .save(&refused_path)
        .expect_err("an oversized in-memory manifest must not be written");
    assert!(matches!(
        save_error,
        GalleryError::ManifestTooLarge { limit: 1_048_576 }
    ));
    assert!(
        !refused_path.exists(),
        "size validation must precede output-file creation"
    );
}

// ------------------------------------------------------- verdict workflow

#[test]
fn verdict_transitions_are_recorded_and_regressions_found() {
    let baseline = GalleryManifest::parse(&format!(
        "{}{}{}",
        prelude("4"),
        row("glow", "at-least-as-good", "seed"),
        row("gradient_fills", "different-but-fine", "BN-06 seed")
    ))
    .expect("parses");
    let mut current = baseline.clone();

    // A deliberate worsening, with its reason — the review act.
    let change = current
        .record_verdict(
            "glow",
            Verdict::Regression,
            "fm-t1v demo: glow falloff visibly clipped",
        )
        .expect("known panel");
    assert_eq!(change.from, Some(Verdict::AtLeastAsGood));
    assert_eq!(change.to, Verdict::Regression);
    assert_eq!(current.revision, baseline.revision + 1);

    // An improvement alone is never a regression.
    let mut improved = baseline.clone();
    improved
        .record_verdict(
            "gradient_fills",
            Verdict::AtLeastAsGood,
            "fm-t1v demo: at-least-as-good",
        )
        .expect("known panel");
    assert!(improved.regressions_since(&baseline).is_empty());

    // Both movements in one span: only the worsening is reported.
    current
        .record_verdict(
            "gradient_fills",
            Verdict::AtLeastAsGood,
            "fm-t1v demo: at-least-as-good",
        )
        .expect("known panel");
    let regressions = current.regressions_since(&baseline);
    assert_eq!(
        regressions.len(),
        1,
        "only the worsened panel is a regression: {regressions:?}"
    );
    assert_eq!(regressions[0].panel, "glow");
    assert_eq!(regressions[0].from, Some(Verdict::AtLeastAsGood));
    assert_eq!(regressions[0].to, Verdict::Regression);
    let backwards = baseline.regressions_since(&current);
    assert_eq!(backwards.len(), 1);
    assert_eq!(backwards[0].panel, "gradient_fills");

    // A reset to unreviewed (a regenerated render) is not a regression either way.
    let mut reset = baseline.clone();
    reset
        .record_verdict("glow", Verdict::Unreviewed, "fm-5wq.50 demo: re-rendered")
        .expect("known panel");
    assert!(reset.regressions_since(&baseline).is_empty());
    assert!(baseline.regressions_since(&reset).is_empty());

    // Persistence: save → load is lossless and the TSV round-trips.
    let path = scratch("verdicts").join("look_gallery.tsv");
    current.save(&path).expect("save");
    let reloaded = GalleryManifest::load(&path).expect("reload");
    assert_eq!(reloaded, current);
    assert_eq!(
        std::fs::read_to_string(&path).expect("file bytes"),
        current.to_text().expect("canonical manifest serializes"),
        "the file holds exactly the canonical text"
    );

    let err = current
        .record_verdict("no_such_panel", Verdict::Regression, "note")
        .expect_err("unknown panel must refuse");
    assert!(err.to_string().contains("unknown gallery panel"), "{err}");
    let err = current
        .record_verdict("glow", Verdict::Regression, "tab\tin note")
        .expect_err("a tab would corrupt the TSV");
    assert!(err.to_string().contains("tabs or newlines"), "{err}");
}

#[test]
fn verdict_updates_refuse_revision_rollover_without_partial_mutation() {
    let text = format!(
        "{}{}",
        prelude("18446744073709551615"),
        row("panel", "at-least-as-good", "original note")
    );
    let mut manifest = GalleryManifest::parse(&text).expect("maximum u64 revision is valid");
    let before = manifest.clone();
    let error = manifest
        .record_verdict("panel", Verdict::Regression, "replacement note")
        .expect_err("the monotone revision must not roll over");
    assert!(matches!(error, GalleryError::RevisionOverflow));
    assert_eq!(
        manifest, before,
        "a refused revision advance must not mutate the verdict or note"
    );
    let error = manifest
        .record_verdict("missing", Verdict::Regression, "replacement note")
        .expect_err("unknown-panel refusal retains precedence");
    assert!(matches!(error, GalleryError::UnknownPanel(panel) if panel == "missing"));
    assert_eq!(manifest, before);
}

#[test]
fn regressions_since_flags_new_regression_panels_only() {
    let parse = |rows: String| {
        GalleryManifest::parse(&format!("{}{rows}", prelude("1"))).expect("well-formed")
    };
    let earlier = parse(row("alpha", "at-least-as-good", "note"));
    let with_review_item = parse(
        row("alpha", "at-least-as-good", "note") + &row("beta", "different-but-fine", "note"),
    );
    assert!(with_review_item.regressions_since(&earlier).is_empty());
    let with_regression =
        parse(row("alpha", "at-least-as-good", "note") + &row("beta", "regression", "note"));
    let found = with_regression.regressions_since(&earlier);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].panel, "beta");
    assert_eq!(found[0].from, None);
}

// --------------------------------------------------- the owner verdict lane

#[test]
fn the_owner_ledger_is_committed_and_binds_verdicts_to_render_digests() {
    let ledger = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/look_gallery_owner_verdicts.tsv"),
    )
    .expect("the owner ledger is committed");
    let owner = OwnerVerdicts::parse(&ledger).expect("the committed owner ledger parses");
    let manifest = GalleryManifest::load(&committed_manifest_path()).expect("manifest parses");
    // Every committed owner verdict names a panel of the gallery.
    for verdict in &owner.rows {
        assert!(
            manifest.row(&verdict.panel).is_some(),
            "{} is not a panel",
            verdict.panel
        );
    }

    let header = "# fmn-look-gallery-owner-verdicts v1\n# columns: panel\trender_sha256\tverdict\tnote\tdate\n";
    let glow = manifest.row("glow").expect("glow panel");
    let current = format!(
        "glow\t{}\tat-least-as-good\tsigned off\t2026-10-05\n",
        glow.render_sha256
    );
    let earlier = format!("glow\t{DIGEST}\tregression\tolder render\t2026-08-01\n");
    let owner = OwnerVerdicts::parse(&format!("{header}{earlier}{current}")).expect("well-formed");
    let verdicts = owner.current(&manifest);
    assert_eq!(verdicts.get("glow"), Some(&Verdict::AtLeastAsGood));
    // A verdict on another render is history, not a verdict on this release.
    let only_old = OwnerVerdicts::parse(&format!("{header}{earlier}")).expect("well-formed");
    assert!(only_old.current(&manifest).is_empty());
}

#[test]
fn the_owner_lane_accepts_only_its_vocabulary() {
    let header = "# fmn-look-gallery-owner-verdicts v1\n# columns: panel\trender_sha256\tverdict\tnote\tdate\n";
    let line = |verdict: &str, note: &str, date: &str| {
        format!("{header}glow\t{DIGEST}\t{verdict}\t{note}\t{date}\n")
    };
    for verdict in ["at-least-as-good", "regression"] {
        OwnerVerdicts::parse(&line(verdict, "seen", "2026-10-05")).expect("a judgment");
    }
    OwnerVerdicts::parse(&line("different-but-fine", "BN-05 face", "2026-10-05"))
        .expect("BN-noted");
    for (text, needle) in [
        (
            line("unreviewed", "seen", "2026-10-05"),
            "is not at-least-as-good",
        ),
        (
            line("reference-capture-missing", "seen", "2026-10-05"),
            "is not at-least-as-good",
        ),
        (
            line("looks-good", "seen", "2026-10-05"),
            "is not at-least-as-good",
        ),
        (
            line("different-but-fine", "fine by me", "2026-10-05"),
            "must name its Behavior Note",
        ),
        (line("regression", "", "2026-10-05"), "needs a note"),
        (line("regression", "seen", "10/05/2026"), "invalid date"),
        (
            format!("{header}glow\tabc\tregression\tseen\t2026-10-05\n"),
            "invalid render_sha256",
        ),
        (
            format!("# fmn-look-gallery-owner-verdicts v9\n{}", &header[37..]),
            "line 1 must be",
        ),
    ] {
        let error = OwnerVerdicts::parse(&text).expect_err("outside the owner vocabulary");
        assert!(
            error.to_string().contains(needle),
            "expected {needle:?} in: {error}"
        );
    }
}

// ------------------------------------------------------------ the demo run

/// Maximum compressed size of one 1920×1080 Look Gallery image.
///
/// Sixteen MiB is roughly twice a tight RGBA8 frame, leaving ample room for
/// PNG framing and incompressible image data while keeping this file-backed
/// smoke-alarm input inside an explicit allocation envelope.
const MAX_GALLERY_PNG_BYTES: u64 = 16 * 1024 * 1024;

fn read_gallery_png_bytes(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_GALLERY_PNG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("could not read gallery PNG: {error}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_GALLERY_PNG_BYTES {
        return Err(format!(
            "gallery PNG exceeds the {MAX_GALLERY_PNG_BYTES}-byte compressed-input limit"
        ));
    }
    Ok(bytes)
}

fn read_gallery_png(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("could not inspect gallery PNG {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "gallery PNG {} is not a regular file",
            path.display()
        ));
    }
    if metadata.len() > MAX_GALLERY_PNG_BYTES {
        return Err(format!(
            "gallery PNG {} exceeds the {MAX_GALLERY_PNG_BYTES}-byte compressed-input limit",
            path.display()
        ));
    }
    let file = File::open(path)
        .map_err(|error| format!("could not open gallery PNG {}: {error}", path.display()))?;
    read_gallery_png_bytes(file).map_err(|error| format!("{}: {error}", path.display()))
}

#[test]
fn gallery_png_reader_accepts_exact_limit_and_refuses_the_next_byte() {
    let exact = read_gallery_png_bytes(std::io::repeat(0).take(MAX_GALLERY_PNG_BYTES))
        .expect("the exact compressed-input limit is accepted");
    assert_eq!(
        u64::try_from(exact.len()).expect("the bounded length fits u64"),
        MAX_GALLERY_PNG_BYTES
    );
    drop(exact);

    let error = read_gallery_png_bytes(std::io::repeat(0).take(MAX_GALLERY_PNG_BYTES + 1))
        .expect_err("one byte beyond the compressed-input limit must refuse");
    assert_eq!(
        error,
        format!("gallery PNG exceeds the {MAX_GALLERY_PNG_BYTES}-byte compressed-input limit")
    );
}

/// Decode a bounded PNG via the dev-dependency codec into an RGBA8 buffer.
fn load_rgba(path: &Path) -> (u32, u32, Vec<u8>) {
    let bytes = read_gallery_png(path).expect("read bounded gallery image");
    let decoded = fmn_codec::decode_png(&bytes, &fmn_codec::PngLimits::default())
        .expect("the gallery images are valid PNGs");
    (decoded.width, decoded.height, decoded.rgba)
}

/// The smoke-alarm demo over the real captured pairs: every committed render
/// named by the manifest must exist (a missing pair fails), and where the
/// private Reference captures are present the three metrics are computed and
/// reported — never asserted.
#[test]
fn smoke_alarm_over_the_real_pairs() {
    let manifest = GalleryManifest::load(&committed_manifest_path()).expect("manifest parses");
    let pairs = render_pairs(&manifest, &repo_root()).expect("no missing committed renders");
    assert_eq!(
        pairs.len(),
        manifest.rows.len(),
        "every manifest panel resolves to its committed render"
    );

    let measurable: Vec<_> = pairs.iter().filter(|p| p.reference_present).collect();
    // The Reference captures are private §15.3 fixtures
    // (gallery/reference_captures/ is gitignored): name every absent one.
    for pair in pairs.iter().filter(|p| !p.reference_present) {
        fmn_core::test_inputs::skip_or_fail(
            &format!(
                "fmn-conformance::smoke_alarm_over_the_real_pairs[{}]",
                pair.panel
            ),
            &format!("Reference capture {}", pair.reference.display()),
        );
    }
    if measurable.is_empty() {
        return;
    }

    eprintln!("look gallery smoke alarm (informational; thresholds are verdict inputs):");
    eprintln!(
        "{panel:<20} {ssim:>12} {edge:>10} {p50:>8} {p95:>8} {p99:>8} {max:>8}",
        panel = "panel",
        ssim = "ssim",
        edge = "edge-px",
        p50 = "p50",
        p95 = "p95",
        p99 = "p99",
        max = "max"
    );
    for pair in &measurable {
        let (rw, rh, reference_pixels) = load_rgba(&pair.reference);
        let (cw, ch, candidate_pixels) = load_rgba(&pair.render);
        let reference = RgbaView::new(rw, rh, &reference_pixels).expect("reference view");
        let candidate = RgbaView::new(cw, ch, &candidate_pixels).expect("candidate view");
        let m = compare_pair(&reference, &candidate)
            .expect("captures and renders share the capture resolution");
        eprintln!(
            "{panel:<20} {ssim:>12.9} {edge:>10.3} {p50:>8.5} {p95:>8.5} {p99:>8.5} {max:>8.5}",
            panel = pair.panel,
            ssim = m.ssim,
            edge = m.edge.symmetric,
            p50 = m.error.p50,
            p95 = m.error.p95,
            p99 = m.error.p99,
            max = m.error.max
        );
    }
}
