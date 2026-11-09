# cc3 source preservation notes 220

Dispatch-3 (cc3), 2026-11-08 UTC. This note goes with `cc3-source-preservation-manifest220.json`, which is the list of files to copy. These two files are the only ones I wrote in the repository.

On the source host (`csd`) I ran only `ls`, `stat`, `sha256` and reads of small JSON files, at `nice 19`; the host load was about 49. I copied nothing into the repository and changed no remote or local input. There were no builds, renders, synthesis, Beads, TODO edits or commits. Scratch work stayed in my session scratchpad, outside the repository.

## What root copies

All destinations are `production-overlays-preserved220/` plus the remote relative path, so job names stay as they are.

| Tier | Files | Bytes | Contents |
|---|---|---|---|
| `copy_required` | 23 | 1,256,934 | Kit view root `Cargo.toml` and `Cargo.lock`; engine `crates/fmn/tests/native_render.rs`; the 4 rebind outputs (`narration_units.json` and the label69/label165 direction maps); build19 `source-copy.json` and `resource-receipt.json`; build68 and build164 `source-copy.json`, `qualification.json` and `resource-receipt.json`; rebind69 and rebind165 `rebind-summary.json`, `resource-receipt.json`, `narration/assembly.json` and `narration/report.json` |
| `copy_recommended` | 12 | 31,903,516 | The five guard frozen-input lists (build19, build68, build164, rebind69, rebind165), which are the hashed closure of everything those jobs read; assembly19's 08_5/09_1 unit WAVs plus its assembly, report, summary, receipt and input list |
| `copy_optional` | 22 | 179,067 | Job logs (`command.log`, build/clippy/test output) and assembly19 side records |

Every row gives the host, absolute source path, sha256, bytes and destination. Each row also has a `destination_gitignored` flag and the other records that carry the same hash.

**build164 workspace inventory.** The manifest lists all 71 files.
- 69 have an exact local copy: 66 in snapshot214 at the same path, plus `boundary-label-repair52.rs`, `loop-sums-label-repair164.rs`, and snapshot214's `narration_units.json` as `narration_units.before-label165.json`.
- The other 2 are the rebind165 outputs in `copy_required`.

The build68 workspace is listed the same way.

## What the inspection adds to review205

1. **Correction to review205: the 08_5/09_1 audio in the cut is not local.**
   - Review205 marked the 08_5/09_1 takes as preserved. That is true of the mapped takes (`audio17/08_5.wav`, `audio17/09_1.wav`).
   - The cut uses the assembled units `jobs/csd-production-assembly19/narration/08_5.wav` (`fb88052b…`, 8,574,764 bytes) and `09_1.wav` (`cf37ead1…`, 6,608,684 bytes). Each is 34 bytes smaller than its local take, and no local file has those hashes.
   - Both rebind folders hold the same bytes.
   - The other 103 assembly19 units have exact local copies in `production-audio-stage07-01/native-narration01/`.
   - I only compared hashes; I listened to nothing.
2. **The tree is unchanged since build164 froze it.**
   - Today's hashes of all 4,184 `work/tree` files equal the `work/tree` rows of `two-label-build164-inputs.json`.
   - By class:
     - 4,047 equal franken_manim a399d626;
     - 9 equal my replay of the four tracked engine patches;
     - 62 equal kit revision 589ab3df;
     - 4 equal the tracked kit-patch replay;
     - 58 belong to `work/tree/crate`, which neither the build scripts, `engine-budget.toml` nor the film manifest names;
     - 4 have no local copy: the three `copy_required` tree rows, and `engine-budget.toml`, which reproduces from the tracked template.
3. **Each build's receipts bind its source.**
   - Each `qualification.json` binary hash equals the binary now on the host: `f4836eee…`, `352aa5ff…` and `5a66979e…`.
   - Each guard receipt has exit 0, and its `inputs_sha256` equals both `final_inputs_sha256` and the sha256 of its frozen-input list.
   - Each rebind summary's `map_sha256` and `canonical_sha256` equal the workspace files listed for copying.
4. **build19's source at build time differs from snapshot214 in two files, and both are recoverable.**
   - `source-copy.json` records 66 files. Since the build, assembly19 changed `narration_units.json` (`22492be9…` to `995e25c7…`) and `tools/coherent_narration.py` (`95615cfa…` to `a7fc6705…`), and added 3 files.
   - Snapshot214's `.before-production19` backups hold exactly the build-time bytes.
   - Neither file is compiled into the binary.
   - build68 and build164 differ from their `source-copy.json` only in the rebind outputs.
5. **Cargo configuration.**
   - The build directory picks up `work/.cargo/config.toml` (`79fb8766…`). It is byte-identical to the tracked `.cargo/config.toml` at HEAD and 589ab3df.
   - The build user's `$CARGO_HOME/config.toml` (`14ae6b73…`) sets only `[profile.dev]`/`[profile.test]` debug options and a `target-dir`, which `CARGO_TARGET_DIR` overrides. It does not change the release binary, so it is recorded only.
6. **The rebind69 and rebind165 input lists have no second hash record.** They are bound only through their guard receipts' `inputs_sha256`, which matches. Every other copy row's hash matches at least one other record.

## Cautions for the copy

- Use receipt214's method:
  - refuse if the destination exists;
  - hash on the host before copying, then copy;
  - hash on the host again, and hash locally;
  - write `receipt.json`.
- `.gitignore` would drop 14 destinations silently if anyone ran `git add` later:
  - `**/narration/` (line 50) covers the four `copy_required` rebind `narration/assembly.json`/`report.json` files and four assembly19 rows;
  - `*.log` (line 71) covers the six `command.log` rows.

  This does not matter while commits are prohibited. For a later commit, those files would need a path outside a `narration/` folder.
- The manifest records the host alias and the absolute `/data/projects/…` paths, as root asked and as receipt214 does. Several copied files, including the frozen-input lists and `engine-budget.toml`, also embed host paths. Treat the folder as an archive. Leave it out of any public commit, or strip those paths first.
- Copying preserves bytes on a second disk of the same machine. The single-disk risk ends only when a copy also exists away from this checkout.

## Still open

- No listening by anyone.
- No independent final audiovisual review; the final master is not built.
- Byte-identical binary rebuilds have not been attempted.
- crates.io and git dependencies stay pinned by the lock and the cargo cache; they are not copied.
- Narration remains ElevenLabs Frederick Surrey, as every request and receipt records. That is a recorded setting, not a judgment from listening.
