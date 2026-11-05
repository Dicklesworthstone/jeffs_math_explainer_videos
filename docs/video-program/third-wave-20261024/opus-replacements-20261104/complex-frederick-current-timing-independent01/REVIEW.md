# complex-frederick-current-timing-independent01

RubyRaven, for BlueSalmon Mail 4474 (bead 87do.39), 2026-11-05. This is an independent review of WhiteSwan's sealed `P/complex-frederick-current-timing-review01/` (SHA256SUMS `42f34a75…`).

**What was not done.**
- Nothing was heard, synthesized, recognized, cut, decoded with ffmpeg or launched. No job or guard ran.
- No production, take, lane, root, Beads, TODO or WhiteSwan file was written. No take was selected.
- The logarithm timeline and its inputs were not touched.

**What was read.**
- WhiteSwan's four files, after checking his seal.
- The 36 lane files he pinned. They are unchanged on the live lane; I copied them into scratch.
- The repaired bridge `47069583`, imported in-process. Its `report` builder ran on the copy.
- The 94 takes' 48 kHz PCM, read with Python's `wave`.
- The guard jobs' fw records, located through the bridge's job and index binding, hash-checked.
- `targeted-small01`.
- After the review script was finished: the 08_2 lines of root's `targeted-small02` (§5).

## Verdict

**I found no wrong candidate.** All 61 timing answers and all 13 `proceed` dispositions hold on the evidence available.

| WhiteSwan's item | Verdict |
|---|---|
| 60 `sample` answers at the pause end | **Confirmed.** Each sits before the first sound by my own envelope, never inside the word (§1) |
| 05_4 dl02 `use_candidate` (10.44 s) | **Confirmed.** The candidate is in the last 30 ms of the pause (§2) |
| 05_4 dl01 `sample` 254760 | **Confirmed** (§2) |
| 11 notation/same-sound `proceed` | **Defensible** (§5) |
| 01_3 ×2 `x^40` `proceed` | **Defensible, with one residual.** The notation erases exactly the "-th" of "fortieth" (§5) |
| 02_4 dl02 `term`, open | **Agree.** Hearing only |
| 02_5, 08_2 held | **Agree.** The durations and root's small02 favour the script's length for 08_2 (§5) |
| 191 in-pause onsets and 03_4 dl02 s1, not proposed | **Agree.** These are a policy choice; 03_4 is the extreme case (§4) |

**Optional consistency additions:** six onsets his rule missed (§3). Each lags its first sound by 0.045–0.055 s, as much as his five smallest answered points. They are optional provisional samples. With them, `resolve()` still accepts 94/94.

**Mark 03_2 b90 "doubles":** my own rule flagged it. Voicing does not support the flag, so it is withdrawn (§4).

## Method, and what it shares with WhiteSwan's

| | WhiteSwan | Mine |
|---|---|---|
| Level | 5 ms frames | 10 ms RMS at a 1 ms hop |
| Pause | ≥ 0.06 s at or below −70 dBFS, the **last** such pause | ≥ 0.05 s at or below −60 dBFS, the **longest** such pause |
| Start of sound | the pause end | the run end ("leave") |
| Body | first frame ≥ −30 dBFS | end of the first 10 ms window ≥ −40 dBFS |
| Coverage | 62 late points, plus classes | all 322 sentence onsets and recognized marks of the 94 takes, plus threshold sweeps from −50 to −75 dBFS |
| Extra | — | voicing (normalized autocorrelation, 75–300 Hz pitch, 30 ms windows) at the points in §3 and §4 |

**Independence.**
- My envelope code is my own and shares no code with his `timing_review.py`.
- Both reviews rest on the same recognizer timestamps and the same bridge alignment: which segment is which script word, and the sentence slots. This review is not independent of those.
- I wrote the checks myself, so they are not independent of me.

## 1. The 60 sample answers

His 62 late points are 59 supported onsets, the 05_4 dl01 candidate and the 2 held 02_5 points.

**Placement against my envelope:**
- **His sample is never after my leave.** My leave is 2–22 ms after his pause end at every point. 45 of the 60 are within 10 ms; the other 15 differ by 11–22 ms, with his always earlier.
- **Class:** 61 of the 62 are `inside_word` by my rule too. The exception is `onset:04_1:s5:04_1-download02.wav`, which I class `at_onset`: 0.050 s after his pause end and 0.035 s after mine. His sample still sits at the first sound, so it does no harm.
- **The pause choice is unambiguous:**
  - The chosen run is both the longest and the last at −60 dBFS.
  - His rule re-run on my 1 ms envelope moves his pause end by at most 11 ms.
  - Across thresholds from −50 to −75 dBFS, the leave moves by at most 39 ms. The extremes are 07_6 s3 dl02 "from", a soft /f/, and 06_2 s3 dl02 at 35 ms.
- **One stretch of sound:**
  - Pauses are at least 0.36 s (median 0.508 s).
  - The rise from leave to −40 dBFS takes at most 0.043 s.
  - No quiet run of 20 ms or more at or below −60 dBFS lies between leave and the recognizer start.
- **His samples:** each equals `round(pause_end × 24000)`. The peak in the 5 ms before each sample is −84.3 to −58.9 dBFS. That makes this the conservative boundary.

**How late the recognizer is.** The lag is the recognizer start minus his pause end:
- min 0.045 s, median 0.12 s, max 0.51 s;
- 42 of the 60 are at least 0.1 s late, and 20 at least 0.2 s.

At 60 fps, 0.045 s is about 3 frames.

**The sound before each recognizer start is about one word long.** The longest are:

| Point | Word | Sound before the recognizer start |
|---|---|---|
| 03_5 s3 dl01 | "inside" | 0.498 s |
| 02_4 s2 dl02 | "multiply" | 0.402 s |
| 01_6 s1 dl02 | "complex" | 0.385 s |
| 01_6 s1 dl01 | "complex" | 0.371 s |
| 02_6 s2 dl02 | "lengths" | 0.353 s |

That fits the point's own word by duration. It is not content proof.

**The bridge's own `resolve()`:**
- with his 61 timing answers: 94/94 takes;
- with no answers: 92/94. Both 05_4 takes give "no onset".

**What these answers move.** The bridge writes `gap_after_samples` 0 for every sentence end (`frederick_timing_bridge.py:1226`), and the measured starts feed `film_starts` (`:1228–1229`). So, as far as the bridge goes, an onset answer moves where the film places a sentence. I did not trace the assembler beyond that.

## 2. The two 05_4 timing questions

- **dl02, `use_candidate`:** the recognizer's "Sign" is at 10.44 s. My pause is [10.107, 10.469] s, so the candidate is 29 ms before my leave and 15 ms before his pause end.
  - The 20 ms trace reads −86 dB at 10.44 and −59 at 10.46.
  - A `sample` at his pause end, 250920, would differ by 15 ms. Either is defensible.
- **dl01, `sample` 254760 (10.615 s):** my leave is 10.622 s. The trace reads −84 dB at 10.60 and −54 at 10.62. Confirmed.

## 3. Onsets his rule missed (optional, provisional)

**Why they were missed.** His late test measures from the body (−30 dBFS). Measured from his own pause end instead, these onsets lag as much as his smallest answered points:

| His answered point | Lag |
|---|---|
| 01_1 s4 dl02 | 0.045 s |
| 04_1 s5 dl02 | 0.050 s |
| 01_6 s2 dl02, 06_1 s1 dl01, 06_1 s1 dl02 | 0.055 s |

These six he classed `at_onset`:

| Point | Word | His pause end / mine | Recognizer | Lag | Voicing r at +10/+20/+30 ms | Provisional sample |
|---|---|---|---|---|---|---|
| `onset:01_1:s1:01_1-download01.wav` | one | 2.865 / 2.870 | 2.92 | 0.055 | 0.91 0.93 0.94 | 68760 |
| `onset:03_5:s2:03_5-download01.wav` | the | 7.505 / 7.511 | 7.56 | 0.055 | 0.65 0.84 0.90 | 180120 |
| `onset:03_7:s1:03_7-download02.wav` | the | 3.835 / 3.853 | 3.88 | 0.045 | 0.74 0.88 0.92 | 92040 |
| `onset:04_1:s1:04_1-download02.wav` | to | 3.330 / 3.336 | 3.38 | 0.050 | 0.18 0.15 0.50 | 79920 |
| `onset:04_3:s2:04_3-download01.wav` | and | 6.825 / 6.831 | 6.88 | 0.055 | 0.82 0.77 0.83 | 163800 |
| `onset:05_6:s2:05_6-download01.wav` | it | 6.355 / 6.366 | 6.40 | 0.045 | 0.88 0.81 0.81 | 152520 |
| `onset:08_2:s1:08_2-download01.wav` (held) | for | 4.545 / 4.557 | 4.60 | 0.055 | 0.17 0.15 0.16 | 109080 |
| `onset:08_2:s1:08_2-download02.wav` (held) | for | 4.600 / 4.620 | 4.70 | 0.100 | 0.17 0.14 0.12 | 110400 |

**The first sound is voice.**
- In the four vowel- or voiced-initial words ("one", "the", "and", "it"), the sound is periodic (r ≥ 0.65) within 10–20 ms of his pause end. So the first sound is voice, not a breath.
- "to" (/t/ burst) and "for" (/f/) are aperiodic, as their first consonants are. Voicing cannot tell an /f/ from a breath.
- root's small02 also starts "For" late: 4.62 s (dl01) and 4.68 s (dl02).

**Size and use.** These are 45–55 ms, about 3 frames; 08_2 s1 dl02 is 0.10 s.
- The provisional sample is `round(his pause end × 24000)`, his own convention.
- With the six non-held ones added to his 61, `resolve()` accepts 94/94.
- Root can add them for consistency, or treat lags under about 0.06 s as below what matters. In that case his five answered points at or under 0.055 s become optional too.

## 4. Points not proposed

**The 191 in-pause onsets.** I agree they cut no word, and how early a sentence's visuals may start is root's policy. My classes for all 322 points are:

| Class | Points |
|---|---|
| `in_pause` | 208 |
| `at_onset` | 24 |
| `inside_word` | 66 |
| `no_pause` | 24 |

There are 47 class disagreements with WhiteSwan:
- **32 marks** he left `not_assessed`: all are `in_pause` or `no_pause` by mine, except 03_2 b90 dl02 (below).
- **15 onsets:** the four §3 cases my rule calls `inside_word`, 04_1 s5, the two 03_4 takes, and eight that are `in_pause` by mine against his `at_onset`.

None of them changes a verdict.

**`onset:03_4:s1:03_4-download02.wav` "Every".**
- The recognizer's 0.64 s lies in the decay of "No.": −61 to −64 dBFS over 0.62–0.70. The level falls to −83 by 0.86, and the voice comes in at 0.92 (−21).
- My −60 rule puts 0.64 at the start of the pause [0.635, 0.917]. His −70 rule calls it `before_pause`.
- Either way no word is cut. It is the most extreme early case, about 0.28 s ahead of the voice.
- A voice-onset policy would use his 0.89 s (21360) or my 0.917 s (22008).

**Mark `anchor:03_2:b90` "doubles", withdrawn.**
- **What my rule saw:** in dl02 it classed the mark `inside_word`, with a quiet run [6.624, 6.697] and the recognizer at 6.78.
- **Voicing does not support that.** In dl02:
  - the recognizer's "and" [6.44–6.78] begins with 0.2 s of aperiodic sound (r 0.11–0.22, −37 to −51 dB), consistent with the /θ/ of "length";
  - then comes a short gap;
  - then voice over 6.68–6.84 (r 0.56–0.96);
  - then a quiet closure over 6.86–6.90 (−63 to −72);
  - and voice again from 6.92.
- **dl01 has the same order:** aperiodic 6.54–6.70, voiced 6.82–6.96, closure 6.98–7.04, voiced from 7.06.
- **On that reading,** the first voiced burst is "and" and the closure is the /d/ of "doubles":
  - dl01's 7.00 sits on the closure;
  - dl02's 6.78 is about 0.08 s early, inside "and".
- **Conclusion:** the flag was a false positive of my own rule, and no correction is proposed. Voicing is not content proof; it only removes the case for "late".

**The 33 points he did not assess.** My rule finds no other `inside_word` among them.

## 5. Content dispositions

Each verdict below rests on the raw segment text from the fw records. That is a reading of text, not of audio.

| Question | Raw text at the difference | Verdict |
|---|---|---|
| 01_3 dl01, dl02 | `x^40` [11.28–12.22] / [11.30–12.24] for "x to the fortieth" | **Defensible `proceed`, one residual.** "x to the forty" would also be written `x^40`, so the "-th" is not evidenced. Both are acceptable readings of x⁴⁰, so the consequence is small. One listen settles it |
| 01_6 dl01, dl02 | `routes` [9.26–9.56] / [9.42–9.68] for "roots" | **Defensible.** In British English, /ruːts/ is the same sound. The spelling is uninformative either way; that a British voice would not say /raʊts/ is a prior, not evidence |
| 03_2 dl01 | `1/2.`, `1/4,` | **Defensible.** "a half" or "one over two" would be written the same way |
| 04_7 dl01, dl02 | `-i,` | **Defensible.** "negative i" would be written the same way |
| 05_4 dl01, dl02 | `Sign`, `sign` (three per take) for sine | **Defensible.** A true homophone |
| 06_4 dl01, dl02 | `1+i`, `1+i,`, `2+2i.` / `2 + 2i.` | **Defensible.** Notation |
| 07_2 dl01, dl02 | `non-zero` | **Defensible.** Spelling only |
| 02_4 dl02 | `term.` [14.60–15.19] for turn | **Agree: open.** Both recognizers also write `term` in 02_5. In a non-rhotic voice, "turn" and "term" differ only in the final nasal |

**Held 02_5 (duration only).**
- **Where small.en's words fall:** its "1 plus i" segments, [0.84–1.74] and [8.22–9.26] in dl01, lie inside continuous sound. Turbo's neighbouring words absorb those intervals: "makes" runs 1.58 s and "squared?" 2.09 s.
- **Speaking rates:**
  - The one unaffected phrase runs at 4.7–4.8 syllables per second.
  - The first stretch runs at 3.9 syl/s with the script's 15 syllables, or 3.1 with turbo's 12.
  - The last stretch runs at 2.9 with the script's 8, or 1.8 with turbo's 5.
- **Reading:** this is consistent with more being spoken than turbo wrote. It is weaker than for 08_2, because both readings are slower than the unaffected phrase; question-final "squared?" and the comma may account for that.

**Held 08_2 (duration, plus root's small02).**
- **Speaking rates:**
  - "For f of z equals z," fills [4.55–6.30] s in dl01 and [4.61–6.34] s in dl02. With the script's 7 syllables that is 4.0 syl/s in both takes, within the range of the other phrases (4.0–5.3 in dl01, 4.2–4.3 in dl02).
  - Turbo's `For f = z,` (5 syllables, reading `=` as "equals") would be 2.9 syl/s, slower than any other phrase in either take.
- **root's small02** (`command.log` `ef73ebdb…`, Mail 4527; I read it after sealing the script, and the script does not use it):
  - dl01: `[4.62-4.70]For [4.70-4.94]f [4.94-5.04]of [5.04-5.32]z [5.32-5.68]equals [5.68-6.46]z,`
  - dl02: `[4.68-4.76]For [4.76-5.48]f(z) [5.48-5.78]= [5.78-6.60]z,`
- **Reading:** the durations and both small02 texts agree with the script's length. None of this says what was spoken.

The syllable counts are mine, counted by hand with a British reading ("eff", "zed"), and are written in the script.

## Newer evidence outside this review

- Not reviewed: plan-10 and plan-11 (the 8 + 8 newer takes), small03 (08_3) and small04 (09_1), per Mails 4527, 4541 and 4550.
- The only newer record I read is the two 08_2 lines of small02 above.
- The 94-take review was not redone.

## Limits

- Levels and voicing are not listening. They cannot say which word was spoken, or tell an /f/ or /h/ from a breath.
- The thresholds are fixed constants in the script. Frames are 10 ms at a 1 ms hop, so a leave can be off by a few ms.
- No AV, sync, picture or acceptance claim is made.
- **Sweep:** commits `cee49ec` and `8a934bf` (not mine) captured earlier drafts of `checks/independent_review.py`. The seal here is authoritative.

## Replay

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-current-timing-independent01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/checks/independent_review.py NEW_ABSOLUTE_SCRATCH_DIR
```

**Running it:**
- Run it from the repository root. It writes only `SCRATCH/lane` (the copy) and `SCRATCH/review.json`.
- It exits 1 with a `FAIL` line if WhiteSwan's seal, the bridge, a take, a `command.log` or a live lane file moved.

**Two runs into fresh folders (10 and 11)** gave byte-equal transcripts and byte-equal `review.json`. Their outputs are the files here.

**When replaying today:**
- The live lane now holds ingest-10 and ingest-11 (Mail 4541).
- The pin fails only if one of the 36 files WhiteSwan bound changes. The newer ingests change only the counts printed on line 2 (`ingest-01..ingest-09 (9); plans 10` at the time of runs 10 and 11).
- The bridge runs on the copy of the 36 files, so every later line should still equal `checks/outputs/independent_review.out`.

## Files

| File | Purpose |
|---|---|
| `REVIEW.md` | this review |
| `checks/independent_review.py` | the measurements, the bridge in-process, `resolve()` and the held-cue durations |
| `checks/outputs/independent_review.out` | transcript of run 10 (equal to run 11) |
| `review.json` | all 322 points: my pause, body, class and WhiteSwan's, plus the six optional samples |
| `SHA256SUMS` | these files |
