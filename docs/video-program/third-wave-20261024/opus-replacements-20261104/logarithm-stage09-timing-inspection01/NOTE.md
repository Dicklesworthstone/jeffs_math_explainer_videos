# Stage09 timing inspection

This answers root's Mail 3960: "inspect remaining actual timing questions with the already-recorded root PCM provenance check". It also gives the timing recommendations that Mail 4018 asks for. Bead .63.

**These are sample-level energy measurements only.** Nothing was heard. Nothing was synthesized, transcribed, decoded, assembled, built or rendered. No selection, answers file or patch was written, and no canonical, stage, root or sealed file was edited. Every recording was read through Python's `wave` module, and nothing was written next to it.

Paths:
- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- EL = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`
- Stage09 = `P/logarithm-stage09-audio-preparation01`, sealed `2bb8e280…` (Mail 3994). Both scripts here verify that seal first and exit 3 if any byte has changed.

## Summary for root

1. **Provenance (00_1).** All four files agree with the inventory, the report and your check `EL/stage09-provenance-root-check01.json` (`ee03130d…`) on frames, PCM hash and file hash. download03 and download04 have the same samples as download02.
   - **Recommendation:** select download01 or download02, as your check's decision prefers. Each was recognized directly, and the two provenance questions then do not apply. If you select download03 or download04, the only allowed answer is `use_pcm_identity`, and your check is its basis.
2. **Early onsets.** 181 recognizer points start at least 0.2 s before the recording's energy leaves the pause. That is 180 of the 570 sentence onsets plus 1 of the 66 marks, spread over 67 recordings of 41 cues.
   - **113 of the 180 onsets do not abut the previous word**, so the report's `abuts_previous_word` flag misses them. In those rows the recognizer leaves a gap after the previous word (median 0.14 s), but the actual pause starts where that word ends (median 0.03 s earlier). The onset therefore still lands inside the pause.
   - The largest is 12_21 download01 sentence 1, at 0.73 s: the file sits at about −85 dBFS from 12.36 to 13.21 s, and the recognizer's onset is at 12.50 s.
3. **Recommendation for the early points:** for each selected recording, give `sample` answers at `leave_24k` (in `outputs/timing-rows.tsv`) for its `recognizer_early` rows, and keep the recognizer's values elsewhere. One exception: for 08_5 download02 sentence 2, a value near the vowel (about 167040) is better than its `leave_24k` (see the clarity section).
   - `checks/whatif_onsets.py` shows that these answers keep the sentence starts strictly increasing.
   - They also keep every mark inside its own sentence, under the L1 rule of Mail 4018. The smallest margin is 1.56 s, and the largest move is 0.73 s.
   - Every early point can be corrected this way, so the per-cue counts below are not a reason to prefer one take over another.
   - The note does not decide whether the change is needed. That depends on how early a picture may start before its sentence, which is root's decision.
4. **Clarity takes.**
   - **04_4:** all 10 onsets of both takes fall within 0.14 s of the energy bracket, so no change is suggested.
   - **08_5:** 4 of 5 onsets are early in each take. download01 sentence 1 ("here") is the one found earlier: 0.59 s.
   - The four 08_5 mid-sentence marks cannot be checked by this method.
5. **Not measured here** (the recognizer's values stand, unverified):
   - 43 marks have no pause before them. This method can only find onsets that follow a pause.
   - 10 points have only a short quiet run (under 0.15 s): 9 marks and 1 sentence onset.

## What an onset is used for

In `prepare`, the helper writes each selected recording's sentence starts (`sentence_starts_24k`, each with its basis) and its marks. The audio itself is not cut: "No samples removed, no silence inserted". So an early onset changes only timing. The next sentence's start lands inside the pause before it, up to 0.73 s before its first word. This note claims nothing about how the kit or the film uses that start.

## Method (`inspect_timing.py`, version 5)

The script measures 10 ms RMS frames of each recording's own samples (48 kHz, 480 samples per frame). Frame k is k/100 s, or k × 240 samples at 24 kHz. Every supported sentence onset (after the file start) and every supported mark is measured.

- **Window:** from 0.3 s before the recognizer's start to the end of its word segment.
- **Pause:** the longest run of frames at or below −60 dBFS, at least 0.05 s long, that ends inside the window. A run is measured in full, even if it begins up to 1.5 s before the window.
- **Bracket:**
  - `leave` is the first frame above −60 dBFS after the pause.
  - `body` is the first frame at or above −40 dBFS after the pause.
  - The word's energy begins within [leave, body] to frame resolution. A soft attack can start a frame or two earlier, below −60 dBFS.
- **Classes:**
  - `recognizer_early`: leave ≥ recognizer + 0.2 s.
  - `energy_before_recognizer`: body ≤ recognizer − 0.2 s. No rows fall in this class.
  - `within`: everything else.
  - A pause shorter than 0.15 s is reported as `short_quiet_run` and not classified. It may be a stop closure, or a floor that crosses −60 dBFS.
- **Two extra fields per row:**
  - `median_db_recognizer_to_leave`: the median level between the recognizer's onset and leave.
  - `pause_start_minus_recognizer_s`: where the measured pause starts, relative to the recognizer's onset.

### Versions (all kept in `superseded/`)

| version | change | why replaced |
|---|---|---|
| v1 | last quiet run; step back over every frame above −75 dBFS; abutting rows plus the clarity takes only | Probe 2: the 04_4 download01 pause sits between −66 and −75 dBFS, and v1 stepped back across all of it, giving 13.05 s against a real rise at 13.35 s. Probe 4: in 08_5 download02 it took a 70 ms dip after a small blip as the pause. |
| v2 | longest run ending in the window; bracket [leave, body]; 0.3 s lead for every row | measured only the abutting rows and the clarity takes |
| v3 | every supported point | — |
| v4 | adds the loudest frame from the recognizer's onset to leave | that maximum lands on the attack frames just before leave, not on the pause |
| v5 | median level instead of the maximum | delivered |

v5's measurement is the same as v2's. On every measured field, the v2 run agrees with v5 on all 501 rows they share, and the v3 and v4 runs agree on all 636 (checked from the scratch runs; the superseded scripts reproduce them).

## Results

### Provenance

| file | frames (48 kHz) | file sha256 | PCM sha256 | inventory | report | root check | recognized from |
|---|---|---|---|---|---|---|---|
| 00_1-download01.wav | 936960 | 919ee92f… | f12f719d… | agrees | agrees | agrees | itself |
| 00_1-download02.wav | 936960 | 781147c2… | f4a2dcff… | agrees | agrees | agrees | itself |
| 00_1-download03.wav | 936960 | 4f150e90… | f4a2dcff… | agrees | agrees | agrees | download02 |
| 00_1-download04.wav | 936960 | b3cc09a7… | f4a2dcff… | agrees | agrees | agrees | download02 |

Root's check (`2026-11-05T04:38:18Z`), quoted: "Recognition of download02 may be reused for byte-identical PCM download03 and download04; prefer directly recognized original rather than duplicate for assembly." Its limits: "No inference about number of provider generations, voice identity, listening, or fine sync." The file hashes differ but the PCM hashes match, which is consistent with your older record `EL/ui-takes01/00_1-pcm-identity01.json`: "repeated downloads contain metadata differences".

### Points by group

| group | rows | recognizer_early | within | short quiet run | no pause found |
|---|---|---|---|---|---|
| sentence onsets, abutting | 431 | 67 | 363 | 1 | 0 |
| sentence onsets, not abutting | 139 | 113 | 26 | 0 | 0 |
| marks, abutting | 64 | 0 | 13 | 9 | 42 |
| marks, not abutting | 2 | 1 | 0 | 0 | 1 |

The 431 abutting onsets match the report's count. `delta_leave_s` (leave minus the recognizer's onset), for rows with a pause:

| group | n | min | median | p90 | max |
|---|---|---|---|---|---|
| sentence onsets, abutting | 430 | −0.20 | 0.03 | 0.34 | 0.64 |
| sentence onsets, not abutting | 139 | −0.17 | 0.33 | 0.47 | 0.73 |
| marks, abutting | 13 | −0.12 | 0.00 | 0.04 | 0.12 |

The most negative delta is −0.20 s (07_6 download01 sentence 2, where body is −0.16 s), so no row reaches `energy_before_recognizer`. 08_2 download01 sentence 1 is the one short-quiet-run onset. Probe 9 shows a floor between −56 and −64 dBFS, with the recognizer's 4.44 s inside the rise at 4.36–4.51 s.

### The 181 early points

- **By delta_leave:** 44 at 0.2–0.3 s, 79 at 0.3–0.4 s, 41 at 0.4–0.5 s, 17 at 0.5 s or more.
- **Level between the recognizer's onset and leave:**
  - 157 have a median at or below −75 dBFS, near the silence floor. The plain-silence pauses probed (probes 1, 5 and 6) sit at −84 to −90 dBFS. The probes were chosen to include problem cases, and the probed files' p05 levels run from −70 to −85 dBFS.
  - 24 sit on a floor of −63 to −73 dBFS. That is quiet but not digital silence, and might be a breath or noise; it is not classified.
  - Probe 8 shows one: 08_2 download02 sentence 1 hovers between −58 and −75 dBFS (mostly −60 to −67) from 3.85 to 4.25 s, and its vowel starts at 4.44 s.
- **Five rows have a positive `pause_start_minus_recognizer_s`** (0.01 to 0.11 s). Probe 7 shows the largest, 00_6 download01 sentence 4. The previous word has decayed to −68 dBFS by 22.00 s, then the floor touches −59 dBFS at 22.08 and 22.12 s, which splits the quiet run. The recognizer's 22.02 s is in the pause, not in the previous word.

The 15 largest (all on a silence floor):

| point | recognizer 24k | leave 24k | body 24k | delta leave s | pause s |
|---|---|---|---|---|---|
| onset:12_21:s1:12_21-download01.wav | 300000 | 317520 | 318240 | 0.73 | 0.92 |
| onset:00_6:s2:00_6-download01.wav | 259200 | 274560 | 274800 | 0.64 | 0.68 |
| onset:08_1:s2:08_1-download02.wav | 172800 | 187440 | 187440 | 0.61 | 0.83 |
| onset:08_5:s1:…/oliver-clarity-takes01/08_5-download01.wav | 39360 | 53520 | 54240 | 0.59 | 0.67 |
| onset:11_8:s1:11_8-download01.wav | 99360 | 113280 | 114000 | 0.58 | 0.79 |
| onset:12_3:s2:12_3-download01.wav | 301440 | 315120 | 315360 | 0.57 | 0.80 |
| onset:07_4:s2:07_4-download01.wav | 298080 | 311520 | 311520 | 0.56 | 0.62 |
| onset:08_5:s3:…/oliver-clarity-takes01/08_5-download01.wav | 353280 | 366720 | 367200 | 0.56 | 0.63 |
| onset:06_2:s3:06_2-download02.wav | 384000 | 397200 | 397440 | 0.55 | 0.81 |
| onset:07_2:s1:07_2-download01.wav | 170880 | 184080 | 184560 | 0.55 | 0.60 |
| onset:00_6:s5:00_6-download01.wav | 636000 | 648960 | 649200 | 0.54 | 0.65 |
| onset:07_6:s1:07_6-download02.wav | 31200 | 44160 | 44880 | 0.54 | 0.67 |
| onset:06_6:s2:06_6-download02.wav | 255360 | 268080 | 268320 | 0.53 | 0.56 |
| onset:07_2:s1:07_2-download02.wav | 170880 | 183600 | 183840 | 0.53 | 0.58 |
| onset:06_20:s2:06_20-download01.wav | 292320 | 304800 | 305040 | 0.52 | 0.59 |

All 181 are in `outputs/timing-inspection.json` under `summary.recognizer_early`. Every row is in `outputs/timing-rows.tsv`. The full ids of the clarity points carry `EL/combined-repair-stage09/oliver-clarity-takes01/<file>`.

### Early sentence onsets per cue (early / sentence onsets after the first; * = clarity take)

| cue | download01 | download02 | | cue | download01 | download02 |
|---|---|---|---|---|---|---|
| 00_4 | 2/3 | 2/3 | | 06_5 | 0/2 | 2/2 |
| 00_5 | 0/3 | 1/3 | | 06_6 | 4/4 | 4/4 |
| 00_6 | 5/5 | 0/5 | | 06_7 | 5/6 | 5/6 |
| 01_1 | 4/4 | 3/4 | | 07_2 | 3/3 | 3/3 |
| 01_3 | 1/3 | 3/3 | | 07_4 | 2/2 | 2/2 |
| 01_4 | 0/4 | 2/4 | | 07_6 | 2/6 | 1/6 |
| 02_2 | 0/3 | 3/3 | | 08_1 | 6/6 | 6/6 |
| 02_3 | 1/3 | 1/3 | | 08_2 | 0/3 | 3/3 |
| 02_5 | 1/5 | 1/5 | | 08_5 | 4/5* | 4/5* |
| 03_2 | 2/4 | 0/4 | | 09_2 | 4/4 | 4/4 |
| 04_5 | 3/4 | 0/4 | | 09_3 | 2/2 | 2/2 |
| 04_6 | 1/2 | 1/2 | | 10_1 | 1/3 | 1/3 |
| 04_7 | 3/5 | 0/5 | | 10_4 | 2/4 | 4/4 |
| 05_20 | 0/4 | 3/4 | | 11_1 | 4/4 | 0/4 |
| 05_4 | 0/2 | 2/2 | | 11_20 | 1/1 | 1/1 |
| 05_5 | 1/1 | 1/1 | | 11_4 | 5/5 | 5/5 |
| 05_6 | 4/4 | 4/4 | | 11_8 | 3/3 | 0/3 |
| 06_1 | 2/2 | 2/2 | | 12_20 | 0/2 | 1/2 |
| 06_2 | 3/3 | 3/3 | | 12_21 | 1/1 | 1/1 |
| 06_20 | 2/2 | 2/2 | | 12_3 | 3/3 | 0/3 |
| 06_3 | 5/5 | 5/5 | | | | |

The other 39 cues have no early onset in either take, 00_1 among them. The one early mark is `anchor:05_20:b225:05_20-download02.wav` ("the"): the recognizer gives 19.40 s and the energy rises at 19.69 s, with the body at 19.81 s.

### Clarity takes (`EL/combined-repair-stage09/oliver-clarity-takes01`)

| take | point | word | recognizer s | leave s | body s | delta leave s | class |
|---|---|---|---|---|---|---|---|
| 04_4 dl01 | s1 | reflecting | 2.80 | 2.82 | 2.92 | 0.02 | within |
| 04_4 dl01 | s2 | where | 6.54 | 6.62 | 6.64 | 0.08 | within |
| 04_4 dl01 | s3 | at | 13.34 | 13.35 | 13.44 | 0.01 | within |
| 04_4 dl01 | s4 | so | 16.84 | 16.70 | 16.93 | −0.14 | within |
| 04_4 dl01 | s5 | a | 23.46 | 23.54 | 23.56 | 0.08 | within |
| 04_4 dl02 | s1 | reflecting | 2.80 | 2.79 | 2.92 | −0.01 | within |
| 04_4 dl02 | s2 | where | 6.24 | 6.31 | 6.33 | 0.07 | within |
| 04_4 dl02 | s3 | at | 12.92 | 12.99 | 13.02 | 0.07 | within |
| 04_4 dl02 | s4 | so | 16.36 | 16.42 | 16.44 | 0.06 | within |
| 04_4 dl02 | s5 | a | 23.00 | 23.05 | 23.07 | 0.05 | within |
| 08_5 dl01 | s1 | here | 1.64 | 2.23 | 2.26 | 0.59 | early |
| 08_5 dl01 | s2 | each | 6.70 | 7.09 | 7.10 | 0.39 | early |
| 08_5 dl01 | s3 | but | 14.72 | 15.28 | 15.30 | 0.56 | early |
| 08_5 dl01 | s4 | taking | 20.36 | 20.75 | 20.77 | 0.39 | early |
| 08_5 dl01 | s5 | the | 27.52 | 27.52 | 27.54 | 0.00 | within |
| 08_5 dl01 | mark | leaves / a | 22.42 / 25.38 | — | — | — | no pause |
| 08_5 dl02 | s1 | here | 2.16 | 2.10 | 2.11 | −0.06 | within |
| 08_5 dl02 | s2 | each | 6.50 | 6.86 | 6.99 | 0.36 | early |
| 08_5 dl02 | s3 | but | 14.82 | 15.28 | 15.29 | 0.46 | early |
| 08_5 dl02 | s4 | taking | 20.32 | 20.69 | 20.71 | 0.37 | early |
| 08_5 dl02 | s5 | the | 27.06 | 27.41 | 27.44 | 0.35 | early |
| 08_5 dl02 | mark | leaves / a | 22.32 / 25.30 | — | — | — | no pause |

- **04_4 dl01 s4 "so":** the /s/ frication runs from about 16.72 s at −42 dBFS (probe 3), so the recognizer's 16.84 s is inside the bracket.
- **08_5 dl02 s2 "each":** leave (6.86 s) is a small blip after the 0.50 s pause; the vowel rises at 6.97–6.99 s (probe 4). For this row a sample near 6.96 s (167040) fits the word better than `leave_24k` 164640.
- **04_4 marks:** the 04_4 takes have no marks.

### What-if: the recommended answers and the L1 rule (`checks/whatif_onsets.py`)

The script uses the sealed helper's own `read_script` and `kit_sentences` to map each mark to its sentence. It checks two point sets: the report's current points, and the same points with all 181 early points moved to `leave_24k`.

| set | order problems | marks outside their sentence | smallest margin of a mark to its sentence bounds |
|---|---|---|---|
| current | 0 | 0 | 1.56 s (05_20 dl02 b276, before the next start) |
| adjusted | 0 | 0 | 1.56 s (same mark); 1.59 s after a moved start (12_21 dl01 b192) |

- **Planted controls:** a start moved one sample past a mark (04_2 dl01 b306) and a start equal to the one before it (00_2 dl01) are both reported. The script exits 1 if either one is missed.
- **Lavender's L1 fix:** these numbers say what the L1 rule of Mail 4018 would find on both sets. The fix itself is Lavender's work. I wrote no patch.

## Limits

- **What the measurement can't tell:** energy levels are not listening. A level cannot say whether a sound is a breath, a click or a word. The thresholds are fixed, while the files' silence floors differ (p05 from about −70 to −85 dBFS). Resolution is 10 ms.
- **Unchecked points:** onsets of words that follow no pause cannot be measured this way. That covers the 43 marks reported as no pause.
- **Unused class:** the `energy_before_recognizer` class would need the previous word's start, which the report does not carry. No row fell in it.
- **Not independent:** I wrote the stage09 helper and its report.
- **What was read:** all reads were read-only: the sealed bundle, 162 selectable recordings, the four 00_1 files, `EL/full80-retained-inventory01.json` and root's check.

## Replay (from the repository root; H = `P/logarithm-stage09-timing-inspection01`)

```
(cd H && shasum -a 256 -c SHA256SUMS)
python3 -I -B H/inspect_timing.py NEW_SCRATCH_A        # timing-inspection.json 7d195199…, timing-rows.tsv 5a7e84d8…
python3 -I -B H/checks/whatif_onsets.py NEW_SCRATCH_B  # whatif.json e7d2bbfc…
python3 -I -B H/checks/energy_probe.py WAV START_S END_S
```

Both scripts write only into a folder that must not exist yet: exit 2 if it does, exit 3 if a pinned input changed. Two runs of each gave byte-identical output and stdout. Two controls were run on copies:
- A one-byte change to a scratch copy of the stage09 bundle gave exit 3, `STALE: … report/questions.json`, and no output folder.
- An existing output folder gave exit 2.

## Files

| file | sha256 |
|---|---|
| inspect_timing.py (v5) | 5d4661b0e58c800da548c0b3710a6983e2465703cf3e7fabf89b4f91d32425e3 |
| checks/whatif_onsets.py | b271fe62560265a27bc2a9d8a650bbc11e1025b7ad699cfa7731f2754a35c67b |
| checks/energy_probe.py | 918bc1bb03343df909f081c1b0b8892208fe2606e3fcbb4b0f24eef749cdaef5 |
| outputs/timing-inspection.json | 7d19519935c529227b42d2baf94f5614c841061526e4be7b2b9f8896b86a81de |
| outputs/timing-rows.tsv | 5a7e84d86b672a2672d2ff7210855db55acd14b7bd6ffc47d06bf4f4d676babe |
| outputs/inspect_timing-01.stdout | b2bb73bd50191dcd876e393e495a1df91fec550a500e9b51bb9d3ed21577e099 |
| outputs/whatif.json | e7d2bbfc3b70f5641b7c8fb2f2918f859ffd6197d111f79bd0b1bf38e4887577 |
| outputs/whatif_onsets-01.stdout | 93559497959c30bc6c437186184a97a2768013606cafb208b9af62f2cd8145c0 |
| outputs/probes-01.txt (9 probes, numbered as cited above) | 041d273f24622dc9bc45c6fb053375705dfe7e01bf2b822771f833049323a275 |
| superseded/inspect_timing-v1.py … v4.py, whatif_onsets-v1.py | in SHA256SUMS |
