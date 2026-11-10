//! Complete low-degree diagonals of an infinite series-product grid.
pub const SIDE: usize = 4;
/// The line 03_7 marks: integers from LADDER_LOW to LADDER_HIGH, in steps of 1/LADDER_STEPS.
pub const LADDER_LOW: i32 = -2;
pub const LADDER_HIGH: i32 = 3;
pub const LADDER_STEPS: i32 = 4;
pub fn factorial(n: usize) -> u64 {
    (1..=n as u64).product()
}
#[cfg(test)]
pub fn choose(n: usize, k: usize) -> u64 {
    assert!(k <= n && n <= 20);
    factorial(n) / (factorial(k) * factorial(n - k))
}
fn power(symbol: &str, degree: usize) -> String {
    match degree {
        0 => String::new(),
        1 => symbol.into(),
        _ => format!("{symbol}^{{{degree}}}"),
    }
}
pub fn cell_tex(i: usize, j: usize) -> String {
    let numerator = format!("{}{}", power("x", i), power("y", j));
    let numerator = if numerator.is_empty() { "1".into() } else { numerator };
    let d = factorial(i) * factorial(j);
    if d == 1 {
        numerator
    } else {
        format!(r"\frac{{{numerator}}}{{{d}}}")
    }
}
pub fn diagonal(m: usize) -> Vec<(usize, usize)> {
    assert!(m < SIDE, "only complete displayed diagonals");
    (0..=m).map(|i| (i, m - i)).collect()
}
pub fn total_tex(m: usize) -> String {
    match m {
        0 => "1".into(),
        1 => "x+y".into(),
        _ => format!(r"\frac{{(x+y)^{{{m}}}}}{{{}}}", factorial(m)),
    }
}
/// E(1) to five places, from the partial sums of the series itself.
pub fn e_label() -> String {
    let mut term = 1.0;
    let mut total = 1.0;
    for k in 1..=20 {
        term /= f64::from(k);
        total += term;
    }
    format!("{total:.5}")
}
/// The marks on the 03_7 line, by what earns them. The law gives the whole numbers from zero up.
pub fn ladder_whole() -> Vec<f64> {
    (0..=LADDER_HIGH).map(f64::from).collect()
}
/// Positive roots give the positive fractions.
pub fn ladder_fractions() -> Vec<f64> {
    (1..LADDER_HIGH * LADDER_STEPS)
        .filter(|j| j % LADDER_STEPS != 0)
        .map(|j| f64::from(j) / f64::from(LADDER_STEPS))
        .collect()
}
/// Reciprocals give everything below zero.
pub fn ladder_negatives() -> Vec<f64> {
    (LADDER_LOW * LADDER_STEPS..0)
        .map(|j| f64::from(j) / f64::from(LADDER_STEPS))
        .collect()
}
/// Export every denominator the grid prints, the totals beside it and the spoken cell count.
pub fn append_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add("product.side", SIDE)?;
    facts.add("product.cells", SIDE * SIDE)?;
    for i in 0..SIDE {
        for j in 0..SIDE {
            facts.add(
                &format!("product.cell.{i}.{j}.denominator"),
                factorial(i) * factorial(j),
            )?;
        }
    }
    for m in 0..SIDE {
        facts.add(&format!("product.diagonal.{m}.cells"), diagonal(m).len())?;
        facts.add(&format!("product.total.{m}.denominator"), factorial(m))?;
    }
    facts.add("product.e", e_label())?;
    facts.add("product.ladder.low", LADDER_LOW)?;
    facts.add("product.ladder.high", LADDER_HIGH)?;
    facts.add("product.ladder.steps", LADDER_STEPS)?;
    facts.add("product.ladder.whole", ladder_whole().len())?;
    facts.add("product.ladder.fractions", ladder_fractions().len())?;
    facts.add("product.ladder.negatives", ladder_negatives().len())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagonals_have_every_term_once_and_no_truncated_degree() {
        for m in 0..SIDE {
            let d = diagonal(m);
            assert_eq!(d.len(), m + 1);
            for (i, j) in d {
                assert_eq!(i + j, m);
                assert!(i < SIDE && j < SIDE);
            }
        }
    }
    #[test]
    fn factorial_weights_are_exact_binomial_coefficients() {
        for m in 0..=12 {
            for k in 0..=m {
                assert_eq!(choose(m, k) * factorial(k) * factorial(m - k), factorial(m));
            }
        }
    }
    #[test]
    fn every_printed_cell_is_exact() {
        assert_eq!(cell_tex(0, 0), "1");
        assert_eq!(cell_tex(1, 1), "xy");
        assert_eq!(cell_tex(2, 0), r"\frac{x^{2}}{2}");
        assert_eq!(cell_tex(2, 1), r"\frac{x^{2}y}{2}");
        assert_eq!(cell_tex(3, 3), r"\frac{x^{3}y^{3}}{36}");
        assert_eq!(total_tex(3), r"\frac{(x+y)^{3}}{6}");
    }
    #[test]
    fn the_named_constant_is_e() {
        assert_eq!(e_label(), format!("{:.5}", 1f64.exp()));
        assert_eq!(e_label(), "2.71828");
    }
    #[test]
    fn the_line_marks_every_step_once_and_in_its_own_group() {
        let mut all = ladder_negatives();
        all.extend(ladder_whole());
        all.extend(ladder_fractions());
        all.sort_by(f64::total_cmp);
        let span = (LADDER_HIGH - LADDER_LOW) * LADDER_STEPS;
        assert_eq!(all.len(), span as usize + 1);
        for (j, x) in all.iter().enumerate() {
            assert_eq!(*x, f64::from(LADDER_LOW) + j as f64 / f64::from(LADDER_STEPS));
        }
        assert!(ladder_negatives().iter().all(|x| *x < 0.0));
        assert!(ladder_fractions().iter().all(|x| *x > 0.0 && x.fract() != 0.0));
    }
}
