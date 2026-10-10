> 2026-10-09. Computed numbers below come from the scripts in `docs/research/sound_design/scripts/` (see `scripts/README.md`).

# Sound design memo: warm, unobtrusive procedural music for franken_manim explainers

## 0. Summary

1. **The current pad is mostly not too loud.** It sits at −38.3 LUFS against the voice's −23.3, which is 15 LU below. The problems are where its energy sits, that it never changes, and that it is a bare test tone. Its fundamentals sit inside the narrator's pitch range, and slow beating gives it a 12 dB "wah-wah" tremolo.
2. **The chimes are the real intelligibility problem.** Their loudest 400 ms reaches −26 LUFS, only about 6 LU below the speech's own loudest 400 ms. Their partials land at 1.4–4.8 kHz, dead centre in the stereo image, which is exactly where the narrator sits. My simplified SII estimate (a speech-intelligibility score from 0 to 1, defined in §2.4) drops by 0.11–0.17 during each chime onset.
3. **Levels.** Bed under speech at −18 LU relative to the voice. In authored gaps of 2 s or more, the bed rises to −10 LU. Tonal cues during speech must sit 18–20 LU below the speech's loudest moments, or move into the gaps.
4. **Bake the ducking into the music stems.** Compute it offline from the narration WAVs with exact look-ahead. franken_manim's `gain_to_background` is a step gain with no ramps, applied in insertion order, so it can't do this job.
5. **The franken_manim limits change the stem plan:**
   - Every sound file is capped at 64 MiB. That allows at most 174.8 s per stem as stereo 32-bit float, 233 s as 24-bit, or 349 s as 16-bit.
   - The chapter soundtrack is handed to ffmpeg as **16-bit without dither** and then encoded to AAC per chapter.
   - Write stems as 32-bit float. Keep chapter seams near silence unless mastering moves to PCM.
6. **Synthesis.** Band-limited (PolyBLEP) saws with slow random pitch drift, a zero-delay-feedback ladder filter, Juno/Solina-style chorus, a modal mallet bank, a 2-operator FM electric piano, Karplus–Strong pizzicato, saturation with ADAA (an anti-aliasing technique for distortion), and an 8-line FDN reverb.
   - A prototype of the pad, chorus and reverb rendered 10 minutes of stereo 48 kHz in **6.3 s of CPU time on one core** (M4 Pro).

---

## 1. Diagnosis of the current design

I measured everything below against the actual narration (`explainer_corpus/hoeffding_d/narration_v5/`: 61 sentences, −23.3 LUFS median, loudest true peak −7.5 dBTP).

| Problem | Measurement | Why it grates or gets in the way |
|---|---|---|
| **Monotony** | One static 4–5-note chord for an entire 60–90 s chapter. The 2.5 s fade in and out at every chapter edge makes 8 audible "resets" per video. | Grant's own 3b1b piano pieces change chord every ~8 s ("Grant's Opus") to ~26 s ("Etude"), per analysis of the [published MIDI](https://vincentrubinetti.com/sheets/3blue1brown/). One chord for 90 s is a drone, not harmony. |
| **Test-tone timbre** | Each note has only partials f and 2f. Energy above 2 kHz is effectively zero (−145 dB relative). Spectral centroid is 257 Hz. | Sine-wave sonifications "became annoying" over time ([Sonification Handbook ch. 7](https://sonification.de/handbook/)). Earcon guidelines say "use musical timbres, not sine tones" (ch. 14). |
| **Beating tremolo** | A 1.003 ratio is a 5.2-cent detune, giving beats of 0.29 Hz (G2) to 1.32 Hz (A4). The amplitude pair 1.0/0.6 swings 0.4↔1.6, a **12 dB modulation depth**. Each chord tone beats at its own rate. | On a saw, detune beats are spread across many harmonics at k·Δf and read as shimmer. On a bare sine the whole note pumps, which is the "cheap organ celeste" sound. |
| **Periodic swell** | The `breathe` LFO is 0.07 Hz (14.3 s period) with a 4.4 dB swing, the same rate on every voice. | Over a chapter the swell becomes predictable. |
| **Collides with the voice** | Robert's pitch (YIN analysis): median **136 Hz**, interquartile range 121–154 Hz, 5th–95th percentile 107–200 Hz. Pad tones B2 123.5, D3 146.8, E3 164.8 and F#3 185 Hz sit inside it. In the 160 and 200 Hz third-octave bands the pad is only **5 dB below the voice**. | Masking and roughness: dissonance peaks at about a quarter of a critical bandwidth ([Plomp & Levelt 1965](https://www.mpi.nl/publications/item66382/tonal-consonance-and-critical-bandwidth)). The critical bandwidth is about 100 Hz near 140 Hz ([Zwicker & Terhardt 1980](https://doi.org/10.1121/1.385079)). So as the voice's harmonics glide past a fixed pad tone, they keep passing through maximum roughness, about 25 Hz apart. |
| **Chimes sit in the speech band** | Partials are the free-free bar ratios (1, 2.76, 5.4), the brightest "glockenspiel ping" family. At 880 Hz that is 2.43 and 4.75 kHz. Loudest 400 ms: −26 LUFS, against the speech's loudest 400 ms of −20.1 LUFS. | The 1.6–2.5 kHz third-octave bands carry the most weight in the ANSI S3.5 intelligibility index, and 1–4 kHz holds 59% of the total ([ANSI tables via the CRAN SII package](https://cran.r-project.org/src/contrib/Archive/SII/SII_1.0.3.1.tar.gz)). The ear is most sensitive at 2–5 kHz ([ISO 226](https://en.wikipedia.org/wiki/Equal-loudness_contour)). |
| **Masking estimate** (SII-lite, my estimate) | Pad loses 0.013 (harmless). **A chime's first 400 ms loses 0.11–0.165**, which is like deleting the whole 2–3 kHz region for that moment. | That shows up as a consonant smeared during a reveal sentence, exactly when clarity matters most. |
| **Both mono, both in the centre** | Chimes are mono duplicated to L/R, the same phantom centre as the narrator. | Separating a masker from the target in space improves intelligibility (spatial release from masking, reviewed by Bronkhorst 2000, Acustica 86:117–128; see [this review](https://pmc.ncbi.nlm.nih.gov/articles/PMC5023537)). Co-locating them throws that away. |
| **Exact repetition** | Chime WAVs are cached per pitch: 659 Hz plays 4 times and 880 Hz 3 times, bit-identical every time. | Repeated identical samples are the "machine-gun" effect. Material guidance asks for "minor variations in sound timbre" ([archived](https://web.archive.org/web/2019/https://material.io/design/sound/sound-choreography.html)). |
| **Harmonic clashes** | The C5 (523 Hz) chime plays over Gmaj7 in the `quadruples` chapter: a minor ninth against B3 and a tritone against F#4. The chime's 0.5× partial (262 Hz) is the longest-lived (τ = 2.2 s), right in the voice's strongest band (250–630 Hz, the long-term average speech spectrum peak I measured). | Cues aren't drawn from the current chord, and the longest-ringing partial sits in the voice's busiest band. |
| **No space** | No reverb, early reflections, chorus, filtering or saturation. Pans are linear 0.35/0.65. | The sound is dry and stuck "in the speaker", with no sense of room. |

**Verdict.** The pad's level is fine. Its pitch placement, spectrum and stasis are the problem. The chimes really do mask speech.

---

## 2. Mixing music under narration

### 2.1 What the standards and research say

- **Loudness basics.**
  - EBU R128 targets −23 LUFS with a −1 dBTP maximum ([R128](https://tech.ebu.ch/docs/r/r128v4_0.pdf)).
  - Momentary loudness uses 0.4 s windows and short-term uses 3 s ([Tech 3341](https://tech.ebu.ch/docs/tech/tech3341.pdf)).
  - True peak is measured with 4× oversampling ([BS.1770-5](https://www.itu.int/rec/R-REC-BS.1770)).
  - ATSC A/85 makes dialogue the "anchor element" ([A/85:2026](https://www.atsc.org/wp-content/uploads/2026/07/A85-2026-07.pdf)). Netflix measures dialogue-gated at −27 LKFS ([spec](https://backlothelp.netflix.com/hc/en-us/articles/360001794307-Netflix-Sound-Mix-Specifications-Best-Practices-v1-5)).
  - Your −16 LUFS master matches Apple Podcasts ([−16 ±1](https://podcasters.apple.com/support/audio-requirements)). YouTube only turns loud uploads down, toward about −14 LUFS ([productionadvice](https://productionadvice.co.uk/stats-for-nerds/)).
- **Music-under-voice difference.**
  - Torcoli et al. 2019 (JAES 67(12)) recommend **at least 10 LU commentary-over-music** and 15 LU over ambience. Non-experts preferred differences about 4 LU larger than experts did ([FAU record](https://cris.fau.de/publications/234640600), [Fraunhofer blog](https://audioblog.iis.fraunhofer.com/?p=4251)).
  - Listeners over 65 chose a median of 13.5–15.3 LU, with an interquartile range of 9.8–20.8 ([Straninger 2020](https://arxiv.org/abs/2006.14282)).
  - WCAG 1.4.7 (AAA) asks for background at least 20 dB below speech ([W3C](https://www.w3.org/WAI/WCAG21/Understanding/low-or-no-background-audio.html)).
- **Why music under teaching hurts.**
  - Background music reduced retention and transfer in multimedia lessons ([Moreno & Mayer 2000](https://tecfa.unige.ch/tecfa/teaching/methodo/Moreno_Mayer00.pdf)).
  - Music with "changing state" (moving melodies) disrupts verbal memory more than steady sound ([Jones & Macken 1993](https://orca.cardiff.ac.uk/46903/); [Salamé & Baddeley 1989](https://www.gwern.net/doc/music/music-distraction/1989-salame.pdf)).
  - A meta-analysis of 65 studies finds music hurts reading a little but reliably ([Vasilev et al. 2018](https://eprints.bournemouth.ac.uk/29995/)).
  - **Consequence:** melodic motion belongs in the gaps. Under speech, music should be harmonically slow and rhythmically still.
- **What practitioners do.**
  - Sound On Sound suggests voice-over ducking at about 2:1 with a 15–20 ms attack and 600–700 ms release, so music doesn't swell in short pauses ([Duck Tales](https://www.soundonsound.com/techniques/duck-tales)).
  - iZotope suggests a sidechain dynamic-EQ dip of "2 dB or so in the upper midrange" ([iZotope](https://www.izotope.com/en/learn/5-ways-to-use-dynamic-eq-with-sidechain)).
  - The Planet Earth II composers: the voice is "the lead instrument", the music "holds still" under narration and "plays out when he's not speaking", and "we carved out a certain frequency" ([SPIN](https://www.spinmagazine.com/2017/02/planet-earth-ii-composers-talk-music-david-attenboroughs-voice-interview/), [Television Academy](https://www.televisionacademy.com/features/news/mix/earth-wind-and-choir)).

### 2.2 Level targets

V is the narration's integrated loudness before mastering, ≈ −23 LUFS. All targets are relative to V.

| Situation | Music short-term (3 s) | Notes |
|---|---|---|
| Under speech | **V − 18 LU** (allowed range −16 to −22) | Above Torcoli's 10 LU minimum, with room for non-experts and older listeners. |
| Authored gap of 2 s or more | V − 10 LU | This is where the call-and-response happens. |
| Music-only (title, outro, chapter cards) | Up to V − 4 LU | Stays below the voice so the voice still sets the master gain. |
| Any cue, loudest 400 ms, during speech | ≤ speech's loudest 400 ms − 18 LU (≈ −38 LUFS), low-passed ≤ 2.2 kHz, panned ≥ 30% off centre | In my test, a soft marimba at −14 LU still cost 0.03–0.05 SII; at −20 LU it cost 0.010–0.016. |
| Music stem true peak | ≤ −12 dBTP | Narration peaks at −7.5 dBTP, so the music never limits the master gain. |

### 2.3 Offline ducking with perfect look-ahead (baked into the stem)

Measured from the narration: pauses inside a sentence (≥150 ms) occur about 4.1 times per sentence, median 0.20 s, 90th percentile 0.52 s, maximum 0.91 s. With `BREATH = 0.6`, gaps between sentences are at most 0.75 s. **So a whole chapter merges into one speech region.** The music can only "answer" in holds the author writes into the script. The score API therefore needs `narrator.music_breath(2.5)`.

```text
VAD: 10 ms frames per placed narration line; active ⇔ rms_dB > L_line − 25 dB; dilate ±30 ms
merge regions whose gap < 1.2 s
for each region r = [s, e]:                      // per-region shape, values in [0,1]
  u_r(t) = 0                          t < s − P − A
         = rcos((t − (s−P−A))/A)      ramp up, A = 0.30 s, P = 0.08 s pre-roll
         = 1                          s − P ≤ t ≤ e + H     (H = 0.25 s)
         = 1 − rcos((t − e − H)/R)    release, R = 1.2 s
duck(t) = max_r u_r(t)                           // overlapping ramps never re-open the gap
gain_dB(t) = −D · duck(t),  D = 8 dB             // gap level V−10 → under-voice V−18
pocket(t)  = duck(t)
out = 10^(gain_dB/20) · [(1−pocket)·x + pocket·EQ_lin(x)]   // dynamic EQ as a crossfade
rcos(u) = ½ − ½·cos(π·clamp(u,0,1))
```

- `EQ_lin` is a linear-phase FIR of 1,023 taps at 48 kHz. Apply it with the delay compensated, or zero-phase via FFT, since this is offline.
  - It applies a −5 dB broad bell centred at 2 kHz, spanning about 1–4 kHz.
  - It applies an extra −3 dB on the **mid (centre) channel only** from 300 Hz to 1 kHz. That opens the centre for the narrator and leaves the sides wide.
- Because both paths have identical phase, crossfading between them is an exact time-varying EQ, with no zipper noise and no filter-instability risk.
- A "key line" option ducks the music to silence 0.5 s before a crucial sentence. Use it at most twice per video.
- **Never use `gain_to_background` for this.** In `fmn-output/src/sound.rs`, the mixer multiplies the already-mixed sum by a constant during the cue's active interval, with no ramps. It also follows insertion order, and `pad_under` is added *after* the narration, so narration cues can't duck it anyway.

### 2.4 How intelligibility is measured

- **SII (ANSI S3.5).** The core of each band's score is (speech − masker + 15)/30, clipped to 0–1, then weighted by band importance. The 2 kHz third-octave band has the largest weight, 0.0898 ([SII tables](https://cran.r-project.org/src/contrib/Archive/SII/SII_1.0.3.1.tar.gz)).
  - This is the right model for a masker sitting on top of speech, and it is what my QA check (§5) implements per 400 ms window.
- **STI (IEC 60268-16)** models what a transmission channel does to speech modulations: 7 octave bands, modulation frequencies from 0.63 to 12.5 Hz ([MathWorks summary](https://www.mathworks.com/help/audio/ref/speechtransmissionindex.html)). It suits rooms and channels, not additive music.
- Tang, Fazenda & Cox found that listeners wanted extra margin beyond what objective metrics predicted ([Salford](https://salford-repository.worktribe.com/output/1477484/automatic-speech-to-background-ratio-selection-to-maintain-speech-intelligibility-in-broadcasts-using-an-objective-intelligibility-metric)). Treat SII thresholds as necessary, not sufficient.

---

## 3. Warm analog-style synthesis in plain Rust

Everything runs at 48 kHz in f64. Control-rate updates (coefficients, LFOs, drift) happen every 32 samples.

### 3.1 Oscillators

**PolyBLEP** is a 2-sample polynomial correction applied at each waveform jump ([Finke](https://www.martin-finke.de/articles/audio-plugins-018-polyblep-oscillator/); Välimäki & Huovilainen 2007, IEEE SPM, [doi:10.1109/MSP.2007.323276](https://doi.org/10.1109/MSP.2007.323276); band-limited background in [Stilson & Smith, BLIT](https://ccrma.stanford.edu/~stilti/papers/blit.pdf)):

```rust
fn blep(t: f64, dt: f64) -> f64 {           // t = phase in [0,1), dt = f/fs
    if t < dt { let x = t/dt; x+x - x*x - 1.0 }
    else if t > 1.0-dt { let x = (t-1.0)/dt; x*x + x+x + 1.0 } else { 0.0 } }
saw   = 2t−1 − blep(t,dt)
pulse = naive_pulse(t,pw) + blep(t,dt) − blep((t+1−pw).fract(),dt)     // PWM: pw = 0.5 ± 0.15, LFO 0.3–0.6 Hz
tri   = leaky-integrate(square): y = dt·sq + (1−dt)·y                    // Finke
```

My numpy check: a saw at 1.78 kHz has inharmonic (aliased) energy below 5 kHz of −20.3 dB when naive and −55.5 dB with PolyBLEP.

**Analog drift.** Each oscillator gets an Ornstein–Uhlenbeck random walk in cents: `x += −θ·x·dt + σ·√dt·N(0,1)`, with θ = 0.5–1 s⁻¹ and σ chosen so the stationary spread σ/√(2θ) is 2–4 cents. Static detune is ±5–9 cents between the two saws. Start phases are random from the seeded RNG.

### 3.2 Filters (Zavalishin, *The Art of VA Filter Design*, [NI PDF](https://www.native-instruments.com/fileadmin/ni_media/downloads/pdf/VAFilterDesign_2.1.0.pdf))

```rust
// TPT one-pole LP: g = tan(π fc/fs), G = g/(1+g)
v = (x − s)*G;  y = v + s;  s = y + v;            // ⇒ y = G·x + s/(1+g)
// 4-pole ZDF ladder, feedback k ∈ [0,4): solve the zero-delay loop exactly, then saturate
S  = (G³s₁ + G²s₂ + G·s₃ + s₄)/(1+g)
y4 = (G⁴·x + S)/(1 + k·G⁴)
u  = tanh_adaa(drive·(x − k·y4))/drive;  run 4 one-poles on u;  out = y4·(1 + 0.5k)   // passband compensation
// Simper SVF (trapezoidal), k = 1/Q:  a1 = 1/(1+g(g+k)), a2 = g·a1, a3 = g·a2
v3 = x − ic2; v1 = a1·ic1 + a2·v3; v2 = ic2 + a2·ic1 + a3·v3; ic1 = 2v1 − ic1; ic2 = 2v2 − ic2
low = v2; band = v1; high = x − k·v1 − v2
```

- References: [Huovilainen DAFx-04](https://dafx.de/paper-archive/2004/P_061.PDF) (nonlinear ladder), [Stilson & Smith 1996](https://ccrma.stanford.edu/~stilti/papers/moogvcf.pdf), [Simper SVF](https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf).
- **Key tracking:** `fc = clamp(2.5–4 × f0 + env_amt × env, 250, 2000 Hz)` under speech (up to 6 kHz in gaps). Resonance k = 0.4–1.5. Never self-oscillate.
- **Envelopes:** exponential segments, `y += (target − y)·(1 − e^(−1/(τ·fs)))`, for natural decays. Raised-cosine ramps for automation.

### 3.3 Instrument recipes

| Instrument | Recipe |
|---|---|
| **Juno-style pad** | 2 drifting saws (±7 c) + PWM pulse + sub triangle −1 oct at −8 dB → ladder (k 0.8, fc 3×f0, filter env A 0.8 s / D 3 s / S 0.6, +600 Hz) → amp A 0.8–1.5 s / R 2–3 s → **chorus**. The Juno chorus is two MN3009 bucket-brigade delay lines with filters around them ([SoundForce](https://modulargrid.net/e/modules/view/43993)), triangle LFOs in opposite phase for stereo ([GroupDIY](https://groupdiy.com/threads/juno-60-chorus-lfo-frequency-too-fast.53861/)), commonly quoted at ≈0.5 Hz (I) and ≈0.83 Hz (II) ([TC June-60 manual](https://www.allfordj.ru/upload/iblock/a19/wmxxexutu0d00gcp3g30kemcjy0zqd4r/rukovodstvo_polzovatelya_tc_electronic_june_60_v2.pdf), secondary source). My design values: 3.5 ms ± 1.8 ms, wet low-passed at 8 kHz, L tap +LFO, R tap −LFO. |
| **Solina strings** | Saw + saw an octave up, high-pass 200 Hz, low-pass 1.5 kHz (speech) / 3 kHz (gaps). **Ensemble:** three delay lines (~5 ms) each driven by a slow + fast LFO at 0/120/240° ([electro-music](https://www.electro-music.com/forum/viewtopic.php?p=202164)). Cherry Audio's model of a similar ensemble uses 1.21 and 5.82 Hz ([forum](https://forums.cherryaudio.com/viewtopic.php?p=8675)). I use 0.6 Hz at ±1.5 ms and 6 Hz at ±0.15 ms. |
| **FM electric piano** | [Chowning 1973](https://web.eecs.umich.edu/%7Efessler/course/100/misc/chowning-73-tso.pdf): out = A(t)·sin(ωc·t + I(t)·sin(ωm·t)). Body pair 1:1 with I(t) = Iv·(0.15 + 0.85e^(−t/0.25)), Iv 0.5–2.5 by velocity (≤1.2 under speech). "Tine" pair with modulator ratio **14**, index 1.5·e^(−t/0.02), −18 dB. DX7 E.PIANO 1 is algorithm 5 with one operator at ratio 14 ([patch dump](https://github.com/itsjoesullivan/dx7-patches)). Add tremolo/autopan at 4.5 Hz ±25% and a slight asymmetric clip for pickup "bark". |
| **Felt piano** (additive) | Partials f_n = n·f0·√(1+B·n²) ([stiff-string relation](https://www.ioa.org.uk/system/files/publications/P%20CHUMNANTAS%20CA%20GREATED%20DM%20CAMPBELL%20INHARMONICITY%20OF%20STEPPED%20STIFF%20STRINGS.pdf)), B = 1–4×10⁻⁴ (my choice). A_n ∝ v·n^(−1.2)/(1+(n·f0/fc)²), with felt fc = 300–900 Hz. τ_n = τ₁/(1+0.25(n−1)). Each partial is 2 components detuned ±0.7 c (double decay). Attack 8–15 ms raised cosine, plus a 20 ms low-passed noise "thump" at −32 dB. |
| **Modal mallets** | A two-pole resonator per mode: y = 2r·cos(ω)·y₁ − r²·y₂ + x, r = e^(−1/(τ·fs)). Excite with a half-cosine of contact width 0.4 ms (hard) to 2 ms (soft). **Marimba** 1 : 3.9–4.0 : 9.2 ([CCRMA](https://ccrma.stanford.edu/CCRMA/Courses/150/percussion.html)); **vibraphone** 1:4:10 ([1:4:10 tuning](https://run.unl.pt/entities/publication/1e4adbba-3dbd-49ea-8652-43bf54a97cb0)). Vibe motor tremolo: product ranges run 0.33–2.67 Hz ([Adams](https://www.thomann.co.uk/adams_nomad_vibe_vibra_a442_g.htm)) and up to ~12 Hz ([US9922628](https://patents.google.com/patent/US9922628)). Use 3–5 Hz, depth 0.5 on the fundamental and 0.15 on upper modes, since the resonator mainly amplifies the fundamental. **Kalimba:** first overtone is inharmonic at ~5.3–6.8× ([Chapman](https://acoustics.org/pressroom/httpdocs/155th/chapman.htm), [JCAA](https://jcaa.caa-aca.ca/index.php/jcaa/article/view/1805)), at −15 dB with τ 0.12 s. **Avoid** the free-free 2.76/5.4 bar ratios under speech. |
| **Pizzicato** (Karplus–Strong) | Delay N = fs/f0 − ½ − Δ, tuned with a first-order allpass C = (1−Δ)/(1+Δ). Loop filter ρ[(1−S) + S·z⁻¹], S = 0.5, ρ 0.995–0.999. Excite with a 1.5 ms raised-cosine plus a little noise. Pick-position comb 1 − z^(−βN), β ≈ 0.15 ([PASP KS](https://ccrma.stanford.edu/~jos/pasp/Karplus_Strong_Algorithms.html)). |
| **Warm bass** | Sine + 25% triangle → tanh (drive 1.5, ADAA) → SVF low-pass at 350 Hz, 15 ms attack. Range E1–D2 (41–73 Hz). Brighter only in gaps. |
| **Noise FX** | **Whoosh:** pink noise → SVF band-pass, fc swept on a log scale 250→1600→400 Hz, Q 1.2, independent L/R seeds, pan following the camera. Cap fc at 1.2 kHz under speech. **Riser:** in gaps only; ends 30 ms before the reveal frame. **Bubble:** damped sinusoid at the Minnaert frequency, f ≈ 3.26/r (r in metres), so r = 3–8 mm gives ~1.1 kHz–400 Hz, with a small upward glide (after [van den Doel's liquid-sound model](https://repository.gatech.edu/entities/publication/dcc121b6-5fdb-42db-b758-971522f01c70); check the exact glide law in the paper). |

### 3.4 Colour and space

- **Saturation.** First-order ADAA tanh: y = (F(x_n) − F(x_{n−1}))/(x_n − x_{n−1}) with F = ln cosh, written as |x| + ln(1+e^(−2|x|)) − ln 2 so it can't overflow. Fall back to tanh((x_n + x_{n−1})/2) when |Δ| < 1e−6 ([Parker, Zavalishin & Le Bivic, DAFx-16](https://dafx.de/paper-archive/details/vem_XXF5qBbfiWOH2RVVAA); [Vicanek](https://www.vicanek.de/articles/AADistortion.pdf)).
  - Asymmetric warmth: tanh(x + b) − tanh(b), b = 0.1–0.2, followed by a DC blocker.
- **Tape.** Model wow and flutter as a modulated delay. For pitch deviation δ at rate f_m, the delay amplitude is A = δ/(2π·f_m). Wow: 0.08% at 0.8 Hz gives A = 0.16 ms. Flutter: 0.03% at 8 Hz gives A = 6 µs, so interpolate with a cubic.
  - Wow is below 4 Hz and flutter above. Good machines measure about 0.02% weighted ([Wikipedia](https://en.wikipedia.org/wiki/Wow_and_flutter_measurement)).
  - Add a head bump (+1.5 dB bell at 70 Hz) and an HF shelf (−2 dB at 10 kHz) ([Chowdhury DAFx-19](https://dafx.de/paper-archive/details/Z5Ow_E5YHVxvndTAyeEuzA)).
  - Hiss: none, or −72 dBFS in music-only sections. **No vinyl crackle:** it is a stream of 2–8 kHz clicks.
- **Reverb.** Default is an 8-line FDN: Householder feedback y = x − (2/N)·Σx, an O(N) operation ([PASP](https://ccrma.stanford.edu/~jos/pasp/Householder_Feedback_Matrix.html)).
  - Per-line gain g_i = 10^(−3·M_i/(T60·fs)) ([PASP T60](https://ccrma.stanford.edu/~jos/pasp/Achieving_Desired_Reverberation_Times.html)), with a one-pole damping filter in each loop.
  - Mutually prime lengths of 1,427–3,373 samples, slow ±3-sample modulation, 4 input diffusion allpasses, 20–40 ms pre-delay, and 6 early-reflection taps.
  - T60: pads 2.5 s, mallets 1.6 s, bakery room 0.9 s.
  - Alternatives: the [Dattorro plate](https://ccrma.stanford.edu/~dattorro/EffectDesignPart1.pdf) for bells, or [Freeverb](https://ccrma.stanford.edu/~jos/pasp/Freeverb.html) (combs 1116…1617, allpasses 556/441/341/225, spread 23, at 44.1 kHz so scale by 48/44.1; [tuning.h](https://dosbox-x.com/doxygen/html/tuning_8h_source.html)).
- **Stereo.** Constant-power panning. Width comes from chorus and reverb decorrelation. No Haas delays on anything with 1–4 kHz content, because it combs in mono. Ping-pong delay (dotted eighth, feedback 0.35, LP 3 kHz in the loop) in gaps only.

### 3.5 WAV output (checked against `fmn-codec/src/wav.rs` and `fmn-output`)

- **Decoder:** reads u8, s16, s24, s32 and f32 (format tags 1 and 3, plus EXTENSIBLE), mono or stereo only. Mono is duplicated to stereo. Other sample rates are resampled with a 64-tap Blackman-windowed sinc.
- **Size cap:** 64 MiB per file (`MAX_SOUND_ASSET_BYTES`, `MAX_CUE_ASSET_BYTES`).
- **Recommendation:** write **32-bit float stereo at 48 kHz**: lossless, no dither needed, ≤174.8 s per file. Longer chapters use s24 (≤233 s) or split stems that abut on frame boundaries (at 60 fps one frame is exactly 800 samples).
- **What happens after the mix:** the soundtrack is clamped, **quantized to s16 with `DitherPolicy::None`** (`sinks.rs`), and AAC-encoded per chapter. `master.sh` then AAC-encodes again at 256 kb/s.
  - **Upstream ask 1:** switch that handoff to `DitherPolicy::Tpdf` (the API already exists). This follows [Lipshitz et al. 1992](https://hajim.rochester.edu/ece/sites/zduan/teaching/ece472/reading/Lipshitz_1992.pdf).
  - **Upstream ask 2:** master from concatenated PCM WAV soundtracks. Until then, music must be ≤ −50 dBFS within ±50 ms of every chapter cut, because AAC priming at the joins can click.

---

## 4. Musical design

### 4.1 What 3Blue1Brown's music is (verified only as far as sources allow)

- **Who:** "Almost all of it was written by Vincent Rubinetti." Grant wrote the piano pieces for the Linear Algebra and Calculus series ([3b1b FAQ](https://www.3blue1brown.com/faq)).
- **The album:** *The Music of 3Blue1Brown* on Bandcamp has 34 tracks in volumes I–IV. Tags are ambient, chamber, classical, electronic, orchestral, soundtrack and videogame ([Bandcamp](https://vincerubinetti.bandcamp.com/album/the-music-of-3blue1brown)).
- **Rubinetti's general toolset:** FL Studio, EWQL, Komplete (Massive, FM8, Absynth), Omnisphere, Zebra 2, Sylenth1 and a Yamaha piano ([studio page](https://vincentrubinetti.com/studio)).
- **Harmony in Grant's two published pieces** ([MIDI](https://vincentrubinetti.com/sheets/3blue1brown/)):
  - "Etude": Emaj7 with 6/9 colour, moving to C#m9 and back, about 110 BPM, about 26 s per chord.
  - "Opus": Dmaj7(add9) → F#m(add9) → Bm7(add11) → Amaj7(add9) → Dmaj7 with a G# (a Lydian raised 4th), in 3/4 at about 90 BPM, about 8 s per chord.
- **Not verifiable:** how the music is mixed under narration. No interview covers it. So "piano-led, maj7/add9, Lydian colour, slow harmonic rhythm" is backed up for Grant's pieces only. Rubinetti's catalogue spans ambient, electronic and orchestral.
- **Takeaway:** borrow the *grammar* (major-7th/added-9th colour, a Lydian raised 4th for wonder, chords lasting 8 s or more, piano plus soft synths), not any particular sound.

### 4.2 How documentaries score explanation

- The voice leads. Under speech the music holds still (pedals, sustained textures, slow harmonic motion) and plays out in the gaps ([Planet Earth II](https://www.spinmagazine.com/2017/02/planet-earth-ii-composers-talk-music-david-attenboroughs-voice-interview/); "sparse in the frequency ranges occupied by the human voice", [IRPR glossary](https://sounddesign.irpr.agency/glossary/underscore/)).
- Avoid big melodies under dialogue ([Thinkspace](https://thinkspace.ac.uk/blog/blog-how-to-underscore/)). Remove music entirely before a key line ([Berklee](https://online.berklee.edu/takenote/film-scoring-tips-from-5-classic-movies/)).
- Mickey-Mousing (music copying on-screen action) is "out of favor" but works for playful discrete events ([Wikipedia](https://en.wikipedia.org/wiki/Mickey_Mousing)). Here it means counting plucks, used sparingly.
- Kurzgesagt scores the finished picture, and sound effects carry much of the experience ([production video](https://www.lingq.com/fr/apprendre-anglais-en-ligne/courses/689474/how-to-make-a-kurzgesagt-video-in-120-4887128/)). That is our approach too: the timeline is known before we synthesize.

### 4.3 Generative but composed: a fixed per-video identity, with variation inside it

- **Identity (fixed per video):** key and mode, tempo (with ±8% elastic stretch so phrases land exactly on the next sentence or reveal), palette (bed / colour / cue instruments), and a 3–6-note leitmotif stored as **scale degrees**, so it moves with the harmony (transpose, invert, augment). Plus a seed.
- **Harmonic plan per chapter:** chords lasting 4–16 s.
  - Chord changes happen only in gaps or in pauses inside a sentence of 0.2 s or more, never within 150 ms after a speech onset.
  - A voice-leading solver minimizes total movement in semitones, subject to these constraints:
    - **bass in E1–D2**
    - **upper voices from A3 (220 Hz) upward**, so the voice's pitch band (100–200 Hz) stays empty
    - common tones held, not retriggered
    - moving voices glide 100 ms or crossfade
  - Use pedal points during dense derivations.
- **Intensity arc (0–1) per chapter**, driving density, filter cutoff, register spread and reverb send:
  1. Intro: the motif played in full at V−6, then down to bed level at the first line.
  2. Development: bed at density 1–2.
  3. Before each reveal: thin out or drop the bed 1 s ahead.
  4. Reveal cue in the gap.
  5. Main theorem: the fullest arrangement in a 4–8 s window.
  6. Outro: the motif "resolved" (rising, ending on the tonic) at V−4.

**Cue types** (each with at least 4 round-robin variants):

| Cue | Build | During speech? |
|---|---|---|
| Reveal stinger | Mallet or bell dyad drawn from the current chord, plus a reverb swell | Gap only, or ≤ −18 LU with low-pass |
| Transition swell | Pad filter opens over 1–2 s, plus a reversed reverb tail | Gaps / music-only |
| "Aha" resolution | Motif's final note resolves (e.g. 2̂→1̂ over IV→I) | Gap |
| Tick / counter | Two-mode "tok" at 600–900 Hz, 20–40 ms, ≤ V−20 LU momentary, panned ±40%. Pitch climbs a pentatonic ladder (more = higher, per [Walker's polarity work, Handbook ch. 2](https://sonification.de/handbook/)). Above 6 events/s it becomes a granular texture | Yes, quiet |
| Pizzicato count | Karplus–Strong on chord tones, −0.5 dB per hit (floor −6 dB) | Yes, sparse |
| Whoosh / riser | §3.3 | Whoosh yes (capped); riser no |
| Sonification | Value → pitch on a scale; reference tone at the maximum (shown to help in the [Handbook](https://sonification.de/handbook/)) | As designed |

### 4.4 Three proposals

**(A) Capture–recapture lake: playful, curious, watery**

- **Key and mode:** F major with Lydian colour (B♮) for curiosity, D Dorian for "uncertainty".
- **Metre:** 6/8, dotted-quarter = 56 (one bar = 2.14 s). Harmonic rhythm 2–4 bars.
- **Harmony:** Fmaj9 → Dm9 → B♭maj9(#11) → C7sus4 → F/A.
- **Palette:** Juno "water" pad (cutoff wandering by random walk, not a fixed LFO), **kalimba = unmarked fish**, **vibraphone with slow motor = tagged fish**, felt piano for the motif, sine-ish bass, bubbles. Lake ambience (low-passed pink noise at V−28) in intro and outro only.
- **Motif:** 5̂–1̂–2̂–3̂ (C–F–G–A), rising and left open. Lydian variant 5̂–1̂–#4̂–5̂. Resolves to 1̂ at the estimate reveal.
- **Cues:**
  1. Cast net: whoosh plus a low pizzicato thump.
  2. Catch M fish: a kalimba stream that turns into texture.
  3. Tag: one vibraphone note per tag.
  4. Release: descending bubble cluster.
  5. Recapture n fish: kalimba plinks, but **marked fish ring as vibraphone**, so the listener literally hears k. Each marked fish steps up the scale.
  6. Reveal N̂ = Mn/k: the motif on felt piano and vibraphone in a gap, resolving to Fmaj9.
  7. Hypergeometric simulation: each trial ticks at the pitch of its k. Thousands of trials become "rain" whose most frequent pitch is the distribution's mode, so the peak can be heard.
  8. Interval narrowing: the pad filter closes.
  9. Outro.

**(B) GF(256) and RaptorQ: crystalline but warm**

- **Key and mode:** E major with Lydian A#. **96 BPM**. 16th-note plucked arpeggios in gaps only; under speech they drop to 8ths with a darker filter.
- **Harmony:** Emaj9 → C#m9 → Amaj9(#11) → B6sus4. Dense algebra sits on an E pedal, alternating I ↔ II/I every 8 s.
- **Palette:** "warm glass" bell (FM 1:1 plus 1:2 at low index, low-passed; harmonic ratios rather than clangy ones), Juno PWM string pad, Karplus–Strong pluck, sub bass, tape wow on everything, a plate reverb, ping-pong delay on bells in gaps.
- **Byte sonification:** the 8 bits map to 8 slots, E4 F#4 G#4 B4 C#5 E5 F#5 G#5.
  - **Addition (XOR) = notes toggling.**
  - **Multiplying by α = every note steps up one slot.** When the top bit overflows, the reduction XORs in 0x1D, a little chord flick. RFC 6330's OCT_EXP table is consistent with the reduction polynomial x⁸+x⁴+x³+x²+1 ([RFC 6330](https://www.rfc-editor.org/rfc/rfc6330); inferred from the table).
- **Motif:** taken from the polynomial's low byte 0x1D = bits {0,2,3,4}, giving E–G#–B–C# (1̂–3̂–5̂–6̂).
- **Cues:**
  1. Packets arriving: glass droplets on pentatonic pitches, density proportional to packet rate.
  2. Lost packet: a dull tick low-passed at 500 Hz.
  3. Gaussian elimination: each pivot is a pluck stepping down; reaching the identity matrix is a cadence.
  4. Decoding: RFC 6330 §5.8 bounds failure at ≤1/100 with K' symbols, ≤1/10⁴ with K'+1, and ≤1/10⁶ with K'+2. Each extra symbol adds one chord tone, so the chord is complete at K'+2.

**(C) Lamport's bakery algorithm: warm, rhythmic, clockwork**

- **Key:** B♭ major. **104 BPM** with light swing (56%). Small-room reverb (T60 0.9 s, strong early reflections) for a bakery feel.
- **Harmony:** B♭maj7 → Gm9 → E♭maj9(#11) → F7sus4. The race-condition chapter adds whole-tone and augmented tension under the scenario, which resolves when `choosing[]` is introduced.
- **Each process is a timbre in its own register:** felt piano, marimba, pizzicato and FM electric piano.
- **Mapping:**
  - The `choosing` flag is a grace note.
  - The **ticket number is a scale degree** (later tickets are higher, so the queue order is audible).
  - Waiting is a soft repeated note on each beat.
  - **The critical section is that process playing the motif. Only one motif ever sounds at a time, so mutual exclusion is audible, and an overlap would sound like a bug.**
  - Equal tickets sound as a unison, then the lower id proceeds.
- **Motif:** "take a number" 1̂–2̂–3̂, then "served" 5̂–4̂–2̂–1̂.
- **Clockwork tick:** a soft "tok" (700 Hz, low-passed) in intro and gaps. Under speech the pulse is only a quiet bass pizzicato on beats 1 and 3.

---

## 5. Rules and automated QA

**Hard rules (rendering fails if violated):**

1. No sustained tones between 100 and 220 Hz during speech, except the bass below 100 Hz.
2. Under speech, music content above 2.5 kHz is cut by at least 12 dB (low-pass plus the pocket).
3. No pure sine voices above 100 Hz. At least 3 partials, or chorus or noise.
4. Minimum attack is 3 ms for mallets and 8 ms for plucks during speech. Nothing below 1 ms except deliberate ticks that are low-passed under 3 kHz.
5. Cue density:
   - no more than 1 foreground cue per sentence during speech
   - inter-onset interval of at least 1.5 s
   - no more than 6 cues per 30 s
   - streams faster than 6/s become textures
6. No identical cue buffer within 10 s. Use round-robin plus seeded micro-variation: ±1.5 dB, ±8 ms, ±10% decay, ±0.1 pan, ±15% mallet hardness. Pitched notes stay in tune; only timbre varies.
7. Cue pitches come from the current chord's pentatonic subset. Flag minor 9ths against any chord tone.
8. Levels per §2.2. No melodic phrase starts during speech (the [changing-state effect](https://orca.cardiff.ac.uk/46903/)).

**QA checks** (std-only, computed per chapter, written to a JSON report):

| Metric | Threshold |
|---|---|
| Voice minus music short-term loudness over speech regions (BS.1770 implemented in the crate) | median ≥ 16 LU, 5th percentile ≥ 12 LU |
| Music's loudest 400 ms during speech | ≤ speech's loudest 400 ms − 15 LU |
| SII-lite loss per 400 ms (third-octave bands, ANSI weights) | median ≤ 0.01, maximum ≤ 0.03 |
| Music-to-voice band ratio across 1–4 kHz during speech | ≤ −20 dB |
| Music stem true peak (4× oversampled) / click detector (onset energy above 8 kHz) | ≤ −12 dBTP / 0 flags |
| Monotony: chroma change rate | ≥ 1 chord change per 16 s; no 8 s segment repeated more than 3× with correlation > 0.98 |
| Busyness: music onsets per second during speech | ≤ 2 (≤ 6 in gaps) |
| Roughness and pitch collision, from the score itself | no partial pair 15–60 Hz apart, both above −20 dB, sustained; no beating pair with modulation depth > 6 dB on exposed tones |
| Mono compatibility | L/R correlation ≥ 0 averaged per 3 s |
| Sanity | no NaN or inf, DC ≤ −60 dBFS, length = scene samples exactly |

**Listening protocol:**
- Laptop speakers, earbuds, and headphones or monitors. Each at normal volume and 10 dB lower, plus a mono phone check.
- A/B against a no-music render on 3 random 20 s excerpts per chapter. Second listener blind.
- One uninterrupted full watch. Log every moment music was *noticed* during speech (target: none) and every gap that felt dead.

---

## 6. Reusable crate: `explainer_score` (std only)

```text
rng      SplitMix64/xoshiro; seed = fnv1a64(video, chapter, track, event_idx)   // never std DefaultHasher (unstable across Rust versions)
units    dB/LU/Hz/MIDI; frames↔samples exactly as fmn (round-half-away, 48k)
theory   scales/modes, chords as pitch-class sets, voice-leading solver, motif transforms
osc      polyBLEP saw/pulse/tri, sine, white/pink noise, OU drift
filter   TPT one-pole, Simper SVF, ZDF ladder, RBJ biquads, linear-phase FIR designer, DC blocker
env      exp ADSR, raised-cosine ramps, breakpoint automation
fx       adaa_tanh, juno/solina chorus, tape, fdn8/16, plate, ping-pong, M/S, pan
instr    Pad, Strings, FmEp, FeltPiano, Mallet{Marimba,Vibes,Kalimba,Glass}, Pluck, Bass, Whoosh, Riser, Bubble, Tick
score    Identity, ChapterPlan{harmony, intensity, cues}; anchors keyed to narration line ids/marks
timeline LinePlacement{id, start_frame, wav}, marks, fps, total frames
duck     VAD on narration WAVs → regions → duck/pocket curves (§2.3)
mix      buses (bed/colour/cue/sfx) → sends → duck+pocket → stem; BS.1770 meter, true peak
wav      f32/s24/s16+TPDF writer, 64 MiB guard, frame-aligned splitting
qa       §5 metrics → JSON; render fails on hard-rule violations
```

**Integration.**
- `Narrator::say` already knows (id, now, duration). It also records into `score.timeline`, and `score.mark(stage, "reveal_formula")` records authored events.
- At the end of `construct` (as `pad_under` does today), call `score.render_chapter()`. It writes `music_chNN.f32.wav` at exactly the scene's length and calls `add_sound(path, -secs, Some(0.0), None)`.
- One pass is enough: the narration WAVs exist before rendering, and the timeline is complete at the end of construct.
- Use two passes only if the animation must follow the music. Note that `add_sound` does nothing in skip mode, so pass 1 has to record through our own recorder.
- Cache stems by a hash of (identity, plan, timeline, narration file digests, crate version).

**Determinism.**
- Seeds are derived per event, so editing chapter 3 never changes chapter 1.
- f64 `sin`, `tan` and `exp` come from the platform's libm and can differ in the last bit between macOS and Linux. So golden tests compare **spectral fingerprints** (third-octave energy per second plus chroma, within ±0.5 dB), not byte hashes. Byte-exact output is required only on the same platform.

**Performance (measured).** My throwaway prototype rendered 10 minutes of 48 kHz stereo in **6.27 s of user CPU on one M4 Pro core**. It played 375 note-voices (up to 10 at once), each with 2 PolyBLEP saws, a sub oscillator and a saturated ZDF ladder, through chorus and an 8-line FDN, and wrote a 24-bit file.
- Wall-clock time was inflated by a machine load average of about 220.
- Making it faster:
  - render chapters and buses in parallel with `std::thread::scope`
  - update coefficients every 32 samples
  - use power-of-two ring buffers with bit masks instead of `%`
  - use a rational tanh
  - skip silent voices

**Tests.**
- **Unit tests:**
  - Aliasing: inharmonic energy ≤ −50 dB for f0 ≤ 2 kHz.
  - Ladder: −3 dB point within 5% of the requested cutoff, and stable for k < 4 at any cutoff up to 0.45 fs.
  - FDN: T60 within ±10% of the target (Schroeder backward integration).
  - Karplus–Strong: tuning within ±2 cents.
  - Loudness meter: matches the EBU test vectors.
- **Property tests:** no NaN, peak ≤ −1 dBFS, deterministic re-render, chapter independence.
- **Golden fingerprints:** for each example video.
- **QA gate:** runs on recorded narration fixtures, such as `explainer_corpus/hoeffding_d/narration_v5/`.

**Measurement scripts.** These are throwaway, not deliverables. All in `docs/research/sound_design/scripts/`:
- `voice.py`, `yin2.py`: robert's pitch and spectrum
- `current.py`, `sii.py`: loudness and masking of the current design
- `cue.py`: cue level offsets
- `duck.py`: narration gaps
- `proto/main.rs`: the Rust timing prototype
