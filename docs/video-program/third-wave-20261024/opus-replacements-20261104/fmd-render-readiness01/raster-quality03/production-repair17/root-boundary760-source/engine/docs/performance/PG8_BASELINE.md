# PG-8 binding-tax baseline

> fm-zoi, plan §15.2 Rev 4 / §17.2. Recorded 2026-07-30 by the canonical
> producer (`fmn_conformance::perf_pg8`) driving the real bridge
> (`fmn_python::perf_harness`) under the `release-perf` profile.

## Status and verdict

This is the **recorded PG-8 class-table baseline**, measured on the shared
development host (AMD EPYC 7282, Linux x86-64, CPython 3.13.7, pyo3 0.26,
NumPy 2.2.4). Following the rig's comparability law, the recorded keys are
honestly unqualified (`bare_metal = false`, `isolated = false`): these
bundles are calibration evidence, and self-evaluation through the common
verifier is `inconclusive`, never a pass. A passing PG-8 result requires the
same benchmark definitions on a declared pinned bare-metal profile after
fm-inr.1's live attestation lands; fm-inr.4 publishes this table at that
point without re-recording.

Until 2026-10-11 the four `baseline.tsv` files claimed `bare_metal true` and
`isolated true` for the same `pg8-shared-dev-host` profile (fm-5wq.32). They
now say `false`, as the samples always did, and
`perf_host::check_baseline_host_claims` refuses any committed baseline that
claims either field for a `profile_id` without a reviewed profile in
`crates/fmn-conformance/tests/artifacts/perf/hosts/`.

The PyO3 0.26 identity above is intentionally historical: these committed
samples were produced on that exact graph. The fm-kg6g security migration to
PyO3 0.29.2 replays the bit-locked bundles and bridge state goldens but does
not relabel or overwrite their producer identity. A new timing baseline would
require a fresh canonical producer run under the same pinned-host attestation
and comparability rules, not an incidental dependency-upgrade run.
Consequently, the current `pg8-definitions` digest intentionally differs from
the four historical `baseline.tsv` files below, and `fmn-perf-pg8 measure-pg8`
refuses those files before reading the clock. A pinned-host publication must
create a new target baseline from the current compiled identities; it must not
edit these old keys to make old timings appear current.

All four workloads are fixed before the clock is read: 64 built-in mobjects
(`point` ×3 + `rgba` ×4 lanes, 4 records each), 60 frames of `dt = 1/30`
per repetition, 24 retained repetitions (21 valid + 3 host-quality budget),
3 untimed warm-ups. Every recorded repetition was valid (0 host-quality
failures). Final scene states are bit-locked self-goldens; the
`native-builtins` bridge state is bit-identical to its pure-Rust twin, and
bit-identical to the `point-transform-callback` state — the ladder's
declared arithmetic contract holding at PG-8 scale.

## The class table

| Class | Sample unit | Published budget | Observed median | MAD (bps) | Per frame | Per mobject-frame |
|---|---|---|---|---|---|---|
| native-builtins | ratio-ppm | 1,100,000 (1.10×) | **3,545,767 (3.55×)** | 86 | 57.2 µs bridge / 16.7 µs twin | 893 ns / 261 ns |
| per-frame-callback | nanoseconds | 108,999,999 | **72,517,928** | 37 | 1.209 ms | 18.9 µs |
| point-transform-callback | nanoseconds | 210,999,999 | **143,938,670** | 49 | 2.399 ms | 37.5 µs |
| dynamic-subclass | nanoseconds | 160,999,999 | **106,000,320** | 24 | 1.767 ms | 27.6 µs |

Budgets for the three nanosecond classes are published as the smallest
x9-boundary value at or above 1.5× the recorded median — the ratchet
starting point: the gate is green at baseline and blocks regressions past
the policy envelope (1,000 bps). The long-term calibration target remains
G0-5's seed costs (§ reading below); closing that gap is the binding-tax
program's optimization work, not a baseline-recording concern.
`native-builtins` keeps the pre-committed 1.10× policy target; the observed
3.55× is the measured gap on a deliberately binding-heavy microbenchmark.

## What the numbers say (the tax, made visible)

- **The default `Scene.update` walk costs ~435 ns per mobject per frame on
  a callback-free scene** (bridge 57.2 µs/frame vs 16.7 µs/frame pure
  Rust). On a workload with near-zero native work that is the 3.55× ratio;
  on real scenes with render work in the frame the same fixed tax amortizes
  toward 1.0×. The rung-1 batched dispatch (`Scene.update_batched`) exists
  precisely to collapse this walk's crossings (mechanism landed by the
  instrumentation half of fm-zoi: dispatch-path crossings 30 → 2 on the
  6×4 corpus).
- **`inspect.signature` inside `_dispatch_updater` is roughly a third of
  per-callback cost.** Dev-profile attribution: dispatch 23.3 µs, of which
  signature introspection 7.7 µs, updater body 9.0 µs. Caching updater
  signatures per callable is the top measured optimization lead.
- **View-based callbacks cost more per call than `set_field` callbacks at
  small record counts** (37.5 µs vs 18.9 µs per mobject-frame): a fresh
  NumPy structured export per frame dominates. The ladder's rungs 2/3
  (declared array/native updaters) remove that cost entirely on the
  declared classes — their frame cost is the pure-Rust twin's.
- **Dynamic-subclass construction is not the tax**; per-frame override
  dispatch is. Construction of 64 subclass instances amortizes to under
  5% of the class's repetition time.

## Evidence

- Raw bundles (24 retained repetitions each, `fmn-perf-samples/1`):
  `crates/fmn-conformance/tests/artifacts/perf/pg8-<scenario>/samples.tsv`
- Phase traces (`fmn-perf-pg8-trace/1`): same directories, `trace.tsv`
- Targeted baselines (`fmn-perf-baseline/1`): same directories, `baseline.tsv`
- Bundle SHA-256 and robust statistics are locked in
  `crates/fmn-conformance/tests/perf_pg8.rs`
  (`committed_pg8_baseline_bundles_replay_through_the_verifier`); any edit
  to the committed bundles fails that test.
- Compiled identities: `fmn-perf pg8-definitions` (four NDJSON records; no
  PyO3 in the shipped binary).
- Real producer: from `crates/fmn-conformance/`,
  `cargo run --manifest-path ../../Cargo.toml --profile release-perf \
  -p fmn-conformance --example fmn-perf-pg8 -- measure-pg8 \
  <current-target-baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>`. The optional final
  `<host-profile.tsv> <host-attestation.tsv>` pair enables the same live
  qualification and postflight authority as the native gate producers.

| Bundle | SHA-256 |
|---|---|
| pg8-native-builtins/samples.tsv | `375b939dd3968268b367f7d900debcbdba88e0491bcf0ebca104ff558f3c95ac` |
| pg8-per-frame-callback/samples.tsv | `78fe983b61944ceca9c4dbb5aa4e63aa9ee39983f58955032680589fe709abd2` |
| pg8-point-transform-callback/samples.tsv | `36cf9c181f9fc1e36e9fc7040e4df7277ed74182e440707483a619cb64d42ef9` |
| pg8-dynamic-subclass/samples.tsv | `ddbf04f085598e7a5af2748c4960cc5299404752c7d645821c02e82a2016d6aa` |
