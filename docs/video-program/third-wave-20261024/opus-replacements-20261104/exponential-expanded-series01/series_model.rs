//! Bounded numerical picture model, not a proof from sampled curves.
pub const ORDERS: [usize; 5] = [0, 1, 2, 3, 4];
pub const PLOT_SEGMENTS: usize = 60;
pub const RADIUS: f64 = 1.5;
pub const PLOT_TOP: f64 = 5.0;
pub const TAIL_ORDER: usize = 4;
pub fn plot_x(index: usize) -> f64 {
    assert!(index <= PLOT_SEGMENTS);
    RADIUS * index as f64 / PLOT_SEGMENTS as f64
}
pub fn factorial(n: usize) -> u64 {
    assert!(n <= 20);
    (1..=n as u64).product()
}
pub fn sum(x: f64, n: usize) -> f64 {
    let mut term = 1.0;
    let mut total = term;
    for k in 1..=n {
        term *= x / k as f64;
        total += term;
    }
    total
}
/// Uniform bound for |x|<=radius after degree n. The first omitted term is
/// radius^(n+1)/(n+1)!; every subsequent ratio is <=radius/(n+2).
pub fn tail_bound(radius: f64, n: usize) -> Option<f64> {
    if !radius.is_finite() || radius < 0.0 || n > 18 {
        return None;
    }
    let ratio = radius / (n + 2) as f64;
    if ratio >= 1.0 {
        return None;
    }
    Some(radius.powi((n + 1) as i32) / factorial(n + 1) as f64 / (1.0 - ratio))
}
pub fn coefficient(n: usize) -> String {
    let d = factorial(n);
    if d == 1 { "1".into() } else { format!("1/{d}") }
}
/// Match the actual drawn grid, coefficients, curves and rounded bound.
pub fn append_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add("factorials.plot.radius", format!("{RADIUS:.6}"))?;
    facts.add("factorials.plot.top", format!("{PLOT_TOP:.6}"))?;
    facts.add("factorials.plot.segments", PLOT_SEGMENTS)?;
    for n in ORDERS {
        facts.add(&format!("factorials.coefficient.{n}"), coefficient(n))?;
        facts.add(&format!("factorials.denominator.{n}"), factorial(n))?;
        for j in 0..=PLOT_SEGMENTS {
            let x = plot_x(j);
            facts.add(&format!("factorials.curve.{n}.{j}.x"), format!("{x:.6}"))?;
            facts.add(&format!("factorials.curve.{n}.{j}.y"), format!("{:.6}", sum(x, n)))?;
        }
    }
    facts.add("factorials.tail.order", TAIL_ORDER)?;
    facts.add(
        "factorials.tail.bound",
        format!("{:.5}", tail_bound(RADIUS, TAIL_ORDER).ok_or("invalid tail bound")?),
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derivative_coefficients_match_the_previous_power() {
        for n in 1..=18 {
            assert_eq!(factorial(n), n as u64 * factorial(n - 1));
        }
    }
    #[test]
    fn bound_contains_positive_and_negative_sample_errors() {
        for n in [2, 4, 8, 12] {
            let bound = tail_bound(2.0, n).unwrap();
            for k in -40..=40 {
                let x = k as f64 / 20.0;
                assert!((x.exp() - sum(x, n)).abs() <= bound + 1e-13);
            }
        }
        assert!(tail_bound(4.0, 2).is_none());
        assert_eq!(tail_bound(0.0, 4), Some(0.0));
    }
}
