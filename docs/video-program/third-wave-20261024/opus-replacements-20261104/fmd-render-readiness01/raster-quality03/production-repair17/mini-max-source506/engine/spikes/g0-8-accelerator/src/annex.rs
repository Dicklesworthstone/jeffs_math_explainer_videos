//! The annex engine: the same IR, the same stage, dispatched to Metal through
//! frankentorch.
//!
//! **frankentorch is the only GPU gateway** (D-22, §10.7) — no wgpu, ever — so
//! every device call below goes through `ft_kernel_metal::compute`, the generic
//! Metal compute gateway this spike contributed upstream (UPSTREAM_LEDGER row
//! 8). Nothing in this file touches `metal-rs` or `unsafe`; the crate keeps its
//! `#![forbid(unsafe_code)]`, which was one of the properties in question.
//!
//! The annex is **standard-mode only, never certified** (D-18's permanent
//! refusal of GPU work in the certified path). Nothing here is bit-promised;
//! [`crate::compare`] measures how far it lands from the CPU reference, and
//! that measurement is the deliverable.

use crate::analytic_fill::{
    FILL_PATH_F32_STRIDE, FILL_PATH_U32_STRIDE, FILL_STYLE_STRIDE, FlatFill, MonoTable,
    PIECE_STRIDE, TileClasses, flatten_fill,
};
use crate::ir::{
    DrawKind, FlatIr, PATH_F32_STRIDE, PATH_U32_STRIDE, RenderIr, SEGMENT_STRIDE, STYLE_STRIDE,
};
use ft_kernel_metal::Error;
use ft_kernel_metal::compute::{Gateway, Grid, MathMode};

/// The kernel source, compiled at run time by the gateway.
pub const KERNEL_SOURCE: &str = include_str!("shaders/stroke_aa.metal");

/// The kernel's entry point.
pub const KERNEL_NAME: &str = "stroke_aa_resolve";

/// What the annex reports about a dispatch, for the PG-A record (§17.2) and
/// for the input closure's backend identity (§16.7, §10.5f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnexReport {
    /// The device the frame was rendered on, e.g. `"Apple M4 Pro"`.
    pub device: String,
    /// Whether the device has unified memory — true on all Apple silicon, and
    /// the premise §10.7 leans on when it calls the handoff "nearly free".
    pub unified_memory: bool,
    /// Threads per threadgroup actually dispatched.
    pub threads_per_threadgroup: usize,
    /// The pipeline's occupancy ceiling, read from introspection rather than
    /// assumed ("threadgroup sizes taken from pipeline introspection, never
    /// CUDA habit" — §17.6).
    pub max_threads_per_threadgroup: usize,
    /// The device SIMD width for this pipeline.
    pub thread_execution_width: usize,
    /// Bytes uploaded for this frame.
    pub upload_bytes: usize,
    /// Bytes read back for this frame.
    pub readback_bytes: usize,
    /// Which math mode the kernel was compiled under — part of the backend
    /// identity the input closure journals (§10.5f), because it changes the
    /// numbers.
    pub math_mode: MathMode,
}

/// Render `ir` on Metal, returning the linear-light surface and the report.
///
/// Returns [`Error::Unavailable`] off macOS or on a machine with no Metal
/// device — the caller's answer to which is the CPU engine, exactly as §10.7
/// intends ("the CPU engine must stand on its own so acceleration can never
/// mask a core regression").
pub fn render(ir: &RenderIr) -> Result<(crate::cpu::Surface, AnnexReport), Error> {
    render_with(ir, MathMode::Safe)
}

/// Render `ir` on Metal under an explicit [`MathMode`].
///
/// Exposed because the difference is measurable and load-bearing: Metal's own
/// default is fast math, and the spike's whole point is that the annex's
/// divergence from the CPU reference is a number somebody has to look at.
pub fn render_with(
    ir: &RenderIr,
    math_mode: MathMode,
) -> Result<(crate::cpu::Surface, AnnexReport), Error> {
    // The kernel implements the stroke stage and only the stroke stage. A GPU
    // that silently skipped the fills would produce a plausible, wrong picture
    // — and D-18 refuses GPU work in the certified path permanently, so the
    // frame that defines the certified bits must never arrive here at all.
    // Refusing by name is both the safe behaviour and the doctrinal one.
    if let Some(bad) = ir.paths.iter().find(|p| p.kind != DrawKind::Stroke) {
        return Err(Error::Kernel(format!(
            "the annex renders {:?} only; this IR contains a {:?} path. \
             Fills and glow are CPU-side (fm-orn spikes the fill's GPU mapping), \
             and a certified frame must never reach a GPU at all (D-18)",
            DrawKind::Stroke,
            bad.kind
        )));
    }
    // Same reasoning, one field lower: the shader implements no joint-angle
    // widening, and rendering without it would silently disagree with the CPU
    // reference rather than fail.
    if ir.styles.iter().any(|s| s.miter_gain != 0.0) {
        return Err(Error::Kernel(
            "the annex's kernel implements no miter gain; a style with a \
             nonzero `miter_gain` would render differently on the two engines. \
             The joint-angle stand-in is CPU-side (see Style::miter_gain)"
                .into(),
        ));
    }

    let gw = Gateway::open()?;
    let lib = gw.library_with(KERNEL_SOURCE, math_mode)?;
    let pso = lib.pipeline(KERNEL_NAME)?;

    let flat = ir.flatten();
    assert_strides(&flat, ir);

    let params_u32 = gw.buffer_u32(&flat.params_u32)?;
    let params_f32 = gw.buffer_f32(&flat.params_f32)?;
    // A frame with no strokes still needs valid bindings; an empty Metal
    // buffer is an error, not an empty array.
    let segments = gw.buffer_f32(nonempty_f32(&flat.segments))?;
    let path_u32 = gw.buffer_u32(nonempty_u32(&flat.path_u32))?;
    let path_f32 = gw.buffer_f32(nonempty_f32(&flat.path_f32))?;
    let styles = gw.buffer_f32(nonempty_f32(&flat.styles))?;
    let tile_offsets = gw.buffer_u32(&flat.tile_offsets)?;
    let tile_draws = gw.buffer_u32(&flat.tile_draws)?;

    let n = (ir.grid.width * ir.grid.height * 4) as usize;
    let readback_bytes = n * 4;
    let surface = gw.buffer_zeroed(readback_bytes)?;

    // One threadgroup per tile, one thread per pixel. If the tile is larger
    // than the pipeline's occupancy ceiling this is a caller error, not
    // something to silently reshape — a quietly-halved threadgroup would make
    // the pixel-to-thread mapping a mystery.
    let tile = ir.grid.tile as usize;
    let want = tile * tile;
    let max = pso.max_threads_per_threadgroup();
    if want > max {
        return Err(Error::Kernel(format!(
            "tile {tile}x{tile} needs {want} threads/threadgroup; this pipeline allows {max}"
        )));
    }

    gw.dispatch(
        &pso,
        &[
            &params_u32,
            &params_f32,
            &segments,
            &path_u32,
            &path_f32,
            &styles,
            &tile_offsets,
            &tile_draws,
            &surface,
        ],
        Grid::grid_2d(ir.grid.cols() as usize, ir.grid.rows() as usize, tile, tile),
    )?;

    let mut pixels = vec![0.0f32; n];
    surface.read_f32(&mut pixels)?;

    Ok((
        crate::cpu::Surface {
            width: ir.grid.width,
            height: ir.grid.height,
            pixels,
        },
        AnnexReport {
            device: gw.device_name(),
            unified_memory: gw.has_unified_memory(),
            threads_per_threadgroup: want,
            max_threads_per_threadgroup: max,
            thread_execution_width: pso.thread_execution_width(),
            upload_bytes: flat.upload_bytes(),
            readback_bytes,
            math_mode,
        },
    ))
}

/// True iff this machine can run the annex at all.
pub fn is_available() -> bool {
    Gateway::open().is_ok()
}

// --------------------------------------------------------------- the fill stage

/// §10.2's analytic fill as Metal kernels — fm-orn's subject.
pub const FILL_KERNEL_SOURCE: &str = include_str!("shaders/analytic_fill.metal");

/// The largest tile edge the fill kernels' per-thread arrays are sized for,
/// mirrored from the shader's `MAX_TILE`.
pub const FILL_MAX_TILE: u32 = 32;

/// Which dispatch shape the fill runs under.
///
/// The whole question fm-orn exists to answer, made a parameter so both answers
/// are measured rather than one being assumed. Neither shape uses an atomic, a
/// barrier, or threadgroup memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillShape {
    /// One thread per scanline of the tile: §10.2 literally, an accumulator of
    /// `tile + 1` cells and a serial prefix along x. Uses `tile` of the
    /// threadgroup's lanes and `tile` pixels of per-thread state.
    Scanline,
    /// One thread per pixel — the stroke kernel's shape. Each pixel re-derives
    /// the winding that passed to its left; no accumulator, no scan.
    PerPixel,
}

impl FillShape {
    /// The kernel entry point this shape dispatches.
    pub fn kernel_name(self) -> &'static str {
        match self {
            FillShape::Scanline => "fill_scanline",
            FillShape::PerPixel => "fill_pixel",
        }
    }
}

/// Render a fill-only `ir` on Metal.
///
/// Refuses a non-`Fill` IR by name, for the same two reasons the stroke entry
/// point refuses a non-`Stroke` one: a kernel that silently skipped the strokes
/// would produce a plausible, wrong picture, and D-18 refuses GPU work in the
/// certified path permanently.
pub fn render_fill(
    ir: &RenderIr,
    mono: &MonoTable,
    classes: &TileClasses,
    shape: FillShape,
    math_mode: MathMode,
) -> Result<(crate::cpu::Surface, AnnexReport), Error> {
    if let Some(bad) = ir.paths.iter().find(|p| p.kind != DrawKind::Fill) {
        return Err(Error::Kernel(format!(
            "the fill annex renders {:?} only; this IR contains a {:?} path. \
             Strokes are the other kernel (see `render`), and a certified frame \
             must never reach a GPU at all (D-18)",
            DrawKind::Fill,
            bad.kind
        )));
    }
    if ir.grid.tile > FILL_MAX_TILE {
        return Err(Error::Kernel(format!(
            "tile {0}x{0} exceeds the fill kernels' MAX_TILE of {FILL_MAX_TILE}; \
             the per-thread accumulator is sized at compile time and a quietly \
             truncated tile would render garbage",
            ir.grid.tile
        )));
    }
    assert_eq!(
        classes.flags.len(),
        ir.tiles.draws.len(),
        "the tile classification must be parallel to the command list — a stale \
         classification is the one input that can make the two engines disagree \
         about something neither of them computed"
    );

    let gw = Gateway::open()?;
    let lib = gw.library_with(FILL_KERNEL_SOURCE, math_mode)?;
    let pso = lib.pipeline(shape.kernel_name())?;

    let flat = flatten_fill(ir, mono);
    assert_fill_strides(&flat, ir, mono);

    // A frame with no fills still needs valid bindings; an empty Metal buffer is
    // an error, not an empty array. The padding lives here rather than in the
    // derivation so `FlatFill` keeps its declared strides for an empty scene.
    let params_u32 = gw.buffer_u32(&flat.params_u32)?;
    let params_f32 = gw.buffer_f32(&flat.params_f32)?;
    let pieces = gw.buffer_f32(nonempty_f32(&flat.pieces))?;
    let path_u32 = gw.buffer_u32(nonempty_u32(&flat.path_u32))?;
    let path_f32 = gw.buffer_f32(nonempty_f32(&flat.path_f32))?;
    let styles = gw.buffer_f32(nonempty_f32(&flat.styles))?;
    let tile_offsets = gw.buffer_u32(&flat.tile_offsets)?;
    let tile_draws = gw.buffer_u32(nonempty_u32(&flat.tile_draws))?;
    let tile_flags = gw.buffer_u32(nonempty_u32(&classes.flags))?;

    let n = (ir.grid.width * ir.grid.height * 4) as usize;
    let readback_bytes = n * 4;
    let surface = gw.buffer_zeroed(readback_bytes)?;

    let tile = ir.grid.tile as usize;
    let (tx, ty) = match shape {
        FillShape::Scanline => (1, tile),
        FillShape::PerPixel => (tile, tile),
    };
    let want = tx * ty;
    let max = pso.max_threads_per_threadgroup();
    if want > max {
        return Err(Error::Kernel(format!(
            "{:?} needs {want} threads/threadgroup; this pipeline allows {max}",
            shape
        )));
    }

    gw.dispatch(
        &pso,
        &[
            &params_u32,
            &params_f32,
            &pieces,
            &path_u32,
            &path_f32,
            &styles,
            &tile_offsets,
            &tile_draws,
            &tile_flags,
            &surface,
        ],
        Grid::grid_2d(ir.grid.cols() as usize, ir.grid.rows() as usize, tx, ty),
    )?;

    let mut pixels = vec![0.0f32; n];
    surface.read_f32(&mut pixels)?;

    Ok((
        crate::cpu::Surface {
            width: ir.grid.width,
            height: ir.grid.height,
            pixels,
        },
        AnnexReport {
            device: gw.device_name(),
            unified_memory: gw.has_unified_memory(),
            threads_per_threadgroup: want,
            max_threads_per_threadgroup: max,
            thread_execution_width: pso.thread_execution_width(),
            upload_bytes: flat.upload_bytes() + 4 * classes.flags.len(),
            readback_bytes,
            math_mode,
        },
    ))
}

/// The fill stage's half of the mirror rule. See [`assert_strides`].
fn assert_fill_strides(flat: &FlatFill, ir: &RenderIr, mono: &MonoTable) {
    assert_eq!(PIECE_STRIDE, 6, "shader hard-codes PIECE_STRIDE 6");
    assert_eq!(
        FILL_PATH_U32_STRIDE, 4,
        "shader hard-codes FILL_PATH_U32_STRIDE 4"
    );
    assert_eq!(
        FILL_PATH_F32_STRIDE, 4,
        "shader hard-codes FILL_PATH_F32_STRIDE 4"
    );
    assert_eq!(
        FILL_STYLE_STRIDE, 12,
        "shader hard-codes FILL_STYLE_STRIDE 12"
    );
    assert_eq!(flat.pieces.len(), mono.pieces.len() * PIECE_STRIDE);
    assert_eq!(flat.tile_offsets.len(), ir.grid.count() + 1);
}

/// The host's half of the mirror rule: the strides the shader hard-codes must
/// equal the strides [`FlatIr`] packs. A silent mismatch would render garbage
/// that looks like a maths bug, so it is checked once per dispatch — the cost
/// is four comparisons against a frame's worth of work.
fn assert_strides(flat: &FlatIr, ir: &RenderIr) {
    assert_eq!(SEGMENT_STRIDE, 8, "shader hard-codes SEGMENT_STRIDE 8");
    assert_eq!(PATH_U32_STRIDE, 4, "shader hard-codes PATH_U32_STRIDE 4");
    assert_eq!(PATH_F32_STRIDE, 4, "shader hard-codes PATH_F32_STRIDE 4");
    assert_eq!(STYLE_STRIDE, 8, "shader hard-codes STYLE_STRIDE 8");
    assert_eq!(flat.segments.len(), ir.segments.len() * SEGMENT_STRIDE);
    assert_eq!(flat.tile_offsets.len(), ir.grid.count() + 1);
}

fn nonempty_f32(v: &[f32]) -> &[f32] {
    if v.is_empty() { &[0.0] } else { v }
}

fn nonempty_u32(v: &[u32]) -> &[u32] {
    if v.is_empty() { &[0] } else { v }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_non_stroke_ir_is_refused_by_name() {
        use crate::ir::{RenderIr, Style, TileGrid};
        use fmn_geom::quadpath::QuadPath;
        let mut p = QuadPath::default();
        p.start_new_path([1.0, 1.0, 0.0]);
        p.add_line_to([9.0, 9.0, 0.0], false).unwrap();
        let mut ir = RenderIr::new(
            TileGrid {
                width: 16,
                height: 16,
                tile: 16,
            },
            [0.0; 4],
        );
        ir.compile_path(&p, Style::flat([1.0; 4], 2.0, 1.5), DrawKind::Fill)
            .unwrap();
        ir.bin();
        // Refused BEFORE the device is opened, so the refusal is the same on a
        // machine with no GPU as on one with a GPU — otherwise this contract
        // would only be testable on Apple silicon.
        match render(&ir) {
            Err(Error::Kernel(msg)) => {
                assert!(msg.contains("Fill"), "message must name the kind: {msg}")
            }
            other => panic!("a fill IR must be refused, got {other:?}"),
        }
    }

    #[test]
    fn the_kernel_source_declares_the_strides_the_ir_packs() {
        // Cheap, runs everywhere, and catches the one class of error that
        // would otherwise only appear as wrong pixels on one machine.
        for (name, value) in [
            ("SEGMENT_STRIDE", SEGMENT_STRIDE),
            ("PATH_U32_STRIDE", PATH_U32_STRIDE),
            ("PATH_F32_STRIDE", PATH_F32_STRIDE),
            ("STYLE_STRIDE", STYLE_STRIDE),
        ] {
            let want = format!("#define {name} {value}");
            assert!(
                KERNEL_SOURCE.contains(&want),
                "shader is missing `{want}` — the mirror rule is broken"
            );
        }
    }

    #[test]
    fn the_kernel_entry_point_exists_in_the_source() {
        assert!(KERNEL_SOURCE.contains(&format!("kernel void {KERNEL_NAME}(")));
    }

    #[test]
    fn the_shader_binds_every_table_the_host_supplies() {
        for i in 0..=8 {
            assert!(
                KERNEL_SOURCE.contains(&format!("[[buffer({i})]]")),
                "shader does not bind buffer {i}"
            );
        }
    }

    #[test]
    fn the_fill_shader_declares_the_strides_the_host_packs() {
        for (name, value) in [
            ("PIECE_STRIDE", PIECE_STRIDE),
            ("FILL_PATH_U32_STRIDE", FILL_PATH_U32_STRIDE),
            ("FILL_PATH_F32_STRIDE", FILL_PATH_F32_STRIDE),
            ("FILL_STYLE_STRIDE", FILL_STYLE_STRIDE),
            ("MAX_TILE", FILL_MAX_TILE as usize),
            (
                "CLASS_INTERIOR",
                crate::analytic_fill::CLASS_INTERIOR as usize,
            ),
        ] {
            let want = format!("#define {name} {value}");
            let want_u = format!("#define {name} {value}u");
            assert!(
                FILL_KERNEL_SOURCE.contains(&want) || FILL_KERNEL_SOURCE.contains(&want_u),
                "fill shader is missing `{want}` — the mirror rule is broken"
            );
        }
    }

    #[test]
    fn both_fill_entry_points_exist_and_bind_every_table() {
        for shape in [FillShape::Scanline, FillShape::PerPixel] {
            assert!(
                FILL_KERNEL_SOURCE.contains(&format!("kernel void {}(", shape.kernel_name())),
                "fill shader has no entry point for {shape:?}"
            );
        }
        for i in 0..=9 {
            assert!(
                FILL_KERNEL_SOURCE.contains(&format!("[[buffer({i})]]")),
                "fill shader does not bind buffer {i}"
            );
        }
    }

    #[test]
    fn a_non_fill_ir_is_refused_by_the_fill_annex_by_name() {
        use crate::analytic_fill::{MonoTable, classify};
        use crate::ir::{RenderIr, Style, TileGrid};
        use fmn_geom::quadpath::QuadPath;
        let mut p = QuadPath::default();
        p.start_new_path([1.0, 1.0, 0.0]);
        p.add_line_to([9.0, 9.0, 0.0], false).unwrap();
        let mut ir = RenderIr::new(
            TileGrid {
                width: 16,
                height: 16,
                tile: 16,
            },
            [0.0; 4],
        );
        ir.compile_path(&p, Style::flat([1.0; 4], 2.0, 1.5), DrawKind::Stroke)
            .unwrap();
        ir.bin();
        let mono = MonoTable::build(&ir);
        let classes = classify(&ir, &mono);
        // Refused before the device is opened, so the contract is testable on a
        // machine with no GPU — which is where CI lives.
        match render_fill(&ir, &mono, &classes, FillShape::Scanline, MathMode::Safe) {
            Err(Error::Kernel(msg)) => {
                assert!(msg.contains("Stroke"), "message must name the kind: {msg}")
            }
            other => panic!("a stroke IR must be refused, got {other:?}"),
        }
    }

    #[test]
    fn an_oversized_tile_is_a_typed_error_not_a_reshaped_dispatch() {
        use crate::analytic_fill::{MonoTable, classify};
        use crate::ir::{RenderIr, Style, TileGrid};
        use fmn_geom::quadpath::QuadPath;
        let mut p = QuadPath::default();
        p.start_new_path([1.0, 1.0, 0.0]);
        for q in [[60.0, 1.0], [60.0, 60.0], [1.0, 60.0], [1.0, 1.0]] {
            p.add_line_to([q[0], q[1], 0.0], false).unwrap();
        }
        let mut ir = RenderIr::new(
            TileGrid {
                width: 64,
                height: 64,
                tile: 64,
            },
            [0.0; 4],
        );
        ir.compile_path(&p, Style::flat([1.0; 4], 0.0, 1.5), DrawKind::Fill)
            .unwrap();
        ir.bin();
        let mono = MonoTable::build(&ir);
        let classes = classify(&ir, &mono);
        match render_fill(&ir, &mono, &classes, FillShape::Scanline, MathMode::Safe) {
            Err(Error::Kernel(msg)) => assert!(msg.contains("MAX_TILE"), "{msg}"),
            other => panic!("an oversized tile must be refused, got {other:?}"),
        }
    }
}
