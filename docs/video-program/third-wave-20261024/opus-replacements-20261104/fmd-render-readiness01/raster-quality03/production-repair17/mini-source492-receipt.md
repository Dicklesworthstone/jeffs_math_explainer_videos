# 492: mini staging source tree (CC0)

cc0, 2026-11-09, 05:42 UTC. Q is this folder.

**`Q/mini-source492` is ready to transfer.** It is to be placed so that its contents become `/Volumes/EERCHExport/jmev-complex-native-20261109/source` on the mini (TARGET).

**Manifest.** The manifest is `mini-source492-manifest.json` (`e5a6d12f4f505362be12ca4309b03e22199ecbfc37a7ecf8d6998b31fc954a0f`). Its `files` rows give the path, sha256 and size of every staged file. The listing sha256 is `32f99c8309f3e93c9b3ff55cb926ad2f892d62dbaa5321351d00673d852c8418`, computed over the sorted `"<sha256>  <path>\n"` lines.

## Contents

The tree holds 4,197 files, 49,004,872 bytes:

| Folder | Files | Source |
|---|---|---|
| `engine/` | 4,057 | the owner Mac's original engine |
| `repo-view/` | 68 | kit 39, score 23 and the shared workspace files |
| `workspace/` | 70 | build 268's scene source, from the 489 map: 62 from `production-source-preserved214/workspace`, 8 from the recipe 268 candidates. The three `.orig` files are part of 268's own source. |
| `engine-budget.toml` | 1 | path-mapped, see below |
| `.cargo/config.toml` | 1 | placed, see below |

## The only changes

1. **The engine patch.** `stroke-index.patch` (`da6b42b6`) was applied to the **copied** engine only, with GNU patch 2.8 after a dry run. It changed two files:
   - `fmn-render/src/engine.rs`: `85bdc6b7` → `de6551fc`;
   - `fmn-render/src/stroke.rs`: `e0feeab4` → `668867df`.
2. **`engine-budget.toml`.** The local root was replaced by TARGET in its 19 `[patch]` path lines. The staged file is `088dd6b6`; with TARGET mapped back to the CSD path it is `7277a6c0`.
3. **The crate's `Cargo.toml`.** Only its two kit and score `path` values changed, from the CSD paths to `TARGET/repo-view/crates/…`. The staged file is `356eff0f`; mapped back it is `7644838e`. `Cargo.lock` is unchanged and has no absolute paths.
4. **Placement.** `.cargo/config.toml` holds the bytes of CSD's `work/.cargo/config.toml` (`79fb8766`). Cargo reads it from any ancestor of the build directory, so run cargo from `workspace/explainer_corpus/complex_analysis_visual`, as 268 did.

No staged Cargo file has any absolute path left other than TARGET. Every other file is byte for byte.

## Binding

**Against 268's actual inputs.** Every staged file equals its row in root's retrieved `root-csd-build268-inputs496.json` (`53f1360e`, the 268 receipt's inputs sha):
- engine 4,057 and repo-view 68, at `work/tree`;
- `engine-budget.toml` and the crate `Cargo.toml`, after mapping TARGET back to the CSD path;
- `.cargo/config.toml`, at `work/.cargo`;
- 62 scene files at `production-repair19/workspace`, and 8 at their candidate rows.

**Originals.** After staging I re-hashed the originals: the local engine, repo-view and budget file, and the Q scene sources. They are unchanged.

## For the transfer

- **Keep file modes.** 14 files are executable; the manifest's `executable` list names them.
- **Never commit this tree.** Q is not git-ignored, and the tree holds exact third-party engine bytes and machine paths.
- **Not in the tree:** the crates.io and git caches for the locked dependencies, and a Mac toolchain `nightly-2026-08-31`.
- **The binary will differ.** A Mac build will not reproduce 268's Linux binary `983954c5`.

I made no SSH, build, test, transfer, launch or Beads change. No audio heard or checked.
