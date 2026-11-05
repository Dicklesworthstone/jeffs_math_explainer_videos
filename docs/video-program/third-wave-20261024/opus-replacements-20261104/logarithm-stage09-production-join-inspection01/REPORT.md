# logarithm stage09: production-audio join inspection

LavenderPike, 2026-11-05, for root Mail 4247. Read-only.

This is a read-only inspection of `EL/production-audio-stage09-01`: the prepared narration, the decoded source and the
timings receipt. It uses sample counts and 10 ms RMS levels only. Nothing was heard. No boundary, mark, take, hold or
fine sync is accepted or rejected here. Each finding gives the cue, the sample at 24 kHz and the time. Root decides
what, if anything, to change.

`EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`.

## Chain: clean

80 cues, 284 internal sentence boundaries and 33 phrase marks. Problems: none. Input digest `c3c96e450a9f…`.

- Every `prepared01/narration/<cue>.wav` equals `decoded01/decoded/<cue>.wav` sample for sample (24 kHz mono 16-bit).
- Every decoded length is ⌈n/2⌉ of its 48 kHz recording.
- `report.json` sentences equal the kit's split of the script. Its spans run back to back over the whole take, with
  `gap_after_samples` 0 and `pause_cut_s` 0. So a boundary is a caption and beat switch, not an audio splice, and no
  silence is inserted anywhere.
- The boundaries are equal in `prepare-receipt.json`, `spans.json` and `report.json`.
- The installed `phrase-timings.json` equals the timings job's copy and binds this prepare receipt and each WAV.

Where the boundaries come from:

| Basis | Count | Where they sit |
|---|---:|---|
| BronzeJaguar's answers | 92 | 91 exactly at the leave frame; 1 is 0.01 s before it |
| Recognizer onsets | 192 | 146 in the pause (0.01–0.19 s before leave); 46 at or after leave (14 exactly at it; up to 0.20 s after) |

All 33 marks are recognizer onsets, each at exactly the previous word's recognizer end.

## Findings, most concrete first

### F1. 04_2 mark b306 "it only ever climbs," sits 0.23 s before the next sound

- The mark is at 21.86 s (sample 524640). It is the recognizer onset of "it", at exactly the previous word's end.
- The level there is −59.3 dB, the tail of "positive,".
- A quiet run follows from 21.95 to 22.09 s. The next sound leaves at 22.09 s (sample 530160) and reaches body level
  at 22.19 s.
- So the phrase reveal would lead the next sound by 0.23 s, and the body by 0.33 s.
- This is the only mark at or past Bronze's 0.2 s threshold. It is a candidate for an answer at the leave, 530160.

### F2. 07_6: the four recognizer boundaries trail the voice, and the take ends on a rising sound

Boundaries s2–s5 all sit after the next sentence's body. The loudest frame before each switch is −13 to −17 dB,
which is speech level in this file. The two answered boundaries, s1 (39840, 1.66 s) and s6 (840720, 35.03 s), sit
exactly at leave.

| Join | Sample | Time | First word | Leave (sample) | Body | After leave | Loudest frame before the switch |
|---|---:|---:|---|---|---:|---:|---:|
| 07_6 s2 | 242880 | 10.12 | But | 9.92 (238080) | 9.96 | 0.20 | −15.5 dB |
| 07_6 s3 | 384000 | 16.00 | And | 15.92 (382080) | 15.95 | 0.08 | −16.6 dB |
| 07_6 s4 | 503040 | 20.96 | Exponentiating | 20.82 (499680) | 20.83 | 0.14 | −14.0 dB |
| 07_6 s5 | 636000 | 26.50 | But | 26.45 (634800) | 26.46 | 0.05 | −13.1 dB |

- At s2, frames 9.96–10.14 s run from −15 to −26 dB before the recognizer's "But" at 10.12 s.
- The take's tail is 0.00 s. The last 20 ms rise from −89.6 dB to −37.5 dB in the final millisecond. The last samples
  are −374, −429 and −420; the largest absolute sample in the last 20 ms is 584.
- The 48 kHz source has the same last two frames (−82.6, −45.3 dB), so the decode did not cause this.

### F3. Cue edges: four takes end on a rising sound; three start with sound in the first 10 ms

All seven match the 48 kHz source's edge frames to within 0.1 dB. Each take stops, or starts, during a sound. Whether this is heard
depends on how the assembly follows or precedes each cue: silence, a fade or the next cue. That was not checked here.

| Cue | Edge | 1 ms levels at the edge | Largest abs sample in 20 ms | Edge samples |
|---|---|---|---:|---|
| 07_6 | end | −77 dB at −11 ms → −43.6, −41.5, −37.5 in the last 3 ms | 584 | −374, −429, −420 |
| 11_5 | end | −73 at −12 ms → −51.6, −50.5, −47.3 | 360 | 262, 348, 360 |
| 01_2 | end | −80 at −11 ms → −52.5, −51.4, −49.4 | 239 | 126, 173, 179 |
| 12_2 | end | −75 at −10 ms → −55.1, −55.6, −51.4 | 162 | 127, 101, 70 |
| 04_3 | start | −47.3 dB in the first ms (sound at sample 0) | 710 | −22, −36, −4 |
| 00_5 | start | −74 to −72 for 3 ms, −60.1 at 4 ms, −44.7 at 7 ms | 1429 | −12, −12, −7 |
| 09_4 | start | −87 at 0 ms, −62 at 3 ms, −50 at 7 ms | 315 | −2, 0, −1 |

Across all 80 cues:

- leads are 0.00–0.08 s and tails 0.00–0.20 s;
- peaks are −4.61 to −0.37 dBFS, with no full-scale samples;
- the cues total 1800.80 s.

### F4. 00_5 s1 has the shortest pause of all 284 boundaries

| Join | Sample | Time | Basis | Pause | Boundary position | Body |
|---|---:|---:|---|---|---|---:|
| 00_5 s1 "So each one…" | 126720 | 5.28 | recognizer | 5.02–5.20 (0.18 s) | 0.08 s after leave, −47.8 dB, in the attack | 5.38 |
| 00_5 s2 "The note…" | 243840 | 10.16 | recognizer | 9.84–10.25 (0.41 s) | in the pause, 0.09 s before leave | 10.27 |
| 00_5 s3 "Equal steps…" | 357120 | 14.88 | recognizer | 14.49–14.93 (0.44 s) | in the pause, 0.05 s before leave | 14.95 |

- The next shortest pause is 0.27 s, at 09_1 s2.
- At s1 the caption switch is in the soft onset of "So", 0.10 s before body. This finding is about the 0.18 s gap
  between sentences in the take, not about where the switch is placed.
- 00_5 also starts within 4 ms of its first sample (F3).
- Its tail of 0.20 s ties 12_20's for the longest of the 80 cues. Its 0.29 s from the last body frame to the end is
  the longest alone.

### F5. The other late boundaries: switch after the next sentence's body

| Join | Sample | Time | First word | Leave (sample) | Body | After body | Loudest frame before the switch |
|---|---:|---:|---|---|---:|---:|---:|
| 00_3 s1 | 66720 | 2.78 | count | 2.74 (65760) | 2.74 | 0.04 | −23.3 dB |
| 00_4 s1 | 42720 | 1.78 | start | 1.69 (40560) | 1.70 | 0.08 | −20.2 dB |
| 01_3 s2 | 97920 | 4.08 | it | 4.00 (96000) | 4.03 | 0.05 | −14.3 dB |
| 01_3 s3 | 233760 | 9.74 | but | 9.65 (231600) | 9.67 | 0.07 | −15.4 dB |
| 01_4 s1 | 56640 | 2.36 | twelve | 2.29 (54960) | 2.31 | 0.05 | −15.0 dB |
| 03_2 s2 | 151680 | 6.32 | six | 6.29 (150960) | 6.30 | 0.02 | −29.6 dB |
| 08_4 s1 | 85440 | 3.56 | minus | 3.54 (84960) | 3.55 | 0.01 | −34.9 dB |
| 12_20 s2 | 451200 | 18.80 | so | 18.68 (448320) | 18.73 | 0.07 | −18.6 dB |

- All are 0.02–0.12 s after leave.
- Bronze's timing-inspection01 classed every one of them "within".
- Six of them, and all four 07_6 rows, have speech-level energy before the switch.

### F6. Early boundaries: switch 0.10 s or more before the next sound

All are in the pause, all are under Bronze's 0.2 s threshold, and Bronze classed all of them "within".

| Join | Sample | Time | First word | Pause | Leave (sample) | Body | Before leave |
|---|---:|---:|---|---|---|---:|---:|
| 03_2 s3 | 228000 | 9.50 | together | 9.03–9.60 (0.57) | 9.60 (230400) | 9.60 | 0.10 |
| 03_2 s4 | 398400 | 16.60 | the | 16.19–16.71 (0.52) | 16.71 (401040) | 16.74 | 0.11 |
| 05_7 s1 | 70080 | 2.92 | its | 2.67–3.03 (0.36) | 3.03 (72720) | 3.06 | 0.11 |
| 05_7 s2 | 280320 | 11.68 | only | 11.30–11.78 (0.48) | 11.78 (282720) | 11.80 | 0.10 |
| 06_5 s1 | 109440 | 4.56 | what | 4.39–4.75 (0.36) | 4.75 (114000) | 4.87 | 0.19 |
| 06_5 s2 | 209760 | 8.74 | that | 8.59–8.91 (0.32) | 8.91 (213840) | 9.16 | 0.17 |
| 09_1 s4 | 410880 | 17.12 | the | 16.64–17.22 (0.58) | 17.22 (413280) | 17.24 | 0.10 |
| 10_4 s1 | 144000 | 6.00 | whenever | 5.66–6.14 (0.48) | 6.14 (147360) | 6.16 | 0.14 |
| 10_4 s2 | 360000 | 15.00 | it | 14.58–15.12 (0.54) | 15.12 (362880) | 15.15 | 0.12 |

At 06_5 the switches lead the body by 0.31 s (s1) and 0.42 s (s2), because the sound after leave stays soft for 0.12 s
and 0.25 s.

### F7. Marks that sit ahead of the next sound, by less than 0.2 s

| Mark | Time | Phrase | Where it sits | Next leave (sample) | Ahead of leave |
|---|---:|---|---|---|---:|
| 11_7 b404 | 28.56 | only when it does | at −22.3 dB, the end of the previous word; 0.08 s quiet run 28.65–28.73 | 28.73 (689520) | 0.17 |
| 11_7 b174 | 12.82 | so nothing forces a | at −53.7 dB, the decaying "equal,"; 0.12 s quiet run 12.86–12.98 | 12.98 (311520) | 0.16 |
| 10_3 b302 | 21.46 | or does not turn | in a 0.19 s pause, 21.39–21.58 | 21.58 (517920) | 0.12 |

- 12_21 b230 "the honest answer is" (17.78 s) has no quiet run within 0.30 s; the nearest is 16.88–17.33, before
  "Here,". It sits in a 0.10 s dip (−40 to −51 dB, 17.75–17.84) between two stretches of speech. Energy is consistent
  with a word junction there; no anomaly is claimed.

Seven marks sit within 0.11 s of the next sound's leave:

- 02_2 b47 (3.50 s, 0.04 s before leave);
- 05_20 b225 (19.50, 0.01 before);
- 11_5 b178 (15.32, 0.05 before);
- 11_7 b315 (23.38, 0.11 after leave, before body);
- 11_7 b453 (31.80, 0.05 after);
- 12_1 b167 (12.50, 0.04 after, in a blip);
- 12_6 b105 (9.08, 0.03 after).

### F8. Energy cannot place 21 marks

No quiet run lies within ±0.5 s for 19 of them, so they are in continuous speech. The other two, 11_6 b273 (21.66 s)
and 11_7 b226 (16.38 s), sit 0.09–0.10 s into speech that follows a pause before the previous word ("and"). The dip
is the lowest 10 ms frame within ±0.10 s; a deep dip near a mark may be a word junction or a stop closure.

| Mark | Time | Level | Dip (offset) |
|---|---:|---:|---|
| 02_1 b48 | 3.52 | −20.3 | −36.4 (−0.10) |
| 03_3 b35 | 2.14 | −21.8 | −42.9 (+0.09) |
| 05_20 b180 | 15.58 | −20.6 | −28.0 (−0.09) |
| 05_20 b214 | 18.02 | −19.8 | −43.5 (−0.09) |
| 05_20 b276 | 23.70 | −21.3 | −25.1 (+0.02) |
| 06_6 b267 | 18.44 | −17.4 | −58.2 (+0.10) |
| 06_6 b338 | 23.24 | −19.0 | −33.3 (−0.10) |
| 08_5 b267 | 22.42 | −22.4 | −52.4 (+0.04) |
| 08_5 b306 | 25.38 | −21.3 | −38.7 (−0.10) |
| 10_3 b208 | 15.06 | −20.6 | −59.7 (+0.09) |
| 10_4 b39 | 2.76 | −20.2 | −23.5 (+0.10) |
| 11_1 b320 | 21.74 | −18.7 | −31.9 (−0.01) |
| 11_1 b337 | 22.64 | −28.7 | −58.2 (+0.07) |
| 11_2 b185 | 12.84 | −22.6 | −26.6 (−0.06) |
| 11_5 b72 | 5.46 | −21.7 | −24.4 (−0.09) |
| 11_20 b91 | 6.16 | −45.7 | −62.3 (+0.02) |
| 11_20 b137 | 8.72 | −27.4 | −45.0 (−0.02) |
| 12_6 b89 | 7.38 | −21.5 | −24.8 (+0.10) |
| 12_21 b192 | 14.82 | −49.6 | −51.6 (+0.02) |
| 11_6 b273 | 21.66 | −23.1 | −35.8 (−0.10) |
| 11_7 b226 | 16.38 | −23.0 | −47.9 (−0.10) |

### F9. Blip boundaries: a short sound, then quiet again, before the body

- **03_1 s3** (374400, 15.60 s, "one"): pause 15.14–15.54; sound 15.54–15.57; quiet 15.57–15.65; body 15.66. The
  switch is in the second quiet run, 0.06 s after the first leave and 0.06 s before body.
- **04_4 s1** (67200, 2.80 s, "reflecting"; clarity take): pause 2.47–2.82; one frame of sound at 2.82; quiet
  2.83–2.90; body 2.92. The switch is 0.02 s before the first leave and 0.12 s before body.

## Priority cues

**00_5:** see F4 and F3. Its two recognizer differences (deleted "and" in "622," and "660.") are in the table below.

**04_4** (clarity take, one recognizer: large-v3-turbo; `04_4-download01.wav`, 30.24 s):

- lead 0.06 s, tail 0.16 s, peak −0.68 dBFS;
- no marks;
- s1 is the blip in F9;
- s4 (404160, 16.84 s, "so") is 0.14 s after leave, in a soft onset: pause 16.33–16.70, −41.4 dB at the switch,
  body 16.93;
- s2 (156960), s3 (320160) and s5 (563040) are in the pause, 0.01–0.08 s before leave.

**08_5** (clarity take, one recognizer; `08_5-download01.wav`, 32.40 s):

- lead 0.07 s, tail 0.16 s, peak −1.02 dBFS;
- the four answered boundaries are exactly at leave: s1 53520 (2.23 s), s2 170160 (7.09), s3 366720 (15.28),
  s4 498000 (20.75);
- the recognizer boundary s5 660480 (27.52) is also exactly at leave;
- both marks, b267 (22.42 s) and b306 (25.38 s), are in continuous speech (F8).

### The 17 discrepant cues

There are 36 recognizer differences, each located by the recognizer segment that holds it. Every difference falls
inside a segment the recognizer wrote as a numeral, a hyphenated or variant spelling, or a homophone. None is a word
missing between segments: each deleted "and", "a" or "thousand" is inside a numeral segment. Whether the spoken word matches the script (the "and" inside the numbers, "too" or "two", "laps"
or "lapse", "quarters" or "over four") is a question for hearing, not energy.

| Kind | Cue (sentence): recognizer segment, time |
|---|---|
| Numerals (spoken numbers written as digits; some drop an "and", "a" or "thousand") | 00_1 s1: 110, 220, 440, 880, 1760, 3520, 9.06–17.86 s; 00_2 s1: 1760 4.94–5.94, 3520 6.30–7.68; 00_4 s1: 440 2.96–4.02; 00_4 s3: 660 12.06–13.67; 00_5 s2: 622 11.02–12.84, 660 13.18–14.88; 01_4 s1: 531,441 4.60–8.30; 01_4 s2: 524,288 9.72–13.34; 02_5 s4: 240 16.42–17.32; 02_5 s5: 100 24.86–25.70; 04_7 s5: 138 29.52–30.64; 11_20 s0: 1966 2.92–3.54 |
| Numeral forms | 02_2 s1: "2.5's" 10.28–11.28; 03_2 s3: "2.4's" 14.78–15.76; 04_7 s5: "10th" 34.12–34.52; 08_1 s2: "3/4" 10.44–11.10 |
| Hyphenated | 04_7 s4: thousand-fold 21.26–22.08; 05_7 s0: non-zero 1.40–1.94; 07_6 s4: non-zero 24.72–25.38; 11_5 s1: non-zero 3.76–4.46 |
| Spelling | 01_4 s4: threes 17.74–18.14; 10_3 s3: spiraling 19.46–20.14; 12_4 s1: centers 8.20–8.52 |
| Homophones | 10_3 s2: "two." 11.82–12.92; 12_4 s4: "two," 30.48–31.00; 12_2 s1: "lapse." 8.96–10.16; 12_2 s2: "lapse," 12.04–12.80 |

Seven differences end at a sentence's last word, within 0.3 s of the next boundary. They fall at five boundaries:
00_5 s3, 01_4 s2, 01_4 s3, 10_3 s3 and 12_2 s2. Each of those boundaries sits within 0.05 s of the next sound's leave, so the difference does not move
the switch.

### Bronze's seven blip answers

All seven sit exactly at the leave of the first sound after the pause, 0.12–0.19 s before the body. No further quiet
run of 0.05 s or more lies between leave and body at 24 kHz.

| Join | Sample | Time | Pause | Body | Leave to body |
|---|---:|---:|---|---:|---:|
| 00_6 s1 | 131280 | 5.47 | 5.11–5.47 | 5.60 | 0.13 |
| 04_5 s1 | 58560 | 2.44 | 2.05–2.44 | 2.61 | 0.17 |
| 05_4 s1 | 139200 | 5.80 | 5.36–5.80 | 5.97 | 0.17 |
| 06_6 s2 | 261120 | 10.88 | 10.25–10.88 | 11.02 | 0.14 |
| 06_6 s4 | 621840 | 25.91 | 25.43–25.91 | 26.04 | 0.13 |
| 08_2 s1 | 102240 | 4.26 | 3.93–4.26 | 4.38 | 0.12 |
| 08_2 s3 | 420960 | 17.54 | 17.08–17.54 | 17.73 | 0.19 |

### Holds

The hold audit lists 15 cues: 13 prediction holds (floor 4.85 s after the voice) and 2 reading holds (10_4, 0.6 s;
11_20, 18.33 s).

- Every one ends quietly. Tails to the last frame above −60 dB are 0.09–0.15 s; last frames are −84.96 to −90.03 dB.
- None has an edge flag.
- Since no silence is inserted, the audio carries none of the hold. The floor is entirely the picture's waiting after
  the voice.

## Method and limits

- **Frames and levels.** 10 ms frames of 240 samples at 24 kHz, RMS in dBFS. A quiet run is at or below −60 dB for at
  least 0.05 s. Leave is the first frame above −60 after a run; body is the first frame at or above −40. These are
  BronzeJaguar's timing-inspection01 levels.
- **Choosing the pause.** For a boundary or mark, the pause is the longest quiet run within 0.30 s of the point. With
  none that close, the nearest run within ±1.0 s (±0.5 s for marks) is reported but nothing is concluded from it.
- **Flags.**
  - early: in the pause, 0.10 s or more before leave;
  - late: boundaries only, after body;
  - tight: a pause under 0.20 s;
  - blip: a further quiet run between leave and body.

  A mark after a pause is never called late, because the sound may be the previous word.
- **What energy cannot tell.** It cannot tell a breath, lip sound or the word itself apart.
- **How independent this is.** The recognizer words and times come from the installed helper's own records
  (`logarithm-stage09-audio-preparation02/prepare_audio.py`, sha256 `54df274a…`, loaded read-only). So this is not
  independent of the helper's recognizer.
- **What was not done.** Nothing was heard; no fine sync is accepted. Nothing was decoded, rendered or rerun, and the
  assembly was not inspected.

## Files

- `inspect_production_joins.py`: the inspection. It writes into a fresh folder and exits 1 on any chain problem.
- `outputs/joins.tsv`: 284 boundaries.
- `outputs/marks.tsv`: 33 marks.
- `outputs/cue-edges.tsv`: 80 cues, including the 15 holds.
- `outputs/discrepancies.tsv`: 36 rows.
- `outputs/inspection.json`: the chain, sha256 of all 252 inputs, parameters, the flag lists, priority cues and blip
  answers.
- `outputs/stdout.txt`: the summary the script prints.

Replay from the repository root (about a minute; `FRESH_OUT` must not exist):

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-production-join-inspection01/inspect_production_joins.py FRESH_OUT
```

Three runs were byte-equal in every output and in stdout: two before sealing and one inside the seal.

Earlier, unfinished states of `inspect_production_joins.py` were committed by others, in f925fb6, 86bf1ac, c4ce01e,
3ce2915, aeac056 and afe7577. The sealed bytes are the ones in `SHA256SUMS`.
