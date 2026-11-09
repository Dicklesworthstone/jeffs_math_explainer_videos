//! Nonzero-winding fill and radial glow — the two stages G0-6's determinism
//! frame needs that the stroke stage does not provide.
//!
//! ## What this is, and what it is not
//!
//! §10.2's fill evaluates coverage **analytically on the curves**: y-monotone
//! splits, exact intersections from the closed-form quadratic root, and signed
//! trapezoidal *area* accumulation per cell. That is fm-5oi's deliverable and
//! fm-orn's GPU-mapping question, and this module is emphatically not it.
//!
//! What this is: coverage that is **exact in x and supersampled in y**. Each
//! pixel row is sampled at `SUBSAMPLES_Y` sub-scanlines; on each sub-scanline
//! every segment's crossings come from the same closed-form quadratic root the
//! stroke stage already uses, the winding number is accumulated in ascending
//! x order, and the covered spans contribute exact fractional coverage to the
//! pixels they partially overlap.
//!
//! That is a *defined, deterministic, and fully specified* fill — which is
//! precisely what a determinism spike needs, because the question under test is
//! "do two platforms compute the same bits from the same algorithm", not "is
//! this the algorithm W5 will ship". Calling it §10.2 would have been the
//! easier sentence to write and the wrong one: the G0-6 result would then have
//! looked like evidence about a fill nobody has built yet.
//!
//! ## Why it still exercises what the spike needs
//!
//! Root-solving per sub-scanline per segment, a signed accumulation whose order
//! is fixed, a gradient evaluated per pixel, and alpha compositing into the
//! same painter-ordered accumulator the strokes use. Every one of those is a
//! place where two platforms could disagree, which is the point.

use crate::ir::{RenderIr, Style};
use crate::sdf;

/// Sub-scanlines per pixel row.
///
/// Four is the Reference's own SSAA habit and enough that the fill's edges are
/// visibly antialiased rather than stair-stepped. It is a **semantic** constant
/// here, not a quality knob: change it and the frame hash changes, which is
/// exactly why it is named, fixed, and journaled into the input closure rather
/// than passed in.
pub const SUBSAMPLES_Y: usize = 4;

/// Coverage of one filled path over one pixel row, written into `row`.
///
/// `row` is `x_hi - x_lo` wide and starts at pixel `x_lo`. Coverage
/// accumulates additively across sub-scanlines and is scaled once at the end,
/// so the division happens in one place and in one order.
pub fn fill_row(
    ir: &RenderIr,
    first: u32,
    count: u32,
    py: u32,
    x_lo: u32,
    x_hi: u32,
    row: &mut [f64],
) {
    debug_assert_eq!(row.len(), (x_hi - x_lo) as usize);
    row.fill(0.0);
    if x_hi <= x_lo {
        return;
    }

    // Reused across sub-scanlines so the allocation happens once per row, not
    // once per sample — the zero-steady-state-allocation habit §17.2 measures,
    // practised early because retrofitting it is what makes it expensive.
    let mut crossings: Vec<(f64, i32)> = Vec::with_capacity(16);

    for sub in 0..SUBSAMPLES_Y {
        // Sample centres of the sub-rows: (sub + 0.5) / SUBSAMPLES_Y.
        let sy = py as f64 + (sub as f64 + 0.5) / SUBSAMPLES_Y as f64;
        crossings.clear();

        for i in first..(first + count) {
            let seg = &ir.segments[i as usize];
            let y0 = seg.p0[1] as f64;
            let y1 = seg.p1[1] as f64;
            let y2 = seg.p2[1] as f64;

            // B_y(t) - sy = 0, in the same coefficient form the stroke stage
            // uses: a + b t + c t².
            let a = y0 - sy;
            let b = 2.0 * (y1 - y0);
            let c = (y2 - y1) - (y1 - y0);

            let mut roots = [0.0f64; 3];
            let n = sdf::solve_quadratic(c, b, a, &mut roots);
            for &t in roots.iter().take(n) {
                // Half-open in t so a crossing exactly at a shared anchor is
                // counted once, not twice or zero times. Without this a closed
                // path leaks winding at every joint.
                if !(0.0..1.0).contains(&t) {
                    continue;
                }
                let x = eval_quadratic(seg.p0[0] as f64, seg.p1[0] as f64, seg.p2[0] as f64, t);
                // dB_y/dt decides the direction the boundary crosses the ray.
                let dy = b + 2.0 * c * t;
                let dir = if dy > 0.0 {
                    1
                } else if dy < 0.0 {
                    -1
                } else {
                    continue; // tangential touch: no winding change
                };
                crossings.push((x, dir));
            }
        }

        if crossings.is_empty() {
            continue;
        }
        // Ascending x, with the direction breaking ties, so the accumulation
        // order is total and identical on every platform. `sort_by` is a stable
        // merge sort; a total key makes stability irrelevant, which is the
        // property we actually want.
        crossings.sort_by(|l, r| {
            l.0.partial_cmp(&r.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(l.1.cmp(&r.1))
        });

        // Walk the crossings; a span is inside wherever the running winding
        // number is nonzero.
        let mut winding = 0;
        let mut span_start = 0.0f64;
        for &(x, dir) in crossings.iter() {
            if winding != 0 {
                add_span(row, x_lo, x_hi, span_start, x);
            }
            winding += dir;
            span_start = x;
        }
    }

    let scale = 1.0 / SUBSAMPLES_Y as f64;
    for c in row.iter_mut() {
        *c = (*c * scale).clamp(0.0, 1.0);
    }
}

/// Add `[x0, x1)`'s exact horizontal coverage to the row's pixels.
fn add_span(row: &mut [f64], x_lo: u32, x_hi: u32, x0: f64, x1: f64) {
    let lo = x0.max(x_lo as f64);
    let hi = x1.min(x_hi as f64);
    if hi <= lo {
        return;
    }
    let first = lo.floor() as i64;
    let last = (hi.ceil() as i64) - 1;
    for px in first..=last {
        if px < x_lo as i64 || px >= x_hi as i64 {
            continue;
        }
        let cell_lo = px as f64;
        let cell_hi = cell_lo + 1.0;
        let covered = hi.min(cell_hi) - lo.max(cell_lo);
        if covered > 0.0 {
            row[(px - x_lo as i64) as usize] += covered;
        }
    }
}

fn eval_quadratic(p0: f64, p1: f64, p2: f64, t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * p0 + 2.0 * u * t * p1 + t * t * p2
}

/// The gradient colour at a screen point, for a fill.
///
/// Projection onto [`Style::gradient_axis`], clamped to `[0, 1]`, then a
/// straight linear-light lerp. Defined and boring on purpose: §10.2's real
/// interior field is fm-5oi's to design, and a spike that invented one would
/// be pre-empting that design with something nobody reviewed.
pub fn gradient_at(style: &Style, p: [f64; 2]) -> [f64; 4] {
    let ax = style.gradient_axis;
    let dx = (ax[2] - ax[0]) as f64;
    let dy = (ax[3] - ax[1]) as f64;
    let denom = dx * dx + dy * dy;
    let t = if denom <= 0.0 {
        0.0
    } else {
        (((p[0] - ax[0] as f64) * dx + (p[1] - ax[1] as f64) * dy) / denom).clamp(0.0, 1.0)
    };
    let mut out = [0.0f64; 4];
    for (k, o) in out.iter_mut().enumerate() {
        let a = style.rgba[k] as f64;
        let b = style.rgba_end[k] as f64;
        *o = a + (b - a) * t;
    }
    out
}

/// The Reference's `true_dot` radial profile, kept in full.
///
/// `manimlib/shaders/true_dot/frag.glsl`, in its own order:
///
/// ```glsl
/// float r = length(uv_coords.xy);
/// if (r > 1.0) discard;
/// if (glow_factor > 0) { frag_color.a *= pow(1 - r, glow_factor); }
/// frag_color.a *= smoothstep(1.0, 1.0 - scaled_aaw, r);
/// ```
///
/// with `scaled_aaw = anti_alias_width * pixel_size / radius` and `r` the
/// distance from the centre in radius units. `glow_factor = 0` is `DotCloud`'s
/// hard-edged antialiased dot; `2.0` is what `GlowDots` uses, and it is the
/// `pow` that makes this "glow falloff" rather than "a circle".
///
/// The `pow` routes through fmn-dmath like everything else in the certified
/// layer — which is the point of putting a glow in the determinism frame at
/// all: it is the only per-pixel `pow` in the picture.
pub fn glow_coverage(distance: f64, radius: f64, aa_width: f64, glow_factor: f64) -> f64 {
    if radius <= 0.0 {
        return 0.0;
    }
    let r = distance / radius;
    if r > 1.0 {
        return 0.0; // the shader's `discard`
    }
    let mut a = 1.0;
    if glow_factor > 0.0 {
        a *= fmn_dmath::pow(1.0 - r, glow_factor);
    }
    let scaled_aaw = (aa_width / radius).max(sdf::MIN_AA_WIDTH);
    // smoothstep(1.0, 1.0 - scaled_aaw, r): 1 at the centre, 0 past the rim.
    let s = ((1.0 - r) / scaled_aaw).clamp(0.0, 1.0);
    a * s * s * (3.0 - 2.0 * s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{DrawKind, Style, TileGrid};
    use fmn_geom::quadpath::QuadPath;

    fn rect_ir(x0: f64, y0: f64, x1: f64, y1: f64) -> RenderIr {
        let mut p = QuadPath::default();
        p.start_new_path([x0, y0, 0.0]);
        p.add_line_to([x1, y0, 0.0], false).unwrap();
        p.add_line_to([x1, y1, 0.0], false).unwrap();
        p.add_line_to([x0, y1, 0.0], false).unwrap();
        p.add_line_to([x0, y0, 0.0], false).unwrap();
        let mut ir = RenderIr::new(
            TileGrid {
                width: 64,
                height: 64,
                tile: 16,
            },
            [0.0, 0.0, 0.0, 1.0],
        );
        ir.compile_path(
            &p,
            Style::flat([1.0, 1.0, 1.0, 1.0], 0.0, 1.5),
            DrawKind::Fill,
        )
        .unwrap();
        ir
    }

    fn row_of(ir: &RenderIr, py: u32, w: u32) -> Vec<f64> {
        let mut row = vec![0.0; w as usize];
        let h = &ir.paths[0];
        fill_row(ir, h.first_segment, h.segment_count, py, 0, w, &mut row);
        row
    }

    #[test]
    fn a_pixel_aligned_rectangle_fills_exactly() {
        // Integer bounds: interior pixels must be fully covered and exterior
        // pixels exactly clear, with no AA smear where there is no edge.
        let ir = rect_ir(10.0, 10.0, 30.0, 30.0);
        let row = row_of(&ir, 20, 64);
        for (px, c) in row.iter().enumerate().take(30).skip(10) {
            assert!((c - 1.0).abs() < 1e-12, "interior pixel {px} = {c}");
        }
        // The pixels just outside the edges get *essentially* nothing, not
        // exactly nothing: the edge's x comes from a quadratic root solve, so
        // it lands within an ulp of 10.0 rather than on it, and a span
        // beginning at 9.999999999999998 genuinely covers 2e-15 of pixel 9.
        // Snapping that to zero would put a magic threshold in the fill's
        // semantics to make a test read nicer — the leak is deterministic,
        // which is the property that actually matters here.
        assert!(row[9] < 1e-12, "left of the edge: {}", row[9]);
        assert!(row[30] < 1e-12, "right of the edge: {}", row[30]);
        // A row outside the rectangle is empty.
        let out = row_of(&ir, 40, 64);
        assert!(out.iter().all(|c| *c == 0.0));
    }

    #[test]
    fn a_half_pixel_edge_gives_half_coverage() {
        // The exact-in-x property: an edge at x = 10.5 covers half of pixel 10.
        let ir = rect_ir(10.5, 10.0, 30.0, 30.0);
        let row = row_of(&ir, 20, 64);
        assert!((row[10] - 0.5).abs() < 1e-12, "got {}", row[10]);
        assert!((row[11] - 1.0).abs() < 1e-12, "got {}", row[11]);
    }

    #[test]
    fn total_coverage_integrates_to_the_analytic_area() {
        // The oracle a fill owes: summed coverage equals the true area. A
        // rectangle is exact in both axes; the tolerance covers only the
        // y-supersampling of the two horizontal edges, which are pixel-aligned
        // here, so it should in fact be exact.
        let ir = rect_ir(8.25, 12.0, 41.75, 44.0);
        let total: f64 = (0..64)
            .map(|py| row_of(&ir, py, 64).iter().sum::<f64>())
            .sum();
        let want = (41.75 - 8.25) * (44.0 - 12.0);
        assert!(
            (total - want).abs() < 1e-9,
            "coverage {total} vs analytic area {want}"
        );
    }

    #[test]
    fn a_circle_integrates_to_pi_r_squared() {
        // The curved case, where the closed-form root solve is actually doing
        // work. Quadratics approximate a circle to a small but nonzero error,
        // so the bar is 0.5 % of the area rather than exactness — tight enough
        // that a winding or ordering bug cannot hide inside it.
        let r = 22.0;
        let p = crate::arc(0.0, std::f64::consts::TAU, r, [32.0, 32.0, 0.0], None);
        let mut ir = RenderIr::new(
            TileGrid {
                width: 64,
                height: 64,
                tile: 16,
            },
            [0.0; 4],
        );
        ir.compile_path(
            &p,
            Style::flat([1.0, 1.0, 1.0, 1.0], 0.0, 1.5),
            DrawKind::Fill,
        )
        .unwrap();
        let total: f64 = (0..64)
            .map(|py| row_of(&ir, py, 64).iter().sum::<f64>())
            .sum();
        let want = std::f64::consts::PI * r * r;
        assert!(
            (total - want).abs() / want < 5e-3,
            "coverage {total} vs pi r^2 {want}"
        );
    }

    #[test]
    fn a_hole_is_a_hole_under_the_nonzero_rule() {
        // Outer square CCW, inner square CW: the winding cancels inside the
        // inner square, so it must read as empty. This is the test that fails
        // loudly if crossing directions or the sort order are wrong.
        let mut p = QuadPath::default();
        for (a, b, rev) in [(8.0, 56.0, false), (24.0, 40.0, true)] {
            let pts = if rev {
                [[a, a], [a, b], [b, b], [b, a], [a, a]]
            } else {
                [[a, a], [b, a], [b, b], [a, b], [a, a]]
            };
            p.start_new_path([pts[0][0], pts[0][1], 0.0]);
            for q in &pts[1..] {
                p.add_line_to([q[0], q[1], 0.0], false).unwrap();
            }
        }
        let mut ir = RenderIr::new(
            TileGrid {
                width: 64,
                height: 64,
                tile: 16,
            },
            [0.0; 4],
        );
        ir.compile_path(
            &p,
            Style::flat([1.0, 1.0, 1.0, 1.0], 0.0, 1.5),
            DrawKind::Fill,
        )
        .unwrap();
        let row = row_of(&ir, 32, 64);
        assert!(
            (row[16] - 1.0).abs() < 1e-9,
            "ring should be solid: {}",
            row[16]
        );
        assert!(row[32] < 1e-9, "hole should be empty: {}", row[32]);
    }

    #[test]
    fn the_glow_profile_matches_the_reference_shape() {
        let (r, aaw) = (20.0, 1.5);
        // glow_factor = 0 is DotCloud's hard-edged antialiased dot: flat inside,
        // falling only across the rim band.
        assert_eq!(glow_coverage(0.0, r, aaw, 0.0), 1.0);
        assert_eq!(glow_coverage(r, r, aaw, 0.0), 0.0);
        assert_eq!(glow_coverage(r * 2.0, r, aaw, 0.0), 0.0);
        let mut prev = 1.0;
        for i in 0..=32 {
            let d = r - aaw + aaw * (i as f64 / 32.0);
            let c = glow_coverage(d, r, aaw, 0.0);
            assert!(c <= prev + 1e-15, "not monotone at d={d}");
            prev = c;
        }
        assert_eq!(
            glow_coverage(1.0, 0.0, aaw, 0.0),
            0.0,
            "zero radius draws nothing"
        );

        // glow_factor > 0 is GlowDots: `pow(1 - r, glow_factor)` across the
        // whole disc, which is what makes it a glow rather than a circle.
        assert_eq!(glow_coverage(0.0, r, aaw, 2.0), 1.0);
        assert!(
            (glow_coverage(r * 0.5, r, aaw, 0.0) - 1.0).abs() < 1e-12,
            "the hard dot must be flat at half radius"
        );
        let mid = glow_coverage(r * 0.5, r, aaw, 2.0);
        assert!(
            (mid - 0.25).abs() < 1e-9,
            "pow(1 - 0.5, 2) = 0.25, got {mid}"
        );
        let mut prev = 1.0;
        for i in 0..=64 {
            let c = glow_coverage(r * (i as f64 / 64.0), r, aaw, 2.0);
            assert!(c <= prev + 1e-15, "the glow is not monotone at i={i}");
            prev = c;
        }
    }

    #[test]
    fn the_gradient_is_a_clamped_projection() {
        let mut st = Style::flat([0.0, 0.0, 0.0, 1.0], 1.0, 1.5);
        st.rgba_end = [1.0, 1.0, 1.0, 1.0];
        st.gradient_axis = [0.0, 0.0, 100.0, 0.0];
        assert_eq!(gradient_at(&st, [0.0, 0.0])[0], 0.0);
        assert_eq!(gradient_at(&st, [100.0, 0.0])[0], 1.0);
        assert!((gradient_at(&st, [50.0, 0.0])[0] - 0.5).abs() < 1e-12);
        // Clamped, not extrapolated, at both ends.
        assert_eq!(gradient_at(&st, [-50.0, 0.0])[0], 0.0);
        assert_eq!(gradient_at(&st, [500.0, 0.0])[0], 1.0);
        // A degenerate axis is defined, not a division by zero.
        st.gradient_axis = [7.0, 7.0, 7.0, 7.0];
        assert_eq!(gradient_at(&st, [9.0, 9.0])[0], 0.0);
    }
}
