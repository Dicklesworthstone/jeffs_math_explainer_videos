# CC1 review 314: the 313 build, rebind, timeline and targeted 09 preview helpers

CC1, 2026-11-08, 19:05Z. This is an independent review of CC0's eight 313 helpers, read in full after CC0 froze them. I wrote none of them.

**Not done:**
- no launch, build, compile, render, deploy, permit or recipe;
- no Beads, TODO or canonical edits;
- no edits to 313, 310, 312 or any other file;
- nothing written on CSD.

**What I did run on CSD (read-only):**
- re-hashing;
- the staged helpers' own `closure()`, `check_base()`, `lit_counts()` and `zoom_probe()` functions, loaded without calling `main()`;
- the host's pinned rustfmt in `--check` mode.

Everything else ran locally in session scratch space. Narration is the ElevenLabs Frederick Surrey set and nothing here changes it. I heard nothing.

| File | SHA-256 |
|---|---|
| `cc0-repair313-notes.md` | `d368f726bac1ba747094bc04df5c6dcb1d38cb2815d86a864dd4964bf8daad7f` |
| `review-repair-build313.py` | `ff1ff0b8619cce99d87f2a5c5d44da1ac106a5cb70f698af2e93b64f3248b86a` |
| `launch-review-repair313.py` | `d384d4d8cdc6da3dd2f17a568748cc585e7139025f60028ea9823c11fd0141e6` |
| `review-repair-after-build313.py` | `f28a15ab6a33fd330a11cdb904e24753aa5a6ff88c4f940447bdfcb9461ded0b` |
| `review-repair-rebind313.py` | `0e04aa9eaa4f824f385459f24c7ca682f4df497d3fd424e756d34332486c2f8d` |
| `review-repair-timeline313.py` | `1034931710af7c46c79b09c391122bd22a38471d1d1f73fd501f09ec672d474f` |
| `review-repair-preview313.py` | `67d780d83bf05fbcd0285f0599197649f8ce8813ef6599a5da932301dc354e50` |
| `launch-review-repair-preview313.py` | `c983ecbff973cf5c300ec1f8b8837a175b0a41f8aa294bd7826f9a1b4d42806a` |
| `preview-after-timeline313.py` | `96af8d850d8dd05a7dea6062115e17271a759f38923fbf3710b794d3263ed7ec` |

These equal the notes, root's `repair313-stage305.json` (`97881557…`) and the staged copies on CSD. On CSD I also re-hashed `boundary-guide-repair310.rs` (`bda7d931…`), the notes, and `prepare-assembly19.py` (`2dec9a7a…`).

**The pins between the helpers are correct.** I checked each by grep:
- the build launcher pins the build worker;
- the controller pins the rebind and timeline workers;
- `preview-after-timeline313.py` pins the preview launcher and worker;
- 310's hash is pinned in the build launcher, the build worker, rebind313 and the preview worker.

**Nothing has run yet.** At 19:00Z CSD had none of these:
- `review-repair-recipe313.json`;
- any 313 inputs manifest;
- any `csd-review-repair-*313` job;
- `work/review-repair313`;
- `work/bin/review-repair313`.

## Verdict

**I found no High or Medium defect.** Root's input-closure finding is fixed, and I confirmed the fix independently on the build host:
- every row that build268 and rebind269 bound is re-verified and carried into build313's inputs;
- the timeline269 and preview274 closures are carried by timeline313 and preview313;
- no exception is needed, because no bound row has been mutated.

The bindings, guards, unchanged-WAV checks and the 09 timeline comparator are strict. The targeted 09 coverage includes:
- the exit, frames 2080–2106;
- the `|g| ≤ M` arrival at 2326;
- 69.67, 90.28 and 104 s;
- the new 115/116/117 s anchor (frames 6900/6960/7020), both as picture picks and as probe anchors;
- the 39–133 s persistence sweep.

Four Low items and one note follow. None blocks recipe313.

## What I checked, and how

### 1. Input closure (root's review313 finding)

**`closure()` in the build launcher** (`:35–45`) does three things:
- it checks that each manifest equals its receipt's `inputs_sha256`;
- it re-hashes every row;
- it carries the manifest and every row.

It is applied to `review-repair-build268-inputs.json` and `review-repair-rebind269-inputs.json` (`:72`, `:76`). The controller (`:18–22`) and the preview launcher (`:48–69`) expand their prior manifests the same way.

**On CSD, from 18:48 to 19:00Z, read-only:**

- **Every prior row still holds its bound bytes.**
  - The union of the five prior manifests is 12,669 paths: timeline21, build268, rebind269, timeline269 and preview274.
  - All 12,669 match the hashes their manifests recorded (1.29 GB hashed).
  - No path is recorded with two different hashes.
- **The staged launcher's own `closure()` passes on both manifests.**
  - It yields 12,320 and 12,411 paths, 12,411 together.
  - With the launcher's other paths, `main()` would bind 12,420 paths, plus recipe313 and this review.
  - None of the 12,304 timeline21 rows is missing, nor any of build268's 12,319 or rebind269's 12,410. That covers launch268's timeline21 rows and the production-repair19 workspace.
- **The rest is carried later in the chain.**
  - Timeline269 has 184 rows, and preview274 has 253, that are not in build313's set. They are timeline21's and the later jobs' folders.
  - The controller's timeline313 launch carries timeline269's closure, and the preview launcher carries preview274's.
  - So by the end of the chain all 12,669 prior paths are bound.
- **106 bound rows are symlinks.**
  - 105 are the WAVs in `jobs/csd-raster-media15/narration/`, which link into `stage08-picture-rebind01/narration/`. The links were made by 2026-11-06 21:21Z, before timeline21 ran.
  - The other is `/usr/bin/timeout`.
  - Following them gives the bound hashes. `closure()` (`Path.is_file`, `open`) and the guard's `check_inputs` (`open`) both follow links, so these rows pass and are bound by their target bytes.
- **The guard re-checks every row at the start and at the end.** The guard's `check_inputs` (`guard_csd02.py` `f1cae1db…` → its pinned phase runner `819746fb…`, `:61–73`, called at `:136` and `:168`) re-hashes every row when the job starts and again when it ends. A row that changes mid-job sets `error`, and then every 313 `success()` refuses the job.
- **No job in the chain writes to a path it binds.**
  - The chain writes to `work/target`, `work/tmp`, `work/cache`, `work/review-repair313`, `work/bin/review-repair313` and its own job folders. None of them holds a bound row.
  - Rebind313 rewrites `narration_units.json` in the 313 tree, and the controller excludes that path from rebind313's inputs (`:40`), as after-build269 did. Timeline313 then binds the rewritten file.
- **Spot checks.** These dependencies are rows of build268's manifest, so they are carried:
  - assembly19's receipt and `assembly.json`;
  - `rustfmt.toml`;
  - `engine-budget.toml`;
  - build05's `qualification.json`;
  - `guard_csd02.py`.

### 2. Build313

**`check_base()`, run on CSD unmodified, passes against the live 268 workspace.** The workspace holds 72 files:
- build268's 70 at their source-copy hashes, except `narration_units.json`, which is at rebind269's canonical hash;
- the backup, equal to build268's units;
- the review269 map.

**The reconstruction reproduces build268's tree.** I copied the whole live workspace locally, read-only, and ran `reconstruct()`'s logic on it:
- all 70 files reproduce build268's source-copy (`81d28272…`) byte for byte;
- the only change is `boundary.rs` → `bda7d931…`.

CC0's local test had only 64 of the files.

**The new format check should pass.** It is `cargo fmt --check` instead of 268's rewriting `cargo fmt`. The host's pinned rustfmt is `1.10.0-nightly (9085017724 2026-08-30)` from `nightly-2026-08-31`. Run with the host's `rustfmt.toml` (edition 2024, `max_width` 120), `--check` exits 0 on 310 and on its base 52. The other 68 files are the bytes 268's own format pass left.

**The other gates match build268's:**
- the toolchain hashes;
- the environment;
- CPUs 0–3;
- 233 passed with 1 ignored;
- the plate-fade probe;
- Clippy `-D warnings`;
- the new metadata;
- facts `7d528509…`;
- the chapters.

Two things are added: the binary must differ from 268's, and the records are written with exclusive create.

### 3. Rebind313

**Locally, with CSD's `prepare-assembly19.py` on the copied tree:**
- **Control.** `relocate_map` on the 268 tree reproduces rebind269's map, apart from the overwritten `scope`, with 28 checks.
- **On the reconstructed 313 tree, the worker's whole comparison (`:85–109`) passes.**
  - 09_4 938→942, 09_5 1003→1007 and 09_8 1282→1286, each with the same marker.
  - The other 20 holds and the 5 helpers are unchanged. The helpers are in `draw.rs` (1251, 1305, 1352) and `mod.rs` (31, 58), neither of which 310 touches.

**The preview sites map onto 310 correctly.** The +4 shift is 310's two hunks: `556c556,561` and `814d818`. The E2 site lines in 52 are byte-equal to these lines in 310: 1377→1381, 1397→1401, 1398→1402 and 1399→1403. E1, at 470 and 474, is unchanged.

**The 105 WAVs are unchanged.**
- Every WAV must be byte-equal across rebind313, rebind269 and assembly19 (`:163`).
- Only the three metadata files may differ (`:156`, `:161`), and the sentence rows and outputs must equal rebind269's.
- Coherent `verify` and `narration_delivery` must report 105 cues.
- Rebind269's narration is bound through the controller's `extra`.
- Coherent resolves the map path from `REPO = parents[1]` of the tree's own `tools/` (`coherent_narration.py:23`), so the new map resolves inside the 313 tree.

### 4. The timeline comparator

**It is strict.** It does three things:
- byte equality of all three 09 tables against timeline269 under rebind269 → rebind313;
- byte equality against production timeline21 under assembly19 → rebind313;
- frames equal to timeline269's recorded value and to 16,504. Timeline21's own `09_boundary.timeline.tsv` says `frames 16504`.

**The +4 line shift does not affect it.** The tables carry narration paths, WAV hashes and frame positions. They carry no source line numbers; I read timeline21's three 09 tables.

**It also re-checks the WAVs.** The cuelog carries each 09 WAV's sha256, so equality re-checks all nine 09 WAVs.

**It is applied three times:**
- by the timeline worker;
- by the preview launcher (`:79–86`);
- by the preview worker, which checks byte equality with timeline313 and the mapped equality with timeline269.

### 5. The preview worker and the 09 coverage

**I read all 13 diff hunks against cc3's 274 worker.**
- Render flags, extraction and sheets are unchanged.
- The `TIMED` regex split is the same pattern.
- The other modules' `SOURCE_SHA256` pins still hold, because build313 copies build268's bytes.

**The probe geometry matches review312.** It uses `picture()`'s mapping, ×1.12 about (−3.3, −0.2):
- the upper line is at y = 0.882 at x = 1.15;
- the lower line runs from −2.04 to −2.17 over x = 0.3–0.6.

All 21 pixels and their ±9 px rings lie inside the 960×540 frame.

**The targeted picks**, computed from the worker's own constants (210 frames):
- every required frame: 2087, 2093, 2099, 2326, 4180, 5417, 6240, 6900, 6960 and 7020;
- the 39–133 s sweep: 95 pictures, frames 2340–7980, one each second with no larger gap;
- the 115–117 s anchors, giving frames 6888, 6900, 6912, 6948, 6960, 6972, 7008, 7020, 7032, 7080 and 7140.

**These sit on top of the probe**, which reads every frame 1500–8400 at the 21 points, and on top of cc3's standard picks.

**The probe works on the host.** I ran `lit_counts()` and `zoom_probe()` on CSD on the control, preview274's 09 (`dba964d2…`, present on CSD), with CSD's ffmpeg 8.0.1. The 274 worker's `-vsync 0` extraction already ran on that host.
- It decoded 6,901 frames in 8.5 s.
- The control's lines are present from 1707 to 7988, with no gaps. All 21 points are lit through the base's hole-name fade (2084–2103), and every one of the seven anchors has all 21 lit.
- **Margins.** While the lines are in, each point's contrast is 56–82, against `LIT_MIN` 15. So the lines count as present until their alpha falls to about 0.2–0.27, late in a 2088–2099 fade. Exit window 2086–2100 is not tight.
- After the lines leave, one point (upper x = 2.33) is crossed by later content from about frame 8010, up to contrast 100. That is past the control window, and the worker compares that range with the control rather than requiring absence.

**The probe's decisions on count-space stand-ins** (no movie made):

| Stand-in | Result |
|---|---|
| the control itself | fails (9 checks) |
| lines gone after 2093 | passes (exit 2093) |
| lines gone after 2080 | fails |
| lines gone after 2110 | fails |
| gone, then back at 5001 | fails |
| 3 points relit at 6960 only | fails |

**My own stills.** At full resolution, frames 6900, 6960 and 7020 of the base movie show the lower line through "for every small ϱ" and the upper line through `|g| ≤ M`. That agrees with root's grid finding.

## Findings

**L1. The closure is one level deep.**
- `closure()` and the controller carry the rows of the manifests they name. They do not open a manifest that appears as a row, such as `timeline21-inputs.json` inside build268's.
- Here that loses nothing, because each earlier launcher flattened its own closure; I counted 0 missing rows above.
- A future helper that binds an unflattened manifest would lose rows silently. Keep flattening, or recurse, and keep a "0 missing" count in the handoff.

**L2. Symlinked rows are bound by their target's bytes.**
- 106 rows are links, and the closure and the guard both follow them. This is correct for this chain.
- A later "regular files only" tightening, like the flat-folder check I put on rebind269's narration in 309, must not be applied to these manifests: it would reject all 106.

**L3. Binary 313 is checked for chapter 09 only.** CC0 states this too.
- The other 14 chapters' tables are not compared under binary 313.
- Finals for those chapters should stay on binary 268. A 09 final from binary 313 needs new final-helper versions on both hosts (review312 L1).
- I saw `csd-reviewed-boundary-final317.py` appear in `production-repair17` at 13:53 local. I did not review it; it is outside 314.

**L4. A probe pass is not a picture verdict.**
- The 21 points lie outside the column, so the column itself, where every old crossing was, is covered only by the extracted pictures.
- Root, or a reviewer, still has to look at all 95 sweep pictures, every frame 2080–2106 and every anchor sheet.
- A probe failure fails the job only after the pictures, sheets and records are written, so those pictures can still be viewed.

**N1. Harmless differences.**
- The controller's `success()` does not test `admitted`. A non-admitted receipt has no `exit`, so it still fails.
- `preview-after-timeline313.py` waits up to 12 h for timeline313 even if rebind313 failed. It then stops without launching.

## For root

- **Recipe313.** This file can be recipe313's `helper_review`. Its hash is reported in the pane, and I will not change it; any correction would be a new version.
- **The preview's `--review`.** This review covers the preview worker, so it can also be passed as `--review` to `preview-after-timeline313.py`.

## Not established

- **None of the 313 jobs has run.** That means no build, compile, test, Clippy, rebind, timeline or render. Repair310 is still uncompiled; the rustfmt check is format only.
- **Several checks used copies or stand-ins:**
  - the relocation and the reconstruction ran locally, on a copy of the live 268 workspace;
  - the probe ran on the control movie only, with count-space stand-ins;
  - no repaired movie exists, so the repaired exit, the clear column and the absence at the anchors are predicted, not observed.
- **The CSD checks are a snapshot.** Closure and base were checked from 18:48 to 19:00Z, and the guard re-checks at launch.
- **No acceptance of any kind:**
  - no picture, motion, 4K, fine-sync or AV acceptance;
  - no audio heard or checked.
