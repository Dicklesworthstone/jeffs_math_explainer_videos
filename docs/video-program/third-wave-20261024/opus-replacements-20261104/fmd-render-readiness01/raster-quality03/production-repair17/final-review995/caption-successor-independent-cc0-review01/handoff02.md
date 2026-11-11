# English captions for the successor master: CC0 check (handoff02)

CC0 (LavenderPike), 2026-11-10T23:32Z. This file supersedes handoff01.md (`c668bfde…`, kept). It corrects the status of the counting02 schedule: counting02's timeline job has **not run**, and its controller is waiting on the combined preview.

**Scope:**
- Only text and JSON reads plus two local scripts; no media was processed and nothing was launched or edited.
- No new captions were authored.
- No audio heard or checked.
- This is not caption-playback, AV or publication acceptance.

## Upload these (English)

| File | sha256 | Captions |
|---|---|---|
| Q/cc1-caption-repair347-candidate.en.srt | `0ea9277e6d598540e5e9057a84ee84daf7f88420b076434659fc3d3ad0ff3230` | 579 |
| Q/cc1-caption-repair347-candidate.en.vtt | `6a383cb7f207be81254e9a820ecb701ca77d00a26557194c675dc1717ef6979b` | 579 |

This is CC1's 347, adopted by root's review354 (`749aeba8…`) and exact-checked by verify995 against master993's schedule.

**Do not upload complex-analysis-master993.en.srt/.vtt.** Those are the 571-caption native export (`5d064232…` / `26e0d7e0…`, the same bytes as native327). They are the binding reference only.

## Successor binding: established vs conditional

**Established:**
1. **Narration inputs are identical.** The 347 tool pins the rebind269 report (`b4a4c91e…`); counting02 delivery reads the rebind313 report (`69cb32ee…`). All 461 sentence rows of the two have the same `sentence`, `out_samples` and `gap_after_samples`, at rate 24000. Only `meta.direction_contract` and `units_sha256` differ. (report-compare01.out)
2. **combined01's actual 01/08/09 timelines** passed root's exact normalized comparison (combined-timeline-terminal01). Against those actual tables:
   - every cue start equals the first caption start in both the 571 and the 579, within 1 ms;
   - the 01 and 09 tables equal the 347-pinned timeline269 tables row for row, except for the narration folder;
   - 08 is bound by its 5 cue starts, because its pinned table (`e2276be4`) has no local copy.

   (cue-binding01.out)

**Conditional (not yet proven):**
3. **Chapter 11.**
   - counting02's timeline job (timeline02) has not run, so its exact-table assertion is still required: the normalized 11_counting table must equal the timeline269 table `29a3afd3…` at 11932 frames.
   - Root's 11_counting source changes are timing-neutral by source reading, and the delivery keeps all 105 WAVs. That predicts the schedule is preserved but does not prove it.
4. **01/08/09 if rendered by the counting02 binary.**
   - bundle03 carries the same the_bump, loop_sums, mod and boundary sources as combined01 (`90b51c2b`, `a8d62f56`, `4de1e4a5`, `4b689791`). That predicts the same 01/08/09 schedule.
   - timeline02 checks chapter 11 only. If the successor master takes 01/08/09 from counting02 renders rather than combined01's, their actual tables need the same check.

**Rule for the successor:**
- Upload the 347 pair unchanged only if all of these hold:
  - the chapter 11 exact-table assertion passes;
  - every 01/08/09 render the successor uses has a passed actual timeline check;
  - the other 11 chapters reuse master993's rows.
- The successor's verify must repeat verify995's `delivery_captions_347.exact_check` on its own schedule.
- Its native caption export should equal `5d064232…` / `26e0d7e0…`. If any of this fails, do not upload 347. Re-derive with cc1-caption-repair347.py into a new prefix with re-pinned inputs; its positive control must reproduce the new native stamps.

## Splits, sentence boundaries, readability

**Already done (not repeated):**
- CC1's 347 applied the rules C1, C2 and L1–L3 to all 461 sentences.
- Root's review354 read all 163 overrides and ran its own checks.
- verify995 on master993 found the SRT equal to the VTT and the text equal to the script per cue. No caption is over 20 chars/s and no line is over 42; the last caption ends at 2220.147 s.

**Added here:**
- All 105 cues keep the same first start and last end in the 571 and the 579.
- The 8 extra captions come from 12 cues:
  - 9 cues gain captions (08_5 gains 3);
  - 3 cues lose one, from the answer merges.
- Captions that run across a sentence end: 0 of 571, and 3 of 579. The 3 are exactly review354's accepted merges: #64 (03_4 "No."), #390 (10_5 "No.") and #426 (11_6 "One.").

**Still open:**
- Caption playback on the 4K picture.
- Split times inside a sentence are character-share estimates, not voice timing.
- The visual context of 04_6's two-caption question.

## Keep the two receipts apart

- **completed-master993-summary.json** (written at master time) records `full_decode`, `final_picture_review`, `motion_review` and `audio_heard` as false.
- **completed-verification995.json** (later; receipt exit 0, finished 19:27Z) is technical only:
  - a full software decode of video and audio with 0 errors;
  - 133662 frames identical to the staged chapters;
  - framemd5 timing;
  - loudness of −19.0 LUFS at −1.5 dBTP, LRA 4.1 (fixed gain at the true-peak ceiling);
  - exact native captions;
  - the 347's exact text and screen.

  It still records false for pictures, motion, hearing, fine sync, 4K detail, independent AV, publication and caption playback.
- It supersedes summary993's `full_decode` false for the technical decode only. It changes no acceptance field.
