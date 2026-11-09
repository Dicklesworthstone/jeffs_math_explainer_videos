# Review 456: gallery import 448 against import 371 (CC2)

cc2, 2026-11-09, about 04:20 UTC. This is an independent review of `csd-gallery-import448.py` against the original `csd-css352-import371.py`. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

Host paths below use 448's own names:
- `D`, `CSS` and `RECOVERY` (448:70, :74, :84);
- `DATA` (448:76), the top folder that holds every project tree;
- `TOP` = `D/work/imported-css-root311`;
- `TOP_SSD` = `D/work/imported-css-ssd442`.

Nothing ran on any host. There was no transfer, import, alias change or remote command, and I did not touch the live 442 render, CSD09 or any queue. Every test ran locally in small scratch fixtures, outside the repo. No audio was heard or checked.

## Verdict

- **No High and no Medium finding in 448.** Its changes from 371 are confined to:
  - the second foreign root and its exact alias;
  - the gallery-only names;
  - the second physical root.

  The rules for placement, no overwrite, growth, the native lane, admission and attempts are 371's, and unchanged.
- My fixture passed **67/67** on the real 448 bytes. It covered:
  - export and send on a CSS view;
  - prepare, receive and check on a CSD view;
  - positive placement across both roots;
  - 7 corrupt streams and 6 no-overwrite cases (one is a race);
  - 11 alias and physical-root cases;
  - 8 manifest-row rules, the growth boundary, and 7 admission and 4 attempt cases.

  It is synthetic. It is not evidence from the real 442 job, which has not finished.
- **Five Lows or Infos**, F1–F5. F1 matters for planning: a 446 export cannot feed 448.
- **Five read-only pre-checks** for root, P1–P5. P1 is the one that could block: where the 442 launcher sat on CSS.

## 1. Files

| File | sha256 | Use |
|---|---|---|
| `csd-gallery-import448.py` | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` | reviewed in full (932 lines) |
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | the base; full diff read |
| `css-gallery-export446.py` | `0b26eecc544020e8bda4945d4932fd12eef2acaa0d5d0ec59a8e3370b8160138` | diffed against 448 |
| `css-gallery-ssd442.py` | `ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429` | `render()`, the summary and the names it gives; diffed against 440 (only `job_name` differs) |
| `launch_css_ssd442.py` | `6daf9713f10dad918a4b17c62202bd155ec25debeac2d51b5e6156ac2fa8fb49` | read in full: what `gallery-inputs442.json` binds and the render command |
| `guard_css_ssd441.py` | `40febbd05cae9f3bee3c9b38154216678bc81fe565d81016cfd1d3e9abdf2529` | read in full: the original guard's lane and limits; jobs under RECOVERY |
| `combined-master447.py` | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` | `combined_source447`, `imported()`, the combined gate |
| `master447-manifest-draft449.json` | `0108537698028e0b7389533f85c552c652e9b7aef72f86b6028ae209870be5aa` | 05 row names |
| `css-gallery-relay453.py` | `c8f5539d2290416543e4a9bee50d7ef5bc2af82b61cf687ea73f30198881ec0d` | diffed against 394 at `371f5d1f`; see §5 |
| `import448-root-checks.json`, `import448-tests450/results.json`, `recovery440/441/442` records, `gallery-export446-prepared.json` | as in Q | root's scope |
| `css352-import371-01-the-bump-manifest392.json`, `…-12-residues-manifest397.json` | `633c06d0…`, `73c62b8d…` | real export sizes, for P4 |

## 2. What 448 changes from 371, checked

| Area | 448 | Checked |
|---|---|---|
| Two foreign roots | `kind_of` checks D first, then CSS or RECOVERY. `physical()` maps RECOVERY to TOP_SSD and CSS to TOP, after `clean_path`. TOP_SSD joins `ROOTS311`, so no row may sit inside it. | Read; fixture §4.1, §4.6 |
| Aliases | `alias_problems()` requires both: CSS → `str(TOP)` and RECOVERY → `str(TOP_SSD)`. Each must be a symlink with exactly that readlink, to a real directory with no symlink, on D/jobs' device. It is checked in prepare and check, in receive before staging, before placement and after placement. 448 never creates, replaces or changes an alias. | Read; fixture §4.5 |
| Placement | `place()` picks the root by destination, `make_dirs` only below that root, and adds the second name with `os.link`, so an existing path is never replaced. The staged name stays. | Fixture §4.2, §4.4 |
| Gallery names | `css_job` = `css-reviewed-final-05-gallery352-ssd442`; `css_inputs` = `RECOVERY/gallery-inputs442.json`; `WORKER` = the 442 worker at `ce9baf6a`; `GUARD_CSS` = RECOVERY's 441 guard; the render job lives under `RECOVERY/jobs`. | All equal 442's and the 442 launcher's names, and draft 449's 05 row |
| 442 closure | The rules are 371's, applied to the 442 job. The render command must start with `python3 -I -B WORKER render` and name the pinned chapter, job, inputs and permit (442's extra `--threads 16` is allowed). The summary needs `host css`, `worker352_sha256` equal to the 442 worker's own hash (442 writes `sha(Path(__file__))`), and the narration of delivery 309. The permit must sit in CSS's `repair352-permits` and be named for 05. The inputs must bind the RECOVERY guard. | Read against 442 `render()` (:860–917) and the launcher; fixture §4.1 |
| Unchanged | `GROWTH_LIMIT` 3.5 GiB, so 2 × new bytes ≤ 3.5 GiB; free space and caps; `POLICY` 10/8/4/56/20; `guard_command --phase native` with `timeout 14400`; `receive` refuses outside an admitted guard job; `earlier_attempts`; one host per chapter; `import-result.json` opened `'x'`; `check`. | Line diff; fixture §4.7, §4.8 |
| Version | `VERSION`/`MAGIC` 448 (446: `446`; 371: `371`). | F1 |

The rows350 names that 448 prints are `D/jobs/csd-css352-import448-05-gallery-a1/resource-receipt.json` and `Q/csd-css352-import448-05-gallery-a1-inputs.json`. In the fixture, with the box prefix removed, they equal draft 449's 05 row exactly.

## 3. Findings

### F1 (Low, planning; refuses safely). A 446 export or stream cannot feed 448

- **Cause:** 446 writes `version: '446'` and `MAGIC …IMPORT446`; 448 requires `448`.
- **At prepare:** a 446 manifest is refused with "not a version-371 manifest".
- **At receive:** a 446 stream is refused with "not a 371 import stream" (fixture §4.1, §4.3).
- **So:** the gallery's export and send on CSS must be 448's own `export` and `send` modes (448 allows them on CSS).
- **446's place:** it is useful only as a read-only dry run. Root's 446 record says the positive export has not run, so nothing is lost.
- **Link to 457:** CC3's 457 asks that 393/449 pin each chapter's manifest to root's named export. For the gallery, that named export has to be 448's.

### F2 (Low; refuses with a traceback). Any chapter other than 05 raises a bare `AssertionError`

- **Cause:** `css_job()` and `css_inputs()` assert `chapter == '05_gallery'`, and `pin_problems()` calls them, so `manifest_problems()` raises instead of returning a problem.
- **Effect:** export, prepare, check and receive then end in a traceback with no `refused` line. The argparse choices still offer all ten chapters.
- **Nothing is written.**
- **Successor fix:** narrow `CHAPTERS` to `('05_gallery',)`, or return a problem.

### F3 (Low, reporting). Before an import, `check` reports only the CSS alias

The pre-import report's `alias` and `alias_state` fields name CSS only. A missing or wrong RECOVERY alias shows up only in the `problems` text. A successor could report both aliases.

### F4 (Low, wording). Stale and contradictory text

- **Three module docstrings.** The second is 446's, which says import modes are "disabled" and "NOT authorized here", the opposite of 448. Only the first becomes `__doc__`, the argparse description, and it is correct.
- **Stale refusal text:** 448 refuses its own version mismatches with "version-371" and "371 import stream", even when it is refusing a 371 manifest.
- **Stale 371 description:** the docs still give the job pattern `csd-css352-import371-…` and only `imported-css-root311`. The growth comment still describes css-a1's 1.4 GB.

### F5 (Info; inherited from 371). A placement race leaves earlier links placed

- **The race:** a different file appears at a destination after the plan but before `place()`.
- **What 448 does:** `os.link` raises `FileExistsError`, and the raced bytes are left untouched.
- **What is left behind:** the new links placed earlier in plan order stay, with no `import-result.json`. In the fixture that was 19 links.
- **Why it is harmless:** they are links to copies verified at their manifest hashes, and a later attempt keeps them as identical existing files.
- **When it can happen:** only if something writes into TOP or TOP_SSD during the import.

## 4. Fixture (scratch, outside the repo)

| File | sha256 | Result |
|---|---|---|
| `fixture456-run01.py` (first version, kept) | `73e5d9a7c68972416e1cf3b8f5d05e3d69353468f9800d190c2d00f1959f8d94` | 32 passed. Then a harness bug: the identical-file case read its bytes through the CSD-view alias, which pointed at the still-empty TOP_SSD. Its boxes are kept. |
| `fixture456.py` | `b48250aa4021d7f4a8bdefd9d08ba959412f997b6d4bcb2ac6b0bc0d7bdeb46b` | **67/67**, 0 FAIL; output `0f5e7d0cccab3c469ca415aac92f18fdb979dc86218d2e581789dc7b67c25d36` |

**Method.**
- **Real files:** the fixture reads the real 448, 442 worker, 441 guard, 442 launcher and draft 449 from local Q, after a hash check.
- **Substitutions:** in memory, it replaces only the three roots `D`, `CSS` and `RECOVERY`, `DATA`, and binary 268's pin (pointed at a small stand-in). Each replacement must occur exactly once. The result is compiled and run with `compile()`; every mode runs as written.
- **Boxes:** each case builds its own small box (about 150 files) in a new folder. Nothing in a box is deleted; anything replaced is renamed into the box's aside folder.
- **The CSS view:** CSS and RECOVERY are real, and D is the alias into FROZEN. Export and send run here, and the manifest is staged in RECOVERY, as 453 does.
- **The CSD view:** reached by renames only. D becomes real, `CSS → str(TOP)` and `RECOVERY → str(TOP_SSD)`. css-a1's files are pre-placed in TOP as identical copies (110 files), and root's permit copy and the helper go in Q.
- **The guard:** a fake guard writes the admitted receipt and later finishes it, after re-hashing every input row.
- **The job:** the 442 job content is synthetic: a 128 KiB movie, the five tables, one soundtrack capture, logs, the summary and the receipt.

### 4.1 CSS side (9)

- **Accepted:** a terminal 442 render exports a 448 manifest:
  - 150 rows: 135 foreign (17 RECOVERY, 118 CSS), 14 shared and 1 system;
  - the RECOVERY rows are the worker, guard, launcher, inputs, RECOVERY baseline, ssd440 receipt and every file of the 442 job;
  - the stream is the 448 header plus every foreign byte, in manifest order.
- **Export refuses:**
  - a running receipt;
  - a summary with another `worker352_sha256`;
  - a symlink in the job;
  - a launcher in another tree under DATA;
  - a launcher outside DATA, as a system row not in CSD's manifests.
- **446:** 448 refuses a 446 manifest (F1).

### 4.2 Positive import on CSD (12)

- **prepare:**
  - refuses nothing and writes the guard inputs once;
  - prints `… --phase native … -- /usr/bin/timeout 14400 …`;
  - 25 new files and 110 identical;
  - the inputs are the helper, manifest, guard and Q permit, plus 14 shared and 1 system row (19 rows).
- **receive:** "Imported 135 files; 25 written, 110 already identical".
- **Placement:**
  - all 135 foreign rows resolve through their own alias to their own physical root, at their hash and mode;
  - each written file is a second hard link of its staged copy in `incoming/`;
  - css-a1's 110 files keep their inodes;
  - outside the job, only new names under TOP or TOP_SSD and the guard inputs appear, and nothing existing changes.
- **`import-result`:** names both foreign roots and both physical roots, with `alias_created` false.
- **`check` after the finish:** exit 0, `problems` empty, and rows350 equal to draft 449's 05 row.
- **No duplicate:**
  - prepare a1 again refuses: the job exists and the inputs exist;
  - prepare a2 refuses: "already imported by";
  - a RECOVERY file changed later is reported by `check` (exit 1).

### 4.3 Corrupt streams (7)

Every case is refused before any destination write, and nothing outside the job changes:
- the stream truncated by one byte;
- one byte flipped in the RECOVERY movie;
- one byte flipped in a CSS narration WAV that already exists identically in TOP (the stream bytes are still hashed);
- one trailing byte;
- a header naming another manifest;
- a 446 magic;
- an empty stream.

### 4.4 No overwrite (6)

- **Different file at a RECOVERY destination, planted before prepare:** prepare refuses "existing destination differs; never overwritten", and the file keeps its inode and bytes.
- **Different file at a CSS destination, planted before prepare:** the same.
- **Different file at a RECOVERY destination, planted after admission:** receive refuses before reading one byte (stdin is still at 0, and no `incoming/` exists).
- **Different file at a CSS destination, planted after admission:** the same.
- **An identical file at a RECOVERY destination** is kept, written false with the same inode, and the rest import.
- **Race:** see F5.

### 4.5 Aliases and physical roots (11)

Prepare refuses each of these:
- the RECOVERY alias missing;
- the RECOVERY alias pointing at TOP;
- a relative RECOVERY link that resolves to TOP_SSD (the exact string is required);
- RECOVERY as a real directory;
- the CSS alias pointing at TOP_SSD;
- TOP_SSD missing;
- TOP_SSD a symlink to a real directory;
- a symlinked `jobs/` inside TOP_SSD (the link target stays empty);
- a destination that is itself a symlink.

The RECOVERY alias retargeted after staging is refused before the first placement, and D/work does not change.

### 4.6 Manifest rows (8)

Refused:
- a RECOVERY row marked shared;
- a row inside TOP_SSD;
- a row beside RECOVERY under its `ssd/work` parent;
- a row naming RECOVERY itself;
- a version 371 manifest;
- the ssd440 job;
- the 352 inputs;
- another chapter, which raises (F2).

### 4.7 Growth (3)

- The limit is 3,758,096,384 bytes.
- New bytes of exactly half the limit pass, and one byte more is refused. This used `plan_problems` on an edited manifest; no large file was made.

### 4.8 Admission, attempts and hosts (11)

- **receive refuses:**
  - a guard-refused admission;
  - a finished receipt;
  - policy phase growth 5;
  - a command without the timeout;
  - a command for another attempt;
  - another inputs hash;
  - the Q manifest changed after admission.

  Each is refused before stdin is read.
- **Attempts:**
  - a2 is refused while a1 runs;
  - after the guard refuses a1, a2 is not refused. 448 never increments, so that choice is root's (390).
- **Hosts:** a CSD `csd-reviewed-final-05-gallery*` job refuses.
- **Legacy folder:** a 371 05 import folder does not block 448 (Info: 448 counts only its own attempts).

### 4.9 Against root's 450 (8 checks)

My cases repeat root's eight in separate code: positive two-root placement, identical kept, different refused, symlink parent, missing second alias, truncated, wrong hash and trailing. They add:
- the CSS-side export and send, and their refusals;
- the guard inputs, `check` after the import, and draft 449's names;
- receive refusing before it reads stdin;
- the header cases;
- the alias variants and the retarget after staging;
- the race;
- the manifest-row rules and the growth boundary;
- admission, attempts and hosts;
- the 446 incompatibility.

## 5. Pre-checks for root (read-only; not 448 defects)

- **P1 (could block). Where the 442 launcher sat on CSS.**
  - `launch_css_ssd442.py` binds its own resolved path into `gallery-inputs442.json`.
  - 448's export accepts that row only under CSS or RECOVERY. Under another tree in DATA it raises "in another project tree". Outside DATA it is a system row and refused, because it is not in CSD's manifests. My fixture reproduced both refusals.
  - Root's 442 record gives the launcher's hash but not its path. A read of `gallery-inputs442.json` on CSS answers it. If the launcher is outside both roots, 448 cannot import this render without a code change.
- **P2. The CSD alias must exist before prepare.**
  - `RECOVERY` on CSD must be a symlink whose readlink is exactly `str(TOP_SSD)`, the absolute string; a relative link is refused.
  - TOP_SSD must be a real directory on D/jobs' device.
  - 448 makes directories only below TOP_SSD. It never creates RECOVERY's parents on CSD, the alias or TOP_SSD.
- **P3. The import uses CSD's single native lane.**
  - `guard_csd02 --phase native` serializes the import with every other native job, including CSD09 while it is live.
  - A guard-refused admission uses up a1, since the job folder exists, and 448 then allows a2.
  - The relay's lane probe and root's attempt decision are the only things that stop this.
- **P4. Growth fits.**
  - From root's real exports 392 and 397, a 352 CSS job folder is 19–28 MB: a movie of 6.6–7.9 MB and one 12.6–20.2 MB soundtrack WAV. At the gallery's 9,879 frames that scales to about 45 MB.
  - The CSS rows outside the job are about 2.20 GB, of which about 1.4 GB is css-a1's already (371 notes). So the gallery's new bytes are about 0.85 GB if it is imported first, and fewer after the 371 imports.
  - Doubled, that is under the 3.5 GiB limit. `check` prints the real `new_bytes` once 442 is terminal.
- **P5. The failed 352 receipt becomes visible on CSD.**
  - 442's inputs bind `CSS/jobs/css-reviewed-final-05-gallery352/resource-receipt.json`: exit −15, a resource stop.
  - After the import it is visible on CSD at that path through the CSS alias.
  - Anything still pointing at the 352 gallery job, such as 393 or draft 384, reads that failure and stops. That is correct, and fails safely.

## 6. Related, outside 448

- **Relay 453.** CC3's `cc3-relay453-review457.md` (`7f1d807b…`) reports a Medium: 453 lacks 394's post-import validation a–h, its `verify` mode and its inputs re-hash. My diff of 453 against 394 at `371f5d1f` shows the same, and also that 453 lacks:
  - the `None not in attempts` condition;
  - the record of an unexpected exception.

  I defer to 457.
- **449.** My review 395 F1 applies to it: `verify_css_imports()` runs after the launch manifest is written (449 :330, then :333).

## 7. Not claimed

- No real export, transfer, import, alias, render, master or 442 evidence.
- No picture, motion, listening, fine-sync, caption or independent AV acceptance.
- No publication authorization.
