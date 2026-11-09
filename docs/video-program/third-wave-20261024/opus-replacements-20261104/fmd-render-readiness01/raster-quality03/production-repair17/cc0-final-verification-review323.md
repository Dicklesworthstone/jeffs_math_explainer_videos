# Review 323: final verification 321 (CC2)

cc0, 2026-11-08. Independent source review of CC2's 321 handoff, against the real master 318 output contract and the existing verifier chain.

I did not edit any CC2, CC1, CC3 or root file, or my own 318. I launched nothing and ran no job or decode on any host. I wrote no permit and made no Beads, TODO or canonical edit. **No audio heard or checked.**

Paths use the helpers' own names: D is the CSD root, and Q is `D/qualification01/production-repair17` (locally, this folder). `:N` is a line of `csd-final-verify321.py` at `0a956ec8` unless another file is named.

## Files reviewed

| File | sha256 | Read |
|---|---|---|
| `Q/csd-final-verify321.py` (682 lines) | `0a956ec8c0613971ad33e90a0a9eb4a496ba42bd2cb06d8044105397bfc5b7fb` | in full: the handoff the notes name |
| `Q/cc2-final-verify321-notes.md` | `c7e537a4b511193164648cfd0fa61ab612a1df0aa20afa038bcbfc24d3098a91` | in full. It changed once after it appeared (from `22baa69e…`); the change is line citations only. |
| `Q/cc0-combined-master318.py`, the contract 321 reads | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | unchanged. Root's source review `root-master318-review305.json` (`c1619b8b…`) has no findings. |
| `Q/cc0-master318-notes.md` | `423132ff…` | unchanged |
| shared `verify_stage09_final.py` | `f266a231…` | in full |
| `verify_video_preserve_logs01.sh`, the strict script | `3f4c68db…` | in full |
| `caption-interval-audit01.py`, the caption screen | `3cc26436…` | in full |
| `Q/final-picture-plan182.json` | `20a36f91…` | data checks |
| base: `Q/trj-final-verify19-02.py`, `Q/trj-final-pictures182.py`, `Q/trj-pictures-after-verify182.py` | `35578152…`, `0f9b2de7…`, `86e8224b…` | in full |
| `Q/review-repair-preview313.py`, which writes the preview313 records 321 reads | `67d780d8…` | the writers 321 depends on |
| `crates/explainer_kit/src/master.rs` (repository copy) | `7d1952db…` | `join()`, :193–251 |
| earlier CSD verifier: `fmd-render-readiness01/final-verify05.py` and `launch-final-verify05.py` | `6849ac34…`, `c61f9a2f…` | for comparison |
| `Q/cc1-master-review322.md` | `71a6aefc…` | its F1 and N2, where they touch 321 |

**Coordination.**
- 321 changed four times while I prepared: `611545c7` → `12ddcdd3` → `2dc51c9d` → `0a956ec8`.
- I read `2dc51c9d` line by line and diffed it against the handoff. The handoff adds one docstring line (:20–21) and `check`'s growth estimate (:666–672). Nothing else changed.
- Everything below is against `0a956ec8`. I re-hashed both 321 files immediately before writing this.

## Result

**No High. One Medium (M1) and four Low (L1–L4), all in the picture plan or the launch path.**

The verification gates are sound against 318's actual output contract: decode, frame identity, schedules, captions, rates and loudness. I found no way for a wrong master to pass them.

## Findings

### M1 (Medium). Root's frame-by-frame transition windows (305 and 319) are not in the final picture plan

**What exists.** Root viewed every encoded frame of 31 windows in preview274's chapter movies. CC1's 319 covers the 05 windows, and root accepted it. These are the only frame-window records in Q.

| Chapter | Windows, chapter-local frames, inclusive | Frames | Record (sha256) |
|---|---|---|---|
| 01_the_bump | 5496–5531, 6273–6310 | 74 | `bump-transition-review305.json` (`bc48a64d…`) |
| 04_reading_color | 235–311, 6742–6773 | 109 | `color-transition-review305.json` (`fdd04eab…`) |
| 05_gallery | 34–98, 1989–2072, 2894–2977, 4772–4855, 6181–6264, 7312–7365, 8715–8859, 9168–9220, 9817–9878 | 715 | `gallery319-disposition305.json` (`de33e72e…`); sites C1–C17 in `cc1-gallery-transition-review319.md` (`d0df7480…`) |
| 07_one_point | 2216–2280, 4744–4778 | 100 | `one-point-transition-review305.json` (`02e8594d…`); windows in `one-point-transition-frames305/manifest.json` (`e6fa64dd…`) |
| 10_rigidity | 3735–3799, 6241–6335, 7410–7441 | 192 | `rigidity-transition-review305.json` (`ac897e91…`); manifest `ba0eb53d…` |
| 11_counting | 34–87; 7464–7493, 8294–8344 | 135 | `counting-opening-review305.json` (`5e1f5552…`), manifest `9b889e04…`; `counting-transition-review305.json` (`a8552f24…`) |
| 12_residues | 2475–2516, 2536–2577, 2763–2804, 3025–3154, 3911–3942 | 288 | `residue-transition-review305.json` (`b3a90d15…`) |
| 14_continuation | 34–98, 1223–1257, 8876–8940, 10179–10300, 11724–11755 | 319 | `continuation-transition-review305.json` (`1a92e0f7…`) |

There are also root's five 07 playback instants: 80.96, 81.20, 84.98, 89.25 and 94.69 s, nearest frames 4858, 4872, 5099, 5355 and 5681 (`open-region-playback-second305.json`, `8ede28c4…`). These are screenshots taken during real-time playback, so each frame is approximate.

**Root's own requirement.** `gallery319-disposition305.json` says: "Low-amplitude preview compression and static edge contacts must be checked in final4K."

**The frame numbers carry over.** Every window manifest names `preview274-review305/preview/<chapter>.mp4`. Those are binary 268's renders (`983954c5…` in its `summary.json`) at 960×540, 60/1. Each of the nine has exactly its timeline21 table's frame count. That is the schedule 318 requires of every final (318 `schedule()` :354–384), so a window frame is the same local frame of the final chapter.

**What 321 samples.** I ran 321's own `picture_plan()` (:351–423) on local stand-ins (see "Local checks"):
- Of the 1,932 window frames, 81 are sampled (4.2%).
- No native 4K frame falls in any window. Chapter 05 gets 8 native frames, none of them in a 319 window.
- None of the five 07 instants is sampled; one has a sample within 6 frames.

**Why it matters.**
- These windows were checked frame by frame because the faults they look for last a frame or two: a one-frame flash, a stale plate coming back, a plate edge contact. Sampling every 10 s and at layout anchors cannot see them.
- 321's framemd5 identity (:503–520) transfers any review of the final chapter renders to the master exactly. But no review of those 4K renders exists. These reviews were done on previews, and a final's permit is written before its render (318 `permit_gate`, :338–351).

**Fix.**
- Add a constant like `PREVIOUS_LAYOUT_SOURCES` holding the records above at their hashes, with the windows and instants.
- Add every window frame to `samples`, labelled with its record and window, and each 07 instant ±2 frames.
- Add native 4K frames in each window: at least one, and for 05 one full-plate frame per 319 site (C1–C17).
- Cost of the overview frames: +1,932 frames and about +242 sheets. At the lane maxima `check` uses (78 KB, 897 KB), that is about +0.35 GiB.
- Cost of the native frames: one per window adds 31 × 24.9 MB at raw RGB, about +0.72 GiB. With the notes' F4 upper bound of 2.25 GiB, the total is about 3.3 GiB, still under the 4 GiB phase cap. Three per window would push the raw-RGB upper estimate past the cap, though measured PNGs are far smaller.

### L1 (Low). framemd5's timestamps are discarded, so whole-frame join timing rests on a rounding margin

**Where.**
- `frame_hashes()` (:438–447) keeps only the hash column.
- The shared `media_facts` counts packets exactly. But it allows the video's duration to be within `1/FPS` of the timelines' total.

**What I measured.**
- A one-frame timestamp gap at a join leaves the packet count right and makes the video last one frame longer.
- I called the shared `media_facts` with such a probe: 133,662 packets and a video lasting 133,663/60 s, which ffprobe prints to 6 decimals as 2227.716667.
- It refuses, but only because 2227.716667 − 2227.7 exceeds 1/60 by about 3×10⁻⁷ s.
- Audio lengths are checked only against the total.

**Why it matters.** The kit's own join comment (master.rs :197–202) names gaps at joins, and dropped or misordered frames, as the failure modes it was written against.

**Fix.** framemd5 already prints the timebase, pts and duration. I confirmed `#tb 0: 1/60`, pts 0, 1, 2 and duration 1 with ffmpeg 8.1.2 on the local 08 render. Assert the master's `#tb 0: 1/60`, pts equal to `range(133662)`, and every duration 1. This makes the join timing exact, independent of rounding.

### L2 (Low). No free-space check before the inputs file spends the version

**What happens now.**
- `launch()` writes the inputs with `'x'` (:643) and only then calls the guard.
- The guard admits only at 10 GiB free (`admission_gib` 10 in the receipts).
- After a refusal the inputs file remains, and the next attempt refuses (:640–641). Notes §5: "Each version gets one attempt."

**Precedent.** 318 checks free space before writing its inputs, for exactly this reason (307 L2): 318 :807–808, with `LAUNCH_FREE_BYTES` = 12 GiB at :183.

**Fix.** Add the same check before :643. For `launch-pictures`, require at least admission plus `check`'s upper estimate.

### L3 (Low). Sheet labels: 944 plan182 frames show only part of a local checkout path

**Where.**
- plan182's reasons are copied first, in order (:366–369).
- A sheet label is the first 80 characters of the first reason (:622–623).

**What I measured.**
- In plan182 (`20a36f91…`), 944 of the 5,341 samples have a first reason that begins with an absolute local checkout path about 228 characters long.
- That path is followed by "original absolute anchor: <row>" (457 samples) or "cue-relative historical anchor…" (487 samples).
- Their labels therefore read "plan182: " plus part of the path, and 434 of the 944 have no other reason.

**Effect.**
- On the sheet itself, root cannot tell which anchor a frame serves.
- The home path is copied into `plan.json`, `sampling.json` and the sheet pixels. That carries the hygiene issue of notes F1 into new outputs.
- Layout reasons also carry tab characters, which are drawn with PIL's default font.

**Fix.**
- Label from the text after " original absolute anchor: ", or from the first reason that is not a path.
- Replace tabs in labels.
- Write reasons without the home prefix, for example as their Q-relative tail.

### L4 (Low). The decode and picture runtime is neither frozen nor exercised before launch

**What 321 does.**
- 321 binds no `ffmpeg`, `ffprobe`, `bash` or Python binary and no Pillow files.
- It records no tool version in `verification.json`.
- `PATH` is `/usr/bin:/bin` (:465).

**What the earlier CSD verifier did.** `launch-final-verify05.py` :62–78 freezes:
- `ffmpeg`, `ffprobe` and `bash`, resolved;
- `sys.executable`;
- Pillow's files and libraries, from `pillow_runtime()`.

That routine (`raster-quality03/continuous-quality16/pictures-after-preview16.py` :6–31, `86227e83…`) also draws text, saves and decodes a PNG, and checks `ldd`, all before anything is spent.

**Effect.**
- The record's "0 error lines" does not say which decoder build produced it.
- A Pillow fault surfaces only inside the spent picture job. A missing import fails at :587, after admission. A drawing fault appears only after the full extraction pass (:603–606).

**Fix.**
- Freeze the four resolved tools and Pillow's files into both inputs files.
- Record the first line of `ffmpeg -version` in `verification.json` and `sampling.json`.
- Exercise PIL once in `launch-pictures` before :643.

## What I checked and found sound

### 1. The 318 output contract (318 at `478b44a5`)

- **The command layout.**
  - Guard receipts record the child argv exactly. I checked 3 local receipts: final08-terminal218, review-build268-terminal272 and review-rebind269-terminal276.
  - So :211–213 matches 318's launch (318 :812–814): 11 items, the helper at index 3 and the manifest at index 10, with the manifest bound at its hash (:222–223).
  - 321's own membership checks (:455) fit its `/usr/bin/timeout` wrapper. The same wrapper is in the rebind269 receipt (`/usr/bin/timeout 172900 …`).
- **Summary and rows.**
  - Version, host, frames, registry, consumer binary and narration all match.
  - So do 15 replays in registry order, `chapter_binaries`, and `boundary_replay` with its capture at the staged 09 soundtrack's source and digest.
  - Each row's kind, binary, host, job, movie digest and `start_frame` match 318 :679–691 and :705–713.
- **Staging and render summaries.**
  - The staged set must be exactly 90 links (15 × 5 tables and movies, plus 15 soundtracks). Each must be the same file (`os.path.samefile`) as a source that `stable()` re-hashed as a bound row (:241–248; 318 :868–884).
  - Each render summary must list every staged file at its digest. 09's must also carry repair310's hash and build313's qualification hash (:258–266; 318 :611–626).
  - Soundtracks must be 48 kHz, 2 channels, 16-bit, with frames × 800 samples. Replay counts must equal the staged cue-log rows (:267–270).
- **09's rebind313 paths.** 09's lines name rebind313 WAVs. 321 compares bytes against rebind269 and assembly19 (:297–302, :309–313), never paths, as 318 does.
- **A narrower acceptance than 318.** 321 accepts 02/03/13/15 only as retained (:87–91, :226–231). 318 would also accept a combined 268 render for those four. This matches the current plan.

### 2. Full decode

- **The strict script.** It always runs a software decode with `-xerror`. When `nvidia-smi` is on `PATH`, it adds a CUDA second opinion: pictures that decode rejects fail the file, while a hardware decode that cannot run is only a note. It also gates container against extension, zero packets, A/V within 0.2 s, BT.709 tags, resolution, fps and loudness.
- **How 321 runs it.**
  - `EXPECT_*` equals the shared module's values (:193).
  - `TMPDIR` is the job's own `decode-logs`, and a retained decode log is required.
  - The last line must be `VERIFY PASS`, and `software_decode_clean` must hold (:488–496).
- **BT.709 tags.**
  - The kit's join writes all three tags: `h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1` (master.rs :243–244).
  - The local 08 chapter render carries only `color_space=bt709`. So `EXPECT_BT709` holds on the master, and is not expected to hold on chapters.
- **framemd5.**
  - It adds two more full decodes, of all 15 chapters and of the master.
  - With ffmpeg 8.1.2 on 2–3 frames of the local 08 render, I confirmed:
    - the row format `frame_hashes()` parses: six cells, stream 0, `#` headers;
    - that `-show_data_hash MD5` prints `extradata_hash` (:521–524).
  - Decoded-frame identity is the right answer to 322 N2 (09's `avcC` is not compared before the join). Any parameter-set difference that changes a picture refuses, and names the frame.
- **`ffmpeg -n`.**
  - Locally, 8.1.2 exits 0 when the output exists, wherever `-n` is placed.
  - 321 does not rely on it. The `framemd5`, `pictures` and `native4k` folders are created fresh with `mkdir()` (:503, :601–602), so no stale output can be read.

### 3. Captions

- **`check_captions`.**
  - It requires SRT = VTT in the kit's exact format, every caption inside the video and in order, and the text equal to the script both as a whole and cue by cue. Each caption goes to the latest cue starting no later than 1 ms after it. The report's sentences must also equal the script.
  - 321 passes the report the captions consumer used (rebind269's), placements from the staged tables, and 133,662 frames (:325).
- **`check_phrases`.** It runs against rebind269's `phrase-timings.json`. rebind269 kept assembly19's file: its summary snapshot counts 106 unchanged files, the 105 WAVs plus that file (307 N8).
- **The caption screen.**
  - Captions over 20 characters per second and lines over 42 characters are flags only. 321 records the counts without gating, and binds the SRT hash (:530–534), as 19-02 did.
  - Prior anchor: the algebra master on CSD had one caption over 20 cps (21.562, `algebra-fixes/verification-terminal14.json`, `39ca69c7…`), which root then read in context. Reading flagged captions here is likewise root's.

### 4. Rates

- **Video.**
  - `media_facts` requires h264 3840×2160 at `r_frame_rate` 60/1, 133,662 counted packets, and video and audio lengths against the total. The strict check requires `EXPECT_FPS` 60.
  - 318's own ffprobe gate also requires `avg_frame_rate` 60/1 and `nb_frames` 133,662 (318 :911–912).
  - Per chapter: the framemd5 count must equal the frames (:512), and each table must be at fps 60 (:295).
- **Audio.** AAC at 48 kHz stereo, the soundtrack sizes and the replay counts, all as in §1.
- **The remaining gap.** Timing at the joins: see L1.

### 5. Loudness

- **The strict script.**
  - `EXPECT_LUFS` is −16 ±1 LU.
  - A quieter master passes only if its true peak sits at the ceiling, −1.6 to −0.95 dBTP, and the script then prints a `note:`.
  - 321 records the notes (:544) and separately gates TP ≤ −1 dBTP from its own ebur128 log (:525–529), as 19-02 and final-verify05 do.
- **Prior anchor.** CSD's algebra master14 measured −18.9 LUFS, LRA 3.8 and TP −1.5 dBTP (`verification-terminal14.json`), and passed through that ceiling path. A complex master at the kit's fixed gain may land the same way.
- **What a fail would mean.** A quiet master whose post-AAC peak falls below −1.6 dBTP would fail the strict check. That is the gate working, not a 321 defect.

### 6. Frame extraction

- **The pass.**
  - One ffmpeg pass, as in 182, with a balanced `eq(n,…)` select. `-vsync 0` is still accepted by 8.1.2.
  - Every picture is size-checked and hashed. Sheets are written `'xb'` and `sampling.json` `'x'`, and every flag is False.
  - It runs only after a successful verification job (`verified()`, :560–568).
- **Simulation on stand-ins.**
  - Result: 5,777 overview samples, 723 sheets and 82 native frames.
  - All 5,341 plan182 samples and all 39 of its native frames are kept.
  - 09's exit window, 2,084–2,102, is fully sampled.
  - No label prefix holds a character above U+00FF.
  - Realistic size is about 1 GiB, inside the 4 GiB phase cap. `check`'s printed upper estimate uses lane maxima and raw RGB.
- **The preview313 records**, as my 313 source writes them:
  - `summary.json` at the job root and `09_boundary/sampling.json`;
  - `zoom_probe_exit_frame` and `zoom_probe_fails`;
  - picks with a chapter-local `frame` and `reasons`;
  - `fades` with `id`, `how`, `start`, `frames` and `repeat`;
  - `search`, present for fitted sites and for sites that follow an unlocalized one;
  - `flag`, set only by the exact-site cross-check (preview313 :735).

  321 reads them that way (:401–413, :426–435).
- **Pillow.** Notes §6 lists Pillow under CSD's `/usr/bin/python3` as a deploy prerequisite. I have no local evidence either way. See L4.

### 7. Previous anchors

- **Frame totals.** The film has 133,662 frames. I parsed all 15 local timeline21 tables with the shared `parse_timeline`: 105 lines, and 09 starts at 67,065 with 16,504 frames. The chapter starts and lengths equal plan182's, which 321 asserts (:355–356).
- **Cues and replays.** 105 cues in script order and 15 bit-identical replays.
- **`PREVIOUS_LAYOUT`.**
  - All 18 listed sources are at their hashes.
  - The 30 rows equal the distinct non-empty rows of those sources, checked both ways.
  - No other layout snapshot under Q adds a row.
  - `anchors()` accepts all 49 local layout rows.
- **09.**
  - `EXIT_WINDOW` 2,084–2,102 covers the probe's enforced 2,086–2,100 with ±2 frames, and the label now says so (:387–388; review320 L1).
  - `ABSENT_ANCHORS` equals preview313 :841–849.
  - Persistence covers 39–133 s, including root's 305 grid at 115–117 s.
- **Not covered:** root's 305 and 319 windows. See M1.

## Notes (no change asked)

- **N1. 321 is bound to 318 by name**: the helper, job, inputs, `version` '318' and the 11-item command. If root acts on 322 F1 with a new master version, 321 needs a matching version. Notes §4 says the 318 keys must be re-read if 322 changes 318.
- **N2. 321 depends on 318's preflight file in `D/work/tmp`.** `stable()` re-hashes every 318 input, including that `preflight.json` (322 N1). Anything that clears that folder before 321 launches makes 321 refuse; it cannot pass.
- **N3. `verified()` reads `strict-video.log` without `errors='replace'`** (:566), unlike `verify()` (:492). This is harmless while the log is UTF-8.

## Local checks (outside the repo; not host runs)

**Code reading.** I read 321 at `2dc51c9d` line by line and diffed it against `0a956ec8`.

**Python.** Read-only, with `python3 -I -B`, loading 321's own functions from its file:
- the anchor and source check;
- the picture-plan simulation, on these stand-ins:
  - the local timeline21 tables, for chapters and line starts;
  - the 30 previous rows, as current layouts;
  - preview274's 09 sampling (100 picks, 2 sites), as preview313;
  - a zoom exit at 2,093;
- the window coverage;
- `media_facts` with zero, one and two extra frames.

**ffmpeg and ffprobe 8.1.2** on the local 08 chapter render: framemd5 of 2–3 frames, `-show_data_hash`, `-n` twice, and `-vsync 0` on one frame. Outputs were written only outside the repo.

**Limits.**
- No real 318 or preview313 records exist yet, and the stand-ins are not the final layouts.
- I do not know the ffmpeg or Pillow versions on CSD.
- Nothing ran on CSD.

## Still open after a successful 321

- The 318 master itself, then 321's `check`, `verify` and `pictures` runs.
- Root's viewing of every sheet and native frame.
- Motion.
- Joins, seen and heard.
- Fine sync.
- Audio review.
- Independent final AV review.
- Public 4K and uploaded-caption verification.
- Publication.

No audio heard or checked, here or by any check above.
