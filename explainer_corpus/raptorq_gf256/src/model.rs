//! What Video B draws and computes: the byte field (RaptorQ's fold 0x11D and
//! AES's 0x11B), ranks over GF(2) and GF(256), the worked examples (pieces
//! A–E and their cards), the exact rank probabilities, and the hook's million
//! phones. `facts.rs` registers what the chapters show or say from here, and
//! `oracle.py` recomputes every value independently.
//!
//! The oracle computes probabilities as exact fractions (or at 50–120 decimal
//! digits) and rounds once to a double. Here they are computed in
//! double-double arithmetic (about 32 digits) and rounded once, so they print
//! the oracle's doubles exactly (facts-check compares the strings). Values
//! close to 1 carry their distance from 1 instead ([`Near1`]), so a tiny
//! failure chance keeps all its digits.

// ------------------------------------------------------------ the byte field

/// RaptorQ's fold polynomial x⁸ + x⁴ + x³ + x² + 1 (0x11D), low byte.
pub const FOLD: u8 = 0x1D;
/// AES's polynomial x⁸ + x⁴ + x³ + x + 1 (0x11B), low byte.
pub const AES_FOLD: u8 = 0x1B;

/// Multiply by x (the byte 2): shift, and fold the bit that falls off.
pub fn xtime(a: u8, fold: u8) -> u8 {
    (a << 1) ^ if a & 0x80 != 0 { fold } else { 0 }
}

/// Byte multiplication, shift-and-add (the video's code panel).
pub fn mul(mut a: u8, mut b: u8, fold: u8) -> u8 {
    let mut acc = 0;
    while b != 0 {
        if b & 1 != 0 {
            acc ^= a;
        }
        a = xtime(a, fold);
        b >>= 1;
    }
    acc
}

/// The powers of 2 under RaptorQ's fold: `exps()[k]` = 2ᵏ, k = 0…254.
pub fn exps() -> [u8; 255] {
    let mut exp = [0u8; 255];
    let mut a = 1u8;
    for e in &mut exp {
        *e = a;
        a = xtime(a, FOLD);
    }
    exp
}

/// Discrete logs base 2 under RaptorQ's fold: `log[a]` for every nonzero byte.
pub fn logs() -> [u8; 256] {
    let mut log = [0u8; 256];
    for (k, &a) in exps().iter().enumerate() {
        log[a as usize] = k as u8;
    }
    log
}

/// The partner of a nonzero byte under RaptorQ's fold (a · a⁻¹ = 1): it sits
/// at the mirror image on the byte clock, log a⁻¹ = 255 − log a.
///
/// # Panics
/// For 0, which has no partner.
pub fn inv(a: u8) -> u8 {
    assert!(a != 0, "0 has no partner");
    let k = logs()[a as usize] as usize;
    exps()[(255 - k) % 255]
}

/// The multiplicative order of `a` (the smallest k ≥ 1 with aᵏ = 1).
///
/// # Panics
/// If `a` is 0 (no power of 0 is 1).
pub fn order(a: u8, fold: u8) -> u32 {
    let mut x = a;
    let mut k = 1;
    while x != 1 {
        x = mul(x, a, fold);
        k += 1;
        assert!(k <= 255, "{a} is zero or not invertible");
    }
    k
}

// -------------------------------------------------------------------- ranks

/// A phone's bucket of ex-or cards (bit i of a card = piece i): it keeps a
/// card when the card is useful, so the number it holds is the rank over
/// GF(2), and everything it can make is the span of what it holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bucket {
    /// Kept cards, reduced: no two share a leading bit.
    basis: Vec<u32>,
}

impl Bucket {
    pub fn new() -> Self {
        Self::default()
    }

    /// What is left of `card` after ex-or-ing away everything the bucket can
    /// make (0: the bucket could make it already).
    fn reduce(&self, mut card: u32) -> u32 {
        for &b in &self.basis {
            let top = 31 - b.leading_zeros();
            if card >> top & 1 == 1 {
                card ^= b;
            }
        }
        card
    }

    /// Offer a card; `true` if it was useful (and is now held).
    pub fn add(&mut self, card: u32) -> bool {
        let r = self.reduce(card);
        if r == 0 {
            return false;
        }
        self.basis.push(r);
        self.basis.sort_unstable_by(|a, b| b.cmp(a));
        true
    }

    /// Useful cards held.
    pub fn len(&self) -> usize {
        self.basis.len()
    }

    pub fn is_empty(&self) -> bool {
        self.basis.is_empty()
    }

    /// Could the bucket make `card` from what it holds?
    pub fn can_make(&self, card: u32) -> bool {
        self.reduce(card) == 0
    }

    /// Every card the bucket can make (2^len of them, "nothing" included),
    /// ascending.
    pub fn span(&self) -> Vec<u32> {
        let mut out = vec![0u32];
        for &b in &self.basis {
            let more: Vec<u32> = out.iter().map(|&x| x ^ b).collect();
            out.extend(more);
        }
        out.sort_unstable();
        out
    }
}

/// The rank of ex-or cards over GF(2).
pub fn rank_gf2(cards: &[u32]) -> usize {
    let mut b = Bucket::new();
    cards.iter().filter(|&&c| b.add(c)).count()
}

/// The rank of byte cards over GF(256) under RaptorQ's fold (Gaussian
/// elimination; a 0/1 matrix gives its GF(2) rank).
pub fn rank_gf256(rows: &[Vec<u8>]) -> usize {
    let mut m: Vec<Vec<u8>> = rows.to_vec();
    let cols = m.first().map_or(0, Vec::len);
    let mut r = 0;
    for c in 0..cols {
        let Some(p) = (r..m.len()).find(|&i| m[i][c] != 0) else {
            continue;
        };
        m.swap(r, p);
        let s = inv(m[r][c]);
        m[r] = m[r].iter().map(|&v| mul(s, v, FOLD)).collect();
        for i in 0..m.len() {
            if i != r && m[i][c] != 0 {
                let f = m[i][c];
                let pivot = m[r].clone();
                for (v, w) in m[i].iter_mut().zip(pivot) {
                    *v ^= mul(f, w, FOLD);
                }
            }
        }
        r += 1;
        if r == m.len() {
            break;
        }
    }
    r
}

// --------------------------------------------------------- the worked example

/// The pieces' letters, A–E.
pub const PIECE_NAMES: [&str; 5] = ["A", "B", "C", "D", "E"];
/// The pieces' values: A = 5, B = 3, C = 7, D = 2, E = 12 (fact sheet B-F3,
/// B-F9).
pub const PIECES: [u8; 5] = [5, 3, 7, 2, 12];
/// One bit per piece in a card.
pub const A: u32 = 1;
pub const B: u32 = 1 << 1;
pub const C: u32 = 1 << 2;
pub const D: u32 = 1 << 3;
pub const E: u32 = 1 << 4;

/// Chapter 03's cards: A ⊕ C, B ⊕ C ⊕ D, A ⊕ B, A ⊕ B ⊕ C ⊕ D.
pub const CARDS: [u32; 4] = [A | C, B | C | D, A | B, A | B | C | D];
/// 03_6's fifth card, B ⊕ C: cards one and three already make it.
pub const FIFTH_CARD: u32 = B | C;
/// The ring of five (06_4, 16): each card the ex-or of two neighbours.
pub const RING: [u32; 5] = [A | B, B | C, C | D, D | E, E | A];

/// A card's result: the ex-or of the pieces it uses.
pub fn result(card: u32) -> u8 {
    (0..5).filter(|&i| card >> i & 1 == 1).fold(0, |acc, i| acc ^ PIECES[i])
}

/// The pieces a card uses, in order.
pub fn pieces_of(card: u32) -> Vec<usize> {
    (0..32).filter(|&i| card >> i & 1 == 1).collect()
}

// ------------------------------------------------- double-double arithmetic

/// A double-double number hi + lo, |lo| ≤ ulp(hi)/2: about 32 digits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dd {
    hi: f64,
    lo: f64,
}

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    (s, b - (s - a))
}

impl Dd {
    pub const ZERO: Dd = Dd { hi: 0.0, lo: 0.0 };
    pub const ONE: Dd = Dd { hi: 1.0, lo: 0.0 };

    pub fn new(x: f64) -> Self {
        Self { hi: x, lo: 0.0 }
    }

    fn norm(hi: f64, lo: f64) -> Self {
        let (hi, lo) = quick_two_sum(hi, lo);
        Self { hi, lo }
    }

    /// 1/n, for a nonzero integer n below 2⁵³.
    pub fn recip(n: f64) -> Self {
        let hi = 1.0 / n;
        // the residual 1 − n·hi is exact with a fused multiply-add
        let r = (-hi).mul_add(n, 1.0);
        Self::norm(hi, r / n)
    }

    pub fn add(self, o: Self) -> Self {
        let (s, e) = two_sum(self.hi, o.hi);
        let (t, f) = two_sum(self.lo, o.lo);
        let (s, e) = quick_two_sum(s, e + t);
        Self::norm(s, e + f)
    }

    pub fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    pub fn sub(self, o: Self) -> Self {
        self.add(o.neg())
    }

    pub fn mul(self, o: Self) -> Self {
        let p = self.hi * o.hi;
        let e = self.hi.mul_add(o.hi, -p);
        Self::norm(p, e + (self.hi * o.lo + self.lo * o.hi))
    }

    pub fn div(self, o: Self) -> Self {
        // long division: a quotient digit, the exact remainder, a correction
        let q1 = self.hi / o.hi;
        let r = self.sub(o.mul(Self::new(q1)));
        let q2 = r.hi / o.hi;
        let r = r.sub(o.mul(Self::new(q2)));
        let q3 = r.hi / o.hi;
        Self::norm(q1, q2).add(Self::new(q3))
    }

    /// The nearest double.
    pub fn to_f64(self) -> f64 {
        self.hi + self.lo
    }
}

/// A number near 1, held as its distance below 1 (1 − δ), so products of
/// such numbers keep δ's digits however small it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Near1 {
    pub delta: Dd,
}

impl Near1 {
    pub const ONE: Near1 = Near1 { delta: Dd::ZERO };

    /// 1 − d.
    pub fn below(d: Dd) -> Self {
        Self { delta: d }
    }

    /// (1 − a)(1 − b) = 1 − (a + b − ab).
    pub fn mul(self, o: Self) -> Self {
        Self {
            delta: self.delta.add(o.delta).sub(self.delta.mul(o.delta)),
        }
    }

    /// (1 − δ)ⁿ by squaring.
    pub fn pow(self, mut n: u64) -> Self {
        let (mut base, mut acc) = (self, Self::ONE);
        while n > 0 {
            if n & 1 == 1 {
                acc = acc.mul(base);
            }
            base = base.mul(base);
            n >>= 1;
        }
        acc
    }

    /// The value itself, 1 − δ (loses δ's digits when δ is tiny).
    pub fn value(self) -> Dd {
        Dd::ONE.sub(self.delta)
    }
}

fn dd_pow(mut base: Dd, mut n: u64) -> Dd {
    let mut acc = Dd::ONE;
    while n > 0 {
        if n & 1 == 1 {
            acc = acc.mul(base);
        }
        base = base.mul(base);
        n >>= 1;
    }
    acc
}

// ------------------------------------------------ the lottery's probabilities

/// q⁻ⁱ, exact: q is 2 or 256, and the exponents stay far inside a double's.
fn q_pow_neg(q: u32, i: u32) -> f64 {
    let bits = match q {
        2 => 1,
        256 => 8,
        _ => panic!("the video's fields are GF(2) and GF(256), not GF({q})"),
    };
    2f64.powi(-((bits * i) as i32))
}

/// P(k + h uniform random cards over GF(q) pin down k pieces) as 1 − δ:
/// ∏ (1 − q⁻ⁱ), i = h + 1 … h + k.
fn full_near1(q: u32, k: u32, h: u32) -> Near1 {
    (h + 1..=h + k).fold(Near1::ONE, |acc, i| acc.mul(Near1::below(Dd::new(q_pow_neg(q, i)))))
}

/// The oracle's truncation of the many-piece limit: i ≤ h + 150 (q = 2) or
/// h + 20 (q = 256); the omitted factor is within 2⁻¹⁴⁹ (256⁻¹⁹) of 1.
fn limit_terms(q: u32) -> u32 {
    if q == 2 { 150 } else { 20 }
}

/// The chance that k + h uniform random cards over GF(q) pin down k pieces.
pub fn p_full(q: u32, k: u32, h: u32) -> f64 {
    full_near1(q, k, h).value().to_f64()
}

/// The chance that they do not (the lottery fails).
pub fn p_fail(q: u32, k: u32, h: u32) -> f64 {
    full_near1(q, k, h).delta.to_f64()
}

/// [`p_full`] with many pieces (k → ∞).
pub fn p_full_limit(q: u32, h: u32) -> f64 {
    full_near1(q, limit_terms(q), h).value().to_f64()
}

/// [`p_fail`] with many pieces (k → ∞): ex-or's 71%, 42%, 23%, 12% at
/// h = 0…3 spares.
pub fn p_fail_limit(q: u32, h: u32) -> f64 {
    full_near1(q, limit_terms(q), h).delta.to_f64()
}

/// [`p_fail_limit`] times a count (phones stuck of a million), rounded once.
pub fn p_fail_limit_times(q: u32, h: u32, n: u32) -> f64 {
    full_near1(q, limit_terms(q), h)
        .delta
        .mul(Dd::new(f64::from(n)))
        .to_f64()
}

/// The fewest ex-or spares that bring the many-piece failure to at most
/// 1 in `one_in`.
pub fn spares_for(one_in: u32) -> u32 {
    let target = Dd::recip(f64::from(one_in));
    (0..)
        .find(|&h| full_near1(2, limit_terms(2), h).delta.sub(target).to_f64() <= 0.0)
        .expect("the failure halves with every spare")
}

/// The danger meter: holding `held` useful ex-or cards of `pieces`, the
/// chance that the next random card is useless (it lands in what the bucket
/// can already make): 2^held of the 2^pieces cards.
pub fn useless_chance(held: u32, pieces: u32) -> f64 {
    assert!(held <= pieces, "a bucket holds at most one useful card per piece");
    2f64.powi(held as i32 - pieces as i32)
}

// ------------------------------------------------------ the million phones

/// The picture's pieces (01_4: 10,000).
pub const HOOK_PIECES: u32 = 10_000;
/// The phones (01_1).
pub const PHONES: u64 = 1_000_000;
/// Each phone loses each packet independently with probability 1 in 100.
pub const LOSS_ONE_IN: u32 = 100;
/// Phone glyphs drawn (01_1): a sample, one glyph for every 1,000 phones.
pub const GLYPHS: usize = 1_000;

/// The sticker album (01_7–01_8): random pieces until all `k` are seen take
/// k·H_k on average (H_k = 1 + 1/2 + … + 1/k).
pub fn album_mean(k: u32) -> f64 {
    album_mean_dd(k).to_f64()
}

fn album_mean_dd(k: u32) -> Dd {
    // smallest terms first
    let h = (1..=k).rev().fold(Dd::ZERO, |acc, i| acc.add(Dd::recip(f64::from(i))));
    h.mul(Dd::new(f64::from(k)))
}

/// The album's average, whole draws (the count-up's last value).
pub fn album_mean_floor(k: u32) -> u64 {
    album_mean_dd(k).to_f64().floor() as u64
}

/// Album slots filled after `draws` random pieces, on average:
/// k(1 − (1 − 1/k)^draws). Drives the picture only.
pub fn album_filled(k: u32, draws: f64) -> f64 {
    let k = f64::from(k);
    k * -(draws * (-1.0 / k).ln_1p()).exp_m1()
}

fn loss() -> Dd {
    Dd::recip(f64::from(LOSS_ONE_IN))
}

/// P(one phone, or each of `copies` phones' pieces, all arrive within `m`
/// carousel loops) = (1 − p^m)^(pieces·copies), as 1 − δ.
fn done_by_near1(m: u32, copies: u64) -> Near1 {
    let pm = dd_pow(loss(), u64::from(m));
    Near1::below(pm).pow(u64::from(HOOK_PIECES) * copies)
}

/// P(all of `phones` phones hold every piece after `m` carousel loops).
pub fn p_done_by(m: u32, phones: u64) -> f64 {
    if m == 0 {
        return 0.0;
    }
    let near = done_by_near1(m, phones);
    // far from 1 the plain power keeps the digits (2.2 × 10⁻⁴⁴ after one loop)
    let direct = dd_pow(
        Dd::ONE.sub(dd_pow(loss(), u64::from(m))),
        u64::from(HOOK_PIECES) * phones,
    );
    if direct.to_f64() < 0.5 {
        direct.to_f64()
    } else {
        near.value().to_f64()
    }
}

/// P(a phone still waits after `m` loops).
pub fn p_waiting_after(m: u32) -> f64 {
    done_by_near1(m, 1).delta.to_f64()
}

/// P(a phone finishes in loop 2 or 3): "Most phones need two or three loops".
pub fn p_done_in_loop_2_or_3() -> f64 {
    let p3 = done_by_near1(3, 1).value();
    let p1 = dd_pow(Dd::ONE.sub(loss()), u64::from(HOOK_PIECES));
    p3.sub(p1).to_f64()
}

/// A phone's average number of loops, E[T] = Σ P(T > m).
pub fn mean_loops() -> f64 {
    (1..40)
        .fold(Dd::ONE, |acc, m| acc.add(done_by_near1(m, 1).delta))
        .to_f64()
}

/// Packets sent until a carousel phone holds every piece, on average (the
/// oracle's closed form: E[N] = K + Σ_m (K − S_m), S_m = b(aᴷ − bᴷ)/(a − b),
/// a = 1 − p^(m+1), b = 1 − p^m). aᴷ and bᴷ are held as distances below 1,
/// so their difference keeps its digits however late the loop.
fn carousel_sent_dd() -> Dd {
    let k = Dd::new(f64::from(HOOK_PIECES));
    let n = u64::from(HOOK_PIECES);
    let p = loss();
    (1..31).fold(k, |acc, m| {
        let beta = dd_pow(p, m);
        let alpha = beta.mul(p);
        let ak = Near1::below(alpha).pow(n).delta;
        let bk = Near1::below(beta).pow(n).delta;
        // aᴷ − bᴷ = (1 − ak) − (1 − bk) = bk − ak; a − b = β − α
        let s = Dd::ONE.sub(beta).mul(bk.sub(ak)).div(beta.sub(alpha));
        acc.add(k.sub(s))
    })
}

/// Packets sent until a carousel phone is done, on average (23,730).
pub fn carousel_mean_sent() -> f64 {
    carousel_sent_dd().to_f64()
}

/// Packets caught by then, on average (23,493): 99% of those sent.
pub fn carousel_mean_caught() -> f64 {
    let keep = Dd::new(f64::from(LOSS_ONE_IN - 1)).div(Dd::new(f64::from(LOSS_ONE_IN)));
    carousel_sent_dd().mul(keep).to_f64()
}

/// SplitMix64 (Steele, Lea & Flood 2014; Vigna's constants): the portable
/// generator `oracle.py` mirrors, so the picture is the oracle's draw.
#[derive(Clone, Debug)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1): the top 53 bits.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
}

/// The hook's seed (the storyboard's date); never re-rolled for a nicer draw.
pub const FIELD_SEED: u64 = 20_261_011;

/// The loop in which a phone finishes, from one uniform draw by the inverse
/// distribution: the smallest m with u < P(done by m) (`phones` phones at
/// once: the last of them).
fn finishing_loop(u: f64, phones: u64) -> u32 {
    (1..)
        .find(|&m| u < p_done_by(m, phones))
        .expect("P(done by m) reaches 1")
}

/// The drawn field (01_1, 01_6): the loop each of the [`GLYPHS`] sample
/// phones finishes in, from the first draws of [`FIELD_SEED`].
pub fn phone_field() -> Vec<u32> {
    let mut rng = SplitMix64::new(FIELD_SEED);
    let thresholds: Vec<f64> = (1..=12).map(|m| p_done_by(m, 1)).collect();
    (0..GLYPHS)
        .map(|_| {
            let u = rng.uniform();
            thresholds.iter().position(|&t| u < t).map_or(13, |i| i as u32 + 1)
        })
        .collect()
}

/// Sample phones done by the end of each loop 1…6.
pub fn field_done_by_loop() -> [usize; 6] {
    let field = phone_field();
    std::array::from_fn(|m| field.iter().filter(|&&t| t as usize <= m + 1).count())
}

/// The loop the unluckiest of the million phones finishes in (01_6), drawn
/// right after the field from the distribution of the last of a million.
pub fn unluckiest_of_a_million() -> u32 {
    let mut rng = SplitMix64::new(FIELD_SEED);
    for _ in 0..GLYPHS {
        rng.next_u64();
    }
    finishing_loop(rng.uniform(), PHONES)
}

/// The toy deck (03_8): sender and phone both turn a packet's number into
/// the same mix of A–D (a nonempty subset), seeded by the number. RFC 6330's
/// own recipe (Tuple[K′, ISI]) is out of scope; this stands in for it.
pub fn deck_mix(packet: u32) -> u32 {
    let mut rng = SplitMix64::new(FIELD_SEED ^ u64::from(packet));
    1 + (rng.next_u64() % 15) as u32
}

/// The packet numbers 03_8 shows.
pub const DECK_PACKETS: [u32; 3] = [48_213, 48_214, 48_215];

/// Red loss flecks for the drawn phone `glyph` (01_1): which of `shown`
/// packets it lost, each with probability 1 in [`LOSS_ONE_IN`]. The picture
/// only; no number on screen comes from here.
pub fn loss_flecks(glyph: usize, shown: usize) -> Vec<usize> {
    let mut rng = SplitMix64::new(FIELD_SEED ^ 0x0F1E_C500 ^ glyph as u64);
    (0..shown)
        .filter(|_| rng.uniform() < 1.0 / f64::from(LOSS_ONE_IN))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carry-less multiply, then reduce by the full degree-8 polynomial: an
    /// independent route to the same products.
    fn clmul_reduce(a: u8, b: u8, poly: u16) -> u8 {
        let mut p = 0u16;
        for i in 0..8 {
            if b >> i & 1 != 0 {
                p ^= u16::from(a) << i;
            }
        }
        for bit in (8..16).rev() {
            if p >> bit & 1 != 0 {
                p ^= poly << (bit - 8);
            }
        }
        p as u8
    }

    #[test]
    fn the_panel_multiply_agrees_with_carryless_reduction_on_every_pair() {
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                assert_eq!(mul(a, b, FOLD), clmul_reduce(a, b, 0x11D), "{a} * {b}");
                assert_eq!(mul(a, b, AES_FOLD), clmul_reduce(a, b, 0x11B), "{a} * {b} (AES)");
            }
        }
    }

    #[test]
    fn multiplying_is_adding_clicks_and_every_byte_has_a_partner() {
        let (log, exp) = (logs(), exps());
        for a in 1..=255u8 {
            for b in 1..=255u8 {
                let clicks = (u16::from(log[a as usize]) + u16::from(log[b as usize])) % 255;
                assert_eq!(mul(a, b, FOLD), exp[clicks as usize]);
            }
            assert_eq!(mul(a, inv(a), FOLD), 1, "the partner of {a}");
        }
        // the oracle's spot values (B-F11, B-F13)
        assert_eq!((inv(2), inv(3)), (142, 244));
        assert_eq!((log[3], log[29], log[128], log[150]), (25, 8, 7, 180));
        assert_eq!(&exp[..13], &[1, 2, 4, 8, 16, 32, 64, 128, 29, 58, 116, 232, 205]);
    }

    #[test]
    #[should_panic(expected = "no partner")]
    fn zero_has_no_partner() {
        let _ = inv(0);
    }

    #[test]
    fn the_cards_of_chapter_03_are_the_fact_sheets() {
        // B-F3: 2, 6, 6, 3; rank 4; the fifth card B ⊕ C = 4 is redundant
        let results: Vec<u8> = CARDS.iter().map(|&c| result(c)).collect();
        assert_eq!(results, [2, 6, 6, 3]);
        assert_eq!(rank_gf2(&CARDS), 4);
        assert_eq!(result(FIFTH_CARD), 4);
        assert_eq!(result(CARDS[0]) ^ result(CARDS[2]), result(FIFTH_CARD));
        let mut bucket = Bucket::new();
        assert!(CARDS.iter().all(|&c| bucket.add(c)), "every card of 03 is useful");
        assert!(!bucket.add(FIFTH_CARD), "03_7: the fifth card is not");
        assert_eq!(pieces_of(CARDS[1]), [1, 2, 3]);
    }

    #[test]
    fn the_ring_of_five_is_worth_four_over_both_fields() {
        // B-F9: rank 4 over GF(2) and over GF(256) for the same 0/1 matrix
        assert_eq!(rank_gf2(&RING), 4);
        assert_eq!(RING.iter().fold(0, |x, &y| x ^ y), 0, "every letter twice");
        let rows = |amount_ea: u8| -> Vec<Vec<u8>> {
            RING.iter()
                .enumerate()
                .map(|(i, &card)| {
                    (0..5)
                        .map(|j| match card >> j & 1 {
                            0 => 0,
                            _ if i == 4 && j == 0 => amount_ea,
                            _ => 1,
                        })
                        .collect()
                })
                .collect()
        };
        assert_eq!(rank_gf256(&rows(1)), 4);
        // 16: E + 2A breaks the ring
        assert_eq!(rank_gf256(&rows(2)), 5);
        // every four of the five are useful together
        for skip in 0..5 {
            let four: Vec<u32> = (0..5).filter(|&i| i != skip).map(|i| RING[i]).collect();
            assert_eq!(rank_gf2(&four), 4);
        }
    }

    #[test]
    fn the_bucket_spans_double_with_each_useful_card() {
        // 04_3: one useful card makes 2 things, two make 4 (the cube's lit corners)
        let mut b = Bucket::new();
        assert_eq!(b.span(), [0]);
        b.add(A);
        assert_eq!(b.span(), [0, A]);
        b.add(B);
        assert_eq!(b.span(), [0, A, B, A | B]);
        assert!(b.can_make(A | B) && !b.can_make(C));
        b.add(A | B | C);
        assert_eq!(b.span().len(), 8);
    }

    #[test]
    fn rank_over_gf256_matches_gf2_on_random_01_matrices_and_brute_force() {
        let mut rng = SplitMix64::new(1307);
        for _ in 0..300 {
            let cards: Vec<u32> = (0..6).map(|_| (rng.next_u64() % 64) as u32).collect();
            let rows: Vec<Vec<u8>> = cards
                .iter()
                .map(|&c| (0..6).map(|j| (c >> j & 1) as u8).collect())
                .collect();
            assert_eq!(rank_gf2(&cards), rank_gf256(&rows));
            // brute force: the rank is log2 of how many distinct ex-ors the cards make
            let made: std::collections::BTreeSet<u32> = (0..64u32)
                .map(|s| (0..6).filter(|&i| s >> i & 1 == 1).fold(0, |x, i| x ^ cards[i]))
                .collect();
            assert_eq!(1 << rank_gf2(&cards), made.len());
        }
    }

    #[test]
    fn the_lottery_matches_enumeration_of_every_card_triple() {
        // brute force against the closed form: all 8³ ways to draw three cards over three pieces
        let full = (0..512u32)
            .filter(|&t| rank_gf2(&[t & 7, t >> 3 & 7, t >> 6 & 7]) == 3)
            .count();
        assert_eq!(full, 168, "7 · 6 · 4 of the 512");
        assert_eq!(p_full(2, 3, 0), 168.0 / 512.0);
        assert_eq!(p_full(2, 1, 0), 0.5);
        assert_eq!(p_full(2, 2, 0), 0.375);
        // two pieces, one spare: all 4³ triples of cards over two pieces
        let full = (0..64u32)
            .filter(|&t| rank_gf2(&[t & 3, t >> 2 & 3, t >> 4 & 3]) == 2)
            .count();
        assert_eq!(full as f64 / 64.0, p_full(2, 2, 1));
    }

    #[test]
    fn the_probabilities_print_the_oracles_doubles() {
        // the doubles oracle.py prints, bit for bit (B-F4, B-F5, B-F7, B-F8, B-F28)
        assert_eq!(p_full_limit(2, 0), 0.2887880950866024);
        assert_eq!(p_fail_limit(2, 0), 0.7112119049133976);
        assert_eq!(p_fail_limit(2, 1), 0.42242380982679517);
        assert_eq!(p_fail_limit(2, 2), 0.22989841310239353);
        assert_eq!(p_fail_limit(2, 3), 0.11988390068844976);
        assert_eq!(p_fail(2, 10, 0), 0.710929701580251);
        assert_eq!(p_fail_limit(2, 19), 1.907347420153228e-6);
        assert_eq!(p_fail_limit(2, 20), 9.536740132413907e-7);
        assert_eq!(spares_for(1_000_000), 20);
        assert_eq!(p_fail_limit_times(2, 2, 1_000_000), 229898.41310239353);
        // B-F7, B-F8: the byte field's lottery
        assert_eq!(p_full(256, 1, 0), 0.99609375);
        assert_eq!(p_full_limit(256, 0), 0.996078491211847);
        assert_eq!(p_fail_limit(256, 0), 0.003921508788152991);
    }

    #[test]
    fn the_danger_meter_rises_only_at_the_end() {
        // 05_4: three slots from the end 1/8, then 1/4, then 1/2, for ten pieces or ten million
        for pieces in [10, 10_000_000] {
            assert_eq!(useless_chance(pieces - 3, pieces), 0.125);
            assert_eq!(useless_chance(pieces - 2, pieces), 0.25);
            assert_eq!(useless_chance(pieces - 1, pieces), 0.5);
        }
        assert!(useless_chance(5, 10) < 0.04, "flat near zero for most of the fill");
    }

    #[test]
    fn the_hook_numbers_print_the_oracles_doubles() {
        // the doubles oracle.py prints, bit for bit (B-F1, B-F24)
        assert_eq!(album_mean(HOOK_PIECES), 97876.06036044382);
        assert_eq!(album_mean_floor(HOOK_PIECES), 97_876);
        assert_eq!(p_done_by(1, 1), 2.248774849816482e-44);
        assert_eq!(p_done_by(2, 1), 0.36786104643292994);
        assert_eq!(p_done_by(3, 1), 0.9900498287989156);
        assert_eq!(p_done_by(4, 1), 0.9999000049993334);
        assert_eq!(p_done_by(4, PHONES), 3.7200741159833003e-44);
        assert_eq!(p_done_by(5, PHONES), 0.3678794411530483);
        assert_eq!(p_done_by(6, PHONES), 0.9900498337491631);
        assert_eq!(p_done_by(7, PHONES), 0.9999000049998333);
        assert_eq!(mean_loops(), 2.642190129869331);
        assert_eq!(p_done_in_loop_2_or_3(), 0.9900498287989156);
        assert_eq!(p_waiting_after(3), 0.009950171201084403);
        assert_eq!(carousel_mean_sent(), 23729.95434759208);
        assert_eq!(carousel_mean_caught(), 23492.65480411616);
    }

    #[test]
    fn the_hooks_spoken_verdicts_hold_on_the_model() {
        // 01_6 "Most phones need two or three loops"
        assert!(p_done_in_loop_2_or_3() > 0.5);
        assert!(p_done_by(1, 1) < 1e-40, "nobody is done after one loop");
        // "The unluckiest of a million need five or six": all are done by 6
        // far more often than by 5, and almost never by 4
        assert!(p_done_by(4, PHONES) < 1e-40);
        assert!(p_done_by(6, PHONES) > 0.99);
        // the drawn unluckiest phone is one of those
        assert!((5..=6).contains(&unluckiest_of_a_million()));
        // 01_8: a carousel phone catches fewer packets than whole loops would say
        assert!(carousel_mean_caught() < mean_loops() * f64::from(HOOK_PIECES));
        // "about ninety-eight thousand"
        assert_eq!((album_mean(HOOK_PIECES) / 1000.0).round(), 98.0);
        // the expected album curve fills to within one slot by the average
        let left = f64::from(HOOK_PIECES) - album_filled(HOOK_PIECES, album_mean(HOOK_PIECES));
        assert!((0.0..1.0).contains(&left), "{left}");
    }

    #[test]
    fn the_drawn_field_is_the_models_and_a_few_straggle_after_loop_three() {
        let field = phone_field();
        assert_eq!(field.len(), GLYPHS);
        let by = field_done_by_loop();
        assert_eq!(by[0], 0, "nobody after one loop");
        // a fair draw: each count within 4 standard deviations of its mean
        for (m, &n) in by.iter().enumerate() {
            let p = p_done_by(m as u32 + 1, 1);
            let sd = (GLYPHS as f64 * p * (1.0 - p)).sqrt();
            assert!(
                (n as f64 - GLYPHS as f64 * p).abs() <= 4.0 * sd + 1e-9,
                "loop {}: {n} done",
                m + 1
            );
        }
        // "after loop 3 a few red stragglers"
        let stragglers = GLYPHS - by[2];
        assert!((1..=25).contains(&stragglers), "{stragglers}");
    }

    #[test]
    fn splitmix64_matches_the_reference_sequence() {
        // Vigna's splitmix64.c from seed 0
        let mut g = SplitMix64::new(0);
        assert_eq!(g.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(g.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(g.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn the_deck_gives_both_sides_the_same_nonempty_mix() {
        for p in DECK_PACKETS {
            let m = deck_mix(p);
            assert!((1..16).contains(&m));
            assert_eq!(m, deck_mix(p), "a recipe, not a coin");
        }
    }

    #[test]
    fn double_double_beats_doubles() {
        // 1/3 · 3 = 1 to 32 digits; 0.1 + 0.2 − 0.3 = 0 in decimal, not in doubles
        let third = Dd::recip(3.0);
        let one = third.mul(Dd::new(3.0)).sub(Dd::ONE);
        assert!(one.to_f64().abs() < 1e-31);
        let q = Dd::new(1.0).div(Dd::new(3.0)).sub(third);
        assert!(q.to_f64().abs() < 1e-31);
        let tiny = Near1::below(Dd::new(1e-60)).pow(10_000);
        assert!((tiny.delta.to_f64() / 1e-56 - 1.0).abs() < 1e-12, "δ keeps its digits");
    }
}
