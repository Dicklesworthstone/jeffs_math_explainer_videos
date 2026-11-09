//! Cubic→quadratic reduction (§7.2, fm-6cf).
//!
//! [`quadratic_approximation_of_cubic`] is the exact port of the Reference's
//! fixed two-quad splitter. It remains only as a measured reference oracle;
//! production paths all use [`cubic_to_quadratics`].
//!
//! # The error-bounded converter
//!
//! [`cubic_to_quadratics`] reduces any cubic to a chain of quadratics that
//! holds a stated tolerance. Its output is a single quadratic spline, not a
//! bag of independently fitted pieces. For `n ≥ 2`, let `q_i` be the
//! off-curve control for piece `i`; the internal shared anchor is
//! `a_i = (q_(i-1) + q_i) / 2`. Therefore the derivatives on either side are
//!
//! ```text
//! Q'_(i-1)(1) = 2(a_i - q_(i-1)) = q_i - q_(i-1)
//! Q'_i(0)     = 2(q_i - a_i)     = q_i - q_(i-1)
//! ```
//!
//! so every artificial join is C¹ in any dimension. No tangent-line
//! intersection is needed, which matters in 3D: two endpoint tangent lines
//! are generally skew.
//!
//! Each `q_i` blends the two endpoint-tangent candidates of its uniform
//! sub-cubic. If `L_i = c0 + 3/2(c1-c0)` and
//! `R_i = c3 + 3/2(c2-c3)`, then
//! `q_i = lerp(L_i, R_i, i/(n-1))`. The first and last controls consequently
//! preserve the original cubic's endpoint derivatives exactly, except that a
//! stationary handle may receive the one-ulp, bound-checked separation needed
//! by the shared-anchor path encoding.
//!
//! # The proven bound
//!
//! A quadratic `(a, q, b)` degree-elevates to the cubic
//! `(a, (a+2q)/3, (b+2q)/3, b)`. Subtract those four controls from the
//! corresponding source sub-cubic. The difference curve is itself a cubic
//! Bézier with those four difference controls, and the Bernstein weights are
//! non-negative and sum to one. Convexity therefore gives
//!
//! ```text
//! max_t |C(t) - Q(t)| ≤ max_j |d_j|.
//! ```
//!
//! `chain_holds_tolerance` checks that bound for every output piece. It is a
//! proof over the entire parameter interval, never a sampled estimate.
//!
//! # The one-piece identity and subdivision count
//!
//! For `n = 1`, use the natural control
//! `q = (3p1 - p0 + 3p2 - p3) / 4`. Its two degree-elevated control
//! differences are exactly antisymmetric:
//!
//! ```text
//! d1 = p1 - (p0 + 2q)/3 =  (p3 - p0 + 3p1 - 3p2) / 6
//! d2 = p2 - (p3 + 2q)/3 = -(p3 - p0 + 3p1 - 3p2) / 6 = -d1
//! ```
//!
//! Two cubics sharing endpoints differ by `3(1-t)²t·d1 + 3(1-t)t²·d2`, so with
//! `d2 = -d1` the whole error collapses to a *scalar* function times one fixed
//! vector:
//!
//! ```text
//! C(t) - Q(t) = 3t(1-t)(1-2t) · d1
//! ```
//!
//! `max |3t(1-t)(1-2t)|` on `[0,1]` is `1/(2√3)` at `t = ½ ± √3/6`, so the
//! maximum deviation is exactly `|d1| / (2√3) = |Δ| / (12√3)` where
//! `Δ = p3 - 3p2 + 3p1 - p0`. There is no inequality anywhere in that
//! derivation — see [`max_single_quadratic_deviation`].
//!
//! `Δ` is the cubic's third difference, and `C'''(t) = 6Δ` is *constant*, so a
//! sub-cubic spanning a parameter interval of width `h` has `Δ_sub = Δ·h³`.
//! Splitting uniformly into `n` pieces therefore divides that deviation by
//! exactly `n³`, which gives a closed-form starting count:
//!
//! ```text
//! n = ceil( cbrt( |Δ| / (12√3 · tolerance) ) )
//! ```
//!
//! This exact count is a tight starting guess. The C¹ spline is then checked
//! with the convex-hull bound and `n` doubles until it fits. The construction
//! reproduces every quadratic exactly, so its error depends on the cubic's
//! constant third derivative and falls as `1/n³`; doubling terminates unless
//! the resource or f64-representation guard is reached. Fixed arithmetic order
//! and fixed geometric growth make the output deterministic in the §10.5
//! sense.

use crate::GeomError;
use crate::bezier;
use crate::space_ops;
use crate::vec;
use fmn_core::constants::{DEFAULT_PIXEL_WIDTH, FRAME_WIDTH};
use fmn_core::types::Vec3;

/// The default conversion tolerance, **in output pixels**, fixed by the G0-2
/// look study (`docs/g0/G0-2-look-study-ratification.md`, decision (f)).
///
/// The justification is the antialiasing band: coverage transitions over
/// 1.5 px, so 0.1 px of curve error perturbs it by roughly 7 % of one band
/// edge — invisible. For scale, the Reference's fixed two-quad split
/// ([`quadratic_approximation_of_cubic`]) was measured at up to **7.71 px** of
/// deviation at 1080p, over five times the entire AA band; 0.1 px is 77×
/// tighter than that worst case.
///
/// It is expressed in pixels *deliberately*: it is a visibility criterion, so
/// it must scale with resolution. Callers working in scene units convert with
/// [`tolerance_for_scale`].
pub const DEFAULT_TOLERANCE_PX: f64 = 0.1;

/// The default tolerance in scene units at the default 1920-pixel frame
/// scale fixed by G0-2: `0.1 / 135`.
pub const DEFAULT_TOLERANCE_SCENE: f64 =
    tolerance_for_scale(DEFAULT_PIXEL_WIDTH as f64 / FRAME_WIDTH);

/// The largest number of quadratics [`cubic_to_quadratics`] will emit.
///
/// A resource guard, not a quality knob. The closed-form lower estimate and
/// dyadic search would otherwise let a pathological tolerance (or a
/// non-finite input) size an allocation directly from arithmetic — a
/// decompression-bomb shape, and §16's fuzzing plane treats those as real.
/// Reaching the cap is an error
/// ([`GeomError::ToleranceUnreachable`]) rather than a silently coarser curve.
pub const MAX_SEGMENTS: usize = 4096;

/// `1 / (12√3)` — the exact single-quadratic deviation constant, derived in
/// the module docs.
const DEVIATION_CONSTANT: f64 = 0.048112522432468816;

/// The default tolerance in *scene units*, given the render scale.
///
/// `px_per_unit` is the Reference's mapping: `FRAME_WIDTH / pixel_width`
/// inverted, i.e. 135.0 at 1920×1080 with the default frame.
#[must_use]
pub const fn tolerance_for_scale(px_per_unit: f64) -> f64 {
    DEFAULT_TOLERANCE_PX / px_per_unit
}

/// The quadratic control point that makes the single-quadratic error
/// antisymmetric, and therefore exactly computable (module docs).
///
/// The converter emits it when the source is effectively quadratic and the
/// resulting piece holds the tolerance. Generic cubics use the C¹ spline
/// construction from the module docs even under a loose tolerance.
#[must_use]
pub fn natural_quadratic_handle(a0: Vec3, h0: Vec3, h1: Vec3, a1: Vec3) -> Vec3 {
    let mut q = [0.0; 3];
    for i in 0..3 {
        q[i] = (3.0 * h0[i] - a0[i] + 3.0 * h1[i] - a1[i]) * 0.25;
    }
    q
}

/// A scale-safe Euclidean norm.
fn scaled_norm(v: Vec3) -> f64 {
    let scale = v.iter().map(|x| x.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return 0.0;
    }
    if !scale.is_finite() {
        return f64::INFINITY;
    }
    let x = v[0] / scale;
    let y = v[1] / scale;
    let z = v[2] / scale;
    scale * (x * x + y * y + z * z).sqrt()
}

/// A scale-safe norm rounded outward for use in a proof bound.
///
/// The epsilon margin covers the fixed sequence of divisions, products,
/// additions, and the correctly-rounded square root. It affects only
/// admission of a piece, never its coordinates.
fn conservative_norm(v: Vec3) -> f64 {
    scaled_norm(v) * (1.0 + 16.0 * f64::EPSILON)
}

/// Absolute guard for the fixed affine-arithmetic sequence that derives a
/// sub-cubic from the original f64 controls.
fn subdivision_rounding_guard(p: [Vec3; 4]) -> f64 {
    let coordinate_scale = p
        .iter()
        .flat_map(|point| point.iter())
        .map(|component| component.abs())
        .fold(0.0, f64::max);
    coordinate_scale * 64.0 * f64::EPSILON
}

/// One C¹-spline off-curve control from a uniform source sub-cubic.
fn spline_control(piece: [Vec3; 4], blend: f64) -> Vec3 {
    let mut left = [0.0; 3];
    let mut right = [0.0; 3];
    for axis in 0..3 {
        left[axis] = piece[0][axis] + 1.5 * (piece[1][axis] - piece[0][axis]);
        right[axis] = piece[3][axis] + 1.5 * (piece[2][axis] - piece[3][axis]);
    }
    vec::lerp(left, right, blend)
}

/// Move an off-curve control by the smallest representable amount when it
/// would otherwise be indistinguishable from an adjacent anchor.
///
/// `[anchor, anchor, distinct]` is the shared-anchor encoding's subpath-break
/// marker, so a stationary cubic endpoint cannot use that exact point run.
/// The full Bernstein bound is checked after this nudge; if even one ulp is
/// too large for the requested tolerance, conversion refuses rather than
/// silently violating it.
fn nudge_one_ulp(control: &mut Vec3) {
    for component in control {
        let upward = component.next_up();
        if upward.is_finite() && upward != *component {
            *component = upward;
            return;
        }
        let downward = component.next_down();
        if downward.is_finite() && downward != *component {
            *component = downward;
            return;
        }
    }
}

/// Build the candidate chain for a fixed uniform piece count.
fn approximation_for_segments(p: [Vec3; 4], n: usize) -> Vec<Vec3> {
    let is_constant = p.iter().all(|point| *point == p[0]);
    if n == 1 {
        let mut control = natural_quadratic_handle(p[0], p[1], p[2], p[3]);
        if !is_constant {
            if control == p[0] {
                nudge_one_ulp(&mut control);
            }
            if control == p[3] {
                nudge_one_ulp(&mut control);
            }
        }
        return vec![p[0], control, p[3]];
    }

    let mut controls: Vec<Vec3> = (0..n)
        .map(|i| {
            let piece = subsegment(p, i as f64 / n as f64, (i + 1) as f64 / n as f64);
            spline_control(piece, i as f64 / (n - 1) as f64)
        })
        .collect();

    // Keep every real curve distinct from both adjacent anchors. Besides the
    // point-run break ambiguity, downstream anchor-mode cleanup treats a
    // handle on either anchor as degenerate. Moving the later control grows
    // a representable gap while midpoint anchors retain C1 exactly.
    if !is_constant {
        for _ in 0..4 {
            let mut changed = false;
            if controls[0] == p[0] {
                nudge_one_ulp(&mut controls[0]);
                changed = true;
            }
            for i in 1..n {
                let midpoint = space_ops::midpoint(controls[i - 1], controls[i]);
                if midpoint == controls[i - 1] || midpoint == controls[i] {
                    nudge_one_ulp(&mut controls[i]);
                    changed = true;
                }
            }
            if controls[n - 1] == p[3] {
                nudge_one_ulp(&mut controls[n - 1]);
                changed = true;
            }
            if !changed {
                break;
            }
        }
    }

    let mut out = Vec::with_capacity(2 * n + 1);
    out.push(p[0]);
    for i in 0..n {
        out.push(controls[i]);
        let anchor = if i + 1 == n {
            p[3]
        } else {
            space_ops::midpoint(controls[i], controls[i + 1])
        };
        out.push(anchor);
    }
    out
}

/// Whether the candidate's Bernstein convex-hull bound holds on every piece.
fn chain_holds_tolerance(p: [Vec3; 4], approximation: &[Vec3], n: usize, tolerance: f64) -> bool {
    let rounding_guard = subdivision_rounding_guard(p);
    (0..n).all(|i| {
        let source = subsegment(p, i as f64 / n as f64, (i + 1) as f64 / n as f64);
        let a = approximation[2 * i];
        let q = approximation[2 * i + 1];
        let b = approximation[2 * i + 2];
        if a != b && (a == q || b == q) {
            return false;
        }
        let mut elevated = [a, [0.0; 3], [0.0; 3], b];
        for axis in 0..3 {
            // Weighted form avoids overflowing `a + 2q` when the elevated
            // control itself is representable.
            elevated[1][axis] = a[axis] / 3.0 + q[axis] * (2.0 / 3.0);
            elevated[2][axis] = b[axis] / 3.0 + q[axis] * (2.0 / 3.0);
        }
        source.into_iter().zip(elevated).all(|(c, e)| {
            conservative_norm([c[0] - e[0], c[1] - e[1], c[2] - e[2]]) + rounding_guard <= tolerance
        })
    })
}

fn third_difference(p: [Vec3; 4]) -> Vec3 {
    let mut delta = [0.0; 3];
    for axis in 0..3 {
        delta[axis] = p[3][axis] - 3.0 * p[2][axis] + 3.0 * p[1][axis] - p[0][axis];
    }
    delta
}

/// Whether the controls are quadratic to f64 working precision.
fn is_effectively_quadratic(p: [Vec3; 4]) -> bool {
    let span = p
        .windows(2)
        .map(|pair| scaled_norm(vec::sub(pair[1], pair[0])))
        .fold(0.0, f64::max);
    scaled_norm(third_difference(p)) <= 64.0 * f64::EPSILON * span
}

/// The **exact** maximum deviation between this cubic and its single-quadratic
/// approximation (module docs). Not an upper bound: the error polynomial is a
/// scalar function times one vector, so its maximum is closed-form.
#[must_use]
pub fn max_single_quadratic_deviation(a0: Vec3, h0: Vec3, h1: Vec3, a1: Vec3) -> f64 {
    scaled_norm(third_difference([a0, h0, h1, a1])) * DEVIATION_CONSTANT
}

/// Build and validate the candidate retained by the public converter.
fn approximation_with_tolerance(
    a0: Vec3,
    h0: Vec3,
    h1: Vec3,
    a1: Vec3,
    tolerance: f64,
) -> Result<(usize, Vec<Vec3>), GeomError> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(GeomError::InvalidTolerance);
    }
    let p = [a0, h0, h1, a1];
    let deviation = max_single_quadratic_deviation(a0, h0, h1, a1);
    if !deviation.is_finite() {
        // Non-finite control points. Refuse rather than size an allocation
        // from a NaN.
        return Err(GeomError::ToleranceUnreachable { needed: usize::MAX });
    }
    let mut n = if deviation <= tolerance && is_effectively_quadratic(p) {
        1
    } else {
        // The closed form is a lower estimate for the C¹ candidate. Round it
        // to a dyadic count so geometric growth always reaches the resource
        // cap instead of skipping an admissible 4096-piece result.
        let estimate = if deviation <= tolerance {
            2
        } else {
            // Through the deterministic transcendental funnel, not
            // `f64::cbrt`: this value becomes a semantic point count.
            let exact = crate::scalar::cbrt(deviation / tolerance).ceil();
            if !exact.is_finite() || exact > MAX_SEGMENTS as f64 {
                return Err(GeomError::ToleranceUnreachable { needed: usize::MAX });
            }
            (exact as usize).max(2)
        };
        let dyadic = estimate.next_power_of_two();
        if dyadic > MAX_SEGMENTS {
            return Err(GeomError::ToleranceUnreachable { needed: usize::MAX });
        }
        dyadic
    };

    loop {
        let approximation = approximation_for_segments(p, n);
        if chain_holds_tolerance(p, &approximation, n, tolerance) {
            return Ok((n, approximation));
        }
        if n == MAX_SEGMENTS {
            break;
        }
        n *= 2;
    }
    Err(GeomError::ToleranceUnreachable {
        needed: MAX_SEGMENTS + 1,
    })
}

/// How many uniform pieces this cubic needs to hold `tolerance`.
///
/// The closed form of the module docs sizes the first guess. The C¹ candidate
/// is checked piece by piece with the Bernstein convex-hull bound, and the
/// dyadic count grows through [`MAX_SEGMENTS`] until every piece holds.
///
/// The loop is deterministic: it compares f64 values produced in a fixed
/// order from the inputs, so the same cubic and tolerance gives the same
/// count on every certified build (§10.5).
pub fn segments_for_tolerance(
    a0: Vec3,
    h0: Vec3,
    h1: Vec3,
    a1: Vec3,
    tolerance: f64,
) -> Result<usize, GeomError> {
    approximation_with_tolerance(a0, h0, h1, a1, tolerance).map(|(n, _)| n)
}

/// **The one converter** (§7.2): reduce a cubic to a chain of quadratics whose
/// deviation from it is at most `tolerance`, in shared-anchor layout
/// `[a0, h, a1, h, a2, …]` (odd length, `2n + 1` points for `n` quadratics).
///
/// Every implemented cubic source routes here — API cubics (including the
/// library's `CubicBezier`) and smoothing output. The future SVG importer is
/// required to use this same seam. TrueType glyf outlines are already
/// quadratic and are a zero-loss passthrough that never reaches this function.
/// One audited converter is what makes curve fidelity a property of the system
/// rather than of the call site.
///
/// # Failure
///
/// Zero-length, collinear, exact-quadratic, inflection, cusp, and spatial
/// inputs all have defined output. Failure is limited to an invalid tolerance
/// ([`GeomError::InvalidTolerance`]) or a request/input whose finite,
/// representable approximation cannot fit below [`MAX_SEGMENTS`]
/// ([`GeomError::ToleranceUnreachable`]).
pub fn cubic_to_quadratics(
    a0: Vec3,
    h0: Vec3,
    h1: Vec3,
    a1: Vec3,
    tolerance: f64,
) -> Result<Vec<Vec3>, GeomError> {
    approximation_with_tolerance(a0, h0, h1, a1, tolerance).map(|(_, approximation)| approximation)
}

/// de Casteljau split at `t`, returning `(left, right)` control quadruples.
fn split(p: [Vec3; 4], t: f64) -> ([Vec3; 4], [Vec3; 4]) {
    let l = |a: Vec3, b: Vec3| -> Vec3 {
        let mut o = [0.0; 3];
        for i in 0..3 {
            o[i] = a[i] + (b[i] - a[i]) * t;
        }
        o
    };
    let q1 = l(p[0], p[1]);
    let r = l(p[1], p[2]);
    let s1 = l(p[2], p[3]);
    let q2 = l(q1, r);
    let s = l(r, s1);
    let q3 = l(q2, s);
    ([p[0], q1, q2, q3], [q3, s, s1, p[3]])
}

/// The control points of the cubic restricted to `[t0, t1] ⊆ [0, 1]`.
fn subsegment(p: [Vec3; 4], t0: f64, t1: f64) -> [Vec3; 4] {
    if t0 <= 0.0 && t1 >= 1.0 {
        return p;
    }
    if t0 <= 0.0 {
        return split(p, t1).0;
    }
    let right = split(p, t0).1;
    // Re-normalize t1 into the right piece's own parameter.
    let u = (t1 - t0) / (1.0 - t0);
    split(right, u.clamp(0.0, 1.0)).0
}

/// Approximate the cubic `(a0, h0, h1, a1)` with two joined quadratics,
/// returned in shared-anchor layout: `[a0, i0, mid, i1, a1]`.
#[must_use]
pub fn quadratic_approximation_of_cubic(a0: Vec3, h0: Vec3, h1: Vec3, a1: Vec3) -> [Vec3; 5] {
    // Tangent directions at the ends.
    let t0 = vec::sub(h0, a0);
    let t1 = vec::sub(a1, h1);

    // Inflection points of the planar cubic, per
    // caffeineowl.com/graphics/2d/vectorial/cubic-inflexion.html.
    let p = vec::sub(h0, a0);
    let q = vec::add(vec::sub(h1, vec::scale(h0, 2.0)), a0);
    let r = vec::sub(
        vec::add(a1, vec::scale(h0, 3.0)),
        vec::add(vec::scale(h1, 3.0), a0),
    );

    let a = space_ops::cross2d(q, r);
    let b = space_ops::cross2d(p, r);
    let c = space_ops::cross2d(p, q);

    let disc = b * b - 4.0 * a * c;
    let has_infl = disc > 0.0;
    let sqrt_disc = disc.abs().sqrt();
    let root = |sgn: f64| -> f64 {
        if a == 0.0 {
            if b == 0.0 { 0.0 } else { -c / b }
        } else {
            (-b + sgn * sqrt_disc) / (2.0 * a)
        }
    };
    let ti_min = root(-1.0);
    let ti_max = root(1.0);

    // t starts at ½ and is replaced by an interior inflection if one exists;
    // when both roots are interior the Reference lets the larger win.
    let mut t_mid = 0.5;
    if has_infl && 0.0 < ti_min && ti_min < 1.0 {
        t_mid = ti_min;
    }
    if has_infl && 0.0 < ti_max && ti_max < 1.0 {
        t_mid = ti_max;
    }

    let mid = bezier::cubic_point(a0, h0, h1, a1, t_mid);
    // The derivative direction, via the quadratic on the difference points.
    let tm = bezier::quadratic_point(vec::sub(h0, a0), vec::sub(h1, h0), vec::sub(a1, h1), t_mid);

    let i0 = space_ops::find_intersection(a0, t0, mid, tm, 1e-5);
    let i1 = space_ops::find_intersection(a1, t1, mid, tm, 1e-5);

    [a0, i0, mid, i1, a1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approximation_interpolates_endpoints_and_midpoint() {
        let a0 = [0.0, 0.0, 0.0];
        let h0 = [0.0, 1.0, 0.0];
        let h1 = [1.0, 2.0, 0.0];
        let a1 = [2.0, 2.0, 0.0];
        let out = quadratic_approximation_of_cubic(a0, h0, h1, a1);
        assert_eq!(out[0], a0);
        assert_eq!(out[4], a1);
        // The split point lies on the cubic.
        let on_curve = bezier::cubic_point(a0, h0, h1, a1, 0.5);
        for i in 0..3 {
            assert!((out[2][i] - on_curve[i]).abs() < 1e-12);
        }
    }

    #[test]
    fn degenerate_collinear_cubic_stays_on_line() {
        let out = quadratic_approximation_of_cubic(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        );
        for p in out {
            assert!(p[1].abs() < 1e-12 && p[2].abs() < 1e-12);
        }
    }

    // ---------------------------------------------------------------- fm-6cf

    /// Deterministic pseudo-random cubics from the one RNG (§6.5), so these
    /// property tests are the same test on every machine and every run.
    fn random_cubics(n: usize) -> Vec<[Vec3; 4]> {
        let mut rng = fmn_core::rng::RngRoot::from_seed(0x6cf)
            .substream("cubic-converter")
            .sequential();
        let mut span = || (rng.next_f64() - 0.5) * 12.0;
        (0..n)
            .map(|_| {
                [
                    [span(), span(), span()],
                    [span(), span(), span()],
                    [span(), span(), span()],
                    [span(), span(), span()],
                ]
            })
            .collect()
    }

    fn dist(a: Vec3, b: Vec3) -> f64 {
        scaled_norm(vec::sub(a, b))
    }

    /// The largest parameter-matched deviation between a cubic and its single
    /// natural quadratic, by dense sampling.
    fn sampled_single_deviation(p: [Vec3; 4], steps: usize) -> f64 {
        let q = natural_quadratic_handle(p[0], p[1], p[2], p[3]);
        (0..=steps)
            .map(|k| {
                let t = k as f64 / steps as f64;
                dist(
                    bezier::cubic_point(p[0], p[1], p[2], p[3], t),
                    bezier::quadratic_point(p[0], q, p[3], t),
                )
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn single_quadratic_deviation_is_exact_not_a_bound() {
        // The claim in the module docs is equality, not `<=`. If the closed
        // form were merely an upper bound this test would show it drifting
        // above the sampled maximum on curved inputs.
        for (i, p) in random_cubics(200).into_iter().enumerate() {
            let closed = max_single_quadratic_deviation(p[0], p[1], p[2], p[3]);
            let sampled = sampled_single_deviation(p, 20_000);
            let scale = closed.max(1e-12);
            assert!(
                (closed - sampled).abs() / scale < 1e-6,
                "case {i}: closed form {closed} vs sampled {sampled}"
            );
        }
    }

    #[test]
    fn converted_chain_holds_the_tolerance() {
        // Parameter-matched deviation is an upper bound on geometric
        // deviation, so holding it is the stronger statement.
        for tolerance in [1.0, 0.1, 0.01, 1e-3, 1e-5] {
            for (i, p) in random_cubics(60).into_iter().enumerate() {
                let n = segments_for_tolerance(p[0], p[1], p[2], p[3], tolerance).unwrap();
                let out = cubic_to_quadratics(p[0], p[1], p[2], p[3], tolerance).unwrap();
                assert_eq!(out.len(), 2 * n + 1);
                let mut worst: f64 = 0.0;
                for piece in 0..n {
                    let a = out[2 * piece];
                    let h = out[2 * piece + 1];
                    let b = out[2 * piece + 2];
                    for k in 0..=400 {
                        let u = k as f64 / 400.0;
                        let t = (piece as f64 + u) / n as f64;
                        worst = worst.max(dist(
                            bezier::cubic_point(p[0], p[1], p[2], p[3], t),
                            bezier::quadratic_point(a, h, b, u),
                        ));
                    }
                }
                assert!(
                    worst <= tolerance * (1.0 + 1e-9),
                    "tol {tolerance}, case {i}: worst deviation {worst} over {n} pieces"
                );
            }
        }
    }

    #[test]
    fn converted_chain_is_c1_in_three_dimensions() {
        for p in random_cubics(50) {
            let tol = 0.01;
            let out = cubic_to_quadratics(p[0], p[1], p[2], p[3], tol).unwrap();
            let n = (out.len() - 1) / 2;
            assert_eq!(out[0], p[0]);
            assert_eq!(out[2 * n], p[3]);
            for piece in 1..n {
                let anchor = out[2 * piece];
                let incoming = vec::sub(anchor, out[2 * piece - 1]);
                let outgoing = vec::sub(out[2 * piece + 1], anchor);
                let scale = scaled_norm(incoming).max(scaled_norm(outgoing)).max(1.0);
                assert!(
                    dist(incoming, outgoing) <= 8.0 * f64::EPSILON * scale,
                    "piece {piece}/{n}: incoming {incoming:?}, outgoing {outgoing:?}"
                );
            }
        }
    }

    #[test]
    fn generic_loose_tolerance_still_preserves_endpoint_tangents() {
        let p = [
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 3.0],
            [4.0, -1.0, 5.0],
            [7.0, 2.0, -2.0],
        ];
        let out = cubic_to_quadratics(p[0], p[1], p[2], p[3], 100.0).unwrap();
        let n = (out.len() - 1) / 2;
        assert!(n >= 2, "a generic cubic needs the C1 spline policy");

        let start_cubic = vec::sub(p[1], p[0]);
        let start_quad = vec::sub(out[1], out[0]);
        let end_cubic = vec::sub(p[3], p[2]);
        let end_quad = vec::sub(out[2 * n], out[2 * n - 1]);
        assert!(scaled_norm(space_ops::cross(start_cubic, start_quad)) < 1e-12);
        assert!(scaled_norm(space_ops::cross(end_cubic, end_quad)) < 1e-12);
        assert!(space_ops::dot(start_cubic, start_quad) > 0.0);
        assert!(space_ops::dot(end_cubic, end_quad) > 0.0);
    }

    #[test]
    fn deviation_falls_as_the_cube_of_the_piece_count() {
        // The closed-form n rests on Δ_sub = Δ·h³. If that were wrong the
        // converter would still "work" but would over- or under-subdivide.
        let p = [
            [-1.0, -1.0, 0.0],
            [-1.0, 1.6, 0.0],
            [1.0, 1.6, 0.0],
            [1.0, -1.0, 0.0],
        ];
        let one = max_single_quadratic_deviation(p[0], p[1], p[2], p[3]);
        for n in [2usize, 3, 5, 8] {
            let sub = subsegment(p, 0.0, 1.0 / n as f64);
            let got = max_single_quadratic_deviation(sub[0], sub[1], sub[2], sub[3]);
            let want = one / (n as f64).powi(3);
            assert!(
                (got - want).abs() / want < 1e-9,
                "n={n}: piece deviation {got}, expected {want}"
            );
        }
    }

    #[test]
    fn an_exact_quadratic_converts_to_one_piece_with_no_error() {
        // Degree-elevate a quadratic to a cubic: Δ is exactly zero, so the
        // converter must not subdivide at all and must lose nothing.
        let (a, q, b) = ([0.0, 0.0, 0.0], [1.0, 2.0, 0.0], [3.0, 0.0, 0.0]);
        let h0 = [
            a[0] + 2.0 / 3.0 * (q[0] - a[0]),
            a[1] + 2.0 / 3.0 * (q[1] - a[1]),
            0.0,
        ];
        let h1 = [
            b[0] + 2.0 / 3.0 * (q[0] - b[0]),
            b[1] + 2.0 / 3.0 * (q[1] - b[1]),
            0.0,
        ];
        assert!(max_single_quadratic_deviation(a, h0, h1, b) < 1e-15);
        let out = cubic_to_quadratics(a, h0, h1, b, 1e-9).unwrap();
        assert_eq!(out.len(), 3);
        assert!(dist(out[1], q) < 1e-12, "recovered handle {:?}", out[1]);
    }

    #[test]
    fn degenerate_inputs_have_defined_output() {
        let tol = 0.01;
        // Zero-length: every control point coincident.
        let z = [7.0, -2.0, 1.0];
        let out = cubic_to_quadratics(z, z, z, z, tol).unwrap();
        assert_eq!(out.len(), 3);
        assert!(out.iter().all(|p| dist(*p, z) < 1e-15));

        // Collinear controls stay on the line (a straight cubic has Δ ≠ 0 in
        // general, so this exercises subdivision, not the trivial path).
        let out = cubic_to_quadratics(
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            tol,
        )
        .unwrap();
        assert!(out.iter().all(|p| p[1].abs() < 1e-12 && p[2].abs() < 1e-12));

        // An inflection exactly at a parameter bound.
        let out = cubic_to_quadratics(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [3.0, 0.0, 0.0],
            tol,
        )
        .unwrap();
        assert!(out.len() % 2 == 1 && out.len() >= 3);

        // A cusp.
        let out = cubic_to_quadratics(
            [0.0, 0.0, 0.0],
            [1.4, 0.0, 0.0],
            [-1.4, 0.0, 0.0],
            [0.0, 0.2, 0.0],
            tol,
        )
        .unwrap();
        assert!(out.len() % 2 == 1);
    }

    #[test]
    fn invalid_or_unrepresentable_requests_are_refused() {
        let (a0, h0, h1, a1) = (
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        );
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                cubic_to_quadratics(a0, h0, h1, a1, bad),
                Err(GeomError::InvalidTolerance),
                "tolerance {bad} should be refused"
            );
        }
        // A tolerance so fine it would blow the segment cap is refused by
        // name rather than silently coarsened.
        assert!(matches!(
            cubic_to_quadratics(a0, h0, h1, a1, 1e-300),
            Err(GeomError::ToleranceUnreachable { .. })
        ));
        // Non-finite geometry is refused, never turned into an allocation.
        assert!(matches!(
            cubic_to_quadratics(a0, [f64::NAN, 0.0, 0.0], h1, a1, 0.1),
            Err(GeomError::ToleranceUnreachable { .. })
        ));
    }

    #[test]
    fn subnormal_geometry_cannot_underflow_the_error_bound() {
        let p = [
            [0.0, 0.0, 0.0],
            [1e-200, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let tolerance = 1e-210;
        assert!(
            max_single_quadratic_deviation(p[0], p[1], p[2], p[3]) > tolerance,
            "the scale-safe norm must not collapse the cubic term to zero"
        );
        let out = cubic_to_quadratics(p[0], p[1], p[2], p[3], tolerance).unwrap();
        let n = (out.len() - 1) / 2;
        let mut worst: f64 = 0.0;
        for piece in 0..n {
            for sample in 0..=32 {
                let u = sample as f64 / 32.0;
                let t = (piece as f64 + u) / n as f64;
                worst = worst.max(dist(
                    bezier::cubic_point(p[0], p[1], p[2], p[3], t),
                    bezier::quadratic_point(
                        out[2 * piece],
                        out[2 * piece + 1],
                        out[2 * piece + 2],
                        u,
                    ),
                ));
            }
        }
        assert!(worst <= tolerance, "{worst} exceeds {tolerance}");
    }

    #[test]
    fn the_dyadic_search_tests_the_segment_cap() {
        let p = [
            [-1.0, -1.0, 0.0],
            [-1.0, 1.6, 0.0],
            [1.0, 1.6, 0.0],
            [1.0, -1.0, 0.0],
        ];
        // The closed-form estimate rounds to 2048, whose C1 candidate fails;
        // the 4096-piece candidate fits. A search that jumped past the cap
        // would incorrectly refuse this request.
        let tolerance = 3.0e-11;
        assert_eq!(
            segments_for_tolerance(p[0], p[1], p[2], p[3], tolerance).unwrap(),
            MAX_SEGMENTS
        );
    }

    fn golden_hash(points: &[Vec3]) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in (points.len() as u64).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        for point in points {
            for component in point {
                for byte in component.to_bits().to_le_bytes() {
                    hash ^= u64::from(byte);
                    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        hash
    }

    #[test]
    fn converter_fixture_outputs_are_bit_locked() {
        let fixtures = [
            (
                [
                    [-1.0, -1.0, 0.0],
                    [-1.0, 1.6, 0.0],
                    [1.0, 1.6, 0.0],
                    [1.0, -1.0, 0.0],
                ],
                DEFAULT_TOLERANCE_SCENE,
            ),
            (
                [
                    [0.0, 0.0, 0.0],
                    [2.0, 3.0, 0.0],
                    [-2.0, 3.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                0.01,
            ),
            (
                [
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [1.0 / 3.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                ],
                DEFAULT_TOLERANCE_SCENE,
            ),
            (
                [
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0, 1.0],
                ],
                0.01,
            ),
            (
                [
                    [0.0, 0.0, 0.0],
                    [4.0, 0.0, 0.0],
                    [-2.0, 0.0, 0.0],
                    [3.0, 0.0, 0.0],
                ],
                0.01,
            ),
            (
                [
                    [2.0, -3.0, 1.0],
                    [2.0, -3.0, 1.0],
                    [2.0, -3.0, 1.0],
                    [2.0, -3.0, 1.0],
                ],
                DEFAULT_TOLERANCE_SCENE,
            ),
        ];
        let actual: Vec<u64> = fixtures
            .into_iter()
            .map(|(p, tolerance)| {
                let points = cubic_to_quadratics(p[0], p[1], p[2], p[3], tolerance).unwrap();
                golden_hash(&points)
            })
            .collect();
        assert_eq!(
            actual,
            [
                4_294_836_761_618_142_727,
                9_775_589_468_788_572_111,
                6_965_456_409_420_322_806,
                9_618_753_173_515_767_359,
                1_350_664_560_155_362_047,
                2_491_867_785_928_297_583,
            ]
        );
    }

    #[test]
    fn conversion_is_bit_identical_across_calls() {
        for p in random_cubics(40) {
            let a = cubic_to_quadratics(p[0], p[1], p[2], p[3], 0.01).unwrap();
            let b = cubic_to_quadratics(p[0], p[1], p[2], p[3], 0.01).unwrap();
            assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b.iter()) {
                for i in 0..3 {
                    assert_eq!(x[i].to_bits(), y[i].to_bits());
                }
            }
        }
    }

    #[test]
    fn the_default_tolerance_beats_the_reference_by_the_margin_g0_2_measured() {
        // G0-2 L8 measured the Reference's fixed two-quad split at up to
        // 7.71 px on this cubic at 135 px/unit. Ours must land at or under
        // the 0.1 px default — the claim that "curve fidelity visibly exceeds
        // the Reference's" is this assertion.
        const PX_PER_UNIT: f64 = 135.0;
        let p = [
            [-1.0, -1.0, 0.0],
            [-1.0, 1.6, 0.0],
            [1.0, 1.6, 0.0],
            [1.0, -1.0, 0.0],
        ];
        let tol = tolerance_for_scale(PX_PER_UNIT);
        let n = segments_for_tolerance(p[0], p[1], p[2], p[3], tol).unwrap();
        let out = cubic_to_quadratics(p[0], p[1], p[2], p[3], tol).unwrap();
        let mut worst: f64 = 0.0;
        for piece in 0..n {
            for k in 0..=500 {
                let u = k as f64 / 500.0;
                let t = (piece as f64 + u) / n as f64;
                worst = worst.max(dist(
                    bezier::cubic_point(p[0], p[1], p[2], p[3], t),
                    bezier::quadratic_point(
                        out[2 * piece],
                        out[2 * piece + 1],
                        out[2 * piece + 2],
                        u,
                    ),
                ));
            }
        }
        let worst_px = worst * PX_PER_UNIT;
        assert!(
            worst_px <= DEFAULT_TOLERANCE_PX * (1.0 + 1e-9),
            "{n} pieces, worst {worst_px} px"
        );
        // And the Reference's own splitter on the same curve, for the record.
        let reference = quadratic_approximation_of_cubic(p[0], p[1], p[2], p[3]);
        let mut ref_worst: f64 = 0.0;
        for k in 0..=2000 {
            let t = k as f64 / 2000.0;
            let c = bezier::cubic_point(p[0], p[1], p[2], p[3], t);
            let mut best = f64::INFINITY;
            for half in 0..2 {
                for j in 0..=400 {
                    let u = j as f64 / 400.0;
                    let q = bezier::quadratic_point(
                        reference[2 * half],
                        reference[2 * half + 1],
                        reference[2 * half + 2],
                        u,
                    );
                    best = best.min(dist(c, q));
                }
            }
            ref_worst = ref_worst.max(best);
        }
        assert!(
            ref_worst * PX_PER_UNIT > 5.0,
            "the Reference splitter was measured at 7.71 px here; got {}",
            ref_worst * PX_PER_UNIT
        );
    }
}
