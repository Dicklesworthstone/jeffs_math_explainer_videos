# Portal planar route: interleaved A/B (fm-sq8.11)

**ADR-0024 label: calibration.** This ran on an unqualified host, the shared dev box: dual EPYC 7282, no isolated slice or cgroup. The load average was 17–40 during the run. It is not a PG observation, and it feeds no gate verdict.

- **A, the incumbent:** a wheel built at `28eeb9df`. Every portal frame takes the camera route.
- **B, the candidate:** the same commit plus fm-sq8.11. Planar vector content under the default camera frame takes Lumen's retained 2D route.
- **Protocol:**
  - Both wheels ran in one invocation, interleaved, with the order alternating per repetition (3 repetitions).
  - Each run was `fmn-python SCENE --format y4m --fps 30 --threads 8 --resolution 1920x1080`, timed as process wall-clock seconds.
  - Run on 2026-10-04.

| Scene | Frames | A: camera route (s) | B: planar route (s) | Median A ÷ B |
|---|---:|---|---|---:|
| SquareToCircle (Tex, square → circle) | 150 | 17.68 · 20.88 · 19.43 | 7.71 · 8.40 · 8.16 | 2.38× |
| Flat (Transform square → circle) | 23 | 5.25 · 4.89 · 4.87 | 1.94 · 1.92 · 1.91 | 2.55× |
| Curved (surface; the control) | 60 | 13.59 · 14.09 · 12.64 | 14.39 · 12.84 · 13.00 | 1.05× |

## Countermetrics

- **The control scene didn't move.** The surface scene takes the camera route on both wheels, and its times agree within noise. So the speedup belongs to the route change, not to the host.
- **Equality with native fmn:**
  - On B, the portal's certified PNG sequence for a pure-2D scene is byte-identical to native `fmn --reproducible` replaying the same scene's FMTL/1 export: 23/23 files. On A it matches 0/23 frames.
  - In standard mode at one thread, it matches 23/23 files on B and 0/23 on A.
  - `render_matrix.python_portal_planar_route.v1` keeps the standard-mode equality.
- **Equality with the camera route:** an identical picture forced onto the camera route differs by at most 8 luma codes (mean 0.0017, p99 0). This is checked in `crates/fmn-python/tests/scene_execution.py`.
