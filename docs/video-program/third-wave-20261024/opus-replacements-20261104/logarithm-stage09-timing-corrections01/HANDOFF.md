# logarithm stage09: timing corrections for 04_2 b306 and 07_6 s2–s5, and the cue-edge reconciliation

BronzeJaguar, 2026-11-05, for root (BlueSalmon Mail 4383, bead 87do.63). These are proposals for root to adjudicate;
nothing here is applied.

P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`,
EL = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`,
O = `EL/production-audio-stage09-01`.

## What was done, and what was not

- **Read only:** Lavender's production-join-inspection01 F1–F3, the script text, the installed helper's own recognizer
  records, the PCM of every selected and alternative take involved, all 162 retained recordings' edges, the 80
  production WAVs, and fmn's mixer source.
- **Written:** only this folder and scratch.
- **Not done:**
  - no decode, assembly, render, ASR, TTS, reroll or job;
  - no production, audio or source-WAV edit;
  - nothing heard.
- **timeline01:** its inputs were neither touched nor read.
- **Limits of the evidence:**
  - levels and periodicity are measurements, not listening;
  - the recognizer words come from the installed helper (`54df274a`), so this is not independent of that recognizer;
  - the 10 ms leave values come from my own timing-inspection01 `measure()`, the same one Lavender used. The script
    asserts they equal hers. The 1 ms, sample-peak and voicing evidence is new.

## Supported corrections (selected takes)

| Point | Script at the point | Now | Proposed (10 ms leave) | Shift |
|---|---|---:|---:|---:|
| 04_2 b306 (mark) | "…which is positive, ‖ it only ever climbs," | 21.86 s (recognizer) | 530160 (22.09 s) | +0.23 s |
| 07_6 s2 | "…exists there too. ‖ But it jumps:" | 242880 (10.12 s) | 238080 (9.92 s) | −0.20 s |
| 07_6 s3 | "…close to minus pi. ‖ And the principal logarithm…" | 384000 (16.00 s) | 382080 (15.92 s) | −0.08 s |
| 07_6 s4 | "…from one side only. ‖ Exponentiating the logarithm of z…" | 503040 (20.96 s) | 499680 (20.82 s) | −0.14 s |
| 07_6 s5 | "…for every nonzero z. ‖ But the logarithm of e to the w…" | 636000 (26.50 s) | 634800 (26.45 s) | −0.05 s |

Every row passes all eight checks in `outputs/points.tsv`:

- **Script:** the point opens a sentence after "." (07_6) or the phrase after "," (04_2). No script word lies between
  the previous word and the point.
- **Recognizer:** its onset abuts the previous word. It is off the leave by at least 0.05 s.
  - In 07_6 the previous word absorbs the pause: "too" 1.32 s, "pi" 1.36 s, "only" 1.34 s, "z" 1.12 s.
  - The squeezed word that follows is implausibly short: "but" 0.06 s, "and" 0.04 s.
  - In 04_2, "positive" ends at 21.86 and "it" runs 21.86–22.28 s. So the recognizer gave "it" the comma pause.
- **Pause, every 1 ms window and every sample:**
  - no 1 ms window above −55.7 dB, and no sample above −51.0 dBFS;
  - 1 ms medians −84.7 to −85.5 dB in 07_6 and −64.4 dB in 04_2;
  - each pause lasts at least 0.10 s.
- **Speech level after the pause:** the first 1 ms window at or above −40 dBFS comes 0.009–0.105 s after leave.
  - In 07_6 that is at 9.966, 15.945, 20.829 and 26.460 s.
  - These are 0.154, 0.055, 0.131 and 0.040 s **before** the recognizer's word.
  - At the recognizer's onset the 10 ms level is −18.5 to −22.4 dB, which is mid-word.
  - In 04_2 the recognizer's onset sits at −59.3 dB, in "positive,"'s tail.
- **Voicing:** the largest normalized autocorrelation (70–300 Hz) in 40 ms windows over the first 100 ms of speech is
  0.84–0.996, at 119–132 Hz. The exceptions are s4 at 216 Hz and download02's s2 at 211 Hz, probably octave errors of
  the same voice; only r is used.
- **Other take:** its recognizer is off in the same direction:
  - 04_2 download02: −0.21 s;
  - 07_6 download02: s2 +0.17, s3 +0.09, s4 +0.15, s5 +0.04 s.

  This is the recognizer's pattern on this voice, not a take defect.
- **Consistent with the words, not proof of them:**
  - **s4:** voiced sound runs 20.82–20.91 s, then a dip of about 17 ms (−38 to −53 dB at 1 ms), then sound again from 20.93 s,
    rising into the recognizer's word at 20.96. That fits "Ex-" /ɛks/ before the recognizer's start.
  - **s2:** a soft rise (−72 to −50 dB at 1 ms) runs 9.911–9.96 s, then speech level 9.966–10.15 s, then a dip to
    −50 to −56 dB at 10.153–10.161 s, before the recognizer's "it" at 10.18. That fits "But" with its /t/.

**Size.** s2 (12 frames at 60 fps) and s4 (8 frames) are the substantive changes; s3 (5 frames) and s5 (3 frames) are
small. All four are in one cue, so applying s3 and s5 adds no cost to s2 and s4. Root may still leave them out.

## 04_2 b306: leave or body (open; needs hearing)

- **The lead-in after the pause is weak and aperiodic.**
  - 10 ms levels: −59.1 dB at 22.09, −49.9 at 22.13, back to −59.9 at 22.15, −44.7 at 22.18.
  - Autocorrelation 0.25–0.45, apart from one 0.61 at −60 dB; zero-crossing rate 0.05–0.21.
- **Speech-level voiced sound** (r 0.79–0.95 at about 132 Hz) starts at 22.18–22.195 s. The 10 ms body is 22.19 s.
- **What energy cannot tell.** The first 0.10 s after leave could be a breath, a lip sound, or a breathy start of "it".
- **The two candidates:**
  - **Proposed: 530160 (22.09 s, leave).** This follows the convention of the 92 applied answers, including my seven
    blip answers, which are at leave, 0.12–0.19 s before body.
  - **Alternative: 532560 (22.19 s, body),** if hearing finds that the lead-in is not the word.

  Either one ends the reveal's 0.23 s lead over the next sound.

## Not proposed

- **07_6 s5 in download02:** its recognizer is 0.04 s after leave, under the 0.05 s rule, so it is left unanswered. It
  matters only if 07_6 is swapped. `answers02.json` carries download02 answers for 04_2 and 07_6 s2–s4, used only
  then.
- **07_6 s1 and s6:** already answered (39840, 840720) and unchanged.
- **Lavender's F4–F9:** outside Mail 4383. I examined none of them.

## Why the earlier reports said what they said (owned)

**production-choices01 (mine).** HANDOFF line 56 reads: "Tails are 0.12–0.32 s, and speech starts 0.01–0.10 s in."

- **What that measured:** `screen()` (`choose_takes.py:104–123`), the time after the last 10 ms frame at or above
  −40 dBFS. Re-measured here, it is still 0.12–0.32 s for all 162 recordings (`edges-recordings.tsv`, screen
  columns, asserted equal to the screen).
- **What it is not:** a silence measure. The handoff did not name the threshold, so it read as "at least 0.12 s of
  quiet at each end". That is my error.
- **Why the flag missed these:** the edge flag fired only for a frame of −40 dBFS or louder in the very first or last
  10 ms. A file whose last frame rises from −82.6 to −45.3 dB, like 07_6, passed.
- **Measured to the last 10 ms frame above −60 dBFS,** tails are 0.00–0.20 s.
  - 10 of 162 recordings end with their last 1 ms above −60 dB. Selected: 01_2, 07_6, 11_5, 12_2; alternatives:
    00_5, 02_1, 03_3, 07_2, 07_3, 10_4 download02.
  - 6 start with sound above −60 dB in their first 1 ms or first 10 ms frame. Selected: 00_5, 04_3, 05_5, 09_4;
    alternatives: 06_4 and 08_2 download01.
- **What the sentence should have said:** "at least 0.12 s after the last body-level frame; 10 recordings end, and 6
  start, on low-level sound."
- **05_5 adds to Lavender's F3.** Its first 10 ms frame is −60.1 dB, just under her threshold. But its first 1 ms is
  −59.1 dB, and its first 48 kHz bus sample, at −57.6 dBFS, is the largest start step of the 80.

**timing-inspection01 (mine).** It looked only at sentence boundaries, never at file edges.

- **04_2 b306:** its quiet run is 0.14 s, under the 0.15 s pause floor. So it became `short_quiet_run` and was never
  classified.
- **07_6 s2–s5:** classed "within", because body lies only 0.16, 0.05, 0.13 and 0.04 s before the recognizer onset.
  That is under its 0.2 s material threshold.
- **The result:** a switch after the body went unflagged whenever the body was less than 0.2 s ahead. Lavender's F2
  (and F5) catch exactly that case.

## Alternative takes at the edges (read only)

| Cue | Edge | Selected | Other take |
|---|---|---|---|
| 07_6 | end | dl01: last 1 ms −37.4 dB, samples −441, −505, −311 (48 kHz) | dl02: −92.2 dB, −60 tail 0.11 s |
| 11_5 | end | dl02: −47.0 dB, samples 373, 433, 267 | dl01: −94.1 dB, tail 0.10 s |
| 01_2 | end | dl01: −49.3 dB, samples 185, 217, 134 | dl02: −91.9 dB, tail 0.14 s |
| 12_2 | end | dl01: −51.5 dB, samples 83, 85, 51 | dl02: −91.4 dB, tail 0.16 s |
| 04_3 | start | dl01: first 1 ms −47.2 dB, samples −3, −36, −47 | dl02: −79.0 dB, lead 0.02 s |
| 00_5 | start | dl01: −74.4 dB, first 10 ms −50.8 dB | dl02: starts quiet (−85.5), but **ends** at −33.6 dB (samples 1132, 1258, 761) |
| 09_4 | start | dl02: −86.7 dB, first 10 ms −54.0 dB | dl01: −84.7 dB, lead 0.03 s |
| 05_5 | start | dl01: −59.1 dB, samples −30, −50, −48 | dl02: −86.7 dB, lead 0.04 s |

Seven of these alternatives have a quiet edge. 00_5's alternative trades its start for a worse end.

A swap is a selection change, root's selection01. It needs a new decode and prepare, and its own timing answers. It
also changes the reading itself, and no reading has been compared by ear.

## What the mixer does at a cue edge

- **Source:** fmn-output `sound.rs` at a399d626, sha256 `7bb983c0…`. These are the same bytes as cargo's checkout. The
  script checks the hash and eight mirrored lines.
- **No edge treatment:** no fade, envelope, taper or declick.
- **Resampling:** 24 to 48 kHz by a 64-tap Blackman-windowed sinc, normalized by its weight sum.
- **Placement:** at the nearest sample. After a cue's last sample the voice bus is 0.
- **Mirror on the 80 production WAVs (`edges-production.tsv`), last 48 kHz bus sample before the step to 0:**

  | Cue | End step |
  |---|---:|
  | 07_6 | −38.1 dBFS (−432, −420, −406 / 32768) |
  | 11_5 | −39.5 dBFS |
  | 01_2 | −45.5 dBFS |
  | 12_2 | −53.9 dBFS |
  | next largest | −79.1 dBFS |

- **Start steps:** 05_5 −57.6, 08_3 −62.0, 04_3 −63.5, 00_5 −68.7 dBFS. 09_4's first bus sample is −84.3 dBFS, because
  its sound rises inside the first 10 ms.
- **The sound before the end:** in each of the four end cues it starts inside the last 10–12 ms, with the same frames
  in the 48 kHz source (Lavender F2/F3).
- **A likely cause (inference only):** every one of the 162 lengths is a whole multiple of 3840 samples at 48 kHz
  (80 ms). That fits a provider that returns whole 80 ms blocks, cutting off a sound that starts in the last one.
- **What follows each end cue:** 07_6 ends chapter 07; 01_2, 11_5 and 12_2 are followed by 01_3, 11_6 and 12_3.
  Whether the next cue follows at once or after a hold, and what the score does there, is in the timeline. I did not
  read it.
- **No answer can remove an edge.** The installed helper ends each recording's last sentence at its full length
  (`prepare_audio.py:802`, `ends = [...] + [frames24]`, "whole recordings"), and the mixer has no fade. So the options
  are:
  - accept it after a listen;
  - swap the take (above);
  - trim or fade in code, which would mean a helper, assembler or kit change. That is not proposed here.

## Hearing uncertainties (root or the owner)

1. Whether the end steps (07_6 and 11_5 near −38 to −40 dBFS, 01_2 −45, 12_2 −54) and the 05_5 and 04_3 starts
   are heard, as a click or a cut-off sound, under the score.
2. What the rising sound in the last 10 ms of 07_6, 11_5, 01_2 and 12_2 is: a breath, the start of a sound, or noise.
3. 04_2 b306: does the lead-in from 22.09 to 22.18 s belong to "it"? This decides leave (530160) or body (532560).
4. Whether the 07_6 s3 (0.08 s) and s5 (0.05 s) shifts are perceptible at all.
5. Whether the sound before each 07_6 recognizer word is the script's word ("But", "And", "Ex-"). Energy and voicing
   are consistent with it. Nothing was heard.
6. Style: whether a sentence's caption or beat should land at leave, the convention so far, or later.

## Route to apply later (root runs every step; nothing here ran them)

Everything stays as for prepared01 except the answers:

- binding `2d7f4731`;
- `O/selection01.json` `a5c4dfa1`;
- `O/decoded01` (decode receipt `4e442a6c`);
- helper `54df274a`;
- reviewer BlueSalmon, whole film, production.

No source or decoded WAV changes.

```
PA=P/logarithm-stage09-audio-preparation02/prepare_audio.py
B=P/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json
T=P/logarithm-stage09-timing-corrections01
python3 -I -B $PA --binding $B prepare --selection O/selection01.json --answers $T/inputs/answers02.json \
    --decoded O/decoded01 --reviewer BlueSalmon --out O/prepared02
cmp O/prepared02/spans.json $T/outputs/predicted-spans.json
cmp O/prepared02/phrase-marks-plan.json $T/outputs/predicted-phrase-marks-plan.json
```

(Spell P and O out in full.)

1. **Prepare.** The two `cmp`s should pass. On prepared01's own inputs, the writer reproduces prepared01's spans and
   plan byte for byte. Against prepared01, the spans differ only in 07_6's four sentence ends, and the plan only in
   04_2 b306.
2. **Assemble** with `O/prepared02/assemble-command.json`'s `argv`.
   - **Expected WAVs:** all 80 equal prepared01's. Their hashes are in `validation.json`; `cmp` each pair.
   - **Expected report:** only 07_6's rows change, to 0–1.66, 1.66–9.92, 9.92–15.92, 15.92–20.82, 20.82–26.45,
     26.45–35.03 and 35.03–38.88 s.
   - **Checked here, in process:** unit_spans gives all 80 cues the same PCM under both span sets.
3. **Timings** with its `then` argv, into `O/prepared02-timings`. Install `phrase-timings.json` into
   `O/prepared02/narration` as for prepared01. Expected: only 04_2 b306 changes, to 22.09 s.
4. **Render preparation (Ruby's v3, `aa645803`):**
   - `check-audio --audio <absolute O/prepared02>`;
   - `bind` with a new attempt;
   - then a new **series 02**: `logarithm-stage09-timeline02`, score, 13 renders, master.

   Why a new series:
   - timeline01 binds prepared01's report and phrase timings, so it cannot carry these corrections;
   - v3's job names are `{stage}-{step}{series}`;
   - the timeline step covers all 13 chapters, although only chapters 04 and 07 change.

**The choice for root:** apply before the final render (one more full timeline, score and render series), or record
these for a later pass and keep timeline01.

## Files

- **`timing_corrections.py`:** the whole check.
  - **Run:** `python3 -I -B P/logarithm-stage09-timing-corrections01/timing_corrections.py FRESH_OUT --fmn-sound SOUND_RS`,
    from the repository root.
  - **SOUND_RS:** `crates/fmn-output/src/sound.rs` at a399d626. For example, `git show a399d626:crates/fmn-output/src/sound.rs`
    in a franken_manim checkout, written to a scratch file. Cargo's git checkout `franken_manim-6641d0e09642f63e/a399d62`
    has the same bytes.
  - **Pins:** 3 seals (Lavender `e05ea847`, timing-inspection01 `8c18a243`, production-choices01 `357613e5`), file by
    file, plus 13 inputs, including all of prepared01's receipts.
  - **Exit codes:** refuses with exit 1 on any change. About 30 s.
- **`inputs/answers02.json`** (`075c83bd…`): production-choices01's `answers.json` plus 9 sample answers, 5 selected and
  4 alternative. Each basis states its measurements and "not heard". The helper applies 97 (92 before).
- **`outputs/`:**
  - **`points.tsv` and `points.json`:** the ten points, with script text, the recognizer's ±2 words, the pause, speech
    onset, lead-in, voicing, checks and proposals.
  - **`edges-recordings.tsv`:** all 162 recordings.
  - **`edges-production.tsv`:** the 80 production WAVs and their bus edges.
  - **`validation.json`:** the helper's acceptance, the 5 changes, the dry run (80 cues), the WAV hashes and the
    predicted rows.
  - **`predicted-spans.json` and `predicted-phrase-marks-plan.json`.**
  - **`stdout.txt`.**
- **Reproduction:** two fresh runs of this script were byte-equal on every output, and both confirmed the bundled
  answers equal the regenerated file. The dry run used a scratch copy of prepared01's workspace, with its pins checked.
- **Sweep:** commit 35aab47 (not mine) captured an earlier draft of `timing_corrections.py`. This seal is
  authoritative.

**Not claimed:** listening, viewing, fine sync, acceptance or independence from the helper's recognizer.
