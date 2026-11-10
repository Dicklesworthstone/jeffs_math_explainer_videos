//! The complex rate law of chapters 13 and 14, and the backward readings of chapter 15. Author source, not built
//! or run. Nothing here calls a complex exponential: `on_circle` is the circle, `tangent_steps` is the tiny
//! multipliers and `partial_sums` is the series, so a test can compare three separate constructions.
use explainer_kit::Facts;
use std::f64::consts::PI;

/// A complex number, as the arrow from zero to the point (re, im).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

pub const ONE: C = C::new(1.0, 0.0);
/// The worked case of 13_1: one and a half across and one up, chosen off the axes and inside the plane when turned.
pub const EXAMPLE: C = C::new(1.5, 1.0);

impl C {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn plus(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
    pub fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }
    pub fn times(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    /// A quarter turn counterclockwise: multiplying a + ib by i gives -b + ia.
    pub fn times_i(self) -> Self {
        Self::new(-self.im, self.re)
    }
}

/// The cell counts drawn for the walk to pi.
pub const CELLS: [u32; 4] = [4, 16, 64, 1024];
/// The series terms drawn, counting the leading one as term zero.
pub const TERMS: usize = 16;
/// The rate law's two parts in the last cue: real part `SPIRAL_REAL`, imaginary part one.
pub const SPIRAL_REAL: f64 = -0.15;
/// Where the spiral picture stops: two full turns.
pub const SPIRAL_END: f64 = 4.0 * PI;
/// The input of the backward reading of the tiny multipliers, and its cell counts.
pub const ROOT_INPUT: f64 = 2.0;
pub const ROOT_CELLS: [u32; 5] = [1, 4, 16, 64, 1024];

/// The point at distance t counterclockwise along the unit circle, from one.
pub fn on_circle(t: f64) -> C {
    C::new(t.cos(), t.sin())
}
/// Each step adds a tangent: the current arrow, turned a quarter turn and scaled by the width. n + 1 points.
pub fn tangent_steps(t: f64, n: u32) -> Vec<C> {
    let width = t / f64::from(n);
    let mut z = ONE;
    let mut out = vec![z];
    for _ in 0..n {
        z = z.plus(z.times_i().scale(width));
        out.push(z);
    }
    out
}
pub fn tangent_end(t: f64, n: u32) -> C {
    *tangent_steps(t, n).last().unwrap_or(&ONE)
}
/// The terms (it)^k / k!, each one the last turned a quarter turn and scaled by t / k.
pub fn series_terms(t: f64, count: usize) -> Vec<C> {
    let mut term = ONE;
    let mut out = vec![term];
    for k in 1..count {
        term = term.times_i().scale(t / k as f64);
        out.push(term);
    }
    out
}
/// The partial sums, tip to tail, starting from zero: `count + 1` points.
pub fn partial_sums(t: f64, count: usize) -> Vec<C> {
    let mut at = C::new(0.0, 0.0);
    let mut out = vec![at];
    for term in series_terms(t, count) {
        at = at.plus(term);
        out.push(at);
    }
    out
}
/// Rate a + i for time t, as a stretch and a turn.
pub fn spiral(t: f64) -> C {
    on_circle(t).scale((SPIRAL_REAL * t).exp())
}
/// The rate that gives x after n cells: solving (1 + u/n)^n = x for u.
pub fn backward_rate(x: f64, n: u32) -> f64 {
    let n = f64::from(n);
    n * (x.powf(1.0 / n) - 1.0)
}

/// Five places, with no minus sign on a value that rounds to zero.
pub fn five(x: f64) -> String {
    let s = format!("{x:.5}");
    if s == "-0.00000" { "0.00000".to_owned() } else { s }
}
pub fn point_label(z: C) -> String {
    format!("{} {}", five(z.re), five(z.im))
}

pub fn append_facts(f: &mut Facts) -> Result<(), String> {
    let mut z = ONE;
    for k in 0..4 {
        f.add(&format!("turn.quarter.{k}"), format!("{} {}", z.re as i32, z.im as i32))?;
        z = z.times_i();
    }
    f.add("turn.half", point_label(on_circle(PI)))?;
    f.add("turn.pi", five(PI))?;
    f.add("turn.full_lap", five(2.0 * PI))?;
    for n in CELLS {
        let end = tangent_end(PI, n);
        f.add(&format!("turn.cells.{n}.end"), point_label(end))?;
        f.add(&format!("turn.cells.{n}.abs"), five(end.abs()))?;
        f.add(&format!("turn.cells.{n}.miss"), five(end.plus(ONE).abs()))?;
    }
    for (k, term) in series_terms(PI, TERMS).into_iter().enumerate() {
        f.add(&format!("turn.term.{k}"), five(term.abs()))?;
    }
    for (k, at) in partial_sums(PI, TERMS).into_iter().enumerate() {
        f.add(&format!("turn.partial.{k}"), point_label(at))?;
    }
    f.add("bridge.value", five(ROOT_INPUT.ln()))?;
    for n in ROOT_CELLS {
        f.add(&format!("bridge.root.{n}"), five(backward_rate(ROOT_INPUT, n)))?;
    }
    // Stage10, review 721: the spoken worked case of 13_1 (M4), and how many of E's units one slide-rule unit spans,
    // which is why 15_2 calls the slide rule's unit longer (H3).
    f.add("turn.example", point_label(EXAMPLE))?;
    f.add("turn.example_turned", point_label(EXAMPLE.times_i()))?;
    f.add("bridge.unit_ratio", five(10.0_f64.ln()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplying_by_i_is_the_quarter_turn() {
        let (a, b) = (1.3, -0.4);
        let z = C::new(a, b);
        assert_eq!(z.times_i(), C::new(-b, a));
        assert_eq!(z.times(C::new(0.0, 1.0)), z.times_i());
        // Same length, at a right angle.
        assert!((z.times_i().abs() - z.abs()).abs() < 1e-15);
        assert!((z.re * z.times_i().re + z.im * z.times_i().im).abs() < 1e-15);
    }
    #[test]
    fn i_squared_is_minus_one_and_the_worked_case_turns_as_spoken() {
        let i = C::new(0.0, 1.0);
        assert_eq!(i.times(i), C::new(-1.0, 0.0));
        assert_eq!(EXAMPLE.times_i(), C::new(-1.0, 1.5));
        assert_eq!(point_label(EXAMPLE.times_i()), "-1.00000 1.50000");
        // Both arrows, (1.5, 1) and (-1, 1.5), stay inside the 13_1 plane: 1.45 scene units per unit, 2.55 up and down.
        const { assert!(EXAMPLE.re * 1.45 < 2.55 && EXAMPLE.im * 1.45 < 2.55) };
    }
    #[test]
    fn a_slide_rule_unit_is_longer_than_the_exponential_unit() {
        // One base-ten unit of the slide rule spans ln 10 of E's inputs, and a position in E's unit is ln x.
        for x in [2.0_f64, 3.0, 6.0] {
            assert!((x.log10() * 10.0_f64.ln() - x.ln()).abs() < 1e-14);
            assert!(x.ln() > x.log10(), "{x}");
        }
    }
    #[test]
    fn the_circle_solves_the_rate_law_and_keeps_its_distance_and_speed() {
        for k in 0..=64 {
            let t = f64::from(k) / 8.0;
            let h = 1e-6;
            let (p, m) = (on_circle(t + h), on_circle(t - h));
            let velocity = C::new((p.re - m.re) / (2.0 * h), (p.im - m.im) / (2.0 * h));
            let law = on_circle(t).times_i();
            assert!(
                (velocity.re - law.re).abs() < 1e-8 && (velocity.im - law.im).abs() < 1e-8,
                "{t}"
            );
            assert!((on_circle(t).abs() - 1.0).abs() < 1e-15 && (velocity.abs() - 1.0).abs() < 1e-8);
        }
    }
    #[test]
    fn three_constructions_land_on_minus_one() {
        assert!(on_circle(PI).plus(ONE).abs() < 1e-15);
        let misses: Vec<f64> = CELLS.iter().map(|&n| tangent_end(PI, n).plus(ONE).abs()).collect();
        assert!(misses.windows(2).all(|p| p[1] < p[0]));
        assert!(misses[3] < 0.005);
        assert!(partial_sums(PI, 30).last().unwrap().plus(ONE).abs() < 1e-12);
    }
    #[test]
    fn every_tangent_step_leaves_the_circle_outward_by_a_shrinking_amount() {
        for n in CELLS {
            let width = PI / f64::from(n);
            let points = tangent_steps(PI, n);
            assert!(points.windows(2).all(|p| p[1].abs() > p[0].abs()));
            let per_step = points[1].abs() - 1.0;
            assert!((per_step - ((1.0 + width * width).sqrt() - 1.0)).abs() < 1e-12);
        }
    }
    #[test]
    fn the_series_arrows_grow_while_k_is_under_pi_then_shrink() {
        let sizes: Vec<f64> = series_terms(PI, TERMS).iter().map(|z| z.abs()).collect();
        assert!(sizes[..=3].windows(2).all(|p| p[1] > p[0]));
        assert!(sizes[3..].windows(2).all(|p| p[1] < p[0]));
        for (k, pair) in series_terms(PI, TERMS).windows(2).enumerate() {
            let expected = pair[0].times_i().scale(PI / (k + 1) as f64);
            assert!((pair[1].re - expected.re).abs() < 1e-12 && (pair[1].im - expected.im).abs() < 1e-12);
        }
    }
    #[test]
    fn the_series_real_and_imaginary_parts_are_cosine_and_sine() {
        for k in 0..=16 {
            let t = f64::from(k) / 4.0;
            let end = *partial_sums(t, 40).last().unwrap();
            assert!(
                (end.re - t.cos()).abs() < 1e-12 && (end.im - t.sin()).abs() < 1e-12,
                "{t}"
            );
        }
    }
    #[test]
    fn multiplying_circle_points_adds_their_angles() {
        for (s, t) in [(0.3, 1.1), (2.0, 4.0), (-1.2, 0.7)] {
            let product = on_circle(s).times(on_circle(t));
            let sum = on_circle(s + t);
            assert!((product.re - sum.re).abs() < 1e-14 && (product.im - sum.im).abs() < 1e-14);
        }
    }
    #[test]
    fn the_spiral_splits_into_a_stretch_and_a_turn() {
        for k in 0..=20 {
            let t = SPIRAL_END * f64::from(k) / 20.0;
            assert!((spiral(t).abs() - (SPIRAL_REAL * t).exp()).abs() < 1e-14);
        }
    }
    #[test]
    fn the_backward_rate_approaches_the_logarithm_from_above() {
        let rates: Vec<f64> = ROOT_CELLS.iter().map(|&n| backward_rate(ROOT_INPUT, n)).collect();
        assert!(rates.windows(2).all(|p| p[1] < p[0]));
        assert!(rates.iter().all(|&r| r > ROOT_INPUT.ln()));
        assert!(rates[4] - ROOT_INPUT.ln() < 0.0003);
        for n in ROOT_CELLS {
            let u = backward_rate(ROOT_INPUT, n);
            assert!(((1.0 + u / f64::from(n)).powf(f64::from(n)) - ROOT_INPUT).abs() < 1e-12);
        }
    }
    #[test]
    fn a_rounded_zero_has_no_sign() {
        assert_eq!(five(-1e-9), "0.00000");
        assert_eq!(five(0.0), "0.00000");
        assert_eq!(five(-0.5), "-0.50000");
    }
}
