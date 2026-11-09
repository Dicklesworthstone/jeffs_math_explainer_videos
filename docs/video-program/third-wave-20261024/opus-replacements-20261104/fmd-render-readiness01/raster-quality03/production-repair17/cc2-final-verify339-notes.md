# CC2 339: fix for CC0 review 332 M1 (picture and sheet bytes checked before binding)

cc2, 2026-11-08. Files:
- `csd-final-verify339.py`: sha256 `1e200220a1a7fd794abe318c0a1d815c188976f57c0707902c71915345dc05af`, 979 lines.
- These notes.

**Status.** Prepared only. Nothing in 339 has run on the master, on CSD or anywhere else. Nothing was deployed, launched or rendered, and no remote command was run for 339. No source, Beads or TODO file was edited. No picture, motion, listening, fine-sync, 4K-detail, caption-reading or independent AV acceptance is claimed. Every review flag 339 writes is False.

**What is preserved.** These are unchanged:
- `csd-final-verify330.py` (`4fc4594d…`) and `cc2-final-verify330-notes.md` (`9e11d396…`);
- 321, its notes, 328 and the 318 helper;
- CC0's `cc0-final-verification-review332.md` (`75cc0fa1…`).

339 is a new file, copied from 330 and then edited.

D is the CSD root and Q is `D/qualification01/production-repair17`. Line references are to 339.

## 1. The finding (332 M1) and the fix

**The gap.** In 330, `part_done()` re-hashed only the overview job's *inputs*. The native launch then bound the overview folder's *current* bytes. Nothing compared an overview picture or sheet with the sha256 that the overview job wrote into its own `sampling.json`. CC0 reproduced this through 330's own launch and job code: seven changes made between the overview job's success and the native launch were all accepted, and the native job then recorded the union as exact.

**The fix in `part_done()`** (:808–827) adds the following. It runs after the existing `stable()`, plan and flag checks, and before anything is returned for binding.
1. **No repeated paths.** Every recorded `{path: sha256}` must be distinct: the samples for the overview job, the natives for a native job, and the sheets.
2. **Exactly the recorded pictures.** The job's `pictures/` folder (`native4k/` for a native job) must hold exactly the recorded paths: no file missing, and no file or folder added (`iterdir`, :822).
3. **Exactly the recorded sheets.** The `sheet-*` files must be exactly the recorded sheets, and there must be `ceil(samples/8)` of them (none for a native job).
4. **Unchanged bytes.** Every recorded picture and sheet must still have its recorded sha256 (:825–826). The refusal names the file.

**Where it runs.** `part_done()` runs at the native launch, before the inputs file is written, so a refusal spends nothing. It runs again inside the native job, after the guard's own re-hash.

**The anchor for a consistent forgery (332 C7).**
- **The case.** A picture changed *together with* its recorded sha256 in `sampling.json` cannot be seen from inside the folder.
- **What 339 adds.** It follows CC0's fix 2, minus the optional argument:
  - the overview job prints its `sampling.json` sha256 as its last line (:907–909);
  - the native launch prints the hash of the `sampling.json` it binds, as `overview_sampling_sha256` (:969–975);
  - the native job records that hash in its `union` (:859–860).
- **What root does.** Keep the overview job's last line, and compare it with the native launch's line or with `union.overview_sampling_sha256`.
- **It detects; it does not prevent.** The native job starts as soon as its launch prints. To prevent a native run on a forged record, root would hash the overview `sampling.json` read-only before the native launch. Or a later version could take the recorded hash as a required argument, CC0's optional item. I did not add that, to keep 339 minimal.

**Unchanged from 330:**
- the exact sample and reason union (:855–860);
- every guard and gate;
- the two fixed parts, overview then native, with their 3.5 GiB upper budgets under the unchanged 4 GiB guard cap;
- the plan and its full M1 coverage;
- L1–L4;
- the 318 contract.

**AST check against 330:**
- The only functions that differ are `part_done`, `pictures` (the union's anchor field and the overview's last line) and `main` (the launch's anchor and the 339 message). None was added or removed.
- Among constants, only `VERSION`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES` and `PICTURES_INPUTS` differ. The new names are `csd-final-verify339`, `csd-final-pictures339-<part>` and `final-…339-…-inputs.json`.
- The 16 functions that read 318's records and the guard records are still AST-identical to 321's.

## 2. CC0's cases, run through 339's own launch and job code (fixture, §3)

Every case is applied after the overview job's receipt is final and before the native launch. It is then restored, and 339's `part_done` accepts the overview again.

The "330" column is 330's own `part_done()`, loaded on the same tree with its job names pointed at it. It is the control that reproduces 332.

| Case | Change in `csd-final-pictures339-overview/` | 330 | 339 |
|---|---|---|---|
| C0 | none: full overview → native → union run (fixture C) | accepts | **accepts**: union exact, all coverage checks hold |
| C1 | sample 5: one byte XOR 0xFF in place | accepts | **refused** at the native launch: `changed since the overview job recorded it: frame-000005.jpg`. No native inputs, no guard call |
| C2 | sample 3 overwritten with sample 4's bytes (a valid JPG of the wrong frame) | accepts | **refused** (`frame-000003.jpg`), as C1 |
| C3 | sample 2 truncated to 500 bytes | accepts | **refused** (`frame-000002.jpg`), as C1 |
| C4 | sample 7 missing (moved aside) | accepts | **refused**: `the overview pictures are not the ones it recorded` |
| C5 | `sheet-0002.jpg`: one byte flipped | accepts | **refused** (`sheet-0002.jpg`), as C1 |
| C6 | an extra picture: a copy of sample 1, at the first unused name `frame-008248.jpg`* | accepts | **refused**: `the overview pictures are not the ones it recorded` |
| C6b | an extra sheet: a copy of sheet 1, as `sheet-9999.jpg` (an added case) | accepts | **refused**: `the overview sheets are not the ones it recorded` |
| C7 | sample 1 flipped, and its `sha256` rewritten in `sampling.json` | accepts | accepted inside the folder (as CC0 found). **Detected by the anchor:** the bound hash printed at the native launch differs from the overview job's last line |
| C8 | sample 5 flipped after the native inputs were written | refused | **refused** at native admission (the bound-row re-hash), naming the file, before any native output |
| C9 | one sample's reasons changed in the overview `sampling.json` (fixture G) | refused | **refused** by the native union check, before any native frame |

\* CC0's fixture had 13 samples, so `frame-000099.jpg` was unused there. This fixture has 8,247, so the first unused name is used instead.

## 3. Tests (scratch, not in the repo; local only; nothing real decoded or extracted)

**`unit_test339.py`: 61/61.** This is 330's 59 unit checks retargeted to 339, plus the 330→339 AST delta of §1. It covers:
- M1 records, windows, sites and instants against the records;
- the L1 framemd5 positives and negatives, including real ffmpeg 8.1.2 output;
- L3 on all of plan182's reasons;
- the budget arithmetic;
- L4 locally;
- 318 identity.

**`fixture_test339.py`: 115/115.** This is 330's whole end-to-end suite, run under 339's names on the same synthetic CSD tree at the real 133,662 frames, with the real plan182 and the 21 records at their pins. Block H adds the cases above.
- **Inherited from 330:**
  - the 318 negatives;
  - record pins;
  - the budget refusals;
  - L2 free space at both launches;
  - the L4 bindings and their refusals;
  - L1 pts gap and timebase;
  - the tampered master frame;
  - the full positive run;
  - the reason forgery.
- **What changed in the fixture.** Its fake overview JPEGs now use well-separated colours. In 330's fixture, neighbouring solid colours could encode to identical bytes, which made C2 a no-op. Block H checks first that each case really changes the folder.
- **Real:** the shared verifier, the caption screen, PIL, and the local tools.
- **Faked:** ffmpeg and ffprobe runs, the strict script, the guard, CPU affinity, free space and `ldd`.

**Not repeated for 339.** The CSD runtime probe of 330 (`runtime()`, `pillow_runtime()`) applies unchanged, since those functions are identical. No new remote command was run.

## 4. Other points from review 332

- **332 L1: the CSD framemd5 timebase.**
  - CC0 reports that root's CSD probe342 (`c3e833dc…`) found `#tb 0: 1/60`, pts 0, 1, 2 and duration 1 under ffmpeg 8.0.1 (Lavf62.3.100).
  - I have not read probe342. Nothing in 339 changes framemd5.
- **332 M1 fix 3: re-hash before viewing.** Not in 339, because root's 339 order is the check before binding.
  - No job checks the native PNGs after the native job, or the overview files after it.
  - `part_done()` already handles a native part (natives, no sheets). So a read-only re-hash of both jobs' recorded outputs before viewing would be a small later addition, if root wants it.
  - Until then, root can hash the files listed in both `sampling.json` files read-only before viewing.
- **332 N2.** `verified()` binds the verify job's outputs at their current bytes, the same pattern. It is inherited unchanged from 321, and is outside this order.

## 5. Deploying and running (root's)

**Deploy.** This is 330 notes §5 with 339 in place of 330. Before `check`, Q on CSD must hold these at their exact bytes:
- `csd-final-verify339.py` at `1e200220a1a7fd794abe318c0a1d815c188976f57c0707902c71915345dc05af`;
- the 318 helper, the three shared checks and plan182, at the pins in 330 notes §5;
- the 21 records at the hashes in `TRANSITION_SOURCES` (:197–219).

**Run.** The order is 330 notes §6, plus two records for root:
1. `check`.
2. `launch-verify`, then the verify job.
3. `launch-pictures`, which starts the overview job. **Record its last line's `sampling.json sha256`.**
4. `launch-pictures` again, after the overview job succeeds. This starts the native job. **Compare its printed `overview_sampling_sha256` with the hash recorded in step 3.**
5. Root views every sheet and native frame, ideally after a read-only re-hash (§4).

**Retry rule.** Unchanged from 330.
- **Refusals before inputs spend nothing.** This now includes an overview picture or sheet that is changed, missing or extra. That state has to be explained before the native job can launch. Restoring the exact recorded bytes is enough for 339 to accept again.
- **A job once its inputs exist.** If it then fails, that job is spent for 339.

## 6. Hygiene

The only path in `csd-final-verify339.py` is its CSD project root, as for 330. These notes name no host path. Compile checks for 339 were in memory only, so no new byte-code file was written.
