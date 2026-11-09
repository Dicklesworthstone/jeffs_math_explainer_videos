# Review 487: CC1 controller 476 and draft 476, the 464→476 delta (CC0)

cc0, 2026-11-09, 05:22 UTC. Q is this folder. This is a source and fixture review only. No host was contacted, and nothing was launched, staged, signalled, transferred or imported. I did not edit 476, its draft, receipt or fixtures, or 464, 447, 371 or 448.

## Result

**No High, Medium or Low defect in the delta.** The 08 move to CSS is a bounded change to 464, and every 464 gate is kept in 464's order. Four notes follow; none needs a change to 476. I re-hashed the sealed files at 05:21Z:

| File | sha256 |
|---|---|
| `cc1-master-controller476.py` | `77cd7be786d87b847f31a31a281ef2d0d76f789276e9666af2667a45f691107e` |
| `cc1-master447-manifest-draft476.json` | `3e7b59d0cfb843cd3cd9741f72746139ee39359975d6bd99bb1c69e50cd3b457` |
| `cc1-master-controller476-receipt.md` | `1250a0bc0c611c7c41f5158bc9e32ce75153838534b4048cada5d7ce38a48c48` |
| `cc1-master-controller476-fixtures/MANIFEST476.json` | `5966e1a38d5593fa1702eabda6805e8fd63b1b866d4ee135b09d0d35e49b9fe6` |
| base `cc2-master-controller464.py` | `89ff9659…` |

## The delta, read against the source

**Draft 476.**
- It has 15 rows in registry order. The 14 non-08 rows equal draft 460's, which agrees with root's own comparison.
- Row 08 is built only from frozen 371's names for attempt 1:
  - `css_job`, `css_inputs` and `job_name(08, 1)`;
  - `inputs_path`, and the receipt in that job;
  - `WORKER` at `1074051b`, which is also 447's `WORKER352_CSS`;
  - `repair352-permits/08_loop_sums-css-permit474.json`.
- Row 08's keys are `ROW_KEYS | IMPORT_KEYS`, and its kind is `combined`.
- Root's Q copy of permit 474 is at `92119d3a`.
- 447's `combined_source447` is generic for CSS rows. It accepts row 08 and resolves it to `CSS/jobs/css-reviewed-final-08-loop-sums352`.

**Controller, AST against 464.**
- 13 functions are identical to 464's (docstrings aside): `sha`, `load`, `utc`, `say`, `rows_and_receipts`, `gallery_arguments`, `live_guard`, `state`, `one_pass`, `master_absent`, `verify_css_imports`, `ancestors` and `no_duplicates`.
- 5 functions changed and 1 is new. None is gone.
- At top level, 44 statements are identical, 12 named statements and the module docstring changed, and 2 were added (`LOOP08`, `loop_sums_arguments`). This matches the receipt, which counts the docstring separately.

**What changed, by function:**
- **`run()`:**
  - One line is added: `loop_sums_arguments(required=True)`, right after `gallery_arguments`. It runs before any pin, receipt or `/proc` read.
  - Three lines now pass `loop08` on, to `pinned_exports`, the start line and `manifest_bytes`.
  - Every other 464 line is kept in order: stable receipts, pin rechecks, `master_absent`, `no_duplicates`, `verify_css_imports`, the second `no_duplicates`, and one 447 launch.
- **`pinned_exports()`:**
  - The CSS set is now 01/04/08/12/14 (371) plus 05 (448).
  - A given 08 pair gets the same checks as the four 371 pins: a symlink-free file in Q at its sha, version 371, the right chapter, 371's `manifest_problems()`, and row 08's job, worker, worker sha, inputs and permit.
- **`loop_sums_arguments()`** follows `gallery_arguments`' rules. It refuses:
  - a name that matches the 371 export names, the draft or the 476 launch manifest;
  - a sha that matches a 371 export's;
  - a name or sha that is the gallery's, when the gallery pair is given.
- **`check()`:** the 08 pair is optional, and check prints whether it was given.
- **`main()`:** adds the two arguments and refuses with `not draft 476`.
- **`manifest_bytes()`:** the block is `launch476`, and it adds `loop_sums_arguments`.
- **`rows_and_receipts()`:** only the docstring changed. Row 08 now takes the generic CSS branch, which gives 25 receipts: 08's 371 a1 import receipt, then its CSS render receipt, in 08's place.
- **`verify_css_imports()`** is unchanged. Row 08 is checked by 371 at attempt 1, against its pin.
- **`CONTROLLERS`** adds 464 and 476 itself.

## Fixtures (scratch, outside the repo)

**CC1's 16 cases, rerun.** `test476.py` reran against Q into a new scratch folder. All 16 pass, and its output is byte-identical to CC1's recorded `test476.out.json` (`8c0df7fd…`). CC1's fixture folder is unchanged.

**My own, reused.** I adapted my review-469 harness to 476. It uses the same stubs: `subprocess.run`, `sleep`, a fake `/proc`, and stub checker `check()` functions. The file is `test477.py` (`d227c520…`); it was named before this review got its number.

The first run gave 61 of 62. The one failure was my own expectation for the `pinned_exports` diff: I expected 3 lines out and 5 in, and the real diff is 2 out and 4 in. I corrected that count only. The second run, in a fresh folder, gave **62 of 62** (output `0c92ae0f…`). Both versions and both outputs are kept.

The second run covered:

| Section | Cases | What it showed |
|---|---|---|
| Pins and draft | 8 | the pins hold; 25 receipts in the right order; the real four still pass the widened set |
| `pinned_exports` with the 08 pair | 10 | see below |
| `loop_sums_arguments` | 20 | every malformed or reused name or sha is refused, including the gallery's |
| `verify_css_imports` | 3 | 08 is checked by 371 at attempt 1; exports without 08 fail; a wrong 08 pin is refused |
| `run()` replay | 13 | see below |
| `no_duplicates` | 5 | see below |
| AST | 3 | as described above |

In the `pinned_exports` cases:
- A valid pair gives six pins. 08 alone, or the gallery alone, gives five.
- These are refused:
  - an 08 sha one digit off;
  - a version-448 file;
  - export 14's manifest given as 08;
  - a symlink;
  - a draft without row 08.
- **An export bound to the CSD permit 375 is refused** as binding another permit. An export naming the CSD job is refused by 371 itself.

In the `run()` replay:
- With both pairs, it launches once, after the six-export verify and the two `no_duplicates` checks.
- The start line and the `launch476` block carry both pairs and the six imports.
- With no 08 pair, or with the 08 pair equal to the gallery pair, it is refused before any read.
- An 08 export that changes while waiting is refused.
- An existing 476 manifest with another 08 sha is refused.
- A 464 launch manifest present is neither adopted nor touched.
- A pending 08 import ends at the deadline (exit 3). A failed 08 CSS render exits 1. Neither launches.

In the `no_duplicates` cases (fake `/proc`), 476 refuses:
- a live 461, 464 or second 476 run;
- a live 447 launch.

It allows a 476 `check`.

## Notes

- **N1. Run only 476.** Frozen 464 and older do not know 476's name: in the fake `/proc`, 464 allowed a live 476 run. 476 refuses the reverse. So once a 476 run is waiting, no older controller should be started.
- **N2. Check passing is not launch readiness.** Without the 08 pair, check reports "not given" and exits on its receipts (CC1's K1: exit 3). `run` refuses without both pairs.
- **N3. 08 still has to reach CSD.** These are dependencies outside 476:
  - relay394's `ORDER` has no 08, so the 08 render receipt and export reach CSD only through a relay successor;
  - 371's import preflight refuses while `D/jobs/csd-reviewed-final-08-loop-sums*` matches anything, and root's 480 record says that folder is absent.

  Until both are met, a 476 run waits and exits 3 without launching.
- **N4. Unchanged from 464.** `main` maps `OSError`, `ValueError` and `KeyError` to refusals, but a `TypeError` still prints a traceback. This is the same as note N3 in review 469.

## Not tested

- any CSD or CSS run;
- the real relay, import or export for 08;
- 447's own launch gates on real receipts.

## Not claimed

This review accepts no render, import or master. It accepts no picture, motion, sound, sync or AV result, and every final AV gap stays open. It authorises no launch or staging. No audio heard or checked.
