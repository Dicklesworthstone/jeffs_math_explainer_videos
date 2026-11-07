# Independent algebra guarded-build review01

## Verdict and correction

Original01 helpers are correctly rooted and preserve the intended source/text/test bindings, but their default output is outside the guard's measured WORK/TARGET trees. **Do not use the01 default path while claiming full output-growth coverage.** The guard samples only `five-film-release-20261027` and repo/target; Q/algebra-fixes/build01 is neither. Global free-space monitoring survives, but the copied workspace, saved old binary, new binary and logs evade work and phase-growth accounting.

At root's explicit request I added **prepare_build02.py and build02.sh**, preserving every01 file. The02 output is restricted to `B/algebra-fixes-inputs02`, with B the original guard's measured WORK. build02.sh requires that exact canonical path as its one argument and refuses a symlink output/WORK. Preparation requires the same path and retains refusal if output or the eventual algebra-fixes-build02 job already exists. Old input pins and rebind/test logic are unchanged. New helper names and the generated build02-inputs.json are frozen for the later build. The shared target remains repo/target/geometry-kit01, inside measured TARGET.

**Use distinct preparation and build jobs.** The guard creates its job directory before invoking a child; combining preparation with the same algebra-fixes-build02 job would correctly trip preparation's existing-job refusal. Use algebra-fixes-prepare02 first, then algebra-fixes-build02 after terminal prepare exit0 and invariant inputs. Neither was launched here.

## Checks actually performed

- Imported preparation helpers with bytecode disabled; inferred repository root equals the actual cwd. Eight directory ascents in shell agree with Python parents[7]. No output preparation executed.
- Fresh read-only check_root_build passed **4271/4271 prior manifest rows**, the pinned successful receipt/log/facts/source files, and **122 prior passing test names**. Engine budget path is itself among the frozen rows; lock-verifier and both guard sources are also included.
- Exactly one unit is oliver_11_17. Replacement text is70 words, within the native validator's100-word ceiling. No11_17 direction override exists in stage13. The single script row, unit text, both script hashes and stage14 map binding are updated together. Rebind calls the copied workspace's actual read_units(require_directions=True), not a permissive stub.
- Read-only hold resolution checks all7 changed-file holds: six permutation holds retain lines4124/4174/4186/4198/4285/4342; cosets14_2 moves277→304. Marker, nearest preceding cue and source hash checks are retained. New map alone is added; changed workspace allowlist remains exactly cosets.rs, permutations.rs, script.tsv and narration_units.json.
- Independent in-memory checks exercise expect_build01.py: **9/9 cases passed** (valid123-test log; missing new test; failed; ignored; stale count; duplicate test; valid facts; changed fact; duplicate fact key). Synthetic log additions are checker controls, not claims the new Rust test ran. Filesystem was not used for these fixtures.
- `sh -n` passed for01 and02; Python AST/import checks passed for02. The02 delta preserves all cargo flags, original lock check, strict Clippy, expected123-test identity comparison, fact byte comparison and chapter cmp. Prior24-check fixture output was read but not represented as rerun. No cargo/native build, guard admission or resource preflight was executed.

## Exact root invocation

From the repository root, define these shell variables (Q here is the algebra-fixes directory):

```sh
Q="$PWD/docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/algebra-fixes"
B="$PWD/narration_lab/third-wave-20261024/root/five-film-release-20261027"
G="$PWD/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py"
```

Root must first materialize a **preparation input manifest** at an unused path, represented below as `$PREP_INPUTS`. Its exact rows are the union of the4271 old manifest rows, every current prepare_build02.PINS path (including prior outputs/receipt/log), and every prepare_build02.OWN helper/input path. Deduplicate equal paths, verify each digest, and include the guard files already bound by the old manifest. Do not put future output files in this manifest. No such manifest was written by this reviewer.

```sh
python3 -I -B "$G" --phase native --job algebra-fixes-prepare02 --inputs "$PREP_INPUTS" -- python3 -I -B "$Q/prepare_build02.py" --out "$B/algebra-fixes-inputs02"
```

Wait for terminal admitted exit0, no error/resource stop, unchanged input hash; check the prepare receipt and generated build02-inputs.json. Then:

```sh
python3 -I -B "$G" --phase native --job algebra-fixes-build02 --inputs "$B/algebra-fixes-inputs02/build02-inputs.json" -- sh "$Q/build02.sh" "$B/algebra-fixes-inputs02"
```

The guard captures command.log within B; no external output redirection is needed. Keep original lock, allocation baseline,10GiB admission,8GiB stop,4GiB phase growth,56GiB WORK cap and20GiB target-growth cap. Preparation now contributes to measured WORK and may legitimately be refused. Do not reset the baseline or weaken a gate to force admission.

## Remaining limitations

The prepared02 route has not run end to end. Current disk headroom/admission and concurrent native-lane availability are for the original guard to decide at action time. The old scripts' use of system cargo through the guard's pinned RUSTUP_TOOLCHAIN is preserved; no new toolchain qualification is claimed. This is not picture acceptance of14_14 or11_17, nor acceptance of the retained Frederick takes. Whole-cue assembly, measured timing, glyph/frame review and listening remain subsequent work.

## Exact files reviewed / added (SHA-256)

- `prepare_build01.py`: `058826d2d6973954b358a856969da05cd70d8a668df195f434bf186d5155d9ae`
- `build01.sh`: `5701e56c990c53a22b022491b0e8943545bb6e07f24a9ca4d193238b1afef493`
- `expect_build01.py`: `3bdf70cdc15eae903d2db23e2da9c59ee76b0a8f347eda40c9d2c2c84c21a001`
- `checks/test_build_helper.py`: `5dd5a3fe3abd43dbcfd3d0f79dd47fb33ac690aba2d2e71f5d8c589be63211e0`
- `HANDOFF.md`: `ebaaaee805fd8588957b067f372d44bc9bf0ca86930af3ff19d4ecb621171f43`
- `prepare_build02.py`: `3fda5067c255440876e9235d41ae957d196eed38e564d40c455ce9ec67b03e7c`
- `build02.sh`: `7f964c86ba46e1fb4044944f657ff34465092ec50783b5156b5a65f841d5df9e`
