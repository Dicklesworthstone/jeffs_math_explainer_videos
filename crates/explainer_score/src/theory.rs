//! Music theory for the score: scales, chords as pitch-class sets, motifs as
//! scale degrees, and a voice-leading solver that keeps the music out of the
//! narrator's way.
//!
//! The registral rule (plan §7.3 hard rule 1): during speech no sustained
//! music fundamental sits between 95 and 250 Hz, because the narrator's pitch
//! lives there (5th–95th percentile ≈ 105–216 Hz for the `robert` voice), and
//! a fixed tone in that band roughens and masks the voice as its harmonics
//! glide past. So the bass stays in E1–D2 (41–73 Hz) and the upper voices start
//! at C4 (261.6 Hz; B3 at 246.9 Hz would sit inside the 95–250 Hz band).

/// MIDI note numbers for the registral limits.
pub const BASS_LOW: i32 = 28; // E1, 41.2 Hz
pub const BASS_HIGH: i32 = 38; // D2, 73.4 Hz
pub const UPPER_LOW: i32 = 60; // C4, 261.6 Hz
pub const UPPER_HIGH: i32 = 79; // G5, 784 Hz

/// Mode intervals (semitones above the tonic).
pub const IONIAN: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
pub const LYDIAN: [i32; 7] = [0, 2, 4, 6, 7, 9, 11];
pub const DORIAN: [i32; 7] = [0, 2, 3, 5, 7, 9, 10];
pub const AEOLIAN: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
pub const MIXOLYDIAN: [i32; 7] = [0, 2, 4, 5, 7, 9, 10];

/// A key: tonic pitch class (0 = C) and mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: i32,
    pub mode: [i32; 7],
}

impl Key {
    pub fn new(tonic: i32, mode: [i32; 7]) -> Self {
        Self {
            tonic: tonic.rem_euclid(12),
            mode,
        }
    }

    /// MIDI pitch of scale degree `degree` (1-based; may be ≤ 0 or > 7 to wrap
    /// octaves) in the octave whose tonic is `tonic_midi`.
    pub fn degree(&self, degree: i32, tonic_midi: i32) -> i32 {
        let d = degree - 1;
        let oct = d.div_euclid(7);
        tonic_midi + 12 * oct + self.mode[d.rem_euclid(7) as usize]
    }

    /// Pitch classes of the scale.
    pub fn pitch_classes(&self) -> Vec<i32> {
        self.mode.iter().map(|i| (self.tonic + i).rem_euclid(12)).collect()
    }
}

/// A chord: root pitch class and intervals above it (e.g. maj9 = [0,4,7,11,14]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub root: i32,
    pub intervals: Vec<i32>,
}

impl Chord {
    pub fn new(root: i32, intervals: &[i32]) -> Self {
        Self {
            root: root.rem_euclid(12),
            intervals: intervals.to_vec(),
        }
    }
    pub fn maj7(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 11])
    }
    pub fn maj9(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 11, 14])
    }
    pub fn maj9_sharp11(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 11, 14, 18])
    }
    pub fn min9(root: i32) -> Self {
        Self::new(root, &[0, 3, 7, 10, 14])
    }
    pub fn dom7sus4(root: i32) -> Self {
        Self::new(root, &[0, 5, 7, 10])
    }
    pub fn sus4(root: i32) -> Self {
        Self::new(root, &[0, 5, 7])
    }
    pub fn six_sus4(root: i32) -> Self {
        Self::new(root, &[0, 5, 7, 9])
    }

    /// Distinct pitch classes of the chord.
    pub fn pitch_classes(&self) -> Vec<i32> {
        let mut v: Vec<i32> = self.intervals.iter().map(|i| (self.root + i).rem_euclid(12)).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Pitch classes worth keeping in a 4-voice upper structure, in priority
    /// order: 3rd (or 4th in a sus), 7th/6th, 9th, 5th, then the rest. The root
    /// lives in the bass.
    fn upper_priority(&self) -> Vec<i32> {
        let mut ranked: Vec<(i32, i32)> = self
            .intervals
            .iter()
            .map(|&i| {
                let rank = match i.rem_euclid(12) {
                    3..=5 => 0,
                    9..=11 => 1,
                    2 => 2,
                    6 => 2, // #11 colour
                    7 => 3,
                    _ => 4,
                };
                (rank, (self.root + i).rem_euclid(12))
            })
            .collect();
        ranked.sort();
        let mut out = Vec::new();
        for (_, pc) in ranked {
            if !out.contains(&pc) && pc != self.root {
                out.push(pc);
            }
        }
        out
    }
}

/// A voicing: one bass note and upper voices (MIDI), all within the registral
/// limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Voicing {
    pub bass: i32,
    pub upper: Vec<i32>,
}

/// Choose the voicing of `chord` with `voices` upper voices that moves least
/// from `prev` (total semitones), holding common tones, inside the registral
/// limits, avoiding semitone clusters between upper voices, and covering the
/// chord's most characteristic tones.
pub fn voice_lead(chord: &Chord, voices: usize, prev: Option<&Voicing>) -> Voicing {
    let pcs = chord.upper_priority();
    let want: Vec<i32> = pcs.iter().take(voices).copied().collect();
    // candidate notes: every chord tone, the root included (a triad or a sus
    // chord has fewer non-root tones than voices, so the root and doublings
    // must be available); `want` still ranks the characteristic tones first
    let tones: Vec<i32> = chord.pitch_classes();
    let candidates: Vec<i32> = (UPPER_LOW..=UPPER_HIGH)
        .filter(|m| tones.contains(&m.rem_euclid(12)))
        .collect();
    let bass = {
        let options: Vec<i32> = (BASS_LOW..=BASS_HIGH)
            .filter(|m| m.rem_euclid(12) == chord.root)
            .collect();
        match prev {
            Some(p) => *options
                .iter()
                .min_by_key(|&&b| (b - p.bass).abs())
                .unwrap_or(&options[0]),
            None => options[options.len() / 2],
        }
    };
    let mut best: Option<(i64, Vec<i32>)> = None;
    let mut stack = Vec::with_capacity(voices);
    search(&candidates, 0, voices, &mut stack, &mut |cand: &[i32]| {
        let cost = voicing_cost(cand, &want, prev);
        if best.as_ref().is_none_or(|(c, _)| cost < *c) {
            best = Some((cost, cand.to_vec()));
        }
    });
    let upper = best
        .map(|(_, v)| v)
        .unwrap_or_else(|| candidates.iter().take(voices).copied().collect());
    Voicing { bass, upper }
}

fn search(cands: &[i32], start: usize, left: usize, stack: &mut Vec<i32>, f: &mut impl FnMut(&[i32])) {
    if left == 0 {
        f(stack);
        return;
    }
    for i in start..cands.len() {
        stack.push(cands[i]);
        search(cands, i + 1, left - 1, stack, f);
        stack.pop();
    }
}

fn voicing_cost(cand: &[i32], want: &[i32], prev: Option<&Voicing>) -> i64 {
    let mut cost: i64 = 0;
    // cover the characteristic tones
    for pc in want {
        if !cand.iter().any(|m| m.rem_euclid(12) == *pc) {
            cost += 40;
        }
    }
    // no semitone clusters (they buzz), and keep the voicing compact
    for w in cand.windows(2) {
        let gap = w[1] - w[0];
        if gap < 2 {
            cost += 60;
        }
        if gap > 9 {
            cost += 3 * i64::from(gap - 9);
        }
    }
    // smooth motion from the previous voicing: for equal-size sets of pitches,
    // matching sorted to sorted minimizes total motion (common tones then cost 0)
    if let Some(p) = prev {
        let mut old = p.upper.clone();
        old.sort_unstable();
        if old.len() == cand.len() {
            cost += 2 * cand
                .iter()
                .zip(&old)
                .map(|(a, b)| i64::from((a - b).abs()))
                .sum::<i64>();
        } else {
            for &o in &old {
                cost += 2 * i64::from(cand.iter().map(|&m| (m - o).abs()).min().unwrap_or(12));
            }
        }
    } else {
        // first chord: centre the voicing around E4–A4
        let mean = cand.iter().sum::<i32>() / cand.len().max(1) as i32;
        cost += i64::from((mean - 66).abs());
    }
    cost
}

/// Pitch classes safe for a cue over `chord`: the key's pentatonic-ish subset
/// (scale degrees 1, 2, 3, 5, 6) that are chord tones or a whole tone from
/// one, and never a semitone (minor 9th) from any chord tone (hard rule 7).
pub fn safe_cue_pitch_classes(key: &Key, chord: &Chord) -> Vec<i32> {
    let chord_pcs = chord.pitch_classes();
    let penta: Vec<i32> = [0usize, 1, 2, 4, 5]
        .iter()
        .map(|&d| (key.tonic + key.mode[d]).rem_euclid(12))
        .collect();
    penta
        .into_iter()
        .filter(|pc| {
            chord_pcs
                .iter()
                .all(|c| (pc - c).rem_euclid(12) != 1 && (c - pc).rem_euclid(12) != 1)
        })
        .collect()
}

/// A motif as scale degrees with relative durations.
#[derive(Clone, Debug, PartialEq)]
pub struct Motif {
    pub degrees: Vec<i32>,
    pub durations: Vec<f64>,
}

impl Motif {
    pub fn new(degrees: &[i32], durations: &[f64]) -> Self {
        assert_eq!(degrees.len(), durations.len(), "one duration per degree");
        Self {
            degrees: degrees.to_vec(),
            durations: durations.to_vec(),
        }
    }
    /// Transpose by scale steps.
    pub fn transposed(&self, steps: i32) -> Self {
        Self {
            degrees: self.degrees.iter().map(|d| d + steps).collect(),
            durations: self.durations.clone(),
        }
    }
    /// Mirror around the first degree.
    pub fn inverted(&self) -> Self {
        let pivot = self.degrees[0];
        Self {
            degrees: self.degrees.iter().map(|d| 2 * pivot - d).collect(),
            durations: self.durations.clone(),
        }
    }
    /// Stretch durations.
    pub fn augmented(&self, factor: f64) -> Self {
        Self {
            degrees: self.degrees.clone(),
            durations: self.durations.iter().map(|d| d * factor).collect(),
        }
    }
    /// MIDI pitches in `key` with the tonic at `tonic_midi`.
    pub fn pitches(&self, key: &Key, tonic_midi: i32) -> Vec<i32> {
        self.degrees.iter().map(|&d| key.degree(d, tonic_midi)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::midi_to_hz;

    fn d_major() -> Key {
        Key::new(2, IONIAN)
    }

    #[test]
    fn registral_limits_keep_out_of_the_voice_band() {
        assert!(midi_to_hz(f64::from(BASS_HIGH)) < 95.0);
        assert!(midi_to_hz(f64::from(UPPER_LOW)) > 245.0);
        let prog = [Chord::maj9(2), Chord::min9(11), Chord::maj9_sharp11(7), Chord::sus4(9)];
        let mut prev: Option<Voicing> = None;
        for _ in 0..3 {
            for c in &prog {
                let v = voice_lead(c, 4, prev.as_ref());
                assert!((BASS_LOW..=BASS_HIGH).contains(&v.bass), "{v:?}");
                for &m in &v.upper {
                    let hz = midi_to_hz(f64::from(m));
                    assert!(!(95.0..250.0).contains(&hz), "{c:?}: {m} = {hz:.1} Hz");
                }
                prev = Some(v);
            }
        }
    }

    #[test]
    fn voice_leading_is_minimal_among_valid_voicings_and_holds_common_tones() {
        // Brute force: every 4-note upper voicing in range that covers the
        // chord's characteristic tones and has no semitone clusters. The solver
        // may trade a little motion for compactness, never more than 2 semitones.
        let motion = |a: &[i32], b: &[i32]| a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<i32>();
        let prog = [
            Chord::maj9(2),
            Chord::min9(11),
            Chord::maj9_sharp11(7),
            Chord::sus4(9),
            Chord::maj9(2),
        ];
        let mut prev = voice_lead(&prog[0], 4, None);
        for c in &prog[1..] {
            let got = voice_lead(c, 4, Some(&prev));
            let want: Vec<i32> = c.upper_priority().into_iter().take(4).collect();
            let cands: Vec<i32> = (UPPER_LOW..=UPPER_HIGH)
                .filter(|m| c.pitch_classes().contains(&m.rem_euclid(12)))
                .collect();
            let mut best = i32::MAX;
            let mut stack = Vec::new();
            search(&cands, 0, 4, &mut stack, &mut |v: &[i32]| {
                let covers = want.iter().all(|pc| v.iter().any(|m| m.rem_euclid(12) == *pc));
                let clean = v.windows(2).all(|w| w[1] - w[0] >= 2);
                if covers && clean {
                    best = best.min(motion(v, &prev.upper));
                }
            });
            let m = motion(&got.upper, &prev.upper);
            assert!(
                m <= best + 2,
                "{c:?}: {prev:?} → {got:?} moves {m}, minimum valid {best}"
            );
            let common = prev.upper.iter().filter(|x| got.upper.contains(x)).count();
            let shared_pcs = prev
                .upper
                .iter()
                .filter(|x| c.pitch_classes().contains(&x.rem_euclid(12)))
                .count();
            assert!(common >= shared_pcs.min(2), "{prev:?} → {got:?}");
            prev = got;
        }
    }

    #[test]
    fn voicings_cover_the_third_and_have_no_clusters() {
        for c in [
            Chord::maj7(5),
            Chord::min9(2),
            Chord::maj9_sharp11(10),
            Chord::dom7sus4(0),
        ] {
            let v = voice_lead(&c, 4, None);
            let third = c.upper_priority()[0];
            assert!(v.upper.iter().any(|m| m.rem_euclid(12) == third), "{c:?} {v:?}");
            for w in v.upper.windows(2) {
                assert!(w[1] - w[0] >= 2, "{c:?} {v:?}");
            }
        }
    }

    #[test]
    fn cue_pitches_never_form_a_minor_ninth_with_the_chord() {
        let key = Key::new(5, LYDIAN); // F Lydian
        for chord in [
            Chord::maj9(5),
            Chord::min9(2),
            Chord::maj9_sharp11(10),
            Chord::dom7sus4(0),
        ] {
            for pc in safe_cue_pitch_classes(&key, &chord) {
                for c in chord.pitch_classes() {
                    let d = (pc - c).rem_euclid(12);
                    assert!(d != 1 && d != 11, "{pc} vs chord tone {c} in {chord:?}");
                }
            }
            assert!(!safe_cue_pitch_classes(&key, &chord).is_empty(), "{chord:?}");
        }
    }

    #[test]
    fn motif_transforms() {
        let key = d_major();
        let m = Motif::new(&[1, 5, 6, 3], &[1.0, 1.0, 1.0, 2.0]);
        assert_eq!(m.pitches(&key, 62), vec![62, 69, 71, 66]); // D A B F#
        assert_eq!(m.transposed(1).pitches(&key, 62), vec![64, 71, 73, 67]);
        assert_eq!(m.inverted().degrees, vec![1, -3, -4, -1]);
        assert_eq!(m.augmented(2.0).durations, vec![2.0, 2.0, 2.0, 4.0]);
        assert_eq!(key.degree(8, 62), 74);
        assert_eq!(key.degree(0, 62), 61); // the leading tone below
    }
}
