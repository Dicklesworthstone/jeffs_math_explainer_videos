# CI Coverage, the Windows Promise, and Time Budgets (fm-sol, W1)

What CI runs where, what each lane does and does not promise, and the R22
time budget that keeps the matrix from bloating per-commit CI. This note is
the record; `.github/workflows/ci.yml` is the mechanism.

## What Windows is promised — and what it is not

The `windows` job (`windows-latest`, x86-64, the pinned nightly from
`rust-toolchain.toml`) runs the **fmt / check / clippy `-D warnings` / test
equivalents of `scripts/check.sh`** in bash, including the `fmn-cli --features
batch` and `fmn-output --features ffmpeg-test-fixture` variants, plus a named
step re-running the Windows watch items (§17.4/§17.6):

- **Processor groups above 64 logical CPUs** — `fmn-platform`'s synthetic
  topologies (`from_group_sizes(&[64, 32])`, `fallback(128)`) prove the
  group model holds on a host that cannot physically exhibit it;
- **path/filesystem semantics** in `fmn-platform`'s std implementations;
- **locale pinning in subprocess tests** — the `fmn-output` boundary suite
  asserts the child sees exactly `LANG=C`, `LC_ALL=C`, and a job-scoped
  `TMPDIR`.

**What this promises:** the workspace compiles warning-free and its tests
pass on Windows — *functional* coverage. A Windows break fails the commit
that caused it.

**What this does not promise:** bit-certification. windows-x86-64 is **out of**
the certified matrix — **ADR-0019** ruled it out until bit-identity is measured
on a native Windows host (`../INPUT_CLOSURE.md` §5 — the certified list is
linux-x86-64, linux-aarch64, macos-aarch64), and this job being green changes
nothing about that. Rejoining takes one successor ADR backed by native-hardware
measurement (fmn-dmath vectors, fm-ig3-style corpus hashes, an MSVC object-code
FMA audit). Adding a platform to the certified list is an ADR, because the
list *is* the promise `--reproducible` makes.

## The certified-matrix CI legs

The `certified-matrix` job gives G0-6's frozen matrix its CI legs before W5
needs them (linux-x86-64's leg is the per-commit `portable` job, so the
matrix job carries linux-aarch64 on `ubuntu-24.04-arm` and macos-aarch64 on
`macos-14`). Each leg runs, from day one:

1. the workspace tests — which include the one certified `scene_goldens`
   lock, so cross-platform bit-identity of the locked corpus is exercised
   per leg;
2. the **fmn-dmath cross-platform vector gate** (`--test vectors`) by name —
   the same committed mpmath vectors on every certified target, which is
   what makes bit-identity a CI property instead of a hope;
3. the **PG-5 {1,4,16} thread sweep** on real multicore hardware, exactly as
   `../INPUT_CLOSURE.md` §5 describes.

## The PG-5 thread-count determinism lanes

Thread count is declared *outside* the input closure (§16.7 — "proven inert
under §10.5"), and these lanes are the standing proof. Both run
`scripts/pg5_thread_determinism.sh`, which sweeps the scene-corpus
thread-invariance gate at `FMN_PG5_THREAD_COUNTS` and then the
certified-engine corpus at its fixed {1,4,16}. The counts are env-gated —
the same harness, no code-path change between cadences:

- **per commit** — `pg5-thread-determinism` at **{1,4,16}**;
- **weekly** — `pg5-high-core` at **{1,32,96}**, schedule-only, never a
  merge gate. Runner size is irrelevant to the verdict: the gate proves the
  frame's bytes are invariant under thread oversubscription, not throughput.

Every assertion in both lanes is byte-equality of rendered frames — no
wall-clock assertions anywhere.

## CI time budgets (R22)

R22's rule: **per-commit CI runs the two pinned linux-x86-64 profiles
(portable + one SIMD tier); the full matrix runs weekly.** W1 adds two
per-commit lanes on top of that base — Windows functional and the PG-5 lane —
and everything else stays on the weekly cron (`23 5 * * 1`) or manual
dispatch.

| Lane | Cadence | Runner | Estimate | Basis |
|---|---|---|---|---|
| `portable` | per commit | ubuntu-latest (4 vCPU) | 25–40 min | full `scripts/check.sh`: fmt + 3× check + 3× clippy + 3× test over the 25-crate workspace, plus the wasm32 check and node smoke |
| `x86-64-v3` | per commit | ubuntu-latest | 25–40 min | same gate under v3 `RUSTFLAGS` — the second pinned profile R22 allows |
| `windows` | per commit | windows-latest (4 vCPU) | 40–70 min | the check.sh equivalents above, minus wasm/DAG/node legs; Windows compile+link runs measurably slower than Linux on the same class of runner |
| `pg5-thread-determinism` | per commit | ubuntu-latest | 10–20 min | one crate's two test targets, not the workspace |
| `x86-64-v4` | weekly | ubuntu-latest + qemu-user | 45–60 min | full gate, tier tests under emulation |
| `aarch64-neon` | weekly | ubuntu-24.04-arm (4 vCPU) | 25–40 min | full gate, native ARM |
| `certified-matrix` (2 legs) | weekly + dispatch | ubuntu-24.04-arm, macos-14 | 30–60 min/leg | workspace tests + named vector gate + PG-5 sweep; macos-14's 3 vCPUs are the slow leg |
| `pg5-high-core` | weekly | ubuntu-latest | 15–30 min | the PG-5 harness at {1,32,96} — more render passes than per-commit |
| `exact-image-native` (3 legs) | per commit | arm / macos-14 / windows | 10–20 min/leg | one crate's capability suite per native target (fm-x4pp) |
| `fuzz-codec` | weekly | ubuntu-latest | ≤30 min | scheduled fuzz campaign (fm-ntp), timeout-capped |

**Honesty note on the numbers:** these are planning estimates from job shape
and published runner specs, not receipts; the `portable`/`x86-64-v3` rows are
the only lanes with run history at the time of writing. The first scheduled
weekly run is the receipt that replaces the weekly rows' estimates — if any
lane lands outside its range, the fix is scope or caching, never a quietly
longer `timeout-minutes`. Cache keys are per-(OS, arch, profile) over
`rust-toolchain.toml` + `Cargo.lock`, so a warm per-commit lane should sit
well under the cold estimate above.
