//! Computed values for the examples, not a proof of their general claims.
use explainer_kit::Facts;

pub const SLIDE_A: f64 = 2.0;
pub const SLIDE_B: f64 = 3.0;
pub const BASE: u64 = 2;
pub const FIRST: u32 = 3;
pub const SECOND: u32 = 2;
pub const SAMPLE_X: f64 = 1.0;
pub const COUNTS: [i32; 6] = [1, 2, 4, 16, 64, 1024];
pub const SERIES_ORDER: u32 = 8;

pub fn limit_sample(x: f64, n: i32) -> f64 {
    (1.0 + x / f64::from(n)).powi(n)
}
pub fn series(x: f64, order: u32) -> f64 {
    let mut term = 1.0;
    let mut sum = term;
    for k in 1..=order {
        term *= x / f64::from(k);
        sum += term;
    }
    sum
}

pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    facts.add("slide.a", SLIDE_A as u64)?;
    facts.add("slide.b", SLIDE_B as u64)?;
    facts.add("slide.product", (SLIDE_A * SLIDE_B) as u64)?;
    facts.add("growth.base", BASE)?;
    facts.add("growth.first", FIRST)?;
    facts.add("growth.second", SECOND)?;
    facts.add("growth.first_factor", BASE.pow(FIRST))?;
    facts.add("growth.second_factor", BASE.pow(SECOND))?;
    facts.add("growth.total_factor", BASE.pow(FIRST + SECOND))?;
    for n in COUNTS {
        facts.add(&format!("limit.{n}"), format!("{:.5}", limit_sample(SAMPLE_X, n)))?;
    }
    facts.add("series.order", SERIES_ORDER)?;
    facts.add("series.sum", format!("{:.5}", series(SAMPLE_X, SERIES_ORDER)))?;
    crate::rate_model::append_facts(&mut facts)?;
    crate::series_model::append_facts(&mut facts)?;
    crate::limit_model::append_facts(&mut facts)?;
    crate::solution_model::append_facts(&mut facts)?;
    crate::product_model::append_facts(&mut facts)?;
    crate::slide_model::append_facts(&mut facts)?;
    crate::chapters::decay_scene::append_facts(&mut facts)?;
    crate::chapters::circuit_scene::append_facts(&mut facts)?;
    crate::changing_rate_model::append_facts(&mut facts)?;
    crate::turning_model::append_facts(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn growth_and_slide_examples_are_consistent() {
        assert_eq!(SLIDE_A * SLIDE_B, 6.0);
        assert_eq!(BASE.pow(FIRST) * BASE.pow(SECOND), BASE.pow(FIRST + SECOND));
    }
    #[test]
    fn finite_examples_are_below_exp_one() {
        for n in COUNTS {
            assert!(limit_sample(SAMPLE_X, n) < std::f64::consts::E);
        }
        assert!((series(SAMPLE_X, SERIES_ORDER) - std::f64::consts::E).abs() < 0.000004);
    }
}
