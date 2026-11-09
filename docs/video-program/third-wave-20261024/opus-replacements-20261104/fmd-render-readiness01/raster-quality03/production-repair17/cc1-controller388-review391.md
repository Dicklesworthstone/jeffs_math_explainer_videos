# 391: independent review of controller 388 (CC1)

cc1 (BronzeJaguar), 2026-11-09, for root. Read-only review and local stand-in tests. Nothing was launched, staged or edited, and no CSD job was touched. D, Q, CSS and TOP are as in 371/374.

**Verdict: no High, no Medium.** Against the actual 377 and the actual guard chain, 388 does what its notes say:

- It reads 377's real printed shapes.
- It writes the ledger once, after the overview job succeeds and before the native launch.
- Every failure stops the chain, and nothing is retried.
- It never relaunches an existing 377 job.

There are five Lows. The one that matters most is L1: notes §3 overstates what root can continue by hand after a failed 377 job. The rest are launch-time checks root can add to its own command line, or wording. None needs a source change before use.

| File | sha256 | Read |
|---|---|---|
| `csd-final-controller388.py` | `5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e` | all 188 lines |
| `cc2-final-controller388-notes.md` | `c5b7718aaf22fd6c41f240d8674af3d968590a87c6961665516e5ba344cd68b1` | all |
| `csd-final-verify377.py` | `827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe` | docstring and constants :1–147, :339–400, :835–1219 (every function 388 calls or whose output it parses) |
| `cc2-final-verify377-notes.md` §4, `cc2-final-verify363-notes.md` §5 | `d176024b…` / (363 notes) | the run order 388 follows |
| guard chain: `guard_csd02.py`, `guard_owner56g_target20g.py`, `run_bounded_phase_oliver_owner48g_target14g_20261102.py` | `f1cae1db…`, `1c19a9b8…`, `819746fb…` | all three, in full (repo copies, byte-equal to CSD's, see §2) |
| `verify_stage09_final.py` (377's pinned shared module) | `f266a231…` | top level only (AST) |
| `root-review378-374-disposition390.json`, `verifier377-stage387.json`, `retention-stage389.json` | | all |

## 1. Root's conditions

- **No automatic retries.** 388 has one loop. It polls the master receipt and launches nothing. Every other failure is a terminal `assert`, and scenarios S11–S19 below stop at the first failure. 388's only subprocess is `Popen([/usr/bin/python3, -I, -B, 377, MODE])`, and the AST shows exactly four such calls: `check`, `launch-verify` and `launch-pictures` twice.
- **No CSD same-chapter queues.** 388 launches no chapter final. It does queue master-level jobs on CSD's one native lane, because every 377 launch goes through the guard's blocking lane lock. See L2.
- **Duplicates.**
  - Before waiting, 388 refuses if any of the seven 377/388 paths exists (S10, all seven).
  - A second 388 run on a finished tree refuses and changes nothing (S02).
  - Two 388s started together (S20): one completes; the other stops at 377's `preserve any earlier attempt: final-verify377-inputs.json`, before any guard runs. The tree ends with three jobs and one ledger. 388 takes no lock of its own: 377's exclusive-create of the inputs file is what serializes them.

## 2. Actual shapes 388 parses (evidence)

- **CSD state.** I ran one read-only listing/hash script on CSD before root reported that CSD command channels time out. I have made no CSD contact since.
  - Q holds 377 at `827be351` and its notes at `d176024b`.
  - **388 is not staged on CSD.** There is no `csd-final-controller388.py`, no log and no ledger.
  - There is no `csd-combined-master350` job, no `combined-master350-inputs.json`, and no 377 job or inputs file.
  - The chain on CSD is at `f1cae1db` (guard_csd02.py), `1c19a9b8`, `819746fb` and `a2fe7985` (owned-process-stop02.py), so my repo copies are the CSD bytes.
  - `csd-reviewed-final-09-boundary344` held the native lane with an unfinished receipt.
  - One job folder, `fmd-raster-rebind12`, has no receipt at all.
- **What the guard prints.** It prints exactly one single-line JSON to its own stdout, which goes to 377's stdout and so into 388's pipe:
  - after a run: `{"exit", "resource_stop", "receipt"}`;
  - when admission is refused: `{"admitted": false, "launched": false, "receipt"}`, with exit 75.

  Neither line has a `job` or `check` key, so `launched()`'s single-row unpack always gets 377's own launch line. In S01, 388 parsed eight JSON lines in order: check; launch and guard (verify); launch and guard (overview); launch and guard (native); 388's last line.
- **What the guard writes to `command.log`.** It opens `command.log` with `x` and gives the child stdout+stderr. It writes nothing else there, so the overview job's last line is 377's line.
  - Actual CSD: in the 14 most recent job folders, the `command.log` tails are the children's own last lines (for example `Imported 858 files; …`, `AssertionError: ('tests', 101)`), and no guard line appears in any of them.
  - In S01 the overview `command.log` is exactly one line: 377's.
- **How the guard writes the receipt.** It rewrites `resource-receipt.json` in place (`write_text`, truncate and write) about every 10 s, and writes `finished` exactly once, as the last key. Actual CSD receipts have the key order `started … pid, exit, final_inputs_sha256, finished`.
  - A truncated read is a `JSONDecodeError`, a `ValueError`, which 388 treats as pending. S04 (a partial receipt completed 1.5 s later) proceeded normally.
- **The last-line regex** matches 377's own format string, rendered from 377's AST, including 0 sheets. It rejects uppercase hex, a 63-digit hash, a trailing space and the native line.
- **Keys 388 reads from 377's printed JSON.**
  - From `check`: `check`, `frames`, `picture_jobs[].job` and `plan_sha256`.
  - From `launch` (`job` plus `**note`): `part`, `plan_sha256` and `overview_sampling_sha256`.

  All are in 377's `json.dumps(dict(…))` calls. All 15 names 388 reads from `v` exist at 377's top level. At load time, the top levels of 377, 388 and the shared module call only `Path`, `dict`, `range`, `re.compile` and `Path(__file__).resolve`.

## 3. Tests

**Harness.** These tests ran locally, on throwaway scratch trees of a few KB each, under Python 3.15.

- 388's frozen source runs unchanged, hash-checked. Rebound for the tree: `D`, `Q`, `VERIFIER`, `VERIFIER_SHA`, `LEDGER` and `POLL`. `/usr/bin/python3` in argv maps to 3.15, because the Mac's 3.9 lacks `hashlib.file_digest`.
- Root's runner `819746fb` runs unchanged, with the in-memory overrides that guard_csd02 and owner56g make.
- Test-only guard settings:
  - admission 0 and stop 0 (the Mac had about 3 GiB free when the harness was written), each raised for its fault scenario;
  - `sbh` stubbed;
  - `/usr/bin/timeout` dropped;
  - owned-process-stop02 not installed.
- The 377 stand-in is assembled from 377's own source, verbatim by AST segment: its constants, `sha`, `read`, `success`, `stable`, `growth_upper`, `part_paths`, `plan_digest`, `part_upper`, `part_done`, `verified`, `launch`, `main`, and the closing prints of `verify()` and `pictures()`.
- Stubbed in the stand-in: `film` (which still runs 377's `stable()` on the master), `plan_for` (19 samples, 3 natives), `verify`, `pictures`, `runtime` and `pillow_runtime`. Changed constants: `D`, and byte budgets of 1 MiB.

| Scenario | 388 stopped at | Left behind |
|---|---|---|
| S01 happy | exit 0; ran check, launch-verify, launch-pictures ×2 | 3 jobs, 3 inputs, ledger; ledger = sampling.json = native inputs row = `union.overview_sampling_sha256` |
| S02 rerun after S01 | `already exists; nothing launched` | tree unchanged |
| S03 / S09b master absent / running, short wait | `master350 not finished; no 377 job launched` | unchanged |
| S04 partial receipt, then success | exit 0 | as S01 |
| S05 / S06 / S07 master exit 1 / refused admission / resource stop | 377 `success()` on the master receipt | unchanged; no 377 path |
| S08 master receipt rewritten after `finished` | `receipt changed after it finished` | no 377 path |
| S09 377 changed during the wait | `changed: …csd-final-verify377.py` | no 377 path |
| S10 ×7 one 377/388 path pre-existing | `already exists` before waiting | unchanged |
| S11 a master row changed after it finished | `377 check exited 1` | unchanged |
| S12 verify job exit 1 | `377 launch-verify exited 1` | verify job (exit 1) and inputs; nothing else |
| S13 verify refused by the guard | same | verify folder with an `admitted: false` receipt |
| S14, S14b 377 free-space refusal, then rerun | same, both times | nothing: no inputs, no job |
| S15 overview resource stop (guard exit 75, receipt `stopped`, exit −15) | `377 launch-pictures exited 1` | no ledger, no native |
| S16 a line after 377's overview line | `did not print its sampling.json sha256 last` | no ledger, no native |
| S17 sampling.json changed after the overview printed | `not the one its job printed` | no ledger, no native |
| S18 sampling.json changed right after the ledger was written | `overview sampling.json or the ledger changed; no native job` | ledger, no native |
| S19 native job exit 1 | `377 launch-pictures exited 1` | ledger, native (exit 1) |
| S20 two 388s at once | A exit 0; B at 377's inputs refusal | one set of jobs, one ledger |
| S21 `--wait-hours` 49, −1, nan | assert, before anything | unchanged |

**Ledger ordering (S01, guard and 388 UTC stamps).** Each step began after the previous one ended:

- verify finished at 14.478 s;
- overview started at 15.327 s and finished at 15.410 s;
- the ledger was written at 15.492 s;
- native started at 15.698 s.

The ledger has 11 keys, all of them strings: no booleans and no flags.

## 4. Findings

**L1. Notes §3 overstates continuing by hand.** §3 says that once a 377 job exists, "root continues by hand with 377's remaining modes". That holds only when every existing 377 job succeeded: S16–S18, or 388 dying. After a failed or refused 377 job, 377 itself cannot continue:

- After a failed or refused verify job (S12, S13), `verified()` fails for good.
- After a failed overview (S15), the next `launch-pictures` picks the native part, because the overview folder exists, and then `part_done('overview')` fails.
- After a failed native (S19), 377 reports "both picture jobs of 377 already exist".

Each of these needs a new verifier version with new job names. That fits "no automatic retry", but root should read §3 that way.

**L2. The lane-lock wait is unbounded, and a change during it spends a job name.** `--wait-hours` bounds only the master wait. Each 377 launch first freezes its inputs file and then calls the guard, which blocks on the CSD lane `flock` with no timeout. The guard makes the job folder before `check_inputs`, and saves its first receipt only after. So a bound file that changes while the guard waits leaves a job folder with no receipt, 377 exits 1, 388 stops, and that job name is spent.

CSD already has a folder with no receipt (`fmd-raster-rebind12`), and 09 final344 held the lane at my read. Before starting 388, root should confirm that nothing else is queued on CSD's lane behind the master.

**L3. The ledger is outside the job folder, but not off the host.** It lives in CSD Q, under the same account as the job folders. That meets 339 L1's letter (not in the job's own folder). It becomes "root's own ledger" (363 §5) only when root copies its sha256 off CSD.

Notes §2.4 say to copy it at the end. Copying the `388 ledger … sha256 …` line as soon as it appears in the log is better: it is printed before the native launch.

**L4. Nothing pins the guard to a known hash.** 388 pins `guard_csd02.py` at whatever bytes it has when 388 starts. 377 binds it in its inputs at freeze time. Neither compares it with `f1cae1db`.

CSD has `f1cae1db` now. Adding `f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6  <D>/guard_csd02.py` to root's `sha256sum -c` line in notes §2 closes this without a source change.

**L5. The notes' §4 caveat is now answered.** CC2 wrote that they had not read `guard_csd02.py`. I have read all three guard files. The facts 388 relies on hold (§2): `command.log` holds the child's output only, the receipt is rewritten in place, and the guard's stdout line has no `job` key.

**Infos**

- **I1.** The "same one poll later" check on the master receipt cannot fire with this guard, which writes nothing after `finished`. It is harmless, and S08 shows it does fire when a receipt is rewritten.
- **I2.** If the master's guard dies before its first save (an `sbh` timeout or a changed input) or is killed, `finished` is never written. 388 then waits the full `--wait-hours`, up to 48 h, and stops, as in S03/S09b. That is fail-closed. Root may prefer a shorter wait.
- **I3.** `step()` decodes the merged child output strictly (`text=True`). A byte that is invalid in the locale's encoding would stop 388 while the guarded job runs on. The job's record stays intact, because the guard prints only after its final save; 388 just stops before the ledger.
  - 377's JSON is ASCII. Lines :276, :607 and :810 contain U+2212, which can appear only in a traceback, that is on a failure path.
  - Normal UTF-8 locales are unaffected.
- **I4.** 388 runs 377's `check` and `launch-*` outside the guard at normal priority, as the manual order would. Each run re-hashes every bound file, including the 4K master, in `check`, `launch-verify`, both `launch-pictures` runs and step 7. A `nice`/`ionice` prefix on the `setsid` line would also be inherited by the guarded jobs. That is root's call.

## 5. Fixtures

They are in `Q/cc1-controller388-review391-fixtures/`. `MANIFEST391.json` is `07301fcf286061cbd31235775454c93e09dab3e746e55e5ea0368614f82ac14a`.

| File | Role |
|---|---|
| `fake377_gen391c.py` | builds the 377 stand-in from 377's own source |
| `fake377-391c.py` | the stand-in; it is not 377 |
| `fakeguard391.py` | runs runner `819746fb` with the overrides listed in §3 |
| `run388_391.py` | runs 388's frozen source |
| `test391.py` | the scenarios |
| `static391.py` | the AST checks |
| `csd_state391.py` | the read-only CSD script |
| `results391-summary.json` | all outputs, with local paths replaced by T |

Rerun: `python3 -I -B test391.py <388> <819746fb runner>` from a copy of the folder. It needs Python 3.11 or later.

## 6. Limits

- These are source reading and stand-in tests only.
- None of 377's verification, decode, frame identity, captions, pictures or plan logic ran: those bodies are stubs.
- `guard_csd02.py` and `guard_owner56g` were read but not executed: their overrides were replicated in memory, and owned-process-stop02 was not installed.
- Nothing ran on CSD except the one read-only script in §2.
- No launch, staging, permit or source edit.
- Narration is ElevenLabs Frederick, unchanged.
- No listening, picture, motion, fine-sync, caption-playback, 4K or independent AV acceptance is claimed, and nothing here authorizes publication. All finalAV acceptance is open.
