# Performance-gate evidence contract

`PERF_GATES.tsv` is the machine-readable target catalog for plan §17.2.
It is policy, not a table of achieved numbers. In particular, parsing a row
does not create a passing PG result.

The decision core is `fmn_conformance::perf`. A gate result is attributable
only when all of these identities match a committed observed baseline:

- named pinned bare-metal profile, build profile, and canonical host
  fingerprint;
- exact toolchain and `SUITE.lock` bytes;
- exact benchmark-definition digest;
- gate/scenario, engine, build tier, thread profile, derived ExecutionPlan,
  semantic configuration, cache state, output mode, and optional ffmpeg/tool
  fingerprint.

Every run retains raw repetitions in canonical `fmn-perf-samples/1` TSV. A
host-quality failure is recorded with its measured value and reason, excluded
from statistics, and never silently dropped or averaged in. Evidence records
are path-canonicalized before the bundle is hashed. An observed baseline's
`raw-samples` digest is derived from those exact bytes rather than supplied by
the caller. The evaluator requires that raw bundle and replays its identity,
producer commit, robust statistics, and digest before any `pass`; merely parsing
a plausible baseline TSV is not trusted evidence. Comparable valid repetitions
produce an integer median, nearest-rank p95/p99, median absolute deviation, and
MAD ratio in basis points. Each policy also bounds invalid repetitions, and a
current run must retain the same total repetition count as its observed
baseline; too few valid samples, an exceeded invalid budget, a changed sample
plan, or excessive dispersion is `inconclusive`, not pass. For `exactly`
policies, every valid repetition must equal the target; a median of zero cannot
hide one nonzero mismatch, allocation, or leak sample, and variation among
those values cannot downgrade an attributable exact failure to a
dispersion-only `inconclusive`.
For strict plan bounds expressed in integer nanoseconds (`< 150 ms`, for
example), the catalog stores the greatest admissible integer
(`149,999,999 ns`) so equality at the prose boundary still fails.

## Verdicts

| Verdict | Meaning |
|---|---|
| `pass` | Comparable observed baseline, target met, regression envelope met |
| `alert` | Comparable miss under observe/alert policy, or an alert-band regression |
| `block` | Comparable miss under blocking policy and within the policy's change scope |
| `inconclusive` | Unobserved baseline, identity/host mismatch, malformed evidence, insufficient samples, or excessive dispersion |

PG-8 has `python-only` scope. PG-A has `annex-only` scope, so it cannot mask or
block an ordinary CPU/core change (R21). All alerting timing regressions require
a content-addressed flamegraph or CPU profile below
`tests/artifacts/perf/<run-id>/`.

## Current coverage boundary

Existing repository evidence remains distinct:

- W1's Python Reference report is calibration-only shared-host evidence, not a
  PG-1 denominator.
- PG-5's canonical producer covers the complete committed scene corpus at
  `{1,4,16}` plus the production frame-parallel and ordered-emitter paths. Its
  checked-in output remains host-unqualified because no real pinned-machine
  profile/observation is committed yet; weekly high-core and
  certified-platform receipts are separate evidence, not inferred successes.
- PG-6's primitive steady-state allocation producer covers the complete
  committed scene-golden corpus, and the leak-soak producer covers
  `one-hour-soak-leak` on residency-capable (Linux) hosts. Its peak-RSS
  producer covers a bit-locked three-view UHD 3D gallery through the public
  library and renderer path. All three checked-in runs remain host-unqualified;
  no observed pinned-host PG-6 verdict is inferred from shared-host runs.
- the ignored native Metal probes are PG-A measurement producers, not
  committed pinned-profile baselines.
- Studio's edit-to-frame report remains an input to PG-4, not a whole-gate
  result.

Pinned profile fingerprints, raw observed baseline bundles, and runnable
scenario producers land separately. Until then, target-only baselines must
report `inconclusive`.

## Robot verifier

From `crates/fmn-conformance/`,
`cargo run --manifest-path ../../Cargo.toml --profile release-perf \
-p fmn-conformance --bin fmn-perf -- ...` provides NDJSON-only policy,
evidence, and producer commands. This working directory is part of the
artifact-path contract: canonical evidence paths and baseline source rows are
rooted at this crate's `tests/artifacts/perf/`, while the policy catalog is
`../../docs/performance/PERF_GATES.tsv`.

- `catalog ../../docs/performance/PERF_GATES.tsv` validates the complete policy
  catalog and reports its canonical digest and rows.
- `verify-baseline <baseline.tsv>` validates a versioned observed baseline,
  loads its declared repository-relative raw-sample source, requires the
  canonical `fmn-perf-samples/1` schema, recomputes its robust statistics, and
  verifies identity, producer commit, and exact digest.

Every catalog, baseline, and raw-source input must classify as a regular-file
leaf without following that leaf before its bounded read begins. A declared
repository-relative raw source additionally requires real directory components
throughout its path; links, Windows reparse points, special nodes, missing
nodes, and wrong kinds are refused. Safe `std` cannot bind these checks to the
subsequent path-based open, so verification assumes the inputs are not
concurrently retargeted by their owner.
- `pg2-definitions` reports the exact compiled benchmark-definition and C7/C10
  configuration digests for both canonical raster workloads.
- `pg5-definitions` reports the exact compiled corpus lock, semantic
  configuration, fixed certified ExecutionPlan, one-thread reference
  self-golden, and all three schedule mechanisms.
- `pg6-definitions` reports the exact compiled corpus-lock, benchmark,
  configuration, and frame-result self-golden identities for both the
  primitive steady-state allocation workload and the UHD 3D peak-residency
  workload.
- `pg7-definitions` reports the exact compiled benchmark-definition,
  configuration, fixture-input, and result-self-golden digests for all three
  native-typesetting workloads.
- `pg8-definitions` reports the exact compiled benchmark-definition,
  configuration, fixture-input, and result-self-golden digests for all four
  Python binding classes. This introspection remains in the shipped,
  PyO3-free `fmn-perf` binary.
- `frontdoor-definitions <fmn-executable> <reference.tsv>` probes an absolute
  `release-perf` `fmn` executable, hashes its bytes, and reports the six exact
  PG-1/PG-3/PG-4 process-level definitions. The artifact itself must report its
  build ID, target triple, actual Cargo profile, compiled SIMD tier, and
  embedded `SUITE.lock` digest. The Reference TSV is strict
  `fmn-perf-pg1-reference/1` evidence for the pinned Reference commit and the
  same Opening-class fixture; it is used only by the two PG-1 rows.
- `measure-pg2 <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>` checks
  that the supplied baseline identity names that exact compiled producer, runs
  it, and exclusively creates a content-addressed phase trace followed by the
  canonical raw sample bundle. Both output paths must be distinct canonical
  files below `tests/artifacts/perf/`; their parent directories must already
  exist and may not contain links or Windows reparse points, and existing paths
  are never overwritten. This component preflight has the same safe-`std`
  concurrent-retargeting boundary as replay-source validation.
- `measure-pg5 <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>` applies
  the same identity and exclusive-publication rules to the certified schedule
  matrix. It records exactly three mismatch-count samples: direct `{1,4,16}`
  renders, the production frame-parallel runtime, and that runtime publishing
  through the preallocated ordered emitter.
- `measure-pg6 <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>` applies
  the same identity and exclusive-publication rules to the allocation corpus.
  It renders one warm frame and one caller-buffer/arena reuse frame for every
  committed scene, retaining both frame digests and the engine-owned allocation
  ledger for each measured frame.
- `measure-pg6-peak <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>`
  runs eleven independent complete passes over the certified three-view UHD 3D
  gallery. A concurrent one-millisecond resident-set sampler plus explicit
  before/after probes retains one peak byte count per pass; unsupported hosts
  retain eleven named invalid samples and do no synthetic rendering work.
- `measure-pg6-soak <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>
  <iterations-per-window>` applies the same identity and exclusive-publication
  rules to the leak soak: three windows of full-corpus steady-state rendering,
  one `leaked-bytes` RSS-delta sample per window through the fmn-platform
  residency capability, with unsupported hosts retained as invalid samples.
- `measure-pg7 <baseline.tsv> <producer-commit> <cache-root-or-dash>
  <trace.tsv> <raw.tsv>` applies the same identity and exclusive-publication
  rules to PG-7. `formula-cached` requires a fresh, nonexistent cache root
  below `tests/artifacts/perf/`; the other scenarios require `-`. The producer
  leaves the owned cache root intact as evidence-supporting state and never
  cleans it up or reuses it.
- `measure-frontdoor <baseline.tsv> <producer-commit> <fmn-executable>
  <work-root> <reference.tsv-or-dash> <trace.tsv> <raw.tsv>` drives the actual
  render or Studio process selected by the baseline. The exact `fmn` artifact
  must be `release-perf` and report `git:<producer-commit>`; the benchmark key
  binds its bytes and all compile provenance. `work-root`, trace, and raw paths
  must be distinct nonexistent paths below `tests/artifacts/perf/`. The source,
  edit variant, caches, and product artifacts remain there for inspection.
  PG-1 requires the Reference TSV; PG-3/4 require `-`.

PG-8's real sampler necessarily links the optional `fmn-python` portal and
therefore cannot enter the shipped `fmn-perf` dependency graph. Its stable
robot producer is the Gauntlet-only example target:

```bash
cargo run --manifest-path ../../Cargo.toml --profile release-perf \
  -p fmn-conformance --example fmn-perf-pg8 -- \
  measure-pg8 <current-target-baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv>
```

It drives `manimlib::perf_harness` through the existing dev-only edge, then
routes the observations through the same `measure_pg8`, live-host authority,
canonical sample/trace schemas, and no-clobber publication rules as the native
producers. It accepts the same optional final host-profile/attestation pair.
For a qualified dedicated-cgroup run, build the example first and invoke
`target/release-perf/examples/fmn-perf-pg8` directly; `cargo run` is for
calibration because Cargo would violate the exclusive-process check.
The July shared-host bundles are deliberately historical and their baseline
files do not name the current compiled definition. The producer refuses them;
the pinned-host lane must derive a fresh target baseline from the current
`pg8-definitions` identities rather than rekeying or relabelling old samples.

Every `measure-*` command above accepts one optional, indivisible final pair:
`<host-profile.tsv> <host-attestation.tsv>`. Omitting both is explicit
calibration mode and always records `bare_metal=false`, `isolated=false`, and
null attestation fields in the robot result. Supplying only one is a usage
error. Supplying both invokes the live authority described below before any
benchmark work and repeats the same live checks after the workload; the
attestation output, trace, and raw bundle must be three distinct nonexistent
files below `tests/artifacts/perf/`.

Exit `0` means the requested structural/evidence check succeeded; `64` is
usage error, `65` is malformed, missing, or mismatched data, and `74` is an
output I/O failure. A measurement record has status
`measured-not-evaluated`: even a qualified producer cannot turn a target-only
baseline into green evidence. Without the optional host pair, every producer
forcibly records `bare_metal=false` and `isolated=false` even when its baseline
claims otherwise. With the pair, only the opaque in-process token minted by
the live authority may set both fields; the baseline's profile, host, compiler,
`SUITE.lock`, and build-profile identities must exactly match that token. A
mismatch fails before workload execution. Postflight host drift fails before
any output is published. The common evaluator still requires a committed
observed baseline and replayed raw source before it can pass.

## Pinned-host profile and live-attestation authority

`fmn_conformance::perf_host` owns two bounded schemas:
`fmn-perf-host-profile/1` and `fmn-perf-host-attestation/3`. A profile is an
exact key/value TSV manifest for one machine generation. It names the profile
and platform and pins hashes of `/etc/os-release`, stable CPU identity rows,
non-serial DMI identity, the complete `HardwareTopology` snapshot, and the
storage source. It also pins the exact kernel, eight benchmark CPU IDs,
cgroup-v2 path, governor, turbo/boost leaf and value, thermal sensor and
ceiling, load ceiling, mount point, and filesystem type. Unknown, duplicate,
oversized, traversal-bearing, caller-controlled thermal, and incomplete
profiles are rejected.

On `linux-x86_64`, qualification re-reads every pinned value through bounded
host capabilities and additionally requires all of the following:

- a benchmark CPU set of exactly eight distinct physical cores, whatever the
  machine's total size (ADR-0024 qualifies an isolated slice, not a machine
  shape; the topology digest still pins the whole machine);
- every benchmark CPU in one NUMA node, with the cgroup's effective
  `cpuset.mems` equal to exactly that node;
- no CPU hypervisor flag, `/sys/hypervisor/type`, nested PID namespace, or
  known virtual/cloud DMI marker;
- exact process affinity, and `isolated`, `nohz_full`, and `rcu_nocbs` CPU
  lists that each include the benchmark cores and all their SMT siblings;
- one unified cgroup-v2 whose effective cpuset is exact and whose
  `cgroup.procs` contains only the direct measurement binary's PID;
- the pinned governor on every benchmark CPU, exact turbo policy, temperature
  and one-minute load within their ceilings, and exact mount/source identity
  for the raw artifact path.

The package build embeds `rustc --version --verbose` from Cargo's actual
compiler, the exact `rust-toolchain.toml`, and the exact `SUITE.lock` bytes.
Those compiled identities—not caller text—must match the baseline. The live
authority starts a fixed 250 ms monitor around every qualified canonical
producer. Each sample rechecks process affinity and PID namespace, isolated /
nohz-full / RCU CPU sets, exclusive cgroup membership, governor and boost,
thermal and load ceilings, and artifact-mount identity. It also judges
quiescence from `/proc/stat`, cumulatively from the monitor's start: the SMT
siblings of the benchmark cores may be busy for at most 10‰ of their CPU time,
and all CPUs outside the slice together for at most 50‰, each with one tick of
grace per CPU for USER_HZ quantization in short windows. The monitor retains
the first failure even if the host later recovers and performs a mandatory
final sample, so the final sample covers the whole measurement window. The
attestation records the exact monitor policy and interval, the reserved CPU
set, the NUMA node, and both quiescence ceilings.
After the monitor finishes, the authority repeats the complete static
machine, topology, compiler, suite-lock, affinity, isolation, power, thermal,
load, cgroup, and storage checks. Only then is the preflight attestation
published as content-addressed `host-attestation` evidence, followed by the
phase trace and raw bundle. Publication is create-new/no-clobber throughout;
a later race can leave an obviously partial evidence set, and the tool never
deletes it. Sub-250 ms excursions wholly between samples remain outside this
bounded monitor's observation model; they are not claimed as detected.

For a qualified run, change to `crates/fmn-conformance/` and run the already
built `release-perf` binary directly inside the dedicated cgroup. Store the
reviewed profile under `tests/artifacts/perf/hosts/`. `cargo run` keeps Cargo in
the same cgroup, so the exclusive-process check correctly refuses it. RCH,
containers, VMs, shared runners, missing
sysfs/cgroup leaves, and ordinary developer machines are calibration-only.
The `macos-aarch64` schema family is reserved, but qualification currently
returns a precise unavailable error: `HardwareTopology::fallback` is not
evidence, and no subprocess-based `sysctl` escape is allowed by D-02. A safe
native macOS topology/power/isolation capability and real profile still remain
open under fm-inr.1.

### Baseline-version update ritual

1. Change one reviewed host profile generation; never edit a prior profile in
   place. A kernel, firmware, topology, power, storage, compiler, or suite-lock
   change therefore produces a new `BenchmarkKey`.
2. Build `fmn-perf` under `release-perf`, enter the exact dedicated cgroup,
   cool/quiesce the host, and run every applicable canonical producer with the
   profile/attestation pair. Retain invalid repetitions and partial publication
   artifacts; do not delete or average them away.
3. Commit the profile, host attestation, phase trace, and canonical raw TSV.
   Derive the observed baseline only from those exact bytes and verify it with
   `fmn-perf verify-baseline`; hand-authored medians or copied qualification
   booleans are invalid.
4. Run the complete Gauntlet, compare the prior generation, adjudicate every
   alert/block, then update the scheduled host lane. Core PG rows close on
   real replayable observations from the Linux profile. The Apple profile is
   required only for rows the plan names for Apple silicon — PG-A and G3's
   annex-preview criterion — and no macOS host qualifies until a probe exists
   that D-02 and D3 admit; macOS observations are calibration (ADR-0024).

## Canonical PG-1/PG-3/PG-4 front-door workloads

These six producers deliberately time shipped process boundaries rather than
Lumen microbenchmarks. Build both binaries with `--profile release-perf`; bind
the `fmn` artifact to the source commit by setting
`FMN_BUILD_ID=git:<40-hex-commit>` while building it. A generic release build
identity is valid for distribution but is refused as performance evidence.
The `fmn --robot --version` record is the authority for the custom profile —
the build does not trust Cargo's inherited `PROFILE=release` value.

| Scenario | Production path | Fixed sample plan | Measurement |
|---|---|---:|---|
| `opening-class-g2`, `opening-class-g4` | fresh `fmn --format video` process, 1920×1080, 30 fps, eight threads, pinned ffmpeg | 11 each | integer wall-time ratio in ppm against content-addressed Reference evidence on the identical host fingerprint |
| `export-4k-2d` | fresh `fmn --format video` process, 3840×2160, 30 fps, eight threads, pinned ffmpeg | 24 | emitted frames per second |
| `preview-1080p` | one warm Studio supervisor/worker and real HTTP scrub-to-PNG responses | 24 | one completed preview frame per request |
| `cold-cli-first-frame` | 24 fresh Studio supervisor/worker processes with distinct empty cache roots | 24 | process spawn through the `studio_ready` record, which follows frame zero |
| `trailing-edit-to-frame` | one warm Studio over a 30-second FMTL scene; alternate source bytes, restart, and retain frame 899 | 24 | source write through the restarted retained-frame response |

The Opening-class fixture is one second of native `Text` plus `NumberPlane`:
concurrent `FadeIn`/`ShowCreation`, followed by concurrent matrix and title
transforms. Its exact FMTL digest is self-golden. Preview/cold uses a one-second
Circle transform; the trailing case has a 29-second wait plus one-second
transform and binds both source variants. Every child has a fixed timeout and
bounded robot/HTTP output. Phase traces retain each elapsed interval, source
identity, closure digest where available, ffmpeg identity where applicable,
and declared cache state. Injected blocking regressions still require a
content-addressed flamegraph or CPU profile through the common evaluator.

## Canonical PG-2 workloads

The `fmn-perf-pg2-definition/1` bytes state every workload axis required by
plan §17.2. Both execute Lumen's fast CPU engine at the artifact's compiled
SIMD tier, fixed at eight threads, adaptive AA, 128 px macrotiles, 16 px fine
tiles, and pooled raw RGBA16F output. Fixture construction, render-plan sync,
monotone-table construction, binning, output allocation/prime, warmup, and
every timed raster repetition are distinguished in the
`fmn-perf-pg2-trace/1` evidence. Each definition also binds an exact raw-frame
self-golden. The producer checks the primed frame before timing and refuses
output if the rendered workload drifts; it checks the final frame again so
nondeterminism during the repetitions cannot masquerade as throughput.

| Scenario | Viewport and geometry | Overdraw / style | Throughput denominator |
|---|---|---|---|
| `fill-canonical` | 512×512; 32 closed rectangles, four flat quadratic-line segments each | 32 translucent layers; alpha 0.08 | one output pixel evaluated against one fill layer |
| `stroke-canonical` | 512×256; one 64-quadratic chain with alternating 56 px control and 20 px endpoint offsets | opaque six-pixel miter stroke | one output pixel |

Each run performs three untimed warmups and retains 24 repetitions: 21 valid
observations required by policy plus room for the three explicit invalid
observations the policy permits. Fill times one frame per repetition; stroke
times four to keep timer quantization below the work being measured. Integer
conversion produces `mpx-per-second-milli` without floating-point rounding.
Zero elapsed time or numeric overflow is retained as an invalid sample, never
replaced by a plausible throughput. The final frame is canonically hashed
outside the timed region, and the trace itself is the batch's
content-addressed `phase-trace` evidence row.

## Canonical PG-5 schedule workload

The `fmn-perf-pg5-definition/1` bytes bind all 27 committed `scene_goldens`
cases in corpus order, their certified lock digest, the 320×180 semantic
configuration, certified CPU at the artifact's crate-wide compiled tier, raw
RGBA16F digest output, and an aggregate one-thread reference self-golden. They
also bind the fixed certified offline `ExecutionPlan`: a synthetic 64-core
topology whose planner capability is pinned to the portable scalar definition,
two 32-thread render teams, and at most two frames in flight. The synthetic
topology defines a host-independent replayable schedule contract; it is not a
claim about the measurement host or the separately recorded compiled renderer
tier.

The producer emits exactly three valid `mismatches` samples:

| Sample | Production path | Comparison |
|---|---|---|
| `direct-thread-matrix` | Each prepared scene renders directly at requested thread counts `{1,4,16}` | Four- and sixteen-thread digests against the one-thread digest |
| `frame-parallel` | `FramePipeline` fans the corpus across both certified render teams and emits in sequence | Every emitted digest against its one-thread scene digest |
| `ordered-pipeline` | The same runtime reserves a capacity-two `OrderedEmitter` ring, renders into reservations, and publishes through a reliable digest sink | Every sink digest against its one-thread scene digest |

The exact blocking policy requires all three samples to be zero; one candidate
drift therefore reaches the common verifier as a valid nonzero sample instead
of being hidden by a median. Independently, the ordered one-thread aggregate
must match the compiled self-golden before any evidence is emitted, preventing
corpus or renderer drift from silently redefining the reference.

The `fmn-perf-pg5-trace/1` artifact retains every per-scene digest, all three
mismatch counts, maximum in-flight observations, per-team frame counts, and
the ordered emitter's maximum outstanding reservations. The producer validates
that both teams did real work, every frame was emitted in order, and the emitter
finished with no outstanding or dropped frames. Its batch forces host
qualification false in calibration mode and accepts true only from the live
profile token.

This is the permanent per-commit `{1,4,16}` surface only. Weekly `{32,96}+`
runs and certified-platform receipts are separate lifecycle evidence. When
those receipts are absent or a DSR platform cannot stage the repository, their
status is `inconclusive`; the per-commit producer does not manufacture a
platform verdict.

## Canonical PG-6 allocation workload

The `fmn-perf-pg6-definition/1` bytes bind all 27 committed
`scene_goldens` cases in corpus order, the certified lock-file digest, the
320×180 semantic configuration, certified CPU at the artifact's crate-wide
compiled tier, fixed four-thread scheduling, raw RGBA16F output, and the exact
aggregate frame-result self-golden. This is the
`primitive-steady-allocations` policy row only; it neither estimates nor
substitutes for `gallery-4k-3d-peak` or `one-hour-soak-leak`.

For each scene the producer creates a fresh `FrameArena`, renders one excluded
warm frame to size every typed bump pool and worker scratch slot, then creates a
second `FrameJob` over the same arena and renders into the same frame buffer.
The second job's `heap_allocs_this_frame` counter is retained as one valid
`allocations` sample. The engine owns that counter: every typed-pool capacity
growth and new/outgrown worker slot is counted at its allocation point, without
an allocator shim. The exact policy requires every sample to equal zero, so a
single nonzero scene blocks even when the median remains zero.

The `fmn-perf-pg6-trace/1` artifact retains, per scene, the warm and measured
frame digests, warm allocation count, measured allocation count, reserved arena
bytes, and worker-pool slot count. Warm and measured frame digests must be equal,
and the ordered corpus aggregate must match the compiled self-golden before the
producer emits evidence. The measured batch still forces host qualification
false in calibration mode; this makes shared-host output useful without
inventing a closeable whole-PG-6 verdict.

## Canonical PG-6 4K 3D peak-residency workload

The `fmn-perf-pg6-peak-definition/1` bytes bind three camera states over one
native gallery assembled from `Sphere`, `Torus`, `Cylinder`, and all six
`Cube` faces. Each state renders at 3840×2160 through Lumen's public certified
`ThreeDJob`, with four camera samples, four render threads, fixed 128/16 tiling,
raw RGBA16F output, and default fail-closed 3D preparation limits. The three
resulting frame digests are exact rows in
`gallery_3d.certified.lock`; corpus, camera, renderer, or library drift is a
hard producer error before evidence publication.

One sample is one complete front/quarter/orbit gallery pass. The producer runs
eleven real passes: nine required by policy plus two retained-invalid slots.
It never repeats one observation to enlarge the denominator. While each pass
is live, a one-millisecond sampler polls `current_rss_bytes` concurrently and
combines those observations with explicit before/after probes. The largest
observed `VmRSS` becomes that pass's `bytes` sample, while the phase trace
retains every frame digest and each job's conservative preparation-byte
admission. If the host cannot report resident set size, all eleven observations
are retained as invalid with reason `rss-unsupported-host`; no plausible value
is substituted.

The strict blocking target is at most 1.5 GB. Like the other current producers,
the batch forcibly records `bare_metal=false` and `isolated=false` unless the
live profile token is supplied. The full-tier e2e catalog runs
the exact production UHD gallery and certified lock path; scheduling a weekly
measurement and committing an observed host-qualified baseline remain separate
evidence work, not properties manufactured by the renderer test.

## Canonical PG-6 leak-soak workload

The `fmn-perf-pg6-soak-definition/1` bytes bind the same 27-scene certified
corpus, configuration, engine identity, and four-thread schedule as the
allocation workload, plus the two soak axes: three measurement windows and the
per-window full-corpus iteration count. The count is a real benchmark-identity
axis — the weekly lane sizes it so three windows fill roughly an hour on the
pinned host, and a shorter definition hashes to a different benchmark, so
abbreviated soaks can never masquerade as the scheduled one.

After one excluded warm frame per scene, each window records resident-set size
through `fmn-platform`'s `current_rss_bytes` capability (Linux `VmRSS`),
renders the full corpus for the window's iterations with every frame identity
re-checked against its warm digest, and retains `rss_end - rss_start` (floored
at zero) as one valid `leaked-bytes` sample. The exact policy requires every
window to report zero. Hosts without a residency capability (macOS, Windows,
wasm) retain all three samples as invalid with reason `rss-unsupported-host`
and skip the burn: the gate is inconclusive there, never synthesized. The
`fmn-perf-pg6-soak-trace/1` artifact retains each window's start/end resident
bytes, leaked bytes, and iteration count. `fmn-perf measure-pg6-soak` is the
producing front door.

## Canonical PG-7 workloads

The `fmn-perf-pg7-definition/1` bytes bind the source, semantic configuration,
engine identity, cache state, fixed sample plan, and an exact output
self-golden. All initialization, priming, cache proof, warmup, and final-result
checks remain outside the timed regions and are recorded separately in the
`fmn-perf-pg7-trace/2` evidence.

| Scenario | Timed production path | Required state | Strict target |
|---|---|---|---|
| `formula-cold` | `fmn-tex` math-display layout of the fixed synthetic structural isomorph of the G0-4 corpus median | content cache disabled; engine and CPU warmed; engine memory front cleared before every repetition | < 3 ms |
| `formula-cached` | `fmn-tex` verified store read and bit-exact payload decode for that formula | exact-key miss proven, one-item production `preflight` succeeds, exact stored payload decodes and matches the self-golden, the entry is pinned against in-process eviction, engine memory front cleared before every repetition, exact-key hit proven again after timing | < 100 µs |
| `text-10k-glyph` | `fmn-text::layout_text` with the bundled `FontBook` and plain-text defaults | cache absent; deterministic ASCII source produces exactly 10,000 non-whitespace glyphs, one line, and no decorations | < 20 ms |

The formula result digest is SHA-256 over `Typeset::to_bytes`; the native-text
digest is a versioned canonical hash of every layout, line, glyph, face,
position, span, and style field. A producer refuses to emit timing evidence if
either primed or final output differs. The cached path also refuses an already
warm store: the miss-to-hit transition must be observed in this run, never
declared by caller metadata.

`TexEngine::typeset` answers a repeated source from its engine-local memory
front before it consults a store or lays anything out. A formula repetition
therefore clears that front first, outside the timed region, and reads the
engine's layout, store-hit and memory-front-hit counters on both sides of the
clock. The producer refuses the run unless each `formula-cold` repetition was
exactly one layout and each `formula-cached` repetition exactly one verified
store hit, with no memory-front hit in either. The trace records the summed
counts (`timed_layer`, `timed_layouts`, `timed_store_hits`,
`timed_memory_front_hits`). Trace schema `/1` predates this check (fm-a87y):
from 472a19c0 until the fix, both formula workloads timed memory-front hits
(about 1.7 µs) and no `/1` formula observation is evidence of a layout or a
store hit.

The formula workload derives reproducibly from G0-4's ratified corpus rules
v1: math-mode entries are ordered by UTF-8 byte length, construct count, then
`sha256(mode + NUL + string)`, while occurrence counts supply the weight. The
lower median is occurrence 6,010 of 12,020; its corpus-pair digest is
`5d79ab0a3d5eaf9db101f5dcef2630326c1eff6766a31734e1155f2487988a4d`.
The authored corpus string remains private under G0-4 §15.3. The public timed
fixture `q^7+z` is a synthetic structural isomorph with the median's five
UTF-8 bytes and one harvested construct. The definition binds that fixture
relation, selection rule, rank, median pair digest, rules version, and the
corpus digest
`a8325e49e0ce78fcc735533952740e9adeaaa5cb10f9c13d73aaa3ba4bf883fc`.

Each scenario performs three untimed warmups and retains 24 raw integer
nanosecond repetitions, requiring 21 valid observations and permitting three
explicit host-quality failures. Zero elapsed time or a value outside `u64` is
retained as an invalid sample. A native typesetting error is preserved
verbatim inside the typed PG-7 workload error, including unsupported-construct
name and tier; it is never converted into an empty layout or a timing sample.
