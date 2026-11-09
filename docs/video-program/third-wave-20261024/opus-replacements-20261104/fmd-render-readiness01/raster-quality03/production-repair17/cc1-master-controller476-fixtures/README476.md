# Controller 476 fixtures (CC1)

These are the checks behind `cc1-master-controller476-receipt.md`. They cover only the route delta from controller 464 to controller 476: the 08 row, its receipts, root's 08 export pair, and the launch manifest's new block. They run locally and only read. Nothing contacts a host. They are fixtures, not an actual import, and not final AV acceptance.

| File | What it is |
|---|---|
| `build476.py` | wrote `cc1-master447-manifest-draft476.json` from draft 460, once. It takes every 08 name from frozen 371's functions. Its printed `rows_byte_identical` count was wrong (0): it searched for unindented row text. Case D2 below does that check correctly, and finds all 14 rows byte-identical. |
| `delta476.py` | the top-level statement delta from 464 to 476 (ast) |
| `cc1-master-controller476.diff` | `diff -u` from 464 to 476 |
| `test476.py` | the 16 route-delta cases, listed below |
| `fixture476-08-manifest.json` | the synthetic 08 manifest that P2 and K2 pin. **It is not a 371 export.** Its names and `permit_sha256` (92119d3a, permit 474) are real. Its other digests, and its one file row, are dummies. |
| `test476.out.json`, `delta476.out.json` | the two scripts' outputs |
| `test476-K1.stdout`, `test476-K2.stdout`, `test476-K3.stdout` | what `check()` printed in K1–K3. The test's OUTDIR path is replaced by `OUTDIR`; nothing else is changed. |
| `MANIFEST476.json` | the sha256 and size of every file here |

## The cases

| Case | Shows |
|---|---|
| D0 | 476 pins draft 476 at 3e7b59d0; 464 still pins draft 460 at fcf0e7d7 |
| D1 | draft 476 is `json.dumps(indent=2)` plus a newline, as draft 460 is |
| D2 | each of draft 460's 14 non-08 rows occurs byte for byte, once, in draft 476; all 15 rows keep their order; the old 08 row is gone |
| D3 | row 08 is exactly the planned row. Its names are 371's own for attempt 1, its worker and sha are 447's CSS worker, its keys are 447's `ROW_KEYS \| IMPORT_KEYS` in row 01's order, and 08 is in 447 `CHANGED`. Root's Q permit copy, which `q_permit` names, is at 92119d3a. |
| R1 | 476's receipts are 464's, except 08's, in the same place. 08 now waits for its 371 a1 import receipt and then its CSS render receipt, as 01/04/12/14 do. No CSD 08 job is read. That is 25 receipts, against 464's 24. |
| A1 | `loop_sums_arguments` refuses 13 bad pairs: missing, malformed, or reusing a pinned 371 export's, the draft's, the launch manifest's or the gallery's name or sha. It returns None for check given neither argument, refuses run given neither, and accepts a plain new pair. |
| P1 | the four pinned 371 exports still pass the widened CSS set, with 08 and the gallery not given |
| P2 | with 08 given, its manifest goes through 371's own checks and must bind row 08's job, worker, worker sha, inputs and permit; the four pins pass beside it |
| P3 | an 08 sha that is not the file's is refused |
| P4 | an 08 manifest that 371 accepts but that binds permit 375 instead of 474 is refused |
| P5 | a draft without a CSS 08 row is refused |
| M1 | the launch manifest holds draft 476's 15 rows and a `launch476` block: the draft 476 source, six import pins with 08 among them, and both argument pairs |
| K1 | `check()` with no root arguments: pins ok, 08 and the gallery "not given", 25 pending, exit 3 |
| K2 | `check()` given the fixture's 08 pair: the 08 export is pinned beside the four, exit 3 |
| K3 | `check()` given an 08 pair whose file is not in Q: pins refused, exit 1 |
| U1 | `run()` given the gallery pair but no 08 pair is refused in `loop_sums_arguments`, before any receipt is read and before anything is printed |

## Stand-ins

447's Q is the CSD path, which does not exist on this machine. `rows_and_receipts()` compares the draft's `/data` names with it, so the real 447 module is used there. For `pinned_exports()` (P1–P4), and for `check()` in K1–K3, the test passes a copy of the 447 module's namespace in which only `Q` differs:
- in P1, this checkout's Q;
- in P2–P4 and K1–K3, OUTDIR/q;
- in U1, this checkout's Q, so that run's CSD-only location check passes.

OUTDIR/q holds hard links to the four pinned 371 exports (the same inodes, not copies) and the synthetic 08 files. On this machine every `/data` receipt is absent, so every receipt is pending.

## Running

Run these from the repository root. Q is production-repair17. OUTDIR must be new and outside Q.

```
python3 -I -B delta476.py Q
python3 -I -B test476.py  Q OUTDIR
```

The outputs here came from macOS with Python 3.15.0.
