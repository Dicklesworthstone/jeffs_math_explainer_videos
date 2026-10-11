//! Music theory for the score: scales, chords as pitch-class sets, motifs as
//! scale degrees, and a voice-leading solver that keeps the music out of the
//! narrator's way.
//!
//! The registral rule (plan §7.3 hard rule 1): during speech no sustained
//! music fundamental sits in the narrator's pitch band, because a fixed tone
//! there roughens and masks the voice as its harmonics glide past. The band
//! is measured afresh for every narrator ([`Register::from_f0`]): `robert`
//! (Hoeffding) speaks at ≈ 105–216 Hz (5th–95th percentile), `matt` (video A)
//! at 77–151 Hz, `leo` (B) at 107–205 Hz and `michael` (C) at 76–163 Hz, so
//! each identity carries its own [`Register`]: a bass octave below the band
//! and upper voices above it.

/// Where the music may sit around one narrator (plan §7.3 hard rule 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Register {
    /// The narrator's f0 over voiced frames, 5th and 95th percentiles (Hz),
    /// as measured (`docs/research/sound_design/scripts/yin2.py`).
    pub f0: (f64, f64),
    /// No sustained music fundamental in `[no_fly.0, no_fly.1)` Hz during speech.
    pub no_fly: (f64, f64),
    /// The bass octave (MIDI, inclusive): exactly twelve semitones, so every
    /// root has exactly one bass note, all below the band.
    pub bass: (i32, i32),
    /// The upper voices (MIDI, inclusive), all above the band.
    pub upper: (i32, i32),
    /// Where the solver centres a first chord's upper voices (MIDI).
    pub centre: i32,
}

/// The top of the upper voices' range for every narrator: G5, 784 Hz.
pub const UPPER_TOP: i32 = 79;

impl Register {
    /// `robert`, Hoeffding's narrator, as plan §7.3 fixed it from the v5
    /// takes (≈ 105–216 Hz): no music fundamental in 95–250 Hz under speech,
    /// the bass in E1–E♭2 (41–78 Hz) and the upper voices from C4 (261.6 Hz;
    /// B3 at 246.9 Hz would sit inside the band) to G5.
    pub const ROBERT: Self = Self {
        f0: (105.0, 216.0),
        no_fly: (95.0, 250.0),
        bass: (28, 39),
        upper: (60, UPPER_TOP),
        centre: 66,
    };

    /// The register for a narrator whose voiced f0 runs from `p5` to `p95` Hz
    /// (5th and 95th percentiles): the band reaches two semitones below `p5`
    /// and a minor third above `p95` (as `robert`'s 95–250 Hz does around his
    /// ≈ 105–216 Hz: stressed words rise past the 95th percentile, and an
    /// upper voice's chorus and drift need room); the bass is the highest
    /// whole octave under the band; the upper voices start at the first
    /// semitone above it and run to G5, centred six semitones up.
    pub fn from_f0(p5: f64, p95: f64) -> Self {
        let no_fly = (p5 * 2f64.powf(-2.0 / 12.0), p95 * 2f64.powf(3.0 / 12.0));
        let hz = |m: i32| crate::units::midi_to_hz(f64::from(m));
        let bass_top = (0..127).rev().find(|&m| hz(m) < no_fly.0).unwrap_or(0);
        let upper_low = (0..127).find(|&m| hz(m) >= no_fly.1).unwrap_or(UPPER_TOP);
        Self {
            f0: (p5, p95),
            no_fly,
            bass: (bass_top - 11, bass_top),
            upper: (upper_low, UPPER_TOP),
            centre: upper_low + 6,
        }
    }

    /// True when a sustained fundamental at `hz` would sit in the band.
    pub fn in_band(&self, hz: f64) -> bool {
        (self.no_fly.0..self.no_fly.1).contains(&hz)
    }

    /// A register the engine can use: a twelve-semitone bass entirely under
    /// the band and an upper range entirely over it.
    pub fn check(&self) -> Result<(), String> {
        let hz = |m: i32| crate::units::midi_to_hz(f64::from(m));
        let (lo, hi) = self.no_fly;
        if !(lo.is_finite() && hi.is_finite() && 0.0 < lo && lo < hi) {
            return Err(format!("no-fly band {lo}–{hi} Hz"));
        }
        if self.bass.1 - self.bass.0 != 11 || hz(self.bass.1) >= lo || self.bass.0 < 0 {
            return Err(format!("bass {:?} must be one octave below {lo:.1} Hz", self.bass));
        }
        if hz(self.upper.0) < hi || self.upper.1 - self.upper.0 < 12 {
            return Err(format!(
                "upper voices {:?} must span an octave from {hi:.1} Hz",
                self.upper
            ));
        }
        if !(self.upper.0..=self.upper.1).contains(&self.centre) {
            return Err(format!("centre {} outside {:?}", self.centre, self.upper));
        }
        Ok(())
    }
}

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

/// A chord: root pitch class and intervals above it (e.g. maj9 = [0,4,7,11,14]),
/// and the pitch class its bass plays (the root, or a chord tone for an
/// inversion: F/A).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub root: i32,
    pub intervals: Vec<i32>,
    pub bass: i32,
}

impl Chord {
    pub fn new(root: i32, intervals: &[i32]) -> Self {
        let root = root.rem_euclid(12);
        Self {
            root,
            intervals: intervals.to_vec(),
            bass: root,
        }
    }
    /// The same chord over another of its tones in the bass (`Fmaj7/A`).
    pub fn over(mut self, bass: i32) -> Self {
        let bass = bass.rem_euclid(12);
        assert!(self.pitch_classes().contains(&bass), "{bass} is not a tone of {self:?}");
        self.bass = bass;
        self
    }
    /// The same chord over a pedal: the bass holds `pc` whether or not the
    /// chord contains it (`F♯/E`, the Lydian II over an E pedal).
    pub fn on(mut self, pc: i32) -> Self {
        self.bass = pc.rem_euclid(12);
        self
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
    pub fn min11(root: i32) -> Self {
        Self::new(root, &[0, 3, 7, 10, 14, 17])
    }
    /// The major pentatonic as one chord (1 3 5 6 9): no two tones a
    /// semitone apart, so any tone of that pentatonic sounds clean over it.
    pub fn six_nine(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 9, 14])
    }
    pub fn dom9(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 10, 14])
    }
    pub fn dom13(root: i32) -> Self {
        Self::new(root, &[0, 4, 10, 14, 21])
    }
    /// Augmented colour: the race's tension (whole-tone, plan §7.5 C).
    pub fn dom7_sharp5(root: i32) -> Self {
        Self::new(root, &[0, 4, 8, 10])
    }
    pub fn maj7_sharp5(root: i32) -> Self {
        Self::new(root, &[0, 4, 8, 11])
    }
    pub fn dom9_sharp11(root: i32) -> Self {
        Self::new(root, &[0, 4, 7, 10, 14, 18])
    }

    /// Distinct pitch classes of the chord.
    pub fn pitch_classes(&self) -> Vec<i32> {
        let mut v: Vec<i32> = self.intervals.iter().map(|i| (self.root + i).rem_euclid(12)).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Pitch classes worth keeping in a 4-voice upper structure, in priority
    /// order: 3rd (or 4th in a sus), 7th/6th, 9th, 5th, then the rest. The bass
    /// note (the root, or an inversion's chord tone) is left to the bass.
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
            if !out.contains(&pc) && pc != self.bass {
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
/// from `prev` (total semitones), holding common tones, inside `reg`'s
/// limits, avoiding semitone clusters between upper voices, and covering the
/// chord's most characteristic tones.
pub fn voice_lead(chord: &Chord, voices: usize, prev: Option<&Voicing>, reg: &Register) -> Voicing {
    let pcs = chord.upper_priority();
    let want: Vec<i32> = pcs.iter().take(voices).copied().collect();
    // candidate notes: every chord tone, the root included (a triad or a sus
    // chord has fewer non-root tones than voices, so the root and doublings
    // must be available); `want` still ranks the characteristic tones first.
    // An inversion's bass tone is not doubled above (the third of F/A would
    // thicken and point the ear at the bass)
    let mut tones: Vec<i32> = chord.pitch_classes();
    if chord.bass != chord.root && tones.len() > 3 {
        tones.retain(|&pc| pc != chord.bass);
    }
    let candidates: Vec<i32> = (reg.upper.0..=reg.upper.1)
        .filter(|m| tones.contains(&m.rem_euclid(12)))
        .collect();
    let bass = {
        let options: Vec<i32> = (reg.bass.0..=reg.bass.1)
            .filter(|m| m.rem_euclid(12) == chord.bass)
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
        let cost = voicing_cost(cand, &want, prev, reg.centre, bass);
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

/// Two sustained tones whose fundamentals are 15–60 Hz apart beat in the
/// range the ear hears as roughness (plan §7.6). Low in the register even a
/// minor third qualifies (C4–E♭4 is 50 Hz apart), so the solver spreads
/// voices there and keeps seconds for the top of the range.
pub fn rough(a: i32, b: i32) -> bool {
    let d = (crate::units::midi_to_hz(f64::from(b)) - crate::units::midi_to_hz(f64::from(a))).abs();
    (15.0..=60.0).contains(&d)
}

fn voicing_cost(cand: &[i32], want: &[i32], prev: Option<&Voicing>, centre: i32, bass: i32) -> i64 {
    let mut cost: i64 = 0;
    // cover the characteristic tones; the first (the third, or a sus chord's
    // fourth) says major or minor, so losing it costs more than a rough pair
    for (k, pc) in want.iter().enumerate() {
        if !cand.iter().any(|m| m.rem_euclid(12) == *pc) {
            cost += if k == 0 { 90 } else { 40 };
        }
    }
    // no minor ninths (a semitone and octaves above any lower note, the bass
    // included: E4 under F5 is harsh where F4 under E5 is a major seventh)
    let all: Vec<i32> = std::iter::once(bass).chain(cand.iter().copied()).collect();
    for (i, &lo) in all.iter().enumerate() {
        for &hi in &all[i + 1..] {
            if hi - lo > 1 && (hi - lo).rem_euclid(12) == 1 {
                cost += 60;
            }
        }
    }
    // no semitone clusters (they buzz), no rough pairs, and keep the voicing compact
    for w in cand.windows(2) {
        let gap = w[1] - w[0];
        if gap < 2 {
            cost += 60;
        }
        if rough(w[0], w[1]) {
            cost += 50;
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
        // first chord: centred where the register says (E4–A4 for robert)
        let mean = cand.iter().sum::<i32>() / cand.len().max(1) as i32;
        cost += i64::from((mean - centre).abs());
    }
    cost
}

/// Pitch classes safe for a cue over `chord`: the key's pentatonic subset
/// (scale degrees 1, 2, 3, 5, 6) minus any pitch class a semitone from a chord
/// tone (a minor 9th against it, hard rule 7).
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
        let Some(&pivot) = self.degrees.first() else {
            return self.clone();
        };
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

    /// `robert`'s fixed register and registers measured the same way for a
    /// low voice (`matt`-like, 81–146 Hz) and a high one (`leo`-like, 107–190 Hz).
    fn registers() -> [Register; 3] {
        [
            Register::ROBERT,
            Register::from_f0(80.8, 145.5),
            Register::from_f0(106.7, 190.5),
        ]
    }

    #[test]
    fn a_measured_register_keeps_out_of_its_band_as_robert_s_does() {
        Register::ROBERT.check().unwrap();
        // from_f0 on robert's own f0 lands where plan §7.3 put his band
        let r = Register::from_f0(105.0, 216.0);
        assert!(
            (r.no_fly.0 - 93.5).abs() < 0.1 && (r.no_fly.1 - 256.9).abs() < 0.1,
            "{r:?}"
        );
        for reg in registers() {
            reg.check().unwrap();
            let hz = |m: i32| midi_to_hz(f64::from(m));
            assert!(hz(reg.bass.1) < reg.no_fly.0 && hz(reg.bass.1 + 1) >= reg.no_fly.0 || reg == Register::ROBERT);
            assert!(hz(reg.upper.0) >= reg.no_fly.1 && hz(reg.upper.0 - 1) < reg.no_fly.1 || reg == Register::ROBERT);
            assert!(reg.in_band(reg.f0.0) && reg.in_band(reg.f0.1));
            assert!(!reg.in_band(hz(reg.bass.1)) && !reg.in_band(hz(reg.upper.0)));
        }
        // matt's low voice: bass D1–C♯2 (36.7–69.3 Hz), upper voices from F3 (174.6 Hz)
        let matt = Register::from_f0(80.8, 145.5);
        assert_eq!((matt.bass, matt.upper.0), ((26, 37), 53));
        // a band no register can respect is refused
        let mut bad = Register::ROBERT;
        bad.bass = (32, 43);
        assert!(bad.check().is_err(), "G2 at 98 Hz is not under 95 Hz");
        let mut bad = Register::ROBERT;
        bad.upper.0 = 59;
        assert!(bad.check().is_err(), "B3 at 246.9 Hz is inside the band");
    }

    #[test]
    fn an_inversion_puts_its_tone_in_the_bass_and_the_root_above() {
        // Fmaj7/A: A in the bass, F among the upper voices
        let c = Chord::maj7(5).over(9);
        assert_eq!(c.pitch_classes(), Chord::maj7(5).pitch_classes());
        for reg in registers() {
            let v = voice_lead(&c, 4, None, &reg);
            assert_eq!(v.bass.rem_euclid(12), 9, "{v:?}");
            assert!(v.upper.iter().any(|m| m.rem_euclid(12) == 5), "{v:?}");
            assert!(v.upper.iter().all(|m| m.rem_euclid(12) != 9), "{v:?}");
        }
    }

    #[test]
    #[should_panic(expected = "is not a tone of")]
    fn a_bass_outside_the_chord_is_refused() {
        let _ = Chord::maj7(5).over(2);
    }

    #[test]
    fn every_root_and_quality_voices_without_panicking() {
        // E1..=D2 once missed E♭ entirely, so an E♭ chord (IV in B♭ major,
        // video C's key) had no bass note and the solver panicked.
        let makers: [fn(i32) -> Chord; 7] = [
            Chord::maj7,
            Chord::maj9,
            Chord::maj9_sharp11,
            Chord::min9,
            Chord::dom7sus4,
            Chord::sus4,
            Chord::six_sus4,
        ];
        for reg in registers() {
            for root in 0..12 {
                for make in makers {
                    let c = make(root);
                    let first = voice_lead(&c, 4, None, &reg);
                    let next = voice_lead(&c, 4, Some(&first), &reg);
                    for v in [first, next] {
                        assert_eq!(v.bass.rem_euclid(12), root, "{c:?}");
                        assert!((reg.bass.0..=reg.bass.1).contains(&v.bass));
                        assert!(midi_to_hz(f64::from(v.bass)) < reg.no_fly.0);
                        assert_eq!(v.upper.len(), 4);
                    }
                }
            }
        }
        assert_eq!(Motif::new(&[], &[]).inverted(), Motif::new(&[], &[]));
    }

    #[test]
    fn the_four_identities_voice_without_rough_pairs() {
        // plan §7.5: A (F), B (E), C (B♭), Hoeffding (D)
        let progressions = [
            vec![
                Chord::maj9(5),
                Chord::min9(2),
                Chord::maj9_sharp11(10),
                Chord::dom7sus4(0),
            ],
            vec![
                Chord::maj9(4),
                Chord::min9(1),
                Chord::maj9_sharp11(9),
                Chord::six_sus4(11),
            ],
            vec![
                Chord::maj7(10),
                Chord::min9(7),
                Chord::maj9_sharp11(3),
                Chord::dom7sus4(5),
            ],
            vec![Chord::maj9(2), Chord::min9(11), Chord::maj9_sharp11(7), Chord::sus4(9)],
        ];
        for (prog, reg) in progressions.iter().flat_map(|p| registers().map(|r| (p, r))) {
            let mut prev: Option<Voicing> = None;
            for _ in 0..3 {
                for c in prog {
                    let v = voice_lead(c, 4, prev.as_ref(), &reg);
                    for w in v.upper.windows(2) {
                        assert!(!rough(w[0], w[1]), "{c:?}: {v:?}");
                        assert!(w[1] - w[0] >= 2, "{c:?}: {v:?}");
                    }
                    // the third (or the sus fourth) is always there
                    let third = c.upper_priority()[0];
                    assert!(v.upper.iter().any(|m| m.rem_euclid(12) == third), "{c:?}: {v:?}");
                    prev = Some(v);
                }
            }
        }
        assert!(rough(60, 63) && rough(69, 71) && !rough(60, 64) && !rough(72, 74));
    }

    #[test]
    fn registral_limits_keep_out_of_the_voice_band() {
        assert!(midi_to_hz(f64::from(Register::ROBERT.bass.1)) < 95.0);
        assert!(midi_to_hz(f64::from(Register::ROBERT.upper.0)) > 245.0);
        let prog = [Chord::maj9(2), Chord::min9(11), Chord::maj9_sharp11(7), Chord::sus4(9)];
        for reg in registers() {
            let mut prev: Option<Voicing> = None;
            for _ in 0..3 {
                for c in &prog {
                    let v = voice_lead(c, 4, prev.as_ref(), &reg);
                    assert!((reg.bass.0..=reg.bass.1).contains(&v.bass), "{v:?}");
                    for &m in v.upper.iter().chain([&v.bass]) {
                        let hz = midi_to_hz(f64::from(m));
                        assert!(!reg.in_band(hz), "{c:?}: {m} = {hz:.1} Hz in {:?}", reg.no_fly);
                    }
                    prev = Some(v);
                }
            }
        }
    }

    #[test]
    fn voice_leading_is_minimal_among_valid_voicings_and_holds_common_tones() {
        // Brute force: every 4-note upper voicing in range with no semitone
        // clusters, no rough pairs, and at least the solver's coverage of the
        // chord's characteristic tones. Among those the solver may trade a
        // little motion for compactness, never more than 2 semitones.
        let motion = |a: &[i32], b: &[i32]| a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum::<i32>();
        let prog = [
            Chord::maj9(2),
            Chord::min9(11),
            Chord::maj9_sharp11(7),
            Chord::sus4(9),
            Chord::maj9(2),
        ];
        let reg = Register::ROBERT;
        let mut prev = voice_lead(&prog[0], 4, None, &reg);
        for c in &prog[1..] {
            let got = voice_lead(c, 4, Some(&prev), &reg);
            let want: Vec<i32> = c.upper_priority().into_iter().take(4).collect();
            let cands: Vec<i32> = (reg.upper.0..=reg.upper.1)
                .filter(|m| c.pitch_classes().contains(&m.rem_euclid(12)))
                .collect();
            let mut best = i32::MAX;
            let mut stack = Vec::new();
            let coverage = |v: &[i32]| {
                want.iter()
                    .filter(|pc| v.iter().any(|m| m.rem_euclid(12) == **pc))
                    .count()
            };
            let got_coverage = coverage(&got.upper);
            search(&cands, 0, 4, &mut stack, &mut |v: &[i32]| {
                let clean = v.windows(2).all(|w| w[1] - w[0] >= 2 && !rough(w[0], w[1]));
                if clean && coverage(v) >= got_coverage {
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
            for reg in registers() {
                let v = voice_lead(&c, 4, None, &reg);
                let third = c.upper_priority()[0];
                assert!(v.upper.iter().any(|m| m.rem_euclid(12) == third), "{c:?} {v:?}");
                for w in v.upper.windows(2) {
                    assert!(w[1] - w[0] >= 2, "{c:?} {v:?}");
                }
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
