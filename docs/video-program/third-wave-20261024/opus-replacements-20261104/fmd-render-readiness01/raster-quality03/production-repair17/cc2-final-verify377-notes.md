# 377: successor to verifier363 for review 367 (CC2)

cc2, 2026-11-08, frozen at about 22:24 UTC. Q is this folder locally, or `D/qualification01/production-repair17` on CSD, where D is the CSD job root.

Nothing has run on any host: no launch, permit, render, build or commit. `csd-final-verify363.py` stays frozen and unchanged. 377 is a separate file, made by copying 363 byte for byte and then editing it by hand.

## Files

| File | sha256 | State |
|---|---|---|
| `csd-final-verify377.py` (this task) | `827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe` | frozen; 1219 lines; compiles |
| `csd-final-verify363.py` (base) | `b4f687a58bc7233341e2b19fcfcf9b886faf31217d255d68073e62c9ecf09c1a` | frozen, unchanged |
| `cc2-final-verify363-notes.md` | `90f08901434408d4aa49bd4b8ead8d9aeffd276a973cbccb6b53e1a7e0186272` | unchanged |
| `cc0-verifier363-review367.md` | `f577a1b36768ac3b59caaec6f96b0790398a683a2525c6d70a72427bc2b62314` | read; pinned via root 377 |
| `root-verifier367-disposition377.json` | `ecc64557495d90d7762291df7e22284a22f8b98969eb58ad5175a95bc48537ff` | root's; pinned |
| `derivative-sheets359-review.json` | `40e2fca805e0dfb7b138b2ef68c2770b947bd7ae5e435e3d712bf2ea476d34de` | root's; pinned |
| `final-boundary-permit351.json` | `873c33bde99650c7f471c690852dff8860c03ef44b9ff8c695ce57788ba7b419` | read only; 377 reads its scope through 350's row |
| `06_one_derivative-csd-permit375.json` / `08_loop_sums-csd-permit375.json` | `c43801cc…` / `c2d0b172…` | read only, to confirm the required windows are inside their intervals |

## 1. The direction 377 follows

- **Root's direction, as relayed by the user in this session.** Root read the whole of review 367 and the 363 notes and asked for a minimal preserved successor:
  - M1: carry the actual `playback_seconds`; sample `round(s*60)` ±2 with named reasons and a native at the centre;
  - L1: bind 359 explicitly, require its 06 windows, and make sure the 08 run 10200–10380 is covered;
  - L2: add native 14 1140 and 11 2010, 4050, 2280, and sample the observation endpoints 1080/1200, 1860/2160 and 3840/4260.
  It also said: keep every earlier gate, keep 363 frozen, launch nothing, and use only small or in-memory fixtures because disk is tight.
- **Root's record.** `root-verifier367-disposition377.json` (`ecc64557…`) holds the same selection:
  - it names review 367 by path and hash;
  - `required_native_frames`: 11: 2010, 2280, 4050, 4348, 4408, 7448, 8121, 8303, 9731, 11880; 12: 2532, 2766, 3002; 14: 1140, 10260, 10278, 10301;
  - `additional_overview_endpoints`: 11: 1860, 2160, 3840, 4260; 14: 1080, 1200;
  - three requirements, and four `*_acceptance` flags, all False.
- **What that record is, and is not.** It is root's decision about which frames the final pictures must include. It records no viewing, and 377 claims none. It replaces the unpinned "named by root after CC1's review 357" basis that 363 used for 11/12/14, so review 367 L2's provenance gap is closed by a pinned root record.
- **The observations behind the new frames are CC1's** (`cc1-chapter-picture-review357.json`):
  - O9 (14, 1080–1200): the footer chip's top edge meets the plate edge;
  - O12 (11, 1860–2160): the footer's top edge meets the plate edge;
  - O13 (11, 3840–4260): an inset chip over the output panel's border corner;
  - O17 (11, 2280): three chips stacked, the third on the axis.
  357 classes all four as low and reports no collision.

## 2. What changed from 363

AST delta over the top level:
- **Added:** `DISPOSITION377`, `REVIEW367`, `FINAL_ENDPOINT_CHECKS`, `REQUIRED_WINDOWS`.
- **Changed:**
  - constants: `FINAL_FRAME_SOURCES`, `FINAL_FRAME_CHECKS`, `VERSION` (`'377'`), `VERIFY`, `VERIFY_INPUTS`, `PICTURES`, `PICTURES_INPUTS`. The job names are now `csd-final-verify377` and `csd-final-pictures377-{overview,native}`, with `Q/final-*377*-inputs.json`;
  - functions: `master350`, `picture_plan`, `plan_for`, `pictures` (three more `sampling.json` fields), `main` (message only), and the docstring.
- **Unchanged in AST:** every other function, including `stable`, `success`, `schedules`, `delivery347`, `film`, `verify`, `verified`, `part_done`, `launch`, `runtime`, `pillow_runtime` and `frame_hashes`, and every other constant, including `PINS`, `TRANSITION_WINDOWS`, `TRANSITION_SOURCES`, `ONE_POINT_INSTANTS`, the caption pins and `WORKERS`. So the decode, framemd5, loudness, native caption and 347 delivery checks are 363's, unchanged, as are the integrity and resource gates and the overview ledger handling.

The changes, and why:

1. **M1: `master350()` carries the playback instants.** For each sampled_preview row, `review_scope.playback_seconds` must be a list of numbers whose centre `round(s*60)` and 2 frames either side lie inside the chapter, or 377 refuses. The list is returned as `playback`. The 352 rows still must have `[]`, as in 363. Only 09 has instants: permit 351's four, 27.687, 31.901, 35.512 and 39.123 s, whose centres are 1661, 1914, 2131 and 2347.
   - **`picture_plan()`:** each instant's centre ±2 is a sample ("permit playback instant S s ±d (job review_scope)"), and the centre is a native. This is 339's 07 rule, `ONE_POINT_INSTANTS`, applied from the permit's own record.
   - **Rounding:** Python `round`; none of the four values is near a half frame.
2. **L1: `REQUIRED_WINDOWS`.** These are 359's two 06 windows, 9630–9730 and 9970–10029, and the 08 run 10200–10380, citing root 377, which retains it.
   - `plan_for()` checks that 359 names chapter 06, the preview243 movie `5e45d72d` and exactly these `viewed_intervals`, and that root 377's requirements retain `10200-10380`.
   - Each window must lie inside one of the sampled intervals its final rests on (06 and 08 rows); 377 refuses otherwise.
   - `picture_plan()` samples every frame of each window ("required window a-b") and adds a middle native to any window without one.
   - Root's real permits 375 hold exactly 359's windows for 06 and include 10200–10380 for 08, so this is a cross-check and adds no new frames on the real plan. Those frames' reason lists now also name 359 or root 377; the sheet label still shows the first reason, the interval's.
3. **L2: `FINAL_FRAME_CHECKS` for 11/12/14 now cite root 377,** with the four added natives. `plan_for()` requires their union per chapter to equal root 377's `required_native_frames`. 06's entry is unchanged: it comes from root 362, still checked against 362's `required_final_checks`.
   - `FINAL_ENDPOINT_CHECKS` (overview samples only) must equal root 377's `additional_overview_endpoints`.
   - root 377 must name review 367 at `f577a1b3`, and its `*_acceptance` flags must be False, as 362's and 357's are.
   - 357 stays pinned, and its chapter set is still checked.
4. **`pictures()`.** `sampling.json` also records `final_endpoint_checks`, `required_windows` and `playback_seconds`, so the reasons can be traced from the job's own record.

## 3. Root's staging on CSD

363's list (its notes §6) still applies under 377's names: copy `csd-final-verify377.py`, and add `root-verifier367-disposition377.json` and `derivative-sheets359-review.json`, as real files at their pins.

CC0 reported, from a read-only hash pass on CSD at 22:17Z, which I have not checked myself:
- root 377, 359, 362, 357, 354 and the 347 SRT/VTT are already in CSD Q at their hashes;
- two of the 339/363 `TRANSITION_SOURCES` are missing there: `cc1-gallery-transition-review319.md` (`d0df7480…`) and `open-region-playback-second305.json` (`8ede28c4…`). Both are in local Q at their pins. `plan_for` refuses until they are staged.

## 4. Run order

As 363 notes §5, with 377's job names: `check`; `launch-verify` and the verify job `csd-final-verify377`; `launch-pictures` overview, then root's ledger and a read-only re-hash; a re-hash just before the native launch; `launch-pictures` native, then the ledger compared with the printed hash, the native inputs row and `union.overview_sampling_sha256`; a re-hash of every recorded file before viewing.

I know of no launch of 363's jobs. If one exists, 377's job and input names differ, so the two do not collide.

## 5. Tests (scratch, outside the repo; local only)

| File | sha256 | Result |
|---|---|---|
| `harness377.py` | `6616bd981dd97663ca2e0a06d8b738a4b347cf7474dbe6400c76bdb64d359e8c` | **31/31**, 6 s; output `96543c8c…` |

**Method.**
- The harness runs 377 in `check` mode (which writes nothing) and calls its own `film()` and `plan_for()` on the 363 fixture tree that the 154/154 run left in place, reusing that fixture's fakes.
- It wrote about 100 KB in all: 377's text with the CSD roots swapped, copies (not links) of root 377 and 359, and small JSON edits that it restores. It checks that the six edited files end at their starting hashes.
- No movie, picture or WAV was written.

**Results.**
- 377 passes `check` with the fixture's stand-in 09 instant (34.73 s, centre 2084) and with permit 351's four real instants carried through the 09 row and its render summary. The centres are 1661, 1914, 2131 and 2347, each ±2 a sample with its instant named, and a native at each.
- All 24 required native frames (06's seven; 11's ten, 12's three and 14's four from root 377) are natives and samples. The four new natives cite root 377. The six endpoints are samples.
- 359's windows and the 08 run are sampled on all 342 frames, with a native inside each.
- Root 377 and 359 are bound inputs of the picture jobs, and the new `sampling.json` fields serialize.
- 15 refusals:
  - an instant beyond the chapter, at 0.01 s, or given as text;
  - a 352 row with an instant;
  - 06 intervals without 359's window; 08 intervals without the run;
  - root 377 or 359 changed;
  - root 377 claiming an acceptance, or without 14 1140 (pins moved with each);
  - code without 11 2010 or without endpoint 4260; a 359 window that 359 does not name; an 08 window wider than its interval;
  - root 377 naming another review.
  `check` passes again after every restore.
- **Counts are the fixture's,** on stand-in layouts and on the 352 notes' intervals for the ten rows. With permit 351's instants they are 8,674 samples and 126 natives, and the parts come to 1.64 and 2.98 GiB of 3.5. CC0's replay with the real permits estimated about 127 of 148 natives. The real counts come only from `check` on CSD.
- **Not rerun:** 363's 154-case suite. 377's other functions are AST-identical to 363's, and the harness's `check` runs every static gate on the same tree.

## 6. Not claimed

- No picture, motion, listening, fine-sync, caption-playback, 4K caption-legibility or independent AV acceptance. Every such flag the jobs write is False.
- No root viewing beyond what root's own records state.
- Nothing ran on CSD or CSS.
- Not publication authorization.

## 7. For CC0 (review 378)

- **Where the fixture's 05/06/08 intervals came from.** The 363 fixture's sampled intervals for the ten 352 rows came from my 352 notes §5, the intervals the 352 permits were then expected to carry. Root's real permits 364/366/370/375 now in Q carry exactly those. Only 09's two fixture intervals were stand-ins; permit 351 has [2080, 2109] and [2316, 2336].
- **Rounding.** `round(s*60)` is used, giving 1661, 1914, 2131 and 2347, as you planned to check.

## 8. Hygiene

- `python3 -I -B`, in-memory `compile()` only.
- `find` was not used.
- This task wrote only `csd-final-verify377.py` and these notes in the repo.
- No local user, temp or scratch paths in either 377 file.
