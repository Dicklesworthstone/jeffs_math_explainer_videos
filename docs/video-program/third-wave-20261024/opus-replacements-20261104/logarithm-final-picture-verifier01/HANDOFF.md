# Final verifier for the repaired logarithm master

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. Source and helper preparation only.

**Nothing here has run on a repaired master, because none exists.** No job was launched and no render, decode, ASR,
build or synthesis was run. Beads, TODO, memory and production files were not touched. All 11 check groups pass, but
they validate the helper's behaviour on old records and stand-ins. They accept nothing about the new film.

## Job name

The job is **`logarithm-final-picture-verify01`**, as root confirmed. I had first derived `-final-verify01` in my working
draft; I changed it before sealing, so no sealed file and no job carries the old name. `verification.json` is written under
that job, at `L/logarithm-final-picture-verify01/verification.json`.

## Paths

| Short | Path |
|---|---|
| R | repository root, the working directory |
| P | `docs/video-program/third-wave-20261024/opus-replacements-20261104` |
| L | `narration_lab/third-wave-20261024/root/five-film-release-20261027` |
| S | `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01` |

## Commands

All helper paths must be absolute. Run from R, after the master job has passed. Nothing below has been run on real
outputs.

```sh
# replace P, L and S with the paths above; `set --` keeps this working in zsh as well as bash
V="$PWD/P/logarithm-final-picture-verifier01/verify_final_picture.py"
set -- --lane "$PWD/L" --jobs "$PWD/S/render-jobs01" --series 01 \
  --crate "$PWD/S/workspace/explainer_corpus/logarithm_mysteries" --narration "$PWD/S/narration01" \
  --integration "$PWD/S/integration.json" --binding "$PWD/S/final-picture-binding01.json"

python3 -I -B "$V" check "$@"                                                    # read only, every gate
python3 -I -B "$V" write-job "$@" --verify 01 --job-dir "$PWD/S/verify-jobs01"   # prints the guard line
# root launches the printed line
python3 -I -B "$V" verify-job "$@" --verify 01 --job-dir "$PWD/S/verify-jobs01"  # after the guarded run
```

`--job-dir` is my suggestion and any new folder works. `write-job` refuses an existing script, inputs file or job
folder, so a second attempt needs `--verify 02`.

## What changed from version 02

The base is `P/logarithm-stage09-final-verification02/verify_stage09_final.py`, sealed e8e19275.
`docs/v2-to-final-picture.diff` is the whole change, and G1 proves it matches the helper exactly.

**45 definitions are identical to version 02, compared by parsed code (AST):** the timeline, layout, caption, script and
report parsers, `read_schedule`, `check_captions`, `check_phrases`, `media_facts`, `loudness_summary`, `make_sheets`, the
decode-log checks, `PINS`, `EXPECT`, `OUTPUTS`, and the master, timeline and verify success lines. `run_job` differs in
two places only: its receipt check uses `ran`, and `verification.json` gains the new keys.

**Four stage09 checks kept their purpose but changed form, because root's real outputs require it:**

| Check | Stage09 | Now |
|---|---|---|
| Direction map | a row in the master manifest | `report.json`'s hash, plus the binding's crate row |
| Script row | first in the manifest | found by name, because root's rows are sorted by path |
| Interpreter in receipts | literal `python3` | `python3` or an absolute `python3.NN`, because root launches with `sys.executable` |
| Render success line | stage09's line | root's new line |

**Added gates:**

- **Fresh master:** not the retired master `00236d1a…` or binary `e88e58e0…`. Build, assembly, timeline, score, 13
  renders and master each have a successful, stable receipt, in that order in time.
- **Repaired source:** each `integration.json` changed row leaves its stage09 hash, is on disk at its new hash, is bound
  by the master and was compiled by the build. The build link follows the build receipt's input hash to its manifest.
- **New metadata:** oliver17 with 13 prediction and 2 reading rows against current source, the report's script and unit
  hashes, and audio byte-identical to the assembly's record.
- **Holds:** every hold's real window, read from `mark` and `breath` rows, using the kit's own rounding.
- **Closing:** hold and fade after 12_21, from real rows.
- **Anchors:** see below.

**Schema:** `verification.json` keeps every version 02 key, including `video`, `sha256`, `frames`, `fps` and
`placements`, with placement and chapter rows of the same keys. `frames` is the sum of the real timelines, so encoded-ASR
tooling can bind the actual count. The retired film's frame count appears nowhere in the helper or `anchors.json`. New
keys: `anchors`, `pause_sites`, `closing`, `source_changes`, `integration_sha256`, `binding_sha256`.

## Anchors

`anchors.json` has 289 rows and 636 anchors. Each anchor is a schedule base plus a frame offset. **`was` is never read by
the helper:** it records the old frame for people and tests, and G5 proves that rewriting or deleting it changes nothing.

Bases: `cue`, `voice_end`, `sentence`, `phrase`, `hold`, `chapter_start`, `chapter_end`, `closing`.

| Carried | Rows |
|---|---|
| Panes 167, 168 and 169 findings, and join review J1–J4 | from the sealed reviews, as a base plus offset |
| All 12 chapter joins | `join-NN-MM` |
| All 13 pause-label sites | `pause-<cue>`: start 0/30/120, think end −1/+7, next cue 0 |
| The 2 reading holds | `reading-<cue>` |
| 04 answer reveal | `rep-04-reveal`, from sentence 6 of 04_7 |
| 11 gloss, sample X, one-logarithm formula | `rep-11-gloss`, `rep-11-sample`, `rep-11-formula` |
| 12 closing hold and fade | `rep-12-closing`, from the closing bases |

Each is seeded into the picture plan, with its id first in the caption. The ledger in `verification.json` gives its
picture and sheet. A base that does not resolve refuses. An offset that crosses the next cue also refuses, as a sentinel:
that site's timing has moved and someone should look, so edit `anchors.json` in a new path rather than widening the check.

## Integration assumptions

The helper refuses, rather than adapts, if one of these is false.

1. **Names are constants:** stage `logarithm-final-picture`; jobs `-build01`, `-assemble01`, `-timeline01`, `-score01`,
   `-render-<chapter>-01`, `-master01`, `-verify01`; work folder `L/logarithm-final-picture-render-work01` holding
   `logarithm_mysteries_master.mp4`, `.en.srt`, `.en.vtt`. Render and master job scripts are in `S/render-jobs01`.
2. **The master job's script** is what `S/render-master01.py` (sha256 `752fe43f…`, as read) writes, with its CONFIG in a
   sidecar `.config.json`, which the manifest must bind. The fixture uses that file's real `worker_text()` and
   `old_config()`, so if root edits it, rerun the checks.
3. **Render success line:** `4K60 chapter rendered; full timeline/cue tables match; picture review remains open.`
   Score line: `SCORE PASS: all 13 chapter score QA reports passed.`
4. **The binding** exists at `S/final-picture-binding01.json` in schema `logarithm-final-picture-binding-v1`, and its
   `files` holds the build manifest. `render-master01.py` does not read it, but the pipeline writes it, and this helper
   requires it. Keys were matched to `gates()` in the pipeline's source, sealed `3df900b0…`; no real binding exists yet.
5. **`integration.json`** keeps its fields, including `approved_for_guarded_build`, `audio_changed`, `script_sha256`,
   `changed_files` and `editorial_closing_hold`. It may be re-issued if the binding approves the new hash. Only stage09
   "before" hashes are pinned; new hashes come from the record, so a compile repair needs no helper change. A seventh
   changed source passes and is reported; a missing one refuses.
6. **oliver17** is at `docs/narration-direction-map-oliver17.json` in the crate, and `report.json`'s
   `meta.direction_contract.binding.contract.map` names it by file name and hash. No real oliver17 exists yet; the
   fixture's is a stand-in, oliver16 re-hashed to current sources.
7. **Assembly record:** `L/logarithm-final-picture-assemble01/audio-comparison.json` has `cues` (80),
   `wav_sha256`, `phrase_timings_sha256`, `all_full_wavs_byte_identical` and `all_phrase_timings_byte_identical`, and the
   master binds that file.
8. **Closing arithmetic:** one `breath` row outside every hold, in chapter 12, at or after 12_21's last word, with
   330 frames of hold (5.5 s) and then exactly 90 frames of fade. I read this from the kit source: `music_breath` writes
   the row after the last line plus a breath, and `fade_all(1.5)` is 90 frames. `finish` should then add nothing, because
   the window's time has passed. `integration.json` records `native_timing_verified: false`, and **this has never been
   measured.** A surprise refuses with the measured numbers.
9. **Anchor bases resolve:** words and sentence numbers are unchanged, because script and audio are unchanged. Stage09
   numbers will shift, but no absolute frame is assumed. The synthetic schedule used 116689 frames (chapter 04 +24,
   chapter 12 +420). That prediction is not an input, and the real total will come from the timelines.

## Traps

- **Shared target rows.** 59 build-manifest rows are in shared `target/geometry-kit01`. They were unchanged when read,
  but the master manifest inherits them, so another lane's build that moves one fails the guard and `check` with "bound
  inputs moved". Root already excluded one such binary.
- **Any edit under the crate after binding** refuses, because `check` compares the binding's crate tree.
- **`check` re-hashes every master row,** including the 4K chapters, so expect minutes.
- **Another worker's rehearsal is not covered.** `P/logarithm-final-render-master-review01` was unsealed when I looked,
  and I did not rely on it.

## Checks

From R, with a fresh scratch path: `python3 -I -B P/logarithm-final-picture-verifier01/checks/run_checks.py SCRATCH`.
The transcript is `checks/outputs/run_checks.txt`.

| Group | Evidence kind | Result |
|---|---|---|
| G1 | helper against version 02: 45 same, 15 changed, 22 added, 0 removed, diff exact | pass |
| G2 | real stage09 records, no anchors: all 1356 sealed pictures reproduced, reasons and order included | pass |
| G3 | real stage09 records: all 494 flagged frames come back from their bases | pass |
| G4 | plan with anchors keeps every old frame and reason, with the anchor first | pass |
| G5 | **synthetic** shifted schedule: anchors move with it | pass |
| G6 | 15 planted resolver faults | all refused |
| G7 | real holds, then 18 planted hold and closing faults | all refused |
| G8 | **stand-in** fixture from root's real worker text and config shape: 64 planted faults, each at its expected gate, and 5 more | all refused |
| G9 | **stand-in** ffmpeg, ffprobe and strict check, 20 pictures of one blank image: `run_job` and `verify-job` control flow | pass |
| G10 | no local paths in delivered files | pass |

The transcript reports 11 results: G1–G8, G9a, G9b and G10.

`build_anchors.py` documents how rows were made. It refuses an existing output, and I did not rerun it after generating
`anchors.json`.

## Limits

- **The real media path is untested here.** `run_job` ran only with stand-in tools, so the real ffmpeg extraction,
  strict decode and loudness steps were not exercised. Their code is the version 02 code, which ran on the stage09 master.
- **Real outputs do not exist yet:** no binding, oliver17, `S/narration01`, timelines or master. Their shapes come from
  source reading and stage09 records.
- **Passing proves technical facts only.** No frame was viewed or sound heard for this work. Independent picture review
  and listening remain open.
- **Not independent.** I authored part of the repaired source (the O1, O2, O3 patches in `draw.rs`, `complex_turns.rs`,
  `lifting_loops.rs`), reviewed sheets 113–225, and reviewed version 02. An independent reader should check this helper
  and view the sheets.
- **No Rust was compiled for this work.**
