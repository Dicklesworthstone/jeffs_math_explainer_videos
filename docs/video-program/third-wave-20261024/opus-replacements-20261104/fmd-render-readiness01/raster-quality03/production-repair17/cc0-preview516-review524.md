# Review 524: CC2 recovery helper 516 (CC0)

cc0, 2026-11-09, 06:28 UTC. Q is this folder. B is `narration_lab/third-wave-20261024/root/five-film-release-20261027`. This review covers `cc2-apportionment-preview-recovery516.py`, sha256 `7547515c183cebb28a8ef49f9832dda1bc3a222ed93bdb6579f834f0398a4cd6`, 233 lines, read whole.

Method: source review plus read-only checks. I reused my 520 evidence and did not re-hash or re-probe the eight movies. I did not run the helper, not even `--check-only`, which decodes all eight movies. Nothing was rendered or launched, and no file outside this note was written.

## Verdict

**No blocker, and no High or Medium defect.** 516 can resume local production as written. One Low finding and three notes follow, and none needs a change before launch.

## The six points root asked for

1. **Output isolation holds.** 516 writes only to `B/apportionment-frederick-preview516/preview`, and refuses to start if that folder exists.
   - The guard creates `B/<job>` because `guard_owner56g_target20g.py` sets WORK to B, so `JOB.is_dir()` holds only under the guard.
   - The job folder does not exist yet, at 06:27Z.
   - The kit moves earlier outputs aside only inside `--out`.
   - 516 never writes into preview458, timeline444 or score445.
2. **The inherited 458 inputs and the 444/445/458 receipts are pinned exactly.**
   - The guard gets the same `--inputs` file as 458, `5817cb64`. 516 hashes that file and all 3,820 of its rows.
   - Those rows include the timeline444 and score445 receipts, score445's `command.log`, `preview458.py`, the binary `c7730356` and `tools/narration_delivery.py`.
   - So both receipts are bound by hash, not only by their success fields. 520 found every row matching at 06:16:41Z.
   - The guard re-hashes every row after the command (`check_inputs` at start and end).
   - 516 also pins 458's receipt (`a8d25c31`) and checks that `preview458-summary.json` is absent.
3. **The 01–08 bindings are right.** The files are reused in place, not copied; nothing in preview458 is copied, moved or edited.
   - 516 pins the whole preview458 folder by `LISTING458_SHA` = `fc5100da`.
   - I rebuilt that listing from my 520 hashes, without re-hashing: the 52 files are the receipt, `command.log`, the 8 movies, 32 tables, 8 soundtracks and 2 `previous/*.v1` soundtracks, with their sizes. It gives `fc5100dab6c12d9fbb320be155872e556501512b0719f57a6513bb7fa1a30ed3` exactly.
   - The listing is checked before and after the renders.
   - `chapter()` applies the checks 520 made to each of 01–08: tables equal to 444's; the cue log equal to 444's plus the stem row (D1); stream shape and packets; the x264 string, which 520 found identical in all eight; and the soundtrack length. Those checks pass on the 520 data, including 04's D3, since its AAC and video durations are within 1024/48000 s.
4. **Only 09–15 are rendered.** `todo = chapters[8:]`, after checking that the binary lists 15 chapters and that the first 8 are 01–08.
   - It makes one `render <chapter>` call per chapter, with 458's exact flags; only the target and `--out` change.
   - **A one-chapter render passes the delivery contract.** The kit passes one `--chapter-key` to `tools/narration_delivery.py`. Because the narration assembly covers all 93 cues, the scope stays canonical, and `--score` is refused only for score and master.
   - **Each new chapter is checked like 01–08.** Its timeline, cues and layout must equal timeline444's. timeline444 rendered 64×36 with `--lint`, and its 01–08 layouts already equal 458's 960×540 ones, 08's 4 findings included.
5. **The new success summary is sound.** It is written only after all 15 chapters pass and the 458 listing is checked again, with mode `x`, as `preview516-summary.json` in the job folder.
   - It maps 01–08 to preview458's folder, receipt and listing.
   - It maps 09–15 to the new folder.
   - It gives every chapter's frames and file hashes.
   - It records `all_timelines_byte_identical`. That is the check 458 never reached, now made for all 15.
6. **Sources stay stable.** The guard hashes all input rows before and after the run, and 516 checks the 458 listing before and after. The one gap is 516 itself (L1).

## Finding

- **L1 (Low): 516's own hash is bound nowhere in the run.**
  - Its file is not a row of `preview458-inputs.json`, and that manifest must stay `5817cb64`.
  - The guard's receipt records the command path but not the hash.
  - The summary records neither the helper's hash, nor the binary's, nor the manifest's.
  - So the record cannot tell which 516 ran.
  - **Smallest fix:** add `helper_sha256=sha(__file__)`, `binary_sha256=BINARY_SHA` and `inputs_sha256=INPUTS_SHA` to the summary and to the first `say()`. Or root records `7547515c` in its launch note before starting.

## Notes (no change needed)

- **N1. A stop part-way leaves only frame counts in the log.** 458 failed under process pressure, and the pressure check or a render error can stop 516 after some chapters.
  - When that happens, the chapters already rendered are kept in the 516 folder, but their file hashes appear only in memory.
  - Each chapter's log line `say(chapter=c, checked=...)` prints only its frame count.
  - Printing the whole `record[c]` there would let a later recovery reuse those chapters by hash.
  - Any rerun needs a new job and helper, since the folder must not already exist.
- **N2. 12_both_promises is the first scored render of a split stem in this lane.** Its stem is `parts.tsv` plus 2 parts.
  - Both 516's `stems()` and the kit's `score_stem::read` and `finish` handle the parts.
  - The expected cue-log rows (part paths, start samples 0 and 5,760,000) follow from the kit's offsets.
  - This path has not run here before. If the rows differ, 516 stops after rendering 12 and writes no summary. That is a safe stop, not a false pass.
- **N3. The pressure check counts only this user's processes.** It checks against half of `kern.maxprocperuid` (10,666 here; `kern.maxproc` is 16,000), once before each chapter. Another agent's burst during a chapter can still cause the same EAGAIN failure. In that case 516 stops, as in N1.

## Not checked

- running 516, including `--check-only`;
- re-hashing or re-probing the eight movies;
- decode, pictures, sound, loudness or sync;
- the guard's behaviour beyond the lines cited;
- whether 516 is sealed beyond the hash above (CC2 had not yet confirmed at 06:27Z).

No audio heard or checked. This review accepts no picture, sound, sync, AV or release result, and it authorises no launch; root launches.
