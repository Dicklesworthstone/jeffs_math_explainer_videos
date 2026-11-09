# cc0 repair 313: guarded build, rebind and targeted 09 preview helpers for repair310

cc0, 2026-11-08. Helpers prepared and frozen for root; none of them has been run on the build host.

**Not done:**
- no launch, build, render, deploy or permit;
- no recipe313 (root writes it);
- no canonical edits;
- no Beads or TODO edits;
- no edits to 303, 310 or any older helper.

Narration stays the current ElevenLabs Frederick Surrey set. No audio heard or checked.

## Files

All files are new and sit in `production-repair17/`.

| File | sha256 | Base it adapts |
|---|---|---|
| `review-repair-build313.py` | `ff1ff0b8619cce99d87f2a5c5d44da1ac106a5cb70f698af2e93b64f3248b86a` | `review-repair-build268.py` |
| `launch-review-repair313.py` | `d384d4d8cdc6da3dd2f17a568748cc585e7139025f60028ea9823c11fd0141e6` | `launch-review-repair268.py` |
| `review-repair-after-build313.py` | `f28a15ab6a33fd330a11cdb904e24753aa5a6ff88c4f940447bdfcb9461ded0b` | `review-repair-after-build269.py` |
| `review-repair-rebind313.py` | `0e04aa9eaa4f824f385459f24c7ca682f4df497d3fd424e756d34332486c2f8d` | `review-repair-rebind269.py` |
| `review-repair-timeline313.py` | `1034931710af7c46c79b09c391122bd22a38471d1d1f73fd501f09ec672d474f` | `review-repair-timeline269.py` |
| `review-repair-preview313.py` | `67d780d83bf05fbcd0285f0599197649f8ce8813ef6599a5da932301dc354e50` | cc3's `cc3-preview-fade274.py` |
| `launch-review-repair-preview313.py` | `c983ecbff973cf5c300ec1f8b8837a175b0a41f8aa294bd7826f9a1b4d42806a` | `launch-reviewed-preview296.py` |
| `preview-after-timeline313.py` | `96af8d850d8dd05a7dea6062115e17271a759f38923fbf3710b794d3263ed7ec` | `preview-after-timeline301.py` |

**Pins between the files:**
- the build launcher pins the build worker;
- the controller pins the rebind and timeline workers;
- `preview-after-timeline313.py` pins the preview launcher and the preview worker.

**Inputs from earlier work:**
- the source is `boundary-guide-repair310.rs` (`bda7d931…`, frozen);
- root's acceptance of it for guarded qualification is `boundary312-disposition305.json` (`e4152e15…`);
- the review is `cc1-boundary-review312.md` (`15ae2d9a…`).

## The chain, and what each step proves

**1. Build313** (`launch-review-repair313.py`, then the worker under the guard, timeout 22000).
- **Base check.** The base is build268's own `source-copy.json` (pinned `81d28272…`, 70 files). The worker checks the live 268 workspace against it.
  - The workspace must hold exactly those 70 files plus the two that rebind269 added: `narration_units.before-review269.json` and the review269 map.
  - Every file must be at its source-copy hash, with one exception: `narration_units.json` must be at rebind269's recorded `canonical_sha256`.
  - The backup must equal build268's `narration_units.json`, and the map must equal rebind269's `map_sha256`.
  - Anything else stops the build.
- **Rebuild.** `W/review-repair313/workspace` is rebuilt file by file. `narration_units.json` comes from the byte-exact backup, so the tree equals build268's compiled tree. The worker asserts this.
- **The one substitution.** It replaces `boundary.rs`, `dc80c541…` → `bda7d931…`, and asserts that the change list is exactly that file. The candidate must also be named in root's recipe313.
- **Gates.** These are build268's gates and pins, unchanged:
  - toolchain pins (cargo `b854c942…`, rustc `49f9fa06…`), the same environment and CPUs 0–3;
  - 233 tests passed with 1 ignored;
  - the plate-fade probe;
  - Clippy `-D warnings`;
  - `rustc -C metadata=jmev_review_repair313`, giving the new binary `W/bin/review-repair313`, which must not already exist;
  - facts hash `7d528509…`, unchanged;
  - chapters equal build268's.
- **One change: format runs as `cargo fmt --check`.** Build268 ran a mutating `cargo fmt`. Here the compiled bytes must stay exactly the reviewed candidate's, so a format difference fails the build instead of rewriting the file.
- **Records.** `source-copy.json` and `qualification.json` are written with exclusive create, and record the base and the substitution.

**2. Rebind313** (the controller waits for build313, then runs it under the guard with `--run`).
- **Inherited from rebind269.** Most of the logic is rebind269's:
  - relocation from the production19 map;
  - the allowed-file check (build268's eight files, relative to production19);
  - the three narration metadata files rewritten;
  - coherent `verify`, and `narration_delivery` with 105 verified cues.
- **Added: an exact comparison with rebind269's map.**
  - The helpers and 20 holds must be equal.
  - Only the three `boundary.rs` holds may change: the new file hash, the same marker, and exactly +4 lines: 09_4 938→942, 09_5 1003→1007, 09_8 1282→1286.
- **Added: an exact comparison with rebind269's narration.**
  - The same file set.
  - All 105 WAVs byte-identical to rebind269's and to assembly19's.
  - Only `external-units.json`, `report.json` and `assembly.json` may differ.
  - The same report sentence rows and outputs.
- **Two other changes.**
  - The backup is named `narration_units.before-review313.json`.
  - The disposition reason text now ends "no audio heard or checked". `coherent_narration.py:181` only requires a non-empty reason.

**3. Timeline313** (from the same controller).
- It is targeted: only `09_boundary`, timeline-only, on 4 CPUs.
- Its three tables must equal:
  - timeline269's, under the named rebind269 → rebind313 path mapping;
  - production timeline21's, under assembly19 → rebind313.
- The frame total must be 16,504.
- **The other 14 chapters are not re-run under binary 313.** If a later final binds chapters other than 09 to binary 313, it needs a full 15-chapter comparison first: about 94 min, as for timeline269. That is root's call.

**4. Preview313** (`preview-after-timeline313.py --review … --review-sha256 …` waits for timeline313, then calls `launch-review-repair-preview313.py`).

The worker is cc3's 274 worker with narrow edits:
- 313 paths, and chapter 09 only (the other chapters' site tables are kept as cc3 wrote them, unused);
- `SOURCE_SHA256['boundary']` = `bda7d931…`;
- **site E2 rebound to repair310's lines:**
  - `painter.leave` 1398 → 1402;
  - breath anchor 1377 → 1381;
  - before-play 1397 → 1401;
  - `kit.finish` 1399 → 1403;
  - E1 stays at 474;
- the schedule is byte-equal to timeline313 and equal to timeline269 under path mapping;
- the same render flags as 274 (540p60, `--lint`, CRF 18 veryfast, 16 threads).

**Targeted picks**, added to cc3's standard ones (the first and last frame, line and mark starts, both plate sites, the lint lines):

| Pick | Frames | Source |
|---|---|---|
| 28–40 s, every 0.25 s | lines in, held, gone | review312 L2 |
| every frame 2080–2106 | the new exit, inside the hole name's fade | root, review312 L2 |
| every frame 2316–2336 | `\|g\| ≤ M` arrival at 2326 | root |
| 33.48, 69.67, 90.28 s, each at −0.2, 0, +0.2, +1, +2 s | root's three samples | `boundary-proof-gap305.json` |
| 104 s, same offsets | lower line into "L: arc + chord lengths" | review312 L3 |
| 115, 116, 117 s, same offsets | lower line through "for every small ρ" | root's `coverage-grid-review305.json` |
| 39–133 s, every second | persistence sweep | root grid, review312 L2 |
| each probe anchor | exact frames | probe |
| exit −2 to +15 frames, and each flagged frame | where the probe found something | probe |

**Zoom-line probe** (new; it runs before the picks, so its findings are picked too).
- **What it reads.** It decodes every frame 1500–8400 of the new movie and of the control movie: preview274's 09 (`dba964d2…`, the build268 base, frozen and hash-pinned). It counts how many of 21 points on the two lines are lit.
  - The points are taken from the source geometry: `zoom_links` corners (2.4, 1.05) and (2.4, −2.95), and the circle at ±0.7 through `picture()`'s mapping.
  - They lie outside the column where the proof text sits.
  - A point is lit when its 5-pixel peak exceeds the median of its ±6–9 px ring by 15 or more.
- **It fails unless all of these hold:**
  - the control shows the lines fully (≥ 19 of 21) by frame 1740, continuously, to at least frame 7800;
  - before the lines arrive, the new movie matches the control within 2 points;
  - from 15 frames after the lines arrive until 2085, the new movie holds them like the control;
  - the new movie's last present frame is in 2086–2100;
  - from 15 frames after that exit to the control's last present frame, at most 2 points are lit;
  - at 2326, 4180, 5417, 6240, 6900, 6960 and 7020 the control has the lines and the new movie does not;
  - after the control's lines leave, the new movie is no brighter than the control by more than 2 points.
- **Why 2086–2100.** In the base movie, the hole name "g is undefined at a" fades over frames 2088–2099, measured from its pixels. Repair310 puts the lines into that same 0.2 s play.
- **What it writes.** `zoom-probe.json`, with the per-frame counts for both movies and the record, plus the probe record in `sampling.json`, all before anything is asserted.
- **If it fails.** The pictures, sheets, records and `summary.json` are kept, and then the job fails with the probe's findings.

## Input closure (root's review313 finding, fixed)

**What was missing.** Root found that `launch-review-repair313.py` checked the hash of `review-repair-build268-inputs.json` but never expanded its rows. Launch268 had bound the timeline21 rows and the production-repair19 workspace, so those dependency bindings were dropped. That is correct, and it is fixed in the hashes above.

**The fix.**
- **The build launcher.** It now re-verifies every row of both prior jobs' bound manifests, and carries each manifest and every row into `review-repair-build313-inputs.json`:
  - build268's `review-repair-build268-inputs.json`, which must equal its receipt's `inputs_sha256`;
  - rebind269's `review-repair-rebind269-inputs.json`, likewise.
- **No exceptions.** I know of no mutated row: rebind269, timeline269 and preview274 re-verified build268's rows when they launched. A changed row stops the launch before any manifest or job exists, and names the manifest and the path. If root finds a row that really was mutated, an exception should be root's explicit, justified decision in a new version, not a loosened check here.
- **The preview launcher.** It now does the same for preview274's own bound inputs (`review-repair-preview274-inputs.json`), since preview313 uses preview274's movie as its control.
- **Downstream.** The controller already expands every prior manifest's rows, as after-build269 did. Rebind313 therefore carries build313's closure, timeline313 carries rebind313's and timeline269's, and preview313 carries timeline313's.

CC1 is to check this independently in 314.

## What I ran (local only, in session scratch space outside the repository)

All of these run the actual helper code. The mocks substitute only:
- the root path;
- PATH, to reach the local ffmpeg;
- the interpreter path;
- a no-op CPU affinity on macOS.

The build mock also replaces the two base pins that name build-host bytes with its own fixtures' hashes. The checks:

- **Site tables.** `check_sources` from the 313 worker accepts `boundary-guide-repair310.rs` and refuses build268's `boundary.rs`. Cc3's 274 table refuses 310 at E2 line 1398 (`all.extend(poles);`), confirming review312 L1 that the old pins fail.
- **Hold relocation.** I used the 64 of build268's 70 files that exist locally, by hash; the 6 missing ones are `Cargo.*`, `storyboard.md` and three `.orig` files, none of which hold markers.
  - On the 268 tree, `relocate_map` reproduces rebind269's 28 recorded marker checks exactly.
  - On the 313 tree, rebind313's comparison passes: 20 holds and 5 helpers unchanged, and the boundary holds at 942, 1007 and 1286.
- **Probe on the control** (`dba964d2…`). The lines are present 1707–7988 with no gaps, and all 21 points are lit at every anchor. Decoding 6,901 frames took about 4 s.
- **Probe on synthetic variants of the control.** The lines are masked with the background colour (`0c1114`) by ffmpeg `drawbox`. These are stand-ins, not the repaired movie.
  - **A** (removed at 2093): passes, exit at 2092.
  - **F** (smooth fade over 2088–2099, as the real play should be): passes, exit at 2094.
  - **B** (removed at 1900): fails, early exit and not held.
  - **C** (reappears at 5001): fails.
  - **The control itself as "new":** fails.
- **Preview worker end to end** (`main()`), with a fake binary that writes preview274's 09 tables (path renamed) and a movie.
  - With full-length F: it completes. 313 samples on 40 sheets, exit 2094, E1 and E2 placed exactly, and E2 ends at the 16,504-frame total.
  - With the control: every picture and record is kept, then it fails on the probe.
- **Build worker base and reconstruction.** On a synthetic 70-file tree, the good case rebuilds 70 files, only `boundary.rs` changes, and `narration_units.json` equals build268's bytes. Each of these is refused:
  - an extra file;
  - units not rebound;
  - a changed backup;
  - a changed file;
  - a changed `boundary.rs`;
  - an existing target;
  - the wrong candidate;
  - a recipe not reviewed by root;
  - a recipe with an extra file;
  - a changed review;
  - a failed rebind269 receipt.
- **All three launch paths up to a fake guard** that re-hashes every row.
  - Both launchers write their manifests, carry the bound rows and pass the expected guard commands.
  - A mutated build268 row or rebind269 row stops the build launch with no manifest written.
  - Second launches and a wrong worker pin are refused.
  - The controller launches rebind313 with `--run`, then timeline313. It freezes rebind269's narration, leaves out the to-be-rewritten `narration_units.json`, and refuses a second run.
- **Compile and scan.** All 8 files compile, and the public-hygiene word scan is clean.

**How the edits were made.** I made `review-repair-preview313.py` as a byte copy of cc3's file, then applied exact, single-match replacements with a short script; the diff against the base is the list above. A few early wording and pin edits to my own new files were made the same way. All later edits used the editor.

## For root

**1. Review314 by CC1.** It covers these 8 files, including the input closure.

**2. Recipe313.** Root writes `review-repair-recipe313.json` after 314. The worker and launcher require exactly these keys; other keys, such as `time` or limits, are allowed:

```json
{
  "reviewed_by_root": true,
  "scope": "chapter 09 zoom-line repair",
  "files": {"boundary.rs": {"candidate": "boundary-guide-repair310.rs",
                            "sha256": "bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53"}},
  "source_review": "cc1-boundary-review312.md",
  "source_review_sha256": "15ae2d9a331e0a638806ff3c819d6511de9c7c6e59923e57221149d21d5a0fe4",
  "helper_review": "<CC1's review314 file in production-repair17>",
  "helper_review_sha256": "<its sha256>"
}
```

**3. Transfer to the build host's `qualification01/production-repair17`.**
- the 8 files above;
- `boundary-guide-repair310.rs`;
- `cc1-boundary-review312.md`;
- the review314 file;
- recipe313.

`prepare-assembly19.py` and the 268/269/274 inputs manifests are already there, because those jobs bound them.

**4. Order.** Each guarded job takes the lane lock, so they run one at a time.
- **Build:** `/usr/bin/python3 -I -B …/launch-review-repair313.py` (build268 took 63 s).
- **Rebind and timeline:** `/usr/bin/python3 -I -B …/review-repair-after-build313.py`. It can be started beside the build launcher, since it waits up to 12 h for build313's receipt. Then rebind313 runs (about 10 s), then timeline313: about 12 min, estimated as 09's share of timeline269's 94 min.
- **Preview:** `/usr/bin/python3 -I -B …/preview-after-timeline313.py --review <review of the preview worker> --review-sha256 <hash>`. That review is 314 if it covers the preview worker. The preview takes about 45 min, estimated as 09's share (16,504 of 81,032 frames) of preview274's 3 h 40 min, plus the chroma pass and the probe.

**5. What to read afterwards.** `jobs/csd-review-repair-preview313/09_boundary/`:
- `zoom-probe.json`;
- `sampling.json`, with every sample's reasons;
- about 40 `sheet-*.jpg`;
- `summary.json`.

A probe pass is pixel arithmetic on 21 points, not picture acceptance. The sheets still have to be looked at.

## Not established

- **Nothing has run on the build host.** Repair310 is still uncompiled. Build313 is its first compile, test, Clippy and render.
- **`cargo fmt --check` uses the build host's pinned rustfmt.** Locally, 310 passed a different rustfmt's `--check` with the same `max_width` 120. A format difference fails build313, closed; it does not rewrite the file.
- **The exit window comes from the base movie's hole-name fade.** The repaired lines' own fade is predicted from source (the same play), not observed.
- **The probe checks only 21 points.** The column itself, where the old crossings were, is covered by the extracted pictures, not by the probe. The probe proves the lines are gone where it looks. The pictures show whether the column is clear.
- **The other 14 chapters are not compared under binary 313** (see step 3).
- **No acceptance of any kind.** No motion, 4K, fine-sync or AV acceptance; no audio heard or checked; no picture acceptance.
- **Not independent.** I wrote repair310 and these helpers.
