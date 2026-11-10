//! The stage's mathematics, written once here in `f64` and mirrored
//! statement-for-statement in `shaders/stroke_aa.metal` in `f32`.
//!
//! Keeping the two readable side by side is the whole point of the spike: an
//! engine-equivalence budget (§16.3) is only meaningful if the divergence
//! being measured is *precision*, not *algorithm*. Every difference between
//! this module and the shader is therefore deliberate and named in the mapping
//! report — today there is exactly one, the scalar width (`f64` here, `f32`
//! there).
//!
//! Three pieces:
//!
//! 1. [`distance_to_quadratic`] — the true distance from a point to a
//!    quadratic Bézier, by closed-form cubic root-finding. This is §10.3's
//!    "exact/high-accuracy signed distance to the quadratic", and it is what
//!    replaces the Reference's ≤32-segment polyline ribbon.
//! 2. [`coverage`] — the Reference's AA profile, kept verbatim (§1.5's kept
//!    look constants, Appendix B): `smoothstep(0.5, -0.5, (d - w/2) / aaw)`
//!    with `anti_alias_width = 1.5`.
//! 3. [`over`] — source-over compositing in linear light, straight alpha.

/// The Reference's `anti_alias_width` default
/// (`manimlib/mobject/types/vectorized_mobject.py:96`), in pixels. Kept, not
/// re-derived: §1.5 lists the AA feel among the constants this program ports
/// deliberately.
pub const ANTI_ALIAS_WIDTH_PX: f64 = 1.5;

/// Below this the AA band is treated as a hard edge rather than divided by.
/// Mirrors the Reference's own `max(anti_alias_width * pixel_size, 1e-8)`.
pub const MIN_AA_WIDTH: f64 = 1e-8;

/// The degeneracy threshold, **relative to the polynomial's own scale** and
/// **sized to this engine's precision** (a few f64 epsilons; the shader's twin
/// is a few f32 epsilons).
///
/// It was an absolute `1e-12` in the spike's first draft, and that was wrong in
/// a way worth recording. Screen-space coordinates run to ~10³, so the cubic's
/// leading coefficient for a *nearly* straight curve lands around `1e-8` in f32
/// purely from cancellation — comfortably above an absolute `1e-12`, so the
/// shader entered the genuine-cubic branch, divided by that noise, and produced
/// garbage roots on the Metal engine while the f64 CPU sailed through. It cost
/// 235 visibly-wrong components out of 2 million on the first end-to-end run.
///
/// **The finding for W5:** a root solver's degeneracy test must be relative to
/// coefficient scale and sized per engine precision. A single shared absolute
/// constant is not a shared semantics — it is a bug that only one of the two
/// engines can see.
const DEGENERATE_REL: f64 = 1e-14;

/// `+1` for zero and positives, `-1` for negatives.
///
/// Deliberately not `f64::signum` and deliberately not MSL's `sign()`: they
/// disagree at exactly zero (`signum(0.0) == 1.0`, `sign(0.0f) == 0.0f`), and
/// that disagreement silently rewrote the stable-quadratic pairing below into
/// "both roots are zero" on the GPU alone. Writing the predicate out in both
/// languages is the only way the mirror rule survives contact with two standard
/// libraries.
fn sign_or_positive(x: f64) -> f64 {
    if x >= 0.0 { 1.0 } else { -1.0 }
}

/// Real roots of `a3 t³ + a2 t² + a1 t + a0`, written into `out`, returning how
/// many were found.
///
/// Degenerate leading coefficients fall through to the quadratic and then the
/// linear case rather than dividing by something near zero — which is not
/// pedantry here: a quadratic Bézier whose control points are collinear (every
/// straight line in the corpus, every glyph stem) has an exactly-zero cubic
/// term, and a curve that is *nearly* straight has a tiny one.
pub fn solve_cubic(a3: f64, a2: f64, a1: f64, a0: f64, out: &mut [f64; 3]) -> usize {
    let scale = a3.abs().max(a2.abs()).max(a1.abs()).max(a0.abs());
    if scale <= 0.0 {
        return 0;
    }
    let tol = DEGENERATE_REL * scale;
    if a3.abs() <= tol {
        return solve_quadratic(a2, a1, a0, out);
    }
    // Monic, then depressed: t = x - b/3 gives x³ + p x + q.
    let b = a2 / a3;
    let c = a1 / a3;
    let d = a0 / a3;
    let shift = b / 3.0;
    let p = c - b * b / 3.0;
    let q = 2.0 * b * b * b / 27.0 - b * c / 3.0 + d;

    // The depressed cubic's own scale, not the original polynomial's: `p` and
    // `q` are post-normalization quantities and comparing them against `tol`
    // would compare unlike things.
    if p.abs() <= DEGENERATE_REL * p.abs().max(q.abs()).max(1.0) {
        // x³ = -q.
        out[0] = cbrt(-q) - shift;
        return 1;
    }

    // The discriminant is a *difference of two computed quantities*, so testing
    // it against exact zero tests the sign of the cancellation error, not the
    // sign of the discriminant. Near a multiple root the two terms very nearly
    // annihilate, and picking the one-real-root branch there loses the root
    // that mattered — which is what the annex did, at a stroke's curvature
    // extremum, turning a fully-covered pixel into background.
    let e1 = q * q / 4.0;
    let e2 = p * p * p / 27.0;
    let disc = e1 + e2;
    if disc.abs() <= DEGENERATE_REL.sqrt() * (e1.abs() + e2.abs()) {
        // A (near-)multiple root. For `x³ + px + q` with zero discriminant the
        // roots are `3q/p` and the double root `-3q/(2p)`, which is stable in
        // exactly the regime the general formulae are not.
        out[0] = 3.0 * q / p - shift;
        out[1] = -1.5 * q / p - shift;
        return 2;
    }
    if disc > 0.0 {
        // One real root (Cardano).
        let s = disc.sqrt();
        out[0] = cbrt(-q / 2.0 + s) + cbrt(-q / 2.0 - s) - shift;
        1
    } else {
        // Three real roots (the trigonometric form; `p < 0` is implied here).
        let m = 2.0 * (-p / 3.0).sqrt();
        let arg = (3.0 * q) / (p * m);
        let phi = acos(arg.clamp(-1.0, 1.0)) / 3.0;
        let third_turn = std::f64::consts::TAU / 3.0;
        out[0] = m * cos(phi) - shift;
        out[1] = m * cos(phi - third_turn) - shift;
        out[2] = m * cos(phi - 2.0 * third_turn) - shift;
        3
    }
}

/// Real roots of `a2 t² + a1 t + a0`, falling through to the linear case.
pub fn solve_quadratic(a2: f64, a1: f64, a0: f64, out: &mut [f64; 3]) -> usize {
    let scale = a2.abs().max(a1.abs()).max(a0.abs());
    if scale <= 0.0 {
        return 0;
    }
    let tol = DEGENERATE_REL * scale;
    if a2.abs() <= tol {
        if a1.abs() <= tol {
            return 0;
        }
        out[0] = -a0 / a1;
        return 1;
    }
    let disc = a1 * a1 - 4.0 * a2 * a0;
    if disc < 0.0 {
        return 0;
    }
    let s = disc.sqrt();
    // The numerically stable pairing: form the root that does not cancel, then
    // get the other from the product of roots.
    let q = -0.5 * (a1 + sign_or_positive(a1) * s);
    if q == 0.0 {
        // Reachable only when `a1` and the discriminant are both zero, i.e. the
        // double root at the origin.
        out[0] = 0.0;
        out[1] = 0.0;
        return 2;
    }
    out[0] = q / a2;
    out[1] = a0 / q;
    2
}

/// Distance from `p` to the quadratic Bézier `(p0, p1, p2)`, and the parameter
/// `t ∈ [0, 1]` where it is attained.
///
/// `B(t) - p = A + B t + C t²` with `A = p0 - p`, `B = 2(p1 - p0)`,
/// `C = p0 - 2p1 + p2`, so `d/dt |B(t) - p|² / 2` is the cubic
/// `2(C·C) t³ + 3(B·C) t² + (B·B + 2 A·C) t + (A·B)`. Its real roots inside
/// `[0, 1]`, plus both endpoints, are the complete candidate set: endpoints are
/// always included, which is what makes a degenerate "curve" (a point, or a
/// zero-length segment) return the right answer with no special case, and what
/// gives open ends their **round caps** for free.
pub fn distance_to_quadratic(p: [f64; 2], p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> (f64, f64) {
    let a = [p0[0] - p[0], p0[1] - p[1]];
    let b = [2.0 * (p1[0] - p0[0]), 2.0 * (p1[1] - p0[1])];
    // `(p2 − p1) − (p1 − p0)`, not `p0 − 2p1 + p2`. Algebraically identical;
    // numerically not close. The second form subtracts quantities of order
    // 10³ (screen coordinates) to reach a result of order 10⁻³ on a
    // near-straight curve, losing every significant digit in f32. The first
    // form differences neighbouring points first — each difference is small
    // and well-conditioned — and only then differences the differences.
    let c = [
        (p2[0] - p1[0]) - (p1[0] - p0[0]),
        (p2[1] - p1[1]) - (p1[1] - p0[1]),
    ];

    let cc = c[0] * c[0] + c[1] * c[1];
    let bc = b[0] * c[0] + b[1] * c[1];
    let bb = b[0] * b[0] + b[1] * b[1];
    let ac = a[0] * c[0] + a[1] * c[1];
    let ab = a[0] * b[0] + a[1] * b[1];

    let mut roots = [0.0f64; 3];
    let n = solve_cubic(2.0 * cc, 3.0 * bc, bb + 2.0 * ac, ab, &mut roots);

    // Endpoints first, so a curve with no interior critical point still gets an
    // answer, and so the comparison order is identical on both engines.
    let mut best_t = 0.0;
    let mut best_d2 = a[0] * a[0] + a[1] * a[1];
    let e1 = [a[0] + b[0] + c[0], a[1] + b[1] + c[1]];
    let d1 = e1[0] * e1[0] + e1[1] * e1[1];
    if d1 < best_d2 {
        best_d2 = d1;
        best_t = 1.0;
    }
    for &t in roots.iter().take(n) {
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let v = [
            a[0] + b[0] * t + c[0] * t * t,
            a[1] + b[1] * t + c[1] * t * t,
        ];
        let d2 = v[0] * v[0] + v[1] * v[1];
        if d2 < best_d2 {
            best_d2 = d2;
            best_t = t;
        }
    }
    (best_d2.sqrt(), best_t)
}

/// [`distance_to_quadratic`] transcribed to `f32`: the annex's arithmetic,
/// evaluated on the CPU.
///
/// This is not a diagnostic afterthought — it is how the annex's *numerical*
/// behaviour becomes testable without a GPU. The engine-equivalence question
/// "how far can the Metal engine land from the reference, and where?" splits
/// cleanly into two: what does f32 arithmetic do to this algorithm (answerable
/// here, on any machine, in CI), and what does the Metal compiler do on top of
/// that (answerable only on Apple silicon). Keeping the first half on the CPU
/// means a W5 change that wrecks the annex's conditioning fails on a Linux
/// runner instead of surviving to the one machine that has a GPU.
///
/// It is a literal transcription — same branches, same order, same relative
/// thresholds scaled to f32 — so it is also the readable statement of what the
/// shader does.
pub fn distance_to_quadratic_f32(
    p: [f32; 2],
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
) -> (f32, f32) {
    /// The f32 twin of [`DEGENERATE_REL`], and of the shader's own constant.
    const REL: f32 = 1e-6;

    fn sign_or_positive(x: f32) -> f32 {
        if x >= 0.0 { 1.0 } else { -1.0 }
    }

    fn solve_quadratic(a2: f32, a1: f32, a0: f32, out: &mut [f32; 3]) -> usize {
        let scale = a2.abs().max(a1.abs()).max(a0.abs());
        if scale <= 0.0 {
            return 0;
        }
        let tol = REL * scale;
        if a2.abs() <= tol {
            if a1.abs() <= tol {
                return 0;
            }
            out[0] = -a0 / a1;
            return 1;
        }
        let disc = a1 * a1 - 4.0 * a2 * a0;
        if disc < 0.0 {
            return 0;
        }
        let s = disc.sqrt();
        let q = -0.5 * (a1 + sign_or_positive(a1) * s);
        if q == 0.0 {
            out[0] = 0.0;
            out[1] = 0.0;
            return 2;
        }
        out[0] = q / a2;
        out[1] = a0 / q;
        2
    }

    fn solve_cubic(a3: f32, a2: f32, a1: f32, a0: f32, out: &mut [f32; 3]) -> usize {
        let scale = a3.abs().max(a2.abs()).max(a1.abs()).max(a0.abs());
        if scale <= 0.0 {
            return 0;
        }
        if a3.abs() <= REL * scale {
            return solve_quadratic(a2, a1, a0, out);
        }
        let b = a2 / a3;
        let c = a1 / a3;
        let d = a0 / a3;
        let shift = b / 3.0;
        let p = c - b * b / 3.0;
        let q = 2.0 * b * b * b / 27.0 - b * c / 3.0 + d;

        if p.abs() <= REL * p.abs().max(q.abs()).max(1.0) {
            out[0] = (-q).cbrt() - shift;
            return 1;
        }
        let e1 = q * q / 4.0;
        let e2 = p * p * p / 27.0;
        let disc = e1 + e2;
        if disc.abs() <= REL.sqrt() * (e1.abs() + e2.abs()) {
            out[0] = 3.0 * q / p - shift;
            out[1] = -1.5 * q / p - shift;
            return 2;
        }
        if disc > 0.0 {
            let s = disc.sqrt();
            out[0] = (-q / 2.0 + s).cbrt() + (-q / 2.0 - s).cbrt() - shift;
            1
        } else {
            let m = 2.0 * (-p / 3.0).sqrt();
            let arg = ((3.0 * q) / (p * m)).clamp(-1.0, 1.0);
            let phi = arg.acos() / 3.0;
            let third_turn = std::f32::consts::TAU / 3.0;
            out[0] = m * phi.cos() - shift;
            out[1] = m * (phi - third_turn).cos() - shift;
            out[2] = m * (phi - 2.0 * third_turn).cos() - shift;
            3
        }
    }

    let a = [p0[0] - p[0], p0[1] - p[1]];
    let b = [2.0 * (p1[0] - p0[0]), 2.0 * (p1[1] - p0[1])];
    let c = [
        (p2[0] - p1[0]) - (p1[0] - p0[0]),
        (p2[1] - p1[1]) - (p1[1] - p0[1]),
    ];

    let cc = c[0] * c[0] + c[1] * c[1];
    let bc = b[0] * c[0] + b[1] * c[1];
    let bb = b[0] * b[0] + b[1] * b[1];
    let ac = a[0] * c[0] + a[1] * c[1];
    let ab = a[0] * b[0] + a[1] * b[1];

    let mut roots = [0.0f32; 3];
    let n = solve_cubic(2.0 * cc, 3.0 * bc, bb + 2.0 * ac, ab, &mut roots);

    let mut best_t = 0.0f32;
    let mut best_d2 = a[0] * a[0] + a[1] * a[1];
    let e1 = [a[0] + b[0] + c[0], a[1] + b[1] + c[1]];
    let d1 = e1[0] * e1[0] + e1[1] * e1[1];
    if d1 < best_d2 {
        best_d2 = d1;
        best_t = 1.0;
    }
    for &t in roots.iter().take(n) {
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let v = [
            a[0] + b[0] * t + c[0] * t * t,
            a[1] + b[1] * t + c[1] * t * t,
        ];
        let d2 = v[0] * v[0] + v[1] * v[1];
        if d2 < best_d2 {
            best_d2 = d2;
            best_t = t;
        }
    }
    (best_d2.sqrt(), best_t)
}

/// The Reference's stroke AA profile, kept exactly.
///
/// `manimlib/shaders/quadratic_bezier/stroke/frag.glsl` computes
/// `smoothstep(0.5, -0.5, |d|/aaw - (w/2)/aaw)`; factoring out `1/aaw` gives
/// the form below. The transition band is therefore `aaw` wide — 1.5 px by
/// default — centred on the stroke boundary, which is the "~1.5 px feel"
/// §10.4 commits to keeping.
///
/// What is *not* kept is the distance: the Reference feeds this profile a
/// ribbon coordinate interpolated across a ≤32-segment triangle strip, and we
/// feed it [`distance_to_quadratic`].
pub fn coverage(distance: f64, half_width: f64, aa_width: f64) -> f64 {
    let aaw = aa_width.max(MIN_AA_WIDTH);
    let signed = (distance - half_width) / aaw;
    // smoothstep(0.5, -0.5, signed): descending edges, so the normalized
    // parameter is (signed - 0.5) / (-0.5 - 0.5) = 0.5 - signed.
    let s = (0.5 - signed).clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}

/// Source-over compositing in linear light with straight (non-premultiplied)
/// alpha — the operator §10.2's painter-order model composes tiles under.
pub fn over(src: [f64; 4], dst: [f64; 4]) -> [f64; 4] {
    let sa = src[3];
    if sa <= 0.0 {
        return dst;
    }
    let out_a = sa + dst[3] * (1.0 - sa);
    if out_a <= 0.0 {
        return [0.0; 4];
    }
    let mix = |s: f64, d: f64| (s * sa + d * dst[3] * (1.0 - sa)) / out_a;
    [
        mix(src[0], dst[0]),
        mix(src[1], dst[1]),
        mix(src[2], dst[2]),
        out_a,
    ]
}

// The transcendental funnel. Object-space geometry in this program routes
// through fmn-dmath so the certified engine has one definition of `cos`
// everywhere (`fmn-geom/src/scalar.rs` does the same). The annex is
// standard-only and never certified, but the *CPU reference* this spike
// compares against stands in for the certified engine, so it uses the
// certified functions — otherwise the measured divergence would include the
// platform libm's contribution and the budget would be measuring the wrong
// thing.
fn cos(x: f64) -> f64 {
    fmn_dmath::cos(x)
}
fn acos(x: f64) -> f64 {
    fmn_dmath::acos(x)
}
fn cbrt(x: f64) -> f64 {
    fmn_dmath::cbrt(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn a_straight_quadratic_is_a_point_line_distance() {
        // p1 at the midpoint makes the quadratic term vanish exactly.
        let p0 = [0.0, 0.0];
        let p1 = [5.0, 0.0];
        let p2 = [10.0, 0.0];
        // Perpendicular from above the middle.
        let (d, t) = distance_to_quadratic([4.0, 3.0], p0, p1, p2);
        assert!(approx(d, 3.0, 1e-12), "distance {d}");
        assert!(approx(t, 0.4, 1e-12), "t {t}");
        // Beyond the end: the endpoint wins, which is the round cap.
        let (d, t) = distance_to_quadratic([13.0, 4.0], p0, p1, p2);
        assert!(approx(d, 5.0, 1e-12), "distance {d}");
        assert!(approx(t, 1.0, 1e-12), "t {t}");
    }

    #[test]
    fn a_degenerate_point_curve_has_a_defined_answer() {
        let p = [3.0, 4.0];
        let (d, t) = distance_to_quadratic(p, [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]);
        assert!(approx(d, 5.0, 1e-12), "distance {d}");
        assert_eq!(t, 0.0);
    }

    #[test]
    fn the_closest_point_is_never_beaten_by_a_dense_sampling() {
        // The property that matters: a closed-form root solve must not lose to
        // brute force anywhere on a curved arc, including near the cusp of a
        // sharply-bent quadratic.
        let cases = [
            ([0.0, 0.0], [4.0, 8.0], [8.0, 0.0]),
            ([0.0, 0.0], [10.0, 0.0], [0.0, 0.0]), // fold-back: an exact cusp
            ([-3.0, 1.0], [0.5, -7.0], [4.0, 2.0]),
        ];
        for (p0, p1, p2) in cases {
            for i in 0..37 {
                for j in 0..29 {
                    let p = [-6.0 + i as f64 * 0.5, -6.0 + j as f64 * 0.5];
                    let (d, _) = distance_to_quadratic(p, p0, p1, p2);
                    let mut brute = f64::INFINITY;
                    for k in 0..=20000 {
                        let t = k as f64 / 20000.0;
                        let u = 1.0 - t;
                        let bx = u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0];
                        let by = u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1];
                        let dd = ((bx - p[0]).powi(2) + (by - p[1]).powi(2)).sqrt();
                        if dd < brute {
                            brute = dd;
                        }
                    }
                    assert!(
                        d <= brute + 1e-6,
                        "closed form {d} lost to brute force {brute} at {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_zero_linear_term_still_yields_both_quadratic_roots() {
        // Regression for the `sign(0)` divergence: MSL's `sign` returns 0 where
        // Rust's `signum` returns 1, which turned `t² - 4 = 0` into the double
        // root {0, 0} on the GPU alone. Both engines now write the predicate
        // out longhand, and this pins the answer they must agree on.
        let mut out = [0.0f64; 3];
        let n = solve_quadratic(1.0, 0.0, -4.0, &mut out);
        assert_eq!(n, 2);
        let mut got = [out[0], out[1]];
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(approx(got[0], -2.0, 1e-12), "{got:?}");
        assert!(approx(got[1], 2.0, 1e-12), "{got:?}");
    }

    #[test]
    fn a_near_straight_curve_at_screen_scale_stays_accurate() {
        // Regression for the absolute-degeneracy-threshold bug. A curve whose
        // handle is a thousandth of a pixel off the chord, at coordinates of
        // order 10³ — the regime where forming `p0 - 2p1 + p2` directly, and
        // testing the cubic's leading coefficient against an absolute epsilon,
        // both fall apart. The answer must still be the perpendicular distance.
        let p0 = [120.0, 700.0];
        let p1 = [520.0, 700.001];
        let p2 = [920.0, 700.0];
        let (d, t) = distance_to_quadratic([520.0, 690.0], p0, p1, p2);
        assert!(approx(d, 10.001, 1e-3), "distance {d}");
        assert!(approx(t, 0.5, 1e-3), "t {t}");
        // And the degenerate-scale guard must not swallow a real answer.
        let (d, _) = distance_to_quadratic([120.0, 700.0], p0, p1, p2);
        assert!(d <= 1e-9, "the start anchor is on the curve, got {d}");
    }

    #[test]
    fn the_degeneracy_test_scales_with_the_polynomial() {
        // The same root structure at two magnitudes must give the same roots.
        // An absolute threshold gets this wrong at one end or the other.
        let mut out = [0.0f64; 3];
        for k in [1.0f64, 1e6, 1e-6] {
            let n = solve_cubic(k, -6.0 * k, 11.0 * k, -6.0 * k, &mut out);
            assert_eq!(n, 3, "scale {k} lost roots");
            let mut got = out;
            got.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for (g, w) in got.iter().zip([1.0, 2.0, 3.0]) {
                assert!(approx(*g, w, 1e-6), "scale {k}: {got:?}");
            }
        }
    }

    #[test]
    fn the_aa_profile_reproduces_the_reference_shader() {
        let hw = 2.0;
        let aaw = ANTI_ALIAS_WIDTH_PX;
        // Deep inside: fully covered. Far outside: fully clear.
        assert_eq!(coverage(0.0, hw, aaw), 1.0);
        assert_eq!(coverage(100.0, hw, aaw), 0.0);
        // The band is exactly aaw wide, centred on the boundary.
        assert_eq!(coverage(hw - 0.5 * aaw, hw, aaw), 1.0);
        assert_eq!(coverage(hw + 0.5 * aaw, hw, aaw), 0.0);
        // Dead centre of the band is the smoothstep midpoint.
        assert!(approx(coverage(hw, hw, aaw), 0.5, 1e-12));
        // Monotone across the band.
        let mut prev = 1.0;
        for i in 0..=64 {
            let d = hw - 0.5 * aaw + aaw * (i as f64 / 64.0);
            let c = coverage(d, hw, aaw);
            assert!(c <= prev + 1e-15, "not monotone at {d}");
            prev = c;
        }
    }

    #[test]
    fn over_is_associative_enough_and_respects_the_identities() {
        let c = [0.2, 0.5, 0.9, 0.6];
        let clear = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(over(clear, c), c);
        let opaque = [0.1, 0.2, 0.3, 1.0];
        let r = over(opaque, c);
        assert!(approx(r[3], 1.0, 1e-15));
        assert!(approx(r[0], 0.1, 1e-15));
    }

    #[test]
    fn cubic_roots_are_found_in_every_degenerate_branch() {
        let mut out = [0.0f64; 3];
        // Genuine cubic, three real roots: (t-1)(t-2)(t-3).
        let n = solve_cubic(1.0, -6.0, 11.0, -6.0, &mut out);
        assert_eq!(n, 3);
        let mut got = out;
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (g, w) in got.iter().zip([1.0, 2.0, 3.0]) {
            assert!(approx(*g, w, 1e-9), "{got:?}");
        }
        // Cubic with one real root: t³ + t + 1.
        let n = solve_cubic(1.0, 0.0, 1.0, 1.0, &mut out);
        assert_eq!(n, 1);
        let t = out[0];
        assert!(approx(t * t * t + t + 1.0, 0.0, 1e-9), "root {t}");
        // Degenerate to quadratic, then to linear, then to nothing.
        assert_eq!(solve_cubic(0.0, 1.0, -3.0, 2.0, &mut out), 2);
        assert_eq!(solve_cubic(0.0, 0.0, 2.0, -4.0, &mut out), 1);
        assert!(approx(out[0], 2.0, 1e-12));
        assert_eq!(solve_cubic(0.0, 0.0, 0.0, 5.0, &mut out), 0);
    }
}
