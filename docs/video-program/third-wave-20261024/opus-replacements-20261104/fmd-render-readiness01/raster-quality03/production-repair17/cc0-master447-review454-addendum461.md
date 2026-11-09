# Review 454 addendum: controllers 459 and 461, draft 460 (CC0)

cc0, 2026-11-09. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

This addendum follows review 454 (`cc0-master447-review454.md`, sha `bbbcf330…`). Root has read 454, so it stays unchanged. As root asked, **461 with draft 460 is the final target**. The 459 replay is kept as its own fixture.

Nothing ran on any host: no ssh, render, import, launch or master. Every check below ran locally, on read-only files and small synthetic fixtures. None of it is CSD evidence, and none of it is picture, motion, sound, sync or AV acceptance. No audio heard or checked.

## Verdict

- **No High or Medium finding on 461 and draft 460.**
- **Review 454 findings:**
  - **L1** (manifest before post-import check) is fixed in 459 and kept in 461;
  - **L2** (launch-manifest provenance) is fixed in 461;
  - **L3** (stale draft block) is fixed in draft 460;
  - **L4** (stale wording) is partly fixed. The residue is listed below. 447's own docstrings stay a Low by root's decision.
- **Unchanged from review 454:**
  - the 15 rows (draft 460's equal draft 449's);
  - 447 (`2b01ecaa`) and the 05 connection;
  - the gates. 461 changes none: it differs from 459 only in names and wording (section 2).
- **One new Low, H1:** a banned public-repo word in 461 and in draft 460.

## 1. Files at review time (hashes re-read just before handoff)

| File | sha256 |
|---|---|
| `csd-master-controller461.py` (final target) | `6f5fa07818383f96c74d21c4d014d8c7395ccf892f46c1337e438f2850556d61` |
| `master447-manifest-draft460.json` (final target) | `fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3` |
| `csd-master-controller459.py` | `bc53e8e5bd7808a117c8e1d49d84907cf964bb22b1a4e9c4f3184a642ecb1c37` |
| `csd-master-controller449.py` (unchanged) | `3e8d5fa215399b590a27ce95591c6df7ff3d05f8667c419b5b5ac8247ecfb38b` |
| `master447-manifest-draft449.json` (unchanged) | `0108537698028e0b7389533f85c552c652e9b7aef72f86b6028ae209870be5aa` |
| `combined-master447.py` (unchanged) | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` |
| `csd-gallery-import448.py` (unchanged) | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` |
| `csd-css352-import371.py` (unchanged) | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` |

## 2. What changed

**449 → 459** (`diff`):
- a new module docstring;
- `VERSION` is `459`; the launch manifest is `master447-launch-manifest459.json`; the self-path is `csd-master-controller459.py`;
- `no_duplicates` also matches 459 runs;
- the gate text names 459 and 447;
- **in `run()`**, `verify_css_imports()`, then the pin recheck, then `master_absent()` and `no_duplicates()` now run before the manifest is written. Root's order check gives line 329 for the post-import check and line 338 for the manifest `open('xb')`.

In 459 the manifest block key is still `launch393` and its `rows` text still says "Frozen 350".

**459 → 461** (`diff`):
- the old 393 docstring is gone; the module docstring is new;
- `VERSION` is `461`; `DRAFT` is draft 460 at `fcf0e7d7`; the launch manifest is `master447-launch-manifest461.json`; the self-path is `csd-master-controller461.py`;
- `no_duplicates` also matches 461 runs;
- the block key is `launch461`, the `rows` text says "Frozen 447", and the gate text names 461 and 447.

Nothing else differs. `rows_and_receipts`, `one_pass`, `verify_css_imports`, the waiting loop and the launch command are byte-identical to 459's.

**Draft 449 → 460:**
- `chapters` are equal;
- the `draft384` block is replaced by a `draft460` block, which says it supersedes draft 449 at `01085376`.

Its bindings agree with its rows:
- 05 = the 442 worker with the 448 a1 import;
- 01/04/12/14 = the CSS 352 worker with 371 a1 imports;
- 06/07/08/10/11 = the CSD 352 worker;
- 09 = 344;
- 02/03/13/15 retained;
- "133662 frames" equals 447's `TOTAL_FRAMES`.

The block claims no result.

## 3. Replays

The fixtures are scratch files outside the repo. Each runs in a new scratch folder, which is kept. They stub `subprocess.run`, `time.sleep` and the import checkers, and give `no_duplicates()` a fake `/proc` folder. Nothing can launch or import.

| Fixture | sha256 | Target | Result |
|---|---|---|---|
| `test449.py` (kept unchanged; not re-run, since 449 is unchanged) | `3b4e9fc6…` | 449 | 21/21 (review 454) |
| `check449.py` (kept unchanged) | `3d81e429…` | 447 + draft 449 rows | 43/43 (review 454) |
| `test459.py` | `b5ab83b61c65a5df3dfd2a78a96349b96fe96b2b7af7c3f59367069083dc409d` | 459 | **36/36** (output sha `485e571d…`) |
| `test461.py` (a copy of test459 with 461's constants and 3 more checks) | `b7706e3f109f705241b8a94a0397bc56cbb3325b79f422db91824e6948cea702` | 461 | **40/40** (output sha `ed3a5e42…`) |

Each case below passes on both 459 and 461 unless marked.

**Pins and rows:**
- the controller's own sha;
- the pins: 447, the draft, 371, 448 and the manifest name;
- the draft rows equal draft 449's;
- 15 rows, 24 receipts;
- 05 waits on the 448 a1 import, then the SSD442 render.

**Positive run:**
- exit 0 with one launch: `447 launch --manifest Q/master447-launch-manifest<v>.json --wait-hours 0`;
- `verify_css_imports` runs once, on 15 rows, while no manifest exists;
- the event order from the check onward is exactly: check (no manifest), `no_duplicates`, manifest write, `no_duplicates`, launch;
- the manifest's rows equal the draft's;
- its `source` is the pinned draft;
- its bytes equal `manifest_bytes()`.

**No manifest and no launch when the post-import stage refuses:**
- the import check refuses (AssertionError): a `refused` line;
- `import-result.json` is missing (FileNotFoundError). This ends in a traceback, as 454 N5 said, and still writes nothing;
- the guard changes during the check: "changed during post-import checks";
- the master job folder appears during the check: `master_absent` refuses;
- another controller run appears during the check: `no_duplicates` refuses.

**Existing manifests:**
- an identical one is adopted and the run launches;
- a different one is refused, after the import check and before any launch;
- older 393/449 launch manifests (and 459's, in the 461 replay) stay byte-identical and are not adopted.

**Regression:**
- a pending 05 import gives exit 3, with no import check and no manifest;
- a failed SSD render gives exit 1, with no manifest;
- existing master inputs are refused;
- when 447 exits 1, the controller exits 1 and the manifest stays.

**Fake `/proc`:**
- refused: another `run` of 393, 449 or 459, and of 461 too in the 461 replay;
- refused: a 447 `launch` or `master` process, and a 350 `launch`;
- allowed: the controller's own process, a `check`, and an unrelated process.

In the 459 replay a 461 run is not detected (info). 461 detects all four controllers.

**Wiring:** `verify_css_imports` checks 01 with 371 and 05 with 448, both at attempt 1.

**461 only:**
- the block key is `launch461`, the `rows` text names 447, and the block names neither 350 nor 393;
- draft 460's keys and `supersedes` entry;
- draft 460's bindings text against its rows and 447's `TOTAL_FRAMES`.

## 4. Findings and notes

### H1 (Low, public-repo hygiene). A banned word appears in two root files.

- **461 line 4** and **draft 460 line 196** each contain one word on the public-repo banned list. It is the audio-review verb, found in "visual, … or release acceptance" and "picture/…/fine-sync".
- CC0's banned-word check (`compile350.py`) flags both lines. It finds nothing in 459.
- Fix: write "audio" or "sound check" in a successor. The behaviour is unaffected.

### L4 residue (Low, wording; no behaviour)

In 461:
- `:63` the `rows_and_receipts` docstring still says "Draft 384's rows, checked against 350's row shapes";
- `:332` the refusal says "not draft 384";
- `:227` the duplicate label is "controller393 run" for all four controllers;
- `:229` the label is "350" for a 447 process;
- `:142`, `:144` and `:319` say "350 success()" and "350 launch". 447's `success()` is 350's code unchanged, so this is wording only.

447's own docstrings stay a Low, as root decided.

### Notes (no change needed)

- **N6. A short window remains after the write.** `master_absent()`, `no_duplicates()` and the re-read assertion can still refuse after the manifest is written. The manifest would then stay in Q and a later run would adopt it byte for byte. That manifest exists only after a full post-import pass, so this matches root 390's order.
- **N7. Run only 461.** 459 and 449 do not detect a 461 run; 461 detects all of them. Even if two controllers raced, 447's launch asserts that the master inputs are absent and creates them with `open('x')` (:1216, :1244), so a second 447 launch stops there.

## 5. Preconditions for root (from review 454, updated)

- **P1.** The CSD alias `/data/ssd/work/jmev-gallery-recovery440 → D/work/imported-css-ssd442` must be established before 448 runs.
- **P2.** 448's closure must carry every row of `gallery-inputs442.json`, because 447 re-hashes each one.
- **P3.** CSD Q must hold 447, **461**, **draft 460**, 448 and 371 at the pins above. 393, 449 and 459 stay preserved and are not run.

## Not tested

- any CSD run;
- 448's or 371's real `check()` on a real import;
- the Linux `/proc` scan itself, since the fake folder only feeds `no_duplicates()` its entries;
- 447's full `chapter_gate`, which needs CSD files.
