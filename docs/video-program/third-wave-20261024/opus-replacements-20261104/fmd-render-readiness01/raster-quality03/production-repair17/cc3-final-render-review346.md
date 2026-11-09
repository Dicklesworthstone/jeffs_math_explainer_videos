# cc3 final render review 346: worker 344, launcher 344 and permit 351

cc3, 2026-11-08, written at about 21:05 UTC. This is an independent, read-only review of:
- CC2's frozen 344 worker, launcher and notes;
- root's permit 351, for the technical 4K60 render of `09_boundary`.

Here Q is `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17` (locally), or `D/qualification01/production-repair17` on the build host, where D is the job root.

**What this is not.** It is a review of code, records and hashes. I launched nothing. I wrote nothing on the build host or in any source, candidate, permit, TODO or Beads file. It is not picture, motion, listening, fine-sync, caption, final-AV, 4K or publication acceptance, and it does not authorize a launch.

## 1. Answer

**No findings.** I found nothing High, Medium or Low in 344 or 351. There is one cosmetic nit in CC2's notes (N1, §8).

**No 317 gate is lost.** Proved by AST (§3–§4):
- every 317 gate is still in 344: source, tables, score, resource, encoder, evidence, binary, narration, preview binding and independent review;
- the only gates removed are `actually_reviewed_motion: true` and `motion_intervals`.

**The replacement is stricter and states what was actually reviewed** (§5):
- `review_scope` must say `sampled_preview` / `technical_4k_render`.
- Every frame of a "sampled interval" must be one of preview313's own sampled pictures.
- `playback_seconds` must be exactly root 343's observed instants.
- Six acceptance flags must be stated `false`.
- Any key at any depth naming an acceptance, release or publication that is not exactly `false` is refused.
- Final picture, motion and audio must be listed as open.
- The four review records 340, 341, 343 and 345 are pinned by hash and frozen into the inputs.

**Permit 351 passes the new gate.** I ran it with its host paths mapped to the local copies (§6). Every hash it binds equals the build host's bytes as I read them at 20:55 UTC (§7).

**What stays open, in three places.** In the permit, `open_reviews` lists final_picture, motion, audio, fine_sync, captions, independent_final_av and publication, and all six flags are false. 344 records `review_scope` in `render-summary.json`, with a scope sentence that leaves final picture, motion, audio, decode, master replay, captions and publication open.

**Disposition.** On this review, 344 with permit 351 is fit for the one technical 4K60 render of `09_boundary` once root stages them on the build host. Root must stage the worker, launcher and permit at the hashes in §2 and pass those hashes at launch. This is not a launch, and not release or publication authorization.

## 2. What I reviewed

| File (Q) | sha256 | State |
|---|---|---|
| `csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | 354 lines, as CC2's notes state |
| `csd-launch-boundary-final344.py` | `70278078c8fd5e1cb414afcf02c3ff107a9dd956f0b2551a1ca0bdca360d75b4` | 110 lines, as the notes state |
| `cc2-boundary-final344-notes.md` | `86da6e2319f2f8adafcc56a2d2f27cd3af51d94a041e8c04077810c3ec296d01` | read in full |
| `final-boundary-permit351.json` | `873c33bde99650c7f471c690852dff8860c03ef44b9ff8c695ce57788ba7b419` | read in full |
| `cc3-preview313-pictures340.md` | `7a0c1f9fd38a62c437b66cf1d356ef3d479008070c9ff9af06a51e1d3b6b9c14` | my own; equals 344's pin |
| `root-preview313-pictures341.json` | `7350b30a4bcc595a1fffd31c7670c18c97ef4dac00045f46ec916c225132dca9` | read in full; equals the pin |
| `root-preview313-playback343.json` | `d9f250ec00007fe0687c7bf2a3155652b30df45956c5dfe53e9922dd0ec67e81` | read in full; equals the pin |
| `root-preview313-disposition345.json` | `cf42fd8ebd43d4ae30fc24dee9826fcc630d7d9f50aeba9a2290bee489a4f08e` | read in full; equals the pin |

The following files are preserved unchanged; I hashed each before writing:
- the 317 worker `19abeee9…`;
- the 317 launcher `47e0f30a…`;
- the 317 notes `08be95be…`;
- `cc0-combined-master318.py` `478b44a5…`.

The 317 hashes equal the values I recorded at 20:41 UTC, before 344 appeared. The 344 worker and launcher were unchanged for at least two minutes before I read them. Their hashes equal the hashes in CC2's notes.

## 3. The diff against 317, by AST

I compared the files by AST, with docstrings set aside.

**Worker functions.**
- Removed: none.
- Added: `sampled_review_scope` and `disposition`.
- AST-identical to 317: `sha`, `read`, `success`, `job_name`, `absolute_permit`, `completed`, `build313`, `narration313`, `schedule313`, `independent` and `verify_tables`.

**Changes in `check()`.** The 317 version has 42 statements and the 344 version 41.

| Removed (317) | Added (344) |
|---|---|
| `assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True` | `assert set(review) == PERMIT_KEYS, …`, placed directly after `review=read(permit)` |
| `assert review['evidence'] and review['motion_intervals']` | `assert review['actually_reviewed_pictures'] is True` |
| `for a,b in review['motion_intervals']: assert 0 <= a < b < frames` | `assert review['evidence']` |

**Changes in `preview_binding()`.** The 317 version has 23 statements and the 344 version 28.

| Removed (317) | Added (344) |
|---|---|
| the motion-interval exit assertion | `sampled` frames from preview313's own `sampling.json`, refusing duplicate frames |
| | `sampled_review_scope(...)` |
| | the same exit assertion, made against the returned sampled intervals |
| | `disposition(...)` |
| | the four record paths appended to the returned binding |

**Changes in `run()`.** Both versions have 63 statements. Only the `result = dict(...)` statement differs:
- it keeps every 317 key;
- it adds `review_scope` and `disposition`;
- it has a new `scope` sentence.

**Worker constants.**
- `VERSION` goes from `317` to `344`.
- `EARLIER` gains `'317'`, so an existing 317 09 job is refused.
- New: `INDEPENDENT340`, `PICTURES341`, `PLAYBACK343`, `DISPOSITION345`, `RECORDS` and `PERMIT_KEYS`.

**Launcher.**
- `digest` is identical to 317's.
- In `main` (56 statements in each), exactly two statements change: the worker file name `…final317.py` → `…final344.py`, and `worker.VERSION == '317'` → `'344'`.
- Everything else matches: the guard command, the single native lane, 16 CPUs, `'x'` inputs and the input closure. The closure also takes in the four records, because they are now part of `binding`.

**`sampled_review_scope` uses builtins only.** Its free names are `all`, `any`, `dict`, `enumerate`, `float`, `int`, `len`, `list`, `range`, `set`, `str`, `tuple` and `type`. A later master can therefore copy it, and prove the copy by AST, as CC2 intends.

## 4. Gate-by-gate preservation

| 317 gate family | Where, in 344 | Kept |
|---|---|---|
| Source: build313 = build268 + exactly repair310's `boundary.rs`, tests/Clippy/fmt, workspace hashes | `build313()` | AST-identical |
| Narration: 105 WAVs byte-identical across rebind313, rebind269 and assembly19; only the three metadata files differ | `narration313()` | AST-identical |
| Tables: timeline313's 09 tables equal timeline269's and timeline21's under the named mappings; output tables after the render | `schedule313()`, `verify_tables()`, and the all-chapter timeline269/21 loop in `check()` | identical / unchanged statements |
| Score: score21 summary, every stem file, stems QA, parts offsets | `check()`, `verify_tables()` | unchanged statements |
| Resources: successful receipts (build313, rebind313, timeline313, preview313, encoder-stop13, timeline21, score21, build268, rebind269, timeline269); admission; `stop_free_gib 8`; `phase_growth_gib 4`; inputs hash; unique rows; every row re-hashed before and after | `completed()`, `success()`, `run()` | identical / unchanged statements |
| Encoder: 3840×2160, `60/1`, `nb_frames`, h264/yuv420p, aac/48000/2, x264 `rc=crf` and `crf=16.0`, last soundtrack capture, one movie | `run()` | unchanged statements |
| Evidence: every `evidence` and `source_binding_evidence` item at hash; preview313 `sampling.json` and `zoom-probe.json` bound; exit frame covered | `check()`, `preview_binding()` | kept; the exit rule now runs on sampled intervals |
| Binary: from build313's qualification, equal to `sha(BIN)`, to the permit and to preview313's summary | `build313()`, `check()`, `preview_binding()`, launcher | unchanged |
| Independent review: a reviewer other than the permit's; files in Q at hash; required in the inputs | `independent()`, `run()` | AST-identical; 340 also pinned by `disposition()` |
| Earlier jobs: 244/278/287/303 refused | `EARLIER` | kept, plus 317 |

## 5. The new review gate, exercised

I tested the frozen worker by importing it; its top level only defines constants and functions. I pointed its `Q` at the local Q and called only `sampled_review_scope` and `disposition`. I never called `check()`, `run()` or the launcher. Inputs were the real local `sampling.json` and movie, the four real records, and a permit shaped like 351.

**Positive case.** It returns the intervals `[2080, 2109]` and `[2316, 2336]` and the four record paths.

**Refusals (23 of 23).** Two of these mirror one-line checks that live in `preview_binding()` and `check()` rather than in the two functions. The positive case and 21 refusals came from the two functions themselves.

- **Motion fields:** `actually_reviewed_motion` present; `motion_intervals` present.
- **Acceptance flags and claims:**
  - `continuous_motion_acceptance: true`;
  - `release_acceptance` missing;
  - `publication_acceptance: null`;
  - a nested `evidence.0.accepted: true`;
  - a nested `review_scope.released`.
- **`review_scope` contents:**
  - `open_reviews` without `audio`;
  - `open_reviews` with a duplicate;
  - basis `continuous_playback`;
  - purpose `release`;
  - another movie hash;
  - an extra scope field.
- **Sampled intervals:**
  - an interval reaching the unsampled frame 2110;
  - a single-frame interval;
  - unsorted intervals;
  - a tuple pair.
- **Playback seconds:** rounded instead of 343's exact values; a bool value.
- **Record binding:** 341 missing from `evidence`; 340 moved from `independent_review` to `evidence`.
- **Mirrored lines:** no interval covering 2094; an extra permit key.

**The `disposition()` checks hold on the real records:**
- 340 names the movie `5e50811b…` and `sampling.json` `4e7ab1d8…`;
- 341 viewed 313 pictures on sheets 1–40, with `concrete_picture_defects_observed: []`;
- 343 and 341 name that movie;
- 345 read the independent review completely, has the exact technical-only scope, and cites 340, 341 and 343 by hash;
- every `*accept*` key in 341, 343 and 345 is `false`.

**Which sampled intervals are possible.** The only runs of consecutive sampled frames longer than three frames are 2080–2109 and 2316–2336. The others are pairs, plus 16501–16503. So the two intervals in 351 are the largest a permit can claim, and each consists solely of pictures that 340 and 341 viewed. The first contains the measured zoom exit frame, 2094.

**One limit of the scanner.** The acceptance scanner checks key names, not values. For example, `observations` is free text ("This authorizes technical rendering, not release."), and the scanner does not read it. That is acceptable, because the six flags carry the decision.

## 6. Permit 351, field by field

| Field | Value | Checked against |
|---|---|---|
| keys | exactly `PERMIT_KEYS` | worker constant |
| `chapter`, `host`, `preview_job`, `frames` | `09_boundary`, the build host's name, `csd-review-repair-preview313`, `16504` | worker constants (`CHAPTER`, `HOST`, `PREVIEW313.name`, `FRAMES`) |
| `binary_sha256` | `1a82f72c…11db` | host `review-repair313` binary; build313 qualification (324) |
| `timeline_sha256` | `5a91b8f9…a9e` | host timeline313 09 table; local preview table |
| `actually_reviewed_pictures`, `unresolved_defects` | `true`, `[]` | 340 and 341 viewed every sampled picture; 345 accepts L1/L2 for the technical render only |
| `reviewer` / `independent_reviewer` | `root` / `cc3 WhiteSwan` | different, as `independent()` requires |
| `evidence` | 341, 343, 345 at their pins; `sampling.json` `4e7ab1d8…`; `zoom-probe.json` `e2d05d81…` | host hashes (§7); `disposition()` |
| `source_binding_evidence` | build313 qualification `1e0116f7…`, source copy `cb4d402e…`, timeline313 comparison `e63cbc5e…`, rebind313 summary `359052a7…` | host hashes (§7); equal to my 324 reads |
| `independent_review` | Q `cc3-preview313-pictures340.md` `7a0c1f9f…` | host hash; the pin |
| `review_scope` | sampled_preview / technical_4k_render, movie `5e50811b…`, intervals `[2080, 2109]` and `[2316, 2336]`, 343's four instants, seven open reviews | passes `sampled_review_scope` (§5) |
| six flags | all `false` | passes |

The `observations` field is truthful against the records. It names:
- 341's 313 pictures;
- 340's confirmation;
- 343's instants only;
- L1/L2 disposed in 345, with a mandatory native-4K recheck;
- "not release".

## 7. Build host state (read-only, 20:55 UTC)

- **The four records** are in host Q at their exact pins, as regular files (not symlinks).
- **Not yet in host Q:** the 344 worker and launcher, permit 351, and `final-09_boundary-inputs344.json`. The 317 worker and launcher are not there either.
- **No `csd-reviewed-final-09-*` job exists**, so `EARLIER` refuses nothing today.
- **The binary and every bound source equal the permit's hashes:**
  - `D/work/bin/review-repair313` is `1a82f72c…`;
  - build313 `qualification.json` and `source-copy.json`;
  - timeline313 `comparison.json` and the 09 timeline table;
  - rebind313 `rebind-summary.json`;
  - preview313 `sampling.json` and `zoom-probe.json`.

## 8. Findings

**High 0, Medium 0, Low 0.**

### Nit

**N1 (notes only).** CC2's notes §2 say that in `render-summary.json` "every key 317 wrote is still written with the same value". The `scope` key's value did change, and the line just above in the notes says so. The code is as intended.

### Observations (not defects)

**O1. The L1/L2 native-4K rechecks are prose only.**
- They live in 345 and in the permit's `observations`. No code carries the frame list forward.
- At the final-picture step, view the 4K at:
  - the 340 §6(d) frames: 2088–2100, 2326, 4180, 5417, 6240, 6900–7020 and 7980–8040;
  - 345's L1/L2 frames: 5280, 5400, 5537 and 6228–6360.

**O2. Two pinned records contain local home paths.**
- Root's 343 and 345 contain local absolute home paths in their values. 341 contains none.
- 344 now pins both files, and both are staged on the build host. Editing them would need new pins in a successor.
- Whether to commit them as they are is root's call.

**O3. The master cannot take a 344 job yet.**
- Master 318 cannot take it unchanged. CC2 reports that 318's permit gate raises `KeyError: 'actually_reviewed_motion'` (notes F8).
- CC2 names the adaptation (notes §7), and CC0's 350 is to carry it.
- I did not review 318 or 350.

**O4. Launch arguments.**
- `--worker-sha256` must be `b8458dbc…`.
- `--permit-sha256` must be the hash of the host copy of 351: `873c33bd…` if copied byte for byte.
- 344 resolves the permit path strictly. The launcher requires 16 CPUs.

**O5. Guard growth cap.** Whether the render fits the guard's 4 GiB phase-growth cap is the same open question as for 317. 344 does not change it.

## 9. Not established

- **`check()`, `run()` and the launcher were not run** against the real host tree. That would need 344 staged, and it is root's launch.
- **My tests cover only the two new functions** and the permit's host-independent fields, on local copies with host paths mapped.
- **CC2's 70-case fixture suite** was not re-run or read.
- **No render, decode or 4K pixels**, and no acceptance of any kind: picture, motion, listening, fine-sync, caption, final AV or publication.
- **No review of downstream code:** master 318 and successor 350, the combined workers 303 and 309, and the guard.

## 10. Method and hygiene

- **Tools.** `python3 -I -B` with `ast`, `importlib` and `hashlib` on the local files. Plain `diff` for the textual delta.
- **Host access.** One read-only `ssh` read that hashed and listed files on the build host.
- **Scratch.** My test scripts stay in my session scratch area, not in the repo.
- **Writes.** This file is my only write in Q. Nothing was written on the build host.
- **Frozen files.** 340, 324, 320, 316, 315 and 308 were left as they were.
