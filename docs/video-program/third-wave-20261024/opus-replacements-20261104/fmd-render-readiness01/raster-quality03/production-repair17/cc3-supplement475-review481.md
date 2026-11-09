# Review 481: CC2's supplemental helper 475 and its frame list (CC3)

CC3, 2026-11-09, about 05:10 UTC. Q is this folder. This is an independent review of:

| File | sha256 |
|---|---|
| `cc2-apportionment-supplement475.py` | `e2dbec70ba6e17811f7942109fbee8d67fa8a4d51b0b857e5722f35ef4b4ff90` |
| `cc2-apportionment-supplement475-frames.json` | `de89f991660ac899572c8f8e4d306ca03429032dd8f9c2871f04d2d96e9943df` |
| `cc0-apportionment-preview-review462.py`, the base it loads | `235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37` |
| `cc2-apportionment462-review470.md`, whose F1 it answers | `21fa8cab…` |

**Not done.** I edited nothing and launched nothing. No mode of 475 was run, no guard was touched and nothing was written under WORK. preview458 and pictures462 do not exist yet, so no 540p frame was read. No picture was judged and no audio played. This is no picture, motion, listening, fine-sync or AV acceptance.

## Verdict

- **475 closes 470 F1** at the scope of 470's own proxy.
- **The binding** to 462's own preview movies and frame numbers is exact.
- **No unsafe extraction or guard gap.** 475 runs only after a successful 462.
- **Findings:** three Lows, none blocking.

## Frame list: one independent recomputation

I wrote my own held-run detection (scratch `check481.py`, `c0dcc67b…`, output `check481-01.json`, `ca67bbd3…`). It reuses nothing from 475. The frames 462 takes come from 462's own `plan_chapter()`, because that plan defines what 475 supplements. The script decoded all 15 of timeline444's 64×36 movies once at `nice 19`, read-only. timeline444 is terminal: exit 0 at 04:55:22Z, inputs equal to final inputs.

| Check | Result |
|---|---|
| timeline444's timeline, cues, layout and movie sha256, all 15 chapters | equal to the list's pins |
| 462 plan per chapter | equals `plan462_samples` (01–13 equal 470's counts; 14: 62, 15: 82) |
| Held runs of 1 s or more | 01–13: 309, as 470 found; all 15: 379 |
| Runs with no 462 frame inside | 137 (105 in 01–13, 32 in 14–15), frame for frame the list's `held` entries, with the same first and last frames |
| 467's 08 frames 3450 and 5738 | listed and not in 462's plan |
| Total | 139 = 137 + 2, as pinned |

**F1 coverage.**
- All 50 of 470's F1 frames are in the list, including 10_5's 4526–4884 hold at its middle, 4705.
- The other 55 in 01–13 are the held pictures 470 found matching a planned picture at 64×36. They are redundant at that size but cost one frame each.
- 14 and 15, which 470 could not check, add 27 and 5.
- The proxy's limits stay as 470 states them: a change under one 64×36 pixel does not register, and holds under 1 s are not listed.

## Binding and guards (source)

**Same preview and same frames.**
- `build()` requires successful, stable receipts for timeline444, score445, preview458 and pictures462. 462 must have run from its own inputs (`INPUTS` hash equals 462's receipt).
- Per chapter:
  - each preview timeline is byte-equal to timeline444's and to the list's pin;
  - each 462 `sampling.json` is at the hash 462's summary records;
  - its `movie` record equals `{path: PREVIEW/<c>.mp4, sha256: <the movie now>}`. 462 writes exactly that pair (`462:354`), so the match is exact;
  - 462's `probe()` passes (960×540, 60/1, packets equal to frames);
  - no listed frame is one 462 took.
- Frame n is the same scene frame in both movies because the timelines are byte-identical. 470 showed `select=eq(n)` with passthrough gives the decoded picture at index n.

**Extraction.**
- 475 calls 462's own `extract_frames()` and `sheets()` with the fields they read: `frame`, `seconds`, `reasons`, `path`.
- Every write goes into a fresh job folder and refuses an existing name: `mkdir` without `exist_ok`, ffmpeg `-n`, a rename check, and `sampling475.json` and `pictures475-summary.json` opened with `x`.
- Nothing is written into 462's job. The output is 139 pictures and 24 sheets at 540p.

**Guard.**
- Same guard and pins as 462 (`4f34e0d1`, `1c19a9b8`, `819746fb`), so it uses the same lane and work folder.
- It runs as its own job `apportionment-frederick-pictures475`. Its inputs are written once, with `x`, after `inputs` checks that neither the inputs nor the job exist.
- `extract` makes 462's guard-child checks unchanged: admitted, unfinished, native, the command ending `-I -B <475> extract`, the receipt's inputs hash, a fresh folder.
- It then re-binds every file `build()` reads, plus 475, the list, 462, the guard chain, the preview helpers, ffmpeg and ffprobe.
- The runner re-hashes the manifest at the end.

**Python.** `inputs` must be run with the Homebrew Python 3.15, as 462's is, because the guard command uses `sys.executable`. It has numpy 2.5.4 (needed only by `inventory`) and Pillow 13.0.0. The system Python 3.9 has neither and fails 475's 3.11 check first.

## Findings

**L1. 475's list was computed from timeline444's layout rows; 462's real plan also merges preview458's rows (`462:253`).**
- If preview458 adds a layout row that timeline444 lacks, the real plan gains a 2 s grid. `plan` then refuses with "462 already took…" wherever a grid frame equals a listed frame.
- This fails safe, but it would block until the list is re-pinned. An identical row merges without change.
- **Suggestion:** run `plan` right after 462 succeeds, before `inputs`.

**L2. `build()` re-checks only each timeline's pin and frame count.**
- The cue, layout and 64×36 movie pins the list was derived from are re-checked only by `inventory`. They hold today (table above).
- **Suggestion:** if root wants that proof at launch time, run `inventory` once before `inputs`. It is read-only and decodes the 15 small movies.

**L3. Each entry's `nearest_462_level` and reason text measure the run's first frame, not the listed middle frame (`475:96`).**
- A slowly drifting run (each step at most 2 levels) can label its middle frame wrongly. This affects the label only, never which frames are taken.
- I did not measure the drift.

## Order for root

1. 462 succeeds.
2. `plan` (L1), and optionally `inventory` (L2).
3. `inputs`.
4. The printed guard command, once.
