# Apportionment repair03: bound narration plan

LavenderPike for BlueSalmon, Mail 1585, bead `.30`, candidate `combined-repair-stage03` ("staged03").

This is a proposal for root to integrate after review. ElevenLabs Oliver only. Nothing was synthesized, selected, heard or timed, and nothing here says a recording of any cue exists: unchanged wording is not evidence of a usable take.

The files mirror canonical paths, so the contract's map path holds after integration:

- `explainer_corpus/apportionment/docs/narration-direction-map-oliver04.json`
- `explainer_corpus/apportionment/narration_units.json`
- `explainer_corpus/apportionment/docs/storyboard.md`
- `source-audit.json`: every wait in the registered chapters, classified
- `checks/`: binder, corrections, validator, `run_checks.sh`, `run.out.txt`

## Proposed versus installed

| Path under `explainer_corpus/apportionment/` | Proposed | Installed canonical now |
|---|---|---|
| `docs/narration-direction-map-oliver04.json` | `8136388e58082b74395b2b8141207bc5fd6dc7d5f80fb93f9f8f16eb518c7bd8` | none; `oliver03` is `fb59607a…`, left alone |
| `narration_units.json` | `2aaceadd55624524f9665022ea6a4fc2f861d703f33dc0ba0556e118a4a9b4d9` | `a9d90b6c…`, bound to `oliver03` |
| `docs/storyboard.md` | `f883e0229d6fe1a69bbe7f58aeb6801a5064651b1e1f99664280b2364bb8f45f` | `85d3b36b…` |
| `src/chapters/both_promises.rs` (root's staged03, not in this folder) | `c00c98e4f6b4b3094cfcd8643ecea1d61e5157963459b589aecdd17aa34c9144` | `c667ad1a…` |
| `src/chapters/sixteenth.rs` (root's staged03, not in this folder) | `d65ef2d84a7a3b6d5b133c956e5fd1694f9173c398f4af476f167111293ae618` | `fbf41f13…` |
| 13 other registered chapters, `draw.rs`, `script.tsv` | unchanged | same bytes; `script.tsv` is `cfcec069…` |

`oliver04` fits staged03, not canonical. Against the installed chapters the validator reports three problems, as expected: 04_3 hash, 12_1 hash, and 12_1's marker, which is at line 347 there and 351 in staged03. `oliver03` still fits canonical. The two chapters, map, units and storyboard therefore need to go in together.

Any further edit to `both_promises.rs` or `sixteenth.rs`, including from BronzeJaguar's check of the rewind and grey key, changes the map hash and so the unit plan's hash. Rebind if that happens. The binder refuses to overwrite a differing file, so give it a new output folder.

## Resolved counts

| | |
|---|---|
| Cues, units | 93, 93; one cue per unit, ids `oliver_<cue>`, in order, text exact |
| Words | 4,129 in total; maximum 100 (11_2, exactly at the limit), next 93 (12_9) |
| Directions resolved by `direction_contract` | 15 of 15, each ending its cue |
| Authored direction rows changed from `oliver03` | 0 of 15 overrides; 2 of 15 hold rows (04_3 hash; 12_1 line and hash) |
| Storyboard `⟨predict⟩` rows, `listen 4 s` rows, map rows | 15, 15, 15, the same cues |
| Hold rows | 15 prediction, 0 reading |

Boundaries after the 93 cues, from source: 15 prediction, 20 wipe, 1 hold plus pause, 2 hold directly before say, 40 plain breath, 15 chapter ends.

Calls in the 15 registered chapters: 15 `think`, 20 `wipe`, 3 direct `kit.hold`, 19 `pause` (15 closing a chapter, 4 others), 2 `wait_until`, 254 literal `wait_for`, 9 non-literal `wait_for`. No direct `stage.wait`, `kit.listen`, `music_breath`, `kit.mark` or `wait_frames`.

## Why there are no reading rows: a judgement for root to overrule

I followed the complex film's rule: a reading row exists only where the source adds a wait after the voice so that a card can be read. No site here does that. These are the nearest, and none got a row. Reasons are in `source-audit.json` under `other_waits`.

| Site | Where | What it is |
|---|---|---|
| `shares.rs:153-154` | 02_1 → 02_2 | `kit.hold` + `pause 0.6`. A real wait after the voice, but its comment says it keeps the wipe's timing. It is the same 1.2 s as the 20 wipes, which have no row. |
| `not_every_time.rs:195` | inside 06_6 | The film's only authored reading time: the quote stays at full strength for 159 chars at 15 chars/s, 10.6 s from when it appears. It delays a picture change inside the cue. It adds time after the voice only if 06_6 ends first. |
| `one_more_seat.rs:117` | under 01_4 | `pause 0.8`, timed from picture time, before a wipe. |
| `both_promises.rs:544` | under 12_3 | `pause 1.0` on "16 seats: 1, 4, 11", timed from picture time, before a wipe. |
| `both_promises.rs:799` | inside the 12_7 walk | `pause 1.2`, reached at 6 and 15. |

Also classified with no row: the two `kit.hold` calls before `say` in `not_every_time.rs` (82, 131), which add no time, and `shares.rs:165`, a word sync inside the voice.

If the shares beat or the quote needs a declared reading row, that is root's call. I did not invent one.

## Scratch check of the classification

`source-audit.json` compares all 78 in-chapter gaps on the preview02 scratch voice with their source class. preview02 predates staged03's chapters 04 and 12. This tests the classification only. It is not timing for Oliver.

| Class | Pairs | Scratch gap |
|---|---|---|
| Prediction (5.0 s in source: 0.6 breath, 0.2, 4.0, 0.2) | 15 | 5.067 to 5.087 s |
| Wipe | 20 | 1.200 to 1.231 s |
| Hold plus pause (02_1) | 1 | 1.218 s |
| Hold directly before say | 2 | 0.618 and 0.631 s |
| Plain breath | 37 | about 0.6 s |
| Plain breath plus picture overrun | 3 | below |

The three overruns are picture running past the voice, not authored pauses, so they have no row:

- 01_2 → 01_3, 1.615 s: Cedar's ten tokens take 2.8 s after "Cedar gets ten".
- 12_2 → 12_3, 2.515 s: the quota walk.
- 12_7 → 12_8, 2.012 s: the walk with its two 1.2 s waits.

Their size depends on the voice, so they must be measured on real takes and may need a picture fix.

## Storyboard corrections

Picture cells only; every other cell and line is byte-identical. Exact old and new text is in `checks/storyboard_corrections.py`.

- 12_5: the first tiles pulse, with no growth.
- 12_8: words only at "Suppose one were"; the rewind, then "then", legend and red ring at "Go back"; labelled chips; separate grey key; red items above the band; the opening sentence reworded, since fills are in view until the rewind.
- 12_9: chips pulse first, and the rings land at "But each town's row fits at most". "under them" became "under the chips", because the rings now come first in that sentence.
- **04_2, beyond S2:** "whole parts: 1 + 3 + 10 = 14", the same drift, from root's subtotal repair. Drop it if unwanted.

These rows describe staged03 as merged. The rewind and grey key still await BronzeJaguar's independent check; this wording does not accept them.

## For root before integrating

1. Do not install `src/chapters/both_promises.rs.orig` from staged03. It is a `patch` backup from an offset hunk, holds the n3-only result, and is not in canonical.
2. Root's `checks01/bind_oliver_plan.py` and `validate_oliver_plan.py` hard-code `oliver03`, and `build-picture05.sh` calls both. Pointed at the integrated tree unchanged they will fail or check the wrong map, and the binder without `--check` writes an `oliver03` file. On copies, the validator with only that name changed passes, and the binder's unmodified rows and units on staged03 are identical to these.
3. No crate source names the map or the unit plan, so no Rust edit is needed for the rename.
4. `narration_director.validate_map` wants `reason`, `evidence` and `policy`. Neither `oliver03` nor `oliver04` has them. The consumers that read these rows (`coherent_narration.direction_contract`, `narration_delivery.prediction_end`) do not need them.

## Actual coverage and its limits

Checked, on the current exact script:

- The repo's `read_units` and `direction_contract` in an isolated root, with `REPO` repointed.
- `check_coherent_units` rules transliterated to Python, plus one cue per unit.
- Every hold row's hash, marker at line, and position between adjacent cues in staged03.
- 9 planted defects, all refused.
- Root's validator, binder and `phrase_scan.py` on copies: 254 phrases, 0 problems.
- `tools/storyboard.py --check` in isolated roots: canonical baseline passes, corrected passes, a planted word change is refused, `script.tsv` unchanged.
- Staged03 and canonical topic tree digests equal before and after the run.

Not covered:

- **Static text only.** Nothing was compiled, tested, built or rendered. The Rust `check_coherent_units` and `check_video` were not run.
- **Line-based scan.** Only the 15 registered chapters were scanned. `draw.rs`, `kit.rs` and `narration.rs` were read by hand, and the kit and both tools are uncommitted working-tree copies (hashes in `run.out.txt`). Ignored and named in the audit: `alabama`, `divisor`, `priority`, `quota`, `remainders`, `seat_disappears`, `seventeen`, `three_rows`, `visual`, and the `.orig`.
- **No seconds are resolved for voice-dependent waits.** Whether the 01_4 and 12_3 beats fall inside the voice, whether the 10.6 s floor ever adds time, and how large overruns are all depend on takes. No Oliver take was used.
- **Reading sufficiency was not audited.** This records what holds the source has, not whether a card gets enough time.
- **Non-literal `wait_for` phrases are listed, not resolved** (9 sites in the audit). A wrong phrase fails at run time rather than here.
- **Picture text was checked against source only for the corrected rows.** Other storyboard rows were not audited.
- **Not established:** picture, sync, listening, AV or independent acceptance of anything.

## Reproduce

From the repo root, with an empty scratch folder:

`sh docs/video-program/third-wave-20261024/opus-replacements-20261104/apportionment-repair03-bound-plan/checks/run_checks.sh EMPTY_SCRATCH`

It runs Python only and writes only inside the scratch folder. `HASHES.sha256` is relative to this folder.
