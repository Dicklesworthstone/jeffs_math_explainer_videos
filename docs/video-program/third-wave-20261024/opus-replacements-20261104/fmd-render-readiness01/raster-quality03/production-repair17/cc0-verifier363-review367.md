# Review 367: CC2's frozen verifier 363

cc0, 2026-11-08. This is an independent review of CC2's frozen 363, the final verification and final picture extraction for master350. It covers the 339 → 363 delta and how 363 reads my frozen 350.

I edited no CC2, CC1, CC3 or root file. I did not edit 363, 339, 350 or any earlier review. I ran nothing on CSD or CSS, launched no guarded job, wrote no permit and made no Beads, TODO or canonical edit. Every local check ran in memory against the read-only files in this folder. No data fixture was written, because the local disk is near its stop level. **No audio heard or checked.**

D is the CSD root and Q is `D/qualification01/production-repair17` (locally, this folder). `:N` is a line of `csd-final-verify363.py` at `b4f687a5`.

## Files reviewed (frozen; hashes confirmed with CC2's handoff and on disk)

| File | sha256 |
|---|---|
| `Q/csd-final-verify363.py` (1135 lines) | `b4f687a58bc7233341e2b19fcfcf9b886faf31217d255d68073e62c9ecf09c1a` |
| `Q/cc2-final-verify363-notes.md` (192 lines) | `90f08901434408d4aa49bd4b8ead8d9aeffd276a973cbccb6b53e1a7e0186272` |
| `Q/csd-final-verify339.py`, the delta base, unchanged | `1e200220a1a7fd794abe318c0a1d815c188976f57c0707902c71915345dc05af` |
| `Q/cc0-combined-master350.py`, mine, the contract, unchanged | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` |
| `Q/root-preview06-disposition362.json`, root's | `1822a5908199c1cf8ae935e50c803e6a419c0109487c4051bef31d0ef0bf0b86` |
| `Q/cc1-chapter-picture-review357.json`, CC1's | `47d51709ed54205c053f9789d5aea194e282a27a8d63305b11d995aa00e71490` |
| `Q/derivative-sheets359-review.json`, root's (not pinned by 363) | `40e2fca805e0dfb7b138b2ef68c2770b947bd7ae5e435e3d712bf2ea476d34de` |
| `Q/root-preview313-playback343.json`, root's (not pinned by 363) | `d9f250ec…`, as permit 351 binds it |
| `Q/final-boundary-permit351.json`, root's 09 permit | `873c33bde996…` |
| `Q/root-caption347-review354.json`, root's | `749aeba8350d46e339a517af971086bd2d6e2fe039e6501f16e480edd3ad3cb4` |
| `Q/cc1-caption-repair347-checks.json`, CC1's | read, not pinned |
| `Q/root-master353-permit-hygiene369.json`, root's | `606b0c9fa8167c183539dd13906ed1876faa6504061ef33412205878ab9c889c` |
| permits 364 (01), 366 (04, 07, 10), 370 (11, 12, 14) | read for their `review_scope`; 370: `f6222de0…`, `689164d6…`, `778ca111…` |

## Result

**No High. One Medium (M1), two Low (L1, L2) and three notes.**

- **The 350 interface holds.** Every field 363 reads from master350 is one 350 writes, with the same names, kinds and job patterns (§1).
- **The required anchors are carried.** Root 362's seven 06 frames and the thirteen 11/12/14 frames are all both overview samples and native 4K frames, and every frame of every sampled interval is a sample with a native in each interval (§2).
- **Captions keep their distinction.** The native caption proof is 339's, unchanged. The 347 check is a separate record and claims no caption playback (§3).
- **The external overview ledger procedure is retained.** Notes §5 are my 339 §4 with L1 and N1. `part_done`, `launch` and the printed anchors are AST-identical to 339 (§4).
- **Nothing claims an acceptance.** Every review flag stays False (§4).
- **Findings.**
  - M1: 09's final rests on four playback instants that 363 never plans.
  - L1: root 359's 06 windows are claimed in the notes but are not in the code.
  - L2: root's 357 frame selection has no pinned root record, and four Low observations have no native frame.

## 1. What 363 changes, and how it reads 350

**AST delta (my `ast367.py`; docstrings set aside).**
- 78 top-level names are identical to 339's, among them `stable`, `success`, `part_done`, `launch`, `runtime`, `pillow_runtime`, `frame_hashes`, `growth_upper`, `part_upper`, `verified` and `PINS`.
- Changed:
  - the constants `VERSION`, `MASTER`, `MASTER_HELPER`, `MASTER_INPUTS`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES` and `PICTURES_INPUTS`;
  - the functions `film`, `main`, `picture_plan`, `pictures`, `plan_for`, `schedules` and `verify`.
- Added: `master350`, `delivery347` and 12 constants. Removed: `master318`, `FINAL317`.
- `preview313` differs only in its docstring. This matches notes §1.

**Against frozen 350 (`60575944`).**
- **Row fields.** 350's `chapter_gate` record writes `host`, `job`, `worker`, `worker_sha256`, `permit_sha256` and `**basis`. For a sampled row the basis is `review_basis`, `review_scope`, `sampled_intervals`, `sampled_frames`, `open_reviews` and `stated_false`; for a retained row it is `review_basis: historical_permit`. `master()` writes `version '350'`, `host 'csd'` and `review_bases`. 363 reads only these (:358–447).
- **Worker paths and jobs.**
  - `WORKERS` names the same three workers at the same hashes as 350's `WORKER344` and `WORKERS352`.
  - The job patterns `csd-reviewed-final-%s352` and `css-reviewed-final-%s352` (with the chapter dashed) are 350's.
  - 350 puts each row's worker into the master's bound files (`files = {worker, inputs, permit, module.BIN}`), so 363's `rows.get(str(worker)) == digest` can hold.
- **Interval rule.** 363 requires `0 <= a < b < frames` for every interval. 350's `sampled_scope` requires `end < a < b < frames`, sorted and disjoint. So no interval 350 accepts can be refused here.
- **Render summaries.** 350 asserts that every 344 and 352 render summary carries `permit_sha256` and the permit's `review_scope`. That is what 363 compares with the row (:425–435).

## 2. Anchors: an in-memory replay of 363's own plan

**Method.** `plan367.py` loads frozen 363 read-only and calls its own `picture_plan()` and `part_upper()`, writing nothing. The inputs are local stand-ins:
- plan182's chapter table, which 363 asserts equal to the master's;
- the real `review_scope` intervals of permits 351, 364, 366 and 370;
- for 06, root 359's two windows; for 08, the 10200–10380 run the docstring names; for 05, its 319 windows. Q holds no 05, 06 or 08 permit yet;
- the preview274, preview243 and preview313 layout tables, and the retained finals' tables. Cue starts are left empty, so layout anchors use their `@t` stamps only;
- the local preview313 copy.

**Result.**
- 8,055 samples and 113 natives: overview 1.52 GiB and native 2.68 GiB of 3.5 GiB. 148 natives would fit.
- CC2's fixture gave 8,659 and 118. The real counts come only from `check` on CSD.
- In my replay:
  - every frame of every interval is a sample, and every interval holds a native. Only 09's two intervals needed a middle native, and those middles, 2094 and 2326, are the preview313 exit native and the `ABSENT_ANCHORS` 2326 native that 339 already plans;
  - all 20 `FINAL_FRAME_CHECKS` frames are both samples and natives;
  - `plan_for` asserts that 362's `required_final_checks.frames` equal 363's 06 list, on preview243's movie `5e45d72d`;
  - both records must have every `*_acceptance` False. 362 has six such flags and 357 has six, all False on disk.

### M1 (Medium). 09's playback instants are never planned

**Where.** `master350()` lets the 09 row keep its `playback_seconds` (:434). `picture_plan()` then plans for 09's intervals only (:608–615).

**What the 09 final rests on.**
- Permit 351's `review_scope.playback_seconds` are root 343's four instants on the preview313 movie: 27.687, 31.901, 35.512 and 39.123 s.
- At 60 fps these are 09 frames 1661, 1914, 2131 and 2347.
- 339 handled the same kind of basis for 07: each of root 305's 07 instants gets ±2 frames (:602–604).

**What 363 plans for them** (my replay):
- 1661: sampled, but by plan182 with another reason (`09_1 provisional phrase byte`);
- 1914: not sampled (1913 is);
- 2131: not sampled (2129 and 2130 are);
- 2347: no sample within ±2.
- None is a native, and no sheet label names root 343.

**Why it matters.** These four instants are one of the two things the 09 final's permit rests on. Without them, root's final picture review does not show root the same moments in the final master's pixels, as it does for every 07 instant and every sampled interval.

**Recommendation** (CC2's successor; root decides).
- `master350()` also returns each sampled row's `playback_seconds`.
- `picture_plan()` adds `round(s*60)` ±2 as samples, labelled with the permit's job, and one native at `round(s*60)`.
- This is the 07 rule, applied to 09.
- Cost: 20 samples and 4 natives.

### L1 (Low). Root 359's 06 windows are in the notes, not in the code

**Where.**
- The docstring (:14) says "06: root's 359 consecutive frames".
- Notes §3 says the M1 transition windows include "root 359's consecutive frames" and that "06's intervals are root's 359 windows".

**What the code does.**
- `TRANSITION_WINDOWS` (:280) has no 06 window and no 08 window.
- Nothing pins `derivative-sheets359-review.json` (`40e2fca8`). Nothing asserts that its `viewed_intervals`, `[[9630, 9730], [9970, 10029]]`, are among 06's intervals.
- 06 is covered only if root's 06 permit, not yet in Q, names those windows as sampled intervals.
- 08's 10200–10380 run has the same status.

**Why it is Low.** Root writes the 06 permit. 350 requires every interval frame to be sampled in records the permit binds, so if 359 is bound its windows can be named.

**Recommendation.**
- Pin 359 in `FINAL_FRAME_SOURCES` and assert that 06's intervals contain its `viewed_intervals`, or add the two windows outright. The cost is 161 samples and at most 2 natives.
- Correct the notes §3 sentence. 359 is not one of 339's windows.

### L2 (Low). Root's 357 selection is not a pinned root record, and four Low observations have no native

**Provenance.**
- 363 names thirteen 11/12/14 frames "named by root after CC1's review 357" (:21–22, :152–165). It pins CC1's record and checks only its chapter set and flags.
- No root record at a hash holds that selection, as 362's `required_final_checks` does for 06.
- 357 itself asks for a 4K judgement only for O5.

**Coverage** (my replay). Four Low observations of the kind root 362 required in native 4K have no native frame anywhere in their range:
- O9: 14, frames 1080–1200, a footer chip meeting the plate edge;
- O12: 11, frames 1860–2160, a footer meeting the plate edge;
- O13: 11, frames 3840–4260, an inset chip over the panel-border corner;
- O17: 11, frame 2280, chips stacked on the axis.

Each is sampled at one end or both (O13 at 4260 only), at 540p.

357 states its own limit: "a defect one or two pixels wide at 4K may not show". The native PNGs are where a contact like these can be judged.

**Recommendation** (root's call).
- Root records its 11/12/14 selection as it did for 06, and 363's successor pins it.
- If root wants these four observations judged in native 4K too, add one native and both endpoints for each. That is 4 natives. With M1, the replay count becomes about 121 of 148.

### N1. The 357 and 362 frames are on preview movies

The 362 frames are on preview243's 06 movie, and the 357 frames on the preview274 movies. 363 uses them as chapter-local final frames. That holds because the frame counts are equal (350 asserts the permit's frames equal the chapter's), and both the preview and the final take their frame numbers from the same timeline. 363 does not assert that the 06 row's `review_scope.preview_movie_sha256` is `5e45d72d`. 350 checks that against 352's pinned movie, so this is a note only.

## 3. Captions: native proof and 347 delivery, kept apart

**The native proof is 339's.**
- The pinned `check_captions` runs on the master's own SRT/VTT (:531).
- The pinned screen runs on the master's SRT (:864–867).
- Both are unchanged, and the result keeps `captions` and the master's `srt_sha256`.

**`delivery347()` (:492–519) is separate.** It requires all of these:
- native327's receipt succeeds, and its SRT and VTT are at their pins;
- the master's native SRT and VTT are byte-equal to native327's;
- the 347 files are link-free and at their pins;
- root 354 cites `0ea9277e`, `6a383cb7` and `5d064232`. I checked this on disk.
- 354's `decision` starts `Adopt347 `, and all four of its `*_acceptance` fields are False;
- the same pinned `check_captions` passes on 347.

**The 347 screen.**
- The verify job runs the same `SCREEN` argv with the 347 path (:871).
- It refuses unless the count matches the exact check and both lists are empty.
- The keys it reads are the ones the native call already uses.

**The record.**
- The result sits under `delivery_captions_347`, with `caption_playback_reviewed=False`.
- `native_srt_and_vtt_equal_native327=True` is set only after the byte asserts.

**347 against the pinned checks (CC1's record, not my run).** `cc1-caption-repair347-checks.json` records:
- the pinned `check_captions_f266a231`: 579 blocks, SRT = VTT, text equal to the script "whole and per cue";
- `screen_3cc26436`: 0 over 20 characters per second, 0 lines over 42.

So the three merged answer captions stay inside their cues, and 363's 347 gate can pass. CC2's `real347_control.py` reports the same; I did not run it.

### N2. A delivery refusal also stops the native verification

`delivery347()` runs inside `film()` (:532). So any refusal there blocks `check`, `launch-verify` and the verify job, including the native proof. That covers native327 not matching the master, or a later root caption decision that replaces 354.

- **The equality is expected but not proven here.** CC3 346 records that timeline313's 09 tables equal timeline269's under the named mappings, and 350 checks that rebind313's 105 WAVs are byte-equal to rebind269's.
- **It refuses safely.** It refuses at `check`, before anything is spent.
- **The distinction survives in the records** either way.

Coupling is a design choice and root's. If root later changes the delivery candidate, 363 needs a successor even though the master is unchanged.

## 4. External ledger and no acceptance

- **The ledger procedure.**
  - Notes §5 steps 3–6 are my review 339 §4: copy the hash to a ledger outside the job folder at overview success; re-hash read-only before the native launch; compare the ledger with the printed value, the native inputs row and `union.overview_sampling_sha256`; re-hash every recorded file before viewing.
  - The comments at :1060–1063 and :1128 say the same.
  - `part_done`, `launch` and the printing in `main` are AST-identical to 339, apart from names and messages, so 339's tested behaviour carries over (my review 339 C0–C15).
- **No acceptance.**
  - `FLAGS` (:306) is unchanged, with every flag False.
  - The verify scope and the closing line say that pictures, motion, fine sync, caption playback, 4K caption legibility and independent AV acceptance remain root's.
  - 363 writes no flag that claims an audio, picture or AV review.

### N3. 350 F1 (CC3 353) stays, and root's per-permit check carries it

363 pins 350 at `60575944`, so CC3's F1 (an alias path can view its own record) stays in the master gate. Root 369 handles it operationally for permits 364 and 366 and says to recheck every further permit.

**My host-free check of permits 370 (11, 12, 14), and of 364 and 351.** This is `hygiene367.py`, on the evidence and independent_review items that 350's `frame_basis` reads:
- no item views its own sha256;
- no two items share a sha256;
- every path is absolute and lexically normal;
- every `views` entry names a bound item.

All pass. Hashing the bound files needs the host. The 05, 06 and 08 permits are not in Q yet.

## 5. Recommended order (root's)

1. Decide M1, L1 and L2. If any is taken, the successor is CC2's, under a new number. 363 stays frozen.
2. Stage CSD Q as notes §6 says.
3. Run `check` (read-only) as soon as master350 succeeds. It measures the real counts and budgets and the native327 equality, and spends nothing on a refusal.
4. Then notes §5 steps 2–6, with the ledger.

## 6. Evidence (scratch, outside the repo; local only; nothing written beyond these small scripts)

| File | sha256 | Use |
|---|---|---|
| `ast367.py` | `f547fa8a136d4e4e35ebb928823ec412182e9f5ed79398675c4849c710e65e92` | 339 → 363 top-level AST delta |
| `plan367.py` | `ccf17b76be45b874b8393068e76afab90cecb69f7325181ac160116460e4821d` | in-memory replay of 363's `picture_plan` and `part_upper` |
| `plan367b.py` | `ebbe44b868f2d9081ef0125b33de786a182a6d44277dbfdc85073818b0819c44` | the same, plus 357 observation coverage; output `9a24f958…` |
| `intervals367.py` | `d1c73332b92ddc82c1f24c6dd7fb997232ed2099db0f157f4a6f7111ad8f5df5` | interval natives in 363's planning order |
| `scopes367.py` | `029229bca2629a8b909c318a7429c9012a7e2edd50ca64f0f0562f4f9fcaeacc` | the permits' `review_scope` |
| `hygiene367.py` | `56db6b396daa19c88236fda4393dcdeb3fb3117eb2d48eabff0c6982b47c4b89` | host-free permit checks |
| `near367.py`, `plan09.py`, `obs357.py`, `runs367.py`, `budget367.py` | `5090bb9e…`, `3d2bce0b…`, `68c23b31…`, `4663884e…`, `2416cf77…` | record reads |

**Local tools.** python 3.15.0. 363 was loaded read-only with `-I -B`, so no byte-code was written to Q.

**Faked:** the stand-ins named in §2.

**Real:** 363's own `picture_plan` and `part_upper`, the constants, and the Q records and permits listed above.

**Not done.**
- Nothing ran on CSD or CSS.
- No master, chapter, picture or caption file of the real film was decoded or rendered.
- I did not run CC2's fixture or its real347 control.

## 7. Limits

- This is a technical review of code and records. **No picture, motion, fine-sync, caption-reading, caption-playback, 4K-detail or final AV acceptance is claimed or implied. No audio heard or checked.**
- Every review flag 363 writes stays False. They remain root's and the owner's.
- I did not change any file named above.
