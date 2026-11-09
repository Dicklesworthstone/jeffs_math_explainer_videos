# Controller 476: implementation receipt (CC1)

CC1 (BronzeJaguar), 2026-11-09 UTC, task 476. This moves only 08_loop_sums from the CSD352 route to the existing CSS352 + 371 route, as a successor to draft 460 and controller 464. It is source plus local fixtures only. Nothing ran on either host: no ssh, staging, transfer, import, job or queue change. Root inspects and launches; root coordinates the CSD 08 job's withdrawal or transfer with CC3.

## Files

| File | sha256 | |
|---|---|---|
| `cc1-master447-manifest-draft476.json` | `3e7b59d0cfb843cd3cd9741f72746139ee39359975d6bd99bb1c69e50cd3b457` | new, 14,850 B, sealed |
| `cc1-master-controller476.py` | `77cd7be786d87b847f31a31a281ef2d0d76f789276e9666af2667a45f691107e` | new, 498 lines, sealed |
| `cc1-master-controller476-fixtures/MANIFEST476.json` | `5966e1a38d5593fa1702eabda6805e8fd63b1b866d4ee135b09d0d35e49b9fe6` | the fixtures, 12 files |
| `master447-manifest-draft460.json` | `fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3` | base, unchanged |
| `cc2-master-controller464.py` | `89ff965940c3ed8146f9f849f2cd8736856fb547b0646a85ecb54f113047bfff` | base, unchanged |
| `08_loop_sums-css-permit474.json` | `92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a` | root's permit, unchanged |

These pinned or cited files are unchanged:

| File | sha256 |
|---|---|
| `combined-master447.py` | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` |
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` |
| `csd-gallery-import448.py` | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` |
| `csd-final-verify451.py` | `c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03` |
| `csd-final-controller452.py` | `beb4ea0650f577d3da70731bb5a01417c2d3550b7c84eb66aa77c3335d1fceae` |
| `cc3-css352-import-relay394.py` | `371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c` |

## Draft 476: one row changes

Draft 476 is draft 460 with only the 08 row replaced, plus a `draft476` metadata block (447 reads only `chapters`). The other 14 rows occur byte for byte in it, in the same order, gallery 05 and its 448 names included (fixture case D2).

| Key | Draft 460 (CSD352) | Draft 476 (CSS352 + 371 a1) |
|---|---|---|
| worker | `D/qualification01/production-repair17/csd-reviewed-final352.py` (23d55c28) | `CSS/qualification01/css-final-repair352.py` (1074051b) |
| job | `csd-reviewed-final-08-loop-sums352` | `css-reviewed-final-08-loop-sums352` |
| inputs | `Q/final-08_loop_sums-inputs352.json` | `CSS/qualification01/final-08_loop_sums-inputs352.json` |
| permit | `Q/08_loop_sums-csd-permit375.json` | `CSS/qualification01/repair352-permits/08_loop_sums-css-permit474.json` |
| import_receipt | — | `D/jobs/csd-css352-import371-08-loop-sums-a1/resource-receipt.json` |
| import_inputs | — | `Q/csd-css352-import371-08-loop-sums-a1-inputs.json` |

D is the CSD project root, CSS the CSS one and Q production-repair17 on CSD.

Every new name comes from frozen 371's own functions: `css_job`, `css_inputs`, `WORKER`, `PERMITS`, `job_name(08, 1)` and `inputs_path`. 371's `pin_problems` accepts them. The keys are 447's `ROW_KEYS | IMPORT_KEYS`, in row 01's order. `build476.py` wrote the draft once.

## Controller 476: the delta from 464

`delta476.py` (ast) finds 2 top-level statements added, 12 changed and 44 byte-identical. None were removed, and the order is kept. The full diff is `fixtures/cc1-master-controller476.diff`.

**Added:**
- `LOOP08 = '08_loop_sums'`.
- `loop_sums_arguments(args, required, gallery)`. Root gives `--loop-sums-manifest NAME --loop-sums-sha SHA` after the CSS 08 job is terminal and its 371 export succeeds. The name and sha follow the gallery pair's rules: a plain `.json` name directly in Q, and the full lowercase sha256. The function also refuses:
  - a pinned 371 export's name or sha;
  - the draft's or the launch manifest's name;
  - the gallery pair's name or sha.

  `check` takes the 08 pair optionally; `run` requires it.

**Changed:**
- **Constants:** `VERSION` 476, `SELF_NAME`, `DRAFT` and `DRAFT_SHA` (draft 476), and `LAUNCH_MANIFEST` `master447-launch-manifest476.json`. `CONTROLLERS` adds `cc2-master-controller464.py`, so `no_duplicates` refuses a running 464.
- **`pinned_exports`:** the CSS-imported set is 01/04/08/12/14 (371) and 05 (448). When 08's pair is given, its manifest gets the four pinned exports' checks: a symlink-free file in Q at its sha, version 371, its chapter, 371's `manifest_problems()`, and row 08's job, worker, worker sha, inputs and permit.
- **`manifest_bytes`:** writes a `launch476` block that records the six import pins and both argument pairs.
- **`check`, `run`, `main`:** parse the 08 pair and pass it on. `run` prints the pair in its start line.
- **`rows_and_receipts`:** the docstring only. Its CSS branch was already generic: for 08 it yields the 371 a1 import receipt, then `CSS/jobs/css-reviewed-final-08-loop-sums352/resource-receipt.json`.

**Unchanged:** `gallery_arguments`, `live_guard`, `state`, `one_pass`, `master_absent`, `verify_css_imports` (08's import result must name its pinned export, at its sha), `ancestors`, `no_duplicates`, `EXPORTS371`, `REGISTRY`, `IMPORT311`, the timing constants and the exit codes. Every 464 gate stays, in 464's order:
1. arguments;
2. rows;
3. export pins;
4. self, master, draft, 371, 448 and guard pins;
5. `master_absent`, `no_duplicates`;
6. the receipts, waited on and read twice for stability;
7. the pin recheck, `master_absent` and `no_duplicates`;
8. `verify_css_imports`;
9. the pin recheck, `master_absent` and `no_duplicates`;
10. the launch manifest (an existing one must be byte-equal);
11. `master_absent` and `no_duplicates`, then one 447 launch with `--wait-hours 0`.

## Do 447, 352, 371, 448, 451 and 452 support a generic CSS 08 row?

They do, with no change. One script needs a change: relay 394 (below).

- **447** (2b01ecaa) reads only `chapters`. 08 is in `CHANGED` (163–164), and `PREVIEW243_CHAPTERS` names 06 and 08 (247), so 08's preview job is 243 on either host (677). A non-CSD row must carry `ROW_KEYS | IMPORT_KEYS` (948), and its 352 module must report host `css` (962–965).
- **The CSS worker 352** (1074051b) lists 08 and maps it to preview 243 (135). Its `permit_sources` (265–282) needs:
  - permit host `css`;
  - an `08_loop_sums-` name;
  - preview job 243;
  - the timeline at its sha;
  - every bound evidence file at its hash.

  Permit 474 is permit 375 with host `css`.
- **371** (ba337aca) has 08 among its CHAPTERS, and its names are the draft's. No change.
- **448 and the gallery:** row 05 is byte-identical to draft 460's. The gallery pair and its checks are unchanged.
- **451** (c876caf6) selects the worker by `(kind, Path(row['host']))`. Its WORKERS (160–165) already hold `('combined', CSS)`. `source_job451` is `Path(row['host'])/'jobs'/job` (410–415), and only 05 is special-cased. 08's required sampled window (10200, 10380) at line 200 does not depend on the host: the CSS final must still contain it, as the CSD final had to.
- **452** (beb4ea06) pins only 451's sha (38) and has no chapter logic.

**Needed change: relay 394** (371f5d1f). Its `ORDER` (60) is 01, 04, 12, 14, 05. `--chapter` takes only `ORDER` (489), and line 214 refuses any other chapter, so 394 cannot carry the 08 import. Root needs a narrow 394 successor, or another root-approved transport, that runs 371 receive for 08 at attempt 1.

In a successor, put 08 after 14 and before 05. The serial gate (244–250) requires a completed 371 import for each chapter that comes earlier in `ORDER`. 05 now imports through 448, not 371, so 08 placed after 05 would never pass. Controller 476 itself does not care which transport runs, as long as 371's a1 receipt and import result appear at the draft's names.

## Root's preconditions (not done here)

1. **The CSS 08 render.**
   - Permit 474 must be in `CSS/qualification01/repair352-permits/`.
   - Its evidence, source-binding evidence and independent-review files, the preview 243 08 movie and the 08 timeline must be reachable on CSS at their hashes.
   - The CSS worker checks each of these and refuses otherwise (352: 272, 279–281, 611, 630–631).
   - The worker also refuses while its CSD alias shows any `csd-reviewed-final-08-loop-sums*` job (276–277).
   - 08 is the first CSS row on preview 243.
2. **The CSD side.** 371 refuses the 08 import while any `D/jobs/csd-reviewed-final-08-loop-sums*` folder exists (371: 504–505; CC0 check 477). A folder renamed aside inside `D/jobs` still matches the glob. The CSD 08 job's withdrawal or transfer is root's, with CC3.
3. **The Q permit copy.** 371 also needs root's Q copy `Q/08_loop_sums-css-permit474.json` at 92119d3a (`q_permit`). The local copy is at that sha.
4. **Attempt 1.** The draft names attempt 1. If a1 does not succeed, the import names change, and a new draft and controller are needed.
5. **Root's 08 export.** Once the CSS 08 job is terminal and its 371 export succeeds, root passes the export's Q file name and sha256 to `run` as `--loop-sums-manifest` and `--loop-sums-sha`, beside the gallery pair.
6. **Run only 476.** 464 does not list 476 in its duplicate set, so it cannot see a 476 run. 476 does refuse a running 464.

## Results (fixtures; macOS, Python 3.15.0)

`test476.py`: 16 of 16 cases pass. They cover only the route delta. The README in the fixtures folder says what each case shows.

| Case | Result |
|---|---|
| D0–D3, the draft | the pins hold. 14 rows are byte-identical to draft 460. Row 08 equals the row built from 371's names, has 447's keys and CSS worker, and its Q permit copy is at 92119d3a. |
| R1, the receipts | 476's receipts are 464's 24 with only 08's entry replaced, in place: the 371 a1 import receipt, then the CSS render receipt. That is 25 in all. No CSD 08 job is read. |
| A1, the arguments | 13 bad 08 pairs refused; neither given is None for check and refused for run; a good pair is accepted |
| P1–P5, the export pins | the four real exports pass the widened set. A synthetic 08 manifest passes 371's checks and binds row 08. A wrong sha is refused, a manifest binding permit 375 is refused, and a draft without a CSS 08 row is refused. |
| M1, the launch manifest | 15 draft 476 rows; `launch476` holds the draft 476 source, six import pins and both argument pairs |
| K1–K3, check | no root arguments: pins ok, 25 pending, exit 3. The 08 pair given: 08 is pinned, exit 3. An 08 file absent from Q: pins refused, exit 1. |
| U1, run | the gallery pair without the 08 pair is refused before any receipt is read |

**Stand-ins.** 447's Q is the CSD path, which does not exist on this machine. So for `pinned_exports()` the test passes a copy of the 447 namespace in which only Q differs. In K1–K3, `check()` runs with `pinned_exports()` redirected the same way. The test's Q holds hard links to the four pinned exports, not copies. Every `/data` receipt is absent here, so each is pending.

**Static checks.** `ast` parses both new Python files, and `delta476.py` gives the statement delta. No linter is installed on this machine.

## Limits

- These are fixtures, not an actual import, render or launch.
- The 08 export in P2 and K2, `fixture476-08-manifest.json`, is synthetic. Only its names and `permit_sha256` are real. It says nothing about the CSS 08 job, which has not run.
- No host was contacted. Nothing here withdraws, transfers or admits any job.
- This is not acceptance of any chapter, import or master. All final master, technical, picture, audio-review, fine-sync, independent AV and public playback checks remain open. There is no audio-review claim. Frederick remains the narration.
- CC0 is reviewing the 464 to 476 delta independently after this seal.
