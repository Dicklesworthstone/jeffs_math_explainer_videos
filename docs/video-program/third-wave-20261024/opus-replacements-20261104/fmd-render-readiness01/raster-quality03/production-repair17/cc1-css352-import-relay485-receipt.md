# Relay 485: implementation receipt (CC1)

CC1 (BronzeJaguar), 2026-11-09 UTC, task 485. Relay 485 is the preserved successor to CC3's relay 394. It carries only the CSS 08_loop_sums import, through frozen 371. It is source only. Nothing ran on either host: no ssh, staging, transfer, render, import or job. Root launches it; CC3 reviews it independently.

## Files

| File | sha256 | |
|---|---|---|
| `cc1-css352-import-relay485.py` | `05617cb88da8786232ad7234a4e3208d024fe3560bd4713ac035f82f07f162d0` | new, 561 lines, sealed |
| `cc3-css352-import-relay394.py` | `371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c` | base, unchanged |
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | pinned helper, unchanged |
| `cc1-master447-manifest-draft476.json` | `3e7b59d0cfb843cd3cd9741f72746139ee39359975d6bd99bb1c69e50cd3b457` | pinned, read for row 08 |
| `08_loop_sums-css-permit474.json` | `92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a` | root's permit, pinned |
| `root-08-host-move480.json` | `9c0d5aab91af6e1e40a53e415e05adcfe00bf7770b5b96f24d7f907eaf13ee40` | root's record, read only |

No other file was written in Q. The local tests are kept outside Q, because this task owns only the relay and this receipt. I can package them into Q if root or CC3 wants them.

## What 485 does

- **One chapter.** `--chapter` accepts only `08_loop_sums`. `local_manifest()` refuses any other chapter too.
- **The order.** `ORDER` is 01, 04, 12, 14, 08, 05. The serial gate is 394's, unchanged: 08 is refused until 01, 04, 12 and 14 each have a completed 371 import on CSD. 08 does not wait for 05, which imports through 448 (relay 472).
- **The 476 route.** The new `route476()` runs inside `local_manifest()`, so `status`, `run` and `verify` all check it. It refuses unless:
  - root's manifest pin has draft 476 row 08's job, worker, worker sha, inputs and permit;
  - the pin's permit is `08_loop_sums-css-permit474.json` at `permit_sha256` 92119d3a;
  - root's copy of permit 474 here is at that sha;
  - draft 476 here is at 3e7b59d0;
  - the attempt's import names are draft 476's.

  The last condition limits the relay to attempt 1, the only attempt controller 476 accepts. Another attempt needs a new draft and controller.

## Delta, 394 to 485

`ast` finds 10 top-level statements added and 10 changed. None were removed, 34 are byte-identical, and the order is kept.

**Added:**
- `CHAPTER`, `DRAFT476` and `DRAFT476_SHA`, `PERMIT474` and `PERMIT474_SHA`, `ROW_PIN`;
- `route476()`;
- `frozen_rows()`, `is_frozen_set()` and `pinned_receive()`, as in relay 472.

**Relay 472's three changes, applied:**
1. **c, the exact receive command and helper hash.** The receipt's `command` must equal `pinned_receive()`, which is 371's `receive_command(Q/371, 08, attempt, Q/manifest)`. That is the same list `start()` requires at admission. 371 check's `helper_sha256` must be ba337aca.
2. **g, the exact path:hash set.** The inputs must be exactly `frozen_rows()` plus the guard's row, as `--prepared` checks:
   - every shared and system row at its manifest hash;
   - Q/371 at ba337aca;
   - the manifest at `--manifest-sha`;
   - Q's permit copy at `permit_sha256`.

   394's count and re-hash checks stay. `POST_PROBE` now also prints `receipt_command` and `listed_rows`.
3. **Stream loss.** A transport failure during the stream now says "run verify --attempt N when both hosts answer" instead of "run status".

**Changed only as above:**

| Statement | Change |
|---|---|
| `VERSION` | 485. Record folders become `relay485-08-loop-sums-a<N>-run<K>/`. |
| `ORDER` | 08 inserted after 14 |
| `local_manifest` | the one-chapter refusal, then `route476()` |
| `check_prepared` | calls `is_frozen_set()`. Its refusal and return value are unchanged. |
| `start` | `pinned_receive()` replaces its former inline command; the list is the same |
| `verify`, `main` | a docstring; `--chapter` choices |

**Unchanged:**
- the helper loading and the CSS, CSD, JOB and INPUTS probes;
- `gather` and its attempt rules: at or below an existing attempt refused, `--prepared`, already imported refused;
- the lane waits, 371 check and prepare, and prepare's output binding;
- admission: the receive first, the send only after admission, never retried;
- the stops and exit codes;
- the record folders, made with `mkdir` and opened with `x`, so nothing is overwritten;
- `status`.

## Root's run

```
python3 -I -B cc1-css352-import-relay485.py status --chapter 08_loop_sums --attempt 1 --manifest NAME --manifest-sha SHA
python3 -I -B cc1-css352-import-relay485.py run    --chapter 08_loop_sums --attempt 1 --manifest NAME --manifest-sha SHA [--lane-wait-minutes M]
python3 -I -B cc1-css352-import-relay485.py verify --chapter 08_loop_sums --attempt 1 --manifest NAME --manifest-sha SHA
```

NAME and SHA are root's 371 export of the CSS 08 job, the same pair controller 476 takes as `--loop-sums-manifest` and `--loop-sums-sha`.

What the relay, or 371 itself, checks before anything starts:
- the export at SHA in Q here, in CQ on CSS and in Q on CSD;
- 371 at ba337aca on all three;
- the CSS 08 render receipt a terminal success, whose receipt and summary are the bytes the export pins;
- 01/04/12/14 imported on CSD;
- a free native lane.

371 check, inside `run`, also refuses on CSD while any `csd-reviewed-final-08-loop-sums*` job exists (371: 504–505), or while root's Q copy of permit 474 is missing or differs. Root's record 480 shows the CSD 08 waiter withdrawn with no job folder created.

Relay 472 is unchanged and does not know 08: the 05 import does not wait for 08. If root wants 05 to follow 08, that is a change to 472's gate, which this task does not make.

## Results (local; macOS, Python 3.15.0)

`test485.py`: 18 of 18 cases pass. Every host call was replaced:
- `remote()` and `spawn()` raise if called;
- `probe()` and `run_371()` return canned facts.

The 08 manifest is a labelled synthetic file: only its names and `permit_sha256` are real, and 371's own `manifest_problems()` accepts it. 371, draft 476 and permit 474 were hard-linked, not copied.

| Case | Shows |
|---|---|
| M0 | 371 accepts the synthetic 08 manifest |
| L1 | the draft 476 route with permit 474 at attempt 1 is accepted |
| L2 | another chapter (14) is refused |
| L3 | attempt 2 is refused (draft 476 binds a1) |
| L4 | a manifest that 371 accepts, but that binds permit 375's name, is refused |
| L5 | permit 474's name at another sha is refused |
| L6 | 394 itself still refuses 08 |
| G1 | 08's serial gate asks CSD only about 01, 04, 12, 14 and 08, not 05; with the four imported it passes |
| G2 | with 14 not imported, 08 is refused |
| V1 | validate a–h on exact facts: no failure under 485 or 394 |
| V2 | a shared inputs row substituted at the same count: 485 fails g; 394 passes |
| V3 | the Q permit copy's row substituted: 485 fails g; 394 passes |
| V4 | the receipt command names attempt 7: 485 fails c; 394 passes |
| V5 | 371 check reports another helper sha: 485 fails c; 394 passes |
| P1 | `--prepared` accepts the exact set and refuses a substituted row |
| S1 | stream loss stops with code 2 and names `verify --attempt 1` |
| A1 | `pinned_receive()` is the tail of 371's guard command for 08 a1 |
| C1 | the command line refuses `--chapter 14_continuation` (argparse, before anything runs) |

**Static checks.** `ast` parses the file, and `delta485.py` gives the delta above. No linter is installed on this machine.

## Limits

- No test drove `run()` end to end for 08: not 18 cases above, and not CC3's `cc3-css352-import-relay394-tests.py`. That file drives 394's run flow for 01 and 04 with root's real manifests, and 485 refuses those chapters, so it was not adapted or run. The run, start and finish control flow is 394's, byte for byte, except two things: the admission command comes from `pinned_receive()`, the same list (A1), and the stream-loss text changes (S1).
- These are local tests on fakes, not an actual import. The CSS 08 render was still running at root's last record (480: `terminal_success` false), and no 08 export exists yet.
- No host was contacted. Nothing here stages, transfers, withdraws or admits anything.
- This is not acceptance of any chapter, import or master. All final master, technical, picture, audio-review, fine-sync, independent AV and public playback checks remain open. There is no audio-review claim. Frederick remains the narration.
- CC3 reviews this delta independently.
