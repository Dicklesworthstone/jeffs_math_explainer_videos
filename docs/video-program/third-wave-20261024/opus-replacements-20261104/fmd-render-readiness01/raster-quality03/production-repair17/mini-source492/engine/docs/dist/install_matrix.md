# Install Matrix and SIMD Build Tiers (W11)

FrankenManim ships **one artifact per (platform, SIMD build tier) pair**.
SIMD is selected at *build* time, not at runtime: each artifact is compiled
with exactly one certified target-feature set from `SUITE.lock`, and
`cfg(target_feature)` picks the compiled kernels (ADR-0016 — crate-wide build
tiers, no function-level `#[target_feature]`, no runtime dispatch, no per-call
`unsafe`). There is no binary-local choice to make at launch; selecting the
artifact *is* selecting the tier.

## Native artifacts and the optional Python portal

W11 ships two deliberately separate artifact families (ADR-0017):

| Artifact | Host requirements | Entry point | Python source |
|---|---|---|---|
| `fmn-<version>-<platform>-<tier>` | none; ffmpeg remains optional | `fmn` | rejected with a named `fmn-python` capability error |
| `franken-manim` wheel for the declared CPython/platform ABI | supported host CPython and NumPy | `fmn-python`, plus `import manimlib` | supported through the portal |

The native archive never contains or downloads CPython, libpython, PyO3, or
NumPy and never searches `PATH` for an interpreter. The wheel contains the
compiled portal and bundled FrankenManim assets, not a Python runtime. Its
CPython ABI/tag matrix, `manimlib` namespace-conflict policy, and clean-venv
render proof remain the acceptance surface of fm-vsq.

## The tiers

| Tier | Instruction set | Meaning |
|---|---|---|
| `portable` | baseline (x86-64 or AArch64, no optional extensions assumed) | Runs on every supported machine. Always available, always correct — the reference tier. |
| `x86-64-v3` | AVX2 + FMA + BMI2 class | x86-64 microarchitecture level 3. |
| `x86-64-v4` | AVX-512 (F/BW/VL/DQ) class | x86-64 microarchitecture level 4. |
| `aarch64-neon` | NEON (the AArch64 baseline) | All 64-bit ARM targets; NEON is architecturally mandatory there. |

The tier set is capped at these four by policy (plan risk R22): no partial or
improvised feature sets are supported tiers.

## The matrix

| Platform | Tiers shipped | Certified-determinism status |
|---|---|---|
| linux-x86-64 | portable, x86-64-v3, x86-64-v4 | **Certified matrix member** (§16.7) |
| linux-aarch64 | portable (≡ aarch64-neon) | **Certified matrix member** |
| macos-aarch64 | aarch64-neon | **Certified matrix member** |
| windows-x86-64 | portable, x86-64-v3 | Functional from W1; **out of the certified matrix** per ADR-0019 (revisit trigger in that ADR) |

On AArch64 the portable and NEON tiers coincide (NEON is the baseline), so one
artifact serves both names.

## Artifact naming and selection

**Today:** CI builds each tier as a separate job with the exact `SUITE.lock`
feature set in `RUSTFLAGS` (`+avx2,+bmi2,+fma` for v3; `+avx512f,+avx512bw,
+avx512dq,+avx512vl` for v4; `+neon` for aarch64; none for portable), and the
tier-equivalence corpus holds every compiled tier to the scalar definition.
The per-tier *release artifact* packaging and its download-time selection UX
land with W11's binary/wheel work (fm-vsq and siblings).

**Target convention (recorded here so packaging implements it, not invents
it):** artifacts named `fmn-<version>-<platform>-<tier>`, tier spelled as in
the table above. Pick the highest tier your hardware supports; when in doubt,
install `portable` — it is never wrong, only slower. A build cannot
dynamically widen itself on a newer CPU, and a v3/v4 artifact on unsupported
hardware fails at process start (an illegal-instruction class failure), not
mid-render.

`fmn doctor` reports both sides of this choice so a mismatch is visible
before any render:

- `hardware_supported_tier` — the highest tier the machine supports, from
  `fmn_platform::topology` (`SimdTier::name()`: `portable`, `x86-64-v3`,
  `x86-64-v4`, `aarch64-neon`).
- `active_compiled_tier` — the tier compiled into the running binary
  (`portable`, `x86-64-v3`, `x86-64-v4`, `aarch64+neon`).

This works only where the binary can start. On unsupported hardware a tier
binary cannot run `fmn doctor` either. An x86-64-v4 build stops with SIGILL in
Rust's runtime startup, before `main`, measured on an AVX2-only host
(ADR-0027). Choose with the portable binary's doctor, or let `install.sh` pick.

**Naming divergence, recorded honestly:** the two reports use different
spellings for the ARM tier — `aarch64-neon` (hardware side, from
`fmn-platform`) versus `aarch64+neon` (compiled side, from `fmn-cli`). They
name the same tier. Unifying the spelling is a cosmetic code change, not a
behavioral one.

Running a tier *below* your hardware is fully supported (that is the normal
case for portable installs). The compiled tier is part of the input closure
(§16.7, C3/C7), so two artifacts at different tiers are different certified
identities — bit-identity holds *within* a tier across the certified matrix,
and tier-vs-scalar equivalence is proven per tier by the engine-equivalence
harness (`Tier::ALL` holds every tier to the scalar definition byte-for-byte;
ADR-0013/ADR-0016).
