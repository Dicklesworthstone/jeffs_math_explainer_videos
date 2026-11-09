# Controller 393: brief notes (CC0)

cc0, 2026-11-09. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

Nothing has run on any host: no launch, import, render or master. 393 accepts no picture, motion, sound, sync or AV result. No audio heard or checked.

## Files

| File | sha256 | Author |
|---|---|---|
| `csd-master-controller393.py` (current, 359 lines) | `8fb9a8c1cf524b55da68cdf52a27b43159f64caa1377fa52332bde9720ecd05b` | CC0's source plus root401's post-import check |
| `before-postimport-root401-controller393.py` | `60ccf765ae233f105e052293e6c45e0fb3e9e0b9dd8913f3a62084289d61cf6b` | CC0's source, kept by root before its 401 edit |
| `cc0-combined-master350.py` (pinned) | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | frozen |
| `master350-manifest-draft384.json` (pinned) | `ddbe3f2b4225fc580e14e8f3890c63caa6e51061e52dcb2664e1680773436968` | CC0's draft |
| `csd-css352-import371.py` (pinned) | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | CC3's, frozen |

Root's 401 edit only adds lines; it removes none of CC0's. CC0 made no edit after it, so root's 401 tests and 427 replay apply to the current bytes. 393 is frozen at `8fb9a8c1`.

## What 393 does

- **Before waiting**, it checks the 15 rows of draft 384 against:
  - 350's row shapes and RETAINED pins;
  - the import311 a1 names;
  - for the five CSS rows, the names that frozen 371's own `job_name(chapter, 1)`, `inputs_path()`, `css_inputs()` and `css_job()` give.
- **It waits on 24 receipts:**
  - 4 import311 and 4 retained renders;
  - 5 import371 and 5 CSS renders;
  - 6 CSD renders, 09 included.

  A foreign render receipt is checked only after its own import is successful.
- **What counts as successful:** 350's own `success()`, plus `admitted` not false and `preflight_only` not true. The receipt must read the same bytes twice, `STABLE_SECONDS` (60 s) apart. A CSD receipt whose guard pid is still live counts as pending.
- **Before the launch:**
  - it writes `Q/master350-launch-manifest393.json` once. That file holds draft 384's rows unchanged and a block that claims nothing. A later run accepts the file only byte for byte;
  - root401's `verify_css_imports()` re-runs frozen 371's `check()` on each CSS import. It requires import-result.json's `rows350` to equal the row's names.
- **The launch** is frozen 350's own `launch --manifest Q/master350-launch-manifest393.json --wait-hours 0`, under 350's own guard. 393 returns 350's exit code.

## Stops

393 stops, launching nothing and never retrying, when:
- any receipt finishes without success;
- a pin changes (393 itself, 350, draft 384, 371 or 350's guard);
- the master job folder or `Q/combined-master350-inputs.json` exists;
- another 393 run or a 350 launch/master process is running;
- the deadline passes (exit 3).

A later attempt needs a new manifest and a new controller version.

## Commands (on CSD, from Q)

```
python3 -I -B csd-master-controller393.py check
python3 -I -B csd-master-controller393.py run --wait-hours N      # 0 < N <= 96
```

What `check` returns:
- it reads only;
- exit 0 when all 24 receipts are successful, 3 while any is pending, 1 on a failure.

Staging 393 needs in CSD Q: this file, draft 384 and 371 at their pins. 350 is already there (root 379).

## State after root's 2026-11-09 update

Root reports that the CSS 352 gallery job (05) failed and is preserved. Draft 384's 05 row names that job and its a1 import, so 393 as pinned cannot launch:
- if the a1 import is never made, 393 stops at its deadline;
- if the import is made, the failed render receipt stops 393.

A replacement 05 path is root's 447/449, outside these notes.

## Tests (local, synthetic; not CSD evidence)

**CC0's test393** (a scratch file outside the repo, sha `6124da603e03c792…`): 37 of 37 pass on `8fb9a8c1`.
- Stubs: `subprocess.run`, `time.sleep`, the `/proc` scan, and root401's `verify_css_imports`.
- Cases covered:
  - draft 384's 24 receipts;
  - 6 refused row edits, including an a2 import name;
  - 16 receipt states;
  - 14 run scenarios: launch, idempotent manifest, a different manifest, prerequisite failure, deadline, pending then finished, a change between the two stable reads, the master job appearing, the master inputs existing, the guard changing, 350 exiting 1, a wait of 0.

**Root's tests:**
- 401: five synthetic post-import cases;
- 427: a reviewer harness replay, 29 of 29.

**Not tested locally:** the Linux `/proc` scans (`no_duplicates`, `ancestors`, `live_guard`), a CSD run, and root401's check against real imports.

## Known limits (no change made)

- An exception other than an assertion, such as a missing `import-result.json`, ends 393 with a traceback and exit 1 rather than a `refused` line. Nothing is launched either way.
- A CSS render that fails on CSS shows on CSD only as an import that never arrives. 393 then stops at its deadline.
- `check` refuses once the master job exists.
