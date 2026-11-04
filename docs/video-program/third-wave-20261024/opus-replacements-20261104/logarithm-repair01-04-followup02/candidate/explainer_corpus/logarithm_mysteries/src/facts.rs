//! Every number the logarithm film shows or says, computed from `model`, in
//! the order and format `oracle.py` prints (`./x facts-check`). The tests
//! below test each spoken always/never claim at its edges.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled here.

use std::f64::consts::PI;

use explainer_kit::Facts;

use crate::model::{self, C};

/// `x` to `places` decimals, never `-0.0…` (oracle.py's `dec`).
pub fn dec(x: f64, places: usize) -> String {
    let s = format!("{x:.places$}");
    if s.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') {
        s.trim_start_matches('-').to_owned()
    } else {
        s
    }
}

/// A short number: integers without a point, otherwise the shortest form
/// (oracle.py's `num`).
pub fn num(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{}", x as i64)
    } else {
        format!("{x}")
    }
}

/// A fraction in lowest terms, `p/q` (or `p` when q = 1).
pub fn frac(p: i64, q: i64) -> String {
    fn gcd(a: i64, b: i64) -> i64 {
        if b == 0 { a.abs() } else { gcd(b, a % b) }
    }
    let g = gcd(p, q).max(1);
    let (p, q) = (p / g * q.signum(), q.abs() / g);
    if q == 1 { p.to_string() } else { format!("{p}/{q}") }
}

/// The seven A's, A1..A7.
pub fn a_frequencies() -> Vec<f64> {
    (0..7).map(|k| model::key_hz(12 * (k - 3))).collect()
}

/// The slide-rule examples.
pub const SLIDE_A: f64 = 2.5;
pub const SLIDE_B: f64 = 3.2;
pub const SLIDE_C: f64 = 4.0;
pub const SLIDE_D: f64 = 6.0;
/// The continuous rate of the doubling example, per year.
pub const RATE: f64 = 0.05;
/// 05_1's arrows: (length, degrees).
pub const MUL_Z: (f64, f64) = (1.2, 30.0);
pub const MUL_W: (f64, f64) = (1.5, 50.0);
/// 05_7's generic number: length 2 at 0.6π.
pub const GENERIC: (f64, f64) = (2.0, 0.6);
/// 06_4's off-centre loop: centre on the real axis and radius.
pub const SIDE_LOOP: (f64, f64) = (1.7, 0.8);
/// 08's angle, in units of π (numerator, denominator).
pub const PRODUCT_ANGLE: (i64, i64) = (3, 4);
/// 09_4's three speeds, in turns per unit of time.
pub const QUARTER_SPEEDS: [f64; 3] = [0.25, 1.25, -0.75];
/// 10_2's two ellipses: semi-axes and tilt in degrees.
pub const ELLIPSES: [(f64, f64, f64); 2] = [(2.0, 1.0, 0.0), (1.6, 0.7, 30.0)];
/// 11_2's generic flow.
pub const FLOW: model::M2 = [[0.3, -1.0], [0.8, -0.1]];
/// 11_3's flip pair, diag(−2, −½).
pub const FLIP_PAIR: (f64, f64) = (-2.0, -0.5);
/// 11_4/11_5's probe vector, with two nonzero parts.
pub const PROBE: [f64; 2] = [1.0, 1.0];
/// 11_5's candidate X, whose off-diagonal entries shrink to zero on screen.
pub const PROBE_CANDIDATE: model::M2 = [[0.4, 0.6], [-0.5, -0.3]];
/// 12_4–12_6's spins about the rod, in laps per unit time, run on screen.
pub const SPIN_RATES: [f64; 4] = [0.5, 1.5, 2.5, -0.5];

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();

    // --- music: equal temperament, A4 = 440 Hz
    let a = a_frequencies();
    let joined = |v: &[f64]| v.iter().map(|&x| num(x)).collect::<Vec<_>>().join(",");
    f.pin(
        "music.freqs_hz",
        joined(&a),
        "55,110,220,440,880,1760,3520",
        "ISO 16 standard pitch A4 = 440 Hz, octaves 2:1",
    )?;
    let top = a[6];
    f.add("music.ruler_top_hz", num(top))?;
    f.add("music.first_four_share", frac(a[3] as i64, top as i64))?;
    f.add("music.last_step_share", frac((a[6] - a[5]) as i64, top as i64))?;
    f.add("music.keys_per_octave", 12)?;
    f.add("music.start_hz", num(a[3]))?;
    f.add("music.half_up_keys", 6)?;
    let half = model::key_hz(6);
    f.add("music.half_octave_hz", dec(half, 2))?;
    f.add("music.half_octave_spoken", half.round() as i64)?;
    f.add("music.linear_mid_hz", num((a[3] + a[4]) / 2.0))?;
    f.add("music.root_two", dec(2f64.sqrt(), 3))?;
    let nineteen = model::key_hz(19) / model::key_hz(0);
    f.add("music.nineteen_keys_ratio", dec(nineteen, 3))?;
    f.add("music.nineteen_keys_hz", dec(model::key_hz(19), 2))?;
    f.add("music.three_times_hz", num(3.0 * a[3]))?;

    // --- the ruler: where does three go?
    f.add("ruler.three_pow", 12)?;
    f.add("ruler.two_pow", 19)?;
    let (p3, p2) = (3_u64.pow(12), 2_u64.pow(19));
    f.pin("ruler.three_pow12", p3, "531441", "integer arithmetic")?;
    f.pin("ruler.two_pow19", p2, "524288", "integer arithmetic")?;
    f.add("ruler.power_gap_percent", dec((p3 as f64 / p2 as f64 - 1.0) * 100.0, 2))?;
    f.add("ruler.nineteen_twelfths", dec(19.0 / 12.0, 3))?;
    f.add("ruler.log2_3", dec(model::position(3.0, 2.0), 3))?;
    f.add("ruler.log2_8", num(model::position(8.0, 2.0).round()))?;
    f.add(
        "ruler.log2_sqrt2",
        num((model::position(2f64.sqrt(), 2.0) * 1e12).round() / 1e12),
    )?;
    f.add("ruler.log2_half", num(model::position(0.5, 2.0)))?;
    f.add("ruler.change_base_mid", 2_u32.pow(2))?;
    f.add("ruler.change_base_final", 10)?;

    // --- the slide rule (decade scales, positions in decades)
    let p = |x: f64| model::position(x, 10.0);
    f.add("slide.a", num(SLIDE_A))?;
    f.add("slide.b", num(SLIDE_B))?;
    f.add("slide.ab", num((SLIDE_A * SLIDE_B * 1e9).round() / 1e9))?;
    f.add("slide.pos_a", dec(p(SLIDE_A), 3))?;
    f.add("slide.pos_b", dec(p(SLIDE_B), 3))?;
    f.add("slide.pos_ab", dec(p(SLIDE_A * SLIDE_B), 3))?;
    f.add("slide.c", num(SLIDE_C))?;
    f.add("slide.d", num(SLIDE_D))?;
    let cd = SLIDE_C * SLIDE_D;
    f.add("slide.cd", num(cd))?;
    let (digits, laps) = model::fold_decade(cd);
    f.add("slide.cd_digits", num((digits * 1e9).round() / 1e9))?;
    f.add("slide.pos_c", dec(p(SLIDE_C), 3))?;
    f.add("slide.pos_d", dec(p(SLIDE_D), 3))?;
    f.add("slide.pos_cd", dec(p(cd), 3))?;
    f.add("slide.pos_cd_folded", dec(p(cd) - f64::from(laps), 3))?;
    f.add("slide.cd_laps", laps)?;

    // --- the dial
    let values = [0.24, 2.4, 24.0, 240.0];
    f.add("dial.values", joined(&values))?;
    let laps: Vec<String> = values.iter().map(|&x| model::fold_decade(x).1.to_string()).collect();
    f.add("dial.laps", laps.join(","))?;
    f.add("dial.mark", dec(p(2.4), 3))?;

    // --- the real logarithm
    f.add("real.e", dec(std::f64::consts::E, 3))?;
    let (product_a, product_b) = (2_u32, 4_u32);
    f.add("real.product_a", product_a)?;
    f.add("real.product_b", product_b)?;
    f.add("real.product_ab", product_a * product_b)?;
    let growth_target = 10_u32.pow(3);
    f.add("real.growth_target", growth_target)?;
    f.add("real.ln_one_point_oh_one", dec(1.01_f64.ln(), 5))?;
    f.add("real.rate", num(RATE))?;
    f.add("real.ln2", dec(2f64.ln(), 3))?;
    f.add("real.double_years", dec(2f64.ln() / RATE, 1))?;
    f.add("real.ln_thousand", dec(f64::from(growth_target).ln(), 2))?;
    f.add(
        "real.thousand_years",
        (f64::from(growth_target).ln() / RATE).round() as i64,
    )?;
    f.add("real.two_pow10", 2_u32.pow(10))?;
    f.add("real.ten_doublings_years", dec(10.0 * 2f64.ln() / RATE, 1))?;
    f.add("real.rule_of_seventy", dec(100.0 * 2f64.ln(), 1))?;

    // --- complex numbers
    let polar_deg = |(r, d): (f64, f64)| C::polar(r, d.to_radians());
    let zw = polar_deg(MUL_Z).mul(polar_deg(MUL_W));
    f.add("complex.mul_z", format!("{},{}", num(MUL_Z.0), num(MUL_Z.1)))?;
    f.add("complex.mul_w", format!("{},{}", num(MUL_W.0), num(MUL_W.1)))?;
    f.add(
        "complex.mul_zw",
        format!("{},{}", dec(zw.abs(), 1), zw.arg().to_degrees().round() as i64),
    )?;
    let minus_one: Vec<String> = (-2..2).map(|k| (2 * k + 1).to_string()).collect();
    f.add("complex.minus_one_logs_pi", minus_one.join(","))?;
    let one: Vec<String> = (-1..3).map(|k| (2 * k).to_string()).collect();
    f.add("complex.one_logs_pi", one.join(","))?;
    f.add("complex.generic", format!("{},{}", num(GENERIC.0), num(GENERIC.1)))?;
    f.add(
        "complex.generic_ln_len",
        dec(C::polar(GENERIC.0, GENERIC.1 * PI).abs().ln(), 3),
    )?;

    // --- carrying the logarithm around loops
    let unit = model::circle_path(C::new(0.0, 0.0), 1.0, 1.0, 2000);
    let side = model::circle_path(C::new(SIDE_LOOP.0, 0.0), SIDE_LOOP.1, 1.0, 2000);
    f.add("loop.unit_winding", model::winding(&unit))?;
    f.add("loop.side_winding", model::winding(&side))?;
    f.add("loop.side", format!("{},{}", num(SIDE_LOOP.0), num(SIDE_LOOP.1)))?;
    let l = model::lift(&unit);
    let climb = [l[500].im, l[1000].im, l[2000].im].map(|y| num((y / PI * 1e9).round() / 1e9));
    f.add("loop.climb_pi", climb.join(","))?;

    // --- the product rule (exact, in units of π)
    let (pn, pd) = PRODUCT_ANGLE;
    let sum = 2 * pn;
    let principal = model::principal_units(sum, pd);
    f.add("product.angle_pi", frac(pn, pd))?;
    f.add("product.sum_pi", frac(sum, pd))?;
    f.add("product.principal_pi", frac(principal, pd))?;
    f.add("product.lap_pi", frac(sum - principal, pd))?;
    let minus = C::new(-1.0, 0.0);
    f.add("sqrt.product_of_roots", dec(minus.sqrt().mul(minus.sqrt()).re, 0))?;
    f.add("sqrt.root_of_product", dec(minus.mul(minus).sqrt().re, 0))?;

    // --- matrices
    let speeds: Vec<String> = QUARTER_SPEEDS.iter().map(|&s| num(s)).collect();
    f.add("matrix.quarter_turn_speeds", speeds.join(","))?;
    let ellipses: Vec<String> = ELLIPSES
        .iter()
        .map(|&(a, b, t)| format!("{},{},{}", num(a), num(b), num(t)))
        .collect();
    f.add("matrix.ellipses", ellipses.join(";"))?;
    let home = ELLIPSES
        .iter()
        .all(|&(a, b, t)| model::mdist(model::mexp(model::ellipse_generator(a, b, t.to_radians())), model::I2) < 1e-9);
    f.add("matrix.ellipses_return", if home { "yes" } else { "no" })?;
    f.add(
        "matrix.flow",
        format!(
            "{},{};{},{}",
            num(FLOW[0][0]),
            num(FLOW[0][1]),
            num(FLOW[1][0]),
            num(FLOW[1][1])
        ),
    )?;
    f.add("matrix.flow_trace", num((model::trace(FLOW) * 1e12).round() / 1e12))?;
    f.add("matrix.flow_det_t1", dec(model::det(model::flow(FLOW, 1.0)), 3))?;
    f.add("matrix.flow_det_t3", dec(model::det(model::flow(FLOW, 3.0)), 3))?;
    f.add("matrix.mirror_det", num(model::det([[-1.0, 0.0], [0.0, 1.0]])))?;
    f.add("matrix.flip_pair", format!("{},{}", num(FLIP_PAIR.0), num(FLIP_PAIR.1)))?;
    f.add("matrix.flip_pair_det", num(FLIP_PAIR.0 * FLIP_PAIR.1))?;
    f.add(
        "matrix.flip_pair_log_real",
        format!("{},{}", dec(FLIP_PAIR.0.abs().ln(), 3), dec(FLIP_PAIR.1.abs().ln(), 3)),
    )?;
    f.add("matrix.half_turn_speed", "0.5")?;
    // 10_3: the spiral flow turns 0.872 per unit time (its eigenvalues' imaginary part)
    f.add("matrix.flow_turning", dec(model::turning_rate(FLOW), 3))?;
    f.add("matrix.flow_turning_pi", dec(model::turning_rate(FLOW) / PI, 2))?;
    // 11_5: each part of the commuting gap is an off-diagonal entry times this
    f.add("matrix.flip_pair_gap", num(FLIP_PAIR.1 - FLIP_PAIR.0))?;
    // 11_1: the plain average of the identity and the mirror is flat
    let mirror = [[-1.0, 0.0], [0.0, 1.0]];
    f.add(
        "matrix.mirror_average_det",
        num(model::det(model::mscale(model::madd(model::I2, mirror), 0.5))),
    )?;
    // 11_4/11_5: the probe, the candidate X, and the gap between its two routes
    f.add("matrix.probe", format!("{},{}", num(PROBE[0]), num(PROBE[1])))?;
    let p = PROBE_CANDIDATE;
    f.add(
        "matrix.probe_candidate",
        format!("{},{};{},{}", num(p[0][0]), num(p[0][1]), num(p[1][0]), num(p[1][1])),
    )?;
    let flip = [[FLIP_PAIR.0, 0.0], [0.0, FLIP_PAIR.1]];
    let gap = model::apply(model::commutator(PROBE_CANDIDATE, flip), PROBE);
    f.add("matrix.probe_gap", dec(gap[0].hypot(gap[1]), 2))?;
    // 12_4–12_6: each shown spin rate ends, at time 1, in the half turn
    for r in SPIN_RATES {
        let (s, c) = (2.0 * PI * r).sin_cos();
        if (c + 1.0).abs() > 1e-12 || s.abs() > 1e-12 {
            return Err(format!("puzzle.spin_rates: {r} laps does not end in a half turn"));
        }
    }
    let rates: Vec<String> = SPIN_RATES.iter().map(|&r| num(r)).collect();
    f.add("puzzle.spin_rates", rates.join(","))?;
    // 05_1–05_2 (Mail 803 repair 5): i × i, and the worked e^{ln 2 + iπ/2}
    let i = C::new(0.0, 1.0);
    f.add("complex.i_times_i", num(i.mul(i).re))?;
    let worked = C::new(2f64.ln(), PI / 2.0).exp();
    if worked.re.abs() > 1e-12 || (worked.im - 2.0).abs() > 1e-12 {
        return Err("complex.worked: e^(ln 2 + iπ/2) is not 2i".into());
    }
    f.add(
        "complex.worked",
        format!(
            "{},{}",
            num(worked.abs().round()),
            num(worked.arg().to_degrees().round())
        ),
    )?;
    // 06_6: the candidate keeping its angle in (−π, π] minus the carried
    // logarithm, in laps, just before and just after the walk passes −1
    let gap = |t: f64| ((model::wrap(2.0 * PI * t) - 2.0 * PI * t) / (2.0 * PI)).round();
    f.add("loop.principal_gap", format!("{},{}", num(gap(0.49)), num(gap(0.51))))?;
    // 07_2: Log i and Log(−i), imaginary parts in units of π
    f.add(
        "branch.log_i_pi",
        format!("{},{}", num(i.log().im / PI), num(C::new(0.0, -1.0).log().im / PI)),
    )?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn fact(key: &str) -> String {
        facts()
            .unwrap()
            .get(key)
            .unwrap_or_else(|| panic!("no fact {key}"))
            .to_owned()
    }

    #[test]
    fn the_facts_compute_and_self_check() {
        let f = facts().unwrap();
        assert_eq!(f.len(), 99, "oracle.py prints 99 keys");
    }

    #[test]
    fn repaired_ruler_labels_describe_the_same_geometry() {
        let read = |key: &str| fact(key).parse::<f64>().unwrap();
        assert_eq!(read("ruler.change_base_mid"), 4.0);
        assert_eq!(read("ruler.change_base_final"), 10.0);
        let a = read("real.product_a");
        let b = read("real.product_b");
        let ab = read("real.product_ab");
        assert_eq!((a, b, ab), (2.0, 4.0, 8.0));
        assert!((a.ln() + b.ln() - ab.ln()).abs() < 1e-12);
        assert_eq!(read("real.growth_target"), 1000.0);
    }

    /// 00_2: "the last step alone fills half the ruler; the first four notes
    /// crowd into the first eighth" — exactly, not roughly.
    #[test]
    fn the_linear_ruler_crowds_the_low_notes() {
        assert_eq!(fact("music.last_step_share"), "1/2");
        assert_eq!(fact("music.first_four_share"), "1/8");
    }

    /// 00_5: the half-octave note is the geometric, not arithmetic, middle,
    /// and the spoken "about six hundred and twenty-two" rounds the shown value.
    #[test]
    fn six_keys_up_is_the_square_root_of_two() {
        let h = key_hz(6);
        assert!((h * h - 440.0 * 880.0).abs() < 1e-6);
        assert_eq!(fact("music.half_octave_spoken"), "622");
        assert!(fact("music.half_octave_hz").starts_with("622."));
        assert!(h < 660.0 - 30.0, "the guess is far off, not a rounding");
    }

    /// 01_4 / 01_5: "less than one and a half percent apart"; "nineteen keys
    /// above any A is almost exactly three times its frequency".
    #[test]
    fn powers_pin_three_between_two_and_four() {
        let gap: f64 = fact("ruler.power_gap_percent").parse().unwrap();
        assert!(gap > 0.0 && gap < 1.5);
        let r: f64 = fact("music.nineteen_keys_ratio").parse().unwrap();
        assert!((r - 3.0).abs() / 3.0 < 0.002);
        let p3 = position(3.0, 2.0);
        assert!(p3 > 1.0 && p3 < 2.0);
        assert!((p3 - 19.0 / 12.0).abs() < 0.002);
    }

    /// 02_3 and 02_5: the slide adds positions; past ten it folds a decade.
    #[test]
    fn the_slide_rule_examples_read_as_spoken() {
        assert_eq!(fact("slide.ab"), "8");
        assert_eq!(fact("slide.cd"), "24");
        assert_eq!(fact("slide.cd_digits"), "2.4");
        assert_eq!(fact("slide.cd_laps"), "1");
        // the shown three-place positions add on screen as well
        let (a, b, ab): (f64, f64, f64) = (
            fact("slide.pos_a").parse().unwrap(),
            fact("slide.pos_b").parse().unwrap(),
            fact("slide.pos_ab").parse().unwrap(),
        );
        assert!((a + b - ab).abs() < 1e-9);
        let (c, d, cd): (f64, f64, f64) = (
            fact("slide.pos_c").parse().unwrap(),
            fact("slide.pos_d").parse().unwrap(),
            fact("slide.pos_cd").parse().unwrap(),
        );
        assert!((c + d - cd).abs() < 1e-9);
        // 03_2: "four sits a little past half a lap; six adds about three quarters"
        assert!(c > 0.5 && c < 0.65);
        assert!((d - 0.75).abs() < 0.05);
    }

    /// 04_2: "every positive number is reached exactly once" — strictly
    /// increasing, positive, and both ends unbounded on a sample.
    #[test]
    fn the_exponential_hits_each_positive_number_once() {
        let xs: Vec<f64> = (-400..=400).map(|k| f64::from(k) / 20.0).collect();
        for w in xs.windows(2) {
            assert!(w[0].exp() < w[1].exp());
        }
        assert!((-20f64).exp() > 0.0 && (-20f64).exp() < 1e-8);
        assert!(20f64.exp() > 1e8);
        for y in [1e-6, 0.5, 1.0, std::f64::consts::E, 1e6] {
            assert!((y.ln().exp() - y).abs() / y < 1e-12);
        }
    }

    /// 04_4: "the natural logarithm of one point zero one is almost exactly
    /// zero point zero one" (within one percent of it).
    #[test]
    fn small_ratios_become_small_steps() {
        let v: f64 = fact("real.ln_one_point_oh_one").parse().unwrap();
        assert!((v - 0.01).abs() / 0.01 < 0.01);
    }

    /// 04_7: thirteen point nine years; a hundred and thirty-eight; "close to
    /// ten doublings, since two to the tenth is just over a thousand".
    #[test]
    fn doubling_and_thousandfold_times() {
        assert_eq!(fact("real.double_years"), "13.9");
        assert_eq!(fact("real.thousand_years"), "138");
        // both thousandfold numbers follow the same target as the graph's label
        let target: f64 = fact("real.growth_target").parse().unwrap();
        let ln_target: f64 = fact("real.ln_thousand").parse().unwrap();
        assert!((target.ln() - ln_target).abs() < 0.005);
        assert_eq!(
            fact("real.thousand_years"),
            ((target.ln() / RATE).round() as i64).to_string()
        );
        let ten: f64 = fact("real.ten_doublings_years").parse().unwrap();
        assert!(ten > 138.0 && ten < 139.0);
        let two_pow10: u32 = fact("real.two_pow10").parse().unwrap();
        assert!(two_pow10 > 1000 && two_pow10 < 1030);
    }

    /// 05_6 and 05_7: every lap of the column lands on the same number; zero
    /// is never an output.
    #[test]
    fn columns_of_logarithms() {
        for m in fact("complex.minus_one_logs_pi").split(',') {
            let m: f64 = m.parse().unwrap();
            assert!(C::new(0.0, m * PI).exp().add(C::new(1.0, 0.0)).abs() < 1e-12);
        }
        for x in [-50.0, -5.0, 0.0, 5.0] {
            for y in [0.0, 1.0, 3.0] {
                assert!(C::new(x, y).exp().abs() > 0.0);
            }
        }
    }

    /// 06_3 and 06_4: one lap climbs two pi; the side loop returns.
    #[test]
    fn loops_and_winding_numbers() {
        assert_eq!(fact("loop.climb_pi"), "0.5,1,2");
        assert_eq!(fact("loop.unit_winding"), "1");
        assert_eq!(fact("loop.side_winding"), "0");
        // the side loop really stays clear of zero
        assert!(SIDE_LOOP.0 - SIDE_LOOP.1 > 0.5);
    }

    /// 08_2 and 08_5: one lap of difference; the puzzle's two sides.
    #[test]
    fn the_product_rule_keeps_a_lap_count() {
        assert_eq!(fact("product.sum_pi"), "3/2");
        assert_eq!(fact("product.principal_pi"), "-1/2");
        assert_eq!(fact("product.lap_pi"), "2");
        assert_eq!(fact("sqrt.product_of_roots"), "-1");
        assert_eq!(fact("sqrt.root_of_product"), "1");
        // the lap correction is only ever −1, 0 or +1 laps for principal logs
        for p in -7..=8 {
            for q in -7..=8 {
                let (a, b) = (principal_units(p, 8), principal_units(q, 8));
                let lap = a + b - principal_units(a + b, 8);
                assert!([-16, 0, 16].contains(&lap), "{p} {q}");
            }
        }
    }

    /// 10_2: "a whole continuous family" — a sweep of ellipses all return,
    /// and their generators differ from one another.
    #[test]
    fn the_identity_has_a_continuum_of_logarithms() {
        let mut previous: Option<M2> = None;
        for k in 0..=20 {
            let a = 1.0 + f64::from(k) / 10.0;
            let x = ellipse_generator(a, 1.0, 0.0);
            assert!(mdist(mexp(x), I2) < 1e-9);
            if let Some(p) = previous {
                assert!(mdist(p, x) > 1e-3);
            }
            previous = Some(x);
        }
    }

    /// 11_2 and 11_6: negative determinant never; diag(−2, −½) has a complex
    /// logarithm diag(ln 2 + iπ, −ln 2 + iπ), checked entrywise in C.
    #[test]
    fn unreachable_by_real_flows_reachable_in_complex() {
        assert_eq!(fact("matrix.mirror_det"), "-1");
        assert_eq!(fact("matrix.flip_pair_det"), "1");
        let first = C::new(2f64.ln(), PI).exp();
        let second = C::new(-(2f64.ln()), PI).exp();
        assert!(first.sub(C::new(-2.0, 0.0)).abs() < 1e-12);
        assert!(second.sub(C::new(-0.5, 0.0)).abs() < 1e-12);
        let t3: f64 = fact("matrix.flow_det_t3").parse().unwrap();
        assert!(t3 > 0.0);
    }
}
