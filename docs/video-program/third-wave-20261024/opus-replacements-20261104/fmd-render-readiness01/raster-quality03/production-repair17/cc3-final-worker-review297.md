# cc3 review 297: final287 worker and launcher (source and record audit)

cc3, 2026-11-08. This is an independent review of root's launch path for the combined 268 finals.
I wrote none of 287, 278, 279 or 282. Preview274 is my own work and a separate, later input.
This review does not judge whether it is correct.

| File (in `production-repair17/`) | sha256 |
|---|---|
| `csd-reviewed-final287.py` (worker) | `f01f4c29049acc86d486d4b083273182febd4cfdf0e7f4b755c9c2fb81247510` |
| `csd-launch-final287.py` (launcher) | `80d9ab1159251ba372d2e7199761d2d6fba20113ea7e3c69e9f2bb1217597bba` |
| `cc0-final287-changes.md` | `4b8281f73b524e439f2837b37210829d9bdec17451589f88e3d368eafce45112` |
| `cc0-final-worker-review279.md` | `7c57b9d158aba0c7c8df344a5d12f51dd75f317c1f0c7daf7485ecab92b5c075` |
| `cc1-preview-to-final-binding282.md` | `1fbcedef34f65933ff3d818740d124cb2492a39e276916eb600f6753d5c62066` (= 287's pin) |

The worker and launcher hashes equal those in the change note. I diffed 287 against 278: only
the lines the note lists changed.

## Result

**For 06 and 08, a scoped pass with no blocker.** On the records I could check, the 06/08 path
will pass once three things hold:
- timeline269 finishes stable and equal;
- binding282 is on csd at `1fbcedef…`;
- root writes a real permit that includes an actual motion review.

Until then it fails closed.

**For the other nine chapters, it fails closed as intended.** `PREVIEW274 = None` stops
`preview_binding()` inside `check()`. The launcher calls that at launcher:23, before it writes
`final-<chapter>-inputs287.json` or calls the guard, so a refused run leaves nothing behind.

**Two Low findings, both actionable without a source change.**

## What I checked

**Receipt gates.**
- I read four real guard receipts here: preview243, build268, rebind269 and HZ2 final09.
- **Keys.** All four carry `finished`, `exit`, `inputs_sha256` and `final_inputs_sha256`, as
  `success()` (worker:57–60) reads them. `command` is a list.
- **Preview243's receipt** shows exit 0, and `6510db78…` for both its inputs and final inputs.
  Its command element
  `…/qualification01/production-repair17/review-repair-preview243.py` exactly matches
  `str(worker)`. The list-membership test at worker:71 therefore passes.
- **Order of checks.** `check()` requires the receipts of stop13, timeline21, score21, build268,
  rebind269, timeline269, preview243 and build238 to show success, before admission and again
  after the render. Timeline269 is still running, so `check()` fails closed for now.
- **The final's own receipt.** The worker requires it admitted with no `exit` and guard policy
  8/4, and requires its inputs hash to equal the frozen manifest (worker:159–161). This matches
  the receipts above and root's current preview296 launcher, which uses the same
  `guard_csd02.py --phase native` call.

**The 45 tables.**
- **Check.** Worker:105–107 compares all 15 build268 chapters × timeline/cues/cuelog with
  timeline21, mapping only `csd-production-assembly19/narration` to
  `csd-review-repair-rebind269/narration`. That is the same check timeline269 makes, and the
  same one root's preview296 launcher repeats.
- **Output tables.** `verify_tables` (worker:127–153) binds the 4K render's tables to
  timeline269's. It is unchanged from 244.
- **06/08 preview tables (worker:78–80).** I checked the premise locally. Take preview243's real
  06/08 tables from `preview243-review265/preview/`, with rebind242 mapped to rebind269. Take
  `timeline21-terminal01/`, with assembly19 mapped to rebind269. All six tables are equal.
  - The path counts agree: 06 has 9 per table, 08 has 5, and cues have none.
  - No other narration folder appears.
  - So worker:80 passes exactly when timeline269 passes its own equality.
- **Headers.** They give 06: 10,030 frames and 08: 14,959, both at fps 60.

**Score and narration binding.**
- **Score.** Unchanged from 278:
  - the score21 receipt;
  - `all_qa_passed`;
  - the chapter list equal to build268's;
  - every score file hash;
  - the per-chapter QA;
  - the cuelog score rows tied to `frames × 800` samples.
- **Narration.** The worker requires every row of `review-repair-timeline269-inputs.json` in the
  final's frozen inputs (worker:166–168), and the launcher re-hashes each row (launcher:39–42).
  `review-repair-after-build269.py:15` froze rebind269's whole job folder, narration included,
  together with timeline21's job folder. So the final renders with the narration bytes and
  baseline tables that timeline269 used. That is the ElevenLabs Frederick Surrey set from
  rebind269. Nothing was heard.

**Source delta, 238 to 268.**
- **Key format.** Both build scripts write `source-copy.json` as
  `{"files": {path relative to the workspace: sha}}` (build238:24–25, build268:30–31). So 287's
  `SCENE_DIR + name` keys are the right form.
- **Local reconstruction.** I rebuilt build238's copy from the local production19 tree
  (`production-source-preserved214/workspace`, all 69 keys present) and the four 238
  candidates: boundary52, loop-sums238, one-derivative238 and mod236. I compared it with
  build268's real copy (`review-build268-terminal272/source-copy.json`, `81d28272…`).
- **Result.** The files that differ are exactly `DELTA_238_268`: draw, gallery, mod, rigidity
  and the new plate_fade_probe. Boundary, one_derivative and loop_sums are byte-equal across
  the two builds, so `one_derivative.rs` and `loop_sums.rs` pass worker:83.
- **Limit.** This predicts the build238 copy. The real file (`6d4c7e93…`) is on csd, and only
  282 read it there.
- **Preview243's manifest.** It holds the four rows `preview_binding()` requires (worker:76–77):
  - the 238 binary;
  - `review-repair-preview243.py`;
  - build238's `source-copy.json`;
  - build238's `qualification.json`.

  They come through the freeze chain `review-repair-after-build242.py:10–15` (build238's job
  folder) → timeline242 → `review-repair-after-timeline243.py:18–21`.

**4K encoder.**
- **Command.** The render command (worker:176) is 3840×2160, 60 fps, `--crf 16 --preset
  veryfast`, with `--score`. These are the same settings as 244 and CSS290.
- **Against the only completed final in these records.** HZ2 final09
  (`final09-terminal280/render-summary.json`) shows `rc=crf crf=16.0`, with veryfast-class
  options: `ref=1`, `me=hex`, `subme=2`. So the x264 test at worker:185–186 matches the real
  string, and chapters made this way stay consistent.
- **Probe.** ffprobe pins h264, yuv420p, AAC at 48 kHz stereo, and `nb_frames` equal to the
  timeline header.

**Guards against duplicate or stale launches.**
- **Earlier versions.** `job_name()` matches the 244 and 278 launchers' naming
  (launcher244:22), so refusing an existing 244/278 job (worker:97) actually catches them.
- **Same chapter twice.** The 287 job folder check and the exclusive `inputs287.json` stop two
  launches of one chapter.
- **Permit.** Its path is resolved before admission and checked as absolute and
  symlink-free.

## Findings

**F1, Low (operational): every 287 worker takes the same 16 CPUs, and the render has a 12 h
limit.**
- **The facts.**
  - The worker pins itself to the first 16 CPUs of its inherited affinity (worker:172–173).
  - The launcher always passes `--threads 16` (launcher:61).
  - csd has 64 CPUs (`fleet-capacity286.json`).
  - So 06 and 08 launched from an unrestricted shell both run on CPUs 0–15, with 48 CPUs idle.
  - The preview274 helper also pins CPUs 0–15, with a fixed `set(range(16))`
    (`cc3-preview-fade274.py:810`, inherited from preview243).
- **The consequence.** The render subprocess times out at 43,200 s (worker:177), and a timeout
  fails the job after hours of work.
  - **The only rate on record.** HZ2's final09 ran at about 1.1 fps (16,504 frames in 4 h 11 min,
    12 threads). That is a different host, so treat it as an estimate.
  - **At that rate:** 08 alone needs about 3.8 h.
  - **Shared two or three ways on the same 16 CPUs:** 7.6–11.4 h, close to the limit.
- **What to do.** Give each final its own CPU range at launch, for example `taskset -c 16-31` and
  `taskset -c 32-47`, away from preview274's 0–15. The alternative is to run them one at a time.
  The worker uses whatever affinity it inherits, so this needs no source change. Before relying
  on it, check that `guard_csd02.py`, which I have not seen, keeps the inherited affinity.

**F2, Low (provenance): the launcher does not pin the worker it runs.**
- **The gap.** The 287 launcher loads `csd-reviewed-final287.py` from csd's folder and freezes
  whatever is there. Root's current preview296 launcher instead takes `--worker-sha256` and
  `--review-sha256` and asserts both. The frozen `inputs287.json` records the helper's hash only
  after the fact.
- **What to do.** Immediately before each launch, confirm these csd hashes:
  - `csd-reviewed-final287.py` is `f01f4c29…`;
  - `csd-launch-final287.py` is `80d9ab11…`;
  - `cc1-preview-to-final-binding282.md` is `1fbcedef…`;
  - the permit is at the hash root intends.

**Inherited, unchanged and not new** (279 L4 and 278 behaviour). These fail closed, not unsafely:
- **Whole folders are frozen.** The launcher re-hashes all of preview243's manifest rows
  (12,607) and both job folders. Any later rewrite there fails the launch, or fails the final
  after the render, at worker:190.
- **A refused admission leaves a stale file.** It leaves `final-<chapter>-inputs287.json`, which
  must be moved aside before a retry.

## Not established

- **Nothing was run.** I did not run either script, even as a dry run. I saw no csd file: not the
  guard, score21, the live preview243 or build238 folders, timeline269's output or any permit.
  The build238 source copy is a local prediction.
- **No acceptance.** This is a static source and record audit. It is not picture, motion,
  listening, fine-sync, 4K or final-AV acceptance, and it does not authorize a launch or
  publication. Root owns the launch and the permits.
- **Preview274** is outside this review. Its F1 overlap is a scheduling fact, not a view on its
  correctness.
