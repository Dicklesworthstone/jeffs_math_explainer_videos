# logarithm-stage09 ws-build recovery 01

Answers BlueSalmon Mail 4120 (bead 87do.63). RubyRaven, 2026-11-05.

I launched, compiled, cleaned and moved nothing. I did not write to target/, W, the lane, EL, canonical source, Beads or the TODO. The sealed render preparation (`fa654b83`) is unchanged.

## What happened in ws-build01

Cargo treated the workspace crate as already built and reused build01's compiled units in `target/geometry-kit01`. It compiled nothing, so the binary is still build01's, with topic `W/logarithm-stage09`.

Why cargo did this:
- Cargo names and fingerprints a path package relative to its workspace root.
- `W/logarithm-stage09` and `WS/explainer_corpus/logarithm_mysteries` are each their own workspace root (`[workspace]` in identical `Cargo.toml` bytes). They hold the same 48 files and the same dependencies, all at absolute W paths, with the same profile, flags and config.
- So ws-build01 resolved to build01's own units, with the same unit hashes.
- Freshness is by content (`[unstable] checksum-freshness = true` in REPO/.cargo/config.toml), and the content is equal. The install also kept the sources' modification times (03:58:31Z, older than build01's 04:57Z units), so a timestamp check would have reported the units fresh too.
- rustc's dep-info records `CARGO_MANIFEST_DIR`. Cargo's own freshness record does not, so moving the crate root is invisible to it.

`diagnose` reproduces this from root's files, read-only:

| Fact | Evidence |
|---|---|
| Clean exit, nothing compiled | ws-build01 exit 0; no `Compiling`/`Checking logarithm_mysteries` line. Finished in 2.87 s (test), 1.20 s (build) and 1.06 s (Clippy); build01 took 7.74 s, 5.74 s and 2.64 s |
| Same test executable as build01 | Both logs run `target/geometry-kit01/debug/build/logarithm_mysteries/c1b3bb6b1a138a1e/out/logarithm_mysteries-c1b3bb6b1a138a1e` |
| Old binary kept | Before = after = `187e119d` = `W/logarithm-stage09-build01-binary`. It names `W/logarithm-stage09` 17 times and the workspace 0 times |
| All 4 units are build01's | Units `c1b3bb6b…` (test), `2e30d21d…` (bin), `1dd2648f…` and `b25ca8db…` (Clippy). rustc dep-info `# env-dep:CARGO_MANIFEST_DIR=W/logarithm-stage09`; newest outputs 04:57:12–04:57:21Z, before the 05:33:21Z start |
| Relocation invisible to cargo | Cargo's freshness records (`fingerprint/dep-*`) hold package-relative paths and blake3 sums only: no absolute root and no `CARGO_MANIFEST_DIR` |
| Only the `.d` was rewritten | Cargo rewrote only the uplifted `debug/logarithm_mysteries.d` (05:34:00Z, now naming the workspace sources). It left the 04:57:18Z binary in place |
| Not RCH | The script exports `RCH_SHIM_LOCAL_IDE=1`, and the guard sets it too. The `cargo` on PATH, `~/.rch/shims/cargo`, then runs local cargo (`exec_local`, shim line 101). All the units are in the local target with local times |

The guard's exit 0 was correct: every cargo command succeeded. The sealed build-verify was right to refuse. The script trusted cargo to rebuild, and nothing checked the compiled topic inside the job.

## Recovery: attempt 02 in its own new target

`recover_ws_build.py build-script --attempt 02` writes `WS/logarithm-stage09-ws-build02.sh`. The script exports `CARGO_TARGET_DIR=<guard target>/logarithm-stage09-ws-build02` and refuses if that folder already exists. A target that has never held a unit compiled from another root leaves cargo nothing to reuse, so it must compile every unit from `WS/explainer_corpus/logarithm_mysteries`.

Against what the sealed generator writes for the same attempt, only these lines change (check B3):

- **New target:** the `CARGO_TARGET_DIR` export, plus a refusal if the folder exists.
- **No shared binary:** the shared-binary preservation lines are dropped, since no binary exists in a new target. The script only refuses if a `before-` file exists.
- **In-job proof:** after Clippy, the job runs `recover_ws_build.py artifacts`. Exit 0 from the guard now means the compiled topic was proved. A replay of ws-build01 makes the job itself fail (check P1).

The three cargo lines and the lock check are build01's and ws-build01's, line for line (B2). The engine config is the same: `W/engine-budget.toml`.

Alternatives I rejected:
- `cargo clean`: forbidden by Mail 4120.
- Moving build01's units aside: it changes the shared cache. Those units would also come back on the next W-root build.
- Changing a per-package profile setting or using `cargo rustc` to force a rebuild: either changes build01's compile flags or commands.

### Commands for root (from the repository root)

```
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-ws-build-recovery01/recover_ws_build.py
python3 -I -B $H diagnose     --w "$W" --ws "$W/logarithm-stage09-workspace01"                # read-only
python3 -I -B $H build-script --w "$W" --ws "$W/logarithm-stage09-workspace01" --attempt 02   # prints the guard line
#   python3 -I -B <guard_owner56g_target20g.py> --phase native --job logarithm-stage09-ws-build02 --inputs <WS>/logarithm-stage09-ws-build02-inputs.json -- sh <WS>/logarithm-stage09-ws-build02.sh
python3 -I -B $H build-verify --w "$W" --ws "$W/logarithm-stage09-workspace01" --attempt 02
```

### What build-verify 02 requires

1. Everything the sealed `prepare_stage09_render.py build-verify` requires:
   - the receipt runs this exact script with the guard's input hash, and the script is input row 0;
   - the lock check passes;
   - exactly one `ok. 241 passed`;
   - the log ends with the 13 chapters;
   - the sums match;
   - the binary names the workspace and never `W/logarithm-stage09`.
2. The input rows include this helper at its current hash.
3. Finished lines `test`, `dev`, `dev`, in that order. Each phase has exactly one line naming `logarithm_mysteries v0.1.0 (WS/explainer_corpus/logarithm_mysteries)`: `Compiling` in test and build, `Checking` in Clippy. No line names another root. Build01's log shows that cargo prints one such line per command (log lines 9, 263, 270).
4. Exactly one `Running unittests src/main.rs (…)` line. Its executable lies under `target/logarithm-stage09-ws-build02/debug/`, names the workspace, and does not name `W/logarithm-stage09`.
5. The log holds the job's own `"artifacts": "PASS"`.
6. `artifacts`, run again:
   - The new target's `debug/logarithm_mysteries` is not `187e119d`. It names the workspace and never `W/logarithm-stage09`.
   - Every logarithm unit's rustc dep-info in the new target has exactly `# env-dep:CARGO_MANIFEST_DIR=WS/explainer_corpus/logarithm_mysteries`.
   - The binary's bytes equal the output of one of those units.
   - Cargo's uplifted `.d` names the workspace `src/main.rs` and not the old root.
   - The workspace copy `WS/logarithm-stage09-ws-build02-binary` equals the binary.

Any difference in cargo's real output fails closed. Then I adapt the check to the real log; I do not weaken it.

### Preservation

The inputs file binds, so the guard proves they did not change:
- all ws-build01 files in WS: script, inputs, `before-`/after binaries and sums;
- the shared `target/geometry-kit01/debug/logarithm_mysteries` (`187e119d`) and its `.d`;
- all 56 files of build01's four logarithm units;
- everything the sealed helper binds: W stage, W engine, repo-view, tools and the complex film binaries.

Nothing is deleted or moved, and the new target stays after the job. One rule for later: a target folder belongs to one crate root. Don't build `W/logarithm-stage09` in `target/logarithm-stage09-ws-build02`, and don't rebuild the workspace in `geometry-kit01`; cargo would reuse units across the two roots in either direction.

### Resources (estimates, not measured)

- **Growth:** a cold target for this crate is at most the size of `target/geometry-kit01`, which is 2.5 GiB and also holds the complex film's units. Root's cold builds of that folder grew it by 1.18 GiB (kit-check02) and 1.00 GiB (lab-build01). The guard allows 4 GiB per native job.
- **Target headroom:** ws-build01's receipt shows target 14.63 GiB against a 3.82 GiB baseline, so 10.81 of the 20 GiB growth cap is used. Free space was 119 GiB and work 44.1 GiB.
- **Time:** about 10 to 15 minutes with the guard's `CARGO_BUILD_JOBS=2`. kit-check02 took 9 min 18 s and lab-build01 2 min 29 s.
- **If the guard stops the job:** its partial target is kept. A retry is attempt 03 with its own new target.

## After a pass

The attempt keeps the sealed names: `WS/logarithm-stage09-ws-build02.sh`, `-inputs.json`, `-binary`, `-binaries.sha256`, and lane job `logarithm-stage09-ws-build02`. So the sealed `prepare_stage09_render.py build-verify --attempt 02` accepts it (check S3), and so does `bind --build 02`.

The versioned bind adaptation for the installed audio helper (Mails 4108 and 4164) is my next task. It will take the build from this helper's build-verify.

## Checks: 29 PASS, 0 FAIL

`checks/run_checks.py SCRATCH --w W`, run twice into fresh scratch folders. The transcripts are byte-equal; one is `checks/outputs/run_checks.out`. Each run takes about 22 s.

- **Real, read-only:**
  - D1–D4: `diagnose` on root's ws-build01.
  - B5: the preserved rows on the real WS and target.
  - B7: `build-script` against the real target root. The script is written into scratch, and the new target is confirmed absent.
  - U1: the real W stage, W, lane, target names, real WS files, and build01's shared units (bytes and modification times) are unchanged.
- **Emulated:**
  - The guard binds the inputs before and after. It does no admission, lane lock or sampling.
  - `cargo` is a stub. It writes the files cargo writes for this crate and prints cargo's lines.
  - Planted modes:

    | Check | Planted mode | Result |
    |---|---|---|
    | P1 | ws-build01 replayed | the job exits 1 |
    | P2 | no Compiling line | build-verify refuses |
    | P3 | one Clippy unit from the old root | the job exits 1 |
    | P4 | tests from another target | build-verify refuses |
    | P5 | binary not a unit's output | the job exits 1 |
    | P6 | cargo also names the old root | build-verify refuses |

  - Tampering after a pass: S4–S7 each refuse.
  - Script refusals (B1, B8–B11): attempt 01; writing an attempt twice; an existing target; launching outside the guard; a target that appears before launch.
- **Not run:** real cargo, the real guard and any real binary. Nothing was compiled.

## Files

| File | Purpose |
|---|---|
| `recover_ws_build.py` | `diagnose`, `build-script`, `build-verify`, `artifacts`. It loads the sealed render helper by hash (`9ddc2003`) and pins its seal (`fa654b83`) |
| `checks/run_checks.py` | the checks above |
| `checks/outputs/run_checks.out` | transcript, 29 PASS |
| `SHA256SUMS` | these files |
