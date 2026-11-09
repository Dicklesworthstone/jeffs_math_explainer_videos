# 486: offload readiness and the native Mac build (CC3), sealed

CC3 (WhiteSwan), 2026-11-09, about 06:15 UTC. Q is this folder. All probes were bounded and read-only. I made no build, staging, transfer, launch or tracker change.

## Status

- **The target host is mac-mini-max.** mac-mini-old is unusable (CC1's 499); its findings are kept below because three of them apply to any Mac. The root is `~/scratch/jmev-complex-native-20261109` on the internal APFS volume. The source is CC0's 506 tree (listing `770e5239`), which root's 511 transfer reproduced on the host.
- **The helper is `mini-max-native-build507.py`.** Its sha256 is in the receipt line at the end. It is new and has not run on any host. 494 was never written, so there was nothing to preserve.

## Helper 507

**What it runs.** It keeps 268's contract word for word:
- the flags `--release --locked --offline --config SOURCE/engine-budget.toml --manifest-path CRATE/Cargo.toml --bin complex_analysis_visual`, run from the crate folder;
- test: exactly one result line, 233 passed, 0 failed, 1 ignored;
- `plate_fade_probe -- --ignored --nocapture`: 1 passed;
- clippy `--no-deps -D warnings`, then `rustc -C metadata=jmev_review_repair268`;
- facts sha256 `7d528509…`, and the 15 chapters;
- `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=2`, `RCH_SHIM_LOCAL_IDE=1`, and PATH = toolchain bin, `/usr/bin`, `/bin`.

**What differs from 268, and why:**

| 268 on the build host | 507 on mac-mini-max |
|---|---|
| copies production-repair19, then the 8 candidates | none: 506 is already that source; its listing must be `770e5239` (4,197 files) before the first Cargo step and after the last |
| `cargo fmt` step | none: no source mutation |
| `rustup which … cargo`, Linux file hashes pinned | `rustup which … rustc`; runs `cargo-rch-real` when present, else `bin/cargo`. It refuses any of the four tools that is not a thin arm64 Mach-O binary. It pins the cargo version and rustc's commit `90850177…`, the same nightly as 268, and records the four hashes |
| `cargo clippy` | toolchain `cargo-clippy clippy …` with `CARGO` set to that cargo: what `cargo clippy` dispatches, without a rustup proxy from `CARGO_HOME/bin` |
| `sched_setaffinity` to 4 CPUs | `RUST_TEST_THREADS=4`; `RAYON_NUM_THREADS=4` comes from the guard |
| `TMPDIR`, `XDG_CACHE_HOME` in the work folder | `ROOT/tmp`, `ROOT/cache` (created if missing); `CARGO_TARGET_DIR` is the guard's `ROOT/target` |
| binary copied to the work `bin` folder | `ROOT/jobs/JOB/complex_analysis_visual-507`, with `preflight507.json`, the step logs and `qualification.json`, all written with `x` |

**It refuses before Cargo** unless all of these hold:
- **The job.** `ROOT/jobs/JOB` holds only `resource-receipt.json` and `command.log`.
- **The receipt.** It is admitted and not finished, for the native phase, with the policy 20/8/4/80/20/10.
- **The guard's environment.** `CARGO_TARGET_DIR` is `ROOT/target` and `RUSTUP_TOOLCHAIN` is `nightly-2026-08-31`.
- **File systems.** These lie on APFS: the source, target, tmp, cache and job folders; `CARGO_HOME`, its `registry` and its `git`; and the toolchain directory, all by real path.
- **Environment.** There are no rustflags, rustc or rustdoc overrides, and no `CARGO_PROFILE_`, `CARGO_TARGET_` (other than `CARGO_TARGET_DIR`), `CARGO_BUILD_RUSTC*` or `CARGO_UNSTABLE_` variables.
- **Cargo configs.** The only ones on the crate's path are `SOURCE/.cargo/config.toml` (bound by the listing) and the `CARGO_HOME` config. That one may not set `target`, `profile`, `patch`, `source`, `env`, `unstable` or `replace`, or build flags, rustc or the target.

**Root's launch, once CC2's guard 508 is sealed:**

```
python3 -I -B <guard508> --phase native --job JOB --inputs MANIFEST -- <python3.11+> -I -B <path>/mini-max-native-build507.py --job JOB
```

- **The job name.** Give the same JOB to both, and use a new one for each attempt.
- **The inputs manifest.** It should list the helper and the 4,197 source files, by absolute path.
- **Timeouts.** The helper bounds each step itself: 14,400 s for the first test step, which builds the cold target, and 7,200 s for each of the others. macOS has no `/usr/bin/timeout`. On a timeout cargo is killed; its rustc children finish their current unit and exit.
- **An assumption to check.** 507 assumes guard 508 keeps 490's layout under the new root: `ROOT/jobs` and `ROOT/target`, with the same policy. If it differs, 507 refuses at the binding check; it does not build.

**Local evidence.** `test507.py` (`670c1c0a…`) passed all 20 of its stand-in checks on this Mac, with no Cargo build, test or clippy run (output `test507-01.json`, `8360b430…`; both are scratch files outside Q). The cases:
- **Listing.** It reproduces CC0's 506 listing `770e5239` and 492 listing `32f99c83`. One changed byte changes it, and a symlink is refused.
- **Toolchain identity.** It passes on this Mac's own nightly (version queries only). A script and a universal binary are not accepted as thin arm64, and the file-system lookup gives `apfs` here.
- **Cargo configs.** A target-dir-only home config passes. Flags and profiles are refused, and so is an unbound ancestor config. The source's own config is allowed.
- **The guard.** Its binding passes on a fresh admitted stand-in job, and it refuses a stray file, a wrong policy, a finished receipt and another target.
- **Environment.** A profile variable is refused.
- **The real file.** With its real constants it refuses here (exit 1) and creates nothing.

## Findings that carry over to mac-mini-max

1. **RCH wraps the toolchain's cargo.** On this Mac and on mac-mini-old, `bin/cargo` in `nightly-2026-08-31-aarch64-apple-darwin` is RCH's script, `b854c942…`. That is the file 268 pinned as "cargo" on the build host, so 268 ran cargo through this wrapper with `RCH_SHIM_LOCAL_IDE=1`. The real cargo is `bin/cargo-rch-real`, which is `619e58ab…` on this Mac. 507 runs the real binary and refuses scripts.
2. **The other tool hashes match across Macs.** This Mac and mac-mini-old have the same rustc `0c43651a…`, cargo-clippy `de6e9364…` and clippy-driver `32ed579e…`, the official release bytes. Root can compare mac-mini-max's `preflight507.json` against them.
3. **Cargo merges the home config.** mac-mini-old's `~/.cargo/config.toml` sets only `build.target-dir`, which `CARGO_TARGET_DIR` overrides. Its non-interactive SSH environment also set `CARGO_TARGET_DIR` to an ExFAT folder and put the RCH shims first in PATH. The guard and 507 override all three. `preflight507.json` records what 507 used.
4. **The target will be cold, and the 4 GiB cap may bind.** This is an estimate. 268's whole target on the build host was 2.2 GiB (receipt 272: 2.17 GiB at admission, 2.20 GiB at the end), so a cold target for the same steps should come to roughly 2 to 4 GiB. That is close to the guard's 4 GiB per-job growth. If the guard stops with 75, Cargo keeps the finished units. Whether to rerun is root's and the owner's call (CC2's 498, N3).
5. **The facts on arm64 are a real gate.** 507 requires the facts sha to equal 268's `7d528509` byte for byte. If Apple's libm prints a digit differently from glibc, 507 stops after writing `facts.stdout`. Root then compares it with the build host's facts output before anything renders.

## mac-mini-old (unusable; kept for the record)

- **The git cache is on ExFAT.** `~/.cargo/git` is a symlink into the ExFAT drive. Its packs carry AppleDouble `._pack-*.idx` files, which git reports as "non-monotonic index". Five of the six locked revisions resolve. `franken_markdown 126a5304` is absent.
- **So is the toolchain.** Its directory resolves onto ExFAT, and 60 of the 120 files in its std library folder are AppleDouble copies.
- **The registry is complete.** All 231 locked crates are cached, on internal APFS with 12 GiB free.
- **EERCHExport** is an APFS image whose backing file sits on the ExFAT drive.
- **A clean git source.** This Mac's Cargo git cache holds all six revisions, on APFS with no AppleDouble files. It can supply any Mac that lacks one.

## Other hosts (brief)

- **TRJ and TS1 are not ready with 352's bindings.** They hold only production19-era imports: no binary 268, no delivery309 and no 352 worker. A route there needs a new worker and imports, about 0.73 GB of runtime and the 96 MB delivery. I don't recommend it.
- **The CSS serial route (474's) is the only one that needs no new code** for 06, 07, 10 and 11. By closure 424 the movie, timeline, sampling and sheets are already on CSS. Each permit transfer carries 5 or 6 Q records:

  | Chapter | Bytes |
  |---|---|
  | 06 | 100,398 |
  | 07 | 71,766 |
  | 10 | 84,108 |
  | 11 | 144,737 |

## Not claimed

- No build, test, clippy, render, transfer or staging on any host. 507 has not run anywhere.
- The Mac binary cannot equal 268's Linux binary `983954c5`. Timelines and frames from it are not compared yet.
- No listening, AV, picture or motion claim.

Helper: `mini-max-native-build507.py`, sha256 `8e5db00a40c58ce37fc7643de7eff0421f8aa86ac33eb7b31043d7d2c62c5cfb`.
