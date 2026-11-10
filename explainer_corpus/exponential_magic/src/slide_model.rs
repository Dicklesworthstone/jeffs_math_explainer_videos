//! One-decade slide rule: geometry is computed, not hand-spaced.
use explainer_kit::Facts;
pub const LEFT: f64 = -3.0;
pub const SPAN: f64 = 5.5;
pub const LABELS: [u32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
pub const EXAMPLES: [(f64, f64); 3] = [(2.0, 3.0), (2.5, 4.0), (4.0, 3.0)];
pub fn distance(value: f64) -> f64 {
    assert!(value.is_finite() && value > 0.0);
    SPAN * value.log10()
}
pub fn position(value: f64) -> f64 {
    LEFT + distance(value)
}
pub fn offset(factor: f64, right_index: bool) -> f64 {
    distance(factor) - if right_index { SPAN } else { 0.0 }
}
pub fn reading(a: f64, b: f64, right_index: bool) -> f64 {
    a * b / if right_index { 10.0 } else { 1.0 }
}
pub fn label(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}
pub fn equation(a: f64, b: f64) -> String {
    format!("{} × {} = {}", label(a), label(b), label(a * b))
}
pub fn append_facts(f: &mut Facts) -> Result<(), String> {
    for value in LABELS {
        f.add(
            &format!("ruler.tick.{value}"),
            format!("{:.8}", position(f64::from(value))),
        )?;
    }
    for (i, (a, b)) in EXAMPLES.into_iter().enumerate() {
        f.add(&format!("ruler.example.{i}.equation"), equation(a, b))?;
        f.add(
            &format!("ruler.example.{i}.left_offset"),
            format!("{:.8}", offset(a, false)),
        )?;
        f.add(
            &format!("ruler.example.{i}.reading"),
            label(reading(a, b, a * b > 10.0)),
        )?;
    }
    f.add("ruler.wrap.offset", format!("{:.8}", offset(EXAMPLES[2].0, true)))?;
    f.add(
        "ruler.division",
        format!(
            "{} ÷ {} = {}",
            label(EXAMPLES[0].0 * EXAMPLES[0].1),
            label(EXAMPLES[0].1),
            label(EXAMPLES[0].0)
        ),
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alignments_and_decade_are_exact_within_float_precision() {
        for (a, b) in EXAMPLES {
            let right = a * b > 10.0;
            assert!((position(b) + offset(a, right) - position(reading(a, b, right))).abs() < 1e-12);
        }
        assert!((position(10.0) - position(1.0) - SPAN).abs() < 1e-12);
    }
    #[test]
    fn off_scale_is_not_a_tick_and_division_inverts_alignment() {
        let (a, b) = EXAMPLES[2];
        assert!(position(a * b) > position(10.0));
        let (a, b) = EXAMPLES[0];
        assert!((position(a * b) - position(b) - distance(a)).abs() < 1e-12);
    }
    #[test]
    fn every_whole_number_is_labelled_and_the_labelled_gaps_shrink() {
        assert!(LABELS.iter().copied().eq(1..=10_u32));
        let marks = LABELS.map(|n| position(f64::from(n)));
        let gaps: Vec<f64> = marks.windows(2).map(|p| p[1] - p[0]).collect();
        assert!(gaps.windows(2).all(|p| p[1] < p[0]));
    }
    #[test]
    fn equal_ratios_have_equal_lengths_and_spacing_contracts() {
        assert!((distance(4.0) - distance(2.0) - distance(2.0)).abs() < 1e-12);
        assert!(distance(3.0) - distance(2.0) < distance(2.0) - distance(1.0));
    }
}
