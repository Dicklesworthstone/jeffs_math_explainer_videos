# Review 320: boundary final 317 (worker, launcher and notes)

cc3, 2026-11-08. Independent source-only review of CC2's frozen 317 handoff. Root verified the handoff at 19:06.

I did not edit any CC2 or CC0 file, launch or stage anything, write a permit, or touch Beads, TODO or source. The 313 jobs have produced no records I can read yet, so every 313 dependency is checked against the code of the helpers that will write it.

Paths use the helpers' own names: D is the build host's project root, and Q is `D/qualification01/production-repair17`.

## Files reviewed

| File | sha256 |
|---|---|
| `Q/csd-reviewed-boundary-final317.py`, the worker (252 lines) | `19abeee93dbdb98a0217c69b005f05fb18daff716c10406c67ed36fb7210b036` |
| `Q/csd-launch-boundary-final317.py`, the launcher (105 lines) | `47e0f30ac3aa86e27936a04b3706bc9972b7a9d6eb656c27c37cfbe8350e4c8c` |
| `Q/cc2-boundary-final317-notes.md` (199 lines) | `08be95bea457de1bb25a64fd434801a7a9742019ea19023f1c413fe0d3ac9fee` |
| base: `Q/csd-reviewed-final303.py` (unchanged) | `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` |
| base: `Q/csd-launch-final303.py` (unchanged) | `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` |
| `Q/cc0-combined-master293.py` (integration only) | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` |

The 313 dependencies were read at these hashes:
- `review-repair-build313.py` `ff1ff0b8…`;
- `launch-review-repair313.py` `d384d4d8…`;
- `review-repair-after-build313.py` `f28a15ab…`;
- `review-repair-rebind313.py` `0e04aa9e…`;
- `review-repair-timeline313.py` `10349317…`;
- `review-repair-preview313.py` `67d780d8…`;
- `launch-review-repair-preview313.py` `c983ecbf…`;
- `preview-after-timeline313.py` `96af8d85…`;
- `boundary-guide-repair310.rs` `bda7d931…`.

All 11 files in root's staging record `repair313-stage305.json` (`97881557…`) equal the local copies byte for byte. They are these 9, plus `cc1-boundary-review312.md` and `cc0-repair313-notes.md`.

Recipe313 is `fbc807d9…`, and 314 is `56a3ad5d…`; `repair313-approval-stage305.json` records both as staged.

## Result

**No High or Medium defects. One Low finding, in the notes only.**

As written, 317 will launch once four things are done:
- build313, rebind313, timeline313 and preview313 finish;
- root completes its own review of preview313;
- an independent review exists;
- root writes the permit.

317 cannot be exercised at run time before then, and master 293 cannot take its output. Root has recorded that as a known prerequisite for master318; the exact points are under "For master318" below.

## Finding

### L1 (Low, notes only). The notes cite 2087–2099 for the zoom-line exit, but the gate 317 inherits accepts 2086–2100

- **Where the window is stated.** Notes :113 put "312 L2 predicts that exit at 2087–2099" beside the motion-coverage gate.
- **What 317 actually enforces.** It ties the exit to preview313's probe, through `zoom_probe_fails == []` (worker :139). The probe's window is `EXIT_WINDOW = (2086, 2100)`, inclusive (preview313 :837–838, :910).
- **Why the two differ.** 312's figures are a by-eye estimate, ±2 frames (312 :61, :104). CC0 measured the base movie's hole-name fade at 2088–2099 and set the window one frame wider on each side (313 notes :127).
- **Effect.** An exit at frame 2086 or 2100 passes every 317 gate. The code is sound; the notes just state a tighter window than the one enforced.
- **Fix.** One line in the notes saying the enforced window is the probe's 2086–2100. Otherwise someone writing the permit may take 2087–2099 for a gate that 317 does not check.

## What I checked

### 1. The exact diff against 303

**Unchanged lines (Python `difflib`).** These 303 regions are identical in 317, which confirms notes :39–61:

| Region | 303 lines | 317 lines |
|---|---|---|
| `verify_tables` | :140–166 | :181–207 |
| admission, the 8/4 policy, the inputs hash, unique rows and the re-hash of every frozen row | :170–177 | :212–219 |
| CPU affinity, the render command and environment, ffprobe, x264 `crf=16.0`, the soundtrack capture, the repeated `check()` and the second frozen re-hash | :183–203 | :227–247 |
| `render-summary.json` opened `'x'`, and argparse | :206–208 | :250–252 |
| permit gates | :125–129, :131–132, :135–138 | :165–169, :171–172, :176–179 |

**Changed lines in `check()`.** Each one is narrowed or replaced, never loosened:

| 303 | 317 | Change |
|---|---|---|
| :110 | :151 | The binary comes from `build313()`, not a constant. |
| :113–116 | :154 | The build268 qualification still binds binary 268. |
| :119 | :157 | Same 15-table comparison against timeline21; only the constants are renamed. |
| :124 | :164 | The permit binary must be build313's. |
| :130 | :170 | Message only. |
| :134 | :175 | Adds `frames == 16504`. |
| none | :159–160, :173 | Adds `narration313()`, `schedule313()` and `independent()`. |

**Launcher diff.** Against 303, the launcher:
- limits the chapter to 09;
- loads the 317 worker, and asserts `VERSION == '317'`, `CHAPTERS == (09,)` and `frames == FRAMES`;
- replaces preview274's closure with four `completed()` closures, one per 313 job;
- adds rows for the independent review, the three narration folders, the three schedules, five workspace files and repair310;
- adds the timeline21, build268 and rebind269 receipts;
- prints the binary hash and the four 313 input hashes.

**Nothing 303 bound has been dropped.** Preview274's receipt, manifest and every row now arrive through preview313's own manifest (preview313 launcher :58–69). The worker-hash-before-load guard (:36–41), the inputs file opened `'x'` (:91) and the guard argv (:96–102) are 303's, unchanged.

### 2. The 313 dependencies, key by key

**Keys and names.** I checked the 30 record keys 317 reads, and the 16 job, manifest, record and folder names it uses, against the helpers that write them. Every one is written under the same name.

**Build313.**
- **Qualification** (build313 :174–180). It holds:
  - `binary_sha256`, `base_binary_sha256`, `tests_passed`, `explicit_native_plate_fade_probe_passed`, `clippy_passed` and `fmt_check_passed`;
  - `facts_sha256`, `chapters` and `changed`.
- **Source copy** (:152–154). `files`, `changed` and `base_source_copy_sha256`.
- **The tree.** It has build268's 70 keys with only `boundary.rs` changed (:119–125), as `build313()` requires (worker :87–91). The contract files and `boundary.rs` are copied into the 313 workspace (:113–118), so worker :92 holds.

**Rebind313.**
- **Summary** (rebind313 :174–180). `wav_files_byte_identical` and `wav_files_equal_rebind269` are both 105. `canonical_sha256` is taken after the rewrite at :127, and `map_sha256` is that of the review313 map.
- **Narration folder.** It is a copy of assembly19's folder (:129). Its names equal rebind269's, and only the three metadata files differ, as `narration313()` checks (worker :100–104).
- **The rewrite keeps `completed()` true.** Rebind313's guard inputs leave out `narration_units.json` (after-build313 :40). Rebind313 only adds new files to the tree: the backup and the review313 map. Timeline313 then binds the tree after the rewrite. So `completed()` holds for both jobs after rebind313 has run.

**Timeline313.**
- **Record** (timeline313 :66–68). It writes `frames`, `named_path_mapping` and `production_path_mapping`. Each table row carries `new_sha256` and both equality flags.
- **The mappings** are `{rebind269: rebind313}` and `{assembly19: rebind313}`, exactly as `schedule313()` asserts (worker :111–113).

**Preview313.**
- **Tables.** They are written to `J/preview` (:24, :970–972) and must equal timeline313's (:984). Worker :137 checks the same equality.
- **Records.** `summary.json` (:1069–1078), `zoom-probe.json` (:1006–1009) and `sampling.json` (:1060–1066). Both sub-records carry `movie_sha256`.
- **Fail field.** `fails` is a list (:898, :942), so `== []` holds on a pass.
- **The job fails on any probe failure** (:1080–1081). A successful preview313 receipt therefore already means the probe passed.
- **Worker pin.** The worker is listed in its manifest and in its receipt command (preview313 launcher :38–49 and the `--` command), as worker :133 requires.

### 3. Input closure and stable receipts

- **`completed()`** (worker :74–80) requires three things for each 313 job:
  - a successful receipt;
  - the job's own manifest at the receipt's `inputs_sha256`;
  - every row of that manifest still at its hash.

  The manifest names equal the ones the 313 launchers write: build313 launcher :51, after-build313 `freeze_launch`, preview313 launcher :44. That makes the receipts stable in the sense root asked for.
- **Every file `run()` checks is in the launcher's inputs.** I compared `run()`'s list (worker :220–228) with the launcher's file set (:52–87) entry by entry:
  - the source-binding evidence, the preview binding and the independent review;
  - BIN, the worker, the permit and the timeline313 tables;
  - the timeline313 manifest and its rows;
  - all three narration folders and the three schedules;
  - the five workspace files and repair310;
  - the score files.
- **The binary hash is never written in advance.**
  - `build313()` reads it from the build313 qualification and checks that it equals `sha(BIN)`, differs from 268, and names 268 as its base (worker :84–85).
  - The permit must name the same hash (:164).
  - The launcher asserts it again and prints it before the guard call (:88–95).
  - The render summary records it, together with the qualification hash and repair310's hash (:249).

### 4. Gates preserved

Every 303 gate below is kept as is:
- the encoder, resource, table and score gates;
- the permit gates: binary, chapter, host, both `actually_reviewed_*` flags, `unresolved_defects == []`, reviewer and observations, `timeline_sha256`, evidence and source-binding items at their hashes, motion intervals inside `[0, frames)`, and stems QA.

317 adds:
- the four 313 closures;
- the source check for exactly one substitution;
- WAV equality across rebind313, rebind269 and assembly19;
- the double schedule equality;
- the independent review;
- the preview313 records, with motion covering the exit frame.

It also refuses an existing 303 job for 09 (:60, :150), as notes R1 says. Both files compile, and a scan of names found nothing undefined except `__file__`, which exists at run time.

## For master318 (known prerequisite, from 293's actual lines)

**293 refuses a 317 row at three points:**
- `:304` requires the worker name to start with `csd-reviewed-final`. The 317 worker is `csd-reviewed-boundary-final317.py`.
- `:312–314` require binary 268 and `module.BINARY_SHA`. The 317 module has no `BINARY_SHA`, and its binary is build313's.
- `:215` (through `permit_gate`, with the table from :312) requires the permit's `timeline_sha256` to be timeline269's 09 table. A 317 permit binds timeline313's 09 table instead. The two differ only in the narration path, so their hashes differ.

**293 already accepts these parts unchanged:**
- the job name: the `:310` pattern matches `csd-reviewed-final-09-boundary317`;
- the `preview_binding(key, review)` and `check(key, permit)` signatures, including the 3-tuple compared at :359–360;
- `schedule()` (:234–248). `module.NARRATION` is rebind313, timeline313 equals timeline269 under exactly that mapping, and the 105 WAVs equal rebind269's.

## Observations (no change asked)

1. **How the permit is tied to the preview run.** 303 pinned preview274's inputs hash as a constant, because preview274 had already run. 317 cannot do that. It relies instead on:
   - preview313 being unique: its launcher refuses an existing job or manifest (:43–45);
   - the permit's hashes of `sampling.json` and `zoom-probe.json`, both of which must name the movie hash (worker :141, :144).

   The render summary records preview313's inputs hash (:249). This is equivalent for one preview run.
2. **The plate-site flags are not gated.** Preview313's summary records `flags` and `not_localized` for the plate sites, including E2 rebound to line 1402 (:1069–1076). 317 reads only frames and the zoom fields (:139). 303 read no preview summary at all, so no gate has been weakened, but root should read those two fields when writing the permit.
3. **`source_binding_evidence` is still required to be non-empty** (:170). Preview and final share binary 313, and `preview_binding()` already proves that. The notes' permit section names the field without saying what it should hold, and any hashed file passes.
4. **Hashing passes.** Each `check()` makes one full pass over the four 313 closures, which include preview274's 12,669 rows. Before the guard returns there are 8 such passes:

   | Where | Passes |
   |---|---|
   | launcher: `check()` and the `completed()` loop | 2 |
   | `run()`: `check()` before and after the render | 2 |
   | `run()`: frozen re-hash at :219 and :247 | 2 |
   | guard: start and end | 2 |

   Notes R4 says "at least three". This costs time only.
5. **One attempt per version.** The job name and inputs name are fixed by `VERSION` (:69, :213; launcher :90–91, `'x'`). The guard creates the job folder before admission. So a refused admission, or any failure, needs a new worker version, as with 303. That is consistent with root's rule of no automatic retry.
6. **`independent()` doesn't constrain the review's content.** It accepts any file directly in Q at its hash, under a reviewer name different from `reviewer`. It says nothing about what that review must cover, so whether it covers preview313's sheets is root's call.

## Method and limits

My checks were run read-only from the repository root with `python3 -I -B`:
- `difflib` on both pairs of files;
- a key-and-name cross-check against the 313 writers;
- a comparison of the staged 313 hashes with the local copies;
- compiling both 317 files in memory, plus a scan for undefined names.

The scratch scripts wrote nothing in the repository. I read CC2's 24 fixture cases but did not rerun them.

Not done:
- no 313 or 317 run, and no reach to any remote host;
- no use of runtime records, since none exist yet.

No listening, picture, motion, 4K, fine-sync, caption or AV acceptance is claimed. The Frederick takes are checked as bytes only, through the rebind and worker records. 308, 315 and 316 are unchanged.
