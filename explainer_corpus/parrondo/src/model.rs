//! The mathematics of the two-games video: Harmer and Abbott's games A and B
//! on the three remainders of the player's dollars, their long-run shares,
//! averages and products, the schedules that switch between them, the crowd
//! that splits by the coins' chances, and one simulated player.
//!
//! Every chance is a whole number of [`Game::den`]ths (200ths for every game
//! the video plays), so the long-run arithmetic is exact on integers:
//! - a game's **long-run shares** (its weights) come from the spanning-tree
//!   formula on the triangle, [`Game::tree_weights`]; the oracle instead
//!   solves π P = π by elimination;
//! - its **average per round** is three times the net flow round the
//!   triangle, 3 (p₀p₁p₂ − q₀q₁q₂) / W ([`Game::mean`]), the video's sign
//!   rule; the oracle instead sums π_i (2 p_i − 1);
//! - a **periodic schedule** multiplies its rounds' integer move matrices
//!   and takes the period's spanning-tree weights ([`periodic`]).
//!
//! The finite-play numbers (the dollars after 100 and 1,000 rounds, where
//! the share of game A stops winning, the crowd's settling round) are f64:
//! the oracle computes them exactly and checks that each is far from a
//! rounding boundary, so the floats print the same digits.
//!
//! Assumptions, as the video states them: the capital is any whole number,
//! negatives allowed, and play never stops; "divides by three" means
//! remainder 0 (`rem_euclid`, so −1 leaves 2); the mix is a fair coin
//! choosing A or B independently before every round.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// The remainder game B reads: the player's dollars divided by three.
pub const MODULUS: i64 = 3;

/// Every chance in the video is a whole number of 200ths.
pub const DEN: i64 = 200;

// ------------------------------------------------------------ exact numbers

const fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

fn times(a: i128, b: i128) -> i128 {
    a.checked_mul(b).expect("exact arithmetic overflows i128")
}

fn plus(a: i128, b: i128) -> i128 {
    a.checked_add(b).expect("exact arithmetic overflows i128")
}

fn divisor(a: i128, b: i128) -> i128 {
    i128::try_from(gcd(a.unsigned_abs(), b.unsigned_abs())).expect("a divisor fits i128")
}

/// The least common multiple of two positive integers.
pub fn lcm(a: i128, b: i128) -> i128 {
    times(a / divisor(a, b), b)
}

/// A rational number in lowest terms, its denominator positive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ratio {
    num: i128,
    den: i128,
}

impl Ratio {
    pub const ZERO: Self = Self { num: 0, den: 1 };
    pub const ONE: Self = Self { num: 1, den: 1 };

    /// `num / den` in lowest terms.
    ///
    /// # Panics
    /// If `den` is zero.
    pub fn new(num: i128, den: i128) -> Self {
        assert!(den != 0, "a ratio with a zero denominator");
        let g = divisor(num, den);
        let (num, den) = (num / g, den / g);
        if den < 0 {
            Self { num: -num, den: -den }
        } else {
            Self { num, den }
        }
    }

    pub fn int(n: i64) -> Self {
        Self::new(n.into(), 1)
    }

    pub fn num(self) -> i128 {
        self.num
    }

    pub fn den(self) -> i128 {
        self.den
    }

    pub fn signum(self) -> i32 {
        self.num.signum() as i32
    }

    pub fn is_integer(self) -> bool {
        self.den == 1
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Rounded half away from zero to `places` decimals, exactly: "5.9" for
    /// 5.85 at one place, "-0.87", "0.0" (never "-0.0").
    pub fn fixed(self, places: u32) -> String {
        let scaled = times(self.num.abs(), 10i128.pow(places));
        let mut whole = scaled / self.den;
        if 2 * (scaled % self.den) >= self.den {
            whole += 1;
        }
        let sign = if self.num < 0 && whole != 0 { "-" } else { "" };
        let digits = format!("{whole:0>width$}", width = places as usize + 1);
        if places == 0 {
            return format!("{sign}{digits}");
        }
        let (int, dec) = digits.split_at(digits.len() - places as usize);
        format!("{sign}{int}.{dec}")
    }

    /// As [`Ratio::fixed`], with "+" before a positive value that does not
    /// round to zero: "+1.57".
    pub fn signed(self, places: u32) -> String {
        let text = self.fixed(places);
        if self.num > 0 && text.chars().any(|c| c.is_ascii_digit() && c != '0') {
            format!("+{text}")
        } else {
            text
        }
    }

    /// The percentage, rounded: 0.38361… → "38.4" at one place.
    pub fn pct(self, places: u32) -> String {
        (self * Self::int(100)).fixed(places)
    }

    /// A percentage that is exactly a whole number of tenths, without a
    /// trailing ".0": "49.5", "50", "62.5".
    pub fn exact_pct(self) -> Result<String, String> {
        let tenths = self * Self::int(1000);
        if !tenths.is_integer() {
            return Err(format!("{self} is not a whole number of tenths of a percent"));
        }
        let text = (self * Self::int(100)).fixed(1);
        Ok(text.strip_suffix(".0").map(str::to_owned).unwrap_or(text))
    }

    /// Dollars as signed cents: −73443/8446300 → "-0.87" at two places.
    pub fn cents(self, places: u32) -> String {
        (self * Self::int(100)).signed(places)
    }
}

impl fmt::Display for Ratio {
    /// "3/4", "-1/100", "0", as the oracle's fractions print.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

impl Add for Ratio {
    type Output = Self;
    /// Over the least common denominator.
    fn add(self, o: Self) -> Self {
        let g = divisor(self.den, o.den);
        let (left, right) = (self.den / g, o.den / g);
        Self::new(plus(times(self.num, right), times(o.num, left)), times(self.den, right))
    }
}

impl Sub for Ratio {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        self + -o
    }
}

impl Mul for Ratio {
    type Output = Self;
    /// Cross-cancelled first, so products stay as small as their result.
    fn mul(self, o: Self) -> Self {
        let a = divisor(self.num, o.den).max(1);
        let b = divisor(o.num, self.den).max(1);
        Self::new(times(self.num / a, o.num / b), times(self.den / b, o.den / a))
    }
}

impl Div for Ratio {
    type Output = Self;
    /// # Panics
    /// If `o` is zero.
    fn div(self, o: Self) -> Self {
        assert!(o.num != 0, "division by a zero ratio");
        self * Self::new(o.den, o.num)
    }
}

impl Neg for Ratio {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            num: -self.num,
            den: self.den,
        }
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Ratio {
    fn cmp(&self, o: &Self) -> Ordering {
        (*self - *o).num.cmp(&0)
    }
}

// ---------------------------------------------------------------- the games

/// A game on the triangle: the chance of winning (moving one corner
/// forward) at each remainder 0, 1, 2, as `up[i] / den`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Game {
    pub up: [i64; 3],
    pub den: i64,
}

impl Game {
    /// The chance of moving back from corner `i`, in `den`ths.
    pub fn down(&self, i: usize) -> i64 {
        self.den - self.up[i]
    }

    /// The chance of winning at corner `i`.
    pub fn chance(&self, i: usize) -> Ratio {
        Ratio::new(self.up[i].into(), self.den.into())
    }

    /// The spanning-tree weights of the corners, in `den`⁻² (Markov chain
    /// tree theorem on the triangle): corner 0's is q₁p₂ + p₁p₂ + q₁q₂, the
    /// three ways for corners 1 and 2 to lead into 0, and so on round.
    pub fn tree_weights(&self) -> [i128; 3] {
        let p = |i: usize| i128::from(self.up[i]);
        let q = |i: usize| i128::from(self.down(i));
        [
            q(1) * p(2) + p(1) * p(2) + q(1) * q(2),
            p(0) * q(2) + p(0) * p(2) + q(0) * q(2),
            q(0) * p(1) + p(0) * p(1) + q(0) * q(1),
        ]
    }

    /// Each corner's long-run share of the rounds (its weight).
    pub fn shares(&self) -> [Ratio; 3] {
        let w = self.tree_weights();
        let total: i128 = w.iter().sum();
        w.map(|x| Ratio::new(x, total))
    }

    /// The forward and backward products, p₀p₁p₂ and q₀q₁q₂: winning three
    /// times round the triangle against losing three times the other way.
    pub fn products(&self) -> (Ratio, Ratio) {
        let d = i128::from(self.den).pow(3);
        let f = (0..3).map(|i| i128::from(self.up[i])).product::<i128>();
        let b = (0..3).map(|i| i128::from(self.down(i))).product::<i128>();
        (Ratio::new(f, d), Ratio::new(b, d))
    }

    /// The long-run average gain per round, in dollars: three times the net
    /// flow across each edge, 3 (p₀p₁p₂ − q₀q₁q₂) / W, with W the sum of the
    /// tree weights (in den⁻², so one more den below).
    pub fn mean(&self) -> Ratio {
        let f = (0..3).map(|i| i128::from(self.up[i])).product::<i128>();
        let b = (0..3).map(|i| i128::from(self.down(i))).product::<i128>();
        let w: i128 = self.tree_weights().iter().sum();
        Ratio::new(3 * (f - b), times(i128::from(self.den), w))
    }

    /// The share of rounds won when the corners hold `weights`.
    pub fn win_at(&self, weights: &[Ratio; 3]) -> Ratio {
        (0..3).fold(Ratio::ZERO, |acc, i| acc + weights[i] * self.chance(i))
    }

    /// The long-run share of rounds won.
    pub fn win(&self) -> Ratio {
        self.win_at(&self.shares())
    }

    /// This game on a share `num/den` of the rounds and `other` on the rest,
    /// the choice made by an independent coin: one game with the averaged
    /// chances (in this den times the share's).
    pub fn mixed(&self, other: &Game, num: i64, den: i64) -> Game {
        assert_eq!(self.den, other.den, "games mixed over different denominators");
        assert!((0..=den).contains(&num) && den > 0, "a share is between 0 and 1");
        Game {
            up: [0, 1, 2].map(|i| num * self.up[i] + (den - num) * other.up[i]),
            den: self.den * den,
        }
    }

    /// Chances as floats, for the finite-play numbers.
    pub fn chances_f64(&self) -> [f64; 3] {
        [0, 1, 2].map(|i| self.up[i] as f64 / self.den as f64)
    }
}

/// The three coins of the two games: game A's one coin, and game B's bad
/// coin (at remainder 0) and good coin (elsewhere), in [`DEN`]ths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coins {
    pub a: i64,
    pub bad: i64,
    pub good: i64,
}

/// The handicap Harmer and Abbott shave off every coin: ε = 1/200, one
/// [`DEN`]th, half a percentage point.
pub const HANDICAP: i64 = 1;

/// The round-number coins, the video's worked example (no handicap):
/// heads 1/2, 1/10, 3/4.
pub const ROUND: Coins = Coins {
    a: 100,
    bad: 20,
    good: 150,
};

/// The published coins (Harmer & Abbott, Nature 402, 864, 1999): 1/2 − ε,
/// 1/10 − ε, 3/4 − ε with ε = 0.005.
pub const PUBLISHED: Coins = Coins {
    a: ROUND.a - HANDICAP,
    bad: ROUND.bad - HANDICAP,
    good: ROUND.good - HANDICAP,
};

impl Coins {
    /// Game A: the same coin at every corner.
    pub fn game_a(&self) -> Game {
        Game {
            up: [self.a; 3],
            den: DEN,
        }
    }

    /// Game B: the bad coin when the dollars divide by three.
    pub fn game_b(&self) -> Game {
        Game {
            up: [self.bad, self.good, self.good],
            den: DEN,
        }
    }

    /// The mix: a fair coin picks A or B before every round. Its chances
    /// are back in [`DEN`]ths when the two games' chances add to even
    /// numbers, as they do for both sets of coins.
    pub fn mix(&self) -> Game {
        let m = self.game_a().mixed(&self.game_b(), 1, 2);
        if m.up.iter().all(|u| u % 2 == 0) {
            Game {
                up: m.up.map(|u| u / 2),
                den: m.den / 2,
            }
        } else {
            m
        }
    }

    /// The mix with game A on a share `num/den` of the rounds.
    pub fn mix_share(&self, num: i64, den: i64) -> Game {
        self.game_a().mixed(&self.game_b(), num, den)
    }

    /// Switching by the count: B when the dollars divide by three, A
    /// otherwise (chapter 18's last puzzle).
    pub fn b_at_zero(&self) -> Game {
        Game {
            up: [self.bad, self.a, self.a],
            den: DEN,
        }
    }

    /// The same switch swapped: A when the dollars divide by three, B
    /// otherwise.
    pub fn a_at_zero(&self) -> Game {
        Game {
            up: [self.a, self.good, self.good],
            den: DEN,
        }
    }

    /// B's coins chosen by a die instead of the money: the bad coin one round
    /// in three, wherever the player stands (chapter 10's control).
    pub fn die_chosen_win(&self) -> Ratio {
        Ratio::new((self.bad + 2 * self.good).into(), (3 * DEN).into())
    }
}

// ----------------------------------------------------------------- schedules

type Moves = [[i128; 3]; 3];

/// One round's moves as whole numbers: entry (i, j) is den times the chance
/// of going from corner i to corner j.
fn moves(g: &Game) -> Moves {
    let mut m = [[0i128; 3]; 3];
    for i in 0..3 {
        m[i][(i + 1) % 3] += i128::from(g.up[i]);
        m[i][(i + 2) % 3] += i128::from(g.down(i));
    }
    m
}

fn compose(a: &Moves, b: &Moves) -> Moves {
    let mut out = [[0i128; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = (0..3).fold(0, |acc, k| plus(acc, times(a[i][k], b[k][j])));
        }
    }
    out
}

fn after(v: &[i128; 3], m: &Moves) -> [i128; 3] {
    [0, 1, 2].map(|j| (0..3).fold(0, |acc, i| plus(acc, times(v[i], m[i][j]))))
}

/// The spanning-tree weights of any three-state chain with moves `t`
/// (self-moves do not enter).
fn chain_weights(t: &Moves) -> [i128; 3] {
    [
        times(t[1][0], t[2][0]) + times(t[1][0], t[2][1]) + times(t[1][2], t[2][0]),
        times(t[0][1], t[2][1]) + times(t[0][1], t[2][0]) + times(t[0][2], t[2][1]),
        times(t[0][2], t[1][2]) + times(t[0][2], t[1][0]) + times(t[0][1], t[1][2]),
    ]
}

/// A schedule of games repeated forever (AB, AABB, ...): the long-run
/// average per round played, and whether the shares at the period's start
/// come back after one period (they must). All games share one den.
pub fn periodic(games: &[Game]) -> (Ratio, bool) {
    assert!(!games.is_empty(), "an empty schedule");
    let den = i128::from(games[0].den);
    assert!(
        games.iter().all(|g| i128::from(g.den) == den),
        "a schedule over different denominators"
    );
    let steps: Vec<Moves> = games.iter().map(moves).collect();
    let period = steps[1..].iter().fold(steps[0], |acc, m| compose(&acc, m));
    let w = chain_weights(&period);
    let total: i128 = w.iter().sum();
    let len = games.len() as u32;
    // phase k's gain over the common denominator W · den^len
    let mut u = w;
    let mut gained = 0i128;
    for (k, (g, m)) in games.iter().zip(&steps).enumerate() {
        let term = (0..3).fold(0, |acc, i| plus(acc, times(u[i], i128::from(2 * g.up[i] - g.den))));
        gained = plus(gained, times(term, den.pow(len - 1 - k as u32)));
        u = after(&u, m);
    }
    let scale = den.pow(len);
    let back = u.iter().zip(&w).all(|(&x, &y)| x == times(y, scale));
    (Ratio::new(gained, times(times(total, scale), len.into())), back)
}

/// The games of a schedule written as letters, "AABB".
pub fn schedule(pattern: &str, coins: &Coins) -> Vec<Game> {
    pattern
        .chars()
        .map(|c| match c {
            'A' => coins.game_a(),
            'B' => coins.game_b(),
            other => panic!("a schedule letter is A or B, not {other:?}"),
        })
        .collect()
}

/// A schedule repeated forever, once it has settled into its cycle: the
/// shares each of its rounds meets (entry k for game k of the period),
/// exactly. The period's spanning-tree weights, carried round by round.
pub fn cycle(games: &[Game]) -> Vec<[Ratio; 3]> {
    assert!(!games.is_empty(), "an empty schedule");
    let den = i128::from(games[0].den);
    assert!(
        games.iter().all(|g| i128::from(g.den) == den),
        "a schedule over different denominators"
    );
    let steps: Vec<Moves> = games.iter().map(moves).collect();
    let period = steps[1..].iter().fold(steps[0], |acc, m| compose(&acc, m));
    let w = chain_weights(&period);
    let total: i128 = w.iter().sum();
    let mut u = w;
    let mut out = Vec::with_capacity(games.len());
    for (k, m) in steps.iter().enumerate() {
        let scale = times(total, den.pow(k as u32));
        out.push(u.map(|x| Ratio::new(x, scale)));
        u = after(&u, m);
    }
    out
}

/// The crowd's shares before each of `rounds` rounds of `games` repeated,
/// from `start`, in floats (chapter 16's schedules, one round at a time).
pub fn played(games: &[Game], start: [f64; 3], rounds: usize) -> Vec<[f64; 3]> {
    assert!(!games.is_empty(), "an empty schedule");
    let mut v = start;
    (0..rounds)
        .map(|k| {
            let before = v;
            v = evolve_f64(&v, &games[k % games.len()]);
            before
        })
        .collect()
}

// ------------------------------------------------------------------ the crowd

/// The players going forward and back from each corner when `n` players
/// sit at the long-run shares and split exactly by the coins' chances.
pub fn flows(g: &Game, n: i64) -> ([Ratio; 3], [Ratio; 3]) {
    let shares = g.shares();
    let n = Ratio::int(n);
    let forward = [0, 1, 2].map(|i| n * shares[i] * g.chance(i));
    let back = [0, 1, 2].map(|i| n * shares[i] * (Ratio::ONE - g.chance(i)));
    (forward, back)
}

/// The smallest crowd whose settled flows are all whole players: the least
/// common multiple of the flows' denominators per player.
pub fn smallest_whole_crowd(g: &Game) -> i64 {
    let (forward, back) = flows(g, 1);
    let n = forward.iter().chain(&back).fold(1i128, |acc, x| lcm(acc, x.den()));
    i64::try_from(n).expect("a crowd size fits i64")
}

/// The crowd one round later: every corner sends its forward share on and
/// its backward share back.
pub fn evolve(v: &[Ratio; 3], g: &Game) -> [Ratio; 3] {
    let mut out = [Ratio::ZERO; 3];
    for i in 0..3 {
        out[(i + 1) % 3] = out[(i + 1) % 3] + v[i] * g.chance(i);
        out[(i + 2) % 3] = out[(i + 2) % 3] + v[i] * (Ratio::ONE - g.chance(i));
    }
    out
}

/// As [`evolve`], in floats (for the settling round and the scenes).
pub fn evolve_f64(v: &[f64; 3], g: &Game) -> [f64; 3] {
    let p = g.chances_f64();
    let mut out = [0.0; 3];
    for i in 0..3 {
        out[(i + 1) % 3] += v[i] * p[i];
        out[(i + 2) % 3] += v[i] * (1.0 - p[i]);
    }
    out
}

/// The first round from which the crowd started at corner 0 stays within
/// `within` players of `target` at every corner, through round `look`.
pub fn settles_from(g: &Game, n: f64, target: &[f64; 3], within: f64, look: usize) -> Option<usize> {
    let mut v = [n, 0.0, 0.0];
    let mut close = Vec::with_capacity(look);
    for _ in 0..look {
        v = evolve_f64(&v, g);
        close.push(v.iter().zip(target).all(|(x, t)| (x - t).abs() < within));
    }
    let last_far = close.iter().rposition(|&c| !c);
    match last_far {
        None => Some(1),
        Some(k) if k + 1 < look => Some(k + 2),
        Some(_) => None,
    }
}

/// A crowd of real players, not split by the chances: `start[i]` players
/// begin at corner i (listed corner by corner), and every round each
/// player, in that order, tosses a coin drawn from one SplitMix64 stream,
/// one corner forward on a win and one back on a loss. The counts at the
/// corners after each round (chapter 08's contrast with the split crowd).
pub fn real_crowd(g: &Game, start: [usize; 3], rounds: usize, seed: u64) -> Vec<[usize; 3]> {
    let mut rng = SplitMix64::new(seed);
    let mut corners: Vec<usize> = (0..3).flat_map(|i| std::iter::repeat_n(i, start[i])).collect();
    (0..rounds)
        .map(|_| {
            for c in corners.iter_mut() {
                let toss = rng.next_u64() % g.den as u64;
                *c = if toss < g.up[*c] as u64 {
                    (*c + 1) % 3
                } else {
                    (*c + 2) % 3
                };
            }
            let mut counts = [0usize; 3];
            for &c in &corners {
                counts[c] += 1;
            }
            counts
        })
        .collect()
}

/// The same real crowd as [`real_crowd`], from the same stream in the same
/// order, counted as crossings: for every round, how many players stepped
/// forward from each corner (`[0]`) and how many stepped back (`[1]`)
/// (chapter 14's real round against the split crowd's flows).
pub fn real_crossings(g: &Game, start: [usize; 3], rounds: usize, seed: u64) -> Vec<[[usize; 3]; 2]> {
    let mut rng = SplitMix64::new(seed);
    let mut corners: Vec<usize> = (0..3).flat_map(|i| std::iter::repeat_n(i, start[i])).collect();
    (0..rounds)
        .map(|_| {
            let mut moved = [[0usize; 3]; 2];
            for c in corners.iter_mut() {
                let toss = rng.next_u64() % g.den as u64;
                if toss < g.up[*c] as u64 {
                    moved[0][*c] += 1;
                    *c = (*c + 1) % 3;
                } else {
                    moved[1][*c] += 1;
                    *c = (*c + 2) % 3;
                }
            }
            moved
        })
        .collect()
}

// ----------------------------------------------------------------- the player

/// SplitMix64 (Steele, Lea and Flood, 2014): the video's one simulated
/// player draws every coin from it. A contract shared with `oracle.py`.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// The seed of the video's simulated player, fixed before any run was looked
/// at (the year of Harmer and Abbott's paper).
pub const SEED: u64 = 1999;
/// The player's starting dollars.
pub const START: i64 = 7;
/// The length of each of the player's runs.
pub const RUN_ROUNDS: usize = 100_000;

/// Which game the player plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Play {
    A,
    B,
    Mix,
}

/// One simulated player's run: the dollars after every round (the start
/// first), and how many rounds began at a multiple of three.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub dollars: Vec<i64>,
    pub at_zero: usize,
}

impl Run {
    /// Dollars won or lost over the first `rounds` rounds.
    pub fn net_after(&self, rounds: usize) -> i64 {
        self.dollars[rounds] - self.dollars[0]
    }

    pub fn net(&self) -> i64 {
        self.net_after(self.dollars.len() - 1)
    }

    pub fn low(&self) -> i64 {
        *self.dollars.iter().min().expect("a run has a start")
    }

    pub fn high(&self) -> i64 {
        *self.dollars.iter().max().expect("a run has a start")
    }

    /// The share of rounds that began at a multiple of three.
    pub fn share_at_zero(&self) -> Ratio {
        Ratio::new(self.at_zero as i128, (self.dollars.len() - 1) as i128)
    }
}

/// The player's run of `play` for `rounds` rounds from `start` dollars. The
/// mix draws its deciding coin first (even: A), then every round draws its
/// toss, which wins when the draw mod den is below the chance in denths.
pub fn run(play: Play, coins: &Coins, rounds: usize, seed: u64, start: i64) -> Run {
    let mut rng = SplitMix64::new(seed);
    let (a, b) = (coins.game_a(), coins.game_b());
    let mut dollars = Vec::with_capacity(rounds + 1);
    dollars.push(start);
    let (mut now, mut at_zero) = (start, 0);
    for _ in 0..rounds {
        let game = match play {
            Play::A => a,
            Play::B => b,
            Play::Mix => {
                if rng.next_u64().is_multiple_of(2) {
                    a
                } else {
                    b
                }
            }
        };
        let r = now.rem_euclid(MODULUS) as usize;
        at_zero += usize::from(r == 0);
        let toss = rng.next_u64() % game.den as u64;
        now += if toss < game.up[r] as u64 { 1 } else { -1 };
        dollars.push(now);
    }
    Run { dollars, at_zero }
}

/// The video's three runs: seed [`SEED`], [`START`] dollars,
/// [`RUN_ROUNDS`] rounds, published coins.
pub fn player(play: Play) -> Run {
    run(play, &PUBLISHED, RUN_ROUNDS, SEED, START)
}

// ---------------------------------------------------------------- finite play

/// The shares of a crowd started at `start` dollars that are ahead, behind
/// and level after `rounds` rounds of `g` (an exact distribution, in
/// floats).
pub fn horizon(g: &Game, rounds: usize, start: i64) -> (f64, f64, f64) {
    let p = g.chances_f64();
    // index i holds the dollars start − rounds + i
    let lowest = start - rounds as i64;
    let mut dist = vec![0.0f64; 2 * rounds + 1];
    dist[rounds] = 1.0;
    for step in 0..rounds {
        let mut next = vec![0.0f64; dist.len()];
        for i in (rounds - step)..=(rounds + step) {
            let w = dist[i];
            if w == 0.0 {
                continue;
            }
            let r = (lowest + i as i64).rem_euclid(MODULUS) as usize;
            next[i + 1] += w * p[r];
            next[i - 1] += w * (1.0 - p[r]);
        }
        dist = next;
    }
    let ahead = dist[rounds + 1..].iter().sum();
    let behind = dist[..rounds].iter().sum();
    (ahead, behind, dist[rounds])
}

/// The forward minus backward product of the mix with game A on a share
/// `s` of the rounds, in floats: its sign is the mix's verdict.
pub fn mix_verdict_f64(coins: &Coins, s: f64) -> f64 {
    let (a, b) = (coins.game_a().chances_f64(), coins.game_b().chances_f64());
    let p: Vec<f64> = (0..3).map(|i| s * a[i] + (1.0 - s) * b[i]).collect();
    p[0] * p[1] * p[2] - (1.0 - p[0]) * (1.0 - p[1]) * (1.0 - p[2])
}

/// Where the verdict changes sign between `lo` and `hi` (bisection).
pub fn crossing(coins: &Coins, mut lo: f64, mut hi: f64) -> f64 {
    let up = mix_verdict_f64(coins, lo) > 0.0;
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if (mix_verdict_f64(coins, mid) > 0.0) == up {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The exact distribution of the dollars won or lost by a crowd that
/// starts at `start` dollars and plays `g`, after each of `rounds`
/// (increasing), in floats: entry k of each is the share of the crowd at
/// net k − last, last the longest of `rounds` (chapter 17's bars).
pub fn snapshots(g: &Game, start: i64, rounds: &[usize]) -> Vec<Vec<f64>> {
    assert!(rounds.windows(2).all(|w| w[0] < w[1]), "the rounds increase");
    let p = g.chances_f64();
    let last = *rounds.last().expect("at least one number of rounds");
    let mut dist = vec![0.0f64; 2 * last + 1];
    dist[last] = 1.0;
    let mut wanted = rounds.iter().peekable();
    let mut out = Vec::with_capacity(rounds.len());
    for step in 0..=last {
        if wanted.next_if(|&&n| n == step).is_some() {
            out.push(dist.clone());
        }
        if step == last {
            break;
        }
        let mut next = vec![0.0f64; dist.len()];
        for i in (last - step)..=(last + step) {
            let w = dist[i];
            if w == 0.0 {
                continue;
            }
            let r = (start + i as i64 - last as i64).rem_euclid(MODULUS) as usize;
            next[i + 1] += w * p[r];
            next[i - 1] += w * (1.0 - p[r]);
        }
        dist = next;
    }
    out
}

/// The shares of a distribution from [`snapshots`] ahead of the start,
/// behind it and level with it.
pub fn categories(dist: &[f64], last: usize) -> (f64, f64, f64) {
    (dist[last + 1..].iter().sum(), dist[..last].iter().sum(), dist[last])
}

/// The mean and the standard deviation of a distribution from
/// [`snapshots`], in dollars won or lost.
pub fn moments(dist: &[f64], last: usize) -> (f64, f64) {
    let at = |k: usize| k as f64 - last as f64;
    let mean: f64 = dist.iter().enumerate().map(|(k, w)| w * at(k)).sum();
    let var: f64 = dist.iter().enumerate().map(|(k, w)| w * (at(k) - mean).powi(2)).sum();
    (mean, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(n: i128, d: i128) -> Ratio {
        Ratio::new(n, d)
    }

    #[test]
    fn ratios_reduce_round_and_print_as_the_oracle_does() {
        assert_eq!(r(6, -8).to_string(), "-3/4");
        assert_eq!(r(4, 2).to_string(), "2");
        assert_eq!(r(585, 100).fixed(1), "5.9", "half away from zero, not to even");
        assert_eq!(r(-585, 100).fixed(1), "-5.9");
        assert_eq!(r(-1, 1000).fixed(2), "0.00");
        assert_eq!(r(-1, 100).cents(2), "-1.00");
        assert_eq!(r(223, 14200).cents(2), "+1.57");
        assert_eq!(r(99, 200).exact_pct().unwrap(), "49.5");
        assert_eq!(r(1, 2).exact_pct().unwrap(), "50");
        assert!(r(1, 3).exact_pct().is_err());
        assert_eq!(r(32401, 84463).pct(1), "38.4");
        assert!(r(1, 3) < r(5, 13) && r(-1, 2) < Ratio::ZERO);
        assert_eq!(r(3, 4) * r(4, 9), r(1, 3));
        assert_eq!(r(1, 6) + r(1, 3), r(1, 2));
        assert_eq!(r(1, 2) / r(1, 4), Ratio::int(2));
    }

    /// The spanning-tree shares are stationary: one round of the game leaves
    /// them where they are, on every game of a grid.
    #[test]
    fn tree_shares_are_left_in_place_by_a_round() {
        for p0 in [1, 20, 99, 100, 150, 199] {
            for p1 in [1, 60, 100, 149] {
                for p2 in [19, 100, 125, 199] {
                    let g = Game {
                        up: [p0, p1, p2],
                        den: DEN,
                    };
                    let s = g.shares();
                    assert_eq!(evolve(&s, &g), s, "{g:?}");
                    assert_eq!(s.iter().fold(Ratio::ZERO, |a, &x| a + x), Ratio::ONE);
                }
            }
        }
    }

    /// The mean by the net flow equals the mean summed corner by corner,
    /// Σ share · (2p − 1), and the flow is the same on all three edges.
    #[test]
    fn the_mean_is_three_times_the_net_flow() {
        for p0 in (10..200).step_by(30) {
            for p1 in (5..200).step_by(45) {
                for p2 in (15..200).step_by(40) {
                    let g = Game {
                        up: [p0, p1, p2],
                        den: DEN,
                    };
                    let s = g.shares();
                    let summed = (0..3).fold(Ratio::ZERO, |a, i| {
                        a + s[i] * (Ratio::int(2) * g.chance(i) - Ratio::ONE)
                    });
                    assert_eq!(g.mean(), summed, "{g:?}");
                    let edge = |i: usize| s[i] * g.chance(i) - s[(i + 1) % 3] * (Ratio::ONE - g.chance((i + 1) % 3));
                    assert_eq!(edge(0), edge(1));
                    assert_eq!(edge(1), edge(2));
                    assert_eq!(Ratio::int(3) * edge(0), g.mean());
                    let (f, b) = g.products();
                    assert_eq!(g.mean().signum(), (f - b).signum());
                }
            }
        }
    }

    #[test]
    fn the_published_and_round_games() {
        assert_eq!(ROUND.game_b().shares(), [r(5, 13), r(2, 13), r(6, 13)]);
        assert_eq!(ROUND.game_b().mean(), Ratio::ZERO);
        assert_eq!(ROUND.mix().mean(), r(18, 709), "Ethier & Lee 2009");
        assert_eq!(
            ROUND.mix(),
            Game {
                up: [60, 125, 125],
                den: DEN
            }
        );
        assert_eq!(PUBLISHED.game_a().mean(), r(-1, 100));
        assert_eq!(PUBLISHED.game_b().mean(), r(-73443, 8446300));
        assert_eq!(PUBLISHED.mix().mean(), r(223, 14200));
        assert_eq!(PUBLISHED.mix().shares()[0], r(49, 142));
        assert_eq!(ROUND.die_chosen_win(), r(8, 15));
    }

    /// Ethier & Lee (2009) at ε = 0, per round played; the shares return
    /// after a period; with the handicap AB loses and AABB wins.
    #[test]
    fn periodic_schedules_match_ethier_and_lee() {
        for (pattern, want) in [
            ("AB", r(0, 1)),
            ("AABB", r(4, 163)),
            ("ABB", r(2416, 35601)),
            ("AAB", r(32, 1609)),
        ] {
            let (got, back) = periodic(&schedule(pattern, &ROUND));
            assert_eq!(got, want, "{pattern}");
            assert!(back, "{pattern}: the shares do not return");
        }
        let (ab, _) = periodic(&schedule("AB", &PUBLISHED));
        let (aabb, _) = periodic(&schedule("AABB", &PUBLISHED));
        assert_eq!(ab, r(-14568381, 2162265100));
        assert_eq!(aabb, r(232467650483270373, 15867555287549271700));
        // a one-game schedule is the game itself
        assert_eq!(periodic(&schedule("B", &PUBLISHED)).0, PUBLISHED.game_b().mean());
    }

    /// The smallest whole crowd by lcm agrees with trying every size.
    #[test]
    fn the_smallest_whole_crowds() {
        for (g, want) in [(ROUND.game_b(), 26), (ROUND.mix(), 1418)] {
            assert_eq!(smallest_whole_crowd(&g), want);
            let search = (1..=want)
                .find(|&n| {
                    let (f, b) = flows(&g, n);
                    f.iter().chain(&b).all(|x| x.is_integer())
                })
                .unwrap();
            assert_eq!(search, want);
        }
        let (f, b) = flows(&ROUND.game_b(), 26);
        assert_eq!(f.map(|x| x.to_string()), ["1", "3", "9"]);
        assert_eq!(b.map(|x| x.to_string()), ["9", "1", "3"]);
    }

    /// SplitMix64's published first output for seed 0, and the player's
    /// first rounds as the oracle printed them (2026-10-24).
    #[test]
    fn the_generator_and_the_player() {
        let mut g = SplitMix64::new(0);
        assert_eq!(g.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(g.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        let mut g = SplitMix64::new(SEED);
        assert_eq!(g.next_u64(), 12_531_769_384_086_060_215);
        let b = run(Play::B, &PUBLISHED, 12, SEED, START);
        assert_eq!(b.dollars, [7, 8, 9, 8, 9, 10, 11, 12, 11, 12, 11, 12, 13]);
        assert_eq!(b.at_zero, 5);
        let mix = run(Play::Mix, &PUBLISHED, 12, SEED, START);
        assert_eq!(mix.dollars, [7, 8, 9, 10, 9, 8, 9, 8, 9, 10, 11, 12, 11]);
        assert!(b.dollars.windows(2).all(|w| (w[0] - w[1]).abs() == 1));
    }

    /// The horizon's three shares add to one; a fair coin started anywhere
    /// is symmetric; with the round coins game B started at 0 is not.
    #[test]
    fn horizons_are_distributions() {
        let (a, b, l) = horizon(&PUBLISHED.mix(), 100, 0);
        assert!((a + b + l - 1.0).abs() < 1e-12);
        let fair = Game { up: [100; 3], den: DEN };
        let (a, b, _) = horizon(&fair, 51, 7);
        assert!((a - b).abs() < 1e-12);
        assert!(horizon(&fair, 10, 0).2 > 0.0);
        assert_eq!(
            horizon(&fair, 11, 0).2,
            0.0,
            "after an odd number of rounds nobody is level"
        );
    }

    #[test]
    fn the_mix_share_crossings_bracket_the_winning_range() {
        let lo = crossing(&PUBLISHED, 0.0, 0.5);
        let hi = crossing(&PUBLISHED, 0.5, 1.0);
        assert!((0.07..0.071).contains(&lo), "{lo}");
        assert!((0.847..0.848).contains(&hi), "{hi}");
        assert!(PUBLISHED.mix_share(1, 20).mean() < Ratio::ZERO);
        assert!(PUBLISHED.mix_share(9, 10).mean() < Ratio::ZERO);
        assert!(PUBLISHED.mix_share(1, 2).mean() > Ratio::ZERO);
    }

    /// Real players keep the crowd's size but not its counts: the first
    /// rounds as the oracle printed them (2026-10-24), started at 10, 4, 12.
    #[test]
    fn a_real_crowd_jumps_while_the_split_one_stays() {
        let real = real_crowd(&ROUND.game_b(), [10, 4, 12], 3, SEED);
        assert_eq!(real, [[9, 6, 11], [7, 5, 14], [9, 6, 11]]);
        let split = [10, 4, 12].map(Ratio::int);
        assert_eq!(evolve(&split, &ROUND.game_b()), split);
        assert!(
            real_crowd(&ROUND.game_b(), [10, 4, 12], 200, SEED)
                .iter()
                .all(|c| c.iter().sum::<usize>() == 26)
        );
    }

    #[test]
    fn the_crowd_settles_from_round_24() {
        assert_eq!(
            settles_from(&ROUND.game_b(), 26.0, &[10.0, 4.0, 12.0], 0.5, 200),
            Some(24)
        );
        // a crowd that never comes close is reported as never settling
        assert_eq!(settles_from(&ROUND.game_b(), 26.0, &[0.0, 0.0, 26.0], 0.5, 200), None);
    }
}
