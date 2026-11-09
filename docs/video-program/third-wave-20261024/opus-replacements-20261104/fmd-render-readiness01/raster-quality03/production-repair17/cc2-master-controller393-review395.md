# Review 395: master launch controller 393 (CC2)

cc2, 2026-11-09, about 04:00 UTC. This is an independent review of CC0's `csd-master-controller393.py` as frozen, which includes root's 401 post-import gate. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

Nothing ran on any host. I sent no host command and did not touch the live jobs. All tests below ran locally, in memory, against the real local files: draft 384 and frozen 350, 371 and 393. Every host path was served from an in-memory store, and nothing was written. No audio was heard or checked.

## Verdict

- **No High and no Medium finding** in 393's code. It does what its notes say: a tolerant wait, then root's post-import recheck, then exactly one frozen 350 `launch --wait-hours 0`, with no retry.
- **Status: do not run 393.** Its 05 row is the CSS 352 gallery job, which root reports failed. 393 can never launch: it would wait for an a1 import that cannot come and stop at its deadline with exit 3, launching nothing. Root's replacement path for 05 is 447/449.
- **Three Lows.**
  - F1: root's post-import check runs after the launch manifest is written. The same lines are in 449.
  - F2: some refusals end in a traceback rather than a `refused` line. This confirms a limit CC0's notes already list.
  - F3: wording only.

## 1. Files

| File | sha256 | State |
|---|---|---|
| `csd-master-controller393.py` | `8fb9a8c1cf524b55da68cdf52a27b43159f64caa1377fa52332bde9720ecd05b` | reviewed; 359 lines; frozen (CC0) |
| `before-postimport-root401-controller393.py` | `60ccf765ae233f105e052293e6c45e0fb3e9e0b9dd8913f3a62084289d61cf6b` | CC0's source before root 401. The diff adds lines only: `verify_css_imports()` and its call |
| `cc0-master-controller393-notes.md` | `11a33478b84f93f42272d7e3c004f2966586d033647da6b081b9bfe9102fa5a5` | read |
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | pinned; `launch`, `gates`, `chapter_gate`, `preflight`, `master` read in full |
| `master350-manifest-draft384.json` | `ddbe3f2b4225fc580e14e8f3890c63caa6e51061e52dcb2664e1680773436968` | pinned; all 15 rows checked |
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | pinned; naming helpers, `receive`, `check` read |
| `root-review378-374-disposition390.json` | as in Q | the post-import rule |

## 2. The actual 350 interface, against 393

| What | 350 (frozen) | 393 |
|---|---|---|
| Command | `launch --manifest M --wait-hours N`. M must be absolute, symlink-free and directly in Q, and 0 ≤ N ≤ 48 (:1201, `absolute()`). | `[python3, -I, -B, Q/cc0-combined-master350.py, launch, --manifest, Q/master350-launch-manifest393.json, --wait-hours, 0]`. `run` asserts its own resolved folder equals 350's Q, so the manifest path is symlink-free too. |
| Names | `JOB = csd-combined-master350` and `INPUTS = Q/combined-master350-inputs.json` are constants. The guard is `D/guard_csd02.py`, native phase. The guarded child is `master --job JOB --inputs INPUTS --manifest M` (:1233–1235). | It checks the same two paths with `master_absent()`: at start, at every poll, before the manifest write and before the launch. |
| No duplicate | Refuses if the job or INPUTS exists (:1203). The inputs file is created exclusively. | `master_absent()`, plus a `/proc` scan for another 393 `run` or any 350 `launch`/`master` process (§4.2). |
| Waiting | Each row waits on its import receipt, else its render receipt. A finished failure stops it. Every 30 s it calls `read()`, which raises if it catches the guard rewriting a running job's receipt in place. | 393 does its own waiting, in which an unreadable receipt counts as pending. It calls 350 only with `--wait-hours 0` after two equal all-successful passes, so 350's read can't meet a rewrite. |
| Before spending | `gates()` (every row), `preflight()` (a new attempt-numbered folder in the jobs' TMPDIR, needing that folder to exist), then 12 GiB free. | All are 350's, unchanged. |
| Outputs | `master-summary.json`, `staged-inputs.json`, `master-native.log`, `captions-native.log`, and `final/` (master, SRT, VTT). | 393 reports only 350's exit and the master receipt's top-level `exit` and `finished`. It records no completion itself. |

The draft 384 rows pass every row-shape gate in 350's `chapter_gate()` (§4.1). Its CSS names are the ones frozen 371's `job_name(chapter, 1)`, `inputs_path()`, `css_inputs()` and `css_job()` give.

## 3. Findings

### F1 (Low). The post-import recheck runs after the launch manifest is written

- **Where:** in `run()`, the write of `Q/master350-launch-manifest393.json` (:315–321) comes before `verify_css_imports()` (:322).
- **Root's rule:** 390 says "first real import requires post-import check before master manifest adoption."
- **What happens:** nothing is launched before the check, so no unchecked import reaches the master. But when the check refuses, Q is left holding a launch manifest whose block says it was written "only after its prerequisite pass", and a later run accepts those bytes again.
- **Replay:** the 401 refusal leaves the manifest written and launches nothing. In the success case the order is: write, 401 check, `master_absent`, `no_duplicates`, launch.
- **Repair (in a successor, not 393):** call `verify_css_imports()` before the manifest write.
- **449:** `csd-master-controller449.py` as read (`3e8d5fa215399b590a27ce95591c6df7ff3d05f8667c419b5b5ac8247ecfb38b`) has the same order: the write at :330, the check at :333. So this applies to it too. I have sent it to CC0, who reviews 449 as 454.

### F2 (Low). Refusals that are not assertions end in a traceback

This is the limit CC0's notes list; I confirmed it in the 401 gate.
- A missing `import-result.json` raises `FileNotFoundError`.
- A 371 `check` that prints anything before its JSON raises `JSONDecodeError`.
- Both escape `main()`'s `AssertionError` handler: exit 1 with a traceback and no `refused` line.

Nothing is launched in either case. There is no change to make in frozen 393. A successor could catch `Exception` in `main()`, or turn these reads into assertions.

### F3 (Low, wording). `live_guard()` says "guard pid"

- In the owner guard in this repo (`run_bounded_phase_oliver_owner48g_target14g_20261102.py:149`), the receipt's `pid` is the guarded child's, not the guard's.
- That guard writes `finished` only after the child has exited and the inputs have been re-hashed (:167–177). So for a finished receipt, `live_guard()` can return true only if the pid has been reused by a process whose command line names the job.
- It is harmless. I did not read `guard_csd02.py` itself.

### Notes (no change needed)

- **N1.** The display pass before the launch (:310) prints states without asserting them. There is no gap, because 350's `launch --wait-hours 0` re-checks that every row's receipt is a terminal success.
- **N2.** `check` mode does not run root's 401 recheck; only `run` does. `check` exit 0 therefore means "all receipts successful", not "imports rechecked".
- **N3.** The launch manifest keeps the draft's 15 rows and drops draft 384's own `draft384` block, with its pending list and its "not a launch manifest" status. 350 reads only `chapters`. Root's adoption of those pending items is root's record, not 393's.
- **N4.** 393 waits up to 96 h, then runs the master in the foreground. Root must start it detached from any ssh session.

## 4. Tests (scratch, outside the repo)

| File | sha256 | Result |
|---|---|---|
| `replay395.py` (first version, kept unchanged) | — | stopped at `run`'s CSD-only assertion. On this Mac `/data` resolves through `/Volumes`; root's 398 records it |
| `replay395b.py` | `7f52cacd301de823e908837bb832645204154913164a5cd835822a61f930ca12` | **35/35**; output `736a7f05…` |

Method: 393 is loaded at `8fb9a8c1`, and 350 and 371 through 393's own `load()`. Paths, `sha`, `/proc`, time, subprocess and 371 `check` are replaced in memory per case, then restored. I wrote no file anywhere, and the local `/data` tree has no project folder.

### 4.1 Rows (10 cases)

- Draft 384 is accepted, with 24 receipts: 5 import371, 5 CSS render, 4 import311, 4 retained render and 6 CSD render.
- Nine mutations are refused:
  - a CSS a2 import;
  - other CSS inputs;
  - a retained a2 import;
  - rows out of order;
  - 09 given as combined;
  - another CSD job;
  - an extra key on a CSS row;
  - import keys on a CSD row;
  - another retained worker.

  These ran in the first replay, which got through this part.

### 4.2 Receipt states and /proc (25 cases)

**Receipt states** (in the first replay; all as the notes say):
- *Pending:* missing, half-written, unfinished, and a render whose import is pending.
- *Failed:* refused admission, preflight-only, exit 1, resource stop, changed inputs, a recorded error, no `inputs_sha256`, and a render missing after its import succeeded.
- *Successful:* a good receipt.

**`/proc` scans** (CC0 had not tested them):

| `no_duplicates()` case | Result |
|---|---|
| only this run and its bash wrapper | clear |
| a second 393 run | refused |
| a 393 `check` alongside | clear |
| a 350 launch | refused |
| the 350 master under its guard | refused |
| a 350 `check` alongside | clear |
| a process that exits before its cmdline is read | clear |

- `ancestors()` returns this process and its wrapper.
- `live_guard()`: a live pid naming its job is true; a reused pid, a gone pid and a foreign receipt are false.

### 4.3 Root's 401 gate (13 cases)

- Five valid imports are accepted, with each row rechecked.
- Refused:
  - the import receipt finished with exit 1;
  - the import inputs changed after the receipt;
  - `import-result` names another manifest hash;
  - `import-result` `rows350` name a2;
  - the export manifest is for another chapter, with every hash consistent;
  - 371 `check` reports a problem, exits 1, is not terminal, or has different `rows350`.
- Two refusals escape as exceptions (F2): an extra output line, and a missing `import-result.json`.

### 4.4 `run()` (9 cases)

- Stable success writes the manifest, runs 401, then makes exactly one launch with `--wait-hours 0`.
- A failed receipt gives exit 1, and a pending receipt past the deadline gives exit 3. Neither writes nor launches anything.
- A change between the two stable reads keeps it waiting.
- A 401 refusal leaves the manifest written, with no launch (F1).
- A byte-identical earlier manifest is reused, and a different one is refused.
- 350 exiting 1 makes 393 exit 1.
- The order checks above hold.

**Not tested:** a run on CSD, real imports, and the real `guard_csd02.py`.

## 5. Not claimed

No launch, import, render or master evidence. No picture, motion, listening, fine-sync, caption or independent AV acceptance. No publication authorization.
