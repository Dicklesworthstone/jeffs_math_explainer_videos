# 993/994 successors for the four repaired rows: CC3 scratch drafts (final-master-cc3)

CC3 (WhiteSwan), 2026-11-11. Drafts only and not yet reviewed. Nothing was launched, built, rendered or
decoded, and no production file was edited. No audio was heard or checked. Nothing here grants picture,
motion, fine-sync, caption or AV acceptance.

## Files

| File | sha256 | What |
|---|---|---|
| combined-master-repaired01.py | `17388c9dcb20326475ca47d93be98aa04f185c1839d2db097310ed84e5108beb` | master993 successor |
| csd-native-repair-stage01.py | `432301faa58e103f5bfdfca591dc4d2c030ffb451da43403b83252510f01193e` | 994 successor: `stage --chapter` and `compose` |
| csd-native-repair-pictures01-after-final02.py | `332ee0007ed5c05a332bff9544aa1f63aadeeafd8c2a26a6b624862d95affad8` | after-final c27f4ae2 plus CC2 L1 (c27f4ae2 kept) |
| master-vs-993.diff | `ad7817b5…` | +56 −20 against 1955ca32 |
| stage-vs-994.diff | `8dc95779…` | +128 −32 against f0641032 |
| after-final02-vs-c27f4ae2.diff | `c6eae123…` | +4 −1 |
| check-final-master-cc3.py | `0e844001…` | local checks (syntax, AST, schema, synthetic) |
| check-final-master-cc3-03.out | `ebd23287…` | current result: 101 passed, 0 failed, 1 finding for CC0 |

`check-final-master-cc3.out` (`1725230d…`) is an earlier run, kept. It predates the S1 assert and the
captured compose output.

## Sequence (CC2 S1, accepted by root)

1. Qualify counting build02.
2. Fill **both** `binary_sha256` pins in the run helper (combined01 and counting02).
3. Stage the run helper in Q once. It is immutable from then on.
4. Only then launch any final.

There is no early combined-only staging. The run helper's file name is fixed, and `lineage()` and
`success()` rehash it, so a later edit would fail a running final or refuse a finished one's extraction.
The stage successor refuses unless every chapter's binary pin is filled in the staged runner
(`not any(run.unresolved(k) for k in run.CHAPTERS)`).

## combined-master-repaired01.py (993 successor)

Changed from 993, and nothing else:
- `VERSION`, `REPAIRED_GATE_NAME` and `REPAIRED_GATE_SHA` are `None`, and every mode refuses until root sets them.
- The base is master993's own manifest, `8da77ed7` (`csd-native-stage970-09-a2-master-manifest.json`, as
  master993's summary names it), not 625. Its 11 rows other than 01/08/09/11 must be equal, row for row.
  Those 11 rows still run their existing gates unchanged: the retained, combined and native521 branches,
  `film()`, `schedule()`, the preflight, `master` and `captions all`, all 15 replays, and `claimed()`.
- `REPAIRED`:
  - 01, 08 and 09 must be kind `native-combined01`; 11 must be `native-counting02`.
  - Both kinds go to the adopted 992 successor, CC0's gate, through 992's interface, pinned by name and sha.
  - No `boundary313` or `native844` row is eligible.
- `launch` waits on the stage receipts of both kinds.
- The boundary replay takes the 09 `native-combined01` row. `capture_check_before_master` is still the
  gate's original313 check.

**M1.** No intervals are carried from master993. The gate's `admitted_scope()` builds each repaired row's
scope from that chapter's own admission, extraction and viewing.

## csd-native-repair-stage01.py (994 successor)

**`stage --chapter CH --job J --root-review R --independent-review I`** does what 994 did, with these changes:
- The chain, final and final extraction come from the adopted worker, run helper and extraction helper.
  Each is pinned by sha, and the chain and frame tables must agree with each other.
- Prerequisites, each a receipt plus that job's own guard inputs: the chain's build, delivery, timeline,
  preview and preview pictures, plus the chapter's final and its final extraction.
- `run.proofs(ch)` runs again, read-only. That binds the admission, both preview reviews, the preview
  extraction record with its pictures and sheets, the preview movie and capture, both comparison records,
  the delivery record, qualification, binary, scene and stems.
- The final extraction must hold every preview extraction frame.
- Both reviews must name the chapter's scene sha.
- The base manifest is `8da77ed7`.
- The guarded worker argv is exactly the 14 items that CC0's materializer and gate assert.

**`compose --stage J1 J2 J3 J4`** writes one new Q file, `csd-native-stage<VER>-composed-master-manifest.json`,
with `open('x')`. It is `8da77ed7` with exactly the four rows replaced. Each row is taken from a
successful stage whose job manifest equals its Q copy, was made from `8da77ed7`, and changes only its own
row.

## Pending pins (`None`; each helper refuses while any is unset)

**Master:**
- `VERSION`
- `REPAIRED_GATE_NAME` and `REPAIRED_GATE_SHA`

**Stage:**
- `WORKER_NAME` and `WORKER_SHA`
- `RUNNER_SHA`, set after step 3 above
- `PICTURES_SHA`

**Consistency for root when filling:**
- The gate's `FINAL_RUNNER_NAME` is `csd-native-repair-final01-run.py`.
- `FINAL_JOBS[ch]` is `csd-native-repair-final01-<ch with - for _>`.
- `ADMISSIONS[ch]` is the run helper's `admission(ch)` name, `csd-native-repair-final01-<ch>-picture-admission.json`.
  That admission also carries `review_scope` (the run helper ignores extra keys).
- `STAGE_VERSION` must be the same in the materializer and the gate.

## What the checks prove (local, static and synthetic)

- Against 993, only `chapter_gate`, `gates`, `launch` and `master` change, plus the new `unresolved`.
- Every `base.<name>` that CC0's gate reads is defined in the master successor.
- `manifest625` plus master993's 09 row rebuilds `8da77ed7` byte for byte.
- On synthetic stages made as the materializer makes them:
  - compose gives 11 base rows and 4 stage rows, in any stage order;
  - compose refuses 7 bad cases and leaves no output after each, and refuses to overwrite its own output;
  - `gates()` passes the composed manifest at 133662 frames and refuses 6 bad manifests, including
    master993's own manifest;
  - `chapter_gate` sends all four rows to the pinned gate and refuses an unpinned or wrong gate, a
    `native844` 09 row, and 11 as `native-combined01`.

Not exercised: anything on CSD, guard admission, the real gate and materializer on real records,
`run.proofs()` on real files, and the master body.

## Open points

1. **Finding for CC0.** 01 has a single unsplit score stem, `01_the_bump.music.f32.wav`, in the run
   table and in final01's cuelog. The materializer builds `stem_names` from the `.wav.part` prefix, so the
   set is empty for 01 and the materializer refuses chapter 01. Mailed to CC0; the fix is CC0's.
2. **08 scope frames.** CC0 resolves these (root's assignment) without weakening 995 and without
   modifying any active frozen input.
3. **995 successor.** Not done here; it stays for after the actual reviews. Its known points:
   - `REVIEW_BASES` and `KINDS` for both kinds;
   - the 09 job prefix (421–425);
   - version '993' (462);
   - the boundary checks (496–500);
   - the final969 carry-over (708–718), which reads `local_proofs`;
   - root 343's four 09 playback instants, if they should still be sampled.
4. **CSD facts to read before any stage.** 994's no-symlink rule now covers the combined01 and counting02
   build job folders. Loading the extraction helper needs PIL under `/usr/bin/python3 -I`, as 969's
   extraction did.
5. **Interpreter.** Run after-final02 as `/usr/bin/python3` (CC2 note).
