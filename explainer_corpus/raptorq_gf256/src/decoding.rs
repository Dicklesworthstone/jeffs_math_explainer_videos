//! How cards get solved, as chapters 13–16 show it, computed from the same
//! data and the same random draws as `oracle.py`:
//!
//! - [`PyRandom`]: CPython's `random.Random` (MT19937 seeded by
//!   `init_by_array`), so the oracle's seeded runs replay here bit for bit;
//! - the 13_6 race (B-F6.race_K10_seeded): random cards until ten pieces are
//!   pinned down, ex-or against bytes;
//! - the 14_6 peeling runs (B-F18): a plain sparse code with RFC 6330's
//!   degrees, peeled until it stalls;
//! - the five-piece toy of 14_3 and 16 (B-F20): its peel, and the same peel
//!   with C parked as an unknown;
//! - RFC 6330's degree table (Table 1), the rows of Table 2 the video names,
//!   and the LDPC rows of the ten-piece grid (15_2).

use crate::model::{A, B, Bucket, C, D, Dd, E, FOLD, inv, mul, result};

// ------------------------------------------------------------- CPython's RNG

/// CPython's `random.Random(seed)` for a seed below 2³²: MT19937, seeded
/// through `init_by_array` with the one-word key `[seed]`, as CPython seeds
/// an `int`. Only what the oracle's runs call: `getrandbits(k ≤ 32)` and
/// `randrange(n)`.
pub struct PyRandom {
    mt: [u32; 624],
    next: usize,
}

impl PyRandom {
    pub fn new(seed: u32) -> Self {
        let mut mt = [0u32; 624];
        mt[0] = 19_650_218;
        for i in 1..624 {
            mt[i] = 1_812_433_253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        // init_by_array with a one-word key: 624 mixing steps, then 623
        let mut i = 1;
        for _ in 0..624 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_664_525)).wrapping_add(seed);
            i += 1;
            if i == 624 {
                mt[0] = mt[623];
                i = 1;
            }
        }
        for _ in 0..623 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_566_083_941)).wrapping_sub(i as u32);
            i += 1;
            if i == 624 {
                mt[0] = mt[623];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        Self { mt, next: 624 }
    }

    /// One tempered 32-bit output (`genrand_uint32`).
    pub fn genrand(&mut self) -> u32 {
        if self.next == 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff);
                self.mt[k] = self.mt[(k + 397) % 624] ^ (y >> 1) ^ if y & 1 == 1 { 0x9908_b0df } else { 0 };
            }
            self.next = 0;
        }
        let mut y = self.mt[self.next];
        self.next += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// `getrandbits(k)` for 1 ≤ k ≤ 32: the top `k` bits of one output.
    pub fn getrandbits(&mut self, k: u32) -> u32 {
        assert!((1..=32).contains(&k), "getrandbits({k})");
        self.genrand() >> (32 - k)
    }

    /// `randrange(n)`: `getrandbits(n.bit_length())`, redrawn until below `n`.
    pub fn randrange(&mut self, n: u32) -> u32 {
        assert!(n > 0, "randrange(0)");
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.getrandbits(k);
            if r < n {
                return r;
            }
        }
    }
}

// ------------------------------------------------------------- the race, 13_6

/// The race's seed, pieces and races per side (the oracle's `race_k10`).
pub const RACE_SEED: u32 = 1306;
pub const RACE_PIECES: usize = 10;
pub const RACES: usize = 1000;

/// One race: for each card drawn, whether it was useful.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceRun {
    pub useful: Vec<bool>,
}

impl RaceRun {
    /// Cards beyond one per piece.
    pub fn spares(&self) -> usize {
        self.useful.len() - RACE_PIECES
    }
}

/// Both sides of the race, in the oracle's order (every ex-or race first,
/// then every byte race, from one stream).
pub struct Race {
    pub ex_or: Vec<RaceRun>,
    pub byte: Vec<RaceRun>,
}

/// How many races needed 0, 1, 2, … spares (up to the largest).
pub fn histogram(runs: &[RaceRun]) -> Vec<usize> {
    let top = runs.iter().map(RaceRun::spares).max().unwrap_or(0);
    let mut h = vec![0; top + 1];
    for r in runs {
        h[r.spares()] += 1;
    }
    h
}

/// The mean spares of one side, as the oracle prints it (an exact fraction
/// of the race count, rounded once).
pub fn mean_spares(runs: &[RaceRun]) -> f64 {
    runs.iter().map(RaceRun::spares).sum::<usize>() as f64 / runs.len() as f64
}

/// The race: uniform random cards (the zero card included) until the cards
/// pin down all ten pieces. Ex-or cards go into a [`Bucket`]; byte cards are
/// reduced at their pivots in the order they arrived and stored scaled to 1
/// there (the oracle's elimination, step for step).
pub fn race() -> Race {
    let mut rng = PyRandom::new(RACE_SEED);
    let ex_or = (0..RACES)
        .map(|_| {
            let mut bucket = Bucket::new();
            let mut useful = Vec::new();
            while bucket.len() < RACE_PIECES {
                useful.push(bucket.add(rng.getrandbits(RACE_PIECES as u32)));
            }
            RaceRun { useful }
        })
        .collect();
    let byte = (0..RACES)
        .map(|_| {
            let mut rows: Vec<(usize, [u8; RACE_PIECES])> = Vec::new();
            let mut useful = Vec::new();
            while rows.len() < RACE_PIECES {
                let mut v = [0u8; RACE_PIECES];
                for x in &mut v {
                    *x = rng.randrange(256) as u8;
                }
                for (c, r) in &rows {
                    let m = v[*c];
                    if m != 0 {
                        for (x, y) in v.iter_mut().zip(r) {
                            *x ^= mul(m, *y, FOLD);
                        }
                    }
                }
                match v.iter().position(|&x| x != 0) {
                    Some(c) => {
                        let s = inv(v[c]);
                        rows.push((c, v.map(|x| mul(s, x, FOLD))));
                        useful.push(true);
                    }
                    None => useful.push(false),
                }
            }
            RaceRun { useful }
        })
        .collect();
    Race { ex_or, byte }
}

/// The exact mean spares with `k` pieces over GF(2^bits): Σ 1/(q^i − 1),
/// i = 1…k (a uniform card is useful with chance 1 − q^(i−k) holding i
/// useful cards), in double-double, rounded once.
pub fn exact_mean_spares(bits: u32, k: u32) -> f64 {
    (1..=k)
        .fold(Dd::ZERO, |acc, i| acc.add(inv_pow2_minus_1(bits * i)))
        .to_f64()
}

/// 1/(2^e − 1) in double-double: exact division below 2⁵³, else the series
/// 2^−e + 2^−2e (the next term is below 2^−3e, far past a double-double).
fn inv_pow2_minus_1(e: u32) -> Dd {
    if e <= 52 {
        Dd::recip(((1u64 << e) - 1) as f64)
    } else {
        Dd::new(2f64.powi(-(e as i32))).add(Dd::new(2f64.powi(-2 * e as i32)))
    }
}

/// The sample spread check the oracle prints: each side's mean is within
/// four standard errors of the exact mean.
pub fn race_means_within_4se(race: &Race) -> bool {
    [(1u32, &race.ex_or), (8, &race.byte)].iter().all(|&(bits, runs)| {
        let n = runs.len() as f64;
        let mean = mean_spares(runs);
        let var = runs.iter().map(|r| (r.spares() as f64 - mean).powi(2)).sum::<f64>() / (n - 1.0);
        (mean - exact_mean_spares(bits, RACE_PIECES as u32)).abs() <= 4.0 * (var / n).sqrt()
    })
}

/// Erdős proved Σ 1/(2^i − 1) irrational in 1948 (J. Indian Math. Soc. 12,
/// 63–66; fact sheet, open item 1), the sum 13_6's lower-third names.
pub const ERDOS_YEAR: u32 = 1948;

// ------------------------------------------------- one amount at the last slot, 13

/// 13_1's card: random byte amounts (1–255) for the pieces A–E, drawn by
/// CPython's `random.Random(1301).randrange(1, 256)`.
pub const CARD_SEED: u32 = 1301;

pub fn random_byte_card() -> [u8; 5] {
    let mut rng = PyRandom::new(CARD_SEED);
    [0u8; 5].map(|_| 1 + rng.randrange(255) as u8)
}

/// At two pieces the lit part at the last slot is a line through 0 of
/// GF(256)² (one of 257). For every line and every piece whose own card is
/// off it, fix the other amount: exactly one of the 256 values of this
/// amount puts the card on the line. Returns (lines checked, all held).
pub fn one_amount_lights_exhaustive() -> (usize, bool) {
    let lines: Vec<(u8, u8)> = (0..=255u8).map(|t| (1, t)).chain([(0, 1)]).collect();
    let mut ok = true;
    for &(u0, u1) in &lines {
        let mut lit = vec![[false; 256]; 256];
        for s in 0..=255u8 {
            lit[mul(s, u0, FOLD) as usize][mul(s, u1, FOLD) as usize] = true;
        }
        if u0 != 0 {
            // piece 2's own card (0, 1) is off the line: fix c1, vary c2
            ok &= lit.iter().all(|row| row.iter().filter(|&&x| x).count() == 1);
        }
        if u1 != 0 {
            // piece 1's own card (1, 0) is off the line: fix c2, vary c1
            ok &= (0..256).all(|c2| lit.iter().filter(|row| row[c2]).count() == 1);
        }
    }
    (lines.len(), ok)
}

/// 13_4's instance: the bucket holds A ⊕ t·B (the line through (1, t)); the
/// new card's A amount is `c1`. The values of its B amount that land in the
/// lit part (exactly one: c1·t).
pub fn lit_values(t: u8, c1: u8) -> Vec<u8> {
    (0..=255u8)
        .filter(|&c2| (0..=255u8).any(|s| s == c1 && mul(s, t, FOLD) == c2))
        .collect()
}

// ------------------------------------------------------ dense solving, 14_1

/// 14_1's file of pieces, and the steps a second it is solved at.
pub const DENSE_PIECES: u64 = 50_000;
pub const STEPS_PER_SECOND: u64 = 1_000_000_000;

/// Gaussian elimination's ≈ K³/3 multiply-adds for `k` dense cards, rounded
/// once (K³ is exact in a double for these sizes).
pub fn dense_steps(k: u64) -> f64 {
    (k * k * k) as f64 / 3.0
}

/// How long those steps take at `per_second`, in `unit` seconds (3,600 for
/// hours), rounded once.
pub fn dense_time(k: u64, per_second: u64, unit: u64) -> f64 {
    (k * k * k) as f64 / (3 * per_second * unit) as f64
}

// ------------------------------------------------- RFC 6330's tables, 14, 15

/// RFC 6330 §5.3.5.2, Table 1: the degree generator's thresholds f[d] (in
/// units of 2⁻²⁰); degree d has mass f[d] − f[d − 1].
pub const DEGREE_TABLE: [u32; 31] = [
    0, 5243, 529_531, 704_294, 791_675, 844_104, 879_057, 904_023, 922_747, 937_311, 948_962, 958_494, 966_438,
    973_160, 978_921, 983_914, 988_283, 992_138, 995_565, 998_631, 1_001_391, 1_003_887, 1_006_157, 1_008_229,
    1_010_129, 1_011_876, 1_013_490, 1_014_983, 1_016_370, 1_017_662, 1_048_576,
];

/// Mass of degree `d` (1 ≤ d ≤ 30), exactly, as a fraction of 2²⁰.
pub fn degree_mass(d: usize) -> f64 {
    f64::from(DEGREE_TABLE[d] - DEGREE_TABLE[d - 1]) / f64::from(1u32 << 20)
}

/// The mean degree Σ d·P(d), exact (a dyadic fraction).
pub fn mean_degree() -> f64 {
    let sum: u64 = (1..=30)
        .map(|d| d as u64 * u64::from(DEGREE_TABLE[d] - DEGREE_TABLE[d - 1]))
        .sum();
    sum as f64 / f64::from(1u32 << 20)
}

/// Pieces per packet including the PI symbols: the mean degree plus
/// E[d1] = 2 + P(d < 4)/2 (RFC 6330 §5.3.5.4, Rand[X, 3, 2] taken as a fair
/// coin), exact.
pub fn pieces_per_packet() -> f64 {
    mean_degree() + 2.0 + f64::from(DEGREE_TABLE[3]) / f64::from(1u32 << 21)
}

/// One row of RFC 6330 §5.6, Table 2 that the video names: K′, S, H, W.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub k: usize,
    pub s: usize,
    pub h: usize,
    pub w: usize,
}

impl Block {
    /// Every unknown: the K′ pieces plus the S + H extra pieces.
    pub fn l(&self) -> usize {
        self.k + self.s + self.h
    }
    /// The PI ("permanently inactivated") symbols: L − W.
    pub fn p(&self) -> usize {
        self.l() - self.w
    }
    pub fn extra(&self) -> usize {
        self.s + self.h
    }
    /// The free cards and packets that are ex-or cards fill all but H slots.
    pub fn ex_or_slots(&self) -> usize {
        self.l() - self.h
    }
}

/// Ten pieces (15_2), the big block (15_3, 15_6, 16_1) and the largest.
pub const TEN: Block = Block {
    k: 10,
    s: 7,
    h: 10,
    w: 17,
};
pub const BIG: Block = Block {
    k: 10_017,
    s: 241,
    h: 11,
    w: 10_111,
};
pub const LARGEST: Block = Block {
    k: 56_403,
    s: 907,
    h: 16,
    w: 56_951,
};
/// The fewest and most byte cards (H) over all 477 rows of Table 2.
pub const H_RANGE: (usize, usize) = (10, 16);

/// The columns of LDPC row `i` (RFC 6330 §5.3.3.3): each of the B = W − S
/// pieces C[0..B] goes into three rows, stepping by a = 1 + ⌊j/S⌋; the row's
/// own LDPC symbol C[B + i]; and two PI symbols C[W + i mod P],
/// C[W + (i + 1) mod P]. Sorted.
pub fn ldpc_row(block: Block, i: usize) -> Vec<usize> {
    let (s, b, p) = (block.s, block.w - block.s, block.p());
    let mut cols = Vec::new();
    for j in 0..b {
        let a = 1 + j / s;
        let mut r = j % s;
        for _ in 0..3 {
            if r == i {
                cols.push(j);
            }
            r = (r + a) % s;
        }
    }
    cols.push(b + i);
    cols.push(block.w + i % p);
    cols.push(block.w + (i + 1) % p);
    cols.sort_unstable();
    cols
}

// ------------------------------------------------- RaptorQ's shape, 15

/// A packet's mix in the standard's shape: its degree d from Table 1
/// (`randrange(2²⁰)`, bisected), and d1 PI symbols, 2 + a fair coin when
/// d < 4, else 2 (RFC 6330 §5.3.5.4, Rand[X, 3, 2] taken as a coin).
fn packet_shape(rng: &mut PyRandom) -> (usize, usize) {
    let r = rng.randrange(1 << 20);
    let d = DEGREE_TABLE.partition_point(|&f| f <= r);
    let d1 = 2 + if d < 4 { rng.randrange(2) as usize } else { 0 };
    (d, d1)
}

/// How many pieces each of `n` packets mixes (d + d1), drawn from `seed`:
/// 15_1's cards.
pub fn packet_sizes(seed: u32, n: usize) -> Vec<usize> {
    let mut rng = PyRandom::new(seed);
    (0..n)
        .map(|_| {
            let (d, d1) = packet_shape(&mut rng);
            d + d1
        })
        .collect()
}

/// The first seed from `from` whose `n` packet rows over `block` (d of the
/// W LT columns, d1 of the P PI columns, distinct) leave some original piece
/// (a column below K′) in no packet: 15_2's schematic grid. Returns the
/// seed, the rows (sorted columns) and that piece.
pub fn schematic_packets(block: Block, n: usize, from: u32) -> (u32, Vec<Vec<usize>>, usize) {
    for seed in from.. {
        let mut rng = PyRandom::new(seed);
        let rows: Vec<Vec<usize>> = (0..n)
            .map(|_| {
                let (d, d1) = packet_shape(&mut rng);
                let mut cols = Vec::new();
                while cols.len() < d.min(block.w) {
                    let c = rng.randrange(block.w as u32) as usize;
                    if !cols.contains(&c) {
                        cols.push(c);
                    }
                }
                let mut pi = Vec::new();
                while pi.len() < d1.min(block.p()) {
                    let c = block.w + rng.randrange(block.p() as u32) as usize;
                    if !pi.contains(&c) {
                        pi.push(c);
                    }
                }
                cols.extend(pi);
                cols.sort_unstable();
                cols
            })
            .collect();
        if let Some(c) = (0..block.k).find(|c| rows.iter().all(|r| !r.contains(c))) {
            return (seed, rows, c);
        }
    }
    unreachable!("a seed leaves a piece out")
}

/// 15_5's ablation, RECORDED (fact sheet B-F21; `rq_ablate.py` on our RFC
/// 6330 constraint matrix at ten pieces, exactly enough packets): failures
/// and trials with the standard's byte rows, and with those rows replaced by
/// random ex-or rows. Not re-run here: the harness needs RFC 6330's random
/// tables, which this repository does not carry.
pub const ABLATION_BYTES: (u32, u32) = (646, 100_000);
pub const ABLATION_EX_OR: (u32, u32) = (7_066, 10_000);

// ------------------------------------------------ the measured decoder, 17

/// RECORDED (fact sheet B-F22; `rq_mc.py` with PyPI `raptorq` 2.0.0, the Rust
/// decoder): failures and trials at ten pieces with exactly enough packets,
/// one extra and two extra; and (pieces, failures, trials) with exactly
/// enough at larger blocks. Not re-run here (the K = 10 run alone took
/// 869 s with that package).
pub const MEASURED_K10: [(u32, u32); 3] = [(6_316, 1_000_000), (24, 1_000_000), (1, 1_000_000)];
pub const MEASURED_LARGER: [(u32, u32, u32); 2] = [(101, 1_262, 200_000), (1_002, 108, 20_000)];

/// RFC 6330 §5.8 (SOURCED, fact sheet B-F23): a compliant decoder fails at
/// most one time in this many with exactly enough packets, one extra, two
/// extra (averaged over random packet IDs, for each padded block size K′).
pub const RFC_ONE_IN: [u32; 3] = [100, 10_000, 1_000_000];

/// RECORDED (fact sheet B-F22, `rq_rust/src/bin/rq_stream.rs` on the
/// raptorq 2.0.0 crate): the hook's stream at 1,002 pieces, where the
/// sender sends the originals in order and then repair packets, each phone
/// loses each packet with probability 1 in 100 and keeps the first K + h it
/// receives: failures and phones for h = 0, 1, 2 (the same phones for each h).
pub const STREAM_PIECES: u32 = 1_002;
pub const MEASURED_STREAM: [(u32, u32); 3] = [(21_186, 5_000_000), (98, 5_000_000), (3, 5_000_000)];

/// RECORDED (`rq_rust/src/bin/rq_cost.rs`, one core of an Apple M5 Ultra):
/// one decode of a 50,000-piece block from K + 2 packets, the median of 5
/// runs in ms, for 1% of the originals lost then replaced by repair packets
/// and for repair packets only, in two sets of runs each: with 8-byte pieces
/// and with 1,280-byte pieces.
pub const DECODE_PIECES: u32 = 50_000;
pub const DECODE_MS_8_BYTES: [f64; 4] = [112.868, 95.967, 126.986, 98.253];
pub const DECODE_MS_1280_BYTES: [f64; 4] = [294.388, 224.698, 315.406, 246.471];

/// SOURCED years on 17_6's cards and 17's credits: LT codes (RFC 6330
/// §9.2, [LTCodes], November 2002; fact sheet B-S11), RFC 6330 itself
/// (August 2011; B-S1) and the ATSC 3.0 edition read (A/331:2021; B-S1).
pub const LT_CODES_YEAR: u32 = 2002;
pub const RFC_6330_YEAR: u32 = 2011;
pub const ATSC_A331_YEAR: u32 = 2021;

/// The exact (Clopper–Pearson) two-sided 95% interval for `x` failures in
/// `n` trials, by bisection on the binomial tails (each term summed in log
/// space); 0 is the lower end when x = 0.
pub fn clopper_pearson(x: u32, n: u32) -> (f64, f64) {
    // P(X ≤ k) for X ~ Binomial(n, p)
    let cdf = |k: u32, p: f64| -> f64 {
        if p <= 0.0 {
            return 1.0;
        }
        if p >= 1.0 {
            return if k < n { 0.0 } else { 1.0 };
        }
        let (lp, lq) = (p.ln(), (-p).ln_1p());
        let mut log_c = 0.0; // log C(n, j)
        let mut total = 0.0;
        for j in 0..=k {
            if j > 0 {
                log_c += f64::from(n - j + 1).ln() - f64::from(j).ln();
            }
            total += (log_c + f64::from(j) * lp + f64::from(n - j) * lq).exp();
        }
        total
    };
    let solve = |f: &dyn Fn(f64) -> bool| {
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for _ in 0..100 {
            let mid = (lo + hi) / 2.0;
            if f(mid) { hi = mid } else { lo = mid }
        }
        (lo + hi) / 2.0
    };
    let lower = if x == 0 {
        0.0
    } else {
        solve(&|p| 1.0 - cdf(x - 1, p) >= 0.025)
    };
    let upper = solve(&|p| cdf(x, p) <= 0.025);
    (lower, upper)
}

// ------------------------------------------------------ the peeling runs, 14_6

/// The runs' seed and size (the oracle's `bf18`, from `lt_peel_sim.py`).
pub const PEEL_SEED: u32 = 42;
pub const PEEL_PIECES: usize = 10_000;
/// Extra cards, percent of the pieces, in the oracle's order.
pub const PEEL_EXTRA_PERCENT: [usize; 3] = [0, 2, 5];
pub const PEEL_RUNS: usize = 5;

/// One run of the plain sparse code.
#[derive(Clone, Debug)]
pub struct PeelRun {
    pub cards: usize,
    /// The pieces in the order peeling recovered them.
    pub order: Vec<u32>,
    /// The pieces no card mixed.
    pub untouched: Vec<u32>,
}

impl PeelRun {
    pub fn recovered(&self) -> usize {
        self.order.len()
    }
}

/// Every run, `[extra][run]`, from one stream: each card draws a degree from
/// Table 1 (`randrange(2²⁰)`, bisected), then that many distinct pieces
/// (`random.sample`'s set branch, which these sizes always take); peeling
/// pops the newest single-unknown card.
pub fn peel_runs() -> Vec<Vec<PeelRun>> {
    let mut rng = PyRandom::new(PEEL_SEED);
    let k = PEEL_PIECES;
    PEEL_EXTRA_PERCENT
        .iter()
        .map(|&pct| {
            (0..PEEL_RUNS)
                .map(|_| {
                    let n = k + k * pct / 100;
                    let mut cards: Vec<Vec<u32>> = Vec::with_capacity(n);
                    let mut uses: Vec<Vec<usize>> = vec![Vec::new(); k];
                    for j in 0..n {
                        let r = rng.randrange(1 << 20);
                        let d = DEGREE_TABLE.partition_point(|&f| f <= r);
                        let mut s: Vec<u32> = Vec::with_capacity(d);
                        for _ in 0..d {
                            let mut piece = rng.randrange(k as u32);
                            while s.contains(&piece) {
                                piece = rng.randrange(k as u32);
                            }
                            s.push(piece);
                            uses[piece as usize].push(j);
                        }
                        cards.push(s);
                    }
                    let untouched = (0..k as u32).filter(|&i| uses[i as usize].is_empty()).collect();
                    let mut known = vec![false; k];
                    let mut ripple: Vec<usize> = (0..n).filter(|&j| cards[j].len() == 1).collect();
                    let mut order = Vec::new();
                    while let Some(j) = ripple.pop() {
                        if cards[j].len() != 1 {
                            continue;
                        }
                        let i = cards[j][0] as usize;
                        if known[i] {
                            continue;
                        }
                        known[i] = true;
                        order.push(i as u32);
                        for &other in &uses[i] {
                            if let Some(at) = cards[other].iter().position(|&x| x as usize == i) {
                                cards[other].swap_remove(at);
                                if cards[other].len() == 1 {
                                    ripple.push(other);
                                }
                            }
                        }
                    }
                    PeelRun {
                        cards: n,
                        order,
                        untouched,
                    }
                })
                .collect()
        })
        .collect()
}

/// A plain sparse code with 2% extra cards (N = 10,200 of K = 10,000): on
/// average how many pieces no card mixes. Each card of degree d misses a
/// given piece with chance 1 − d/K, so a piece is in no card with chance
/// (1 − mean/K)^N (B-F17; model, not a run).
pub fn expected_untouched() -> f64 {
    let k = PEEL_PIECES as f64;
    let n = PEEL_PIECES + PEEL_PIECES * PEEL_EXTRA_PERCENT[1] / 100;
    k * (1.0 - mean_degree() / k).powf(n as f64)
}

// --------------------------------------------------- the five-piece toy, 14, 16

/// The toy's six cards y1–y6 (B-F20): C alone, then 03's A ⊕ C, B ⊕ C ⊕ D,
/// A ⊕ B, then D ⊕ E and B ⊕ E. Bit i of a card is piece i (`model`).
pub const TOY_CARDS: [u32; 6] = [C, A | C, B | C | D, A | B, D | E, B | E];

/// A value known up to the parked piece: `constant ⊕ (c if with_c)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Affine {
    pub constant: u8,
    pub with_c: bool,
}

impl Affine {
    fn xor(self, o: Self) -> Self {
        Self {
            constant: self.constant ^ o.constant,
            with_c: self.with_c ^ o.with_c,
        }
    }
}

/// One peel: card `card` (an index into [`TOY_CARDS`]) had one unknown left,
/// `piece`, now worth `value`; `ripple` cards had one unknown left when it
/// was chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeelStep {
    pub card: usize,
    pub piece: usize,
    pub value: Affine,
    pub ripple: usize,
}

/// A peel of the toy's cards `cards` (indices into [`TOY_CARDS`]), with the
/// piece `parked` carried along as the unknown c (16 parks C). As the
/// oracle's `bf20`: take the lowest-numbered card with one unknown left,
/// until none is left. Returns the steps and the cards left over with no
/// unknown but c (16_3's leftover card).
pub fn peel_toy(cards: &[usize], parked: Option<usize>) -> (Vec<PeelStep>, Vec<usize>) {
    let parked_bit = parked.map_or(0, |p| 1u32 << p);
    let mut left: Vec<(usize, u32, Affine)> = cards
        .iter()
        .map(|&c| {
            let card = TOY_CARDS[c];
            // the parked piece moves to the result's side: result ⊕ c
            let value = Affine {
                constant: result(card),
                with_c: card & parked_bit != 0,
            };
            (c, card & !parked_bit, value)
        })
        .collect();
    let mut steps = Vec::new();
    loop {
        let ripple: Vec<usize> = (0..left.len()).filter(|&i| left[i].1.count_ones() == 1).collect();
        let Some(&at) = ripple.iter().min_by_key(|&&i| left[i].0) else {
            break;
        };
        let (card, unknown, value) = left.remove(at);
        let piece = unknown.trailing_zeros() as usize;
        steps.push(PeelStep {
            card,
            piece,
            value,
            ripple: ripple.len(),
        });
        for (_, u, v) in &mut left {
            if *u & unknown != 0 {
                *u &= !unknown;
                *v = v.xor(value);
            }
        }
    }
    let leftover = left
        .iter()
        .filter(|(_, u, v)| *u == 0 && v.with_c)
        .map(|(c, _, _)| *c)
        .collect();
    (steps, leftover)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PIECE_NAMES, PIECES};

    /// The first outputs of CPython's `random.Random(42)`
    /// (`[r.getrandbits(32) for _ in range(3)]`, CPython 3.14.6).
    #[test]
    fn mt19937_matches_cpython_on_its_first_outputs() {
        let mut r = PyRandom::new(42);
        let first: Vec<u32> = (0..3).map(|_| r.getrandbits(32)).collect();
        assert_eq!(first, [2_746_317_213, 478_163_327, 107_420_369]);
    }

    #[test]
    fn the_race_replays_the_oracles_counts() {
        let race = race();
        let join = |h: Vec<usize>| h.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
        assert_eq!(
            join(histogram(&race.ex_or)),
            "297,303,171,94,65,39,17,4,7,0,2,0,0,0,0,1"
        );
        assert_eq!(join(histogram(&race.byte)), "995,5");
        // every race ends with its tenth useful card
        for r in race.ex_or.iter().chain(&race.byte) {
            assert_eq!(r.useful.iter().filter(|&&u| u).count(), RACE_PIECES);
            assert_eq!(r.useful.last(), Some(&true));
        }
    }

    #[test]
    fn the_peeling_runs_replay_the_oracles_fractions() {
        let runs = peel_runs();
        let got: Vec<Vec<usize>> = runs
            .iter()
            .map(|r| r.iter().map(PeelRun::recovered).collect())
            .collect();
        assert_eq!(
            got,
            [
                vec![924, 4210, 2249, 1735, 825],
                vec![9824, 9871, 9853, 9884, 1578],
                vec![9902, 9877, 9877, 9865, 9903]
            ]
        );
        // an untouched piece is never recovered; every recovered piece is distinct
        for r in runs.iter().flatten() {
            let mut seen = vec![false; PEEL_PIECES];
            for &i in &r.order {
                assert!(!seen[i as usize]);
                seen[i as usize] = true;
            }
            assert!(r.untouched.iter().all(|&i| !seen[i as usize]));
        }
    }

    #[test]
    fn the_toy_peels_c_a_b_d_e_and_stalls_without_its_first_card() {
        let (steps, leftover) = peel_toy(&[0, 1, 2, 3, 4, 5], None);
        let order: String = steps.iter().map(|s| PIECE_NAMES[s.piece]).collect();
        assert_eq!(order, "CABDE");
        assert_eq!(steps.iter().map(|s| s.ripple).collect::<Vec<_>>(), [1, 1, 1, 2, 2]);
        for s in &steps {
            assert_eq!(s.value.constant, PIECES[s.piece]);
        }
        assert!(leftover.is_empty());
        let (stuck, _) = peel_toy(&[1, 2, 3, 4, 5], None);
        assert!(stuck.is_empty(), "nothing moves");
    }

    /// 16_2–16_3, as spoken: A is 2 ⊕ c, B is 6 ⊕ A, D's c's cancel to 2, E
    /// follows from D, and the leftover card B ⊕ E gives c = 7.
    #[test]
    fn parking_c_gives_the_spoken_steps() {
        let (steps, leftover) = peel_toy(&[1, 2, 3, 4, 5], Some(2));
        let by_piece = |p: usize| steps.iter().find(|s| s.piece == p).expect("peeled").value;
        let order: String = steps.iter().map(|s| PIECE_NAMES[s.piece]).collect();
        assert_eq!(order, "ABDE");
        assert_eq!(
            by_piece(0),
            Affine {
                constant: 2,
                with_c: true
            }
        );
        assert_eq!(
            by_piece(1),
            Affine {
                constant: 6 ^ 2,
                with_c: true
            }
        );
        assert_eq!(
            by_piece(3),
            Affine {
                constant: 2,
                with_c: false
            }
        );
        assert_eq!(
            by_piece(4),
            Affine {
                constant: 14 ^ 2,
                with_c: false
            }
        );
        assert_eq!(leftover, [5]);
        // the leftover card: B ⊕ E = 15, so c = 15 ⊕ B's and E's constants
        let c = result(TOY_CARDS[5]) ^ by_piece(1).constant ^ by_piece(4).constant;
        assert_eq!(c, PIECES[2]);
        assert_eq!(
            result(TOY_CARDS[5]) ^ result(TOY_CARDS[4]) ^ result(TOY_CARDS[2]),
            c,
            "c = 15 ⊕ 14 ⊕ 6"
        );
    }

    #[test]
    fn ldpc_rows_put_every_piece_in_three_rows_and_cover_every_column() {
        for block in [TEN, BIG] {
            let rows: Vec<Vec<usize>> = (0..block.s).map(|i| ldpc_row(block, i)).collect();
            for j in 0..block.w - block.s {
                assert_eq!(rows.iter().filter(|r| r.contains(&j)).count(), 3, "piece {j}");
            }
            for r in &rows {
                assert!(r.windows(2).all(|w| w[0] < w[1]), "distinct columns");
            }
        }
        // ten pieces: 7 free ex-or rows over 27 columns
        assert_eq!((TEN.l(), TEN.extra(), TEN.p()), (27, 17, 10));
    }

    #[test]
    fn exact_means_and_the_one_amount_rule() {
        // ten pieces: 1 + 1/3 + 1/7 + … + 1/1023, and the bytes' 1/255 + …
        assert!((exact_mean_spares(1, 10) - 1.605_718_271_890_746).abs() < 1e-15);
        assert!((exact_mean_spares(8, 10) - 0.003_936_887_487_739_71).abs() < 1e-17);
        assert!(race_means_within_4se(&race()));
        assert_eq!(one_amount_lights_exhaustive(), (257, true));
        for (t, c1) in [(3u8, 7u8), (1, 1), (200, 99)] {
            assert_eq!(lit_values(t, c1), [mul(c1, t, FOLD)]);
        }
        assert!(random_byte_card().iter().all(|&a| a >= 1));
    }

    #[test]
    fn the_schematic_grid_leaves_a_piece_out_and_the_ldpc_rows_hold_it() {
        let (seed, rows, out) = schematic_packets(TEN, TEN.k, 1502);
        assert!(seed >= 1502 && out < TEN.k);
        assert_eq!(rows.len(), TEN.k);
        assert!(rows.iter().all(|r| r.iter().all(|&c| c < TEN.l())));
        // the piece no packet mixed is in some free ex-or card
        assert!((0..TEN.s).any(|i| ldpc_row(TEN, i).contains(&out)));
        let sizes = packet_sizes(1501, 400);
        let mean = sizes.iter().sum::<usize>() as f64 / sizes.len() as f64;
        assert!((mean - pieces_per_packet()).abs() < 0.8, "{mean}");
    }

    #[test]
    fn clopper_pearson_agrees_with_the_fact_sheet() {
        // fact sheet B-F22: 0.616–0.647%; 1.54–3.57e-5; 2.53e-8–5.57e-6
        let (lo, hi) = clopper_pearson(6316, 1_000_000);
        assert!(
            (lo - 0.006_161_659).abs() < 1e-8 && (hi - 0.006_473_211).abs() < 1e-8,
            "{lo} {hi}"
        );
        let (lo, hi) = clopper_pearson(24, 1_000_000);
        assert!(
            (lo - 1.537_731e-5).abs() < 1e-10 && (hi - 3.570_989e-5).abs() < 1e-10,
            "{lo} {hi}"
        );
        let (lo, hi) = clopper_pearson(1, 1_000_000);
        assert!(
            (lo - 2.531_781e-8).abs() < 1e-13 && (hi - 5.571_631e-6).abs() < 1e-11,
            "{lo} {hi}"
        );
    }

    #[test]
    fn table_1_masses_sum_to_one_and_half_the_cards_mix_two() {
        let total: f64 = (1..=30).map(degree_mass).sum();
        assert!((total - 1.0).abs() < 1e-12);
        assert_eq!(degree_mass(2), 0.5);
    }
}
