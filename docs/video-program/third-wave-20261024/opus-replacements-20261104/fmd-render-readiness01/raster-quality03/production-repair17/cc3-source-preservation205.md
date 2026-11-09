# cc3 source preservation review 205: complex analysis cut

Dispatch-3 (cc3), 2026-11-08 UTC. This is a source and reproducibility review only.

This file is the only file I wrote in the repository. I made no edits to source, shared files, Beads or TODO, did no commit or push, and ran no build, render or synthesis. All frozen render inputs are untouched.

All checks below ran read-only, or on copies in my session scratchpad, outside the repository:
- the engine and kit patch replays;
- one rustfmt run on a copy of `draw.rs`;
- a hash index of the checkout.

I did no listening, and nothing here is picture, sync or audiovisual acceptance.

Paths are relative to this directory (`production-repair17/`, "Q") unless they start with `explainer_corpus/`, `tools/`, `crates/` or `narration_lab/`, which are relative to the repository root. Remote files are named relative to the remote project root (`work/…`, `jobs/…`), as root's own records name them.

## Answer

1. **Is the actual cut preserved by source snapshots plus pinned dependencies? Mostly, and in every place that decides the drawing code. But not completely, and only on this one disk.**
   - The cut uses three binaries:
     - `f4836eee…` for 13 chapters;
     - `5a66979e…` for chapter 08;
     - `352aa5ff…` for chapter 09.
   - Root's `production-source-preserved214/` now holds the complete production19 workspace, all 69 files. I verified this independently: the files on disk, the receipt, the tar archive and `trj-production19-transfer-manifest01.json` agree on every hash.
   - The code deltas for chapters 08 and 09 are preserved locally.
   - Apart from the three files named in item 2, the engine, the kit, `engine-budget.toml` and the film's `Cargo.toml` reproduce byte for byte from tracked patches and pinned revisions. I replayed each one in scratch (details below).
2. **Missing source assets.** These exist only on the remote host:
   - the kit view's root `Cargo.toml`, which the kit crates inherit from;
   - its `Cargo.lock`;
   - one engine test file;
   - the rebind metadata of builds 68 and 164;
   - the three builds' `source-copy.json`/`qualification.json` receipts;
   - an inventory of the build164 workspace.

   The exact list is in the next section.
3. **Single-disk risk.** Almost all the preserved material is untracked or ignored in git, so it survives only in this working tree:
   - snapshot214 is untracked, and `.gitignore:161` (`*.orig`) silently drops its three `.orig` files from any future `git add`;
   - the three repair sources are untracked;
   - the build68/164 and rebind scripts are untracked;
   - the narration WAVs are ignored, by policy.
4. **Smallest plan.** Fetch the missing small files into a new folder, verify them in scratch, and later make one crate-scoped commit plus one manifest, when the owner allows commits. Details are in the final section.

## What reproduces, and how I checked

| Asset | Bound hash | Local evidence | Status |
|---|---|---|---|
| production19 workspace (69 files: crate, `script.tsv`, `docs/storyboard.md`, plans, maps, 4 tools) | receipt214 | `production-source-preserved214/` (untracked). Disk = receipt = tar `453daff4…` = transfer manifest (my check) | Preserved |
| Previously missing: `Cargo.toml`, `draw.rs`, `narration_units.json`, `before-raster-stage09`, production19/stage08 maps | in receipt214 | All 6 now present in snapshot214 | Preserved |
| `src/chapters/draw.rs` | `cd394e5a…` | The tracked `../continuous-quality16/draw.rs` (`80bf6cf2…`), run through rustfmt nightly-2026-08-31 with the repo `rustfmt.toml` (same at 589ab3df and HEAD), gives `cd394e5a…` exactly (scratch) | Reproducible from tracked input |
| Crate `Cargo.toml` | `7644838e…` | The committed `explainer_corpus/complex_analysis_visual/Cargo.toml`, repointed by the tracked `../../../engine-geometry-validation02/repoint_lab.py` rule to the remote `work/tree/repo-view`, gives `7644838e…`. The bound file embeds an absolute remote path; the committed relative one is the right public form | Reproducible |
| Crate `Cargo.lock` | `6a080058…` | In snapshot214. The bound lock lists fmn and its 18 sibling crates as path packages (no franken_manim source line), where HEAD's lock takes the same 19 from git at a399d626. It therefore works only with the engine patch config | Preserved, but differs from canonical in kind |
| Engine: franken_manim a399d626 plus patches | 686 recorded engine files | 676 match a399d626 in the local franken_manim clone. In a scratch export, `engine-plan-limits.patch`, `engine-compiled-route-test.patch`, `engine-tests-repair.patch` and `stroke-index.patch` (all tracked) reproduce 9 of the 10 patched files exactly: `engine.rs`, `plan.rs`, `retained.rs`, `stroke.rs`, `vector_frame.rs`, `stroke_profiles.rs`, `rendering.rs`, `pipeline.rs`, `native_bundle_pipeline.rs` | 685 of 686 reproducible |
| Engine `crates/fmn/tests/native_render.rs` | `d64416e4…` | No patch and no local copy. `../../../engine-secondary-budget01/BUILD.md` records it as root's frame-clock repair: "That repair isn't in a patch file". The plain patch stack gives `ce3ef48e…` | **Missing**. It is an integration test, so it is not compiled into the film binary, but engine test qualification needs it |
| `engine-budget.toml` | `7277a6c0…` | The tracked `engine-budget.config.toml.in`, instantiated with the remote export path, gives `7277a6c0…` | Reproducible |
| Kit view `crates/` | 127 recorded rows | 117 match KITREV 589ab3df (also unchanged at HEAD). The other four files (`cli.rs`, `glyphs.rs`, `cli_pipeline.rs`, `options_hook_routes.rs`) come out of 589ab3df plus the tracked `kit-render-options-hook.patch` and `kit-hook-routes-test.patch`, and pass the tracked `expected-kit.sha256` (scratch replay) | Reproducible |
| Kit view root `Cargo.toml` | `2fa23046…` | Not in any of the 7 Cargo.toml commits in history. HEAD is `269714e2…`. No local copy | **Missing, and material**: `explainer_kit` takes `edition`, `publish`, `fmn`, `fmn-output`, `fmn-codec` and `explainer_score` from `[workspace]` |
| Kit view root `Cargo.lock` | `472710f8…` | Not in any of the 17 commits; no local copy | **Missing**. The film build uses its own lock, so this matters only for kit-level checks |
| Git dependencies (asupersync, franken_markdown, franken_networkx, franken_numpy, frankenpandas, frankenscipy) | revisions in the lock | All six checkouts are cached at their locked revisions under the user's cargo git cache | Available here; outside the repo |
| crates.io dependencies | lock checksums | Pinned by `Cargo.lock` | Pinned |
| Toolchain | nightly-2026-08-31; remote cargo/rustc sha256 pinned in `build19.py` | The toolchain is installed locally, but on a different host type | Pinned; byte-identical binaries need the same host and paths |
| What the binary embeds at compile time | | Its Rust sources, `script.tsv` and `docs/storyboard.md` (`include_str!`). Narration plans and maps are read by tools, not compiled in | Preserved |
| Retained narration: 103 decoded Frederick takes plus 103 original downloads | decode receipt | All present here with matching hashes (`narration_lab/…/production-audio-stage07-01/decoded01/`; the receipt is untracked) | Preserved; never committed, by policy |
| 08_5/09_1 Frederick takes | `audio17/mapped-receipt.json` | `audio17/08_5.wav` and `audio17/09_1.wav` match (ignored). The raw takes and receipts are in `raw08_5-ui01/` and `raw09_1-ui01/` | Preserved |

## The two repair overlays

Both repaired builds are production19 plus two source files plus rebind metadata (`two-label-build68.py` and `two-label-build164.py`, both untracked):
- **build68** (chapter 09 binary) adds `boundary-label-repair52.rs` (`dc80c541…`) and `loop-sums-label-repair68.rs` (`a13f176c…`).
- **build164** (chapter 08 binary) adds `boundary-label-repair52.rs` and `loop-sums-label-repair164.rs` (`1cf25968…`).

The code changes against snapshot214 are small and stay inside their chapters:
- `boundary.rs` changes two constants, `RHO_AT` and `POLYGON_NAMES_AT`. They are referenced only at `boundary.rs:582` and `:680`.
- `loop_sums.rs` changes the one literal at line 502, the `c` label offset.

So a single source state, build164, carries the exact drawing code of all 15 chapters. That is a source-level statement. Nobody has re-rendered chapters 01–07, 09 or 10–15 from build164, so equal pictures are not established.

What is missing locally for the overlays:

| Overlay | Missing file (remote) | Known hash | Note |
|---|---|---|---|
| build68 | `work/two-label68/workspace/explainer_corpus/complex_analysis_visual/narration_units.json` | `662eafca…` | Listed in `repair-runtime-inventory156.json` (untracked) |
| build68 | `…/docs/narration-direction-map-frederick-label69.json` | `846ee418…` | Same inventory |
| build68 | `…/narration_units.before-label69.json` | `995e25c7…` | Already equal to snapshot214's `narration_units.json` |
| build164 | `work/two-label164/workspace/…/narration_units.json`, `narration_units.before-label165.json`, `docs/narration-direction-map-frederick-label165.json` | unknown here | `two-label-rebind165.py` equals `two-label-rebind69.py` apart from job numbers. No local inventory of the two-label164 workspace exists |
| all three builds | `jobs/csd-production-repair-build19/{source-copy.json, qualification.json}` (`b6799c42…`, `2731a529…`), and the same two files for `csd-two-label-build68` and `csd-two-label-build164` | 19's known | These bind source to binary. The transfer manifest lists build19's pair by hash, but no local copy exists |
| engine/kit tree | a complete file list with hashes of `work/tree/{engine,repo-view,crate}` | not recorded | `unchanged-dependency-intersection01.json` is, in its own words, an intersection; "missing closure remains separately audited" |

The rebind metadata does not enter any binary. It bound the 08/09 rebind and timeline steps, whose tables root already compared (`timeline165-terminal176/comparison.json`, `two-label-terminal156/comparison.json`).

## Smallest source-only plan, after rendering

No step below touches a frozen input or an existing path, and none renders or synthesizes. Root runs every step.

**A. Preserve (no commit; do it now or after the master).**
- Copy the missing files above into a new folder, for example `production-overlays-preserved<N>/`. Use receipt214's method: hash before, copy, hash after, hash locally.
- All of them are small text files:
  - kit view `Cargo.toml` and `Cargo.lock`;
  - engine `native_render.rs`;
  - build68 and build164 rebind outputs;
  - three `source-copy.json` and `qualification.json` pairs;
  - one hash list of the full `work/tree`.
- Also record the full build164 workspace file list with hashes (71 files, if it matches build68's inventory), so the chapter 08 source state is inventoried as production19's is.
- Because commits are prohibited, the only off-disk preservation is an archive kept outside git: a tarball plus `SHA256SUMS`, stored with the release assets. That is root's or the owner's call.

**B. Verify in scratch (no render).**
- Rebuild three trees from tracked or preserved inputs only, and compare every hash with the records:
  - the engine (a399d626 plus 4 patches plus `native_render.rs`);
  - the kit view (589ab3df plus 2 patches plus root `Cargo.toml`/`Cargo.lock`);
  - the build164 workspace (snapshot214 plus 2 repair files plus rebind165 outputs).
- A guarded rebuild on the same host to compare binary hashes is optional. It is a build, not a render. It needs new paths, and root decides.

**C. Integrate (only when the owner allows a commit).** One commit touching only `explainer_corpus/complex_analysis_visual/` plus one manifest file.

Crate content, the build164 state:
1. The 22 crate files that differ from HEAD, taken from snapshot214:
   - `Cargo.lock`, `docs/storyboard.md`, `narration_units.json`, `oracle.py`, `script.tsv`, `src/main.rs`, `src/model.rs`;
   - 15 chapter files.

   Then make two replacements:
   - `boundary.rs` comes from `boundary-label-repair52.rs`;
   - `loop_sums.rs` comes from `loop-sums-label-repair164.rs`.

   `narration_units.json` and the label165 map come from rebind165.
2. `src/plate_layout.rs`, which is new and is a compiled module.
3. The bound direction map, which is a decision for root:
   - production19's `narration-direction-map-frederick-production19.json` binds the unrepaired chapters;
   - `…-label165.json` binds the build164 rebind.
4. **Keep HEAD's relative `Cargo.toml`.** It reproduces the bound one through the repoint rule. Do not commit the repointed file, which carries an absolute remote path.
5. **Leave out** the `.orig` files, the `before-*` plans and the historical maps (stage06–09, oliver12, quality16). The manifest records their hashes instead.

The manifest names, by hash and without absolute paths or host names:
- receipt214 and the overlay receipts;
- a399d626 and the four tracked engine patches, plus the preserved `native_render.rs`;
- 589ab3df and the two tracked kit patches, plus the view root `Cargo.toml`/`Cargo.lock`;
- the `engine-budget` template;
- the toolchain and its cargo/rustc hashes;
- the build flags (`--release --locked --offline --config <engine-budget> … -C metadata=jmev_production_repair18` for build19; build68 and build164 take theirs from their scripts);
- the three binary hashes, with the chapters each one rendered;
- the narration receipts and WAV hashes;
- the 15 timeline hashes.

Decisions for root:
- **Engine form.** The cut's engine is a399d626 plus four repo-tracked patches applied in an export, not an upstream revision. AGENTS.md says franken_manim is consumed pinned by revision and never changed from here, and that a render from a dirty engine tree is never a final. The shared engine tree was not patched, but the tagged commit must carry the patches, the `engine-budget` template and the export recipe (`../../../engine-geometry-validation02/BUILD.md`), or it does not describe the cut.
- **Lock form.** The bound `Cargo.lock` resolves fmn as path packages, so a canonical build with it needs the engine patch config. Committing it keeps exact reproduction; committing a git-sourced lock would describe a different build.
- **Shared files** (orchestrator-owned) are left unchanged by this plan:
  - `tools/coherent_narration.py`. The production19 adapter (`a7fc6705…`) is already tracked in Q as `coherent_narration.py`. It is needed only to re-run assembly, not to rebuild or re-render from the preserved narration.
  - `crates/explainer_kit` and franken_manim. AGENTS.md says never to change franken_manim from here, and the kit hook would affect every film. Both stay as recorded recipes.

## Gaps that remain open

- **No listening, by anyone, of any audio.** That includes the 09_1 "of a" that the recognizer dropped (review199).
- **No independent final audiovisual review.** The final master is not built, so there is no master timing recheck and no picture or sync acceptance.
- Byte-identical re-renders and binary rebuilds are not established. Embedded absolute paths and the host toolchain make a binary rebuilt elsewhere differ in bytes. Even on the same host, identity needs the guarded rebuild in step B.
- The thumbnail's render route is cc0's review (`cc0-thumbnail-route205.md`) and is not covered here.
- Narration remains ElevenLabs Frederick Surrey, as every request and receipt records. That is a recorded voice setting, not a judgment from listening.
