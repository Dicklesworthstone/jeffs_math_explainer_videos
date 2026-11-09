# CC1 master review 322

Reviewer: CC1, independent of root and CC0. Date: 2026-11-08. Read-only review.

I did not run 318, its preflight, the delivery contract, any binary or the guard, and I reached no remote host. Nothing was rendered, mastered or launched. No source, helper, TODO or Beads was edited. Paths use 318's own names: D, Q, WORKSPACE268, REPO07, NARRATION (rebind269), NARRATION313 (rebind313) and STEMS.

## Verdict

**No High or Medium defects. One Low finding, F1.**

F1: the kit's own caption build and validation is not preflighted, and 318 runs it only after the master is written.

These parts of 318 hold against the actual sources:
- **Preflight.** It reproduces the kit's own call to the delivery contract for both consumers: interpreter, flags, script, argv, chapter order, tables, cwd, and an environment that makes no difference.
- **N7 closure.** It is complete. Every file the contract opens for `master` and `captions` is frozen. I checked this read site by read site in the contract source, not against a mock.
- **Mixed 09.** Binary 268 consumes the 09 tables and WAVs by hash only. `replays_of` uses the kit's own line format and counting rules.
- **Retained 02/03/13/15.** These rows use 293's code unchanged, which review 316 accepted. The preflight is the first run of the contract on their staged tables.

Listening, fine sync and final AV acceptance remain open (see Still open).

## Files reviewed

| File | sha256 | Read |
|---|---|---|
| `Q/cc0-combined-master318.py` | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | all 950 lines |
| `Q/cc0-master318-notes.md` | `423132ff11f827ba9ff986c9ec02e7190efbfa4ce8805c2a2718bbecedc22a0b` | all 198 lines |
| `Q/cc0-combined-master293.py`, the base | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` | the 420 changed lines of the diff |
| `Q/csd-reviewed-boundary-final317.py` | `19abeee93dbdb98a0217c69b005f05fb18daff716c10406c67ed36fb7210b036` | all 252 lines |
| `Q/cc2-master-review307.md` | `dcf5e307ee94956318d9859f9aee91fe570be3c367f8c88b3e4a43783b41e837` | L1, L2, N4–N7 |
| `Q/cc3-import-review316.md` | `7cc987d4af127ecbd2e5c871838759db921efc2ff7304ed0e60de27436ed8523` | all |

**Delivery contract, as build268 compiled it.** I read reviewer-local copies of the build268 workspace tools. Each hash equals the entry in `Q/review-build268-terminal272/source-copy.json` (`81d28272…`, which is 318's `SOURCE_COPY_268_SHA`):

| File | sha256 |
|---|---|
| `tools/narration_delivery.py` | `961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878` |
| `tools/coherent_narration.py` | `a7fc6705aead7999134d8619458c8b2973ef771be069d4ce98462ac5f97fc42f` |
| `tools/narration_cache.py` | `47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0` |
| `explainer_corpus/complex_analysis_visual/src/main.rs` | `19a9e1051360caacba25a9276d1f32a4b7b94700bb25a4c72ca663805619f798` |
| `explainer_corpus/complex_analysis_visual/script.tsv` | `234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c` |

The repository's own `tools/coherent_narration.py` (`95615cfa…`) is not the build268 file. It equals build268's `coherent_narration.before-production19.py`.

**Kit.**
- `cli.rs`: I read the repository copy (`a63970e0…`). Its `check_delivery_contract`, `captions_video` and `master_video` are identical, function for function, to CSD's frozen repo-view copy `Q/cross-host-master-source50/0-cli.rs` (`3c5221febc3e3d92d37aa97e2f8d38c319121215a67a1dc68d24534d41d862c9`).
- `master.rs`: `7d1952db…`, the same in both trees.
- `captions.rs`: `28e1aa51…`, from the repository only.
- Line numbers below are the repository's.

**Records.**

| File | sha256 (prefix) | Used for |
|---|---|---|
| `Q/review-build268-terminal272/source-copy.json` | `81d28272` | contract files, `main.rs` |
| `Q/review-build268-terminal272/clippy.stderr` | `ba8ace82` | crate location |
| `Q/master-caption-contract305.json` | `dc24242b` | root's first preflight |
| `Q/master-caption-contract305b.json` | `1bc05010` | root's second preflight |
| `Q/score21-terminal01.json` | `746ca65c` | stem files |
| `Q/final-picture-source41/*.timeline.tsv` | — | record kinds and levels only, not the staged bytes |

## Finding

### F1 (Low): the kit's own caption build and validation is not preflighted and runs after the master is written

**Where.** 318 `master()` runs `master`, then `captions all` (:886–893). `preflight()` (:738–775) runs only the Python delivery contract.

**What the kit checks that the preflight does not.**
- `captions_video` (`cli.rs:892`) builds the captions with `captions::video_captions` (`captions.rs:541`).
  - That reads each line's WAV (`:553`).
  - It refuses when the recorded end and the WAV length differ, or when no speech is found.
- `captions::validate` (`captions.rs:590`) then refuses on any of:
  - start not before end;
  - line count;
  - line length;
  - overlap with the previous caption (`:607`).

  A refusal makes the kit exit 1 (`cli.rs:916–919`).
- None of this is in `narration_delivery.py`. For `captions` it runs only `timeline()` (`:322–325`). Directed caption segments are computed only for `master` (`:328–340`), and the kit reads only the contract's exit status (`cli.rs:468–470`).

**No earlier run covers it.** Q has no record of binary 268's native `captions` on these tables. Root's 305 and 305b are Python-only by their own scope ("no native master or caption export").

**Cost.** A caption refusal comes after `master_video` has written:
- the master WAV (`cli.rs:993–994`);
- the join copies and the joined MP4 (`:997`).

307 N6 puts that at about 1.2–1.7 GB. `check=True` (:892–893) then fails the job, and 318 is spent. Nothing wrong can be accepted this way; the cost is the version and the disk.

**Notes gap.** Notes §5, "What still spends 318", names only guard admission and a null-test failure. It leaves out three later points:
- this caption refusal;
- the kit master's own post-join gates (`cli.rs:1004–1012`);
- 318's post-run ffprobe and re-hash asserts (:905–919).

**Fix (root's choice).**
- **(a) Smallest change: run `captions all` before `master`** by swapping the list at :886.
  - In the source, `captions_video` reads only the `--out` timelines, the report and the line WAVs, and it writes its two text files only after `validate` passes.
  - `master_video` reads no caption output.
  - A caption refusal then spends only the job name, with nothing written in `final/` beyond the staged links.
- **(b) Stronger: add binary 268's `captions all --narration NARRATION --out <preflight folder>` to the preflight.**
  - It renders nothing. It writes only `complex_analysis_visual.en.srt` and `.vtt` into that folder, and the kit runs its own contract there first (`cli.rs:356`), as in the job.
  - A refusal then spends nothing.
  - It is a native binary run outside the guard, which 318 deliberately avoids ("No native binary runs"), so root decides.

## What I checked and found sound

### 1. The preflight against the kit's own contract call (307 L1)

| Item | Kit (`cli.rs:390–471`, run from 318 `master()`) | 318 `preflight()` (:754–763) |
|---|---|---|
| Interpreter | `Command::new("python3")` (`:428`), found on the PATH `/usr/bin:/bin` that `master()` sets (:885) | `/usr/bin/python3` |
| Flags | `-I -B` | `-I -B` |
| Script | `topic.parent().parent()/tools/narration_delivery.py`, with topic = `CARGO_MANIFEST_DIR` (build268 `main.rs:48`, `run_with_contract_and_options`) | `WORKSPACE268/tools/narration_delivery.py` |
| `--topic` | `CARGO_MANIFEST_DIR` | `WORKSPACE268/explainer_corpus/complex_analysis_visual` |
| `--narration` | rebind269 | rebind269 |
| `--consumer` | `master`, then `captions` | both |
| `--gain-db` | `GAIN_DB.to_string()`; `GAIN_DB` is `0.0` (`narration.rs:46`), which prints as `0` | `0` |
| `--chapter-key` | all 15, in registry order (`:411–426`; `captions all`; `master` requires the whole registry) | all 15, records in registry order (:698) |
| `--timelines` | `--out` = `job/final` for both consumers (`:457–463`) | the attempt folder |
| `--score` | `master` only (`:465–467`) | `master` only |
| cwd | REPO07 (318 :892) | REPO07 |
| Environment | the guard's environment, plus PATH, TMPDIR and XDG_CACHE_HOME | PATH only |

- **Crate location.** Build268's clippy log checks the crate at `D/work/review-repair268/workspace/explainer_corpus/complex_analysis_visual`. That supports 318's reading that the compiled-in manifest directory lies inside WORKSPACE268. I did not read it from the binary itself.
- **The same table bytes.**
  - The contract reads only `KEY.timeline.tsv` and `KEY.cuelog.tsv` from `--timelines` (`narration_delivery.py:324`, `:329–330`).
  - The preflight's symlinks and the job's hard links (:877–879) point at the same staged files.
  - The `.cues.tsv` link is never read. It does no harm.
- **The environment makes no difference.**
  - The three modules read no environment variable on the verify path. `os.environ` appears only in generation code: `coherent_narration.py:288` and `narration_cache.py:319–328`.
  - They load each other by explicit file path (`narration_delivery.py:17–20`, `coherent_narration.py:25`), so `-I` changes nothing.
  - The only locale-dependent step is `read_text()`, and `script.tsv` is pure ASCII.
- **Script check.** The kit's own check that `script.tsv` matches the compiled script (`cli.rs:400–402`) is not run by the preflight. `film()` covers it instead: WORKSPACE268's `script.tsv` must be at the source-copy hash (:265–267).
- **The asserted result.** `verify()` returns `consumer`, `verified_cues` = len(outputs), `selected_cues` and `plan_mode` (`:342–347`). These are exactly the four values 318 asserts. Root's 305b `master` run returned 105, 105 and `external-ui`, without stem rows.
- **A refusal costs nothing.** It leaves only the attempt folder. No inputs file is written, and earlier attempts are kept.

### 2. N7: what the contract actually reads (307 N7)

Every file opened for `master` and `captions` with the external UI plan, and where 318 freezes it:

| Read site | File | Frozen by 318 |
|---|---|---|
| `narration_delivery.py:17–20`, `coherent_narration.py:25` | the three contract modules | `CONTRACT` (:91–92) at build268 source-copy hashes (:265–267); the copy lists all three |
| `verify` `:278`, `read_script` | `script.tsv` | `CONTRACT` |
| `external_plan` `:254–272` | canonical `narration_units.json`, `external-units.json`, `assembly.json`, every unit's generation receipt | `contract_reads` :295–298 and :318–321; NARRATION `rglob` (:273) |
| `coherent.verify` `:500–501`, `:561` | `assembly.json`, `report.json`, the 105 cue WAVs | `rglob` (:273) |
| `direction_contract` `:154–156`, `direction_map_path` `:122–139` (relative to the module's own REPO, WORKSPACE268) | the review269 map | :299–305 |
| `review_directions` `:186–188`, `verify` `:546–549` (relative paths against the cwd) | disposition evidence | :306–309 |
| `assembly_voice` `:89`, `unit_spans` `:348–388` | record WAV, receipt, the row's WAV, source and request | :314–329 |
| `verify` `:550–560` | each report row's source WAV | :330–334 |
| `timeline()` `:117` | each table line's WAV (absolute) | `schedule()` :368–371; for 09 also `provenance313` :475 |
| `cue_log()` `:162` | the narration WAVs in the cue log | `schedule()` :375–377 |
| `cue_log()` `:152–156`, `score_parts` `:122–146` | the chapter stem, or its `parts.tsv` and part WAVs (parts for 08, 09, 11, 13 and 14) | `film()` :260–262, all 44 files of score21's summary, including every `parts.tsv` and part WAV |
| `timeline()` `:100–103` | `listen` sounds | none to freeze: every `listen` row in the local production tables has 4 fields, so no sound path |

- **Result.** I found no file the contract reads that 318 does not freeze.
- **Path resolution agrees.** Relative paths resolve against REPO07 in both: 318's `at()` (:285–289) and the contract run with REPO07 as its cwd.
- **Scope of this check.** This confirms the code. Whether every file exists on CSD at its hash is for 318's own `check` to show.

### 3. Mixed 09: binary 313's render, consumed by binary 268

- **Paths are not compared.**
  - The contract compares tables and cue logs with the assembly by hash only (`timeline()` `:117`, `cue_log()` `:162`).
  - The kit's `replay` reads each cue-log path and refuses a hash mismatch (`master.rs:96–98`).
  - Captions read each line's WAV by path (`captions.rs:553`).
  - Nothing compares a table's WAV directory with `--narration`, so 09's rebind313 paths are consumed as bytes, as the notes say.
- **The replay line.**
  - The kit prints `"{:<14} replay = fmn's soundtrack, bit for bit ({} cues, {} samples)"` (`cli.rs:978`; `:1007` in `3c5221fe`). It prints it only after `null_test` (`:970`) and the length check against the picture.
  - 318's `REPLAY` (:184) matches both padded keys and the longer, unpadded ones; the longest key has 18 characters.
  - The cue count is `parse_cue_log`'s rows. Like `replays_of` (:832), it skips blank lines and `#` lines (`master.rs:62`).
  - The sample count is per channel, so frames × 800 at 60 fps.
  - `replays_of` therefore cannot reject a run that the kit accepted. It is a consistency record.
- **The before-master check is narration-only.** `capture_check09` compares narration-only captures. Final317's cue log adds three score-part rows (317 `verify_tables` :190–206).
  - Binary 313's mix of those rows is checked only by the master's null test.
  - That test runs before any master write (`cli.rs:970` against `:993`), so a failure spends only the job name. The notes say the same.
- **317's interface, as 318 reads it, holds at `19abeee9…`.** It has:
  - `VERSION` `'317'`, `CHAPTERS`, `HOST` `'csd'`, and `ROOT` = D;
  - `BIN`, `NARRATION` and `TIMELINES` at binary 313, rebind313 and timeline313;
  - `job_name`, `preview_binding`, `independent` and `verify_tables`;
  - `check`, which returns review, permit sha and binding;
  - summary fields `host`, `preview_job`, `boundary_sha256` and `build313_qualification_sha256`.

  `EARLIER` leaves out `'317'`, so `check()` still passes inside the master after final317 exists. An existing 303 09 job still blocks it (notes R1).
- **Two maps.** Rebind313 binds its own map, 317 `MAP313`, while 318's consumers bind rebind269's map for all 15 chapters. This has no effect on the consumer run:
  - the contract uses the map only for its prediction-hold and directed-caption output, which the kit ignores;
  - the kit's captions take prediction holds from the timeline marks;
  - 09's tables equal timeline269's under the narration mapping.

### 4. Retained 02/03/13/15 through import311

- **Unchanged code.** The retained branch, `imported()`, `retention()` and `schedule()` are 293's. Review 316 found no High or Medium against 293's consumption of the imports, and its one Low asks only for a sentence in the notes. 318 keeps `IMPORT_KEYS` and `ROW_KEYS`.
- **What is new for these rows.** The preflight runs the contract on their staged tables: score21 stem rows, with the narration paths of the foreign worker. It reads them through the import aliases, which exist only after the imports. A refusal there still costs nothing.

### 5. Other changes

- **Launch order.** Wait, pins, `gates`, preflight, the 12 GiB check on `D/jobs`, the `'x'` inputs file, then the guard (:778–816), as the notes give it.
- **`narration_cache.py`.** Build268's source copy lists it at `47331efd…`, so the `CONTRACT` loop in `film()` cannot raise KeyError.
- **Summary flags.** The summary leaves every acceptance flag false (:929–931). `bit_identical` claims only the audio replay, which the kit prints only after a passing null test.

## Notes (no change asked)

- **N1: `preflight.json` sits in `D/work/tmp`.**
  - That folder is the TMPDIR of the master job's own kit and ffmpeg processes (:885).
  - The file is a bound input, re-hashed at the start and again at the end of the job (:851–852, :918–919).
  - If anything cleared that folder during the job, the final re-hash would fail after the outputs exist. I found no such cleaner.
  - Writing it in Q, beside the inputs file, would remove the dependency.
- **N2: 09's encoder parameters are not compared before the join.**
  - The master joins with `-c:v copy`.
  - 317 runs 303's encoder flags and checks `rc=crf crf=16.0` (317 :233, :243).
  - Nothing compares 09's `avcC` with the other 14 chapters' before the join. 307 N5's identical-`avcC` evidence predates 317.
  - This stays with full decode and verification.
- **N3: kit source scope.** I compared the three `cli.rs` functions and `master.rs` with the CSD repo-view copies. I read `captions.rs` only from the repository. I did not see binary 268's compiled bytes.

## Still open after a successful 318

- **Listening.** No audio was heard, here or by any check above.
- **Fine A/V sync.**
- **Final AV acceptance,** including independent AV acceptance.
- **Full 4K decode,** and verification of the master.
- **Exact captions.** The kit's caption checks are technical. Nobody has read the caption text against the speech.
- **The master's pictures, motion and chapter joins.**
- **Loudness and true peak.**
- **Equal mixer code across binaries (E5/273 U1).** For 19 → 268 and now 313 → 268, the linked mixer code is not proven equal. The null test is the check.
- **Publication306's binding** to the final master.

## Method and limits

- **What I read.**
  - 318 and its notes in full.
  - The diff against 293.
  - 317 in full.
  - The three build268 contract modules at their read sites.
  - The kit's `check_delivery_contract`, `captions_video`, `master_video`, `parse_cue_log`, `replay`, `video_captions` and `validate`.
- **Local records.** The production timeline copies in `final-picture-source41` were used only for record kinds and gain levels: all 210 `line` rows have level `0`, and every `listen` row has 4 fields. They are not the staged bytes.
- **What I did not run.** No mock, contract, binary, guard, ffprobe or remote read. Every result above comes from the source and the records in Q.
- **What I did not judge.** No listening, picture, motion, caption-text, 4K or AV judgement of any kind.

## Not done

- No render, synthesis, build, master, launch, import or transfer.
- No edit to 318, its notes, 317, 293 or any other file.
- No writes besides this file.
