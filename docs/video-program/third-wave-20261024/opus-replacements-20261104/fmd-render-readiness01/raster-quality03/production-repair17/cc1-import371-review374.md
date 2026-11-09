# CC1 review 374 of CC3's import 371

CC1 (BronzeJaguar), 2026-11-08, about 22:35Z. This is an independent source and closure review of `csd-css352-import371.py`. It was checked against `csd-retained-import311.py`, `cc0-combined-master350.py`, `css-final-repair352.py` and the actual CSS352 closure of chapter 01.

**Scope and limits**
- Read-only throughout. I ran no export, send, prepare, receive, check, render, master or permit.
- I issued no permit and wrote nothing on either host. On both hosts I only listed, read, stat'ed and hashed, at nice 19 for the large hashes.
- I made no edits to any helper, test or note, and no TODO or Beads change.
- This review accepts none of the following: final AV, listening, fine sync, motion, picture, 4K, pixel equivalence or publication. No audio was heard and no frame was viewed.
- Narration stays the existing ElevenLabs Frederick Surrey delivery309/rebind269 WAVs, unchanged.

Notation:
- D is the CSD job root, and Q is `D/qualification01/production-repair17`.
- `CSS` is the second render host's root (on CSD, css-a1's alias), and `CQ` is `CSS/qualification01`.
- TOP is `D/work/imported-css-root311`.

## Files reviewed

| File | sha256 | Source |
|---|---|---|
| `csd-css352-import371.py` | `ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f` | Q, as CC3 froze it, 911 lines |
| `cc3-css352-import371-notes.md` | `2bd27f9f3a02e7c3421516385c60a2fef4ef39afe8aa7d6de1f5d57084bfb003` | Q |
| CC3's `test371.py` | `d5e8fe89b97065a99f12f46603f164415e830bd8d03ed5d65f3b2bb697b7ede2` | CC3's scratch, read in full, then a copy run (11/11) |
| `csd-retained-import311.py` | `862968830a076c2030599b65e1f48b776501f87d99045cb662570045e613d5ea` | Q, unchanged |
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | Q, the same bytes root staged to CSD (379) |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | Q, 350 `WORKERS352` |
| `master350-manifest-draft384.json` | `ddbe3f2b4225fc580e14e8f3890c63caa6e51061e52dcb2664e1680773436968` | Q, CC0's draft rows |

An earlier draft of the helper, `15d8a845…` (22:17Z), differs from `ba337aca` only in three docstring lines about the alias. I rebuilt it from my saved diff and compared the two.

## Verdict

There is no High or Medium finding. As source, 371 is fit for root to run one chapter at a time, as the notes' section 6 relays it, after that chapter's CSS render is a terminal success.

**Exact preservation**
- 371 never creates or changes the alias or TOP.
- It writes only new names, with `x`/`xb` and one `os.link` per new file.
- It keeps every staged name, and it refuses a differing existing destination before the first write.
- It leaves 311-css-a1's 858 placed rows and its records untouched.

**Import closure**
- The closure is the CSS job's actual admitted inputs and every row of them, plus the receipt, the summary and its files, and the job folder.
- That covers every CSS-path file that 350's `chapter_gate`, `schedule`, `imported` and `x264_settings` read for a CSS row.

**One host per chapter.** It holds today for all ten 352 chapters (section 4).

Three Lows follow (section 5). None blocks.

## 1. The actual CSS352 closure, measured independently

I read 01's admitted inputs on CSS: `CQ/final-01_the_bump-inputs352.json`, sha `b95913176d1e6f402a58894b2266cc118bc1a20428db3d1fcfb0363138b7d282`. It equals `inputs_sha256` in the receipt of `css-reviewed-final-01-the-bump352`, which was admitted at 21:51:57Z and was still unfinished at 22:33Z, with no summary yet.

| Rows | Count | Measured on the hosts |
|---|---|---|
| foreign (under `CSS`) | 3,455 bound rows, plus the inputs file and the job folder | 724 are in css-a1's import-result at the same path and hash (1,368,443,735 bytes), 0 differ and 2,731 are new (0.770 GiB, outputs not yet included). All are real, symlink-free paths on CSS. |
| shared (under D) | 3,203 | all 3,203 hash equal on CSD, and all are symlink-free there |
| system (`/usr`) | 2,269 | all 2,269 hash equal on CSD, and all 2,269 are rows of CSD's `review-repair-timeline269-inputs.json` (`2c4e86e5…`), `review-repair-preview243-inputs.json` (`6510db78…`) or `review-repair-preview274-inputs.json` (`20d91bdb…`) at the same hash. One row, `/usr/bin/timeout`, is a symlink on CSD (to `/usr/lib/cargo/bin/coreutils/timeout`, which is also a row). |

These figures agree with notes sections 3 and 7, which give 724 rows at 1,368.4 MB and 829,059,714 new bytes including the job folder so far.

The system rows enter through 352 `carried_rows()`. 350 `chapter_gate` re-hashes every bound row on CSD, and 311 refuses any row outside its root and D. A third kind is therefore needed, and 371's rule is the right one: never streamed or written, a carried row at the same hash, and at that hash on CSD.

I also checked that 350's freezing never demands a symlink-free path for these files. `launch` and `master` only `sha()` the frozen rows, and `absolute(real=True)` applies only to the import receipt and inputs, the manifest and Q files. So the `/usr/bin/timeout` row in `import_inputs` is accepted.

## 2. The closure checks against the real records

Each field that `closure_problems` compares was read from the real record.

- **CSS receipts.**
  - Six receipts were read on CSS: `css-encoder-stop79`, `css-runtime-transfer75`, `css-repair-transfer309`, `css-repair-delivery309`, and the 01 and 04 permit transfers. All are terminal successes with policy 10/8/4/56/20.
  - The admitted 01 render receipt has the same policy. Its command starts `/usr/bin/python3 -I -B CQ/css-final-repair352.py render` and names the pinned chapter, job, inputs and permit.
- **01's inputs.** They bind, at their frozen hashes:
  - the worker `1074051b`, binary 268 `983954c5…`, `guard_css74.py`, the delivery309 inputs and result, the transfer309 result and `css-runtime-manifest75.json`;
  - the four receipts, the three CSD manifests, all 105 narration WAVs and 01's three timeline269 tables.
- **delivery309 result.** It reports binary `983954c5…`, `verified_cues` 105 and source `D/jobs/csd-review-repair-rebind269/narration`.
- **Root's CSS permits.** For the five permits (01 `299b8fe0…`, 04 `110168fe…`, 05 `f3ef9aea…`, 12 `689164d6…`, 14 `778ca111…`):
  - each names its chapter, host `css`, binary 268 and its frames;
  - each one's `timeline_sha256` equals the timeline269 table bound in 01's inputs, all five checked;
  - each has a non-empty source-binding and independent list, and the independent files sit directly in Q.
- **Root's Q copies.** The five Q copies exist on CSD at those hashes. 05's permit375 has the same bytes as candidate 368.
- **The host-field gap.** I closed the gap named in notes section 9 with the real 01 permit bytes. Each altered permit had its row and pin hashes re-pointed at the altered bytes, and every other closure file was reported unavailable.
  - Unaltered: no permit problem.
  - Host `csd`, frames + 1 or chapter `04_reading_color`: refused, "permit does not match the pins".
  - An empty independent list: refused, "permit lists no independent_review".
- **Shape on the real rows.** 371's `manifest_problems` on the real 01 rows, with the four outputs pending, returns no problems. It counts 3,459 foreign, 3,203 shared and 2,269 system rows.
- **The render outputs.** These two are checked by reading only, because 01 had not finished:
  - 352 `render()` writes every summary file under `render/`;
  - `verify_tables` writes the timeline with the delivery309 narration in place of rebind269's, which is what 371's line and cue-log checks expect.

## 3. Preservation and capacity on CSD (read-only)

**Alias and TOP**
- The alias is a symlink to TOP, which css-a1 created (`alias_created: true`).
- TOP is a real directory on the D/jobs device.
- css-a1's import-result is `1e504aa0…`: 858 written, 0 identical, manifest `589a03b6…`.

**Existing jobs**
- No `csd-reviewed-final-01*`, `-04*`, `-05*`, `-12*` or `-14*` job exists on CSD.
- No 371 job or Q file exists.
- `csd-reviewed-final-09-boundary344` holds the native lane: started 20:58Z and unfinished at 22:21Z.

**Guard semantics.** CSD's guard, `guard_csd02.py` `f1cae1db…`, wraps `guard_owner56g_target20g.py` `1c19a9b8…`, which in turn wraps root's bounded guard `819746fb…`. It measures:
- WORK as D/jobs, capped at 56 GiB;
- TARGET as D/work, with growth capped at 20 GiB above `D/jobs/allocation-baseline.json` `target_bytes`, which is 0;
- phase growth as ΔD/jobs + ΔD/work, at most 4 GiB.

371's `preflight` uses exactly these figures, and its `allocated()` is the guard's. Because hard links are deduplicated only within each tree, the measured growth is twice the new bytes. css-a1 confirms this: 2.822 GiB measured for 1.409 GiB.

**Headroom.** The guard's last sample at 22:21Z showed 97.40 GiB free, D/jobs at 9.45 GiB and D/work at 5.12 GiB.
- 01 adds about 0.8–0.9 GiB once its outputs exist; css-a1's 4K movie was 13.5 MB.
- Each later chapter adds tens of MB.
- That stays far inside the 3.5 GiB, 56 GiB and 20 GiB limits.

**The guard admits these inputs.** Its `check_inputs` requires only absolute paths at their hashes (it follows symlinks), so guard inputs that include system rows are admitted.

**Write paths**
- `prepare` writes only `Q/csd-css352-import371-<dashed>-a<N>-inputs.json`, with `x`.
- `receive` writes only inside its job (`incoming/`, then `import-result.json` with `x`), then new links and directories under TOP.
- An identical existing file keeps its inode, and a racing writer is never overwritten.

**311's rows stay valid.** 311's `check` mode re-hashes only its own rows, and 371 neither changes nor removes any of them.

CC3's test passed 11/11 on my copy (`d5e8fe89`), on a scratch tree under my own scratch folder. It covers:
- the stage-and-place round trip into an existing alias folder, with the alias unchanged and staged names kept as second links;
- re-runs and existing identical files;
- refusals of the stream, destination, alias, TOP, shared, system and permit-copy kinds, plus same-chapter CSD final jobs and earlier attempts;
- the guard caps and the command shape;
- the source write scan;
- AST identity with 311 for `Unavailable`, `clean_path`, `csd_side`, `flag`, `header_line`, `inputs_path`, `make_dirs`, `q_manifest`, `read`, `sha`, `sha_bytes`, `stage` and `utc`.

## 4. One host per chapter: evidence today

| Chapter | Permit in Q (host) | Job (host) | 371 import in draft384 |
|---|---|---|---|
| 01_the_bump | `01_the_bump-css-permit364.json` (css) | `css-reviewed-final-01-the-bump352` (CSS, running); no CSD final | `csd-css352-import371-01-the-bump-a1` |
| 04_reading_color | `04_reading_color-css-permit366.json` (css) | not yet started; permit transfer done 21:51Z; no CSD final | `…-04-reading-color-a1` |
| 05_gallery | `05_gallery-css-permit375.json` (css) | not started; no CSD final | `…-05-gallery-a1` |
| 12_residues | `12_residues-css-permit370.json` (css) | not started; no CSD final | `…-12-residues-a1` |
| 14_continuation | `14_continuation-css-permit370.json` (css) | not started; no CSD final | `…-14-continuation-a1` |
| 06, 07, 08, 10, 11 | `-csd-permit375/366/375/366/370` (csd) | CSD rows; no `css-reviewed-final-*` for them on CSS | none (CSD rows) |

**What 371 enforces**
- The permit and the summary must both name host `css`.
- Before any write, no `csd-reviewed-final-<dashed>*` job may exist on CSD.
- The guard's native lane is exclusive, so no CSD final can start between that check and placement.
- In draft384, each chapter appears exactly once in 15 rows, and only the five CSS 352 chapters carry 371 import rows.

## 5. Findings

**L1 (Low): the one-host refusal sees only existing CSD job folders.**
- The guard takes the lane lock before `job.mkdir()`. So a CSD final for the same chapter that is queued (launched and waiting for the lane) has no folder yet, and `plan_problems` cannot see it.
- Root's own record of intent, a `<chapter>-csd-permit*.json` in Q, is not checked either.
- **Today:** the CSD permits in Q are exactly 06, 07, 08, 10 and 11, disjoint from the five CSS chapters, so nothing is wrong now.
- **Suggestion** (for a later version, not an edit to this one): also refuse when Q holds a permit for the chapter naming host `csd`, and record the observed permit hosts in `import-result.json`. Until then, root should not queue a CSD final of a chapter it imports. Notes section 9 already calls the check point-in-time.

**L2 (Low): the notes' section 6 wording on retries.**
- The notes say "if an attempt fails or is refused, use the next attempt number."
- Root's 316 disposition, L1, says a job without a finished receipt needs investigation and recovery, never an automatic increment.
- The code is safe: `earlier_attempts` refuses while an earlier attempt's folder lacks a finished receipt. The wording should still match.
  - The next number is used only after root has read the failed attempt's receipt and job folder.
  - A refused `check` or `prepare` creates no job, so the same number stays usable. If `prepare` already wrote inputs, it refuses "inputs exist" on a repeat.
- If the attempt number changes, the 350 row's `import_receipt` and `import_inputs` change too, so CC0's draft384 must follow.

**L3 (Low): `receive()` and `prepare()` have not run anywhere yet.**
- CC3's round trip recomposes `plan_problems`, `read_header`, `stage` and `place` itself.
- Never executed so far are the guard-state and command asserts, the bound-input checks, the staged `closure_problems`, the alias re-check after placement, the per-row check through the original paths and the `import-result.json` write.
- I read these lines against 311's `receive`, which css-a1 ran for real. The new branches (system rows, permit copy, `alias_problems`) are straightforward, and I found no defect.
- The first real import is therefore also the first runtime test of `receive()` and `prepare()`. Root should treat that chapter's post-import `check` (re-hash through the original paths and the printed `rows350`) as the proof, before writing the row into the 350 manifest.

## 6. Notes for root (information, not findings)

- **I1. Job-folder logs.** The closure check does not demand the job-folder logs (`command.log`, `native.stdout`, `native.stderr`). `export` includes them, and neither 350 nor 363 reads them. Their absence from a hand-made manifest would not be refused, but it would not matter either.
- **I2. When to start the pipe.** Start it only while CSD's native lane is free, as notes section 6 says. While it waits for the lane, the CSD guard has written no receipt, and the CSS `send` blocks on the pipe. The four-hour `/usr/bin/timeout` starts only when the guard runs `receive`.
- **I3. Re-streaming.** Every chapter re-streams its already-placed bytes, about 1.4 GB for 01 (311's behaviour). These are verified and discarded, never written.
- **I4. CSD's `/usr`.** CSD's `/usr` must stay as it is until 350 has run. Any package change makes 350, and 371's `check`, refuse the rows that bind it.
- **I5. Ordering.** Import order is free. With `--after`, 04's inputs bind 01's render receipt, which travels as a foreign row and is found identical once 01 has imported. 350's launch waits on each row's import receipt.
