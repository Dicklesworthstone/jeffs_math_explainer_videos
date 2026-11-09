// §10.2's analytic fill, expressed as Metal compute kernels — fm-orn, the
// follow-on G0-8's report named in its §5 ("it should be spiked before W5
// commits to a fill layout").
//
// WHY A SECOND SHADER AND NOT A SECOND BRANCH IN THE FIRST. The stroke stage is
// per-pixel independent; the fill's coverage is a per-scanline signed
// accumulation whose ordering matters. That is a different dispatch question,
// not a different kernel body, so the honest experiment is two kernels over the
// same IR rather than one kernel with a mode flag.
//
// THE MIRROR RULE. Every statement below has a twin in `src/analytic_fill.rs`,
// in the same order, with the same branches and the same guards. The Rust side
// is generic over the scalar width so its f64 and f32 instantiations cannot
// drift; this file is the one hand-kept copy, which is why the host asserts its
// strides and entry points in `annex.rs`.
//
// TWO DISPATCH SHAPES, MEASURED AGAINST EACH OTHER:
//
//   fill_scanline — one thread per scanline of the tile. This is §10.2
//     literally: an accumulator of `tile + 1` cells, one serial prefix sum
//     along x. It needs `tile` pixels of per-thread state, and it fills only
//     `tile` of the threadgroup's lanes.
//
//   fill_pixel    — one thread per pixel, the same shape the stroke kernel
//     already uses. Each pixel sums, over the pieces, the winding that passed
//     to its left plus its own cell's trapezoid; no accumulator, no scan, no
//     per-thread arrays.
//
// Neither uses an atomic, a barrier, or threadgroup memory. The threadgroup-
// local scanline reduction G0-8 assessed as "the natural shape" turns out not to
// be needed by either: the accumulation never crosses a thread.

#include <metal_stdlib>
using namespace metal;

// Strides — mirrored from `analytic_fill.rs`'s PIECE_STRIDE / FILL_* consts.
#define PIECE_STRIDE 6
#define FILL_PATH_U32_STRIDE 4
#define FILL_PATH_F32_STRIDE 4
#define FILL_STYLE_STRIDE 12

// §10.4's per-command tile class.
#define CLASS_INTERIOR 1u

// The largest tile edge the per-thread arrays are sized for. The host refuses a
// larger tile by name rather than silently reshaping the dispatch — G0-8's F4
// rule, applied to a second resource.
#define MAX_TILE 32

// A few f32 epsilons, relative to the polynomial's own scale. Sized to THIS
// engine's precision: G0-8's finding F8 is that one absolute constant shared
// between an f64 engine and an f32 engine is a bug only one of them can see.
#define DEGENERATE_REL 1e-6f

// Written longhand, not `min`/`max`, for the same reason `sign_or_positive`
// exists: two standard libraries agreeing about ordinary values is not the same
// as agreeing about zero and NaN.
static float fmin_(float a, float b) { return (a < b) ? a : b; }
static float fmax_(float a, float b) { return (a > b) ? a : b; }

// +1 for zero and positives, -1 for negatives. NOT MSL's `sign()`.
static float sign_or_positive(float x) { return (x >= 0.0f) ? 1.0f : -1.0f; }

// Real roots of a2 t^2 + a1 t + a0. Mirrors analytic_fill::solve_quadratic,
// which is itself asserted equal to sdf::solve_quadratic at f64.
static int solve_quadratic(float a2, float a1, float a0, thread float *out) {
    float scale = fmax_(fmax_(fabs(a2), fabs(a1)), fabs(a0));
    if (scale <= 0.0f) return 0;
    float tol = DEGENERATE_REL * scale;
    if (fabs(a2) <= tol) {
        if (fabs(a1) <= tol) return 0;
        out[0] = -a0 / a1;
        return 1;
    }
    float disc = a1 * a1 - 4.0f * a2 * a0;
    if (disc < 0.0f) return 0;
    float s = sqrt(disc);
    float q = -0.5f * (a1 + sign_or_positive(a1) * s);
    if (q == 0.0f) { out[0] = 0.0f; out[1] = 0.0f; return 2; }
    out[0] = q / a2;
    out[1] = a0 / q;
    return 2;
}

// One piece's component polynomials, v(t) = a + b t + c t^2.
struct Coeffs { float ax; float bx; float cx; float ay; float by; float cy; };

// `c` is formed as (v2 - v1) - (v1 - v0), not v0 - 2 v1 + v2: algebraically the
// same, numerically not, and G0-8 measured the difference at f32.
static Coeffs coeffs_of(device const float *pieces, uint index) {
    uint b = index * PIECE_STRIDE;
    float x0 = pieces[b + 0], y0 = pieces[b + 1];
    float x1 = pieces[b + 2], y1 = pieces[b + 3];
    float x2 = pieces[b + 4], y2 = pieces[b + 5];
    Coeffs c;
    c.ax = x0; c.bx = 2.0f * (x1 - x0); c.cx = (x2 - x1) - (x1 - x0);
    c.ay = y0; c.by = 2.0f * (y1 - y0); c.cy = (y2 - y1) - (y1 - y0);
    return c;
}

static float eval_x(Coeffs c, float t) { return c.ax + c.bx * t + c.cx * t * t; }
static float eval_y(Coeffs c, float t) { return c.ay + c.by * t + c.cy * t * t; }

// Invert a component that is monotone on [t_lo, t_hi]. Total by construction:
// out-of-range targets clamp to an endpoint, and a root solve that finds nothing
// in range falls back to the secant.
static float invert(float a, float b, float c, float target,
                    float t_lo, float t_hi, float v_lo, float v_hi) {
    bool ascending = v_hi >= v_lo;
    float vmin = fmin_(v_lo, v_hi);
    float vmax = fmax_(v_lo, v_hi);
    if (target <= vmin) return ascending ? t_lo : t_hi;
    if (target >= vmax) return ascending ? t_hi : t_lo;

    float roots[2];
    int n = solve_quadratic(c, b, a - target, roots);
    for (int i = 0; i < n; i++) {
        if (roots[i] >= t_lo && roots[i] <= t_hi) return roots[i];
    }
    return t_lo + (t_hi - t_lo) * ((target - v_lo) / (v_hi - v_lo));
}

static float t_at_x(Coeffs c, float target, float t_lo, float t_hi) {
    return invert(c.ax, c.bx, c.cx, target, t_lo, t_hi, eval_x(c, t_lo), eval_x(c, t_hi));
}

static float t_at_y(Coeffs c, float target, float t_lo, float t_hi) {
    return invert(c.ay, c.by, c.cy, target, t_lo, t_hi, eval_y(c, t_lo), eval_y(c, t_hi));
}

// Deposit one sub-span's signed trapezoid. `cells` is (x_hi - x_lo + 1) wide;
// the extra entry catches the spill from the last in-tile cell and is never
// read, which keeps this branch-free at the tile's right edge.
static void deposit(thread float *cells, thread float *carry,
                    uint x_lo, uint x_hi,
                    float x_a, float x_b, float y_a, float y_b) {
    float d = y_b - y_a;
    if (d == 0.0f) return;
    float xm = 0.5f * (x_a + x_b);
    float cell_f = floor(xm);
    int cell = (int)cell_f;
    if (cell < (int)x_lo) { *carry = *carry + d; return; }
    if (cell >= (int)x_hi) return;
    float xmf = xm - cell_f;
    int i = cell - (int)x_lo;
    cells[i] = cells[i] + d * (1.0f - xmf);
    cells[i + 1] = cells[i + 1] + d * xmf;
}

// The whole algorithm. Both kernels are built from this: the scanline shape
// calls it with the tile's full width, the per-pixel shape with a one-cell
// window.
static void accumulate_piece_row(Coeffs c, uint row_y, uint x_lo, uint x_hi,
                                 thread float *cells, thread float *carry) {
    float row = (float)row_y;
    float row_end = row + 1.0f;

    // The piece's y-extent, and the part of it inside this scanline band.
    // Monotone in y, so the endpoints are the extremes.
    float y0 = eval_y(c, 0.0f);
    float y1 = eval_y(c, 1.0f);
    float band_lo = fmax_(fmin_(y0, y1), row);
    float band_hi = fmin_(fmax_(y0, y1), row_end);
    if (band_hi <= band_lo) return;
    float u = t_at_y(c, band_lo, 0.0f, 1.0f);
    float v = t_at_y(c, band_hi, 0.0f, 1.0f);
    float ta = fmin_(u, v);
    float tb = fmax_(u, v);
    if (tb <= ta) return;

    float xa = eval_x(c, ta);
    float xb = eval_x(c, tb);
    bool increasing = xb >= xa;
    float left = (float)x_lo;
    float right = (float)x_hi;

    float t_left = t_at_x(c, left, ta, tb);
    float t_right = t_at_x(c, right, ta, tb);

    // Everything left of the tile contributes its full signed dy to the row's
    // carry — in one subtraction, never by walking cells from the frame edge.
    float l0 = increasing ? ta : t_left;
    float l1 = increasing ? t_left : tb;
    if (l1 > l0) *carry = *carry + (eval_y(c, l1) - eval_y(c, l0));

    float t_prev = increasing ? t_left : t_right;
    float t_end  = increasing ? t_right : t_left;
    if (t_end <= t_prev) return;
    // Clamped into the tile, and load-bearing rather than tidy: the walk's span
    // is by construction inside the tile, but x(t_prev) is a root solve's answer
    // re-evaluated and lands an ulp outside about half the time. Unclamped, an
    // entry at exactly the right edge makes `ceil(x) - 1` name the column the
    // walk is already in, the step stalls, and the fallback deposits a
    // three-column span as one trapezoid. See analytic_fill.rs for the incident.
    float x_prev = clamp(eval_x(c, t_prev), left, right);
    float y_prev = eval_y(c, t_prev);
    float x_end = clamp(eval_x(c, t_end), left, right);
    float y_end = eval_y(c, t_end);

    uint steps = (x_hi - x_lo) + 2u;
    for (uint step = 0; step < steps; step++) {
        float boundary = increasing ? (floor(x_prev) + 1.0f) : (ceil(x_prev) - 1.0f);
        bool past = increasing ? (boundary >= x_end) : (boundary <= x_end);
        float t_next, x_next;
        if (past) {
            t_next = t_end;
            x_next = x_end;
        } else {
            // The crossing's x IS the boundary by construction; using the exact
            // integer rather than re-evaluating x(t) keeps the cell index
            // unambiguous when the root solve lands an ulp to one side.
            t_next = t_at_x(c, boundary, t_prev, t_end);
            x_next = boundary;
        }
        if (t_next <= t_prev && !past) {
            // The closed form could not separate two adjacent column
            // boundaries. The secant in x is monotone and advances unless the
            // span is degenerate; advancing matters more than the parameter's
            // exactness, because a multi-column span deposited as one trapezoid
            // is the failure this guard exists to prevent.
            float denom = x_end - x_prev;
            if (denom != 0.0f) {
                t_next = t_prev + (t_end - t_prev) * ((boundary - x_prev) / denom);
            }
        }
        if (t_next <= t_prev) { t_next = t_end; x_next = x_end; }
        float y_next = (t_next >= t_end) ? y_end : eval_y(c, t_next);
        deposit(cells, carry, x_lo, x_hi, x_prev, x_next, y_prev, y_next);
        if (t_next >= t_end) break;
        t_prev = t_next;
        x_prev = x_next;
        y_prev = y_next;
    }
}

// Nonzero-winding coverage of one path at one pixel, without an accumulator.
// The same terms as the scanline form in a different association.
static float coverage_at_cell(device const float *pieces, uint first, uint count,
                              uint row_y, uint cell) {
    float acc = 0.0f;
    for (uint i = first; i < first + count; i++) {
        float window[2] = {0.0f, 0.0f};
        float carry = 0.0f;
        accumulate_piece_row(coeffs_of(pieces, i), row_y, cell, cell + 1u, window, &carry);
        acc = acc + carry + window[0];
    }
    float a = fabs(acc);
    return (a > 1.0f) ? 1.0f : a;
}

// The fill's colour at a screen point: projection onto the gradient axis,
// clamped, then a linear-light lerp. Mirrors fill::gradient_at.
static float4 gradient_at(float4 rgba, float4 rgba_end, float4 axis, float2 p) {
    float dx = axis.z - axis.x;
    float dy = axis.w - axis.y;
    float denom = dx * dx + dy * dy;
    float t = 0.0f;
    if (denom > 0.0f) {
        t = clamp(((p.x - axis.x) * dx + (p.y - axis.y) * dy) / denom, 0.0f, 1.0f);
    }
    return rgba + (rgba_end - rgba) * t;
}

// Source-over in linear light, straight alpha. Mirrors sdf::over.
static float4 src_over(float4 src, float4 dst) {
    float sa = src.a;
    if (sa <= 0.0f) return dst;
    float out_a = sa + dst.a * (1.0f - sa);
    if (out_a <= 0.0f) return float4(0.0f);
    float3 rgb = (src.rgb * sa + dst.rgb * dst.a * (1.0f - sa)) / out_a;
    return float4(rgb, out_a);
}

// -------------------------------------------------------- the scanline kernel

kernel void fill_scanline(
    constant uint      *params_u32   [[buffer(0)]],  // width, height, tile, cols
    constant float     *params_f32   [[buffer(1)]],  // background rgba
    device const float *pieces       [[buffer(2)]],
    device const uint  *path_u32     [[buffer(3)]],
    device const float *path_f32     [[buffer(4)]],
    device const float *styles       [[buffer(5)]],
    device const uint  *tile_offsets [[buffer(6)]],
    device const uint  *tile_draws   [[buffer(7)]],
    device const uint  *tile_flags   [[buffer(8)]],
    device float       *surface      [[buffer(9)]],
    uint2 group_id [[threadgroup_position_in_grid]],
    uint2 local_id [[thread_position_in_threadgroup]])
{
    const uint width  = params_u32[0];
    const uint height = params_u32[1];
    const uint tile   = params_u32[2];
    const uint cols   = params_u32[3];

    const uint py = group_id.y * tile + local_id.y;
    if (py >= height) return;
    const uint x_lo = group_id.x * tile;
    if (x_lo >= width) return;
    const uint x_hi = min(x_lo + tile, width);
    const uint w = x_hi - x_lo;

    const float4 background = float4(params_f32[0], params_f32[1],
                                     params_f32[2], params_f32[3]);
    float4 acc[MAX_TILE];
    for (uint i = 0; i < w; i++) acc[i] = background;

    const uint tile_index = group_id.y * cols + group_id.x;
    const uint lo = tile_offsets[tile_index];
    const uint hi = tile_offsets[tile_index + 1];

    float cells[MAX_TILE + 1];

    for (uint k = lo; k < hi; k++) {
        const uint path = tile_draws[k];
        const uint pu = path * FILL_PATH_U32_STRIDE;
        const uint pf = path * FILL_PATH_F32_STRIDE;

        // The conservative slab, as a row reject: a path whose slab misses this
        // scanline cannot touch any pixel of it.
        const float4 slab = float4(path_f32[pf], path_f32[pf + 1],
                                   path_f32[pf + 2], path_f32[pf + 3]);
        if (slab.w <= (float)py || slab.y >= (float)py + 1.0f) continue;

        const uint first = path_u32[pu];
        const uint count = path_u32[pu + 1];
        const uint style = path_u32[pu + 2];
        const uint st = style * FILL_STYLE_STRIDE;
        const float4 rgba = float4(styles[st+0], styles[st+1], styles[st+2], styles[st+3]);
        const float4 rgba_end = float4(styles[st+4], styles[st+5], styles[st+6], styles[st+7]);
        const float4 axis = float4(styles[st+8], styles[st+9], styles[st+10], styles[st+11]);

        // §10.4's interior class: the whole tile is inside the path, so coverage
        // is exactly one and the accumulation is skipped entirely.
        const bool interior = tile_flags[k] == CLASS_INTERIOR;

        if (!interior) {
            for (uint i = 0; i <= w; i++) cells[i] = 0.0f;
            float carry = 0.0f;
            for (uint i = first; i < first + count; i++) {
                accumulate_piece_row(coeffs_of(pieces, i), py, x_lo, x_hi, cells, &carry);
            }
            float running = carry;
            for (uint i = 0; i < w; i++) {
                running = running + cells[i];
                float a = fabs(running);
                cells[i] = (a > 1.0f) ? 1.0f : a;   // reuse: coverage overwrites the cell
            }
        }

        for (uint i = 0; i < w; i++) {
            float cov = interior ? 1.0f : cells[i];
            if (cov <= 0.0f) continue;
            float2 p = float2((float)(x_lo + i) + 0.5f, (float)py + 0.5f);
            if (p.x < slab.x || p.x > slab.z) continue;
            float4 colour = gradient_at(rgba, rgba_end, axis, p);
            acc[i] = src_over(float4(colour.rgb, colour.a * cov), acc[i]);
        }
    }

    for (uint i = 0; i < w; i++) {
        const uint o = (py * width + (x_lo + i)) * 4;
        surface[o + 0] = acc[i].r;
        surface[o + 1] = acc[i].g;
        surface[o + 2] = acc[i].b;
        surface[o + 3] = acc[i].a;
    }
}

// ------------------------------------------------------- the per-pixel kernel

kernel void fill_pixel(
    constant uint      *params_u32   [[buffer(0)]],
    constant float     *params_f32   [[buffer(1)]],
    device const float *pieces       [[buffer(2)]],
    device const uint  *path_u32     [[buffer(3)]],
    device const float *path_f32     [[buffer(4)]],
    device const float *styles       [[buffer(5)]],
    device const uint  *tile_offsets [[buffer(6)]],
    device const uint  *tile_draws   [[buffer(7)]],
    device const uint  *tile_flags   [[buffer(8)]],
    device float       *surface      [[buffer(9)]],
    uint2 group_id [[threadgroup_position_in_grid]],
    uint2 local_id [[thread_position_in_threadgroup]])
{
    const uint width  = params_u32[0];
    const uint height = params_u32[1];
    const uint tile   = params_u32[2];
    const uint cols   = params_u32[3];

    const uint px = group_id.x * tile + local_id.x;
    const uint py = group_id.y * tile + local_id.y;
    if (px >= width || py >= height) return;

    const uint tile_index = group_id.y * cols + group_id.x;
    const uint lo = tile_offsets[tile_index];
    const uint hi = tile_offsets[tile_index + 1];

    float4 acc = float4(params_f32[0], params_f32[1], params_f32[2], params_f32[3]);
    const float2 p = float2((float)px + 0.5f, (float)py + 0.5f);

    for (uint k = lo; k < hi; k++) {
        const uint path = tile_draws[k];
        const uint pu = path * FILL_PATH_U32_STRIDE;
        const uint pf = path * FILL_PATH_F32_STRIDE;

        const float4 slab = float4(path_f32[pf], path_f32[pf + 1],
                                   path_f32[pf + 2], path_f32[pf + 3]);
        if (p.x < slab.x || p.x > slab.z || p.y < slab.y || p.y > slab.w) continue;

        const uint first = path_u32[pu];
        const uint count = path_u32[pu + 1];
        const uint style = path_u32[pu + 2];

        const float cov = (tile_flags[k] == CLASS_INTERIOR)
                        ? 1.0f
                        : coverage_at_cell(pieces, first, count, py, px);
        if (cov <= 0.0f) continue;

        const uint st = style * FILL_STYLE_STRIDE;
        const float4 rgba = float4(styles[st+0], styles[st+1], styles[st+2], styles[st+3]);
        const float4 rgba_end = float4(styles[st+4], styles[st+5], styles[st+6], styles[st+7]);
        const float4 axis = float4(styles[st+8], styles[st+9], styles[st+10], styles[st+11]);
        const float4 colour = gradient_at(rgba, rgba_end, axis, p);
        acc = src_over(float4(colour.rgb, colour.a * cov), acc);
    }

    const uint o = (py * width + px) * 4;
    surface[o + 0] = acc.r;
    surface[o + 1] = acc.g;
    surface[o + 2] = acc.b;
    surface[o + 3] = acc.a;
}
