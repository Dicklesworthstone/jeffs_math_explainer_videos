# complex-frederick-current-timing-review01

WhiteSwan, for root Mail 4394. Read-only. Frozen once sealed.

**What was not done.**
- Nothing was heard, synthesized, decoded, transcribed, cut or accepted.
- No job, guard, recognition or ffmpeg ran.
- No root, lane, take, canonical, Bead or TODO file was written.

**What was read.**
- The takes' 48 kHz mono 16-bit PCM, read with Python's `wave` module for 5 ms levels.
- Root's fw records.
- The repaired bridge's own `report` and `resolve()`.

## Scope

- **Bridge:** the repaired bridge `47069583` (seal `d5a413dd`), run with `report` only.
- **Inputs:** a pinned copy of the 36 lane files the report reads (plans and ingests 01–09). That is 94 processed takes of 47 cues; chapters 01–04 and 07 are complete.
  - Root's `targeted-small01` (small.en on 02_5) is passed as a supplement.
  - The takes and the guard jobs are read in place; the bridge binds both by hash.
- **The live lane matches the copy.** A report on the live lane at 06:34Z, before `plan-10` existed, is byte-identical to the copy's in all 5 files once the lane path is mapped back.
- **Outside scope:** `plan-10` (06:36Z: 05_1, 08_3, 08_4 and 08_5, 8 takes) is not ingested, so it is not covered here.
- **Held, as root instructed:** 02_5 (hearing open) and 08_2 (`f(z)=z` pending `targeted-small02`). Their points are reported but nothing is proposed for them.

## 1. The bridge's 20 blocking questions

**Timing: 2 questions, both on sentence 3 of 05_4, "Sine of i…".** The recognizer heard "Sign", so the slot has a candidate but no supported onset.

| Question | Recognizer "Sign" | The take's own levels | Proposed |
|---|---|---|---|
| `onset:05_4:s3:05_4-download02.wav` | [10.44, 10.78] s | pause [10.125, 10.455] s, body 10.495 s: the candidate sits at the end of the pause | `use_candidate` (sample 250560) |
| `onset:05_4:s3:05_4-download01.wav` | [10.86, 10.92] s, a 60 ms segment after `fast.` [9.46, 10.86] | pause [10.29, 10.615] s, body 10.655 s: the candidate is **0.205 s inside the word** | `sample` 254760 (10.615 s, the pause end) |

**Content: 18 questions.** For each one I read the recognizer's raw segment text at the difference, from the guard jobs' fw records:

| Reading of the raw text | Questions | Proposed |
|---|---|---|
| **Notation** the normalizer reads as other words: `x^40` (01_3 ×2), `1/2.` and `1/4,` (03_2 dl01), `-i,` (04_7 ×2), `1+i`, `1+i,`, `2+2i.` and `2 + 2i.` (06_4 ×2), `non-zero` (07_2 ×2) | 9 | `proceed` |
| **Same sound:** `Sign` for sine (05_4 ×2); `routes` for roots, which British English says alike (01_6 ×2) | 4 | `proceed` |
| **Open:** `term` for turn (02_4 dl02). Only hearing settles it. | 1 | none |
| **Held:** 02_5 ×2 and 08_2 ×2 | 4 | none |

**The held cues.**
- **02_5:** turbo writes nothing between `by` [0.64, 0.88] and `once` [0.88, 1.18], or between `is` and `squared?`. small.en writes "1 plus i" in both places. Both models write `term`.
- **08_2:** the raw text is `f` `=` `z,`, so "equals" is written as `=` and "of z" is absent.

These are readings of the recognizer's text. None of them is a claim about the audio.

## 2. Supported onsets that start inside their word

The bridge accepts a recognizer onset as supported by its text agreement alone. Read against each take's own levels:

- **62 sentence onsets start more than 0.03 s after their word is clearly sounding:** 61 supported ones, plus the 05_4 download01 candidate above.
  - They fall in 28 cues and 41 takes.
  - 27 of them are late by ≥ 0.1 s, and 18 by ≥ 0.2 s.
  - The worst is `onset:03_5:s3:03_5-download01.wav`. The recognizer puts "inside" at [14.08, 14.10], but the sound runs without a break from the end of the pause at 13.57 s to a dip at 14.11 s. That places the onset at the word's end, 0.49 s late.
- **A sentence onset is the end of the previous sentence.** So in every one of these cases, the first part of the sentence's first word falls inside the previous sentence's span. 9 anchor marks that sit at sentence starts inherit the error.
- **The signature:**
  - 61 of the 62 abut the previous word, and 55 have a window of 0.1 s or less.
  - The previous word's segment is long (median 1.2 s, minimum 0.68 s): it absorbs both the pause and the start of the next word.
  - An example: `fast.` [9.46, 10.86], then `Sign` [10.86, 10.92].

**The rule** (written out in full in `candidates.json` → `envelope_rule`):
- **Pause:** at least 0.06 s of 5 ms frames at or below −70 dBFS. The pause must lie wholly after the start of the previous recognized word, and it must have begun by the recognizer start.
- **Body:** the first frame at or above −30 dBFS after the pause.
- **Late:** the recognizer start is more than 0.03 s after the body. `late_by_s = start − body` is a lower bound.

**Why the sound after the pause is the point's own word:**
- **The guard:** the pause comes after the previous word's start.
- **One stretch of sound (e2):** in 60 of the 62 cases the sound never drops below −50 dBFS between the body and the recognizer start. In the other 2 it drops for 5 ms and 10 ms: 06_1 "step" in each take, consistent with the /t/ closure.
- **Independent recognizer (x1):** small.en is the only second recognizer root has run. On the two 02_5 points where turbo is late, it starts inside the pause instead: s1 4.50 s against turbo's 4.60 s, and s2 7.32 s against 7.46 s.

**Proposed: 60 `sample` answers at each pause end.** That is 59 supported onsets plus the 05_4 download01 question. The 2 late points in held 02_5 get no proposal.
- At the pause end, the take is at or below −70 dBFS before the sample and sounding within 5 ms after it.
- The sample is `round(pause_end × 24000)`, the bridge's own convention.
- With these 60 answers and the 05_4 download02 `use_candidate`, the bridge's own `resolve()` accepts all 94 takes under its sentence-order, range and mark-in-sentence rules (s1). Without them it accepts 92.

## 3. Points not proposed

- **191 onsets lie in the pause before their word.** By rise minus start they are early by a median of 0.100 s and at most 0.455 s; 76 are at least 0.2 s early.
  - As sentence ends they cut no word: the boundary falls in the pause.
  - Whether a sentence's visuals may start that far before its voice is a policy choice for root. `other_points` carries each one's pause and body, so a voice-onset policy would be `round(pause_s[1] × 24000)` for each.
- **1 onset sits in the faint tail of the previous word:** `onset:03_4:s1:03_4-download02.wav`. "Every" is placed at 0.64 s, inside the −60 dBFS decay of "No." The word sounds at 0.925 s, after the pause [0.79, 0.89].
- **33 points are not assessed:** 32 mid-sentence marks and 1 onset. They sit in connected speech with no pause, so the levels cannot place them, and they stay recognizer estimates.

## Use

- **`candidates.json` → `proposed_answers`** is a `complex-frederick-answers-v1` block for this source side (`source_sha256`). It holds 74 answers: 60 `sample`, 1 `use_candidate` and 13 `proceed`.
  - **Every `by` is `null`,** so the bridge refuses each answer until root signs it. Root copies only the answers root accepts.
- **`late_points` and `content` bind each row to its evidence:**
  - the take sha256;
  - the job, its `command.log` sha256, the record index and the segment index;
  - the recognizer window, the previous word, the pause, the body, and the lowest level between body and start.
- **`inputs`** pins the bridge, the 36 lane files, the supplement and the report's 5 output files.

## Replay

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-current-timing-review01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/checks/timing_review.py NEW_SCRATCH_DIR
```

- **Where and how long:** run it from the repository root. It takes about 4 s and writes only under the new folder.
- **Exit codes:** 0 PASS, 1 FAIL, 3 if a pin moved.
- **What it rebuilds:** `candidates.json`, which must be byte-identical to this bundle's (h1).
- **Line r2** compares a live-lane report only while the lane still holds exactly plans and ingests 01–09. Since `plan-10` it prints a NOTE instead. The 06:34Z comparison above is the record of that check.

## Limits

- **Levels are not listening.** Whether a boundary is right by ear, and whether a word was said, stays root's call.
- **Thresholds:**
  - All thresholds are fixed constants, stated in the script.
  - Frames are 5 ms wide, so a pause end can be off by up to one frame.
  - Soft onsets (/s/, /f/, /h/) are measured from the body, so `late_by_s` understates them.
- **Recognition:** only root's turbo records and the one small.en job were read; no recognition was run.
- **Coverage:** the 8 takes in `plan-10`, and any later takes, are not covered.
