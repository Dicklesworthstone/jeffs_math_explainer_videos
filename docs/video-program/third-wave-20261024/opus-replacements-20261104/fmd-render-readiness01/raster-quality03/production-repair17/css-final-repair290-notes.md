# CSS final-render adaptation 290: notes and copy mapping

Prepared only. Nothing was deployed, transferred, launched or written on CSD or CSS. All remote work
was read-only SSH. CSS here means superserver; D is the CSD workspace, `/data/projects/jmev-stroke-probe-20261106`;
R is the CSS workspace, `/data/projects/jmev-css-render-worker-20261107`; Q is CSD's
`D/qualification01/production-repair17`.

| File | SHA-256 | Notes |
|---|---|---|
| `css-final-repair290.py` | `720e63a66d9e67d04c1cd99d2131d76401e42a5e2d60240fb1afc9d3a93ac8f2` | 720 lines; parsed; partly exercised (below) |
| base `csd-reviewed-final287.py` | `f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510` | unchanged |
| base `csd-launch-final287.py` | `80d9ab1159251ba372d2e7199761d2d6fba20113ea7e3c69e9f2bb1217597bba` | unchanged |
| base `cc0-final287-changes.md` | `4b8281f73b524e439f2837b37210829d9bdec17451589f88e3d368eafce45112` | read in full; its CSS hand-off list is carried |

The CSS render runs the same binary 268 with the same command line as 287. That covers
`--res 3840x2160 --fps 60 --crf 16 --preset veryfast --lint`, score21 stems and cwd
`D/work/production-repo07`. It uses the same 105 Frederick Surrey WAVs, which are byte-identical in
assembly19 and rebind269, and the same timeline269 schedule tables. It sits behind the same permit and
preview gates. On CSS, D is the existing alias into `R/work/frozen-runtime75`, so every CSD path in
287, a permit and the binary resolves to the frozen copy.

## 1. Kept exactly from 287

- **Binary, schedule and narration identity:**
  - binary 268 at `983954c5…`, checked against build268's qualification;
  - all 45 timeline269 tables equal timeline21 after the one assembly19→rebind269 path mapping, for every qualified chapter;
  - `verify_tables` is semantically identical to 287's and 78's. A diff shows only joined lines and a comment.
- **Score:** every score21 file matches its summary hash, `all_qa_passed`, the chapter list equals build268's, and each chapter's stem QA passed. The score rows are checked against the stems and the `.parts.tsv` mapping.
- **Permit:** `binary_sha256`, `chapter`, `actually_reviewed_pictures` and `actually_reviewed_motion` must be true. `unresolved_defects == []`, a non-empty reviewer and observations, `timeline_sha256`, the evidence and `source_binding_evidence` hashes, the frames header and the motion intervals are all still required. The permit path must be absolute and free of symlinks.
- **Preview binding:** `preview_binding()` is unchanged. For preview243, it checks:
  - the worker command and hash;
  - the manifest equals its receipt;
  - binary 238, the worker and build238's source copy and qualification appear in that manifest as they are;
  - the 06/08 tables equal timeline269 after the 242→269 mapping;
  - the scene file is equal in 238 and 268;
  - the exact five-file delta;
  - the one named `build238-to-build268-delta` item for binding282 at `1fbcedef…`.

  `PREVIEW274 = None` stays, so the nine other chapters fail closed until a later 287 names CC3's actual preview274.
- **Render checks:**
  - a fresh guard admission with stop 8 GiB and growth 4 GiB (290 also asserts admission 10 GiB);
  - every bound input is re-hashed before and after the render;
  - one MP4: 3840×2160, 60/1, frame count equal to the permit's, h264/yuv420p, AAC 48 kHz stereo;
  - x264 `rc=crf` and `crf=16.0`;
  - the soundtrack capture has frames × 800 samples;
  - `check()` is repeated after the render and must return the same values;
  - `render-summary.json` is opened with `x`.

## 2. Changes against 287, each needed on CSS

| # | 287 on CSD | 290 on CSS | Why |
|---|---|---|---|
| 1 | Jobs, inputs and Q under the CSD root | Jobs under `R/jobs`, inputs under `R/qualification01`; CSD reads through D | CSS precedent (75/76/78/96) |
| 2 | Guard `guard_csd02.py` | `R/guard_css74.py`; I read its pinned owner and base guards | Same policy: 10 GiB admission, 8 GiB stop, 4 GiB per native job, one exclusive lane lock |
| 3 | Encoder prerequisite `csd-raster-encoder-stop13` | `css-encoder-stop79` | The CSS lane's own encoder stop, as in 78 (receipt finished, exit 0) |
| 4 | `--narration D/jobs/csd-review-repair-rebind269/narration` | `R/jobs/css-repair-delivery290/narration`, made by the `delivery` mode | delivery76's pattern: a copy whose `report.json` line sources point at their resolved frozen paths (hash-checked), with the matching `assembly.json` `report_sha256`. Everything else is asserted byte-equal, and the 105 WAVs equal assembly19's. `facts` and `chapters` are checked against build268. The native `narration_delivery.py --consumer render` must report 105/105 `external-ui` before any render. |
| 5 | `run()` binds all 12,600 rows of `review-repair-timeline269-inputs.json` | `carried_rows()`: every row of the timeline269 and preview243 manifests that exists on CSS must match, and binary 268, the workspace, rebind269 and build268 must be among them. The delivery290 inputs are also bound. | The other 9,785 rows are not on CSS and are not runtime reads (§4). The 2,815 present rows include numpy, scipy, python3.14, ffmpeg and ffprobe. 0 differ today. |
| 6 | Refuses CSD 244/278 jobs for the chapter | On CSD, `manifest --permit` refuses while a `csd-reviewed-final-<chapter>` 244, 278 or 287 job exists. On CSS, `check()` refuses any other `css-reviewed-final-<chapter>*` job. | CSS cannot see CSD's jobs, so the CSD check is point-in-time only: **root must give each chapter to one host** |
| 7 | Permit anywhere symlink-free on CSD | The CSS copy under `R/qualification01/repair290-permits/`; its evidence at the same D paths through the alias | A path through the alias is not symlink-free. One permit file with the same bytes serves both hosts. |
| 8 | Job `csd-reviewed-final-<ch>287`, inputs `final-<ch>-inputs287.json` | `css-reviewed-final-<ch>290`, `final-<ch>-inputs290.json` | New paths only |
| 9 | Launcher is a separate file | `launch` mode | It binds what 287's launcher binds: whole timeline269 and preview243 job folders, evidence, binding, narration, score, encoder stop, plus the CSS transfer and delivery receipts. It also has 96's optional `--after` wait. |
| 10 | Summary fields | Adds `host='css'`, the 290 and 287 hashes, the carried-row count and manifest hashes, and the narration path | Provenance |

The threads stay at 16, as in 287's launcher and 78/96.

## 3. Copy mapping: CSD → CSS

Rule: each file goes to `R/work/frozen-runtime75/<its absolute CSD path>`, so D on CSS serves it at the
path every gate names. The one exception is a permit, which goes to `R/qualification01/repair290-permits/<name>`.
Files are created with `x`; an existing destination must already hold the same bytes, or the job is
refused before reading. The transfer job stages everything inside its own job folder and places files
only after the whole stream verifies. I tested this on a scratch tree: a truncated stream placed nothing,
trailing bytes were refused, a repeat wrote nothing, a tampered destination was refused, and the file
mode was kept.

**A. Runtime copy**, one job `css-repair-transfer290`. Measured 2026-11-08 while timeline269 was still running:

| CSD source | Files | Bytes | Read by |
|---|---:|---:|---|
| `D/work/bin/review-repair268` | 1 | 18,465,816 | render, delivery, `check()` |
| `D/work/bin/review-repair238` | 1 | 18,451,464 | `preview_binding()` |
| `D/work/review-repair268/workspace/**` | 72 | 2,035,988 | the binary's topic, which is its only embedded runtime path, and the delivery gate tools |
| `D/jobs/csd-review-repair-build268/**` | 18 | 41,475 | qualification, source copy, receipt |
| `D/jobs/csd-review-repair-build238/**` | 16 | 38,371 | preview binding |
| `D/jobs/csd-review-repair-rebind269/**` (narration 109 files, ~96 MB) | 117 | 95,930,289 | delivery source, receipt |
| `D/jobs/csd-review-repair-timeline269/**` | 36 now, final count after exit | 19,099,363 now | 45 tables, `comparison.json`, receipt |
| `D/jobs/csd-review-repair-preview243/**` | 1,064 | 149,790,898 | preview receipt and tables; 287's launcher binds the folder |
| `Q/review-repair-timeline269-inputs.json`, `Q/review-repair-preview243-inputs.json`, `Q/review-repair-preview243.py` | 3 | 5,608,749 | manifest/receipt equality, carried rows |
| `Q/cc1-preview-to-final-binding282.md` | 1 | 22,755 | binding282 item; **not on CSD yet** |
| `D/jobs/csd-production-timeline21/timeline-output/*.tsv` | 45 (36 new; 03/11/13 already present) | 42,081 | timeline269 = timeline21 check |
| **Total** | **1,374** | **≈ 310 MB** | |

The exact list is `runtime_sources()`. The `manifest` mode refuses to print it unless all of these hold:

- the five CSD jobs finished cleanly;
- the timeline269 and preview243 manifests are the ones their receipts name;
- binary 268, the workspace, rebind269 and build268 are byte-equal to timeline269's frozen rows;
- binary 238, the worker and build238 are byte-equal to preview243's rows;
- all 45 timeline269 tables equal timeline21 after the mapping;
- preview243's 06/08 tables equal timeline269's;
- 282 is in Q at its pinned hash.

**B. Per permit**, one job `css-repair-permit290-<chapter>[suffix]`: the permit JSON plus every
`evidence` and `source_binding_evidence` file it names, at their hashes. Each must be under D. A
re-issued permit takes a new job suffix.

**Already on CSS at matching hashes, not copied** (checked read-only today):

- score21: all 44 summary files, including stem QA for 15 chapters and the 08/09/11/13/14 parts;
- assembly19 narration (109 files);
- all 105 report sources, which are identical (source, hash) pairs in assembly19 and rebind269;
- both generation receipts and all 210 unit WAV references;
- `production-repo07` (316 files);
- the timeline21 receipt;
- the CSS guard, runtime75 and encoder-stop79 receipts;
- the alias. It is exact, and nothing here replaces it.

**Deliberately not copied:** the 9,785 timeline269 rows that are absent from CSS. They include:

- `work/tree` (4,184) and `inputs/frozen` (4,126);
- older jobs and `work/` folders;
- 174 `production-repo07` files: the stage05/06 ElevenLabs downloads and requests, the native-stage07 workspace and the cue-repair folder.

The evidence that they are not runtime reads:

- binary 268 embeds only one absolute path that is not a `.rs` source path, the workspace topic;
- the 268 workspace's `coherent_narration.py`, `narration_delivery.py` and `narration_cache.py` are byte-identical to production19's, which passed delivery76 on CSS against the same receipts without those 174 files.

`delivery` re-proves this natively before any render. If the gate needs a missing file, it stops there.

## 4. Order of operations for root

None of this has been run. Each step is a separate root decision.

1. Wait for timeline269 to exit 0 with `comparison.json`. It had 24 of 45 tables at the last check.
2. Put `cc1-preview-to-final-binding282.md` (`1fbcedef…`) into CSD's Q. **287 itself needs this on CSD too:** its `DELTA_EVIDENCE` is that path, and today the file is missing there.
3. Place `css-final-repair290.py` with identical bytes in CSD's Q and in `R/qualification01/`.
4. On CSD, run `/usr/bin/python3 -I -B Q/css-final-repair290.py manifest`, with stdout written to a new file, for example `Q/css-repair-runtime290-manifest.json`. Note its SHA-256, then copy it unchanged to `R/qualification01/`.
5. On CSS, run `/usr/bin/python3 -I -B R/qualification01/css-final-repair290.py freeze-transfer --job css-repair-transfer290 --manifest R/qualification01/css-repair-runtime290-manifest.json`. It prints the guarded `receive` command.
6. Relay the stream:
   ```
   ssh csd '/usr/bin/python3 -I -B Q/css-final-repair290.py send --manifest Q/css-repair-runtime290-manifest.json --manifest-sha <sha>' | ssh css '<printed command>'
   ```
   This uses 75's stdin pattern. The guard child inherits stdin.
7. On CSS, run `… preflight`. Only `css-repair-delivery290` and `permits` should still show missing. Then run `… launch-delivery` and check its `delivery-result.json`.
8. For each reviewed chapter (only 06/08 can pass today):
   - root writes the permit on CSD;
   - `… manifest --chapter <ch> --permit <absolute CSD permit>` writes a new file; copy it to `R/qualification01/`;
   - `… freeze-transfer --job css-repair-permit290-<ch-with-hyphens>`, then the relay as in step 6;
   - `… launch --chapter <ch> --permit R/qualification01/repair290-permits/<name> --permit-transfer <that job> [--after <previous CSS job>]`.

A failed transfer or delivery leaves FROZEN unchanged, but the fixed job names stay taken. A retry is
a new helper version, as with 244/278/287.

**CSS capacity** (read-only, today):

| Item | Value |
|---|---|
| Free space | 24.25 GiB |
| Load | about 15–17, from other processes |
| CPUs | 64 |
| Python | 3.14.4 |
| Final 11 (78) / final 13 (96) | grew 0.06 / 0.04 GiB |
| This transfer's expected growth | about 0.6 GiB (staging plus placement) of the 4 GiB per-job cap |
| Target growth since the guard's baseline | 1.28 GiB of 20, about 1.6 after the transfer |

Only one native job runs at a time on CSS.

## 5. What was checked, and how

- **CSD, read-only:**
  - the helper's own `manifest` mode refused correctly on timeline269;
  - a scratch harness relaxed only timeline269's finished test. Every other runtime check passed on actual bytes, leaving three open items: `comparison.json`, the 21 tables not yet written, and 282 missing from Q;
  - written tables 01–08 equal timeline21 after the mapping, and preview243's 06/08 tables equal timeline269's.
- **CSS, read-only:**
  - the helper's `preflight` lists exactly the copy set above as missing;
  - the guard source (base `819746fb…`, owner `1c19a9b8…`) confirms the stdin pass-through, the lane lock, `job.mkdir()` and the receipt keys that `success()` reads;
  - all present manifest rows match.
- **Local scratch only:** `manifest_rows` → `send` → `receive` round trip, as in §3.

## 6. Not established

`check()`, `launch`, `render`, `delivery` and `launch-delivery` have not been executed anywhere. They
need the copied runtime, a finished timeline269 and a real permit. There is no claim of compiled,
rendered, pixel, motion, audio or listening equivalence between the hosts. A CSS chapter is still only
a technical scored 4K60 chapter: full decode, master replay, final pictures, motion, audio and captions,
listening, fine sync, the independent final AV check and publication all remain. The movie stays in
`R/jobs/<job>/render/`; export follows 80/97 and is not covered here. The carried-row gate will also
refuse if CSS's system Python, numpy, scipy or ffmpeg change before launch. That is intended.
