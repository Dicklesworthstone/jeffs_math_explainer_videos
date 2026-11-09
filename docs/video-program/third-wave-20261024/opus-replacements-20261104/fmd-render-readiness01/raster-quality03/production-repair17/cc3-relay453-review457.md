# Review 457: gallery relay 453 against relay 394 and import 448

CC3, 2026-11-09 UTC. This is a source review backed by fake-transport fixtures. I ran nothing on either host: no ssh, no staging, no transfer, no remote read. Nothing here accepts a render, picture, motion, sound, sync or AV result.

## Files

| File | sha256 | Role |
|---|---|---|
| `css-gallery-relay453.py` | `c8f5539d2290416543e4a9bee50d7ef5bc2af82b61cf687ea73f30198881ec0d` | reviewed (root) |
| `csd-gallery-import448.py` | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` | reference (root) |
| `cc3-css352-import-relay394.py` | `371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c` | reference; frozen at this hash (CC3) |
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | frozen |
| `csd-master-controller449.py` | `3e8d5fa215399b590a27ce95591c6df7ff3d05f8667c419b5b5ac8247ecfb38b` | read: row names and `verify_css_imports` |
| `combined-master447.py` | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` | read: `imported()`, `SSD442` |
| `cc3-relay453-review457-fixtures.py` | `c07f25bb53a903e76c69da2736e93337a776dde305c7960bccdffd73286d3d0b` | my fixtures, 71/71 |
| `cc3-css352-import-relay394-tests.py` | `1b2315b09aa3734e835837e86b58c428076bab4a2d100ced5197d6e5eea51b55` | my 394 tests, 60/60 |

Root read 394 at `93f677ff` and derived 453 from it. Since then 394 has gained an explicit post-import validation (a–h), a read-only `verify` mode, a probe timeout, the `--prepared` parse fix, and a record for unexpected errors. Its timeout message now also says that the remote outcome is unknown. 394 is now frozen at `371f5d1f`.

## Verdict

There are no High findings: one Medium (M1), two Lows (L1, L2) and seven Infos.

The routing meets root's requirements:
- the sender's helper and manifest are read under `RECOVERY`;
- the receiving guard, lane, admission and attempt rules are 394's, unchanged;
- only gallery goes through 448;
- gallery is refused until the four earlier chapters have completed 371 imports.

The gap is the post-import check. 453 still has the implicit one from 394 at `93f677ff`, not 394's current a–h validation.

## What 453 changes

The fixture's `HUNKS` lists seven hunks. Each one's old text occurs exactly once in 394 and its new text exactly once in 453:
- **Constants:** `VERSION` 453; `HELPER` is 448 at `b6653757`; `ORDER` is gallery only; a new `EARLIER371` holds the four 371 chapters.
- **`CSD_PROBE`:** it takes the job prefix per chapter (`prefixes`, defaulting to `prefix`).
- **`gather`:**
  - `CSS_PROBE` reads helper, manifest, receipt and summary under `RECOVERY`;
  - `earlier` is `EARLIER371`;
  - `CSD_PROBE` is given the 371 prefix for the four earlier chapters.
- **`start`:** the sender runs the `RECOVERY` helper with the `RECOVERY` manifest.

453 also puts a new module docstring in front of 394's.

Candidate B is these seven hunks applied to 394 `371f5d1f` (it exists only in the fixture's temp folder). Its top-level statements differ from 453 in exactly the 11 that 394 changed after `93f677ff`:
- `remote`, `probe`, `POST_PROBE`, `local_manifest`, `gather`, `expected_inputs`;
- `validate`, `run`, `finish`, `verify`, `main`.

`start`, `wait_lane`, `check_prepared` and every other probe are byte-identical. So 453 adds nothing of its own outside the seven hunks.

## Requirements checked

| Requirement | Result | Evidence |
|---|---|---|
| The SSD sender's helper and manifest are under `RECOVERY` | holds | Fixtures check the `CSS_PROBE` arguments and the exact send command. 448 `send` has no helper-location assertion. It reads the manifest from the path it is given and checks its sha. It requires only that the `CSS` root is a real directory on the sending host. |
| Same receiving lane | holds | The `D/expanded-native-lane.lock` holder and waiter scan is unchanged. It runs before 448 check and again before the receive. A busy lane stops with 4 before any 448 call. |
| Same admission | holds | The receive is 448 `guard_command`: `guard_csd02.py --phase native` around 448 `receive` under `timeout 14400`. The send starts only after the job's receipt shows admission, a pid and this exact receive command. Any stop before that closes the receiver's stdin and starts no send. |
| Only gallery uses 448 | holds | `ORDER` and argparse refuse the four 371 chapters (fixtures). 448 `css_job` asserts gallery. |
| The four earlier chapters need completed 371 imports | holds | I ran the real `CSD_PROBE` on a synthetic jobs tree. It counts only `csd-css352-import371-<chapter>-a*` jobs with a terminal receipt and `import-result.json`. A 448-named job for an earlier chapter does not count. Gallery attempts come from 448 names only. Each missing chapter is refused by name. |
| Attempt numbers | holds | Unchanged from 394: never picked or incremented; an existing job or inputs at or above N refuses N. |
| Names that 449 expects | holds | rows350 names `csd-css352-import448-05-gallery-a1`'s receipt in D/jobs and its inputs in Q. 449 binds `recovery.job_name(key, 1)`. 447 `SSD442` equals 448 `RECOVERY`. |
| A transport failure stops the run | holds | Exit 2 at `CSS_PROBE`, at 448 check, at the admission poll and in the stream. Nothing is retried, and no send starts after an admission poll fails. |
| Same post-import checks as 394 | does not hold | M1 |

## Findings

### M1 (Medium). 453 keeps the old, implicit post-import check

`finish()` runs 448 check once and requires exit 0, `problems == []` and the expected `rows350`. It never reads the guard inputs after the import: 453 has no `POST_PROBE`, no `validate` and no `verify`.

448's check covers a, c, d, e and h on its own. Its done path reports `rows350` only for a terminal receipt with an `import-result.json` that names this manifest. Two conditions are left unchecked:
- f: the guard inputs file must still be the one the guard admitted;
- g: every frozen inputs row must re-hash, including the four that are not manifest rows (the 448 helper, the manifest, `guard_csd02.py` and Q's permit copy), and the set must be exactly what 448 prepare writes.

Failure scenario (fixtures, each of three cases): the import ends 0/0 and 448 check passes, but either the inputs file differs from the receipt's `inputs_sha256`, or one frozen non-manifest row no longer hashes, or the inputs hold fewer rows than prepare writes. 453 exits 0, prints rows350 and records "imported". B stops with 3 and names the failed condition.

This is not a safety hole. 447 `imported()` checks `sha(inputs) == inputs_sha256` and re-hashes every row at launch, so the master would refuse the import. But the relay would report as complete an import that the master later refuses.

There is a second cost, and it matters while the build host's command channels are timing out. When a run stops during its post-import check, 453 has no read-only way to ask again. Its `status` mode reports an imported attempt as a refusal.

Fix: rebase 453 on 394 `371f5d1f` by applying the same seven hunks, plus the L1 message changes. Fixture B shows the result:
- passes a–h with 448;
- its `verify` exits 0 with the 449 row names;
- it expects shared + system + 4 inputs rows. That count holds because 448's `prepare` and `check` are unchanged from 371.

### L1 (Low). Stop messages still name 371

- **Helper mismatch:** 453 `gather` says `CSS helper copy is not ba337aca` and `CSD helper copy is not ba337aca`, but the pin is `b6653757`. A root reading that stop would look for the wrong file.
- **Mode names:** "manifest refused by 371", "371 check refused" and "371 prepare refused" all describe 448 calls.
- **`load_helper`:** it names the module `import371`.
- **448:** `manifest_problems` refuses any version other than '448' with the text "not a version-371 manifest". A fixture confirms that a version-371 manifest is refused cleanly, with exactly that text.

Fix: build the helper messages from `HELPER` and `HELPER_SHA256[:8]` during the rebase. The 448 wording is root's decision, since 448 is root's file.

### L2 (Low). Two robustness gaps that 394 has since closed

Neither gap can start anything, because both happen before the receive.
- **`--prepared` crash:** with an inputs name whose attempt does not parse (for example a hand-made `…-a1b-inputs.json`), `gather` calls `max()` over a list holding None. It raises TypeError and the run ends with a traceback (fixture). 394 `371f5d1f` refuses by name.
- **Unrecorded errors:** any non-`Stop` error, such as a malformed probe answer, leaves `record.json` with status null (fixture). 394 now records "unexpected <type>: <text>". Both re-raise.

The same rebase fixes both.

### Infos

- **I1. Historical text:** the 394 docstring kept below 453's still says `CQ on CSS`, `relay394-<chapter>…` and `371`. 453's own records are `relay453-05-gallery-a<N>-run<K>/`. The new header says the text is historical.
- **I2. Ordering, not validation:** the serial gate counts any terminal 371 import that has `import-result.json`. Validation of those four chapters is 394 `verify` or the controller's re-check (see below).
- **I3. `RECOVERY` alias:** neither 453 nor 448 creates the `RECOVERY` alias on the build host. While the alias or `TOP_SSD` is absent or different, 448 check stops the run before prepare, so no inputs are written. Establishing them is root's separate step; this needs no relay change.
- **I4. Growth:** gallery's `RECOVERY` rows land in a new physical root (`TOP_SSD`), so none of them can be "already identical". New bytes and stream time will exceed chapter 01's. 448 check reports `new_bytes` and `space` before prepare, and the relay records that report. I did not measure them here.
- **I5. Attempt 1 only:** 449 binds gallery to import448 a1 and the four chapters to import371 a1. A run with `--attempt 2` would import under names that 449 refuses. The relay never picks a number, so an a2 needs a new controller version, the same rule as 393.
- **I6. Cosmetic label:** 449 gives gallery's import receipt the role name `import371`. The role only gates the render receipt.
- **I7. Hung channels:** if the server stops answering, ssh exits 255 after `ServerAliveInterval` 30 s × 10. If the server answers keepalives but the command never returns, the subprocess timeout ends the run: 300 s for probes, 3600 s for check and prepare. A prepare that timed out may still write its inputs later. `status` then lists that attempt under `inputs`, and the relay requires `--prepared`, which checks the exact set. 394 `371f5d1f`'s timeout message now says the outcome is unknown.

## Post-import validation that 393/449 must require (for CC0 and CC2)

Requirement: before an imported chapter's `rows350` is used, a–h must hold for that chapter, attempt and manifest, re-checked on the bytes as they are immediately before the 447 launch. A relay record or a relay exit 0 is not a substitute. This applies to rows 01, 04, 12 and 14 through 371 and row 05 through 448.

How 449's `verify_css_imports` (393's root401 check, carried over) and 447 `imported()` cover a–h today:

| | Condition | Covered by |
|---|---|---|
| a | the helper's check exits 0 | 449: `checker.check(...) == 0` |
| b | it is this job, a1 | the row names are pinned to `job_name(key, 1)` before polling |
| c | the receipt is a terminal success | 449: `success(receipt)` and `terminal_success` |
| d | `import-result.json` names this manifest | 449: `result['manifest_sha256']` equals the manifest bound once in the inputs |
| e | every manifest row is at its hash through its original path | 449: `problems == []` |
| f | the inputs file is the admitted one | 449 and 447: `sha(inputs) == receipt inputs_sha256` |
| g | every inputs row re-hashes, and the set is exact | the re-hash is in 447 `imported()` at launch; the exact set is not checked (an extra row is harmless, and 448/371 receive refuses a missing one) |
| h | `rows350` equals the row's names | 449: both `result['rows350']` and the check's `rows350` |

Two asks:
1. **Pin the manifests:** pin each chapter's manifest sha to root's named export: 392 `633c06d0`, 396 `04d1cd99`, 397 `73c62b8d`, and 14 and gallery when they are exported. Today 449 accepts whichever manifest the import result names, as long as it is consistent.
2. **Gallery checker:** keep gallery's checker as 448 at `b6653757`. Its done path re-hashes the `RECOVERY` rows on the build host only through the exact `RECOVERY` alias.

## How to run

```
python3 -I -B cc3-relay453-review457-fixtures.py <Q> <temp-parent>     # 71 expectations, A and B
python3 -I -B cc3-css352-import-relay394-tests.py <Q> <temp-parent>    # 60 expectations on 394
```

Both use fake transports. Each writes under one new temp folder: at most 231 KB of records, a synthetic six-row version-448 gallery manifest and candidate B. They read root's manifests 392 and 396 in Q, and nothing else of root's beyond the source files listed above.

Not run: anything on either host; 448's real check, prepare, receive and send (they assert the build host's D and Q); the guard; a stream.
