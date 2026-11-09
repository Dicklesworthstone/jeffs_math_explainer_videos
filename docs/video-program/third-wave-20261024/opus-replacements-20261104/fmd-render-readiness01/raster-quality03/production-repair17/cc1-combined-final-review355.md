# cc1 combined-final review 355: frozen 352 (CSD worker, launcher, CSS worker)

Reviewer: cc1 (BronzeJaguar). Written 2026-11-08 (UTC). Author of 352: CC2 (TealKite). I did not write or edit any 352 file.

Scope, per root: an independent, minimal delta check and runtime-record check of the frozen 352 handoff. No new framework, no launch, no renders, no host access, no source, TODO or Beads edits. Every file named here is Q-relative. `D/...` means the CSD project root that the workers name, not a local path.

Disclosure: I also wrote `cc1-chapter-picture-review357.json`, the independent picture record that 11, 12 and 14 need. So on one point this review is not independent: it tests whether 352 accepts my own 357 record. That part is a mechanical check of hashes and frame lists, and its result is given in full below.

## Verdict

- No blocker found. The delta does what `cc2-final352-notes.md` says, and nothing else.
- In the workers' own code, 11, 12 and 14 pass with root's 305 records plus cc1 357. The runs equal CC2's intervals exactly.
- Without 357, all three are refused. That is today's state.
- One Low (L1): a preview sampling.json can count as viewed by itself. Root's 01 permit364 does not trigger it, and an operational rule avoids it. No code change is needed for this launch.
- This review accepts no final picture, motion, audio, fine sync, captions, final AV, master or publication. It does not authorize a launch. Root decides.

## Files, at their sha256

| File | sha256 | Lines |
|---|---|---|
| `csd-reviewed-final352.py` | `23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760` | 364 |
| `csd-launch-final352.py` | `06451123b958db9d4806e2645fd84d8e0b37d604867dae9f25d4cb3c5155859e` | 95 |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | 982 |
| `cc2-final352-notes.md` | `4479363724325af8b0f53417dfe6822df9998634ea968e4213d531d1c6a72fee` | read in full |
| base `csd-reviewed-final303.py` | `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` | |
| base `csd-launch-final303.py` | `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` | |
| base `css-final-repair309.py` | `579fe7cee38af0d9fb249621f53b631645180b5799fc323e6a1ddd157814d18a` | |
| `csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | |
| `cc1-chapter-picture-review357.json` | `47d51709ed54205c053f9789d5aea194e282a27a8d63305b11d995aa00e71490` | |
| `picture-review-progress305.json` (root 305) | `d4547c553a9c6dddabbc5d07448a86f447b0ddc9cac9a70930b2cec63fe9abb2` | |

The three 352 hashes equal the ones root gave. The notes' "21:45" timestamp is a documentation typo, per root. I treat it as that and nothing more.

## 1. Delta read

I produced line diffs (difflib) and a top-level AST comparison with docstrings ignored, then read every hunk.

**CSD worker, 303 → 352** (244 diff lines)
- Added: `MOVIES`, `PERMIT_KEYS`, `sampled_review_scope`, `independent`, `review_basis`.
- Changed: `CHAPTERS` (09 dropped), `EARLIER` (now `244, 278, 287, 303`), `VERSION` (`352`), `check`, `preview_binding`, `run`.
- Removed: nothing at top level.
- The only 303 gate lines removed are these three:
  - `actually_reviewed_motion is True`;
  - `review['motion_intervals']` non-empty;
  - the `motion_intervals` range loop.

  They are replaced by 344's key set and review scope.
- Every other removed line comes back as a superset:
  - `preview_binding`'s return adds the movie and the sheets read;
  - `run()`'s `required` adds `independent(review)`;
  - the render summary adds `independent_review`, `review_scope` and `review_basis`, plus the 344 scope sentence.
- All 303 encoder, resource, source, table, score, binary, narration, preview and delta (binding282) lines are unchanged.

**Launcher, 303 → 352** (52 diff lines)
- The worker path is now 352, the version assert is now `352`, and the chapter choices drop 09.
- One functional line is added: `files.update(worker.independent(review))`, which freezes the independent-review files.
- `--worker-sha256` and `--permit-sha256` are still checked before any worker code runs.

**CSS worker, 309 → 352** (287 diff lines)
- Added: `MOVIES`, `PERMIT_KEYS`, `WORKER309_SHA`, `sampled_review_scope`, `independent`, `review_basis`.
- Changed:
  - `CHAPTERS`, `PERMITS` (now `repair352-permits`) and `VERSION`;
  - `check`, `preview_binding`, `render` and `required`;
  - `permit_sources`, which also carries the `independent_review` files.
- Removed: nothing at top level.
- The same three motion lines are the only gates removed.
- `TRANSFER` and `DELIVERY` stay 309's, as the notes say.
- CSS `independent()` is 344's, except that it calls `alias_is_exact()` and requires `resolve == FROZEN/<path>`.

**Segments and constants**, from the files' own AST and `ast.get_source_segment`:
- `sampled_review_scope` is `0b9d0ca5f8a3060f041f32898b1b966326db0bf225c97492527ab907194a2672` in 344, CSD352 and CSS352.
- `review_basis` is `a6c923203190492e728163828f6cd449e151933c2ec6758d9fa77c9af0ad8944` in CSD352 and CSS352. It does not exist in 344.
- CSD352 `independent()` equals 344's apart from its docstring.
- `PERMIT_KEYS` is equal to 344's in both workers.
- `MOVIES`, `CHAPTERS` (ten, no 09) and `VERSION` are equal across the two workers.
- The 352 tail of `preview_binding` is the same 9 lines in each worker, with only formatting different.
- Both workers keep the preview274 job name.
- For 11, 12 and 14, `MOVIES` equals the preview sampling's `movie_sha256`, which equals the local preview movie's sha256. Each sampling's `frames` equals CC2's frame count (11932, 3943, 11756).

## 2. Runtime-record check, 11 / 12 / 14

**What ran**
- The workers' own source text, exec'd unchanged from the frozen files:
  - `sampled_review_scope`, `review_basis` and `independent` from CSD352;
  - `review_basis` and `sampled_review_scope` from CSS352;
  - in each worker, the 352 tail of `preview_binding`: the lines from `# 352: the sampled-preview scope` up to the `return`, wrapped as a function of `(chapter, review, job)`;
  - `check()`'s key-set test.
- The real Q records at their hashes.
- Permits were in-memory dicts built per CC2 §5. They were never written to disk, and none is a permit.

**Harness substitutions** (the only ones):
- D is a session folder whose `jobs/csd-review-repair-preview274` points at `Q/preview274-review305`.
- `sha()` maps the D prefix of the sheet paths inside each sampling.json to that folder.
- Q is the local Q.

**Not run:**
- the 303/309 gates that need CSD or CSS files (receipts, tables, binary, score, narration);
- CSS `alias_is_exact()` and CSS `independent()`;
- the launcher;
- anything on a host;
- any render.

Python compiled the exec'd segments in memory; no `.pyc` was written.

**Result: 93 of 93 checks pass, 0 fail.** The 93 include the 41 refusal cases listed below. The script's results file is `cd3873f15636e51be4365f31555d53ea17f3697f179726c1c7c51af8c95260bd` and the script is `6274076b54771ea46553251ae1f77b97a049d7299f9346a7d3f438c29e1df673`, both in session scratch.

| Chapter | Bound records, each `viewed_by` root's review and cc1 357 | `runs` (CSD = CSS) | Equals CC2 | `independent` | `unviewed` |
|---|---|---|---|---|---|
| 11_counting | preview274 sampling (root 305); counting-transition manifest (its 305 review); counting-opening manifest (its 305 review) | `[[34,87],[7464,7494],[8294,8344]]` | yes | cc1 357 | none |
| 12_residues | preview274 sampling (root 305); residue-transition manifest (its 305 review) | `[[2471,2516],[2532,2577],[2763,2806],[3025,3154],[3911,3942]]` | yes | cc1 357 | none |
| 14_continuation | preview274 sampling (root 305); continuation-transition manifest (its 305 review) | `[[34,98],[1223,1258],[8876,8940],[10179,10301],[11724,11755]]` | yes | cc1 357 | none |

- **CSD equals CSS.** Both workers' `review_basis` and tail give identical results for every row.
- **357 adds no frames.** Without 357's `views`, the runs are the same and `independent` is empty. 357 confirms the chapter; root's 305 records supply the frames.
- **Optional 12 grid.** If root also binds `coverage-grid305/12_residues/sampling.json` (`bd9bd996…`) and lists it in 357's views, one more run appears: `[2399, 2400]`. Frame 2399 comes from the preview274 sampling and 2400 from the grid. 357 cites both records and I viewed both pictures. CC2's five intervals remain valid as sub-intervals. Binding the grid is optional.
- **Unviewed grids are harmless.** For 11, 12 and 14, a per-chapter grid sampling bound as evidence without any `views` is listed as unviewed. The row still passes and the runs do not change.

**Refusals (41 cases).** All are refused, each with the expected message.
- **No independent_review**, which is the state before 357. CSD refuses in `independent()`; CSS refuses in the tail ("no independent_review file views a frame record…").
- **357 bound with `views: []`.**
- **357 views `163f404d…`**, the top-level `coverage-grid305b/sampling.json`. It is a JSON list, not a frame record, so it is refused as "views names no bound frame record", bound or not.
- **357 views a per-chapter grid that 357 does not cite:** `4c3973dd…` for 11, `18ac2871…` for 14. Refused with "does not cite the record it views".
- **357 views another chapter's sampling.**
- **First interval one frame past its run.** "Unsampled frames".
- **Last interval one frame past its run.** Refused: 11 and 12 as unsampled, 14 as outside the chapter.
- **Root 305 views nothing.** "The permit must bind … sampling.json and root evidence that views it".
- **357 at another hash.**
- **`playback_seconds: [1.0]`.**
- **`final_4k_acceptance: true`.**
- **`motion` removed from `open_reviews`.**

## 3. Findings

**L1 (Low). A preview sampling.json can count as viewed by itself.**
- **Mechanism.** In `review_basis`, a `samples` record is cited when the viewing file contains every `sheets[].sha256` of that record. A sampling.json contains its own sheet list, so it always cites itself.
- **Effect.** If the permit's sampling item carries `"views": [<its own sha256>]`, two things follow. The sampling is credited as "viewed by" an evidence file. And the tail's first check ("root evidence that views it") passes even when root 305 views nothing.
- **Test.** I ran this in both workers for 11, 12 and 14, with 357 viewing only the manifests and root 305 viewing nothing. It was accepted, with the sampling's `viewed_by` equal to the sampling itself.
- **Why Low.**
  - Only root writes `views`, so reaching this takes a root mistake, not an outside input.
  - Every other gate still holds.
  - Root's `01_the_bump-css-permit364.json` (`299b8fe0…`) has no `views` on any frame record. It follows the rule below.
- **Recommendation.** No code change for this launch. `review_basis` is pinned by AST in both workers and in master350, so a fix would mean re-pinning all three. Root should keep one rule in every permit: put `views` only on viewing files (root 305, the transition reviews, independent files), never on a sampling, manifest or grid item. If a later version touches `review_basis`, it could refuse an item that views its own path (`x['path'] == named[digest]`).

**I1 (Info). Independent confirmation is per chapter, not per interval.**
- An independent file that views any one frame record of the chapter, and names the movie, confirms the chapter. That includes windows only root viewed.
- This is the stated design (notes §4).
- For 11, 12 and 14 it makes no difference: 357 views the sampling and every window manifest.
- Nobody should read `review_basis.independent` as a statement about each sampled interval.

**I2 (Info). 357 names `163f404d…`, which is not a view.**
- My 357 JSON lists the top-level grid305b sampling under `viewed_picture_bindings` for 11 and 14. That file is a list, so `views` of it are refused (section 2).
- 357 also does not contain the per-chapter grid hashes for 11 and 14, so those cannot be credited to 357 either.
- Neither changes any run. Use the views table below.

**I3 (Info). On CSS, the movie and sheets come from the 309 runtime transfer.**
- `review_basis` reads the preview movie and, for root 305's sheet-hash citation, every preview274 sheet of the chapter, through the alias.
- In code, `runtime_sources()` carries the whole `D/jobs/csd-review-repair-preview274` folder (`JOB_DIRS`).
- CC2 spot-checked three files on CSS. I did not check CSS.
- If 11, 12 or 14 goes to CSS, a missing or changed movie or sheet makes the worker fail closed.

**I4 (Info). master350 is outside this review.**
- `cc0-combined-master350.py` is now `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6`.
- No 352 file refers to 350. Its fit to 352 is CC0's open item (notes §9).

**I5 (Info, hygiene, root's existing files).** The `sheets` lists in `coverage-grid305b/*/sampling.json` and `coverage-grid305/12_residues/sampling.json`, and in the counting-transition and residue manifests, hold local absolute paths. This has no effect on 352, because `review_basis` reads `sheets` only for `samples` records. For the public repo, root may want those records with Q-relative paths before they are published.

## 4. `views` for root's 11 / 12 / 14 permits (cc1 357)

Bind `Q/cc1-chapter-picture-review357.json`, sha256 `47d51709ed54205c053f9789d5aea194e282a27a8d63305b11d995aa00e71490`, under `independent_review`, with `independent_reviewer` `cc1`. Give it these `views`. Every hash listed is in 357's bytes, and each was tested above.

| Chapter | `views` on the cc1 357 item |
|---|---|
| 11_counting | `eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846` (sampling), `9e74486edaaac78d1a4a868f83507a69378f1be9d64ab0456ddd453a34fc64d6` (counting-transition), `9b889e043ce97bfbb2ce8b72f8306c3dbd3718a29c52b97f4adefa2b702bea8a` (counting-opening) |
| 12_residues | `df4ac4872fe729b2cb11b91b44041f143f9cfca707a4872c3056503cda59272e` (sampling), `c6cca51f4290b32b7e4f2587900cdf1adefde66bf48d032b32f322edd684eea2` (residue-transition); optionally `bd9bd9969c332a18b141947030a37aba1b472191960f0b9844d5f6825ece6c00` (12 grid), only if root binds that grid |
| 14_continuation | `5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b` (sampling), `a787e39d74c54651b8bbb3a97f0c75e7c863be4ab946c3004bb5aa153e34649b` (continuation-transition) |

- **Never list:**
  - `163f404d…` (refused);
  - `4c3973dd…` or `18ac2871…` (refused: not in 357's bytes).
- **Optional grids.** The 11 and 14 per-chapter grids may be bound without `views`. They are then reported as unviewed, which is harmless.
- **Evidence.** Per CC2 §5: the sampling (no `views`); root 305 → sampling; each transition manifest (no `views`); its 305 review → that manifest.
- **`sampled_intervals`.** CC2's intervals, or sorted, disjoint sub-intervals of them.

## 5. Limits

- This is a code and record check. I did not compile or run the full workers, the launcher or anything on CSD or CSS. The 303/309 gates outside the 352 tail were read in the diffs, not executed.
- The runtime check used local copies of the records and preview files. CC2's notes say these equal the CSD copies; I did not check the hosts.
- 357 is a still-picture review of the preview274 movies. Nothing here concerns motion, listening, fine sync, captions, 4K output, the master or publication. All of those remain open.
