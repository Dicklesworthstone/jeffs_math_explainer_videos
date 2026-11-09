# ADR-0027 — A SIMD-tier binary cannot refuse a wrong-tier CPU itself; only a portable first stage can

**Status:** Proposed (owner decision required: which option below; fm-7wm.10)
**Date:** 2026-09-30
**Bead:** fm-7wm.10
**Amends:** none yet. Option A needs D-02 and D-15/ADR-0016 clarifications, quoted below.

## Context

§17.3 and §20.2 W11 require a wrong-tier launch to fail cleanly with guidance and exit 4 (capability),
never SIGILL. `fmn_cli::check_simd_tier_support` implements that check inside the binary, and
`scripts/install.sh` promised it when a requested tier exceeds the host. Neither holds.

Measured on 2026-09-30 on an AMD EPYC 7282 (Zen 2: AVX2, no AVX-512), main cd6fbf74:
`RUSTFLAGS="-C target-feature=+avx512f,+avx512bw,+avx512dq,+avx512vl" cargo build --release -p fmn-cli
--bin fmn --features cli,batch --target x86_64-unknown-linux-gnu`, then:
- `fmn --version`, `fmn doctor`, `fmn doctor --robot`, `fmn render --help` and an unknown subcommand all
  exit 132: SIGILL.
- gdb puts the fault in `std::rt::lang_start_internal` (`vmovups %zmm0,-0xa8(%rbp)`), in runtime startup
  before `main`.

The same build without `--target` fails earlier: proc-macro2's build script, also compiled with the tier
flags, dies with SIGILL on the build host. CI avoids that with `CARGO_BUILD_TARGET`. ci.yml's v4 lane
already notes that its test binaries "SIGILL under emulation by construction".

The in-binary guard is therefore unreachable, and cannot be made reachable within ADR-0016. The tier flags
are crate-wide, and release thin LTO re-optimizes std's startup under them. `#[target_feature]` can only
add features, never remove them, and ADR-0016 forbids it anyway. The bead's alternative, an object-code
audit showing no tier instruction runs before the check, is refuted by the fault above.

## Decision (proposed; one option to be chosen)

**A. A portable first stage selects the tier.** A tier artifact ships `bin/fmn`, the portable build, and
`libexec/fmn/fmn-<tier>`.
- Before anything else, `bin/fmn`'s `main` detects the host tier (`fmn_platform::topology`) and replaces
  itself with the best supported sibling via `exec` (Unix), passing argv unchanged.
- With no supported sibling, or `FMN_TIER=portable`, it continues as the portable engine.
- Tier binaries are never put on `PATH`, so a wrong-tier SIGILL needs a user to run a libexec file by
  hand. `fmn doctor` reports the chosen tier.
- Windows has no `exec` and D-02 forbids a spawn, so Windows keeps installer-selected single binaries.

Clarifications this needs:
- **D-02** ("ffmpeg is the only subprocess the engine will ever invoke"): add that replacing the current
  process image with a sibling fmn tier binary of the same release (content hash in provenance) is not a
  subprocess.
- **D-15/ADR-0016** ("authoritative code uses neither function-level `#[target_feature]` nor runtime
  feature detection"): allow runtime detection in the first stage only, to choose which whole binary runs.

**B. Installer-only selection, documented.** Keep one binary per tier artifact.
- `install.sh` picks the tier, and warns that a tier above the host's will not start (done in the same
  commit as this ADR).
- `check_simd_tier_support` stays for portable binaries and for tier binaries on capable hardware. Its doc
  now states the limit.
- A copied wrong-tier binary still dies with SIGILL, and the §17.3/§20.2 "never SIGILL" requirement is
  amended to "never via the installer".

**C. Portable only, until tiers earn their place.** Stop publishing tier artifacts until a measurement on
the pinned perf profiles (fm-inr.1) shows a tier's speedup on the PG-2 rasterizer and PG-1 end-to-end
workloads. Tiers stay a build-time and CI option (the weekly matrix keeps them compiling and bit-exact).

Recommendation: **B now, measure, then A or C.** B is true today and costs nothing. A adds a launcher,
two plan clarifications and a new artifact layout, which is worth it only if tiers are measurably faster
for users. No measurement of the tier speedup exists yet. C is the honest default if it turns out small.

## Consequences

- The false exit-4 promise is gone from `install.sh` and `check_simd_tier_support`'s docs (this commit).
- fm-7wm.10's "SIGILL-safety test" has an exact, reproducible probe: the build and run above, on any
  AVX2-only x86-64 host. Under A it becomes an automated check that `bin/fmn` never execs an unsupported
  tier, plus the libexec-direct SIGILL as a documented negative.
- fm-7wm.10 stays open until the owner picks an option. Signing and a published prerelease were already
  owner actions.
