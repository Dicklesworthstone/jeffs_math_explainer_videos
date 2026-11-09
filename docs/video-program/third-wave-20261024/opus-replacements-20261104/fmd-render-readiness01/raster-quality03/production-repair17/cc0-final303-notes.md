# cc0 final303: CSD worker and launcher that bind preview274

cc0, 2026-11-08. The two scripts are prepared only.

**Not done:**
- no launch or dry run on csd;
- no deploy;
- no render;
- no permit;
- no Beads or TODO edits.

**Preserved:** worker and launcher 287 are unchanged, and 293 is untouched.

## Files

| File (in `production-repair17/`) | sha256 |
|---|---|
| `csd-reviewed-final303.py` (worker, new) | `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` |
| `csd-launch-final303.py` (launcher, new) | `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` |
| `csd-reviewed-final287.py` (preserved, base) | `f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510` |
| `csd-launch-final287.py` (preserved, base) | `80d9ab1159251ba372d2e7199761d2d6fba20113ea7e3c69e9f2bb1217597bba` |

## What 303 changes against 287

The worker diff against 287 contains only these changes.

1. **The real preview274 is named** (worker:47–51). Each value comes from CC3's helper (`cc3-preview-fade274.py:3–15`) and from csd's records.

   | Field | Value |
   |---|---|
   | job | `csd-review-repair-preview274` |
   | manifest | `review-repair-preview274-inputs.json` |
   | worker | `cc3-preview-fade274.py` at `d80cf7b6…` |
   | tables | `preview/` |
   | narration | rebind269 (identity mapping) |
   | binary | `review-repair268` (`983954c5…`) |
   | build | build268 (source copy `81d28272…`) |
   | receipt inputs | `20d91bdb8585fb229c105c717a0cf486e544b3b37b92f46907e009fe695fb38c` |

   - **Routing.** 287's `PREVIEW274 = None` and its fail-closed assert are gone. The nine chapters (01, 04, 05, 07, 09, 10, 11, 12 and 14) now route to preview274. 06 and 08 stay on preview243 through the build238 bridge and binding282, unchanged.
   - **No delta item.** Preview274 has the same build as the final, so it takes 287's `build == BUILD268` branch (worker:97). That branch needs no 238-to-268 delta item, but asserts that the preview binary equals the final binary.
2. **Each preview receipt's inputs hash is pinned** (worker:82).
   - Preview274 must be the run with inputs = final inputs = `20d91bdb…`, finished 10:50:17 UTC.
   - Preview243 is pinned at `6510db78…`, read from its csd receipt.
   - A rerun under the same job name is refused.
3. **Prior 287 exclusion:** `EARLIER = ('244', '278', '287')` (worker:64, used at worker:109). An existing 244, 278 or 287 job folder for the same chapter refuses 303.
4. **One chapter, one host** (CC2 300 L4; worker:55, :125). The permit must carry `"host": "csd"`.
   - This is the only new permit field.
   - CSS's 290 does not check the field, so it protects CSD only. Root still assigns each chapter to one host in writing.
5. **Version** (worker:65, :171). `VERSION = '303'`, and the job must be `csd-reviewed-final-<chapter with dashes>303`.
6. **Render summary** (worker:205). It also records `host` and `preview_inputs_sha256`.

**Kept exactly as in 287:**
- every upstream receipt gate (stop13, timeline21, score21, build268, rebind269, timeline269);
- the 45-table equality with timeline21;
- score21 files and per-chapter stem QA;
- every permit gate:
  - actual pictures `true`;
  - actual motion `true`;
  - `unresolved_defects == []`;
  - reviewer and observations;
  - timeline hash;
  - frames equal to the header;
  - motion intervals;
  - evidence and source-binding hashes;
- the absolute, symlink-free permit path;
- the preview table equality with timeline269;
- the scene-file equality;
- the guard receipt policy 8/4;
- the frozen inputs, re-hashed before and after the render;
- the 4K60 command `--crf 16 --preset veryfast --score --lint` with a 43,200 s timeout;
- the ffprobe and x264 `crf=16.0` checks;
- the soundtrack length;
- the second `check()` after the render;
- the exclusive `render-summary.json`.

**Launcher, against 287:**
- **Worker hash before any worker code runs** (CC3 297 F2). The new required `--worker-sha256` is asserted on `csd-reviewed-final303.py` before `exec_module` (launcher:32, :37).
- **Permit hash.** `--permit-sha256` is required and asserted (launcher:34), as in `launch-reviewed-preview296.py`.
- **Version.** It asserts `worker.VERSION == '303'` and uses job and `final-<chapter>-inputs303.json` names built from it (launcher:38, :42, :71).
- **CPUs before any file is written.** It refuses fewer than 16 inherited CPUs (launcher:45), so that refusal leaves no inputs file.
- **Unchanged from 287:** the freeze set, which covers:
  - the launcher, worker and permit;
  - the guard and the binary;
  - the timeline269 and preview job receipts, manifests, every manifest row and every job-folder file;
  - the evidence and the binding;
  - the narration;
  - score21;
  - the stop13 receipt and build268's qualification.

  The guard call (`--phase native`, `--threads 16`) is also unchanged.

## csd facts read

These were read-only SSH reads on 2026-11-08, at 17:47 and again at 17:59 UTC.

- **Preview274.**
  - Receipt: exit 0, no error, no resource stop; inputs = final inputs = `20d91bdb…`.
  - Command: `/usr/bin/timeout 172900 /usr/bin/python3 -I -B …/cc3-preview-fade274.py`, so the worker path is a list element, as worker:83 requires.
  - Manifest: `20d91bdb…`, 12,669 rows (1.29 GiB). All rows were at their hashes at 17:59, including binary 268, the preview worker, build268's `source-copy.json` and `qualification.json` (`2f3430a8…`), and 109 of 109 rebind269 narration files.
- **Tables.** All 27 preview274 tables (nine chapters × timeline/cues/cuelog) are byte-equal to timeline269's. The only narration folder in them is rebind269's.
- **Timeline269 manifest.** `2c4e86e5…`, 12,600 rows. Preview243's manifest is `6510db78…`, 12,607 rows. All rows were at their hashes.
- **Upstream receipts.** Build268, rebind269, timeline269, timeline21, score21, stop13, preview243 and build238 all show success. Score21's per-chapter stem QA passed for all nine.
- **Nothing exists yet:** no `csd-reviewed-final-*` job, no `final-*inputs*` file, no permit and no 303 file on csd.
- **Pins on csd:** 287 `f01f4c29…`, launcher 287 `80d9ab11…`, binding282 `1fbcedef…`.
- **Guard chain.**
  - `guard_csd02.py` → `guard_owner56g_target20g.py` (`1c19a9b8…`) → runner `run_bounded_phase_oliver_owner48g_target14g_20261102.py` (`819746fb…`).
  - The runner holds `expanded-native-lane.lock` (runner:101–105), and csd's guard points it at the csd root. So every csd guard job, finals included, runs one at a time.
  - The child is started with `start_new_session=True`, and nothing changes its CPU affinity.
- **Load.** csd was at 27.5 on 64 CPUs at 17:59 UTC, from other work.

## Mock check

The mock is scratch-only and outside the repo. It builds a fake csd tree and runs the real 303 worker and launcher code against it, with the real preview worker files at their pinned hashes. **43 of 43 cases behaved as expected.**

- **Binding.**
  - **Pins and mapping.** The real `PREVIEW274` pins equal the final binary and source copy. The nine chapters map to preview274, and 06 and 08 to preview243.
  - **Positive paths.** 01 binds through preview274 with no binding282 item, and 06 still binds through the 238 bridge.
  - **Refusals:**
    - a wrong preview job;
    - receipt inputs other than the pinned run, for 274 and for 243;
    - a changed preview worker hash;
    - a preview table that differs from timeline269's;
    - a changed 268 source copy;
    - a 274 entry pointing at another binary.
- **Full `check()` with fake upstream records.**
  - **Positive:** 01's permit passes.
  - **Refusals:**
    - no host, or host `css`;
    - motion not reviewed;
    - an unresolved defect listed;
    - empty `source_binding_evidence`;
    - wrong frames;
    - a permit naming preview243;
    - an existing 244, 278 or 287 job;
    - stem QA not passed;
    - another chapter's table differing from timeline21's.
  - **Not an earlier job:** a 303 folder does not count as an earlier job for `check()`.
- **Launcher.**
  - **Refused before the worker file runs:** a wrong worker hash, and a wrong permit hash. The worker file was a stand-in that exits if executed.
  - **Under 16 CPUs:** refused, with no inputs file left behind.
  - **Positive run, with a stand-in worker module.** The freeze set holds the launcher, worker, permit, guard, binary, preview274 receipt, manifest, rows and tables, the narration, the evidence and the binding. The guard argv names job `csd-reviewed-final-01-the-bump303`, `inputs303` and `--threads 16`.
  - **Second launch:** refused by the existing `inputs303` file, without calling the guard.
- **Not covered by the mock:** `run()` (render, ffprobe, encoder string, soundtrack) is unchanged from 287 and was not exercised, and nothing ran on csd.

## Before any 303 launch (root)

1. **Deploy.** Copy both 303 files into csd's `qualification01/production-repair17/` at the hashes above.
2. **Finish the actual reviews.** Root's `picture-review-progress305.json` (17:52:59 UTC) shows:
   - 690 of 996 pictures and 89 of 129 sheets reviewed; 10 and 11 not started;
   - **07 on HOLD:** sampled frames are blank inside cue 07_6, and CC3 counts 142 empty frames after a fade. A source-aware review is pending.
   - **14 I5:** a progressive-reveal disposition from CC3 is pending;
   - **09:** the selection jumps from 2.6 s to 138.883 s, so it does not cover the Cauchy proof interval;
   - motion review, audio, fine sync and final AV acceptance all not done.

   Preview274's own summary also flags:
   - **11:** G3 (one not localized);
   - **14:** I5;
   - **layout:** one layout finding in 01 and two in 09.

   303 needs `unresolved_defects == []`. Each item must therefore be fixed, or recorded in the permit's observations as not a defect.

   **If 07 or 14 needs a source fix**, the result is a new build and binary. 303 pins binary 268, so it cannot render that chapter, and a later version must bind the new build and preview.
3. **Permits, one per chapter, written by root after the actual picture and motion review.** I made none. Each is an absolute, symlink-free path on csd with these fields:

   | Field | Value |
   |---|---|
   | `binary_sha256` | `983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7` |
   | `chapter` | the chapter |
   | `host` | `"csd"` |
   | `preview_job` | `csd-review-repair-preview274` (preview243 for 06 and 08) |
   | `actually_reviewed_pictures`, `actually_reviewed_motion` | `true` |
   | `unresolved_defects` | `[]` |
   | `reviewer`, `observations` | filled in |
   | `timeline_sha256` | the hash of timeline269's `<chapter>.timeline.tsv` |
   | `frames` | the header value |
   | `evidence` | `[{path, sha256}]`, the sheets actually viewed |
   | `motion_intervals` | `[[a, b]]`, with 0 ≤ a < b < frames |
   | `source_binding_evidence` | non-empty |

   **Source-binding evidence.**
   - **Nine 274 chapters:** any real items at their hashes. For example, the preview274 manifest at `20d91bdb…` and build268's `qualification.json` at `2f3430a8…`.
   - **06 and 08:** exactly one item named `build238-to-build268-delta` for binding282, as in 287.

   **Frames per header**, as recorded for preview274:

   | Chapter | Frames |
   |---|---|
   | 01 | 6,311 |
   | 04 | 6,774 |
   | 05 | 9,879 |
   | 07 | 6,491 |
   | 09 | 16,504 |
   | 10 | 7,442 |
   | 11 | 11,932 |
   | 12 | 3,943 |
   | 14 | 11,756 |
4. **One host per chapter.** Record which host renders each chapter.
   - CSS's 290 knows only preview243 and so cannot render the nine (CC2 300 L5).
   - A 287 run of 06 or 08 on csd blocks 303 for that chapter, by design.
5. **Hash check on csd, immediately before invoking the launcher:**

   ```
   cd /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17
   sha256sum -c <<'EOF'
   db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057  csd-reviewed-final303.py
   def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0  csd-launch-final303.py
   d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669  cc3-preview-fade274.py
   20d91bdb8585fb229c105c717a0cf486e544b3b37b92f46907e009fe695fb38c  review-repair-preview274-inputs.json
   1fbcedef34f65933ff3d818740d124cb2492a39e276916eb600f6753d5c62066  cc1-preview-to-final-binding282.md
   EOF
   ```
6. **Launch, one chapter per call:**

   ```
   /usr/bin/python3 -I -B /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/csd-launch-final303.py \
     --chapter 01_the_bump --permit <absolute permit path> --permit-sha256 <permit sha256> \
     --worker-sha256 db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057
   ```

## Operational notes

- **Runs are serialized.** Because of the lane lock, a second launch waits for the first final, or for any other guard job, to finish. Its inputs are re-hashed after it gets the lock.
  - **CC3 297 F1 does not arise through the guard.** The concern was finals sharing CPUs 0–15. A `taskset -c` range is still inherited by the worker, but it is not needed to keep finals apart.
- **Time, estimated.**
  - The only measured rate is HZ2's 09: 16,504 frames in 4 h 11 min, at 1.1 fps on 12 threads.
  - At that rate the nine take about 20.5 h one after another (81,032 frames), or about 26.8 h with 06 and 08 (106,021 frames).
  - Each chapter stays well inside the 12 h limit.
  - csd's own 16-thread rate has not been measured.
- **Disk.** On the same 8/4 policy, the receipt from HZ2's final09 peaked at 0.132 GiB of growth, against the 4 GiB native cap. That is one run on another host.
- **A refused admission blocks retries.** The runner creates `jobs/<job>/` before testing admission (runner:107). It writes a receipt with `admitted` false and exits 75, and the launcher's `final-<chapter>-inputs303.json` also remains.
  - **To retry:** move both aside by renaming them; nothing is deleted.
  - **Earlier versions:** a refused 244, 278 or 287 folder also blocks 303 for that chapter until moved aside.
  - **Input changed while queued:** the runner raises after admission (runner:136), before any receipt is written, which leaves the job folder empty.
- **The freeze is broad.** At launch the launcher re-hashes about 25,000 manifest rows (about 2.5 GiB) and both job folders (1,211 files in preview274 and 65 in timeline269). Any later write in those folders fails the final at its last re-hash (worker:203), so keep review records outside the preview274 job folder.

## Limits and independence

- **Not independent.** I wrote 287 and 303. CC3 reviewed 287 (297), but nobody else has reviewed 303 yet.
- **Not run on csd.** Neither script was run there, even as a dry run. The mock is a scratch tree with fake records and stand-in binaries.
- **Narration.** It stays the current ElevenLabs Frederick Surrey set from rebind269. 303 binds those exact files through the timeline269 and preview274 manifests. No audio heard or checked.
- **No acceptance.** This is not picture, motion, fine-sync, 4K or final-AV acceptance, and it is not a permit. Even after a successful render, the render summary still says full decode, master replay, final pictures, motion, audio, captions and publication remain.
- **Ownership.** Root owns deployment, permits, launches, Beads and TODO.
