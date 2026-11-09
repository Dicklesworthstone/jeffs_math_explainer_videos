# CC2 review 284: preview274 helper, its notes, and launcher 296

cc2, 2026-11-08. This is an independent review of CC3's preview helper, its notes and root's launcher. It covers the source only:
- nothing was launched, deployed, rendered or built;
- no Beads, TODO, memory or source file was edited;
- no 274 frame exists yet, so none was viewed.

Narration is ElevenLabs Frederick Surrey. Root owns any launch.

| File (in `production-repair17/`) | sha256 | State reviewed |
|---|---|---|
| `cc3-preview-fade274.py` (908 lines) | `d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669` | final. Unchanged from 06:11:19Z through this review. |
| `cc3-preview-fade274-notes.md` (362 lines) | `a14dfe9702414e1b3e3945b7b0e1fc0a9b4b8a152b4b33b16eba478afbacd01a` | corrected version. I first read `84d96a1f…`. |
| `launch-reviewed-preview296.py` (80 lines) | `8f754ec7b743615cfc6fa27642936462f24a24237a9c83cea1864c8b78d9a2b0` | prepared, not run |

## Result

**0 High, 0 Medium, 5 Low, 6 notes.**

I found nothing that should stop root launching 296 once timeline269's terminal receipt succeeds. The Lows are wording, one latent rule, and one host check to make before the run.

**Gate still open.** Timeline269 is still running: root reports native 258555 live at 06:17 with no terminal receipt. Nothing here claims timeline269 equal to timeline21. Every start time below comes from timeline21's tables. Each is conditional on two equalities, both of which 296 and the helper enforce at run time:
- timeline269 equal to timeline21;
- the preview's own tables equal to timeline269's.

## What I ran (local, read-only)

These ran outside the repository and are not delivered.

- **Sources.** I imported the helper and ran its own `check_sources` on local copies of all 11 compiled files. Each was matched by sha256 to the helper's pins and to build268's `review-build268-terminal272/source-copy.json`.
  - All 11 match.
  - The check passed: 44 sites and 51 plays. Per chapter, 01 2, 04 2, 05 22, 07 2, 09 2, 10 4, 11 3, 12 8, 14 6.
- **Exact placement.** I ran the helper's own `place()` on the nine real `timeline21-terminal01/` tables, using a flat signal.
  - Every exact start in notes §3 is reproduced to the 0.001 s shown.
  - All 7 "end = total" checks pass: 6311, 6774, 9879, 16504, 7442, 3943, 11756.
  - Every fit site falls back to NOT LOCALIZED, as it should on a flat signal.
- **Picks.** I ran `picks_for` on those placements.
  - With every fit NOT LOCALIZED (the worst case), it picks 1,274 frames on 160 sheets, before lint lines.
  - The gallery anchors give 33 frames, 153.72–164.15 s.
  - The rigidity windows give 59 frames, 12.00–104.00 s.
  - The last frame of 10 (7441), at the 10→11 join, is picked.
- **Parsers.** I exercised the helper's `chroma()` on preview243's local `08_loop_sums.mp4`, capped at 120 frames by a `trim` in its own graph.
  - It parsed 120 values with frame indices 0…119, under local ffmpeg 8.1.2.
  - Its ffprobe tuple reads `(960, 540, '60/1', 14959)`.
- **Gap scan.** I listed every line that mentions `stage` but is not matched by `TIMED`, inside each anchor→site span and each fit chain. I then read the bodies of the helpers those lines call.

## Lows

**L1. Fit "acceptance" is localization only; keep every fitted placement a sampling hypothesis.**
- **Where.** Notes §4 "Acceptance" and helper `:694–699` (`how='chroma fit'`).
- **The problem.** CC3's own negative test (notes §4) shows the floors do not tell a plate from a nearby vector fade: a site with no plate scored t = 50.9 with slope 0.75, and only the ambiguity rule refused it.
- **Fix the wording.** Rename it "localization threshold".
- **For root and permits:**
  - No fit score, cross-check score, `flag` or `chroma` value is timing, picture or motion evidence.
  - A fitted start counts only where the sheet frames around it visibly show the plate ramping.
  - The helper itself claims nothing more. Its `sampling.json` scope says "picks only … no picture or AV acceptance".

**L2. Latent pause rule.**
- **Where.** Helper `frames_covering` (`:368–371`) applies fmn's exact-rational ceil to `pause` steps as well as plays.
- **The two rules.**
  - Plays: fmn `a399d626` `crates/fmn-anim/src/clock.rs:191` counts ceil on the exact rational. So 0.4 s is 25 frames, and CC3 is right for plays.
  - Pauses: `draw.rs` `pause` (candidate 264, `:198–201`, imported at `:15`) uses the kit's `narration.rs:605`, `ceil(s·60 − 1e‑6)` in f64. Under that rule 0.4 s is 24 frames.
- **No current effect.** The only modelled pause is 0.6 s, in A1's fit post, where both rules give 36. `check_sources` refuses any unlisted pause.
- **Fix.** If a pause step is ever added, use the kit rule for it.

**L3. Notes §7 is out of date on `narration.rs`.**
- **Root's check.** Root hashed csd `work/tree/repo-view/crates/explainer_kit/src/narration.rs` as `8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8`. It equals the local `crates/explainer_kit/src/narration.rs`, which I hashed to the same value.
- **What that file says.** `Narrator::say` (`:1182–1238`) runs `wait_gap`, then captures the frame, places and logs the sound, and sets `busy_until`. It never waits after logging the line. The `line` anchor holds.
- **Breath anchor.** `window` (`:1302`) runs `hold_breath`, then logs the breath frame, then waits `frames_covering(seconds)`, which is 120 frames for 2.0 s. The `breath` anchor holds.
- **Fix in the notes.** Replace "not compared" with this path and hash. This is source-line support, not runtime or picture acceptance.
- **Remaining binding (root).** The binary compiles the kit through the crate's relative path inside `work/review-repair268/workspace`, and `source-copy.json` covers only `explainer_corpus/…` and `tools`. If that workspace's `crates/explainer_kit/src/narration.rs` is a separate copy rather than repo-view's file, hash it too.

**L4. Notes §3 says "`Kit::finish` adds no frames".**
- **Why that is not generally true.** `kit.rs:201` `finish` calls `hold()` first, and `hold()` waits for a sounding line.
- **Why it is zero here.** It adds nothing in these nine chapters only because the 7 end = total checks show it.
- **Fix.** State it as measured, not as a rule.

**L5. Host-only steps run after the 1.5 h render.**
- **The steps.** Schedule asserts, ffprobe, the `signalstats` parse, lint parse and extraction all run after the render. They fail closed. But outputs are exclusive, so a failure costs the render, and a rerun needs a new job and manifest name.
- **Already established.**
  - Preview243's 540p `--lint` tables equalled timeline242's (its `:14` assert passed, receipt exit 0).
  - The parsers work locally (above).
- **Still unproved on csd:**
  - `/usr/bin/ffprobe` exists. The worker's PATH is `/usr/bin:/bin`, and preview243 never called ffprobe.
  - csd's ffmpeg prints `frame:N` / `lavfi.signalstats.SATAVG=` lines in this format.
- **Check before launch (root).** Confirm the ffprobe path with a plain `ls`, not a job.

## Confirmed

- **Binary 268.**
  - The helper requires `sha(work/bin/review-repair268) == qualification['binary_sha256']`, 233 tests, the native plate-fade probe and Clippy (`:799–801`).
  - The local terminal272 qualification has exactly these keys: `983954c5…`, 233, True, True, and all 15 chapters.
  - The source-copy format is a `files` dict keyed by repo-relative path, as the helper reads it (`:802–805`).
  - Root confirmed the csd file exists (10,797 bytes).
- **Narration.** It renders with `--narration` set to rebind269's folder (`:23`, `:813–815`). Terminal276 records rebind269 finished, exit 0: 105 cues, 105 WAVs byte-identical, `listening_acceptance` false. The voice is ElevenLabs Frederick Surrey. No audio was heard.
- **Schedules (pending).**
  - **What the helper asserts.** It reads timeline269's `comparison.json` before the render (`:807`). After it, it asserts, per chapter (`:824–842`):
    - preview table == timeline269 table, byte for byte;
    - preview table == timeline21 table after the assembly19→rebind269 path mapping;
    - the comparison row says equal, with the same sha256;
    - frames equal the comparison count and ffprobe `nb_frames`, at 960×540, 60/1.
  - **Shapes.** These match `review-repair-timeline269.py`'s writer. The table rows (`line id frame …`, `breath frame 2`, `fps`, `frames`, `#` header) and the lint format (`@205.6s`, "still there at N later looks") parse as written.
  - **Not available yet.** Timeline269's actual output.
- **All nine raster chapters.** `CHAPTERS` holds 01, 04, 05, 07, 09, 10, 11, 12 and 14 with the right modules (14 → `continuing`). These are exactly final278/287's nine non-06/08 chapters.
- **Site completeness.**
  - The completeness scan passes on the real bytes.
  - The gap scan found only builders and scene adds or removes in every span: `Painter::build`, `border`, `tex`/`text`, `axes`, `axis_names`, `carded`, `previews`, `jagged_mosaic`, `window_box`, `marks`, `Turning::build`/`stop`, `add_to_scene`, `remove_from_scene`, `clear_updaters`, `add_value_tracker`. I read each body; none advances the clock.
  - **One play per call.** `Painter::show`/`leave` are one `fade_in_all`/`fade_out_all` (`draw.rs:1047–1072`), and each fade is one `play_prepared` batch. fmn emits no frames for an empty batch, so I checked the variable lists in the modelled chains: none can be empty (for example, residues `extra` always holds a border, two TeX labels and a text).
  - **Closing fades.** Every closing `all` list holds only vector parts, except the_bump's `thumbs`, which is site A2.
  - CC3's caveat stands: a plate passed under another name would slip past a text guard. I found none.
- **Anchors.**
  - The counts in the corrected notes hold: line 16 (18 plays), breath 7, after-exact 10 (15), fit 9, after-fitted 2. That is 33 exact sites and 40 exact plays.
  - fmn frames are 1-based, with alpha = t/rt clamped. This matches `visibility()`.
  - H1a's `skip {275, 278}` is correct for k = 0. H1b, at k = 1, follows H1a by the extras fade alone.
- **Picks.**
  - **Every play gets** s−1, s, the middle, N, N+1 and N+30.
  - **Fade-outs also get** N−3…N+3, truncated at the film's end. **Fitted plays also get** ±3 at the start and the end.
  - **NOT LOCALIZED sites** get their window every 15 or 30 frames, never a guess.
  - **Old anchors.** These are the 249/255 gallery times (153.92–163.15 s, plus the closing vector fade at 9789/9803/9818) and the 247/253 rigidity windows (12.0–16.7 s every 0.25 s, 62.3–104 s every 2 s, 88–103.6 s every 1 s). Their frame totals, 9879 and 7442, match the old records.
  - **Expected transitions.** The G2 strips field, added without a fade, and the I5 blank painter both get frames.
  - **Size.** The worst case is 1,274 frames, not the notes' ~784, if all 11 fit-dependent plays fail.
- **Guard, exclusive lane, no duplicate.**
  - **Admission.** The helper requires `admitted`, no `exit`, and policy 8/4 (`:795–797`). Finished receipts keep `admitted` and that policy (local terminal272, 276 and 265 receipts).
  - **Exclusive writes.** `native.stdout`/`native.stderr` open with `'xb'`, `sampling.json`/`summary.json` with `'x'`, per-chapter `mkdir()` has no `exist_ok`, and extraction uses `ffmpeg -n`.
  - **CPUs.** The helper is pinned to CPUs 0–15 with 16 threads, as preview243 was.
  - **No self-approval.** It launches no other job.

## Launcher 296

No defect found. The gate semantics are right.

- **Terminal-receipt gate.** `success()` (`:18–24`) requires all of these:
  - `finished`, `exit == 0`, `admitted`;
  - `inputs_sha256 == final_inputs_sha256`;
  - no `error`, `resource_stop` or `stopped_for_resource_limit`.

  It is applied to timeline269, build268 and rebind269, and after the guard to preview274 itself. It replaces the helper's weaker test, that `comparison.json` exists. **Run the helper only through 296.**
- **All 45 tables rechecked** (`:54–70`):
  - 15 unique chapters;
  - frames summing to 133,662;
  - 45 rows;
  - each table's new and old sha256 equal to its row;
  - byte equality after the named mapping.
- **Frozen hashes preserved.** Timeline269's manifest must hash to its receipt (`:43`). Every row is re-hashed (`:45–48`). The new manifest then holds:
  - those rows, which carry the compiled workspace tree;
  - all files of the timeline269, build268 and rebind269 jobs, so `source-copy.json` and `qualification.json` are included;
  - the rebind269 narration;
  - the binary, guard, worker, this review and the launcher.
- **Exclusive.** It refuses if the job directory or `review-repair-preview274-inputs.json` already exists, and writes the manifest with `'x'`. A refused admission therefore needs new names, by design. The guard call is unchanged in form from 243/269.
- **For root.**
  - **This review is in the manifest.** This file becomes a frozen input, so keep it byte-stable through the run.
  - **CPUs.** Confirm no concurrent job is pinned to CPUs 0–15. Timeline269 used 0–3, and must be finished first.

## For final287's preview274 entry

These values are taken from the actual files:

| Field | Value |
|---|---|
| job | `csd-review-repair-preview274` |
| manifest | `review-repair-preview274-inputs.json` |
| worker | `cc3-preview-fade274.py` at `d80cf7b6…`; 296 passes that path, so the receipt command names it |
| tables folder | `preview` |
| narration | rebind269's folder, so the mapping is the identity |
| build | build268 |
| binary | `review-repair268`, `983954c5…` |

No 238→268 delta item applies, because it is the same build.

## Limits

This is a static source and logic review, plus local parser and placement runs. It establishes no picture, motion, 4K, listening, fine-sync, publication or AV acceptance.
- **Not run on csd.** The helper, 296 and the fits have not run there.
- **Sheets are stills.** The fades are sampled at a few frames each, never watched in motion.
- **Narration was not heard.**
