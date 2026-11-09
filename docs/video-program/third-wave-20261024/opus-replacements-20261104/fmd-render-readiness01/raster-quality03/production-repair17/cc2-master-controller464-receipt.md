# Controller 464: review and test receipt (CC2)

cc2, 2026-11-09, about 04:30 UTC. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

`cc2-master-controller464.py` is new and derived from 461. It closes CC3 457's gap 1, the named-export pin.

Nothing ran on any host. There was no remote call, launch, import or staging, and no edits to TODO, Beads or memory. 461, draft 460 and every earlier controller are unchanged. The gallery is not exported yet, so 464 contains no gallery digest. Root supplies it.

## Files

| File | sha256 |
|---|---|
| `cc2-master-controller464.py` (new, 469 lines) | `89ff965940c3ed8146f9f849f2cd8736856fb547b0646a85ecb54f113047bfff` |
| `csd-master-controller461.py` (base, unchanged) | `6f5fa07818383f96c74d21c4d014d8c7395ccf892f46c1337e438f2850556d61` |
| `master447-manifest-draft460.json` (pinned, unchanged) | `fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3` |
| 447 / 371 / 448 (pinned, unchanged) | `2b01ecaa…` / `ba337aca…` / `b6653757…`, as in 461 |

## What 464 adds to 461

**Hard pins.** `EXPORTS371` pins root's actual exports by file name in Q and full sha256:

| Chapter | File | sha256 |
|---|---|---|
| 01 | `css352-import371-01-the-bump-manifest392.json` | `633c06d0542f6669eeb42993a3b1b91a1403a752ec259757ef5ab7297a24ac4b` |
| 04 | `css352-import371-04-reading-color-manifest396.json` | `04d1cd99e0114e63abfeaefa23aab7d3ce6e56092d77d8a4459fa3cb2da6896c` |
| 12 | `css352-import371-12-residues-manifest397.json` | `73c62b8d3c92ced7b4f48a9316ce12440b72424ae29ddc6d8b9a8346451a5359` |
| 14 | `css352-import371-14-continuation-manifest424.json` | `79fc3bf9e5c9ffa86f5f30df19a22431acb2b51d0ac620ac259706344caa5206` |

I checked all four locally. Each is version 371, names its chapter, and passes 371's `manifest_problems()` with no problems. Each binds its draft 460 row's job, worker, worker sha, inputs and permit.

**Gallery arguments.**
- `run` requires `--gallery-manifest NAME --gallery-sha SHA`. Root gives them only after a successful 448 export.
- It refuses:
  - either argument missing;
  - a name with a folder, `..`, an absolute path, a leading dot or no `.json` ending;
  - a sha that is not 64 lowercase hex digits (upper case, a prefix or non-hex);
  - a name or sha that is a 371 export's, the draft's or the launch manifest's.
- `check` takes both arguments, or neither. With neither it checks the four 371 pins and reports the gallery as "not given".

**The pin check, before any receipt is read.** Each of the five manifests must:
- be a regular, symlink-free file directly in Q;
- have bytes at its sha;
- be valid JSON;
- carry its importer's version (371, or 448 for the gallery);
- name its row's chapter;
- pass that importer's `manifest_problems()`;
- bind the draft row's job, worker, worker sha, inputs and permit.

**Pins rechecked.** The five manifests join 461's pins, so they are rechecked while waiting, before the post-import check and after it.

**The post-import cross-check.** In `verify_css_imports()`, each `import-result.json` must name exactly its pinned file in Q, and that file must be at its pinned sha. This comes before 461's own binding, rows350 and 371/448 `check` assertions.

**Launch provenance.**
- The launch manifest is the new `master447-launch-manifest464.json`. Its `launch464` block records the five pins (`imports`) and the exact gallery arguments.
- So a run with other gallery arguments refuses an existing launch manifest instead of adopting it.
- The `start` line also prints the arguments and pins.

**Duplicates.** `no_duplicates()` also refuses a running 464 (and 393/449/459/461) `run`, and any 447 or 350 `launch` or `master` process.

**Kept from 461.**
- Every gate, in 461's order:
  1. the stable double read;
  2. the failure and deadline exits (1/3);
  3. the pin rechecks, `master_absent`, `no_duplicates`;
  4. the post-import check before the manifest write;
  5. the byte-for-byte reuse of an existing launch manifest;
  6. one 447 `launch --wait-hours 0`.
- AST-identical to 461: `sha`, `load`, `utc`, `say`, `live_guard`, `state`, `one_pass`, `master_absent` and `ancestors`. `rows_and_receipts` is identical apart from its docstring.

**Small repairs.**
- `main()` now gives a `refused` line, not a traceback, for `OSError`, `ValueError` and `KeyError` (review 395 F2). It also checks the draft sha inside that handler.
- The wording residue of CC0's L4 and H1 is gone: the draft-460 text and the duplicate labels are corrected, and the docstring no longer uses the banned audio-review verb.

## Commands (on CSD, from Q, root only)

```
python3 -I -B <Q>/cc2-master-controller464.py check [--gallery-manifest NAME --gallery-sha SHA]
python3 -I -B <Q>/cc2-master-controller464.py run --wait-hours N --gallery-manifest NAME --gallery-sha SHA
```

- **Before `run`**, CSD Q must hold:
  - 464, 447, draft 460, 371 and 448 at the pins above;
  - the four export files above;
  - the gallery manifest named by `NAME`. That is the output of 448's own `export` on CSS (review 456 F1), staged into Q, and `SHA` is its full sha256.
- **While `run` waits** (up to N hours), it then launches in the foreground, so start it detached from any ssh session.
- **Run only 464.** 461 and older do not detect a 464 run.

## Tests (scratch, outside the repo)

| File | sha256 | Result |
|---|---|---|
| `cc0-test461.py` (CC0's test, copied unchanged) | `b7706e3f109f705241b8a94a0397bc56cbb3325b79f422db91824e6948cea702` | replayed on 461: **40/40** (output `b666eed2…`) |
| `test464-run01.py` (first version, kept) | `944155c92ed0fcb3228e289cb16d51771e6e7d15772565f1864275b670d01bdb` | 110/111. The one FAIL was the harness comparing a 60-character-truncated refusal with full text; the controller refused correctly (output `f1bc6c96…`) |
| `test464.py` | `89c7c3b51113bcdd67806293775058d3a20d9b3f01f6ff8cb724632ac6470da5` | **111/111**, 0 FAIL (output `731243a1…`) |

**Method.**
- `test464.py` is CC0's test461 adapted to 464, using the same stubs: `subprocess.run`, `time.sleep`, the import checkers and a fake `/proc` folder. Nothing can launch or import.
- The four real export manifests are copied into a new scratch Q.
- The gallery manifest there is synthetic. It is a minimal 448-shaped file, and its sha is computed from its own bytes. It is not an export.

**Cases (111).** CC0's 461 cases are marked (CC0); the rest are new for 464.
- **Source (5):**
  - the only 64-hex constants are the four pinned shas and the 447, draft, 371 and 448 pins, so there is no gallery digest;
  - `EXPORTS371` is root's four;
  - no local paths or banned claim words;
  - 9 functions AST-identical to 461;
  - `rows_and_receipts` identical apart from its docstring.
- **Setup (5):** the pins, draft rows, 15 rows with 24 receipts, and the 05 receipt order (CC0); the synthetic gallery manifest passes 448's shape check.
- **Positive run (10):**
  - (CC0) exit 0 with one launch;
  - (CC0) the post-import check runs once with no manifest yet, then `no_duplicates`, the write, `no_duplicates`, the launch;
  - (CC0) the launch command, the rows, the source, the gate text and the manifest bytes;
  - the check receives all five pins;
  - `launch464` binds the five pins and the exact gallery arguments;
  - the start line records them.
- **Post-import refusals (5, CC0):** refuse, missing file (exception), guard changed, master appears, duplicate appears. None writes a manifest or launches.
- **Existing manifests (6):**
  - (CC0) identical is adopted;
  - (CC0) different is refused;
  - (CC0) older 393/449/459/461 are left unchanged and not adopted;
  - a manifest from other gallery arguments is refused and left byte-identical.
- **Regression (4, CC0):** pending gives exit 3 with no check; a failed render gives exit 1; existing master inputs are refused; a 447 exit 1 gives exit 1.
- **Gallery arguments (34):** 17 refusals, each also shown to stop before any receipt pass, with no manifest or launch:
  - 3 missing combinations;
  - 6 bad names: folder, `..`, absolute, leading dot, not `.json`, empty;
  - 4 bad shas: upper case, 63 digits, 12-digit prefix, non-hex;
  - 4 reserved: a 371 export's name, a 371 export's sha, the launch manifest's name, the draft's name.
- **Gallery file (10):** refused for each of these:
  - absent from Q;
  - a symlink;
  - a digest altered by one hex digit;
  - bytes changed after the sha was taken;
  - not JSON;
  - version 371;
  - version 446;
  - chapter 01;
  - a pin naming the ssd440 job;
  - a permit other than the draft row's.
- **371 pins (4):** refused for each of these:
  - 392 missing;
  - 396 changed by one byte;
  - 397 present only under another name;
  - 424 a symlink to the right bytes.
- **Rechecks (4):**
  - the gallery file changes while waiting: refused;
  - it changes during the post-import check: refused;
  - restored, the run launches again;
  - with byte-identical launch-manifest bytes.
- **Fake `/proc` (11):** refused: another `run` of 393, 449, 459, 461 or 464, and a 447 `launch` or `master` or a 350 `launch`. Allowed: own pid, a 464 `check`, an unrelated process. The 464 additions are new; the rest is CC0's.
- **Real `verify_css_imports` with stub checkers (6):**
  - pinned exports pass;
  - 01 is checked by 371 and 05 by 448 (CC0's wiring);
  - refused: an import result naming another file in Q, even one bound and at its hash;
  - refused: the pinned name with other bytes, in a self-consistent import;
  - refused: an import result naming another manifest sha;
  - with no gallery pin it cannot pass.
- **`check`, `main`, frozen files (7):**
  - `check` with no gallery arguments returns 0, with "not given";
  - `check` with valid arguments returns 0, with "given";
  - `check` with one argument is refused;
  - `check` with 397 missing exits 1 but still shows the receipts;
  - `main` with one gallery argument prints a `refused` line;
  - `main` `run` prints a `refused` line (locally the CSD-only gate stops it first);
  - 461, draft 460, 447, 371 and 448 are unchanged afterwards.

**Not tested:**
- any CSD run;
- real 371 or 448 `check()` on a real import;
- the Linux `/proc` scan itself;
- 447's full gates;
- a real gallery export, which does not exist yet.

## Not claimed

No import, master or chapter result. No picture, motion, audio, sync, caption or independent AV acceptance. No publication authorization.
