# cc0 master350: the combined master for final344 (09) and the ten 352 chapters

cc0, 2026-11-08. Q is this folder locally, or `D/qualification01/production-repair17` on CSD, where D is the CSD job root. CSS is the CSS render-worker root (309's `CSS`).

**What this is not.**
- I launched, built and rendered nothing, and ran nothing on any host. I edited no permit, record, worker, TODO, Beads or memory file. I wrote only `cc0-combined-master350.py` and these notes, both new.
- `cc0-combined-master318.py` is unchanged (`478b44a5…`).
- 350 accepts no picture, motion, audio, fine-sync, caption, final-AV, 4K, release or publication review, and it does not authorize a launch.
- No audio heard or checked.

## 1. Answer

`cc0-combined-master350.py` assembles the master from the renders root will actually have:
- **09** comes from CC2's final344, `csd-reviewed-boundary-final344.py`, job `csd-reviewed-final-09-boundary344`.
- **The other ten changed chapters** come from CC2's frozen 352 workers:
  - CSD: `Q/csd-reviewed-final352.py`, job `csd-reviewed-final-<dashed>352`;
  - CSS: `CSS/qualification01/css-final-repair352.py`, job `css-reviewed-final-<dashed>352`, through a CSD import.
- **02, 03, 13 and 15** stay the retained production19 finals, on their historical permits. Their gate is 318's `permit_gate()`, byte for byte.

Every source, audio105, schedule, table, score, import, encoder, replay, caption and resource gate of 318 is kept. The 18 functions that hold them are AST-identical (§3).

The 344 and 352 permits are one schema: 344's 21 fields and its `sampled_review_scope()`. 350 accepts that schema only on the 09 row and the ten 352 rows. It refuses:
- a permit with a `review_scope` on a retained row;
- a historical permit (with `actually_reviewed_motion` or `motion_intervals`) on 09 or a 352 row.

The master runs binary 268's native `captions all` unchanged. Caption347 is bound by root separately, outside this file.

## 2. Files

| File | sha256 | Notes |
|---|---|---|
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | 1381 lines, new |
| `cc0-combined-master318.py` | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | base, unchanged |
| `csd-reviewed-final352.py` | `23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760` | CC2, frozen; pinned in 350 |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | CC2, frozen; pinned in 350 |
| `csd-launch-final352.py` | `06451123b958db9d4806e2645fd84d8e0b37d604867dae9f25d4cb3c5155859e` | CC2, read only |
| `cc2-final352-notes.md` | `4479363724325af8b0f53417dfe6822df9998634ea968e4213d531d1c6a72fee` | CC2, read in full |
| `csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | CC2, frozen |
| `csd-launch-boundary-final344.py` | `70278078c8fd5e1cb414afcf02c3ff107a9dd956f0b2551a1ca0bdca360d75b4` | CC2, read only |
| `cc2-boundary-final344-notes.md` | `86da6e2319f2f8adafcc56a2d2f27cd3af51d94a041e8c04077810c3ec296d01` | CC2 |

The 318→350 `diff` is 533 lines: 482 added, 51 removed. Its sha256 is `01cb804c681723799d07a4afb382cc150ac90cd8090efb93e55e3b43262d5092`, and it is kept in my session scratch, not in the repo.

## 3. Exact delta against 318, by AST

**Functions identical (18):**
`absolute`, `capture_check09`, `completed313`, `contract_reads`, `film`, `flag`, `gates`, `header`, `imported`, `last_capture`, `load`, `permit_gate`, `provenance313`, `read`, `retention`, `schedule`, `sha`, `success`.

**Functions changed (6):**
- `chapter_gate`: the 09 row comes from 344, and the combined rows from the two 352 workers. The new permit dispatch, summary gates and encoder gate are described in §4.
- `check`: prints each row's review basis and, for sampled rows, its intervals.
- `launch`: its refusal message names 350.
- `master`: the summary adds `review_bases` and four acceptance flags stated false, and the result is refused if it would claim anything.
- `preflight`: the folder is `combined-master350-preflight-a<n>`.
- `replays_of`: the docstring names final344.

**Functions added (10):**
- `claimed`
- `pinned_scope`
- `disposition344`
- `sampled_scope`
- `sampled_permit_gate`
- `frame_basis` (:590)
- `combined_scope` (:666)
- `independent_files` (:698)
- `x264_settings`
- `independent344`

**Function removed:** `independent317`, which becomes `independent344`. Its first half is now `independent_files`.

**Constants.**
- Changed: `VERSION` `'350'`, `JOB` `csd-combined-master350` and `INPUTS` `Q/combined-master350-inputs.json`.
- Removed: `WORKER317` and `JOB317`.
- Added for 09:
  - `WORKER344`, `JOB344`, `PERMIT344_KEYS_SHA`, `PERMIT344_FLAGS`, `SAMPLED_SCOPE_SOURCE_SHA`;
  - `MOTION_KEYS`, `SCOPE_KEYS`, `OPEN_REVIEWS`, `CLAIM_WORDS`;
  - the four record names, `DISPOSITION344`, `DISPOSITION345_SCOPE`;
  - `PREVIEW313_MOVIE09`, `PREVIEW313_SAMPLING09`, `PREVIEW313_PROBE09`.
- Added for 352 (:240–251): `CHAPTERS352`, `WORKER352`, `WORKER352_CSS`, `WORKERS352`, `PREVIEW243`, `PREVIEW243_CHAPTERS`, `FRAME_HASH_KEYS`, `FRAME_LIST_KEYS`.
- 53 constants are identical.

## 4. What 350 checks, by row kind

### 4.1 09: kind `boundary313`, unchanged from the 344 integration

- The row names worker `Q/csd-reviewed-boundary-final344.py`, job `csd-reviewed-final-09-boundary344` and a `worker_review` that cites the worker's sha256.
- The module must be version 344, with chapters `('09_boundary',)`, host csd, binary 313's paths and `RECORDS == DISPOSITION344`.
- `sampled_permit_gate()` checks the permit:
  - exactly 344's keys;
  - the six flags false, and no claim at any depth;
  - preview313's 09 movie, sampling and probe bound;
  - intervals made only of sampled frames, with one covering the zoom exit (frame 2094);
  - cc3 340 and root 341, 343 and 345 at their pins;
  - `playback_seconds` equal to root 343's instants.
- 344's own `check()` and `preview_binding()` are re-run.
- The render summary must match the permit's scope, the four records, preview313's inputs and the binding.
- The x264 settings read from the movie must equal the summary's `encoder` and be crf 16.
- `provenance313` and `capture_check09` are unchanged.

### 4.2 The ten 352 chapters: kind `combined`

**The row.**
- The worker is one of `WORKERS352`, and its bytes are the pinned sha256 (:931).
- The host follows from the worker: `D` for `Q/csd-reviewed-final352.py`, `CSS` for `CSS/qualification01/css-final-repair352.py`.
- The job is exactly `csd-reviewed-final-<dashed>352` or `css-reviewed-final-<dashed>352`.
- The row keys are 318's; CSS rows also carry the import keys.

**The module (:947).**
- `VERSION '352'`, `CHAPTERS == CHAPTERS352` (the ten, in order), and `HOST` `csd` or `css`.
- `job_name` gives the row's job: `(chapter, version)` on CSD, `(chapter)` on CSS.
- `MOVIES` and `PREVIEWS` cover exactly the ten.
- A CSS permit must sit in `module.PERMITS` (`repair352-permits`) and pass `permit_name_ok`.
- Kept from 318: `BINARY_SHA` is 268, `sha(module.BIN)` is 268, and the module's root is the host.

**The permit: `combined_scope()` (:666).**
1. **Preview job.** It is 303's for the chapter: `csd-review-repair-preview243` for 06 and 08, `csd-review-repair-preview274` otherwise. It must equal `module.PREVIEWS[c]['job']`.
2. **Movie.** The movie is `D/jobs/<preview>/preview/<c>.mp4`. Its sha256 must equal three things: the preview's own `<c>/sampling.json` `movie_sha256`, `module.MOVIES[c]`, and the scope's `preview_movie_sha256`.
3. **350's own reading of the records.** `frame_basis()` (:590) reads the 352 record format CC2 states (its notes §4; 352's `review_basis` docstring):
   - It reads every file bound in `evidence` and `independent_review` at its stated hash.
   - A frame record has the movie hash under the first of `movie_sha256`, `source_sha256` or `sha256`. Its frames are under the first list of `samples`, `frames` or `pictures`.
   - A `views` item must cite each listed record, either by the record's sha256 or by every sheet sha256 of a `samples` record. Each such sheet is re-hashed.
   - It returns the shape `review_basis()` returns, and the worker's `review_basis(chapter, review)` must be **equal to it, whole**.
   - One rule is stricter than 352. 350 refuses an item whose `views` names its own record (§6, F1).
4. **The tail of 352's `preview_binding`, re-checked:**
   - root evidence views the chapter's own sampling.json;
   - an independent_review file views a record and contains the movie hash;
   - at least one run of two or more viewed frames exists.
5. **The common sampled scope.** `sampled_scope()` is 350's code, shared with 09:
   - exactly 344's keys, with the key-set sha pin;
   - no motion keys, the six flags false, and no claim at any depth in the permit;
   - binary, chapter, host and preview job;
   - pictures reviewed, no defects, the timeline hash and the evidence hashes;
   - the scope's fields, basis and purpose;
   - final-picture, motion and audio review open;
   - intervals sorted, disjoint, inside the chapter, and made only of viewed frames.
   Then the worker's `sampled_review_scope()`, pinned by the sha256 of its source text (`0b9d0ca5…`, the same in 344 and both 352 workers), must return the same intervals.
6. **Playback.** `playback_seconds == []`, because 352 binds no playback record.

**Then (:1032).**
- `source_binding_evidence` is at its hashes, as in 318.
- `module.preview_binding()` is run on CSD for both hosts. CSS's `preview_binding()` does not call `alias_is_exact`; only `receive`, `delivery`, `independent` and `check` do.
- `independent_files()` is 350's own check of 344's and 352's `independent()` rule: the reviewer differs, and the files are directly in Q at their hashes. CSS's `independent()` needs the CSS alias, so 350 does not call it.
- For CSD rows, the worker's own `check()` is re-run. Its `(review, sha, binding)` must equal 350's: the binding was compared as paths, where 318 compared only the first two values.
- **Summary gates.** The render summary must carry:
  - `host`, `preview_job`, `preview_inputs_sha256` (equal to the preview receipt's inputs) and `preview_binary_sha256`;
  - the permit's `review_scope` and `independent_review`;
  - the worker's `review_basis` subset `{movie, frame_records, independent, runs}`;
  - the worker's `preview_binding` at current hashes;
  - for CSS, `worker352_sha256 == sha(worker)` and `narration == module.NARRATION`.
- **Encoder (:1077).** The x264 settings read from the movie equal the summary's `encoder` and include ` rc=crf` and ` crf=16.0`. This now applies to 09 and the 352 rows, but not to the retained rows.
- **Kept from 318:**
  - the receipt, command flags, frozen inputs, outputs at their hashes, and CSS import;
  - `module.verify_tables` and 350's own `schedule()`: timeline269 tables, the 105-WAV narration equality and the score21 stems;
  - the 48 kHz stereo capture of `frames×800` samples;
  - the staging device.

**The row record** gains:
- `review_basis 'sampled_preview'`, `review_scope`, `sampled_intervals`, `sampled_frames`, `open_reviews` and `stated_false`;
- `preview_movie`, `frame_records` and `independent_confirmation`;
- `independent_reviewer`, `preview_job` and `encoder`.

### 4.3 02, 03, 13, 15: kind `retained`

These are unchanged from 318: `RETAINED` pins, `permit_gate()`, `retention()`, the worker's own `check()` and the movie hash. A permit carrying a `review_scope` is refused before `permit_gate()`. The record's `review_basis` is `'historical_permit'`.

### 4.4 Master summary

As in 318, plus:
- `review_bases` per chapter;
- `continuous_motion_acceptance`, `fine_sync_acceptance`, `final_4k_acceptance` and `release_acceptance`, all false;
- a scope sentence.

`claimed()` walks the whole result before the exclusive write and refuses if any key naming an acceptance, a release or a publication is not exactly `false`.

For the ten 352 rows, `claimed()` walks only the permit and the master result, never the bound records. So `Q/grid315-disposition305.json` (`"no_master_or_release_acceptance": true`, CC2 §7 Q8) can be bound without being flagged.

For 09, root 341, 343 and 345 are walked, as before.

## 5. Design choices

- **Pinned, not copied.** 350 does not copy `sampled_review_scope()`: the copy would put the motion-flag name into this file. It pins the function's exact source text and requires the function to be compiled from the worker, with no worker global shadowing a name it uses. It runs that function only after 350's own checks agree.
- **Whole-worker pins for 352.** The 352 bytes are frozen and CC3 reviews 350 together with them. So 350 binds exactly those two files and nothing else, rather than pinning only a function. 344 stays bound through the row's `worker_sha256` and the cited worker review, as before.
- **Two readings that must agree.** For the sampled frames, 350 has its own `frame_basis()`, and the worker's `review_basis()` must equal it whole. A later drift in either refuses, with no silent difference.
- **No 303 or 309 combined rows.** CC2 found no `csd-reviewed-final-*` other than final344 on CSD, and no `css-reviewed-final-*` on CSS. 352 refuses to run beside a 303 job. If root does have a 303 or 309 final, 350 refuses it, and it becomes a root decision.
- **Captions** are unchanged (§7).

## 6. Findings on 352, sent to CC2 (CC2 agreed; 352 stays frozen)

- **F1 (Medium, permit hygiene): a `samples` record can view itself.** A preview sampling.json contains every one of its own sheet sha256s. So an evidence item for the sampling with `views: [its own sha]` passes 352's citation check. 352's tail rule "root evidence views sampling.json" is then met without root's 305/277 pass. Suite case "352 worker alone accepts a self-view" shows the worker accepting it. 350 refuses it (`a record cannot view itself`), so such a permit cannot reach the master. **Root: never write a self-view.**
- **N1 (notes only).** `cc2-final352-notes.md` :259 says CSS `preview_binding()` calls `alias_is_exact`. It does not. CC2 confirmed this by AST and will correct it in its 363 notes. Its freeze time was about 21:31 UTC, not 21:45.

## 7. Captions and 363

- The master runs `BIN captions all --narration rebind269 --out final` (cwd REPO07) after the same hard-link staging, as in 318. 350 adds no caption code.
- Byte equality with native327 is not guaranteed by 350. It depends on 327 having read the same staged tables, and 09's are final344's (timeline313, naming rebind313 narration paths). CC2's 363 should test it as an actual check.
- What I told CC2 for its 363 verifier:
  - the receipt child argv is 11 items starting with `/usr/bin/python3`;
  - each row's `host` is the root path string, not `csd`/`css`;
  - the ten 352 rows carry `review_basis` exactly `'sampled_preview'` and their `sampled_intervals`, equal to their render summaries' `review_scope.sampled_intervals`.

## 8. Focused tests (session scratch, not in the repo)

**Suite.** `mock_suite350c.py` `0769869f162f9889b9e97457ebc108c8bcc62f3fa3a48cdf77713a34634f5960`, derived from `mock_suite350.py` (`4a6f7882…`, kept).
- **75 cases, 75 as expected**, under Python 3.14.0 and again under 3.15.0.
- 350 also compiles in memory under both, and a grep finds no local path, user name or scratch path.

**What is real in the 352 cases.**
- CC2's frozen `csd-reviewed-final352.py` and `css-final-repair352.py`. Their `check()` (CSD), `preview_binding()`, `review_basis()`, `verify_tables()` and `sampled_review_scope()` run for real inside 350's `chapter_gate`.
- The preview movies (06 `5e45d72d…`, 01 `1f6834a6…`), the samplings and their sheets.
- The real Q records:
  - 06: root 277, root 359 manifest and review, the unviewed 358 manifest, cc0 270, binding282;
  - 01: root 305, the bump manifest and its review, the 01 grid sampling, cc3 315.
- The real preview workers, `review-repair-preview243.py` and `cc3-preview-fade274.py`.

**What the mock replaces.**
- The CSD and CSS root literals, in 350 and both workers.
- In the workers, the base pins of mock-built files: binary 268 and 238, both source copies, and the chapter's preview manifest.
- In each sampling.json, the CSD root prefix of its sheet paths. For 06, cc0 270's copy cites that relocated sampling's sha256 in place of the real one.
- `WORKERS352`, set to the mock worker bytes, except in the pin-refusal case.
- Captures are WAV headers declaring `frames×800` samples, and movies are stubs holding an x264 settings string.
- Files of 64 KiB or more are hard links to one copy, to protect a full disk. The one case that edits such a file builds its world with copies.

**Results.**
- **352 good:**
  - 06 on CSD: intervals [[9630,9730],[9970,10029]], 161 sampled frames; the sampling (376 frames) is viewed by root 277 and cc0 270, the 359 manifest (161) by root 359, and 358 is unviewed; independent cc0 270; six flags stated false; crf 16; no claims.
  - 01 on CSS: intervals [[5493,5531],[6273,6310]], 77 frames; the sampling (74), bump manifest (74) and grid (88) are viewed; independent cc3 315.
- **352 refusals (29), each at its intended check.** Every one has its own message except the CSS import-keys case, which is 318's bare row-key assert:
  - permit content: motion boolean; a flag true; a nested claim; self-view; root not viewing the sampling; no 359; an independent file without views; independent reviewer equal to the reviewer; playback instants; an unsampled frame in an interval; another movie; motion review not open; host css on the CSD worker;
  - summary: review_basis, review_scope, preview_binding, preview inputs or encoder differing; the movie at crf 18;
  - worker: not at the pinned bytes; an altered `review_basis` with its pin moved (refused by the 350/worker disagreement); an altered scope function;
  - row and job: job suffix 303; worker 303 on the row; kind boundary313; an existing 303 job;
  - CSS: permit outside `repair352-permits`; the summary naming another worker; no import keys.
- **The finding:** the 352 worker alone accepts the self-view.
- **Interface:** the real frozen 352 bytes equal 350's pins, the chapters, hosts, job names, `PREVIEWS` and `MOVIES` agree, the key-set pin holds, and `pinned_scope` accepts both workers.
- **09 and 318 regressions:** the 42 cases of the earlier suite all pass again after the refactor. They are: 09 good and 25 refusals, `claimed` ×3, the historical `permit_gate` pair, the two retained rows (a review_scope refused by the new message), `provenance313` and `capture09` ×2 each, gates kinds ×3, preflight, and the real 344 interface.

**Not tested here:**
- 350's `film()`, `contract_reads()`, `launch()` and `master()` on real CSD state;
- the eight other 352 chapters with real records (CC2's fixture covered all ten: 93/93);
- anything on a host.

## 9. For root, before any 350 launch

1. Stage the 344 records (cc3 340 and root 341, 343 and 345) and every Q record named in CC2's notes §5 into CSD Q, unchanged, at those hashes. Stage the two 352 workers as CC2 §6 says.
2. Write the 352 permits per CC2 §5:
   - no self-view (F1);
   - `playback_seconds: []`;
   - final-picture, motion and audio left open.
   11, 12 and 14 also need cc1's 357 independent record first.
3. Decide 06 R1/R2 (root 362 exists) and each chapter's host.
4. The 09 row needs its `worker_review` naming CC3's 344 review, which must cite `b8458dbc…`.
5. Then `check --manifest`, read-only, and read every row line before any `launch`.
6. Later steps, all still open: CC3's review353 of this integration, root's launch, the full decode, final pictures, motion, audio, fine sync, captions (caption347 bound separately), final AV, and publication.
