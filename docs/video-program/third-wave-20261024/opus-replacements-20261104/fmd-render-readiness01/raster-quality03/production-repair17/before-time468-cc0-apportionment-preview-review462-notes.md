# Helper 462: picture samples from apportionment preview458 (CC0 notes)

cc0, 2026-11-09. Q is this folder.

`cc0-apportionment-preview-review462.py` (sha `235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37`, 410 lines) is prepared only. It has not been run under the guard, and no extraction has run. No inputs file or job folder exists.

The frames it extracts are material for a person to look at. They accept no picture, motion, sound, fine-sync or AV result. Narration stays the retained ElevenLabs Frederick takes; the helper reads the preview's movies with `-an`. No audio heard or checked.

## What it requires before it extracts

- **Receipts:** timeline444, score445 and preview458 each finished with exit 0, admitted, not preflight-only, no error or resource stop, and inputs equal to final inputs. All three are read twice, 15 s apart, with identical bytes.
- **preview458's inputs file** is the one its receipt hashed.
- **The guard chain and root's preview helpers** are at their pins: guard 80g `4f34e0d1`, 56g `1c19a9b8`, base runner `819746fb`, preview458 `7ab178d1` and controller458 `3a7c4b0c`.
- **Chapters:** `preview458-summary.json` says `all_timelines_byte_identical`. Its 15 chapters equal timeline444's summary.
- **Timelines:** every preview timeline is byte-equal to timeline444's, and its frame count equals timeline444's.
- **Cue tables:** each cue table names the timeline's cues in order, and each cue starts at its timeline frame.
- **Movies:** each preview movie is 960×540 at 60/1 with as many packets as its timeline has frames (ffprobe `-count_packets`).
- **Chapter 08** has layout anchors at 0.8, 53.4 and 93.2 s.

## What it samples per chapter (all from the actual files)

| Source | Frames |
|---|---|
| chapter | first, last |
| each cue (timeline start, cue-table audio length) | start, +0.5 s, middle, last spoken frame |
| each hold (lead-in, every gap, end hold) | first, middle, last frame (before the next cue) |
| each prediction hold (`mark predict` between a cue's end and the next start, as `captions.rs` reads it) | mark, +0.3 s ("?" faded in), +2.0 s (`PREDICT_HOLD_S`), last frame before the answer cue, its start, +0.5 s |
| each layout row, preview458's and timeline444's (identical rows merged, sources kept) | first look, +0.2 s, +1.0 s (the lint's persist window), +2.0 s, every 2 s of the chapter grid through the chapter end, the last frame, and the end of the span the "later looks" would cover if consecutive, plus 1 s after it |

**Merged frames.** When several reasons land on one frame, the frame keeps them all, in order. A sample past the chapter end is kept under `skipped` with its reason.

**Later looks are an estimate.** The lint counts sampled looks every 0.2 s and checkpoint looks (`layout.rs`). A flag may clear and return, so the "consecutive" span end is an estimate only. The 2 s grid is what shows persistence.

**Dry run** (`plan --dry-run-timeline444`, read-only) on the timeline444 files present at 2026-11-09 ~05:00Z covers chapters 01–11, 626 samples:
- **08:** 115 samples. The anchors are 08-L1 edge @0.8 s (535 later looks), 08-L2 crossed and 08-L3 overlap @53.4 s (18 and 19), and 08-L4 tight @93.2 s (67). The predict hold runs from frame 3198 to 08_4 at 3464.
- **11:** one new row, `11_2 +2.3s @13.4s tight` "Balinski & Young 1982, via Gölz," at 0.23 from the right edge (155 later looks). It is sampled the same way.

## Commands (root, after preview458 succeeds)

```
python3 -I -B Q/cc0-apportionment-preview-review462.py plan [--show 08_signposts]   # read-only
python3 -I -B Q/cc0-apportionment-preview-review462.py inputs                        # writes the manifest once, prints the guard command
<homebrew python> Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-pictures462 \
    --inputs <generation89>/pictures462-inputs.json -- <homebrew python> -I -B Q/cc0-apportionment-preview-review462.py extract
```

**`inputs`** binds the following, refusing if the file or the job already exists:
- the helper, the guard chain and root's two preview helpers;
- ffmpeg and ffprobe (resolved);
- every file under the preview458 job;
- the timeline444 summary, timelines, cue tables and layout rows;
- the three receipts.

**`extract`** runs only as the guard's child. It requires:
- an admitted, unfinished native receipt whose command ends `-I -B <helper> extract`;
- that receipt's inputs hash equal to the manifest;
- a fresh job folder;
- every file it reads bound in the manifest at its current hash.

Then it decodes each preview movie once with `select=eq(n,…)` and `-fps_mode passthrough`. Per chapter it writes:
- `frame-<frame>.jpg` pictures;
- `select.filter`;
- `sampling.json` (frames, seconds, all reasons, anchors, holds, predicts, skipped, hashes, movie hash);
- 8-picture sheets labelled with frame, seconds and reasons.

It ends with `pictures462-summary.json`.

## Fixture (scratch file outside the repo; synthetic movies, nothing launched)

`test462.py` (sha `cc74e35b…`), output sha `55928f18…`: **35/35 pass on the bytes above.**

**Selection:**
- On a B-frame clip with several GOPs, the selected pictures are byte-identical to a full decode at those indices.
- On the sheets, testsrc2's burned-in counter equals each labelled frame.

**End to end:** a fake 15-chapter tree, with chapter 08 carrying the real timeline444 timeline, cues and layout, runs `build`, `inputs` and `extract`.
- 08's plan equals the dry run on the real files.
- Merged reasons survive; frame 6447 carries 6.
- Two 08 pictures equal an independent extraction.

**Refusals:**
- a failed receipt;
- a summary without identical timelines;
- a differing timeline;
- a cue-start mismatch;
- a layout row without a time;
- 08 without 93.2 s;
- 14 chapters;
- a movie one frame short;
- an extract that is not a guard child, has another manifest, a stale job folder, or a file changed after the manifest;
- a second extract.

**Not tested:** the real preview458 (it does not exist yet), the guard and its lane lock, and real 540p sizes. The plan has a cap of 2,500 samples, and the dry run suggests about 800 for all 15 chapters.
