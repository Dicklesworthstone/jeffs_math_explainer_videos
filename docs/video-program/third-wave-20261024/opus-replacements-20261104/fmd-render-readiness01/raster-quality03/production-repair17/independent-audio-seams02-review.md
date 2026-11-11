# Recovery 1006: narrow re-review of master993-audio-seams02 (D1/D2 fixes)

2026-11-10 19:38 UTC, CC0 (LavenderPike). This follows review 1005 (`dba689c8…`), which found D1 and D2 in seams01.

## What was reviewed

| File (CC2's session folder) | sha256 |
|---|---|
| `master993-audio-seams02.py`, 531 lines, 28,283 bytes | `3d4ac2a3ff11a39e032ee3d4f910dd78fc62002d6ec22109a5d02a1e00cceec3` |
| `master993-audio-seams02-handoff.md` | `773c224a2fe90e00253de6fa20d696cec2d50b4c7ea4708b16310b3007d3d2c5` |
| `mutant-seams02-nocleanup.py` (test evidence) | `9907d4d0807056a55c711ef611440a65984f9daa52db88f3388c13a34a666e84` |
| `master993-audio-seams01.py` (baseline, unchanged) | `984f5a6e26b3dd984746b3f10977f69dcd5e2ba6a58183d22e5e245dc5a6ce50` |

The hashes in the handoff table match these.

**Scope:**
- Done:
  - a diff of 01 against 02;
  - an AST comparison;
  - local runs of the selftest and of the mutant, with synthetic arrays and benign local child processes only.
- Not done:
  - no decode or ffprobe of the master;
  - no `check`, `launch` or `measure`;
  - no ssh and no CSD read;
  - no job, and no Beads, TODO or repo edit.

The running overview995 job was not touched.

No audio heard or checked. Nothing here is hearing, fine-sync, AV or release acceptance.

## Verdict

No blockers remain. D1 and D2 are fixed, the receipt command check is exact, and the measurement maths is unchanged. No new limit or gate was added. The 3600 s decode bound is 01's own wait, and the 600 s ffprobe bound is verify995's.

## D1: bounded decode and cleanup (fixed)

The new `stream()` (02:295-329) replaces 01's read loop.

**How it reads:**
- One monotonic deadline of `DECODE_SECONDS = 3600` covers every `select()` and the final `wait()`.
- `os.read()` takes whatever is ready.

**How it fails:**
- A stall raises `TimeoutError`.
- So does a closed pipe with no exit.
- A nonzero exit raises `AssertionError`.

**Cleanup:**
- On every exit from the `try`, the `finally` kills the child if it is still alive, reaps it, and closes the pipe.
- So under guard_csd02, a normal helper exit leaves no live descendant. This closes the gap noted in 1005 (fact 5).
- The guard's stopper still covers stops on the guard's own path.

**ffprobe** now has `timeout=FFPROBE_SECONDS` (600). `check_output` kills the child on timeout.

**Chunk sizes** are now whatever the pipe yields, not 8 MiB reads. The window copy uses absolute byte offsets, so the kept windows, the digest and `pos % 8` do not depend on chunk length.

## D2: guard pin and exact receipt command (fixed)

**Guard pin.** `GUARD_SHA` is `f1cae1db…`, the TODO L4 value. The guard is checked at four points:
- it is in `PINS`, so `gates()` checks it in `check`, `launch` and `measure`;
- `launch()` asserts it before the inputs file is written (02:448);
- it is a row of the inputs file, so the guard's own input check hashes it at the start and at the end;
- `measure()` requires that row (02:392).

The chain beneath it stays pinned by guard_csd02's own `load()`.

**Exact command.**
- `launch()` passes `*command()` after `--`.
- `measure()` requires `receipt['command'] == command()` exactly, plus `phase == 'native'` and no preflight (02:386-387).
- master993 called the same guard the same way (`combined-master993.py` `1955ca32…`, :1269-1271). Its tail also contained `--job` and `--inputs`, which collide with the guard's own options.
- The completed master993 receipt (`f6dd8081…`) recorded that tail exactly: 11 items, starting `/usr/bin/python3 -I -B …/combined-master993.py`, with no leading `--`.
- So on the same CSD Python and guard, `command()` equality will hold.

## Measurement maths (unchanged)

The AST comparison script is `seams02review/astcompare.py` (`21dd42d4…`).

**Functions:**
- 18 top-level functions are AST-identical between 01 and 02: `boundaries_from_summary`, `channel_stats`, `check`, `db`, `deinterleave`, `gates`, `lane_status`, `master_log`, `provenance`, `read`, `residuals`, `save`, `sha`, `success`, `wav_layout`, `wav_window`, `when`, `zero_runs`.
- Changed: `decode`, `launch`, `measure`, `selftest`.
- Added: `command`, `stream`, `selftest_stream`.

**`decode`:** the body of `sink()` is AST-equal to 01's loop body without `pos += len(chunk)`. `stream()` now keeps the position.

**Module assignments:**
- Changed: only `JOB` and `INPUTS` (the new names) and `PINS` (the added `GUARD` row).
- Added: `DECODE_SECONDS`, `FFPROBE_SECONDS`, `GUARD_SHA`.
- Not changed: boundaries, window, lags, pad, header and `NOT_CLAIMED`.

**`measure`:** the only changes are the receipt and guard checks, the ffprobe timeout, and three recorded fields: `guard`, `receipt_command` and `lane995`. The join loop is untouched.

`lane995` is recorded only. No rule reads it.

## Tests run (local, Python 3.15.0, `-I -B`)

**seams02 selftest:**
- `selftest OK (synthetic arrays and benign local child processes only)`, exit 0, 2.34 s.
- A process scan afterwards found no leftover child.

**Mutant** (its only change is that the `finally` body is replaced by `pass`):
- exit 1;
- `AssertionError` at the stall case, as the handoff states.

It left one benign orphan, the 120 s sleeping child (local pid 14221). It was left to exit by itself. Nothing was signalled.

## Lane

`lane_status()` is unchanged, so `launch()` still refuses until verify995 and both picture receipts are finished. The overview995 job is running now. Root serializes and launches; do not duplicate.
