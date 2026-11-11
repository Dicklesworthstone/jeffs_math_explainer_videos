# recovery1000: complex release review (verify995 coverage, publication965 against master993)

Reviewer: RubyRaven (CC2), 2026-11-10, read-only. No helper, source, TODO, Beads or frozen input was edited. No job was
launched and no verification was repeated. Remote reads were `ssh csd` reads of exact paths only.

Q is `production-repair17` (local repo path `docs/video-program/third-wave-20261024/opus-replacements-20261104/
fmd-render-readiness01/raster-quality03/production-repair17`; on CSD `/data/projects/jmev-stroke-probe-20261106/
qualification01/production-repair17`). Job directories on CSD are under `/data/projects/jmev-stroke-probe-20261106/jobs`.

## Verdict

No defect found in the master993 chapter starts, the publication965 chapter marks, or either caption set.

There is one stale record and one open decision:

- **S1, stale record.** `publication965/review965.json` line 22 still says "Must bind actual successful874 master,
  captions, thumbnail and final chapter starts before upload", and line 20 says `final_master_times_verified: false`.
  The chapter starts now match master993 exactly (section 2). The record has not been updated to say so. The title,
  tags and description text need no change.
- **D1, caption file to upload.** 995 and root's adoption record 354 treat cc1-caption-repair347 (579 blocks) as the
  delivery captions and the master's native 571-block captions as the proof. The TODO entry at 17:46 calls 571 "the
  993 delivery count". Both sets are correct (section 3). Recommendation: upload 347, because it has no block shorter
  than 5/6 s and native 571 has three. Whichever set is chosen, the record should name it.

Live job: verify995 is still running (section 5). Its terminal result and the picture jobs after it are pending.

## 1. Inputs (sha256)

| File (in Q unless noted) | sha256 |
|---|---|
| completed-master993-summary.json | 38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185 |
| complex-analysis-master993.mp4 (local copy, 219661850 bytes) | 80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90 |
| complex-analysis-master993.en.srt (= native327, = CSD master SRT) | 5d06423248f088a34774f361721bf05e1c4950ac834086e3666661e1adfe5e1b |
| complex-analysis-master993.en.vtt | 26e0d7e0eec26eb6f9136bb56f795a6ff8ab9cead26f29fc76a75e9abc4bb2e3 |
| cc1-caption-repair347-candidate.en.srt (same bytes on CSD) | 0ea9277e6d598540e5e9057a84ee84daf7f88420b076434659fc3d3ad0ff3230 |
| cc1-caption-repair347-candidate.en.vtt | 6a383cb7f207be81254e9a820ecb701ca77d00a26557194c675dc1717ef6979b |
| root-caption347-review354.json | 749aeba8350d46e339a517af971086bd2d6e2fe039e6501f16e480edd3ad3cb4 |
| publication-chapters-checked993.json | 6ef775a5255d9ad96e022e672b80417f37762efb9b5cb3d8fe3da41a214a7ad9 |
| publication965/description.txt | c78723f3dffe6476cf68afa56edb1b4d0cae372f44fc7d9596bd607a87f2fd0b |
| publication965/title.txt | 48a242e3e4209b446850be61f1868d8f6280881d00d34d2bc387e529c0edc920 |
| publication965/tags.txt | 40e49939e363bb750de3884b2c8b0ed3664303887829417622e5ac79d52594a8 |
| publication965/review965.json | 5469adeb78c675a5ca7864d162c20226c253ab7abefb5c0598b2b5d85d52129f |
| combined-master993.py | 1955ca3202c8a4f382fea1682ac754eadf19fd708b2ef983c3f91f12607b6edb |
| csd-final-verify995.py | 69fa3ded0960f6344f3cbad557caf951a769dfe7e0d57426a614ea8d7dd2b7a2 |
| csd-final-after-master995.py | 9a055c264f47649dcac7e76fae82002ce21e79a2f6a81488bd3a845cf6e7daeb |

The shared helpers on CSD match the pins in 995: verify_stage09_final.py f266a231..., verify_video_preserve_logs01.sh
3f4c68db..., caption-interval-audit01.py 3cc26436..., final-verify995-inputs.json fa2367ad.... The script.tsv files of
build268 and build313 on CSD are both 234f7257..., the same bytes as the local root-boundary552 script.

## 2. Chapter timestamps

Chapter starts are the cumulative `rows[].frames` of the master993 summary at 60 fps. The rows sum to 133662 frames,
2227.700 s (37:07.700).

All 15 chapters agree three ways: the summary rows, publication-chapters-checked993.json (chapter, start frame,
timestamp, title) and the `Chapters:` block of publication965/description.txt. YouTube's rules hold: the first mark is
0:00, the marks ascend, and the shortest chapter is 65.7 s, well over 10 s.

Each mark is the start time rounded down to the second, so a viewer who jumps lands this early:
0.000, 0.183, 0.717, 0.533, 0.433, 0.083, 0.250, 0.433, 0.750, 0.817, 0.850, 0.717, 0.433, 0.783, 0.717 s.
At most that is 0.85 s. That time falls in the previous chapter's silent tail: the last caption before every chapter
start ends at least 3.2 s before it, and each chapter's first caption starts 0.600 s after its start.

## 3. Captions

Native 571 (the master's own SRT/VTT):

- SRT and VTT have equal times (to 0.5 ms) and equal text.
- Captions are in order with no overlap. The last one ends at 2220.147 s, inside the master.
- No caption crosses a chapter start.
- The whole text equals the script's whole text.
- caption-interval-audit01 screen: 571 blocks, 0 over 20 characters per second, 0 lines over 42 characters.

347 (579 blocks):

- The whole text equals the script's, and its word stream equals native 571's.
- Each chapter's first caption start and last caption end are the same as in 571.
- No caption crosses a chapter start.
- Screen: 579 blocks, 0 over 20 characters per second, 0 lines over 42 characters.

Differences between the two sets:

- The fastest block is 19.23 characters per second in both.
- 347 has no block shorter than 5/6 s.
- Native 571 has three: block 420 "One." (0.66 s), block 63 "No." (0.70 s) and block 385 "No." (0.817 s).

This is the basis for recommending 347 in D1.

## 4. What verify995 covers, and what it does not

I read the code at 69fa3ded. It covers:

- strict decode with EXPECT gates;
- the ffprobe media facts;
- framemd5 identity between the master and the 15 staged chapter movies (timebase 1/60, pts 0..n-1);
- ebur128 loudness, with true peak at or below -1;
- the caption screens of the master SRT and the 347 SRT;
- the film gates: master350 rows, binaries and hard links, schedules against timeline269 and timeline21, 105
  placements, takes byte-identical, `check_captions`, delivery347 (master SRT/VTT equal native327, 347 exact, 354
  pinned) and `check_phrases`.

Gaps. None of them is a defect in the master. They are limits on what a PASS will mean:

- **G1.** The master's audio is not compared with the 15 chapter soundtracks. Decode, duration and loudness are checked,
  and the joins are not measured for clicks.
- **G2.** The hardware second-opinion decode did not run on CSD. strict-video.log has no hwaccel line, so only the
  software decode ran.
- **G3.** The native SRT's screen counts are recorded but not asserted. They are 0 and 0 here, so nothing is hidden.
- **G4.** Extradata MD5s are recorded, not compared. framemd5 identity already covers the picture at the joins.
- **G5.** Nothing binds the thumbnail or the publication copy (publication965) to the master. That binding is
  root-chapters993 plus this review.
- **G6.** The 995 picture plan puts no native frame inside the chapter 09 B1 window (71 to 85 s). The natives are at
  4180 and 5417. The window is covered by 86 of final969's 814 samples, which are carried in through framemd5 identity,
  and by the two crops in section 6.
- **G7.** Loudness is -19.0 LUFS under the fixed-gain ceiling exception. YouTube will not raise it.

## 5. Live verify995 status (read at 18:37:19Z)

From jobs/csd-final-verify995:

- **strict-video.log: VERIFY PASS.**
  - h264 3840x2160 with bt709 tags, 60/1 fps, 133662 frames, AAC 48 kHz.
  - A/V delta 0.000 s.
  - I -19.0 LUFS, LRA 4.1, peak -1.5 dBFS, with the ceiling note.
  - Full software decode in 151 s with 0 error lines.
- **framemd5.** 01 to 13 are complete; 13_real_integral was written at 18:34Z. 14_continuation is in progress. 15 and
  the master's own framemd5 have not started.
- **Still pending.** The identity comparison, ebur128, the caption screens, the film gates and verification.json.
- **resource-receipt.json.** Still being updated. Pid 2385942, started 17:47:35Z.
- **Picture jobs.** No `csd-final-pictures995-*` job directory exists yet. The controller launches them after
  verify995 succeeds.

## 6. Chapter 09 B1 window, two native crops from the local master993

09 starts at 1117.750 s in the master. I extracted two frames:

- 1189.750 s (09 local 72 s, clockwise heads);
- 1196.750 s (79 s, counterclockwise heads).

Each frame was cropped 800x800 at (2708, 937), the 1:1 view of the close-up centred on (3108, 1336.5). Each was also
scaled to 1080p and cropped 400x400 at the same place.

What I saw:

- At both times and at both scales, the eight yellow inner arrowheads, with their dark rims, stand apart from the green
  ring. Their directions read: clockwise at 72 s, counterclockwise at 79 s.
- `p` sits in the clear band between the green ring and the grey dashed circle and touches neither.
- `a`, `P` and the outer arrowheads are unobstructed.
- At 1080p the inner heads are small, about 10 px, but distinct.

These are two still frames. They are not a motion review.

Crops (scratchpad, preserved): r1000/b1-72-native.png, b1-72-1080p.png, b1-79-native.png, b1-79-1080p.png.

## 7. Not established here

- verify995's terminal result.
- The 995 picture extraction and its reading.
- Listening, motion, fine sync, AV acceptance, the thumbnail and release.
- Final969's root and independent reviews read downscaled contact sheets, not every frame at full 4K. This review adds
  two full-resolution stills and nothing more.

## 8. For root

1. Record D1: which SRT/VTT is uploaded. 347 is recommended. If 571 is chosen, say that 354's adoption is superseded.
2. Bring S1 up to date: review965's scope sentence names 874. The times are now verified against master993 (this
   review, and publication-chapters-checked993 6ef775a5).
3. Let verify995 and the picture jobs finish. Nothing in this review asks for a re-run or a helper change.
4. If G1 or G2 matters before release, a separate read-only check can cover it later. I did not run one.

Evidence scripts (scratchpad, preserved): r1000_times.py, r1000_captions.py, r1000-screen-master993.json,
r1000-screen-347.json, r1000/ crops.
