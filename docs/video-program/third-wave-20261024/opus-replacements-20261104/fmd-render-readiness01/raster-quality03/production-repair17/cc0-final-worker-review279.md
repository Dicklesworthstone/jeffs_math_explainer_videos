# cc0 review 279: final278 worker and launcher (source audit, prepared only)

## Subject

Paths are relative to this file's folder (`production-repair17/`). Every hash below was taken during this audit.

| File | sha256 |
|---|---|
| `csd-reviewed-final278.py` (worker) | `7d1dc9b203660f294353cff221c2db3822c54dc669464eeb5882431fb0f1f2ee` |
| `csd-launch-final278.py` (launcher) | `67372165b8166e537addc62fe33ad35cb50aafe10c18662374ad55fb070e3f62` |
| `final278-preparation.json` | `4decb8942c07f297444e2a6953c2aef239cbc0d1f8c584a6b690e90089e727b3` |
| `csd-reviewed-final244.py` (preserved244, worker) | `03dc1c92673ce508873c56281b943f1abab3eed20c5e68ff95c83e8695b2571c` |
| `csd-launch-final244.py` (preserved244, launcher) | `72224ccafa16f83910c8f966be8f45373ebf721a4c0bcbb8f3dcebbbe412d894` |

The worker and launcher hashes equal those in `final278-preparation.json`, whose status is `prepared_not_deployed_or_run`.

**Contracts read:**
- **Build268.** `review-repair-build268.py`, `review-repair-recipe268.json`, `transfer-review-repair268.json` and `review-build268-terminal272/`: the qualification, receipt and source copy. The receipt shows exit 0, inputs equal to final, and no resource stop.
- **Rebind269 and timeline269.** `review-repair-rebind269.py`, `review-repair-timeline269.py`, `review-repair-after-build269.py` and `review-rebind269-terminal276/`. The rebind receipt shows exit 0, inputs equal to final, and no resource stop. Delivery verified 105 of 105 cues.
- **Earlier links.** `review-repair-build238.py`, `review-repair-timeline242.py`, `review-repair-after-timeline243.py` and `review-repair-preview243.py`.
- **Impact and review records.** `cc1-combined-impact261.md` and `preview243-root-review-complete277.json`.
- **Candidates.** The recipe268 files, all seven of which match their recipe hashes, and production19 `src/chapters/` in `production-source-preserved214/`.

**Not seen:**
- `guard_csd02.py`, which is not in this folder.
- The CC3 preview274 helper, which is still being prepared.
- The remote job folders and the score21 summary.
- The timeline269 terminal record. Timeline269 is still pending in the preparation record.

**Independence.** Root prepared final278; I wrote none of it. I wrote review224 and the 06 repair candidate 231 that build268 carries.

**What I did not do.** I ran nothing remote, launched nothing, built and tested nothing, and edited no source, frozen or canonical file and no Beads. This file is my only write.

## Result

There is no blocker.
- **Gate.** The actual picture/motion gate is not weakened against preserved244. Every review assertion is kept, and the frame check now comes from the real timeline header instead of a constant.
- **Selection and tables.** The 11-chapter selection and the 45-table normalization match the build268 and timeline269 contracts exactly.
- **Findings.**
  - **M1, Medium.** For chapters 06 and 08 the new source-binding guard checks only that evidence is present and its hashes match. It does not show what the evidence proves.
  - **L1–L5, Low.** One open naming dependency on CC3, and four inherited or operational items.
- **Present permits.** No record that exists yet could honestly pass the gate for 06 or 08. Root's 277 and my review270 both exclude motion, and the worker requires `actually_reviewed_motion is True` (worker:43).

## Checks

### 1. All-11 selection: correct

- **The lists.** The worker's `CHAPTERS` (worker:14) and the launcher's `choices` (launcher:13) are the same 11 keys in the same order. They are a subset of build268's 15 chapters, in build268's order. The excluded chapters are 02_turning_numbers, 03_two_planes, 13_real_integral and 15_closing. The worker asserts the chapter at worker:24, so these four cannot be launched.
- **What build268 changes.**
  - Recipe268 equals recipe262 except for draw.rs: candidate 264 replaces 260.
  - The 260→264 diff is lines 1887–1889 only (`add_updater` → `add_dt_updater`). That lies inside `a_raster_plate_fades_through_its_opacity_lane` (line 1875), in `mod tests` (from line 1440), so the runtime is identical. cc1's impact table for recipe262 therefore carries over to 268.
- **My own check against production19 `src/chapters/`, outside `#[cfg(test)] mod tests`.**
  - The registry maps 02→`turning`, 03→`two_planes`, 13→`real_integral`, 15→`closing` and 14→`continuing`.
  - `turning`, `two_planes`, `real_integral` and `closing` contain no `Painter`, no `portrait(` and no `PLATE`.
  - All 11 selected chapters either have a replaced scene file (05, 06, 08, 09, 10) or a raster entry point (01, 04, 05, 07, 09, 10, 11, 12, 14).
  - 03 reads `gallery::TITLES_TEX`, and that item is byte-equal in gallery249.
  - 13 and 15 read only from `the_bump`, which recipe268 does not replace.
- **Limit.** Retaining the existing finals for 02, 03, 13 and 15 still depends on cc1's E1–E6 evidence: stems, joins, and the family-cache risk. final278 does not gate that, and it is outside its scope.

### 2. All-45 table normalization: matches the contract

- **Baseline check.** Worker:32–34 compares every build268 chapter (all 15) × timeline/cues/cuelog against timeline21. It swaps only the bytes of `csd-production-assembly19/narration` for `csd-review-repair-rebind269/narration`.
- **Same check upstream.** This is the same comparison as `review-repair-timeline269.py:41`, which also asserts 15 unique chapters (:26) and a total of 133,662 frames (:45).
- **Output tables.** `verify_tables` (worker:56–82) checks the render's tables against timeline269. It is unchanged from 244. Its narration replace is a no-op, because `ROOT` and `D` are the same path.
- **Narration.** Rebind269 keeps all 105 WAVs byte-identical to assembly19 (rebind269:85, 111–113) and changes only `external-units.json`, `report.json` and `assembly.json`. The direction map is the Frederick production19 map, so the narration stays ElevenLabs Frederick. No audio heard or checked.

### 3. Frame and header validation: correct and stronger than 244

- **Header format.** Worker:51–52 parses the timeline's `chapter`, `fps` and `frames` lines. In the real format (`preview243-review265/preview/06_one_derivative.timeline.tsv`) each is exactly two tab-separated fields, so the dict comparison works.
- **What the frames bind to.** The permit's `frames` must equal the timeline269 header, which item 2 pins to timeline21's frames. The same `frames` then drives:
  - `verify_tables`;
  - the ffprobe `nb_frames` check at 3840×2160, 60/1 (worker:111);
  - the soundtrack length of `frames×800` (worker:117).
- **Against 244.** The launcher's old constant `{06: 10030, 08: 14959}` became `frames > 0`. That is not a weakening, because `check()` runs first (launcher:20).
- **Residual.** Duplicate header lines would be silently overwritten. The tables are binary output pinned to timeline21, so this is minor.

### 4. Score, binary and review guards

- **Score.** Unchanged from 244 (worker:26, 36–37, 54):
  - the score21 receipt;
  - `all_qa_passed`;
  - score chapters equal build268's 15;
  - every score file hash;
  - the per-chapter QA `passed`.

  I could not see score21 locally, so the equality of the chapter lists is unverified here. It is the same assertion 244 carried against build238's list, and build268 asserts the same chapter list as build05 (build268:39).
- **Binary.** `BINARY_SHA` is `983954c5…34d6ec7` (worker:10), equal to `review-build268-terminal272/qualification.json`. The worker checks the binary file, the qualification and the permit (worker:25, 29, 42). The receipts for build268, rebind269 and timeline269 must all succeed (worker:27).
- **Review.** Unchanged from 244, with additions:
  - `actually_reviewed_pictures` and `actually_reviewed_motion` must be `True`;
  - `unresolved_defects == []`;
  - reviewer and observations must be non-empty;
  - `timeline_sha256` must equal timeline269's table;
  - evidence and motion intervals must be non-empty and their evidence hashes must match.

  New in 278:
  - `preview_job` must equal the chapter's preview (worker:41);
  - the chapter's preview receipt must succeed (worker:39);
  - source-binding evidence is required and its hashes must match (worker:47–48).

  The permit is re-checked after the render (worker:118), and every frozen file is re-hashed before and after (worker:93, 119).
- **No synthesized authorization.** Neither file writes a permit. The worker's only subprocesses are the binary's `render` and `ffprobe`.

### 5. No duplicate job semantics

- **Launcher.** The job name is fixed per chapter as `csd-reviewed-final-<key>278` (launcher:22, worker:87). The launcher refuses an existing job folder (launcher:23). It writes `final-<chapter>-inputs278.json` with exclusive create (launcher:48), so of two simultaneous launches for one chapter, one fails.
- **Worker.**
  - It requires an admitted receipt with no `exit` (worker:88);
  - it requires that `render/` does not exist (worker:103);
  - it creates `native.stdout`, `native.stderr` and `render-summary.json` exclusively (worker:106, 121).
- **No chaining.** Nothing launches another job.

## Findings

**M1, Medium: the 06/08 source binding is presence-only, and the permit no longer names the binary that made the reviewed pictures.**
- **The split.** For 06 and 08 the permit must name `preview_job = csd-review-repair-preview243` and binary 268. Preview243, however, was rendered by `review-repair238` with timeline242/rebind242 (`review-repair-preview243.py:3, 11`).
- **In 244 they were the same binary.** The previewed binary and the final binary were both 238 by construction.
- **In 278 the only bridge is root's `source_binding_evidence`.** The worker requires it to be non-empty, hash-matching and frozen, but it does not check what the evidence shows (worker:47–48, 94). Any file with a correct hash passes.
- **`timeline_sha256` changes meaning too.** For 06 and 08 it binds timeline269, not the preview243 timeline that was actually reviewed.
- **The delta I verified, for root's evidence.**
  - Build238 = production19 + boundary52 + one-derivative238 + loop-sums238 + mod236 (build238:16–17).
  - Build268 adds mod256, rigidity247, gallery249, draw264 and the probe file.
  - mod256 against mod236 adds only `gallery::UNLESS_NOTE` to the text list.
  - The draw264 runtime change (lines 269–326) is `image_records` plus a `PlateFade` added only when a faded mob's family holds an image record.
  - Neither 06 nor 08 (nor `one_point`, `two_planes`, `residues` or `turning`, which they import) has `Painter`, `portrait(` or `PLATE` outside tests. So 268 adds no plate fade to them.
  - The remaining difference is cc1's family-cache timing risk: `family()` is filled earlier. It is not shown to change any output.
  - Both timeline242 (:19) and timeline269 (:41) equal timeline21 after their own path mapping, so the 06 and 08 schedules agree by these contracts.
- **Proposal: mechanical checks.** Before any 06/08 launch, either root records this delta as the binding evidence, or the worker checks it. The checks would be:
  - (a) the preview job's `preview/<chapter>.{timeline,cues,cuelog}.tsv` equal timeline269's after mapping the preview's narration folder;
  - (b) the preview's binary hash is either `BINARY_SHA`, or 238 together with a named delta record;
  - (c) the chapter's scene file hash is the same in build268 `source-copy.json` and in the preview build.

  Rendering 06/08 again under 268 would remove the question, but that is root's decision.

**L1, Low: the preview274 names are unconfirmed (CC3 owns them).**
- **The hard-coded names.** The worker and launcher hard-code the job `csd-review-repair-preview274` (worker:38, launcher:28) and the manifest `review-repair-preview274-inputs.json` (launcher:28) for nine chapters: 01, 04, 05, 07, 09, 10, 11, 12 and 14.
- **Status.** CC3's helper is still being prepared, and I have not seen its names. They are not confirmed here.
- **Preview243 for comparison.** Its manifest name is confirmed by `review-repair-after-timeline243.py:23`.
- **If CC3's names differ, or it splits the preview into several jobs,** the launch fails closed (missing receipt or hash mismatch). It does not fail unsafely.
- **Coverage.** The worker also does not check that preview274 contains the chapter, or which binary rendered it. M1's checks (a) and (b) would cover that.

**L2, Low: the superseded 244 launcher can still start a 06/08 final with binary 238.**
- 278 has no exclusion across versions: it does not refuse when `csd-reviewed-final-<key>244` exists or is running.
- I found no local record that 244 was ever launched.
- Recommendation: root retires 244 in writing, or 278 refuses when a 244 job folder exists.

**L3, Low (inherited from 244): the permit path must be absolute and free of symlinks.**
- The launcher freezes `args.permit.resolve()` (launcher:24), but the worker looks up `args.permit` as given (worker:94, 100).
- A relative or symlinked permit path therefore fails closed after guard admission. The same applies to `helper` against the worker's `Path(__file__).resolve()` if the qualification path ever contains a symlink.

**L4, Low (inherited): whole job folders are frozen.**
- The launcher freezes every file under the preview job and the timeline269 job (launcher:39).
- Any later rewrite of a listed file fails the final. That might be extraction outputs or notes written into the preview274 folder after the receipt. Rewrites caught at worker:119 fail only after the 4K render has run.
- New files are harmless.

**L5, Low (operational, inherited): CPU pinning.**
- Each worker pins itself to the first 16 CPUs of its inherited affinity (worker:101–102).
- With 11 chapters, concurrent launches on one host would share those cores.
- Fleet distribution is still listed as pending.

**Notes (not defects, unchanged from 244).**
- `motion_intervals` is checked only for shape (`0 ≤ a < b < frames`), not for coverage of the repaired spans.
- `unresolved_defects == []` cannot tell an accepted, deferred Low from an open one. Root should record accepted Lows, such as review270's R1, R2, L2, L3, F3 and N1, in its observations or disposition.

## Not established

- **Not run.** I did not execute either script, even as a dry run. I saw no remote file: not the guard, score21, the timeline269 terminal, the preview274 helper, or any permit.
- **Static only.** This is a source and contract audit. It is not picture, motion or AV acceptance, it does not authorize any launch or publication, and no audio was heard or checked.
