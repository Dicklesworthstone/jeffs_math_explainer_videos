# master993-audio-seams02: handoff to CC0

RubyRaven (CC2), 2026-11-10. This is a sealed scratch helper. It has not been launched, and it ran no decode and no
ffprobe. There was no CSD or CSS job. No active file, repo file, TODO or Beads was touched.

| File (CC2 scratchpad) | sha256 |
|---|---|
| master993-audio-seams02.py (531 lines) | 3d4ac2a3ff11a39e032ee3d4f910dd78fc62002d6ec22109a5d02a1e00cceec3 |
| master993-audio-seams01.py (preserved, unchanged) | 984f5a6e26b3dd984746b3f10977f69dcd5e2ba6a58183d22e5e245dc5a6ce50 |
| mutant-seams02-nocleanup.py (test evidence only) | 9907d4d0807056a55c711ef611440a65984f9daa52db88f3388c13a34a666e84 |
| CC0 review answered: recovery1005-master993-audio-seams01-review.md | dba689c8324897c70e67804e7df10d9caa431034e325e52310653e286260f04c |

## Changes

### D1: bounded decode and child cleanup

The new function `stream()` replaces the read loop in `decode()`.

- **Deadline.** One monotonic deadline of 3600 s covers the whole read and the exit. 3600 s is the wait seams01
  already had.
- **Reading.** `select()` waits at most the time left, and `os.read()` takes whatever is ready.
- **Timeouts.** A stall, or a pipe that closes without the process exiting, raises `TimeoutError`.
- **Cleanup.** On every way out, `finally` kills ffmpeg if it is still alive, then reaps it.
- **Unchanged.** The window copy and hashing are seams01's loop body, moved verbatim into `sink()`.
- **ffprobe.** It now has `timeout=600`, as verify995 does.

### D2: guard pin, inputs binding and exact receipt command

- `GUARD_SHA` is f1cae1db, the TODO L4 value. `GUARD` is also in `PINS`, so `gates()` checks it.
- `launch()` asserts that hash before the inputs file is written and adds `GUARD` to the inputs file.
- `measure()` requires the guard row at f1cae1db.
- `measure()` also requires the receipt's `command` to equal `command()` exactly. That is the argv `launch()` passes
  after `--`, which the guard records unchanged (checked against the master993 receipt).
- It also requires `phase == native` and no preflight.

### Other changes

- **Lane state.** `seams.json` records the 995 lane: the terminal flag and each receipt's finished and exit values,
  with the time they were read. The helper sets no rule on it.
- **Names.** The job is `csd-master993-audio-seams02`, with inputs `Q/master993-audio-seams02-inputs.json`.

## What did not change

These 18 functions are byte-identical to seams01, compared with `ast`:

- `channel_stats`, `residuals`, `zero_runs`, `deinterleave`;
- `wav_layout`, `wav_window`, `provenance`, `gates`;
- `boundaries_from_summary`, `master_log`, `lane_status`, `check`;
- `save`, `success`, `sha`, `read`, `when`, `db`.

The constants are unchanged too: boundaries, window, lags, pad, header, `NOT_CLAIMED`. There are still no
thresholds and no acceptance claims.

## Tests (local, Python 3.15.0)

`python3 -I -B master993-audio-seams02.py selftest` prints the OK line, exits 0 and takes 2.4 s.

- **The arrays tests from 01 still pass.**
- **Five benign local Python children are run through `stream()`:**

  | Child | Result |
  |---|---|
  | Clean, 3 x 64 KiB | count, contiguous positions, exit 0 |
  | Writes 8 bytes, then sleeps 120 s | `TimeoutError` at the 1 s deadline, killed |
  | Closes stdout, then sleeps 120 s | `TimeoutError`, killed |
  | Writes forever while the sink raises | `RuntimeError`, killed |
  | Exits 3 | `AssertionError`, returncode 3 |

  After each, the child must be reaped (`os.kill(pid, 0)` finds no process).
- **Mutant.** A copy with the `finally` cleanup removed fails the stall case, because the child is left alive.

## Not done, not claimed

- No `check`, `launch` or `measure` run anywhere, so no CSD read for this step.
- No audio heard.
- No ASR, pacing, fine sync, AV or release acceptance.
- Optional rows CC0 named but did not ask for were not added: the guard's own pinned dependencies, and the
  `/usr/bin/timeout` wrapper.

Lane at handoff, as root reported: complex 995 passed at 19:27, and the overview picture job is running.
`launch()` refuses until both picture receipts are finished. Root serializes and launches. Do not duplicate.
