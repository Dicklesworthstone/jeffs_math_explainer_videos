# 363: final verification and final pictures of master350 (CC2)

cc2, 2026-11-08, frozen at about 22:02 UTC. Q is this folder locally, or `D/qualification01/production-repair17` on CSD, where D is the CSD job root. R is the CSS render-worker root (the `CSS` constant).

Nothing here has run on any host. No launch, permit, render, build, synthesis or commit. The verifier is prepared only; root runs it after 355 and its own staging.

## Files

| File | sha256 | State |
|---|---|---|
| `csd-final-verify363.py` (this task) | `b4f687a58bc7233341e2b19fcfcf9b886faf31217d255d68073e62c9ecf09c1a` | frozen; 1135 lines; compiles |
| `csd-final-verify339.py` (base, CC0-reviewed as 339) | `1e200220a1a7fd794abe318c0a1d815c188976f57c0707902c71915345dc05af` | preserved unchanged |
| `cc2-final-verify339-notes.md` | `60d97ad6cf3d6aaa5319d202fd83a13f2ce568e57dd366fb163e3ee32fa35e4f` | preserved unchanged |
| `cc0-final-verification-review339.md` | `5e326473b2b95d4a23125a7be37ce1c7361a234f3a5e2bcf5801bd1e4ac0b10e` | read (L1, N1, §4) |
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | CC0's, frozen; pinned in 363 |
| `cc0-master350-notes.md` | `e30db95759918745131e1b53a31e6bc033d20a1125ea835707ff095d6123b151` | CC0's, frozen; read |
| `csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | frozen; pinned |
| `csd-reviewed-final352.py` (CSD) | `23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760` | frozen; pinned |
| `R/qualification01/css-final-repair352.py` (CSS) | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | frozen; pinned |
| `cc2-final352-notes.md` | `4479363724325af8b0f53417dfe6822df9998634ea968e4213d531d1c6a72fee` | frozen; errata in §9 below, bytes unchanged |
| `root-preview06-disposition362.json` | `1822a5908199c1cf8ae935e50c803e6a419c0109487c4051bef31d0ef0bf0b86` | root's; pinned |
| `cc1-chapter-picture-review357.json` | `47d51709ed54205c053f9789d5aea194e282a27a8d63305b11d995aa00e71490` | CC1's; pinned |
| `root-caption347-review354.json` | `749aeba8350d46e339a517af971086bd2d6e2fe039e6501f16e480edd3ad3cb4` | root's; pinned |
| `cc1-caption-repair347-candidate.en.srt` | `0ea9277e6d598540e5e9057a84ee84daf7f88420b076434659fc3d3ad0ff3230` | CC1's; pinned |
| `cc1-caption-repair347-candidate.en.vtt` | `6a383cb7f207be81254e9a820ecb701ca77d00a26557194c675dc1717ef6979b` | CC1's; pinned |
| native327 `captions/complex_analysis_visual.en.srt` (CSD job `csd-caption-preflight327`) | `5d06423248f088a34774f361721bf05e1c4950ac834086e3666661e1adfe5e1b` | on CSD; pinned |
| native327 `captions/complex_analysis_visual.en.vtt` | `26e0d7e0eec26eb6f9136bb56f795a6ff8ab9cead26f29fc76a75e9abc4bb2e3` | on CSD; pinned |

There is no separate launcher. As in 339, the verifier launches its own guarded jobs (`launch-verify`, `launch-pictures`).

## 1. What changed from 339

The whole delta, by AST over the top level (functions, assignments, docstring):

- **Removed:** `master318`, `FINAL317`.
- **Added:** `master350`, `delivery347`; constants `MASTER_HELPER_SHA`, `FINAL344`, `CSS`, `WORKERS`, `REVIEW_BASES`, `FINAL_FRAME_SOURCES`, `FINAL_FRAME_CHECKS`, `PREVIEW243_MOVIE06`, `NATIVE327`, `NATIVE327_CAPTIONS`, `DELIVERY347`, `ADOPTION354`.
- **Changed, logic:** `film`, `picture_plan`, `plan_for`, `verify`, `pictures`.
- **Changed, names and messages only:** `main`, `schedules`, `preview313`; `VERSION` (`'363'`), `MASTER`, `MASTER_HELPER`, `MASTER_INPUTS`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES`, `PICTURES_INPUTS` (350's and 363's job and input names); the docstring.

Every other top-level function and constant is identical in AST to 339's, among them `stable`, `success`, `shared`, `build313`, `anchors`, `frame_hashes`, `runtime`, `pillow_runtime`, `growth_upper`, `admitted`, `require`, `run`, `verified`, `part_paths`, `plan_digest`, `part_upper`, `part_done` (the 332 M1 fix), `strip`, `launch` and `PINS`. So every 339 gate holds unchanged: the shared verifier and strict decode script at their pins, the caption screen, L1 framemd5, L2 free space, L4 runtime, M1 windows, `check_captions`, loudness, exact picture and sheet sets.

What each change is for:

1. **`master350` (:358).** 318's checks, plus:
   - the helper must be the frozen 350 (`60575944…`) and the one its job bound (:367);
   - every row's `review_basis` equals 350's for its kind: `sampled_preview` for 09 and the ten 352 rows, `historical_permit` for 02/03/13/15 (:383–390);
   - every non-retained row names the frozen worker, its sha256 and its job: `csd-reviewed-final-09-boundary344` for 09; `csd-reviewed-final-<chapter>352` on D or `css-reviewed-final-<chapter>352` on R for the ten; and the master's bound inputs hold that worker at that hash;
   - `summary.review_bases` equals the rows (:391);
   - for each sampled_preview row, its own render summary carries the permit hash and `review_scope` the row records, with basis `sampled_preview`, purpose `technical_4k_render`, non-empty integer intervals inside the chapter, and no playback instant for a 352 row (09 may have its own) (:425–435). These intervals are returned to the picture plan.
2. **`delivery347` (:492) and `film` (:532).** See §4.
3. **`picture_plan` (:605–620) and `plan_for` (:921).** See §3.
4. **`verify`.** Runs the same pinned screen on the 347 SRT (:871) and records `review_bases` and `delivery_captions_347` (:895). `caption-screen347.json` joins the must-not-exist list. Scope text: 350 master; caption playback and 4K caption legibility stay open.
5. **`pictures`.** `sampling.json` also records `final_frame_sources` and `final_frame_checks` (:1056). The overview comment now says what review 339 L1/N1 found (:1060).

## 2. What 363 reads from master350

- **Receipt.** `stable(MASTER, MASTER_INPUTS)` as in 339: a successful receipt, and every bound input re-hashed. The command must be 350's 11 items: `/usr/bin/python3 -I -B <350 resolved> master --job csd-combined-master350 --inputs Q/combined-master350-inputs.json --manifest M`.
- **`master-summary.json`.** `version` `'350'`, `host` `'csd'`, `frames`, `chapters`, `consumer_binary_sha256`, `narration`, `native_bit_identical_replays`, `replays`, `manifest`, `rows`, `review_bases`, `chapter_binaries`, `boundary_replay` (318's keys plus `review_bases`).
- **Each row.** 318's `chapter`, `kind`, `binary_sha256`, `host`, `job`, `narration`, `movie_sha256`, `frames`, `start_frame`; and 350's `review_basis`, `worker`, `worker_sha256`, `permit_sha256`, `review_scope`, `sampled_intervals`. Not read: `permit` (the path), `open_reviews`, `stated_false`, `historical_permit`.
- **`staged-inputs.json`** and the `final/` hard links: unchanged from 339.
- **Each chapter's `render-summary.json`** (344 and 352 jobs, and the retained ones): 339's `chapter`, `frames`, `binary_sha256`, `files`; and for sampled_preview rows `permit_sha256` and `review_scope` (`basis`, `purpose`, `sampled_intervals`, `playback_seconds`).

CC0 confirmed this interface for 350 before freezing it.

## 3. Picture anchors

All of 339's anchors stay: plan182's samples, previous layouts, the M1 transition windows (root 359's consecutive frames among them), the gallery sites and the 07 playback instants.

363 adds two things. `add()` merges reasons, so a frame that is already planned keeps its picture and its first reason, and gains the new one.

- **Sampled intervals (M1 on the new basis).** Every frame of every `review_scope.sampled_intervals` pair of the eleven sampled_preview rows is an overview sample. Each interval also holds a native 4K frame; if none is planned yet, its middle frame is added. 06's intervals are root's 359 windows. 08 includes the preview243 run 10200–10380. The 09 intervals are whatever its 344 render summary on CSD records.
- **Frames root requires in the native final** (`FINAL_FRAME_CHECKS`; each is an overview sample and a native 4K frame):
  - 06: 600, 900, 1200, 1290, 1320, 1680, 1890. Source: root 362 (review 270 R1/R2). `plan_for` checks 362's `required_final_checks.frames` equal these, on preview243's 06 movie `5e45d72d…`.
  - 11: 4348, 4408, 7448, 8121, 8303, 9731, 11880 (after 357 O1–O3, O5).
  - 12: 2532, 2766, 3002 (after 357 O7/O8: bottom residue label against player captions).
  - 14: 10260, 10278, 10301 (after 357 O11: faint Gamma-outline tint).
  - 11 8303 sits inside root's window 8294–8344, so it is one sample with both reasons. That is the reuse root asked for.
- Both records must be at their pins with every `*_acceptance` False, and 357's `chapters` must be {11, 12, 14}.

No new framework and no preview rerender: the frames go through 339's plan, jobs and sheets.

## 4. Captions: the native proof and the 347 delivery check, kept apart

- **The native proof is 339's, unchanged.** It runs the pinned `check_captions(srt_text, vtt_text, script, report, placements, total_frames)` on the master's own SRT/VTT, plus the pinned screen on the master's SRT. The inputs are:
  - `script`: the shared `read_script` of the build268 workspace's `script.tsv` (equal to build313's);
  - `report`: rebind269's `narration/report.json`;
  - `placements`: from `schedules()`, the master's own timelines;
  - `total`: 133,662 frames.
- **The delivery check (`delivery347`) is separate.**
  1. native327's receipt succeeded and its SRT/VTT are at their pins;
  2. the master's native SRT and VTT are byte-identical to native327's. 347 was cut from native327 and root 354 adopted it against native327, so this is what ties 347 to this master;
  3. the 347 SRT/VTT are real files (no links) at their pins;
  4. root 354 is at its pin, cites both 347 hashes and native327's SRT hash, decides `Adopt347 …`, and every `*_acceptance` is False;
  5. the 347 SRT/VTT pass the same pinned `check_captions` with the same inputs.
  The verify job then runs the same pinned screen on the 347 SRT. It refuses unless the count matches and nothing is over 20 characters per second or 42 per line (root 354's adoption terms). The result goes under `delivery_captions_347`, apart from `captions`. `caption_playback_reviewed` is False.
- **Real-data control** (§7): with the fetched native327 inputs, native327 gives 571 blocks and 347 gives 579 blocks. Both have SRT = VTT, text equal to the script whole and cue by cue, and last end 2,220,147 ms of 2,227,700. The 347 screen gives 579 captions, 0 over 20 characters per second, 0 lines over 42, fastest 19.231.

## 5. Run order and the external overview ledger (review 339 §4, with L1 and N1)

1. `check` (read-only).
2. `launch-verify`, then the verify job `D/jobs/csd-final-verify363`.
3. `launch-pictures` starts `csd-final-pictures363-overview`. **At overview success, before anything else touches that folder:**
   - copy the `sampling.json sha256` from the job's last line into root's own ledger, outside the job folder;
   - hash `sampling.json` read-only and confirm it equals the line.
   The line itself is written into the job's folder (L1), so it is not an anchor by itself.
4. **Immediately before the second launch,** hash the overview `sampling.json` read-only again and compare it with the ledger. Go on only on a match.
5. The second `launch-pictures` starts `csd-final-pictures363-native`. **Compare the ledger with all three:**
   - the printed `overview_sampling_sha256`;
   - the native inputs row for the overview `sampling.json` (in `Q/final-pictures363-native-inputs.json`);
   - `union.overview_sampling_sha256` in the native `sampling.json` (:1012).
6. **Before viewing,** re-hash read-only every file recorded in both `sampling.json` files. Then root views every sheet and every native frame.

## 6. Root's staging on CSD before `check`

- **Copy to CSD Q, at the pins above, as real files (no links):**
  - `csd-final-verify363.py`;
  - `root-preview06-disposition362.json`;
  - `cc1-chapter-picture-review357.json`;
  - `root-caption347-review354.json`;
  - `cc1-caption-repair347-candidate.en.srt` and `.vtt`.
- **Already required by 350, read only through its bound inputs:** the 344 and 352 workers in CSD Q, and the CSS worker at its R path.
- **Already on CSD:** native327 (`D/jobs/csd-caption-preflight327`, receipt and captions).
- **The master job itself:** `csd-combined-master350` must have finished; `check` refuses until its receipt is final and successful.

## 7. Tests (scratch, outside the repo; local only)

| File | sha256 | Result |
|---|---|---|
| `fixture_test363c.py` | `72dcf69eea59953d20962bc33e0e316f31161be5a30b8fdfdbef55cdee55a62e` | **154/154** expectations, 197 s; output `95daf5fb…` |
| made by `make_fixture363.py` from 339's `fixture_test339c.py` (`cb65a343…`) | `b09d50e4…` | adds block I/K from `block_i363.py` (`1f9d23a5…`) |
| `real347_control.py` | `5bd4968a739444bc738a2fbda7cb68abdbb3dfb35dc1a1f8aba01c6d1ee8afe8` | **PASS**; output `aafb0a3f…` |

**Fixture.**
- It carries 339's whole suite under 363's names: static gates, launches, never-duplicate, guard argv, C1–C7 picture/sheet forgeries.
- It adds 39 lines of 363 expectations (block I, block K):
  - interval coverage: 2,404 interval frames, every one a sample, a native in each interval;
  - the 20 required frames are all natives and samples, with 11 8303 reused;
  - the 362/357 records are bound;
  - 21 refusals:
    - worker 303, job 303, 09 final317, a CSS row naming the CSD worker;
    - wrong review basis both ways, and a `review_bases` map that differs;
    - intervals that differ, a playback instant in a row or in a render summary, a permit hash that differs;
    - a changed bound worker, a master helper off 60575944;
    - changed 362 or 357;
    - native327 changed, the master's SRT not native327's, a 347 text unlike the script;
    - 354 that does not cite native327, or claims an acceptance;
    - 347 off its pin;
  - a passing verify that records `delivery_captions_347`;
  - a 300 ms 347 stand-in that passes `check` and is refused inside `verify` at the 347 screen, with no `verification.json`.
- **Real in the fixture:**
  - the 363 text (only the D and R roots swapped);
  - the shared verifier, the screen and PIL;
  - the real 350, 344, 352 and CSS worker bytes;
  - the real 362, 357, plan182 and M1 records;
  - the real frame counts and the ten 352 rows' real sampled intervals.
- **Faked in the fixture:**
  - ffmpeg runs, the strict script, the guard, CPU affinity, free space, ldd;
  - the master tree, layouts, timelines, narration and script text;
  - native327, 347 and 354 (stand-ins);
  - the 09 intervals (two stand-ins).
- **Counts are the fixture's, not the real plan's:** 8,659 samples, 118 natives, 1,083 sheets; part budgets 1.63 and 2.80 GiB of 3.5. The real counts come from `check` on CSD.

**Real control.** It calls 363's real `delivery347()`, the pinned `check_captions` and the pinned screen on:
- the real native327 SRT/VTT, its 15 timeline269 tables, rebind269 `report.json` and receipt, each checked against the 347 tool's pins;
- the real 347 candidate and root 354;
- Q's `script.tsv` (`234f7257…`).
The only stand-in is that the master's native captions are taken to be native327, because that equality is the check 363 makes on CSD. Q was not written (7,280 status lines before and after).

## 8. Not claimed

- No picture, motion, listening, fine-sync, caption-playback, 4K caption-legibility or independent AV acceptance. Every such flag the jobs write is False.
- Nothing ran on CSD or CSS, and the real picture counts are unmeasured.
- Not publication authorization.

## 9. Errata to `cc2-final352-notes.md` (frozen bytes `4479363…` left unchanged)

- **E1. Freeze time.** Line 3 says "frozen at about 21:45 UTC". The clock at freeze was about 21:31 UTC; 21:45 is a typo.
- **E2. :259 names the wrong function.** It says the CSS worker's `check()`, `independent()` and `preview_binding()` call `alias_is_exact`. `preview_binding` does not; the callers are `receive`, `delivery`, `independent` and `check` (CC0; confirmed by AST). The answer "yes, safe on CSD for a CSS row" rests on that line's first two sentences: `review_basis` reads only D paths and the review dict, and the module only defines constants at import.
- **E3. Self-view (CC0 F1; CC1 355 L1).** The 352 workers' `review_basis` credits a `samples` record whose `views` names its own sha256, because a sampling.json holds its own sheet hashes. 350 refuses an item whose views name its own record. Rule for root's permits: put `views` only on viewing files (root 305, the transition reviews, independent files), never on a sampling, manifest or grid item. Root's `01_the_bump-css-permit364.json` has no `views` on any frame record.
- **For the 11/12/14 permits,** CC1 355 lists the `views` to declare on the 357 item, with full hashes in its table:
  - 11: `eb67bf33…`, `9e74486e…`, `9b889e04…`;
  - 12: `df4ac487…`, `c6cca51f…`, plus `bd9bd996…` only if that grid is bound;
  - 14: `5faa7299…`, `a787e39d…`.
  Never `163f404d…`, `4c3973dd…` or `18ac2871…`.

## 10. Hygiene

- `python3 -I -B`, in-memory `compile()` only.
- `find` was not used.
- No repo file other than these two was written.
- No `Q/__pycache__` change.
- No local user, temp or scratch paths in either file.
