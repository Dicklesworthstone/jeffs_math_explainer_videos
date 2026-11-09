# Review 470: helper 462 and controller 468 against the real timeline444 (CC2)

CC2, 2026-11-09, about 04:50 UTC. This is an independent review of CC0's frozen helper and controller. It covers the timeline444 chapters finished by then, 01–13. Q is this folder, WORK the five-film work folder.

**Nothing was changed or launched.**
- No file of 462, 468, root, CC0 or CC3 was edited.
- Controller 468 (PID 55014) was left alone. Its last log line still reads all three receipts as pending, with nothing launched.
- Nothing was written under WORK.
- The only movies read were timeline444's own 64×36 renders of 01–13. They were decoded read-only at `nice 19`, using 462's own `extract_frames()` loaded from its Q path, with output kept in scratch.
- No picture was judged, and no audio was played. This is no picture, motion, audio, sync or AV acceptance.

| Input | sha256 |
|---|---|
| `cc0-apportionment-preview-review462.py` | `235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37` |
| `cc0-apportionment-pictures-after-preview468.py` | `6b08e2c0bc5a9fc271d660d73d428b9869f083220fc06707a2f9c618574234a5` |
| `cc3-apportionment-layout-review467.md` | `065362e8392010abcc99bae127beed30e1fcba87f299d97006d5827121d35189` |
| timeline444 `12_both_promises.timeline.tsv` / `.layout.tsv` | `8340f8a4…` / `fd5bcaac…` |

## Verdict

- **Serious problems:** none. The selection, the decoding, the guards and 468's gates hold on every real file there is so far. Nothing needed to be raised with root early.
- **One coverage finding (F1, Medium).** It does not affect the frames 462 does take, and it does not need any change to 462 or 468.

## F1 (Medium): held pictures during speech that no sample shows

- **What 462 takes:** every silent hold, and per spoken cue four frames: the start, +0.5 s, the middle and the last spoken frame.
- **What it misses:** a picture that builds in stages during a long cue and holds each stage is sampled only if one of those four frames lands in it. Chapters with a layout flag get a 2 s grid and are dense; the other chapters get only these four frames per cue.
- **How this was measured:** a "held picture" is a run of 1 s or more with no 64×36 pixel change (at most 2 of 255 levels, frame to frame). It counts as distinct when it differs from every planned picture of its chapter by more than 24 levels.

| | 01–13 |
|---|---|
| Held pictures of 1 s or more | 309 |
| … with a planned frame inside | 204 |
| … not inside, but matching a planned picture | 55 |
| **… not inside, and differing from every planned picture** | **50** (105 s in all) |
| … of those, in a silent hold | 0 |

- **The worst case:** `10_rounding_rules` frames 4526–4884, a 6.0 s hold in 10_5. It falls between the phrases "it rounds up once a quotient passes" and "it agrees with Webster", which is the square-root picture of the Huntington–Hill rule. 462's 10_5 frames are 3788, 3818, 4453 and 5117.
- **The other long ones:** 10_7 at 7916–8229 (5.2 s) and 10_6 at 5698–5912 (3.6 s).

**What to do (root's call).** If those pictures are to be looked at, a small supplemental extraction can take them from the same preview458 movies after 462 finishes. It needs no change to 462 or 468. Taking the middle frame of each run gives these 50 frames:

| Chapter | Frames | Cues |
|---|---|---|
| 01 | 4366 4508 | 01_6 |
| 02 | 176 281 1219 1441 1799 2623 2727 4294 4549 4844 5407 5484 6078 | 02_1–02_3, 02_5–02_7 |
| 03 | 4753 4872 | 03_6 |
| 04 | 151 3437 3580 | 04_1, 04_5 |
| 05 | 346 892 2304 2440 4173 | 05_1, 05_2, 05_4 |
| 06 | 2065 3872 | 06_3, 06_5 |
| 09 | 2766 | 09_3 |
| 10 | 1442 1548 2710 2820 3993 4107 4705 5371 5522 5805 6352 6486 7832 8072 | 10_2, 10_4–10_7 |
| 12 | 6563 10238 10492 | 12_5, 12_8 |
| 13 | 175 304 897 1078 1446 | 13_1, 13_2 |

The same check can be rerun on 14 and 15 when timeline444 finishes. 07, 08 and 11 have none.

**Limit.** This proxy is coarse. A change smaller than a 64×36 pixel does not register, and a "held picture" is not necessarily an authored `pause()`. It can find pictures that are missing from the plan. It cannot show that a picture is covered at 540p.

## Coverage verified on the real files (01–13)

**The plan.**
- 462's own `plan_chapter()` gives 934 samples: 48 / 55 / 48 / 41 / 41 / 48 / 41 / 115 / 41 / 55 / 93 / 195 / 113, for 01 to 13.
- In every chapter it equals my independent rebuild from the notes' rules, frame for frame. No sample is skipped.
- 13 chapters, 84,755 frames, 80 cues.

**Silent holds.**
- There are 93 in all: 13 lead-ins and 80 holds after cues, including the end holds. Each gets its first, middle and last frame.

**Predict holds.**
- There are 13, one per chapter. Each mark is 36–37 frames after its cue ends, so it is inside the hold, as `captions.rs` requires.
- In every chapter the next cue starts at mark + 266. That is `think()`: a 13-frame fade, a 240-frame drain and a 13-frame fade.
- Every window is sampled at +0, +18, +114/115 (the hold's middle), +120, +265, +266 and +296.
- CC3's label notes (467) hold for all 13 windows, not just 08 and 11. "+0.3 s ('?' faded in)" is 5 frames into the drain, and "+2.0 s (PREDICT_HOLD_S)" shows a half ring. Only the labels are wrong; the frames are right.

**Layout flags.**
- There are 11 rows, all of them anchors: 08 has 4, 11 has 1, 12 has 4 (12_1 `edge` @1.8 s, and three 12_3 `crossed` @54.0 s) and 13 has 2.
- 13's two rows are checkpoint row "before 13_3 @25.6s" `overlap` and 13_5 `edge` @56.2 s.
- 462 parses both row forms, so new flags enter the plan as they appear.

**Sample cap.**
- With 934 samples for 01–13 and 13 of 93 cues left, the 2,500 cap cannot be reached. Even a flag at frame 0 in every remaining chapter adds only about one sample per 2 s of film plus a few per cue.

## Exact decoded-frame sampling (real timeline444 movies, 64×36)

These movies come from the same binary as preview458, with the same encoder settings: 60 fps, `--crf 16 --preset slow`. Their B-frames use 2 reorder frames.

- In all 13, ffprobe reports `avg_frame_rate` as 60/1 with `nb_read_packets` equal to the timeline's frames, so 462's `probe()` gate passes on this output.
- The decoded frame count equals the packet count. Every decoded frame's timestamp is n/60, starting at 0.
- 462's own `extract_frames()` ran on each movie with 462's own plan. All 934 pictures are byte-identical to a full decode's picture at the same index, made with the same flags.
- Pillow 13.0.0 imports under the pinned Python 3.15.0 with `-I`, and `load_default(size=15)` works.
- ffmpeg 8.1.2 accepts `-/filter:v`.

Not covered: 540p itself, and the audio track. 462's own probe re-checks the 540p movies at run time.

## Guards and 468

- **Guard chain:** read at its pins. The policy is 10 GiB native admission, 8 GiB stop, 4 GiB growth, an 80 GiB work cap and 20 GiB target growth. The guard's WORK is the folder 462 reads. Loading the guard in-process has no side effects.
- **Headroom:** at 04:45Z, timeline444's receipt shows 179.9 GiB free and 63.4 GiB of work, under the 80 GiB cap.
- **Receipts:** the base runner's terminal receipt carries every key 462's `success()` reads (`finished`, `exit`, `error`, `stopped_for_resource_limit`, `final_inputs_sha256`, `preflight_only`). `extract()`'s guard-child checks match what the runner writes before it starts the child.
- **468's order:**
  1. pins, Python, absent outputs and duplicates;
  2. a stable double read of preview458;
  3. 462's `build()`;
  4. one input manifest, written once;
  5. one guard launch, with no retry.

  Any failure stops it with nothing launched.
- **Duplicate check:** the only matching process is 468's own `run` (PID 55014), which the check excludes as itself. No 462 `inputs` or `extract` process exists. My checks ran under their own script names and match neither pattern.

## CC3's fourteen frames (467): is a supplemental extraction needed?

Each frame below is compared with the nearest frames 462 plans, at 64×36 on timeline444.

| Group | Frames | Why |
|---|---|---|
| Already shown by an identical planned picture | 3211 (= 3216), 4974 (= 4982), 5616 (= 5640), 5711 (= 5712), 5766 (= 5760 and 5880), 11: 831 (= 864) | No pixel differs. Source agrees that nothing animates between them. |
| In motion, so a still cannot settle them | 3457 (ring fading), 3482 (card 15 fading in), 4932, 4944, 5688 (the cutoff sliding), 11: 793 (citation fading in) | Each differs from its neighbours frame to frame. |
| A short state not in the plan | 3450 (the shortest arc, last drain frame; 37 levels from 3463) and 5738 (the "=" flash peak, still at its turning point; 14–16 levels from 5716 and 5760) | Each is the extreme of a brief motion: the end of the 4 s drain, and the peak of a 0.9 s flash. |

**For the five rows, no.**
- The held states of every row are already planned.
- Of the fourteen, only 3450 and 5738 would add a picture, and each is the extreme of a brief motion.
- If root runs a supplemental extraction for F1 anyway, adding these two costs nothing.

## Scratch evidence (outside the repo)

| File | sha256 |
|---|---|
| `check470.py` (plan, independent rebuild, timestamps, 462's extraction, held pictures, CC3's frames) | `17c647e8…329f1c` |
| `check470b.py` (byte identity against a full decode) | `c0f8c43c…2a2077` |
| `check470c.py` (held pictures no sample shows) | `deb6aeeb…28c5` |
| `check470d.py` (all of the above for 13, into a new folder) | `b259fb30…60d6b2` |
| Outputs: 470-01 / 470b-01 / 470c-01 / 470d-01 | `f747dee3…` / `b427b803…` / `85cd50fc…` / `95ec6b7a…` |
| `supplement470.json` (01–12 list) / `report470d.json` (13) | `d29eb30f…` / `9ca3c206…` |

The first alignment test in `check470` compared each JPEG with its raw neighbours. It flagged ties that were JPEG noise, under 0.01 levels apart. Its output is kept, and `check470b`'s byte comparison replaces it.

## Not done

- No preview458 frame was read: it does not exist yet.
- Chapters 14–15 were not checked.
- Nothing was judged by eye.
- No change was made to 462, 468 or any other file.
