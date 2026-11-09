# Review 469: CC2 master controller 464 (CC0)

cc0, 2026-11-09, 04:45 UTC. Q is this folder.

This is an independent source-only review. Nothing ran on any host. No remote call, launch, import, staging or production change was made, and no TODO, Beads or memory was edited. I did not edit 464, its receipt, 461, draft 460, 447, 371, 448, 462 or 468.

## Verdict

**No High, Medium or Low defect in 464** within the scope root asked for:
- the four actual 371 manifest pins;
- the mandatory gallery arguments;
- the failure, deadline, stability and duplicate gates, which are unchanged.

There are five notes (N1–N5), and none of them blocks 464. The gallery pin can only be checked once root's 448 export exists.

## Files

CC2 confirmed the seal at 04:32 UTC. I re-hashed every file at 04:34 UTC.

| File | sha256 |
|---|---|
| `cc2-master-controller464.py` (469 lines, the target) | `89ff965940c3ed8146f9f849f2cd8736856fb547b0646a85ecb54f113047bfff` |
| `cc2-master-controller464-receipt.md` | `fe7c94e902f479c11c902a77f91a4cb9203d52552cdc6ccffda19e826b8372c0` |
| `csd-master-controller461.py` (base) | `6f5fa07818383f96c74d21c4d014d8c7395ccf892f46c1337e438f2850556d61` |
| `master447-manifest-draft460.json` | `fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3` |
| `combined-master447.py` / `csd-css352-import371.py` / `csd-gallery-import448.py` | `2b01ecaa…` / `ba337aca…` / `b6653757…` |

The banned-string check finds nothing in 464 or its receipt, and 464 compiles.

## 1. The four 371 pins, checked on the real files in Q

`EXPORTS371` names exactly root's four exports, and each full sha starts with the prefix root announced (the sha for 14 matches in full). I read each file with my own code rather than 464's.

| Chapter | File | Rows | Frames | Permit |
|---|---|---|---|---|
| 01 | `…-01-the-bump-manifest392.json` | 8,939 | 6,311 | `01_the_bump-css-permit364.json` |
| 04 | `…-04-reading-color-manifest396.json` | 8,940 | 6,774 | `04_reading_color-css-permit366.json` |
| 12 | `…-12-residues-manifest397.json` | 8,939 | 3,943 | `12_residues-css-permit370.json` |
| 14 | `…-14-continuation-manifest424.json` | 8,939 | 11,756 | `14_continuation-css-permit370.json` |

Every file passes all of these checks:
- It is a regular file, not a symlink, and its sha equals the pin.
- Its `version` is `371`.
- 371's `chapter_of()` gives its row's chapter.
- 371's `manifest_problems()` returns no problems.
- Its pin keys are exactly 371's `PIN_KEYS`.
- Its job, worker, worker sha, inputs and permit equal the draft 460 row's, and that row's worker is 447's `WORKER352_CSS`.
- 448's `manifest_problems()` refuses it, so a 371 export cannot stand in for the gallery.

I also ran 464's own `pinned_exports()` against these files, with Q set to this folder and no gallery given. It returns exactly the four pins as (Q/name, sha), with 05 absent.

## 2. The mandatory gallery arguments and the gallery pin

**The draft's 05 row matches 448's names.**
- Job: `css-reviewed-final-05-gallery352-ssd442`, which is 448's `css_job`.
- Inputs: `RECOVERY/gallery-inputs442.json`, which is 448's `css_inputs`.
- Worker: `css-gallery-ssd442.py` at `ce9baf6a`. The local copy has the same sha.
- Permit: `05_gallery-css-permit375.json`, directly in 448's `PERMITS`.

448's own `pin_problems()` accepts a pin built from this row. So a real 448 export whose pin carries this row's permit passes both 448 and 464 (but see N5).

**`gallery_arguments()`: 38 cases, all as expected.**
- `run` refuses when either argument is missing, and so does `check` when only one is given. `check` with neither returns None.
- Refused names:
  - an empty name;
  - a folder, `..` or an absolute path;
  - a leading dot or dash;
  - a trailing newline;
  - `.JSON`, `.jsonx`, no extension, or `.json` alone;
  - a space, a non-ASCII character or a backslash;
  - more than 205 characters;
  - the four 371 export names, the draft's name and the 464 launch manifest's name.
- Refused shas: upper case, 63 or 65 digits, a trailing newline, non-hex, empty, and each of the four 371 export shas.
- Accepted: a 205-character name, `a..b.json`, and an older launch manifest name. Later checks refuse that last one by version.

**`pinned_exports()` on small synthetic manifests: 20 cases.**
- With no gallery it returns 4 pins; with a valid 448 manifest it returns 5.
- Gallery refusals:
  - the file is missing, is a symlink, or is a folder;
  - the sha is off by one digit;
  - the bytes are not JSON, or the JSON is a list;
  - it is a valid 371-version manifest for 05;
  - it is a 448 manifest for 04, or one with two chapters;
  - its host is `csd` (448's `manifest_problems()`);
  - its pin names another valid permit (376).
- 371 refusals:
  - 01 names another valid permit (999), and 371 itself accepts that manifest, so only 464's row-pin check catches it;
  - 04 names other inputs (371 refuses);
  - 12 is written as a version-448 manifest;
  - 14 is one byte longer than its pin;
  - 01 is missing;
  - the draft rows lack 14.

In every case the refusal comes from the intended check.

**Post-import cross-check: `verify_css_imports()`, with 371's real `q_manifest()` and stubbed checkers.**
- The positive case passes. 01 is checked by 371 and 05 by 448, and each post-import line names its manifest.
- It refuses:
  - an import result naming another file in Q, even with the same bytes;
  - a pinned path whose pinned sha differs from the file;
  - an import result naming a file in a subfolder of Q;
  - an import result naming a symlink in Q.

**`run()` and `check()`.**
- Without gallery arguments, or with an upper-case sha, `run` refuses before it reads any pin, receipt or process.
- A refusal from `pinned_exports()` stops `run` before `master_absent`, `no_duplicates` and any receipt read.
- The start line records the gallery arguments, and the pins include all five exports.
- The `launch464` block of the launch manifest records the five imports and the exact arguments. A 464 manifest written with another gallery sha is refused, not adopted.
- The 393/449/459/461 launch manifests are left byte-identical and are not adopted.
- `check` behaves the same way: a wrong gallery sha prints the refusal, still reads the receipts, and exits 1 with `pins: refused`.

## 3. The gates are unchanged from 461 (AST and replay)

**The AST, compared with docstrings set aside:**
- These are identical: `sha`, `load`, `utc`, `say`, `rows_and_receipts`, `live_guard`, `state`, `one_pass`, `master_absent` and `ancestors`. No 461 function was dropped.
- **`run()`:** exactly five 461 lines are replaced. They are the self-path name, the start line (adds the arguments), the `verify_css_imports` call (adds the exports), the `manifest_bytes` call, and the final note text (350 becomes 447). The only additions are `gallery_arguments`, `pinned_exports` and the two-line export-pin loop. Every other 461 line is kept, in order.
- **`no_duplicates()`:** only the controller tuple, now `CONTROLLERS` (461's tuple plus 464), and the two labels change.
- **`verify_css_imports()`:** 461's lines are kept. The two pinned path and sha asserts are added after `q_manifest()` and `sha(manifest)`, which is where my relay of CC3's pin proposal to root placed them.

**The 461 scenarios, replayed on 464** (stubbed `subprocess.run`, `time.sleep` and a fake `/proc`):
- In the positive case the order is: pins, no_duplicates, waiting, verify (no manifest yet), no_duplicates, the manifest write, no_duplicates, then one launch of `447 launch --manifest Q/master447-launch-manifest464.json --wait-hours 0`.
- The launch manifest's `chapters` equal draft 460's rows.
- With nothing launched and no manifest written:
  - a refusal from the post-import check;
  - a missing import-result file;
  - a master job folder or a duplicate run appearing during the post-import check;
  - the gallery export changing while waiting;
  - a 371 export changing during the post-import check;
  - a pending import (exit 3 at the deadline);
  - a failed SSD render (exit 1);
  - an import that finished with no render receipt (exit 1);
  - the master inputs already existing;
  - a receipt that turns failed during the stable window.
- A receipt rewritten during the stable window, while still successful, is read again: stable, poll and stable sleeps (60 s each), then the launch.
- A 447 launch that exits 1 gives exit 1.

**Duplicates.**
- 464 refuses another `run` of 393, 449, 459, 461 or 464, and any 447 `launch`/`master` or 350 `launch`.
- It allows its own pid, a 464 `check` and an unrelated process.

## Notes

- **N1. `check` without the gallery is not launch readiness.** `check` with neither argument can exit 0 with `pins: ok`, because it checked only the four 371 pins. Its line says the gallery was not given. Root should run `check --gallery-manifest NAME --gallery-sha SHA` before `run`.
- **N2. Run only 464.** On a fake `/proc`, 461's `no_duplicates()` allows a 464 run, so a 461 (or older) run started after 464 would not see it. 464 does see them. CC2's receipt already says this.
- **N3. `main()` refusal mapping.**
  - `OSError`, `ValueError` and `KeyError` now print a `refused` line with exit 1. 461 printed a traceback with the same exit code. A `TypeError` still prints a traceback (exit 1).
  - After the launch, the one mapped case is an `OSError` from `subprocess.run` failing to start 447. Its `refused … launched false` line follows the `launched true` line, and it is accurate because no child started.
  - Once the child has run, the receipt read is already wrapped, so no later line can claim `launched false`.
- **N4. The receipt overstates the sha check.** Receipt line 37 says the sha reservation also covers the draft and the launch manifest. In the code it covers only the four 371 shas. No harm follows: a file at the draft's sha has no 448 version and is refused.
- **N5. Readiness dependencies, which I could not check here.**
  - The real 448 export's pin carries the SSD442 render command's literal `--permit`. 464 accepts it only if that string equals draft 460's `…/repair352-permits/05_gallery-css-permit375.json`. I could not see the SSD442 render command. If the strings differ, 464 refuses before waiting, which fails closed. The repair would then be a new draft, not a change to 464.
  - The manifest must come from 448's own `export`. `css-gallery-export446.py` (`0b26eecc`) writes version `446`, which both relay453 (through 448's `manifest_problems()`) and 464 refuse.
  - `verify_css_imports()` reads the 448 row through 371's `q_manifest()`. That function's text and its Q are the same as 448's, so the result is the same.

## Fixture (scratch file outside the repo)

`test469.py` (`6063311f8bd67c25f741aba2b6c8a154c3c74e1e4027b377a973a79ffbe51ab0`) gives **143/143** in a fresh folder, with output `6a5d6cdb43adc2491f884f88fadf5e4ab7a3683d2379c07bec88f7a24dca3bfe`.

**What it does:**
- It reads the real Q files and writes only in its own new folder.
- The synthetic manifests are small files built through 371's and 448's own `manifest_from_rows()`. I copied no real export.
- `subprocess.run`, `time.sleep`, `/proc` and the checkers' `check()` are stubbed, so nothing can launch or import.

**Two earlier runs, both kept:**
- The first stopped on a fixture bug, `startswith` called on a tuple.
- The second gave 141/143. The two failures were my own expected strings in the AST test: the `ast.unparse` spelling, and a missed fifth replaced line (the note text). The final run corrects only those expectations.

## Not tested

- any CSD run;
- the Linux `/proc` scan itself;
- a real gallery export, which does not exist yet;
- the SSD442 render command's permit;
- the real 371 and 448 `check()` on a real import;
- 447's own launch gates;
- the guard.

## Not claimed

This review accepts no chapter, import or master result. It accepts no picture, motion, sound, sync, caption or AV result, and it authorises no launch or publication. No audio heard or checked.
