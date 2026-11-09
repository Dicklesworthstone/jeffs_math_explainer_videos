// The stroke-SDF + AA-resolve stage of the compiled render IR (§10.8),
// expressed as one Metal compute kernel — G0-8's proof that the IR maps onto a
// GPU before W5 freezes it (fm-ekx, plan §20.1 spike 8).
//
// THE MIRROR RULE. Every mathematical statement below has a twin in
// `src/sdf.rs` and `src/cpu.rs`, in the same order, with the same branches.
// Exactly two differences are intended, and both are named where they occur:
// scalar width (f64 there, f32 here), and `cbrt_f` standing in for MSL's
// missing `cbrt`. That is what makes the measured CPU-vs-Metal divergence a
// *precision* budget (§16.3) rather than an unexplained pile of pixels.
//
// THE DISPATCH SHAPE. One threadgroup per tile, one thread per pixel. Each
// thread walks its tile's command run in painter order, accumulating into a
// register-resident RGBA — the tile-local compositing Apple's TBDR
// architecture rewards (§17.6). Nothing is written to device memory until the
// tile is finished, so the surface takes exactly one store per pixel and no
// intermediate ever makes a second trip through memory.
//
// WHY THERE ARE NO ATOMICS. §10.8 forbids "unordered atomic appends" on the
// annex, and there is nothing here to append: binning happened on the host by
// deterministic count/prefix/scatter, and this kernel only *reads* the CSR
// command lists in index order. Painter order is therefore structural, not
// enforced.

#include <metal_stdlib>
using namespace metal;

// Strides — mirrored from `ir.rs`'s SEGMENT_STRIDE / PATH_*_STRIDE /
// STYLE_STRIDE consts. A mismatch would be silent garbage, so the host asserts
// them against these names in `annex.rs`.
#define SEGMENT_STRIDE 8
#define PATH_U32_STRIDE 4
#define PATH_F32_STRIDE 4
#define STYLE_STRIDE 8

#define HINT_GENERAL 0u
#define HINT_LINE 1u

// The degeneracy threshold, RELATIVE to the polynomial's own scale and sized to
// THIS engine's precision — a few f32 epsilons, against the CPU's few f64
// epsilons. The two constants differ on purpose: an absolute constant shared
// between an f64 engine and an f32 engine is not a shared semantics, it is a
// bug only one of them can see. See sdf.rs's DEGENERATE_REL for the incident.
#define DEGENERATE_REL 1e-6f
#define MIN_AA_WIDTH 1e-8f

// +1 for zero and positives, -1 for negatives. NOT MSL's `sign()`, which
// returns 0 at zero where Rust's `signum` returns 1 — a disagreement that
// silently collapsed the stable-quadratic pairing to "both roots are zero" on
// the GPU alone.
static float sign_or_positive(float x) {
    return (x >= 0.0f) ? 1.0f : -1.0f;
}

// ---------------------------------------------------------------- root finding

// MSL has no `cbrt`. THE SECOND NAMED MIRROR-RULE DIVERGENCE (the first is
// scalar width): the CPU routes cube roots through `fmn_dmath::cbrt`, whose
// documented bound is < 0.667 ulp, while this is `sign(x) * pow(|x|, 1/3)` at
// f32 with whatever `pow` the Metal compiler emits. It is exact at 0 and
// odd-symmetric, which is all the cubic solver's Cardano branch needs, and the
// resulting divergence is part of what the equivalence measurement reports —
// not something hidden behind an equality that was never true.
static float cbrt_f(float x) {
    return sign(x) * pow(fabs(x), 1.0f / 3.0f);
}

// Real roots of a2 t^2 + a1 t + a0. Mirrors sdf::solve_quadratic.
static int solve_quadratic(float a2, float a1, float a0, thread float *out) {
    float scale = max(max(fabs(a2), fabs(a1)), fabs(a0));
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
    if (q == 0.0f) {
        out[0] = 0.0f;
        out[1] = 0.0f;
        return 2;
    }
    out[0] = q / a2;
    out[1] = a0 / q;
    return 2;
}

// Real roots of a3 t^3 + a2 t^2 + a1 t + a0. Mirrors sdf::solve_cubic,
// including its degenerate fall-through order.
static int solve_cubic(float a3, float a2, float a1, float a0, thread float *out) {
    float scale = max(max(fabs(a3), fabs(a2)), max(fabs(a1), fabs(a0)));
    if (scale <= 0.0f) return 0;
    float tol = DEGENERATE_REL * scale;
    if (fabs(a3) <= tol) return solve_quadratic(a2, a1, a0, out);

    float b = a2 / a3;
    float c = a1 / a3;
    float d = a0 / a3;
    float shift = b / 3.0f;
    float p = c - b * b / 3.0f;
    float q = 2.0f * b * b * b / 27.0f - b * c / 3.0f + d;

    // The depressed cubic's own scale, not the original polynomial's.
    if (fabs(p) <= DEGENERATE_REL * max(max(fabs(p), fabs(q)), 1.0f)) {
        out[0] = cbrt_f(-q) - shift;
        return 1;
    }

    // The discriminant is a difference of two computed quantities; testing it
    // against exact zero tests the sign of the cancellation error. See sdf.rs.
    float e1 = q * q / 4.0f;
    float e2 = p * p * p / 27.0f;
    float disc = e1 + e2;
    if (fabs(disc) <= sqrt(DEGENERATE_REL) * (fabs(e1) + fabs(e2))) {
        out[0] = 3.0f * q / p - shift;
        out[1] = -1.5f * q / p - shift;
        return 2;
    }
    if (disc > 0.0f) {
        float s = sqrt(disc);
        out[0] = cbrt_f(-q / 2.0f + s) + cbrt_f(-q / 2.0f - s) - shift;
        return 1;
    }
    float m = 2.0f * sqrt(-p / 3.0f);
    float arg = clamp((3.0f * q) / (p * m), -1.0f, 1.0f);
    float phi = acos(arg) / 3.0f;
    const float THIRD_TURN = 2.0943951023931953f; // 2*pi/3
    out[0] = m * cos(phi) - shift;
    out[1] = m * cos(phi - THIRD_TURN) - shift;
    out[2] = m * cos(phi - 2.0f * THIRD_TURN) - shift;
    return 3;
}

// ------------------------------------------------------------------- distance

// Distance from p to the quadratic (p0,p1,p2) and the t where it is attained.
// Mirrors sdf::distance_to_quadratic, endpoints-first so the candidate
// comparison order matches the CPU exactly.
static float2 distance_to_quadratic(float2 p, float2 p0, float2 p1, float2 p2) {
    float2 a = p0 - p;
    float2 b = 2.0f * (p1 - p0);
    // (p2 - p1) - (p1 - p0), not p0 - 2p1 + p2: algebraically identical,
    // numerically not close. See sdf.rs for why this matters at f32.
    float2 c = (p2 - p1) - (p1 - p0);

    float cc = dot(c, c);
    float bc = dot(b, c);
    float bb = dot(b, b);
    float ac = dot(a, c);
    float ab = dot(a, b);

    float roots[3];
    int n = solve_cubic(2.0f * cc, 3.0f * bc, bb + 2.0f * ac, ab, roots);

    float best_t = 0.0f;
    float best_d2 = dot(a, a);
    float2 e1 = a + b + c;
    float d1 = dot(e1, e1);
    if (d1 < best_d2) { best_d2 = d1; best_t = 1.0f; }

    for (int i = 0; i < n; i++) {
        float t = roots[i];
        if (t < 0.0f || t > 1.0f) continue;
        float2 v = a + b * t + c * t * t;
        float d2 = dot(v, v);
        if (d2 < best_d2) { best_d2 = d2; best_t = t; }
    }
    return float2(sqrt(best_d2), best_t);
}

// The PrimitiveHint::Line fast path. Mirrors cpu::distance_to_capsule.
static float2 distance_to_capsule(float2 p, float2 a, float2 b) {
    float2 ab = b - a;
    float2 ap = p - a;
    float denom = dot(ab, ab);
    float t = (denom <= 0.0f) ? 0.0f : clamp(dot(ap, ab) / denom, 0.0f, 1.0f);
    float2 d = ap - ab * t;
    return float2(length(d), t);
}

// The Reference's kept AA profile: smoothstep(0.5, -0.5, (d - w/2) / aaw).
// Mirrors sdf::coverage. Written open rather than via Metal's smoothstep()
// so the descending-edge form is visibly the same expression as the CPU's.
static float coverage(float distance, float half_width, float aa_width) {
    float aaw = max(aa_width, MIN_AA_WIDTH);
    float signed_dist = (distance - half_width) / aaw;
    float s = clamp(0.5f - signed_dist, 0.0f, 1.0f);
    return s * s * (3.0f - 2.0f * s);
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

// ---------------------------------------------------------------- the kernel

kernel void stroke_aa_resolve(
    constant uint   *params_u32   [[buffer(0)]],  // width, height, tile, cols
    constant float  *params_f32   [[buffer(1)]],  // background rgba
    device const float *segments  [[buffer(2)]],
    device const uint  *path_u32  [[buffer(3)]],
    device const float *path_f32  [[buffer(4)]],
    device const float *styles    [[buffer(5)]],
    device const uint  *tile_offsets [[buffer(6)]],
    device const uint  *tile_draws   [[buffer(7)]],
    device float       *surface      [[buffer(8)]],
    uint2 group_id  [[threadgroup_position_in_grid]],
    uint2 local_id  [[thread_position_in_threadgroup]])
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
    // Pixel centre — the CPU uses (px + 0.5, py + 0.5) and a half-pixel
    // disagreement here would read as a systematic AA-band shift, which is
    // exactly the class of bug this spike exists to catch early.
    const float2 p = float2((float)px + 0.5f, (float)py + 0.5f);

    for (uint k = lo; k < hi; k++) {
        const uint path = tile_draws[k];
        const uint pu = path * PATH_U32_STRIDE;
        const uint pf = path * PATH_F32_STRIDE;

        // The conservative slab: the per-pixel early-out. Most pixels of most
        // tiles leave here, which is why the slab lives in the IR rather than
        // being recomputed.
        const float4 slab = float4(path_f32[pf], path_f32[pf + 1],
                                   path_f32[pf + 2], path_f32[pf + 3]);
        if (p.x < slab.x || p.x > slab.z || p.y < slab.y || p.y > slab.w) continue;

        const uint first = path_u32[pu];
        const uint count = path_u32[pu + 1];
        const uint style = path_u32[pu + 2];
        const uint hint  = path_u32[pu + 3];

        float best_d = INFINITY;
        float best_s = 0.0f;
        for (uint i = first; i < first + count; i++) {
            const uint s = i * SEGMENT_STRIDE;
            const float2 p0 = float2(segments[s + 0], segments[s + 1]);
            const float2 p1 = float2(segments[s + 2], segments[s + 3]);
            const float2 p2 = float2(segments[s + 4], segments[s + 5]);
            const float s0 = segments[s + 6];
            const float s1 = segments[s + 7];

            float2 dt = (hint == HINT_LINE) ? distance_to_capsule(p, p0, p2)
                                            : distance_to_quadratic(p, p0, p1, p2);
            if (dt.x < best_d) {
                best_d = dt.x;
                best_s = s0 + (s1 - s0) * dt.y;
            }
        }

        const uint st = style * STYLE_STRIDE;
        const float4 rgba = float4(styles[st + 0], styles[st + 1],
                                   styles[st + 2], styles[st + 3]);
        const float w0 = styles[st + 4];
        const float w1 = styles[st + 5];
        const float aaw = styles[st + 6];

        const float w = w0 + (w1 - w0) * best_s;
        const float alpha = coverage(best_d, 0.5f * w, aaw);
        if (alpha <= 0.0f) continue;

        acc = src_over(float4(rgba.rgb, rgba.a * alpha), acc);
    }

    // One store per pixel, at the end of the tile's whole command run.
    //
    // The surface stays linear-light f32 rather than being quantized here, for
    // two reasons: the divergence against the CPU is measured in the space the
    // compositing actually happened in, and the sRGB encode remains a single
    // host-side code path shared by both engines — so a colour difference in
    // the PNG can only have come from the kernel.
    const uint o = (py * width + px) * 4;
    surface[o + 0] = acc.r;
    surface[o + 1] = acc.g;
    surface[o + 2] = acc.b;
    surface[o + 3] = acc.a;
}
