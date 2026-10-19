//! The mathematics of the coin rule, with no drawing: the rule itself, what
//! it does to one person's answer, to a class of twenty, to the count the
//! survey can recover, and to one person asked again and again. Every
//! probability is an exact fraction ([`Q`]), counted from the coins' equally
//! likely outcomes wherever the video counts them.
//!
//! The rule (01_3, 01_4), defined before any formula: each person flips a
//! fair coin where nobody can see it. Heads, they answer the question
//! truthfully. Tails, they flip a second fair coin and say what it says:
//! heads for yes, tails for no. This is the two-coin randomized response of
//! Dwork and Roth, *The Algorithmic Foundations of Differential Privacy*,
//! §2.3 and §3.2, with the first coin's faces swapped (their tails tells the
//! truth; for a fair coin nothing else changes). It is not Warner's 1965
//! spinner, though it gives exactly the answer chances of Warner's design
//! with its spinner set to three quarters ([`Channel::warner_extra_variance`]).

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use explainer_kit::choose;

// ---------------------------------------------------------------- fractions

/// An exact fraction in lowest terms, its denominator positive. It prints the
/// way Python's `fractions.Fraction` does ("3/4", "7", "-1/2"), so the
/// oracle's values compare as strings. Arithmetic is checked: a result that
/// would overflow `i128` panics rather than wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    num: i128,
    den: i128,
}

const fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

impl Q {
    pub const ZERO: Q = Q { num: 0, den: 1 };
    pub const ONE: Q = Q { num: 1, den: 1 };

    /// `num / den`, reduced.
    ///
    /// # Panics
    /// If `den` is zero: every division the model makes is by a probability
    /// it has checked, so this is a programming error.
    pub const fn new(num: i128, den: i128) -> Q {
        assert!(den != 0, "a fraction with denominator zero");
        let g = gcd(num, den);
        let (num, den) = (num / g, den / g);
        if den < 0 {
            Q { num: -num, den: -den }
        } else {
            Q { num, den }
        }
    }

    /// The whole number `n`.
    pub const fn int(n: i128) -> Q {
        Q { num: n, den: 1 }
    }

    /// The numerator in lowest terms (its sign is the fraction's).
    pub const fn num(self) -> i128 {
        self.num
    }

    /// The denominator in lowest terms, always positive.
    pub const fn den(self) -> i128 {
        self.den
    }

    /// The whole number this fraction is, if it is one.
    pub fn integer(self) -> Option<i128> {
        (self.den == 1).then_some(self.num)
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// `self / other`, or `None` when `other` is zero.
    pub fn checked_div(self, other: Q) -> Option<Q> {
        (other.num != 0).then(|| self / other)
    }

    /// `self` to the power `k`.
    pub fn pow(self, k: u32) -> Q {
        let up = |x: i128| x.checked_pow(k).expect("a power overflows i128");
        Q::new(up(self.num), up(self.den))
    }

    /// The percentage, rounded to the nearest whole number with halves going
    /// up, in exact arithmetic: floor(100 x + 1/2).
    pub fn percent(self) -> i128 {
        add_i(mul_i(200, self.num), self.den).div_euclid(mul_i(2, self.den))
    }
}

fn mul_i(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect("a fraction overflows i128")
}

fn add_i(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect("a fraction overflows i128")
}

fn sub_i(a: i128, b: i128) -> i128 {
    a.checked_sub(b).expect("a fraction overflows i128")
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(
            add_i(mul_i(self.num, o.den), mul_i(o.num, self.den)),
            mul_i(self.den, o.den),
        )
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(
            sub_i(mul_i(self.num, o.den), mul_i(o.num, self.den)),
            mul_i(self.den, o.den),
        )
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(mul_i(self.num, o.num), mul_i(self.den, o.den))
    }
}

impl Div for Q {
    type Output = Q;
    /// # Panics
    /// On division by zero (use [`Q::checked_div`] where zero can occur).
    fn div(self, o: Q) -> Q {
        Q::new(mul_i(self.num, o.den), mul_i(self.den, o.num))
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        // denominators are positive, so cross-multiplying keeps the order
        mul_i(self.num, o.den).cmp(&mul_i(o.num, self.den))
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// Values joined by commas, as the facts print lists.
pub fn csv<T: fmt::Display>(values: &[T]) -> String {
    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

// ------------------------------------------------------------------ the rule

/// One fair coin's face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coin {
    Heads,
    Tails,
}

/// The two coins' four outcomes, equally likely, in the order the video
/// counts them: heads heads, heads tails, tails heads, tails tails. After a
/// first heads the second coin is never looked at; counting both its faces
/// keeps the four outcomes equally likely without changing any answer.
pub const COIN_PAIRS: [(Coin, Coin); 4] = [
    (Coin::Heads, Coin::Heads),
    (Coin::Heads, Coin::Tails),
    (Coin::Tails, Coin::Heads),
    (Coin::Tails, Coin::Tails),
];

/// What decided a person's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Route {
    /// The first coin landed heads: the person told the truth.
    Truth,
    /// Tails, then heads: the second coin said yes.
    ForcedYes,
    /// Tails, then tails: the second coin said no.
    ForcedNo,
}

impl Route {
    /// Every route, in the order the facts list them.
    pub const ALL: [Route; 3] = [Route::Truth, Route::ForcedYes, Route::ForcedNo];

    fn index(self) -> usize {
        match self {
            Route::Truth => 0,
            Route::ForcedYes => 1,
            Route::ForcedNo => 2,
        }
    }
}

/// The rule: where the two coins send a person's answer.
pub fn route(first: Coin, second: Coin) -> Route {
    match (first, second) {
        (Coin::Heads, _) => Route::Truth,
        (Coin::Tails, Coin::Heads) => Route::ForcedYes,
        (Coin::Tails, Coin::Tails) => Route::ForcedNo,
    }
}

/// The answer someone gives (`true` is yes) when their true answer is
/// `copied` and their coins land `first`, `second`.
pub fn answer(copied: bool, first: Coin, second: Coin) -> bool {
    match route(first, second) {
        Route::Truth => copied,
        Route::ForcedYes => true,
        Route::ForcedNo => false,
    }
}

/// How many of the four coin outcomes make someone whose true answer is
/// `copied` say `said`.
pub fn outcomes_saying(copied: bool, said: bool) -> usize {
    COIN_PAIRS
        .iter()
        .filter(|&&(a, b)| answer(copied, a, b) == said)
        .count()
}

/// The chance of saying `said` when the true answer is `copied`, counted:
/// outcomes out of four.
pub fn p_said(copied: bool, said: bool) -> Q {
    Q::new(outcomes_saying(copied, said) as i128, COIN_PAIRS.len() as i128)
}

/// How many times likelier the answer `said` is from someone whose truth is
/// that answer than from someone whose truth is the other: for a yes, those
/// who copied over those who didn't; for a no, the reverse.
pub fn likelihood_ratio(said: bool) -> Q {
    p_said(said, said) / p_said(!said, said)
}

// ------------------------------------------------------------------ the dial

/// The same rule with a first coin that says "tell the truth" with chance
/// `truth` (the video's coin: one half) and a fair second coin. Its edges
/// are what "never more than three times" rests on: truth 1 is the direct
/// question (a yes is impossible from someone who didn't copy, so no ratio
/// bounds it); truth 0 is a pure coin (every answer as likely from everyone,
/// and nothing left to invert).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    truth: Q,
}

impl Channel {
    /// A rule that tells the truth with chance `truth`, from 0 to 1.
    pub fn new(truth: Q) -> Result<Channel, String> {
        if !(Q::ZERO..=Q::ONE).contains(&truth) {
            return Err(format!(
                "a chance of telling the truth of {truth} is not between 0 and 1"
            ));
        }
        Ok(Channel { truth })
    }

    /// The video's two fair coins.
    pub const fn two_coins() -> Channel {
        Channel { truth: Q::new(1, 2) }
    }

    pub const fn truth(self) -> Q {
        self.truth
    }

    /// The chance of saying `said` when the true answer is `copied`.
    pub fn p_said(self, copied: bool, said: bool) -> Q {
        let forced = (Q::ONE - self.truth) / Q::int(2);
        if copied == said { self.truth + forced } else { forced }
    }

    /// The likelihood ratio of the answer `said` (as [`likelihood_ratio`]),
    /// or `None` when that answer is impossible for the other truth.
    pub fn ratio(self, said: bool) -> Option<Q> {
        self.p_said(said, said).checked_div(self.p_said(!said, said))
    }

    /// The largest ratio any answer carries between the two truths, or
    /// `None` when some answer is impossible under one truth: then no ratio
    /// bounds what it reveals.
    pub fn worst_ratio(self) -> Option<Q> {
        let (yes, no) = (self.ratio(true)?, self.ratio(false)?);
        // each ratio is at least 1 (truth >= 0), so its inverse never exceeds it
        Some(yes.max(no))
    }

    /// The estimate of how many of a class of `n` have the secret, from
    /// `yes` answers: the yes count less the second coin's yeses on average,
    /// divided by the share that told the truth. `None` when nobody tells
    /// the truth.
    pub fn estimate(self, yes: i128, n: i128) -> Option<Q> {
        let forced_yes = Q::int(n) * (Q::ONE - self.truth) / Q::int(2);
        (Q::int(yes) - forced_yes).checked_div(self.truth)
    }

    /// The coins' own variance in one person's share of the estimate: the
    /// variance of their answer, divided by the square of the share that
    /// tells the truth. It is the same for both truths (the tests check).
    /// `None` when nobody tells the truth.
    pub fn coin_variance(self) -> Option<Q> {
        let p = self.p_said(true, true);
        (p * (Q::ONE - p)).checked_div(self.truth * self.truth)
    }

    /// Warner's 1965 extra variance term, P(1 − P)/(2P − 1)², for the spinner
    /// chance P that gives this rule's answer chances (P = the chance of a
    /// yes from someone who copied); the formula as the secondary sources in
    /// README.md give it. `None` at P = 1/2.
    pub fn warner_extra_variance(self) -> Option<Q> {
        let p = self.p_said(true, true);
        let two_p_less_one = Q::int(2) * p - Q::ONE;
        (p * (Q::ONE - p)).checked_div(two_p_less_one * two_p_less_one)
    }
}

// ----------------------------------------------------------------- the class

/// Chapter 01's class, invented for the video: twenty seats in four rows of
/// five, and the four seats of the people who copied (row by row).
pub const CLASS_SIZE: usize = 20;
pub const CLASS_COLUMNS: usize = 5;
pub const COPIERS: [usize; 4] = [2, 6, 13, 17];

/// A class: each seat's true answer (`true`: copied).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Class {
    copied: Vec<bool>,
}

impl Class {
    /// `size` seats, those in `copiers` copied.
    pub fn new(size: usize, copiers: &[usize]) -> Result<Class, String> {
        let mut copied = vec![false; size];
        for &seat in copiers {
            let Some(c) = copied.get_mut(seat) else {
                return Err(format!("seat {seat} is not in a class of {size}"));
            };
            if *c {
                return Err(format!("seat {seat} is listed twice"));
            }
            *c = true;
        }
        Ok(Class { copied })
    }

    /// The video's class.
    pub fn video() -> Result<Class, String> {
        Class::new(CLASS_SIZE, &COPIERS)
    }

    pub fn size(&self) -> usize {
        self.copied.len()
    }

    /// Whether seat `i` copied.
    pub fn copied(&self, i: usize) -> bool {
        self.copied[i]
    }

    /// How many copied.
    pub fn copiers(&self) -> usize {
        self.copied.iter().filter(|&&c| c).count()
    }

    /// Every seat's coins as they land on average: within those who copied,
    /// and within those who didn't, each of the four coin outcomes equally
    /// often (the k-th person of each group, in seat order, gets outcome k
    /// mod 4). Only a group whose size is a multiple of four can land
    /// exactly on average; any other class is refused.
    pub fn average_coins(&self) -> Result<Vec<(Coin, Coin)>, String> {
        let mut seen = [0usize; 2];
        let mut coins = Vec::with_capacity(self.size());
        for &c in &self.copied {
            let k = &mut seen[usize::from(c)];
            coins.push(COIN_PAIRS[*k % COIN_PAIRS.len()]);
            *k += 1;
        }
        for (group, n) in ["who didn't copy", "who copied"].iter().zip(seen) {
            if !n.is_multiple_of(COIN_PAIRS.len()) {
                return Err(format!(
                    "the {n} people {group} cannot land exactly on average: not a multiple of {}",
                    COIN_PAIRS.len()
                ));
            }
        }
        Ok(coins)
    }
}

/// Where a class's answers came from: per route, how many of those who
/// didn't copy and of those who did took it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    counts: [[usize; 2]; 3],
}

impl Tally {
    /// The class's answers under `coins`, one pair per seat.
    pub fn of(class: &Class, coins: &[(Coin, Coin)]) -> Result<Tally, String> {
        if coins.len() != class.size() {
            return Err(format!("{} coin pairs for {} seats", coins.len(), class.size()));
        }
        let mut t = Tally::default();
        for (i, &(a, b)) in coins.iter().enumerate() {
            t.counts[route(a, b).index()][usize::from(class.copied(i))] += 1;
        }
        Ok(t)
    }

    /// How many with true answer `copied` took `route`.
    pub fn count(&self, route: Route, copied: bool) -> usize {
        self.counts[route.index()][usize::from(copied)]
    }

    /// How many took `route`, whatever their truth.
    pub fn route(&self, route: Route) -> usize {
        self.counts[route.index()].iter().sum()
    }

    /// How many with true answer `copied` said `said`.
    pub fn said(&self, copied: bool, said: bool) -> usize {
        let truthful = if copied == said {
            self.count(Route::Truth, copied)
        } else {
            0
        };
        let forced = if said { Route::ForcedYes } else { Route::ForcedNo };
        truthful + self.count(forced, copied)
    }

    /// How many said yes (`true`) or no.
    pub fn answers(&self, said: bool) -> usize {
        self.said(true, said) + self.said(false, said)
    }
}

/// Undo the coins on a class of `n` with `yes` answers, as chapter 01 does
/// it (01_9, 01_10): the second coin's yeses on average, a quarter of the
/// class; the yeses left, from the half that told the truth; doubled. The
/// last is the estimate of how many copied.
pub fn undo(yes: usize, n: usize) -> [Q; 3] {
    let truth = Channel::two_coins().truth();
    // the second coin answers for the rest, and says yes to half of them
    let forced = Q::int(n as i128) * (Q::ONE - truth) / Q::int(2);
    let left = Q::int(yes as i128) - forced;
    [forced, left, left / truth]
}

// ---------------------------------------------------- what one answer says

/// Every student of `class` with every one of the four coin outcomes, all
/// n × 4 equally likely: how many give each answer, as
/// `counts[said][copied]` (index 1 is yes, and copied). Counted by running
/// the rule on each student and outcome, not from the chances.
pub fn class_outcomes(class: &Class) -> [[u64; 2]; 2] {
    let mut counts = [[0u64; 2]; 2];
    for i in 0..class.size() {
        let copied = class.copied(i);
        for &(first, second) in &COIN_PAIRS {
            counts[usize::from(answer(copied, first, second))][usize::from(copied)] += 1;
        }
    }
    counts
}

/// The chance that someone who said `said` copied, read off the counted
/// outcomes: the copiers' share of the outcomes giving that answer. `None`
/// when no outcome gives it.
pub fn share_who_copied(counts: &[[u64; 2]; 2], said: bool) -> Option<Q> {
    let [didnt, copied] = counts[usize::from(said)];
    Q::int(i128::from(copied)).checked_div(Q::int(i128::from(didnt + copied)))
}

/// The odds of a chance `q`, q : (1 − q) as one number; `None` at q = 1.
pub fn odds(q: Q) -> Option<Q> {
    q.checked_div(Q::ONE - q)
}

// ------------------------------------------------------- the estimate's scatter

/// The largest class [`yes_count_distribution`] takes: its chances share the
/// denominator 4ⁿ, and summing them, or their squares times counts as
/// [`variance`] does, multiplies two such denominators by up to n², which
/// must stay inside `i128` (the tests compute both at this limit).
pub const MAX_EXACT_CLASS: usize = 24;

/// The chance of each yes count, 0 to n, in a class of `n` of whom `copiers`
/// copied, every coin fresh and fair: of the 4ⁿ equally likely coin
/// outcomes, how many give that count, over 4ⁿ. A copier says yes in three
/// outcomes of four, anyone else in one.
pub fn yes_count_distribution(n: usize, copiers: usize) -> Result<Vec<Q>, String> {
    if copiers > n {
        return Err(format!("{copiers} copiers in a class of {n}"));
    }
    if n > MAX_EXACT_CLASS {
        return Err(format!("a class of {n} is past the exact limit of {MAX_EXACT_CLASS}"));
    }
    let others = n - copiers;
    let pow3 = |k: usize| 3i128.pow(k as u32);
    // outcomes giving i yeses among the copiers, j among the others
    let from_copiers = |i: usize| choose(copiers as u64, i as u64) as i128 * pow3(i);
    let from_others = |j: usize| choose(others as u64, j as u64) as i128 * pow3(others - j);
    let all = 4i128.pow(n as u32);
    let mut out = Vec::with_capacity(n + 1);
    for y in 0..=n {
        let lo = y.saturating_sub(others);
        let hi = y.min(copiers);
        let ways: i128 = (lo..=hi).map(|i| from_copiers(i) * from_others(y - i)).sum();
        out.push(Q::new(ways, all));
    }
    Ok(out)
}

/// The mean of a distribution over 0, 1, 2, ...
pub fn mean(dist: &[Q]) -> Q {
    dist.iter()
        .enumerate()
        .fold(Q::ZERO, |m, (y, &p)| m + Q::int(y as i128) * p)
}

/// The variance of a distribution over 0, 1, 2, ...
pub fn variance(dist: &[Q]) -> Q {
    let square = dist
        .iter()
        .enumerate()
        .fold(Q::ZERO, |m, (y, &p)| m + Q::int((y * y) as i128) * p);
    let m = mean(dist);
    square - m * m
}

/// The chance that the estimate from a class of `n` (2 · yes − n/2) is
/// below zero: fewer yeses than a quarter of the class.
pub fn p_negative_estimate(dist: &[Q]) -> Q {
    let n = dist.len().saturating_sub(1);
    dist.iter()
        .enumerate()
        .filter(|&(y, _)| 4 * y < n)
        .fold(Q::ZERO, |s, (_, &p)| s + p)
}

/// Drawing `n` people independently, with replacement (or from a population
/// so large that drawing without replacement makes no difference), where a
/// share `p` has the secret: the variance of the estimated share from their
/// answers under `channel` (`None` when nobody tells the truth) … Without
/// replacement from N people the sampling part shrinks by (N − n)/(N − 1);
/// at N = n it vanishes and only the coins' part is left, the fixed class.
pub fn sampling_variance(channel: Channel, p: Q, n: i128) -> Option<Q> {
    Some((channel.coin_variance()? + p * (Q::ONE - p)) / Q::int(n))
}

/// … and from the same independent draws' truthful answers.
pub fn direct_variance(p: Q, n: i128) -> Q {
    p * (Q::ONE - p) / Q::int(n)
}

// ------------------------------------------------------------- asking again

/// The most times [`repeated_answers`] asks one person (4¹⁰ outcomes).
pub const MAX_ASKS: u32 = 10;

/// How many times chapter 04's person is asked: 4³ = 64 coin outcomes, few
/// enough to draw every one.
pub const ASKED_AGAIN: u32 = 3;

/// One person asked `asks` times, fresh coins every time: of all 4^asks
/// equally likely coin outcomes, how many give each sequence of answers
/// (bit j of the index: answer j was yes), as `[didn't copy, copied]`.
/// Counted by running the rule on every outcome.
pub fn repeated_answers(asks: u32) -> Result<Vec<[u64; 2]>, String> {
    if asks == 0 || asks > MAX_ASKS {
        return Err(format!("asked {asks} times: from 1 to {MAX_ASKS}"));
    }
    let mut counts = vec![[0u64; 2]; 1 << asks];
    for outcome in 0..(1u64 << (2 * asks)) {
        for copied in [false, true] {
            let mut sequence = 0usize;
            for j in 0..asks {
                let (a, b) = COIN_PAIRS[((outcome >> (2 * j)) & 3) as usize];
                if answer(copied, a, b) {
                    sequence |= 1 << j;
                }
            }
            counts[sequence][usize::from(copied)] += 1;
        }
    }
    Ok(counts)
}

/// The same outcomes, by how many answers were yes (the order of the
/// answers never changes the odds; the tests check): `[didn't, copied]` for
/// 0 to `asks` yeses.
pub fn by_yeses(counts: &[[u64; 2]]) -> Vec<[u64; 2]> {
    let asks = counts.len().trailing_zeros() as usize;
    let mut out = vec![[0u64; 2]; asks + 1];
    for (sequence, c) in counts.iter().enumerate() {
        let y = sequence.count_ones() as usize;
        out[y][0] += c[0];
        out[y][1] += c[1];
    }
    out
}

/// The likelihood ratio, copied over didn't, of an answer pattern counted
/// `[didn't, copied]`; `None` if it never happens to someone who didn't.
pub fn pattern_ratio(c: [u64; 2]) -> Option<Q> {
    Q::int(i128::from(c[1])).checked_div(Q::int(i128::from(c[0])))
}

/// The chance someone who copied is told apart by a majority of `asks`
/// answers (asks odd): more yeses than noes. By the rule's symmetry the same
/// holds for someone who didn't, with noes.
pub fn majority_right(asks: u32) -> Result<Q, String> {
    if asks.is_multiple_of(2) {
        return Err(format!("{asks} answers can tie"));
    }
    let counts = by_yeses(&repeated_answers(asks)?);
    let right: u64 = counts[(asks as usize).div_ceil(2)..].iter().map(|c| c[1]).sum();
    Ok(Q::new(i128::from(right), 4i128.pow(asks)))
}

/// The chance that someone who said yes `yes` times out of `asks` copied,
/// from the prior share `prior` (0 to 1, both ends included): the closed
/// form, one pattern being 3^yes ways in 4^asks for someone who copied and
/// 3^(asks − yes) for anyone else (the tests check it against the counted
/// outcomes). Its ratio is 3^(2·yes − asks).
///
/// # Panics
/// - If `yes` exceeds `asks`.
/// - If `prior` is outside 0 to 1. It is then no chance at all, and it can
///   make the denominator zero: 3/2 with no yes in one ask gives
///   3/2 · 1 + (1 − 3/2) · 3 = 0.
/// - If the exact arithmetic leaves `i128`. 3^yes and 3^(asks − yes) are
///   computed whole and then cross-multiplied by the prior's denominator, so
///   3^max(yes, asks − yes) times the square of that denominator must stay
///   under about 1.7 × 10³⁸ (3^80 alone fits, 3^81 does not); beyond it the
///   checked fractions panic rather than wrap. RAPPOR's 75 of 100 from an
///   even prior needs 3^75 · 4 ≈ 2.4 × 10³⁶, every chapter far less.
pub fn posterior(prior: Q, yes: u32, asks: u32) -> Q {
    assert!(
        (Q::ZERO..=Q::ONE).contains(&prior),
        "a prior chance outside 0 to 1: {prior}"
    );
    let noes = asks.checked_sub(yes).expect("more yeses than answers");
    let with = prior * Q::int(3).pow(yes);
    with / (with + (Q::ONE - prior) * Q::int(3).pow(noes))
}

/// One person asked `asks` times who flips once and repeats that answer
/// (the memoized answer of RAPPOR's permanent randomized response): as
/// [`repeated_answers`], over the four coin outcomes of the one flip.
pub fn memoized_answers(asks: u32) -> Result<Vec<[u64; 2]>, String> {
    if asks == 0 || asks > MAX_ASKS {
        return Err(format!("asked {asks} times: from 1 to {MAX_ASKS}"));
    }
    let mut counts = vec![[0u64; 2]; 1 << asks];
    for &(a, b) in &COIN_PAIRS {
        for copied in [false, true] {
            let said = answer(copied, a, b);
            let sequence = (0..asks).fold(0usize, |s, j| if said { s | (1 << j) } else { s });
            counts[sequence][usize::from(copied)] += 1;
        }
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractions_reduce_order_and_print_like_python() {
        assert_eq!(Q::new(6, 8), Q::new(3, 4));
        assert_eq!(Q::new(3, -4).to_string(), "-3/4");
        assert_eq!(Q::new(-14, 2).to_string(), "-7");
        assert_eq!(Q::new(0, 5), Q::ZERO);
        assert_eq!(Q::new(1, 4) + Q::new(1, 2), Q::new(3, 4));
        assert_eq!(Q::new(1, 4) - Q::new(1, 2), Q::new(-1, 4));
        assert_eq!(Q::new(3, 4) / Q::new(1, 4), Q::int(3));
        assert!(Q::new(1, 3) < Q::new(1, 2) && Q::new(-1, 2) < Q::ZERO);
        assert_eq!(Q::new(2, 3).pow(3), Q::new(8, 27));
        assert_eq!(Q::ONE.checked_div(Q::ZERO), None);
        assert_eq!(Q::new(7, 2).integer(), None);
        assert_eq!(Q::new(8, 2).integer(), Some(4));
        // percentages round half up, exactly
        assert_eq!(Q::new(3, 7).percent(), 43);
        assert_eq!(Q::new(1, 8).percent(), 13);
        assert_eq!(Q::new(1, 13).percent(), 8);
        assert_eq!(Q::new(1, 5).percent(), 20);
    }

    /// The rule, outcome by outcome: heads tells the truth whatever the
    /// second coin; tails lets the second coin answer whatever the truth.
    #[test]
    fn the_rule_routes_every_coin_outcome() {
        use Coin::{Heads, Tails};
        for copied in [false, true] {
            assert_eq!(answer(copied, Heads, Heads), copied);
            assert_eq!(answer(copied, Heads, Tails), copied);
            assert!(answer(copied, Tails, Heads));
            assert!(!answer(copied, Tails, Tails));
        }
        assert_eq!(outcomes_saying(true, true), 3);
        assert_eq!(outcomes_saying(false, true), 1);
        assert_eq!(outcomes_saying(true, false), 1);
        assert_eq!(outcomes_saying(false, false), 3);
    }

    /// The counted chances are the dial's formula at one half.
    #[test]
    fn counting_the_coins_agrees_with_the_dial_at_one_half() {
        let c = Channel::two_coins();
        for copied in [false, true] {
            for said in [false, true] {
                assert_eq!(p_said(copied, said), c.p_said(copied, said));
            }
            assert_eq!(p_said(copied, true) + p_said(copied, false), Q::ONE);
        }
        assert_eq!(p_said(true, true), Q::new(3, 4));
        assert_eq!(p_said(false, true), Q::new(1, 4));
    }

    /// "Three times as often", for both answers, and "never more": the
    /// edges are the direct question (no bound at all) and a pure coin (no
    /// information, and no estimate).
    #[test]
    fn every_answer_is_at_most_three_times_likelier_and_the_dial_has_two_edges() {
        assert_eq!(likelihood_ratio(true), Q::int(3));
        assert_eq!(likelihood_ratio(false), Q::int(3));
        assert_eq!(Channel::two_coins().worst_ratio(), Some(Q::int(3)));
        let direct = Channel::new(Q::ONE).unwrap();
        assert_eq!(
            direct.ratio(true),
            None,
            "a yes from someone who didn't copy is impossible"
        );
        assert_eq!(direct.worst_ratio(), None);
        let coin_only = Channel::new(Q::ZERO).unwrap();
        assert_eq!(coin_only.worst_ratio(), Some(Q::ONE));
        assert_eq!(coin_only.estimate(7, 20), None);
        assert_eq!(coin_only.coin_variance(), None);
        // the ratio (1 + t)/(1 − t) climbs with the chance of the truth
        for (t, r) in [((1, 4), (5, 3)), ((1, 2), (3, 1)), ((3, 4), (7, 1))] {
            let c = Channel::new(Q::new(t.0, t.1)).unwrap();
            assert_eq!(c.worst_ratio(), Some(Q::new(r.0, r.1)));
        }
        assert!(Channel::new(Q::new(5, 4)).is_err());
        assert!(Channel::new(Q::new(-1, 4)).is_err());
    }

    #[test]
    fn a_class_refuses_seats_outside_it_and_repeated_seats() {
        assert!(Class::new(20, &[20]).is_err());
        assert!(Class::new(20, &[3, 3]).is_err());
        let class = Class::video().unwrap();
        assert_eq!((class.size(), class.copiers()), (20, 4));
    }

    /// The class on average: half tells the truth (two copiers say yes,
    /// eight others no), the second coin answers the other ten, five each
    /// way; seven yes and thirteen no. A class whose groups are not
    /// multiples of four cannot land exactly on average and is refused.
    #[test]
    fn the_class_on_average_says_seven_yes_and_thirteen_no() {
        let class = Class::video().unwrap();
        let t = Tally::of(&class, &class.average_coins().unwrap()).unwrap();
        assert_eq!(t.route(Route::Truth), 10);
        assert_eq!((t.count(Route::Truth, true), t.count(Route::Truth, false)), (2, 8));
        assert_eq!((t.route(Route::ForcedYes), t.route(Route::ForcedNo)), (5, 5));
        assert_eq!(
            (t.count(Route::ForcedYes, true), t.count(Route::ForcedNo, true)),
            (1, 1)
        );
        assert_eq!((t.answers(true), t.answers(false)), (7, 13));
        assert_eq!((t.said(true, true), t.said(true, false)), (3, 1));
        assert!(Class::new(20, &[0, 1, 2]).unwrap().average_coins().is_err());
        assert!(Class::new(18, &[0, 1, 2, 3]).unwrap().average_coins().is_err());
        assert!(Tally::of(&class, &[]).is_err());
    }

    /// Chapter 01's arithmetic: five forced yeses, two left, doubled to four,
    /// the same as the dial's estimate 2 · yes − n/2.
    #[test]
    fn undoing_the_coins_gives_the_estimate() {
        assert_eq!(undo(7, 20), [Q::int(5), Q::int(2), Q::int(4)]);
        for yes in 0..=20 {
            assert_eq!(Some(undo(yes, 20)[2]), Channel::two_coins().estimate(yes as i128, 20));
        }
        // the edges: nobody says yes, or everybody does; outside 0 to 20
        assert_eq!(undo(0, 20)[2], Q::int(-10));
        assert_eq!(undo(20, 20)[2], Q::int(30));
    }

    /// For every number of copiers in a class of twenty, the estimate is
    /// right on average, and the coins alone give it variance 3n/4 whoever
    /// copied.
    #[test]
    fn the_estimate_is_unbiased_with_variance_three_quarters_per_person() {
        for copiers in 0..=20 {
            let dist = yes_count_distribution(20, copiers).unwrap();
            assert_eq!(dist.iter().fold(Q::ZERO, |s, &p| s + p), Q::ONE);
            let estimate_mean = Q::int(2) * mean(&dist) - Q::int(10);
            assert_eq!(estimate_mean, Q::int(copiers as i128), "{copiers} copiers");
            assert_eq!(Q::int(4) * variance(&dist), Q::int(15), "{copiers} copiers");
        }
        assert!(yes_count_distribution(4, 5).is_err());
        assert!(yes_count_distribution(MAX_EXACT_CLASS + 1, 0).is_err());
        // at the exact limit the moments still compute: 3n/4 = 18 for 24
        let n = MAX_EXACT_CLASS as i128;
        let dist = yes_count_distribution(MAX_EXACT_CLASS, 7).unwrap();
        assert_eq!(Q::int(2) * mean(&dist) - Q::new(n, 2), Q::int(7));
        assert_eq!(Q::int(4) * variance(&dist), Q::new(3 * n, 4));
    }

    /// The exact distribution against brute force on a class small enough to
    /// list every coin outcome (4⁵).
    #[test]
    fn the_distribution_counts_every_coin_outcome() {
        let class = Class::new(5, &[1, 3]).unwrap();
        let mut ways = [0i128; 6];
        for outcome in 0..(1u32 << 10) {
            let yes = (0..5)
                .filter(|&i| {
                    let (a, b) = COIN_PAIRS[((outcome >> (2 * i)) & 3) as usize];
                    answer(class.copied(i), a, b)
                })
                .count();
            ways[yes] += 1;
        }
        let want: Vec<Q> = ways.iter().map(|&w| Q::new(w, 1024)).collect();
        assert_eq!(yes_count_distribution(5, 2).unwrap(), want);
    }

    #[test]
    fn the_coins_add_the_same_variance_whatever_the_truth() {
        for t in [(1, 4), (1, 2), (3, 4), (1, 1)] {
            let c = Channel::new(Q::new(t.0, t.1)).unwrap();
            let both: Vec<Q> = [true, false]
                .iter()
                .map(|&copied| {
                    let p = c.p_said(copied, true);
                    p * (Q::ONE - p) / (c.truth() * c.truth())
                })
                .collect();
            assert_eq!(both[0], both[1]);
            assert_eq!(c.coin_variance(), Some(both[0]));
        }
        assert_eq!(Channel::two_coins().coin_variance(), Some(Q::new(3, 4)));
    }

    /// For independent draws, sampling variance with the coins is the direct
    /// question's plus 3/(4n), Warner's extra term at his spinner's three
    /// quarters; at a share of one fifth the total is 91/16 times the direct
    /// variance.
    #[test]
    fn the_coins_add_three_quarters_over_n_to_the_sampling_variance() {
        let c = Channel::two_coins();
        let p = Q::new(1, 5);
        for n in [1, 20, 1000] {
            let extra = c.coin_variance().unwrap() / Q::int(n);
            assert_eq!(sampling_variance(c, p, n), Some(direct_variance(p, n) + extra));
        }
        assert_eq!(
            sampling_variance(c, p, 1).unwrap() / direct_variance(p, 1),
            Q::new(91, 16)
        );
        for t in [(1, 4), (1, 2), (3, 4)] {
            let c = Channel::new(Q::new(t.0, t.1)).unwrap();
            assert_eq!(c.warner_extra_variance(), c.coin_variance());
        }
        assert_eq!(Channel::new(Q::ZERO).unwrap().warner_extra_variance(), None);
    }

    /// Fresh coins every time: every answer pattern's ratio is
    /// 3^(2·yes − asks), whatever its order, and the counted outcomes add up
    /// to 4^asks for each truth.
    #[test]
    fn every_repeated_pattern_multiplies_the_odds_by_three_per_yes() {
        for asks in 1..=6u32 {
            let counts = repeated_answers(asks).unwrap();
            let total: [u64; 2] = counts.iter().fold([0, 0], |t, c| [t[0] + c[0], t[1] + c[1]]);
            assert_eq!(total, [4u64.pow(asks), 4u64.pow(asks)]);
            for (sequence, &c) in counts.iter().enumerate() {
                let yes = sequence.count_ones();
                let want = Q::int(3).pow(2 * yes) / Q::int(3).pow(asks);
                assert_eq!(pattern_ratio(c), Some(want), "{asks} asks, pattern {sequence:b}");
                assert_eq!(posterior(Q::new(1, 5), yes, asks), {
                    let odds = Q::new(1, 4) * want;
                    odds / (Q::ONE + odds)
                });
            }
        }
        assert!(repeated_answers(0).is_err());
        assert!(repeated_answers(MAX_ASKS + 1).is_err());
    }

    /// Chapter 02's count: the class's 80 equally likely outcomes; 12 of the
    /// 28 yes outcomes are a copier's and 4 of the 52 no outcomes. The
    /// shares are the closed-form posterior from the same prior for every
    /// class of twenty, from nobody copying (a yes then means 0) to everybody
    /// (1); at half the class a yes means 3/4, the chance a copier says yes.
    #[test]
    fn counting_every_students_outcomes_gives_the_posteriors() {
        let c = class_outcomes(&Class::video().unwrap());
        assert_eq!(c, [[48, 4], [16, 12]]);
        assert_eq!(share_who_copied(&c, true), Some(Q::new(3, 7)));
        assert_eq!(share_who_copied(&c, false), Some(Q::new(1, 13)));
        for copiers in 0..=CLASS_SIZE {
            let seats: Vec<usize> = (0..copiers).collect();
            let c = class_outcomes(&Class::new(CLASS_SIZE, &seats).unwrap());
            let prior = Q::new(copiers as i128, CLASS_SIZE as i128);
            assert_eq!(
                share_who_copied(&c, true),
                Some(posterior(prior, 1, 1)),
                "{copiers} copied"
            );
            assert_eq!(
                share_who_copied(&c, false),
                Some(posterior(prior, 0, 1)),
                "{copiers} copied"
            );
        }
        assert_eq!(posterior(Q::ZERO, 1, 1), Q::ZERO);
        assert_eq!(posterior(Q::ONE, 0, 1), Q::ONE);
        assert_eq!(posterior(Q::new(1, 2), 1, 1), p_said(true, true));
        // no outcome gives any answer in an empty class
        assert_eq!(
            share_who_copied(&class_outcomes(&Class::new(0, &[]).unwrap()), true),
            None
        );
        assert_eq!(odds(Q::new(1, 5)), Some(Q::new(1, 4)));
        assert_eq!(odds(Q::new(3, 7)), Some(Q::new(3, 4)));
        assert_eq!(odds(Q::ONE), None);
    }

    /// Planted negative: a prior that is not a chance is refused before it
    /// can zero the denominator (3/2, no yes in one ask).
    #[test]
    #[should_panic(expected = "a prior chance outside 0 to 1")]
    fn a_prior_outside_zero_to_one_is_refused() {
        posterior(Q::new(3, 2), 0, 1);
    }

    /// Planted negative: more yeses than answers is refused.
    #[test]
    #[should_panic(expected = "more yeses than answers")]
    fn more_yeses_than_asks_is_refused() {
        posterior(Q::new(1, 5), 2, 1);
    }

    /// Three asks: of 64 outcomes, 27 make a copier say yes three times, one
    /// does it to anyone else; a majority vote is right 27 times in 32.
    #[test]
    fn three_asks_count_sixty_four_outcomes() {
        let counts = by_yeses(&repeated_answers(3).unwrap());
        assert_eq!(counts, vec![[27, 1], [27, 9], [9, 27], [1, 27]]);
        assert_eq!(pattern_ratio(counts[3]), Some(Q::int(27)));
        assert_eq!(majority_right(3), Ok(Q::new(27, 32)));
        assert!(majority_right(2).is_err());
        assert_eq!(posterior(Q::new(1, 5), 3, 3), Q::new(27, 31));
        assert_eq!(posterior(Q::new(1, 5), 1, 1), Q::new(3, 7));
        assert_eq!(posterior(Q::new(1, 5), 0, 1), Q::new(1, 13));
    }

    /// Repeating one answer (memoized) spends nothing more: only all-yes and
    /// all-no ever happen, and all-yes stays three times likelier however
    /// often it is asked.
    #[test]
    fn a_memoized_answer_stays_at_three() {
        for asks in 1..=MAX_ASKS {
            let counts = memoized_answers(asks).unwrap();
            let all_yes = (1usize << asks) - 1;
            for (sequence, &c) in counts.iter().enumerate() {
                if sequence != 0 && sequence != all_yes {
                    assert_eq!(c, [0, 0], "{asks} asks, pattern {sequence:b}");
                }
            }
            assert_eq!(pattern_ratio(counts[all_yes]), Some(Q::int(3)));
            assert_eq!(pattern_ratio(counts[0]), Some(Q::new(1, 3)));
        }
        assert!(memoized_answers(0).is_err());
        assert!(memoized_answers(MAX_ASKS + 1).is_err());
    }

    /// RAPPOR's example, from its introduction: 75 yeses in 100 answers
    /// leave "No" a 1.39e-24 chance (an even prior), 1/(3⁵⁰ + 1).
    #[test]
    fn rappors_seventy_five_of_a_hundred() {
        let no = Q::ONE - posterior(Q::new(1, 2), 75, 100);
        assert_eq!(no, Q::new(1, 3i128.pow(50) + 1));
        assert_eq!(format!("{:.2e}", no.to_f64()), "1.39e-24");
    }
}
