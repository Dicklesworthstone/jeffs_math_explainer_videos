//! Source-only model for the expanded native rate-law chapter.
pub const START: f64 = 1.0;
pub const DURATION: f64 = 1.0;
pub const STEP_COUNTS: [u32; 3] = [4, 8, 16];

/// Forward Euler on y'=y. Every plotted endpoint follows this recurrence.
pub fn proportional_steps(n: u32) -> Vec<(f64, f64)> {
    assert!(n > 0);
    let h = DURATION / f64::from(n);
    let mut y = START;
    let mut points = vec![(0.0, y)];
    for k in 1..=n {
        y += h * y;
        points.push((f64::from(k) * h, y));
    }
    points
}
pub fn constant_steps(n: u32) -> Vec<(f64, f64)> {
    assert!(n > 0);
    (0..=n)
        .map(|k| {
            let t = f64::from(k) * DURATION / f64::from(n);
            (t, START + t)
        })
        .collect()
}
/// What a panel reads at a checkpoint: its height, then the slope its rule takes from it.
pub fn reading_label(height: f64, rate: f64) -> String {
    format!("height {height:.2}: slope {rate:.2}")
}
/// The slope the last step kept, beside the height that step reached.
pub fn last_step_label(slope: f64, height: f64) -> String {
    format!("last step: slope {slope:.2}, height now {height:.2}")
}
/// The endpoints drawn so far, oldest first, so the shrinking gaps stay readable.
pub fn endpoints_label(counts: &[u32]) -> String {
    let steps: Vec<String> = counts.iter().map(u32::to_string).collect();
    let ends: Vec<String> = counts
        .iter()
        .map(|&n| format!("{:.3}", proportional_steps(n)[n as usize].1))
        .collect();
    format!("{} steps: {}", steps.join(", "), ends.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_step_agrees_but_the_next_slope_does_not() {
        let p = proportional_steps(4);
        let c = constant_steps(4);
        assert_eq!(p[1], c[1]);
        assert_eq!(p[1].1, 1.25);
        assert_eq!(p[2].1, 1.5625);
        assert_eq!(c[2].1, 1.5);
        assert_eq!(p[4].1, 625.0 / 256.0);
    }
    #[test]
    fn displayed_refinements_stay_on_the_same_domain_and_improve_endpoint() {
        let mut previous = 0.0;
        for n in STEP_COUNTS {
            let p = proportional_steps(n);
            assert_eq!(p.len(), n as usize + 1);
            assert_eq!(p[0], (0.0, START));
            assert_eq!(p[n as usize].0, DURATION);
            let end = p[n as usize].1;
            assert!(end > previous && end < std::f64::consts::E);
            previous = end;
        }
    }
    #[test]
    fn every_printed_number_is_the_exact_rational_rounded() {
        let p = proportional_steps(4);
        let c = constant_steps(4);
        let right: Vec<String> = p.iter().map(|&(_, y)| reading_label(y, y)).collect();
        assert_eq!(
            right,
            [
                "height 1.00: slope 1.00",
                "height 1.25: slope 1.25",
                "height 1.56: slope 1.56",
                "height 1.95: slope 1.95",
                "height 2.44: slope 2.44",
            ]
        );
        assert_eq!(reading_label(c[4].1, 1.0), "height 2.00: slope 1.00");
        assert_eq!(
            last_step_label(p[3].1, p[4].1),
            "last step: slope 1.95, height now 2.44"
        );
        assert_eq!(endpoints_label(&STEP_COUNTS[..1]), "4 steps: 2.441");
        assert_eq!(endpoints_label(&STEP_COUNTS[..2]), "4, 8 steps: 2.441, 2.566");
        assert_eq!(endpoints_label(&STEP_COUNTS), "4, 8, 16 steps: 2.441, 2.566, 2.638");
    }
    #[test]
    fn each_refinement_climbs_above_the_coarser_drawing_and_the_end_gap_shrinks() {
        for pair in STEP_COUNTS.windows(2) {
            let (coarse, fine) = (proportional_steps(pair[0]), proportional_steps(pair[1]));
            let ratio = (pair[1] / pair[0]) as usize;
            for (k, point) in coarse.iter().enumerate().skip(1) {
                assert!(fine[k * ratio].1 > point.1);
            }
        }
        let end = |n: u32| proportional_steps(n)[n as usize].1;
        assert!(end(16) - end(8) < end(8) - end(4));
    }
}
