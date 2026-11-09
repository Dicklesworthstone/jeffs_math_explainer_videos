//! The G0-8 runner: build the preview frame's IR, render it on both engines,
//! write the PNGs, and print the equivalence measurement.
//!
//! ```text
//! cargo run --release --bin g0_8_preview -- <output-dir>
//! ```
//!
//! On a machine with no Metal device it renders the CPU frame, says plainly
//! that the annex was unavailable, and exits 0 — the CPU engine standing on its
//! own is the designed behaviour (§10.7), not a failure. It exits non-zero only
//! when the annex was available and something actually went wrong, so CI on
//! either kind of machine reads the exit code the same way.

use std::time::Instant;

use ft_kernel_metal::compute::MathMode;

use fmn_spike_accelerator::{annex, compare, cpu, scene};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let dir = std::path::Path::new(&out_dir);
    std::fs::create_dir_all(dir)?;

    println!("G0-8 accelerator proof spike (fm-ekx)");
    println!("=====================================");

    let t = Instant::now();
    let ir = scene::preview_frame();
    let compile_ms = t.elapsed().as_secs_f64() * 1e3;
    let flat = ir.flatten();

    println!(
        "IR:   {} paths, {} segments, {} styles, {} tiles, {} tile-draws",
        ir.paths.len(),
        ir.segments.len(),
        ir.styles.len(),
        ir.grid.count(),
        ir.tiles.draws.len()
    );
    println!(
        "      {}x{} @ tile {}, upload {} bytes ({:.1} KiB), IR compile {:.2} ms",
        ir.grid.width,
        ir.grid.height,
        ir.grid.tile,
        flat.upload_bytes(),
        flat.upload_bytes() as f64 / 1024.0,
        compile_ms
    );

    let t = Instant::now();
    let cpu_frame = cpu::render(&ir);
    let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
    println!("CPU:  {cpu_ms:.1} ms (reference engine, single-threaded, unoptimized)");
    write_png(dir, "g0-8-cpu.png", &cpu_frame)?;

    // The controlled experiment: the annex's ARITHMETIC without the annex's
    // hardware. Whatever this reports is the floor under the Metal engine's
    // divergence, and it runs on any machine — so the split between "f32 did
    // this" and "the GPU did this on top of f32" is measured, never assumed.
    let f32_frame = cpu::render_at(&ir, cpu::Precision::AnnexF32);
    println!(
        "CPU-f32 vs CPU-f64, the arithmetic floor: {}",
        compare::diverge(&cpu_frame, &f32_frame).summary()
    );

    match annex::render(&ir) {
        Ok((gpu_frame, report)) => {
            // A second dispatch, timed after the library is compiled and the
            // pipeline is warm: the first call pays a one-time MSL compile that
            // production would do at engine construction, and reporting it as
            // the frame time would be a lie in the annex's favour.
            let t = Instant::now();
            let (warm, _) = annex::render(&ir)?;
            let gpu_ms = t.elapsed().as_secs_f64() * 1e3;
            assert_eq!(
                warm.pixels, gpu_frame.pixels,
                "two dispatches of one IR disagreed — the annex is not deterministic within a run"
            );

            println!(
                "Metal: {} (unified memory {}), {}/{} threads per threadgroup, SIMD width {}",
                report.device,
                report.unified_memory,
                report.threads_per_threadgroup,
                report.max_threads_per_threadgroup,
                report.thread_execution_width
            );
            println!(
                "      {gpu_ms:.1} ms warm, upload {} B, readback {} B",
                report.upload_bytes, report.readback_bytes
            );
            write_png(dir, "g0-8-metal.png", &gpu_frame)?;

            let d = compare::diverge(&cpu_frame, &gpu_frame);
            println!("Equivalence, safe math (§16.3): {}", d.summary());

            // The same kernel under Metal's own default. Reported, not hidden:
            // an annex whose numbers depend on an unstated compiler default is
            // an annex nobody can budget.
            let (fast_frame, _) = annex::render_with(&ir, MathMode::Fast)?;
            let df = compare::diverge(&cpu_frame, &fast_frame);
            println!("Equivalence, FAST math (§16.3): {}", df.summary());
            if std::env::var("G0_8_DIAGNOSE").is_ok() {
                diagnose(&ir, &cpu_frame, &gpu_frame);
            }
            write_png(
                dir,
                "g0-8-diff.png",
                &amplified_diff(&cpu_frame, &gpu_frame),
            )?;
        }
        Err(ft_kernel_metal::Error::Unavailable) => {
            println!("Metal: unavailable on this machine — CPU frame written, annex skipped.");
            println!("      (This is the designed fallback, not a failure: §10.7.)");
        }
        Err(e) => return Err(Box::new(e)),
    }

    println!("Wrote PNGs to {}", dir.display());
    Ok(())
}

/// Name the worst-diverging pixels and what was drawn there.
///
/// A divergence summary that says "0.01 % of components differ" is only useful
/// if you can find out *which* — otherwise the budget is a number nobody can
/// act on. This prints the offending pixels with the paths binned to their
/// tile, so a divergence can be traced to a primitive hint, a stroke width, or
/// a specific curve rather than attributed to "f32".
fn diagnose(ir: &fmn_spike_accelerator::ir::RenderIr, a: &cpu::Surface, b: &cpu::Surface) {
    let mut worst: Vec<(f32, u32, u32)> = Vec::new();
    for y in 0..a.height {
        for x in 0..a.width {
            let pa = a.get(x, y);
            let pb = b.get(x, y);
            let d = (0..4).fold(0.0f32, |m, i| m.max((pa[i] - pb[i]).abs()));
            if d > 0.0 {
                worst.push((d, x, y));
            }
        }
    }
    worst.sort_by(|l, r| r.0.partial_cmp(&l.0).unwrap());
    println!("Diagnose: {} pixels differ at all", worst.len());
    let cols = ir.grid.cols();
    for &(d, x, y) in worst.iter().take(10) {
        let t = ((y / ir.grid.tile) * cols + x / ir.grid.tile) as usize;
        let run = &ir.tiles.draws[ir.tiles.offsets[t] as usize..ir.tiles.offsets[t + 1] as usize];
        let paths: Vec<String> = run
            .iter()
            .map(|&p| {
                let h = &ir.paths[p as usize];
                let s = &ir.styles[h.style as usize];
                format!(
                    "#{p}({:?},segs={},w={}..{})",
                    h.hint, h.segment_count, s.width_start, s.width_end
                )
            })
            .collect();
        println!(
            "  ({x:4},{y:4}) Δ{d:.4}  cpu {:?}  gpu {:?}  tile draws: {}",
            a.get(x, y),
            b.get(x, y),
            paths.join(" ")
        );
    }
}

/// A viewable difference image: per-channel absolute difference multiplied by
/// 32 and clamped, so a divergence a human could never see in the frames
/// themselves is still visible in the diff.
fn amplified_diff(a: &cpu::Surface, b: &cpu::Surface) -> cpu::Surface {
    let pixels = a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0)
        .flat_map(|(pa, pb)| {
            let d = |i: usize| ((pa[i] - pb[i]).abs() * 32.0).min(1.0);
            [d(0), d(1), d(2), 1.0]
        })
        .collect();
    cpu::Surface {
        width: a.width,
        height: a.height,
        pixels,
    }
}

fn write_png(
    dir: &std::path::Path,
    name: &str,
    surface: &cpu::Surface,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fmn_codec::png::encode_rgba8(
        surface.width,
        surface.height,
        &surface.to_srgb8(),
        fmn_codec::deflate::CompressionLevel::Default,
    );
    std::fs::write(dir.join(name), bytes)?;
    Ok(())
}
