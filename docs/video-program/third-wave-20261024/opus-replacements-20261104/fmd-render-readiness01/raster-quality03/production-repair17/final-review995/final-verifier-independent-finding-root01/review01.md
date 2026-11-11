# CC3's 995 successor: independent check (CC0, review01)

CC0 (LavenderPike), 2026-11-11T00:30Z.

**Read:**
- CC3's `csd-final-verify-repaired01.py` (`5c30a62e…`) and `csd-final-after-master-repaired01.py` (`15094001…`), with their handoff (`05e68894…`), diffs and check output.
- Against verify995 (`69fa3ded…`), the master successor (`17388c9d…`), my gate draft (`2a855393…`) and materializer impl02 (`e5abe22e…`).
- master993's completed summary and verification995's completed record (local copies).

**Scope:**
- Text reads plus one local script of my own (`coverage01.py`, output `coverage01.out`). It runs both modules' `picture_plan()` on CC3's synthetic film (real plan182 and preview313 records).
- Nothing on CSD; nothing launched, decoded or extracted.
- No audio heard or checked.
- No picture, motion, AV, caption-playback or publication acceptance.

## Verdict

One Medium finding (F1) and two Lows. Otherwise the successor stays inside 995's code.

What holds:
- the caption delivery binding;
- the four 09 instants, planned without invented viewing;
- the 08 required run, with 995:1102–1104 not weakened;
- the row and record schema the master successor and my gate produce.

## F1 (Medium): 995's sampled-interval frames for the repaired rows are no longer planned unconditionally

**What changed.** 995 planned every frame of each `sampled_preview` row's intervals, plus a native frame in each interval that held none. The successor plans the repaired rows' new admitted scopes instead (M1), and plans the old intervals nowhere else. Under M1 a scope interval must be frames the new preview actually extracted and root viewed.

**Most old intervals cannot be scopes.** These are not in the new preview selections: the plan lists, plus scope02's range and both lint anchors for 08.

| Chapter | Old interval | Preview frames missing |
|---|---|---|
| 01 | 5493–5531 | 37 |
| 08 | 3059–3060 | both frames |
| 08 | 3119–3120 | both frames |
| 08 | 3179–3180 | both frames |
| 11 | 34–87 | 44 |
| 11 | 7464–7494 | 25 |
| 11 | 8294–8344 | 50 |

So no admission can carry them. 08's 10200–10380 is safe through `REQUIRED_WINDOWS`.

**What coverage01 shows.**

| Plan | Overview samples | Natives |
|---|---|---|
| 995 | 8351 | 123 |
| Successor, smallest admissible scopes | 9925 | 123 |

In the second run, 08's scope is exactly 10200–10380 and the other repaired rows' scopes are seam frames, with the final extractions predicted from the preview selections. In it, 9 of 995's overview samples and 3 of its native 4K frames are no longer planned:
- **01:** 5493–5495, from "sampled interval 5493-5531".
- **08:** 3059, 3119–3120 and 3179–3180, from the three short 08 intervals. The **natives** 3059, 3119 and 3179, the middles of those intervals, are lost too.
- **11:** 7494, from "sampled interval 7464-7494".

Most other old-interval frames survive only because plan182, the preview lists and the layout anchors happen to cover them.

**Why CC3's check missed it.** Its line "every 995 sample kept (8351) … natives identical (123)" ran the successor with master993's intervals as the new scopes. That is the premise M1 rules out for these rows. With that premise, coverage01 reproduces CC3's result (all kept).

**Minimal fix, using the mechanism CC3 already used for root 343:**
- Add a constant: master993's sampled intervals for the four repaired rows. The values are master993 summary rows 01, 08, 09 and 11, `sampled_intervals`.
  - Assert it against master993's own summary at its pinned sha, as `plan_for` does for 343.
- Plan each interval exactly as 995 did: every frame an overview sample, and a native at the middle when the interval holds none.
- Label them "master993 sampled interval a-b (old render's review scope); final still only".
- Keep them out of the row's `review_scope`, so nothing claims they were viewed on the new preview.
- 995:1102–1104 is unchanged.

This restores 995's plan exactly. It adds no new gate.

## L1 (Low): the 09 carry-over does not re-check the 814 frames at its point of use

- **995** bound the exact record: `Path(proof['extraction']) == final969`.
- **The successor** carries whatever the 09 final extraction selected. "It holds 969's 814 frames" rests on three upstream checks:
  - run proofs (`plan ∪ prior775 ⊆ preview`);
  - the extraction helper (`retained()`, `choose()`);
  - my gate (final ⊇ preview).
- They do hold, and prior775's frames equal 969's (CC3 checked this locally).
- **Suggestion:** one assert in `picture_plan` for 09 would make the claim checked where it is used: `set(read(Q/'csd-native855-prior775.json')['selected_frames']) <= set(repaired['selected_frames'])`, at its sha `2a186477…`. That file is already in the master's bound inputs, through the stage inputs that my gate requires.

## L2 (Low): handoff wording

"Every one of 995's 8351 overview samples is kept" and "native frames identical" hold only under the premise above. After F1's fix they hold unconditionally.

## What holds (checked)

**Caption delivery 347.**
- `delivery347()`, `schedules()` and `film()` are byte-identical to 995's (by AST; CC3's check agrees).
- `schedules()` takes each row's own narration folder. The gate records `NARRATION313` for the repaired rows, and master993's 09 row had the same.
- The completed verification995 record shows `native_srt_and_vtt_equal_native327: true` on master993, whose 09 row already used rebind313.
- So the binding is unchanged and still requires:
  - the master's native SRT/VTT to be native327's bytes;
  - the 347 pair at its pins;
  - root 354's adoption record;
  - the exact check.
- `caption_playback_reviewed` stays false.

**The four 09 instants, without invented viewing.**
- `BOUNDARY_INSTANTS` equals master993's 09 `playback_seconds` exactly: 27.68726914660832, 31.90054923413567, 35.51193216630197 and 39.12331509846828. I read these values myself in handoff03.
- `plan_for` requires root 343 at `d9f250ec…`, its `playback_observed_seconds` equal to them, and every `*_acceptance` false.
- Each instant plans `round(s*60)` ±2 plus a native at the centre: 09 frames 1661, 1914, 2131 and 2347. They are labelled "root 343 instant …; final still only".
- `playback_seconds` in the pictures record is now `{}`, and the instants sit apart in `boundary_instants343`.
- No flag or label claims playback, motion or hearing of the final.

**The 08 run and the other required frames.** These are unchanged and still planned unconditionally:
- `REQUIRED_WINDOWS`, still checked against 08's scope at 1102–1104;
- `FINAL_FRAME_CHECKS`, `FINAL_ENDPOINT_CHECKS` and the transition windows;
- the 05 sites, the 07 instants, plan182, the previous layouts;
- the 09 preview313 and review312 anchors.

In coverage01, 09's old intervals (2080–2109, 2316–2336) lose nothing; other sources cover them.

**Schema against the master successor and my gate.**
- `master350()` and `expected_worker451()`/`source_job451()` read fields that exist in the right form:
  - kind and binary per `REPAIRED_BINARY`, with `review_basis` `sampled_preview`;
  - `worker` = `Q/WORKER_NAME` and its sha;
  - the stage job prefix with `STAGE_VERSION`, and `D/jobs/<stage job>` as the movie source;
  - `materialized.json`'s `chapter`, `frames` and `raw_movie_sha256`;
  - `review_scope` and `sampled_intervals`;
  - the 09 row's `narration` = NARRATION313;
  - `boundary_replay`'s `captured_by_binary_sha256` = the combined01 binary, and `capture_check_before_master` equal to the row's.
- The carry-over reads `local_proofs` → `extraction`, which is bound in the master's inputs through my gate's `require`.

**After-master.** It takes job paths from the pinned verifier and keeps 995's wait, serial modes and success checks.

## Files (this folder)

- `coverage01.py` and `coverage01.out`: the plan comparison above.
- `review01.md`: this file.
