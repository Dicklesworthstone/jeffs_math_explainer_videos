# combined01 build and launch helpers: CC0 executable review (review01)

CC0 (LavenderPike), 2026-11-10T22:21Z. This review covers CC2's frozen files:
- the `csd/` folder, with SHA256SUMS `48a77b17…`, all of whose entries recompute OK;
- handoff.md `a548a1d5…`.

| File | sha256 |
|---|---|
| csd-combined01-build.py | `bfb59efdcd2c55d07d6ea5c602265162e4858ebc77ce842c78c7af04e3e8d156` |
| csd-combined01-launch.py | `ff9083eb230661b1744c8b1fea9a1887999baad508e782e2bd111d9cba209630` |

Only source reads and cheap local checks were made. I did no build or SSH, and wrote nothing on CSD. No audio heard or checked. This is not native acceptance.

## Verdict: no blocker

The helpers are fit to launch as staged.

## What was checked

**Template delta.** `diff` against csd-native844-build.py `7932fe78`, the helper build844 ran, shows only the changes the docstring declares:
- the pinned bundle and the four substitutions;
- four fmt checks;
- 247 tests, plus the 12 named new tests;
- toolchain pins, together with preflight844's tool_hashes;
- new names (job, source, binary, metadata, preflight file).

Unchanged from 844: the base761 group and signals, POLICY, the receipt checks, dependency_check, OLD's two extras, the config checks, affinity and environment, the test regex form, plate_fade_probe, the Clippy argv, `cargo rustc`, facts, chapters, and the final listing, dependency and config re-checks.

**Old-source and base hashes.**
- The 760 manifest is pinned (`5afe200e`), and its listing equals base761's LISTING_506.
- `bundle_check` asserts that each of the four 760 rows equals the bundle's base and that each replacement equals SUBST. It also asserts the combined listing `689a5731…`.
- OLD is hash-checked against the 760 rows except for boundary, Cargo.toml and the plan, as in 844. That proves OLD's the_bump, loop_sums and mod are the 760 bases.
- Every copied file is re-hashed against the combined rows. Cargo.toml is the exception: as in 844, its Linux bytes are asserted equal to OLD's.

**No overwrite.**
- The helper refuses an existing SOURCE.
- Every file write is `xb` or `x`: the copies, the configs, the preflight, base761's step outputs and qualification.json.
- The binary path is asserted absent before `copy2`.
- The launcher asserts JOB, INPUTS and SOURCE are all absent, writes the inputs with `x`, runs the guard once with `check=True` and never retries.
- `--check` refuses an unfrozen launcher and writes nothing.

**Guard and toolchain.**
- POLICY and the guard argv are as for 844.
- Every one of 844's 21 non-tree, non-OLD guard inputs is pinned at the same hash, from CC2's probe02 rows. The one exception is csd-native844-boundary.rs, which the staged boundary replaces. allocation-baseline is bound by hash at launch, as before.
- The inputs manifest also binds all 4125 work/tree rows and all 72 OLD files. 844 bound the 70 expected OLD rows, so this is a superset.
- cargo-rch-real is required, and the five tools are pinned to build844's recorded hashes.

**Tests.**
- `NEW_TESTS` is exactly the set of `#[test]` functions in the four replacement files that are absent from their 760 bases: the_bump 1, loop_sums 4, mod 0 and boundary 7. None is removed. 235 + 12 = 247 (new-tests01.out).
- main.rs declares `mod chapters`, so libtest prints `test chapters::<module>::tests::<name> ... ok`. That matches the form in CC2's probe02 samples from build844.

**rustfmt steps.** The same rustfmt build (1.10.0-nightly 9085017724) and the 760 rustfmt.toml were run on a combined src copy. All four `--check` steps exit 0, on both the combined and the 760 trees (rustfmt-steps01.out). The mod.rs step recurses into all 19 chapter module files, so it covers more than its name says, and it still passes.

**Qualifying the exact new binary.**
- SOURCE is a new path, so cargo must compile the crate afresh; the engine path dependencies are unchanged.
- `cargo rustc` writes target/release/complex_analysis_visual, which is copied to `complex_analysis_visual-combined01`.
- facts and chapters are run on that copy, and its sha256 goes into qualification.json.

**Expected SOURCE listing, for root after the run.** preflight-combined01.json and qualification.json should record `source_listing` = `[72, "ada599ee55047e764ba1bd60be1c1d50d5731fbb61f23cfb98cac9140c76438f"]`. The same derivation reproduces build844's recorded `[72, 90591a15…]` exactly (source-listing01.out).

**Disk.** build844 grew the target by about 1.1 GiB. Root's `--check` shows 18.9 GiB free, against a stop at 8 GiB.

## Low items (not blockers; no change asked)

- **The new binary is not proven fresh.** The helper does not assert that the copied binary differs from 844's `e7525873…` or 268's `983954c5…`. Root can compare `binary_sha256` against both after the run.
- **Clippy noise.** Per root, the "unclosed table" noise was not investigated. The cargo exit code and the scoped tests remain the gate.

## Limits

- Nothing was run on CSD by me. The CSD facts come from CC2's read-only probes and root's `--check`.
- The test outcome, the Clippy outcome, the facts hash and the binary itself are for the native run to decide.
- The narration rebind rows in CC2's handoff are render-time and were not reviewed here.
