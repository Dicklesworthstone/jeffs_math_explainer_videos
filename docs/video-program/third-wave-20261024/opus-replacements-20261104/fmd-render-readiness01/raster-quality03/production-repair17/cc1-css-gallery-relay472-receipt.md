# Relay 472: implementation receipt (CC1)

CC1 (BronzeJaguar), 2026-11-09 UTC. Relay 472 is the narrow successor to CC3's relay 463 that root asked for after review 466. It is source plus local fixtures only. Nothing ran on either host: no ssh, no staging, no transfer, no import, no job. Root inspects and launches.

## Files

| File | sha256 | |
|---|---|---|
| `cc1-css-gallery-relay472.py` | `32f36d16e3cb3402e4c632b44a77c9852546fd8e96a90ff5127e8a1a66ecd5f1` | new, 535 lines, sealed |
| `cc3-css-gallery-relay463.py` | `d1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892` | base, unchanged |
| `csd-gallery-import448.py` | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` | pinned helper, unchanged |
| `cc1-css-gallery-relay472-fixtures02/MANIFEST472.json` | `f0d1d6093a17f88b2b2c9543d3e4aa2c2e83e4fb26efcbd33c6d8db4c9936db4` | the fixtures, 14 files |
| `cc1-relay463-review466-fixtures/MANIFEST466.json` | `088bae1f10962bbc3b3918a3999c4f8077f51a0333ba01b4a05504ca23008e98` | reused, unchanged |

`cc1-css-gallery-relay472-fixtures/` (MANIFEST `fa9dfb46…`) is superseded by `fixtures02`. It is kept unchanged because nothing is deleted. Its `compare472.py` held a scratch-path regex literal; the folders otherwise hold the same tests and outputs.

## Delta, 463 to 472

`delta472.py` finds 6 top-level statements changed and 3 added. None were removed, 40 are byte-identical, and the order is kept. The full diff is `fixtures02/cc1-css-gallery-relay472.diff`.

**Root's three changes:**
1. **g: exact path:hash set.** g still requires `rows == distinct == want` and `changed_count == 0`. It now also requires `is_frozen_set(listed rows)`: every row except the guard's must equal `frozen_rows()`, and the guard row must be present. These are `check_prepared`'s model and code:
   - every shared and system row at its manifest hash;
   - `Q/448` at `HELPER_SHA256`;
   - the manifest at `--manifest-sha`;
   - the Q permit copy at `permit_sha256`.

   `POST_PROBE` now also prints `listed_rows`.
2. **c: exact receive command and helper hash.** c still requires the terminal receipt, from 448 check and the direct read. It now also requires:
   - the receipt's `command == pinned_receive()`, which is `receive_command(Q/448, chapter, attempt, Q/manifest)`. That is the same list that `start()` requires at admission;
   - 448 check's `helper_sha256 == HELPER_SHA256`.

   `POST_PROBE` now also prints `receipt_command`.
3. **Stream-loss recovery text.** `finish()` now says "run verify --attempt N when both hosts answer". Before, it said "run status".

**Supporting changes, with no change in behavior:**
- `frozen_rows` and `is_frozen_set` are `check_prepared`'s former inline map and guard test. `check_prepared` now calls them, and its refusal text and return value are unchanged.
- `pinned_receive` is `start()`'s former inline admission command. `start()` now calls it.
- `VERSION = '472'`. Record folders become `relay472-05-gallery-a<N>-run<K>/`, and the validator label becomes `relay472 validate a-h`.
- The module docstring: its header, and steps 8c and 8g.

**Unchanged:** 448; the CSS, CSD, JOB and INPUTS probes; `gather`; the lane waits; admission; the attempt and `--prepared` rules; the serial gate; RECOVERY routing; the run, status and verify control flow; exit codes.

## Results (fixtures; macOS, Python 3.15.0)

**Reuse (`run472.py`, then `compare472b.py`).** Review 466's trials a, b and c were rerun against 472. Their test files were checked against MANIFEST466, and only trial a's record glob was renamed. Compared with 463 there are **14 differences, all intended**:

| Case | 463 | 472 |
|---|---|---|
| Fabricated completion, shared row substituted at the same count | 8/8 | fails g |
| Fabricated completion, Q permit copy substituted at the same count | 8/8 | fails g |
| S02 and S03: the stop and the record message | "run status" | "run verify --attempt 1" |
| F5: the receipt command names another helper path | verify 0 | verify 3, fails c |
| F5: the receipt command names attempt 7 | verify 0 | verify 3, fails c |

Everything else is identical:
- 448 prepare and all of 448 receive's binding refusals (F1, F1r);
- `check_prepared`: the exact set is accepted, a substituted set refused;
- S01, the happy path: 8/8 under 472 in the fixture world;
- S02–S13: exit codes, transport order, the send gate, the follow-up calls, and the CSD jobs and Q left unchanged;
- the F3 single-condition breaks, the F4 routing and serial-gate refusals, and 448 receive in F5r.

**Targeted (`test472x.py`).** 463 and 472 were run on the same state.

| Case | 463 | 472 |
|---|---|---|
| X1, exact rows (control) | 8/8 | 8/8 |
| X1, shared row substituted, same count | 8/8 | fails g |
| X1, system row substituted | 8/8 | fails g |
| X1, Q permit copy substituted | 8/8 | fails g |
| X1, helper row substituted | 8/8 | fails g |
| X1, guard row substituted | 8/8 | fails g |
| X1, manifest row substituted | 8/8 | fails g |
| X2, 448 check unchanged (control) | verify 0 | verify 0 |
| X2, 448 check reports another `helper_sha256` | verify 0 | verify 3, fails c |

X2's helper hash is a fixture forgery of check's output. It stands for check run from a Q copy that is not the pin.

**Static checks.** `ast` parses the file. No linter is installed on this machine, so nothing beyond the parse and `delta472.py` was run.

## Limits

- These are fixtures, not an actual import.
- The CSD receive end is scripted, except where 448's real `prepare`, `check` and `receive` run through the review 466 twin, whose only rewrites are the four root literals.
- No host was contacted. CSS442 is still live.
- This is not acceptance of any import or master. All final AV acceptance is open, and there is no audio-review claim. Frederick remains the narration.
- CC0 is reviewing the 463-to-472 delta at root's request.
