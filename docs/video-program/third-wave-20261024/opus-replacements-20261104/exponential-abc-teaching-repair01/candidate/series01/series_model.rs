//! Bounded numerical picture model, not a proof from sampled curves.
pub const ORDERS: [usize; 5] = [0, 1, 2, 3, 4];
pub const PLOT_SEGMENTS: usize = 60;
pub const RADIUS: f64 = 1.5;
/// RADIUS as a fraction, so the drawn ratios print exactly.
const RADIUS_FRACTION: (u64, u64) = (3, 2);
pub const PLOT_TOP: f64 = 5.0;
/// The degrees whose tails the proof card prints; the first is the one it zooms.
pub const TAIL_ORDERS: [usize; 2] = [4, 8];
/// The last bar in the main panel.
pub const BAR_TOP: usize = 8;
/// The last bar in the zoomed panel.
pub const INSET_TOP: usize = 9;
/// How much the zoomed panel enlarges the main panel's heights.
pub const ZOOM: u32 = 16;
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
/// One absolute term of the series: |x|^k / k!.
pub fn term_size(x: f64, k: usize) -> f64 {
    x.abs().powi(k as i32) / factorial(k) as f64
}
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
/// Term `index` over the term before it, at |x| = RADIUS: R / index, in lowest terms.
pub fn ratio_label(index: usize) -> String {
    assert!(index >= 1);
    let (n, d) = (RADIUS_FRACTION.0, RADIUS_FRACTION.1 * index as u64);
    let g = gcd(n, d);
    format!("{}/{}", n / g, d / g)
}
/// The first index from which R / index is at most one half.
pub fn halving_from(radius: f64) -> usize {
    (2.0 * radius).ceil() as usize
}
/// Uniform bound for |x|<=radius after degree n: the first omitted term times
/// 1 + 1/2 + 1/4 + ... = 2. Every later ratio is at most R/(n+2), so n+2>=2R makes each at most one half.
pub fn tail_bound(radius: f64, n: usize) -> Option<f64> {
    if !radius.is_finite() || radius < 0.0 || n > 18 || ((n + 2) as f64) < 2.0 * radius {
        return None;
    }
    Some(2.0 * term_size(radius, n + 1))
}
/// The geometric ceiling over term k in the tail after degree n: the first omitted term, halved at each step.
pub fn envelope(radius: f64, n: usize, k: usize) -> f64 {
    assert!(k > n);
    term_size(radius, n + 1) / 2f64.powi((k - n - 1) as i32)
}
/// The bound as printed: rounded up at the fifth place, so the printed number is still an upper bound.
pub fn tail_label(bound: f64) -> String {
    format!("{:.5}", (bound * 1e5).ceil() / 1e5)
}
pub fn coefficient(n: usize) -> String {
    let d = factorial(n);
    if d == 1 { "1".into() } else { format!("1/{d}") }
}
/// The table's name for the coefficient of x^n, for n from 1 to 4.
pub fn letter(n: usize) -> &'static str {
    ["a", "b", "c", "d"][n - 1]
}
/// What differentiating the term of degree n leaves in the column to its left.
pub fn derivative_label(n: usize) -> String {
    if n == 1 {
        letter(1).into()
    } else {
        format!("{n}{}", letter(n))
    }
}
/// The equation column n-1 forces: the derivative's entry against the function's entry above it.
pub fn equation_label(n: usize) -> String {
    format!("{} = {}", derivative_label(n), coefficient(n - 1))
}
/// The same equation, solved.
pub fn value_label(n: usize) -> String {
    format!("{} = {}", letter(n), coefficient(n))
}
/// The divisions the rule has piled up in the denominator of degree k.
pub fn running_product_tex(k: usize) -> String {
    if k == 0 {
        return "1".into();
    }
    (1..=k).map(|i| i.to_string()).collect::<Vec<_>>().join(r"\cdot ")
}
pub fn factorial_tex(k: usize) -> String {
    format!("{k}!")
}
/// One term of the series as drawn, with its factorial showing from degree two.
pub fn term_tex(k: usize) -> String {
    match k {
        0 => "1".into(),
        1 => "x".into(),
        _ => format!(r"\frac{{x^{{{k}}}}}{{{k}!}}"),
    }
}
/// The same term as a column of a sum: every column after the first carries its plus sign.
pub fn column_tex(k: usize) -> String {
    if k == 0 {
        term_tex(0)
    } else {
        format!("+{}", term_tex(k))
    }
}
/// Match the actual drawn grid, coefficients, equations, curves, bars and rounded bounds.
/// Heights print to five places: at six, exact decimal ties make the native sum and the
/// oracle round apart (see WhiteSwan's review, finding S2).
pub fn append_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add("factorials.plot.radius", format!("{RADIUS:.6}"))?;
    facts.add("factorials.plot.top", format!("{PLOT_TOP:.6}"))?;
    facts.add("factorials.plot.segments", PLOT_SEGMENTS)?;
    for n in ORDERS {
        facts.add(&format!("factorials.coefficient.{n}"), coefficient(n))?;
        facts.add(&format!("factorials.denominator.{n}"), factorial(n))?;
        facts.add(&format!("factorials.running.{n}"), running_product_tex(n))?;
        for j in 0..=PLOT_SEGMENTS {
            let x = plot_x(j);
            facts.add(&format!("factorials.curve.{n}.{j}.x"), format!("{x:.6}"))?;
            facts.add(&format!("factorials.curve.{n}.{j}.y"), format!("{:.5}", sum(x, n)))?;
        }
    }
    for n in 1..=4 {
        facts.add(&format!("factorials.derivative.{n}"), derivative_label(n))?;
        facts.add(&format!("factorials.match.{n}.equation"), equation_label(n))?;
        facts.add(&format!("factorials.match.{n}.value"), value_label(n))?;
    }
    facts.add("factorials.halving.from", halving_from(RADIUS))?;
    for index in 1..=BAR_TOP {
        facts.add(&format!("factorials.ratio.{index}"), ratio_label(index))?;
    }
    for k in 0..=INSET_TOP {
        facts.add(&format!("factorials.bar.{k}"), format!("{:.5}", term_size(RADIUS, k)))?;
    }
    let zoomed = TAIL_ORDERS[0];
    for k in zoomed + 1..=INSET_TOP {
        facts.add(
            &format!("factorials.envelope.{k}"),
            format!("{:.5}", envelope(RADIUS, zoomed, k)),
        )?;
    }
    facts.add("factorials.zoom", ZOOM)?;
    for n in TAIL_ORDERS {
        facts.add(
            &format!("factorials.tail.{n}.bound"),
            tail_label(tail_bound(RADIUS, n).ok_or("invalid tail bound")?),
        )?;
    }
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
    #[test]
    fn the_hypothesis_is_exact_at_its_edge() {
        // n + 2 >= 2R: at R = 3/2 that is n >= 1.
        assert!(tail_bound(RADIUS, 0).is_none());
        assert!(tail_bound(RADIUS, 1).is_some());
        assert_eq!(halving_from(RADIUS), 3);
    }
    #[test]
    fn the_printed_bounds_are_rounded_up_and_still_contain_the_true_tail() {
        assert_eq!(tail_label(tail_bound(RADIUS, 4).unwrap()), "0.12657");
        assert_eq!(tail_label(tail_bound(RADIUS, 8).unwrap()), "0.00022");
        for n in TAIL_ORDERS {
            let printed: f64 = tail_label(tail_bound(RADIUS, n).unwrap()).parse().unwrap();
            assert!(RADIUS.exp() - sum(RADIUS, n) <= printed);
        }
    }
    #[test]
    fn every_tail_bar_sits_under_its_ceiling_at_every_input() {
        for n in TAIL_ORDERS {
            for k in n + 1..=n + 12 {
                for j in -30..=30 {
                    let x = RADIUS * j as f64 / 30.0;
                    assert!(term_size(x, k) <= envelope(RADIUS, n, k) * (1.0 + 1e-12));
                }
            }
        }
    }
    #[test]
    fn the_derivative_of_each_term_is_the_previous_term() {
        // d/dx of x^k/k! is x^(k-1)/(k-1)!, so the derivative series is the same bars, one column left.
        for k in 1..=18 {
            let derivative = k as f64 * RADIUS.powi(k as i32 - 1) / factorial(k) as f64;
            assert!((derivative - term_size(RADIUS, k - 1)).abs() < 1e-9);
        }
    }
    #[test]
    fn every_ratio_is_the_exact_step_between_two_bars() {
        for index in 1..=BAR_TOP {
            let (n, d) = ratio_label(index)
                .split_once('/')
                .map(|(a, b)| (a.to_owned(), b.to_owned()))
                .unwrap();
            let printed = n.parse::<f64>().unwrap() / d.parse::<f64>().unwrap();
            assert!((term_size(RADIUS, index) / term_size(RADIUS, index - 1) - printed).abs() < 1e-12);
        }
        assert_eq!(ratio_label(3), "1/2");
    }
    #[test]
    fn the_table_prints_the_exact_equations() {
        let equations: Vec<String> = (1..=4).map(equation_label).collect();
        assert_eq!(equations, ["a = 1", "2b = 1", "3c = 1/2", "4d = 1/6"]);
        let values: Vec<String> = (1..=4).map(value_label).collect();
        assert_eq!(values, ["a = 1", "b = 1/2", "c = 1/6", "d = 1/24"]);
    }
    #[test]
    fn the_running_products_are_the_factorials() {
        for k in 0..=4 {
            let product: u64 = running_product_tex(k)
                .split(r"\cdot ")
                .map(|p| p.parse::<u64>().unwrap())
                .product();
            assert_eq!(product, factorial(k));
        }
        assert_eq!(running_product_tex(4), r"1\cdot 2\cdot 3\cdot 4");
        assert_eq!(column_tex(4), r"+\frac{x^{4}}{4!}");
        assert_eq!(column_tex(0), "1");
    }
}
