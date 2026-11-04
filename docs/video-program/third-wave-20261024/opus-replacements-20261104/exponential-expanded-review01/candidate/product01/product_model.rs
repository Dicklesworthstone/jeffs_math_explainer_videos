//! Complete low-degree diagonals of an infinite series-product grid.
pub const SIDE: usize = 4;
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
}
