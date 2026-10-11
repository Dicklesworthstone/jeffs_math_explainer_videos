//! Every number Video A shows or says, computed here and checked two ways:
//! `pin` against a literal from an independent source (the fact sheet's
//! exact values), and `./x capture_recapture facts-check` against
//! `oracle.py` (same keys). Chapters read their numbers from here; a typed
//! number is a defect (doctrine 2).

use explainer_kit::{Facts, choose};

use crate::exact::{Dd, estimate_law, feller_ratio, hypergeometric};
use crate::model::mixing::{self, Mixing};
use crate::model::{Lake, SPOTS};
use crate::panel;

/// The second catch, marked count and caught-twice count of Feller's lake.
pub const FELLER: (u64, u64, u64) = (1_000, 1_000, 100);
/// The toy likelihood: marked, caught, caught twice.
pub const TOY: (u64, u64, u64) = (10, 10, 3);
/// Chapter 04's toy lake: fish, marked, the net's size.
pub const NETFULS: (u64, u64, u64) = (20, 5, 4);

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
    chapters_07_to_11(&mut f)?;
    chapters_04_to_06(&mut f)?;
    Ok(f)
}

/// Chapters 04-06: the netfuls' chances, the mystery lake's stills and
/// sums, all from the toy lakes' own counts.
fn chapters_04_to_06(f: &mut Facts) -> Result<(), String> {
    // 04: the five piles over every netful, exact and as the screen rounds
    let (lake, marked, net) = NETFULS;
    let all = choose(lake, net);
    let falling: u64 = (0..net).map(|j| lake - j).product();
    let orders: u64 = (1..=net).product();
    f.pin("A-F1.falling_over_orders", falling / orders, "4845", "04_3 screen")?;
    let whole = ["28", "47", "22", "3", "<1"];
    for k in 0..=net {
        let pile = choose(marked, k) * choose(lake - marked, net - k);
        let g = gcd(pile, all);
        f.add(&format!("A-F3.p{k}.exact"), format!("{}/{}", pile / g, all / g))?;
        let pct = 100.0 * pile as f64 / all as f64;
        let shown = if pct < 1.0 {
            "<1".to_owned()
        } else {
            format!("{}", pct.round())
        };
        f.pin(
            &format!("A-F3.screen_whole_pct{k}"),
            shown,
            whole[k as usize],
            "04_8 voice",
        )?;
    }
    // 05: the mystery lake's three stills and the partial sum the screen shows
    let score = |n: u64| {
        let (num, den) = likelihood(n, TOY);
        num as f64 / den as f64
    };
    for (n, want) in [(20u64, "7.8"), (33, "31.8"), (60, "15.9")] {
        f.pin(
            &format!("A-F5.still_N{n}_pct"),
            py(round_to(100.0 * score(n), 1)),
            want,
            "05_4 voice",
        )?;
    }
    let (m, c, k) = TOY;
    let first = m + c - k;
    let partial: f64 = (first..=60).map(score).sum();
    f.pin("A-F5.area_17_to_60", py(round_to(partial, 2)), "9.89", "05_7 screen")?;
    // every size's score adds to Mn/(k(k−1)) when k ≥ 2 (A-F7; Ruth's proof)
    let (num, den) = (m * c, k * (k - 1));
    let g = gcd(u128::from(num), u128::from(den)) as u64;
    f.pin(
        "A-F7.toy_area_closed_form.exact",
        format!("{}/{}", num / g, den / g),
        "50/3",
        "05_7 voice",
    )?;
    f.pin(
        "A-F7.toy_area_pct_rounded",
        (100.0 * num as f64 / den as f64).round(),
        "1667",
        "05_7 screen",
    )?;
    Ok(())
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
    // a catch with nobody caught twice: open above (Codex review A-3 R1)
    let (zero_lo, zero_hi) = crate::law::careful_range_any(fm, fnn, 0)?;
    f.pin(
        "A-CH12.zero_catch.ci95_lo",
        zero_lo,
        "272086",
        "Codex review A-3 R1 (exact integers)",
    )?;
    f.add(
        "A-CH12.zero_catch.ci95_hi",
        zero_hi.map_or_else(|| "none".to_owned(), |hi| hi.to_string()),
    )?;
    // 12_3's lower-third: Feller's own range and its one-sided chances
    let (bet_lo, bet_hi) = crate::law::FELLER_BET;
    f.add("A-F14.recorded.feller_bet_range", format!("{bet_lo},{bet_hi}"))?;
    let (low, high) = crate::law::range_tails(fm, fnn, fk, crate::law::FELLER_BET);
    f.add(
        "A-F14.pct_1dp",
        format!(
            "{:.1},{:.1},{:.1}",
            100.0 * low,
            100.0 * high,
            100.0 * (1.0 - low - high)
        ),
    )?;
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
    let (run1_lo, run1_hi) = careful_range(fm, fnn, u64::from(panel[0]))?;
    f.add(
        &format!("{pre}.run1_k_ci95"),
        format!("{},{run1_lo},{run1_hi}", panel[0]),
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
        // the gauge's reading (13)
        f.add(
            &format!("A-F22.median_rounded_10.{}", failure.key),
            ((failure.median() / 10.0).round() * 10.0) as u64,
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
    // 14_2's screen: "≈ 28.35 people per birth", "≈ 28.4 million people"
    f.add(
        "A-F23.ratio_2dp",
        format!("{:.2}", (3 * people) as f64 / births3 as f64),
    )?;
    f.add(
        "A-F23.population_estimate_millions_1dp",
        format!("{:.1}", est as f64 / 1e6),
    )?;
    // the two reviewers (A-F24) and the careful range for them
    let (ra, rb, rk) = REVIEWERS;
    f.add("A-F24.reviewer_a", ra)?;
    f.add("A-F24.reviewer_b", rb)?;
    f.add("A-F24.shared", rk)?;
    f.add("A-F24.a_only", ra - rk)?;
    f.add("A-F24.b_only", rb - rk)?;
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
        f.add(
            &format!("{pre}.ci95_missed_pct_0dp"),
            (100.0 * (sides[0] + sides[2]) as f64 / found.len() as f64).round() as u64,
        )?;
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

/// Chapters 07–11 (lane mx-e-a-8y5.9): Feller's comparison, the table's
/// balance, the panel's 10,000 experiments, the small study, how sure.
fn chapters_07_to_11(f: &mut Facts) -> Result<(), String> {
    // 07: the comparison at the mystery lake's peak, factor by factor in
    // table words (07_2) and Mn − kN (07_4)
    let (m, n, k) = TOY;
    for lake in [33u64, 34] {
        let pre = format!("A-F6.N{lake}");
        f.add(&format!("{pre}.unmarked_N_minus_M"), lake - m)?;
        f.add(&format!("{pre}.missed_by_second_N_minus_n"), lake - n)?;
        f.add(&format!("{pre}.dark_cell_u"), lake + k - m - n)?;
        f.add(&format!("{pre}.Mn"), m * n)?;
        f.add(&format!("{pre}.kN"), k * lake)?;
        f.add(&format!("{pre}.Mn_minus_kN"), (m * n) as i64 - (k * lake) as i64)?;
    }
    // 07_5–07_6: the same number from the table's cells, 7 × 7 − 3 × 16
    let (a, b) = (m - k, n - k);
    let u = a * b / k;
    f.pin("A-F29.toy_a", a, "7", "07_6 voice")?;
    f.pin("A-F29.toy_b", b, "7", "07_6 voice")?;
    f.pin("A-F29.toy_u_hat", u, "16", "fact sheet A-F29")?;
    f.add("A-F29.toy_N_hat", a + b + k + u)?;
    f.pin("A-F29.toy_ab", a * b, "49", "07_6 voice")?;
    f.pin("A-F29.toy_k_times_u_hat", k * u, "48", "07_6 voice")?;
    // 08: Feller's table at the balance, and the neighbouring ratios (08_5)
    let (fm, fnn, fk) = FELLER;
    f.add("A-F29.feller_a", fm - fk)?;
    f.add("A-F29.feller_b", fnn - fk)?;
    let truth = fm * fnn / fk;
    f.add("A-F29.green_share_marked_column_N10000", py(fk as f64 / fm as f64))?;
    f.add(
        "A-F29.green_share_unmarked_column_N10000",
        py((fnn - fk) as f64 / (truth - fm) as f64),
    )?;
    for (lake, (num_want, den_want)) in [
        (9_999u64, ("80982001", "80981901")),
        (10_000, ("81000000", "81000000")),
        (10_001, ("81018001", "81018101")),
    ] {
        let (num, den) = feller_ratio(lake, FELLER);
        f.pin(
            &format!("A-F10.N{lake}.ratio_numerator"),
            num,
            num_want,
            "fact sheet A-F10",
        )?;
        f.pin(
            &format!("A-F10.N{lake}.ratio_denominator"),
            den,
            den_want,
            "fact sheet A-F10",
        )?;
    }
    // the soup guess at the mystery lake, 33⅓ → 33 (08_4)
    let g = gcd((m * n) as u128, k as u128);
    f.add("A-F5.soup", py((m * n) as f64 / k as f64))?;
    f.pin(
        "A-F5.soup.exact",
        format!("{}/{}", (m * n) as u128 / g, k as u128 / g),
        "100/3",
        "fact sheet A-F5",
    )?;
    // 09: the code panel's 10,000 experiments (src/panel.rs replays numpy)
    let panel = panel::run();
    let s = panel.stats()?;
    let pre = "A-CH09.panel";
    f.pin(
        &format!("{pre}.k_polyhash"),
        panel::polyhash(&panel.k),
        "746464907710603445",
        "numpy 2.5.4 run of the panel",
    )?;
    f.add(&format!("{pre}.experiments"), panel.k.len())?;
    f.add(
        &format!("{pre}.k_zero_count"),
        panel.k.iter().filter(|&&k| k == 0).count(),
    )?;
    f.pin(
        &format!("{pre}.first12_k"),
        panel.k[..12].iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        "101,103,91,97,94,103,100,91,108,101,90,107",
        "fact sheet R2 (09_1 listen window)",
    )?;
    f.add(&format!("{pre}.k_min"), s.k_min)?;
    f.add(&format!("{pre}.k_max"), s.k_max)?;
    f.add(&format!("{pre}.k_within_9_of_100_share"), py(s.k_within_9_share))?;
    f.add(&format!("{pre}.sim_k_mean"), py(s.k_mean))?;
    f.pin(
        &format!("{pre}.sim_k_sd"),
        py(s.k_sd),
        "8.960120782668056",
        "fact sheet R2 (SD of k 8.960)",
    )?;
    f.add(&format!("{pre}.sim_mean"), py(s.mean))?;
    f.pin(
        &format!("{pre}.sim_mean_rounded_to_10"),
        (s.mean / 10.0).round() as u64 * 10,
        "10090",
        "09_5 voice",
    )?;
    f.add(&format!("{pre}.sim_median"), py(s.median))?;
    f.add(&format!("{pre}.sim_p2_5"), py(s.p2_5))?;
    f.add(&format!("{pre}.sim_p97_5"), py(s.p97_5))?;
    f.add(&format!("{pre}.exact_line_mean"), py(s.exact_mean))?;
    f.pin(
        &format!("{pre}.print_line"),
        format!("{} {}", py(s.mean), py(s.exact_mean)),
        "10088.586031204479 10082.498763370464",
        "fact sheet R2 (the panel's printed line, numpy 2.5.4)",
    )?;
    f.add(&format!("{pre}.sim_mean_standard_error"), py(s.standard_error))?;
    f.add(
        &format!("{pre}.mean_gap_in_standard_errors"),
        py(s.gap_in_standard_errors),
    )?;
    // the exact spread of k (09_3 "SD 9.0"): n·M(N−M)(N−n) / (N²(N−1))
    let (vn, vd) = (
        u128::from(fnn * fm) * u128::from(truth - fm) * u128::from(truth - fnn),
        u128::from(truth * truth) * u128::from(truth - 1),
    );
    let g = gcd(vn, vd);
    f.pin(
        "A-F15.var_k_exact.exact",
        format!("{}/{}", vn / g, vd / g),
        "90000/1111",
        "fact sheet A-F15",
    )?;
    f.add("A-F15.sd_k_exact", py(((vn / g) as f64 / (vd / g) as f64).sqrt()))?;
    // 09_4: ten too few pushes harder than ten too many
    let near = |c: u64| (fm * fnn + c / 2) / c;
    f.pin("A-F31.excess_above_10000", near(90) - truth, "1111", "fact sheet A-F31")?;
    f.pin(
        "A-F31.shortfall_below_10000",
        truth - near(110),
        "909",
        "fact sheet A-F31",
    )?;
    // 10: the small study, N = 1,000, M = n = 50 (all exact, then rounded once)
    let (sn, sm) = (1_000u64, 50u64);
    let law = estimate_law(sn, sm, sm);
    f.pin(
        "A-F21.P_k0",
        py(law.p0),
        "0.07197699188484312",
        "fact sheet A-F21 (7.20%)",
    )?;
    f.add("A-F21.one_in", py(law.one_in))?;
    f.add("A-F21.chapman_exact_mean", py(law.chapman_mean))?;
    f.add("A-F21.chapman_bias_pct", py(law.chapman_bias_pct))?;
    f.add("A-F21.plain_mean_given_k_pos", py(law.plain_mean))?;
    f.add("A-F21.plain_bias_pct_given_k_pos", py(law.plain_bias_pct))?;
    f.pin(
        "A-F21.chapman_at_k0",
        (sm + 1) * (sm + 1) - 1,
        "2600",
        "fact sheet claim walk (10_3 notes)",
    )?;
    let g = gcd(u128::from(sm * sm), u128::from(sn));
    f.add("A-F21.expected_k", py((sm * sm) as f64 / sn as f64))?;
    f.add(
        "A-F21.expected_k.exact",
        format!("{}/{}", u128::from(sm * sm) / g, u128::from(sn) / g),
    )?;
    f.add("A-F18.bound_pct_at_expected_k4", py(100.0 * (-4.0f64).exp()))?;
    f.add("A-CH09.numpy_estimate_at_k0", py((sm * sm) as f64 / 0.0))?;
    // 11: the smallest lake's chance, in scientific form and in zeros
    let p1900 = hypergeometric(fm + fnn - fk, FELLER);
    let (mant, ex) = p1900.sci10();
    f.add("A-F11.P1900_exponent_exact", ex)?;
    f.pin(
        "A-F11.P1900_sci_2sf",
        format!("{mant:.1}e{ex}"),
        "5.4e-430",
        "fact sheet A-F11",
    )?;
    let digits = p1900.mul(Dd::pow10(ex.unsigned_abs()));
    let zero_at: Vec<u32> = (1..=ex.unsigned_abs())
        .filter(|&z| digits.div(Dd::pow10(z)).to_f64() == 0.0)
        .collect();
    let first_zero = zero_at.first().copied().unwrap_or(0);
    f.pin(
        "A-F32.zeros.last_Z_float64_nonzero",
        first_zero - 1,
        "324",
        "fact sheet claim walk (11_2 marker)",
    )?;
    f.add("A-F32.zeros.first_Z_float64_zero", first_zero)?;
    f.add(
        "A-F32.zeros.all_later_Z_zero",
        zero_at == (first_zero..=ex.unsigned_abs()).collect::<Vec<_>>(),
    )?;
    f.add(
        "A-F32.smallest_subnormal_first_nonzero_decimal_place",
        -Dd::pow2(-1074).sci10().1,
    )?;
    // 11_3: a million fish, about one marked fish expected
    let million = 1_000_000u64;
    let g = gcd(u128::from(fm * fnn), u128::from(million));
    f.add("A-CH11.expected_k_N1000000", py((fm * fnn) as f64 / million as f64))?;
    f.add(
        "A-CH11.expected_k_N1000000.exact",
        format!("{}/{}", u128::from(fm * fnn) / g, u128::from(million) / g),
    )?;
    // 11_4: the top score; 11_5: fifty heads in a hundred flips
    f.pin(
        "A-F9.L_max",
        py(hypergeometric(truth, FELLER).to_f64()),
        "0.04428917172067476",
        "fact sheet A-F9 (0.0442892)",
    )?;
    let coin = Dd::int128(choose(100, 50)).div(Dd::pow2(100)).to_f64();
    f.add("A-F30.P50_of_100", py(coin))?;
    f.pin(
        "A-F30.P50_of_100_pct_2dp",
        py(round_to(coin * 100.0, 2)),
        "7.96",
        "fact sheet A-F30",
    )?;
    f.add("A-F30.50_is_mode", (0..=100).all(|j| choose(100, 50) >= choose(100, j)))?;
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
