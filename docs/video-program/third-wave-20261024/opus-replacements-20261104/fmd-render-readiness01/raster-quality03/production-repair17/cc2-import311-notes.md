# CSD retained-final importer 311: notes for root

CC2, 2026-11-08. Prepared only. Nothing in this note has run on any host: no export, send, prepare, receive, import, render, master or permit, and no Beads change. Paths are written relative to D (CSD), TRJ, CSS, HZ2 and Q (`D/qualification01/production-repair17`).

| File | Lines | sha256 |
|---|---|---|
| `csd-retained-import311.py` (the helper) | 858 | `862968830a076c2030599b65e1f48b776501f87d99045cb662570045e613d5ea` |
| `csd-retained-import311-test.py` (source-level tests) | 595 | `ff39db345cc719ed667c0af836d89dbc3d7177b9a9832741e5f20abe9fe6d74a` |

Revised after root's review: `place()` no longer removes the staged name. Each new file now keeps two hard links, its staged name in the job and its placed name. Sections 6, 8, 10 and 11 describe that.
| `cc2-import311-notes.md` (this file) | n/a | given in the pane handoff |

The test file is the one extra file. Root asked for source-level tests that check the actual closure manifests, and keeping them out of the helper keeps the deployed helper free of test code.

Unchanged and re-hashed: 293 `8745c179…a06daf3`, 307 `dcf5e307…b41e837`, 284 `6e9b312f…066365`, 300 `dfc16db3…c881ec`.

## 1. What it fixes

307 B1: 293 can retain 02, 03, 13 and 15 only through a completed guarded CSD import. 293 checks this in two places:
- `imported()` requires a successful receipt at `D/jobs/<job>/resource-receipt.json` and import inputs under D, with every listed file at its hash.
- `chapter_gate()` then reads each final's worker, inputs, permit, evidence, receipts, render folder and narration at their original absolute paths, on the `D/jobs` filesystem.

Until now no CSD import existed. The existing imports of 13 and 15 (`trj-import-css-final13-97`, `trj-import-hz2-final15-56`) are TRJ imports, so 293 does not accept them.

## 2. Jobs and paths

There is one guarded job per source host. Each job is unique, and its name carries an attempt number.

| Host | Chapters | Job (attempt 1) | Guard inputs | Physical root on CSD | Alias created on CSD |
|---|---|---|---|---|---|
| trj | 02, 03 | `csd-retained-import311-trj-a1` | `Q/csd-retained-import311-trj-a1-inputs.json` | `D/work/imported-trj-root311` | the TRJ root path |
| css | 13 | `csd-retained-import311-css-a1` | `Q/csd-retained-import311-css-a1-inputs.json` | `D/work/imported-css-root311` | the CSS root path |
| hz2 | 15 | `csd-retained-import311-hz2-a1` | `Q/csd-retained-import311-hz2-a1-inputs.json` | `D/work/imported-hz2-root311` | the HZ2 root path |

The helper accepts a manifest only when it sits directly in Q as a regular file on a symlink-free path. Suggested names: `Q/retained-import311-<host>-manifest.json`.

Each foreign root becomes a symlink on CSD, at the same absolute path it has on its own host, pointing to the physical root. Every imported file therefore keeps its original absolute path. 293's `absolute(real=False)` accepts that, and `st_dev` is the D filesystem's device, because `D/work` and `D/jobs` are one filesystem. `prepare` and `receive` both check that.

## 3. Modes

| Mode | Host | Writes | What it does |
|---|---|---|---|
| `export --host H` | source host | stdout only | Computes the closure (section 5), hashes it, runs the manifest and closure checks on the source, and prints the manifest. |
| `send --host H --manifest M --manifest-sha S` | source host | stdout only | Checks the manifest's sha and shape. Re-hashes every foreign file before the first byte, then streams a header line and the files, hashing as it goes. |
| `check --host H [--attempt N] --manifest Q/M` | CSD | nothing | Reports what `prepare` would refuse. After an import, re-hashes every file through its original path and prints the two 293 row fields. |
| `prepare --host H [--attempt N] --manifest Q/M` | CSD | the guard inputs, opened with `'x'` | Runs the same checks as `check`, then writes the inputs and prints the guard command, the counts and the 293 row fields. |
| `receive …` | CSD, inside the guard only | the job folder, the physical root and the alias | The import itself (section 6). |

## 4. Relay for root

Run each host's import on its own, one after another. CSD has one native lane.

1. Copy the helper (same sha) to `Q` on CSD and to each source host, for example `TRJ/qualification01/production19/`, `CSS/qualification01/` and `HZ2/qualification01/production19/`.
2. On each source host, run `python3 -I -B <helper> export --host <h>`. Save stdout without clobbering anything on the controller as `Q/retained-import311-<h>-manifest.json`. A non-zero exit lists the refusals on stderr.
3. Copy that manifest to CSD `Q` and next to the helper on the source host. Check that all three shas are equal.
4. On CSD, run `/usr/bin/python3 -I -B Q/csd-retained-import311.py check --host <h> --manifest Q/retained-import311-<h>-manifest.json`. It must exit 0 with `problems: []`. It also reports `alias_state` (expected `absent`) and the new byte count.
5. On CSD, run `prepare` with the same arguments. It prints `guard_command`, `inputs_sha256` and `rows293`.
6. Pipe the two ends, as the 97/56 relays did, and record both exits:
   `ssh <h> 'python3 -I -B <helper> send --host <h> --manifest <copy> --manifest-sha <sha>' | ssh csd '<guard_command>'`
   - The receive succeeds only when both exits are 0 and `D/jobs/<job>/import-result.json` exists.
   - The guard returns 75 on a resource stop.
   - Start the pipe only while CSD's native lane is free. The guard takes the lane lock before it admits the job, so until then the sender sits blocked on a full pipe.
   - The receive runs under `/usr/bin/timeout 14400`, so a stalled sender cannot hold the lane.
7. Run `check` again with the same attempt. It re-hashes every file through the original paths, then prints `rows293`.
8. In 293's manifest, give each retained row the fields from its host's job:
   `import_receipt = D/jobs/<job>/resource-receipt.json` and `import_inputs = Q/<job>-inputs.json`. Rows 02 and 03 both name the trj job.

## 5. What each closure holds

- **The 97 rule** (`css-final13-export97.py`), applied to each chapter:
  - the render inputs and each of their rows;
  - the render receipt;
  - `render-summary.json` and each of its files;
  - every file in the job folder.
- **Every file 293 or the retained worker reads:**
  - the worker and the permit;
  - the permit's evidence: 9 items for 02, 5 for 03, 6 for 13 and 12 for 15, all on the chapter's own host;
  - the receipts `check()` reads:
    - TRJ: `trj-production19-delivery02`;
    - CSS: `css-runtime-transfer75`, `css-encoder-stop79` and `css-production19-delivery76`;
    - HZ2: `hz2-runtime-transfer47`, `hz2-encoder-stop03` and `hz2-production19-delivery49`;
  - the inherited delivery inputs and their rows;
  - the delivery narration folder: 105 WAVs plus four JSON files;
  - the timeline21 tables;
  - the score21 receipt, summary and files;
  - the timeline21 receipt;
  - the build19 qualification;
  - binary 19.

`export` records any path it adds beyond the 97 rule in `beyond_rule97`. For 13 and 15, the actual 97 and 56 exports already contain every such path (test `actual_closures_pass`), so that list should come back empty.

Rows under D are marked `shared`. They are never streamed or written. Each must already be a regular CSD file at the hash the source host recorded, and the guard inputs bind them, so the guard and 293's `imported()` re-hash them. Every other row is `foreign` and must sit under that host's root. Anything outside both is refused.

| Host | Foreign files | Foreign bytes (disk use) | Guard-measured growth (2 × foreign) | Shared D files (not streamed) |
|---|---|---|---|---|
| css (13) | 859 | 1,512,776,121 (1.41 GiB) | 2.82 GiB | 52 / 874,042,513 B |
| hz2 (15) | 855 | 1,492,420,117 (1.39 GiB) | 2.78 GiB | 52 / 874,039,915 B |
| trj (02 + 03) | unknown until `export` | unknown | unknown | unknown |

About 1.37 GiB of each of the CSS and HZ2 imports is the frozen runtime (`work/frozen-runtime75` and `work/frozen-runtime47`). These rows are needed because they sit in the chapters' render inputs, and 293 re-hashes every render-input row.

## 6. Receive, in order

1. **Receipt.** The receipt shows `admitted` true, with no `exit` and no `finished`. The policy is exactly 10/8/4/56/20. The recorded command is exactly this receive (timeout, python, helper, host, attempt, manifest).
2. **Guard inputs.** The inputs are at the admitted hash, and every row is still at its hash. They bind the helper, the manifest, the guard and every shared row.
3. **Manifest checks.** The manifest passes its own checks (section 7). Before reading a byte, the job then checks every shared file and every destination:
   - the foreign root is absent, or is a symlink to exactly this physical root;
   - the physical root is absent, or is a real directory on the D filesystem;
   - every destination is absent, or is a regular file with identical bytes;
   - no directory in the physical root is a symlink.
4. **Header line.** `CSD-RETAINED-IMPORT311 {"bytes","files","host","manifest_sha256","version"}`. It must name this manifest's sha, the host, the foreign file count and the foreign byte count.
5. **Staging.** Each new file goes to `job/incoming/NNNNN`, opened with `'xb'`. An already identical file is read and hashed but not kept. The job then checks every hash and that no data follows the last file. Nothing outside the job has been written yet.
6. **Closure, again.** The closure check runs again on the staged bytes (with CSD's own files for the shared rows). The same check ran on the source during `export`.
7. **Placement.** Each staged file gets its destination name under the physical root:
   - `os.link`, a second hard link to the staged file. It refuses an existing path, so nothing is replaced;
   - the staged name stays in `job/incoming`, so nothing is removed. Both names share one inode and the same disk blocks, which the job checks with `samefile`;
   - then `chmod` to the source mode, which applies to both names. New folders are created with mode 0700.
8. **Alias.** Only after every file is placed does the job create the foreign root symlink, and only if it is absent.
9. **Final check and result.** Every row is re-hashed through its original path, on the D device. `import-result.json` is written once, with `'x'`. It holds:
   - every file, with `written` or already identical, and each new file's kept staged name;
   - the counts;
   - `alias_created`;
   - `rows293`;
   - a scope line: exact import only; no render, master, permit, picture, motion, listening or AV acceptance.

## 7. Refusals

**Manifest checks** (`manifest_problems`), run in every mode:
- a wrong version, host, root or chapter pins;
- the pins are 293 RETAINED plus each final's terminal receipt, summary and inputs hashes, and the evidence counts;
- a malformed row, a non-normal or relative path, or `..`;
- a duplicate row;
- a row outside its host root and D, or a row naming a root itself;
- a D row marked foreign, meaning it would be streamed, or a host row marked shared;
- a row inside a 311 physical root, a 311 import job or a 311 inputs file;
- a row that is also another row's directory;
- a bad sha, size or mode (setuid bits, or no owner read);
- totals that disagree with the rows;
- guard-measured growth (twice the foreign bytes) above 3.5 GiB, that is, more than 1.75 GiB of foreign bytes.

**Closure checks** (`closure_problems`), run by `export` on the source and by `receive` on the staged bytes:
- any 293 or worker read missing, or at a hash other than its own record gives;
- render receipt:
  - not the pinned terminal success;
  - render policy not 8/4;
  - command not naming the pinned worker, job, inputs and permit;
- render inputs not binding the worker, permit, check receipts, delivery inputs, BIN19 and the 105 WAVs;
- render summary:
  - not matching the pinned chapter, frames, binary and permit;
  - an output outside `render/`;
  - the movie not the pinned final;
  - a missing table or soundtrack capture;
- permit:
  - not matching the pins, or not pointing at timeline21;
  - the wrong evidence count, or evidence off its host;
- timeline or cuelog rows outside the worker's narration folder or score21 stems, or not at their cuelog hash;
- a delivery receipt that does not bind its inputs;
- a check receipt that is not a terminal success;
- fewer or more than 105 narration WAVs.

**Duplicate jobs:** `prepare` also refuses an existing job or inputs name, and any earlier attempt for the host that is still running or has already imported.

## 8. Guard and disk

The guard measures growth as the growth of `D/jobs` plus the growth of `D/work`. Its `allocated()` counts a hard link once within each tree, but it walks the two trees separately. So when an import of S foreign bytes finishes:
- staging adds S to `D/jobs`, and the staged names stay there;
- placement adds S to `D/work`, as second names of the same inodes;
- the guard measures 2·S of growth, while the disk holds the bytes once and free space drops by S only.

The manifest check refuses 2·S above 3.5 GiB, which leaves half a GiB of the 4 GiB phase cap. CSS measures 2.82 GiB and HZ2 2.78 GiB. The round-trip test reproduces this accounting on its scratch tree with a copy of the guard's `allocated()`: `D/jobs` and `D/work` each count the full size, while one walk over both counts it once.

`prepare` and `check` require at least 10 GiB free, and at least 9 GiB still free after the new bytes. At 10:50Z, CSD's last guard sample (`preview274-review305/resource-receipt.json`) showed:
- 96.7 GiB free;
- `D/jobs` at 6.3 GiB;
- `D/work` at 2.2 GiB.

The imported files stay for good under both names. That uses about 2.8 GiB of disk plus TRJ's closure. In the guard's figures for later CSD jobs, the same bytes appear twice:
- once in `work_bytes` (`D/jobs`, measured against the 56 GiB work cap);
- once in `target_bytes` (`D/work`, measured against the 20 GiB target growth since the allocation baseline).

Root holds the baseline figure; I do not. Nothing in 311 removes or moves either name.

Hashing per job:
- the guard reads its inputs, including about 0.87 GB of shared rows, at start and at end;
- `receive` reads the shared rows once before the stream, the stream once, and everything once more through the alias;
- `check` after an import re-hashes everything again.

## 9. How 293 uses the result

- **`imported(row)`:**
  - the receipt is `D/jobs/<job>/resource-receipt.json` and must be a terminal success;
  - the inputs file is under D, at the receipt's `inputs_sha256`, and every row in it is at its hash (helper, manifest, guard, shared rows).
- **`chapter_gate(row)`:**
  - loads the worker through the alias;
  - re-runs its `check()`, which reads the host check receipts (imported), the D receipts, the qualification and score21 (CSD's own files) and the permit evidence (imported);
  - re-hashes every render-input row (imported, or CSD's own for D rows) and the render summary files (imported);
  - checks the tables, the narration bytes against rebind269, the score stems, the WAV capture, and `st_dev` against `D/jobs` for the staged outputs.

The importer makes no claim about `check()` succeeding on CSD beyond these files being present at their hashes. Root's 293 run is the gate.

The CSD-side contract lesson from 305b still applies to 293 and does not affect the import: run the master and captions consumers with cwd `D/work/production-repo07`. The import touches nothing there.

## 10. Failure and retry

The helper removes, renames and replaces nothing. Every staged name stays in its job folder, whether the job succeeds or fails.

- **Before placement** (a bad header, a truncated or corrupted stream, a closure refusal): the job folder keeps its staged files and nothing outside the job exists.
- **During placement:** some files sit in the physical root and there is no alias.

In either case, use the next attempt number. `prepare --attempt 2` keeps identical files, refuses different ones, fills in the rest and creates the alias at the end. A failed attempt's staged files stay in its job folder and keep counting in `D/jobs`. Nothing here authorizes cleaning them up.

## 11. Tests

Run: `python3 -I -B Q/csd-retained-import311-test.py --tmp <an existing scratch folder>`.
- It reads only its own folder.
- It writes only inside a new folder under `--tmp`, and deletes nothing.

The local run on 2026-11-08 passed 12 of 12. These are source-level checks: they open no connection to any host and do not exercise an import on CSD.

| Test | What it checks |
|---|---|
| `pins_equal_293_retained` | The four chapters' worker, job, inputs, permit, permit sha, movie sha and frames equal 293 RETAINED. D, Q, guard, python, binary 19 and the roots are equal. The job and inputs names fit `imported()` and the guard's job-name rule. |
| `host_constants_equal_workers` | Each host's root, narration, `check()` receipts and inherited delivery inputs equal the three retained workers' own source. The worker hashes equal the pins. |
| `terminal_records_equal_pins` | The receipt, summary and inputs pins equal the local terminal copies. Permit and movie hashes and evidence counts are equal too, and export97/56 `receipt_sha256` and `source_job` agree. |
| `actual_closures_pass` | `final13-export97.json` and `final15-export56.json`, as 311 manifests, pass both checks with zero problems. Every record with a local copy is read. Exactly five host files per host have no local copy: the three or two check receipts, the delivery inputs and the render inputs. |
| `shared_rows_equal_csd_records` | All 52 shared rows in each actual manifest equal CSD-recorded hashes, from score21-terminal01, timeline21-terminal01 (tables and receipt), the build19 qualification copy and binary 19. |
| `trj_records_closure` | No TRJ export exists here. The TRJ rows named by the local 02/03 records (14 evidence items, 13 narration WAVs, render outputs, receipts, summaries, permits, tables) pass. The check still lacks exactly the two render inputs and the delivery02 inputs and receipt, which have no local copy. |
| `mutations_refused` | 30 broken or hostile manifests are each refused with the named reason: missing evidence, WAV, check receipt, table, worker or job file; wrong movie, evidence or summary hash; altered receipt bytes; another host's root; a D row in the stream; `..` or relative paths; a duplicate; a row in the physical root or an import job; a file under a file; setuid or unreadable modes; a bad sha; an extra key; changed pins; root; totals; size; version; host. |
| `stream_round_trip_and_rerun` | On a scratch tree with the module's D and TRJ constants pointed into it: `send`, then header, staging, placement and alias. Every staged name is kept as a second link to the placed inode (`samefile`, link count 2), modes are kept, and bytes read back equal through the alias. A second run writes nothing. Guard-style `allocated()` counts the full size in each of `D/jobs` and `D/work`, and once over both. |
| `stream_refusals` | A wrong manifest hash, count or magic, a missing header, and a truncated, trailing or corrupted stream are refused. In every case nothing exists outside the job. |
| `destination_refusals` | A divergent file (kept unchanged), a real foreign root, an alias pointing elsewhere, a changed shared file, a symlink inside the physical root, a racing writer (its file kept and no alias), and a running or already imported earlier attempt are all refused. |
| `commands_bind_the_job` | The guard command is the guard argv, then `--`, then the exact receive argv the receipt must record. |
| `helper_source_writes_only_new_paths` | The helper writes only with `'x'`/`'xb'`. It has one hard link (placement) and one symlink (the alias), and no unlink, remove, rmdir, rename, replace, truncate, chown or shutil. It starts no process. |

Both files parse under the local python 3.15 and 3.9. Nothing was compiled to bytecode.

## 12. Limits and open items

- **Nothing has run.** No host has exported, sent, prepared or received anything. Every number for trj is unknown until `export` runs there. The size check refuses more than 1.75 GiB of foreign bytes (3.5 GiB measured growth), and splitting TRJ would need a code change.
- **TRJ's render inputs and delivery02 inputs are not local.** Their rows, including any D rows they carry, were not checked. Any D row that is absent on CSD, or differs from CSD's own file, stops `check` and `prepare` with `shared CSD file missing or different`.
- **Source hosts only.** `export` and `send` refuse a symlinked root, so they read the original host only. TRJ's alias copies of 13 and 15 are not used.
- **The alias needs a writable root parent.** It needs write access to the parent of the foreign root paths on CSD. `check` reports `cannot create the alias` if that is missing.
- **No acceptance claimed.** This is an exact copy by hash. It gives no listening, picture, motion, caption, 4K or AV acceptance. The Frederick narration is copied as bytes: nothing is synthesized or changed, and 293 compares those bytes to rebind269.
- **Not a 293 run.** A completed import does not by itself show that 293's `chapter_gate()` passes. Root's run of 293 decides that.

## 13. Against 307

- **B1:** resolved once root runs these three imports and puts their fields into the 293 manifest.
- **L2** (inputs written before admission): mitigated here. A refused admission costs only an attempt number, and `prepare --attempt N+1` starts clean.
- **L1:** cleared by root's 305b preflight, as recorded. Not touched here.
