//! Deterministic randomness. Seeds are derived per event from stable names
//! (FNV-1a over the bytes, never std's `DefaultHasher`, whose output is not
//! stable across Rust versions), so editing chapter 3 never changes chapter 1.

/// 64-bit FNV-1a over a byte string.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The seed for one event: `fnv1a64("video/chapter/track/event")`.
pub fn seed_for(video: &str, chapter: &str, track: &str, event: u64) -> u64 {
    fnv1a64(format!("{video}/{chapter}/{track}/{event}").as_bytes())
}

/// SplitMix64: used to expand one seed into the xoshiro state.
#[derive(Clone, Debug)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// xoshiro256** (Blackman & Vigna): fast, well-distributed, reproducible.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut sm = SplitMix64::new(seed);
        Self {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1) with 53 bits of precision.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }

    /// Standard normal (Box–Muller; one value per call, the pair's twin discarded
    /// so the stream position is a simple function of the call count).
    pub fn normal(&mut self) -> f64 {
        let u1 = self.uniform().max(f64::MIN_POSITIVE);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    /// Symmetric jitter in [-amount, +amount].
    pub fn jitter(&mut self, amount: f64) -> f64 {
        self.range(-amount, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a64_known_answers() {
        // Published FNV-1a 64-bit test vectors.
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn splitmix_known_answers() {
        // Reference outputs of SplitMix64 seeded with 1234567.
        let mut s = SplitMix64::new(1_234_567);
        assert_eq!(s.next_u64(), 6_457_827_717_110_365_317);
        assert_eq!(s.next_u64(), 3_203_168_211_198_807_973);
    }

    #[test]
    fn rng_is_deterministic_and_seed_sensitive() {
        let a: Vec<u64> = (0..8)
            .map({
                let mut r = Rng::new(42);
                move |_| r.next_u64()
            })
            .collect();
        let b: Vec<u64> = (0..8)
            .map({
                let mut r = Rng::new(42);
                move |_| r.next_u64()
            })
            .collect();
        let c: Vec<u64> = (0..8)
            .map({
                let mut r = Rng::new(43);
                move |_| r.next_u64()
            })
            .collect();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn uniform_and_normal_moments() {
        let mut r = Rng::new(7);
        let n = 200_000;
        let u: Vec<f64> = (0..n).map(|_| r.uniform()).collect();
        assert!(u.iter().all(|&x| (0.0..1.0).contains(&x)));
        let mean = u.iter().sum::<f64>() / n as f64;
        assert!((mean - 0.5).abs() < 0.005, "{mean}");
        let z: Vec<f64> = (0..n).map(|_| r.normal()).collect();
        let zm = z.iter().sum::<f64>() / n as f64;
        let zv = z.iter().map(|x| (x - zm) * (x - zm)).sum::<f64>() / n as f64;
        assert!(zm.abs() < 0.01 && (zv - 1.0).abs() < 0.02, "{zm} {zv}");
    }

    #[test]
    fn event_seeds_differ_by_name() {
        assert_ne!(seed_for("a", "01", "bed", 0), seed_for("a", "02", "bed", 0));
        assert_ne!(seed_for("a", "01", "bed", 0), seed_for("a", "01", "bed", 1));
        assert_eq!(seed_for("a", "01", "bed", 3), seed_for("a", "01", "bed", 3));
    }
}
