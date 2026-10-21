//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x randomized_response
//! facts-check`): the rows of docs/fact-sheet.md. R-F rows are computed,
//! R-X rows are the edges of the always/never sentences and the primary
//! sources' own words. In the 13-chapter video: R-F1 to R-F5 and R-F17 serve
//! chapters 01 to 03, R-F18 chapter 04, R-F6, R-F11, R-F12 and R-X5 chapter
//! 05, R-F7, R-F13, R-F14 and R-F19 chapters 06 and 07, R-F8 and R-F20
//! chapter 08, R-F21 and R-X6 chapter 09, R-F9 and R-F15 chapter 10, R-F10,
//! R-F22, R-X3 and R-X7 chapter 11, R-F16 chapter 12, R-F23 chapter 13, and
//! R-X8 chapter 02.

use explainer_kit::Facts;
use explainer_kit::stats::Data;

use crate::model::{
    self, ASKED_AGAIN, COIN_PAIRS, Channel, Class, Coin, MAX_ASKS, Q, Route, Tally, by_yeses, csv, likelihood_ratio,
    pattern_ratio,
};

/// How many asks R-F9.worst_ratio_by_asks lists.
const WORST_LISTED: u32 = 5;

/// RAPPOR's example (its section 1.3): yeses out of answers, from an even
/// prior; chapter 11 speaks it.
pub const RAPPOR_EXAMPLE: (u32, u32) = (75, 100);

/// RAPPOR (Erlingsson, Pihur and Korolova, arXiv 1407.6981v2, CCS 2014),
/// in its own words, as chapters 09 and 11 quote it (read 2026-10-20).
pub const RAPPOR_COMPLIANCE: &str = "an assumption that may not hold for human subjects";
pub const RAPPOR_YEAR: u32 = 2014;
const RAPPOR_MEMOIZED: &str = "Permanent randomized response";
const RAPPOR_EACH_REPORT: &str = "Instantaneous randomized response";
const RAPPOR_DEPLOYMENT: &str = "limited deployment in Google's Chrome Web browser";
const RAPPOR_PRIOR: &str = "irrespective of any attacker's prior knowledge";

/// Chapter 04's worked example, invented: a class of forty that says yes
/// eighteen times.
pub const FORTY: (usize, usize) = (40, 18);

/// Chapter 04's one real class: the fifth of chapter 03's seeded classes
/// (9 yeses, an estimate of 8 against the 4 who copied), a miss that stays
/// inside 0 to n (the tests check).
pub const SHOWN_RUN: usize = 4;

/// Chapter 07's bigger group, with the same coins; and chapter 09's sizes
/// for "asking more people won't fix it".
pub const BIG_CLASS: usize = 300;
pub const ASKED: [usize; 3] = [model::CLASS_SIZE, 100, BIG_CLASS];

/// Chapter 13's check that a fixed class's estimate variance is 3n/4 for
/// every number of copiers: these sizes, every k (the exact limit last).
const CHECKED_SIZES: [usize; 5] = [1, 2, 7, model::CLASS_SIZE, model::MAX_EXACT_CLASS];

/// Chapter 03's real classes: the seed (the chapter's number, fixed before
/// any draw was looked at, the only one tried) and how many classes.
pub const SEED: u64 = 3;
pub const RUNS: usize = 8;

/// Chapter 05's dial: a spinner of `SLICES` equal slices, and the settings
/// the chapter turns it to, each a name and how many slices say "tell the
/// truth" (the rest hand the answer to the fair second coin).
pub const SLICES: i128 = 4;
pub const DIAL: [(&str, i128); 5] = [("pure", 0), ("down", 1), ("ours", 2), ("up", 3), ("direct", 4)];

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

/// The whole number nearest √v for v ≥ 0, in exact arithmetic: the largest
/// m with (m − 1/2)² ≤ v (halves up, as `nearest`).
fn nearest_root(v: Q) -> i128 {
    let mut m = 0;
    while Q::new(2 * m + 1, 2).pow(2) <= v {
        m += 1;
    }
    m
}

/// A fixed class of `n`'s count variance under `channel`: n times one
/// person's coins' variance, `None` when nobody tells the truth.
fn setting_class(channel: Channel, n: usize) -> Option<Q> {
    channel.coin_variance().map(|w| Q::int(n as i128) * w)
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

    // chapter 04: one person asked three times, fresh coins
    let asks = ASKED_AGAIN;
    let counts = by_yeses(&model::repeated_answers(asks)?);
    let total = (COIN_PAIRS.len() as u64).pow(asks);
    f.add("R-F9.asks", asks)?;
    f.add("R-F9.outcomes", total)?;
    let column = |i: usize| counts.iter().map(|c| c[i]).collect::<Vec<u64>>();
    f.add("R-F9.by_yeses.didnt", csv(&column(0)))?;
    f.add("R-F9.by_yeses.copied", csv(&column(1)))?;
    let all_yes = counts[asks as usize];
    f.add("R-F9.all_yes.copied", all_yes[1])?;
    f.add("R-F9.all_yes.didnt", all_yes[0])?;
    let all_yes_ratio = pattern_ratio(all_yes).ok_or("three yeses never happen to someone who didn't copy")?;
    f.add("R-F9.ratio.all_yes", all_yes_ratio)?;
    // three noes: the same factor, the other way
    let all_no = counts[0];
    if all_no != [all_yes[1], all_yes[0]] {
        return Err(format!("three noes count {all_no:?}, three yeses {all_yes:?} reversed"));
    }
    f.add("R-F9.all_no.didnt", all_no[0])?;
    f.add("R-F9.all_no.copied", all_no[1])?;
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
    // "whichever the truth": the majority's outcomes counted for each truth,
    // more yeses for someone who copied, more noes for anyone else
    let ahead = (asks as usize).div_ceil(2);
    let right_copied: u64 = counts[ahead..].iter().map(|c| c[1]).sum();
    let right_didnt: u64 = counts[..ahead].iter().map(|c| c[0]).sum();
    let right_didnt_share = Q::new(i128::from(right_didnt), i128::from(total));
    if Q::new(i128::from(right_copied), i128::from(total)) != majority || right_didnt_share != majority {
        return Err(format!(
            "a majority is right in {right_copied} and {right_didnt} of {total}, not {majority}"
        ));
    }
    f.add("R-F9.majority.outcomes", right_copied)?;
    f.add("R-F9.majority_right.didnt", right_didnt_share)?;
    let worst_by_asks: Vec<Q> = (1..=WORST_LISTED).map(worst_after).collect::<Result<_, _>>()?;
    f.add("R-F9.worst_ratio_by_asks", csv(&worst_by_asks))?;
    let memo = model::memoized_answers(asks)?;
    let memo_all_yes = memo[(1 << asks) - 1];
    let memo_ratio = pattern_ratio(memo_all_yes).ok_or("the memoized all-yes never happens")?;
    f.add("R-F10.memoized.ratio", memo_ratio)?;
    // one flip, its answer sent every time: only all-yes and all-no happen
    let mixed: u64 = memo[1..(1 << asks) - 1].iter().flatten().sum();
    if mixed != 0 {
        return Err(format!("a memoized answer changed between asks in {mixed} outcomes"));
    }
    f.add("R-F10.memoized.outcomes", COIN_PAIRS.len())?;
    f.add("R-F10.memoized.all_yes.copied", memo_all_yes[1])?;
    f.add("R-F10.memoized.all_yes.didnt", memo_all_yes[0])?;
    // chapter 04's class, every student with every outcome: three yeses
    // counted, the copiers' share of them; and a memoized answer's
    let others = others as u64;
    let class_all_yes = [others * all_yes[0], copiers as u64 * all_yes[1]];
    let counted = Q::new(
        i128::from(class_all_yes[1]),
        i128::from(class_all_yes[0] + class_all_yes[1]),
    );
    if counted != after_three {
        return Err(format!(
            "counting the class gives {counted}, the closed form {after_three}"
        ));
    }
    f.add("R-F15.all_yes.copied", class_all_yes[1])?;
    f.add("R-F15.all_yes.didnt", class_all_yes[0])?;
    f.add("R-F15.all_yes", class_all_yes[0] + class_all_yes[1])?;
    let memo_after = Q::new(
        i128::from(copiers as u64 * memo_all_yes[1]),
        i128::from(copiers as u64 * memo_all_yes[1] + others * memo_all_yes[0]),
    );
    if memo_after != model::posterior(share, 1, 1) {
        return Err(format!("a memoized three yeses leave {memo_after}, unlike one yes"));
    }
    f.add("R-F10.memoized.posterior", memo_after)?;
    // the same three yeses from other classes: the chance is the class's
    f.add(
        "R-F15.posterior.all_yes_at_half",
        model::posterior(Q::new(1, 2), asks, asks),
    )?;
    f.add(
        "R-F15.posterior.all_yes_none_copied",
        model::posterior(Q::ZERO, asks, asks),
    )?;

    // chapter 05: the dial. Each setting is `Channel` at k/SLICES; its yes
    // counts are its chances over the 2 · SLICES equally likely outcomes of
    // a slice and the second coin's face
    let outcomes = 2 * SLICES;
    f.add("R-F16.slices", SLICES)?;
    f.add("R-F16.outcomes", outcomes)?;
    for (name, k) in DIAL {
        let dial = Channel::new(Q::new(k, SLICES))?;
        let t = dial.truth();
        f.add(&format!("R-F16.{name}.truth"), t)?;
        f.add(&format!("R-F16.{name}.truth_slices"), k)?;
        for (copied, who) in [(true, "copied"), (false, "didnt")] {
            let yes = whole(dial.p_said(copied, true) * Q::int(outcomes), "a yes count")?;
            f.add(&format!("R-F16.{name}.yes.{who}"), yes)?;
        }
        // both answers carry the same ratio, (1 + t)/(1 − t) below t = 1
        let ratio = dial.ratio(true);
        if ratio != dial.ratio(false) || ratio != (Q::ONE + t).checked_div(Q::ONE - t) {
            return Err(format!(
                "at {t} a yes carries {ratio:?} and a no {:?}",
                dial.ratio(false)
            ));
        }
        f.add(
            &format!("R-F16.{name}.ratio"),
            ratio.map_or("unbounded".into(), |r| r.to_string()),
        )?;
        // the coins' variance in one person's share, (1 − t²)/(4t²) above 0
        let wobble = dial.coin_variance();
        if wobble != (Q::ONE - t * t).checked_div(Q::int(4) * t * t) {
            return Err(format!("at {t} the coins' variance is {wobble:?}"));
        }
        f.add(
            &format!("R-F16.{name}.coin_variance"),
            wobble.map_or("none".into(), |w| w.to_string()),
        )?;
        // what the voice says (as 03_8 did): how far the fixed class's count
        // usually misses, the square root of n times one person's variance
        let class = setting_class(dial, n);
        f.add(
            &format!("R-F16.{name}.class_variance"),
            class.map_or("none".into(), |v| v.to_string()),
        )?;
        f.add(
            &format!("R-F16.{name}.miss_2dp"),
            class.map_or("none".into(), |v| format!("{:.2}", v.to_f64().sqrt())),
        )?;
        f.add(
            &format!("R-F16.{name}.miss_rounded"),
            class.map_or("none".into(), |v| nearest_root(v).to_string()),
        )?;
    }
    // the dial's middle misses as chapter 03 said: "about four people"
    if Some(sd.round() as i128) != setting_class(Channel::two_coins(), n).map(nearest_root) {
        return Err(format!("the two coins' class misses by {sd:.2}, not as the dial says"));
    }
    let setting = |name: &str| -> Result<Channel, String> {
        let (_, k) = DIAL
            .iter()
            .find(|(d, _)| *d == name)
            .ok_or_else(|| format!("no dial setting {name}"))?;
        Channel::new(Q::new(*k, SLICES))
    };
    let wobble = |c: Channel| {
        c.coin_variance()
            .ok_or_else(|| format!("no coins' variance at {}", c.truth()))
    };
    let ours = setting("ours")?;
    if ours != Channel::two_coins() {
        return Err(format!("the dial's middle, {}, is not the two coins", ours.truth()));
    }
    let (up, down) = (setting("up")?, setting("down")?);
    // the per-person variances against ours: exact checks only, never said
    // (they are variances; the voice speaks the class's misses, above)
    let up_vs = wobble(up)? / wobble(ours)?;
    f.add("R-F16.up.wobble_vs_ours", up_vs)?;
    f.add(
        "R-F16.up.wobble_vs_ours_nearest_quarter",
        Q::new(nearest(up_vs * Q::int(4)), 4),
    )?;
    f.add("R-F16.down.wobble_vs_ours", wobble(down)? / wobble(ours)?)?;
    // "under twice as likely": the whole number just above the ratio
    let down_ratio = down.ratio(true).ok_or("no ratio a quarter of the way up")?;
    f.add("R-F16.down.ratio_under", down_ratio.num() / down_ratio.den() + 1)?;
    // one person's variance, a fixed class's count (n times it, chapter
    // 03's 15) and its estimated share (divided by n)
    let class_count = Q::int(n as i128) * wobble(ours)?;
    if class_count != var_estimate {
        return Err(format!("{n} people's coins make {class_count}, not {var_estimate}"));
    }
    f.add("R-F16.class.count_variance", class_count)?;
    f.add("R-F16.class.share_variance", wobble(ours)? / Q::int(n as i128))?;

    // chapter 01's chart: each person's chance of a yes, added up, for the
    // direct question, a coin alone and the rule
    let coin_alone = Channel::new(Q::ZERO)?;
    let alone: Vec<Q> = (0..=n).map(|k| model::average_yes(coin_alone, n, k)).collect();
    if alone.iter().any(|&y| y != alone[0]) {
        return Err(format!("a coin alone's yeses move with the copiers: {}", csv(&alone)));
    }
    f.add("R-F17.coin_alone.average_yes", alone[0])?;
    f.add(
        "R-F17.direct.average_yes",
        model::average_yes(Channel::new(Q::ONE)?, n, copiers),
    )?;
    let rule_line = model::average_yes(Channel::two_coins(), n, copiers);
    if rule_line != Q::int(yes as i128) {
        return Err(format!(
            "the rule's line gives {rule_line} yeses, the class on average {yes}"
        ));
    }
    f.add("R-F17.rule.average_yes", rule_line)?;
    f.add("R-F17.rule.at_none", model::average_yes(Channel::two_coins(), n, 0))?;
    f.add("R-F17.rule.at_all", model::average_yes(Channel::two_coins(), n, n))?;

    // chapter 04: a class of forty that says yes eighteen times, and the
    // recipe right on average for every number of copiers in our class
    let (forty, forty_yes) = FORTY;
    let [forty_forced, forty_left, forty_estimate] = model::undo(forty_yes, forty);
    f.add("R-F18.class", forty)?;
    f.add("R-F18.yes", forty_yes)?;
    f.add("R-F18.forced_yes", whole(forty_forced, "a quarter of forty")?)?;
    f.add("R-F18.left", whole(forty_left, "the yeses left")?)?;
    f.add(
        "R-F18.estimate",
        whole(forty_estimate, "the class of forty's estimate")?,
    )?;
    for k in 0..=n {
        let d = model::yes_count_distribution(n, k)?;
        if model::estimate_mean(&d) != Q::int(k as i128) {
            return Err(format!("with {k} copiers the estimate does not average {k}"));
        }
    }
    f.add("R-F18.unbiased_every_k", true)?;
    let shown = Tally::of(&a.class, &runs()?[SHOWN_RUN])?.answers(true);
    let shown_estimate = model::undo(shown, n)[2];
    if shown_estimate == truth || shown_estimate < Q::ZERO || shown_estimate > Q::int(n as i128) {
        return Err(format!(
            "the shown class's estimate {shown_estimate} is not a miss inside 0 to {n}"
        ));
    }
    f.add("R-F18.shown.yes", shown)?;
    f.add("R-F18.shown.estimate", shown_estimate)?;

    // chapter 07: how far one survey misses. "Miss by about four" is the
    // standard deviation (the root of the average squared miss); a typical
    // size, not a limit
    let miss_by = Q::int(nearest_root(var_estimate));
    let p_miss = model::p_miss_at_least(&dist, truth, miss_by);
    let two_fifths = Q::new(2, 5);
    if !(p_miss > two_fifths && p_miss < Q::new(1, 2)) {
        return Err(format!(
            "{p_miss} of classes miss by {miss_by} or more, not just over two in five"
        ));
    }
    f.add("R-F19.miss_by", miss_by)?;
    f.add("R-F19.miss_at_least", p_miss)?;
    f.add("R-F19.miss_at_least.over", two_fifths)?;
    f.add("R-F19.percent.miss_at_least", p_miss.percent())?;
    let p_miss_twice = model::p_miss_at_least(&dist, truth, Q::int(2) * miss_by);
    f.add("R-F19.miss_at_least_twice", p_miss_twice)?;
    f.add("R-F19.one_in.miss_at_least_twice", one_in(p_miss_twice)?)?;
    // three hundred with the same coins: the count's variance is n times one
    // person's whoever copied (chapter 13 checks it at every exact size)
    let big_var = setting_class(Channel::two_coins(), BIG_CLASS).ok_or("no variance for the big class")?;
    let big_miss = nearest_root(big_var);
    if Q::int(big_miss * big_miss) != big_var {
        return Err(format!("{big_var} is not a whole square"));
    }
    f.add("R-F19.big.class", BIG_CLASS)?;
    f.add("R-F19.big.copied", Q::int(BIG_CLASS as i128) * share)?;
    f.add("R-F19.big.variance", big_var)?;
    f.add("R-F19.big.miss", big_miss)?;
    let big_share = Q::new(big_miss, BIG_CLASS as i128);
    f.add("R-F19.big.miss_share", big_share)?;
    f.add("R-F19.big.percent.miss_share", big_share.percent())?;
    let class_share = sd / n as f64;
    f.add("R-F19.class.miss_share_2dp", format!("{class_share:.2}"))?;
    f.add("R-F19.class.percent.miss_share", format!("{:.0}", 100.0 * class_share))?;
    f.add("R-F19.class.one_in.miss_share", (1.0 / class_share).round())?;

    // chapter 08: a sample of twenty from a big school, in the share's
    // percentage points; and how many coin answers match twenty direct ones
    let direct_n = model::direct_variance(share, n as i128);
    let sampled = |m: i128| model::sampling_variance(Channel::two_coins(), share, m).ok_or("no sampling variance");
    let points = |v: Q| 100.0 * v.to_f64().sqrt();
    f.add("R-F20.direct.points_2dp", format!("{:.2}", points(direct_n)))?;
    f.add("R-F20.direct.points", points(direct_n).round())?;
    f.add("R-F20.coins.points_2dp", format!("{:.2}", points(sampled(n as i128)?)))?;
    f.add("R-F20.coins.points", points(sampled(n as i128)?).round())?;
    let match_exact = coins / direct_n;
    let match_n = model::at_least(match_exact);
    if sampled(match_n)? > direct_n || sampled(match_n - 1)? <= direct_n {
        return Err(format!(
            "{match_n} coin answers are not the fewest to match {n} direct ones"
        ));
    }
    if match_exact / Q::int(n as i128) != cost {
        return Err(format!("{match_exact} people over {n} is not the cost {cost}"));
    }
    f.add("R-F20.match.exact", match_exact)?;
    f.add("R-F20.match", match_n)?;
    f.add("R-F20.match.points_2dp", format!("{:.2}", points(sampled(match_n)?)))?;

    // chapter 09: the four who copied always say no, the others follow the
    // rule (one pattern of noncompliance; others push either way or cancel)
    let broken = model::yes_count_distribution_copiers_say_no(n, copiers)?;
    let broken_yes = model::mean(&broken);
    let broken_estimate = model::estimate_mean(&broken);
    if broken_estimate != Q::ZERO - truth / Q::int(2) {
        return Err(format!(
            "the broken rule averages {broken_estimate}, not minus half of {truth}"
        ));
    }
    f.add("R-F21.average_yes", broken_yes)?;
    f.add("R-F21.twice_yes", factor * broken_yes)?;
    f.add("R-F21.average_estimate", broken_estimate)?;
    f.add("R-F21.bias", broken_estimate - truth)?;
    let share_mean = model::share_mean_copiers_say_no(Channel::two_coins(), share).ok_or("no share estimate")?;
    if share_mean * Q::int(n as i128) != broken_estimate {
        return Err(format!(
            "the share's average {share_mean} is not the class's {broken_estimate} over {n}"
        ));
    }
    f.add("R-F21.share_mean", share_mean)?;
    f.add("R-F21.asked", csv(&ASKED))?;

    // chapter 11: a hundred fresh answers, seventy-five yes, from even odds
    let (many_yes, many_asks) = RAPPOR_EXAMPLE;
    let many_no = many_asks - many_yes;
    let no_many = Q::ONE - model::posterior(Q::new(1, 2), many_yes, many_asks);
    // each yes multiplies the evidence by three, each no divides it by three
    let ratio_many = likelihood_ratio(true).pow(many_yes) / likelihood_ratio(false).pow(many_no);
    if ratio_many != Q::int(3).pow(many_yes - many_no) || Q::ONE / (ratio_many + Q::ONE) != no_many {
        return Err(format!(
            "{many_yes} yes in {many_asks} do not give 3 to the {}",
            many_yes - many_no
        ));
    }
    f.add("R-F22.asks", many_asks)?;
    f.add("R-F22.yes", many_yes)?;
    f.add("R-F22.no", many_no)?;
    f.add("R-F22.excess", many_yes - many_no)?;
    f.add("R-F22.ratio", ratio_many)?;
    f.add("R-F22.ratio_digits", ratio_many.num().to_string().len())?;
    f.add("R-F22.posterior_no", no_many)?;
    f.add("R-F22.zeros", model::zeros_after_point(no_many)?)?;

    // chapter 13: a fixed class's estimate variance is 3n/4 for every n and
    // every number of copiers, so its typical miss can be stated in advance
    for size in CHECKED_SIZES {
        for k in 0..=size {
            let d = model::yes_count_distribution(size, k)?;
            if factor * factor * model::variance(&d) != per_person * Q::int(size as i128) {
                return Err(format!(
                    "a class of {size} with {k} copiers does not have variance 3n/4"
                ));
            }
        }
    }
    f.add("R-F23.per_person", per_person)?;
    f.add("R-F23.checked_sizes", csv(&CHECKED_SIZES))?;

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
    // RAPPOR's own words, as the cards and chapter 11's voice use them
    let read = "Erlingsson, Pihur and Korolova, RAPPOR, arXiv 1407.6981v2 (CCS 2014), read 2026-10-20";
    f.pin(
        "R-X6.rappor.compliance",
        RAPPOR_COMPLIANCE,
        "an assumption that may not hold for human subjects",
        &format!("{read}, section 1"),
    )?;
    f.pin(
        "R-X7.rappor.year",
        RAPPOR_YEAR,
        "2014",
        &format!("{read}, page 1: CCS'14, November 3-7, 2014"),
    )?;
    f.pin(
        "R-X7.rappor.memoized",
        RAPPOR_MEMOIZED,
        "Permanent randomized response",
        &format!("{read}, sections 1.3 and 2"),
    )?;
    f.pin(
        "R-X7.rappor.each_report",
        RAPPOR_EACH_REPORT,
        "Instantaneous randomized response",
        &format!("{read}, sections 1.3 and 2"),
    )?;
    f.pin(
        "R-X7.rappor.deployment",
        RAPPOR_DEPLOYMENT,
        "limited deployment in Google's Chrome Web browser",
        &format!("{read}, section 1.1"),
    )?;
    f.pin(
        "R-X8.rappor.prior",
        RAPPOR_PRIOR,
        "irrespective of any attacker's prior knowledge",
        &format!("{read}, section 1"),
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
            ("R-F9.all_yes.copied", "27"),
            ("R-F9.all_yes.didnt", "1"),
            ("R-F9.ratio.all_yes", "27"),
            ("R-F9.all_no.didnt", "27"),
            ("R-F9.all_no.copied", "1"),
            ("R-F9.majority_right", "27/32"),
            ("R-F9.percent.majority_right", "84"),
            ("R-F9.majority.outcomes", "54"),
            ("R-F9.majority_right.didnt", "27/32"),
            ("R-F9.worst_ratio_by_asks", "3,9,27,81,243"),
            ("R-F10.memoized.ratio", "3"),
            ("R-F10.memoized.outcomes", "4"),
            ("R-F10.memoized.all_yes.copied", "3"),
            ("R-F10.memoized.all_yes.didnt", "1"),
            ("R-F10.memoized.posterior", "3/7"),
            ("R-F15.all_yes.copied", "108"),
            ("R-F15.all_yes.didnt", "16"),
            ("R-F15.all_yes", "124"),
            ("R-F15.posterior.all_yes_at_half", "27/28"),
            ("R-F15.posterior.all_yes_none_copied", "0"),
            ("R-F16.slices", "4"),
            ("R-F16.outcomes", "8"),
            ("R-F16.pure.truth", "0"),
            ("R-F16.pure.truth_slices", "0"),
            ("R-F16.pure.yes.copied", "4"),
            ("R-F16.pure.yes.didnt", "4"),
            ("R-F16.pure.ratio", "1"),
            ("R-F16.pure.coin_variance", "none"),
            ("R-F16.pure.class_variance", "none"),
            ("R-F16.pure.miss_2dp", "none"),
            ("R-F16.pure.miss_rounded", "none"),
            ("R-F16.down.class_variance", "75"),
            ("R-F16.down.miss_2dp", "8.66"),
            ("R-F16.down.miss_rounded", "9"),
            ("R-F16.ours.class_variance", "15"),
            ("R-F16.ours.miss_2dp", "3.87"),
            ("R-F16.ours.miss_rounded", "4"),
            ("R-F16.up.class_variance", "35/9"),
            ("R-F16.up.miss_2dp", "1.97"),
            ("R-F16.up.miss_rounded", "2"),
            ("R-F16.direct.class_variance", "0"),
            ("R-F16.direct.miss_2dp", "0.00"),
            ("R-F16.direct.miss_rounded", "0"),
            ("R-F16.down.truth", "1/4"),
            ("R-F16.down.truth_slices", "1"),
            ("R-F16.down.yes.copied", "5"),
            ("R-F16.down.yes.didnt", "3"),
            ("R-F16.down.ratio", "5/3"),
            ("R-F16.down.coin_variance", "15/4"),
            ("R-F16.ours.truth", "1/2"),
            ("R-F16.ours.truth_slices", "2"),
            ("R-F16.ours.yes.copied", "6"),
            ("R-F16.ours.yes.didnt", "2"),
            ("R-F16.ours.ratio", "3"),
            ("R-F16.ours.coin_variance", "3/4"),
            ("R-F16.up.truth", "3/4"),
            ("R-F16.up.truth_slices", "3"),
            ("R-F16.up.yes.copied", "7"),
            ("R-F16.up.yes.didnt", "1"),
            ("R-F16.up.ratio", "7"),
            ("R-F16.up.coin_variance", "7/36"),
            ("R-F16.direct.truth", "1"),
            ("R-F16.direct.truth_slices", "4"),
            ("R-F16.direct.yes.copied", "8"),
            ("R-F16.direct.yes.didnt", "0"),
            ("R-F16.direct.ratio", "unbounded"),
            ("R-F16.direct.coin_variance", "0"),
            ("R-F16.up.wobble_vs_ours", "7/27"),
            ("R-F16.up.wobble_vs_ours_nearest_quarter", "1/4"),
            ("R-F16.down.wobble_vs_ours", "5"),
            ("R-F16.down.ratio_under", "2"),
            ("R-F16.class.count_variance", "15"),
            ("R-F16.class.share_variance", "3/80"),
            ("R-X1.direct.worst_ratio", "unbounded"),
            ("R-X1.pure_coin.worst_ratio", "1"),
            ("R-X1.pure_coin.estimate", "none"),
            ("R-X2.estimate.no_yes", "-10"),
            ("R-X2.estimate.all_yes", "30"),
            ("R-X3.rappor.no_after_75_of_100", "1.39e-24"),
            ("R-F17.coin_alone.average_yes", "10"),
            ("R-F17.direct.average_yes", "4"),
            ("R-F17.rule.average_yes", "7"),
            ("R-F17.rule.at_none", "5"),
            ("R-F17.rule.at_all", "15"),
            ("R-F18.class", "40"),
            ("R-F18.yes", "18"),
            ("R-F18.forced_yes", "10"),
            ("R-F18.left", "8"),
            ("R-F18.estimate", "16"),
            ("R-F18.unbiased_every_k", "true"),
            ("R-F18.shown.yes", "9"),
            ("R-F18.shown.estimate", "8"),
            ("R-F19.miss_by", "4"),
            ("R-F19.miss_at_least", "239955963659/549755813888"),
            ("R-F19.miss_at_least.over", "2/5"),
            ("R-F19.percent.miss_at_least", "44"),
            ("R-F19.miss_at_least_twice", "73349641357/1099511627776"),
            ("R-F19.one_in.miss_at_least_twice", "15"),
            ("R-F19.big.class", "300"),
            ("R-F19.big.copied", "60"),
            ("R-F19.big.variance", "225"),
            ("R-F19.big.miss", "15"),
            ("R-F19.big.miss_share", "1/20"),
            ("R-F19.big.percent.miss_share", "5"),
            ("R-F19.class.miss_share_2dp", "0.19"),
            ("R-F19.class.percent.miss_share", "19"),
            ("R-F19.class.one_in.miss_share", "5"),
            ("R-F20.direct.points_2dp", "8.94"),
            ("R-F20.direct.points", "9"),
            ("R-F20.coins.points_2dp", "21.33"),
            ("R-F20.coins.points", "21"),
            ("R-F20.match.exact", "455/4"),
            ("R-F20.match", "114"),
            ("R-F20.match.points_2dp", "8.93"),
            ("R-F21.average_yes", "4"),
            ("R-F21.twice_yes", "8"),
            ("R-F21.average_estimate", "-2"),
            ("R-F21.bias", "-6"),
            ("R-F21.share_mean", "-1/10"),
            ("R-F21.asked", "20,100,300"),
            ("R-F22.asks", "100"),
            ("R-F22.yes", "75"),
            ("R-F22.no", "25"),
            ("R-F22.excess", "50"),
            ("R-F22.ratio", "717897987691852588770249"),
            ("R-F22.ratio_digits", "24"),
            ("R-F22.posterior_no", "1/717897987691852588770250"),
            ("R-F22.zeros", "23"),
            ("R-F23.per_person", "3/4"),
            ("R-F23.checked_sizes", "1,2,7,20,24"),
            (
                "R-X6.rappor.compliance",
                "an assumption that may not hold for human subjects",
            ),
            ("R-X7.rappor.year", "2014"),
            ("R-X7.rappor.memoized", "Permanent randomized response"),
            ("R-X7.rappor.each_report", "Instantaneous randomized response"),
            (
                "R-X7.rappor.deployment",
                "limited deployment in Google's Chrome Web browser",
            ),
            ("R-X8.rappor.prior", "irrespective of any attacker's prior knowledge"),
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
