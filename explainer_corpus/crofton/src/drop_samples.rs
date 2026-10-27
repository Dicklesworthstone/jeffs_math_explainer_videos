//! Retained inputs from the existing oracle's two seeded sheet experiments.
//! Coordinates round-trip binary64; native geometry computes every count.
use crate::{chapters::sheet, facts, model};

pub const SEED: u64 = 20261024;
pub const REPEAT_SEED: u64 = SEED + 2;
pub const REPEATS: usize = 1000;
pub const ERROR_CM: f64 = 1.;
#[derive(Clone, Copy, Debug)]
pub struct Drop {
    pub degrees: f64,
    pub slide: f64,
}
impl Drop {
    pub fn count(self, c: &model::Curve) -> usize {
        sheet::crossings(c, self.degrees.to_radians(), self.slide).len()
    }
}
fn parse(text: &str, seed: u64, count: usize) -> Result<Vec<Drop>, String> {
    let mut lines = text.lines();
    if lines.next() != Some(format!("# PCG64DXSM seed={seed}; beta_degrees\tslide_cm").as_str()) {
        return Err("drop seed/units header mismatch".into());
    }
    let mut out = Vec::new();
    for row in lines {
        let cells = row
            .split('\t')
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("invalid drop: {e}"))?;
        if cells.len() != 2
            || !cells.iter().all(|x| x.is_finite())
            || !(0. ..180.).contains(&cells[0])
            || !(0. ..model::GAP).contains(&cells[1])
        {
            return Err("drop needs angle in [0,180) and slide in [0,gap)".into());
        }
        out.push(Drop {
            degrees: cells[0],
            slide: cells[1],
        });
    }
    if out.len() != count {
        return Err(format!("expected {count} drops, got {}", out.len()));
    }
    Ok(out)
}
pub fn samples() -> Result<Vec<Drop>, String> {
    parse(include_str!("../data/drops-20261024.tsv"), SEED, facts::MANY_DROPS)
}
pub fn repeated() -> Result<Vec<Drop>, String> {
    parse(
        include_str!("../data/ten-drops-20261026.tsv"),
        REPEAT_SEED,
        REPEATS * facts::FEW_DROPS,
    )
}
pub fn counts(draws: &[Drop], c: &model::Curve) -> Vec<usize> {
    draws.iter().map(|d| d.count(c)).collect()
}
pub fn estimate(total: usize, n: usize) -> f64 {
    total as f64 / n as f64 * model::worth(model::GAP)
}
pub fn totals(counts: &[usize]) -> Vec<usize> {
    assert!(counts.len().is_multiple_of(facts::FEW_DROPS));
    counts
        .as_chunks::<{ facts::FEW_DROPS }>()
        .0
        .iter()
        .map(|row| row.iter().sum())
        .collect()
}
pub fn histogram(counts: &[usize]) -> Vec<usize> {
    let mut hist = vec![0; counts.iter().copied().max().unwrap_or(0) + 1];
    for &n in counts {
        hist[n] += 1;
    }
    hist
}
pub fn far(total: usize, c: &model::Curve) -> bool {
    (estimate(total, facts::FEW_DROPS) - c.length).abs() > ERROR_CM
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_draws_match_existing_oracle_counts() {
        let c = model::coast();
        let a = counts(&samples().unwrap(), &c);
        assert_eq!(&a[..10], &[7, 9, 7, 8, 11, 7, 9, 7, 9, 8]);
        assert_eq!(a.iter().min(), Some(&5));
        assert_eq!(a.iter().max(), Some(&12));
        let b = totals(&counts(&repeated().unwrap(), &c));
        assert_eq!(b.iter().filter(|&&x| far(x, &c)).count(), 430);
    }
    #[test]
    fn all_retained_counts_agree_with_model_meetings() {
        let c = model::coast();
        for d in samples().unwrap().into_iter().chain(repeated().unwrap()) {
            let n: usize = (-8..=8)
                .map(|j| c.meetings(d.degrees.to_radians(), d.slide + j as f64 * model::GAP))
                .sum();
            assert_eq!(d.count(&c), n);
        }
    }
    #[test]
    fn error_threshold_is_strict_and_parse_rejects_bad_inputs() {
        let c = model::coast();
        assert!(!far(82, &c));
        assert!(far(78, &c));
        assert!(far(86, &c));
        for row in ["nan\t0", "180\t0", "0\t2", "0\t-1", "0\t0\t0"] {
            assert!(
                parse(
                    &format!("# PCG64DXSM seed={SEED}; beta_degrees\tslide_cm\n{row}"),
                    SEED,
                    1
                )
                .is_err()
            );
        }
    }
}
