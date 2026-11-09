# Review 454: master 447, controller 449 and draft 449 (CC0)

cc0, 2026-11-09, 04:00 UTC. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

This is an independent review of root's `combined-master447.py` and `csd-master-controller449.py`, against frozen 350, controller 393 and `master447-manifest-draft449.json`. It also checks whether the SSD442 gallery source and the 448 import connect to them.

Nothing ran on any host: no ssh command and no render, import or master. Every check below ran locally, on read-only files and small synthetic fixtures. None of it is CSD evidence, and none of it is picture, motion, sound, sync or AV acceptance. No audio heard or checked.

## Verdict

- No High and no Medium finding.
- **The SSD442 gallery source and the 448 import connect** to 447 and 449 as written (section 2). That holds once root's preconditions in section 4 are met.
- **The other 14 rows and their gates are not weakened:**
  - all 14 rows are byte-equal to draft 384's;
  - under 447 each resolves to the same source as under 350;
  - 447 changes one gate function, `chapter_gate`, and only in its combined branch. That change is equivalent for both 352 workers (section 3).
- There are four Low findings. L1 is an ordering problem, the same one CC2 reported for 393 as F1 in review 395. L2–L4 are wording and provenance.

## 1. Files

| File | sha256 | Lines |
|---|---|---|
| `combined-master447.py` | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` | 1394 |
| `csd-master-controller449.py` | `3e8d5fa215399b590a27ce95591c6df7ff3d05f8667c419b5b5ac8247ecfb38b` | 370 |
| `master447-manifest-draft449.json` | `0108537698028e0b7389533f85c552c652e9b7aef72f86b6028ae209870be5aa` | |
| `cc0-combined-master350.py` (base) | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | unchanged |
| `csd-master-controller393.py` (base) | `8fb9a8c1cf524b55da68cdf52a27b43159f64caa1377fa52332bde9720ecd05b` | unchanged |
| `master350-manifest-draft384.json` (base) | `ddbe3f2b4225fc580e14e8f3890c63caa6e51061e52dcb2664e1680773436968` | unchanged |
| `css-gallery-ssd442.py` (read for the connection) | `ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429` | |
| `csd-gallery-import448.py` (read for the connection) | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` | |
| `guard_css_ssd441.py`, `launch_css_ssd442.py` (read) | `40febbd0…`, `6daf9713…` | |

448 itself, 451/452 and relay 453 are other reviewers' work. I read 448 and 451 only where 447 or 449 depends on them.

## 2. The 05 connection, link by link

1. **Draft 449 row 05** names:
   - worker `SSD442/css-gallery-ssd442.py` at `ce9baf6a`;
   - job `css-reviewed-final-05-gallery352-ssd442`;
   - inputs `SSD442/gallery-inputs442.json`;
   - the unchanged CSS permit `CQ/repair352-permits/05_gallery-css-permit375.json`;
   - import `D/jobs/csd-css352-import448-05-gallery-a1/resource-receipt.json`, with inputs `Q/csd-css352-import448-05-gallery-a1-inputs.json`.

   Here `SSD442` is `/data/ssd/work/jmev-gallery-recovery440`.
2. **447 accepts the row.**
   - `WORKERS352` gains exactly `WORKER442` at `ce9baf6a` (:910).
   - `combined_source447()` (:912–919) allows that worker only for 05 with exactly that job, and returns `SSD442/jobs/<job>`.
   - The row must carry import keys, because its host is CSS.
3. **The 442 module passes every check 447 makes on a CSS worker** (check449). 442 is CSS 352's worker with five functions changed: `job_name` (+`-ssd442`), `check`, `render`, `admitted` and `launch` (disabled). It keeps:
   - VERSION `352`, the ten CHAPTERS and HOST `css`;
   - `job_name('05_gallery')` equal to the row's job;
   - the MOVIES and PREVIEWS sets;
   - PERMITS = `CQ/repair352-permits`, with `permit_name_ok` true;
   - BIN = binary 268 and CSS = the CSS root.
4. **The render receipt.**
   - `launch_css_ssd442.py` runs `guard_css_ssd441.py`, which is the original `guard_css74.py` with WORK, REPO and TARGET moved to SSD442. The receipt schema is guard74's, and the receipt lands at `SSD442/jobs/<job>/resource-receipt.json`, which is 447's source.
   - The child command is `<442 worker> render --chapter 05_gallery --job <job> --inputs SSD442/gallery-inputs442.json --permit CQ/.../05_gallery-css-permit375.json`, exactly the flags 447 compares (:979–982).
5. **The summary.** 442's `render()` writes:
   - `host='css'`, which 447 compares with `module.HOST`;
   - `worker352_sha256=sha(Path(__file__).resolve())`, its own `ce9baf6a`, which 447 asserts for a CSS-host row;
   - `narration=str(NARRATION)`, the same delivery309 narration as the other CSS rows.
6. **On CSD.** 448 requires an existing exact symlink `SSD442 → D/work/imported-css-ssd442`, a real directory on the jobs filesystem, and never creates it (`alias_problems`, 448 :492–501).
   - Through that alias, 447's same-device check on every staged output passes (:1102–1104), and so do 447's and 451's hard links.
   - 448's `q_manifest` and `chapter_of` are the same code as 371's.
   - Its `rows350` uses the draft 449 names.
7. **449 waits** on the 448 a1 receipt first, then the SSD442 render receipt, and checks the 05 import with 448's own `check()` (test449). Its launch command is 447's own `launch --manifest Q/master447-launch-manifest449.json --wait-hours 0`.
8. **Downstream,** 447 records the 05 row as `host=CSS` with the SSD442 job. 451 rebuilds that row's source from `GALLERY442_JOB` under SSD442, not from `host/'jobs'/job` (451 :405–415), so it connects. 451 is reviewed elsewhere.

## 3. The other 14 rows and the gates

- **AST.** 112 of 116 top-level items are identical to 350. `VERSION`, `JOB`, `INPUTS` and `chapter_gate` changed; `SSD442`, `WORKER442` and `combined_source447` were added; nothing was removed.
- **chapter_gate.** Two lines changed:
  - the job-name assertion moved into `combined_source447`, with the same strings for both 352 workers;
  - the combined row's source folder now comes from that function, the same `host/'jobs'/job` for both 352 workers.
- **The rest.** Every other gate is unchanged: `success`, `imported`, `combined_scope`, `schedule`, `capture_check09`, `preflight`, `master`, the retention, permit and independence gates, and film().
- **WORKERS352** is read only in `chapter_gate` (:944–945). Loading 447 leaves 350's dict at two entries.
- **Rows.** All 14 rows are byte-equal to draft 384's. Each passes 447's static gate and resolves to the same source folder as under 350 (check449, 14 cases).
- **Refusals** in 447's static gate (7 cases):
  - the 442 worker on chapter 12;
  - the 442 worker with the failed job name;
  - the CSS 352 worker with the ssd442 job;
  - 05 without import keys;
  - another worker sha;
  - a permit outside 442's PERMITS;
  - an import receipt outside `D/jobs`.
- **Not a weakening:** 447 would still accept the old 384 shape of 05 (the CSS 352 worker with the failed job) if a manifest named it. Its receipt fails `success()`, so that row would be refused at run time. In any case 449 pins draft 449.

## 4. Preconditions for root (not defects)

- **P1.** The CSD alias `/data/ssd/work/jmev-gallery-recovery440 → D/work/imported-css-ssd442` must exist before 448 runs. 448 and 447 both depend on it. Without it, 448 refuses, and so does 447's device check.
- **P2.** 447 re-hashes every row of `gallery-inputs442.json` at its original path on CSD (:983–988). 448's closure must carry all of them, which includes:
  - the failed 352 job's receipt and `final-05_gallery-inputs352.json` with all its rows;
  - the ssd440 receipt;
  - both allocation baselines;
  - `guard_css_ssd441.py` and `launch_css_ssd442.py`.

  The failed 352 receipt will then sit on CSD at `CSS/jobs/css-reviewed-final-05-gallery352/` through the CSS alias. That is harmless to 447 and 449.
- **P3.** CSD Q must hold 447, 449, draft 449, 448 and 371 at their pins.

## 5. Findings

### L1 (Low). 449 writes the launch manifest before the post-import check.

- `run()` writes `Q/master447-launch-manifest449.json` at :330 and only then runs `verify_css_imports()` at :333.
- If that check refuses, Q keeps a launch manifest although nothing launched, and a later run accepts the same bytes (:328).
- That is the reverse of root 390's "post-import check before master manifest adoption".
- CC2 reported the same order in 393 as F1 in review 395; this is my independent confirmation for 449.
- The file claims nothing, so this is Low. The fix is to call `verify_css_imports()` before the manifest block, in a successor.

### L2 (Low). The 449 launch manifest names the wrong writer and reader.

- 449's `manifest_bytes()` is unchanged from 393, so the launch manifest carries a `launch393` block with these texts:
  - `gate`: "csd-master-controller393.py writes this file only after its prerequisite pass; 350 launch then runs its own gates";
  - `rows`: "Frozen 350 reads only 'chapters'" (test449 shows both).
- In fact 449 writes it and 447 reads it. The file becomes a bound input of the master job, so the wrong provenance would be permanent. Only the `source` entry (draft 449 at `01085376`) is right.
- Fix in a successor: block key `launch449`, naming 449 and 447.

### L3 (Low). Draft 449 still carries draft 384's block.

- Its only top-level key besides `chapters` is `draft384`, unchanged. That block says:
  - "DRAFT 384";
  - "cc0-combined-master350.py (60575944) reads only 'chapters'";
  - that row 05 uses import371 names and CSS 352 inputs.
- Each of these is false for 449's 05 row.
- 447 and 449 read only `chapters`, so behaviour is unaffected. But the pinned draft misdescribes itself.
- Fix: a new draft carrying its own block, with 449's `DRAFT_SHA` re-pinned in the same successor as L1 and L2.

### L4 (Low, wording). The docstrings describe their predecessors.

- **447** still opens "Combined-repair complex master on CSD, version 350", and its rules still say "only the two 352 workers" (:27). The refusal at :1216 says "master350 job or inputs already exist".
- **449's** docstring is 393's word for word: it says it launches "frozen cc0-combined-master350.py" with draft 384's rows. Its `rows_and_receipts` docstring and refusal texts also name 384 and 350 (:94, :361).

### Notes (no change needed)

- **N1.** 449 refuses only on 447's master job and inputs. A `csd-combined-master350` folder does not stop it (test449). No 350 master can succeed while 05 is the failed CSS job, so I see no path to two masters.
- **N2.** 449 labels the 448 receipt `import371` (:126). That only names the role; the ordering logic is right.
- **N3.** 449 uses 371's `q_manifest` for the 448 row too (:217). The function is identical in 448 and Q is the same.
- **N4.** In 447 at :946, `pattern` is now set but never used.
- **N5.** As in 393 (CC2 F2), an exception other than an assertion ends 449 with a traceback, not a `refused` line. Nothing launches either way.

## 6. Fixtures (scratch files outside the repo; local, synthetic, writing nothing in Q)

| Fixture | sha256 | Result |
|---|---|---|
| `check449.py` | `3d81e429b41e2e03c4b9ef1642464837020c91668025c822ea708ce683e331dd` | 43/43 |
| `test449.py` | `3b4e9fc6b0e6be3c339ba1c8dba9016b2f6d373000a6fb11f22eccbe319eed62` | 21/21 |

**check449** loads 350, 447 and the 442, 352, 344 and retained workers from their local copies, at their pins. It runs 447's pre-read row checks and module checks on every draft 449 row, compares sources under 350 and 447 for the 14 unchanged rows, and runs the 7 refusals above.

**test449** stubs launch, sleep, the `/proc` scan and the import checkers. Its tiny receipts are in a new scratch folder (`ctl449a`, kept). It covers:
- the pins;
- the 24 receipts, and the 05 import-then-render order;
- 6 refused row edits (05 at a2, 05 with 371 names, the failed inputs, the failed job, 442 on 12, 01 with a 448 name);
- run scenarios: launch, the 447 command, the manifest rows and block, a 05 render waiting for its import, a missing SSD receipt, a failed SSD render, the master inputs existing, 447 exiting 1, a 350 master folder;
- the checker wiring for 01 (371) and 05 (448).

**Not tested:** any CSD run, 448's real `check()` on a real import, and the Linux `/proc` scans. CC2 reports a fake-`/proc` pass in review 395.
