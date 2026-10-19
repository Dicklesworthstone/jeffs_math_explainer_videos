//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x randomized_response
//! facts-check`): the rows of docs/fact-sheet.md. R-F rows are computed,
//! R-X rows are the edges of the always/never sentences. Rows R-F1 to R-F5
//! are chapter 01's, R-F6, R-F11, R-F12 and R-X5 chapter 02's, R-F7, R-F8,
//! R-F13 and R-F14 chapter 03's; R-F9 and R-F10 belong to the planned
//! chapter 04 (storyboard, "The arc"), computed now so the earlier
//! chapters' promises are checked against them.

use explainer_kit::Facts;
use explainer_kit::stats::Data;

use crate::model::{
    self, ASKED_AGAIN, COIN_PAIRS, Channel, Class, Coin, MAX_ASKS, Q, Route, Tally, by_yeses, csv, likelihood_ratio,
    pattern_ratio,
};

/// How many asks R-F9.worst_ratio_by_asks lists.
const WORST_LISTED: u32 = 5;

/// RAPPOR's example (its section 1): yeses out of answers, from an even prior.
const RAPPOR_EXAMPLE: (u32, u32) = (75, 100);

/// Chapter 03's real classes: the seed (the chapter's number, fixed before
/// any draw was looked at, the only one tried) and how many classes.
pub const SEED: u64 = 3;
pub const RUNS: usize = 8;

/// Chapter 03's eight classes, asked with fresh coins: one raw draw each
/// from the seeded generator (NumPy's `PCG64DXSM(3).random_raw()`, which the
/// oracle replays), read two bits a student.
pub fn runs() -> Result<Vec<Vec<(Coin, Coin)>>, String> {
    let mut data = Data::new(SEED);
    (0..RUNS)
        .map(|_| model::coins_from_bits(data.raw(), model::CLASS_SIZE))
        .collect()
}

/// The nearest whole number to `q`, halves up: floor(q + 1/2) =
/// floor((2q + 1) / 2), in exact arithmetic.
fn nearest(q: Q) -> i128 {
    let twice = Q::int(2) * q + Q::ONE;
    twice.num().div_euclid(2 * twice.den())
}

/// The nearest whole "one in k" to a chance `p`: k = round(1/p).
fn one_in(p: Q) -> Result<i128, String> {
    let k = Q::ONE.checked_div(p).ok_or("a chance of 0 is not one in anything")?;
    Ok(nearest(k))
}

/// The class's prior share of people who copied, as chapter 02 uses it.
fn prior(class: &Class) -> Q {
    Q::new(class.copiers() as i128, class.size() as i128)
}

/// Chapter 01's class, with every seat's coins landing on average and where
/// they sent each answer.
pub struct Average {
    pub class: Class,
    pub coins: Vec<(Coin, Coin)>,
    pub tally: Tally,
}

/// The class on average (01_5 to 01_7).
pub fn average() -> Result<Average, String> {
    let class = Class::video()?;
    let coins = class.average_coins()?;
    let tally = Tally::of(&class, &coins)?;
    Ok(Average { class, coins, tally })
}

/// Chapter 01's undoing of the coins (01_9, 01_10): the second coin's yeses
/// on average, the yeses left, and the estimate. It is compared with the
/// dial's estimate 2 · yes − n/2 (`Channel::estimate`); both compute the same
/// formula today, so the comparison guards a later edit of one of them, not
/// independent arithmetic (the oracle is that).
pub fn undo(a: &Average) -> Result<[Q; 3], String> {
    let (yes, n) = (a.tally.answers(true), a.class.size());
    let steps = model::undo(yes, n);
    let dial = Channel::two_coins()
        .estimate(yes as i128, n as i128)
        .ok_or("the two coins give no estimate")?;
    if steps[2] != dial {
        return Err(format!("undoing the coins gives {}, the estimate {dial}", steps[2]));
    }
    Ok(steps)
}

/// A whole number from an exact fraction that must be one.
fn whole(q: Q, what: &str) -> Result<i128, String> {
    q.integer().ok_or_else(|| format!("{what} is {q}, not a whole number"))
}

/// The largest pattern ratio after `asks` fresh answers.
fn worst_after(asks: u32) -> Result<Q, String> {
    model::repeated_answers(asks)?
        .into_iter()
        .map(pattern_ratio)
        .try_fold(Q::ZERO, |m, r| r.map(|r| m.max(r)))
        .ok_or_else(|| format!("after {asks} answers some pattern never happens to someone who didn't copy"))
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let dwork_roth = "Dwork and Roth, The Algorithmic Foundations of Differential Privacy (2014), section 3.2, \
                      claim 3.5: Pr[Yes|Yes] = 3/4, Pr[Yes|No] = 1/4, both ratios 3, (ln 3, 0)-private";
    let expected = "Dwork and Roth, section 2.3: the expected yes share is (1/4)(1 - p) + (3/4)p = 1/4 + p/2";
    let rappor = "Erlingsson, Pihur and Korolova, RAPPOR (CCS 2014), section 1: 75 yes of 100 answers leave No \
                  a 1.39 x 10^-24 fraction of cases";

    // 01_1, 01_2: the class (invented for the video)
    let a = average()?;
    let (n, copiers) = (a.class.size(), a.class.copiers());
    f.add("R-F1.class", n)?;
    f.add("R-F1.columns", model::CLASS_COLUMNS)?;
    f.add("R-F1.rows", n.div_ceil(model::CLASS_COLUMNS))?;
    f.add("R-F1.copied", copiers)?;
    f.add("R-F1.copied_seats", csv(&model::COPIERS))?;
    f.add("R-F1.copied_share", prior(&a.class))?;

    // 01_3, 01_4: the rule, counted over the two coins' four outcomes
    // two fair coins, four equally likely outcomes
    f.add("R-F2.coins", COIN_PAIRS.len().ilog2())?;
    f.add("R-F2.coin_outcomes", COIN_PAIRS.len())?;
    f.add("R-F2.yes_outcomes.copied", model::outcomes_saying(true, true))?;
    f.add("R-F2.yes_outcomes.didnt", model::outcomes_saying(false, true))?;
    f.pin("R-F2.p_yes.copied", model::p_said(true, true), "3/4", dwork_roth)?;
    f.pin("R-F2.p_yes.didnt", model::p_said(false, true), "1/4", dwork_roth)?;
    f.add("R-F2.p_no.copied", model::p_said(true, false))?;
    f.add("R-F2.p_no.didnt", model::p_said(false, false))?;
    let share = prior(&a.class);
    let yes_share = share * model::p_said(true, true) + (Q::ONE - share) * model::p_said(false, true);
    f.pin(
        "R-F2.yes_share",
        yes_share,
        &(Q::new(1, 4) + share / Q::int(2)).to_string(),
        expected,
    )?;
    let routes = Route::ALL.map(|r| {
        let hits = COIN_PAIRS.iter().filter(|&&(x, y)| model::route(x, y) == r).count();
        Q::new(hits as i128, COIN_PAIRS.len() as i128)
    });
    f.add("R-F2.route_chances", csv(&routes))?;

    // 01_5 to 01_7: the class on average
    let t = &a.tally;
    let truthful = t.route(Route::Truth);
    f.add("R-F3.truthful", truthful)?;
    f.add("R-F3.truthful_share", Q::new(truthful as i128, n as i128))?;
    f.add("R-F3.truthful.yes", t.count(Route::Truth, true))?;
    f.add("R-F3.truthful.no", t.count(Route::Truth, false))?;
    f.add("R-F3.forced", t.route(Route::ForcedYes) + t.route(Route::ForcedNo))?;
    f.add("R-F3.forced.yes", t.route(Route::ForcedYes))?;
    f.add("R-F3.forced.no", t.route(Route::ForcedNo))?;
    f.add("R-F3.forced.yes.copied", t.count(Route::ForcedYes, true))?;
    f.add("R-F3.forced.no.copied", t.count(Route::ForcedNo, true))?;
    let (yes, no) = (t.answers(true), t.answers(false));
    f.add("R-F3.yes", yes)?;
    f.add("R-F3.no", no)?;
    // 01_8's "those two numbers": the survey sees one count per answer
    f.add("R-F3.counts_seen", [yes, no].len())?;
    if Q::new(yes as i128, n as i128) != yes_share {
        return Err(format!("{yes} of {n} is not the average yes share {yes_share}"));
    }

    // 01_9, 01_10: undo the coins
    let [forced, left, estimate] = undo(&a)?;
    // only the half that tells the truth reports it: double what is left
    let factor = Q::ONE / Channel::two_coins().truth();
    f.add("R-F4.forced_yes", whole(forced, "the second coin's yeses")?)?;
    f.add("R-F4.forced_yes_share", forced / Q::int(n as i128))?;
    f.add("R-F4.left", whole(left, "the yeses left")?)?;
    f.add("R-F4.factor", whole(factor, "the factor")?)?;
    f.add("R-F4.estimate", whole(estimate, "the estimate")?)?;
    f.add("R-F4.matches_truth", estimate == Q::int(copiers as i128))?;
    f.add("R-F4.twice_yes", factor * Q::int(yes as i128))?;
    f.add("R-F4.half_class", Q::int(n as i128) / factor)?;

    // 01_11: what one answer carries, both answers
    f.pin("R-F5.ratio.yes", likelihood_ratio(true), "3", dwork_roth)?;
    f.pin("R-F5.ratio.no", likelihood_ratio(false), "3", dwork_roth)?;
    let worst = Channel::two_coins()
        .worst_ratio()
        .ok_or("the two coins leave a ratio unbounded")?;
    f.add("R-F5.worst_ratio", worst)?;
    f.pin(
        "R-F5.epsilon",
        format!("{:.4}", worst.to_f64().ln()),
        "1.0986",
        dwork_roth,
    )?;

    // chapter 02: who the yeses and the noes are, on average
    f.add("R-F6.yes.copied", t.said(true, true))?;
    f.add("R-F6.no.copied", t.said(true, false))?;
    let given_yes = Q::new(t.said(true, true) as i128, yes as i128);
    let given_no = Q::new(t.said(true, false) as i128, no as i128);
    for (said, counted) in [(1, given_yes), (0, given_no)] {
        let bayes = model::posterior(share, said, 1);
        if bayes != counted {
            return Err(format!("counted {counted} copiers per answer, Bayes {bayes}"));
        }
    }
    f.add("R-F6.posterior.yes", given_yes)?;
    f.add("R-F6.posterior.no", given_no)?;
    f.add("R-F6.percent.prior", share.percent())?;
    f.add("R-F6.percent.yes", given_yes.percent())?;
    f.add("R-F6.percent.no", given_no.percent())?;

    // 02_1 to 02_8: every student with every coin outcome, counted
    let c = model::class_outcomes(&a.class);
    let outcomes = COIN_PAIRS.len() as u64;
    let others = n - copiers;
    f.add("R-F11.class_outcomes", c.iter().flatten().sum::<u64>())?;
    f.add("R-F11.others", others)?;
    f.add("R-F11.outnumber", Q::new(others as i128, copiers as i128))?;
    for (said, name) in [(true, "yes"), (false, "no")] {
        let [didnt, copied] = c[usize::from(said)];
        f.add(&format!("R-F11.{name}.copied"), copied)?;
        f.add(&format!("R-F11.{name}.didnt"), didnt)?;
        f.add(&format!("R-F11.{name}"), didnt + copied)?;
        let counted = model::share_who_copied(&c, said).ok_or("an answer no outcome gives")?;
        let tallied = if said { given_yes } else { given_no };
        let bayes = model::posterior(share, u32::from(said), 1);
        if counted != tallied || counted != bayes {
            return Err(format!(
                "a {name}: counted {counted}, on average {tallied}, Bayes {bayes}"
            ));
        }
        f.add(&format!("R-F11.posterior.{name}"), counted)?;
        // the same outcomes on average: a quarter of them, one per outcome
        if !copied.is_multiple_of(outcomes) || !(didnt + copied).is_multiple_of(outcomes) {
            return Err(format!("the {name} outcomes do not divide into {outcomes}"));
        }
        let (on_average, from_copiers) = ((didnt + copied) / outcomes, copied / outcomes);
        if on_average as usize != t.answers(said) || from_copiers as usize != t.said(true, said) {
            return Err(format!(
                "the {name} outcomes over {outcomes} are not the class on average"
            ));
        }
        f.add(&format!("R-F11.average.{name}"), on_average)?;
        f.add(&format!("R-F11.average.{name}.copied"), from_copiers)?;
    }

    // 02_9, 02_10: the odds the coin multiplies, and where they start
    let prior_odds = model::odds(share).ok_or("everyone copied")?;
    let yes_odds = model::odds(given_yes).ok_or("a yes is certain")?;
    let no_odds = model::odds(given_no).ok_or("a no is certain")?;
    if yes_odds != prior_odds * likelihood_ratio(true) || no_odds != prior_odds / likelihood_ratio(false) {
        return Err(format!(
            "odds {prior_odds} do not become {yes_odds} and {no_odds} by the ratios"
        ));
    }
    f.add("R-F12.odds.prior", prior_odds)?;
    f.add("R-F12.odds.yes", yes_odds)?;
    f.add("R-F12.odds.no", no_odds)?;
    // a class where half copied: there a yes means the copier's own chance
    let half = n / 2;
    let half_class = Class::new(n, &(0..half).collect::<Vec<usize>>())?;
    let at_half = model::share_who_copied(&model::class_outcomes(&half_class), true).ok_or("no yes outcome")?;
    f.add("R-F12.half_class", half)?;
    f.add("R-F12.posterior.yes_at_half", at_half)?;
    // the edges of "where the chance ends up depends on the class"
    let none = model::share_who_copied(&model::class_outcomes(&Class::new(n, &[])?), true).ok_or("no yes")?;
    let all_seats: Vec<usize> = (0..n).collect();
    let all = model::share_who_copied(&model::class_outcomes(&Class::new(n, &all_seats)?), false).ok_or("no no")?;
    f.add("R-X5.posterior.yes_none_copied", none)?;
    f.add("R-X5.posterior.no_all_copied", all)?;

    // chapter 03: how a real class of twenty scatters
    let dist = model::yes_count_distribution(n, copiers)?;
    let (mean, var) = (model::mean(&dist), model::variance(&dist));
    f.add("R-F7.outcomes", (COIN_PAIRS.len() as i128).pow(n as u32))?;
    f.add("R-F7.mean_yes", mean)?;
    f.add("R-F7.var_yes", var)?;
    let var_estimate = factor * factor * var;
    f.add("R-F7.var_estimate", var_estimate)?;
    f.add("R-F7.sd_estimate", format!("{:.2}", var_estimate.to_f64().sqrt()))?;
    let per_person = Channel::two_coins()
        .coin_variance()
        .ok_or("the two coins have no coin variance")?;
    f.add("R-F7.coin_variance_per_person", per_person)?;
    if var_estimate != per_person * Q::int(n as i128) {
        return Err(format!(
            "the estimate's variance {var_estimate} is not {per_person} a person"
        ));
    }
    let exactly = dist[whole(mean, "the mean yes count")? as usize];
    f.add("R-F7.p_exactly_right", exactly)?;
    f.add("R-F7.percent.exactly_right", exactly.percent())?;
    let negative = model::p_negative_estimate(&dist);
    f.add("R-F7.p_negative", negative)?;
    f.add("R-F7.percent.negative", negative.percent())?;

    // chapter 03: independent draws (with replacement, or from a
    // large population) at the class's share
    let coins = model::sampling_variance(Channel::two_coins(), share, 1).ok_or("no sampling variance")?;
    let direct = model::direct_variance(share, 1);
    if coins != direct + per_person {
        return Err(format!(
            "{coins} with the coins is not {direct} plus the coins' {per_person}"
        ));
    }
    f.add("R-F8.variance_n1.direct", direct)?;
    f.add("R-F8.variance_n1.coins", coins)?;
    f.add("R-F8.cost", coins / direct)?;
    f.add("R-F8.cost_2dp", format!("{:.2}", (coins / direct).to_f64()))?;
    // the coins' part alone, against the direct question's whole variance
    f.add("R-F8.added_cost", per_person / direct)?;
    f.add(
        "R-F8.warner_extra",
        Channel::two_coins().warner_extra_variance().ok_or("no Warner term")?,
    )?;

    // 03_1, 03_2: the class asked eight times, fresh coins from the seed
    let mut yeses = Vec::with_capacity(RUNS);
    for drawn in runs()? {
        yeses.push(Tally::of(&a.class, &drawn)?.answers(true));
    }
    let estimates: Vec<Q> = yeses.iter().map(|&y| model::undo(y, n)[2]).collect();
    f.add("R-F13.seed", SEED)?;
    f.add("R-F13.runs", RUNS)?;
    f.add("R-F13.yes", csv(&yeses))?;
    f.add("R-F13.estimates", csv(&estimates))?;
    let lowest = estimates.iter().copied().min().ok_or("no runs")?;
    let highest = estimates.iter().copied().max().ok_or("no runs")?;
    f.add("R-F13.lowest", lowest)?;
    f.add("R-F13.highest", highest)?;
    let truth = Q::int(copiers as i128);
    f.add("R-F13.on_four", estimates.iter().filter(|&&e| e == truth).count())?;
    f.add("R-F13.below_zero", estimates.iter().filter(|&&e| e < Q::ZERO).count())?;

    // 03_3 to 03_10: the exact distribution read for the scene
    let estimate_mean = model::estimate_mean(&dist);
    if estimate_mean != truth {
        return Err(format!(
            "the estimate averages {estimate_mean}, not the {truth} who copied"
        ));
    }
    f.add("R-F14.estimate_mean", estimate_mean)?;
    let configurations = (COIN_PAIRS.len() as i128).pow(n as u32);
    f.add("R-F14.outcomes_trillions", configurations / 10i128.pow(12))?;
    f.add("R-F14.one_in.exactly", one_in(exactly)?)?;
    f.add("R-F14.one_in.negative", one_in(negative)?)?;
    let clamped = model::clamped_mean(&dist);
    if clamped <= estimate_mean {
        return Err(format!("rounding negatives up leaves the average at {clamped}"));
    }
    f.add("R-F14.clamp_at", Q::ZERO)?;
    f.add("R-F14.clamped_mean", clamped)?;
    f.add("R-F14.clamped_mean_2dp", format!("{:.2}", clamped.to_f64()))?;
    let answer_variance = Channel::two_coins().answer_variance(true);
    if answer_variance != Channel::two_coins().answer_variance(false) || answer_variance * Q::int(n as i128) != var {
        return Err(format!(
            "{answer_variance} an answer does not add up to the count's {var}"
        ));
    }
    f.add("R-F14.answer_variance", answer_variance)?;
    let sd = var_estimate.to_f64().sqrt();
    f.add("R-F14.sd_rounded", sd.round())?;
    // "almost six times as many people": the total cost, just under its
    // nearest whole number
    let cost = coins / direct;
    let rounded = nearest(cost);
    if Q::int(rounded) <= cost {
        return Err(format!("{cost} is not just under {rounded}"));
    }
    f.add("R-F14.cost_rounded", rounded)?;

    // chapter 04 (planned): one person asked three times, fresh coins
    let asks = ASKED_AGAIN;
    let counts = by_yeses(&model::repeated_answers(asks)?);
    f.add("R-F9.asks", asks)?;
    f.add("R-F9.outcomes", (COIN_PAIRS.len() as u64).pow(asks))?;
    let column = |i: usize| counts.iter().map(|c| c[i]).collect::<Vec<u64>>();
    f.add("R-F9.by_yeses.didnt", csv(&column(0)))?;
    f.add("R-F9.by_yeses.copied", csv(&column(1)))?;
    let all_yes = counts[asks as usize];
    f.add("R-F9.all_yes.copied", all_yes[1])?;
    f.add("R-F9.all_yes.didnt", all_yes[0])?;
    let ratios: Vec<Q> = counts
        .iter()
        .map(|&c| pattern_ratio(c).ok_or("a pattern no one who didn't copy gives"))
        .collect::<Result<_, _>>()?;
    f.add("R-F9.ratios", csv(&ratios))?;
    let after_three = model::posterior(share, asks, asks);
    f.add("R-F9.posterior.all_yes", after_three)?;
    f.add("R-F9.percent.all_yes", after_three.percent())?;
    let majority = model::majority_right(asks)?;
    f.add("R-F9.majority_right", majority)?;
    f.add("R-F9.percent.majority_right", majority.percent())?;
    let worst_by_asks: Vec<Q> = (1..=WORST_LISTED).map(worst_after).collect::<Result<_, _>>()?;
    f.add("R-F9.worst_ratio_by_asks", csv(&worst_by_asks))?;
    let memo = model::memoized_answers(asks)?;
    let memo_ratio = pattern_ratio(memo[(1 << asks) - 1]).ok_or("the memoized all-yes never happens")?;
    f.add("R-F10.memoized.ratio", memo_ratio)?;

    // the edges of "three times, never more", "on average" and "four"
    let direct_question = Channel::new(Q::ONE)?;
    f.add(
        "R-X1.direct.worst_ratio",
        direct_question
            .worst_ratio()
            .map_or("unbounded".into(), |r| r.to_string()),
    )?;
    let pure_coin = Channel::new(Q::ZERO)?;
    f.add(
        "R-X1.pure_coin.worst_ratio",
        pure_coin.worst_ratio().map_or("unbounded".into(), |r| r.to_string()),
    )?;
    f.add(
        "R-X1.pure_coin.estimate",
        pure_coin
            .estimate(yes as i128, n as i128)
            .map_or("none".into(), |e| e.to_string()),
    )?;
    let [_, _, lowest] = model::undo(0, n);
    let [_, _, highest] = model::undo(n, n);
    f.add("R-X2.estimate.no_yes", lowest)?;
    f.add("R-X2.estimate.all_yes", highest)?;
    f.add("R-X4.average_needs_multiple_of", COIN_PAIRS.len())?;
    let (rappor_yes, rappor_asks) = RAPPOR_EXAMPLE;
    let no_after = Q::ONE - model::posterior(Q::new(1, 2), rappor_yes, rappor_asks);
    f.pin(
        "R-X3.rappor.no_after_75_of_100",
        format!("{:.2e}", no_after.to_f64()),
        "1.39e-24",
        rappor,
    )?;
    let memo_max = (1..=MAX_ASKS)
        .map(|k| model::memoized_answers(k).map(|m| pattern_ratio(m[(1 << k) - 1])))
        .collect::<Result<Vec<_>, _>>()?;
    if memo_max.iter().any(|&r| r != Some(memo_ratio)) {
        return Err(format!("a memoized answer's ratio moves with the asks: {memo_max:?}"));
    }
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the oracle prints for these keys (kept in step with
    /// oracle.py; `./x randomized_response facts-check` compares the two
    /// outputs in full).
    #[test]
    fn the_facts_print_what_the_oracle_prints() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        for (key, value) in [
            ("R-F1.class", "20"),
            ("R-F1.rows", "4"),
            ("R-F1.copied", "4"),
            ("R-F1.copied_share", "1/5"),
            ("R-F2.yes_outcomes.copied", "3"),
            ("R-F2.yes_outcomes.didnt", "1"),
            ("R-F2.p_yes.copied", "3/4"),
            ("R-F2.p_no.didnt", "3/4"),
            ("R-F2.yes_share", "7/20"),
            ("R-F2.route_chances", "1/2,1/4,1/4"),
            ("R-F3.truthful", "10"),
            ("R-F3.truthful_share", "1/2"),
            ("R-F3.truthful.yes", "2"),
            ("R-F3.truthful.no", "8"),
            ("R-F3.forced", "10"),
            ("R-F3.forced.yes", "5"),
            ("R-F3.forced.no", "5"),
            ("R-F3.forced.yes.copied", "1"),
            ("R-F3.forced.no.copied", "1"),
            ("R-F3.yes", "7"),
            ("R-F3.no", "13"),
            ("R-F3.counts_seen", "2"),
            ("R-F4.forced_yes", "5"),
            ("R-F4.forced_yes_share", "1/4"),
            ("R-F4.left", "2"),
            ("R-F4.factor", "2"),
            ("R-F4.estimate", "4"),
            ("R-F4.matches_truth", "true"),
            ("R-F4.twice_yes", "14"),
            ("R-F4.half_class", "10"),
            ("R-F5.ratio.yes", "3"),
            ("R-F5.ratio.no", "3"),
            ("R-F5.epsilon", "1.0986"),
            ("R-F6.yes.copied", "3"),
            ("R-F6.no.copied", "1"),
            ("R-F6.posterior.yes", "3/7"),
            ("R-F6.posterior.no", "1/13"),
            ("R-F6.percent.yes", "43"),
            ("R-F6.percent.no", "8"),
            ("R-F11.class_outcomes", "80"),
            ("R-F11.others", "16"),
            ("R-F11.outnumber", "4"),
            ("R-F11.yes.copied", "12"),
            ("R-F11.yes.didnt", "16"),
            ("R-F11.yes", "28"),
            ("R-F11.posterior.yes", "3/7"),
            ("R-F11.average.yes", "7"),
            ("R-F11.average.yes.copied", "3"),
            ("R-F11.no.copied", "4"),
            ("R-F11.no.didnt", "48"),
            ("R-F11.no", "52"),
            ("R-F11.posterior.no", "1/13"),
            ("R-F11.average.no", "13"),
            ("R-F11.average.no.copied", "1"),
            ("R-F12.odds.prior", "1/4"),
            ("R-F12.odds.yes", "3/4"),
            ("R-F12.odds.no", "1/12"),
            ("R-F12.half_class", "10"),
            ("R-F12.posterior.yes_at_half", "3/4"),
            ("R-X5.posterior.yes_none_copied", "0"),
            ("R-X5.posterior.no_all_copied", "1"),
            ("R-F7.outcomes", "1099511627776"),
            ("R-F7.mean_yes", "7"),
            ("R-F7.var_yes", "15/4"),
            ("R-F7.var_estimate", "15"),
            ("R-F7.sd_estimate", "3.87"),
            ("R-F7.coin_variance_per_person", "3/4"),
            ("R-F7.p_exactly_right", "14028881103/68719476736"),
            ("R-F7.percent.exactly_right", "20"),
            ("R-F7.p_negative", "25556466249/274877906944"),
            ("R-F7.percent.negative", "9"),
            ("R-F8.variance_n1.direct", "4/25"),
            ("R-F8.variance_n1.coins", "91/100"),
            ("R-F8.cost", "91/16"),
            ("R-F8.cost_2dp", "5.69"),
            ("R-F8.added_cost", "75/16"),
            ("R-F8.warner_extra", "3/4"),
            ("R-F13.seed", "3"),
            ("R-F13.runs", "8"),
            ("R-F13.yes", "3,5,12,10,9,7,4,8"),
            ("R-F13.estimates", "-4,0,14,10,8,4,-2,6"),
            ("R-F13.lowest", "-4"),
            ("R-F13.highest", "14"),
            ("R-F13.on_four", "1"),
            ("R-F13.below_zero", "2"),
            ("R-F14.estimate_mean", "4"),
            ("R-F14.outcomes_trillions", "1"),
            ("R-F14.one_in.exactly", "5"),
            ("R-F14.one_in.negative", "11"),
            ("R-F14.clamp_at", "0"),
            ("R-F14.clamped_mean", "584948368349/137438953472"),
            ("R-F14.clamped_mean_2dp", "4.26"),
            ("R-F14.answer_variance", "3/16"),
            ("R-F14.sd_rounded", "4"),
            ("R-F14.cost_rounded", "6"),
            ("R-F9.outcomes", "64"),
            ("R-F9.by_yeses.didnt", "27,27,9,1"),
            ("R-F9.by_yeses.copied", "1,9,27,27"),
            ("R-F9.ratios", "1/27,1/3,3,27"),
            ("R-F9.posterior.all_yes", "27/31"),
            ("R-F9.percent.all_yes", "87"),
            ("R-F9.majority_right", "27/32"),
            ("R-F9.percent.majority_right", "84"),
            ("R-F9.worst_ratio_by_asks", "3,9,27,81,243"),
            ("R-F10.memoized.ratio", "3"),
            ("R-X1.direct.worst_ratio", "unbounded"),
            ("R-X1.pure_coin.worst_ratio", "1"),
            ("R-X1.pure_coin.estimate", "none"),
            ("R-X2.estimate.no_yes", "-10"),
            ("R-X2.estimate.all_yes", "30"),
            ("R-X3.rappor.no_after_75_of_100", "1.39e-24"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// Planted negative: a class that cannot land on average has no facts,
    /// rather than facts rounded to fit.
    #[test]
    fn a_class_off_the_average_is_refused_not_rounded() {
        let class = Class::new(20, &[0, 1, 2]).unwrap();
        let err = class.average_coins().unwrap_err();
        assert!(err.contains("multiple of 4"), "{err}");
    }

    /// Positive checks only: undoing the video's average class gives 5, 2
    /// and 4 (and `undo` returns them, so its comparison with the dial
    /// passed), and `whole` refuses a fraction with its message. No tally
    /// whose estimate disagrees with the dial is planted: `undo` and the dial
    /// share one formula, so none can be built from a real class.
    #[test]
    fn undo_gives_five_two_four_and_whole_refuses_a_fraction() {
        let a = average().unwrap();
        assert_eq!(undo(&a).unwrap(), [Q::int(5), Q::int(2), Q::int(4)]);
        assert_eq!(whole(Q::new(5, 2), "x").unwrap_err(), "x is 5/2, not a whole number");
    }
}
