//! Exact apportionment arithmetic for Adding a Seat: the film's towns and
//! every rule it compares, in integers only (std, no engine), so the facts
//! and the chapters share one source and the tests can run without fmn.
//!
//! The independent check is `oracle.py`, which solves the divisor rules a
//! different way (bisection on the yardstick, exact rationals) and must
//! print the same facts. Here a divisor rule walks its list of signposts,
//! the yardstick at which each town's next seat opens, highest first.
//!
//! A tie at a cutoff (two signposts or two tails exactly equal where only
//! one can win) is an error, never broken by a hidden convention: the film's
//! worked examples have none, and the range scans count tie houses apart.

use std::cmp::Ordering;

/// The three towns, in row order (top to bottom on screen).
pub const NAMES: [&str; 3] = ["Ash", "Birch", "Cedar"];
pub const KEYS: [&str; 3] = ["ash", "birch", "cedar"];
pub const ASH: usize = 0;
pub const BIRCH: usize = 1;
pub const CEDAR: usize = 2;
/// The populations that never change (chapters 1 to 12).
pub const POP: [u64; 3] = [4_000, 9_000, 26_000];
/// The later census of chapter 13: Ash shrinks, Birch and Cedar grow.
pub const POP2: [u64; 3] = [3_900, 9_500, 26_100];
/// The later census one town at a time (chapter 13 moves the signpost cards
/// in three steps): first only Birch's growth, then Cedar's too, then Ash
/// shrinks to `POP2`.
pub const POP_BIRCH_GROWN: [u64; 3] = [POP[ASH], POP2[BIRCH], POP[CEDAR]];
pub const POP_GROWN: [u64; 3] = [POP[ASH], POP2[BIRCH], POP2[CEDAR]];
/// Chapter 15's try-it-yourself: Cedar grows, Ash and Birch stay the same.
pub const POP_PUZZLE: [u64; 3] = [4_000, 9_000, 30_000];
/// The four-town map where Webster leaves quota (chapter 11), 14 seats.
pub const POP4: [u64; 4] = [2_000, 5_000, 8_000, 27_000];
pub const KEYS4: [&str; 4] = ["a", "b", "c", "d"];
pub const HOUSE4: u64 = 14;
/// Gölz, Peters and Procaccia (2024), Table 1: five states, ten seats.
pub const GPP_HOUSE: u64 = 10;
pub const GPP_A: [u64; 5] = [824, 44, 44, 44, 44];
pub const GPP_B: [u64; 5] = [824, 44, 44, 44, 222];
pub const GPP_C: [u64; 5] = [824, 1, 1, 44, 44];
/// The yardstick chapter 7 slides to.
pub const EXAMPLE_YARDSTICK: u64 = 2_300;
/// The stretch of house sizes chapter 12 counts whole tiles in.
pub const STRETCH: (u64, u64) = (6, 13);

// ------------------------------------------------------------------ ratios

/// An exact non-negative ratio `num / den`; `den == 0` is +infinity (a
/// town's first Adams or Hill seat, which opens at any yardstick).
#[derive(Clone, Copy, Debug)]
pub struct Ratio {
    pub num: u128,
    pub den: u128,
}

impl Ratio {
    pub fn new(num: u128, den: u128) -> Self {
        Self { num, den }
    }
    pub fn floor(self) -> u128 {
        self.num / self.den
    }
    pub fn ceil(self) -> u128 {
        self.num.div_ceil(self.den)
    }
    /// The part after the whole part, as a ratio with the same denominator.
    pub fn tail(self) -> Ratio {
        Ratio::new(self.num % self.den, self.den)
    }
    pub fn is_whole(self) -> bool {
        self.num.is_multiple_of(self.den)
    }
    /// For drawing only: every comparison uses the exact form.
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl PartialEq for Ratio {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Ratio {}
impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        // a/b against c/d by cross-multiplication; two infinities are equal
        (self.num * other.den).cmp(&(other.num * self.den))
    }
}

pub fn total(pops: &[u64]) -> u64 {
    pops.iter().sum()
}

/// Town `i`'s share (quota) of a house of `house` seats.
pub fn share(pops: &[u64], i: usize, house: u64) -> Ratio {
    Ratio::new(u128::from(house) * u128::from(pops[i]), u128::from(total(pops)))
}

/// Every town's seats within its share rounded down and rounded up.
pub fn within_quota(seats: &[u64], pops: &[u64], house: u64) -> bool {
    (0..pops.len()).all(|i| {
        let q = share(pops, i, house);
        q.floor() <= u128::from(seats[i]) && u128::from(seats[i]) <= q.ceil()
    })
}

// ---------------------------------------------------------------- Hamilton

/// Whole parts first, then one seat each to the largest tails.
pub fn hamilton(pops: &[u64], house: u64) -> Result<Vec<u64>, String> {
    let n = pops.len();
    let mut seats: Vec<u64> = (0..n).map(|i| share(pops, i, house).floor() as u64).collect();
    let left = (house - seats.iter().sum::<u64>()) as usize;
    let mut order: Vec<usize> = (0..n).collect();
    // tails share one denominator (the total), so compare numerators
    let tail = |i: usize| share(pops, i, house).tail().num;
    order.sort_by(|&a, &b| tail(b).cmp(&tail(a)).then(a.cmp(&b)));
    if left > 0 && left < n && tail(order[left - 1]) == tail(order[left]) {
        return Err(format!("Hamilton: tied tails at the cutoff for {house} seats"));
    }
    for &i in order.iter().take(left) {
        seats[i] += 1;
    }
    Ok(seats)
}

/// Rounding to the nearest whole seat, each town on its own (halves up).
pub fn naive(pops: &[u64], house: u64) -> Vec<u64> {
    (0..pops.len())
        .map(|i| {
            let q = share(pops, i, house);
            ((2 * q.num + q.den) / (2 * q.den)) as u64
        })
        .collect()
}

// ------------------------------------------------------------ divisor rules

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// round every quotient down
    Jefferson,
    /// round every quotient up
    Adams,
    /// round to the nearest, halves up
    Webster,
    /// round up past the geometric mean of the neighbouring whole numbers
    Hill,
}

pub const RULES: [Rule; 4] = [Rule::Jefferson, Rule::Adams, Rule::Webster, Rule::Hill];

impl Rule {
    pub fn key(self) -> &'static str {
        match self {
            Rule::Jefferson => "jefferson",
            Rule::Adams => "adams",
            Rule::Webster => "webster",
            Rule::Hill => "hill",
        }
    }
}

/// The signpost of a town's `k`th seat (k ≥ 1): the yardstick at which it
/// opens. Jefferson p/k, Adams p/(k−1), Webster p/(k−½) = 2p/(2k−1). Hill's
/// is p/√(k(k−1)); it is returned SQUARED, p²/(k(k−1)), which orders Hill
/// signposts exactly (only compare Hill signposts with each other).
pub fn signpost(rule: Rule, p: u64, k: u64) -> Ratio {
    let (p, k) = (u128::from(p), u128::from(k));
    match rule {
        Rule::Jefferson => Ratio::new(p, k),
        Rule::Adams => Ratio::new(p, k - 1),
        Rule::Webster => Ratio::new(2 * p, 2 * k - 1),
        Rule::Hill => Ratio::new(p * p, k * (k - 1)),
    }
}

/// A divisor rule's seats for a house: award seats one at a time to the
/// highest next signpost. A tie on the last award has no single answer.
pub fn divisor(rule: Rule, pops: &[u64], house: u64) -> Result<Vec<u64>, String> {
    let n = pops.len();
    let mut seats = vec![0_u64; n];
    for step in 1..=house {
        let next = |i: usize, seats: &[u64]| signpost(rule, pops[i], seats[i] + 1);
        let best = (0..n)
            .max_by(|&a, &b| next(a, &seats).cmp(&next(b, &seats)).then(b.cmp(&a)))
            .ok_or("no towns")?;
        let tied = (0..n).filter(|&i| next(i, &seats) == next(best, &seats)).count() > 1;
        if tied && step == house {
            return Err(format!(
                "{}: tied signposts at the cutoff for {house} seats",
                rule.key()
            ));
        }
        seats[best] += 1;
    }
    Ok(seats)
}

/// The first `count` signposts of a rule, highest first: (town, seat
/// number, signpost). Equal signposts keep row order (the oracle's order).
pub fn signposts(rule: Rule, pops: &[u64], count: usize) -> Vec<(usize, u64, Ratio)> {
    let mut all: Vec<(usize, u64, Ratio)> = Vec::new();
    for (i, &p) in pops.iter().enumerate() {
        for k in 1..=(count as u64 + 1) {
            all.push((i, k, signpost(rule, p, k)));
        }
    }
    all.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    all.truncate(count);
    all
}

/// The yardsticks that give exactly `house` seats under Jefferson or
/// Webster: above the (house+1)th signpost, up to the house-th.
pub fn window(rule: Rule, pops: &[u64], house: u64) -> Result<(Ratio, Ratio), String> {
    if rule == Rule::Hill || rule == Rule::Adams {
        return Err("window: Jefferson and Webster only".into());
    }
    let list = signposts(rule, pops, house as usize + 1);
    let (high, low) = (list[house as usize - 1].2, list[house as usize].2);
    if high == low {
        return Err(format!("window: tie at {house} seats"));
    }
    Ok((low, high))
}

// ------------------------------------------------- the quota method (1974)

/// Balinski and Young's quota method, exactly as defined in their 1974
/// paper (PNAS 71(11), p. 4604, the recursion under Theorem 2): seats one
/// at a time; the next seat may go only to a town in E(h+1) = {i : a_i <
/// p_i (h+1) / P}, one that stays within its share rounded up at the new
/// house size, and among those to the largest p_i / (a_i + 1), the top of
/// Jefferson's list. Their definition lets a tie go either way; here a tie
/// between eligible towns is an error, so the film shows one walk. Returns
/// every house from 0 to `house`.
pub fn quota_rule(pops: &[u64], house: u64) -> Result<Vec<Vec<u64>>, String> {
    let n = pops.len();
    let mut seats = vec![0_u64; n];
    let mut history = vec![seats.clone()];
    for h in 1..=house {
        // eligible: seats + 1 <= ceil(h p / P), i.e. seats < h p / P
        let eligible: Vec<usize> = (0..n)
            .filter(|&i| u128::from(seats[i]) * u128::from(total(pops)) < u128::from(h) * u128::from(pops[i]))
            .collect();
        let pri = |i: usize| signpost(Rule::Jefferson, pops[i], seats[i] + 1);
        let best = *eligible
            .iter()
            .max_by(|&&a, &&b| pri(a).cmp(&pri(b)).then(b.cmp(&a)))
            .ok_or("quota rule: nobody eligible")?;
        if eligible.iter().filter(|&&i| pri(i) == pri(best)).count() > 1 {
            return Err(format!("quota rule: tie at {h} seats"));
        }
        seats[best] += 1;
        history.push(seats.clone());
    }
    Ok(history)
}

// ------------------------------------------------------------------ tiles

/// The length of each of town `i`'s seat tiles along the house sizes: the
/// house sizes it takes its share to grow by one whole seat, P / p_i.
pub fn tile_len(pops: &[u64], i: usize) -> Ratio {
    Ratio::new(u128::from(total(pops)), u128::from(pops[i]))
}

/// Where town `i`'s `k`th tile ends (k ≥ 1): k P / p_i, the house size at
/// which its share reaches k. It starts where tile k − 1 ends.
pub fn tile_end(pops: &[u64], i: usize, k: u64) -> Ratio {
    Ratio::new(u128::from(k) * u128::from(total(pops)), u128::from(pops[i]))
}

/// Town `i`'s whole tiles inside the stretch of house sizes from `from` to
/// `to`: tiles that start at or after `from` and end at or before `to`.
pub fn tiles_inside(pops: &[u64], i: usize, from: u64, to: u64) -> u64 {
    (1..)
        .take_while(|&k| tile_end(pops, i, k - 1) < Ratio::new(u128::from(to), 1))
        .filter(|&k| {
            tile_end(pops, i, k - 1) >= Ratio::new(u128::from(from), 1)
                && tile_end(pops, i, k) <= Ratio::new(u128::from(to), 1)
        })
        .count() as u64
}

/// One seat tile of a town's row: seat `k`, from house size `start` to `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatTile {
    pub town: usize,
    pub k: u64,
    pub start: Ratio,
    pub end: Ratio,
}

fn seat_tile(pops: &[u64], town: usize, k: u64) -> SeatTile {
    SeatTile {
        town,
        k,
        start: tile_end(pops, town, k - 1),
        end: tile_end(pops, town, k),
    }
}

/// What chapter 12 draws its supposition on, over the stretch from house size
/// `from` to `to`. No real run can leave a passed tile out, so the picture
/// keeps two things apart: the real run up to `from`, and a supposition that
/// starts there.
#[derive(Debug)]
pub struct StretchProof {
    /// The seats the rule really holds at `from` and at `to`.
    pub then_seats: Vec<u64>,
    pub now_seats: Vec<u64>,
    /// The tile the rule really hands out at `from`.
    pub later: SeatTile,
    /// The tile supposed left out: a town's next tile at `from`, not yet
    /// entered there, ending sooner than `later` and passed by `to`.
    pub missing: SeatTile,
    /// The tiles the rule really hands out after `from`, through `to`, in order.
    pub real_after: Vec<SeatTile>,
    /// Every whole tile inside the stretch, town by town.
    pub inside: Vec<SeatTile>,
}

pub fn stretch_proof(pops: &[u64], from: u64, to: u64) -> Result<StretchProof, String> {
    let history = quota_rule(pops, to)?;
    let (from_r, to_r) = (Ratio::new(u128::from(from), 1), Ratio::new(u128::from(to), 1));
    let handed_out = |h: usize| -> Result<SeatTile, String> {
        let town = (0..pops.len())
            .find(|&i| history[h][i] == history[h - 1][i] + 1)
            .ok_or_else(|| format!("no seat handed out at {h}"))?;
        Ok(seat_tile(pops, town, history[h][town]))
    };
    let later = handed_out(from as usize)?;
    let then_seats = history[from as usize].clone();
    let waiting: Vec<SeatTile> = (0..pops.len())
        .map(|i| seat_tile(pops, i, then_seats[i] + 1))
        .filter(|t| t.start >= from_r && t.end < later.end && t.end <= to_r)
        .collect();
    let [missing] = waiting[..] else {
        return Err(format!("{} tiles could be the one left out at {from}", waiting.len()));
    };
    let real_after = (from as usize + 1..=to as usize)
        .map(handed_out)
        .collect::<Result<Vec<_>, _>>()?;
    let mut inside = Vec::new();
    for i in 0..pops.len() {
        inside.extend(
            (1..)
                .map(|k| seat_tile(pops, i, k))
                .take_while(|t| t.start < to_r)
                .filter(|t| t.start >= from_r && t.end <= to_r),
        );
    }
    Ok(StretchProof {
        then_seats,
        now_seats: history[to as usize].clone(),
        later,
        missing,
        real_after,
        inside,
    })
}

// --------------------------------------------------------- Hill thresholds

/// The largest whole number whose square is at most `n`.
pub fn isqrt(n: u128) -> u128 {
    let mut r = (n as f64).sqrt() as u128;
    while r * r > n {
        r -= 1;
    }
    while (r + 1) * (r + 1) <= n {
        r += 1;
    }
    r
}

/// Hill's threshold between `n` and `n + 1`, √(n(n+1)), rounded half up to
/// two decimals, exactly: m = ⌊100 √x⌋, and it rounds up when 100 √x ≥ m + ½,
/// that is when 4 · 10⁴ x ≥ (2m + 1)².
pub fn hill_threshold(n: u64) -> String {
    let x = u128::from(n) * u128::from(n + 1);
    let m = isqrt(x * 10_000);
    let r = if 4 * 10_000 * x >= (2 * m + 1) * (2 * m + 1) {
        m + 1
    } else {
        m
    };
    decimal(Ratio::new(r, 100), 2)
}

// -------------------------------------------------------------- formatting

/// An exact ratio as a whole number and a proper fraction in lowest terms:
/// "9 3/4", "4 1/3", "1 1/2", or just the whole number.
pub fn mixed(r: Ratio) -> String {
    fn gcd(a: u128, b: u128) -> u128 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    let (whole, rest) = (r.num / r.den, r.num % r.den);
    if rest == 0 {
        return whole.to_string();
    }
    let g = gcd(rest, r.den);
    if whole == 0 {
        format!("{}/{}", rest / g, r.den / g)
    } else {
        format!("{whole} {}/{}", rest / g, r.den / g)
    }
}

/// `r` rounded to `places` decimals, halves up, exact.
pub fn decimal(r: Ratio, places: u32) -> String {
    let scale = 10_u128.pow(places);
    let n = (2 * r.num * scale + r.den) / (2 * r.den);
    if places == 0 {
        return n.to_string();
    }
    let s = format!("{n:0width$}", width = places as usize + 1);
    let (whole, frac) = s.split_at(s.len() - places as usize);
    format!("{whole}.{frac}")
}

/// A whole number with thousands commas.
pub fn grouped(n: u128) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A whole number with commas, or else one decimal place (with commas).
pub fn number(r: Ratio) -> String {
    if r.is_whole() {
        return grouped(r.floor());
    }
    let d = decimal(r, 1);
    let (whole, frac) = d.split_once('.').unwrap_or((&d, "0"));
    format!("{}.{frac}", grouped(whole.parse().unwrap_or(0)))
}

/// A signpost (or any yardstick) rounded to a whole person, with commas.
pub fn people(r: Ratio) -> String {
    grouped((2 * r.num + r.den) / (2 * r.den))
}

// -------------------------------------------------------------- scans

/// Every addition of one seat (h → h+1, h from 1 to 99) under Hamilton
/// that leaves some town with fewer seats: (h, losers).
pub fn alabama_events(pops: &[u64]) -> Result<Vec<(u64, Vec<usize>)>, String> {
    let mut out = Vec::new();
    for h in 1..100 {
        let (a, b) = (hamilton(pops, h)?, hamilton(pops, h + 1)?);
        let losers: Vec<usize> = (0..pops.len()).filter(|&i| b[i] < a[i]).collect();
        if !losers.is_empty() {
            out.push((h, losers));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamilton_the_paradox_and_its_recovery() {
        assert_eq!(hamilton(&POP, 14).unwrap(), vec![2, 3, 9]);
        assert_eq!(hamilton(&POP, 15).unwrap(), vec![2, 3, 10]);
        assert_eq!(hamilton(&POP, 16).unwrap(), vec![1, 4, 11]);
        assert_eq!(hamilton(&POP, 17).unwrap(), vec![2, 4, 11]);
    }

    #[test]
    fn naive_rounding_misses_the_total() {
        assert_eq!(naive(&POP, 14).iter().sum::<u64>(), 13);
        assert_eq!(naive(&POP, 15).iter().sum::<u64>(), 15);
        assert_eq!(naive(&POP, 16).iter().sum::<u64>(), 17);
    }

    #[test]
    fn tails_add_up_to_the_leftover_seats() {
        for h in 1..=100 {
            let tails: u128 = (0..3).map(|i| share(&POP, i, h).tail().num).sum();
            let wholes: u128 = (0..3).map(|i| share(&POP, i, h).floor()).sum();
            assert_eq!(tails, (u128::from(h) - wholes) * u128::from(total(&POP)));
        }
    }

    #[test]
    fn divisor_rules_at_the_film_houses() {
        assert_eq!(divisor(Rule::Jefferson, &POP, 15).unwrap(), vec![1, 3, 11]);
        assert_eq!(divisor(Rule::Jefferson, &POP, 16).unwrap(), vec![1, 4, 11]);
        assert_eq!(divisor(Rule::Jefferson, &POP, 17).unwrap(), vec![1, 4, 12]);
        assert_eq!(divisor(Rule::Adams, &POP, 15).unwrap(), vec![2, 4, 9]);
        for (h, want) in [(15, vec![2, 3, 10]), (16, vec![2, 4, 10]), (17, vec![2, 4, 11])] {
            assert_eq!(divisor(Rule::Webster, &POP, h).unwrap(), want);
            assert_eq!(divisor(Rule::Hill, &POP, h).unwrap(), want);
        }
        assert_eq!(divisor(Rule::Hill, &POP, 3).unwrap(), vec![1, 1, 1]);
        assert!(
            divisor(Rule::Jefferson, &POP, 18).is_err(),
            "Ash 2 and Cedar 13 tie at 2,000"
        );
    }

    #[test]
    fn signposts_and_windows() {
        let list = signposts(Rule::Jefferson, &POP, 19);
        assert_eq!((list[15].0, list[15].1), (BIRCH, 4));
        assert_eq!(people(list[15].2), "2,250");
        assert_eq!((list[16].0, list[16].1), (CEDAR, 12));
        assert_eq!(list[17].2, list[18].2);
        let (low, high) = window(Rule::Jefferson, &POP, 15).unwrap();
        assert_eq!(
            (number(low), number(high)),
            ("2,250".to_string(), "2,363.6".to_string())
        );
        let (low, high) = window(Rule::Webster, &POP4, HOUSE4).unwrap();
        assert_eq!(
            (number(low), number(high)),
            ("3,176.5".to_string(), "3,200".to_string())
        );
        assert_eq!(divisor(Rule::Webster, &POP4, HOUSE4).unwrap(), vec![1, 2, 3, 8]);
    }

    #[test]
    fn the_quota_rule_skips_cedar_at_fifteen() {
        let hist = quota_rule(&POP, 17).unwrap();
        assert_eq!(hist[15], vec![1, 4, 10]);
        assert_eq!(hist[16], vec![1, 4, 11]);
        for h in 1..=17 {
            assert!(within_quota(&hist[h as usize], &POP, h));
        }
        assert!(quota_rule(&POP, 36).is_err(), "the oracle's first tie is at 36 seats");
    }

    #[test]
    fn tiles_follow_jeffersons_list() {
        assert_eq!(mixed(tile_len(&POP, ASH)), "9 3/4");
        assert_eq!(mixed(tile_len(&POP, BIRCH)), "4 1/3");
        assert_eq!(mixed(tile_len(&POP, CEDAR)), "1 1/2");
        assert_eq!(mixed(tile_end(&POP, CEDAR, 11)), "16 1/2");
        assert_eq!(mixed(tile_end(&POP, BIRCH, 4)), "17 1/3");
        // a larger signpost is a tile that ends sooner: the same order
        let list = signposts(Rule::Jefferson, &POP, 17);
        for pair in list.windows(2) {
            assert!(tile_end(&POP, pair[0].0, pair[0].1) <= tile_end(&POP, pair[1].0, pair[1].1));
        }
        // the stretch from 6 to 13 seats holds 0, 1 and 4 whole tiles
        let inside: Vec<u64> = (0..3).map(|i| tiles_inside(&POP, i, 6, 13)).collect();
        assert_eq!(inside, vec![0, 1, 4]);
    }

    #[test]
    fn hill_thresholds_sit_below_the_halfway_marks() {
        let want = ["0.00", "1.41", "2.45", "3.46"];
        for (n, w) in want.iter().enumerate() {
            assert_eq!(hill_threshold(n as u64), *w);
        }
        for n in 0..1000_u128 {
            // √(n(n+1)) < n + ½ because n(n+1) < n² + n + ¼
            assert!(4 * n * (n + 1) < (2 * n + 1) * (2 * n + 1));
        }
        assert_eq!(isqrt(120_000), 346);
    }

    #[test]
    fn the_population_paradox() {
        assert_eq!(hamilton(&POP2, 16).unwrap(), vec![2, 4, 10]);
        assert_eq!(divisor(Rule::Webster, &POP2, 16).unwrap(), vec![2, 4, 10]);
        assert_eq!(divisor(Rule::Jefferson, &POP2, 16).unwrap(), vec![1, 4, 11]);
    }

    #[test]
    fn alabama_scan() {
        let ev = alabama_events(&POP).unwrap();
        let houses: Vec<u64> = ev.iter().map(|e| e.0).collect();
        assert_eq!(houses, vec![15, 23, 54, 62, 93]);
        assert!(ev.iter().all(|e| e.1 == vec![ASH]));
    }

    /// Each town's held tiles, as (town, seat).
    fn held(seats: &[u64]) -> Vec<(usize, u64)> {
        let mut out: Vec<(usize, u64)> = Vec::new();
        for (i, &n) in seats.iter().enumerate() {
            out.extend((1..=n).map(|k| (i, k)));
        }
        out
    }

    /// Could a run hold exactly these tiles? Seats go out one at a time, so a
    /// town's tiles must be its first ones, with no hole.
    fn is_a_run(tiles: &[(usize, u64)]) -> bool {
        tiles.iter().all(|&(i, k)| k == 1 || tiles.contains(&(i, k - 1)))
    }

    #[test]
    fn the_supposition_starts_from_a_real_moment() {
        let p = stretch_proof(&POP, 6, 13).unwrap();
        let (from, to) = (Ratio::new(6, 1), Ratio::new(13, 1));
        assert_eq!(p.then_seats, vec![0, 2, 4]);
        assert_eq!(p.now_seats, vec![1, 3, 9]);
        // at 6 the rule really hands out Birch's second, already entered,
        // and it ends later than the tile supposed missing
        assert_eq!((p.later.town, p.later.k), (BIRCH, 2));
        assert!(p.later.start < from && p.later.end > p.missing.end);
        assert_eq!(mixed(p.later.end), "8 2/3");
        // the missing tile is Cedar's next at 6: not entered there (it starts
        // exactly at 6), ending sooner, and passed by 13
        assert_eq!((p.missing.town, p.missing.k), (CEDAR, p.then_seats[CEDAR] + 1));
        assert_eq!(p.missing.start, from);
        assert!(p.missing.end <= to);
        assert_eq!(mixed(p.missing.end), "7 1/2");
    }

    #[test]
    fn no_state_of_the_picture_is_an_impossible_run() {
        let p = stretch_proof(&POP, 6, 13).unwrap();
        let before = held(&p.now_seats);
        let during = held(&p.then_seats);
        let mut after = during.clone();
        after.extend(p.real_after.iter().map(|t| (t.town, t.k)));
        // what fades at "Suppose one were" comes back at "no passed tile":
        // seven tiles out, the same seven in
        assert_eq!(p.real_after.len(), 7);
        let (mut sorted_before, mut sorted_after) = (before.clone(), after.clone());
        sorted_before.sort_unstable();
        sorted_after.sort_unstable();
        assert_eq!(sorted_before, sorted_after);
        assert!(is_a_run(&before) && is_a_run(&during) && is_a_run(&after));
        // nothing solid during the supposition was handed out after `then`
        assert!(p.real_after.iter().all(|t| !during.contains(&(t.town, t.k))));
        // negative control: the old picture, the run at 13 less Cedar's eighth
        let old: Vec<(usize, u64)> = before.iter().copied().filter(|&t| t != (CEDAR, 8)).collect();
        assert!(!is_a_run(&old));
        // and the missing tile itself is really handed out, right after `then`
        assert_eq!(p.real_after[0], p.missing);
    }

    #[test]
    fn the_counts_on_screen_hold() {
        let p = stretch_proof(&POP, 6, 13).unwrap();
        let seats = 7_u64;
        // the rings: one per whole tile inside, the missing one among them
        assert!(p.inside.contains(&p.missing));
        let mut sum = 0;
        for (i, &pop) in POP.iter().enumerate() {
            let rings = p.inside.iter().filter(|t| t.town == i).count() as u64;
            assert_eq!(rings, tiles_inside(&POP, i, 6, 13));
            // the bound beside each row, n <= seats * p / P, exactly
            assert!(u128::from(rings) * u128::from(total(&POP)) <= u128::from(seats) * u128::from(pop));
            sum += rings;
        }
        assert_eq!(sum, 5);
        // the chips: one per seat handed out since then, plus the missing
        // tile, one more than the stretch holds or has room for
        let chips = p.real_after.len() as u64 + 1;
        assert_eq!(chips, seats + 1);
        assert!(sum < chips && seats < chips);
    }

    #[test]
    fn every_tile_is_ruled_out_or_inside_exactly_once() {
        let p = stretch_proof(&POP, 6, 13).unwrap();
        let (from, to) = (Ratio::new(6, 1), Ratio::new(13, 1));
        for i in 0..POP.len() {
            for k in (1..).take_while(|&k| tile_end(&POP, i, k - 1) < Ratio::new(39, 2)) {
                let t = seat_tile(&POP, i, k);
                let classes = [t.start < from, t.start >= from && t.end > to, p.inside.contains(&t)];
                assert_eq!(classes.iter().filter(|&&c| c).count(), 1, "{i} {k}");
            }
        }
    }

    #[test]
    fn the_real_run_at_now_keeps_both_halves_of_the_promise() {
        let p = stretch_proof(&POP, 6, 13).unwrap();
        let to = Ratio::new(13, 1);
        assert!(within_quota(&p.now_seats, &POP, 13));
        for (i, &n) in p.now_seats.iter().enumerate() {
            // no tile it has not entered is handed out
            assert!(seat_tile(&POP, i, n).start < to);
            // and every tile it has passed is
            assert!(seat_tile(&POP, i, n + 1).end > to);
        }
    }

    #[test]
    fn formatting() {
        assert_eq!(decimal(share(&POP, ASH, 15), 2), "1.54");
        assert_eq!(decimal(share(&POP, CEDAR, 15), 2), "10.00");
        assert_eq!(decimal(Ratio::new(17, 40), 3), "0.425");
        assert_eq!(grouped(39_000), "39,000");
        assert_eq!(grouped(999), "999");
        assert_eq!(people(signpost(Rule::Jefferson, 26_000, 3)), "8,667");
        assert_eq!(decimal(share(&GPP_B, 0, GPP_HOUSE), 2), "6.99");
    }
}
