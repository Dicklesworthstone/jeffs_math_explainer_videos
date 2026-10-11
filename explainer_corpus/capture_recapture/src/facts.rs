//! Every number Video A shows or says, computed here and checked two ways:
//! `pin` against a literal from an independent source (the fact sheet's
//! exact values), and `./x capture_recapture facts-check` against
//! `oracle.py` (same keys). Chapters read their numbers from here; a typed
//! number is a defect (doctrine 2).

use explainer_kit::{Facts, choose};

/// The second catch, marked count and caught-twice count of Feller's lake.
pub const FELLER: (u64, u64, u64) = (1_000, 1_000, 100);
/// The toy likelihood: marked, caught, caught twice.
pub const TOY: (u64, u64, u64) = (10, 10, 3);

/// P_N(K = k) as an exact fraction (numerator, denominator).
fn likelihood(n_lake: u64, (m, n, k): (u64, u64, u64)) -> (u128, u128) {
    (choose(m, k) * choose(n_lake - m, n - k), choose(n_lake, n))
}

/// The best lake sizes by exact comparison of neighbours, Feller's way:
/// L(N)/L(N−1) = (N−M)(N−n) / (N(N−M−n+k)), so the score rises while
/// Mn − kN > 0. Returns every N achieving the maximum.
pub fn best_sizes((m, n, k): (u64, u64, u64)) -> Vec<u64> {
    assert!(k > 0, "no best size when nobody is caught twice");
    let (m, n, k) = (m as i128, n as i128, k as i128);
    let lo = m + n - k;
    // Mn − kN changes sign once; the best N is ⌊Mn/k⌋, tied with ⌊Mn/k⌋ − 1
    // exactly when Mn/k is whole (and k < min(M, n))
    let best = (m * n) / k;
    let mut out = Vec::new();
    if (m * n) % k == 0 && best > lo && k < m.min(n) {
        out.push((best - 1) as u64);
    }
    out.push(best.max(lo) as u64);
    out
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    // 04: the toy lake, 20 fish, 5 marked, nets of 4
    f.pin("A-F1.C20_4", choose(20, 4), "4845", "fact sheet A-F1")?;
    f.pin(
        "A-F1.falling_product_20_19_18_17",
        20u64 * 19 * 18 * 17,
        "116280",
        "fact sheet A-F1",
    )?;
    f.pin("A-F1.orders_of_4_fish", 4u64 * 3 * 2, "24", "fact sheet A-F1")?;
    let piles: Vec<u128> = (0..=4).map(|k| choose(5, k) * choose(15, 4 - k)).collect();
    for (k, want) in ["1365", "2275", "1050", "150", "5"].iter().enumerate() {
        f.pin(&format!("A-F2.k{k}"), piles[k], want, "fact sheet A-F2")?;
    }
    f.pin(
        "A-F2.sum",
        piles.iter().sum::<u128>(),
        "4845",
        "fact sheet A-F2 (Vandermonde)",
    )?;
    f.pin("A-F4.C5_2", choose(5, 2), "10", "fact sheet A-F4")?;
    f.pin("A-F4.C15_2", choose(15, 2), "105", "fact sheet A-F4")?;
    f.pin("A-F4.product", choose(5, 2) * choose(15, 2), "1050", "fact sheet A-F4")?;
    // 05–08: the toy likelihood (marked 10, caught 10, caught twice 3)
    let (m, n, k) = TOY;
    f.pin("A-F5.min_lake", m + n - k, "17", "fact sheet A-F5")?;
    let toy_best = best_sizes(TOY);
    f.pin(
        "A-F5.argmax",
        toy_best.last().copied().unwrap_or(0),
        "33",
        "fact sheet A-F5",
    )?;
    // the neighbour comparisons the voice says (07_2): 529 vs 528, 576 vs 578
    let ratio = |lake: u64| -> (u64, u64) { ((lake - m) * (lake - n), lake * (lake - m - n + k)) };
    let (up_num, up_den) = ratio(33);
    let (down_num, down_den) = ratio(34);
    f.pin("A-F6.N33.ratio_numerator", up_num, "529", "fact sheet A-F6")?;
    f.pin("A-F6.N33.ratio_denominator", up_den, "528", "fact sheet A-F6")?;
    f.pin("A-F6.N34.ratio_numerator", down_num, "576", "fact sheet A-F6")?;
    f.pin("A-F6.N34.ratio_denominator", down_den, "578", "fact sheet A-F6")?;
    // the exact likelihood at 33 is 7429/23374 (≈ 0.318)
    let (num, den) = likelihood(33, TOY);
    let g = gcd(num, den);
    f.pin(
        "A-F5.L33.exact",
        format!("{}/{}", num / g, den / g),
        "7429/23374",
        "Ruth R1 check (exact)",
    )?;
    // 08: Feller's lake, the comparisons at the tie
    let (fm, fnn, fk) = FELLER;
    for lake in [9_999i64, 10_000, 10_001] {
        let v = fm as i64 * fnn as i64 - fk as i64 * lake;
        f.add(&format!("A-F10.N{lake}.Mn_minus_kN"), v)?;
    }
    let feller_best = best_sizes(FELLER);
    f.pin(
        "A-F9.argmax",
        feller_best.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
        "9999,10000",
        "fact sheet A-F9",
    )?;
    // 01–02: the table
    f.pin(
        "A-F29.feller_u_hat",
        (fm - fk) * (fnn - fk) / fk,
        "8100",
        "fact sheet A-F29",
    )?;
    f.pin("A-F11.min_lake", fm + fnn - fk, "1900", "fact sheet A-F11")?;
    // the dark cell balances the diagonals: k·u = a·b
    let (a, b, u) = (fm - fk, fnn - fk, (fm - fk) * (fnn - fk) / fk);
    f.pin("A-F29.feller_ab", a * b, "810000", "Maya R1 m5 (checked by hand)")?;
    f.pin(
        "A-F29.feller_k_times_u_hat",
        fk * u,
        "810000",
        "Maya R1 m5 (checked by hand)",
    )?;
    // 09: bending, 10⁶/90 and 10⁶/110
    f.add("A-F31.Mn_over_90_rounded", (fm * fnn + 45) / 90)?;
    f.add("A-F31.Mn_over_110_rounded", (fm * fnn + 55) / 110)?;
    // 14: two reviewers, 30 / 24 / 12
    f.pin(
        "A-F24.estimate.exact",
        format!("{}/1", 30u64 * 24 / 12),
        "60/1",
        "fact sheet A-F24",
    )?;
    f.pin(
        "A-F24.hiding",
        30u64 * 24 / 12 - (30 + 24 - 12),
        "18",
        "fact sheet A-F24",
    )?;
    f.pin("A-F24.seen", 30u64 + 24 - 12, "42", "fact sheet A-F24")?;
    Ok(f)
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_best_size_rule_matches_brute_force_on_small_lakes() {
        // ⌊Mn/k⌋, tied with one less when Mn/k is whole and k < min(M, n),
        // against the exact likelihood maximised over every lake size
        for m in 1..=8u64 {
            for n in 1..=8u64 {
                for k in 1..=m.min(n) {
                    let lo = m + n - k;
                    let mut best: Vec<u64> = Vec::new();
                    let mut top = (0u128, 1u128);
                    for lake in lo..=lo + 400 {
                        let (num, den) = likelihood(lake, (m, n, k));
                        // compare num/den with top.0/top.1 exactly
                        let lhs = num * top.1;
                        let rhs = top.0 * den;
                        if lhs > rhs {
                            top = (num, den);
                            best = vec![lake];
                        } else if lhs == rhs && num > 0 {
                            best.push(lake);
                        }
                    }
                    assert_eq!(best_sizes((m, n, k)), best, "M {m} n {n} k {k}");
                }
            }
        }
    }

    #[test]
    fn mn_minus_kn_is_the_tables_balance() {
        // Mn − kN = ab − k·u, exactly, for every small table (Ruth R-29)
        for m in 1..=12i64 {
            for n in 1..=12i64 {
                for k in 0..=m.min(n) {
                    for lake in (m + n - k)..=(m + n - k + 30) {
                        let (a, b, u) = (m - k, n - k, lake - m - n + k);
                        assert_eq!(m * n - k * lake, a * b - k * u);
                    }
                }
            }
        }
    }
}
