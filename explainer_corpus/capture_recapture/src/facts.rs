//! Every number Video A shows or says, computed here and checked two ways:
//! `pin` against a literal from an independent source (the fact sheet's
//! exact values), and `./x capture_recapture facts-check` against
//! `oracle.py` (same keys). Chapters read their numbers from here; a typed
//! number is a defect (doctrine 2).

use explainer_kit::{Facts, choose};

use crate::model::mixing::{self, Mixing};
use crate::model::{Lake, SPOTS};

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
    // 01: the seeded clustered lake the scene draws (model.rs = oracle A-F27)
    let lake = Lake::draw();
    f.pin(
        "A-F27.rejection_candidates",
        lake.candidates,
        "139559",
        "oracle A-F27 (SplitMix64 rejection)",
    )?;
    // the dips the voice says (01_5, 01_7): weeds, open water, deep end
    for ((name, spot), (fish_want, estimate_want)) in
        SPOTS
            .into_iter()
            .zip([("880", "88000"), ("28", "2800"), ("451", "45100")])
    {
        let fish = lake.count_at(spot);
        f.pin(
            &format!("A-F27.spot.{name}.fish_in_net"),
            fish,
            fish_want,
            "01_5-01_7 voice",
        )?;
        f.pin(
            &format!("A-F27.spot.{name}.estimate"),
            fish * 100,
            estimate_want,
            "01_5-01_7 voice",
        )?;
    }
    f.add("A-F27.all_cells.min_fish", lake.all_counts().min().unwrap_or(0))?;
    f.add("A-F27.all_cells.max_fish", lake.all_counts().max().unwrap_or(0))?;
    f.pin(
        "A-F27.all_cells.dips_within_2x_of_truth",
        lake.dips_within_2x(),
        "22",
        "01_8 lower-third",
    )?;
    f.add("A-F27.cell_counts_rows_south_to_north", lake.counts_string())?;
    // 03: the mixing the scene runs (model::mixing = oracle A-CH03.portable)
    let join = |v: &[u32]| v.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
    let mut mix = Mixing::start(&lake);
    let (all, marked) = mix.regions();
    f.add("A-CH03.portable.t0.count", join(&all))?;
    f.add("A-CH03.portable.t0.marked", join(&marked))?;
    mix.advance(mixing::STEPS);
    let (all, marked) = mix.regions();
    f.add("A-CH03.portable.t400.count", join(&all))?;
    f.add("A-CH03.portable.t400.marked", join(&marked))?;
    const REGIONS: [&str; 9] = ["SW", "S", "SE", "W", "C", "E", "NW", "N", "NE"];
    let most = (0..9).max_by_key(|&r| all[r]).unwrap_or(0);
    let least = (0..9).min_by_key(|&r| all[r]).unwrap_or(0);
    for (tag, r, (region, fish, marks)) in [
        ("most_crowded", most, ("SE", "2811", "282")),
        ("least_crowded", least, ("NE", "204", "25")),
    ] {
        let pre = format!("A-CH03.portable.corners.{tag}");
        f.pin(&format!("{pre}.region"), REGIONS[r], region, "03_6 picture")?;
        f.pin(&format!("{pre}.fish"), all[r], fish, "03_6 voice")?;
        f.pin(&format!("{pre}.marked"), marked[r], marks, "03_6 voice")?;
    }
    let nets: Vec<u32> = mixing::region_centres().iter().map(|&c| mix.net_at(c, 1_000)).collect();
    f.pin(
        "A-CH03.portable.nets1000.marked_at_region_centres",
        join(&nets),
        "90,93,103,100,94,100,105,100,99",
        "03_5 picture",
    )?;
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
    late_chapters(&mut f)?;
    Ok(f)
}

/// Laplace's 1802 figures, recorded from the *Essai philosophique* (tr.
/// Truscott & Emory 1902, pp. 66–67): inhabitants counted in the chosen
/// communes, boys and girls born there in 1800–1802, and the yearly births
/// of France he assumed.
pub const LAPLACE: (u64, u64, u64, u64) = (2_037_615, 110_312, 105_287, 1_000_000);
/// Kosovo, recorded (Ball et al. 2002, p. 6; Spagat 2014, p. 9): unique
/// named victims, the estimate killed, its 95% range, the later
/// name-by-name count.
pub const KOSOVO: (u64, u64, (u64, u64), u64) = (4_400, 10_356, (9_002, 12_122), 9_790);
/// The two reviewers of 14_4 (a made-up example): bugs found by each, and
/// by both.
pub const REVIEWERS: (u64, u64, u64) = (30, 24, 12);

/// A float as Python's `repr` spells it (the oracle's format): the same
/// shortest round-trip digits as Rust's `{:?}`, with Python's exponent
/// (`1e-05`, `1e+16`).
pub fn py(x: f64) -> String {
    let s = format!("{x:?}");
    match s.split_once('e') {
        Some((mantissa, exp)) => {
            let (sign, digits) = exp.strip_prefix('-').map_or(("+", exp), |d| ("-", d));
            format!("{mantissa}e{sign}{digits:0>2}")
        }
        None => s,
    }
}

/// Rounded to `places` decimals, as Python's `round(x, places)` gives for
/// values away from a tie.
pub fn round_to(x: f64, places: i32) -> f64 {
    let scale = 10f64.powi(places);
    (x * scale).round() / scale
}

/// Chapters 12–15: the careful range and the ring toss (12), the failure
/// modes (13), Laplace and the reviewers (14), Kosovo (15).
fn late_chapters(f: &mut Facts) -> Result<(), String> {
    use crate::law::{Law, careful_range};
    use crate::runs::{self, BUGS, FAILURE_RUNS, Review};

    let (fm, fnn, fk) = FELLER;
    let lake = runs::LAKE as u64;
    // 12_1–12_3: the careful range for Feller's catch, and the true lake's
    // middle 95%, the catches whose ranges ring it
    let (lo, hi) = careful_range(fm, fnn, fk)?;
    f.pin("A-F13.ci95_lo", lo, "8400", "fact sheet A-F13")?;
    f.pin("A-F13.ci95_hi", hi, "12073", "fact sheet A-F13")?;
    let truth = Law::new(lake, fm, fnn);
    let (ring_lo, ring_hi) = truth.middle()?;
    f.pin(
        "A-CH12.ring_toss.ringing_k",
        format!("{ring_lo},{ring_hi}"),
        "83,118",
        "oracle ch12_ring_toss (exact integers)",
    )?;
    let coverage = truth.middle_mass()?;
    f.add(
        "A-CH12.ring_toss.exact_coverage_N10000_pct_2dp",
        py(round_to(coverage * 100.0, 2)),
    )?;
    // 12_2: the rings are the 09_6 panel's own experiments, in run order
    let panel = runs::panel();
    let rings = |k: u32| (ring_lo..=ring_hi).contains(&u64::from(k));
    let first100_missed: Vec<String> = (0..100)
        .filter(|&i| !rings(panel[i]))
        .map(|i| (i + 1).to_string())
        .collect();
    let pre = "A-CH12.ring_toss_panel";
    f.add(&format!("{pre}.first100_ringed"), 100 - first100_missed.len())?;
    f.add(&format!("{pre}.first100_missed"), first100_missed.len())?;
    f.pin(
        &format!("{pre}.first100_missed_runs_1based"),
        first100_missed.join(","),
        "13,71",
        "storyboard 12_2",
    )?;
    f.add(&format!("{pre}.all_runs"), panel.len())?;
    f.pin(
        &format!("{pre}.all_missed"),
        panel.iter().filter(|&&k| !rings(k)).count(),
        "464",
        "storyboard 12_2",
    )?;
    f.add(
        &format!("{pre}.expected_missed_rounded"),
        ((1.0 - coverage) * panel.len() as f64).round() as u64,
    )?;
    // 12_3: chapter 09's range, the soup guesses at the true lake's 2.5%
    // and 97.5% points (exact law of Mn/K given K > 0)
    let (q025, q975) = truth.estimate_quantiles()?;
    f.add("A-F19.exact_q025_k", q025)?;
    f.add("A-F19.exact_q975_k", q975)?;
    // 12_5: the slider, lake fixed at 10,000, M = n = √(10,000 k)
    let mut widths = Vec::new();
    for k in [25u64, 100, 400] {
        let m = (lake * k).isqrt();
        let pre = format!("A-CH12.slider.k{k}");
        f.add(&format!("{pre}.M_eq_n"), m)?;
        f.add(&format!("{pre}.M_eq_n_is_whole"), m * m == lake * k)?;
        let (lo, hi) = careful_range(m, m, k)?;
        widths.push((hi - lo) as f64);
        f.add(&format!("{pre}.ci95"), format!("{lo},{hi}"))?;
        f.add(&format!("{pre}.width"), hi - lo)?;
        let pct = |end: u64| py((100 * (end as i64 - lake as i64)) as f64 / lake as f64);
        f.add(&format!("{pre}.lo_pct"), pct(lo))?;
        f.add(&format!("{pre}.hi_pct"), pct(hi))?;
        let share = 1.0 - m as f64 / lake as f64;
        f.add(
            &format!("{pre}.typical_error_pct"),
            py(round_to(100.0 * (share * share / k as f64).sqrt(), 6)),
        )?;
        f.add(
            &format!("{pre}.one_over_sqrt_k_pct"),
            py(round_to(100.0 / (k as f64).sqrt(), 6)),
        )?;
    }
    f.add("A-CH12.slider.width_ratio_k25_to_k100", py(widths[1] / widths[0]))?;
    f.add("A-CH12.slider.width_ratio_k100_to_k400", py(widths[2] / widths[1]))?;
    let (lo, hi) = careful_range(lake, lake, lake)?;
    f.add("A-CH12.slider.whole_lake.ci95", format!("{lo},{hi}"))?;

    // 13: verify2.py's failure modes, replayed draw for draw
    let v = runs::verify2();
    f.pin(
        "A-CH12.ring_toss.all_missed",
        v.feller_k
            .iter()
            .filter(|&&k| !(ring_lo as i64..=ring_hi as i64).contains(&k))
            .count(),
        "447",
        "oracle verify2_replay section E (the HRUA draws before section F)",
    )?;
    f.add("A-F22.params", runs::failure_params())?;
    for (failure, want) in v.failures.iter().zip([
        "10000.0",
        "7751.937984496124",
        "7142.857142857143",
        "14492.753623188406",
        "12500.0",
        "12121.657361151925",
        "10000.0",
        "6250.0",
        "25000.0",
    ]) {
        assert_eq!(failure.estimates.len(), FAILURE_RUNS);
        f.pin(
            &format!("A-F22.median.{}", failure.key),
            py(failure.median()),
            want,
            "fact sheet A-F22 (verify2_out.txt section F)",
        )?;
    }
    // what the soup guess aims at under each model: Mn/E[k], E[k] exact
    for (name, (num, den)) in runs::targets() {
        let value = if num % den == 0 {
            (num / den).to_string()
        } else {
            py(num as f64 / den as f64)
        };
        f.add(&format!("A-CH13.{name}.target_Mn_over_Ek"), value)?;
    }
    let (left_num, left_den) = runs::deaths_left();
    f.pin(
        "A-CH13.deaths_30pct.expected_lake_at_second_catch",
        left_num / left_den,
        "7000",
        "storyboard 13_8",
    )?;
    f.pin(
        "A-CH13.newcomers_2000.lake_at_second_catch",
        lake + runs::NEWCOMERS as u64,
        "12000",
        "storyboard 13_6",
    )?;

    // 14: Laplace's arithmetic (inputs recorded, A-F23)
    let (people, boys, girls, assumed) = LAPLACE;
    f.add("A-F23.recorded.inhabitants", people)?;
    f.add("A-F23.recorded.births_boys_1800_1802", boys)?;
    f.add("A-F23.recorded.births_girls_1800_1802", girls)?;
    f.add("A-F23.recorded.assumed_births_of_France", assumed)?;
    let births3 = boys + girls;
    f.pin("A-F23.births_3yr", births3, "215599", "fact sheet A-F23")?;
    f.add("A-F23.births_per_year", py(births3 as f64 / 3.0))?;
    f.add("A-F23.births_per_year.exact", format!("{births3}/3"))?;
    f.pin(
        "A-F23.ratio_6dp",
        format!("{:.6}", (3 * people) as f64 / births3 as f64),
        "28.352845",
        "fact sheet A-F23",
    )?;
    let est = (3 * people * assumed + births3 / 2) / births3;
    f.pin(
        "A-F23.population_estimate_rounded",
        est,
        "28352845",
        "Laplace's printed 28352845 (Essai)",
    )?;
    // the two reviewers (A-F24) and the careful range for them
    let (ra, rb, rk) = REVIEWERS;
    f.add("A-F24.reviewer_a", ra)?;
    f.add("A-F24.reviewer_b", rb)?;
    f.add("A-F24.shared", rk)?;
    let (lo, hi) = careful_range(ra, rb, rk)?;
    f.pin("A-F24.ci95_lo", lo, "47", "fact sheet A-F24")?;
    f.pin("A-F24.ci95_hi", hi, "94", "fact sheet A-F24")?;
    // 14_6: the reviewers' models, run as the oracle runs them
    f.add("A-CH14.reviewers.params", runs::review_params())?;
    for (name, reviews) in runs::reviewers() {
        let pre = format!("A-CH14.reviewers.{name}");
        let found: Vec<&Review> = reviews.iter().filter(|r| r.both > 0).collect();
        let estimates: Vec<f64> = found.iter().map(|r| (r.a * r.b) as f64 / r.both as f64).collect();
        f.add(&format!("{pre}.true_bugs"), BUGS)?;
        f.add(&format!("{pre}.runs_k0"), reviews.len() - found.len())?;
        f.add(&format!("{pre}.sim_median_estimate"), py(runs::median(&estimates)))?;
        let mut sides = [0usize; 3];
        for r in &found {
            sides[runs::range_side(r)? as usize] += 1;
        }
        let share = |n: usize| py(n as f64 / found.len() as f64);
        f.add(&format!("{pre}.ci95_wholly_below_true_share"), share(sides[0]))?;
        f.add(&format!("{pre}.ci95_contains_true_share"), share(sides[1]))?;
        f.add(&format!("{pre}.ci95_wholly_above_true_share"), share(sides[2]))?;
    }

    // 15: Kosovo (recorded, A-F25)
    let (named, estimate, (k_lo, k_hi), later) = KOSOVO;
    f.add("A-F25.recorded.unique_named_victims", named)?;
    f.add("A-F25.recorded.estimate", estimate)?;
    f.add("A-F25.recorded.ci95_lo", k_lo)?;
    f.add("A-F25.recorded.ci95_hi", k_hi)?;
    f.add("A-F25.recorded.later_count_kmb", later)?;
    f.pin(
        "A-F25.later_count_inside_interval",
        (k_lo..=k_hi).contains(&later),
        "true",
        "Spagat 2014, p. 9",
    )?;
    Ok(())
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
