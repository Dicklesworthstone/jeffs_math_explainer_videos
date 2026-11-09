//! §10.1's **certified CPU engine** — the arithmetic whose output *is* the
//! definition of the bits.
//!
//! > … the **certified CPU engine** (canonical arithmetic; the definition of the
//! > bits), the **fast CPU engine** … and the standard-only **annex engines** …
//! > which share the IR and the test suite, never a lowest-common-denominator
//! > kernel. … Fast-CPU and annex engines are held to an explicit, versioned
//! > visual-equivalence budget against certified reference frames; **the
//! > certified engine is held to bits.**
//!
//! Everything above this module computes coverage and colour; this module is
//! where a *frame* exists, and therefore where §10.5's parallelism contract
//! stops being a doctrine and becomes a data structure.
//!
//! ## The four properties, and where each one lives
//!
//! ADR-0010 retired §10.5's fixed-point raster boundary on measured evidence and
//! made four properties binding in its place. They are not implementation
//! details — an engine that broke one would emit a provenance manifest claiming
//! a reproducibility it no longer has — so each one has an address:
//!
//! | Property | Where it is enforced |
//! |---|---|
//! | fmn-dmath owns every certified transcendental | `fmn-geom`'s `scalar` funnel, `fmn-frame`'s `transfer`, and the repo-wide guard `fmn-conformance/tests/certified_arithmetic.rs` |
//! | No FMA contraction (§10.5d) | the same guard, over every certified crate root |
//! | Fixed-order reductions (§10.5c) | this module: [`FrameJob`]'s draw order, and ADR-0013 |
//! | IEEE-754 basic operations only | certified accumulation is `f64`; `sqrt` is the only non-arithmetic primitive, and IEEE-754 requires it correctly rounded |
//!
//! ## Why a band is the unit of parallelism
//!
//! §10.5(b) asks for tiles that are *write-disjoint* and composite in a *fixed
//! order*. Those are two different requirements and it is worth separating them,
//! because only one of them is about threads.
//!
//! Compositing order is **painter order within a tile**, and it is fixed by the
//! command list [`crate::bin::Binning`] built — a property of the scene, not of
//! the schedule. Across tiles there is no compositing at all: a pixel belongs to
//! exactly one tile, so two tiles cannot disagree about it.
//!
//! That leaves write-disjointness, which is a *typing* problem: the frame is one
//! allocation and several threads must write it. Rather than hand out overlapping
//! views and argue about safety, the engine slices the frame into **bands** — one
//! fine-tile row of pixels each — with [`slice::chunks_mut`], which yields
//! provably disjoint `&mut [u8]`. Disjointness is then something the borrow
//! checker knows rather than something a comment claims, and `#![forbid(unsafe_code)]`
//! survives a multi-threaded rasterizer without a single escape hatch.
//!
//! Bands are handed out from a shared queue, so the *assignment* is dynamic (a
//! band full of glyphs does not stall a thread that drew empty sky) while the
//! *bytes* are not: band `b`'s bytes are a pure function of `b`, the plan, and
//! the config. That is the whole of thread-count independence, and
//! `the_frame_is_identical_at_every_thread_count` is what proves it.
//!
//! ## The raw frame is linear-light `Rgba16F`
//!
//! §16.7's certified artifact kinds are *raw frames*, *canonical PNG* and *WAV*.
//! The raw frame is the linear-light master: [`fmn_frame::PixelFormat::Rgba16F`],
//! straight alpha, output orientation (D-23 — row 0 is the top row, and there is
//! no vflip anywhere in the system). The canonical PNG is one table lookup away
//! through [`fmn_frame::convert::rgba16f_to_rgba8`], which is bit-exact by
//! construction because it does no arithmetic at all.
//!
//! Certified compositing runs in `f64` in a row accumulator and is narrowed
//! **once**, at writeback. The `f64 → f32 → f16` narrowing is a double rounding,
//! and it is recorded rather than hidden: it is deterministic on every platform
//! (both steps are IEEE round-to-nearest-even), and reaching a different `f16`
//! than a direct `f64 → f16` would require the accumulated value to sit within
//! `2⁻²⁴` of an `f16` midpoint — below the width at which the 8-bit output can
//! express a difference. The standard-only fast engine instead accumulates
//! colour and alpha in `f32`; its error is held to the versioned visual budget
//! below, while coverage and the ill-conditioned stroke-distance solve stay
//! `f64`.
//!
//! ## Point-transform SIMD audit: eliminate first, vectorize declined
//!
//! fm-4wt.1 measured the complete frame-job transform path at 8, 64, and 1,024
//! quadratic segments before retaining a kernel. The portable 1,024-segment
//! controls took 115.89 µs for translation, 193.51 µs for uniform scale, and
//! 380.20 µs for a general affine map. Uniform scale was needlessly rerunning
//! the transcendental-heavy arc-length quadrature even though it cannot change
//! normalized spans. `retains_normalized_arc_length` now admits
//! only structurally proven signed-axis similarities; the same uniform-scale
//! case fell to 140.71–141.89 µs (26.7–27.3% faster), while translation and the
//! general-affine control stayed effectively unchanged. The governed v3
//! artifact measured 129.09 µs after the change. On a loaded aarch64+NEON host,
//! the raw uniform-scale result moved from 65.68 µs to 53.76–57.27 µs; its
//! same-run general-affine control was 77.75–80.62 µs.
//!
//! A separate no-horizontal-reduction `std::simd` prototype then batched the
//! remaining array-of-structs matrix pass. At 1,024 segments, x86-64-v3 `f64x4`
//! took 4.74–4.81 µs versus 4.69–4.70 µs scalar. NEON `f64x2` sometimes won the
//! isolated kernel (2.91–3.22 µs versus 3.35–4.14 µs), but saved at most 1.23
//! µs against a 65.70 µs complete affine job and carried substantial host
//! variance. No slower or immaterial SIMD route is retained. The
//! `point_transform_{translation,uniform_scale,general_affine}_*` cases in the
//! existing `compositor` benchmark keep the exact production boundary
//! reproducible for a future structure-of-arrays layout.

use crate::arena::{AllocStats, FrameArena, PoolRange, PoolRangeError};
use crate::bin::{Binning, CLASS_INTERIOR, ScreenMap, Tiling, Viewport};
use crate::cache::{CacheStats, PixelTileCache, PixelTileCacheError, TileWork};
use crate::fill::{
    self, FillKernel, GradientField, RowScratch, fill_is_flat, fill_rgba_at, fill_rgba_with_border,
};
use crate::hint::Hint;
use crate::plan::RenderPlan;
#[cfg(test)]
use crate::stroke::stroke_shade;
use crate::stroke::{self, JoinWedge, half_width_px, stroke_rgba_at};
use crate::table::{Segment, Style, reparameterize_arc_length, retains_normalized_arc_length};
pub use fmn_core::AaPolicy;
use fmn_core::color::{LinearRgba, PremulRgba};
use fmn_frame::{FrameBuffer, FrameError, FrameLayout, PixelFormat};
use fmn_hash::{Digest, Schema, Writer};
use std::simd::Simd;
use std::sync::{Condvar, Mutex, PoisonError};

pub(crate) trait ScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope;
}

pub(crate) struct NativeScopedSpawner;

impl ScopedSpawner for NativeScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope,
    {
        std::thread::Builder::new().spawn_scoped(scope, work)
    }
}

/// Start a complete scoped worker team before permitting any worker to run.
///
/// Already-started workers wait behind `start`; if a later spawn is refused,
/// every waiter is cancelled and the scope joins them before the typed error
/// reaches the caller. This keeps destination storage untouched on startup
/// failure without hiding panics from worker code itself.
pub(crate) fn run_scoped_workers<S, F>(
    workers: usize,
    spawner: &S,
    work: F,
) -> Result<(), FrameError>
where
    S: ScopedSpawner,
    F: Fn() + Sync,
{
    debug_assert!(workers > 1);
    let start = (Mutex::new(None), Condvar::new());
    let work = &work;
    let failed_after = std::thread::scope(|scope| {
        let mut failure = None;
        for spawned in 0..workers {
            let start = &start;
            let result = spawner.spawn(scope, move || {
                let mut state = start.0.lock().unwrap_or_else(PoisonError::into_inner);
                while state.is_none() {
                    state = start.1.wait(state).unwrap_or_else(PoisonError::into_inner);
                }
                let run = *state == Some(true);
                drop(state);
                if run {
                    work();
                }
            });
            if result.is_err() {
                failure = Some(spawned);
                break;
            }
        }
        *start.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(failure.is_none());
        start.1.notify_all();
        failure
    });
    failed_after.map_or(Ok(()), |spawned| {
        Err(FrameError::WorkerSpawnFailed {
            requested: workers,
            spawned,
        })
    })
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{NativeScopedSpawner, ScopedSpawner};
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(crate) struct RefusingScopedSpawner {
        refuse_at: usize,
        attempts: AtomicUsize,
    }

    impl RefusingScopedSpawner {
        pub(crate) const fn new(refuse_at: usize) -> Self {
            Self {
                refuse_at,
                attempts: AtomicUsize::new(0),
            }
        }

        pub(crate) fn attempts(&self) -> usize {
            self.attempts.load(Ordering::Relaxed)
        }
    }

    impl ScopedSpawner for RefusingScopedSpawner {
        fn spawn<'scope, 'env: 'scope, F>(
            &self,
            scope: &'scope std::thread::Scope<'scope, 'env>,
            work: F,
        ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
        where
            F: FnOnce() + Send + 'scope,
        {
            let attempt = self.attempts.fetch_add(1, Ordering::Relaxed);
            if attempt == self.refuse_at {
                return Err(std::io::ErrorKind::WouldBlock.into());
            }
            NativeScopedSpawner.spawn(scope, work)
        }
    }
}

// A binary that requires an improvised subset must not journal itself as the
// portable tier. ADR-0016 supports only the exact SUITE.lock feature sets.
#[cfg(all(
    target_arch = "x86_64",
    any(
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    ),
    not(any(
        all(
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma",
            not(target_feature = "avx512f")
        ),
        all(
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        )
    ))
))]
compile_error!(
    "unsupported partial x86 SIMD tier; use SUITE.lock's portable, x86-64-v3, or x86-64-v4 flags"
);

// --------------------------------------------------------------- the identity

/// The schema family for engine-identity and frame documents.
pub const ENGINE_SCHEMA: Schema = Schema::new(*b"FMNE", 1, 2, 0);

/// The schema family for a canonical raw-frame document.
pub const FRAME_SCHEMA: Schema = Schema::new(*b"FMNE", 2, 1, 0);

/// The **semantic renderer version**: C7's first component.
///
/// Bumped when the renderer's *meaning* changes — a different coverage
/// definition, a different composite, a different colour pipeline — so that two
/// provenance manifests cannot claim equal inputs across a change that moves
/// every pixel. It is deliberately not the crate version: a refactor that cannot
/// move a bit must not invalidate a manifest, and a one-line change to the AA
/// profile must.
///
/// 7: the 2D screen map sends object +Y up the frame (fm-sq8.9); version 6
/// rendered every 2D frame vertically mirrored.
pub const RENDERER_VERSION: u32 = 7;

/// Boundary-sheet count at which an adaptive cell escalates to 2×2 samples.
///
/// G0-2 fixed the criterion — more than one boundary in a cell — and assigned
/// the numeric threshold to fm-gmr. Two is the first count for which the
/// ordinary one-edge analytic model is no longer sufficient.
pub const AA_COMPLEX_2X_CROSSINGS: u8 = 2;

/// Boundary-sheet count at which an adaptive cell escalates from 2×2 to 4×4.
///
/// Four independent crossings identify the dense/cusped end of the measured
/// corpus. The count is deliberately an integer geometric fact, not a
/// machine- or frame-load-dependent heuristic.
pub const AA_COMPLEX_4X_CROSSINGS: u8 = 4;

/// Minimum full stroke width, in AA-band widths, that contributes complexity.
///
/// G0-2 measured cap/join distinctions below roughly two AA bands as wholly
/// contained by the analytic smoothstep. Supersampling those strokes buys no
/// new silhouette information, so they cannot promote an otherwise-simple
/// cell merely by overlapping.
pub const AA_STROKE_COMPLEX_MIN_WIDTH_BANDS: f64 = 2.0;

/// Version-1 maximum linear channel error versus forced 4× on the W5 corpus.
///
/// The measured adaptive result is `0.219482421875`; the blocking budget leaves
/// roughly five percent headroom for corpus growth while still catching a
/// changed edge profile or a missed complex-cell class.
pub const AA_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR: f64 = 0.23;

/// Version-1 RMS linear channel error versus forced 4× on the W5 corpus.
///
/// The measured adaptive result is `0.008526512734628627`.
pub const AA_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR: f64 = 0.009;

/// Version-1 maximum linear channel error for fast CPU versus certified.
///
/// The locked three-frame corpus measures `0.087890625` at one pentagram edge;
/// the blocking budget leaves roughly five percent headroom. A controlled
/// canonical-AA run measures mixed precision alone at `0.00048828125`, so the
/// larger residual belongs to standard mode's adaptive edge sampling rather
/// than an f32 distance solve.
pub const FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR: f64 = 0.092;

/// Version-1 RMS linear channel error for fast CPU versus certified.
///
/// The locked corpus measures `0.0004011347492380775`; the blocking budget
/// leaves roughly five percent headroom.
pub const FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR: f64 = 0.000_42;

/// Version-1 minimum global sRGB-luma SSIM for fast CPU versus certified.
///
/// The locked corpus minimum is `0.9999983928862303`. SSIM is the perceptual
/// half of §16.3's engine-equivalence budget; max and RMS channel error remain
/// the local-error tripwires.
pub const FAST_VISUAL_BUDGET_V1_MIN_SSIM: f64 = 0.999_99;

/// Which execution engine produced a frame (§10.1, C7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineKind {
    /// The certified CPU engine: canonical arithmetic, the definition of the bits.
    CertifiedCpu,
    /// The fast CPU engine: mixed precision, SIMD tiers, FMA permitted (§17.3).
    FastCpu,
    /// The Metal annex engine (§10.7) — `standard` only, never certified.
    Metal,
    /// The CUDA annex engine (§10.7) — `standard` only, never certified.
    Cuda,
}

impl EngineKind {
    /// The stable name journaled into the input closure.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CertifiedCpu => "certified-cpu",
            Self::FastCpu => "fast-cpu",
            Self::Metal => "metal",
            Self::Cuda => "cuda",
        }
    }

    /// May this engine serve a `certified` run?
    ///
    /// §16.7's C7 says it plainly — "`certified` requires the certified CPU
    /// engine" — and D-18 makes GPU work in the certified path one of the three
    /// permanent refusals. The predicate exists so that rule is enforced by a
    /// match rather than by everyone remembering it.
    #[must_use]
    pub const fn certifiable(self) -> bool {
        matches!(self, Self::CertifiedCpu)
    }
}

/// The scalar definition and the one SIMD build tier this artifact offers
/// (§17.3, C3).
///
/// A distributed artifact contains exactly two arithmetic routes: [`Tier::Scalar`]
/// for the certified equivalence oracle, and the tier selected by the
/// SUITE.lock-governed crate flags. There is no per-call feature detection and
/// no artifact can claim a tier it was not compiled to execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    /// Portable scalar: the definition every other tier must reproduce.
    Scalar,
    /// The baseline artifact with no extra target-feature flags.
    #[cfg(not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma",
            not(target_feature = "avx512f")
        ),
        all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        ),
        all(target_arch = "aarch64", target_feature = "neon")
    )))]
    Portable,
    /// The x86-64-v3 artifact (`+avx2,+bmi2,+fma`).
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma",
        not(target_feature = "avx512f")
    ))]
    X86_64V3,
    /// The x86-64-v4 artifact (the four SUITE.lock AVX-512 features).
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    ))]
    X86_64V4,
    /// The aarch64 + NEON artifact.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    Aarch64Neon,
}

impl Tier {
    /// The stable name journaled into the input closure.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            #[cfg(not(any(
                all(
                    target_arch = "x86_64",
                    target_feature = "avx2",
                    target_feature = "bmi2",
                    target_feature = "fma",
                    not(target_feature = "avx512f")
                ),
                all(
                    target_arch = "x86_64",
                    target_feature = "avx512f",
                    target_feature = "avx512bw",
                    target_feature = "avx512dq",
                    target_feature = "avx512vl"
                ),
                all(target_arch = "aarch64", target_feature = "neon")
            )))]
            Self::Portable => "portable",
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "bmi2",
                target_feature = "fma",
                not(target_feature = "avx512f")
            ))]
            Self::X86_64V3 => "x86-64-v3",
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx512f",
                target_feature = "avx512bw",
                target_feature = "avx512dq",
                target_feature = "avx512vl"
            ))]
            Self::X86_64V4 => "x86-64-v4",
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            Self::Aarch64Neon => "aarch64-neon",
        }
    }

    /// Every tier this build offers: the scalar oracle, then the built tier.
    ///
    /// The certified harness sweeps this, so a tier that lands without being
    /// listed here is a tier nothing checks.
    pub const ALL: &'static [Tier] = &[
        Tier::Scalar,
        #[cfg(not(any(
            all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "bmi2",
                target_feature = "fma",
                not(target_feature = "avx512f")
            ),
            all(
                target_arch = "x86_64",
                target_feature = "avx512f",
                target_feature = "avx512bw",
                target_feature = "avx512dq",
                target_feature = "avx512vl"
            ),
            all(target_arch = "aarch64", target_feature = "neon")
        )))]
        Tier::Portable,
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma",
            not(target_feature = "avx512f")
        ))]
        Tier::X86_64V3,
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        ))]
        Tier::X86_64V4,
        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        Tier::Aarch64Neon,
    ];

    /// The SIMD tier selected by this artifact's crate-wide build flags.
    #[cfg(not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma",
            not(target_feature = "avx512f")
        ),
        all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        ),
        all(target_arch = "aarch64", target_feature = "neon")
    )))]
    pub const COMPILED: Tier = Tier::Portable;

    /// The SIMD tier selected by this x86-64-v3 artifact.
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma",
        not(target_feature = "avx512f")
    ))]
    pub const COMPILED: Tier = Tier::X86_64V3;

    /// The SIMD tier selected by this x86-64-v4 artifact.
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    ))]
    pub const COMPILED: Tier = Tier::X86_64V4;

    /// The SIMD tier selected by this aarch64 + NEON artifact.
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    pub const COMPILED: Tier = Tier::Aarch64Neon;
}

/// C7's "execution-engine and backend identities", as a hashable document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EngineIdentity {
    /// Which engine.
    pub engine: EngineKind,
    /// Which SIMD build tier.
    pub tier: Tier,
    /// [`RENDERER_VERSION`] at the time of the render.
    pub renderer_version: u32,
}

impl EngineIdentity {
    /// This build's certified identity.
    #[must_use]
    pub const fn certified() -> EngineIdentity {
        EngineIdentity {
            engine: EngineKind::CertifiedCpu,
            tier: Tier::Scalar,
            renderer_version: RENDERER_VERSION,
        }
    }

    /// This artifact's standard-mode default: fast CPU on its compiled tier.
    #[must_use]
    pub const fn fast() -> EngineIdentity {
        EngineIdentity {
            engine: EngineKind::FastCpu,
            tier: Tier::COMPILED,
            renderer_version: RENDERER_VERSION,
        }
    }

    /// This artifact's standard-only Metal-annex identity.
    ///
    /// The host build tier remains part of C7 because it prepares the derived
    /// device layout. The backend's device, math mode, and pipeline identity
    /// are journaled by the Metal renderer itself.
    #[must_use]
    pub const fn metal() -> EngineIdentity {
        EngineIdentity {
            engine: EngineKind::Metal,
            tier: Tier::COMPILED,
            renderer_version: RENDERER_VERSION,
        }
    }
}

impl Default for EngineIdentity {
    fn default() -> Self {
        Self::fast()
    }
}

impl EngineIdentity {
    /// The canonical one-string form of the identity the certified input
    /// closure journals field-wise (`journal` writes engine name, tier
    /// name, and renderer version): `<engine>:<tier>:<renderer_version>`.
    /// FMTL/1 combines this with Choreo's reconstruction-law version in
    /// its `engine_version`; its player refuses either identity mismatch.
    #[must_use]
    pub fn closure_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.engine.name(),
            self.tier.name(),
            self.renderer_version
        )
    }
}

/// §10.4's aggregate class for one fine tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageClass {
    /// No visible command reaches the tile.
    Empty,
    /// Every visible command is a classified interior fill.
    FullyCovered,
    /// An edge reaches the tile, but every cell contains at most one boundary.
    SimpleEdge,
    /// At least one cell crosses the adaptive 2× threshold.
    ComplexEdge,
}

/// Deterministic instrumentation for one adaptive-AA render.
///
/// Counts are accumulated per write-disjoint band and merged after workers
/// finish, so the report is as thread-count-independent as the frame itself.
/// `native_cells` includes the native classifier pass for adaptive complex
/// cells; `ssaa*_cells` records the fused replacement work on top of it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AaStats {
    /// Native output cells written.
    pub output_cells: u64,
    /// Native-resolution cell passes, including background/interior fast paths.
    pub native_cells: u64,
    /// Cells immediately resolved from 2×2 subcells.
    pub ssaa2x_cells: u64,
    /// Cells immediately resolved from 4×4 subcells.
    pub ssaa4x_cells: u64,
    /// Fine tiles with no visible command.
    pub empty_tiles: u64,
    /// Fine tiles containing only classified interior fills.
    pub fully_covered_tiles: u64,
    /// Fine tiles whose edge cells stay below the 2× threshold.
    pub simple_edge_tiles: u64,
    /// Fine tiles containing at least one escalated cell.
    pub complex_edge_tiles: u64,
}

/// Work evidence from one retained-compositor render.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CachedRenderStats {
    /// Fine-tile reuse and invalidation counts.
    pub cache: CacheStats,
    /// Coverage and sampling work that actually executed. Reused tiles do not
    /// contribute to these counters because their rasterizer was bypassed.
    pub raster: AaStats,
}

/// A retained-compositor render failed before it could publish a frame.
#[derive(Debug)]
pub enum CachedRenderError {
    /// Destination layout or worker execution failed.
    Frame(FrameError),
    /// Tile planning or payload retention failed.
    Cache(PixelTileCacheError),
}

impl std::fmt::Display for CachedRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Frame(error) => error.fmt(f),
            Self::Cache(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CachedRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::Cache(error) => Some(error),
        }
    }
}

impl From<FrameError> for CachedRenderError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<PixelTileCacheError> for CachedRenderError {
    fn from(error: PixelTileCacheError) -> Self {
        Self::Cache(error)
    }
}

impl AaStats {
    /// Native-equivalent cell-sample units.
    ///
    /// A native pass counts one, 2×2 counts four and 4×4 counts sixteen.
    /// Adaptive's native classifier pass remains included for cells it later
    /// replaces. This intentionally measures coverage-grid work, not
    /// per-command kernel cost or wall time.
    #[must_use]
    pub fn sample_evaluations(&self) -> u64 {
        self.native_cells
            .saturating_add(self.ssaa2x_cells.saturating_mul(4))
            .saturating_add(self.ssaa4x_cells.saturating_mul(16))
    }

    /// The work a full-frame forced-4× comparison performs.
    #[must_use]
    pub fn forced_4x_evaluations(&self) -> u64 {
        self.output_cells.saturating_mul(16)
    }

    /// Fraction of coverage/sample work avoided versus full-frame forced 4×.
    ///
    /// A pathological frame may return a negative value: adaptive first
    /// evaluates the native classifier, so escalating every cell to 4× costs
    /// `17/16` of forced 4×. Reporting that honestly is more useful than
    /// clamping an instrumentation result into looking successful.
    #[must_use]
    pub fn work_reduction_vs_forced_4x(&self) -> f64 {
        let forced = self.forced_4x_evaluations();
        if forced == 0 {
            0.0
        } else {
            1.0 - self.sample_evaluations() as f64 / forced as f64
        }
    }

    /// Fine tiles represented by the four class counters.
    #[must_use]
    pub fn classified_tiles(&self) -> u64 {
        self.empty_tiles
            .saturating_add(self.fully_covered_tiles)
            .saturating_add(self.simple_edge_tiles)
            .saturating_add(self.complex_edge_tiles)
    }

    fn merge(&mut self, other: AaStats) {
        self.output_cells = self.output_cells.saturating_add(other.output_cells);
        self.native_cells = self.native_cells.saturating_add(other.native_cells);
        self.ssaa2x_cells = self.ssaa2x_cells.saturating_add(other.ssaa2x_cells);
        self.ssaa4x_cells = self.ssaa4x_cells.saturating_add(other.ssaa4x_cells);
        self.empty_tiles = self.empty_tiles.saturating_add(other.empty_tiles);
        self.fully_covered_tiles = self
            .fully_covered_tiles
            .saturating_add(other.fully_covered_tiles);
        self.simple_edge_tiles = self
            .simple_edge_tiles
            .saturating_add(other.simple_edge_tiles);
        self.complex_edge_tiles = self
            .complex_edge_tiles
            .saturating_add(other.complex_edge_tiles);
    }

    fn count_tile(&mut self, class: CoverageClass) {
        match class {
            CoverageClass::Empty => self.empty_tiles = self.empty_tiles.saturating_add(1),
            CoverageClass::FullyCovered => {
                self.fully_covered_tiles = self.fully_covered_tiles.saturating_add(1);
            }
            CoverageClass::SimpleEdge => {
                self.simple_edge_tiles = self.simple_edge_tiles.saturating_add(1);
            }
            CoverageClass::ComplexEdge => {
                self.complex_edge_tiles = self.complex_edge_tiles.saturating_add(1);
            }
        }
    }
}

/// The rest of what a frame's bits depend on: C10's **declared certified
/// configuration**, plus the semantic constants §10.2/§10.3 fixed by
/// measurement.
///
/// The constants are journaled explicitly even though they are `const` in this
/// crate, and the reason is what a manifest is *for*. C2 already hashes the
/// engine commit, so a changed constant does move the closure digest — but only
/// as an opaque commit difference. Naming them makes a manifest say *which*
/// number moved, which is the difference between a provenance record and a
/// receipt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameConfig {
    /// The pixel rectangle.
    pub viewport: Viewport,
    /// The object→screen mapping.
    pub map: ScreenMap,
    /// The background, linear light, straight alpha. Opaque for ordinary
    /// renders; a zero alpha is `--transparent`.
    pub background: LinearRgba,
    /// Standard-mode coverage policy. Certified jobs always execute the
    /// canonical analytic path, whatever A/B policy was requested.
    pub aa: AaPolicy,
}

impl FrameConfig {
    /// A frame of `viewport` on the default map over an opaque background.
    #[must_use]
    pub fn new(viewport: Viewport, map: ScreenMap, background: LinearRgba) -> FrameConfig {
        FrameConfig {
            viewport,
            map,
            background,
            aa: AaPolicy::Adaptive,
        }
    }

    /// Select the standard-mode adaptive/forced-AA policy.
    #[must_use]
    pub fn with_aa_policy(mut self, aa: AaPolicy) -> FrameConfig {
        self.aa = aa;
        self
    }

    /// The layout of the raw certified frame this config renders into.
    ///
    /// Tight rows: the certified raw frame carries no stride padding, so its
    /// bytes are a function of the picture and nothing else. A padded layout is
    /// legal for an intermediate ([`FrameJob::render_into`] accepts one), but
    /// [`frame_digest`] hashes payload rows precisely so the choice cannot reach
    /// a golden either way.
    ///
    /// # Errors
    /// [`FrameError::ZeroDimension`] for an empty viewport.
    pub fn layout(&self) -> Result<FrameLayout, FrameError> {
        FrameLayout::tight(
            PixelFormat::Rgba16F,
            self.viewport.width,
            self.viewport.height,
        )
    }
}

/// The engine's full identity document, hashed into the input closure (§10.5f,
/// §16.7 C7 and C10).
///
/// Ordered, versioned and canonical: [`fmn_hash::Writer`] pins the field order
/// and canonicalizes floats at the boundary, so `-0.0` and `+0.0` cannot make
/// two identical configurations hash differently.
#[must_use]
pub fn journal(identity: EngineIdentity, config: &FrameConfig, tiling: Tiling) -> Vec<u8> {
    let mut w = Writer::new(ENGINE_SCHEMA);

    // C7 — execution-engine and backend identities.
    w.put_str(identity.engine.name());
    w.put_bool(identity.engine.certifiable());
    w.put_str(identity.tier.name());
    w.put_u32(identity.renderer_version);

    // C10 — the declared configuration.
    w.put_u32(config.viewport.width);
    w.put_u32(config.viewport.height);
    w.put_u32(tiling.macro_tile);
    w.put_u32(tiling.fine_tile);
    w.put_f64(config.map.scale);
    w.put_f64(config.map.origin[0]);
    w.put_f64(config.map.origin[1]);
    w.put_bool(config.map.y_up);
    w.put_f64(config.background.r);
    w.put_f64(config.background.g);
    w.put_f64(config.background.b);
    w.put_f64(config.background.a);
    w.put_str(match config.aa {
        AaPolicy::Adaptive => "adaptive",
        AaPolicy::Ssaa2x => "ssaa2x",
        AaPolicy::Ssaa4x => "ssaa4x",
    });
    // The format's **name**, not `as u32`. A discriminant is a property of
    // fmn-frame's declaration order, so reordering that enum would silently move
    // every manifest digest ever issued — and a closure digest whose meaning can
    // change without anyone editing it is worse than no digest. `cache.rs` may
    // use the discriminant because a tile key lives for one run; a provenance
    // manifest is durable (§16.7, D-17).
    w.put_str(RAW_FRAME_FORMAT_NAME);

    // C10 — the semantic constants. Measured, not tuned; see each one's own
    // documentation for the measurement that fixed it.
    w.put_u64(fill::GRADIENT_STATIONS as u64);
    w.put_f64(fill::HINT_BUDGET_PX);
    w.put_u32(fill::FLATTEN_MAX_DEPTH);
    w.put_f64(stroke::MITER_LIMIT);
    w.put_f64(crate::hint::NEARLY_LINEAR_PX);
    w.put_u32(u32::from(AA_COMPLEX_2X_CROSSINGS));
    w.put_u32(u32::from(AA_COMPLEX_4X_CROSSINGS));
    w.put_f64(AA_STROKE_COMPLEX_MIN_WIDTH_BANDS);

    w.finish().expect("the identity document fits any limit")
}

/// The digest of [`journal`] — the value C7 and C10 contribute to the closure.
#[must_use]
pub fn journal_digest(identity: EngineIdentity, config: &FrameConfig, tiling: Tiling) -> Digest {
    fmn_hash::sha256(&journal(identity, config, tiling))
}

/// Why a frame could not be reduced to its canonical document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameEncodeError {
    /// The buffer has more than one plane, so a single-plane document would
    /// describe the luma and silently drop the chroma.
    MultiPlane {
        /// The format that was refused.
        format: PixelFormat,
    },
    /// The canonical envelope refused the document (a limit, in practice).
    Serial(fmn_hash::SerialError),
}

impl std::fmt::Display for FrameEncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MultiPlane { format } => write!(
                f,
                "{format:?} has {} planes; a canonical frame document describes one",
                format.plane_count()
            ),
            Self::Serial(e) => write!(f, "canonical frame document: {e:?}"),
        }
    }
}

impl std::error::Error for FrameEncodeError {}

/// The canonical digest of a rendered raw frame.
///
/// Hashes the geometry and the **payload rows only** — never the whole
/// allocation. A pooled [`FrameBuffer`] (PG-6's steady state) carries whatever a
/// previous frame left in its stride padding, and hashing that would make a
/// golden depend on pool history rather than on the picture. Rows go through
/// [`fmn_hash::Writer`], so the digest is a function of a declared schema rather
/// than of a `Vec<u8>`'s layout.
///
/// # Errors
/// Propagates the envelope's limit errors for a frame larger than the writer
/// admits.
pub fn frame_digest(buffer: &FrameBuffer) -> Result<Digest, FrameEncodeError> {
    Ok(fmn_hash::sha256(&encode_frame(buffer)?))
}

/// The canonical byte document a [`frame_digest`] is taken over — the form the
/// self-golden rig locks.
///
/// The document is materialized rather than streamed, which puts a ceiling on it:
/// [`fmn_hash::Limits::DEFAULT`] admits 256 MiB, so `Rgba16F` frames up to about
/// 4K (66 MB) encode comfortably and 8K (265 MB) does not. That is a deliberate
/// trade for the versioned envelope — a streamed digest would have no schema and
/// no float canonicalization — and it is stated here because the failure is a
/// typed error at an unusual resolution rather than anything a test would meet.
///
/// # Errors
/// [`FrameEncodeError::MultiPlane`] for a planar format;
/// [`FrameEncodeError::Serial`] for a frame past the envelope's limits.
pub fn encode_frame(buffer: &FrameBuffer) -> Result<Vec<u8>, FrameEncodeError> {
    let layout = buffer.layout();
    // Single-plane only, and refused rather than guessed. This walks plane 0, so
    // handing it an NV12 or P010 buffer would hash the luma and silently drop the
    // chroma — a document that looks like a frame and is not one. The certified
    // raw frame is always `Rgba16F`; `Rgba8` is admitted because it is the
    // canonical-PNG payload and locking that is the obvious next want.
    let format = layout.format();
    if format.plane_count() != 1 {
        return Err(FrameEncodeError::MultiPlane { format });
    }
    let width = layout.width() as usize;
    let height = layout.height() as usize;
    let stride = layout.stride(0);
    // The payload width the format itself declares, rather than a table here
    // that could drift from it.
    let row_bytes = format.min_row_bytes(layout.width(), 0).unwrap_or(width * 4);
    let mut w = Writer::new(FRAME_SCHEMA);
    w.put_u32(layout.width());
    w.put_u32(layout.height());
    w.put_str(format_name(format));
    let plane = buffer.plane(0);
    for y in 0..height {
        w.put_bytes(&plane[y * stride..y * stride + row_bytes]);
    }
    w.finish().map_err(FrameEncodeError::Serial)
}

/// The certified raw frame's pixel format, by name.
///
/// The engine's output format is a constant of the engine rather than a knob, so
/// it enters the closure as a fixed name.
pub const RAW_FRAME_FORMAT_NAME: &str = "rgba16f";

/// A stable name for a pixel format, for documents that outlive a build.
const fn format_name(format: PixelFormat) -> &'static str {
    match format {
        PixelFormat::Rgba8 => "rgba8",
        PixelFormat::Bgra8 => "bgra8",
        PixelFormat::Rgba16F => RAW_FRAME_FORMAT_NAME,
        PixelFormat::Nv12 => "nv12",
        PixelFormat::P010 => "p010",
    }
}

// ------------------------------------------------------------ the frame job

/// One instance's per-frame derived draw state.
///
/// Derived **once per frame, before any tile is touched**, for two reasons that
/// pull the same way. It is faster — `FillKernel::select` measures the compiled
/// outline's deviation, `join_wedges` solves every corner, and a
/// [`GradientField`] samples 64 stations, none of which may happen per tile per
/// row. And it makes the tile function take `&self`: nothing a worker touches is
/// mutable except its own scratch and its own band, which is what lets
/// [`FrameJob::render_into`] hand bands to threads with no synchronization
/// beyond the queue that assigns them.
#[derive(Debug, Clone)]
pub(crate) struct Draw {
    /// Index into [`RenderPlan::segments`] of this shape's first segment.
    pub(crate) first_segment: u32,
    /// How many segments the shape has.
    pub(crate) segment_count: u32,
    /// The shape index, for [`crate::fill::MonoTable::pieces_of`].
    pub(crate) shape: u32,
    /// The instance's screen translation.
    pub(crate) translate: [f64; 2],
    /// Frame-local world-space segments for a non-translation affine
    /// placement, as a range into the frame arena's segment pool. Pure
    /// translations keep borrowing the retained object-space table and pay
    /// only [`Draw::translate`].
    pub(crate) transformed_segments: Option<PoolRange>,
    /// Doubly-monotone pieces derived from [`Draw::transformed_segments`],
    /// as a range into the frame arena's piece pool.
    pub(crate) transformed_pieces: Option<PoolRange>,
    /// The still-valid semantic hint proves every segment is a monotone line.
    ///
    /// This is stronger than inferring straightness from RecordBuffer `f32`
    /// coordinates after they have been widened to `f64`: the latter retains
    /// small quantization bends in paths constructed as lines.
    #[cfg(feature = "metal")]
    pub(crate) straight_segments: bool,
    /// The interned style, copied so the tile loop indexes no table.
    pub(crate) style: Style,
    /// The hinted fill route, or [`FillKernel::General`].
    pub(crate) kernel: FillKernel,
    /// The joint overrides, as a range into the frame arena's wedge pool;
    /// empty for the round settings (ADR-0012).
    pub(crate) joins: PoolRange,
    /// Per-segment slabs and arc lengths derived for this styled occurrence,
    /// as a range into the frame arena's prepared-segment pool.
    pub(crate) stroke: Option<StrokeRef>,
    /// The interior colour field — `None` when the fill is flat, which is the
    /// overwhelming majority and the case that must not pay for the field.
    pub(crate) field: Option<FieldRef>,
    /// The one fill colour, when the fill is flat.
    pub(crate) flat_fill: Option<[f32; 4]>,
    /// Screen AABB of the outline hull: rows outside it have zero fill coverage.
    pub(crate) fill_slab: [f64; 4],
    /// Does this instance contribute a fill pass at all?
    pub(crate) draws_fill: bool,
    /// Does it contribute a stroke pass?
    pub(crate) draws_stroke: bool,
}

/// The arena coordinates of a draw's [`stroke::PreparedStroke`].
///
/// The stroke's prepared segments live in the frame arena; what the draw
/// carries is where they are and the aggregate slab that was derived with
/// them. [`FrameJob::stroke_of`] reconstitutes the borrowed view.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrokeRef {
    /// Range into the arena's prepared-segment pool.
    pub(crate) segments: PoolRange,
    /// The aggregate conservative slab.
    pub(crate) slab: [f64; 4],
    /// Whether [`stroke::segments_are_x_sorted`] accepted the segments.
    pub(crate) x_sorted: bool,
}

/// The arena coordinates of a draw's [`GradientField`] stations.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldRef {
    /// Range into the arena's station-position pool.
    pub(crate) points: PoolRange,
    /// Range into the arena's station-parameter pool, index-aligned.
    pub(crate) params: PoolRange,
    /// Range into the arena's explicit next-station pool.
    pub(crate) next: PoolRange,
    /// Range into the arena's explicit boundary-edge parameter pool.
    pub(crate) edge_params: PoolRange,
}

/// A frame was assembled from derived artifacts that do not share one input
/// closure.
///
/// Each variant names the stale axis directly so callers can rebuild only the
/// artifact whose key moved. No mismatch is recoverable by rendering anyway:
/// doing so would return a valid-looking, deterministically wrong frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameJobError {
    /// The caller supplied an identity for renderer semantics other than the
    /// code that will execute.
    RendererVersionMismatch {
        /// Version named by the requested identity.
        requested: u32,
        /// Version implemented by this artifact.
        compiled: u32,
    },
    /// The requested engine has no implementation in this artifact.
    UnsupportedEngine {
        /// The engine that was requested.
        engine: EngineKind,
    },
    /// The monotone pieces were transformed under another screen mapping.
    MonoMapMismatch,
    /// The monotone table's shape-indexed geometry is stale.
    MonoPlanMismatch,
    /// Frame-local affine geometry exceeded a monotone-table resource or
    /// representation boundary.
    MonoTable(fill::MonoTableError),
    /// A frame-local pool outgrew the compact coordinates carried by prepared
    /// draws. No range is truncated or consumed after this refusal.
    ArenaIndexCapacityExceeded {
        /// Pool whose coordinates could not be represented.
        resource: &'static str,
        /// First requested row.
        start: u64,
        /// Number of requested rows.
        len: u64,
    },
    /// The tile command lists were scattered under another screen mapping.
    BinningMapMismatch,
    /// The tile command lists cover a different exact viewport.
    BinningViewportMismatch,
    /// The tile command indices name a different painter-ordered plan.
    BinningPlanMismatch,
    /// The affine-only frame job received vector semantics that require
    /// [`crate::three_d::ThreeDJob`]'s captured camera and shared painter
    /// sequence.
    CameraProjectionRequired {
        /// Painter-sequence instance that requires the camera route.
        instance: u32,
    },
}

impl std::fmt::Display for FrameJobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RendererVersionMismatch {
                requested,
                compiled,
            } => write!(
                f,
                "renderer identity names version {requested}, but this artifact implements {compiled}"
            ),
            Self::UnsupportedEngine { engine } => {
                write!(
                    f,
                    "{} has no rendering backend in this artifact",
                    engine.name()
                )
            }
            Self::MonoMapMismatch => {
                f.write_str("monotone table screen map does not match the frame")
            }
            Self::MonoPlanMismatch => {
                f.write_str("monotone table geometry does not match the render plan")
            }
            Self::MonoTable(error) => {
                write!(f, "could not derive frame-local monotone pieces: {error}")
            }
            Self::ArenaIndexCapacityExceeded {
                resource,
                start,
                len,
            } => write!(
                f,
                "frame-local {resource} range at {start} with length {len} exceeds u32 coordinates"
            ),
            Self::BinningMapMismatch => f.write_str("binning screen map does not match the frame"),
            Self::BinningViewportMismatch => {
                f.write_str("binning viewport does not match the frame")
            }
            Self::BinningPlanMismatch => f.write_str("binning does not match the render plan"),
            Self::CameraProjectionRequired { instance } => write!(
                f,
                "retained instance {instance} requires camera projection; render it through ThreeDJob"
            ),
        }
    }
}

impl From<PoolRangeError> for FrameJobError {
    fn from(error: PoolRangeError) -> Self {
        Self::ArenaIndexCapacityExceeded {
            resource: error.resource,
            start: error.start,
            len: error.len,
        }
    }
}

impl std::error::Error for FrameJobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MonoTable(error) => Some(error),
            _ => None,
        }
    }
}

/// A frame, compiled and ready to rasterize.
///
/// Borrows the retained plan, its derived monotone table and its binning;
/// owns only the per-instance derivation above. Construction is the serial
/// front-end; [`FrameJob::render_into`] is the parallel back-end, and the split
/// between them is exactly §9.3's FramePacket boundary one layer down.
///
/// The derivation's *storage* lives in a [`FrameArena`] (PG-6, fm-e9h): every
/// `Vec` a draw used to own is now a range into one of the arena's typed bump
/// pools, so a job that shares an arena across frames
/// ([`FrameJob::new_in`]) allocates nothing once the arena is warm. The
/// convenience constructors ([`FrameJob::new`], [`FrameJob::with_identity`])
/// own a fresh arena per job instead — same bits, same single representation,
/// no caller-visible arena for one-shot renders.
#[derive(Debug)]
pub struct FrameJob<'a> {
    plan: &'a RenderPlan,
    mono: &'a fill::MonoTable,
    binning: &'a Binning,
    config: FrameConfig,
    identity: EngineIdentity,
    /// The bump pools the draw derivation lives in, plus the worker pool.
    arena: ArenaStorage<'a>,
    /// The arena range of the draw list. Index-aligned with
    /// `plan.shapes().instances()`; `None` where the instance contributes no
    /// pass. See [`FrameJob::with_identity`].
    draws: PoolRange,
    cols: u32,
}

/// Who owns the arena: the job (one-shot renders) or the caller (the
/// reused-across-frames PG-6 path).
#[derive(Debug)]
enum ArenaStorage<'a> {
    /// A fresh arena per job; today's one-shot behaviour. Boxed so the
    /// shared-arena path does not pay for the owned variant's size.
    Owned(Box<FrameArena>),
    /// The caller's arena, reused across frames; `begin_frame` ran at
    /// construction.
    Shared(&'a FrameArena),
}

impl FrameJob<'_> {
    /// The arena, regardless of who owns it.
    fn arena(&self) -> &FrameArena {
        match &self.arena {
            ArenaStorage::Owned(arena) => arena,
            ArenaStorage::Shared(arena) => arena,
        }
    }
}

/// The render path after certified-mode normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderAa {
    /// The canonical analytic definition; no adaptive classifier executes.
    Canonical,
    /// Native analytic classifier plus fused resolve on complex cells.
    Adaptive,
    /// Full-frame forced sampling for A/B and debugging.
    Forced(u32),
}

/// One fixed subcell in a native output pixel.
#[derive(Debug, Clone, Copy)]
struct Subcell {
    px: u32,
    py: u32,
    samples: u32,
    x: u32,
    y: u32,
}

impl Subcell {
    fn centre(self) -> [f64; 2] {
        let inverse = 1.0 / f64::from(self.samples);
        [
            f64::from(self.px) + (f64::from(self.x) + 0.5) * inverse,
            f64::from(self.py) + (f64::from(self.y) + 0.5) * inverse,
        ]
    }
}

impl<'a> FrameJob<'a> {
    /// Compile a frame from a synchronized plan, its monotone table and its
    /// binning.
    ///
    /// All inputs are checked against collision-resistant content identities and
    /// exact map/viewport values. Carrying them separately is what lets a
    /// retained plan skip rebuilding the ones whose axes did not move; checking
    /// their keys is what makes that reuse safe.
    ///
    /// # Errors
    /// [`FrameJobError`] names the first derived artifact whose plan, map, or
    /// viewport does not match.
    pub fn new(
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
    ) -> Result<FrameJob<'a>, FrameJobError> {
        Self::with_identity(plan, mono, binning, config, EngineIdentity::certified())
    }

    /// [`FrameJob::new`] with an explicit engine identity.
    ///
    /// The identity selects the certified refusal boundary as well as recording
    /// what the frame claims: [`EngineKind::CertifiedCpu`] always executes the
    /// canonical analytic path, while standard-only identities honor
    /// [`FrameConfig::aa`]. Every CPU route shares the semantic front-end;
    /// certified uses the scalar `f64` definition, while fast CPU substitutes
    /// its budgeted `f32` compositor. Recording which one ran is §10.5(f).
    ///
    /// # Errors
    /// [`FrameJobError::RendererVersionMismatch`] when the requested identity
    /// does not name this artifact's renderer semantics;
    /// [`FrameJobError::UnsupportedEngine`] for an annex backend that has not
    /// landed; otherwise see [`FrameJob::new`].
    pub fn with_identity(
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
        identity: EngineIdentity,
    ) -> Result<FrameJob<'a>, FrameJobError> {
        if matches!(identity.engine, EngineKind::Metal | EngineKind::Cuda) {
            return Err(FrameJobError::UnsupportedEngine {
                engine: identity.engine,
            });
        }
        let mut arena = FrameArena::new();
        arena.begin_frame();
        let (draws, cols) = Self::prepare(plan, mono, binning, config, identity, &mut arena)?;
        Ok(FrameJob {
            plan,
            mono,
            binning,
            config,
            identity,
            arena: ArenaStorage::Owned(Box::new(arena)),
            draws,
            cols,
        })
    }

    /// [`FrameJob::new`] with the caller's [`FrameArena`] — the PG-6 path.
    ///
    /// The arena is *reset* for this frame (every bump pool truncated, none
    /// of its buffers released) and then holds the whole draw derivation:
    /// the draw list, join wedges, prepared stroke segments, gradient
    /// stations, and any affine-transformed geometry. Reusing one arena
    /// across frames is what makes the steady state allocation-free; frame 1
    /// sizes every buffer, and [`FrameJob::allocation_stats`] reports it.
    ///
    /// # Errors
    /// See [`FrameJob::new`].
    pub fn new_in(
        arena: &'a mut FrameArena,
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
    ) -> Result<FrameJob<'a>, FrameJobError> {
        Self::with_identity_in(
            arena,
            plan,
            mono,
            binning,
            config,
            EngineIdentity::certified(),
        )
    }

    /// [`FrameJob::new_in`] with an explicit engine identity.
    ///
    /// # Errors
    /// [`FrameJobError::RendererVersionMismatch`] when the requested identity
    /// does not name this artifact's renderer semantics;
    /// [`FrameJobError::UnsupportedEngine`] for an annex backend that has not
    /// landed; otherwise see [`FrameJob::new`].
    pub fn with_identity_in(
        arena: &'a mut FrameArena,
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
        identity: EngineIdentity,
    ) -> Result<FrameJob<'a>, FrameJobError> {
        if matches!(identity.engine, EngineKind::Metal | EngineKind::Cuda) {
            return Err(FrameJobError::UnsupportedEngine {
                engine: identity.engine,
            });
        }
        arena.begin_frame();
        let (draws, cols) = Self::prepare(plan, mono, binning, config, identity, arena)?;
        Ok(FrameJob {
            plan,
            mono,
            binning,
            config,
            identity,
            arena: ArenaStorage::Shared(arena),
            draws,
            cols,
        })
    }

    /// Prepare the shared semantic front-end for the Metal-specific executor.
    ///
    /// Kept crate-private so a caller cannot construct a Metal-identified job
    /// and then accidentally ask the CPU-only [`FrameJob::render`] entry point
    /// to execute it. The public Metal renderer is the only truthful executor.
    #[cfg(feature = "metal")]
    pub(crate) fn for_metal(
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
    ) -> Result<FrameJob<'a>, FrameJobError> {
        let mut arena = FrameArena::new();
        arena.begin_frame();
        let identity = EngineIdentity::metal();
        let (draws, cols) = Self::prepare(plan, mono, binning, config, identity, &mut arena)?;
        Ok(FrameJob {
            plan,
            mono,
            binning,
            config,
            identity,
            arena: ArenaStorage::Owned(Box::new(arena)),
            draws,
            cols,
        })
    }

    /// The draw derivation, into `arena`'s bump pools. Returns the arena
    /// range of the draw list and the frame's fine-tile column count.
    fn prepare(
        plan: &'a RenderPlan,
        mono: &'a fill::MonoTable,
        binning: &'a Binning,
        config: FrameConfig,
        identity: EngineIdentity,
        arena: &mut FrameArena,
    ) -> Result<(PoolRange, u32), FrameJobError> {
        if identity.renderer_version != RENDERER_VERSION {
            return Err(FrameJobError::RendererVersionMismatch {
                requested: identity.renderer_version,
                compiled: RENDERER_VERSION,
            });
        }
        if mono.map() != config.map {
            return Err(FrameJobError::MonoMapMismatch);
        }
        if !mono.matches_plan(plan) {
            return Err(FrameJobError::MonoPlanMismatch);
        }
        if binning.map() != config.map {
            return Err(FrameJobError::BinningMapMismatch);
        }
        if binning.viewport() != config.viewport {
            return Err(FrameJobError::BinningViewportMismatch);
        }
        if !binning.matches_plan(plan) {
            return Err(FrameJobError::BinningPlanMismatch);
        }

        let map = config.map;
        let segments = plan.segments();
        let draws_start = arena.draws.len();

        for (instance_index, inst) in plan.shapes().instances().iter().enumerate() {
            // Every instance gets a slot, including the ones that draw nothing.
            // `Binning`'s command lists hold **instance** indices, so a compacted
            // draw list would silently pair each command with the wrong shape and
            // style from the first skipped instance onward. `None` is the slot
            // for "this instance contributes no pass"; it costs a word and it is
            // the difference between an index and a coincidence.
            let Some(shape) = plan.shapes().shape(inst.shape) else {
                arena.draws.put(None);
                continue;
            };
            let Some(style) = plan.styles().get(inst.style).cloned() else {
                arena.draws.put(None);
                continue;
            };
            let lo = shape.first_segment as usize;
            let hi = lo + shape.segment_count as usize;
            let segs = &segments[lo.min(segments.len())..hi.min(segments.len())];
            let camera_uniform = style.is_fixed_in_frame != 0.0
                || style.shading != [0.0; 3]
                || style
                    .clip_planes
                    .iter()
                    .flatten()
                    .any(|component| *component != 0.0)
                || style.scale_stroke_with_zoom
                || style.depth_test;
            let transformed_segments = if !inst.placement.is_translation() {
                let range = arena.segments.extend(
                    "transformed segment rows",
                    segs.iter().map(|segment| Segment {
                        p0: inst.placement.apply_vector(segment.p0),
                        p1: inst.placement.apply_vector(segment.p1),
                        p2: inst.placement.apply_vector(segment.p2),
                        s0: segment.s0,
                        s1: segment.s1,
                    }),
                )?;
                let transformed = arena.segments.slice_mut(range);
                // Exact signed-axis similarities preserve normalized spans;
                // general affine maps take the scalar arc-length oracle.
                // Translation stays out of either route so a large world
                // origin cannot perturb normalized spans through cancellation.
                if !retains_normalized_arc_length(inst.placement) {
                    reparameterize_arc_length(transformed);
                }
                let translation = inst.placement.translation();
                for segment in transformed.iter_mut() {
                    for point in [&mut segment.p0, &mut segment.p1, &mut segment.p2] {
                        for axis in 0..3 {
                            point[axis] += translation[axis];
                        }
                    }
                }
                Some(range)
            } else {
                None
            };
            let effective_segments: &[Segment] = match transformed_segments {
                Some(range) => arena.segments.slice(range),
                None => segs,
            };
            let placement_z = if transformed_segments.is_some() {
                0.0
            } else {
                inst.placement.translation()[2]
            };
            let leaves_view_plane = effective_segments.iter().any(|segment| {
                [segment.p0, segment.p1, segment.p2]
                    .iter()
                    .any(|point| point[2] != 0.0)
            }) || placement_z != 0.0;
            if camera_uniform || leaves_view_plane {
                return Err(FrameJobError::CameraProjectionRequired {
                    instance: instance_index as u32,
                });
            }
            let translate = if transformed_segments.is_some() {
                [0.0; 2]
            } else {
                fill::instance_translation(inst, map)
            };
            let transformed_pieces = if transformed_segments.is_some() {
                let start = arena.pieces.len();
                fill::MonoTable::pieces_for_segments_into(
                    &mut arena.pieces,
                    &mut arena.piece_curves,
                    effective_segments,
                    &shape.subpath_starts,
                    map,
                    fill::MonoTableLimits::default(),
                )
                .map_err(FrameJobError::MonoTable)?;
                Some(arena.pieces.range_from("monotone piece rows", start)?)
            } else {
                None
            };

            let draws_fill = style.draws_fill();
            let draws_stroke = style.draws_stroke();
            if !draws_fill && !draws_stroke {
                // An instance with no visible pass composites as the identity, so
                // skipping its work cannot change a byte — but it still holds its
                // index. Both halves of that sentence matter.
                arena.draws.put(None);
                continue;
            }

            let flat = fill_is_flat(&style);
            let straight_segments =
                !inst.hint_unsafe && matches!(shape.hint, Hint::Line | Hint::Polyline { .. });
            let kernel = if inst.hint_unsafe || !inst.placement.is_translation() {
                FillKernel::General
            } else {
                FillKernel::select(shape, segs, map, translate)
            };
            let joins = {
                let start = arena.joins.len();
                stroke::join_wedges_into(
                    &mut arena.joins,
                    &mut arena.join_pairs,
                    effective_segments,
                    &shape.subpath_starts,
                    &style,
                    map,
                    translate,
                );
                arena.joins.range_from("join rows", start)?
            };
            let stroke = if draws_stroke {
                let start = arena.stroke_segments.len();
                let slab = stroke::PreparedStroke::prepare_into(
                    &mut arena.stroke_segments,
                    effective_segments,
                    &style,
                    map,
                    translate,
                    straight_segments,
                );
                let segments = arena
                    .stroke_segments
                    .range_from("prepared stroke rows", start)?;
                Some(StrokeRef {
                    segments,
                    slab,
                    x_sorted: stroke::segments_are_x_sorted(
                        arena.stroke_segments.slice(segments),
                        slab,
                    ),
                })
            } else {
                None
            };
            let field = if draws_fill && !flat {
                let points_start = arena.gradient_points.len();
                let params_start = arena.gradient_params.len();
                let next_start = arena.gradient_next.len();
                let edge_params_start = arena.gradient_edge_params.len();
                if let Some(profile) = &style.fill_profile {
                    GradientField::build_profile_into(
                        &mut arena.gradient_points,
                        &mut arena.gradient_params,
                        &mut arena.gradient_next,
                        &mut arena.gradient_edge_params,
                        effective_segments,
                        &shape.subpath_starts,
                        map,
                        profile,
                    );
                } else {
                    GradientField::build_contours_into(
                        &mut arena.gradient_points,
                        &mut arena.gradient_params,
                        &mut arena.gradient_next,
                        &mut arena.gradient_edge_params,
                        effective_segments,
                        &shape.subpath_starts,
                        map,
                    );
                }
                Some(FieldRef {
                    points: arena
                        .gradient_points
                        .range_from("gradient point rows", points_start)?,
                    params: arena
                        .gradient_params
                        .range_from("gradient parameter rows", params_start)?,
                    next: arena
                        .gradient_next
                        .range_from("gradient next-station rows", next_start)?,
                    edge_params: arena
                        .gradient_edge_params
                        .range_from("gradient edge-parameter rows", edge_params_start)?,
                })
            } else {
                None
            };
            let fill_slab = hull_slab(effective_segments, map, translate);
            arena.draws.put(Some(Draw {
                first_segment: shape.first_segment,
                segment_count: shape.segment_count,
                shape: inst.shape,
                translate,
                transformed_segments,
                transformed_pieces,
                #[cfg(feature = "metal")]
                straight_segments,
                style: style.clone(),
                kernel,
                joins,
                stroke,
                field,
                flat_fill: if flat {
                    Some(fill_rgba_at(&style, 0.0))
                } else {
                    None
                },
                fill_slab,
                draws_fill,
                draws_stroke,
            }));
        }
        debug_assert_eq!(
            arena.draws.len(),
            plan.shapes().instances().len(),
            "the draw list must be index-aligned with the instance list"
        );

        let cols = config
            .viewport
            .width
            .div_ceil(binning.tiling().fine_tile.max(1));

        Ok((arena.draws.range_from("draw rows", draws_start)?, cols))
    }

    /// The engine identity this frame will claim.
    #[must_use]
    pub fn identity(&self) -> EngineIdentity {
        self.identity
    }

    /// The requested policy after applying certified's permanent analytic-path
    /// refusal.
    fn render_aa(&self) -> RenderAa {
        if self.identity.engine.certifiable() {
            return RenderAa::Canonical;
        }
        match self.config.aa {
            AaPolicy::Adaptive => RenderAa::Adaptive,
            AaPolicy::Ssaa2x => RenderAa::Forced(2),
            AaPolicy::Ssaa4x => RenderAa::Forced(4),
        }
    }

    /// This frame's contribution to the input closure (C7 + C10).
    #[must_use]
    pub fn journal_digest(&self) -> Digest {
        journal_digest(self.identity, &self.config, self.binning.tiling())
    }

    /// How many instances contribute a pass.
    ///
    /// Not `draws.len()`: that is the instance count, because the draw list is
    /// index-aligned with the instance list and carries a `None` for every
    /// instance that draws nothing.
    #[must_use]
    pub fn draw_count(&self) -> usize {
        self.draws().iter().flatten().count()
    }

    /// The draw list, index-aligned with the instance list.
    ///
    /// The list lives in the arena; this resolves its range. Every engine
    /// read of a draw goes through here or the `_of` accessors below, so the
    /// range indirection cannot leak into the arithmetic.
    pub(crate) fn draws(&self) -> &[Option<Draw>] {
        self.arena().draws.slice(self.draws)
    }

    /// One draw's join wedges.
    pub(crate) fn joins_of(&self, rec: &Draw) -> &[JoinWedge] {
        self.arena().joins.slice(rec.joins)
    }

    /// One draw's prepared stroke, reconstituted as the borrowed view.
    pub(crate) fn stroke_of(&self, rec: &Draw) -> Option<stroke::PreparedStroke<'_>> {
        rec.stroke.map(|stroke| {
            stroke::PreparedStroke::from_parts_indexed(
                self.arena().stroke_segments.slice(stroke.segments),
                stroke.slab,
                stroke.x_sorted,
            )
        })
    }

    /// One draw's gradient field, reconstituted as the borrowed view.
    pub(crate) fn field_of(&self, rec: &Draw) -> Option<GradientField<'_>> {
        rec.field.map(|field| {
            let points = self.arena().gradient_points.slice(field.points);
            let params = self.arena().gradient_params.slice(field.params);
            if field.next.len == 0 {
                GradientField::from_parts(points, params)
            } else {
                GradientField::from_contours(
                    points,
                    params,
                    self.arena().gradient_next.slice(field.next),
                    self.arena().gradient_edge_params.slice(field.edge_params),
                )
            }
        })
    }

    /// The PG-6 allocation ledger for this frame (fm-e9h): heap allocations
    /// the arena and worker pool have performed since this job's
    /// construction, the arena's reserved buffer bytes, and the worker
    /// pool's slot count. On the [`FrameJob::new_in`] path with a reused
    /// arena, frames after the first report zero allocations for a stable
    /// scene — the steady-state property PG-6 asks for, proven on the
    /// engine's own counters rather than on an allocator shim.
    #[must_use]
    pub fn allocation_stats(&self) -> AllocStats {
        self.arena().stats()
    }

    #[cfg(feature = "metal")]
    pub(crate) fn prepared_draws(&self) -> &[Option<Draw>] {
        self.draws()
    }

    #[cfg(feature = "metal")]
    pub(crate) fn frame_config(&self) -> FrameConfig {
        self.config
    }

    #[cfg(feature = "metal")]
    pub(crate) fn frame_binning(&self) -> &Binning {
        self.binning
    }

    /// Rasterize into a freshly allocated raw frame.
    ///
    /// # Errors
    /// [`FrameError::ZeroDimension`] for an empty viewport.
    pub fn render(&self, threads: usize) -> Result<FrameBuffer, FrameError> {
        let mut buffer = FrameBuffer::new(self.config.layout()?);
        self.render_into(threads, &mut buffer)?;
        Ok(buffer)
    }

    /// Rasterize a raw frame and return deterministic adaptive-AA work counters.
    ///
    /// The image is identical to [`FrameJob::render`] for the same job and
    /// thread count; instrumentation is integer-only, accumulated outside the
    /// pixel arithmetic.
    ///
    /// # Errors
    /// See [`FrameJob::render`].
    pub fn render_with_stats(&self, threads: usize) -> Result<(FrameBuffer, AaStats), FrameError> {
        let mut buffer = FrameBuffer::new(self.config.layout()?);
        let stats = self.render_into_profiled(threads, &mut buffer)?;
        Ok((buffer, stats))
    }

    /// Rasterize into a caller-owned frame — the pooled-buffer path PG-6 measures.
    ///
    /// `threads` is a **scheduling** choice and nothing else: the bytes written
    /// are identical at every value, which is PG-5's per-commit property and is
    /// asserted directly rather than argued.
    ///
    /// # Errors
    /// [`FrameError::FormatMismatch`] unless `dst` is `Rgba16F`;
    /// [`FrameError::DimensionMismatch`] unless it matches the configured
    /// viewport; [`FrameError::TooLarge`] if the covered band-byte product is
    /// not addressable; or [`FrameError::WorkerSpawnFailed`] if the host cannot
    /// start the complete requested CPU render team.
    pub fn render_into(&self, threads: usize, dst: &mut FrameBuffer) -> Result<(), FrameError> {
        self.render_into_profiled(threads, dst).map(|_| ())
    }

    /// Rasterize only dirty fine tiles and restore cache hits byte-for-byte.
    ///
    /// `camera_revision` names camera state independently of the scene's
    /// object revisions. A fixed 2D camera uses one stable value across frames;
    /// changing it invalidates every retained tile. The renderer/configuration
    /// closure is derived internally, so changing background, AA policy,
    /// engine tier, renderer semantics, or tiling also invalidates all pixels.
    ///
    /// # Errors
    /// See [`FrameJob::render_into`], plus [`PixelTileCacheError`] when retained
    /// work or payload geometry cannot be represented.
    pub fn render_into_cached(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        camera_revision: u64,
        cache: &mut PixelTileCache,
    ) -> Result<CachedRenderStats, CachedRenderError> {
        self.render_into_cached_resuming(threads, dst, camera_revision, cache, None)
    }

    /// [`Self::render_into_cached`] into the surface of the cache's previous
    /// completed render.
    ///
    /// `resume` must be the [`PixelTileCache::serial`] read right after that
    /// render, and the surface must not have been written since. Then every
    /// reused tile already holds its cached bytes, so the restore copies are
    /// skipped and the output is unchanged. A stale or absent token restores
    /// as usual.
    ///
    /// # Errors
    /// As [`Self::render_into_cached`].
    pub fn render_into_cached_resuming(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        camera_revision: u64,
        cache: &mut PixelTileCache,
        resume: Option<u64>,
    ) -> Result<CachedRenderStats, CachedRenderError> {
        if dst.layout().format() != PixelFormat::Rgba16F {
            return Err(FrameError::FormatMismatch {
                expected: "Rgba16F raw frame",
                got: dst.layout().format(),
            }
            .into());
        }
        if dst.layout().width() != self.config.viewport.width
            || dst.layout().height() != self.config.viewport.height
        {
            return Err(FrameError::DimensionMismatch.into());
        }
        let resumed = cache.begin_render(resume);
        cache.prepare_frame(
            self.binning,
            self.plan,
            camera_revision,
            self.journal_digest(),
        )?;
        if !resumed {
            cache.restore_reused_tiles(dst, self.binning)?;
        }
        let raster = self.render_into_profiled_cached(threads, dst, cache)?;
        cache.retain_rasterized_tiles(dst, self.binning)?;
        cache.complete_render();
        Ok(CachedRenderStats {
            cache: cache.stats(),
            raster,
        })
    }

    /// Shared render body for ordinary and instrumented entry points.
    fn render_into_profiled(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
    ) -> Result<AaStats, FrameError> {
        match self.identity.engine {
            EngineKind::CertifiedCpu => {
                if self.identity.tier == Tier::Scalar {
                    self.render_into_profiled_with::<CertifiedScalar>(threads, dst)
                } else {
                    self.render_into_profiled_with::<CertifiedBuildTier>(threads, dst)
                }
            }
            EngineKind::FastCpu => {
                if self.identity.tier == Tier::Scalar {
                    self.render_into_profiled_with::<FastScalar>(threads, dst)
                } else {
                    self.render_into_profiled_with::<FastBuildTier>(threads, dst)
                }
            }
            // Annex jobs are executable only through their backend-specific,
            // stateful renderer. Reaching this arm would publish CPU bytes
            // under a false engine identity, so fail closed.
            EngineKind::Metal | EngineKind::Cuda => {
                unreachable!("annex jobs must use their backend-specific renderer")
            }
        }
    }

    fn render_into_profiled_cached(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        cache: &PixelTileCache,
    ) -> Result<AaStats, FrameError> {
        match self.identity.engine {
            EngineKind::CertifiedCpu => {
                if self.identity.tier == Tier::Scalar {
                    self.render_into_profiled_with_cache::<CertifiedScalar>(threads, dst, cache)
                } else {
                    self.render_into_profiled_with_cache::<CertifiedBuildTier>(threads, dst, cache)
                }
            }
            EngineKind::FastCpu => {
                if self.identity.tier == Tier::Scalar {
                    self.render_into_profiled_with_cache::<FastScalar>(threads, dst, cache)
                } else {
                    self.render_into_profiled_with_cache::<FastBuildTier>(threads, dst, cache)
                }
            }
            EngineKind::Metal | EngineKind::Cuda => {
                unreachable!("annex jobs must use their backend-specific renderer")
            }
        }
    }

    /// Monomorphized render body selected once per frame.
    fn render_into_profiled_with<K: PixelKernel>(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
    ) -> Result<AaStats, FrameError> {
        self.render_into_profiled_with_spawner::<K, _>(threads, dst, &NativeScopedSpawner)
    }

    fn render_into_profiled_with_cache<K: PixelKernel>(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        cache: &PixelTileCache,
    ) -> Result<AaStats, FrameError> {
        self.render_into_profiled_with_spawner_and_cache::<K, _, true>(
            threads,
            dst,
            &NativeScopedSpawner,
            Some(cache),
        )
    }

    fn render_into_profiled_with_spawner<K: PixelKernel, S: ScopedSpawner>(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        spawner: &S,
    ) -> Result<AaStats, FrameError> {
        self.render_into_profiled_with_spawner_and_cache::<K, S, false>(threads, dst, spawner, None)
    }

    fn render_into_profiled_with_spawner_and_cache<
        K: PixelKernel,
        S: ScopedSpawner,
        const CACHED: bool,
    >(
        &self,
        threads: usize,
        dst: &mut FrameBuffer,
        spawner: &S,
        cache: Option<&PixelTileCache>,
    ) -> Result<AaStats, FrameError> {
        // W5 wasm tier 1: collapses to 1 on wasm32 (no spawnable threads);
        // the identity on native. See crate::effective_threads.
        let threads = crate::effective_threads(threads);
        if dst.layout().format() != PixelFormat::Rgba16F {
            return Err(FrameError::FormatMismatch {
                expected: "Rgba16F raw frame",
                got: dst.layout().format(),
            });
        }
        // The binning must have been built for this viewport and this tiling.
        // If it was not, `cols` disagrees with the grid the command lists were
        // scattered into and every tile index names the wrong tile — a wrong
        // picture with no error anywhere, which is the failure mode this crate
        // is least able to afford. Checked here rather than in `new` because
        // this is where the grid is walked.
        let rows = self
            .config
            .viewport
            .height
            .div_ceil(self.binning.tiling().fine_tile.max(1));
        if self.binning.tile_count() != self.cols as usize * rows as usize {
            return Err(FrameError::DimensionMismatch);
        }
        if dst.layout().width() != self.config.viewport.width
            || dst.layout().height() != self.config.viewport.height
        {
            return Err(FrameError::DimensionMismatch);
        }

        let scheduling_tile = self.binning.tiling().fine_tile.max(1);
        let scratch_width = usize::try_from(scheduling_tile.min(self.config.viewport.width.max(1)))
            .map_err(|_| FrameError::TooLarge)?;
        let band_rows = usize::try_from(scheduling_tile.min(self.config.viewport.height.max(1)))
            .map_err(|_| FrameError::TooLarge)?;
        // Size the requested team before any worker can begin draining the
        // queue. Lazy creation inside spawned closures made warm-up depend on
        // scheduling: one fast worker could finish and return its slot before
        // a late closure checked out, leaving a later frame to grow the pool.
        // Synchronous preparation makes frame-one sizing deterministic and
        // every subsequent checkout allocation-free for the same geometry.
        self.arena()
            .workers
            .prepare::<K>(scratch_width, self.cols as usize, threads.max(1))?;
        let stride = dst.layout().stride(0);
        let band_bytes = stride.checked_mul(band_rows).ok_or(FrameError::TooLarge)?;
        let plane = dst.plane_mut(0);

        // `chunks_mut` is the whole safety argument: it yields provably disjoint
        // `&mut [u8]`, one per band, so write-disjointness (§10.5b) is a fact the
        // borrow checker enforces rather than a claim a comment makes. The
        // iterator itself is the work queue — pulled one band at a time under
        // the mutex — so no per-frame `Vec` of band slices is ever
        // materialized (PG-6 site 2, fm-e9h).
        if threads <= 1 {
            let mut worker = self
                .arena()
                .workers
                .checkout::<K>(scratch_width, self.cols as usize);
            for (band, bytes) in plane.chunks_mut(band_bytes).enumerate() {
                self.render_band::<K, CACHED>(&mut worker, band, bytes, stride, cache);
            }
            return Ok(worker.stats);
        }

        // Which worker takes which band is deliberately unspecified, because a
        // band's bytes do not depend on who computed them.
        let queue = Mutex::new(plane.chunks_mut(band_bytes).enumerate());
        let stats = Mutex::new(AaStats::default());
        run_scoped_workers(threads, spawner, || {
            let mut worker = self
                .arena()
                .workers
                .checkout::<K>(scratch_width, self.cols as usize);
            loop {
                let next = queue.lock().unwrap_or_else(PoisonError::into_inner).next();
                let Some((band, bytes)) = next else { break };
                self.render_band::<K, CACHED>(&mut worker, band, bytes, stride, cache);
            }
            stats
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .merge(worker.stats);
        })?;
        Ok(stats.into_inner().unwrap_or_else(PoisonError::into_inner))
    }

    /// Rasterize one band: `tile` pixel rows spanning the full frame width.
    fn render_band<K: PixelKernel, const CACHED: bool>(
        &self,
        worker: &mut Worker<K>,
        band: usize,
        bytes: &mut [u8],
        stride: usize,
        cache: Option<&PixelTileCache>,
    ) {
        let tile = self.binning.tiling().fine_tile.max(1);
        let width = self.config.viewport.width;
        let height = self.config.viewport.height;
        let y0 = band as u32 * tile;
        let y1 = (y0 + tile).min(height);
        let aa = self.render_aa();

        for tx in 0..self.cols {
            let t = band * self.cols as usize + tx as usize;
            worker.tile_classes[tx as usize] =
                if CACHED && cache.is_some_and(|cache| cache.work(t) == Some(TileWork::Reuse)) {
                    CoverageClass::Empty
                } else if t < self.binning.tile_count() {
                    self.initial_tile_class(t)
                } else {
                    CoverageClass::Empty
                };
        }

        // The background is written first and unconditionally, so a band with no
        // commands still costs exactly one pass — and so a tile that draws
        // nothing is not distinguishable from one that never ran.
        let bg = K::from_premul(self.config.background.premultiply());
        for py in y0..y1 {
            let row = &mut bytes[(py - y0) as usize * stride..];
            for tx in 0..self.cols {
                let x_lo = tx * tile;
                let x_hi = (x_lo + tile).min(width);
                if x_lo >= x_hi {
                    continue;
                }
                let w = (x_hi - x_lo) as usize;
                let t = band * self.cols as usize + tx as usize;
                if CACHED && cache.is_some_and(|cache| cache.work(t) == Some(TileWork::Reuse)) {
                    // The typed, bounds-checked serial restore ran before
                    // worker fan-out. A hit's only worker action is to skip.
                    continue;
                }
                worker.acc[..w].fill(bg);
                worker.edges[..w].fill(0);
                worker.stats.output_cells = worker.stats.output_cells.saturating_add(w as u64);

                if t < self.binning.tile_count() {
                    match aa {
                        RenderAa::Forced(samples) => {
                            for (i, pixel) in worker.acc[..w].iter_mut().enumerate() {
                                *pixel = self.composite_pixel_supersampled::<K>(
                                    t,
                                    x_lo + i as u32,
                                    py,
                                    samples,
                                );
                            }
                            if samples == 2 {
                                worker.stats.ssaa2x_cells =
                                    worker.stats.ssaa2x_cells.saturating_add(w as u64);
                            } else {
                                worker.stats.ssaa4x_cells =
                                    worker.stats.ssaa4x_cells.saturating_add(w as u64);
                            }
                        }
                        RenderAa::Canonical | RenderAa::Adaptive => {
                            let classify = aa == RenderAa::Adaptive;
                            self.composite_row::<K>(worker, t, py, x_lo, x_hi, classify);
                            worker.stats.native_cells =
                                worker.stats.native_cells.saturating_add(w as u64);
                            if classify {
                                for i in 0..w {
                                    let crossings = worker.edges[i];
                                    let samples = if crossings >= AA_COMPLEX_4X_CROSSINGS {
                                        4
                                    } else if crossings >= AA_COMPLEX_2X_CROSSINGS {
                                        2
                                    } else {
                                        continue;
                                    };
                                    worker.tile_classes[tx as usize] = CoverageClass::ComplexEdge;
                                    worker.acc[i] = self.composite_pixel_supersampled::<K>(
                                        t,
                                        x_lo + i as u32,
                                        py,
                                        samples,
                                    );
                                    if samples == 2 {
                                        worker.stats.ssaa2x_cells =
                                            worker.stats.ssaa2x_cells.saturating_add(1);
                                    } else {
                                        worker.stats.ssaa4x_cells =
                                            worker.stats.ssaa4x_cells.saturating_add(1);
                                    }
                                }
                            }
                        }
                    }
                }

                let base = x_lo as usize * 8;
                K::write_row(&worker.acc[..w], &mut row[base..base + w * 8]);
            }
        }

        for (tx, class) in worker
            .tile_classes
            .iter()
            .take(self.cols as usize)
            .enumerate()
        {
            let tile = band * self.cols as usize + tx;
            if !CACHED || cache.is_none_or(|cache| cache.work(tile) != Some(TileWork::Reuse)) {
                worker.stats.count_tile(*class);
            }
        }
    }

    /// Initial aggregate class from the command flags alone.
    ///
    /// Adaptive's per-cell pass can upgrade `SimpleEdge` to `ComplexEdge`; it
    /// can never downgrade a conservative edge classification to an interior
    /// promise.
    fn initial_tile_class(&self, tile: usize) -> CoverageClass {
        let Some(draws) = self.binning.tile(tile) else {
            return CoverageClass::Empty;
        };
        let Some(flags) = self.binning.tile_flags(tile) else {
            return CoverageClass::Empty;
        };
        let mut visible = false;
        let mut edge = false;
        for (k, &d) in draws.iter().enumerate() {
            let Some(Some(rec)) = self.draws().get(d as usize) else {
                continue;
            };
            visible = true;
            if rec.draws_stroke || flags.get(k).copied() != Some(CLASS_INTERIOR) {
                edge = true;
            }
        }
        if !visible {
            CoverageClass::Empty
        } else if edge {
            CoverageClass::SimpleEdge
        } else {
            CoverageClass::FullyCovered
        }
    }

    /// Composite one tile's command list over one pixel row of the accumulator.
    ///
    /// The draw loop is the **outer** one and painter order is its order: that is
    /// what lets a fill contribute a whole row of coverage from one pass over its
    /// pieces while a stroke still shades per pixel, without either being able to
    /// reorder the composite.
    fn composite_row<K: PixelKernel>(
        &self,
        worker: &mut Worker<K>,
        tile: usize,
        py: u32,
        x_lo: u32,
        x_hi: u32,
        classify: bool,
    ) {
        let Some(draws) = self.binning.tile(tile) else {
            return;
        };
        let Some(flags) = self.binning.tile_flags(tile) else {
            return;
        };
        for (k, &d) in draws.iter().enumerate() {
            // `d` is an instance index, and `draws` is indexed by instance
            // index. A `None` is an instance with no pass; an out-of-range index
            // would be a binning built against a different plan.
            let Some(Some(rec)) = self.draws().get(d as usize) else {
                continue;
            };
            let interior = flags.get(k).copied() == Some(CLASS_INTERIOR);
            // R-5: within one object the fill draws before the stroke, unless
            // `stroke_behind` swaps them (docs/RENDER_ORDER.md).
            if rec.style.stroke_behind {
                self.stroke_pass::<K>(worker, rec, py, x_lo, x_hi, classify);
                self.fill_pass::<K>(worker, rec, interior, py, x_lo..x_hi, classify);
            } else {
                self.fill_pass::<K>(worker, rec, interior, py, x_lo..x_hi, classify);
                self.stroke_pass::<K>(worker, rec, py, x_lo, x_hi, classify);
            }
        }
    }

    /// §10.2's fill over one row of one tile.
    fn fill_pass<K: PixelKernel>(
        &self,
        worker: &mut Worker<K>,
        rec: &Draw,
        interior: bool,
        py: u32,
        x: std::ops::Range<u32>,
        classify: bool,
    ) {
        let (x_lo, x_hi) = (x.start, x.end);
        if !rec.draws_fill || row_misses(rec.fill_slab, py) {
            return;
        }
        let w = (x_hi - x_lo) as usize;

        // Binning already proved exact unit coverage for the whole tile. A
        // flat fill therefore needs neither a coverage buffer nor the pixel
        // classification loop. Preserve painter order and blend every layer.
        if interior && let Some(rgba) = rec.flat_fill {
            K::source_over_full_span(rgba, &mut worker.acc[..w]);
            return;
        }

        // Exactly one of three sources fills the row, and the order of the tests
        // is the order of §10.4's own argument: a classified interior costs
        // nothing, a hinted kernel costs a closed form, and the general machinery
        // is the fallback rather than the toll road.
        let pieces = self.pieces_of(rec);
        let mut general_classified = false;
        if interior {
            worker.cov[..w].copy_from_slice(worker.scratch.interior_row(x_lo, x_hi));
        } else if !rec.kernel.row(py, x_lo, x_hi, &mut worker.cov[..w]) {
            if classify {
                let (coverage, crossings) =
                    worker
                        .scratch
                        .fill_row_classified(pieces, rec.translate, py, x_lo, x_hi);
                worker.cov[..w].copy_from_slice(coverage);
                for (edge, count) in worker.edges[..w].iter_mut().zip(crossings) {
                    *edge = edge.saturating_add(*count);
                }
                general_classified = true;
            } else {
                worker.cov[..w].copy_from_slice(worker.scratch.fill_row(
                    pieces,
                    rec.translate,
                    py,
                    x_lo,
                    x_hi,
                ));
            }
        }

        let segments = self.segments_of(rec);
        for i in 0..w {
            let coverage = worker.cov[i];
            if coverage <= 0.0 {
                continue;
            }
            if classify && !interior && !general_classified {
                let crossings =
                    fill::boundary_crossings_at_cell(pieces, rec.translate, py, x_lo + i as u32);
                let contribution = if coverage < 1.0 && crossings == 0 {
                    // A partial cell whose boundary escaped all six interior
                    // probes contains a sub-probe feature. Conservatively call
                    // it complex rather than silently missing it.
                    AA_COMPLEX_2X_CROSSINGS
                } else {
                    crossings
                };
                worker.edges[i] = worker.edges[i].saturating_add(contribution);
            }
            if rec.flat_fill.is_none() {
                let p = [f64::from(x_lo + i as u32) + 0.5, f64::from(py) + 0.5];
                let field = self.field_of(rec).expect("a non-flat fill carries a field");
                let rgba = fill_rgba_with_border(
                    &rec.style,
                    &field,
                    segments,
                    self.config.map,
                    rec.translate,
                    p,
                );
                worker.acc[i] = K::source_over(rgba, coverage, worker.acc[i]);
            }
        }
        if let Some(rgba) = rec.flat_fill {
            K::source_over_span(rgba, &worker.cov[..w], &mut worker.acc[..w]);
        }
    }

    /// §10.3's stroke over one row of one tile.
    fn stroke_pass<K: PixelKernel>(
        &self,
        worker: &mut Worker<K>,
        rec: &Draw,
        py: u32,
        x_lo: u32,
        x_hi: u32,
        classify: bool,
    ) {
        let Some(stroke) = self.stroke_of(rec) else {
            return;
        };
        let slab = stroke.slab();
        if !rec.draws_stroke || row_misses(slab, py) {
            return;
        }
        let segments = self.segments_of(rec);
        let joins = self.joins_of(rec);
        // A constant-width stroke narrower than the complexity threshold is
        // rejected by `stroke_contributes_complexity` at every station, so
        // neither the centre test nor any subcell probe can count an edge.
        // Skipping the classification then changes no bit and saves the four
        // probe shades per AA-band pixel.
        let classify = classify && stroke_may_contribute_complexity(&rec.style, self.config.map);
        // Only cells meeting the stroke's row span can receive coverage or an
        // edge count; every other pixel of the row is an exact no-op.
        let Some((span_lo, span_hi)) = stroke.row_span(f64::from(py)) else {
            return;
        };
        let w = (x_hi - x_lo) as usize;
        let cell = |x: f64| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let index = (x - f64::from(x_lo)).clamp(0.0, w as f64) as usize;
            index
        };
        let (first, last) = (cell(span_lo.floor()), cell(span_hi.floor() + 1.0));
        for i in first..last {
            let p = [f64::from(x_lo + i as u32) + 0.5, f64::from(py) + 0.5];
            if p[0] < slab[0] || p[0] > slab[2] {
                continue;
            }
            let (coverage, s) = stroke.shade(
                segments,
                joins,
                &rec.style,
                self.config.map,
                rec.translate,
                p,
            );
            if classify {
                let centre_is_eligible_edge = coverage > 0.0
                    && coverage < 1.0
                    && stroke_contributes_complexity(&rec.style, self.config.map, s);
                if centre_is_eligible_edge
                    || self.stroke_has_eligible_subcell_edge(
                        rec,
                        segments,
                        py,
                        x_lo + i as u32,
                        coverage,
                    )
                {
                    worker.edges[i] = worker.edges[i].saturating_add(1);
                }
            }
            if coverage <= 0.0 {
                continue;
            }
            worker.acc[i] = K::source_over(stroke_rgba_at(&rec.style, s), coverage, worker.acc[i]);
        }
    }

    /// Does an eligible off-centre stroke edge reach a 2×2 probe point?
    ///
    /// The canonical stroke pass shades the centre, which can be fully inside or
    /// outside while an edge still crosses a corner of the cell. Adaptive
    /// classification therefore probes the four 2× positions when the centre
    /// did not already establish an eligible edge. Each draw contributes at
    /// most one boundary sheet.
    fn stroke_has_eligible_subcell_edge(
        &self,
        rec: &Draw,
        segments: &[Segment],
        py: u32,
        px: u32,
        centre_coverage: f64,
    ) -> bool {
        // For a constant-width stroke, a saturated smoothstep proves that the
        // silhouette is at least half an AA band from the centre. The furthest
        // 2× probe is only `sqrt(1/8)` pixels away, so at the default 1.5 px
        // band no probe can possibly cross the edge. A width ramp does not have
        // that Lipschitz bound: its silhouette can move between the centre and
        // a probe, so ramped strokes retain the probes.
        let aa_band = effective_aa_band(&rec.style);
        let centre_is_saturated = centre_coverage <= 0.0 || centre_coverage >= 1.0;
        let constant_width = rec.style.has_constant_stroke_width();
        if constant_width && centre_is_saturated && aa_band > std::f64::consts::FRAC_1_SQRT_2 {
            return false;
        }
        let Some(stroke) = self.stroke_of(rec) else {
            return false;
        };
        let joins = self.joins_of(rec);
        for dy in [0.25, 0.75] {
            for dx in [0.25, 0.75] {
                let p = [f64::from(px) + dx, f64::from(py) + dy];
                let (coverage, s) = stroke.shade(
                    segments,
                    joins,
                    &rec.style,
                    self.config.map,
                    rec.translate,
                    p,
                );
                if !stroke_contributes_complexity(&rec.style, self.config.map, s) {
                    continue;
                }
                let sample_is_edge = coverage > 0.0 && coverage < 1.0;
                let crosses_from_centre = centre_coverage <= 0.0 && coverage > 0.0
                    || centre_coverage >= 1.0 && coverage < 1.0;
                if sample_is_edge || crosses_from_centre {
                    return true;
                }
            }
        }
        false
    }

    /// Recompose one native pixel from a fixed subcell grid and resolve it
    /// immediately.
    ///
    /// Painter order is repeated independently at every sample; averaging
    /// already-composited samples is what makes overlapping edges correct.
    /// Only the four `f64` sums below survive the loop, so there is no
    /// supersampled frame or second resolve pass.
    fn composite_pixel_supersampled<K: PixelKernel>(
        &self,
        tile: usize,
        px: u32,
        py: u32,
        samples: u32,
    ) -> K::Pixel {
        let samples = samples.max(1);
        let draws = self.binning.tile(tile).unwrap_or_default();
        let flags = self.binning.tile_flags(tile).unwrap_or_default();
        let mut sum = PremulRgba::TRANSPARENT;

        for sample_y in 0..samples {
            for sample_x in 0..samples {
                let subcell = Subcell {
                    px,
                    py,
                    samples,
                    x: sample_x,
                    y: sample_y,
                };
                let mut acc = K::from_premul(self.config.background.premultiply());
                for (k, &d) in draws.iter().enumerate() {
                    let Some(Some(rec)) = self.draws().get(d as usize) else {
                        continue;
                    };
                    let interior = flags.get(k).copied() == Some(CLASS_INTERIOR);
                    if rec.style.stroke_behind {
                        acc = self.stroke_sample::<K>(rec, subcell, acc);
                        acc = self.fill_sample::<K>(rec, interior, subcell, acc);
                    } else {
                        acc = self.fill_sample::<K>(rec, interior, subcell, acc);
                        acc = self.stroke_sample::<K>(rec, subcell, acc);
                    }
                }
                let acc = K::to_premul(acc);
                sum.r += acc.r;
                sum.g += acc.g;
                sum.b += acc.b;
                sum.a += acc.a;
            }
        }

        let inverse = 1.0 / f64::from(samples * samples);
        K::from_premul(PremulRgba {
            r: sum.r * inverse,
            g: sum.g * inverse,
            b: sum.b * inverse,
            a: sum.a * inverse,
        })
    }

    /// One fill pass at one subcell, preserving the native exact coverage
    /// kernel (primitive hint or general curve-area integral).
    fn fill_sample<K: PixelKernel>(
        &self,
        rec: &Draw,
        interior: bool,
        subcell: Subcell,
        dst: K::Pixel,
    ) -> K::Pixel {
        if !rec.draws_fill {
            return dst;
        }
        let coverage = if interior {
            1.0
        } else {
            rec.kernel
                .coverage_subcell(
                    subcell.px,
                    subcell.py,
                    subcell.samples,
                    subcell.x,
                    subcell.y,
                )
                .unwrap_or_else(|| {
                    fill::coverage_at_subcell(
                        self.pieces_of(rec),
                        rec.translate,
                        subcell.py,
                        subcell.px,
                        subcell.samples,
                        subcell.x,
                        subcell.y,
                    )
                })
        };
        if coverage <= 0.0 {
            return dst;
        }
        let p = subcell.centre();
        let rgba = match rec.flat_fill {
            Some(c) => c,
            None => fill_rgba_with_border(
                &rec.style,
                &self.field_of(rec).expect("a non-flat fill carries a field"),
                self.segments_of(rec),
                self.config.map,
                rec.translate,
                p,
            ),
        };
        K::source_over(rgba, coverage, dst)
    }

    /// One stroke pass at one subcell centre.
    fn stroke_sample<K: PixelKernel>(
        &self,
        rec: &Draw,
        subcell: Subcell,
        dst: K::Pixel,
    ) -> K::Pixel {
        let Some(stroke) = self.stroke_of(rec) else {
            return dst;
        };
        let p = subcell.centre();
        let (coverage, s) = stroke.shade(
            self.segments_of(rec),
            self.joins_of(rec),
            &rec.style,
            self.config.map,
            rec.translate,
            p,
        );
        if coverage <= 0.0 {
            dst
        } else {
            K::source_over(stroke_rgba_at(&rec.style, s), coverage, dst)
        }
    }

    /// This draw's slice of the plan's one flat segment table.
    pub(crate) fn segments_of(&self, rec: &Draw) -> &[Segment] {
        if let Some(range) = rec.transformed_segments {
            return self.arena().segments.slice(range);
        }
        let all = self.plan.segments();
        let lo = (rec.first_segment as usize).min(all.len());
        let hi = (rec.first_segment as usize + rec.segment_count as usize).min(all.len());
        &all[lo..hi]
    }

    /// This draw's fill pieces, transformed once per frame when its placement
    /// has a non-identity linear part.
    pub(crate) fn pieces_of(&self, rec: &Draw) -> &[fill::MonoPiece] {
        match rec.transformed_pieces {
            Some(range) => self.arena().pieces.slice(range),
            None => self.mono.pieces_of(rec.shape),
        }
    }
}

// ------------------------------------------------------------------ the worker

/// Does this stroke carry silhouette detail outside the analytic AA band?
fn stroke_contributes_complexity(style: &Style, map: ScreenMap, s: f64) -> bool {
    let aa_band = effective_aa_band(style);
    let full_width = 2.0 * half_width_px(style, map, s);
    full_width >= AA_STROKE_COMPLEX_MIN_WIDTH_BANDS * aa_band
}

/// Whether [`stroke_contributes_complexity`] can hold at *some* station.
///
/// False only for a constant-width stroke whose one (finite) width fails the
/// threshold: [`Style::has_constant_stroke_width`] means every station's width
/// is numerically the same as station 0's, so the predicate's verdict at
/// station 0 is its verdict everywhere. Any other stroke answers true and keeps
/// the per-station test.
fn stroke_may_contribute_complexity(style: &Style, map: ScreenMap) -> bool {
    !style.has_constant_stroke_width() || stroke_contributes_complexity(style, map, 0.0)
}

/// The same nonzero AA width [`crate::stroke::aa_coverage`] evaluates.
fn effective_aa_band(style: &Style) -> f64 {
    if style.anti_alias_width > 0.0 {
        f64::from(style.anti_alias_width)
    } else {
        1e-8
    }
}

/// One thread's scratch, allocated once per pool slot rather than per row,
/// per band, or — critically for PG-6 — per `render_into` call.
///
/// PG-6 forbids steady-state per-frame heap allocation. A worker's buffers are
/// sized for the widest covered tile row the frame can present and then reused
/// for every band it takes; the [`WorkerPool`] keeps the whole worker alive across
/// frames, so the allocation happens once at pool warm-up and never again.
struct Worker<K: PixelKernel> {
    /// The maximum covered row width the buffers are sized for — the pool's key.
    tile: usize,
    /// The row accumulator, premultiplied linear light.
    acc: Vec<K::Pixel>,
    /// One draw's coverage over the row.
    cov: Vec<f64>,
    /// Saturating independent-boundary count for each cell in the row.
    edges: Vec<u8>,
    /// The fill's own scratch.
    scratch: RowScratch,
    /// Aggregate class for each fine tile in the current band.
    tile_classes: Vec<CoverageClass>,
    /// This worker's integer-only instrumentation.
    stats: AaStats,
    marker: std::marker::PhantomData<K>,
}

impl<K: PixelKernel> Worker<K> {
    /// Heap allocations [`Worker::new`] performs: `acc`, `cov`, `edges`, the
    /// three `Vec`s inside [`RowScratch`], and `tile_classes`. Counted into
    /// the arena's ledger when the pool creates a slot, so the PG-6 report
    /// sees worker warm-up exactly once.
    const SCRATCH_ALLOCATIONS: u64 = 7;

    fn preflight(tile: usize, cols: usize) -> Result<(), FrameError> {
        fn check_len<T>(len: usize) -> Result<(), FrameError> {
            let element = std::mem::size_of::<T>();
            if element != 0 && len > isize::MAX as usize / element {
                return Err(FrameError::TooLarge);
            }
            Ok(())
        }

        let cells = tile.checked_add(1).ok_or(FrameError::TooLarge)?;
        check_len::<K::Pixel>(tile)?;
        check_len::<f64>(tile)?;
        check_len::<u8>(tile)?;
        check_len::<f64>(cells)?;
        check_len::<f64>(tile)?;
        check_len::<u8>(tile)?;
        check_len::<CoverageClass>(cols)
    }

    fn try_new(tile: usize, cols: usize) -> Result<Worker<K>, FrameError> {
        fn zeroed<T: Clone>(len: usize, value: T) -> Result<Vec<T>, FrameError> {
            let mut values = Vec::new();
            values
                .try_reserve_exact(len)
                .map_err(|_| FrameError::TooLarge)?;
            values.resize(len, value);
            Ok(values)
        }

        Self::preflight(tile, cols)?;
        Ok(Worker {
            tile,
            acc: zeroed(tile, K::from_premul(PremulRgba::TRANSPARENT))?,
            cov: zeroed(tile, 0.0)?,
            edges: zeroed(tile, 0)?,
            scratch: RowScratch::try_for_width(tile)?,
            tile_classes: zeroed(cols, CoverageClass::Empty)?,
            stats: AaStats::default(),
            marker: std::marker::PhantomData,
        })
    }

    fn resize_tile_classes(&mut self, cols: usize) -> Result<bool, FrameError> {
        if self.tile_classes.len() == cols {
            return Ok(false);
        }
        Self::preflight(self.tile, cols)?;
        let capacity = self.tile_classes.capacity();
        if self.tile_classes.len() < cols {
            self.tile_classes
                .try_reserve_exact(cols - self.tile_classes.len())
                .map_err(|_| FrameError::TooLarge)?;
        }
        self.tile_classes.resize(cols, CoverageClass::Empty);
        Ok(self.tile_classes.capacity() != capacity)
    }
}

/// The worker-scratch pool: one slot per (kernel, covered row width), reused across
/// [`FrameJob::render_into`] calls (PG-6 site 3, fm-e9h).
///
/// Owned by the [`FrameArena`], so on the [`FrameJob::new_in`] path a slot's
/// buffers are allocated once — at the warm-up frame — and checked out by
/// every later render. There is one sub-pool per monomorphized kernel rather
/// than any type erasure: the pool serves exactly the four kernels this
/// module compiles, and the typed slot keeps `#![forbid(unsafe_code)]`
/// absolute. The pool is keyed on maximum covered row width because the buffers are;
/// `tile_classes` is resized on checkout when the column count differs.
///
/// Sizing is by use, which is sizing by the execution plan one level down:
/// a render team of `t` threads synchronously installs exactly `t` slots on
/// the first frame, and the pool never grows past the largest team it has
/// served for that kernel and covered row width.
pub(crate) struct WorkerPool {
    certified_scalar: KernelSlots<CertifiedScalar>,
    certified_tier: KernelSlots<CertifiedBuildTier>,
    fast_scalar: KernelSlots<FastScalar>,
    fast_tier: KernelSlots<FastBuildTier>,
    /// Heap allocations slot creation and growth have performed since the
    /// arena's last `begin_frame`.
    allocs: std::sync::atomic::AtomicU64,
}

impl std::fmt::Debug for WorkerPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerPool")
            .field("slots", &self.slots())
            .finish()
    }
}

/// One kernel's idle workers.
pub(crate) struct KernelSlots<K: PixelKernel> {
    slots: Mutex<Vec<Worker<K>>>,
    available: Condvar,
}

impl<K: PixelKernel> Default for KernelSlots<K> {
    fn default() -> Self {
        KernelSlots {
            slots: Mutex::new(Vec::new()),
            available: Condvar::new(),
        }
    }
}

impl WorkerPool {
    pub(crate) fn new() -> WorkerPool {
        WorkerPool {
            certified_scalar: KernelSlots::default(),
            certified_tier: KernelSlots::default(),
            fast_scalar: KernelSlots::default(),
            fast_tier: KernelSlots::default(),
            allocs: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Zero the per-frame allocation counter. The slots persist — they *are*
    /// the steady state.
    pub(crate) fn begin_frame(&self) {
        self.allocs.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Heap allocations since [`WorkerPool::begin_frame`].
    pub(crate) fn allocs(&self) -> u64 {
        self.allocs.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Idle slots across all four kernel sub-pools.
    pub(crate) fn slots(&self) -> usize {
        fn idle<K: PixelKernel>(sub: &KernelSlots<K>) -> usize {
            sub.slots
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .len()
        }
        idle(&self.certified_scalar)
            + idle(&self.certified_tier)
            + idle(&self.fast_scalar)
            + idle(&self.fast_tier)
    }

    /// Synchronously size one kernel/tile sub-pool for the requested render
    /// team before fan-out.
    ///
    /// Doing this outside the spawned closures is what makes the warm-up
    /// independent of which worker the scheduler starts first. Existing slots
    /// are also brought to the current column count here, so checkout cannot
    /// discover a late `tile_classes` growth after the warm frame.
    fn prepare<K: PixelKernel>(
        &self,
        tile: usize,
        cols: usize,
        workers: usize,
    ) -> Result<(), FrameError> {
        Worker::<K>::preflight(tile, cols)?;
        let sub = K::sub_pool(self);
        let mut slots = sub.slots.lock().unwrap_or_else(PoisonError::into_inner);
        let mut matching = 0usize;
        for worker in slots.iter_mut().filter(|worker| worker.tile == tile) {
            matching += 1;
            if worker.resize_tile_classes(cols)? {
                self.allocs
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let additional = workers.saturating_sub(matching);
        if additional == 0 {
            return Ok(());
        }
        // Preserve capacity for every checked-out slot as well as the workers
        // added here. Returning a worker in `Drop` must never discover a need
        // to grow the idle vector after destination rendering has begun.
        let checked_out_capacity = slots.capacity().saturating_sub(slots.len());
        let reserve = checked_out_capacity
            .checked_add(additional)
            .ok_or(FrameError::TooLarge)?;
        let capacity = slots.capacity();
        slots
            .try_reserve_exact(reserve)
            .map_err(|_| FrameError::TooLarge)?;
        if slots.capacity() != capacity {
            self.allocs
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        for _ in 0..additional {
            let worker = Worker::<K>::try_new(tile, cols)?;
            slots.push(worker);
            self.allocs.fetch_add(
                Worker::<K>::SCRATCH_ALLOCATIONS,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        Ok(())
    }

    /// Take a prepared worker for `tile`, waiting when another render call has
    /// borrowed the whole matching team.
    fn checkout<K: PixelKernel>(&self, tile: usize, cols: usize) -> PooledWorker<'_, K> {
        let sub = K::sub_pool(self);
        let mut slots = sub.slots.lock().unwrap_or_else(PoisonError::into_inner);
        let mut worker = loop {
            if let Some(index) = slots.iter().position(|idle| idle.tile == tile) {
                break slots.swap_remove(index);
            }
            slots = sub
                .available
                .wait(slots)
                .unwrap_or_else(PoisonError::into_inner);
        };
        worker.stats = AaStats::default();
        debug_assert_eq!(worker.tile_classes.len(), cols);
        PooledWorker {
            pool: sub,
            worker: Some(worker),
        }
    }
}

/// A checked-out worker; returns to its sub-pool on drop.
struct PooledWorker<'a, K: PixelKernel> {
    pool: &'a KernelSlots<K>,
    worker: Option<Worker<K>>,
}

impl<K: PixelKernel> std::ops::Deref for PooledWorker<'_, K> {
    type Target = Worker<K>;

    fn deref(&self) -> &Worker<K> {
        // `Some` from checkout to drop.
        self.worker.as_ref().expect("pooled worker is populated")
    }
}

impl<K: PixelKernel> std::ops::DerefMut for PooledWorker<'_, K> {
    fn deref_mut(&mut self) -> &mut Worker<K> {
        // `Some` from checkout to drop.
        self.worker.as_mut().expect("pooled worker is populated")
    }
}

impl<K: PixelKernel> Drop for PooledWorker<'_, K> {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            let mut slots = self
                .pool
                .slots
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            debug_assert!(slots.len() < slots.capacity());
            slots.push(worker);
            drop(slots);
            self.pool.available.notify_one();
        }
    }
}

// ------------------------------------------------------------------ the pieces

/// One compositing arithmetic policy.
///
/// The type is selected once in [`FrameJob::render_into_profiled`], so the hot
/// pixel loop contains neither runtime feature detection nor a function-pointer
/// dispatch. Every implementation is elementwise across RGBA: there is no
/// horizontal reduction whose association could depend on lane width.
pub(crate) trait PixelKernel {
    type Pixel: Copy + Send;

    fn from_premul(pixel: PremulRgba) -> Self::Pixel;
    fn to_premul(pixel: Self::Pixel) -> PremulRgba;
    fn source_over(rgba: [f32; 4], coverage: f64, dst: Self::Pixel) -> Self::Pixel;
    fn write_row(acc: &[Self::Pixel], out: &mut [u8]);

    /// This kernel's sub-pool — how one [`WorkerPool`] serves all four
    /// monomorphized kernels with no type erasure.
    fn sub_pool(pool: &WorkerPool) -> &KernelSlots<Self>
    where
        Self: Sized;

    #[inline]
    fn source_over_span(rgba: [f32; 4], coverage: &[f64], dst: &mut [Self::Pixel]) {
        for (&coverage, dst) in coverage.iter().zip(dst) {
            if coverage <= 0.0 {
                continue;
            }
            *dst = Self::source_over(rgba, coverage, *dst);
        }
    }

    /// Binning-proven unit coverage; the scalar definition remains ordinary
    /// source-over with coverage one, including its alpha-only coverage rule.
    #[inline]
    fn source_over_full_span(rgba: [f32; 4], dst: &mut [Self::Pixel]) {
        for dst in dst {
            *dst = Self::source_over(rgba, 1.0, *dst);
        }
    }
}

/// The certified scalar definition.
struct CertifiedScalar;

impl PixelKernel for CertifiedScalar {
    type Pixel = PremulRgba;

    #[inline]
    fn from_premul(pixel: PremulRgba) -> Self::Pixel {
        pixel
    }

    #[inline]
    fn to_premul(pixel: Self::Pixel) -> PremulRgba {
        pixel
    }

    #[inline]
    fn source_over(rgba: [f32; 4], coverage: f64, dst: Self::Pixel) -> Self::Pixel {
        source_over(rgba, coverage, dst)
    }

    #[inline]
    fn write_row(acc: &[Self::Pixel], out: &mut [u8]) {
        write_row(acc, out);
    }

    #[inline]
    fn sub_pool(pool: &WorkerPool) -> &KernelSlots<Self> {
        &pool.certified_scalar
    }
}

/// Certified build-tier route.
///
/// Each accumulator entry holds one pixel's RGBA lanes. Keeping that layout
/// through the worker pool avoids gathering channels from different pixels on
/// every layer. All arithmetic is elementwise, in the scalar definition's
/// multiply-then-add order; alpha is never a horizontal reduction.
struct CertifiedBuildTier;

impl PixelKernel for CertifiedBuildTier {
    type Pixel = Simd<f64, 4>;

    #[inline]
    fn from_premul(pixel: PremulRgba) -> Self::Pixel {
        Simd::from_array([pixel.r, pixel.g, pixel.b, pixel.a])
    }

    #[inline]
    fn to_premul(pixel: Self::Pixel) -> PremulRgba {
        let [r, g, b, a] = pixel.to_array();
        PremulRgba { r, g, b, a }
    }

    #[inline]
    fn source_over(rgba: [f32; 4], coverage: f64, dst: Self::Pixel) -> Self::Pixel {
        let alpha = f64::from(rgba[3]) * coverage;
        let mut source = Simd::from_array(rgba.map(f64::from)) * Simd::splat(alpha);
        source[3] = alpha;
        source + Simd::splat(1.0 - alpha) * dst
    }

    #[inline]
    fn write_row(acc: &[Self::Pixel], out: &mut [u8]) {
        for (&pixel, out) in acc.iter().zip(out.as_chunks_mut::<8>().0) {
            write_row(&[Self::to_premul(pixel)], out);
        }
    }

    #[inline]
    fn sub_pool(pool: &WorkerPool) -> &KernelSlots<Self> {
        &pool.certified_tier
    }

    #[inline]
    fn source_over_full_span(rgba: [f32; 4], dst: &mut [Self::Pixel]) {
        let alpha = f64::from(rgba[3]);
        let mut source = Simd::from_array(rgba.map(f64::from)) * Simd::splat(alpha);
        source[3] = alpha;
        let inverse = Simd::splat(1.0 - alpha);
        for dst in dst {
            *dst = source + inverse * *dst;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PremulRgba32 {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

/// Standard-mode scalar mixed precision: f32 colour and blend arithmetic.
struct FastScalar;

impl PixelKernel for FastScalar {
    type Pixel = PremulRgba32;

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn from_premul(pixel: PremulRgba) -> Self::Pixel {
        PremulRgba32 {
            r: pixel.r as f32,
            g: pixel.g as f32,
            b: pixel.b as f32,
            a: pixel.a as f32,
        }
    }

    #[inline]
    fn to_premul(pixel: Self::Pixel) -> PremulRgba {
        PremulRgba {
            r: f64::from(pixel.r),
            g: f64::from(pixel.g),
            b: f64::from(pixel.b),
            a: f64::from(pixel.a),
        }
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn source_over(rgba: [f32; 4], coverage: f64, dst: Self::Pixel) -> Self::Pixel {
        let alpha = rgba[3] * coverage as f32;
        let inverse = 1.0 - alpha;
        PremulRgba32 {
            r: rgba[0] * alpha + inverse * dst.r,
            g: rgba[1] * alpha + inverse * dst.g,
            b: rgba[2] * alpha + inverse * dst.b,
            a: alpha + inverse * dst.a,
        }
    }

    #[inline]
    fn write_row(acc: &[Self::Pixel], out: &mut [u8]) {
        write_row_f32(acc, out);
    }

    #[inline]
    fn sub_pool(pool: &WorkerPool) -> &KernelSlots<Self> {
        &pool.fast_scalar
    }
}

/// Standard-mode build-tier route.
struct FastBuildTier;

impl PixelKernel for FastBuildTier {
    type Pixel = Simd<f32, 4>;

    #[inline]
    fn from_premul(pixel: PremulRgba) -> Self::Pixel {
        let pixel = FastScalar::from_premul(pixel);
        Simd::from_array([pixel.r, pixel.g, pixel.b, pixel.a])
    }

    #[inline]
    fn to_premul(pixel: Self::Pixel) -> PremulRgba {
        let [r, g, b, a] = pixel.to_array();
        FastScalar::to_premul(PremulRgba32 { r, g, b, a })
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn source_over(rgba: [f32; 4], coverage: f64, dst: Self::Pixel) -> Self::Pixel {
        let alpha = rgba[3] * coverage as f32;
        let mut source = Simd::from_array(rgba) * Simd::splat(alpha);
        source[3] = alpha;
        source + Simd::splat(1.0 - alpha) * dst
    }

    #[inline]
    fn write_row(acc: &[Self::Pixel], out: &mut [u8]) {
        write_premul_f32_pixels(acc.iter().map(|pixel| pixel.to_array()), out);
    }

    #[inline]
    fn sub_pool(pool: &WorkerPool) -> &KernelSlots<Self> {
        &pool.fast_tier
    }

    #[inline]
    fn source_over_full_span(rgba: [f32; 4], dst: &mut [Self::Pixel]) {
        let alpha = rgba[3];
        let mut source = Simd::from_array(rgba) * Simd::splat(alpha);
        source[3] = alpha;
        let inverse = Simd::splat(1.0 - alpha);
        for dst in dst {
            *dst = source + inverse * *dst;
        }
    }
}

/// Porter–Duff source-over of a straight-alpha linear colour at a coverage.
///
/// Coverage multiplies **alpha only**: a partly-covered pixel is the same colour
/// over a smaller area, not a paler colour over the whole one. Doing it the other
/// way is the classic antialiasing bug that turns a red line on white into a pink
/// line rather than a red line with soft edges.
fn source_over(rgba: [f32; 4], coverage: f64, dst: PremulRgba) -> PremulRgba {
    LinearRgba {
        r: f64::from(rgba[0]),
        g: f64::from(rgba[1]),
        b: f64::from(rgba[2]),
        a: f64::from(rgba[3]) * coverage,
    }
    .premultiply()
    .over(dst)
}

/// Write one row of the accumulator as linear-light straight-alpha `Rgba16F`.
///
/// Like [`write_premul_f32_pixels`], a pixel bit-identical to its predecessor
/// reuses the predecessor's encoded bytes.
fn write_row(acc: &[PremulRgba], out: &mut [u8]) {
    let mut memo: Option<([u64; 4], [u8; 8])> = None;
    for (px, dst) in acc.iter().zip(out.as_chunks_mut::<8>().0) {
        let key = [px.r, px.g, px.b, px.a].map(f64::to_bits);
        if let Some((previous, bytes)) = memo
            && previous == key
        {
            *dst = bytes;
            continue;
        }
        let lin = px.unpremultiply();
        let mut bytes = [0u8; 8];
        for (k, v) in [lin.r, lin.g, lin.b, lin.a].into_iter().enumerate() {
            let bits = fmn_frame::half::f16_from_f32(v as f32);
            bytes[k * 2..k * 2 + 2].copy_from_slice(&bits.to_le_bytes());
        }
        *dst = bytes;
        memo = Some((key, bytes));
    }
}

/// Standard-mode writeback from the f32 premultiplied accumulator.
fn write_row_f32(acc: &[PremulRgba32], out: &mut [u8]) {
    write_premul_f32_pixels(acc.iter().map(|px| [px.r, px.g, px.b, px.a]), out);
}

/// Unpremultiply and encode premultiplied `f32` pixels as `Rgba16F`.
///
/// The encoding is a pure function of the four input bit patterns, so a run of
/// bit-identical pixels — background, flat fills, saturated stroke cores —
/// reuses the previous pixel's eight bytes instead of repeating four divisions
/// and four half-float roundings. Same bytes, less work.
fn write_premul_f32_pixels(pixels: impl Iterator<Item = [f32; 4]>, out: &mut [u8]) {
    let mut memo: Option<([u32; 4], [u8; 8])> = None;
    for (px, dst) in pixels.zip(out.as_chunks_mut::<8>().0) {
        let key = px.map(f32::to_bits);
        if let Some((previous, bytes)) = memo
            && previous == key
        {
            *dst = bytes;
            continue;
        }
        let [r, g, b, a] = px;
        let rgba = if a == 0.0 {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [r / a, g / a, b / a, a]
        };
        let mut bytes = [0u8; 8];
        for (k, value) in rgba.into_iter().enumerate() {
            let bits = fmn_frame::half::f16_from_f32(value);
            bytes[k * 2..k * 2 + 2].copy_from_slice(&bits.to_le_bytes());
        }
        *dst = bytes;
        memo = Some((key, bytes));
    }
}

/// Does the pixel row `[py, py+1]` lie entirely outside a slab?
///
/// Rejection is exact, not conservative-in-the-useful-direction: outside the
/// hull a closed path's winding is zero and a stroke's distance exceeds its
/// padded slab, so both passes would compute a coverage of zero and composite
/// the identity. `the_engine_matches_a_brute_force_reference` is what holds that
/// claim to bytes.
fn row_misses(slab: [f64; 4], py: u32) -> bool {
    let lo = f64::from(py);
    slab[3] <= lo || slab[1] >= lo + 1.0
}

/// A prepared segment run's screen-space control-hull AABB.
fn hull_slab(segments: &[Segment], map: ScreenMap, translate: [f64; 2]) -> [f64; 4] {
    if segments.is_empty() {
        return [0.0; 4];
    }
    let to_px = |p: fmn_core::types::Vec3| {
        let q = map.to_pixel(p[0], p[1]);
        [q[0] + translate[0], q[1] + translate[1]]
    };
    let mut slab = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for point in segments
        .iter()
        .flat_map(|segment| [segment.p0, segment.p1, segment.p2])
    {
        let pixel = to_px(point);
        slab[0] = slab[0].min(pixel[0]);
        slab[1] = slab[1].min(pixel[1]);
        slab[2] = slab[2].max(pixel[0]);
        slab[3] = slab[3].max(pixel[1]);
    }
    slab
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bin::covers_tile;
    use crate::fill::MonoTable;
    use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, ShapeTag, Stage};

    /// A vmobject with a filled and/or stroked style written across its records.
    ///
    /// Colours are written as the record buffer holds them — **sRGB-encoded**,
    /// the way `mobject.data` presents them — so these fixtures exercise
    /// `read_style`'s decode rather than sneaking past it.
    fn vmob(points: &[[f64; 3]], fill: [f32; 4], stroke: [f32; 4], width: f32) -> Mobject {
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
        for (i, p) in points.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "fill_rgba", &fill);
            buffer.write(i, "stroke_rgba", &stroke);
            buffer.write(i, "stroke_width", &[width]);
        }
        Mobject::from_buffer(buffer)
    }

    /// A closed quadratic ring approximating a circle, as a shared-anchor point run.
    fn ring(cx: f64, cy: f64, r: f64, n: usize, ccw: bool) -> Vec<[f64; 3]> {
        let mut out = Vec::with_capacity(2 * n + 1);
        let step = std::f64::consts::TAU / n as f64;
        let sgn = if ccw { 1.0 } else { -1.0 };
        // The handle sits at the tangent intersection, which is the radius
        // scaled by sec(step/2) — the same construction QuadPath::try_arc uses.
        let sec = 1.0 / fmn_dmath::cos(step / 2.0);
        for k in 0..n {
            let a0 = sgn * step * k as f64;
            let a1 = sgn * step * (k as f64 + 0.5);
            let a2 = sgn * step * (k as f64 + 1.0);
            let p = |a: f64, rad: f64| {
                [
                    cx + rad * fmn_dmath::cos(a),
                    cy + rad * fmn_dmath::sin(a),
                    0.0,
                ]
            };
            if k == 0 {
                out.push(p(a0, r));
            }
            out.push(p(a1, r * sec));
            out.push(p(a2, r));
        }
        out
    }

    /// A closed axis-aligned rectangle as a shared-anchor point run: four
    /// straight quadratics, handles at the edge midpoints, `2n + 1` points.
    fn rect_points(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<[f64; 3]> {
        let corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]];
        let mut out = vec![[corners[0][0], corners[0][1], 0.0]];
        for pair in corners.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            out.push([0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1]), 0.0]);
            out.push([b[0], b[1], 0.0]);
        }
        out
    }

    /// The engine's corpus scene: overlapping translucent fills, a winding hole,
    /// an open stroke with round caps, a hairline finer than the AA band, and a
    /// four-boundary dense-stem cell.
    fn corpus() -> (Stage, Vec<Mob>) {
        let mut stage = Stage::new();
        let mut mobs = Vec::new();
        let mut add = |stage: &mut Stage, m: Mobject| {
            let h = stage.add(m);
            stage.add_to_scene(h).expect("live");
            mobs.push(h);
        };

        // A translucent filled disc.
        add(
            &mut stage,
            vmob(
                &ring(26.0, 30.0, 16.0, 8, true),
                [1.0, 0.0, 0.0, 0.7],
                [0.0; 4],
                0.0,
            ),
        );
        // A second one overlapping it, so the composite order is observable.
        add(
            &mut stage,
            vmob(
                &ring(38.0, 34.0, 14.0, 8, true),
                [0.0, 1.0, 0.0, 0.5],
                [0.0; 4],
                0.0,
            ),
        );
        // An opaque disc with a clockwise hole: the nonzero rule must leave it.
        let outer = ring(72.0, 32.0, 20.0, 8, true);
        let inner = ring(72.0, 32.0, 9.0, 8, false);
        let mut annulus_path = fmn_geom::quadpath::QuadPath::new();
        annulus_path.add_subpath(&outer).expect("outer ring");
        annulus_path.add_subpath(&inner).expect("inner ring");
        add(
            &mut stage,
            vmob(annulus_path.points(), [1.0, 1.0, 0.0, 1.0], [0.0; 4], 0.0),
        );
        // An open stroked polyline with a sharp corner.
        add(
            &mut stage,
            vmob(
                &[
                    [8.0, 60.0, 0.0],
                    [22.0, 60.0, 0.0],
                    [36.0, 60.0, 0.0],
                    [36.0, 74.0, 0.0],
                    [36.0, 88.0, 0.0],
                ],
                [0.0; 4],
                [1.0, 1.0, 1.0, 1.0],
                600.0,
            ),
        );
        // A hairline finer than the AA band.
        add(
            &mut stage,
            vmob(
                &[[6.0, 96.0, 0.0], [48.0, 100.0, 0.0], [90.0, 96.0, 0.0]],
                [0.0; 4],
                [0.0, 0.5, 1.0, 1.0],
                40.0,
            ),
        );
        // Two subpixel stems in one shape: four independent boundary sheets in
        // each occupied cell, so the 4× threshold is exercised rather than
        // merely declared.
        let left_stem = rect_points(104.1, 52.0, 104.2, 86.0);
        let right_stem = rect_points(104.5, 52.0, 104.6, 86.0);
        let mut dense_stems = fmn_geom::quadpath::QuadPath::new();
        dense_stems
            .add_subpath(&left_stem)
            .expect("left dense stem");
        dense_stems
            .add_subpath(&right_stem)
            .expect("right dense stem");
        add(
            &mut stage,
            vmob(dense_stems.points(), [1.0, 0.2, 0.8, 1.0], [0.0; 4], 0.0),
        );
        (stage, mobs)
    }

    fn config() -> FrameConfig {
        FrameConfig::new(
            Viewport {
                width: 112,
                height: 112,
            },
            ScreenMap {
                scale: 1.0,
                origin: [0.0, 0.0],
                y_up: false,
            },
            LinearRgba {
                r: 0.04,
                g: 0.04,
                b: 0.06,
                a: 1.0,
            },
        )
    }

    /// Compile the three derived structures a frame needs.
    fn derive(stage: &Stage, cfg: FrameConfig, tiling: Tiling) -> (RenderPlan, MonoTable, Binning) {
        let mut plan = RenderPlan::new();
        plan.sync(stage, 0).expect("valid engine fixture");
        let mono = MonoTable::build(&plan, cfg.map).expect("bounded test monotone table");
        let binning =
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("bounded test binning");
        (plan, mono, binning)
    }

    fn default_tiling() -> Tiling {
        Tiling {
            macro_tile: 64,
            fine_tile: 16,
        }
    }

    /// The PG-6 corpus (fm-e9h): a glyph field. Dozens of small two-subpath
    /// outlines — an angular bowl with a counter, the way a majuscule is —
    /// miter-stroked and gradient-filled, with affine placements on some, so
    /// every per-draw allocation the bead named is exercised: the draw list,
    /// join wedges, prepared stroke segments, gradient stations, and
    /// transformed segments and pieces.
    fn glyph_field() -> Stage {
        let mut stage = Stage::new();
        for row in 0..6u32 {
            for col in 0..8u32 {
                let x = 6.0 + f64::from(col) * 13.0;
                let y = 8.0 + f64::from(row) * 17.0;
                let mut path = fmn_geom::quadpath::QuadPath::new();
                path.add_subpath(&rect_points(x, y, x + 9.0, y + 12.0))
                    .expect("bowl");
                path.add_subpath(&rect_points(x + 3.0, y + 4.0, x + 6.0, y + 8.0))
                    .expect("counter");
                let points = path.points().to_vec();
                let last = points.len() - 1;
                let gradient = (row + col) % 3 == 0;
                let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
                for (i, p) in points.iter().enumerate() {
                    buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
                    // A different colour on the last record is how the IR
                    // expresses a fill ramp (see vmob's decode note above).
                    let fill = if gradient && i == last {
                        [1.0, 0.4, 0.1, 1.0]
                    } else {
                        [0.1, 0.4, 1.0, 1.0]
                    };
                    buffer.write(i, "fill_rgba", &fill);
                    buffer.write(i, "stroke_rgba", &[0.9, 0.9, 0.95, 1.0]);
                    buffer.write(i, "stroke_width", &[120.0]);
                }
                let mob = stage.add(Mobject::from_buffer(buffer));
                stage.uniforms_mut(mob).expect("live").joint_type = fmn_mobject::JointType::Miter;
                stage.add_to_scene(mob).expect("live");
                if (row + col) % 4 == 1 {
                    // A general rotation: off the signed-axis similarity
                    // route, so the arc-length oracle and the transformed
                    // segment and piece pools all run.
                    stage.apply_affine(
                        mob,
                        Placement::new(
                            [[0.875, -0.484, 0.0], [0.484, 0.875, 0.0], [0.0, 0.0, 1.0]],
                            [1.5, -1.0, 0.0],
                        ),
                    );
                }
            }
        }
        stage
    }

    /// PG-6 (fm-e9h): the certified engine allocates nothing per frame in the
    /// steady state.
    ///
    /// The same glyph-heavy scene rendered `N + 1` times through one
    /// caller-owned arena: frame 1 is the documented warm-up that sizes every
    /// bump pool and every worker slot; frames 2..=N+1 must report **zero**
    /// heap allocations on the engine's own counters, identical arena buffer
    /// bytes (the buffer was allocated exactly once), an identical worker
    /// pool, and — the bit-lock — identical frame digests.
    #[test]
    fn steady_state_frames_allocate_nothing() {
        let stage = glyph_field();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let mut arena = FrameArena::new();
        let mut warm: Option<AllocStats> = None;
        let mut first_digest = None;
        for frame in 0..4u32 {
            let job = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
                .expect("matching frame artifacts");
            // A caller-owned pooled buffer, exactly PG-6's steady-state shape.
            let mut buffer = FrameBuffer::new(cfg.layout().expect("layout"));
            job.render_into(4, &mut buffer).expect("the engine renders");
            let stats = job.allocation_stats();
            let digest = frame_digest(&buffer).expect("frame digest");
            drop(job);
            match warm {
                None => {
                    assert!(
                        stats.heap_allocs_this_frame > 0,
                        "frame 1 is the warm-up: it must size the arena"
                    );
                    assert_eq!(
                        stats.pool_slots, 4,
                        "warm-up must size the requested worker team before fan-out"
                    );
                    warm = Some(stats);
                    first_digest = Some(digest);
                }
                Some(warm_stats) => {
                    assert_eq!(
                        stats.heap_allocs_this_frame,
                        0,
                        "frame {} allocated in the steady state",
                        frame + 1
                    );
                    assert_eq!(
                        stats.arena_buffer_bytes, warm_stats.arena_buffer_bytes,
                        "the arena buffer must be allocated exactly once"
                    );
                    assert_eq!(
                        stats.pool_slots, warm_stats.pool_slots,
                        "the worker pool must not grow in the steady state"
                    );
                    assert_eq!(
                        digest,
                        first_digest.expect("first frame digest"),
                        "storage-only means the bits cannot move"
                    );
                }
            }
        }
    }

    #[test]
    fn retained_pixels_bypass_unchanged_raster_work_and_match_cold_rendering() {
        let (mut stage, mobs) = corpus();
        let cfg = FrameConfig {
            viewport: Viewport {
                width: 109,
                height: 107,
            },
            ..config()
        };
        let tiling = default_tiling();
        let mut plan = RenderPlan::new();
        let mut arena = FrameArena::new();
        let mut cache = PixelTileCache::new();
        let layout = FrameLayout::with_row_alignment(
            PixelFormat::Rgba16F,
            cfg.viewport.width,
            cfg.viewport.height,
            64,
        )
        .expect("padded layout");
        let mut frame = FrameBuffer::new(layout.clone());

        plan.sync(&stage, 0).expect("valid first frame");
        let first_mono = MonoTable::build(&plan, cfg.map).expect("bounded first monotone table");
        let first_binning =
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("bounded first binning");
        let first = FrameJob::new_in(&mut arena, &plan, &first_mono, &first_binning, cfg)
            .expect("matching first frame artifacts")
            .render_into_cached(4, &mut frame, 0, &mut cache)
            .expect("first cached render");
        let first_bytes = frame.as_bytes().to_vec();
        assert_eq!(first.cache.hits, 0, "a cold cache cannot hit");
        assert!(first.cache.misses > 0, "the corpus populated no tiles");

        plan.sync(&stage, 0).expect("valid wait frame");
        let wait_mono = MonoTable::build(&plan, cfg.map).expect("bounded wait monotone table");
        let wait_binning =
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("bounded wait binning");
        let wait = FrameJob::new_in(&mut arena, &plan, &wait_mono, &wait_binning, cfg)
            .expect("matching wait frame artifacts")
            .render_into_cached(4, &mut frame, 0, &mut cache)
            .expect("wait cached render");
        assert_eq!(frame.as_bytes(), first_bytes);
        assert_eq!(wait.cache.misses, 0, "an unchanged tile was rasterized");
        assert_eq!(wait.cache.hits, first.cache.misses);
        assert!(
            wait.raster.sample_evaluations() < first.raster.sample_evaluations(),
            "a cache hit must bypass coverage work, not re-render and compare"
        );

        stage.shift(mobs[0], [24.0, 0.0, 0.0]);
        plan.sync(&stage, 0).expect("valid moved frame");
        let moved_mono = MonoTable::build(&plan, cfg.map).expect("bounded moved monotone table");
        let moved_binning =
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("bounded moved binning");
        let moved = FrameJob::new_in(&mut arena, &plan, &moved_mono, &moved_binning, cfg)
            .expect("matching moved frame artifacts")
            .render_into_cached(4, &mut frame, 0, &mut cache)
            .expect("moved cached render");
        let mut cold = FrameBuffer::new(layout);
        FrameJob::new(&plan, &moved_mono, &moved_binning, cfg)
            .expect("matching cold frame artifacts")
            .render_into(1, &mut cold)
            .expect("cold moved render");
        assert_frames_equal(
            &frame,
            &cold,
            "cached dirty tiles diverged from cold rendering",
        );
        assert!(moved.cache.hits > 0, "movement invalidated unrelated tiles");
        assert!(
            moved.cache.misses > 0,
            "movement reused stale affected tiles"
        );
    }

    #[test]
    fn resumed_surfaces_skip_restores_and_stale_tokens_restore() {
        let (mut stage, mobs) = corpus();
        let cfg = FrameConfig {
            viewport: Viewport {
                width: 109,
                height: 107,
            },
            ..config()
        };
        let tiling = default_tiling();
        let mut plan = RenderPlan::new();
        let mut arena = FrameArena::new();
        let mut cache = PixelTileCache::new();
        let layout = cfg.layout().expect("layout");
        let mut frame = FrameBuffer::new(layout.clone());
        assert_eq!(cache.serial(), 0, "a fresh cache names no completed render");

        plan.sync(&stage, 0).expect("valid first frame");
        let (mono, binning) = (
            MonoTable::build(&plan, cfg.map).expect("first monotone table"),
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("first binning"),
        );
        FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("first artifacts")
            .render_into_cached_resuming(4, &mut frame, 0, &mut cache, Some(0))
            .expect("first render");
        let token = cache.serial();
        assert_ne!(token, 0, "a completed render must name itself");

        // Resume into the surface the cache just filled: reused tiles stay put.
        stage.shift(mobs[0], [24.0, 0.0, 0.0]);
        plan.sync(&stage, 0).expect("valid moved frame");
        let (mono, binning) = (
            MonoTable::build(&plan, cfg.map).expect("moved monotone table"),
            Binning::build(&plan, cfg.viewport, tiling, cfg.map).expect("moved binning"),
        );
        let moved = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("moved artifacts")
            .render_into_cached_resuming(4, &mut frame, 0, &mut cache, Some(token))
            .expect("resumed render");
        let mut cold = FrameBuffer::new(layout.clone());
        FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("cold artifacts")
            .render_into(1, &mut cold)
            .expect("cold render");
        assert_frames_equal(
            &frame,
            &cold,
            "a resumed surface diverged from cold rendering",
        );
        assert!(moved.cache.hits > 0, "the resumed frame reused no tiles");
        assert_ne!(
            cache.serial(),
            token,
            "a new render must retire the old token"
        );

        // The old token is stale now: a foreign surface must be fully restored.
        let mut foreign = FrameBuffer::new(layout);
        foreign.as_bytes_mut().fill(0xEE);
        let again = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("repeat artifacts")
            .render_into_cached_resuming(4, &mut foreign, 0, &mut cache, Some(token))
            .expect("stale-token render");
        assert_eq!(
            again.cache.misses, 0,
            "an unchanged frame rasterized a tile"
        );
        assert_frames_equal(&foreign, &cold, "a stale token skipped a restore");
    }

    #[test]
    fn retained_pixels_invalidate_when_renderer_configuration_changes() {
        let (stage, _) = corpus();
        let cfg = config();
        let tiling = default_tiling();
        let (plan, mono, binning) = derive(&stage, cfg, tiling);
        let mut cache = PixelTileCache::new();
        let mut frame = FrameBuffer::new(cfg.layout().expect("layout"));
        FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching first artifacts")
            .render_into_cached(1, &mut frame, 0, &mut cache)
            .expect("first cached render");

        let changed = FrameConfig {
            background: LinearRgba {
                r: 0.2,
                g: 0.1,
                b: 0.05,
                a: 1.0,
            },
            ..cfg
        };
        let changed_render = FrameJob::new(&plan, &mono, &binning, changed)
            .expect("matching changed artifacts")
            .render_into_cached(1, &mut frame, 0, &mut cache)
            .expect("changed cached render");
        let cold = FrameJob::new(&plan, &mono, &binning, changed)
            .expect("matching cold changed artifacts")
            .render(1)
            .expect("cold changed render");
        assert_eq!(
            changed_render.cache.hits, 0,
            "background changes must invalidate every retained pixel"
        );
        assert_frames_equal(
            &frame,
            &cold,
            "renderer-state invalidation served stale background pixels",
        );
    }

    /// Even a one-band frame must warm the whole requested team. Without
    /// synchronous preparation, the first worker can finish that single band
    /// and return its slot before later closures start, making the next frame's
    /// allocation count depend on scheduler timing.
    #[test]
    fn one_band_warmup_sizes_the_requested_team_before_fanout() {
        let stage = Stage::new();
        let cfg = FrameConfig::new(
            Viewport {
                width: 16,
                height: 16,
            },
            ScreenMap::default(),
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        );
        let tiling = Tiling {
            macro_tile: 16,
            fine_tile: 16,
        };
        let (plan, mono, binning) = derive(&stage, cfg, tiling);
        let mut arena = FrameArena::new();
        let mut buffer = FrameBuffer::new(cfg.layout().expect("layout"));

        let warm = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("matching warm frame artifacts");
        warm.render_into(4, &mut buffer)
            .expect("the warm frame renders");
        let warm_stats = warm.allocation_stats();
        assert_eq!(warm_stats.pool_slots, 4);
        assert!(warm_stats.heap_allocs_this_frame > 0);
        drop(warm);

        let measured = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("matching measured frame artifacts");
        measured
            .render_into(4, &mut buffer)
            .expect("the measured frame renders");
        let measured_stats = measured.allocation_stats();
        assert_eq!(measured_stats.pool_slots, 4);
        assert_eq!(measured_stats.heap_allocs_this_frame, 0);
    }

    #[test]
    fn worker_pool_refuses_unrepresentable_warmup_atomically() {
        for (tile, cols, workers) in [(usize::MAX, 1, 1), (1, usize::MAX, 1), (1, 1, usize::MAX)] {
            let pool = WorkerPool::new();
            assert_eq!(
                pool.prepare::<CertifiedScalar>(tile, cols, workers),
                Err(FrameError::TooLarge)
            );
            assert_eq!(pool.slots(), 0);
            assert_eq!(pool.allocs(), 0);
        }
    }

    #[test]
    fn refused_2d_worker_start_leaves_destination_untouched() {
        let stage = Stage::new();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let mut arena = FrameArena::new();
        let job = FrameJob::new_in(&mut arena, &plan, &mono, &binning, cfg)
            .expect("matching frame artifacts");
        let mut destination = FrameBuffer::new(cfg.layout().expect("layout"));
        destination.plane_mut(0).fill(0xA5);
        let untouched = destination.plane(0).to_vec();
        let spawner = crate::engine::test_support::RefusingScopedSpawner::new(1);

        assert_eq!(
            job.render_into_profiled_with_spawner::<CertifiedScalar, _>(
                4,
                &mut destination,
                &spawner,
            ),
            Err(FrameError::WorkerSpawnFailed {
                requested: 4,
                spawned: 1,
            })
        );
        assert_eq!(spawner.attempts(), 2);
        assert_eq!(destination.plane(0), untouched);
    }

    #[test]
    fn returning_prepared_workers_never_grows_the_idle_pool() {
        let pool = WorkerPool::new();
        pool.prepare::<CertifiedScalar>(1, 1, 1)
            .expect("first scratch key");
        let first = pool.checkout::<CertifiedScalar>(1, 1);
        pool.prepare::<CertifiedScalar>(2, 1, 1)
            .expect("second scratch key while the first is checked out");
        let second = pool.checkout::<CertifiedScalar>(2, 1);

        pool.begin_frame();
        drop(first);
        drop(second);
        assert_eq!(pool.allocs(), 0);
        assert_eq!(pool.slots(), 2);
    }

    #[test]
    fn oversized_scheduling_tiles_size_scratch_to_the_covered_row() {
        let stage = Stage::new();
        let cfg = FrameConfig::new(
            Viewport {
                width: 1,
                height: 1,
            },
            ScreenMap::default(),
            LinearRgba {
                r: 0.125,
                g: 0.25,
                b: 0.5,
                a: 1.0,
            },
        );
        let oversized = Tiling {
            macro_tile: u32::MAX,
            fine_tile: u32::MAX,
        };
        let (plan, mono, oversized_binning) = derive(&stage, cfg, oversized);
        let ordinary_binning = Binning::build(&plan, cfg.viewport, Tiling::default(), cfg.map)
            .expect("ordinary tiny-frame binning");
        let mut arena = FrameArena::new();

        let mut oversized_frame = FrameBuffer::new(cfg.layout().expect("tiny layout"));
        let oversized_job = FrameJob::new_in(&mut arena, &plan, &mono, &oversized_binning, cfg)
            .expect("matching oversized-tile artifacts");
        oversized_job
            .render_into(1, &mut oversized_frame)
            .expect("one covered pixel cannot inherit the requested tile allocation");
        assert_eq!(oversized_job.allocation_stats().pool_slots, 1);
        let oversized_digest = frame_digest(&oversized_frame).expect("oversized-tile digest");
        drop(oversized_job);

        let mut ordinary_frame = FrameBuffer::new(cfg.layout().expect("tiny layout"));
        let ordinary_job = FrameJob::new_in(&mut arena, &plan, &mono, &ordinary_binning, cfg)
            .expect("matching ordinary artifacts");
        ordinary_job
            .render_into(1, &mut ordinary_frame)
            .expect("ordinary tiny frame");
        assert_eq!(
            ordinary_job.allocation_stats().heap_allocs_this_frame,
            0,
            "both schedules cover the same one-pixel scratch width"
        );
        assert_eq!(
            frame_digest(&ordinary_frame).expect("ordinary digest"),
            oversized_digest,
            "scratch sizing is not a semantic tiling choice"
        );
    }

    /// The brute-force reference: the same tiling and the same classification,
    /// but **every instance considered for every tile** and no slab rejection.
    ///
    /// This is the test the rest of the engine's optimizations are measured
    /// against. Binning, its two levels, its per-command class word, and the
    /// per-row/per-column slab rejects all claim to remove only work that could
    /// not have changed a pixel; this recomputes the frame with none of them and
    /// compares bytes.
    fn render_reference(job: &FrameJob<'_>) -> FrameBuffer {
        let cfg = job.config;
        let tile = job.binning.tiling().fine_tile;
        let mut buffer = FrameBuffer::new(cfg.layout().expect("layout"));
        let stride = buffer.layout().stride(0);
        let bg = cfg.background.premultiply();
        let mut scratch = RowScratch::for_tile(tile).expect("reference row scratch");
        let mut cov = vec![0.0f64; tile as usize];

        let rows = cfg.viewport.height.div_ceil(tile);
        for ty in 0..rows {
            for tx in 0..job.cols {
                let x_lo = tx * tile;
                let x_hi = (x_lo + tile).min(cfg.viewport.width);
                let rect = [
                    f64::from(x_lo),
                    f64::from(ty * tile),
                    f64::from(x_hi),
                    f64::from((ty * tile + tile).min(cfg.viewport.height)),
                ];
                let w = (x_hi - x_lo) as usize;
                for py in (ty * tile)..((ty * tile + tile).min(cfg.viewport.height)) {
                    let mut acc = vec![bg; w];
                    for (d, rec) in job.draws().iter().enumerate() {
                        // The draw list is index-aligned with the instance list,
                        // so the enumeration index IS the instance index — which
                        // is exactly the property the engine relies on and this
                        // reference therefore must not paper over.
                        let Some(rec) = rec.as_ref() else { continue };
                        let inst = &job.plan.shapes().instances()[d];
                        let interior = covers_tile(job.plan, inst, cfg.map, rect);
                        let mut fill = |acc: &mut Vec<PremulRgba>| {
                            if !rec.draws_fill {
                                return;
                            }
                            if interior {
                                cov[..w].copy_from_slice(scratch.interior_row(x_lo, x_hi));
                            } else if !rec.kernel.row(py, x_lo, x_hi, &mut cov[..w]) {
                                cov[..w].copy_from_slice(scratch.fill_row(
                                    job.pieces_of(rec),
                                    rec.translate,
                                    py,
                                    x_lo,
                                    x_hi,
                                ));
                            }
                            let segments = job.segments_of(rec);
                            for i in 0..w {
                                if cov[i] <= 0.0 {
                                    continue;
                                }
                                let p = [f64::from(x_lo + i as u32) + 0.5, f64::from(py) + 0.5];
                                let rgba = match rec.flat_fill {
                                    Some(c) => c,
                                    None => fill_rgba_with_border(
                                        &rec.style,
                                        &job.field_of(rec).expect("field"),
                                        segments,
                                        cfg.map,
                                        rec.translate,
                                        p,
                                    ),
                                };
                                acc[i] = source_over(rgba, cov[i], acc[i]);
                            }
                        };
                        let stroke = |acc: &mut Vec<PremulRgba>| {
                            if !rec.draws_stroke {
                                return;
                            }
                            let segments = job.segments_of(rec);
                            for (i, dst) in acc.iter_mut().enumerate().take(w) {
                                let p = [f64::from(x_lo + i as u32) + 0.5, f64::from(py) + 0.5];
                                let (c, s) = stroke_shade(
                                    segments,
                                    job.joins_of(rec),
                                    &rec.style,
                                    cfg.map,
                                    rec.translate,
                                    p,
                                );
                                if c <= 0.0 {
                                    continue;
                                }
                                *dst = source_over(stroke_rgba_at(&rec.style, s), c, *dst);
                            }
                        };
                        if rec.style.stroke_behind {
                            stroke(&mut acc);
                            fill(&mut acc);
                        } else {
                            fill(&mut acc);
                            stroke(&mut acc);
                        }
                    }
                    let base = py as usize * stride + x_lo as usize * 8;
                    write_row(&acc, &mut buffer.plane_mut(0)[base..base + w * 8]);
                }
            }
        }
        buffer
    }

    #[test]
    fn the_draw_list_is_index_aligned_with_the_instance_list() {
        // Stated as its own assertion because it is a *representation* invariant
        // the whole tile loop rests on, and because the first version of this
        // module compacted the list and was wrong for it.
        let mut stage = Stage::new();
        let ghost = stage.add(vmob(
            &ring(40.0, 40.0, 10.0, 8, true),
            [1.0, 1.0, 1.0, 0.0],
            [0.0; 4],
            0.0,
        ));
        stage.add_to_scene(ghost).expect("live");
        let solid = stage.add(vmob(
            &ring(70.0, 40.0, 10.0, 8, true),
            [1.0, 0.0, 0.0, 1.0],
            [0.0; 4],
            0.0,
        ));
        stage.add_to_scene(solid).expect("live");

        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        assert_eq!(job.draws().len(), plan.shapes().instances().len());
        assert_eq!(job.draws().len(), 2);
        assert!(job.draws()[0].is_none(), "the ghost must hold its slot");
        assert!(job.draws()[1].is_some());
        assert_eq!(job.draw_count(), 1, "and must not be counted as a draw");
    }

    #[test]
    fn affine_instance_render_matches_the_same_geometry_baked_to_points() {
        let make_stage = |bake: bool| {
            let mut stage = Stage::new();
            let points = rect_points(36.0, 44.0, 76.0, 68.0);
            let mob = stage.add(vmob(
                &points,
                [0.2, 0.7, 0.9, 0.8],
                [1.0, 0.3, 0.1, 1.0],
                240.0,
            ));
            let last = points.len() - 1;
            let entry = stage.get_mut(mob).expect("live");
            entry.buffer.write(last, "fill_rgba", &[0.9, 0.2, 0.4, 0.8]);
            entry
                .buffer
                .write(last, "stroke_rgba", &[0.1, 0.8, 0.3, 1.0]);
            entry.buffer.write(last, "stroke_width", &[480.0]);
            stage.add_to_scene(mob).expect("live");
            let pivot = [56.0, 56.0, 0.0];
            stage.apply_affine(
                mob,
                Placement::about([[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], pivot),
            );
            stage.stretch_about(mob, 1.5, 0, Some(pivot), None);
            stage.stretch_about(mob, 0.75, 1, Some(pivot), None);
            if bake {
                stage.bake_placement(mob).expect("live");
            }
            stage
        };
        let placed = make_stage(false);
        let baked = make_stage(true);
        let cfg = config();

        let (placed_plan, placed_mono, placed_binning) = derive(&placed, cfg, default_tiling());
        let placed_frame = FrameJob::new(&placed_plan, &placed_mono, &placed_binning, cfg)
            .expect("placed artifacts")
            .render(1)
            .expect("placed render");
        let (baked_plan, baked_mono, baked_binning) = derive(&baked, cfg, default_tiling());
        let baked_frame = FrameJob::new(&baked_plan, &baked_mono, &baked_binning, cfg)
            .expect("baked artifacts")
            .render(1)
            .expect("baked render");

        assert_frames_equal(
            &placed_frame,
            &baked_frame,
            "affine placement diverged from the same world-space geometry",
        );
    }

    #[test]
    fn uniform_scale_keeps_the_retained_normalized_arc_length_bits() {
        let mut stage = Stage::new();
        let points = [
            [12.0, 18.0, 0.0],
            [18.0, 2.0, 0.0],
            [42.0, 20.0, 0.0],
            [43.0, 20.5, 0.0],
            [48.0, 26.0, 0.0],
        ];
        let mob = stage.add(vmob(
            &points,
            [0.2, 0.7, 0.9, 0.8],
            [1.0, 0.3, 0.1, 1.0],
            240.0,
        ));
        let last = points.len() - 1;
        let entry = stage.get_mut(mob).expect("live");
        entry.buffer.write(last, "fill_rgba", &[0.9, 0.2, 0.4, 0.8]);
        entry
            .buffer
            .write(last, "stroke_rgba", &[0.1, 0.8, 0.3, 1.0]);
        entry.buffer.write(last, "stroke_width", &[480.0]);
        stage.add_to_scene(mob).expect("live");
        stage.apply_affine(
            mob,
            Placement::new(
                [[1.125, 0.0, 0.0], [0.0, 1.125, 0.0], [0.0, 0.0, 1.125]],
                [7.0, -3.0, 0.0],
            ),
        );

        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        let instance = &plan.shapes().instances()[0];
        let shape = plan.shapes().shape(instance.shape).expect("instance shape");
        let lo = shape.first_segment as usize;
        let hi = lo + shape.segment_count as usize;
        let retained = &plan.segments()[lo..hi];
        let rec = job.draws()[0].as_ref().expect("visible draw");
        assert!(
            rec.transformed_segments.is_some(),
            "uniform scale still transforms coefficients"
        );
        let transformed = job.segments_of(rec);

        assert_eq!(transformed.len(), retained.len());
        for (index, (actual, expected)) in transformed.iter().zip(retained).enumerate() {
            assert_eq!(
                [actual.s0.to_bits(), actual.s1.to_bits()],
                [expected.s0.to_bits(), expected.s1.to_bits()],
                "segment {index} did not retain its normalized arc-length span"
            );
        }
    }

    #[test]
    fn the_engine_matches_a_brute_force_reference() {
        // The load-bearing test of the whole module. Two-level binning, the
        // per-command interior class, the per-row slab reject and the per-column
        // one all claim to remove only work that could not have moved a pixel.
        // Here is the frame with none of them, byte for byte.
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        let fast = job.render(1).expect("render");
        let reference = render_reference(&job);
        assert_frames_equal(&fast, &reference, "an optimization changed the picture");
    }

    #[test]
    fn the_frame_is_identical_at_every_thread_count() {
        // PG-5, per commit. Thread count is outside the input closure (§16.7)
        // and this is the assertion that makes that true rather than hoped.
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        let one = job.render(1).expect("render");
        for threads in [2usize, 4, 16] {
            let many = job.render(threads).expect("render");
            assert_eq!(
                one.as_bytes(),
                many.as_bytes(),
                "the frame moved at {threads} threads"
            );
        }
    }

    #[test]
    fn certified_bits_ignore_adaptive_and_forced_aa_policies() {
        // §10.4 is absolute: certified executes the canonical analytic path.
        // The A/B knob remains journaled, but it cannot move a raw-frame bit.
        let (stage, _) = corpus();
        let tiling = default_tiling();
        let mut definition = None;
        for aa in [AaPolicy::Adaptive, AaPolicy::Ssaa2x, AaPolicy::Ssaa4x] {
            let cfg = config().with_aa_policy(aa);
            let (plan, mono, binning) = derive(&stage, cfg, tiling);
            let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
            let (frame, stats) = job.render_with_stats(4).expect("render");
            assert_eq!(stats.native_cells, stats.output_cells);
            assert_eq!(stats.ssaa2x_cells, 0);
            assert_eq!(stats.ssaa4x_cells, 0);
            match &definition {
                Some(bytes) => {
                    assert_eq!(bytes, frame.as_bytes(), "certified bits moved under {aa:?}")
                }
                None => definition = Some(frame.as_bytes().to_vec()),
            }
        }
    }

    #[test]
    fn adaptive_classification_is_thread_independent_and_avoids_forced_work() {
        let (stage, _) = corpus();
        let tiling = default_tiling();
        let identity = EngineIdentity {
            engine: EngineKind::FastCpu,
            ..EngineIdentity::certified()
        };

        let adaptive_cfg = config().with_aa_policy(AaPolicy::Adaptive);
        let (plan, mono, binning) = derive(&stage, adaptive_cfg, tiling);
        let job = FrameJob::with_identity(&plan, &mono, &binning, adaptive_cfg, identity)
            .expect("matching frame artifacts");
        let (one, one_stats) = job.render_with_stats(1).expect("render");
        let (many, many_stats) = job.render_with_stats(4).expect("render");
        assert_frames_equal(&one, &many, "adaptive output depends on thread count");
        assert_eq!(one_stats, many_stats, "adaptive stats depend on workers");
        assert_eq!(one_stats.classified_tiles(), binning.tile_count() as u64);
        assert!(
            one_stats.complex_edge_tiles > 0,
            "the overlap corpus classified no complex tile: {one_stats:?}"
        );
        assert!(
            one_stats.ssaa2x_cells > 0 && one_stats.ssaa4x_cells > 0,
            "both adaptive thresholds must be exercised: {one_stats:?}"
        );
        assert_eq!(
            (one_stats.ssaa2x_cells, one_stats.ssaa4x_cells),
            (29, 34),
            "the documented W5 work measurement moved"
        );
        assert_eq!(
            one_stats.sample_evaluations(),
            13_204,
            "the documented W5 work measurement moved"
        );
        assert!(
            one_stats.work_reduction_vs_forced_4x() > 0.5,
            "adaptive did not avoid most forced work: {one_stats:?}"
        );

        let forced_cfg = config().with_aa_policy(AaPolicy::Ssaa4x);
        let forced_job = FrameJob::with_identity(&plan, &mono, &binning, forced_cfg, identity)
            .expect("AA policy does not stale geometry");
        let (_, forced_stats) = forced_job.render_with_stats(4).expect("render");
        assert_eq!(
            forced_stats.sample_evaluations(),
            forced_stats.forced_4x_evaluations()
        );
        assert!(
            one_stats.sample_evaluations() < forced_stats.sample_evaluations(),
            "{one_stats:?} vs {forced_stats:?}"
        );

        let forced_2x_cfg = config().with_aa_policy(AaPolicy::Ssaa2x);
        let forced_2x_job =
            FrameJob::with_identity(&plan, &mono, &binning, forced_2x_cfg, identity)
                .expect("AA policy does not stale geometry");
        let (_, forced_2x_stats) = forced_2x_job.render_with_stats(4).expect("render");
        assert_eq!(forced_2x_stats.native_cells, 0);
        assert_eq!(forced_2x_stats.ssaa2x_cells, forced_2x_stats.output_cells);
        assert_eq!(
            forced_2x_stats.sample_evaluations(),
            forced_2x_stats.output_cells * 4
        );
    }

    /// Strips as a film's blended plate draws them at 64×36: stacked rows far
    /// thinner than a pixel, no AA band, paint varying along each row, and
    /// about 25 curves to the pixel. The corner sits at the frame's centre.
    fn plate_stage(rows: u32, quads: u32) -> Stage {
        let (cell, thick) = (0.00875, 0.00885);
        let count = (2 * quads + 1) as usize;
        let mut stage = Stage::new();
        for row in 0..rows {
            let y = cell * f64::from(row);
            let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), count).unwrap();
            for i in 0..count {
                let x = 0.5 * cell * i as f64;
                let t = i as f32 / (count - 1) as f32;
                buffer.write(i, "point", &[x as f32, y as f32, 0.0]);
                buffer.write(i, "fill_rgba", &[0.0f32; 4]);
                buffer.write(i, "stroke_rgba", &[t, 1.0 - t, 0.5, 1.0]);
                buffer.write(i, "stroke_width", &[(thick / 0.01) as f32]);
            }
            let h = stage.add(Mobject::from_buffer(buffer));
            stage.add_to_scene(h).expect("live");
            stage.set_anti_alias_width(h, 0.0, false);
        }
        stage
    }

    #[test]
    fn the_x_index_shades_a_plate_exactly_as_the_scan() {
        const ROWS: u32 = 64;
        let stage = plate_stage(ROWS, 64);
        let cfg = FrameConfig::new(
            Viewport {
                width: 64,
                height: 36,
            },
            ScreenMap::y_up(4.5, [32.0, 18.0]),
            LinearRgba {
                r: 0.04,
                g: 0.04,
                b: 0.06,
                a: 1.0,
            },
        )
        .with_aa_policy(AaPolicy::Adaptive);
        let identity = EngineIdentity {
            engine: EngineKind::FastCpu,
            ..EngineIdentity::certified()
        };
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::with_identity(&plan, &mono, &binning, cfg, identity)
            .expect("matching frame artifacts");
        let (_, stats) = job.render_with_stats(1).expect("render");
        assert!(
            stats.ssaa2x_cells + stats.ssaa4x_cells > 0,
            "the fixture must reach the supersampled route: {stats:?}"
        );

        // Every point any route of the frame can shade, on every strip. The
        // plate spans about (32, 15.5) to (34.5, 18).
        let (mut indexed, mut inked) = (0u32, 0u64);
        for rec in job.draws().iter().flatten() {
            let Some(prepared) = rec.stroke else {
                continue;
            };
            indexed += u32::from(prepared.x_sorted);
            let index = job.stroke_of(rec).expect("a stroked draw");
            let scan = stroke::PreparedStroke::from_parts(
                job.arena().stroke_segments.slice(prepared.segments),
                prepared.slab,
            );
            let (segments, joins) = (job.segments_of(rec), job.joins_of(rec));
            for py in 13..20 {
                for px in 30..37 {
                    for samples in [1, 2, 4] {
                        for y in 0..samples {
                            for x in 0..samples {
                                let p = Subcell {
                                    px,
                                    py,
                                    samples,
                                    x,
                                    y,
                                }
                                .centre();
                                let shade = |view: stroke::PreparedStroke<'_>| {
                                    let (coverage, s) = view.shade(
                                        segments,
                                        joins,
                                        &rec.style,
                                        cfg.map,
                                        rec.translate,
                                        p,
                                    );
                                    (coverage.to_bits(), s.to_bits())
                                };
                                let expected = shade(scan);
                                assert_eq!(shade(index), expected, "shade at {p:?}");
                                inked += u64::from(f64::from_bits(expected.0) > 0.0);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(indexed, ROWS, "every strip must take the index");
        assert!(inked > 0, "the fixture must ink some sample");
    }

    #[test]
    fn strokes_inside_two_aa_bands_do_not_promote_complex_cells() {
        let map = ScreenMap {
            scale: 1.0,
            origin: [0.0, 0.0],
            y_up: false,
        };
        let style = Style {
            stroke_width: 299.0,
            stroke_width_end: 299.0,
            anti_alias_width: 1.5,
            ..Style::default()
        };
        assert!(!stroke_contributes_complexity(&style, map, 0.5));

        let wide = Style {
            stroke_width: 301.0,
            stroke_width_end: 301.0,
            ..style
        };
        assert!(stroke_contributes_complexity(&wide, map, 0.5));
    }

    #[test]
    fn only_width_eligible_overlapping_strokes_escalate() {
        let render_stats = |width| {
            let mut stage = Stage::new();
            for colour in [[1.0, 0.2, 0.1, 1.0], [0.1, 0.4, 1.0, 0.8]] {
                let stroke = vmob(
                    &[[18.0, 40.0, 0.0], [40.0, 40.0, 0.0], [62.0, 40.0, 0.0]],
                    [0.0; 4],
                    colour,
                    width,
                );
                let h = stage.add(stroke);
                stage.add_to_scene(h).expect("live");
            }
            let cfg = config().with_aa_policy(AaPolicy::Adaptive);
            let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
            let identity = EngineIdentity {
                engine: EngineKind::FastCpu,
                ..EngineIdentity::certified()
            };
            FrameJob::with_identity(&plan, &mono, &binning, cfg, identity)
                .expect("matching frame artifacts")
                .render_with_stats(1)
                .expect("render")
                .1
        };

        let inside_band = render_stats(299.0);
        assert_eq!(inside_band.ssaa2x_cells, 0, "{inside_band:?}");
        assert_eq!(inside_band.ssaa4x_cells, 0, "{inside_band:?}");

        let eligible = render_stats(301.0);
        assert!(eligible.ssaa2x_cells > 0, "{eligible:?}");
    }

    #[test]
    fn the_four_tile_classes_are_observable() {
        let mut stage = Stage::new();
        for points in [
            rect_points(2.25, 2.25, 50.25, 50.25),
            ring(72.0, 76.0, 17.0, 8, true),
            ring(84.0, 76.0, 17.0, 8, true),
        ] {
            let h = stage.add(vmob(&points, [1.0, 1.0, 1.0, 1.0], [0.0; 4], 0.0));
            stage.add_to_scene(h).expect("live");
        }
        let cfg = config().with_aa_policy(AaPolicy::Adaptive);
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let identity = EngineIdentity {
            engine: EngineKind::FastCpu,
            ..EngineIdentity::certified()
        };
        let (_, stats) = FrameJob::with_identity(&plan, &mono, &binning, cfg, identity)
            .expect("matching frame artifacts")
            .render_with_stats(1)
            .expect("render");
        assert_eq!(stats.classified_tiles(), binning.tile_count() as u64);
        assert!(
            stats.empty_tiles > 0
                && stats.fully_covered_tiles > 0
                && stats.simple_edge_tiles > 0
                && stats.complex_edge_tiles > 0,
            "not every §10.4 class was reached: {stats:?}"
        );
    }

    #[test]
    fn adaptive_quality_stays_inside_the_forced_4x_visual_budget_v1() {
        let (stage, _) = corpus();
        let tiling = default_tiling();
        let identity = EngineIdentity {
            engine: EngineKind::FastCpu,
            ..EngineIdentity::certified()
        };
        let render = |aa| {
            let cfg = config().with_aa_policy(aa);
            let (plan, mono, binning) = derive(&stage, cfg, tiling);
            FrameJob::with_identity(&plan, &mono, &binning, cfg, identity)
                .expect("matching frame artifacts")
                .render(4)
                .expect("render")
        };
        let adaptive = render(AaPolicy::Adaptive);
        let forced = render(AaPolicy::Ssaa4x);
        let (maximum, rms) = visual_error(&adaptive, &forced);
        assert!(
            maximum <= AA_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
            "max={maximum}, budget={AA_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR}"
        );
        assert!(
            rms <= AA_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
            "rms={rms}, budget={AA_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR}"
        );
    }

    #[test]
    fn fast_cpu_stays_inside_the_certified_visual_budget_v1() {
        let (stage, _) = corpus();
        let cfg = config().with_aa_policy(AaPolicy::Adaptive);
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let certified =
            FrameJob::with_identity(&plan, &mono, &binning, cfg, EngineIdentity::certified())
                .expect("matching certified artifacts")
                .render(4)
                .expect("certified render");
        let fast = FrameJob::with_identity(&plan, &mono, &binning, cfg, EngineIdentity::fast())
            .expect("matching fast artifacts")
            .render(4)
            .expect("fast render");
        let (maximum, rms) = visual_error(&fast, &certified);
        assert!(
            maximum <= FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR,
            "max={maximum}, budget={FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR}"
        );
        assert!(
            rms <= FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR,
            "rms={rms}, budget={FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR}"
        );
    }

    #[test]
    fn an_invisible_instance_does_not_shift_the_ones_behind_it() {
        // Binning's command lists hold **instance** indices, so anything the
        // engine indexes with one has to be aligned with the instance list. An
        // instance with no visible pass — zero fill alpha and zero stroke width,
        // which `set_opacity(0)` and a bare `Group` both produce — must therefore
        // still occupy its slot.
        //
        // The failure this pins is not subtle: with the draw list compacted, a
        // scene whose first object is invisible renders its second object's
        // geometry in its third object's colour, and drops the last one
        // entirely. Every corpus frame happened to have a visible pass on every
        // instance, which is exactly why a scene that does not is the test.
        let build = |with_ghost: bool| {
            let mut stage = Stage::new();
            if with_ghost {
                // Fully transparent fill, zero stroke: composites as the identity.
                let ghost = stage.add(vmob(
                    &ring(56.0, 56.0, 30.0, 8, true),
                    [1.0, 1.0, 1.0, 0.0],
                    [0.0; 4],
                    0.0,
                ));
                stage.add_to_scene(ghost).expect("live");
            }
            for (cx, colour) in [
                (30.0, [1.0f32, 0.0, 0.0, 1.0]),
                (60.0, [0.0, 0.0, 1.0, 1.0]),
                (90.0, [0.0, 1.0, 0.0, 1.0]),
            ] {
                let m = stage.add(vmob(&ring(cx, 40.0, 12.0, 8, true), colour, [0.0; 4], 0.0));
                stage.add_to_scene(m).expect("live");
            }
            let cfg = config();
            let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
            FrameJob::new(&plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render")
        };
        let without = build(false);
        let with = build(true);
        // The ghost draws nothing, so adding it cannot change one pixel.
        assert_frames_equal(&with, &without, "an invisible instance changed the frame");
        // And the three discs must actually be their own colours, which is what
        // says the alignment is right rather than uniformly wrong.
        for (cx, expect) in [(30u32, 0usize), (60, 2), (90, 1)] {
            let px = read_px(&with, cx, 40);
            assert!(
                px[expect] > 0.9,
                "the disc at x={cx} is not its own colour: {px:?}"
            );
        }
    }

    #[test]
    fn occlusion_pruning_is_byte_exact() {
        // G0-8b's finding F13: a tile whose earlier commands are provably hidden
        // must produce the same bytes whether they were drawn or skipped. It is
        // only true because the interior class is exactly 1.0 and not an
        // accumulation an ulp below it.
        let mut stage = Stage::new();
        // A small opaque square, then a larger opaque one drawn over it: inside
        // the overlap the first is provably hidden and may be skipped.
        for (rect, colour) in [
            ([24.0, 24.0, 60.0, 60.0], [1.0f32, 0.0, 0.0, 1.0]),
            ([8.0, 8.0, 96.0, 96.0], [0.0, 0.0, 1.0, 1.0]),
        ] {
            let m = vmob(
                &rect_points(rect[0], rect[1], rect[2], rect[3]),
                colour,
                [0.0; 4],
                0.0,
            );
            let h = stage.add(m);
            stage.add_to_scene(h).expect("live");
        }
        let cfg = config();
        let (plan, mono, mut binning) = derive(&stage, cfg, default_tiling());
        let unpruned = FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching frame artifacts")
            .render(1)
            .expect("render");
        let report = binning.prune_occluded(&plan).expect("matching plan");
        let pruned = FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching frame artifacts")
            .render(1)
            .expect("render");
        assert!(report.before > report.after, "nothing was pruned to test");
        assert_eq!(unpruned.as_bytes(), pruned.as_bytes());
    }

    #[test]
    fn tile_size_is_a_scheduling_choice_within_the_fills_own_tolerance() {
        // Tiling cannot change the picture — but the fill's per-tile carry is a
        // different *association* of the same sum than a longer row's prefix, so
        // the invariance is to floating-point tolerance and not to bits. That is
        // exactly why C10 puts the tile dimensions in the declared certified
        // configuration: a certified run pins them rather than relying on an
        // invariance the arithmetic does not owe.
        let (stage, _) = corpus();
        let cfg = config();
        let reference = {
            let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
            FrameJob::new(&plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render")
        };
        for fine in [8u32, 32, 56] {
            let tiling = Tiling {
                macro_tile: 112,
                fine_tile: fine,
            };
            let (plan, mono, binning) = derive(&stage, cfg, tiling);
            let got = FrameJob::new(&plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render");
            let differing = reference
                .as_bytes()
                .iter()
                .zip(got.as_bytes())
                .filter(|(a, b)| a != b)
                .count();
            // A handful of f16 codes may land on either side of a rounding
            // boundary; a structural difference would move thousands.
            assert!(
                differing * 200 < reference.as_bytes().len(),
                "tile {fine}: {differing} of {} bytes differ",
                reference.as_bytes().len()
            );
        }
    }

    #[test]
    fn a_retained_plan_and_a_fresh_one_render_the_same_frame() {
        // ADR-0013's pinning, as an experiment rather than an assurance.
        //
        // The experiment has to make the interned indices actually diverge, and
        // the obvious way does not: `RenderPlan::sync` walks the draw plan in
        // painter order and interns as it goes, so on a *fresh* plan the
        // interning order IS painter order and nothing varies. Indices diverge
        // when a plan is **retained** and an object is added behind ones already
        // compiled — the newcomer draws first and interns last. That is also the
        // normal path, so it is the one worth pinning.
        let scene = |stage: &mut Stage| {
            for (cx, colour) in [
                (34.0, [1.0f32, 0.0, 0.0, 0.8]),
                (60.0, [0.0, 0.0, 1.0, 0.6]),
            ] {
                let h = stage.add(vmob(&ring(cx, 40.0, 15.0, 8, true), colour, [0.0; 4], 0.0));
                stage.add_to_scene(h).expect("live");
            }
        };
        let newcomer = |stage: &mut Stage| {
            let h = stage.add(vmob(
                &ring(46.0, 62.0, 19.0, 8, true),
                [0.0, 1.0, 0.0, 0.7],
                [0.0; 4],
                0.0,
            ));
            stage.set_z_index(h, -5, false);
            stage.add_to_scene(h).expect("live");
        };

        let cfg = config();

        // Retained: two objects compiled, then a third inserted *behind* them.
        let mut retained_stage = Stage::new();
        scene(&mut retained_stage);
        let mut retained = RenderPlan::new();
        retained
            .sync(&retained_stage, 0)
            .expect("valid retained fixture");
        newcomer(&mut retained_stage);
        retained
            .sync(&retained_stage, 0)
            .expect("valid retained fixture");

        // Fresh: the same final scene, compiled in one pass.
        let mut fresh_stage = Stage::new();
        scene(&mut fresh_stage);
        newcomer(&mut fresh_stage);
        let mut fresh = RenderPlan::new();
        fresh.sync(&fresh_stage, 0).expect("valid fresh fixture");

        // The precondition, asserted rather than assumed: the two plans really
        // do assign different table indices to the same painter position. Without
        // this the test could pass by comparing a scene with itself.
        let idx = |p: &RenderPlan| -> Vec<u32> {
            p.shapes().instances().iter().map(|i| i.shape).collect()
        };
        assert_eq!(idx(&retained).len(), 3);
        assert_ne!(
            idx(&retained),
            idx(&fresh),
            "the interning orders did not diverge, so this proves nothing"
        );

        let render = |plan: &RenderPlan| {
            let mono = MonoTable::build(plan, cfg.map).expect("bounded test monotone table");
            let binning = Binning::build(plan, cfg.viewport, default_tiling(), cfg.map)
                .expect("bounded test binning");
            FrameJob::new(plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render")
        };
        assert_frames_equal(
            &render(&retained),
            &render(&fresh),
            "the interning order reached a pixel",
        );
    }

    #[test]
    fn painter_order_decides_which_colour_wins() {
        let mut stage = Stage::new();
        let mut handles = Vec::new();
        for colour in [[1.0f32, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]] {
            let m = vmob(&ring(40.0, 40.0, 20.0, 8, true), colour, [0.0; 4], 0.0);
            let h = stage.add(m);
            stage.add_to_scene(h).expect("live");
            handles.push(h);
        }
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let frame = FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching frame artifacts")
            .render(1)
            .expect("render");
        // Dead centre: the second disc, drawn last and opaque, wins outright.
        let px = read_px(&frame, 40, 40);
        assert!(px[0] < 1e-3, "red leaked through an opaque green: {px:?}");
        assert!((px[1] - 1.0).abs() < 1e-3, "green is not opaque: {px:?}");
        assert!((px[3] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn the_background_survives_where_nothing_draws() {
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let frame = FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching frame artifacts")
            .render(1)
            .expect("render");
        let px = read_px(&frame, 108, 4);
        assert!((px[0] - 0.04).abs() < 2e-3, "{px:?}");
        assert!((px[2] - 0.06).abs() < 2e-3, "{px:?}");
        assert!((px[3] - 1.0).abs() < 1e-3, "{px:?}");
    }

    #[test]
    fn stroke_behind_swaps_the_passes() {
        // R-5: within one object the fill draws before the stroke, unless
        // `stroke_behind`. A wide stroke under a translucent fill reads
        // differently from the same stroke over it, and that difference is the
        // whole observable content of the flag.
        let make = |behind: bool| {
            let mut stage = Stage::new();
            let m = vmob(
                &ring(40.0, 40.0, 18.0, 8, true),
                [1.0, 1.0, 1.0, 0.5],
                [1.0, 0.0, 0.0, 1.0],
                800.0,
            );
            let h = stage.add(m);
            stage.set_stroke_behind(h, behind, false);
            stage.add_to_scene(h).expect("live");
            let cfg = config();
            let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
            let frame = FrameJob::new(&plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render");
            read_px(&frame, 40, 22)
        };
        let over = make(false);
        let behind = make(true);
        assert!(
            (over[1] - behind[1]).abs() > 0.05,
            "stroke_behind changed nothing: {over:?} vs {behind:?}"
        );
        // With the stroke on top the red is undiluted; behind, the translucent
        // white fill lightens it.
        assert!(over[1] < behind[1], "{over:?} vs {behind:?}");
    }

    #[test]
    fn a_pooled_buffer_renders_the_same_frame_as_a_fresh_one() {
        // PG-6's steady state: the engine writes every payload byte it is
        // responsible for, so a buffer carrying a previous frame's contents must
        // not leak any of them.
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        let fresh = job.render(1).expect("render");
        let mut dirty = FrameBuffer::new(cfg.layout().expect("layout"));
        dirty.as_bytes_mut().fill(0xa5);
        job.render_into(4, &mut dirty).expect("render");
        assert_eq!(fresh.as_bytes(), dirty.as_bytes());
    }

    #[test]
    fn the_frame_digest_ignores_stride_padding() {
        // A pooled frame carries whatever the last one left in its padding.
        // Hashing the whole allocation would make a self-golden a function of
        // pool history; hashing payload rows makes it a function of the picture.
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        let tight = job.render(1).expect("render");

        let padded_layout = FrameLayout::with_row_alignment(
            PixelFormat::Rgba16F,
            cfg.viewport.width,
            cfg.viewport.height,
            256,
        )
        .expect("padded layout");
        let mut padded = FrameBuffer::new(padded_layout);
        padded.as_bytes_mut().fill(0x5c);
        job.render_into(1, &mut padded).expect("render");

        assert!(
            padded.layout().stride(0) > tight.layout().stride(0),
            "the padded layout is not actually padded"
        );
        assert_ne!(
            padded.as_bytes(),
            tight.as_bytes(),
            "padding did not differ"
        );
        assert_eq!(
            frame_digest(&tight).expect("digest"),
            frame_digest(&padded).expect("digest"),
            "the digest saw the padding"
        );
    }

    #[test]
    fn the_raw_frame_converts_to_a_canonical_png_payload() {
        // The certified artifact chain: raw frame -> canonical RGBA8. The
        // conversion is fmn-frame's table-driven kernel, so this asserts the
        // engine's output is in the format that kernel accepts, and that an
        // opaque frame stays opaque through it.
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let raw = FrameJob::new(&plan, &mono, &binning, cfg)
            .expect("matching frame artifacts")
            .render(1)
            .expect("render");
        let mut rgba8 = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, cfg.viewport.width, cfg.viewport.height)
                .expect("layout"),
        );
        fmn_frame::convert::rgba16f_to_rgba8(&raw, &mut rgba8).expect("convert");
        assert!(
            rgba8
                .plane(0)
                .as_chunks::<4>()
                .0
                .iter()
                .all(|px| px[3] == 255)
        );
        // The background is 0.04 linear, which is emphatically not 10/255.
        let corner = &rgba8.plane(0)[..4];
        assert!(corner[0] > 40 && corner[0] < 70, "{corner:?}");
    }

    #[test]
    fn an_empty_scene_is_exactly_the_background() {
        let stage = Stage::new();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
        assert_eq!(job.draw_count(), 0);
        let frame = job.render(4).expect("render");
        let expect = {
            let bg = cfg.background;
            let mut px = [0u8; 8];
            for (k, v) in [bg.r, bg.g, bg.b, bg.a].into_iter().enumerate() {
                px[k * 2..k * 2 + 2]
                    .copy_from_slice(&fmn_frame::half::f16_from_f32(v as f32).to_le_bytes());
            }
            px
        };
        assert!(
            frame
                .plane(0)
                .as_chunks::<8>()
                .0
                .iter()
                .all(|px| px.eq(&expect))
        );
    }

    #[test]
    fn the_identity_journal_separates_what_it_must() {
        let cfg = config();
        let tiling = default_tiling();
        assert_eq!(EngineIdentity::default(), EngineIdentity::fast());
        assert_eq!(EngineIdentity::fast().tier, Tier::COMPILED);
        let base = journal_digest(EngineIdentity::certified(), &cfg, tiling);
        assert_eq!(
            base,
            journal_digest(EngineIdentity::certified(), &cfg, tiling)
        );

        // A different engine is a different identity even at the same config —
        // §10.5(f) exists so a manifest cannot claim the certified engine
        // produced what an annex engine did.
        let annex = EngineIdentity {
            engine: EngineKind::Metal,
            ..EngineIdentity::certified()
        };
        assert_ne!(base, journal_digest(annex, &cfg, tiling));
        assert!(!annex.engine.certifiable());
        assert!(EngineKind::CertifiedCpu.certifiable());

        let built_tier = EngineIdentity {
            tier: Tier::COMPILED,
            ..EngineIdentity::certified()
        };
        assert_ne!(
            base,
            journal_digest(built_tier, &cfg, tiling),
            "the build tier is part of C3/C7 provenance"
        );

        // C10's declared configuration: the tile dimensions are in the closure
        // because the fill's per-tile carry is association-dependent.
        let other_tiling = Tiling {
            macro_tile: 64,
            fine_tile: 32,
        };
        assert_ne!(
            base,
            journal_digest(EngineIdentity::certified(), &cfg, other_tiling)
        );

        // And the viewport, the map, and the background.
        let mut moved = cfg;
        moved.map.origin[0] += 1.0;
        assert_ne!(
            base,
            journal_digest(EngineIdentity::certified(), &moved, tiling)
        );
        let mut recoloured = cfg;
        recoloured.background.g = 0.5;
        assert_ne!(
            base,
            journal_digest(EngineIdentity::certified(), &recoloured, tiling)
        );

        // The declared A/B policy is provenance even though certified
        // normalizes every policy to the same analytic pixel path.
        let forced = cfg.with_aa_policy(AaPolicy::Ssaa4x);
        assert_ne!(
            base,
            journal_digest(EngineIdentity::certified(), &forced, tiling)
        );
    }

    #[test]
    fn a_job_refuses_an_identity_it_cannot_truthfully_execute() {
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());

        let requested = RENDERER_VERSION.wrapping_add(1);
        let stale = EngineIdentity {
            renderer_version: requested,
            ..EngineIdentity::certified()
        };
        assert!(matches!(
            FrameJob::with_identity(&plan, &mono, &binning, cfg, stale),
            Err(FrameJobError::RendererVersionMismatch {
                requested: got,
                compiled: RENDERER_VERSION,
            }) if got == requested
        ));

        for engine in [EngineKind::Metal, EngineKind::Cuda] {
            let annex = EngineIdentity {
                engine,
                ..EngineIdentity::certified()
            };
            assert!(matches!(
                FrameJob::with_identity(&plan, &mono, &binning, cfg, annex),
                Err(FrameJobError::UnsupportedEngine { engine: got }) if got == engine
            ));
        }
    }

    #[test]
    fn affine_job_refuses_camera_and_depth_vector_semantics() {
        let (mut stage, mobs) = corpus();
        stage.uniforms_mut(mobs[0]).expect("live").depth_test = true;
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        assert!(matches!(
            FrameJob::new(&plan, &mono, &binning, cfg),
            Err(FrameJobError::CameraProjectionRequired { instance: 0 })
        ));
    }

    #[test]
    fn the_scalar_oracle_and_compiled_tier_are_both_listed() {
        // The harness in fmn-conformance sweeps `Tier::ALL`; a tier that lands
        // without being listed is a tier nothing checks.
        assert_eq!(Tier::ALL, &[Tier::Scalar, Tier::COMPILED]);
        assert_eq!(Tier::ALL[0].name(), "scalar");
        assert_ne!(Tier::COMPILED, Tier::Scalar);
        assert_ne!(Tier::COMPILED.name(), "scalar");
    }

    #[test]
    fn compiled_pixel_kernels_preserve_scalar_accumulator_bits() {
        fn bits(pixel: PremulRgba) -> [u64; 4] {
            [pixel.r, pixel.g, pixel.b, pixel.a].map(f64::to_bits)
        }

        fn check<S: PixelKernel, T: PixelKernel>() {
            let colours = [
                [0.0, 0.0, 0.0, 0.0],
                [1.0, 0.25, 0.5, 1.0],
                [0.25, 0.5, 0.75, 0.123],
                [-0.25, 2.0, 0.000_000_1, 0.99],
                [f32::MIN_POSITIVE, f32::from_bits(1), 1.0, f32::MIN_POSITIVE],
            ];
            let coverages = [
                0.0,
                -0.0,
                -0.5,
                1.0 / 3.0,
                f64::MIN_POSITIVE,
                f64::from_bits(1),
                1.0,
                0.500_000_000_000_000_1,
            ];
            let backgrounds = [
                PremulRgba::TRANSPARENT,
                PremulRgba {
                    r: -0.0,
                    g: 0.0,
                    b: -0.0,
                    a: -0.0,
                },
                LinearRgba {
                    r: 0.11,
                    g: 0.37,
                    b: 0.93,
                    a: 0.73,
                }
                .premultiply(),
            ];
            for background in backgrounds {
                // Also check the single-pixel entry used by gradients and strokes.
                for colour in colours {
                    for coverage in coverages {
                        let scalar = S::source_over(colour, coverage, S::from_premul(background));
                        let tier = T::source_over(colour, coverage, T::from_premul(background));
                        assert_eq!(bits(S::to_premul(scalar)), bits(T::to_premul(tier)));
                    }
                }
                for len in [0, 1, 2, 3, 4, 5, 7, 8, 15, 16, 17, 31, 32, 33] {
                    let mut scalar = vec![S::from_premul(background); len];
                    let mut tier = vec![T::from_premul(background); len];
                    for layer in 0..64 {
                        let coverage: Vec<_> = (0..len)
                            .map(|index| {
                                if layer % 2 == 0 {
                                    1.0
                                } else {
                                    coverages[index % coverages.len()]
                                }
                            })
                            .collect();
                        let colour = colours[layer % colours.len()];
                        S::source_over_span(colour, &coverage, &mut scalar);
                        if layer % 2 == 0 {
                            T::source_over_full_span(colour, &mut tier);
                        } else {
                            T::source_over_span(colour, &coverage, &mut tier);
                        }
                        for (&scalar, &tier) in scalar.iter().zip(&tier) {
                            assert_eq!(bits(S::to_premul(scalar)), bits(T::to_premul(tier)));
                        }
                        // Incomplete output pixels remain untouched in both routes.
                        let mut scalar_bytes = vec![0xa5; len * 8 + 7];
                        let mut tier_bytes = scalar_bytes.clone();
                        S::write_row(&scalar, &mut scalar_bytes);
                        T::write_row(&tier, &mut tier_bytes);
                        assert_eq!(scalar_bytes, tier_bytes);
                    }
                }
            }
        }

        check::<CertifiedScalar, CertifiedBuildTier>();
        check::<FastScalar, FastBuildTier>();
    }

    #[test]
    fn a_mismatched_destination_is_refused_rather_than_reinterpreted() {
        let (stage, _) = corpus();
        let cfg = config();
        let (plan, mono, binning) = derive(&stage, cfg, default_tiling());
        let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");

        let mut wrong_format = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, cfg.viewport.width, cfg.viewport.height)
                .expect("layout"),
        );
        assert!(matches!(
            job.render_into(1, &mut wrong_format),
            Err(FrameError::FormatMismatch { .. })
        ));

        let mut wrong_size =
            FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba16F, 8, 8).expect("layout"));
        assert!(matches!(
            job.render_into(1, &mut wrong_size),
            Err(FrameError::DimensionMismatch)
        ));
    }

    #[test]
    fn stale_derived_artifacts_are_refused_before_rasterization() {
        let (mut stage, mobs) = corpus();
        let cfg = config();
        let (mut plan, mono, binning) = derive(&stage, cfg, default_tiling());

        // Mono pieces contain both scale and origin, so either part of the map
        // moving makes the table stale even when geometry is unchanged.
        let moved_map = ScreenMap {
            origin: [cfg.map.origin[0] + 1.0, cfg.map.origin[1]],
            ..cfg.map
        };
        let moved_mono = MonoTable::build(&plan, moved_map).expect("bounded moved monotone table");
        assert!(matches!(
            FrameJob::new(&plan, &moved_mono, &binning, cfg),
            Err(FrameJobError::MonoMapMismatch)
        ));

        // A different geometry table can have the same number of shapes and
        // still make every shape index name someone else's monotone pieces.
        let mut other_stage = Stage::new();
        let other = other_stage.add(vmob(
            &rect_points(8.0, 8.0, 80.0, 80.0),
            [1.0, 0.0, 0.0, 1.0],
            [0.0; 4],
            0.0,
        ));
        other_stage.add_to_scene(other).expect("live");
        let (other_plan, _, other_binning) = derive(&other_stage, cfg, default_tiling());
        assert!(matches!(
            FrameJob::new(&other_plan, &mono, &other_binning, cfg),
            Err(FrameJobError::MonoPlanMismatch)
        ));

        // Binning consumes the map independently for AABBs and interior flags.
        let moved_binning = Binning::build(&plan, cfg.viewport, default_tiling(), moved_map)
            .expect("bounded test binning");
        assert!(matches!(
            FrameJob::new(&plan, &mono, &moved_binning, cfg),
            Err(FrameJobError::BinningMapMismatch)
        ));

        // Grid cardinality is not viewport identity: both widths occupy seven
        // 16-pixel columns, but the final tile clips at a different pixel.
        let near_viewport = Viewport {
            width: cfg.viewport.width - 1,
            height: cfg.viewport.height,
        };
        let near_binning = Binning::build(&plan, near_viewport, default_tiling(), cfg.map)
            .expect("bounded test binning");
        assert_eq!(near_binning.tile_count(), binning.tile_count());
        assert!(matches!(
            FrameJob::new(&plan, &mono, &near_binning, cfg),
            Err(FrameJobError::BinningViewportMismatch)
        ));

        // Re-syncing can reorder the painter list while every table length and
        // grid dimension remains unchanged. The old command indices must not be
        // reinterpreted under the new order.
        stage.set_z_index(mobs[0], 100, false);
        stage.add_to_scene(mobs[0]).expect("live");
        plan.sync(&stage, 0).expect("valid engine fixture");
        assert!(matches!(
            FrameJob::new(&plan, &mono, &binning, cfg),
            Err(FrameJobError::BinningPlanMismatch)
        ));
    }

    #[test]
    fn a_foreign_style_view_write_changes_the_frame_without_a_revision_callback() {
        let (mut stage, mobs) = corpus();
        let cfg = config();
        let render = |stage: &Stage| {
            let (plan, mono, binning) = derive(stage, cfg, default_tiling());
            FrameJob::new(&plan, &mono, &binning, cfg)
                .expect("matching frame artifacts")
                .render(1)
                .expect("render")
        };
        let before = render(&stage);

        let revision = stage
            .get(mobs[0])
            .expect("live")
            .buffer
            .field_revision("fill_rgba");
        let view = stage
            .get_mut(mobs[0])
            .expect("live")
            .buffer
            .export_field_view("fill_rgba", true)
            .expect("fill field");
        for record in 0..view.len() {
            assert!(view.write_foreign(record, "fill_rgba", &[0.0, 0.0, 1.0, 0.7]));
        }
        assert_eq!(
            stage
                .get(mobs[0])
                .expect("live")
                .buffer
                .field_revision("fill_rgba"),
            revision,
            "the foreign writer must not accidentally exercise revision invalidation"
        );
        let after = render(&stage);
        assert_ne!(
            before.as_bytes(),
            after.as_bytes(),
            "an untracked live style edit must reach the current frame"
        );
        drop(view);
    }

    #[test]
    fn a_writable_point_view_forces_the_general_fill_kernel() {
        let mut stage = Stage::new();
        let mob = stage.add(vmob(
            &rect_points(20.0, 20.0, 60.0, 60.0),
            [1.0, 0.0, 0.0, 1.0],
            [0.0; 4],
            0.0,
        ));
        stage.set_shape(
            mob,
            ShapeTag::Rect {
                center: [40.0, 40.0, 0.0],
                width: 40.0,
                height: 40.0,
            },
        );
        stage.add_to_scene(mob).expect("live");
        let cfg = config();

        let (mut plan, mono, binning) = derive(&stage, cfg, default_tiling());
        {
            let job = FrameJob::new(&plan, &mono, &binning, cfg).expect("matching frame artifacts");
            assert!(matches!(
                job.draws()[0].as_ref().expect("visible").kernel,
                FillKernel::Rect { .. }
            ));
        }

        let view = stage
            .get_mut(mob)
            .expect("live")
            .buffer
            .export_field_view("point", true)
            .expect("point field");
        plan.sync(&stage, 0).expect("valid engine fixture");
        assert!(plan.shapes().instances()[0].hint_unsafe);
        assert!(matches!(
            FrameJob::new(&plan, &mono, &binning, cfg),
            Err(FrameJobError::BinningPlanMismatch)
        ));
        let viewed_binning = Binning::build(&plan, cfg.viewport, default_tiling(), cfg.map)
            .expect("bounded test binning");
        let viewed =
            FrameJob::new(&plan, &mono, &viewed_binning, cfg).expect("matching viewed artifacts");
        assert_eq!(
            viewed.draws()[0].as_ref().expect("visible").kernel,
            FillKernel::General
        );
        drop(view);
    }

    /// Assert two frames are byte-identical, reporting the **first differing
    /// pixel** rather than dumping the buffers.
    ///
    /// A raw `assert_eq!` on two hundred-kilobyte byte slices produces output
    /// nobody can read and therefore nobody debugs from; a coordinate and two
    /// decoded pixels say immediately whether the difference is one rounding or
    /// a structural fault.
    fn assert_frames_equal(a: &FrameBuffer, b: &FrameBuffer, what: &str) {
        assert_eq!(a.layout(), b.layout(), "{what}: different layouts");
        let width = a.layout().width();
        let height = a.layout().height();
        let stride = a.layout().stride(0);
        let mut differing = 0usize;
        let mut first = None;
        for y in 0..height {
            for x in 0..width {
                let base = y as usize * stride + x as usize * 8;
                if !a.plane(0)[base..base + 8].eq(&b.plane(0)[base..base + 8]) {
                    differing += 1;
                    first.get_or_insert((x, y));
                }
            }
        }
        if let Some((x, y)) = first {
            // Asserted on the bytes rather than on the decoded pixels, so the
            // check stays exact while the message stays legible: two f16 codes
            // that decode to the same `f64` are still two different frames.
            let base = y as usize * stride + x as usize * 8;
            assert_eq!(
                a.plane(0)[base..base + 8],
                b.plane(0)[base..base + 8],
                "{what}: {differing} of {} pixels differ; first at ({x}, {y}): {:?} vs {:?}",
                width as usize * height as usize,
                read_px(a, x, y),
                read_px(b, x, y),
            );
        }
    }

    /// Maximum and RMS linear-channel error over two equally laid-out frames.
    fn visual_error(a: &FrameBuffer, b: &FrameBuffer) -> (f64, f64) {
        assert_eq!(a.layout(), b.layout(), "visual comparison layouts differ");
        let mut squared = 0.0;
        let mut maximum = 0.0f64;
        let mut channels = 0u64;
        for y in 0..a.layout().height() {
            for x in 0..a.layout().width() {
                for (a, b) in read_px(a, x, y).into_iter().zip(read_px(b, x, y)) {
                    let error = (a - b).abs();
                    squared += error * error;
                    maximum = maximum.max(error);
                    channels += 1;
                }
            }
        }
        (maximum, (squared / channels as f64).sqrt())
    }

    /// Decode one pixel back to linear-light straight alpha.
    fn read_px(frame: &FrameBuffer, x: u32, y: u32) -> [f64; 4] {
        let stride = frame.layout().stride(0);
        let base = y as usize * stride + x as usize * 8;
        let px = &frame.plane(0)[base..base + 8];
        let mut out = [0.0; 4];
        for (k, o) in out.iter_mut().enumerate() {
            *o = fmn_frame::half::f16_to_f64(u16::from_le_bytes([px[k * 2], px[k * 2 + 1]]));
        }
        out
    }
}
