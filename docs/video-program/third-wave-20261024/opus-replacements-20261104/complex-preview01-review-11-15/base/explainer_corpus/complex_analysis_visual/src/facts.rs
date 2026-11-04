//! Every number the film shows or says, computed from `model.rs` and printed
//! as `key<TAB>value` for `./x facts-check` against `oracle.py`, which
//! recomputes each one independently (Python, other algorithms). Keys name
//! the storyboard beat that uses them. A few values are also pinned against
//! literals from published sources.
//!
//! Spoken roundings follow the printed precision: the narration says "about
//! two point eight" for `deriv.scale` = 2.83 only because the script rounds
//! that printed value once more; every such pair is listed in the storyboard.

use std::f64::consts::{PI, TAU};

use explainer_kit::Facts;

use crate::model::{
    self, C, Func, I, ONE, ZERO, ZETA_ZERO_HEIGHTS, arc_bound, bump, bump_radius, bump_recipe, bump_recipe0, circle,
    circle_average, circle_coefficient, gamma, geometric_partial, loop_products, loop_sum, midpoint_integral,
    quartic_roots, quartic_upper_residue_sum, quintic_roots, quotient, sqrt_carried, square_gap_ratio,
    winding_certified, winding_count, zeta, zeta_zero_near,
};

/// `x` to `places` decimals, never "−0.000": a value that rounds to zero
/// prints without a sign (the oracle does the same).
pub fn fixed(x: f64, places: usize) -> String {
    let s = format!("{x:.places$}");
    match s.strip_prefix('-') {
        Some(rest) if rest.chars().all(|c| c == '0' || c == '.') => rest.to_owned(),
        _ => s,
    }
}

/// A list of numbers, comma separated.
fn list(xs: &[f64], places: usize) -> String {
    xs.iter().map(|&x| fixed(x, places)).collect::<Vec<_>>().join(",")
}

fn need<T>(x: Option<T>, what: &str) -> Result<T, String> {
    x.ok_or_else(|| format!("facts: {what} is undefined"))
}

/// A proven bound on |f′| over the circle `|z − centre| = r`, for each loop
/// the film counts on (triangle inequality on the closed-form derivative).
fn derivative_bound(f: Func, centre: C, r: f64) -> Result<f64, String> {
    let gap = |p: C| (p - centre).abs() - r; // least |z − p| on the circle, p outside
    let reach = centre.abs() + r; // largest |z| on the circle
    let bound = match f {
        // 1
        Func::Identity => 1.0,
        // 2z
        Func::Square => 2.0 * reach,
        // −1/z², loop around 0
        Func::Recip if centre == ZERO => 1.0 / (r * r),
        // −2z/((z − i)(z + i))²: one factor is r, the other at least the gap
        Func::Bump if centre == I || centre == -I => {
            let other = if centre == I { -I } else { I };
            2.0 * reach / (r * gap(other)).powi(2)
        }
        // 3z²
        Func::Cubic => 3.0 * reach * reach,
        // (3 − z)/(z + 1)³
        Func::ZeroDoublePole => {
            let near = if centre == -ONE { r } else { gap(-ONE) };
            ((C::real(3.0) - centre).abs() + r) / near.powi(3)
        }
        // (zero − pole)/(z − pole)²
        Func::Moebius if centre == ZERO => {
            (model::MOEBIUS_POLE - model::MOEBIUS_ZERO) / (r - model::MOEBIUS_POLE).powi(2)
        }
        // 5z⁴ − 3
        Func::Quintic => 5.0 * reach.powi(4) + 3.0,
        _ => return Err(format!("facts: no derivative bound for {f:?} at {centre:?}")),
    };
    if bound.is_finite() && bound > 0.0 {
        Ok(bound)
    } else {
        Err(format!("facts: degenerate derivative bound for {f:?}"))
    }
}

/// Turns of `f` around zero along the circle, as a whole number, certified:
/// no turn can hide between the 4096 samples (see `winding_certified`). The
/// sampled estimate must agree.
fn turns(f: Func, centre: C, r: f64) -> Result<i64, String> {
    let bound = derivative_bound(f, centre, r)?;
    let count = need(
        winding_certified(|z| f.eval(z), centre, r, 4096, bound),
        "a certified winding",
    )?;
    let sampled = need(
        winding_count(|z| f.eval(z), &circle(centre, r, 4096)),
        "a sampled winding",
    )?;
    if sampled != count {
        return Err(format!("facts: {f:?} sampled {sampled} ≠ certified {count}"));
    }
    Ok(count)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let deg = |w: C| w.arg().to_degrees();

    // 01 the bump
    f.add("bump.peak", fixed(bump(0.0), 0))?;
    f.add("bump.read_at", fixed(model::BUMP_READ_AT, 1))?;
    f.add("bump.at1_1", fixed(bump(model::BUMP_READ_AT), 3))?;
    f.add("bump.recipe40.at0_5", fixed(bump_recipe0(0.5, 40), 6))?;
    f.add("bump.recipe40.at0_9", fixed(bump_recipe0(0.9, 40), 3))?;
    f.add("bump.recipe40.at1_1", fixed(bump_recipe0(model::BUMP_READ_AT, 40), 1))?;
    f.add("bump.radius0", fixed(bump_radius(0.0), 0))?;
    let poles: Vec<f64> = Func::Bump
        .poles((-2.0, 2.0), (-2.0, 2.0))
        .iter()
        .map(|(p, _)| p.im)
        .collect();
    f.add("bump.poles", list(&poles, 0))?;

    // 02 numbers that turn
    let sum = model::TURN_POINT + model::TURN_ADDED;
    f.add("turn.sum", list(&[sum.re, sum.im], 0))?;
    let w = model::TURN_BY;
    f.add("turn.one_plus_i.length", fixed(w.abs(), 3))?;
    f.add("turn.one_plus_i.angle_deg", fixed(deg(w), 0))?;
    let w2 = w * w;
    f.add("turn.square.value", list(&[w2.re, w2.im], 0))?;
    f.add("turn.square.length", fixed(w2.abs(), 0))?;
    f.add("turn.square.angle_deg", fixed(deg(w2), 0))?;

    // 03 two planes
    f.add("square.circle.out_radius", fixed(0.5_f64.powi(2), 2))?;
    f.add("identity.circle.turns", turns(Func::Identity, ZERO, 0.5)?)?;
    f.add("square.circle.turns", turns(Func::Square, ZERO, 0.5)?)?;
    let radii: Vec<f64> = [-1.0, 0.0, 1.0]
        .iter()
        .map(|&x: &f64| need(Func::Exp.eval(C::real(x)), "exp").map(|w| w.abs()))
        .collect::<Result<_, _>>()?;
    f.add("exp.circle_radii", list(&radii, 3))?;
    let climbed = need(Func::Exp.eval(C::new(0.0, TAU)), "exp(2πi)")?;
    f.add("exp.after_2pi", list(&[climbed.re, climbed.im], 6))?;

    // 04 reading colour
    f.add("color.band_ratio", fixed(model::band_ratio(), 2))?;
    f.add("color.square.turns", turns(Func::Square, ZERO, 0.5)?)?;
    f.add("color.recip.turns", turns(Func::Recip, ZERO, 0.5)?)?;
    f.add("color.bump.turns_at_i", turns(Func::Bump, I, 0.5)?)?;
    f.add("color.bump.turns_at_minus_i", turns(Func::Bump, -I, 0.5)?)?;

    // 05 gallery
    f.add("gallery.cubic.turns_r1_8", turns(Func::Cubic, ZERO, 1.8)?)?;
    f.add("gallery.zdp.turns_at_minus1", turns(Func::ZeroDoublePole, -ONE, 0.3)?)?;
    f.add("gallery.zdp.turns_at_1", turns(Func::ZeroDoublePole, ONE, 0.3)?)?;
    f.add("gallery.exp.period", fixed(TAU, 2))?;
    let sin_i = need(Func::Sin.eval(I), "sin i")?;
    let sin_3i = need(Func::Sin.eval(C::new(0.0, 3.0)), "sin 3i")?;
    f.add("gallery.sin.at_i", list(&[sin_i.re, sin_i.im], 2))?;
    f.add("gallery.sin.at_3i", list(&[sin_3i.re, sin_3i.im], 1))?;
    let far_up = need(Func::Tan.eval(model::TAN_FAR), "tan far up")?;
    let far_down = need(Func::Tan.eval(model::TAN_FAR.conj()), "tan far down")?;
    f.add(
        "gallery.tan.far",
        list(&[far_up.re, far_up.im, far_down.re, far_down.im], 4),
    )?;
    let g4 = need(gamma(C::real(4.0)), "Γ(4)")?;
    let g5 = need(gamma(C::real(5.0)), "Γ(5)")?;
    f.add("gallery.gamma.at4", fixed(g4.re, 0))?;
    f.pin("gallery.gamma.at5", fixed(g5.re, 0), "24", "Γ(5) = 4! = 24")?;
    let poles: Vec<f64> = (0..=4)
        .map(|k| -f64::from(k))
        .filter(|&x| gamma(C::real(x)).is_none())
        .collect();
    f.add("gallery.gamma.poles", list(&poles, 0))?;
    let heights: Vec<f64> = ZETA_ZERO_HEIGHTS
        .iter()
        .map(|&t| need(zeta_zero_near(C::new(0.5, t.round())), "a zeta zero").map(|z| z.im))
        .collect::<Result<_, _>>()?;
    f.pin(
        "gallery.zeta.zeros",
        list(&heights, 2),
        "14.13,21.02,25.01,30.42",
        "A. Odlyzko, tables of zeros of the Riemann zeta function (first four ordinates)",
    )?;
    f.add("gallery.zeta.strip_top", fixed(model::ZETA_STRIP_TOP, 0))?;
    let trivial: Vec<f64> = [-2.0, -4.0, -6.0]
        .into_iter()
        .filter(|&x| zeta(C::real(x)).is_some_and(|w| w.abs() < 1e-12))
        .collect();
    f.add("gallery.zeta.trivial", list(&trivial, 0))?;

    // 06 one derivative
    let z0 = C::new(1.0, 1.0);
    let sq = |z: C| Func::Square.eval(z);
    let d = need(quotient(sq, z0, C::polar(1e-7, 0.7)), "z² quotient")?;
    f.add("deriv.value", list(&[d.re, d.im], 3))?;
    f.add("deriv.scale", fixed(d.abs(), 2))?;
    f.add("deriv.turn_deg", fixed(deg(d), 0))?;
    f.add("deriv.gap.side0_5", fixed(square_gap_ratio(z0, 0.5), 3))?;
    f.add("deriv.gap.side0_05", fixed(square_gap_ratio(z0, 0.05), 4))?;
    // the four rates by finite differences of u and v
    let h = 1e-6;
    let along = |dz: C| need(quotient(sq, z0, dz), "a rate");
    let (dx, dy) = (along(C::real(h))?, along(C::new(0.0, h))? * I);
    // dx = u_x + i v_x; i·(quotient up) = u_y + i v_y
    f.add("deriv.jacobian", list(&[dx.re, dy.re, dx.im, dy.im], 3))?;
    let conj = |z: C| Some(z.conj());
    let squash = |z: C| Some(C::new(z.re, 2.0 * z.im));
    let right = |g: &dyn Fn(C) -> Option<C>| need(quotient(g, z0, C::real(1e-3)), "a quotient");
    let up = |g: &dyn Fn(C) -> Option<C>| need(quotient(g, z0, C::new(0.0, 1e-3)), "a quotient");
    f.add("deriv.conj", list(&[right(&conj)?.re, up(&conj)?.re], 0))?;
    f.add("deriv.squash", list(&[right(&squash)?.re, up(&squash)?.re], 0))?;

    // 07 one point
    let rays = [C::real(1.0), I].map(|u| sq(u).map(deg));
    f.add("critical.angle_in", fixed(deg(I) - deg(ONE), 0))?;
    f.add(
        "critical.angle_out",
        fixed(need(rays[1], "ray")? - need(rays[0], "ray")?, 0),
    )?;
    // 07_2: the images of a small step right and a small step up, at z₁
    let z1 = model::CONFORMAL_POINT;
    let image = |h: C| (z1 + h) * (z1 + h) - z1 * z1;
    let kept = image(C::new(0.0, 1e-6)) / image(C::real(1e-6));
    f.add("critical.angle_kept", fixed(kept.arg().to_degrees(), 0))?;
    let abs_sq = |z: C| Func::AbsSquare.eval(z);
    let at0 = need(quotient(abs_sq, ZERO, C::polar(1e-6, 1.1)), "|z|² at 0")?;
    f.add("onepoint.at0", fixed(at0.abs(), 4))?;
    let r1 = need(quotient(abs_sq, ONE, C::real(1e-8)), "|z|² right")?;
    let u1 = need(quotient(abs_sq, ONE, C::new(0.0, 1e-8)), "|z|² up")?;
    f.add("onepoint.at1", list(&[r1.abs(), u1.abs()], 0))?;
    f.add("onepoint.squeeze.radii", list(&model::SQUEEZE_RADII, 2))?;

    // 08 loop sums
    let n = 24;
    let z_total = need(loop_sum(Some, ZERO, 1.0, n), "∮ z")?;
    f.add("loop.z.total", fixed(z_total.abs(), 6))?;
    let inv = need(loop_sum(|z: C| Some(z.recip()), ZERO, 1.0, n), "∮ 1/z")?;
    f.add("loop.inv.total24", fixed(inv.im, 2))?;
    let all_up = need(loop_products(|z: C| Some(z.recip()), ZERO, 2.0, n), "1/z products")?
        .iter()
        .all(|p| p.re.abs() < 1e-12 && p.im > 0.0);
    f.add("loop.inv.all_up", all_up)?;
    f.pin("loop.two_pi", fixed(TAU, 2), "6.28", "2π")?;
    let closed: Vec<f64> = [-3, -2, -1, 0, 1, 2]
        .into_iter()
        .filter(|&p| loop_sum(|z: C| Some(z.powi(p)), ZERO, 1.0, n).is_some_and(|s| s.abs() < 1e-12))
        .map(f64::from)
        .collect();
    f.add("loop.powers.closed", list(&closed, 0))?;

    // 09 the boundary
    let bump_f = |z: C| Func::Bump.eval(z);
    let [few, many] = model::AVERAGE_SAMPLES;
    f.add("cauchy.samples", format!("{few},{many}"))?;
    let a8 = need(circle_average(bump_f, ZERO, 0.5, few), "average r = ½, 8")?;
    let a16 = need(circle_average(bump_f, ZERO, 0.5, many), "average r = ½, 16")?;
    let b16 = need(circle_average(bump_f, ZERO, 2.0, many), "average r = 2, 16")?;
    f.add("cauchy.r_half.n8", fixed(a8.re, 4))?;
    f.add("cauchy.r_half.n16", fixed(a16.re, 6))?;
    f.add("cauchy.r_two.n16", fixed(b16.re, 6))?;
    // each pole's share of the radius-2 average: the residue of f(z)/z there
    let share = (I * (I + I)).recip();
    f.add("cauchy.pole_share", fixed(share.re, 1))?;
    // past both poles: the bump's value at 0 plus one share for each pole
    f.add("cauchy.r_two.limit", fixed(bump(0.0) + 2.0 * share.re, 0))?;
    let coefs: Vec<f64> = (0..=4)
        .map(|k| need(circle_coefficient(bump_f, ZERO, 0.5, many, k), "a coefficient").map(|c| c.re))
        .collect::<Result<_, _>>()?;
    f.add("cauchy.coef", list(&coefs, 3))?;
    let r = bump_radius(1.0);
    f.add("recentre1.radius", fixed(r, 3))?;
    f.add("recentre1.left", fixed(1.0 - r, 2))?;
    f.add("recentre1.right", fixed(1.0 + r, 2))?;
    let (terms, [near, far]) = (model::RECENTRE_CHECK_TERMS, model::RECENTRE_CHECK_AT);
    f.add("recentre1.check.terms", terms)?;
    f.add("recentre1.check.at", format!("{near},{far}"))?;
    f.add("recentre1.check.at2", fixed(bump_recipe(1.0, near, terms), 6))?;
    f.add("recentre1.check.at2_6", fixed(bump_recipe(1.0, far, terms), 0))?;

    // 10 rigidity
    let three_i = C::new(0.0, 3.0);
    let (s3, c3) = (three_i.sin(), three_i.cos());
    let (s2, c2) = (s3 * s3, c3 * c3);
    f.add("rigid.trig", list(&[s2.re, c2.re], 2))?;
    f.add("rigid.trig.sum", fixed((s2 + c2).re, 6))?;
    let pile: Vec<f64> = (1..=3).map(|k| 1.0 / (PI * f64::from(k))).collect();
    f.add("rigid.sin_recip.zeros", list(&pile, 3))?;

    // 11 counting
    let local = [Func::Identity, Func::Square, Func::Recip]
        .into_iter()
        .map(|g| turns(g, ZERO, 0.5))
        .collect::<Result<Vec<_>, _>>()?;
    f.add(
        "argument.local",
        local.iter().map(i64::to_string).collect::<Vec<_>>().join(","),
    )?;
    f.add("argument.rational.r0_8", turns(Func::Moebius, ZERO, 0.8)?)?;
    f.add("argument.rational.r1_7", turns(Func::Moebius, ZERO, 1.7)?)?;
    f.add(
        "argument.rational.r1_35_refused",
        winding_certified(|z| Func::Moebius.eval(z), ZERO, model::MOEBIUS_POLE, 4096, f64::MAX).is_none(),
    )?;
    // 11_8: the coarse loop, z⁸ on the unit circle, refused and then certified
    let coarse = |z: C| Some(z.powi(model::COARSE_POWER));
    let lipschitz = f64::from(model::COARSE_POWER);
    let [few, enough] = model::COARSE_STEPS;
    f.add("argument.coarse.power", model::COARSE_POWER)?;
    f.add("argument.coarse.steps", format!("{few},{enough}"))?;
    f.add(
        "argument.coarse.naive",
        need(
            winding_count(coarse, &circle(ZERO, 1.0, few)),
            "the sampled count of z⁸",
        )?,
    )?;
    f.add("argument.coarse.lipschitz", fixed(lipschitz, 0))?;
    f.add("argument.coarse.size", fixed(ONE.powi(model::COARSE_POWER).abs(), 0))?;
    f.add(
        "argument.coarse.reach",
        list(&[lipschitz * TAU / few as f64, lipschitz * TAU / enough as f64], 2),
    )?;
    f.add(
        "argument.coarse.refused",
        winding_certified(coarse, ZERO, 1.0, few, lipschitz).is_none(),
    )?;
    f.add(
        "argument.coarse.certified",
        need(
            winding_certified(coarse, ZERO, 1.0, enough, lipschitz),
            "the certified count of z⁸",
        )?,
    )?;
    f.add("argument.quintic.unit", turns(Func::Quintic, ZERO, 1.0)?)?;
    f.add("argument.quintic.big", turns(Func::Quintic, ZERO, 3.0)?)?;
    let roots = quintic_roots();
    f.add(
        "argument.quintic.inside",
        roots.iter().filter(|z| z.abs() < 1.0).count(),
    )?;
    f.add("argument.quintic.smallest", fixed(roots[0].abs(), 3))?;

    // 12 residues
    let res_i = (I.scale(2.0)).recip();
    f.add("residue.bump.at_i", list(&[res_i.re, res_i.im], 1))?;

    // 13 a real integral
    let bump_loop = C::new(0.0, TAU) * res_i;
    f.add("integral.bump.loop", fixed(bump_loop.re, 5))?;
    f.add("integral.bump.radii", list(&model::ARC_RADII, 0))?;
    f.add("integral.bump.arc", list(&model::ARC_RADII.map(|r| arc_bound(2, r)), 3))?;
    let degrees: Vec<f64> = quartic_roots().iter().map(|&p| deg(p).rem_euclid(360.0)).collect();
    f.add("integral.quartic.roots_deg", list(&degrees, 0))?;
    let upper = quartic_roots().iter().filter(|p| p.im > 0.0).count();
    f.add("integral.quartic.inside", upper)?;
    let sum = quartic_upper_residue_sum();
    f.add("integral.quartic.sum", list(&[sum.re, sum.im], 5))?;
    let value = (C::new(0.0, TAU) * sum).re;
    f.pin("integral.quartic.value", fixed(value, 4), "2.2214", "π/√2 = 2.22144…")?;
    f.add("integral.quartic.value2", fixed(value, 2))?;
    let quartic_radii = model::QUARTIC_ARC_RADII;
    f.add("integral.quartic.radii", list(&quartic_radii, 0))?;
    f.add("integral.quartic.arc", list(&quartic_radii.map(|r| arc_bound(4, r)), 4))?;
    // "less than one three hundredth": 1/bound at the last radius, down to hundreds
    let last = quartic_radii[quartic_radii.len() - 1];
    f.add(
        "integral.quartic.under",
        fixed((1.0 / arc_bound(4, last) / 100.0).floor() * 100.0, 0),
    )?;
    f.add("integral.quartic.numeric_reach", fixed(model::NUMERIC_REACH, 0))?;
    let numeric = midpoint_integral(|x| 1.0 / (1.0 + x.powi(4)), model::NUMERIC_REACH, 400_000);
    f.add("integral.quartic.numeric", fixed(numeric, 4))?;

    // 14 continuation
    let (top, inside, outside) = (model::SERIES_TOP, model::SERIES_INSIDE, model::SERIES_OUTSIDE);
    f.add("series.terms", top + 1)?;
    f.add("series.half.at", format!("1/{}", fixed(1.0 / inside, 0)))?;
    f.add("series.half.n10", fixed(geometric_partial(inside, top), 6))?;
    f.add("series.half.function", fixed(1.0 / (1.0 - inside), 0))?;
    f.add("series.one_two.at", outside)?;
    f.add("series.one_two.n10", fixed(geometric_partial(outside, top), 2))?;
    f.add("series.one_two.function", fixed(1.0 / (1.0 - outside), 0))?;
    f.add("continuation.first_disk.radius", fixed((ONE - C::real(-0.5)).abs(), 1))?;
    f.add("continuation.two", fixed(1.0 / (1.0 - 2.0), 0))?;
    let z_minus1 = need(zeta(C::real(-1.0)), "ζ(−1)")?;
    f.pin(
        "continuation.zeta_minus1",
        fixed(z_minus1.re, 6),
        "-0.083333",
        "ζ(−1) = −B₂/2 = −1/12",
    )?;
    f.add(
        "continuation.sqrt.end",
        list(&[sqrt_carried(TAU).re, sqrt_carried(TAU).im], 0),
    )?;
    f.add("lacunary.terms", model::LACUNARY_TERMS)?;
    f.add("lacunary.limit", model::LACUNARY_LIMIT)?;

    // 15 closing
    let a3 = model::RECENTRE_AT;
    let r3 = bump_radius(a3);
    f.add("recentre3.centre", fixed(a3, 0))?;
    f.add("recentre3.radius_sq", fixed(r3 * r3, 0))?;
    f.add("recentre3.radius", fixed(r3, 3))?;
    f.add("recentre3.ends", list(&[a3 - r3, a3 + r3], 2))?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Narration truth: what the voice says, on the numbers the film shows.
    #[test]
    fn the_spoken_claims_hold_on_the_shown_numbers() {
        let f = facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}")).to_owned();
        let num = |k: &str| get(k).parse::<f64>().unwrap();
        // 01_3: "about twenty five", "less than one half"
        assert_eq!(get("bump.recipe40.at1_1"), "25.2");
        assert!(num("bump.at1_1") < 0.5);
        // 02_4 / 02_6: "the square root of two", "an eighth of a turn", "two i"
        assert_eq!(get("turn.one_plus_i.angle_deg"), "45");
        assert_eq!(get("turn.square.value"), "0,2");
        // 04_2: "about one point seven"
        assert_eq!(get("color.band_ratio"), "1.69");
        // 04_3, 04_5, 04_7: twice forward, once back, once back at each pole
        assert_eq!(get("color.square.turns"), "2");
        assert_eq!(get("color.recip.turns"), "-1");
        assert_eq!(get("color.bump.turns_at_i"), "-1");
        assert_eq!(get("color.bump.turns_at_minus_i"), "-1");
        // 05_2: a pole of order two runs backwards twice
        assert_eq!(get("gallery.zdp.turns_at_minus1"), "-2");
        // 05_4: "about one point one eight i", "about ten i"
        assert_eq!(get("gallery.sin.at_i"), "0.00,1.18");
        assert_eq!(get("gallery.sin.at_3i"), "0.0,10.0");
        // 06_4: "about two point eight", "an eighth of a turn"
        assert_eq!(get("deriv.scale"), "2.83");
        assert_eq!(get("deriv.turn_deg"), "45");
        // 06_7: "two, minus two, two, and two"
        assert_eq!(get("deriv.jacobian"), "2.000,-2.000,2.000,2.000");
        // 06_9: "one", "minus one"; the squash: 1 and 2
        assert_eq!(get("deriv.conj"), "1,-1");
        assert_eq!(get("deriv.squash"), "1,2");
        // 07_1 and 07_5
        assert_eq!(get("critical.angle_out"), "180");
        assert_eq!(get("onepoint.at1"), "2,0");
        // 08: the 1/z column; only z⁻¹ survives
        assert_eq!(get("loop.inv.all_up"), "true");
        assert_eq!(get("loop.powers.closed"), "-3,-2,0,1,2");
        // 09_3: "within half a percent", "about fifteen millionths"
        assert!((num("cauchy.r_half.n8") - 1.0).abs() < 0.005);
        assert_eq!(get("cauchy.r_half.n16"), "1.000015");
        // 09_5: "it is zero", "each pole pulls the average down by one half"
        assert!(num("cauchy.r_two.n16").abs() < 1e-4);
        assert_eq!(get("cauchy.pole_share"), "-0.5");
        // 09_9: "about minus point four one to about two point four one"
        assert_eq!(get("recentre1.left"), "-0.41");
        assert_eq!(get("recentre1.right"), "2.41");
        // 10_2: "about minus one hundred", "about one hundred and one", "one"
        assert_eq!(get("rigid.trig"), "-100.36,101.36");
        assert_eq!(get("rigid.trig.sum"), "1.000000");
        // 11_3, 11_4, 11_6, 11_7
        assert_eq!(get("argument.rational.r0_8"), "1");
        assert_eq!(get("argument.rational.r1_7"), "0");
        assert_eq!(get("argument.rational.r1_35_refused"), "true");
        // 11_8: "samples alone can miss a turn": z⁸ reads 0, is refused, then counts 8
        assert_eq!(get("argument.coarse.naive"), "0");
        assert_eq!(get("argument.coarse.refused"), "true");
        assert_eq!(get("argument.coarse.reach"), "6.28,0.79");
        assert_eq!(get("argument.coarse.certified"), "8");
        assert_eq!(get("argument.quintic.unit"), "1");
        assert_eq!(get("argument.quintic.inside"), "1");
        assert_eq!(get("argument.quintic.big"), "5");
        // 12_4: one over two i = −i/2
        assert_eq!(get("residue.bump.at_i"), "0.0,-0.5");
        // 13: π; two poles inside; −i√2/4; "about two point two two"; "less than one three hundredth"
        assert_eq!(get("integral.bump.loop"), "3.14159");
        assert_eq!(get("integral.quartic.roots_deg"), "45,135,225,315");
        assert_eq!(get("integral.quartic.inside"), "2");
        assert_eq!(get("integral.quartic.sum"), "0.00000,-0.35355");
        assert_eq!(get("integral.quartic.value2"), "2.22");
        assert!(arc_bound(4, 10.0) < 1.0 / 300.0);
        // 13_10: "At R equals ten ... less than one three hundredth"
        assert_eq!(get("integral.quartic.radii"), "2,10");
        assert_eq!(get("integral.quartic.under"), "300");
        assert_eq!(get("integral.quartic.numeric"), get("integral.quartic.value"));
        // 14: "settle on two", "minus five", "minus one", "minus one twelfth"
        assert_eq!(get("series.one_two.function"), "-5");
        assert_eq!(get("continuation.two"), "-1");
        assert_eq!(get("continuation.sqrt.end"), "-1,0");
        // 15_4: "about three point one six"
        assert_eq!(get("recentre3.radius"), "3.162");
    }

    #[test]
    fn negative_zero_never_prints() {
        assert_eq!(fixed(-1e-17, 3), "0.000");
        assert_eq!(fixed(-0.0004, 3), "0.000");
        assert_eq!(fixed(-0.0006, 3), "-0.001");
        assert_eq!(fixed(-2.0, 0), "-2");
    }
}
