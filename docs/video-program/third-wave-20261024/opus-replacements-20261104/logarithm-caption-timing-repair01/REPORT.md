# Logarithm caption timing repair 01

BronzeJaguar, answering root Mails 5753, 5754, 5765 and 5838.

**Status:** new sidecars for root to judge. This is not audiovisual acceptance. Nobody has heard or watched these captions against the film. The master was not rerendered, and the native SRT and VTT were only read.

## New sidecars

| File | SHA-256 |
|---|---|
| `sidecars-timing01/captions.en.srt` | `b2914c6fdb517f6316117b539ff84a1f64ded7f0a44d29fceea79a01cb20cb22` |
| `sidecars-timing01/captions.en.vtt` | `a2fb0d69b99652b9457fe39bf724f7affc64296674ea2e3a08265d44c0a64397` |

They belong to master `8534ec21…7efd69` and replace native SRT `c9321cd7…1c7b` and VTT `45833988…0b45`.

## Did the bound phrase timings support the tool?

No, for three reasons:

1. `phrase-timings.json` has a mark at 1 of the 111 same-sentence boundaries (block 465).
2. It has no `asr_log_sha256`, so the tool would stop on a missing key.
3. No final-master timeline was local until root imported the 13 real files (Mail 5838).

So `tools/retime_captions_from_phrases.py` ran unchanged (`028a934b…a78f`, its 5 tests pass), on a new timing input written by `build_inputs.py`.

## Result

**66 of the 111 boundaries moved:** 61 earlier and 5 later, by −1.155 to +0.565 s. Another 40 sit within 0.25 s of their word and stayed. The last 5 could not be measured and stayed.

Across all 475 blocks, no word, line break, block number, sentence start or sentence end changed. No guard limited a move.

Lag is word onset minus block start. Negative means the word is spoken before its block appears.

| Measured by | | Median | Earliest | Latest | Over 1.0 s | Over 0.75 s | Over 0.5 s | Over 0.25 s |
|---|---|---|---|---|---|---|---|---|
| Estimate used for the marks (n 106) | before | −0.343 | −1.155 | +0.565 | 3 | 12 | 31 | 66 |
| | after | +0.000 | −0.241 | +0.200 | 0 | 0 | 0 | 0 |
| ASR02 on the master (n 105) | before | −0.366 | −1.267 | +0.496 | 3 | 13 | 33 | 67 |
| | after | −0.010 | −0.553 | +0.173 | 0 | 0 | 1 | 4 |
| Clean-take recognition (n 106) | before | −0.347 | −1.250 | +0.505 | 3 | 14 | 34 | 70 |
| | after | −0.009 | −0.540 | +0.176 | 0 | 0 | 1 | 5 |

**The after rows are close to circular.** The marks came from these same sources, so a small after-lag proves the tool applied them, not that the captions now match the speech.

The worst boundaries, by the estimate:

| Block | Word | Native start | New start | Before | After |
|---|---|---:|---:|---:|---:|
| 145 | a | 555.455 | 554.300 | −1.155 | 0 |
| 409 | both | 1630.287 | 1629.257 | −1.030 | 0 |
| 364 | spiralling | 1447.932 | 1446.910 | −1.022 | 0 |
| 262 | strip, | 1041.727 | 1040.783 | −0.944 | 0 |
| 178 | pi: | 694.185 | 693.263 | −0.922 | 0 |

Block 364 is a third lag over 1 s that my review missed: the recognizers spell it "spiraling", so that review never matched it. The review's −1.27 for block 409 is ASR02's figure. The clean WAV puts the word 0.24 s later, so −1.03 is the better number.

The largest remaining lags by the estimate are block 243 "returns" −0.241, block 287 "when" −0.236 and block 441 "perfect" +0.200. All are inside the floor.

### Reading speed

Counted by the production screen `caption-interval-audit01.py`, unchanged:

| | Over 20 cps | Lines over 42 | Fastest | Median | Over 18 cps | Shortest block |
|---|---:|---:|---:|---:|---:|---:|
| Native | 0 | 0 | 17.13 (block 63) | 13.31 | 0 | 1.030 s |
| New | 0 | 0 | 18.753 (block 294) | 13.32 | 6 | 1.030 s |

Reading speed is the cost of this repair. Blocks over 15 cps rise from 61 to 88, and over 17 cps from 1 to 17, because a block now gets the time the voice actually gives it. My review's 16.9 cps left out the space that joins two lines; the screen's 17.13 is the right native figure.

## How each boundary was timed

1. **Two recognizer runs.** ASR02 ran on the encoded master. The per-take logs in the stage09 binding ran on the selected clean takes, mapped into cue time through `report.json`. At the 103 boundaries both runs have, the median difference is 0.000 s; 6 differ by over 0.1 s, 3 by over 0.2 s and 1 by over 0.3 s.
2. **The later onset is used,** because these onsets run early.
3. **The clean cue WAV is consulted.** Runs of 10 ms frames at or below −60 dBFS lasting 50 ms or more are quiet runs. The end of a run is used if the onset sits inside it, or if the run starts within 0.35 s after the onset and leaves at least 0.10 s of the recognized word after it.
4. **Then it advances to speech,** to the first frame at or above −40 dBFS within 0.35 s.
5. **Moves under 0.25 s are not made.** They cannot be told from the recognizer's bias.
6. **Guards.** Neither neighbouring block may pass 20 cps or fall under 1.0 s. Neither guard was needed.

A word must have a matched neighbour, or both runs must agree within 0.10 s. One-word numbers match their digits ("six", "6").

## Limits

- **Both recognizers are one model** (large-v3-turbo), so their agreement cannot rule out a shared bias. Frame dumps show onsets running early even with no pause: "a" about 0.1 s and "strip," about 0.15 s, though those two readings are mine, from energy alone.
- **Moved blocks probably lead their words slightly,** by something like 0.1 to 0.2 s, where no energy evidence corrected the onset. That is a judgment from those dumps, not a measurement across all 66.
- **The rules were fitted to this film.** I set them after reading the energy at all 25 boundaries with a quiet run nearby, then added the speech step for block 254. Nothing tests them on unseen data, and energy cannot tell a breath from a word.
- **Five boundaries are unresolved**, all inside numbers the recognizers wrote as one token, in recording time, which equals cue time for these three cues:

| Block | Cue | Word | Block starts | Recognized token | Reading |
|---:|---|---|---:|---|---|
| 3 | 00_1 | ten, | 10.133 | "110," 9.06–10.46 | starts inside the token; size unknown |
| 4 | 00_1 | eighty, | 14.861 | "880," 13.66–15.06 | starts inside the token; size unknown |
| 47 | 01_4 | five | 4.663 | "531,441." from 4.60 | first word of the number, about 0.06 s late |
| 49 | 01_4 | five | 9.420 | "524,288." from 9.72 | leads by about 0.30 s |
| 88 | 02_5 | and | 17.461 | "240" 16.42–17.32 | at least 0.14 s late, and more if "and" starts mid-number |

- **Author check only.** I wrote the review that found this, the rules and these checks. Nobody else has reviewed this bundle.

## Real-input binding

`check_output.py` passes 36 of 36 (`outputs/check_output.txt`):

- Master, native SRT and VTT, script, report, verification and every recognition source match their pins, measured afterwards.
- The 13 real timelines match `verification.json`. The tool opens each WAV named in a line row and the remote job folder is not local, so `inputs/timelines/` holds copies differing only in that field, pointing at `prepared02/narration`.
- All 364 native sentence starts reproduce within 0.333 ms. Sentence ends are 351 exact and 13 held, none earlier.
- Two planted faults are caught: a sentence start moved 1 ms, and changed text.
- The VTT equals the SRT in all 475 blocks. Each differs from its native file only on timestamp lines, plus one fewer final newline.

**One binding is indirect.** The render used `logarithm-remote-assemble01/narration01`, which is not local. `final-picture-repair01/assemble01.py` asserts all 80 WAVs byte-identical to `prepared02/narration`, the 80 local durations equal `verification.json`, and word times mapped through the local report agree with the master run. I have not seen that job's `audio-comparison.json`. Root could confirm it with one read.

`--recognition` takes one file, so it gets `inputs/recognition-sources.json`, which lists all six recognition sources by hash, and `asr_log_sha256` binds that list.

## Positioning and samples

- **Positioning stays with Lavender (Mail 5844).** This VTT has no cue settings. All 475 cue numbers are unchanged, and no split or merge is coming.
- **None of Lavender's 37 window edges moved.** The windows keep identical times.
- **Overlay samples:** none of the 23 sample frames shows different text. At 11 frames the same block is up with moved times (`outputs/overlay-sample-impact.tsv`).
- **Only 66 intervals differ,** 35.500 s of 1936.817 s and 2,135 frames, listed with frames and both texts in `outputs/changed-intervals.tsv`. A fresh sample set only needs frames inside those.
- **YouTube is not tested.** Upload behaviour and the positioned result need root's private draft check.

## Files

| Path | What it is |
|---|---|
| `build_inputs.py` | Pins inputs, times the boundaries, writes `inputs/` |
| `check_output.py` | The bounded checks |
| `inputs/boundaries.tsv` | All 111 boundaries: both onsets, estimate, basis, decision |
| `inputs/timings.json`, `inputs/recognition-sources.json`, `inputs/timelines/` | What the tool read |
| `sidecars-timing01/` | The tool's output, with `retiming.json` |
| `outputs/` | Tool, test and check transcripts, plus the two impact tables |
| `superseded/` | Four earlier trials, kept and not for use |

Lavender read an earlier trial with 65 moves. The final file adds block 90 "six".

## Reproduce

From the repository root, with paths relative to it:

```
python3 -I -B <bundle>/build_inputs.py
python3 -I -B tools/retime_captions_from_phrases.py --script <script.tsv> --report <report.json> \
  --timings <bundle>/inputs/timings.json --recognition <bundle>/inputs/recognition-sources.json \
  --timelines <bundle>/inputs/timelines --srt <native SRT> --output <new folder>
python3 -I -B <bundle>/check_output.py
```

Each step refuses to overwrite existing output, so use a copy of the folder.
