//! Arithmetic behind the small-multiplier chapter. These are printed values and bar heights at named inputs;
//! the proof is the chain of inequalities the chapter draws, and these numbers only illustrate it.
use crate::series_model::{RADIUS, sum, tail_bound, tail_label, term_size};

/// Rate-one endpoints in the table. The first three are the rate chapter's own step counts.
pub const TABLE_COUNTS: [u32; 5] = [4, 8, 16, 64, 1024];
/// The cell count the graph's sweep uses.
pub const SWEEP_CELLS: u32 = 16;
/// The worked strip: four cells, and the columns its sixteen choices sort into.
pub const STRIP_CELLS: usize = 4;
/// The fixed-k picture: bars for these n, in columns 0 to FILL_TOP.
pub const FILL_COUNTS: [u32; 5] = [4, 16, 64, 256, 1024];
pub const FILL_TOP: usize = 100;
/// The near and the far column the fixed-k picture reads aloud on screen.
pub const FILL_NEAR: usize = 3;
pub const FILL_FAR: usize = 100;
/// The warning sum's frames: n terms, each 1/n.
pub const WARNING_COUNTS: [u32; 4] = [4, 16, 40, 100];
/// The fixed column the warning picture reads out.
pub const WARNING_COLUMN: usize = 3;
/// The bars under their ceilings, at the series chapter's own radius and columns.
pub const BAR_COUNTS: [u32; 4] = [4, 16, 64, 1024];
pub const BAR_TOP: usize = 9;
/// The two cuts the chapter tries, and the distance it aims for.
pub const EARLY_CUT: usize = 4;
pub const CUT: usize = 8;
pub const AIM: &str = "0.01";
/// Cell counts tried once the cut is fixed.
pub const HEAD_COUNTS: [u32; 3] = [64, 256, 1024];
/// What 05_6 draws: few enough cells that the gaps under the ceilings are wide enough to see.
/// A drawing only: the chapter's numerical bound is still computed at BOUND_CELLS.
pub const DISPLAY_CELLS: u32 = 16;
/// The cell count the chapter's numerical bound is computed at.
pub const BOUND_CELLS: u32 = HEAD_COUNTS[HEAD_COUNTS.len() - 1];
/// The edge table: x = EDGE_INPUT, base and value for these cell counts.
pub const EDGE_INPUT: f64 = -3.0;
pub const EDGE_COUNTS: [u32; 7] = [1, 2, 3, 4, 16, 64, 1024];
/// The closing graph: inputs from GRAPH_LOW to RADIUS at steps of 1/20, and walks of these cell counts.
pub const GRAPH_LOW: f64 = -3.0;
pub const GRAPH_SAMPLES: usize = 90;
pub const WALK_COUNTS: [u32; 2] = [4, 16];
/// Series degree used wherever a value of E is drawn or printed. Its error is far below the printed places.
const SERIES_DEGREE: usize = 18;

/// The value of E at x, as a partial sum well past the printed places.
pub fn e_value(x: f64) -> f64 {
    sum(x, SERIES_DEGREE)
}
/// What one cell multiplies by: 1 + x/n.
pub fn multiplier(x: f64, n: u32) -> f64 {
    1.0 + x / f64::from(n)
}
/// After all n cells: (1 + x/n)^n.
pub fn endpoint(x: f64, n: u32) -> f64 {
    multiplier(x, n).powi(n as i32)
}
/// The heights of a walk to input x in n equal steps: (1 + x/n)^j at input jx/n.
pub fn walk(x: f64, n: u32) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|j| (x * f64::from(j) / f64::from(n), multiplier(x, n).powi(j as i32)))
        .collect()
}
/// The chain of correction factors: the product of 1 - j/n for j below k. It is exactly zero once k passes n.
pub fn chain(n: u32, k: usize) -> f64 {
    if k > n as usize {
        return 0.0;
    }
    (0..k).map(|j| 1.0 - j as f64 / f64::from(n)).product()
}
/// Term k of the expanded product, signed: x^k / k! times the chain.
pub fn term(x: f64, n: u32, k: usize) -> f64 {
    let sign = if x < 0.0 && k % 2 == 1 { -1.0 } else { 1.0 };
    sign * term_size(x, k) * chain(n, k)
}
/// Empty space under ceiling k: the series term's size less the bar's.
pub fn gap(x: f64, n: u32, k: usize) -> f64 {
    term_size(x, k) * (1.0 - chain(n, k))
}
/// The gaps before and at the cut.
pub fn head_gaps(x: f64, n: u32, cut: usize) -> f64 {
    (0..=cut).map(|k| gap(x, n, k)).sum()
}
/// Choosing k of the strip's cells: Pascal's row, read as a count of choices.
pub fn ways(k: usize) -> u64 {
    let mut count = 1;
    for j in 0..k {
        count = count * (STRIP_CELLS - j) as u64 / (j as u64 + 1);
    }
    count
}
/// Which cells the i-th choice of k cells lights, in a fixed order: the subsets of k cells, as bit masks ascending.
pub fn choices(k: usize) -> Vec<[bool; STRIP_CELLS]> {
    (0u32..1 << STRIP_CELLS)
        .filter(|mask| mask.count_ones() as usize == k)
        .map(|mask| std::array::from_fn(|cell| mask & (1 << cell) != 0))
        .collect()
}
/// The strip's coefficient of x^k before reduction: ways over the power of the cell count.
pub fn strip_coefficient(k: usize) -> String {
    format!("{}/{}", ways(k), (STRIP_CELLS as u64).pow(k as u32))
}
/// A rate-one endpoint as the table prints it.
pub fn rate_one_label(n: u32) -> String {
    format!("{:.3}", endpoint(1.0, n))
}
/// One chain's value as the fixed-k picture reads it.
pub fn column_label(n: u32, k: usize) -> String {
    format!("{:.3}", chain(n, k))
}
/// The sweep's endpoint and series value at one input.
pub fn sweep_label(x: f64) -> (String, String) {
    (format!("{:.3}", endpoint(x, SWEEP_CELLS)), format!("{:.3}", e_value(x)))
}
/// The bound as a string: rounded up at the fifth place, the same rule the tail bound prints by.
pub fn up_label(value: f64) -> String {
    tail_label(value)
}
/// Printed tail of the series at the chapter's radius after degree `cut`.
pub fn tail_print(cut: usize) -> String {
    tail_label(tail_bound(RADIUS, cut).expect("the drawn cuts meet the hypothesis"))
}
/// Head plus tail at the chapter's radius, each already rounded up, so the sum is still an upper bound.
pub fn total_label(n: u32) -> String {
    let head: f64 = up_label(head_gaps(RADIUS, n, CUT)).parse().expect("a printed decimal");
    let tail: f64 = tail_print(CUT).parse().expect("a printed decimal");
    format!("{:.5}", head + tail)
}
/// The true distance from the endpoint to E at input x.
pub fn distance(x: f64, n: u32) -> f64 {
    (e_value(x) - endpoint(x, n)).abs()
}
/// One row of the edge table: the cell's multiplier, exact.
pub fn edge_base(n: u32) -> String {
    match n {
        1 => "-2".into(),
        2 => "-1/2".into(),
        3 => "0".into(),
        _ => format!("{}/{n}", n - 3),
    }
}
/// One row of the edge table: the endpoint, exact while it is short, then to four places.
pub fn edge_value(n: u32) -> String {
    match n {
        1 => "-2".into(),
        2 => "1/4".into(),
        3 => "0".into(),
        _ => format!("{:.4}", endpoint(EDGE_INPUT, n)),
    }
}
/// The input at sample j of the closing graph.
pub fn graph_x(j: usize) -> f64 {
    assert!(j <= GRAPH_SAMPLES);
    GRAPH_LOW + j as f64 / 20.0
}

/// Export every number the chapter prints, and the data behind every drawn height.
pub fn append_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add("tiny.step.multiplier", "5/4")?;
    for n in TABLE_COUNTS {
        facts.add(&format!("tiny.one.{n}"), rate_one_label(n))?;
    }
    facts.add("tiny.sweep.cells", SWEEP_CELLS)?;
    for x in [RADIUS, -RADIUS] {
        let (end, series) = sweep_label(x);
        facts.add(&format!("tiny.sweep.{x}.endpoint"), end)?;
        facts.add(&format!("tiny.sweep.{x}.series"), series)?;
    }
    for k in 0..=STRIP_CELLS {
        facts.add(&format!("tiny.ways.{k}"), ways(k))?;
        facts.add(&format!("tiny.strip.{k}"), strip_coefficient(k))?;
        facts.add(&format!("tiny.chain.4.{k}"), format!("{:.5}", chain(4, k)))?;
    }
    for n in FILL_COUNTS {
        let row: Vec<String> = (0..=FILL_TOP).map(|k| format!("{:.5}", chain(n, k))).collect();
        facts.add(&format!("tiny.fill.{n}"), row.join(" "))?;
        for k in [FILL_NEAR, FILL_FAR] {
            facts.add(&format!("tiny.column.{n}.{k}"), column_label(n, k))?;
        }
    }
    facts.add("tiny.warning.column", WARNING_COLUMN)?;
    for n in WARNING_COUNTS {
        facts.add(&format!("tiny.warning.{n}"), format!("1/{n}"))?;
    }
    facts.add("tiny.warning.total", 1)?;
    for n in BAR_COUNTS {
        let row: Vec<String> = (0..=BAR_TOP)
            .map(|k| format!("{:.5}", term(RADIUS, n, k).abs()))
            .collect();
        facts.add(&format!("tiny.bars.{n}"), row.join(" "))?;
    }
    facts.add("tiny.aim", AIM)?;
    for cut in [EARLY_CUT, CUT] {
        facts.add(&format!("tiny.cut.{cut}.tail"), tail_print(cut))?;
    }
    for n in HEAD_COUNTS {
        facts.add(&format!("tiny.head.{n}"), up_label(head_gaps(RADIUS, n, CUT)))?;
        facts.add(&format!("tiny.total.{n}"), total_label(n))?;
        facts.add(&format!("tiny.distance.{n}"), format!("{:.5}", distance(RADIUS, n)))?;
        facts.add(&format!("tiny.negative.{n}"), format!("{:.5}", distance(-RADIUS, n)))?;
    }
    facts.add("tiny.display.cells", DISPLAY_CELLS)?;
    facts.add("tiny.display.bound_cells", BOUND_CELLS)?;
    for n in [DISPLAY_CELLS, BOUND_CELLS] {
        let row: Vec<String> = (0..=BAR_TOP).map(|k| format!("{:.5}", gap(RADIUS, n, k))).collect();
        facts.add(&format!("tiny.display.gaps.{n}"), row.join(" "))?;
    }
    for n in EDGE_COUNTS {
        facts.add(&format!("tiny.edge.{n}.base"), edge_base(n))?;
        facts.add(&format!("tiny.edge.{n}.value"), edge_value(n))?;
    }
    facts.add("tiny.edge.series", format!("{:.4}", e_value(EDGE_INPUT)))?;
    for j in 0..=GRAPH_SAMPLES {
        let x = graph_x(j);
        facts.add(&format!("tiny.curve.{j}"), format!("{x:.2} {:.5}", e_value(x)))?;
    }
    for n in WALK_COUNTS {
        for (side, x) in [("right", RADIUS), ("left", GRAPH_LOW)] {
            let row: Vec<String> = walk(x, n).iter().map(|&(_, y)| format!("{y:.5}")).collect();
            facts.add(&format!("tiny.walk.{n}.{side}"), row.join(" "))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series_model::factorial;

    /// The Pascal row by its own recurrence, for the exact-statement checks below.
    fn pascal(n: usize) -> Vec<f64> {
        let mut row = vec![1.0];
        for _ in 0..n {
            let mut next = vec![1.0; row.len() + 1];
            for i in 1..row.len() {
                next[i] = row[i - 1] + row[i];
            }
            row = next;
        }
        row
    }
    #[test]
    fn the_chain_is_the_binomial_coefficient_shared_out() {
        for n in [1u32, 2, 3, 4, 8, 16, 32] {
            let row = pascal(n as usize);
            // The series model's factorials stop at twenty, and no scene reaches past nine.
            for (k, &choose) in row.iter().enumerate().take(21) {
                let x = 1.5f64;
                let direct = choose * (x / f64::from(n)).powi(k as i32);
                assert!(
                    (direct - term(x, n, k)).abs() < 1e-9 * direct.abs().max(1.0),
                    "n={n} k={k}"
                );
            }
        }
    }
    #[test]
    fn the_expansion_adds_up_to_the_product_at_both_signs_and_every_base() {
        for x in [1.5, -1.5, -3.0, 1.0, 0.0] {
            for n in [1u32, 2, 3, 4, 8, 16] {
                let total: f64 = (0..=n as usize).map(|k| term(x, n, k)).sum();
                assert!((total - endpoint(x, n)).abs() < 1e-9, "x={x} n={n}");
            }
        }
    }
    #[test]
    fn every_chain_stays_in_range_and_empties_past_n() {
        for n in [1u32, 2, 3, 4, 16, 1024] {
            for k in 0..=40 {
                let c = chain(n, k);
                assert!((0.0..=1.0).contains(&c), "n={n} k={k}");
                if k > n as usize {
                    assert_eq!(c, 0.0);
                }
            }
            assert_eq!(chain(n, 0), 1.0);
            assert_eq!(chain(n, 1), 1.0);
        }
    }
    #[test]
    fn no_bar_passes_its_ceiling_at_either_sign() {
        for x in [RADIUS, -RADIUS, EDGE_INPUT] {
            for n in [1u32, 2, 3, 4, 16, 64, 1024] {
                for k in 0..=18 {
                    assert!(
                        term(x, n, k).abs() <= term_size(x, k) * (1.0 + 1e-12),
                        "x={x} n={n} k={k}"
                    );
                }
            }
        }
    }
    #[test]
    fn the_distance_is_under_the_total_of_the_gaps() {
        for n in HEAD_COUNTS {
            let total: f64 = total_label(n).parse().unwrap();
            assert!(distance(RADIUS, n) <= total);
            assert!(distance(-RADIUS, n) <= total);
        }
    }
    #[test]
    fn the_printed_numbers_tell_the_story_of_the_cut() {
        let aim: f64 = AIM.parse().unwrap();
        let early: f64 = tail_print(EARLY_CUT).parse().unwrap();
        let late: f64 = tail_print(CUT).parse().unwrap();
        assert!(early > aim, "the early cut has to fail the aim");
        assert!(late < aim, "the later cut has to leave room");
        let totals: Vec<f64> = HEAD_COUNTS.iter().map(|&n| total_label(n).parse().unwrap()).collect();
        assert!(
            totals[0] > aim && totals[1] > aim,
            "the two smaller counts must fail the aim"
        );
        assert!(totals[2] < aim, "the largest count must meet the aim");
    }
    #[test]
    fn the_far_column_is_still_nearly_empty_at_every_drawn_count() {
        for n in FILL_COUNTS {
            assert!(chain(n, FILL_FAR) < 0.01, "n={n}");
        }
        assert!(chain(1024, FILL_NEAR) > 0.99);
        assert_eq!(column_label(4, FILL_NEAR), "0.375");
    }
    #[test]
    fn the_drawn_count_shows_gaps_that_the_bound_count_closes() {
        assert!(BAR_COUNTS.contains(&DISPLAY_CELLS));
        const { assert!(DISPLAY_CELLS < BOUND_CELLS) };
        let widest = |n: u32| (0..=BAR_TOP).map(|k| gap(RADIUS, n, k)).fold(0.0_f64, f64::max);
        assert!(widest(DISPLAY_CELLS) > 0.1);
        assert!(widest(BOUND_CELLS) < 0.002);
    }
    #[test]
    fn the_rate_one_table_climbs_by_less_and_less_and_stays_below_e() {
        let values: Vec<f64> = TABLE_COUNTS.iter().map(|&n| endpoint(1.0, n)).collect();
        assert!(values.windows(2).all(|p| p[0] < p[1]));
        assert!(values.windows(3).all(|p| p[2] - p[1] < p[1] - p[0]));
        assert!(values.iter().all(|&v| v < e_value(1.0)));
        assert_eq!(format!("{:.3}", values[0]), "2.441");
        assert_eq!(format!("{:.3}", values[1]), "2.566");
        assert_eq!(format!("{:.3}", values[2]), "2.638");
    }
    #[test]
    fn the_strip_counts_are_pascals_row_and_the_choices_list_them() {
        let counts: Vec<u64> = (0..=STRIP_CELLS).map(ways).collect();
        assert_eq!(counts, [1, 4, 6, 4, 1]);
        assert_eq!(counts.iter().sum::<u64>(), 16);
        for k in 0..=STRIP_CELLS {
            assert_eq!(choices(k).len() as u64, ways(k));
            assert!(choices(k).iter().all(|c| c.iter().filter(|&&lit| lit).count() == k));
        }
        assert_eq!(strip_coefficient(2), "6/16");
        assert!((6.0 / 16.0 - chain(4, 2) / factorial(2) as f64).abs() < 1e-15);
    }
    #[test]
    fn the_edge_table_shows_each_kind_of_base_and_is_not_monotone() {
        assert_eq!(edge_value(1), "-2");
        assert_eq!(edge_value(2), "1/4");
        assert_eq!(edge_value(3), "0");
        assert_eq!(multiplier(EDGE_INPUT, 3), 0.0);
        assert!(multiplier(EDGE_INPUT, 2) < 0.0 && multiplier(EDGE_INPUT, 4) > 0.0);
        assert!(endpoint(EDGE_INPUT, 2) > endpoint(EDGE_INPUT, 3) && endpoint(EDGE_INPUT, 3) < endpoint(EDGE_INPUT, 4));
        assert_eq!(edge_base(4), "1/4");
        assert_eq!(edge_base(16), "13/16");
        assert!(distance(EDGE_INPUT, 1024) < distance(EDGE_INPUT, 64));
    }
    #[test]
    fn the_walks_end_at_the_printed_endpoints() {
        for n in WALK_COUNTS {
            for x in [RADIUS, GRAPH_LOW] {
                let w = walk(x, n);
                assert!((w[n as usize].1 - endpoint(x, n)).abs() < 1e-12);
                assert!((w[n as usize].0 - x).abs() < 1e-12);
                assert_eq!(w[0], (0.0, 1.0));
            }
        }
        assert!((graph_x(GRAPH_SAMPLES) - RADIUS).abs() < 1e-12);
        assert_eq!(graph_x(0), GRAPH_LOW);
    }
    #[test]
    fn the_warning_sum_stays_at_one_while_every_fixed_column_falls() {
        for &n in &WARNING_COUNTS {
            let column_height = 1.0 / f64::from(n);
            assert!((column_height * f64::from(n) - 1.0).abs() < 1e-12);
            assert!(n as usize >= WARNING_COLUMN, "the watched column exists in every frame");
        }
        assert!(WARNING_COUNTS.windows(2).all(|p| p[0] < p[1]));
        // The ceiling a column would need: its largest value, at n = k. These do not add up to anything bounded.
        let partial = |top: u32| (1..=top).map(|k| 1.0 / f64::from(k)).sum::<f64>();
        assert!(partial(100) > 5.0 && partial(10_000) > 9.0);
        assert_eq!(
            FILL_TOP, WARNING_COUNTS[3] as usize,
            "the warning sum fills the same axis"
        );
    }
    #[test]
    fn the_zero_input_is_the_trivial_edge() {
        for n in [1u32, 4, 1024] {
            assert_eq!(endpoint(0.0, n), 1.0);
            assert_eq!(term(0.0, n, 0), 1.0);
            assert_eq!(term(0.0, n, 3), 0.0);
        }
    }
}
