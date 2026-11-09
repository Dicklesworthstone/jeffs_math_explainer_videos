# Retention dependencies 273 (cc1): closing E2 of impact261 for chapters 02, 03, 13 and 15

## Summary

This file closes E2 (the dependency closure) from `cc1-combined-impact261.md`. The retention candidates are 02 turning_numbers, 03 two_planes, 13 real_integral and 15 closing.

**E2 is closed at the level of inputs, not of compiled artifacts.**
- Build268 produced binary `983954c5…`. Build19 produced binary `f4836eee…`, which rendered all four existing finals.
- Outside the eight recipe files, everything build268 compiled has the same content as what build19 compiled:
  - the workspace, `Cargo.toml` and `Cargo.lock`;
  - the engine actually compiled;
  - the kit and the score crate;
  - both cargo configurations, the toolchain binaries and the cargo invocation;
  - all six git dependencies, at their locked revisions.

**One uncertainty matters for retention (U1).** 113 of the 216 dependency rlibs in the shared target directory were recompiled in place after build19, overwriting what build19 linked. They include 13 engine crates, `explainer_kit` and `asupersync`. So the engine machine code in `f4836eee` can no longer be compared directly. One small build would settle it (U1 below).

**How the evidence was gathered.**
- Locally: records in this folder.
- On csd, read only, over SSH: Python reading files, `sha256sum`, `cat`, and `git --no-optional-locks rev-parse` / `status`.
- Nothing was written on csd, and nothing was built, rendered, run or synthesized. No Beads or TODO edit.
- Timeline269 was running and was not touched or duplicated. This file is the only file I wrote.
- No pixel or audio equivalence is claimed.

## Bound evidence

| Item | Value |
|---|---|
| Build268 binary (`review-build268-terminal272/qualification.json`, `2f3430a8…`) | `983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7` |
| Build268 `source-copy.json` (local copy) | `81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c` |
| Build268 `resource-receipt.json` | `3504e1ab…`; exit 0; `inputs_sha256 == final_inputs_sha256` (`53f1360e…`) |
| Build268 input manifest (csd `review-repair-build268-inputs.json`) | `53f1360e1102252a59a82e088f51152601d356562d152270f989bf1e86251663`, 12,319 files |
| Build19 binary (`jobs/csd-production-repair-build19`) | `f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22`; exit 0; inputs == final inputs |
| Build19 input manifest (csd `build19-inputs.json`) | `290dd1e4b0ae129e22c6157953a5583ee213793023aae8c79863fcbcaed4aec6`, 11,770 files |
| Timeline21 input manifest (csd, folded into the 268 manifest by its launcher) | `6ca99e8ef9946f20b0e9827da5af13b4b534b22b65caf06871788df67ce35968`, 12,304 files |
| Recipe268 | `edadbffc1a0740594fee11ec458bc67131016765dc5721f0157675a421b2a08b`; draw is `cc2-plate-fade-candidate264-draw.rs` `9bb07b3f…` |

**Draw 264.** Its code outside `#[cfg(test)]` is byte-identical to 246 and 260, so the runtime analysed in 261 is unchanged.

**The four finals were rendered by `f4836eee`:**
- final02: TRJ job `trj-production19-final-02-turning-numbers32`;
- final03: TRJ job `…-final-03-two-planes28`;
- final13: CSS job `css-production19-final-13-real-integral96`;
- final15: HZ2 job `hz2-production19-final-15-closing53`.

**The CSS and HZ2 copies are the same binary.** `css-runtime-manifest75.json` and `hz2-runtime-manifest47.json` list `work/bin/production19` at `f4836eee…`.

## What both builds compiled

### 1. Workspace

**Build268 `source-copy.json`:**
- It records 70 files, 8 of them changed.
- The 62 unchanged files all equal the production19 receipt (`production-source-preserved214/receipt.json`). Among them are `turning.rs`, `two_planes.rs`, `real_integral.rs`, `closing.rs`, `the_bump.rs`, `model.rs`, `main.rs`, `score.rs`, `facts.rs`, `plate_layout.rs` and `thumbnail.rs`.
- The 8 changed targets equal their recipe268 hashes after `cargo fmt`. This independently confirms root's check.

**Bin crate dep-info.** The dep-info of the build268 bin unit lists 24 workspace sources, all inside the copied workspace.

### 2. Lock and manifest

**Same files in both builds.** `Cargo.lock` (`6a080058…`) and `Cargo.toml` (`7644838e…`) are the same, and both builds ran `--locked --offline`.

**The cargo invocation is identical** in `build19.py`, `two-label-build68.py`, `thumbnail24.py` and `review-repair-build268.py`:
- the same asserted cargo binary (`b854c942…`) and rustc binary (`49f9fa06…`), with the `RUSTFLAGS` and `RUSTC` wrapper variables asserted absent;
- the same environment (`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`, `CARGO_TARGET_DIR=work/target`);
- the same flags: `--release --locked --offline --config work/tree/engine-budget.toml --bin complex_analysis_visual`.

**The only invocation difference is the final rustc metadata string**, `jmev_production_repair18` for build19 against `jmev_review_repair268`. It affects only the bin crate.

### 3. Engine: the pin is nominal; the compiled engine is a patched export

**What `Cargo.toml` names against what is compiled.**
- `Cargo.toml` names `fmn` from git at rev `a399d626…`.
- But `work/tree/engine-budget.toml` (`7277a6c0…`, mtime 2026-11-06 02:02) patches every `franken_manim` package to `work/tree/engine/crates/*`. The file describes these as "path copies of one patched export".
- Both builds pass the same file.

**The export.**
- `work/tree/engine` has no `.git`, so its relation to `a399d626` cannot be read from the tree itself.
- What can be shown is that the export is unchanged: all 4,057 files on disk are in the build19 manifest, and every one re-hashes today to build19's recorded value. Re-hashing build19's engine, kit, score and config rows gives 0 mismatches.

**The engine files 261 relied on are byte-identical** between `a399d626` (local clone, `git show`) and the compiled export:

| File | sha256 |
|---|---|
| `fmn-mobject/src/stage.rs` | `390b41e0…` |
| `fmn-anim/src/fading.rs` | `db40b8c3…` |
| `fmn-anim/src/animation.rs` | `0724de75…` |
| `fmn/src/lib.rs` | `29a71589…` |

So 261's account of `family()`, `get()`, `image_resource()`, `fade_in`/`fade_out` and `prepare` applies to the compiled engine.

**Two dep-info paths look unlisted but are not.** The dep-info names two engine files through un-normalized paths: `fmn-tex/src/../../../SUITE.lock` and `fmn/src/../../../README.md`. Normalized, they are in the build19 manifest at equal hashes (`e4758123…`, `ca3c932b…`).

### 4. Kit and score: the actual paths

**The path dependencies** are `work/tree/repo-view/crates/explainer_kit` and `…/explainer_score`.
- All 39 kit files and all 23 score files on disk are in both manifests at equal hashes and are unchanged today.
- The whole `repo-view` directory has 68 files, all in the manifest.

**Correction to 261.** 261 cited my local `crates/explainer_kit` copy, and two of its files are older than the compiled ones:
- `glyphs.rs`: local `8652dd29…`, compiled `225504a6…`;
- `cli.rs`: local `a63970e0…`, compiled `3c5221fe…`.

The compiled files add only the `run_with_contract_and_options` hook (the `adjust` argument), which production19's `main.rs` calls. Re-checked on the compiled bytes, both 261 claims hold:
- `glyph_text` is still read only in `glyphs.rs::probe` (line 112);
- `score_video` still renders the whole video's music from every chapter's timeline (`score::render(&identity, &sections, …)`) and slices it with `chapter_music(k)`.

**Files that match exactly:** kit `kit.rs` (`ef035faf…`) and `lib.rs` (`6fd69ffe…`), and score `score.rs` (`8ba3f501…`), match between local and compiled. So E4's statement that the score is one whole-video render stands on the compiled `score.rs`.

### 5. Configuration not in any manifest

**Unhashed configuration:**
- `work/.cargo/config.toml` (`79fb8766…`, mtime 2026-11-06 02:02) sets `rustflags = ["-Z", "threads=4"]` and turns on content-checksum freshness. Every recorded unit fingerprint shows those rustflags and one config hash (`8362119626465528914`).
- `/home/ubuntu/.cargo/config.toml` (`14ae6b73…`) was last modified 2026-10-18. It sets only dev/test debug levels and a `target-dir` that `CARGO_TARGET_DIR` overrides.
- In the toolchain `nightly-2026-08-31` sysroot, the newest of 4,130 library files is from 2026-11-07 02:59, before build19.

**None of these changed between the two builds.** That conclusion rests on mtimes, not hashes.

### 6. Git and registry dependencies

**The six git sources in the lock are clean checkouts at their locked revisions.** `git status` shows only cargo's own `.cargo-ok`:

| Package | Locked revision |
|---|---|
| asupersync | `c48399f20a97…` |
| franken_markdown (`fmd-font`, `fmd-math`) | `126a5304e8d9…` |
| franken_networkx | `9d710b1c33e9…` |
| franken_numpy | `a15d5c32e933…` |
| frankenpandas | `da85cf65df48…` |
| frankenscipy | `85edd7a99f2a…` |

- Only `fmn-platform` depends on asupersync. Its 849 sources are the only compiled sources outside every manifest. Since the checkout is clean, its content is that of the locked revision.
- The 3,119 extracted registry source directories are not hashed by any manifest (U2).

### 7. Dependency artifacts actually linked

**Neither build compiled a dependency.** In `tests.stderr`, `clippy.stderr` and `build.stderr` of both builds, the only crate compiled or checked is `complex_analysis_visual`. Both linked artifacts that were already in the shared `work/target`.

**But the shared target changed between them:**
- `csd-production-thumbnail24` (started 2026-11-07 17:22) recompiled 93 dependencies in place.
- `csd-two-label-build68` (2026-11-08 00:50:04 to 00:59:07) recompiled 113 in place. Its first rebuilt rlib was `libc`, its last `asupersync`.
- Build268 (05:35) reused build68's artifacts.
- Both jobs used the invocation in section 2. Their logs do not show what made the fingerprints stale.

**What the target directory holds now (216 rlibs):**
- 103 rlibs (101 packages) predate build19 and were never rebuilt, so build19 and build268 linked the very same files. They include `fmn-codec`, `fmn-config`, `fmn-core`, `fmn-dmath`, `fmn-frame`, `fmn-hash`, `fmd-font`, `fmd-math` and `explainer_score`.
- 113 rlibs (106 packages) were rebuilt after build19. They include `fmn`, `fmn-anim`, `fmn-cache`, `fmn-geom`, `fmn-library`, `fmn-mobject`, `fmn-output`, `fmn-platform`, `fmn-render`, `fmn-runtime`, `fmn-scene`, `fmn-tex`, `fmn-text`, `explainer_kit` and `asupersync`.

**Why the rebuilt rlibs have the right inputs.**
- Their fingerprints use `CheckDepInfo` with content checksums. Build268 found them fresh, so for path dependencies they were compiled from sources whose checksums match today's bytes, which are build19's.
- Build19 likewise found its own older artifacts fresh under an identical request.
- So both builds linked artifacts compiled from equal inputs. **The bytes build19 linked for those 113 rlibs no longer exist.**

## Remaining uncertainty that matters for retention

### U1. Whether the rebuilt engine crates compile to the same code

**The doubt.** Equal sources, toolchain, flags and configuration are expected to give equal machine code. But this build runs rustc's parallel front end (`-Z threads=4`, nightly), and that is not guaranteed bit-reproducible. The artifacts build19 linked for `fmn-render`, `fmn-scene`, `fmn-mobject`, `fmn-anim`, `fmn-runtime`, `fmn-output`, `fmn-text`, `fmn-tex`, `explainer_kit` and the rest are gone.

**What this means for 02/03/13/15.** Nothing so far shows that the engine code build268 links equals the code that rendered the existing finals. 261 argued that the only runtime difference for these chapters is the `image_records` read in the two fade helpers. That argument holds for source, and only by expectation for linked code.

**A decisive check, for root to decide.** It is a small build, so I did not run it. Rebuild the unchanged production19 workspace and compare the result with `f4836eee`:
- **Where.** Rebuild in place at `work/production-repair19/workspace`: the same path build19 used, so no path differences enter.
- **Command.** Use build19's exact `cargo rustc … --bin complex_analysis_visual -- -C metadata=jmev_production_repair18`, with the same environment, against the current `work/target`. Skip `cargo fmt`, since the workspace is already formatted.
- **Cost.** Only the bin crate should compile.
- **Side effects.** It overwrites `work/target/release/complex_analysis_visual`, which is already copied to `work/bin/review-repair268`, and the shared bin unit. Timeline269 runs `work/bin/review-repair268`, not the target binary.

**Reading the result:**
- **Byte-equal:** today's dependency artifacts, toolchain and flags reproduce production19 exactly, and build268 differs from `f4836eee` only in the workspace crate (the eight recipe files and the metadata string). That closes E2 at the artifact level.
- **Not equal:** this does not show a difference in behaviour, but artifact equality stays unproven. Then only a frame comparison of a re-render with binary 268 (E5) can stand in for it.

### U2. Registry sources (minor)

- The 3,119 extracted crates under `/home/ubuntu/.cargo/registry/src` are not hashed by any manifest.
- They come from archives that the lock pins by checksum, but cargo does not re-verify them after extraction.
- Only an edit to an extracted registry source after build19 would matter. No evidence of one was found, and none was sought file by file.

### What E2 does not change

- E3 (timeline269 against timeline21, 45 tables), E4 (reusing the score21 stems and identical narration across all fifteen chapters) and E5 (frames) stay as stated in 261.
- On the encoders the existing finals already used: all four are x264 core 165, CRF 16, with identical option strings except final15 (HZ2), which has `threads=18 lookahead_threads=6` against `24`/`8`. This is a fact about the existing files. It is not a dependency of retention.

## Limits

- Read-only, as described in the summary.
- Unit times come from file mtimes. Freshness was inferred from cargo's logs and fingerprint files, not by re-running cargo.
- The engine export's relation to `a399d626` was checked only for the four files 261 used, not for the whole export.
- Registry sources were not verified.
- No compile, test, render, extraction, synthesis or listening.
- No acceptance of pixels, audio, motion, fine sync or final AV.
- Narration stays ElevenLabs Frederick Surrey.
