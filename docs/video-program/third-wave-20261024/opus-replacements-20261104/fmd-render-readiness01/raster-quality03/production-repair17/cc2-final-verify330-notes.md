# CC2 330: the final verifier with review 323's M1 and L1–L4 fixed

cc2, 2026-11-08. Files:
- `csd-final-verify330.py`, sha256 `4fc4594d4dccd58e407824fd7ccb59a5b786ba9c8c85dd38603916b669749437`, 956 lines;
- these notes.

**Status.** Prepared only. Nothing in 330 has run on the master, on CSD or anywhere else. Nothing was deployed, launched or rendered. No source, Beads or TODO file was edited. No picture, motion, listening, fine-sync, 4K-detail, caption-reading or independent AV acceptance is claimed. Every review flag that 330 writes is False.

**What is preserved.**
- `csd-final-verify321.py` (`0a956ec8…`) and `cc2-final-verify321-notes.md` (`c7e537a4…`) are unchanged. 330 is a new file, copied from 321 and then edited.
- `cc2-verifier-deployment328.md` (`4ac5b9f8…`) is unchanged.

**Scope.** 330 fixes CC0's review 323 findings M1 and L1–L4, and nothing else. N1–N3 are untouched.

The picture extraction is now two guarded jobs instead of one. Root asked for this so that the plan fits under the guard without dropping frames, and I coordinated it with CC0 (G0, below). D is the CSD root and Q is `D/qualification01/production-repair17`. Line references are to 330.

## 1. The fixes, finding by finding

### M1: root's 305 and 319 windows and the 07 instants are in the final plan

**The records, pinned.**
- `TRANSITION_SOURCES` (:189–211) holds 21 Q-relative records at their sha256:
  - the eight 305 review JSONs and their eight frame manifests;
  - the 05 manifest `gallery-transition-frames305/manifest.json`;
  - `cc1-gallery-transition-review319.json` and `.md`;
  - `gallery319-disposition305.json`;
  - `open-region-playback-second305.json`.
- `plan_for()` (:768–775) refuses unless every record is a file at its pin. The records are bound in each picture job's inputs and copied into `sampling.json`.

**Windows, sites and instants.**
- `TRANSITION_WINDOWS` (:213–232): the 31 windows, 1,932 frames, as CC0's table lists them.
- `GALLERY_SITES` (:235–236): the 319 `measured_curves` full-plate frame for C1–C17. C13's six panels share frame 8,859.
- `ONE_POINT_INSTANTS` (:238): the five 07 instants, frames 4858, 4872, 5099, 5355 and 5681.
- The focused tests (§4) show all three equal what the records say.
- Every manifest names `preview274-review305/preview/<chapter>.mp4`, so a window frame is the same chapter-local frame of the final (review 323 M1, "The frame numbers carry over").

**Samples and natives.** `picture_plan()` adds these before plan182 (:467–483), so a window frame's first reason, and therefore its sheet label, names its window:
- **Samples.**
  - Every window frame: `root window A-B, every frame (<record>)`.
  - Each 07 instant, ±2 frames.
- **Natives.**
  - Each non-05 window gets one native 4K frame at its middle (22 windows).
  - Each 319 site C1–C17 gets one at its full-plate frame. Every 05 window contains at least one site, and the plan refuses otherwise.
  - That gives 39 new natives and at least one native in each of the 31 windows.

**Coverage**, from the local estimate and the fixture (§4):
- all 1,932 window frames are sampled;
- 31 of 31 windows have a native;
- all 17 sites have a native;
- all 25 instant frames are sampled.

### G0 (CC0's message, root's order): the plan must fit under the guard without dropping frames

**The problem.**
- With M1, the whole plan's upper estimate is 3.56 GiB on local stand-ins that leave out preview313's picks and fades. CC0 measures 3.54–4.25 GiB.
- One job could therefore exceed the guard's 4 GiB native phase cap.
- Root's order: keep every frame and keep the 4 GiB guard unchanged.

**The fix: two jobs, fixed by kind.** This is CC0's suggestion, adopted.

| Job | Holds | Output |
|---|---|---|
| `csd-final-pictures330-overview` | every sample | 960×540 JPEGs and all sheets |
| `csd-final-pictures330-native` | every native | 3840×2160 PNGs |

- `PICTURE_PARTS` is at :80. The inputs are `Q/final-pictures330-<part>-inputs.json`. The guard command ends `pictures --part <part>`.

**The budget.**
- Each job's upper estimate must be at most `PICTURE_PART_BYTES` = 3.5 GiB (:87–90). That leaves 0.5 GiB under the cap for the guard's other counted growth: logs, the receipt and the D/work cache.
- `part_upper()` (:794–797) uses the same bound as 321:
  - 78 KiB per JPEG and 897 KiB per sheet (the lane's measured maxima);
  - raw RGB per native PNG;
  - plus 64 MiB for `plan.json`, the filter script and `sampling.json` (`growth_upper()` :636–640).
- A job over budget refuses in `check`, `launch-pictures` and the job itself (:933–936, :829–830). Nothing is regrouped, split further or dropped.

**Fixed membership.**
- Membership does not depend on counts.
- `plan_sha256` is the sha256 of the whole plan's JSON (`plan_digest()` :789–791).
- `check` and each launch print it. Both jobs write it into `plan.json` and `sampling.json`.

**Ordering.**
- `launch-pictures` launches the first part with no job folder (:946–952).
- The native launch and the native job call `part_done('overview', plan_sha256)` (:800–808). It refuses unless:
  - the overview job finished (`stable()`, which re-hashes every row the overview job bound);
  - its `sampling.json` names this same plan.
- The native inputs bind every file the overview job wrote, so the native guard re-hashes them at admission and at the end.

**The union proof.**
- Before extracting anything, the native job asserts that the overview job's samples equal the plan's samples, frames and reasons together, and that the overview job extracted no natives (:836–841). The native job's natives are the plan's natives.
- It writes `union` (parts, `plan_sha256`, counts, `frames_and_reasons_equal_plan: true`) into its `sampling.json`.

**The decode.**
- Each job runs one ffmpeg pass with `select='eq(n,…)'` over the whole master. There is no input seek, because `n` restarts after `-ss`. Each pass decodes up to its last selected frame, so the film is decoded twice where 321 decoded it once.
- Each job's graph is the single-output subset of 321's `split=2` graph (:848–855). It uses the same `-vsync 0`, `-q:v 2` and png `-compression_level 3` options.

**Retry rule.** Each version still gets one attempt per job.
- **Refusals before inputs are written spend nothing.** These include free space, Pillow, budget, record pins and every static gate. They can be retried once the cause is fixed.
- **A failed or refused job.** Once a job's inputs file exists, that job is spent for 330.
  - If the overview job fails, the native job can never launch, because `part_done` refuses.
  - If the native job fails, the overview outputs stay preserved, but 330's picture set is incomplete.
  - Either way, finishing the pictures needs a new version. Whether a later version may reuse a successful overview job would be that version's own question, for review.

### L1: exact framemd5 timing

- `frame_hashes(path, frames)` (:556–579) requires all of the following:
  - exactly one timebase line, `#tb 0: 1/60`;
  - stream 0 only;
  - six columns;
  - exactly `frames` rows;
  - pts equal to 0..n−1 in order, so a gap, repeat or reorder is named by its first bad frame;
  - every duration equal to 1.
- It is called with each chapter's frame count and with the master's 133,662 frames (:706, :709).
- `verification.json` records `framemd5_timing` with timebase 1/60, pts 0..n−1 in order, duration 1, and the 16 files checked (:741–742).

### L2: free space before an inputs file is spent

- `launch()` (:895–913) checks `shutil.disk_usage(D/'jobs').free` before it writes the inputs with `'x'`, as 318 does (:807–808 there).
  - `launch-verify` needs 12 GiB, 318's `LAUNCH_FREE_BYTES`.
  - Each picture job needs 10 GiB of admission plus that job's upper estimate.
- A refusal writes nothing and starts no guard.

### L3: labels without home prefixes

- `clean()` (:436–440) is applied to every reason the plan stores. It rewrites plan182's absolute checkout prefix to `Q/` and elides any other home path.
- `sheet_label()` (:443–447) drops plan182's `Q/<snapshot> ` prefix, so the 944 affected labels now read the anchor (`plan182: original absolute anchor: …` or `plan182: cue-relative historical anchor…`).
- `sheet_label()` also replaces tabs with ` | ` and writes `-` for `−`. It maps any remaining character outside Latin-1 to `?`.
- Sheets use it (:871). No reason in the 330 plan contains a home path.

### L4: the runtime is bound and exercised

**`runtime()`** (:589–602), the `launch-final-verify05.py` precedent:
- resolves ffmpeg, ffprobe and bash on `/usr/bin:/bin`;
- takes `sys.executable`, which must be the `/usr/bin/python3` the guard runs;
- reads ffmpeg's first `-version` line.

**`pillow_runtime()`** (:605–633), the precedent from `pictures-after-preview16.py` :6–31:
- draws a label through `sheet_label()` with the default font;
- round-trips a PNG exactly and a JPEG by size and mode;
- checks that label pixels exist;
- returns Pillow's package files and the libraries `/usr/bin/ldd` resolves for its extensions. An unresolved library refuses.

**Where they run.**
- `launch()` runs both before either inputs file is written and binds their files into both inputs files.
- The verify job requires the four tools bound (:675–676).
- Each picture job re-runs both under the guard and requires their files bound (:821–823).
- The ffmpeg version line is recorded in `verification.json` and both `sampling.json` files.

**Not included.** ffmpeg's and ffprobe's own shared libraries are not bound, because the finding asks for the four tools and Pillow's files.

## 2. 318 compatibility

These functions, which read 318's records and the guard records, are AST-identical to 321's: `sha`, `read`, `success`, `stable`, `shared`, `build313`, `master318`, `schedules`, `film`, `anchors`, `preview313`, `admitted`, `require`, `run`, `verified`, `balanced`.

Among 321's constants, only `VERSION`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES` and `PICTURES_INPUTS` changed. The 318 contract that 330 reads is therefore exactly 321's: the guard command, the summary keys, the rows, the source jobs and the boundary replay.

The fixture (§4) still refuses 318-side faults at the 321 points:
- a 09 row on binary 268;
- a replay that is not bit-identical;
- a retained chapter labelled combined;
- a wrong replay count;
- a retained WAV that differs from the retained take;
- a staged table that differs from timeline269;
- a changed master input.

## 3. Numbers

| Plan | Samples | Natives | Sheets | Upper, whole plan | Overview job | Native job |
|---|---|---|---|---|---|---|
| 321, local estimate | 5,770 | 52 | 722 | 2.25 GiB | — | — |
| 330, local estimate (timeline21 tables, plan182, previous and 268-era layouts; no preview313 picks or fades, exit at 2093) | 7,645 | 91 | 956 | 3.56 GiB | 1.45 GiB | 2.17 GiB |
| 330, fixture (real plan182; fixture layouts and preview313 sampling) | 8,247 | 88 | 1,031 | — | 1.56 GiB | 2.10 GiB |

- **Typical size.** With local medians (40 KB per JPEG, 233 KB per sheet, an assumed 3 MB per PNG), the 330 estimate is 0.77 GiB.
- **Free space.** The estimated free-space needs at launch are 12 GiB for verify, about 11.5 GiB for the overview job and about 12.2 GiB for the native job. CSD had 100.1 GiB free on `D/jobs` at the 328 probe.
- **Not hard bounds.** The JPEG and sheet maxima are measured maxima. Raw RGB is effectively the PNG ceiling for these frames.
- **preview313's addition is still unknown.** The `check` of the real plan prints both jobs' estimates and refuses if either exceeds 3.5 GiB.

## 4. Focused tests (scratch, not in the repo; nothing real decoded or extracted)

### `unit_test330.py`: 59/59, local files only

**M1.**
- The 21 records are at their pins.
- Each 305 review's `intervals_inclusive` and its manifest's frames equal `TRANSITION_WINDOWS`, and the review's `manifest_sha256` equals the pin.
- The 05 windows equal 319's `coverage.windows`.
- Every manifest names its `preview274-review305/preview/<chapter>.mp4`.
- The sites equal 319's `measured_curves` (C13.1–C13.6 share 8,859).
- The disposition binds 319 at its pin.
- The instants equal `round(observed_seconds × 60)`.

**L1, positive.**
- Real ffmpeg 8.1.2 framemd5 of a finite 6-frame 60/1 lavfi test pattern passes.
- A synthetic 133,662-frame body passes.

**L1, negative.**
- Real ffmpeg output refused:
  - a 30/1 source (`#tb 0: 1/30`);
  - a real `setpts='N+gte(N,3)'` gap (pts 0, 1, 2, 4, 5, 6).
- Synthetic files refused:
  - no `#tb` line;
  - a second `#tb` line;
  - `#tb 0: 1/120`;
  - pts starting at 1;
  - a missing pts;
  - a duplicate pts;
  - a reorder;
  - one frame short;
  - one frame extra;
  - duration 2;
  - duration 0;
  - a stream-1 row;
  - a five-column row;
  - a one-frame pts gap at the 01/02 join, refused naming frame 6311.

**L3.**
- Exactly 944 plan182 first reasons start with a home path, and `clean()` leaves none in any of the 9,483 plan182 reasons.
- The 944 labels read the anchor.
- No label has a tab, a home path or a character outside Latin-1.

**Budget.** The `growth_upper` and `part_upper` arithmetic, and the budget constants.

**L4.**
- `pillow_runtime()` passes on local Pillow 13.0.0, with an otool stand-in for `ldd`, which macOS lacks.
- A Pillow that draws no text is refused.
- `runtime()` refuses tools that are not on `/usr/bin:/bin`, and a python3 other than `PYTHON`.

**318.** The AST identity in §2.

### `fixture_test330.py`: 62/62

**What the fixture is.** A synthetic CSD tree shaped like 318's outputs, at the real chapter frame counts (133,662 frames). The real plan182 and the 21 real records are copied at their pins. The helper's text is unchanged except for the CSD root.
- **Real:** the shared verifier, the caption screen, PIL, and the local tools `runtime()` binds.
- **Faked:** ffmpeg and ffprobe runs, the strict script, the guard, CPU affinity, free space and `ldd`.

**Gates.**
- `check` passes and writes nothing.
- The seven 318-side refusals of §2.
- A changed record and a missing record are each refused by name.
- A 1 GiB budget refuses the overview job and a 2 GiB budget refuses the native job, with nothing regrouped.

**L2.**
- `launch-verify` is refused at 1 byte under 12 GiB, with no inputs written and no guard called. It passes at exactly 12 GiB.
- The overview launch is refused 1 byte under 10 GiB plus its estimate, with no inputs written.

**L4.**
- The verify inputs bind the four resolved tools and 132 Pillow files and libraries.
- Verify refuses an unbound tool before any output.
- `launch-pictures` refuses a Pillow that draws no label, with no inputs written and no guard called.

**L1, end to end.**
- Verify refuses a master whose pts jump one frame at the 01/02 join while its hashes and count are intact.
- Verify refuses a chapter framemd5 at 1/30.
- Verify still refuses 321's tampered master frame.

**Full positive run.**
- `verification.json` shows 133,662 identical frames, the `framemd5_timing` record, the ffmpeg version line and all flags False.
- The overview job runs, then the native launch, which is refused while the overview receipt is unfinished.
- The native inputs bind the overview job's files. A third launch is refused.
- The native job passes and its union record holds.
- In `sampling.json`:
  - all 1,932 window frames sampled with their window reason;
  - 31 windows with a native;
  - 17 site natives;
  - 25 instant frames;
  - `ceil(samples/8)` sheets, all from the overview job;
  - no home path in any reason;
  - the records, runtime and Pillow recorded;
  - both jobs within 3.5 GiB;
  - all flags False.

**Consistent forgery.**
- An overview record naming another plan is refused at the native launch.
- An overview record with one changed reason, bound at the native launch, is refused by the native job's union check before any native frame is extracted.

### `estimate_plan330.py`

The numbers in §3, computed with 330's own `picture_plan()`, `growth_upper()` and `part_upper()`.

### `probe330.py`: read-only on CSD, 2026-11-08 about 20:14 UTC

**What ran.** 330's own `runtime()` and `pillow_runtime()`, with the helpers they call, copied by AST and fed to `/usr/bin/python3 -I -B -` over ssh under `nice -n 19`. Nothing was written. ldd only reads, and Pillow worked in memory.

**Results.**
- **Tools.** ffmpeg 8.0.1-3ubuntu2. ffmpeg, ffprobe and bash are in `/usr/bin`, and python3 resolves to `/usr/bin/python3.14`.
- **Pillow.** Pillow 12.1.1 from `/usr/lib/python3/dist-packages`. The label pixels are present and the PNG round trip is exact; the JPEG round trip also passes.
- **Libraries.** `ldd` resolves every library of its 7 extensions, 35 system libraries in all.
- **Size.** 152 files, 28.7 MB, would be bound into each inputs file.

This tests the runtime only. It is not a run of 330.

## 5. Deploying to CSD (root's)

These must be in Q on CSD at these exact bytes before `check`. It is the 328 list with 330 in place of 321, plus the 21 records.

**Files.**

| Q file | sha256 |
|---|---|
| `csd-final-verify330.py` | `4fc4594d4dccd58e407824fd7ccb59a5b786ba9c8c85dd38603916b669749437` |
| `cc0-combined-master318.py` | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` |
| `verify_stage09_final.py` | `f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94` |
| `verify_video_preserve_logs01.sh` | `3f4c68dbdcb02c1c90ff03cb4f20afc36792b7f20c6cc111a5eef21617a11410` |
| `caption-interval-audit01.py` | `3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4` |
| `final-picture-plan182.json` | `20a36f91989e0f1e0ca5149a74483ec22fe3728be44b872fc31a0859747b962a` |

- The local sources for all but the first row are listed in `cc2-verifier-deployment328.md` §1.
- 330 and the 21 records below are in this folder locally.

**Records**, Q-relative, at the hashes in `TRANSITION_SOURCES` (:189–211):
- `bump-`, `color-`, `one-point-`, `rigidity-`, `counting-opening-`, `counting-`, `residue-` and `continuation-transition-review305.json`, plus each one's `…-frames305/manifest.json`;
- `gallery-transition-frames305/manifest.json`;
- `gallery319-disposition305.json`;
- `cc1-gallery-transition-review319.json` and `cc1-gallery-transition-review319.md`;
- `open-region-playback-second305.json`.

Only these files are pinned: the frames and sheets in those folders are not needed. The manifests and records name absolute local paths. That matters only for hygiene if they are committed; 330 reads only their hashes, and copies only their Q-relative names into its records.

**After deploying.** Hash every file in Q, read-only, and compare it with these pins. The comparison is raw sha256, with no line-ending or permission normalization.

**CSD runtime facts** (328 and the probe above):
- No hardware second opinion. CSD has no `nvidia-smi` and no `/dev/nvidia0`, so the strict script's software `-xerror` decode decides alone.
- ffmpeg 8.0.1 warns that `-filter_complex_script` and `-vsync` are deprecated, but exits 0.
- The jobs pin themselves to CPUs 0–15, which the native lane (preview313) also uses. Root should confirm the lane is free before each launch.

## 6. Order of runs (root's)

1. **Wait for 318.** 313, then the permit, then 317, then 318.
2. **`check`.** It prints counts, `plan_sha256`, both picture jobs' estimates and free-space needs, and the free space now.
3. **`launch-verify`.** The verify job runs.
4. **`launch-pictures`.** This launches the overview job.
5. **`launch-pictures` again,** after the overview job succeeds. This launches the native job.
6. **Viewing.** Root views every sheet and every native frame.

330 accepts nothing.

## 7. Limits

- **No acceptance.** Nothing here is a review of pictures, motion, sound, sync or AV, and no flag says otherwise.
- **The upper estimate.** It counts the real plan only once CSD's `check` runs, after preview313 exists. If preview313 adds enough natives to put the native job over 3.5 GiB (about 145 natives in all), `check` and `launch-pictures` refuse. Root decides what follows; 330 does not regroup.
- **Two decodes.** The two picture jobs decode the master twice.
- **Not fixed (outside M1 and L1–L4).**
  - ffmpeg's and ffprobe's shared libraries are not bound.
  - The strict script's other tools are not bound either: awk, grep, sed and the rest.
- **A local cache file.** Local compile checks wrote one byte-code cache file, `__pycache__/csd-final-verify330.cpython-315.pyc`, beside the helper. It is git-ignored and was left in place, because no deletion is authorized. Later checks compiled in memory only.
