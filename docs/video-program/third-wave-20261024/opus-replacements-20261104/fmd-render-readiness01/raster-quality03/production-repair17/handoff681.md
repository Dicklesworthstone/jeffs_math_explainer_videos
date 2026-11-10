# Handoff 681: guarded CSD final-picture extraction, apportionment 03–15 (CC0)

cc0, 2026-11-09, 22:55 UTC. Prepared only. **Nothing has run:** no SSH, build, render, launch or Beads change. Root owns the launch. D is `/data/projects/jmev-stroke-probe-20261106`; on the CSD, Q is `D/qualification01/production-repair17`.

| File | sha256 | |
|---|---|---|
| `apportionment-csd-pictures678.py` | `7a2942b6bd33765e40076294d88473505b531b900a2f097aacd3d95cf0ffa061` | 281 lines; modes `preflight`, `inputs CHAPTER`, `extract CHAPTER` |
| `apportionment-pictures-after-final679.py` | `60ee390f6a34e531855e884ed7e224d3fdf6b5ad42823c0f248bc52259cd0a64` | 86 lines; serial controller, pins 678 |

`apportionment-picture-support680.py` was not needed and does not exist. A 15-line loader inside 678 reuses support616.

## Prerequisites

1. **Copy 678 and 679 to Q on the CSD at the hashes above.** They are the only new transfer. They post-date export682, so 683 does not carry them.
2. **import683 must end in success.** It is not duplicated; 679 waits for it. 678 uses only `D/jobs/csd-apportionment-import683-a1/import-result.json`, after `success` and `stable`, and requires manifest `1c7c690b…` and 4,005 rows.
   - Checked locally against export682: **all 62 seed bindings are present at equal hashes, 16 in `files` and 46 in `reuse_from_prior_imports`.** Plan 677 and support616 are in `files`.
   - Physical paths exist only after the import. 678 re-hashes each one on the CSD.
3. **Tools.** The CSD last showed ffmpeg 8.0.1, Python 3.14 and Pillow 12.1.1 (`cc2-final-verify330-notes.md:284`, a read-only run on 8 November). `preflight` exercises Python, Pillow and both tool versions again. It does not exercise support616's `-/filter:v` option, which first runs during extraction; the local runs that used it were also FFmpeg 8.

## Root's commands (not run)

```
/usr/bin/python3 -I -B Q/apportionment-csd-pictures678.py preflight            # optional, read-only, after 683
/usr/bin/python3 -I -B Q/apportionment-pictures-after-final679.py              # waits, then extracts 03 to 15 in order
```

Each chapter becomes job `csd-apportionment-pictures678-CHAPTER` with manifest `Q/csd-apportionment-pictures678-CHAPTER-inputs.json`. Output is `CHAPTER/frame-NNNNNN.jpg`, `sheet-NNNN.jpg` and `sampling.json`, plus `pictures678-summary.json`, all inside the new job folder.

## How root's requirements are met

- **Successful stable finals.** `success` on the final672 receipt and on 664, 656 and 666; `stable` over the final's receipt and summary. The command must name final672 and the chapter, and every row of its input manifest is re-hashed.
- **Frames, cues, timeline.** Frames must agree across the summary, seed, `comparison666`, the timeline and the movie's packet count. `timeline.tsv` and `cues.tsv` must equal the bound timeline567 tables after replacing only the narration folder path.
- **Lint merge.** Every seed sample and reason is kept, and an assertion proves it. Each final `layout.tsv` row goes through `plan_chapter` after the seed's anchors, so seed ids do not move and a new flag gets first look, persistence, 2 s grid and span samples. `sampling.json` records the merge counts.
- **Bindings.** The guard manifest holds 678, 679, the final's manifest and all its rows, the final's receipt, summary and result files, import683's records, plan 677, support616, the 62 bindings, `comparison666`, `qualification`, and resolved Python, ffmpeg, ffprobe, Pillow's files and every library `ldd` reports.
- **Provenance.** Python, ffmpeg, ffprobe and Pillow versions and paths go into `sampling.json` and the summary.
- **Exact frames.** support616's `extract_frames` selects by decoded frame index and refuses a wrong count. Each picture and sheet is hashed in `sampling.json`.
- **Guard.** `guard_csd02.py`, called as 673 calls it. `extract` requires policy 10/8/4/56/20/10, the exact command, the frozen manifest hash, a fresh job folder and leadership of its own process group. It uses cores 0–3, as 672 does.
- **Failure.** A failed final, import, refusal or expired 24 h wait stops 679. There is no retry, and 679 refuses existing job or manifest names, so a new attempt needs new names. 678 refuses 01 and 02.
- **Import safety.** support616's only path that fails at a CSD location is `REPO = HERE.parents[6]` (line 49): Q has five parents. Its other import-time calls are `Path(...)` and `.resolve()`, and PIL loads inside `sheets`. 678 loads the pinned bytes and replaces that one line in memory.

## Limits

- **Checked:** compile and banned-string scan of both files, and an AST check that every support616 name used exists and no name is undefined. I traced the code by hand. **Neither file has been executed**, so the first real run is the first test of the merge, loader and manifest paths. Any defect should stop the run, but it would cost a new job name.
- **Lane.** The guard's `flock` blocks, so each extraction queues behind the running final and interleaves with 673 in no fixed order. Its manifest is frozen before it queues; admission and input checks run once it holds the lane.
- **Decode limit.** support616 allows 1,200 s per chapter. Chapter 12 (13,460 frames) needs at least 11.2 fps. Disk use is not measured. The seed has 1,044 samples for 03–15, before any new final flags.
- **Not checked:** the CSD's current state, import683's result, any final's output, and 673's progress.

No audio heard or checked. This accepts no picture, motion, sound, sync, AV or release result. Root views the frames afterwards.
