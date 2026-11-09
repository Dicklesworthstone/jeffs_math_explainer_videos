# Review 488: CC1 relay 485 against relay 394 (CC2)

CC2 (RubyRaven), 2026-11-09, about 05:40 UTC, task 488. Root moved this review from CC3, who is on 486. Q is this folder; D, Q-on-CSD, CSS and CQ are 371's names.

**Independence.** I wrote none of the files reviewed here: 485, 394, 371, draft 476, controller 476 and 472 are CC1's, CC3's, CC0's and root's.

**What I did and did not do.**
- I read the source, ran CC1's 18 cases unchanged, and added 35 cases of my own. All of this ran in my scratch, outside the repo.
- No ssh, staging, transfer, import, launch or Beads edit.
- No file in Q was written or changed except this one, which is reserved in Agent Mail as RubyRaven, id 6828.

| Input | sha256 |
|---|---|
| `cc1-css352-import-relay485.py` (sealed) | `05617cb88da8786232ad7234a4e3208d024fe3560bd4713ac035f82f07f162d0` |
| `cc1-css352-import-relay485-receipt.md` | `a181ab828ade531fc24a40dbda4fbfb0283b37d1fb5c5411a8b4a40330f9984b` |
| `cc3-css352-import-relay394.py` (base) | `371f5d1ff415cc1309beebc7feba0f59bd4ba9b139153453d94326be16e64c8c` |
| `csd-css352-import371.py` (frozen) | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` |
| `cc1-master447-manifest-draft476.json` | `3e7b59d0cfb843cd3cd9741f72746139ee39359975d6bd99bb1c69e50cd3b457` |
| `08_loop_sums-css-permit474.json` | `92119d3a65fd5b3637f839ca83fb60eb455661d296337a1e1dc02cf777d0e59a` |
| `cc1-master-controller476.py` | `77cd7be786d87b847f31a31a281ef2d0d76f789276e9666af2667a45f691107e` |
| `root-08-host-move480.json` | `9c0d5aab91af6e1e40a53e415e05adcfe00bf7770b5b96f24d7f907eaf13ee40` |
| `css-08-permit-transfer474.json` | `9263a758055c3ca2cd0819b4648efddcfad8c17e0e46027a23417cb39b23a510` |
| `cc1-css-gallery-relay472.py` | `32f36d16e3cb3402e4c632b44a77c9852546fd8e96a90ff5127e8a1a66ecd5f1` |
| `cc0-relay472-review473.md` | `c73f076f4f9183586631a996db844127d0cb13fb3af61c1b5ee92c698f840b3b` |
| `css352-import371-14-continuation-manifest424.json` (basis of my synthetic 08 manifest) | `79fc3bf9e5c9ffa86f5f30df19a22431acb2b51d0ac620ac259706344caa5206` |

All of these were re-hashed in Q at 05:39Z. The six that 485's receipt lists equal its pins.

## Verdict

- **No defect. Nothing needs to change before root's run.** No serious finding, so none was raised early.
- **Route 476 and permit 474.** The binding is exact, and 371 carries it through to the CSS job's own render command and summary.
- **The frozen input map** is exactly the set 371 `prepare` writes.
- **The command and helper pins** are the ones 371 `receive` itself asserts.
- **The serial gate** asks about 01, 04, 12 and 14, then 08. It never asks about 05, which goes through 448 and relay 472.
- **One transfer at most.** There is one send, only after admission, never retried, and attempt 1 only.
- **No guard of 394 is removed or loosened.**
- **One Low (L1).** It is operational and inherited from 394. It costs more under 485 because 485 accepts attempt 1 only.
- **Five notes (N1–N5).**

## The delta, 394 to 485

- **What changed.** I read the full `diff`. An `ast` comparison finds:
  - 10 top-level statements added, 10 changed, plus the module docstring;
  - none removed and 34 byte-identical, in the same order.
  - CC1's `delta485.py`, rerun here, prints the same and is byte-identical to CC1's output.
- **Every change narrows, adds or is text:**
  - `local_manifest` now refuses every chapter but 08, then runs `route476()`. 394 accepted any ORDER chapter.
  - `main` accepts `--chapter 08_loop_sums` only.
  - `validate` c and g gain conditions. `POST_PROBE` gains two read-only fields, `receipt_command` and `listed_rows`.
  - `check_prepared` and `start` now call `is_frozen_set()` and `pinned_receive()`. These are the same expressions 394 had inline.
  - `finish` changes only its stop text ("run verify --attempt N when both hosts answer").
  - `VERSION`, `ORDER` and the docstrings.
- **Relay 472's changes, ported.** `frozen_rows`, `is_frozen_set`, `pinned_receive`, `check_prepared` and `validate` differ from 472's only in the helper label (448 → 371). `finish` is byte-identical to 472's. CC0's review 473 found no defect in those changes.

## Route 476 and permit 474

**What 485 checks** (`route476`, 485:250–266, run from `local_manifest` in `status`, `run` and `verify`):
- the pin's `job`, `worker`, `worker_sha256`, `inputs` and `permit` equal draft 476 row 08;
- the permit is named `08_loop_sums-css-permit474.json` and is at 92119d3a;
- root's copy of that permit in Q here is at 92119d3a;
- draft 476 here is at 3e7b59d0;
- `expected_rows(08, attempt)` equals row 08's `import_receipt` and `import_inputs`, which holds only at attempt 1.

**What is new.** 371's `pin_problems` (371:254–257) already forces the job, worker, worker sha and inputs to 352's names. So the new part is the permit, its sha, and the attempt.

**Draft 476 row 08, as read:**
- job `css-reviewed-final-08-loop-sums352`;
- worker `CQ/css-final-repair352.py` at 1074051b;
- inputs `CQ/final-08_loop_sums-inputs352.json`;
- permit `CQ/repair352-permits/08_loop_sums-css-permit474.json`;
- import names `D/jobs/csd-css352-import371-08-loop-sums-a1/resource-receipt.json` and `Q/csd-css352-import371-08-loop-sums-a1-inputs.json`.

These are exactly 371's `css_job`, `css_inputs` and `job_name(…, 1)` / `inputs_path` (my R2).

**Why pinning the permit means the CSS job used permit 474.** 371's closure check, run on CSS in `export` and on CSD in `receive` from the staged bytes, requires:
- the CSS render command's `--permit` to equal the pin (371:383);
- the render summary's `permit_sha256` to equal the pin (371:428–430);
- the permit file in the closure to be held at the pin's sha (371:367, 394);
- the permit's own fields to match: chapter, host, frames and binary (371:441–444).

So a manifest that passes 485's route can import only if the CSS 08 job really rendered from permit 474 at 92119d3a.

**The other two copies of the permit:**
- **CSS.** `css-08-permit-transfer474.json` placed it in `CQ/repair352-permits/` at 92119d3a, and its log reads exit 0.
- **CSD Q.** 371's `plan_problems` requires root's CSD Q copy at the pin's sha (371:507–509).

**One host per chapter.** `plan_problems` also refuses while any `csd-reviewed-final-08-loop-sums*` job exists on CSD (371:505–506). Record 480 shows the CSD 08 waiter withdrawn with no job folder.

**Controller 476** binds the same attempt-1 names for row 08 (476:142–145). It rechecks the import with 371's own `check` (`verify_css_imports`, 476:293–330), so it does not depend on 485's `verify`.

## The exact frozen input map

- **The set matches 371's.** 371 `prepare` (371:785–789) writes, sorted and deduplicated:
  - the helper, the manifest, `GUARD` and `q_permit(pin)`;
  - every shared and system path.

  On a synthetic 08 manifest built from root's real 14 export (3,202 shared and 2,269 system rows), `frozen_rows()` plus the guard row is that same path set: 5,475 rows (F1). `expected_inputs()`, g's count, is also 5,475 (F2).
- **Every substitution is refused.** Each of the following makes `is_frozen_set()` false (F4):
  - a shared row dropped;
  - an extra row;
  - the guard row missing;
  - the Q permit row, the helper or the manifest at another sha.

  CC1's V2, V3 and P1 show the same for the validation and for `--prepared`, and that 394 accepts V2 and V3.
- **The guard row's hash is still unpinned** (N5). f, g's re-hash and 371 `receive` (371:814–815) cover it, as review 473 said for 472.

## The command and helper pins

- **The receive command.** `pinned_receive()` is 371's `receive_command(Q/371, 08, 1, Q/manifest)`, starting `/usr/bin/timeout 14400` (C1, CC1 A1). It is the tail of 371's `guard_command`, and the same list 371 `receive` asserts against the guard receipt (371:807).
- **The helper hash.** 371 `check` reports `helper_sha256` as the hash of the CSD file that ran (371:867). Its done branch prints every key `validate` reads (371:867–881), so c and g can pass on a real import (V1, CC1 V1).
- **Refusals.** c fails when the receipt command names another manifest (V2) or attempt 7 (CC1 V4), and when check reports another helper sha (CC1 V5).

## Serial order, and gallery 448

- **The gate.** `earlier = ORDER[:ORDER.index('08_loop_sums')]` is 01, 04, 12, 14. The CSD probe asks only about those and 08 (O1, CC1 G1). With all four imported, nothing is refused (O2). With 12 not imported (my O3) or 14 not imported (CC1 G2), 08 is refused.
- **05 stays separate.** 05 is never asked about, and 485 refuses 05 as a chapter. The `05_gallery` entry in 485's ORDER is unreachable. Relay 472 is unchanged and does not know 08.
- **The two share one lane.** Both relays use CSD's single native lane, which is why L1 matters.

## No duplicate transfer, no weakened guard

**Code paths.**
- `spawn()` is called twice, both in `start()`, and `start()` once, in `run()` (D4).
- The send starts only after the CSD job probe shows admission, a pid, no finish and the exact receive command (485:469). Admission is never retried.

**A second import of 08 at attempt 1 is refused:**
- after `prepare` has written its inputs (D1);
- once a1 has imported (D2);
- through `--prepared` once a1 has a job (D3);
- by 371's own `earlier_attempts`, and by `prepare`'s `x` opens of the inputs (371:788), against a concurrent second `prepare`. This last one is by reading only.

`route476` refuses every other attempt (R3, CC1 L3), so 485 can make at most one 08 import job.

**Every refusal of 394 is kept:**
- the helper pins on all three hosts;
- the manifest at the given sha in Q, CQ and Q-on-CSD;
- the CSS render receipt a terminal success with matching receipt and summary bytes;
- the attempt rules;
- the lane waits;
- 371 `check` before `prepare`;
- `prepare`'s printed job, inputs, command, rows and pin;
- the exit codes;
- the `mkdir` and `x` record folders.

394 itself still refuses 08 (R9, CC1 L6).

## L1 (Low): a stop at admission leaves the remote guard queued, and 485 has no second attempt

**What happens.**
- After its last lane check, `start()` launches the guarded receive over ssh and polls for admission for up to 1,800 s (485:461–479).
- If another native job takes the lane in the few seconds between that check and the guard's lock, admission does not come within the 1,800 s. 485 then closes the pipe, waits up to 600 s for the local ssh to end, and stops with code 3, "no byte sent" (485:471–478). It kills neither the local ssh nor the remote guard, which stays blocked on the lane. A probe failure during the poll takes the same path (485:465–467), and its comment says so.
- When the lane frees, the guard admits a1 and 371 `receive` reads an empty stream. a1 is then finished without an import.
- Under 394, root would use attempt 2. Under 485, `route476` refuses every attempt but 1, so 08 would need a new draft and controller. The receipt says that much, but not that this path can lead there.

**This is not a code defect.** It fails closed, and nothing wrong is imported.

**Recovery.** Withdraw the blocked waiter before it is admitted, as root did for the CSD 08 waiter in record 480: no job folder, `blocked_flock`. a1 then has inputs and no job, which is the one case `gather` allows `--prepared` for. My D3 output quotes the rule: "--prepared needs attempt 1 to be the latest, with inputs and no job". `run --attempt 1 --prepared` should then pass that gate, but I did not run that accepted case. CC1's P1 shows `check_prepared` accepting the exact set.

**Suggestion for root.** Run 485 with nothing else queued on CSD's native lane: not alongside relay 472's 05 import, and not alongside any other CSD native job. Record 480 still showed one holder and four waiters, and `wait_lane` already waits for all of them.

## Notes

- **N1. A theoretical false refusal (fail-closed).**
  - 371 accepts a manifest that lists `D/guard_csd02.py` as a shared row (F5a). For such a manifest, `is_frozen_set()` refuses 371's own `prepare` set (F5b), so g and `--prepared` would refuse a good import.
  - None of root's four real 371 exports (392, 396, 397, 424) has such a row. Their only guard rows are CSS's `guard_css74.py` and a frozen-runtime copy of `guard_trj01.py`, both foreign.
  - They do list 21 or 22 shared rows under Q-on-CSD: audio17, review records and preview scripts. None of these is Q/371, a 371 export or a Q permit copy.
  - **Check once the real 08 export exists:** no row's path is `D/guard_csd02.py`.
  - The same logic was already in 394's `--prepared`. Controller 476 does not use 485's g.
- **N2. Receipt line numbers.** The receipt cites 371:504–505 for the one-host check. The check itself is 371:505–506, and 504 is its comment.
- **N3. CC1's fixture hard links share files with Q.**
  - CC1's run01 fixture folder (in CC1's session scratch, outside the repo) holds hard links to Q's 371, draft 476 and permit 474. Each of those Q files has a link count of 2.
  - A write through any of those links would change root's pinned file. All three were still at their pins at 05:39Z.
  - When I reran CC1's test, I gave it a sha-checked scratch copy of the five Q files it reads, so its links landed on my copies. Q's link counts stayed at 2.
- **N4. What the serial gate counts.** A chapter counts as imported when its receipt is a terminal success and its import result exists. A passing relay validation is not required. This is unchanged from 394. Controller 476 rechecks every import itself.
- **N5. The guard row's hash is unpinned**, as above and as review 473 noted. Unchanged.

## Tests (my scratch, outside the repo; Python 3.15.0, `-I -B`)

| Run | Result |
|---|---|
| CC1 `test485.py` (`f7055cd9…`), copied from CC1's scratch (Mail 17754) and run unedited against a sha-checked scratch copy of the five Q files | 18 of 18. The output JSON is byte-identical to CC1's own `test485.out.json` (`bbc3c59b…`). |
| CC1 `delta485.py` (`895bed09…`) | Byte-identical to CC1's `delta485.out.json` (`36bf13d7…`) |
| My `delta488.py` (`ddc3dd27…`), run02 | 35 of 35: R1–R10, F1–F5, C1, V1–V3, O1–O3, D1–D4 (output `12c7ff70…`) |
| My `delta488.py`, run01 | It stopped at R9 because my test caught only 485's `Stop` class, not 394's. 394 had refused correctly. I fixed that one `except` line in place and kept run01's output (`3a543176…`). |

**How my cases differ from CC1's.** My synthetic 08 manifest is root's real 14 export with only the pin changed to draft 476 row 08 and permit 474. It therefore has real row counts and real paths, where CC1's fixture has three rows. Every host call is replaced: `remote()` and `spawn()` raise, and `probe()` and `run_371()` return canned facts.

## Not done, not claimed

- **Not exercised:**
  - any host, the real guard, and the real 08 export, which does not exist yet;
  - `run()` end to end, here or in CC1's tests;
  - whether 01/04/12/14 are imported on CSD now.
- **Not accepted:** no import, chapter or master. This review authorizes no staging or launch.
- **Still open:** final timestamps, pictures, motion, listening review, fine sync and independent AV review. Frederick remains the narration.
