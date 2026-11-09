# Review 332: the 321 → 330 delta (CC2)

cc0, 2026-11-08. This is an independent review of the delta from CC2's 321 to its frozen 330. It is checked against my review 323's M1 and L1–L4, against the actual 318 output contract, and against root's question about overview-output integrity.

I edited no CC2, CC1 or root file. I ran nothing on CSD, launched no guarded job, wrote no permit and made no Beads, TODO or canonical edit. All local work ran on fixtures outside the repo. **No audio heard or checked.**

D is the CSD root and Q is `D/qualification01/production-repair17` (locally, this folder). `:N` is a line of `csd-final-verify330.py` at `4fc4594d`.

## Files reviewed (frozen; hashes confirmed with CC2 and on disk)

| File | sha256 |
|---|---|
| `Q/csd-final-verify330.py` (956 lines) | `4fc4594d4dccd58e407824fd7ccb59a5b786ba9c8c85dd38603916b669749437` |
| `Q/cc2-final-verify330-notes.md` (344 lines) | `9e11d3964897079a7b86f32425208c11df427eddbb9c723c8a0d24ad0513e781` |
| `Q/csd-final-verify321.py`, the delta base, unchanged | `0a956ec8c0613971ad33e90a0a9eb4a496ba42bd2cb06d8044105397bfc5b7fb` |
| `Q/cc2-final-verify321-notes.md`, unchanged | `c7e537a4b511193164648cfd0fa61ab612a1df0aa20afa038bcbfc24d3098a91` |
| `Q/cc0-combined-master318.py`, the contract, unchanged | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` |
| `Q/cc2-verifier-deployment328.md`, unchanged | `4ac5b9f8cd05e08a739394de464ac292e46597fda7c2e3b6800cd5f2696f1c4e` |

**Coordination.**
- I asked CC2 (pane %2) for frozen hashes and reviewed nothing while 330 was moving. It passed through `35401289`, `1788cc63` and `d6e6fac3`.
- At CC2's request I commented on the picture-job split before the freeze, and it adopted the fixed overview/native split.
- After root's question below, I sent CC2 the exact corruption cases. CC2 has started a successor, `csd-final-verify339.py`. This review covers 330 only; 339 needs its own.

## Result

**One Medium (M1, new), one Low (L1, new). Every 323 finding is fixed as asked.**

- The 318 contract that 330 reads is exactly 321's: the guard command, summary keys, rows, source jobs and boundary replay.
- In my own regression cases, 47 of 48 came out as expected on frozen 330. The 48th (W3) is a note, not a defect.
- Root's suspicion about overview outputs is a **real gap**, reproduced end to end through 330's own launch and job code (M1).

## Findings

### M1 (Medium). The overview job's pictures and sheets are never checked against their recorded sha256

**Where.**
- `part_done()` (:800–808) runs `stable()` over the overview job's **inputs** only. It then checks `part`, `plan_sha256` and the flags in its `sampling.json`, and returns every file now in the overview folder.
- The native launch (:946–952) binds those **current** bytes into the native inputs, so the native guard and `admitted()` re-hash against whatever is there at native launch.
- The union check (:836–841, the assertion at :839) compares only frame, chapter, local frame and reasons; it ignores each sample's `path` and `sha256`.
- So nothing in 330 compares an overview picture or sheet with the sha256 its own `sampling.json` recorded.

**Reproduced.**
- **The driver.** 330's own `main('launch-pictures')` → `launch()` → guard → `pictures(part)` → `part_done()`, run twice (overview, then native).
- **The fixture.** A synthetic 24-frame 3840×2160 H.264 master, and a plan of 13 samples, 2 sheets and 1 native.
- **What was replaced.** Only `shared`/`film`/`verified`/`plan_for`, CPU affinity, ldd for Pillow, and the guard, by a small emulator that does what the guard's receipts show:
  - at admission, a receipt with the child command and the inputs sha256, with the rows re-hashed;
  - at the end, the rows re-hashed again, then exit 0 and `final_inputs_sha256`.
- **When.** Every change below was made after the overview receipt was final (exit 0) and before the second `launch-pictures`.

| Case | Change to the overview job's files | 330 at `4fc4594d` | Prototype fix |
|---|---|---|---|
| C0 | none (positive) | accepts | accepts |
| C1 | `pictures/` sample 5: one byte XOR 0xFF, same size | **accepts** | refuses |
| C2 | sample 3 overwritten by sample 4's bytes: a valid JPG of the wrong frame | **accepts** | refuses |
| C3 | sample 2 truncated to 500 bytes | **accepts** | refuses |
| C4 | sample 7 deleted | **accepts** | refuses |
| C5 | `sheet-0002.jpg`: one byte flipped | **accepts** | refuses |
| C6 | an extra `pictures/frame-000099.jpg` | **accepts** | refuses |
| C7 | sample 1 flipped, and its `sha256` rewritten in `sampling.json` | **accepts** | accepts (needs an outside anchor; see fix 2) |
| C8 | control: sample 5 flipped after the native inputs were written | refuses (native admission re-hash) | refuses |
| C9 | control: one sample's reasons changed in `sampling.json` | refuses (union check) | refuses |

**What 330 did in C1–C7.** The native launch and the native job both passed. The native `sampling.json` records `union.frames_and_reasons_equal_plan: true`, and the job prints that the plan's samples and natives "are all extracted".

**Effect.**
- M1's purpose is root viewing every frame of the 305/319 windows. Pictures that differ from the extracted frames would be viewed under a record that calls the set exact and complete. That covers a damaged file, the wrong frame, or a missing one.
- Corruption can come from another agent writing into `D/jobs`, an in-place conversion or copy, or a disk fault. Any of these may happen in the gap, possibly hours long, between the two jobs.
- The master's verification is not affected. Only the integrity of the picture set is.

**Fix (recommendation only).**
1. **Check every recorded output in `part_done()`.** My prototype adds 9 lines after the flags assert, on a copy outside the repo (sha256 `9b15684c…`).
   - Every `{path: sha256}` in `samples` + `natives` + `sheets` must be a file at its recorded sha256, and no path may repeat.
   - The files under `pictures/*`, `native4k/*` and `sheet-*` must be exactly the recorded paths.
   - `len(sheets) == ceil(len(samples)/8)`.

   Because `part_done()` already runs at the native launch and again inside the native job, this checks both times. It refuses C1–C6 and keeps C0, C8 and C9 unchanged. CC2's planned 339 fix is the same check.
2. **For C7, anchor `sampling.json` outside the overview folder.**
   - The overview job prints its `sampling.json` sha256 on its last line.
   - `check` and the native launch print it too, and root records it when the overview job succeeds.
   - Optionally, the native launch takes that value as an argument and refuses on a mismatch.
3. **Re-hash before viewing.** No later job checks the native PNGs, or the overview files after the native job. Before viewing, root should run one read-only re-hash of every output listed in both `sampling.json` files.

### L1 (Low). The exact framemd5 timebase is untested on CSD's ffmpeg

**What 330 requires.** `frame_hashes()` (:556–579) requires the literal `#tb 0: 1/60` line for every chapter and for the master.

**What the evidence covers.**
- My local evidence is ffmpeg 8.1.2: the kit's real 08 chapter render and synthetic joins (T0–T17).
- CSD runs 8.0.1-3ubuntu2 (notes §4 probe). That probe exercised only `runtime()` and `pillow_runtime()`, not framemd5.
- If 8.0.1 printed another timebase for these MP4s, or filled a gap by duplicating a frame, the verify job would refuse. That fails safe, but the inputs file is already written, so it spends 330's one verify attempt.

**Fix.** Before `launch-verify`, run one read-only check on CSD:
- run `/usr/bin/ffmpeg` with 330's exact framemd5 argv, plus `-frames:v 3`, on an existing 4K chapter render already on CSD;
- check that the output has `#tb 0: 1/60`, pts 0, 1, 2 and duration 1.

A gap is refused either way: by pts if ffmpeg passes timestamps through, as 8.1.2 does (T2), or by the frame count if it duplicates.

## The 323 findings, checked in the delta

### M1 (windows) is fixed

**The records.**
- `TRANSITION_SOURCES` (:189–211) pins 21 records, and all 21 are at their pins locally (W1).
- `plan_for()` refuses a changed or missing record (:768–775) and binds them all into the picture inputs.

**The constants, against the records.**
- `TRANSITION_WINDOWS` equals the windows I derived only from the eight frame manifests' frame lists: 31 windows, 1,932 frames (W0).
- `GALLERY_SITES` equals the 319 record's `measured_curves` `full_frame` values: 17 sites from 22 curves, with C13's panels at 8,859 (W2c).
- `ONE_POINT_INSTANTS` equals `round(observed_seconds × 60)`: 4858, 4872, 5099, 5355 and 5681.

**The plan, on local stand-ins** (all 30 previous rows used as current layouts, preview274's 09 sampling standing in for preview313):
- every window frame and all 25 instant frames are sampled;
- every window has a native, and the 17 site natives are present (W2, W2b);
- each window frame's first reason names its window;
- a window ending on a chapter's last frame is accepted, and one past the end refuses (W4, W5).

**Extraction**, by 330's own `pictures('overview')` and `pictures('native')` on the synthetic 4K master:
- every picture is the planned frame;
- the native PNG is byte-identical to that frame decoded to RGB;
- a planned frame past the end refuses, and so does a lost extracted frame (W6–W8).

**The budget split.**
- The upper estimates are overview 1.45 GiB (7,652 samples, 957 sheets) and native 2.87 GiB (121 natives), both within 3.5 GiB (G0).
- With fewer current-layout rows, the native estimate drops to about 2.2 GiB.
- `check`, `launch-pictures` and the job itself all refuse over budget. Nothing is regrouped or dropped.

### L1 (framemd5 timing) is fixed

`frame_hashes(path, frames)` is called with each chapter's frame count and with the master's 133,662 (:706, :709). In my cases (T0–T17):
- **The real gap.** A kit-style join (concat demuxer, `-c:v copy`, `h264_metadata`) with a real one-frame timestamp gap produces 24 framemd5 rows whose pts jump from 11 to 13.
  - 330 refuses it, naming frame 12.
  - **321 accepted the same file** (T2-321): its hashes are identical and its count is right. This confirms 323 L1 was real.
- **The same join with a three-frame gap** is refused too.
- **Synthetic variants are all refused:**
  - a gap at the same frame count;
  - a duplicate pts;
  - a reorder;
  - a duration of 2 or 0;
  - a timebase of 1/30 or 1/120;
  - a second stream;
  - one frame short or one extra;
  - pts starting at 1;
  - no timebase line.
- **Positives accepted:** the clean join, an unmutated rewrite, and 120 frames of the kit's real 08 chapter render.
- `verification.json` records `framemd5_timing` (:741–742).

### L2 (free space) is fixed

- `launch()` checks free space on `D/jobs` before the inputs file is written with `'x'` (:895–913).
- It refuses 1 byte under 12 GiB, with no inputs file written and no guard called. At exactly 12 GiB it writes the inputs and calls the guard (L2a, L2b).
- Each picture job needs 10 GiB of admission plus its own upper estimate.

### L3 (labels and paths) is fixed

- `clean()` is applied to every stored reason.
- All 944 plan182 reasons that start with a path now give sheet labels that read the anchor: "plan182: original absolute anchor…" or "…cue-relative historical anchor…".
- On the stand-ins, no stored reason has a home-directory path, and no label has a tab or a character outside Latin-1.

### L4 (runtime) is fixed as asked

**Binding.** `runtime()` binds ffmpeg, ffprobe and bash, resolved on `/usr/bin:/bin`, plus the running python3, which must be `PYTHON`. `pillow_runtime()` draws a label, round-trips a PNG exactly and a JPEG by size and mode, and binds Pillow's files and the libraries ldd resolves for its extensions. Both run before either inputs file is written.

**Re-checked inside the jobs.** The verify and picture jobs run the runtime checks again under the guard and require the files to be bound, before any work.
- With the runtime as bound, each job reaches `film()` (R10, R12).
- With the PATH ffmpeg retargeted, each refuses before `film()` (R11, R13).

**Also refused:**
- the wrong python3;
- a missing ffprobe;
- a changed Pillow file;
- an unresolved or failing ldd for a Pillow extension (R1, R2, R6, R7, R9).

**Recorded.** The ffmpeg version line goes into `verification.json` and both `sampling.json` files.

**Not covered.** ffmpeg's and ffprobe's own shared libraries are not bound, so a change to one is not detected (R3, R5). That matches the precedent, `launch-final-verify05.py`, which binds the three tools, the python3 and Pillow's runtime files but not ffmpeg's libraries. It also matches 323's wording, and notes §7 states it.

## The 318 contract

- **Functions.** 16 functions are AST-identical to 321's, by my own `ast.dump` comparison: `sha`, `read`, `success`, `stable`, `shared`, `build313`, `master318`, `schedules`, `film`, `anchors`, `preview313`, `admitted`, `require`, `run`, `verified` and `balanced`.
  - The functions that changed are `frame_hashes`, `picture_plan`, `verify`, `plan_for`, `pictures`, `launch` and `main`. None reads 318's records.
- **Constants.** Among 321's constants, only `VERSION`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES` and `PICTURES_INPUTS` changed.
- **Conclusion.** Every 318 check reviewed in 323 holds unchanged: the 11-item guard command, summary keys and rows, the 90 staged links by `samefile`, render summaries, soundtracks and replay counts, and byte comparisons of schedules and takes.

## Notes (no change asked)

- **N1. The window constant is not cross-checked at runtime** (W3). If a window were removed from `TRANSITION_WINDOWS`, nothing in 330 would refuse, and its frames would silently drop from the plan. The constant lives in the pinned helper, though, and W0 and W2c (and CC2's unit tests) show it equals the records at `4fc4594d`. Deploying the helper at its pin is the protection.
- **N2. `verified()` binds the verify job's outputs at their current bytes**, the same pattern as M1. It is inherited unchanged from 321 and is outside this delta. The stakes are lower: those outputs are logs and framemd5 tables that nothing views. `verification.json` is read for its claims. The same per-output check would close it if wanted.
- **N3. Two full decodes for pictures.** Each picture job decodes the master up to its last selected frame, under the same 14,400 s timeout that 321 gave its single pass.

## Local checks and their limits

**Harnesses** (all outside the repo):

| Harness | sha256 | Cases |
|---|---|---|
| `regress332b.py` | `e23b9013…` | 48 cases: T0–T17, W0–W8, L2, L3, R0–R13, G0 |
| `chain332.py` | `460eaefb…` | C0–C9 |
| `fix330-proto.py` | `9b15684c…` | the M1 prototype only |
| `budget332.py` | `69930c47…` | the G0 plan counts and budget arithmetic |

**Tools.** ffmpeg and ffprobe 8.1.2, python 3.15.0, Pillow 13.0.0 (a private copy, for the drift cases).

**Fixtures.** Tiny lavfi H.264 clips; a 24-frame 4K synthetic master; 120 frames of the local 08 chapter render; local copies of Q's records, plan182 and timeline21 tables.

**Limits.**
- No real 318 master, preview313 record or current CSD layout exists locally, so the plan counts are stand-ins.
- macOS has no ldd and no CPU affinity, so both were faked.
- The guard is an emulator.
- Nothing ran on CSD.

## Still open

- **330's own fixes:** M1 here (CC2's 339, which needs its own review) and L1's CSD framemd5 check.
- **The runs:** the 318 master, then `check`, verify, and the overview and native picture jobs.
- **Root's review:** viewing every sheet and native frame.
- **Review that no flag records:**
  - motion;
  - joins, seen and heard;
  - fine sync;
  - audio review;
  - independent final AV review;
  - public 4K and uploaded-caption verification;
  - publication.

No audio heard or checked, here or by any case above.
