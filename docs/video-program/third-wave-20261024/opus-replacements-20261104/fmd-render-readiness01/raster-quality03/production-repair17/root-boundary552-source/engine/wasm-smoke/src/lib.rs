//! W5 wasm tier 1 (fm-l97): the headless smoke probe for the wasm32
//! foundation. Compiled to a cdylib `.wasm` and instantiated directly by
//! `run.mjs` under node/bun — no wasm-bindgen CLI pass, so the probe proves
//! the *Rust* foundation executes in a real wasm VM rather than proving glue
//! generation.
//!
//! What the probes cover, end to end in the VM:
//!
//! - `render_probe_digest` / `render_probe_repeat_is_byte_identical` — the
//!   certified CPU render path (Stage → RenderPlan → MonoTable → Binning →
//!   FrameJob → `encode_frame` → digest) executing on wasm32, single-threaded
//!   by construction ([`fmn_render::effective_threads`] collapses any fan-out
//!   request), and byte-identical across two renders of the same primitive
//!   scene. This is the in-VM half of the bead's determinism contract; the
//!   host-side proxy lives in `crates/fmn-scene/tests/runtime.rs`.
//! - `clock_probe_monotonic_ms` / `clock_probe_wall_ms` —
//!   [`fmn_platform::clock::WasmClock`] reading `performance.now()` /
//!   `Date.now()` through its extern imports (the harness binds the real JS
//!   functions).
//! - `process_probe_capability_absent` —
//!   [`fmn_platform::process::NoProcessRunner`] failing closed with the named
//!   [`fmn_platform::process::ProcessError::CapabilityAbsent`] error: the
//!   ffmpeg boundary is structurally absent on wasm32.
//! - `topology_probe_single_threaded` —
//!   [`fmn_platform::topology::HardwareTopology::current`] reporting exactly
//!   one logical CPU (no atomics / cross-origin isolation is the documented
//!   tier-2 question, not this tier).
//!
//! The whole crate is wasm32-only: on any other target it is an empty
//! library, because every probe exists to exercise wasm-specific capability
//! implementations.
#![forbid(unsafe_code)]
#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use wasm_bindgen::prelude::wasm_bindgen;

use fmn_core::color::Srgb;
use fmn_mobject::{Mobject, Stage};
use fmn_platform::clock::{Clock, WasmClock};
use fmn_platform::process::{NoProcessRunner, ProcessError, ProcessRunner, ProcessSpec};
use fmn_platform::topology::HardwareTopology;
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob, encode_frame, frame_digest};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;

/// The probe scene is deliberately tiny: the smoke test measures execution
/// and determinism, not coverage (the golden corpora own coverage).
const WIDTH: u32 = 96;
const HEIGHT: u32 = 54;
const TILING: Tiling = Tiling {
    macro_tile: 64,
    fine_tile: 8,
};

fn frame_config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        // The raw renderer map, like fmn-render's own fixtures: this probe
        // checks wasm determinism, not the front doors' +Y-up orientation.
        ScreenMap {
            scale: 20.0,
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: false,
        },
        Srgb::from_rgb8(0x22, 0x22, 0x22).to_linear(1.0),
    )
}

/// One small primitive scene: a filled square, shifted off-center so the
/// rasterizer's coverage path is genuinely exercised. The point run is the
/// quad-path anchor/handle interleave a library builder would emit
/// (`set_points_as_corners` over the closed corner ring), written over the
/// VMobject record schema with an opaque fill — a bare
/// `Mobject::from_points` carries neither path structure nor style and
/// would rasterize to background, making the probe vacuous.
fn probe_stage() -> Stage {
    let points: [[f32; 3]; 9] = [
        [-1.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, -1.0, 0.0],
    ];
    let mut buffer =
        fmn_mobject::RecordBuffer::new(fmn_mobject::RecordSchema::vmobject(), points.len())
            .expect("nine probe records cannot overflow the buffer size");
    for (i, point) in points.iter().enumerate() {
        buffer.write(i, "point", point);
        buffer.write(i, "fill_rgba", &[1.0, 0.0, 0.0, 1.0]);
        buffer.write(i, "stroke_rgba", &[1.0, 1.0, 1.0, 1.0]);
        buffer.write(i, "stroke_width", &[4.0]);
    }
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(buffer));
    stage.add_to_scene(mob).expect("live root");
    stage.shift(mob, [0.5, 0.25, 0.0]);
    stage
}

/// Render the probe scene single-threaded — the exact call shape the wasm
/// tier-1 surface uses — and return the canonical encoded frame bytes.
fn render_probe_frame() -> Vec<u8> {
    let stage = probe_stage();
    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&stage, 0).expect("valid wasm smoke fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded wasm smoke monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded wasm smoke binning");
    binning.prune_occluded(&plan).expect("binning prune");
    let job = FrameJob::new(&plan, &mono, &binning, config).expect("frame job");
    // threads = 1: the wasm32 configuration. (`effective_threads` would
    // collapse a larger request to the same serial path on this target.)
    let frame = job.render(1).expect("render");
    encode_frame(&frame).expect("encode")
}

/// The frame digest of the probe scene, truncated to 64 bits for the JS
/// boundary (the full digest is compared in-process by
/// [`render_probe_repeat_is_byte_identical`]).
#[wasm_bindgen]
pub fn render_probe_digest() -> u64 {
    let stage = probe_stage();
    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(&stage, 0).expect("valid wasm smoke fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded wasm smoke monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded wasm smoke binning");
    binning.prune_occluded(&plan).expect("binning prune");
    let job = FrameJob::new(&plan, &mono, &binning, config).expect("frame job");
    let frame = job.render(1).expect("render");
    let digest = frame_digest(&frame).expect("digest");
    u64::from_be_bytes(digest.as_bytes()[..8].try_into().expect("digest prefix"))
}

/// The determinism contract, proven inside the wasm VM: two renders of the
/// same scene under the single-thread configuration produce byte-identical
/// canonical frames.
#[wasm_bindgen]
pub fn render_probe_repeat_is_byte_identical() -> bool {
    render_probe_frame() == render_probe_frame()
}

/// The vacuity guard: the probe scene must actually draw something. Renders
/// the probe and an empty stage and reports whether the canonical bytes
/// differ — without it, byte-identical repeat renders could be two identical
/// background-only frames and the determinism proof would say nothing.
#[wasm_bindgen]
pub fn render_probe_is_not_background() -> bool {
    let probe = render_probe_frame();
    let background = {
        let stage = Stage::new();
        let config = frame_config();
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .expect("valid empty wasm smoke fixture");
        let mono = MonoTable::build(&plan, config.map).expect("bounded empty monotone table");
        let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
            .expect("bounded wasm smoke binning");
        binning.prune_occluded(&plan).expect("binning prune");
        let job = FrameJob::new(&plan, &mono, &binning, config).expect("frame job");
        let frame = job.render(1).expect("render");
        encode_frame(&frame).expect("encode")
    };
    probe != background
}

// Semantic sanity oracles in the VM (fm-5wq.46). The probe above uses the
// raw y-down map on purpose; these render the semantic witness through the
// front doors' +Y-up map and check what a mirrored frame cannot fake. The
// witness is the geometric half of `fmn::builtins::witness` (this target has
// no typesetting crates), drawn from raw quad-path records like the probe.

const ORACLE_WIDTH: u32 = 160;
const ORACLE_HEIGHT: u32 = 90;
/// Bits of [`semantic_oracle_mask`], in order: orientation.triangle,
/// orientation.f_shape, placement.up_dot, placement.left_dot, colour.fill,
/// colour.background.
const ALL_ORACLES: u32 = 0b11_1111;

fn oracle_config(y_up: bool) -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: ORACLE_WIDTH,
            height: ORACLE_HEIGHT,
        },
        ScreenMap {
            scale: f64::from(ORACLE_HEIGHT) / 8.0,
            origin: [
                f64::from(ORACLE_WIDTH) / 2.0,
                f64::from(ORACLE_HEIGHT) / 2.0,
            ],
            y_up,
        },
        Srgb::from_rgb8(0, 0, 0).to_linear(1.0),
    )
}

/// A filled straight-edged polygon: the closed corner ring as the quad-path
/// anchor/handle interleave, opaque fill, no stroke.
fn add_polygon(stage: &mut Stage, corners: &[[f32; 2]], rgba: [f32; 4]) {
    let mut points = Vec::with_capacity(2 * corners.len() + 1);
    for (i, a) in corners.iter().enumerate() {
        let b = corners[(i + 1) % corners.len()];
        points.push([a[0], a[1], 0.0]);
        points.push([(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, 0.0]);
    }
    points.push([corners[0][0], corners[0][1], 0.0]);
    let mut buffer =
        fmn_mobject::RecordBuffer::new(fmn_mobject::RecordSchema::vmobject(), points.len())
            .expect("a witness polygon cannot overflow the buffer size");
    for (i, point) in points.iter().enumerate() {
        buffer.write(i, "point", point);
        buffer.write(i, "fill_rgba", &rgba);
        buffer.write(i, "stroke_rgba", &rgba);
        buffer.write(i, "stroke_width", &[0.0]);
    }
    let mob = stage.add(Mobject::from_buffer(buffer));
    stage.add_to_scene(mob).expect("live root");
}

fn square(center: [f32; 2], half: f32) -> [[f32; 2]; 4] {
    let [x, y] = center;
    [
        [x - half, y - half],
        [x + half, y - half],
        [x + half, y + half],
        [x - half, y + half],
    ]
}

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

fn witness_stage() -> Stage {
    let mut stage = Stage::new();
    add_polygon(&mut stage, &[[-6.6, 3.6], [-4.2, 3.6], [-6.6, 1.4]], RED);
    add_polygon(
        &mut stage,
        &[
            [3.2, 0.8],
            [3.7, 0.8],
            [3.7, 2.0],
            [5.0, 2.0],
            [5.0, 2.5],
            [3.7, 2.5],
            [3.7, 3.1],
            [5.6, 3.1],
            [5.6, 3.6],
            [3.2, 3.6],
        ],
        WHITE,
    );
    add_polygon(&mut stage, &square([0.0, 3.0], 0.25), GREEN);
    add_polygon(&mut stage, &square([-5.0, 0.0], 0.25), BLUE);
    stage
}

fn f16_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = i32::from((bits >> 10) & 0x1f);
    let mantissa = f32::from(bits & 0x3ff);
    match exponent {
        0 => sign * mantissa * 2f32.powi(-24),
        31 => sign * f32::INFINITY,
        _ => sign * (1.0 + mantissa / 1024.0) * 2f32.powi(exponent - 15),
    }
}

/// The witness's linear RGB pixels, top row first.
fn witness_pixels(y_up: bool) -> Vec<[f32; 3]> {
    let stage = witness_stage();
    let config = oracle_config(y_up);
    let mut plan = RenderPlan::new();
    plan.sync(&stage, 0).expect("valid witness fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded witness monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded witness binning");
    binning.prune_occluded(&plan).expect("binning prune");
    let job = FrameJob::new(&plan, &mono, &binning, config).expect("frame job");
    let frame = job.render(1).expect("render");
    let stride = frame.layout().stride(0);
    let plane = frame.plane(0);
    let (w, h) = (ORACLE_WIDTH as usize, ORACLE_HEIGHT as usize);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let at = y * stride + x * 8;
            pixels.push(std::array::from_fn(|c| {
                f16_to_f32(u16::from_le_bytes([
                    plane[at + 2 * c],
                    plane[at + 2 * c + 1],
                ]))
            }));
        }
    }
    pixels
}

fn is(pixel: [f32; 3], rgba: [f32; 4]) -> bool {
    (0..3).all(|c| (pixel[c] - rgba[c]).abs() <= 0.1)
}

/// `(column, row)` of every interior pixel of one witness colour.
fn coords(pixels: &[[f32; 3]], rgba: [f32; 4]) -> Vec<(usize, usize)> {
    let w = ORACLE_WIDTH as usize;
    pixels
        .iter()
        .enumerate()
        .filter(|(_, p)| is(**p, rgba))
        .map(|(i, _)| (i % w, i / w))
        .collect()
}

/// Mass in the first and last third of the pixels' extent along an axis.
fn thirds(pixels: &[(usize, usize)], vertical: bool) -> (usize, usize) {
    let axis = |&(x, y): &(usize, usize)| if vertical { y } else { x };
    let (Some(lo), Some(hi)) = (pixels.iter().map(axis).min(), pixels.iter().map(axis).max())
    else {
        return (0, 0);
    };
    let span = hi - lo + 1;
    let first = pixels.iter().filter(|p| (axis(p) - lo) * 3 < span).count();
    let last = pixels
        .iter()
        .filter(|p| (axis(p) - lo) * 3 >= 2 * span)
        .count();
    (first, last)
}

fn heavier(a: usize, b: usize) -> bool {
    a > 0 && a >= 2 * b
}

fn centroid(pixels: &[(usize, usize)]) -> Option<(f64, f64)> {
    if pixels.is_empty() {
        return None;
    }
    let n = pixels.len() as f64;
    let (sx, sy) = pixels.iter().fold((0.0, 0.0), |(sx, sy), &(x, y)| {
        (sx + x as f64, sy + y as f64)
    });
    Some((sx / n, sy / n))
}

/// Scene point to pixel, +Y up: the map the front doors use.
fn expected_pixel(x: f64, y: f64) -> (f64, f64) {
    let scale = f64::from(ORACLE_HEIGHT) / 8.0;
    (
        f64::from(ORACLE_WIDTH) / 2.0 + x * scale,
        f64::from(ORACLE_HEIGHT) / 2.0 - y * scale,
    )
}

fn near(centroid: Option<(f64, f64)>, expected: (f64, f64)) -> bool {
    centroid.is_some_and(|(cx, cy)| {
        (cx - expected.0).abs() <= 0.02 * f64::from(ORACLE_WIDTH)
            && (cy - expected.1).abs() <= 0.02 * f64::from(ORACLE_HEIGHT)
    })
}

fn oracle_mask(pixels: &[[f32; 3]]) -> u32 {
    let (w, h) = (ORACLE_WIDTH as usize, ORACLE_HEIGHT as usize);
    let red = coords(pixels, RED);
    let upper_left = red.iter().filter(|&&(x, y)| 2 * x < w && 2 * y < h).count();
    let (red_top, red_bottom) = thirds(&red, true);
    let (red_left, red_right) = thirds(&red, false);
    let white = coords(pixels, WHITE);
    let (f_top, f_bottom) = thirds(&white, true);
    let (f_left, f_right) = thirds(&white, false);
    let up = centroid(&coords(pixels, GREEN));
    let left = centroid(&coords(pixels, BLUE));
    let (fill_x, fill_y) = expected_pixel((-6.6 - 4.2 - 6.6) / 3.0, (3.6 + 3.6 + 1.4) / 3.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let fill = pixels
        .get(fill_y as usize * w + fill_x as usize)
        .is_some_and(|p| is(*p, RED));
    let background = [0, w - 1, (h - 1) * w, h * w - 1]
        .iter()
        .all(|&i| is(pixels[i], [0.0, 0.0, 0.0, 1.0]));
    [
        !red.is_empty()
            && upper_left * 20 >= red.len() * 19
            && heavier(red_top, red_bottom)
            && heavier(red_left, red_right),
        heavier(f_top, f_bottom) && heavier(f_left, f_right),
        near(up, expected_pixel(0.0, 3.0)) && up.is_some_and(|(_, y)| y < h as f64 / 4.0),
        near(left, expected_pixel(-5.0, 0.0)) && left.is_some_and(|(x, _)| x < w as f64 / 4.0),
        fill,
        background,
    ]
    .iter()
    .enumerate()
    .fold(0, |mask, (bit, &pass)| mask | (u32::from(pass) << bit))
}

/// The oracles that hold on the witness rendered through the +Y-up map,
/// one bit each (see [`ALL_ORACLES`]); a right-side-up frame sets all six.
#[wasm_bindgen]
pub fn semantic_oracle_mask() -> u32 {
    oracle_mask(&witness_pixels(true))
}

/// The same oracles on a planted vertical mirror (the y-down map): the
/// orientation, up-dot and fill bits must all clear.
#[wasm_bindgen]
pub fn semantic_oracle_mask_mirrored() -> u32 {
    oracle_mask(&witness_pixels(false))
}

/// The pass mask's expected value, exported so the harness never hardcodes it.
#[wasm_bindgen]
pub fn semantic_oracle_all() -> u32 {
    ALL_ORACLES
}

/// Monotonic milliseconds from the browser clock capability.
#[wasm_bindgen]
pub fn clock_probe_monotonic_ms() -> f64 {
    WasmClock::new().monotonic().as_secs_f64() * 1000.0
}

/// Wall-clock milliseconds since the Unix epoch from the browser clock
/// capability.
#[wasm_bindgen]
pub fn clock_probe_wall_ms() -> f64 {
    let wall = WasmClock::new().wall();
    wall.duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs_f64()
        * 1000.0
}

/// The process capability fails closed on wasm32: every request is the named
/// [`ProcessError::CapabilityAbsent`], never a spawn attempt.
#[wasm_bindgen]
pub fn process_probe_capability_absent() -> bool {
    let spec = ProcessSpec {
        program: "/nonexistent/ffmpeg".into(),
        argv: Vec::new(),
        env: Vec::new(),
        cwd: None,
        stdin: None,
        timeout: Duration::from_secs(1),
        max_output_bytes: 1024,
    };
    matches!(
        NoProcessRunner.run(&spec),
        Err(ProcessError::CapabilityAbsent { .. })
    )
}

/// The planner-visible machine shape on wasm32 is exactly one logical CPU.
#[wasm_bindgen]
pub fn topology_probe_single_threaded() -> bool {
    let topology = HardwareTopology::current();
    topology.logical_cores() == 1 && topology.physical_cores == 1 && !topology.smt_active()
}
// fingerprint probe
