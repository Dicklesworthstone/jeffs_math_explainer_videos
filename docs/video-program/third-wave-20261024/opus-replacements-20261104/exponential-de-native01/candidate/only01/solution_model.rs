//! Arithmetic behind the uniqueness chapter. The proof is the derivative the chapter writes out; these curves only
//! show the test at work on functions we can name.
use crate::limit_model::e_value;
use crate::series_model::sum;

/// The test's panels: inputs from -PANEL_RADIUS to PANEL_RADIUS at steps of 1/10.
pub const PANEL_RADIUS: f64 = 1.5;
pub const PANEL_SAMPLES: usize = 30;
/// The heights the four panels run up to, as their top ticks print them.
pub const FUNCTION_TOP: f64 = 5.0;
pub const PRODUCT_TOP: f64 = 3.0;
pub const PIECE_TOP: f64 = 3.0;
pub const RATE_TOP: f64 = 6.0;
/// The polynomials the test is tried on. Their rule breaks by exactly their top term.
pub const DEGREES: [usize; 2] = [2, 4];
/// The gap in the domain: a far piece and a near piece, and the far piece's multiple of E.
pub const FAR_PIECE: (f64, f64) = (-2.0, -0.5);
pub const NEAR_PIECE: (f64, f64) = (-0.25, 1.0);
pub const FAR_MULTIPLE: f64 = 3.0;
pub const PIECE_SAMPLES: usize = 15;
/// The constant-rate picture: y(0) = RATE_START on 0 to RATE_END, for these rates.
pub const RATES: [(f64, &str); 3] = [(0.5, "1/2"), (0.0, "0"), (-0.5, "-1/2")];
pub const RATE_START: f64 = 2.0;
pub const RATE_END: f64 = 2.0;
pub const RATE_SAMPLES: usize = 20;

pub fn panel_x(j: usize) -> f64 {
    assert!(j <= PANEL_SAMPLES);
    -PANEL_RADIUS + PANEL_RADIUS * 2.0 * j as f64 / PANEL_SAMPLES as f64
}
/// The partial sum of degree d, a function that starts at one and breaks the rule.
pub fn polynomial(x: f64, degree: usize) -> f64 {
    sum(x, degree)
}
/// What the rule's test reads for it: its product with E(-x).
pub fn product(x: f64, degree: usize) -> f64 {
    polynomial(x, degree) * e_value(-x)
}
/// d! as a whole number, for the slope's printed denominator. Degrees here are small enough for exact floats.
pub fn top_term_denominator(degree: usize) -> u64 {
    (1..=degree as u64).product()
}
/// The slope of that product as written: y' - y is minus the top term, times E(-x).
pub fn defect_tex(degree: usize) -> String {
    format!(r"-\frac{{x^{{{degree}}}}}{{{}}}E(-x)", top_term_denominator(degree))
}
/// The picture's question: a curve that starts at one and is not E. It breaks the rule, and is labelled with "?".
pub fn imagined(x: f64) -> f64 {
    e_value(x) * (1.0 - x * x / 6.0)
}
/// The function on a domain with a gap: E near zero, FAR_MULTIPLE times E across the gap.
pub fn far_piece(j: usize) -> (f64, f64) {
    let x = FAR_PIECE.0 + (FAR_PIECE.1 - FAR_PIECE.0) * j as f64 / PIECE_SAMPLES as f64;
    (x, FAR_MULTIPLE * e_value(x))
}
pub fn near_piece(j: usize) -> (f64, f64) {
    let x = NEAR_PIECE.0 + (NEAR_PIECE.1 - NEAR_PIECE.0) * j as f64 / PIECE_SAMPLES as f64;
    (x, e_value(x))
}
/// The constant-rate solution: y(t) = y(0) E(kt).
pub fn rate_solution(rate: f64, start: f64, t: f64) -> f64 {
    start * e_value(rate * t)
}
pub fn rate_time(j: usize) -> f64 {
    RATE_END * j as f64 / RATE_SAMPLES as f64
}
/// The value each rate has reached at the end of the picture, as printed.
pub fn rate_label(rate: f64) -> String {
    format!("{:.3}", rate_solution(rate, RATE_START, RATE_END))
}
fn row(values: impl Iterator<Item = f64>) -> String {
    values.map(|y| format!("{y:.5}")).collect::<Vec<_>>().join(" ")
}

/// Export every number the chapter prints, and the data behind every drawn curve.
pub fn append_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let inputs = || (0..=PANEL_SAMPLES).map(panel_x);
    facts.add("only.e", row(inputs().map(e_value)))?;
    facts.add("only.imagined", row(inputs().map(imagined)))?;
    facts.add(
        "only.axes.tops",
        format!("{FUNCTION_TOP} {PRODUCT_TOP} {PIECE_TOP} {RATE_TOP}"),
    )?;
    for d in DEGREES {
        facts.add(&format!("only.polynomial.{d}"), row(inputs().map(|x| polynomial(x, d))))?;
        facts.add(&format!("only.product.{d}"), row(inputs().map(|x| product(x, d))))?;
        facts.add(&format!("only.defect.{d}"), top_term_denominator(d))?;
    }
    facts.add("only.far.multiple", FAR_MULTIPLE)?;
    facts.add("only.far", row((0..=PIECE_SAMPLES).map(|j| far_piece(j).1)))?;
    facts.add("only.near", row((0..=PIECE_SAMPLES).map(|j| near_piece(j).1)))?;
    facts.add("only.rate.start", RATE_START)?;
    facts.add("only.rate.end", RATE_END)?;
    for (rate, name) in RATES {
        facts.add(
            &format!("only.rate.{name}"),
            row((0..=RATE_SAMPLES).map(|j| rate_solution(rate, RATE_START, rate_time(j)))),
        )?;
        facts.add(&format!("only.rate.{name}.end"), rate_label(rate))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series_model::sum;

    /// The slope the chapter's derivative gives, from the polynomial's own derivative.
    fn product_slope(x: f64, degree: usize) -> f64 {
        (sum(x, degree - 1) - sum(x, degree)) * e_value(-x)
    }
    fn derivative(f: impl Fn(f64) -> f64, x: f64) -> f64 {
        let h = 1e-5;
        (f(x + h) - f(x - h)) / (2.0 * h)
    }
    #[test]
    fn the_written_derivative_matches_the_slope_numerically() {
        for d in DEGREES {
            for j in 0..=PANEL_SAMPLES {
                let x = panel_x(j);
                assert!(
                    (derivative(|t| product(t, d), x) - product_slope(x, d)).abs() < 1e-7,
                    "d={d} x={x}"
                );
            }
        }
    }
    #[test]
    fn the_written_slope_is_exactly_minus_the_top_term_times_e_of_minus_x() {
        assert_eq!(defect_tex(2), r"-\frac{x^{2}}{2}E(-x)");
        assert_eq!(defect_tex(4), r"-\frac{x^{4}}{24}E(-x)");
        for d in DEGREES {
            for j in 0..=PANEL_SAMPLES {
                let x = panel_x(j);
                let top = x.powi(d as i32) / top_term_denominator(d) as f64;
                assert!((product_slope(x, d) + top * e_value(-x)).abs() < 1e-12, "d={d} x={x}");
            }
        }
    }
    #[test]
    fn every_curve_fits_under_its_panel_top() {
        for j in 0..=PANEL_SAMPLES {
            let x = panel_x(j);
            assert!(e_value(x) < FUNCTION_TOP && imagined(x) < FUNCTION_TOP);
            for d in DEGREES {
                assert!(polynomial(x, d) < FUNCTION_TOP && product(x, d) < PRODUCT_TOP);
            }
        }
        for j in 0..=PIECE_SAMPLES {
            assert!(far_piece(j).1 < PIECE_TOP && near_piece(j).1 < PIECE_TOP);
        }
        for (rate, _) in RATES {
            for j in 0..=RATE_SAMPLES {
                assert!(rate_solution(rate, RATE_START, rate_time(j)) < RATE_TOP);
            }
        }
    }
    #[test]
    fn the_polynomials_start_at_one_and_tilt_down_with_the_higher_degree_flatter() {
        for d in DEGREES {
            assert!((product(0.0, d) - 1.0).abs() < 1e-12);
            for j in 0..PANEL_SAMPLES {
                assert!(product(panel_x(j + 1), d) <= product(panel_x(j), d) + 1e-12);
                assert!(product_slope(panel_x(j), d) <= 1e-12);
            }
        }
        let drop = |d| product(-PANEL_RADIUS, d) - product(PANEL_RADIUS, d);
        assert!(drop(4) < drop(2));
    }
    #[test]
    fn e_itself_reads_level_at_one() {
        for j in 0..=PANEL_SAMPLES {
            let x = panel_x(j);
            assert!((e_value(x) * e_value(-x) - 1.0).abs() < 1e-10);
        }
    }
    #[test]
    fn the_imagined_curve_starts_at_one_and_is_not_a_solution() {
        assert!((imagined(0.0) - 1.0).abs() < 1e-12);
        let defect = derivative(imagined, 1.0) - imagined(1.0);
        assert!(
            defect.abs() > 0.1,
            "it must break the rule, so the picture claims no real solution"
        );
        for j in 0..=PANEL_SAMPLES {
            assert!(imagined(panel_x(j)) > 0.0, "it stays on the panel");
        }
    }
    #[test]
    fn both_pieces_obey_the_rule_and_the_far_piece_is_not_e() {
        for j in 0..=PIECE_SAMPLES {
            let (x, y) = far_piece(j);
            assert!((derivative(|t| FAR_MULTIPLE * e_value(t), x) - y).abs() < 1e-7);
            assert!(
                (y * e_value(-x) - FAR_MULTIPLE).abs() < 1e-9,
                "the far piece reads level at the multiple"
            );
            let (x, y) = near_piece(j);
            assert!((derivative(e_value, x) - y).abs() < 1e-7);
            assert!(
                (y * e_value(-x) - 1.0).abs() < 1e-9,
                "the near piece reads level at one"
            );
        }
        assert!(FAR_PIECE.1 < NEAR_PIECE.0 && NEAR_PIECE.0 < 0.0 && NEAR_PIECE.1 > 0.0);
    }
    #[test]
    fn each_rate_solves_its_law_and_the_edges_behave() {
        for (rate, _) in RATES {
            for j in 0..=RATE_SAMPLES {
                let t = rate_time(j);
                let y = |s: f64| rate_solution(rate, RATE_START, s);
                assert!((derivative(y, t) - rate * y(t)).abs() < 1e-6);
                assert!((y(t) * e_value(-rate * t) - RATE_START).abs() < 1e-9);
            }
        }
        assert_eq!(rate_label(0.0), "2.000");
        assert_eq!(rate_solution(0.5, 0.0, 1.7), 0.0);
    }
}
