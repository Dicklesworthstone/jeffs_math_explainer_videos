# Secondary engine validation: runner

**Status: prepared, not run.** Nothing here has been compiled, queued on the lane or rendered. `checks/rehearse.py` exercised every part that needs no Cargo on a scratch stage: the copy, both patches, the guard's own input check, the format steps and the guard's caps. RubyRaven wrote it, so it isn't an independent review. Root owns the run.

## Run

```sh
V=<repo>/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01
python3 -I -B "$V/run_validation.py" --self-test
python3 -I -B "$V/run_validation.py" stage --source <root's geometry stage> --stage <new folder outside git>
python3 -I -B "$V/run_validation.py" plan --stage <that folder>
python3 -I -B "$V/run_validation.py" run --source <root's geometry stage> --stage <that folder>
```

`--source` is root's original geometry stage, the one holding `engine`, `repo-view` and `crate`. `stage` refuses any byte that differs from `inputs.tsv`. The merged complex film (combined-repair-stage03) is not an input here. It needs its own bind and its own job.

`run` puts each job on root's exclusive lane, one guard receipt per job (`engine-secondary-<job>01`). It stops at the first failure or resource stop and never retries. After a stop, `--from <job> --attempt 02` is root's call. The base jobs refuse a stage that is already patched.

## Jobs, in order

| Job | What | Target growth bound, GiB |
|---|---|---|
| `deps` | Metadata of the three workspaces, both lock checks. Compiles nothing | 0 |
| `base-build` | Root's unpatched tree: the pixel probe and `native_camera_pipeline` | 1.68, all of `geometry-native01` today |
| `base-run` | Default frames on five routes, before the patches | 0 |
| (between jobs) | Both patches applied, 8 + 8 sealed hashes checked | |
| `engine-lint` | rustfmt on 13 files, Clippy `-D warnings` on `fmn-render` and `fmn` | 0.35, `native-check03` |
| `engine-build` | Test binaries | 1.68 |
| `engine-test` | All `fmn-render` tests; `native_render`, `native_bundle_pipeline`, `native_camera_pipeline`; the probe; default frames compared byte for byte | 0 |
| `engine-ignored` | The two controls over the real defaults, under `/usr/bin/time -l`, one hour each | 0 |
| `engine-fronts` | `fmn-cli --all-targets`, `fmn-conformance --lib` | 0.5, a guess |
| `kit-build` | rustfmt, Clippy and the `explainer_kit` test binaries, against the stage engine | 2.22, all of `geometry-kit01` today |
| `kit-test` | At least 111 tests, `the_hook_governs_every_render_route` by name | 0 |
| `film-check` | Clippy on `complex_analysis_visual` and `plate_lab` against the stage. Links nothing, so root's film binaries stay | 1.0, `lab-build01` |

The bounds total 7.43 GiB, and no job's bound is above 2.22. At the rehearsal, target growth stood at 9.98 of 20 GiB and 58.6 GiB was free. The runner refuses to start under 16 GiB free (the 8 GiB stop plus the bounds).

## What a pass shows

- Both patches compile and pass Clippy. All 20 tests they add pass by name, the 2 ignored ones included. A test counts only when it prints an `... ok` line.
- Lowered budgets refuse by name, and raised budgets give identical frames, on every native route. Those are the serial facade, the compiled affine and camera workers, the camera rig, and the retained renderer's caller and worker routes.
- **Default pixels don't move.** `secondary_default_pixels.rs` names no new field, so it builds on both trees. It writes every frame of one small scene on five routes: affine serial, affine worker, camera serial, camera worker and camera rig. It runs once before the patches and once after, and the runner compares the frames byte for byte.
- The kit's options hook still governs every render route on the patched engine, and the film still compiles against it.

## Guard

`guard_owner56g_target20g.py` loads root's guard from its own file, pinned by sha256 `819746fb…`. It changes two caps in memory: work from 48 to 56 GiB, and target growth from 14 to 20 GiB. It sets native admission to 10 GiB. These stay as they were:

- the same lane lock, because `ROOT` is root's;
- the 8 GiB stop and the 4 GiB growth per job;
- the input manifest, checked before and after each job;
- the original baseline of 2026-10-27.

The self-test drives these edges with fake samples in a temporary lane, and no real job runs.

## Frozen inputs

`inputs.tsv` has 324 rows covering 154,577 files and 8,992 MiB:

- root's three trees and two pinned locks, byte-identical to the `film-cells4-routes01` manifest;
- the 296 registry crates and 9 git checkouts the five locks name;
- the nightly toolchain's `bin` and `lib`, and ffmpeg;
- root's guard, the wrapper and the probe;
- both patches with their hashes, and validation02's four tools.

`run` rechecks all of these but root's tree before and after, about 45 seconds each time. Not frozen: ffmpeg's libraries, the system, `sbh`.

## Not measured

- Every bound is an estimate from root's receipts, and `fmn-conformance` has never been built on this stack.
- Memory is recorded only for the two ignored controls. Max resident is the largest process Cargo waited for. Peak footprint is Cargo's own. The guard samples disk, not memory.
- There's no dense camera scene: `plate_lab` is affine. Default pixels cover one small scene per route, not film chapters.

## Departures from the sealed build files

- `fmn-conformance` is checked with `--lib`, not `--all-targets`. The only literal of a changed struct outside the patches is `perf_pg6_peak.rs:719`, in the library. `--all-targets` would first build pyo3, studio and wasm dev-dependencies.
- rustfmt also covers the patched `src` files.
- The before-and-after pixel comparison is new.

**Errata for the sealed camera `BUILD.md`.** `both_routes…` runs 130 small renders, not about 110. In section 5, the column "Fixed route" is the caller route.

## Stops

| Result | Meaning |
|---|---|
| `deps` fails | A lock moved or a source is missing offline. Nothing was compiled |
| `base-build` or `base-run` fails | A fault on root's tree, before any patch |
| An engine job fails | Send back the output. Don't relax a lint or a test |
| `default pixels:` lists files | A patch changed default frames. Stop |
| Exit 75 | Resource stop. The receipt has the samples |

## Files

| File | Purpose |
|---|---|
| `run_validation.py` | Stage, bind, plan, run, and the job body the guard launches |
| `guard_owner56g_target20g.py` | Root's guard with the owner's two raised caps |
| `secondary_default_pixels.rs` | Default-frame probe, copied into the stage's `fmn/tests` |
| `inputs.tsv` | Frozen inputs |
| `checks/rehearse.py` and its `.out` files | The rehearsal described above |
| `bundle.sha256` | Every file here |
