//! The mathematics of Video E as pure functions: exact fractions for every
//! chance a pair of tosses or a few rolls can have, what each trick costs,
//! a coin that remembers its last toss (a two-state Markov chain) under the
//! plain rule, under Samuelson's rule and under Blum's two algorithms, the
//! rates of von Neumann's, Peres's and Elias's procedures against the
//! entropy, and the seeded runs the chapters draw. `facts.rs` prints them;
//! `oracle.py` recomputes each one on its own.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};

use explainer_kit::choose;
use explainer_kit::stats::Data;

// ---------------------------------------------------------------------------
// exact fractions
// ---------------------------------------------------------------------------

/// An exact fraction, reduced, with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Q {
    n: i128,
    d: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect("a fraction overflowed i128")
}

impl Q {
    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub const ONE: Q = Q { n: 1, d: 1 };
    pub const HALF: Q = Q { n: 1, d: 2 };

    /// # Panics
    /// If `d` is zero.
    pub fn new(n: i128, d: i128) -> Self {
        assert!(d != 0, "a fraction needs a non-zero denominator");
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q {
            n: s * n / g,
            d: s * d / g,
        }
    }

    pub const fn int(n: i128) -> Self {
        Q { n, d: 1 }
    }

    pub fn is_zero(self) -> bool {
        self.n == 0
    }

    pub fn f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }

    /// One minus this: the chance of the other face.
    pub fn other(self) -> Q {
        Q::ONE - self
    }

    pub fn pow(self, k: u32) -> Q {
        (0..k).fold(Q::ONE, |acc, _| acc * self)
    }

    /// `n/d`, or `n` when whole.
    pub fn exact(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }

    /// The decimal with `places` digits after the point, rounded half away
    /// from zero, computed in integers (no float ever rounds it).
    pub fn decimal(self, places: u32) -> String {
        let scale = 10i128.pow(places);
        let scaled = times(self.n.abs(), scale);
        let q = (times(2, scaled) + self.d) / times(2, self.d);
        let sign = if self.n < 0 && q != 0 { "-" } else { "" };
        if places == 0 {
            format!("{sign}{q}")
        } else {
            format!("{sign}{}.{:0width$}", q / scale, q % scale, width = places as usize)
        }
    }

    /// As a percentage with `places` decimals: 4/5 is "80".
    pub fn percent(self, places: u32) -> String {
        (self * Q::int(100)).decimal(places)
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        let g = gcd(self.d, o.d);
        Q::new(times(self.n, o.d / g) + times(o.n, self.d / g), times(self.d / g, o.d))
    }
}

impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        self + Q { n: -o.n, d: o.d }
    }
}

impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        let (g1, g2) = (gcd(self.n, o.d).max(1), gcd(o.n, self.d).max(1));
        Q::new(times(self.n / g1, o.n / g2), times(self.d / g2, o.d / g1))
    }
}

impl Div for Q {
    type Output = Q;
    /// # Panics
    /// On division by zero.
    fn div(self, o: Q) -> Q {
        assert!(o.n != 0, "division by a zero fraction");
        self * Q::new(o.d, o.n)
    }
}

impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        times(self.n, o.d).cmp(&times(o.n, self.d))
    }
}

/// Solve `a x = b` exactly by elimination; None when the system is singular.
pub fn solve(mut a: Vec<Vec<Q>>, mut b: Vec<Q>) -> Option<Vec<Q>> {
    let n = b.len();
    for col in 0..n {
        let pivot = (col..n).find(|&r| !a[r][col].is_zero())?;
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in 0..n {
            if row == col || a[row][col].is_zero() {
                continue;
            }
            let f = a[row][col] / a[col][col];
            for k in col..n {
                let delta = f * a[col][k];
                a[row][k] = a[row][k] - delta;
            }
            b[row] = b[row] - f * b[col];
        }
    }
    Some((0..n).map(|i| b[i] / a[i][i]).collect())
}

// ---------------------------------------------------------------------------
// one coin, two tosses
// ---------------------------------------------------------------------------

/// A face of the coin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    H,
    T,
}

use Face::{H, T};

impl Face {
    pub fn letter(self) -> char {
        match self {
            H => 'H',
            T => 'T',
        }
    }
}

/// The four results of two tosses, in the order the square is read.
pub const PAIRS: [(Face, Face); 4] = [(H, H), (H, T), (T, H), (T, T)];

/// The video's coin: heads four times in five.
pub fn bent() -> Q {
    Q::new(4, 5)
}

/// The chances of two tosses in a row: heads on the first, then heads on the
/// second after a head and after a tail. The unit square draws exactly this:
/// the first toss cuts the width, the second cuts each column's height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairLaw {
    pub first: Q,
    pub after_h: Q,
    pub after_t: Q,
}

impl PairLaw {
    /// One coin, tossed twice, each toss on its own.
    pub fn same_coin(p: Q) -> Self {
        Self {
            first: p,
            after_h: p,
            after_t: p,
        }
    }

    /// A first coin, then a different second coin.
    pub fn two_coins(first: Q, second: Q) -> Self {
        Self {
            first,
            after_h: second,
            after_t: second,
        }
    }

    pub fn prob(&self, a: Face, b: Face) -> Q {
        let pa = match a {
            H => self.first,
            T => self.first.other(),
        };
        let second = match a {
            H => self.after_h,
            T => self.after_t,
        };
        pa * match b {
            H => second,
            T => second.other(),
        }
    }

    /// The chance the two tosses differ, so the pair settles the bet.
    pub fn decides(&self) -> Q {
        self.prob(H, T) + self.prob(T, H)
    }

    /// Among the pairs that decide, the share that are heads then tails
    /// (None: the two tosses never differ).
    pub fn heads_first_share(&self) -> Option<Q> {
        let d = self.decides();
        (!d.is_zero()).then(|| self.prob(H, T) / d)
    }
}

/// The expected number of tosses until a pair decides: 1 / (p (1 − p)).
/// None for a coin that only ever shows one face.
pub fn expected_tosses(p: Q) -> Option<Q> {
    let pq = p * p.other();
    (!pq.is_zero()).then(|| Q::ONE / pq)
}

/// The tallies within two standard errors of what `n` tries at chance `p`
/// give on average: where a fair game's count lands about 19 times in 20.
/// Found in whole numbers, with no square root: (k − np)² ≤ 4np(1 − p).
pub fn two_sigma_band(n: u32, p: Q) -> (u32, u32) {
    let mean = Q::int(i128::from(n)) * p;
    let limit = Q::int(4) * mean * p.other();
    let inside = |k: &u32| {
        let d = Q::int(i128::from(*k)) - mean;
        d * d <= limit
    };
    let low = (0..=n).find(inside).unwrap_or(0);
    let high = (0..=n).rev().find(inside).unwrap_or(n);
    (low, high)
}

/// The chance that `pairs` pairs in a row are all thrown away.
pub fn undecided_after(p: Q, pairs: u32) -> Q {
    PairLaw::same_coin(p).decides().other().pow(pairs)
}

/// The rules that read exactly `n` tosses and then name a winner, never
/// asking for more, sorted by how many of the patterns that the first
/// player wins on have 0, 1, …, n heads (her chance of winning depends on
/// nothing else): returns (how many such count lists there are, how many
/// give her exactly one half at every one of `chances`). Her chance is a
/// polynomial of degree `n` in p, so it is one half for every coin just
/// when it is one half at n + 1 different chances ([`spread_chances`]).
pub fn fixed_length_rules(n: u32, chances: &[Q]) -> (u64, u64) {
    let row = pascal_row(n);
    let mut wins = vec![0u128; row.len()];
    let (mut lists, mut fair) = (0, 0);
    loop {
        lists += 1;
        let fair_here = chances.iter().all(|&p| {
            let chance = wins.iter().enumerate().fold(Q::ZERO, |acc, (k, &a)| {
                acc + Q::int(a as i128) * pattern_chance(p, n, k as u32)
            });
            chance == Q::HALF
        });
        fair += u64::from(fair_here);
        // the next count list, like an odometer
        let Some(k) = (0..wins.len()).find(|&k| wins[k] < row[k]) else {
            return (lists, fair);
        };
        wins[..k].fill(0);
        wins[k] += 1;
    }
}

/// `n + 1` different chances strictly between 0 and 1: enough to pin down a
/// polynomial of degree `n`, without leaning on a coin that cannot land
/// both ways.
pub fn spread_chances(n: u32) -> Vec<Q> {
    (1..=i128::from(n) + 1).map(|i| Q::new(i, i128::from(n) + 2)).collect()
}

// ---------------------------------------------------------------------------
// a coin that remembers its last toss
// ---------------------------------------------------------------------------

/// A coin whose chance of heads depends on its last toss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Memory {
    pub after_h: Q,
    pub after_t: Q,
}

impl Memory {
    /// A coin that repeats its last face with chance `s`.
    pub fn sticky(s: Q) -> Self {
        Self {
            after_h: s,
            after_t: s.other(),
        }
    }

    pub fn heads_after(&self, prev: Face) -> Q {
        match prev {
            H => self.after_h,
            T => self.after_t,
        }
    }

    /// The next two tosses, given the toss just before them.
    pub fn pair_after(&self, prev: Face) -> PairLaw {
        PairLaw {
            first: self.heads_after(prev),
            after_h: self.after_h,
            after_t: self.after_t,
        }
    }

    /// The long-run share of heads (None: the coin can lock on either face).
    pub fn long_run_heads(&self) -> Option<Q> {
        let leave = self.after_h.other() + self.after_t;
        (!leave.is_zero()).then(|| self.after_t / leave)
    }

    /// A pair taken after the coin has been tossed for a long time.
    pub fn long_run_pair(&self) -> Option<PairLaw> {
        Some(PairLaw {
            first: self.long_run_heads()?,
            after_h: self.after_h,
            after_t: self.after_t,
        })
    }

    /// The plain rule (pairs, throw away the matching ones) played to the
    /// end: the chance heads-first wins, given the toss before the first
    /// pair. None when no pair can ever decide.
    pub fn plain_rule(&self, prev: Face) -> Option<Q> {
        let (h, t) = (self.pair_after(H), self.pair_after(T));
        let x = solve(
            vec![
                vec![Q::ONE - h.prob(H, H), Q::ZERO - h.prob(T, T)],
                vec![Q::ZERO - t.prob(H, H), Q::ONE - t.prob(T, T)],
            ],
            vec![h.prob(H, T), t.prob(H, T)],
        )?;
        Some(match prev {
            H => x[0],
            T => x[1],
        })
    }

    /// The plain rule started after a long run of tosses.
    pub fn plain_rule_long_run(&self) -> Option<Q> {
        let h = self.long_run_heads()?;
        Some(h * self.plain_rule(H)? + h.other() * self.plain_rule(T)?)
    }

    /// Samuelson's rule: look only at the tosses that come right after an
    /// `after`, pair those up, throw away the matching pairs. The chance the
    /// first deciding pair is heads then tails, given the toss before the
    /// start. None when such a pair can never complete.
    pub fn after_only_rule(&self, after: Face, prev: Face) -> Option<Q> {
        // a state is (the last toss, the first toss of an open pair if any)
        let index = |cur: Face, open: Option<Face>| {
            3 * usize::from(cur == T)
                + match open {
                    None => 0,
                    Some(H) => 1,
                    Some(T) => 2,
                }
        };
        let mut a = vec![vec![Q::ZERO; 6]; 6];
        let mut b = vec![Q::ZERO; 6];
        for cur in [H, T] {
            for open in [None, Some(H), Some(T)] {
                let row = index(cur, open);
                a[row][row] = Q::ONE;
                for x in [H, T] {
                    let px = match x {
                        H => self.heads_after(cur),
                        T => self.heads_after(cur).other(),
                    };
                    if cur != after {
                        a[row][index(x, open)] = a[row][index(x, open)] - px;
                        continue;
                    }
                    match open {
                        None => a[row][index(x, Some(x))] = a[row][index(x, Some(x))] - px,
                        Some(y) if y == x => a[row][index(x, None)] = a[row][index(x, None)] - px,
                        Some(y) => {
                            if y == H {
                                b[row] = b[row] + px;
                            }
                        }
                    }
                }
            }
        }
        Some(solve(a, b)?[index(prev, None)])
    }
}

/// When Blum's two algorithms announce a result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Announce {
    /// Algorithm A: as soon as a state's two exits differ.
    AtOnce,
    /// Algorithm B: when the chain is next in the state whose exits differed.
    OnReturn,
}

/// What Blum's algorithms remember about a state: nothing, the first exit
/// of an open pair, or (algorithm B) a result waiting to be announced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Slot {
    Empty,
    Open(bool),
    Waiting(bool),
}

/// Blum's chain of Theorem A with two states, which is the sticky coin: in
/// either state the coin repeats its last face with chance `stay` (his exit
/// bit 0) or changes face (his exit bit 1, with chance ε = 1 − `stay`). Each
/// state is treated as its own von Neumann coin on its exit bits: exits 1
/// then 0 give an H, exits 0 then 1 a T. Returns the chances that the first
/// two announced results are HH, HT, TH, TT, by following every history
/// until less than 1e-15 of the chance is left.
pub fn blum_first_two(stay: f64, when: Announce) -> [f64; 4] {
    // (current state, slot of state 0, slot of state 1, first result if announced)
    type State = (usize, Slot, Slot, Option<bool>);
    let mut live: HashMap<State, f64> = HashMap::from([((0, Slot::Empty, Slot::Empty, None), 1.0)]);
    let mut done = [0.0f64; 4];
    let mut left = 1.0f64;
    while left > 1e-15 {
        let mut next: HashMap<State, f64> = HashMap::new();
        for ((cur, s0, s1, said), mass) in live.drain() {
            let mut slots = [s0, s1];
            // algorithm B's step 1: a waiting result is announced on return
            let returned = match slots[cur] {
                Slot::Waiting(r) => {
                    slots[cur] = Slot::Empty;
                    Some(r)
                }
                _ => None,
            };
            for (bit, p) in [(false, stay), (true, 1.0 - stay)] {
                let mut slots = slots;
                // exits 1 then 0 are an H (true), exits 0 then 1 a T
                let at_once = match (slots[cur], when) {
                    (Slot::Empty, _) => {
                        slots[cur] = Slot::Open(bit);
                        None
                    }
                    (Slot::Open(b), _) if b == bit => {
                        slots[cur] = Slot::Empty;
                        None
                    }
                    (Slot::Open(b), Announce::AtOnce) => {
                        slots[cur] = Slot::Empty;
                        Some(b)
                    }
                    (Slot::Open(b), Announce::OnReturn) => {
                        slots[cur] = Slot::Waiting(b);
                        None
                    }
                    (Slot::Waiting(_), _) => unreachable!("a waiting result was announced on return"),
                };
                let mut results = said.into_iter().chain(returned).chain(at_once);
                let (first, second) = (results.next(), results.next());
                let to = if bit { 1 - cur } else { cur };
                match (first, second) {
                    (Some(f), Some(s)) => done[2 * usize::from(!f) + usize::from(!s)] += mass * p,
                    _ => *next.entry((to, slots[0], slots[1], first)).or_insert(0.0) += mass * p,
                }
            }
        }
        left = next.values().sum();
        live = next;
    }
    done
}

// ---------------------------------------------------------------------------
// same counts, same chance: patterns, Pascal's rows, dice
// ---------------------------------------------------------------------------

/// The chance of one particular pattern of `n` tosses with `k` heads.
pub fn pattern_chance(p: Q, n: u32, k: u32) -> Q {
    p.pow(k) * p.other().pow(n - k)
}

/// Row `n` of Pascal's triangle: how many patterns of `n` tosses have
/// 0, 1, …, n heads.
pub fn pascal_row(n: u32) -> Vec<u128> {
    (0..=n).map(|k| choose(u64::from(n), u64::from(k))).collect()
}

/// Every pattern of `n` tosses with exactly `k` heads, heads sorting first.
pub fn patterns(n: u32, k: u32) -> Vec<Vec<Face>> {
    (0..1u32 << n)
        .filter(|m| m.count_zeros() - (32 - n) == k)
        .map(|m| (0..n).map(|i| if m >> (n - 1 - i) & 1 == 0 { H } else { T }).collect())
        .collect()
}

/// The chance that `n` tosses show exactly `k` heads.
pub fn count_chance(p: Q, n: u32, k: u32) -> Q {
    Q::int(choose(u64::from(n), u64::from(k)) as i128) * pattern_chance(p, n, k)
}

/// The loaded die of the video, in twentieths: six comes up half the time.
pub const LOADED: [u32; 6] = [1, 1, 2, 2, 4, 10];
/// A fair die on the same scale.
pub const FAIR_DIE: [u32; 6] = [1, 1, 1, 1, 1, 1];

/// The chance of each face of a die with these weights.
pub fn die(weights: &[u32]) -> Vec<Q> {
    let total: u32 = weights.iter().sum();
    weights
        .iter()
        .map(|&w| Q::new(i128::from(w), i128::from(total)))
        .collect()
}

/// The sum, over every choice of `n` different faces, of the product of
/// their chances: the chance of seeing those faces in one given order.
pub fn elementary(w: &[Q], n: usize) -> Q {
    let mut e = vec![Q::ZERO; n + 1];
    e[0] = Q::ONE;
    for &x in w {
        for k in (1..=n).rev() {
            e[k] = e[k] + e[k - 1] * x;
        }
    }
    e[n]
}

/// The chance that `n` rolls all show different faces: n! orders, each
/// equally likely.
pub fn all_different(w: &[Q], n: usize) -> Q {
    Q::int((1..=n as i128).product()) * elementary(w, n)
}

/// The chance that two rolls match.
pub fn tie(w: &[Q]) -> Q {
    w.iter().fold(Q::ZERO, |acc, &x| acc + x * x)
}

/// Which of the n! orders a run of all-different values came in, numbered
/// by sorting the orders themselves ("smallest first" is order 0).
pub fn order_index<V: Ord>(values: &[V]) -> usize {
    let n = values.len();
    let mut left: Vec<usize> = (0..n).collect();
    let mut index = 0;
    for (i, v) in values.iter().enumerate() {
        let rank = values.iter().filter(|o| *o < v).count();
        let pos = left.iter().position(|&r| r == rank).expect("values are all different");
        left.remove(pos);
        index = index * (n - i) + pos;
    }
    index
}

// ---------------------------------------------------------------------------
// what the trick throws away
// ---------------------------------------------------------------------------

/// The entropy of a coin, in fair tosses per toss.
pub fn entropy(p: f64) -> f64 {
    let term = |x: f64| if x <= 0.0 { 0.0 } else { -x * x.log2() };
    term(p) + term(1.0 - p)
}

/// Fair tosses out per toss in, for Peres's procedure iterated to `depth`
/// (depth 1 is von Neumann's rule): his Proposition 2.
pub fn peres_rate(depth: u32, p: f64) -> f64 {
    let q = 1.0 - p;
    if depth == 0 || p <= 0.0 || q <= 0.0 {
        return 0.0;
    }
    let same = p * p + q * q;
    p * q + 0.5 * peres_rate(depth - 1, same) + 0.5 * same * peres_rate(depth - 1, p * p / same)
}

/// The expected number of fair tosses Elias's rule reads from one fair
/// choice among `m` things: for each power of two in `m`, that many of the
/// choices give that many tosses.
pub fn elias_bits(m: u128) -> f64 {
    (0..128)
        .filter(|j| m >> j & 1 == 1)
        .map(|j| (1u128 << j) as f64 * f64::from(j))
        .sum::<f64>()
        / m as f64
}

/// Fair tosses out per toss in, for Elias's rule on blocks of `n` tosses.
pub fn elias_rate(n: u32, p: f64) -> f64 {
    (0..=n)
        .map(|k| {
            let m = choose(u64::from(n), u64::from(k));
            m as f64 * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32) * elias_bits(m)
        })
        .sum::<f64>()
        / f64::from(n)
}

/// Von Neumann's rule on a run of tosses: each pair that differs gives its
/// first toss.
pub fn von_neumann(tosses: &[Face]) -> Vec<Face> {
    let (pairs, _) = tosses.as_chunks::<2>();
    pairs.iter().filter(|c| c[0] != c[1]).map(|c| c[0]).collect()
}

/// Peres's procedure to `depth` (1 is von Neumann's rule; a large depth runs
/// until nothing is left): the rule itself, then the same procedure on
/// whether each pair matched, then on the faces of the matching pairs.
pub fn peres(tosses: &[Face], depth: u32) -> Vec<Face> {
    if depth == 0 || tosses.len() < 2 {
        return Vec::new();
    }
    let mut out = von_neumann(tosses);
    let (pairs, _) = tosses.as_chunks::<2>();
    let differs: Vec<Face> = pairs.iter().map(|c| if c[0] != c[1] { H } else { T }).collect();
    let matched: Vec<Face> = pairs.iter().filter(|c| c[0] == c[1]).map(|c| c[0]).collect();
    out.extend(peres(&differs, depth - 1));
    out.extend(peres(&matched, depth - 1));
    out
}

/// Elias's rule on blocks of `n` tosses: a block's place among the patterns
/// with the same number of heads is a fair choice, read off as fair tosses.
pub fn elias(tosses: &[Face], n: usize) -> Vec<Face> {
    let mut out = Vec::new();
    for block in tosses.chunks_exact(n) {
        // the block's rank among the patterns with as many heads, heads first
        let mut heads_left = block.iter().filter(|&&f| f == H).count() as u64;
        let m = choose(n as u64, heads_left);
        let mut rank = 0u128;
        for (i, &f) in block.iter().enumerate() {
            let rest = (n - 1 - i) as u64;
            if f == T {
                // every pattern with a head here comes first
                if heads_left > 0 {
                    rank += choose(rest, heads_left - 1);
                }
            } else {
                heads_left -= 1;
            }
        }
        // the first 2^j ranks give j tosses, for the biggest power of two in m, and so on down
        for j in (0..128).rev().filter(|j| m >> j & 1 == 1) {
            let size = 1u128 << j;
            if rank < size {
                out.extend((0..j).rev().map(|b| if rank >> b & 1 == 0 { H } else { T }));
                break;
            }
            rank -= size;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// seeded runs (bit-exact with NumPy's PCG64DXSM, so the oracle replays them)
// ---------------------------------------------------------------------------

/// 1951: the year von Neumann's note was printed. Run `i` uses its own
/// stream, so adding a run never changes another.
pub const SEED: u64 = 1951;

pub fn stream(i: u64) -> Data {
    Data::new(SEED * 100 + i)
}

/// One toss of a coin that shows heads with chance `p`.
pub fn toss(d: &mut Data, p: Q) -> Face {
    if d.unit() < p.f64() { H } else { T }
}

/// `n` tosses of one coin.
pub fn tosses(d: &mut Data, n: usize, p: Q) -> Vec<Face> {
    (0..n).map(|_| toss(d, p)).collect()
}

/// `n` tosses of a coin with memory, after a toss that showed `prev`.
pub fn memory_tosses(d: &mut Data, n: usize, m: Memory, mut prev: Face) -> Vec<Face> {
    (0..n)
        .map(|_| {
            prev = toss(d, m.heads_after(prev));
            prev
        })
        .collect()
}

/// One game of the trick: the pairs tossed, the last of which decides.
pub type Game = Vec<(Face, Face)>;

/// `n` games with a first coin and a second coin (the same chance twice for
/// one coin).
pub fn games(d: &mut Data, n: usize, first: Q, second: Q) -> Vec<Game> {
    (0..n)
        .map(|_| {
            let mut game = Vec::new();
            loop {
                let pair = (toss(d, first), toss(d, second));
                game.push(pair);
                if pair.0 != pair.1 {
                    return game;
                }
            }
        })
        .collect()
}

/// How many of these games heads-first won.
pub fn heads_first_wins(games: &[Game]) -> usize {
    games.iter().filter(|g| g.last().is_some_and(|p| p.0 == H)).count()
}

/// How many tosses these games used.
pub fn tosses_used(games: &[Game]) -> usize {
    games.iter().map(|g| 2 * g.len()).sum()
}

/// One roll of a weighted die: its face, 1 to 6.
pub fn roll(d: &mut Data, weights: &[u32]) -> usize {
    let total: u32 = weights.iter().sum();
    let u = d.unit();
    let mut upto = 0;
    for (face, &w) in weights.iter().enumerate() {
        upto += w;
        if u < f64::from(upto) / f64::from(total) {
            return face + 1;
        }
    }
    weights.len()
}

/// `n` rolls of a weighted die.
pub fn rolls(d: &mut Data, n: usize, weights: &[u32]) -> Vec<usize> {
    (0..n).map(|_| roll(d, weights)).collect()
}

/// Cut `values` into blocks of `n` and keep the blocks `keep` accepts, each
/// as the index `keep` gives it.
pub fn blocks<V>(values: &[V], n: usize, keep: impl Fn(&[V]) -> Option<usize>) -> Vec<usize> {
    values.chunks_exact(n).filter_map(keep).collect()
}

/// A fair die face (0 to 5) from four tosses with exactly two heads: which
/// of the six patterns came up. None for any other count of heads.
pub fn die_from_four_tosses(block: &[Face]) -> Option<usize> {
    let heads = block.iter().filter(|&&f| f == H).count();
    (block.len() == 4 && heads == 2).then(|| {
        patterns(4, 2)
            .iter()
            .position(|p| p == block)
            .expect("a pattern of four tosses with two heads")
    })
}

/// A fair choice among n! from `n` rolls that all differ: the order they
/// came in. None when two rolls match.
pub fn order_of_rolls(block: &[usize]) -> Option<usize> {
    let mut sorted = block.to_vec();
    sorted.sort_unstable();
    sorted.windows(2).all(|w| w[0] != w[1]).then(|| order_index(block))
}

/// How many times each of `kinds` values appears.
pub fn tally(values: &[usize], kinds: usize) -> Vec<usize> {
    let mut t = vec![0; kinds];
    for &v in values {
        t[v] += 1;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(n: i128, d: i128) -> Q {
        Q::new(n, d)
    }

    /// Chances to try an always/never sentence at: both ends, the middle,
    /// and values near each end.
    fn chances() -> Vec<Q> {
        let mut v = vec![Q::ZERO, Q::ONE, Q::HALF, q(1, 1000), q(999, 1000), bent()];
        v.extend((1..20).map(|i| q(i, 20)));
        v.extend((1..7).map(|i| q(i, 7)));
        v
    }

    #[test]
    fn fractions_reduce_compare_and_round_exactly() {
        assert_eq!(q(6, -8), q(-3, 4));
        assert_eq!(q(1, 3) + q(1, 6), Q::HALF);
        assert_eq!(q(4, 5) * q(1, 5), q(4, 25));
        assert_eq!(Q::ONE / q(4, 25), q(25, 4));
        assert!(q(1, 3) < q(2, 5));
        assert_eq!(q(25, 4).decimal(2), "6.25");
        assert_eq!(q(1, 8).decimal(2), "0.13"); // 0.125 rounds half away from zero
        assert_eq!(q(-1, 8).decimal(2), "-0.13");
        assert_eq!(q(10000, 99).decimal(0), "101");
        assert_eq!(q(1, 3).decimal(4), "0.3333");
        assert_eq!(q(-1, 1000).decimal(2), "0.00");
        assert_eq!(bent().percent(0), "80");
        assert_eq!(q(4, 25).exact(), "4/25");
        assert_eq!(Q::int(6).exact(), "6");
    }

    #[test]
    fn the_solver_solves_and_refuses_a_singular_system() {
        let x = solve(
            vec![vec![q(2, 1), q(1, 1)], vec![q(1, 1), q(3, 1)]],
            vec![q(3, 1), q(5, 1)],
        )
        .unwrap();
        assert_eq!(x, [q(4, 5), q(7, 5)]);
        assert!(
            solve(
                vec![vec![q(1, 1), q(2, 1)], vec![q(2, 1), q(4, 1)]],
                vec![Q::ONE, Q::ONE]
            )
            .is_none()
        );
    }

    /// "Heads-tails and tails-heads are equally likely, whatever the coin":
    /// every chance, both ends included.
    #[test]
    fn edge_heads_tails_equals_tails_heads_for_every_chance() {
        for p in chances() {
            let law = PairLaw::same_coin(p);
            assert_eq!(law.prob(H, T), law.prob(T, H), "p = {}", p.exact());
            let total = PAIRS.iter().fold(Q::ZERO, |acc, &(a, b)| acc + law.prob(a, b));
            assert_eq!(total, Q::ONE);
            match law.heads_first_share() {
                Some(share) => assert_eq!(share, Q::HALF),
                // the two ends: a coin with one face never decides
                None => assert!(p == Q::ZERO || p == Q::ONE),
            }
        }
    }

    /// "A pair is thrown away at least half the time": equality only for a
    /// fair coin.
    #[test]
    fn edge_at_least_half_the_pairs_are_thrown_away() {
        for p in chances() {
            let thrown = PairLaw::same_coin(p).decides().other();
            assert!(thrown >= Q::HALF, "p = {}", p.exact());
            assert_eq!(thrown == Q::HALF, p == Q::HALF);
        }
    }

    /// "It never takes fewer than four tosses on average": the cost is
    /// smallest for a fair coin, and has no bound near the ends.
    #[test]
    fn edge_expected_tosses_bottom_out_at_four() {
        for p in chances() {
            match expected_tosses(p) {
                Some(e) => {
                    assert!(e >= Q::int(4));
                    assert_eq!(e == Q::int(4), p == Q::HALF);
                }
                None => assert!(p == Q::ZERO || p == Q::ONE),
            }
        }
        assert_eq!(expected_tosses(bent()), Some(q(25, 4)));
        assert!(expected_tosses(q(999, 1000)).unwrap() > Q::int(1000));
    }

    #[test]
    fn the_band_is_two_standard_errors_either_side() {
        // 1,000 fair games: 500 ± 2 × 15.81
        assert_eq!(two_sigma_band(1000, Q::HALF), (469, 531));
        // one face of a fair die in 949 rolls: 158.17 ± 2 × 11.48
        assert_eq!(two_sigma_band(949, q(1, 6)), (136, 181));
        assert_eq!(two_sigma_band(10, Q::ZERO), (0, 0));
        assert_eq!(two_sigma_band(10, Q::ONE), (10, 10));
    }

    /// "Two different coins break it": the two chances agree only when the
    /// coins do (or neither pair can come up at all).
    #[test]
    fn edge_two_coins_are_fair_only_when_they_match() {
        for p1 in chances() {
            for p2 in chances() {
                let law = PairLaw::two_coins(p1, p2);
                assert_eq!(
                    law.prob(H, T) == law.prob(T, H),
                    p1 == p2,
                    "{} then {}",
                    p1.exact(),
                    p2.exact()
                );
            }
        }
        // a fair second coin hands back the first coin's chance untouched
        for p1 in chances() {
            assert_eq!(PairLaw::two_coins(p1, Q::HALF).heads_first_share(), Some(p1));
        }
    }

    /// "No rule that stops after a fixed number of tosses is fair for every
    /// coin": every rule on up to five tosses, by its counts.
    #[test]
    fn edge_no_fixed_length_rule_is_fair_for_every_coin() {
        for (n, lists) in [(1u32, 4u64), (2, 12), (3, 64), (4, 700), (5, 17_424)] {
            assert_eq!(fixed_length_rules(n, &spread_chances(n)), (lists, 0), "n = {n}");
        }
        // the check can say yes: plenty of rules are fair for a fair coin alone
        assert_eq!(fixed_length_rules(1, &[Q::HALF]), (4, 2));
        assert_eq!(fixed_length_rules(2, &[Q::HALF]), (12, 4));
        // and none survives one more chance
        assert_eq!(fixed_length_rules(2, &[Q::HALF, bent()]), (12, 0));
    }

    /// A coin with memory: the first pair of a long run is fair, the plain
    /// rule after a thrown-away pair is not, and a coin with no memory is
    /// the one case where the plain rule is fair from every start.
    #[test]
    fn edge_memory_breaks_the_plain_rule_but_not_the_first_long_run_pair() {
        for a in chances() {
            for b in chances() {
                let m = Memory { after_h: a, after_t: b };
                if let Some(pair) = m.long_run_pair() {
                    assert_eq!(pair.prob(H, T), pair.prob(T, H), "{} {}", a.exact(), b.exact());
                }
                let (h, t) = (m.pair_after(H), m.pair_after(T));
                let inner = |x: Q| x > Q::ZERO && x < Q::ONE;
                if inner(a) && inner(b) {
                    assert_eq!(h.prob(H, T) == h.prob(T, H), a == b);
                    assert_eq!(t.prob(H, T) == t.prob(T, H), a == b);
                    let fair_from_both = m.plain_rule(H) == Some(Q::HALF) && m.plain_rule(T) == Some(Q::HALF);
                    assert_eq!(fair_from_both, a == b, "{} {}", a.exact(), b.exact());
                }
            }
        }
        let sticky = Memory::sticky(bent());
        assert_eq!(sticky.pair_after(H).heads_first_share(), Some(q(4, 5)));
        assert_eq!(sticky.pair_after(T).heads_first_share(), Some(q(1, 5)));
        assert_eq!(sticky.plain_rule_long_run(), Some(Q::HALF)); // by symmetry, before any toss
        // a lopsided memory is unfair even from a long-run start
        let lopsided = Memory {
            after_h: q(9, 10),
            after_t: q(1, 2),
        };
        assert_eq!(lopsided.plain_rule(H), Some(q(7, 12)));
        assert_eq!(lopsided.plain_rule(T), Some(q(5, 12)));
        assert_eq!(lopsided.plain_rule_long_run(), Some(q(5, 9)));
    }

    /// Samuelson's rule is fair from every start, for every memory that can
    /// still show both faces after the chosen face and keeps returning to it.
    #[test]
    fn edge_after_only_rule_is_fair_for_every_memory() {
        for a in chances() {
            for b in chances() {
                let m = Memory { after_h: a, after_t: b };
                for after in [H, T] {
                    for prev in [H, T] {
                        let usable = match after {
                            H => a > Q::ZERO && a < Q::ONE && b > Q::ZERO,
                            T => b > Q::ZERO && b < Q::ONE && a < Q::ONE,
                        };
                        let got = m.after_only_rule(after, prev);
                        if usable {
                            assert_eq!(got, Some(Q::HALF), "{} {} {after:?} {prev:?}", a.exact(), b.exact());
                        } else {
                            assert_ne!(got, Some(Q::HALF), "{} {} {after:?} {prev:?}", a.exact(), b.exact());
                        }
                    }
                }
            }
        }
    }

    /// Blum's Theorems A and B on his own chain: announcing at once is not
    /// fair coin tossing, announcing on return is.
    #[test]
    fn blum_at_once_is_bad_and_on_return_is_good() {
        for stay in [0.8, 0.9, 0.5, 0.3] {
            let b = blum_first_two(stay, Announce::OnReturn);
            for x in b {
                assert!((x - 0.25).abs() < 1e-12, "stay {stay}: {b:?}");
            }
        }
        let a = blum_first_two(0.8, Announce::AtOnce);
        assert!((a.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        for (got, want) in a.iter().zip([0.15, 0.15, 0.35, 0.35]) {
            assert!((got - want).abs() < 1e-12, "{a:?}");
        }
        // his "alternatively ε = 1/2": even when every exit is a fair toss,
        // announcing at once makes a T the likelier first result (5 in 8)
        let fair_exits = blum_first_two(0.5, Announce::AtOnce);
        assert!((fair_exits[2] + fair_exits[3] - 0.625).abs() < 1e-12, "{fair_exits:?}");
    }

    /// "For tosses like ours, with the same chance every time and no memory,
    /// order never changes a chance": every pattern, every chance, checked
    /// against the product of its own tosses. The fence is needed: for the
    /// coin with memory, two patterns with the same counts differ.
    #[test]
    fn edge_same_counts_same_chance_without_memory() {
        let sticky = Memory::sticky(bent()).pair_after(H);
        assert_eq!(sticky.prob(H, T), q(4, 25));
        assert_eq!(sticky.prob(T, H), q(1, 25));
        // and with no memory the same pair law gives them one chance again
        let plain = Memory {
            after_h: bent(),
            after_t: bent(),
        }
        .pair_after(H);
        assert_eq!(plain.prob(H, T), plain.prob(T, H));
        for p in chances() {
            for n in 1..=6u32 {
                let mut total = Q::ZERO;
                for k in 0..=n {
                    let ps = patterns(n, k);
                    assert_eq!(ps.len() as u128, pascal_row(n)[k as usize]);
                    for pat in &ps {
                        let direct = pat.iter().fold(Q::ONE, |acc, &f| {
                            acc * match f {
                                H => p,
                                T => p.other(),
                            }
                        });
                        assert_eq!(direct, pattern_chance(p, n, k));
                    }
                    total = total + count_chance(p, n, k);
                }
                assert_eq!(total, Q::ONE);
            }
        }
        assert_eq!(patterns(4, 2).len(), 6);
        assert_eq!(patterns(4, 2)[0], [H, H, T, T]);
        assert_eq!(patterns(4, 2)[5], [T, T, H, H]);
        assert_eq!(patterns(2, 1), [vec![H, T], vec![T, H]]);
    }

    /// "The n! orders of n different rolls are equally likely": by listing
    /// every run of rolls of the loaded die and of a fair one.
    #[test]
    fn edge_every_order_of_different_rolls_is_equally_likely() {
        for weights in [LOADED, FAIR_DIE, [1, 2, 3, 4, 5, 6]] {
            let w = die(&weights);
            for n in 2..=3usize {
                let kinds: usize = (1..=n).product();
                let mut by_order = vec![Q::ZERO; kinds];
                let mut ties = Q::ZERO;
                for code in 0..6usize.pow(n as u32) {
                    let faces: Vec<usize> = (0..n).map(|i| code / 6usize.pow(i as u32) % 6 + 1).collect();
                    let chance = faces.iter().fold(Q::ONE, |acc, &f| acc * w[f - 1]);
                    match order_of_rolls(&faces) {
                        Some(i) => by_order[i] = by_order[i] + chance,
                        None => ties = ties + chance,
                    }
                }
                for &c in &by_order {
                    assert_eq!(c, elementary(&w, n));
                }
                assert_eq!(all_different(&w, n) + ties, Q::ONE);
            }
            assert_eq!(all_different(&w, 2), tie(&w).other());
        }
        assert_eq!(all_different(&die(&FAIR_DIE), 3), q(5, 9));
        assert_eq!(order_index(&[1, 2, 3]), 0);
        assert_eq!(order_index(&[3, 2, 1]), 5);
        assert_eq!(order_index(&[2, 6]), 0);
        assert_eq!(order_index(&[6, 2]), 1);
    }

    /// "No rule beats the entropy": every rate here stays under it, and
    /// Peres's and Elias's climb towards it.
    #[test]
    fn edge_rates_climb_to_the_entropy_and_never_pass_it() {
        for p in [0.5, 0.6, 0.8, 0.9, 0.99, 0.01] {
            let h = entropy(p);
            let mut last = 0.0;
            for depth in 1..=10 {
                let r = peres_rate(depth, p);
                assert!(r > last && r < h + 1e-12, "p {p} depth {depth}: {r} vs {h}");
                last = r;
            }
            assert!((peres_rate(1, p) - p * (1.0 - p)).abs() < 1e-15);
            assert!((elias_rate(2, p) - p * (1.0 - p)).abs() < 1e-15);
            let mut last = 0.0;
            for n in [2u32, 4, 8, 16, 32, 64] {
                let r = elias_rate(n, p);
                assert!(r > last && r < h + 1e-12, "p {p} block {n}: {r} vs {h}");
                last = r;
            }
        }
        assert!((entropy(0.5) - 1.0).abs() < 1e-15);
        assert_eq!(entropy(0.0), 0.0);
        assert_eq!(entropy(1.0), 0.0);
        assert!((elias_bits(6) - 5.0 / 3.0).abs() < 1e-15); // 4 of the 6 give two tosses, 2 give one
        assert_eq!(elias_bits(1), 0.0);
        assert_eq!(elias_bits(4), 2.0);
    }

    /// The three procedures on every run of eight tosses of a lopsided
    /// coin: any two outputs of the same length are exactly equally likely,
    /// which is what "fair tosses" means for a run of fixed length.
    #[test]
    fn the_procedures_give_fair_output_on_every_short_run() {
        type Procedure = Box<dyn Fn(&[Face]) -> Vec<Face>>;
        let p = bent();
        let procedures: [(&str, Procedure); 4] = [
            ("von Neumann", Box::new(von_neumann)),
            ("Peres 3", Box::new(|t| peres(t, 3))),
            ("Peres all", Box::new(|t| peres(t, 32))),
            ("Elias 4", Box::new(|t| elias(t, 4))),
        ];
        for (name, f) in &procedures {
            let n = 8u32;
            let mut chance: HashMap<Vec<Face>, Q> = HashMap::new();
            for code in 0..1u32 << n {
                let run: Vec<Face> = (0..n).map(|i| if code >> i & 1 == 1 { H } else { T }).collect();
                let heads = run.iter().filter(|&&x| x == H).count() as u32;
                let e = chance.entry(f(&run)).or_insert(Q::ZERO);
                *e = *e + pattern_chance(p, n, heads);
            }
            let mut by_length: HashMap<usize, (Q, usize)> = HashMap::new();
            for (out, &c) in &chance {
                let (first, seen) = by_length.entry(out.len()).or_insert((c, 0));
                assert_eq!(*first, c, "{name}: {out:?}");
                *seen += 1;
            }
            // and every output of each length that occurs does occur
            for (len, (_, seen)) in by_length {
                assert_eq!(seen, 1 << len, "{name}: outputs of length {len}");
            }
        }
        assert_eq!(von_neumann(&[H, T, T, T, T, H, H, H]), [H, T]);
        assert_eq!(peres(&[H, T, T, T, T, H, H, H], 1), [H, T]);
    }

    #[test]
    fn seeded_runs_are_reproducible_and_plausible() {
        let a = tosses(&mut stream(1), 1000, bent());
        assert_eq!(a, tosses(&mut stream(1), 1000, bent()));
        let heads = a.iter().filter(|&&f| f == H).count();
        assert!((700..=900).contains(&heads));
        let g = games(&mut stream(2), 1000, bent(), bent());
        assert_eq!(g.len(), 1000);
        assert!(g.iter().all(|game| {
            let (last, rest) = game.split_last().unwrap();
            last.0 != last.1 && rest.iter().all(|p| p.0 == p.1)
        }));
        let rolled = rolls(&mut stream(3), 600, &LOADED);
        assert!(rolled.iter().all(|&f| (1..=6).contains(&f)));
        assert_eq!(die_from_four_tosses(&[H, H, T, T]), Some(0));
        assert_eq!(die_from_four_tosses(&[T, T, H, H]), Some(5));
        assert_eq!(die_from_four_tosses(&[H, H, H, T]), None);
        assert_eq!(order_of_rolls(&[2, 6, 4]), Some(order_index(&[2, 6, 4])));
        assert_eq!(order_of_rolls(&[2, 6, 2]), None);
        assert_eq!(tally(&[0, 2, 2], 3), [1, 0, 2]);
    }
}
