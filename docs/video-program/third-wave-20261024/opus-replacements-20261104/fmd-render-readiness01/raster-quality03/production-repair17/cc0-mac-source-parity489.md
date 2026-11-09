# 489: owner Mac original source against CSD build 268's inputs (CC0)

cc0, 2026-11-09, 05:37 UTC. Q is this folder. Full record: `cc0-mac-source-parity489.json`.

L is the owner Mac's original copy `jmev-geometry-native-20261104-01`, which is outside the repo. W is `/data/projects/jmev-stroke-probe-20261106/work/tree` on CSD.

This is content comparison only, and read-only. Nothing was built, tested, staged or launched, and no tracker was edited.

## Result

**The source is exact after one known patch and one path mapping.**

| Input | Result |
|---|---|
| `L/repo-view` against `W/repo-view` | **68 of 68 equal**: kit 39, score 23, plus `Cargo.toml` `2fa23046`, `Cargo.lock` `472710f8`, `rustfmt.toml` and `.cargo/config.toml` |
| `L/engine` against `W/engine` | **4,055 of 4,057 equal**, with no extra or missing files |
| the 2 engine files that differ | `crates/fmn-render/src/engine.rs` and `stroke.rs`. The local files are the pinned `a399d626` bytes; CSD's are patched |
| `L/engine-budget.toml` against `W/engine-budget.toml` | equal (`7277a6c0`) after replacing the local root with W in its 19 `[patch]` path lines |
| CSD `work/.cargo/config.toml` (`79fb8766`) | byte-identical to `L/repo-view/.cargo/config.toml` |
| 268's 70 scene source files (`review-build268-terminal272/source-copy.json`) | all 70 are in Q byte for byte: 62 in `production-source-preserved214/workspace` and 8 as the recipe 268 candidates |

**The engine patch.** `../../../complex-stroke-index-candidate01/stroke-index.patch` (`da6b42b6`) explains the two engine differences. I applied it with `git apply` to copies of the two local files, outside the repo, and both came out at the exact CSD hashes:
- `engine.rs`: `85bdc6b7` → `de6551fc`;
- `stroke.rs`: `e0feeab4` → `668867df`.

Its `HANDOFF.md` lists the same frozen and candidate hashes. L itself is unchanged.

## How the CSD rows are bound to 268

**The hashes I compared against.** The CSD hashes come from build 19's guard manifest, preserved locally as `production-overlays-preserved220/qualification01/production-repair17/build19-inputs.json` (`290dd1e4`, 11,770 rows).

**Why those rows are 268's.** That rests on two CC1 records:
- CC1 282 found that the 238 and 268 input manifests have identical rows for the engine, repo-view, budget and Cargo config.
- CC1 273 found that 268 compiled the same engine, kit and score content as build 19.

**Not read.** I could not read CSD's own `review-repair-build268-inputs.json`, which 268's receipt records as `53f1360e`. Two bounded read-only ssh attempts, ending 05:32:43Z and 05:35:29Z, got "server not responding".

## For mini staging

1. **Engine:** use `L/engine` with `stroke-index.patch` applied to the two files above. Unpatched `L/engine` is not what 268 compiled.
2. **Kit and score:** use `L/repo-view`, **not this repo checkout's `crates/explainer_kit`**. The checkout's kit:
   - differs in `src/cli.rs`, `src/glyphs.rs` and `tests/cli_pipeline.rs`;
   - lacks `tests/options_hook_routes.rs`;
   - holds a stray `src/.ee/hook-state` file.

   The checkout's `explainer_score` matches.
3. **Scene source:** use the 70 files above. Do not use `L/crate`: against `W/tree/crate` it has 21 files that differ, 7 missing and 2 extra, and 268 did not use `W/tree/crate`.
4. **Path edits.** The crate's `Cargo.toml` (`7644838e`) has exactly two absolute CSD lines, the kit and score `path` values. They must point at the copied `repo-view`. That edit changes the file's hash, so record the bytes before and after. `Cargo.lock` (`6a080058`) has no absolute paths.
5. **Cargo config placement.** The `.cargo/config.toml` (`-Z threads=4`, checksum freshness) applies on CSD because cargo finds it in an ancestor directory of the crate. On the mini it must also sit in an ancestor of the crate directory.

## Outside content equality

These were not compared:
- the crates.io registry and the six locked git sources;
- the CSD user's own cargo config;
- the Linux toolchain binaries, which a Mac cannot match byte for byte;
- 268's flags and environment, which are in `review-repair-build268.py`.

A Mac build is a new binary, not 268's `983954c5`. Its platform-independent comparison points are facts sha `7d528509` and the 15-chapter list.

No publication, picture or AV claim. No audio heard or checked.
