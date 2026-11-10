# Handoff 736: apportionment master 735

Author: CC3 (WhiteSwan). Status: source only, staged for independent review. Nothing was launched, staged on a host, built, rendered or synthesized, and nothing went over SSH. No Beads edit or commit. Master 700, verifier 701, the 727 set and record 734 are unchanged.

## Files

| File | sha256 |
|---|---|
| `apportionment-csd-master735.py` | `373b62de2deb462435e4fe4571b7087a655e6b0099698b4aaab0af4ee71f4fd6` |
| `apportionment-csd-master735-tests.py` | `0ca733b6595359040640e61dfd87d2490f3f7829b415d3180b51d2500c9af739` |
| `apportionment-csd-master735-from700.diff` | `8abb70ae37f63d25b3392c7c538d2ea95ecd106b326971426b1599a0c0339131` |

Read, not changed: master 700 `e589f7f3302f9d97a9ad5b6b6f244ea3fccac71ffb91f5364f83fe555575743d`, verifier 701 `02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f`, helper 727 `d94aab217a6ee6639328414b774a7a374fb610c0716f3213222ca421a3762b9f`, record 734 `3659f8fa12f535e5adbbe37d69f23110e27f9df6aeff66ec7a8bd395aee83324`.

## What 735 changes from 700

### 1. Caption 168, after `captions all` and before any summary hash

`repair_captions` (`master735.py:481-502`) runs from `finish` (`505-514`), which `run` calls after the native commands, the replay check and the final `recheck(bound)`:

1. It loads helper 727 by its pin and checks its values (cue 168, 612110, 616290, 616310, 616900, limit 20, file names). `freeze` also checks it and freezes the file, and `run` checks it again right after the first recheck, so a missing or changed helper stops the job before the master command, which may take up to 14400 s.
2. The kit's pair in `final/` must equal master 700's actual pair from record 734 (`0568f2dd…`, `482c5817…`).
3. The helper writes the repaired pair to the new `captions-repaired735/`. Its output must equal record 734's candidate (`61e636ae…`, `214e15cb…`).
4. The kit's two files are renamed, not copied or rewritten, into the new `captions-generated735/`. `final/` then gets hard links to the repaired pair under `apportionment.en.srt` and `apportionment.en.vtt`.
5. The summary is `master735-summary.json`. Its `srt_sha256` and `vtt_sha256` are the hashes of what `final/` now holds, the repaired pair. Every master 700 field is kept and computed the same way. Three fields are new:
   - `caption_repair`: both pairs' hashes and folders, the helper's path, hash and JSON report, and the scope.
   - `chapter06_manifest` and `chapter06_manifest_sha256`.

This is a display dwell adjustment only: caption 168 stays on screen 20 ms longer. It is not evidence of speech timing. No speech, narration, timing table or verifier threshold changes.

### 2. Chapter 06 from root's reviewed manifest

`CHAPTER06_MANIFEST` and `CHAPTER06_MANIFEST_SHA256` are `None`. While they are unset, `inputs` and `run` both refuse before loading anything. The file names no chapter 06 job, script or receipt.

When root sets them, the manifest must be a JSON file in Q with exactly these keys. This format is my proposal; root's actual 731 results may need another shape.

| Key | Must be |
|---|---|
| `chapter` | the registry's sixth chapter, starting `06_` |
| `job` | the successor final job folder in the jobs folder, not `csd-apportionment-final672-06_…` |
| `script` | its guarded script, in Q |
| `inputs` | its frozen inputs manifest, in Q |
| `summary` | its summary file name inside the job |
| `layout_sha256` | the reviewed hash of its new `06_….layout.tsv` |

`successor_chapter` (`272-290`) applies every check that `csd_chapter` applies to a final 672 job. It checks:
- a successful receipt, with this exact guarded command and the policy;
- the inputs manifest hash;
- the summary's chapter, ffmpeg version and encoder tags;
- the render folder and the frame count from comparison 666;
- every file a real, non-symlinked file.

The manifest is frozen with the job's records.

In `check_tables`, chapter 06's timeline and cue tables must still equal timeline 666's byte for byte, since the timing is unchanged. Only its layout is held to the manifest's hash instead of timeline 666's. Every other chapter keeps 700's checks.

### 3. Names

The job is `csd-apportionment-master735`, the inputs manifest `csd-apportionment-master735-inputs.json`. The outputs are `materialized735.json` and `master735-summary.json`, and the log lines say `INPUTS735` and `MASTER735_COMPLETED`.

Everything else is master 700: 18 of its 22 functions are unchanged, as are all its constants except `JOB`, `INPUTS` and `SCOPE`, and its imports. That includes chapters 01-02 through import 683, the other twelve from final 672, build 656's binary for the `chapters`, `master` and `captions all` commands, the resource policy, CPUs 0-3 with `RAYON_NUM_THREADS=4`, and the bit-for-bit replay check.

## Before root runs `inputs`

- Stage `apportionment-caption-repair727/repair_caption168.py` under Q on the build host at that same relative path, at the hash above.
- After 731 and the full timing comparison, write the chapter 06 manifest from the actual reviewed results and set the two constants. That changes master 735's hash, so the edit needs its own review.
- 735 keeps build 656's binary for the master and captions commands, as 700 did. Running them with the 731/732 binary instead would mean changing `BUILD` and `BINARY`, which 735 does not do.
- A verify701 successor has to point at master 735's job and `master735-summary.json`. Its line-203 check then holds unchanged, because `final/` holds the repaired pair and the summary hashes it. Its `rglob` over the job will also freeze both caption folders; hard links pass its no-symlink rule.

## Where it can stop

- At `inputs`: if chapter 06's timeline or cue tables are only normalized-equal to timeline 666's (for example, a different path column) and not byte-equal, 735 refuses. That costs nothing and needs root's decision.
- After the long run: if the kit writes any caption pair other than master 700's actual pair, the job stops after the master and captions commands and before the summary. The movie and the kit's captions stay in `final/`. This is deliberate, because the 727 repair was reviewed on exactly that pair. A different pair needs a new review, not a relaxed pin.

## Tests

Run beside the file: `python3 -I -B apportionment-csd-master735-tests.py --actual DIR`, where DIR holds root's copies of master 700's actual SRT and VTT. With those copies: 39 tests, OK. Without `--actual`: 37 run and pass, and the 2 tests that need the copies are skipped and reported. 22 planted faults in scratch copies of master 735 were all caught. Two earlier misses led to the inode check and the rebound-helper test below.

Actual inputs:
- **Pins (5):** the reviewed file hashes; the caption pins equal record 734's values; the helper sits at the same path under Q; verifier 701's limit line is unchanged; chapter 06 is unbound and refused.
- **Same as 700 (7), on the actual source:**
  - the 18 unchanged functions and all other constants and imports are AST-identical;
  - the four changed functions keep every other 700 statement, in order;
  - each replaced statement is exactly the intended one;
  - the summary keeps every 700 field, computed the same way;
  - order: the chapter 06 gate comes before `load()`, the helper check right after the first recheck, the repair after `captions all` and before the summary;
  - four CPUs and four Rayon threads.
- **Actual caption bytes (1):** the copies go into a scratch `final/`.
  - The kit's files themselves (same inodes) and their exact bytes survive in `captions-generated735/`.
  - `final/` holds hard links to the repaired pair, with 2 changed bytes per file.
  - Verifier 701's own parser reads 443 cues, SRT equal to VTT. Its own fast-caption line finds cue 168 alone before and nothing after, and only cue 168 differs (616290 to 616310 ms).
  - The source copies' hashes were the same before and after the run.

Synthetic, named as such in the file:
- **Synthetic master integration (1):** `finish()` as `run()` calls it, on a synthetic job and movie around the actual caption bytes.
  - The summary's caption hashes equal what `final/` holds, which is verifier 701's line-203 check.
  - The kit's hashes are kept, and the scope says display dwell.
  - A second `finish()` refuses.
- **Chapter 06 binding (9), successor record (7), tables (4):** a scratch tree with a manifest, a successor job and reference tables. They cover a good case and each refusal: unbound, changed or misplaced manifest, keys, chapter, the old final 672 job, paths, summary name, layout hash, command, policy, inputs, encoder, folder, frames, unsafe or missing or symlinked files, a layout other than the manifest's, changed timing, and another chapter's new layout.
- **Caption refusals (5):** another pair, a changed helper, a rebound helper with other values, existing caption folders, and no summary when the repair refuses. In each, the job folder is left untouched.

Real production has not run: no build-host job, guard, support671, native binary, ffmpeg or Rayon run exercised any of this.

## Not established

The chapter 06 manifest format is a proposal, and no actual chapter 06 successor exists to test it against. I have not seen any build-host hash of these files. Nothing here was rendered, listened to or watched. A cache folder from root's 734 run (created 01:00:39 UTC) sits in the 727 folder; it is git-ignored, and I left it in place.
