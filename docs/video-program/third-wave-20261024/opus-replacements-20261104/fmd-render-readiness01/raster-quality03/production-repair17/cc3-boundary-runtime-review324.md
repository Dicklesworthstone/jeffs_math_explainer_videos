# Review 324: actual runtime of the 313 chain (build, rebind, timeline)

cc3, 2026-11-08, written at about 19:40 UTC. This is an independent, read-only audit of the records the 313 jobs actually wrote on the build host.

Paths use the helpers' own names:
- D is the build host's project root;
- Q is `D/qualification01/production-repair17`;
- job folders are `D/jobs/<job>`.

**What I did.** Over ssh, I listed, read and hashed files with `python3 -I -B` scripts piped on stdin. Nothing was written on the build host. I started, stopped or signalled no job and no controller. Root's two controllers ran throughout, and I left them alone:
- `review-repair-after-build313.py`;
- `preview-after-timeline313.py`.

I edited no source, candidate, permit, TODO or Beads file. This file and its `.json` twin are my only outputs.

**What this is not.** It is a review of records and bytes only. I make no picture, motion, 4K, listening, fine-sync, pronunciation or AV acceptance, and no audio was heard or checked. Preview313 was still running when I wrote this, so no repaired picture has been seen in this review. "Preview313" below lists what I will view once its pictures are transferred.

## Result

**Build313, rebind313 and timeline313 each finished with a successful receipt, and every claim I could check against bytes holds:**
- Each receipt is finished and admitted, with exit 0 and inputs equal to final inputs. None has an error or a resource stop, and each uses the current policy.
- Each job's bound-input manifest in Q hashes to its receipt's `inputs_sha256`. I re-hashed every row of each, and all rows still match.
- Exactly one source file changed from build268: `boundary.rs`, which now equals the repair310 candidate.
- All 105 WAVs are byte-identical across rebind313, rebind269 and assembly19. I hashed every file myself.
- Chapter 09's three tables equal timeline269's and production timeline21's under the named path mappings. I re-did both comparisons myself on the raw bytes. The total is 16,504 frames.

**No findings of any severity.** Three observations are recorded below, none of them a defect: a Cargo `error:` line in the build logs, the by-product thumbnail movie that timeline-only runs write, and the timeline's run time against its estimate.

## 1. Build313

**Receipt.** `jobs/csd-review-repair-build313/resource-receipt.json`, sha256 `44cdc80b22402ffd606c5c97a6d488ec70c2c0153068badb974e5b614ba91304`:
- Started 19:05:23, finished 19:17:52 UTC.
- Admitted, exit 0, phase native, not preflight-only.
- `inputs_sha256` = `final_inputs_sha256` = `e96410ba29c5e1e96692ff159355ee2fd428ed62846281ffa9c19b3499404399`. There is no error or resource-stop key.
- The policy is the current one: admission 10, stop-free 8, phase growth 4, work cap 56 and target growth cap 20 GiB, with 10 s samples.
- There are 72 samples. Free space was 104.05 GiB at admission and never below 103.52 GiB. Work plus target grew by at most 0.05 GiB.
- The command is `/usr/bin/timeout 22000 /usr/bin/python3 -I -B Q/review-repair-build313.py`.

**Bound inputs.** `Q/review-repair-build313-inputs.json` hashes to the receipt's `inputs_sha256`. I re-hashed all 12,422 rows (1.21 GB): none is missing and none has changed.

**Qualification.** `qualification.json`, sha256 `1e0116f7ea850eab0afb959c6ad9faf3bcd7806ca48408200aad82bdb954afc6`. Each field below was checked against the job's own logs and files:
- **`binary_sha256`** is `1a82f72c53dba2d5db8a3478bac252e46bf711afd59ec9d425ee6639708611db`. It equals my hash of `D/work/bin/review-repair313` (18,466,496 bytes).
- **`base_binary_sha256`** `983954c5…` equals my hash of `D/work/bin/review-repair268`.
- **`tests_passed`** is 233. `tests.stdout` has one result line: "233 passed; 0 failed; 1 ignored".
- **`explicit_native_plate_fade_probe_passed`** is backed by `plate-fade-probe.stdout`: `plates_fade_in_pixels_on_the_films_own_frames ... ok`, 1 passed.
- **`clippy_passed` and `fmt_check_passed`.** Both fmt-check logs are 0 bytes, and the Clippy log ends `Finished release`.
- **`facts_sha256`** `7d528509…` equals my hash of `facts.stdout`, and equals build268's `facts_sha256`.
- **`chapters`** lists the 15 chapters 01–15, the same list as `chapters.stdout`.
- **`changed`** lists only `src/chapters/boundary.rs`.

The build helper gates on exit statuses and on these test-result lines (helper :68–70 and :158–162).

**Source copy.** `source-copy.json`, sha256 `cb4d402ef6086aebd10dcf18e95df5f5bc69b89a2e613bbbc2cd7420a9d6d5b8`:
- **Keys.** It has 70 keys, the same set as build268's `source-copy.json`. That file is `81d28272…`, which is also this file's recorded `base_source_copy_sha256`.
- **The one changed value.** Exactly one value differs from build268's: `boundary.rs`, from `dc80c541…` to `bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53`.
- **The candidate.** That new hash equals my hash of `Q/boundary-guide-repair310.rs`, and the file's `substituted` field names that candidate.

**Live workspace** (`D/work/review-repair313/workspace`, 72 files):
- 69 of the 70 copied files still equal `source-copy.json`.
- The 70th is `narration_units.json`, which rebind313 rewrites by design (section 2).
- The two extra files are rebind313's backup, `narration_units.before-review313.json`, and its map, `docs/narration-direction-map-frederick-review313.json`.

**Observation O1: a Cargo `error:` line that is not a failure.**
- **The line.** `build.stderr` and `clippy.stderr` both contain `error: unclosed table, expected ]`. Cargo prints it while scanning a deliberately malformed test fixture in a dependency checkout (`…/tests/fixtures/migration_readiness_planner/malformed/Cargo.toml`).
- **It predates 313.** Build268's `build.stderr` carries the same message. A `diff` of the two logs shows only the workspace name and the build time (15.78 s against 13.13 s).
- **Effect.** No 313 gate reads this text. Anyone grepping these logs for `error:` should expect it.

## 2. Rebind313

**Receipt.** `jobs/csd-review-repair-rebind313/resource-receipt.json`, sha256 `35a2dbdb302ae1fcccf03d2ca181233db02c96d8ff9f2958713d951ae608741b`:
- Started 19:21:14, finished 19:21:22 UTC.
- Admitted, exit 0, with no error or resource-stop key.
- `inputs_sha256` = `final_inputs_sha256` = `28ab671cab755c9c4c1e3a4ccad008738c47d1b1b027ebb62cecfd4d51f622bf`.
- The policy is current. There are 2 samples. Free space never fell below 100.41 GiB, and growth was at most 0.09 GiB.
- The command is `… Q/review-repair-rebind313.py --run`, under `timeout 172900`.

**Bound inputs.** `Q/review-repair-rebind313-inputs.json` hashes to the receipt's `inputs_sha256`. I re-hashed all 12,628 rows (1.33 GB): none is missing and none has changed.

**Summary.** `rebind-summary.json`, sha256 `359052a7f1d01b5811b1817bccd3164cffa9f97e40cb56a48faf8c54caed6bd3`, records:
- 23 holds and 5 helpers;
- `boundary_holds_moved`: 09_4 938→942, 09_5 1003→1007 and 09_8 1282→1286;
- `other_holds_and_helpers_equal_review269`, `report_sentence_rows_unchanged` and `generation_records_unchanged`, all true;
- 105 `wav_files_byte_identical` and 105 `wav_files_equal_rebind269`, with 106 unchanged files;
- `map_sha256` `c3ccd2f2…` and `canonical_sha256` `9308c2b2…`.

Two more records back the summary:
- **`delivery`**: 105 verified and 105 selected cues, plan mode `external-ui`.
- **`coherent.stdout`**: "supplied script/unit/generation binding plus required direction coverage plus sample reconstruction verified; listening is separate".

**My own checks of those claims:**
- **File names.** The three narration folders hold the same 109 file names: rebind313's (`jobs/csd-review-repair-rebind313/narration`), rebind269's and assembly19's.
- **WAVs.** All 105 WAVs are byte-identical across all three folders. I hashed every file in each folder.
- **The files that differ.** Only `external-units.json`, `report.json` and `assembly.json` differ, both against rebind269 and against assembly19.
- **Inside those three files.** I mapped the narration folder name and compared them key by key against rebind269's. The differences are exactly these:
  - the direction map hash, `7eb0e40d…` → `c3ccd2f2…`;
  - the units hash, now `9308c2b2…`;
  - the three boundary holds' evidence hash, `dc80c541…` → `bda7d931…`, and their line numbers in the reason text, +4 each;
  - the closing words of all 23 hold reasons, "listening unreviewed." → "no audio heard or checked.". The 313 notes record this change at :76, and the coherent checker requires only a non-empty reason;
  - `assembly.json`'s `report_sha256`, which follows from the report changes.
- **Workspace files.**
  - `narration_units.json` hashes to `canonical_sha256`.
  - The backup equals build313's copied `narration_units.json`.
  - The map hashes to `map_sha256`.
- **The map against rebind269's.** The new map has the same 23 holds; only the three boundary holds differ, in line and file hash. The helpers are equal.
- **The two rewritten units files.** Rebind269's and rebind313's differ only in the direction map hash, once 269 names are mapped to 313.

## 3. Timeline313

**Receipt.** `jobs/csd-review-repair-timeline313/resource-receipt.json`, sha256 `420b422137996fb2d440392ec6cfc242e33f33efad2409b5eff8b788aa940301`:
- Started 19:21:28, finished 19:37:22 UTC.
- Admitted, exit 0, with no error or resource-stop key.
- `inputs_sha256` = `final_inputs_sha256` = `ebdf8c137c2e323219733f40ef719c5c7334f493652edf927d4793e8fc8c67b1`.
- The policy is current. There are 91 samples. Free space never fell below 100.15 GiB, and growth was at most 0.05 GiB.
- The command is `/usr/bin/timeout 172900 /usr/bin/python3 -I -B Q/review-repair-timeline313.py`. The job's `native.stderr` is empty.

**Bound inputs.** `Q/review-repair-timeline313-inputs.json` hashes to the receipt's `inputs_sha256`. I re-hashed all 12,885 rows (1.51 GB): none is missing and none has changed. The rows include:
- rebind313's 109 narration files and rebind269's 109;
- 60 paths under timeline269's output folder;
- 69 paths under production timeline21;
- binary 313.

**Comparison record.** `comparison.json`, sha256 `e63cbc5e97bc2037e62153f9f48d37e2ef935a112f0103132f13aa7d081ae033`:
- **`frames`** is `{09_boundary: 16504}`.
- **`named_path_mapping`** maps rebind269's narration folder to rebind313's.
- **`production_path_mapping`** maps assembly19's narration folder to rebind313's.
- **Both equality flags** are true for each of the three tables.
- **`command.log`** reads "09 timeline, cues and cuelog equal timeline269 and production timeline21 after named path mapping."

**My own comparison.** I compared the raw bytes again, without using the helper's flags. The table below lists the new file, then each old file before mapping:

| Table | timeline313 sha256 (bytes) | timeline269 | timeline21 | = 269 mapped | = 21 mapped |
|---|---|---|---|---|---|
| `09_boundary.timeline.tsv` | `5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e` (1,183) | `e05a6679…` | `ae7a13f4…` | yes | yes |
| `09_boundary.cues.tsv` | `eed5a50ad5797fa258ee3fcf9319e29946f403a86b3909090e73a3a15dc74249` (177) | `eed5a50a…` | `eed5a50a…` | yes | yes |
| `09_boundary.cuelog.tsv` | `7ed46c80cede7e49daeaf729337b24c66c8e7ccc125bb18ca6ce2c411d201bbf` (2,296) | `a36389b7…` | `cb69ba38…` | yes | yes |

My hashes equal the record's `new_sha256`, `timeline269_sha256` and `production21_sha256` for every table.

Two further checks on the new timeline table:
- **Header.** It reads chapter `09_boundary`, fps 60, frames 16504.
- **Line rows.** All 9 line rows name a WAV in rebind313's folder, and each named WAV equals rebind269's file of the same name.

**Observation O2: the by-product thumbnail movie.**
- **What it is.** Timeline-only runs also write `timeline-output/timeline/09_boundary.mp4`: 64×36, 60/1, 16,504 video frames, with an AAC track. Timeline269 and production timeline21 wrote the same kind of file for each chapter.
- **Not checked by the helper.** It neither compares nor hashes these movies. `comparison.json` lists only the folder name, `timeline`.
- **The movies differ.** Their sha256 values:
  - 313: `964d1e1b…`;
  - 269: `e63fbbd8…`;
  - 21: `f16b1219…`.
  
  A picture difference is what the repair should produce, but I did not use these thumbnails for any picture judgement.
- **Their audio is identical.** For all three movies, both the copied AAC packet MD5 (`0f174b59…`) and the decoded PCM MD5 (`dd3b2e82…`) are equal. Both are byte comparisons, not listening.

**Observation O3: run time.** The render reported 947.5 s, at 17.4 fps; the job took 15 min 54 s. The 313 notes estimated about 12 min (:220). The timeout is 172,900 s, so nothing was near a limit.

## 4. Imports (context only)

The three retained imports ran one after another, between build313 and rebind313. Each receipt is terminal and successful:
- admitted, exit 0, inputs equal to final inputs;
- no error, and the current policy;
- `receive --attempt 1`, under `timeout 14400`.

| Import | Receipt sha256 | Started–finished (UTC) | Inputs |
|---|---|---|---|
| trj | `c262a7aa7adfb1bd86e8605869001d9776ceb3238f8e83ad759079db534fa0d2` | 19:17:52–19:18:03 | `1666f58e…` |
| css | `246633e296fa1abb5a50e8042c3582fe5818dded1e7c0f9321b150eb5dd7b065` | 19:18:09–19:18:59 | `f94ed318…` |
| hz2 | `c54af76ad5cbc48d8863fb3298feadd548116db6368dbcc35294536980869449` | 19:19:00–19:21:08 | `37d3b6f8…` |

I read only the receipts. I did not re-check the imported files themselves.

## 5. Preview313: state now, and what I will view

**State at 19:39 UTC.** Root's controller launched preview313 under the guard. Its receipt was sha256 `8b9ac821…` at that moment, and it will change as the job runs:
- started 19:37:37, admitted, not finished;
- `inputs_sha256` `3ca1a2949bd48389204cb1a8eae4f93519ad7e9ac3ebbb40aa87b71103ef6f19`, with the current policy.

`Q/review-repair-preview313-inputs.json` hashes to that value. I re-hashed all 12,903 rows, and all still match. They include:
- the control movie, `jobs/csd-review-repair-preview274/preview/09_boundary.mp4`, which I hashed as `dba964d2…`, the worker's `CONTROL_SHA256`;
- timeline313's `comparison.json` and its three tables;
- binary 313.

**What I will read once the job finishes and root transfers its outputs:**
- **The receipt.** Terminal success, plus the rows of its bound-input manifest.
- **`summary.json`.** `zoom_probe_fails`, `zoom_probe_exit_frame`, `flags` and `not_localized`.
- **`zoom-probe.json`.**
  - The control window.
  - The new exit frame against the enforced window, 2086–2100 inclusive (the 320 L1 disposition).
  - Each absent anchor, with the per-frame counts for both movies: 2326, 4180, 5417, 6240, 6900, 6960 and 7020.
  - The flagged frames.
- **`sampling.json`.** Every sample's full reason list, since the sheet captions show only the first reason. Also the schedule rows, the fades and `fade_flags`, and the sheet list.

**What I will view.** Every `sheet-*.jpg` and the individual pictures behind them, each opened and looked at, never inferred from hashes or from the probe. On them I will check:
- **28–40 s, every 0.25 s.** The lines fade in at about 28.5 s, are held while the close-up is introduced, and leave with "g is undefined at a".
- **Every frame from 2080 to 2106.** The exit itself, and nothing left behind after it.
- **Every frame from 2316 to 2336.** `|g| ≤ M` arrives near 2326 with no line through it.
- **Root's three samples and the 312 L3 crossing**, each at −0.2, 0, +0.2, +1 and +2 s:
  - 33.48 s, where the lines should still show;
  - 69.67 s, upper line through `|g| ≤ M`, lower line through the cancellation text;
  - 90.28 s, upper line through `|g| ≤ M`;
  - 104 s, lower line into "L: arc + chord lengths".
- **115, 116 and 117 s, at the same offsets.** The lower line through "for every small ρ".
- **39–133 s, every second.** The persistence sweep: the column must be clear in every frame.
- **133 s, frame 7980.** This falls inside the base movie's `proof_gone` fade, where the base lines were last present at 7988 (313 notes :159). The lines must not reappear there (312 :45).
- **The standard picks.** Line and mark starts, both plate sites, the lint lines, and the first and last frames.

The probe passing will not count as picture acceptance. As the 313 notes say (:229, :236), it reads 21 points outside the column; only the pictures can show the column clear. Listening, fine-sync and final AV stay open whatever the pictures show.

## Not established

- **No repaired picture has been seen.** Every check here is of records and bytes. Two things remain unobserved: that the zoom lines are gone from about 34.8 s, and that the column is clear over 39–133 s.
- **No audio was heard or checked.** WAV and AAC byte equality shows the narration is the same Frederick samples that assembly19 delivered. It says nothing about how they sound or how they sync.
- **The other 14 chapters were not re-run under binary 313.** This is by design (313 notes :84; 314 L3). A final that binds any chapter other than 09 to binary 313 needs its own 15-chapter comparison first.
- **The imports.** Only their receipts were read.
