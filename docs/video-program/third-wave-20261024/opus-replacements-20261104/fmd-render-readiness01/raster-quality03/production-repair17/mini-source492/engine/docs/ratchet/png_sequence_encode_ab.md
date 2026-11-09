# PNG-sequence encode: interleaved A/B (fm-hyqz)

**ADR-0024 label: calibration.** This ran on an unqualified host, the shared dev box: dual AMD EPYC 7282, 64 logical CPUs, no isolated slice or cgroup. The load average was 25–40 during the main run. It is not a PG observation, and it feeds no gate verdict.

- **A, the incumbent:** release `fmn` (`--features cli,batch`) built at `3a658eb0`. The PNG sink encodes one frame at a time on the emitter thread, fanning only that frame's fixed 256 KiB DEFLATE segments across the plan's output team (two SMT siblings).
- **B, the candidate:** the same commit plus fm-hyqz. The sink encodes whole frames concurrently on a bounded pool (frames in flight capped by `max_resident_bytes`), sized from `ExecutionPlan::encoder_threads()`: every planned CPU thread.
- **Protocol:**
  - Both binaries ran in one invocation, interleaved per repetition (2 repetitions).
  - Each run was `fmn --robot [--reproducible] --format png_sequence --resolution 1920x1080 --fps 60 --write_all @builtin`: all 25 primitive builtins, 575 frames. Default threads. Timed as process wall-clock seconds with `/usr/bin/time`.
  - Run on 2026-10-07.

| Mode | A: incumbent (s) | B: frame-concurrent (s) | Median A ÷ B |
|---|---|---|---:|
| Certified (`--reproducible`, `CompressionLevel::Best`) | 136.78 · 193.11 | 27.09 · 24.03 | 6.4× |
| Standard (`CompressionLevel::Default`) | 55.83 · 42.77 | 16.36 · 12.72 | 3.4× |

## Countermetrics

- **Bytes:** in both modes the SHA-256 over every published PNG, in path order, is identical between A and B (`1a0354ab216bf99b` certified, `a83e5283594aff3e` standard). The e2e lock also pins every CLI builtin's certified frames (`render_matrix.cli_primitive_builtins.v1`, `render_matrix.cli_camera_builtins.v1`).
- **CPU cost:** B spends more total CPU for the shorter wall time. Certified user time rose from 224–230 s to 310–314 s, and standard from 53–55 s to 57–60 s. System time roughly doubled (thread hand-offs). The bounded pool trades CPU for wall time only while cores are idle.
- **Control:** y4m export doesn't use the PNG sink. `layered_polygon.v1` as certified 1080p60 y4m, A and B interleaved over 3 repetitions: 0.90 · 0.91 · 0.99 s against 0.96 · 0.92 · 0.90 s, with identical y4m digests (`93de26281140456d`). Unchanged.
- **Before this change** (same host, 2026-10-06), certified PNG export was flat beyond 4 threads: `--threads 1/4/16/auto` gave 200 · 113 · 113 · 112 s with about 200 s of user time. That was the encoder bottleneck this change removes. The raster side's per-frame thread spawning (fm-sq8.7 item 2) remains: y4m at 1 thread is as fast as at 32.
