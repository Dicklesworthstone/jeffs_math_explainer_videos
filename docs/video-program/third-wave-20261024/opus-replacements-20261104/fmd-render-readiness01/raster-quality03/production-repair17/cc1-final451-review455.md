# Review 455: verifier 451 and controller 452 (CC1)

This is an independent review of `csd-final-verify451.py` and `csd-final-controller452.py`. I read them against:

- verifier 377 and controller 388;
- the exact master 447;
- the SSD442 gallery worker.

Every file reviewed is prepared only. Nothing was launched or staged, and no permit was issued. This task made no contact with CSD or CSS. Root reports that CSD command channels time out, with outcomes UNKNOWN.

All evidence is source reading plus synthetic runs on this Mac. It is not final AV acceptance, picture review or audio review, and **all final AV acceptance stays open**. Narration stays ElevenLabs Frederick Surrey, unchanged. Written 2026-11-09 (UTC).

## Verdict

- **No High or Medium finding.**
- **Two Lows, both wording. Behaviour is unaffected.**
  - **L1:** 452's docstring names the wrong helper, manifest and notes.
  - **L2:** 451's `master350()` still prints 350/344/352 in its refusals.
- **Positive mapping holds on the actual rows.** For all 15 rows of draft 449, 451 reads the same source job as 447. Draft 460 has the same rows. The gallery maps to the SSD442 job on the 442 worker.
- **451's whole `master350()` accepts a tree built the way 447 builds it.** Each of 8 mutated trees is refused at the line meant to catch it.
- **451 = 377 except `master350()`.** All other functions are AST-equal to 377's. That covers the frame, caption, plan and picture-part guards.
- **452 = 388 after renames.** The module AST is equal to 388's. The 31 scenario runs on root's guard runner give outcomes identical to review 391's.
- **Info notes:**
  - **I1:** 451 is stricter than 447's row mapping on the gallery.
  - **I2:** 451 does not re-derive which soundtrack capture 447 staged; this is unchanged from 377.
  - **I3:** the SSD alias on CSD is a precondition.
  - **I4:** what root's checks cover.
  - **I5:** the guard hash list.

## 1. Files (re-hashed at review time)

| File | sha256 |
|---|---|
| `csd-final-verify451.py` | `c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03` |
| `csd-final-controller452.py` | `beb4ea0650f577d3da70731bb5a01417c2d3550b7c84eb66aa77c3335d1fceae` |
| `combined-master447.py` | `2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524` |
| `css-gallery-ssd442.py` (Q copy, equal to 451's `WORKER442_SHA`) | `ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429` |
| `csd-final-verify377.py` | `827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe` |
| `csd-final-controller388.py` | `5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e` |
| `master447-manifest-draft449.json` | `0108537698028e0b7389533f85c552c652e9b7aef72f86b6028ae209870be5aa` |
| `master447-manifest-draft460.json` | `fcf0e7d738c5bbe142f3a938a7526f7847cdb16fe006eed2ba53596f5a6525b3` |
| `final451-controller452-root-checks.json` | `f866fc02c422555493bcbce87617bcb91b1a105f0b7935a9d73ce1cf405a74ed` |
| root guard runner (`run_bounded_phase_oliver_owner48g_target14g_20261102.py`) | `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da` |

Other files read:

- 448 (`b6653757`);
- the 440 worker (`9c80ceb6`);
- 350's helper (`60575944`);
- the 344 and 352 workers at 451's pins.

## 2. What 451 and 452 change

### 451 against 377

Evidence: `ast455.py`, `ast455b.py`.

- **What is unchanged.** 377 has 125 top-level nodes, and 115 of them are AST-equal in 451. That includes every function except `master350()`, and the `__main__` block. 377's docstring survives as 451's second expression.
- **Changed constants.** `VERSION` now reads `'451'`. The master pins move to 447: helper `combined-master447.py` at `2b01ecaa`, job `csd-combined-master447`, inputs `combined-master447-inputs.json`. The verify and picture job and inputs names move to 451.
- **New code** (:394–415):
  - the SSD442 root, `WORKER442`, `WORKER442_SHA` and `GALLERY442_JOB`;
  - `expected_worker451()` (:400);
  - `source_job451()` (:410).
- **`master350()` changes in exactly three lines:**
  - :432 expects summary version `'447'`;
  - :447 takes worker, sha and job from `expected_worker451()`;
  - :481 takes the source job from `source_job451()`.

The frame, caption, plan and picture-part guards are therefore 377's code, unchanged at AST level. That code is:

- `verify()`: the strict video check, and the decoded frames against the staged chapters;
- the caption checks;
- `plan_for()`, `plan_digest()`, `part_upper()`, `part_done()`;
- `launch()` with its never-duplicate, free-space and CPU refusals.

What feeds those guards is the dictionary `master350()` returns. §3.2 exercises that return positively.

### 452 against 388

Map 452's names back to 388's in six counted replacements:

1. the verifier hash;
2. the ledger name;
3. the docstring's `cc0-combined-master447.py`;
4. 452→388;
5. 451→377;
6. 447→350.

After that, 452's whole module AST equals 388's. The only remaining text difference is the comment on `VERIFIER_SHA`. 452 has no logic change.

## 3. Positive mapping tests

### 3.1 The actual manifest rows (`mapping455.py`)

447 and 451 are loaded unchanged and hash-checked. The AST shows that their top levels only build Paths, dicts, tuples and regexes, so loading them does no I/O.

For each of the 15 draft-449 rows:

- 447's host rule and its own `combined_source447()` give the source job that 447's `chapter_gate()` reads.
- 451's `expected_worker451()` and `source_job451()`, applied as `master350()` applies them, give a source job too.
- The two are **equal for 15 of 15 rows**.

The gallery maps to `SSD442/jobs/css-reviewed-final-05-gallery352-ssd442` on the 442 worker `ce9baf6a`, with its host recorded as CSS. Draft 460 has the same 15 rows field for field; only its top-level status key differs. So the result holds for draft 460 as well.

Nine mutated records were run through 451. **All nine are refused.** The last column shows what 447's own row mapping does with the same mutation:

| Mutation | 451 | 447 row mapping |
|---|---|---|
| gallery host recorded as D | refused | passes |
| gallery on the old CSS 352 worker and its failed job | refused | passes |
| gallery on the CSD 352 worker | refused | passes |
| gallery job of the failed 440 attempt | refused | refuses |
| gallery worker sha of 440 | refused | refuses |
| gallery kind `retained` | refused | refuses |
| SSD442 worker on 04 | refused | refuses |
| SSD442 worker on 06 (CSD row) | refused | refuses |
| SSD442 job name on 04 | refused | refuses |

### 3.2 451's whole `master350()` on a tree shaped like 447's output (`master350_455*.py`)

**How the tree is built:**

- **451 itself.** 451 runs with only its three host-root literals rewritten to scratch roots. Reversing the rewrite reproduces `c876caf6` byte for byte.
- **Host roots.** The CSS root and the SSD442 root are symlinks into D/work (`imported-css-root311`, `imported-css-ssd442`), as 371 and 448 make them on CSD. The retained hosts (TRJ, HZ2) are plain folders.
- **Real bytes at the pinned paths:** 447, the 344 boundary worker, both 352 workers, and 442.
- **The master's records follow 447's own writers:**
  - `chapter_gate()`'s record (:1105–1119);
  - `master()`'s staged-inputs hard links and summary (:1302–1373);
  - `launch()`'s guard command (:1246–1248). The receipt command is the child argv, 11 items, because the runner strips `--`.
- **Render summaries.** Each one lists its files by the job's own lexical render path, as 442's `render()` does (:904–913).
- **Size.** Chapters are 12 frames long, with 9,600-sample WAVs.

**The positive tree is accepted:**

- **Gallery source.** `sources[OUT/05_gallery.mp4]` is the SSD442 lexical path, and it resolves into D through the alias.
- **Bound files.** The SSD job's `render-summary.json` and receipt are in the file set that the verify job binds.
- **Intervals and playback.** Intervals exist for the 11 non-retained chapters. Playback exists only for 09 (`[0.1]`).
- **Staged links.** There are 90 = 15 × 6 staged links. By root: D 36, CSS 30, SSD442 6, TRJ 12, HZ2 6. In total 242 files are bound.

**Negatives**, each built in a fresh tree. Every one is refused, at the 451 line shown (`master350_455b.out.json`):

| Variant | Refused at |
|---|---|
| gallery host D | :402 `Path(row['host']) == CSS` |
| gallery on the old CSS worker and job | :403 worker/sha/job triple |
| gallery job `-ssd440` | :403 |
| SSD442 worker on 04 | :405 "SSD442 worker belongs only to gallery" |
| gallery render summary lists CSS-root paths | :505 render-summary listing |
| gallery render summary frames off by one | :484 chapter/frames/binary |
| gallery staged through the CSS root (same inodes, other lexical path) | :482 `movie == job/'render'/…` |
| summary version `'350'` | :432 |

### 3.3 452 on root's guard runner (`test455.py`)

**The harness:**

- `make455.py` derives it from the review-391 harness by counted replacements; the counts are in `make455.out.json`.
- `fake451_gen455.py` builds the stand-in `fake451-455c.py` from 451's own source. It copies verbatim `sha`, `read`, `success`, `stable`, `growth_upper`, `part_paths`, `plan_digest`, `part_upper`, `part_done`, `verified`, `launch` and `main`, plus the closing prints of `verify()` and `pictures()`.
- Root's runner `819746fb` runs under `fakeguard455.py` with its code unchanged.
- 452 runs unchanged, hash-checked by `run452_455.py`.

**The runs.** There are 31 scenario entries:

- 26 scenarios;
- the S02 and S14b reruns;
- S20 with two controllers on one tree;
- the three S21 bounds.

**All 31 match review 391's 388 outcomes** (`compare455.out.json`), once names, scratch paths, hashes and times are normalized.

- **Happy path.** It runs check → launch-verify → launch-pictures (overview) → ledger → launch-pictures (native). The ledger then equals three things: the overview `sampling.json`, the native inputs row, and the native union.
- **Every failure stops the chain with nothing further launched:**
  - a missing, running, failed, refused, resource-stopped or rewritten master;
  - a pin change while waiting;
  - any of the seven 451/452 paths already present;
  - a check refusal, a verify failure, a guard refusal or a free-space refusal;
  - an overview stop, an extra last line, or a changed `sampling.json` before or after the ledger;
  - a native failure;
  - the second of two controllers;
  - out-of-range `--wait-hours`.

So review 391's ledger, ordering and failure/duplicate results carry over to 452 unchanged. Two of its Lows apply equally:

- **its L2:** the lane-lock wait is unbounded, and a changed input spends the job name;
- **its L3:** copy the ledger hash off CSD at once.

## 4. Findings

### L1 (Low, wording). 452's docstring names the wrong helper, manifest and notes.

452:3–5 says the master job is "cc0-combined-master447.py, launched by root from CC0's manifest 384". It also says 451 runs "in the run order of its notes (451 notes section 4 …)". In fact:

- The helper that 451 pins is `combined-master447.py` (`2b01ecaa`), with no `cc0-` prefix.
- The manifest is draft 449/460.
- No 451 notes exist in Q. The run order is 377 notes §4 (`cc2-final-verify377-notes.md`).

The code is right. `load()` asserts `MASTER == D/jobs/csd-combined-master447` and `MASTER_INPUTS == Q/combined-master447-inputs.json` (452:59–60).

Fix in any successor: name `combined-master447.py`, the launched manifest, and 377 notes §4.

### L2 (Low, wording). 451's `master350()` still speaks of 350, 344 and 352.

- **Refusal texts:**
  - :427 "not frozen master350";
  - :436 "not the manifest 350 bound";
  - :445 "the review basis 350 records";
  - :449 "not the frozen 344/352 worker and job";
  - :489 "not the permit 350 bound";
  - :492 "350 recorded other sampled intervals".
- **Comments:** :422, :443 and :486–487.
- **The :480 comment** says "source = host/'jobs'/job", which is no longer true for the gallery.
- **The function** is still named `master350()`.

These are diagnostics only. For the gallery, :449 repeats the :403 equality, so it cannot fire once :403 has passed. Fixing this is root's call; it can wait for a successor.

### I1 (Info, fail-closed). 451 is stricter than 447 on the gallery.

447's row mapping lets three gallery mutations through (§3.1), and 451 refuses each one. A master built from a manifest that sends the gallery anywhere but the SSD442 job on the 442 worker cannot pass 451.

### I2 (Info, unchanged from 377). 451 does not re-derive which capture 447 staged.

447's `chapter_gate()` stages the last `soundtrack_N.wav` of a render (:1095–1097). 451 checks three things about the staged capture:

- it is a hard link of a bound row;
- the render summary lists it at that hash;
- it has 48 kHz, 2 channels, 16 bits and frames × 800 samples.

451 does not check that it is the last capture. `master350_455c.py` gives the gallery render two valid captures. 451 accepts a master staging `soundtrack_1` (447's choice), and **also** one staging `soundtrack_0`.

The pinned 447 always stages the last capture, so this matters only if 451 were meant to check 447's choice independently. I did not read the live 442 job, so I have not checked how many captures it holds.

### I3 (Info, precondition). The SSD alias must exist on CSD before 451 runs.

`master350()` reads the gallery through the SSD442 lexical path (:413, :481), and on CSD that path exists only through 448's alias. Without the alias, `stable()` fails on the SSD rows. `check` then refuses, and 452 launches nothing (S11 shape).

My tree models the alias as 448 makes it: a symlink into D/work on the jobs device. I did not observe the alias on CSD.

### I4 (Info). What root's checks cover.

`final451-controller452-root-checks.json` binds:

- the current 452 `beb4ea06` (the initial controller was `23601f6d`);
- 451 `c876caf6`.

Its nine checks:

- one exact gallery mapping;
- six refusals in `expected_worker451()`/`source_job451()`;
- two unchanged-source checks.

Its own stated scope is "Root source and mapping checks only". This review adds:

- the actual-row mapping against 447's own functions;
- the full positive `master350()` with in-context negatives;
- AST equivalence;
- the 452 runs.

All of that is still synthetic.

### I5 (Info). Hash list for 452.

452 has no notes file. Whatever pre-run hash list root uses for 452 should also carry `guard_csd02.py` `f1cae1db` (review 391's L4), beside 451 `c876caf6`, 452 `beb4ea06` and 447 `2b01ecaa`.

## 5. Limits

- **No host contact.** This task contacted neither CSD nor CSS. The CSD state is unknown: the alias, the 448 import, the master job, and whether any 451 job folder exists.
- **The trees are synthetic.**
  - The mp4/srt/vtt files are stub bytes, and the WAVs are short, mostly silent ones.
  - No ffprobe, decode or x264 check ran. Those belong to 447 and to 451's `verify()`.
  - 451's `verify()`, captions, `plan_for()` and `pictures()` did not run here on media. Their code is 377's (§2), and the earlier evidence on it (CC0 review 378 of 377) was not re-run.
  - The 452 harness stubs them exactly as review 391 did.
- **The runtime differs from CSD.** I ran Python 3.15 on macOS, not CSD's `/usr/bin/python3`. The guard runs with test-only admission and stop thresholds (0 GiB) and an `sbh` stub, as in review 391.
- **Not claimed:** compiling, rendering, motion, 4K or pixel equivalence, audio review, picture review.
- **Root's checks** are not independent acceptance. This review is independent code, but its data is synthetic.

## 6. Fixtures

They are in `cc1-final451-review455-fixtures/`; `MANIFEST455.json` is `354fad9d14dd408d57f8700f4e2375488a7fd0db0bb012a949950747c86a5cdc`. Scratch paths in the outputs are replaced by `T`, and the repo root by `REPO`.

| File | sha256 (prefix) | Role |
|---|---|---|
| `mapping455.py` / `.out.json` | `e59bfebacd2c` / `ac10584d5039` | §3.1: 15 actual rows and 9 mutations |
| `master350_455.py` / `.out.json` | `59a53fa43719` / `322346528112` | §3.2: positive tree and 8 negatives |
| `master350_455b.py` / `.out.json` | `6af0f0b9f0c5` / `45d2ba2b8f37` | the same runs, plus the refusing 451 line |
| `master350_455c.py` / `.out.json` | `5625c8e6a994` / `a45e115a03ce` | I2: two captures |
| `ast455.py` / `.out.json` | `ec04378f0ec6` / `6af8020bff08` | §2: top-level nodes and text diffs |
| `ast455b.py` / `.out.json` | `6fdce56fa4b7` / `04561a05dee7` | §2: main block, docstring, 452 renamed AST |
| `make455.py` / `.out.json` | `d9d7746e499a` / `7a1f0247b7ad` | derives the 452 harness from review 391's |
| `fake451_gen455.py`, `fake451-455c.py` | `ca7652c7871f`, `a1f67abfa8e7` | the 451 stand-in (generator and output) |
| `run452_455.py`, `fakeguard455.py`, `test455.py` / `.out.json` | `466cd6ee4c67`, `0952bcffef22`, `9c6ba8571546` / `9a66a1c92a38` | §3.3: 31 runs |
| `compare455.py` / `.out.json` | `3e48052dbcb0` / `9f9efd501982` | §3.3 against review 391 |

**To rerun,** from the repo root:

- `python3 -I -B mapping455.py Q`
- `python3 -I -B master350_455.py Q NEWDIR` (also `_455b.py` and `_455c.py`)
- `python3 -I -B ast455.py Q` and `python3 -I -B ast455b.py Q`
- For §3.3:
  1. `python3 -I -B make455.py DIR391 NEWDIR`, where DIR391 holds review 391's harness files;
  2. `python3 -I -B fake451_gen455.py Q/csd-final-verify451.py NEWDIR/fake451-455c.py`;
  3. `python3 -I -B NEWDIR/test455.py Q/csd-final-controller452.py RUNNER`;
  4. `compare455.py`. It read the local `test391.out.json`; the cleaned copy is `results391-summary.json` → `harness` in review 391's fixtures.
