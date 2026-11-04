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
}
