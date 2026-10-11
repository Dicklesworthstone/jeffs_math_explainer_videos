//! The part of numpy's `Generator(PCG64)` the oracle's seeded runs use,
//! draw for draw, so the scene computes the very experiments `oracle.py`
//! runs (chapter 09's panel, verify2.py's replay, the reviewers): PCG64
//! (XSL-RR) seeded through numpy's `SeedSequence`, `random`, the buffered
//! 32-bit draws behind bounded integers (Lemire), `choice` without
//! replacement (Floyd's algorithm or a tail shuffle, then a shuffle) and
//! `hypergeometric` (Stadlober's HRUA). Each follows numpy's C source in
//! its order of operations; the tests and `./x capture_recapture
//! facts-check` hold it to the oracle's outputs.
//!
//! numpy keeps the spare half of a 64-bit draw for the next 32-bit draw
//! across `random()` calls (`pcg64_next_double` never touches it). That is
//! why this is written here rather than taken from FrankenNumPy's
//! `fnp-random`, which drops the spare on every `random()` and so leaves
//! numpy's stream wherever bounded integers and floats interleave.

use fmn::core::rng::SeedSequence;

/// PCG's 128-bit LCG multiplier.
const MULTIPLIER: u128 = 0x2360_ed05_1fc6_5da4_4385_df64_9fcc_f645;

/// log(k!) for k = 0..125, numpy's `logfactorial.c` table verbatim (its
/// HRUA sampler reads these; computing them would differ in the last bits).
#[allow(clippy::excessive_precision, clippy::approx_constant)]
const LOGFACT: [f64; 126] = [
    0.0,
    0.0,
    0.69314718055994529,
    1.791759469228055,
    3.1780538303479458,
    4.7874917427820458,
    6.5792512120101012,
    8.5251613610654147,
    10.604602902745251,
    12.801827480081469,
    15.104412573075516,
    17.502307845873887,
    19.987214495661885,
    22.552163853123425,
    25.19122118273868,
    27.89927138384089,
    30.671860106080672,
    33.505073450136891,
    36.395445208033053,
    39.339884187199495,
    42.335616460753485,
    45.380138898476908,
    48.471181351835227,
    51.606675567764377,
    54.784729398112319,
    58.003605222980518,
    61.261701761002001,
    64.557538627006338,
    67.88974313718154,
    71.257038967168015,
    74.658236348830158,
    78.092223553315307,
    81.557959456115043,
    85.054467017581516,
    88.580827542197682,
    92.136175603687093,
    95.719694542143202,
    99.330612454787428,
    102.96819861451381,
    106.63176026064346,
    110.32063971475739,
    114.03421178146171,
    117.77188139974507,
    121.53308151543864,
    125.3172711493569,
    129.12393363912722,
    132.95257503561632,
    136.80272263732635,
    140.67392364823425,
    144.5657439463449,
    148.47776695177302,
    152.40959258449735,
    156.3608363030788,
    160.3311282166309,
    164.32011226319517,
    168.32744544842765,
    172.35279713916279,
    176.39584840699735,
    180.45629141754378,
    184.53382886144948,
    188.6281734236716,
    192.7390472878449,
    196.86618167289001,
    201.00931639928152,
    205.1681994826412,
    209.34258675253685,
    213.53224149456327,
    217.73693411395422,
    221.95644181913033,
    226.1905483237276,
    230.43904356577696,
    234.70172344281826,
    238.97838956183432,
    243.26884900298271,
    247.57291409618688,
    251.89040220972319,
    256.22113555000954,
    260.56494097186322,
    264.92164979855278,
    269.29109765101981,
    273.67312428569369,
    278.06757344036612,
    282.4742926876304,
    286.89313329542699,
    291.32395009427029,
    295.76660135076065,
    300.22094864701415,
    304.68685676566872,
    309.1641935801469,
    313.65282994987905,
    318.1526396202093,
    322.66349912672615,
    327.1852877037752,
    331.71788719692847,
    336.26118197919845,
    340.81505887079902,
    345.37940706226686,
    349.95411804077025,
    354.53908551944079,
    359.1342053695754,
    363.73937555556347,
    368.35449607240474,
    372.97946888568902,
    377.61419787391867,
    382.25858877306001,
    386.91254912321756,
    391.57598821732961,
    396.24881705179155,
    400.93094827891576,
    405.6222961611449,
    410.32277652693733,
    415.03230672824964,
    419.75080559954472,
    424.47819341825709,
    429.21439186665157,
    433.95932399501481,
    438.71291418612117,
    443.47508812091894,
    448.24577274538461,
    453.02489623849613,
    457.81238798127816,
    462.60817852687489,
    467.4121995716082,
    472.22438392698058,
    477.04466549258564,
    481.87297922988796,
];

/// log(k!): the table, then the Stirling series to the 1/k³ term
/// (`logfactorial.c`).
fn logfactorial(k: i64) -> f64 {
    if let Ok(i) = usize::try_from(k)
        && i < LOGFACT.len()
    {
        return LOGFACT[i];
    }
    let k = k as f64;
    let half_ln_2pi = 0.918_938_533_204_672_8;
    (k + 0.5) * k.ln() - k + (half_ln_2pi + (1.0 / k) * (1.0 / 12.0 - 1.0 / (360.0 * k * k)))
}

/// `numpy.random.Generator(PCG64(seed))`, the methods the oracle calls.
#[derive(Clone, Debug)]
pub struct Generator {
    state: u128,
    inc: u128,
    /// The high half of the last 64-bit draw, kept for the next 32-bit draw.
    spare: Option<u32>,
}

impl Generator {
    /// `numpy.random.default_rng(seed)` for an integer seed.
    pub fn from_seed(seed: u64) -> Self {
        Self::from_sequence(&SeedSequence::from_seed(seed))
    }

    /// `numpy.random.default_rng([w0, w1, …])` for a list of 32-bit words.
    pub fn from_words(words: &[u32]) -> Self {
        Self::from_sequence(&SeedSequence::from_entropy_words(words, &[]))
    }

    /// PCG64's seeding (`pcg_setseq_128_srandom_r`) from four 64-bit words.
    fn from_sequence(seq: &SeedSequence) -> Self {
        let w = seq.generate_state_u64(4);
        let initstate = (u128::from(w[0]) << 64) | u128::from(w[1]);
        let initseq = (u128::from(w[2]) << 64) | u128::from(w[3]);
        let inc = (initseq << 1) | 1;
        let mut g = Self {
            state: 0,
            inc,
            spare: None,
        };
        g.step();
        g.state = g.state.wrapping_add(initstate);
        g.step();
        g
    }

    fn step(&mut self) {
        self.state = self.state.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
    }

    /// One raw draw: step, then the XSL-RR output.
    pub fn next_u64(&mut self) -> u64 {
        self.step();
        let s = self.state;
        ((s >> 64) as u64 ^ s as u64).rotate_right((s >> 122) as u32)
    }

    /// `pcg64_next32`: the low half of a fresh draw, its high half kept.
    fn next_u32(&mut self) -> u32 {
        if let Some(v) = self.spare.take() {
            return v;
        }
        let v = self.next_u64();
        self.spare = Some((v >> 32) as u32);
        v as u32
    }

    /// `Generator.random()`: 53 bits in [0, 1). Leaves the spare alone.
    pub fn random(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `Generator.random(n)`.
    pub fn random_n(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.random()).collect()
    }

    /// `random_bounded_uint64(0, rng, 0, false)`: uniform on 0..=rng, by
    /// Lemire's method on 32-bit draws (every range the oracle asks for
    /// fits in 32 bits).
    fn bounded(&mut self, rng: u64) -> u64 {
        if rng == 0 {
            return 0;
        }
        let rng = u32::try_from(rng).expect("the oracle's ranges fit in 32 bits");
        if rng == u32::MAX {
            return u64::from(self.next_u32());
        }
        let excl = u64::from(rng) + 1;
        let mut m = u64::from(self.next_u32()) * excl;
        let mut leftover = m as u32;
        if u64::from(leftover) < excl {
            let threshold = (u32::MAX - rng) % (rng + 1);
            while leftover < threshold {
                m = u64::from(self.next_u32()) * excl;
                leftover = m as u32;
            }
        }
        m >> 32
    }

    /// `_shuffle_int(n, first, data)`: swap each place from the last down
    /// to `first` with a bounded draw.
    fn shuffle_from(&mut self, data: &mut [u64], first: usize) {
        for i in (first..data.len()).rev() {
            let j = self.bounded(i as u64) as usize;
            data.swap(i, j);
        }
    }

    /// `Generator.choice(pop, size, replace=False)` (shuffle on): indices
    /// into 0..pop, in numpy's order.
    pub fn choice(&mut self, pop: usize, size: usize) -> Vec<usize> {
        assert!(size <= pop, "a sample larger than the population");
        let (pop64, size64) = (pop as u64, size as u64);
        if pop > 10_000 && size > pop / 50 {
            // tail shuffle: shuffle the last `size` places of 0..pop
            let mut idx: Vec<u64> = (0..pop64).collect();
            self.shuffle_from(&mut idx, (pop - size).max(1));
            return idx[pop - size..].iter().map(|&i| i as usize).collect();
        }
        // Floyd's algorithm with numpy's open-addressing set, then a shuffle
        let mask = {
            let mut m = (1.2 * size as f64) as u64;
            for s in [1, 2, 4, 8, 16, 32] {
                m |= m >> s;
            }
            m
        };
        let mut set = vec![u64::MAX; (mask + 1) as usize];
        let mut out = Vec::with_capacity(size);
        for j in pop64 - size64..pop64 {
            let val = self.bounded(j);
            let mut loc = (val & mask) as usize;
            while set[loc] != u64::MAX && set[loc] != val {
                loc = (loc + 1) & mask as usize;
            }
            if set[loc] == u64::MAX {
                set[loc] = val;
                out.push(val);
            } else {
                let mut loc = (j & mask) as usize;
                while set[loc] != u64::MAX {
                    loc = (loc + 1) & mask as usize;
                }
                set[loc] = j;
                out.push(j);
            }
        }
        self.shuffle_from(&mut out, 1);
        out.into_iter().map(|i| i as usize).collect()
    }

    /// `Generator.hypergeometric(good, bad, sample)` where numpy takes the
    /// ratio-of-uniforms path (10 ≤ sample ≤ good + bad − 10), the only one
    /// the oracle's runs reach.
    pub fn hypergeometric(&mut self, good: i64, bad: i64, sample: i64) -> i64 {
        // D1 = 2√(2/e), D2 = 3 − 2√(3/e)
        const D1: f64 = 1.715_527_769_921_413_5;
        const D2: f64 = 0.898_916_162_058_898_8;
        let popsize = good + bad;
        assert!(
            (10..=popsize - 10).contains(&sample),
            "numpy samples this size by counting, not HRUA"
        );
        let computed = sample.min(popsize - sample);
        let (min_gb, max_gb) = (good.min(bad), good.max(bad));
        let p = min_gb as f64 / popsize as f64;
        let q = max_gb as f64 / popsize as f64;
        let a = computed as f64 * p + 0.5;
        let var = (popsize - computed) as f64 * computed as f64 * p * q / (popsize - 1) as f64;
        let c = (var + 0.5).sqrt();
        let h = D1 * c + D2;
        let m = ((computed + 1) as f64 * (min_gb + 1) as f64 / (popsize + 2) as f64).floor() as i64;
        let g = logfactorial(m)
            + logfactorial(min_gb - m)
            + logfactorial(computed - m)
            + logfactorial(max_gb - computed + m);
        let b = (((computed.min(min_gb) + 1) as f64).min(a + 16.0 * c)).floor();
        let mut k;
        loop {
            let u = self.random();
            let v = self.random();
            let x = a + h * (v - 0.5) / u;
            if x < 0.0 || x >= b {
                continue;
            }
            k = x.floor() as i64;
            let gp = logfactorial(k)
                + logfactorial(min_gb - k)
                + logfactorial(computed - k)
                + logfactorial(max_gb - computed + k);
            let t = g - gp;
            if u * (4.0 - u) - 3.0 <= t {
                break;
            }
            if u * (u - t) >= 1.0 {
                continue;
            }
            if 2.0 * u.ln() <= t {
                break;
            }
        }
        if good > bad {
            k = computed - k;
        }
        if computed < sample {
            k = good - k;
        }
        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_factorial_table_is_log_k_factorial() {
        let mut sum = 0.0f64;
        for (k, &v) in LOGFACT.iter().enumerate() {
            if k > 1 {
                sum += (k as f64).ln();
            }
            assert!((v - sum).abs() <= 1e-12 * sum.max(1.0), "log({k}!)");
        }
        // the Stirling series takes over smoothly where the table stops
        let ln_126_factorial = LOGFACT[125] + 126f64.ln();
        assert!((logfactorial(126) - ln_126_factorial).abs() < 1e-12 * ln_126_factorial);
    }

    #[test]
    fn default_rng_matches_numpy() {
        // numpy 2.x: default_rng(20261009).random() and the raw stream are
        // checked through the oracle's panel (runs.rs); here, the first
        // bounded draws reproduce A-CH09.panel.first12_k's first netful
        let mut g = Generator::from_seed(20_261_009);
        let first = g.choice(10_000, 1_000);
        assert_eq!(first.len(), 1_000);
        let mut sorted = first.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 1_000, "without replacement");
        assert_eq!(first.iter().filter(|&&i| i < 1_000).count(), 101);
    }

    #[test]
    fn the_spare_half_survives_random() {
        // two 32-bit draws come from one 64-bit draw even with a float between
        let mut a = Generator::from_seed(7);
        let mut b = a.clone();
        let lo = a.next_u32();
        let _ = a.random();
        let hi = a.next_u32();
        let raw = b.next_u64();
        assert_eq!((lo, hi), (raw as u32, (raw >> 32) as u32));
    }
}
