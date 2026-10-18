//! The mathematics of the two echoes (plan §3.5: pure math with tests;
//! chapters only draw). Everything here is exact integer arithmetic.
//!
//! A *code* is a row of pulses, each up (+1) or down (−1). The *scene* is
//! what sends echoes back: `scene[d]` is the strength of the echo that
//! returns `d` steps late. The *recording* is the sum of those delayed,
//! scaled copies of the code ([`echo`]), plus whatever noise the receiver
//! adds. This is the measurement model of Chi, Pezeshki and Calderbank
//! (*Complementary Waveforms for Sidelobe Suppression and Radar
//! Polarimetry*, eq. 33: delay, scattering, additive noise), sampled one
//! value per pulse and with no Doppler.
//!
//! The receiver slides its own copy of the code along the recording and, at
//! every shift, adds the products of the pulses that line up
//! ([`correlate`], the matched filter): that sum is the shift's *score*. A
//! product is +1 where two pulses agree and −1 where they disagree. The
//! scores of a code against itself are its aperiodic autocorrelation.
//!
//! Golay's pair: neither code's score row is clean on its own (no row of
//! ±1 pulses longer than one is, [`far_shift_score`]), but the two rows
//! cancel at every shift where there is no echo, provided both
//! measurements see the same scene ([`two_measurements`]): the sum is then
//! the scene itself, magnified 2N times (the chapter's definition 1:
//! C_a(k) + C_b(k) = 2N·δ(k)). That identity removes the patterns' own
//! ghosts; noise added at the receiver is scored like any recording, and
//! nothing in the identity makes it cancel.

/// The first code: up, up, up, down.
pub const CODE_A: [i64; 4] = [1, 1, 1, -1];

/// Its partner: up, up, down, up.
pub const CODE_B: [i64; 4] = [1, 1, -1, 1];

/// One thing that reflects, right where the recording starts (chapter 01).
pub const STILL: [i64; 1] = [1];

/// The same thing one step further away: it moved between the two sends.
pub const MOVED: [i64; 2] = [0, 1];

/// The same thing at the same place, its echo upside down (chapter 04): a
/// strength of −1, not a new reflector and not a negative distance. In radar
/// a half turn of phase between the sends does this (Chi, Pezeshki and
/// Calderbank, eq. 8: the second response picks up e^{jνT}).
pub const FLIPPED: [i64; 1] = [-1];

/// One stray blip the receiver picks up during the first send, just after
/// the echo (chapter 04), added to the recording sample by sample.
pub const NOISE_BLIP: [i64; 5] = [0, 0, 0, 0, 1];

/// Chapter 02's scene: a loud thing (strength 4) where the recording starts
/// and a quiet one (strength 1) three steps farther, the same for both sends.
pub const LOUD_AND_QUIET: [i64; 4] = [4, 0, 0, 1];

/// Golay's complementary pair of length 10, as Wikipedia's *Complementary
/// sequences* gives it (read 2026-10-17): a pair whose ghosts cancel that the
/// doubling cannot build, since 10 is not a power of two (chapter 03).
pub const GOLAY_10: ([i64; 10], [i64; 10]) = ([1, 1, -1, 1, -1, 1, -1, -1, 1, 1], [1, 1, -1, 1, 1, 1, 1, 1, -1, -1]);

/// A row of scores: `values[i]` is the score at shift `first + i`, and
/// every shift outside the row scores zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scores {
    pub first: i64,
    pub values: Vec<i64>,
}

impl Scores {
    /// The last shift the row holds (`first − 1` when it is empty).
    pub fn last(&self) -> i64 {
        self.first + self.values.len() as i64 - 1
    }

    /// The score at `shift` (zero outside the row).
    pub fn at(&self, shift: i64) -> i64 {
        usize::try_from(shift - self.first)
            .ok()
            .and_then(|i| self.values.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// The two rows added shift by shift, over every shift either holds.
    pub fn plus(&self, other: &Self) -> Self {
        if other.values.is_empty() {
            return self.clone();
        }
        if self.values.is_empty() {
            return other.clone();
        }
        let first = self.first.min(other.first);
        let last = self.last().max(other.last());
        Self {
            first,
            values: (first..=last).map(|k| self.at(k) + other.at(k)).collect(),
        }
    }

    /// The largest score, in size, at a shift with no echo (`echoes` are the
    /// shifts where the scene has one): the tallest ghost, or 0.
    pub fn largest_ghost(&self, echoes: &[i64]) -> i64 {
        (self.first..=self.last())
            .filter(|k| !echoes.contains(k))
            .map(|k| self.at(k).abs())
            .max()
            .unwrap_or(0)
    }

    /// The shifts with no echo whose score is not zero.
    pub fn ghost_shifts(&self, echoes: &[i64]) -> Vec<i64> {
        (self.first..=self.last())
            .filter(|k| !echoes.contains(k) && self.at(*k) != 0)
            .collect()
    }

    /// `-1,0,1,4,1,0,-1`: the values as the facts print them.
    pub fn csv(&self) -> String {
        csv(&self.values)
    }

    /// Every score times `factor`.
    pub fn scaled(&self, factor: i64) -> Self {
        Self {
            first: self.first,
            values: self.values.iter().map(|v| v * factor).collect(),
        }
    }

    /// The same scores, `by` shifts later.
    pub fn shifted(&self, by: i64) -> Self {
        Self {
            first: self.first + by,
            values: self.values.clone(),
        }
    }
}

/// `1,1,1,-1`: integers as the facts print them.
pub fn csv(values: &[i64]) -> String {
    values.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
}

/// The pairs that line up at shift `k` when a copy of a code `copy_len`
/// long lies `k` steps along a recording `recording_len` long: (index in
/// the recording, index in the copy), the recording's index increasing.
pub fn overlaps(recording_len: usize, copy_len: usize, k: i64) -> Vec<(usize, usize)> {
    (0..recording_len)
        .filter_map(|n| {
            let j = usize::try_from(n as i64 - k).ok()?;
            (j < copy_len).then_some((n, j))
        })
        .collect()
}

/// One lined-up pair at a shift: where it sits, and its product (+1 when
/// the two pulses agree, −1 when they disagree, for ±1 pulses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Product {
    pub recording: usize,
    pub copy: usize,
    pub value: i64,
}

/// Where a lined-up pair sits in a pattern made of two halves (chapter 03).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Span {
    /// Both pulses in the same half.
    Inside,
    /// One pulse in each half: the pair reaches across the middle.
    Across,
}

/// Where the pair `p` of a pattern whose halves are `half` pulses long sits.
pub fn span(half: usize, p: &Product) -> Span {
    if (p.recording < half) == (p.copy < half) {
        Span::Inside
    } else {
        Span::Across
    }
}

/// The products the receiver adds at shift `k`, in the recording's order.
pub fn products(recording: &[i64], code: &[i64], k: i64) -> Vec<Product> {
    overlaps(recording.len(), code.len(), k)
        .into_iter()
        .map(|(n, j)| Product {
            recording: n,
            copy: j,
            value: recording[n] * code[j],
        })
        .collect()
}

/// The matched filter: the recording scored against the code at every
/// shift where they overlap, score(k) = Σₙ recording[n]·code[n − k], for k
/// from −(code length − 1) to recording length − 1.
pub fn correlate(recording: &[i64], code: &[i64]) -> Scores {
    if recording.is_empty() || code.is_empty() {
        return Scores {
            first: 0,
            values: Vec::new(),
        };
    }
    let first = 1 - code.len() as i64;
    let last = recording.len() as i64 - 1;
    Scores {
        first,
        values: (first..=last)
            .map(|k| products(recording, code, k).iter().map(|p| p.value).sum())
            .collect(),
    }
}

/// A code's scores against itself (its aperiodic autocorrelation), shifts
/// −(N − 1) to N − 1.
pub fn autocorrelation(code: &[i64]) -> Scores {
    correlate(code, code)
}

/// What comes back from `scene`: every echo returns the code, delayed by its
/// step and scaled by its strength, and the recording is their sum,
/// recording[n] = Σ_d scene[d]·code[n − d].
pub fn echo(code: &[i64], scene: &[i64]) -> Vec<i64> {
    if code.is_empty() || scene.is_empty() {
        return Vec::new();
    }
    let mut recording = vec![0; code.len() + scene.len() - 1];
    for (d, &strength) in scene.iter().enumerate() {
        for (i, &pulse) in code.iter().enumerate() {
            recording[d + i] += strength * pulse;
        }
    }
    recording
}

/// One measurement: send `code` into `scene`, add `noise` to the recording
/// (sample by sample from its start; empty for none), and score the
/// recording against the code.
pub fn measure(code: &[i64], scene: &[i64], noise: &[i64]) -> Scores {
    let mut recording = echo(code, scene);
    if recording.len() < noise.len() {
        recording.resize(noise.len(), 0);
    }
    for (r, &n) in recording.iter_mut().zip(noise) {
        *r += n;
    }
    correlate(&recording, code)
}

/// Golay's two measurements: send `a` while the scene is `scene_a`, then
/// `b` while it is `scene_b`, score each recording against its own code and
/// add the two score rows. With one scene for both and a complementary
/// pair, the sum is that scene magnified 2N times, with no ghosts.
pub fn two_measurements(a: &[i64], scene_a: &[i64], b: &[i64], scene_b: &[i64]) -> Scores {
    measure(a, scene_a, &[]).plus(&measure(b, scene_b, &[]))
}

/// What changed between two scenes, step by step: `first − second`, the
/// shorter scene extended by zeros past its end (chapter 04).
///
/// With a complementary pair (a, b) of N pulses, the two measurements add to
/// 2N times the second scene plus pattern a's row of this difference:
/// S_a(first) + S_b(second) = S_a(second) + S_b(second) + S_a(first − second),
/// because a measurement is linear in its scene, and the first two terms are
/// the pair's rows added (2N at shift 0, zero elsewhere) laid on the second
/// scene. Rows are finite and extended by zero; nothing wraps around.
pub fn difference(first: &[i64], second: &[i64]) -> Vec<i64> {
    (0..first.len().max(second.len()))
        .map(|d| first.get(d).copied().unwrap_or(0) - second.get(d).copied().unwrap_or(0))
        .collect()
}

/// What each echo of `scene` adds to `code`'s scores, one entry per echo:
/// (its step, the code's own row scaled by its strength and moved to its
/// step). Recording and scoring are both sums of products, so the scene's
/// row is exactly the sum of these (`measure(code, scene, &[])`): two echoes
/// never interact except by adding, which is how a ghost of one can erase,
/// or double, the peak of another.
pub fn contributions(code: &[i64], scene: &[i64]) -> Vec<(i64, Scores)> {
    let own = autocorrelation(code);
    scene
        .iter()
        .enumerate()
        .filter(|&(_, &strength)| strength != 0)
        .map(|(d, &strength)| (d as i64, own.scaled(strength).shifted(d as i64)))
        .collect()
}

/// The score at the far shift, N − 1, where only the code's two end pulses
/// overlap: their product, never zero for a code of nonzero pulses longer
/// than one (±1 for up and down pulses). So no single such code is free of
/// ghosts.
pub fn far_shift_score(code: &[i64]) -> i64 {
    autocorrelation(code).at(code.len() as i64 - 1)
}

/// `a` and `b` are complementary: equal lengths, and their score rows add
/// to 2N at shift zero and to zero at every other shift.
pub fn is_complementary(a: &[i64], b: &[i64]) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    let sum = autocorrelation(a).plus(&autocorrelation(b));
    let peak = 2 * a.len() as i64;
    (sum.first..=sum.last()).all(|k| {
        let want = if k == 0 { peak } else { 0 };
        sum.at(k) == want
    })
}

/// The doubling construction (chapter 03): from a pair `(a, b)` of equal
/// length, the pair (a then b, a then −b), twice as long.
///
/// Why it keeps a cancelling pair cancelling, at every shift k and for any
/// equal-length halves: a lined-up pair of the new pattern either sits
/// inside one half or reaches across the middle ([`span`]). Inside the first
/// half both new patterns hold a, inside the second they hold b and −b, and
/// (−x)(−y) = xy, so every inside product is the same in both rows; summed
/// over both halves they are a's row plus b's row. Across the middle exactly
/// one of the two pulses is negated in the second pattern, so every crossing
/// product changes sign. Added:
///
/// S(a then b)(k) + S(a then −b)(k) = 2 (S(a)(k) + S(b)(k)),
///
/// the old rows extended by zero past their ends. If a's and b's rows add to
/// 2N at shift 0 and to 0 elsewhere, the new rows add to 4N = 2(2N) at 0 and
/// to 0 elsewhere: a complementary pair twice as long. This builds pairs of
/// every power-of-two length from one pulse; it does not build every
/// complementary pair ([`GOLAY_10`]).
pub fn double(a: &[i64], b: &[i64]) -> (Vec<i64>, Vec<i64>) {
    let first = a.iter().chain(b).copied().collect();
    let second = a.iter().copied().chain(b.iter().map(|&x| -x)).collect();
    (first, second)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(first: i64, values: &[i64]) -> Scores {
        Scores {
            first,
            values: values.to_vec(),
        }
    }

    /// Every code of `n` up/down pulses.
    fn codes(n: usize) -> Vec<Vec<i64>> {
        (0u32..(1 << n))
            .map(|m| (0..n).map(|i| if (m >> i) & 1 == 1 { -1 } else { 1 }).collect())
            .collect()
    }

    /// Every complementary pair of `n` up/down pulses, by search.
    fn pairs(n: usize) -> Vec<(Vec<i64>, Vec<i64>)> {
        let all = codes(n);
        let mut out = Vec::new();
        for a in &all {
            for b in &all {
                if is_complementary(a, b) {
                    out.push((a.clone(), b.clone()));
                }
            }
        }
        out
    }

    /// Pattern 1, narration truth: the rows chapter 01 draws and says.
    #[test]
    fn the_chapter_scores_are_the_ones_on_screen() {
        let a = autocorrelation(&CODE_A);
        let b = autocorrelation(&CODE_B);
        assert_eq!(a, row(-3, &[-1, 0, 1, 4, 1, 0, -1]), "code A's scores");
        assert_eq!(b, row(-3, &[1, 0, -1, 4, -1, 0, 1]), "code B's scores");
        // "Lined up exactly, all four agree."
        assert!(products(&CODE_A, &CODE_A, 0).iter().all(|p| p.value == 1));
        // "One step off, three pairs line up. Two agree, one disagrees."
        let one_off: Vec<i64> = products(&CODE_A, &CODE_A, 1).iter().map(|p| p.value).collect();
        assert_eq!(one_off, [1, 1, -1]);
        assert_eq!(a.at(1), 1, "a score of one, where nothing is");
        // "a quarter as tall": the tallest ghost against the echo
        assert_eq!((a.largest_ghost(&[0]), a.at(0)), (1, 4));
        assert_eq!(b.largest_ghost(&[0]), 1, "ghosts of the same size");
        assert_eq!(a.ghost_shifts(&[0]), b.ghost_shifts(&[0]), "at the same shifts");
        // "Every ghost meets its opposite. Only the echo is left, twice as tall."
        let both = two_measurements(&CODE_A, &STILL, &CODE_B, &STILL);
        assert_eq!(both, row(-3, &[0, 0, 0, 8, 0, 0, 0]));
        assert_eq!(both.at(0), 2 * a.at(0));
        // "Move the echo one step between them, and ghosts return."
        let moved = two_measurements(&CODE_A, &STILL, &CODE_B, &MOVED);
        assert_eq!(moved, row(-3, &[-1, 1, 1, 3, 5, -1, -1, 1]));
        assert_eq!(moved.ghost_shifts(&[0, 1]), [-3, -2, -1, 2, 3, 4]);
    }

    /// Pattern 2, a second formula: the score at shift k is the recording
    /// convolved with the code reversed, read at k + N − 1.
    #[test]
    fn the_matched_filter_is_convolution_with_the_reversed_code() {
        for n in 1..=4 {
            for code in codes(n) {
                let reversed: Vec<i64> = code.iter().rev().copied().collect();
                for len in 1..=6 {
                    for recording in codes(len) {
                        let conv = echo(&recording, &reversed);
                        let scores = correlate(&recording, &code);
                        assert_eq!(scores.values.len(), conv.len());
                        assert_eq!(scores.first, 1 - n as i64);
                        assert_eq!(scores.values, conv, "{recording:?} against {code:?}");
                    }
                }
            }
        }
    }

    /// The echo of a scene is the sum of its echoes, each a shifted, scaled
    /// copy of the code (written out the long way).
    #[test]
    fn a_recording_is_the_sum_of_shifted_scaled_copies() {
        let scene = [4, 0, -2, 1];
        let mut want = vec![0; CODE_A.len() + scene.len() - 1];
        for (d, &h) in scene.iter().enumerate() {
            let mut copy = vec![0; want.len()];
            copy[d..d + CODE_A.len()].copy_from_slice(&CODE_A.map(|x| h * x));
            for (w, c) in want.iter_mut().zip(copy) {
                *w += c;
            }
        }
        assert_eq!(echo(&CODE_A, &scene), want);
        assert_eq!(echo(&CODE_A, &STILL), CODE_A, "one echo returns the code itself");
    }

    /// The claim: one scene for both measurements gives back the scene
    /// magnified 2N times, ghost-free, for every complementary pair of up to
    /// eight pulses and every scene of three steps with strengths −2 to 2.
    #[test]
    fn the_same_scene_twice_returns_the_scene_magnified() {
        let strengths = [-2, -1, 0, 1, 2];
        for n in [1, 2, 4, 8] {
            for (a, b) in pairs(n) {
                for &h0 in &strengths {
                    for &h1 in &strengths {
                        for &h2 in &strengths {
                            let scene = [h0, h1, h2];
                            let sum = two_measurements(&a, &scene, &b, &scene);
                            for k in sum.first..=sum.last() {
                                let want = usize::try_from(k)
                                    .ok()
                                    .and_then(|d| scene.get(d))
                                    .map_or(0, |&h| 2 * n as i64 * h);
                                assert_eq!(sum.at(k), want, "{a:?} {b:?} {scene:?} at {k}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Pattern 3, the edges of every always/never sentence.
    ///
    /// "Every ghost meets its opposite": for a complementary pair, code B's
    /// score is minus code A's at every shift but zero.
    #[test]
    fn a_pairs_ghosts_are_always_opposite() {
        for n in 1..=8 {
            for (a, b) in pairs(n) {
                let (ra, rb) = (autocorrelation(&a), autocorrelation(&b));
                for k in ra.first..=ra.last() {
                    if k != 0 {
                        assert_eq!(ra.at(k), -rb.at(k), "{a:?} {b:?} at {k}");
                    }
                }
            }
        }
    }

    /// Complementary pairs of up/down pulses exist only at some lengths
    /// (exhaustive counts of ordered pairs; oracle.py counts them its own
    /// way).
    #[test]
    fn complementary_pairs_by_length() {
        let counts: Vec<usize> = (1..=8).map(|n| pairs(n).len()).collect();
        assert_eq!(counts, [4, 8, 0, 32, 0, 0, 0, 192]);
        assert!(is_complementary(&CODE_A, &CODE_B));
        assert!(!is_complementary(&CODE_A, &CODE_A), "a code is never its own partner");
        assert!(!is_complementary(&CODE_A, &CODE_B[..3]), "different lengths");
        assert!(!is_complementary(&[], &[]));
    }

    /// "No single code is free of ghosts": at the far shift only the two end
    /// pulses overlap, so the score there is ±1, for every code longer than
    /// one pulse. One pulse has no other shift (the edge).
    #[test]
    fn every_code_longer_than_one_pulse_has_a_ghost() {
        for n in 2..=10 {
            for code in codes(n) {
                let s = far_shift_score(&code);
                assert_eq!(s.abs(), 1, "{code:?}");
                assert_eq!(s, code[0] * code[n - 1]);
                assert!(autocorrelation(&code).largest_ghost(&[0]) >= 1);
            }
        }
        assert_eq!(autocorrelation(&[1]), row(0, &[1]));
        assert_eq!(autocorrelation(&[-1]).largest_ghost(&[0]), 0);
    }

    /// The condition: when the scene changes between the two sends, the
    /// ghosts are not cancelled.
    #[test]
    fn a_changed_scene_leaves_ghosts() {
        // moved one step: the chapter's case
        let moved = two_measurements(&CODE_A, &STILL, &CODE_B, &MOVED);
        assert_eq!(moved.largest_ghost(&[0, 1]), 1);
        assert_eq!((moved.at(0), moved.at(1)), (3, 5), "and neither echo is the full 8");
        // the second echo comes back upside down (in radar, a phase change
        // of half a turn between the sends): the echo itself cancels and
        // every ghost doubles
        let flipped = two_measurements(&CODE_A, &STILL, &CODE_B, &[-1]);
        assert_eq!(flipped, row(-3, &[-2, 0, 2, 0, 2, 0, -2]));
        // the second echo comes back twice as strong
        let stronger = two_measurements(&CODE_A, &STILL, &CODE_B, &[2]);
        assert_eq!(stronger, row(-3, &[1, 0, -1, 12, -1, 0, 1]));
        // what is left is always the change, scored by code A alone:
        // A·sa + B·sb = A·(sa − sb) + (A + B)·sb
        let scenes: [&[i64]; 5] = [&[0, 1], &[-1], &[2], &[1, 1], &[0, 0, 3]];
        for scene_b in scenes {
            let change: Vec<i64> = (0..scene_b.len().max(STILL.len()))
                .map(|d| STILL.get(d).copied().unwrap_or(0) - scene_b.get(d).copied().unwrap_or(0))
                .collect();
            let want = measure(&CODE_A, &change, &[]).plus(&two_measurements(&CODE_A, scene_b, &CODE_B, scene_b));
            let have = two_measurements(&CODE_A, &STILL, &CODE_B, scene_b);
            for k in -4..=6 {
                assert_eq!(have.at(k), want.at(k), "{scene_b:?} at {k}");
            }
        }
    }

    /// Noise added at the receiver is scored like any recording: the sum is
    /// the magnified scene plus the noise's own scores, which the identity
    /// does not remove (here one noise sample leaves ghosts at four shifts).
    #[test]
    fn receiver_noise_is_scored_not_removed() {
        let noise = [0, 0, 0, 0, 0, 1];
        let clean = two_measurements(&CODE_A, &STILL, &CODE_B, &STILL);
        let noisy = measure(&CODE_A, &STILL, &noise).plus(&measure(&CODE_B, &STILL, &[]));
        let left = correlate(&noise, &CODE_A);
        for k in -3..=5 {
            assert_eq!(noisy.at(k), clean.at(k) + left.at(k), "at {k}");
        }
        assert_eq!(noisy.largest_ghost(&[0]), 1);
        assert_eq!(noisy.ghost_shifts(&[0]), [2, 3, 4, 5]);
    }

    /// The doubling construction builds the chapter's pair from one pulse,
    /// and every doubling stays complementary (lengths up to 64).
    #[test]
    fn doubling_builds_the_pair_and_keeps_it_complementary() {
        let two = double(&[1], &[1]);
        assert_eq!(two, (vec![1, 1], vec![1, -1]));
        let four = double(&two.0, &two.1);
        assert_eq!((four.0.as_slice(), four.1.as_slice()), (&CODE_A[..], &CODE_B[..]));
        let (mut a, mut b) = (vec![1], vec![1]);
        for _ in 0..6 {
            (a, b) = double(&a, &b);
            assert!(is_complementary(&a, &b), "length {}", a.len());
        }
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn rows_add_and_window_by_shift() {
        let r = row(-1, &[1, 2, 3]);
        assert_eq!((r.last(), r.at(-2), r.at(-1), r.at(1), r.at(2)), (1, 0, 1, 3, 0));
        assert_eq!(r.plus(&row(1, &[10, 20])), row(-1, &[1, 2, 13, 20]));
        assert_eq!(r.plus(&row(0, &[])), r);
        assert_eq!(row(0, &[]).plus(&r), r);
        assert_eq!(r.csv(), "1,2,3");
        assert_eq!(csv(&CODE_A), "1,1,1,-1");
        assert_eq!(overlaps(4, 4, 3), [(3, 0)]);
        assert_eq!(overlaps(4, 4, -3), [(0, 3)]);
        assert!(overlaps(4, 4, 4).is_empty());
        assert_eq!(r.scaled(-2), row(-1, &[-2, -4, -6]));
        assert_eq!(r.shifted(3), row(2, &[1, 2, 3]));
    }

    /// Every scene of exactly `len` steps with strengths −2 to 2 (shorter
    /// scenes appear with trailing zeros).
    fn scenes(len: usize) -> Vec<Vec<i64>> {
        let mut out = vec![Vec::new()];
        for _ in 0..len {
            out = out
                .into_iter()
                .flat_map(|s: Vec<i64>| {
                    (-2..=2).map(move |h| {
                        let mut t = s.clone();
                        t.push(h);
                        t
                    })
                })
                .collect();
        }
        out
    }

    /// The scene's strength at shift `k` (zero off the scene).
    fn strength(scene: &[i64], k: i64) -> i64 {
        usize::try_from(k).ok().and_then(|d| scene.get(d)).copied().unwrap_or(0)
    }

    /// The last shift with a nonzero score, and that score.
    fn lead(r: &Scores) -> Option<(i64, i64)> {
        (r.first..=r.last()).rev().find(|&k| r.at(k) != 0).map(|k| (k, r.at(k)))
    }

    /// Pattern 1, narration truth for chapter 02.
    #[test]
    fn the_loud_and_quiet_rows_are_the_ones_on_screen() {
        let scene = LOUD_AND_QUIET;
        // "Pattern A's two echoes overlap, and the receiver records their sum."
        assert_eq!(echo(&CODE_A, &scene), [4, 4, 4, -3, 1, 1, -1]);
        let a = measure(&CODE_A, &scene, &[]);
        let b = measure(&CODE_B, &scene, &[]);
        assert_eq!(a, row(-3, &[-4, 0, 4, 15, 4, 1, 0, 1, 0, -1]));
        assert_eq!(b, row(-3, &[4, 0, -4, 17, -4, -1, 8, -1, 0, 1]));
        // "At three steps, where the quiet one is, the score is zero."
        assert_eq!((scene[3], a.at(3)), (1, 0));
        // "The loud one's ghost there is minus four. The quiet one's peak is
        // plus four."
        let parts = contributions(&CODE_A, &scene);
        assert_eq!(parts.iter().map(|&(d, _)| d).collect::<Vec<_>>(), [0, 3]);
        assert_eq!((parts[0].1.at(3), parts[1].1.at(3)), (-4, 4));
        // "One step either side of the loud echo, its ghosts stand four
        // tall, as tall as the quiet echo."
        assert_eq!((parts[0].1.at(-1), parts[0].1.at(1)), (4, 4));
        assert_eq!((a.at(-1), a.at(1)), (4, 4), "and so does the row there");
        // either side of the quiet echo the row is not four tall
        assert_eq!((a.at(2), a.at(4)), (1, 1));
        // "Eight. Its ghost lands on the quiet echo instead of erasing it,
        // and doubles it."
        let parts_b = contributions(&CODE_B, &scene);
        assert_eq!((parts_b[0].1.at(3), parts_b[1].1.at(3), b.at(3)), (4, 4, 8));
        // "Thirty-two at zero, eight at three, zero everywhere else. The
        // scene, eight times over."
        let sum = two_measurements(&CODE_A, &scene, &CODE_B, &scene);
        assert_eq!(sum, row(-3, &[0, 0, 0, 32, 0, 0, 8, 0, 0, 0]));
        assert!((sum.first..=sum.last()).all(|k| sum.at(k) == 8 * strength(&scene, k)));
        // "Sending A twice would not help ... the quiet one stays hidden."
        let twice = two_measurements(&CODE_A, &scene, &CODE_A, &scene);
        assert_eq!(twice, a.scaled(2));
        assert_eq!((twice.at(3), twice.at(-1), twice.at(1)), (0, 8, 8));
    }

    /// A scene's row is the sum of its echoes' rows (the code's own row,
    /// scaled and moved), for every code of up to four pulses and every
    /// scene of three or four steps with strengths −2 to 2.
    #[test]
    fn a_scenes_row_is_the_sum_of_its_echoes_rows() {
        for n in 1..=4 {
            for code in codes(n) {
                for scene in scenes(3).into_iter().chain(scenes(4)) {
                    let whole = measure(&code, &scene, &[]);
                    let summed = contributions(&code, &scene)
                        .into_iter()
                        .fold(row(0, &[]), |acc, (_, r)| acc.plus(&r));
                    for k in -8..=8 {
                        assert_eq!(whole.at(k), summed.at(k), "{code:?} {scene:?} at {k}");
                    }
                }
            }
        }
    }

    /// "Zero is not empty": a zero score is not an absent echo, and a
    /// nonzero score is not an echo. In the chapter's scene pattern A scores
    /// zero at the quiet echo and scores at six shifts where nothing is;
    /// only the pair's sum is zero exactly where the scene is empty.
    #[test]
    fn a_zero_score_is_not_an_absent_echo() {
        let scene = LOUD_AND_QUIET;
        let a = measure(&CODE_A, &scene, &[]);
        let hidden: Vec<i64> = (a.first..=a.last())
            .filter(|&k| strength(&scene, k) != 0 && a.at(k) == 0)
            .collect();
        let ghosts: Vec<i64> = (a.first..=a.last())
            .filter(|&k| strength(&scene, k) == 0 && a.at(k) != 0)
            .collect();
        assert_eq!(hidden, [3]);
        assert_eq!(ghosts, [-3, -1, 1, 2, 4, 6]);
        let sum = two_measurements(&CODE_A, &scene, &CODE_B, &scene);
        assert!((sum.first..=sum.last()).all(|k| (sum.at(k) != 0) == (strength(&scene, k) != 0)));
    }

    /// The wrong-pair case, as a theorem: for every ordered pair of
    /// four-pulse patterns and every scene with an echo in it, the two rows
    /// add to 8 times the scene exactly when the pair is complementary. The
    /// reason, checked as it is: the sum minus 8 times the scene is the scene
    /// convolved with the pair's leftover g = C_a + C_b − 8δ, and the last
    /// nonzero score of a convolution is the product of the two rows' last
    /// nonzero scores, so it is never zero when both are nonzero. Exactly
    /// the 32 complementary pairs give the chapter's scene back.
    #[test]
    fn only_a_pair_whose_ghosts_cancel_gives_the_scene_back() {
        let mut checked: Vec<Vec<i64>> = scenes(3).into_iter().filter(|s| s.iter().any(|&h| h != 0)).collect();
        checked.push(LOUD_AND_QUIET.to_vec());
        let (mut pairs_seen, mut giving_back) = (0, 0);
        for a in codes(4) {
            for b in codes(4) {
                pairs_seen += 1;
                let g = autocorrelation(&a).plus(&autocorrelation(&b)).plus(&row(0, &[-8]));
                assert_eq!(lead(&g).is_none(), is_complementary(&a, &b), "{a:?} {b:?}");
                for scene in &checked {
                    let eight: Vec<i64> = scene.iter().map(|h| -8 * h).collect();
                    let residual = two_measurements(&a, scene, &b, scene).plus(&row(0, &eight));
                    let want = match (lead(&g), lead(&row(0, scene))) {
                        (None, _) => None,
                        (Some((kg, cg)), Some((ks, cs))) => Some((kg + ks, cg * cs)),
                        (Some(_), None) => unreachable!("every scene checked has an echo"),
                    };
                    assert_eq!(lead(&residual), want, "{a:?} {b:?} {scene:?}");
                }
                let back = two_measurements(&a, &LOUD_AND_QUIET, &b, &LOUD_AND_QUIET);
                if (back.first..=back.last()).all(|k| back.at(k) == 8 * strength(&LOUD_AND_QUIET, k)) {
                    giving_back += 1;
                }
            }
        }
        assert_eq!((pairs_seen, giving_back), (256, 32));
    }

    /// The products of the pattern `x then y` at shift `k`, with where each
    /// sits.
    fn sorted(x: &[i64], y: &[i64], k: i64) -> Vec<(Product, Span)> {
        let whole: Vec<i64> = x.iter().chain(y).copied().collect();
        products(&whole, &whole, k)
            .into_iter()
            .map(|p| (p, span(x.len(), &p)))
            .collect()
    }

    /// Pattern 1, narration truth for chapter 03.
    #[test]
    fn the_doubling_rows_are_the_ones_on_screen() {
        // "Start with one pulse, paired with itself."
        assert!(is_complementary(&[1], &[1]));
        // "Write the first, then the other. Then the first again, and the
        // other flipped."
        let two = double(&[1], &[1]);
        assert_eq!(two, (vec![1, 1], vec![1, -1]));
        // "Double again, and out come patterns A and B."
        let four = double(&two.0, &two.1);
        assert_eq!((four.0.as_slice(), four.1.as_slice()), (&CODE_A[..], &CODE_B[..]));
        // "Slide each copy one step": A's pairs inside, across, inside; B's
        // the same pairs, the crossing product flipped
        let (x, y) = (&two.0, &two.1);
        let neg: Vec<i64> = y.iter().map(|v| -v).collect();
        let a = sorted(x, y, 1);
        let b = sorted(x, &neg, 1);
        let spans: Vec<Span> = a.iter().map(|&(_, s)| s).collect();
        assert_eq!(spans, [Span::Inside, Span::Across, Span::Inside]);
        let values = |r: &[(Product, Span)]| r.iter().map(|(p, _)| p.value).collect::<Vec<_>>();
        assert_eq!((values(&a[..]), values(&b[..])), (vec![1, 1, -1], vec![1, -1, -1]));
        // "The crossing products cancel in pairs. The rest is twice the old
        // pair's rows, whose ghosts already cancel."
        let old = autocorrelation(x).plus(&autocorrelation(y));
        assert_eq!(old.at(1), 0);
        let inside: i64 = a
            .iter()
            .chain(&b)
            .filter(|(_, s)| *s == Span::Inside)
            .map(|(p, _)| p.value)
            .sum();
        assert_eq!(inside, 2 * old.at(1));
        // "twice as long", and the new rows add to twice the old peak
        assert_eq!(four.0.len(), 2 * two.0.len());
        let new = autocorrelation(&four.0).plus(&autocorrelation(&four.1));
        assert_eq!(new.at(0), 2 * old.at(0));
    }

    /// The doubling identity, checked as its proof argues it: for every pair
    /// of equal-length halves (all up/down halves of 1 to 4 pulses, and all
    /// halves of 1 or 2 values from −2 to 2) and every shift, each lined-up
    /// pair of (x then −y) has the same product as the matching pair of
    /// (x then y) when it sits inside a half and the opposite product when it
    /// reaches across; so the rows add to twice the halves' rows.
    #[test]
    fn the_doubling_identity_holds_pair_by_pair() {
        let mut halves: Vec<Vec<i64>> = (1..=4).flat_map(codes).collect();
        halves.extend(scenes(1).into_iter().chain(scenes(2)));
        for x in &halves {
            for y in halves.iter().filter(|y| y.len() == x.len()) {
                let neg: Vec<i64> = y.iter().map(|v| -v).collect();
                let n = 2 * x.len() as i64;
                for k in 1 - n..n {
                    let (first, second) = (sorted(x, y, k), sorted(x, &neg, k));
                    assert_eq!(first.len(), second.len());
                    for ((p, s), (q, t)) in first.iter().zip(&second) {
                        assert_eq!((p.recording, p.copy, s), (q.recording, q.copy, t));
                        let want = if *s == Span::Inside { p.value } else { -p.value };
                        assert_eq!(q.value, want, "{x:?} {y:?} at {k}: {p:?}");
                    }
                    let (a, b) = double(x, y);
                    let left = autocorrelation(&a).at(k) + autocorrelation(&b).at(k);
                    let right = 2 * (autocorrelation(x).at(k) + autocorrelation(y).at(k));
                    assert_eq!(left, right, "{x:?} {y:?} at {k}");
                }
            }
        }
    }

    /// Where crossing products begin and end (the boundary shifts): with
    /// halves of n pulses, shift 0 has none, shift k from 1 to 2n − 1 has
    /// min(k, 2n − k), and from shift n on there are no inside pairs at all.
    #[test]
    fn crossing_pairs_begin_at_one_step_and_fill_the_far_shifts() {
        for n in 1..=6usize {
            let x = vec![1; n];
            for k in 0..2 * n as i64 {
                let pairs = sorted(&x, &x, k);
                let across = pairs.iter().filter(|(_, s)| *s == Span::Across).count() as i64;
                let inside = pairs.len() as i64 - across;
                let want = if k == 0 { 0 } else { k.min(2 * n as i64 - k) };
                assert_eq!(across, want, "halves of {n} at {k}");
                assert_eq!(inside > 0, k < n as i64, "halves of {n} at {k}");
            }
        }
    }

    /// The planted negative: leave out the flip and the crossing products
    /// add instead of cancelling. Sending (x then y) twice gives twice the
    /// halves' rows plus twice the crossing products, and at the far shift,
    /// where only the two end pulses meet across the middle, that is
    /// 2·x[0]·y[n − 1], never zero: the ghosts come back for every pair.
    /// Flipping the first half instead works as well; flipping both does
    /// not.
    #[test]
    fn leaving_out_the_flip_brings_the_ghosts_back() {
        for n in 1..=4 {
            for (x, y) in pairs(n) {
                let whole: Vec<i64> = x.iter().chain(&y).copied().collect();
                let twice = autocorrelation(&whole).scaled(2);
                let far = 2 * n as i64 - 1;
                assert_eq!(twice.at(far), 2 * x[0] * y[n - 1], "{x:?} {y:?}");
                assert!(!is_complementary(&whole, &whole));
                let flip_first: Vec<i64> = x.iter().map(|v| -v).chain(y.iter().copied()).collect();
                assert!(is_complementary(&whole, &flip_first), "{x:?} {y:?}");
                let flip_both: Vec<i64> = whole.iter().map(|v| -v).collect();
                assert!(!is_complementary(&whole, &flip_both), "{x:?} {y:?}");
            }
        }
        // the chapter's case: A with A
        assert_eq!(autocorrelation(&CODE_A).scaled(2), row(-3, &[-2, 0, 2, 8, 2, 0, -2]));
    }

    /// Not every complementary pair is a doubling: Golay's pair of length 10
    /// cancels, and 10 is not a power of two.
    #[test]
    fn some_pairs_are_not_doublings() {
        let (a, b) = GOLAY_10;
        assert!(is_complementary(&a, &b));
        assert!(!a.len().is_power_of_two());
        assert_eq!(
            autocorrelation(&a),
            row(-9, &[1, 2, -1, -2, 1, 0, -1, 0, -3, 10, -3, 0, -1, 0, 1, -2, -1, 2, 1])
        );
    }

    /// Pattern 1, narration truth for chapter 04: the control, the moved
    /// echo, the flipped echo, the identity's factor, and the noise blip.
    #[test]
    fn the_broken_cases_are_the_ones_on_screen() {
        // "The same scene for both sends, and every ghost cancels."
        let control = two_measurements(&CODE_A, &STILL, &CODE_B, &STILL);
        assert_eq!(control, row(-3, &[0, 0, 0, 8, 0, 0, 0]));
        // "The echo splits into three and five, with ghosts at six shifts."
        let moved = two_measurements(&CODE_A, &STILL, &CODE_B, &MOVED);
        assert_eq!((moved.at(0), moved.at(1)), (3, 5));
        assert_eq!(moved.ghost_shifts(&[0, 1]).len(), 6);
        // "Not a new object, and not a negative distance": one echo, at step
        // 0, with strength −1
        assert_eq!(FLIPPED.len(), STILL.len());
        assert_eq!(FLIPPED[0], -STILL[0]);
        // "It cancels, and every ghost doubles instead."
        let flipped = two_measurements(&CODE_A, &STILL, &CODE_B, &FLIPPED);
        let a = autocorrelation(&CODE_A);
        assert_eq!(flipped.at(0), 0);
        for k in [-3, -2, -1, 1, 2, 3] {
            assert_eq!(flipped.at(k), 2 * a.at(k), "at {k}");
        }
        // "eight times the second scene, plus pattern A's row of whatever
        // changed"
        for second in [&MOVED[..], &FLIPPED[..], &STILL[..]] {
            let sum = two_measurements(&CODE_A, &STILL, &CODE_B, second);
            let want = measure(&CODE_A, &difference(&STILL, second), &[])
                .plus(&row(0, &second.iter().map(|h| 8 * h).collect::<Vec<_>>()));
            for k in -4..=5 {
                assert_eq!(sum.at(k), want.at(k), "{second:?} at {k}");
            }
        }
        // "A stray blip gets its own little row": the peak stays 8
        let noisy = measure(&CODE_A, &STILL, &NOISE_BLIP).plus(&measure(&CODE_B, &STILL, &[]));
        assert_eq!(noisy, row(-3, &[0, 0, 0, 8, -1, 1, 1, 1]));
        let blip = correlate(&NOISE_BLIP, &CODE_A);
        let scored: Vec<(i64, i64)> = (blip.first..=blip.last())
            .filter(|&k| blip.at(k) != 0)
            .map(|k| (k, blip.at(k)))
            .collect();
        assert_eq!(scored, [(1, -1), (2, 1), (3, 1), (4, 1)]);
    }

    /// The changed-scene identity, through its argument: for every
    /// complementary pair of 1 to 4 pulses and every two scenes of 2 steps
    /// with strengths −1 to 1, and for patterns A and B over every two scenes
    /// of 3 steps with strengths −2 to 2, the two rows add to 2N times the
    /// second scene plus pattern A's row of the difference, at every shift.
    #[test]
    fn a_changed_scene_leaves_pattern_as_row_of_the_change() {
        let check = |a: &[i64], b: &[i64], first: &[i64], second: &[i64]| {
            let n = a.len() as i64;
            let sum = two_measurements(a, first, b, second);
            let scaled: Vec<i64> = second.iter().map(|h| 2 * n * h).collect();
            let want = measure(a, &difference(first, second), &[]).plus(&row(0, &scaled));
            let reach = (n + first.len().max(second.len()) as i64) + 1;
            for k in -reach..=reach {
                assert_eq!(sum.at(k), want.at(k), "{a:?} {b:?} {first:?} {second:?} at {k}");
            }
        };
        let small: Vec<Vec<i64>> = (0..9).map(|m| vec![m % 3 - 1, m / 3 - 1]).collect();
        for n in 1..=4 {
            for (a, b) in pairs(n) {
                for first in &small {
                    for second in &small {
                        check(&a[..], &b[..], first.as_slice(), second.as_slice());
                    }
                }
            }
        }
        let wide = scenes(3);
        for first in &wide {
            for second in &wide {
                check(&CODE_A[..], &CODE_B[..], first.as_slice(), second.as_slice());
            }
        }
    }

    /// The flipped echo, as a theorem: for every complementary pair of up to
    /// eight pulses, a single echo whose second send comes back upside down
    /// scores zero where it is (each pattern's own peak is N, and N − N = 0),
    /// while every other shift holds twice the first pattern's ghost (B's
    /// ghosts are minus A's). A boundary: the moved echo still scores at its
    /// two places, so a change need not erase the echo.
    #[test]
    fn a_flipped_echo_cancels_itself_and_doubles_the_ghosts() {
        for n in 1..=8 {
            for (a, b) in pairs(n) {
                let sum = two_measurements(&a, &STILL, &b, &FLIPPED);
                let own = autocorrelation(&a);
                assert_eq!(sum.at(0), 0, "{a:?} {b:?}");
                for k in (1 - n as i64)..(n as i64) {
                    if k != 0 {
                        assert_eq!(sum.at(k), 2 * own.at(k), "{a:?} {b:?} at {k}");
                    }
                }
            }
        }
        let moved = two_measurements(&CODE_A, &STILL, &CODE_B, &MOVED);
        assert!(moved.at(0) != 0 && moved.at(1) != 0);
    }

    /// Noise is scored like any recording, and the pair's identity says
    /// nothing about it: the blip's own row is the code read backwards from
    /// where the blip sits, it is not zero, and the clean peak is untouched.
    #[test]
    fn a_noise_blip_is_scored_and_the_peak_survives() {
        let clean = two_measurements(&CODE_A, &STILL, &CODE_B, &STILL);
        let noisy = measure(&CODE_A, &STILL, &NOISE_BLIP).plus(&measure(&CODE_B, &STILL, &[]));
        let blip = correlate(&NOISE_BLIP, &CODE_A);
        let at = NOISE_BLIP.iter().position(|&v| v != 0).unwrap() as i64;
        for k in -4..=6 {
            assert_eq!(noisy.at(k), clean.at(k) + blip.at(k), "at {k}");
            // the blip at sample `at` meets copy pulse at − k
            let want = usize::try_from(at - k)
                .ok()
                .and_then(|j| CODE_A.get(j))
                .copied()
                .unwrap_or(0);
            assert_eq!(blip.at(k), want, "at {k}");
        }
        assert_eq!(noisy.at(0), clean.at(0), "the peak survives");
        assert!(noisy.largest_ghost(&[0]) > 0, "and the blip is not cancelled");
    }
}
