//! fm-orn — the analytic-fill GPU mapping spike.
//!
//! Renders the fill frame through §10.2's analytic coverage on four engines and
//! prints the equivalence table the bead asks for:
//!
//! | row | what it isolates |
//! |---|---|
//! | CPU-f64 | the reference — what the stage *means* |
//! | CPU-f32 | the **arithmetic floor**: the annex's arithmetic without its hardware |
//! | Metal, safe math, per-scanline | §10.2's own dispatch shape |
//! | Metal, safe math, per-pixel | the stroke kernel's dispatch shape |
//! | Metal, fast math | the platform default G0-8's finding F9 refused |
//!
//! Off Apple silicon the Metal rows report "unavailable" and the CPU rows still
//! run, which is the point of having an arithmetic floor at all: CI can measure
//! the part that is not about hardware.
//!
//! ```text
//! cargo run --release --bin g0_orn_fill [-- <output-dir>]
//! ```

use fmn_spike_accelerator::analytic_fill::{MonoTable, classify, prune_occluded};
use fmn_spike_accelerator::annex::{self, FillShape};
use fmn_spike_accelerator::compare;
use fmn_spike_accelerator::cpu::{self, FillKernel, Precision, Surface};
use fmn_spike_accelerator::scene;
use ft_kernel_metal::compute::MathMode;
use std::time::Instant;

/// Warm dispatches per configuration. Odd, so the median is a sample.
const REPS: usize = 33;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let dir = std::path::Path::new(&dir);
    std::fs::create_dir_all(dir)?;

    let t = Instant::now();
    let mut ir = scene::fill_frame();
    let mono = MonoTable::build(&ir);
    let mut classes = classify(&ir, &mono);
    let compile_ms = t.elapsed().as_secs_f64() * 1e3;

    println!("fm-orn — the analytic fill on Metal (G0-8 follow-on)");
    println!(
        "IR:   {} paths, {} segments -> {} monotone pieces ({} splits), {} styles, \
         {} tile commands",
        ir.paths.len(),
        ir.segments.len(),
        mono.pieces.len(),
        mono.pieces.len() - ir.segments.len(),
        ir.styles.len(),
        ir.tiles.draws.len()
    );
    println!(
        "      {}x{} @ tile {}, IR+monotone+classify {:.2} ms",
        ir.grid.width, ir.grid.height, ir.grid.tile, compile_ms
    );
    println!(
        "      tile classes: {} of {} commands interior ({:.1}%)",
        classes.interior_count(),
        classes.flags.len(),
        100.0 * classes.interior_count() as f64 / classes.flags.len().max(1) as f64
    );

    // ---- occlusion pruning: measured, and proved to change nothing.
    let unpruned = render_cpu(&ir, &mono, &classes, Precision::Reference);
    let t = Instant::now();
    let prune = prune_occluded(&mut ir, &mut classes);
    let prune_ms = t.elapsed().as_secs_f64() * 1e3;
    let pruned = render_cpu(&ir, &mono, &classes, Precision::Reference);
    println!(
        "Prune: {} -> {} commands ({:.1}% removed) in {} tiles, {:.2} ms; \
         frames byte-identical: {}",
        prune.before,
        prune.after,
        100.0 * prune.removed_fraction(),
        prune.tiles_touched,
        prune_ms,
        unpruned.pixels == pruned.pixels
    );
    assert_eq!(
        unpruned.pixels, pruned.pixels,
        "occlusion pruning changed a pixel — that is a bug, not a trade-off"
    );

    let t = Instant::now();
    let cpu_frame = render_cpu(&ir, &mono, &classes, Precision::Reference);
    let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
    println!("CPU:  {cpu_ms:.1} ms (reference engine, single-threaded, unoptimized)");
    write_png(dir, "orn-fill-cpu.png", &cpu_frame)?;

    // The arithmetic floor: identical algorithm, only the scalar width changes.
    let f32_frame = render_cpu(&ir, &mono, &classes, Precision::AnnexF32);
    println!(
        "CPU-f32 vs CPU-f64, the arithmetic floor: {}",
        compare::diverge(&cpu_frame, &f32_frame).summary()
    );
    report_worst_pixels("CPU-f32", &cpu_frame, &f32_frame);

    let flat_bytes = {
        let f = fmn_spike_accelerator::analytic_fill::flatten_fill(&ir, &mono);
        f.upload_bytes() + 4 * classes.flags.len()
    };
    println!(
        "Upload: {} bytes ({:.1} KiB) for the whole fill IR; readback {} bytes ({:.1} MiB)",
        flat_bytes,
        flat_bytes as f64 / 1024.0,
        (ir.grid.width * ir.grid.height * 16) as usize,
        (ir.grid.width * ir.grid.height * 16) as f64 / (1024.0 * 1024.0)
    );

    if !annex::is_available() {
        println!("Metal: unavailable on this machine — the CPU rows above still stand.");
        println!(
            "       (that is the §10.7 contract: the CPU engine must stand on its own, \
             so acceleration can never mask a core regression)"
        );
        return Ok(());
    }

    // ---- the dispatch floor: the same grid with nothing in it.
    //
    // Every kernel row below sits on top of this. Without it, four numbers in
    // the same 1.5 ms neighbourhood invite a ranking that is really a
    // measurement of buffer allocation and readback — the cost G0-8's F6 already
    // identified as the boundary's real one.
    {
        let empty = fmn_spike_accelerator::ir::RenderIr::new(ir.grid, ir.background);
        let mut empty = empty;
        empty.bin();
        let empty_mono = MonoTable::build(&empty);
        let empty_classes = classify(&empty, &empty_mono);
        let mut samples = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            let t = Instant::now();
            annex::render_fill(
                &empty,
                &empty_mono,
                &empty_classes,
                FillShape::PerPixel,
                MathMode::Safe,
            )?;
            samples.push(t.elapsed().as_secs_f64() * 1e3);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN timings"));
        println!(
            "Metal dispatch floor (empty scene, same grid): {:.2} ms median \
             (min {:.2}, max {:.2}, n={REPS}) — allocate + clear + read back {:.1} MiB",
            samples[REPS / 2],
            samples[0],
            samples[REPS - 1],
            (ir.grid.width * ir.grid.height * 16) as f64 / (1024.0 * 1024.0)
        );
    }

    for shape in [FillShape::Scanline, FillShape::PerPixel] {
        for mode in [MathMode::Safe, MathMode::Fast] {
            match annex::render_fill(&ir, &mono, &classes, shape, mode) {
                Ok((frame, report)) => {
                    // Warm dispatches only, and a median of them: the first call
                    // pays a one-time MSL compile that production would do at
                    // engine construction, and a single sample of a sub-2 ms
                    // dispatch is noise with a decimal point. §17.2's rule for
                    // every published number is medians over repetitions.
                    let mut samples = Vec::with_capacity(REPS);
                    for _ in 0..REPS {
                        let t = Instant::now();
                        let (warm, _) = annex::render_fill(&ir, &mono, &classes, shape, mode)?;
                        samples.push(t.elapsed().as_secs_f64() * 1e3);
                        assert_eq!(
                            warm.pixels, frame.pixels,
                            "two dispatches of one IR disagreed — {shape:?}/{mode:?} is not \
                             deterministic within a run"
                        );
                    }
                    samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN timings"));
                    println!(
                        "Metal {shape:?}/{mode:?}: {:.2} ms median (min {:.2}, max {:.2}, \
                         n={REPS}), {} threads/threadgroup (max {}, SIMD width {}) on {}",
                        samples[REPS / 2],
                        samples[0],
                        samples[REPS - 1],
                        report.threads_per_threadgroup,
                        report.max_threads_per_threadgroup,
                        report.thread_execution_width,
                        report.device
                    );
                    println!(
                        "      vs CPU-f64: {}",
                        compare::diverge(&cpu_frame, &frame).summary()
                    );
                    println!(
                        "      vs CPU-f32: {}",
                        compare::diverge(&f32_frame, &frame).summary()
                    );
                    if mode == MathMode::Safe {
                        let name = match shape {
                            FillShape::Scanline => "orn-fill-metal-scanline.png",
                            FillShape::PerPixel => "orn-fill-metal-perpixel.png",
                        };
                        write_png(dir, name, &frame)?;
                    }
                }
                Err(e) => println!("Metal {shape:?}/{mode:?}: {e}"),
            }
        }
    }

    // The two GPU shapes against each other: same mathematics, different
    // association of one sum. Whatever this reports is the cost of the reorder,
    // separated from the cost of the hardware.
    let (a, _) = annex::render_fill(&ir, &mono, &classes, FillShape::Scanline, MathMode::Safe)?;
    let (b, _) = annex::render_fill(&ir, &mono, &classes, FillShape::PerPixel, MathMode::Safe)?;
    println!(
        "Metal scanline vs Metal per-pixel (the reorder alone): {}",
        compare::diverge(&a, &b).summary()
    );

    Ok(())
}

/// Name the handful of pixels that actually differ, and where they sit.
///
/// A divergence summary answers "how much"; a Look-Gallery argument needs "where",
/// and a *few* isolated pixels with a large error is a completely different
/// finding from a smooth drift of the same magnitude. G0-8 learned this the
/// expensive way — its residual turned out to be `f32` conditioning at curvature
/// extrema, which was only diagnosable once the pixels had coordinates.
fn report_worst_pixels(label: &str, reference: &Surface, other: &Surface) {
    let mut worst: Vec<(f64, u32, u32)> = Vec::new();
    for y in 0..reference.height {
        for x in 0..reference.width {
            let a = reference.get(x, y);
            let b = other.get(x, y);
            let d = (0..4)
                .map(|c| (a[c] as f64 - b[c] as f64).abs())
                .fold(0.0f64, f64::max);
            if d > 1e-3 {
                worst.push((d, x, y));
            }
        }
    }
    worst.sort_by(|l, r| r.0.partial_cmp(&l.0).unwrap_or(std::cmp::Ordering::Equal));
    if worst.is_empty() {
        println!("      {label}: no pixel differs by more than 1e-3");
        return;
    }
    let shown: Vec<String> = worst
        .iter()
        .take(8)
        .map(|(d, x, y)| format!("({x},{y})={d:.3}"))
        .collect();
    println!(
        "      {label}: {} pixels differ by >1e-3; worst: {}",
        worst.len(),
        shown.join(" ")
    );
}

fn render_cpu(
    ir: &fmn_spike_accelerator::ir::RenderIr,
    mono: &MonoTable,
    classes: &fmn_spike_accelerator::analytic_fill::TileClasses,
    precision: Precision,
) -> Surface {
    cpu::render_with(
        ir,
        precision,
        FillKernel::Analytic {
            mono,
            classes: Some(classes),
        },
    )
}

fn write_png(
    dir: &std::path::Path,
    name: &str,
    s: &Surface,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = dir.join(name);
    let bytes = fmn_codec::png::encode_rgba8(
        s.width,
        s.height,
        &s.to_srgb8(),
        fmn_codec::deflate::CompressionLevel::Default,
    );
    std::fs::write(&path, bytes)?;
    println!("      wrote {}", path.display());
    Ok(())
}
