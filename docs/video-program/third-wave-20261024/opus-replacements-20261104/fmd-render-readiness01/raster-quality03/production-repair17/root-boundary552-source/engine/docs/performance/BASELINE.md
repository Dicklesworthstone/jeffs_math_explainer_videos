# Python Reference performance baseline

> W1 / `fm-bgr`, plan §17.1. Captured 2026-07-28 before the performance
> gates are finalized.

## Status and verdict

This is the checked-in calibration baseline for the pinned Python Reference,
not a FrankenManim performance result and not a passing PG-1 gate. The host is
shared and unisolated, and the Reference rendered through Mesa llvmpipe rather
than a hardware GPU. The numbers are therefore suitable for workload
attribution, harness validation, and local target calibration. A PG result
still requires the same benchmark definition on a declared pinned bare-metal
profile.

The premise that “Python churn dominates Manim render time” is **not a
corpus-wide law on this host**:

- Without ffmpeg, the four warm scenes together spend 48.0% of scene time in
  the Python-side bucket and 52.0% in render dispatch.
- Python dominates the text-heavy scene (67.0%) but not the Opening-class
  scene (35.3%), 3D scene (13.8%), or dense-stroke scene (22.2%).
- With ffmpeg, output readback/feed/drain is the largest aggregate bucket at
  42.5%; Python is 30.9% and render dispatch is 26.6%.
- Fresh-process import and setup are themselves material. Summing the eight
  warm group medians, 40.211 s of 54.891 s outer wall time (73.3%) is outside
  `Scene.run`; the Reference import alone has warm medians of 4.103–4.243 s.

The useful ruling is narrower: Python text construction is a demonstrated hot
path, while non-text scenes on this effective software-rendering backend are
render- or output-bound. Optimization and PG calibration must retain all three
buckets instead of treating Python churn as a universal prior.

## Evidence

- Raw data: [`reference-baseline-2026-07-28.json`](reference-baseline-2026-07-28.json)
- Benchmark and profiler:
  [`scripts/profile_reference_baseline.py`](../../scripts/profile_reference_baseline.py)
- Raw-data SHA-256:
  `26b15606b60daa1817639e60227b65ab39fa5ba14b8d6d4102247cc58fd87c1f`
- Benchmark/profiler SHA-256 recorded by the run:
  `e371e0e363d5209787a604bb7d46af09028524e7b38e84cb219b1c3507bd533f`
- Raw schema: `fmn-reference-baseline/1`; worker schema:
  `fmn-reference-baseline-worker/1`

The raw file contains every sample, inclusive and exclusive phase totals,
call counts, worker/outer wall times, final-frame hashes, encoded-file hashes
and ffprobe records, the top 40 cProfile rows for each attribution run, exact
commands, stderr fingerprints/tails, the full `lscpu --json` record, package
versions, and ffmpeg/ffprobe identities.

## Pinned definition

| Field | Recorded value |
|---|---|
| Reference | `3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13`; tracked worktree clean |
| Resolution / rate | 1920×1080, 30 fps |
| Duration | exactly 1.0 s / 30 emitted frames per scene |
| Samples | 32 unprofiled timing runs plus 4 separate cProfile attribution runs |
| Cold state | a newly created, empty XDG cache namespace for each `(scene, mode)` pair |
| Warm state | three fresh-process runs reusing only that pair’s populated namespace |
| Without ffmpeg | no per-frame framebuffer readback, pipe feed, or encode; one post-timing readback validates the final frame |
| With ffmpeg | Reference `libx264` / `yuv420p`, including per-frame readback, stdin feed, concurrent encode, final drain, and publication |
| Process policy | a fresh CPython process and GL context per sample; no cache was cleared or deleted |
| Sample order | cold mode order alternates by scene; warm order alternates by scene + repetition parity |

The four one-second scenes are intentionally small enough to repeat while
covering different mechanisms:

| Scene id | Definition |
|---|---|
| `opening_class` | OpeningManimExample-class native text + coordinate plane, concurrent entrance, affine grid transform |
| `text_heavy` | twelve independently shaped and styled Pango text lines, then two group animations |
| `three_d` | lit sphere + surface mesh, oblique camera, one-second rotation |
| `dense_stroke` | forty independent 96-vertex stroked polylines animated as one group |

These are owned benchmark scenes, not copied Reference examples. They avoid
network assets and LaTeX subprocesses, keeping the measured external-tool
boundary to ffmpeg in the encoded lane.

## Recorded environment

| Field | Recorded value |
|---|---|
| Host | `sensedemobox`; Linux 6.17.0-41-generic x86-64, glibc 2.42 |
| CPU topology | 2× AMD EPYC 7282, 16 cores/socket, SMT2; 64 logical CPUs; 2 NUMA nodes |
| Effective graphics backend | Mesa llvmpipe, LLVM 20.1.8, 256-bit; OpenGL 4.5 Core, Mesa 25.2.8 |
| Python | CPython 3.13.7, GCC 15.2.0 |
| Core Python closure | numpy 2.5.1, scipy 1.18.0, moderngl 5.12.0, moderngl-window 3.1.1, PyOpenGL 3.1.10, manimpango 0.6.1, Pillow 12.3.0, fonttools 4.63.0, diskcache 5.6.3 |
| ffmpeg | `/usr/bin/ffmpeg`, 7.1.1-1ubuntu4.2 |
| ffmpeg SHA-256 | `437c72289719f4145f6677aa0bbd454a151cf482097549270761c5d0a8b04512` |
| Display | Xvfb, 1920×1080×24; all samples reported the same GL identity |

There was no hardware GPU in the measured rendering path. This is recorded,
not normalized away: it explains why these figures must not be presented as
the hardware-GPU Reference denominator for PG-1.

## Timing results

All values are seconds. “Outer” is fresh worker process launch through result
validation and exit; it includes Python import and GL initialization. “Scene”
is `Scene.run`, including encode publication in the ffmpeg lane. A cold cell
is one descriptive sample; a warm cell is the median of three samples.

| Scene | Cold outer, no ffmpeg | Cold outer, ffmpeg | Warm outer, no ffmpeg | Warm outer, ffmpeg | Warm scene, no ffmpeg | Warm scene, ffmpeg |
|---|---:|---:|---:|---:|---:|---:|
| Opening-class | 8.408 | 9.311 | 6.310 | 7.414 | 1.302 | 2.265 |
| Text-heavy | 10.239 | 11.057 | 7.988 | 8.892 | 3.083 | 3.810 |
| 3D | 7.721 | 8.937 | 5.704 | 6.483 | 0.714 | 1.457 |
| Dense-stroke | 7.830 | 9.506 | 5.552 | 6.548 | 0.676 | 1.374 |

Cold cells are retained as raw observations and establish cache semantics, but
one sample per cell does not estimate a distribution. Cross-mode conclusions
below use the three-sample warm medians.

For warm samples, the measured ffmpeg-boundary cost is consistent:

| Scene | Outer-wall delta | `Scene.run` delta |
|---|---:|---:|
| Opening-class | +1.105 | +0.964 |
| Text-heavy | +0.904 | +0.727 |
| 3D | +0.779 | +0.742 |
| Dense-stroke | +0.995 | +0.698 |

Every encoded sample probed as one H.264/yuv420p stream at 1920×1080,
30 fps, and 30 decoded frames. Within each scene, all four cold/warm encoded
files were byte-identical on this host. That is a harness consistency check,
not a cross-platform encoded-video promise.

## Phase attribution

The explicit profiler uses one monotonic, single-threaded nesting stack and
records inclusive and exclusive nanoseconds. The report’s non-overlapping
buckets are:

- **Python:** exclusive `scene_construct` (scene program, object construction,
  and uninstrumented orchestration), `animation_interpolate`,
  `animation_update`, and `scene_update`.
- **Render:** exclusive `render_dispatch` and `camera_capture`.
- **Output:** exclusive `readback`, `ffmpeg_feed`, and `encode_drain`.
- **Residual:** the measured frame/run scaffolding left after those exclusive
  spans.

`encode_drain` is only the final blocking wait. ffmpeg also encodes
concurrently while frames are fed, so the with/without wall delta above is
the authoritative end-to-end output-boundary cost.

The following values are medians of each warm sample’s own phase percentage,
not ratios assembled from independently rounded component medians.

| Scene | Python, no ffmpeg | Render, no ffmpeg | Python, ffmpeg | Render, ffmpeg | Output, ffmpeg |
|---|---:|---:|---:|---:|---:|
| Opening-class | 35.3% | 64.7% | 20.4% | 33.5% | 45.7% |
| Text-heavy | 67.0% | 33.0% | 53.5% | 25.1% | 21.0% |
| 3D | 13.8% | 86.1% | 6.2% | 18.3% | 75.4% |
| Dense-stroke | 22.2% | 77.7% | 11.4% | 29.1% | 59.4% |

Pooling each repetition’s four equally long scenes and taking the median:

| Lane | Python | Render | Output |
|---|---:|---:|---:|
| Without ffmpeg | 48.0% | 52.0% | — |
| With ffmpeg | 30.9% | 26.6% | 42.5% |

The separate warm, no-ffmpeg cProfile runs are excluded from timing medians.
They explain the buckets:

- Text-heavy spends 2.509 s cumulative in `Text.__init__`; 2.356 s flows
  through SVG-string construction/parsing. This is the measured
  Python-churn case.
- The built-in ModernGL vertex-array `render` call accounts for 0.696 s own
  time in Opening-class, 0.422 s in 3D, and 0.460 s in dense-stroke. On
  llvmpipe, render submission/execution is the measured non-text hot path.

## Relationship to permanent FrankenManim instrumentation

The Reference does not possess FrankenManim’s retained render IR, tile
binning, certified color-conversion boundary, or Accelerator Annex, so a
Reference monkey-patch cannot honestly invent those phase boundaries.
`render_dispatch` is the closest Reference aggregate, with cProfile preserving
the function-level evidence beneath it.

FrankenManim’s permanent `fmn-profile/1` facility lives in
[`crates/fmn-platform/src/profile.rs`](../../crates/fmn-platform/src/profile.rs).
It records the full plan taxonomy—scene/update, Python callback, geometry
compilation, render-IR synchronization, binning, raster, color conversion,
annex upload/readback, ffmpeg feed, encode—under
scene → play → frame → phase → tile paths, plus reuse/cache, dirty-tile,
allocation, in-flight, team-utilization, annex-byte, and encode-queue
counters. Disabled profiling has a one-branch hot path and does not read its
clock; enabled smoke coverage exercises real FramePipeline and Lumen paths
with stable NDJSON and folded exports.

That distinction is deliberate: this report measures the Reference at the
boundaries it actually has, while the owned engine retains the finer
instrumentation needed to decide which work to eliminate.

## Consequences for performance gates

1. PG-1 denominators must name both cache state and ffmpeg mode. Combining
   them would hide a 0.698–0.964 s scene-level output-boundary cost in these
   one-second scenes.
2. PG-1 should retain outer-wall and engine/scene measurements. Import/setup
   dominates fresh-process wall here and is directly relevant to PG-4’s cold
   start target.
3. The text-heavy class needs its own workload and Scribe cache counters;
   averaging it into geometry scenes would erase the demonstrated Python
   construction tax.
4. Raster/render and output readback remain first-class targets. This corpus
   rejects a program that optimizes only Python crossings.
5. These llvmpipe figures do not freeze the G2/G4 ratio threshold. Re-run the
   checked-in definition on each declared pinned bare-metal profile, including
   its actual GPU/backend identity, before calling PG-1 calibrated.

## Reproduction

Use a new empty work directory. The runner refuses non-empty work directories
and refuses to overwrite its JSON output; it never clears or deletes caches.

```bash
run_dir="$(mktemp -d /data/tmp/fmn-reference-baseline-XXXXXXXX)"
env PYTHONDONTWRITEBYTECODE=1 \
    UV_CACHE_DIR=/data/tmp/fmn-uv-cache \
  xvfb-run -a -s "-screen 0 1920x1080x24" \
  uv run --no-project --python /usr/bin/python3 \
    --with-requirements scripts/manim_ref/requirements.txt \
    python scripts/profile_reference_baseline.py \
      --work-dir "$run_dir" \
      --json-out "$run_dir.json"
```

The 2026-07-28 run used three warm repetitions (the default). Requirements are
resolved outside the repository; the exact resolved versions are historical
evidence in the raw file. A later resolution with different versions is a new
environment and must produce a new dated raw record rather than replacing
this one.

## Limitations

- This was a shared host, not an isolated or frequency-pinned PG machine.
- llvmpipe is useful for software-rendering attribution but is not the
  hardware-GPU Reference profile required by PG-1.
- One cold sample per pair exposes cold behavior but does not estimate a cold
  distribution.
- The corpus is deliberately short. It exercises representative mechanisms,
  not long-scene steady-state scaling, multi-minute cache behavior, audio, or
  network assets.
- cProfile is attribution evidence only; its instrumented runs are excluded
  from performance medians.
- Encoded-file identity was observed only within this one recorded host and
  run. FrankenManim’s certification contract continues to exclude encoded
  video by construction.
