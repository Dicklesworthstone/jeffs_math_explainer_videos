//! The chosen models of chapters 10, 11 and 12. Every number here is invented for teaching: none is a measurement,
//! a forecast or a recommendation. Author source, not built or run.
use explainer_kit::{Facts, choose};

// ---------------------------------------------------------------- 10: a rate that changes with the clock

/// The chapter's clock runs from zero to here.
pub const HORIZON: f64 = 2.0;
pub const STEP_COUNTS: [u32; 4] = [4, 16, 64, 1024];

/// The rate at time t: from one, in a straight line through zero at time one, to minus one at time two.
pub fn rate(t: f64) -> f64 {
    1.0 - t
}
/// The signed area under the rate from zero to t. Its slope is the rate.
pub fn area(t: f64) -> f64 {
    t - t * t / 2.0
}
/// The amount, from a start of one: the exponential of the area so far.
pub fn amount(t: f64) -> f64 {
    area(t).exp()
}
/// The tiny multipliers with n cells, reading the rate at each cell's left edge: (time, amount), n + 1 points.
pub fn stepped(n: u32) -> Vec<(f64, f64)> {
    let width = HORIZON / f64::from(n);
    let mut y = 1.0;
    let mut out = vec![(0.0, y)];
    for j in 0..n {
        let t = f64::from(j) * width;
        y *= multiplier(t, width);
        out.push((f64::from(j + 1) * width, y));
    }
    out
}
/// One cell's multiplier.
pub fn multiplier(t: f64, width: f64) -> f64 {
    1.0 + rate(t) * width
}
pub fn stepped_end(n: u32) -> f64 {
    stepped(n).last().map_or(1.0, |p| p.1)
}
/// The same cells reading the rate at their midpoints instead. Four of them finish below the start, so it is the
/// left-edge reading, where this falling rate is highest, that lifts four cells above it (review 721 H2).
pub fn midpoint_end(n: u32) -> f64 {
    let width = HORIZON / f64::from(n);
    (0..n)
        .map(|j| multiplier((f64::from(j) + 0.5) * width, width))
        .product()
}

// ---------------------------------------------------------------- 11: quoted rates and credits

pub const BALANCE: f64 = 100.0;
pub const QUOTED: f64 = 0.12;
pub const CREDITS: [i32; 5] = [1, 2, 4, 12, 365];

/// The balance after one year, with the quoted rate credited `n` times.
pub fn credited(n: i32) -> f64 {
    BALANCE * (1.0 + QUOTED / f64::from(n)).powi(n)
}
/// The limit of `credited` as the credits become continuous.
pub fn continuous() -> f64 {
    BALANCE * QUOTED.exp()
}

// ---------------------------------------------------------------- 11: two years and one withdrawal

pub const GOOD: f64 = 1.2;
pub const BAD: f64 = 0.9;
pub const TAKEN: f64 = 50.0;

/// A year's multiplier as the continuous rate that gives it.
pub fn continuous_rate(multiplier: f64) -> f64 {
    multiplier.ln()
}
/// Two years with no money moved.
pub fn two_years(first: f64, second: f64) -> [f64; 2] {
    let a = BALANCE * first;
    [a, a * second]
}
/// Two years, with `TAKEN` removed between them: after year one, after the withdrawal, after year two.
pub fn with_withdrawal(first: f64, second: f64) -> [f64; 3] {
    let a = BALANCE * first;
    let b = a - TAKEN;
    [a, b, b * second]
}
/// What the removed money would have become over the year after it left.
pub fn cost_of_withdrawal(second: f64) -> f64 {
    TAKEN * second
}

// ---------------------------------------------------------------- 12: multipliers chosen by a fair coin

pub const HEADS: f64 = 1.5;
pub const TAILS: f64 = 0.6;
pub const TOSSES: u32 = 100;
/// The histogram shows this many heads.
pub const SHOWN_HEADS: (u32, u32) = (30, 70);
/// The rare runs whose share of the average the narration names.
pub const RARE_HEADS: u32 = 60;
/// How many heads more or fewer than the middle run 12_2 names.
pub const SPREAD_HEADS: i32 = 5;

/// A run's final multiple of its start, given how many of its tosses were heads.
pub fn outcome(heads: u32) -> f64 {
    HEADS.powi(heads as i32) * TAILS.powi((TOSSES - heads) as i32)
}
/// The chance of exactly this many heads in `TOSSES` fair tosses.
pub fn share(heads: u32) -> f64 {
    choose(u64::from(TOSSES), u64::from(heads)) as f64 / 2.0_f64.powi(TOSSES as i32)
}
/// The fewest heads that finish above the start.
pub fn above_start() -> u32 {
    (0..=TOSSES).find(|&h| outcome(h) > 1.0).unwrap_or(TOSSES)
}
/// The chance of at least `from` heads.
pub fn share_from(from: u32) -> f64 {
    (from..=TOSSES).map(share).sum()
}
/// The average final multiple across all runs.
pub fn average_multiple() -> f64 {
    ((HEADS + TAILS) / 2.0).powi(TOSSES as i32)
}
/// How much of that average comes from runs with at least `from` heads.
pub fn average_share_from(from: u32) -> f64 {
    (from..=TOSSES).map(|h| share(h) * outcome(h)).sum::<f64>() / average_multiple()
}
/// A run with heads and tails in equal number: the median run, since the outcome rises with the heads.
pub fn typical_multiple() -> f64 {
    (HEADS * TAILS).powi((TOSSES / 2) as i32)
}
/// What trading `swaps` tails for heads does to a run's result: each trade multiplies it by heads over tails.
pub fn heads_for_tails(swaps: i32) -> f64 {
    (HEADS / TAILS).powi(swaps)
}
/// One run with heads and tails in turn, heads first: its distance on the ruler, in inputs of E, after each toss.
pub fn alternating_run() -> Vec<f64> {
    let mut at = 0.0;
    let mut out = vec![at];
    for toss in 0..TOSSES {
        at += if toss.is_multiple_of(2) { HEADS.ln() } else { TAILS.ln() };
        out.push(at);
    }
    out
}

// ---------------------------------------------------------------- 12: feedback from a capacity

/// Both curves start here, as a fraction of a capacity of one. The rate is one. Same model as the earlier draft.
pub const CAP_START: f64 = 0.08;
pub const CAP_TIMES: [f64; 5] = [0.0, 0.5, 1.0, 1.5, 2.0];

pub fn unlimited(t: f64) -> f64 {
    CAP_START * t.exp()
}
/// Solves y' = y(1 - y), y(0) = CAP_START.
pub fn limited(t: f64) -> f64 {
    1.0 / (1.0 + (1.0 / CAP_START - 1.0) * (-t).exp())
}
/// When the unlimited curve reaches the capacity: the chart stops drawing it there instead of bending it.
pub fn unlimited_top() -> f64 {
    (1.0 / CAP_START).ln()
}
/// When the unlimited curve reaches `level`: the chart draws it past the capacity to its own top (review 721 L2).
pub fn unlimited_reaches(level: f64) -> f64 {
    (level / CAP_START).ln()
}
/// The distance of the reciprocal from the reciprocal of the capacity. It decays exactly exponentially.
pub fn reciprocal_gap(t: f64) -> f64 {
    1.0 / limited(t) - 1.0
}

pub fn append_facts(f: &mut Facts) -> Result<(), String> {
    for k in 0..=8_u32 {
        let t = f64::from(k) / 4.0;
        f.add(&format!("change.{k}.t"), format!("{t:.2}"))?;
        f.add(&format!("change.{k}.rate"), format!("{:.2}", rate(t)))?;
        f.add(&format!("change.{k}.area"), format!("{:.5}", area(t)))?;
        f.add(&format!("change.{k}.amount"), format!("{:.5}", amount(t)))?;
    }
    f.add("change.peak", format!("{:.5}", amount(1.0)))?;
    f.add("change.back", format!("{:.5}", amount(HORIZON)))?;
    for n in STEP_COUNTS {
        f.add(&format!("change.steps.{n}"), format!("{:.5}", stepped_end(n)))?;
    }
    for j in 0..4_u32 {
        let width = HORIZON / 4.0;
        f.add(
            &format!("change.cell.{j}"),
            format!("{:.2}", multiplier(f64::from(j) * width, width)),
        )?;
    }
    for n in CREDITS {
        f.add(&format!("money.credit.{n}"), format!("{:.2}", credited(n)))?;
    }
    f.add("money.continuous", format!("{:.2}", continuous()))?;
    f.add("money.rate.good", format!("{:.5}", continuous_rate(GOOD)))?;
    f.add("money.rate.bad", format!("{:.5}", continuous_rate(BAD)))?;
    f.add("money.rate.both", format!("{:.5}", continuous_rate(GOOD * BAD)))?;
    f.add("money.both", format!("{:.2}", GOOD * BAD))?;
    for (name, first, second) in [("good_first", GOOD, BAD), ("bad_first", BAD, GOOD)] {
        let [a, b] = two_years(first, second);
        f.add(&format!("money.{name}.year1"), format!("{a:.2}"))?;
        f.add(&format!("money.{name}.year2"), format!("{b:.2}"))?;
        let [c, d, e] = with_withdrawal(first, second);
        f.add(&format!("money.{name}.cash.year1"), format!("{c:.2}"))?;
        f.add(&format!("money.{name}.cash.after"), format!("{d:.2}"))?;
        f.add(&format!("money.{name}.cash.end"), format!("{e:.2}"))?;
        f.add(
            &format!("money.{name}.cost"),
            format!("{:.2}", cost_of_withdrawal(second)),
        )?;
    }
    f.add("coin.mean", format!("{:.2}", (HEADS + TAILS) / 2.0))?;
    f.add("coin.pair", format!("{:.2}", HEADS * TAILS))?;
    f.add("coin.heads_step", format!("{:.5}", HEADS.ln()))?;
    f.add("coin.tails_step", format!("{:.5}", TAILS.ln()))?;
    f.add("coin.pair_step", format!("{:.5}", (HEADS * TAILS).ln()))?;
    f.add("coin.average", format!("{:.2}", average_multiple()))?;
    f.add("coin.typical", format!("{:.5}", typical_multiple()))?;
    f.add("coin.above_heads", above_start())?;
    f.add("coin.above_share", format!("{:.5}", share_from(above_start())))?;
    f.add("coin.rare_heads", RARE_HEADS)?;
    f.add("coin.rare_share", format!("{:.5}", share_from(RARE_HEADS)))?;
    f.add(
        "coin.rare_average_share",
        format!("{:.5}", average_share_from(RARE_HEADS)),
    )?;
    for h in SHOWN_HEADS.0..=SHOWN_HEADS.1 {
        f.add(&format!("coin.share.{h}"), format!("{:.5}", share(h)))?;
    }
    f.add("cap.start", format!("{CAP_START:.2}"))?;
    f.add("cap.top", format!("{:.5}", unlimited_top()))?;
    for (k, t) in CAP_TIMES.into_iter().enumerate() {
        f.add(&format!("cap.{k}.t"), format!("{t:.1}"))?;
        f.add(&format!("cap.{k}.unlimited"), format!("{:.5}", unlimited(t)))?;
        f.add(&format!("cap.{k}.limited"), format!("{:.5}", limited(t)))?;
        f.add(&format!("cap.{k}.ratio"), format!("{:.5}", unlimited(t) / limited(t)))?;
        f.add(&format!("cap.{k}.gap"), format!("{:.5}", reciprocal_gap(t)))?;
    }
    // Stage10, review 721: the midpoint reading behind H2, and the numbers of the middle run and its spread (M1, L3).
    f.add("change.midpoint.4", format!("{:.5}", midpoint_end(4)))?;
    f.add("coin.swap", format!("{:.2}", heads_for_tails(1)))?;
    f.add("coin.five_heads", format!("{:.5}", heads_for_tails(SPREAD_HEADS)))?;
    let finish = alternating_run().last().copied().unwrap_or(0.0);
    f.add("coin.alternating_end", format!("{finish:.5}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_area_has_the_rate_as_its_slope() {
        for k in 0..=20 {
            let t = f64::from(k) / 10.0;
            let h = 1e-6;
            assert!(((area(t + h) - area(t - h)) / (2.0 * h) - rate(t)).abs() < 1e-8, "{t}");
        }
    }
    #[test]
    fn the_amount_obeys_its_rate_law_and_returns_to_the_start() {
        for k in 1..20 {
            let t = f64::from(k) / 10.0;
            let h = 1e-6;
            let slope = (amount(t + h) - amount(t - h)) / (2.0 * h);
            assert!((slope - rate(t) * amount(t)).abs() < 1e-6, "{t}");
        }
        assert_eq!(amount(0.0), 1.0);
        assert!((amount(HORIZON) - 1.0).abs() < 1e-12);
        assert!((amount(1.0) - 0.5_f64.exp()).abs() < 1e-12);
    }
    #[test]
    fn the_cells_close_in_on_the_exact_ending_from_above() {
        let ends: Vec<f64> = STEP_COUNTS.iter().map(|&n| stepped_end(n)).collect();
        assert!(ends.windows(2).all(|p| p[1] < p[0]));
        assert!(ends.iter().all(|&e| e > 1.0));
        assert!(ends[3] - 1.0 < 0.002);
        assert_eq!(multiplier(0.0, 0.5), 1.5);
        assert_eq!(multiplier(1.0, 0.5), 1.0);
        assert_eq!(multiplier(1.5, 0.5), 0.75);
    }
    #[test]
    fn four_cells_finish_above_because_they_read_the_left_edges() {
        assert_eq!(stepped_end(4), 1.40625);
        // Read at the left edges, the four rate-times-width terms sum to one half. Read at the midpoints they sum to
        // zero, and the cells finish below the start, by about a third of a cell width.
        let width = HORIZON / 4.0;
        let left: f64 = (0..4).map(|j| rate(f64::from(j) * width) * width).sum();
        let middle: f64 = (0..4).map(|j| rate((f64::from(j) + 0.5) * width) * width).sum();
        assert!((left - 0.5).abs() < 1e-12 && middle.abs() < 1e-12);
        assert_eq!(midpoint_end(4), 0.845_947_265_625);
        let below = 1.0 - midpoint_end(1024);
        assert!(below > 0.0 && (below - HORIZON / 1024.0 / 3.0).abs() < 1e-5, "{below}");
    }
    #[test]
    fn more_credits_raise_the_balance_by_less_and_less_up_to_the_limit() {
        let all: Vec<f64> = CREDITS.iter().map(|&n| credited(n)).collect();
        assert!(all.windows(2).all(|p| p[1] > p[0]));
        assert!(all.iter().all(|&x| x < continuous()));
        let rises: Vec<f64> = all.windows(2).map(|p| p[1] - p[0]).collect();
        assert!(rises.windows(2).all(|r| r[1] < r[0]));
        assert_eq!(format!("{:.2}", credited(365)), format!("{:.2}", continuous()));
    }
    #[test]
    fn order_is_irrelevant_until_money_moves() {
        assert!((two_years(GOOD, BAD)[1] - two_years(BAD, GOOD)[1]).abs() < 1e-12);
        assert!((continuous_rate(GOOD) + continuous_rate(BAD) - continuous_rate(GOOD * BAD)).abs() < 1e-14);
        let (good, bad) = (with_withdrawal(GOOD, BAD)[2], with_withdrawal(BAD, GOOD)[2]);
        assert!((good - 63.0).abs() < 1e-9 && (bad - 48.0).abs() < 1e-9);
        // Each answer is the undisturbed total, less what the removed money would have become.
        assert!((two_years(GOOD, BAD)[1] - cost_of_withdrawal(BAD) - good).abs() < 1e-9);
        assert!((two_years(BAD, GOOD)[1] - cost_of_withdrawal(GOOD) - bad).abs() < 1e-9);
    }
    #[test]
    fn the_coin_separates_the_average_from_the_typical_run() {
        assert!(((HEADS + TAILS) / 2.0 - 1.05).abs() < 1e-12);
        const { assert!(HEADS * TAILS < 1.0) };
        assert!(HEADS.ln() + TAILS.ln() < 0.0 && TAILS.ln().abs() > HEADS.ln());
        assert!(average_multiple() > 131.0 && typical_multiple() < 0.0052);
        assert!((share_from(0) - 1.0).abs() < 1e-12);
        assert!((average_share_from(0) - 1.0).abs() < 1e-9);
        assert_eq!(above_start(), 56);
        assert!(outcome(55) < 1.0 && outcome(56) > 1.0);
        assert!(share_from(RARE_HEADS) < 0.03 && average_share_from(RARE_HEADS) > 0.99);
        // Fifty heads is the median: at most half the runs fall on either side of it.
        assert!(share_from(51) <= 0.5 && share_from(50) >= 0.5);
    }
    #[test]
    fn the_middle_run_spreads_by_about_a_hundred_per_five_heads_and_alternation_ends_on_it() {
        assert!((heads_for_tails(1) - 2.5).abs() < 1e-12);
        assert!((heads_for_tails(SPREAD_HEADS) - 97.656_25).abs() < 1e-9);
        for heads in 45..=55_u32 {
            let ratio = outcome(heads + 5) / outcome(heads);
            assert!((ratio - heads_for_tails(SPREAD_HEADS)).abs() < 1e-6 * ratio, "{heads}");
        }
        let run = alternating_run();
        assert_eq!(run.len(), TOSSES as usize + 1);
        assert_eq!(run[0], 0.0);
        let finish = run[run.len() - 1];
        assert!((finish - 50.0 * (HEADS * TAILS).ln()).abs() < 1e-11);
        assert!((finish.exp() - typical_multiple()).abs() < 1e-12);
        assert_eq!(format!("{finish:.3}"), "-5.268");
        // Every pair of tosses moves one pair step left, so the run's lowest point is its end.
        assert!(run.iter().all(|&d| d >= finish - 1e-12));
    }
    #[test]
    fn the_capacity_curve_tracks_the_exponential_early_and_its_reciprocal_gap_decays() {
        for k in 0..=40 {
            let t = f64::from(k) / 10.0;
            let h = 1e-6;
            let slope = (limited(t + h) - limited(t - h)) / (2.0 * h);
            assert!((slope - limited(t) * (1.0 - limited(t))).abs() < 1e-6, "{t}");
            assert!(limited(t) <= unlimited(t) + 1e-12 && limited(t) < 1.0);
            let ratio = reciprocal_gap(t + 1.0) / reciprocal_gap(t);
            assert!((ratio - (-1.0_f64).exp()).abs() < 1e-9, "{t}");
        }
        assert!((unlimited(0.5) / limited(0.5) - 1.0) < 0.06);
        assert!((unlimited(1.0) / limited(1.0) - 1.0) < 0.15);
        assert!((unlimited(unlimited_top()) - 1.0).abs() < 1e-12);
        assert!((unlimited_reaches(1.0) - unlimited_top()).abs() < 1e-15);
        assert!((unlimited(unlimited_reaches(1.15)) - 1.15).abs() < 1e-12 && unlimited_reaches(1.15) < 5.0);
    }
}
