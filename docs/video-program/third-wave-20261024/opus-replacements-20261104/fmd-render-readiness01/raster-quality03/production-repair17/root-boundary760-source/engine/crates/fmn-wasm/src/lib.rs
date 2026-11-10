#![forbid(unsafe_code)]
//! The W5 tier-1 wasm32 surface (fm-l97, §10.7): the frame renderer compiled
//! to `wasm32-unknown-unknown`, single-threaded, rendering scenes to RGBA8
//! pixels a browser drops into `ImageData` on a `<canvas>`.
//!
//! # What this is (and is not)
//!
//! This crate is a *surface*, not a DSL: [`FmnScene`] constructs one of a
//! small, fixed set of primitive-corpus scenes (circle shift, parametric
//! Lissajous wave, and a duet of the two) and renders captured
//! [`FramePacket`]s through the same Lumen path the conformance corpus uses
//! (`materialize_stage` → `RenderPlan` → `MonoTable` → `Binning` →
//! `FrameJob`), then converts the certified `Rgba16F` frame through
//! `fmn-frame`'s certified transfer kernel (`rgba16f_to_rgba8`). There is no
//! process capability at this boundary — the ffmpeg sink is structurally
//! absent, and canvas pixels are the only output.
//!
//! # Determinism
//!
//! Wasm is **not** in the certified matrix. Standard-mode determinism holds:
//! the same scene kind, dimensions, and seed produce byte-identical RGBA8
//! output on the same build, because every transcendental on the path comes
//! from `fmn-dmath` (ADR-0014's sovereign funnel — `std` trig is never
//! consulted). Each render instance executes single-threaded (`threads = 1`).
//! The separately built `fmn-wasm/threads` npm subpath runs independent frame
//! renders concurrently in module workers over one instantiated shared Wasm
//! memory; it does not change this per-frame semantic renderer.
//!
//! # JS usage
//!
//! ```text
//! import init, { FmnScene } from "./pkg/fmn_wasm.js";
//! await init();
//! const scene = new FmnScene("orbit_duet", 640, 360);
//! const pixels = scene.render_frame(0);        // Uint8Array, w*h*4
//! const image = new ImageData(new Uint8ClampedArray(pixels), scene.width, scene.height);
//! ctx.putImageData(image, 0, 0);
//! // caller-storage variant: reuse one JS buffer across frames
//! const scratch = new Uint8Array(scene.width * scene.height * 4);
//! scene.render_into(1, scratch);
//! ```
//!
//! The demo lives in `demo/wasm/` (build commands in `demo/wasm/README.md`).

use fmn_anim::{FramePacket, prepare_animation};
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, DEFAULT_STROKE_WIDTH, TAU, TEAL_B, WHITE};
use fmn_core::types::Vec3;
use fmn_frame::convert::rgba16f_to_rgba8_slice;
use fmn_frame::{FrameLayout, PixelFormat};
use fmn_geom::bezier::{arc_n_components, quadratic_points_for_arc};
use fmn_geom::quadpath::QuadPath;
use fmn_mobject::record::RecordBuffer;
use fmn_mobject::record::RecordSchema;
use fmn_mobject::shape::ShapeTag;
use fmn_mobject::{Mob, Mobject, Stage};
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use fmn_scene::{CaptureReason, PlayOverrides, RuntimeConfig, Scene, SceneSink};
use wasm_bindgen::JsError;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod player;

/// The tier-2 demo bundle builder — host-side tooling only; the wasm32
/// artifact never carries the exporter.
#[cfg(not(target_arch = "wasm32"))]
pub mod demo_bundle;

/// Fixed scene seed: construction takes no entropy, so the seed is a
/// named constant rather than an input (see the determinism note above).
const SCENE_SEED: u64 = 0;
/// Demo frame rate; the JS side scrubs frames, it does not clock them.
const FPS: u32 = 30;
/// Every scene plays one one-second animation, then holds a half-second
/// wait so the terminal state is scrubbable too.
const RUN_TIME_SECONDS: f64 = 1.0;
const WAIT_SECONDS: f64 = 0.5;
/// One semantic renderer remains single-threaded by construction. The npm
/// threads subpath parallelizes independent frames across Wasm instances.
const RENDER_THREADS: usize = 1;
/// The conformance corpus tiling, reused so the wasm path bins exactly like
/// the certified test path.
const TILING: Tiling = Tiling {
    macro_tile: 64,
    fine_tile: 8,
};
/// The Reference's frame is 8 units tall; the map derives from the pixel
/// height so a scene is resolution-independent.
const FRAME_HEIGHT_UNITS: f64 = 8.0;
/// Pixel-dimension ceiling: enough headroom for a 4K canvas, small enough
/// that a hostile or mistaken dimension cannot allocate absurd buffers.
const MAX_DIMENSION: u32 = 4096;

/// The engine/package version compiled into this module.
///
/// Release packaging verifies this value against both the workspace version
/// and the generated npm `package.json`, so a stale JS/WASM artifact cannot
/// masquerade as the current engine.
#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[cfg(test)]
std::thread_local! {
    static OWNED_RGBA8_OUTPUT_ALLOCATIONS: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
    static RASTERIZED_SURFACES: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

fn note_owned_rgba8_output_allocation() {
    #[cfg(test)]
    OWNED_RGBA8_OUTPUT_ALLOCATIONS.with(|count| count.set(count.get() + 1));
}

fn note_rasterized_surface() {
    #[cfg(test)]
    RASTERIZED_SURFACES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
fn owned_rgba8_output_allocations() -> usize {
    OWNED_RGBA8_OUTPUT_ALLOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
fn rasterized_surface_count() -> usize {
    RASTERIZED_SURFACES.with(std::cell::Cell::get)
}

/// The scene kinds [`FmnScene::new`] accepts. This is the whole "scene
/// description language" — deliberately a closed set of landed primitive
/// scenes, not a general DSL.
const SCENE_KINDS: [&str; 3] = ["circle_shift", "parametric_wave", "orbit_duet"];

/// Any failure the surface can produce, as a message: the JS boundary
/// stringifies errors anyway, so a structured error taxonomy would be
/// decoration here.
#[derive(Debug)]
struct SurfaceError(String);

impl SurfaceError {
    fn new(context: &str, error: impl std::fmt::Display) -> Self {
        Self(format!("{context}: {error}"))
    }
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SurfaceError {}

/// A `SceneSink` that retains the immutable capture stream: the wasm
/// surface renders packets lazily, so construction stores front-end state,
/// not pixels.
struct PacketSink {
    packets: Vec<FramePacket>,
}

impl SceneSink for PacketSink {
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), fmn_scene::IntegrationError> {
        // Segment captures are the play/wait frame stream; lifecycle
        // previews (show/presenter) carry no new picture here.
        if matches!(reason, CaptureReason::Segment) {
            self.packets.push(packet);
        }
        Ok(())
    }
}

/// The host-testable core: everything [`FmnScene`] exposes, with no bindgen
/// types in the way.
struct SceneBuild {
    packets: Vec<FramePacket>,
    width: u32,
    height: u32,
}

/// The style a built vmobject carries, mirroring `fmn-library`'s
/// `Style::write` convention field for field (the library tier itself is
/// deliberately not a dependency of this leaf crate).
#[derive(Clone, Copy)]
struct StyleSpec {
    fill: Srgb,
    fill_opacity: f64,
    stroke: Srgb,
    stroke_opacity: f64,
    stroke_width: f64,
}

impl StyleSpec {
    /// A filled, stroked shape (the circle).
    fn filled(fill: Srgb, fill_opacity: f64) -> Self {
        Self {
            fill,
            fill_opacity,
            stroke: WHITE,
            stroke_opacity: 1.0,
            stroke_width: DEFAULT_STROKE_WIDTH,
        }
    }

    /// An outline with no fill (the parametric curve).
    fn outline(stroke: Srgb) -> Self {
        Self {
            fill: stroke,
            fill_opacity: 0.0,
            stroke,
            stroke_opacity: 1.0,
            stroke_width: DEFAULT_STROKE_WIDTH,
        }
    }
}

/// Build a vector mobject over the `vmobject` record schema, replicating
/// the library's `From<VMobject> for Mobject` conversion: flat points,
/// joint angles from the Chisel path, then style columns.
fn build_vmobject(points: &[Vec3], shape: ShapeTag, style: StyleSpec) -> Mobject {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len())
        .expect("record sizing bounded by the point list");
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    buffer.write_range("point", 0, &flat);
    if let Ok(path) = QuadPath::from_points(points.to_vec()) {
        let angles: Vec<f32> = path.joint_angles().iter().map(|a| *a as f32).collect();
        buffer.write_range("joint_angle", 0, &angles);
    }
    let n = buffer.len();
    let stroke_rgba: Vec<f32> = (0..n)
        .flat_map(|_| {
            [
                style.stroke.r as f32,
                style.stroke.g as f32,
                style.stroke.b as f32,
                style.stroke_opacity as f32,
            ]
        })
        .collect();
    buffer.write_range("stroke_rgba", 0, &stroke_rgba);
    let fill_rgba: Vec<f32> = (0..n)
        .flat_map(|_| {
            [
                style.fill.r as f32,
                style.fill.g as f32,
                style.fill.b as f32,
                style.fill_opacity as f32,
            ]
        })
        .collect();
    buffer.write_range("fill_rgba", 0, &fill_rgba);
    let widths = vec![style.stroke_width as f32; n];
    buffer.write_range("stroke_width", 0, &widths);
    Mobject::from_buffer(buffer).with_shape(shape)
}

/// The corpus circle: a full unit-circle quadratic arc run scaled to
/// `radius` about `center`, tagged so the primitive kernel can claim it.
pub(crate) fn circle_mobject(center: Vec3, radius: f64) -> Mobject {
    let unit = quadratic_points_for_arc(TAU, arc_n_components(TAU).expect("TAU is a finite angle"))
        .expect("TAU fits the arc budget");
    let points: Vec<Vec3> = unit
        .iter()
        .map(|p| {
            [
                center[0] + radius * p[0],
                center[1] + radius * p[1],
                center[2],
            ]
        })
        .collect();
    build_vmobject(
        &points,
        ShapeTag::Circle { center, radius },
        StyleSpec::filled(BLUE_C, 0.35),
    )
}

/// The parametric curve: a Lissajous figure sampled as corners and
/// converted to the anchor/handle run by the Chisel path. Every sample
/// goes through `fmn_dmath` — the determinism note is only true because no
/// platform trig ever runs here.
pub(crate) fn parametric_mobject() -> Mobject {
    const SAMPLES: usize = 161;
    let corners: Vec<Vec3> = (0..SAMPLES)
        .map(|i| {
            let t = TAU * (i as f64) / ((SAMPLES - 1) as f64);
            [
                2.4 * fmn_dmath::sin(3.0 * t),
                1.5 * fmn_dmath::sin(2.0 * t),
                0.0,
            ]
        })
        .collect();
    let mut path = QuadPath::new();
    // 161 corners describe a closed figure (the last corner equals the
    // first); corner conversion can only fail on degenerate input, which
    // this fixed sampling cannot produce — but a failure must not silently
    // render nothing, so fall back to the raw corners as anchors.
    if path.set_points_as_corners(&corners).is_err() {
        return build_vmobject(&corners, ShapeTag::General, StyleSpec::outline(TEAL_B));
    }
    build_vmobject(path.points(), ShapeTag::General, StyleSpec::outline(TEAL_B))
}

fn animate_args() -> fmn_mobject::animate::AnimateArgs {
    fmn_mobject::animate::AnimateArgs {
        run_time: Some(RUN_TIME_SECONDS),
        rate_func: Some(fmn_core::rate::smooth),
        ..fmn_mobject::animate::AnimateArgs::default()
    }
}

/// Construct the scene, run it headless, and retain the capture stream.
fn build_scene(kind: &str, width: u32, height: u32) -> Result<SceneBuild, SurfaceError> {
    if !SCENE_KINDS.contains(&kind) {
        return Err(SurfaceError(format!(
            "unknown scene kind {kind:?}; expected one of: {}",
            SCENE_KINDS.join(", ")
        )));
    }
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(SurfaceError(format!(
            "invalid dimensions {width}x{height}: each must be in 1..={MAX_DIMENSION}"
        )));
    }

    let config = RuntimeConfig {
        fps: FPS,
        ..RuntimeConfig::default()
    };
    let mut scene =
        Scene::new(config, SCENE_SEED).map_err(|e| SurfaceError::new("scene init", e))?;

    let circle = scene
        .add_mobject(circle_mobject([-1.6, 0.0, 0.0], 1.0))
        .map_err(|e| SurfaceError::new("add circle", e))?;
    let mut animations = Vec::new();
    let mut circle_anchor: Option<Mob> = None;

    match kind {
        "circle_shift" => {
            circle_anchor = Some(circle);
        }
        "parametric_wave" => {
            // The wave scene hides the circle's paint but keeps it offstage:
            // a single mobject list keeps the three constructions uniform.
            shift_offstage(scene.stage_mut(), circle);
            let wave = scene
                .add_mobject(parametric_mobject())
                .map_err(|e| SurfaceError::new("add wave", e))?;
            let animation = wave
                .animate()
                .set_anim_args(animate_args())
                .and_then(|b| b.scale(1.25))
                .map_err(|e| SurfaceError::new("wave animation", e))?;
            animations.push(
                prepare_animation(animation, scene.stage_mut())
                    .map_err(|e| SurfaceError::new("prepare wave", e))?,
            );
        }
        "orbit_duet" => {
            circle_anchor = Some(circle);
            let wave = scene
                .add_mobject(parametric_mobject())
                .map_err(|e| SurfaceError::new("add wave", e))?;
            let animation = wave
                .animate()
                .set_anim_args(animate_args())
                .and_then(|b| b.scale(1.25))
                .map_err(|e| SurfaceError::new("wave animation", e))?;
            animations.push(
                prepare_animation(animation, scene.stage_mut())
                    .map_err(|e| SurfaceError::new("prepare wave", e))?,
            );
        }
        _ => {
            // SCENE_KINDS.contains above makes this unreachable by
            // construction; surface it as an error rather than a panic.
            return Err(SurfaceError(format!("scene kind {kind:?} not implemented")));
        }
    }

    if let Some(mob) = circle_anchor {
        let animation = mob
            .animate()
            .set_anim_args(animate_args())
            .and_then(|b| b.shift([3.2, 0.0, 0.0]))
            .map_err(|e| SurfaceError::new("circle animation", e))?;
        animations.push(
            prepare_animation(animation, scene.stage_mut())
                .map_err(|e| SurfaceError::new("prepare circle", e))?,
        );
    }

    let mut sink = PacketSink {
        packets: Vec::new(),
    };
    scene
        .play(animations, PlayOverrides::default(), &mut sink)
        .map_err(|e| SurfaceError::new("play", e))?;
    scene
        .wait(Some(WAIT_SECONDS), &mut sink)
        .map_err(|e| SurfaceError::new("wait", e))?;

    if sink.packets.is_empty() {
        return Err(SurfaceError(
            "scene produced no frames; nothing to render".to_string(),
        ));
    }
    Ok(SceneBuild {
        packets: sink.packets,
        width,
        height,
    })
}

/// Park a mobject far outside the frame (used to keep the circle in the
/// stage for scenes that do not show it, so construction order is uniform).
fn shift_offstage(stage: &mut Stage, mob: Mob) {
    stage.shift(mob, [0.0, -1000.0, 0.0]);
}

/// The frame configuration for one render: the Reference's 8-unit frame
/// height mapped onto the pixel viewport over an opaque black background.
fn frame_config(width: u32, height: u32) -> FrameConfig {
    FrameConfig::new(
        Viewport { width, height },
        ScreenMap {
            scale: f64::from(height) / FRAME_HEIGHT_UNITS,
            origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
            y_up: true,
        },
        Srgb::from_rgb8(0x00, 0x00, 0x00).to_linear(1.0),
    )
}

fn rgba8_output_len(width: u32, height: u32) -> Result<usize, SurfaceError> {
    FrameLayout::tight(PixelFormat::Rgba8, width, height)
        .map(|layout| layout.total_bytes())
        .map_err(|e| SurfaceError::new("rgba8 layout", e))
}

fn validate_rgba8_destination(width: u32, height: u32, dst: &[u8]) -> Result<(), SurfaceError> {
    let expected = rgba8_output_len(width, height)?;
    if dst.len() != expected {
        return Err(SurfaceError(format!(
            "render_into destination is {} bytes; expected {expected} ({width}x{height}x4)",
            dst.len()
        )));
    }
    Ok(())
}

/// Render one materialized stage directly into a caller-owned tight sRGB
/// RGBA8 slice, top row first. Destination geometry is validated before any
/// render-plan derivation or raster work.
pub(crate) fn render_stage_rgba8_into(
    stage: &Stage,
    width: u32,
    height: u32,
    camera_revision: u64,
    dst: &mut [u8],
) -> Result<(), SurfaceError> {
    validate_rgba8_destination(width, height, dst)?;
    let config = frame_config(width, height);
    let mut plan = RenderPlan::new();
    plan.sync(stage, camera_revision)
        .map_err(|error| SurfaceError::new("render plan", error))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| SurfaceError::new("monotone table", error))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .map_err(|e| SurfaceError::new("binning", e))?;
    binning
        .prune_occluded(&plan)
        .map_err(|e| SurfaceError::new("binning", e))?;
    let job =
        FrameJob::new(&plan, &mono, &binning, config).map_err(|e| SurfaceError::new("job", e))?;
    note_rasterized_surface();
    let frame = job
        .render(RENDER_THREADS)
        .map_err(|e| SurfaceError::new("render", e))?;
    rgba16f_to_rgba8_slice(&frame, dst).map_err(|e| SurfaceError::new("transfer", e))
}

/// Render one materialized stage to sRGB RGBA8 through the tier-1 Lumen
/// path, top row first (D-23's orientation rule — `ImageData` wants
/// exactly this). Shared by the packet renderer and the tier-2 player
/// (fm-oee): both render a stage, differing only in where the stage came
/// from.
pub(crate) fn render_stage_rgba8(
    stage: &Stage,
    width: u32,
    height: u32,
    camera_revision: u64,
) -> Result<Vec<u8>, SurfaceError> {
    let len = rgba8_output_len(width, height)?;
    let mut rgba8 = Vec::new();
    rgba8
        .try_reserve_exact(len)
        .map_err(|_| SurfaceError(format!("could not allocate {len}-byte RGBA8 output frame")))?;
    rgba8.resize(len, 0);
    note_owned_rgba8_output_allocation();
    render_stage_rgba8_into(stage, width, height, camera_revision, &mut rgba8)?;
    Ok(rgba8)
}

/// Render one captured packet to sRGB RGBA8, top row first (D-23's
/// orientation rule — `ImageData` wants exactly this).
fn render_packet_rgba8(build: &SceneBuild, index: usize) -> Result<Vec<u8>, SurfaceError> {
    let Some(packet) = build.packets.get(index) else {
        return Err(SurfaceError(format!(
            "frame index {index} out of range 0..{}",
            build.packets.len()
        )));
    };
    let stage = packet.materialize_stage();
    let camera_revision = u64::try_from(packet.frame_index())
        .map_err(|_| SurfaceError("negative frame index reached the renderer".to_string()))?;
    render_stage_rgba8(&stage, build.width, build.height, camera_revision)
}

/// Render one captured packet directly into caller storage. Frame range is
/// validated first; destination length is then checked before the packet's
/// snapshot is materialized.
fn render_packet_rgba8_into(
    build: &SceneBuild,
    index: usize,
    dst: &mut [u8],
) -> Result<(), SurfaceError> {
    let Some(packet) = build.packets.get(index) else {
        return Err(SurfaceError(format!(
            "frame index {index} out of range 0..{}",
            build.packets.len()
        )));
    };
    validate_rgba8_destination(build.width, build.height, dst)?;
    let stage = packet.materialize_stage();
    let camera_revision = u64::try_from(packet.frame_index())
        .map_err(|_| SurfaceError("negative frame index reached the renderer".to_string()))?;
    render_stage_rgba8_into(&stage, build.width, build.height, camera_revision, dst)
}

/// A constructed scene, ready to render frames to RGBA8 pixels.
///
/// Construction is eager: the scene runs headless once and retains the
/// immutable [`FramePacket`] stream, so `render_frame` never re-executes
/// animations — it materializes and rasterizes one captured packet.
#[wasm_bindgen]
pub struct FmnScene {
    build: SceneBuild,
}

#[wasm_bindgen]
impl FmnScene {
    /// Construct one of the fixed primitive-corpus scenes. `kind` is one of
    /// `scene_kinds()`; `width`/`height` are canvas pixels.
    ///
    /// # Errors
    /// `JsError` for an unknown kind, invalid dimensions, or any
    /// scene/render pipeline failure during construction.
    #[wasm_bindgen(constructor)]
    pub fn new(kind: &str, width: u32, height: u32) -> Result<FmnScene, JsError> {
        Ok(FmnScene {
            build: build_scene(kind, width, height)?,
        })
    }

    /// The scene kinds the constructor accepts (a closed set, by design).
    #[wasm_bindgen]
    pub fn scene_kinds() -> Vec<String> {
        SCENE_KINDS.iter().map(|s| (*s).to_string()).collect()
    }

    /// Canvas pixel width.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.build.width
    }

    /// Canvas pixel height.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.build.height
    }

    /// Frames captured during construction (`play` + terminal `wait`).
    #[wasm_bindgen(getter)]
    pub fn frame_count(&self) -> u32 {
        u32::try_from(self.build.packets.len()).unwrap_or(u32::MAX)
    }

    /// Render `index` to a fresh `width * height * 4` sRGB RGBA8 buffer
    /// (top row first). In JS this arrives as a `Uint8Array`; wrap it for
    /// `ImageData` via `new Uint8ClampedArray(pixels)`.
    ///
    /// # Errors
    /// `JsError` for an out-of-range index or a render failure.
    pub fn render_frame(&self, index: u32) -> Result<Vec<u8>, JsError> {
        Ok(render_packet_rgba8(&self.build, index as usize)?)
    }

    /// Render `index` into caller-owned JS storage of exactly
    /// `width * height * 4` bytes, so the caller can reuse one destination
    /// rather than receive a fresh `Uint8Array` per frame. wasm-bindgen still
    /// marshals the mutable slice through WebAssembly memory; this is storage
    /// reuse, not a zero-copy JS/WASM transfer.
    ///
    /// # Errors
    /// `JsError` for a wrong-length destination, out-of-range index, or a
    /// render failure.
    pub fn render_into(&self, index: u32, dst: &mut [u8]) -> Result<(), JsError> {
        Ok(render_packet_rgba8_into(&self.build, index as usize, dst)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_engine_version_tracks_the_workspace_package() {
        assert_eq!(engine_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn every_kind_constructs_and_renders_a_nonempty_frame() {
        for kind in SCENE_KINDS {
            let build = build_scene(kind, 96, 54).expect("construct scene");
            assert!(!build.packets.is_empty(), "{kind}: no frames captured");
            let pixels = render_packet_rgba8(&build, 0).expect("render frame 0");
            assert_eq!(pixels.len(), 96 * 54 * 4, "{kind}: wrong buffer length");
            assert!(
                pixels.iter().any(|&b| b != 0),
                "{kind}: frame 0 is uniformly black — the scene did not reach the pixels"
            );
            let last =
                render_packet_rgba8(&build, build.packets.len() - 1).expect("render last frame");
            assert_eq!(last.len(), pixels.len(), "{kind}: last frame length");
            assert!(
                last.iter().any(|&b| b != 0),
                "{kind}: last frame is uniformly black"
            );
        }
    }

    #[test]
    fn the_same_build_renders_byte_identical_frames() {
        // Standard-mode determinism: same seed, same build ⇒ same bytes.
        let a = build_scene("orbit_duet", 96, 54).expect("construct a");
        let b = build_scene("orbit_duet", 96, 54).expect("construct b");
        assert_eq!(a.packets.len(), b.packets.len());
        for index in [0, a.packets.len() / 2, a.packets.len() - 1] {
            let pa = render_packet_rgba8(&a, index).expect("render a");
            let pb = render_packet_rgba8(&b, index).expect("render b");
            assert_eq!(pa, pb, "frame {index} differs across identical builds");
        }
    }

    #[test]
    fn animation_actually_moves_pixels() {
        let build = build_scene("circle_shift", 96, 54).expect("construct");
        let first = render_packet_rgba8(&build, 0).expect("first");
        let last = render_packet_rgba8(&build, build.packets.len() - 1).expect("last");
        assert_ne!(first, last, "the shift animation changed nothing");
    }

    #[test]
    fn bad_inputs_are_errors_not_panics() {
        assert!(build_scene("not_a_scene", 96, 54).is_err());
        assert!(build_scene("circle_shift", 0, 54).is_err());
        assert!(build_scene("circle_shift", 96, 0).is_err());
        assert!(build_scene("circle_shift", 5000, 54).is_err());
        let build = build_scene("circle_shift", 32, 18).expect("construct");
        assert!(render_packet_rgba8(&build, build.packets.len()).is_err());
    }

    #[test]
    fn bindgen_surface_matches_the_core() {
        // Success paths only: `JsError::new` calls the JS `Error`
        // constructor import, which does not exist off-wasm, so error
        // branches are browser-side (see `browser_smoke_manual`).
        let scene = FmnScene::new("parametric_wave", 64, 36).expect("construct");
        assert_eq!(scene.width(), 64);
        assert_eq!(scene.height(), 36);
        assert!(scene.frame_count() > 0);
        let pixels = scene.render_frame(0).expect("render");
        assert_eq!(pixels.len(), 64 * 36 * 4);
        assert!(pixels.iter().any(|&b| b != 0));
        let mut scratch = vec![0u8; 64 * 36 * 4];
        scene.render_into(0, &mut scratch).expect("render_into");
        assert_eq!(scratch, pixels);
        assert!(FmnScene::scene_kinds().contains(&"orbit_duet".to_string()));
    }

    #[test]
    fn tier1_caller_storage_is_identical_and_avoids_owned_rgba8_frames() {
        let build = build_scene("orbit_duet", 64, 36).expect("construct");
        let expected = render_packet_rgba8(&build, 0).expect("allocating render");
        let owned_before = owned_rgba8_output_allocations();
        let mut scratch = vec![0xA5; expected.len()];
        let storage = scratch.as_ptr();
        let capacity = scratch.capacity();

        for index in [0, build.packets.len() / 2, build.packets.len() - 1] {
            let expected = render_packet_rgba8(&build, index).expect("allocating render");
            let owned_after_expected = owned_rgba8_output_allocations();
            render_packet_rgba8_into(&build, index, &mut scratch).expect("caller render");
            assert_eq!(scratch, expected, "frame {index} differs by destination");
            assert_eq!(scratch.as_ptr(), storage, "caller storage moved");
            assert_eq!(scratch.capacity(), capacity, "caller capacity changed");
            assert_eq!(
                owned_rgba8_output_allocations(),
                owned_after_expected,
                "render_into routed through an owned RGBA8 frame"
            );
        }
        assert_eq!(
            owned_rgba8_output_allocations(),
            owned_before + 3,
            "only the three allocating convenience renders should own RGBA8 frames"
        );
    }

    #[test]
    fn tier1_caller_storage_length_refuses_before_rasterization() {
        let build = build_scene("circle_shift", 32, 18).expect("construct");
        let mut short = vec![0; 32 * 18 * 4 - 1];
        let renders_before = rasterized_surface_count();
        let error = render_packet_rgba8_into(&build, 0, &mut short)
            .expect_err("short destination must refuse");
        assert!(error.to_string().contains("expected 2304 (32x18x4)"));
        assert_eq!(
            rasterized_surface_count(),
            renders_before,
            "wrong-length storage must refuse before rasterization"
        );
    }

    /// The R19 artifact-size budget: if the release wasm artifact has been
    /// built on this host, it must fit the recorded budget (measured + 10%
    /// headroom, `SIZE_BUDGET.tsv`). When no artifact exists the test
    /// reports the skip (and fails under `FMN_REQUIRE_FULL_INPUTS=1`) —
    /// building the artifact is a deliberate step, never a test side effect.
    #[test]
    fn size_budget_within_headroom() {
        let budget_text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("SIZE_BUDGET.tsv"),
        )
        .expect("SIZE_BUDGET.tsv reads");
        let budget_for = |artifact: &str| -> u64 {
            budget_text
                .lines()
                .filter(|line| !line.starts_with('#'))
                .find_map(|line| {
                    let mut fields = line.split('\t');
                    (fields.next() == Some(artifact))
                        .then(|| fields.nth(1).and_then(|b| b.parse().ok()))
                        .flatten()
                })
                .expect("budget row exists")
        };

        let target_dir = std::env::var("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target")
            });
        let candidates = [
            (
                "cargo-release-wasm",
                target_dir.join("wasm32-unknown-unknown/release/fmn_wasm.wasm"),
            ),
            (
                "wasm-bindgen-web-pkg",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../demo/wasm/pkg/fmn_wasm_bg.wasm"),
            ),
        ];
        let mut checked = 0;
        for (artifact, path) in &candidates {
            let Ok(metadata) = std::fs::metadata(path) else {
                continue;
            };
            let size = metadata.len();
            let budget = budget_for(artifact);
            assert!(
                size <= budget,
                "{artifact} is {size} bytes, over the {budget}-byte budget — \
                 re-measure deliberately and update SIZE_BUDGET.tsv"
            );
            checked += 1;
        }
        if checked == 0 {
            fmn_core::test_inputs::skip_or_fail(
                "fmn-wasm::size_budget_within_headroom",
                "a built wasm artifact (demo/wasm/README.md)",
            );
        }
    }

    /// Keep the tier-1 half of the executable package/browser harness in the
    /// source closure. `scripts/check_wasm_package.sh` is the behavioral gate:
    /// it installs the packed tarball, bundles this fixture, and requires its
    /// success marker from real headless Chromium.
    #[test]
    fn tier1_package_browser_harness_assets_exist() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for relative in [
            "demo/wasm/smoke.html",
            "demo/wasm/smoke.mjs",
            "demo/wasm/webpack.config.cjs",
            "scripts/check_wasm_package.sh",
            "crates/fmn-wasm/js/threads.js",
            "crates/fmn-wasm/js/threads_worker.js",
            "crates/fmn-wasm/js/threads.d.ts",
        ] {
            assert!(root.join(relative).is_file(), "{relative} is missing");
        }
    }
}
