# cc0 combined master 293: notes and operational handoff

## Scope

| File | sha256 | State |
|---|---|---|
| `cc0-combined-master293.py` | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` | new, prepared only |
| `cc0-combined-master293-notes.md` | this file | new |
| `trj-final-master184.py`, `trj-master-after-chapters184.py`, `trj-master-after-imports184.py`, `trj-verify-after-master40.py`, `trj-final-verify19-02.py` | unchanged | read, not edited |

**What was done.** Preparation, syntax checks and a scratch mock only. Nothing ran on any host. There was no remote mutation, render, synthesis, upload, permit, Beads, TODO or source edit.

**Narration.** ElevenLabs Frederick is unchanged. The master uses rebind269's 105 WAVs and refuses unless each is byte-identical to assembly19's. No audio heard or checked.

**Why a new master.** Retirement289 stopped the four production19 waiters (master-after-imports184, verify-after-master40, pictures-after-verify182, joins-after-verify83). Their files are preserved. None of them can master the combined film:
- they bind TRJ, binary 19 and timeline21;
- 184's job-name allow-list admits only production19 finals and the repaired 08/09 pair.

## What 293 is

It is one concrete script on CSD (`D = /data/projects/jmev-stroke-probe-20261106`). It is not a framework.

**Mastering.** 184's body, kept almost verbatim:
- each chapter's MP4, timeline, cues, cuelog and layout, plus its last soundtrack capture as `soundtrack_0.wav`, are hard-linked into `jobs/csd-combined-master293/final`;
- `review-repair268 master --narration rebind269 --score score21-stems --out final`, then `captions all`;
- all 15 `replay = fmn's soundtrack, bit for bit` lines, in registry order;
- ffprobe gates: 3840×2160, `60/1`, `nb_frames` = 133,662, AAC 48 kHz stereo, audio duration within one AAC frame;
- SRT not empty and VTT starting `WEBVTT`;
- every link and every bound file re-hashed;
- `master-summary.json` with 184's keys.

**The waiters.** The launcher replaces 184's two waiters:
- an optional bounded wait (`--wait-hours`, 0–48, default 0) for each row's terminal receipt;
- a finished failure stops it, and there is no retry;
- then every gate runs again, the inputs are frozen with exclusive create (`combined-master293-inputs.json`), and `guard_csd02.py --phase native` starts once;
- inside the guard, the policy must be 8/4.

**Modes and commands** (run from the csd Q folder, with `/usr/bin/python3 -I -B`):

```
cc0-combined-master293.py check  --manifest <Q>/<root manifest>.json        # read-only; one JSON line per chapter
cc0-combined-master293.py launch --manifest <Q>/<root manifest>.json [--wait-hours H]
```

The `master` mode is only the guard's inner command.

## Root's final chapter manifest

**Format.** `{"chapters": [15 rows]}`, written by root inside csd Q, with absolute paths. Rows are in exact registry order (build268's and build19's qualification lists must agree). Extra or missing keys are refused.

**Combined row** (binary 268). It is required for all 11 changed chapters: 01, 04–12 and 14.

```json
{"chapter": "05_gallery", "kind": "combined",
 "worker": "<Q>/csd-reviewed-final287.py", "worker_sha256": "<actual>",
 "job": "csd-reviewed-final-05-gallery287", "inputs": "<Q>/final-05_gallery-inputs287.json",
 "permit": "<actual root permit>"}
```

- **A CSS-290 row** has `worker` = `<CSS>/qualification01/css-final-repair290.py`, job `css-reviewed-final-<chapter>290`, and also:
  - `import_receipt`: `<D>/jobs/<csd import job>/resource-receipt.json`;
  - `import_inputs`: that import's inputs manifest, under D.
- **Accepted worker families.** Only `Q/csd-reviewed-final*.py` and `<CSS>/qualification01/css-final-repair*.py`, with the job pattern `{csd,css}-reviewed-final-<chapter-dashed><digits>`. Later versions that keep these names pass without an edit here.

**Retained row** (02, 03, 13 or 15 only):

```json
{"chapter": "15_closing", "kind": "retained", "worker": "...", "worker_sha256": "<actual>",
 "job": "hz2-production19-final-15-closing53", "inputs": "...", "permit": "...",
 "import_receipt": "...", "import_inputs": "...",
 "retention_decision": {"path": "<Q>/<root decision file>", "sha256": "<actual>"}}
```

- **Pinned values in 293.** Worker, job, inputs and permit must equal the plan183 row. The permit sha, the movie sha and the frames must equal that final's render summary:

  | Chapter | Final | Movie | Frames |
  |---|---|---|---|
  | 02 | final02-terminal167 | `bf0891a4…` | 5,372 |
  | 03 | final03-terminal115 | `20353d01…` | 7,249 |
  | 13 | final13-terminal200 | `49934034…` | 10,641 |
  | 15 | final15-terminal81 | `3e2440a8…` | 4,379 |

- **Re-rendering instead.** If root re-renders one of these four, it is a combined row. That needs a worker version that covers the chapter (287/290 do not).

**Nothing guessed.** 293 contains no future receipt hash and no future job name except its own master job. Every pin is an existing value; the next section lists the cross-checks.

## Gates

**Whole film:**
- receipts succeed (finished, exit 0, inputs == final inputs, no error or stop) for build268, build19, rebind269, timeline269, timeline21 and score21;
- binary 268 is `983954c5…`; the build19 binary is `f4836eee…`;
- exactly 15 registry chapters, which are exactly the 11 changed plus the 4 retainable;
- timeline269's manifest is its receipt's inputs;
- **all 45 timeline269 tables** equal timeline21's after the one assembly19 → rebind269 mapping, every header has fps 60, and the frames sum to 133,662;
- score21: `all_qa_passed`, chapters equal the registry, every listed file is at its hash (stems, parts, QA), and every chapter's QA passed;
- build268's `source-copy.json` is `81d28272…`. The binary's own contract files must equal its entries: the topic `script.tsv`, `tools/narration_delivery.py` and `tools/coherent_narration.py` in `work/review-repair268/workspace`. Production19 `main.rs:48` binds the topic to `CARGO_MANIFEST_DIR`, so the master reads these files and not `production-repo07`.

**Every chapter row:**
- the worker is at the manifest's sha, its `BIN` hashes to the expected binary, and its root is the row's host;
- **foreign rows:** a successful CSD import whose manifest files are all at their hashes on CSD (184's rule);
- the render receipt succeeds;
- its command names the worker and exactly one `--chapter`/`--job`/`--inputs`/`--permit`, equal to the row;
- **all inputs present:** the inputs manifest is the receipt's inputs, and every file in it is present on CSD at its hash (184's rule). This includes the worker and the permit;
- **render summary:** chapter, frames, binary and permit sha agree, and every output is at its hash;
- the frames equal timeline269's header;
- **permit gates on any host** (check()'s permit assertions): actually reviewed pictures and motion, no unresolved defects, reviewer and observations present, the timeline sha (timeline269 for combined rows, timeline21 for retained), evidence hashes and motion intervals. Combined rows also need source-binding evidence and the worker's `preview_binding()`;
- **re-running `check()`:**
  - The CSD worker's `check()` is re-run, as 184 did for every row.
  - The retained rows' production19 `check()` is re-run on CSD.
  - **Not re-run: CSS-290 `check()`.** Its first step asserts the CSS-only alias, so it cannot pass on CSD. Those rows rest on the CSS job's own guarded pre- and post-render `check()`, its bound bytes re-hashed here, and every host-independent gate above. Re-running `check()` on CSD would add only four things: the alias, the same-chapter-job scan, and `success()` on the CSS-local receipts (encoder stop79, transfer75, transfer290, delivery290) and on `delivery-result.json`. The CSS job asserted all four. This is the one place 293 re-checks less than 184 did.
- **schedule and audio**, for all 15 chapters: the worker's `verify_tables()`, then 293's own check:
  - the tables equal timeline269's after mapping rebind269 to the worker's own narration folder;
  - every timeline and cuelog narration row lies in that folder and is byte-identical to rebind269's;
  - every stem row is a score21 stem at score21's hash;
- the last capture is 48 kHz stereo s16 at frames × 800;
- every staged file is in the render summary and on the same device as `D/jobs`, so the hard links cannot fail midway.

**Retained rows also need:**
- **Source:** the scene file (`turning.rs`, `two_planes.rs`, `real_integral.rs`, `closing.rs`) is the pinned hash in three places, and it is not in the changed list:
  - build19's workspace;
  - build268's pinned source copy;
  - build268's workspace.
- **Schedule and audio:** as above. Because score21 is one whole-film score and every chapter uses its stems, the music seams at 01|02, 03|04, 12|13 and 14|15 come from the same score (impact261 E4).
- **Root's decision:** the file named in `retention_decision`, inside Q, at its sha.
- **What retention does not add.** No claim of pixel identity with binary 268 (E5 stays open). The existing review state carries forward, per disposition283.

## Before `launch` (root)

1. **Put 293 on csd** at `qualification01/production-repair17/cc0-combined-master293.py`, at the sha above. Then write the manifest and any retention decision there.
2. **Finish the eleven changed chapters.** They must render terminal and stable with their root permits.
   - 287 and 290 still fail closed for the nine chapters waiting on preview274's names.
   - A later 287/290 version must name preview274 first.
3. **Import every foreign row to CSD** (CSS-290 renders and retained 02/03 TRJ, 13 CSS, 15 HZ2) with a guarded CSD import job. The import must place, at the original absolute paths, on the same filesystem as `D/jobs`:
   - the render job folder: the receipt, `render-summary.json` and `render/` with the soundtrack;
   - every file in the render's inputs manifest;
   - the permit's evidence and source-binding files;
   - the narration copies the tables name;
   - for retained rows, the receipts the old `check()` reads:
     - TRJ: `trj-production19-delivery02`;
     - CSS: `css-runtime-transfer75`, `css-encoder-stop79` and `css-production19-delivery76`;
     - HZ2: `hz2-runtime-transfer47`, `hz2-encoder-stop03` and `hz2-production19-delivery49`.

   **D-path rows in a foreign manifest** (that host's copies of CSD files) are compared with CSD's own files. A drifted CSD file refuses that row, for example anything in `work/production-repo07` edited after the copy. `check` prints the path.
4. **Free space on CSD.** The job writes a PCM master and a joined MP4 about the size of all fifteen chapter MP4s. Compare that with the guard caps (target growth 20 GiB, work 56 GiB) and CSD's free space (about 104 GB at fleet286) before `launch`.
5. **Run `check`, then `launch`.** Run `check` and read every line. Launch only when all 15 rows are `ok`. There is no retry; a failed master needs a new version and job name.

## Reusing the actual verify40 with the new master paths

**Do not relaunch `trj-verify-after-master40.py` or `trj-final-verify19-02.py` unchanged.** Both hard-code TRJ production19 names:
- master job `trj-production19-master01`, which never ran;
- manifest `master01-inputs.json`;
- job `trj-production19-verify01`;
- `guard_trj01.py`.

Creating those names now would label the combined film as production19.

**Safe reuse means new versioned copies of both files with only these constants changed.** Every assertion and command stays byte for byte, so the verification logic is the actual old one:

| File | Constant | 40 / 19-02 value | Combined value |
|---|---|---|---|
| both | `R` | TRJ root | `D` (CSD) |
| both | `Q` | `R/qualification01/production19` | `D/qualification01/production-repair17` |
| both | `M` (and `O = M/final`) | `R/jobs/trj-production19-master01` | `D/jobs/csd-combined-master293` |
| both | `N` | `R/jobs/trj-production19-delivery02/narration` | `D/jobs/csd-review-repair-rebind269/narration` |
| 40 | `JOB`, 19-02 `J` | `trj-production19-verify01` | a new name root picks, for example `csd-combined-verify293` |
| 40 | `HELPER` | `Q/trj-final-verify19-02.py` | the new 19-02 copy |
| 40 | master inputs | `Q/master01-inputs.json` | `Q/combined-master293-inputs.json` |
| both | verify inputs | `Q/verify01-inputs.json` | a new name, still exclusive create |
| 40 | guard and Python | `R/guard_trj01.py`, `python3` | `D/guard_csd02.py`, `/usr/bin/python3` (as on CSD) |
| 19-02 | `D`, `I` | unchanged | unchanged. `I`'s `script.tsv` exists on CSD and is `234f7257…`, the same bytes as binary 268's topic script |

**Why the old logic still holds:**
- 293's summary keeps the keys verify reads: `frames` = 133,662, `native_bit_identical_replays` = 15, `chapters` and `outputs` (master/srt/vtt with path and sha256). The `final/` layout is the same.
- 19-02 line 31 compares each timeline row's WAV with `N`'s by content. Rebind269 holds the same Frederick bytes, and 293 has already asserted that every chapter's narration copy equals rebind269's.

**Confirm on csd before writing the copies:**
- **`N/report.json` and `N/phrase-timings.json` exist in rebind269's narration.** 19-02 line 18 binds them and line 23 reads them. 290's delivery code compares a rebind269 report and phrase marks, which suggests they exist, but I did not see the folder.
- **The three shared helpers are in csd Q at their pinned bytes:**
  - `verify_stage09_final.py`, `f266a231…`;
  - `verify_video_preserve_logs01.sh`, `3f4c68db…`;
  - `caption-interval-audit01.py`, `3cc26436…`.

  They are not in this Q folder locally. A local `verify_stage09_final.py` at `f266a231…` is in `opus-replacements-20261104/logarithm-stage09-final-verification02/`.
- **ffmpeg and ffprobe on CSD.** CSD's repaired170 render already used `/usr/bin` with `PATH=/usr/bin:/bin`.

The picture and join waiters (182, 83) are outside this task. Chapter start frames are unchanged, because all 45 schedules equal timeline21's. But eleven chapters' pictures changed, so any picture pass is on the new master.

## Differences from 184

**Host and environment:**
- Host CSD, binary 268, rebind269 narration and `guard_csd02.py`.
- `PATH=/usr/bin:/bin` for the native commands, as in the CSD workers.

**Output files:**
- `staged-inputs.json` and `master-summary.json` are written with exclusive create.
- 184's human audio acceptance flag is named `audio_heard` here, still `false`. Verify does not read it.
- The summary adds: version, host, binary, narration, score, the manifest path and sha, and per-row records (kind, host, job, worker, permit and movie hashes, import receipt, retention decision, start frame).

**Gates:**
- **New:** every whole-film gate, the receipt-command check, schedule and audio for every chapter, the retention gates, the same-device check, and the exact row keys.
- **Replaced:** 184's production19 prefix allow-list and its repaired 08/09 special cases. In their place: the two combined worker families, plus pinned retained rows.
- **The one reduction:** CSS-290 `check()` is not re-run (see Gates).

## Checks run (local only)

**Syntax.** `compile()` under Python 3.15.0 with `-I -B`; no bytecode written.

**Pins (25/25).** Every pin was compared with local records:
- binary 268 and `CHANGED` against 287;
- binary 19 against `trj-final-chapter19-02.py`;
- the source copy against `review-build268-terminal272/source-copy.json`;
- the retained rows against `chapter-plan183.json` and the four terminal render summaries;
- the scene hashes against the 268 source copy and the `production-source-preserved214` receipt.

**Command gate.** Run on the four actual retained render receipts: all `ok`, each finished, exit 0 and stable.

**Scratch mock (31/31 as expected).** It ran outside the repo, on a fake CSD/TRJ/CSS/HZ2 tree, with fake workers, a fake binary and patched pins. It exercises 293's logic, not csd.

- **Passes:**
  - the film gates;
  - `check` mode with all 15 rows ok (11 combined, one of them CSS-290, whose fake `check()` raises and was correctly not called; 4 retained);
  - `gates()`;
  - `launch`, which froze 574 files including workers, binary, stems, decision, manifest, permits, narration, tables and contract files. It called the guard once with the master argv.
- **Refused:**
  - a retained changed chapter;
  - a missing decision;
  - another retained job;
  - a wrong worker sha;
  - a bad job pattern;
  - import keys on a CSD row;
  - rows out of order;
  - a table or narration change, including self-consistent forgeries with the summary re-hashed, so that `schedule()` itself refused them;
  - a non-score21 stem row;
  - stem bytes;
  - a narration byte;
  - a scene file;
  - one of the 45 tables;
  - a rebind269 WAV;
  - a failed render receipt;
  - a failed import receipt;
  - motion not reviewed;
  - another permit in the command;
  - changed output bytes;
  - a second launch.

**Not run:**
- the `master` mode body: the binary's `master`/`captions`, ffprobe, hard links on csd;
- the real 287/290/production19 workers' `check()` and `verify_tables()`;
- the guard.

## Independence and limits

- **Not independent.** I wrote 287, review279 and now 293. 290 is CC1's, and its CSS behaviour here is read from its source. 293 needs a reviewer other than cc0 before launch.
- **No acceptance of any kind.** 293 adds no pixel, motion, fine-sync, exact-caption, loudness or final AV acceptance. Its summary keeps all those flags `false`. Verify still needs to run, and human audio, fine-sync, motion and independent final AV acceptance stay open with root. No audio heard or checked.
