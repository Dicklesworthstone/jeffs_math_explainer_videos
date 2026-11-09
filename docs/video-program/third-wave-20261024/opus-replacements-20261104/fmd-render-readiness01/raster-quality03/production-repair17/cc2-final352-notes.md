# cc2 work352: sampled-preview successors to 303 and 309 for the ten changed chapters other than 09

cc2, 2026-11-08, frozen at about 21:45 UTC. Q is this folder locally, or `D/qualification01/production-repair17` on CSD, where D is the CSD job root. R is the CSS render-worker root (309's `CSS`), and CQ is `R/qualification01`. Paths below are written with these prefixes. In a permit, every path is the absolute host path.

**What this is not.** I launched, rendered, synthesized and committed nothing. I wrote nothing on any host, and changed no source, permit, TODO, Beads or memory file. The only host commands I ran were read-only: `ls` and `sha256sum` on CSD and CSS. Nothing here accepts any picture, motion, listening, fine-sync, caption, final-AV, 4K, release or publication review, and it does not authorize a launch.

## 1. Answer

The 352 workers take 344's sampled-preview permit, for a technical 4K60 render only. They use 344's exact key set and its byte-identical `sampled_review_scope()`. They replace 303/309's `actually_reviewed_motion` and `motion_intervals`, and every other 303/309 gate is kept (§3).

Which chapters the real records support (fixture test, §8):

| Chapter | State | Basis |
|---|---|---|
| 01, 04, 07, 10 | **Pass** | root's full sampled pass + root framewise transition windows; independent: cc3 315 (grid stills) |
| 05 | **Pass** | root sampled pass; independent framewise windows (cc1 319, 715 frames) + cc3 315 |
| 06 | **Pass** | root sampled pass + root's new 359 framewise record (161 frames: 9630–9730, 9970–10029); independent: cc0 270 |
| 08 | **Pass** | root sampled pass, which includes 10200–10380 consecutively; independent: cc2 271 |
| 11, 12, 14 | **Refused until an independent picture record is bound** | the root records are in place; cc1's review357 is in progress |

Records are not pinned in code. The permit binds them, and once cc1's 357 record exists, root adds it to the 11/12/14 permits with no new worker. For 06, root's new 359 record is used as it stands. playback_seconds is `[]` on every row.

## 2. Files

| File (Q) | sha256 | Lines | Base, preserved unchanged |
|---|---|---|---|
| `csd-reviewed-final352.py` | `23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760` | 364 | `csd-reviewed-final303.py` `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` |
| `csd-launch-final352.py` | `06451123b958db9d4806e2645fd84d8e0b37d604867dae9f25d4cb3c5155859e` | 95 | `csd-launch-final303.py` `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | 982 | `css-final-repair309.py` `579fe7cee38af0d9fb249621f53b631645180b5799fc323e6a1ddd157814d18a` |
| `cc2-final352-notes.md` | this file | | |

344's worker (`b8458dbc…`) and launcher (`70278078…`) are also unchanged. The diffs are kept in my session scratch, not in the repo:
- 303→352 worker: `6d4c8d12d1a4e215f58857f951cec636d03d8e690d83a2c5231c87bbd8586454`
- launcher: `465a6ea67e413c6f2d327ff185ef0f9de7f78decc0d145aa08c0c4a0708da1aa`
- 309→352 CSS worker: `a2d2d5b57d4807c9a82aaf1cd7274a09be1cd441a58bdf7cbf6001cca5036968`

**Shared source.** Shared function segments are compared by sha256 of `ast.get_source_segment`:
- `sampled_review_scope()` is `0b9d0ca5f8a3060f041f32898b1b966326db0bf225c97492527ab907194a2672` in 344, in 352 CSD (:113–157) and in 352 CSS (:555–599). That is master350's pin.
- `review_basis()` is `a6c923203190492e728163828f6cd449e151933c2ec6758d9fa77c9af0ad8944` in both 352 workers (CSD :165–223, CSS :615–673).
- `MOVIES`, `PERMIT_KEYS`, `CHAPTERS` and `PREVIEWS` are equal in the two workers.

## 3. What changed, by AST (docstrings set aside)

**CSD worker, 303→352.**
- Added: `sampled_review_scope` (344's, byte for byte), `independent` (344's, byte for byte apart from the docstring) and `review_basis`.
- Changed:
  - `preview_binding`: a scope tail at :251–259, and the return adds the preview movie and the sheets the basis read.
  - `check`: :279 requires exactly `PERMIT_KEYS`; `actually_reviewed_motion` and `motion_intervals` are no longer read; :289 calls `independent(review)`.
  - `run`: freezes the independent files and adds the summary keys in §7, Q5.
- Assignments:
  - `CHAPTERS` has the ten chapters, without 09.
  - `EARLIER=('244','278','287','303')` and `VERSION='352'`.
  - New `MOVIES` (:41) and `PERMIT_KEYS` (:97, 344's).
- Removed: nothing.

**Launcher, 303→352.** `main` only:
- the ten chapters;
- worker `csd-reviewed-final352.py` (:39);
- `VERSION=='352'` (:47);
- `files.update(worker.independent(review))` (:74), as 344's launcher does.

The inputs file is `final-<chapter>-inputs352.json`. The guard command, the 16-CPU gate, the job-exists refusal and the preview job folder freeze are 303's.

**CSS worker, 309→352.**
- Added: the same three functions. `independent` is 344's, with the files read through the exact alias (`resolve == FROZEN/<path>`).
- Changed:
  - `permit_sources` (:279): the permit transfer also carries `independent_review` files.
  - `preview_binding`: the same tail, at :711.
  - `check`: :749 requires `PERMIT_KEYS`; the motion checks are gone; :761 calls `independent()`.
  - `required`: adds the independent files.
  - `render`: summary keys.
- Assignments:
  - `CHAPTERS` has the ten chapters, without 09.
  - `VERSION='352'` gives job `css-reviewed-final-<chapter>352`, permit-transfer prefix `css-repair-permit352-<chapter>` and stream header `css-final-repair352`.
  - `PERMITS=CQ/repair352-permits`, a new folder; `receive` creates it.
  - `MOVIES`, `PERMIT_KEYS` and `WORKER309_SHA`.
- Removed: nothing.

`TRANSFER`, `DELIVERY` and `DELIVERY_INPUTS` stay 309's: `css-repair-transfer309` and `css-repair-delivery309`, both completed with exit 0. 352 reuses them. 309's runtime-manifest, `launch-delivery` and `delivery` modes refuse under 352, because those jobs exist. No mode was added.

**Kept, checked by AST:** every 303/309 encoder, resource/guard, exclusive-lane, source, table, score, binary, narration, carried-row, preview-binding, delta (binding282 for 06/08), host and EARLIER gate.

## 4. The rules

`review_basis(chapter, review)` reads only:
- the pinned movie `D/jobs/<preview job>/preview/<chapter>.mp4`, whose sha256 must be `MOVIES[chapter]`;
- the files bound in `evidence` and `independent_review`, each at its stated sha256.

**Frame record.** A bound JSON object whose movie hash equals `MOVIES[chapter]`. The movie hash is read from the first of these keys present: `movie_sha256`, `source_sha256`, `sha256`. The frames are read from the first of these lists present:
- `samples` as `[{"frame": n}]` (a preview sampling.json);
- `frames` as `[n]` or `[{"frame": n}]` (305 transition, gallery and grid manifests);
- `pictures` as `[{"frame": n}]` (root's 358 and 359 manifests).

**Viewed, explicitly.** A permit item, in evidence or independent_review, may carry `"views": [<sha256 of frame records>]`. That is root's statement that this file records viewing those pictures.
- Each listed hash must be a bound frame record of this chapter.
- The viewing file must cite it: either its bytes contain that sha256, or, for a `samples` record, they contain the sha256 of every item of its `sheets`. In the second case each sheet is re-hashed and frozen.
- Mentioning a hash views nothing. For example, cc1 319 lists 05's sampling under `context_read`; 05's sampled frames are credited to root 305 only.
- A frame record no item views adds no frames and is returned in `unviewed`.

**Returned:**
```
{chapter, preview_job, movie: {path, sha256},
 frame_records: [{path, sha256, frames, viewed_by: [paths]}], unviewed: [paths],
 independent: [paths], sampled: [frames], runs: [[a, b], ...], sheets: [paths]}
```
`runs` are the runs of two or more consecutive frames in `sampled`, which are exactly the intervals a permit may name. `independent` lists the independent_review files that carry `views` and contain the movie hash.

**The tail of `preview_binding`** refuses unless:
1. the chapter's own `D/jobs/<preview job>/<chapter>/sampling.json` is viewed by an evidence file (root's full sampled pass);
2. `independent` is non-empty;
3. `runs` is non-empty;
4. `sampled_review_scope(review, frames, MOVIES[chapter], sampled)` passes;
5. `playback_seconds == []`.

`check()` also requires the 344 key set and `independent()`: an independent reviewer other than the reviewer, with files directly in Q at their hashes.

## 5. Permits

**Common to every chapter:**
- the fields as in 344's template: `reviewer` `root`, `actually_reviewed_pictures` true and `unresolved_defects` `[]`;
- `binary_sha256` `983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7`;
- `timeline_sha256` = sha256 of `D/jobs/csd-review-repair-timeline269/timeline-output/<chapter>.timeline.tsv`;
- `host` `csd` or `css`, one host per chapter;
- `review_scope`: `{"basis": "sampled_preview", "purpose": "technical_4k_render", "preview_movie_sha256": MOVIES[chapter], "sampled_intervals": <runs>, "playback_seconds": [], "open_reviews": ["final_picture", "motion", "audio", "fine_sync", "captions", "independent_final_av", "publication"]}`;
- the six `*_acceptance` flags `false`;
- `source_binding_evidence` as in 303. For 06 and 08 it must hold the one named `build238-to-build268-delta` item (binding282, `1fbcedef…`).

**Root's sampled-pass records** (they `views` the chapter's sampling, cited by every sheet hash):
- `Q/picture-review-progress305.json` `d4547c553a9c6dddabbc5d07448a86f447b0ddc9cac9a70930b2cec63fe9abb2` for the eight preview274 chapters;
- `Q/preview243-root-review-complete277.json` `0c8299120c29cf1f6aebd2e76891dc9cfe5467c3654bb6f8ef8617effd00f9cc` for 06 and 08.

**Sampling files** are at `D/jobs/csd-review-repair-preview274/<ch>/sampling.json` (or preview243 for 06 and 08), with sha256:

| Chapter | Sampling sha256 |
|---|---|
| 01 | `35854fb507a7d6b2f01db0ebe0c336bfedcaaf1df1cb72f408cea2eec2b08abc` |
| 04 | `bbd19ad68735188e86d51732427a1f5faa33a39122cc126c39caae9ca7a33b21` |
| 05 | `b8011d9025f5128b3582990cff2cfc59337f3d939ec94c226accd7fccf23c1b9` |
| 06 | `34ee8c6ae24ccd33e234ac2b1b27cf5441fae1087af8f53f4b4436942efd22ba` |
| 07 | `efa82174af0134e4a82fbf77a3006927340b4430fd94ea7174b9515c8b77f0c4` |
| 08 | `951eab4a4c64517a5cff9b8ff824f79c83cca4d5b1f614a6157498383d48a7da` |
| 10 | `e5cd3c52068643b029f7788d443de0e0aea2311eec56fdb98f56ac02c6d49d16` |
| 11 | `eb67bf33a9ad2f7c91c5601026816d7ebd5d1fc3dafb5a68ae3c5004e8307846` |
| 12 | `df4ac4872fe729b2cb11b91b44041f143f9cfca707a4872c3056503cda59272e` |
| 14 | `5faa729974a76a216bfcc5d2a36975df67fabd0bd79a07a1912fc784db5feb2b` |

**Per chapter.** The format is "file `sha256` → what its `views` lists". Frames are the timeline frame counts.

- **01_the_bump** (preview274, 6311 frames). Intervals `[[5493,5531],[6273,6310]]`.
  - Evidence:
    - the sampling;
    - root 305 → sampling;
    - `Q/bump-transition-frames305/manifest.json` `e2341ee728c464a4220aada1459a064d29f48efac7c6cc851c64b9c5753f7f9a`;
    - `Q/bump-transition-review305.json` `bc48a64de5fe9c69e00874fed6ba1326959f4fbb4da07409cfed35a4a0e6e2b3` → bump manifest;
    - `Q/coverage-grid305b/01_the_bump/sampling.json` `54071925d3b867add04ac26ed93da0e63603fbb3d40ffef4ca44ef8554a27ab8`;
    - optionally `Q/grid315-disposition305.json` `c92a747877ff7ada1c5f2546038312bef9e337a9adf1eaf075376b8f5c873f0c` (see §7, Q8).
  - Independent (`cc3`): `Q/cc3-current-grid-review315.json` `e9a881ffd9a0a2cd813eecdcb4e94a3824f55c4e51743ec69feae389d943d866` → 01 grid.
- **04_reading_color** (preview274, 6774 frames). Intervals `[[235,315],[6742,6773]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/color-transition-frames305/manifest.json` `a58e25460f12f09ed228a1ddd7b5df097048c7d2b27cb746c428a78c9b75e192`;
    - `Q/color-transition-review305.json` `fdd04eab7439fb3a82b230992c67d6b8534b467126b82f059dc9e7ac1ba6f181` → color manifest;
    - 04 grid `073e69f845e69817e720fa14584403a672236494451c3537c5b42281e4c8fd75`.
  - Independent: cc3 315 → 04 grid.
- **05_gallery** (preview274, 9879 frames).
  - Intervals `[[34,98],[1989,2072],[2894,2977],[4772,4855],[6181,6264],[6872,6877],[6921,6927],[7312,7365],[8715,8859],[9167,9221],[9247,9248],[9817,9878]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/gallery-transition-frames305/manifest.json` `5a98dd205a39f9b596f61e9c3dcc048b75c3aa9f1aac286f1872ab26acfe7d49`;
    - `Q/gallery319-disposition305.json` `de33e72ed1f3e7bd10d2e8f41f6e2252fac5fcaa089054861cf93a16c376fac6`;
    - 05 grid `f67f171bd2d126c92ed98b1c3c99c759cd4df8cee0dd70bb5ac8c853d05c85f1`.
  - Independent (`cc1 and cc3`):
    - `Q/cc1-gallery-transition-review319.json` `58a7381dd8506497e10bd147d54837a6472e72651e3724e9dd99747f83e6e651` → gallery manifest;
    - cc3 315 → 05 grid.
- **06_one_derivative** (preview243, 10030 frames). Intervals `[[9630,9730],[9970,10029]]`.
  - Evidence:
    - the sampling;
    - root 277 → sampling;
    - `Q/derivative-sheets359/manifest.json` `ca470cb023df865af986c44601dc64c0c0c80f3a56d04c2f393aad936b6c78b1`;
    - `Q/derivative-sheets359-review.json` `40e2fca805e0dfb7b138b2ef68c2770b947bd7ae5e435e3d712bf2ea476d34de` → 359 manifest.
  - Do not give 358's manifest any `views` (root: not a complete review). If it is bound, it is reported as unviewed.
  - Independent (`cc0`): `Q/cc0-preview06-review270.md` `d4ad3533de6a6034285973a2a921a235d319fc569073d36c1b41e999c4d6b531` → sampling. It viewed all 47 sheets.
  - Binding282 delta item as above.
  - 270's cosmetic Lows R1 and R2 have no root disposition record that I found. Root decides whether `unresolved_defects: []` holds.
- **07_one_point** (preview274, 6491 frames). Intervals `[[2216,2284],[4743,4779]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/one-point-transition-frames305/manifest.json` `e6fa64dd79ec4f2e2478c65e2ea8b4cec6a1bf12bb82504c31fe7e8bace874a9`;
    - `Q/one-point-transition-review305.json` `02e8594d4b8b92bb4654b115e5f1b13fbc4f1d3c20e14aad3ac943c31ad857ee` → its manifest;
    - 07 grid `fbc372dca14d06048eacea1be913646a9e0041ff4cf0453cbc26598a25fb7916`.
  - Independent: cc3 315 → 07 grid.
  - Two playback records exist (open-region-playback305 and -second305). 352 binds no playback.
- **08_loop_sums** (preview243, 14959 frames). Intervals `[[3059,3060],[3119,3120],[3179,3180],[10200,10380]]`.
  - Evidence:
    - the sampling and root 277 → sampling;
    - `Q/preview08-disposition285.json` `40ad7b1250ad016aa98535440abe6a48b451d0f4e67d585eba4b2b98091aaaa5`.
  - Independent (`cc2`): `Q/cc2-preview08-review271.md` `d70fb0deec2bfc562c0815e57678019769cc84544c940bbfe192f5ac5b242478` → sampling. It viewed all 69 sheets.
  - Binding282 delta item as above.
  - targeted-playback305 exists and is not bound.
  - cc2 wrote both 271 and 352, so 352's own review must be by someone else.
- **10_rigidity** (preview274, 7442 frames). Intervals `[[3735,3799],[6240,6335],[7410,7441]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/rigidity-transition-frames305/manifest.json` `ba0eb53d847dd803015545c924b3a951f85be8257736a747772bc9ae39122c26`;
    - `Q/rigidity-transition-review305.json` `ac897e91918f60183d0c500b2922375bf5d682317a982c0c68baaf3960032bea` → its manifest;
    - 10 grid `04b35730a9171aae67d2a5c2f064c5ac32ca9898dd6e0efd0ca8670af8483c82`.
  - Independent: cc3 315 → 10 grid.
- **11_counting** (preview274, 11932 frames). Intervals `[[34,87],[7464,7494],[8294,8344]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/counting-transition-frames305/manifest.json` `9e74486edaaac78d1a4a868f83507a69378f1be9d64ab0456ddd453a34fc64d6`;
    - `Q/counting-transition-review305.json` `a8552f249637b7348e1367fb0b74708b727be498aa92760eba67995b3bdc91e7` → it;
    - `Q/counting-opening-frames305/manifest.json` `9b889e043ce97bfbb2ce8b72f8306c3dbd3718a29c52b97f4adefa2b702bea8a`;
    - `Q/counting-opening-review305.json` `5e1f55523d92cca846e060276bb4b2635d305fceee8ff516ae290e8277573aec` → it.
  - Independent: cc1 357 (pending) → the manifests it actually viewed.
- **12_residues** (preview274, 3943 frames). Intervals `[[2471,2516],[2532,2577],[2763,2806],[3025,3154],[3911,3942]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/residue-transition-frames305/manifest.json` `c6cca51f4290b32b7e4f2587900cdf1adefde66bf48d032b32f322edd684eea2`;
    - `Q/residue-transition-review305.json` `b3a90d158b6df6453908c95a755a0cb26257673ccbda61e715efdacf4dee397d` → it.
  - Independent: cc1 357 (pending).
- **14_continuation** (preview274, 11756 frames). Intervals `[[34,98],[1223,1258],[8876,8940],[10179,10301],[11724,11755]]`.
  - Evidence:
    - the sampling and root 305;
    - `Q/continuation-transition-frames305/manifest.json` `a787e39d74c54651b8bbb3a97f0c75e7c863be4ab946c3004bb5aa153e34649b`;
    - `Q/continuation-transition-review305.json` `1a92e0f771a65fc385ad07ee453e32dc71e2aec57106210bc6b9c381a2eaa9a8` → it.
  - Independent: cc1 357 (pending).

**Intervals.** The intervals listed are the maximal runs. Root may name any sorted, disjoint sub-intervals of them, each at least two frames long.

**What cc1 was told for 357** (Agent message, 21:2x UTC). One file directly in Q, containing each chapter's exact movie sha256 and the full sha256 of each manifest or sampling it actually viewed. Root then binds it with `views` listing those records.

## 6. Staging (root)

- **CSD.** Copy the Q records named in §5 into CSD Q at the same relative paths, unchanged, at those hashes. On CSD today Q holds none of them except the 344 records.
- **Preview job folders.** The preview movies, samplings and sheets are already at the D paths, in the preview274 and preview243 job folders on CSD. All 267 movie, sampling and sheet files there equal the reviewed local copies by sha256. On CSS they are under the 309 runtime transfer. I spot-checked three on CSS: 11 and 06 sampling, and the 12 movie.
- **Workers.** Stage `csd-reviewed-final352.py` and `csd-launch-final352.py` in CSD Q. Stage `css-final-repair352.py` in CQ and also in CSD Q: its `manifest` and `send` modes run on CSD, as 309's did.
- **CSS permits.** The permit file sits under D on CSD and is named `<chapter>-….json`. The 309 flow, unchanged:
  1. `manifest --chapter --permit` (CSD);
  2. `land-manifest` (CSS);
  3. `freeze-transfer` + guarded `receive` with job `css-repair-permit352-<chapter>…` (CSS);
  4. `launch --permit-sha256 --permit-transfer` (CSS).

  The permit lands in `CQ/repair352-permits/`. The transfer carries evidence, source_binding_evidence and independent_review files to `FROZEN/<their D path>`.
- **CSD launch.** `csd-launch-final352.py --chapter --permit --permit-sha256 --worker-sha256 23d55c28…`. Root passes the hashes.

## 7. Answers to CC0 (master350)

1. **Hashes.** As in §2. On CSS the worker is `CQ/css-final-repair352.py` (`R/qualification01/css-final-repair352.py`), sha256 `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a`.
2. **`review_basis`.** It takes two arguments, `(chapter, review)`, because the records come from the permit; no record is pinned in code. The return and formats are in §4.
   - **To recompute the sampled frames:** for each bound item with `views`, for each listed hash, find the bound file with that hash and read its frames by the first list present: `samples[].frame`, `frames[]` (int or `.frame`), or `pictures[].frame`. Read its movie by the first key present: `movie_sha256`, `source_sha256`, `sha256`.
   - **Citation check:** the viewing file's bytes contain the hash, or every `sheets[].sha256` of a samples record.
   - **Exact formats per record kind:**
     - preview sampling.json: `movie_sha256`, `samples`;
     - bump, color, counting-transition and residue manifests: `sha256`, `frames` [int];
     - one-point, rigidity, counting-opening, continuation and gallery manifests: `source_sha256`, `frames` [{frame}];
     - coverage-grid305b/<ch>/sampling.json: `movie_sha256`, `frames` [int];
     - derivative-sheets359/manifest.json: `movie_sha256`, `pictures` [{frame}].
3. **Is it safe on CSD for a CSS row?** Yes. `review_basis` only reads D paths and the review dict. The CSS module defines only constants at import, and its `main` is guarded. `check()`, `independent()` and `preview_binding()` of the CSS worker are CSS-only: they call `alias_is_exact`.
4. **Per host.**
   - CSD: `host` `csd`, job `csd-reviewed-final-<dashed>352`.
   - CSS: `host` `css`, job `css-reviewed-final-<dashed>352`.
   - `preview_job` from `PREVIEWS`, the same on both hosts: `csd-review-repair-preview243` for 06 and 08, `csd-review-repair-preview274` for the other eight. The values are 303's.
5. **Render-summary keys.**
   - CSD: 303's keys plus `independent_review` (the permit's), `review_scope` and `review_basis` = `{movie, frame_records, independent, runs}`. `preview_binding` now also lists the movie and the sheets read. `encoder` is the x264 settings read from the movie. `scope` is the 344 sentence.
   - CSS: 309's keys, with `worker352_sha256` in place of `worker309_sha256`, plus `derived_from_worker309_sha256`, `independent_review`, `review_scope` and `review_basis`.
6. **Existing finals.** On CSD, among `csd-reviewed-final-*` only `csd-reviewed-final-09-boundary344` exists. On CSS there is no `css-reviewed-final-*`. `css-production19-final-11-counting78` and `css-production19-final-13-real-integral96` exist, and no 352 glob matches them. I plan to keep none.
7. **playback_seconds.** `[]` is confirmed on all ten rows; 352 refuses anything else. 09 is unaffected: final344 still needs 343's instants.
8. **Two things 350 should know.**
   - Items may carry a new key, `views`. 350's per-item sha checks ignore it.
   - `Q/grid315-disposition305.json` has `"no_master_or_release_acceptance": true`. This is a negation, but 350's `claimed()` would flag it if 350 walks records. 352 never walks records for claim words. Either exempt the file, or root leaves it out: it is optional, and 01/04/05/07/10 pass without it.

## 8. Checks I ran (local, read-only on the records)

- **Fixture test: 93/93 pass.** It is session scratch, `fixture_test352.py` `31d54fc6…`. It maps D to the reviewed local copies and runs the real Q records. It covered:
  - `review_basis` on all ten chapters, with nothing unviewed, and CSD equal to CSS;
  - the preview_binding tail, exec'd from the worker's own source, plus the key-set and `independent()` gates;
  - 01/04/05/06/07/08/10 passing;
  - 11/12/14 refused with the real records, and passing the tail with a synthetic 357-form record held in scratch;
  - **master350's own `sampled_scope`** agreeing on the intervals for all seven passing rows, and its `pinned_scope` accepting both workers.

  Refusals tested:
  - motion boolean;
  - unviewed frames in an interval;
  - playback instants;
  - an acceptance claim;
  - motion review not open;
  - another movie;
  - sampling not bound;
  - sampling without root 305;
  - a manifest whose review is removed (unviewed, window refused);
  - a hash mismatch;
  - another chapter's records (no frames added);
  - 06 without 359 ("no two consecutive viewed frames");
  - 358/359 without `views` (unviewed);
  - 08 without independent;
  - 05 without cc1 (gallery windows lost);
  - mentions alone viewing nothing;
  - `views` of an uncited record, of a non-frame file, or of another chapter's record;
  - an independent file without `views`;
  - a sampling viewed only by an independent file;
  - 05's sampling credited to root 305 only;
  - its 28 sheets frozen.
- **Pins and names.** The 344 key-set pin `b051136c…` holds in both workers, and so do the names in §3.
- **AST delta** as in §3. All three files compile in memory; no `.pyc` was written.
- **Path hygiene.** The three code files contain no local-machine or temporary paths: grep for `/Users/`, `/private/`, `jemanuel`, `/tmp/`, `scratchpad` finds nothing. They contain host paths only.

## 9. Still open

- An independent picture record for 11, 12 and 14 (cc1 357), then root's permits.
- 06 R1/R2 disposition, root's call.
- Root's host choice per chapter.
- An independent review of 352 (not cc2).
- 350's fit to these bytes (CC0).
- Then launch, final pictures, motion, audio, fine sync, captions, final AV, master and publication, all still open.
