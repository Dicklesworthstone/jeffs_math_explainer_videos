# Stroke probe 02

RubyRaven, for root, answering Mails 5832, 5846, 5847, 5850 and 5852. **Nothing here has been compiled, and no speed is measured.** The patch is unchanged. `complex-stroke-index-candidate01` stays sealed and still holds the patch, and its `HANDOFF.md` still governs the design, the gates and how to read results. Only its commands and the note that `film` runs in parallel are replaced here.

## Why `prepare01` failed
It was my check. `probe01.sh` asked the shell where `cargo` was, under a PATH that put `~/.local/bin` first, so it found csd's wrapper. My fake `rustup` simply printed the answer the check wanted, so no rehearsal could have failed. The new rehearsal's fakes follow what you reported from csd, and the sealed `probe01.sh` now reproduces your exact refusal before `probe02.sh` is tried.

## What changed
**Toolchain (Mails 5846, 5847).** `rustup which --toolchain` finds the real toolchain once. After that, PATH is only that `bin` folder plus `/usr/bin:/bin`, so neither the wrapper nor the RCH shim can be reached. Every `prepare`, `deps`, `base` and `cand` runs these gates first, and the old check is stricter, not removed:
- the folder must be the pinned toolchain's
- `cargo` and `rustc` on PATH must be that folder's
- both binaries must match your SHA-256 pins
- `rustc -Vv` must show the pinned commit

**Home cargo config (Mails 5850, 5852).** No `CARGO_HOME` machinery. The file must match your pin `14ae6b73…` or be absent, and a legacy `~/.cargo/config` is refused. `CARGO_TARGET_DIR` beats its `build.target-dir`, and each step fails if its output didn't land under `$WORK`.

**Inherited environment.** `RUSTFLAGS`, `RUSTC`, the wrapper variables, `CARGO_HOME`, `CARGO_BUILD_*` and `CARGO_PROFILE_*` are refused by name. If the storage environment leaks in, you'll see which variable did it.

**Other repairs.** These cover what my fakes couldn't show, or answer your review:
- `film` now runs one binary at a time (Mail 5832).
- `TMPDIR` and `XDG_CACHE_HOME` now sit under `$WORK`. Renders would otherwise have written to `/tmp` and `~/.cache`, where the guard can't see them.
- `deps` is a separate step that can be rerun. Before, a dependency failure left `tree/` behind and blocked a second `prepare`.
- `cand` can be rerun. Before, a stop after patching left a tree that failed its own pin.
- `cargo metadata` now uses `--filter-platform`, because as I recall cargo's source, an unfiltered run wants every platform's crates offline. That is not tested here.
- `CONFIG` is pinned, the copied tree is made writable, and reading `rustc -Vv` can no longer lose a race with `grep -q`.

## Running it
Stage `complex-stroke-index-probe02/` beside `complex-stroke-index-candidate01/`, because the script reads the patch from that sibling. The rest of your staged layout works as it stands. `ab_bench.sh` is not among your staged inputs yet, and `ab64` needs it with hash `9a3e6084…`.

`WORK` must stay absolute, and it is the same folder your guard calls `TARGET`.

```bash
env WORK=$LANE/work SOURCE=$LANE/inputs/frozen CRATE=$LANE/inputs/complex_analysis_visual \
    CONFIG=$LANE/inputs/repo/.cargo/config.toml bash probe02.sh prepare
env WORK=$LANE/work bash probe02.sh base
env WORK=$LANE/work bash probe02.sh cand
env WORK=$LANE/work AB=<absolute path>/ab_bench.sh bash probe02.sh ab64
env WORK=$LANE/work AB=<absolute path>/ab_bench.sh THREADS=24 bash probe02.sh ab4k
env WORK=$LANE/work bash probe02.sh film      # optional, before any adoption
env WORK=$LANE/work bash probe02.sh profile   # optional, not rehearsed
```

`SOURCE`, `CRATE` and `CONFIG` are needed only by `prepare`. If `prepare` reports a missing crate, fill the cache and run `deps`; `prepare` will refuse a second run.

**Check these first:**
- **Guard limit:** your guard sets `TARGET` to the whole work tree, so its 4 GiB per job applies to the cold release build and to `cand`'s test build plus relink. I have no measured size for either. Cargo keeps finished units, so `base` and `cand` both resume if rerun after a stop, and a stop says nothing about the candidate. Whether to rerun or raise the bound is your call.
- **Offline crates:** `deps` is still the first real test of them. The film lock has 276 packages and the engine lock 305.
- **Home config:** the pin assumes `[build]` holds only `target-dir`, as I read Mail 5850. Any `rustflags` or `rustc-wrapper` there would reach both binaries.

## Still different from production
- `cargo` runs directly rather than through `rustup run`. It is the same pinned binary, and both sides are built the same way.
- Tests pick up the home config's dev and test profiles. Mail 5852 shows no release keys, so `base` and `cand` should not.
- The tree path differs, so `base` is not expected to match production's binary bytes. Identity here compares `base` with `cand`, not either one with production.

## Evidence
| Check | Result | What it does not show |
|---|---|---|
| Sealed `probe01.sh` under csd-shaped fakes | your exact refusal | |
| `probe02.sh` with its real pins, under fakes | refuses the fake cargo | that the real binaries match the pins |
| Copy differing only in the 3 fake-tool pins: 6 planted faults | each refused, none created `tree/` | |
| Same copy: dependency gap, then `deps`; `cand` stopped after patching, then rerun | both recovered; tree ended on the candidate hashes | |
| Same copy: `ab64`, `ab4k`, `film` with the real `ab_bench.sh` and patch | all passed; a chapter changed in the candidate gave `IDENTITY FAIL` | a real build or render |
| Wrapper marker across the whole rehearsal | never reached | real PATH behaviour on csd |

The cargo, rustc and home-config pins are your reported values. I can't check them here, but `CONFIG` matches this repository's file. Real `rustup`, `cargo`, `rustc`, `perf` and csd have not been run by me.
