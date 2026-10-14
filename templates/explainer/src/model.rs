//! Pure mathematical model for `__SLUG__` (plan §3.5: `model.rs` is pure math
//! with unit tests; chapters only draw).

use explainer_kit::choose;

/// Total number of points in the demo configuration.
pub const POINTS: u64 = 5_000;

/// Number of points chosen per subset.
pub const PICK: u64 = 4;

/// Exact number of `k`-element subsets of an `n`-element set.
pub fn quadruples(n: u64, k: u64) -> u128 {
    choose(n, k)
}

/// Whole trillions (`10^12`) contained in `count`.
pub fn trillions_floor(count: u128) -> u128 {
    count / 1_000_000_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pattern 1 — narration truth: script.tsv 01_2 says
    /// "There are more than twenty-six trillion ways."
    #[test]
    fn narration_claims_hold_for_the_rendered_numbers() {
        let q = quadruples(POINTS, PICK);
        assert!(q > 26_000_000_000_000, "\"more than twenty-six trillion\"");
        assert!(
            q < 27_000_000_000_000,
            "and not so much more that \"twenty-six\" misleads"
        );
        assert_eq!(trillions_floor(q), 26);
    }

    /// Pattern 2 — brute force against the closed form on small sizes.
    #[test]
    fn closed_form_matches_enumeration() {
        for n in 0..=12u64 {
            let subsets_of_four = (0u32..(1 << n)).filter(|m| m.count_ones() == 4).count() as u128;
            assert_eq!(quadruples(n, 4), subsets_of_four, "n = {n}");
        }
    }

    /// Pattern 3 — edge cases of every "always/never" sentence.
    #[test]
    fn always_never_claims_hold_at_the_edges() {
        assert_eq!(quadruples(4, 4), 1, "C(n, n) = 1");
        assert_eq!(quadruples(0, 4), 0, "no quadruples among fewer than four points");
    }
}
