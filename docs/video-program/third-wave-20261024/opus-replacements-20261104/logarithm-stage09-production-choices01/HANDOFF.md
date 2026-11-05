# Stage09 production choices: selection and provisional answers (87do.63, Mail 4153)

BronzeJaguar, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**What this is.** These are the inputs for root's guarded decode, prepare, assembly and timings of all 80 stage09 cues:
- a provisional selection of one recording per cue;
- measured `sample` answers.

Both are bound to the stage09 binding `2d7f4731…`, and the combined reviewed helper (`08a26927…`) accepts them read-only (V1–V5 below). **Nothing was heard.** The choices rest on recognizer, energy and timing evidence. Voice, accent, emphasis and cadence are not judged. The decision is root's.

## Root's commands

Run from the repository root. `H` is root's new helper folder: the reviewed `08a26927…` with `HERE` set to `H`.

```bash
H=<root's new helper folder>
B=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json
C=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-production-choices01/inputs
O=narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/production-audio01   # any new folder outside the bound inputs
python3 -I -B $H/prepare_audio.py --binding $B report --check docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-audio-preparation01/report
python3 -I -B $H/prepare_audio.py --binding $B decode --selection $C/selection.json --out $O/decoded01
python3 -I -B $H/prepare_audio.py --binding $B prepare --selection $C/selection.json --answers $C/answers.json \
  --decoded $O/decoded01 --reviewer BlueSalmon --out $O/prepared01
# then: the argv in $O/prepared01/assemble-command.json, then its "then" timings line, then the cp -n line in timings-receipt.json
```

The selection names the whole film, so run it without `--chapter`.

**To change a choice:**
1. Copy `inputs/selection.json` to a new path, edit `takes`, and set `by`/`basis` to root's.
2. Keep `answers.json` as it is. It answers every recognizer-early point of **every** selectable recording, and the helper applies only those of the selected recordings. A changed choice therefore picks up its measured answers, including the 08_5 download02 exception.

## Files

| File | sha256 | What it is |
|---|---|---|
| `inputs/selection.json` | `9c47f58f…` | `audio-preparation-selection-v1`, binding `2d7f4731…`, script `559afa36…`. 80 cues in script order, each named by its repository-relative path: 65 download01 and 15 download02. 04_4 and 08_5 are clarity download01 |
| `inputs/answers.json` | `8f9a0cd0…` | `audio-preparation-answers-v1`. Holds 181 `sample` answers, of which 92 apply to this selection. The report's 4 questions are left unanswered |
| `inputs/choices.tsv` | `f0c3678c…` | Per cue: the choice, the other take, the deciding rule, the answers applied, and open discrepancies |
| `inputs/evidence.tsv` | `5ff1bee3…` | Per selectable recording: the integrity screen, recognizer differences, shortest boundary pause and early points |
| `choose_takes.py`, `choose_takes-01.stdout` | | Writes the four inputs. The rerun into scratch is byte-equal |
| `validate_inputs.py`, `validate_inputs-01.stdout` | | V1–V5 with the combined helper. The rerun is byte-equal |

## The rule (per cue; the first difference decides)

| Rule | Test | Cues decided |
|---|---|---|
| R1 | Directly recognized, with no blocking question. This excludes 00_1 download03/04, the PCM twins of download02, as root's provenance check prefers | — |
| R2 | Fewer integrity flags. The flags are: a run of 3 or more full-scale samples; speech in the first or last 10 ms frame; a quiet run of 2.0 s or more inside the speech; a sentence boundary with less than 0.15 s of pause | 1 (08_2 → download02: download01's s1 pause is 0.07 s) |
| R3 | Fewer recognizer word differences that the other take does not share, once spacing, homophones and digits are set aside | 0 |
| R4 | The longer shortest sentence-boundary pause, when the two takes differ by at least 0.05 s. This gives more room at the tightest join. The pause is my timing inspection's energy measurement | 27 |
| R5 | The lower download number | 52 |

**Screen results.**
- All 162 recordings pass integrity: there are no full-scale samples (peaks −5.4 to −0.35 dBFS), and the longest internal quiet run is 0.92 s.
- Tails are 0.12–0.32 s, and speech starts 0.01–0.10 s in.
- The median speech level is −22.9 to −21.0 dBFS.
- The 36 recognizer difference lists are identical between the two takes of every cue except three, all notation: 04_7 dl01 "thousand fold", 09_1 dl02 "x" for "by" (two by two), 09_2 dl01 "two" for "too".

**Why not "fewer early onsets".** Early recognizer onsets are a timing artefact, not a delivery defect. Their pauses are no shorter (median 0.52 s against 0.49 s for onsets the energy confirms). The answers correct them.

**download02 choices:**
- by R2: 08_2;
- by R4: 00_3, 01_5, 03_2, 03_4, 04_6, 05_2, 05_4, 06_2, 06_4, 09_1, 09_4, 11_3, 11_5, 12_3.

R4 picks 00_1 download01 (0.53 s against 0.31 s) and 08_5 download01 (0.57 s against 0.50 s).

## The answers

Each answer is `sample` at the timing inspection's `leave_24k` (`8c18a243…`). That is the first 10 ms frame above −60 dBFS after the pause, and its basis names the measurement. **One exception:** 08_5 download02 s2 is 167040, at the vowel; there, leave is a blip (probe 4).

**Applied to this selection:**
- 92 onsets move later, by 0.20–0.73 s (median 0.38 s).
- No mark answer applies: the one early mark is 05_20 download02, which is not selected.
- 08_5 download01 gets s1 53520, s2 170160, s3 366720 and s4 498000, replacing the recognizer's 39360, 160800, 353280 and 488640. This settles the stage09 HANDOFF's note on download01 s1.
- With these, every mark is at least 1.59 s after its sentence onset (12_21 b192) and 1.58 s before the next bound (05_20 b276).

## Validation with the combined helper (`validate_inputs.py`, 5/5)

| Check | Result |
|---|---|
| V1 | The CLI `report --check` against the sealed stage09 report: in step, 5 files, 4 questions |
| V2 | `load_binding` gives `2d7f4731…`, and `load_selection` accepts 80 cues in script order |
| V3 | `check_answers` applies 92 sample answers. The blocking 00_1 twin questions do not touch the selection. The advisory `voice:04_3:04_3-download01.wav` stays open |
| V4 | `resolved_points`, with the cue text, at ⌈n/2⌉ for all 80 chosen recordings: 284 onsets and 33 marks, no refusal, every mark inside its sentence |
| V5 | Every applied answer moves its onset later, and nothing else changes |

## Open discrepancies (also per cue in `choices.tsv`)

1. **No listening.** R4 and R5 are weak signals: 27 cues turn on pause differences of 0.05–0.28 s, and 52 on the download number. The owner's or root's ear overrides any of them.
2. **Recognizer differences shared by both takes, in 17 cues.** These are spelled numbers, a dropped "and" or "thousand" in numbers, "a"→"one", "quarters"→"over four", "laps"→"lapse", "nonzero" and British spellings. They are probably recognizer formatting; only listening can rule out a misread. The cues are 00_1, 00_2, 00_4, 00_5, 01_4, 02_2, 02_5, 03_2, 04_7, 05_7, 07_6, 08_1, 10_3, 11_5, 11_20, 12_2, 12_4.
3. **04_3:** the advisory voice question is open for download01. Both takes carry it.
4. **04_4 and 08_5:** the clarity takes have one recognizer (large-v3-turbo) only.
5. **Answers that may land on a blip:** in 7 applied answers, leave is more than 0.1 s before the body: 00_6 s1, 04_5 s1, 05_4 s1, 06_6 s2 and s4, 08_2 s1 and s3. The answers use leave. The inspection's `outputs/timing-rows.tsv` gives `body_24k` if root prefers the body.
6. **Answers after a raised floor:** 13 applied answers follow a −63 to −73 dBFS floor rather than digital silence, perhaps a breath. Leave is after that floor.
7. **00_5:** its tightest join is 0.18 s (s2) in both takes.
8. **The 33 marks keep recognizer times.** They fall mid-sentence, where energy cannot place a word. The L1 check places each at least 1.58 s inside its sentence.

## Limits

- Nothing heard. There was no production decode, assembly, timing or render, and no edit to canonical tools, Beads, the TODO or any sealed bundle.
- The energy screen reads the original 48 kHz files read-only. The pauses and leave values are energy estimates, not heard boundaries.
- The selection's `by` names me as provisional. If root adopts it, root may restate `by`/`basis` in a copy.
- `H` must be root's integrated helper. The staged `08a26927…` still sets `HERE` to the sealed base folder (review01 R11).
