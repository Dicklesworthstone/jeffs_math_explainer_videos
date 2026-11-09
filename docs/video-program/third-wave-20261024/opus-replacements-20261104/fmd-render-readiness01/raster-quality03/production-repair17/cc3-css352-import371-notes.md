# Import 371 of the second render host's 352 finals: notes

CC3, 2026-11-08. Prepared only. No import step has run on any host: no export, send, prepare, receive, render, master or permit, and no Beads or TODO change. Every check in section 8 only read: locally on a scratch tree, or in memory on the two hosts.

Names used here:
- D is the build host's job root (350's `D`), and Q is `D/qualification01/production-repair17`.
- `CSS` is 350's constant for the second render host's root, and `CQ` is `CSS/qualification01`.
- On the build host, `CSS` is the alias that css-a1 (`csd-retained-import311-css-a1`) created. It points to `D/work/imported-css-root311`, called TOP below.

| File | sha256 | Note |
|---|---|---|
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | new helper, 911 lines, in Q |
| `cc3-css352-import371-notes.md` | sent with the cover message | this note |
| scratch `test371.py` | `d5e8fe89b97065a99f12f46603f164415e830bd8d03ed5d65f3b2bb697b7ede2` | local tests, kept outside Q (root named two outputs); available on request |

`csd-retained-import311.py` (`86296883…`) and every other file are unchanged.

## 1. Why 311 cannot import a 352 chapter

I ran 311's own `manifest_problems` locally on the real 01 rows: the running job's admitted inputs `CQ/final-01_the_bump-inputs352.json` (8,927 rows, sha `b9591317…`, equal to its receipt's `inputs_sha256`), plus its job folder. It returns 2,270 problems:
- 2,269 of the form "row outside `CSS` and D". These are the `/usr` rows of section 3.
- One growth refusal. There are 2,197,503,449 foreign bytes, so the measured growth is 4.09 GiB, over the 3.5 GiB limit. Of those bytes, 1,368 MB are files that css-a1 has already placed.

Reading the code gives three more reasons:
- `closure_problems` checks production19's closure. 311 pins chapter 13 only (`CHAPTERS`, `HOSTS['css']['chapters']`), and its rules are binary 19, timeline21, worker 78, and the delivery76 narration and receipts. A 352 job binds binary 268, the timeline269 tables, delivery309 with its 105 WAVs, and worker 352.
- `job_name(host, attempt)` gives one job per host. css-a1 exists and has imported. So attempt 1 refuses "job exists", and any later attempt refuses with "already imported by …css-a1" (`earlier_attempts`).
- 311 creates the alias when it is absent. A 352 import must instead find css-a1's alias, and never make one.

So a 352 import needs a new version. 311 stays as it is.

## 2. What 371 changes, function by function against 311

Thirteen definitions are identical by AST:
- `Unavailable`, `clean_path`, `csd_side`, `flag`;
- `header_line`, `inputs_path`, `make_dirs`, `q_manifest`;
- `read`, `sha`, `sha_bytes`, `stage`, `utc`.

Three more differ only in a docstring, a message or the per-chapter key: `terminal`, `read_header` and `earlier_attempts`.

Three definitions were removed: `physical_root`, `pins` and `make_alias`. The changes, grouped by purpose:

1. **One chapter per job.**
   - `job_name(chapter, attempt)` gives `csd-css352-import371-<chapter with dashes>-a<N>`, and the inputs file is `Q/<job>-inputs.json`.
   - Every mode takes `--chapter`, with the ten 352 chapters as its choices, in place of `--host`.
   - `earlier_attempts` refuses a running or imported earlier attempt of the same chapter only.
   - The stream header names the chapter.
2. **Pins come from the job, not from constants.**
   - `export` refuses unless the render receipt is a terminal success. It then records the job; the worker (sha fixed at `1074051b`, as in 350 `WORKERS352`); the inputs, the permit, the receipt, the summary and the movie with their shas; and the frame count.
   - `pin_problems` checks the names 352 itself derives: the job `css-reviewed-final-<dashed>352`, the inputs `CQ/final-<chapter>-inputs352.json` (underscores kept), and a permit directly in `CQ/repair352-permits` whose name starts with the chapter key.
3. **Three row kinds** (`kind_of`): foreign, shared and system (section 3). A row in any other project tree (under the helper's `DATA` constant) is refused.
4. **A 352 closure.** `closure_problems` is rewritten for 352 (section 4).
5. **A fixed placement root that is never created.**
   - `alias_problems` requires `CSS` to be exactly css-a1's symlink to TOP, and TOP to be a real directory on the D/jobs filesystem.
   - `prepare`, `check` and `receive` (before and after placement) refuse otherwise.
   - `place` refuses when TOP is missing and never creates it. Nothing in 371 creates or changes a symlink.
6. **Own paths.** `own_paths` covers 311's three physical roots and its jobs, plus 371's own jobs and inputs.
7. **More checks before the first write.** `plan_problems` adds four:
   - every system file is at its hash;
   - root's Q copy of the permit is present: the same file name, directly in Q, at the permit's hash;
   - no `csd-reviewed-final-<dashed>*` job exists on the build host. CC1 asked for this per-chapter refusal; 352 makes the same check against its own frozen view;
   - the growth limit applies to new bytes only. Identical files that already exist are streamed to devnull, as in 311, and add nothing.
8. **The guard's whole-tree figures.** `preflight` checks that, after the new bytes, D/jobs stays below 56 GiB and D/work growth since `D/jobs/allocation-baseline.json` stays within 20 GiB, each with half a GiB to spare. `allocated()` is the guard's own function; its body is AST-equal to the guard's, checked on the build host.
9. **Binding and records.**
   - `prepare` binds in the guard inputs: the helper, the manifest, the guard, root's Q permit copy, and every shared and system row. It prints `chapter_pin`, the counts, `space`, `guard_command` and `rows350`.
   - `receive` asserts that the permit copy and every shared and system row are frozen at their hashes. At the end it re-hashes the system files, and it records `alias_created: false`.

Kept from 311:
- guard admission and the exact policy and command checks;
- writes with `x`/`xb` only, and staging inside the job;
- the closure re-check from the staged bytes;
- one hard link per new file, with the staged name kept;
- an identical existing file is kept, and a different one is refused;
- the four-hour `/usr/bin/timeout`, and a single `import-result.json`.

## 3. Row kinds, counted on the real 01 closure

| Kind | Where | Rule | 01 rows | 01 bytes |
|---|---|---|---|---|
| foreign | under `CSS` | streamed, staged, placed under TOP | 3,460 | 2,197.5 MB |
| shared | under D | never streamed; must be on the build host, symlink-free, at the bound hash | 3,203 | 1,994.1 MB |
| system | outside `CSS`, D and every other project tree (`DATA`) | never streamed; must be a row of one of the three build-host manifests below at the same hash, and at that hash on the build host | 2,269 | 127.7 MB |

The system rows come from 352's `carried_rows()`. It takes every row of `Q/review-repair-timeline269-inputs.json`, `Q/review-repair-preview243-inputs.json` and `Q/review-repair-preview274-inputs.json` that also exists on the second render host, asserts that its hash there equals the row, and binds it. Those manifests bind the build host's runtime:
- `/usr/bin/python3.14` and 2,263 files under `/usr/lib/python3/dist-packages`;
- `ffmpeg`, `ffprobe` and `bash`;
- `/usr/bin/timeout` and `/usr/lib/cargo/bin/coreutils/timeout`.

350's `chapter_gate` re-hashes every inputs row on the build host. These rows therefore have to hold there, and an import that dropped them would leave 350 reading them anyway.

Measured today: all 2,269 are rows of those manifests at the same hash, and all hash equal on the build host. `/usr/bin/timeout` is a symlink there; `sha` follows it, as 350 and the guard do. CC1 found the same independently. CC1's and CC2's first replies had said that no system file was bound; CC1 has since corrected this.

## 4. Closure checks

The closure is the 97 rule applied to the 352 job:
- the admitted inputs file and each of its rows;
- the receipt;
- the render summary and each of its files;
- every file in the job folder.

352's `launch()` freezes into those inputs every file its `render()` and `check()` read, and `render()` re-checks `required()` against them. That includes what 350 reads: the permit's evidence, source-binding and independent files, the delivery309 inputs and narration, the timeline269 tables, score21 and binary 268.

`closure_problems` runs on the second render host at export. On the build host it runs again from the staged bytes, before any placement. It checks eight things:
1. **Pinned files.** The worker is at `1074051b`. The permit, inputs, receipt and summary are at their pinned hashes. The binary 268 row and the chapter's three timeline269 tables are present.
2. **The render receipt.**
   - It is a terminal success, and its inputs hash is the pinned one.
   - Its policy is 10/8/4/56/20.
   - Its command starts `/usr/bin/python3 -I -B CQ/css-final-repair352.py render`, and `--chapter`, `--job`, `--inputs` and `--permit` equal the pins.

   CC0 pointed out why this matters: 350's launch waits only on `import_receipt` for a foreign row, so the render on the second render host must already be finished when the import happens.
3. **The 97 rule.** Every inputs row is held at its frozen hash, with no duplicates. The inputs bind:
   - the worker, the permit, binary 268 and `guard_css74.py`;
   - the delivery309 inputs and result, the transfer309 result and `css-runtime-manifest75.json`;
   - the receipts of `css-encoder-stop79`, `css-runtime-transfer75`, `css-repair-transfer309` and `css-repair-delivery309`;
   - the three build-host manifests;
   - all 105 narration WAVs.

   With `--after`, the predecessor's receipt is one more inputs row, so it travels as a foreign row.
4. **Delivery309.** The four receipts are terminal, and delivery309's receipt binds its inputs. Every delivery309 input is held at its hash. The delivery result names binary 268, 105 verified cues and the rebind269 narration as its source.
5. **System rows.** Each is a row of one of the three build-host manifests at the same hash.
6. **The summary.** It names host `css`, the chapter, the frames, binary 268, the permit sha, `worker352_sha256` and the delivery309 narration. Every output lies under `render/` and is held. The movie is at its pin. The five suffixes and a soundtrack capture are present.
7. **The permit.**
   - It names the chapter, host `css`, the frames and binary 268, and its `timeline_sha256` equals the timeline269 row.
   - Every evidence, source-binding and independent-review item is held at its hash, and the last two lists are not empty.
   - The independent files sit directly in Q (350 `independent_files`).
8. **The render timeline and cue log.** Timeline lines name delivery309 WAVs that are held. The header gives the chapter, 60 fps and the frame count. Cue-log rows sit under the narration or the score21 stems and are held at their hashes.

## 5. Interface with 350 and 363

- **CC0, 350 (`60575944`).**
  - A `CSS` row's keys are `ROW_KEYS | {import_receipt, import_inputs}`.
  - `imported()` needs `D/jobs/<job>/resource-receipt.json` and inputs under D, and re-hashes every inputs row. Those rows now include the system rows, which `chapter_gate` re-hashes anyway.
  - `chapter_gate` reads the job's files at their `CSS` path strings through the alias, and needs the independent files in Q.
  - 371 prints, and records in `import-result.json`, `rows350 = {chapter: {import_receipt: D/jobs/<job>/resource-receipt.json, import_inputs: Q/<job>-inputs.json}}`.
  - CC0's draft `master350-manifest-draft384.json` (`ddbe3f2b…`) already uses these names at attempt a1 for 01, 04, 05, 12 and 14.
- **CC2, 363.** It reads the job's summary, receipt, movie and staged files and the worker through the alias. It does not read `import_receipt` or `import_inputs`.

## 6. Relay for root, one chapter at a time

Start a chapter only after its render on the second render host is a terminal success; `export` refuses before then.

1. Copy the helper (`ba337aca`) to Q on the build host and to `CQ` on the second render host.
2. On the second render host, run `python3 -I -B CQ/csd-css352-import371.py export --chapter <chapter>`. Save stdout, without clobbering anything, as `Q/css352-import371-<dashed>-manifest.json` (a suggested name). A non-zero exit lists the refusals on stderr.
3. Copy that manifest to Q on the build host and next to the helper in `CQ`. Check that the three shas are equal.
4. On the build host, run `/usr/bin/python3 -I -B Q/csd-css352-import371.py check --chapter <chapter> --manifest Q/<manifest>`.
   - It must exit 0 with `problems: []` and `alias_state` "symlink to `D/work/imported-css-root311`".
   - It reports the new files and bytes and the guard's space figures.
5. Run `prepare` with the same arguments. Record `inputs_sha256`, `guard_command`, `chapter_pin` and `rows350`. Compare `chapter_pin` with the job's terminal receipt and summary on the second render host.
6. Pipe the two ends, as for 311, and record both exits: `ssh <second render host> 'python3 -I -B CQ/csd-css352-import371.py send --chapter <chapter> --manifest CQ/<manifest> --manifest-sha <sha>' | ssh <build host> '<guard_command>'`.
   - Start the pipe only while the build host's native lane is free. At 22:28Z, `csd-reviewed-final-09-boundary344` held it.
   - Success means both exits are 0 and `D/jobs/<job>/import-result.json` exists. The guard returns 75 on a resource stop.
7. Run `check` again with the same attempt. It re-hashes every row through its original path and prints `rows350`.
8. Put `rows350` into that chapter's row of the 350 manifest.

Chapters can be imported in any order:
- The first chapter writes the 2,610 delivery309 runtime copies under `CSS/work/frozen-runtime75` (727 MB) and the delivery309 job (96 MB).
- Each later chapter adds only its own job folder, its permit-transfer files and its inputs, which come to tens of MB.
- Every import still streams all of its foreign bytes, about 2.2 GB at 01's size. Bytes that are already placed are verified and discarded.

If an attempt fails or is refused, use the next attempt number. That changes the job name in the 350 row, so tell CC0. A failed attempt's staged files stay in its job folder and keep counting in D/jobs; nothing here authorizes cleaning them up.

## 7. Sizes and guard figures for 01

These were measured read-only at 22:28Z, while 01 was still rendering, so the new rows exclude its outputs.

| Foreign rows | Count | MB |
|---|---|---|
| already placed by css-a1, identical | 724 | 1,368.4 |
| new: `CSS/work/frozen-runtime75` (delivery309 runtime copies) | 2,610 | 727.2 |
| new: `css-repair-delivery309` (105 WAVs, result, receipt and the rest) | 111 | 96.1 |
| new: `CQ` files (worker 352, helper 309, permit, manifests, inputs) | 7 | 4.5 |
| new: `css-repair-transfer309` | 2 | 1.2 |
| new: the 01 job folder so far, and `css-repair-permit352-01-the-bump364` | 6 | 0.0 |

New bytes total 829,059,714. Twice that is 1.54 GiB, against the 3.5 GiB limit. The outputs will add tens of MB: 13's 4K movie was 13.5 MB, and a soundtrack capture is frames × 3,200 bytes.

Guard figures on the build host:
- 97.4 GiB free;
- D/jobs at 9.45 GiB of 56;
- D/work at 5.12 GiB above its baseline, of 20.

The second render host has 23.7 GiB free; `export` and `send` write nothing there.

## 8. Checks run

Local scratch test (`test371.py`, helper `ba337aca`): 11 of 11 pass.
- **Constants.** The worker sha equals the local `css-final-repair352.py` and 350 `WORKERS352`. 35 helper constants equal their counterparts in 352, 350 and 311. Job, inputs and import names fit 352, the guard's job-name characters and 350 `imported()`.
- **Kinds.** Row kinds come out as above, other project trees are refused, and own paths are recognised.
- **Pin refusals (12):** two chapters, chapter 09, chapter 13, another worker sha, the build host's 352 worker, another job, other inputs, a permit outside `repair352-permits`, another chapter's permit, a bad hash, zero frames, an extra key.
- **Round trip.** On a scratch tree with css-a1's alias and folder already in place: staging, then placement into the existing folder. The alias is unchanged, staged names are kept as second links, and modes are kept. A re-run writes nothing. The guard-style `allocated()` counts the bytes in each tree.
- **Existing identical files.** One is streamed to devnull, kept at its inode, and left out of the growth figure; one byte over the limit refuses.
- **Stream refusals:** a wrong hash, chapter, count or magic; no header; a truncated, trailing or corrupted stream. Nothing is placed in any of these cases.
- **Destination refusals:**
  - the alias absent (never created), TOP absent (never created; `place` refuses), the alias pointing elsewhere, `CSS` a real directory;
  - a divergent file (kept);
  - a changed shared file, a changed system file, a symlinked shared file;
  - the Q permit copy absent or different;
  - a build-host final of the same chapter (other chapters' finals and 09 are ignored);
  - a symlink inside TOP;
  - a racing writer (never overwritten);
  - an earlier attempt of the same chapter that is running or finished.
- **Guard caps.** The 56 GiB work cap and the 20 GiB target cap are checked with the new bytes.
- **Commands.** The guard command is exactly the guard argv, then `--`, then the receive argv.
- **Source scan.** Writes use `x`/`xb` only. There is one `os.link` and one `os.chmod`, no symlink creation, and no remove, rename, replace, truncate, `subprocess` or `shutil`.

On the second render host, in memory, read-only, at nice 19, with helper `ba337aca`:
- `export` refuses, because 01's receipt is not terminal.
- The real 01 closure passes `manifest_problems`. `closure_problems` reports only four problems, all due to the unfinished job: no summary, a non-terminal receipt, and no render timeline or cue log yet.
- All 19 mutations are refused with a named reason:
  - dropping a permit evidence, independent-review or source-binding file, a narration WAV, a frozen copy, the delivery309 inputs, a build-host manifest, the worker, or a bound receipt;
  - a system row at another hash, marked shared, or not carried;
  - a row in another project tree or in TOP;
  - a frozen row marked shared;
  - a wrong frames or inputs pin;
  - altered permit bytes, and a permit naming host `csd` (both refused by the permit hash).
- The receipt of the running job was read once as a snapshot, because it changes every 10 s.

On the build host, in memory, read-only, at nice 19 and idle I/O priority, against the real state and the same manifest:
- `plan_problems` returns 0 problems. All 3,203 shared and 2,269 system files are at their hashes. The 724 existing destinations are identical, and 2,736 rows are new.
- The alias and TOP are as css-a1 left them, and root's Q permit copy matches.
- No `csd-reviewed-final-01*` or 371 job or file exists.
- `allocated()` equals the guard's.

## 9. Open items

- **Nothing has run for real.** 01 was still rendering at 22:28Z, so no real manifest exists yet.
- **Independent review is required before execution.** CC1 holds task 374 (`Q/cc1-import371-review374.md`) and gets these hashes.
- **The one-host check is point-in-time,** as it is in 352.
- **System rows depend on the build host's `/usr` staying as it is until 350 runs.** A package update would make 350 refuse every row that binds those files.
- **One field check is exercised only by code reading.** The "permit names host csd" mutation was refused by the permit's hash before the host field was read. `closure_problems` does compare the field, but no test on real data reaches that comparison.
- **Already-placed bytes are re-streamed (311's behaviour).** A list of files to skip would need a second relay leg, from the build host back to the second render host, so it is not done.
- **The scope is exact import only.** These are hash and structure checks. Nothing was viewed or listened to, and they do not accept any picture, motion, audio, AV or release.
