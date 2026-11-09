# CC2 review 307: combined master 293

cc2, 2026-11-08. This is an independent review of cc0's combined master 293. As root asked, it covers:
- real assembly correctness;
- the 15 rows;
- the source, score and narration tables;
- imported CSS provenance;
- captions, soundtrack and guard behaviour;
- compatibility with the new final303 worker.

**What was done.** Read-only work. Nothing ran on CSD, CSS, TRJ or HZ2: no job, import, permit, render, Beads or TODO edit, and 293 itself was not run anywhere. The only checks were local, read-only scripts over the retained evidence copies in Q. They are not in the repo, and they are not mocks of any host run.

**Not claimed.** No picture, motion, audio, listening, caption or AV claim. The narration is the ElevenLabs Frederick Surrey assembly. No audio was heard.

| File | sha256 | State |
|---|---|---|
| `cc0-combined-master293.py` (570 lines) | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` | reviewed; unchanged 17:45–17:58Z |
| `cc0-combined-master293-notes.md` (254 lines) | `ea554bbde609b72f5711c7f240fa4a3c78f69b146c7bf28fb6ed61043f17724e` | reviewed |
| `csd-reviewed-final303.py` | `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` | new at 17:54Z; read for compatibility only, not reviewed as a worker |
| `csd-launch-final303.py` | `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` | same |

**Terms.**
- D is the CSD workspace. TRJ, CSS and HZ2 are those workers' roots.
- `:N` is a line of 293 unless another file is named.

## Result

**0 High, 0 Medium, 2 Low, 8 notes, plus one blocking prerequisite that lies outside 293.**

293's own assembly logic is sound for what it checks:
- every gate I could reproduce from actual evidence passes for the four retained rows;
- its worker interface matches 303;
- every Low fails closed. The worst outcome is a spent version 293 and job name, never a wrong master.

What stops a launch today is B1: no CSD import exists. It is missing work, not a defect in 293.

## B1. Blocking prerequisite: no CSD import path exists

**Why.**
- All four retained finals live on other hosts: 02 and 03 on TRJ, 13 on CSS, 15 on HZ2.
- 293 requires every foreign row to have a successful import receipt at `D/jobs/<job>/resource-receipt.json` (`imported()`, `:262–266`).
- These finals' existing imports are TRJ imports (`trj-import-css-final13-97`, `trj-import-hz2-final15-56`, per plan183), so they do not qualify.
- Q holds no CSD import or receive helper. It has only the TRJ importers and the CSS, HZ2, TS1 and TS2 runtime receivers.
- Re-rendering the four on CSD is not possible either, because 287 and 303 cover only the 11 changed chapters.
- So nothing launches until a new CSD import helper is written, reviewed and run.

**What it must place.** This list comes from 293's actual reads. Every file goes at its original absolute path, on the same filesystem as `D/jobs` (`:377–379`).

| Row | Files 293 reads on CSD |
|---|---|
| 02, 03 | the worker `trj-final-chapter19-02.py`; `final02-32-inputs.json` / `final03-28-inputs.json` and every TRJ-path row in them; the permit and its evidence (9 and 5 items, all TRJ paths); each job's receipt, `render-summary.json` and `render/` (6 files, including `soundtrack_0.wav`); the `trj-production19-delivery02` receipt (read by `check()`) and its narration folder |
| 13 | `css-final-chapter78.py`; `final13-96-inputs.json` and its CSS rows; the permit and its 6 evidence items (CSS paths); the job folder; the `css-runtime-transfer75`, `css-encoder-stop79` and `css-production19-delivery76` receipts; the delivery76 narration |
| 15 | `hz2-final-chapter53.py`; `final15-53-inputs.json`; the permit and its 12 evidence items (HZ2 paths); the job folder; the `hz2-runtime-transfer47`, `hz2-encoder-stop03` and `hz2-production19-delivery49` receipts; the delivery49 narration |

**D-path rows.** D-path rows in those manifests are not imported. They stay CSD's own files and must still be at their recorded hashes. `check` names any file that has drifted.

**CSS-290 rows are now optional.** 303 covers all 11 changed chapters on CSD: PREVIEW243 for 06/08 and PREVIEW274 for the nine. Rendering all 11 on CSD leaves only the four retained rows to import. The cost is CSD's single native lane, so the 11 renders run one after another.

## Lows

**L1. The master's and captions' delivery-contract paths have never run on these tables.**
- **Why.**
  - Inside the job, `master` and `captions all` each first run the workspace's `tools/narration_delivery.py` with `--consumer master` or `captions` and `--timelines final/` (kit `cli.rs:390–471`).
  - For `master`, that adds `cue_log()`, `prediction_end()` and `directed_segments()` (`narration_delivery.py:149–253` and `:324–340`). The kit's caption validation follows (`cli.rs:892–936`).
  - Every earlier job using rebind269 ran the `render` consumer: rebind269's own delivery, timeline269 and preview274. Production19's master never ran.
  - A refusal there comes after the hard links, and it spends 293.
- **Cheap read-only pre-check (root, on CSD).** The script only prints JSON (`main()`, `:350–369`). Run the workspace268 copy of `tools/narration_delivery.py` with:
  - `--topic` = the workspace268 topic folder;
  - `--narration` = rebind269's narration folder;
  - `--consumer master` and `--gain-db 0` (kit `GAIN_DB = 0.0`, `narration.rs:46`);
  - one `--chapter-key` per registry chapter;
  - `--timelines` = timeline269's `timeline-output`;
  - no `--score`.

  Timeline269's cue logs have no stem rows, so without `--score` this exercises every master check except the stem rows. Exit 0 with `verified_cues` 105 clears the Python half.
- **Optional.** Binary 268's `captions all`, with `--out` set to a new folder holding copies of the 15 timeline269 `.timeline.tsv` files, exercises the kit's caption validation. It writes two small files, so it is root's call.

**L2. Any admission refusal spends 293.**
- **Why.**
  - `launch` writes `combined-master293-inputs.json` with `'x'` (`:459`) before the guard admits the job.
  - Its first check refuses if that file or the job exists (`:434`).
  - So if `guard_csd02.py` refuses for any reason (free space under the 10 GiB admission, or anything else), 293 can never launch again. A new version and job name are needed.
  - 287 and 303 share this pattern.
- **Fix (root, no code change).** Just before `launch`, confirm that free space is well above 10 GiB and that no other native job is running. Otherwise, accept that a refusal costs a version bump.

## Notes

**N1. The retained rows pass every 293 gate on the actual evidence.**

The checks read the local copies of `final02-terminal167`, `final03-terminal115`, `final13-terminal200`, `final15-terminal81` and `retained15-tables281`, the four permits, plan183, `score21-terminal01.json`, `timeline21-terminal01` and `timeline269-comparison305.json`.

All four rows pass these checks:

- **Receipts.** Each receipt succeeds with policy 8/4. The command names the pinned worker and exactly one `--chapter`, `--job`, `--inputs` and `--permit`, equal to plan183 and 293's pins.
- **Render summaries.** Chapter, frames, binary 19 (`f4836eee…`) and permit sha equal the pins.
- **Movies and permits.** Each local MP4 is at its movie pin, and each local permit copy is at its permit pin.
- **Timeline binding.** Each permit's `timeline_sha256` is timeline21's table, which is the table 293 uses for retained rows.
- **`schedule()`.**
  - The timeline and cues equal timeline269 after mapping rebind269 to the worker's narration (delivery02, delivery76 or delivery49).
  - Every line row lies in that folder, and the cue-log prefix is equal.
  - The stem rows are score21 stems at score21's hash; 13 has two parts.
  - Each summary lists one capture.
  - All three tables match the render summary's hashes. For 15 the tables come from `retained15-tables281`.

**Whole film.**
- The 45 local timeline21 tables reproduce timeline269-comparison305's old and new hashes under the one assembly19 → rebind269 mapping: 45/45.
- The frames sum to 133,662.

**Not rechecked.** I did not recheck the scene-file pins; the notes report 25/25.

**N2. The 15 rows and their start frames.**

| Chapter | Frames | Start | Row |
|---|---|---|---|
| 01_the_bump | 6,311 | 0 | combined (303) |
| 02_turning_numbers | 5,372 | 6,311 | retained, TRJ |
| 03_two_planes | 7,249 | 11,683 | retained, TRJ |
| 04_reading_color | 6,774 | 18,932 | combined (303) |
| 05_gallery | 9,879 | 25,706 | combined (303) |
| 06_one_derivative | 10,030 | 35,585 | combined (287, 303 or CSS-290) |
| 07_one_point | 6,491 | 45,615 | combined (303) |
| 08_loop_sums | 14,959 | 52,106 | combined (287, 303 or CSS-290) |
| 09_boundary | 16,504 | 67,065 | combined (303) |
| 10_rigidity | 7,442 | 83,569 | combined (303) |
| 11_counting | 11,932 | 91,011 | combined (303) |
| 12_residues | 3,943 | 102,943 | combined (303) |
| 13_real_integral | 10,641 | 106,886 | retained, CSS |
| 14_continuation | 11,756 | 117,527 | combined (303) |
| 15_closing | 4,379 | 129,283 | retained, HZ2 |

- Publication306's `chapter-times.json` start frames equal 293's cumulative `start_frame` for all 15.
- I saw no terminal record of any 287, 290 or 303 final in Q, so no combined row exists yet for `check` to pass.

**N3. 303 is compatible with 293.**
- **Interface.** 293 needs these things from a CSD worker, and 303 has all of them:
  - `ROOT` = D, plus `BIN`, `BINARY_SHA` and `NARRATION`;
  - `check()` returning `(review, permit sha, …)`;
  - `preview_binding(chapter, review)`;
  - `verify_tables(chapter, out, frames)`;
  - the job `csd-reviewed-final-<chapter>303`;
  - a launcher command with each of the four flags exactly once (`csd-launch-final303.py:75–81`).
- **303 against 287.** The diff changes PREVIEW274, the pinned receipt inputs, the permit `host` requirement, 287 in `EARLIER` and `VERSION`, and adds `host` and `preview_inputs_sha256` to the render summary. Nothing 293 reads changes.
- **Preview274 evidence.** The local receipt has exit 0, and its inputs equal its final inputs at `20d91bdb…`, which is 303's pin. All 27 local preview tables equal the timeline269 reconstruction.
- **Identity mapping.** `NARRATION` is rebind269 in both 287 and 303, so `schedule()`'s mapping is the identity for every CSD row.
- **Operational point.** 303's `check()` refuses any existing 287 job for the chapter, even a failed one. Such a chapter must stay a 287 row (293 accepts both families), go to CSS-290 (06/08 only) or wait for a later version.
- **Host field.** 293 does not read the permit's `host` field. 303's `check()`, which 293 re-runs, enforces it for CSD rows. Nothing enforces it for a CSS row's permit. That is harmless for the master, because the manifest names exactly one row per chapter.

**N4. Soundtrack and replay.**
- **Captures.**
  - Each chapter's last capture must be 48 kHz stereo s16 at frames × 800, and 293 stages it as `soundtrack_0.wav`.
  - The kit's master takes the last contiguous `soundtrack_N` (`cli.rs:959–962`). With one staged file, the two agree.
- **Cross-binary replay.** For the four retained rows, binary 268 replays captures that binary 19 mixed.
  - 273 §2 and §4 show the same `Cargo.lock`, the same `Cargo.toml` and the same 39 kit and 23 score files in both builds.
  - The linked code is not proven equal (273 U1).
  - The master's null test is the direct check; a failure spends 293.

**N5. The video join.**
- **How the join works.** The master joins with `-c:v copy` (`master.rs:203–255`), and MP4 output takes its parameter sets from the first file.
- **Evidence.** All 15 local 4K finals carry the identical `avcC` (High@5.2, `caf9d50d…`), with identical x264 options apart from thread counts. They came from six hosts and three binaries: 19, `5a66979e` (CSD's 08) and `352aa5ff`.
- **For the 268 rows.** The 268 renders on CSD use the same encoder settings.
- **Decoding.** 293 itself decodes nothing; the full decode stays with verify.

**N6. Disk and hashing.**
- **Disk growth.**
  - The parts are:
    - per-chapter video-only copies in `final/master/join/`, about the sum of the chapter MP4s;
    - the joined master, about the same plus 0.07 GB of AAC;
    - an F32 stereo master WAV of about 0.86 GB.
  - Production19's 15 chapter MP4s total 0.20 GB.
  - Counting the hard-linked staged files as well, growth is about 1.2–1.7 GB. That is within the 4 GiB phase cap and the 20 GiB target cap.
  - The notes' "about the size of all fifteen chapter MP4s" leaves out the join copies. That does not matter at these sizes.
- **Hashing.**
  - Each 303 row's inputs manifest binds the whole preview274 job folder, its 12,669-row manifest and timeline269's manifest.
  - 293 re-hashes every row's manifest during `check` and `launch`, and twice during `master`.
  - Expect minutes to tens of minutes. That is time, not a failure.

**N7. Contract inputs the master reads but 293 does not freeze.**
- **The files.** The native contract also reads:
  - the workspace268 topic's `narration_units.json`;
  - rebind269's review269 direction map, written into the workspace (`review-repair-rebind269.py:22–23,77`);
  - the generation receipts that `assembly.json` names.
- **Effect.** A drifted file would refuse, not mislead.
- **Optional fix.** Add those paths to `film()`'s file set.

**N8. Notes item answered: rebind269's narration folder.**
- **The open item.** For the verify copies, the notes ask whether rebind269's narration folder holds `report.json` and `phrase-timings.json`.
- **What the source shows.**
  - `review-repair-rebind269.py:87–112` writes `report.json`, `assembly.json` and `external-units.json`.
  - Its summary counts 106 unchanged files: the 105 WAVs plus one, which is `phrase-timings.json` from assembly19's set of 109.
- **Limit.** This comes from source and summary, not a listing on CSD.

## Checked and found sound

- **Whole-film gates (`film()`).**
  - It checks six receipts, binaries 268 and 19, and the 15-chapter registry (11 changed plus 4 retained, with no overlap).
  - All 45 tables match under one mapping, every header has fps 60, and the frames sum is checked.
  - Score21's files, QA and chapters are at their hashes.
  - The three contract files equal build268's `source-copy.json`.
  - The 105 WAVs are byte-identical to assembly19's.
- **Rows.**
  - They must be in exact registry order, with exact keys and 15 distinct jobs.
  - A changed chapter is never retained, and retained rows are pinned to plan183 and the terminal summaries.
- **Imported CSS provenance for CSS-290 rows.**
  - On CSD, the D-path rows of the CSS inputs manifest are compared with CSD's originals. Binary 268's own row shows the CSS render used CSD's binary bytes.
  - The outputs are bound through the render summary.
  - I agree with the notes that skipping 290's `check()` loses only what the CSS job's own guarded pre- and post-render `check()` asserted. The receipts it asserted are bound by hash in that job's inputs manifest.
- **Master body.**
  - It is 184's body, with `PATH=/usr/bin:/bin`.
  - The kit names give `complex_analysis_visual_master.mp4`, `.en.srt` and `.en.vtt` (crate name at `main.rs:28`).
  - The replay line's first token is the chapter key (the format is `{:<14}`), and the lines must come in registry order.
  - The ffprobe gates match the kit's own.
- **Captions.**
  - They are built from the staged timelines and rebind269's `report.json`, whose sentence rows are unchanged (rebind269 summary). They do not come from `script-reviewed.tsv`, so 304 item 7 does not affect them.
  - 293's own caption gate is minimal: the SRT is not empty and the VTT starts with `WEBVTT`. The summary keeps `exact_caption_check` false.
- **Guard.**
  - The invocation is the same as 287, 303 and 296.
  - The inner `master` asserts admission, no exit and policy 8/4.
  - Outputs and staged inputs use `'x'`, the job is never duplicated, and there is one launch with no retry.
  - Every acceptance flag in the summary stays `false`.

## Unresolved review and listening gaps (root)

- **No combined 268 final exists yet.**
  - Permits need an actual picture and motion review of preview274 and preview243. Root's review is in progress (`picture-review-progress305`).
  - 303 also needs `host: csd` in each permit.
- **Listening.**
  - Nobody has listened to a master, including me. The ElevenLabs Frederick Surrey narration is bound only by byte equality with assembly19.
  - Fine sync, loudness and true peak are unmeasured.
- **Open after the master.**
  - full 4K decode;
  - exact captions;
  - final pictures and motion on the master;
  - the join and picture waiters;
  - independent final AV acceptance.
- **Verify reuse.**
  - The versioned verify copies are not written yet.
  - The three shared helpers on CSD are unconfirmed (notes, last section).
- **Also open:**
  - E5, pixel identity of retained 02/03/13/15 under binary 268, and 273 U1;
  - publication306's final-master matching and the SRT binding.

## Limits

**What I read.**
- 293 and its notes;
- 287, and 303 with its launcher;
- 290's `preview_binding`, `check`, `verify_tables`, `required`, `launch` and `render`;
- the three production19 workers;
- `tools/narration_delivery.py` (`961f17cc…`, the same bytes as build268's copy);
- the kit's local `cli.rs` (`a63970e0…`) and `master.rs`.

**Kit sources.** Per 273, the compiled `cli.rs` (`3c5221fe…`) adds only the `adjust` hook to the local file. I did not see the compiled bytes.

**Not established here.**
- Nothing here is a compiled, rendered, decoded, pixel, motion, audio, listening, caption or AV result.
- Every check above comes from source and local evidence copies. None of 293's modes was run.
