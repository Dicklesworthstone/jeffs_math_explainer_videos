//! The Look Gallery (§16.3 plane 3, fm-t1v): perceptual smoke alarms over
//! captured Reference/FrankenManim image pairs, plus the file-backed manifest
//! that records the human verdict on each pair.
//!
//! Two halves, deliberately separated:
//!
//! 1. **Metrics — smoke alarms, never hard gates.** [`compare_pair`] computes
//!    three documented numbers over an RGBA8 image pair:
//!
//!    - **Global SSIM** ([`PairMetrics::ssim`]): the single-statistic SSIM of
//!      Wang et al. over the Rec. 709 luma plane (`0.2126 R + 0.7152 G +
//!      0.0722 B`, computed on the sRGB8 codes exactly as the engine-equivalence
//!      lane's `ssim_luma` does after canonical RGBA8 encoding — the formula
//!      family `engine.rs`'s `FAST_VISUAL_BUDGET_V1_MIN_SSIM` and
//!      `metal.rs`'s `METAL_VISUAL_BUDGET_V1_MIN_SSIM` consume). Global means,
//!      sample variances and covariance (divisor `max(n−1, 1)`), stability
//!      constants `C1 = (0.01·255)²`, `C2 = (0.03·255)²`. The *global* form
//!      was chosen by the accelerator spike as a stable smoke alarm in place
//!      of an unreviewed windowed metric; this module keeps that ruling.
//!    - **Edge distance** ([`EdgeDistance`]): symmetric chamfer distance, in
//!      pixels, between thresholded Sobel edge maps — the boring standard
//!      boundary metric. Edges are pixels whose 3×3 Sobel L1 gradient
//!      magnitude on the luma plane reaches [`EDGE_L1_THRESHOLD`]; distances
//!      come from the Borgefors 3-4 two-pass chamfer transform (3 orthogonal,
//!      4 diagonal, reported divided by 3). Each directed score is the mean
//!      distance from one side's edge pixels to the other side's nearest edge
//!      pixel; [`EdgeDistance::symmetric`] is their mean. Conventions: no
//!      edge pixels on either side → 0; one side empty → that directed score
//!      is the frame diagonal (the worst possible) and the empty→full
//!      direction is 0.
//!    - **Local-error percentiles** ([`ErrorPercentiles`]): per-pixel error =
//!      max `|Δ|` over the R, G and B channels, normalized to `[0, 1]` by
//!      ÷255 (alpha is excluded: both pipelines composite it to identity on
//!      these stills, and the metric exists to see *colour* drift). p50/p95/
//!      p99 by the nearest-rank method on the sorted errors, plus the maximum.
//!
//!    Each returns numbers in a struct. Thresholds are verdict *inputs* for a
//!    human reviewer, never assertions — per D-16 the Reference is a design
//!    oracle and aesthetic bar, never a pixel warden.
//!
//! 2. **The verdict workflow.** [`Verdict`] is §16.3's vocabulary —
//!    `AtLeastAsGood`, `DifferentButFine` (Behavior-Noted), `Regression` — and
//!    [`GalleryManifest`] is its ledger: a versioned TSV artifact
//!    (`fixtures/look_gallery.tsv`, format `# fmn-look-gallery v2`) written by
//!    `scripts/regenerate_look_gallery.py` (fm-5wq.50). It names the release
//!    under review once, and pairs each panel's scene source
//!    (`gallery/scenes/`) with its committed render (`gallery/renders/`), the
//!    render's SHA-256, the build id that produced it, and the digest of the
//!    private Reference capture of the same source (`gallery/reference_captures/`,
//!    §15.3). The manifest's verdict column is **advisory** (agent review); a
//!    row may carry `unreviewed` or the `reference-capture-missing` refusal,
//!    neither an aesthetic verdict. Gates cite only the owner's verdicts
//!    ([`OwnerVerdicts`], `fixtures/look_gallery_owner_verdicts.tsv`), each
//!    bound to the render digest it judged. The review tooling:
//!    [`render_pairs`] resolves a manifest against a checkout and refuses a
//!    missing or altered render, or one built by anything but the release
//!    under review; [`GalleryManifest::record_verdict`] moves one panel's
//!    advisory verdict with its reason and bumps the revision; and
//!    [`GalleryManifest::regressions_since`] diffs two revisions for panels
//!    whose verdict worsened.
//!
//! [`canonical_png_panel`] is the deliberate bridge from certified Lumen
//! frames to this review plane. It applies fmn-frame's bit-exact canonical
//! transfer and fmn-codec's owned deterministic PNG encoder, so the panel a
//! human reviews is itself a certified, lockable artifact rather than output
//! from a throwaway decoder.

use fmn_codec::{CompressionLevel, encode_rgba8};
use fmn_frame::convert::rgba16f_to_rgba8;
use fmn_frame::{FrameBuffer, FrameError, FrameLayout, PixelFormat};
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic counter making concurrent tmp-file names unique (same pattern as
/// the self-golden rig: `cargo test` is parallel, tmp-and-rename must not
/// collide within the process).
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The manifest format tag; the first line of every look-gallery TSV.
pub const MANIFEST_HEADER: &str = "# fmn-look-gallery v2";

const MANIFEST_REVISION_PREFIX: &str = "# revision: ";
const MANIFEST_RELEASE_PREFIX: &str = "# release: ";
const MANIFEST_REFERENCE_PREFIX: &str = "# reference: ";
const MANIFEST_COLUMNS: &str = "# columns: panel\tsource\treference\treference_sha256\trender\t\
     render_sha256\tbuild_id\tadvisory_verdict\tchanged";
const MANIFEST_FIELDS: usize = 9;
/// The `reference_sha256` spelling of "no private capture recorded".
const NO_CAPTURE: &str = "-";

/// The owner-verdict ledger's format tag and column legend.
pub const OWNER_HEADER: &str = "# fmn-look-gallery-owner-verdicts v1";
const OWNER_COLUMNS: &str = "# columns: panel\trender_sha256\tverdict\tnote\tdate";

/// Maximum byte length of one look-gallery manifest. The ledger is a small,
/// reviewed TSV; one MiB leaves ample growth room while keeping malformed
/// file and in-memory inputs bounded before row ownership begins.
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;

/// Sobel L1 gradient magnitude at or above which a luma pixel is an edge.
///
/// The 3×3 Sobel kernel's maximum L1 response is `4·255` (a perfect step); a
/// quarter of that is a full-scale step over roughly eight pixels, which
/// rejects smooth shading ramps (lighting, glow falloff) while keeping every
/// silhouette, stroke boundary and glyph stem. Documented, fixed, and
/// deliberately not Otsu: a data-dependent threshold would make the metric
/// incomparable across revisions of the same panel.
pub const EDGE_L1_THRESHOLD: f64 = 255.0;

/// A borrowed tight-row RGBA8 image: `width × height × 4` bytes, row 0 on top
/// (the D-23 output orientation fmn-codec's `DecodedPng` normalizes to).
#[derive(Clone, Copy, Debug)]
pub struct RgbaView<'a> {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width × height × 4` bytes of RGBA, row-major, tight rows.
    pub pixels: &'a [u8],
}

impl<'a> RgbaView<'a> {
    /// Wrap a buffer, refusing zero dimensions and wrong lengths.
    ///
    /// # Errors
    /// [`GalleryError::InvalidImage`] if either dimension is zero or the
    /// buffer is not exactly `width × height × 4` bytes.
    pub fn new(width: u32, height: u32, pixels: &'a [u8]) -> Result<Self, GalleryError> {
        if width == 0 || height == 0 {
            return Err(GalleryError::InvalidImage(format!(
                "zero dimension ({width}x{height})"
            )));
        }
        let expected = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|pixels| pixels.checked_mul(4))
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or_else(|| {
                GalleryError::InvalidImage(format!(
                    "dimensions {width}x{height} overflow the addressable RGBA8 byte length"
                ))
            })?;
        if pixels.len() != expected {
            return Err(GalleryError::InvalidImage(format!(
                "{} bytes for {width}x{height} RGBA8, expected {expected}",
                pixels.len()
            )));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
}

/// Encode one certified linear-light frame as the canonical Look Gallery PNG.
///
/// The conversion is the certified `Rgba16F -> Rgba8` table lookup and the
/// encoder uses the owned deterministic best-compression route. The resulting
/// bytes contain no timestamps and are identical across thread counts and the
/// certified platform matrix when the input frame is identical.
///
/// # Errors
/// [`FrameError`] if `frame` is not `Rgba16F` or its dimensions cannot form a
/// valid tight `Rgba8` layout. No alternate conversion is selected silently.
pub fn canonical_png_panel(frame: &FrameBuffer) -> Result<Vec<u8>, FrameError> {
    let width = frame.layout().width();
    let height = frame.layout().height();
    let layout = FrameLayout::tight(PixelFormat::Rgba8, width, height)?;
    let mut rgba8 = FrameBuffer::new(layout);
    rgba16f_to_rgba8(frame, &mut rgba8)?;
    Ok(encode_rgba8(
        width,
        height,
        rgba8.plane(0),
        CompressionLevel::Best,
    ))
}

/// The three smoke-alarm numbers for one reference/candidate pair.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PairMetrics {
    /// Global luma SSIM in `[-1, 1]`; `1.0` iff the luma planes are identical.
    pub ssim: f64,
    /// Symmetric chamfer distance between thresholded Sobel edge maps.
    pub edge: EdgeDistance,
    /// Nearest-rank percentiles of the per-pixel max RGB channel error.
    pub error: ErrorPercentiles,
}

/// Boundary disagreement between two images, in pixels. See the module docs
/// for the exact definition and the empty-edge-map conventions.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct EdgeDistance {
    /// Edge pixels found in the reference image.
    pub reference_edges: u64,
    /// Edge pixels found in the candidate image.
    pub candidate_edges: u64,
    /// Mean distance from reference edge pixels to the nearest candidate edge.
    pub reference_to_candidate: f64,
    /// Mean distance from candidate edge pixels to the nearest reference edge.
    pub candidate_to_reference: f64,
    /// Mean of the two directed scores.
    pub symmetric: f64,
}

/// Nearest-rank percentiles of the per-pixel error distribution, plus the max.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ErrorPercentiles {
    /// Median per-pixel max-RGB-channel error, in `[0, 1]`.
    pub p50: f64,
    /// 95th percentile.
    pub p95: f64,
    /// 99th percentile.
    pub p99: f64,
    /// Largest per-pixel error.
    pub max: f64,
}

/// §16.3's verdict vocabulary for one gallery pair, plus the explicit
/// `ReferenceCaptureMissing` refusal used when no comparison pixels exist.
/// Among reviewed panels the ordering is a severity order:
/// `AtLeastAsGood < DifferentButFine < Regression`.
/// `DifferentButFine` is always Behavior-Noted — the `changed` field of its
/// manifest row names the BN note or ratification that carries the behavior.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Verdict {
    /// The render has a Reference capture but nobody has judged this render
    /// yet (a regenerated panel resets to it). Not a review result and not
    /// part of the severity order.
    Unreviewed,
    /// No captured Reference image exists, so assigning an aesthetic verdict
    /// would fabricate evidence. This is an evidence-gap refusal, not a
    /// human review result and not part of the severity order.
    ReferenceCaptureMissing,
    /// The FrankenManim panel is at least as good as the Reference capture.
    AtLeastAsGood,
    /// The panels differ in a way that is fine *and written down* (a
    /// Behavior-Noted ruling, e.g. BN-06's fill field).
    DifferentButFine,
    /// The FrankenManim panel is worse in a way nobody has signed off. The
    /// only verdict that demands action.
    Regression,
}

impl Verdict {
    /// The TSV token for this verdict.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Unreviewed => "unreviewed",
            Self::ReferenceCaptureMissing => "reference-capture-missing",
            Self::AtLeastAsGood => "at-least-as-good",
            Self::DifferentButFine => "different-but-fine",
            Self::Regression => "regression",
        }
    }

    /// Whether this is a review result (on the severity order), not a
    /// no-verdict state.
    #[must_use]
    pub fn is_judgment(self) -> bool {
        !matches!(self, Self::Unreviewed | Self::ReferenceCaptureMissing)
    }

    /// Parse a TSV verdict token.
    #[must_use]
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "unreviewed" => Some(Self::Unreviewed),
            "reference-capture-missing" => Some(Self::ReferenceCaptureMissing),
            "at-least-as-good" => Some(Self::AtLeastAsGood),
            "different-but-fine" => Some(Self::DifferentButFine),
            "regression" => Some(Self::Regression),
            _ => None,
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// One manifest row: a named pair, where both sides came from, its advisory
/// verdict, and the change that last moved the verdict.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GalleryRow {
    /// Panel id (`[a-z0-9._-]`, e.g. `gradient_fills`); a path-safe name.
    pub panel: String,
    /// The scene both engines rendered: `repo/relative/path.py:SceneClass`.
    pub source: String,
    /// Repo-relative path of the captured Reference image.
    pub reference: String,
    /// SHA-256 (lowercase hex) of the private Reference capture, or `None`
    /// when no capture was recorded.
    pub reference_sha256: Option<String>,
    /// Repo-relative path of the committed FrankenManim render.
    pub render: String,
    /// SHA-256 (lowercase hex) of the committed render's bytes.
    pub render_sha256: String,
    /// The build id of the release artefact that produced the render.
    pub build_id: String,
    /// The advisory (agent) verdict, `unreviewed`, or the explicit
    /// missing-capture refusal. Gates cite [`OwnerVerdicts`] instead.
    pub verdict: Verdict,
    /// Free-text record of the change that last moved the verdict (no tabs
    /// or newlines; by convention `bead date: reason`).
    pub changed: String,
}

/// A verdict movement between two manifest revisions.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VerdictChange {
    /// The panel that moved.
    pub panel: String,
    /// The verdict in the earlier manifest (`None` for a panel that did not
    /// exist there).
    pub from: Option<Verdict>,
    /// The verdict in the later manifest.
    pub to: Verdict,
}

/// A parsed look-gallery manifest: format version 2, a monotone `revision`
/// bumped by every verdict change or regeneration, the release under review,
/// the Reference identity, and rows sorted by panel on write.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GalleryManifest {
    /// Manifest revision; `record_verdict` bumps it.
    pub revision: u64,
    /// The build id of the release under review; every row must match it.
    pub release: String,
    /// The Reference identity the captures came from (`3b1b/manim@<commit>`).
    pub reference: String,
    /// The gallery rows.
    pub rows: Vec<GalleryRow>,
}

/// One manifest row resolved against a checkout.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedPair {
    /// Panel id.
    pub panel: String,
    /// Absolute path of the Reference capture.
    pub reference: PathBuf,
    /// Absolute path of the FrankenManim render (existence verified by
    /// [`render_pairs`]).
    pub render: PathBuf,
    /// Whether the private Reference capture exists in this checkout. Absence
    /// mutes the smoke alarm for this pair; it is never an error (§15.3
    /// fixtures are gitignored).
    pub reference_present: bool,
}

/// Everything the gallery can refuse to do, as a named error.
#[derive(Debug)]
pub enum GalleryError {
    /// An `RgbaView` failed validation.
    InvalidImage(String),
    /// The two images of a pair differ in dimensions; no metric is defined.
    DimensionMismatch {
        /// `(width, height)` of the reference.
        reference: (u32, u32),
        /// `(width, height)` of the candidate.
        candidate: (u32, u32),
    },
    /// The manifest text violates format v1.
    Corrupt {
        /// 1-based line number of the offending line.
        line: usize,
        /// What was wrong with it.
        detail: String,
    },
    /// The manifest exceeds the format envelope and therefore cannot be
    /// parsed or emitted canonically.
    ManifestTooLarge {
        /// Maximum permitted UTF-8 byte length.
        limit: usize,
    },
    /// Filesystem failure reading or writing the manifest.
    Io {
        /// The path being read or written.
        path: PathBuf,
        /// The underlying error.
        err: std::io::Error,
    },
    /// `record_verdict` was given a panel the manifest does not contain.
    UnknownPanel(String),
    /// `record_verdict` was given a change note containing a tab or newline,
    /// which would corrupt the TSV.
    InvalidChangeNote,
    /// `record_verdict` cannot advance an already maximal manifest revision.
    RevisionOverflow,
    /// `record_verdict` was asked for a judgment on a panel with no Reference
    /// capture: there is nothing it could have been judged against.
    NoCapture(String),
    /// A committed FrankenManim render named by the manifest is missing from
    /// the checkout. This is the "missing pair" the tests fail on.
    MissingRender {
        /// The panel whose render is missing.
        panel: String,
        /// The path that was expected to exist.
        path: PathBuf,
    },
    /// A panel was rendered by a build other than the release under review:
    /// a stale panel, never evidence about this release.
    StalePanel {
        /// The stale panel.
        panel: String,
        /// The build id that produced its render.
        build_id: String,
        /// The release the manifest is under review for.
        release: String,
    },
    /// A file's bytes do not have the digest the manifest records: the
    /// render (or a present private capture) changed after regeneration.
    DigestMismatch {
        /// The panel whose file changed.
        panel: String,
        /// The file that was hashed.
        path: PathBuf,
        /// The digest the manifest records.
        expected: String,
        /// The digest of the bytes on disk.
        actual: String,
    },
}

impl fmt::Display for GalleryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidImage(detail) => write!(f, "invalid image: {detail}"),
            Self::DimensionMismatch {
                reference,
                candidate,
            } => write!(
                f,
                "dimension mismatch: reference {}x{}, candidate {}x{}",
                reference.0, reference.1, candidate.0, candidate.1
            ),
            Self::Corrupt { line, detail } => {
                write!(f, "corrupt look-gallery manifest, line {line}: {detail}")
            }
            Self::ManifestTooLarge { limit } => {
                write!(
                    f,
                    "look-gallery manifest exceeds the {limit}-byte format limit"
                )
            }
            Self::Io { path, err } => {
                write!(f, "look-gallery I/O failure at {}: {err}", path.display())
            }
            Self::UnknownPanel(panel) => write!(f, "unknown gallery panel {panel:?}"),
            Self::InvalidChangeNote => {
                write!(f, "change note must not contain tabs or newlines")
            }
            Self::RevisionOverflow => {
                write!(
                    f,
                    "look-gallery manifest revision cannot advance past u64::MAX"
                )
            }
            Self::NoCapture(panel) => write!(
                f,
                "gallery panel {panel:?} has no Reference capture to judge it against"
            ),
            Self::MissingRender { panel, path } => write!(
                f,
                "gallery panel {panel:?} names a render that is missing from the \
                 checkout: {}",
                path.display()
            ),
            Self::StalePanel {
                panel,
                build_id,
                release,
            } => write!(
                f,
                "gallery panel {panel:?} was rendered by {build_id}, not by the release \
                 under review {release}; regenerate it"
            ),
            Self::DigestMismatch {
                panel,
                path,
                expected,
                actual,
            } => write!(
                f,
                "gallery panel {panel:?}: {} has SHA-256 {actual}, the manifest records \
                 {expected}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for GalleryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { err, .. } => Some(err),
            _ => None,
        }
    }
}

// ------------------------------------------------------------ the metrics

/// The Rec. 709 luma plane of an image, per the engine-equivalence lane's
/// canonical conversion (`0.2126 R + 0.7152 G + 0.0722 B` over sRGB8 codes).
fn luma_plane(view: &RgbaView<'_>) -> Vec<f64> {
    view.pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]))
        .collect()
}

/// Global SSIM over two luma planes of equal, nonzero length.
///
/// This is the formula family the engine-equivalence budgets consume (see the
/// module docs): global statistics, sample-variance divisor `max(n−1, 1)`,
/// `C1 = (0.01·255)²`, `C2 = (0.03·255)²`. Identical planes score exactly 1.0.
#[must_use]
pub fn global_ssim_luma(reference: &[f64], candidate: &[f64]) -> f64 {
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
    for (&r, &c) in reference.iter().zip(candidate) {
        reference_variance += (r - reference_mean) * (r - reference_mean);
        candidate_variance += (c - candidate_mean) * (c - candidate_mean);
        covariance += (r - reference_mean) * (c - candidate_mean);
    }
    let divisor = (count - 1.0).max(1.0);
    reference_variance /= divisor;
    candidate_variance /= divisor;
    covariance /= divisor;

    let c1 = fmn_dmath::powi(0.01 * 255.0, 2);
    let c2 = fmn_dmath::powi(0.03 * 255.0, 2);
    ((2.0 * reference_mean * candidate_mean + c1) * (2.0 * covariance + c2))
        / ((reference_mean * reference_mean + candidate_mean * candidate_mean + c1)
            * (reference_variance + candidate_variance + c2))
}

/// The thresholded Sobel edge map of a luma plane (interior pixels only;
/// border pixels can never be edges because the kernel needs a full 3×3
/// neighborhood).
fn edge_map(luma: &[f64], width: usize, height: usize) -> Vec<bool> {
    let mut edges = vec![false; luma.len()];
    if width < 3 || height < 3 {
        return edges;
    }
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let at = |dx: usize, dy: usize| luma[(y + dy - 1) * width + (x + dx - 1)];
            // 3×3 Sobel: gx weights the right column positive, gy the bottom.
            let gx = at(2, 0) + 2.0 * at(2, 1) + at(2, 2) - at(0, 0) - 2.0 * at(0, 1) - at(0, 2);
            let gy = at(0, 2) + 2.0 * at(1, 2) + at(2, 2) - at(0, 0) - 2.0 * at(1, 0) - at(2, 0);
            edges[y * width + x] = gx.abs() + gy.abs() >= EDGE_L1_THRESHOLD;
        }
    }
    edges
}

/// Borgefors 3-4 chamfer distance transform: each pixel's distance (in thirds
/// of a pixel — 3 orthogonal, 4 diagonal) to the nearest `true` cell of
/// `target`, by one forward and one backward raster pass.
const fn chamfer_far(width: u32, height: u32) -> u64 {
    // The largest inputs sum to less than 2^33, so widening before the
    // arithmetic covers the complete public dimension domain exactly.
    3 * (width as u64 + height as u64) + 4
}

fn chamfer34(target: &[bool], width: u32, height: u32) -> Vec<u64> {
    // Larger than any true 3-4 distance inside the frame.
    let far = chamfer_far(width, height);
    let (width, height) = (width as usize, height as usize);
    let mut dt: Vec<u64> = target.iter().map(|&e| if e { 0 } else { far }).collect();
    let at = |dt: &[u64], x: usize, y: usize| dt[y * width + x];
    for y in 0..height {
        for x in 0..width {
            let mut best = at(&dt, x, y);
            if x > 0 {
                best = best.min(at(&dt, x - 1, y) + 3);
            }
            if y > 0 {
                best = best.min(at(&dt, x, y - 1) + 3);
                if x > 0 {
                    best = best.min(at(&dt, x - 1, y - 1) + 4);
                }
                if x + 1 < width {
                    best = best.min(at(&dt, x + 1, y - 1) + 4);
                }
            }
            dt[y * width + x] = best;
        }
    }
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let mut best = at(&dt, x, y);
            if x + 1 < width {
                best = best.min(at(&dt, x + 1, y) + 3);
            }
            if y + 1 < height {
                best = best.min(at(&dt, x, y + 1) + 3);
                if x + 1 < width {
                    best = best.min(at(&dt, x + 1, y + 1) + 4);
                }
                if x > 0 {
                    best = best.min(at(&dt, x - 1, y + 1) + 4);
                }
            }
            dt[y * width + x] = best;
        }
    }
    dt
}

/// Mean chamfer distance, in pixels, from the `true` cells of `source` to the
/// nearest `true` cell of `target`, under the conventions in the module docs.
fn directed_chamfer(source: &[bool], target: &[bool], width: u32, height: u32) -> (u64, f64) {
    let source_count = source.iter().filter(|&&e| e).count() as u64;
    let target_count = target.iter().filter(|&&e| e).count() as u64;
    if source_count == 0 {
        return (0, 0.0);
    }
    if target_count == 0 {
        let width = f64::from(width);
        let height = f64::from(height);
        let diagonal = (width * width + height * height).sqrt();
        return (source_count, diagonal);
    }
    let dt = chamfer34(target, width, height);
    let total: u128 = source
        .iter()
        .zip(&dt)
        .filter(|(e, _)| **e)
        .map(|(_, &d)| u128::from(d))
        .sum();
    (source_count, total as f64 / 3.0 / source_count as f64)
}

/// The symmetric chamfer edge distance between two luma planes.
///
/// # Errors
/// [`GalleryError::InvalidImage`] if the dimensions are zero, their pixel
/// count is not addressable on this target, or either luma plane is not
/// exactly `width * height` samples.
pub fn edge_distance_luma(
    reference: &[f64],
    candidate: &[f64],
    width: u32,
    height: u32,
) -> Result<EdgeDistance, GalleryError> {
    if width == 0 || height == 0 {
        return Err(GalleryError::InvalidImage(format!(
            "zero raw-luma dimension ({width}x{height})"
        )));
    }
    let expected = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| usize::try_from(pixels).ok())
        .ok_or_else(|| {
            GalleryError::InvalidImage(format!(
                "raw-luma dimensions {width}x{height} overflow the addressable sample length"
            ))
        })?;
    for (side, actual) in [
        ("reference", reference.len()),
        ("candidate", candidate.len()),
    ] {
        if actual != expected {
            return Err(GalleryError::InvalidImage(format!(
                "raw-luma {side} has {actual} samples for {width}x{height}, expected {expected}"
            )));
        }
    }

    let (w, h) = (width as usize, height as usize);
    let reference_edges = edge_map(reference, w, h);
    let candidate_edges = edge_map(candidate, w, h);
    let (reference_count, reference_to_candidate) =
        directed_chamfer(&reference_edges, &candidate_edges, width, height);
    let (candidate_count, candidate_to_reference) =
        directed_chamfer(&candidate_edges, &reference_edges, width, height);
    Ok(EdgeDistance {
        reference_edges: reference_count,
        candidate_edges: candidate_count,
        reference_to_candidate,
        candidate_to_reference,
        symmetric: (reference_to_candidate + candidate_to_reference) / 2.0,
    })
}

/// Nearest-rank percentiles of the per-pixel max RGB channel error.
///
/// Nearest-rank: for quantile `q`, the element at index `⌈q·n⌉−1` of the
/// sorted errors. Deterministic, interpolation-free, and defined for every
/// nonempty image.
#[must_use]
pub fn error_percentiles(reference: &RgbaView<'_>, candidate: &RgbaView<'_>) -> ErrorPercentiles {
    let mut errors: Vec<f64> = reference
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(candidate.pixels.as_chunks::<4>().0)
        .map(|(a, b)| {
            let mut worst = 0.0_f64;
            for channel in 0..3 {
                let error = f64::from(a[channel].abs_diff(b[channel])) / 255.0;
                if error > worst {
                    worst = error;
                }
            }
            worst
        })
        .collect();
    errors.sort_by(f64::total_cmp);
    let n = errors.len();
    let pick = |q: f64| {
        let rank = (q * n as f64).ceil() as usize;
        errors[rank.max(1) - 1]
    };
    ErrorPercentiles {
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        max: errors[n - 1],
    }
}

/// Compute all three smoke-alarm metrics for one pair.
///
/// # Errors
/// [`GalleryError::DimensionMismatch`] if the two views differ in size.
pub fn compare_pair(
    reference: &RgbaView<'_>,
    candidate: &RgbaView<'_>,
) -> Result<PairMetrics, GalleryError> {
    if (reference.width, reference.height) != (candidate.width, candidate.height) {
        return Err(GalleryError::DimensionMismatch {
            reference: (reference.width, reference.height),
            candidate: (candidate.width, candidate.height),
        });
    }
    let reference_luma = luma_plane(reference);
    let candidate_luma = luma_plane(candidate);
    Ok(PairMetrics {
        ssim: global_ssim_luma(&reference_luma, &candidate_luma),
        edge: edge_distance_luma(
            &reference_luma,
            &candidate_luma,
            reference.width,
            reference.height,
        )?,
        error: error_percentiles(reference, candidate),
    })
}

// ------------------------------------------------------------ the manifest

/// Panel ids are path components: the golden rig's conservative charset.
fn valid_panel(panel: &str) -> bool {
    !panel.is_empty()
        && panel.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
        && !panel.starts_with('.')
}

/// A manifest path is repo-relative, `/`-separated, and stays inside the
/// repository: no leading or trailing `/`, no `..`, no `.` components, no
/// backslashes, no control characters.
fn valid_repo_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/'))
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

/// Split one row into exactly `N` fields without allocating a field vector
/// for malformed input carrying an arbitrary number of tab separators.
fn split_row<const N: usize>(line: &str) -> Option<[&str; N]> {
    let mut fields = line.split('\t');
    let mut exact = [""; N];
    for slot in &mut exact {
        *slot = fields.next()?;
    }
    fields.next().is_none().then_some(exact)
}

/// A lowercase hexadecimal SHA-256 digest.
fn valid_sha256(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A build or Reference identity: short printable ASCII without separators,
/// e.g. `git:<commit>`, `git:<commit>+dirty:<digest>` or `3b1b/manim@<commit>`.
fn valid_identity(identity: &str) -> bool {
    (1..=200).contains(&identity.len())
        && identity.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b':' | b'+' | b'.' | b'_' | b'-' | b'/' | b'@')
        })
}

/// A scene source: a repo path and a Python class name, `path.py:SceneClass`.
fn valid_source(source: &str) -> bool {
    source.split_once(':').is_some_and(|(path, scene)| {
        valid_repo_path(path)
            && path.ends_with(".py")
            && scene.starts_with(|c: char| c.is_ascii_alphabetic())
            && scene
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    })
}

fn digest_hex(bytes: &[u8]) -> String {
    fmn_hash::sha256(bytes).to_hex()
}

fn oversized_manifest() -> GalleryError {
    GalleryError::ManifestTooLarge {
        limit: MAX_MANIFEST_BYTES,
    }
}

impl GalleryManifest {
    /// Parse manifest text in format v2.
    ///
    /// # Errors
    /// [`GalleryError::Corrupt`] on any format violation: wrong header, bad
    /// revision or columns line, non-canonical line endings or row order,
    /// wrong field count, unknown verdict token, invalid panel or path, or a
    /// duplicate panel.
    pub fn parse(text: &str) -> Result<Self, GalleryError> {
        if text.len() > MAX_MANIFEST_BYTES {
            return Err(oversized_manifest());
        }
        if text.is_empty() {
            return Err(GalleryError::Corrupt {
                line: 1,
                detail: "empty manifest".to_string(),
            });
        }
        if let Some(offset) = text.find('\r') {
            let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
            return Err(GalleryError::Corrupt {
                line,
                detail: "carriage returns are not canonical; use LF line endings".to_string(),
            });
        }
        let Some(body) = text.strip_suffix('\n') else {
            return Err(GalleryError::Corrupt {
                line: text.bytes().filter(|byte| *byte == b'\n').count() + 1,
                detail: "manifest must end with a final LF".to_string(),
            });
        };

        let mut rows: Vec<GalleryRow> = Vec::new();
        let mut previous_panel: Option<&str> = None;
        let mut lines = body.split('\n').enumerate();
        let Some((_, header)) = lines.next() else {
            return Err(GalleryError::Corrupt {
                line: 1,
                detail: "empty manifest".to_string(),
            });
        };
        if header != MANIFEST_HEADER {
            return Err(GalleryError::Corrupt {
                line: 1,
                detail: format!("first line must be {MANIFEST_HEADER:?}"),
            });
        }

        let Some((_, revision_line)) = lines.next() else {
            return Err(GalleryError::Corrupt {
                line: 2,
                detail: "missing '# revision: N' line".to_string(),
            });
        };
        let revision_text = revision_line
            .strip_prefix(MANIFEST_REVISION_PREFIX)
            .ok_or_else(|| GalleryError::Corrupt {
                line: 2,
                detail: format!("second line must start with {MANIFEST_REVISION_PREFIX:?}"),
            })?;
        let revision = revision_text
            .parse::<u64>()
            .map_err(|_| GalleryError::Corrupt {
                line: 2,
                detail: format!("revision is not a non-negative integer: {revision_text:?}"),
            })?;
        if revision.to_string() != revision_text {
            return Err(GalleryError::Corrupt {
                line: 2,
                detail: format!(
                    "revision is not in canonical unsigned-decimal form: {revision_text:?}"
                ),
            });
        }

        let mut identity = |line: usize, prefix: &str, what: &str| {
            let text = lines
                .next()
                .and_then(|(_, text)| text.strip_prefix(prefix))
                .ok_or_else(|| GalleryError::Corrupt {
                    line,
                    detail: format!("line {line} must start with {prefix:?}"),
                })?;
            if valid_identity(text) {
                Ok(text.to_string())
            } else {
                Err(GalleryError::Corrupt {
                    line,
                    detail: format!("invalid {what} identity {text:?}"),
                })
            }
        };
        let release = identity(3, MANIFEST_RELEASE_PREFIX, "release")?;
        let reference_identity = identity(4, MANIFEST_REFERENCE_PREFIX, "reference")?;

        let Some((_, columns)) = lines.next() else {
            return Err(GalleryError::Corrupt {
                line: 5,
                detail: format!("missing columns line {MANIFEST_COLUMNS:?}"),
            });
        };
        if columns != MANIFEST_COLUMNS {
            return Err(GalleryError::Corrupt {
                line: 5,
                detail: format!("fifth line must be {MANIFEST_COLUMNS:?}"),
            });
        }

        for (index, line) in lines {
            let line_number = index + 1;
            let corrupt = |detail: String| GalleryError::Corrupt {
                line: line_number,
                detail,
            };
            if line.is_empty() {
                return Err(corrupt("blank lines are not canonical".to_string()));
            }
            if line.starts_with(MANIFEST_REVISION_PREFIX) {
                return Err(corrupt(
                    "duplicate revision line: format v2 requires exactly one".to_string(),
                ));
            }
            if line.starts_with('#') {
                return Err(corrupt(format!(
                    "unexpected comment line {line:?}: format v2 has exactly five prelude lines"
                )));
            }
            let Some(
                [
                    panel,
                    source,
                    reference,
                    reference_sha256,
                    render,
                    render_sha256,
                    build_id,
                    verdict,
                    changed,
                ],
            ) = split_row::<MANIFEST_FIELDS>(line)
            else {
                let field_count = line.split('\t').count();
                return Err(corrupt(format!(
                    "expected {MANIFEST_FIELDS} tab-separated fields, found {field_count}"
                )));
            };
            if !valid_panel(panel) {
                return Err(corrupt(format!(
                    "invalid panel id {panel:?}: use only [a-z0-9._-], not starting with '.'"
                )));
            }
            if let Some(previous) = previous_panel {
                if panel == previous {
                    return Err(corrupt(format!("duplicate panel {panel:?}")));
                }
                if panel < previous {
                    return Err(corrupt(format!(
                        "panel {panel:?} is out of canonical order after {previous:?}"
                    )));
                }
            }
            previous_panel = Some(panel);
            if !valid_repo_path(reference) {
                return Err(corrupt(format!(
                    "invalid reference path {reference:?}: must be repo-relative and stay \
                     inside the repository"
                )));
            }
            if !valid_repo_path(render) {
                return Err(corrupt(format!(
                    "invalid render path {render:?}: must be repo-relative and stay inside \
                     the repository"
                )));
            }
            if !valid_source(source) {
                return Err(corrupt(format!(
                    "invalid source {source:?}: expected repo/relative/path.py:SceneClass"
                )));
            }
            let reference_sha256 = match reference_sha256 {
                NO_CAPTURE => None,
                digest if valid_sha256(digest) => Some(digest.to_string()),
                digest => {
                    return Err(corrupt(format!(
                        "invalid reference_sha256 {digest:?}: lowercase SHA-256 hex or '-'"
                    )));
                }
            };
            if !valid_sha256(render_sha256) {
                return Err(corrupt(format!(
                    "invalid render_sha256 {render_sha256:?}: lowercase SHA-256 hex"
                )));
            }
            if !valid_identity(build_id) {
                return Err(corrupt(format!("invalid build_id {build_id:?}")));
            }
            let verdict = Verdict::from_token(verdict).ok_or_else(|| {
                corrupt(format!(
                    "unknown verdict {verdict:?}: expected unreviewed, \
                     reference-capture-missing, at-least-as-good, different-but-fine, \
                     or regression"
                ))
            })?;
            if verdict.is_judgment() && reference_sha256.is_none() {
                return Err(corrupt(format!(
                    "panel {panel:?} carries the verdict {verdict} but records no Reference \
                     capture to have judged it against"
                )));
            }
            if changed.is_empty() {
                return Err(corrupt(format!(
                    "panel {panel:?} has an empty change note: record what moved the verdict"
                )));
            }
            rows.push(GalleryRow {
                panel: panel.to_string(),
                source: source.to_string(),
                reference: reference.to_string(),
                reference_sha256,
                render: render.to_string(),
                render_sha256: render_sha256.to_string(),
                build_id: build_id.to_string(),
                verdict,
                changed: changed.to_string(),
            });
        }
        Ok(Self {
            revision,
            release,
            reference: reference_identity,
            rows,
        })
    }

    /// Load a manifest from disk.
    ///
    /// # Errors
    /// [`GalleryError::Io`] on read failure, [`GalleryError::Corrupt`] on a
    /// format violation.
    pub fn load(path: &Path) -> Result<Self, GalleryError> {
        let file = std::fs::File::open(path).map_err(|err| GalleryError::Io {
            path: path.to_path_buf(),
            err,
        })?;
        let mut bytes = Vec::new();
        file.take((MAX_MANIFEST_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|err| GalleryError::Io {
                path: path.to_path_buf(),
                err,
            })?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(oversized_manifest());
        }
        let text = std::str::from_utf8(&bytes).map_err(|err| GalleryError::Io {
            path: path.to_path_buf(),
            err: std::io::Error::new(std::io::ErrorKind::InvalidData, err),
        })?;
        Self::parse(text)
    }

    fn serialized_len(&self) -> Result<usize, GalleryError> {
        let revision = self.revision.to_string();
        let mut len = MANIFEST_HEADER
            .len()
            .saturating_add(MANIFEST_REVISION_PREFIX.len())
            .saturating_add(revision.len())
            .saturating_add(MANIFEST_RELEASE_PREFIX.len())
            .saturating_add(self.release.len())
            .saturating_add(MANIFEST_REFERENCE_PREFIX.len())
            .saturating_add(self.reference.len())
            .saturating_add(MANIFEST_COLUMNS.len())
            .saturating_add(5);
        for row in &self.rows {
            len = len
                .saturating_add(row.panel.len())
                .saturating_add(row.source.len())
                .saturating_add(row.reference.len())
                .saturating_add(row.reference_sha256.as_deref().unwrap_or(NO_CAPTURE).len())
                .saturating_add(row.render.len())
                .saturating_add(row.render_sha256.len())
                .saturating_add(row.build_id.len())
                .saturating_add(row.verdict.token().len())
                .saturating_add(row.changed.len())
                .saturating_add(MANIFEST_FIELDS);
        }
        if len > MAX_MANIFEST_BYTES {
            Err(oversized_manifest())
        } else {
            Ok(len)
        }
    }

    /// The canonical text form: header, revision, column legend, rows sorted
    /// by panel. `to_text(parse(text))` is byte-identical for canonical input.
    ///
    /// # Errors
    /// [`GalleryError::ManifestTooLarge`] if the canonical document would
    /// exceed the format envelope.
    pub fn to_text(&self) -> Result<String, GalleryError> {
        let mut rows: Vec<&GalleryRow> = self.rows.iter().collect();
        rows.sort_by(|a, b| a.panel.cmp(&b.panel));
        let mut out = String::with_capacity(self.serialized_len()?);
        out.push_str(MANIFEST_HEADER);
        out.push('\n');
        out.push_str(MANIFEST_REVISION_PREFIX);
        out.push_str(&self.revision.to_string());
        out.push('\n');
        out.push_str(MANIFEST_RELEASE_PREFIX);
        out.push_str(&self.release);
        out.push('\n');
        out.push_str(MANIFEST_REFERENCE_PREFIX);
        out.push_str(&self.reference);
        out.push('\n');
        out.push_str(MANIFEST_COLUMNS);
        out.push('\n');
        for row in rows {
            let fields = [
                row.panel.as_str(),
                row.source.as_str(),
                row.reference.as_str(),
                row.reference_sha256.as_deref().unwrap_or(NO_CAPTURE),
                row.render.as_str(),
                row.render_sha256.as_str(),
                row.build_id.as_str(),
                row.verdict.token(),
                row.changed.as_str(),
            ];
            out.push_str(&fields.join("\t"));
            out.push('\n');
        }
        Ok(out)
    }

    /// Save the manifest atomically (tmp file in the same directory, then
    /// rename — the self-golden rig's pattern).
    ///
    /// # Errors
    /// [`GalleryError::ManifestTooLarge`] if the canonical document exceeds
    /// the format envelope; [`GalleryError::Io`] on any filesystem failure.
    pub fn save(&self, path: &Path) -> Result<(), GalleryError> {
        let text = self.to_text()?;
        let io = |err| GalleryError::Io {
            path: path.to_path_buf(),
            err,
        };
        let sequence = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_extension(format!("tmp{sequence}"));
        {
            let mut file = std::fs::File::create(&tmp).map_err(io)?;
            file.write_all(text.as_bytes()).map_err(io)?;
            file.sync_all().map_err(io)?;
        }
        std::fs::rename(&tmp, path).map_err(io)
    }

    /// Move one panel's verdict, recording why, and bump the revision.
    ///
    /// # Errors
    /// [`GalleryError::UnknownPanel`] if the panel is not in the manifest;
    /// [`GalleryError::InvalidChangeNote`] if the note contains a tab or
    /// newline (or is empty); [`GalleryError::RevisionOverflow`] if the
    /// manifest revision is already `u64::MAX`;
    /// [`GalleryError::ManifestTooLarge`] if the updated canonical document
    /// would exceed the format envelope. Every refusal leaves `self`
    /// unchanged.
    pub fn record_verdict(
        &mut self,
        panel: &str,
        verdict: Verdict,
        changed: &str,
    ) -> Result<VerdictChange, GalleryError> {
        if changed.is_empty() || changed.contains(['\t', '\n', '\r']) {
            return Err(GalleryError::InvalidChangeNote);
        }
        let row_index = self
            .rows
            .iter()
            .position(|row| row.panel == panel)
            .ok_or_else(|| GalleryError::UnknownPanel(panel.to_string()))?;
        if verdict.is_judgment() && self.rows[row_index].reference_sha256.is_none() {
            return Err(GalleryError::NoCapture(panel.to_string()));
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(GalleryError::RevisionOverflow)?;
        let current_len = self.serialized_len()?;
        let row = &self.rows[row_index];
        let from = row.verdict;
        let next_len = current_len
            .saturating_sub(self.revision.to_string().len())
            .saturating_sub(row.verdict.token().len())
            .saturating_sub(row.changed.len())
            .saturating_add(next_revision.to_string().len())
            .saturating_add(verdict.token().len())
            .saturating_add(changed.len());
        if next_len > MAX_MANIFEST_BYTES {
            return Err(oversized_manifest());
        }
        let row = &mut self.rows[row_index];
        row.verdict = verdict;
        row.changed = changed.to_string();
        self.revision = next_revision;
        Ok(VerdictChange {
            panel: panel.to_string(),
            from: Some(from),
            to: verdict,
        })
    }

    /// The panels whose verdict *worsened* relative to an earlier manifest
    /// revision, sorted by panel. Worsened means strictly higher on the
    /// severity order `AtLeastAsGood < DifferentButFine < Regression`; the
    /// `Unreviewed` and `ReferenceCaptureMissing` states are incomparable and
    /// therefore never a regression. A
    /// panel that did not exist in `earlier` counts only when it enters at
    /// `Regression` (a new `DifferentButFine` panel is a review item, not a
    /// regression).
    #[must_use]
    pub fn regressions_since(&self, earlier: &Self) -> Vec<VerdictChange> {
        let old: BTreeMap<&str, Verdict> = earlier
            .rows
            .iter()
            .map(|row| (row.panel.as_str(), row.verdict))
            .collect();
        let mut changes = Vec::new();
        for row in &self.rows {
            match old.get(row.panel.as_str()) {
                Some(&from)
                    if from.is_judgment() && row.verdict.is_judgment() && row.verdict > from =>
                {
                    changes.push(VerdictChange {
                        panel: row.panel.clone(),
                        from: Some(from),
                        to: row.verdict,
                    });
                }
                // ubs:ignore — enum equality on the verdict vocabulary, never a secret.
                None if row.verdict == Verdict::Regression => changes.push(VerdictChange {
                    panel: row.panel.clone(),
                    from: None,
                    to: row.verdict,
                }),
                _ => {}
            }
        }
        changes.sort_by(|a, b| a.panel.cmp(&b.panel));
        changes
    }

    /// Look up one panel's row.
    #[must_use]
    pub fn row(&self, panel: &str) -> Option<&GalleryRow> {
        self.rows.iter().find(|row| row.panel == panel)
    }
}

/// Resolve a manifest against a checkout rooted at `repo_root`, sorted by
/// panel, refusing anything that is not evidence about the release under
/// review.
///
/// - A panel whose `build_id` is not the manifest's `release` is a
///   [`GalleryError::StalePanel`]: it shows another build's pixels.
/// - A missing **render** is [`GalleryError::MissingRender`]: the panels are
///   committed artifacts and their absence is a broken checkout.
/// - A render whose bytes do not hash to `render_sha256`, or a present
///   private capture that does not hash to `reference_sha256`, is a
///   [`GalleryError::DigestMismatch`]: the file changed after regeneration.
/// - A missing **reference** is not an error. The captures are private §15.3
///   fixtures; [`ResolvedPair::reference_present`] says which pairs are
///   measurable.
///
/// # Errors
/// The first refusal above, in panel order.
pub fn render_pairs(
    manifest: &GalleryManifest,
    repo_root: &Path,
) -> Result<Vec<ResolvedPair>, GalleryError> {
    let mut rows: Vec<&GalleryRow> = manifest.rows.iter().collect();
    rows.sort_by(|a, b| a.panel.cmp(&b.panel));
    let mut pairs = Vec::with_capacity(rows.len());
    for row in rows {
        if row.build_id != manifest.release {
            return Err(GalleryError::StalePanel {
                panel: row.panel.clone(),
                build_id: row.build_id.clone(),
                release: manifest.release.clone(),
            });
        }
        let render = repo_root.join(&row.render);
        if !render.is_file() {
            return Err(GalleryError::MissingRender {
                panel: row.panel.clone(),
                path: render,
            });
        }
        verify_digest(&row.panel, &render, &row.render_sha256)?;
        let reference = repo_root.join(&row.reference);
        let reference_present = reference.is_file();
        if let (true, Some(expected)) = (reference_present, &row.reference_sha256) {
            verify_digest(&row.panel, &reference, expected)?;
        }
        pairs.push(ResolvedPair {
            panel: row.panel.clone(),
            reference_present,
            reference,
            render,
        });
    }
    Ok(pairs)
}

fn verify_digest(panel: &str, path: &Path, expected: &str) -> Result<(), GalleryError> {
    let bytes = std::fs::read(path).map_err(|err| GalleryError::Io {
        path: path.to_path_buf(),
        err,
    })?;
    let actual = digest_hex(&bytes);
    if actual == expected {
        Ok(())
    } else {
        Err(GalleryError::DigestMismatch {
            panel: panel.to_string(),
            path: path.to_path_buf(),
            expected: expected.to_string(),
            actual,
        })
    }
}

// --------------------------------------------------- the owner verdict lane

/// One owner verdict: a judgment of one panel's render, identified by digest.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnerVerdict {
    /// The panel judged.
    pub panel: String,
    /// SHA-256 of the exact render judged; a regenerated render is unjudged.
    pub render_sha256: String,
    /// `AtLeastAsGood`, `DifferentButFine` or `Regression` only.
    pub verdict: Verdict,
    /// Why; a `DifferentButFine` note names its Behavior Note (`BN-nn`).
    pub note: String,
    /// When, `YYYY-MM-DD`.
    pub date: String,
}

/// The owner's verdict ledger (`fixtures/look_gallery_owner_verdicts.tsv`,
/// format `# fmn-look-gallery-owner-verdicts v1`). G2 and G4a cite only these;
/// the manifest's verdict column is advisory. Rows are append-only history:
/// the current verdict on a panel is the last row whose digest matches the
/// manifest's render.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnerVerdicts {
    /// Every recorded owner verdict, in file order.
    pub rows: Vec<OwnerVerdict>,
}

impl OwnerVerdicts {
    /// Parse the owner ledger. Only judgments are accepted: `unreviewed` and
    /// `reference-capture-missing` are not verdicts an owner records.
    ///
    /// # Errors
    /// [`GalleryError::Corrupt`] on a wrong header or column line, a
    /// non-canonical line, a wrong field count, an invalid panel, digest or
    /// date, a non-judgment verdict, or a `different-but-fine` note that
    /// names no Behavior Note.
    pub fn parse(text: &str) -> Result<Self, GalleryError> {
        if text.len() > MAX_MANIFEST_BYTES {
            return Err(oversized_manifest());
        }
        let Some(body) = text.strip_suffix('\n') else {
            return Err(GalleryError::Corrupt {
                line: text.bytes().filter(|byte| *byte == b'\n').count() + 1,
                detail: "owner verdicts must end with a final LF".to_string(),
            });
        };
        let mut lines = body.split('\n').enumerate();
        for (line, expected) in [(1, OWNER_HEADER), (2, OWNER_COLUMNS)] {
            if lines.next().map(|(_, text)| text) != Some(expected) {
                return Err(GalleryError::Corrupt {
                    line,
                    detail: format!("line {line} must be {expected:?}"),
                });
            }
        }
        let mut rows = Vec::new();
        for (index, line) in lines {
            let corrupt = |detail: String| GalleryError::Corrupt {
                line: index + 1,
                detail,
            };
            let Some([panel, render_sha256, verdict, note, date]) = split_row::<5>(line) else {
                return Err(corrupt(format!(
                    "expected 5 tab-separated fields, found {}",
                    line.split('\t').count()
                )));
            };
            if !valid_panel(panel) {
                return Err(corrupt(format!("invalid panel id {panel:?}")));
            }
            if !valid_sha256(render_sha256) {
                return Err(corrupt(format!("invalid render_sha256 {render_sha256:?}")));
            }
            let verdict = Verdict::from_token(verdict)
                .filter(|v| v.is_judgment())
                .ok_or_else(|| {
                    corrupt(format!(
                        "owner verdict {verdict:?} is not at-least-as-good, \
                         different-but-fine or regression"
                    ))
                })?;
            if note.is_empty() || note.contains('\r') {
                return Err(corrupt("an owner verdict needs a note".to_string()));
            }
            if verdict == Verdict::DifferentButFine && !names_behavior_note(note) {
                return Err(corrupt(format!(
                    "a different-but-fine verdict must name its Behavior Note (BN-nn): {note:?}"
                )));
            }
            if !valid_date(date) {
                return Err(corrupt(format!(
                    "invalid date {date:?}: expected YYYY-MM-DD"
                )));
            }
            rows.push(OwnerVerdict {
                panel: panel.to_string(),
                render_sha256: render_sha256.to_string(),
                verdict,
                note: note.to_string(),
                date: date.to_string(),
            });
        }
        Ok(Self { rows })
    }

    /// The owner's current verdict per panel: the last row whose digest is
    /// the manifest's render. A verdict on an earlier render is history, not
    /// a verdict on this release.
    #[must_use]
    pub fn current(&self, manifest: &GalleryManifest) -> BTreeMap<String, Verdict> {
        let digests: BTreeMap<&str, &str> = manifest
            .rows
            .iter()
            .map(|row| (row.panel.as_str(), row.render_sha256.as_str()))
            .collect();
        let mut current = BTreeMap::new();
        for row in &self.rows {
            if digests.get(row.panel.as_str()) == Some(&row.render_sha256.as_str()) {
                current.insert(row.panel.clone(), row.verdict);
            }
        }
        current
    }
}

fn names_behavior_note(note: &str) -> bool {
    note.as_bytes()
        .windows(5)
        .any(|w| w.starts_with(b"BN-") && w[3].is_ascii_digit() && w[4].is_ascii_digit())
}

fn valid_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::chamfer_far;

    #[test]
    fn chamfer_sentinel_covers_the_full_u32_dimension_domain() {
        let far = chamfer_far(u32::MAX, u32::MAX);
        assert_eq!(far, 25_769_803_774);
        assert!(far > u64::from(u32::MAX));
    }
}
