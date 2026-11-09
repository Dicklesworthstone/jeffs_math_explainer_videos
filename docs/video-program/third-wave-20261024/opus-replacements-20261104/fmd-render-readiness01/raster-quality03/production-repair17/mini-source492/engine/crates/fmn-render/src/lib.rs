//! Lumen: the compiled render IR and engines — analytic coverage, strokes,
//! lighting, adaptive AA (§10).
//!
//! ## What is here today
//!
//! §10.8's **retained core** (fm-gw7): between Marionette's authoritative state
//! and the execution engines sits a compiled, backend-neutral render IR,
//! synchronized *lazily* under §8.2's mirror rule, with backend layouts derived
//! from it rather than baked into it.
//!
//! | module | what it owns |
//! |---|---|
//! | [`revision`] | the seven independent revision axes and the staleness rule |
//! | [`engine`] | §10.1's certified CPU engine: the frame, the bands, the bits |
//! | [`fill`] | §10.2's analytic nonzero-winding coverage, on the curves |
//! | [`stroke`] | §10.3's true curve-distance strokes, round caps, arc-length ramps |
//! | [`hint`] | primitive hints: kernel routing, and the invalidation rule |
//! | [`table`] | the IR's tables: segments, interned styles, interned shapes and their instances |
//! | [`bin`] | two-level binning, per-tile command lists, occlusion pruning |
//! | [`cache`] | the retained compositor's tile cache and its key |
//! | [`plan`] | the retained plan: lazy synchronization from Marionette |
//! | [`snapshot`] | the canonical, bit-lockable form of a compiled plan (§16.5) |
//!
//! [`PixelTileCache`] and [`FrameJob::render_into_cached`] bind the retained
//! compositor to real CPU-engine pixels; callers retain the plan, frame arena,
//! raw frame, and cache across captures.
#![forbid(unsafe_code)]
#![feature(portable_simd)]

mod arena;
pub mod bin;
pub mod cache;
pub mod camera;
pub mod engine;
pub mod fill;
pub mod fill_profile;
pub mod hint;
#[cfg(feature = "metal")]
pub mod metal;
pub mod plan;
pub mod retained;
pub mod revision;
pub mod snapshot;
pub mod stroke;
pub use fill_profile::{FillKnot, FillProfile, FillProfileError};
pub mod stroke_profile;
pub use stroke_profile::{StrokeKnot, StrokeProfile, StrokeProfileError};
pub mod table;
pub mod texture;
pub mod three_d;
pub mod vector_frame;
pub use vector_frame::{OwnedVectorFrame, VectorFrameCompiler};

pub use arena::{AllocStats, FrameArena};
pub use bin::{Binning, BinningError, BinningLimits, ScreenMap, Tiling, Viewport};
pub use cache::{
    CacheStats, OutputTransform, PixelTileCache, PixelTileCacheError, TileCache, TileKey, TileWork,
};
pub use camera::{
    CAMERA_FRAME_Z_INDEX, Camera, CameraConfig, CameraError, CameraFrame, ClipPoint,
    ClippedQuadratic, DEFAULT_FOVY, DEFAULT_LIGHT_POSITION, EdgeSampleLimit,
    THREE_D_CAMERA_SAMPLES, ThreeDCamera,
};
pub use engine::{
    AA_COMPLEX_2X_CROSSINGS, AA_COMPLEX_4X_CROSSINGS, AA_STROKE_COMPLEX_MIN_WIDTH_BANDS,
    AA_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR, AA_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR, AaPolicy,
    AaStats, CachedRenderError, CachedRenderStats, CoverageClass, EngineIdentity, EngineKind,
    FAST_VISUAL_BUDGET_V1_MAX_CHANNEL_ERROR, FAST_VISUAL_BUDGET_V1_MIN_SSIM,
    FAST_VISUAL_BUDGET_V1_RMS_CHANNEL_ERROR, FrameConfig, FrameJob, FrameJobError,
    RENDERER_VERSION, Tier, frame_digest,
};
pub use fill::{
    FillKernel, FlattenReport, GradientField, MonoPiece, MonoTable, MonoTableError,
    MonoTableLimits, RationalPiece, RowScratch,
};
pub use hint::Hint;
pub use plan::{
    CompactionReason, CompactionStats, RenderPlan, RenderPlanEpoch, RenderPlanLimits, SyncError,
    SyncStats,
};
pub use retained::{
    PreparedCameraFrame, RetainedFrameRenderer, RetainedFrameRendererConfig,
    RetainedFrameRendererError,
};
pub use revision::{Axis, Dependency, Revisions};
pub use stroke::{JoinWedge, MITER_LIMIT};
pub use table::{
    Image, ImageDescriptor, ImageTable, Instance, Segment, Shape, ShapeTable, Style, StyleTable,
    TableError,
};
pub use texture::{
    SamplerPolicy, TEXTURE_ORIENTATION, Texture, TextureEncoding, TextureError, TextureLimits,
    TextureOrientation, TextureSource, TextureWrap,
};
pub use three_d::{
    DARK_SHIFT, GLOW_DOT_FACTOR, SURFACE_SHADING, SurfaceDraw, SurfaceMaterial, SurfaceMesh,
    SurfaceVertex, TRUE_DOT_AA_WIDTH, TextureMaterial, ThreeDDraw, ThreeDError, ThreeDJob,
    ThreeDPreparationLimits, TrueDotDraw, VectorDraw, finalize_color, smoothstep, true_dot_alpha,
    unit_normal,
};

/// Thread fan-out clamp (W5 wasm tier 1, fm-l97).
///
/// The engines take a `threads` scheduling hint and run their serial band
/// loop when it is 1. On `wasm32-unknown-unknown` no thread can be spawned
/// at all — `std::thread::scope`'s spawn panics — so any request collapses
/// to the serial path: same bytes (fan-out is documented as a scheduling
/// choice with no effect on output), never a panic. Native builds honor the
/// request unchanged. `HardwareTopology::current` already reports a single
/// logical CPU on wasm32, so a topology-derived request arrives as 1; this
/// clamp is the explicit guarantee for direct callers.
#[inline]
#[must_use]
pub const fn effective_threads(requested: usize) -> usize {
    if cfg!(target_arch = "wasm32") {
        1
    } else {
        requested
    }
}
