# cc0 final287 changes: worker and launcher, prepared only

## Scope

- **What this is.** Bounded fixes for review279's M1, L2 and L3, written as new files. 278 and 244 are preserved byte-unchanged.
- **What I did not do.** No deployment, transfer, launch, render, build, permit, motion acceptance, Beads, TODO or canonical edit.
- **Narration.** Unchanged: the rebind269 folder, whose 105 WAVs are byte-identical to assembly19's, ElevenLabs Frederick. No audio heard or checked.

| File | sha256 | State |
|---|---|---|
| `csd-reviewed-final287.py` | `f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510` | new |
| `csd-launch-final287.py` | `80d9ab1159251ba372d2e7199761d2d6fba20113ea7e3c69e9f2bb1217597bba` | new |
| `csd-reviewed-final278.py` | `7d1dc9b203660f294353cff221c2db3822c54dc669464eeb5882431fb0f1f2ee` | unchanged |
| `csd-launch-final278.py` | `67372165b8166e537addc62fe33ad35cb50aafe10c18662374ad55fb070e3f62` | unchanged |
| `csd-reviewed-final244.py` / `csd-launch-final244.py` | `03dc1c92…` / `72224cca…` | unchanged |

## Changes against 278

1. **Preview tables (M1).** `preview_binding()` checks the reviewed preview's `<chapter>.timeline/cues/cuelog.tsv`. Each must equal timeline269's table byte for byte after one replacement: the preview's named narration folder becomes rebind269's. For preview243 that folder is `csd-review-repair-rebind242/narration`. Any other difference fails.
2. **Preview binary and source (M1).** The preview job's receipt must succeed, and its command must name the pinned preview worker (`review-repair-preview243.py`, `3e5dd06c…`). Its inputs manifest must hash to the receipt's `inputs_sha256`.
   - **What the manifest must list, at current hashes:**
     - the preview binary;
     - that worker;
     - the preview build's `source-copy.json` and `qualification.json`.
   - **Binary.** Its hash must equal the pinned value and the build's qualification.
   - **Scene file.** The chapter's scene file hash must be equal in the preview build's and build268's source copies.
   - **For 06/08,** the binary is `review-repair238` (`f31690e6…`) and the build is build238, with source copy `6d4c7e93…`. Build268's source copy is pinned to `81d28272…`.
3. **Named 238→268 delta (M1).** For a preview built before 268, the permit's `source_binding_evidence` must hold exactly one item named `build238-to-build268-delta`. Its path must be `qualification01/production-repair17/cc1-preview-to-final-binding282.md` at `1fbcedef…`.
   - **What the worker computes.** The files that differ between the two pinned source copies must be exactly binding282's set: `draw.rs`, `gallery.rs`, `mod.rs` and `rigidity.rs` changed, and `plate_fade_probe.rs` added.
   - **What 282 is not.** It is source-level carry-forward only. It is not 540p pixel, audio or motion equivalence. Its schedule half stays conditional on timeline269's actual tables, which `check()` (all 45) and item 1 assert.
4. **Permit path (L3).**
   - The launcher resolves the permit and its worker path (`strict=True`) before admission, freezes them, and passes them on resolved.
   - `check()` refuses a permit path that is relative or goes through a symlink.
5. **Earlier jobs (L2).** `check()` refuses to run if `csd-reviewed-final-<chapter>244` or `…278` exists. It runs in the launcher before admission and in the worker before and after the render.
6. **preview274.** The other nine chapters fail closed with an "unconfirmed" message (`PREVIEW274 = None`). No CC3 file names preview274's job, manifest, worker, table folder or narration, and I did not invent them (see the handoff).

**What is unchanged from 278:**
- the all-11 choice and `CHAPTERS`;
- the all-45 baseline normalization;
- the receipts of stop13, timeline21, score21, build268, rebind269 and timeline269;
- the score summary, files and QA;
- the binary and qualification;
- every permit field check, including `actually_reviewed_pictures` and `actually_reviewed_motion` being `True` and `unresolved_defects == []`;
- the evidence hashes, the timeline header and frames, and the motion-interval range;
- the job-name pattern (now `287`), the admitted receipt, guard policy 8/4 and the frozen inputs before and after;
- the render command, `verify_tables`, ffprobe, x264 CRF 16, the soundtrack and the exclusive outputs.

The diff against 278 touches only the lines that implement items 1–6.

## Checks run (local, read-only)

- **Syntax.** Both files pass `compile()` under Python 3.15.0 with `-I -B`; no bytecode was written.
- **Pinned values against local copies.**

  | Pinned value | Local copy |
  |---|---|
  | build268 source copy `81d28272…` | `review-build268-terminal272/source-copy.json` |
  | preview worker `3e5dd06c…` | `review-repair-preview243.py` |
  | binding282 `1fbcedef…` | `cc1-preview-to-final-binding282.md` |
  | binary 238 `f31690e6…` | 244's `BINARY_SHA` and 282's build238 qualification |

  The build238 source copy `6d4c7e93…` comes from 282's read on csd; I have no local copy of it.
- **Names.** The registry keys and scene files in `SCENES` match production19 `mod.rs`, where 14 maps to `continuing.rs`.
- **Mock run of the new logic.** I ran it in a scratch mock outside the repo, with fake files and the pinned values patched to the mock's hashes, so the logic is exercised and not csd. All 17 cases gave the expected result.
  - **Passes:**
    - the 06 binding;
    - the same binding again after a restore;
    - an absolute real permit;
    - no earlier job present.
  - **Refused:**
    - a 274 chapter;
    - the wrong `preview_job`;
    - the delta item missing;
    - the delta item with the wrong hash;
    - a changed cues byte;
    - an unmapped narration folder;
    - a wrong binary hash;
    - an extra changed file;
    - a changed scene file;
    - an unpinned 268 source copy;
    - a relative permit;
    - a symlinked permit;
    - an existing 278 job.
- **Not run.** Neither script ran against csd or any real job, and the launcher's guard call was not exercised.

## Before any 287 launch (root)

- **Timeline269.** It must finish with exit 0. `check()` then enforces all 45 tables and the 06/08 preview tables against it.
- **Files on csd at their pinned hashes.** `csd-reviewed-final287.py`, `csd-launch-final287.py` and `cc1-preview-to-final-binding282.md` must be in `qualification01/production-repair17/`.
- **A root permit per chapter.** It must have absolute paths and `actually_reviewed_motion` from a real motion review; 270 and 271 are stills only. It needs `unresolved_defects == []` after root's disposition of the persisting Lows. The `source_binding_evidence` must include the named 282 item: `{"name": "build238-to-build268-delta", "path": ".../cc1-preview-to-final-binding282.md", "sha256": "1fbcedef…"}`.
- **The other nine chapters** need a later version once preview274's real names exist.

## Handoff: CC3 (preview274) and CC1 (CSS290)

**For the 274 entry, a later version needs these from CC3's actual helper and outputs:**
- the job name and inputs manifest name in `qualification01/production-repair17/`;
- the preview worker file name and its sha;
- the table folder relative to the job;
- the narration folder it rendered with.

**It also needs `build = build268`, `binary = review-repair268` and `binary_sha256 = 983954c5…`.**

**The gate requires of preview274:**
- **Its manifest** lists the binary, the worker and build268's `source-copy.json` and `qualification.json`. A freeze built from timeline269's manifest rows, like `review-repair-after-timeline243.py`, does this.
- **Its receipt command** names the worker.
- **Its tables** equal timeline269's after its named narration mapping, which is the identity if it rendered with rebind269.

**For CC1's CSS adaptation:**
- **Host-bound constants.** Everything in 287 is csd-bound through `ROOT`/`D`/`R`: the jobs, the qualification folder, the guard, the score and the narration.
- **Carry these invariants over unchanged:**
  - items 1–5 above;
  - every 278 gate listed above;
  - the pinned hashes of the binary, the source copies, the preview worker and 282.

## Independence and limits

- **Not independent.** I wrote review279 and now its fixes. I also wrote the 06 repair 231 that both builds carry. 287 needs a reviewer other than cc0.
- **Binding282 is CC1's work.** I relied on its csd hashes for the build238 source copy and on its static argument. I re-derived neither from csd.
- **What 287 does not establish.** It adds no 4K, pixel, motion, audio or AV acceptance. Its mock run is a logic exercise, not a run against csd.
