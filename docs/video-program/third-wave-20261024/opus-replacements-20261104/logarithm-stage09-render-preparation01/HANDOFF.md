# Logarithm stage09 narrated render preparation (BlueSalmon Mail 3999, bead 87do.63)

RubyRaven, 2026-11-05. Root runs every step. This bundle holds one helper and its checks. I launched,
compiled and rendered nothing. I ran no cargo, guard, binary or kit. I wrote nothing into W, EL, the
receipts lane, canonical source, Beads or the TODO.

## Finding: build01 cannot run a narrated consumer

Root's `logarithm-stage09-build01` passed: 241 tests, strict Clippy, 13 chapters, binary `187e119d`.
That binary still cannot make a narrated timeline, score, render or master.

- **The topic path is compiled in.** `main.rs` calls `run_with_contract(&EXPLAINER, args,
  Path::new(env!("CARGO_MANIFEST_DIR")))`, so the topic is `W/logarithm-stage09`. The binary names that
  path 17 times.
- **A tool runs before any narrated output.** The W kit's `check_delivery_contract` (`cli.rs`
  `3c5221fe…`) runs first for `render --narration`, `captions`, `score` and `master`:
  `python3 -I -B <topic>/../../tools/narration_delivery.py --topic … --consumer … --gain-db 0 --chapter-key …`.
- **That tool is missing, and would refuse anyway.**
  - For build01 the path is W's parent `tools/narration_delivery.py`, which does not exist.
  - A copy placed there would still refuse. `narration_delivery.py:277` requires
    `topic.parent == <its repository>/explainer_corpus`.
- **Shown by a run (check D3).** I used a narration that passes through the route below. Both the
  canonical tool and the workspace copy refuse topic `W/logarithm-stage09` with "topic must be in this
  repository".

So build01 can render only silent or scratch-voice pictures. I read this from the kit and tool sources
and confirmed it with the tool runs above. I did not run the binary.

## Route: a workspace and one guarded rebuild

The workspace WS sits outside the repository. It holds:

| Path | Bytes |
|---|---|
| `explainer_corpus/logarithm_mysteries/` | the 48 files root built in build01, copied from `W/logarithm-stage09` and checked against root's stage binding |
| `tools/narration_delivery.py`, `coherent_narration.py`, `narration_cache.py` | byte copies, pinned `961f17cc…`, `95615cfa…`, `47331efd…` (the working-tree tools root installed; the audio binding pins the last two) |
| `workspace-binding01.json` | the record of both |

**Why it works.** The kit's tool resolves its repository as WS, the topic sits in WS/explainer_corpus,
and the direction map resolves to the crate's own oliver16 copy (`14e87893`). Sources, receipts and hold
evidence resolve from the repository root, because the guard runs jobs there. The audio preparation used
the same workspace idea for its assembler.

**One rebuild.** It uses the same W engine (`engine-budget.toml`) and root's three cargo lines from build01
(check B2 compares them line for line). Only the manifest path changes, to the workspace crate. Nothing is
promoted. No kit or engine bytes change, and no canonical file changes.

**Rejected alternative.** Root could promote stage09 into canonical `explainer_corpus/logarithm_mysteries`.
That needs root's decision on two problems:
- The canonical kit differs from W/repo-view's (`cli.rs`, `glyphs.rs` and two test files). Building there
  with the W engine mixes kits.
- Pointing the canonical Cargo.toml at W/repo-view edits canonical source.

The workspace avoids both.

## Commands for root (from the repository root)

```bash
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation01
W=…    # root's geometry stage (holds engine-budget.toml, logarithm-stage09/, build01's receipt pins)
WS=…   # new, outside the repository; for example $W/logarithm-stage09-workspace01
python3 -I -B $H/prepare_stage09_render.py check        --w "$W"
python3 -I -B $H/prepare_stage09_render.py install      --w "$W" --ws "$WS"
python3 -I -B $H/prepare_stage09_render.py build-script --w "$W" --ws "$WS" --attempt 01
#   then the printed line:
#   python3 -I -B …/engine-secondary-validation01/guard_owner56g_target20g.py --phase native \
#     --job logarithm-stage09-ws-build01 --inputs "$WS/logarithm-stage09-ws-build01-inputs.json" -- sh "$WS/logarithm-stage09-ws-build01.sh"
python3 -I -B $H/prepare_stage09_render.py build-verify --w "$W" --ws "$WS" --attempt 01
```

### What the steps check

| Step | Requires before writing | Then checks |
|---|---|---|
| `check` (read only) | — | the sealed native helper (`05e48275`) and its seal; build01 through that helper's own `verify`: receipt, 241 tests, Clippy, 13 chapters, binary `187e119d`, stage08 `e9233401` preserved; the three tools' pins; the blocker as above |
| `install` | WS is new, absolute, outside the repository and outside W/engine, W/repo-view, W/logarithm-stage08 and W/logarithm-stage09 | — |
| `build-script` | — | writes the shell job and its guard inputs (4,248 rows: the job, WS, W's stage09 with binding and originals, build01's binaries, W/engine, W/repo-view, W's pinned files, both helpers, the lock checker, the guard and its source, two film binaries) |
| `build-verify` | — | receipt, lock PASS, exactly one `ok. 241 passed`, the 13 chapters at the end of the log, the sums; the new binary names `WS/explainer_corpus/logarithm_mysteries` and never `W/logarithm-stage09` |

**The rebuild job** runs root's build01 shell lines unchanged in substance. It also:
- refuses without the guard's `CARGO_TARGET_DIR`;
- preserves the shared binary (today `187e119d`) as `WS/before-logarithm-stage09-ws-build01-binary`;
- copies the new binary to `WS/logarithm-stage09-ws-build01-binary`, the only binary the narrated jobs
  run, pinned by hash.

After the job, the shared target path holds the workspace binary. The bytes differ from `187e119d`
because the manifest directory changed.

### Narration: only from real production receipts

```bash
O=…    # BronzeJaguar's prepared folder after root's real decode, prepare, assemble, timings and install
python3 -I -B $H/prepare_stage09_render.py bind --w "$W" --ws "$WS" --build 01 --audio "$O" --attempt 01
```

Today `bind` refuses: no production receipt exists yet (check D1, on
`EL/combined-repair-stage09/audio-preparation01`). It writes `WS/narration-binding01.json` only when all of
the following hold.

1. **The audio bundle** is as LavenderPike reviewed it: seal `2bb8e280`, binding `2d7f4731`, helper
   `c6c854ce`.
2. **`O/prepare-receipt.json`:**
   - binding `2d7f4731`, use `production`, voice ElevenLabs Oliver;
   - `production: true` and chapter none;
   - the script's 80 cues in order;
   - spans hash still current, spans for units `1132a5a4`;
   - decode receipt production, source "ElevenLabs UI, Oliver", hash current, every decoded WAV current.
3. **The assembly workspace** tools are the canonical bytes, and the assembly manifest equals spans.json.
4. **`O-timings/timings-receipt.json`:**
   - binding and voice as above, `production: true`, 80 cues, file `phrase-timings.json`;
   - assembly and report hashes equal `O/narration`'s.
5. **`phrase-timings.json`:**
   - voice Oliver, `production: true`, binding `2d7f4731`, script `559afa36`, this prepare receipt's hash;
   - the 80 cues in order, each WAV and text hash current (the kit's own rule);
   - installed in `O/narration` with identical bytes, which is the receipt's `install` line.
6. **Two real runs** on the workspace copies, from the repository root:
   - `coherent_narration.py verify N --script … --units … --require-directions`;
   - the kit's own `render` contract for all 13 chapters, which must return 80 verified cues, topic
     `logarithm_mysteries` and no promotion.

The binding pins about 178 files: the receipts, the whole narration folder, the decoded WAVs and the bound
plan. Every later step re-hashes them all.

### The narrated jobs, one step at a time

```bash
python3 -I -B $H/prepare_stage09_render.py step        --w "$W" --ws "$WS" --binding 01 --series 01 --step timeline
#   launch the printed guard line, then:
python3 -I -B $H/prepare_stage09_render.py verify-step --w "$W" --ws "$WS" --binding 01 --series 01 --step timeline
# then preview (optional), score, render --chapter CH for each of the 13 chapters, master
```

**What `step` does.**
1. Re-verifies the workspace, the rebuild and the narration binding, and runs `verify-step` on every
   earlier job.
2. Writes `WS/jobs/<job>.py` and its inputs file. The inputs bind the script, both helpers, WS, the
   binary, every bound narration file, the guard, and each earlier job's receipt, log and outputs.
3. Prints `python3 -I -B …/guard_owner56g_target20g.py --phase native --job <job> --inputs <inputs> --
   python3 -I -B <script>`.

Each job script refuses unless the guard's receipt for that job shows it running, with the guard's
`CARGO_TARGET_DIR`, the pinned binary, and every earlier job finished cleanly. Outputs go to the guard's
lane, as in root's Nov 3 machine production.

| Job (series 01) | Consumer argv (`B` = the workspace binary, `N` = `O/narration`) | In-job gate, re-checked by `verify-step` |
|---|---|---|
| `logarithm-stage09-timeline01` | `coherent_narration.py verify` again, `B chapters`, then `B render all --timeline-only --fps 60 --narration N --out <job>/timeline-output` (2,400 s) | 13 chapters in registry order at 60 fps; every line plays `N/<cue>.wav`; the 80 cues in script order; `timeline-summary.json` |
| `logarithm-stage09-preview01` (optional) | `B render all --res 960x540 --fps 30 --narration N --out <job>/preview --lint --crf 28 --preset ultrafast` (14,400 s) | each chapter's cue order equals the timeline's; mp4 and layout-finding counts in `preview-summary.json` |
| `logarithm-stage09-score01` | `B score --narration N --timelines <timeline>/timeline-output --out <job>/scored` (7,200 s) | `scored/stems/qa/<chapter>.json` `passed: true` for all 13 |
| `logarithm-stage09-render-<CH>-01` ×13 | `B render CH --res 3840x2160 --fps 60 --narration N --score <score>/scored/stems --out <lane>/logarithm-stage09-render-work01 --lint --crf 16 --preset veryfast` (43,200 s) | the chapter's mp4 was absent before; afterwards its `chapter`/`fps`/`frames`/`line` rows equal the 60 fps timeline's |
| `logarithm-stage09-master01` | `B master --narration N --score <score>/scored/stems --out <work>` (14,400 s), then `B captions all --narration N --out <work>` (1,200 s) | all 13 chapter mp4s present; master, SRT and VTT written and hashed in `master-summary.json` |

**When a job fails.**
- A render that fails leaves its files in the work folder. Root moves that chapter's files aside, then
  writes `--retry N`, which gives job `…-<CH>-01-rN`. The master binds the latest retry.
- Any other failure means a new `--series`.
- A consumer that writes into a bound input (for example a render writing into the score stems) makes the
  guard end with "input manifest changed". That fails closed.

## Previous quality gates and where they sit

| Gate (precedent) | Here |
|---|---|
| Nov 3 machine production, root's `complex-timeline01.py`, `complex-score01.py`, `complex-render-chapter01.py`, `complex-master01.py` | adapted as the five steps above, with the same consumer arguments, timeouts and assertions, plus the guard-launch, binary-pin and predecessor checks |
| Nov 2 logarithm narrated review chain: per chapter, `--res 960x540 --fps 15 --review-units U --lint --crf 28 --preset ultrafast`, AV decode, captions, sheets read | replaced by the canonical-plan preview at 960x540, 30 fps (`--review-units` needs its plan and output under `<tool repository>/narration_lab`, which would be `WS/narration_lab`; I did not prepare one). Picture and listening review of the preview remain open |
| `complex-verify01.py`: `verify_video_preserve_logs01.sh` with `EXPECT_RES=3840x2160 EXPECT_FPS=60 EXPECT_BT709=1 EXPECT_LUFS=-16 EXPECT_TP_MAX=-1.0`, ffprobe stream shape, SRT = VTT = script text, pictures at cue starts, middles, ends, sentences, post-cue holds, layout findings and every 10 s, sheets, `pictures_viewed: false` | **not generated**; root adapts it: lane names above, topic `logarithm_mysteries`, 13 chapters, 80 cues, holds from the workspace crate's `docs/narration-direction-map-oliver16.json` |
| `complex-final-asr01.py` (whisper large-v3-turbo on the exact master, raw and number-normalized differences) | not generated; same adaptation |
| `caption-screen-after-verification01.py` (rate and line-length flags) and `complex-stokes-loudness-after-verify01.py` (ffmpeg `ebur128=peak=true` under the guard) | not generated; same adaptation |
| final pictures viewed, continuous listening, fine sync, independent final AV | open; nothing here claims them |

**Resources.** The jobs use the same 56g/20g guard as build01. At build01's admission the lane held
44.1 GiB of its 56 GiB cap, with 106.7 GiB free. Native admission needs 10 GiB, and each job may grow
4 GiB. Root's complex 4K60 chapters at CRF 16 were 1.4–5.9 MB each and its master 28 MB. I have not
measured logarithm's sizes.

## Open gates (nothing below is done)

1. The workspace rebuild: tests, Clippy, the new binary and its embedded topic. Only root's guard runs it.
2. Real production audio for stage09 does not exist yet. LavenderPike's independent review is root's
   highest priority (Mail 3999). Root's selection, answers, decode, prepare, assemble, timings and install
   all come before `bind`.
3. Real `phrase-timings.json` marks. The stage09 chapters use `until_phrase` in two cues. The fixture
   below has no marks, so the kit's mark handling is not exercised here.
4. Timeline, preview, score QA, the thirteen 4K renders, master and captions: none has run.
5. Picture review of the preview and the final, continuous listening, fine sync, caption and ASR screens,
   strict verification, loudness, and independent final AV acceptance.
6. Oliver only.
   - `bind` requires ElevenLabs Oliver in the prepare receipt, the decode receipt, the timings receipt and
     the phrase timings.
   - The canonical assembler's verify refuses mixed narrator voices.
   - No other voice and no mixed-voice final.

## Checks

```bash
python3 -I -B $H/checks/run_checks.py FRESH_SCRATCH --w "$W" > out.txt    # 52 PASS, 0 FAIL, about 55 s
```

`checks/outputs/run_checks.out` is the transcript of a fresh run. Paths are shortened to SCRATCH, W, REPO
and ~. An earlier run passed the same 52 checks, but some of its truncated details still showed part of
the scratch path. The checks now shorten paths before truncating; the earlier transcript stays in my
scratch.

**What runs for real.**
- `check` on root's W, build01 receipt and binaries (R1).
- `install` (I1–I6) and the real-target `build-script` (B1–B3), whose output stays in scratch.
- Root's lock checker inside the stub build.
- The workspace copy of the canonical assembler, which builds the narration fixture.
- `coherent_narration.py verify` and `narration_delivery.py`, as `bind` and the timeline job run them
  (D2, D3, D8, S3).

**What is emulated.**
- **The guard.** It binds the inputs before and after and writes receipt and log in the same shape. It
  has no admission, lane lock or resource sampling; I did not run root's guard.
- **cargo**: a stub first on PATH that logs its argv.
- **The binary**: a stand-in script. It writes timeline, score QA, chapter, master and caption files
  under the names root's Nov 3 complex outputs use.
- **The narration**: a fixture of 80 synthetic noise takes in the decode-receipt and span shapes,
  assembled with the bound plan's 15 holds, then wrapped in production-shaped receipts labelled TEST
  FIXTURE. This is a consistent forgery, so only the check under test can refuse. LavenderPike's K10 is
  the recipe; I wrote the code anew.

| Group | Shows |
|---|---|
| R1 | build01 passes the native helper's verify, and the blocker's facts hold |
| I1–I6 | the 48 bytes and 3 tools only; refusals for an existing, in-repository, in-stage or relative WS; the workspace check catches a changed crate byte, an extra tool and a second topic |
| B1–B9 | root's cargo lines exactly, the guard-target refusal before any cargo, and the stub run's order of test, build, copy-out, Clippy and chapters. `build-verify` passes the emulated run and refuses a failed job, 240 tests, an inputs file changed after launch, a binary without the workspace topic, and one still naming `W/logarithm-stage09` |
| D1–D10 | refusal on the real (absent) audio folder. A pass on the fixture, with the delivery contract accepting the workspace topic. The blocker run. Refusals for: test-only receipts, another voice or binding, a one-chapter run, 79 cues, timings for another assembly, phrase timings not installed or different, a WAV the timings do not name, and a changed WAV once the timings are forged (the real verifier refuses it). Non-canonical assembly tools are refused, and no refused bind writes a binding |
| S1–S15 | the exact argv per step; refusal outside the guard before the binary runs. The full chain: timeline, preview, score, 13 renders with one failed schedule and its retry, then master. Step-order refusal; a second write of the same step is refused. In-job failures for cue order and score QA; a moved narration file stops the next step |
| U1 | W's stage, the receipts lane, EL stage09, the tools and the shared binary are unchanged |

Nothing here claims listening, picture review, fine sync, a render, or acceptance of the audio or the
film.

## Files

| File | sha256 |
|---|---|
| `prepare_stage09_render.py` | `9ddc2003aa666844214aace166343bb37c1d5baeb6b62abb252f517c634b5388` |
| `checks/run_checks.py` | `010753d0d8897d77c8de773239a464aec8b7c376305558f507b6835a1f1e3314` |
| `checks/outputs/run_checks.out` | `e1495cec835683ceaeefd9d7f6f220de6c6050015f141cb314d325e18991fbd2` |

`SHA256SUMS` seals these three files and this one.
