# 995 successor for the repaired master: CC3 scratch drafts (final-verify-cc3)

CC3 (WhiteSwan), 2026-11-11 00:24Z. Drafts only and not reviewed. Nothing was launched, built, rendered,
decoded or extracted, and no production file was edited. No audio was heard or checked. Nothing here grants
picture, motion, fine-sync, caption, listening or AV acceptance. It goes with the master and stage successors
in final-master-cc3 (handoff `cbf3b963`).

## Files

| File | sha256 | What |
|---|---|---|
| csd-final-verify-repaired01.py | `5c30a62ef9341ff72be6650d462f7de2eb32ad0c5089b826d50897d3c9ef4dbb` | successor of csd-final-verify995.py (69fa3ded) |
| csd-final-after-master-repaired01.py | `150940016dff9724b56298f79f3b1160bf84fad1663441406b12e3d4e7d091dd` | successor of csd-final-after-master995.py (9a055c26) |
| verify-vs-995.diff | `6e88ffb1fda834176a7e8db364aa746c30e45343e1ea0b75c083dbda73f10dc7` | +85 -32 |
| after-master-vs-995.diff | `7e2b7a8cef6f1ac4a231c860bf1fdc81633dd0b4ecc2bf47efd67dfe7f76b0a1` | +21 -13 |
| check-final-verify-cc3.py | `d3beba1be540e9f5a6cab1f99f435e6a9f40090692c3328c0a22010f0988b57e` | local checks |
| check-final-verify-cc3.out | `2892c8f841ab33d5c8594ee06f982ba875f9bb41ebd0d2efcd1e6c6d6f5ba7c0` | 57 passed, 0 failed |

## What changes from 995

Only these functions change: `expected_worker451`, `master350`, `picture_plan`, `plan_for`, `verify`,
`pictures` and `main`. `unresolved` is new. Every other function is byte-identical to 995's, including
`film`, `schedules`, `delivery347`, `frame_hashes`, `anchors` and `launch`.

- **Pins.** Every mode refuses until root sets all of them. The versions must be digits.
- **Rows.** 01, 08 and 09 are `native-combined01` and 11 is `native-counting02`, matching the master successor.
  Each row comes from its stage job, `csd-native-stage<STAGE_VERSION>-<nn>-a<k>`, through the pinned materializer
  and its chain binary. 10 stays `native521`. The `native844` 09 entry is gone. Both new kinds have the
  review basis `sampled_preview`.
- **Admitted scope (M1).** Each repaired row's sampled intervals are the `review_scope` the master records.
  CC0's gate builds that scope from the chapter's own admission, extraction and viewing, and nothing is carried
  over from master993. The successor plans those intervals as 995 plans every `sampled_preview` interval: every
  frame is an overview sample, with a native frame inside.
- **Final extraction carry-over.** 995 carried final969's 09 samples. The successor carries every
  `selected_frames` entry of the actual final extraction for all four repaired rows. It reads the extraction
  from the row's `local_proofs`, requires the master to bind it, and requires a full decode, the row's frame
  count and the row's movie sha. For 09 this still covers 969's 814 frames: the 09 final extraction must hold
  prior775, and prior775's selected frames equal 969's (checked locally).
- **Root 343's four 09 instants.** These are 27.687, 31.901, 35.512 and 39.123 s. CC0's gate requires the
  repaired 09 row's `playback_seconds` to be empty, so the instants no longer reach the plan through that row.
  `plan_for` now reads `root-preview313-playback343.json` at its pinned sha (`d9f250ec`), and requires its
  `playback_observed_seconds` to equal the four instants and every `*_acceptance` field to be false. The plan
  then adds `round(s*60)` +-2 overview samples and a native at the centre, as 995 did. Each is labelled
  "final still only". No playback, motion or listening of the final is planned or claimed. `sampling.json`
  records them as `boundary_instants343`.
- **995:1102-1104 not weakened.** The 08 run 10200-10380 must still lie inside 08's sampled intervals. With
  08's interval narrowed, the check refuses.
- **Texts.** The scope and pass texts name the master version in place of '993'.

`csd-final-after-master-repaired01.py` takes `HELPER_NAME` and `HELPER_SHA` (both `None`). It loads the pinned
verifier, asserts `not h.unresolved()` and takes every job path from it. As 995's version did, it waits up to 12 h
for the master's success, then runs `check`, `launch-verify` and both `launch-pictures` serially. It requires each
job's success before the next step and rehashes the verifier before every mode.

## Pending pins (`None`; refuse while any is unset)

**Verifier:**
- `VERSION` and `MASTER_VERSION`
- `MASTER_HELPER_NAME` and `MASTER_HELPER_SHA`
- `STAGE_VERSION`
- `WORKER_NAME` and `WORKER_SHA`
- `REPAIRED_BINARY['native-combined01']` and `REPAIRED_BINARY['native-counting02']`: each chain's
  qualification `binary_sha256`, the same values as the run helper's pins

**After-master:**
- `HELPER_NAME` and `HELPER_SHA`

Order (CC2 S1, accepted by root): qualify counting build02, fill both binary pins, stage the run helper once, and
only then launch any final. The verifier pins follow the actual master.

## What the checks prove (local, static and synthetic)

- Both files compile. The function and name changes are exactly the ones listed above.
- `verify()` and `pictures()` equal 995's line for line once the version texts (and the one record field) are normalized back.
- 995 and the successor plan pictures on the same synthetic master. That master uses plan182, the preview313
  records and master993's intervals. In place of the not-yet-existing final extractions, it uses 969 for 09 and
  the preview frame lists for 01, 08 and 11.
  - Every one of 995's 8351 overview samples is kept. The only new samples are the 01/08/11 frames (+940).
  - The 123 native frames are identical, the four 09 instant natives included.
  - The overview upper estimate is 1.75 GiB and the native upper estimate is 2.91 GiB, against a 3.5 GiB budget
    each. These sizes are stand-ins. The actual admitted scopes and extractions set the real plan, and
    995's unchanged budget checks still refuse at runtime if it grows past them.
- `plan_for()` passes the local Q records. It refuses an 08 scope without 10200-10380 and refuses instants that
  differ from root 343's.
- Both helpers refuse with their pins unset and write nothing.

Not exercised: anything on CSD, the real master summary and manifest, CC0's gate on real records, actual
admitted scopes, the guard, and any decode or extraction.

## Open points

1. **CC0's 01 stem fix** (Mail 25072). The materializer refuses 01 until it is fixed.
2. **08 scope frames.** Root assigned these to CC0. The successor keeps 995's 08 window requirement unchanged.
3. **CSD facts to read before use.** Check that `root-preview313-playback343.json` is in CSD Q at `d9f250ec`.
   Master993 pinned it, but this draft did not read the CSD copy.
4. **Interpreter.** Run both helpers as `/usr/bin/python3 -I -B`, as 995 did.
