# Candidate technical-render permits 368: 05_gallery (css), 06_one_derivative (csd), 08_loop_sums (csd)

**Status: candidates for root review.** cc1 (BronzeJaguar) prepared them on 2026-11-08 (UTC). They are not staged, not transferred, not launched and not approved. Each one becomes a permit only if root reviews it and adopts it under root's own permit name. They cover a technical 4K render from a sampled-preview disposition only. Final picture, motion, audio, fine sync, captions, final AV and publication all stay open.

Disclosure: 05 binds my own `cc1-gallery-transition-review319.json` as one of its two independent reviews. The 06/08 delta item is my `cc1-preview-to-final-binding282.md`, as the 352 worker requires by constant.

`D/...` below is the CSD project root named in `csd-reviewed-final352.py`. `Q/...` is this production-repair17 folder, which is `D/qualification01/production-repair17` on CSD. The candidate JSONs hold absolute D paths, as root's permit364/366 files do.

## Files

| File | sha256 | Host / worker | Preview job | Frames | timeline_sha256 |
|---|---|---|---|---|---|
| `05_gallery-css-candidate368.json` | `f3ef9aeabeb5220ad0cdd42e10868845719e67feac2680fbbad469dfae126b10` | css / `css-final-repair352.py` | preview274 | 9879 | `df3cdeae1dab…` |
| `06_one_derivative-csd-candidate368.json` | `c43801cc7fce42e3ff44cbd2dfd8ed353e14b14b2a62dff7aabcbc4e6542eab2` | csd / `csd-reviewed-final352.py` | preview243 | 10030 | `8abb0519b294…` |
| `08_loop_sums-csd-candidate368.json` | `c2d0b172933e2cb2505ec87b91f3bb343b5591e8a5818a8f6ee2e5b240f2a79c` | csd / `csd-reviewed-final352.py` | preview243 | 14959 | `e2276be4d8b1…` |
| `checks368.json` | `49b927ec52c9ac02a53f0267ce5c97c323a488c2a29f28644504b98236a16f09` | 72 checks, 0 failed; per-chapter frame records, viewers, runs | | | |

## How they were built

- **Format.** Key order, the binary and the four `source_binding_evidence` items are copied from root's own permits: 01 permit364, and 04, 07 and 10 permit366. They are identical in all four.
- **Delta item.** 06 and 08 add the one named `build238-to-build268-delta` item (binding282, `1fbcedef…`). These two chapters come from preview243, whose build is 238.
- **`timeline_sha256`.** This is the `new_sha256` in `Q/timeline269-comparison305.json`. I also recomputed it from the local preview tables. The preview243 tables first get the rebind242 → rebind269 narration-path mapping, as `preview_binding` applies it. The same method reproduces root's timeline hash and frame count for 01, 04, 07 and 10.
- **Hashes.** Every evidence and independent file is hashed from the local reviewed copy (Q, `Q/preview274-review305`, `Q/preview243-review265`).
- **`views`.** These go only on viewing files: root's records and the independent reviews. Each listed record is cited in that file's bytes, under the worker's own citation rule. No sampling, manifest, grid or disposition carries `views`, and no item lists its own hash. This avoids the self-view case, L1 of cc1 355.
- **`sampled_intervals`.** These are the maximal runs that 352's `review_basis` returns on the candidate. They equal the intervals in cc2's 352 notes, §5.
- **Worker run.** Each candidate went through its host's frozen 352 code, exec'd unchanged on the local copies:
  - `review_basis`;
  - the 352 tail of `preview_binding`;
  - `sampled_review_scope`;
  - the key set;
  - `independent()` (CSD);
  - the binding282 named-item lines (06/08).
- **Both workers agree.** CSD and CSS `review_basis` give the same result for every chapter.
- **Not run:**
  - the 303/309 gates that need host files (receipts, tables, score, narration, encoder);
  - CSS `alias_is_exact()` and CSS `independent()`;
  - any transfer or launch.

## Evidence per chapter

`→ x` means the item's `views` lists x.

**05_gallery** (css). Runs:
`[[34,98],[1989,2072],[2894,2977],[4772,4855],[6181,6264],[6872,6877],[6921,6927],[7312,7365],[8715,8859],[9167,9221],[9247,9248],[9817,9878]]`

| Item | sha256 | Role |
|---|---|---|
| `D/jobs/csd-review-repair-preview274/05_gallery/sampling.json` | `b8011d90…` | frame record (222), viewed by root 305 |
| `Q/picture-review-progress305.json` | `d4547c55…` | root → sampling (all 28 sheet hashes) |
| `Q/gallery-transition-frames305/manifest.json` | `5a98dd20…` | frame record (715), viewed by cc1 319 |
| `Q/gallery319-disposition305.json` | `de33e72e…` | root disposition of 319; no views |
| `Q/coverage-grid305b/05_gallery/sampling.json` | `f67f171b…` | frame record (129), viewed by cc3 315 |
| independent `Q/cc1-gallery-transition-review319.json` | `58a7381d…` | cc1 → gallery manifest |
| independent `Q/cc3-current-grid-review315.json` | `e9a881ff…` | cc3 → 05 grid |

- Nothing is unviewed.
- The gallery windows rest on the independent cc1 319 view of the manifest; root's 319 disposition accepts that review.
- D3 (sin z zero marks at the panel edge) was accepted in `Q/grid315-disposition305.json`. That file is named in the observations, as root's 04 permit366 does with its D1/D2, and is not bound. Its `no_master_or_release_acceptance` key would be flagged by master350's `claimed()` (cc2 352 notes §7.8).

**06_one_derivative** (csd). Runs: `[[9630,9730],[9970,10029]]`

| Item | sha256 | Role |
|---|---|---|
| `D/jobs/csd-review-repair-preview243/06_one_derivative/sampling.json` | `34ee8c6a…` | frame record (376), viewed by root 277 and cc0 270 |
| `Q/preview243-root-review-complete277.json` | `0c829912…` | root → sampling (sheet hashes) |
| `Q/derivative-sheets359/manifest.json` | `ca470cb0…` | frame record (161), viewed by root 359 review |
| `Q/derivative-sheets359-review.json` | `40e2fca8…` | root → 359 manifest |
| `Q/root-preview06-disposition362.json` | `1822a590…` | root disposition of cc0 270 R1/R2. It holds a two-picture list (frames 600 and 1680), so 352 counts it as a frame record. Nothing views it, so it is reported as unviewed and adds no frames. Do not give it views. |
| independent `Q/cc0-preview06-review270.md` | `d4ad3533…` | cc0 → sampling (47 sheet hashes) |
| source binding `Q/cc1-preview-to-final-binding282.md` | `1fbcedef…` | named delta item |

- `Q/derivative-detail358/manifest.json` is not bound. Root said it is not a complete review.

**08_loop_sums** (csd). Runs: `[[3059,3060],[3119,3120],[3179,3180],[10200,10380]]`

| Item | sha256 | Role |
|---|---|---|
| `D/jobs/csd-review-repair-preview243/08_loop_sums/sampling.json` | `951eab4a…` | frame record (552), viewed by root 277 and cc2 271 |
| `Q/preview243-root-review-complete277.json` | `0c829912…` | root → sampling (sheet hashes) |
| `Q/preview08-disposition285.json` | `40ad7b12…` | root disposition of cc2 271. Its `sha256` key names 271, not the movie, so it is not a frame record. No views. |
| independent `Q/cc2-preview08-review271.md` | `d70fb0de…` | cc2 → sampling (69 sheet hashes) |
| source binding `Q/cc1-preview-to-final-binding282.md` | `1fbcedef…` | named delta item |

- Nothing is unviewed.
- `[10200,10380]` is root 277's "every frame from 170 to 173 s" inside the sampling. The three pairs are consecutive samples in the same sampling.

## Root to decide or confirm

1. **`observations`.** I drafted the text from the records. It is root's statement, so replace it as needed.
2. **`unresolved_defects: []`.**
   - 05: D3 is accepted in grid315-disposition305.
   - 06: R1/R2 are retained in 362, with final-output frames listed for recheck.
   - 08: every 271 item is disposed in 285. Its `remaining` list is the final 4K, motion and listening reviews.
3. **`independent_reviewer`.** The strings are `cc1, cc3` for 05, `cc0` for 06 and `cc2` for 08.
4. **Hosts.** These are as instructed: css for 05, csd for 06 and 08.

## Checklist before any staging (root; every command reads only)

**A. Each bound file is on CSD at its hash.**

Run this on CSD with the candidate copied there. For 05, run it on CSD too: the CSS permit transfer copies from these D paths.

```
python3 -I -B -c 'import json,hashlib,sys
p=json.load(open(sys.argv[1]))
for k in ("evidence","source_binding_evidence","independent_review"):
    for x in p[k]:
        try: ok=hashlib.sha256(open(x["path"],"rb").read()).hexdigest()==x["sha256"]
        except OSError: ok=None
        print({True:"OK      ",False:"BAD     ",None:"MISSING "}[ok]+x["path"])' CANDIDATE.json
```

These are the files most likely still missing from CSD Q, judging by root's 01/04/07/10 permits. Root 305 and cc3 315 are already bound there.
- 05:
  - `gallery-transition-frames305/manifest.json`;
  - `gallery319-disposition305.json`;
  - `coverage-grid305b/05_gallery/sampling.json`;
  - `cc1-gallery-transition-review319.json`.
- 06:
  - `preview243-root-review-complete277.json`;
  - `derivative-sheets359/manifest.json`;
  - `derivative-sheets359-review.json`;
  - `root-preview06-disposition362.json`;
  - `cc0-preview06-review270.md`;
  - `cc1-preview-to-final-binding282.md`.
- 08:
  - `preview243-root-review-complete277.json`;
  - `preview08-disposition285.json`;
  - `cc2-preview08-review271.md`;
  - `cc1-preview-to-final-binding282.md`.

Copy each one unchanged, at the same relative path, per cc2 352 notes §6.

**B. Movie, sampling, sheets and timeline on CSD.**

`review_basis` re-hashes the 28 (05), 47 (06) or 69 (08) sheets of the sampling, because root 277/305 and cc0/cc2 cite it by sheet hashes. The worker refuses on any mismatch.

```
sha256sum "$D"/jobs/csd-review-repair-preview274/preview/05_gallery.mp4 "$D"/jobs/csd-review-repair-preview243/preview/06_one_derivative.mp4 \
  "$D"/jobs/csd-review-repair-preview243/preview/08_loop_sums.mp4 \
  "$D"/jobs/csd-review-repair-timeline269/timeline-output/{05_gallery,06_one_derivative,08_loop_sums}.timeline.tsv
```

Expected:
- movies `f5123300…` (05), `5e45d72d…` (06), `1ac1ee71…` (08), the workers' `MOVIES`;
- timelines `df3cdeae…`, `8abb0519…`, `e2276be4…`.

**C. Sampled intervals, recomputed by the worker itself.**

For 06 and 08, the full CSD `check()` reads only: its only `open` is `'rb'`. Run it with the permit at an absolute, symlink-free path:

```
python3 -I -B -c 'import importlib.util as u,sys
s=u.spec_from_file_location("w",sys.argv[1]);w=u.module_from_spec(s);s.loader.exec_module(w)
r,h,b=w.check(sys.argv[2],sys.argv[3]);x=w.review_basis(sys.argv[2],r)
print(h,x["runs"],[p.rsplit("/",1)[1] for p in x["unviewed"]],[p.rsplit("/",1)[1] for p in x["independent"]])' \
  "$D"/qualification01/production-repair17/csd-reviewed-final352.py 06_one_derivative /ABSOLUTE/PERMIT.json
```

For 05, CSS `check()` needs the CSS alias. Before the 309 manifest/transfer flow, root can run the CSS module's `review_basis` on CSD (cc2 352 notes §7.3). Load it the same way and print `w.review_basis("05_gallery", json.load(open(PERMIT)))["runs"]`.

Expected, as computed here:

| Chapter | runs | unviewed | independent |
|---|---|---|---|
| 05 | the 12 runs above | none | cc1 319, cc3 315 |
| 06 | `[[9630,9730],[9970,10029]]` | `root-preview06-disposition362.json` | cc0 270 |
| 08 | `[[3059,3060],[3119,3120],[3179,3180],[10200,10380]]` | none | cc2 271 |

Root may name any sorted, disjoint sub-intervals, each at least two frames long. Anything else is refused.

**D. Then stage per cc2 352 notes §6.**
- 06 and 08: `csd-launch-final352.py --chapter --permit --permit-sha256 --worker-sha256 23d55c28…`.
- 05: the 309 CSS flow (`manifest` → `land-manifest` → `freeze-transfer` + `receive` → `launch`) with worker `1074051b…`. The permit lands in `CQ/repair352-permits/`.

These are root's steps; I ran none of them.

## Limits

- I used local copies only. cc2's 352 notes say the CSD copies of the preview movies, samplings and sheets equal them. I did not check any host.
- The 303/309 host gates were not executed here; check A to C covers them on CSD.
- The candidates rest on still-picture reviews. They make no claim of motion, listening, fine sync, captions, 4K output, master or publication.
